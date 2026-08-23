//! Integration T16 — API + SDK + metadata end-to-end.

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;
use trust_escrow_api::{
    app_with_state,
    integration::{
        create_job_integration, derive_job_pda_string, get_job_enriched, list_jobs_enriched,
    },
    models::CreateJobRequest,
    state::AppState,
};

fn future_deadline() -> i64 {
    chrono::Utc::now().timestamp() + 3600
}

#[tokio::test]
async fn http_create_list_enrichment() {
    let state = AppState::default();
    let app = app_with_state(state.clone());

    let payload = serde_json::json!({
        "title": "Integration job",
        "description": "via SDK + repo integration",
        "amount": 2_000_000,
        "deadline": future_deadline()
    });

    let (pk, sig, msg) = {
        use base64::Engine as _;
        use ed25519_dalek::{Signer, SigningKey};
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let pk = bs58::encode(sk.verifying_key().to_bytes()).into_string();
        let m = "http-create";
        let s = base64::engine::general_purpose::STANDARD.encode(sk.sign(m.as_bytes()).to_bytes());
        (pk, s, m.to_string())
    };

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method(Method::POST)
                .header("content-type", "application/json")
                .header("x-pubkey", pk)
                .header("x-signature", sig)
                .header("x-message", msg)
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = to_bytes(resp.into_body(), 16384).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["title"], "Integration job");
    assert_eq!(v["description"], "via SDK + repo integration");

    let enriched = get_job_enriched(&state, 0).await.unwrap();
    assert_eq!(enriched.title, "Integration job");
    assert_eq!(enriched.description, "via SDK + repo integration");

    let list_int = list_jobs_enriched(&state).await.unwrap();
    assert_eq!(list_int.len(), 1);
    assert_eq!(list_int[0].title, "Integration job");

    let resp = app
        .clone()
        .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = to_bytes(resp.into_body(), 16384).await.unwrap();
    let list_http: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list_http.len(), 1);
    assert_eq!(list_http[0]["title"], "Integration job");
}

#[tokio::test]
async fn integration_service_multiple_jobs_and_list_order() {
    let state = AppState::default();
    for i in 0..5u64 {
        let req = CreateJobRequest {
            title: format!("Job {}", i),
            description: format!("desc {}", i),
            amount: 1_000_000 + i * 1000,
            deadline: future_deadline(),
        };
        let resp = create_job_integration(&state, req).await.unwrap();
        assert_eq!(resp.job_id, i);
    }
    let list = list_jobs_enriched(&state).await.unwrap();
    assert_eq!(list.len(), 5);
    for i in 0..5 {
        assert!(list.iter().any(|j| j.title == format!("Job {}", i)));
    }
}

#[tokio::test]
async fn validation_blocks_and_no_repo_side_effect() {
    let state = AppState::default();
    let bad = CreateJobRequest {
        title: "".into(),
        description: "desc".into(),
        amount: 1000,
        deadline: future_deadline(),
    };
    let err = create_job_integration(&state, bad)
        .await
        .expect_err("must fail");
    assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    assert!(list_jobs_enriched(&state).await.unwrap().is_empty());

    let app = app_with_state(state.clone());
    let (pk2, sig2, msg2) = {
        use base64::Engine as _;
        use ed25519_dalek::{Signer, SigningKey};
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let pk = bs58::encode(sk.verifying_key().to_bytes()).into_string();
        let m = "bad-job";
        let s = base64::engine::general_purpose::STANDARD.encode(sk.sign(m.as_bytes()).to_bytes());
        (pk, s, m.to_string())
    };
    let payload = serde_json::json!({"title":"","description":"desc","amount":0,"deadline":0});
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/jobs")
                .method(Method::POST)
                .header("content-type", "application/json")
                .header("x-pubkey", pk2)
                .header("x-signature", sig2)
                .header("x-message", msg2)
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert!(list_jobs_enriched(&state).await.unwrap().is_empty());
}

#[tokio::test]
async fn derive_pda_string_matches_routes_and_is_validation_valid() {
    let pda = derive_job_pda_string(42);
    let dl = chrono::Utc::now().timestamp() + 86400;
    let client = format!("7a2YhCd7iivXfyySkp1pf5jjClient{:0>20}{:02}", 1u8, 1u8);
    let m =
        trust_escrow_api::metadata::JobMetadata::new(pda.clone(), "t".into(), "d".into(), 1_000_000, 25_000, dl, client).unwrap();
    assert_eq!(m.pda_address, pda);
    assert!(pda.starts_with("7a2Y"));
}

// ---------------------------------------------------------------------------
// Validator-gated integration (only with --features solana and validator UP)
// Uses manual Runtime to avoid `can call blocking` inside `#[tokio::test]`.
// ---------------------------------------------------------------------------

#[cfg(feature = "solana")]
mod solana_integration {
    use super::*;
    use anchor_client::Cluster;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::{
        commitment_config::CommitmentConfig, signature::read_keypair_file, signer::Signer,
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use trust_escrow_api::{
        integration::{create_job_full_flow, list_jobs_full_flow},
        state::AppState,
    };
    use trust_escrow_sdk::client::TrustEscrowClient;

    const RPC_URL: &str = "http://127.0.0.1:8899";
    const KEYPAIR_PATH: &str = "~/.config/solana/id.json";

    fn rpc() -> RpcClient {
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed())
    }
    fn validator_available() -> bool {
        rpc().get_health().is_ok()
    }
    fn expand(p: &str) -> String {
        if let Some(r) = p.strip_prefix("~/") {
            format!("{}/{}", std::env::var("HOME").unwrap_or_default(), r)
        } else {
            p.to_string()
        }
    }

    #[test]
    fn validator_create_job_via_sdk_and_metadata_and_list() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { inner().await });
        async fn inner() {
            if !validator_available() {
                eprintln!("validator not available at {}, skipping", RPC_URL);
                return;
            }
            let kp = match read_keypair_file(expand(KEYPAIR_PATH)) {
                Ok(kp) => kp,
                Err(_) => {
                    eprintln!("no keypair at {}, skipping", KEYPAIR_PATH);
                    return;
                }
            };
            let client = match TrustEscrowClient::new(Cluster::Localnet, kp) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("client build failed: {}, skipping", e);
                    return;
                }
            };
            let state = AppState::default();
            let job_id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                % 1_000_000
                + 300_000;
            let req = CreateJobRequest {
                title: "Validator job".into(),
                description: "created via SDK + stored off-chain".into(),
                amount: 500_000,
                deadline: chrono::Utc::now().timestamp() + 3600,
            };
            let (sig, resp) = create_job_full_flow(&client, &state, job_id, req)
                .await
                .expect("full flow");
            assert!(!sig.to_string().is_empty());
            assert_eq!(resp.title, "Validator job");

            let list = list_jobs_enriched(&state).await.unwrap();
            assert!(list.iter().any(|j| j.title == "Validator job"));

            let payer = read_keypair_file(expand(KEYPAIR_PATH)).unwrap();
            let job = client
                .get_job(&payer.pubkey(), job_id)
                .unwrap()
                .expect("job on-chain");
            assert_eq!(job.amount, 500_000);

            let enriched = list_jobs_full_flow(&client, &state, Some(payer.pubkey()))
                .await
                .unwrap();
            assert!(!enriched.is_empty());

            tokio::task::block_in_place(|| drop(client));
        }
    }

    #[test]
    fn validator_invalid_body_never_hits_chain() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { inner().await });
        async fn inner() {
            if !validator_available() {
                eprintln!("validator not available, skipping");
                return;
            }
            let kp = match read_keypair_file(expand(KEYPAIR_PATH)) {
                Ok(kp) => kp,
                Err(_) => {
                    eprintln!("no keypair, skipping");
                    return;
                }
            };
            let client = match TrustEscrowClient::new(Cluster::Localnet, kp) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("client build failed: {}, skipping", e);
                    return;
                }
            };
            let state = AppState::default();
            let bad = CreateJobRequest {
                title: "".into(),
                description: "desc".into(),
                amount: 500_000,
                deadline: chrono::Utc::now().timestamp() + 3600,
            };
            let err = create_job_full_flow(&client, &state, 999_998, bad)
                .await
                .expect_err("must fail");
            assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
            tokio::task::block_in_place(|| drop(client));
        }
    }
}

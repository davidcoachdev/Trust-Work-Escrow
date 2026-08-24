use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatedJob {
    pub job_id: u64,
    pub client: String,
    pub title: String,
    pub description: String,
    pub amount: u64,
    pub deadline: i64,
    pub transaction_signature: Option<String>,
    pub job_pda: Option<String>,
    pub on_chain_status: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnsignedJobTransaction {
    pub job_id: u64,
    pub signer: String,
    pub transaction: String,
    pub job_pda: String,
    pub cluster: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RelayedJobTransaction {
    pub signature: String,
    pub cluster: String,
}

/// Ask the backend SDK for a transaction template. The caller supplies the
/// wallet authentication proof; only the browser/Phantom can sign the bytes.
#[server]
pub async fn request_create_job_transaction(
    signer: String,
    auth_signature: String,
    auth_message: String,
    amount: u64,
    deadline: i64,
) -> Result<UnsignedJobTransaction, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let base =
            std::env::var("API_INTERNAL_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
        let token = std::env::var("API_SERVICE_TOKEN").ok();
        let mut request = reqwest::Client::new()
            .post(format!(
                "{}/jobs/transactions/create-unsigned",
                base.trim_end_matches('/')
            ))
            .header("x-pubkey", &signer)
            .header("x-signature", &auth_signature)
            .header("x-message", &auth_message)
            .json(&serde_json::json!({"signer": signer, "amount": amount, "deadline": deadline}));
        if let Some(token) = token {
            request = request.header("x-service-token", token);
        }
        let response = request
            .send()
            .await
            .map_err(|e| ServerFnError::new(format!("backend unavailable: {e}")))?;
        if !response.status().is_success() {
            return Err(ServerFnError::new(format!(
                "backend rejected unsigned transaction ({})",
                response.status()
            )));
        }
        response
            .json()
            .await
            .map_err(|e| ServerFnError::new(format!("invalid backend response: {e}")))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (signer, auth_signature, auth_message, amount, deadline);
        Err(ServerFnError::new("server only"))
    }
}

/// Relay bytes after Phantom has signed them. The server function never sees a
/// private key and delegates validation/transmission to the backend SDK.
#[server]
pub async fn relay_signed_job_transaction(
    signer: String,
    auth_signature: String,
    auth_message: String,
    transaction: String,
) -> Result<RelayedJobTransaction, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let base =
            std::env::var("API_INTERNAL_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
        let mut request = reqwest::Client::new()
            .post(format!(
                "{}/jobs/transactions/relay",
                base.trim_end_matches('/')
            ))
            .header("x-pubkey", &signer)
            .header("x-signature", &auth_signature)
            .header("x-message", &auth_message)
            .json(&serde_json::json!({"signer": signer, "transaction": transaction}));
        if let Ok(token) = std::env::var("API_SERVICE_TOKEN") {
            request = request.header("x-service-token", token);
        }
        let response = request
            .send()
            .await
            .map_err(|e| ServerFnError::new(format!("backend unavailable: {e}")))?;
        if !response.status().is_success() {
            return Err(ServerFnError::new(format!(
                "backend rejected signed transaction ({})",
                response.status()
            )));
        }
        response
            .json()
            .await
            .map_err(|e| ServerFnError::new(format!("invalid backend response: {e}")))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (signer, auth_signature, auth_message, transaction);
        Err(ServerFnError::new("server only"))
    }
}

/// Server-only proxy: the browser never receives the SDK or a signing key.
#[server]
pub async fn create_job(
    title: String,
    description: String,
    amount: u64,
    deadline: i64,
) -> Result<CreatedJob, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let base =
            std::env::var("API_INTERNAL_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
        let token = std::env::var("API_SERVICE_TOKEN")
            .map_err(|_| ServerFnError::new("API_SERVICE_TOKEN is not configured"))?;
        let response = reqwest::Client::new()
            .post(format!("{}/jobs", base.trim_end_matches('/')))
            .header("x-service-token", token)
            .json(&serde_json::json!({"title": title, "description": description, "amount": amount, "deadline": deadline}))
            .send().await
            .map_err(|e| ServerFnError::new(format!("backend unavailable: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            return Err(ServerFnError::new(format!(
                "backend rejected create job ({status})"
            )));
        }
        response
            .json::<CreatedJob>()
            .await
            .map_err(|e| ServerFnError::new(format!("invalid backend response: {e}")))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (title, description, amount, deadline);
        Err(ServerFnError::new("server only"))
    }
}

use std::sync::{Arc, Mutex};

use trust_escrow_api::{
    application::{ApplicationService, CreateJobRequest, SdkBoundary},
    intents::{IntentStore, IntentStatus},
    projection::{AccountKey, ProjectionStore, ProjectionStatus, Tombstone},
    signer::{SignerMode, SignerPolicy},
    Finality,
};

#[test]
fn signer_policy_requires_explicit_mode_and_expected_signer() {
    let policy = SignerPolicy::new(SignerMode::UserSigned, "subject-1", "job.client");
    assert!(policy.validate(Some("subject-1")).is_ok());
    assert!(policy.validate(Some("other")).is_err());
    assert!(policy.validate(None).is_err());
}

#[test]
fn finality_transitions_reject_regressions_and_unsupported_finalized_claims() {
    assert!(Finality::Intent.can_transition_to(Finality::Submitted));
    assert!(Finality::Submitted.can_transition_to(Finality::Confirmed));
    assert!(Finality::Confirmed.can_transition_to(Finality::Finalized));
    assert!(!Finality::Finalized.can_transition_to(Finality::Confirmed));
    assert!(!Finality::Intent.can_transition_to(Finality::Finalized));
}

#[test]
fn intent_store_is_idempotent_and_prevents_double_terminal_application() {
    let store = IntentStore::default();
    let first = store.create("same-key", "create_job").unwrap();
    let second = store.create("same-key", "create_job").unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(store.len(), 1);
    store.transition(&first.id, IntentStatus::Submitted).unwrap();
    store.transition(&first.id, IntentStatus::Confirmed).unwrap();
    assert!(store.transition(&first.id, IntentStatus::Submitted).is_err());
}

#[test]
fn projection_isolated_by_cluster_and_tombstone_blocks_stale_resurrection() {
    let store = ProjectionStore::default();
    let key = AccountKey::new("localnet", "program", "account").unwrap();
    store.upsert(key.clone(), 10, ProjectionStatus::Current).unwrap();
    store.tombstone(Tombstone::new(key.clone(), 11, Finality::Finalized)).unwrap();
    assert!(store.upsert(key, 10, ProjectionStatus::Current).is_err());
    assert_eq!(store.len(), 0);
    assert!(store.has_tombstone(&AccountKey::new("localnet", "program", "account").unwrap()));
}

#[test]
fn application_service_delegates_without_constructing_rpc_transactions() {
    #[derive(Default)]
    struct FakeSdk(Arc<Mutex<Vec<String>>>);
    impl SdkBoundary for FakeSdk {
        fn create_job(&self, request: &CreateJobRequest) -> Result<String, String> {
            self.0.lock().unwrap().push(request.title.clone());
            Ok("signature-1".into())
        }
    }
    let calls = Arc::new(Mutex::new(Vec::new()));
    let service = ApplicationService::new(FakeSdk(calls.clone()));
    let result = service.create_job(CreateJobRequest {
        title: "deterministic fixture".into(),
        description: "offline only".into(),
        amount: 100,
    }).unwrap();
    assert_eq!(result.signature, "signature-1");
    assert_eq!(calls.lock().unwrap().as_slice(), &["deterministic fixture"]);
}

//! Repository layer for off-chain metadata.
//!
//! Defines an async `MetadataRepository` trait covering all off-chain entities
//! (jobs, applications, milestones, disputes, support tickets, evidence) plus an
//! in-memory implementation suitable for unit tests. Postgres/Mongo backends
//! will implement the same trait once Docker is available — handlers depend only
//! on `dyn MetadataRepository` so no API change is needed.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::metadata::{
    ApplicationMetadata, DisputeMetadata, EvidenceMetadata, JobMetadata, MilestoneMetadata,
    SupportTicketMetadata, UserMetadata, ValidationError,
};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors returned by the repository.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RepositoryError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("storage error: {0}")]
    Storage(String),
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Async repository for all off-chain metadata entities.
///
/// Each entity is addressed by its PDA (or composite key `job_pda + index` for
/// milestones/evidence). Implementations must be `Send + Sync` so they can be
/// shared via `Arc<dyn MetadataRepository>` in `AppState`.
#[async_trait::async_trait]
pub trait MetadataRepository: Send + Sync {
    // ---- Jobs ----
    async fn create_job(&self, job: JobMetadata) -> Result<JobMetadata, RepositoryError>;
    async fn get_job(&self, pda_address: &str) -> Result<Option<JobMetadata>, RepositoryError>;
    async fn update_job(&self, job: JobMetadata) -> Result<JobMetadata, RepositoryError>;
    async fn delete_job(&self, pda_address: &str) -> Result<(), RepositoryError>;
    async fn list_jobs(&self) -> Result<Vec<JobMetadata>, RepositoryError>;

    // ---- Applications ----
    async fn create_application(
        &self,
        app: ApplicationMetadata,
    ) -> Result<ApplicationMetadata, RepositoryError>;
    async fn get_application(
        &self,
        application_pda: &str,
    ) -> Result<Option<ApplicationMetadata>, RepositoryError>;
    async fn list_applications_by_job(
        &self,
        job_pda: &str,
    ) -> Result<Vec<ApplicationMetadata>, RepositoryError>;
    async fn delete_application(&self, application_pda: &str) -> Result<(), RepositoryError>;

    // ---- Milestones ----
    async fn create_milestone(
        &self,
        ms: MilestoneMetadata,
    ) -> Result<MilestoneMetadata, RepositoryError>;
    async fn get_milestone(
        &self,
        job_pda: &str,
        index: u8,
    ) -> Result<Option<MilestoneMetadata>, RepositoryError>;
    async fn list_milestones_by_job(
        &self,
        job_pda: &str,
    ) -> Result<Vec<MilestoneMetadata>, RepositoryError>;
    async fn delete_milestone(&self, job_pda: &str, index: u8) -> Result<(), RepositoryError>;

    // ---- Disputes ----
    async fn create_dispute(
        &self,
        dispute: DisputeMetadata,
    ) -> Result<DisputeMetadata, RepositoryError>;
    async fn get_dispute(
        &self,
        dispute_pda: &str,
    ) -> Result<Option<DisputeMetadata>, RepositoryError>;
    async fn update_dispute(
        &self,
        dispute: DisputeMetadata,
    ) -> Result<DisputeMetadata, RepositoryError>;
    async fn delete_dispute(&self, dispute_pda: &str) -> Result<(), RepositoryError>;

    // ---- Support tickets ----
    async fn create_support_ticket(
        &self,
        ticket: SupportTicketMetadata,
    ) -> Result<SupportTicketMetadata, RepositoryError>;
    async fn get_support_ticket(
        &self,
        ticket_pda: &str,
    ) -> Result<Option<SupportTicketMetadata>, RepositoryError>;
    async fn update_support_ticket(
        &self,
        ticket: SupportTicketMetadata,
    ) -> Result<SupportTicketMetadata, RepositoryError>;
    async fn delete_support_ticket(&self, ticket_pda: &str) -> Result<(), RepositoryError>;

    // ---- Evidence (Mongo) ----
    async fn create_evidence(
        &self,
        evidence: EvidenceMetadata,
    ) -> Result<EvidenceMetadata, RepositoryError>;
    async fn get_evidence(
        &self,
        dispute_pda: &str,
        index: u8,
    ) -> Result<Option<EvidenceMetadata>, RepositoryError>;
    async fn list_evidence_by_dispute(
        &self,
        dispute_pda: &str,
    ) -> Result<Vec<EvidenceMetadata>, RepositoryError>;
    async fn delete_evidence(&self, dispute_pda: &str, index: u8) -> Result<(), RepositoryError>;

    // ---- Users (Postgres `users`) ----
    async fn upsert_user(&self, user: UserMetadata) -> Result<UserMetadata, RepositoryError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserMetadata>, RepositoryError>;
    async fn update_wallet(
        &self,
        email: &str,
        wallet_pubkey: Option<String>,
    ) -> Result<UserMetadata, RepositoryError>;
    async fn clear_wallet(&self, email: &str) -> Result<UserMetadata, RepositoryError>;

    // ---- Wallets (user_wallets 1..N) ----
    async fn add_wallet(&self, wallet: crate::metadata::UserWallet) -> Result<crate::metadata::UserWallet, RepositoryError>;
    async fn list_wallets_by_email(&self, email: &str) -> Result<Vec<crate::metadata::UserWallet>, RepositoryError>;
    async fn get_wallet(&self, email: &str, pubkey: &str) -> Result<Option<crate::metadata::UserWallet>, RepositoryError>;
    async fn remove_wallet(&self, email: &str, pubkey: &str, actor: &str) -> Result<(), RepositoryError>;
    async fn get_wallet_for_purpose(&self, email: &str, purpose: crate::metadata::WalletPurpose) -> Result<Option<crate::metadata::UserWallet>, RepositoryError>;

    // ---- Job participants (per-job authority) ----
    async fn add_participant(&self, p: crate::metadata::JobParticipant) -> Result<crate::metadata::JobParticipant, RepositoryError>;
    async fn get_participant(&self, job_pda: &str, email: &str) -> Result<Option<crate::metadata::JobParticipant>, RepositoryError>;
    async fn list_participants_by_job(&self, job_pda: &str) -> Result<Vec<crate::metadata::JobParticipant>, RepositoryError>;
    async fn list_participants_by_email(&self, email: &str) -> Result<Vec<crate::metadata::JobParticipant>, RepositoryError>;
    async fn find_wallet_by_pubkey(&self, pubkey: &str) -> Result<Option<crate::metadata::UserWallet>, RepositoryError>;
}

// ---------------------------------------------------------------------------
// In-memory implementation
// ---------------------------------------------------------------------------

/// In-memory `MetadataRepository` for unit / integration tests.
///
/// All maps are protected by `RwLock` so the repo can be shared across tasks.
/// Validation is performed on `create`/`update` via `validate()`. Duplicate
/// primary keys return `AlreadyExists`; missing keys return `NotFound` on
/// update/delete where appropriate (gets return `None` instead).
#[derive(Debug, Default)]
pub struct InMemoryMetadataRepository {
    jobs: RwLock<HashMap<String, JobMetadata>>,
    applications: RwLock<HashMap<String, ApplicationMetadata>>,
    milestones: RwLock<HashMap<(String, u8), MilestoneMetadata>>,
    disputes: RwLock<HashMap<String, DisputeMetadata>>,
    support_tickets: RwLock<HashMap<String, SupportTicketMetadata>>,
    evidence: RwLock<HashMap<(String, u8), EvidenceMetadata>>,
    users: RwLock<HashMap<String, UserMetadata>>,
    wallets: RwLock<HashMap<(String, String), crate::metadata::UserWallet>>,
    participants: RwLock<HashMap<(String, String), crate::metadata::JobParticipant>>,
}

impl InMemoryMetadataRepository {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience: wrap in `Arc` for sharing with `AppState`.
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

#[async_trait::async_trait]
impl MetadataRepository for InMemoryMetadataRepository {
    // ---- Jobs ----
    async fn create_job(&self, job: JobMetadata) -> Result<JobMetadata, RepositoryError> {
        job.validate()?;
        let mut map = self.jobs.write().await;
        if map.contains_key(&job.pda_address) {
            return Err(RepositoryError::AlreadyExists(job.pda_address.clone()));
        }
        map.insert(job.pda_address.clone(), job.clone());
        Ok(job)
    }

    async fn get_job(&self, pda_address: &str) -> Result<Option<JobMetadata>, RepositoryError> {
        let map = self.jobs.read().await;
        Ok(map.get(pda_address).cloned())
    }

    async fn update_job(&self, job: JobMetadata) -> Result<JobMetadata, RepositoryError> {
        job.validate()?;
        let mut map = self.jobs.write().await;
        if !map.contains_key(&job.pda_address) {
            return Err(RepositoryError::NotFound(job.pda_address.clone()));
        }
        let mut j = job.clone();
        j.updated_at = chrono::Utc::now().timestamp();
        if j.created_by.is_empty() {
            if let Some(existing) = map.get(&job.pda_address) {
                j.created_by = existing.created_by.clone();
                j.created_at = existing.created_at;
            }
        }
        map.insert(job.pda_address.clone(), j.clone());
        Ok(j)
    }

    async fn delete_job(&self, pda_address: &str) -> Result<(), RepositoryError> {
        let mut map = self.jobs.write().await;
        let mut job = map.get(pda_address).cloned().ok_or_else(|| RepositoryError::NotFound(pda_address.to_string()))?;
        job.is_active = false;
        job.deleted_at = Some(chrono::Utc::now().timestamp());
        job.updated_at = chrono::Utc::now().timestamp();
        map.insert(pda_address.to_string(), job);
        Ok(())
    }

    async fn list_jobs(&self) -> Result<Vec<JobMetadata>, RepositoryError> {
        let map = self.jobs.read().await;
        Ok(map.values().filter(|j| j.is_active).cloned().collect())
    }

    // ---- Applications ----
    async fn create_application(
        &self,
        app: ApplicationMetadata,
    ) -> Result<ApplicationMetadata, RepositoryError> {
        app.validate()?;
        let mut map = self.applications.write().await;
        if map.contains_key(&app.application_pda) {
            return Err(RepositoryError::AlreadyExists(app.application_pda.clone()));
        }
        map.insert(app.application_pda.clone(), app.clone());
        Ok(app)
    }

    async fn get_application(
        &self,
        application_pda: &str,
    ) -> Result<Option<ApplicationMetadata>, RepositoryError> {
        let map = self.applications.read().await;
        Ok(map.get(application_pda).cloned())
    }

    async fn list_applications_by_job(
        &self,
        job_pda: &str,
    ) -> Result<Vec<ApplicationMetadata>, RepositoryError> {
        let map = self.applications.read().await;
        Ok(map
            .values()
            .filter(|a| a.job_pda == job_pda && a.is_active)
            .cloned()
            .collect())
    }

    async fn delete_application(&self, application_pda: &str) -> Result<(), RepositoryError> {
        let mut map = self.applications.write().await;
        let mut app = map.get(application_pda).cloned().ok_or_else(|| RepositoryError::NotFound(application_pda.to_string()))?;
        app.is_active = false;
        app.deleted_at = Some(chrono::Utc::now().timestamp());
        app.updated_at = chrono::Utc::now().timestamp();
        map.insert(application_pda.to_string(), app);
        Ok(())
    }

    // ---- Milestones ----
    async fn create_milestone(
        &self,
        ms: MilestoneMetadata,
    ) -> Result<MilestoneMetadata, RepositoryError> {
        ms.validate()?;
        let mut map = self.milestones.write().await;
        let key = (ms.job_pda.clone(), ms.index);
        if map.contains_key(&key) {
            return Err(RepositoryError::AlreadyExists(format!(
                "{}#{}",
                key.0, key.1
            )));
        }
        map.insert(key, ms.clone());
        Ok(ms)
    }

    async fn get_milestone(
        &self,
        job_pda: &str,
        index: u8,
    ) -> Result<Option<MilestoneMetadata>, RepositoryError> {
        let map = self.milestones.read().await;
        Ok(map.get(&(job_pda.to_string(), index)).cloned())
    }

    async fn list_milestones_by_job(
        &self,
        job_pda: &str,
    ) -> Result<Vec<MilestoneMetadata>, RepositoryError> {
        let map = self.milestones.read().await;
        let mut out: Vec<_> = map
            .iter()
            .filter(|((job, _), _)| job == job_pda)
            .filter(|(_, ms)| ms.is_active)
            .map(|(_, ms)| ms.clone())
            .collect();
        out.sort_by_key(|ms| ms.index);
        Ok(out)
    }

    async fn delete_milestone(&self, job_pda: &str, index: u8) -> Result<(), RepositoryError> {
        let mut map = self.milestones.write().await;
        let key = (job_pda.to_string(), index);
        let mut ms = map.get(&key).cloned().ok_or_else(|| RepositoryError::NotFound(format!("{job_pda}#{index}")))?;
        ms.is_active = false;
        ms.deleted_at = Some(chrono::Utc::now().timestamp());
        ms.updated_at = chrono::Utc::now().timestamp();
        map.insert(key, ms);
        Ok(())
    }

    // ---- Disputes ----
    async fn create_dispute(
        &self,
        dispute: DisputeMetadata,
    ) -> Result<DisputeMetadata, RepositoryError> {
        dispute.validate()?;
        let mut map = self.disputes.write().await;
        if map.contains_key(&dispute.dispute_pda) {
            return Err(RepositoryError::AlreadyExists(dispute.dispute_pda.clone()));
        }
        map.insert(dispute.dispute_pda.clone(), dispute.clone());
        Ok(dispute)
    }

    async fn get_dispute(
        &self,
        dispute_pda: &str,
    ) -> Result<Option<DisputeMetadata>, RepositoryError> {
        let map = self.disputes.read().await;
        Ok(map.get(dispute_pda).cloned())
    }

    async fn update_dispute(
        &self,
        dispute: DisputeMetadata,
    ) -> Result<DisputeMetadata, RepositoryError> {
        dispute.validate()?;
        let mut map = self.disputes.write().await;
        if !map.contains_key(&dispute.dispute_pda) {
            return Err(RepositoryError::NotFound(dispute.dispute_pda.clone()));
        }
        let mut d = dispute.clone();
        d.updated_at = chrono::Utc::now().timestamp();
        if d.created_by.is_empty() {
            if let Some(existing) = map.get(&dispute.dispute_pda) {
                d.created_by = existing.created_by.clone();
                d.created_at = existing.created_at;
            }
        }
        map.insert(dispute.dispute_pda.clone(), d.clone());
        Ok(d)
    }

    async fn delete_dispute(&self, dispute_pda: &str) -> Result<(), RepositoryError> {
        let mut map = self.disputes.write().await;
        let mut d = map.get(dispute_pda).cloned().ok_or_else(|| RepositoryError::NotFound(dispute_pda.to_string()))?;
        d.is_active = false;
        d.deleted_at = Some(chrono::Utc::now().timestamp());
        d.updated_at = chrono::Utc::now().timestamp();
        map.insert(dispute_pda.to_string(), d);
        Ok(())
    }

    // ---- Support tickets ----
    async fn create_support_ticket(
        &self,
        ticket: SupportTicketMetadata,
    ) -> Result<SupportTicketMetadata, RepositoryError> {
        ticket.validate()?;
        let mut map = self.support_tickets.write().await;
        if map.contains_key(&ticket.ticket_pda) {
            return Err(RepositoryError::AlreadyExists(ticket.ticket_pda.clone()));
        }
        map.insert(ticket.ticket_pda.clone(), ticket.clone());
        Ok(ticket)
    }

    async fn get_support_ticket(
        &self,
        ticket_pda: &str,
    ) -> Result<Option<SupportTicketMetadata>, RepositoryError> {
        let map = self.support_tickets.read().await;
        Ok(map.get(ticket_pda).cloned())
    }

    async fn update_support_ticket(
        &self,
        ticket: SupportTicketMetadata,
    ) -> Result<SupportTicketMetadata, RepositoryError> {
        ticket.validate()?;
        let mut map = self.support_tickets.write().await;
        if !map.contains_key(&ticket.ticket_pda) {
            return Err(RepositoryError::NotFound(ticket.ticket_pda.clone()));
        }
        let mut t = ticket.clone();
        t.updated_at = chrono::Utc::now().timestamp();
        if t.created_by.is_empty() {
            if let Some(existing) = map.get(&ticket.ticket_pda) {
                t.created_by = existing.created_by.clone();
                t.created_at = existing.created_at;
            }
        }
        map.insert(ticket.ticket_pda.clone(), t.clone());
        Ok(t)
    }

    async fn delete_support_ticket(&self, ticket_pda: &str) -> Result<(), RepositoryError> {
        let mut map = self.support_tickets.write().await;
        let mut t = map.get(ticket_pda).cloned().ok_or_else(|| RepositoryError::NotFound(ticket_pda.to_string()))?;
        t.is_active = false;
        t.deleted_at = Some(chrono::Utc::now().timestamp());
        t.updated_at = chrono::Utc::now().timestamp();
        map.insert(ticket_pda.to_string(), t);
        Ok(())
    }

    // ---- Evidence ----
    async fn create_evidence(
        &self,
        evidence: EvidenceMetadata,
    ) -> Result<EvidenceMetadata, RepositoryError> {
        evidence.validate()?;
        let mut map = self.evidence.write().await;
        let key = (evidence.dispute_pda.clone(), evidence.index);
        if map.contains_key(&key) {
            return Err(RepositoryError::AlreadyExists(format!(
                "{}#{}",
                key.0, key.1
            )));
        }
        map.insert(key, evidence.clone());
        Ok(evidence)
    }

    async fn get_evidence(
        &self,
        dispute_pda: &str,
        index: u8,
    ) -> Result<Option<EvidenceMetadata>, RepositoryError> {
        let map = self.evidence.read().await;
        Ok(map.get(&(dispute_pda.to_string(), index)).cloned())
    }

    async fn list_evidence_by_dispute(
        &self,
        dispute_pda: &str,
    ) -> Result<Vec<EvidenceMetadata>, RepositoryError> {
        let map = self.evidence.read().await;
        let mut out: Vec<_> = map
            .iter()
            .filter(|((dispute, _), _)| dispute == dispute_pda)
            .filter(|(_, ev)| ev.is_active)
            .map(|(_, ev)| ev.clone())
            .collect();
        out.sort_by_key(|ev| ev.index);
        Ok(out)
    }

    async fn delete_evidence(&self, dispute_pda: &str, index: u8) -> Result<(), RepositoryError> {
        let mut map = self.evidence.write().await;
        let key = (dispute_pda.to_string(), index);
        let mut ev = map.get(&key).cloned().ok_or_else(|| RepositoryError::NotFound(format!("{dispute_pda}#{index}")))?;
        ev.is_active = false;
        ev.deleted_at = Some(chrono::Utc::now().timestamp());
        ev.updated_at = chrono::Utc::now().timestamp();
        map.insert(key, ev);
        Ok(())
    }

    // ---- Users ----
    async fn upsert_user(&self, user: UserMetadata) -> Result<UserMetadata, RepositoryError> {
        user.validate()?;
        let email_n = UserMetadata::normalize_email(&user.email);
        // normalize fields before storage
        let mut stored = user.clone();
        stored.email = email_n.clone();
        // Normalize roles: handle legacy alias
        let mut roles = stored.roles.clone();
        if roles.is_empty() && !stored.role.trim().is_empty() {
            roles = vec![UserMetadata::normalize_role(&stored.role)];
        }
        stored.roles = roles.iter().map(|r| UserMetadata::normalize_role(r)).collect();
        stored.role = stored.roles.first().cloned().unwrap_or_else(|| "guest".to_string());
        if stored.permissions.is_empty() && !stored.roles.is_empty() {
            stored.permissions = UserMetadata::default_permissions(&stored.roles);
        }
        stored.updated_at = chrono::Utc::now().timestamp();
        stored.updated_by = email_n.clone();
        if stored.created_at == 0 {
            stored.created_at = stored.updated_at;
        }
        if stored.created_by.is_empty() {
            stored.created_by = email_n.clone();
        }
        stored.is_active = true;
        stored.deleted_at = None;
        // Preserve existing wallet/created if exists
        let mut map = self.users.write().await;
        if let Some(existing) = map.get(&email_n) {
            if stored.wallet_pubkey.is_none() {
                stored.wallet_pubkey = existing.wallet_pubkey.clone();
            }
            if stored.created_at == existing.created_at || stored.created_at == 0 {
                stored.created_at = existing.created_at;
                stored.created_by = existing.created_by.clone();
            }
            // preserve is_active state unless explicitly re-activating
            if !existing.is_active && stored.is_active {
                // re-activate: keep new
            } else if !existing.is_active {
                stored.is_active = existing.is_active;
                stored.deleted_at = existing.deleted_at;
            }
        }
        // filter empty wallet => None
        if let Some(pk) = &stored.wallet_pubkey {
            if pk.trim().is_empty() {
                stored.wallet_pubkey = None;
            }
        }
        stored.validate()?;
        map.insert(email_n.clone(), stored.clone());
        Ok(stored)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<UserMetadata>, RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        let map = self.users.read().await;
        Ok(map.get(&email_n).cloned())
    }

    async fn update_wallet(
        &self,
        email: &str,
        wallet_pubkey: Option<String>,
    ) -> Result<UserMetadata, RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        // validate pubkey if Some
        if let Some(pk) = &wallet_pubkey {
            let trimmed = pk.trim();
            if !trimmed.is_empty() {
                let bytes = bs58::decode(trimmed)
                    .into_vec()
                    .map_err(|e| RepositoryError::Validation(crate::metadata::ValidationError::InvalidPda(format!("wallet pubkey base58: {}", e))))?;
                if bytes.len() != 32 {
                    return Err(RepositoryError::Validation(
                        crate::metadata::ValidationError::InvalidPda(
                            "wallet pubkey must be 32 bytes".to_string(),
                        ),
                    ));
                }
            }
        }
        let mut map = self.users.write().await;
        let mut user = map
            .get(&email_n)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(email_n.clone()))?;
        // normalize: empty string => None, else validated Some(trimmed)
        let normalized = match wallet_pubkey {
            None => None,
            Some(s) if s.trim().is_empty() => None,
            Some(s) => Some(s.trim().to_string()),
        };
        user.wallet_pubkey = normalized;
        user.updated_at = chrono::Utc::now().timestamp();
        user.validate()?;
        map.insert(email_n.clone(), user.clone());
        Ok(user)
    }

    async fn clear_wallet(&self, email: &str) -> Result<UserMetadata, RepositoryError> {
        self.update_wallet(email, None).await
    }

    async fn add_wallet(&self, wallet: crate::metadata::UserWallet) -> Result<crate::metadata::UserWallet, RepositoryError> {
        wallet.validate().map_err(RepositoryError::Validation)?;
        let email_n = UserMetadata::normalize_email(&wallet.email);
        let pubkey_n = wallet.pubkey.trim().to_string();
        let key = (email_n.clone(), pubkey_n.clone());
        // ensure user exists
        {
            let users = self.users.read().await;
            if !users.contains_key(&email_n) {
                return Err(RepositoryError::NotFound(email_n.clone()));
            }
        }
        let mut map = self.wallets.write().await;
        if let Some(existing) = map.get(&key) {
            if existing.is_active {
                return Err(RepositoryError::AlreadyExists(pubkey_n.clone()));
            }
        }
        let mut w = wallet.clone();
        w.email = email_n.clone();
        w.pubkey = pubkey_n.clone();
        w.validate().map_err(RepositoryError::Validation)?;
        map.insert(key, w.clone());
        // also keep legacy single wallet_pubkey in sync for backward compat (first active wallet)
        {
            let mut users = self.users.write().await;
            if let Some(u) = users.get_mut(&email_n) {
                if u.wallet_pubkey.is_none() || u.wallet_pubkey.as_deref().map(|s| s.trim().is_empty()).unwrap_or(true) {
                    u.wallet_pubkey = Some(pubkey_n.clone());
                    u.updated_at = chrono::Utc::now().timestamp();
                }
            }
        }
        Ok(w)
    }

    async fn list_wallets_by_email(&self, email: &str) -> Result<Vec<crate::metadata::UserWallet>, RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        let wallets = self.wallets.read().await;
        let mut out: Vec<crate::metadata::UserWallet> = wallets.values().filter(|w| w.email==email_n && w.is_active).cloned().collect();
        // migration: if no wallets but legacy wallet_pubkey exists, synthesize publish wallet
        if out.is_empty() {
            // clone user info before dropping locks to avoid borrow issues
            let legacy = {
                let users = self.users.read().await;
                users.get(&email_n).and_then(|u| {
                    u.wallet_pubkey.clone().filter(|s| !s.trim().is_empty()).map(|pk| {
                        (pk, u.created_at, u.updated_at, u.created_by.clone(), u.updated_by.clone())
                    })
                })
            };
            if let Some((pk, c_at, u_at, c_by, u_by)) = legacy {
                if crate::metadata::validate_pubkey_bs58(&pk).is_ok() {
                    drop(wallets);
                    let synth = crate::metadata::UserWallet {
                        email: email_n.clone(),
                        pubkey: pk.clone(),
                        purpose: crate::metadata::WalletPurpose::Publish,
                        label: Some("Principal".to_string()),
                        created_at: c_at,
                        updated_at: u_at,
                        created_by: c_by,
                        updated_by: u_by,
                        is_active: true,
                        deleted_at: None,
                    };
                    let mut wm = self.wallets.write().await;
                    wm.insert((email_n.clone(), pk.clone()), synth.clone());
                    out.push(synth);
                }
            }
        } else {
            // sort by created_at
            out.sort_by(|a,b| a.created_at.cmp(&b.created_at));
        }
        Ok(out)
    }

    async fn get_wallet(&self, email: &str, pubkey: &str) -> Result<Option<crate::metadata::UserWallet>, RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        let pk_n = pubkey.trim().to_string();
        let map = self.wallets.read().await;
        Ok(map.get(&(email_n, pk_n)).cloned().filter(|w| w.is_active))
    }

    async fn remove_wallet(&self, email: &str, pubkey: &str, actor: &str) -> Result<(), RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        let pk_n = pubkey.trim().to_string();
        let mut map = self.wallets.write().await;
        let key = (email_n.clone(), pk_n.clone());
        let mut w = map.get(&key).cloned().ok_or_else(|| RepositoryError::NotFound(pk_n.clone()))?;
        if !w.is_active {
            return Err(RepositoryError::NotFound(pk_n));
        }
        w.soft_delete(actor);
        map.insert(key, w.clone());
        // if legacy wallet equals this pubkey, clear it
        {
            let mut users = self.users.write().await;
            if let Some(u) = users.get_mut(&email_n) {
                if u.wallet_pubkey.as_deref() == Some(pk_n.as_str()) {
                    u.wallet_pubkey = None;
                    u.updated_at = chrono::Utc::now().timestamp();
                }
            }
        }
        Ok(())
    }

    async fn get_wallet_for_purpose(&self, email: &str, purpose: crate::metadata::WalletPurpose) -> Result<Option<crate::metadata::UserWallet>, RepositoryError> {
        let list = self.list_wallets_by_email(email).await?;
        Ok(list.into_iter().find(|w| w.purpose == purpose))
    }

    async fn add_participant(&self, p: crate::metadata::JobParticipant) -> Result<crate::metadata::JobParticipant, RepositoryError> {
        p.validate().map_err(RepositoryError::Validation)?;
        let email_n = UserMetadata::normalize_email(&p.email);
        let job_n = p.job_pda.trim().to_string();
        let key = (job_n.clone(), email_n.clone());
        let mut map = self.participants.write().await;
        if map.contains_key(&key) {
            return Err(RepositoryError::AlreadyExists(format!("{}:{}", job_n, email_n)));
        }
        let mut stored = p.clone();
        stored.email = email_n.clone();
        stored.job_pda = job_n.clone();
        stored.validate().map_err(RepositoryError::Validation)?;
        map.insert(key, stored.clone());
        Ok(stored)
    }

    async fn get_participant(&self, job_pda: &str, email: &str) -> Result<Option<crate::metadata::JobParticipant>, RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        let job_n = job_pda.trim().to_string();
        let map = self.participants.read().await;
        Ok(map.get(&(job_n, email_n)).cloned().filter(|p| p.is_active))
    }

    async fn list_participants_by_job(&self, job_pda: &str) -> Result<Vec<crate::metadata::JobParticipant>, RepositoryError> {
        let job_n = job_pda.trim().to_string();
        let map = self.participants.read().await;
        Ok(map.values().filter(|p| p.job_pda==job_n && p.is_active).cloned().collect())
    }

    async fn list_participants_by_email(&self, email: &str) -> Result<Vec<crate::metadata::JobParticipant>, RepositoryError> {
        let email_n = UserMetadata::normalize_email(email);
        let map = self.participants.read().await;
        Ok(map.values().filter(|p| p.email==email_n && p.is_active).cloned().collect())
    }

    async fn find_wallet_by_pubkey(&self, pubkey: &str) -> Result<Option<crate::metadata::UserWallet>, RepositoryError> {
        let pk_n = pubkey.trim().to_string();
        let map = self.wallets.read().await;
        Ok(map.values().find(|w| w.pubkey==pk_n && w.is_active).cloned().or_else(|| {
            // also check legacy users table not needed here
            None
        }))
    }
}

// ---------------------------------------------------------------------------
// Tests — async in-memory CRUD
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{
        ApplicationMetadata, DisputeMetadata, EvidenceMetadata, JobMetadata, MilestoneMetadata,
        SupportTicketMetadata,
    };

    fn pda(n: u8) -> String {
        format!("7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}", n, n)
    }

    #[tokio::test]
    async fn job_crud() {
        let repo = InMemoryMetadataRepository::new();
        let dl = chrono::Utc::now().timestamp() + 86400;
        let client = format!("7a2YhCd7iivXfyySkp1pf5jjClient{:0>20}{:02}", 1u8, 1u8);
        let job = JobMetadata::new(pda(1), "Title".into(), "Desc".into(), 1_000_000, 25_000, dl, client).unwrap();

        // create
        repo.create_job(job.clone()).await.unwrap();
        // duplicate -> AlreadyExists
        assert!(matches!(
            repo.create_job(job.clone()).await,
            Err(RepositoryError::AlreadyExists(_))
        ));
        // get
        let fetched = repo.get_job(&pda(1)).await.unwrap().unwrap();
        assert_eq!(fetched.title, "Title");
        assert!(fetched.is_active);
        // update
        let mut updated = fetched.clone();
        updated.title = "New title".into();
        repo.update_job(updated.clone()).await.unwrap();
        assert_eq!(
            repo.get_job(&pda(1)).await.unwrap().unwrap().title,
            "New title"
        );
        // list
        assert_eq!(repo.list_jobs().await.unwrap().len(), 1);
        // soft delete: list hides, get still returns with is_active false
        repo.delete_job(&pda(1)).await.unwrap();
        assert_eq!(repo.list_jobs().await.unwrap().len(), 0);
        let soft = repo.get_job(&pda(1)).await.unwrap().unwrap();
        assert!(!soft.is_active);
        assert!(soft.deleted_at.is_some());
    }

    #[tokio::test]
    async fn job_validation_rejected_on_create() {
        let repo = InMemoryMetadataRepository::new();
        // Empty title should be rejected by validate() before storage.
        let bad = JobMetadata {
            pda_address: pda(2),
            title: "".into(),
            description: "desc".into(),
            amount: 1_000_000,
            fee_amount: 25_000,
            deadline: 0,
            client: "client".into(),
            freelancer: None,
            status: crate::metadata::JobStatus::Created,
            skills: vec![],
            created_at: 0,
            updated_at: 0,
            created_by: String::new(),
            updated_by: String::new(),
            is_active: true,
            deleted_at: None,
        };
        assert!(matches!(
            repo.create_job(bad).await,
            Err(RepositoryError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn is_active_filter() {
        let repo = InMemoryMetadataRepository::new();
        let dl = chrono::Utc::now().timestamp() + 86400;
        let c = format!("7a2YhCd7iivXfyySkp1pf5jjClient{:0>20}{:02}", 9u8, 9u8);
        let j1 = JobMetadata::new(pda(91), "A".into(), "desc".into(), 1000, 25, dl, c.clone()).unwrap();
        let j2 = JobMetadata::new(pda(92), "B".into(), "desc".into(), 1000, 25, dl, c).unwrap();
        repo.create_job(j1).await.unwrap();
        repo.create_job(j2).await.unwrap();
        assert_eq!(repo.list_jobs().await.unwrap().len(), 2);
        repo.delete_job(&pda(91)).await.unwrap();
        // list hides soft-deleted
        let list = repo.list_jobs().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].pda_address, pda(92));
    }

    #[tokio::test]
    async fn has_wildcard() {
        use crate::metadata::has_wildcard;
        let perms = vec!["admin:*".to_string(), "jobs:view".to_string()];
        assert!(has_wildcard(&perms, "admin:users"));
        assert!(has_wildcard(&perms, "admin:wallets"));
        assert!(has_wildcard(&perms, "jobs:view"));
        assert!(!has_wildcard(&perms, "jobs:create"));
        let perms2 = vec!["jobs:view:own".to_string()];
        assert!(!has_wildcard(&perms2, "jobs:view"));
        assert!(has_wildcard(&perms2, "jobs:view:own"));
    }

    #[tokio::test]
    async fn application_crud() {
        let repo = InMemoryMetadataRepository::new();
        let job_pda = pda(10);
        let app_pda = pda(11);
        let app = ApplicationMetadata::new(
            app_pda.clone(),
            job_pda.clone(),
            "applicant111111111111111111111111111".into(),
            "Proposal text".into(),
        )
        .unwrap();
        repo.create_application(app.clone()).await.unwrap();
        let list = repo.list_applications_by_job(&job_pda).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].proposal, "Proposal text");
        repo.delete_application(&app_pda).await.unwrap();
        // soft-delete hides from list, get still returns inactive
        assert_eq!(repo.list_applications_by_job(&job_pda).await.unwrap().len(), 0);
        assert!(!repo.get_application(&app_pda).await.unwrap().unwrap().is_active);
    }

    #[tokio::test]
    async fn milestone_crud() {
        let repo = InMemoryMetadataRepository::new();
        let job_pda = pda(20);
        let ms = MilestoneMetadata::new(job_pda.clone(), 0, "M1".into(), "d".into()).unwrap();
        repo.create_milestone(ms).await.unwrap();
        let ms2 = MilestoneMetadata::new(job_pda.clone(), 1, "M2".into(), "d2".into()).unwrap();
        repo.create_milestone(ms2).await.unwrap();
        let list = repo.list_milestones_by_job(&job_pda).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].index, 0);
        repo.delete_milestone(&job_pda, 0).await.unwrap();
        // soft-delete filters list
        let list2 = repo.list_milestones_by_job(&job_pda).await.unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0].index, 1);
        // but direct get still shows inactive
        let soft = repo.get_milestone(&job_pda, 0).await.unwrap().unwrap();
        assert!(!soft.is_active);
    }

    #[tokio::test]
    async fn dispute_and_evidence_crud() {
        let repo = InMemoryMetadataRepository::new();
        let dispute_pda = pda(30);
        let job_pda = pda(31);
        let dispute =
            DisputeMetadata::new(dispute_pda.clone(), job_pda.clone(), "reason".into()).unwrap();
        repo.create_dispute(dispute).await.unwrap();

        let ev = EvidenceMetadata::new(
            dispute_pda.clone(),
            0,
            "author1111111111111111111111111111".into(),
            "content here".into(),
        )
        .unwrap();
        assert!(ev.verify_hash());
        repo.create_evidence(ev.clone()).await.unwrap();
        let list = repo.list_evidence_by_dispute(&dispute_pda).await.unwrap();
        assert_eq!(list.len(), 1);

        // update dispute with resolution
        let mut d = repo.get_dispute(&dispute_pda).await.unwrap().unwrap();
        d.resolve("resolved 50/50".into()).unwrap();
        repo.update_dispute(d.clone()).await.unwrap();
        assert!(repo
            .get_dispute(&dispute_pda)
            .await
            .unwrap()
            .unwrap()
            .resolution
            .is_some());

        repo.delete_evidence(&dispute_pda, 0).await.unwrap();
        // evidence soft-delete hides from list
        assert_eq!(repo.list_evidence_by_dispute(&dispute_pda).await.unwrap().len(), 0);
        repo.delete_dispute(&dispute_pda).await.unwrap();
        assert!(!repo.get_dispute(&dispute_pda).await.unwrap().unwrap().is_active);
    }

    #[tokio::test]
    async fn support_ticket_crud() {
        let repo = InMemoryMetadataRepository::new();
        let ticket_pda = pda(40);
        let job_pda = pda(41);
        let ticket =
            SupportTicketMetadata::new(ticket_pda.clone(), job_pda.clone(), "help".into()).unwrap();
        repo.create_support_ticket(ticket).await.unwrap();
        let mut t = repo.get_support_ticket(&ticket_pda).await.unwrap().unwrap();
        t.resolve("fixed".into()).unwrap();
        repo.update_support_ticket(t).await.unwrap();
        assert!(repo
            .get_support_ticket(&ticket_pda)
            .await
            .unwrap()
            .unwrap()
            .resolved_at
            .is_some());
        repo.delete_support_ticket(&ticket_pda).await.unwrap();
        assert!(!repo
            .get_support_ticket(&ticket_pda)
            .await
            .unwrap()
            .unwrap()
            .is_active);
    }
}

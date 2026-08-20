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
    SupportTicketMetadata, ValidationError,
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
        map.insert(job.pda_address.clone(), job.clone());
        Ok(job)
    }

    async fn delete_job(&self, pda_address: &str) -> Result<(), RepositoryError> {
        let mut map = self.jobs.write().await;
        map.remove(pda_address)
            .map(|_| ())
            .ok_or_else(|| RepositoryError::NotFound(pda_address.to_string()))
    }

    async fn list_jobs(&self) -> Result<Vec<JobMetadata>, RepositoryError> {
        let map = self.jobs.read().await;
        Ok(map.values().cloned().collect())
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
            .filter(|a| a.job_pda == job_pda)
            .cloned()
            .collect())
    }

    async fn delete_application(&self, application_pda: &str) -> Result<(), RepositoryError> {
        let mut map = self.applications.write().await;
        map.remove(application_pda)
            .map(|_| ())
            .ok_or_else(|| RepositoryError::NotFound(application_pda.to_string()))
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
            .map(|(_, ms)| ms.clone())
            .collect();
        out.sort_by_key(|ms| ms.index);
        Ok(out)
    }

    async fn delete_milestone(&self, job_pda: &str, index: u8) -> Result<(), RepositoryError> {
        let mut map = self.milestones.write().await;
        map.remove(&(job_pda.to_string(), index))
            .map(|_| ())
            .ok_or_else(|| RepositoryError::NotFound(format!("{job_pda}#{index}")))
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
        map.insert(dispute.dispute_pda.clone(), dispute.clone());
        Ok(dispute)
    }

    async fn delete_dispute(&self, dispute_pda: &str) -> Result<(), RepositoryError> {
        let mut map = self.disputes.write().await;
        map.remove(dispute_pda)
            .map(|_| ())
            .ok_or_else(|| RepositoryError::NotFound(dispute_pda.to_string()))
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
        map.insert(ticket.ticket_pda.clone(), ticket.clone());
        Ok(ticket)
    }

    async fn delete_support_ticket(&self, ticket_pda: &str) -> Result<(), RepositoryError> {
        let mut map = self.support_tickets.write().await;
        map.remove(ticket_pda)
            .map(|_| ())
            .ok_or_else(|| RepositoryError::NotFound(ticket_pda.to_string()))
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
            .map(|(_, ev)| ev.clone())
            .collect();
        out.sort_by_key(|ev| ev.index);
        Ok(out)
    }

    async fn delete_evidence(&self, dispute_pda: &str, index: u8) -> Result<(), RepositoryError> {
        let mut map = self.evidence.write().await;
        map.remove(&(dispute_pda.to_string(), index))
            .map(|_| ())
            .ok_or_else(|| RepositoryError::NotFound(format!("{dispute_pda}#{index}")))
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
        let job = JobMetadata::new(pda(1), "Title".into(), "Desc".into()).unwrap();

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
        // delete
        repo.delete_job(&pda(1)).await.unwrap();
        assert!(repo.get_job(&pda(1)).await.unwrap().is_none());
        assert!(matches!(
            repo.delete_job(&pda(1)).await,
            Err(RepositoryError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn job_validation_rejected_on_create() {
        let repo = InMemoryMetadataRepository::new();
        // Empty title should be rejected by validate() before storage.
        let bad = JobMetadata {
            pda_address: pda(2),
            title: "".into(),
            description: "desc".into(),
            skills: vec![],
            created_at: 0,
            updated_at: 0,
        };
        assert!(matches!(
            repo.create_job(bad).await,
            Err(RepositoryError::Validation(_))
        ));
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
        assert!(repo.get_application(&app_pda).await.unwrap().is_none());
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
        assert_eq!(
            repo.list_milestones_by_job(&job_pda).await.unwrap().len(),
            1
        );
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
        repo.delete_dispute(&dispute_pda).await.unwrap();
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
        assert!(repo
            .get_support_ticket(&ticket_pda)
            .await
            .unwrap()
            .is_none());
    }
}

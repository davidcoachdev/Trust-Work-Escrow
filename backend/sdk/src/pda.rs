//! PDA derivation and caching for `trust-escrow-v3`.
//!
//! All seed patterns match the contract's `lib.rs` exactly. Derivation uses
//! `solana-sdk` (behind the `solana` feature); results are cached in a
//! thread-safe global for repeated lookups.

#[cfg(feature = "solana")]
mod inner {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    use solana_sdk::pubkey::Pubkey;

    use crate::error::{BackendError, Result};
    use crate::PROGRAM_ID_STR;

    // Seed constants — must match the contract's `seeds = [...]` exactly.
    const CONFIG_SEED: &[u8] = b"config";
    const JOB_SEED: &[u8] = b"job";
    const APPLICATION_SEED: &[u8] = b"application";
    const ARBITER_POOL_SEED: &[u8] = b"arbiter_pool";
    const DISPUTE_SEED: &[u8] = b"dispute";
    const ARB_FEE_SEED: &[u8] = b"arb_fee";
    const MILESTONE_SEED: &[u8] = b"milestone";
    const EVIDENCE_SEED: &[u8] = b"evidence";
    const SUPPORT_SEED: &[u8] = b"support";

    fn program_id() -> Result<Pubkey> {
        PROGRAM_ID_STR.parse().map_err(BackendError::SolanaSdk)
    }

    // ---- derive_*_pda: pure derivation (no cache) ----

    pub fn derive_config_pda() -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(&[CONFIG_SEED], &pid))
    }

    pub fn derive_job_pda(client: &Pubkey, job_id: u64) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        let id = job_id.to_le_bytes();
        Ok(Pubkey::find_program_address(
            &[JOB_SEED, client.as_ref(), &id],
            &pid,
        ))
    }

    pub fn derive_application_pda(
        job: &Pubkey,
        index: u8,
        applicant: &Pubkey,
    ) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(
            &[APPLICATION_SEED, job.as_ref(), &[index], applicant.as_ref()],
            &pid,
        ))
    }

    pub fn derive_arbiter_pool_pda() -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(&[ARBITER_POOL_SEED], &pid))
    }

    pub fn derive_dispute_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(
            &[DISPUTE_SEED, job.as_ref()],
            &pid,
        ))
    }

    pub fn derive_arb_fee_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(
            &[ARB_FEE_SEED, job.as_ref()],
            &pid,
        ))
    }

    pub fn derive_milestone_pda(job: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(
            &[MILESTONE_SEED, job.as_ref(), &[index]],
            &pid,
        ))
    }

    pub fn derive_evidence_pda(dispute: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(
            &[EVIDENCE_SEED, dispute.as_ref(), &[index]],
            &pid,
        ))
    }

    pub fn derive_support_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
        let pid = program_id()?;
        Ok(Pubkey::find_program_address(
            &[SUPPORT_SEED, job.as_ref()],
            &pid,
        ))
    }

    // ---- Thread-safe cache ----

    #[derive(Default)]
    struct PdaCache {
        config: Option<(Pubkey, u8)>,
        arbiter_pool: Option<(Pubkey, u8)>,
        jobs: HashMap<(Pubkey, u64), (Pubkey, u8)>,
        applications: HashMap<(Pubkey, u8, Pubkey), (Pubkey, u8)>,
        disputes: HashMap<Pubkey, (Pubkey, u8)>,
        arb_fees: HashMap<Pubkey, (Pubkey, u8)>,
        milestones: HashMap<(Pubkey, u8), (Pubkey, u8)>,
        evidences: HashMap<(Pubkey, u8), (Pubkey, u8)>,
        supports: HashMap<Pubkey, (Pubkey, u8)>,
    }

    static PDA_CACHE: OnceLock<Mutex<PdaCache>> = OnceLock::new();

    fn cache() -> &'static Mutex<PdaCache> {
        PDA_CACHE.get_or_init(|| Mutex::new(PdaCache::default()))
    }

    // ---- get_*_pda: cached lookups ----

    pub fn get_config_pda() -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        if let Some(v) = c.config {
            Ok(v)
        } else {
            let v = derive_config_pda()?;
            c.config = Some(v);
            Ok(v)
        }
    }

    pub fn get_job_pda(client: &Pubkey, job_id: u64) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        let key = (*client, job_id);
        if let Some(v) = c.jobs.get(&key) {
            Ok(*v)
        } else {
            let v = derive_job_pda(client, job_id)?;
            c.jobs.insert(key, v);
            Ok(v)
        }
    }

    pub fn get_application_pda(
        job: &Pubkey,
        index: u8,
        applicant: &Pubkey,
    ) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        let key = (*job, index, *applicant);
        if let Some(v) = c.applications.get(&key) {
            Ok(*v)
        } else {
            let v = derive_application_pda(job, index, applicant)?;
            c.applications.insert(key, v);
            Ok(v)
        }
    }

    pub fn get_arbiter_pool_pda() -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        if let Some(v) = c.arbiter_pool {
            Ok(v)
        } else {
            let v = derive_arbiter_pool_pda()?;
            c.arbiter_pool = Some(v);
            Ok(v)
        }
    }

    pub fn get_dispute_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        if let Some(v) = c.disputes.get(job) {
            Ok(*v)
        } else {
            let v = derive_dispute_pda(job)?;
            c.disputes.insert(*job, v);
            Ok(v)
        }
    }

    pub fn get_arb_fee_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        if let Some(v) = c.arb_fees.get(job) {
            Ok(*v)
        } else {
            let v = derive_arb_fee_pda(job)?;
            c.arb_fees.insert(*job, v);
            Ok(v)
        }
    }

    pub fn get_milestone_pda(job: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        let key = (*job, index);
        if let Some(v) = c.milestones.get(&key) {
            Ok(*v)
        } else {
            let v = derive_milestone_pda(job, index)?;
            c.milestones.insert(key, v);
            Ok(v)
        }
    }

    pub fn get_evidence_pda(dispute: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        let key = (*dispute, index);
        if let Some(v) = c.evidences.get(&key) {
            Ok(*v)
        } else {
            let v = derive_evidence_pda(dispute, index)?;
            c.evidences.insert(key, v);
            Ok(v)
        }
    }

    pub fn get_support_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
        let mut c = cache().lock().unwrap();
        if let Some(v) = c.supports.get(job) {
            Ok(*v)
        } else {
            let v = derive_support_pda(job)?;
            c.supports.insert(*job, v);
            Ok(v)
        }
    }

    /// Clear the global PDA cache (useful for tests).
    pub fn clear_pda_cache() {
        let mut c = cache().lock().unwrap();
        *c = PdaCache::default();
    }
}

#[cfg(feature = "solana")]
pub use inner::*;

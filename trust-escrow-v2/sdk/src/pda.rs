//! Program Derived Address (PDA) utilities for Trust Escrow v2
//!
//! This module provides PDA derivation functions and caching infrastructure
//! for all account types in the Trust Escrow program. All seed patterns
//! match exactly with the v2 smart contract implementation.

use dashmap::DashMap;
use lazy_static::lazy_static;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

use crate::error::Result;
use crate::PROGRAM_ID;

/// PDA seed constants matching v2 contract patterns
pub const CONFIG_SEED: &[u8] = b"config";
pub const USER_SEED: &[u8] = b"user";
pub const TEAM_SEED: &[u8] = b"team";
pub const JOB_SEED: &[u8] = b"job";
pub const ARBITER_POOL_SEED: &[u8] = b"arbiter_pool";
pub const DISPUTE_SEED: &[u8] = b"dispute";
pub const MILESTONE_SEED: &[u8] = b"milestone";

/// Thread-safe PDA cache for performance optimization
pub struct PdaCache {
    /// Config PDA (singleton)
    pub config: Option<Pubkey>,
    /// User PDAs by authority
    pub users: Arc<DashMap<Pubkey, (Pubkey, u8)>>,
    /// Team PDAs by owner
    pub teams: Arc<DashMap<Pubkey, (Pubkey, u8)>>,
    /// Job PDAs by (client, job_id)
    pub jobs: Arc<DashMap<(Pubkey, u64), (Pubkey, u8)>>,
    /// Arbiter pool PDA (singleton)
    pub arbiter_pool: Option<(Pubkey, u8)>,
    /// Dispute PDAs by job
    pub disputes: Arc<DashMap<Pubkey, (Pubkey, u8)>>,
    /// Milestone PDAs by (job, index)
    pub milestones: Arc<DashMap<(Pubkey, u8), (Pubkey, u8)>>,
}

impl Default for PdaCache {
    fn default() -> Self {
        Self {
            config: None,
            users: Arc::new(DashMap::new()),
            teams: Arc::new(DashMap::new()),
            jobs: Arc::new(DashMap::new()),
            arbiter_pool: None,
            disputes: Arc::new(DashMap::new()),
            milestones: Arc::new(DashMap::new()),
        }
    }
}

impl PdaCache {
    /// Get or derive config PDA
    pub fn get_config_pda(&mut self) -> Result<(Pubkey, u8)> {
        if let Some(config_pda) = self.config {
            // Config is cached but we need the bump, derive it
            let (pda, bump) = derive_config_pda()?;
            Ok((pda, bump))
        } else {
            let (pda, bump) = derive_config_pda()?;
            self.config = Some(pda);
            Ok((pda, bump))
        }
    }

    /// Get or derive user PDA with caching
    pub fn get_user_pda(&self, authority: &Pubkey) -> Result<(Pubkey, u8)> {
        if let Some(entry) = self.users.get(authority) {
            Ok(*entry.value())
        } else {
            let (pda, bump) = derive_user_pda(authority)?;
            self.users.insert(*authority, (pda, bump));
            Ok((pda, bump))
        }
    }

    /// Get or derive team PDA with caching
    pub fn get_team_pda(&self, owner: &Pubkey) -> Result<(Pubkey, u8)> {
        if let Some(entry) = self.teams.get(owner) {
            Ok(*entry.value())
        } else {
            let (pda, bump) = derive_team_pda(owner)?;
            self.teams.insert(*owner, (pda, bump));
            Ok((pda, bump))
        }
    }

    /// Get or derive job PDA with caching
    pub fn get_job_pda(&self, client: &Pubkey, job_id: u64) -> Result<(Pubkey, u8)> {
        let key = (*client, job_id);
        if let Some(entry) = self.jobs.get(&key) {
            Ok(*entry.value())
        } else {
            let (pda, bump) = derive_job_pda(client, job_id)?;
            self.jobs.insert(key, (pda, bump));
            Ok((pda, bump))
        }
    }

    /// Get or derive arbiter pool PDA
    pub fn get_arbiter_pool_pda(&mut self) -> Result<(Pubkey, u8)> {
        if let Some(arbiter_pool) = self.arbiter_pool {
            Ok(arbiter_pool)
        } else {
            let (pda, bump) = derive_arbiter_pool_pda()?;
            self.arbiter_pool = Some((pda, bump));
            Ok((pda, bump))
        }
    }

    /// Get or derive dispute PDA with caching
    pub fn get_dispute_pda(&self, job: &Pubkey) -> Result<(Pubkey, u8)> {
        if let Some(entry) = self.disputes.get(job) {
            Ok(*entry.value())
        } else {
            let (pda, bump) = derive_dispute_pda(job)?;
            self.disputes.insert(*job, (pda, bump));
            Ok((pda, bump))
        }
    }

    /// Get or derive milestone PDA with caching
    pub fn get_milestone_pda(&self, job: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
        let key = (*job, index);
        if let Some(entry) = self.milestones.get(&key) {
            Ok(*entry.value())
        } else {
            let (pda, bump) = derive_milestone_pda(job, index)?;
            self.milestones.insert(key, (pda, bump));
            Ok((pda, bump))
        }
    }

    /// Clear cache (useful for testing or cache management)
    pub fn clear(&mut self) {
        self.config = None;
        self.users.clear();
        self.teams.clear();
        self.jobs.clear();
        self.arbiter_pool = None;
        self.disputes.clear();
        self.milestones.clear();
    }

    /// Get cache statistics for monitoring
    pub fn stats(&self) -> PdaCacheStats {
        PdaCacheStats {
            config_cached: self.config.is_some(),
            users_cached: self.users.len(),
            teams_cached: self.teams.len(),
            jobs_cached: self.jobs.len(),
            arbiter_pool_cached: self.arbiter_pool.is_some(),
            disputes_cached: self.disputes.len(),
            milestones_cached: self.milestones.len(),
        }
    }
}

/// Cache statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct PdaCacheStats {
    pub config_cached: bool,
    pub users_cached: usize,
    pub teams_cached: usize,
    pub jobs_cached: usize,
    pub arbiter_pool_cached: bool,
    pub disputes_cached: usize,
    pub milestones_cached: usize,
}

// Global PDA cache instance
lazy_static! {
    static ref PDA_CACHE: std::sync::Mutex<PdaCache> = std::sync::Mutex::new(PdaCache::default());
}

/// Derive config PDA
/// Seed: ["config"]
pub fn derive_config_pda() -> Result<(Pubkey, u8)> {
    let (pda, bump) = Pubkey::find_program_address(&[CONFIG_SEED], &PROGRAM_ID);
    Ok((pda, bump))
}

/// Derive user PDA  
/// Seed: ["user", authority]
pub fn derive_user_pda(authority: &Pubkey) -> Result<(Pubkey, u8)> {
    let (pda, bump) = Pubkey::find_program_address(&[USER_SEED, authority.as_ref()], &PROGRAM_ID);
    Ok((pda, bump))
}

/// Derive team PDA
/// Seed: ["team", owner]
pub fn derive_team_pda(owner: &Pubkey) -> Result<(Pubkey, u8)> {
    let (pda, bump) = Pubkey::find_program_address(&[TEAM_SEED, owner.as_ref()], &PROGRAM_ID);
    Ok((pda, bump))
}

/// Derive job PDA
/// Seed: ["job", client, job_id]
pub fn derive_job_pda(client: &Pubkey, job_id: u64) -> Result<(Pubkey, u8)> {
    let job_id_bytes = job_id.to_le_bytes();
    let (pda, bump) =
        Pubkey::find_program_address(&[JOB_SEED, client.as_ref(), &job_id_bytes], &PROGRAM_ID);
    Ok((pda, bump))
}

/// Derive arbiter pool PDA
/// Seed: ["arbiter_pool"]  
pub fn derive_arbiter_pool_pda() -> Result<(Pubkey, u8)> {
    let (pda, bump) = Pubkey::find_program_address(&[ARBITER_POOL_SEED], &PROGRAM_ID);
    Ok((pda, bump))
}

/// Derive dispute PDA
/// Seed: ["dispute", job]
pub fn derive_dispute_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
    let (pda, bump) = Pubkey::find_program_address(&[DISPUTE_SEED, job.as_ref()], &PROGRAM_ID);
    Ok((pda, bump))
}

/// Derive milestone PDA
/// Seed: ["milestone", job, index]
pub fn derive_milestone_pda(job: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
    let index_bytes = [index];
    let (pda, bump) =
        Pubkey::find_program_address(&[MILESTONE_SEED, job.as_ref(), &index_bytes], &PROGRAM_ID);
    Ok((pda, bump))
}

/// High-level cached PDA functions using global cache

/// Get config PDA with caching
pub fn get_config_pda() -> Result<(Pubkey, u8)> {
    let mut cache = PDA_CACHE.lock().unwrap();
    cache.get_config_pda()
}

/// Get user PDA with caching
pub fn get_user_pda(authority: &Pubkey) -> Result<(Pubkey, u8)> {
    let cache = PDA_CACHE.lock().unwrap();
    cache.get_user_pda(authority)
}

/// Get team PDA with caching
pub fn get_team_pda(owner: &Pubkey) -> Result<(Pubkey, u8)> {
    let cache = PDA_CACHE.lock().unwrap();
    cache.get_team_pda(owner)
}

/// Get job PDA with caching
pub fn get_job_pda(client: &Pubkey, job_id: u64) -> Result<(Pubkey, u8)> {
    let cache = PDA_CACHE.lock().unwrap();
    cache.get_job_pda(client, job_id)
}

/// Get arbiter pool PDA with caching
pub fn get_arbiter_pool_pda() -> Result<(Pubkey, u8)> {
    let mut cache = PDA_CACHE.lock().unwrap();
    cache.get_arbiter_pool_pda()
}

/// Get dispute PDA with caching  
pub fn get_dispute_pda(job: &Pubkey) -> Result<(Pubkey, u8)> {
    let cache = PDA_CACHE.lock().unwrap();
    cache.get_dispute_pda(job)
}

/// Get milestone PDA with caching
pub fn get_milestone_pda(job: &Pubkey, index: u8) -> Result<(Pubkey, u8)> {
    let cache = PDA_CACHE.lock().unwrap();
    cache.get_milestone_pda(job, index)
}

/// Clear global PDA cache (useful for testing)
pub fn clear_pda_cache() {
    let mut cache = PDA_CACHE.lock().unwrap();
    cache.clear();
}

/// Get PDA cache statistics
pub fn get_pda_cache_stats() -> PdaCacheStats {
    let cache = PDA_CACHE.lock().unwrap();
    cache.stats()
}

/// Batch derive multiple PDAs for efficiency
pub struct BatchPdaBuilder {
    operations: Vec<PdaOperation>,
}

#[derive(Debug, Clone)]
pub enum PdaOperation {
    Config,
    User(Pubkey),
    Team(Pubkey),
    Job(Pubkey, u64),
    ArbiterPool,
    Dispute(Pubkey),
    Milestone(Pubkey, u8),
}

impl BatchPdaBuilder {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn add_config(mut self) -> Self {
        self.operations.push(PdaOperation::Config);
        self
    }

    pub fn add_user(mut self, authority: Pubkey) -> Self {
        self.operations.push(PdaOperation::User(authority));
        self
    }

    pub fn add_team(mut self, owner: Pubkey) -> Self {
        self.operations.push(PdaOperation::Team(owner));
        self
    }

    pub fn add_job(mut self, client: Pubkey, job_id: u64) -> Self {
        self.operations.push(PdaOperation::Job(client, job_id));
        self
    }

    pub fn add_arbiter_pool(mut self) -> Self {
        self.operations.push(PdaOperation::ArbiterPool);
        self
    }

    pub fn add_dispute(mut self, job: Pubkey) -> Self {
        self.operations.push(PdaOperation::Dispute(job));
        self
    }

    pub fn add_milestone(mut self, job: Pubkey, index: u8) -> Self {
        self.operations.push(PdaOperation::Milestone(job, index));
        self
    }

    /// Execute all PDA derivations
    pub fn build(self) -> Result<Vec<(Pubkey, u8)>> {
        let mut results = Vec::with_capacity(self.operations.len());

        for operation in self.operations {
            let pda = match operation {
                PdaOperation::Config => get_config_pda()?,
                PdaOperation::User(authority) => get_user_pda(&authority)?,
                PdaOperation::Team(owner) => get_team_pda(&owner)?,
                PdaOperation::Job(client, job_id) => get_job_pda(&client, job_id)?,
                PdaOperation::ArbiterPool => get_arbiter_pool_pda()?,
                PdaOperation::Dispute(job) => get_dispute_pda(&job)?,
                PdaOperation::Milestone(job, index) => get_milestone_pda(&job, index)?,
            };
            results.push(pda);
        }

        Ok(results)
    }
}

impl Default for BatchPdaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pda_derivation() {
        let authority = Pubkey::new_unique();
        let client = Pubkey::new_unique();
        let job = Pubkey::new_unique();

        // Test basic PDA derivation
        let (config_pda, config_bump) = derive_config_pda().unwrap();
        assert_ne!(config_pda, Pubkey::default());
        assert!(config_bump <= 255);

        let (user_pda, user_bump) = derive_user_pda(&authority).unwrap();
        assert_ne!(user_pda, Pubkey::default());
        assert!(user_bump <= 255);

        let (job_pda, job_bump) = derive_job_pda(&client, 1).unwrap();
        assert_ne!(job_pda, Pubkey::default());
        assert!(job_bump <= 255);

        let (dispute_pda, dispute_bump) = derive_dispute_pda(&job).unwrap();
        assert_ne!(dispute_pda, Pubkey::default());
        assert!(dispute_bump <= 255);

        let (milestone_pda, milestone_bump) = derive_milestone_pda(&job, 0).unwrap();
        assert_ne!(milestone_pda, Pubkey::default());
        assert!(milestone_bump <= 255);
    }

    #[test]
    fn test_pda_determinism() {
        let authority = Pubkey::new_unique();

        // Same inputs should produce same PDAs
        let (pda1, bump1) = derive_user_pda(&authority).unwrap();
        let (pda2, bump2) = derive_user_pda(&authority).unwrap();

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_pda_caching() {
        clear_pda_cache();

        let authority = Pubkey::new_unique();

        // First call should derive and cache
        let (pda1, bump1) = get_user_pda(&authority).unwrap();

        // Second call should use cache
        let (pda2, bump2) = get_user_pda(&authority).unwrap();

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);

        // Verify cache has entry
        let stats = get_pda_cache_stats();
        assert_eq!(stats.users_cached, 1);
    }

    #[test]
    fn test_batch_pda_builder() {
        clear_pda_cache();

        let authority = Pubkey::new_unique();
        let client = Pubkey::new_unique();

        let pdas = BatchPdaBuilder::new()
            .add_config()
            .add_user(authority)
            .add_job(client, 1)
            .add_arbiter_pool()
            .build()
            .unwrap();

        assert_eq!(pdas.len(), 4);

        // Verify all PDAs are valid
        for (pda, bump) in pdas {
            assert_ne!(pda, Pubkey::default());
            assert!(bump <= 255);
        }
    }
}

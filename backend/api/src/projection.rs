use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::Finality;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountKey { pub cluster: String, pub program_id: String, pub account_pubkey: String }

impl AccountKey {
    pub fn new(cluster: &str, program_id: &str, account_pubkey: &str) -> Result<Self, String> {
        if cluster.trim().is_empty() || program_id.trim().is_empty() || account_pubkey.trim().is_empty() { return Err("cluster, program_id and account_pubkey are required".into()); }
        if cluster.contains("devnet") || cluster.contains("mainnet") || cluster.contains("testnet") { return Err("public clusters are not allowed for local deterministic fixtures".into()); }
        Ok(Self { cluster: cluster.into(), program_id: program_id.into(), account_pubkey: account_pubkey.into() })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionStatus { Current, Stale, Divergent, Closed }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection { pub slot: u64, pub status: ProjectionStatus }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tombstone { pub key: AccountKey, pub closed_slot: u64, pub finality: Finality }

impl Tombstone { pub fn new(key: AccountKey, closed_slot: u64, finality: Finality) -> Self { Self { key, closed_slot, finality } } }

#[derive(Clone, Default)]
pub struct ProjectionStore { projections: Arc<Mutex<HashMap<AccountKey, Projection>>>, tombstones: Arc<Mutex<HashMap<AccountKey, Tombstone>>> }

impl ProjectionStore {
    pub fn upsert(&self, key: AccountKey, slot: u64, status: ProjectionStatus) -> Result<(), String> {
        if self.tombstones.lock().map_err(|_| "tombstone lock poisoned")?.get(&key).is_some_and(|t| slot <= t.closed_slot) { return Err("stale data cannot resurrect a tombstoned account".into()); }
        self.projections.lock().map_err(|_| "projection lock poisoned")?.insert(key, Projection { slot, status });
        Ok(())
    }
    pub fn tombstone(&self, tombstone: Tombstone) -> Result<(), String> {
        let key = tombstone.key.clone();
        self.projections.lock().map_err(|_| "projection lock poisoned")?.remove(&key);
        self.tombstones.lock().map_err(|_| "tombstone lock poisoned")?.insert(key, tombstone);
        Ok(())
    }
    pub fn len(&self) -> usize { self.projections.lock().map(|p| p.len()).unwrap_or(0) }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn has_tombstone(&self, key: &AccountKey) -> bool { self.tombstones.lock().map(|t| t.contains_key(key)).unwrap_or(false) }
}

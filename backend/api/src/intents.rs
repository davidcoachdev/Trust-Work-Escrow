use std::{collections::HashMap, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finality { Intent, Submitted, Processed, Confirmed, Finalized, Failed, Reorged }

impl Finality {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next),
            (Self::Intent, Self::Submitted | Self::Failed) |
            (Self::Submitted, Self::Processed | Self::Confirmed | Self::Failed | Self::Reorged) |
            (Self::Processed, Self::Confirmed | Self::Failed | Self::Reorged) |
            (Self::Confirmed, Self::Finalized | Self::Failed | Self::Reorged))
    }
}

pub type IntentStatus = Finality;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionIntent { pub id: String, pub idempotency_key: String, pub operation: String, pub status: IntentStatus, pub attempt_count: u32, pub created_at: u64 }

#[derive(Clone, Default)]
pub struct IntentStore { inner: Arc<Mutex<HashMap<String, TransactionIntent>>> }

impl IntentStore {
    pub fn create(&self, key: &str, operation: &str) -> Result<TransactionIntent, String> {
        if key.trim().is_empty() || operation.trim().is_empty() { return Err("idempotency key and operation are required".into()); }
        let mut entries = self.inner.lock().map_err(|_| "intent store lock poisoned".to_string())?;
        if let Some(existing) = entries.get(key) { return Ok(existing.clone()); }
        let id = format!("intent-{}", entries.len() + 1);
        let intent = TransactionIntent { id, idempotency_key: key.into(), operation: operation.into(), status: Finality::Intent, attempt_count: 0, created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() };
        entries.insert(key.into(), intent.clone());
        Ok(intent)
    }

    pub fn transition(&self, id: &str, next: IntentStatus) -> Result<TransactionIntent, String> {
        let mut entries = self.inner.lock().map_err(|_| "intent store lock poisoned".to_string())?;
        let intent = entries.values_mut().find(|i| i.id == id).ok_or_else(|| "intent not found".to_string())?;
        if !intent.status.can_transition_to(next) { return Err(format!("invalid finality transition {:?} -> {:?}", intent.status, next)); }
        intent.status = next;
        if matches!(next, Finality::Submitted | Finality::Processed | Finality::Confirmed) { intent.attempt_count += 1; }
        Ok(intent.clone())
    }

    pub fn len(&self) -> usize { self.inner.lock().map(|e| e.len()).unwrap_or(0) }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}

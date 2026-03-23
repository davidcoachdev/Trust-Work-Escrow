//! Event monitoring and subscription for Trust Escrow v2 contract
//!
//! This module provides capabilities to monitor and subscribe to events emitted by the
//! Trust Escrow v2 smart contract, enabling real-time updates and notifications.

use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::option_serializer::OptionSerializer;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc;

use crate::{error::Result, EscrowError, PROGRAM_ID};

/// Event types that can be emitted by the Trust Escrow contract
#[derive(Debug, Clone, PartialEq)]
pub enum EscrowEvent {
    /// User account created
    UserCreated {
        user: Pubkey,
        username: String,
        authority: Pubkey,
    },
    /// Team created
    TeamCreated {
        team: Pubkey,
        owner: Pubkey,
        name: String,
    },
    /// Job created
    JobCreated {
        job: Pubkey,
        client: Pubkey,
        amount: u64,
        title: String,
    },
    /// Job funded by client
    JobFunded {
        job: Pubkey,
        client: Pubkey,
        amount: u64,
    },
    /// Freelancer applied to job
    JobApplication {
        job: Pubkey,
        freelancer: Pubkey,
        proposal: String,
    },
    /// Application accepted by client
    ApplicationAccepted {
        job: Pubkey,
        freelancer: Pubkey,
        client: Pubkey,
    },
    /// Work submitted by freelancer
    WorkSubmitted {
        job: Pubkey,
        freelancer: Pubkey,
        work_url: String,
    },
    /// Work approved by client
    WorkApproved {
        job: Pubkey,
        client: Pubkey,
        freelancer: Pubkey,
        amount: u64,
    },
    /// Dispute raised
    DisputeRaised {
        dispute: Pubkey,
        job: Pubkey,
        initiator: Pubkey,
        evidence: String,
    },
    /// Dispute resolved
    DisputeResolved {
        dispute: Pubkey,
        job: Pubkey,
        winner: Pubkey,
        client_amount: u64,
        freelancer_amount: u64,
    },
    /// Milestone created
    MilestoneCreated {
        milestone: Pubkey,
        job: Pubkey,
        title: String,
        amount: u64,
    },
    /// Milestone submitted
    MilestoneSubmitted {
        milestone: Pubkey,
        job: Pubkey,
        freelancer: Pubkey,
        work_url: String,
    },
    /// Milestone approved
    MilestoneApproved {
        milestone: Pubkey,
        job: Pubkey,
        client: Pubkey,
        amount: u64,
    },
}

/// Event listener configuration
#[derive(Debug, Clone)]
pub struct EventListenerConfig {
    /// Polling interval for fetching new transactions
    pub polling_interval: Duration,
    /// Number of signatures to fetch per request
    pub batch_size: usize,
    /// Maximum number of events to buffer
    pub buffer_size: usize,
    /// Commitment level for transaction fetching
    pub commitment: CommitmentConfig,
}

impl Default for EventListenerConfig {
    fn default() -> Self {
        Self {
            polling_interval: Duration::from_millis(1000),
            batch_size: 100,
            buffer_size: 1000,
            commitment: CommitmentConfig::confirmed(),
        }
    }
}

/// Event listener for Trust Escrow contract events
pub struct EventListener {
    rpc: Arc<RpcClient>,
    config: EventListenerConfig,
    event_sender: Option<mpsc::UnboundedSender<EscrowEvent>>,
}

impl EventListener {
    /// Create a new event listener
    pub fn new(rpc: Arc<RpcClient>, config: EventListenerConfig) -> Self {
        Self {
            rpc,
            config,
            event_sender: None,
        }
    }

    /// Start listening for events
    /// Returns a receiver that will yield events as they are detected
    pub fn start_listening(&mut self) -> mpsc::UnboundedReceiver<EscrowEvent> {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.event_sender = Some(sender);
        receiver
    }

    /// Stop listening for events
    pub fn stop_listening(&mut self) {
        self.event_sender = None;
    }

    /// Parse transaction logs to extract events
    pub fn parse_transaction_logs(&self, logs: &[String]) -> Vec<EscrowEvent> {
        let mut events = Vec::new();

        for log in logs {
            if let Some(event) = self.parse_log_entry(log) {
                events.push(event);
            }
        }

        events
    }

    /// Get recent events from the last N transactions
    pub async fn get_recent_events(&self, limit: usize) -> Result<Vec<EscrowEvent>> {
        let signatures = self
            .rpc
            .get_signatures_for_address_with_config(
                &PROGRAM_ID,
                solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                    limit: Some(limit),
                    commitment: Some(self.config.commitment),
                    ..Default::default()
                },
            )
            .map_err(|e| EscrowError::network_error(&format!("Failed to get signatures: {}", e)))?;

        let mut all_events = Vec::new();

        for sig_info in signatures {
            if let Ok(signature) = sig_info.signature.parse::<Signature>() {
                if let Ok(transaction) = self.rpc.get_transaction_with_config(
                    &signature,
                    solana_client::rpc_config::RpcTransactionConfig {
                        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
                        commitment: Some(self.config.commitment),
                        max_supported_transaction_version: Some(0),
                    },
                ) {
                    if let Some(meta) = transaction.transaction.meta {
                        if let OptionSerializer::Some(log_messages) = meta.log_messages {
                            let events = self.parse_transaction_logs(&log_messages);
                            all_events.extend(events);
                        }
                    }
                }
            }
        }

        Ok(all_events)
    }

    /// Parse a single log entry to extract an event
    fn parse_log_entry(&self, log: &str) -> Option<EscrowEvent> {
        // This is a simplified parser - in a real implementation you'd parse
        // the actual event data structure from the contract logs

        if log.contains("Program log: UserCreated") {
            // Parse user created event
            // Format: "Program log: UserCreated { user: ..., username: ..., authority: ... }"
            self.parse_user_created(log)
        } else if log.contains("Program log: TeamCreated") {
            self.parse_team_created(log)
        } else if log.contains("Program log: JobCreated") {
            self.parse_job_created(log)
        } else if log.contains("Program log: JobFunded") {
            self.parse_job_funded(log)
        } else if log.contains("Program log: DisputeRaised") {
            self.parse_dispute_raised(log)
        } else if log.contains("Program log: MilestoneCreated") {
            self.parse_milestone_created(log)
        } else {
            None
        }
    }

    fn parse_user_created(&self, _log: &str) -> Option<EscrowEvent> {
        // Placeholder implementation - would parse actual event data
        None
    }

    fn parse_team_created(&self, _log: &str) -> Option<EscrowEvent> {
        // Placeholder implementation - would parse actual event data
        None
    }

    fn parse_job_created(&self, _log: &str) -> Option<EscrowEvent> {
        // Placeholder implementation - would parse actual event data
        None
    }

    fn parse_job_funded(&self, _log: &str) -> Option<EscrowEvent> {
        // Placeholder implementation - would parse actual event data
        None
    }

    fn parse_dispute_raised(&self, _log: &str) -> Option<EscrowEvent> {
        // Placeholder implementation - would parse actual event data
        None
    }

    fn parse_milestone_created(&self, _log: &str) -> Option<EscrowEvent> {
        // Placeholder implementation - would parse actual event data
        None
    }
}

/// Event subscription handle
pub struct EventSubscription {
    receiver: mpsc::UnboundedReceiver<EscrowEvent>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventSubscription {
    /// Receive the next event (blocking)
    pub async fn recv(&mut self) -> Option<EscrowEvent> {
        self.receiver.recv().await
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<EscrowEvent> {
        self.receiver
            .try_recv()
            .map_err(|e| EscrowError::sdk_error(&format!("Failed to receive event: {}", e)))
    }

    /// Close the subscription
    pub fn close(self) {
        drop(self);
    }
}

/// Event filter for subscribing to specific types of events
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by specific accounts
    pub accounts: Option<Vec<Pubkey>>,
    /// Filter by specific users
    pub users: Option<Vec<Pubkey>>,
}

impl EventFilter {
    /// Create a new empty filter (matches all events)
    pub fn new() -> Self {
        Self {
            event_types: None,
            accounts: None,
            users: None,
        }
    }

    /// Filter by event types
    pub fn with_event_types(mut self, types: Vec<String>) -> Self {
        self.event_types = Some(types);
        self
    }

    /// Filter by specific accounts
    pub fn with_accounts(mut self, accounts: Vec<Pubkey>) -> Self {
        self.accounts = Some(accounts);
        self
    }

    /// Filter by specific users
    pub fn with_users(mut self, users: Vec<Pubkey>) -> Self {
        self.users = Some(users);
        self
    }

    /// Check if an event matches this filter
    pub fn matches(&self, event: &EscrowEvent) -> bool {
        // Implement filtering logic based on event type and associated accounts
        // This is a simplified version
        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filter_creation() {
        let filter = EventFilter::new();
        assert!(filter.event_types.is_none());
        assert!(filter.accounts.is_none());
        assert!(filter.users.is_none());
    }

    #[test]
    fn test_event_listener_config() {
        let config = EventListenerConfig::default();
        assert_eq!(config.polling_interval, Duration::from_millis(1000));
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_event_parsing() {
        let rpc = Arc::new(RpcClient::new("https://api.devnet.solana.com"));
        let config = EventListenerConfig::default();
        let listener = EventListener::new(rpc, config);

        // Test parsing empty logs
        let events = listener.parse_transaction_logs(&[]);
        assert!(events.is_empty());
    }
}

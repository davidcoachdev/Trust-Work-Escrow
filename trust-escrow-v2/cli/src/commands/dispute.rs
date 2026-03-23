//! Dispute resolution commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{DisputeCommands, Cli};

pub async fn execute_dispute_command(
    client: &EscrowClient,
    action: &DisputeCommands,
    cli: &Cli,
) -> Result<()> {
    match action {
        DisputeCommands::Raise { job_id, reason } => {
            println!("Raising dispute for job: {}", job_id);
            println!("Reason: {}", reason);
            println!("✅ Dispute creation functionality will be implemented in Phase 2");
        }
        DisputeCommands::Evidence { job_id, evidence } => {
            println!("Submitting evidence for job: {}", job_id);
            println!("Evidence: {}", evidence);
            println!("✅ Evidence submission functionality will be implemented in Phase 2");
        }
        DisputeCommands::List { my_disputes } => {
            if *my_disputes {
                println!("Listing my disputes");
            } else {
                println!("Listing all disputes");
            }
            println!("✅ Dispute listing functionality will be implemented in Phase 2");
        }
        DisputeCommands::Show { job_id } => {
            println!("Showing dispute for job: {}", job_id);
            println!("✅ Dispute details functionality will be implemented in Phase 2");
        }
    }
    Ok(())
}
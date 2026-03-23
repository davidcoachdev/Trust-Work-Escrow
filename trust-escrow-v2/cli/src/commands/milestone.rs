//! Milestone management commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{MilestoneCommands, Cli};

pub async fn execute_milestone_command(
    client: &EscrowClient,
    action: &MilestoneCommands,
    cli: &Cli,
) -> Result<()> {
    match action {
        MilestoneCommands::Create { job_id, description, amount } => {
            println!("Creating milestone for job: {}", job_id);
            println!("Description: {}", description);
            println!("Amount: {} SOL", amount);
            println!("✅ Milestone creation functionality will be implemented in Phase 2");
        }
        MilestoneCommands::List { job_id } => {
            println!("Listing milestones for job: {}", job_id);
            println!("✅ Milestone listing functionality will be implemented in Phase 2");
        }
        MilestoneCommands::Submit { job_id, milestone_id, details } => {
            println!("Submitting milestone {} for job: {}", milestone_id, job_id);
            println!("Details: {}", details);
            println!("✅ Milestone submission functionality will be implemented in Phase 2");
        }
        MilestoneCommands::Approve { job_id, milestone_id } => {
            println!("Approving milestone {} for job: {}", milestone_id, job_id);
            println!("✅ Milestone approval functionality will be implemented in Phase 2");
        }
        MilestoneCommands::Reject { job_id, milestone_id, reason } => {
            println!("Rejecting milestone {} for job: {}", milestone_id, job_id);
            println!("Reason: {}", reason);
            println!("✅ Milestone rejection functionality will be implemented in Phase 2");
        }
    }
    Ok(())
}
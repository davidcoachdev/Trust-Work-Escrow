//! Milestone management commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{MilestoneCommands, Cli};
use solana_sdk::pubkey::Pubkey;
use tabled::{Table, Tabled};
use tokio::time::{sleep, Duration};

#[derive(Tabled)]
struct MilestoneDisplay {
    field: String,
    value: String,
}

#[derive(Tabled)]
struct MilestoneListItem {
    milestone_id: String,
    title: String,
    amount: String,
    status: String,
    deadline: String,
}

pub async fn execute_milestone_command(
    client: &EscrowClient,
    action: &MilestoneCommands,
    cli: &Cli,
) -> Result<()> {
    // Check if SDK client is available
    let sdk_client = client.sdk()
        .ok_or_else(|| anyhow::anyhow!("No wallet configured. Use --wallet or configure a wallet first."))?;

    match action {
        MilestoneCommands::Create { job_id, description, amount } => {
            println!("🔄 Creating milestone for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if description.trim().is_empty() {
                return Err(anyhow::anyhow!("Milestone description cannot be empty"));
            }

            if description.len() > 1000 {
                return Err(anyhow::anyhow!("Milestone description cannot exceed 1000 characters"));
            }

            if *amount <= 0.0 {
                return Err(anyhow::anyhow!("Milestone amount must be greater than 0"));
            }

            // Convert SOL to lamports
            let amount_lamports = (*amount * 1_000_000_000.0) as u64;
            
            // Check if job can accept milestones
            print!("🔍 Checking job status... ");
            let can_create = sdk_client.can_create_milestones(job_id_num).await
                .map_err(|e| anyhow::anyhow!("Failed to check job status: {}", e))?;
            
            if !can_create {
                return Err(anyhow::anyhow!("Cannot create milestones for this job. Job must be in 'Created' or 'Applications Open' status."));
            }
            println!("✅");

            // Find next milestone index (simple approach - in production this would be more sophisticated)
            let milestone_index = chrono::Utc::now().timestamp() as u8 % 20; // Max 20 milestones

            // Create milestone using SDK
            print!("📋 Creating milestone... ");
            let (milestone_pda, signature) = sdk_client.create_milestone(
                job_id_num,
                "Milestone", // Simple title, could be configurable
                description,
                amount_lamports,
                milestone_index,
            ).await.map_err(|e| anyhow::anyhow!("Failed to create milestone: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "milestone_index": milestone_index,
                    "milestone_pda": milestone_pda.to_string(),
                    "transaction": signature.to_string(),
                    "description": description,
                    "amount": amount
                }));
            } else {
                println!("\n✅ Milestone created successfully!");
                let milestone_data = vec![
                    MilestoneDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    MilestoneDisplay { field: "Milestone Index".to_string(), value: milestone_index.to_string() },
                    MilestoneDisplay { field: "Milestone PDA".to_string(), value: milestone_pda.to_string() },
                    MilestoneDisplay { field: "Description".to_string(), value: description.clone() },
                    MilestoneDisplay { field: "Amount".to_string(), value: format!("{} SOL", amount) },
                    MilestoneDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    MilestoneDisplay { field: "Status".to_string(), value: "Created".to_string() },
                ];

                println!("{}", Table::new(milestone_data));
                println!("\n💡 Freelancer can now submit work for this milestone");
            }
        }

        MilestoneCommands::List { job_id } => {
            println!("🔍 Listing milestones for job: {}", job_id);
            
            let _job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            // Note: SDK doesn't currently have list_milestones method, so we'll provide a helpful message
            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "not_implemented",
                    "message": "Milestone listing not yet implemented in SDK",
                    "job_id": job_id,
                    "suggestion": "Use individual milestone operations for now"
                }));
            } else {
                println!("📭 Milestone listing not yet implemented in SDK");
                println!("💡 You can interact with individual milestones using their index numbers");
                println!("💡 Use 'trust-escrow milestone submit/approve/reject' commands");
            }
        }

        MilestoneCommands::Submit { job_id, milestone_id, details: _ } => {
            println!("🔄 Submitting milestone {} for job: {}", milestone_id, job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            let milestone_index = *milestone_id as u8;

            // Submit milestone using SDK
            print!("📤 Submitting milestone work... ");
            let signature = sdk_client.submit_milestone(job_id_num, milestone_index).await
                .map_err(|e| anyhow::anyhow!("Failed to submit milestone: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "milestone_id": milestone_id,
                    "transaction": signature.to_string()
                }));
            } else {
                println!("\n✅ Milestone work submitted successfully!");
                let submit_data = vec![
                    MilestoneDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    MilestoneDisplay { field: "Milestone ID".to_string(), value: milestone_id.to_string() },
                    MilestoneDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    MilestoneDisplay { field: "Status".to_string(), value: "Submitted - Pending Review".to_string() },
                ];

                println!("{}", Table::new(submit_data));
                println!("\n💡 Wait for the client to review your milestone submission");
            }
        }

        MilestoneCommands::Approve { job_id, milestone_id } => {
            println!("🔄 Approving milestone {} for job: {}", milestone_id, job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            let milestone_index = *milestone_id as u8;

            // Get job to find freelancer
            print!("🔍 Getting job details... ");
            let job = sdk_client.get_escrow(job_id_num).await
                .map_err(|e| anyhow::anyhow!("Failed to fetch job: {}", e))?;
            
            let freelancer = job.freelancer
                .ok_or_else(|| anyhow::anyhow!("No freelancer assigned to this job"))?;
            println!("✅");

            // Approve milestone using SDK
            print!("✅ Approving milestone and releasing payment... ");
            let signature = sdk_client.approve_milestone(job_id_num, milestone_index, &freelancer).await
                .map_err(|e| anyhow::anyhow!("Failed to approve milestone: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "milestone_id": milestone_id,
                    "freelancer": freelancer.to_string(),
                    "transaction": signature.to_string()
                }));
            } else {
                println!("\n✅ Milestone approved and payment released!");
                let approve_data = vec![
                    MilestoneDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    MilestoneDisplay { field: "Milestone ID".to_string(), value: milestone_id.to_string() },
                    MilestoneDisplay { field: "Freelancer".to_string(), value: freelancer.to_string() },
                    MilestoneDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    MilestoneDisplay { field: "Status".to_string(), value: "Approved - Payment Sent".to_string() },
                ];

                println!("{}", Table::new(approve_data));
                println!("\n💰 Payment has been sent to the freelancer");
            }
        }

        MilestoneCommands::Reject { job_id, milestone_id, reason } => {
            println!("🔄 Rejecting milestone {} for job: {}", milestone_id, job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            let milestone_index = *milestone_id as u8;

            if reason.trim().is_empty() {
                return Err(anyhow::anyhow!("Rejection reason cannot be empty"));
            }

            if reason.len() > 500 {
                return Err(anyhow::anyhow!("Rejection reason cannot exceed 500 characters"));
            }

            // Reject milestone using SDK
            print!("❌ Rejecting milestone... ");
            let signature = sdk_client.reject_milestone(job_id_num, milestone_index).await
                .map_err(|e| anyhow::anyhow!("Failed to reject milestone: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "milestone_id": milestone_id,
                    "reason": reason,
                    "transaction": signature.to_string()
                }));
            } else {
                println!("\n❌ Milestone rejected!");
                let reject_data = vec![
                    MilestoneDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    MilestoneDisplay { field: "Milestone ID".to_string(), value: milestone_id.to_string() },
                    MilestoneDisplay { field: "Reason".to_string(), value: reason.clone() },
                    MilestoneDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    MilestoneDisplay { field: "Status".to_string(), value: "Rejected - Needs Revision".to_string() },
                ];

                println!("{}", Table::new(reject_data));
                println!("\n💡 The freelancer can revise and resubmit this milestone");
            }
        }
    }
    Ok(())
}
//! Dispute resolution commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{DisputeCommands, Cli};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct DisputeDisplay {
    field: String,
    value: String,
}

#[derive(Tabled)]
struct DisputeListItem {
    job_id: String,
    status: String,
    raised_by: String,
    evidence_count: String,
    deadline: String,
}

pub async fn execute_dispute_command(
    client: &EscrowClient,
    action: &DisputeCommands,
    cli: &Cli,
) -> Result<()> {
    // Check if SDK client is available
    let sdk_client = client.sdk()
        .ok_or_else(|| anyhow::anyhow!("No wallet configured. Use --wallet or configure a wallet first."))?;

    match action {
        DisputeCommands::Raise { job_id, reason } => {
            println!("🚨 Raising dispute for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if reason.trim().is_empty() {
                return Err(anyhow::anyhow!("Dispute reason cannot be empty"));
            }

            if reason.len() > 2048 {
                return Err(anyhow::anyhow!("Dispute reason cannot exceed 2048 characters"));
            }

            // Check job status first
            print!("🔍 Verifying job status... ");
            let job = sdk_client.get_escrow(job_id_num).await
                .map_err(|e| anyhow::anyhow!("Failed to fetch job: {}", e))?;
                
            println!("✅");

            // Raise dispute using SDK
            print!("⚖️  Creating dispute... ");
            let (dispute_pda, signature) = sdk_client.raise_dispute(job_id_num, reason).await
                .map_err(|e| anyhow::anyhow!("Failed to raise dispute: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "dispute_pda": dispute_pda.to_string(),
                    "transaction": signature.to_string(),
                    "reason": reason,
                    "raised_by": client.wallet_pubkey().unwrap().to_string()
                }));
            } else {
                println!("\n🚨 Dispute raised successfully!");
                let dispute_data = vec![
                    DisputeDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    DisputeDisplay { field: "Dispute PDA".to_string(), value: dispute_pda.to_string() },
                    DisputeDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    DisputeDisplay { field: "Raised by".to_string(), value: client.wallet_pubkey().unwrap().to_string() },
                    DisputeDisplay { field: "Reason".to_string(), value: reason.clone() },
                    DisputeDisplay { field: "Status".to_string(), value: "Open - Awaiting Arbiter".to_string() },
                    DisputeDisplay { field: "Deadline".to_string(), value: "7 days (default)".to_string() },
                ];

                println!("{}", Table::new(dispute_data));
                
                println!("\n📋 Dispute Process:");
                println!("  1. ✅ Dispute created and recorded on-chain");
                println!("  2. 🔄 An arbiter will be assigned");
                println!("  3. 📝 Both parties can submit additional evidence");
                println!("  4. ⚖️  Arbiter will review and resolve the dispute");
                
                println!("\n💡 Next steps:");
                println!("  • Submit additional evidence: trust-escrow dispute evidence {} \"<evidence>\"", job_id);
                println!("  • Check dispute status: trust-escrow dispute show {}", job_id);
            }
        }

        DisputeCommands::Evidence { job_id, evidence } => {
            println!("📝 Submitting evidence for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if evidence.trim().is_empty() {
                return Err(anyhow::anyhow!("Evidence cannot be empty"));
            }

            if evidence.len() > 2048 {
                return Err(anyhow::anyhow!("Evidence cannot exceed 2048 characters"));
            }

            // Submit evidence using SDK
            print!("📤 Submitting evidence to dispute... ");
            let signature = sdk_client.submit_evidence(job_id_num, evidence).await
                .map_err(|e| anyhow::anyhow!("Failed to submit evidence: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string(),
                    "evidence": evidence,
                    "submitted_by": client.wallet_pubkey().unwrap().to_string()
                }));
            } else {
                println!("\n📝 Evidence submitted successfully!");
                let evidence_data = vec![
                    DisputeDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    DisputeDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    DisputeDisplay { field: "Submitted by".to_string(), value: client.wallet_pubkey().unwrap().to_string() },
                    DisputeDisplay { field: "Evidence".to_string(), value: evidence.clone() },
                    DisputeDisplay { field: "Status".to_string(), value: "Evidence Recorded".to_string() },
                ];

                println!("{}", Table::new(evidence_data));
                println!("\n💡 Evidence has been added to the dispute record");
                println!("💡 The arbiter will consider all evidence when making a decision");
            }
        }

        DisputeCommands::List { my_disputes } => {
            println!("🔍 {} disputes...", if *my_disputes { "Fetching your" } else { "Fetching all" });
            
            let wallet_pubkey = client.wallet_pubkey()
                .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

            // Note: SDK doesn't have a direct dispute listing method, so we'll simulate
            // In production, this would iterate through dispute PDAs or use an indexer

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "not_implemented",
                    "message": "Dispute listing not yet fully implemented in SDK",
                    "wallet": wallet_pubkey.to_string(),
                    "filter_my_disputes": my_disputes,
                    "suggestion": "Use 'trust-escrow dispute show <job_id>' for specific disputes"
                }));
            } else {
                println!("📭 Dispute listing not yet fully implemented in SDK");
                println!("\n💡 Available dispute operations:");
                println!("  • trust-escrow dispute show <job_id>     - Show dispute details");
                println!("  • trust-escrow dispute raise <job_id>    - Create new dispute");
                println!("  • trust-escrow dispute evidence <job_id> - Submit evidence");
                
                println!("\n🔍 To check if a job has disputes, use:");
                println!("  trust-escrow job show <job_id>");
                println!("  (Look for status: 'Disputed')");
            }
        }

        DisputeCommands::Show { job_id } => {
            println!("🔍 Fetching dispute details for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            // Get job first to see if it's disputed
            print!("📋 Checking job status... ");
            let job = sdk_client.get_escrow(job_id_num).await
                .map_err(|e| anyhow::anyhow!("Failed to fetch job: {}", e))?;
            
            println!("✅");

            // Check if job is in dispute status
            let job_status = sdk_client.format_job_status(job.status);
            
            if !job_status.to_lowercase().contains("disput") {
                if cli.output == "json" {
                    println!("{}", serde_json::json!({
                        "status": "no_dispute",
                        "job_id": job_id,
                        "job_status": job_status,
                        "message": "This job does not have an active dispute"
                    }));
                } else {
                    println!("ℹ️  No active dispute found for job {}", job_id);
                    println!("📊 Current job status: {}", job_status);
                    println!("\n💡 To raise a dispute:");
                    println!("  trust-escrow dispute raise {} \"<reason>\"", job_id);
                }
                return Ok(());
            }

            // Try to get dispute data (using SDK client methods)
            let client_pubkey = job.client;
            let dispute_pda_result = trust_escrow_sdk::pda::derive_dispute_pda(&trust_escrow_sdk::pda::derive_job_pda(&client_pubkey, job_id_num).unwrap().0);

            match dispute_pda_result {
                Ok((dispute_pda, _)) => {
                    // Try to get dispute data
                    match sdk_client.get_dispute(&dispute_pda).await {
                        Ok(dispute) => {
                            if cli.output == "json" {
                                println!("{}", serde_json::json!({
                                    "job_id": job_id,
                                    "dispute_pda": dispute_pda.to_string(),
                                    "status": "disputed",
                                    "raised_by": dispute.raised_by.to_string(),
                                    "created_at": dispute.created_at,
                                    "resolved_at": dispute.resolved_at,
                                    "evidence_count": dispute.evidence.len()
                                }));
                            } else {
                                let dispute_data = vec![
                                    DisputeDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                                    DisputeDisplay { field: "Dispute PDA".to_string(), value: dispute_pda.to_string() },
                                    DisputeDisplay { field: "Status".to_string(), value: "Active Dispute".to_string() },
                                    DisputeDisplay { field: "Raised by".to_string(), value: dispute.raised_by.to_string() },
                                    DisputeDisplay { field: "Created".to_string(), value: chrono::DateTime::from_timestamp(dispute.created_at, 0)
                                        .map_or("Invalid".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()) },
                                    DisputeDisplay { field: "Created".to_string(), value: chrono::DateTime::from_timestamp(dispute.created_at, 0)
                                        .map_or("Invalid".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()) },
                                    DisputeDisplay { field: "Evidence Count".to_string(), value: dispute.evidence.len().to_string() },
                                ];

                                println!("\n⚖️  Dispute Details");
                                println!("{}", Table::new(dispute_data));
                                
                                println!("\n📋 Dispute Timeline:");
                                println!("  • Dispute raised and recorded on-chain");
                if dispute.evidence.len() > 0 {
                    println!("  • {} evidence submission(s) received", dispute.evidence.len());
                                }
                                println!("  • Awaiting arbiter assignment and resolution");
                                
                                println!("\n💡 Available actions:");
                                println!("  • Submit additional evidence");
                                println!("  • Wait for arbiter decision");
                            }
                        }
                        Err(_) => {
                            if cli.output == "json" {
                                println!("{}", serde_json::json!({
                                    "status": "error",
                                    "job_id": job_id,
                                    "message": "Could not fetch dispute data",
                                    "dispute_pda": dispute_pda.to_string()
                                }));
                            } else {
                                println!("❌ Could not fetch dispute data");
                                println!("💡 The dispute PDA might not be initialized yet");
                                println!("💡 Try checking again in a few moments");
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to derive dispute PDA: {}", e));
                }
            }
        }
    }
    Ok(())
}
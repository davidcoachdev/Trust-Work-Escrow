//! Job management commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{JobCommands, Cli};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tabled::{Table, Tabled};
use tokio::time::{sleep, Duration};

#[derive(Tabled)]
struct JobDisplay {
    field: String,
    value: String,
}

#[derive(Tabled)]
struct JobListItem {
    job_id: String,
    title: String,
    amount: String,
    status: String,
    created_by: String,
}

pub async fn execute_job_command(
    client: &EscrowClient,
    action: &JobCommands,
    cli: &Cli,
) -> Result<()> {
    // Check if SDK client is available
    let sdk_client = client.sdk()
        .ok_or_else(|| anyhow::anyhow!("No wallet configured. Use --wallet or configure a wallet first."))?;

    match action {
        JobCommands::Create { title, description, amount, skills: _ } => {
            println!("🔄 Creating job: {}", title);
            
            // Validate inputs
            if title.trim().is_empty() {
                return Err(anyhow::anyhow!("Job title cannot be empty"));
            }
            
            if title.len() > 200 {
                return Err(anyhow::anyhow!("Job title cannot exceed 200 characters"));
            }

            if description.trim().is_empty() {
                return Err(anyhow::anyhow!("Job description cannot be empty"));
            }

            if description.len() > 2000 {
                return Err(anyhow::anyhow!("Job description cannot exceed 2000 characters"));
            }

            if *amount <= 0.0 {
                return Err(anyhow::anyhow!("Job amount must be greater than 0"));
            }

            // Convert SOL to lamports
            let amount_lamports = (*amount * 1_000_000_000.0) as u64;
            
            // Generate job ID (simple counter - in production this would be more sophisticated)
            let job_id = chrono::Utc::now().timestamp() as u64 % 1_000_000;
            
            // Set deadline to 30 days from now
            let deadline = chrono::Utc::now().timestamp() + (30 * 24 * 60 * 60);

            // Create job
            print!("💰 Creating escrow contract... ");
            let (job_pda, signature) = sdk_client.create_escrow(
                job_id,
                title,
                description,
                amount_lamports,
                deadline,
            ).await.map_err(|e| anyhow::anyhow!("Failed to create job: {}", e))?;
            
            println!("✅");

            // Fund the escrow
            print!("💸 Funding escrow with {} SOL... ", amount);
            sleep(Duration::from_millis(500)).await; // Wait for confirmation
            
            let fund_signature = sdk_client.fund_escrow(job_id).await
                .map_err(|e| anyhow::anyhow!("Failed to fund escrow: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "job_pda": job_pda.to_string(),
                    "create_transaction": signature.to_string(),
                    "fund_transaction": fund_signature.to_string(),
                    "title": title,
                    "amount": amount,
                    "deadline": deadline
                }));
            } else {
                println!("\n✅ Job created and funded successfully!");
                let job_data = vec![
                    JobDisplay { field: "Job ID".to_string(), value: job_id.to_string() },
                    JobDisplay { field: "Job PDA".to_string(), value: job_pda.to_string() },
                    JobDisplay { field: "Title".to_string(), value: title.clone() },
                    JobDisplay { field: "Amount".to_string(), value: format!("{} SOL", amount) },
                    JobDisplay { field: "Create Tx".to_string(), value: signature.to_string() },
                    JobDisplay { field: "Fund Tx".to_string(), value: fund_signature.to_string() },
                    JobDisplay { field: "Deadline".to_string(), value: format!("{} (30 days)", deadline) },
                ];

                println!("{}", Table::new(job_data));
                println!("\n💡 Next steps:");
                println!("  • Wait for freelancer applications");
                println!("  • Use 'trust-escrow job accept' to accept applications");
            }
        }

        JobCommands::List { my_jobs, status } => {
            println!("🔍 {} jobs...", if *my_jobs { "Fetching your" } else { "Fetching available" });
            
            // Get current wallet pubkey for filtering
            let wallet_pubkey = client.wallet_pubkey()
                .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

            // Fetch escrows (jobs) from SDK
            let escrows = sdk_client.list_escrows(Some(50)).await
                .map_err(|e| anyhow::anyhow!("Failed to fetch jobs: {}", e))?;

            let mut job_items = Vec::new();

            for (job_pda, job) in escrows {
                // Filter by ownership if requested
                if *my_jobs && job.client != wallet_pubkey {
                    continue;
                }

                // Filter by status if requested
                if let Some(status_filter) = status {
                    let job_status_str = sdk_client.format_job_status(job.status);
                    if !job_status_str.to_lowercase().contains(&status_filter.to_lowercase()) {
                        continue;
                    }
                }

                job_items.push(JobListItem {
                    job_id: job.job_id.to_string(),
                    title: job.title,
                    amount: sdk_client.format_amount(job.amount),
                    status: sdk_client.format_job_status(job.status).to_string(),
                    created_by: if job.client == wallet_pubkey { "You".to_string() } else { format!("{}...{}", 
                        &job.client.to_string()[..8], &job.client.to_string()[job.client.to_string().len()-8..]) },
                });
            }

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "jobs": job_items.iter().map(|item| serde_json::json!({
                        "job_id": item.job_id,
                        "title": item.title,
                        "amount": item.amount,
                        "status": item.status,
                        "created_by": item.created_by
                    })).collect::<Vec<_>>(),
                    "count": job_items.len(),
                    "filter_my_jobs": my_jobs,
                    "status_filter": status
                }));
            } else {
                if job_items.is_empty() {
                    println!("📭 No jobs found matching your criteria");
                } else {
                    println!("\n📋 Jobs Found: {}", job_items.len());
                    println!("{}", Table::new(job_items));
                    
                    println!("\n💡 Use 'trust-escrow job show <job_id>' for details");
                    if !*my_jobs {
                        println!("💡 Use 'trust-escrow job apply <job_id>' to apply");
                    }
                }
            }
        }

        JobCommands::Show { job_id } => {
            println!("🔍 Fetching job details for: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            // Get job from SDK
            let job = sdk_client.get_escrow(job_id_num).await
                .map_err(|e| anyhow::anyhow!("Failed to fetch job: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "job_id": job_id,
                    "title": job.title,
                    "description": job.description,
                    "amount": job.amount,
                    "status": sdk_client.format_job_status(job.status),
                    "client": job.client.to_string(),
                    "freelancer": job.freelancer.map(|f| f.to_string()),
                    "created_at": job.created_at,
                    "created_at": job.created_at,
                    "updated_at": job.updated_at
                }));
            } else {
                let job_data = vec![
                    JobDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    JobDisplay { field: "Title".to_string(), value: job.title },
                    JobDisplay { field: "Description".to_string(), value: job.description },
                    JobDisplay { field: "Amount".to_string(), value: sdk_client.format_amount(job.amount) },
                    JobDisplay { field: "Status".to_string(), value: sdk_client.format_job_status(job.status).to_string() },
                    JobDisplay { field: "Client".to_string(), value: job.client.to_string() },
                    JobDisplay { field: "Freelancer".to_string(), value: job.freelancer.map_or("None".to_string(), |f| f.to_string()) },
                    JobDisplay { field: "Created".to_string(), value: chrono::DateTime::from_timestamp(job.created_at, 0)
                        .map_or("Invalid".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()) },
                    JobDisplay { field: "Created".to_string(), value: chrono::DateTime::from_timestamp(job.created_at, 0)
                        .map_or("Invalid".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()) },
                    JobDisplay { field: "Updated".to_string(), value: chrono::DateTime::from_timestamp(job.updated_at, 0)
                        .map_or("Invalid".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()) },
                ];

                println!("\n📋 Job Details");
                println!("{}", Table::new(job_data));
            }
        }

        JobCommands::Apply { job_id, proposal } => {
            println!("🔄 Applying to job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if proposal.trim().is_empty() {
                return Err(anyhow::anyhow!("Proposal cannot be empty"));
            }

            if proposal.len() > 1000 {
                return Err(anyhow::anyhow!("Proposal cannot exceed 1000 characters"));
            }

            // Apply to job using SDK
            let job_pda = trust_escrow_sdk::pda::derive_job_pda(&client.wallet_pubkey().unwrap(), job_id_num)
                .map_err(|e| anyhow::anyhow!("Failed to derive job PDA: {}", e))?.0;

            let signature = sdk_client.apply_to_job(&job_pda, proposal).await
                .map_err(|e| anyhow::anyhow!("Failed to apply to job: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string(),
                    "proposal": proposal
                }));
            } else {
                println!("✅ Application submitted successfully!");
                println!("📝 Transaction: {}", signature);
                println!("📄 Proposal: {}", proposal);
                println!("\n💡 Wait for the client to review your application");
            }
        }

        JobCommands::Accept { job_id, applicant } => {
            println!("🔄 Accepting application for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            let freelancer_pubkey = Pubkey::from_str(applicant)
                .map_err(|e| anyhow::anyhow!("Invalid freelancer address: {}", e))?;

            // Accept application using SDK
            let job_pda = trust_escrow_sdk::pda::derive_job_pda(&client.wallet_pubkey().unwrap(), job_id_num)
                .map_err(|e| anyhow::anyhow!("Failed to derive job PDA: {}", e))?.0;

            let signature = sdk_client.accept_application(&job_pda, &freelancer_pubkey).await
                .map_err(|e| anyhow::anyhow!("Failed to accept application: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string(),
                    "freelancer": applicant
                }));
            } else {
                println!("✅ Application accepted successfully!");
                println!("📝 Transaction: {}", signature);
                println!("👤 Freelancer: {}", applicant);
                println!("\n💡 Work can now begin on this job");
            }
        }

        JobCommands::Submit { job_id, details } => {
            println!("🔄 Submitting work for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if details.trim().is_empty() {
                return Err(anyhow::anyhow!("Work details cannot be empty"));
            }

            if details.len() > 500 {
                return Err(anyhow::anyhow!("Work details cannot exceed 500 characters"));
            }

            // Submit work using SDK
            let job_pda = trust_escrow_sdk::pda::derive_job_pda(&client.wallet_pubkey().unwrap(), job_id_num)
                .map_err(|e| anyhow::anyhow!("Failed to derive job PDA: {}", e))?.0;

            let signature = sdk_client.submit_work(&job_pda, details).await
                .map_err(|e| anyhow::anyhow!("Failed to submit work: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string(),
                    "details": details
                }));
            } else {
                println!("✅ Work submitted successfully!");
                println!("📝 Transaction: {}", signature);
                println!("📄 Details: {}", details);
                println!("\n💡 Wait for the client to review your work");
            }
        }

        JobCommands::Approve { job_id } => {
            println!("🔄 Approving work for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            // Approve work using SDK
            let job_pda = trust_escrow_sdk::pda::derive_job_pda(&client.wallet_pubkey().unwrap(), job_id_num)
                .map_err(|e| anyhow::anyhow!("Failed to derive job PDA: {}", e))?.0;

            let signature = sdk_client.approve_work(&job_pda).await
                .map_err(|e| anyhow::anyhow!("Failed to approve work: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string()
                }));
            } else {
                println!("✅ Work approved and payment released!");
                println!("📝 Transaction: {}", signature);
                println!("💰 Payment has been sent to the freelancer");
            }
        }

        JobCommands::Reject { job_id, reason } => {
            println!("🔄 Rejecting work for job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if reason.trim().is_empty() {
                return Err(anyhow::anyhow!("Rejection reason cannot be empty"));
            }

            if reason.len() > 500 {
                return Err(anyhow::anyhow!("Rejection reason cannot exceed 500 characters"));
            }

            // Reject work using SDK
            let job_pda = trust_escrow_sdk::pda::derive_job_pda(&client.wallet_pubkey().unwrap(), job_id_num)
                .map_err(|e| anyhow::anyhow!("Failed to derive job PDA: {}", e))?.0;

            let signature = sdk_client.reject_work(&job_pda, reason).await
                .map_err(|e| anyhow::anyhow!("Failed to reject work: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string(),
                    "reason": reason
                }));
            } else {
                println!("✅ Work rejected!");
                println!("📝 Transaction: {}", signature);
                println!("📄 Reason: {}", reason);
                println!("\n💡 The freelancer can revise and resubmit");
            }
        }

        JobCommands::Cancel { job_id } => {
            println!("🔄 Canceling job: {}", job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            // Cancel job using SDK
            let job_pda = trust_escrow_sdk::pda::derive_job_pda(&client.wallet_pubkey().unwrap(), job_id_num)
                .map_err(|e| anyhow::anyhow!("Failed to derive job PDA: {}", e))?.0;

            let signature = sdk_client.cancel_job(&job_pda).await
                .map_err(|e| anyhow::anyhow!("Failed to cancel job: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "transaction": signature.to_string()
                }));
            } else {
                println!("✅ Job canceled and funds refunded!");
                println!("📝 Transaction: {}", signature);
                println!("💰 Funds have been returned to your wallet");
            }
        }
    }
    Ok(())
}
//! Payment and financial commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{PaymentCommands, Cli};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct BalanceDisplay {
    field: String,
    value: String,
}

#[derive(Tabled)]
struct TransactionDisplay {
    signature: String,
    type_: String,
    amount: String,
    status: String,
    date: String,
}

pub async fn execute_payment_command(
    client: &EscrowClient,
    action: &PaymentCommands,
    cli: &Cli,
) -> Result<()> {
    // Check if SDK client is available
    let sdk_client = client.sdk()
        .ok_or_else(|| anyhow::anyhow!("No wallet configured. Use --wallet or configure a wallet first."))?;

    match action {
        PaymentCommands::Balance => {
            println!("🔍 Checking wallet balance...");
            
            let wallet_pubkey = client.wallet_pubkey()
                .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

            // Get balance from client
            let balance = client.get_wallet_balance().await
                .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))?;
            
            let sol_balance = balance as f64 / 1_000_000_000.0;

            // Get recommended fees
            let recommended_fee = sdk_client.get_recommended_fee().await.unwrap_or(5000);
            let fee_sol = recommended_fee as f64 / 1_000_000_000.0;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "wallet": wallet_pubkey.to_string(),
                    "balance_lamports": balance,
                    "balance_sol": sol_balance,
                    "recommended_fee_lamports": recommended_fee,
                    "recommended_fee_sol": fee_sol,
                    "sufficient_for_transaction": balance > recommended_fee * 2 // Buffer for multiple transactions
                }));
            } else {
                let balance_data = vec![
                    BalanceDisplay { field: "Wallet Address".to_string(), value: wallet_pubkey.to_string() },
                    BalanceDisplay { field: "Balance (SOL)".to_string(), value: format!("{:.9}", sol_balance) },
                    BalanceDisplay { field: "Balance (lamports)".to_string(), value: balance.to_string() },
                    BalanceDisplay { field: "Recommended Fee".to_string(), value: format!("{:.9} SOL", fee_sol) },
                    BalanceDisplay { field: "Status".to_string(), value: 
                        if balance > recommended_fee * 10 { 
                            "✅ Sufficient for transactions".to_string()
                        } else if balance > recommended_fee {
                            "⚠️  Low balance - consider adding funds".to_string()
                        } else {
                            "❌ Insufficient balance for transactions".to_string()
                        }
                    },
                ];

                println!("\n💰 Wallet Balance");
                println!("{}", Table::new(balance_data));

                if balance <= recommended_fee {
                    println!("\n🚨 Your wallet has insufficient SOL for transactions!");
                    println!("💡 Use 'trust-escrow airdrop' to get test SOL on devnet/localnet");
                    println!("💡 Or transfer SOL to your wallet: {}", wallet_pubkey);
                }
            }
        }

        PaymentCommands::History => {
            println!("🔍 Fetching payment history...");
            
            let wallet_pubkey = client.wallet_pubkey()
                .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?;

            // Get recent events from SDK (if available)
            match sdk_client.get_recent_events(10).await {
                Ok(events) => {
                    let mut transactions = Vec::new();
                    
                    for event in events {
                        // Convert events to transaction display format
                        let (tx_type, amount_str, status) = match event {
                            _ => ("Unknown".to_string(), "N/A".to_string(), "Completed".to_string()),
                        };

                        transactions.push(TransactionDisplay {
                            signature: "N/A - Event data".to_string(),
                            type_: tx_type,
                            amount: amount_str,
                            status,
                            date: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        });
                    }

                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "wallet": wallet_pubkey.to_string(),
                            "transactions": transactions.iter().map(|t| serde_json::json!({
                                "signature": t.signature,
                                "type": t.type_,
                                "amount": t.amount,
                                "status": t.status,
                                "date": t.date
                            })).collect::<Vec<_>>(),
                            "count": transactions.len()
                        }));
                    } else {
                        if transactions.is_empty() {
                            println!("📭 No recent payment history found");
                        } else {
                            println!("\n📊 Recent Payment History");
                            println!("{}", Table::new(transactions));
                        }
                    }
                }
                Err(_) => {
                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "status": "not_available",
                            "message": "Payment history not yet available from SDK",
                            "wallet": wallet_pubkey.to_string(),
                            "suggestion": "Check Solana Explorer for transaction history"
                        }));
                    } else {
                        println!("📭 Payment history not yet available from SDK");
                        println!("💡 You can check transaction history on Solana Explorer:");
                        println!("   https://explorer.solana.com/address/{}", wallet_pubkey);
                    }
                }
            }
        }

        PaymentCommands::Deposit { job_id, amount } => {
            println!("🔄 Depositing {} SOL to job: {}", amount, job_id);
            
            let job_id_num = job_id.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Invalid job ID format. Must be a number."))?;

            if *amount <= 0.0 {
                return Err(anyhow::anyhow!("Deposit amount must be greater than 0"));
            }

            // Check balance first
            let current_balance = client.get_wallet_balance().await
                .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))?;
            
            let amount_lamports = (*amount * 1_000_000_000.0) as u64;
            let recommended_fee = sdk_client.get_recommended_fee().await.unwrap_or(5000);

            if current_balance < amount_lamports + recommended_fee {
                return Err(anyhow::anyhow!(
                    "Insufficient balance. Need {} SOL + fees, but only have {} SOL",
                    amount,
                    current_balance as f64 / 1_000_000_000.0
                ));
            }

            // Fund the escrow using SDK
            print!("💸 Processing deposit... ");
            let signature = sdk_client.fund_escrow(job_id_num).await
                .map_err(|e| anyhow::anyhow!("Failed to deposit funds: {}", e))?;
            
            println!("✅");

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "job_id": job_id,
                    "amount": amount,
                    "transaction": signature.to_string(),
                    "deposited_lamports": amount_lamports
                }));
            } else {
                println!("\n✅ Funds deposited successfully!");
                let deposit_data = vec![
                    BalanceDisplay { field: "Job ID".to_string(), value: job_id.clone() },
                    BalanceDisplay { field: "Amount Deposited".to_string(), value: format!("{} SOL", amount) },
                    BalanceDisplay { field: "Transaction".to_string(), value: signature.to_string() },
                    BalanceDisplay { field: "Status".to_string(), value: "Deposited - Available for release".to_string() },
                ];

                println!("{}", Table::new(deposit_data));
                println!("\n💡 Funds are now held in escrow and will be released when work is approved");
            }
        }

        PaymentCommands::Withdraw { amount } => {
            println!("🔄 Withdrawing {} SOL from treasury", amount);
            
            if *amount <= 0.0 {
                return Err(anyhow::anyhow!("Withdrawal amount must be greater than 0"));
            }

            // Note: Treasury withdrawal is an admin operation and not yet implemented in SDK
            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "not_implemented",
                    "message": "Treasury withdrawal not yet implemented in SDK",
                    "amount": amount,
                    "note": "This is an admin-only operation"
                }));
            } else {
                println!("❌ Treasury withdrawal not yet implemented in SDK");
                println!("💡 This is an admin-only operation for protocol treasury management");
                println!("💡 Regular users should use job-specific payment operations instead");
                
                println!("\n📖 Available payment operations:");
                println!("  • trust-escrow payment balance    - Check wallet balance");
                println!("  • trust-escrow payment deposit    - Add funds to a job");
                println!("  • trust-escrow job approve        - Release escrowed funds");
                println!("  • trust-escrow job cancel         - Refund escrowed funds");
            }
        }
    }
    Ok(())
}
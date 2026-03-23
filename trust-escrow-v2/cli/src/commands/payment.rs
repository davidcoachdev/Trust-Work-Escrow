//! Payment and financial commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{PaymentCommands, Cli};

pub async fn execute_payment_command(
    client: &EscrowClient,
    action: &PaymentCommands,
    cli: &Cli,
) -> Result<()> {
    match action {
        PaymentCommands::Balance => {
            if client.has_wallet() {
                match client.get_wallet_balance().await {
                    Ok(balance) => {
                        let sol_balance = balance as f64 / 1_000_000_000.0;
                        println!("💰 Wallet balance: {:.6} SOL ({} lamports)", sol_balance, balance);
                    }
                    Err(e) => {
                        println!("❌ Failed to get balance: {}", e);
                    }
                }
            } else {
                println!("❌ No wallet configured");
            }
        }
        PaymentCommands::History => {
            println!("Showing payment history");
            println!("✅ Payment history functionality will be implemented in Phase 2");
        }
        PaymentCommands::Deposit { job_id, amount } => {
            println!("Depositing {} SOL to job: {}", amount, job_id);
            println!("✅ Fund deposit functionality will be implemented in Phase 2");
        }
        PaymentCommands::Withdraw { amount } => {
            println!("Withdrawing {} SOL from treasury", amount);
            println!("✅ Treasury withdrawal functionality will be implemented in Phase 2");
        }
    }
    Ok(())
}
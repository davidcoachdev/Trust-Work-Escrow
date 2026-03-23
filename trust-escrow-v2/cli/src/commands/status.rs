//! Status and connectivity commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::Cli;

pub async fn execute_status_command(
    client: &EscrowClient,
    cli: &Cli,
) -> Result<()> {
    println!("🔍 Checking Trust Work Escrow system status...");
    println!();

    // Network connection
    print!("Network connection... ");
    match client.check_connection().await {
        Ok(_) => println!("✅ Connected"),
        Err(e) => {
            println!("❌ Failed: {}", e);
            return Ok(());
        }
    }

    // Network info
    let config = client.config();
    println!("📡 Network: {} ({})", config.network.cluster, config.network.rpc_url);

    // Current slot
    match client.get_slot().await {
        Ok(slot) => println!("🕐 Current slot: {}", slot),
        Err(e) => println!("❌ Failed to get slot: {}", e),
    }

    // Wallet status
    if client.has_wallet() {
        if let Some(pubkey) = client.wallet_pubkey() {
            println!("🔑 Wallet: {}", pubkey);
            
            match client.get_wallet_balance().await {
                Ok(balance) => {
                    let sol_balance = balance as f64 / 1_000_000_000.0;
                    println!("💰 Balance: {:.6} SOL ({} lamports)", sol_balance, balance);
                }
                Err(e) => println!("❌ Failed to get balance: {}", e),
            }
        }
    } else {
        println!("❌ No wallet configured");
    }

    // Program status
    println!();
    println!("📋 Program: {}", config.programs.trust_escrow_v2);
    
    // Validate client
    match client.validate().await {
        Ok(warnings) => {
            if warnings.is_empty() {
                println!("✅ All systems operational");
            } else {
                println!("⚠️  Warnings:");
                for warning in warnings {
                    println!("   • {}", warning);
                }
            }
        }
        Err(e) => println!("❌ Validation failed: {}", e),
    }

    Ok(())
}
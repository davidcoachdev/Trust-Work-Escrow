//! Airdrop command for devnet/testnet

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::Cli;

pub async fn execute_airdrop_command(
    client: &EscrowClient,
    amount: f64,
    cli: &Cli,
) -> Result<()> {
    let config = client.config();
    
    // Check if airdrop is available on this network
    if config.network.cluster == "mainnet-beta" {
        println!("❌ Airdrops are not available on mainnet");
        return Ok(());
    }
    
    if !client.has_wallet() {
        println!("❌ No wallet configured for airdrop");
        return Ok(());
    }

    let lamports = (amount * 1_000_000_000.0) as u64;
    println!("💧 Requesting {} SOL airdrop to wallet...", amount);
    
    match client.request_airdrop(lamports).await {
        Ok(signature) => {
            println!("✅ Airdrop transaction: {}", signature);
            println!("⏳ Waiting for confirmation...");
            
            // Wait a bit then check balance
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            match client.get_wallet_balance().await {
                Ok(balance) => {
                    let sol_balance = balance as f64 / 1_000_000_000.0;
                    println!("💰 New balance: {:.6} SOL", sol_balance);
                }
                Err(e) => println!("❌ Failed to check new balance: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Airdrop failed: {}", e);
            if e.to_string().contains("rate limit") {
                println!("💡 Try again later - airdrop rate limited");
            }
        }
    }
    
    Ok(())
}
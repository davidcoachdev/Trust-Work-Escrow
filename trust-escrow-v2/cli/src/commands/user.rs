//! User management commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{UserCommands, Cli};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct UserDisplay {
    field: String,
    value: String,
}

pub async fn execute_user_command(
    client: &EscrowClient,
    action: &UserCommands,
    cli: &Cli,
) -> Result<()> {
    // Check if SDK client is available
    let sdk_client = client.sdk()
        .ok_or_else(|| anyhow::anyhow!("No wallet configured. Use --wallet or configure a wallet first."))?;

    match action {
        UserCommands::Create { name, bio } => {
            println!("🔄 Creating user profile: {}", name);
            
            // Validate inputs
            if name.trim().is_empty() {
                return Err(anyhow::anyhow!("User name cannot be empty"));
            }
            
            if name.len() > 32 {
                return Err(anyhow::anyhow!("User name cannot exceed 32 characters"));
            }

            if let Some(bio_text) = bio {
                if bio_text.len() > 500 {
                    return Err(anyhow::anyhow!("Bio cannot exceed 500 characters"));
                }
            }

            // Create user with SDK
            let signature = sdk_client.create_user(name, bio.as_deref()).await
                .map_err(|e| anyhow::anyhow!("Failed to create user: {}", e))?;

            // Display results
            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "transaction": signature.to_string(),
                    "user": name,
                    "bio": bio
                }));
            } else {
                println!("✅ User profile created successfully!");
                println!("📝 Transaction: {}", signature);
                if let Some(bio) = bio {
                    println!("👤 Name: {}", name);
                    println!("📋 Bio: {}", bio);
                } else {
                    println!("👤 Name: {}", name);
                }
            }
        }
        
        UserCommands::Show { address } => {
            let target_address = if let Some(addr) = address {
                Pubkey::from_str(addr)
                    .map_err(|e| anyhow::anyhow!("Invalid address format: {}", e))?
            } else {
                client.wallet_pubkey()
                    .ok_or_else(|| anyhow::anyhow!("No wallet configured"))?
            };

            println!("🔍 Fetching user profile for: {}", target_address);

            // For now, show wallet info since user profile fetching needs implementation
            let balance = client.get_balance(&target_address).await
                .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "address": target_address.to_string(),
                    "balance": balance,
                    "status": "Note: User profile data fetching not yet implemented in SDK"
                }));
            } else {
                let user_data = vec![
                    UserDisplay { field: "Address".to_string(), value: target_address.to_string() },
                    UserDisplay { field: "Balance".to_string(), value: format!("{} SOL", balance as f64 / 1_000_000_000.0) },
                    UserDisplay { field: "Status".to_string(), value: "User profile data fetching pending SDK implementation".to_string() },
                ];

                println!("\n📊 User Profile");
                println!("{}", Table::new(user_data));
            }
        }
        
        UserCommands::Update { name, bio } => {
            println!("🔄 Updating user profile...");
            
            if name.is_none() && bio.is_none() {
                return Err(anyhow::anyhow!("At least one field (name or bio) must be provided"));
            }

            if let Some(name_text) = name {
                if name_text.trim().is_empty() {
                    return Err(anyhow::anyhow!("User name cannot be empty"));
                }
                if name_text.len() > 32 {
                    return Err(anyhow::anyhow!("User name cannot exceed 32 characters"));
                }
            }

            if let Some(bio_text) = bio {
                if bio_text.len() > 500 {
                    return Err(anyhow::anyhow!("Bio cannot exceed 500 characters"));
                }

                // Update bio using SDK
                let signature = sdk_client.update_user(bio_text).await
                    .map_err(|e| anyhow::anyhow!("Failed to update user profile: {}", e))?;

                if cli.output == "json" {
                    println!("{}", serde_json::json!({
                        "status": "success",
                        "transaction": signature.to_string(),
                        "updated_bio": bio_text
                    }));
                } else {
                    println!("✅ User bio updated successfully!");
                    println!("📝 Transaction: {}", signature);
                    println!("📋 New bio: {}", bio_text);
                }
            } else {
                return Err(anyhow::anyhow!("Name updates not yet supported by SDK - only bio updates are available"));
            }
        }
        
        UserCommands::AddWallet { address } => {
            println!("🔄 Adding wallet: {}", address);
            
            let wallet_pubkey = Pubkey::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid wallet address: {}", e))?;

            // Add wallet using SDK
            let signature = sdk_client.add_wallet(&wallet_pubkey).await
                .map_err(|e| anyhow::anyhow!("Failed to add wallet: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success", 
                    "transaction": signature.to_string(),
                    "wallet_added": address
                }));
            } else {
                println!("✅ Wallet added successfully!");
                println!("📝 Transaction: {}", signature);
                println!("🔑 Added wallet: {}", address);
            }
        }
        
        UserCommands::SetWallet { address } => {
            println!("🔄 Setting active wallet: {}", address);
            
            let wallet_pubkey = Pubkey::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid wallet address: {}", e))?;

            // Set active wallet using SDK
            let signature = sdk_client.set_active_wallet(&wallet_pubkey).await
                .map_err(|e| anyhow::anyhow!("Failed to set active wallet: {}", e))?;

            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "success",
                    "transaction": signature.to_string(),
                    "active_wallet": address
                }));
            } else {
                println!("✅ Active wallet updated successfully!");
                println!("📝 Transaction: {}", signature);
                println!("🔑 Active wallet: {}", address);
            }
        }
    }
    Ok(())
}
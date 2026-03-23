//! User management commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::{UserCommands, Cli};

pub async fn execute_user_command(
    client: &EscrowClient,
    action: &UserCommands,
    cli: &Cli,
) -> Result<()> {
    match action {
        UserCommands::Create { name, bio } => {
            println!("Creating user profile: {}", name);
            if let Some(bio) = bio {
                println!("Bio: {}", bio);
            }
            println!("✅ User profile creation functionality will be implemented in Phase 2");
        }
        UserCommands::Show { address } => {
            if let Some(addr) = address {
                println!("Showing user profile for: {}", addr);
            } else {
                println!("Showing profile for current wallet");
            }
            println!("✅ User profile display functionality will be implemented in Phase 2");
        }
        UserCommands::Update { name, bio } => {
            println!("Updating user profile");
            if name.is_some() || bio.is_some() {
                println!("✅ User profile update functionality will be implemented in Phase 2");
            }
        }
        UserCommands::AddWallet { address } => {
            println!("Adding wallet: {}", address);
            println!("✅ Wallet management functionality will be implemented in Phase 2");
        }
        UserCommands::SetWallet { address } => {
            println!("Setting active wallet: {}", address);
            println!("✅ Wallet management functionality will be implemented in Phase 2");
        }
    }
    Ok(())
}
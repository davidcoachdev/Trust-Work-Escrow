//! Configuration management commands

use anyhow::Result;
use trust_escrow_shared::{EscrowConfig};
use crate::{ConfigCommands, Cli};

pub async fn execute_config_command(
    action: &ConfigCommands,
    cli: &Cli,
) -> Result<()> {
    match action {
        ConfigCommands::Show => {
            match EscrowConfig::load() {
                Ok(config) => {
                    if cli.output == "json" {
                        let json = serde_json::to_string_pretty(&config)?;
                        println!("{}", json);
                    } else {
                        println!("📋 Current Configuration:");
                        println!();
                        println!("Network:");
                        println!("  Cluster: {}", config.network.cluster);
                        println!("  RPC URL: {}", config.network.rpc_url);
                        println!("  Commitment: {}", config.network.commitment);
                        println!();
                        println!("Programs:");
                        println!("  Trust Escrow v2: {}", config.programs.trust_escrow_v2);
                        println!();
                        println!("Wallet:");
                        if let Some(path) = &config.wallet.keypair_path {
                            println!("  Keypair Path: {}", path.display());
                        } else {
                            println!("  Keypair Path: (not set)");
                        }
                        println!("  Type: {:?}", config.wallet.wallet_type);
                        println!();
                        println!("App:");
                        println!("  Log Level: {}", config.app.log_level);
                        println!("  Data Dir: {}", config.app.data_dir.display());
                        println!("  Colored: {}", config.app.colored);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to load configuration: {}", e);
                }
            }
        }
        ConfigCommands::Init { force } => {
            let config_dir = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Unable to determine config directory"))?
                .join("trust-escrow");
            
            let config_file = config_dir.join("config.toml");
            
            if config_file.exists() && !force {
                println!("❌ Configuration file already exists at: {}", config_file.display());
                println!("Use --force to overwrite");
                return Ok(());
            }
            
            std::fs::create_dir_all(&config_dir)?;
            
            let default_config = EscrowConfig::default();
            default_config.save_to_file(&config_file)?;
            
            println!("✅ Created configuration file at: {}", config_file.display());
            println!("Edit this file to customize your settings");
        }
        ConfigCommands::Set { key, value } => {
            println!("Setting config {} = {}", key, value);
            println!("✅ Config value setting functionality will be implemented in Phase 2");
        }
        ConfigCommands::Get { key } => {
            println!("Getting config value for: {}", key);
            println!("✅ Config value retrieval functionality will be implemented in Phase 2");
        }
    }
    Ok(())
}
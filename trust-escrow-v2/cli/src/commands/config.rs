//! Configuration management commands

use anyhow::Result;
use trust_escrow_shared::{EscrowConfig, NetworkConfig};
use crate::{ConfigCommands, Cli};
use tabled::{Table, Tabled};
use std::str::FromStr;

#[derive(Tabled)]
struct ConfigDisplay {
    section: String,
    key: String,
    value: String,
}

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
                        // Create a structured table view
                        let mut config_items = Vec::new();
                        
                        // Network section
                        config_items.push(ConfigDisplay {
                            section: "Network".to_string(),
                            key: "cluster".to_string(),
                            value: config.network.cluster.clone(),
                        });
                        config_items.push(ConfigDisplay {
                            section: "Network".to_string(),
                            key: "rpc_url".to_string(),
                            value: config.network.rpc_url.clone(),
                        });
                        config_items.push(ConfigDisplay {
                            section: "Network".to_string(),
                            key: "commitment".to_string(),
                            value: config.network.commitment.clone(),
                        });
                        config_items.push(ConfigDisplay {
                            section: "Network".to_string(),
                            key: "ws_url".to_string(),
                            value: config.network.ws_url.unwrap_or_else(|| "None".to_string()),
                        });

                        // Programs section
                        config_items.push(ConfigDisplay {
                            section: "Programs".to_string(),
                            key: "trust_escrow_v2".to_string(),
                            value: config.programs.trust_escrow_v2.clone(),
                        });

                        // Wallet section
                        config_items.push(ConfigDisplay {
                            section: "Wallet".to_string(),
                            key: "keypair_path".to_string(),
                            value: config.wallet.keypair_path
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "None".to_string()),
                        });
                        config_items.push(ConfigDisplay {
                            section: "Wallet".to_string(),
                            key: "wallet_type".to_string(),
                            value: format!("{:?}", config.wallet.wallet_type),
                        });

                        // App section
                        config_items.push(ConfigDisplay {
                            section: "App".to_string(),
                            key: "log_level".to_string(),
                            value: config.app.log_level.clone(),
                        });
                        config_items.push(ConfigDisplay {
                            section: "App".to_string(),
                            key: "data_dir".to_string(),
                            value: config.app.data_dir.display().to_string(),
                        });
                        config_items.push(ConfigDisplay {
                            section: "App".to_string(),
                            key: "colored".to_string(),
                            value: config.app.colored.to_string(),
                        });

                        println!("📋 Trust Escrow Configuration");
                        println!("{}", Table::new(config_items));

                        // Show config file location
                        let config_path = match dirs::config_dir() {
                            Some(dir) => {
                                let config_dir = dir.join("trust-escrow");
                                std::fs::create_dir_all(&config_dir)?;
                                config_dir.join("config.toml")
                            }
                            None => std::path::PathBuf::from("./trust-escrow.toml"),
                        };
                        println!("\n📁 Config file: {}", config_path.display());
                        
                        println!("\n💡 Available operations:");
                        println!("  • trust-escrow config set <key> <value>  - Update config value");
                        println!("  • trust-escrow config get <key>          - Get config value");
                        println!("  • trust-escrow config init --force       - Reset to defaults");
                        
                        println!("\n🌐 Quick network switching:");
                        println!("  • trust-escrow --network localnet <command>");
                        println!("  • trust-escrow --network devnet <command>");
                        println!("  • trust-escrow --network mainnet-beta <command>");
                    }
                }
                Err(e) => {
                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "status": "error",
                            "message": format!("Failed to load configuration: {}", e),
                            "suggestion": "Run 'trust-escrow config init' to create a new config file"
                        }));
                    } else {
                        println!("❌ Failed to load configuration: {}", e);
                        println!("💡 Run 'trust-escrow config init' to create a new config file");
                    }
                }
            }
        }

        ConfigCommands::Init { force } => {
            let config_dir = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Unable to determine config directory"))?
                .join("trust-escrow");
            
            let config_file = config_dir.join("config.toml");
            
            if config_file.exists() && !force {
                if cli.output == "json" {
                    println!("{}", serde_json::json!({
                        "status": "exists",
                        "message": "Configuration file already exists",
                        "path": config_file.display().to_string(),
                        "suggestion": "Use --force to overwrite"
                    }));
                } else {
                    println!("❌ Configuration file already exists at: {}", config_file.display());
                    println!("💡 Use --force to overwrite");
                }
                return Ok(());
            }
            
            print!("📁 Creating config directory... ");
            std::fs::create_dir_all(&config_dir)?;
            println!("✅");
            
            print!("📝 Generating default configuration... ");
            let default_config = EscrowConfig::default();
            println!("✅");
            
            print!("💾 Writing config file... ");
            default_config.save_to_file(&config_file)?;
            println!("✅");
            
            if cli.output == "json" {
                println!("{}", serde_json::json!({
                    "status": "created",
                    "path": config_file.display().to_string(),
                    "message": "Configuration file created successfully"
                }));
            } else {
                println!("\n✅ Configuration initialized successfully!");
                println!("📁 Config file: {}", config_file.display());
                println!("📋 Default network: devnet");
                
                println!("\n🔧 Next steps:");
                println!("  1. Set your wallet: trust-escrow --wallet ~/.config/solana/id.json <command>");
                println!("  2. Check connection: trust-escrow status");
                println!("  3. Get test SOL: trust-escrow airdrop");
                println!("  4. Start using: trust-escrow user create --name \"Your Name\"");
                
                println!("\n💡 Edit the config file to customize your settings");
            }
        }

        ConfigCommands::Set { key, value } => {
            println!("🔧 Setting config {} = {}", key, value);
            
            // Load current config
            let mut config = EscrowConfig::load()
                .unwrap_or_else(|_| EscrowConfig::default());

            // Parse and set the value based on key
            let result = match key.as_str() {
                "network.cluster" => {
                    config.network.cluster = value.clone();
                    Ok(())
                }
                "network.rpc_url" => {
                    config.network.rpc_url = value.clone();
                    Ok(())
                }
                "network.commitment" => {
                    config.network.commitment = value.clone();
                    Ok(())
                }
                "network.ws_url" => {
                    config.network.ws_url = if value.is_empty() { None } else { Some(value.clone()) };
                    Ok(())
                }
                "programs.trust_escrow_v2" => {
                    // Validate it's a valid pubkey
                    if let Err(_) = value.parse::<solana_sdk::pubkey::Pubkey>() {
                        return Err(anyhow::anyhow!("Invalid program ID format"));
                    }
                    config.programs.trust_escrow_v2 = value.clone();
                    Ok(())
                }
                "wallet.keypair_path" => {
                    let path = if value.is_empty() { None } else { Some(std::path::PathBuf::from(value)) };
                    config.wallet.keypair_path = path;
                    Ok(())
                }
                "app.log_level" => {
                    // Validate log level
                    match value.as_str() {
                        "error" | "warn" | "info" | "debug" | "trace" => {
                            config.app.log_level = value.clone();
                            Ok(())
                        }
                        _ => Err(anyhow::anyhow!("Invalid log level. Valid values: error, warn, info, debug, trace")),
                    }
                }
                "app.colored" => {
                    config.app.colored = value.parse::<bool>()
                        .map_err(|_| anyhow::anyhow!("Invalid boolean value. Use 'true' or 'false'"))?;
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unknown configuration key: {}", key)),
            };

            match result {
                Ok(()) => {
                    // Save the updated config
                    let config_path = match dirs::config_dir() {
                        Some(dir) => {
                            let config_dir = dir.join("trust-escrow");
                            std::fs::create_dir_all(&config_dir)?;
                            config_dir.join("config.toml")
                        }
                        None => std::path::PathBuf::from("./trust-escrow.toml"),
                    };
                    config.save_to_file(&config_path)?;

                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "status": "success",
                            "key": key,
                            "value": value,
                            "message": "Configuration updated successfully"
                        }));
                    } else {
                        println!("✅ Configuration updated: {} = {}", key, value);
                        println!("💾 Saved to: {}", config_path.display());
                        
                        // Show relevant warnings or tips
                        if key.starts_with("network.") {
                            println!("💡 Network settings changed - restart any running processes");
                        }
                        if key == "wallet.keypair_path" {
                            println!("💡 Wallet path changed - verify the file exists and is readable");
                        }
                    }
                }
                Err(e) => {
                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "status": "error",
                            "key": key,
                            "value": value,
                            "message": format!("Failed to set configuration: {}", e)
                        }));
                    } else {
                        println!("❌ Failed to set configuration: {}", e);
                        println!("\n📖 Valid configuration keys:");
                        println!("  Network: network.cluster, network.rpc_url, network.commitment, network.ws_url");
                        println!("  Programs: programs.trust_escrow_v2");
                        println!("  Wallet: wallet.keypair_path");
                        println!("  App: app.log_level, app.colored");
                    }
                }
            }
        }

        ConfigCommands::Get { key } => {
            println!("🔍 Getting config value for: {}", key);
            
            // Load current config
            match EscrowConfig::load() {
                Ok(config) => {
                    let value = match key.as_str() {
                        "network.cluster" => Some(config.network.cluster),
                        "network.rpc_url" => Some(config.network.rpc_url),
                        "network.commitment" => Some(config.network.commitment),
                        "network.ws_url" => config.network.ws_url,
                        "programs.trust_escrow_v2" => Some(config.programs.trust_escrow_v2),
                        "wallet.keypair_path" => config.wallet.keypair_path.map(|p| p.display().to_string()),
                        "wallet.wallet_type" => Some(format!("{:?}", config.wallet.wallet_type)),
                        "app.log_level" => Some(config.app.log_level),
                        "app.data_dir" => Some(config.app.data_dir.display().to_string()),
                        "app.colored" => Some(config.app.colored.to_string()),
                        _ => None,
                    };

                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "key": key,
                            "value": value,
                            "exists": value.is_some()
                        }));
                    } else {
                        match value {
                            Some(val) => println!("📋 {}: {}", key, val),
                            None => {
                                println!("❌ Unknown configuration key: {}", key);
                                println!("\n📖 Available keys:");
                                println!("  Network: network.cluster, network.rpc_url, network.commitment, network.ws_url");
                                println!("  Programs: programs.trust_escrow_v2");
                                println!("  Wallet: wallet.keypair_path, wallet.wallet_type");
                                println!("  App: app.log_level, app.data_dir, app.colored");
                            }
                        }
                    }
                }
                Err(e) => {
                    if cli.output == "json" {
                        println!("{}", serde_json::json!({
                            "status": "error",
                            "key": key,
                            "message": format!("Failed to load configuration: {}", e)
                        }));
                    } else {
                        println!("❌ Failed to load configuration: {}", e);
                        println!("💡 Run 'trust-escrow config init' to create a config file");
                    }
                }
            }
        }
    }
    Ok(())
}
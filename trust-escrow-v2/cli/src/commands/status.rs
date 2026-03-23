//! Status and connectivity commands

use anyhow::Result;
use trust_escrow_shared::EscrowClient;
use crate::Cli;
use tabled::{Table, Tabled};
use solana_sdk::commitment_config::CommitmentConfig;

#[derive(Tabled)]
struct StatusDisplay {
    component: String,
    status: String,
    details: String,
}

#[derive(Tabled)]
struct NetworkInfoDisplay {
    property: String,
    value: String,
}

pub async fn execute_status_command(
    client: &EscrowClient,
    cli: &Cli,
) -> Result<()> {
    if cli.output == "json" {
        return execute_json_status(client).await;
    }

    println!("🔍 Checking Trust Work Escrow system status...");
    println!();

    let mut status_items = Vec::new();
    let mut overall_health = true;

    // Network connection test
    print!("📡 Testing network connection... ");
    let network_status = match client.check_connection().await {
        Ok(_) => {
            println!("✅");
            "✅ Connected"
        }
        Err(e) => {
            println!("❌");
            overall_health = false;
            &format!("❌ Failed: {}", e)
        }
    };

    // Network information
    let config = client.config();
    let mut network_info = Vec::new();
    network_info.push(NetworkInfoDisplay {
        property: "Cluster".to_string(),
        value: config.network.cluster.clone(),
    });
    network_info.push(NetworkInfoDisplay {
        property: "RPC URL".to_string(),
        value: config.network.rpc_url.clone(),
    });
    network_info.push(NetworkInfoDisplay {
        property: "Commitment".to_string(),
        value: config.network.commitment.clone(),
    });
    
    if let Some(ws_url) = &config.network.ws_url {
        network_info.push(NetworkInfoDisplay {
            property: "WebSocket URL".to_string(),
            value: ws_url.clone(),
        });
    }

    // Get current slot and network performance
    let (slot_info, slot_status) = match client.get_slot().await {
        Ok(slot) => {
            (format!("Slot: {}", slot), "✅ Synced")
        }
        Err(e) => {
            overall_health = false;
            (format!("Error: {}", e), "❌ Failed")
        }
    };

    status_items.push(StatusDisplay {
        component: "Network Connection".to_string(),
        status: network_status.to_string(),
        details: format!("{} | {}", config.network.cluster, config.network.rpc_url),
    });

    status_items.push(StatusDisplay {
        component: "Blockchain Sync".to_string(),
        status: slot_status.to_string(),
        details: slot_info,
    });

    // Wallet status
    let (wallet_status, wallet_details) = if client.has_wallet() {
        if let Some(pubkey) = client.wallet_pubkey() {
            match client.get_wallet_balance().await {
                Ok(balance) => {
                    let sol_balance = balance as f64 / 1_000_000_000.0;
                    let status = if balance > 10_000_000 { // > 0.01 SOL
                        "✅ Funded"
                    } else if balance > 0 {
                        "⚠️  Low Balance"
                    } else {
                        "❌ No Balance"
                    };
                    (
                        status.to_string(),
                        format!("{} | {:.6} SOL", pubkey, sol_balance)
                    )
                }
                Err(e) => {
                    overall_health = false;
                    ("❌ Error".to_string(), format!("{} | Error: {}", pubkey, e))
                }
            }
        } else {
            overall_health = false;
            ("❌ Invalid".to_string(), "Wallet configured but invalid".to_string())
        }
    } else {
        overall_health = false;
        ("❌ Not Configured".to_string(), "No wallet keypair configured".to_string())
    };

    status_items.push(StatusDisplay {
        component: "Wallet".to_string(),
        status: wallet_status,
        details: wallet_details,
    });

    // Program validation
    let program_id = &config.programs.trust_escrow_v2;
    print!("🔍 Checking Trust Escrow program... ");
    let (program_status, program_details) = match program_id.parse::<solana_sdk::pubkey::Pubkey>() {
        Ok(pubkey) => {
            match client.rpc().get_account(&pubkey) {
                Ok(account) => {
                    println!("✅");
                    if account.executable {
                        ("✅ Deployed".to_string(), format!("{} | Executable program", pubkey))
                    } else {
                        ("⚠️  Not Executable".to_string(), format!("{} | Account exists but not executable", pubkey))
                    }
                }
                Err(_) => {
                    println!("❌");
                    overall_health = false;
                    ("❌ Not Found".to_string(), format!("{} | Program not deployed", pubkey))
                }
            }
        }
        Err(e) => {
            println!("❌");
            overall_health = false;
            ("❌ Invalid ID".to_string(), format!("Invalid program ID: {}", e))
        }
    };

    status_items.push(StatusDisplay {
        component: "Trust Escrow Program".to_string(),
        status: program_status,
        details: program_details,
    });

    // SDK client status
    let sdk_status = if client.sdk().is_some() {
        "✅ Available".to_string()
    } else {
        "❌ Not Available".to_string()
    };

    status_items.push(StatusDisplay {
        component: "SDK Client".to_string(),
        status: sdk_status,
        details: "Trust Escrow SDK client for operations".to_string(),
    });

    // Display results
    println!("🌐 Network Information");
    println!("{}", Table::new(network_info));

    println!("\n📊 System Status");
    println!("{}", Table::new(status_items));

    // Overall health assessment
    println!();
    if overall_health {
        println!("🎉 All systems operational! Ready to use Trust Work Escrow.");
        println!();
        println!("🚀 Quick start commands:");
        println!("  • trust-escrow user create --name \"Your Name\"");
        println!("  • trust-escrow job create --title \"Job Title\" --description \"Description\" --amount 1.0");
        println!("  • trust-escrow payment balance");
    } else {
        println!("⚠️  System issues detected. Review the status above and fix any problems.");
        println!();
        println!("🔧 Common fixes:");
        println!("  • Network issues: Check your internet connection and RPC URL");
        println!("  • Wallet issues: Set wallet with --wallet or configure in config file");
        println!("  • Balance issues: Get test SOL with 'trust-escrow airdrop' (devnet/localnet)");
        println!("  • Program issues: Ensure you're using the correct network and program ID");
    }

    // Additional validation
    if let Ok(warnings) = client.validate().await {
        if !warnings.is_empty() {
            println!("\n⚠️  Additional warnings:");
            for warning in warnings {
                println!("   • {}", warning);
            }
        }
    }

    Ok(())
}

async fn execute_json_status(client: &EscrowClient) -> Result<()> {
    let mut status = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    });

    // Network status
    let network_connected = client.check_connection().await.is_ok();
    let config = client.config();
    
    let slot_result = client.get_slot().await;
    
    status["network"] = serde_json::json!({
        "connected": network_connected,
        "cluster": config.network.cluster,
        "rpc_url": config.network.rpc_url,
        "commitment": config.network.commitment,
        "ws_url": config.network.ws_url,
        "current_slot": slot_result.as_ref().ok(),
        "slot_error": slot_result.as_ref().err().map(|e| e.to_string()),
    });

    // Wallet status
    let wallet_configured = client.has_wallet();
    let wallet_pubkey = client.wallet_pubkey();
    let wallet_balance = if wallet_configured {
        client.get_wallet_balance().await.ok()
    } else {
        None
    };

    status["wallet"] = serde_json::json!({
        "configured": wallet_configured,
        "public_key": wallet_pubkey.map(|k| k.to_string()),
        "balance_lamports": wallet_balance,
        "balance_sol": wallet_balance.map(|b| b as f64 / 1_000_000_000.0),
    });

    // Program status
    let program_id = &config.programs.trust_escrow_v2;
    let program_validation = match program_id.parse::<solana_sdk::pubkey::Pubkey>() {
        Ok(pubkey) => {
            match client.rpc().get_account(&pubkey) {
                Ok(account) => serde_json::json!({
                    "valid": true,
                    "deployed": true,
                    "executable": account.executable,
                    "program_id": pubkey.to_string(),
                }),
                Err(e) => serde_json::json!({
                    "valid": true,
                    "deployed": false,
                    "error": e.to_string(),
                    "program_id": pubkey.to_string(),
                }),
            }
        }
        Err(e) => serde_json::json!({
            "valid": false,
            "error": e.to_string(),
            "program_id": program_id,
        }),
    };

    status["program"] = program_validation.clone();

    // SDK client status
    status["sdk"] = serde_json::json!({
        "available": client.sdk().is_some(),
    });

    // Overall health
    let overall_health = network_connected && 
                         wallet_configured && 
                         wallet_balance.unwrap_or(0) > 0 &&
                         program_validation["valid"].as_bool().unwrap_or(false) &&
                         program_validation["deployed"].as_bool().unwrap_or(false);

    status["health"] = serde_json::json!({
        "overall": overall_health,
        "ready": overall_health,
    });

    // Validation warnings
    if let Ok(warnings) = client.validate().await {
        status["warnings"] = serde_json::json!(warnings);
    }

    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}
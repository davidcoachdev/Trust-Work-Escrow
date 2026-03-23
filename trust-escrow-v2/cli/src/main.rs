//! Trust Work Escrow CLI
//!
//! Command-line interface for Trust Work Escrow v2 protocol operations

use anyhow::Result;
use std::process;
use trust_escrow_shared::{EscrowClient, EscrowConfig};

use trust_escrow_cli::{Cli, Commands};
use clap::Parser;

mod commands {
    pub use trust_escrow_cli::commands::*;
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(cli.log_level.parse().unwrap_or(log::LevelFilter::Info))
        .init();

    // Handle config override
    let client = match create_client(&cli).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error: {}", e);
            if cli.log_level == "debug" || cli.log_level == "trace" {
                eprintln!("\nDebug info: {:?}", e);
            }
            process::exit(1);
        }
    };

    // Execute command
    let result = match &cli.command {
        Commands::User { action } => {
            commands::user::execute_user_command(&client, action, &cli).await
        }
        Commands::Job { action } => {
            commands::job::execute_job_command(&client, action, &cli).await
        }
        Commands::Milestone { action } => {
            commands::milestone::execute_milestone_command(&client, action, &cli).await
        }
        Commands::Payment { action } => {
            commands::payment::execute_payment_command(&client, action, &cli).await
        }
        Commands::Dispute { action } => {
            commands::dispute::execute_dispute_command(&client, action, &cli).await
        }
        Commands::Config { action } => {
            commands::config::execute_config_command(action, &cli).await
        }
        Commands::Status => {
            commands::status::execute_status_command(&client, &cli).await
        }
        Commands::Airdrop { amount } => {
            commands::airdrop::execute_airdrop_command(&client, *amount, &cli).await
        }
    };

    if let Err(e) = result {
        eprintln!("Command failed: {}", e);
        if cli.log_level == "debug" || cli.log_level == "trace" {
            eprintln!("\nDebug info: {:?}", e);
        }
        process::exit(1);
    }

    Ok(())
}

/// Create client with CLI overrides
async fn create_client(cli: &Cli) -> Result<EscrowClient> {
    let mut config = if let Some(config_path) = &cli.config {
        EscrowConfig::load_from_file(config_path)?
    } else if let Some(network) = &cli.network {
        match network.as_str() {
            "localnet" => EscrowConfig::preset_localnet(),
            "devnet" => EscrowConfig::preset_devnet(), 
            "mainnet-beta" => EscrowConfig::preset_mainnet(),
            _ => return Err(anyhow::anyhow!("Unknown network: {}", network)),
        }
    } else {
        EscrowConfig::load()?
    };

    // Apply CLI overrides
    if let Some(rpc_url) = &cli.rpc_url {
        config.network.rpc_url = rpc_url.clone();
    }
    
    if let Some(wallet_path) = &cli.wallet {
        config.wallet.keypair_path = Some(wallet_path.into());
    }

    config.app.colored = !cli.no_color;

    // Create and validate client
    let mut client = EscrowClient::from_config(config)?;
    
    // Load wallet if specified
    if let Some(wallet_path) = &cli.wallet {
        client.load_wallet_from_file(wallet_path)?;
    }

    Ok(client)
}
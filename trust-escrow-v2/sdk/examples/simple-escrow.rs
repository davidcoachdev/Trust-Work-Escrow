/*!
 * Simple Escrow Example
 *
 * This example demonstrates the most basic escrow workflow:
 * 1. Client creates and funds a job
 * 2. Freelancer applies and gets accepted
 * 3. Freelancer submits work
 * 4. Client approves and payment is released
 *
 * Run with: `cargo run --example simple-escrow`
 */

use std::sync::Arc;
use std::time::Duration;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer};
use trust_escrow_sdk::{CofreClient, EscrowError, Result};

/// Configuration for the example
#[derive(Clone)]
pub struct ExampleConfig {
    pub rpc_url: String,
    pub commitment: CommitmentConfig,
}

impl Default for ExampleConfig {
    fn default() -> Self {
        Self {
            // Use devnet for testing - change to mainnet-beta for production
            rpc_url: "https://api.devnet.solana.com".to_string(),
            commitment: CommitmentConfig::confirmed(),
        }
    }
}

/// Create a new client instance
async fn create_client(keypair: Keypair, config: &ExampleConfig) -> Result<CofreClient> {
    let rpc = Arc::new(RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        config.commitment,
    ));
    let payer = Arc::new(keypair);

    CofreClient::new(rpc, payer)
}

/// Setup users for the escrow example
async fn setup_users() -> Result<(CofreClient, CofreClient)> {
    let config = ExampleConfig::default();

    // Create keypairs for client and freelancer
    let client_keypair = Keypair::new();
    let freelancer_keypair = Keypair::new();

    println!("👤 Setting up users...");
    println!("   Client wallet: {}", client_keypair.pubkey());
    println!("   Freelancer wallet: {}", freelancer_keypair.pubkey());

    // Create SDK clients
    let client = create_client(client_keypair, &config).await?;
    let freelancer = create_client(freelancer_keypair, &config).await?;

    // Create user accounts on-chain
    println!("\n📝 Creating user accounts...");

    match client
        .create_user("tech_startup", Some("Looking for quality developers"))
        .await
    {
        Ok(sig) => println!("   ✅ Client user created: {}", sig),
        Err(EscrowError::Network(_)) => println!("   ⚠️  Client creation failed (need devnet SOL)"),
        Err(e) => return Err(e),
    }

    match freelancer
        .create_user("alice_dev", Some("Full-stack React developer"))
        .await
    {
        Ok(sig) => println!("   ✅ Freelancer user created: {}", sig),
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Freelancer creation failed (need devnet SOL)")
        }
        Err(e) => return Err(e),
    }

    Ok((client, freelancer))
}

/// Create and fund a job
async fn create_job(client: &CofreClient) -> Result<(solana_sdk::pubkey::Pubkey, u64)> {
    println!("\n💼 Creating job...");

    let job_result = client.create_job(
        "React Dashboard Development",
        "Need a responsive admin dashboard built with React and TypeScript. Should include user management, analytics charts, and dark mode support.",
        5_000_000, // 0.005 SOL (about $1-2 depending on SOL price)
        Duration::from_secs(86400 * 14), // 2 weeks deadline
        false, // doesn't require a team
    ).await;

    match job_result {
        Ok((job_pda, signature)) => {
            println!("   ✅ Job created successfully!");
            println!("      Job PDA: {}", job_pda);
            println!("      Transaction: {}", signature);

            // In this example, we'll use job_id = 1
            // In a real application, you'd track job IDs properly
            let job_id = 1u64;

            // Fund the escrow
            println!("\n💰 Funding escrow...");
            match client.fund_escrow(job_id).await {
                Ok(fund_sig) => {
                    println!("   ✅ Escrow funded: {}", fund_sig);
                    Ok((job_pda, job_id))
                }
                Err(EscrowError::InsufficientFunds(_)) => {
                    println!("   ❌ Insufficient funds to create escrow");
                    println!("      Need at least 0.005 SOL + transaction fees");
                    Err(EscrowError::InsufficientFunds("Need more SOL".to_string()))
                }
                Err(e) => Err(e),
            }
        }
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Job creation failed - need devnet SOL for transactions");
            println!("      Get devnet SOL from: https://faucet.solana.com");

            // Return a mock job for demonstration
            let mock_job_pda = solana_sdk::pubkey::Pubkey::new_unique();
            Ok((mock_job_pda, 1))
        }
        Err(e) => Err(e),
    }
}

/// Handle job application process
async fn handle_application(
    client: &CofreClient,
    freelancer: &CofreClient,
    job_pda: &solana_sdk::pubkey::Pubkey,
) -> Result<()> {
    println!("\n📋 Handling job application...");

    // Freelancer applies to the job
    let proposal = "Hi! I'm Alice, a React developer with 5 years of experience. I've built similar dashboards for 10+ companies. I can deliver this project in 10 days with clean code, responsive design, and comprehensive documentation. My portfolio: https://alice-dev.com";

    match freelancer.apply_to_job(job_pda, proposal).await {
        Ok(apply_sig) => {
            println!("   ✅ Application submitted: {}", apply_sig);
        }
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Application submission simulated (need devnet SOL)");
        }
        Err(e) => return Err(e),
    }

    // Client reviews and accepts the application
    let freelancer_pubkey = freelancer.payer().pubkey();

    match client.accept_application(job_pda, &freelancer_pubkey).await {
        Ok(accept_sig) => {
            println!("   ✅ Application accepted: {}", accept_sig);
            println!("      Job is now in progress!");
        }
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Application acceptance simulated (need devnet SOL)");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

/// Handle work submission and approval
async fn handle_work_completion(
    client: &CofreClient,
    freelancer: &CofreClient,
    job_pda: &solana_sdk::pubkey::Pubkey,
) -> Result<()> {
    println!("\n🚀 Handling work completion...");

    // Simulate work being done
    println!("   ⏳ Freelancer is working on the project...");
    println!("      (In reality, this could take days or weeks)");

    // Freelancer submits completed work
    let work_submission = "Dashboard completed! 🎉\n\nDeliverables:\n- Live demo: https://dashboard-demo.alice-dev.com\n- Source code: https://github.com/alice-dev/react-dashboard\n- Documentation: https://docs.alice-dev.com/react-dashboard\n\nFeatures implemented:\n✅ User management with role-based access\n✅ Analytics charts with Chart.js\n✅ Dark/light mode toggle\n✅ Responsive design (mobile-first)\n✅ TypeScript throughout\n✅ 95% test coverage\n\nPlease review and let me know if you need any adjustments!";

    match freelancer.submit_work(job_pda, work_submission).await {
        Ok(submit_sig) => {
            println!("   ✅ Work submitted: {}", submit_sig);
        }
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Work submission simulated (need devnet SOL)");
        }
        Err(e) => return Err(e),
    }

    // Client reviews the work
    println!("\n🔍 Client reviewing submitted work...");
    println!("      (Client checks demo, reviews code, tests functionality)");

    // Client approves the work
    match client.approve_work(job_pda).await {
        Ok(approve_sig) => {
            println!("   ✅ Work approved! Payment released: {}", approve_sig);
            println!("      💰 Freelancer has been paid 0.005 SOL");
            println!("      🎉 Escrow completed successfully!");
        }
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Work approval simulated (need devnet SOL)");
            println!("      💰 Payment would be released to freelancer");
            println!("      🎉 Escrow workflow completed!");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

/// Demonstrate escrow statistics
async fn show_escrow_stats(client: &CofreClient) -> Result<()> {
    println!("\n📊 Escrow Statistics:");

    match client.get_escrow_stats().await {
        Ok(stats) => {
            println!("   Total escrows: {}", stats.total_escrows);
            println!("   Active escrows: {}", stats.active_escrows);
            println!("   Completed escrows: {}", stats.completed_escrows);
            println!("   Total volume: {} lamports", stats.total_volume);

            // Convert to SOL for readability
            let volume_sol =
                trust_escrow_sdk::utils::ConversionUtils::lamports_to_sol(stats.total_volume);
            println!("   Total volume: {:.6} SOL", volume_sol);
        }
        Err(EscrowError::Network(_)) => {
            println!("   ⚠️  Stats unavailable (need validator connection)");
        }
        Err(e) => return Err(e),
    }

    Ok(())
}

/// Main example function
async fn run_simple_escrow_example() -> Result<()> {
    println!("🔐 Trust Escrow SDK - Simple Escrow Example");
    println!("============================================\n");

    // 1. Setup users
    let (client, freelancer) = setup_users().await?;

    // 2. Create and fund job
    let (job_pda, _job_id) = create_job(&client).await?;

    // 3. Handle application
    handle_application(&client, &freelancer, &job_pda).await?;

    // 4. Handle work completion
    handle_work_completion(&client, &freelancer, &job_pda).await?;

    // 5. Show statistics
    show_escrow_stats(&client).await?;

    println!("\n✨ Example completed successfully!");
    println!("   This demonstrates the basic escrow workflow.");
    println!("   For more advanced examples, see:");
    println!("   - team-collaboration.rs (team-based projects)");
    println!("   - milestone-payments.rs (payment in stages)");
    println!("   - dispute-resolution.rs (handling disagreements)");

    Ok(())
}

/// Error handling demonstration
async fn demonstrate_error_handling() -> Result<()> {
    println!("\n🔧 Error Handling Examples:");

    let config = ExampleConfig::default();
    let client = create_client(Keypair::new(), &config).await?;

    // Example 1: Validation error
    match client.create_user("", None).await {
        Err(EscrowError::Validation(msg)) => {
            println!("   ✅ Caught validation error: {}", msg);
        }
        _ => println!("   ⚠️  Expected validation error"),
    }

    // Example 2: Network error (no devnet SOL)
    match client.create_user("test_user", None).await {
        Err(EscrowError::Network(_)) => {
            println!("   ✅ Caught network error (expected without devnet SOL)");
        }
        Err(EscrowError::InsufficientFunds(_)) => {
            println!("   ✅ Caught insufficient funds error");
        }
        Ok(_) => {
            println!("   ✅ User creation succeeded!");
        }
        Err(e) => {
            println!("   ⚠️  Unexpected error: {:?}", e);
        }
    }

    Ok(())
}

/// Performance measurement
async fn measure_performance() -> Result<()> {
    println!("\n⚡ Performance Measurement:");

    let config = ExampleConfig::default();
    let client = create_client(Keypair::new(), &config).await?;

    // Measure client creation time
    let start = std::time::Instant::now();
    let _client2 = create_client(Keypair::new(), &config).await?;
    let creation_time = start.elapsed();

    println!("   Client creation time: {:?}", creation_time);

    // Measure PDA derivation time
    let start = std::time::Instant::now();
    for i in 0..1000 {
        let authority = solana_sdk::pubkey::Pubkey::new_unique();
        let _user_pda = trust_escrow_sdk::pda::find_user_pda(&authority);
        let _job_pda = trust_escrow_sdk::pda::find_job_pda(&authority, i);
    }
    let derivation_time = start.elapsed();

    println!("   1000 PDA derivations: {:?}", derivation_time);
    println!("   Average per derivation: {:?}", derivation_time / 2000); // 2 PDAs per iteration

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up basic logging
    env_logger::init();

    // Run the main example
    if let Err(e) = run_simple_escrow_example().await {
        eprintln!("❌ Example failed: {}", e);
        std::process::exit(1);
    }

    // Demonstrate error handling
    demonstrate_error_handling().await?;

    // Show performance metrics
    measure_performance().await?;

    println!("\n🎯 Example Tips:");
    println!("   1. Get devnet SOL: https://faucet.solana.com");
    println!("   2. Use solana-cli to check balances: `solana balance`");
    println!("   3. Switch to mainnet by changing RPC URL in ExampleConfig");
    println!("   4. Monitor transactions on explorer: https://solscan.io/");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = ExampleConfig::default();
        let keypair = Keypair::new();

        let client = create_client(keypair, &config).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_pda_derivation() {
        let authority = solana_sdk::pubkey::Pubkey::new_unique();
        let (user_pda, bump) = trust_escrow_sdk::pda::find_user_pda(&authority);

        assert_ne!(user_pda, solana_sdk::pubkey::Pubkey::default());
        assert!(bump <= 255);
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let config = ExampleConfig::default();
        let client = create_client(Keypair::new(), &config).await.unwrap();

        // Test empty username validation
        let result = client.create_user("", None).await;
        assert!(result.is_err());
    }
}

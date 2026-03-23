# Trust Escrow SDK

[![Crates.io](https://img.shields.io/crates/v/trust-escrow-sdk.svg)](https://crates.io/crates/trust-escrow-sdk)
[![Documentation](https://docs.rs/trust-escrow-sdk/badge.svg)](https://docs.rs/trust-escrow-sdk)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A comprehensive Rust SDK for interacting with the Trust Work Escrow v2 smart contract on Solana. This SDK provides type-safe, high-level operations for escrow functionality including user management, job lifecycle operations, dispute handling, milestone-based payments, and multi-wallet support.

## Features

- 🔒 **Type-Safe**: Built on Anchor-generated types for compile-time safety
- 🚀 **High-Level API**: Intuitive methods for complex escrow operations  
- 🔄 **Multi-Wallet Support**: Manage up to 5 wallets per user account
- 📊 **Complete Coverage**: All 31 v2 contract instructions supported
- ⚡ **Performance**: PDA caching and optimized transaction building
- 🛡️ **Error Handling**: Comprehensive error types with context
- 📚 **Documentation**: Full API docs with examples
- 🔧 **Integration Ready**: Patterns for CLI, TUI, and backend integration

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
trust-escrow-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
solana-client = "~1.18"
solana-sdk = "~1.18"
```

### Basic Usage

```rust
use trust_escrow_sdk::{CofreClient, error::Result};
use solana_sdk::{signature::Keypair, commitment_config::CommitmentConfig};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Solana devnet
    let rpc = Arc::new(RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed()
    ));
    
    // Load your keypair (replace with actual keypair loading)
    let payer = Arc::new(Keypair::new());
    
    // Create SDK client
    let client = CofreClient::new(rpc, payer)?;
    
    // Create a user account
    let signature = client.create_user("alice", Some("Freelance developer")).await?;
    println!("User created: {}", signature);
    
    // Create a job posting
    let (job_pda, signature) = client.create_job(
        1,                    // job_id
        "Build a website",    // title  
        "Need a React website with modern design", // description
        1_500_000_000,       // 1.5 SOL in lamports
    ).await?;
    println!("Job created at {}: {}", job_pda, signature);
    
    Ok(())
}
```

## Core Components

### CofreClient

The main client provides high-level operations:

```rust
// User operations
client.create_user("username", Some("bio")).await?;
client.add_wallet(&wallet_pubkey).await?;
client.set_active_wallet(&wallet_pubkey).await?;

// Job lifecycle
let (job_pda, sig) = client.create_job(1, "Title", "Description", amount).await?;
client.deposit_funds(&job_pda).await?;
client.apply_to_job(&job_pda, "My proposal").await?;
client.accept_application(&job_pda, &freelancer_pubkey).await?;
client.submit_work(&job_pda, "https://work.example.com").await?;
client.approve_work(&job_pda).await?;

// Team management
let (team_pda, sig) = client.create_team("My Team", "Description").await?;
client.add_team_member(&team_pda, &member_pubkey, MemberRole::Admin).await?;

// Dispute handling
let (dispute_pda, sig) = client.raise_dispute(&job_pda, "Evidence text").await?;
client.submit_evidence(&dispute_pda, "Additional evidence").await?;
client.resolve_dispute(&dispute_pda, 75).await?; // 75% to client

// Milestone payments
let (milestone_pda, sig) = client.create_milestone(
    &job_pda, "Milestone 1", "Description", 500_000_000
).await?;
client.submit_milestone(&milestone_pda, "https://work.example.com").await?;
client.approve_milestone(&milestone_pda).await?;
```

### PDA Management

Efficient Program Derived Address handling with caching:

```rust
use trust_escrow_sdk::pda::*;

// Direct PDA derivation
let (user_pda, bump) = derive_user_pda(&authority)?;
let (job_pda, bump) = derive_job_pda(&client, job_id)?;

// Cached PDA access (recommended for performance)
let (user_pda, bump) = get_user_pda(&authority)?;
let (job_pda, bump) = get_job_pda(&client, job_id)?;

// Batch PDA operations
let pdas = BatchPdaBuilder::new()
    .add_config()
    .add_user(authority)
    .add_job(client, 1)
    .add_arbiter_pool()
    .build()?;
```

### Type System

Rich type definitions with validation:

```rust
use trust_escrow_sdk::types::*;

// Account types
let user = User { username, bio, wallets, active_wallet, .. };
user.validate()?; // Validates constraints
assert!(user.can_add_wallet()); // Business logic methods

// Enums for state management
let status = JobStatus::InProgress;
assert!(status.can_submit_work());
assert!(!status.accepts_applications());

// Configuration and validation
let config = Config { admin, treasury, fee_percentage, .. };
config.ensure_not_paused()?;
let fee = config.calculate_fee(job_amount);
```

### Error Handling

Comprehensive error types with context:

```rust
use trust_escrow_sdk::error::*;

match client.create_user("", None).await {
    Ok(signature) => println!("Success: {}", signature),
    Err(EscrowError::InvalidParameter { msg }) => {
        println!("Invalid input: {}", msg);
    }
    Err(EscrowError::Contract { code, msg }) => {
        println!("Contract error {}: {}", code, msg);
    }
    Err(EscrowError::Network { msg }) => {
        println!("Network error: {}", msg);
    }
    Err(e) => println!("Other error: {}", e),
}

// Utility functions for error handling
if ErrorUtils::is_program_error(&error, 6001) {
    println!("User not found error");
}

let friendly_message = ErrorUtils::user_friendly_message(&error);
println!("User-friendly: {}", friendly_message);
```

## Integration Patterns

### CLI Integration

```rust
use trust_escrow_sdk::{CofreClient, utils::*};

// CLI-friendly error messages
match client.create_job(id, title, desc, amount).await {
    Ok((job_pda, signature)) => {
        println!("✅ Job created successfully!");
        println!("Job ID: {}", job_pda);
        println!("Transaction: {}", signature);
    }
    Err(e) => {
        eprintln!("❌ Error: {}", ErrorUtils::user_friendly_message(&e));
        std::process::exit(1);
    }
}

// SOL amount formatting
let amount_sol = ConversionUtils::format_sol(lamports);
println!("Job amount: {}", amount_sol);
```

### TUI Integration (Ratatui)

```rust
use trust_escrow_sdk::{CofreClient, types::JobStatus};

// State management compatible with TUI patterns
struct AppState {
    client: CofreClient,
    jobs: Vec<Job>,
    selected_job: Option<usize>,
}

impl AppState {
    async fn refresh_jobs(&mut self) -> Result<()> {
        // Fetch jobs using SDK
        // Update TUI state
        Ok(())
    }
    
    fn can_approve_selected(&self) -> bool {
        self.jobs.get(self.selected_job.unwrap_or(0))
            .map(|job| job.status.can_review_work())
            .unwrap_or(false)
    }
}
```

### Backend Integration (Axum)

```rust
use axum::{Json, extract::State};
use trust_escrow_sdk::{CofreClient, types::Job};

async fn create_job_handler(
    State(client): State<CofreClient>,
    Json(request): Json<CreateJobRequest>,
) -> Result<Json<CreateJobResponse>, AppError> {
    let (job_pda, signature) = client
        .create_job(request.job_id, &request.title, &request.description, request.amount)
        .await
        .map_err(AppError::from)?;
    
    Ok(Json(CreateJobResponse {
        job_pda: job_pda.to_string(),
        signature: signature.to_string(),
    }))
}
```

## Development

### Prerequisites

- Rust 1.75+
- Anchor CLI 0.32+
- Solana CLI 1.18+

### Building

```bash
# Clone repository
git clone https://github.com/trust-work/trust-escrow-v2
cd trust-escrow-v2/sdk

# Build the SDK
cargo build

# Run tests
cargo test

# Check formatting and lints
cargo fmt --check
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

### Testing with Local Validator

```bash
# Start local validator with Trust Escrow program
solana-test-validator --bpf-program <PROGRAM_ID> ../trust-escrow-v2/target/deploy/trust_escrow_v2.so

# Run integration tests
cargo test -- --ignored --test-threads=1
```

## Constants and Limits

| Constant | Value | Description |
|----------|--------|-------------|
| `MIN_JOB_AMOUNT` | 100,000 lamports | Minimum job amount (0.0001 SOL) |
| `MAX_WALLETS` | 5 | Maximum wallets per user |
| `MAX_ARBITERS` | 50 | Maximum arbiters in pool |
| `MAX_MILESTONES` | 20 | Maximum milestones per job |
| `MAX_DISPUTE_EVIDENCE` | 2048 chars | Maximum evidence length |

## Contract Instructions Supported

The SDK supports all 31 Trust Escrow v2 contract instructions:

### Config (5)
- `initialize_config` - Initialize global configuration
- `pause` / `unpause` - Program pause controls  
- `withdraw_treasury` / `update_treasury` - Treasury management

### User (4)
- `create_user` - Create user profile
- `add_wallet` - Add wallet (max 5)
- `set_active_wallet` - Change active wallet
- `update_user` - Update bio

### Team (2)  
- `create_team` - Create team
- `add_team_member` - Add member

### Job (8)
- `create_job` - Create job posting
- `deposit_funds` - Client funds job
- `apply_to_job` / `accept_application` - Application flow
- `submit_work` / `approve_work` / `reject_work` - Work submission
- `cancel_job` - Cancel job

### Arbiter Pool (3)
- `create_arbiter_pool` - Create arbiter pool
- `add_arbiter` / `remove_arbiter` - Manage arbiters

### Dispute (5)
- `raise_dispute` - Open dispute
- `submit_evidence` - Submit evidence
- `assign_arbiter` / `resolve_dispute` - Resolution flow
- `finalize_dispute_payouts` - Execute payouts

### Milestone (4)
- `create_milestone` - Create milestone
- `submit_milestone` / `approve_milestone` / `reject_milestone` - Milestone flow

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to our repository.

## Support

- 📖 [Full Documentation](https://docs.rs/trust-escrow-sdk)
- 🐛 [Issue Tracker](https://github.com/trust-work/trust-escrow-v2/issues)
- 💬 [Discord Community](https://discord.gg/trustwork)
- 📧 Email: developers@trustwork.com

---

**Built with ❤️ for the Solana ecosystem**
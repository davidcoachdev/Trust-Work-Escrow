# Getting Started with Trust Escrow SDK

A comprehensive guide to building escrow-powered applications with the Trust Work Escrow v2 SDK.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Basic Concepts](#basic-concepts)
- [Your First Escrow](#your-first-escrow)
- [Working with Teams](#working-with-teams)
- [Milestone-Based Payments](#milestone-based-payments)
- [Handling Disputes](#handling-disputes)
- [Error Handling](#error-handling)
- [Performance Tips](#performance-tips)
- [Next Steps](#next-steps)

## Installation

Add the Trust Escrow SDK to your Rust project:

```toml
[dependencies]
trust-escrow-sdk = "2.0.0"
tokio = { version = "1.0", features = ["full"] }
solana-sdk = "1.18"
solana-client = "1.18"
```

## Quick Start

Here's a minimal example to get you started:

```rust
use trust_escrow_sdk::{CofreClient, Result};
use solana_sdk::{signature::Keypair, commitment_config::CommitmentConfig};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Connect to Solana
    let rpc = Arc::new(RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed()
    ));
    
    // 2. Create or load your keypair
    let payer = Arc::new(Keypair::new());
    
    // 3. Initialize the client
    let client = CofreClient::new(rpc, payer)?;
    
    // 4. Create a user account
    let signature = client.create_user("alice", Some("Freelance developer")).await?;
    println!("User created! Transaction: {}", signature);
    
    Ok(())
}
```

## Basic Concepts

### What is an Escrow?

An escrow is a financial arrangement where a third party holds and regulates payment of funds between two parties. In the context of freelance work:

- **Client** deposits funds for a project
- **Freelancer** delivers work according to agreed terms
- **Escrow system** ensures fair payment upon completion

### Key Components

1. **Users** - Individual accounts for clients and freelancers
2. **Jobs** - Work agreements with defined scope and payment
3. **Teams** - Groups of freelancers working together
4. **Milestones** - Breakdown of work into smaller, payable chunks
5. **Disputes** - Resolution mechanism when parties disagree

### Account Types (PDAs)

The SDK uses Program Derived Addresses (PDAs) for deterministic account management:

```rust
use trust_escrow_sdk::pda;

// Derive a user's PDA
let (user_pda, bump) = pda::find_user_pda(&authority);

// Derive a job's PDA
let (job_pda, bump) = pda::find_job_pda(&client, job_id);
```

## Your First Escrow

Let's build a complete escrow flow step by step:

### Step 1: Create Users

```rust
use trust_escrow_sdk::{CofreClient, Result};
use std::time::Duration;

async fn create_users(client: &CofreClient) -> Result<()> {
    // Create client user
    let client_sig = client.create_user(
        "tech_company", 
        Some("Technology startup looking for developers")
    ).await?;
    println!("Client created: {}", client_sig);
    
    // Note: In practice, the freelancer would create their own account
    // from their own wallet/keypair
    
    Ok(())
}
```

### Step 2: Create a Job

```rust
async fn create_job(client: &CofreClient) -> Result<(solana_sdk::pubkey::Pubkey, solana_sdk::signature::Signature)> {
    let (job_pda, signature) = client.create_job(
        "Build React Dashboard",
        "Need a responsive dashboard with user authentication and data visualization",
        5_000_000, // 0.005 SOL (about $1 at $200/SOL)
        Duration::from_secs(86400 * 14), // 2 weeks deadline
        false, // doesn't require a team
    ).await?;
    
    println!("Job created: {}", signature);
    Ok((job_pda, signature))
}
```

### Step 3: Fund the Escrow

```rust
async fn fund_job(client: &CofreClient, job_id: u64) -> Result<()> {
    let signature = client.fund_escrow(job_id).await?;
    println!("Escrow funded: {}", signature);
    Ok(())
}
```

### Step 4: Application Process

```rust
// Freelancer applies to the job
async fn apply_to_job(
    freelancer_client: &CofreClient, 
    job_pda: &solana_sdk::pubkey::Pubkey
) -> Result<()> {
    let signature = freelancer_client.apply_to_job(
        job_pda,
        "I'm a React expert with 5 years of experience. I can deliver this in 10 days."
    ).await?;
    
    println!("Application submitted: {}", signature);
    Ok(())
}

// Client accepts the application
async fn accept_application(
    client: &CofreClient,
    job_pda: &solana_sdk::pubkey::Pubkey,
    freelancer_pubkey: &solana_sdk::pubkey::Pubkey
) -> Result<()> {
    let signature = client.accept_application(job_pda, freelancer_pubkey).await?;
    println!("Application accepted: {}", signature);
    Ok(())
}
```

### Step 5: Work Submission and Approval

```rust
// Freelancer submits completed work
async fn submit_work(
    freelancer_client: &CofreClient,
    job_pda: &solana_sdk::pubkey::Pubkey
) -> Result<()> {
    let signature = freelancer_client.submit_work(
        job_pda,
        "https://github.com/freelancer/react-dashboard/pull/1"
    ).await?;
    
    println!("Work submitted: {}", signature);
    Ok(())
}

// Client approves and releases payment
async fn approve_work(
    client: &CofreClient,
    job_pda: &solana_sdk::pubkey::Pubkey
) -> Result<()> {
    let signature = client.approve_work(job_pda).await?;
    println!("Work approved, payment released: {}", signature);
    Ok(())
}
```

### Complete Example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let client = setup_client().await?;
    
    // 1. Create users
    create_users(&client).await?;
    
    // 2. Create job
    let (job_pda, _) = create_job(&client).await?;
    
    // 3. Fund escrow
    fund_job(&client, 1).await?;
    
    // 4. Handle applications (simplified for example)
    let freelancer_pubkey = solana_sdk::pubkey::Pubkey::new_unique();
    accept_application(&client, &job_pda, &freelancer_pubkey).await?;
    
    // 5. Work completion
    submit_work(&client, &job_pda).await?;
    approve_work(&client, &job_pda).await?;
    
    println!("Escrow completed successfully!");
    Ok(())
}
```

## Working with Teams

For larger projects, you can create teams of freelancers:

```rust
use trust_escrow_sdk::types::{MemberRole};

async fn create_team_project(client: &CofreClient) -> Result<()> {
    // 1. Create a team
    let (team_pda, _) = client.create_team(
        "Frontend Experts",
        "Specialized team for React/Vue.js projects"
    ).await?;
    
    // 2. Add team members
    let developer_key = solana_sdk::pubkey::Pubkey::new_unique();
    client.add_team_member(&team_pda, &developer_key, MemberRole::Admin).await?;
    
    let designer_key = solana_sdk::pubkey::Pubkey::new_unique();
    client.add_team_member(&team_pda, &designer_key, MemberRole::Member).await?;
    
    // 3. Create job requiring team
    let (_job_pda, _) = client.create_job(
        "E-commerce Platform",
        "Complete e-commerce solution with custom design",
        50_000_000, // 0.05 SOL
        Duration::from_secs(86400 * 30), // 30 days
        true, // requires_team = true
    ).await?;
    
    Ok(())
}
```

## Milestone-Based Payments

Break large projects into smaller, payable milestones:

```rust
use trust_escrow_sdk::types::MilestoneData;

async fn setup_milestone_project(client: &CofreClient) -> Result<()> {
    let job_id = 1u64;
    
    // Define milestones
    let milestones = vec![
        MilestoneData {
            title: "Project Setup & Architecture".to_string(),
            description: "Initial setup, database design, and project structure".to_string(),
            amount: 2_000_000, // 0.002 SOL
            deadline_duration: Duration::from_secs(86400 * 5), // 5 days
        },
        MilestoneData {
            title: "Core Features Implementation".to_string(),
            description: "User authentication, product catalog, shopping cart".to_string(),
            amount: 15_000_000, // 0.015 SOL
            deadline_duration: Duration::from_secs(86400 * 15), // 15 days
        },
        MilestoneData {
            title: "Payment Integration & Testing".to_string(),
            description: "Payment gateway, order processing, testing".to_string(),
            amount: 8_000_000, // 0.008 SOL
            deadline_duration: Duration::from_secs(86400 * 25), // 25 days
        },
    ];
    
    // Create all milestones at once
    let signatures = client.batch_create_milestones(job_id, milestones).await?;
    println!("Created {} milestones", signatures.len());
    
    // Submit and approve milestones individually
    client.submit_milestone(job_id, 0).await?;
    client.approve_milestone(job_id, 0).await?;
    println!("Milestone 0 completed!");
    
    Ok(())
}
```

## Handling Disputes

When work doesn't meet expectations:

```rust
async fn handle_dispute(client: &CofreClient) -> Result<()> {
    let job_id = 1u64;
    
    // 1. Raise a dispute
    let (dispute_pda, _) = client.raise_dispute(
        job_id,
        "The delivered code doesn't match specifications. Missing user authentication feature."
    ).await?;
    
    // 2. Submit additional evidence
    client.submit_evidence(
        job_id,
        "Screenshots showing missing login functionality: https://imgur.com/abc123"
    ).await?;
    
    // 3. Arbiter resolves (in practice, this would be done by a neutral party)
    client.resolve_dispute(
        job_id,
        70, // 70% to client
        30, // 30% to freelancer
    ).await?;
    
    println!("Dispute resolved!");
    Ok(())
}
```

## Error Handling

The SDK provides detailed error types for better error handling:

```rust
use trust_escrow_sdk::{EscrowError, Result};

async fn handle_errors_properly(client: &CofreClient) -> Result<()> {
    match client.create_user("", None).await {
        Ok(signature) => println!("User created: {}", signature),
        Err(EscrowError::Validation(msg)) => {
            println!("Invalid input: {}", msg);
            // Handle validation error - usually user input issue
        }
        Err(EscrowError::Network(msg)) => {
            println!("Network error: {}", msg);
            // Handle network error - retry logic might be appropriate
        }
        Err(EscrowError::InsufficientFunds(msg)) => {
            println!("Not enough funds: {}", msg);
            // Handle insufficient funds - prompt user to add SOL
        }
        Err(EscrowError::Unauthorized(msg)) => {
            println!("Permission denied: {}", msg);
            // Handle authorization error
        }
        Err(other) => {
            println!("Other error: {}", other);
            // Handle other error types
        }
    }
    
    Ok(())
}
```

## Performance Tips

### 1. Connection Pooling

Reuse client instances:

```rust
// Good: Reuse client
let client = Arc::new(CofreClient::new(rpc, payer)?);

// Bad: Create new client for each operation
// let client = CofreClient::new(rpc, payer)?; // Don't do this repeatedly
```

### 2. Batch Operations

Use batch operations when possible:

```rust
// Good: Batch milestone operations
let signatures = client.batch_create_milestones(job_id, milestones).await?;

// Less efficient: Create milestones one by one
// for milestone in milestones {
//     client.create_milestone(job_id, ...).await?;
// }
```

### 3. Commitment Levels

Choose appropriate commitment levels:

```rust
// For user interfaces - faster but less secure
let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::processed());

// For financial operations - slower but more secure  
let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::finalized());
```

## Next Steps

Now that you understand the basics:

1. **Read the [Concepts Guide](./concepts/escrow-basics.md)** - Deeper understanding of escrow principles
2. **Explore [Examples](../examples/)** - Real-world usage patterns
3. **Check [API Reference](./api-reference.md)** - Complete function documentation
4. **Review [Error Handling Guide](./concepts/error-handling.md)** - Advanced error handling patterns

## Common Patterns

### Creating a Simple Escrow Service

```rust
pub struct EscrowService {
    client: Arc<CofreClient>,
}

impl EscrowService {
    pub fn new(client: Arc<CofreClient>) -> Self {
        Self { client }
    }
    
    pub async fn create_simple_escrow(
        &self,
        title: &str,
        description: &str,
        amount_sol: f64,
        deadline_days: u64,
    ) -> Result<u64> {
        // Convert SOL to lamports
        let amount = trust_escrow_sdk::utils::ConversionUtils::sol_to_lamports(amount_sol);
        
        // Create job
        let (_, _) = self.client.create_job(
            title,
            description,
            amount,
            Duration::from_secs(86400 * deadline_days),
            false,
        ).await?;
        
        // Return job ID (in practice, you'd track this)
        Ok(1)
    }
}
```

## Development vs Production

### Development Setup

```rust
// Connect to devnet for testing
let rpc = Arc::new(RpcClient::new_with_commitment(
    "https://api.devnet.solana.com".to_string(),
    CommitmentConfig::confirmed()
));
```

### Production Setup

```rust
// Connect to mainnet for production
let rpc = Arc::new(RpcClient::new_with_commitment(
    "https://api.mainnet-beta.solana.com".to_string(),
    CommitmentConfig::finalized() // Higher security
));
```

## Troubleshooting

### Common Issues

1. **"Program account not found"** - Make sure you're connected to the right network
2. **"Insufficient funds"** - Ensure your wallet has enough SOL for transactions
3. **"Transaction timeout"** - Network congestion, try increasing timeout or retry
4. **"Invalid PDA"** - Check that you're using the correct program ID

### Getting Help

- Check the [API Documentation](./api-reference.md)
- Review [Examples](../examples/)
- Open an issue on [GitHub](https://github.com/davidcoachdev/Trust-Work-Escrow)

---

**Congratulations!** You now have a solid foundation for building escrow-powered applications with the Trust Escrow SDK. The escrow system provides security and trust for both clients and freelancers, making it perfect for freelance platforms, service marketplaces, and collaborative projects.
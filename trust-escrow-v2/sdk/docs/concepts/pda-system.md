# Understanding the PDA System

A comprehensive guide to Program Derived Addresses (PDAs) in the Trust Escrow system and how they enable secure, deterministic account management.

## What are Program Derived Addresses?

**Program Derived Addresses (PDAs)** are special types of accounts on Solana that are:

1. **Deterministic** - Always generate the same address given the same inputs
2. **Owned by Programs** - Only the program that derived them can sign transactions
3. **Secure** - Cannot be controlled by external private keys
4. **Predictable** - Clients can calculate addresses before accounts exist

## Why PDAs Matter for Escrow

In traditional systems, managing user accounts and data requires:
- Complex database relationships
- User authentication systems  
- Permission management
- Data consistency checks

PDAs solve these problems by:
- **Eliminating Database Dependencies** - All data lives on-chain
- **Providing Natural Access Control** - Programs control account modifications
- **Ensuring Data Integrity** - Blockchain guarantees consistency
- **Enabling Composability** - Other programs can interact predictably

## PDA Architecture in Trust Escrow

### Account Hierarchy

```
Trust Escrow Program
├── Config (Global Settings)
├── Arbiter Pool (Dispute Resolvers)
├── Users/
│   ├── User Account (alice)
│   ├── User Account (bob)  
│   └── User Account (carol)
├── Teams/
│   ├── Team Account (dev_team_1)
│   └── Team Account (design_team_1)
├── Jobs/
│   ├── Job Account (client=alice, id=1)
│   ├── Job Account (client=alice, id=2)
│   └── Job Account (client=bob, id=1)
├── Disputes/
│   ├── Dispute Account (job=alice_job_1)
│   └── Dispute Account (job=bob_job_1)
└── Milestones/
    ├── Milestone Account (job=alice_job_1, index=0)
    ├── Milestone Account (job=alice_job_1, index=1)
    └── Milestone Account (job=alice_job_2, index=0)
```

## PDA Seeds and Derivation

### How PDAs are Generated

Each PDA is derived using:
1. **Seed strings** - Human-readable identifiers
2. **Variable data** - User addresses, IDs, etc.
3. **Program ID** - The Trust Escrow program address
4. **Bump seed** - A number to ensure the address is off-curve

```rust
// General PDA derivation formula
let (pda, bump) = Pubkey::find_program_address(
    &[seed1, seed2, variable_data],
    &PROGRAM_ID
);
```

### User Account PDAs

**Purpose**: Store user profile information and wallet management.

```rust
use trust_escrow_sdk::pda;

// Derive user PDA from their wallet address
let user_authority = user_keypair.pubkey();
let (user_pda, bump) = pda::find_user_pda(&user_authority);

// Seeds: [b"user", authority.as_ref()]
```

**What's stored**:
- Username and bio
- Multiple wallet addresses (up to 5)
- Active wallet selection
- Creation timestamp

**Example**:
```rust
// Alice's wallet address: 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
// Alice's User PDA: 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
let alice_wallet = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU".parse().unwrap();
let (alice_user_pda, _) = pda::find_user_pda(&alice_wallet);
```

### Job Account PDAs

**Purpose**: Store job details, status, and payment information.

```rust
// Derive job PDA from client address and job ID
let client = user_keypair.pubkey();
let job_id = 42u64;
let (job_pda, bump) = pda::find_job_pda(&client, job_id);

// Seeds: [b"job", client.as_ref(), job_id.to_le_bytes().as_ref()]
```

**What's stored**:
- Job title, description, requirements
- Payment amount and deadline
- Current status (Created, InProgress, etc.)
- Assigned freelancer (if any)
- Escrow account reference

**Example**:
```rust
// Alice creates her first job
let (alice_job_1_pda, _) = pda::find_job_pda(&alice_wallet, 1);

// Alice creates her second job  
let (alice_job_2_pda, _) = pda::find_job_pda(&alice_wallet, 2);

// Bob creates his first job - different PDA even with same ID
let (bob_job_1_pda, _) = pda::find_job_pda(&bob_wallet, 1);

// All three PDAs are unique and deterministic
assert_ne!(alice_job_1_pda, alice_job_2_pda);
assert_ne!(alice_job_1_pda, bob_job_1_pda);
```

### Team Account PDAs

**Purpose**: Manage groups of freelancers working together.

```rust
// Derive team PDA from owner address
let team_owner = team_leader_keypair.pubkey();
let (team_pda, bump) = pda::find_team_pda(&team_owner);

// Seeds: [b"team", owner.as_ref()]
```

**What's stored**:
- Team name and description
- List of members with roles
- Creation timestamp

**Limitation**: Each user can only own one team. For multiple teams, use different owner addresses.

### Milestone Account PDAs

**Purpose**: Track individual milestones within a job.

```rust
// Derive milestone PDA from job address and milestone index
let job_pda = alice_job_1_pda; // From previous example
let milestone_index = 0u8;
let (milestone_pda, bump) = pda::find_milestone_pda(&job_pda, milestone_index);

// Seeds: [b"milestone", job.as_ref(), index.as_ref()]
```

**What's stored**:
- Milestone title and description  
- Payment amount and deadline
- Submission status
- Work deliverables (URLs, etc.)

**Example**:
```rust
// Alice's job has 3 milestones
let (milestone_0_pda, _) = pda::find_milestone_pda(&alice_job_1_pda, 0);
let (milestone_1_pda, _) = pda::find_milestone_pda(&alice_job_1_pda, 1);  
let (milestone_2_pda, _) = pda::find_milestone_pda(&alice_job_1_pda, 2);

// Each milestone has a unique PDA
assert_ne!(milestone_0_pda, milestone_1_pda);
assert_ne!(milestone_1_pda, milestone_2_pda);
```

### Dispute Account PDAs

**Purpose**: Handle disagreements between clients and freelancers.

```rust
// Derive dispute PDA from job address
let job_pda = alice_job_1_pda;
let (dispute_pda, bump) = pda::find_dispute_pda(&job_pda);

// Seeds: [b"dispute", job.as_ref()]
```

**What's stored**:
- Complainant (who raised the dispute)
- Evidence and documentation
- Assigned arbiter
- Resolution outcome

**Uniqueness**: One dispute per job maximum.

### Global Account PDAs

#### Config Account
**Purpose**: Store global system settings.

```rust
let (config_pda, bump) = pda::find_config_pda();
// Seeds: [b"config"]
```

**What's stored**:
- Platform fee rates
- Admin addresses
- System pause state

#### Arbiter Pool Account
**Purpose**: Manage the pool of dispute arbiters.

```rust
let (arbiter_pool_pda, bump) = pda::find_arbiter_pool_pda();
// Seeds: [b"arbiter_pool"]
```

**What's stored**:
- List of approved arbiters
- Arbiter stakes and reputations

## PDA Security Model

### Access Control

PDAs provide natural access control through program ownership:

```rust
// Only the Trust Escrow program can modify its PDAs
#[derive(Accounts)]
pub struct UpdateUser<'info> {
    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = user_account.bump,
        constraint = user_account.authority == authority.key()
    )]
    pub user_account: Account<'info, User>,
    
    pub authority: Signer<'info>, // Must be the user's authority
}
```

### Ownership Verification

```rust
// Verify that Alice can only modify her own user account
let alice_authority = alice_keypair.pubkey();
let (alice_user_pda, _) = pda::find_user_pda(&alice_authority);

// Alice's signature allows modification of alice_user_pda
// Bob's signature cannot modify alice_user_pda
```

### Cross-Program Invocations (CPI)

Other programs can interact with Trust Escrow PDAs:

```rust
// External program creates jobs through CPI
pub fn external_create_job(ctx: Context<ExternalCreateJob>) -> Result<()> {
    // Call Trust Escrow program
    let cpi_accounts = CreateJob {
        client: ctx.accounts.client.to_account_info(),
        job_account: ctx.accounts.job_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.trust_escrow_program.to_account_info(),
        cpi_accounts
    );
    
    trust_escrow::cpi::create_job(cpi_ctx, title, description, amount, deadline)?;
    Ok(())
}
```

## Working with PDAs in Your Application

### Frontend Integration

```typescript
// TypeScript example for web applications
import { PublicKey } from '@solana/web3.js';

class TrustEscrowClient {
    private programId: PublicKey;
    
    constructor(programId: PublicKey) {
        this.programId = programId;
    }
    
    // Derive user PDA on the frontend
    findUserPDA(authority: PublicKey): [PublicKey, number] {
        return PublicKey.findProgramAddressSync(
            [Buffer.from('user'), authority.toBuffer()],
            this.programId
        );
    }
    
    // Derive job PDA
    findJobPDA(client: PublicKey, jobId: number): [PublicKey, number] {
        const jobIdBuffer = Buffer.alloc(8);
        jobIdBuffer.writeBigUInt64LE(BigInt(jobId), 0);
        
        return PublicKey.findProgramAddressSync(
            [Buffer.from('job'), client.toBuffer(), jobIdBuffer],
            this.programId
        );
    }
}
```

### Backend Integration

```rust
// Rust backend service
use trust_escrow_sdk::{CofreClient, pda};

pub struct EscrowService {
    client: CofreClient,
}

impl EscrowService {
    pub async fn get_user_jobs(&self, user_authority: &Pubkey) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();
        
        // We need to track job IDs separately or scan for them
        for job_id in 0..100 { // In practice, use proper job ID tracking
            let (job_pda, _) = pda::find_job_pda(user_authority, job_id);
            
            if let Ok(job) = self.client.get_job(&job_pda).await {
                jobs.push(job);
            }
        }
        
        Ok(jobs)
    }
}
```

## PDA Best Practices

### 1. **Predictable Derivation**

Always derive PDAs the same way:

```rust
// Good: Consistent seed ordering
let (pda, _) = find_user_pda(&authority);

// Bad: Inconsistent or complex derivation
let seeds = if is_premium { 
    [b"premium_user", authority.as_ref()] 
} else { 
    [b"user", authority.as_ref()] 
};
```

### 2. **Efficient Seed Usage**

Use minimal, unique seeds:

```rust
// Good: Simple, unique seeds
let (pda, _) = find_job_pda(&client, job_id);

// Bad: Redundant or large seeds  
let title_bytes = title.as_bytes(); // Could be very long
let (pda, _) = Pubkey::find_program_address(
    &[b"job", client.as_ref(), title_bytes],
    &PROGRAM_ID
);
```

### 3. **Account Size Planning**

Account for future growth:

```rust
#[account]
pub struct User {
    pub authority: Pubkey,    // 32 bytes
    pub username: String,     // 4 + up to 50 = 54 bytes
    pub bio: Option<String>,  // 1 + 4 + up to 500 = 505 bytes  
    pub wallets: Vec<Pubkey>, // 4 + 5 * 32 = 164 bytes
    pub active_wallet: Pubkey,// 32 bytes
    pub created_at: i64,      // 8 bytes
    pub bump: u8,             // 1 byte
}

// Total: ~800 bytes + padding
```

### 4. **Gas Optimization**

Reuse derived PDAs within transactions:

```rust
// Good: Derive once, use multiple times
let (user_pda, _) = pda::find_user_pda(&authority);
let user = client.get_user(&user_pda).await?;
let updated_user = client.update_user(&user_pda, new_bio).await?;

// Bad: Re-derive repeatedly
let user = client.get_user(&pda::find_user_pda(&authority).0).await?;
let updated = client.update_user(&pda::find_user_pda(&authority).0, bio).await?;
```

## Common PDA Patterns

### 1. **Singleton Pattern** (Config, Arbiter Pool)

One global account per type:

```rust
// Always the same PDA regardless of who calls it
let (config_pda, _) = pda::find_config_pda();
```

### 2. **User-Owned Pattern** (User, Team)

One account per user:

```rust
// Each user has exactly one User account
let (user_pda, _) = pda::find_user_pda(&authority);
```

### 3. **Indexed Pattern** (Jobs, Milestones)

Multiple accounts per user with indices:

```rust
// User can have many jobs with different IDs
let (job_1_pda, _) = pda::find_job_pda(&client, 1);
let (job_2_pda, _) = pda::find_job_pda(&client, 2);
```

### 4. **Reference Pattern** (Disputes)

Account derived from another account's address:

```rust
// Dispute references a specific job
let (dispute_pda, _) = pda::find_dispute_pda(&job_pda);
```

## Troubleshooting PDA Issues

### Account Not Found

```rust
// Check if PDA derivation is correct
let (expected_pda, bump) = pda::find_user_pda(&authority);
let account = client.get_user(&expected_pda).await;

match account {
    Ok(user) => println!("Found user: {}", user.username),
    Err(e) => println!("Account not found: {:?}", e),
}
```

### Wrong Program Owner

```rust
// Verify the account is owned by the correct program
let account_info = connection.get_account(&pda).await?;
assert_eq!(account_info.owner, PROGRAM_ID);
```

### Insufficient Space

```rust
// Account creation with proper space calculation
let space = 8 + User::INIT_SPACE; // 8-byte discriminator + account data
let rent = connection.get_minimum_balance_for_rent_exemption(space).await?;
```

## Advanced PDA Concepts

### Cross-Program PDA References

```rust
// Reference PDA from another program
pub struct ExternalReference {
    pub trust_escrow_job: Pubkey, // PDA from Trust Escrow program
    pub our_tracking_data: String,
}
```

### PDA Chains

```rust
// Milestone PDA depends on Job PDA
let (job_pda, _) = pda::find_job_pda(&client, job_id);
let (milestone_pda, _) = pda::find_milestone_pda(&job_pda, 0);
```

### Composite Seeds

```rust
// Complex PDA with multiple variable components
let seeds = [
    b"complex",
    authority.as_ref(),
    &job_id.to_le_bytes(),
    &timestamp.to_le_bytes(),
];
let (complex_pda, bump) = Pubkey::find_program_address(&seeds, &PROGRAM_ID);
```

---

Understanding PDAs is crucial for working effectively with the Trust Escrow SDK. They provide the foundation for all account management and enable the secure, deterministic behavior that makes the escrow system trustworthy and predictable.
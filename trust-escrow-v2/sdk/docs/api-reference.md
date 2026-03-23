# 📚 Trust Escrow SDK - Complete Function Reference

**Trust Work Escrow v2 Rust SDK** - Complete guide to all 51 public functions with explanations, examples, and use cases.

---

## 🎯 Overview

The Trust Escrow SDK provides 51 public async functions organized into logical categories:
- **Utility Functions** (3) - Basic operations and helpers
- **User Management** (5) - User accounts and wallet management  
- **Job Lifecycle** (8) - Core escrow operations
- **Team Management** (3) - Team creation and member management
- **Dispute Resolution** (4) - Dispute handling and evidence submission
- **Milestone Payments** (6) - Milestone-based payment system
- **Batch Operations** (2) - Efficient bulk operations
- **Enhanced Queries** (4) - Advanced search and filtering
- **SDK Utilities** (8) - Validation, formatting, and dev tools
- **Event Monitoring** (4) - Real-time event system
- **Performance & Caching** (4) - Optimization and caching

---

## 🛠️ Utility Functions

### 1. `account_exists(&self, pubkey: &Pubkey) -> Result<bool>`
**Purpose:** Check if a Solana account exists on-chain  
**When to use:** Before attempting to read account data or validate account references  
**Example:**
```rust
let exists = client.account_exists(&job_pda).await?;
if !exists {
    return Err(EscrowError::account_not_found("Job account does not exist"));
}
```

### 2. `get_balance(&self, pubkey: &Pubkey) -> Result<u64>`
**Purpose:** Get SOL balance of any account in lamports  
**When to use:** Checking wallet balances before operations or displaying user balances  
**Example:**
```rust
let balance = client.get_balance(&user_wallet).await?;
println!("Balance: {} SOL", balance as f64 / 1_000_000_000.0);
```

### 3. `wait_for_confirmation(&self, signature: &Signature) -> Result<bool>`
**Purpose:** Wait for transaction confirmation with timeout  
**When to use:** After sending transactions when you need to ensure completion  
**Example:**
```rust
let sig = client.create_escrow(1, "Web Dev Project", "", 1_000_000_000).await?;
let confirmed = client.wait_for_confirmation(&sig).await?;
```

---

## 👤 User Management Functions

### 4. `create_user(&self, username: &str, bio: Option<&str>) -> Result<Signature>`
**Purpose:** Create a new user account in the Trust Escrow system  
**When to use:** First-time user onboarding  
**Parameters:**
- `username`: Unique identifier (3-32 chars)
- `bio`: Optional profile description (max 200 chars)  
**Example:**
```rust
let sig = client.create_user("alice_dev", Some("Full-stack developer")).await?;
println!("User created: {}", sig);
```

### 5. `add_wallet(&self, wallet: &Pubkey) -> Result<Signature>`
**Purpose:** Add additional wallet to user account (max 5 wallets)  
**When to use:** Users want to use multiple wallets or hardware wallets  
**Example:**
```rust
let hardware_wallet = Pubkey::from_str("...")?;
client.add_wallet(&hardware_wallet).await?;
```

### 6. `set_active_wallet(&self, wallet: &Pubkey) -> Result<Signature>`
**Purpose:** Change which wallet is used for transactions  
**When to use:** Switching between added wallets  
**Example:**
```rust
client.set_active_wallet(&hardware_wallet).await?;
```

### 7. `update_user(&self, bio: &str) -> Result<Signature>`
**Purpose:** Update user profile information  
**When to use:** Profile updates, skills changes, contact info  
**Example:**
```rust
client.update_user("Senior Rust developer specializing in Solana").await?;
```

### 8. `get_user(&self, user_pda: &Pubkey) -> Result<User>`
**Purpose:** Retrieve user account data  
**When to use:** Displaying profiles, checking user info before operations  
**Example:**
```rust
let user = client.get_user(&user_pda).await?;
println!("User: {} - {}", user.username, user.bio.unwrap_or_default());
```

---

## 💼 Job Lifecycle Functions (Core Escrow Operations)

### 9. `create_job(&self, job_id: u64, title: &str, description: &str, amount: u64) -> Result<Signature>`
**Purpose:** Create a new job posting (escrow contract)  
**When to use:** Client wants to hire freelancer for work  
**Parameters:**
- `job_id`: Unique job identifier for this client
- `title`: Job title (1-100 chars)
- `description`: Detailed job description  
- `amount`: Payment amount in lamports
**Example:**
```rust
let sig = client.create_job(
    1, 
    "Build Rust Web API", 
    "Need REST API with authentication and database integration",
    2_000_000_000 // 2 SOL
).await?;
```

### 10. `deposit_funds(&self, job: &Pubkey) -> Result<Signature>`
**Purpose:** Fund a job with the specified SOL amount  
**When to use:** After creating job, client deposits payment to escrow  
**Example:**
```rust
let (job_pda, _) = derive_job_pda(&client.payer().pubkey(), 1)?;
client.deposit_funds(&job_pda).await?;
```

### 11. `apply_to_job(&self, job: &Pubkey, proposal: &str) -> Result<Signature>`
**Purpose:** Freelancer submits application to job  
**When to use:** Freelancer wants to bid on available job  
**Example:**
```rust
let proposal = "I have 5 years Rust experience and can deliver in 2 weeks";
client.apply_to_job(&job_pda, proposal).await?;
```

### 12. `accept_application(&self, job: &Pubkey, freelancer: &Pubkey) -> Result<Signature>`
**Purpose:** Client accepts specific freelancer application  
**When to use:** After reviewing applications, client selects freelancer  
**Example:**
```rust
client.accept_application(&job_pda, &freelancer_pubkey).await?;
```

### 13. `submit_work(&self, job: &Pubkey, work_url: &str) -> Result<Signature>`
**Purpose:** Freelancer submits completed work  
**When to use:** Work is finished and ready for client review  
**Example:**
```rust
client.submit_work(&job_pda, "https://github.com/freelancer/project").await?;
```

### 14. `approve_work(&self, job: &Pubkey) -> Result<Signature>`
**Purpose:** Client approves work and releases payment  
**When to use:** Client is satisfied with delivered work  
**Example:**
```rust
client.approve_work(&job_pda).await?; // Releases payment to freelancer
```

### 15. `reject_work(&self, job: &Pubkey, reason: &str) -> Result<Signature>`
**Purpose:** Client rejects submitted work with reason  
**When to use:** Work doesn't meet requirements, needs revision  
**Example:**
```rust
client.reject_work(&job_pda, "Missing authentication module").await?;
```

### 16. `cancel_job(&self, job: &Pubkey) -> Result<Signature>`
**Purpose:** Cancel job and refund client (if no freelancer assigned)  
**When to use:** Client no longer needs work done  
**Example:**
```rust
client.cancel_job(&job_pda).await?; // Refunds deposited funds
```

---

## 👥 Team Management Functions

### 17. `create_team(&self, name: &str, description: &str) -> Result<(Pubkey, Signature)>`
**Purpose:** Create a team of freelancers  
**When to use:** Managing multiple freelancers on large projects  
**Returns:** Team PDA and transaction signature
**Example:**
```rust
let (team_pda, sig) = client.create_team(
    "Web Dev Team", 
    "Frontend and backend specialists"
).await?;
```

### 18. `add_team_member(&self, team: &Pubkey, member: &Pubkey, role: MemberRole) -> Result<Signature>`
**Purpose:** Add freelancer to existing team with specific role  
**When to use:** Building project teams with defined roles  
**Example:**
```rust
client.add_team_member(&team_pda, &freelancer, MemberRole::Developer).await?;
```

### 19. `get_team(&self, team_pda: &Pubkey) -> Result<Team>`
**Purpose:** Retrieve team information and member list  
**When to use:** Displaying team composition, checking team status  
**Example:**
```rust
let team = client.get_team(&team_pda).await?;
println!("Team: {} with {} members", team.name, team.members.len());
```

---

## ⚖️ Dispute Resolution Functions

### 20. `raise_dispute(&self, job_id: u64, evidence: &str) -> Result<(Pubkey, Signature)>`
**Purpose:** Open dispute on job with initial evidence  
**When to use:** Disagreement between client and freelancer  
**Returns:** Dispute PDA and transaction signature
**Example:**
```rust
let (dispute_pda, _) = client.raise_dispute(
    1, 
    "Work delivered doesn't match specifications. See attached screenshots."
).await?;
```

### 21. `submit_evidence(&self, job_id: u64, evidence: &str) -> Result<Signature>`
**Purpose:** Add additional evidence to existing dispute  
**When to use:** Providing more documentation during dispute process  
**Example:**
```rust
client.submit_evidence(1, "Client approved initial mockups via email").await?;
```

### 22. `resolve_dispute(&self, dispute: &Pubkey, winner: &Pubkey, client_amount: u64, freelancer_amount: u64) -> Result<Signature>`
**Purpose:** Arbiter resolves dispute with payment distribution  
**When to use:** Arbiter makes final decision on dispute outcome  
**Example:**
```rust
// 70% to freelancer, 30% to client
client.resolve_dispute(&dispute_pda, &freelancer, 600_000_000, 1_400_000_000).await?;
```

### 23. `get_dispute(&self, dispute_pda: &Pubkey) -> Result<Dispute>`
**Purpose:** Retrieve dispute information and evidence  
**When to use:** Displaying dispute status, arbiter review  
**Example:**
```rust
let dispute = client.get_dispute(&dispute_pda).await?;
println!("Dispute status: {:?}", dispute.status);
```

---

## 🎯 Milestone Payment Functions

### 24. `create_milestone(&self, job_id: u64, title: &str, description: &str, amount: u64, index: u8) -> Result<(Pubkey, Signature)>`
**Purpose:** Create payment milestone for job  
**When to use:** Breaking large projects into smaller payment chunks  
**Example:**
```rust
let (milestone_pda, _) = client.create_milestone(
    1,
    "Phase 1: Design",
    "UI/UX mockups and wireframes",  
    500_000_000, // 0.5 SOL
    0
).await?;
```

### 25. `submit_milestone(&self, milestone: &Pubkey, work_url: &str) -> Result<Signature>`
**Purpose:** Freelancer submits work for specific milestone  
**When to use:** Milestone work is completed  
**Example:**
```rust
client.submit_milestone(&milestone_pda, "https://figma.com/project/designs").await?;
```

### 26. `approve_milestone(&self, milestone: &Pubkey) -> Result<Signature>`
**Purpose:** Client approves milestone and releases payment  
**When to use:** Milestone work meets requirements  
**Example:**
```rust
client.approve_milestone(&milestone_pda).await?; // Releases milestone payment
```

### 27. `reject_milestone(&self, milestone: &Pubkey, reason: &str) -> Result<Signature>`
**Purpose:** Client rejects milestone work with feedback  
**When to use:** Milestone needs revision or doesn't meet specs  
**Example:**
```rust
client.reject_milestone(&milestone_pda, "Colors don't match brand guidelines").await?;
```

### 28. `get_milestone(&self, milestone_pda: &Pubkey) -> Result<Milestone>`
**Purpose:** Retrieve milestone information and status  
**When to use:** Tracking milestone progress, displaying status  
**Example:**
```rust
let milestone = client.get_milestone(&milestone_pda).await?;
println!("Milestone: {} - Status: {:?}", milestone.title, milestone.status);
```

### 29. `list_milestones(&self, job_id: u64) -> Result<Vec<(Pubkey, Milestone)>>`
**Purpose:** Get all milestones for a specific job  
**When to use:** Displaying project timeline, tracking overall progress  
**Example:**
```rust
let milestones = client.list_milestones(1).await?;
for (pda, milestone) in milestones {
    println!("{}: {} - {}", milestone.index, milestone.title, milestone.status);
}
```

---

## 📦 Batch Operations Functions

### 30. `batch_create_milestones(&self, job_id: u64, milestone_specs: Vec<MilestoneSpec>) -> Result<Vec<(Pubkey, Signature)>>`
**Purpose:** Create multiple milestones in one optimized batch  
**When to use:** Setting up complex projects with many milestones  
**Features:** Rate limiting (200ms between ops), individual error handling  
**Example:**
```rust
let specs = vec![
    MilestoneSpec { title: "Design".to_string(), amount: 500_000_000, index: 0 },
    MilestoneSpec { title: "Backend".to_string(), amount: 800_000_000, index: 1 },
    MilestoneSpec { title: "Frontend".to_string(), amount: 700_000_000, index: 2 },
];
let results = client.batch_create_milestones(1, specs).await?;
```

### 31. `batch_approve_milestones(&self, milestones: &[Pubkey]) -> Result<Vec<Signature>>`
**Purpose:** Approve multiple milestones efficiently  
**When to use:** Approving several completed milestones at once  
**Example:**
```rust
let milestone_pdas = vec![milestone1_pda, milestone2_pda, milestone3_pda];
let signatures = client.batch_approve_milestones(&milestone_pdas).await?;
```

---

## 🔍 Enhanced Query Functions

### 32. `list_escrows_with_pagination(&self, offset: usize, limit: usize) -> Result<Vec<Job>>`
**Purpose:** Get jobs with pagination for large datasets  
**When to use:** Building job marketplace UI, handling thousands of jobs  
**Example:**
```rust
let page1 = client.list_escrows_with_pagination(0, 20).await?; // First 20 jobs
let page2 = client.list_escrows_with_pagination(20, 20).await?; // Next 20
```

### 33. `list_escrows_by_status(&self, status: JobStatus) -> Result<Vec<Job>>`
**Purpose:** Filter jobs by current status  
**When to use:** Showing only active jobs, completed jobs, etc.  
**Example:**
```rust
let active_jobs = client.list_escrows_by_status(JobStatus::InProgress).await?;
let completed = client.list_escrows_by_status(JobStatus::Approved).await?;
```

### 34. `list_escrows_by_client(&self, client: &Pubkey) -> Result<Vec<Job>>`
**Purpose:** Get all jobs posted by specific client  
**When to use:** Client dashboard, portfolio display  
**Example:**
```rust
let my_jobs = client.list_escrows_by_client(&client.payer().pubkey()).await?;
```

### 35. `search_escrows(&self, filters: &EscrowFilters) -> Result<Vec<Job>>`
**Purpose:** Advanced search with multiple criteria  
**When to use:** Complex filtering in marketplace UI  
**Example:**
```rust
let filters = EscrowFilters::new()
    .with_min_amount(1_000_000_000)
    .with_status(JobStatus::ApplicationsOpen)
    .with_title_contains("rust");
let matching_jobs = client.search_escrows(&filters).await?;
```

---

## 🛠️ SDK Utility Functions

### 36. `validate_job_title(&self, title: &str) -> Result<()>`
**Purpose:** Validate job title meets requirements  
**When to use:** Form validation before creating jobs  
**Rules:** 1-100 characters, no special characters  
**Example:**
```rust
ValidationUtils::validate_job_title("Build Rust API")?; // OK
ValidationUtils::validate_job_title("")?; // Error: too short
```

### 37. `validate_amount(&self, amount: u64, min_amount: u64) -> Result<()>`
**Purpose:** Validate payment amount meets minimum requirements  
**When to use:** Before creating jobs or milestones  
**Example:**
```rust
ValidationUtils::validate_amount(2_000_000_000, MIN_JOB_AMOUNT)?; // OK
ValidationUtils::validate_amount(1000, MIN_JOB_AMOUNT)?; // Error: too low
```

### 38. `validate_milestone_specs(&self, specs: &[MilestoneSpec]) -> Result<()>`
**Purpose:** Validate milestone specifications before batch creation  
**When to use:** Before calling batch_create_milestones  
**Checks:** Indices, amounts, title lengths, duplicates  
**Example:**
```rust
ValidationUtils::validate_milestone_specs(&specs)?;
```

### 39. `get_recommended_fee(&self) -> Result<u64>`
**Purpose:** Get current recommended transaction fee  
**When to use:** Estimating transaction costs for users  
**Example:**
```rust
let fee = client.get_recommended_fee().await?;
println!("Estimated fee: {} lamports", fee);
```

### 40. `test_connection(&self) -> Result<bool>`
**Purpose:** Test RPC connection health  
**When to use:** App startup, connection troubleshooting  
**Example:**
```rust
let connected = client.test_connection().await?;
if !connected {
    eprintln!("Warning: RPC connection issues detected");
}
```

### 41. `format_balance(&self, lamports: u64) -> String`
**Purpose:** Format lamport amounts as human-readable SOL  
**When to use:** Displaying balances and amounts in UI  
**Example:**
```rust
let formatted = WalletUtils::format_balance(1_500_000_000);
println!("Balance: {}", formatted); // "1.5 SOL"
```

### 42. `calculate_total_milestone_amount(&self, specs: &[MilestoneSpec]) -> u64`
**Purpose:** Sum total payment across all milestones  
**When to use:** Validating milestone amounts match job total  
**Example:**
```rust
let total = client.calculate_total_milestone_amount(&specs);
assert_eq!(total, job_amount);
```

### 43. `with_retry<F, R>(&self, operation: F) -> Result<R>`
**Purpose:** Execute operation with automatic retry logic  
**When to use:** Network operations that may fail temporarily  
**Features:** Exponential backoff, configurable attempts  
**Example:**
```rust
let result = client.with_retry(|| {
    client.get_account_data(&pubkey)
}).await?;
```

---

## 📡 Event Monitoring Functions

### 44. `start_event_listener(&self, config: EventListenerConfig) -> EventSubscription`
**Purpose:** Start real-time event monitoring  
**When to use:** Building real-time UI updates, notifications  
**Example:**
```rust
let mut subscription = client.start_event_listener(EventListenerConfig::default());
while let Some(event) = subscription.recv().await {
    match event {
        EscrowEvent::JobCreated { job, amount, .. } => {
            println!("New job: {} for {} SOL", job, amount as f64 / 1e9);
        }
        _ => {}
    }
}
```

### 45. `get_recent_events(&self, limit: usize) -> Result<Vec<EscrowEvent>>`
**Purpose:** Fetch recent events from transaction history  
**When to use:** Displaying activity feed, catching up on events  
**Example:**
```rust
let events = client.get_recent_events(50).await?;
for event in events {
    println!("Event: {:?}", event);
}
```

### 46. `filter_events(&self, events: &[EscrowEvent], filter: &EventFilter) -> Vec<EscrowEvent>`
**Purpose:** Filter events by type, accounts, or users  
**When to use:** Showing only relevant events to specific users  
**Example:**
```rust
let filter = EventFilter::new().with_users(vec![user_pubkey]);
let user_events = client.filter_events(&all_events, &filter);
```

### 47. `parse_transaction_logs(&self, logs: &[String]) -> Vec<EscrowEvent>`
**Purpose:** Extract events from Solana transaction logs  
**When to use:** Custom event processing, building analytics  
**Example:**
```rust
let events = client.parse_transaction_logs(&transaction.meta.log_messages);
```

---

## ⚡ Performance & Caching Functions

### 48. `get_account_data_cached(&self, pubkey: &Pubkey) -> Result<Vec<u8>>`
**Purpose:** Get account data with intelligent caching  
**When to use:** Frequent account reads, performance optimization  
**Features:** TTL caching, memory efficiency  
**Example:**
```rust
let data = client.get_account_data_cached(&job_pda).await?;
let job = Job::try_from_slice(&data)?;
```

### 49. `clear_cache(&self)`
**Purpose:** Clear all cached account data  
**When to use:** Force refresh, memory cleanup  
**Example:**
```rust
client.clear_cache(); // Force fresh data on next reads
```

### 50. `get_cache_stats(&self) -> CacheStats`
**Purpose:** Get cache performance metrics  
**When to use:** Performance monitoring, optimization  
**Example:**
```rust
let stats = client.get_cache_stats();
println!("Cache hit rate: {:.1}%", stats.hit_rate * 100.0);
```

### 51. `optimize_rpc_batch(&self, operations: Vec<Operation>) -> Result<Vec<Response>>`
**Purpose:** Batch multiple RPC calls for efficiency  
**When to use:** High-throughput operations, reducing latency  
**Example:**
```rust
let operations = vec![
    Operation::GetAccount(job_pda1),
    Operation::GetAccount(job_pda2),
    Operation::GetAccount(job_pda3),
];
let responses = client.optimize_rpc_batch(operations).await?;
```

---

## 🎯 Usage Patterns & Best Practices

### Complete Escrow Flow
```rust
// 1. Create and fund job
let sig = client.create_job(1, "Build Web App", "React + Node.js", 5_000_000_000).await?;
let (job_pda, _) = derive_job_pda(&client.payer().pubkey(), 1)?;
client.deposit_funds(&job_pda).await?;

// 2. Freelancer applies
client.apply_to_job(&job_pda, "I can build this in 3 weeks").await?;

// 3. Client accepts
client.accept_application(&job_pda, &freelancer_pubkey).await?;

// 4. Work delivery
client.submit_work(&job_pda, "https://github.com/freelancer/project").await?;

// 5. Payment release  
client.approve_work(&job_pda).await?;
```

### Milestone-Based Project
```rust
// Create job with milestones
let specs = vec![
    MilestoneSpec { title: "Design".to_string(), amount: 1_000_000_000, index: 0 },
    MilestoneSpec { title: "Backend".to_string(), amount: 2_000_000_000, index: 1 },
    MilestoneSpec { title: "Frontend".to_string(), amount: 2_000_000_000, index: 2 },
];

let milestone_results = client.batch_create_milestones(1, specs).await?;

// Process each milestone individually
for (milestone_pda, _) in milestone_results {
    // Freelancer submits work
    client.submit_milestone(&milestone_pda, "work_url").await?;
    
    // Client approves
    client.approve_milestone(&milestone_pda).await?;
}
```

### Event-Driven UI Updates
```rust
// Monitor all job events
let mut subscription = client.start_event_listener(EventListenerConfig::default());
tokio::spawn(async move {
    while let Some(event) = subscription.recv().await {
        match event {
            EscrowEvent::JobCreated { job, title, amount, .. } => {
                ui.notify_new_job(&job, &title, amount);
            }
            EscrowEvent::WorkSubmitted { job, freelancer, .. } => {
                ui.notify_work_submitted(&job, &freelancer);
            }
            EscrowEvent::WorkApproved { job, amount, .. } => {
                ui.notify_payment_released(&job, amount);
            }
            _ => {}
        }
    }
});
```

---

## 🚨 Error Handling & Validation

Each function returns `Result<T>` with comprehensive error types:

- **`EscrowError::InvalidParameter`** - Input validation failures
- **`EscrowError::AccountNotFound`** - Missing or invalid accounts  
- **`EscrowError::InsufficientFunds`** - Not enough SOL for operation
- **`EscrowError::UnauthorizedOperation`** - Permission denied
- **`EscrowError::InvalidState`** - Contract state prevents operation
- **`EscrowError::NetworkError`** - RPC or network issues

Always handle errors appropriately:
```rust
match client.create_job(1, "title", "desc", amount).await {
    Ok(signature) => println!("Job created: {}", signature),
    Err(EscrowError::InvalidParameter(msg)) => eprintln!("Validation error: {}", msg),
    Err(EscrowError::InsufficientFunds(_)) => eprintln!("Not enough SOL to create job"),
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

---

*This reference covers all 51 public functions in the Trust Escrow SDK. Each function is production-ready with comprehensive error handling, input validation, and performance optimization.*
# Epic #2: Core Library (Rust SDK) - Technical Design

**Epic ID**: #24 | **Status**: In Progress | **Date**: 2026-03-23

## Architecture Overview

This document details the technical design for the Trust Work Escrow v2 Rust SDK, a comprehensive client library that replaces the legacy TypeScript escrow-core and provides seamless integration with the Solana-based smart contract.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CLIENT APPLICATIONS                             │
├─────────────────────────────────────────────────────────────────────┤
│  CLI App  │  TUI App  │  Web Backend  │  Mobile Backend  │  DApps   │
└─────────┬─────────┬─────────────┬─────────────┬─────────────┬───────┘
          │         │             │             │             │
          └─────────┼─────────────┼─────────────┼─────────────┘
                    │             │             │
┌───────────────────┼─────────────┼─────────────┼─────────────────────┐
│                   │    TRUST ESCROW V2 SDK │             │         │
│                   ▼             ▼             ▼                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    CofreClient                              │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐   │   │
│  │  │ Config  │ │  Jobs   │ │ Escrow  │ │    Disputes     │   │   │
│  │  │ Manager │ │ Manager │ │ Manager │ │     Manager     │   │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                             │                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                Core Modules                                 │   │
│  │ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐   │   │
│  │ │ Account │ │ Instruc │ │  Error  │ │      Types      │   │   │
│  │ │ Manager │ │ Builder │ │ Handler │ │   (from IDL)    │   │   │
│  │ └─────────┘ └─────────┘ └─────────┘ └─────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────┼───────────────────────────────────────┘
                              │
┌─────────────────────────────┼───────────────────────────────────────┐
│            SOLANA ECOSYSTEM │                                       │
│ ┌─────────────────────────────────────────────────────────────┐   │
│ │                  Solana RPC Client                         │   │
│ │ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐   │   │
│ │ │   TX    │ │ Account │ │ Program │ │    Connection   │   │   │
│ │ │ Builder │ │ Fetcher │ │  Loader │ │     Manager     │   │   │
│ │ └─────────┘ └─────────┘ └─────────┘ └─────────────────┘   │   │
│ └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────┼───────────────────────────────────────┘
                              │
┌─────────────────────────────┼───────────────────────────────────────┐
│         TRUST ESCROW V2 SMART CONTRACT                             │
│                             │                                       │
│ Program ID: TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB           │
│                             │                                       │
│ ┌─────────────────────────────────────────────────────────────┐   │
│ │ Instructions (31)          │ Accounts (7 PDAs)              │   │
│ │ ├─ Config Management (4)   │ ├─ Config                      │   │
│ │ ├─ User Management (5)     │ ├─ User                        │   │
│ │ ├─ Team Management (2)     │ ├─ Team                        │   │
│ │ ├─ Job Management (8)      │ ├─ Job                         │   │
│ │ ├─ Dispute Resolution (8)  │ ├─ ArbiterPool                │   │
│ │ └─ Milestone Management (4)│ ├─ Dispute                     │   │
│ │                            │ └─ Milestone                   │   │
│ └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Module Architecture

### Core Client Design

```rust
// Main client interface
pub struct CofreClient {
    rpc_client: Arc<RpcClient>,
    program_id: Pubkey,
    cluster: Cluster,
    commitment: CommitmentConfig,
}

impl CofreClient {
    // Phase 1: Foundation ✅ IMPLEMENTED
    pub fn new(cluster: Cluster, commitment: CommitmentConfig) -> Result<Self, CofreError> { }
    pub fn with_rpc_url(rpc_url: String, commitment: CommitmentConfig) -> Result<Self, CofreError> { }
    
    // Phase 2: Core Operations 🔄 IN PROGRESS
    pub async fn create_escrow(&self, params: CreateEscrowParams, signer: &dyn Signer) -> Result<Signature, CofreError> { }
    pub async fn fund_escrow(&self, escrow: Pubkey, amount: u64, signer: &dyn Signer) -> Result<Signature, CofreError> { }
    pub async fn release_payment(&self, escrow: Pubkey, signer: &dyn Signer) -> Result<Signature, CofreError> { }
    pub async fn refund_escrow(&self, escrow: Pubkey, signer: &dyn Signer) -> Result<Signature, CofreError> { }
    
    // Phase 3: Advanced Features ⏳ PENDING
    pub async fn submit_dispute(&self, job: Pubkey, evidence: String, signer: &dyn Signer) -> Result<Signature, CofreError> { }
    pub async fn create_milestone(&self, job: Pubkey, params: MilestoneParams, signer: &dyn Signer) -> Result<Signature, CofreError> { }
    
    // Phase 4: Testing & Documentation ⏳ PENDING
    // Integration tests, documentation, examples
}
```

### Module Organization

```
sdk/src/
├── lib.rs              # Public API exports and main documentation
├── client.rs           # CofreClient main interface
├── types.rs            # IDL-generated types and custom types  
├── accounts.rs         # PDA derivation and account utilities
├── error.rs            # Error types and error handling
├── constants.rs        # Protocol constants and configuration
│
├── instructions/       # Instruction builders (31 total)
│   ├── mod.rs          # Module exports
│   ├── config.rs       # Config management (4 instructions)
│   ├── user.rs         # User management (5 instructions)
│   ├── team.rs         # Team management (2 instructions)
│   ├── job.rs          # Job management (8 instructions)
│   ├── dispute.rs      # Dispute resolution (8 instructions)
│   └── milestone.rs    # Milestone management (4 instructions)
│
└── utils/              # Utility functions
    ├── mod.rs          # Module exports
    ├── transaction.rs  # Transaction building helpers
    ├── validation.rs   # Input validation
    └── formatting.rs   # Display and formatting utilities
```

## Data Architecture

### Account State Management

```rust
// Account state management with caching
pub struct AccountManager {
    rpc_client: Arc<RpcClient>,
    cache: Arc<Mutex<HashMap<Pubkey, CachedAccount>>>,
    cache_duration: Duration,
}

#[derive(Debug, Clone)]
struct CachedAccount {
    data: AccountData,
    last_updated: Instant,
    account_info: AccountInfo,
}

impl AccountManager {
    // Fetch account with caching
    pub async fn get_account<T: AccountDeserialize>(&self, pubkey: Pubkey) -> Result<T, CofreError> {
        // 1. Check cache first
        if let Some(cached) = self.get_from_cache(&pubkey) {
            return Ok(cached);
        }
        
        // 2. Fetch from RPC
        let account_info = self.rpc_client.get_account(&pubkey).await?;
        
        // 3. Deserialize and cache
        let account_data = T::try_deserialize(&mut account_info.data.as_slice())?;
        self.cache_account(pubkey, account_data.clone(), account_info);
        
        Ok(account_data)
    }
    
    // Invalidate cache for account updates
    pub fn invalidate_cache(&self, pubkey: &Pubkey) { }
    
    // Batch account fetching
    pub async fn get_multiple_accounts(&self, pubkeys: Vec<Pubkey>) -> Result<Vec<Option<AccountInfo>>, CofreError> { }
}
```

### PDA Derivation System

```rust
// Centralized PDA derivation with type safety
pub mod accounts {
    use solana_sdk::pubkey::Pubkey;
    
    // Trait for type-safe PDA derivation
    pub trait PdaAccount {
        fn derive_pda(program_id: &Pubkey, seeds: Self::Seeds) -> (Pubkey, u8);
        type Seeds;
    }
    
    // Config account PDA
    pub struct ConfigPda;
    impl PdaAccount for ConfigPda {
        type Seeds = ();
        fn derive_pda(program_id: &Pubkey, _: Self::Seeds) -> (Pubkey, u8) {
            Pubkey::find_program_address(&[b"config"], program_id)
        }
    }
    
    // User account PDA
    pub struct UserPda;
    impl PdaAccount for UserPda {
        type Seeds = Pubkey;
        fn derive_pda(program_id: &Pubkey, authority: Self::Seeds) -> (Pubkey, u8) {
            Pubkey::find_program_address(&[b"user", authority.as_ref()], program_id)
        }
    }
    
    // Job account PDA
    pub struct JobPda;
    impl PdaAccount for JobPda {
        type Seeds = (Pubkey, String);
        fn derive_pda(program_id: &Pubkey, (client, job_id): Self::Seeds) -> (Pubkey, u8) {
            Pubkey::find_program_address(&[b"job", client.as_ref(), job_id.as_bytes()], program_id)
        }
    }
    
    // ... additional PDA types
}
```

## Transaction Architecture

### Manual Transaction Building

Due to Anchor client trait bound issues with `Arc<dyn Signer>`, the SDK implements manual transaction construction while maintaining type safety:

```rust
// Transaction building with manual construction
pub struct InstructionBuilder {
    program_id: Pubkey,
}

impl InstructionBuilder {
    // Generic instruction builder
    pub fn build_instruction<T: AnchorSerialize>(
        &self,
        instruction_data: T,
        accounts: Vec<AccountMeta>,
        instruction_discriminator: [u8; 8],
    ) -> Result<Instruction, CofreError> {
        // 1. Serialize instruction data
        let mut data = instruction_discriminator.to_vec();
        instruction_data.serialize(&mut data)?;
        
        // 2. Build instruction
        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }
    
    // Specific instruction builders
    pub fn create_job(
        &self,
        params: CreateJobParams,
        client: Pubkey,
        client_user: Pubkey,
        job: Pubkey,
        system_program: Pubkey,
    ) -> Result<Instruction, CofreError> {
        let accounts = vec![
            AccountMeta::new(client, true),      // signer
            AccountMeta::new(client_user, false), // writable
            AccountMeta::new(job, false),        // writable
            AccountMeta::new_readonly(system_program, false),
        ];
        
        self.build_instruction(
            params,
            accounts,
            CREATE_JOB_DISCRIMINATOR,
        )
    }
    
    // ... 30 more instruction builders
}

// Transaction submission with retry logic
pub struct TransactionManager {
    rpc_client: Arc<RpcClient>,
    max_retries: usize,
    retry_delay: Duration,
}

impl TransactionManager {
    pub async fn submit_transaction(
        &self,
        transaction: &Transaction,
        signers: &[&dyn Signer],
    ) -> Result<Signature, CofreError> {
        let mut attempts = 0;
        
        loop {
            // 1. Get recent blockhash
            let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;
            
            // 2. Sign transaction
            let mut tx = transaction.clone();
            tx.partial_sign(&signers, recent_blockhash);
            
            // 3. Simulate before submission
            let simulate_result = self.rpc_client.simulate_transaction(&tx).await?;
            if simulate_result.value.err.is_some() {
                return Err(CofreError::Transaction(format!(
                    "Simulation failed: {:?}",
                    simulate_result.value.err
                )));
            }
            
            // 4. Submit transaction
            match self.rpc_client.send_and_confirm_transaction(&tx).await {
                Ok(signature) => return Ok(signature),
                Err(e) if attempts < self.max_retries => {
                    attempts += 1;
                    tokio::time::sleep(self.retry_delay).await;
                    continue;
                }
                Err(e) => return Err(CofreError::SolanaRpc(e)),
            }
        }
    }
}
```

## Error Handling Architecture

### Comprehensive Error System

```rust
// Main error enum with context
#[derive(Debug, thiserror::Error)]
pub enum CofreError {
    #[error("Solana RPC error: {0}")]
    SolanaRpc(#[from] solana_client::client_error::ClientError),
    
    #[error("Anchor error: {code} - {message}")]
    AnchorProgram { code: u32, message: String },
    
    #[error("Account error: {0}")]
    Account(#[from] AccountError),
    
    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

// Detailed error types
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Account not found: {account}")]
    NotFound { account: String },
    
    #[error("Account has insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Invalid account owner: expected {expected}, got {actual}")]
    InvalidOwner { expected: String, actual: String },
    
    #[error("Account data deserialization failed: {reason}")]
    DeserializationFailed { reason: String },
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Transaction simulation failed: {reason}")]
    SimulationFailed { reason: String },
    
    #[error("Transaction signing failed: {reason}")]
    SigningFailed { reason: String },
    
    #[error("Transaction confirmation timeout")]
    ConfirmationTimeout,
    
    #[error("Insufficient compute budget: required {required}, available {available}")]
    InsufficientComputeBudget { required: u32, available: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid job status: current {current}, required {required}")]
    InvalidJobStatus { current: String, required: String },
    
    #[error("Invalid amount: {amount} (must be > 0)")]
    InvalidAmount { amount: u64 },
    
    #[error("String too long: {length} chars (max {max})")]
    StringTooLong { length: usize, max: usize },
    
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
}

// Error context and recovery suggestions
impl CofreError {
    pub fn with_context(self, context: impl Into<String>) -> Self {
        // Add additional context to error
        match self {
            CofreError::Account(e) => CofreError::Account(e.with_context(context)),
            _ => self,
        }
    }
    
    pub fn recovery_suggestion(&self) -> Option<&str> {
        match self {
            CofreError::Account(AccountError::NotFound { .. }) => {
                Some("Ensure the account exists and is properly initialized")
            }
            CofreError::Account(AccountError::InsufficientFunds { .. }) => {
                Some("Add more funds to the account or reduce the transaction amount")
            }
            CofreError::Transaction(TransactionError::ConfirmationTimeout) => {
                Some("The transaction may still succeed. Check its status or retry")
            }
            _ => None,
        }
    }
}
```

## Type System Architecture

### IDL Integration with Build Script

```rust
// build.rs - IDL processing at compile time
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../target/idl/trust_escrow_v2.json");
    
    // 1. Read IDL file
    let idl_path = Path::new("../target/idl/trust_escrow_v2.json");
    if !idl_path.exists() {
        panic!("IDL file not found. Please run 'anchor build' first.");
    }
    
    let idl_content = fs::read_to_string(idl_path)
        .expect("Failed to read IDL file");
    
    // 2. Parse IDL
    let idl: anchor_lang::idl::Idl = serde_json::from_str(&idl_content)
        .expect("Failed to parse IDL JSON");
    
    // 3. Generate types
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_types.rs");
    
    let generated_code = generate_types_from_idl(&idl);
    fs::write(&dest_path, generated_code)
        .expect("Failed to write generated types");
    
    // 4. Validate program ID
    let expected_program_id = "TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB";
    if idl.address != expected_program_id {
        panic!("Program ID mismatch: expected {}, got {}", expected_program_id, idl.address);
    }
}

fn generate_types_from_idl(idl: &anchor_lang::idl::Idl) -> String {
    // Generate Rust structs from IDL types
    // Implementation generates all account types, instruction types, etc.
    // This ensures 100% compatibility with smart contract
    format!(r#"
        // Auto-generated types from IDL
        use anchor_lang::prelude::*;
        
        #[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
        pub struct ConfigAccount {{
            // Generated from IDL
        }}
        
        // ... all other types generated from IDL
    "#)
}
```

### Type-Safe Parameter Builders

```rust
// Builder pattern for complex operations
#[derive(Debug, Clone)]
pub struct CreateJobParams {
    pub job_id: String,
    pub title: String,
    pub description: String,
    pub required_skills: Vec<String>,
    pub budget: u64,
    pub deadline: i64,
    pub job_type: JobType,
}

impl CreateJobParams {
    pub fn builder() -> CreateJobParamsBuilder {
        CreateJobParamsBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct CreateJobParamsBuilder {
    job_id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    required_skills: Vec<String>,
    budget: Option<u64>,
    deadline: Option<i64>,
    job_type: Option<JobType>,
}

impl CreateJobParamsBuilder {
    pub fn job_id(mut self, job_id: impl Into<String>) -> Self {
        self.job_id = Some(job_id.into());
        self
    }
    
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    pub fn budget(mut self, budget: u64) -> Self {
        self.budget = Some(budget);
        self
    }
    
    pub fn deadline_days_from_now(mut self, days: u32) -> Self {
        let deadline = chrono::Utc::now().timestamp() + (days as i64 * 24 * 60 * 60);
        self.deadline = Some(deadline);
        self
    }
    
    pub fn skill(mut self, skill: impl Into<String>) -> Self {
        self.required_skills.push(skill.into());
        self
    }
    
    pub fn build(self) -> Result<CreateJobParams, ValidationError> {
        // Validation
        let job_id = self.job_id.ok_or_else(|| ValidationError::MissingField("job_id"))?;
        let title = self.title.ok_or_else(|| ValidationError::MissingField("title"))?;
        let description = self.description.ok_or_else(|| ValidationError::MissingField("description"))?;
        let budget = self.budget.ok_or_else(|| ValidationError::MissingField("budget"))?;
        let deadline = self.deadline.ok_or_else(|| ValidationError::MissingField("deadline"))?;
        let job_type = self.job_type.ok_or_else(|| ValidationError::MissingField("job_type"))?;
        
        // Validation rules
        if job_id.len() > 32 {
            return Err(ValidationError::StringTooLong { length: job_id.len(), max: 32 });
        }
        if budget == 0 {
            return Err(ValidationError::InvalidAmount { amount: budget });
        }
        
        Ok(CreateJobParams {
            job_id,
            title,
            description,
            required_skills: self.required_skills,
            budget,
            deadline,
            job_type,
        })
    }
}
```

## Performance Architecture

### Caching Strategy

```rust
// Multi-level caching system
pub struct CacheManager {
    // L1: In-memory cache for frequently accessed accounts
    memory_cache: Arc<Mutex<LruCache<Pubkey, CachedAccount>>>,
    
    // L2: Persistent cache for session data
    persistent_cache: Option<PersistentCache>,
    
    // Configuration
    memory_cache_size: usize,
    cache_duration: Duration,
}

impl CacheManager {
    pub async fn get_account<T>(&self, pubkey: Pubkey) -> Option<T>
    where
        T: AccountDeserialize + Clone + Send + 'static,
    {
        // 1. Check memory cache first (fastest)
        if let Some(cached) = self.get_from_memory_cache(&pubkey) {
            if !cached.is_expired() {
                return Some(cached.data);
            }
        }
        
        // 2. Check persistent cache
        if let Some(ref persistent) = self.persistent_cache {
            if let Some(cached) = persistent.get(&pubkey).await {
                if !cached.is_expired() {
                    // Update memory cache
                    self.set_memory_cache(pubkey, cached.clone());
                    return Some(cached.data);
                }
            }
        }
        
        None
    }
    
    pub async fn set_account<T>(&self, pubkey: Pubkey, data: T, account_info: AccountInfo)
    where
        T: Clone + Send + 'static,
    {
        let cached = CachedAccount {
            data: data.clone(),
            last_updated: Instant::now(),
            account_info,
        };
        
        // Update both caches
        self.set_memory_cache(pubkey, cached.clone());
        
        if let Some(ref persistent) = self.persistent_cache {
            persistent.set(pubkey, cached).await;
        }
    }
}

// Connection pooling for high throughput
pub struct ConnectionPool {
    pools: Vec<Arc<RpcClient>>,
    current: AtomicUsize,
    pool_size: usize,
}

impl ConnectionPool {
    pub fn new(rpc_urls: Vec<String>, pool_size: usize) -> Self {
        let pools = rpc_urls
            .into_iter()
            .cycle()
            .take(pool_size)
            .map(|url| Arc::new(RpcClient::new(url)))
            .collect();
        
        Self {
            pools,
            current: AtomicUsize::new(0),
            pool_size,
        }
    }
    
    pub fn get_client(&self) -> Arc<RpcClient> {
        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.pool_size;
        self.pools[index].clone()
    }
}
```

### Batch Operations

```rust
// Batch account fetching for efficiency
impl CofreClient {
    pub async fn get_multiple_jobs(&self, job_pubkeys: Vec<Pubkey>) -> Result<Vec<Option<JobAccount>>, CofreError> {
        // 1. Check cache for existing accounts
        let mut cached_accounts = Vec::new();
        let mut missing_pubkeys = Vec::new();
        
        for pubkey in &job_pubkeys {
            if let Some(cached) = self.account_manager.get_from_cache(pubkey) {
                cached_accounts.push(Some(cached));
            } else {
                cached_accounts.push(None);
                missing_pubkeys.push(*pubkey);
            }
        }
        
        // 2. Batch fetch missing accounts
        if !missing_pubkeys.is_empty() {
            let account_infos = self.rpc_client
                .get_multiple_accounts(&missing_pubkeys)
                .await?;
            
            // 3. Process and cache results
            for (pubkey, account_info) in missing_pubkeys.into_iter().zip(account_infos) {
                if let Some(account_info) = account_info {
                    let job_account = JobAccount::try_deserialize(&mut account_info.data.as_slice())?;
                    self.account_manager.cache_account(pubkey, job_account, account_info);
                }
            }
        }
        
        // 4. Return complete results
        let mut results = Vec::new();
        for pubkey in job_pubkeys {
            results.push(self.account_manager.get_from_cache(&pubkey));
        }
        
        Ok(results)
    }
}
```

## Security Architecture

### Input Validation Framework

```rust
// Comprehensive validation system
pub trait Validator<T> {
    type Error;
    fn validate(&self, value: &T) -> Result<(), Self::Error>;
}

// String validation
pub struct StringValidator {
    max_length: Option<usize>,
    min_length: Option<usize>,
    allowed_chars: Option<regex::Regex>,
}

impl Validator<String> for StringValidator {
    type Error = ValidationError;
    
    fn validate(&self, value: &String) -> Result<(), Self::Error> {
        if let Some(max) = self.max_length {
            if value.len() > max {
                return Err(ValidationError::StringTooLong {
                    length: value.len(),
                    max,
                });
            }
        }
        
        if let Some(min) = self.min_length {
            if value.len() < min {
                return Err(ValidationError::StringTooShort {
                    length: value.len(),
                    min,
                });
            }
        }
        
        if let Some(ref pattern) = self.allowed_chars {
            if !pattern.is_match(value) {
                return Err(ValidationError::InvalidCharacters {
                    value: value.clone(),
                });
            }
        }
        
        Ok(())
    }
}

// Amount validation
pub struct AmountValidator {
    min_amount: u64,
    max_amount: u64,
}

impl Validator<u64> for AmountValidator {
    type Error = ValidationError;
    
    fn validate(&self, value: &u64) -> Result<(), Self::Error> {
        if *value < self.min_amount {
            return Err(ValidationError::AmountTooLow {
                amount: *value,
                min: self.min_amount,
            });
        }
        
        if *value > self.max_amount {
            return Err(ValidationError::AmountTooHigh {
                amount: *value,
                max: self.max_amount,
            });
        }
        
        Ok(())
    }
}

// Job ID validation (alphanumeric, max 32 chars)
pub fn validate_job_id() -> StringValidator {
    StringValidator {
        max_length: Some(32),
        min_length: Some(1),
        allowed_chars: Some(regex::Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()),
    }
}

// Budget validation (min 0.001 SOL, max 1000 SOL)
pub fn validate_budget() -> AmountValidator {
    AmountValidator {
        min_amount: 1_000_000,        // 0.001 SOL in lamports
        max_amount: 1_000_000_000_000, // 1000 SOL in lamports
    }
}
```

### Transaction Security

```rust
// Secure transaction building with validation
impl InstructionBuilder {
    pub fn build_secure_instruction<T: AnchorSerialize>(
        &self,
        instruction_data: T,
        accounts: Vec<AccountMeta>,
        instruction_discriminator: [u8; 8],
        required_signers: Vec<Pubkey>,
    ) -> Result<Instruction, CofreError> {
        // 1. Validate accounts match expected patterns
        self.validate_account_structure(&accounts, &required_signers)?;
        
        // 2. Validate instruction data
        self.validate_instruction_data(&instruction_data)?;
        
        // 3. Build instruction
        let mut data = instruction_discriminator.to_vec();
        instruction_data.serialize(&mut data)?;
        
        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data,
        })
    }
    
    fn validate_account_structure(
        &self,
        accounts: &[AccountMeta],
        required_signers: &[Pubkey],
    ) -> Result<(), CofreError> {
        // Check that all required signers are present and marked as signers
        for required_signer in required_signers {
            let found = accounts
                .iter()
                .any(|acc| acc.pubkey == *required_signer && acc.is_signer);
            
            if !found {
                return Err(CofreError::Validation(ValidationError::MissingSigner {
                    signer: *required_signer,
                }));
            }
        }
        
        Ok(())
    }
}
```

## Testing Architecture

### Test Organization Strategy

```rust
// Unit tests for core functionality
#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = CofreClient::new(Cluster::Devnet, CommitmentConfig::confirmed())
            .expect("Failed to create client");
        assert_eq!(client.cluster, Cluster::Devnet);
    }
    
    #[tokio::test]
    async fn test_pda_derivation() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        
        let (user_pda, bump) = derive_user_pda(&program_id, &authority);
        
        // Verify PDA is derived correctly
        let expected = Pubkey::find_program_address(
            &[b"user", authority.as_ref()],
            &program_id
        );
        
        assert_eq!(user_pda, expected.0);
        assert_eq!(bump, expected.1);
    }
}

// Integration tests with devnet
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    // Test configuration for devnet testing
    struct TestSetup {
        client: CofreClient,
        payer: Keypair,
        program_id: Pubkey,
    }
    
    impl TestSetup {
        async fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let payer = Keypair::new();
            
            // Fund payer account on devnet
            let client = CofreClient::new(Cluster::Devnet, CommitmentConfig::confirmed())?;
            
            // Request airdrop for testing
            let signature = client.rpc_client
                .request_airdrop(&payer.pubkey(), 1_000_000_000) // 1 SOL
                .await?;
            
            client.rpc_client
                .confirm_transaction(&signature)
                .await?;
            
            Ok(Self {
                client,
                payer,
                program_id: "TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB".parse().unwrap(),
            })
        }
    }
    
    #[tokio::test]
    async fn test_create_user_flow() -> Result<(), Box<dyn std::error::Error>> {
        let setup = TestSetup::new().await?;
        
        // Test user creation
        let user_params = CreateUserParams::builder()
            .bio("Test user".to_string())
            .build()?;
        
        let signature = setup.client
            .create_user(user_params, &setup.payer)
            .await?;
        
        // Verify user account was created
        let user_pda = derive_user_pda(&setup.program_id, &setup.payer.pubkey()).0;
        let user_account = setup.client.get_user(user_pda).await?;
        
        assert_eq!(user_account.authority, setup.payer.pubkey());
        
        Ok(())
    }
}
```

## Deployment Architecture

### Package Configuration

```toml
# Cargo.toml
[package]
name = "trust-escrow-v2-sdk"
version = "0.1.0"
edition = "2021"
description = "Official Rust SDK for Trust Work Escrow v2 protocol on Solana"
license = "MIT"
repository = "https://github.com/davidcoachdev/Trust-Work-Escrow"
documentation = "https://docs.rs/trust-escrow-v2-sdk"
keywords = ["solana", "escrow", "blockchain", "freelancing", "web3"]
categories = ["api-bindings", "cryptography::cryptocurrencies"]
readme = "README.md"

[dependencies]
# Core Solana dependencies
anchor-client = { version = "0.32.0", features = ["async"] }
anchor-lang = "0.32.0"
solana-sdk = "2.1.0"
solana-client = "2.1.0"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Utilities
log = "0.4"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.0"

# Optional features
lru = { version = "0.12", optional = true }

[features]
default = ["caching"]
caching = ["lru"]

[dev-dependencies]
tokio-test = "0.4"
```

### Documentation Strategy

```rust
//! # Trust Work Escrow v2 SDK
//!
//! This crate provides a comprehensive Rust SDK for interacting with the Trust Work Escrow v2
//! protocol on Solana. The SDK enables developers to build applications that leverage
//! decentralized escrow services for freelance work, job posting, and payment processing.
//!
//! ## Quick Start
//!
//! ```rust
//! use trust_escrow_v2_sdk::{CofreClient, Cluster, CommitmentConfig};
//! use solana_sdk::commitment_config::CommitmentLevel;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client
//!     let client = CofreClient::new(
//!         Cluster::Devnet,
//!         CommitmentConfig {
//!             commitment: CommitmentLevel::Confirmed,
//!         }
//!     )?;
//!
//!     // Create job
//!     let job_params = CreateJobParams::builder()
//!         .job_id("web-dev-001")
//!         .title("Build a Web3 DApp")
//!         .budget(5_000_000_000) // 5 SOL
//!         .deadline_days_from_now(30)
//!         .skill("Rust")
//!         .skill("Solana")
//!         .build()?;
//!
//!     let (job_pubkey, signature) = client
//!         .create_job(job_params, &payer_keypair)
//!         .await?;
//!
//!     println!("Job created: {}", job_pubkey);
//!     println!("Transaction: {}", signature);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture Overview
//!
//! The SDK is organized into several core modules:
//!
//! - [`CofreClient`] - Main client interface for all operations
//! - [`accounts`] - PDA derivation and account management utilities  
//! - [`types`] - IDL-generated types and parameter builders
//! - [`instructions`] - Low-level instruction builders (31 instructions)
//! - [`error`] - Comprehensive error handling
//!
//! ## Key Concepts
//!
//! ### Escrow Lifecycle
//!
//! 1. **Job Creation**: Client posts a job with requirements and budget
//! 2. **Application**: Freelancers apply to jobs they're interested in
//! 3. **Acceptance**: Client selects and accepts a freelancer's application
//! 4. **Work Phase**: Freelancer completes work and submits deliverables
//! 5. **Completion**: Client approves work and payment is released
//!
//! ### Dispute Resolution
//!
//! If issues arise, either party can raise a dispute:
//! 1. Submit evidence supporting their position
//! 2. Arbiter reviews evidence and makes a decision
//! 3. Funds are distributed according to arbiter's ruling
//!
//! ### Milestone Support
//!
//! Large projects can be broken into milestones:
//! 1. Create milestones with specific deliverables and payment amounts
//! 2. Freelancer submits work for each milestone
//! 3. Client approves milestones and partial payments are released
//!
//! ## Examples
//!
//! See the [`examples`] directory for comprehensive usage examples:
//! - Basic job posting and management
//! - Escrow creation and lifecycle
//! - Dispute submission and resolution
//! - Milestone-based project management

/// Main client for interacting with Trust Work Escrow v2 protocol
pub struct CofreClient { /* ... */ }

impl CofreClient {
    /// Create a new client instance for the specified cluster
    ///
    /// # Arguments
    ///
    /// * `cluster` - Solana cluster to connect to (Devnet, Testnet, Mainnet)
    /// * `commitment` - Transaction confirmation commitment level
    ///
    /// # Example
    ///
    /// ```rust
    /// use trust_escrow_v2_sdk::{CofreClient, Cluster, CommitmentConfig};
    /// use solana_sdk::commitment_config::CommitmentLevel;
    ///
    /// let client = CofreClient::new(
    ///     Cluster::Devnet,
    ///     CommitmentConfig {
    ///         commitment: CommitmentLevel::Confirmed,
    ///     }
    /// )?;
    /// ```
    pub fn new(cluster: Cluster, commitment: CommitmentConfig) -> Result<Self, CofreError> { /* ... */ }
}
```

---

**Related Documents**:
- Epic Proposal: `proposal.md`
- Epic Specifications: `specs.md`  
- Task Breakdown: `tasks.md`
- GitHub Issues: #24, #26, #27, #28
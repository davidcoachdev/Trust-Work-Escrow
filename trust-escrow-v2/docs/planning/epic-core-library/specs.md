# Epic #2: Core Library (Rust SDK) - Specifications

**Epic ID**: #24 | **Status**: In Progress | **Date**: 2026-03-23

## Technical Requirements

This document defines the exact specifications for the Trust Work Escrow v2 Rust SDK that will replace the legacy TypeScript escrow-core library.

## Functional Requirements

### FR.1 - Smart Contract Integration

#### FR.1.1 - IDL-Based Type Generation
- **MUST** generate Rust types from `trust_escrow_v2.json` IDL
- **MUST** ensure 100% compatibility with smart contract accounts and instructions
- **MUST** handle all 31 instructions defined in the smart contract
- **SHALL** use `build.rs` to detect IDL changes and regenerate types

#### FR.1.2 - Program Integration
- **MUST** integrate with Program ID: `TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB`
- **MUST** support devnet, testnet, and mainnet-beta cluster configurations
- **SHALL** provide cluster-specific configuration options

#### FR.1.3 - Instruction Coverage
**MUST** implement wrappers for all 31 smart contract instructions:

**Config Management (4 instructions):**
- `initialize_config` - Global protocol configuration
- `pause` - Emergency pause functionality  
- `unpause` - Resume protocol operations
- `withdraw_treasury` - Treasury fund management

**User Management (5 instructions):**
- `create_user` - User profile creation
- `add_wallet` - Multi-wallet support (max 5)
- `set_active_wallet` - Wallet switching
- `update_user` - Profile updates
- `update_treasury` - Treasury address updates

**Team Management (2 instructions):**
- `create_team` - Team creation for collaborative work
- `add_team_member` - Team member management

**Job Management (8 instructions):**
- `create_job` - Job posting creation
- `deposit_funds` - Client funding
- `apply_to_job` - Freelancer applications
- `accept_application` - Client acceptance
- `submit_work` - Work delivery
- `approve_work` - Client approval
- `reject_work` - Work rejection
- `cancel_job` - Job cancellation

**Dispute Resolution (8 instructions):**
- `create_arbiter_pool` - Arbiter pool management
- `add_arbiter` - Arbiter addition
- `remove_arbiter` - Arbiter removal  
- `raise_dispute` - Dispute initiation
- `submit_evidence` - Evidence submission
- `assign_arbiter` - Arbiter assignment
- `resolve_dispute` - Dispute resolution
- `finalize_dispute_payouts` - Payment finalization

**Milestone Management (4 instructions):**
- `create_milestone` - Milestone creation
- `submit_milestone` - Milestone delivery
- `approve_milestone` - Milestone approval
- `reject_milestone` - Milestone rejection

### FR.2 - Account Management

#### FR.2.1 - PDA Derivation
**MUST** provide utility functions for all Program Derived Address (PDA) types:

| Account | Seed Pattern | Implementation Required |
|---------|--------------|------------------------|
| Config | `b"config"` | `derive_config_pda()` |
| User | `b"user", authority` | `derive_user_pda(authority)` |
| Team | `b"team", owner` | `derive_team_pda(owner)` |
| Job | `b"job", client, job_id` | `derive_job_pda(client, job_id)` |
| ArbiterPool | `b"arbiter_pool"` | `derive_arbiter_pool_pda()` |
| Dispute | `b"dispute", job` | `derive_dispute_pda(job)` |
| Milestone | `b"milestone", job, index` | `derive_milestone_pda(job, index)` |

#### FR.2.2 - Account State Management
- **MUST** provide functions to fetch and deserialize all account types
- **MUST** handle account existence checks with proper error handling
- **SHALL** cache account data for performance optimization
- **MAY** provide account update notification mechanisms

### FR.3 - Transaction Building

#### FR.3.1 - Manual Transaction Construction
Due to Anchor client trait bound issues, **MUST** implement manual transaction building:
- **MUST** create instruction builders for all 31 instructions
- **MUST** handle account meta construction properly
- **MUST** support both `Transaction` and `VersionedTransaction` types
- **SHALL** provide transaction simulation before submission

#### FR.3.2 - Transaction Signing and Submission
- **MUST** support multiple signer types (`Keypair`, `Arc<dyn Signer>`)
- **MUST** handle transaction submission with retry logic
- **MUST** provide transaction status monitoring
- **SHALL** support batch transaction submission

### FR.4 - Error Handling

#### FR.4.1 - Error Classification
**MUST** provide comprehensive error handling covering:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CofreError {
    #[error("Solana RPC error: {0}")]
    SolanaRpc(#[from] solana_client::client_error::ClientError),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Account not found: {account}")]
    AccountNotFound { account: String },
    
    #[error("Anchor program error: {code} - {message}")]
    AnchorProgram { code: u32, message: String },
    
    #[error("Invalid program ID, expected: {expected}, got: {actual}")]
    InvalidProgramId { expected: String, actual: String },
    
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Job status error: current {current}, required {required}")]
    InvalidJobStatus { current: String, required: String },
    
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
    
    #[error("Configuration error: {0}")]
    Config(String),
}
```

#### FR.4.2 - Error Recovery
- **MUST** provide clear error messages with context
- **SHOULD** suggest remediation steps for common errors
- **MAY** implement automatic retry for transient errors

## Non-Functional Requirements

### NFR.1 - Performance

#### NFR.1.1 - Response Times
- **MUST** complete RPC calls within 5 seconds under normal conditions
- **SHOULD** cache account data to reduce redundant RPC calls
- **MAY** implement connection pooling for high throughput scenarios

#### NFR.1.2 - Resource Usage
- **MUST** maintain memory usage under 100MB for typical operations
- **SHOULD** minimize CPU usage through efficient data structures
- **SHALL** avoid blocking operations in async contexts

### NFR.2 - Reliability

#### NFR.2.1 - Fault Tolerance
- **MUST** handle network disconnections gracefully
- **MUST** provide proper cleanup for failed transactions
- **SHALL** implement retry logic for transient failures
- **SHOULD** validate inputs before transaction submission

#### NFR.2.2 - Data Consistency
- **MUST** ensure transaction atomicity
- **MUST** validate account state before operations
- **SHALL** detect and handle account state changes

### NFR.3 - Usability

#### NFR.3.1 - API Design
- **MUST** follow Rust naming conventions (`snake_case` functions, `PascalCase` types)
- **MUST** provide comprehensive documentation with examples
- **SHALL** use builder patterns for complex operations
- **SHOULD** minimize required boilerplate code

#### NFR.3.2 - Educational Value
- **MUST** include concept explanations in module documentation
- **SHOULD** provide workflow examples for common use cases
- **MAY** include interactive examples in documentation

### NFR.4 - Maintainability

#### NFR.4.1 - Code Quality
- **MUST** achieve 90%+ test coverage
- **MUST** pass all Clippy lints with default configuration
- **SHALL** use consistent error handling patterns
- **SHOULD** minimize external dependencies

#### NFR.4.2 - Documentation
- **MUST** document all public APIs with rustdoc
- **MUST** provide README with quick start guide
- **SHALL** include architecture decision records (ADRs)
- **SHOULD** maintain changelog for version tracking

## Interface Specifications

### API.1 - Core Client Interface

```rust
pub struct CofreClient {
    rpc_client: RpcClient,
    program_id: Pubkey,
    cluster: Cluster,
}

impl CofreClient {
    /// Create new client instance
    pub fn new(cluster: Cluster, commitment: CommitmentConfig) -> Result<Self, CofreError>;
    
    /// Create client with custom RPC endpoint
    pub fn with_rpc_url(rpc_url: String, commitment: CommitmentConfig) -> Result<Self, CofreError>;
    
    // Core escrow operations (Phase 2)
    pub async fn create_escrow(&self, params: CreateEscrowParams, signer: &dyn Signer) -> Result<Signature, CofreError>;
    pub async fn fund_escrow(&self, escrow: Pubkey, amount: u64, signer: &dyn Signer) -> Result<Signature, CofreError>;
    pub async fn release_payment(&self, escrow: Pubkey, signer: &dyn Signer) -> Result<Signature, CofreError>;
    pub async fn refund_escrow(&self, escrow: Pubkey, signer: &dyn Signer) -> Result<Signature, CofreError>;
    
    // Job management
    pub async fn create_job(&self, params: CreateJobParams, signer: &dyn Signer) -> Result<(Pubkey, Signature), CofreError>;
    pub async fn get_job(&self, job_pubkey: Pubkey) -> Result<JobAccount, CofreError>;
    pub async fn list_user_jobs(&self, user: Pubkey) -> Result<Vec<JobAccount>, CofreError>;
    
    // Account utilities
    pub async fn get_user(&self, user_pubkey: Pubkey) -> Result<UserAccount, CofreError>;
    pub async fn get_config(&self) -> Result<ConfigAccount, CofreError>;
}
```

### API.2 - Account Derivation Utilities

```rust
pub mod accounts {
    use solana_sdk::pubkey::Pubkey;
    use crate::error::CofreError;

    /// Derive Config PDA
    pub fn derive_config_pda(program_id: &Pubkey) -> (Pubkey, u8);
    
    /// Derive User PDA
    pub fn derive_user_pda(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8);
    
    /// Derive Job PDA  
    pub fn derive_job_pda(program_id: &Pubkey, client: &Pubkey, job_id: &str) -> (Pubkey, u8);
    
    /// Derive Team PDA
    pub fn derive_team_pda(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8);
    
    /// Derive Arbiter Pool PDA
    pub fn derive_arbiter_pool_pda(program_id: &Pubkey) -> (Pubkey, u8);
    
    /// Derive Dispute PDA
    pub fn derive_dispute_pda(program_id: &Pubkey, job: &Pubkey) -> (Pubkey, u8);
    
    /// Derive Milestone PDA
    pub fn derive_milestone_pda(program_id: &Pubkey, job: &Pubkey, index: u8) -> (Pubkey, u8);
}
```

### API.3 - Type Definitions

```rust
// Generated from IDL
pub use types::*;

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

#[derive(Debug, Clone)]
pub struct CreateEscrowParams {
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub amount: u64,
    pub timeout: i64,
    pub requirements: String,
}

// Account state structs (generated from IDL)
pub struct JobAccount {
    pub client: Pubkey,
    pub freelancer: Option<Pubkey>,
    pub amount: u64,
    pub status: JobStatus,
    pub created_at: i64,
    pub deadline: i64,
    pub title: String,
    pub description: String,
    // ... additional fields from smart contract
}
```

## Integration Requirements

### INT.1 - Solana Integration

#### INT.1.1 - Cluster Support
- **MUST** support all Solana clusters: `devnet`, `testnet`, `mainnet-beta`
- **SHALL** provide cluster-specific program ID validation
- **SHOULD** include cluster detection from RPC endpoint

#### INT.1.2 - Transaction Features
- **MUST** support Solana transaction v0 format
- **SHALL** handle compute budget optimization
- **MAY** implement priority fee calculation

### INT.2 - Anchor Framework Integration

#### INT.2.1 - IDL Processing
- **MUST** process IDL at build time using `build.rs`
- **MUST** generate type-safe account and instruction builders
- **SHALL** validate IDL version compatibility

#### INT.2.2 - Account Constraints
- **MUST** validate all Anchor account constraints before transaction submission
- **SHALL** provide clear error messages for constraint violations
- **SHOULD** implement automatic account derivation where possible

## Testing Requirements

### TST.1 - Unit Testing

#### TST.1.1 - Coverage Requirements
- **MUST** achieve minimum 90% test coverage
- **MUST** test all public API functions
- **SHALL** test error conditions and edge cases
- **SHOULD** include property-based tests for critical functions

#### TST.1.2 - Test Organization
```
tests/
├── unit/                  # Unit tests for individual modules
│   ├── client_test.rs
│   ├── accounts_test.rs
│   ├── instructions_test.rs
│   └── types_test.rs
├── integration/           # Integration tests with devnet
│   ├── escrow_flow_test.rs
│   ├── job_lifecycle_test.rs
│   └── dispute_flow_test.rs
└── e2e/                  # End-to-end workflow tests
    ├── complete_job_test.rs
    └── dispute_resolution_test.rs
```

### TST.2 - Integration Testing

#### TST.2.1 - Devnet Testing
- **MUST** test against live devnet deployment
- **SHALL** use deterministic test accounts
- **SHOULD** clean up test accounts after execution

#### TST.2.2 - Performance Testing
- **SHOULD** include performance benchmarks for critical operations
- **MAY** implement stress testing for high-throughput scenarios

## Security Requirements

### SEC.1 - Input Validation

#### SEC.1.1 - Parameter Validation
- **MUST** validate all user inputs before transaction construction
- **MUST** prevent integer overflow in amount calculations
- **SHALL** validate string inputs for length and content
- **SHOULD** sanitize user-provided strings

#### SEC.1.2 - Account Validation
- **MUST** verify account ownership before operations
- **MUST** validate account types match expected schemas
- **SHALL** check account existence before dereferencing

### SEC.2 - Transaction Security

#### SEC.2.1 - Signature Verification
- **MUST** ensure proper transaction signing
- **SHALL** validate signer authority for operations
- **SHOULD** implement transaction replay protection

#### SEC.2.2 - Error Information
- **MUST NOT** expose sensitive information in error messages
- **SHALL** log security-relevant events appropriately
- **SHOULD** implement rate limiting for repeated failures

## Deployment Requirements

### DEP.1 - Distribution

#### DEP.1.1 - Cargo Publishing
- **MUST** publish to crates.io as `trust-escrow-v2-sdk`
- **SHALL** follow semantic versioning (semver)
- **SHOULD** include comprehensive package metadata

#### DEP.1.2 - Documentation Hosting
- **MUST** generate docs.rs documentation
- **SHALL** include comprehensive examples in documentation
- **SHOULD** provide quick start guide

### DEP.2 - Version Management

#### DEP.2.1 - Compatibility
- **MUST** maintain backward compatibility within major versions
- **SHALL** provide migration guides for breaking changes
- **SHOULD** implement feature flags for optional functionality

---

**Related Documents**:
- Epic Proposal: `proposal.md`
- Technical Design: `design.md`  
- Task Breakdown: `tasks.md`
- GitHub Issues: #24, #26, #27, #28
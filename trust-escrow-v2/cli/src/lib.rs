//! Trust Work Escrow CLI library
//!
//! This library provides the command-line interface functionality
//! for interacting with the Trust Work Escrow v2 protocol.

use clap::{Parser, Subcommand};

pub mod commands;

/// Trust Work Escrow CLI - Decentralized freelance escrow on Solana
#[derive(Parser)]
#[command(name = "trust-escrow")]
#[command(author = "Trust Work Team <dev@trustwork.com>")]
#[command(version = "0.1.0")]
#[command(about = "Command-line interface for Trust Work Escrow v2 protocol")]
#[command(long_about = None)]
pub struct Cli {
    /// Configuration file path
    #[arg(long, value_name = "FILE")]
    pub config: Option<String>,

    /// Network to connect to (localnet, devnet, mainnet-beta)
    #[arg(short, long, value_name = "NETWORK")]
    pub network: Option<String>,

    /// Wallet keypair file path
    #[arg(short, long, value_name = "FILE")]
    pub wallet: Option<String>,

    /// RPC URL to connect to
    #[arg(long, value_name = "URL")]
    pub rpc_url: Option<String>,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    pub output: String,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// User profile management
    User {
        #[command(subcommand)]
        action: UserCommands,
    },
    
    /// Job posting and management
    Job {
        #[command(subcommand)]
        action: JobCommands,
    },
    
    /// Milestone tracking and management
    Milestone {
        #[command(subcommand)]
        action: MilestoneCommands,
    },
    
    /// Payment and financial operations
    Payment {
        #[command(subcommand)]
        action: PaymentCommands,
    },
    
    /// Dispute resolution
    Dispute {
        #[command(subcommand)]
        action: DisputeCommands,
    },
    
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    
    /// Check system status and connectivity
    Status,
    
    /// Request SOL airdrop (devnet/testnet only)
    Airdrop {
        /// Amount of SOL to request
        #[arg(default_value = "1")]
        amount: f64,
    },
}

/// User management commands
#[derive(Subcommand)]
pub enum UserCommands {
    /// Create new user profile
    Create {
        /// User's display name
        #[arg(short, long)]
        name: String,
        /// Optional bio/description
        #[arg(short, long)]
        bio: Option<String>,
    },
    /// Show user profile
    Show {
        /// User's public key (optional, defaults to wallet)
        address: Option<String>,
    },
    /// Update user profile
    Update {
        /// New display name
        #[arg(short, long)]
        name: Option<String>,
        /// New bio/description
        #[arg(short, long)]
        bio: Option<String>,
    },
    /// Add wallet to user profile
    AddWallet {
        /// Wallet address to add
        address: String,
    },
    /// Set active wallet
    SetWallet {
        /// Wallet address to make active
        address: String,
    },
}

/// Job management commands
#[derive(Subcommand)]
pub enum JobCommands {
    /// Create new job posting
    Create {
        /// Job title
        #[arg(short, long)]
        title: String,
        /// Job description
        #[arg(short, long)]
        description: String,
        /// Payment amount in SOL
        #[arg(short, long)]
        amount: f64,
        /// Required skills (comma-separated)
        #[arg(short, long)]
        skills: Option<String>,
    },
    /// List jobs (by client or available)
    List {
        /// Show only jobs created by wallet
        #[arg(long)]
        my_jobs: bool,
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show job details
    Show {
        /// Job ID
        job_id: String,
    },
    /// Apply to a job
    Apply {
        /// Job ID
        job_id: String,
        /// Cover letter/proposal
        #[arg(short, long)]
        proposal: String,
    },
    /// Accept application (job poster only)
    Accept {
        /// Job ID
        job_id: String,
        /// Applicant's address
        applicant: String,
    },
    /// Submit work (freelancer only)
    Submit {
        /// Job ID
        job_id: String,
        /// Work submission details
        #[arg(short, long)]
        details: String,
    },
    /// Approve work (job poster only)
    Approve {
        /// Job ID
        job_id: String,
    },
    /// Reject work (job poster only)
    Reject {
        /// Job ID
        job_id: String,
        /// Reason for rejection
        #[arg(short, long)]
        reason: String,
    },
    /// Cancel job (job poster only)
    Cancel {
        /// Job ID
        job_id: String,
    },
}

/// Milestone management commands
#[derive(Subcommand)]
pub enum MilestoneCommands {
    /// Create milestone for job
    Create {
        /// Job ID
        job_id: String,
        /// Milestone description
        #[arg(short, long)]
        description: String,
        /// Payment amount for milestone
        #[arg(short, long)]
        amount: f64,
    },
    /// List milestones for job
    List {
        /// Job ID
        job_id: String,
    },
    /// Submit milestone (freelancer)
    Submit {
        /// Job ID
        job_id: String,
        /// Milestone index
        milestone_id: u64,
        /// Submission details
        #[arg(short, long)]
        details: String,
    },
    /// Approve milestone (client)
    Approve {
        /// Job ID
        job_id: String,
        /// Milestone index
        milestone_id: u64,
    },
    /// Reject milestone (client)
    Reject {
        /// Job ID
        job_id: String,
        /// Milestone index
        milestone_id: u64,
        /// Rejection reason
        #[arg(short, long)]
        reason: String,
    },
}

/// Payment commands
#[derive(Subcommand)]
pub enum PaymentCommands {
    /// Show wallet balance
    Balance,
    /// Show payment history
    History,
    /// Deposit funds to job
    Deposit {
        /// Job ID
        job_id: String,
        /// Amount to deposit
        amount: f64,
    },
    /// Withdraw treasury funds (admin only)
    Withdraw {
        /// Amount to withdraw
        amount: f64,
    },
}

/// Dispute commands
#[derive(Subcommand)]
pub enum DisputeCommands {
    /// Raise dispute for job
    Raise {
        /// Job ID
        job_id: String,
        /// Dispute reason/description
        #[arg(short, long)]
        reason: String,
    },
    /// Submit evidence for dispute
    Evidence {
        /// Job ID
        job_id: String,
        /// Evidence description
        #[arg(short, long)]
        evidence: String,
    },
    /// List disputes
    List {
        /// Show only disputes involving wallet
        #[arg(long)]
        my_disputes: bool,
    },
    /// Show dispute details
    Show {
        /// Job ID
        job_id: String,
    },
}

/// Configuration commands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Initialize configuration file
    Init {
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Set configuration value
    Set {
        /// Configuration key (e.g., network.cluster)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
}
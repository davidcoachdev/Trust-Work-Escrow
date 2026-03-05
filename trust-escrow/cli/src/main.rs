use anyhow::Result;
use clap::{Parser, Subcommand};
use escrow_core::*;

#[derive(Parser)]
#[command(name = "escrow")]
#[command(about = "Trust Work Escrow CLI — Solana escrow for freelancers")]
struct Cli {
    #[arg(long, global = true)]
    keypair: Option<String>,
    #[arg(long, global = true, default_value = "http://127.0.0.1:8899")]
    url: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize program config (admin only)
    Init {
        #[arg(long)]
        treasury: String,
    },
    /// Create a new job
    Create {
        title: String,
        #[arg(long)]
        amount: f64,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long)]
        arbiter: String,
        #[arg(long)]
        job_id: u64,
        #[arg(long)]
        deadline: Option<i64>,
    },
    /// Deposit funds into an existing job
    Deposit { job_id: u64 },
    /// Accept a job as freelancer
    Accept {
        job_id: u64,
        #[arg(long)]
        client: String,
    },
    /// Submit completed work
    Submit {
        job_id: u64,
        #[arg(long)]
        client: String,
    },
    /// Approve work and release payment
    Approve {
        job_id: u64,
        #[arg(long)]
        freelancer: String,
    },
    /// Reject work and open dispute
    Reject { job_id: u64, reason: String },
    /// Raise a dispute as freelancer
    RaiseDispute {
        job_id: u64,
        #[arg(long)]
        client: String,
        #[arg(long)]
        reason: String,
    },
    /// Resolve a dispute as arbiter
    ResolveDispute {
        job_id: u64,
        #[arg(long)]
        client: String,
        #[arg(long)]
        freelancer: String,
        #[arg(long)]
        freelancer_percent: u8,
    },
    /// Cancel a job (client only, before in-progress)
    Cancel { job_id: u64 },
    /// Show job details
    Show {
        job_id: u64,
        #[arg(long)]
        client: String,
    },
    /// Pause the program (admin only)
    Pause,
    /// Unpause the program (admin only)
    Unpause,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = kp_path(&cli.keypair);
    let payer = load_keypair(&path)?;
    let rpc = make_rpc(&cli.url);

    let msg = match cli.command {
        Commands::Init { treasury } => op_init(&rpc, &payer, &treasury)?,
        Commands::Create { title, amount, description, arbiter, job_id, deadline } => {
            op_create_job(&rpc, &payer, &title, &description, amount, &arbiter, job_id, deadline)?
        }
        Commands::Deposit { job_id } => op_deposit(&rpc, &payer, job_id)?,
        Commands::Accept { job_id, client } => op_accept(&rpc, &payer, job_id, &client)?,
        Commands::Submit { job_id, client } => op_submit(&rpc, &payer, job_id, &client)?,
        Commands::Approve { job_id, freelancer } => op_approve(&rpc, &payer, job_id, &freelancer)?,
        Commands::Reject { job_id, reason } => op_reject(&rpc, &payer, job_id, &reason)?,
        Commands::RaiseDispute { job_id, client, reason } => op_raise_dispute(&rpc, &payer, job_id, &client, &reason)?,
        Commands::ResolveDispute { job_id, client, freelancer, freelancer_percent } => {
            op_resolve_dispute(&rpc, &payer, job_id, &client, &freelancer, freelancer_percent)?
        }
        Commands::Cancel { job_id } => op_cancel(&rpc, &payer, job_id)?,
        Commands::Show { job_id, client } => {
            let info = op_show(&rpc, &client, job_id)?;
            format!("📋 Job Details (ID: {job_id})\n{info}")
        }
        Commands::Pause => op_pause(&rpc, &payer)?,
        Commands::Unpause => op_unpause(&rpc, &payer)?,
    };
    println!("{msg}");
    Ok(())
}

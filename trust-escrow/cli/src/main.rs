use anyhow::Result;
use clap::{Parser, Subcommand};
use escrow_core::*;
use tracing::info;

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
        #[arg(long, default_value = "")]
        notes: String,
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
        #[arg(long, default_value = "")]
        notes: String,
    },
    /// Show SOL balance for a pubkey (or the payer wallet)
    Balance {
        /// Pubkey to check (optional, defaults to payer)
        pubkey: Option<String>,
    },
    /// Request an airdrop (devnet/localhost only)
    Airdrop {
        /// Amount in SOL
        #[arg(default_value = "1.0")]
        amount: f64,
        /// Pubkey to airdrop to (optional, defaults to payer)
        #[arg(long)]
        pubkey: Option<String>,
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
    /// Withdraw funds from treasury (treasury wallet only)
    WithdrawTreasury {
        /// Amount in SOL to withdraw
        amount: f64,
        /// Destination pubkey (defaults to treasury wallet itself)
        #[arg(long)]
        destination: Option<String>,
    },
}

fn main() -> Result<()> {
    let _guard = init_logger();
    let cli = Cli::parse();
    let path = kp_path(&cli.keypair);
    let payer = load_keypair(&path)?;
    let rpc = make_rpc(&cli.url);

    info!(command = ?std::env::args().collect::<Vec<_>>(), "CLI ejecutado");

    let msg = match cli.command {
        Commands::Init { treasury } => op_init(&rpc, &payer, &treasury)?,
        Commands::Create {
            title,
            amount,
            description,
            arbiter,
            job_id,
            deadline,
        } => op_create_job(
            &rpc,
            &payer,
            &title,
            &description,
            amount,
            &arbiter,
            job_id,
            deadline,
        )?,
        Commands::Deposit { job_id } => op_deposit(&rpc, &payer, job_id)?,
        Commands::Accept { job_id, client } => op_accept(&rpc, &payer, job_id, &client)?,
        Commands::Submit { job_id, client, notes } => op_submit(&rpc, &payer, job_id, &client, &notes)?,
        Commands::Approve { job_id, freelancer } => op_approve(&rpc, &payer, job_id, &freelancer)?,
        Commands::Reject { job_id, reason } => op_reject(&rpc, &payer, job_id, &reason)?,
        Commands::RaiseDispute {
            job_id,
            client,
            reason,
        } => op_raise_dispute(&rpc, &payer, job_id, &client, &reason)?,
        Commands::ResolveDispute {
            job_id,
            client,
            freelancer,
            freelancer_percent,
            notes,
        } => op_resolve_dispute(
            &rpc,
            &payer,
            job_id,
            &client,
            &freelancer,
            freelancer_percent,
            &notes,
        )?,
        Commands::Cancel { job_id } => op_cancel(&rpc, &payer, job_id)?,
        Commands::Balance { pubkey } => {
            let pk = pubkey.unwrap_or_else(|| payer.pubkey().to_string());
            let lamports = op_get_balance(&rpc, &pk)?;
            format!("💰 Balance: {:.4} SOL ({lamports} lamports)\n   Pubkey: {pk}",
                lamports as f64 / 1e9)
        }
        Commands::Airdrop { amount, pubkey } => {
            let pk = pubkey.unwrap_or_else(|| payer.pubkey().to_string());
            op_airdrop(&rpc, &pk, amount)?
        }
        Commands::Show { job_id, client } => {
            let info = op_show(&rpc, &client, job_id)?;
            format!("📋 Job Details (ID: {job_id})\n{info}")
        }
        Commands::Pause => op_pause(&rpc, &payer)?,
        Commands::Unpause => op_unpause(&rpc, &payer)?,
        Commands::WithdrawTreasury {
            amount,
            destination,
        } => {
            let dest = destination.unwrap_or_else(|| payer.pubkey().to_string());
            op_withdraw_treasury(&rpc, &payer, amount, &dest)?
        }
    };
    println!("{msg}");
    Ok(())
}

/// Inicializa el logger: escribe en archivo + stderr.
/// El guard devuelto debe mantenerse vivo durante todo el programa.
fn init_logger() -> tracing_appender::non_blocking::WorkerGuard {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("trust-escrow-tui");
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "trust-escrow.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(file_writer).with_ansi(false))
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();

    guard
}

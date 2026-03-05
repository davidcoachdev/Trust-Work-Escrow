use solana_rpc_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::hash::hash;
use anchor_client::solana_sdk::instruction::{AccountMeta, Instruction};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{read_keypair_file, Signer};
#[allow(deprecated)] use anchor_client::solana_sdk::system_program;
use anchor_client::solana_sdk::transaction::Transaction;
use anyhow::{anyhow, Result};
use borsh::BorshSerialize;
use clap::{Parser, Subcommand};
use std::str::FromStr;

const PROGRAM_ID: &str = "5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo";

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

fn kp_path(p: &Option<String>) -> String {
    p.clone().unwrap_or_else(|| {
        let h = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{h}/.config/solana/id.json")
    })
}

fn config_pda(pid: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"config"], pid).0
}

fn job_pda(pid: &Pubkey, client: &Pubkey, job_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"job", client.as_ref(), &job_id.to_le_bytes()], pid).0
}

fn disc(name: &str) -> Vec<u8> {
    hash(format!("global:{name}").as_bytes()).to_bytes()[..8].to_vec()
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn send(rpc: &RpcClient, payer: &dyn Signer, ix: Instruction) -> Result<String> {
    let bh = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer], bh);
    let sig = rpc.send_and_confirm_transaction(&tx)?;
    Ok(sig.to_string())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = kp_path(&cli.keypair);
    let payer =
        read_keypair_file(&path).map_err(|e| anyhow!("Cannot read keypair {path}: {e}"))?;
    let rpc = RpcClient::new_with_commitment(cli.url.clone(), CommitmentConfig::confirmed());
    let pid = Pubkey::from_str(PROGRAM_ID)?;
    let cfg = config_pda(&pid);

    match cli.command {
        Commands::Init { treasury } => {
            let tpk = Pubkey::from_str(&treasury)?;
            let ix = Instruction::new_with_bytes(
                pid,
                &disc("initialize_config"),
                vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new_readonly(tpk, false),
                    AccountMeta::new(cfg, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
            );
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Config initialized!\n   Treasury: {tpk}\n   Tx: {sig}");
        }

        Commands::Create { title, amount, description, arbiter, job_id, deadline } => {
            let apk = Pubkey::from_str(&arbiter)?;
            let lam = (amount * 1e9) as u64;
            let dl = deadline.unwrap_or_else(|| now_ts() + 7 * 86400);
            let job = job_pda(&pid, &payer.pubkey(), job_id);

            #[derive(BorshSerialize)]
            struct A { job_id: u64, title: String, description: String, amount: u64, deadline: i64 }
            let mut data = disc("create_job");
            data.extend_from_slice(&borsh::to_vec(&A { job_id, title: title.clone(), description, amount: lam, deadline: dl })?);

            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(apk, false),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(cfg, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Job created!\n   Title: {title}\n   Amount: {amount} SOL ({lam} lamports)\n   Job PDA: {job}\n   Tx: {sig}");
        }

        Commands::Deposit { job_id } => {
            let job = job_pda(&pid, &payer.pubkey(), job_id);
            let mut data = disc("deposit_funds");
            data.extend_from_slice(&job_id.to_le_bytes());
            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(cfg, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Funds deposited!\n   Job ID: {job_id}\n   Tx: {sig}");
        }

        Commands::Accept { job_id, client: cs } => {
            let cpk = Pubkey::from_str(&cs)?;
            let job = job_pda(&pid, &cpk, job_id);
            let mut data = disc("accept_job");
            data.extend_from_slice(&job_id.to_le_bytes());
            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Job accepted!\n   Job ID: {job_id}\n   Tx: {sig}");
        }

        Commands::Submit { job_id, client: cs } => {
            let cpk = Pubkey::from_str(&cs)?;
            let job = job_pda(&pid, &cpk, job_id);
            let mut data = disc("submit_work");
            data.extend_from_slice(&job_id.to_le_bytes());
            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Work submitted!\n   Job ID: {job_id}\n   Tx: {sig}");
        }

        Commands::Approve { job_id, freelancer } => {
            let fpk = Pubkey::from_str(&freelancer)?;
            let job = job_pda(&pid, &payer.pubkey(), job_id);
            let cd = rpc.get_account_data(&cfg)?;
            let tpk = Pubkey::try_from(&cd[40..72]).map_err(|_| anyhow!("Bad treasury"))?;
            let mut data = disc("approve_work");
            data.extend_from_slice(&job_id.to_le_bytes());
            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new(fpk, false),
                AccountMeta::new(tpk, false),
                AccountMeta::new_readonly(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Work approved! Freelancer paid.\n   Job ID: {job_id}\n   Tx: {sig}");
        }

        Commands::Reject { job_id, reason } => {
            let job = job_pda(&pid, &payer.pubkey(), job_id);
            #[derive(BorshSerialize)]
            struct A { job_id: u64, reason: String }
            let mut data = disc("reject_work");
            data.extend_from_slice(&borsh::to_vec(&A { job_id, reason: reason.clone() })?);
            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("⚠️  Work rejected — dispute opened.\n   Reason: {reason}\n   Tx: {sig}");
        }

        Commands::Cancel { job_id } => {
            let job = job_pda(&pid, &payer.pubkey(), job_id);
            let mut data = disc("cancel_job");
            data.extend_from_slice(&job_id.to_le_bytes());
            let ix = Instruction::new_with_bytes(pid, &data, vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("✅ Job cancelled.\n   Job ID: {job_id}\n   Tx: {sig}");
        }

        Commands::Show { job_id, client: cs } => {
            let cpk = Pubkey::from_str(&cs)?;
            let ja = job_pda(&pid, &cpk, job_id);
            let raw = rpc.get_account_data(&ja)?;
            let d = &raw[8..];
            let mut o: usize = 0;

            let client = Pubkey::try_from(&d[o..o+32]).unwrap(); o += 32;
            let hf = d[o]; o += 1;
            let freelancer = if hf != 0 { let pk = Pubkey::try_from(&d[o..o+32]).unwrap(); o += 32; Some(pk) } else { None };
            let arbiter = Pubkey::try_from(&d[o..o+32]).unwrap(); o += 32;
            let amount = u64::from_le_bytes(d[o..o+8].try_into().unwrap()); o += 8;
            let fp = d[o]; o += 1;
            let fa = u64::from_le_bytes(d[o..o+8].try_into().unwrap()); o += 8;
            let sn = ["Created","Funded","InProgress","Submitted","Released","Disputed","Resolved","Cancelled"];
            let st = sn.get(d[o] as usize).unwrap_or(&"Unknown"); o += 1;
            let rs = |d: &[u8], o: &mut usize| -> String {
                let l = u32::from_le_bytes(d[*o..*o+4].try_into().unwrap()) as usize; *o += 4;
                let s = String::from_utf8_lossy(&d[*o..*o+l]).to_string(); *o += l; s
            };
            let title = rs(d, &mut o);
            let desc = rs(d, &mut o);
            let dl = i64::from_le_bytes(d[o..o+8].try_into().unwrap()); o += 8;
            let ca = i64::from_le_bytes(d[o..o+8].try_into().unwrap()); o += 8;
            let ua = i64::from_le_bytes(d[o..o+8].try_into().unwrap()); o += 8;
            let dr = rs(d, &mut o);

            println!("📋 Job Details (ID: {job_id})");
            println!("   PDA:         {ja}");
            println!("   Client:      {client}");
            println!("   Arbiter:     {arbiter}");
            println!("   Freelancer:  {}", freelancer.map(|f| f.to_string()).unwrap_or("Not assigned".into()));
            println!("   Title:       {title}");
            println!("   Description: {desc}");
            println!("   Amount:      {} SOL ({amount} lamports)", amount as f64 / 1e9);
            println!("   Fee:         {fa} lamports ({fp}%)");
            println!("   Status:      {st}");
            println!("   Deadline:    {dl}");
            println!("   Created:     {ca}");
            println!("   Updated:     {ua}");
            if !dr.is_empty() { println!("   Dispute:     {dr}"); }
        }

        Commands::Pause => {
            let ix = Instruction::new_with_bytes(pid, &disc("pause_program"), vec![
                AccountMeta::new(payer.pubkey(), true), AccountMeta::new(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("⏸️  Program paused. Tx: {sig}");
        }

        Commands::Unpause => {
            let ix = Instruction::new_with_bytes(pid, &disc("unpause_program"), vec![
                AccountMeta::new(payer.pubkey(), true), AccountMeta::new(cfg, false),
            ]);
            let sig = send(&rpc, &payer, ix)?;
            println!("▶️  Program unpaused. Tx: {sig}");
        }
    }
    Ok(())
}

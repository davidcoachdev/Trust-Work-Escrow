//! # escrow-core
//!
//! Shared Solana logic for Trust Work Escrow — used by CLI and TUI.

use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::hash::hash;
use anchor_client::solana_sdk::instruction::{AccountMeta, Instruction};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{read_keypair_file, Keypair};
pub use anchor_client::solana_sdk::signature::Signer;
#[allow(deprecated)]
use anchor_client::solana_sdk::system_program;
use anchor_client::solana_sdk::transaction::Transaction;
use anyhow::{anyhow, Result};
use borsh::BorshSerialize;
use solana_rpc_client::rpc_client::RpcClient;
use std::str::FromStr;

pub const PROGRAM_ID: &str = "5gu5JCSpB8MKyJzhXpGaCt8SruAMnRD6cTPbwPX6JTYo";

/// Returns default keypair path or the one provided.
pub fn kp_path(p: &Option<String>) -> String {
    p.clone().unwrap_or_else(|| {
        let h = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        format!("{h}/.config/solana/id.json")
    })
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

pub fn load_keypair(path: &str) -> Result<Keypair> {
    read_keypair_file(path).map_err(|e| anyhow!("Cannot read keypair {path}: {e}"))
}

pub fn make_rpc(url: &str) -> RpcClient {
    RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed())
}

pub fn program_id() -> Result<Pubkey> {
    Pubkey::from_str(PROGRAM_ID).map_err(|e| anyhow!("Bad program ID: {e}"))
}

pub fn config_pda(pid: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"config"], pid).0
}

pub fn job_pda(pid: &Pubkey, client: &Pubkey, job_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"job", client.as_ref(), &job_id.to_le_bytes()], pid).0
}

fn disc(name: &str) -> Vec<u8> {
    hash(format!("global:{name}").as_bytes()).to_bytes()[..8].to_vec()
}

pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn send(rpc: &RpcClient, payer: &Keypair, ix: Instruction) -> Result<String> {
    let bh = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer], bh);
    let sig = rpc.send_and_confirm_transaction(&tx)?;
    Ok(sig.to_string())
}

fn send_many(rpc: &RpcClient, payer: &Keypair, ixs: Vec<Instruction>) -> Result<String> {
    let bh = rpc.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&ixs, Some(&payer.pubkey()), &[payer], bh);
    let sig = rpc.send_and_confirm_transaction(&tx)?;
    Ok(sig.to_string())
}

fn build_cancel_ix(pid: &Pubkey, cfg: &Pubkey, client: &Pubkey, job_id: u64) -> Instruction {
    let job = job_pda(pid, client, job_id);
    let mut data = disc("cancel_job");
    data.extend_from_slice(&job_id.to_le_bytes());
    Instruction::new_with_bytes(
        *pid,
        &data,
        vec![
            AccountMeta::new(*client, true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(*cfg, false),
        ],
    )
}

fn build_create_ix(
    pid: &Pubkey,
    cfg: &Pubkey,
    client: &Pubkey,
    arbiter: &Pubkey,
    job_id: u64,
    title: &str,
    description: &str,
    amount: u64,
    deadline: i64,
) -> Result<Instruction> {
    let job = job_pda(pid, client, job_id);
    #[derive(BorshSerialize)]
    struct Args {
        job_id: u64,
        title: String,
        description: String,
        amount: u64,
        deadline: i64,
    }
    let mut data = disc("create_job");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        title: title.to_string(),
        description: description.to_string(),
        amount,
        deadline,
    })?);
    Ok(Instruction::new_with_bytes(
        *pid,
        &data,
        vec![
            AccountMeta::new(*client, true),
            AccountMeta::new_readonly(*arbiter, false),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(*cfg, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    ))
}

fn build_deposit_ix(pid: &Pubkey, cfg: &Pubkey, client: &Pubkey, job_id: u64) -> Instruction {
    let job = job_pda(pid, client, job_id);
    let mut data = disc("deposit_funds");
    data.extend_from_slice(&job_id.to_le_bytes());
    Instruction::new_with_bytes(
        *pid,
        &data,
        vec![
            AccountMeta::new(*client, true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(*cfg, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    )
}

// ─── Job Info (parsed from on-chain data) ────────────────────────────────────

pub struct JobInfo {
    pub pda: String,
    pub client: String,
    pub freelancer: Option<String>,
    pub arbiter: String,
    pub amount: u64,
    pub fee_percent: u8,
    pub fee_amount: u64,
    pub status: String,
    pub title: String,
    pub description: String,
    pub deadline: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub dispute_reason: String,
}

impl std::fmt::Display for JobInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PDA:         {}", self.pda)?;
        writeln!(f, "Client:      {}", self.client)?;
        writeln!(f, "Arbiter:     {}", self.arbiter)?;
        writeln!(
            f,
            "Freelancer:  {}",
            self.freelancer.as_deref().unwrap_or("Not assigned")
        )?;
        writeln!(f, "Title:       {}", self.title)?;
        writeln!(f, "Description: {}", self.description)?;
        writeln!(
            f,
            "Amount:      {} SOL ({} lamports)",
            self.amount as f64 / 1e9,
            self.amount
        )?;
        writeln!(
            f,
            "Fee:         {} lamports ({}%)",
            self.fee_amount, self.fee_percent
        )?;
        writeln!(
            f,
            "Neto freel.: {} SOL ({} lamports)",
            (self.amount.saturating_sub(self.fee_amount)) as f64 / 1e9,
            self.amount.saturating_sub(self.fee_amount)
        )?;
        writeln!(f, "Status:      {}", self.status)?;
        writeln!(f, "Deadline:    {}", self.deadline)?;
        writeln!(f, "Created:     {}", self.created_at)?;
        writeln!(f, "Updated:     {}", self.updated_at)?;
        if !self.dispute_reason.is_empty() {
            writeln!(f, "Dispute:     {}", self.dispute_reason)?;
        }
        Ok(())
    }
}

// ─── Operations ──────────────────────────────────────────────────────────────

pub fn op_init(rpc: &RpcClient, payer: &Keypair, treasury: &str) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let tpk = Pubkey::from_str(treasury).map_err(|_| anyhow!("Invalid treasury address"))?;
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
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Config initialized!\n   Treasury: {tpk}\n   Tx: {sig}"
    ))
}

pub fn op_create_job(
    rpc: &RpcClient,
    payer: &Keypair,
    title: &str,
    description: &str,
    amount_sol: f64,
    arbiter: &str,
    job_id: u64,
    deadline: Option<i64>,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let apk = Pubkey::from_str(arbiter).map_err(|_| anyhow!("Invalid arbiter address"))?;
    let lam = (amount_sol * 1e9) as u64;
    let dl = deadline.unwrap_or_else(|| now_ts() + 7 * 86400);
    let job = job_pda(&pid, &payer.pubkey(), job_id);

    #[derive(BorshSerialize)]
    struct Args {
        job_id: u64,
        title: String,
        description: String,
        amount: u64,
        deadline: i64,
    }
    let mut data = disc("create_job");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        title: title.to_string(),
        description: description.to_string(),
        amount: lam,
        deadline: dl,
    })?);

    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(apk, false),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Job created!\n   Title: {title}\n   Amount: {amount_sol} SOL ({lam} lamports)\n   Job PDA: {job}\n   Tx: {sig}"
    ))
}

pub fn op_deposit(rpc: &RpcClient, payer: &Keypair, job_id: u64) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let job = job_pda(&pid, &payer.pubkey(), job_id);
    let mut data = disc("deposit_funds");
    data.extend_from_slice(&job_id.to_le_bytes());
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Funds deposited!\n   Job ID: {job_id}\n   Tx: {sig}"
    ))
}

pub fn op_accept(rpc: &RpcClient, payer: &Keypair, job_id: u64, client: &str) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let cpk = Pubkey::from_str(client).map_err(|_| anyhow!("Invalid client address"))?;
    let job = job_pda(&pid, &cpk, job_id);
    let mut data = disc("accept_job");
    data.extend_from_slice(&job_id.to_le_bytes());
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Job accepted!\n   Job ID: {job_id}\n   Tx: {sig}"
    ))
}

pub fn op_submit(rpc: &RpcClient, payer: &Keypair, job_id: u64, client: &str) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let cpk = Pubkey::from_str(client).map_err(|_| anyhow!("Invalid client address"))?;
    let job = job_pda(&pid, &cpk, job_id);
    let mut data = disc("submit_work");
    data.extend_from_slice(&job_id.to_le_bytes());
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Work submitted!\n   Job ID: {job_id}\n   Tx: {sig}"
    ))
}

pub fn op_approve(
    rpc: &RpcClient,
    payer: &Keypair,
    job_id: u64,
    freelancer: &str,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let fpk = Pubkey::from_str(freelancer).map_err(|_| anyhow!("Invalid freelancer address"))?;
    let job = job_pda(&pid, &payer.pubkey(), job_id);
    let cd = rpc.get_account_data(&cfg)?;
    let tpk = Pubkey::try_from(&cd[40..72]).map_err(|_| anyhow!("Bad treasury in config"))?;
    let mut data = disc("approve_work");
    data.extend_from_slice(&job_id.to_le_bytes());
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new(fpk, false),
            AccountMeta::new(tpk, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Work approved! Freelancer paid.\n   Job ID: {job_id}\n   Tx: {sig}"
    ))
}

pub fn op_reject(
    rpc: &RpcClient,
    payer: &Keypair,
    job_id: u64,
    reason: &str,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let job = job_pda(&pid, &payer.pubkey(), job_id);

    #[derive(BorshSerialize)]
    struct Args {
        job_id: u64,
        reason: String,
    }
    let mut data = disc("reject_work");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        reason: reason.to_string(),
    })?);
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "⚠️  Work rejected — dispute opened.\n   Reason: {reason}\n   Tx: {sig}"
    ))
}

pub fn op_raise_dispute(
    rpc: &RpcClient,
    payer: &Keypair,
    job_id: u64,
    client: &str,
    reason: &str,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let cpk = Pubkey::from_str(client).map_err(|_| anyhow!("Invalid client address"))?;
    let job = job_pda(&pid, &cpk, job_id);

    #[derive(BorshSerialize)]
    struct Args {
        job_id: u64,
        reason: String,
    }
    let mut data = disc("raise_dispute");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        reason: reason.to_string(),
    })?);
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "⚠️  Dispute raised by freelancer.\n   Reason: {reason}\n   Tx: {sig}"
    ))
}

pub fn op_resolve_dispute(
    rpc: &RpcClient,
    payer: &Keypair,
    job_id: u64,
    client: &str,
    freelancer: &str,
    freelancer_percent: u8,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let cpk = Pubkey::from_str(client).map_err(|_| anyhow!("Invalid client address"))?;
    let fpk = Pubkey::from_str(freelancer).map_err(|_| anyhow!("Invalid freelancer address"))?;
    let job = job_pda(&pid, &cpk, job_id);
    let cd = rpc.get_account_data(&cfg)?;
    let tpk = Pubkey::try_from(&cd[40..72]).map_err(|_| anyhow!("Bad treasury in config"))?;

    #[derive(BorshSerialize)]
    struct Args {
        job_id: u64,
        freelancer_percent: u8,
    }
    let mut data = disc("resolve_dispute");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        freelancer_percent,
    })?);
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(cpk, false),
            AccountMeta::new(job, false),
            AccountMeta::new(fpk, false),
            AccountMeta::new(tpk, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "⚖️  Dispute resolved!\n   Freelancer gets: {freelancer_percent}%\n   Tx: {sig}"
    ))
}

pub fn op_cancel(rpc: &RpcClient, payer: &Keypair, job_id: u64) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let job = job_pda(&pid, &payer.pubkey(), job_id);
    let mut data = disc("cancel_job");
    data.extend_from_slice(&job_id.to_le_bytes());
    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "✅ Job cancelled.\n   Job ID: {job_id}\n   Tx: {sig}"
    ))
}

pub fn op_show(rpc: &RpcClient, client: &str, job_id: u64) -> Result<JobInfo> {
    let pid = program_id()?;
    let cpk = Pubkey::from_str(client).map_err(|_| anyhow!("Invalid client address"))?;
    let ja = job_pda(&pid, &cpk, job_id);
    let raw = rpc.get_account_data(&ja)?;
    let d = &raw[8..];
    let mut o: usize = 0;

    let client_pk = Pubkey::try_from(&d[o..o + 32]).map_err(|_| anyhow!("Bad client"))?;
    o += 32;
    let hf = d[o];
    o += 1;
    let freelancer = if hf != 0 {
        let pk = Pubkey::try_from(&d[o..o + 32]).map_err(|_| anyhow!("Bad freelancer"))?;
        o += 32;
        Some(pk.to_string())
    } else {
        None
    };
    let arbiter = Pubkey::try_from(&d[o..o + 32]).map_err(|_| anyhow!("Bad arbiter"))?;
    o += 32;
    let amount = u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    o += 8;
    let fp = d[o];
    o += 1;
    let fa = u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    o += 8;
    let status_names = [
        "Created",
        "Funded",
        "InProgress",
        "Submitted",
        "Released",
        "Disputed",
        "Resolved",
        "Cancelled",
    ];
    let st = status_names
        .get(d[o] as usize)
        .unwrap_or(&"Unknown")
        .to_string();
    o += 1;

    let read_str = |d: &[u8], o: &mut usize| -> String {
        let l = u32::from_le_bytes(d[*o..*o + 4].try_into().unwrap()) as usize;
        *o += 4;
        let s = String::from_utf8_lossy(&d[*o..*o + l]).to_string();
        *o += l;
        s
    };
    let title = read_str(d, &mut o);
    let desc = read_str(d, &mut o);
    let dl = i64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    o += 8;
    let ca = i64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    o += 8;
    let ua = i64::from_le_bytes(d[o..o + 8].try_into().unwrap());
    o += 8;
    let dr = read_str(d, &mut o);

    Ok(JobInfo {
        pda: ja.to_string(),
        client: client_pk.to_string(),
        freelancer,
        arbiter: arbiter.to_string(),
        amount,
        fee_percent: fp,
        fee_amount: fa,
        status: st,
        title,
        description: desc,
        deadline: dl,
        created_at: ca,
        updated_at: ua,
        dispute_reason: dr,
    })
}

pub fn op_update_job(
    rpc: &RpcClient,
    payer: &Keypair,
    old_job_id: u64,
    title: &str,
    description: &str,
    amount_sol: f64,
    arbiter: &str,
    deadline_days: Option<i64>,
    was_funded: bool,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let apk = Pubkey::from_str(arbiter).map_err(|_| anyhow!("Invalid arbiter address"))?;
    let lam = (amount_sol * 1e9) as u64;
    let new_job_id = now_ts() as u64;
    let dl = deadline_days
        .map(|d| now_ts() + d * 86400)
        .unwrap_or_else(|| now_ts() + 7 * 86400);

    let mut ixs = Vec::new();
    // 1. Cancel old job (refunds lamports if Funded)
    ixs.push(build_cancel_ix(&pid, &cfg, &payer.pubkey(), old_job_id));
    // 2. Create new job with same arbiter and new parameters
    ixs.push(build_create_ix(
        &pid, &cfg, &payer.pubkey(), &apk, new_job_id, title, description, lam, dl,
    )?);
    // 3. Deposit funds if the old job was already funded
    if was_funded {
        ixs.push(build_deposit_ix(&pid, &cfg, &payer.pubkey(), new_job_id));
    }

    let sig = send_many(rpc, payer, ixs)?;
    let status = if was_funded { "Funded" } else { "Created" };
    Ok(format!(
        "✅ Job actualizado!\n   Nuevo Job ID: {new_job_id}\n   Título: {title}\n   Monto: {amount_sol} SOL\n   Estado: {status}\n   Tx: {sig}"
    ))
}

pub fn op_pause(rpc: &RpcClient, payer: &Keypair) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let ix = Instruction::new_with_bytes(
        pid,
        &disc("pause_program"),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!("⏸️  Program paused.\n   Tx: {sig}"))
}

pub fn op_unpause(rpc: &RpcClient, payer: &Keypair) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let ix = Instruction::new_with_bytes(
        pid,
        &disc("unpause_program"),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(cfg, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!("▶️  Program unpaused.\n   Tx: {sig}"))
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_id_valid() {
        let pid = program_id().unwrap();
        assert_eq!(pid.to_string(), PROGRAM_ID);
    }

    #[test]
    fn test_config_pda_deterministic() {
        let pid = program_id().unwrap();
        let pda1 = config_pda(&pid);
        let pda2 = config_pda(&pid);
        assert_eq!(pda1, pda2);
    }

    #[test]
    fn test_job_pda_deterministic() {
        let pid = program_id().unwrap();
        let client = Pubkey::new_unique();
        let pda1 = job_pda(&pid, &client, 1);
        let pda2 = job_pda(&pid, &client, 1);
        assert_eq!(pda1, pda2);
    }

    #[test]
    fn test_job_pda_different_ids() {
        let pid = program_id().unwrap();
        let client = Pubkey::new_unique();
        let pda1 = job_pda(&pid, &client, 1);
        let pda2 = job_pda(&pid, &client, 2);
        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_job_pda_different_clients() {
        let pid = program_id().unwrap();
        let c1 = Pubkey::new_unique();
        let c2 = Pubkey::new_unique();
        let pda1 = job_pda(&pid, &c1, 1);
        let pda2 = job_pda(&pid, &c2, 1);
        assert_ne!(pda1, pda2);
    }

    #[test]
    fn test_disc_deterministic() {
        let d1 = disc("create_job");
        let d2 = disc("create_job");
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 8);
    }

    #[test]
    fn test_disc_different_names() {
        let d1 = disc("create_job");
        let d2 = disc("accept_job");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_now_ts_positive() {
        let ts = now_ts();
        assert!(ts > 0);
    }

    #[test]
    fn test_kp_path_default() {
        let path = kp_path(&None);
        assert!(path.ends_with(".config/solana/id.json"));
    }

    #[test]
    fn test_kp_path_custom() {
        let path = kp_path(&Some("/my/key.json".into()));
        assert_eq!(path, "/my/key.json");
    }

    #[test]
    fn test_make_rpc() {
        let rpc = make_rpc("http://localhost:8899");
        // Just check it doesn't panic
        let _ = rpc;
    }

    #[test]
    fn test_job_info_display() {
        let info = JobInfo {
            pda: "PDA123".into(),
            client: "Client123".into(),
            freelancer: Some("Free123".into()),
            arbiter: "Arb123".into(),
            amount: 2_000_000_000,
            fee_percent: 5,
            fee_amount: 100_000_000,
            status: "Created".into(),
            title: "Test Job".into(),
            description: "Test desc".into(),
            deadline: 1700000000,
            created_at: 1699000000,
            updated_at: 1699000000,
            dispute_reason: String::new(),
        };
        let s = info.to_string();
        assert!(s.contains("Test Job"));
        assert!(s.contains("2 SOL"));
        assert!(s.contains("Client123"));
        assert!(s.contains("Free123"));
        assert!(!s.contains("Dispute"));
    }

    #[test]
    fn test_job_info_display_with_dispute() {
        let info = JobInfo {
            pda: "PDA".into(),
            client: "C".into(),
            freelancer: None,
            arbiter: "A".into(),
            amount: 1_000_000_000,
            fee_percent: 5,
            fee_amount: 50_000_000,
            status: "Disputed".into(),
            title: "Job".into(),
            description: "".into(),
            deadline: 0,
            created_at: 0,
            updated_at: 0,
            dispute_reason: "Bad work".into(),
        };
        let s = info.to_string();
        assert!(s.contains("Not assigned"));
        assert!(s.contains("Bad work"));
    }

    #[test]
    fn test_load_keypair_invalid_path() {
        let result = load_keypair("/nonexistent/path.json");
        assert!(result.is_err());
    }
}

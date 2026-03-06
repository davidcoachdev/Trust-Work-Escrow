//! # escrow-core
//!
//! Shared Solana logic for Trust Work Escrow — used by CLI and TUI.

use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::hash::hash;
use anchor_client::solana_sdk::instruction::{AccountMeta, Instruction};
use anchor_client::solana_sdk::pubkey::Pubkey;
pub use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::signature::{read_keypair_file, Keypair};
#[allow(deprecated)]
use anchor_client::solana_sdk::system_program;
use anchor_client::solana_sdk::transaction::Transaction;
use anyhow::{anyhow, Result};
use borsh::BorshSerialize;
use solana_account_decoder_client_types::UiAccountEncoding;
use solana_rpc_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_rpc_client_api::config::RpcTransactionConfig;
use solana_rpc_client_api::config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_rpc_client_api::filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_transaction_status::{EncodedTransaction, UiMessage, UiTransactionEncoding};
use std::str::FromStr;
use tracing::{debug, error, info};

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

#[derive(Clone)]
pub struct JobInfo {
    pub pda: String,
    pub job_id: u64,
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
    pub delivery_notes: String,
    pub resolution_notes: String,
}

/// Formatea un timestamp Unix en "DD/MM/YYYY HH:MM UTC"
fn fmt_date(ts: i64) -> String {
    if ts == 0 {
        return "—".into();
    }
    // Cálculo manual sin dependencias externas
    let secs = ts as u64;
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days_since_epoch = secs / 86400;

    // Algoritmo de civil_from_days (Howard Hinnant)
    let z = days_since_epoch as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{:02}/{:02}/{} {:02}:{:02}:{:02} UTC", d, mo, y, h, m, s)
}

/// Formatea la fecha límite: fecha + días restantes (o "VENCIDO" si ya pasó)
fn fmt_deadline(ts: i64) -> String {
    if ts == 0 {
        return "Sin deadline".into();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let date_str = fmt_date(ts);
    let diff_secs = ts - now;
    if diff_secs <= 0 {
        let days_ago = (-diff_secs) / 86400;
        format!("{date_str}  ⚠️  VENCIDO hace {days_ago} día(s)")
    } else {
        let days_left = diff_secs / 86400;
        let hours_left = (diff_secs % 86400) / 3600;
        if days_left == 0 {
            format!("{date_str}  ⏰  ¡Vence hoy! ({hours_left}h restantes)")
        } else {
            format!("{date_str}  ⏳  {days_left} día(s) restante(s)")
        }
    }
}

impl std::fmt::Display for JobInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PDA:         {}", self.pda)?;
        if self.job_id != 0 {
            writeln!(f, "Job ID:      {}", self.job_id)?;
        }
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
        writeln!(f, "Deadline:    {}", fmt_deadline(self.deadline))?;
        writeln!(f, "Created:     {}", fmt_date(self.created_at))?;
        writeln!(f, "Updated:     {}", fmt_date(self.updated_at))?;
        if !self.dispute_reason.is_empty() {
            let label = match self.status.as_str() {
                "InProgress" => "Rechazo:",
                _ => "Disputa:",
            };
            writeln!(f, "{:<13}{}", label, self.dispute_reason)?;
        }
        if !self.delivery_notes.is_empty() {
            writeln!(f, "Entrega:     {}", self.delivery_notes)?;
        }
        if !self.resolution_notes.is_empty() {
            writeln!(f, "Resolución:  {}", self.resolution_notes)?;
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
        "✅ Job created!\n   Title: {title}\n   Job ID: {job_id}\n   Amount: {amount_sol} SOL ({lam} lamports)\n   Deadline: {dl}\n   Job PDA: {job}\n   Tx: {sig}"
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

pub fn op_submit(
    rpc: &RpcClient,
    payer: &Keypair,
    job_id: u64,
    client: &str,
    notes: &str,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let cpk = Pubkey::from_str(client).map_err(|_| anyhow!("Invalid client address"))?;
    let job = job_pda(&pid, &cpk, job_id);

    #[derive(BorshSerialize)]
    struct Args {
        job_id: u64,
        notes: String,
    }
    let mut data = disc("submit_work");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        notes: notes.to_string(),
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

pub fn op_reject(rpc: &RpcClient, payer: &Keypair, job_id: u64, reason: &str) -> Result<String> {
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
    notes: &str,
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
        notes: String,
    }
    let mut data = disc("resolve_dispute");
    data.extend_from_slice(&borsh::to_vec(&Args {
        job_id,
        freelancer_percent,
        notes: notes.to_string(),
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
    let delivery_notes = read_str(d, &mut o);
    let resolution_notes = read_str(d, &mut o);

    Ok(JobInfo {
        pda: ja.to_string(),
        job_id,
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
        delivery_notes,
        resolution_notes,
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
        &pid,
        &cfg,
        &payer.pubkey(),
        &apk,
        new_job_id,
        title,
        description,
        lam,
        dl,
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

/// Retira `amount` lamports de la cuenta treasury hacia `destination`.
/// El `payer` keypair debe ser la wallet treasury registrada en el config.
pub fn op_withdraw_treasury(
    rpc: &RpcClient,
    payer: &Keypair,
    amount_sol: f64,
    destination: &str,
) -> Result<String> {
    let pid = program_id()?;
    let cfg = config_pda(&pid);
    let dest_pk =
        Pubkey::from_str(destination).map_err(|_| anyhow!("Invalid destination address"))?;
    let amount = (amount_sol * 1e9) as u64;

    #[derive(BorshSerialize)]
    struct Args {
        amount: u64,
    }
    let mut data = disc("withdraw_treasury");
    data.extend_from_slice(&borsh::to_vec(&Args { amount })?);

    let ix = Instruction::new_with_bytes(
        pid,
        &data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(dest_pk, false),
            AccountMeta::new_readonly(cfg, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
    );
    let sig = send(rpc, payer, ix)?;
    Ok(format!(
        "💰 Treasury withdrawal!\n   Amount: {amount_sol} SOL ({amount} lamports)\n   To: {dest_pk}\n   Tx: {sig}"
    ))
}

/// Intenta recuperar el job_id a partir del PDA conocido y una ventana de tiempo
/// alrededor de `created_at`. Funciona porque job_id = now_ts() en el cliente,
/// y created_at = clock.unix_timestamp en cadena (difieren muy poco).
fn recover_job_id(pid: &Pubkey, client: &Pubkey, pda_str: &str, created_at: i64) -> u64 {
    let Ok(target) = Pubkey::from_str(pda_str) else {
        return 0;
    };
    // Ventana de ±30 segundos alrededor de created_at para cubrir latencia de red
    for delta in 0i64..=60 {
        for &sign in &[-1i64, 1i64] {
            let candidate = (created_at + sign * delta) as u64;
            let (computed, _) = Pubkey::find_program_address(
                &[b"job", client.as_ref(), &candidate.to_le_bytes()],
                pid,
            );
            if computed == target {
                return candidate;
            }
        }
    }
    // fallback: usar created_at directamente (debería funcionar en la mayoría de los casos)
    created_at as u64
}

/// Parsea los datos crudos de las cuentas Job y devuelve Vec<JobInfo> ordenado por created_at desc.
fn parse_raw_job_accounts(
    pid: &Pubkey,
    accounts: Vec<(Pubkey, anchor_client::solana_sdk::account::Account)>,
) -> Vec<JobInfo> {
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

    let read_str = |d: &[u8], o: &mut usize| -> Option<String> {
        if d.len() < *o + 4 {
            return None;
        }
        let l = u32::from_le_bytes(d[*o..*o + 4].try_into().unwrap()) as usize;
        *o += 4;
        if d.len() < *o + l {
            return None;
        }
        let s = String::from_utf8_lossy(&d[*o..*o + l]).to_string();
        *o += l;
        Some(s)
    };

    let mut jobs = Vec::new();
    for (pda, account) in accounts {
        let raw = &account.data;
        if raw.len() < 9 {
            continue;
        }
        let d = &raw[8..]; // saltar discriminator
        let mut o: usize = 0;

        let Ok(client_pk) = Pubkey::try_from(&d[o..o + 32]) else {
            continue;
        };
        o += 32;
        let hf = d[o];
        o += 1;
        // Option<Pubkey> en Borsh: None = [0] (1 byte), Some(pk) = [1, pk...] (33 bytes)
        let freelancer = if hf != 0 {
            let Ok(pk) = Pubkey::try_from(&d[o..o + 32]) else {
                continue;
            };
            o += 32;
            Some(pk.to_string())
        } else {
            None // hf=0 consume solo 1 byte (ya avanzado arriba)
        };
        if d.len() < o + 32 {
            continue;
        }
        let Ok(arbiter) = Pubkey::try_from(&d[o..o + 32]) else {
            continue;
        };
        o += 32;
        if d.len() < o + 8 {
            continue;
        }
        let amount = u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
        o += 8;
        let fp = d[o];
        o += 1;
        if d.len() < o + 8 {
            continue;
        }
        let fa = u64::from_le_bytes(d[o..o + 8].try_into().unwrap());
        o += 8;
        let st = status_names
            .get(d[o] as usize)
            .unwrap_or(&"Unknown")
            .to_string();
        o += 1;

        let Some(title) = read_str(d, &mut o) else {
            continue;
        };
        let Some(desc) = read_str(d, &mut o) else {
            continue;
        };
        if d.len() < o + 24 {
            continue;
        }
        let dl = i64::from_le_bytes(d[o..o + 8].try_into().unwrap());
        o += 8;
        let ca = i64::from_le_bytes(d[o..o + 8].try_into().unwrap());
        o += 8;
        let ua = i64::from_le_bytes(d[o..o + 8].try_into().unwrap());
        o += 8;
        let dr = read_str(d, &mut o).unwrap_or_default();
        let delivery_notes = read_str(d, &mut o).unwrap_or_default();
        let resolution_notes = read_str(d, &mut o).unwrap_or_default();

        let pda_str = pda.to_string();
        let job_id = recover_job_id(pid, &client_pk, &pda_str, ca);

        jobs.push(JobInfo {
            pda: pda_str,
            job_id,
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
            delivery_notes,
            resolution_notes,
        });
    }

    // Ordenar por created_at descendente (más reciente primero)
    jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    jobs
}

fn make_job_rpc_config(filters: Vec<RpcFilterType>) -> RpcProgramAccountsConfig {
    RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            commitment: Some(CommitmentConfig::confirmed()),
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        },
        with_context: None,
        sort_results: None,
    }
}

fn job_discriminator_filter() -> RpcFilterType {
    let job_disc = &hash("account:Job".as_bytes()).to_bytes()[..8];
    RpcFilterType::Memcmp(Memcmp::new(0, MemcmpEncodedBytes::Bytes(job_disc.to_vec())))
}

/// Lista todos los jobs creados por `client_pubkey` usando `getProgramAccounts`
/// con un filtro memcmp en el campo `client` del struct Job (offset 8).
pub fn op_list_jobs(rpc: &RpcClient, client_pubkey: &str) -> Result<Vec<JobInfo>> {
    let pid = program_id()?;
    let cpk = Pubkey::from_str(client_pubkey).map_err(|_| anyhow!("Invalid client address"))?;

    let filters = vec![
        job_discriminator_filter(),
        // client (Pubkey) en offset 8 (32 bytes)
        RpcFilterType::Memcmp(Memcmp::new(
            8,
            MemcmpEncodedBytes::Bytes(cpk.to_bytes().to_vec()),
        )),
    ];

    debug!(client = %client_pubkey, "Buscando jobs del cliente");
    let accounts = rpc
        .get_program_accounts_with_config(&pid, make_job_rpc_config(filters))
        .map_err(|e| {
            error!(client = %client_pubkey, error = %e, "Error en getProgramAccounts");
            e
        })?;
    debug!(found = accounts.len(), "Cuentas encontradas en cadena");

    let jobs = parse_raw_job_accounts(&pid, accounts);
    info!(client = %client_pubkey, jobs = jobs.len(), "Jobs cargados correctamente");
    Ok(jobs)
}

/// Lista los jobs donde `freelancer_pubkey` es el freelancer asignado.
/// Filtro memcmp: offset 40 = [1] (has_freelancer = true), offset 41 = pubkey bytes.
pub fn op_list_jobs_as_freelancer(
    rpc: &RpcClient,
    freelancer_pubkey: &str,
) -> Result<Vec<JobInfo>> {
    let pid = program_id()?;
    let fpk =
        Pubkey::from_str(freelancer_pubkey).map_err(|_| anyhow!("Invalid freelancer address"))?;

    let filters = vec![
        job_discriminator_filter(),
        // has_freelancer = 1 en offset 8+32=40
        RpcFilterType::Memcmp(Memcmp::new(40, MemcmpEncodedBytes::Bytes(vec![1u8]))),
        // freelancer pubkey en offset 8+32+1=41 (32 bytes)
        RpcFilterType::Memcmp(Memcmp::new(
            41,
            MemcmpEncodedBytes::Bytes(fpk.to_bytes().to_vec()),
        )),
    ];

    debug!(freelancer = %freelancer_pubkey, "Buscando jobs del freelancer");
    let accounts = rpc
        .get_program_accounts_with_config(&pid, make_job_rpc_config(filters))
        .map_err(|e| {
            error!(freelancer = %freelancer_pubkey, error = %e, "Error en getProgramAccounts (freelancer)");
            e
        })?;
    debug!(found = accounts.len(), "Cuentas encontradas (freelancer)");

    let jobs = parse_raw_job_accounts(&pid, accounts);
    info!(freelancer = %freelancer_pubkey, jobs = jobs.len(), "Jobs del freelancer cargados");
    Ok(jobs)
}

/// Lista los jobs donde `arbiter_pubkey` es el árbitro.
/// Filtro memcmp en offset 8+32+33=73 (arbiter Pubkey, 32 bytes); el freelancer Option<Pubkey> siempre ocupa 1+32=33 bytes.
pub fn op_list_jobs_as_arbiter(rpc: &RpcClient, arbiter_pubkey: &str) -> Result<Vec<JobInfo>> {
    let pid = program_id()?;
    let apk = Pubkey::from_str(arbiter_pubkey).map_err(|_| anyhow!("Invalid arbiter address"))?;

    // Layout: disc(8) + client(32) + Option<Pubkey>(1+32) + arbiter(32) → arbiter at offset 73
    let filters = vec![
        job_discriminator_filter(),
        RpcFilterType::Memcmp(Memcmp::new(
            73,
            MemcmpEncodedBytes::Bytes(apk.to_bytes().to_vec()),
        )),
    ];

    debug!(arbiter = %arbiter_pubkey, "Buscando jobs del árbitro");
    let accounts = rpc
        .get_program_accounts_with_config(&pid, make_job_rpc_config(filters))
        .map_err(|e| {
            error!(arbiter = %arbiter_pubkey, error = %e, "Error en getProgramAccounts (arbiter)");
            e
        })?;
    debug!(found = accounts.len(), "Cuentas encontradas (árbitro)");

    let jobs = parse_raw_job_accounts(&pid, accounts);
    info!(arbiter = %arbiter_pubkey, jobs = jobs.len(), "Jobs del árbitro cargados");
    Ok(jobs)
}

/// Lista TODOS los jobs del programa (sin filtro de pubkey).
/// Útil para que los freelancers vean jobs disponibles (estado Funded).
pub fn op_list_all_jobs(rpc: &RpcClient) -> Result<Vec<JobInfo>> {
    let pid = program_id()?;
    let filters = vec![job_discriminator_filter()];

    debug!("Buscando todos los jobs del programa");
    let accounts = rpc
        .get_program_accounts_with_config(&pid, make_job_rpc_config(filters))
        .map_err(|e| {
            error!(error = %e, "Error en getProgramAccounts (all jobs)");
            e
        })?;
    debug!(found = accounts.len(), "Total cuentas Job encontradas");

    let jobs = parse_raw_job_accounts(&pid, accounts);
    info!(jobs = jobs.len(), "Todos los jobs cargados");
    Ok(jobs)
}

/// Retorna el balance en lamports de la wallet indicada.
pub fn op_get_balance(rpc: &RpcClient, pubkey: &str) -> Result<u64> {
    let pk = Pubkey::from_str(pubkey).map_err(|_| anyhow!("Invalid pubkey"))?;
    Ok(rpc.get_balance(&pk)?)
}

/// Una transacción reciente de la wallet.
#[derive(Clone)]
pub struct TxInfo {
    pub signature: String,
    pub block_time: Option<i64>,
    pub success: bool,
    /// Cambio neto en lamports: positivo = crédito, negativo = débito.
    pub delta_lamports: i64,
}

/// Retorna las últimas `limit` transacciones de la wallet con delta de SOL.
pub fn op_get_recent_txs(rpc: &RpcClient, pubkey: &str, limit: usize) -> Result<Vec<TxInfo>> {
    use anchor_client::solana_sdk::signature::Signature;

    let pk = Pubkey::from_str(pubkey).map_err(|_| anyhow!("Invalid pubkey"))?;

    let sigs = rpc.get_signatures_for_address_with_config(
        &pk,
        GetConfirmedSignaturesForAddress2Config {
            limit: Some(limit),
            before: None,
            until: None,
            commitment: Some(CommitmentConfig::finalized()),
        },
    )?;

    let mut result = Vec::new();
    for sig_info in &sigs {
        let success = sig_info.err.is_none();
        let block_time = sig_info.block_time;

        let delta = sig_info
            .signature
            .parse::<Signature>()
            .ok()
            .and_then(|sig| {
                rpc.get_transaction_with_config(
                    &sig,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Json),
                        commitment: Some(CommitmentConfig::finalized()),
                        max_supported_transaction_version: Some(0),
                    },
                )
                .ok()
            })
            .and_then(|tx| {
                let meta = tx.transaction.meta?;
                let pre = meta.pre_balances;
                let post = meta.post_balances;
                let idx = match &tx.transaction.transaction {
                    EncodedTransaction::Json(ui_tx) => match &ui_tx.message {
                        UiMessage::Raw(raw) => raw.account_keys.iter().position(|k| k == pubkey),
                        _ => None,
                    },
                    _ => None,
                }?;
                Some(post[idx] as i64 - pre[idx] as i64)
            })
            .unwrap_or(0);

        result.push(TxInfo {
            signature: sig_info.signature.clone(),
            block_time,
            success,
            delta_lamports: delta,
        });
    }

    Ok(result)
}

/// Solicita un airdrop de `amount_sol` SOL a la wallet indicada.
/// Solo funciona en devnet o localhost — no en mainnet.
pub fn op_airdrop(rpc: &RpcClient, pubkey: &str, amount_sol: f64) -> Result<String> {
    let pk = Pubkey::from_str(pubkey).map_err(|_| anyhow!("Invalid pubkey"))?;
    let lamports = (amount_sol * 1e9) as u64;
    let sig = rpc.request_airdrop(&pk, lamports)?;
    Ok(format!(
        "💧 Airdrop: {amount_sol} SOL → {pubkey}\n   Tx: {sig}"
    ))
}

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
            job_id: 1699000000,
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
            delivery_notes: String::new(),
            resolution_notes: String::new(),
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
            job_id: 0,
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
            delivery_notes: String::new(),
            resolution_notes: String::new(),
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

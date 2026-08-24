//! Transaction construction and relay primitives.
//!
//! The browser owns the user key and signs the returned bytes. This module
//! deliberately never signs user transactions; it only builds unsigned
//! messages and validates/relays bytes that already contain a user signature.

use crate::cluster::parse_cluster;
use crate::error::{BackendError, Result};
use crate::pda;

#[allow(deprecated)]
use anchor_client::solana_sdk::{
    hash::{hash, Hash},
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    system_program,
    transaction::Transaction,
};
use borsh::ser::BorshSerialize;
use solana_client::rpc_client::RpcClient;

/// A serialized transaction that has no signatures yet.
#[derive(Debug, Clone)]
pub struct UnsignedTransaction {
    pub transaction: Transaction,
    pub signer: Pubkey,
}

impl UnsignedTransaction {
    pub fn from_transaction(transaction: Transaction, signer: Pubkey) -> Self {
        Self {
            transaction,
            signer,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(&self.transaction)
            .map_err(|e| BackendError::serialization_error(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > 1_000_000 {
            return Err(BackendError::serialization_error(
                "invalid transaction size",
            ));
        }
        let transaction: Transaction = bincode::deserialize(bytes)
            .map_err(|e| BackendError::serialization_error(e.to_string()))?;
        let signer = transaction
            .message
            .account_keys
            .first()
            .copied()
            .ok_or_else(|| BackendError::serialization_error("transaction has no signer"))?;
        Ok(Self {
            transaction,
            signer,
        })
    }
}

/// Anchor instruction discriminator for a global instruction.
pub fn discriminator(name: &str) -> Vec<u8> {
    hash(format!("global:{name}").as_bytes())
        .to_bytes()
        .into_iter()
        .take(8)
        .collect()
}

fn instruction(
    name: &str,
    accounts: Vec<AccountMeta>,
    args: impl BorshSerialize,
) -> Result<Instruction> {
    let mut data = discriminator(name);
    data.extend(
        borsh::to_vec(&args).map_err(|e| BackendError::serialization_error(e.to_string()))?,
    );
    Ok(Instruction {
        program_id: crate::PROGRAM_ID_STR
            .parse()
            .map_err(BackendError::SolanaSdk)?,
        accounts,
        data,
    })
}

pub fn build_create_job_instruction(
    signer: &Pubkey,
    job_id: u64,
    amount: u64,
    deadline: i64,
) -> Result<Instruction> {
    let (job, _) = pda::get_job_pda(signer, job_id)?;
    let (config, _) = pda::get_config_pda()?;
    instruction(
        "create_job",
        vec![
            AccountMeta::new(*signer, true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        (job_id, amount, deadline),
    )
}

pub fn build_deposit_funds_instruction(signer: &Pubkey, job_id: u64) -> Result<Instruction> {
    let (job, _) = pda::get_job_pda(signer, job_id)?;
    let (config, _) = pda::get_config_pda()?;
    instruction(
        "deposit_funds",
        vec![
            AccountMeta::new(*signer, true),
            AccountMeta::new(job, false),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        (job_id,),
    )
}

/// Validate a signed transaction before allowing it to cross the RPC boundary.
pub fn validate_signed_transaction(
    transaction: &Transaction,
    expected_signer: &Pubkey,
    cluster: &str,
) -> Result<()> {
    let _ = parse_cluster(cluster)?;
    if transaction.message.header.num_required_signatures == 0
        || transaction.signatures.len()
            < transaction.message.header.num_required_signatures as usize
    {
        return Err(BackendError::sdk_error(
            "transaction has no required signer",
        ));
    }
    if transaction.message.account_keys.first() != Some(expected_signer)
        || transaction
            .signatures
            .first()
            .map_or(true, |s| *s == Signature::default())
    {
        return Err(BackendError::sdk_error(
            "transaction signer does not match request",
        ));
    }
    let program_id: Pubkey = crate::PROGRAM_ID_STR
        .parse()
        .map_err(BackendError::SolanaSdk)?;
    let contains_program = transaction.message.instructions.iter().any(|ix| {
        transaction
            .message
            .account_keys
            .get(ix.program_id_index as usize)
            == Some(&program_id)
    });
    if !contains_program {
        return Err(BackendError::sdk_error(
            "transaction does not target trust escrow program",
        ));
    }
    transaction
        .verify()
        .map_err(|e| BackendError::sdk_error(format!("invalid transaction signature: {e}")))
}

/// Return the blockhash needed by the frontend to sign a transaction.
pub fn build_unsigned_transaction(
    rpc: &RpcClient,
    signer: &Pubkey,
    instructions: Vec<Instruction>,
) -> Result<UnsignedTransaction> {
    let blockhash: Hash = rpc
        .get_latest_blockhash()
        .map_err(|e| BackendError::sdk_error(e.to_string()))?;
    let message = Message::new(&instructions, Some(signer));
    let mut transaction = Transaction::new_unsigned(message);
    transaction.message.recent_blockhash = blockhash;
    Ok(UnsignedTransaction::from_transaction(transaction, *signer))
}

/// Relay a transaction signed by the wallet. No backend signer is involved.
pub fn relay_signed_transaction(rpc: &RpcClient, transaction: &Transaction) -> Result<Signature> {
    rpc.send_and_confirm_transaction(transaction)
        .map_err(|e| BackendError::sdk_error(e.to_string()))
}

#![cfg(feature = "solana")]

use anchor_client::solana_sdk::{
    hash::Hash, pubkey::Pubkey, signature::Signer, transaction::Transaction,
};
use trust_escrow_sdk::pda::{get_config_pda, get_job_pda};
use trust_escrow_sdk::relay::{
    build_create_job_instruction, build_deposit_funds_instruction, discriminator,
    validate_signed_transaction, UnsignedTransaction,
};
use trust_escrow_sdk::PROGRAM_ID_STR;

#[test]
fn discriminators_match_anchor_global_names() {
    assert_eq!(discriminator("create_job").len(), 8);
    assert_ne!(discriminator("create_job"), discriminator("deposit_funds"));
}

#[test]
fn unsigned_create_job_contains_expected_pda_and_serialized_args() {
    let signer = Pubkey::new_unique();
    let job_id = 42;
    let ix = build_create_job_instruction(&signer, job_id, 123, 456).unwrap();
    let (job, _) = get_job_pda(&signer, job_id).unwrap();
    let (config, _) = get_config_pda().unwrap();

    assert_eq!(ix.program_id, PROGRAM_ID_STR.parse().unwrap());
    assert_eq!(ix.accounts[0].pubkey, signer);
    assert_eq!(ix.accounts[1].pubkey, job);
    assert_eq!(ix.accounts[2].pubkey, config);
    assert_eq!(&ix.data[..8], discriminator("create_job").as_slice());
}

#[test]
fn unsigned_transaction_round_trips_without_signatures() {
    let signer = Pubkey::new_unique();
    let ix = build_deposit_funds_instruction(&signer, 7).unwrap();
    let tx = Transaction::new_unsigned(anchor_client::solana_sdk::message::Message::new(
        &[ix],
        Some(&signer),
    ));
    let envelope = UnsignedTransaction::from_transaction(tx.clone(), signer);
    let bytes = envelope.to_bytes().unwrap();
    let decoded = UnsignedTransaction::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.signer, signer);
    assert!(decoded
        .transaction
        .signatures
        .iter()
        .all(|s| s.as_ref() == [0; 64]));
}

#[test]
fn signed_transaction_validation_requires_expected_signer_and_program() {
    let keypair = anchor_client::solana_sdk::signature::Keypair::new();
    let signer = keypair.pubkey();
    let ix = build_deposit_funds_instruction(&signer, 7).unwrap();
    let mut tx = Transaction::new_unsigned(anchor_client::solana_sdk::message::Message::new(
        &[ix],
        Some(&signer),
    ));
    tx.try_sign(&[&keypair], Hash::new_unique()).unwrap();
    assert!(validate_signed_transaction(&tx, &keypair.pubkey(), "localnet").is_ok());
    assert!(validate_signed_transaction(&tx, &Pubkey::new_unique(), "localnet").is_err());
}

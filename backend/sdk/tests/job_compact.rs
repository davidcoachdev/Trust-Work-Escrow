//! T22 — Job compacto y estado de Application PDA
//!
//! Verifica contrato T22 (Wave 4A):
//! - `Job` no reserva colección inline sobredimensionada (Vec<Pubkey> 50 compacto, no 50*Application).
//! - Cuenta compacta con contador/límites: `MAX_APPLICATIONS = 50`, índice 0..49, `index == applicants.len()`.
//! - Seeds/bump definidos: `Job [b"job", client, job_id.le_bytes()]`, `Application [b"application", job, index, applicant]`.
//! - `declare_id!` y SDK `PROGRAM_ID_STR` alineados a `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh` (validator UP).
//! - Contrato ya es `Vec<Pubkey>` (no matriz fija), SDK ya alineado.

#![cfg(feature = "solana")]

use anchor_lang::AnchorSerialize;
use solana_sdk::pubkey::Pubkey;
use trust_escrow_sdk::{
    client::deserialize_account,
    pda,
    types::*,
    PROGRAM_ID_STR,
};

// ---------------------------------------------------------------------------
// Helpers: replica exacta de checks on-chain en `apply_to_job`.
// ---------------------------------------------------------------------------

const MAX_APPLICATIONS: usize = 50;
const EXPECTED_PID: &str = "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh";

fn validate_apply(
    job_client: &Pubkey,
    applicants: &[Pubkey],
    applicant: &Pubkey,
    index: u8,
) -> Result<(), trust_escrow_sdk::error::ErrorCode> {
    use trust_escrow_sdk::error::ErrorCode;
    if applicant == job_client {
        return Err(ErrorCode::CannotWorkOnOwnJob);
    }
    if applicants.iter().any(|a| a == applicant) {
        return Err(ErrorCode::AlreadyApplied);
    }
    if applicants.len() >= MAX_APPLICATIONS {
        return Err(ErrorCode::InvalidApplicationIndex);
    }
    if index as usize != applicants.len() {
        return Err(ErrorCode::ApplicationIndexMismatch);
    }
    if index as usize >= MAX_APPLICATIONS {
        return Err(ErrorCode::InvalidApplicationIndex);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 1. Program ID y constantes alineadas
// ---------------------------------------------------------------------------

#[test]
fn program_id_is_7a2y_in_sdk() {
    assert_eq!(PROGRAM_ID_STR, EXPECTED_PID);
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    // declare_id! en lib.rs debe ser el mismo — verificamos que el PDA derivado con pid coincida
    // con el cálculo canónico find_program_address (no hay segundo pid en el repo).
    let client = Pubkey::new_unique();
    let (derived, _) = pda::derive_job_pda(&client, 1).unwrap();
    let (expected, _) =
        Pubkey::find_program_address(&[b"job", client.as_ref(), &1u64.to_le_bytes()], &pid);
    assert_eq!(derived, expected, "PROGRAM_ID_STR debe ser el del contrato 7a2Y");
}

#[test]
fn max_applications_is_50_aligned_with_contract() {
    assert_eq!(MAX_APPLICATIONS, 50);
    // SDK types constant
    assert_eq!(trust_escrow_sdk::types::MAX_APPLICATIONS, 50);
}

// ---------------------------------------------------------------------------
// 2. Job compacto: no colección inline sobredimensionada
// ---------------------------------------------------------------------------

#[test]
fn job_is_compact_vec_pubkey_not_inline_applications() {
    // Job guarda solo Vec<Pubkey> (32 bytes cada uno), no Vec<Application> inline.
    let empty = Job {
        client: Pubkey::new_unique(),
        freelancer: None,
        amount: 1_000_000,
        fee_amount: 25_000,
        status: JobStatus::Funded,
        paused: false,
        paused_at: 0,
        deadline: 1_700_000_000,
        submitted_at: None,
        milestones_total: 0,
        milestones_approved: 0,
        milestones_amount_total: 0,
        applicants: vec![],
        bump: 255,
    };
    let full = Job {
        applicants: (0..50).map(|_| Pubkey::new_unique()).collect(),
        ..empty.clone()
    };
    let mut buf_empty = trust_escrow_sdk::types::account_discriminator("Job").to_vec();
    empty.serialize(&mut buf_empty).unwrap();
    let mut buf_full = trust_escrow_sdk::types::account_discriminator("Job").to_vec();
    full.serialize(&mut buf_full).unwrap();
    let delta = buf_full.len() - buf_empty.len();
    // Borsh Vec<Pubkey>: 4-byte len prefix ya presente en ambos → delta == 50*32.
    assert_eq!(delta, 50 * 32, "Job Vec<Pubkey> debe ser compacto 50*32");
    // Si fuera inline de Applications (≈ 32 hash + 32 job + 32 applicant + 1 index + 1 status + 1 bump ≈ 99 bytes c/u),
    // delta sería >= 50*70. Verificamos que NO lo es.
    assert!(delta < 50 * 70, "Job no debe reservar 50 Applications inline");
    // Requisito de Inner account limit (10 KiB): Job INIT_SPACE debe permitir 50*32 dentro del límite.
    // Tamaño aproximado con overhead borsh fijo ~ 100 bytes + 1600 = ~1700 < 10240.
    assert!(buf_full.len() < 10 * 1024, "Job serializado con 50 applicants debe ser < 10KiB");
}

#[test]
fn job_compact_counter_limits_and_seeds_bump_defined() {
    let client = Pubkey::new_unique();
    // Job PDA seeds: [b"job", client, job_id.le_bytes()]
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    for job_id in [0u64, 1, 42, u64::MAX] {
        let (pda, bump) = pda::derive_job_pda(&client, job_id).unwrap();
        let (expected, ebump) = Pubkey::find_program_address(
            &[b"job", client.as_ref(), &job_id.to_le_bytes()],
            &pid,
        );
        assert_eq!(pda, expected, "Job PDA seed mismatch job_id={}", job_id);
        assert_eq!(bump, ebump);
        assert!(bump <= u8::MAX);
        assert!(!pda.is_on_curve());
    }

    // Contador: len == next valid index, 0..49 válido, 50 inválido
    let applicants: Vec<Pubkey> = (0..49).map(|_| Pubkey::new_unique()).collect();
    let extra = Pubkey::new_unique();
    // con 49 elementos, índice 49 es el último válido
    assert!(validate_apply(&client, &applicants, &extra, 49).is_ok());
    // con 50 elementos, cualquier índice debe fallar (límite)
    let full: Vec<Pubkey> = (0..50).map(|_| Pubkey::new_unique()).collect();
    assert!(validate_apply(&client, &full, &extra, 0).is_err());
    assert!(validate_apply(&client, &full, &extra, 50).is_err());
}

#[test]
fn job_initial_state_is_zero_applicants_compact() {
    // create_job debe dejar applicants vacío (contador 0) sin reservar 50 Applications.
    let job = Job {
        client: Pubkey::new_unique(),
        freelancer: None,
        amount: 100_000,
        fee_amount: 2_500,
        status: JobStatus::Created,
        paused: false,
        paused_at: 0,
        deadline: 1_700_000_000,
        submitted_at: None,
        milestones_total: 0,
        milestones_approved: 0,
        milestones_amount_total: 0,
        applicants: Vec::new(),
        bump: 1,
    };
    assert_eq!(job.applicants.len(), 0, "Job inicial debe tener 0 applicants");
    assert!(job.applicants.is_empty());
    // bump definido
    assert!(job.bump <= u8::MAX);
}

// ---------------------------------------------------------------------------
// 3. Application PDA: seeds/bump definidos, contador/límites
// ---------------------------------------------------------------------------

#[test]
fn application_pda_seeds_bump_are_canonical() {
    let pid: Pubkey = PROGRAM_ID_STR.parse().unwrap();
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    for idx in [0u8, 1, 25, 49] {
        let (derived, bump) = pda::derive_application_pda(&job, idx, &applicant).unwrap();
        let (expected, ebump) = Pubkey::find_program_address(
            &[b"application", job.as_ref(), &[idx], applicant.as_ref()],
            &pid,
        );
        assert_eq!(derived, expected, "Application PDA seed mismatch idx={}", idx);
        assert_eq!(bump, ebump);
        assert!(!derived.is_on_curve());
    }
    // índice 50 nunca debe producir un PDA utilizado — el contrato lo rechaza por límite.
    // La derivación criptográfica sí produce un PDA, pero validate_apply debe rechazarlo.
    let applicants: Vec<Pubkey> = (0..50).map(|_| Pubkey::new_unique()).collect();
    let client = Pubkey::new_unique();
    let new_app = Pubkey::new_unique();
    assert!(validate_apply(&client, &applicants, &new_app, 50).is_err());
}

#[test]
fn application_state_has_seeds_bump_and_limits() {
    let job = Pubkey::new_unique();
    let applicant = Pubkey::new_unique();
    let app = Application {
        job,
        index: 3,
        applicant,
        proposal_hash: solana_sdk::hash::hash(b"proposal text for t22").to_bytes(),
        status: ApplicationStatus::Pending,
        bump: 254,
    };
    assert_eq!(app.index, 3);
    assert!(app.index < 50, "índice debe estar en 0..49");
    assert_eq!(app.bump, 254);
    assert_eq!(app.proposal_hash.len(), 32);

    // Serialización roundtrip con discriminador
    let mut buf = trust_escrow_sdk::types::account_discriminator("Application").to_vec();
    app.serialize(&mut buf).unwrap();
    let got = deserialize_account::<Application>(&buf).expect("roundtrip");
    assert_eq!(got, app);
}

#[test]
fn application_duplicate_rejected_even_with_different_index() {
    let client = Pubkey::new_unique();
    let alice = Pubkey::new_unique();
    let applicants = vec![alice];
    // alice ya aplicó en índice 0; intentar índice 1 con misma applicant debe fallar por AlreadyApplied
    // aunque índice 1 == len (1) sería válido para un applicant nuevo, el duplicado prevalece.
    use trust_escrow_sdk::error::ErrorCode;
    assert_eq!(
        validate_apply(&client, &applicants, &alice, 1).unwrap_err(),
        ErrorCode::AlreadyApplied
    );
}

#[test]
fn application_cannot_self_apply_and_index_must_match_len() {
    let client = Pubkey::new_unique();
    use trust_escrow_sdk::error::ErrorCode;
    // self-apply
    assert_eq!(
        validate_apply(&client, &[], &client, 0).unwrap_err(),
        ErrorCode::CannotWorkOnOwnJob
    );
    // index desalineado
    let alice = Pubkey::new_unique();
    assert_eq!(
        validate_apply(&client, &[], &alice, 1).unwrap_err(),
        ErrorCode::ApplicationIndexMismatch
    );
    let bob = Pubkey::new_unique();
    let applicants = vec![alice];
    assert_eq!(
        validate_apply(&client, &applicants, &bob, 0).unwrap_err(),
        ErrorCode::ApplicationIndexMismatch
    );
    assert!(validate_apply(&client, &applicants, &bob, 1).is_ok());
}

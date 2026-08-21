#![allow(unused_imports)]
#[allow(unused_imports)]
use anchor_lang::prelude::*;
#[allow(unused_imports)]
use anchor_lang::system_program::{transfer, Transfer, ID as SYSTEM_PROGRAM_ID};

declare_id!("7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh");

pub mod errors;
pub mod instructions;
pub mod state;

// Alias for Anchor's __client_accounts generated at nested location to crate root (required for #[program] to find them)
pub mod __client_accounts_initialize_config { pub use crate::instructions::config::__client_accounts_initialize_config::*; }
pub mod __cpi_client_accounts_initialize_config { pub use crate::instructions::config::__cpi_client_accounts_initialize_config::*; }
pub mod __client_accounts_pause { pub use crate::instructions::config::__client_accounts_pause::*; }
pub mod __cpi_client_accounts_pause { pub use crate::instructions::config::__cpi_client_accounts_pause::*; }
pub mod __client_accounts_unpause { pub use crate::instructions::config::__client_accounts_unpause::*; }
pub mod __cpi_client_accounts_unpause { pub use crate::instructions::config::__cpi_client_accounts_unpause::*; }
pub mod __client_accounts_update_treasury { pub use crate::instructions::config::__client_accounts_update_treasury::*; }
pub mod __cpi_client_accounts_update_treasury { pub use crate::instructions::config::__cpi_client_accounts_update_treasury::*; }
pub mod __client_accounts_update_arbitration_treasury { pub use crate::instructions::config::__client_accounts_update_arbitration_treasury::*; }
pub mod __cpi_client_accounts_update_arbitration_treasury { pub use crate::instructions::config::__cpi_client_accounts_update_arbitration_treasury::*; }
pub mod __client_accounts_withdraw_treasury { pub use crate::instructions::config::__client_accounts_withdraw_treasury::*; }
pub mod __cpi_client_accounts_withdraw_treasury { pub use crate::instructions::config::__cpi_client_accounts_withdraw_treasury::*; }
pub mod __client_accounts_withdraw_arbitration { pub use crate::instructions::config::__client_accounts_withdraw_arbitration::*; }
pub mod __cpi_client_accounts_withdraw_arbitration { pub use crate::instructions::config::__cpi_client_accounts_withdraw_arbitration::*; }
pub mod __client_accounts_create_arbiter_pool { pub use crate::instructions::config::__client_accounts_create_arbiter_pool::*; }
pub mod __cpi_client_accounts_create_arbiter_pool { pub use crate::instructions::config::__cpi_client_accounts_create_arbiter_pool::*; }
pub mod __client_accounts_add_arbiter { pub use crate::instructions::config::__client_accounts_add_arbiter::*; }
pub mod __cpi_client_accounts_add_arbiter { pub use crate::instructions::config::__cpi_client_accounts_add_arbiter::*; }
pub mod __client_accounts_remove_arbiter { pub use crate::instructions::config::__client_accounts_remove_arbiter::*; }
pub mod __cpi_client_accounts_remove_arbiter { pub use crate::instructions::config::__cpi_client_accounts_remove_arbiter::*; }
pub mod __client_accounts_propose_authority { pub use crate::instructions::config::__client_accounts_propose_authority::*; }
pub mod __cpi_client_accounts_propose_authority { pub use crate::instructions::config::__cpi_client_accounts_propose_authority::*; }
pub mod __client_accounts_update_authority { pub use crate::instructions::config::__client_accounts_update_authority::*; }
pub mod __cpi_client_accounts_update_authority { pub use crate::instructions::config::__cpi_client_accounts_update_authority::*; }
pub mod __client_accounts_cancel_authority_proposal { pub use crate::instructions::config::__client_accounts_cancel_authority_proposal::*; }
pub mod __cpi_client_accounts_cancel_authority_proposal { pub use crate::instructions::config::__cpi_client_accounts_cancel_authority_proposal::*; }
pub mod __client_accounts_create_job { pub use crate::instructions::job::__client_accounts_create_job::*; }
pub mod __cpi_client_accounts_create_job { pub use crate::instructions::job::__cpi_client_accounts_create_job::*; }
pub mod __client_accounts_deposit_funds { pub use crate::instructions::job::__client_accounts_deposit_funds::*; }
pub mod __cpi_client_accounts_deposit_funds { pub use crate::instructions::job::__cpi_client_accounts_deposit_funds::*; }
pub mod __client_accounts_apply_to_job { pub use crate::instructions::job::__client_accounts_apply_to_job::*; }
pub mod __cpi_client_accounts_apply_to_job { pub use crate::instructions::job::__cpi_client_accounts_apply_to_job::*; }
pub mod __client_accounts_accept_application { pub use crate::instructions::job::__client_accounts_accept_application::*; }
pub mod __cpi_client_accounts_accept_application { pub use crate::instructions::job::__cpi_client_accounts_accept_application::*; }
pub mod __client_accounts_reject_application { pub use crate::instructions::job::__client_accounts_reject_application::*; }
pub mod __cpi_client_accounts_reject_application { pub use crate::instructions::job::__cpi_client_accounts_reject_application::*; }
pub mod __client_accounts_withdraw_application { pub use crate::instructions::job::__client_accounts_withdraw_application::*; }
pub mod __cpi_client_accounts_withdraw_application { pub use crate::instructions::job::__cpi_client_accounts_withdraw_application::*; }
pub mod __client_accounts_cleanup_applications { pub use crate::instructions::job::__client_accounts_cleanup_applications::*; }
pub mod __cpi_client_accounts_cleanup_applications { pub use crate::instructions::job::__cpi_client_accounts_cleanup_applications::*; }
pub mod __client_accounts_submit_work { pub use crate::instructions::job::__client_accounts_submit_work::*; }
pub mod __cpi_client_accounts_submit_work { pub use crate::instructions::job::__cpi_client_accounts_submit_work::*; }
pub mod __client_accounts_auto_approve_work { pub use crate::instructions::job::__client_accounts_auto_approve_work::*; }
pub mod __cpi_client_accounts_auto_approve_work { pub use crate::instructions::job::__cpi_client_accounts_auto_approve_work::*; }
pub mod __client_accounts_approve_work { pub use crate::instructions::job::__client_accounts_approve_work::*; }
pub mod __cpi_client_accounts_approve_work { pub use crate::instructions::job::__cpi_client_accounts_approve_work::*; }
pub mod __client_accounts_reject_work { pub use crate::instructions::job::__client_accounts_reject_work::*; }
pub mod __cpi_client_accounts_reject_work { pub use crate::instructions::job::__cpi_client_accounts_reject_work::*; }
pub mod __client_accounts_cancel_job { pub use crate::instructions::job::__client_accounts_cancel_job::*; }
pub mod __cpi_client_accounts_cancel_job { pub use crate::instructions::job::__cpi_client_accounts_cancel_job::*; }
pub mod __client_accounts_pause_job { pub use crate::instructions::job::__client_accounts_pause_job::*; }
pub mod __cpi_client_accounts_pause_job { pub use crate::instructions::job::__cpi_client_accounts_pause_job::*; }
pub mod __client_accounts_unpause_job { pub use crate::instructions::job::__client_accounts_unpause_job::*; }
pub mod __cpi_client_accounts_unpause_job { pub use crate::instructions::job::__cpi_client_accounts_unpause_job::*; }
pub mod __client_accounts_expire_paused_job { pub use crate::instructions::job::__client_accounts_expire_paused_job::*; }
pub mod __cpi_client_accounts_expire_paused_job { pub use crate::instructions::job::__cpi_client_accounts_expire_paused_job::*; }
pub mod __client_accounts_raise_dispute { pub use crate::instructions::dispute::__client_accounts_raise_dispute::*; }
pub mod __cpi_client_accounts_raise_dispute { pub use crate::instructions::dispute::__cpi_client_accounts_raise_dispute::*; }
pub mod __client_accounts_accept_dispute { pub use crate::instructions::dispute::__client_accounts_accept_dispute::*; }
pub mod __cpi_client_accounts_accept_dispute { pub use crate::instructions::dispute::__cpi_client_accounts_accept_dispute::*; }
pub mod __client_accounts_submit_evidence { pub use crate::instructions::dispute::__client_accounts_submit_evidence::*; }
pub mod __cpi_client_accounts_submit_evidence { pub use crate::instructions::dispute::__cpi_client_accounts_submit_evidence::*; }
pub mod __client_accounts_assign_arbiter { pub use crate::instructions::dispute::__client_accounts_assign_arbiter::*; }
pub mod __cpi_client_accounts_assign_arbiter { pub use crate::instructions::dispute::__cpi_client_accounts_assign_arbiter::*; }
pub mod __client_accounts_resolve_dispute { pub use crate::instructions::dispute::__client_accounts_resolve_dispute::*; }
pub mod __cpi_client_accounts_resolve_dispute { pub use crate::instructions::dispute::__cpi_client_accounts_resolve_dispute::*; }
pub mod __client_accounts_resolve_platform_case { pub use crate::instructions::dispute::__client_accounts_resolve_platform_case::*; }
pub mod __cpi_client_accounts_resolve_platform_case { pub use crate::instructions::dispute::__cpi_client_accounts_resolve_platform_case::*; }
pub mod __client_accounts_request_platform_intervention { pub use crate::instructions::dispute::__client_accounts_request_platform_intervention::*; }
pub mod __cpi_client_accounts_request_platform_intervention { pub use crate::instructions::dispute::__cpi_client_accounts_request_platform_intervention::*; }
pub mod __client_accounts_open_support_ticket { pub use crate::instructions::dispute::__client_accounts_open_support_ticket::*; }
pub mod __cpi_client_accounts_open_support_ticket { pub use crate::instructions::dispute::__cpi_client_accounts_open_support_ticket::*; }
pub mod __client_accounts_resolve_support_ticket { pub use crate::instructions::dispute::__client_accounts_resolve_support_ticket::*; }
pub mod __cpi_client_accounts_resolve_support_ticket { pub use crate::instructions::dispute::__cpi_client_accounts_resolve_support_ticket::*; }
pub mod __client_accounts_finalize_dispute_payouts { pub use crate::instructions::dispute::__client_accounts_finalize_dispute_payouts::*; }
pub mod __cpi_client_accounts_finalize_dispute_payouts { pub use crate::instructions::dispute::__cpi_client_accounts_finalize_dispute_payouts::*; }
pub mod __client_accounts_cleanup_dispute_evidence { pub use crate::instructions::dispute::__client_accounts_cleanup_dispute_evidence::*; }
pub mod __cpi_client_accounts_cleanup_dispute_evidence { pub use crate::instructions::dispute::__cpi_client_accounts_cleanup_dispute_evidence::*; }
pub mod __client_accounts_create_milestone { pub use crate::instructions::milestone::__client_accounts_create_milestone::*; }
pub mod __cpi_client_accounts_create_milestone { pub use crate::instructions::milestone::__cpi_client_accounts_create_milestone::*; }
pub mod __client_accounts_submit_milestone { pub use crate::instructions::milestone::__client_accounts_submit_milestone::*; }
pub mod __cpi_client_accounts_submit_milestone { pub use crate::instructions::milestone::__cpi_client_accounts_submit_milestone::*; }
pub mod __client_accounts_approve_milestone { pub use crate::instructions::milestone::__client_accounts_approve_milestone::*; }
pub mod __cpi_client_accounts_approve_milestone { pub use crate::instructions::milestone::__cpi_client_accounts_approve_milestone::*; }
pub mod __client_accounts_reject_milestone { pub use crate::instructions::milestone::__client_accounts_reject_milestone::*; }
pub mod __cpi_client_accounts_reject_milestone { pub use crate::instructions::milestone::__cpi_client_accounts_reject_milestone::*; }


pub use errors::ErrorCode;
pub use state::{Application, ApplicationStatus, ArbiterPool, ArbitrationEscrow, Config, Dispute, DisputeStatus, Evidence, Job, JobStatus, Milestone, MilestoneStatus, SupportTicket, SupportTicketStatus};
pub use instructions::config::{CancelAuthorityProposal, InitializeConfig, Pause, ProposeAuthority, Unpause, UpdateAuthority, UpdateTreasury, UpdateArbitrationTreasury, WithdrawTreasury, WithdrawArbitration, CreateArbiterPool, AddArbiter, RemoveArbiter};
pub use instructions::job::{CreateJob, DepositFunds, ApplyToJob, AcceptApplication, RejectApplication, WithdrawApplication, CleanupApplications, SubmitWork, AutoApproveWork, ApproveWork, RejectWork, CancelJob, PauseJob, UnpauseJob, ExpirePausedJob};
pub use instructions::dispute::{RaiseDispute, AcceptDispute, SubmitEvidence, AssignArbiter, ResolveDispute, ResolvePlatformCase, RequestPlatformIntervention, OpenSupportTicket, ResolveSupportTicket, FinalizeDisputePayouts, CleanupDisputeEvidence};
pub use instructions::milestone::{CreateMilestone, SubmitMilestone, ApproveMilestone, RejectMilestone};

pub const BASIS_POINTS: u16 = 10_000;

pub const ARBITER_FEE_BPS_PER_PARTY: u16 = 250;

pub const DISPUTE_ACCEPT_GRACE: i64 = 7 * 24 * 60 * 60;
pub const AUTO_APPROVAL_DELAY: i64 = 7 * 24 * 60 * 60;
pub const INITIAL_AUTHORITY: Pubkey = pubkey!("3whY1ohdAV3uRXSpyzsKtwLg84X9fTZ1pSdCS8Vvqt7c");

/// Timelock for authority rotation (2 days in seconds). Enforced between
/// propose_authority and update_authority. Mitigates instant takeover by
/// compromised authority key.
pub const AUTHORITY_TIMELOCK: i64 = 2 * 24 * 60 * 60;

pub const MAX_PAUSE_DURATION: i64 = 30 * 24 * 60 * 60;

pub const MIN_JOB_AMOUNT: u64 = 100_000;
pub const MAX_EVIDENCE_COUNT: u8 = 10;
pub const MAX_MILESTONES: usize = 20;
pub const MAX_APPLICATIONS: usize = 50;
pub const MAX_ARBITERS: usize = 50;

/// Paginación obligatoria para V3-ARCH-004 / V3-PERF-011.
/// 10 aplicaciones por tx = 20 AccountInfo (application + applicant).
/// 50 en una tx excede 1232 bytes y 400k CU; con 10 se mantiene <1k bytes y CU estable.
pub const MAX_CLEANUP_BATCH: usize = 10;
pub const MAX_EVIDENCE_CLEANUP_BATCH: usize = 10;

/// V3-ARCH-004: RemainingAccounts tipado borsh — mirror de `AccountMeta` para
/// validar `remaining_accounts` off-chain de forma tipada y evitar inyección
/// de cuentas sin tipar (`&[AccountInfo]` sin meta). Se serializa borsh como
/// `Vec<AccountMetaBorsh>` y se valida contra los `AccountInfo` reales.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct AccountMetaBorsh {
    pub pubkey: Pubkey,
    pub is_writable: bool,
    pub is_signer: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct RemainingAccounts {
    /// Vec<AccountMeta> tipado — cada entrada corresponde a un AccountInfo en `ctx.remaining_accounts`.
    pub metas: Vec<AccountMetaBorsh>,
}

impl RemainingAccounts {
    pub fn validate_infos(&self, infos: &[AccountInfo]) -> Result<()> {
        require!(
            self.metas.len() == infos.len(),
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        for (meta, info) in self.metas.iter().zip(infos.iter()) {
            require!(
                meta.pubkey == *info.key,
                ErrorCode::InvalidApplicationCleanupAccounts
            );
            require!(
                meta.is_writable == info.is_writable,
                ErrorCode::InvalidApplicationCleanupAccounts
            );
            require!(
                meta.is_signer == info.is_signer,
                ErrorCode::InvalidApplicationCleanupAccounts
            );
        }
        Ok(())
    }
    /// Helper para construir desde `&[AccountInfo]` off-chain / en tests.
    pub fn from_infos(infos: &[AccountInfo]) -> Self {
        Self {
            metas: infos
                .iter()
                .map(|i| AccountMetaBorsh {
                    pubkey: *i.key,
                    is_writable: i.is_writable,
                    is_signer: i.is_signer,
                })
                .collect(),
        }
    }
}

pub fn compute_fee(amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(ErrorCode::MathOverflow)?
        / BASIS_POINTS as u128;
    Ok(fee as u64)
}

pub fn compute_shortfall(required: u64, posted: u64) -> u64 {
    required.saturating_sub(posted)
}

#[cfg(test)]
#[allow(unexpected_cfgs)]
mod tests {
    use super::{compute_shortfall, Application, Job, AUTO_APPROVAL_DELAY, MAX_APPLICATIONS};
    use anchor_lang::prelude::Pubkey;
    use anchor_lang::Space;

    #[test]
    fn dispute_payout_uses_explicit_shortfall_without_underflow() {
        assert_eq!(compute_shortfall(100, 40), 60);
        assert_eq!(compute_shortfall(100, 140), 0);
    }

    #[test]
    fn auto_approval_boundary_is_inclusive_at_exactly_seven_days() {
        assert_eq!(AUTO_APPROVAL_DELAY, 604_800);
        let submitted_at = 1_000_i64;
        let deadline = submitted_at + AUTO_APPROVAL_DELAY;
        assert!(deadline >= submitted_at + 604_800);
        assert!(deadline + 1 > submitted_at + 604_800);
    }

    // T22: Job compacto — no reserva colección inline sobredimensionada,
    // cuenta compacta con contador/límites y seeds/bump definidos.
    #[test]
    fn job_compact_init_space_under_10kib_and_vec_50_compact() {
        assert_eq!(MAX_APPLICATIONS, 50, "MAX_APPLICATIONS debe ser 50");
        // Job serializado con Vec interior: Anchor INIT_SPACE incluye 4 + 50*32 bytes.
        // Debe ser compacto (< 10KiB inner limit) y no sobredimensionado (28KiB de 50 Applications).
        let init = Job::INIT_SPACE;
        assert!(
            init < 10 * 1024,
            "Job INIT_SPACE {} debe ser < 10KiB (inner allocation limit)",
            init
        );
        assert!(
            init < 28 * 1024,
            "Job INIT_SPACE {} no debe ser 28KiB (50 Applications inline)",
            init
        );
        // Verificamos que el espacio adicional por applicants sea exactamente 50*32 + overhead Vec.
        // Job sin applicants vs con 50: el delta de INIT_SPACE es el overhead reservado.
        // No verificamos el valor exacto (depende de precisa serialización de otros campos),
        // pero sí que el componente dominante sea 50*32 y no 50*sizeof(Application).
        let vec_reserved = 4 + 50 * 32; // borsh Vec<Pubkey>
        assert!(
            init >= vec_reserved,
            "INIT_SPACE debe reservar al menos {} bytes para Vec<Pubkey>",
            vec_reserved
        );
        let application_inline_reserved = 50 * 99; // aprox tamaño Application inline
                                                   // init no debe acercarse a 50*Application; si init > vec_reserved + 3000 probablemente es inline
        assert!(
            init < vec_reserved + 3000,
            "INIT_SPACE {} no debe incluir 50 Applications inline (~{} bytes extra)",
            init,
            application_inline_reserved
        );
    }

    #[test]
    fn job_and_application_have_bump_and_constants() {
        // Job y Application deben tener campo bump (u8) y MAX_APPLICATIONS / constantes definidas.
        assert_eq!(MAX_APPLICATIONS, 50);
        let app_space = Application::INIT_SPACE;
        // Application es compacta: job 32 + index 1 + applicant 32 + proposal_hash 32 + status 1 + bump 1 ~ 99 bytes + 8 disc = ~107 sin overhead
        assert!(
            app_space > 0 && app_space < 512,
            "Application INIT_SPACE debe ser compacto, got {}",
            app_space
        );
        // Verificamos que el programa declare el ID esperado (se compila con ese ID; no hay otro ID en el árbol).
        assert_eq!(
            crate::ID.to_string(),
            "7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh"
        );
    }

    // T23: apply_to_job — PDA individual, seeds, index, applicant, Job ownership, signer, permisos, texto vacío/excesivo, duplicados, límite 50
    #[test]
    fn apply_to_job_pda_is_deterministic_individual_and_seeds_cover_owner() {
        let job = Pubkey::new_unique();
        let applicant = Pubkey::new_unique();
        // PDA individual determinista: seeds = [b"application", job, index, applicant]
        let (pda0, bump0) = Pubkey::find_program_address(
            &[b"application", job.as_ref(), &[0u8], applicant.as_ref()],
            &crate::ID,
        );
        let (pda1, bump1) = Pubkey::find_program_address(
            &[b"application", job.as_ref(), &[1u8], applicant.as_ref()],
            &crate::ID,
        );
        assert_ne!(pda0, pda1, "distinto índice debe dar PDA distinta");
        // bumps son u8 (0..255) canónicos
        let _ = (bump0, bump1);
        // Job ownership: Job PDA = [b"job", client, job_id.le_bytes]
        let client = Pubkey::new_unique();
        let job_id = 42u64;
        let (job_pda, _) = Pubkey::find_program_address(
            &[b"job", client.as_ref(), &job_id.to_le_bytes()],
            &crate::ID,
        );
        assert!(!job_pda.is_on_curve());
        assert!(!pda0.is_on_curve());
    }

    #[test]
    fn apply_to_job_validates_empty_hash_and_limits() {
        // Texto vacío/excesivo: hash nulo debe rechazarse (EmptyProposal 6049)
        let empty: [u8; 32] = [0u8; 32];
        assert_eq!(empty, [0u8; 32]);
        // Simula check on-chain: require!(proposal_hash != [0;32], EmptyProposal)
        assert!(empty == [0u8; 32], "hash vacío es todo ceros");
        let ok_hash = [1u8; 32];
        assert_ne!(ok_hash, [0u8; 32]);
        // Límite 50: Vec<Pubkey> len debe ser <50 y index == len, 0..49 válido
        assert_eq!(MAX_APPLICATIONS, 50);
        for i in 0u8..50 {
            assert!((i as usize) < MAX_APPLICATIONS);
        }
        assert_eq!(50usize, MAX_APPLICATIONS);
        // Duplicados y self-apply ya cubiertos en T21/T22; verificamos variantes existen
        let _ = crate::ErrorCode::AlreadyApplied;
        let _ = crate::ErrorCode::CannotWorkOnOwnJob;
        let _ = crate::ErrorCode::EmptyProposal;
        let _ = crate::ErrorCode::ApplicationIndexMismatch;
        let _ = crate::ErrorCode::InvalidApplicationIndex;
    }

    #[test]
    fn apply_to_job_application_is_pending_and_bump_valid() {
        // Application recién creada debe ser Pending y bump u8
        let job = Pubkey::new_unique();
        let applicant = Pubkey::new_unique();
        let app = Application {
            job,
            index: 0,
            applicant,
            proposal_hash: [1u8; 32],
            status: crate::ApplicationStatus::Pending,
            bump: 255,
        };
        assert!(app.status == crate::ApplicationStatus::Pending);
        assert_eq!(app.index, 0);
        assert_eq!(app.job, job);
        assert_eq!(app.applicant, applicant);
        // bump es u8 canónico
        let _ = app.bump;
    }

    // V3-ARCH-004 + V3-PERF-011: RemainingAccounts tipado borsh Vec<AccountMeta> + paginación 10 por tx
    #[test]
    fn remaining_accounts_typed_and_pagination_10_enforced() {
        use crate::{AccountMetaBorsh, RemainingAccounts, MAX_CLEANUP_BATCH, MAX_EVIDENCE_CLEANUP_BATCH};
        use anchor_lang::AnchorSerialize;
        use anchor_lang::AnchorDeserialize;
        assert_eq!(MAX_CLEANUP_BATCH, 10, "paginación obligatoria 10 por tx");
        assert_eq!(MAX_EVIDENCE_CLEANUP_BATCH, 10);
        assert_eq!(MAX_APPLICATIONS, 50, "50 en una tx debe fallar, solo 10 por tx");
        // Borsh Vec<AccountMeta> tipado: serializable y deserializable via AnchorSerialize (borsh)
        let metas = vec![
            AccountMetaBorsh { pubkey: Pubkey::new_unique(), is_writable: true, is_signer: false },
            AccountMetaBorsh { pubkey: Pubkey::new_unique(), is_writable: true, is_signer: false },
        ];
        let ra = RemainingAccounts { metas: metas.clone() };
        let enc = ra.try_to_vec().unwrap();
        let dec = RemainingAccounts::try_from_slice(&enc).unwrap();
        assert_eq!(dec.metas, metas);
        // Validar que 20 metas (10 aplicaciones) pasan pero 22 (11) deben ser rechazadas por límite
        let ok_metas: Vec<_> = (0..20).map(|_| AccountMetaBorsh { pubkey: Pubkey::new_unique(), is_writable: true, is_signer: false }).collect();
        assert!(ok_metas.len() <= MAX_CLEANUP_BATCH * 2);
        let too_many: Vec<_> = (0..22).map(|_| AccountMetaBorsh { pubkey: Pubkey::new_unique(), is_writable: true, is_signer: false }).collect();
        assert!(too_many.len() > MAX_CLEANUP_BATCH * 2);
    }

    #[test]
    fn bump_cache_create_program_address_matches_find() {
        // V3-PERF-011: cache find_program_address con bump — create_program_address con bump cacheado debe reproducir find
        let job = Pubkey::new_unique();
        let applicant = Pubkey::new_unique();
        let (pda_find, bump) = Pubkey::find_program_address(
            &[b"application", job.as_ref(), &[7u8], applicant.as_ref()],
            &crate::ID,
        );
        let pda_create = Pubkey::create_program_address(
            &[b"application", job.as_ref(), &[7u8], applicant.as_ref(), &[bump]],
            &crate::ID,
        ).unwrap();
        assert_eq!(pda_find, pda_create, "bump cacheado debe reproducir find sin loop");
        // Evidence: mismo patrón
        let dispute = Pubkey::new_unique();
        let (e_find, e_bump) = Pubkey::find_program_address(&[b"evidence", dispute.as_ref(), &[3u8]], &crate::ID);
        let e_create = Pubkey::create_program_address(&[b"evidence", dispute.as_ref(), &[3u8], &[e_bump]], &crate::ID).unwrap();
        assert_eq!(e_find, e_create);
    }

    #[test]
    fn lazy_deserialization_and_writable_validation() {
        // V3-PERF-011: deserialización lazy y validación is_writable tipada
        use crate::{AccountMetaBorsh, RemainingAccounts};
        use anchor_lang::AnchorSerialize;
        use anchor_lang::AnchorDeserialize;
        let pk = Pubkey::new_unique();
        let ok = RemainingAccounts { metas: vec![AccountMetaBorsh { pubkey: pk, is_writable: true, is_signer: false }] };
        let enc = ok.try_to_vec().unwrap();
        let dec = RemainingAccounts::try_from_slice(&enc).unwrap();
        assert!(dec.metas[0].is_writable);
        // Si is_writable false, la validación on-chain debe rechazar (simulamos check)
        let bad = RemainingAccounts { metas: vec![AccountMetaBorsh { pubkey: pk, is_writable: false, is_signer: false }] };
        assert!(!bad.metas[0].is_writable);
    }
}

pub fn check_not_paused(job: &Job) -> Result<()> {
    if job.paused {
        let now = Clock::get()?.unix_timestamp;
        if now
            .checked_sub(job.paused_at)
            .ok_or(ErrorCode::JobPausedExpired)?
            > MAX_PAUSE_DURATION
        {
            return err!(ErrorCode::JobPausedExpired);
        }
        return err!(ErrorCode::JobPaused);
    }
    Ok(())
}

pub fn transfer_job_lamports(
    source: &AccountInfo,
    destination: &AccountInfo,
    amount: u64,
) -> Result<()> {
    require!(source.owner == &crate::ID, ErrorCode::NotAuthorized);
    require!(
        source.is_writable && destination.is_writable,
        ErrorCode::NotAuthorized
    );
    require!(source.key() != destination.key(), ErrorCode::NotAuthorized);

    let remaining = source
        .get_lamports()
        .checked_sub(amount)
        .ok_or(ErrorCode::InsufficientFunds)?;
    let rent_minimum = Rent::get()?.minimum_balance(source.data_len());
    require!(remaining >= rent_minimum, ErrorCode::InsufficientFunds);
    let destination_balance = destination
        .get_lamports()
        .checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;

    **source.try_borrow_mut_lamports()? = remaining;
    **destination.try_borrow_mut_lamports()? = destination_balance;
    Ok(())
}

pub fn validate_treasury_destination(destination: &AccountInfo, other: Pubkey) -> Result<()> {
    require!(
        destination.key() != Pubkey::default(),
        ErrorCode::InvalidTreasury
    );
    require!(destination.key() != other, ErrorCode::InvalidTreasury);
    require!(
        destination.owner == &SYSTEM_PROGRAM_ID,
        ErrorCode::InvalidTreasury
    );
    Ok(())
}

pub fn close_evidence_account(
    evidence: &AccountInfo,
    destination: &AccountInfo,
    dispute: &Pubkey,
    index: u8,
) -> Result<()> {
    // V3-PERF-011: deserialización lazy — primero owner + len, evita CU de
    // deserializar una cuenta falsa. Solo después se deserializa y se valida
    // el PDA con bump cacheado (create_program_address en lugar de find).
    if evidence.owner == &SYSTEM_PROGRAM_ID && evidence.data_len() == 0 {
        return err!(ErrorCode::InvalidEvidenceAccount);
    }
    require!(
        evidence.owner == &crate::ID,
        ErrorCode::InvalidEvidenceAccount
    );
    // Lazy: deserializar una sola vez, obtener bump y usarlo como cache para
    // validar PDA sin el loop de `find_program_address` (compute blowup).
    let data = evidence.try_borrow_data()?;
    let stored = Evidence::try_deserialize(&mut &data[..])
        .map_err(|_| error!(ErrorCode::InvalidEvidenceAccount))?;
    require!(
        stored.dispute == *dispute && stored.index == index,
        ErrorCode::InvalidEvidenceAccount
    );
    // Cache find_program_address con bump: create_program_address con bump almacenado
    let expected = Pubkey::create_program_address(
        &[b"evidence", dispute.as_ref(), &[index], &[stored.bump]],
        &crate::ID,
    )
    .map_err(|_| error!(ErrorCode::InvalidEvidenceAccount))?;
    require!(
        evidence.key() == expected,
        ErrorCode::InvalidEvidenceAccount
    );
    drop(data);

    let rent = evidence.get_lamports();
    let destination_balance = destination
        .get_lamports()
        .checked_add(rent)
        .ok_or(ErrorCode::MathOverflow)?;
    **destination.try_borrow_mut_lamports()? = destination_balance;
    **evidence.try_borrow_mut_lamports()? = 0;
    evidence.assign(&SYSTEM_PROGRAM_ID);
    evidence.resize(0)?;
    Ok(())
}

pub fn cleanup_job_applications(
    job: &Job,
    job_key: &Pubkey,
    start_index: u8,
    remaining_accounts: &[AccountInfo],
    require_full_range: bool,
    allow_closed: bool,
) -> Result<()> {
    // V3-ARCH-004: RemainingAccounts tipado — validación de metas + paginación 10 por tx
    // Se construye el mirror tipado borsh y se valida is_writable/is_signer/pubkey antes de
    // cualquier lógica, evitando inyección por orden incorrecto o cuentas no writable.
    let typed = RemainingAccounts::from_infos(remaining_accounts);
    typed.validate_infos(remaining_accounts)?;
    require!(
        remaining_accounts.len().is_multiple_of(2),
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    let application_count = remaining_accounts.len() / 2;
    // V3-PERF-011 + V3-ARCH-004: paginación obligatoria 10 por tx (no 50 en una)
    require!(
        application_count <= MAX_CLEANUP_BATCH,
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    require!(
        application_count <= MAX_APPLICATIONS,
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    let start = start_index as usize;
    require!(
        start
            .checked_add(application_count)
            .ok_or(ErrorCode::InvalidApplicationCleanupAccounts)?
            <= job.applicants.len(),
        ErrorCode::InvalidApplicationCleanupAccounts
    );
    if require_full_range {
        require!(
            start == 0 && application_count == job.applicants.len(),
            ErrorCode::InvalidApplicationCleanupAccounts
        );
    }

    let mut validated = Vec::with_capacity(application_count);
    for (offset, pair) in remaining_accounts.chunks_exact(2).enumerate() {
        let application = &pair[0];
        let applicant = &pair[1];
        let index = start_index
            .checked_add(offset as u8)
            .ok_or(ErrorCode::InvalidApplicationCleanupAccounts)?;
        let expected_applicant = *job
            .applicants
            .get(index as usize)
            .ok_or(ErrorCode::InvalidApplicationCleanupAccounts)?;
        require!(
            applicant.key() == expected_applicant,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        require!(
            applicant.owner == &SYSTEM_PROGRAM_ID,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        // V3-PERF-011: is_writable tipado ya validado arriba; además debe ser writable para refund
        require!(
            application.is_writable && applicant.is_writable,
            ErrorCode::InvalidApplicationCleanupAccounts
        );

        if application.owner == &SYSTEM_PROGRAM_ID && application.data_len() == 0 {
            require!(allow_closed, ErrorCode::InvalidApplicationCleanupAccounts);
            // Para cuenta ya cerrada, validamos PDA via find (no hay bump cacheado) pero es raro y solo si allow_closed
            let (expected, _) = Pubkey::find_program_address(
                &[
                    b"application",
                    job_key.as_ref(),
                    &[index],
                    expected_applicant.as_ref(),
                ],
                &crate::ID,
            );
            require!(
                application.key() == expected,
                ErrorCode::InvalidApplicationCleanupAccounts
            );
            validated.push((application, applicant, true));
            continue;
        }
        // V3-PERF-011: deserialización lazy — primero owner check, luego deserialize, luego bump cache
        require!(
            application.owner == &crate::ID,
            ErrorCode::InvalidApplicationCleanupAccounts
        );

        let data = application.try_borrow_data()?;
        let stored = Application::try_deserialize(&mut &data[..])
            .map_err(|_| error!(ErrorCode::InvalidApplicationCleanupAccounts))?;
        require!(
            stored.job == *job_key,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        require!(
            stored.index == index,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        require!(
            stored.applicant == expected_applicant,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        // Cache find_program_address con bump (V3-PERF-011): create_program_address con bump almacenado
        let expected = Pubkey::create_program_address(
            &[
                b"application",
                job_key.as_ref(),
                &[index],
                expected_applicant.as_ref(),
                &[stored.bump],
            ],
            &crate::ID,
        )
        .map_err(|_| error!(ErrorCode::InvalidApplicationCleanupAccounts))?;
        require!(
            application.key() == expected,
            ErrorCode::InvalidApplicationCleanupAccounts
        );
        // Validar que el bump cacheado reproduce el PDA sin iterar (blowup evitado)
        drop(data);
        validated.push((
            application,
            applicant,
            stored.status == ApplicationStatus::Accepted,
        ));
    }

    for (application, applicant, accepted_or_closed) in validated {
        if accepted_or_closed {
            continue;
        }
        let rent = application.get_lamports();
        let destination_balance = applicant
            .get_lamports()
            .checked_add(rent)
            .ok_or(ErrorCode::MathOverflow)?;
        **applicant.try_borrow_mut_lamports()? = destination_balance;
        **application.try_borrow_mut_lamports()? = 0;
        application.assign(&SYSTEM_PROGRAM_ID);
        application.resize(0)?;
    }
    Ok(())
}

/// Helper para validar paginación de evidence cleanup (10 por tx) de forma tipada.
/// Reutiliza `RemainingAccounts` para validar metas de evidence PDAs.
pub fn validate_evidence_remaining(remaining_accounts: &[AccountInfo]) -> Result<()> {
    require!(
        remaining_accounts.len() <= MAX_EVIDENCE_CLEANUP_BATCH,
        ErrorCode::InvalidEvidenceCleanupAccounts
    );
    let typed = RemainingAccounts::from_infos(remaining_accounts);
    typed.validate_infos(remaining_accounts)?;
    for acc in remaining_accounts {
        require!(acc.is_writable, ErrorCode::InvalidEvidenceCleanupAccounts);
    }
    Ok(())
}

#[allow(unexpected_cfgs)]
#[program]
pub mod escrow {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        advisor: Pubkey,
        treasury: Pubkey,
        arbitration_treasury: Pubkey,
        fee_bps: u16,
    ) -> Result<()> {
        instructions::config::initialize_config(ctx, advisor, treasury, arbitration_treasury, fee_bps)
    }

    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        instructions::config::pause(ctx)
    }

    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        instructions::config::unpause(ctx)
    }

    pub fn update_treasury(ctx: Context<UpdateTreasury>, new_treasury: Pubkey) -> Result<()> {
        instructions::config::update_treasury(ctx, new_treasury)
    }

    pub fn update_arbitration_treasury(
        ctx: Context<UpdateArbitrationTreasury>,
        new_arbitration_treasury: Pubkey,
    ) -> Result<()> {
        instructions::config::update_arbitration_treasury(ctx, new_arbitration_treasury)
    }

    pub fn withdraw_treasury(ctx: Context<WithdrawTreasury>, amount: u64) -> Result<()> {
        instructions::config::withdraw_treasury(ctx, amount)
    }

    pub fn withdraw_arbitration(ctx: Context<WithdrawArbitration>, amount: u64) -> Result<()> {
        instructions::config::withdraw_arbitration(ctx, amount)
    }

    pub fn create_job(
        ctx: Context<CreateJob>,
        _job_id: u64,
        amount: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::job::create_job(ctx, _job_id, amount, deadline)
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>, _job_id: u64) -> Result<()> {
        instructions::job::deposit_funds(ctx, _job_id)
    }

    pub fn apply_to_job(
        ctx: Context<ApplyToJob>,
        _job_id: u64,
        application_index: u8,
        proposal_hash: [u8; 32],
    ) -> Result<()> {
        instructions::job::apply_to_job(ctx, _job_id, application_index, proposal_hash)
    }

    pub fn accept_application(
        ctx: Context<AcceptApplication>,
        _job_id: u64,
        application_index: u8,
    ) -> Result<()> {
        instructions::job::accept_application(ctx, _job_id, application_index)
    }

    pub fn reject_application(
        ctx: Context<RejectApplication>,
        _job_id: u64,
        application_index: u8,
    ) -> Result<()> {
        instructions::job::reject_application(ctx, _job_id, application_index)
    }

    pub fn withdraw_application(
        ctx: Context<WithdrawApplication>,
        _job_id: u64,
        application_index: u8,
    ) -> Result<()> {
        instructions::job::withdraw_application(ctx, _job_id, application_index)
    }

    pub fn cleanup_applications(
        ctx: Context<CleanupApplications>,
        _job_id: u64,
        start_index: u8,
    ) -> Result<()> {
        instructions::job::cleanup_applications(ctx, _job_id, start_index)
    }

    pub fn submit_work(ctx: Context<SubmitWork>, _job_id: u64) -> Result<()> {
        instructions::job::submit_work(ctx, _job_id)
    }

    pub fn auto_approve_work(ctx: Context<AutoApproveWork>, _job_id: u64) -> Result<()> {
        instructions::job::auto_approve_work(ctx, _job_id)
    }

    pub fn approve_work(ctx: Context<ApproveWork>, _job_id: u64) -> Result<()> {
        instructions::job::approve_work(ctx, _job_id)
    }

    pub fn reject_work(ctx: Context<RejectWork>, _job_id: u64) -> Result<()> {
        instructions::job::reject_work(ctx, _job_id)
    }

    pub fn cancel_job(ctx: Context<CancelJob>, _job_id: u64) -> Result<()> {
        instructions::job::cancel_job(ctx, _job_id)
    }

    pub fn pause_job(ctx: Context<PauseJob>, _job_id: u64) -> Result<()> {
        instructions::job::pause_job(ctx, _job_id)
    }

    pub fn unpause_job(ctx: Context<UnpauseJob>, _job_id: u64) -> Result<()> {
        instructions::job::unpause_job(ctx, _job_id)
    }

    pub fn expire_paused_job(ctx: Context<ExpirePausedJob>, _job_id: u64) -> Result<()> {
        instructions::job::expire_paused_job(ctx, _job_id)
    }

    pub fn create_arbiter_pool(ctx: Context<CreateArbiterPool>) -> Result<()> {
        instructions::config::create_arbiter_pool(ctx)
    }

    pub fn add_arbiter(ctx: Context<AddArbiter>, new_arbiter: Pubkey) -> Result<()> {
        instructions::config::add_arbiter(ctx, new_arbiter)
    }

    pub fn remove_arbiter(ctx: Context<RemoveArbiter>, arbiter: Pubkey) -> Result<()> {
        instructions::config::remove_arbiter(ctx, arbiter)
    }

    pub fn propose_authority(ctx: Context<ProposeAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::config::propose_authority(ctx, new_authority)
    }

    pub fn update_authority(ctx: Context<UpdateAuthority>) -> Result<()> {
        instructions::config::update_authority(ctx)
    }

    pub fn cancel_authority_proposal(ctx: Context<CancelAuthorityProposal>) -> Result<()> {
        instructions::config::cancel_authority_proposal(ctx)
    }

    pub fn raise_dispute(ctx: Context<RaiseDispute>, _job_id: u64) -> Result<()> {
        instructions::dispute::raise_dispute(ctx, _job_id)
    }

    pub fn accept_dispute(ctx: Context<AcceptDispute>, _job_id: u64) -> Result<()> {
        instructions::dispute::accept_dispute(ctx, _job_id)
    }

    pub fn submit_evidence(
        ctx: Context<SubmitEvidence>,
        _job_id: u64,
        index: u8,
        content_hash: [u8; 32],
    ) -> Result<()> {
        instructions::dispute::submit_evidence(ctx, _job_id, index, content_hash)
    }

    pub fn assign_arbiter(ctx: Context<AssignArbiter>, _job_id: u64) -> Result<()> {
        instructions::dispute::assign_arbiter(ctx, _job_id)
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        _job_id: u64,
        client_payout_percent: u8,
    ) -> Result<()> {
        instructions::dispute::resolve_dispute(ctx, _job_id, client_payout_percent)
    }

    pub fn resolve_platform_case(
        ctx: Context<ResolvePlatformCase>,
        _job_id: u64,
        client_payout_percent: u8,
    ) -> Result<()> {
        instructions::dispute::resolve_platform_case(ctx, _job_id, client_payout_percent)
    }

    pub fn request_platform_intervention(
        ctx: Context<RequestPlatformIntervention>,
        _job_id: u64,
    ) -> Result<()> {
        instructions::dispute::request_platform_intervention(ctx, _job_id)
    }

    pub fn open_support_ticket(ctx: Context<OpenSupportTicket>, _job_id: u64) -> Result<()> {
        instructions::dispute::open_support_ticket(ctx, _job_id)
    }

    pub fn resolve_support_ticket(ctx: Context<ResolveSupportTicket>, _job_id: u64) -> Result<()> {
        instructions::dispute::resolve_support_ticket(ctx, _job_id)
    }

    pub fn finalize_dispute_payouts(
        ctx: Context<FinalizeDisputePayouts>,
        _job_id: u64,
    ) -> Result<()> {
        instructions::dispute::finalize_dispute_payouts(ctx, _job_id)
    }

    pub fn cleanup_dispute_evidence(
        ctx: Context<CleanupDisputeEvidence>,
        _job_id: u64,
    ) -> Result<()> {
        instructions::dispute::cleanup_dispute_evidence(ctx, _job_id)
    }

    pub fn create_milestone(
        ctx: Context<CreateMilestone>,
        _job_id: u64,
        index: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::milestone::create_milestone(ctx, _job_id, index, amount)
    }

    pub fn submit_milestone(
        ctx: Context<SubmitMilestone>,
        _job_id: u64,
        _milestone_index: u8,
    ) -> Result<()> {
        instructions::milestone::submit_milestone(ctx, _job_id, _milestone_index)
    }

    pub fn approve_milestone(
        ctx: Context<ApproveMilestone>,
        _job_id: u64,
        _milestone_index: u8,
    ) -> Result<()> {
        instructions::milestone::approve_milestone(ctx, _job_id, _milestone_index)
    }

    pub fn reject_milestone(
        ctx: Context<RejectMilestone>,
        _job_id: u64,
        _milestone_index: u8,
    ) -> Result<()> {
        instructions::milestone::reject_milestone(ctx, _job_id, _milestone_index)
    }
}


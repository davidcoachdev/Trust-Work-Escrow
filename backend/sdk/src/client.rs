//! Configurable client for `trust-escrow-v3`.
//!
//! `TrustEscrowClient` wraps an Anchor program client and exposes getters that
//! deserialize the nine on-chain account families into the SDK types, returning
//! `None` when an account is absent. It also provides instruction wrappers
//! (T4: config / jobs / applications / work lifecycle).

#[cfg(feature = "solana")]
mod inner {
    use crate::cluster::{
        check_keypair_permissions, load_keypair_secure, parse_cluster as cluster_parse,
        validate_cluster,
    };
    use crate::error::{BackendError, ErrorCode, Result};
    use crate::pda;
    use crate::relay::{
        build_create_job_instruction, build_deposit_funds_instruction, build_unsigned_transaction,
        relay_signed_transaction, validate_signed_transaction, UnsignedTransaction,
    };
    use crate::types::*;

    use std::sync::Arc;
    use std::time::Duration;

    #[allow(deprecated)]
    use anchor_client::solana_sdk::system_program;
    use anchor_client::{
        solana_sdk::{
            commitment_config::CommitmentConfig,
            hash::hash,
            instruction::{AccountMeta, Instruction, InstructionError},
            pubkey::Pubkey,
            signature::{Keypair, Signature},
            signer::Signer,
            transaction::{Transaction, TransactionError},
        },
        Client, Cluster, Program,
    };
    use anchor_lang::AccountDeserialize;
    use borsh::ser::BorshSerialize;

    /// Deserialize an Anchor account from raw on-chain bytes.
    pub fn deserialize_account<T: AccountDeserialize>(data: &[u8]) -> Option<T> {
        let mut slice = data;
        T::try_deserialize(&mut slice).ok()
    }

    /// Client for `trust-escrow-v3`.
    pub struct TrustEscrowClient {
        program: Program<Arc<Keypair>>,
        /// Keypair that pays for and signs transactions.
        payer: Arc<Keypair>,
    }

    /// Paginated result for job listings (T7, read-through + cursor).
    #[derive(Debug, Clone)]
    pub struct PaginatedJobs {
        pub jobs: Vec<(Pubkey, Job)>,
        pub next_cursor: Option<String>,
        pub has_more: bool,
    }

    /// Paginated result for application listings per job (T8, read-through + cursor).
    #[derive(Debug, Clone)]
    pub struct PaginatedApplications {
        pub applications: Vec<(Pubkey, Application)>,
        pub next_cursor: Option<String>,
        pub has_more: bool,
    }

    impl TrustEscrowClient {
        /// Build a client from a cluster and an in-memory keypair.
        ///
        /// Blocks `Cluster::Mainnet` (and custom URLs containing `mainnet`) unless
        /// an allowlist env var is set (`TRUST_ESCROW_ALLOW_MAINNET=1` or
        /// `ALLOW_MAINNET=1`). This prevents accidental mainnet use in tests/CI.
        pub fn new(cluster: Cluster, keypair: Keypair) -> Result<Self> {
            // T18: mainnet guard — same binary must switch clusters via env, but
            // mainnet is blocked by default.
            validate_cluster(&cluster)?;
            let payer = Arc::new(keypair);
            let program_id = crate::PROGRAM_ID_STR
                .parse::<Pubkey>()
                .map_err(BackendError::SolanaSdk)?;
            let program =
                Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed())
                    .program(program_id)
                    .map_err(|e| BackendError::from(Box::new(e)))?;
            Ok(Self { program, payer })
        }

        /// Build an RPC client without a custodial signing key. The ephemeral
        /// payer is never used to sign user transactions.
        pub fn readonly(cluster: Cluster) -> Result<Self> {
            Self::new(cluster, Keypair::new())
        }

        /// Build, but never sign, a wallet-owned create-job transaction.
        pub fn build_unsigned_create_job(
            &self,
            signer: &Pubkey,
            job_id: u64,
            amount: u64,
            deadline: i64,
        ) -> Result<UnsignedTransaction> {
            let ix = build_create_job_instruction(signer, job_id, amount, deadline)?;
            build_unsigned_transaction(&self.program.rpc(), signer, vec![ix])
        }

        /// Build, but never sign, a wallet-owned deposit transaction.
        pub fn build_unsigned_deposit_funds(
            &self,
            signer: &Pubkey,
            job_id: u64,
        ) -> Result<UnsignedTransaction> {
            let ix = build_deposit_funds_instruction(signer, job_id)?;
            build_unsigned_transaction(&self.program.rpc(), signer, vec![ix])
        }

        /// Validate and relay bytes signed by the wallet; the backend never
        /// adds a signature or receives a private key.
        pub fn relay_signed_transaction(
            &self,
            bytes: &[u8],
            expected_signer: &Pubkey,
            cluster: &str,
        ) -> Result<Signature> {
            let signed = bincode::deserialize(bytes)
                .map_err(|e| BackendError::serialization_error(e.to_string()))?;
            validate_signed_transaction(&signed, expected_signer, cluster)?;
            relay_signed_transaction(&self.program.rpc(), &signed)
        }

        /// Build a client loading the keypair from a filesystem path.
        ///
        /// Validates file permissions (`0600` or stricter `0400`) before reading,
        /// and never logs secret material — errors only mention the path.
        pub fn from_keypair_path(cluster: Cluster, path: &str) -> Result<Self> {
            // T18: secure keypair — check perms first, then load without logging bytes.
            check_keypair_permissions(path)?;
            let keypair = load_keypair_secure(path)?;
            Self::new(cluster, keypair)
        }

        /// Build a client from the environment (`CLUSTER`/`RPC_CLUSTER` and
        /// `KEYPAIR_PATH`). Falls back to `Localnet` when no cluster is set.
        ///
        /// `CLUSTER` takes precedence over `RPC_CLUSTER`. Both are resolved via
        /// [`crate::cluster::parse_cluster`] which blocks mainnet without allowlist.
        pub fn from_env() -> Result<Self> {
            let cluster = match std::env::var("CLUSTER").or_else(|_| std::env::var("RPC_CLUSTER")) {
                Ok(c) => cluster_parse(&c)?,
                Err(_) => Cluster::Localnet,
            };
            // Also validate the resolved cluster (defense in depth).
            validate_cluster(&cluster)?;
            let path = std::env::var("KEYPAIR_PATH")
                .map_err(|_| BackendError::config_error("KEYPAIR_PATH not set"))?;
            Self::from_keypair_path(cluster, &path)
        }

        /// Public key of the backend-owned payer. The browser never receives
        /// or supplies this keypair; it only receives the public address.
        pub fn payer_pubkey(&self) -> Pubkey {
            self.payer.pubkey()
        }

        /// Fetch raw account bytes, mapping "account not found" to `None`.
        fn fetch_optional(&self, addr: &Pubkey) -> Result<Option<Vec<u8>>> {
            match self.program.rpc().get_account_data(addr) {
                Ok(data) => Ok(Some(data)),
                Err(e) => {
                    if is_account_missing(&e) {
                        Ok(None)
                    } else {
                        Err(BackendError::from(Box::new(e)))
                    }
                }
            }
        }

        // ===== GETTERS =====

        pub fn get_config(&self) -> Result<Option<Config>> {
            let (addr, _) = pda::get_config_pda()?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_job(&self, client: &Pubkey, job_id: u64) -> Result<Option<Job>> {
            let (addr, _) = pda::get_job_pda(client, job_id)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_application(
            &self,
            job: &Pubkey,
            index: u8,
            applicant: &Pubkey,
        ) -> Result<Option<Application>> {
            let (addr, _) = pda::get_application_pda(job, index, applicant)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_arbiter_pool(&self) -> Result<Option<ArbiterPool>> {
            let (addr, _) = pda::get_arbiter_pool_pda()?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_dispute(&self, job: &Pubkey) -> Result<Option<Dispute>> {
            let (addr, _) = pda::get_dispute_pda(job)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_arb_fee(&self, job: &Pubkey) -> Result<Option<ArbitrationEscrow>> {
            let (addr, _) = pda::get_arb_fee_pda(job)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_milestone(&self, job: &Pubkey, index: u8) -> Result<Option<Milestone>> {
            let (addr, _) = pda::get_milestone_pda(job, index)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_evidence(&self, dispute: &Pubkey, index: u8) -> Result<Option<Evidence>> {
            let (addr, _) = pda::get_evidence_pda(dispute, index)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        pub fn get_support_ticket(&self, job: &Pubkey) -> Result<Option<SupportTicket>> {
            let (addr, _) = pda::get_support_pda(job)?;
            Ok(self
                .fetch_optional(&addr)?
                .and_then(|d| deserialize_account(&d)))
        }

        // ===== INSTRUCTION BUILDER =====

        /// Build an Anchor instruction (discriminator = sha256("global:<name>")[..8])
        /// plus borsh-serialized args, sign it with the payer and send it.
        async fn anchor_ix(
            &self,
            name: &str,
            accounts: Vec<AccountMeta>,
            args: Vec<u8>,
        ) -> Result<Signature> {
            let program_id = crate::PROGRAM_ID_STR
                .parse::<Pubkey>()
                .map_err(BackendError::SolanaSdk)?;
            let disc = hash(format!("global:{}", name).as_bytes())
                .to_bytes()
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<u8>>();
            let mut data = disc;
            data.extend_from_slice(&args);

            let ix = Instruction {
                program_id,
                accounts,
                data,
            };

            let blockhash = self
                .program
                .rpc()
                .get_latest_blockhash()
                .map_err(|e| BackendError::from(Box::new(e)))?;
            let signers: [&dyn Signer; 1] = [self.payer.as_ref()];
            let tx = Transaction::new_signed_with_payer(
                &[ix],
                Some(&self.payer.pubkey()),
                &signers,
                blockhash,
            );
            let signature = self
                .program
                .rpc()
                .send_and_confirm_transaction(&tx)
                .map_err(|e| BackendError::from(Box::new(e)))?;

            // `send_and_confirm_transaction` only guarantees the transaction was
            // *confirmed* (included in a block), not that the program succeeded.
            // A program failure still lands as a confirmed transaction whose
            // `meta.err` is set, so verify the program-level result here. Without
            // this, an instruction that fails on-chain returns `Ok(Signature)`
            // and the caller only discovers the failure later (e.g. a missing
            // account), masking the real cause.
            let statuses = self
                .program
                .rpc()
                .get_signature_statuses(&[signature])
                .map_err(|e| BackendError::from(Box::new(e)))?
                .value;
            let status = statuses.into_iter().next().flatten().ok_or_else(|| {
                BackendError::sdk_error("missing status for confirmed transaction")
            })?;
            if let Some(err) = status.err {
                return Err(Self::map_program_error(err));
            }
            Ok(signature)
        }

        /// Map a confirmed-but-failed transaction error to a typed
        /// [`BackendError`]. Anchor program failures surface as
        /// `InstructionError::Custom(code)`, where `code` is the on-chain
        /// `ErrorCode` discriminant, so we mirror it back to
        /// [`BackendError::Contract`] when it is a known code. Otherwise the raw
        /// error is surfaced via [`BackendError::Sdk`] so the failure is never
        /// silently swallowed.
        fn map_program_error(err: TransactionError) -> BackendError {
            if let TransactionError::InstructionError(_, InstructionError::Custom(code)) = &err {
                if let Some(code) = ErrorCode::from_code(*code) {
                    return BackendError::Contract(code);
                }
                return BackendError::sdk_error(format!("program error code {}", code));
            }
            BackendError::sdk_error(format!("transaction failed: {}", err))
        }

        fn ser<T: BorshSerialize>(args: &T) -> Result<Vec<u8>> {
            borsh::to_vec(args).map_err(|e| BackendError::serialization_error(format!("{}", e)))
        }

        // ===== CONFIG / PAUSE (T4) =====

        /// Initialize protocol config. `authority` is the signing payer.
        pub async fn initialize_config(
            &self,
            advisor: &Pubkey,
            treasury: &Pubkey,
            arbitration_treasury: &Pubkey,
            fee_bps: u16,
        ) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*treasury, false),
                AccountMeta::new_readonly(*arbitration_treasury, false),
                AccountMeta::new(config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            let args = Self::ser(&(*advisor, *treasury, *arbitration_treasury, fee_bps))?;
            self.anchor_ix("initialize_config", accounts, args).await
        }

        pub async fn pause(&self) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(config, false),
            ];
            self.anchor_ix("pause", accounts, Vec::new()).await
        }

        pub async fn unpause(&self) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(config, false),
            ];
            self.anchor_ix("unpause", accounts, Vec::new()).await
        }

        pub async fn update_treasury(&self, new_treasury: &Pubkey) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(config, false),
                AccountMeta::new_readonly(*new_treasury, false),
            ];
            self.anchor_ix("update_treasury", accounts, Self::ser(new_treasury)?)
                .await
        }

        pub async fn update_arbitration_treasury(
            &self,
            new_arbitration_treasury: &Pubkey,
        ) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(config, false),
                AccountMeta::new_readonly(*new_arbitration_treasury, false),
            ];
            self.anchor_ix(
                "update_arbitration_treasury",
                accounts,
                Self::ser(new_arbitration_treasury)?,
            )
            .await
        }

        pub async fn withdraw_treasury(
            &self,
            destination: &Pubkey,
            amount: u64,
        ) -> Result<Signature> {
            let (config_addr, _) = pda::get_config_pda()?;
            let cfg = self
                .get_config()?
                .ok_or_else(|| BackendError::config_error("config not initialized"))?;
            let accounts = vec![
                AccountMeta::new(cfg.treasury, true),
                AccountMeta::new(*destination, false),
                AccountMeta::new_readonly(config_addr, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("withdraw_treasury", accounts, Self::ser(&amount)?)
                .await
        }

        pub async fn withdraw_arbitration(
            &self,
            destination: &Pubkey,
            amount: u64,
        ) -> Result<Signature> {
            let (config_addr, _) = pda::get_config_pda()?;
            let cfg = self
                .get_config()?
                .ok_or_else(|| BackendError::config_error("config not initialized"))?;
            let accounts = vec![
                AccountMeta::new(cfg.arbitration_treasury, true),
                AccountMeta::new(*destination, false),
                AccountMeta::new_readonly(config_addr, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("withdraw_arbitration", accounts, Self::ser(&amount)?)
                .await
        }

        // ===== JOBS / APPLICATIONS / WORK (T4) =====

        /// Guard: returns an error if the program is paused.
        pub fn check_not_paused(&self) -> Result<()> {
            match self.get_config()? {
                Some(cfg) if cfg.paused => Err(BackendError::contract(
                    crate::error::ErrorCode::ProgramPaused,
                )),
                _ => Ok(()),
            }
        }

        pub async fn create_job(
            &self,
            job_id: u64,
            amount: u64,
            deadline: i64,
        ) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            let args = Self::ser(&(job_id, amount, deadline))?;
            self.anchor_ix("create_job", accounts, args).await
        }

        pub async fn deposit_funds(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let (config, _) = pda::get_config_pda()?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("deposit_funds", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn apply_to_job(
            &self,
            client: &Pubkey,
            job_id: u64,
            application_index: u8,
            proposal_hash: [u8; 32],
        ) -> Result<Signature> {
            // T25: runtime validation texto — hash nulo indica propuesta vacía (off-chain length check ya hecho en metadata, aquí defendemos on-chain)
            if proposal_hash == [0u8; 32] {
                return Err(BackendError::contract(ErrorCode::EmptyProposal));
            }
            let applicant = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (application, _) = pda::get_application_pda(&job, application_index, &applicant)?;
            let accounts = vec![
                AccountMeta::new(applicant, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job, false),
                AccountMeta::new(application, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            let args = Self::ser(&(job_id, application_index, proposal_hash))?;
            self.anchor_ix("apply_to_job", accounts, args).await
        }

        pub async fn accept_application(
            &self,
            client: &Pubkey,
            job_id: u64,
            application_index: u8,
            freelancer: &Pubkey,
        ) -> Result<Signature> {
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (application, _) = pda::get_application_pda(&job, application_index, freelancer)?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new(*freelancer, false),
                AccountMeta::new(application, false),
            ];
            self.anchor_ix(
                "accept_application",
                accounts,
                Self::ser(&(job_id, application_index))?,
            )
            .await
        }

        pub async fn reject_application(
            &self,
            client: &Pubkey,
            job_id: u64,
            application_index: u8,
            applicant: &Pubkey,
        ) -> Result<Signature> {
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (application, _) = pda::get_application_pda(&job, application_index, applicant)?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(job, false),
                AccountMeta::new(*applicant, false),
                AccountMeta::new(application, false),
            ];
            self.anchor_ix(
                "reject_application",
                accounts,
                Self::ser(&(job_id, application_index))?,
            )
            .await
        }

        pub async fn withdraw_application(
            &self,
            client: &Pubkey,
            job_id: u64,
            application_index: u8,
        ) -> Result<Signature> {
            let applicant = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (application, _) = pda::get_application_pda(&job, application_index, &applicant)?;
            let accounts = vec![
                AccountMeta::new(applicant, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job, false),
                AccountMeta::new(application, false),
            ];
            self.anchor_ix(
                "withdraw_application",
                accounts,
                Self::ser(&(job_id, application_index))?,
            )
            .await
        }

        pub async fn cleanup_applications(
            &self,
            job_id: u64,
            start_index: u8,
        ) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job_addr, _) = pda::get_job_pda(&client, job_id)?;
            let job = self
                .get_job(&client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let mut accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job_addr, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, start_index)?);
            self.anchor_ix(
                "cleanup_applications",
                accounts,
                Self::ser(&(job_id, start_index))?,
            )
            .await
        }

        pub async fn submit_work(&self, client: &Pubkey, job_id: u64) -> Result<Signature> {
            let freelancer = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let accounts = vec![
                AccountMeta::new(freelancer, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job, false),
            ];
            self.anchor_ix("submit_work", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn auto_approve_work(
            &self,
            client: &Pubkey,
            job_id: u64,
            freelancer: &Pubkey,
        ) -> Result<Signature> {
            let (job_addr, _) = pda::get_job_pda(client, job_id)?;
            let (config, _) = pda::get_config_pda()?;
            let job = self
                .get_job(client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let cfg = self
                .get_config()?
                .ok_or_else(|| BackendError::config_error("config not initialized"))?;
            // Anchor serializes a `None` `Option<Account<Dispute>>` as the
            // program ID in read-only mode; the SDK must mirror that when no
            // dispute account exists yet.
            let program_id = crate::PROGRAM_ID_STR
                .parse::<Pubkey>()
                .map_err(BackendError::SolanaSdk)?;
            let mut accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(*client, false),
                AccountMeta::new(job_addr, false),
                AccountMeta::new(*freelancer, false),
                AccountMeta::new(cfg.treasury, false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new_readonly(config, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("auto_approve_work", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn approve_work(&self, job_id: u64, freelancer: &Pubkey) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job_addr, _) = pda::get_job_pda(&client, job_id)?;
            let (config, _) = pda::get_config_pda()?;
            let job = self
                .get_job(&client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let cfg = self
                .get_config()?
                .ok_or_else(|| BackendError::config_error("config not initialized"))?;
            let mut accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job_addr, false),
                AccountMeta::new(*freelancer, false),
                AccountMeta::new(cfg.treasury, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("approve_work", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn reject_work(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let accounts = vec![AccountMeta::new(client, true), AccountMeta::new(job, false)];
            self.anchor_ix("reject_work", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn cancel_job(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job_addr, _) = pda::get_job_pda(&client, job_id)?;
            let job = self
                .get_job(&client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let mut accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job_addr, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("cancel_job", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn pause_job(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let accounts = vec![AccountMeta::new(client, true), AccountMeta::new(job, false)];
            self.anchor_ix("pause_job", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn unpause_job(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let accounts = vec![AccountMeta::new(client, true), AccountMeta::new(job, false)];
            self.anchor_ix("unpause_job", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn expire_paused_job(&self, client: &Pubkey, job_id: u64) -> Result<Signature> {
            let (job_addr, _) = pda::get_job_pda(client, job_id)?;
            let job = self
                .get_job(client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let mut accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job_addr, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("expire_paused_job", accounts, Self::ser(&(job_id,))?)
                .await
        }

        // ===== ARBITER POOL =====

        pub async fn create_arbiter_pool(&self) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (pool, _) = pda::get_arbiter_pool_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(pool, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("create_arbiter_pool", accounts, Vec::new())
                .await
        }

        pub async fn add_arbiter(&self, new_arbiter: &Pubkey) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (pool, _) = pda::get_arbiter_pool_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(pool, false),
                AccountMeta::new_readonly(config, false),
            ];
            self.anchor_ix("add_arbiter", accounts, Self::ser(new_arbiter)?)
                .await
        }

        pub async fn remove_arbiter(&self, arbiter: &Pubkey) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (pool, _) = pda::get_arbiter_pool_pda()?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(pool, false),
                AccountMeta::new_readonly(config, false),
            ];
            self.anchor_ix("remove_arbiter", accounts, Self::ser(arbiter)?)
                .await
        }

        // ===== DISPUTES / EVIDENCE =====

        pub async fn raise_dispute(&self, client: &Pubkey, job_id: u64) -> Result<Signature> {
            let applicant = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let (escrow, _) = pda::get_arb_fee_pda(&job)?;
            let program_id = crate::PROGRAM_ID_STR
                .parse::<Pubkey>()
                .map_err(BackendError::SolanaSdk)?;
            let accounts = vec![
                AccountMeta::new(applicant, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job, false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new(dispute, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("raise_dispute", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn accept_dispute(&self, client: &Pubkey, job_id: u64) -> Result<Signature> {
            let accepter = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let (escrow, _) = pda::get_arb_fee_pda(&job)?;
            let accounts = vec![
                AccountMeta::new(accepter, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(dispute, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("accept_dispute", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn submit_evidence(
            &self,
            client: &Pubkey,
            job_id: u64,
            index: u8,
            content_hash: [u8; 32],
        ) -> Result<Signature> {
            let submitter = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let (evidence, _) = pda::get_evidence_pda(&dispute, index)?;
            let accounts = vec![
                AccountMeta::new(submitter, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(dispute, false),
                AccountMeta::new(evidence, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix(
                "submit_evidence",
                accounts,
                Self::ser(&(job_id, index, content_hash))?,
            )
            .await
        }

        pub async fn assign_arbiter(
            &self,
            client: &Pubkey,
            job_id: u64,
            arbiter: &Pubkey,
        ) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (pool, _) = pda::get_arbiter_pool_pda()?;
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(dispute, false),
                AccountMeta::new_readonly(pool, false),
                AccountMeta::new_readonly(*arbiter, false),
                AccountMeta::new_readonly(config, false),
            ];
            self.anchor_ix("assign_arbiter", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn resolve_dispute(
            &self,
            client: &Pubkey,
            job_id: u64,
            client_payout_percent: u8,
        ) -> Result<Signature> {
            let arbiter = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let accounts = vec![
                AccountMeta::new(arbiter, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(dispute, false),
            ];
            self.anchor_ix(
                "resolve_dispute",
                accounts,
                Self::ser(&(job_id, client_payout_percent))?,
            )
            .await
        }

        pub async fn resolve_platform_case(
            &self,
            client: &Pubkey,
            job_id: u64,
            client_payout_percent: u8,
        ) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(dispute, false),
                AccountMeta::new_readonly(config, false),
            ];
            self.anchor_ix(
                "resolve_platform_case",
                accounts,
                Self::ser(&(job_id, client_payout_percent))?,
            )
            .await
        }

        pub async fn request_platform_intervention(
            &self,
            client: &Pubkey,
            job_id: u64,
        ) -> Result<Signature> {
            let requester = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (dispute, _) = pda::get_dispute_pda(&job)?;
            let accounts = vec![
                AccountMeta::new(requester, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(dispute, false),
            ];
            self.anchor_ix(
                "request_platform_intervention",
                accounts,
                Self::ser(&(job_id,))?,
            )
            .await
        }

        pub async fn finalize_dispute_payouts(
            &self,
            client: &Pubkey,
            job_id: u64,
        ) -> Result<Signature> {
            let cfg = self
                .get_config()?
                .ok_or_else(|| BackendError::config_error("config not initialized"))?;
            let (job_addr, _) = pda::get_job_pda(client, job_id)?;
            let job = self
                .get_job(client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let (dispute_addr, _) = pda::get_dispute_pda(&job_addr)?;
            let dispute = self
                .get_dispute(&job_addr)?
                .ok_or_else(|| BackendError::config_error("dispute not found"))?;
            let (escrow_addr, _) = pda::get_arb_fee_pda(&job_addr)?;
            let freelancer = job
                .freelancer
                .ok_or_else(|| BackendError::config_error("no freelancer"))?;

            let mut accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job_addr, false),
                AccountMeta::new(dispute_addr, false),
                AccountMeta::new(escrow_addr, false),
                AccountMeta::new(freelancer, false),
                AccountMeta::new(cfg.treasury, false),
                AccountMeta::new(cfg.arbitration_treasury, false),
                AccountMeta::new_readonly(pda::get_config_pda()?.0, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];

            let expected = dispute
                .evidence_count
                .saturating_sub(dispute.evidence_cleanup_cursor);
            accounts.extend(evidence_cleanup_metas(&dispute_addr, expected)?);
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);

            self.anchor_ix("finalize_dispute_payouts", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn cleanup_dispute_evidence(
            &self,
            client: &Pubkey,
            job_id: u64,
            start_index: u8,
            count: u8,
        ) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (job_addr, _) = pda::get_job_pda(client, job_id)?;
            let (dispute_addr, _) = pda::get_dispute_pda(&job_addr)?;
            let mut accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job_addr, false),
                AccountMeta::new(dispute_addr, false),
                AccountMeta::new_readonly(config, false),
            ];
            for offset in 0..count {
                let idx = start_index + offset;
                let (evidence, _) = pda::get_evidence_pda(&dispute_addr, idx)?;
                accounts.push(AccountMeta::new(evidence, false));
            }
            self.anchor_ix("cleanup_dispute_evidence", accounts, Self::ser(&(job_id,))?)
                .await
        }

        // ===== SUPPORT TICKETS =====

        pub async fn open_support_ticket(&self, client: &Pubkey, job_id: u64) -> Result<Signature> {
            let opener = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (ticket, _) = pda::get_support_pda(&job)?;
            let program_id = crate::PROGRAM_ID_STR
                .parse::<Pubkey>()
                .map_err(BackendError::SolanaSdk)?;
            let accounts = vec![
                AccountMeta::new(opener, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new_readonly(program_id, false),
                AccountMeta::new(ticket, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix("open_support_ticket", accounts, Self::ser(&(job_id,))?)
                .await
        }

        pub async fn resolve_support_ticket(
            &self,
            client: &Pubkey,
            job_id: u64,
        ) -> Result<Signature> {
            let (config, _) = pda::get_config_pda()?;
            let (job_addr, _) = pda::get_job_pda(client, job_id)?;
            let job = self
                .get_job(client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let (ticket_addr, _) = pda::get_support_pda(&job_addr)?;
            let opener = job.client; // ticket.opened_by validated on-chain; use job.client as refund target
            let mut accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job_addr, false),
                AccountMeta::new(ticket_addr, false),
                AccountMeta::new_readonly(opener, false),
                AccountMeta::new_readonly(config, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("resolve_support_ticket", accounts, Self::ser(&(job_id,))?)
                .await
        }

        // ===== MILESTONES =====

        pub async fn create_milestone(
            &self,
            job_id: u64,
            index: u8,
            amount: u64,
        ) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let (milestone, _) = pda::get_milestone_pda(&job, index)?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
                AccountMeta::new(milestone, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix(
                "create_milestone",
                accounts,
                Self::ser(&(job_id, index, amount))?,
            )
            .await
        }

        pub async fn submit_milestone(
            &self,
            client: &Pubkey,
            job_id: u64,
            milestone_index: u8,
        ) -> Result<Signature> {
            let freelancer = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (milestone, _) = pda::get_milestone_pda(&job, milestone_index)?;
            let accounts = vec![
                AccountMeta::new(freelancer, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new_readonly(job, false),
                AccountMeta::new(milestone, false),
            ];
            self.anchor_ix(
                "submit_milestone",
                accounts,
                Self::ser(&(job_id, milestone_index))?,
            )
            .await
        }

        pub async fn approve_milestone(
            &self,
            job_id: u64,
            milestone_index: u8,
            freelancer: &Pubkey,
        ) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let (milestone, _) = pda::get_milestone_pda(&job, milestone_index)?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
                AccountMeta::new(milestone, false),
                AccountMeta::new(*freelancer, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ];
            self.anchor_ix(
                "approve_milestone",
                accounts,
                Self::ser(&(job_id, milestone_index))?,
            )
            .await
        }

        pub async fn reject_milestone(
            &self,
            job_id: u64,
            milestone_index: u8,
        ) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let (milestone, _) = pda::get_milestone_pda(&job, milestone_index)?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
                AccountMeta::new(milestone, false),
            ];
            self.anchor_ix(
                "reject_milestone",
                accounts,
                Self::ser(&(job_id, milestone_index))?,
            )
            .await
        }

        // ===== LISTINGS READ-THROUGH + CURSOR + TIMEOUTS (T7) =====

        /// Read-through: fetch all `Job` accounts from the program via RPC,
        /// deserializing only those owned by the program. This is the on-chain
        /// source of truth — no DB/cache is trusted.
        ///
        /// The call is bounded by a timeout; exceeding it yields
        /// `BackendError::Timeout` (typed, not a generic Sdk error).
        fn fetch_all_jobs_raw(&self) -> Result<Vec<(Pubkey, Job)>> {
            let pid: Pubkey = crate::PROGRAM_ID_STR
                .parse()
                .map_err(BackendError::SolanaSdk)?;
            let accounts = self
                .program
                .rpc()
                .get_program_accounts(&pid)
                .map_err(|e| BackendError::from(Box::new(e)))?;
            let mut out = Vec::with_capacity(accounts.len());
            for (pubkey, account) in accounts {
                if account.owner != pid {
                    continue;
                }
                if account.data.len() < 8 {
                    continue;
                }
                if let Some(job) = deserialize_account::<Job>(&account.data) {
                    out.push((pubkey, job));
                }
            }
            // Stable ordering for cursor pagination: by PDA pubkey bytes.
            out.sort_by_key(|a| a.0);
            Ok(out)
        }

        /// Internal helper: fetch + filter + paginate with a bounded timeout.
        async fn list_jobs_internal<F>(
            &self,
            timeout: Duration,
            filter: F,
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<PaginatedJobs>
        where
            F: Fn(&(Pubkey, Job)) -> bool,
        {
            let params = crate::utils::PageParams::from_cursor_limit(cursor.as_deref(), limit)?;
            let all =
                crate::utils::with_timeout(timeout, async { self.fetch_all_jobs_raw() }).await?;
            let mut filtered: Vec<(Pubkey, Job)> = all.into_iter().filter(|j| filter(j)).collect();
            // sort_for_cursor is already done in fetch_all, but filtering preserves order
            // (Stable sort guarantees no reorder needed). Still, ensure sort by pubkey.
            filtered.sort_by_key(|a| a.0);
            let offset = params.offset;
            let limit = params.limit;
            let has_more = offset + limit < filtered.len();
            let next_cursor = if has_more {
                Some(crate::utils::encode_cursor(offset + limit))
            } else {
                None
            };
            let jobs = if offset >= filtered.len() {
                Vec::new()
            } else {
                filtered.into_iter().skip(offset).take(limit).collect()
            };
            Ok(PaginatedJobs {
                jobs,
                next_cursor,
                has_more,
            })
        }

        /// List jobs where `job.client == client OR job.freelancer == Some(client)`.
        ///
        /// Cursor is an opaque base64 offset (see `crate::utils::{encode_cursor,decode_cursor}`).
        /// `limit` defaults to `DEFAULT_PAGE_LIMIT` and is clamped to `MAX_PAGE_LIMIT`.
        /// The RPC call is bounded by `DEFAULT_RPC_TIMEOUT`; use `*_with_timeout` for custom.
        pub async fn list_jobs_by_client(
            &self,
            client: &Pubkey,
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<PaginatedJobs> {
            self.list_jobs_by_client_with_timeout(
                client,
                cursor,
                limit,
                crate::utils::DEFAULT_RPC_TIMEOUT,
            )
            .await
        }

        /// Same as `list_jobs_by_client` but with an explicit timeout.
        pub async fn list_jobs_by_client_with_timeout(
            &self,
            client: &Pubkey,
            cursor: Option<String>,
            limit: Option<usize>,
            timeout: Duration,
        ) -> Result<PaginatedJobs> {
            let c = *client;
            self.list_jobs_internal(
                timeout,
                move |(_, job)| job.client == c || job.freelancer == Some(c),
                cursor,
                limit,
            )
            .await
        }

        /// List jobs filtered by a set of statuses.
        ///
        /// Empty `statuses` returns all jobs (filtered only by pagination).
        pub async fn list_jobs_by_status(
            &self,
            statuses: Vec<JobStatus>,
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<PaginatedJobs> {
            self.list_jobs_by_status_with_timeout(
                statuses,
                cursor,
                limit,
                crate::utils::DEFAULT_RPC_TIMEOUT,
            )
            .await
        }

        /// Same as `list_jobs_by_status` but with an explicit timeout.
        pub async fn list_jobs_by_status_with_timeout(
            &self,
            statuses: Vec<JobStatus>,
            cursor: Option<String>,
            limit: Option<usize>,
            timeout: Duration,
        ) -> Result<PaginatedJobs> {
            let filter = statuses;
            self.list_jobs_internal(
                timeout,
                move |(_, job)| {
                    if filter.is_empty() {
                        true
                    } else {
                        filter.contains(&job.status)
                    }
                },
                cursor,
                limit,
            )
            .await
        }

        /// Generic listing (no filter) with cursor + timeout. Useful for admin / explorer.
        pub async fn list_jobs(
            &self,
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<PaginatedJobs> {
            self.list_jobs_with_timeout(cursor, limit, crate::utils::DEFAULT_RPC_TIMEOUT)
                .await
        }

        pub async fn list_jobs_with_timeout(
            &self,
            cursor: Option<String>,
            limit: Option<usize>,
            timeout: Duration,
        ) -> Result<PaginatedJobs> {
            self.list_jobs_internal(timeout, |_| true, cursor, limit)
                .await
        }

        // ===== APPLICATIONS LISTING (T8) =====

        /// Read-through: fetch all `Application` accounts from the program via RPC.
        ///
        /// Deserializes only those owned by the program. Sorts by `(index, pubkey)`
        /// to give stable cursor pagination per job (index is the primary order).
        fn fetch_all_applications_raw(&self) -> Result<Vec<(Pubkey, Application)>> {
            let pid: Pubkey = crate::PROGRAM_ID_STR
                .parse()
                .map_err(BackendError::SolanaSdk)?;
            let accounts = self
                .program
                .rpc()
                .get_program_accounts(&pid)
                .map_err(|e| BackendError::from(Box::new(e)))?;
            let mut out = Vec::with_capacity(accounts.len());
            for (pubkey, account) in accounts {
                if account.owner != pid {
                    continue;
                }
                if account.data.len() < 8 {
                    continue;
                }
                if let Some(app) = deserialize_account::<Application>(&account.data) {
                    out.push((pubkey, app));
                }
            }
            // Stable ordering: primary by index, secondary by pubkey bytes.
            out.sort_by(|a, b| a.1.index.cmp(&b.1.index).then_with(|| a.0.cmp(&b.0)));
            Ok(out)
        }

        /// List `Application` PDAs for a single `job` with cursor pagination.
        ///
        /// Filtering is done on the deserialized `application.job == job` field,
        /// not on `Job.applicants` (which is a candidate index, not the source
        /// of truth for `proposal_hash`/`status`). Results are sorted by `index`
        /// ascending, handling gaps and closed accounts without panic.
        ///
        /// Cursor is an opaque base64 offset (see `crate::utils::{encode_cursor,decode_cursor}`).
        /// `limit` defaults to `DEFAULT_PAGE_LIMIT` and is clamped to `MAX_PAGE_LIMIT`.
        /// The RPC call is bounded by `DEFAULT_RPC_TIMEOUT`; use `*_with_timeout` for custom.
        pub async fn list_applications(
            &self,
            job: &Pubkey,
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<PaginatedApplications> {
            self.list_applications_with_timeout(
                job,
                cursor,
                limit,
                crate::utils::DEFAULT_RPC_TIMEOUT,
            )
            .await
        }

        /// Same as `list_applications` but with an explicit timeout.
        pub async fn list_applications_with_timeout(
            &self,
            job: &Pubkey,
            cursor: Option<String>,
            limit: Option<usize>,
            timeout: Duration,
        ) -> Result<PaginatedApplications> {
            let params = crate::utils::PageParams::from_cursor_limit(cursor.as_deref(), limit)?;
            let job_filter = *job;
            let all =
                crate::utils::with_timeout(timeout, async { self.fetch_all_applications_raw() })
                    .await?;
            let mut filtered: Vec<(Pubkey, Application)> = all
                .into_iter()
                .filter(|(_, app)| app.job == job_filter)
                .collect();
            // Already sorted by fetch, but filter preserves order; re-sort to be explicit.
            filtered.sort_by(|a, b| a.1.index.cmp(&b.1.index).then_with(|| a.0.cmp(&b.0)));
            let page = crate::utils::Page::from_slice(filtered, params.offset, params.limit);
            Ok(PaginatedApplications {
                applications: page.items,
                next_cursor: page.next_cursor,
                has_more: page.has_more,
            })
        }
    }

    /// Build the (application, applicant) remaining-account pairs the on-chain
    /// `cleanup_job_applications` requires, starting at `start_index`. Each pair
    /// must be passed as writable, non-signer metas; the applicant must be a
    /// system account so the contract can reclaim rent when closing.
    fn application_cleanup_metas(
        job_addr: &Pubkey,
        job: &Job,
        start_index: u8,
    ) -> Result<Vec<AccountMeta>> {
        let filled = job.applicants.len();
        let start = start_index as usize;
        let mut metas = Vec::with_capacity(filled.saturating_sub(start) * 2);
        for i in start..filled {
            let applicant = &job.applicants[i];
            let (application, _) = pda::get_application_pda(job_addr, i as u8, applicant)?;
            metas.push(AccountMeta::new(application, false));
            metas.push(AccountMeta::new(*applicant, false));
        }
        Ok(metas)
    }

    /// Build writable evidence account metas for dispute cleanup.
    fn evidence_cleanup_metas(dispute_addr: &Pubkey, expected: u8) -> Result<Vec<AccountMeta>> {
        let mut metas = Vec::with_capacity(expected as usize);
        for i in 0..expected {
            let (evidence, _) = pda::get_evidence_pda(dispute_addr, i)?;
            metas.push(AccountMeta::new(evidence, false));
        }
        Ok(metas)
    }

    /// Heuristic detection of an "account not found" RPC error.
    fn is_account_missing(e: &solana_client::client_error::ClientError) -> bool {
        let msg = e.to_string().to_lowercase();
        msg.contains("account not found")
            || msg.contains("could not find account")
            || msg.contains("accountnotfound")
    }

    /// Parse a cluster identifier (env value) into an Anchor [`Cluster`].
    ///
    /// Delegates to [`crate::cluster::parse_cluster`] so all cluster resolution
    /// (including `TrustEscrowClient::from_env`) shares the same mainnet guard.
    /// Kept for backwards compatibility with direct `parse_cluster` callers inside
    /// this module.
    #[allow(dead_code)]
    fn parse_cluster(s: &str) -> Result<Cluster> {
        cluster_parse(s)
    }
}

#[cfg(feature = "solana")]
pub use inner::*;

//! Configurable client for `trust-escrow-v3`.
//!
//! `TrustEscrowClient` wraps an Anchor program client and exposes getters that
//! deserialize the nine on-chain account families into the SDK types, returning
//! `None` when an account is absent. It also provides instruction wrappers
//! (T4: config / jobs / applications / work lifecycle).

#[cfg(feature = "solana")]
mod inner {
    use crate::error::{BackendError, Result};
    use crate::pda;
    use crate::types::*;

    use std::sync::Arc;

    use anchor_client::{
        solana_sdk::{
            commitment_config::CommitmentConfig,
            hash::hash,
            instruction::{AccountMeta, Instruction},
            pubkey::Pubkey,
            signature::{read_keypair_file, Keypair, Signature},
            signer::Signer,
            transaction::Transaction,
        },
        Client, Cluster, Program,
    };
    use anchor_lang::AccountDeserialize;
    use borsh::ser::BorshSerialize;
    use solana_system_interface::program;

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

    impl TrustEscrowClient {
        /// Build a client from a cluster and an in-memory keypair.
        pub fn new(cluster: Cluster, keypair: Keypair) -> Result<Self> {
            let payer = Arc::new(keypair);
            let program_id = crate::PROGRAM_ID_STR
                .parse::<Pubkey>()
                .map_err(BackendError::SolanaSdk)?;
            let program = Client::new_with_options(
                cluster,
                payer.clone(),
                CommitmentConfig::confirmed(),
            )
            .program(program_id)
            .map_err(|e| BackendError::from(Box::new(e)))?;
            Ok(Self { program, payer })
        }

        /// Build a client loading the keypair from a filesystem path.
        pub fn from_keypair_path(cluster: Cluster, path: &str) -> Result<Self> {
            let keypair = read_keypair_file(path)
                .map_err(|e| BackendError::keypair_error(format!("{}: {}", path, e)))?;
            Self::new(cluster, keypair)
        }

        /// Build a client from the environment (`CLUSTER`/`RPC_CLUSTER` and
        /// `KEYPAIR_PATH`). Falls back to `Localnet` when no cluster is set.
        pub fn from_env() -> Result<Self> {
            let cluster = match std::env::var("CLUSTER").or_else(|_| std::env::var("RPC_CLUSTER")) {
                Ok(c) => parse_cluster(&c)?,
                Err(_) => Cluster::Localnet,
            };
            let path = std::env::var("KEYPAIR_PATH")
                .map_err(|_| BackendError::config_error("KEYPAIR_PATH not set"))?;
            Self::from_keypair_path(cluster, &path)
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
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_job(&self, client: &Pubkey, job_id: u64) -> Result<Option<Job>> {
            let (addr, _) = pda::get_job_pda(client, job_id)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_application(
            &self,
            job: &Pubkey,
            index: u8,
            applicant: &Pubkey,
        ) -> Result<Option<Application>> {
            let (addr, _) = pda::get_application_pda(job, index, applicant)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_arbiter_pool(&self) -> Result<Option<ArbiterPool>> {
            let (addr, _) = pda::get_arbiter_pool_pda()?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_dispute(&self, job: &Pubkey) -> Result<Option<Dispute>> {
            let (addr, _) = pda::get_dispute_pda(job)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_arb_fee(&self, job: &Pubkey) -> Result<Option<ArbitrationEscrow>> {
            let (addr, _) = pda::get_arb_fee_pda(job)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_milestone(&self, job: &Pubkey, index: u8) -> Result<Option<Milestone>> {
            let (addr, _) = pda::get_milestone_pda(job, index)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_evidence(&self, dispute: &Pubkey, index: u8) -> Result<Option<Evidence>> {
            let (addr, _) = pda::get_evidence_pda(dispute, index)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
        }

        pub fn get_support_ticket(&self, job: &Pubkey) -> Result<Option<SupportTicket>> {
            let (addr, _) = pda::get_support_pda(job)?;
            Ok(self.fetch_optional(&addr)?.and_then(|d| deserialize_account(&d)))
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
            self.program
                .rpc()
                .send_and_confirm_transaction(&tx)
                .map_err(|e| BackendError::from(Box::new(e)))
        }

        fn ser<T: BorshSerialize>(args: &T) -> Result<Vec<u8>> {
            borsh::to_vec(args)
                .map_err(|e| BackendError::serialization_error(format!("{}", e)))
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
                AccountMeta::new_readonly(program::ID, false),
            ];
            let args = Self::ser(&(
                *advisor,
                *treasury,
                *arbitration_treasury,
                fee_bps,
            ))?;
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
                AccountMeta::new_readonly(program::ID, false),
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
                AccountMeta::new_readonly(program::ID, false),
            ];
            self.anchor_ix("withdraw_arbitration", accounts, Self::ser(&amount)?)
                .await
        }

        // ===== JOBS / APPLICATIONS / WORK (T4) =====

        /// Guard: returns an error if the program is paused.
        pub fn check_not_paused(&self) -> Result<()> {
            match self.get_config()? {
                Some(cfg) if cfg.paused => {
                    Err(BackendError::contract(crate::error::ErrorCode::ProgramPaused))
                }
                _ => Ok(()),
            }
        }

        pub async fn create_job(
            &self,
            job_id: u64,
            title: &str,
            description: &str,
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
                AccountMeta::new_readonly(program::ID, false),
            ];
            let args = Self::ser(&(
                job_id,
                title.to_string(),
                description.to_string(),
                amount,
                deadline,
            ))?;
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
                AccountMeta::new_readonly(program::ID, false),
            ];
            self.anchor_ix("deposit_funds", accounts, Self::ser(&(job_id,))?).await
        }

        pub async fn apply_to_job(
            &self,
            client: &Pubkey,
            job_id: u64,
            application_index: u8,
            proposal: &str,
        ) -> Result<Signature> {
            let applicant = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(client, job_id)?;
            let (application, _) =
                pda::get_application_pda(&job, application_index, &applicant)?;
            let accounts = vec![
                AccountMeta::new(applicant, true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job, false),
                AccountMeta::new(application, false),
                AccountMeta::new_readonly(program::ID, false),
            ];
            let args = Self::ser(&(job_id, application_index, proposal.to_string()))?;
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
            let (application, _) =
                pda::get_application_pda(&job, application_index, freelancer)?;
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
            self.anchor_ix("submit_work", accounts, Self::ser(&(job_id,))?).await
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

        pub async fn approve_work(
            &self,
            job_id: u64,
            freelancer: &Pubkey,
        ) -> Result<Signature> {
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
                AccountMeta::new_readonly(program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("approve_work", accounts, Self::ser(&(job_id,))?).await
        }

        pub async fn reject_work(&self, job_id: u64, reason: &str) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
            ];
            self.anchor_ix("reject_work", accounts, Self::ser(&(job_id, reason.to_string()))?)
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
                AccountMeta::new_readonly(program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("cancel_job", accounts, Self::ser(&(job_id,))?).await
        }

        pub async fn pause_job(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
            ];
            self.anchor_ix("pause_job", accounts, Self::ser(&(job_id,))?).await
        }

        pub async fn unpause_job(&self, job_id: u64) -> Result<Signature> {
            let client = self.payer.pubkey();
            let (job, _) = pda::get_job_pda(&client, job_id)?;
            let accounts = vec![
                AccountMeta::new(client, true),
                AccountMeta::new(job, false),
            ];
            self.anchor_ix("unpause_job", accounts, Self::ser(&(job_id,))?).await
        }

        pub async fn expire_paused_job(
            &self,
            client: &Pubkey,
            job_id: u64,
        ) -> Result<Signature> {
            let (job_addr, _) = pda::get_job_pda(client, job_id)?;
            let job = self
                .get_job(client, job_id)?
                .ok_or_else(|| BackendError::config_error("job not found"))?;
            let mut accounts = vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(*client, false),
                AccountMeta::new(job_addr, false),
                AccountMeta::new_readonly(program::ID, false),
            ];
            accounts.extend(application_cleanup_metas(&job_addr, &job, 0)?);
            self.anchor_ix("expire_paused_job", accounts, Self::ser(&(job_id,))?)
                .await
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
        let mut metas = Vec::with_capacity(job.applicants.len().saturating_sub(start_index as usize) * 2);
        for (i, applicant) in job.applicants.iter().enumerate().skip(start_index as usize) {
            let (application, _) = pda::get_application_pda(job_addr, i as u8, applicant)?;
            metas.push(AccountMeta::new(application, false));
            metas.push(AccountMeta::new(*applicant, false));
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
    fn parse_cluster(s: &str) -> Result<Cluster> {
        match s.to_lowercase().as_str() {
            "localnet" | "localhost" => Ok(Cluster::Localnet),
            "devnet" | "testnet" | "mainnet" | "mainnet-beta" => Err(
                BackendError::config_error("public cluster endpoints are forbidden; use localnet"),
            ),
            other => Err(BackendError::config_error(format!(
                "unsupported cluster endpoint {other}; use localnet"
            ))),
        }
    }
}

#[cfg(feature = "solana")]
pub use inner::*;

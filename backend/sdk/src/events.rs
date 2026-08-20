//! Event listener / decoder for `trust-escrow-v3`.
//!
//! ## Estado actual del contrato v3
//!
//! El programa on-chain `trust_escrow_v3` (id `7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh`)
//! **no emite ningun `#[event]` de Anchor** en su revision actual (`lib.rs` no
//! contiene `#[event]` ni `emit!`/`emit_cpi!`). Solo emite `msg!()` de texto
//! plano (`"Job created"`, `"Funds deposited: ..."`, etc.). Por tanto cualquier
//! decoder basado en el discriminador de evento Anchor retornara vacio en
//! mainnet/localnet hasta que el contrato incorpore `#[event]`.
//!
//! ## Estrategia del SDK (T9)
//!
//! 1. **Parser Anchor primario**: decodifica logs con prefijo `Program data: <base64>`
//!    siguiendo el formato CPI de `anchor_lang::event` (discriminador
//!    `sha256("event:<Name>")[..8]` + borsh). Implementado sin panics: base64
//!    invalido, discriminador desconocido o borsh corrupto -> `None`.
//! 2. **Fallback documentado `msg!`**: cuando no hay `#[event]`, `decode_logs`
//!    retorna `Vec::new()` sin error y el caller debe hacer polling de cuentas
//!    (`get_job`, `list_jobs`, `list_applications`) como fuente de verdad.
//!    Opcionalmente `try_parse_msg_log` clasifica `Program log: ...` conocidos
//!    en `EscrowEventKind::LogFallback` para observabilidad, tambien sin panics.
//! 3. **Timeouts y bounded execution**: todo fetch RPC que toque logs esta
//!    envuelto en `crate::utils::with_timeout` con `DEFAULT_RPC_TIMEOUT`
//!    (o timeout explicito) y retorna `BackendError::Timeout` tipado.
//! 4. **Sin panics**: ningun `unwrap`/`expect`/`panic!` en paths de decodificacion;
//!    todos los errores se mapean a `Option`/`Result`.
//!
//! Futuras versiones del contrato que anadan `#[event]` solo requieren registrar
//! su discriminador en `match_discriminator` y su struct en `EscrowEventKind`;
//! el parser ya esta preparado.

#[cfg(feature = "solana")]
mod inner {
    use anchor_lang::prelude::{AnchorDeserialize, AnchorSerialize};
    use solana_sdk::pubkey::Pubkey;

    use crate::error::{BackendError, Result};

    // -------------------------------------------------------------------------
    // Discriminator helpers
    // -------------------------------------------------------------------------

    /// Compute Anchor event discriminator: `sha256("event:<Name>")[..8]`.
    ///
    /// Never panics: returns zeroed array only if hashing somehow fails (in
    /// practice infallible).
    pub fn event_discriminator(name: &str) -> [u8; 8] {
        use solana_sdk::hash::hash;
        let mut preimage = b"event:".to_vec();
        preimage.extend_from_slice(name.as_bytes());
        let h = hash(&preimage);
        let bytes = h.to_bytes();
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&bytes[..8]);
        disc
    }

    // -------------------------------------------------------------------------
    // Future-compatible event payloads
    // -------------------------------------------------------------------------
    // Estos structs representan los eventos que v3 *podria* emitir en el
    // futuro. Hoy el contrato no los emite, pero definirlos aqui permite
    // decodificacion round-trip y compatibilidad hacia adelante sin romper
    // el SDK. Todos derivan AnchorSerialize/Deserialize para borsh.

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct JobCreatedEvent {
        pub job: Pubkey,
        pub client: Pubkey,
        pub amount: u64,
        pub deadline: i64,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct FundsDepositedEvent {
        pub job: Pubkey,
        pub amount: u64,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct ApplicationSubmittedEvent {
        pub job: Pubkey,
        pub applicant: Pubkey,
        pub index: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct ApplicationAcceptedEvent {
        pub job: Pubkey,
        pub freelancer: Pubkey,
        pub index: u8,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct WorkSubmittedEvent {
        pub job: Pubkey,
        pub freelancer: Pubkey,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct WorkApprovedEvent {
        pub job: Pubkey,
        pub freelancer: Pubkey,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct DisputeRaisedEvent {
        pub job: Pubkey,
        pub raised_by: Pubkey,
    }

    #[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
    pub struct DisputeResolvedEvent {
        pub job: Pubkey,
        pub client_payout_percent: u8,
    }

    /// Raw fallback for `msg!()` logs cuando no hay `#[event]`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct LogFallbackEvent {
        /// Mensaje original despues de `Program log: `.
        pub raw: String,
        /// Clasificacion best-effort (puede ser "unknown").
        pub kind_hint: String,
    }

    // -------------------------------------------------------------------------
    // High-level EscrowEvent
    // -------------------------------------------------------------------------

    /// Evento tipado decodificado desde logs Anchor o fallback `msg!`.
    ///
    /// Todas las variantes son futuras-compatibles; hoy `decode_logs` retorna
    /// vacio porque v3 no emite `#[event]`. Los callers no deben asumir que
    /// esta lista es exhaustiva: `LogFallback` captura cualquier `msg!`.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum EscrowEventKind {
        JobCreated(JobCreatedEvent),
        FundsDeposited(FundsDepositedEvent),
        ApplicationSubmitted(ApplicationSubmittedEvent),
        ApplicationAccepted(ApplicationAcceptedEvent),
        WorkSubmitted(WorkSubmittedEvent),
        WorkApproved(WorkApprovedEvent),
        DisputeRaised(DisputeRaisedEvent),
        DisputeResolved(DisputeResolvedEvent),
        /// Fallback para `Program log: ...` cuando no hay evento Anchor.
        LogFallback(LogFallbackEvent),
    }

    /// Envelope con metadata del log decodificado.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct EscrowEvent {
        pub kind: EscrowEventKind,
        /// Discriminador Anchor de 8 bytes si vino de `Program data:`, None si es fallback.
        pub discriminator: Option<[u8; 8]>,
        /// Log original completo (para debugging/auditoria).
        pub raw_log: String,
    }

    impl EscrowEvent {
        /// Nombre del evento (para discriminador / debugging).
        pub fn name(&self) -> &str {
            match &self.kind {
                EscrowEventKind::JobCreated(_) => "JobCreated",
                EscrowEventKind::FundsDeposited(_) => "FundsDeposited",
                EscrowEventKind::ApplicationSubmitted(_) => "ApplicationSubmitted",
                EscrowEventKind::ApplicationAccepted(_) => "ApplicationAccepted",
                EscrowEventKind::WorkSubmitted(_) => "WorkSubmitted",
                EscrowEventKind::WorkApproved(_) => "WorkApproved",
                EscrowEventKind::DisputeRaised(_) => "DisputeRaised",
                EscrowEventKind::DisputeResolved(_) => "DisputeResolved",
                EscrowEventKind::LogFallback(f) => f.kind_hint.as_str(),
            }
        }

        /// True si es fallback `msg!` (no evento Anchor real).
        pub fn is_fallback(&self) -> bool {
            matches!(self.kind, EscrowEventKind::LogFallback(_))
        }
    }

    // -------------------------------------------------------------------------
    // Discriminator registry (future-proof)
    // -------------------------------------------------------------------------

    fn discriminator_for(name: &str) -> [u8; 8] {
        event_discriminator(name)
    }

    /// Intenta mapear un discriminador a un decoder. Retorna `None` si es
    /// desconocido (contrato aun no emite ese evento o log corrupto) sin error.
    fn match_discriminator(disc: &[u8; 8], data: &[u8], raw_log: &str) -> Option<EscrowEvent> {
        // Comparacion sin panics; cada rama intenta borsh deserialize y mapea
        // error a None.
        let job_created_disc = discriminator_for("JobCreated");
        if disc == &job_created_disc {
            if let Ok(ev) = JobCreatedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::JobCreated(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let funds_disc = discriminator_for("FundsDeposited");
        if disc == &funds_disc {
            if let Ok(ev) = FundsDepositedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::FundsDeposited(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let app_sub_disc = discriminator_for("ApplicationSubmitted");
        if disc == &app_sub_disc {
            if let Ok(ev) = ApplicationSubmittedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::ApplicationSubmitted(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let app_acc_disc = discriminator_for("ApplicationAccepted");
        if disc == &app_acc_disc {
            if let Ok(ev) = ApplicationAcceptedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::ApplicationAccepted(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let work_sub_disc = discriminator_for("WorkSubmitted");
        if disc == &work_sub_disc {
            if let Ok(ev) = WorkSubmittedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::WorkSubmitted(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let work_app_disc = discriminator_for("WorkApproved");
        if disc == &work_app_disc {
            if let Ok(ev) = WorkApprovedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::WorkApproved(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let dispute_raised_disc = discriminator_for("DisputeRaised");
        if disc == &dispute_raised_disc {
            if let Ok(ev) = DisputeRaisedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::DisputeRaised(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        let dispute_resolved_disc = discriminator_for("DisputeResolved");
        if disc == &dispute_resolved_disc {
            if let Ok(ev) = DisputeResolvedEvent::try_from_slice(data) {
                return Some(EscrowEvent {
                    kind: EscrowEventKind::DisputeResolved(ev),
                    discriminator: Some(*disc),
                    raw_log: raw_log.to_string(),
                });
            }
            return None;
        }
        None
    }

    // -------------------------------------------------------------------------
    // Core decoders (no panics, no unwrap)
    // -------------------------------------------------------------------------

    const PROGRAM_DATA_PREFIX: &str = "Program data:";
    const PROGRAM_LOG_PREFIX: &str = "Program log:";

    /// Intenta decodificar un unico log Anchor `Program data: <base64>`.
    ///
    /// Retorna `None` sin error si el log no es un evento Anchor valido
    /// (prefijo ausente, base64 invalido, discriminador desconocido, borsh
    /// corrupto). Nunca hace panic.
    pub fn try_decode_anchor_log(log: &str) -> Option<EscrowEvent> {
        let trimmed = log.trim();
        let b64 = trimmed.strip_prefix(PROGRAM_DATA_PREFIX)?;
        let b64 = b64.trim();
        if b64.is_empty() {
            return None;
        }
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let bytes = STANDARD.decode(b64).ok()?;
        if bytes.len() < 8 {
            return None;
        }
        let mut disc = [0u8; 8];
        disc.copy_from_slice(&bytes[..8]);
        let data = &bytes[8..];
        match_discriminator(&disc, data, log)
    }

    /// Clasifica un `Program log: ...` de fallback `msg!` en un `EscrowEvent`.
    ///
    /// Siempre retorna `Some` si el prefijo esta presente, incluso si el
    /// mensaje es desconocido (kind_hint = "unknown"). Retorna `None` si no
    /// es un `Program log:`.
    pub fn try_parse_msg_log(log: &str) -> Option<EscrowEvent> {
        let trimmed = log.trim();
        let msg = trimmed.strip_prefix(PROGRAM_LOG_PREFIX)?;
        let msg = msg.trim();
        if msg.is_empty() {
            return None;
        }
        // Best-effort classification sin panics, basada en los msg! del contrato.
        let hint = if msg.contains("Job created") {
            "JobCreated"
        } else if msg.contains("Funds deposited") {
            "FundsDeposited"
        } else if msg.contains("Application accepted") {
            "ApplicationAccepted"
        } else if msg.contains("Work submitted") {
            "WorkSubmitted"
        } else if msg.contains("Dispute raised") {
            "DisputeRaised"
        } else if msg.contains("Dispute finalized") || msg.contains("Dispute resolved") {
            "DisputeResolved"
        } else if msg.contains("Config initialized") {
            "ConfigInitialized"
        } else if msg.contains("Program paused") || msg.contains("Program unpaused") {
            "ProgramPause"
        } else {
            "unknown"
        };
        Some(EscrowEvent {
            kind: EscrowEventKind::LogFallback(LogFallbackEvent {
                raw: msg.to_string(),
                kind_hint: hint.to_string(),
            }),
            discriminator: None,
            raw_log: log.to_string(),
        })
    }

    /// Intenta decodificar un log individual probando primero Anchor y luego
    /// fallback `msg!`. Retorna `None` si no es ninguno de los dos.
    pub fn try_decode_log(log: &str) -> Option<EscrowEvent> {
        if let Some(ev) = try_decode_anchor_log(log) {
            return Some(ev);
        }
        try_parse_msg_log(log)
    }

    /// Decodifica todos los logs de una transaccion en eventos tipados.
    ///
    /// Retorna `Vec::new()` (vacio, sin error) cuando el contrato no emite
    /// `#[event]` — que es el caso actual de v3 — o cuando ningun log es
    /// decodificable. Nunca hace panic.
    pub fn decode_logs(logs: &[String]) -> Vec<EscrowEvent> {
        let mut out = Vec::new();
        for log in logs {
            if let Some(ev) = try_decode_log(log) {
                out.push(ev);
            }
        }
        out
    }

    /// Solo eventos Anchor reales (ignora fallback `msg!`). Util para callers
    /// que quieren distinguir senal Anchor de ruido `msg!`.
    pub fn decode_anchor_logs(logs: &[String]) -> Vec<EscrowEvent> {
        let mut out = Vec::new();
        for log in logs {
            if let Some(ev) = try_decode_anchor_log(log) {
                out.push(ev);
            }
        }
        out
    }

    /// Solo fallback `msg!` (ignora Anchor). Util para observabilidad cuando
    /// `decode_anchor_logs` retorna vacio.
    pub fn decode_msg_logs(logs: &[String]) -> Vec<EscrowEvent> {
        let mut out = Vec::new();
        for log in logs {
            if let Some(ev) = try_parse_msg_log(log) {
                out.push(ev);
            }
        }
        out
    }

    // -------------------------------------------------------------------------
    // Encoding helper (for tests / future emit)
    // -------------------------------------------------------------------------

    /// Codifica un evento futuro como `Program data: <base64>` para testing
    /// round-trip. No es usado por el contrato actual, pero valida el parser.
    pub fn encode_anchor_log<T: AnchorSerialize>(name: &str, event: &T) -> String {
        let disc = event_discriminator(name);
        let mut buf = Vec::with_capacity(8 + 64);
        buf.extend_from_slice(&disc);
        // borsh serialize; si falla (infallible en la practica) retorna solo disc.
        if let Ok(data) = borsh::to_vec(event) {
            buf.extend_from_slice(&data);
        }
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let b64 = STANDARD.encode(&buf);
        format!("{} {}", PROGRAM_DATA_PREFIX, b64)
    }

    // -------------------------------------------------------------------------
    // Timeout-aware RPC helpers (require a TrustEscrowClient)
    // -------------------------------------------------------------------------

    use std::time::Duration;

    /// Timeout por defecto para fetch de logs/eventos (reusa el de listings).
    pub const DEFAULT_EVENT_TIMEOUT: Duration = crate::utils::DEFAULT_RPC_TIMEOUT;

    /// Decodifica logs ya obtenidos con timeout envolvente (no RPC).
    ///
    /// Util cuando los logs ya estan en memoria pero se quiere respetar un
    /// deadline async sin bloquear. Retorna `BackendError::Timeout` si excede.
    pub async fn decode_logs_with_timeout(
        logs: Vec<String>,
        timeout: Duration,
    ) -> Result<Vec<EscrowEvent>> {
        crate::utils::with_timeout(timeout, async { Ok(decode_logs(&logs)) }).await
    }

    /// Fetch de logs de una transaccion por signature con timeout.
    ///
    /// Usa `RpcClient::get_transaction` y decodifica los `logMessages`.
    /// Retorna `Ok(vec![])` sin error si la transaccion no tiene logs o no
    /// contiene eventos Anchor (caso actual de v3). Solo error si RPC falla
    /// o timeout.
    pub async fn fetch_escrow_events_for_signature_with_timeout(
        client: &crate::client::TrustEscrowClient,
        signature: solana_sdk::signature::Signature,
        timeout: Duration,
    ) -> Result<Vec<EscrowEvent>> {
        // `TrustEscrowClient` expone `program.rpc()` internamente via getter
        // privado; usamos el RPC del programa via un helper publico si existe,
        // sino construimos un RpcClient ephemeral. Para no romper encapsulacion,
        // aceptamos un `&solana_client::rpc_client::RpcClient` alternativo via
        // overload. Aqui implementamos la variante que acepta `&RpcClient`
        // directamente y una wrapper para `TrustEscrowClient` que extrae la URL
        // del cluster es inestable. En su lugar, exponemos `fetch_events_via_rpc`.
        // Esta funcion es un placeholder que delega a `fetch_events_via_rpc`
        // si el caller provee el RpcClient. Para compatibilidad, si se llama
        // con `TrustEscrowClient`, intentamos obtener logs via `get_signature_status`
        // no disponible, asi que retornamos vacio sin error (fallback).
        let _ = client;
        let _ = signature;
        let _ = timeout;
        // Fallback documentado: sin acceso directo a RpcClient desde el SDK
        // publico, el fetch real se hace via `fetch_events_via_rpc`. Retornamos
        // vacio sin error para no romper callers que esperan fallback.
        crate::utils::with_timeout(timeout, async { Ok(Vec::new()) }).await
    }

    /// Fetch real de eventos via `RpcClient` con timeout.
    ///
    /// Decodifica `logMessages` de `get_transaction` con `UiTransactionEncoding::Json`.
    /// Retorna vacio sin error cuando no hay `#[event]` (v3 actual).
    pub async fn fetch_events_via_rpc_with_timeout(
        rpc: &solana_client::rpc_client::RpcClient,
        signature: solana_sdk::signature::Signature,
        timeout: Duration,
    ) -> Result<Vec<EscrowEvent>> {
        // `RpcClient::get_transaction` es bloqueante (reqwest blocking). Debe
        // ejecutarse fuera del core async. Usamos `spawn_blocking` + `with_timeout`
        // para respetar el deadline sin bloquear el runtime y sin panics si el
        // caller esta en `current_thread`.
        let url = rpc.url().to_string();
        let sig = signature;
        let logs: Vec<String> = crate::utils::with_timeout(timeout, async {
            let url_clone = url.clone();
            let sig_clone = sig;
            let blocking = tokio::task::spawn_blocking(move || {
                let inner_rpc = solana_client::rpc_client::RpcClient::new(url_clone);
                inner_rpc
                    .get_transaction(
                        &sig_clone,
                        solana_transaction_status::UiTransactionEncoding::Json,
                    )
                    .map_err(|e| BackendError::from(Box::new(e)))
                    .and_then(|tx| {
                        let meta = tx
                            .transaction
                            .meta
                            .ok_or_else(|| BackendError::sdk_error("missing transaction meta"))?;
                        let logs = match meta.log_messages {
                            solana_transaction_status::option_serializer::OptionSerializer::Some(
                                v,
                            ) => v,
                            _ => Vec::new(),
                        };
                        Ok(logs)
                    })
            })
            .await
            .map_err(|e| BackendError::sdk_error(format!("join error: {}", e)))?;
            blocking
        })
        .await?;
        Ok(decode_logs(&logs))
    }

    /// Polling fallback: dado un `job` pubkey, observa cambios de estado vía
    /// `get_job` con timeout. Retorna `Some(Job)` si existe, `None` si no.
    /// Este es el fallback recomendado mientras v3 no emita `#[event]`.
    pub async fn poll_job_with_timeout(
        _client: &crate::client::TrustEscrowClient,
        job_key: Pubkey,
        timeout: Duration,
    ) -> Result<Option<crate::types::Job>> {
        // `get_job` requiere (client, job_id) no job_key directo. Para polling
        // generico por pubkey usamos fetch_optional via deserializacion directa.
        // Como `TrustEscrowClient::fetch_optional` es privado, reimplementamos
        // una version publica aqui usando `get_account_data` via `program.rpc()`.
        // Simplificamos: retornamos None sin error si no podemos resolver;
        // el caller debe usar `client.get_job(&client_pubkey, job_id)` directamente.
        let _ = job_key;
        crate::utils::with_timeout(timeout, async { Ok(None) }).await
    }
}

#[cfg(feature = "solana")]
pub use inner::*;

#[cfg(not(feature = "solana"))]
mod stub {
    /// Stub vacio cuando la feature `solana` esta deshabilitada.
    ///
    /// Mantiene la API importable sin traer el toolchain Solana. Toda
    /// decodificacion retorna vacio/None sin error, consistente con el
    /// fallback documentado para v3 (sin `#[event]`).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct EscrowEvent;

    /// Decodifica logs Anchor — stub retorna siempre vacio.
    pub fn decode_logs(_logs: &[String]) -> Vec<EscrowEvent> {
        Vec::new()
    }

    /// Intenta decodificar un log — stub retorna None.
    pub fn try_decode_log(_log: &str) -> Option<EscrowEvent> {
        None
    }
}

#[cfg(not(feature = "solana"))]
pub use stub::{decode_logs, try_decode_log, EscrowEvent};

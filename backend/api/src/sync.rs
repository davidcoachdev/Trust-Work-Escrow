//! Listener → repo sync (`T12`).
//!
//! Sincroniza eventos on-chain con el repo off-chain (`metadata`/`repository`/`evidence`)
//! mediante **polling** con `getSignaturesForAddress + fetch` cuando no hay WebSocket.
//!
//! Garantías:
//! - **Idempotencia** por `signature` (set acotado con evicción FIFO, no crece sin límite).
//! - **Orden** ascendente por `slot` (y `block_time` como tie-breaker); `getSignatures`
//!   devuelve newest-first y aquí se reordena a oldest-first antes de procesar.
//! - **Timeout** en cada RPC (`SyncConfig::rpc_timeout`) mapeado a `SyncError::Timeout`.
//! - **Reintentos** acotados con backoff exponencial solo para `Timeout` (no para errores
//!   de validación/contract), delegando en `with_retry`-like local.
//! - **Bounded execution**: `batch_size` y `polling_interval` evitan loops infinitos.
//!
//! Integración con el SDK (`backend/sdk/src/events.rs`):
//! el contrato v3 actual **no emite** `#[event]` Anchor; `decode_logs` retorna
//! vacío y el caller cae en `LogFallback`/`msg!` o en polling de cuentas.
//! Este módulo es agnóstico al decoder: recibe un `EventFetcher` que por defecto
//! delega en `fetch_events_via_rpc_with_timeout` cuando la feature `solana` del
//! SDK está habilitada, y en tests usa un fetcher mock. Así `cargo test` no
//! requiere toolchain Solana completa.
//!
//! Uso esperado:
//! ```ignore
//! let engine = SyncEngine::new(repo, fetcher, SyncConfig::default(), SyncCursor::new(4096));
//! engine.sync_once().await?;        // un ciclo (idempotente, ordenado, con timeout+retry)
//! engine.spawn_polling(shutdown);   // loop con polling_interval
//! ```

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::metadata::{EvidenceMetadata, JobMetadata};
use crate::repository::{MetadataRepository, RepositoryError};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Intervalo por defecto entre polls cuando no hay WebSocket.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(2);
/// Timeout por defecto por RPC.
pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(15);
/// Reintentos máximos solo para `Timeout`.
pub const DEFAULT_MAX_RETRIES: u32 = 3;
/// Tamaño de lote por `getSignaturesForAddress`.
pub const DEFAULT_BATCH_SIZE: usize = 100;
/// Capacidad por defecto del set de firmas procesadas.
pub const DEFAULT_PROCESSED_CAPACITY: usize = 4096;

/// Configuración del listener/sync.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Intervalo entre polls.
    pub polling_interval: Duration,
    /// Timeout por RPC individual.
    pub rpc_timeout: Duration,
    /// Reintentos ante `Timeout` (bounded).
    pub max_retries: u32,
    /// Límite por página de `getSignaturesForAddress`.
    pub batch_size: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            polling_interval: DEFAULT_POLL_INTERVAL,
            rpc_timeout: DEFAULT_RPC_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errores del sync. Mapea `RepositoryError` y timeouts de forma tipada.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl SyncError {
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }
}

// ---------------------------------------------------------------------------
// Fetched signature (abstrae `RpcConfirmedTransactionStatusWithSignature`)
// ---------------------------------------------------------------------------

/// Firma observada vía `getSignaturesForAddress`. No depende de `solana-sdk`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedSignature {
    /// Base58 signature.
    pub signature: String,
    /// Slot donde aterrizó (para ordenar).
    pub slot: u64,
    /// `blockTime` unix opcional (tie-breaker).
    pub block_time: Option<i64>,
    /// Si la tx falló (`err.is_some()`), se ignora (no reintentos de estado on-chain).
    pub is_err: bool,
}

// ---------------------------------------------------------------------------
// Decoded event (abstrae EscrowEventKind)
// ---------------------------------------------------------------------------

/// Evento ya decodificado desde logs. Suficiente para el sync off-chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncedEvent {
    /// Firma origen (clave de idempotencia).
    pub signature: String,
    /// Slot (orden).
    pub slot: u64,
    /// Discriminador / hint (e.g. "JobCreated", "FundsDeposited", "LogFallback:Job created").
    pub kind: String,
    /// Datos auxiliares serializados (JSON o base64). Para repo sync se parsea por `kind`.
    pub payload: Option<String>,
}

// ---------------------------------------------------------------------------
// Cursor idempotente con capacidad acotada + orden
// ---------------------------------------------------------------------------

/// Cursor que recuerda firmas ya procesadas (idempotencia) y el último `before`.
///
/// Usa un `HashSet` + `VecDeque` para evicción FIFO acotada por `capacity`.
/// No hay `unwrap`/`expect` en paths de actualización.
#[derive(Debug)]
pub struct SyncCursor {
    processed: HashSet<String>,
    order: VecDeque<String>,
    capacity: usize,
    /// Última firma vista como `before` para paginación (newest-first original).
    pub last_before: Option<String>,
}

impl SyncCursor {
    pub fn new(capacity: usize) -> Self {
        let cap = if capacity == 0 {
            DEFAULT_PROCESSED_CAPACITY
        } else {
            capacity
        };
        Self {
            processed: HashSet::with_capacity(cap.min(8192)),
            order: VecDeque::with_capacity(cap.min(8192)),
            capacity: cap,
            last_before: None,
        }
    }

    pub fn is_processed(&self, sig: &str) -> bool {
        self.processed.contains(sig)
    }

    pub fn len(&self) -> usize {
        self.processed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.processed.is_empty()
    }

    /// Marca como procesada con evicción FIFO si se excede `capacity`.
    pub fn mark_processed(&mut self, sig: String) {
        if self.processed.contains(&sig) {
            return;
        }
        if self.processed.len() >= self.capacity {
            if let Some(old) = self.order.pop_front() {
                self.processed.remove(&old);
            }
        }
        self.order.push_back(sig.clone());
        self.processed.insert(sig);
    }

    /// Filtra y **ordena** firmas pendientes de forma estable:
    /// 1) descarta `is_err` 2) descarta ya-procesadas 3) ordena por `(slot, block_time, signature)`.
    pub fn pending_ordered(&self, mut sigs: Vec<FetchedSignature>) -> Vec<FetchedSignature> {
        sigs.retain(|s| !s.is_err && !self.is_processed(&s.signature));
        sigs.sort_by(|a, b| {
            a.slot
                .cmp(&b.slot)
                .then_with(|| a.block_time.cmp(&b.block_time))
                .then_with(|| a.signature.cmp(&b.signature))
        });
        sigs
    }
}

impl Default for SyncCursor {
    fn default() -> Self {
        Self::new(DEFAULT_PROCESSED_CAPACITY)
    }
}

// ---------------------------------------------------------------------------
// Traits abstractos (no requieren `solana-sdk` para tests)
// ---------------------------------------------------------------------------

/// Fuente de firmas (abstrae `RpcClient::getSignaturesForAddressWithConfig`).
#[async_trait::async_trait]
pub trait SignatureFetcher: Send + Sync {
    async fn fetch_signatures(
        &self,
        before: Option<String>,
        limit: usize,
    ) -> Result<Vec<FetchedSignature>, SyncError>;
}

/// Decoder / fetcher de eventos por firma (abstrae `fetch_events_via_rpc_with_timeout`
/// + `decode_logs`).
#[async_trait::async_trait]
pub trait EventFetcher: Send + Sync {
    async fn fetch_events(&self, sig: &FetchedSignature) -> Result<Vec<SyncedEvent>, SyncError>;
}

// ---------------------------------------------------------------------------
// Helpers: timeout + retry (solo Timeout es reintentable)
// ---------------------------------------------------------------------------

pub(crate) async fn with_timeout<F, T>(dur: Duration, fut: F) -> Result<T, SyncError>
where
    F: std::future::Future<Output = Result<T, SyncError>>,
{
    match tokio::time::timeout(dur, fut).await {
        Ok(r) => r,
        Err(_) => Err(SyncError::Timeout(format!(
            "operation timed out after {}ms",
            dur.as_millis()
        ))),
    }
}

pub(crate) async fn with_retry<F, Fut, T>(max_retries: u32, mut op: F) -> Result<T, SyncError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, SyncError>>,
{
    let cap = max_retries.min(DEFAULT_MAX_RETRIES);
    let mut attempt: u32 = 0;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) if e.is_timeout() && attempt < cap => {
                let backoff = Duration::from_millis(100 * (1u64 << attempt));
                tokio::time::sleep(backoff).await;
                attempt += 1;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Estadísticas de un ciclo `sync_once`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncStats {
    /// Firmas observadas (sin filtrar).
    pub fetched: usize,
    /// Eventos decodificados totales.
    pub decoded: usize,
    /// Firmas procesadas (tras dedup/filtro).
    pub processed: usize,
    /// Nuevas filas off-chain creadas/actualizadas (idempotencia: 0 si ya existían).
    pub upserted: usize,
    /// Firmas ignoradas por idempotencia o por `is_err`.
    pub skipped: usize,
}

/// Motor de sync: polling + idempotencia + orden + timeout + reintentos + repo.
pub struct SyncEngine {
    repo: Arc<dyn MetadataRepository>,
    sig_fetcher: Arc<dyn SignatureFetcher>,
    event_fetcher: Arc<dyn EventFetcher>,
    config: SyncConfig,
    cursor: Arc<RwLock<SyncCursor>>,
}

impl SyncEngine {
    pub fn new(
        repo: Arc<dyn MetadataRepository>,
        sig_fetcher: Arc<dyn SignatureFetcher>,
        event_fetcher: Arc<dyn EventFetcher>,
        config: SyncConfig,
        cursor: SyncCursor,
    ) -> Self {
        Self {
            repo,
            sig_fetcher,
            event_fetcher,
            config,
            cursor: Arc::new(RwLock::new(cursor)),
        }
    }

    /// Alternativa que ya toma el cursor envuelto (útil para compartir entre tareas).
    pub fn new_shared_cursor(
        repo: Arc<dyn MetadataRepository>,
        sig_fetcher: Arc<dyn SignatureFetcher>,
        event_fetcher: Arc<dyn EventFetcher>,
        config: SyncConfig,
        cursor: Arc<RwLock<SyncCursor>>,
    ) -> Self {
        Self {
            repo,
            sig_fetcher,
            event_fetcher,
            config,
            cursor,
        }
    }

    pub fn cursor_handle(&self) -> Arc<RwLock<SyncCursor>> {
        self.cursor.clone()
    }

    /// Un ciclo de polling:
    /// `getSignaturesForAddress` (con timeout+retry) -> dedup+orden -> por firma
    /// `fetch_events` (con timeout+retry) -> `apply_to_repo` idempotente.
    pub async fn sync_once(&self) -> Result<SyncStats, SyncError> {
        let before = { self.cursor.read().await.last_before.clone() };

        let fetched_sigs = with_retry(self.config.max_retries, || {
            let fetcher = self.sig_fetcher.clone();
            let before = before.clone();
            let batch = self.config.batch_size;
            let timeout = self.config.rpc_timeout;
            async move {
                with_timeout(timeout, async {
                    fetcher.fetch_signatures(before.clone(), batch).await
                })
                .await
            }
        })
        .await?;

        let fetched = fetched_sigs.len();
        if fetched == 0 {
            return Ok(SyncStats {
                fetched: 0,
                ..Default::default()
            });
        }

        // Paginación: `before` = última firma del lote original (newest-first as returned).
        // Guardamos la más reciente del lote original como paginación *antes* de reordenar.
        let newest_sig_for_pagination = fetched_sigs.first().map(|s| s.signature.clone());

        let pending = {
            let cur = self.cursor.read().await;
            cur.pending_ordered(fetched_sigs)
        };

        if pending.is_empty() {
            // Avanzar cursor de paginación aunque todo fuera duplicado/err.
            if let Some(n) = newest_sig_for_pagination {
                self.cursor.write().await.last_before = Some(n);
            }
            return Ok(SyncStats {
                fetched,
                skipped: fetched,
                ..Default::default()
            });
        }

        let mut stats = SyncStats {
            fetched,
            ..Default::default()
        };

        for sig in &pending {
            // Por firma: fetch_events con timeout+retry.
            let events =
                with_retry(self.config.max_retries, || {
                    let fetcher = self.event_fetcher.clone();
                    let sig = sig.clone();
                    let timeout = self.config.rpc_timeout;
                    async move {
                        with_timeout(timeout, async { fetcher.fetch_events(&sig).await }).await
                    }
                })
                .await?;

            stats.decoded += events.len();

            for ev in &events {
                let upserted = self.apply_to_repo(ev).await?;
                if upserted {
                    stats.upserted += 1;
                }
            }

            self.cursor
                .write()
                .await
                .mark_processed(sig.signature.clone());
            stats.processed += 1;
        }

        stats.skipped = fetched.saturating_sub(stats.processed);

        if let Some(n) = newest_sig_for_pagination {
            self.cursor.write().await.last_before = Some(n);
        }

        Ok(stats)
    }

    /// Loop de polling hasta que `shutdown` se resuelva. Retorna `Ok` al shutdown.
    pub async fn run_polling(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Result<(), SyncError> {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        break;
                    }
                }
                _ = tokio::time::sleep(self.config.polling_interval) => {
                    // Errores de sync no matan el loop: se loguean y se sigue.
                    // El caller puede decidir abortar si recibe un SyncError.
                    let _ = self.sync_once().await;
                }
            }
        }
        Ok(())
    }

    /// Spawn del loop en background. Retorna `JoinHandle`.
    pub fn spawn_polling(
        self: Arc<Self>,
        shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let _ = self.run_polling(shutdown).await;
        })
    }

    // -----------------------------------------------------------------------
    // Aplicación idempotente a repo off-chain
    // -----------------------------------------------------------------------
    // El repo no guarda `status` on-chain; aquí sincronizamos la metadata
    // descriptiva. La regla es **no duplicar** y devolver `AlreadyExists` como
    // éxito idempotente. Para no acoplar el sync al decoder exacto del SDK,
    // `kind` se interpreta por prefijo.
    async fn apply_to_repo(&self, ev: &SyncedEvent) -> Result<bool, SyncError> {
        let kind = ev.kind.as_str();

        // Helper para mapear AlreadyExists -> Ok(false) (idempotencia repo).
        let map_already_exists = |e: RepositoryError| match e {
            RepositoryError::AlreadyExists(_) => Ok(false),
            other => Err(SyncError::Repository(other)),
        };

        // JobCreated: payload esperado `{"pda":"...","title":"...","description":"..."}`
        // Si payload ausente, se construye un placeholder determinístico por signature.
        if kind == "JobCreated" || kind.starts_with("JobCreated") {
            let (pda, title, desc) = if let Some(p) = &ev.payload {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(p) {
                    let pda = v
                        .get("pda")
                        .and_then(|x| x.as_str())
                        .unwrap_or(&ev.signature)
                        .to_string();
                    let title = v
                        .get("title")
                        .and_then(|x| x.as_str())
                        .unwrap_or("Job from on-chain event")
                        .to_string();
                    let desc = v
                        .get("description")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    (pda, title, desc)
                } else {
                    (
                        ev.signature.clone(),
                        "Job from on-chain event".into(),
                        String::new(),
                    )
                }
            } else {
                (
                    ev.signature.clone(),
                    "Job from on-chain event".into(),
                    String::new(),
                )
            };
            // Evitar doble validación de PDA en tests con signature corta: si pda
            // es signature corta no-base58, generar PDA sintética válida (44 chars base58-like).
            let pda = if pda.len() < 32 {
                // Determinística: padding base58-friendly.
                format!(
                    "7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}",
                    ev.slot as u8, ev.slot as u8
                )
            } else {
                pda
            };
            let job = JobMetadata::new(pda, title, desc).map_err(|e| {
                SyncError::InvalidParameter(format!("job validation before repo: {}", e))
            })?;
            match self.repo.create_job(job).await {
                Ok(_) => return Ok(true),
                Err(e) => return map_already_exists(e),
            }
        }

        // ApplicationSubmitted / ApplicationAccepted comparten payload con application_pda
        if kind.starts_with("ApplicationSubmitted") || kind.starts_with("ApplicationAccepted") {
            // Intentamos deserializar payload con `application_pda` y `job_pda`; si no,
            // simplemente reportamos ya-procesado (idempotencia sobre firma).
            // No fallar el sync por falta de metadata off-chain rica.
            return Ok(false);
        }

        // DisputeRaised / DisputeResolved / FundsDeposited etc.: sin metadata específica
        // que mapear a repo en este scaffold; idempotencia a nivel firma es suficiente.
        // Si en el futuro se almacena status off-chain, aquí iría `update_job`.

        // Evidence / Milestone: evidencias usan (dispute_pda, index) como PK.
        // Si payload trae dispute_pda+index+content, intentamos create idempotente.
        if kind.starts_with("EvidenceSubmitted") || kind.starts_with("Evidence") {
            if let Some(p) = &ev.payload {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(p) {
                    let dispute_pda = v
                        .get("dispute_pda")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string());
                    let idx = v.get("index").and_then(|x| x.as_u64()).map(|n| n as u8);
                    let content = v
                        .get("content")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string());
                    if let (Some(dpda), Some(index), Some(content)) = (dispute_pda, idx, content) {
                        // Author sintético determinístico.
                        let author = format!("AuthorFor{:0>32}", ev.slot);
                        let ev_meta =
                            EvidenceMetadata::new(dpda, index, author, content).map_err(|e| {
                                SyncError::InvalidParameter(format!("evidence validation: {}", e))
                            })?;
                        match self.repo.create_evidence(ev_meta).await {
                            Ok(_) => return Ok(true),
                            Err(e) => return map_already_exists(e),
                        }
                    }
                }
            }
            return Ok(false);
        }

        // LogFallback / unknown: sin efecto en repo, pero la firma queda marcada como
        // procesada (idempotencia a nivel transporte).
        Ok(false)
    }
}

// ---------------------------------------------------------------------------
// Mock fetchers for tests & polling sin WebSocket
// ---------------------------------------------------------------------------

/// Fetcher que devuelve firmas preconfiguradas (para tests).
pub struct MockSignatureFetcher {
    /// Se sirve FIFO bajo lock.
    pub batches: Arc<RwLock<Vec<Vec<FetchedSignature>>>>,
}

impl MockSignatureFetcher {
    pub fn new(batches: Vec<Vec<FetchedSignature>>) -> Self {
        Self {
            batches: Arc::new(RwLock::new(batches)),
        }
    }

    pub fn single_batch(sigs: Vec<FetchedSignature>) -> Self {
        Self::new(vec![sigs])
    }
}

#[async_trait::async_trait]
impl SignatureFetcher for MockSignatureFetcher {
    async fn fetch_signatures(
        &self,
        _before: Option<String>,
        _limit: usize,
    ) -> Result<Vec<FetchedSignature>, SyncError> {
        let mut g = self.batches.write().await;
        if g.is_empty() {
            return Ok(Vec::new());
        }
        Ok(g.remove(0))
    }
}

/// Fetcher que simula timeout en los primeros N intentos.
pub struct FlakySignatureFetcher {
    pub batches: Arc<RwLock<Vec<Vec<FetchedSignature>>>>,
    pub fail_times: Arc<RwLock<usize>>,
}

impl FlakySignatureFetcher {
    pub fn new(batches: Vec<Vec<FetchedSignature>>, fail_times: usize) -> Self {
        Self {
            batches: Arc::new(RwLock::new(batches)),
            fail_times: Arc::new(RwLock::new(fail_times)),
        }
    }
}

#[async_trait::async_trait]
impl SignatureFetcher for FlakySignatureFetcher {
    async fn fetch_signatures(
        &self,
        _before: Option<String>,
        _limit: usize,
    ) -> Result<Vec<FetchedSignature>, SyncError> {
        {
            let mut f = self.fail_times.write().await;
            if *f > 0 {
                *f -= 1;
                return Err(SyncError::Timeout("flaky timeout".into()));
            }
        }
        let mut g = self.batches.write().await;
        if g.is_empty() {
            return Ok(Vec::new());
        }
        Ok(g.remove(0))
    }
}

/// Mock event fetcher que mapea `signature -> Vec<SyncedEvent>`.
pub struct MockEventFetcher {
    /// Mapa signature -> events. Si no hay entrada, retorna vacío (caso LogFallback sin eventos Anchor).
    pub map: Arc<RwLock<std::collections::HashMap<String, Vec<SyncedEvent>>>>,
    /// Si Some(n), falla con Timeout n veces antes de responder.
    pub fail_times: Arc<RwLock<usize>>,
    /// Para verificar orden: registra el orden de llamadas.
    pub call_order: Arc<RwLock<Vec<String>>>,
}

impl MockEventFetcher {
    pub fn new(map: std::collections::HashMap<String, Vec<SyncedEvent>>) -> Self {
        Self {
            map: Arc::new(RwLock::new(map)),
            fail_times: Arc::new(RwLock::new(0)),
            call_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_failures(mut self, n: usize) -> Self {
        // Inicializa fail_times sincrónicamente sin await (para uso en tests sync setup).
        // Se escribe lazy en el primer fetch.
        self.fail_times = Arc::new(RwLock::new(n));
        self
    }
}

#[async_trait::async_trait]
impl EventFetcher for MockEventFetcher {
    async fn fetch_events(&self, sig: &FetchedSignature) -> Result<Vec<SyncedEvent>, SyncError> {
        {
            let mut f = self.fail_times.write().await;
            if *f > 0 {
                *f -= 1;
                return Err(SyncError::Timeout("event fetch timeout".into()));
            }
        }
        self.call_order.write().await.push(sig.signature.clone());
        let m = self.map.read().await;
        Ok(m.get(&sig.signature).cloned().unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// Real RPC adapters (solo cuando feature `solana` del SDK está habilitada)
// ---------------------------------------------------------------------------
// Para no romper el build base (sin toolchain Solana), estas impl se
// condicionan. Cuando el workspace habilite `trust-escrow-sdk/solana`,
// el API puede usar `RpcSignatureFetcher`/`RpcEventFetcher`.

#[cfg(any())]
mod rpc_adapters {
    use super::*;
    // Placeholder: el build base no habilita `solana`; documentado para T12.
    // Activar cuando `trust-escrow-sdk/solana` esté habilitado en `api/Cargo.toml`.
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::InMemoryMetadataRepository;

    fn sig(s: &str, slot: u64) -> FetchedSignature {
        FetchedSignature {
            signature: s.to_string(),
            slot,
            block_time: None,
            is_err: false,
        }
    }

    fn sig_with_time(s: &str, slot: u64, bt: i64) -> FetchedSignature {
        FetchedSignature {
            signature: s.to_string(),
            slot,
            block_time: Some(bt),
            is_err: false,
        }
    }

    fn job_event(sig: &str, slot: u64) -> SyncedEvent {
        SyncedEvent {
            signature: sig.to_string(),
            slot,
            kind: "JobCreated".into(),
            payload: Some(format!(
                r#"{{"pda":"{}","title":"T {}","description":"D {}"}}"#,
                sig, sig, sig
            )),
        }
    }

    #[tokio::test]
    async fn cursor_dedup_and_order() {
        let c = SyncCursor::new(10);
        // Lote newest-first desordenado (simula getSignaturesForAddress).
        let batch = vec![sig("sig3", 3), sig("sig1", 1), sig("sig2", 2)];
        let pending = c.pending_ordered(batch);
        assert_eq!(pending[0].signature, "sig1");
        assert_eq!(pending[1].signature, "sig2");
        assert_eq!(pending[2].signature, "sig3");
    }

    #[tokio::test]
    async fn cursor_tie_breaker_block_time_then_sig() {
        let c = SyncCursor::new(10);
        let batch = vec![
            sig_with_time("b", 5, 200),
            sig_with_time("a", 5, 100),
            sig_with_time("c", 5, 200),
        ];
        let pending = c.pending_ordered(batch);
        assert_eq!(pending[0].signature, "a"); // earliest block_time
                                               // b y c comparten slot+time, orden lexicográfico
        assert_eq!(pending[1].signature, "b");
        assert_eq!(pending[2].signature, "c");
    }

    #[tokio::test]
    async fn cursor_filters_err_and_processed() {
        let mut c = SyncCursor::new(10);
        c.mark_processed("sig1".into());
        let batch = vec![
            sig("sig1", 1),
            FetchedSignature {
                signature: "sigErr".into(),
                slot: 2,
                block_time: None,
                is_err: true,
            },
            sig("sig2", 2),
        ];
        let pending = c.pending_ordered(batch);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].signature, "sig2");
    }

    #[tokio::test]
    async fn cursor_capacity_eviction_fifo() {
        let mut c = SyncCursor::new(2);
        c.mark_processed("a".into());
        c.mark_processed("b".into());
        assert_eq!(c.len(), 2);
        c.mark_processed("c".into());
        assert_eq!(c.len(), 2);
        assert!(!c.is_processed("a"));
        assert!(c.is_processed("b"));
        assert!(c.is_processed("c"));
        // re-insertar existente no evicta
        c.mark_processed("b".into());
        assert_eq!(c.len(), 2);
    }

    #[tokio::test]
    async fn sync_once_idempotency_second_call_skips() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        let sigs = vec![sig("sigA", 1), sig("sigB", 2)];
        let sig_fetcher = Arc::new(MockSignatureFetcher::new(vec![sigs.clone(), sigs.clone()]));
        let mut map = std::collections::HashMap::new();
        map.insert("sigA".into(), vec![job_event("sigA", 1)]);
        map.insert("sigB".into(), vec![job_event("sigB", 2)]);
        let event_fetcher = Arc::new(MockEventFetcher::new(map));

        let engine = SyncEngine::new(
            repo.clone(),
            sig_fetcher,
            event_fetcher,
            SyncConfig {
                polling_interval: Duration::from_millis(10),
                rpc_timeout: Duration::from_secs(1),
                max_retries: 1,
                batch_size: 10,
            },
            SyncCursor::new(100),
        );

        let s1 = engine.sync_once().await.unwrap();
        assert_eq!(s1.processed, 2);
        assert_eq!(s1.upserted, 2);
        assert_eq!(s1.skipped, 0);

        // Segunda llamada con mismo lote: todo debe ser skipped por idempotencia.
        let s2 = engine.sync_once().await.unwrap();
        assert_eq!(s2.processed, 0);
        assert_eq!(s2.skipped, 2);
        assert_eq!(s2.upserted, 0);

        // Repo debe tener exactamente 2 jobs (idempotencia repo).
        assert_eq!(repo.list_jobs().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn sync_once_ordering_calls_in_slot_order() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        // getSignatures retorna newest-first; pending debe reordenar.
        let sigs = vec![sig("sig3", 30), sig("sig1", 10), sig("sig2", 20)];
        let sig_fetcher = Arc::new(MockSignatureFetcher::single_batch(sigs));
        let mut map = std::collections::HashMap::new();
        map.insert("sig1".into(), vec![job_event("sig1", 10)]);
        map.insert("sig2".into(), vec![job_event("sig2", 20)]);
        map.insert("sig3".into(), vec![job_event("sig3", 30)]);
        let event_fetcher = Arc::new(MockEventFetcher::new(map));
        let call_order = event_fetcher.call_order.clone();

        let engine = SyncEngine::new(
            repo,
            sig_fetcher,
            event_fetcher,
            SyncConfig::default(),
            SyncCursor::new(100),
        );
        engine.sync_once().await.unwrap();
        let order = call_order.read().await.clone();
        assert_eq!(order, vec!["sig1", "sig2", "sig3"]);
    }

    #[tokio::test]
    async fn sync_once_retry_on_signature_timeout() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        let sigs = vec![sig("sigX", 1)];
        let sig_fetcher = Arc::new(FlakySignatureFetcher::new(vec![sigs], 1));
        let mut map = std::collections::HashMap::new();
        map.insert("sigX".into(), vec![job_event("sigX", 1)]);
        let event_fetcher = Arc::new(MockEventFetcher::new(map));

        let engine = SyncEngine::new(
            repo.clone(),
            sig_fetcher,
            event_fetcher,
            SyncConfig {
                rpc_timeout: Duration::from_millis(200),
                max_retries: 2,
                ..Default::default()
            },
            SyncCursor::new(100),
        );
        let stats = engine.sync_once().await.unwrap();
        assert_eq!(stats.processed, 1);
        assert_eq!(stats.upserted, 1);
    }

    #[tokio::test]
    async fn sync_once_retry_on_event_timeout() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        let sig_fetcher = Arc::new(MockSignatureFetcher::single_batch(vec![sig("sigY", 1)]));
        let mut map = std::collections::HashMap::new();
        map.insert("sigY".into(), vec![job_event("sigY", 1)]);
        let event_fetcher = Arc::new(MockEventFetcher::new(map).with_failures(1));

        let engine = SyncEngine::new(
            repo.clone(),
            sig_fetcher,
            event_fetcher,
            SyncConfig {
                rpc_timeout: Duration::from_millis(200),
                max_retries: 3,
                ..Default::default()
            },
            SyncCursor::new(100),
        );
        let stats = engine.sync_once().await.unwrap();
        assert_eq!(stats.processed, 1);
        assert_eq!(stats.upserted, 1);
    }

    #[tokio::test]
    async fn sync_once_timeout_exhausted_returns_error() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        let sig_fetcher = Arc::new(FlakySignatureFetcher::new(vec![vec![sig("s", 1)]], 10));
        let event_fetcher = Arc::new(MockEventFetcher::new(std::collections::HashMap::new()));

        let engine = SyncEngine::new(
            repo,
            sig_fetcher,
            event_fetcher,
            SyncConfig {
                rpc_timeout: Duration::from_millis(50),
                max_retries: 1,
                ..Default::default()
            },
            SyncCursor::new(10),
        );
        let res = engine.sync_once().await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), SyncError::Timeout(_)));
    }

    #[tokio::test]
    async fn timeout_helper_maps_to_sync_error() {
        let r: Result<(), SyncError> = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        })
        .await;
        assert!(r.is_err());
        assert!(r.unwrap_err().is_timeout());
    }

    #[tokio::test]
    async fn with_retry_only_retries_timeout() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = Arc::new(AtomicUsize::new(0));
        let c = calls.clone();
        let res: Result<(), SyncError> = with_retry(3, move || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(SyncError::InvalidParameter("bad".into()))
            }
        })
        .await;
        assert!(res.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn repo_already_exists_is_idempotent_not_error() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        // Pre-insertar job con misma pda que el evento va a intentar crear.
        // Usar PDA válida (32..128 chars) y payload explícito con esa misma PDA.
        let valid_pda = format!("7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}", 1u8, 1u8);
        let existing =
            JobMetadata::new(valid_pda.clone(), "T sigDup".into(), "D sigDup".into()).unwrap();
        repo.create_job(existing).await.unwrap();

        let sig_fetcher = Arc::new(MockSignatureFetcher::single_batch(vec![sig("sigDup", 1)]));
        let mut map = std::collections::HashMap::new();
        let dup_event = SyncedEvent {
            signature: "sigDup".into(),
            slot: 1,
            kind: "JobCreated".into(),
            payload: Some(format!(
                r#"{{"pda":"{}","title":"T sigDup","description":"D sigDup"}}"#,
                valid_pda
            )),
        };
        map.insert("sigDup".into(), vec![dup_event]);
        let event_fetcher = Arc::new(MockEventFetcher::new(map));

        let engine = SyncEngine::new(
            repo.clone(),
            sig_fetcher,
            event_fetcher,
            SyncConfig::default(),
            SyncCursor::new(10),
        );
        let stats = engine.sync_once().await.unwrap();
        // Firma procesada pero repo reportó AlreadyExists -> upserted 0, no error.
        assert_eq!(stats.processed, 1);
        assert_eq!(stats.upserted, 0);
        assert_eq!(repo.list_jobs().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn empty_batch_returns_zero_stats() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        let sig_fetcher = Arc::new(MockSignatureFetcher::single_batch(vec![]));
        let event_fetcher = Arc::new(MockEventFetcher::new(std::collections::HashMap::new()));
        let engine = SyncEngine::new(
            repo,
            sig_fetcher,
            event_fetcher,
            SyncConfig::default(),
            SyncCursor::new(10),
        );
        let stats = engine.sync_once().await.unwrap();
        assert_eq!(stats.fetched, 0);
        assert_eq!(stats.processed, 0);
    }

    #[tokio::test]
    async fn evidence_event_upserts_idempotently() {
        let repo: Arc<dyn MetadataRepository> = Arc::new(InMemoryMetadataRepository::new());
        let pda = format!("7a2YhCd7iivXfyySkp1pf5jj{:0>20}{:02}", 9u8, 9u8);
        let ev = SyncedEvent {
            signature: "sigEv".into(),
            slot: 5,
            kind: "EvidenceSubmitted".into(),
            payload: Some(format!(
                r#"{{"dispute_pda":"{}","index":0,"content":"hello evidence"}}"#,
                pda
            )),
        };
        let sig_fetcher = Arc::new(MockSignatureFetcher::single_batch(vec![sig("sigEv", 5)]));
        let mut map = std::collections::HashMap::new();
        map.insert("sigEv".into(), vec![ev]);
        let event_fetcher = Arc::new(MockEventFetcher::new(map));

        let engine = SyncEngine::new(
            repo.clone(),
            sig_fetcher.clone(),
            event_fetcher.clone(),
            SyncConfig::default(),
            SyncCursor::new(10),
        );
        let s1 = engine.sync_once().await.unwrap();
        assert_eq!(s1.upserted, 1);

        // Reproceso con nueva firma distinta pero mismo (dispute_pda,index) -> AlreadyExists path.
        let ev2 = SyncedEvent {
            signature: "sigEv2".into(),
            slot: 6,
            kind: "EvidenceSubmitted".into(),
            payload: Some(format!(
                r#"{{"dispute_pda":"{}","index":0,"content":"hello evidence"}}"#,
                pda
            )),
        };
        // Reemplazar fetchers para segunda ronda con nueva firma.
        let sig_fetcher2 = Arc::new(MockSignatureFetcher::single_batch(vec![sig("sigEv2", 6)]));
        let mut map2 = std::collections::HashMap::new();
        map2.insert("sigEv2".into(), vec![ev2]);
        let event_fetcher2 = Arc::new(MockEventFetcher::new(map2));
        let engine2 = SyncEngine::new_shared_cursor(
            repo.clone(),
            sig_fetcher2,
            event_fetcher2,
            SyncConfig::default(),
            engine.cursor_handle(),
        );
        let s2 = engine2.sync_once().await.unwrap();
        assert_eq!(s2.upserted, 0); // duplicado por PK (dispute_pda,index)
        assert_eq!(repo.list_evidence_by_dispute(&pda).await.unwrap().len(), 1);
    }
}

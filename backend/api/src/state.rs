//! Application state — pool, repo, config, metrics.
//!
//! `AppState` is the single shared object injected via `axum::extract::State`.
//! It holds a `dyn MetadataRepository` (in-memory by default, Postgres/Mongo
//! when Docker is available), typed `ApiConfig` loaded from env, and lightweight
//! counters for metrics. All fields are `Clone` + `Send + Sync` so handlers
//! can be `Clone` without locking.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;

use tokio::sync::RwLock;

use crate::repository::{InMemoryMetadataRepository, MetadataRepository};

/// Typed runtime configuration loaded from environment.
///
/// No secrets are stored here beyond URLs (never keypairs). `from_env` never
/// panics — missing vars fall back to documented defaults so `cargo test`
/// works without a `.env` file.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// HTTP port (env `PORT`, default 3000).
    pub port: u16,
    /// Solana RPC URL (env `SOLANA_RPC_URL` / `RPC_URL` / `ANCHOR_PROVIDER_URL`).
    pub rpc_url: String,
    /// Optional Postgres URL (`DATABASE_URL`).
    pub database_url: Option<String>,
    /// Optional Mongo URL (`MONGO_URL`).
    pub mongo_url: Option<String>,
    /// Crate version (`CARGO_PKG_VERSION`).
    pub version: String,
    /// Environment name (`ENV` or `RUST_ENV`, default `development`).
    pub environment: String,
}

impl ApiConfig {
    /// Load config from environment with safe defaults.
    pub fn from_env() -> Self {
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);

        // Prefer the canonical Anchor/Solana env names, fall back to generic.
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .or_else(|_| std::env::var("RPC_URL"))
            .or_else(|_| std::env::var("ANCHOR_PROVIDER_URL"))
            .unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());

        let database_url = std::env::var("DATABASE_URL").ok();
        let mongo_url = std::env::var("MONGO_URL")
            .or_else(|_| std::env::var("MONGODB_URL"))
            .ok();
        let version = env!("CARGO_PKG_VERSION").to_string();
        let environment = std::env::var("ENV")
            .or_else(|_| std::env::var("RUST_ENV"))
            .unwrap_or_else(|_| "development".to_string());

        Self {
            port,
            rpc_url,
            database_url,
            mongo_url,
            version,
            environment,
        }
    }

    /// Whether the process is running in production (enables stricter middleware).
    pub fn is_production(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
            || self.environment.eq_ignore_ascii_case("prod")
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        // In test context env may be empty — use deterministic defaults.
        Self {
            port: 3000,
            rpc_url: "http://127.0.0.1:8899".to_string(),
            database_url: None,
            mongo_url: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
        }
    }
}

/// Shared application state injected into every handler.
#[derive(Clone)]
pub struct AppState {
    /// Typed config (Arc so `Clone` is cheap).
    pub config: Arc<ApiConfig>,
    /// Off-chain metadata repository.
    pub repo: Arc<dyn MetadataRepository>,
    /// In-memory arbiter pool (authoritative set: authority + arbiters).
    pub arbiter_pool: Arc<RwLock<Option<ArbiterPoolState>>>,
    /// Instant when the process started (for uptime).
    pub start_time: Instant,
    /// Total HTTP requests observed (incremented by middleware).
    pub requests_total: Arc<AtomicU64>,
    /// Total error responses (4xx/5xx).
    pub errors_total: Arc<AtomicU64>,
}

/// In-memory representation of the on-chain `ArbiterPool`.
#[derive(Debug, Clone)]
pub struct ArbiterPoolState {
    pub authority: String,
    pub arbiters: Vec<String>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("start_time", &self.start_time)
            .field(
                "requests_total",
                &self.requests_total.load(Ordering::Relaxed),
            )
            .field("errors_total", &self.errors_total.load(Ordering::Relaxed))
            .finish()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::with_repository(Arc::new(InMemoryMetadataRepository::new()))
    }
}

impl AppState {
    /// Create state with a concrete repository and env-derived config.
    pub fn with_repository(repo: Arc<dyn MetadataRepository>) -> Self {
        Self::with_config_and_repository(ApiConfig::from_env(), repo)
    }

    /// Create state with explicit config and repository (useful in tests).
    pub fn with_config_and_repository(config: ApiConfig, repo: Arc<dyn MetadataRepository>) -> Self {
        Self {
            config: Arc::new(config),
            repo,
            arbiter_pool: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
            requests_total: Arc::new(AtomicU64::new(0)),
            errors_total: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Convenience for tests: in-memory repo + custom config.
    pub fn with_config(config: ApiConfig) -> Self {
        Self::with_config_and_repository(config, Arc::new(InMemoryMetadataRepository::new()))
    }

    /// Seconds since process start.
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Increment request counter (called by middleware).
    pub fn inc_requests(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error counter (called when response is 4xx/5xx).
    pub fn inc_errors(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_env_defaults() {
        // Ensure default config is sensible even when env is empty.
        let cfg = ApiConfig {
            port: 3000,
            rpc_url: "http://127.0.0.1:8899".to_string(),
            database_url: None,
            mongo_url: None,
            version: "3.0.0".to_string(),
            environment: "development".to_string(),
        };
        assert_eq!(cfg.port, 3000);
        assert!(!cfg.rpc_url.is_empty());
        assert!(!cfg.is_production());
    }

    #[test]
    fn state_default_has_repo_and_counters() {
        let state = AppState::default();
        assert_eq!(state.requests_total.load(Ordering::Relaxed), 0);
        assert_eq!(state.errors_total.load(Ordering::Relaxed), 0);
        // uptime should be tiny but non-negative
        assert!(state.uptime_seconds() < 5);
    }

    #[test]
    fn state_inc_counters() {
        let state = AppState::default();
        state.inc_requests();
        state.inc_requests();
        state.inc_errors();
        assert_eq!(state.requests_total.load(Ordering::Relaxed), 2);
        assert_eq!(state.errors_total.load(Ordering::Relaxed), 1);
    }
}

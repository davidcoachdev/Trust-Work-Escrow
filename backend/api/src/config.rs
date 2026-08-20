//! Config loader — reads `.env`, validates env vars, and feeds `ApiConfig::from_env`.
//!
//! Responsibilities:
//! - Load `.env` via `dotenvy` (no panic if missing).
//! - Validate each env var with typed errors (`ConfigError`).
//! - Provide `.env.example` content and helper to ensure file exists.
//! - Expose `try_from_env` (strict) and keep `from_env` (lenient with defaults)
//!   so `cargo test` never requires a `.env` file.

use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Typed validation error for environment configuration.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConfigError {
    #[error("invalid PORT '{value}': {reason}")]
    InvalidPort { value: String, reason: String },

    #[error("invalid RPC URL '{value}': {reason}")]
    InvalidRpcUrl { value: String, reason: String },

    #[error("invalid DATABASE_URL: {reason}")]
    InvalidDatabaseUrl { reason: String },

    #[error("invalid MONGO_URL: {reason}")]
    InvalidMongoUrl { reason: String },

    #[error("invalid ENV/RUST_ENV '{value}': {reason}")]
    InvalidEnv { value: String, reason: String },

    #[error("invalid CORS_ALLOWED_ORIGINS '{value}': {reason}")]
    InvalidCors { value: String, reason: String },

    #[error("invalid RATE_LIMIT_REQUESTS '{value}': {reason}")]
    InvalidRateLimitRequests { value: String, reason: String },

    #[error("invalid RATE_LIMIT_WINDOW_SECS '{value}': {reason}")]
    InvalidRateLimitWindow { value: String, reason: String },
}

// ---------------------------------------------------------------------------
// .env loading
// ---------------------------------------------------------------------------

/// Load `.env` from current working directory (and parents) if present.
///
/// No-op if file is missing — this keeps `cargo test` hermetic. Call at the
/// start of `ApiConfig::from_env` / `try_from_env`.
pub fn load_dotenv() {
    // `dotenv()` walks up from cwd, `from_filename` is not needed.
    // Ignore errors — missing file is expected in CI.
    let _ = dotenvy::dotenv();
}

// ---------------------------------------------------------------------------
// Validators (pure, testable)
// ---------------------------------------------------------------------------

pub fn validate_port(raw: &str) -> Result<u16, ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidPort {
            value: raw.to_string(),
            reason: "empty value".to_string(),
        });
    }
    let port: u16 = trimmed.parse().map_err(|_| ConfigError::InvalidPort {
        value: raw.to_string(),
        reason: "must be an integer 1-65535".to_string(),
    })?;
    if port == 0 {
        return Err(ConfigError::InvalidPort {
            value: raw.to_string(),
            reason: "port 0 is reserved".to_string(),
        });
    }
    Ok(port)
}

pub fn validate_rpc_url(raw: &str) -> Result<String, ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidRpcUrl {
            value: raw.to_string(),
            reason: "empty value".to_string(),
        });
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(ConfigError::InvalidRpcUrl {
            value: raw.to_string(),
            reason: "must start with http:// or https://".to_string(),
        });
    }
    // Basic shape: must contain "://" and host part
    let after_scheme = trimmed.split_once("://").map(|x| x.1).unwrap_or("");
    if after_scheme.is_empty() || !after_scheme.contains('.') && !after_scheme.contains("localhost") && !after_scheme.contains("127.0.0.1") {
        // Allow localhost/127.0.0.1 without dot, otherwise require dot or colon (host:port)
        if !after_scheme.starts_with("127.0.0.1") && !after_scheme.starts_with("localhost") {
            return Err(ConfigError::InvalidRpcUrl {
                value: raw.to_string(),
                reason: "missing host".to_string(),
            });
        }
    }
    // Reject obvious placeholder with spaces
    if trimmed.contains(' ') {
        return Err(ConfigError::InvalidRpcUrl {
            value: raw.to_string(),
            reason: "URL must not contain spaces".to_string(),
        });
    }
    Ok(trimmed.to_string())
}

pub fn validate_database_url(raw: &str) -> Result<String, ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidDatabaseUrl {
            reason: "empty value".to_string(),
        });
    }
    if !(trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://")) {
        return Err(ConfigError::InvalidDatabaseUrl {
            reason: "must start with postgres:// or postgresql://".to_string(),
        });
    }
    if trimmed.contains(' ') {
        return Err(ConfigError::InvalidDatabaseUrl {
            reason: "URL must not contain spaces".to_string(),
        });
    }
    Ok(trimmed.to_string())
}

pub fn validate_mongo_url(raw: &str) -> Result<String, ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidMongoUrl {
            reason: "empty value".to_string(),
        });
    }
    if !(trimmed.starts_with("mongodb://") || trimmed.starts_with("mongodb+srv://")) {
        return Err(ConfigError::InvalidMongoUrl {
            reason: "must start with mongodb:// or mongodb+srv://".to_string(),
        });
    }
    if trimmed.contains(' ') {
        return Err(ConfigError::InvalidMongoUrl {
            reason: "URL must not contain spaces".to_string(),
        });
    }
    Ok(trimmed.to_string())
}

/// Allowed environment names (case-insensitive).
const ALLOWED_ENVS: &[&str] = &["development", "production", "prod", "test", "staging"];

pub fn validate_environment(raw: &str) -> Result<String, ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::InvalidEnv {
            value: raw.to_string(),
            reason: "empty value".to_string(),
        });
    }
    let lower = trimmed.to_ascii_lowercase();
    if !ALLOWED_ENVS.contains(&lower.as_str()) {
        return Err(ConfigError::InvalidEnv {
            value: raw.to_string(),
            reason: format!("must be one of: {}", ALLOWED_ENVS.join(", ")),
        });
    }
    Ok(trimmed.to_string())
}

pub fn validate_cors_origins(raw: &str) -> Result<Vec<String>, ConfigError> {
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for part in raw.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "*" {
            out.push(trimmed.to_string());
            continue;
        }
        if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            return Err(ConfigError::InvalidCors {
                value: trimmed.to_string(),
                reason: "each origin must start with http://, https://, or be \"*\"".to_string(),
            });
        }
        if trimmed.contains(' ') {
            return Err(ConfigError::InvalidCors {
                value: trimmed.to_string(),
                reason: "origin must not contain spaces".to_string(),
            });
        }
        out.push(trimmed.to_string());
    }
    Ok(out)
}

pub fn validate_rate_limit_requests(raw: &str) -> Result<usize, ConfigError> {
    let trimmed = raw.trim();
    let v: usize = trimmed.parse().map_err(|_| ConfigError::InvalidRateLimitRequests {
        value: raw.to_string(),
        reason: "must be a positive integer".to_string(),
    })?;
    if v == 0 {
        return Err(ConfigError::InvalidRateLimitRequests {
            value: raw.to_string(),
            reason: "must be > 0".to_string(),
        });
    }
    if v > 10_000 {
        return Err(ConfigError::InvalidRateLimitRequests {
            value: raw.to_string(),
            reason: "must be <= 10000".to_string(),
        });
    }
    Ok(v)
}

pub fn validate_rate_limit_window(raw: &str) -> Result<u64, ConfigError> {
    let trimmed = raw.trim();
    let v: u64 = trimmed.parse().map_err(|_| ConfigError::InvalidRateLimitWindow {
        value: raw.to_string(),
        reason: "must be a positive integer (seconds)".to_string(),
    })?;
    if v == 0 {
        return Err(ConfigError::InvalidRateLimitWindow {
            value: raw.to_string(),
            reason: "must be > 0".to_string(),
        });
    }
    if v > 86_400 {
        return Err(ConfigError::InvalidRateLimitWindow {
            value: raw.to_string(),
            reason: "must be <= 86400 (24h)".to_string(),
        });
    }
    Ok(v)
}

// ---------------------------------------------------------------------------
// .env.example content
// ---------------------------------------------------------------------------

/// Canonical `.env.example` content — kept in sync with `ApiConfig` fields.
///
/// Generate or refresh the file on disk via `cargo run --bin trust-escrow-api`
/// or by copying this constant into `backend/.env.example`.
pub const ENV_EXAMPLE: &str = r#"# ── Trust Work Escrow — Backend API ──────────────────────────────
# Copy to .env and fill values. Never commit .env with secrets.
# See backend/api/src/config.rs for validation rules.

# HTTP port (1-65535, default 3000)
PORT=3000

# Solana RPC URL — primary key SOLANA_RPC_URL, fallbacks RPC_URL / ANCHOR_PROVIDER_URL
SOLANA_RPC_URL=http://127.0.0.1:8899
# Alternative names (uncomment if needed):
# RPC_URL=http://127.0.0.1:8899
# ANCHOR_PROVIDER_URL=http://127.0.0.1:8899

# Postgres — optional, enables off-chain metadata persistence
DATABASE_URL=postgres://postgres:postgres@localhost:5432/trust_escrow

# MongoDB — optional, for large/flexible content (evidence, logs)
MONGO_URL=mongodb://localhost:27017/trust_escrow
# Alternative: MONGODB_URL=mongodb://localhost:27017/trust_escrow
# Atlas: mongodb+srv://user:pass@cluster.mongodb.net/trust_escrow

# Environment: development | production | prod | test | staging (default development)
ENV=development
# Alternative: RUST_ENV=development

# CORS allowed origins — comma-separated, each http(s)://host or "*"
# Example: https://app.example.com,https://admin.example.com
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173

# Rate limiting — requests per window (1-10000) and window in seconds (1-86400)
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60

# Logging — standard RUST_LOG / tracing filter (optional)
RUST_LOG=trust_escrow_api=debug,tower_http=debug
"#;

/// Return the `.env.example` content.
pub fn env_example_content() -> &'static str {
    ENV_EXAMPLE
}

// ---------------------------------------------------------------------------
// Helpers for ApiConfig parsing (used by state.rs)
// ---------------------------------------------------------------------------

/// Parse `PORT` with validation. Returns `None` if not set, `Err` if set but invalid.
pub fn parse_port() -> Result<Option<u16>, ConfigError> {
    match std::env::var("PORT") {
        Ok(raw) => validate_port(&raw).map(Some),
        Err(_) => Ok(None),
    }
}

/// Parse RPC URL with fallback chain. Returns `Err` only if a set var is invalid.
pub fn parse_rpc_url() -> Result<Option<String>, ConfigError> {
    for key in ["SOLANA_RPC_URL", "RPC_URL", "ANCHOR_PROVIDER_URL"] {
        if let Ok(raw) = std::env::var(key) {
            return validate_rpc_url(&raw).map(Some);
        }
    }
    Ok(None)
}

/// Parse DATABASE_URL (optional).
pub fn parse_database_url() -> Result<Option<String>, ConfigError> {
    match std::env::var("DATABASE_URL") {
        Ok(raw) if raw.trim().is_empty() => Ok(None),
        Ok(raw) => validate_database_url(&raw).map(Some),
        Err(_) => Ok(None),
    }
}

/// Parse MONGO_URL / MONGODB_URL (optional).
pub fn parse_mongo_url() -> Result<Option<String>, ConfigError> {
    for key in ["MONGO_URL", "MONGODB_URL"] {
        if let Ok(raw) = std::env::var(key) {
            if raw.trim().is_empty() {
                return Ok(None);
            }
            return validate_mongo_url(&raw).map(Some);
        }
    }
    Ok(None)
}

/// Parse environment name.
pub fn parse_environment() -> Result<Option<String>, ConfigError> {
    for key in ["ENV", "RUST_ENV"] {
        if let Ok(raw) = std::env::var(key) {
            return validate_environment(&raw).map(Some);
        }
    }
    Ok(None)
}

/// Parse CORS origins.
pub fn parse_cors_origins() -> Result<Vec<String>, ConfigError> {
    match std::env::var("CORS_ALLOWED_ORIGINS") {
        Ok(raw) => validate_cors_origins(&raw),
        Err(_) => Ok(Vec::new()),
    }
}

/// Parse RATE_LIMIT_REQUESTS.
pub fn parse_rate_limit_requests() -> Result<Option<usize>, ConfigError> {
    match std::env::var("RATE_LIMIT_REQUESTS") {
        Ok(raw) => validate_rate_limit_requests(&raw).map(Some),
        Err(_) => Ok(None),
    }
}

/// Parse RATE_LIMIT_WINDOW_SECS.
pub fn parse_rate_limit_window_secs() -> Result<Option<u64>, ConfigError> {
    match std::env::var("RATE_LIMIT_WINDOW_SECS") {
        Ok(raw) => validate_rate_limit_window(raw.trim()).map(Some),
        Err(_) => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_valid() {
        assert_eq!(validate_port("3000").unwrap(), 3000);
        assert_eq!(validate_port(" 8080 ").unwrap(), 8080);
    }

    #[test]
    fn port_invalid() {
        assert!(validate_port("0").is_err());
        assert!(validate_port("99999").is_err());
        assert!(validate_port("abc").is_err());
        assert!(validate_port("").is_err());
    }

    #[test]
    fn rpc_valid() {
        assert!(validate_rpc_url("http://127.0.0.1:8899").is_ok());
        assert!(validate_rpc_url("https://api.mainnet-beta.solana.com").is_ok());
        assert!(validate_rpc_url("http://localhost:8899").is_ok());
    }

    #[test]
    fn rpc_invalid() {
        assert!(validate_rpc_url("127.0.0.1:8899").is_err());
        assert!(validate_rpc_url("").is_err());
        assert!(validate_rpc_url("ftp://example.com").is_err());
    }

    #[test]
    fn database_valid() {
        assert!(validate_database_url("postgres://user:pass@localhost/db").is_ok());
        assert!(validate_database_url("postgresql://localhost/db").is_ok());
    }

    #[test]
    fn database_invalid() {
        assert!(validate_database_url("mysql://localhost/db").is_err());
        assert!(validate_database_url("").is_err());
    }

    #[test]
    fn mongo_valid() {
        assert!(validate_mongo_url("mongodb://localhost:27017/db").is_ok());
        assert!(validate_mongo_url("mongodb+srv://cluster.mongodb.net/db").is_ok());
    }

    #[test]
    fn mongo_invalid() {
        assert!(validate_mongo_url("postgres://localhost/db").is_err());
    }

    #[test]
    fn env_valid() {
        assert!(validate_environment("development").is_ok());
        assert!(validate_environment("PRODUCTION").is_ok());
        assert!(validate_environment("test").is_ok());
        assert!(validate_environment("staging").is_ok());
    }

    #[test]
    fn env_invalid() {
        assert!(validate_environment("prodction").is_err());
        assert!(validate_environment("").is_err());
    }

    #[test]
    fn cors_valid() {
        assert_eq!(validate_cors_origins("").unwrap(), Vec::<String>::new());
        assert_eq!(
            validate_cors_origins("http://localhost:3000, https://example.com").unwrap(),
            vec!["http://localhost:3000", "https://example.com"]
        );
        assert_eq!(validate_cors_origins("*").unwrap(), vec!["*"]);
    }

    #[test]
    fn cors_invalid() {
        assert!(validate_cors_origins("example.com").is_err());
        assert!(validate_cors_origins("http://example.com, not-a-url").is_err());
    }

    #[test]
    fn rate_limit_valid() {
        assert_eq!(validate_rate_limit_requests("100").unwrap(), 100);
        assert_eq!(validate_rate_limit_window("60").unwrap(), 60);
    }

    #[test]
    fn rate_limit_invalid() {
        assert!(validate_rate_limit_requests("0").is_err());
        assert!(validate_rate_limit_requests("20000").is_err());
        assert!(validate_rate_limit_window("0").is_err());
    }

    #[test]
    fn env_example_contains_keys() {
        let content = env_example_content();
        assert!(content.contains("PORT="));
        assert!(content.contains("SOLANA_RPC_URL"));
        assert!(content.contains("DATABASE_URL"));
        assert!(content.contains("MONGO_URL"));
        assert!(content.contains("CORS_ALLOWED_ORIGINS"));
        assert!(content.contains("RATE_LIMIT_REQUESTS"));
    }

    #[test]
    fn dotenv_does_not_panic_when_missing() {
        // Should not panic even without .env
        load_dotenv();
    }
}

//! Configuration management for Trust Work Escrow applications
//!
//! Provides hierarchical configuration loading from multiple sources:
//! 1. Default values
//! 2. System-wide config (/etc/trust-escrow/)
//! 3. User config (~/.config/trust-escrow/)
//! 4. Local project config (./trust-escrow.toml)
//! 5. Environment variables (TRUST_ESCROW_*)
//! 6. Command line arguments

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure for escrow applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowConfig {
    /// Network configuration
    pub network: NetworkConfig,

    /// Wallet configuration
    pub wallet: WalletConfig,

    /// Application-specific settings
    pub app: AppConfig,

    /// Program addresses
    pub programs: ProgramConfig,

    /// Fee configuration
    pub fees: FeeConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// RPC URL for Solana cluster
    pub rpc_url: String,

    /// WebSocket URL for subscriptions
    pub ws_url: Option<String>,

    /// Network name (localnet, devnet, mainnet-beta)
    pub cluster: String,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Max retry attempts
    pub max_retries: u32,

    /// Commitment level
    pub commitment: String,
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Path to keypair file
    pub keypair_path: Option<PathBuf>,

    /// Wallet address (if using external wallet)
    pub address: Option<String>,

    /// Wallet type (filesystem, ledger, etc.)
    pub wallet_type: WalletType,
}

/// Supported wallet types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WalletType {
    Filesystem,
    Ledger,
    Remote,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Log level
    pub log_level: String,

    /// Data directory for cache/storage
    pub data_dir: PathBuf,

    /// Enable colored output
    pub colored: bool,

    /// Custom settings
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Program addresses configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// Trust Escrow v2 program ID
    pub trust_escrow_v2: String,

    /// Additional program addresses
    #[serde(default)]
    pub additional: HashMap<String, String>,
}

/// Fee configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
    /// Default priority fee in microlamports
    pub priority_fee: u64,

    /// Max fee percentage (basis points, e.g., 100 = 1%)
    pub max_fee_bps: u16,
}

impl Default for EscrowConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            wallet: WalletConfig::default(),
            app: AppConfig::default(),
            programs: ProgramConfig::default(),
            fees: FeeConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(),
            ws_url: Some("ws://localhost:8900".to_string()),
            cluster: "localnet".to_string(),
            timeout: 30,
            max_retries: 3,
            commitment: "confirmed".to_string(),
        }
    }
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            keypair_path: dirs::home_dir().map(|h| h.join(".config/solana/id.json")),
            address: None,
            wallet_type: WalletType::Filesystem,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".local/share"))
                .join("trust-escrow"),
            colored: true,
            custom: HashMap::new(),
        }
    }
}

impl Default for ProgramConfig {
    fn default() -> Self {
        Self {
            trust_escrow_v2: "11111111111111111111111111111112".to_string(), // Placeholder
            additional: HashMap::new(),
        }
    }
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            priority_fee: 1000, // 1k microlamports
            max_fee_bps: 500,   // 5%
        }
    }
}

impl EscrowConfig {
    /// Load configuration from all available sources
    pub fn load() -> AppResult<Self> {
        let mut config = Self::default();

        // 1. Load from system config
        if let Ok(system_config) = Self::load_from_file("/etc/trust-escrow/config.toml") {
            config.merge(system_config)?;
        }

        // 2. Load from user config
        if let Some(config_dir) = dirs::config_dir() {
            let user_config_path = config_dir.join("trust-escrow/config.toml");
            if let Ok(user_config) = Self::load_from_file(&user_config_path) {
                config.merge(user_config)?;
            }
        }

        // 3. Load from local config
        if let Ok(local_config) = Self::load_from_file("./trust-escrow.toml") {
            config.merge(local_config)?;
        }

        // 4. Apply environment variables
        config.apply_env_vars()?;

        // 5. Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            AppError::config(format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            ))
        })?;

        let expanded_content = shellexpand::full(&content).map_err(|e| {
            AppError::config(format!("Failed to expand variables in config: {}", e))
        })?;

        let config: Self = toml::from_str(&expanded_content).map_err(|e| {
            AppError::config(format!(
                "Failed to parse config file {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> AppResult<()> {
        let path = path.as_ref();

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Merge another config into this one
    pub fn merge(&mut self, other: Self) -> AppResult<()> {
        // Network config
        if other.network.rpc_url != NetworkConfig::default().rpc_url {
            self.network.rpc_url = other.network.rpc_url;
        }
        if other.network.ws_url.is_some() {
            self.network.ws_url = other.network.ws_url;
        }
        if other.network.cluster != NetworkConfig::default().cluster {
            self.network.cluster = other.network.cluster;
        }
        if other.network.timeout != NetworkConfig::default().timeout {
            self.network.timeout = other.network.timeout;
        }
        if other.network.max_retries != NetworkConfig::default().max_retries {
            self.network.max_retries = other.network.max_retries;
        }
        if other.network.commitment != NetworkConfig::default().commitment {
            self.network.commitment = other.network.commitment;
        }

        // Wallet config
        if other.wallet.keypair_path.is_some() {
            self.wallet.keypair_path = other.wallet.keypair_path;
        }
        if other.wallet.address.is_some() {
            self.wallet.address = other.wallet.address;
        }

        // App config
        if other.app.log_level != AppConfig::default().log_level {
            self.app.log_level = other.app.log_level;
        }
        if other.app.data_dir != AppConfig::default().data_dir {
            self.app.data_dir = other.app.data_dir;
        }

        // Merge custom settings
        for (key, value) in other.app.custom {
            self.app.custom.insert(key, value);
        }

        // Program config
        if other.programs.trust_escrow_v2 != ProgramConfig::default().trust_escrow_v2 {
            self.programs.trust_escrow_v2 = other.programs.trust_escrow_v2;
        }

        // Merge additional programs
        for (key, value) in other.programs.additional {
            self.programs.additional.insert(key, value);
        }

        // Fee config
        if other.fees.priority_fee != FeeConfig::default().priority_fee {
            self.fees.priority_fee = other.fees.priority_fee;
        }
        if other.fees.max_fee_bps != FeeConfig::default().max_fee_bps {
            self.fees.max_fee_bps = other.fees.max_fee_bps;
        }

        Ok(())
    }

    /// Apply environment variables
    fn apply_env_vars(&mut self) -> AppResult<()> {
        if let Ok(rpc_url) = std::env::var("TRUST_ESCROW_RPC_URL") {
            self.network.rpc_url = rpc_url;
        }

        if let Ok(cluster) = std::env::var("TRUST_ESCROW_CLUSTER") {
            self.network.cluster = cluster;
        }

        if let Ok(keypair_path) = std::env::var("TRUST_ESCROW_KEYPAIR_PATH") {
            self.wallet.keypair_path = Some(PathBuf::from(keypair_path));
        }

        if let Ok(program_id) = std::env::var("TRUST_ESCROW_PROGRAM_ID") {
            self.programs.trust_escrow_v2 = program_id;
        }

        if let Ok(log_level) = std::env::var("TRUST_ESCROW_LOG_LEVEL") {
            self.app.log_level = log_level;
        }

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> AppResult<()> {
        // Validate RPC URL
        if self.network.rpc_url.is_empty() {
            return Err(AppError::config("RPC URL cannot be empty"));
        }

        // Validate program ID format
        self.programs
            .trust_escrow_v2
            .parse::<Pubkey>()
            .map_err(|e| AppError::config(format!("Invalid program ID: {}", e)))?;

        // Validate commitment level
        match self.network.commitment.as_str() {
            "processed" | "confirmed" | "finalized" => {}
            _ => {
                return Err(AppError::config(
                    "Invalid commitment level, must be 'processed', 'confirmed', or 'finalized'",
                ))
            }
        }

        // Validate fee percentage
        if self.fees.max_fee_bps > 10000 {
            return Err(AppError::config(
                "Max fee cannot exceed 100% (10000 basis points)",
            ));
        }

        Ok(())
    }

    /// Get network preset configurations
    pub fn preset_localnet() -> Self {
        let mut config = Self::default();
        config.network.rpc_url = "http://localhost:8899".to_string();
        config.network.ws_url = Some("ws://localhost:8900".to_string());
        config.network.cluster = "localnet".to_string();
        config
    }

    pub fn preset_devnet() -> Self {
        let mut config = Self::default();
        config.network.rpc_url = "https://api.devnet.solana.com".to_string();
        config.network.ws_url = Some("wss://api.devnet.solana.com".to_string());
        config.network.cluster = "devnet".to_string();
        config
    }

    pub fn preset_mainnet() -> Self {
        let mut config = Self::default();
        config.network.rpc_url = "https://api.mainnet-beta.solana.com".to_string();
        config.network.ws_url = Some("wss://api.mainnet-beta.solana.com".to_string());
        config.network.cluster = "mainnet-beta".to_string();
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = EscrowConfig::default();
        assert_eq!(config.network.cluster, "localnet");
        assert_eq!(config.network.commitment, "confirmed");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = EscrowConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid program ID should fail
        config.programs.trust_escrow_v2 = "invalid".to_string();
        assert!(config.validate().is_err());

        // Invalid commitment should fail
        config.programs.trust_escrow_v2 = "11111111111111111111111111111112".to_string();
        config.network.commitment = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_merge() {
        let mut base = EscrowConfig::default();
        let mut other = EscrowConfig::default();
        other.network.rpc_url = "https://api.devnet.solana.com".to_string();
        other.app.log_level = "debug".to_string();

        base.merge(other).unwrap();

        assert_eq!(base.network.rpc_url, "https://api.devnet.solana.com");
        assert_eq!(base.app.log_level, "debug");
    }

    #[test]
    fn test_config_file_save_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test-config.toml");

        let mut config = EscrowConfig::default();
        config.network.cluster = "testnet".to_string();
        config.app.log_level = "trace".to_string();

        // Save and load
        config.save_to_file(&config_path).unwrap();
        let loaded = EscrowConfig::load_from_file(&config_path).unwrap();

        assert_eq!(loaded.network.cluster, "testnet");
        assert_eq!(loaded.app.log_level, "trace");
    }
}

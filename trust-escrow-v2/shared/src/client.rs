//! Client wrapper for Trust Work Escrow SDK integration
//!
//! Provides a high-level client interface that wraps the Epic #2 SDK
//! with configuration management and error handling suitable for CLI/TUI apps.

use crate::config::{EscrowConfig, WalletType};
use crate::error::{AppError, AppResult};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use trust_escrow_sdk::client::CofreClient;

/// High-level client wrapper for Trust Work Escrow operations
pub struct EscrowClient {
    /// Configuration
    config: EscrowConfig,
    
    /// SDK client instance (optional - only available with wallet)
    sdk_client: Option<CofreClient>,
    
    /// RPC client for direct Solana operations
    rpc_client: Arc<RpcClient>,
    
    /// Wallet keypair
    wallet: Option<Arc<Keypair>>,
}

impl EscrowClient {
    /// Create new client from configuration
    pub fn from_config(config: EscrowConfig) -> AppResult<Self> {
        // Create RPC client
        let commitment = CommitmentConfig::from_str(&config.network.commitment)
            .map_err(|_| AppError::config(format!("Invalid commitment: {}", config.network.commitment)))?;
        
        let rpc_client = Arc::new(RpcClient::new_with_commitment(&config.network.rpc_url, commitment));

        // Load wallet if configured
        let wallet = match config.wallet.wallet_type {
            WalletType::Filesystem => {
                if let Some(keypair_path) = &config.wallet.keypair_path {
                    Some(Arc::new(Self::load_keypair_from_file(keypair_path)?))
                } else {
                    None
                }
            }
            WalletType::Ledger => {
                return Err(AppError::config("Ledger wallet not yet supported"));
            }
            WalletType::Remote => {
                return Err(AppError::config("Remote wallet not yet supported"));
            }
        };

        // Parse program ID
        let program_id = config.programs.trust_escrow_v2
            .parse::<Pubkey>()
            .map_err(|e| AppError::config(format!("Invalid program ID: {}", e)))?;

        // Create SDK client (only if wallet is available)
        let sdk_client = if let Some(ref wallet) = wallet {
            Some(CofreClient::new(
                rpc_client.clone(),
                wallet.clone(),
            )?)
        } else {
            None
        };

        Ok(Self {
            config,
            sdk_client,
            rpc_client,
            wallet,
        })
    }

    /// Create client with default configuration
    pub fn new() -> AppResult<Self> {
        let config = EscrowConfig::load()?;
        Self::from_config(config)
    }

    /// Create client for specific network preset
    pub fn for_network(network: &str) -> AppResult<Self> {
        let config = match network {
            "localnet" => EscrowConfig::preset_localnet(),
            "devnet" => EscrowConfig::preset_devnet(),
            "mainnet-beta" => EscrowConfig::preset_mainnet(),
            _ => return Err(AppError::invalid_input(format!("Unknown network: {}", network))),
        };
        Self::from_config(config)
    }

    /// Get configuration reference
    pub fn config(&self) -> &EscrowConfig {
        &self.config
    }

    /// Get SDK client reference for direct operations
    pub fn sdk(&self) -> Option<&CofreClient> {
        self.sdk_client.as_ref()
    }

    /// Get RPC client reference
    pub fn rpc(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Get wallet keypair if available
    pub fn wallet(&self) -> Option<&Keypair> {
        self.wallet.as_ref().map(|k| k.as_ref())
    }

    /// Check if wallet is available
    pub fn has_wallet(&self) -> bool {
        self.wallet.is_some()
    }

    /// Get wallet public key if available
    pub fn wallet_pubkey(&self) -> Option<Pubkey> {
        self.wallet.as_ref().map(|k| k.pubkey())
    }

    /// Set or replace the wallet
    pub fn set_wallet(&mut self, keypair: Keypair) -> AppResult<()> {
        self.wallet = Some(Arc::new(keypair));
        // Update SDK client with new wallet
        self.sdk_client = Some(CofreClient::new(
            self.rpc_client.clone(),
            self.wallet.as_ref().unwrap().clone(),
        )?);
        Ok(())
    }

    /// Load wallet from keypair file
    pub fn load_wallet_from_file<P: AsRef<Path>>(&mut self, path: P) -> AppResult<()> {
        let keypair = Self::load_keypair_from_file(path)?;
        self.set_wallet(keypair)?;
        Ok(())
    }

    /// Check network connectivity
    pub async fn check_connection(&self) -> AppResult<()> {
        match self.rpc_client.get_version() {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError::Network(e)),
        }
    }

    /// Get current slot
    pub async fn get_slot(&self) -> AppResult<u64> {
        self.rpc_client
            .get_slot()
            .map_err(|e| AppError::Network(e))
    }

    /// Get SOL balance for wallet
    pub async fn get_wallet_balance(&self) -> AppResult<u64> {
        let wallet_pubkey = self.wallet_pubkey()
            .ok_or_else(|| AppError::missing_data("No wallet configured"))?;
        
        self.rpc_client
            .get_balance(&wallet_pubkey)
            .map_err(|e| AppError::Network(e))
    }

    /// Get SOL balance for any address
    pub async fn get_balance(&self, address: &Pubkey) -> AppResult<u64> {
        self.rpc_client
            .get_balance(address)
            .map_err(|e| AppError::Network(e))
    }

    /// Request airdrop (devnet/testnet only)
    pub async fn request_airdrop(&self, amount: u64) -> AppResult<String> {
        let wallet_pubkey = self.wallet_pubkey()
            .ok_or_else(|| AppError::missing_data("No wallet configured"))?;

        if self.config.network.cluster == "mainnet-beta" {
            return Err(AppError::operation_failed("Airdrops not available on mainnet"));
        }

        let signature = self.rpc_client
            .request_airdrop(&wallet_pubkey, amount)
            .map_err(|e| AppError::Network(e))?;

        Ok(signature.to_string())
    }

    /// Load keypair from file (internal utility)
    fn load_keypair_from_file<P: AsRef<Path>>(path: P) -> AppResult<Keypair> {
        let path = path.as_ref();
        let keypair_bytes = std::fs::read(path)
            .map_err(|e| AppError::config(format!("Failed to read keypair file {}: {}", path.display(), e)))?;

        // Try parsing as JSON array (Solana CLI format)
        if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&keypair_bytes) {
            if let Some(array) = json_value.as_array() {
                if array.len() == 64 {
                    let bytes: Result<Vec<u8>, _> = array.iter()
                        .map(|v| v.as_u64().map(|n| n as u8).ok_or("Invalid byte value"))
                        .collect();
                    
                    if let Ok(bytes) = bytes {
                        return Keypair::from_bytes(&bytes)
                            .map_err(|e| AppError::config(format!("Invalid keypair bytes: {}", e)));
                    }
                }
            }
        }

        // Try parsing as raw bytes
        if keypair_bytes.len() == 64 {
            return Keypair::from_bytes(&keypair_bytes)
                .map_err(|e| AppError::config(format!("Invalid keypair bytes: {}", e)));
        }

        Err(AppError::config("Invalid keypair file format"))
    }

    /// Validate configuration and connections
    pub async fn validate(&self) -> AppResult<Vec<String>> {
        let mut warnings = Vec::new();

        // Check network connection
        if let Err(_) = self.check_connection().await {
            warnings.push("Network connection failed".to_string());
        }

        // Check wallet
        if !self.has_wallet() {
            warnings.push("No wallet configured".to_string());
        } else if let Ok(balance) = self.get_wallet_balance().await {
            if balance == 0 {
                warnings.push("Wallet has zero SOL balance".to_string());
            }
        }

        // Check if program exists
        let program_id = self.config.programs.trust_escrow_v2.parse::<Pubkey>()
            .map_err(|e| AppError::config(format!("Invalid program ID: {}", e)))?;
            
        if let Err(_) = self.rpc_client.get_account(&program_id) {
            warnings.push(format!("Trust Escrow program not found at {}", program_id));
        }

        Ok(warnings)
    }
}

impl Default for EscrowClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_client_creation() {
        let config = EscrowConfig::default();
        let client = EscrowClient::from_config(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_network_presets() {
        let client = EscrowClient::for_network("devnet");
        assert!(client.is_ok());
        
        let devnet_client = client.unwrap();
        assert_eq!(devnet_client.config().network.cluster, "devnet");
    }

    #[test]
    fn test_keypair_loading() {
        // Create temporary keypair file
        let keypair = Keypair::new();
        let mut file = NamedTempFile::new().unwrap();
        let keypair_json = serde_json::to_string(&keypair.to_bytes().to_vec()).unwrap();
        file.write_all(keypair_json.as_bytes()).unwrap();
        
        // Test loading
        let loaded_keypair = EscrowClient::load_keypair_from_file(file.path());
        assert!(loaded_keypair.is_ok());
        
        let loaded = loaded_keypair.unwrap();
        assert_eq!(loaded.pubkey(), keypair.pubkey());
    }

    #[test]
    fn test_invalid_network() {
        let client = EscrowClient::for_network("invalid");
        assert!(client.is_err());
    }
}
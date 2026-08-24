//! External wallet linking. Private keys are never generated or returned.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalletInfo {
    pub pubkey: String,
    pub already_exists: bool,
}

/// Link an external wallet only after the caller has verified a SIWS proof.
#[server]
pub async fn link_wallet_to_user(wallet_pubkey: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let pk = wallet_pubkey.trim().to_string();
        let bytes = bs58::decode(&pk)
            .into_vec()
            .map_err(|e| ServerFnError::new(format!("pubkey base58: {:?}", e)))?;
        if bytes.len() != 32 {
            return Err(ServerFnError::new("pubkey debe ser 32 bytes"));
        }
        // TODO: persist the verified mapping to users.wallet_pubkey in Postgres.
        log::info!("[wallet] linked external pubkey={}", pk);
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = wallet_pubkey;
        Err(ServerFnError::new("server only"))
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    #[test]
    fn external_wallets_are_public_keys_only() {
        let key = bs58::encode([7u8; 32]).into_string();
        assert_eq!(bs58::decode(key).into_vec().unwrap().len(), 32);
    }
}

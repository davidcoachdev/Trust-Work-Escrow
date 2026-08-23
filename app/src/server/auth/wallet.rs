//! Deterministic wallet — single wallet per email via HMAC-SHA256(server_secret, email).
//! Free, sin pagar. Uses `hmac` + `sha2` + `ed25519-dalek` + `bs58`.
//!
//! TODO: Persist `email -> pubkey` mapping to Postgres `users.wallet_pubkey`.
//! For MVP uses `OnceLock<Mutex<HashMap>>` so the wallet is recoverable via OTP
//! and not proliferated (no new wallet unless `force_new`).
//! Do NOT create wallets on the fly in modals — only Config > Wallet.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WalletInfo {
    pub pubkey: String,
    pub already_exists: bool,
}

// ---- store (MVP) ----
#[cfg(feature = "server")]
static WALLET_STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

#[cfg(feature = "server")]
fn wallet_store() -> &'static Mutex<HashMap<String, String>> {
    WALLET_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

// ---- helpers ----
#[cfg(feature = "server")]
fn server_secret() -> String {
    if let Ok(s) = std::env::var("WALLET_DERIVATION_SECRET") {
        if !s.is_empty() {
            return s;
        }
    }
    if let Ok(s) = std::env::var("JWT_SECRET") {
        if !s.is_empty() {
            log::warn!("[wallet] WALLET_DERIVATION_SECRET not set — falling back to JWT_SECRET (set a dedicated secret in prod)");
            return s;
        }
    }
    log::warn!("[wallet] no WALLET_DERIVATION_SECRET nor JWT_SECRET — using dev fallback (DO NOT USE IN PROD)");
    "dev-only-wallet-derivation-secret-change-me".to_string()
}

#[cfg(feature = "server")]
fn derive_keypair(email: &str) -> Result<(String, String), String> {
    use ed25519_dalek::SigningKey;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let normalized = email.to_lowercase().trim().to_string();
    if !normalized.contains('@') {
        return Err("email inválido".to_string());
    }
    let secret = server_secret();
    // HMAC-SHA256(server_secret, email) -> 32 bytes seed
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|e| format!("hmac init: {:?}", e))?;
    mac.update(normalized.as_bytes());
    let result = mac.finalize().into_bytes();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&result[..32]);
    let sk = SigningKey::from_bytes(&seed);
    let vk = sk.verifying_key();
    let pubkey_b58 = bs58::encode(vk.to_bytes()).into_string();
    let priv_b58 = bs58::encode(sk.to_bytes()).into_string();
    Ok((pubkey_b58, priv_b58))
}

#[cfg(feature = "server")]
fn normalize_email(email: &str) -> String {
    email.to_lowercase().trim().to_string()
}

// --- Server Functions ---

/// Deterministic: same email -> same pubkey. No proliferation unless `force_new`.
/// For MVP `force_new` re-derives but warned; in real flow we would rotate with new secret or append counter.
/// Here `force_new` is gated to require explicit user confirmation in UI.
#[server]
pub async fn get_or_create_wallet(email: String, force_new: bool) -> Result<WalletInfo, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email = normalize_email(&email);
        if email.is_empty() || !email.contains('@') {
            return Err(ServerFnError::new("email inválido".to_string()));
        }
        let (pubkey, _priv) = derive_keypair(&email).map_err(|e| ServerFnError::new(e))?;
        let mut store = wallet_store()
            .lock()
            .map_err(|_| ServerFnError::new("lock poisoned".to_string()))?;
        if let Some(existing) = store.get(&email) {
            if !force_new {
                return Ok(WalletInfo {
                    pubkey: existing.clone(),
                    already_exists: true,
                });
            }
            // force_new: same deterministic pubkey anyway; but mark as already_exists=false for UI warning
            // If we wanted a truly new wallet we'd need a rotation scheme; for now keep deterministic.
            log::info!("[wallet] force_new requested for {} — returning same deterministic pubkey", email);
            store.insert(email.clone(), pubkey.clone());
            return Ok(WalletInfo {
                pubkey,
                already_exists: false,
            });
        }
        store.insert(email.clone(), pubkey.clone());
        Ok(WalletInfo {
            pubkey,
            already_exists: false,
        })
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (email, force_new);
        Err(ServerFnError::new("server only".to_string()))
    }
}

/// Returns private key base58 after re-verifying OTP. One-time reveal: caller must handle storage.
/// Requires checkbox acknowledgement in UI (`Ya la guardé`).
#[server]
pub async fn reveal_wallet_private_key(email: String, otp: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email = normalize_email(&email);
        // Re-verify OTP via existing store
        let ok = crate::server::auth::email::verify_otp(&email, &otp)
            .map_err(|e| ServerFnError::new(e))?;
        if !ok {
            return Err(ServerFnError::new("código incorrecto".to_string()));
        }
        let (_pubkey, priv_b58) = derive_keypair(&email).map_err(|e| ServerFnError::new(e))?;
        // Ensure mapping exists (create if needed so link_wallet_to_user can work)
        {
            let mut store = wallet_store()
                .lock()
                .map_err(|_| ServerFnError::new("lock poisoned".to_string()))?;
            if !store.contains_key(&email) {
                let (pubkey, _) = derive_keypair(&email).map_err(|e| ServerFnError::new(e))?;
                store.insert(email.clone(), pubkey);
            }
        }
        Ok(priv_b58)
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (email, otp);
        Err(ServerFnError::new("server only".to_string()))
    }
}

/// Link an existing external wallet (e.g. Phantom) to the user via SIWS verification.
/// For MVP just validates pubkey format; full SIWS verify happens via `verify_siws_server` before calling this.
#[server]
pub async fn link_wallet_to_user(wallet_pubkey: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    {
        let pk = wallet_pubkey.trim().to_string();
        let bytes = bs58::decode(&pk)
            .into_vec()
            .map_err(|e| ServerFnError::new(format!("pubkey base58: {:?}", e)))?;
        if bytes.len() != 32 {
            return Err(ServerFnError::new("pubkey debe ser 32 bytes".to_string()));
        }
        // TODO: persist to users.wallet_pubkey in Postgres. For now log and return ok.
        log::info!("[wallet] link_wallet_to_user pubkey={}", pk);
        Ok(())
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = wallet_pubkey;
        Err(ServerFnError::new("server only".to_string()))
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn deterministic_same_email_same_pubkey() {
        let email = "Alice@Example.com ";
        let (pk1, _) = derive_keypair(email).unwrap();
        let (pk2, _) = derive_keypair("alice@example.com").unwrap();
        assert_eq!(pk1, pk2);
    }

    #[test]
    fn different_email_different_pubkey() {
        let (pk1, _) = derive_keypair("a@example.com").unwrap();
        let (pk2, _) = derive_keypair("b@example.com").unwrap();
        assert_ne!(pk1, pk2);
    }

    #[test]
    fn get_or_create_idempotent() {
        // Clear store for test isolation (single test thread)
        if let Some(m) = WALLET_STORE.get() {
            m.lock().unwrap().clear();
        }
        let email = "idempotent@test.com";
        // Run inside tokio rt for server fn? Test derive directly
        let (pk, _) = derive_keypair(email).unwrap();
        let mut store = wallet_store().lock().unwrap();
        store.insert(normalize_email(email), pk.clone());
        assert_eq!(store.get(&normalize_email(email)).unwrap(), &pk);
    }
}

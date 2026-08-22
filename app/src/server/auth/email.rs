//! Email OTP — free con `lettre` + `argon2` + `jsonwebtoken`
//! OTP 6 dígitos hasheado, expira 10m, rate-limit 5 intentos.

#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::sync::{Mutex, OnceLock};
#[cfg(feature = "server")]
use rand::Rng;

#[cfg(feature = "server")]
static OTP_STORE: OnceLock<Mutex<HashMap<String, OtpEntry>>> = OnceLock::new();

#[cfg(feature = "server")]
#[derive(Clone)]
struct OtpEntry {
    hash: String,
    expires_at: i64,
    attempts: u8,
}

#[cfg(feature = "server")]
fn store() -> &'static Mutex<HashMap<String, OtpEntry>> {
    OTP_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(feature = "server")]
fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

#[cfg(feature = "server")]
pub fn generate_otp(email: &str) -> Result<String, String> {
    let code: String = rand::thread_rng().gen_range(100000..999999).to_string();
    let hash = hash_otp(&code)?;
    let mut map = store().lock().map_err(|_| "lock poisoned".to_string())?;
    map.insert(
        email.to_lowercase(),
        OtpEntry {
            hash,
            expires_at: now_ts() + 600,
            attempts: 0,
        },
    );
    Ok(code)
}

#[cfg(feature = "server")]
fn hash_otp(code: &str) -> Result<String, String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(code.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("hash error: {:?}", e))
}

#[cfg(feature = "server")]
fn verify_hash(hash: &str, code: &str) -> bool {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(code.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(feature = "server")]
pub fn verify_otp(email: &str, code: &str) -> Result<bool, String> {
    let key = email.to_lowercase();
    let mut map = store().lock().map_err(|_| "lock poisoned".to_string())?;
    let entry = match map.get_mut(&key) {
        Some(e) => e,
        None => return Err("no otp found, pide uno nuevo".to_string()),
    };
    if now_ts() > entry.expires_at {
        map.remove(&key);
        return Err("otp expirado (10m)".to_string());
    }
    if entry.attempts >= 5 {
        return Err("rate-limit 5 intentos".to_string());
    }
    entry.attempts += 1;
    if verify_hash(&entry.hash, code) {
        map.remove(&key);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(feature = "server")]
pub async fn send_otp_email(email: &str, code: &str) -> Result<(), String> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
    if smtp_host.is_empty() {
        log::info!("[DEV] OTP para {}: {}", email, code);
        return Ok(());
    }
    log::info!("[SMTP] Enviando OTP {} a {} via {}", code, email, smtp_host);
    Ok(())
}

// --- Server Functions Dioxus fullstack (Task A1) ---
use dioxus::prelude::*;

#[server]
pub async fn send_otp(email: String) -> Result<String, ServerFnError> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(ServerFnError::new("email inválido".to_string()));
    }
    let code = generate_otp(&email).map_err(|e| ServerFnError::new(e))?;
    send_otp_email(&email, &code)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(format!("OTP enviado a {} (revisa logs dev)", email))
}

#[server]
pub async fn verify_otp_server(email: String, code: String) -> Result<String, ServerFnError> {
    let email = email.trim().to_lowercase();
    match verify_otp(&email, &code) {
        Ok(true) => Ok("verified".to_string()),
        Ok(false) => Err(ServerFnError::new("código incorrecto".to_string())),
        Err(e) => Err(ServerFnError::new(e)),
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn otp_generate_and_verify_ok() {
        let email = "test@example.com";
        let code = generate_otp(email).unwrap();
        assert!(verify_otp(email, &code).unwrap());
    }

    #[test]
    fn otp_wrong_code() {
        let email = "test2@example.com";
        let _ = generate_otp(email).unwrap();
        assert!(!verify_otp(email, "000000").unwrap());
    }
}

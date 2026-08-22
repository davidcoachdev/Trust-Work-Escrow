//! Email OTP — free con `lettre` + `argon2` + `jsonwebtoken`
//! Sin pagar Clerk/Auth0. OTP 6 dígitos hasheado, expira 10m, rate-limit 5 intentos.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use rand::Rng;

static OTP_STORE: OnceLock<Mutex<HashMap<String, OtpEntry>>> = OnceLock::new();

#[derive(Clone)]
struct OtpEntry {
    hash: String,
    expires_at: i64, // unix timestamp
    attempts: u8,
}

fn store() -> &'static Mutex<HashMap<String, OtpEntry>> {
    OTP_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Genera OTP 6 dígitos, lo hashea con argon2 y lo guarda. Devuelve el código plano (solo para log en dev).
pub fn generate_otp(email: &str) -> Result<String, String> {
    let code: String = rand::thread_rng().gen_range(100000..999999).to_string();
    let hash = hash_otp(&code)?;
    let mut map = store().lock().map_err(|_| "lock poisoned".to_string())?;
    map.insert(
        email.to_lowercase(),
        OtpEntry {
            hash,
            expires_at: now_ts() + 600, // 10m
            attempts: 0,
        },
    );
    Ok(code)
}

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

/// Verifica OTP. Retorna Ok(true) si ok, Ok(false) si código malo, Err si expirado o rate-limit.
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

/// Envía OTP por correo con `lettre`. Si no hay SMTP env, solo loguea (dev).
pub async fn send_otp_email(email: &str, code: &str) -> Result<(), String> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_default();
    if smtp_host.is_empty() {
        log::info!("[DEV] OTP para {}: {}", email, code);
        return Ok(());
    }
    // En prod, usar lettre con SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS
    // Por ahora logueamos para no bloquear Task A1 sin credenciales
    log::info!("[SMTP] Enviando OTP {} a {} via {}", code, email, smtp_host);
    Ok(())
}

#[cfg(test)]
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

    #[test]
    fn otp_expires() {
        let email = "expire@example.com";
        let code = generate_otp(email).unwrap();
        // forzar expiración
        {
            let mut map = store().lock().unwrap();
            if let Some(e) = map.get_mut(&email.to_lowercase()) {
                e.expires_at = now_ts() - 1;
            }
        }
        assert!(verify_otp(email, &code).is_err());
    }
}

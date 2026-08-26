//! Persistent user server fns — **backend/api via reqwest** (no direct Postgres).
//! `app` never touches `PgPool`/`DATABASE_URL`; all persistence goes through
//! `crate::server::auth::api_client` → `http://api:3000/users/*`.
//! Keeps MVP OTP bypass and client/freelancer role selector intact.

use dioxus::prelude::*;
use crate::server::auth::guest::User;

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_role(role: &str) -> Result<String, String> {
    let r = role.trim().to_lowercase();
    match r.as_str() {
        "client" | "freelancer" => Ok(r),
        "admin" | "arbiter" | "guest" => Ok(r),
        _ => Err(format!("role must be client or freelancer (got '{}')", role)),
    }
}

fn validate_email(email: &str) -> Result<String, String> {
    let e = normalize_email(email);
    if e.is_empty() || !e.contains('@') || !e.contains('.') {
        return Err("email inválido".to_string());
    }
    if e.len() > 320 {
        return Err("email demasiado largo".to_string());
    }
    Ok(e)
}

/// Create or update a user with the given role. Delegates to `backend/api`.
/// Keeps wallet if already set. `is_guest` is false for real users.
#[server]
pub async fn login_or_create_user(email: String, role: String) -> Result<User, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email_n = validate_email(&email).map_err(|e| ServerFnError::new(e))?;
        let role_n = validate_role(&role).map_err(|e| ServerFnError::new(e))?;
        crate::server::auth::api_client::api_login_or_create(&email_n, &role_n)
            .await
            .map_err(|e| ServerFnError::new(e))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (email, role);
        Err(ServerFnError::new("server only"))
    }
}

#[server]
pub async fn get_user_by_email_server(email: String) -> Result<Option<User>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email_n = normalize_email(&email);
        if email_n.is_empty() {
            return Ok(None);
        }
        crate::server::auth::api_client::api_get_user_by_email(&email_n)
            .await
            .map_err(|e| ServerFnError::new(e))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = email;
        Err(ServerFnError::new("server only"))
    }
}

/// Persist wallet pubkey for the given email. Delegates to `backend/api`.
/// Backend validates 32-byte bs58.
#[server]
pub async fn link_wallet_persist(email: String, wallet_pubkey: String) -> Result<User, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email_n = validate_email(&email).map_err(|e| ServerFnError::new(e))?;
        let pk = wallet_pubkey.trim().to_string();
        if pk.is_empty() {
            return Err(ServerFnError::new("wallet pubkey vacío"));
        }
        crate::server::auth::api_client::api_link_wallet(&email_n, &pk)
            .await
            .map_err(|e| ServerFnError::new(e))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = (email, wallet_pubkey);
        Err(ServerFnError::new("server only"))
    }
}

/// Clear wallet (disconnect) — delegates to `backend/api`.
#[server]
pub async fn unlink_wallet_persist(email: String) -> Result<User, ServerFnError> {
    #[cfg(feature = "server")]
    {
        let email_n = validate_email(&email).map_err(|e| ServerFnError::new(e))?;
        crate::server::auth::api_client::api_unlink_wallet(&email_n)
            .await
            .map_err(|e| ServerFnError::new(e))
    }
    #[cfg(not(feature = "server"))]
    {
        let _ = email;
        Err(ServerFnError::new("server only"))
    }
}

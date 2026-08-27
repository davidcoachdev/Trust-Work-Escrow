//! App server → backend/api HTTP client for `users` / `wallet`.
//! `app` never touches Postgres; all persistence goes through `backend/api`
//! via `API_INTERNAL_URL` + `API_SERVICE_TOKEN`.
//!
//! Mirrors `backend/sdk/src/users.rs` logic but stays inside `app` crate so
//! `trust-escrow-sdk` feature is not required on the frontend. The SDK client
//! (`TrustEscrowApiClient`) is still the canonical one; this module keeps the
//! same wire format so switching via feature flag is trivial.

use crate::server::auth::guest::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiUser {
    email: String,
    role: String,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    permissions: Vec<String>,
    wallet_pubkey: Option<String>,
    is_guest: bool,
    #[serde(default)]
    created_at: i64,
    #[serde(default)]
    updated_at: i64,
    #[serde(default)]
    is_active: bool,
}

fn api_base() -> String {
    std::env::var("API_INTERNAL_URL")
        .or_else(|_| std::env::var("API_URL"))
        .unwrap_or_else(|_| "http://api:3000".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn service_token() -> Option<String> {
    std::env::var("API_SERVICE_TOKEN")
        .ok()
        .filter(|v| !v.trim().is_empty())
}

#[cfg(feature = "server")]
fn headers() -> reqwest::header::HeaderMap {
    let mut h = reqwest::header::HeaderMap::new();
    if let Some(t) = service_token() {
        if let Ok(v) = reqwest::header::HeaderValue::from_str(&t) {
            h.insert("x-service-token", v);
        }
    }
    h.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    h
}

fn to_user(u: ApiUser) -> User {
    let roles = if !u.roles.is_empty() { u.roles.clone() } else if !u.role.trim().is_empty() { vec![u.role.clone()] } else { vec!["guest".to_string()] };
    let role = if u.role.trim().is_empty() { roles.first().cloned().unwrap_or_else(|| "guest".to_string()) } else { u.role.clone() };
    User {
        email: u.email.clone(),
        role,
        roles,
        permissions: u.permissions.clone(),
        wallet_pubkey: u.wallet_pubkey.filter(|s| !s.trim().is_empty()),
        is_guest: u.is_guest,
        created_at: u.created_at,
        updated_at: u.updated_at,
        is_active: true,
    }
}

fn encode_email(email: &str) -> String {
    let mut out = String::with_capacity(email.len() * 3);
    for b in email.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(feature = "server")]
pub async fn api_login_or_create(email: &str, role: &str) -> Result<User, String> {
    let url = format!("{}/users/login-or-create", api_base());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("reqwest client: {e}"))?;
    let body = serde_json::json!({ "email": email, "role": role });
    let resp = client
        .post(&url)
        .headers(headers())
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("api login_or_create request failed ({}): {}", url, e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("api login_or_create {}: {}", status, text));
    }
    let api_user = resp
        .json::<ApiUser>()
        .await
        .map_err(|e| format!("decode user: {e}"))?;
    Ok(to_user(api_user))
}

#[cfg(feature = "server")]
pub async fn api_get_user_by_email(email: &str) -> Result<Option<User>, String> {
    let url = format!("{}/users/{}", api_base(), encode_email(email));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("reqwest client: {e}"))?;
    let resp = client
        .get(&url)
        .headers(headers())
        .send()
        .await
        .map_err(|e| format!("api get_user request failed ({}): {}", url, e))?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("api get_user {}: {}", status, text));
    }
    let api_user = resp
        .json::<ApiUser>()
        .await
        .map_err(|e| format!("decode user: {e}"))?;
    Ok(Some(to_user(api_user)))
}

#[cfg(feature = "server")]
pub async fn api_link_wallet(email: &str, wallet_pubkey: &str) -> Result<User, String> {
    let url = format!("{}/users/{}/wallet", api_base(), encode_email(email));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("reqwest client: {e}"))?;
    let body = serde_json::json!({ "wallet_pubkey": wallet_pubkey });
    let resp = client
        .post(&url)
        .headers(headers())
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("api link_wallet request failed ({}): {}", url, e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("api link_wallet {}: {}", status, text));
    }
    let api_user = resp
        .json::<ApiUser>()
        .await
        .map_err(|e| format!("decode user: {e}"))?;
    Ok(to_user(api_user))
}

#[cfg(feature = "server")]
pub async fn api_unlink_wallet(email: &str) -> Result<User, String> {
    let url = format!("{}/users/{}/wallet", api_base(), encode_email(email));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("reqwest client: {e}"))?;
    let resp = client
        .delete(&url)
        .headers(headers())
        .send()
        .await
        .map_err(|e| format!("api unlink_wallet request failed ({}): {}", url, e))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("api unlink_wallet {}: {}", status, text));
    }
    let api_user = resp
        .json::<ApiUser>()
        .await
        .map_err(|e| format!("decode user: {e}"))?;
    Ok(to_user(api_user))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWallet {
    pub email: String,
    pub pubkey: String,
    pub purpose: String,
    pub label: Option<String>,
    pub created_at: i64,
    pub is_active: bool,
}

#[cfg(feature = "server")]
pub async fn api_list_wallets(email: &str) -> Result<Vec<ApiWallet>, String> {
    let url = format!("{}/users/{}/wallets", api_base(), encode_email(email));
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).headers(headers()).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(format!("api list_wallets {}: {}", s, t));
    }
    resp.json::<Vec<ApiWallet>>().await.map_err(|e| e.to_string())
}

#[cfg(feature = "server")]
pub async fn api_add_wallet(email: &str, pubkey: &str, purpose: &str, label: Option<String>) -> Result<ApiWallet, String> {
    let url = format!("{}/users/{}/wallets", api_base(), encode_email(email));
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let body = serde_json::json!({"pubkey": pubkey, "purpose": purpose, "label": label});
    let resp = client.post(&url).headers(headers()).json(&body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(format!("api add_wallet {}: {}", s, t));
    }
    resp.json::<ApiWallet>().await.map_err(|e| e.to_string())
}

#[cfg(feature = "server")]
pub async fn api_remove_wallet(email: &str, pubkey: &str) -> Result<(), String> {
    let url = format!("{}/users/{}/wallets/{}", api_base(), encode_email(email), pubkey.trim());
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let resp = client.delete(&url).headers(headers()).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(format!("api remove_wallet {}: {}", s, t));
    }
    Ok(())
}

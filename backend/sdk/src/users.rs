//! SDK wallet helpers — mirror `backend/api` `/users/:email/wallets` endpoints.
//! Used by `app` server fns; pure `reqwest` without Solana deps.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletEntry {
    pub email: String,
    pub pubkey: String,
    pub purpose: String,
    pub label: Option<String>,
    pub created_at: i64,
    pub is_active: bool,
}

fn api_base() -> String {
    std::env::var("API_INTERNAL_URL")
        .or_else(|_| std::env::var("API_URL"))
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn service_token() -> Option<String> {
    std::env::var("API_SERVICE_TOKEN").ok().filter(|v| !v.trim().is_empty())
}

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

fn encode_email(email: &str) -> String {
    let mut out = String::with_capacity(email.len()*3);
    for b in email.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub async fn list_wallets(email: &str) -> Result<Vec<WalletEntry>, String> {
    let url = format!("{}/users/{}/wallets", api_base(), encode_email(email));
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).headers(headers()).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(format!("list_wallets {}: {}", s, t));
    }
    resp.json::<Vec<WalletEntry>>().await.map_err(|e| e.to_string())
}

pub async fn add_wallet(email: &str, pubkey: &str, purpose: &str, label: Option<String>) -> Result<WalletEntry, String> {
    let url = format!("{}/users/{}/wallets", api_base(), encode_email(email));
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let body = serde_json::json!({"pubkey": pubkey, "purpose": purpose, "label": label});
    let resp = client.post(&url).headers(headers()).json(&body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(format!("add_wallet {}: {}", s, t));
    }
    resp.json::<WalletEntry>().await.map_err(|e| e.to_string())
}

pub async fn remove_wallet(email: &str, pubkey: &str) -> Result<(), String> {
    let encoded_pubkey = pubkey.trim().to_string();
    let url = format!("{}/users/{}/wallets/{}", api_base(), encode_email(email), encoded_pubkey);
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().map_err(|e| e.to_string())?;
    let resp = client.delete(&url).headers(headers()).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(format!("remove_wallet {}: {}", s, t));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encode_email_ok() {
        assert_eq!(encode_email("a@b.com"), "a%40b.com");
    }
}

//! Auth context + guest cookie + JWT helpers.
//! Provides `User` / `AuthContext` for client hydration and JWT verification.
//! Guest: httpOnly `twe-guest` (24h, random id). No DB for MVP.
//! JWT: `twe-jwt` verified via `jsonwebtoken`.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct User {
    pub email: String,
    pub wallet_pubkey: Option<String>,
    pub role: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub is_guest: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub is_active: bool,
}

impl User {
    pub fn normalized_roles(&self) -> Vec<String> {
        if !self.roles.is_empty() {
            self.roles.iter().map(|r| r.trim().to_lowercase()).collect()
        } else if !self.role.trim().is_empty() {
            vec![self.role.trim().to_lowercase()]
        } else {
            vec!["guest".to_string()]
        }
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        has_wildcard(&self.permissions, perm)
    }
}

pub fn has_wildcard(perms: &[String], required: &str) -> bool {
    for p in perms {
        if p == required {
            return true;
        }
        if p.ends_with(":*") {
            let prefix = &p[..p.len() - 1];
            if required.starts_with(prefix) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuConfig {
    pub roles: Vec<String>,
    pub perms: Vec<String>,
}

impl MenuConfig {
    pub fn new(roles: Vec<String>, perms: Vec<String>) -> Self {
        Self { roles, perms }
    }
    pub fn from_user(user: &User) -> Self {
        Self {
            roles: user.normalized_roles(),
            perms: user.permissions.clone(),
        }
    }
    pub fn has(&self, required: &str) -> bool {
        has_wildcard(&self.perms, required)
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }
}

#[derive(Clone, Copy)]
pub struct AuthContext {
    pub user: Signal<Option<User>>,
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>()
}

// For hydration: try to provide a default guest if no provider yet (SSR safe)
pub fn use_auth_opt() -> Option<AuthContext> {
    try_use_context::<AuthContext>()
}

// --- JWT verify (server) ---

#[cfg(feature = "server")]
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // email or guest id
    pub exp: usize,
    pub role: Option<String>,
    pub wallet_pubkey: Option<String>,
}

#[cfg(feature = "server")]
pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-jwt-secret-change-me".to_string());
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let validation = jsonwebtoken::Validation::default();
    jsonwebtoken::decode::<Claims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(|e| format!("jwt: {:?}", e))
}

#[cfg(feature = "server")]
pub fn create_guest_id() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}

// Server fn to hydrate auth state from cookies
#[server]
pub async fn get_me() -> Result<Option<User>, ServerFnError> {
    #[cfg(feature = "server")]
    {
        // Try to read cookies via axum extraction if available.
        // Dioxus fullstack exposes cookies via `axum_extra::extract::CookieJar` through `extract()`.
        // We attempt best-effort extraction without failing if not available (fallback to guest).
        // For now, since we don't have axum-extra extraction wired, we return guest and rely on
        // middleware to set `twe-guest`. Client will hydrate to guest; after login, JWT will be set and
        // this fn will return the verified user once cookie parsing is wired to Postgres.
        //
        // TODO: wire `CookieJar` extraction:
        //   let jar: axum_extra::extract::CookieJar = dioxus::prelude::extract().await
        //       .map_err(|e| ServerFnError::new(format!("extract cookie: {:?}", e)))?;
        //   if let Some(jwt) = jar.get("twe-jwt") { ... verify ... }
        //   if let Some(guest) = jar.get("twe-guest") { return guest user }
        // For MVP we simulate guest.

        // Attempt JWT verification if env has a token passed via header injection (future)
        // Without cookie extraction, always return guest for now.
        Ok(Some(User {
            email: "invitado@guest.local".to_string(),
            wallet_pubkey: None,
            role: "guest".to_string(),
            roles: vec!["guest".to_string()],
            permissions: vec![],
            is_guest: true,
            created_at: 0,
            updated_at: 0,
            is_active: true,
        }))
    }
    #[cfg(not(feature = "server"))]
    {
        Ok(None)
    }
}

/// Helper to check if a request should be treated as guest read-only.
#[cfg(feature = "server")]
pub fn is_mutating_method(method: &str) -> bool {
    matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
}

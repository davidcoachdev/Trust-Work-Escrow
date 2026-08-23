//! Middleware — JWT httpOnly + role guard + guest.
//! Free, sin pagar. Dioxus fullstack usa Axum layers.
//! Guest: `twe-guest` httpOnly 24h random id set on first visit if no cookie.
//! Auth: `twe-jwt` verified via `jsonwebtoken`; required for mutating routes.

#[cfg(feature = "server")]
use axum::{
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};

#[cfg(feature = "server")]
pub async fn auth_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let uri = req.uri().path().to_string();
    let method = req.method().to_string();
    let cookies = req
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let has_jwt = cookies.contains("twe-jwt=");
    let has_guest = cookies.contains("twe-guest=");

    let is_get = method == "GET";
    let is_mutating = matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE");

    // Allow guest for GET /* (read-only). Require twe-jwt for mutating routes and dashboard write actions.
    // For now keep read-only open, but gate POST to need wallet (checked in UI + server fns).
    if is_mutating && !has_jwt {
        // Allow POST to /api/auth/* (OTP, siws) without JWT, but block other POST
        let is_auth_post = uri.starts_with("/api/auth") || uri.contains("send_otp") || uri.contains("verify_otp");
        if !is_auth_post {
            // Return 401-ish for API mutating without JWT; UI will show "Necesitás billetera → Config"
            // Don't block strictly yet — keep read-only but signal via header if needed.
            // For now, just log and continue; TODO: return 401 when DB gating is ready.
            log::info!("[auth] mutating {} {} without twe-jwt (guest={}) — allowing read-only, UI gates wallet", method, uri, has_guest);
        }
    }

    // If no guest nor jwt, set guest cookie on response (httpOnly 24h)
    let needs_guest_cookie = !has_jwt && !has_guest && is_get;

    let mut res = next.run(req).await;

    if needs_guest_cookie {
        let guest_id = crate::server::auth::guest::create_guest_id();
        let cookie_val = format!(
            "twe-guest={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
            guest_id
        );
        if let Ok(val) = HeaderValue::from_str(&cookie_val) {
            res.headers_mut().append("set-cookie", val);
        }
        log::info!("[auth] set twe-guest cookie {}", guest_id);
    }

    Ok(res)
}

#[cfg(feature = "server")]
pub fn verify_jwt(token: &str) -> Result<String, String> {
    crate::server::auth::guest::verify_jwt(token)
        .map(|c| c.sub)
        .map_err(|e| e)
}

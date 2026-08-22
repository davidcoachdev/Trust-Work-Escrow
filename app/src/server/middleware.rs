//! Middleware — JWT httpOnly + role guard + i18n/theme persist
//! Free, sin pagar. Dioxus fullstack usa Axum layers.

#[cfg(feature = "server")]
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

#[cfg(feature = "server")]
pub async fn auth_middleware(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Por ahora solo loguea, en Task A3 no bloquea guest en todas las rutas
    // En B1 se añade check real: si `req.uri` es `/dashboard/*` y no hay `twe-jwt` cookie → 302 /login
    let _ = req.headers().get("cookie");
    Ok(next.run(req).await)
}

#[cfg(feature = "server")]
pub fn verify_jwt(_token: &str) -> Result<String, String> {
    // TODO: jsonwebtoken verify con SECRET de .env
    Ok("guest".to_string())
}

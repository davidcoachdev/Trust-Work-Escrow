use axum::{http::HeaderMap, response::IntoResponse, Json};
use serde_json::Value;

pub async fn whatsapp_webhook(headers: HeaderMap, Json(payload): Json<Value>) -> impl IntoResponse {
    // HMAC check con WHATSAPP_WEBHOOK_SECRET (free)
    let _sig = headers.get("x-hub-signature-256");
    // Por ahora solo loguea, en Fase F se guarda en Mongo y hace broadcast
    log::info!("[webhook] whatsapp: {}", payload);
    Json(serde_json::json!({"ok": true}))
}

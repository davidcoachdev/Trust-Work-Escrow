//! SEO structured data helpers — Organization + WebSite + FAQPage.
//! En Dioxus SPA el JSON-LD se inyecta estático en index.html (SSR-friendly).
//! Este módulo expone las constantes para reuso y un componente opcional
//! que inyecta vía dangerous_inner_html si se necesita hidratación dinámica.

use dioxus::prelude::*;

pub const ORG_JSON_LD: &str = r#"{"@context":"https://schema.org","@type":"Organization","name":"Trust Work Escrow","url":"https://trustworkescrow.com","logo":"https://trustworkescrow.com/assets/favicon.svg","description":"Escrow descentralizado en Solana para pagos seguros entre empleadores y freelancers.","sameAs":["https://github.com/trust-work-escrow"]}"#;

pub const WEBSITE_JSON_LD: &str = r#"{"@context":"https://schema.org","@type":"WebSite","name":"Trust Work Escrow","url":"https://trustworkescrow.com","inLanguage":["es","en"],"description":"Pagos seguros entre empleadores y freelancers, liquidados on-chain con Solana."}"#;

/// Inyecta Organization + WebSite JSON-LD dinámicamente (opcional).
/// Preferir el bloque estático en index.html; usar este componente solo si
/// se necesita actualización runtime (ej. cambio de idioma).
#[component]
pub fn SeoSchema() -> Element {
    rsx! {
        script { r#type: "application/ld+json", dangerous_inner_html: ORG_JSON_LD }
        script { r#type: "application/ld+json", dangerous_inner_html: WEBSITE_JSON_LD }
    }
}

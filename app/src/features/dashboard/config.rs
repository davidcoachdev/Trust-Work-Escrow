//! Wallet configuration. User keys remain exclusively inside Phantom.

use dioxus::prelude::*;

use crate::server::auth::guest::use_auth;
use crate::server::auth::siws::{request_siws_challenge, verify_siws_server};
use crate::solana::phantom;

fn short_pubkey(value: &str) -> String {
    if value.len() <= 12 {
        value.to_string()
    } else {
        format!("{}…{}", &value[..6], &value[value.len() - 4..])
    }
}

#[component]
pub fn ConfigPage() -> Element {
    let mut auth = use_auth();
    let mut status = use_signal(|| String::new());
    let connected = auth
        .user
        .read()
        .as_ref()
        .and_then(|user| user.wallet_pubkey.clone());
    let display = connected.as_deref().map(short_pubkey);

    rsx! {
        div { class: "space-y-6 max-w-2xl",
            h1 { class: "text-3xl font-bold text-primary", "Configuración" }
            div { class: "bg-surface border border-border rounded-2xl p-6 space-y-4",
                h2 { class: "text-lg font-bold", "Wallet Solana" }
                p { class: "text-sm text-muted", "Conectá Phantom para firmar mensajes y transacciones. Trust Work nunca recibe ni almacena tu clave privada." }
                if let Some(pubkey) = connected {
                    div { class: "bg-bg border border-border rounded-xl p-4 space-y-2",
                        div { class: "text-xs text-muted", "Phantom conectada" }
                        div { class: "font-mono text-sm text-primary break-all", "{display.as_deref().unwrap_or_default()}" }
                        div { class: "text-xs text-muted break-all", "{pubkey}" }
                    }
                    button {
                        class: "bg-bg border border-border rounded-xl px-4 py-2 text-sm",
                        r#type: "button",
                        onclick: move |_| {
                            let mut user = auth.user.read().clone();
                            if let Some(ref mut user) = user { user.wallet_pubkey = None; }
                            auth.user.set(user);
                            status.set("Phantom desconectada de esta sesión.".to_string());
                        },
                        "Desconectar"
                    }
                } else {
                    button {
                        class: "bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium hover:opacity-90",
                        r#type: "button",
                        onclick: move |_| {
                            let mut auth = auth;
                            spawn(async move {
                                status.set("Esperando aprobación en Phantom…".to_string());
                                match phantom::connect().await {
                                    Ok(pubkey) => {
                                        match request_siws_challenge(pubkey.clone()).await {
                                            Ok(message) => match phantom::sign_message(&message).await {
                                            Ok(signature) => match verify_siws_server(pubkey.clone(), message, signature).await {
                                                Ok(_) => {
                                                    let mut user = auth.user.read().clone().unwrap_or_default();
                                                    user.wallet_pubkey = Some(pubkey.clone());
                                                    auth.user.set(Some(user));
                                                    status.set(format!("Phantom conectada: {}", short_pubkey(&pubkey)));
                                                }
                                                Err(_) => status.set("Phantom firmó, pero la verificación falló.".to_string()),
                                            },
                                            Err(error) => status.set(error),
                                            },
                                            Err(_) => status.set("No se pudo obtener un challenge SIWS del backend.".to_string()),
                                        }
                                    }
                                    Err(error) => status.set(error),
                                }
                            });
                        },
                        "Conectar Phantom"
                    }
                }
                if !status.read().is_empty() {
                    p { class: "text-sm text-primary", "{status.read()}" }
                }
            }
        }
    }
}

//! Wallet configuration. User keys remain exclusively inside Phantom.
//! Multi-wallet: list, add with purpose (publish|apply|general), remove with WalletHasActiveJob guard.
//! Picker auto(1)/select(2+) + insufficient-funds banner handled in dashboard create/apply.

use dioxus::prelude::*;

use crate::features::wallet::CreateWalletCard;
use crate::server::auth::guest::use_auth;
use crate::server::auth::siws::{request_siws_challenge, verify_siws_server};
use crate::server::auth::users::{add_wallet_persist, list_wallets_persist, remove_wallet_persist};
use crate::server::auth::api_client::ApiWallet;
use crate::solana::phantom;

fn short_pubkey(value: &str) -> String {
    if value.len() <= 12 {
        value.to_string()
    } else {
        format!("{}…{}", &value[..6], &value[value.len() - 4..])
    }
}

fn purpose_badge(p: &str) -> String {
    match p {
        "publish" => "Publicar".to_string(),
        "apply" => "Postular".to_string(),
        _ => "General".to_string(),
    }
}

#[component]
pub fn ConfigPage() -> Element {
    let mut auth = use_auth();
    let mut status = use_signal(|| String::new());
    let mut wallets = use_signal(|| Vec::<ApiWallet>::new());
    let mut loading = use_signal(|| true);
    let mut new_pubkey = use_signal(|| String::new());
    let mut new_purpose = use_signal(|| "publish".to_string());
    let mut removing = use_signal(|| None::<String>);

    let connected = auth.user.read().as_ref().and_then(|u| u.wallet_pubkey.clone());
    let display = connected.as_deref().map(short_pubkey);
    let email = auth.user.read().as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let email_for_phantom = email.clone();
    let email_for_list_refresh = email.clone();
    let email_for_add_btn = email.clone();
    let email_for_remove_list = email.clone();

    // load wallets on mount / when email changes
    let email_for_load = email.clone();
    use_effect(move || {
        let email_c = email_for_load.clone();
        spawn(async move {
            if email_c.is_empty() || !email_c.contains('@') {
                loading.set(false);
                return;
            }
            match list_wallets_persist(email_c).await {
                Ok(list) => wallets.set(list),
                Err(e) => status.set(format!("No se pudieron cargar wallets: {}", e)),
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "space-y-6 max-w-2xl",
            h1 { class: "text-3xl font-bold text-primary", "Configuración" }
            // Wave2: CreateWalletCard when no wallet_pubkey (frontend-only devnet helper)
            if connected.is_none() {
                CreateWalletCard {}
            }
            // Legacy single wallet quick connect
            div { class: "bg-surface border border-border rounded-2xl p-6 space-y-4",
                h2 { class: "text-lg font-bold", "Wallet Solana" }
                p { class: "text-sm text-muted", "Conectá Phantom para firmar mensajes y transacciones. Trust Work nunca recibe ni almacena tu clave privada." }
                if let Some(pubkey) = connected.clone() {
                    div { class: "bg-bg border border-border rounded-xl p-4 space-y-2",
                        div { class: "text-xs text-muted", "Phantom conectada (legacy single)" }
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
                        "Desconectar (sesión)"
                    }
                } else {
                    button {
                        class: "bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium hover:opacity-90",
                        r#type: "button",
                        onclick: move |_| {
                            let mut auth = auth;
                            let email_c = email_for_phantom.clone();
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
                                                    // also persist as publish wallet if we have email
                                                    if !email_c.is_empty() {
                                                        let email_c2 = email_c.clone();
                                                        let _ = add_wallet_persist(email_c, pubkey.clone(), "publish".to_string(), Some("Phantom".to_string())).await;
                                                        // refresh list
                                                        if let Ok(list) = list_wallets_persist(email_c2).await {
                                                            wallets.set(list);
                                                        }
                                                    }
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
                        "Conectar Phantom (publish)"
                    }
                }
                if !status.read().is_empty() {
                    p { class: "text-sm text-primary", "{status.read()}" }
                }
            }

            // Multi-wallet list + add
            div { class: "bg-surface border border-border rounded-2xl p-6 space-y-4",
                h2 { class: "text-lg font-bold", "Mis Wallets ({wallets.read().len()})" }
                p { class: "text-sm text-muted", "1 wallet = auto-selección. 2+ wallets = picker por acción (publish para crear, apply para postular). Validación bs58 32B." }
                if *loading.read() {
                    p { class: "text-sm text-muted", "Cargando wallets..." }
                } else if wallets.read().is_empty() {
                    p { class: "text-sm text-muted", "Sin wallets. Conectá Phantom o pegá un pubkey." }
                } else {
                    for w in wallets.read().iter() {
                        div { class: "bg-bg border border-border rounded-xl p-3 flex items-center justify-between gap-2",
                            div { class: "space-y-1 min-w-0 flex-1",
                                div { class: "flex items-center gap-2",
                                    span { class: "text-xs font-bold px-2 py-0.5 rounded-full bg-primary text-on-primary", "{purpose_badge(&w.purpose)}" }
                                    span { class: "font-mono text-sm truncate", "{short_pubkey(&w.pubkey)}" }
                                }
                                div { class: "text-xs text-muted truncate", "{w.pubkey}" }
                                if let Some(label) = &w.label {
                                    div { class: "text-xs text-muted", "Label: {label}" }
                                }
                            }
                            button {
                                class: "bg-red-500/10 border border-red-500/30 text-red-600 rounded-xl px-3 py-1 text-xs",
                                r#type: "button",
                                disabled: removing.read().as_ref() == Some(&w.pubkey),
                                onclick: {
                                    let email_c = email_for_remove_list.clone();
                                    let pk = w.pubkey.clone();
                                    move |_| {
                                        let email_c = email_c.clone();
                                        let pk_c = pk.clone();
                                        spawn(async move {
                                            removing.set(Some(pk_c.clone()));
                                            match remove_wallet_persist(email_c.clone(), pk_c.clone()).await {
                                                Ok(_) => {
                                                    status.set(format!("Wallet {} eliminada", short_pubkey(&pk_c)));
                                                    if let Ok(list) = list_wallets_persist(email_c).await { wallets.set(list); }
                                                }
                                                Err(e) => {
                                                    let msg = e.to_string();
                                                    if msg.contains("WalletHasActiveJob") {
                                                        status.set("No se puede eliminar: WalletHasActiveJob (tiene job en curso o disputa activa)".to_string());
                                                    } else {
                                                        status.set(format!("Error al eliminar: {}", msg));
                                                    }
                                                }
                                            }
                                            removing.set(None);
                                        });
                                    }
                                },
                                if removing.read().as_ref() == Some(&w.pubkey) { "Eliminando..." } else { "Eliminar" }
                            }
                        }
                    }
                }
                // Add form: pubkey + purpose + label (optional)
                div { class: "border-t border-border pt-4 space-y-3",
                    h3 { class: "text-sm font-bold", "Añadir wallet" }
                    input {
                        class: "w-full bg-bg border border-border rounded-xl px-3 py-2 text-sm font-mono",
                        placeholder: "Pubkey base58 32B (44 chars) o Conectar Phantom arriba",
                        value: "{new_pubkey.read()}",
                        oninput: move |e| new_pubkey.set(e.value()),
                    }
                    div { class: "flex gap-2",
                        select {
                            class: "bg-bg border border-border rounded-xl px-3 py-2 text-sm",
                            value: "{new_purpose.read()}",
                            onchange: move |e| new_purpose.set(e.value()),
                            option { value: "publish", "publish (crear jobs)" }
                            option { value: "apply", "apply (postular)" }
                            option { value: "general", "general" }
                        }
                        button {
                            class: "bg-primary text-on-primary rounded-xl px-5 py-2 text-sm font-medium",
                            r#type: "button",
                            onclick: {
                                let email_c = email_for_add_btn.clone();
                                move |_| {
                                    let email_c = email_c.clone();
                                    let pk = new_pubkey.read().clone().trim().to_string();
                                    let purpose = new_purpose.read().clone();
                                    if pk.is_empty() {
                                        status.set("Pegá un pubkey o conectá Phantom".to_string());
                                        return;
                                    }
                                    spawn(async move {
                                        if email_c.is_empty() { status.set("Iniciá sesión para añadir wallets".to_string()); return; }
                                        match add_wallet_persist(email_c.clone(), pk.clone(), purpose, None).await {
                                            Ok(w) => {
                                                status.set(format!("Wallet {} añadida ({})", short_pubkey(&w.pubkey), w.purpose));
                                                new_pubkey.set(String::new());
                                                if let Ok(list) = list_wallets_persist(email_c).await { wallets.set(list); }
                                            }
                                            Err(e) => status.set(format!("Error: {}", e)),
                                        }
                                    });
                                }
                            },
                            "Añadir"
                        }
                    }
                    p { class: "text-xs text-muted", "La wallet se valida bs58 32B en backend. Usá Phantom para evitar errores." }
                }
            }

            // Picker hint + insufficient funds banner placeholder
            div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl p-4 space-y-2",
                h3 { class: "text-sm font-bold text-amber-700 dark:text-amber-300", "Picker & fondos" }
                p { class: "text-xs text-muted", "Con 1 wallet el sistema auto-selecciona. Con 2+ elegí publish vs apply al crear/postular. Antes de firmar, se verifica getBalance: si balance < amount+fee → bloquado con banner para elegir otra wallet." }
                if wallets.read().len() >= 2 {
                    p { class: "text-xs text-amber-700", "Tenés {wallets.read().len()} wallets — el picker aparecerá en Crear Job y Postular." }
                }
            }
        }
    }
}

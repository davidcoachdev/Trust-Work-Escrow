//! Config > Wallet — single place to create/connect deterministic wallet.
//! Do NOT create wallets on the fly in modals.

use dioxus::prelude::*;
use crate::server::auth::guest::use_auth;
use crate::server::auth::wallet::{get_or_create_wallet, reveal_wallet_private_key, link_wallet_to_user};

#[component]
pub fn ConfigPage() -> Element {
    let auth = use_auth();
    let user_sig = auth.user;
    let user = user_sig.read().clone();

    let mut email_input = use_signal(|| String::new());
    let mut wallet_info = use_signal(|| None::<crate::server::auth::wallet::WalletInfo>);
    let mut msg = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut show_reveal = use_signal(|| false);
    let mut otp_input = use_signal(|| String::new());
    let mut priv_key = use_signal(|| None::<String>);
    let mut checked_saved = use_signal(|| false);
    let mut confirm_replace = use_signal(|| false);
    let mut linked_external = use_signal(|| None::<String>);

    // Prefill email from auth
    let current_email = user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let _has_wallet = user.as_ref().and_then(|u| u.wallet_pubkey.clone()).is_some();
    let current_pubkey = user.as_ref().and_then(|u| u.wallet_pubkey.clone()).unwrap_or_default();
    // clones for closures (avoid move error)
    let current_email_create = current_email.clone();
    let current_email_reveal = current_email.clone();
    let current_email_replace = current_email.clone();

    // If we derived a new wallet, reflect it
    let display_pubkey = if let Some(info) = wallet_info.read().as_ref() {
        info.pubkey.clone()
    } else {
        current_pubkey.clone()
    };
    let has_display_wallet = !display_pubkey.is_empty();
    // Clone for button closure (avoid nested rsx! which breaks onclick)
    let display_pubkey_for_copy = display_pubkey.clone();

    rsx! {
        div { class: "space-y-6 max-w-2xl",
            h1 { class: "text-3xl font-bold text-primary", "Configuración" }
            p { class: "text-muted text-sm", "Gestioná tu billetera Solana. La creación es determinística por email (HMAC) y recuperable vía OTP. Solo se crea aquí." }

            div { class: "bg-surface border border-border rounded-2xl p-6 space-y-4",
                h2 { class: "text-lg font-bold", "Wallet Solana" }
                p { class: "text-xs text-muted", "Soporta Phantom (hoy) y futuro Backpack / Solflare. Tu clave se deriva de tu email + secreto del servidor." }

                if has_display_wallet {
                    div { class: "bg-bg border border-border rounded-xl p-4 space-y-3",
                        div { class: "text-xs text-muted", "Tu billetera" }
                        div { class: "font-mono text-sm break-all text-primary", "{display_pubkey.clone()}" }
                        div { class: "flex flex-wrap gap-2",
                            button {
                                class: "bg-bg border border-border rounded-xl px-3 py-2 text-sm hover:border-primary",
                                r#type: "button",
                                onclick: move |_| {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(win) = web_sys::window() {
                                            let _ = win.navigator().clipboard().write_text(&display_pubkey_for_copy);
                                        }
                                    }
                                    msg.set("Copiado".to_string());
                                },
                                "Copiar"
                            }
                            a {
                                class: "inline-flex bg-primary text-on-primary rounded-xl px-3 py-2 text-sm font-medium",
                                href: "https://phantom.app/",
                                target: "_blank",
                                "Conectar a Phantom"
                            }
                            a {
                                class: "inline-flex bg-bg border border-border rounded-xl px-3 py-2 text-sm",
                                href: format!("solana:{}", display_pubkey.clone()),
                                "Deep link solana:{display_pubkey.clone()}"
                            }
                        }
                        // Export / reveal private key (requires OTP + checkbox)
                        if priv_key.read().is_none() {
                            div { class: "border-t border-border pt-3 space-y-2",
                                button {
                                    class: "text-sm underline text-muted",
                                    r#type: "button",
                                    onclick: move |_| {
                                        let cur = show_reveal.with(|v| *v);
                                        show_reveal.set(!cur);
                                    },
                                    "Ver clave privada"
                                }
                                if *show_reveal.read() {
                                    div { class: "grid gap-2 bg-bg border border-border rounded-xl p-3",
                                        p { class: "text-xs text-muted", "Requiere OTP re-verificado. Solo se muestra una vez." }
                                        input {
                                            class: "bg-surface border border-border rounded-xl px-3 py-2 text-sm",
                                            placeholder: "OTP 6 dígitos",
                                            value: "{otp_input.read()}",
                                            oninput: move |e| otp_input.set(e.value()),
                                        }
                                        label { class: "flex items-center gap-2 text-sm",
                                            input {
                                                r#type: "checkbox",
                                                checked: *checked_saved.read(),
                                                onchange: move |_| {
                                                    let cur = checked_saved.with(|v| *v);
                                                    checked_saved.set(!cur);
                                                },
                                            }
                                            "Ya la guardé / Entiendo que no se muestra de nuevo"
                                        }
                                        button {
                                            class: "bg-primary text-on-primary rounded-xl px-4 py-2 text-sm disabled:opacity-50",
                                            disabled: !*checked_saved.read() || otp_input.read().len() != 6,
                                            r#type: "button",
                                            onclick: move |_| {
                                                let email = if !email_input.read().is_empty() { email_input.read().clone() } else { current_email_reveal.clone() };
                                                let otp = otp_input.read().clone();
                                                spawn(async move {
                                                    match reveal_wallet_private_key(email, otp).await {
                                                        Ok(pk) => { priv_key.set(Some(pk)); msg.set("Guardala — no la mostramos de nuevo".to_string()); },
                                                        Err(e) => msg.set(format!("Error: {}", e)),
                                                    }
                                                });
                                            },
                                            "Revelar clave"
                                        }
                                        p { class: "text-xs text-amber-500", "⚠️ Guardala en lugar seguro. No la mostramos de nuevo." }
                                    }
                                }
                            }
                        } else {
                            div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl p-3 space-y-1",
                                div { class: "text-xs font-bold text-amber-600", "Clave privada (base58) — única vez" }
                                div { class: "font-mono text-xs break-all", "{priv_key.read().as_deref().unwrap_or(\"\")}" }
                                p { class: "text-xs text-muted", "Copiala y guardala offline. Cerrá esta vista." }
                                button {
                                    class: "text-xs underline",
                                    r#type: "button",
                                    onclick: move |_| { priv_key.set(None); show_reveal.set(false); },
                                    "Ocultar"
                                }
                            }
                        }

                        div { class: "flex gap-2 pt-2",
                            button {
                                class: "bg-bg border border-border rounded-xl px-3 py-2 text-sm",
                                r#type: "button",
                                onclick: move |_| {
                                    // Disconnect: clear local wallet display (MVP, no persist)
                                    wallet_info.set(None);
                                    msg.set("Desconectado (local). Volvé a crear o reconectar.".to_string());
                                },
                                "Desconectar"
                            }
                            if !*confirm_replace.read() {
                                button {
                                    class: "bg-amber-500/10 border border-amber-500/30 rounded-xl px-3 py-2 text-sm text-amber-600",
                                    r#type: "button",
                                    onclick: move |_| confirm_replace.set(true),
                                    "Crear nueva (reemplaza) ¿Seguro?"
                                }
                            } else {
                                div { class: "flex gap-2",
                                    button {
                                        class: "bg-red-500 text-white rounded-xl px-3 py-2 text-sm",
                                        r#type: "button",
                                            onclick: move |_| {
                                            let email = if !email_input.read().is_empty() { email_input.read().clone() } else { current_email_replace.clone() };
                                            spawn(async move {
                                                loading.set(true);
                                                match get_or_create_wallet(email, true).await {
                                                    Ok(info) => { wallet_info.set(Some(info)); msg.set("Nueva billetera (mismo derivada) — verificá".to_string()); },
                                                    Err(e) => msg.set(format!("Error: {}", e)),
                                                }
                                                loading.set(false);
                                                confirm_replace.set(false);
                                            });
                                        },
                                        "Confirmar reemplazo"
                                    }
                                    button {
                                        class: "bg-bg border border-border rounded-xl px-3 py-2 text-sm",
                                        r#type: "button",
                                        onclick: move |_| confirm_replace.set(false),
                                        "Cancelar"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "bg-bg border border-dashed border-border rounded-xl p-6 text-center space-y-4",
                        p { class: "text-sm text-muted", "Aún no tenés billetera vinculada." }
                        div { class: "grid gap-2 text-left max-w-sm mx-auto",
                            label { class: "text-sm text-muted", "Tu email (para derivación determinística)" }
                            input {
                                class: "bg-surface border border-border rounded-xl px-3 py-2 text-sm",
                                placeholder: "tu@correo.com",
                                value: "{email_input.read()}",
                                oninput: move |e| email_input.set(e.value()),
                            }
                            p { class: "text-xs text-muted", "Dejalo vacío para usar {current_email}" }
                        }
                        button {
                            class: "inline-flex bg-primary text-on-primary rounded-xl px-6 py-3 font-medium hover:-translate-y-0.5 transition disabled:opacity-50",
                            disabled: *loading.read(),
                            r#type: "button",
                            onclick: move |_| {
                                let email = if !email_input.read().is_empty() { email_input.read().clone() } else { current_email_create.clone() };
                                if email.is_empty() || !email.contains('@') {
                                    msg.set("Ingresá un email válido".to_string());
                                    return;
                                }
                                spawn(async move {
                                    loading.set(true);
                                    msg.set(String::new());
                                    match get_or_create_wallet(email.clone(), false).await {
                                        Ok(info) => {
                                            let pk = info.pubkey.clone();
                                            wallet_info.set(Some(info.clone()));
                                            // Optimistic local update to auth context
                                            // TODO: persist to DB and hydrate via get_me
                                            msg.set(format!("Billetera creada: {} — podés conectarla a Phantom", pk));
                                        },
                                        Err(e) => msg.set(format!("Error: {}", e)),
                                    }
                                    loading.set(false);
                                });
                            },
                            if *loading.read() { "Creando..." } else { "Crear mi billetera Solana" }
                        }
                        p { class: "text-xs text-muted", "Derivada vía HMAC-SHA256(secreto, email) → ed25519. Recuperable con OTP." }
                        div { class: "flex justify-center gap-2 pt-2",
                            a { class: "text-xs text-primary underline", href: "https://phantom.app/", target: "_blank", "Phantom" }
                            span { class: "text-xs text-muted", "· Próximamente Backpack / Solflare" }
                        }
                        // Connect existing Phantom wallet (SIWS already exists)
                        div { class: "border-t border-border pt-4 space-y-2",
                            p { class: "text-sm font-medium", "¿Ya tenés Phantom?" }
                            input {
                                class: "bg-surface border border-border rounded-xl px-3 py-2 text-sm w-full",
                                placeholder: "Pubkey base58 de Phantom",
                                value: "{linked_external.read().clone().unwrap_or_default()}",
                                oninput: move |e| linked_external.set(Some(e.value())),
                            }
                            button {
                                class: "bg-bg border border-border rounded-xl px-4 py-2 text-sm",
                                r#type: "button",
                                onclick: move |_| {
                                    if let Some(pk) = linked_external.read().clone() {
                                        spawn(async move {
                                            match link_wallet_to_user(pk.clone()).await {
                                                Ok(_) => msg.set(format!("Wallet {} vinculada (verificada vía SIWS)", pk)),
                                                Err(e) => msg.set(format!("Error: {}", e)),
                                            }
                                        });
                                    }
                                },
                                "Conectar wallet existente"
                            }
                        }
                    }
                }

                if !msg.read().is_empty() {
                    p { class: "text-sm text-primary text-center", "{msg.read()}" }
                }
            }
        }
    }
}

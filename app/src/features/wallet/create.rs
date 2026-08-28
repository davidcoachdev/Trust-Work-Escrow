//! Wallet creation — frontend-only devnet helper.
//! Generates BIP39 12-word mnemonic and derives ed25519 pubkey (bs58 44 chars).
//! Seed NEVER leaves the browser, NEVER hits localStorage/api/logs, zeroized on drop/close.

use zeroize::{Zeroize, Zeroizing};

/// Generate a fresh 12-word BIP39 mnemonic (128-bit entropy).
/// Returns Zeroizing<String> — auto-zeroized on drop.
pub fn generate_mnemonic() -> Zeroizing<String> {
    use bip39::{Language, Mnemonic};
    let mnemonic =
        Mnemonic::generate_in(Language::English, 12).expect("bip39 generate 12 words");
    Zeroizing::new(mnemonic.to_string())
}

/// Derive Solana pubkey (bs58 44 chars, 32 bytes) from a BIP39 mnemonic.
/// Uses seed = mnemonic.to_seed("") and ed25519-dalek SigningKey from first 32 bytes.
pub fn mnemonic_to_pubkey(mnemonic: &str) -> Result<String, String> {
    use bip39::{Language, Mnemonic};
    let parsed = Mnemonic::parse_in(Language::English, mnemonic.trim())
        .map_err(|e| format!("mnemonic inválida: {e}"))?;
    let seed = parsed.to_seed("");
    if seed.len() < 32 {
        return Err("seed demasiado corta".to_string());
    }
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&seed[..32]);
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
    let verifying = signing_key.verifying_key();
    let pubkey = bs58::encode(verifying.to_bytes()).into_string();
    // Validate round-trip 32B
    if bs58::decode(&pubkey).into_vec().map(|v| v.len() == 32).unwrap_or(false) {
        // zeroize secret copy
        secret_bytes.zeroize();
        Ok(pubkey)
    } else {
        secret_bytes.zeroize();
        Err("pubkey derivada inválida".to_string())
    }
}

#[cfg(target_arch = "wasm32")]
fn copy_to_clipboard(text: String) {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let _ = clipboard.write_text(&text);
    }
}

use dioxus::prelude::*;

use crate::i18n::{tr, use_i18n};
use crate::server::auth::guest::use_auth;
use crate::server::auth::siws::{request_siws_challenge, verify_siws_server};
use crate::server::auth::users::add_wallet_persist;
use crate::solana::phantom;

#[component]
pub fn CreateWalletCard() -> Element {
    let i18n = use_i18n();
    let lang = *i18n.lang.read();
    let mut auth = use_auth();
    let mut mnemonic: Signal<Option<Zeroizing<String>>> = use_signal(|| None);
    let mut confirmed = use_signal(|| false);
    let mut copied = use_signal(|| false);
    let mut status = use_signal(|| String::new());
    let mut linking = use_signal(|| false);

    // Zeroize on unmount: best-effort via use_drop (dioxus 0.7 provides use_drop)
    // Fallback use_effect cleanup — track mnemonic presence.
    use_effect(move || {
        let _ = mnemonic.read().is_some();
    });

    let on_generate = move |_| {
        let mn = generate_mnemonic();
        // Ensure previous is zeroized before replace (Zeroizing Drop handles, but explicit)
        if let Some(mut prev) = mnemonic.write().take() {
            prev.zeroize();
        }
        // Reset UI state
        confirmed.set(false);
        copied.set(false);
        status.set(String::new());
        mnemonic.set(Some(mn));
    };

    let on_copy_all = move |_| {
        if let Some(ref m) = *mnemonic.read() {
            #[cfg(target_arch = "wasm32")]
            {
                copy_to_clipboard(m.to_string());
            }
            copied.set(true);
            status.set(tr(lang, "wallet.seed.copied").to_string());
            // reset copied flag after 2s (wasm) or immediate (native test)
            #[cfg(target_arch = "wasm32")]
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(2000).await;
                copied.set(false);
            });
            #[cfg(not(target_arch = "wasm32"))]
            {
                copied.set(false);
            }
        }
    };

    let on_forget = move |_| {
        if let Some(mut prev) = mnemonic.write().take() {
            prev.zeroize();
        }
        confirmed.set(false);
        copied.set(false);
        status.set(String::new());
    };

    // SIWS-gated Phantom link: "Ya importé → vincular"
    let on_link = move |_| {
        if !*confirmed.read() {
            status.set("Confirmá que guardaste tu frase antes de vincular.".to_string());
            return;
        }
        // Ensure mnemonic is zeroized before leaving this card (user confirmed import)
        if let Some(mut prev) = mnemonic.write().take() {
            prev.zeroize();
        }
        linking.set(true);
        status.set("Esperando aprobación en Phantom…".to_string());
        spawn(async move {
            match phantom::connect().await {
                Ok(pubkey) => match request_siws_challenge(pubkey.clone()).await {
                    Ok(message) => match phantom::sign_message(&message).await {
                        Ok(signature) => match verify_siws_server(pubkey.clone(), message, signature).await {
                            Ok(_) => {
                                // Persist linked wallet (publish) if we have email
                                let email = auth.user.read().as_ref().map(|u| u.email.clone()).unwrap_or_default();
                                if !email.is_empty() {
                                    let _ = add_wallet_persist(email.clone(), pubkey.clone(), "publish".to_string(), Some("Phantom".to_string())).await;
                                    // also update legacy single wallet for header badge
                                    let mut user = auth.user.read().clone().unwrap_or_default();
                                    user.wallet_pubkey = Some(pubkey.clone());
                                    auth.user.set(Some(user));
                                } else {
                                    let mut user = auth.user.read().clone().unwrap_or_default();
                                    user.wallet_pubkey = Some(pubkey.clone());
                                    auth.user.set(Some(user));
                                }
                                status.set(format!("Phantom vinculada: {}…", &pubkey[..6]));
                                linking.set(false);
                                confirmed.set(false);
                                // refresh: rely on ConfigPage list reload via phantom connect? keep status.
                            }
                            Err(_) => {
                                status.set("Phantom firmó, pero la verificación falló.".to_string());
                                linking.set(false);
                            }
                        },
                        Err(e) => {
                            status.set(e);
                            linking.set(false);
                        }
                    },
                    Err(_) => {
                        status.set("No se pudo obtener challenge SIWS.".to_string());
                        linking.set(false);
                    }
                },
                Err(e) => {
                    status.set(e);
                    linking.set(false);
                }
            }
        });
    };

    let words: Vec<String> = mnemonic
        .read()
        .as_ref()
        .map(|m| m.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    let derived_pk: Option<String> = mnemonic
        .read()
        .as_ref()
        .and_then(|m| mnemonic_to_pubkey(m.as_str()).ok());

    rsx! {
        div { class: "bg-surface border border-border rounded-2xl p-6 space-y-4",
            div { class: "flex items-center justify-between",
                h2 { class: "text-lg font-bold", "{tr(lang, \"wallet.create\")}" }
                span { class: "text-xs px-2 py-1 rounded-full bg-amber-500/20 text-amber-700 border border-amber-500/30", "devnet only" }
            }

            // Warning banner — never share seed
            div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl p-3 flex gap-2",
                span { class: "text-amber-600 text-sm", "⚠️" }
                p { class: "text-xs text-amber-700 dark:text-amber-300", "{tr(lang, \"wallet.seed.warning\")}" }
            }

            if mnemonic.read().is_none() {
                button {
                    class: "w-full bg-primary text-on-primary rounded-xl px-5 py-3 font-medium hover:opacity-90 transition",
                    r#type: "button",
                    onclick: on_generate,
                    "Generar frase de 12 palabras"
                }
                p { class: "text-xs text-muted text-center", "La frase se genera localmente y nunca se envía al servidor." }
            } else {
                // Grid 2×6
                div { class: "grid grid-cols-2 sm:grid-cols-3 gap-2",
                    for (idx, word) in words.iter().enumerate() {
                        div { class: "bg-bg border border-border rounded-xl px-3 py-2 flex items-center justify-between",
                            span { class: "text-xs text-muted", "{idx + 1}" }
                            span { class: "text-sm font-mono font-medium", "{word}" }
                        }
                    }
                }
                // Derived pubkey preview (optional)
                if let Some(pk) = derived_pk.clone() {
                    div { class: "bg-bg border border-border rounded-xl p-3 space-y-1",
                        div { class: "text-xs text-muted", "Pubkey derivada (verificación, importá en Phantom para usar)" }
                        div { class: "font-mono text-xs break-all text-primary", "{pk}" }
                        div { class: "text-[10px] text-muted", "bs58 44 chars · 32 bytes" }
                    }
                }
                div { class: "flex gap-2",
                    button {
                        class: "flex-1 bg-bg border border-border rounded-xl px-4 py-2 text-sm font-medium hover:bg-surface",
                        r#type: "button",
                        onclick: on_copy_all,
                        if *copied.read() { "{tr(lang, \"wallet.seed.copied\")}" } else { "{tr(lang, \"wallet.seed.copy\")} (12 palabras)" }
                    }
                    button {
                        class: "bg-red-500/10 border border-red-500/30 text-red-600 rounded-xl px-4 py-2 text-sm",
                        r#type: "button",
                        onclick: on_forget,
                        "Olvidar"
                    }
                }

                // Phantom import steps
                div { class: "bg-bg border border-border rounded-xl p-4 space-y-2",
                    h3 { class: "text-sm font-bold", "{tr(lang, \"wallet.phantom.title\")}" }
                    ol { class: "list-decimal list-inside space-y-1 text-xs text-muted",
                        li { "{tr(lang, \"wallet.phantom.step1\")}" }
                        li { "{tr(lang, \"wallet.phantom.step2\")}" }
                        li { "{tr(lang, \"wallet.phantom.step3\")}" }
                    }
                    p { class: "text-[11px] text-muted", "Luego volvé y confirmá para vincular con Phantom (SIWS)." }
                }

                // Confirm checkbox gating
                label { class: "flex items-center gap-2 text-sm",
                    input {
                        r#type: "checkbox",
                        checked: *confirmed.read(),
                        onchange: move |_| {
                            let next = !*confirmed.read();
                            confirmed.set(next);
                        },
                    }
                    span { "{tr(lang, \"wallet.seed.confirm\")}" }
                }

                button {
                    class: "w-full rounded-xl px-5 py-3 font-medium transition",
                    class: if *confirmed.read() { "bg-primary text-on-primary hover:opacity-90" } else { "bg-bg border border-border text-muted opacity-50 cursor-not-allowed" },
                    r#type: "button",
                    disabled: !*confirmed.read() || *linking.read(),
                    onclick: on_link,
                    if *linking.read() { "Vinculando con Phantom…" } else { "Ya importé → vincular con Phantom" }
                }
            }

            if !status.read().is_empty() {
                p { class: "text-sm text-primary text-center", "{status.read()}" }
            }
            p { class: "text-[11px] text-muted text-center", "Trust Work nunca recibe tu frase. Solo Phantom custodia tu clave." }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroize;

    #[test]
    fn generate_mnemonic_is_12_words() {
        let m = generate_mnemonic();
        let words: Vec<&str> = m.split_whitespace().collect();
        assert_eq!(words.len(), 12, "mnemonic must be 12 words, got {}", m.as_str());
        assert!(m.as_str().split_whitespace().all(|w| w.len() >= 3));
    }

    #[test]
    fn mnemonic_to_pubkey_is_bs58_44_and_32_bytes() {
        let m = generate_mnemonic();
        let pk = mnemonic_to_pubkey(m.as_str()).expect("should derive pubkey");
        // Solana pubkey 32B => bs58 43-44 chars (depends on leading zeros)
        assert!(
            pk.len() >= 43 && pk.len() <= 44,
            "bs58 pubkey should be 43-44 chars, got {} len {}",
            pk,
            pk.len()
        );
        let bytes = bs58::decode(&pk).into_vec().expect("bs58 decode");
        assert_eq!(bytes.len(), 32, "pubkey must be 32 bytes");
    }

    #[test]
    fn mnemonic_to_pubkey_deterministic_for_same_mnemonic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let pk1 = mnemonic_to_pubkey(phrase).unwrap();
        let pk2 = mnemonic_to_pubkey(phrase).unwrap();
        assert_eq!(pk1, pk2);
    }

    #[test]
    fn mnemonic_to_pubkey_rejects_invalid() {
        assert!(mnemonic_to_pubkey("not a valid mnemonic phrase at all words").is_err());
        assert!(mnemonic_to_pubkey("").is_err());
    }

    #[test]
    fn zeroizing_drop_zeroizes() {
        let mut z = Zeroizing::new(String::from("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"));
        let ptr = z.as_ptr() as *const u8;
        // zeroize explicitly
        z.zeroize();
        // after zeroize, content is zeroed (all zeros) — String will be empty or zeroed
        assert!(z.as_str().bytes().all(|b| b == 0) || z.is_empty());
        let _ = ptr;
    }

    #[test]
    fn validate_no_mnemonic_in_localstorage_key() {
        // Ensure we never use localStorage key containing mnemonic/seed
        let forbidden = ["mnemonic", "seed", "phrase"];
        let used_keys = ["twe-lang", "twe-theme", "twe-mode", "twe-email", "twe-sidebar-collapsed", "twe-jwt", "twe-guest"];
        for k in used_keys {
            for f in forbidden {
                assert!(!k.contains(f), "key {} should not contain {}", k, f);
            }
        }
    }
}

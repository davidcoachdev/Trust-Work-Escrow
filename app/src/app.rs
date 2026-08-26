use crate::i18n::{apply_lang, load_lang, I18nContext};
use crate::route::Route;
use crate::server::auth::guest::{AuthContext, User};
use crate::theme::{apply_mode, apply_theme, load_mode, load_theme, ModeContext, ThemeContext};
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

#[cfg(target_arch = "wasm32")]
fn stored_email() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("twe-email").ok().flatten())
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty() && v.contains('@'))
}

#[cfg(not(target_arch = "wasm32"))]
fn stored_email() -> Option<String> {
    None
}

#[component]
pub fn App() -> Element {
    let theme = use_signal(|| load_theme());
    let lang = use_signal(|| load_lang());
    let mode = use_signal(|| load_mode());
    let mut user = use_signal(|| None::<User>);

    // Hydrate: try persisted user from Postgres via localStorage email, fallback to get_me/guest
    use_effect(move || {
        spawn(async move {
            // 1) try localStorage -> Postgres
            if let Some(email) = stored_email() {
                match crate::server::auth::users::get_user_by_email_server(email.clone()).await {
                    Ok(Some(u)) => {
                        user.set(Some(u));
                        return;
                    }
                    Ok(None) => {
                        // email in LS but not in DB (deleted) — clear it and fall through
                        #[cfg(target_arch = "wasm32")]
                        if let Some(win) = web_sys::window() {
                            if let Ok(Some(storage)) = win.local_storage() {
                                let _ = storage.remove_item("twe-email");
                            }
                        }
                    }
                    Err(_) => {
                        // DB unreachable — fall through to get_me
                    }
                }
            }

            // 2) fallback to server guest/JWT
            if let Ok(Some(u)) = crate::server::auth::guest::get_me().await {
                user.set(Some(u));
            } else if user.read().is_none() {
                user.set(Some(User {
                    email: "invitado@guest.local".to_string(),
                    wallet_pubkey: None,
                    role: "guest".to_string(),
                    roles: vec!["guest".to_string()],
                    permissions: vec![],
                    is_guest: true,
                    created_at: 0,
                    updated_at: 0,
                    is_active: true,
                }));
            }
        });
    });

    use_effect(move || {
        let t = *theme.read();
        apply_theme(t);
    });
    use_effect(move || {
        let ln = *lang.read();
        apply_lang(ln);
    });
    use_effect(move || {
        let mo = *mode.read();
        apply_mode(mo);
    });

    use_context_provider(|| ThemeContext { theme });
    use_context_provider(|| I18nContext { lang });
    use_context_provider(|| ModeContext { mode });
    use_context_provider(|| AuthContext { user });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

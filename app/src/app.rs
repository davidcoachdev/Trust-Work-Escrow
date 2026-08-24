use crate::i18n::{apply_lang, load_lang, I18nContext};
use crate::route::Route;
use crate::server::auth::guest::{AuthContext, User};
use crate::theme::{apply_mode, apply_theme, load_mode, load_theme, ModeContext, ThemeContext};
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let theme = use_signal(|| load_theme());
    let lang = use_signal(|| load_lang());
    let mode = use_signal(|| load_mode());
    let mut user = use_signal(|| None::<User>);

    // Hydrate from server via get_me (reads twe-jwt or twe-guest cookie)
    // On client, this runs once after mount; on SSR it would run during render.
    use_effect(move || {
        spawn(async move {
            if let Ok(Some(u)) = crate::server::auth::guest::get_me().await {
                user.set(Some(u));
            } else if user.read().is_none() {
                // Fallback guest for MVP when get_me returns None on WASM
                user.set(Some(User {
                    email: "invitado@guest.local".to_string(),
                    wallet_pubkey: None,
                    role: "guest".to_string(),
                    is_guest: true,
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

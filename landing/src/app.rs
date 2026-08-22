use dioxus::prelude::*;
use crate::theme::{apply_mode, apply_theme, load_mode, load_theme, ModeContext, ThemeContext};
use crate::i18n::{apply_lang, load_lang, I18nContext};
use crate::route::Route;

const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let theme = use_signal(|| load_theme());
    let lang = use_signal(|| load_lang());
    let mode = use_signal(|| load_mode());

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

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

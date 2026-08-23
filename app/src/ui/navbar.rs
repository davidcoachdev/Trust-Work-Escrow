use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::theme::{use_mode, Mode};
use crate::route::Route;
use crate::ui::{LanguageSwitcher, ThemeSwitcher};
use crate::server::auth::guest::use_auth_opt;

#[component]
pub fn Navbar() -> Element {
    let lang = use_i18n().lang;
    let mut mode = use_mode().mode;
    let l = *lang.read();
    let m = *mode.read();
    let auth_opt = use_auth_opt();
    let user_opt = auth_opt.as_ref().and_then(|a| a.user.read().clone());

    rsx! {
        header { class: "sticky top-0 z-10 bg-bg/80 backdrop-blur border-b border-border",
            div { class: "wrap flex items-center justify-between py-4",
                Link { class: "font-bold text-lg tracking-tight", to: Route::LandingPage {}, {tr(l, "brand")} }
                nav { class: "hidden md:flex items-center gap-6",
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.home")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.jobs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.docs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::ContactPage {}, {tr(l, "nav.contact")} }
                    if user_opt.is_some() {
                        if let Some(u) = user_opt.clone() {
                            if let Some(pk) = u.wallet_pubkey.clone() {
                                span { class: "font-mono text-xs bg-surface border border-border rounded-full px-3 py-1", "{&pk[..6.min(pk.len())]}...{&pk[pk.len().saturating_sub(4)..]}" }
                            } else {
                                Link { class: "text-xs bg-amber-500/10 border border-amber-500/30 rounded-full px-3 py-1 text-amber-600", to: Route::ConfigPage {}, "Config · Crear billetera" }
                            }
                        }
                        Link { class: "text-sm text-primary underline", to: Route::ConfigPage {}, "Config" }
                    } else {
                        Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LoginPage {}, {tr(l, "nav.login")} }
                        Link {
                            class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5",
                            to: Route::SignupPage {},
                            {tr(l, "nav.signup")}
                        }
                        span { class: "text-xs bg-surface border border-border rounded-full px-2 py-1 text-muted", "Invitado" }
                    }
                }
                div { class: "flex items-center gap-3",
                    LanguageSwitcher {}
                    ThemeSwitcher {}
                    button {
                        class: "text-sm text-fg cursor-pointer font-inherit bg-transparent border-none",
                        r#type: "button",
                        onclick: move |_| {
                            let next = match *mode.read() { Mode::Dark => Mode::Light, Mode::Light => Mode::Dark };
                            mode.set(next);
                        },
                        { tr(l, if m == Mode::Dark { "switcher.light" } else { "switcher.dark" }) }
                    }
                }
            }
        }
        main {
            Outlet::<Route> {}
        }
    }
}

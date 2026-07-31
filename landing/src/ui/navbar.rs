use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::theme::{use_mode, Mode};
use crate::route::Route;
use crate::ui::{LanguageSwitcher, ThemeSwitcher};

#[component]
pub fn Navbar() -> Element {
    let lang = use_i18n().lang;
    let mut mode = use_mode().mode;
    let l = *lang.read();
    let m = *mode.read();

    rsx! {
        header { class: "sticky top-0 z-10 bg-bg/80 backdrop-blur border-b border-border",
            div { class: "wrap flex items-center justify-between py-4",
                Link { class: "font-bold text-lg tracking-tight", to: Route::LandingPage {}, {tr(l, "brand")} }
                nav { class: "hidden md:flex items-center gap-6",
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.home")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.jobs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.docs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::ContactPage {}, {tr(l, "nav.contact")} }
                    Link {
                        class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5",
                        to: Route::SignupPage {},
                        {tr(l, "nav.signup")}
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

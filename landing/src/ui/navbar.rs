use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::theme::{apply_mode, use_mode, Mode};
use crate::route::Route;
use crate::ui::{LanguageSwitcher, ThemeSwitcher};

#[component]
pub fn Navbar() -> Element {
    let lang = use_i18n().lang;
    let mut mode_ctx = use_mode();
    let l = *lang.read();
    let m = *mode_ctx.mode.read();
    let label = tr(l, if m == Mode::Dark { "switcher.light" } else { "switcher.dark" });

    rsx! {
        header { class: "sticky top-0 z-10 bg-bg/80 backdrop-blur border-b border-border",
            div { class: "wrap flex items-center justify-between py-4",
                Link { class: "font-bold text-lg tracking-tight", to: Route::LandingPage {}, {tr(l, "brand")} }
                nav { class: "hidden md:flex items-center gap-6",
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.home")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.jobs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.docs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::ContactPage {}, {tr(l, "nav.contact")} }
                    Link { class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-sm font-medium border border-primary text-primary bg-transparent hover:bg-primary/10 transition", to: Route::LoginPage {}, {tr(l, "nav.login")} }
                    Link {
                        class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-sm font-medium bg-primary text-on-primary hover:opacity-90 transition hover:-translate-y-0.5",
                        to: Route::SignupPage {},
                        {tr(l, "nav.signup")}
                    }
                }
                div { class: "flex items-center gap-2",
                    LanguageSwitcher {}
                    ThemeSwitcher {}
                    button {
                        class: "relative z-50 inline-flex items-center justify-center w-9 h-9 rounded-full border border-border bg-surface text-fg cursor-pointer select-none transition-transform duration-150 hover:scale-105 active:scale-95",
                        r#type: "button",
                        aria_label: label,
                        title: label,
                        onclick: move |_| {
                            let next = if *mode_ctx.mode.read() == Mode::Dark { Mode::Light } else { Mode::Dark };
                            apply_mode(next);
                            *mode_ctx.mode.write() = next;
                        },
                        if m == Mode::Dark {
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.5",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "w-4 h-4",
                                circle { cx: "12", cy: "12", r: "4" }
                                path { d: "M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" }
                            }
                        } else {
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.5",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                class: "w-4 h-4",
                                path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
                            }
                        }
                    }
                }
            }
        }
        main {
            Outlet::<Route> {}
        }
    }
}

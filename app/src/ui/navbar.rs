use crate::i18n::{tr, use_i18n};
use crate::route::Route;
use crate::server::auth::guest::use_auth_opt;
use crate::theme::{apply_mode, use_mode, Mode};
use crate::ui::{LanguageSwitcher, ThemeSwitcher};
use dioxus::prelude::*;

/// Marketing Navbar — only for (marketing) group.
/// Contains: brand + nav.home/jobs/docs/contact + Language/Theme + sun/moon + Login/Signup + Invitado.
/// No dashboard/sidebar links. Dashboard chrome lives in DashboardLayout.
#[component]
pub fn Navbar() -> Element {
    let lang = use_i18n().lang;
    let mut mode_ctx = use_mode();
    let l = *lang.read();
    let m = *mode_ctx.mode.read();
    let auth_opt = use_auth_opt();
    let user_opt = auth_opt.as_ref().and_then(|a| a.user.read().clone());
    let is_guest = user_opt.as_ref().map(|u| u.is_guest).unwrap_or(true);
    let label = tr(
        l,
        if m == Mode::Dark {
            "switcher.light"
        } else {
            "switcher.dark"
        },
    );

    rsx! {
        // a11y: skip link for keyboard users — targets MarketingLayout main#main-content
        a {
            class: "sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 focus:bg-bg focus:text-fg focus:border focus:border-border focus:rounded-lg focus:px-4 focus:py-2",
            href: "#main-content",
            "Saltar al contenido"
        }
        header { class: "sticky top-0 z-10 bg-bg/80 backdrop-blur border-b border-border animate-nav-in",
            div { class: "wrap flex items-center justify-between py-4",
                Link { class: "font-bold text-lg tracking-tight", to: Route::LandingPage {}, {tr(l, "brand")} }
                nav { class: "hidden md:flex items-center gap-6", aria_label: "Principal",
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "nav.home")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::DocsPage {}, {tr(l, "nav.jobs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::DocsPage {}, {tr(l, "nav.docs")} }
                    Link { class: "text-muted text-[15px] hover:text-fg transition-colors", to: Route::ContactPage {}, {tr(l, "nav.contact")} }
                    if is_guest {
                        Link { class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-sm font-medium border border-primary text-primary bg-transparent hover:bg-primary/10 transition", to: Route::LoginPage {}, {tr(l, "nav.login")} }
                        Link {
                            class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-sm font-medium bg-primary text-on-primary hover:opacity-90 transition hover:-translate-y-0.5",
                            to: Route::SignupPage {},
                            {tr(l, "nav.signup")}
                        }
                    } else {
                        // Authenticated (gmail) — marketing still minimal, just go to dashboard
                        if let Some(u) = user_opt.clone() {
                            span { class: "hidden lg:inline text-xs text-muted truncate max-w-[160px]", "{u.email}" }
                        }
                        Link { class: "inline-flex items-center justify-center rounded-xl px-5 py-2.5 text-sm font-medium bg-primary text-on-primary hover:opacity-90 transition", to: Route::ClientDashboardGuard {}, "Dashboard" }
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
    }
}

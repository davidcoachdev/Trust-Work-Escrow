use crate::i18n::{tr, use_i18n};
use crate::ui::{Reveal, RevealVariant};
use dioxus::prelude::*;

#[component]
pub fn DocsPage() -> Element {
    let lang = *use_i18n().lang.read();
    let mut search = use_signal(|| String::new());
    let mut selected = use_signal(|| "overview".to_string());

    // Sync initial hash on mount
    #[cfg(target_arch = "wasm32")]
    {
        let mut sel = selected;
        use_effect(move || {
            if let Some(win) = web_sys::window() {
                if let Ok(hash) = win.location().hash() {
                    let clean = hash.trim_start_matches('#').trim().to_string();
                    if !clean.is_empty() {
                        sel.set(clean);
                    }
                }
            }
        });
        // Scroll + update hash when selected changes
        use_effect(move || {
            let id = selected.read().clone();
            if let Some(win) = web_sys::window() {
                if let Some(doc) = win.document() {
                    if let Some(el) = doc.get_element_by_id(&id) {
                        let _ = el.scroll_into_view();
                    }
                }
                let _ = win.location().set_hash(&id);
            }
        });
    }

    let link_cls = |id: &str, sel: &str| {
        if sel == id {
            "block px-3 py-1.5 rounded text-sm bg-[#f5f5f5] dark:bg-zinc-800 text-primary font-mono font-medium border-l-2 border-primary"
        } else {
            "block px-3 py-1.5 rounded text-sm text-muted hover:text-fg hover:bg-[#f5f5f5] dark:hover:bg-zinc-800 font-mono border-l-2 border-transparent"
        }
    };
    let sel_str = selected.read().clone();

    // Helper to decide visibility for main sections: only selected visible
    let visible = |id: &str| if sel_str == id { "" } else { "hidden" };

    rsx! {
        div { class: "bg-[#2a2a2a] text-zinc-100 border-b border-zinc-700",
            div { class: "wrap flex items-center gap-4 py-3",
                span { class: "font-mono text-sm font-bold tracking-tight", "trust-work-escrow 0.1.0" }
                span { class: "hidden sm:inline text-zinc-500 font-mono text-xs", "· docs.rs style" }
                div { class: "flex-1" }
                div { class: "relative w-full max-w-[360px]",
                    input {
                        class: "w-full bg-white text-zinc-900 font-mono text-sm rounded px-3 py-1.5 pr-8 border border-zinc-300 focus:outline-none focus:border-primary placeholder:text-zinc-400",
                        placeholder: "{tr(lang, \"docs.search.placeholder\")}",
                        value: "{search.read()}",
                        oninput: move |e| search.set(e.value()),
                    }
                    span { class: "absolute right-2 top-1/2 -translate-y-1/2 text-zinc-400 text-xs", "🔍" }
                }
            }
        }

        div { class: "bg-[#f5f5f5] dark:bg-zinc-900 border-b border-border",
            div { class: "wrap py-2 flex items-center gap-2 text-xs font-mono text-muted",
                a { class: "hover:text-primary", href: "/", "trust-work-escrow" }
                span { "›" }
                span { class: "text-fg", "{tr(lang, \"docs.breadcrumb.docs\")}" }
                span { class: "ml-auto hidden md:inline text-muted", "On crates.io: not yet · devnet 7a2Y…" }
            }
        }

        div { class: "wrap flex gap-0 md:gap-8 py-6",
            aside { class: "hidden md:block w-[240px] shrink-0 sticky top-[72px] h-fit",
                div { class: "bg-surface border border-border rounded-xl overflow-hidden",
                    div { class: "px-3 py-2 bg-[#f5f5f5] dark:bg-zinc-800 border-b border-border font-mono text-xs font-bold text-fg uppercase tracking-widest", "{tr(lang, \"docs.modules\")}" }
                    nav { class: "p-2 space-y-0.5",
                        a { class: "{link_cls(\"overview\", &sel_str)}", href: "#overview", onclick: {
                            let mut s = selected; move |_| s.set("overview".to_string())
                        }, "{tr(lang, \"docs.overview\")}" }
                        a { class: "{link_cls(\"architecture\", &sel_str)}", href: "#architecture", onclick: {
                            let mut s = selected; move |_| s.set("architecture".to_string())
                        }, "{tr(lang, \"docs.architecture\")}" }
                        a { class: "{link_cls(\"getting-started\", &sel_str)}", href: "#getting-started", onclick: {
                            let mut s = selected; move |_| s.set("getting-started".to_string())
                        }, "{tr(lang, \"docs.gettingStarted\")}" }
                        a { class: "{link_cls(\"api\", &sel_str)}", href: "#api", onclick: {
                            let mut s = selected; move |_| s.set("api".to_string())
                        }, "{tr(lang, \"docs.api\")}" }
                        a { class: "{link_cls(\"sdk\", &sel_str)}", href: "#sdk", onclick: {
                            let mut s = selected; move |_| s.set("sdk".to_string())
                        }, "{tr(lang, \"docs.sdk\")}" }
                        a { class: "{link_cls(\"app\", &sel_str)}", href: "#app", onclick: {
                            let mut s = selected; move |_| s.set("app".to_string())
                        }, "{tr(lang, \"docs.app\")}" }
                        a { class: "{link_cls(\"deployment\", &sel_str)}", href: "#deployment", onclick: {
                            let mut s = selected; move |_| s.set("deployment".to_string())
                        }, "{tr(lang, \"docs.deployment\")}" }
                    }
                    div { class: "px-3 py-2 border-t border-border",
                        p { class: "font-mono text-[11px] text-muted leading-relaxed", "Program ID (devnet):" }
                        code { class: "font-mono text-[11px] break-all text-primary", "7a2Y… (see Explorer)" }
                    }
                }
                div { class: "mt-4 bg-amber-500/10 border border-amber-500/30 rounded-xl p-3",
                    p { class: "font-mono text-xs font-bold text-amber-700 dark:text-amber-300", "{tr(lang, \"docs.devnetOnly\")}" }
                    p { class: "font-mono text-[11px] text-muted mt-1", "{tr(lang, \"docs.devnetOnly.body\")}" }
                }
            }

            main { class: "flex-1 min-w-0 space-y-8",
                // Overview
                div { class: "{visible(\"overview\")}",
                    Reveal { variant: RevealVariant::FadeIn,
                        section { id: "overview", class: "scroll-mt-20",
                            h1 { class: "font-mono text-2xl md:text-3xl font-bold tracking-tight text-fg", "{tr(lang, \"docs.overview.title\")}" }
                            p { class: "font-mono text-xs text-muted mt-1", "{tr(lang, \"docs.overview.version\")}" }
                            p { class: "mt-4 text-[15px] leading-7 text-fg/90", "{tr(lang, \"docs.overview.body\")}" }
                            div { class: "mt-4 flex flex-wrap gap-2",
                                span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "solana-program" }
                                span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "axum" }
                                span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "dioxus 0.7" }
                                span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "ed25519 / HMAC" }
                            }
                        }
                    }
                }

                div { class: "{visible(\"architecture\")}",
                    Reveal { delay: 80,
                        section { id: "architecture", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                            h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                                span { class: "w-1 h-5 bg-primary rounded-full" }
                                "{tr(lang, \"docs.architecture.title\")}"
                            }
                            p { class: "text-sm text-muted mt-2", "{tr(lang, \"docs.architecture.body\")}" }
                            div { class: "mt-4 grid md:grid-cols-3 gap-3",
                                div { class: "bg-bg border border-border rounded-xl p-4",
                                    div { class: "font-mono text-xs font-bold text-primary", "trust_escrow_v3" }
                                    p { class: "text-xs text-muted mt-1", "Programa Solana devnet. PDA Job (escrow), fees 2.5%, custodio y tesorerías. ID: 7a2Y…" }
                                    a { class: "font-mono text-xs text-primary underline mt-2 inline-block", href: "https://explorer.solana.com/address/JCR9fRx9eMqr27jk2KvXSVFsewq7JxaAXHZg54YjjLTw?cluster=devnet", target: "_blank", "Ver PDA en Explorer →" }
                                }
                                div { class: "bg-bg border border-border rounded-xl p-4",
                                    div { class: "font-mono text-xs font-bold text-primary", "backend/api" }
                                    p { class: "text-xs text-muted mt-1", "Axum + Postgres + lettre (dev log). Auth guest/JWT, OTP email, jobs, escrow verify." }
                                    code { class: "font-mono text-[11px] bg-[#f5f5f5] dark:bg-zinc-800 rounded px-1.5 py-0.5 mt-2 inline-block", "GET /jobs · POST /jobs" }
                                }
                                div { class: "bg-bg border border-border rounded-xl p-4",
                                    div { class: "font-mono text-xs font-bold text-primary", "app (Dioxus)" }
                                    p { class: "text-xs text-muted mt-1", "Dioxus 0.7 Router + Tailwind. MarketingLayout y DashboardLayout, auth guest-aware." }
                                    code { class: "font-mono text-[11px] bg-[#f5f5f5] dark:bg-zinc-800 rounded px-1.5 py-0.5 mt-2 inline-block", "dx serve · cargo check" }
                                }
                            }
                        }
                    }
                }

                div { class: "{visible(\"getting-started\")}",
                    Reveal { delay: 120,
                        section { id: "getting-started", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                            h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                                span { class: "w-1 h-5 bg-primary rounded-full" }
                                "{tr(lang, \"docs.gettingStarted.title\")}"
                            }
                            div { class: "mt-4 space-y-4",
                                div {
                                    h3 { class: "font-mono text-sm font-bold", "1. Programa (devnet)" }
                                    div { class: "mt-2 bg-[#2a2a2a] text-zinc-100 rounded-lg p-3 overflow-x-auto",
                                        pre { class: "font-mono text-xs leading-relaxed",
                                            code { "cargo build-sbf --manifest-path trust_escrow_v3/Cargo.toml\nsolana program deploy --url devnet target/deploy/trust_escrow_v3.so # 7a2Y…" }
                                        }
                                    }
                                }
                                div {
                                    h3 { class: "font-mono text-sm font-bold", "2. Backend" }
                                    div { class: "mt-2 bg-[#2a2a2a] text-zinc-100 rounded-lg p-3 overflow-x-auto",
                                        pre { class: "font-mono text-xs leading-relaxed",
                                            code { "cargo run -p trust-work-escrow-backend\n# env: DATABASE_URL, JWT_SECRET, SMTP_*" }
                                        }
                                    }
                                }
                                div {
                                    h3 { class: "font-mono text-sm font-bold", "3. App" }
                                    div { class: "mt-2 bg-[#2a2a2a] text-zinc-100 rounded-lg p-3 overflow-x-auto",
                                        pre { class: "font-mono text-xs leading-relaxed",
                                            code { "cargo check -p trust-work-escrow-app\ndx serve --port 8080\n# o docker compose up" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "{visible(\"api\")}",
                    Reveal { delay: 160,
                        section { id: "api", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                            h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                                span { class: "w-1 h-5 bg-primary rounded-full" }
                                "{tr(lang, \"docs.api.title\")}"
                            }
                            p { class: "text-sm text-muted mt-2", "{tr(lang, \"docs.api.body\")}" }
                            div { class: "mt-4 space-y-3 font-mono text-xs",
                                div { class: "bg-bg border border-border rounded-lg p-3",
                                    div { class: "flex gap-2 items-center",
                                        span { class: "bg-green-600 text-white px-1.5 py-0.5 rounded text-[11px]", "GET" }
                                        code { class: "text-primary", "/jobs" }
                                        span { class: "text-muted", " — lista jobs (monto real, no hardcode)" }
                                    }
                                }
                                div { class: "bg-bg border border-border rounded-lg p-3",
                                    div { class: "flex gap-2 items-center",
                                        span { class: "bg-emerald-700 text-white px-1.5 py-0.5 rounded text-[11px]", "POST" }
                                        code { class: "text-primary", "/auth/send-otp" }
                                        span { class: "text-muted", " — email → OTP" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "{visible(\"sdk\")}",
                    Reveal { delay: 200,
                        section { id: "sdk", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                            h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                                span { class: "w-1 h-5 bg-primary rounded-full" }
                                "{tr(lang, \"docs.sdk.title\")}"
                            }
                            p { class: "text-sm text-muted mt-2", "{tr(lang, \"docs.sdk.body\")}" }
                        }
                    }
                }

                div { class: "{visible(\"app\")}",
                    Reveal { delay: 240,
                        section { id: "app", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                            h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                                span { class: "w-1 h-5 bg-primary rounded-full" }
                                "{tr(lang, \"docs.app.title\")}"
                            }
                            p { class: "text-sm text-muted mt-2", "Routes: / · /docs · /login · /dashboard/*" }
                        }
                    }
                }

                div { class: "{visible(\"deployment\")}",
                    Reveal { delay: 280,
                        section { id: "deployment", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                            h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                                span { class: "w-1 h-5 bg-primary rounded-full" }
                                "{tr(lang, \"docs.deployment.title\")}"
                            }
                            p { class: "text-sm text-muted mt-2", "Program ID: 7a2Y… (devnet) · PDA JCR9…LTw" }
                        }
                    }
                }

                div { class: "border-t border-border pt-4 flex flex-wrap gap-4 font-mono text-xs text-muted",
                    a { class: "hover:text-primary", href: "https://github.com/davidcoachdev/Trust-Work-Escrow", target: "_blank", "GitHub" }
                    a { class: "hover:text-primary", href: "/", "← Volver al inicio" }
                    span { class: "ml-auto", "© 2026 Trust Work Escrow · docs style Rust" }
                }
            }
        }
    }
}

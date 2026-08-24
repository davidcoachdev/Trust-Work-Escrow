use crate::ui::{Reveal, RevealVariant};
use dioxus::prelude::*;

/// DocsPage — mimics docs.rs / Rust documentation style for Trust Work Escrow.
/// Header #2a2a2a, sidebar with modules, main with sections, code blocks with font-mono.
#[component]
pub fn DocsPage() -> Element {
    let mut search = use_signal(|| String::new());
    let mut selected = use_signal(|| "overview".to_string());

    // Helper to render active link class
    let link_cls = |id: &str, sel: &str| {
        if sel == id {
            "block px-3 py-1.5 rounded text-sm bg-[#f5f5f5] dark:bg-zinc-800 text-primary font-mono font-medium border-l-2 border-primary"
        } else {
            "block px-3 py-1.5 rounded text-sm text-muted hover:text-fg hover:bg-[#f5f5f5] dark:hover:bg-zinc-800 font-mono border-l-2 border-transparent"
        }
    };

    rsx! {
        // ── Rust docs dark header ──
        div { class: "bg-[#2a2a2a] text-zinc-100 border-b border-zinc-700",
            div { class: "wrap flex items-center gap-4 py-3",
                span { class: "font-mono text-sm font-bold tracking-tight", "trust-work-escrow 0.1.0" }
                span { class: "hidden sm:inline text-zinc-500 font-mono text-xs", "· docs.rs style" }
                div { class: "flex-1" }
                // mock search
                div { class: "relative w-full max-w-[360px]",
                    input {
                        class: "w-full bg-white text-zinc-900 font-mono text-sm rounded px-3 py-1.5 pr-8 border border-zinc-300 focus:outline-none focus:border-primary placeholder:text-zinc-400",
                        placeholder: "Search (mock) — e.g. escrow",
                        value: "{search.read()}",
                        oninput: move |e| search.set(e.value()),
                    }
                    span { class: "absolute right-2 top-1/2 -translate-y-1/2 text-zinc-400 text-xs", "🔍" }
                }
            }
        }

        // ── Breadcrumb ──
        div { class: "bg-[#f5f5f5] dark:bg-zinc-900 border-b border-border",
            div { class: "wrap py-2 flex items-center gap-2 text-xs font-mono text-muted",
                a { class: "hover:text-primary", href: "/", "trust-work-escrow" }
                span { "›" }
                span { class: "text-fg", "docs" }
                span { class: "ml-auto hidden md:inline text-muted", "On crates.io: not yet · devnet 7a2Y…" }
            }
        }

        // ── Layout: sidebar + main ──
        div { class: "wrap flex gap-0 md:gap-8 py-6",
            // Sidebar
            aside { class: "hidden md:block w-[240px] shrink-0 sticky top-[72px] h-fit",
                div { class: "bg-surface border border-border rounded-xl overflow-hidden",
                    div { class: "px-3 py-2 bg-[#f5f5f5] dark:bg-zinc-800 border-b border-border font-mono text-xs font-bold text-fg uppercase tracking-widest", "Modules" }
                    nav { class: "p-2 space-y-0.5",
                        a { class: "{link_cls(\"overview\", &selected.read())}", href: "#overview", onclick: move |_| selected.set("overview".to_string()), "overview" }
                        a { class: "{link_cls(\"architecture\", &selected.read())}", href: "#architecture", onclick: move |_| selected.set("architecture".to_string()), "architecture" }
                        a { class: "{link_cls(\"getting-started\", &selected.read())}", href: "#getting-started", onclick: move |_| selected.set("getting-started".to_string()), "getting_started" }
                        a { class: "{link_cls(\"api\", &selected.read())}", href: "#api", onclick: move |_| selected.set("api".to_string()), "backend::api" }
                        a { class: "{link_cls(\"sdk\", &selected.read())}", href: "#sdk", onclick: move |_| selected.set("sdk".to_string()), "backend::sdk" }
                        a { class: "{link_cls(\"app\", &selected.read())}", href: "#app", onclick: move |_| selected.set("app".to_string()), "app (Dioxus)" }
                        a { class: "{link_cls(\"deployment\", &selected.read())}", href: "#deployment", onclick: move |_| selected.set("deployment".to_string()), "deployment" }
                    }
                    div { class: "px-3 py-2 border-t border-border",
                        p { class: "font-mono text-[11px] text-muted leading-relaxed", "Program ID (devnet):" }
                        code { class: "font-mono text-[11px] break-all text-primary", "7a2Y… (see Explorer)" }
                    }
                }
                div { class: "mt-4 bg-amber-500/10 border border-amber-500/30 rounded-xl p-3",
                    p { class: "font-mono text-xs font-bold text-amber-700 dark:text-amber-300", "Devnet only" }
                    p { class: "font-mono text-[11px] text-muted mt-1", "No mainnet funds. Explorer links open devnet." }
                }
            }

            // Main content
            main { class: "flex-1 min-w-0 space-y-8",
                Reveal { variant: RevealVariant::FadeIn,
                    section { id: "overview", class: "scroll-mt-20",
                        h1 { class: "font-mono text-2xl md:text-3xl font-bold tracking-tight text-fg", "Crate trust_work_escrow" }
                        p { class: "font-mono text-xs text-muted mt-1", "Version 0.1.0 · devnet · Solana · Axum · Dioxus 0.7" }
                        p { class: "mt-4 text-[15px] leading-7 text-fg/90",
                            "Trust Work Escrow es un sistema de escrow descentralizado para trabajo freelance. Los fondos se bloquean on-chain en un programa de Solana (devnet 7a2Y…) hasta que cliente y freelancer acuerdan la liberación. El backend verifica, el SDK firma, y la app Dioxus orquesta la experiencia."
                        }
                        div { class: "mt-4 flex flex-wrap gap-2",
                            span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "solana-program" }
                            span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "axum" }
                            span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "dioxus 0.7" }
                            span { class: "font-mono text-xs bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded px-2 py-1", "ed25519 / HMAC" }
                        }
                    }
                }

                Reveal { delay: 80,
                    section { id: "architecture", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                        h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                            span { class: "w-1 h-5 bg-primary rounded-full" }
                            "Architecture"
                        }
                        p { class: "text-sm text-muted mt-2", "Tres capas: programa on-chain, backend Axum, y app Dioxus (fullstack)." }
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
                        div { class: "mt-4 bg-[#f5f5f5] dark:bg-zinc-900 border border-border rounded-lg p-3 overflow-x-auto",
                            pre { class: "font-mono text-xs leading-relaxed text-fg",
                                "┌─────────┐     ┌──────────┐     ┌──────────────┐\n"
                                "│   App   │────▶│ Backend  │────▶│   Solana     │\n"
                                "│ Dioxus  │◀────│  Axum    │◀────│ 7a2Y program │\n"
                                "└─────────┘     └──────────┘     └──────────────┘\n"
                                "  OTP/JWT        HMAC wallet        PDA escrow"
                            }
                        }
                    }
                }

                Reveal { delay: 120,
                    section { id: "getting-started", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                        h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                            span { class: "w-1 h-5 bg-primary rounded-full" }
                            "Getting Started"
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
                        div { class: "mt-4 bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded-lg p-3",
                            p { class: "font-mono text-xs font-bold", "Requisitos" }
                            p { class: "font-mono text-xs text-muted mt-1", "Rust stable, Dioxus CLI (dx), Solana CLI, Postgres 15+. Ver README y docker-compose.yml." }
                        }
                    }
                }

                Reveal { delay: 160,
                    section { id: "api", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                        h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                            span { class: "w-1 h-5 bg-primary rounded-full" }
                            "API Docs — backend/api"
                        }
                        p { class: "text-sm text-muted mt-2", "Base: /api. Auth via cookie twe-jwt o twe-guest (guest 24h). Mutaciones requieren JWT." }
                        div { class: "mt-4 space-y-3 font-mono text-xs",
                            div { class: "bg-bg border border-border rounded-lg p-3",
                                div { class: "flex gap-2 items-center",
                                    span { class: "bg-green-600 text-white px-1.5 py-0.5 rounded text-[11px]", "GET" }
                                    code { class: "text-primary", "/jobs" }
                                    span { class: "text-muted", " — lista jobs (monto real, no hardcode)" }
                                }
                                div { class: "mt-2 bg-[#f5f5f5] dark:bg-zinc-800 rounded p-2 overflow-x-auto",
                                    pre { "curl http://localhost:3000/api/jobs | jq" }
                                }
                            }
                            div { class: "bg-bg border border-border rounded-lg p-3",
                                div { class: "flex gap-2 items-center",
                                    span { class: "bg-emerald-700 text-white px-1.5 py-0.5 rounded text-[11px]", "POST" }
                                    code { class: "text-primary", "/auth/send-otp" }
                                    span { class: "text-muted", " — email → OTP" }
                                }
                                div { class: "flex gap-2 items-center mt-1",
                                    span { class: "bg-emerald-700 text-white px-1.5 py-0.5 rounded text-[11px]", "POST" }
                                    code { class: "text-primary", "/auth/verify-otp" }
                                    span { class: "text-muted", " — email+code → set-cookie twe-jwt" }
                                }
                                div { class: "mt-2 bg-[#f5f5f5] dark:bg-zinc-800 rounded p-2 overflow-x-auto",
                                    pre { "curl -X POST /api/auth/send-otp -d '{{\"email\":\"tu@correo.com\"}}'\n# dev: revisa logs si no hay SMTP" }
                                }
                            }
                            div { class: "bg-bg border border-border rounded-lg p-3",
                                div { class: "flex gap-2 items-center",
                                    span { class: "bg-emerald-700 text-white px-1.5 py-0.5 rounded text-[11px]", "POST" }
                                    code { class: "text-primary", "/escrow/*" }
                                    span { class: "text-muted", " — create/apply/release, verifica PDA 7a2Y" }
                                }
                            }
                        }
                    }
                }

                Reveal { delay: 200,
                    section { id: "sdk", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                        h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                            span { class: "w-1 h-5 bg-primary rounded-full" }
                            "SDK Docs — backend/sdk"
                        }
                        p { class: "text-sm text-muted mt-2", "El backend usa trust-escrow-sdk (features = [solana]) para construir instrucciones para 7a2Y; el navegador solo solicita, firma con Phantom y retransmite." }
                        div { class: "mt-3 bg-[#2a2a2a] text-zinc-100 rounded-lg p-3 overflow-x-auto",
                            pre { class: "font-mono text-xs leading-relaxed",
                                code {
                                    "use trust_escrow_sdk::instruction::create_job;\n"
                                    "let ix = create_job(\n"
                                    "    &client_pubkey, &pda_job, amount_lamports,\n"
                                    "    &program_id // 7a2Y…\n"
                                    ");\n"
                                    "// el backend devuelve unsigned tx; Phantom firma en el navegador"
                                }
                            }
                        }
                        div { class: "mt-3 bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded-lg p-3",
                            p { class: "font-mono text-xs", "Wallet no custodial: la clave privada permanece en Phantom y nunca se genera ni se expone en Trust Work." }
                        }
                    }
                }

                Reveal { delay: 240,
                    section { id: "app", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                        h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                            span { class: "w-1 h-5 bg-primary rounded-full" }
                            "App — Dioxus 0.7"
                        }
                        div { class: "mt-3 grid md:grid-cols-2 gap-3 font-mono text-xs",
                            div { class: "bg-bg border border-border rounded-lg p-3",
                                div { class: "font-bold", "Routes" }
                                ul { class: "mt-2 space-y-1 text-muted list-disc list-inside",
                                    li { code { class: "bg-[#f5f5f5] dark:bg-zinc-800 px-1 rounded", "/" } " Landing (MarketingLayout)" }
                                    li { code { class: "bg-[#f5f5f5] dark:bg-zinc-800 px-1 rounded", "/docs" } " Docs (esta página)" }
                                    li { code { class: "bg-[#f5f5f5] dark:bg-zinc-800 px-1 rounded", "/login, /signup" } " Auth OTP" }
                                    li { code { class: "bg-[#f5f5f5] dark:bg-zinc-800 px-1 rounded", "/dashboard/*" } " DashboardLayout (único Sidebar)" }
                                }
                            }
                            div { class: "bg-bg border border-border rounded-lg p-3",
                                div { class: "font-bold", "Auth & Wallet" }
                                ul { class: "mt-2 space-y-1 text-muted list-disc list-inside",
                                    li { "Guest cookie twe-guest (24h) → solo lectura" }
                                    li { "OTP → JWT twe-jwt → puede crear jobs" }
                                    li { "Wallet HMAC solo en /dashboard/config" }
                                    li { "Sin wallet → CTA a Config, no a modal" }
                                }
                            }
                        }
                    }
                }

                Reveal { delay: 280,
                    section { id: "deployment", class: "scroll-mt-20 bg-surface border border-border rounded-xl p-6",
                        h2 { class: "font-mono text-lg font-bold text-fg flex items-center gap-2",
                            span { class: "w-1 h-5 bg-primary rounded-full" }
                            "Deployment — devnet"
                        }
                        div { class: "mt-3 bg-[#f5f5f5] dark:bg-zinc-800 border border-border rounded-lg p-3 overflow-x-auto",
                            pre { class: "font-mono text-xs leading-relaxed",
                                "Program ID: 7a2Y… (devnet)\n"
                                "PDA Job demo: JCR9fRx9eMqr27jk2KvXSVFsewq7JxaAXHZg54YjjLTw (0.115 SOL)\n"
                                "Explorer: https://explorer.solana.com/address/JCR9...?cluster=devnet\n"
                                "Backend: Axum + Postgres (docker-compose)\n"
                                "App: dx serve / dx bundle"
                            }
                        }
                        div { class: "mt-3 flex flex-wrap gap-2",
                            a { class: "font-mono text-xs bg-primary text-on-primary rounded px-3 py-1.5", href: "https://explorer.solana.com/address/JCR9fRx9eMqr27jk2KvXSVFsewq7JxaAXHZg54YjjLTw?cluster=devnet", target: "_blank", "Explorer PDA →" }
                            a { class: "font-mono text-xs bg-bg border border-border rounded px-3 py-1.5", href: "https://github.com/davidcoachdev/Trust-Work-Escrow", target: "_blank", "GitHub →" }
                        }
                        p { class: "font-mono text-[11px] text-muted mt-3", "Nota: devnet no usa fondos reales. No desplegar a mainnet sin auditoría." }
                    }
                }

                // Footer inline docs
                div { class: "border-t border-border pt-4 flex flex-wrap gap-4 font-mono text-xs text-muted",
                    a { class: "hover:text-primary", href: "https://github.com/davidcoachdev/Trust-Work-Escrow", target: "_blank", "GitHub" }
                    a { class: "hover:text-primary", href: "/", "← Volver al inicio" }
                    span { class: "ml-auto", "© 2026 Trust Work Escrow · docs style Rust" }
                }
            }
        }
    }
}

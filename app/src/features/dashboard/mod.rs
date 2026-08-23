pub mod config;
pub use config::ConfigPage;

use dioxus::prelude::*;
use crate::route::Route;
use crate::server::auth::guest::use_auth;
use crate::ui::{DashboardLayout, Reveal};

// Re-export single DashboardLayout aliases for backward compat — old layouts were separate and caused 5 sidebars
#[allow(unused_imports)]
pub use crate::ui::DashboardLayout as ClientLayoutComponent;
#[allow(unused_imports)]
pub use crate::ui::DashboardLayout as FreelancerLayoutComponent;
#[allow(unused_imports)]
pub use crate::ui::DashboardLayout as AdminLayoutComponent;

// Keep thin wrappers if someone still imports ClientLayout etc directly
#[component]
pub fn ClientLayout() -> Element {
    rsx! { DashboardLayout {} }
}
#[component]
pub fn FreelancerLayout() -> Element {
    rsx! { DashboardLayout {} }
}
#[component]
pub fn AdminLayout() -> Element {
    rsx! { DashboardLayout {} }
}

#[component]
pub fn ClientDashboard() -> Element {
    let mut title = use_signal(|| String::new());
    let mut amount_sol = use_signal(|| "0.1".to_string());
    let mut msg = use_signal(|| String::new());
    let auth = use_auth();
    let is_guest = auth.user.read().as_ref().map(|u| u.is_guest).unwrap_or(true);
    let has_wallet = auth.user.read().as_ref().and_then(|u| u.wallet_pubkey.clone()).is_some();
    let can_act = !is_guest && has_wallet;
    rsx! {
        div { class: "space-y-6",
            if is_guest {
                Reveal {
                    div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl px-4 py-3 flex items-center justify-between",
                        span { class: "text-sm text-amber-700 dark:text-amber-300", "Modo invitado · Solo lectura" }
                        Link { class: "text-sm text-primary underline", to: Route::LoginPage {}, "Iniciar sesión" }
                    }
                }
            }
            // Wallet gate banner — shown when authenticated but no wallet
            if !is_guest && !has_wallet {
                Reveal { delay: 80,
                    div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl px-4 py-3 flex flex-wrap items-center justify-between gap-3",
                        span { class: "text-sm text-amber-700 dark:text-amber-300", "Conectá tu billetera en Config > Wallet para habilitar acciones" }
                        Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-4 py-2 text-sm font-medium", to: Route::ConfigPage {}, "Ir a Config > Wallet" }
                    }
                }
            }
            Reveal {
                h1 { class: "text-3xl font-bold text-primary", "Dashboard Cliente" }
            }
            Reveal { delay: 80,
                p { class: "text-muted", "Crea jobs on-chain devnet 7a2Y... y ve tu escrow congelado." }
            }
            // Job listing — always visible, read-only
            Reveal { delay: 120,
                div { class: "bg-surface border border-border rounded-2xl p-6 space-y-3",
                    div { class: "flex justify-between items-center",
                        span { class: "text-sm font-medium", "Escrow demo JCR9..." }
                        a { class: "text-xs text-primary underline", href: "https://explorer.solana.com/address/JCR9fRx9eMqr27jk2KvXSVFsewq7JxaAXHZg54YjjLTw?cluster=devnet", target: "_blank", "Ver en Explorer" }
                    }
                    div { class: "grid grid-cols-2 gap-4 text-sm",
                        div { class: "bg-bg border border-border rounded-xl p-3",
                            div { class: "text-muted text-xs", "PDA Balance" }
                            div { class: "font-mono font-bold text-primary", "0.115 SOL (115M lamports)" }
                            div { class: "text-xs text-muted", "100M + 2.5M fee + rent" }
                        }
                        div { class: "bg-bg border border-border rounded-xl p-3",
                            div { class: "text-muted text-xs", "Estado" }
                            div { class: "font-bold text-title", "Funded ✅" }
                            div { class: "text-xs text-muted", "Solo client 3whY... puede liberar" }
                        }
                    }
                    p { class: "text-xs text-muted", "Fix aplicado: GET /jobs ya devuelve amount real (no 1_000_000 hardcode)" }
                }
            }
            // Only show create form when wallet present; otherwise show CTA
            if can_act {
                Reveal { delay: 180,
                    div { class: "bg-surface border border-border rounded-2xl p-6 space-y-4",
                        h2 { class: "text-xl font-bold", "Crear nuevo Job (devnet)" }
                        form { class: "grid gap-3",
                            onsubmit: move |evt| {
                                evt.prevent_default();
                                let t = title.read().clone();
                                let amt = amount_sol.read().clone();
                                spawn(async move {
                                    msg.set(format!("Creando '{}' por {} SOL en devnet 7a2Y... (usa SDK devnet)", t, amt));
                                });
                            },
                            input { class: "bg-bg border border-border rounded-xl px-3 py-2 text-fg",
                                placeholder: "Título (ej: Landing Trust Work)",
                                value: "{title.read()}",
                                oninput: move |e| title.set(e.value()),
                            }
                            input { class: "bg-bg border border-border rounded-xl px-3 py-2 text-fg",
                                placeholder: "0.1",
                                value: "{amount_sol.read()}",
                                oninput: move |e| amount_sol.set(e.value()),
                            }
                            button { class: "bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium hover:opacity-90 transition hover:-translate-y-0.5 active:scale-[0.98]", r#type: "submit", "Crear Job (usa 3whY... en devnet)" }
                            if !msg.read().is_empty() {
                                p { class: "text-sm text-primary", "{msg.read()}" }
                            }
                        }
                    }
                }
            } else {
                Reveal { delay: 180,
                    div { class: "bg-surface border border-dashed border-border rounded-2xl p-6 text-center space-y-3",
                        p { class: "text-sm text-muted", "Necesitás billetera para crear jobs, firmar y liberar escrow." }
                        if is_guest {
                            Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium transition hover:-translate-y-0.5", to: Route::LoginPage {}, "Iniciar sesión" }
                        } else {
                            Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium transition hover:-translate-y-0.5", to: Route::ConfigPage {}, "Crear mi billetera en Config" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn FreelancerDashboard() -> Element {
    let auth = use_auth();
    let is_guest = auth.user.read().as_ref().map(|u| u.is_guest).unwrap_or(true);
    let has_wallet = auth.user.read().as_ref().and_then(|u| u.wallet_pubkey.clone()).is_some();
    let can_act = !is_guest && has_wallet;
    let mut apply_msg = use_signal(|| String::new());
    rsx! {
        div { class: "space-y-6",
            if is_guest {
                Reveal {
                    div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl px-4 py-3 flex items-center justify-between",
                        span { class: "text-sm text-amber-700 dark:text-amber-300", "Modo invitado · Solo lectura" }
                        Link { class: "text-sm text-primary underline", to: Route::LoginPage {}, "Iniciar sesión" }
                    }
                }
            }
            if !is_guest && !has_wallet {
                Reveal { delay: 80,
                    div { class: "bg-amber-500/10 border border-amber-500/30 rounded-xl px-4 py-3 flex flex-wrap items-center justify-between gap-3",
                        span { class: "text-sm text-amber-700 dark:text-amber-300", "Conectá tu billetera en Config > Wallet para habilitar acciones" }
                        Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-4 py-2 text-sm font-medium", to: Route::ConfigPage {}, "Ir a Config > Wallet" }
                    }
                }
            }
            Reveal {
                h1 { class: "text-3xl font-bold text-primary", "Dashboard Freelancer" }
            }
            Reveal { delay: 80,
                p { class: "text-muted", "Jobs disponibles en devnet 7a2Y... y tus postulaciones." }
            }
            // Listing — always visible read-only
            Reveal { delay: 120,
                div { class: "bg-surface border border-border rounded-2xl p-6 space-y-3",
                    div { class: "flex justify-between items-center",
                        span { class: "font-medium", "Job #0 — Landing Trust Work" }
                        span { class: "text-xs bg-primary text-on-primary px-2 py-1 rounded-full", "Funded" }
                    }
                    div { class: "grid grid-cols-2 gap-3 text-sm",
                        div { class: "bg-bg border border-border rounded-xl p-3",
                            div { class: "text-muted text-xs", "PDA Job" }
                            a { class: "font-mono text-xs text-primary underline break-all", href: "https://explorer.solana.com/address/JCR9fRx9eMqr27jk2KvXSVFsewq7JxaAXHZg54YjjLTw?cluster=devnet", target: "_blank", "JCR9...LTw" }
                            div { class: "text-xs", "0.115 SOL congelados" }
                        }
                        div { class: "bg-bg border border-border rounded-xl p-3",
                            div { class: "text-muted text-xs", "Tu aplicación" }
                            a { class: "font-mono text-xs text-primary underline break-all", href: "https://explorer.solana.com/address/B5KscoN4F35K8EAxLUhuobqosWneB82DxnFb3fX64ys7?cluster=devnet", target: "_blank", "B5Ks...ys7" }
                            div { class: "text-xs", "Pending — hash 056934..." }
                        }
                    }
                    a { class: "inline-flex bg-primary text-on-primary rounded-xl px-4 py-2 text-sm font-medium", href: "https://explorer.solana.com/tx/3FDyPuuwEni6KqtafBcNrKDdAhY6vRJf53abFusdbLsHFo5i8wCXvBGeJeEA1mPCNXyTSyNNhNy3mWYLPtU3weRo?cluster=devnet", target: "_blank", "Ver Tx Apply 3FDy..." }
                    // Postular CTA — gated by wallet
                    if can_act {
                        button {
                            class: "mt-3 w-full bg-primary text-on-primary rounded-xl px-4 py-2.5 font-medium hover:opacity-90 transition hover:-translate-y-0.5 active:scale-[0.98]",
                            r#type: "button",
                            onclick: move |_| {
                                apply_msg.set("Postulación enviada (mock) — firmando con tu wallet en devnet 7a2Y… Tx 3FDy…".to_string());
                                log::info!("apply to job #0");
                            },
                            "Postular a este Job"
                        }
                        if !apply_msg.read().is_empty() {
                            p { class: "text-sm text-primary text-center mt-2", "{apply_msg.read()}" }
                        }
                    } else {
                        div { class: "mt-3 space-y-2",
                            button { class: "w-full bg-bg border border-border rounded-xl px-4 py-2.5 font-medium opacity-50 cursor-not-allowed", r#type: "button", disabled: true, "Postular (requiere billetera)" }
                            if is_guest {
                                Link { class: "block text-xs text-primary underline text-center", to: Route::LoginPage {}, "Iniciá sesión para postularte →" }
                            } else {
                                p { class: "text-xs text-amber-600 text-center", "Conectá tu billetera en Config > Wallet para postularte" }
                            }
                        }
                    }
                }
            }
            if !can_act {
                Reveal { delay: 180,
                    div { class: "bg-surface border border-dashed border-border rounded-2xl p-6 text-center space-y-3",
                        p { class: "text-sm text-muted", "Sin billetera no podés firmar la postulación on-chain." }
                        if is_guest {
                            Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium transition hover:-translate-y-0.5", to: Route::LoginPage {}, "Iniciar sesión" }
                        } else {
                            Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium transition hover:-translate-y-0.5", to: Route::ConfigPage {}, "Crear mi billetera en Config" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AdminDashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            Reveal {
                h1 { class: "text-3xl font-bold text-primary", "Dashboard Admin" }
            }
            Reveal { delay: 80,
                p { class: "text-muted", "Métricas, treasuries y custody — solo manager/treasurer/custodian" }
            }
            Reveal { delay: 120,
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    div { class: "bg-surface border border-border rounded-2xl p-4",
                        div { class: "text-xs text-muted", "Manager" }
                        div { class: "text-sm font-bold", "TVL 0.115 SOL" }
                        div { class: "text-xs text-muted", "Jobs: 1 (JCR9...)" }
                    }
                    div { class: "bg-surface border border-border rounded-2xl p-4",
                        div { class: "text-xs text-muted", "Treasury" }
                        a { class: "font-mono text-xs text-primary underline break-all", href: "https://explorer.solana.com/address/6KSy8Mc2siExXqA1KGr6yQ9Dik2rp2rHe1dXHJaGCbi1?cluster=devnet", target: "_blank", "6KSy...GCbi1" }
                        div { class: "text-xs", "0.01 SOL" }
                    }
                    div { class: "bg-surface border border-border rounded-2xl p-4",
                        div { class: "text-xs text-muted", "Arb Treasury" }
                        a { class: "font-mono text-xs text-primary underline break-all", href: "https://explorer.solana.com/address/CfkGxwsaSbczzAePDjsdqvjyKLCS22isFh48Yhzz4dHb?cluster=devnet", target: "_blank", "CfkG...4Hb" }
                        div { class: "text-xs", "0.01 SOL" }
                    }
                }
            }
            Reveal { delay: 200,
                div { class: "bg-surface border border-border rounded-2xl p-6",
                    div { class: "text-sm font-bold", "Custody & Arbitraje" }
                    p { class: "text-xs text-muted", "Config PDA 7j7T... | ArbiterPool (vacío) | Disputas: 0" }
                }
            }
        }
    }
}

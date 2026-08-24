use crate::route::Route;
use crate::server::auth::guest::use_auth;
use crate::ui::{DashboardRole, Sidebar};
use dioxus::prelude::*;

/// Dashboard group layout — Next.js `(dashboard)` equivalent.
/// Single sidebar instance, role-aware. All /dashboard/* nests under this ONE layout
/// so ConfigPage does not create a new sidebar (fixes 5 sidebars funnel).
#[component]
pub fn DashboardLayout() -> Element {
    let auth = use_auth();
    let role = {
        let guard = auth.user.read();
        match guard.as_ref().map(|u| u.role.as_str()) {
            Some("freelancer") => DashboardRole::Freelancer,
            Some("admin") => DashboardRole::Admin,
            Some("arbiter") => DashboardRole::Arbiter,
            _ => DashboardRole::Client,
        }
    };
    let user_opt = auth.user.read().clone();
    rsx! {
        // Dashboard is app, not landing — noindex
        // Dioxus document meta (supported via dioxus::document)
        document::Meta { name: "robots", content: "noindex, nofollow" }
        div { class: "flex min-h-screen bg-bg text-fg",
            Sidebar { role: role }
            div { class: "flex-1 flex flex-col min-w-0",
                header { class: "sticky top-0 z-10 bg-surface/80 backdrop-blur border-b border-border px-6 py-3 flex items-center justify-between gap-4 animate-nav-in",
                    div { class: "flex items-center gap-3 text-sm flex-wrap",
                        if let Some(u) = user_opt.clone() {
                            span { class: "font-medium truncate max-w-[180px]", "{u.email}" }
                            if let Some(pk) = u.wallet_pubkey.clone() {
                                span { class: "font-mono text-xs bg-bg border border-border rounded-full px-3 py-1", "{&pk[..6.min(pk.len())]}...{&pk[pk.len().saturating_sub(4)..]}" }
                            } else {
                                Link { class: "text-xs bg-amber-500/10 border border-amber-500/30 rounded-full px-3 py-1 text-amber-600 hover:bg-amber-500/20 transition", to: Route::ConfigPage {}, "Crear billetera" }
                            }
                            if u.is_guest {
                                span { class: "text-xs bg-surface border border-border rounded-full px-2 py-1 text-muted", "Invitado · Solo lectura" }
                            }
                        } else {
                            span { class: "text-sm text-muted", "Cargando..." }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        Link { class: "text-sm text-primary underline", to: Route::LandingPage {}, "← Landing" }
                    }
                }
                div { class: "flex-1 p-8 page-enter",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum DashboardRole {
    Client,
    Freelancer,
    Arbiter,
    Admin,
}

#[component]
pub fn Sidebar(role: DashboardRole) -> Element {
    // Only routes that actually exist — no broken /create, /escrow/*, /applications etc.
    // Single sidebar instance is rendered by DashboardLayout for all /dashboard/* routes.
    let links = match role {
        DashboardRole::Client => vec![
            ("/dashboard/client", "Mis Jobs"),
            ("/dashboard/config", "Config · Wallet"),
        ],
        DashboardRole::Freelancer => vec![
            ("/dashboard/freelancer", "Jobs disponibles"),
            ("/dashboard/config", "Config · Wallet"),
        ],
        DashboardRole::Arbiter => vec![("/dashboard/config", "Config · Wallet")],
        DashboardRole::Admin => vec![
            ("/dashboard/admin", "Métricas"),
            ("/dashboard/config", "Config · Wallet"),
        ],
    };

    rsx! {
        aside { class: "w-64 bg-surface border-r border-border min-h-screen p-6 flex flex-col gap-6",
            div { class: "text-lg font-bold text-primary", "Trust Work Escrow" }
            nav { class: "flex flex-col gap-2",
                for (href, label) in links {
                    a { class: "px-3 py-2 rounded-xl text-sm text-fg hover:bg-surface-2 hover:text-primary transition-colors border border-transparent hover:border-border",
                        href: "{href}", "{label}"
                    }
                }
            }
            div { class: "mt-auto text-xs text-muted", {format!("Role: {:?} • dcdev #120808", role)} }
        }
    }
}

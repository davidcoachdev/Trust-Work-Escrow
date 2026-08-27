use dioxus::prelude::*;

use crate::server::auth::guest::has_wildcard;

#[derive(Clone, PartialEq, Debug)]
pub enum DashboardRole {
    Client,
    Freelancer,
    Arbiter,
    Admin,
}

/// Legacy fallback: single role to links
fn legacy_links(role: &DashboardRole) -> Vec<(&'static str, &'static str)> {
    match role {
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
    }
}

fn has(perms: &[String], required: &str) -> bool {
    has_wildcard(perms, required)
}

#[component]
pub fn Sidebar(
    #[props(default)] roles: Vec<String>,
    #[props(default)] permissions: Vec<String>,
    // Legacy single role compat: if roles/permissions empty, fallback to this
    #[props(default)] role: Option<DashboardRole>,
) -> Element {
    // Build dynamic menu from permissions, fallback to legacy single role
    let links: Vec<(String, String)> = if !permissions.is_empty() || !roles.is_empty() {
        let mut out = Vec::new();
        // Administración — requires admin:*
        if has(&permissions, "admin:users") || has(&permissions, "admin:*") {
            out.push(("/admin/users".to_string(), "Admin · Usuarios".to_string()));
            out.push(("/admin/permissions".to_string(), "Admin · Permisos".to_string()));
            out.push(("/admin/wallets".to_string(), "Admin · Wallets".to_string()));
            out.push(("/dashboard/admin".to_string(), "Admin · Métricas".to_string()));
        }
        // Soporte
        if has(&permissions, "support:view") || has(&permissions, "admin:*") {
            out.push(("/admin/support".to_string(), "Soporte · Tickets".to_string()));
        }
        // Jobs — any jobs view/create (todos pueden publicar) y apply (todos pueden postular)
        if has(&permissions, "jobs:view") || has(&permissions, "jobs:view:own") || has(&permissions, "jobs:create") || has(&permissions, "admin:*") {
            out.push(("/dashboard/client".to_string(), "Jobs · Mis Jobs".to_string()));
            out.push(("/jobs/published".to_string(), "Jobs · Publicados".to_string()));
            if has(&permissions, "jobs:create") || has(&permissions, "admin:*") {
                out.push(("/jobs/create".to_string(), "Jobs · Crear".to_string()));
            }
        }
        // Jobs disponibles — visible si puede postular (jobs:apply) o ver jobs; con permisos duales todos lo ven
        if has(&permissions, "jobs:apply") || has(&permissions, "jobs:view") || has(&permissions, "admin:*") {
            out.push(("/dashboard/freelancer".to_string(), "Jobs · Disponibles".to_string()));
        }
        // Disputas
        if has(&permissions, "disputes:view") || has(&permissions, "admin:*") {
            out.push(("/disputes/open".to_string(), "Disputas · Abiertas".to_string()));
            out.push(("/disputes/history".to_string(), "Disputas · Historial".to_string()));
        }
        // Arbitraje
        if has(&permissions, "arbitration:assigned") || has(&permissions, "admin:*") {
            out.push(("/arbitraje/asignadas".to_string(), "Arbitraje · Asignadas".to_string()));
        }
        // Config always
        out.push(("/dashboard/config".to_string(), "Config · Wallet".to_string()));
        // De-dup keep order
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.dedup_by(|a, b| a.0 == b.0);
        // If no perms matched, fallback to show at least Config + legacy freelancer view
        if out.len() == 1 {
            out.insert(0, ("/dashboard/freelancer".to_string(), "Jobs disponibles".to_string()));
        }
        out
    } else if let Some(r) = role.as_ref() {
        legacy_links(r).into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    } else {
        vec![
            ("/dashboard/client".to_string(), "Mis Jobs".to_string()),
            ("/dashboard/config".to_string(), "Config · Wallet".to_string()),
        ]
    };

    let role_label = if !roles.is_empty() {
        roles.join(",")
    } else if let Some(r) = role.as_ref() {
        format!("{:?}", r)
    } else {
        "guest".to_string()
    };

    rsx! {
        aside { class: "w-64 bg-surface border-r border-border min-h-screen p-6 flex flex-col gap-6",
            div { class: "text-lg font-bold text-primary", "Trust Work Escrow" }
            nav { class: "flex flex-col gap-2",
                for (href, label) in links.iter() {
                    a { class: "px-3 py-2 rounded-xl text-sm text-fg hover:bg-surface-2 hover:text-primary transition-colors border border-transparent hover:border-border",
                        href: "{href}", "{label}"
                    }
                }
            }
            div { class: "mt-auto text-xs text-muted", {format!("Roles: {} • dcdev #120808", role_label)} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn has_wildcard_admin() {
        let perms = vec!["admin:*".to_string()];
        assert!(has(&perms, "admin:users"));
        assert!(has(&perms, "admin:wallets"));
        assert!(!has(&perms, "jobs:view"));
    }
    #[test]
    fn has_exact() {
        let perms = vec!["jobs:view:own".to_string()];
        assert!(has(&perms, "jobs:view:own"));
        assert!(!has(&perms, "jobs:view"));
    }
}

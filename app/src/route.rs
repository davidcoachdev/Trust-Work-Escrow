use dioxus::prelude::*;

use crate::features::arbitration::{ArbitrationScreens, WebRtcPage};
use crate::features::auth::{LoginPage, SignupPage};
use crate::features::contact::ContactPage;
use crate::features::dashboard::{
    AdminDashboard, ClientDashboard, ConfigPage, FreelancerDashboard,
};
use crate::features::docs::DocsPage;
use crate::features::landing::LandingPage;
use crate::server::auth::guest::{use_auth, has_wildcard};
use crate::ui::{DashboardLayout, MarketingLayout};

/// App routes — Next.js App Router pattern via Dioxus nested layouts:
/// (marketing) group => MarketingLayout (Navbar+Footer) for "/", "/login", "/signup", "/contact"
/// (dashboard) group => DashboardLayout (single Sidebar) for all "/dashboard/*"
/// Fixes 5 sidebars funnel: ConfigPage no longer nests ClientLayout again.
#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    // ── (marketing) ──
    #[nest("")]
    #[layout(MarketingLayout)]
    #[route("/")]
    LandingPage {},
    #[route("/docs")]
    DocsPage {},
    #[route("/login")]
    LoginPage {},
    #[route("/signup")]
    SignupPage {},
    #[route("/contact")]
    ContactPage {},
    #[end_layout]
    #[end_nest]
    // ── (dashboard) — single sidebar, dynamic MenuConfig ──
    #[nest("/dashboard")]
    #[layout(DashboardLayout)]
    #[route("/client")]
    ClientDashboardGuard {},
    #[route("/freelancer")]
    FreelancerDashboardGuard {},
    #[route("/admin")]
    AdminDashboardGuard {},
    #[route("/config")]
    ConfigPage {},
    #[end_layout]
    #[end_nest]
    // ── admin console (7 subroutes) — guards admin:* etc ──
    #[nest("/admin")]
    #[layout(DashboardLayout)]
    #[route("/users")]
    AdminUsersGuard {},
    #[route("/permissions")]
    AdminPermissionsGuard {},
    #[route("/wallets")]
    AdminWalletsGuard {},
    #[route("/accounting")]
    AdminAccountingGuard {},
    #[route("/support")]
    SupportTicketsGuard {},
    #[route("/disputes")]
    AdminDisputesGuard {},
    #[route("/metrics")]
    AdminMetricsGuard {},
    #[end_layout]
    #[end_nest]
    // ── arbitration (outside groups, no layout wrapper) ──
    #[route("/arbitration/webrtc")]
    WebRtcPage {},
    #[route("/arbitration/screens")]
    ArbitrationScreens {},

    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

// Single source of truth allowlist for drift test (must stay subset of backend)
pub const FRONTEND_PERMS: &[&str] = &[
    "admin:*",
    "admin:users",
    "admin:permissions",
    "admin:wallets",
    "admin:accounting",
    "admin:support",
    "support:view",
    "jobs:view",
    "jobs:view:own",
    "jobs:create",
    "jobs:apply",
    "disputes:view",
    "arbitration:assigned",
    "config:wallet",
];

fn has(perms: &[String], required: &str) -> bool {
    has_wildcard(perms, required)
}

fn can_access(perms: &[String], required: &str) -> bool {
    has(perms, required)
}

#[component]
fn ForbiddenPage(required: String) -> Element {
    rsx! {
        div { class: "wrap py-12 text-center space-y-4",
            h1 { class: "text-3xl font-bold text-red-600", "403 — No autorizado" }
            p { class: "text-muted", "Requiere permiso: {required}" }
            Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium", to: Route::LandingPage {}, "Volver al inicio" }
        }
    }
}

#[component]
fn ClientDashboardGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if perms.is_empty() {
        // guest fallback: allow legacy client view via role check
        let roles = auth.user.read().as_ref().map(|u| u.normalized_roles()).unwrap_or_default();
        if roles.contains(&"client".to_string()) || roles.contains(&"guest".to_string()) {
            return rsx! { ClientDashboard {} };
        }
    }
    if can_access(&perms, "jobs:view") || can_access(&perms, "jobs:view:own") || can_access(&perms, "jobs:create") || can_access(&perms, "admin:*") {
        rsx! { ClientDashboard {} }
    } else {
        rsx! { ForbiddenPage { required: "jobs:view".to_string() } }
    }
}

#[component]
fn FreelancerDashboardGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if perms.is_empty() {
        let roles = auth.user.read().as_ref().map(|u| u.normalized_roles()).unwrap_or_default();
        if roles.contains(&"freelancer".to_string()) {
            return rsx! { FreelancerDashboard {} };
        }
        // allow if no perms yet (fallback)
        return rsx! { FreelancerDashboard {} };
    }
    if can_access(&perms, "jobs:view") || can_access(&perms, "jobs:apply") || can_access(&perms, "admin:*") {
        rsx! { FreelancerDashboard {} }
    } else {
        rsx! { ForbiddenPage { required: "jobs:view".to_string() } }
    }
}

#[component]
fn AdminDashboardGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "admin:users") {
        rsx! { AdminDashboard {} }
    } else {
        rsx! { ForbiddenPage { required: "admin:*".to_string() } }
    }
}

#[component]
fn AdminUsersGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "admin:users") {
        rsx! { div { class: "space-y-4", h1 { class: "text-2xl font-bold", "Admin · Usuarios" } p { class: "text-muted", "Gestión de usuarios y roles (Wave0 placeholder)" } } }
    } else {
        rsx! { ForbiddenPage { required: "admin:users".to_string() } }
    }
}
#[component]
fn AdminPermissionsGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "admin:permissions") {
        rsx! { div { h1 { class: "text-2xl font-bold", "Admin · Permisos" } } }
    } else {
        rsx! { ForbiddenPage { required: "admin:permissions".to_string() } }
    }
}
#[component]
fn AdminWalletsGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "admin:wallets") {
        rsx! { div { h1 { class: "text-2xl font-bold", "Admin · Wallets" } } }
    } else {
        rsx! { ForbiddenPage { required: "admin:wallets".to_string() } }
    }
}
#[component]
fn AdminAccountingGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "admin:accounting") || can_access(&perms, "accountant:view") {
        rsx! { div { h1 { class: "text-2xl font-bold", "Admin · Contabilidad" } } }
    } else {
        rsx! { ForbiddenPage { required: "admin:accounting".to_string() } }
    }
}
#[component]
fn SupportTicketsGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "support:view") {
        rsx! { div { h1 { class: "text-2xl font-bold", "Soporte · Tickets" } } }
    } else {
        rsx! { ForbiddenPage { required: "support:view".to_string() } }
    }
}
#[component]
fn AdminDisputesGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") || can_access(&perms, "disputes:view") {
        rsx! { div { h1 { class: "text-2xl font-bold", "Admin · Disputas" } } }
    } else {
        rsx! { ForbiddenPage { required: "disputes:view".to_string() } }
    }
}
#[component]
fn AdminMetricsGuard() -> Element {
    let auth = use_auth();
    let perms = auth.user.read().as_ref().map(|u| u.permissions.clone()).unwrap_or_default();
    if can_access(&perms, "admin:*") {
        rsx! { div { h1 { class: "text-2xl font-bold", "Admin · Métricas" } } }
    } else {
        rsx! { ForbiddenPage { required: "admin:*".to_string() } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drift_frontend_subset_backend() {
        // Backend allowlist must superset frontend perms (drift check)
        let backend = [
            "admin:*",
            "admin:users",
            "admin:permissions",
            "admin:wallets",
            "admin:accounting",
            "admin:support",
            "support:view",
            "support:manage",
            "jobs:view",
            "jobs:view:own",
            "jobs:create",
            "jobs:apply",
            "jobs:manage",
            "jobs:delete:own",
            "disputes:view",
            "arbitration:assigned",
            "config:wallet",
            "accountant:view",
        ];
        for p in FRONTEND_PERMS {
            assert!(backend.contains(p), "frontend perm {} not in backend allowlist", p);
        }
    }
    #[test]
    fn guard_403() {
        let perms = vec!["jobs:view".to_string()];
        assert!(can_access(&perms, "jobs:view"));
        assert!(!can_access(&perms, "admin:users"));
        let admin = vec!["admin:*".to_string()];
        assert!(can_access(&admin, "admin:users"));
    }
}

#[component]
fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div { class: "wrap py-12 text-center space-y-4",
            h1 { class: "text-3xl font-bold", "404 — Página no encontrada" }
            p { class: "text-muted", "Ruta: /{route.join(\"/\")}" }
            Link { class: "inline-flex bg-primary text-on-primary rounded-xl px-5 py-2.5 font-medium", to: Route::LandingPage {}, "Volver al inicio" }
        }
    }
}

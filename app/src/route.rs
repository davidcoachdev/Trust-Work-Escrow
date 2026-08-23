use dioxus::prelude::*;

use crate::features::landing::LandingPage;
use crate::features::auth::{LoginPage, SignupPage};
use crate::features::contact::ContactPage;
use crate::features::dashboard::{AdminDashboard, ClientDashboard, ConfigPage, FreelancerDashboard};
use crate::features::arbitration::{ArbitrationScreens, WebRtcPage};
use crate::ui::{DashboardLayout, MarketingLayout};

/// App routes — Next.js App Router pattern via Dioxus nested layouts:
/// (marketing) group => MarketingLayout (Navbar+Footer) for "/", "/login", "/signup", "/contact"
/// (dashboard) group => DashboardLayout (single Sidebar) for all "/dashboard/*"
/// Fixes 5 sidebars funnel: ConfigPage no longer nests ClientLayout again.
#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    // ── (marketing) ──
    #[nest("/")]
        #[layout(MarketingLayout)]
            #[route("/")]
            LandingPage {},
            #[route("/login")]
            LoginPage {},
            #[route("/signup")]
            SignupPage {},
            #[route("/contact")]
            ContactPage {},
        #[end_layout]
    #[end_nest]

    // ── (dashboard) — single sidebar, role-aware ──
    #[nest("/dashboard")]
        #[layout(DashboardLayout)]
            #[route("/client")]
            ClientDashboard {},
            #[route("/freelancer")]
            FreelancerDashboard {},
            #[route("/admin")]
            AdminDashboard {},
            #[route("/config")]
            ConfigPage {},
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

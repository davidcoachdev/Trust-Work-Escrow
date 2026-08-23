use dioxus::prelude::*;

use crate::features::landing::LandingPage;
use crate::features::auth::{LoginPage, SignupPage};
use crate::features::contact::ContactPage;
use crate::ui::Navbar;

/// Top-level pages of the landing site, powered by dioxus-router.
/// `Navbar` acts as a persistent layout (it renders the routed page via `Outlet`).
#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[nest("/")]
        #[layout(Navbar)]
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

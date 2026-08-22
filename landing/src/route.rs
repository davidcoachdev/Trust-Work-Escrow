use dioxus::prelude::*;

use crate::features::landing::LandingPage;
use crate::features::auth::{LoginPage, SignupPage};
use crate::features::contact::ContactPage;
use crate::ui::Navbar;

/// Top-level pages of the landing site, powered by dioxus-router.
/// `Navbar` acts as a persistent layout (it renders the routed page via `Outlet`).
#[derive(Routable, Clone)]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    LandingPage {},
    #[route("/login")]
    LoginPage {},
    #[route("/signup")]
    SignupPage {},
    #[route("/contact")]
    ContactPage {},
}

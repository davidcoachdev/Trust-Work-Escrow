use dioxus::prelude::*;

use crate::features::landing::LandingPage;
use crate::features::auth::{LoginPage, SignupPage};
use crate::features::contact::ContactPage;
use crate::features::dashboard::{AdminDashboard, ClientDashboard, FreelancerDashboard};
use crate::features::arbitration::WebRtcPage;
use crate::ui::Navbar;
use crate::features::dashboard::{AdminLayoutComponent as AdminLayout, ClientLayoutComponent as ClientLayout, FreelancerLayoutComponent as FreelancerLayout};

/// Top-level pages — dcdev theme, i18n ES/EN, role-based layouts
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

    #[layout(ClientLayout)]
    #[route("/dashboard/client")]
    ClientDashboard {},

    #[layout(FreelancerLayout)]
    #[route("/dashboard/freelancer")]
    FreelancerDashboard {},

    #[layout(AdminLayout)]
    #[route("/dashboard/admin")]
    AdminDashboard {},

    #[route("/arbitration/webrtc")]
    WebRtcPage {},
}

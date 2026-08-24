use crate::features::landing::Footer;
use crate::route::Route;
use crate::ui::Navbar;
use dioxus::prelude::*;

/// Marketing group layout — Next.js `(marketing)` equivalent.
/// Wraps "/", "/login", "/signup", "/contact" with Navbar + Footer.
/// No sidebar, no dashboard chrome.
#[component]
pub fn MarketingLayout() -> Element {
    rsx! {
        Navbar {}
        main { id: "main-content", class: "min-h-[calc(100vh-64px)] page-enter",
            Outlet::<Route> {}
        }
        Footer {}
    }
}

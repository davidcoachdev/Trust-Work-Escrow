use dioxus::prelude::*;

/// Wraps content so it fades/slides in when scrolled into view.
/// The actual reveal is driven by an `IntersectionObserver` set up in
/// `LandingPage` (see the `.reveal` / `.reveal.is-visible` rules in tailwind.css).
#[component]
pub fn Reveal(children: Element) -> Element {
    rsx! {
        div { class: "reveal", {children} }
    }
}

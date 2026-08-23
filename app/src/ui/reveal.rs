use dioxus::prelude::*;

/// Animation variant for scroll reveal.
#[derive(Clone, PartialEq)]
pub enum RevealVariant {
    FadeUp,
    FadeIn,
    SlideLeft,
    SlideRight,
    Scale,
}

impl Default for RevealVariant {
    fn default() -> Self {
        Self::FadeUp
    }
}

impl RevealVariant {
    fn css_class(&self) -> &'static str {
        match self {
            Self::FadeUp => "",
            Self::FadeIn => "reveal--fade",
            Self::SlideLeft => "reveal--left",
            Self::SlideRight => "reveal--right",
            Self::Scale => "reveal--scale",
        }
    }
}

/// Wraps content so it fades/slides in when scrolled into view.
/// The actual reveal is driven by an `IntersectionObserver` set up in
/// `LandingPage` (see the `.reveal` / `.reveal.is-visible` rules in tailwind.css).
#[component]
pub fn Reveal(
    children: Element,
    #[props(default)] delay: u64,
    #[props(default)] variant: RevealVariant,
) -> Element {
    let variant_class = variant.css_class();
    let class = if variant_class.is_empty() {
        "reveal".to_string()
    } else {
        format!("reveal {}", variant_class)
    };
    let style = if delay > 0 {
        format!("transition-delay: {}ms", delay)
    } else {
        String::new()
    };
    rsx! {
        if style.is_empty() {
            div { class: "{class}", {children} }
        } else {
            div { class: "{class}", style: "{style}", {children} }
        }
    }
}

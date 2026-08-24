mod cta;
mod faq;
mod features;
mod footer;
mod hero;
mod how;
mod schema;
mod stats;
mod who;

pub use cta::Cta;
pub use faq::Faq;
pub use features::Features;
pub use footer::Footer;
pub use hero::Hero;
pub use how::HowItWorks;
pub use stats::Stats;
pub use who::ForWhom;

use crate::ui::{Reveal, RevealVariant};
use dioxus::prelude::*;

/// Sets up a cross-browser scroll reveal: every `.reveal` element gets an
/// `IntersectionObserver` that adds `is-visible` (triggering the CSS transition)
/// once it scrolls into view. Falls back to showing everything immediately if
/// IntersectionObserver is unavailable. Runs whenever the landing mounts.
const REVEAL_SCRIPT: &str = r#"
(function () {
  var els = document.querySelectorAll('.reveal');
  if (!('IntersectionObserver' in window)) {
    els.forEach(function (e) { e.classList.add('is-visible'); });
    return;
  }
  var io = new IntersectionObserver(function (entries) {
    entries.forEach(function (entry) {
      if (entry.isIntersecting) {
        entry.target.classList.add('is-visible');
        io.unobserve(entry.target);
      }
    });
  }, { threshold: 0.1 });
  requestAnimationFrame(function () {
    document.querySelectorAll('.reveal').forEach(function (e) { io.observe(e); });
  });
})();
"#;

#[component]
pub fn LandingPage() -> Element {
    #[cfg(target_arch = "wasm32")]
    use_effect(|| {
        spawn(async {
            let _ = document::eval(REVEAL_SCRIPT).await;
        });
    });

    rsx! {
        // SEO: landing is the only indexable page — dashboard uses noindex via DashboardLayout
        // Hero has its own internal stagger, so outer is just FadeIn
        Reveal { variant: RevealVariant::FadeIn, Hero {} }
        Reveal { HowItWorks {} }
        Reveal { Features {} }
        Reveal { Stats {} }
        Reveal { ForWhom {} }
        Reveal { Faq {} }
        Reveal { variant: RevealVariant::Scale, Cta {} }
        // Footer is rendered by MarketingLayout, not here — avoids duplication inside (marketing) group
    }
}

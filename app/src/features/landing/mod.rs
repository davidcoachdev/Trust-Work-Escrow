mod hero;
mod features;
mod how;
mod stats;
mod who;
mod cta;
mod footer;

pub use hero::Hero;
pub use features::Features;
pub use how::HowItWorks;
pub use stats::Stats;
pub use who::ForWhom;
pub use cta::Cta;
pub use footer::Footer;

use dioxus::prelude::*;
use crate::ui::Reveal;

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
    use_effect(|| {
        spawn(async {
            let _ = document::eval(REVEAL_SCRIPT).await;
        });
    });

    rsx! {
        Reveal { Hero {} }
        Reveal { HowItWorks {} }
        Reveal { Features {} }
        Reveal { Stats {} }
        Reveal { ForWhom {} }
        Reveal { Cta {} }
        Reveal { Footer {} }
    }
}

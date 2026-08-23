use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::route::Route;
use crate::ui::{Reveal, RevealVariant};

#[component]
pub fn Footer() -> Element {
    let l = *use_i18n().lang.read();
    rsx! {
        footer { class: "border-t border-border py-8 text-muted",
            Reveal { variant: RevealVariant::FadeIn,
                div { class: "wrap flex items-center justify-between gap-4 flex-wrap",
                    span { {tr(l, "footer.copyright")} }
                    nav { class: "flex gap-5",
                        Link { class: "hover:text-fg transition-colors", to: Route::DocsPage {}, {tr(l, "footer.links.docs")} }
                        a { class: "hover:text-fg transition-colors", href: "https://github.com/davidcoachdev/Trust-Work-Escrow", target: "_blank", rel: "noopener noreferrer", {tr(l, "footer.links.github")} }
                        Link { class: "hover:text-fg transition-colors", to: Route::LandingPage {}, {tr(l, "footer.links.status")} }
                    }
                    span { class: "opacity-80", {tr(l, "footer.onSolana")} }
                }
            }
        }
    }
}

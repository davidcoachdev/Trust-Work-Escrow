use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn Footer() -> Element {
    let l = *use_i18n().lang.read();
    rsx! {
        footer { class: "border-t border-border py-8 text-muted",
            div { class: "wrap flex items-center justify-between gap-4 flex-wrap",
                span { {tr(l, "footer.copyright")} }
                nav { class: "flex gap-5",
                    a { class: "hover:text-fg transition-colors", href: "#", {tr(l, "footer.links.docs")} }
                    a { class: "hover:text-fg transition-colors", href: "#", {tr(l, "footer.links.github")} }
                    a { class: "hover:text-fg transition-colors", href: "#", {tr(l, "footer.links.status")} }
                }
                span { class: "opacity-80", {tr(l, "footer.onSolana")} }
            }
        }
    }
}

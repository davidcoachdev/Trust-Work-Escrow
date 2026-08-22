use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn Features() -> Element {
    let l = *use_i18n().lang.read();
    let items = [
        ("features.secure.title", "features.secure.body"),
        ("features.proof.title", "features.proof.body"),
        ("features.fees.title", "features.fees.body"),
        ("features.transparent.title", "features.transparent.body"),
        ("features.instant.title", "features.instant.body"),
        ("features.noncustodial.title", "features.noncustodial.body"),
    ];
    rsx! {
        section { class: "py-24",
            div { class: "wrap",
                h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "features.title")} }
                div { class: "grid gap-6 sm:grid-cols-2 lg:grid-cols-3",
                    for (title, body) in items {
                        div { class: "bg-surface border border-border rounded-2xl p-6",
                            h3 { class: "text-xl font-medium mb-2", {tr(l, title)} }
                            p { class: "text-muted", {tr(l, body)} }
                        }
                    }
                }
            }
        }
    }
}

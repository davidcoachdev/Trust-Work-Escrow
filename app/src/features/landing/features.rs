use crate::i18n::{tr, use_i18n};
use crate::ui::Reveal;
use dioxus::prelude::*;

#[component]
pub fn Features() -> Element {
    let l = *use_i18n().lang.read();
    let items = [
        ("features.secure.title", "features.secure.body", 0u64),
        ("features.proof.title", "features.proof.body", 80u64),
        ("features.fees.title", "features.fees.body", 160u64),
        (
            "features.transparent.title",
            "features.transparent.body",
            240u64,
        ),
        ("features.instant.title", "features.instant.body", 320u64),
        (
            "features.noncustodial.title",
            "features.noncustodial.body",
            400u64,
        ),
    ];
    rsx! {
        section { id: "features", class: "py-24",
            div { class: "wrap",
                Reveal {
                    h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "features.title")} }
                }
                div { class: "grid gap-6 sm:grid-cols-2 lg:grid-cols-3",
                    for (title, body, delay) in items {
                        Reveal { delay: delay,
                            div { class: "bg-surface border border-border rounded-2xl p-6 transition hover:-translate-y-1 hover:shadow-lg h-full",
                                h3 { class: "text-xl font-medium mb-2", {tr(l, title)} }
                                p { class: "text-muted", {tr(l, body)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

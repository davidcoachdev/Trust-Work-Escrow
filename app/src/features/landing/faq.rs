use crate::i18n::{tr, use_i18n};
use dioxus::prelude::*;

#[component]
pub fn Faq() -> Element {
    let lang = *use_i18n().lang.read();
    let items = [
        (tr(lang, "faq.q1"), tr(lang, "faq.a1")),
        (tr(lang, "faq.q2"), tr(lang, "faq.a2")),
        (tr(lang, "faq.q3"), tr(lang, "faq.a3")),
        (tr(lang, "faq.q4"), tr(lang, "faq.a4")),
        (tr(lang, "faq.q5"), tr(lang, "faq.a5")),
        (tr(lang, "faq.q6"), tr(lang, "faq.a6")),
    ];

    rsx! {
        section { id: "faq", class: "py-24 bg-surface",
            div { class: "wrap",
                h2 { class: "text-3xl font-bold tracking-tight text-center mb-4", "{tr(lang, \"faq.title\")}" }
                p { class: "text-muted text-center max-w-[56ch] mx-auto mb-10", "{tr(lang, \"faq.subtitle\")}" }
                div { class: "grid gap-4 max-w-3xl mx-auto",
                    for (q, a) in items {
                        details { class: "group bg-bg border border-border rounded-2xl p-6 open:ring-1 open:ring-border",
                            summary { class: "font-medium cursor-pointer list-none flex justify-between items-center gap-4",
                                span { {q} }
                                span { class: "text-muted group-open:rotate-180 transition-transform", "▾" }
                            }
                            p { class: "text-muted mt-3 text-[15px] leading-relaxed", {a} }
                        }
                    }
                }
            }
        }
    }
}

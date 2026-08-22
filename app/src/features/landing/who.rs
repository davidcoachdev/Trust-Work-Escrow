use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn ForWhom() -> Element {
    let l = *use_i18n().lang.read();
    rsx! {
        section { class: "py-24 bg-surface",
            div { class: "wrap",
                h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "who.title")} }
                div { class: "grid gap-6 md:grid-cols-2",
                    div { class: "bg-bg border border-border rounded-2xl p-8",
                        h3 { class: "text-2xl font-semibold mb-3 text-primary", {tr(l, "who.freelancers.title")} }
                        p { class: "text-muted", {tr(l, "who.freelancers.body")} }
                    }
                    div { class: "bg-bg border border-border rounded-2xl p-8",
                        h3 { class: "text-2xl font-semibold mb-3 text-primary", {tr(l, "who.employers.title")} }
                        p { class: "text-muted", {tr(l, "who.employers.body")} }
                    }
                }
            }
        }
    }
}

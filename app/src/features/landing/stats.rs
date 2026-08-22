use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn Stats() -> Element {
    let l = *use_i18n().lang.read();
    let items = [
        ("$2.4M", "stats.tvl"),
        ("18k+", "stats.tx"),
        ("3.2k", "stats.users"),
        ("Solana", "stats.chain"),
    ];
    rsx! {
        section { class: "py-24",
            div { class: "wrap",
                h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "stats.title")} }
                div { class: "grid gap-6 text-center grid-cols-2 lg:grid-cols-4",
                    for (value, label) in items {
                        div { class: "flex flex-col gap-1.5",
                            span { class: "text-3xl md:text-4xl font-bold tracking-tight gradient bg-clip-text text-transparent", {value} }
                            span { class: "text-sm text-muted", {tr(l, label)} }
                        }
                    }
                }
            }
        }
    }
}

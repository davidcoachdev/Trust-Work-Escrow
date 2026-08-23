use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::ui::Reveal;

#[component]
pub fn Stats() -> Element {
    let l = *use_i18n().lang.read();
    let items = [
        ("$2.4M", "stats.tvl", 0u64),
        ("18k+", "stats.tx", 100u64),
        ("3.2k", "stats.users", 200u64),
        ("Solana", "stats.chain", 300u64),
    ];
    rsx! {
        section { class: "py-24",
            div { class: "wrap",
                Reveal {
                    h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "stats.title")} }
                }
                div { class: "grid gap-6 text-center grid-cols-2 lg:grid-cols-4",
                    for (value, label, delay) in items {
                        Reveal { delay: delay,
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
}

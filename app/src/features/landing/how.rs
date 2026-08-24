use crate::i18n::{tr, use_i18n};
use crate::ui::{Reveal, RevealVariant};
use dioxus::prelude::*;

#[component]
pub fn HowItWorks() -> Element {
    let l = *use_i18n().lang.read();
    let steps = [
        ("how.step1.title", "how.step1.body", "01", 0u64),
        ("how.step2.title", "how.step2.body", "02", 150u64),
        ("how.step3.title", "how.step3.body", "03", 300u64),
    ];
    rsx! {
        section { id: "how", class: "py-24 bg-surface scroll-mt-20",
            div { class: "wrap",
                Reveal {
                    h2 { class: "text-3xl font-bold tracking-tight text-center mb-12", {tr(l, "how.title")} }
                }
                div { class: "grid gap-6 sm:grid-cols-2 lg:grid-cols-3",
                    for (title, body, n, delay) in steps {
                        Reveal { delay: delay,
                            div { class: "relative bg-bg border border-border rounded-2xl p-7 transition hover:-translate-y-1 hover:shadow-lg",
                                span { class: "inline-block text-sm font-bold tracking-wide text-on-primary bg-primary rounded-lg px-2.5 py-1 mb-4", {n} }
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

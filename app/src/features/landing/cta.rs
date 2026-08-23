use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::ui::{Reveal, RevealVariant};

#[component]
pub fn Cta() -> Element {
    let l = *use_i18n().lang.read();
    rsx! {
        section { class: "py-24",
            div { class: "wrap",
                Reveal { variant: RevealVariant::Scale,
                    div { class: "gradient rounded-3xl p-16 text-center text-on-primary",
                        h2 { class: "text-3xl font-bold", {tr(l, "cta.title")} }
                        p { class: "mt-3 opacity-90", {tr(l, "cta.body")} }
                        a { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium border border-on-primary text-on-primary bg-transparent mt-6 transition hover:-translate-y-0.5 active:scale-[0.98]", href: "#", {tr(l, "cta.button")} }
                    }
                }
            }
        }
    }
}

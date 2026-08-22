use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn Hero() -> Element {
    let l = *use_i18n().lang.read();
    rsx! {
        section { class: "relative text-center py-32 overflow-hidden",
            div { class: "absolute inset-0 gradient opacity-[0.14] blur-3xl pointer-events-none" }
            div { class: "relative wrap",
                span { class: "inline-block text-sm font-medium text-primary border border-border bg-surface rounded-full px-3.5 py-1.5 mb-6", {tr(l, "hero.badge")} }
                h1 { class: "text-[clamp(40px,6vw,64px)] font-bold tracking-tight leading-none max-w-[16ch] mx-auto", {tr(l, "hero.title")} }
                p { class: "mt-5 text-xl text-muted max-w-[56ch] mx-auto", {tr(l, "hero.subtitle")} }
                div { class: "mt-8 flex gap-4 justify-center flex-wrap",
                    a { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5", href: "#cta", {tr(l, "hero.cta")} }
                    a { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium border border-primary text-primary bg-transparent transition hover:-translate-y-0.5", href: "#how", {tr(l, "hero.ctaSecondary")} }
                }
                p { class: "mt-6 text-sm text-muted max-w-[48ch] mx-auto", {tr(l, "hero.trust")} }
            }
        }
    }
}

use crate::i18n::{use_i18n, Lang};
use dioxus::prelude::*;

/// Language switcher — pill button (36x36) showing the current language code
/// (ES / EN). Opens a themed dropdown (no native `<select>`, so it follows the
/// active skin instead of the browser's default popup).
#[component]
pub fn LanguageSwitcher() -> Element {
    let mut lang = use_i18n().lang;
    let mut open = use_signal(|| false);
    let l = *lang.read();

    rsx! {
        div { class: "relative",
            button {
                class: "relative z-50 inline-flex items-center justify-center w-9 h-9 rounded-full border border-border bg-surface text-fg text-[13px] font-semibold cursor-pointer select-none transition-transform duration-150 hover:scale-105 active:scale-95",
                r#type: "button",
                title: "Idioma / Language",
                aria_label: "Idioma",
                onclick: move |_| { let v = *open.read(); open.set(!v); },
                span { {l.label()} }
            }
            if *open.read() {
                div { class: "fixed inset-0 z-40", style: "animation: fade-in 0.15s ease-out;", onclick: move |_| open.set(false) }
                div { class: "absolute right-0 mt-2 z-50 min-w-[120px] rounded-xl border border-border bg-surface text-fg shadow-lg py-1", style: "animation: dropdown-in 0.16s cubic-bezier(0.16,1,0.3,1);",
                    for ln in Lang::all() {
                        button {
                            class: "flex w-full items-center justify-between gap-2 px-3 py-2 text-sm hover:bg-bg/60 cursor-pointer text-left transition-colors",
                            r#type: "button",
                            onclick: move |_| { lang.set(*ln); open.set(false); },
                            span { class: "font-medium" } {ln.label()}
                            if l == *ln {
                                span { class: "text-primary" } "✓"
                            }
                        }
                    }
                }
            }
        }
    }
}

use crate::theme::{use_theme, Theme};
use dioxus::prelude::*;

/// Theme (skin) switcher — pill button (36x36) showing a swatch of the active
/// theme's primary colors. Opens a themed dropdown listing every skin with its
/// own color swatch (no native `<select>`, so it follows the active skin).
#[component]
pub fn ThemeSwitcher() -> Element {
    let mut theme = use_theme().theme;
    let mut open = use_signal(|| false);
    let t = *theme.read();

    rsx! {
        div { class: "relative",
            button {
                class: "relative z-50 inline-flex items-center justify-center w-9 h-9 rounded-full border border-border bg-surface cursor-pointer select-none transition-transform duration-150 hover:scale-105 active:scale-95",
                r#type: "button",
                title: "Tema / Theme",
                aria_label: "Tema",
                onclick: move |_| { let v = *open.read(); open.set(!v); },
                // swatch of the currently active theme
                span { class: "relative block w-4 h-4",
                    span { class: "absolute left-0 top-0 w-4 h-4 rounded-[5px] bg-primary" }
                    span { class: "absolute right-0 bottom-0 w-2 h-2 rounded-[4px] bg-primary-2" }
                }
            }
            if *open.read() {
                div { class: "fixed inset-0 z-40", style: "animation: fade-in 0.15s ease-out;", onclick: move |_| open.set(false) }
                div { class: "absolute right-0 mt-2 z-50 min-w-[160px] rounded-xl border border-border bg-surface text-fg shadow-lg py-1", style: "animation: dropdown-in 0.16s cubic-bezier(0.16,1,0.3,1);",
                    for th in Theme::all() {
                        button {
                            class: "flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-bg/60 cursor-pointer text-left transition-colors",
                            r#type: "button",
                            onclick: move |_| { theme.set(*th); open.set(false); },
                            span { class: "relative block w-4 h-4 shrink-0",
                                span { class: "absolute left-0 top-0 w-4 h-4 rounded-[5px]", style: "background-color: {th.swatch_primary()}" }
                                span { class: "absolute right-0 bottom-0 w-2 h-2 rounded-[4px]", style: "background-color: {th.swatch_secondary()}" }
                            }
                            span { class: "font-medium" } {th.label()}
                            if t == *th {
                                span { class: "ml-auto text-primary" } "✓"
                            }
                        }
                    }
                }
            }
        }
    }
}

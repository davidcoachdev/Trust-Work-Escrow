use dioxus::prelude::*;

#[component]
pub fn MilestonesPage() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-primary", "Milestones — JCR9... 0.115 SOL" }
            p { class: "text-sm text-muted", "Crea hitos para liberar parcial. Ej: 0.05 SOL por entrega." }
            div { class: "bg-surface border border-border rounded-2xl p-6 space-y-3",
                div { class: "grid grid-cols-3 gap-3 text-xs",
                    div { class: "bg-bg border border-border rounded-xl p-3",
                        div { class: "text-muted", "Hito 0" }
                        div { class: "font-bold", "0.05 SOL" }
                        div { class: "text-xs text-muted", "Pending" }
                    }
                    div { class: "bg-bg border border-border rounded-xl p-3",
                        div { class: "text-muted", "Hito 1" }
                        div { class: "font-bold", "0.05 SOL" }
                        div { class: "text-xs text-muted", "Pending" }
                    }
                    div { class: "bg-bg border border-border rounded-xl p-3",
                        div { class: "text-muted", "Total" }
                        div { class: "font-bold text-primary", "0.10 SOL" }
                        div { class: "text-xs text-muted", "+ fee 0.0025" }
                    }
                }
                button { class: "bg-primary text-on-primary rounded-xl px-4 py-2 text-sm", "Crear Hito (devnet)" }
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn ArbitrationScreens() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-primary", "Arbitraje — Sala P2P" }
            p { class: "text-sm text-muted", "Who/How + sala con chat, llamada y envío archivo vía DataChannel. Todo P2P sin SFU." }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                div { class: "bg-surface border border-border rounded-2xl p-6",
                    h2 { class: "font-bold", "Who" }
                    p { class: "text-sm text-muted", "Para quién es el arbitraje" }
                }
                div { class: "bg-surface border border-border rounded-2xl p-6",
                    h2 { class: "font-bold", "How" }
                    p { class: "text-sm text-muted", "Cómo funciona el flujo de disputa" }
                }
            }
            div { class: "bg-surface border border-border rounded-2xl p-6",
                h2 { class: "font-bold", "Sala de Arbitraje" }
                p { class: "text-sm text-muted", "Chat + llamada + archivos — todo por WebRTC DataChannel" }
            }
        }
    }
}

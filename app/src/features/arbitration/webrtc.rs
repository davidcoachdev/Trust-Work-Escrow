use dioxus::prelude::*;

#[component]
pub fn WebRtcPage() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-primary", "Arbitraje P2P — WebRTC" }
            p { class: "text-sm text-muted", "Llamada directa browser→browser sin SFU, DataChannel para chat/archivos, signaling vía ws 2s." }
            div { class: "grid grid-cols-2 gap-4",
                div { class: "bg-surface border border-border rounded-2xl p-6 aspect-video flex items-center justify-center text-muted text-sm", "Video local (P2P)" }
                div { class: "bg-surface border border-border rounded-2xl p-6 aspect-video flex items-center justify-center text-muted text-sm", "Video remoto (P2P)" }
            }
            div { class: "bg-surface border border-border rounded-2xl p-4 flex gap-2",
                button { class: "bg-primary text-on-primary rounded-xl px-4 py-2 text-sm", "Iniciar llamada" }
                button { class: "bg-surface border border-border rounded-xl px-4 py-2 text-sm", "Enviar archivo vía DataChannel" }
            }
        }
    }
}

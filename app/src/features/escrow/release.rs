use dioxus::prelude::*;

#[component]
pub fn ReleasePage() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-primary", "Release — JCR9..." }
            p { class: "text-sm text-muted", "Libera fondos congelados 0.115 SOL. Solo client 3whY... puede aprobar." }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-3",
                div { class: "bg-surface border border-border rounded-2xl p-4 space-y-2",
                    div { class: "text-sm font-bold", "Approve Work" }
                    p { class: "text-xs text-muted", "Envía 0.1 SOL a freelancer QWgp..." }
                    button { class: "w-full bg-primary text-on-primary rounded-xl px-3 py-2 text-sm", "Aprobar y liberar" }
                }
                div { class: "bg-surface border border-border rounded-2xl p-4 space-y-2",
                    div { class: "text-sm font-bold", "Cancel Job" }
                    p { class: "text-xs text-muted", "Devuelve 0.1 SOL a client 3whY..." }
                    button { class: "w-full bg-surface border border-border rounded-xl px-3 py-2 text-sm", "Cancelar" }
                }
                div { class: "bg-surface border border-border rounded-2xl p-4 space-y-2",
                    div { class: "text-sm font-bold", "Resolve Dispute" }
                    p { class: "text-xs text-muted", "Arbiter reparte 50/50" }
                    button { class: "w-full bg-surface border border-border rounded-xl px-3 py-2 text-sm", "Resolver" }
                }
            }
            div { class: "bg-bg border border-border rounded-xl p-3 text-xs font-mono break-all",
                "PDA: JCR9fRx9eMqr27jk2KvXSVFsewq7JxaAXHZg54YjjLTw — devnet"
            }
        }
    }
}

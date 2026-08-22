use dioxus::prelude::*;

#[component]
pub fn DisputesPage() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-primary", "Disputas — JCR9..." }
            p { class: "text-sm text-muted", "Raise dispute y submit evidence con hash on-chain (64 chars) + Cloudinary." }
            div { class: "bg-surface border border-border rounded-2xl p-6 space-y-3",
                div { class: "grid grid-cols-2 gap-3 text-xs",
                    div { class: "bg-bg border border-border rounded-xl p-3",
                        div { class: "text-muted", "Dispute PDA" }
                        div { class: "font-mono text-xs break-all", " seeds [b\"dispute\", JCR9...]" }
                        div { class: "text-xs text-muted", "Status: Open" }
                    }
                    div { class: "bg-bg border border-border rounded-xl p-3",
                        div { class: "text-muted", "Evidence" }
                        div { class: "font-mono text-xs", "hash 64 chars" }
                        div { class: "text-xs text-muted", "Cloudinary raw" }
                    }
                }
                button { class: "bg-primary text-on-primary rounded-xl px-4 py-2 text-sm", "Raise Dispute (devnet)" }
            }
        }
    }
}

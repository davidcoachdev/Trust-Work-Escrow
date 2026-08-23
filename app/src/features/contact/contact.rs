use dioxus::prelude::*;
use std::string::String;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn ContactPage() -> Element {
    let l = *use_i18n().lang.read();
    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut message = use_signal(|| String::new());

    rsx! {
        section { class: "py-24 wrap",
            div { class: "bg-surface border border-border rounded-2xl p-8 max-w-md mx-auto",
                h1 { class: "text-3xl font-bold tracking-tight text-center mb-2", {tr(l, "contact.title")} }
                form { class: "grid gap-4 mt-6",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        log::info!("contact submit: {}/{}", name.read(), email.read());
                    },
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "contact-name", {tr(l, "auth.name")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            id: "contact-name", autocomplete: "name",
                            r#type: "text", name: "name", required: true,
                            value: "{name.read()}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "contact-email", {tr(l, "auth.email")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            id: "contact-email", autocomplete: "email",
                            r#type: "email", name: "email", required: true,
                            value: "{email.read()}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "contact-message", {tr(l, "auth.message")} }
                        textarea { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors resize-y",
                            id: "contact-message", name: "message", rows: 5, required: true,
                            value: "{message.read()}",
                            oninput: move |e| message.set(e.value()),
                        }
                    }
                    button { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5 mt-2", r#type: "submit", {tr(l, "auth.send")} }
                }
            }
        }
    }
}

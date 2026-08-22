use dioxus::prelude::*;
use std::string::String;
use crate::i18n::{tr, use_i18n};

#[component]
pub fn SignupPage() -> Element {
    let l = *use_i18n().lang.read();
    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    rsx! {
        section { class: "py-24 wrap",
            div { class: "bg-surface border border-border rounded-2xl p-8 max-w-sm mx-auto",
                h1 { class: "text-3xl font-bold tracking-tight text-center mb-2", {tr(l, "nav.signup")} }
                form { class: "grid gap-4 mt-6",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        log::info!("signup submit: {}/{}", name.read(), email.read());
                    },
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", {tr(l, "auth.name")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            r#type: "text", name: "name", required: true,
                            value: "{name.read()}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", {tr(l, "auth.email")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            r#type: "email", name: "email", required: true,
                            value: "{email.read()}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", {tr(l, "auth.password")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            r#type: "password", name: "password", required: true,
                            value: "{password.read()}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }
                    button { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5 mt-2", r#type: "submit", {tr(l, "nav.signup")} }
                }
            }
        }
    }
}

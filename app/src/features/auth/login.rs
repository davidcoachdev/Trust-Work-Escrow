use dioxus::prelude::*;
use crate::i18n::{tr, use_i18n};
use crate::route::Route;

#[component]
pub fn LoginPage() -> Element {
    let l = *use_i18n().lang.read();
    let mut email = use_signal(|| String::new());
    let mut otp = use_signal(|| String::new());
    let mut step = use_signal(|| 0u8); // 0 email, 1 otp
    let mut msg = use_signal(|| String::new());
    let mut loading = use_signal(|| false);

    rsx! {
        section { class: "py-24 wrap",
            div { class: "bg-surface border border-border rounded-2xl p-8 max-w-sm mx-auto",
                h1 { class: "text-3xl font-bold tracking-tight text-center mb-2", {tr(l, "nav.login")} }
                p { class: "text-sm text-muted text-center mb-6", {tr(l, "auth.subtitle")} }
                if *step.read() == 0 {
                    form { class: "grid gap-4 mt-2",
                        onsubmit: move |evt| {
                            evt.prevent_default();
                            let e = email.read().clone();
                            spawn(async move {
                                loading.set(true);
                                msg.set(String::new());
                                // Server function free — lettre dev log si no hay SMTP
                                match crate::server::auth::email::send_otp(e).await {
                                    Ok(m) => { msg.set(m); step.set(1); },
                                    Err(err) => msg.set(format!("Error: {}", err)),
                                }
                                loading.set(false);
                            });
                        },
                        div { class: "grid gap-1.5",
                            label { class: "text-sm text-muted", {tr(l, "auth.email")} }
                            input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg outline-none focus:border-primary transition-colors",
                                r#type: "email", required: true,
                                value: "{email.read()}",
                                oninput: move |e| email.set(e.value()),
                                placeholder: "tu@correo.com"
                            }
                        }
                        button { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5 mt-2 disabled:opacity-50", r#type: "submit", disabled: *loading.read(),
                            if *loading.read() { "Enviando..." } else { "Enviar OTP" }
                        }
                        if !msg.read().is_empty() {
                            p { class: "text-sm text-center text-muted mt-2", "{msg.read()}" }
                        }
                    }
                } else {
                    form { class: "grid gap-4 mt-2",
                        onsubmit: move |evt| {
                            evt.prevent_default();
                            let e = email.read().clone();
                            let c = otp.read().clone();
                            spawn(async move {
                                loading.set(true);
                                msg.set(String::new());
                                match crate::server::auth::email::verify_otp_server(e, c).await {
                                    Ok(_) => {
                                        msg.set("¡Verificado! Redirigiendo a Config...".to_string());
                                        let nav = dioxus::prelude::navigator();
                                        nav.push(Route::ConfigPage {});
                                    },
                                    Err(err) => msg.set(format!("Error: {}", err)),
                                }
                                loading.set(false);
                            });
                        },
                        p { class: "text-sm text-muted text-center", "OTP enviado a {email.read()} (mira logs dev si no hay SMTP)" }
                        div { class: "grid gap-1.5",
                            label { class: "text-sm text-muted", "Código OTP (6 dígitos)" }
                            input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg outline-none focus:border-primary transition-colors tracking-widest text-center text-lg",
                                r#type: "text", required: true, maxlength: "6",
                                value: "{otp.read()}",
                                oninput: move |e| otp.set(e.value()),
                                placeholder: "123456"
                            }
                        }
                        button { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5 mt-2 disabled:opacity-50", r#type: "submit", disabled: *loading.read(),
                            if *loading.read() { "Verificando..." } else { "Verificar OTP" }
                        }
                        button { class: "text-sm text-muted text-center mt-2 underline", r#type: "button",
                            onclick: move |_| { step.set(0); msg.set(String::new()); },
                            "← Cambiar correo"
                        }
                        if !msg.read().is_empty() {
                            p { class: "text-sm text-center mt-2", class: if msg.read().contains("Verificado") { "text-primary" } else { "text-muted" }, "{msg.read()}" }
                        }
                    }
                }
            }
        }
    }
}

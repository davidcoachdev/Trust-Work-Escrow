use crate::i18n::{tr, use_i18n};
use crate::route::Route;
use crate::server::auth::guest::{use_auth, User};
use crate::server::auth::users::login_or_create_user;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
fn persist_email(email: &str) {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item("twe-email", email);
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn persist_email(_email: &str) {}

#[component]
pub fn SignupPage() -> Element {
    let l = *use_i18n().lang.read();
    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut msg = use_signal(|| String::new());
    let auth = use_auth();
    let nav = navigator();

    rsx! {
        section { class: "py-24 wrap",
            div { class: "bg-surface border border-border rounded-2xl p-8 max-w-sm mx-auto",
                h1 { class: "text-3xl font-bold tracking-tight text-center mb-2", {tr(l, "nav.signup")} }
                p { class: "text-sm text-muted text-center mb-2", {tr(l, "auth.subtitle")} }
                form { class: "grid gap-4 mt-6",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        let e = email.read().trim().to_lowercase();
                        if !e.contains('@') {
                            msg.set("Email inválido".to_string());
                            return;
                        }
                        let mut auth = auth;
                        let nav = nav.clone();
                        spawn(async move {
                            msg.set("Creando cuenta...".to_string());
                            // Sin distinción global: todos reciben permisos completos.
                            match login_or_create_user(e.clone(), "client".to_string()).await {
                                Ok(user) => {
                                    persist_email(&user.email);
                                    auth.user.set(Some(user.clone()));
                                    msg.set("¡Cuenta creada! Bienvenido...".to_string());
                                    nav.push(Route::ClientDashboardGuard {});
                                }
                                Err(err) => {
                                    log::warn!("signup login_or_create_user failed: {:?}", err);
                                    msg.set(format!("Error: {}", err));
                                    // degraded fallback con permisos completos
                                    let mut user_sig = auth.user;
                                    user_sig.set(Some(User {
                                        email: e.clone(),
                                        wallet_pubkey: None,
                                        role: "client".to_string(),
                                        roles: vec!["client".to_string(), "freelancer".to_string()],
                                        permissions: vec![
                                            "jobs:create".to_string(),
                                            "jobs:apply".to_string(),
                                            "jobs:view".to_string(),
                                            "jobs:view:own".to_string(),
                                            "config:wallet".to_string(),
                                        ],
                                        is_guest: false,
                                        created_at: 0,
                                        updated_at: 0,
                                        is_active: true,
                                    }));
                                    persist_email(&e);
                                    nav.push(Route::ClientDashboardGuard {});
                                }
                            }
                        });
                    },
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "signup-name", {tr(l, "auth.name")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            id: "signup-name", autocomplete: "name",
                            r#type: "text", name: "name",
                            value: "{name.read()}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "signup-email", {tr(l, "auth.email")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            id: "signup-email", autocomplete: "email",
                            r#type: "email", name: "email", required: true,
                            value: "{email.read()}",
                            oninput: move |e| email.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "signup-password", {tr(l, "auth.password")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg font-inherit outline-none focus:border-primary transition-colors",
                            id: "signup-password", autocomplete: "new-password",
                            r#type: "password", name: "password",
                            value: "{password.read()}",
                            oninput: move |e| password.set(e.value()),
                        }
                    }
                    // Rol por job, no global — sin selector. Todos pueden publicar y postular.
                    button { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5 mt-2", r#type: "submit", {tr(l, "nav.signup")} }
                    if !msg.read().is_empty() {
                        p { class: "text-sm text-center text-primary mt-3", "{msg.read()}" }
                    }
                }
                div { class: "text-sm text-center text-muted mt-6",
                    "¿Ya tenés cuenta? "
                    Link { class: "text-primary underline font-medium", to: Route::LoginPage {}, "Ingresá" }
                }
            }
        }
    }
}

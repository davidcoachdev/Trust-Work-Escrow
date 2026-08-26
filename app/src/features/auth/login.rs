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
pub fn LoginPage() -> Element {
    let l = *use_i18n().lang.read();
    let mut email = use_signal(|| String::new());
    let mut role = use_signal(|| "client".to_string());
    let mut msg = use_signal(|| String::new());
    let auth = use_auth();
    let nav = navigator();

    rsx! {
        section { class: "py-24 wrap",
            div { class: "bg-surface border border-border rounded-2xl p-8 max-w-sm mx-auto",
                h1 { class: "text-3xl font-bold tracking-tight text-center mb-2", {tr(l, "nav.login")} }
                p { class: "text-sm text-muted text-center mb-6", {tr(l, "auth.subtitle")} }
                form { class: "grid gap-4 mt-2",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        let e = email.read().trim().to_lowercase();
                        let r = role.read().clone();
                        if !e.contains('@') {
                            msg.set("Email inválido".to_string());
                            return;
                        }
                        if r != "client" && r != "freelancer" {
                            msg.set("Rol inválido".to_string());
                            return;
                        }
                        let mut auth = auth;
                        let nav = nav.clone();
                        spawn(async move {
                            msg.set("Verificando...".to_string());
                            match login_or_create_user(e.clone(), r.clone()).await {
                                Ok(user) => {
                                    persist_email(&user.email);
                                    auth.user.set(Some(user.clone()));
                                    msg.set("¡Bienvenido! Redirigiendo...".to_string());
                                    if user.role == "freelancer" {
                                        nav.push(Route::FreelancerDashboardGuard {});
                                    } else {
                                        nav.push(Route::ClientDashboardGuard {});
                                    }
                                }
                                Err(err) => {
                                    // fallback: keep in-memory auth so UX not blocked if DB down
                                    // but show error
                                    log::warn!("login_or_create_user failed: {:?}", err);
                                    msg.set(format!("Error: {}", err));
                                    // still set in-memory as degraded fallback (MVP bypass preserved)
                                    let mut user_sig = auth.user;
                                    user_sig.set(Some(User {
                                        email: e.clone(),
                                        wallet_pubkey: None,
                                        role: r.clone(),
                                        roles: vec![r.clone()],
                                        permissions: vec![],
                                        is_guest: false,
                                        created_at: 0,
                                        updated_at: 0,
                                        is_active: true,
                                    }));
                                    persist_email(&e);
                                    if r == "freelancer" {
                                        nav.push(Route::FreelancerDashboardGuard {});
                                    } else {
                                        nav.push(Route::ClientDashboardGuard {});
                                    }
                                }
                            }
                        });
                    },
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", r#for: "login-email", {tr(l, "auth.email")} }
                        input { class: "bg-bg border border-border rounded-xl px-3.5 py-3 text-fg outline-none focus:border-primary transition-colors",
                            id: "login-email",
                            r#type: "email", required: true, autocomplete: "email",
                            value: "{email.read()}",
                            oninput: move |e| email.set(e.value()),
                            placeholder: "tu@correo.com"
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm text-muted", "Rol" }
                        div { class: "grid grid-cols-2 gap-2",
                            button {
                                class: if *role.read() == "client" { "bg-primary text-on-primary rounded-xl px-3 py-2.5 text-sm font-medium border border-primary" } else { "bg-bg border border-border rounded-xl px-3 py-2.5 text-sm font-medium" },
                                r#type: "button",
                                onclick: move |_| role.set("client".to_string()),
                                "Cliente"
                            }
                            button {
                                class: if *role.read() == "freelancer" { "bg-primary text-on-primary rounded-xl px-3 py-2.5 text-sm font-medium border border-primary" } else { "bg-bg border border-border rounded-xl px-3 py-2.5 text-sm font-medium" },
                                r#type: "button",
                                onclick: move |_| role.set("freelancer".to_string()),
                                "Freelancer"
                            }
                        }
                        p { class: "text-xs text-muted", "Elegí cómo querés usar Trust Work." }
                    }
                    button { class: "inline-flex items-center justify-center gap-2 rounded-xl px-5 py-3 text-base font-medium bg-primary text-on-primary transition hover:-translate-y-0.5 mt-2", r#type: "submit",
                        "Ingresar"
                    }
                    if !msg.read().is_empty() {
                        p { class: "text-sm text-center mt-2", class: if msg.read().contains("Bienvenido") || msg.read().contains("Verificando") { "text-primary" } else { "text-red-500" }, "{msg.read()}" }
                    }
                }
                div { class: "text-sm text-center text-muted mt-6",
                    "¿No tenés cuenta? "
                    Link { class: "text-primary underline font-medium", to: Route::SignupPage {}, "Registrate" }
                }
             }
         }
     }
}

use crate::i18n::{tr, use_i18n};
use crate::route::Route;
use crate::server::auth::guest::{use_auth, MenuConfig};
use crate::ui::Sidebar;
use dioxus::prelude::*;

const SIDEBAR_KEY: &str = "twe-sidebar-collapsed";

#[cfg(target_arch = "wasm32")]
fn load_collapsed() -> bool {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            if let Ok(Some(v)) = storage.get_item(SIDEBAR_KEY) {
                return v == "1" || v == "true";
            }
        }
        if let Ok(w) = win.inner_width() {
            if let Some(width) = w.as_f64() {
                if width < 768.0 {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn load_collapsed() -> bool {
    false
}

fn short_pubkey(pk: &str) -> String {
    if pk.len() <= 10 {
        pk.to_string()
    } else {
        format!("{}...{}", &pk[..6], &pk[pk.len().saturating_sub(4)..])
    }
}

#[component]
pub fn DashboardLayout() -> Element {
    let auth = use_auth();
    let lang = *use_i18n().lang.read();
    let (roles, perms) = {
        let guard = auth.user.read();
        if let Some(u) = guard.as_ref() {
            (u.normalized_roles(), u.permissions.clone())
        } else {
            (vec!["guest".to_string()], vec![])
        }
    };
    let _menu = MenuConfig::new(roles.clone(), perms.clone());
    let user_opt = auth.user.read().clone();
    let mut collapsed = use_signal(load_collapsed);

    // Persist collapsed to localStorage + auto-collapse <768px handled via load_collapsed and resize poll
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            let val = *collapsed.read();
            if let Some(win) = web_sys::window() {
                if let Ok(Some(storage)) = win.local_storage() {
                    let _ = storage.set_item(SIDEBAR_KEY, if val { "1" } else { "0" });
                }
            }
        });
        // Resize listener: collapsed already derived from localStorage/innerWidth on load; continuous resize handled via CSS md: prefix + backdrop
    }

    let is_collapsed = *collapsed.read();

    rsx! {
        document::Meta { name: "robots", content: "noindex, nofollow" }
        div { class: "flex min-h-screen bg-bg text-fg",
            Sidebar { roles: roles, permissions: perms, collapsed: collapsed }
            // Backdrop for mobile when open
            if !is_collapsed {
                div {
                    class: "fixed inset-0 z-20 bg-black/30 md:hidden",
                    onclick: move |_| collapsed.set(true),
                }
            }
            div { class: "flex-1 flex flex-col min-w-0",
                header { class: "sticky top-0 z-10 bg-surface/80 backdrop-blur border-b border-border px-6 py-3 flex items-center justify-between gap-4 animate-nav-in",
                    div { class: "flex items-center gap-3",
                        button {
                            class: "inline-flex items-center justify-center w-8 h-8 rounded-lg border border-border bg-bg hover:bg-surface transition md:flex",
                            r#type: "button",
                            aria_label: "{tr(lang, \"dashboard.toggleSidebar\")}",
                            aria_expanded: "{!is_collapsed}",
                            onclick: move |_| {
                                let v = *collapsed.read();
                                collapsed.set(!v);
                            },
                            span { class: "text-sm", if is_collapsed { "☰" } else { "✕" } }
                        }
                        div { class: "flex items-center gap-3 text-sm flex-wrap",
                            if let Some(u) = user_opt.clone() {
                                span { class: "font-medium truncate max-w-[180px]", "{u.email}" }
                                if let Some(pk) = u.wallet_pubkey.clone() {
                                    span { class: "font-mono text-xs bg-bg border border-border rounded-full px-3 py-1", "{short_pubkey(&pk)}" }
                                } else {
                                    Link { class: "text-xs bg-amber-500/10 border border-amber-500/30 rounded-full px-3 py-1 text-amber-600 hover:bg-amber-500/20 transition", to: Route::ConfigPage {}, "{tr(lang, \"dashboard.createWallet\")}" }
                                }
                                if u.is_guest {
                                    span { class: "text-xs bg-surface border border-border rounded-full px-2 py-1 text-muted", "{tr(lang, \"dashboard.guestReadOnly\")}" }
                                } else if u.wallet_pubkey.is_none() {
                                    span { class: "text-xs bg-surface border border-border rounded-full px-2 py-1 text-muted", "{tr(lang, \"dashboard.readOnly\")}" }
                                }
                            } else {
                                span { class: "text-sm text-muted", "{tr(lang, \"dashboard.loading\")}" }
                            }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        Link { class: "text-sm text-primary underline", to: Route::LandingPage {}, "← Landing" }
                    }
                }
                div { class: "flex-1 p-8 page-enter",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

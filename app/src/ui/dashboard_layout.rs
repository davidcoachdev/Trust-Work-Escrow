use crate::i18n::{tr, use_i18n};
use crate::route::Route;
use crate::server::auth::guest::{use_auth, MenuConfig};
use crate::ui::Sidebar;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
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

    // Persist collapsed to localStorage
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
        // Continuous resize listener: auto-collapse when viewport shrinks <768px.
        // Uses a leaked Closure (intentional 'static) to keep the listener alive
        // for the app lifetime; no cleanup needed as DashboardLayout is mounted
        // for the entire dashboard session.
        use_effect(move || {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;
            if let Some(win) = web_sys::window() {
                let mut collapsed_sig = collapsed;
                let win_clone = win.clone();
                let closure = Closure::wrap(Box::new(move || {
                    if let Ok(w) = win_clone.inner_width() {
                        if let Some(width) = w.as_f64() {
                            // Auto-collapse on narrow viewports; do not auto-expand
                            // above 768 to respect explicit user toggles (persisted
                            // via SIDEBAR_KEY). Continuous listener ensures 1024→390
                            // without reload still collapses.
                            if width < 768.0 {
                                collapsed_sig.set(true);
                            }
                        }
                    }
                }) as Box<dyn FnMut()>);
                let _ = win.add_event_listener_with_callback(
                    "resize",
                    closure.as_ref().unchecked_ref(),
                );
                closure.forget();
            }
        });
        // Focus trap for mobile drawer: when open as overlay (<768 + !collapsed),
        // trap Tab / Shift+Tab inside the drawer and restore focus on close.
        use_effect(move || {
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;
            let is_open = !*collapsed.read();
            if !is_open {
                return;
            }
            if let Some(win) = web_sys::window() {
                if let Ok(w) = win.inner_width() {
                    let is_mobile = w.as_f64().is_some_and(|v| v < 768.0);
                    if !is_mobile {
                        return;
                    }
                }
                if let Some(doc) = win.document() {
                    // Focus first focusable element inside aside when drawer opens.
                    if let Ok(Some(el)) = doc.query_selector("aside a, aside button") {
                        if let Some(html) = el.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = html.focus();
                        }
                    }
                    let doc_clone = doc.clone();
                    let mut collapsed_for_esc = collapsed;
                    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                        if e.key() == "Escape" {
                            collapsed_for_esc.set(true);
                            e.prevent_default();
                            return;
                        }
                        if e.key() != "Tab" {
                            return;
                        }
                        let Ok(list) =
                            doc_clone.query_selector_all("aside a, aside button, aside [tabindex]")
                        else {
                            return;
                        };
                        if list.length() == 0 {
                            return;
                        }
                        let first = list.get(0).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
                        let last = list
                            .get(list.length() - 1)
                            .and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
                        let active = doc_clone.active_element();
                        let active_el = active.and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok());
                        let shift = e.shift_key();
                        let at_first = first.as_ref().zip(active_el.as_ref()).is_some_and(|(f, a)| f == a);
                        let at_last = last.as_ref().zip(active_el.as_ref()).is_some_and(|(l, a)| l == a);
                        if !shift && at_last {
                            if let Some(f) = first {
                                let _ = f.focus();
                                e.prevent_default();
                            }
                        } else if shift && at_first {
                            if let Some(l) = last {
                                let _ = l.focus();
                                e.prevent_default();
                            }
                        }
                    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
                    let _ = doc.add_event_listener_with_callback(
                        "keydown",
                        closure.as_ref().unchecked_ref(),
                    );
                    closure.forget();
                }
            }
        });
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

use dioxus::prelude::*;
use std::str::FromStr;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Window};

/// Available runtime themes (skins). `dcdev` is the master brand.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Dcdev,
    Cyan,
    Solana,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Dcdev => "dcdev",
            Theme::Cyan => "cyan",
            Theme::Solana => "solana",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Dcdev => "DCdev",
            Theme::Cyan => "Cyan",
            Theme::Solana => "Solana",
        }
    }

    /// Preview swatch (primary) for the theme picker, matching tailwind.css tokens.
    pub fn swatch_primary(self) -> &'static str {
        match self {
            Theme::Dcdev => "#ff3c3c",
            Theme::Cyan => "#00d4ff",
            Theme::Solana => "#14f195",
        }
    }

    /// Preview swatch (secondary) for the theme picker, matching tailwind.css tokens.
    pub fn swatch_secondary(self) -> &'static str {
        match self {
            Theme::Dcdev => "#781414",
            Theme::Cyan => "#6464c8",
            Theme::Solana => "#9945ff",
        }
    }

    pub fn all() -> &'static [Theme] {
        &[Theme::Dcdev, Theme::Cyan, Theme::Solana]
    }
}

impl FromStr for Theme {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "dcdev" => Ok(Theme::Dcdev),
            "cyan" => Ok(Theme::Cyan),
            "solana" => Ok(Theme::Solana),
            _ => Err(()),
        }
    }
}

pub const THEME_KEY: &str = "twe-theme";

/// Light/dark mode, orthogonal to the skin.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
}

impl FromStr for Mode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "dark" => Ok(Mode::Dark),
            "light" => Ok(Mode::Light),
            _ => Err(()),
        }
    }
}

pub const MODE_KEY: &str = "twe-mode";

/// Apply the mode to <html data-mode> and persist it.
#[cfg(target_arch = "wasm32")]
pub fn apply_mode(mode: Mode) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-mode", mode.as_str());
            }
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item(MODE_KEY, mode.as_str());
            }
        }
    }
}

/// SSR: no browser DOM to touch.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_mode(_mode: Mode) {}

/// Read the persisted mode, falling back to `Dark`.
#[cfg(target_arch = "wasm32")]
pub fn load_mode() -> Mode {
    window()
        .and_then(|w: Window| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(MODE_KEY).ok().flatten())
        .and_then(|v| Mode::from_str(&v).ok())
        .unwrap_or(Mode::Dark)
}

/// SSR fallback.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_mode() -> Mode {
    Mode::Dark
}

#[derive(Clone, Copy)]
pub struct ModeContext {
    pub mode: Signal<Mode>,
}

pub fn use_mode() -> ModeContext {
    use_context::<ModeContext>()
}

/// Apply the theme to <html data-theme> and persist it.
#[cfg(target_arch = "wasm32")]
pub fn apply_theme(theme: Theme) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", theme.as_str());
            }
            if let Ok(Some(storage)) = win.local_storage() {
                let _ = storage.set_item(THEME_KEY, theme.as_str());
            }
        }
    }
}

/// SSR: no browser DOM to touch.
#[cfg(not(target_arch = "wasm32"))]
pub fn apply_theme(_theme: Theme) {}

/// Read the persisted theme, falling back to `dcdev`.
#[cfg(target_arch = "wasm32")]
pub fn load_theme() -> Theme {
    window()
        .and_then(|w: Window| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(THEME_KEY).ok().flatten())
        .and_then(|v| Theme::from_str(&v).ok())
        .unwrap_or(Theme::Dcdev)
}

/// SSR fallback.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_theme() -> Theme {
    Theme::Dcdev
}

#[derive(Clone, Copy)]
pub struct ThemeContext {
    pub theme: Signal<Theme>,
}

pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>()
}

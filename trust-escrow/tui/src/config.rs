use anyhow::{anyhow, Result};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Theme ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub highlight: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub border: Color,
    pub title: Color,
    pub muted: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "dark".into(),
            bg: Color::Rgb(26, 26, 46),
            fg: Color::Rgb(224, 224, 224),
            accent: Color::Rgb(0, 212, 255),
            highlight: Color::Rgb(100, 100, 200),
            error: Color::Rgb(255, 85, 85),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 183, 77),
            border: Color::Rgb(80, 80, 120),
            title: Color::Rgb(0, 212, 255),
            muted: Color::Rgb(100, 100, 130),
        }
    }

    pub fn light() -> Self {
        Self {
            name: "light".into(),
            bg: Color::Rgb(245, 245, 250),
            fg: Color::Rgb(30, 30, 50),
            accent: Color::Rgb(0, 120, 200),
            highlight: Color::Rgb(200, 200, 240),
            error: Color::Rgb(200, 50, 50),
            success: Color::Rgb(30, 150, 60),
            warning: Color::Rgb(200, 140, 0),
            border: Color::Rgb(180, 180, 200),
            title: Color::Rgb(0, 100, 180),
            muted: Color::Rgb(140, 140, 160),
        }
    }

    pub fn hacker() -> Self {
        Self {
            name: "hacker".into(),
            bg: Color::Rgb(0, 10, 0),
            fg: Color::Rgb(0, 255, 65),
            accent: Color::Rgb(0, 200, 50),
            highlight: Color::Rgb(0, 80, 30),
            error: Color::Rgb(255, 50, 50),
            success: Color::Rgb(0, 255, 100),
            warning: Color::Rgb(200, 200, 0),
            border: Color::Rgb(0, 100, 30),
            title: Color::Rgb(0, 255, 65),
            muted: Color::Rgb(0, 120, 40),
        }
    }

    pub fn ocean() -> Self {
        Self {
            name: "ocean".into(),
            bg: Color::Rgb(10, 25, 47),
            fg: Color::Rgb(168, 218, 220),
            accent: Color::Rgb(100, 200, 255),
            highlight: Color::Rgb(30, 60, 100),
            error: Color::Rgb(255, 107, 107),
            success: Color::Rgb(46, 213, 115),
            warning: Color::Rgb(255, 200, 87),
            border: Color::Rgb(40, 80, 130),
            title: Color::Rgb(100, 200, 255),
            muted: Color::Rgb(80, 120, 160),
        }
    }

    pub fn by_name(name: &str) -> Self {
        match name {
            "light" => Self::light(),
            "hacker" => Self::hacker(),
            "ocean" => Self::ocean(),
            _ => Self::dark(),
        }
    }

    pub fn names() -> &'static [&'static str] {
        &["dark", "light", "hacker", "ocean"]
    }
}

// ─── Wallet Config ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletConfig {
    pub name: String,
    pub path: String,
    pub role: String,
}

// ─── Settings ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub rpc_url: String,
    pub wallets: Vec<WalletConfig>,
    pub active_wallet: usize,
}

impl Default for Settings {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Self {
            theme: "dark".into(),
            rpc_url: "http://127.0.0.1:8899".into(),
            wallets: vec![WalletConfig {
                name: "Default".into(),
                path: format!("{home}/.config/solana/id.json"),
                role: "admin".into(),
            }],
            active_wallet: 0,
        }
    }
}

impl Settings {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("trust-escrow-tui")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(_) => {}
                },
                Err(_) => {}
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| anyhow!("Cannot create config dir: {e}"))?;
        let content =
            toml::to_string_pretty(self).map_err(|e| anyhow!("Cannot serialize config: {e}"))?;
        std::fs::write(Self::config_path(), content)
            .map_err(|e| anyhow!("Cannot write config: {e}"))?;
        Ok(())
    }
}

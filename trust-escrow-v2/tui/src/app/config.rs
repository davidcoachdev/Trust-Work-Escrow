//! TUI-specific configuration management
//!
//! Extends trust-escrow-shared configuration with TUI-specific settings
//! such as UI preferences, colors, and behavior options.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use trust_escrow_shared::EscrowConfig;

/// TUI-specific configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Base escrow configuration
    pub escrow: EscrowConfig,

    /// UI preferences
    pub ui: UiPreferences,

    /// Performance settings
    pub performance: PerformanceSettings,

    /// Developer/debug settings
    pub debug: DebugSettings,
}

/// UI appearance and behavior preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    /// Auto-refresh interval in seconds (0 = disabled)
    pub auto_refresh_interval: u64,

    /// Show detailed timestamps in UI
    pub show_timestamps: bool,

    /// Use unicode symbols and emojis
    pub use_unicode: bool,

    /// Compact mode (reduced spacing)
    pub compact_mode: bool,

    /// Default number of items per page in lists
    pub items_per_page: usize,

    /// Remember last view on startup
    pub remember_last_view: bool,

    /// Color scheme preference
    pub color_scheme: ColorScheme,
}

/// Color scheme options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ColorScheme {
    Default,      // Standard terminal colors
    Dark,         // Dark theme optimized
    Light,        // Light theme optimized
    Mono,         // Monochrome (no colors)
    HighContrast, // Accessibility focused
}

/// Performance-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum number of jobs to cache
    pub max_cached_jobs: usize,

    /// Maximum number of notifications to keep
    pub max_notifications: usize,

    /// Data refresh rate in seconds
    pub data_refresh_rate: u64,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Enable background data fetching
    pub background_updates: bool,

    /// Batch size for loading operations
    pub batch_size: usize,
}

/// Debug and development settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSettings {
    /// Enable verbose logging
    pub verbose_logging: bool,

    /// Enable performance metrics overlay
    pub show_performance_metrics: bool,

    /// Enable state inspector view
    pub enable_state_inspector: bool,

    /// Log file path (None = no file logging)
    pub log_file: Option<PathBuf>,

    /// Enable network request debugging
    pub debug_network: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            escrow: EscrowConfig::default(),
            ui: UiPreferences::default(),
            performance: PerformanceSettings::default(),
            debug: DebugSettings::default(),
        }
    }
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            auto_refresh_interval: 30, // 30 seconds
            show_timestamps: true,
            use_unicode: true,
            compact_mode: false,
            items_per_page: 20,
            remember_last_view: true,
            color_scheme: ColorScheme::Default,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_cached_jobs: 100,
            max_notifications: 50,
            data_refresh_rate: 10,  // 10 seconds
            connection_timeout: 30, // 30 seconds
            background_updates: true,
            batch_size: 20,
        }
    }
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            verbose_logging: false,
            show_performance_metrics: false,
            enable_state_inspector: false,
            log_file: None,
            debug_network: false,
        }
    }
}

impl TuiConfig {
    /// Load TUI config from file, falling back to default if not found
    pub fn load() -> Result<Self> {
        // In a full implementation, this would load from ~/.config/trust-escrow-tui/config.toml
        // For now, we'll create a default config with the base escrow config loaded
        let escrow_config = EscrowConfig::load().unwrap_or_default();

        Ok(Self {
            escrow: escrow_config,
            ui: UiPreferences::default(),
            performance: PerformanceSettings::default(),
            debug: DebugSettings::default(),
        })
    }

    /// Save TUI config to file
    pub fn save(&self) -> Result<()> {
        // Implementation would save to config file
        // For now, this is a placeholder
        Ok(())
    }

    /// Get configuration file path
    pub fn config_path() -> PathBuf {
        // Would typically be ~/.config/trust-escrow-tui/config.toml
        PathBuf::from("~/.config/trust-escrow-tui/config.toml")
    }

    /// Check if auto-refresh is enabled
    pub fn auto_refresh_enabled(&self) -> bool {
        self.ui.auto_refresh_interval > 0
    }

    /// Get auto-refresh duration
    pub fn auto_refresh_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.ui.auto_refresh_interval)
    }

    /// Check if background updates are enabled
    pub fn background_updates_enabled(&self) -> bool {
        self.performance.background_updates
    }

    /// Get connection timeout duration
    pub fn connection_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.performance.connection_timeout)
    }

    /// Check if verbose logging is enabled
    pub fn verbose_logging(&self) -> bool {
        self.debug.verbose_logging
    }

    /// Apply color scheme to determine if we should use colors
    pub fn should_use_colors(&self) -> bool {
        !matches!(self.ui.color_scheme, ColorScheme::Mono)
    }

    /// Check if unicode symbols should be used
    pub fn use_unicode(&self) -> bool {
        self.ui.use_unicode
    }
}

/// Configuration validation
impl TuiConfig {
    /// Validate configuration settings
    pub fn validate(&self) -> Result<()> {
        // Validate escrow config first
        // Note: EscrowConfig doesn't have a validate method in the current implementation

        // Validate TUI-specific settings
        if self.performance.max_cached_jobs == 0 {
            return Err(anyhow::anyhow!("max_cached_jobs must be greater than 0"));
        }

        if self.performance.batch_size == 0 {
            return Err(anyhow::anyhow!("batch_size must be greater than 0"));
        }

        if self.ui.items_per_page == 0 {
            return Err(anyhow::anyhow!("items_per_page must be greater than 0"));
        }

        if self.performance.connection_timeout == 0 {
            return Err(anyhow::anyhow!("connection_timeout must be greater than 0"));
        }

        Ok(())
    }
}

//! Configuration loading and handling for translatable content
//!
//! This module provides functionality to load and manage configuration
//! settings for localization/translation workflows from a TOML file.

use std::env::var;
use std::fs::read_to_string;
use std::io::Error as IoError;
use std::sync::OnceLock;

use serde::Deserialize;
use thiserror::Error;
use toml::de::Error as TomlError;
use toml::from_str as toml_from_str;

/// Errors that can occur during configuration loading
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO error occurred while reading configuration file
    #[error("IO error reading configuration: {0:#}")]
    Io(#[from] IoError),

    /// TOML parsing error with location information
    #[error(
        "TOML parse error '{}'{}",
        .0.message(),
        .0.span().map(|l| format!(" in ./translatable.toml:{}:{}", l.start, l.end))
            .unwrap_or_else(|| "".into())
    )]
    ParseToml(#[from] TomlError),

    /// Invalid environment variable value for configuration options
    #[error("Invalid value '{1}' for environment variable '{0}'")]
    InvalidEnvVarValue(String, String),
}

/// Wrapper type for locales directory path with validation
#[derive(Deserialize)]
pub struct LocalesPath(String);

impl Default for LocalesPath {
    /// Default path to translations directory
    fn default() -> Self {
        LocalesPath("./translations".into())
    }
}

/// File search order strategy
#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SeekMode {
    /// Alphabetical order (default)
    #[default]
    Alphabetical,

    /// Reverse alphabetical order
    Unalphabetical,
}

/// Translation conflict resolution strategy
#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TranslationOverlap {
    /// Last found translation overwrites previous ones (default)
    #[default]
    Overwrite,

    /// First found translation is preserved
    Ignore,
}

/// Main configuration structure for translation system
#[derive(Deserialize)]
pub struct TranslatableConfig {
    /// Path to directory containing translation files
    ///
    /// # Example
    /// ```toml
    /// path = "./locales"
    /// ```
    #[serde(default)]
    path: LocalesPath,

    /// File processing order strategy
    ///
    /// Default: alphabetical file processing
    #[serde(default)]
    seek_mode: SeekMode,

    /// Translation conflict resolution strategy
    ///
    /// Determines behavior when multiple files contain the same translation
    /// path
    #[serde(default)]
    overlap: TranslationOverlap,
}

impl TranslatableConfig {
    /// Get reference to configured locales path
    pub fn path(&self) -> &str {
        &self.path.0
    }

    /// Get current seek mode strategy
    pub fn seek_mode(&self) -> SeekMode {
        self.seek_mode
    }

    /// Get current overlap resolution strategy
    pub fn overlap(&self) -> TranslationOverlap {
        self.overlap
    }

    /// Set the locales path from a string (e.g., from environment variable)
    fn set_path(&mut self, path: String) {
        self.path = LocalesPath(path);
    }

    /// Update seek mode from string value (for environment variable parsing)
    fn set_seek_mode(&mut self, mode: SeekMode) {
        self.seek_mode = mode;
    }

    /// Update overlap strategy from string value (for environment variable
    /// parsing)
    fn set_overlap(&mut self, strategy: TranslationOverlap) {
        self.overlap = strategy;
    }
}

/// Global configuration cache
static TRANSLATABLE_CONFIG: OnceLock<TranslatableConfig> = OnceLock::new();

/// Load configuration from file or use defaults
///
/// # Implementation Notes
/// - Uses `OnceLock` for thread-safe singleton initialization
/// - Missing config file is not considered an error
/// - Config file must be named `translatable.toml` in root directory
/// - Environment variables take precedence over TOML configuration
/// - Supported environment variables:
///   - `LOCALES_PATH`: Overrides translation directory path
///   - `SEEK_MODE`: Sets file processing order ("alphabetical" or
///     "unalphabetical")
///   - `TRANSLATION_OVERLAP`: Sets conflict strategy ("overwrite" or "ignore")
///
/// # Panics
/// Will not panic but returns ConfigError for:
/// - Malformed TOML syntax
/// - Filesystem permission issues
/// - Invalid environment variable values
pub fn load_config() -> Result<&'static TranslatableConfig, ConfigError> {
    if let Some(config) = TRANSLATABLE_CONFIG.get() {
        return Ok(config);
    }

    // Load base configuration from TOML file
    let toml_content = read_to_string("./translatable.toml").unwrap_or_default();
    let mut config: TranslatableConfig = toml_from_str(&toml_content)?;

    // Environment variable overrides
    // --------------------------------------------------
    // LOCALES_PATH: Highest precedence for translation directory
    if let Ok(env_path) = var("LOCALES_PATH") {
        config.set_path(env_path);
    }

    // SEEK_MODE: Control file processing order
    if let Ok(env_seek) = var("SEEK_MODE") {
        config.set_seek_mode(match env_seek.to_lowercase().as_str() {
            "alphabetical" => SeekMode::Alphabetical,
            "unalphabetical" => SeekMode::Unalphabetical,
            _ => return Err(ConfigError::InvalidEnvVarValue("SEEK_MODE".into(), env_seek)),
        });
    }

    // TRANSLATION_OVERLAP: Manage translation conflicts
    if let Ok(env_overlap) = var("TRANSLATION_OVERLAP") {
        config.set_overlap(match env_overlap.to_lowercase().as_str() {
            "overwrite" => TranslationOverlap::Overwrite,
            "ignore" => TranslationOverlap::Ignore,
            _ => {
                return Err(ConfigError::InvalidEnvVarValue(
                    "TRANSLATION_OVERLAP".into(),
                    env_overlap,
                ));
            },
        });
    }

    // Freeze configuration in global cache
    Ok(TRANSLATABLE_CONFIG.get_or_init(|| config))
}

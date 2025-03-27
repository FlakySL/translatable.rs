//! Configuration loading and handling for translatable content
//!
//! This module provides functionality to load and manage configuration
//! settings for localization/translation workflows from a TOML file.

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
#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SeekMode {
    /// Alphabetical order (default)
    #[default]
    Alphabetical,

    /// Reverse alphabetical order
    Unalphabetical,
}

/// Translation conflict resolution strategy
#[derive(Deserialize, Default)]
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
    pub fn seek_mode(&self) -> &SeekMode {
        &self.seek_mode
    }

    /// Get current overlap resolution strategy
    pub fn overlap(&self) -> &TranslationOverlap {
        &self.overlap
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
///
/// # Panics
/// Will not panic but returns ConfigError for:
/// - Malformed TOML syntax
/// - Filesystem permission issues
pub fn load_config() -> Result<&'static TranslatableConfig, ConfigError> {
    if let Some(config) = TRANSLATABLE_CONFIG.get() {
        return Ok(config);
    }

    let config: TranslatableConfig =
        toml_from_str(read_to_string("./translatable.toml")
            .unwrap_or("".into()) // if no config file is found use defaults.
            .as_str())?;

    Ok(TRANSLATABLE_CONFIG.get_or_init(|| config))
}

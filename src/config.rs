use std::{fs::read_to_string, io::Error as IoError, sync::OnceLock};
use serde::Deserialize;
use thiserror::Error;
use toml::{from_str as toml_from_str, de::Error as TomlError};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{0:#}")]
    Io(#[from] IoError),

    #[error("{0:#}")]
    Toml(#[from] TomlError)
}

#[derive(Deserialize)]
#[serde(rename = "snake_case")]
pub enum SeekMode {
    Alphabetical,
    Unalphabetical
}

#[derive(Deserialize)]
#[serde(rename = "snake_case")]
pub enum TranslationOverlap {
    Overwrite,
    Ignore
}

// tracking issue: https://github.com/serde-rs/serde/issues/1030
#[doc(hidden)]
fn __d_path() -> String { "./translations".into() }
#[doc(hidden)]
fn __d_seek_mode() -> SeekMode { SeekMode::Alphabetical }
#[doc(hidden)]
fn __d_overlap() -> TranslationOverlap { TranslationOverlap::Overwrite }

#[derive(Deserialize)]
pub struct TranslatableConfig {
    #[serde(default = "__d_path")]
    path: String,
    #[serde(default = "__d_seek_mode")]
    seek_mode: SeekMode,
    #[serde(default = "__d_overlap")]
    overlap: TranslationOverlap
}

impl TranslatableConfig {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn seek_mode(&self) -> &SeekMode {
        &self.seek_mode
    }

    pub fn overlap(&self) -> &TranslationOverlap {
        &self.overlap
    }
}

static TRANSLATABLE_CONFIG: OnceLock<TranslatableConfig> = OnceLock::new();

pub fn load_config() -> Result<&'static TranslatableConfig, ConfigError> {
    if let Some(config) = TRANSLATABLE_CONFIG.get() {
        return Ok(config);
    }

    let config = toml_from_str(
        read_to_string("./translatable.toml")
            .unwrap_or("".into()) // if no config file is found use defaults.
            .as_str()
    )?;

    Ok(TRANSLATABLE_CONFIG.get_or_init(|| config))
}

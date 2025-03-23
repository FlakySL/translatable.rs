use crate::{data::{config::ConfigError, translations::TransformError}, languages::Iso639a};
use std::io::Error as IoError;
use thiserror::Error;
use toml::de::Error as TomlError;

/// Errors that can occur during translation processing.
#[derive(Error, Debug)]
pub enum TranslationError {
    /// Configuration-related error
    #[error("{0:#}")]
    Config(#[from] ConfigError),

    /// IO operation error
    #[error("An IO Error occurred: {0:#}")]
    Io(#[from] IoError),

    /// Path contains invalid Unicode characters
    #[error("The path contains invalid unicode characters.")]
    InvalidUnicode,

    /// TOML parsing error with location information
    #[error(
        "Toml parse error '{}'{}",
        .0.message(),
        .0.span()
            .map(|l| format!(" in {}:{}:{}", .1, l.start, l.end))
            .unwrap_or("".into())
    )]
    ParseToml(TomlError, String),

    /// Invalid language code error with suggestions
    #[error(
        "'{0}' is not valid ISO 639-1. These are some valid languages including '{0}':\n{sorted_list}",
        sorted_list = .1.join(",\n")
    )]
    InvalidLanguage(String, Vec<String>),

    /// Invalid TOML structure in specific file
    #[error(
        "Invalid TOML structure in file {1}: {0}"
    )]
    InvalidTomlFormat(TransformError, String),

    #[error("The path '{0}' is not found in any of the translation files.")]
    PathNotFound(String),

    #[error("The language '{0:?}' ({0:#}) is not available for the '{1}' translation.")]
    LanguageNotAvailable(Iso639a, String)
}

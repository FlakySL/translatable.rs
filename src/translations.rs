//! Translation handling module that loads and validates translation files,
//! and provides functionality to retrieve translations based on language and path.

use crate::config::{ConfigError, SeekMode, TranslationOverlap, load_config};
use crate::languages::Iso639a;
use crate::macros::{TranslationLanguageType, TranslationPathType};
use proc_macro::TokenStream;
use std::fs::{read_dir, read_to_string};
use std::io::Error as IoError;
use std::sync::OnceLock;
use strum::IntoEnumIterator;
use thiserror::Error;
use toml::{Table, Value, de::Error as TomlError};

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
        "Invalid TOML structure in file {0}: Translation files must contain either nested tables or language translations, but not both at the same level."
    )]
    InvalidTomlFormat(String),
}

/// Global cache for loaded translations
static TRANSLATIONS: OnceLock<Vec<Table>> = OnceLock::new();

/// Recursively walk a directory and collect all file paths
///
/// # Implementation Details
/// Uses iterative depth-first search to avoid stack overflow
/// Handles filesystem errors and invalid Unicode paths
fn walk_dir(path: &str) -> Result<Vec<String>, TranslationError> {
    let mut stack = vec![path.to_string()];
    let mut result = Vec::new();

    // Use iterative approach to avoid recursion depth limits
    while let Some(current_path) = stack.pop() {
        let directory = read_dir(&current_path)?.collect::<Result<Vec<_>, _>>()?;

        for entry in directory {
            let path = entry.path();
            if path.is_dir() {
                stack.push(
                    path.to_str()
                        .ok_or(TranslationError::InvalidUnicode)?
                        .to_string(),
                );
            } else {
                result.push(path.to_string_lossy().to_string());
            }
        }
    }

    Ok(result)
}

/// Validate TOML structure for translation files
///
/// # Validation Rules
/// 1. Nodes must be either all tables or all translations
/// 2. Translation keys must be valid ISO 639-1 codes
/// 3. Template brackets must be balanced in translation values
fn translations_valid(table: &Table) -> bool {
    let mut contains_translation = false;
    let mut contains_table = false;

    for (key, raw) in table {
        match raw {
            Value::Table(table) => {
                if contains_translation || !translations_valid(table) {
                    return false;
                }
                contains_table = true;
            }
            Value::String(translation) => {
                if contains_table || !Iso639a::is_valid(key) {
                    return false;
                }

                // Check balanced template delimiters
                let balance = translation.chars().fold(0i32, |acc, c| match c {
                    '{' => acc + 1,
                    '}' => acc - 1,
                    _ => acc,
                });
                if balance != 0 {
                    return false;
                }

                contains_translation = true;
            }
            _ => return false,
        }
    }
    true
}

/// Load translations from configured directory with thread-safe caching
///
/// # Returns
/// Reference to loaded translations or TranslationError
fn load_translations() -> Result<&'static Vec<Table>, TranslationError> {
    if let Some(translations) = TRANSLATIONS.get() {
        return Ok(translations);
    }

    let config = load_config()?;
    let mut translation_paths = walk_dir(config.path())?;

    // Sort paths case-insensitively
    translation_paths.sort_by_key(|path| path.to_lowercase());
    if let SeekMode::Unalphabetical = config.seek_mode() {
        translation_paths.reverse();
    }

    let translations = translation_paths
        .iter()
        .map(|path| {
            let content = read_to_string(path)?;
            let table = content
                .parse::<Table>()
                .map_err(|err| TranslationError::ParseToml(err, path.clone()))?;

            if !translations_valid(&table) {
                return Err(TranslationError::InvalidTomlFormat(path.clone()));
            }

            Ok(table)
        })
        .collect::<Result<Vec<_>, TranslationError>>()?;

    Ok(TRANSLATIONS.get_or_init(|| translations))
}

/// Load translation for given language and path
///
/// # Arguments
/// * `lang` - ISO 639-1 language code
/// * `path` - Dot-separated translation path
///
/// # Returns
/// Option<String> with translation or TranslationError
pub fn load_translation_static(lang: &str, path: &str) -> Result<Option<String>, TranslationError> {
    let translations = load_translations()?;
    let config = load_config()?;

    if !Iso639a::is_valid(lang) {
        let lang_lower = lang.to_lowercase();

        let similarities = Iso639a::iter()
            .filter(|lang| format!("{lang:?}").to_lowercase().contains(&lang_lower))
            .map(|lang| format!("{} ({lang:#})", format!("{lang:?}").to_lowercase()))
            .collect::<Vec<_>>();

        return Err(TranslationError::InvalidLanguage(
            lang.to_string(),
            similarities,
        ));
    }

    let mut chosen_translation = None;
    for translation in translations {
        if let Some(value) = path
            .split('.')
            .try_fold(translation, |acc, key| acc.get(key)?.as_table())
            .and_then(|table| table.get(lang))
        {
            chosen_translation = Some(value.to_string());
            if matches!(config.overlap(), TranslationOverlap::Ignore) {
                break;
            }
        }
    }

    Ok(chosen_translation)
}

/// Dynamic translation loading for procedural macros
///
/// # Arguments
/// * `lang` - Language type from macro
/// * `path` - Path type from macro
///
/// # Returns
/// TokenStream for generated code
pub fn load_translation_dynamic(
    lang: TranslationLanguageType,
    path: TranslationPathType,
) -> TokenStream {
    let lang = lang.dynamic();
    let path = path.dynamic();
    todo!("Implement dynamic translation loading")
}

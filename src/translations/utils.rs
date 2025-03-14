use crate::config::{SeekMode, load_config};
use crate::languages::Iso639a;
use std::fs::{read_dir, read_to_string};
use std::sync::OnceLock;
use toml::{Table, Value};
use super::errors::TranslationError;

pub struct AssociatedTranslation {
    pub original_path: String,
    pub translation_table: Table
}

/// Global cache for loaded translations
static TRANSLATIONS: OnceLock<Vec<AssociatedTranslation>> = OnceLock::new();

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
pub fn load_translations() -> Result<&'static Vec<AssociatedTranslation>, TranslationError> {
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

            Ok(AssociatedTranslation {
                original_path: path.to_string(),
                translation_table: table
            })
        })
        .collect::<Result<Vec<_>, TranslationError>>()?;

    Ok(TRANSLATIONS.get_or_init(|| translations))
}

use std::fs::{read_dir, read_to_string};
use std::sync::OnceLock;

use toml::Table;

use translatable_shared::TranslationNode;

use super::config::{SeekMode, TranslationOverlap, load_config};
use crate::translations::errors::CompileTimeError;

/// Translation association with its source file
pub struct AssociatedTranslation {
    /// Original file path of the translation
    original_path: String,
    /// Hierarchical translation data
    translation_table: TranslationNode,
}

/// Global thread-safe cache for loaded translations
static TRANSLATIONS: OnceLock<Vec<AssociatedTranslation>> = OnceLock::new();

/// Recursively walks directory to find all translation files
///
/// # Arguments
/// * `path` - Root directory to scan
///
/// # Returns
/// Vec of file paths or TranslationError
fn walk_dir(path: &str) -> Result<Vec<String>, CompileTimeError> {
    let mut stack = vec![path.to_string()];
    let mut result = Vec::new();

    // Use iterative approach to avoid recursion depth limits
    while let Some(current_path) = stack.pop() {
        let directory = read_dir(&current_path)?.collect::<Result<Vec<_>, _>>()?;

        for entry in directory {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.to_str().ok_or(CompileTimeError::InvalidUnicode)?.to_string());
            } else {
                result.push(path.to_string_lossy().to_string());
            }
        }
    }

    Ok(result)
}

/// Loads and caches translations from configured directory
///
/// # Returns
/// Reference to cached translations or TranslationError
///
/// # Implementation Details
/// - Uses OnceLock for thread-safe initialization
/// - Applies sorting based on configuration
/// - Handles file parsing and validation
pub fn load_translations() -> Result<&'static Vec<AssociatedTranslation>, CompileTimeError> {
    if let Some(translations) = TRANSLATIONS.get() {
        return Ok(translations);
    }

    let config = load_config()?;
    let mut translation_paths = walk_dir(config.path())?;

    // Apply sorting based on configuration
    translation_paths.sort_by_key(|path| path.to_lowercase());
    if let SeekMode::Unalphabetical = config.seek_mode() {
        translation_paths.reverse();
    }

    let mut translations = translation_paths
        .iter()
        .map(|path| {
            let table = read_to_string(path)?
                .parse::<Table>()
                .map_err(|err| CompileTimeError::ParseToml(err, path.clone()))?;

            Ok(AssociatedTranslation {
                original_path: path.to_string(),
                translation_table: TranslationNode::try_from(table)
                    .map_err(|err| CompileTimeError::InvalidTomlFormat(err, path.to_string()))?,
            })
        })
        .collect::<Result<Vec<_>, CompileTimeError>>()?;

    // Handle translation overlap configuration
    if let TranslationOverlap::Overwrite = config.overlap() {
        translations.reverse();
    }

    Ok(TRANSLATIONS.get_or_init(|| translations))
}


impl AssociatedTranslation {
    /// Gets the original file path of the translation
    #[allow(unused)]
    pub fn original_path(&self) -> &str {
        &self.original_path
    }

    /// Gets reference to the translation data structure
    #[allow(unused)]
    pub fn translation_table(&self) -> &TranslationNode {
        &self.translation_table
    }
}

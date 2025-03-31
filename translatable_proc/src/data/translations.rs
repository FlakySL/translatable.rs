use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::sync::OnceLock;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use strum::ParseError;
use syn::LitStr;
use thiserror::Error;
use toml::{Table, Value};

use super::config::{SeekMode, TranslationOverlap, load_config};
use crate::translations::errors::TranslationError;

/// Errors occurring during TOML-to-translation structure transformation
#[derive(Error, Debug)]
pub enum TransformError {
    /// Mixed content found in nesting node (strings and objects cannot coexist)
    #[error("A nesting can contain either strings or other nestings, but not both.")]
    InvalidNesting,

    /// Template syntax error with unbalanced braces
    #[error("Templates in translations should match '{{' and '}}'")]
    UnclosedTemplate,

    /// Invalid value type encountered in translation structure
    #[error("Only strings and objects are allowed for nested objects.")]
    InvalidValue,

    /// Failed to parse language code from translation key
    #[error("Couldn't parse ISO 639-1 string for translation key")]
    LanguageParsing(#[from] ParseError),
}

/// Represents hierarchical translation structure
#[derive(Clone)]
pub enum NestingType {
    /// Nested namespace containing other translation objects
    Object(HashMap<String, NestingType>),
    /// Leaf node containing actual translations per language
    Translation(HashMap<Iso639a, String>),
}

/// Translation association with its source file
pub struct AssociatedTranslation {
    /// Original file path of the translation
    original_path: String,
    /// Hierarchical translation data
    translation_table: NestingType,
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
fn walk_dir(path: &str) -> Result<Vec<String>, TranslationError> {
    let mut stack = vec![path.to_string()];
    let mut result = Vec::new();

    // Use iterative approach to avoid recursion depth limits
    while let Some(current_path) = stack.pop() {
        let directory = read_dir(&current_path)?.collect::<Result<Vec<_>, _>>()?;

        for entry in directory {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.to_str().ok_or(TranslationError::InvalidUnicode)?.to_string());
            } else {
                result.push(path.to_string_lossy().to_string());
            }
        }
    }

    Ok(result)
}

/// Validates template brace balancing in translation strings
fn templates_valid(translation: &str) -> bool {
    let mut nestings = 0;

    for character in translation.chars() {
        match character {
            '{' => nestings += 1,
            '}' => nestings -= 1,
            _ => {},
        }
    }

    nestings == 0
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
pub fn load_translations() -> Result<&'static Vec<AssociatedTranslation>, TranslationError> {
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
                .map_err(|err| TranslationError::ParseToml(err, path.clone()))?;

            Ok(AssociatedTranslation {
                original_path: path.to_string(),
                translation_table: NestingType::try_from(table)
                    .map_err(|err| TranslationError::InvalidTomlFormat(err, path.to_string()))?,
            })
        })
        .collect::<Result<Vec<_>, TranslationError>>()?;

    // Handle translation overlap configuration
    if let TranslationOverlap::Overwrite = config.overlap() {
        translations.reverse();
    }

    Ok(TRANSLATIONS.get_or_init(|| translations))
}

impl NestingType {
    /// Resolves a translation path through the nesting hierarchy
    ///
    /// # Arguments
    /// * `path` - Slice of path segments to resolve
    ///
    /// # Returns
    /// Reference to translations if path exists and points to leaf node
    pub fn get_path(&self, path: Vec<&str>) -> Option<&HashMap<Iso639a, String>> {
        match self {
            Self::Object(nested) => {
                let (first, rest) = path.split_first()?;
                nested.get(*first)?.get_path(rest.to_vec())
            },
            Self::Translation(translation) => path.is_empty().then_some(translation),
        }
    }
}

impl From<NestingType> for TokenStream {
    /// Converts NestingType to procedural macro output tokens
    fn from(val: NestingType) -> Self {
        match val {
            NestingType::Object(nesting) => {
                let entries = nesting.into_iter().map(|(key, value)| -> TokenStream {
                    let key = LitStr::new(&key, Span::call_site());
                    let value: TokenStream = value.into();
                    quote! { (#key.to_string(), #value) }
                });

                quote! {
                    translatable::internal::NestingType::Object(vec![#(#entries),*].into_iter().collect())
                }
            },

            NestingType::Translation(translation) => {
                let entries = translation.into_iter().map(|(lang, value)| {
                    let lang = LitStr::new(&format!("{lang:?}").to_lowercase(), Span::call_site());
                    let value = LitStr::new(&value, Span::call_site());

                    quote! { (#lang.to_string(), #value.to_string()) }
                });

                quote! {
                    translatable::internal::NestingType::Translation(vec![#(#entries),*].into_iter().collect())
                }
            },
        }
    }
}

impl TryFrom<Table> for NestingType {
    type Error = TransformError;

    /// Converts TOML table to validated translation structure
    fn try_from(value: Table) -> Result<Self, Self::Error> {
        let mut result = None;

        for (key, value) in value {
            match value {
                Value::String(translation_value) => {
                    // Initialize result if first entry
                    let result = result.get_or_insert_with(|| Self::Translation(HashMap::new()));

                    match result {
                        Self::Translation(translation) => {
                            if !templates_valid(&translation_value) {
                                return Err(TransformError::UnclosedTemplate);
                            }
                            translation.insert(key.parse()?, translation_value);
                        },
                        Self::Object(_) => return Err(TransformError::InvalidNesting),
                    }
                },

                Value::Table(nesting_value) => {
                    let result = result.get_or_insert_with(|| Self::Object(HashMap::new()));

                    match result {
                        Self::Object(nesting) => {
                            nesting.insert(key, Self::try_from(nesting_value)?);
                        },
                        Self::Translation(_) => return Err(TransformError::InvalidNesting),
                    }
                },

                _ => return Err(TransformError::InvalidValue),
            }
        }

        result.ok_or(TransformError::InvalidValue)
    }
}

impl AssociatedTranslation {
    /// Gets the original file path of the translation
    #[allow(unused)]
    pub fn original_path(&self) -> &str {
        &self.original_path
    }

    /// Gets reference to the translation data structure
    #[allow(unused)]
    pub fn translation_table(&self) -> &NestingType {
        &self.translation_table
    }
}

use super::config::{SeekMode, load_config};
use crate::languages::Iso639a;
use std::clone;
use std::collections::HashMap;
use std::fs::{read_dir, read_to_string};
use std::sync::OnceLock;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use strum::ParseError;
use syn::LitStr;
use thiserror::Error;
use toml::{Table, Value};
use crate::translations::errors::TranslationError;

#[derive(Error, Debug)]
pub enum TransformError {
    #[error("A nesting can contain either strings or other nestings, but not both.")]
    InvalidNesting,

    #[error("Templates in translations should match '{{' and '}}'")]
    UnclosedTemplate,

    #[error("Only strings and objects are allowed for nested objects.")]
    InvalidValue,

    #[error("Couldn't parse ISO 639-1 string for translation key")]
    LanguageParsing(#[from] ParseError)
}

#[derive(Clone)]
pub enum NestingType {
    Object(HashMap<String, NestingType>),
    Translation(HashMap<Iso639a, String>)
}

pub struct AssociatedTranslation {
    original_path: String,
    translation_table: NestingType
}

/// Global cache for loaded translations
static TRANSLATIONS: OnceLock<Vec<AssociatedTranslation>> = OnceLock::new();

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

fn templates_valid(translation: &str) -> bool {
    let mut nestings = 0;

    for character in translation.chars() {
        match character {
            '{' => nestings += 1,
            '}' => nestings -= 1,
            _ => {}
        }
    }

    nestings == 0
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
            let table = read_to_string(path)?
                .parse::<Table>()
                .map_err(|err| TranslationError::ParseToml(err, path.clone()))?;

            Ok(AssociatedTranslation {
                original_path: path.to_string(),
                translation_table: NestingType::try_from(table)
                    .map_err(|err| TranslationError::InvalidTomlFormat(err, path.to_string()))?
            })
        })
        .collect::<Result<Vec<_>, TranslationError>>()?;

    Ok(TRANSLATIONS.get_or_init(|| translations))
}

impl NestingType {
    pub fn get_path(&self, path: Vec<&str>) -> Option<&HashMap<Iso639a, String>> {
        match self {
            Self::Object(nested) => {
                let (first, rest) = path.split_first()?;

                nested
                    .get(*first)
                    .and_then(|n| n.get_path(rest.to_vec()))
            },

            Self::Translation(translation) => {
                if path.is_empty() {
                    return Some(translation)
                }

                None
            }
        }
    }
}

impl Into<TokenStream> for NestingType {
    fn into(self) -> TokenStream {
        match self {
            Self::Object(nesting) => {
                let entries = nesting
                    .into_iter()
                    .map(|(key, value)| -> TokenStream {
                        let key = LitStr::new(&key, Span::call_site());
                        let value: TokenStream = value.into();
                        quote! { (#key.to_string(), #value) }
                    });

                quote! {
                    translatable::internal::NestingType::Object(vec![#(#entries),*].into_iter().collect())
                }
            },

            NestingType::Translation(translation) => {
                let entries = translation
                    .into_iter()
                    .map(|(lang, value)| {
                        let lang = LitStr::new(&lang.to_string(), Span::call_site());
                        let value = LitStr::new(&value, Span::call_site());

                        quote! { (#lang.to_string(), #value.to_string()) }
                    });

                quote! {
                    translatable::internal::NestingType::Translation(vec![#(#entries),*].into_iter().collect())
                }
            }
        }
    }
}

impl TryFrom<Table> for NestingType {
    type Error = TransformError;

    fn try_from(value: Table) -> Result<Self, Self::Error> {
        let mut result = None;

        for (key, value) in value {
            match value {
                Value::String(translation_value) => {
                    if result.is_none() {
                        result = Some(Self::Translation(HashMap::new()));
                    }

                    if !templates_valid(&translation_value) {
                        return Err(TransformError::UnclosedTemplate);
                    }

                    match result {
                        Some(Self::Translation(ref mut translation)) => {
                            translation.insert(key.parse()?, translation_value);
                        },

                        Some(Self::Object(_)) => {
                            return Err(TransformError::InvalidNesting);
                        },

                        None => unreachable!()
                    }
                },

                Value::Table(nesting_value) => {
                    if result.is_none() {
                        result = Some(Self::Object(HashMap::new()));
                    }

                    match result {
                        Some(Self::Object(ref mut nesting)) => {
                            nesting.insert(key, Self::try_from(nesting_value)?);
                        },

                        Some(Self::Translation(_)) => {
                            return Err(TransformError::InvalidNesting);
                        },

                        None => unreachable!()
                    }
                },

                _ => {
                    return Err(TransformError::InvalidValue)
                }
            }
        }

        match result {
            Some(result) => Ok(result),
            None => unreachable!()
        }
    }
}

impl AssociatedTranslation {
    #[allow(unused)]
    pub fn original_path(&self) -> &str {
        &self.original_path
    }

    #[allow(unused)]
    pub fn translation_table(&self) -> &NestingType {
        &self.translation_table
    }
}

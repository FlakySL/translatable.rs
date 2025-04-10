use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, TokenStreamExt, quote};
use strum::ParseError;
use thiserror::Error;
use toml::{Table, Value};

use crate::Language;

/// Errors occurring during TOML-to-translation structure transformation
#[derive(Error, Debug)]
pub enum TranslationNodeError {
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

pub type TranslationNesting = HashMap<String, TranslationNode>;
pub type TranslationObject = HashMap<Language, String>;

/// Represents nested translation structure,
/// as it is on the translation files.
#[derive(Clone)]
pub enum TranslationNode {
    /// Nested namespace containing other translation objects
    Nesting(TranslationNesting),
    /// Leaf node containing actual translations per language
    Translation(TranslationObject),
}

impl TranslationNode {
    /// Resolves a translation path through the nesting hierarchy
    ///
    /// # Arguments
    /// * `path` - Slice of path segments to resolve
    ///
    /// # Returns
    /// Reference to translations if path exists and points to leaf node
    pub fn find_path<I: ToString>(&self, path: &Vec<I>) -> Option<&TranslationObject> {
        let path = path.iter().map(|i| i.to_string()).collect::<Vec<String>>();

        match self {
            Self::Nesting(nested) => {
                let (first, rest) = path.split_first()?;
                nested.get(first)?.find_path(&rest.to_vec())
            },
            Self::Translation(translation) => path.is_empty().then_some(translation),
        }
    }
}

/// This implementation converts the tagged union
/// to an equivalent call from the runtime context.
///
/// This is exclusively meant to be used from the
/// macro generation context.
impl ToTokens for TranslationNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            TranslationNode::Nesting(nesting) => {
                let mapped_nesting = nesting
                    .into_iter()
                    .map(|(key, value)| {
                        let value = value.to_token_stream();
                        quote! { (#key.to_string(), #value) }
                    })
                    .collect::<Vec<_>>();

                tokens.append_all(quote! {{
                    translatable::shared::TranslationNode::Nesting(
                        vec![#(#mapped_nesting),*]
                            .into_iter()
                            .collect::<std::collections::HashMap<_, _>>()
                    )
                }});
            },

            TranslationNode::Translation(translation) => {
                let mapped_translation = translation
                    .into_iter()
                    .map(|(key, value)| {
                        let key = key.into_token_stream();
                        quote! { (#key, #value.to_string()) }
                    })
                    .collect::<Vec<_>>();

                tokens.append_all(quote! {{
                    translatable::shared::TranslationNode::Translation(
                        vec![#(#mapped_translation),*]
                            .into_iter()
                            .collect::<std::collections::HashMap<_, _>>()
                    )
                }});
            },
        }
    }
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

/// This implementation converts a `toml::Table` into a manageable
/// NestingType.
impl TryFrom<Table> for TranslationNode {
    type Error = TranslationNodeError;

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
                                return Err(TranslationNodeError::UnclosedTemplate);
                            }
                            translation.insert(key.parse()?, translation_value);
                        },

                        Self::Nesting(_) => return Err(TranslationNodeError::InvalidNesting),
                    }
                },

                Value::Table(nesting_value) => {
                    let result = result.get_or_insert_with(|| Self::Nesting(HashMap::new()));

                    match result {
                        Self::Nesting(nesting) => {
                            nesting.insert(key, Self::try_from(nesting_value)?);
                        },
                        Self::Translation(_) => return Err(TranslationNodeError::InvalidNesting),
                    }
                },

                _ => return Err(TranslationNodeError::InvalidValue),
            }
        }

        result.ok_or(TranslationNodeError::InvalidValue)
    }
}

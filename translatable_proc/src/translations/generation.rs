use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use syn::{Expr, parse2};

use super::errors::TranslationError;
use crate::data::translations::load_translations;
use crate::languages::Iso639a;

/// Generates compile-time string replacement logic for a single format
/// argument.
///
/// Implements a three-step replacement strategy to safely handle nested
/// templates:
/// 1. Temporarily replace `{{key}}` with `\x01{key}\x01` to protect wrapper
///    braces
/// 2. Replace `{key}` with the provided value
/// 3. Restore original `{key}` syntax from temporary markers
///
/// # Arguments
/// * `key` - Template placeholder name (without braces)
/// * `value` - Expression to substitute, must implement `std::fmt::Display`
///
/// # Example
/// For key = "name" and value = `user.first_name`:
/// ```rust
/// let template = "{{name}} is a user";
///
/// template
///     .replace("{{name}}", "\x01{name}\x01")
///     .replace("{name}", &format!("{:#}", "Juan"))
///     .replace("\x01{name}\x01", "{name}");
/// ```
fn kwarg_static_replaces(key: &str, value: &TokenStream) -> TokenStream {
    quote! {
        .replace(
            format!("{{{{{}}}}}", #key).as_str(), // Replace {{key}} -> a temporary placeholder
            format!("\x01{{{}}}\x01", #key).as_str()
        )
        .replace(
            format!("{{{}}}", #key).as_str(), // Replace {key} -> value
            format!("{:#}", #value).as_str()
        )
        .replace(
            format!("\x01{{{}}}\x01", #key).as_str(), // Restore {key} from the placeholder
            format!("{{{}}}", #key).as_str()
        )
    }
}

/// Generates runtime-safe template substitution chain for multiple format
/// arguments.
///
/// Creates an iterator of chained replacement operations that will be applied
/// sequentially at runtime while preserving nested template syntax.
///
/// # Arguments
/// * `format_kwargs` - Key/value pairs where:
///   - Key: Template placeholder name
///   - Value: Runtime expression implementing `Display`
///
/// # Note
/// The replacement order is important to prevent accidental substitution in
/// nested templates. All replacements are wrapped in `Option::map` to handle
/// potential `None` values from translation lookup.
fn kwarg_dynamic_replaces(format_kwargs: &HashMap<String, TokenStream>) -> Vec<TokenStream> {
    format_kwargs
        .iter()
        .map(|(key, value)| {
            let static_replaces = kwarg_static_replaces(key, value);
            quote! {
                .map(|translation| translation
                    #static_replaces
                )
            }
        })
        .collect::<Vec<_>>()
}

/// Parses a static language string into an Iso639a enum instance with
/// compile-time validation.
///
/// # Arguments
/// * `lang` - A string slice representing the language code to parse
///
/// # Returns
/// - `Ok(Iso639a)` if valid language code
/// - `Err(TranslationError)` if parsing fails
pub fn load_lang_static(lang: &str) -> Result<Iso639a, TranslationError> {
    lang.parse::<Iso639a>().map_err(|_| TranslationError::InvalidLanguage(lang.to_string()))
}

/// Generates runtime validation for a dynamic language expression.
///
/// # Arguments
/// * `lang` - TokenStream representing an expression that implements
///   `Into<String>`
///
/// # Returns
/// TokenStream with code to validate language at runtime
pub fn load_lang_dynamic(lang: TokenStream) -> Result<TokenStream, TranslationError> {
    let lang: Expr = parse2(lang)?;

    // Generate list of available language codes
    let available_langs = Iso639a::iter().map(|language| {
        let language = format!("{language:?}");

        quote! { #language, }
    });

    // The `String` explicit type serves as
    // expression type checking, we accept `impl Into<String>`
    // for any expression that's not static.
    Ok(quote! {
        #[doc(hidden)]
        let language: String = (#lang).into();
        #[doc(hidden)]
        let language = language.to_lowercase();

        #[doc(hidden)]
        let valid_lang = vec![#(#available_langs)*]
            .iter()
            .any(|lang| lang.eq_ignore_ascii_case(&language));
    })
}

/// Loads translations for static language resolution
///
/// # Arguments
/// * `static_lang` - Optional predefined language
/// * `path` - Translation key path as dot-separated string
///
/// # Returns
/// TokenStream with either direct translation or language lookup logic
pub fn load_translation_static(
    static_lang: Option<Iso639a>,
    path: String,
    format_kwargs: HashMap<String, TokenStream>,
) -> Result<TokenStream, TranslationError> {
    let translation_object = load_translations()?
        .iter()
        .find_map(|association| association.translation_table().get_path(path.split('.').collect()))
        .ok_or(TranslationError::PathNotFound(path.to_string()))?;
    let replaces = kwarg_dynamic_replaces(&format_kwargs);

    Ok(match static_lang {
        Some(language) => {
            let translation = translation_object
                .get(&language)
                .ok_or(TranslationError::LanguageNotAvailable(language, path))?;

            let static_replaces = format_kwargs
                .iter()
                .map(|(key, value)| kwarg_static_replaces(key, value))
                .collect::<Vec<_>>();

            quote! {{
                #translation
                #(#static_replaces)*
            }}
        },

        None => {
            let translation_object = translation_object.iter().map(|(key, value)| {
                let key = format!("{key:?}").to_lowercase();
                quote! { (#key, #value) }
            });

            quote! {{
                if valid_lang {
                    vec![#(#translation_object),*]
                        .into_iter()
                        .collect::<std::collections::HashMap<_, _>>()
                        .get(language.as_str())
                        .ok_or(translatable::Error::LanguageNotAvailable(language, #path.to_string()))
                        .cloned()
                        .map(|translation| translation.to_string())
                        #(#replaces)*
                } else {
                    Err(translatable::Error::InvalidLanguage(language))
                }
            }}
        },
    })
}

/// Loads translations for dynamic language and path resolution
///
/// # Arguments
/// * `static_lang` - Optional predefined language
/// * `path` - TokenStream representing dynamic path expression
///
/// # Returns
/// TokenStream with runtime translation resolution logic
pub fn load_translation_dynamic(
    static_lang: Option<Iso639a>,
    path: TokenStream,
    format_kwargs: HashMap<String, TokenStream>,
) -> Result<TokenStream, TranslationError> {
    let nestings = load_translations()?
        .iter()
        .map(|association| association.translation_table().clone().into())
        .collect::<Vec<TokenStream>>();

    let translation_quote = quote! {
        #[doc(hidden)]
        let path: String = #path.into();

        #[doc(hidden)]
        let nested_translations = vec![#(#nestings),*];

        #[doc(hidden)]
        let translation = nested_translations
            .iter()
            .find_map(|nesting| nesting.get_path(
                path
                    .split('.')
                    .collect()
            ));
    };

    let replaces = kwarg_dynamic_replaces(&format_kwargs);

    Ok(match static_lang {
        Some(language) => {
            let language = format!("{language:?}").to_lowercase();

            quote! {{
                #translation_quote

                if let Some(translation) = translation {
                    translation
                        .get(#language)
                        .ok_or(translatable::Error::LanguageNotAvailable(#language.to_string(), path))
                        .cloned()
                        #(#replaces)*
                } else {
                    Err(translatable::Error::PathNotFound(path))
                }
            }}
        },

        None => {
            quote! {{
                #translation_quote

                if valid_lang {
                    if let Some(translation) = translation {
                        translation
                            .get(&language)
                            .ok_or(translatable::Error::LanguageNotAvailable(language, path))
                            .cloned()
                            #(#replaces)*
                    } else {
                        Err(translatable::Error::PathNotFound(path))
                    }
                } else {
                    Err(translatable::Error::InvalidLanguage(language))
                }
            }}
        },
    })
}

use super::errors::TranslationError;
use crate::data::translations::load_translations;
use crate::languages::Iso639a;
use proc_macro2::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use syn::{Expr, parse2};

/// This function parses a statically obtained language
/// as an Iso639a enum instance, along this, the validation
/// is also done at parse time.
pub fn load_lang_static(lang: &str) -> Result<Iso639a, TranslationError> {
    lang.parse::<Iso639a>()
        .map_err(|_| TranslationError::InvalidLanguage(lang.to_string()))
}

/// This function generates a language variable, the only
/// requisite is that the expression evaluates to something
/// that implements Into<String>.
pub fn load_lang_dynamic(lang: TokenStream) -> Result<TokenStream, TranslationError> {
    let lang: Expr = parse2(lang)?;

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

pub fn load_translation_static(
    static_lang: Option<Iso639a>,
    path: String,
) -> Result<TokenStream, TranslationError> {
    let translation_object = load_translations()?
        .iter()
        .find_map(|association| {
            association
                .translation_table()
                .get_path(path.split(".").collect())
        })
        .ok_or(TranslationError::PathNotFound(path.to_string()))?;

    Ok(match static_lang {
        Some(language) => {
            let translation = translation_object
                .get(&language)
                .ok_or(TranslationError::LanguageNotAvailable(language, path))?;

            quote! { #translation }
        }

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
                } else {
                    Err(translatable::Error::InvalidLanguage(language))
                }
            }}
        }
    })
}

pub fn load_translation_dynamic(
    static_lang: Option<Iso639a>,
    path: TokenStream,
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
            .find_map(|nesting| nesting
                .get_path(
                    path
                        .split(".")
                        .collect()
                )
            );
    };

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
                } else {
                    Err(translatable::Error::PathNotFound(path))
                }
            }}
        }

        None => {
            quote! {{
                #translation_quote

                if valid_lang {
                    if let Some(translation) = translation {
                        translation
                            .get(&language)
                            .ok_or(translatable::Error::LanguageNotAvailable(language, path))
                            .cloned()
                    } else {
                        Err(translatable::Error::PathNotFound(path))
                    }
                } else {
                    Err(translatable::Error::InvalidLanguage(language))
                }
            }}
        }
    })
}

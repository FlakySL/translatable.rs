use std::collections::HashMap;
use crate::languages::Iso639a;
use proc_macro::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use syn::{parse_macro_input, Expr};
use super::{errors::TranslationError, utils::{load_translations, AssociatedTranslation}};

/// This function parses a statically obtained language
/// as an Iso639a enum instance, along this, the validation
/// is also done at parse time.
pub fn load_lang_static(lang: &str) -> Result<Iso639a, TranslationError> {
    Ok(
        lang
            .parse::<Iso639a>()
            .map_err(|_| TranslationError::InvalidLanguage(
                lang.to_string(),
                Iso639a::get_similarities(lang)
            ))?
    )

}

/// This function generates a language variable, the only
/// requisite is that the expression evalutes to something
/// that implements Into<String>.
pub fn load_lang_dynamic(lang: TokenStream) -> TokenStream {
    let lang = parse_macro_input!(lang as Expr);

    let available_langs = Iso639a::iter()
        .map(|language| {
            let language = format!("{language:?}");

            quote! { stringify!(#language), }
        });

    // The `String` explicit type serves as
    // expression type checking, we accept `impl Into<String>`
    // for any expression that's not static.
    quote! {
        let language: String = (#lang).into();

        let valid_lang = vec![#(#available_langs)*]
            .iter()
            .any(|lang| lang.eq_ignore_ascii_case(&language));
    }
        .into()
}


pub fn load_translation_static(static_lang: Option<Iso639a>, path: String) -> Result<TokenStream, TranslationError> {
    let mut found_association = None;

    for AssociatedTranslation { translation_table, original_path } in load_translations()? {
        let translation_table = path
            .split(".")
            .try_fold(translation_table, |acc, curr| {
                acc
                    .get(curr)
                    .map(|new| new.as_table())
                    .flatten()
            });

        if let Some(translation_table) = translation_table.cloned() {
            found_association = Some(AssociatedTranslation {
                translation_table,
                original_path: original_path.clone()
            })
        }
    }

    let found_association = found_association
        .ok_or(TranslationError::PathNotFound(path.clone()))?;

    let translation_table = found_association
        .translation_table
        .into_iter()
        .map(|(language, translation)| Some((
            language.to_string(),
            translation.as_str()?.to_string()
        )))
        .collect::<Option<HashMap<String, String>>>()
        .ok_or(TranslationError::InvalidTomlFormat(
            found_association.original_path
        ))?;

    Ok(
        if let Some(static_lang) = static_lang {
            let translation = translation_table
                .iter()
                .find(|(language, _)| static_lang.eq_insensitive(language))
                .map(|(_, translation)| translation)
                .ok_or(TranslationError::LanguageNotAvailable(static_lang, path))?;

            quote! { stringify!(#translation) }
        } else {
            let translation_variants = translation_table
                .iter()
                .map(|(language, translation)| quote! {
                    stringify!(#language) => Ok(stringify!(#translation)),
                });

            quote! {{
                if valid_lang {
                    match language {
                        #(#translation_variants)*,
                        _ => Err(format!(
                            "A translation with the language '{}' was not found for the path '{}'",
                            language,
                            stringify!(#path)
                        ))
                    }
                } else {
                    Err(format!("The language '{}' is not valid ISO 639-1."))
                }
            }}
        }
            .into()
    )
}

pub fn load_translation_dynamic(static_lang: Option<Iso639a>, path: TokenStream) -> Result<TokenStream, TranslationError> {
    let translations = load_translations()?;

    Ok(quote!("").into())
}


use crate::languages::Iso639a;
use proc_macro2::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use syn::{parse2, Expr};
use crate::data::translations::load_translations;
use super::errors::TranslationError;

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
pub fn load_lang_dynamic(lang: TokenStream) -> Result<TokenStream, TranslationError> {
    let lang: Expr = parse2(lang)?;

    let available_langs = Iso639a::iter()
        .map(|language| {
            let language = format!("{language:?}");

            quote! { stringify!(#language), }
        });

    // The `String` explicit type serves as
    // expression type checking, we accept `impl Into<String>`
    // for any expression that's not static.
    Ok(
        quote! {
            let language: String = (#lang).into();
            let language = language.to_lowercase();

            let valid_lang = vec![#(#available_langs)*]
                .iter()
                .any(|lang| lang.eq_ignore_ascii_case(&language));
        }
    )
}


pub fn load_translation_static(static_lang: Option<Iso639a>, path: String) -> Result<TokenStream, TranslationError> {
    let translation_object = load_translations()?
        .iter()
        .find_map(|association| association
            .translation_table()
            .get_path(
                path
                    .split(".")
                    .collect()
            )
        )
        .ok_or(TranslationError::PathNotFound(path.to_string()))?;

    Ok(
        match static_lang {
            Some(language) => {
                let translation = translation_object
                    .get(&language)
                    .ok_or(TranslationError::LanguageNotAvailable(language, path))?;

                quote! { stringify!(#translation) }
            },

            None => {
                let translation_object = translation_object
                    .iter()
                    .map(|(key, value)| {
                        let key = format!("{key:?}").to_lowercase();
                        quote! { (stringify!(#key), stringify!(#value)) }
                    });

                quote! {
                    if valid_lang {
                        vec![#(#translation_object),*]
                            .iter()
                            .collect::<std::collections::HashMap<_, _>>()
                            .get(&language)
                            .ok_or(translatable::Error::LanguageNotAvailable(language, stringify!(#path).to_string()))
                    } else {
                        Err(translatable::Error::InvalidLanguage(language))
                    }
                }
            }
        }
    )
}

pub fn load_translation_dynamic(static_lang: Option<Iso639a>, path: TokenStream) -> Result<TokenStream, TranslationError> {
    let translation = load_translations()?
        .into_iter()
        .map(|association| association
            .translation_table()
            .clone()
            .into()
        )
        .collect::<Vec<TokenStream>>();

    Ok(quote! {})
}


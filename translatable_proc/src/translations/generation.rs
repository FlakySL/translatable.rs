use std::collections::HashMap;
use crate::languages::Iso639a;
use proc_macro::TokenStream;
use quote::quote;
use strum::IntoEnumIterator;
use syn::{parse_macro_input, Expr};
use crate::data::translations::{load_translations, AssociatedTranslation};
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
    todo!()
}

pub fn load_translation_dynamic(static_lang: Option<Iso639a>, path: TokenStream) -> Result<TokenStream, TranslationError> {
    todo!()
}


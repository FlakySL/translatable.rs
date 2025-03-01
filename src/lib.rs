use macros::{translation_macro, RawTranslationArgs};
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod config;
mod macros;
mod translations;

#[proc_macro]
pub fn translation(input: TokenStream) -> TokenStream {
    translation_macro(
        parse_macro_input!(input as RawTranslationArgs)
            .into()
    )
}

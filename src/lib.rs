//! Internationalization library providing compile-time and runtime translation facilities
//!
//! # Features
//! - TOML-based translation files
//! - ISO 639-1 language validation
//! - Configurable loading strategies
//! - Procedural macro for compile-time checking

use macros::{RawTranslationArgs, translation_macro};
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod config;
mod languages;
mod macros;
mod translations;

/// Procedural macro for compile-time translation validation
///
/// # Usage
/// ```
/// translation!("en", static some.path)
/// ```
///
/// # Parameters
/// - Language code/literal
/// - Translation path (supports static analysis)
#[proc_macro]
pub fn translation(input: TokenStream) -> TokenStream {
    translation_macro(parse_macro_input!(input as RawTranslationArgs).into())
}

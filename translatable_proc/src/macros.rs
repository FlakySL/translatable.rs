use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Static;
use syn::{Expr, ExprLit, ExprPath, Lit, Result as SynResult, Token};

use crate::translations::generation::{
    load_lang_dynamic, load_lang_static, load_translation_dynamic, load_translation_static,
};

/// Represents raw input arguments for the translation macro
///
/// Parses input in the format: `(language_spec, static translation_path)`
///
/// # Syntax
/// - `language_spec`: String literal or expression implementing `Into<String>`
/// - `translation_path`: Path expression (either static or dynamic)
pub struct RawMacroArgs {
    /// Language specification (either literal string or expression)
    language: Expr,
    /// Comma separator between arguments
    _comma: Token![,],
    /// Optional `static` keyword marker for path resolution
    static_marker: Option<Static>,
    /// Translation path (either static path or dynamic expression)
    path: Expr,
}

/// Represents the type of translation path resolution
pub enum PathType {
    /// Runtime-resolved path expression
    OnScopeExpression(TokenStream),
    /// Compile-time resolved path string
    CompileTimePath(String),
}

/// Represents the type of language specification
pub enum LanguageType {
    /// Runtime-resolved language expression
    OnScopeExpression(TokenStream),
    /// Compile-time validated language literal
    CompileTimeLiteral(String),
}

/// Processed translation arguments ready for code generation
pub struct TranslationArgs {
    /// Language resolution type
    language: LanguageType,
    /// Path resolution type
    path: PathType,
}

impl Parse for RawMacroArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(RawMacroArgs {
            language: input.parse()?,
            _comma: input.parse()?,
            static_marker: input.parse()?,
            path: input.parse()?,
        })
    }
}

impl From<RawMacroArgs> for TranslationArgs {
    fn from(val: RawMacroArgs) -> Self {
        let is_path_static = val.static_marker.is_some();

        TranslationArgs {
            // Extract language specification
            language: match val.language {
                Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => {
                    LanguageType::CompileTimeLiteral(lit_str.value())
                },
                // Preserve other expressions for runtime resolution
                other => LanguageType::OnScopeExpression(quote!(#other)),
            },

            // Extract path specification
            path: match val.path {
                // Convert path expressions to strings when static marker present
                Expr::Path(ExprPath { path, .. }) if is_path_static => {
                    // Convert path segments to a dot-separated string
                    let path_str = path.segments.iter().map(|s| s.ident.to_string()).fold(
                        String::new(),
                        |mut acc, s| {
                            if !acc.is_empty() {
                                acc.push('.');
                            }
                            acc.push_str(&s);
                            acc
                        },
                    );
                    PathType::CompileTimePath(path_str)
                },

                // Preserve dynamic path expressions
                path => PathType::OnScopeExpression(quote!(#path)),
            },
        }
    }
}

/// Generates translation code based on processed arguments
///
/// # Arguments
/// - `args`: Processed translation arguments
///
/// # Returns
/// TokenStream with either:
/// - Compiled translation string
/// - Runtime translation resolution logic
/// - Compile errors for invalid inputs
pub fn translation_macro(args: TranslationArgs) -> TokenStream {
    let TranslationArgs { language, path } = args;

    // Process language specification
    let (lang_expr, static_lang) = match language {
        LanguageType::CompileTimeLiteral(lang) => (
            None,
            match load_lang_static(&lang) {
                Ok(lang) => Some(lang),
                Err(e) => return error_token(&e),
            },
        ),
        LanguageType::OnScopeExpression(lang) => {
            (Some(load_lang_dynamic(lang).map_err(|e| error_token(&e))), None)
        },
    };

    // Process translation path
    let translation_expr = match path {
        PathType::CompileTimePath(p) => load_translation_static(static_lang, p),
        PathType::OnScopeExpression(p) => load_translation_dynamic(static_lang, p),
    };

    match (lang_expr, translation_expr) {
        (Some(Ok(lang)), Ok(trans)) => quote! {{ #lang #trans }},
        (Some(Err(e)), _) => e,
        (None, Ok(trans)) => trans,
        (_, Err(e)) => error_token(&e),
    }
}

/// Helper function to create compile error tokens
fn error_token(e: &impl std::fmt::Display) -> TokenStream {
    let msg = format!("{e:#}");
    quote! { compile_error!(#msg) }
}

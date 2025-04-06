use std::collections::HashMap;
use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Static;
use syn::{
    Expr,
    ExprLit,
    ExprPath,
    Ident,
    Lit,
    MetaNameValue,
    Path,
    Result as SynResult,
    Token,
    parse_quote,
};

use crate::macro_generation::single::{
    load_lang_dynamic,
    load_lang_static,
    load_translation_dynamic,
    load_translation_static,
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
    /// Optional comma separator for additional arguments
    _comma2: Option<Token![,]>,
    /// Format arguments for string interpolation
    format_kwargs: Punctuated<MetaNameValue, Token![,]>,
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
    /// Format arguments for string interpolation
    format_kwargs: HashMap<String, TokenStream>,
}

impl Parse for RawMacroArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let language = input.parse()?;
        let _comma = input.parse()?;
        let static_marker = input.parse()?;
        let path = input.parse()?;

        // Parse optional comma before format arguments
        let _comma2 = if input.peek(Token![,]) { Some(input.parse()?) } else { None };

        let mut format_kwargs = Punctuated::new();

        // Parse format arguments if comma was present
        if _comma2.is_some() {
            while !input.is_empty() {
                let lookahead = input.lookahead1();

                // Handle both identifier-based and arbitrary key-value pairs
                if lookahead.peek(Ident) {
                    let key: Ident = input.parse()?;
                    let eq_token: Token![=] = input.parse().unwrap_or(Token![=](key.span()));
                    let mut value = input.parse::<Expr>();

                    if let Ok(value) = &mut value {
                        let key_string = key.to_string();
                        if key_string == value.to_token_stream().to_string() {
                            // let warning = format!(
                            //     "redundant field initialier, use
                            //  `{key_string}` instead of `{key_string} = {key_string}`"
                            // );

                            // Generate warning for redundant initializer
                            *value = parse_quote! {{
                                // compile_warn!(#warning);
                                // !!! https://internals.rust-lang.org/t/pre-rfc-add-compile-warning-macro/9370 !!!
                                #value
                            }}
                        }
                    }

                    let value = value.unwrap_or(parse_quote!(#key));

                    format_kwargs.push(MetaNameValue { path: Path::from(key), eq_token, value });
                } else {
                    format_kwargs.push(input.parse()?);
                }

                // Continue parsing while commas are present
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }
        };

        Ok(RawMacroArgs {
            language,
            _comma,
            static_marker,
            path,
            _comma2,
            format_kwargs,
        })
    }
}

impl From<RawMacroArgs> for TranslationArgs {
    fn from(val: RawMacroArgs) -> Self {
        let is_path_static = val.static_marker.is_some();

        TranslationArgs {
            // Extract language specification
            language: match val.language {
                // Handle string literals for compile-time validation
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

            // Convert format arguments to HashMap with string keys
            format_kwargs: val
                .format_kwargs
                .iter()
                .map(|pair| {
                    (
                        // Extract key as identifier or stringified path
                        pair.path
                            .get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_else(|| pair.path.to_token_stream().to_string()),
                        // Store value as token stream
                        pair.value.to_token_stream(),
                    )
                })
                .collect(),
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
    let TranslationArgs { language, path, format_kwargs } = args;

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
        PathType::CompileTimePath(p) => load_translation_static(static_lang, p, format_kwargs),
        PathType::OnScopeExpression(p) => load_translation_dynamic(static_lang, p, format_kwargs),
    };

    match (lang_expr, translation_expr) {
        (Some(Ok(lang)), Ok(trans)) => quote! {{ #lang #trans }},
        (Some(Err(e)), _) => e,
        (None, Ok(trans)) => trans,
        (_, Err(e)) => error_token(&e),
    }
}

/// Helper function to create compile error tokens
fn error_token(e: &impl Display) -> TokenStream {
    let msg = format!("{e:#}");
    quote! { compile_error!(#msg) }
}

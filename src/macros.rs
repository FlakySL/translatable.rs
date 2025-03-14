use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Static;
use syn::{Expr, ExprLit, ExprPath, Lit, Result as SynResult, Token};

use crate::translations::generation::load_translation_static;

/// Internal representation of macro arguments before processing
///
/// Parses input in the format:
/// `(language_expression, static translation_path)`
pub struct RawTranslationArgs {
    /// Language specification (literal or expression)
    language: Expr,
    /// Argument seprator.
    _comma: Token![,],
    /// Static marker for path analysis
    static_marker: Option<Static>,
    /// Translation path specification
    path: Expr,
}

/// A TranslationPathType is a wrapper to the path
/// argument, this provides feedback on how to
/// interact with the user provided path.
pub enum TranslationPathType {
    /// An OnScopeExpression represents
    /// any expresion that evaluates to
    /// a string, that expression is dynamic
    /// and it's evaluated on runtime, so
    /// the means to generate checks and errors
    /// are limited.
    OnScopeExpression(TokenStream),

    /// A CompileTimePath represents a path
    /// that's prefixed with the `static`
    /// keyword, if the passed expression is
    /// this one we read the translations
    /// directly, if a translation for that
    /// path does not exist, a compile time
    /// error is generated.
    CompileTimePath(String),
}

pub enum TranslationLanguageType {
    OnScopeExpression(TokenStream),
    CompileTimeLiteral(String),
}

pub struct TranslationArgs {
    language: TranslationLanguageType,
    path: TranslationPathType,
}

impl Parse for RawTranslationArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(RawTranslationArgs {
            language: input.parse()?,
            _comma: input.parse()?,
            static_marker: input.parse()?,
            path: input.parse()?,
        })
    }
}

impl TranslationPathType {
    pub fn dynamic(self) -> TokenStream {
        match self {
            Self::OnScopeExpression(tokens) => tokens,
            Self::CompileTimePath(cmp_val) => quote!(#cmp_val).into(),
        }
    }
}

impl TranslationLanguageType {
    pub fn dynamic(self) -> TokenStream {
        match self {
            Self::OnScopeExpression(tokens) => tokens,
            Self::CompileTimeLiteral(cmp_val) => quote!(#cmp_val).into(),
        }
    }
}

impl Into<TranslationArgs> for RawTranslationArgs {
    fn into(self) -> TranslationArgs {
        let is_path_static = self.static_marker.is_some();

        TranslationArgs {
            language: match self.language {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) => TranslationLanguageType::CompileTimeLiteral(lit_str.value()),
                other => TranslationLanguageType::OnScopeExpression(quote!(#other).into()),
            },

            path: match self.path {
                Expr::Path(ExprPath { path, .. }) if is_path_static => {
                    TranslationPathType::CompileTimePath(
                        path.segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            .join(".")
                            .to_string(),
                    )
                }

                path => TranslationPathType::OnScopeExpression(quote!(#path).into()),
            },
        }
    }
}

pub fn translation_macro(_args: TranslationArgs) -> TokenStream {
    if let TranslationPathType::CompileTimePath(path) = _args.path {
        load_translation_static(None, path);
    }

    quote!("").into()
}

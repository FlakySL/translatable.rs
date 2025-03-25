use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Static;
use syn::{Expr, ExprLit, ExprPath, Lit, Result as SynResult, Token};
use crate::translations::generation::{load_lang_dynamic, load_lang_static, load_translation_dynamic, load_translation_static};

/// Internal representation of macro arguments before processing
///
/// Parses input in the format:
/// `(language_expression, static translation_path)`
pub struct RawMacroArgs {
    /// Language specification (literal or expression)
    language: Expr,
    /// Argument seprator.
    _comma: Token![,],
    /// Static marker for path analysis
    static_marker: Option<Static>,
    /// Translation path specification
    path: Expr,
}

pub enum PathType {
    OnScopeExpression(TokenStream),
    CompileTimePath(String),
}

pub enum LanguageType {
    OnScopeExpression(TokenStream),
    CompileTimeLiteral(String),
}

pub struct TranslationArgs {
    language: LanguageType,
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

impl Into<TranslationArgs> for RawMacroArgs {
    fn into(self) -> TranslationArgs {
        let is_path_static = self.static_marker.is_some();

        TranslationArgs {
            language: match self.language {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) => LanguageType::CompileTimeLiteral(lit_str.value()),
                other => LanguageType::OnScopeExpression(quote!(#other).into()),
            },

            path: match self.path {
                Expr::Path(ExprPath { path, .. }) if is_path_static => {
                    PathType::CompileTimePath(
                        path.segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            .join(".")
                            .to_string(),
                    )
                }

                path => PathType::OnScopeExpression(quote!(#path).into()),
            },
        }
    }
}

pub fn translation_macro(args: TranslationArgs) -> TokenStream {
    let TranslationArgs { language, path } = args;

    let (lang_expr, static_lang) = match language {
        LanguageType::CompileTimeLiteral(lang) => (
            None,
            match load_lang_static(&lang) {
                Ok(lang) => Some(lang),
                Err(e) => {
                    let e = format!("{e:#}");
                    return quote! { compile_error!(#e) }
                }
            },
        ),
        LanguageType::OnScopeExpression(lang) => (Some(load_lang_dynamic(lang)), None),
    };

    let translation_expr = match path {
        PathType::CompileTimePath(p) => load_translation_static(static_lang, p),
        PathType::OnScopeExpression(p) => load_translation_dynamic(static_lang, p),
    };

    match (lang_expr, translation_expr) {
        (Some(lang), Ok(trans)) => {
            match lang {
                Ok(lang) => {
                    quote! {{
                        #lang
                        #trans
                    }}
                },

                Err(e) => {
                    let e = format!("{e:#}");
                    quote! { compile_error!{#e} }
                }
            }
        },
        (None, Ok(trans)) => trans,
        (_, Err(e)) => {
            let e = format!("{e:#}");
            quote! { compile_error!(#e) }
        },
    }
        .into()
}

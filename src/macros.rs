use crate::{languages::Iso639a, translations::load_translation_static};
use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
use syn::{
    Expr, ExprLit, ExprPath, Lit, Result as SynResult, Token,
    parse::{Parse, ParseStream},
    token::Static,
};

pub struct RawTranslationArgs {
    language: Expr,
    _comma: Token![,],
    static_marker: Option<Static>,
    path: Expr,
}

pub enum TranslationPathType {
    OnScopeExpression(TokenStream),
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

pub fn translation_macro(args: TranslationArgs) -> TokenStream {
    if let TranslationPathType::CompileTimePath(path) = args.path {
        if let TranslationLanguageType::CompileTimeLiteral(lang) = args.language {
            return match load_translation_static(&lang, &path) {
                Ok(Some(translation)) => quote!(#translation).into(),

                Ok(None) => {
                    let lang_name = Iso639a::from_str(&lang)
                        .map(|l| l.to_string())
                        .unwrap_or_else(|_| lang.clone());

                    let error_fmt = format!(
                        "The language '{lang_name} ({lang})' is not available for '{path}'"
                    );

                    quote!(compile_error!(#error_fmt)).into()
                }

                Err(err) => {
                    let error_fmt = err.to_string();
                    quote!(compile_error!(#error_fmt)).into()
                }
            };
        }
    }

    quote!("").into()
}

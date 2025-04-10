use std::collections::HashMap;

use thiserror::Error;
use translatable_shared::{Language, TranslationNodeError};

/// Error type for translation resolution failures
///
/// Returned by the translation macro when dynamic resolution fails.
/// For static resolution failures, errors are reported at compile time.
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("{0:#}")]
    TranslationNode(#[from] TranslationNodeError),

    #[error("The path '{0}' could not be found")]
    PathNotFound(String),

    #[error("The language '{0:?}' ('{0:#}') is not available for the path '{1}'")]
    LanguageNotAvailable(Language, String),
}

impl RuntimeError {
    /// Returns formatted error message as a String
    ///
    /// Useful for error reporting and logging. Marked `#[cold]` to hint to the
    /// compiler that this path is unlikely to be taken (optimization for error
    /// paths).
    #[inline]
    #[cold]
    pub fn cause(&self) -> String {
        format!("{self:#}")
    }
}

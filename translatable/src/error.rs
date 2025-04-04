use thiserror::Error;

/// Error type for translation resolution failures
///
/// Returned by the translation macro when dynamic resolution fails.
/// For static resolution failures, errors are reported at compile time.
#[derive(Error, Debug)]
pub enum RuntimeError {
    /// Invalid ISO 639-1 language code provided
    #[error("The language '{0}' is invalid.")]
    InvalidLanguage(String),

    /// Translation exists but not available for specified language
    #[error("The langauge '{0}' is not available for the path '{1}'")]
    LanguageNotAvailable(String, String),

    /// Requested translation path doesn't exist in any translation files
    #[error("The path '{0}' was not found in any of the translations files.")]
    PathNotFound(String),
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

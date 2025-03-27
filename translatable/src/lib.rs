use thiserror::Error;
/// Re-export the procedural macro for crate users
pub use translatable_proc::translation;

/// Error type for translation resolution failures
///
/// Returned by the translation macro when dynamic resolution fails.
/// For static resolution failures, errors are reported at compile time.
#[derive(Error, Debug)]
pub enum Error {
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

impl Error {
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

/// Internal implementation details for translation resolution
#[doc(hidden)]
pub mod internal {
    use std::collections::HashMap;

    /// Represents nested translation structures
    #[doc(hidden)]
    pub enum NestingType {
        /// Intermediate node containing nested translation objects
        Object(HashMap<String, NestingType>),
        /// Leaf node containing actual translations for different languages
        Translation(HashMap<String, String>),
    }

    impl NestingType {
        /// Resolves a translation path through nested structures
        ///
        /// # Arguments
        /// * `path` - Slice of path segments to resolve
        ///
        /// # Returns
        /// - `Some(&HashMap)` if path resolves to translations
        /// - `None` if path is invalid
        #[doc(hidden)]
        pub fn get_path(&self, path: Vec<&str>) -> Option<&HashMap<String, String>> {
            match self {
                Self::Object(nested) => {
                    let (first, rest) = path.split_first()?;
                    nested.get(*first)?.get_path(rest.to_vec())
                },

                Self::Translation(translation) => path.is_empty().then_some(translation),
            }
        }
    }
}

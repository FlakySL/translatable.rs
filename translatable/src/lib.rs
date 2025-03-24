use thiserror::Error;

// re export the macro in the main crate.
pub use translatable_proc::translation;

/// This error is used on results for the
/// translation procedural macro, the macro
/// will return a Result<Option<&'static str>, Error>,
/// when there is a dynamic expression to resolve.
///
/// For example, if the language is a dynamic expression
/// meaning it's not a literal &'static str, and it evaluates
/// on runtime, if the runtime evaluation is invalid because
/// the language does not match the ISO 639-1 specification
/// or something else, the translation macro will return an
/// Error::InvalidLanguage.
///
/// For more information on the possible errors read each
/// enum branch documentation.
#[derive(Error, Debug)]
pub enum Error {
    #[error("The language '{0}' is invalid.")]
    InvalidLanguage(String),

    #[error("The langauge '{0}' is not available for the path '{1}'")]
    LanguageNotAvailable(String, String)
}

/// This module is for internal usage, it's members
/// are not documented, and there is no support on
/// using it.
pub mod internal {
    use std::collections::HashMap;

    pub enum NestingType {
        Object(HashMap<String, NestingType>),
        Translation(HashMap<String, String>)
    }
}

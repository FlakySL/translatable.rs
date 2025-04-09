mod error;

// Export the private error module
pub use error::RuntimeError as Error;
/// Re-export the procedural macro for crate users
pub use translatable_proc::translation;

/// This module re-exports structures used by macros
/// that should not but could be used by the users
/// of the library
pub mod shared {
    pub use strum::IntoEnumIterator;
    /// Re-export utils used for both runtime and compile-time
    pub use translatable_shared::{
        Language,
        LanguageIter,
        TranslationNode,
        TranslationNodeCollection,
        TranslationNodeError,
    };
}

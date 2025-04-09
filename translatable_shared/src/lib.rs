mod languages;
mod translations;

/// Export all the structures in the common
/// top-level namespace
pub use crate::languages::{Language, LanguageIter, Similarities};
pub use crate::translations::collection::TranslationNodeCollection;
pub use crate::translations::node::{
    TranslationNesting,
    TranslationNode,
    TranslationNodeError,
    TranslationObject,
};

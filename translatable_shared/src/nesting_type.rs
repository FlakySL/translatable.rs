use std::collections::HashMap;

// Everything in this module should be
// marked with #[doc(hidden)] as we don't
// want the LSP to be notifying it's existence.

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


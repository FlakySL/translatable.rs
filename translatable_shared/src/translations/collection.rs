use super::node::{TranslationNesting, TranslationObject};
use crate::TranslationNode;

/// Wraps a collection of translation nodes, these translation nodes
/// are usually directly loaded files, and the keys to access them
/// independently are the complete system path. The collection
/// permits searching for translations by iterating all the files
/// in the specified configuration order, so most likely you don't
/// need to seek for a translation independently.
pub struct TranslationNodeCollection(TranslationNesting);

impl TranslationNodeCollection {
    /// This method may be used to load a translation
    /// independently, if you are looking for an independent
    /// translation you may want to call find_path instead.
    ///
    /// # Arguments
    /// * `path` - The OS path where the file was originally found.
    ///
    /// # Returns
    /// A top level translation node, containing all the translations
    /// in that specific file.
    #[cold]
    #[allow(unused)]
    pub fn get_node(&self, path: &str) -> Option<&TranslationNode> {
        self.0.get(&path.to_string())
    }

    /// This method is used to load a specific translation
    /// file agnostic from a "translation path" which consists
    /// of the necessary TOML object path to reach a specific
    /// translation object.
    ///
    /// # Arguments
    /// * `path` - The sections of the TOML path in order to access
    /// the desired translation object.
    ///
    /// # Returns
    /// A translation object containing a specific translation
    /// in all it's available languages.
    pub fn find_path(&self, path: &Vec<String>) -> Option<&TranslationObject> {
        self.0.values().find_map(|node| node.find_path(path))
    }
}

/// Abstraction to easily collect a `HashMap` and wrap it
/// in a `TranslationNodeCollection`.
impl FromIterator<(String, TranslationNode)> for TranslationNodeCollection {
    fn from_iter<T: IntoIterator<Item = (String, TranslationNode)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

mod error;

// Export the private error module
pub use error::Error;
/// Re-export the procedural macro for crate users
pub use translatable_proc::translation;

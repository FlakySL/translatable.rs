mod error;

// Export the private error module
pub use error::RuntimeError as Error;

/// Re-export the procedural macro for crate users
pub use translatable_proc::translation;

/// Re-export utils used for both runtime and compile-time
pub use translatable_shared::{Language, NestingType};

use crate::primitives::nibble::Nibble;
use crate::primitives::stream::HashStream;
use crate::primitives::GetTail;
use crate::primitives::Peano;

/// Capability Trait
///
/// Implemented by unit structs representing capabilities.
pub trait Capability: 'static {
    /// Type-level nibble stream responsible for Routing (Trie Path).
    /// This is typically a short hash (64-bit) of the identity.
    type Stream: HashStream;

    /// Unique Type-Level Identity.
    /// Typically a Tuple of `Char` types representing the full name.
    /// Used for collision resolution (Equality checks) in leaf nodes.
    type Identity: ?Sized + 'static;

    /// Helper to get the stream head at depth D (used for trie building)
    type At<D: Peano>: Nibble
    where
        Self::Stream: GetTail<D>;
}

// -----------------------------------------------------------------------------
// Macros
// -----------------------------------------------------------------------------

/// Macro to implement Capability for a struct manually (testing only)
#[macro_export]
macro_rules! impl_capability {
    ($name:ty, $id:expr) => {
        // We cannot easily macro-generate the raw byte stream here without proc-macro.
        // So for manual impls, we might fallback to just Hash or a simplified stream.
        // For now, this is a placeholder or requires the user to define the full type.
        // Actually, tests usually use derive.
        // If manual is needed, we need a manual helper.
        compile_error!("Manual impl_capability! is deprecated. Use #[derive(Capability)] or update macro to support new stream types.");
    };
    // Kept for backward compat if tests use it old way
    ($name:ty, $stream:ty, $identity:ty) => {
         impl $crate::Capability for $name {
            type Stream = $stream;
            type Identity = $identity;
            type At<D: $crate::Peano> = <<Self::Stream as $crate::GetTail<D>>::Out as $crate::HashStream>::Head
            where Self::Stream: $crate::GetTail<D>;
        }
    };
}

//! # tola-caps
//!
//! Capability system using 16-ary type-level trie with hash-stream routing.
//!
//! ## Features
//!
//! - **O(1) Compile-time Lookup**: Type-level hash-based routing via 16-ary radix trie
//! - **Zero Runtime Overhead**: All capability checks happen at compile time
//! - **Infinite Extensibility**: Define capabilities anywhere, no central registry needed
//! - **Clean API**: No `_` placeholders or turbofish in function signatures
//!
//! ## Quick Start
//!
//! ```ignore
//! use tola_caps::*;
//!
//! // Define capabilities with derive (auto-generates hash stream)
//! #[derive(Capability)]
//! struct CanRead;
//!
//! #[derive(Capability)]
//! struct CanWrite;
//!
//! // Build capability set
//! type MyCaps = caps![CanRead, CanWrite];
//!
//! // Function with capability requirements
//! #[requires(C: CanRead)]
//! fn process<C>() { }
//!
//! // Call with explicit type
//! process::<MyCaps>();
//! ```

#![no_std]

// Allow `::tola_caps` to work inside the crate itself
extern crate self as tola_caps;

pub mod capability;

// Re-export everything from capability module at crate root
pub use capability::*;

// Re-export proc-macros
pub use tola_caps_macros::{caps, caps_bound, conflicts, requires, requires_not, Capability};

/// Common items for the capability system.
pub mod prelude {
    pub use crate::capability::{
        // Core Traits
        Capability, Evaluate, With,
        // Set Operations
        SetUnion, SetIntersect, SupersetOf,
        // We do NOT export Has, And, Or, Not, Present, Absent, Empty
        // to avoid polluting the namespace with common words.
        // Users can import them explicitly from `tola_caps::*` if needed.
    };
    pub use tola_caps_macros::{caps, caps_bound, Capability};
    pub use crate::{with, union, intersect};
}

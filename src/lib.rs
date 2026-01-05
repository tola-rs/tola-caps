#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::crate_in_macro_def)]

// Feature flags handled:
// - std: default, enables std library
// - alloc: enables alloc types in no_std
// - specialize: enables specialization macros

//! # tola-caps
//!
//! Capability system using 16-ary type-level trie with hash-stream routing.
//!
//! **Type-level capability system for Rust.**
//!
//! ## Architecture
//!
//! `tola-caps` allows types to act as sets of keys (Capabilities).
//!
//! ### 1. Routing
//! We use a **64-bit FNV-1a Hash** of the capability's name to route it into a 16-ary Radix Trie.
//!
//! ```text
//! Type Name -> FNV Hash (u64) -> Nibble Stream -> Trie Path (Node16)
//! ```
//!
//! ### 2. Identity (Finger Tree)
//! Hash collisions are resolved via **Finger Tree Identities**.
//! Each capability encodes its full path (`Name@file:line:col`) as a Finger Tree:
//!
//! ```text
//! FDeep<Measure, Prefix, Spine, Suffix>
//!   |        |        |        |
//!   XOR hash  First   Middle    Last
//!   (O(1))   1-4 B    bytes    1-4 B
//! ```
//!
//! ### 3. Comparison
//! Three-layer comparison: Measure (O(1)) -> Prefix -> Suffix -> Spine (O(log n))
//!
//! ### 4. Fallback Tricks
//! We use **Autoref/Method Priority** to achieve trait detection and stable specialization.
//!
//! ```text
//! +-------------------------------------------------------------------+
//! |  Layer 0: Primitives                                              |
//! |  - Nibble (X0-XF), Stream, Identity (Byte/Char), Finger Tree      |
//! +-------------------------------------------------------------------+
//!                                |
//!                                v
//! +-------------------------------------------------------------------+
//! |  Layer 1: Trie Core                                               |
//! |  - Node16, Leaf, Bucket (Storage)                                 |
//! |  - InsertAt, RemoveAt, Evaluate (Logic)                           |
//! +-------------------------------------------------------------------+
//!                                |
//!                                v
//! +-------------------------------------------------------------------+
//! |  Layer 2: User API                                                |
//! |  - macros (caps!, caps_check!), AutoCaps, Feature Detection       |
//! +-------------------------------------------------------------------+
//! ```
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
//! use tola_caps::prelude::*;
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
//! fn process<C>()
//! where
//!     C: Evaluate<CanRead, Out = Present>,
//! { }
//!
//! // Call with explicit type
//! process::<MyCaps>();
//! ```

// #![cfg_attr(not(test), no_std)] - Handled by top-level attribute

// Allow `::tola_caps` to work inside the crate itself
extern crate self as tola_caps;

#[cfg(feature = "alloc")]
extern crate alloc;

// Re-export paste for define_trait_cap! macro
pub use paste;

// =============================================================================
// Layer 0: Primitives (no dependencies)
// =============================================================================
pub mod primitives;

// =============================================================================
// Layer 1: Trie Core
// =============================================================================
pub mod trie;

// =============================================================================
// Layer 2: Std Trait Detection
// =============================================================================
// TEMPORARILY DISABLED for debugging - this generates massive amounts of code
#[cfg(feature = "detect")]
pub mod detect;

// Placeholder module when detect is disabled
#[cfg(not(feature = "detect"))]
pub mod detect {
    use core::marker::PhantomData;

    // Minimal stub for compatibility
    pub trait AutoCaps {
        const IS_CLONE: bool = false;
        const IS_COPY: bool = false;
        const IS_DEBUG: bool = false;
        const IS_DEFAULT: bool = false;
        const IS_SEND: bool = false;
        const IS_SYNC: bool = false;
    }
    pub trait AutoCapSet {
        type Out;
    }
    // Use PhantomData to consume T
    pub type Cap<T> = PhantomData<T>;
    pub struct InsertIf<S, Cap, const B: bool>(PhantomData<(S, Cap)>);
    pub struct InsertIfType<S, Cap, B>(PhantomData<(S, Cap, B)>);
}

// =============================================================================
// Layer 3: Specialization Sugar
// =============================================================================
pub mod spec;

// Syntax macros (dispatch!, specialize_trait!, impl_specialized!)
pub mod syntax_macros;

// =============================================================================
// Re-exports at Crate Root
// =============================================================================

// Re-export core types from trie and primitives at crate root
pub use trie::*;
pub use primitives::bool::{Bool, Present, Absent, BoolAnd, BoolOr, BoolNot};
pub use primitives::nibble::{
    Nibble, NibbleEq,
    X0, X1, X2, X3, X4, X5, X6, X7,
    X8, X9, XA, XB, XC, XD, XE, XF,
};
pub use primitives::stream::{
    HashStream, GetTail, ConstStream, AltStream, Cons,
    Z, S, DefaultMaxDepth, StreamEq, StreamEqDispatch, D0, D16, Peano,
    HashStream16,
};

// Backward compatibility aliases
pub mod std_caps {
    pub use crate::detect::*;
}
pub mod capability {
    pub use crate::trie::*;
    pub use crate::Evaluate;
}

// Re-export proc-macros
pub use macros::{cap, caps, caps_bound, caps_check, specialize, specialize_inherent, specialization, derive_trait_cap, Capability, AutoCaps, trait_autocaps, define_type_cap, name_stream, make_routing_stream, make_identity_bytes, __internal_make_identity};

// =============================================================================
// Declarative Macro Bridge for #[derive(Capability)]
// =============================================================================
//
// Three-layer macro architecture to get module_path!() into proc-macros:
// 1. #[derive(Capability)] (proc-macro) generates __impl_capability! call
// 2. __impl_capability! (this decl-macro) expands concat!(module_path!(), ...)
// 3. make_routing_stream! (proc-macro) receives string literal

/// Internal macro bridge - DO NOT USE DIRECTLY.
/// Use #[derive(Capability)] instead.
#[macro_export]
#[doc(hidden)]
macro_rules! __impl_capability {
    ($ty:ty, $name:expr) => {
        impl $crate::Capability for $ty {
            // Stream: hash-based routing for trie navigation
            type Stream = $crate::make_routing_stream!(concat!(module_path!(), "::", $name));

            // Identity: Type-level character list for exact comparison
            type Identity = $crate::__make_identity_from_str!(concat!(module_path!(), "::", $name));

            type At<D: $crate::Peano> = <<Self::Stream as $crate::GetTail<D>>::Out as $crate::HashStream>::Head
            where Self::Stream: $crate::GetTail<D>;
        }
    };
}

/// Helper macro to convert concat!() result to proc-macro call
#[macro_export]
#[doc(hidden)]
macro_rules! __make_identity_from_str {
    ($s:expr) => {
        $crate::__internal_make_identity!($s)
    };
}

/// Common items for the capability system.
pub mod prelude {
    pub use crate::trie::{
        // Core Traits
        Capability, Evaluate, With, Inspect,
        // Set Operations
        SetUnion, SetIntersect, SupersetOf,
    };
    pub use crate::detect::AutoCaps;
    #[cfg(feature = "detect")]
    pub use crate::detect::{
        AutoCapSet, Cap,
        // All capability markers
        IsClone, IsCopy, IsDebug, IsDefault, IsSend, IsSync,
        IsEq, IsPartialEq, IsOrd, IsPartialOrd, IsHash,
        IsDisplay, IsSized, IsUnpin,
    };
    pub use macros::{caps, caps_bound, caps_check, Capability};
    // Note: with!, union!, intersect!, check! are #[macro_export] so they're at crate root
}


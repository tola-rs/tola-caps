//! # Layer 2: Std Trait Detection
//!
//! Provides compile-time detection of standard library traits
//! and automatic capability set construction.
//!
//! ## Public API
//!
//! Use `caps_check!` macro for trait detection:
//!
//! ```ignore
//! use tola_caps::caps_check;
//!
//! // Check if a type implements Clone
//! let is_clone: bool = caps_check!(String: Clone);
//!
//! // Boolean expressions
//! let is_copy_or_default: bool = caps_check!(i32: Copy | Default);
//! ```
//!
//! ## Supported Traits
//!
//! Clone, Copy, Debug, Default, Send, Sync, Eq, PartialEq,
//! Ord, PartialOrd, Hash, Display, Sized, Unpin

pub mod autocaps;

macros::define_std_traits!();

pub use autocaps::{AutoCapSet, Cap, InsertIf, InsertIfType};

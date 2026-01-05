//! User-facing macro implementations
//!
//! # Unified Macro Names
//!
//! | Macro | Usage | Purpose |
//! |-------|-------|---------|
//! | `#[cap]` | on trait/struct | Enable caps system support |
//! | `#[derive(Capability)]` | on struct | Define capability marker |
//! | `#[derive(CapHolder)]` | on struct | Add phantom cap field |
//! | `#[specialize]` | on impl | Enable specialization (attribute) |
//! | `specialization!` | function macro | Specialization block syntax |
//! | `caps!` | function macro | Build capability set |

mod auto_caps;
mod cap_set;
pub mod capability;
mod caps_bound;
pub mod specialize;
pub mod specialize_common;

// Re-export all public items
pub use auto_caps::{expand_cap_attr, expand_derive_autocaps, expand_trait_autocaps, define_type_cap, derive_trait_cap};
pub use cap_set::{build_capset, check_duplicates, expand_define_capabilities, CapsInput, DefineCapabilitiesInput};
pub use capability::expand_derive_capability;
pub use caps_bound::{expand_caps_enum, expand_caps_fn, expand_caps_impl, expand_caps_struct, CapsArgs};

// Legacy re-exports (for backward compatibility)
pub use auto_caps::expand_cap_attr as expand_auto_caps;

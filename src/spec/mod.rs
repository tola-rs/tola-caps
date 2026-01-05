//! # Layer 3: Specialization Sugar
//!
//! This module provides specialization mechanisms for Stable Rust.
//!
//! ## Module Structure
//!
//! ```text
//! spec/
//! ├── runtime.rs  - Autoref-based runtime detection (returns bool)
//! ├── compile.rs  - has_impl! macro for trait detection
//! └── dispatch.rs - Type-level dispatch (SelectCap, MethodImpl)
//! ```
//!
//! ## Usage
//!
//! ### Trait Detection (preferred)
//! ```ignore
//! use tola_caps::caps_check;
//!
//! assert!(caps_check!(String: Clone));
//! assert!(caps_check!(String: Clone & !Copy));
//! ```
//!
//! ### Type-Level Dispatch
//! ```ignore
//! type Impl = <Cap<T> as SelectClone<CloneImpl, FallbackImpl>>::Out;
//! Impl::call(&value);
//! ```

pub mod runtime;
pub mod compile;
pub mod dispatch;

// Re-export key types
pub use runtime::SpecializeWrapper;
pub use dispatch::{
    SelectCap, SelectAnd, SelectOr, SelectNot,
    SelectStaticCall, BoolStaticCall, StaticSelect,
    MethodImpl, StaticMethodImpl, TypeSelector, NoImpl,
};

// Select traits are only available when detect feature is enabled
#[cfg(feature = "detect")]
pub use dispatch::{
    SelectClone, SelectCopy, SelectDebug, SelectDefault,
    SelectSend, SelectSync, SelectEq, SelectPartialEq,
    SelectOrd, SelectPartialOrd, SelectHash,
    SelectDisplay, SelectSized, SelectUnpin,
};

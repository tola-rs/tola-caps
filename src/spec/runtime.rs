//! # Runtime Specialization
//!
//! **DEPRECATED**: Use `caps_check!` instead.
//!
//! The macros in this module (`specialize_is_clone!` etc.) are kept for
//! backward compatibility but `caps_check!` is the preferred API.
//!
//! ```ignore
//! use tola_caps::caps_check;
//!
//! // Instead of: specialize_is_clone!(val)
//! // Use:        caps_check!(TypeOfVal: Clone)
//! ```

/// Wrapper for autoref-based specialization.
#[doc(hidden)]
pub struct SpecializeWrapper<T>(pub T);

// Clone detection
#[doc(hidden)]
pub trait CloneFallback<T> { fn is_clone(&self) -> bool { false } }
impl<T> CloneFallback<T> for SpecializeWrapper<T> {}
impl<T: Clone> SpecializeWrapper<T> {
    pub fn is_clone(&self) -> bool { true }
    pub fn do_clone(&self) -> T { self.0.clone() }
}

// Debug detection
#[doc(hidden)]
pub trait DebugFallback<T> { fn is_debug(&self) -> bool { false } }
impl<T> DebugFallback<T> for SpecializeWrapper<T> {}
impl<T: core::fmt::Debug> SpecializeWrapper<T> {
    pub fn is_debug(&self) -> bool { true }
}

// Copy detection
#[doc(hidden)]
pub trait CopyFallback<T> { fn is_copy(&self) -> bool { false } }
impl<T> CopyFallback<T> for SpecializeWrapper<T> {}
impl<T: Copy> SpecializeWrapper<T> {
    pub fn is_copy(&self) -> bool { true }
}
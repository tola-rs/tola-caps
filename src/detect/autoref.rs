//! Autoref-based trait detection machinery.
//!
//! This module implements the "Inherent Const Fallback" pattern for
//! compile-time trait detection on concrete types.
//!
//! ## How it works
//!
//! For each trait T we want to detect:
//! 1. Define a fallback trait with `const IS_T: bool = false`
//! 2. Implement fallback for `Detect<X>` for all X
//! 3. Implement an inherent const `IS_T = true` for `Detect<X>` where `X: T`
//!
//! When resolving `Detect::<Concrete>::IS_T`, the compiler:
//! - If `Concrete: T`, finds the inherent const (true)
//! - Otherwise, finds the trait const (false)
//!
//! ## Limitation
//!
//! This only works for **concrete types** known at the call site.
//! It does NOT work in generic contexts like `fn foo<T>()`.

use core::marker::PhantomData;

/// Detection wrapper type.
#[doc(hidden)]
pub struct Detect<T>(PhantomData<T>);

// =============================================================================
// Std Trait Detection (generated)
// =============================================================================

/// Generate fallback trait + inherent const for a std trait.
macro_rules! impl_detect {
    // Special case for Debug (core::fmt::Debug)
    (Debug) => {
        #[doc(hidden)]
        pub trait DebugFallback { const IS_DEBUG: bool = false; }
        impl<T> DebugFallback for Detect<T> {}
        impl<T: core::fmt::Debug> Detect<T> { pub const IS_DEBUG: bool = true; }
    };
    // General case
    ($Trait:ident) => {
        ::paste::paste! {
            #[doc(hidden)]
            pub trait [<$Trait Fallback>] { const [<IS_ $Trait:upper>]: bool = false; }
            impl<T> [<$Trait Fallback>] for Detect<T> {}
            impl<T: $Trait> Detect<T> { pub const [<IS_ $Trait:upper>]: bool = true; }
        }
    };
}

impl_detect!(Clone);
impl_detect!(Copy);
impl_detect!(Debug);
impl_detect!(Default);
impl_detect!(Send);
impl_detect!(Sync);


//! # Compile-Time Trait Detection
//!
//! This module provides low-level trait detection utilities.
//!
//! ## User API
//!
//! For most use cases, prefer the `caps_check!` macro which supports
//! boolean expressions like `Clone & !Copy`:
//!
//! ```ignore
//! use tola_caps::caps_check;
//!
//! assert!(caps_check!(String: Clone));
//! assert!(caps_check!(String: Clone & !Copy));
//! ```
//!
//! ## Low-level: `has_impl!`
//!
//! For detecting arbitrary traits on concrete types:
//!
//! ```
//! use tola_caps::has_impl;
//!
//! trait MyTrait {}
//! impl MyTrait for i32 {}
//!
//! assert!(has_impl!(i32, MyTrait));
//! assert!(!has_impl!(String, MyTrait));
//! ```

// =============================================================================
// has_impl! - Low-level trait detection (concrete types only)
// =============================================================================

/// Check if a concrete type implements a trait at compile time.
///
/// Uses the "Inherent Const Fallback" pattern: an inherent const shadows
/// a trait const when the bound is satisfied.
///
/// **Note**: Only works for concrete types. For generic contexts, use
/// `caps_check!` with standard traits (Clone, Copy, Debug, Default, Send, Sync).
///
/// # Usage
///
/// ```
/// use tola_caps::has_impl;
///
/// assert!(has_impl!(String, Clone));
/// assert!(!has_impl!(String, Copy));
///
/// trait MyTrait {}
/// impl MyTrait for i32 {}
/// assert!(has_impl!(i32, MyTrait));
/// ```
#[macro_export]
macro_rules! has_impl {
    ($T:ty, $Trait:path) => {{
        struct __Probe<T>(core::marker::PhantomData<T>);

        trait __Fallback { const VAL: bool = false; }
        impl<T> __Fallback for __Probe<T> {}

        impl<T: $Trait> __Probe<T> {
            #[allow(dead_code)]
            const VAL: bool = true;
        }

        __Probe::<$T>::VAL
    }};
}

// =============================================================================
// define_trait_cap! - Register a user trait into the caps system
// =============================================================================

/// Define a capability marker for a user trait.
///
/// This macro generates a capability marker struct `Is<Trait>` with
/// `#[derive(Capability)]` for integration with the type-level capability system.
///
/// # Usage
///
/// ```ignore
/// trait Serialize { fn serialize(&self) -> Vec<u8>; }
///
/// define_trait_cap!(Serialize);
///
/// // Now IsSerialize can be used in capability sets
/// ```
#[macro_export]
macro_rules! define_trait_cap {
    ($Trait:ident) => {
        $crate::paste::paste! {
            /// Capability marker for detecting `$Trait` implementation.
            #[derive($crate::Capability)]
            pub struct [<Is $Trait>];
        }
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn test_has_impl_std_traits() {
        assert!(has_impl!(String, Clone));
        assert!(has_impl!(i32, Clone));
        assert!(has_impl!(i32, Copy));
        assert!(!has_impl!(String, Copy));
        assert!(has_impl!(i32, core::fmt::Debug));
    }

    #[test]
    fn test_has_impl_custom_trait() {
        #[allow(dead_code)]
        trait MyTrait {}
        impl MyTrait for i32 {}

        assert!(has_impl!(i32, MyTrait));
        assert!(!has_impl!(String, MyTrait));
    }
}


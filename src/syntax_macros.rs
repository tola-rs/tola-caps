//! Nightly Specialization Syntax Simulation Macros
//!
//! This module provides macros that mimic Nightly's `default impl` syntax
//! while using pure compile-time type-level dispatch under the hood.

// =============================================================================
// specialize_trait! - Define a trait with default + specialized impls
// =============================================================================

/// Define a trait with default implementation that can be specialized.
///
/// # Example
///
/// ```ignore
/// specialize_trait! {
///     trait MyTrait {
///         fn describe() -> &'static str;
///     }
///
///     // Default implementation for all types
///     default impl<T> {
///         fn describe() -> &'static str { "General type" }
///     }
///
///     // Specialized for Clone types
///     impl<T: Clone> {
///         fn describe() -> &'static str { "Cloneable type" }
///     }
///
///     // Specialized for specific type
///     impl String {
///         fn describe() -> &'static str { "String type" }
///     }
/// }
/// ```
#[macro_export]
macro_rules! specialize_trait {
    (
        trait $trait_name:ident {
            fn $method:ident() -> $ret:ty;
        }

        default impl<T> {
            fn $default_method:ident() -> $default_ret:ty { $default_body:expr }
        }

        $(
            impl<T: $bound:path> {
                fn $spec_method:ident() -> $spec_ret:ty { $spec_body:expr }
            }
        )*
    ) => {
        // Define the trait
        pub trait $trait_name {
            fn $method() -> $ret;
        }

        // Use DispatchSelector to pick implementation
        mod __specialize_impl {
            use super::*;
            use $crate::std_caps::{AutoCapSet, Cap};
            use $crate::capability::{Evaluate, Present, Absent};

            pub trait Selector {
                fn select() -> $ret;
            }

            // Default case (Absent)
            impl Selector for Absent {
                fn select() -> $ret { $default_body }
            }

            // Specialized case (Present)
            impl Selector for Present {
                fn select() -> $ret { $($spec_body)* }
            }
        }

        // Blanket impl that dispatches based on capability
        impl<T: $crate::std_caps::AutoCapSet> $trait_name for T
        where
            $crate::std_caps::Cap<T>: $crate::capability::Evaluate<$crate::std_caps::IsClone>,
            <$crate::std_caps::Cap<T> as $crate::capability::Evaluate<$crate::std_caps::IsClone>>::Out:
                __specialize_impl::Selector,
        {
            fn $method() -> $ret {
                <<$crate::std_caps::Cap<T> as $crate::capability::Evaluate<$crate::std_caps::IsClone>>::Out
                    as __specialize_impl::Selector>::select()
            }
        }
    };
}

// =============================================================================
// dispatch! - Inline dispatch based on capability
// =============================================================================

/// Inline compile-time dispatch based on capability.
///
/// # Example
///
/// ```ignore
/// fn process<T: AutoCapSet>() -> &'static str
/// where Cap<T>: Evaluate<IsClone>
/// {
///     dispatch!(Cap<T>, IsClone, {
///         Present => "Has Clone",
///         Absent => "No Clone",
///     })
/// }
/// ```
#[macro_export]
macro_rules! dispatch {
    ($cap:ty, $query:ty, {
        Present => $present:expr,
        Absent => $absent:expr $(,)?
    }) => {{
        // Compile-time const evaluation
        if <$cap as $crate::Evaluate<$query>>::RESULT {
            $present
        } else {
            $absent
        }
    }};
}

// =============================================================================
// impl_specialized! - Simpler specialization syntax
// =============================================================================

/// Implement a trait with automatic specialization based on capabilities.
///
/// # Example
///
/// ```ignore
/// impl_specialized! {
///     impl<T> Display for Wrapper<T> {
///         where Clone => { write!(f, "Cloneable") }
///         where !Clone => { write!(f, "Not cloneable") }
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_specialized {
    (
        impl<$T:ident : AutoCapSet> $trait:ident for $type:ty {
            where $cap:ident => { $present_body:expr }
            default => { $default_body:expr }
        }
    ) => {
        impl<$T: $crate::detect::AutoCapSet> $trait for $type
        where
            $crate::detect::Cap<$T>: $crate::Evaluate<$crate::detect::$cap>,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                if <$crate::detect::Cap<$T> as $crate::Evaluate<$crate::detect::$cap>>::RESULT {
                    $present_body
                } else {
                    $default_body
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused

    #[test]
    fn test_dispatch_macro() {
        use crate::detect::{AutoCapSet, Cap, IsClone};
        use crate::Evaluate;
        use crate::AutoCaps;

        #[derive(Clone, AutoCaps)]
        struct Yes;

        #[derive(AutoCaps)]
        struct No;

        fn check<T: AutoCapSet>() -> &'static str
        where Cap<T>: Evaluate<IsClone>
        {
            dispatch!(Cap<T>, IsClone, {
                Present => "Clone!",
                Absent => "No Clone",
            })
        }

        assert_eq!(check::<Yes>(), "Clone!");
        assert_eq!(check::<No>(), "No Clone");
    }
}

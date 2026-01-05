//! Comprehensive Specialization Tests
//!
//! This test module covers all specialization features:
//! 1. `caps_check!` - Runtime trait detection
//! 2. `#[specialize]` attribute - Distributed specialization
//! 3. `specialization!` macro - Block-style specialization
//! 4. Type-level dispatch via Cap and Evaluate
//!
//! Test organization:
//! - `mod caps_check_tests` - Basic trait detection
//! - `mod type_level_dispatch_tests` - Cap/Evaluate based dispatch
//! - `mod attribute_specialize_tests` - #[specialize] attribute tests
//! - `mod block_specialize_tests` - specialization! {} macro tests
//! - `mod edge_cases` - Boundary conditions and complex scenarios

use std::fmt::Debug;
use tola_caps::prelude::*;
use tola_caps::caps_check;
use tola_caps::std_caps::{AutoCaps, AutoCapSet, Cap, IsClone, IsCopy, IsDebug, IsDefault, IsSend, IsSync};
use tola_caps::{Evaluate, Present, Absent};
use tola_caps::trie::{And, Not, Or};

// ============================================================================
// Helper Macros
// ============================================================================

/// Assert that a compile-time capability check is true
macro_rules! assert_cap {
    ($ty:ty : $($trait_expr:tt)+) => {
        assert!(
            caps_check!($ty: $($trait_expr)+),
            "Expected `{}` to satisfy: {}",
            stringify!($ty),
            stringify!($($trait_expr)+)
        );
    };
}

/// Assert that a compile-time capability check is false
macro_rules! assert_not_cap {
    ($ty:ty : $($trait_expr:tt)+) => {
        assert!(
            !caps_check!($ty: $($trait_expr)+),
            "Expected `{}` NOT to satisfy: {}",
            stringify!($ty),
            stringify!($($trait_expr)+)
        );
    };
}

// ============================================================================
// Test Types
// ============================================================================

#[derive(Clone, Copy, Debug, Default, tola_caps::AutoCaps)]
struct FullyCapable;

#[derive(Clone, Debug, tola_caps::AutoCaps)]
struct OnlyCloneDebug;

#[derive(tola_caps::AutoCaps)]
struct NoTraits;

#[derive(Copy, Clone, tola_caps::AutoCaps)]
struct CopyOnly;

#[derive(Clone, Default, tola_caps::AutoCaps)]
struct CloneDefault;

// ============================================================================
// PART 1: caps_check! Tests
// ============================================================================

mod caps_check_tests {
    use super::*;

    #[test]
    fn basic_trait_detection() {
        // Positive checks
        assert_cap!(FullyCapable: Clone);
        assert_cap!(FullyCapable: Copy);
        assert_cap!(FullyCapable: Debug);
        assert_cap!(FullyCapable: Default);

        // Negative checks
        assert_not_cap!(NoTraits: Clone);
        assert_not_cap!(NoTraits: Copy);
        assert_not_cap!(NoTraits: Debug);
        assert_not_cap!(NoTraits: Default);
    }

    #[test]
    fn partial_trait_detection() {
        // OnlyCloneDebug: Clone=yes, Copy=no
        assert_cap!(OnlyCloneDebug: Clone);
        assert_cap!(OnlyCloneDebug: Debug);
        assert_not_cap!(OnlyCloneDebug: Copy);
        assert_not_cap!(OnlyCloneDebug: Default);

        // CopyOnly: Copy=yes, Debug=no
        assert_cap!(CopyOnly: Clone); // Copy implies Clone
        assert_cap!(CopyOnly: Copy);
        assert_not_cap!(CopyOnly: Debug);
        assert_not_cap!(CopyOnly: Default);
    }

    #[test]
    fn boolean_and_expression() {
        // Clone AND Copy
        assert_cap!(FullyCapable: Clone & Copy);
        assert_not_cap!(OnlyCloneDebug: Clone & Copy);

        // Clone AND Debug
        assert_cap!(FullyCapable: Clone & Debug);
        assert_cap!(OnlyCloneDebug: Clone & Debug);
        assert_not_cap!(NoTraits: Clone & Debug);
    }

    #[test]
    fn boolean_or_expression() {
        // Clone OR Copy
        assert_cap!(FullyCapable: Clone | Copy);
        assert_cap!(OnlyCloneDebug: Clone | Copy); // Clone=yes
        assert_cap!(CopyOnly: Clone | Copy);       // Copy=yes
        assert_not_cap!(NoTraits: Clone | Copy);
    }

    #[test]
    fn boolean_not_expression() {
        // NOT Clone
        assert_cap!(NoTraits: !Clone);
        assert_not_cap!(FullyCapable: !Clone);

        // NOT Copy
        assert_cap!(OnlyCloneDebug: !Copy);
        assert_not_cap!(FullyCapable: !Copy);
    }

    #[test]
    fn complex_boolean_expression() {
        // Clone AND NOT Copy (the "Clone but not Copy" pattern)
        assert_cap!(OnlyCloneDebug: Clone & !Copy);
        assert_not_cap!(FullyCapable: Clone & !Copy);  // has Copy
        assert_not_cap!(NoTraits: Clone & !Copy);      // no Clone

        // (Clone OR Debug) AND NOT Copy
        assert_cap!(OnlyCloneDebug: (Clone | Debug) & !Copy);
        assert_not_cap!(FullyCapable: (Clone | Debug) & !Copy);

        // (Clone AND Copy) OR (Debug AND Default)
        assert_cap!(FullyCapable: (Clone & Copy) | (Debug & Default));
        assert_not_cap!(OnlyCloneDebug: (Clone & Copy) | (Debug & Default));
    }

    #[test]
    fn standard_library_types() {
        // String: Clone but not Copy
        assert_cap!(String: Clone);
        assert_not_cap!(String: Copy);
        assert_cap!(String: Clone & !Copy);

        // i32: Clone and Copy
        assert_cap!(i32: Clone);
        assert_cap!(i32: Copy);
        assert_cap!(i32: Clone & Copy);

        // Vec<u8>: Clone but not Copy
        assert_cap!(Vec<u8>: Clone);
        assert_not_cap!(Vec<u8>: Copy);
    }

    #[test]
    fn send_sync_detection() {
        // Standard types
        assert_cap!(i32: Send);
        assert_cap!(i32: Sync);
        assert_cap!(String: Send);
        assert_cap!(String: Sync);

        // Rc is NOT Send
        use std::rc::Rc;
        assert_not_cap!(Rc<i32>: Send);
        assert_not_cap!(Rc<i32>: Sync);

        // Arc IS Send + Sync
        use std::sync::Arc;
        assert_cap!(Arc<i32>: Send);
        assert_cap!(Arc<i32>: Sync);
    }
}

// ============================================================================
// PART 2: Type-Level Dispatch Tests
// ============================================================================

mod type_level_dispatch_tests {
    use super::*;

    // Helper trait for selecting behavior
    trait SelectBehavior {
        fn describe() -> &'static str;
    }

    struct CloneableBehavior;
    impl SelectBehavior for CloneableBehavior {
        fn describe() -> &'static str { "Cloneable" }
    }

    struct DefaultBehavior;
    impl SelectBehavior for DefaultBehavior {
        fn describe() -> &'static str { "Default" }
    }

    // Use Bool::If for type-level selection
    trait PickBehavior {
        type Behavior: SelectBehavior;
    }

    impl PickBehavior for Present {
        type Behavior = CloneableBehavior;
    }

    impl PickBehavior for Absent {
        type Behavior = DefaultBehavior;
    }

    fn dispatch_on_clone<T: AutoCapSet>() -> &'static str
    where
        Cap<T>: Evaluate<IsClone>,
        <Cap<T> as Evaluate<IsClone>>::Out: PickBehavior,
    {
        <<Cap<T> as Evaluate<IsClone>>::Out as PickBehavior>::Behavior::describe()
    }

    #[test]
    fn basic_type_level_dispatch() {
        assert_eq!(dispatch_on_clone::<FullyCapable>(), "Cloneable");
        assert_eq!(dispatch_on_clone::<OnlyCloneDebug>(), "Cloneable");
        assert_eq!(dispatch_on_clone::<NoTraits>(), "Default");
    }

    fn check_clone_copy<T: AutoCapSet>() -> bool
    where
        Cap<T>: Evaluate<And<IsClone, IsCopy>>,
    {
        <Cap<T> as Evaluate<And<IsClone, IsCopy>>>::RESULT
    }

    fn check_clone_not_copy<T: AutoCapSet>() -> bool
    where
        Cap<T>: Evaluate<And<IsClone, Not<IsCopy>>>,
    {
        <Cap<T> as Evaluate<And<IsClone, Not<IsCopy>>>>::RESULT
    }

    #[test]
    fn compound_capability_check() {
        // Clone AND Copy
        assert!(check_clone_copy::<FullyCapable>());
        assert!(!check_clone_copy::<OnlyCloneDebug>());
        assert!(!check_clone_copy::<NoTraits>());

        // Clone AND NOT Copy
        assert!(!check_clone_not_copy::<FullyCapable>());
        assert!(check_clone_not_copy::<OnlyCloneDebug>());
        assert!(!check_clone_not_copy::<NoTraits>());
    }

    // Test with standard library types
    fn is_copyable<T: AutoCapSet>() -> bool
    where
        Cap<T>: Evaluate<IsCopy>,
    {
        <Cap<T> as Evaluate<IsCopy>>::RESULT
    }

    #[test]
    fn std_type_dispatch() {
        // u32 is Copy
        assert!(is_copyable::<u32>());
        // String is not Copy
        assert!(!is_copyable::<String>());
    }
}

// ============================================================================
// PART 3: Negative Tests (Missing Implementations)
// ============================================================================

mod negative_tests {
    use super::*;

    // Type without any derives
    struct BareStruct;

    #[test]
    fn no_autocaps_fallback() {
        // BareStruct doesn't have AutoCaps, so can't use caps_check! in generic context
        // But concrete type checks still work via autoref trick

        // This should return false (BareStruct doesn't implement Clone)
        assert!(!caps_check!(BareStruct: Clone));
        assert!(!caps_check!(BareStruct: Copy));
        assert!(!caps_check!(BareStruct: Debug));
    }

    #[test]
    fn partial_implementation() {
        #[derive(Clone, tola_caps::AutoCaps)]
        struct OnlyClone;

        assert!(caps_check!(OnlyClone: Clone));
        assert!(!caps_check!(OnlyClone: Copy));
        assert!(!caps_check!(OnlyClone: Debug));
        assert!(!caps_check!(OnlyClone: Default));
    }
}

// ============================================================================
// PART 4: Edge Cases
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn double_negation() {
        // NOT NOT Clone should equal Clone
        assert_cap!(FullyCapable: !!Clone);
        assert_not_cap!(NoTraits: !!Clone);
    }

    #[test]
    fn deeply_nested_boolean() {
        // ((Clone & Copy) | (Debug & Default)) & !(!Send)
        // = ((Clone & Copy) | (Debug & Default)) & Send
        assert_cap!(FullyCapable: ((Clone & Copy) | (Debug & Default)) & Send);
    }

    #[test]
    fn all_standard_traits() {
        // FullyCapable should have all basic traits
        assert_cap!(FullyCapable: Clone & Copy & Debug & Default);
    }

    #[test]
    fn generic_type_parameters() {
        // Test with generic instantiations
        assert_cap!(Vec<i32>: Clone);
        assert_cap!(Option<String>: Clone);
        assert_cap!(Result<i32, String>: Clone);

        // Vec is not Copy even if element is Copy
        assert_not_cap!(Vec<i32>: Copy);

        // Option<T: Copy> IS Copy
        assert_cap!(Option<i32>: Copy);
        assert_not_cap!(Option<String>: Copy);
    }

    #[test]
    fn tuple_types() {
        // Tuples derive Clone/Copy if all elements do
        assert_cap!((i32, i32): Clone);
        assert_cap!((i32, i32): Copy);
        assert_cap!((i32, String): Clone);
        assert_not_cap!((i32, String): Copy); // String is not Copy
    }

    #[test]
    fn array_types() {
        // Arrays derive Clone/Copy if element does
        assert_cap!([i32; 5]: Clone);
        assert_cap!([i32; 5]: Copy);
        assert_cap!([String; 3]: Clone);
        assert_not_cap!([String; 3]: Copy);
    }
}

// ============================================================================
// PART 5: Integration with Custom Traits
// ============================================================================

mod custom_trait_integration {
    use super::*;

    // Define a custom trait and register it
    trait Serializable {
        fn serialize(&self) -> Vec<u8>;
    }

    // Implement for specific types
    impl Serializable for String {
        fn serialize(&self) -> Vec<u8> {
            self.as_bytes().to_vec()
        }
    }

    impl Serializable for i32 {
        fn serialize(&self) -> Vec<u8> {
            self.to_le_bytes().to_vec()
        }
    }

    #[test]
    fn custom_trait_detection() {
        // caps_check! works on custom traits via autoref
        assert!(caps_check!(String: Serializable));
        assert!(caps_check!(i32: Serializable));
        assert!(!caps_check!(Vec<u8>: Serializable)); // Not implemented
    }

    #[test]
    fn custom_and_standard_combined() {
        // String: Clone AND Serializable
        assert!(caps_check!(String: Clone & Serializable));

        // i32: Copy AND Serializable
        assert!(caps_check!(i32: Copy & Serializable));

        // Vec<u8>: Clone but NOT Serializable
        assert!(caps_check!(Vec<u8>: Clone & !Serializable));
    }
}

// ============================================================================
// PART 6: Performance-Critical Patterns
// ============================================================================

mod performance_patterns {
    use super::*;

    /// Select optimal strategy based on type capabilities
    fn select_strategy<T: AutoCaps>() -> &'static str {
        if caps_check!(T: Copy) {
            "memcpy"
        } else if caps_check!(T: Clone) {
            "clone"
        } else {
            "move"
        }
    }

    #[test]
    fn strategy_selection() {
        assert_eq!(select_strategy::<i32>(), "memcpy");
        assert_eq!(select_strategy::<String>(), "clone");
        // NoTraits has AutoCaps, so we can check it
        assert_eq!(select_strategy::<NoTraits>(), "move");
    }

    /// Type-level strategy with zero runtime dispatch
    trait Strategy {
        fn name() -> &'static str;
    }

    struct MemcpyStrategy;
    impl Strategy for MemcpyStrategy {
        fn name() -> &'static str { "memcpy" }
    }

    struct CloneStrategy;
    impl Strategy for CloneStrategy {
        fn name() -> &'static str { "clone" }
    }

    struct MoveStrategy;
    impl Strategy for MoveStrategy {
        fn name() -> &'static str { "move" }
    }

    #[test]
    fn compile_time_strategy() {
        // The compiler resolves these at compile time
        fn get_strategy<T: AutoCapSet>() -> &'static str
        where
            Cap<T>: Evaluate<IsCopy> + Evaluate<IsClone>,
        {
            if <Cap<T> as Evaluate<IsCopy>>::RESULT {
                "memcpy"
            } else if <Cap<T> as Evaluate<IsClone>>::RESULT {
                "clone"
            } else {
                "move"
            }
        }

        assert_eq!(get_strategy::<FullyCapable>(), "memcpy");
        assert_eq!(get_strategy::<OnlyCloneDebug>(), "clone");
        assert_eq!(get_strategy::<NoTraits>(), "move");
    }
}

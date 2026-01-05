#![allow(unused_imports, non_upper_case_globals)]
//! Comprehensive caps_check! tests covering all scenarios and edge cases.
//!
//! This test file covers:
//! 1. All boolean expression combinations (AND, OR, NOT, nested)
//! 2. All type scenarios (concrete, generic, with/without AutoCaps)
//! 3. All trait scenarios (built-in, custom, mixed)
//! 4. Edge cases and potential bug scenarios
//! 5. #[derive(AutoCaps)] macro correctness verification

use std::fmt::Debug;
use tola_caps::caps_check;
use tola_caps::AutoCaps;
use tola_caps::std_caps::AutoCaps as AutoCapsTrait;

// =============================================================================
// Test Traits
// =============================================================================

#[allow(dead_code)]
trait CustomTrait1 {}
#[allow(dead_code)]
trait CustomTrait2 {}
#[allow(dead_code)]
trait CustomTrait3 {}

// =============================================================================
// Test Types
// =============================================================================

// Type with all traits
#[derive(Clone, Copy, Debug, Default)]
struct AllTraits;
impl CustomTrait1 for AllTraits {}
impl CustomTrait2 for AllTraits {}
impl CustomTrait3 for AllTraits {}

// Type with some traits
#[derive(Clone, Debug)]
struct SomeTraits;
impl CustomTrait1 for SomeTraits {}

// Type with no traits (except auto traits)
struct NoTraits;

// Type using #[derive(AutoCaps)] (should be honest)
#[derive(Clone, Copy, Debug, Default, AutoCaps)]
struct AutoCapsType;

// Type using #[derive(AutoCaps)] with fewer traits
#[derive(Clone, Debug, AutoCaps)]
struct PartialAutoCapsType;

// =============================================================================
// Part 1: Single Trait Tests (Built-in)
// =============================================================================

mod single_builtin {
    use super::*;

    #[test]
    fn test_clone_positive() {
        assert!(caps_check!(AllTraits: Clone));
        assert!(caps_check!(SomeTraits: Clone));
        assert!(caps_check!(String: Clone));
        assert!(caps_check!(i32: Clone));
    }

    #[test]
    fn test_clone_negative() {
        assert!(!caps_check!(NoTraits: Clone));
    }

    #[test]
    fn test_copy_positive() {
        assert!(caps_check!(AllTraits: Copy));
        assert!(caps_check!(i32: Copy));
        assert!(caps_check!(bool: Copy));
    }

    #[test]
    fn test_copy_negative() {
        assert!(!caps_check!(SomeTraits: Copy));
        assert!(!caps_check!(String: Copy));
        assert!(!caps_check!(NoTraits: Copy));
    }

    #[test]
    fn test_debug_positive() {
        assert!(caps_check!(AllTraits: Debug));
        assert!(caps_check!(SomeTraits: Debug));
        assert!(caps_check!(String: Debug));
    }

    #[test]
    fn test_debug_negative() {
        assert!(!caps_check!(NoTraits: Debug));
    }

    #[test]
    fn test_default_positive() {
        assert!(caps_check!(AllTraits: Default));
        assert!(caps_check!(String: Default));
        assert!(caps_check!(i32: Default));
    }

    #[test]
    fn test_default_negative() {
        assert!(!caps_check!(SomeTraits: Default));
        assert!(!caps_check!(NoTraits: Default));
    }

    #[test]
    fn test_send_sync() {
        // Most types are Send + Sync
        assert!(caps_check!(AllTraits: Send));
        assert!(caps_check!(AllTraits: Sync));
        assert!(caps_check!(i32: Send));
        assert!(caps_check!(i32: Sync));
    }
}

// =============================================================================
// Part 2: Single Trait Tests (Custom)
// =============================================================================

mod single_custom {
    use super::*;

    #[test]
    fn test_custom_trait_positive() {
        assert!(caps_check!(AllTraits: CustomTrait1));
        assert!(caps_check!(AllTraits: CustomTrait2));
        assert!(caps_check!(AllTraits: CustomTrait3));
        assert!(caps_check!(SomeTraits: CustomTrait1));
    }

    #[test]
    fn test_custom_trait_negative() {
        assert!(!caps_check!(SomeTraits: CustomTrait2));
        assert!(!caps_check!(SomeTraits: CustomTrait3));
        assert!(!caps_check!(NoTraits: CustomTrait1));
        assert!(!caps_check!(NoTraits: CustomTrait2));
    }
}

// =============================================================================
// Part 3: AND Expressions
// =============================================================================

mod and_expressions {
    use super::*;

    #[test]
    fn test_builtin_and_builtin() {
        assert!(caps_check!(AllTraits: Clone & Copy));
        assert!(caps_check!(AllTraits: Clone & Debug));
        assert!(caps_check!(i32: Clone & Copy & Debug & Default));

        assert!(!caps_check!(SomeTraits: Clone & Copy)); // SomeTraits is not Copy
        assert!(!caps_check!(String: Clone & Copy)); // String is not Copy
    }

    #[test]
    fn test_custom_and_custom() {
        assert!(caps_check!(AllTraits: CustomTrait1 & CustomTrait2));
        assert!(caps_check!(AllTraits: CustomTrait1 & CustomTrait2 & CustomTrait3));

        assert!(!caps_check!(SomeTraits: CustomTrait1 & CustomTrait2));
    }

    #[test]
    fn test_builtin_and_custom() {
        assert!(caps_check!(AllTraits: Clone & CustomTrait1));
        assert!(caps_check!(SomeTraits: Clone & CustomTrait1));

        assert!(!caps_check!(SomeTraits: Clone & CustomTrait2));
        assert!(!caps_check!(NoTraits: Clone & CustomTrait1));
    }

    #[test]
    fn test_three_way_and() {
        assert!(caps_check!(AllTraits: Clone & Copy & Debug));
        assert!(caps_check!(AllTraits: Clone & CustomTrait1 & CustomTrait2));

        assert!(!caps_check!(SomeTraits: Clone & Copy & Debug));
    }
}

// =============================================================================
// Part 4: OR Expressions
// =============================================================================

mod or_expressions {
    use super::*;

    #[test]
    fn test_builtin_or_builtin() {
        assert!(caps_check!(AllTraits: Clone | Copy));
        assert!(caps_check!(SomeTraits: Clone | Copy)); // Clone is true
        assert!(caps_check!(String: Clone | Copy)); // Clone is true

        // Neither is true
        assert!(!caps_check!(NoTraits: Clone | Copy));
    }

    #[test]
    fn test_custom_or_custom() {
        assert!(caps_check!(AllTraits: CustomTrait1 | CustomTrait2));
        assert!(caps_check!(SomeTraits: CustomTrait1 | CustomTrait2)); // CustomTrait1 is true

        assert!(!caps_check!(NoTraits: CustomTrait1 | CustomTrait2));
    }

    #[test]
    fn test_builtin_or_custom() {
        assert!(caps_check!(SomeTraits: Copy | CustomTrait1)); // CustomTrait1 is true
        assert!(caps_check!(String: Clone | CustomTrait1)); // Clone is true (String doesn't impl CustomTrait1)

        assert!(!caps_check!(NoTraits: Clone | CustomTrait1));
    }
}

// =============================================================================
// Part 5: NOT Expressions
// =============================================================================

mod not_expressions {
    use super::*;

    #[test]
    fn test_not_builtin() {
        assert!(caps_check!(NoTraits: !Clone));
        assert!(caps_check!(SomeTraits: !Copy));
        assert!(caps_check!(String: !Copy));

        assert!(!caps_check!(AllTraits: !Clone));
        assert!(!caps_check!(i32: !Copy));
    }

    #[test]
    fn test_not_custom() {
        assert!(caps_check!(NoTraits: !CustomTrait1));
        assert!(caps_check!(SomeTraits: !CustomTrait2));

        assert!(!caps_check!(AllTraits: !CustomTrait1));
    }

    #[test]
    fn test_double_negation() {
        assert!(caps_check!(AllTraits: !!Clone));
        assert!(!caps_check!(NoTraits: !!Clone));
    }
}

// =============================================================================
// Part 6: Complex Expressions
// =============================================================================

mod complex_expressions {
    use super::*;

    #[test]
    fn test_and_or_combination() {
        // Clone AND (Copy OR Debug)
        assert!(caps_check!(AllTraits: Clone & (Copy | Debug)));
        assert!(caps_check!(SomeTraits: Clone & (Copy | Debug))); // Debug is true

        // (Clone AND Copy) OR Debug
        assert!(caps_check!(AllTraits: (Clone & Copy) | Debug));
        assert!(caps_check!(SomeTraits: (Clone & Copy) | Debug)); // Debug is true
    }

    #[test]
    fn test_not_with_and_or() {
        // Clone AND NOT Copy
        assert!(caps_check!(SomeTraits: Clone & !Copy));
        assert!(caps_check!(String: Clone & !Copy));
        assert!(!caps_check!(AllTraits: Clone & !Copy)); // AllTraits is Copy

        // NOT Clone OR Copy
        assert!(caps_check!(AllTraits: !Clone | Copy)); // Copy is true
        assert!(caps_check!(NoTraits: !Clone | Copy)); // !Clone is true
    }

    #[test]
    fn test_deeply_nested() {
        // ((Clone & Debug) | Copy) & !Default
        assert!(caps_check!(SomeTraits: ((Clone & Debug) | Copy) & !Default));

        // Complex: (Clone & !Copy) | (Debug & Default)
        assert!(caps_check!(SomeTraits: (Clone & !Copy) | (Debug & Default))); // Clone & !Copy
        assert!(caps_check!(AllTraits: (Clone & !Copy) | (Debug & Default))); // Debug & Default
    }

    #[test]
    fn test_all_operators_combined() {
        // Clone & Debug & !Copy | Default
        // Parsed as: ((Clone & Debug) & !Copy) | Default
        assert!(caps_check!(SomeTraits: Clone & Debug & !Copy | Default));
        assert!(caps_check!(AllTraits: Clone & Debug & !Copy | Default)); // Default is true
    }
}

// =============================================================================
// Part 7: #[derive(AutoCaps)] Macro Correctness
// =============================================================================

mod derive_auto_caps_correctness {
    use super::*;

    #[test]
    fn test_auto_caps_matches_reality() {
        // AutoCapsType has Clone, Copy, Debug, Default
        assert!(caps_check!(AutoCapsType: Clone));
        assert!(caps_check!(AutoCapsType: Copy));
        assert!(caps_check!(AutoCapsType: Debug));
        assert!(caps_check!(AutoCapsType: Default));
        assert!(caps_check!(AutoCapsType: Send));
        assert!(caps_check!(AutoCapsType: Sync));
    }

    #[test]
    fn test_partial_auto_caps_matches_reality() {
        // PartialAutoCapsType has Clone, Debug but NOT Copy, Default
        assert!(caps_check!(PartialAutoCapsType: Clone));
        assert!(!caps_check!(PartialAutoCapsType: Copy));
        assert!(caps_check!(PartialAutoCapsType: Debug));
        assert!(!caps_check!(PartialAutoCapsType: Default));
    }

    #[test]
    fn test_std_types_auto_caps_correct() {
        // Verify standard library types have correct AutoCaps
        assert!(caps_check!(String: Clone));
        assert!(!caps_check!(String: Copy));
        assert!(caps_check!(String: Debug));
        assert!(caps_check!(String: Default));

        assert!(caps_check!(i32: Clone));
        assert!(caps_check!(i32: Copy));
        assert!(caps_check!(i32: Debug));
        assert!(caps_check!(i32: Default));

        assert!(caps_check!(bool: Clone));
        assert!(caps_check!(bool: Copy));
        assert!(caps_check!(bool: Debug));
        assert!(caps_check!(bool: Default));
    }
}

// =============================================================================
// Part 8: Generic Context Tests
// =============================================================================

mod generic_context {
    use super::*;

    fn check_clone<T: AutoCapsTrait>() -> bool {
        caps_check!(T: Clone)
    }

    fn check_clone_and_copy<T: AutoCapsTrait>() -> bool {
        caps_check!(T: Clone & Copy)
    }

    fn check_clone_or_copy<T: AutoCapsTrait>() -> bool {
        caps_check!(T: Clone | Copy)
    }

    fn check_not_copy<T: AutoCapsTrait>() -> bool {
        caps_check!(T: !Copy)
    }

    #[test]
    fn test_generic_single_trait() {
        assert!(check_clone::<String>());
        assert!(check_clone::<i32>());
        assert!(check_clone::<AutoCapsType>());
    }

    #[test]
    fn test_generic_and() {
        assert!(check_clone_and_copy::<i32>());
        assert!(check_clone_and_copy::<AutoCapsType>());
        assert!(!check_clone_and_copy::<String>());
        assert!(!check_clone_and_copy::<PartialAutoCapsType>());
    }

    #[test]
    fn test_generic_or() {
        assert!(check_clone_or_copy::<String>()); // Clone
        assert!(check_clone_or_copy::<i32>()); // Both
    }

    #[test]
    fn test_generic_not() {
        assert!(check_not_copy::<String>());
        assert!(check_not_copy::<PartialAutoCapsType>());
        assert!(!check_not_copy::<i32>());
        assert!(!check_not_copy::<AutoCapsType>());
    }
}

// =============================================================================
// Part 9: Edge Cases and Potential Bug Scenarios
// =============================================================================

mod edge_cases {
    use super::*;

    // Test type that shadows a standard trait name
    mod shadow_test {
        #[allow(dead_code)]
        pub trait Clone {} // Shadow std::clone::Clone
    }

    #[test]
    fn test_shadowed_trait_name() {
        // When we write Clone, it should resolve to std::clone::Clone
        // not shadow_test::Clone
        assert!(caps_check!(String: Clone));
    }

    #[test]
    fn test_unit_type() {
        assert!(caps_check!(() : Clone));
        assert!(caps_check!(() : Copy));
        assert!(caps_check!(() : Debug));
        assert!(caps_check!(() : Default));
    }

    #[test]
    fn test_reference_types() {
        // References have different trait impls
        assert!(caps_check!(&str: Clone));
        assert!(caps_check!(&str: Copy));
        assert!(caps_check!(&i32: Clone));
        assert!(caps_check!(&i32: Copy));
    }

    #[test]
    fn test_complex_generic_types() {
        // Option and Result
        assert!(caps_check!(Option<i32>: Clone));
        assert!(caps_check!(Option<i32>: Copy));
        assert!(caps_check!(Result<i32, ()>: Clone));
        assert!(caps_check!(Result<i32, ()>: Copy));

        // Vec (not Copy)
        assert!(caps_check!(Vec<i32>: Clone));
        assert!(!caps_check!(Vec<i32>: Copy));
    }

    #[test]
    fn test_tuple_types() {
        assert!(caps_check!((i32, i32): Clone));
        assert!(caps_check!((i32, i32): Copy));
        assert!(caps_check!((String, i32): Clone));
        assert!(!caps_check!((String, i32): Copy)); // String is not Copy
    }

    #[test]
    fn test_array_types() {
        assert!(caps_check!([i32; 3]: Clone));
        assert!(caps_check!([i32; 3]: Copy));
        assert!(caps_check!([String; 3]: Clone));
        assert!(!caps_check!([String; 3]: Copy));
    }
}

// =============================================================================
// Part 10: False Positive Prevention (Type Doesn't Implement Trait)
// =============================================================================

mod false_positive_prevention {
    use super::*;

    struct ReallyNoTraits;

    #[test]
    fn test_no_false_positives_builtin() {
        assert!(!caps_check!(ReallyNoTraits: Clone));
        assert!(!caps_check!(ReallyNoTraits: Copy));
        assert!(!caps_check!(ReallyNoTraits: Debug));
        assert!(!caps_check!(ReallyNoTraits: Default));
    }

    #[test]
    fn test_no_false_positives_custom() {
        assert!(!caps_check!(ReallyNoTraits: CustomTrait1));
        assert!(!caps_check!(ReallyNoTraits: CustomTrait2));
    }

    #[test]
    fn test_partial_implementation() {
        // Type that only implements some traits
        #[derive(Clone)]
        struct OnlyClone;

        assert!(caps_check!(OnlyClone: Clone));
        assert!(!caps_check!(OnlyClone: Copy));
        assert!(!caps_check!(OnlyClone: Debug));
        assert!(!caps_check!(OnlyClone: Default));
    }
}

// NOTE: Part 11 (probe_or_autocaps_logic) was removed because:
// 1. Manual impl AutoCaps is fragile (50+ constants change with STD_TRAITS)
// 2. Users should never manually implement AutoCaps
// 3. #[derive(AutoCaps)] never lies - it uses the same Probe technique internally

// =============================================================================
// Part 12: Tricky Scenarios - Nested Generics and Type Aliases
// =============================================================================

mod tricky_scenarios {
    use super::*;

    // Type alias tests
    type StringAlias = String;
    type I32Alias = i32;

    #[test]
    fn test_type_aliases() {
        // Type aliases should work the same as the original type
        assert!(caps_check!(StringAlias: Clone));
        assert!(!caps_check!(StringAlias: Copy));
        assert!(caps_check!(I32Alias: Clone));
        assert!(caps_check!(I32Alias: Copy));
    }

    // Nested Option/Result tests
    #[test]
    fn test_deeply_nested_generics() {
        // Option<Option<i32>>
        assert!(caps_check!(Option<Option<i32>>: Clone));
        assert!(caps_check!(Option<Option<i32>>: Copy));

        // Option<Option<String>> - String is not Copy
        assert!(caps_check!(Option<Option<String>>: Clone));
        assert!(!caps_check!(Option<Option<String>>: Copy));

        // Result<Option<i32>, String>
        assert!(caps_check!(Result<Option<i32>, String>: Clone));
        assert!(!caps_check!(Result<Option<i32>, String>: Copy));
    }

    // Box, Rc, Arc tests
    #[test]
    fn test_smart_pointers() {
        use std::rc::Rc;
        use std::sync::Arc;

        // Box<T> is Clone if T: Clone
        assert!(caps_check!(Box<i32>: Clone));
        assert!(!caps_check!(Box<i32>: Copy)); // Box is never Copy

        // Rc<T> is always Clone (doesn't require T: Clone)
        assert!(caps_check!(Rc<NoTraits>: Clone));
        assert!(!caps_check!(Rc<i32>: Copy));

        // Arc<T> is always Clone
        assert!(caps_check!(Arc<NoTraits>: Clone));
        assert!(!caps_check!(Arc<i32>: Copy));
    }

    // Function pointer tests
    #[test]
    fn test_function_pointers() {
        assert!(caps_check!(fn(): Clone));
        assert!(caps_check!(fn(): Copy));
        assert!(caps_check!(fn(i32) -> i32 : Clone));
        assert!(caps_check!(fn(i32) -> i32 : Copy));
    }

    // Raw pointer tests
    #[test]
    fn test_raw_pointers() {
        assert!(caps_check!(*const i32: Clone));
        assert!(caps_check!(*const i32: Copy));
        assert!(caps_check!(*mut i32: Clone));
        assert!(caps_check!(*mut i32: Copy));
    }

    // PhantomData tests
    #[test]
    fn test_phantom_data() {
        use std::marker::PhantomData;

        // PhantomData<T> is always Clone, Copy, etc regardless of T
        assert!(caps_check!(PhantomData<NoTraits>: Clone));
        assert!(caps_check!(PhantomData<NoTraits>: Copy));
        assert!(caps_check!(PhantomData<String>: Copy)); // Even though String is not Copy!
    }
}

// =============================================================================
// Part 13: Generic Function Combinations
// =============================================================================

mod generic_combinations {
    use super::*;

    fn check_complex<T: AutoCapsTrait>() -> (bool, bool, bool, bool) {
        (
            caps_check!(T: Clone),
            caps_check!(T: Clone & Copy),
            caps_check!(T: Clone | Copy),
            caps_check!(T: !Copy),
        )
    }

    #[test]
    fn test_generic_complex_combinations() {
        // i32: Clone=true, Copy=true
        let (clone, and, or, not_copy) = check_complex::<i32>();
        assert!(clone);
        assert!(and);   // Clone & Copy = true
        assert!(or);    // Clone | Copy = true
        assert!(!not_copy); // !Copy = false

        // String: Clone=true, Copy=false
        let (clone, and, or, not_copy) = check_complex::<String>();
        assert!(clone);
        assert!(!and);  // Clone & Copy = false
        assert!(or);    // Clone | Copy = true
        assert!(not_copy); // !Copy = true
    }

    fn check_double_not<T: AutoCapsTrait>() -> bool {
        caps_check!(T: !!Clone)
    }

    #[test]
    fn test_generic_double_not() {
        assert!(check_double_not::<String>());
        assert!(check_double_not::<i32>());
    }

    fn check_de_morgan<T: AutoCapsTrait>() -> (bool, bool) {
        // De Morgan's laws:
        // !(A & B) == !A | !B
        // !(A | B) == !A & !B
        let not_and = caps_check!(T: !(Clone & Copy));
        let or_not = caps_check!(T: !Clone | !Copy);
        (not_and, or_not)
    }

    #[test]
    fn test_de_morgan_laws() {
        // For i32: Clone=true, Copy=true
        // !(true & true) = !true = false
        // !true | !true = false | false = false
        let (not_and, or_not) = check_de_morgan::<i32>();
        assert_eq!(not_and, or_not);
        assert!(!not_and);

        // For String: Clone=true, Copy=false
        // !(true & false) = !false = true
        // !true | !false = false | true = true
        let (not_and, or_not) = check_de_morgan::<String>();
        assert_eq!(not_and, or_not);
        assert!(not_and);
    }
}

// =============================================================================
// Part 14: Boundary Conditions
// =============================================================================

mod boundary_conditions {
    use super::*;

    #[test]
    fn test_zero_sized_types() {
        // () is ZST
        assert!(caps_check!(() : Clone));
        assert!(caps_check!(() : Copy));
        assert!(caps_check!(() : Default));

        // PhantomData is ZST
        use std::marker::PhantomData;
        type P = PhantomData<()>;
        assert!(caps_check!(P: Clone));
        assert!(caps_check!(P: Copy));
    }

    #[test]
    fn test_never_type_workaround() {
        // We can't directly test ! (never type) on stable
        // but we can test Infallible which is similar
        use std::convert::Infallible;
        assert!(caps_check!(Infallible: Clone));
        assert!(caps_check!(Infallible: Copy));
    }

    #[test]
    fn test_large_tuples() {
        // Rust implements traits for tuples up to 12 elements
        type BigTuple = (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32);
        assert!(caps_check!(BigTuple: Clone));
        assert!(caps_check!(BigTuple: Copy));
    }

    #[test]
    fn test_large_arrays() {
        assert!(caps_check!([i32; 100]: Clone));
        assert!(caps_check!([i32; 100]: Copy));
        assert!(caps_check!([String; 100]: Clone));
        assert!(!caps_check!([String; 100]: Copy));
    }
}

// =============================================================================
// Part 15: Regression Tests - Previously Found Bugs
// =============================================================================

mod regression_tests {
    use super::*;

    /// Regression: NOT expression in generic context was broken
    /// The bug was: !Copy for i32 returned true instead of false
    #[test]
    fn test_not_in_generic_context_regression() {
        fn check<T: AutoCapsTrait>() -> bool {
            caps_check!(T: !Copy)
        }

        // i32 is Copy, so !Copy should be false
        assert!(!check::<i32>());

        // String is not Copy, so !Copy should be true
        assert!(check::<String>());
    }

    /// Regression: Complex NOT expressions
    #[test]
    fn test_complex_not_expressions_regression() {
        fn check_not_clone_and_not_copy<T: AutoCapsTrait>() -> bool {
            caps_check!(T: !Clone & !Copy)
        }

        fn check_not_clone_or_copy<T: AutoCapsTrait>() -> bool {
            caps_check!(T: !(Clone | Copy))
        }

        // i32: Clone=true, Copy=true
        // !Clone & !Copy = false & false = false
        assert!(!check_not_clone_and_not_copy::<i32>());

        // !(Clone | Copy) = !(true | true) = !true = false
        assert!(!check_not_clone_or_copy::<i32>());

        // String: Clone=true, Copy=false
        // !Clone & !Copy = false & true = false
        assert!(!check_not_clone_and_not_copy::<String>());

        // !(Clone | Copy) = !(true | false) = !true = false
        assert!(!check_not_clone_or_copy::<String>());
    }

    /// Ensure Probe || AutoCaps correctly handles all NOT cases
    #[test]
    fn test_not_with_probe_autocaps_combination() {
        // Type with honest AutoCaps (using #[derive(AutoCaps)])
        #[derive(Clone, tola_caps::AutoCaps)]
        struct HonestClone;

        // Direct check
        assert!(caps_check!(HonestClone: Clone));
        assert!(!caps_check!(HonestClone: Copy));
        assert!(!caps_check!(HonestClone: !Clone));
        assert!(caps_check!(HonestClone: !Copy));

        // In generic context
        fn check_not_clone<T: AutoCapsTrait>() -> bool {
            caps_check!(T: !Clone)
        }
        fn check_not_copy<T: AutoCapsTrait>() -> bool {
            caps_check!(T: !Copy)
        }

        assert!(!check_not_clone::<HonestClone>());
        assert!(check_not_copy::<HonestClone>());
    }
}


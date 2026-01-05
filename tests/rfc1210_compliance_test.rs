//! Tests for RFC 1210 compliance and specialization patterns
//!
//! This test module verifies that tola-caps specialization implementation
//! follows the patterns and rules described in RFC 1210.

use tola_caps::prelude::*;
use tola_caps::std_caps::AutoCaps;

/// Test RFC 1210 Specificity Ordering
///
/// According to RFC 1210:
/// - Concrete types are most specific
/// - Bounded generics are less specific
/// - Unbounded generics are least specific (default fallback)
mod specificity_ordering {
    use super::*;

    /// Helper trait for testing specificity
    trait SpecificityTest {
        fn specificity_level() -> &'static str;
    }

    // Unbounded generic - least specific (default fallback)
    // Use AutoCaps trait bound which automatically provides all capability checks
    impl<T: AutoCaps> SpecificityTest for T {
        fn specificity_level() -> &'static str {
            "unbounded_generic"
        }
    }

    #[test]
    fn test_unbounded_generic_is_default() {
        // Any type should use the unbounded generic impl
        assert_eq!(<String as SpecificityTest>::specificity_level(), "unbounded_generic");
        // Note: Vec<i32> is not AutoCaps, so we only test String and i32
        assert_eq!(<i32 as SpecificityTest>::specificity_level(), "unbounded_generic");
    }
}

/// Test RFC 1210 Chain Rule
///
/// RFC 1210 uses the "chain rule" for specialization:
/// impl A < impl B < impl C means A is more specific than B, B more specific than C.
/// The chain must be strictly ordered (no partial overlaps).
mod chain_rule {
    use super::*;

    // Test multi-level specialization chain
    // Level 3 (least specific): Any T
    // Level 2: T: Clone
    // Level 1 (most specific): T: Clone + Copy

    #[test]
    fn test_capability_based_selection() {
        // Test that we can detect different capability levels

        // String: Clone but not Copy
        assert!(caps_check!(String: Clone));
        assert!(!caps_check!(String: Copy));

        // i32: Clone and Copy
        assert!(caps_check!(i32: Clone));
        assert!(caps_check!(i32: Copy));

        // Rc<i32>: Clone but not Copy
        assert!(caps_check!(std::rc::Rc<i32>: Clone));
        assert!(!caps_check!(std::rc::Rc<i32>: Copy));
    }
}

/// Test boolean expressions in capability checks
mod boolean_expressions {
    use super::*;

    #[test]
    fn test_and_expression() {
        // Clone AND Copy
        assert!(caps_check!(i32: Clone & Copy));
        assert!(!caps_check!(String: Clone & Copy));
    }

    #[test]
    fn test_or_expression() {
        // Clone OR Copy
        assert!(caps_check!(i32: Clone | Copy));
        assert!(caps_check!(String: Clone | Copy));

        // Neither Clone nor Copy should fail
        struct NoTraits;
        assert!(!caps_check!(NoTraits: Clone | Copy));
    }

    #[test]
    fn test_not_expression() {
        // NOT Copy
        assert!(!caps_check!(i32: !Copy));
        assert!(caps_check!(String: !Copy));
    }

    #[test]
    fn test_complex_expression() {
        // (Clone AND NOT Copy) should match String
        assert!(caps_check!(String: Clone & !Copy));
        // (Clone AND NOT Copy) should not match i32
        assert!(!caps_check!(i32: Clone & !Copy));
    }
}

/// Test associated type specialization patterns
mod associated_type_patterns {
    use super::*;

    // RFC 1210 warns about associated type specialization
    // because types marked with `default` cannot be normalized during type checking

    // Our approach: use type-level selection

    #[allow(dead_code)]
    trait OutputType {
        type Output;
    }

    // Test that we can select different output types based on capabilities
    #[test]
    fn test_type_selection_based_on_cap() {
        // For Clone types, we might want Output = Self
        // For non-Clone types, we might want Output = ()

        // Use caps_check! macro in generic context with AutoCaps bound
        fn clone_detected<T: AutoCaps>() -> bool {
            caps_check!(T: Clone)
        }

        assert!(clone_detected::<String>());
        assert!(clone_detected::<i32>());
        // Note: Vec<u8> is not AutoCaps
    }
}

/// Test generic context behavior
mod generic_context {
    use super::*;

    // Test that capabilities work correctly in generic contexts using caps_check! macro
    // Note: Use AutoCaps trait bound for generic context capability checks

    fn check_clone_in_generic<T: AutoCaps>() -> bool {
        caps_check!(T: Clone)
    }

    fn check_copy_in_generic<T: AutoCaps>() -> bool {
        caps_check!(T: Copy)
    }

    fn check_clone_and_copy_in_generic<T: AutoCaps>() -> bool {
        caps_check!(T: Clone & Copy)
    }

    #[test]
    fn test_generic_clone_detection() {
        assert!(check_clone_in_generic::<String>());
        assert!(check_clone_in_generic::<i32>());
        // Note: Vec<u8> is not AutoCapSet
    }

    #[test]
    fn test_generic_copy_detection() {
        assert!(check_copy_in_generic::<i32>());
        assert!(check_copy_in_generic::<u64>());
        assert!(!check_copy_in_generic::<String>());
    }

    #[test]
    fn test_generic_combined_detection() {
        assert!(check_clone_and_copy_in_generic::<i32>());
        assert!(!check_clone_and_copy_in_generic::<String>());
    }
}

/// Test edge cases from RFC discussions
mod edge_cases {
    use super::*;

    // RFC 1210 mentions several edge cases:
    // 1. Lifetime dispatch is forbidden
    // 2. Associated types with `default` are treated opaquely
    // 3. Marker traits need special handling

    #[test]
    fn test_marker_traits() {
        // Send and Sync are marker traits
        assert!(caps_check!(i32: Send));
        assert!(caps_check!(i32: Sync));

        // Rc is not Send or Sync
        assert!(!caps_check!(std::rc::Rc<i32>: Send));
        assert!(!caps_check!(std::rc::Rc<i32>: Sync));

        // Arc is Send and Sync
        assert!(caps_check!(std::sync::Arc<i32>: Send));
        assert!(caps_check!(std::sync::Arc<i32>: Sync));
    }

    #[test]
    fn test_zero_sized_types() {
        // PhantomData is a ZST
        use std::marker::PhantomData;

        assert!(caps_check!(PhantomData<i32>: Clone));
        assert!(caps_check!(PhantomData<i32>: Copy));
        assert!(caps_check!(PhantomData<i32>: Default));
        assert!(caps_check!(PhantomData<i32>: Send));
        assert!(caps_check!(PhantomData<i32>: Sync));
    }

    #[test]
    fn test_function_pointers() {
        // Function pointers implement Copy but not Clone (in the traditional sense)
        type FnPtr = fn() -> i32;

        assert!(caps_check!(FnPtr: Copy));
        // Clone is auto-implemented for Copy types
        assert!(caps_check!(FnPtr: Clone));
    }

    #[test]
    fn test_nested_generic_types() {
        // Test nested generics like Vec<Option<Box<T>>>
        assert!(caps_check!(Vec<Option<Box<i32>>>: Clone));
        assert!(!caps_check!(Vec<Option<Box<i32>>>: Copy));

        // Option<T> is Clone if T is Clone
        assert!(caps_check!(Option<String>: Clone));
        assert!(!caps_check!(Option<String>: Copy));

        // Option<i32> is both Clone and Copy
        assert!(caps_check!(Option<i32>: Clone));
        assert!(caps_check!(Option<i32>: Copy));
    }
}

/// Test min_specialization patterns
///
/// min_specialization is a sound subset of specialization used in std.
/// Key restriction: can only specialize on "always applicable" impls.
mod min_specialization_patterns {
    use super::*;

    // The "always applicable" rule means:
    // - Specialized impl must be applicable whenever the base impl is
    // - No lifetime-dependent selection
    // - Each generic param appears at most once in unconstrained positions

    #[test]
    fn test_always_applicable_pattern() {
        // Example from RFC:
        // impl<T> SpecExtend<T> for std::vec::IntoIter<T> { /* specialized */ }
        // impl<T, I: Iterator<Item=T>> SpecExtend<T> for I { /* default */ }
        //
        // The specialized impl is "always applicable" because:
        // - IntoIter<T> always implements Iterator<Item=T>
        // - No lifetime constraints

        // Our equivalent test:
        // Clone is always applicable for types that implement Copy
        // (because Copy: Clone is a supertrait relationship)

        // Use AutoCaps bound + caps_check! macro for capability detection
        fn clone_from_copy<T: AutoCaps>() -> bool {
            // If Copy is present, Clone must also be present
            let has_copy = caps_check!(T: Copy);
            let has_clone = caps_check!(T: Clone);

            // Copy implies Clone
            !has_copy || has_clone
        }

        // This should always be true
        assert!(clone_from_copy::<i32>());
        assert!(clone_from_copy::<String>()); // Not Copy, so implication holds
        // Note: Vec<u8> is not AutoCaps
    }
}
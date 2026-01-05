//! Tests for caps_check! macro - UNIFIED SYNTAX
//!
//! `caps_check!(T: Trait)` works for both:
//! - Concrete types (String, i32, etc.)
//! - Generic parameters (T with T: AutoCaps bound)

use std::fmt::Debug;
use tola_caps::caps_check;
use tola_caps::detect::AutoCaps;

// =============================================================================
// Single Trait Tests
// =============================================================================

#[test]
fn test_single_clone() {
    assert!(caps_check!(String: Clone));
    assert!(caps_check!(i32: Clone));
}

#[test]
fn test_single_copy() {
    assert!(caps_check!(i32: Copy));
    assert!(caps_check!(bool: Copy));
    assert!(!caps_check!(String: Copy));
}

#[test]
fn test_single_debug() {
    assert!(caps_check!(String: Debug));
    assert!(caps_check!(i32: Debug));
}

#[test]
fn test_single_default() {
    assert!(caps_check!(String: Default));
    assert!(caps_check!(i32: Default));
    assert!(caps_check!(bool: Default));
}

#[test]
fn test_single_send_sync() {
    assert!(caps_check!(String: Send));
    assert!(caps_check!(String: Sync));
    assert!(caps_check!(i32: Send));
    assert!(caps_check!(i32: Sync));
}

// =============================================================================
// Boolean AND Tests
// =============================================================================

#[test]
fn test_and_clone_copy() {
    // i32 is both Clone and Copy
    assert!(caps_check!(i32: Clone & Copy));

    // String is Clone but not Copy
    assert!(!caps_check!(String: Clone & Copy));
}

#[test]
fn test_and_multiple() {
    // i32 is Clone, Copy, Debug, Default
    assert!(caps_check!(i32: Clone & Copy & Debug));
    assert!(caps_check!(i32: Clone & Copy & Debug & Default));
}

// =============================================================================
// Boolean OR Tests
// =============================================================================

#[test]
fn test_or_clone_copy() {
    // String is Clone (but not Copy), so Clone | Copy is true
    assert!(caps_check!(String: Clone | Copy));

    // i32 is both
    assert!(caps_check!(i32: Clone | Copy));
}

// =============================================================================
// Negation Tests
// =============================================================================

#[test]
fn test_not_copy() {
    // String is not Copy
    assert!(caps_check!(String: !Copy));

    // i32 is Copy
    assert!(!caps_check!(i32: !Copy));
}

#[test]
fn test_clone_and_not_copy() {
    // String is Clone but not Copy
    assert!(caps_check!(String: Clone & !Copy));

    // i32 is Clone AND Copy, so Clone & !Copy is false
    assert!(!caps_check!(i32: Clone & !Copy));
}

// =============================================================================
// Complex Expression Tests
// =============================================================================

#[test]
fn test_complex_parentheses() {
    // (Clone | Copy) & Debug
    // String: (true | false) & true = true
    assert!(caps_check!(String: (Clone | Copy) & Debug));

    // i32: (true | true) & true = true
    assert!(caps_check!(i32: (Clone | Copy) & Debug));
}

#[test]
fn test_complex_mixed() {
    // Clone & (Copy | !Copy) should always be true if Clone is true
    assert!(caps_check!(String: Clone & (Copy | !Copy)));
    assert!(caps_check!(i32: Clone & (Copy | !Copy)));
}

// =============================================================================
// Generic Context Tests - SAME SYNTAX as concrete types!
// =============================================================================

#[test]
fn test_in_generic_context() {
    fn is_clone_and_debug<T: AutoCaps>() -> bool {
        caps_check!(T: Clone & Debug)  // No <T> needed!
    }

    fn is_clone_not_copy<T: AutoCaps>() -> bool {
        caps_check!(T: Clone & !Copy)
    }

    assert!(is_clone_and_debug::<String>());
    assert!(is_clone_and_debug::<i32>());

    assert!(is_clone_not_copy::<String>());
    assert!(!is_clone_not_copy::<i32>());
}

// =============================================================================
// More Complex Generic Tests
// =============================================================================

#[test]
fn test_generic_with_complex_expr() {
    fn has_clone_or_copy<T: AutoCaps>() -> bool {
        caps_check!(T: Clone | Copy)
    }

    fn has_all_basic<T: AutoCaps>() -> bool {
        caps_check!(T: Clone & Debug & Default)
    }

    assert!(has_clone_or_copy::<String>());
    assert!(has_clone_or_copy::<i32>());

    assert!(has_all_basic::<String>());
    assert!(has_all_basic::<i32>());
}

// =============================================================================
// Custom Traits - work with concrete types
// =============================================================================

#[allow(dead_code)]
trait MyCustomTrait {
    fn custom_method(&self);
}

impl MyCustomTrait for String {
    fn custom_method(&self) {}
}

#[test]
fn test_custom_trait_concrete() {
    // Custom traits work on concrete types
    assert!(caps_check!(String: MyCustomTrait));
    assert!(!caps_check!(i32: MyCustomTrait));

    // Combined with std traits
    assert!(caps_check!(String: Clone & MyCustomTrait));
    assert!(!caps_check!(i32: Clone & MyCustomTrait));
}

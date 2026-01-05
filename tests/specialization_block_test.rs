//! Test the specialization! {} block macro
//!
//! This tests the nightly-like syntax for specialization on stable Rust.

use tola_caps::specialization;

// ============================================================================
// TEST 1: Basic specialization! syntax with default fn
// ============================================================================

specialization! {
    trait Describe {
        fn describe() -> &'static str;
    }

    impl<T> Describe for T {
        default fn describe() -> &'static str { "unknown type" }
    }

    impl<T: Clone> Describe for T {
        default fn describe() -> &'static str { "cloneable type" }
    }

    impl<T: Clone + Copy> Describe for T {
        fn describe() -> &'static str { "copyable type" }
    }
}

// Test types
#[derive(tola_caps::AutoCaps)]
struct NoTraits;

#[derive(Clone, tola_caps::AutoCaps)]
struct OnlyClone;

#[derive(Clone, Copy, tola_caps::AutoCaps)]
struct CloneAndCopy;

#[test]
fn test_basic_specialization() {
    // Verify the trait is generated and works for different types

    // Type with no traits -> "unknown type"
    assert_eq!(<NoTraits as Describe>::describe(), "unknown type");

    // Type with Clone only -> "cloneable type"
    assert_eq!(<OnlyClone as Describe>::describe(), "cloneable type");

    // Type with Clone + Copy -> "copyable type"
    assert_eq!(<CloneAndCopy as Describe>::describe(), "copyable type");
}

#[test]
fn test_primitives() {
    // i32 is Clone + Copy, so should get "copyable type"
    assert_eq!(<i32 as Describe>::describe(), "copyable type");

    // String is Clone but not Copy, so should get "cloneable type"
    assert_eq!(<String as Describe>::describe(), "cloneable type");
}

fn main() {}

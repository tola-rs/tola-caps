//! # caps_check! Scenario Matrix and Limitations
//!
//! ## Strategy: Probe || AutoCaps
//!
//! For built-in traits (Clone, Copy, Debug, Default, Send, Sync), we use:
//! ```
//! result = Probe<concrete_type> || AutoCaps<T>::IS_*
//! ```
//!
//! This "Probe || AutoCaps" strategy means:
//! - Probe detects if the **concrete type** implements the trait (always truthful)
//! - AutoCaps carries capability info from **generic bounds**
//! - Either being true means result is true
//!
//! ## Scenario Matrix
//!
//! | Type          | Has AutoCaps? | Trait    | Works? | Notes                        |
//! |---------------|---------------|----------|--------|------------------------------|
//! | Concrete      | No            | Built-in | Yes    | Probe fallback               |
//! | Concrete      | No            | Custom   | Yes    | Probe only                   |
//! | Concrete      | Yes (honest)  | Built-in | Yes    | Probe || AutoCaps            |
//! | Concrete      | Yes (lies FN) | Built-in | Yes    | Probe corrects false negative|
//! | Concrete      | Yes (lies FP) | Built-in | WARN   | Cannot correct false positive|
//! | Concrete      | Yes           | Custom   | Yes    | Probe only                   |
//! | Generic T     | Required      | Built-in | Yes    | AutoCaps carries info        |
//! | Generic T     | Required      | Custom   | No     | Cannot detect at compile time|
//!
//! FN = False Negative (says no but actually has)
//! FP = False Positive (says yes but actually doesn't)
//!
//! ## Key Insight: Why Probe || AutoCaps Works
//!
//! For concrete types with AutoCaps:
//! - If type truly implements trait: Probe=true, AutoCaps=true -> true (correct)
//! - If type lies (false negative): Probe=true, AutoCaps=false -> true (CORRECTED!)
//! - If type lies (false positive): Probe=false, AutoCaps=true -> true (WRONG, unavoidable)
//!
//! For generic T with AutoCaps:
//! - Probe<T> cannot detect traits on generic types, so always false
//! - AutoCaps<T> carries the capability info from the bound
//! - Result: false || AutoCaps = AutoCaps (trusted)
//!
//! ## Known Limitations
//!
//! ### 1. Custom Traits on Generic T
//!
//! `caps_check!(T: CustomTrait)` does NOT work for generic type parameter T.
//! This is fundamental - the Probe pattern only works for concrete types.
//!
//! ### 2. False Positive AutoCaps Cannot Be Corrected
//!
//! If AutoCaps claims a trait is implemented but it isn't (false positive),
//! we cannot detect this. However, this is rare because:
//! - impl_auto_caps! uses the same Probe technique, so it cannot lie
//! - Only manually written AutoCaps impls can have false positives
//!
//! ### Workarounds for Custom Traits on Generics
//!
//! 1. Add the trait as a bound: `T: CustomTrait`
//! 2. Register custom capability in AutoCaps (requires extending the trait)

use tola_caps::caps_check;
use tola_caps::std_caps::AutoCaps;

#[allow(dead_code)]
trait MyCustomTrait {}
impl MyCustomTrait for String {}

// This function demonstrates the limitation
fn check_custom_in_generic<T: AutoCaps>() -> bool {
    caps_check!(T: MyCustomTrait)
}

#[test]
fn test_generic_custom_trait_limitation() {
    // String implements MyCustomTrait
    // But caps_check!(T: MyCustomTrait) returns false for generic T
    let result = check_custom_in_generic::<String>();

    // This is a KNOWN LIMITATION, not a bug to fix
    // The Probe pattern cannot work with generic type parameters
    // because inherent const vs trait const resolution happens at
    // compile time based on the type parameter, not the concrete type
    assert!(!result, "If this passes, the limitation has been fixed!");
}

#[test]
fn test_concrete_custom_trait_works() {
    // Direct concrete type check works fine
    assert!(caps_check!(String: MyCustomTrait));
}

#[test]
fn test_generic_builtin_trait_works() {
    fn check_clone<T: AutoCaps>() -> bool {
        caps_check!(T: Clone)
    }

    // Built-in traits work via AutoCaps
    assert!(check_clone::<String>());
}

// ============================================================
// Additional edge case tests
// ============================================================

#[test]
fn test_mixed_builtin_and_custom() {
    // Mixed expression: Clone (built-in) & MyCustomTrait (custom)
    // Since it contains custom trait, uses Probe for BOTH
    // Works for concrete types
    assert!(caps_check!(String: Clone & MyCustomTrait));

    // Negative case
    assert!(!caps_check!(i32: Clone & MyCustomTrait)); // i32 doesn't impl MyCustomTrait
}

#[test]
fn test_no_autocaps_builtin_works() {
    #[derive(Clone)]
    struct LocalType;

    // LocalType doesn't implement AutoCaps, but Clone check works via Probe fallback
    assert!(caps_check!(LocalType: Clone));
}

#[test]
fn test_no_autocaps_custom_works() {
    struct LocalType;
    impl MyCustomTrait for LocalType {}

    // LocalType doesn't implement AutoCaps, custom trait check works via Probe
    assert!(caps_check!(LocalType: MyCustomTrait));
}

// ============================================================
// AutoCaps Lying Tests
// ============================================================

/// Test that we now prioritize Truth (Probe) over Trust (AutoCaps) for concrete types.
///
/// Improvement: Even if a type implements AutoCaps and lies about a property,
/// if the type is concrete, `caps_check!` uses `Probe || AutoCaps` logic.
/// - Probe detects the implementation (True)
/// - AutoCaps lies (False)
/// - Result: True || False = True (Correct!)
#[test]
fn test_lying_autocaps_concrete_is_now_corrected() {
    // This type implements Clone but lies in AutoCaps
    #[derive(Clone)]
    struct LyingType;

    impl AutoCaps for LyingType {
        // Default is false, so we don't need to specify others
        const IS_CLONE: bool = false; // Explicit lie (though false is default)
        const IS_SEND: bool = true;
        const IS_SYNC: bool = true;
        const IS_SIZED: bool = true;
        const IS_UNPIN: bool = true;
    }

    // Now returns TRUE because Probe detects the truth
    assert!(
        caps_check!(LyingType: Clone),
        "Concrete type check should use Probe logic to CORRECT the AutoCaps lie"
    );
}

/// Test that types WITHOUT AutoCaps use Probe (always truth)
#[test]
fn test_no_autocaps_uses_probe() {
    #[derive(Clone)]
    struct HonestType; // No AutoCaps impl

    // Without AutoCaps, Probe is used and tells the truth
    assert!(caps_check!(HonestType: Clone));
}

/// Test that generic T check trusts AutoCaps (can be wrong)
#[test]
fn test_generic_check_trusts_autocaps() {
    #[derive(Clone)]
    struct LyingType;

    impl AutoCaps for LyingType {
        const IS_CLONE: bool = false; // LIE!
        const IS_SEND: bool = true;
        const IS_SYNC: bool = true;
        const IS_SIZED: bool = true;
        const IS_UNPIN: bool = true;
    }

    fn check_clone<T: AutoCaps>() -> bool {
        caps_check!(T: Clone)
    }

    // Generic check trusts AutoCaps::IS_CLONE, which lies
    // This is EXPECTED behavior - we document this limitation
    assert!(
        !check_clone::<LyingType>(),
        "Generic check trusts AutoCaps, which lies"
    );
}

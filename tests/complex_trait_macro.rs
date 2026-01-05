use tola_caps::prelude::*;
use tola_caps::derive_trait_cap;

// Define a capability for Fn(u8) -> bool using the new syntax
derive_trait_cap!(Fn(u8) -> bool as FnU8Bool);

// Define a capability for Iterator<Item=i32> using alias
derive_trait_cap!(Iterator<Item=i32> as IterI32);

// Define a simple trait alias (trait object style parsing)
derive_trait_cap!(core::fmt::Display as MyDisplay);

#[test]
fn test_complex_trait_macro() {
    // Helper to check capabilities on a type T
    // Since we can't name closure types directly, we use this generic function.
    fn check_fn_u8_bool<T: FnU8Bool>(_: &T) {
        assert!(caps_check!(T: FnU8Bool));
    }

    fn check_iter_i32<T: IterI32>(_: &T) {
        assert!(caps_check!(T: IterI32));
    }

    // Check Fn(u8) -> bool
    fn my_fn(_: u8) -> bool { true }
    check_fn_u8_bool(&my_fn);

    // Check closure
    let closure = |_: u8| true;
    check_fn_u8_bool(&closure);

    // Check Iterator
    let iter = vec![1, 2, 3].into_iter();
    check_iter_i32(&iter);

    // Check Display
    assert!(caps_check!(String: MyDisplay));
    // Vec<u8> generally doesn't implement Display
    assert!(!caps_check!(Vec<u8>: MyDisplay));
}

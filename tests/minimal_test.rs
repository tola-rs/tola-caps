//! Minimal test to find infinite loop

use tola_caps::prelude::*;

// Test 1: Just the Capability derive
#[derive(Capability)]
struct TestCap;

#[test]
fn test_basic() {
    // Just check that the type exists
    let _ = std::any::type_name::<TestCap>();
}

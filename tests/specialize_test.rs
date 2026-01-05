#![allow(clippy::assertions_on_constants)]
//! Tests for Stable Specialization features.

use std::fmt::Debug;
use tola_caps::std_caps::AutoCaps;
use tola_caps::caps_check;

// =============================================================================
// AutoCaps Tests (Constants - always work)
// =============================================================================

#[test]
fn test_autocaps_primitives() {
    // Primitives should be Clone + Copy + Debug + Default + Send + Sync
    assert!(u32::IS_CLONE);
    assert!(u32::IS_COPY);
    assert!(u32::IS_DEBUG);
    assert!(u32::IS_DEFAULT);
    assert!(u32::IS_SEND);
    assert!(u32::IS_SYNC);
}

#[test]
fn test_autocaps_bool() {
    assert!(bool::IS_CLONE);
    assert!(bool::IS_COPY);
}

// =============================================================================
// caps_check! Macro Tests
// =============================================================================

#[derive(Clone, Debug, Copy)]
struct Copyable;

#[derive(Debug, Clone)]
struct CloneableNotCopy;

#[test]
fn test_caps_check_clone() {
    assert!(caps_check!(Copyable: Clone));
    assert!(caps_check!(CloneableNotCopy: Clone));
}

#[test]
fn test_caps_check_debug() {
    assert!(caps_check!(Copyable: Debug));
    assert!(caps_check!(CloneableNotCopy: Debug));
}

#[test]
fn test_caps_check_copy() {
    assert!(caps_check!(Copyable: Copy));
    assert!(!caps_check!(CloneableNotCopy: Copy));
}

#[test]
fn test_clone_not_copy() {
    // Clone: yes, Copy: no
    assert!(caps_check!(CloneableNotCopy: Clone & !Copy));
}

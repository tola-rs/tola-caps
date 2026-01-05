
//! Unified Specification Test
//!
//! Verifies that the AutoCaps detection system correctly feeds into the
//! Capability Trie (Unified Hash Architecture) for both manual and macro-generated types.

use tola_caps::prelude::*;
use tola_caps::std_caps::{Cap, AutoCapSet};
use tola_caps::{Evaluate, Present, Absent};


// =============================================================================
// Helper: Resolve Bool to string
// =============================================================================

trait Resolve {
    fn name() -> &'static str;
}
impl Resolve for Present { fn name() -> &'static str { "Present" } }
impl Resolve for Absent { fn name() -> &'static str { "Absent" } }

// =============================================================================
// Generic Verification Functions
// =============================================================================

fn check_clone<T: AutoCaps + AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsClone>,
    <Cap<T> as Evaluate<IsClone>>::Out: Resolve
{
    <<Cap<T> as Evaluate<IsClone>>::Out as Resolve>::name()
}

#[allow(dead_code)]
fn check_debug<T: AutoCaps + AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsDebug>,
    <Cap<T> as Evaluate<IsDebug>>::Out: Resolve
{
    <<Cap<T> as Evaluate<IsDebug>>::Out as Resolve>::name()
}

// =============================================================================
// Concrete Types
// =============================================================================

// Manual Struct for debugging
#[derive(tola_caps::AutoCaps)]
struct ManualStruct;

// Manual Clone Struct for debugging
#[derive(Clone, tola_caps::AutoCaps)]
struct ManualCloneStruct;

#[derive(Clone, Copy, tola_caps::AutoCaps)]
#[allow(dead_code)]
struct Copyable(i32);

#[derive(Clone, tola_caps::AutoCaps)]
#[allow(dead_code)]
struct CloneableNotCopy(i32);

#[derive(Debug, tola_caps::AutoCaps)]
#[allow(dead_code)]
struct Debuggable(i32);

#[derive(tola_caps::AutoCaps)]
#[allow(dead_code)]
struct Displayable(i32);

use std::fmt;
impl fmt::Display for Displayable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Displayable({})", self.0)
    }
}

// =============================================================================
// Test Cases
// =============================================================================

#[test]
fn test_concrete_specialization() {
    // 0. ManualStruct (Non-Clone) - should return Absent
    assert_eq!(check_clone::<ManualStruct>(), "Absent");

    // 0.5 ManualCloneStruct (Proof that logic works for Clone)
    assert_eq!(check_clone::<ManualCloneStruct>(), "Present");
}

#[cfg(feature = "std")]
#[test]
fn test_std_types() {
    // u32 is Clone + Copy + Debug + Default + Send + Sync
    assert_eq!(check_clone::<u32>(), "Present");

    // f32 is Clone + Copy + Debug + Default + Send + Sync
    assert_eq!(check_clone::<f32>(), "Present");
}

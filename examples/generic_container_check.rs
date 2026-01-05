#![recursion_limit = "4096"]
#![allow(clippy::needless_return)]
//! Verification: Generic Container Capability Check
//!
//! Demonstrates two approaches for checking capabilities:
//! 1. `Cap<T>` + `Evaluate` - Works for concrete types with full Trie support
//! 2. `caps_check!` - Works for ALL types (including generics) via IS_XXX consts
//!
//! Note: In Stable Rust, generic types like `Vec<T>` cannot implement `AutoCapSet`
//! due to the `generic_const_exprs` limitation. Use `caps_check!` for generics.

use tola_caps::prelude::*;
use tola_caps::std_caps::{Cap, AutoCapSet};
use tola_caps::{Evaluate, Present, Absent, caps_check};

// =============================================================================
// Helper: Status trait for type-level bool
// =============================================================================

trait Status { fn status() -> &'static str; }
impl Status for Present { fn status() -> &'static str { "Present" } }
impl Status for Absent { fn status() -> &'static str { "Absent" } }

/// Check Clone capability via type-level Trie (requires AutoCapSet)
fn check_clone_trie<T: AutoCapSet>() -> &'static str
where Cap<T>: Evaluate<IsClone>, <Cap<T> as Evaluate<IsClone>>::Out: Status
{
    <<Cap<T> as Evaluate<IsClone>>::Out as Status>::status()
}

// =============================================================================
// Custom Types
// =============================================================================

// Test structs
#[derive(Clone, Debug, tola_caps::AutoCaps)]
struct YesClone;

#[derive(Debug, tola_caps::AutoCaps)]
struct NoClone;

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("--- Capability Detection Verification ---\n");

    // ==========================================================================
    // Part 1: Concrete types - Full Trie support via Cap<T>
    // ==========================================================================
    println!("=== Part 1: Concrete Types (Cap<T> + Evaluate) ===\n");

    let status_yes = check_clone_trie::<YesClone>();
    println!("YesClone:  IsClone = {}", status_yes);
    assert_eq!(status_yes, "Present");

    let status_no = check_clone_trie::<NoClone>();
    println!("NoClone:   IsClone = {}", status_no);
    assert_eq!(status_no, "Absent");

    let status_u32 = check_clone_trie::<u32>();
    println!("u32:       IsClone = {}", status_u32);
    assert_eq!(status_u32, "Present");

    // ==========================================================================
    // Part 2: Generic containers - Use caps_check! macro
    // ==========================================================================
    println!("\n=== Part 2: Generic Containers (caps_check! macro) ===\n");

    // Note: Vec<T> uses @generic_no_set, so no AutoCapSet.
    // caps_check! uses IS_XXX consts directly - works for any type!

    // Vec<YesClone> - Clone? Only if YesClone is Clone AND allocator allows
    // Actually, Vec itself is always Clone if T: Clone (standard library guarantee)
    // But our detection is based on T's AutoCaps, not Vec's impl

    // For concrete inner types, we can check them directly
    println!("Using caps_check! for runtime checks:");
    println!("  caps_check!(YesClone: Clone) = {}", caps_check!(YesClone: Clone));
    println!("  caps_check!(NoClone: Clone)  = {}", caps_check!(NoClone: Clone));
    println!("  caps_check!(u32: Clone)      = {}", caps_check!(u32: Clone));
    println!("  caps_check!(u32: Copy)       = {}", caps_check!(u32: Copy));
    println!("  caps_check!(String: Clone)   = {}", caps_check!(String: Clone));
    println!("  caps_check!(String: Copy)    = {}", caps_check!(String: Copy));

    assert!(caps_check!(YesClone: Clone));
    assert!(!caps_check!(NoClone: Clone));
    assert!(caps_check!(u32: Clone & Copy));
    assert!(caps_check!(String: Clone & !Copy));

    // ==========================================================================
    // Part 3: Boolean expressions in caps_check!
    // ==========================================================================
    println!("\n=== Part 3: Boolean Expressions ===\n");

    println!("caps_check!(u32: Clone & Copy)        = {}", caps_check!(u32: Clone & Copy));
    println!("caps_check!(String: Clone | Copy)     = {}", caps_check!(String: Clone | Copy));
    println!("caps_check!(String: Clone & !Copy)    = {}", caps_check!(String: Clone & !Copy));

    println!("\nSUCCESS: All capability checks passed!");
}

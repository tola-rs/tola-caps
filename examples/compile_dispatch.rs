//! Compile-Time Generic Dispatch Demonstration
//!
//! This example shows how to use `Cap<T>` to perform compile-time dispatch
//! in generic functions based on trait capabilities.

use tola_caps::std_caps::{AutoCapSet, Cap, IsClone};
use tola_caps::{Evaluate, Present, Absent};
// --- Test Types ---

#[derive(Clone, tola_caps::AutoCaps)]
struct YesClone;

#[derive(tola_caps::AutoCaps)]
struct NoClone;

// =============================================================================
// Compile-Time Dispatch via Const Generics
// =============================================================================

// Strategy: Use `Evaluate::RESULT` as a const bool to select implementation.

fn dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsClone>,
{
    // RESULT is a const bool determined at compile time
    if <Cap<T> as Evaluate<IsClone>>::RESULT {
        "Clone: YES"
    } else {
        "Clone: NO"
    }
}

// =============================================================================
// Type-Level Dispatch via Bool::If
// =============================================================================

// This allows selecting different TYPES at compile time based on capability.

trait Action {
    fn perform() -> &'static str;
}

struct CloneAction;
impl Action for CloneAction {
    fn perform() -> &'static str { "Performing Clone Action" }
}

struct FallbackAction;
impl Action for FallbackAction {
    fn perform() -> &'static str { "Performing Fallback Action" }
}

fn type_dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsClone>,
    <Cap<T> as Evaluate<IsClone>>::Out: DispatchAction,
{
    <<Cap<T> as Evaluate<IsClone>>::Out as DispatchAction>::dispatch()
}

trait DispatchAction {
    fn dispatch() -> &'static str;
}

impl DispatchAction for Present {
    fn dispatch() -> &'static str { CloneAction::perform() }
}

impl DispatchAction for Absent {
    fn dispatch() -> &'static str { FallbackAction::perform() }
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!("=== Compile-Time Generic Dispatch ===\n");

    // Strategy 1: Const bool dispatch
    println!("Strategy 1: Const Bool Dispatch");
    println!("  YesClone: {}", dispatch::<YesClone>());
    println!("  NoClone:  {}", dispatch::<NoClone>());
    assert_eq!(dispatch::<YesClone>(), "Clone: YES");
    assert_eq!(dispatch::<NoClone>(), "Clone: NO");

    // Strategy 2: Type-level dispatch
    println!("\nStrategy 2: Type-Level Dispatch");
    println!("  YesClone: {}", type_dispatch::<YesClone>());
    println!("  NoClone:  {}", type_dispatch::<NoClone>());
    assert_eq!(type_dispatch::<YesClone>(), "Performing Clone Action");
    assert_eq!(type_dispatch::<NoClone>(), "Performing Fallback Action");

    println!("\n=== SUCCESS ===");
}

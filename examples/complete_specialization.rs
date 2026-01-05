//! Complete specialization features demo.

use tola_caps::std_caps::{AutoCapSet, Cap, IsClone, IsCopy, IsDefault};
use tola_caps::{Evaluate, Present, Absent};
use tola_caps::trie::{And, Not, Or};

// Test Types

#[derive(Clone, Copy, Debug, Default, tola_caps::AutoCaps)]
struct FullyCapable;

#[derive(Clone, Debug, tola_caps::AutoCaps)]
struct OnlyCloneDebug;

#[derive(tola_caps::AutoCaps)]
struct NoTraits;

// Feature 1: Positive trait check

fn is_clone<T: AutoCapSet>() -> bool
where
    Cap<T>: Evaluate<IsClone>,
{
    <Cap<T> as Evaluate<IsClone>>::RESULT
}

// Feature 2: Negative trait check (Not<Trait>)

fn is_not_clone<T: AutoCapSet>() -> bool
where
    Cap<T>: Evaluate<Not<IsClone>>,
{
    <Cap<T> as Evaluate<Not<IsClone>>>::RESULT
}

// Feature 3: Compound queries (And, Or)

fn is_clone_and_copy<T: AutoCapSet>() -> bool
where
    Cap<T>: Evaluate<And<IsClone, IsCopy>>,
{
    <Cap<T> as Evaluate<And<IsClone, IsCopy>>>::RESULT
}

fn is_clone_or_default<T: AutoCapSet>() -> bool
where
    Cap<T>: Evaluate<Or<IsClone, IsDefault>>,
{
    <Cap<T> as Evaluate<Or<IsClone, IsDefault>>>::RESULT
}

// Feature 4: Complex compound (Clone AND NOT Copy)

fn is_clone_but_not_copy<T: AutoCapSet>() -> bool
where
    Cap<T>: Evaluate<And<IsClone, Not<IsCopy>>>,
{
    <Cap<T> as Evaluate<And<IsClone, Not<IsCopy>>>>::RESULT
}

// Feature 5: Type-level dispatch via Bool::If

trait Action {
    fn act() -> &'static str;
}

struct CloneAction;
impl Action for CloneAction {
    fn act() -> &'static str { "Clone path" }
}

struct FallbackAction;
impl Action for FallbackAction {
    fn act() -> &'static str { "Fallback path" }
}

trait PickAction {
    type Selected: Action;
}

impl PickAction for Present {
    type Selected = CloneAction;
}

impl PickAction for Absent {
    type Selected = FallbackAction;
}

fn type_dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsClone>,
    <Cap<T> as Evaluate<IsClone>>::Out: PickAction,
{
    <<Cap<T> as Evaluate<IsClone>>::Out as PickAction>::Selected::act()
}

fn main() {
    println!("=== Complete Nightly Specialization on Stable Rust ===\n");

    // Feature 1: Positive check
    println!("1. Positive Trait Check (Is Clone?)");
    println!("   FullyCapable: {}", is_clone::<FullyCapable>());
    println!("   OnlyCloneDebug: {}", is_clone::<OnlyCloneDebug>());
    println!("   NoTraits: {}", is_clone::<NoTraits>());
    assert!(is_clone::<FullyCapable>());
    assert!(is_clone::<OnlyCloneDebug>());
    assert!(!is_clone::<NoTraits>());

    // Feature 2: Negative check
    println!("\n2. Negative Trait Check (Is NOT Clone?)");
    println!("   FullyCapable: {}", is_not_clone::<FullyCapable>());
    println!("   NoTraits: {}", is_not_clone::<NoTraits>());
    assert!(!is_not_clone::<FullyCapable>());
    assert!(is_not_clone::<NoTraits>());

    // Feature 3: Compound queries
    println!("\n3. Compound Queries");
    println!("   Clone AND Copy:");
    println!("     FullyCapable: {}", is_clone_and_copy::<FullyCapable>());
    println!("     OnlyCloneDebug: {}", is_clone_and_copy::<OnlyCloneDebug>());
    assert!(is_clone_and_copy::<FullyCapable>());
    assert!(!is_clone_and_copy::<OnlyCloneDebug>());

    println!("   Clone OR Default:");
    println!("     FullyCapable: {}", is_clone_or_default::<FullyCapable>());
    println!("     NoTraits: {}", is_clone_or_default::<NoTraits>());
    assert!(is_clone_or_default::<FullyCapable>());
    assert!(!is_clone_or_default::<NoTraits>());

    // Feature 4: Clone but NOT Copy
    println!("\n4. Complex Compound (Clone AND NOT Copy)");
    println!("   FullyCapable (has Copy): {}", is_clone_but_not_copy::<FullyCapable>());
    println!("   OnlyCloneDebug (no Copy): {}", is_clone_but_not_copy::<OnlyCloneDebug>());
    assert!(!is_clone_but_not_copy::<FullyCapable>());  // Has Copy, so fails NOT Copy
    assert!(is_clone_but_not_copy::<OnlyCloneDebug>());  // Clone but no Copy

    // Feature 5: Type-level dispatch
    println!("\n5. Type-Level Dispatch");
    println!("   FullyCapable: {}", type_dispatch::<FullyCapable>());
    println!("   NoTraits: {}", type_dispatch::<NoTraits>());
    assert_eq!(type_dispatch::<FullyCapable>(), "Clone path");
    assert_eq!(type_dispatch::<NoTraits>(), "Fallback path");

    println!("\n=== ALL FEATURES VERIFIED ===");
}

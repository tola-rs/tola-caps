//! Pure compile-time type-level specialization.
//!
//! No TypeId, no runtime checks - everything resolved at the type level.

use tola_caps::std_caps::{AutoCapSet, Cap, IsClone, IsCopy};
use tola_caps::{Evaluate, Present, Absent};
use tola_caps::trie::{And, Not};

// Test Types

#[derive(Clone, Copy, Debug, Default, tola_caps::AutoCaps)]
struct FullyCapable;

#[derive(Clone, Debug, tola_caps::AutoCaps)]
struct OnlyCloneDebug;

#[derive(tola_caps::AutoCaps)]
struct NoTraits;

// Const Bool Dispatch
//
// `Evaluate::RESULT` is a `const bool` computed at compile time.
// The compiler eliminates dead branches completely.

fn const_dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsClone>,
{
    // This const is computed at compile time!
    const { <Cap<T> as Evaluate<IsClone>>::RESULT };

    // The if branch is eliminated by the compiler
    if <Cap<T> as Evaluate<IsClone>>::RESULT {
        "COMPILE-TIME: Clone present"
    } else {
        "COMPILE-TIME: Clone absent"
    }
}

// Type-level selection
//
// Select different types based on capability - no runtime at all.

trait Action {
    const NAME: &'static str;
    fn act() -> &'static str { Self::NAME }
}

struct FastAction;
impl Action for FastAction {
    const NAME: &'static str = "FAST ACTION (Clone+Copy)";
}

struct SlowAction;
impl Action for SlowAction {
    const NAME: &'static str = "SLOW ACTION (General)";
}

trait PickAction {
    type Selected: Action;
}

impl PickAction for Present {
    type Selected = FastAction;
}

impl PickAction for Absent {
    type Selected = SlowAction;
}

fn type_level_dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<And<IsClone, IsCopy>>,
    <Cap<T> as Evaluate<And<IsClone, IsCopy>>>::Out: PickAction,
{
    <<Cap<T> as Evaluate<And<IsClone, IsCopy>>>::Out as PickAction>::Selected::act()
}

// Negative bounds

fn not_clone_dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<Not<IsClone>>,
{
    if <Cap<T> as Evaluate<Not<IsClone>>>::RESULT {
        "Type does NOT implement Clone"
    } else {
        "Type DOES implement Clone"
    }
}

// Complex compound queries

fn complex_query<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<And<IsClone, Not<IsCopy>>>,
{
    if <Cap<T> as Evaluate<And<IsClone, Not<IsCopy>>>>::RESULT {
        "Clone but NOT Copy"
    } else {
        "Either not Clone, or has Copy"
    }
}

// Compile-time assertions

const _: () = {
    // FullyCapable has Clone
    assert!(<Cap<FullyCapable> as Evaluate<IsClone>>::RESULT);

    // FullyCapable has Copy
    assert!(<Cap<FullyCapable> as Evaluate<IsCopy>>::RESULT);

    // NoTraits does NOT have Clone
    assert!(!<Cap<NoTraits> as Evaluate<IsClone>>::RESULT);

    // OnlyCloneDebug has Clone but NOT Copy
    assert!(<Cap<OnlyCloneDebug> as Evaluate<And<IsClone, Not<IsCopy>>>>::RESULT);
};

fn main() {
    println!("=== Pure Compile-Time Specialization ===\n");

    println!("Const Bool Dispatch:");
    println!("  FullyCapable: {}", const_dispatch::<FullyCapable>());
    println!("  NoTraits:     {}", const_dispatch::<NoTraits>());

    println!("\nType-Level Selection:");
    println!("  FullyCapable: {}", type_level_dispatch::<FullyCapable>());
    println!("  OnlyCloneDebug: {}", type_level_dispatch::<OnlyCloneDebug>());

    println!("\nNegative Bounds:");
    println!("  FullyCapable: {}", not_clone_dispatch::<FullyCapable>());
    println!("  NoTraits:     {}", not_clone_dispatch::<NoTraits>());

    println!("\nComplex Compound:");
    println!("  FullyCapable:   {}", complex_query::<FullyCapable>());
    println!("  OnlyCloneDebug: {}", complex_query::<OnlyCloneDebug>());

    println!("\n=== All Checks Passed ===");
}

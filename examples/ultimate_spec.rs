//! Ultimate specialization test - all features on stable Rust.

use core::any::TypeId;
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

// TEST 1: TypeId dispatch (optimized at compile time)

fn overlapping_dispatch<T: 'static>(_val: &T) -> &'static str {
    if TypeId::of::<T>() == TypeId::of::<String>() {
        "String: SPECIALIZED PATH"
    } else if TypeId::of::<T>() == TypeId::of::<i32>() {
        "i32: SPECIALIZED PATH"
    } else if TypeId::of::<T>() == TypeId::of::<Vec<u8>>() {
        "Vec<u8>: SPECIALIZED PATH"
    } else {
        "DEFAULT PATH"
    }
}

// TEST 2: TypeId macro dispatch

fn macro_dispatch<T: 'static>() -> &'static str {
    let type_id = core::any::TypeId::of::<T>();
    if type_id == core::any::TypeId::of::<String>() {
        "String via macro"
    } else if type_id == core::any::TypeId::of::<i32>() {
        "i32 via macro"
    } else if type_id == core::any::TypeId::of::<Vec<u8>>() {
        "Vec<u8> via macro"
    } else {
        "Default via macro"
    }
}

// TEST 3: Trait-based capability dispatch via Cap<T>

fn cap_based_dispatch<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<And<IsClone, IsCopy>>,
{
    if <Cap<T> as Evaluate<And<IsClone, IsCopy>>>::RESULT {
        "Clone + Copy: FAST PATH"
    } else {
        "General: SLOW PATH"
    }
}

// TEST 4: Negative bounds (Not<Trait>)

fn not_clone_check<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<Not<IsClone>>,
{
    if <Cap<T> as Evaluate<Not<IsClone>>>::RESULT {
        "NOT Clone: Special handling"
    } else {
        "IS Clone: Normal handling"
    }
}

// TEST 5: Complex compound (Clone AND NOT Copy)

fn clone_not_copy<T: AutoCapSet>() -> bool
where
    Cap<T>: Evaluate<And<IsClone, Not<IsCopy>>>,
{
    <Cap<T> as Evaluate<And<IsClone, Not<IsCopy>>>>::RESULT
}

// TEST 6: Type-level selection

trait SelectPath {
    type Path;
}

struct FastPath;
struct SlowPath;

impl SelectPath for Present {
    type Path = FastPath;
}

impl SelectPath for Absent {
    type Path = SlowPath;
}

fn type_level_select<T: AutoCapSet>() -> &'static str
where
    Cap<T>: Evaluate<IsClone>,
    <Cap<T> as Evaluate<IsClone>>::Out: SelectPath,
{
    // The Path type is selected at compile time
    if TypeId::of::<<<Cap<T> as Evaluate<IsClone>>::Out as SelectPath>::Path>()
        == TypeId::of::<FastPath>()
    {
        "FastPath selected"
    } else {
        "SlowPath selected"
    }
}

// TEST 7: Lifetime marker simulation (TypeId-based)

#[allow(dead_code)]
trait StaticMarker: 'static {}
impl StaticMarker for String {}
impl StaticMarker for i32 {}

fn lifetime_dispatch<T: 'static>() -> &'static str {
    if TypeId::of::<T>() == TypeId::of::<String>()
        || TypeId::of::<T>() == TypeId::of::<i32>()
    {
        "Known static type"
    } else {
        "Unknown static type"
    }
}

fn main() {
    println!("=== Ultimate Spec Tests ===\n");

    // Test 1: Overlapping Impl Simulation
    println!("TEST 1: Overlapping Impl Simulation (TypeId)");
    let s = String::from("hello");
    let n = 42i32;
    let v: Vec<u8> = vec![1, 2, 3];
    let f = 3.14f64;
    println!("  String:  {}", overlapping_dispatch(&s));
    println!("  i32:     {}", overlapping_dispatch(&n));
    println!("  Vec<u8>: {}", overlapping_dispatch(&v));
    println!("  f64:     {}", overlapping_dispatch(&f));
    assert!(overlapping_dispatch(&s).contains("SPECIALIZED"));
    assert!(overlapping_dispatch(&n).contains("SPECIALIZED"));
    assert!(overlapping_dispatch(&f).contains("DEFAULT"));

    // Test 2: specialize! Macro
    println!("\nTEST 2: specialize! Macro");
    println!("  String:  {}", macro_dispatch::<String>());
    println!("  i32:     {}", macro_dispatch::<i32>());
    println!("  f64:     {}", macro_dispatch::<f64>());
    assert!(macro_dispatch::<String>().contains("String"));
    assert!(macro_dispatch::<i32>().contains("i32"));
    assert!(macro_dispatch::<f64>().contains("Default"));

    // Test 3: Capability-Based Dispatch
    println!("\nTEST 3: Capability-Based Dispatch (Cap<T>)");
    println!("  FullyCapable:   {}", cap_based_dispatch::<FullyCapable>());
    println!("  OnlyCloneDebug: {}", cap_based_dispatch::<OnlyCloneDebug>());
    assert!(cap_based_dispatch::<FullyCapable>().contains("FAST"));
    assert!(cap_based_dispatch::<OnlyCloneDebug>().contains("SLOW"));

    // Test 4: Negative Bounds
    println!("\nTEST 4: Negative Bounds (Not<Clone>)");
    println!("  FullyCapable: {}", not_clone_check::<FullyCapable>());
    println!("  NoTraits:     {}", not_clone_check::<NoTraits>());
    assert!(not_clone_check::<FullyCapable>().contains("IS Clone"));
    assert!(not_clone_check::<NoTraits>().contains("NOT Clone"));

    // Test 5: Complex Compound
    println!("\nTEST 5: Complex Compound (Clone AND NOT Copy)");
    println!("  FullyCapable (Clone+Copy):  {}", clone_not_copy::<FullyCapable>());
    println!("  OnlyCloneDebug (Clone only): {}", clone_not_copy::<OnlyCloneDebug>());
    assert!(!clone_not_copy::<FullyCapable>());  // Has Copy
    assert!(clone_not_copy::<OnlyCloneDebug>()); // Clone but no Copy

    // Test 6: Type-Level Selection
    println!("\nTEST 6: Type-Level Selection (GAT-style)");
    println!("  FullyCapable: {}", type_level_select::<FullyCapable>());
    println!("  NoTraits:     {}", type_level_select::<NoTraits>());
    assert!(type_level_select::<FullyCapable>().contains("Fast"));
    assert!(type_level_select::<NoTraits>().contains("Slow"));

    println!("\nTEST 7: Lifetime marker simulation");
    println!("  String: {}", lifetime_dispatch::<String>());
    println!("  f64:    {}", lifetime_dispatch::<f64>());

    println!("\n=== All Tests Passed ===");
}

//! Compile-time specialization via Cap<T> dispatch.
//!
//! Demonstrates generic function `specialize_on_clone<T>` that dispatches
//! based on whether T is Clone, using pure type-level computation.

use tola_caps::prelude::*;
use tola_caps::std_caps::{Cap, AutoCapSet};
use tola_caps::{Evaluate, Present, Absent};

// Trait for specialized behavior
trait DoSomethingSpec {
    fn describe() -> &'static str;
}

// Clone present
impl DoSomethingSpec for Present {
    fn describe() -> &'static str {
        "SPECIALIZED: I am Cloneable! I can be duplicated!"
    }
}

// Clone absent (fallback)
impl DoSomethingSpec for Absent {
    fn describe() -> &'static str {
        "DEFAULT: I am NOT Cloneable. I am unique."
    }
}

/// Dispatches based on Clone capability.
fn specialize_on_clone<T: AutoCaps + AutoCapSet>(_val: &T)
where
    Cap<T>: Evaluate<IsClone>,
    <Cap<T> as Evaluate<IsClone>>::Out: DoSomethingSpec
{
    // Dispatch to the correct implementation
    println!("Type: {}", core::any::type_name::<T>());
    println!("  -> {}", <<Cap<T> as Evaluate<IsClone>>::Out as DoSomethingSpec>::describe());
}

// Test types

// NotClone: custom type without Clone
#[derive(tola_caps::AutoCaps)]
struct NotClone;

fn main() {
    println!("--- Perfect Specialization Demo ---\n");

    let val_u32 = 42u32;
    specialize_on_clone(&val_u32);

    println!();

    let val_struct = NotClone;
    specialize_on_clone(&val_struct);
}

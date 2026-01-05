// This example demonstrates a "TypeMap Registry" approach for specialization.
//
// Instead of relying on static dispatch (which requires trait bounds),
// we use a runtime registry keyed by `TypeId` to store capability information.
//
// This allows `generic<T>` to query capabilities of `T` without explicit bounds,
// provided that `T` has been registered beforehand.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

// Mock registry for demonstration
static REGISTRY: RwLock<Option<HashMap<TypeId, bool>>> = RwLock::new(None);

fn register<T: ?Sized + 'static>(is_clone: bool) {
    let mut lock = REGISTRY.write().unwrap();
    let map = lock.get_or_insert_with(HashMap::new);
    map.insert(TypeId::of::<T>(), is_clone);
}

fn check_registry<T: 'static>() -> bool {
    let lock = REGISTRY.read().unwrap();
    lock.as_ref().and_then(|m| m.get(&TypeId::of::<T>()).copied()).unwrap_or(false)
}

fn user_code() {
    // User registers their type (ONCE)
    register::<MyType>(true);
}

struct MyType;

fn generic_func<T: 'static>() {
    if check_registry::<T>() {
        println!("Generic func knows T is special!");
    } else {
        println!("Generic func knows nothing.");
    }
}

fn main() {
    user_code();
    generic_func::<MyType>();
    generic_func::<i32>(); // Not registered
}

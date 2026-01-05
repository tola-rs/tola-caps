use tola_caps::prelude::*;

// Define two modules with identical struct names
mod a {
    use super::*;
    // Previously NameStream would be "S" -> collision with b::S
    // Now automatic unique ID via module path
    #[derive(Capability)]
    pub struct S;
}

mod b {
    use super::*;
    #[derive(Capability)]
    pub struct S;
}

#[derive(Capability)]
#[allow(dead_code)]
struct CollidingS;

mod c {
    use super::*;
    #[derive(Capability)]
    pub struct S;
}
mod d {
    use super::*;
    #[derive(Capability)]
    pub struct S;
}

#[test]
fn test_name_collision_resolution() {
    // Helper to check capability presence in a SET
    fn check_set<Set, Target>() -> bool
    where
        Target: tola_caps::trie::Capability,
        Set: tola_caps::trie::Evaluate<Target>,
    {
        <Set as tola_caps::trie::Evaluate<Target>>::RESULT
    }

    // 1. a::S and b::S should be DISTINCT automatically (different module paths)
    // Previously required #[cap_name], now automatic.
    // Set{a::S} should NOT contain b::S.
    assert!(!check_set::<caps![a::S], b::S>());
    assert!(check_set::<caps![a::S], a::S>());
    assert!(check_set::<caps![b::S], b::S>());

    // 2. c::S and d::S should ALSO be DISTINCT automatically
    // Previously collided (Ident only). Now automatic via module path.
    // Set{c::S} should NOT contain d::S.
    assert!(!check_set::<caps![c::S], d::S>()); // FAIL -> PASS
    assert!(!check_set::<caps![d::S], c::S>()); // FAIL -> PASS
}

// 3. Verify derive works for subsequent modules (Global Counter logic)
mod e { use super::*; #[derive(Capability)] pub struct S; }
mod f { use super::*; #[derive(Capability)] pub struct S; }

// No manual impl needed - derive does it automatically and uniquely
// tola_caps::impl_capability!(...);

// Implement AutoCapSet globally
impl tola_caps::detect::AutoCapSet for e::S { type Out = tola_caps::trie::Leaf<e::S>; }
impl tola_caps::detect::AutoCapSet for f::S { type Out = tola_caps::trie::Leaf<f::S>; }

#[test]
fn test_manual_path_resolution() {
    // e::S and f::S use manual IDs.
    // They should NOT collide.

    // Helper again inside this function
    fn check<H: tola_caps::detect::AutoCapSet, T: tola_caps::trie::Capability>() -> bool
    where H::Out: tola_caps::trie::Evaluate<T> {
        <H::Out as tola_caps::trie::Evaluate<T>>::RESULT
    }

    assert!(!check::<e::S, f::S>());
    assert!(check::<e::S, e::S>());
}

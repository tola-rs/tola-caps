use tola_caps::prelude::*;
use std::marker::PhantomData;

#[derive(Capability)] struct CapA;
#[derive(Capability)] struct CapB;

struct Wrapper<C>(PhantomData<C>);

// Multi-generic function.
// default caps_bound would pick C1. We explicitly target C2.
#[caps_bound(CapB, target = C2)]
fn target_c2<C1, C2>(_: Wrapper<C1>, _: Wrapper<C2>) {}

#[test]
fn test_explicit_target_compiles() {
    // C1: Has CapA.
    // C2: Has CapB.
    type SetA = caps![CapA];
    type SetB = caps![CapB];

    let w1 = Wrapper::<SetA>(PhantomData);
    let w2 = Wrapper::<SetB>(PhantomData);

    // Should compile because C2 has CapB.
    target_c2(w1, w2);
}

// Ensure it enforces the bound on C2
// Uncommenting specific line should fail if we tested compile_fail,
// but here we just ensure correct compilation of the constraint.

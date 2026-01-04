#![allow(dead_code, unused)]

use tola_caps::prelude::*;
use std::marker::PhantomData;

struct Wrapper<C>(PhantomData<C>);
type Doc<C> = Wrapper<C>;

#[derive(Capability)] struct A;
#[derive(Capability)] struct B;

// Scenario 1: Function requires !B, but we provide B
#[caps_bound(A, !B, transparent)]
fn require_a_no_b(doc: Doc) { let _ = doc; }

// Scenario 2: Function requires A and !A (Logical Impossibility)
#[caps_bound(A, !A, transparent)]
fn impossible_requirements(doc: Doc) { let _ = doc; }

// Scenario X: Complex Logic
#[caps_bound((A | B) & !C, transparent)]
fn complex_logic(doc: Doc) { let _ = doc; }

#[test]
fn test_conflict_diagnosis() {
    // Case 1: Set has A and B. We call 'require_a_no_b' (Requires !B).
    type SetAB = caps![A, B];
    let doc_ab = Wrapper::<SetAB>(PhantomData);

    // require_a_no_b(doc_ab);

    // Case 2: Set has A. We call 'impossible' (Requires A & !A).
    type SetA = caps![A];
    let doc_a = Wrapper::<SetA>(PhantomData);

    // impossible_requirements(doc_a);

    // Case 3: Complex Logic (A | B) & !C
    // SetA has A (satisfies A|B). It does not have C (satisfies !C).
    // So SetA should PASS complex_logic.

    // SetC has C. Fails !C condition.
    type SetC = caps![C];
    let doc_c = Wrapper::<SetC>(PhantomData);
    // complex_logic(doc_c);
}

// Scenario 3: Bad State Transition
// We add C but forget to remove A. Next step requires !A.
#[derive(Capability)] struct C;

#[caps_bound(A, with(C), transparent)]
fn partial_transition(doc: Doc) -> Doc<with![C]> {
    // Should use without![A] but didn't!
    Wrapper(PhantomData)
}

#[caps_bound(!A, C, transparent)]
fn final_step(doc: Doc) { let _ = doc; }

#[test]
fn test_transition_conflict() {
    type SetA = caps![A];
    let doc = Wrapper::<SetA>(PhantomData);

    let doc_next = partial_transition(doc);
    // doc_next is {A, C}.
    // final_step requires !A.

    // final_step(doc_next);
}

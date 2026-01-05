#![allow(dead_code, unused)]

use tola_caps::prelude::*;
use tola_caps::{with, without};
use std::marker::PhantomData;

#[derive(Capability)]
pub struct PublicCap;

#[derive(Capability)]
struct PrivateCap;

// =============================================================================
// 1. Function Tests
// =============================================================================

#[caps_bound(requires = PublicCap)]
pub fn public_func<C>(_: C) {}

#[caps_bound(requires = PublicCap)]
pub(crate) fn crate_func<C>(_: C) {}

#[caps_bound(requires = PublicCap)]
fn private_func<C>(_: C) {}

// =============================================================================
// 2. Struct Tests
// =============================================================================

#[caps_bound(requires = PublicCap)]
pub struct PublicStruct<C> {
    _c: PhantomData<C>,
}

#[caps_bound(requires = PublicCap)]
pub(crate) struct CrateStruct<C> {
    _c: PhantomData<C>,
}

#[caps_bound(requires = PublicCap)]
struct PrivateStruct<C> {
    _c: PhantomData<C>,
}

// =============================================================================
// 3. Enum Tests
// =============================================================================

#[caps_bound(requires = PublicCap)]
pub enum PublicEnum<C> {
    Variant(PhantomData<C>),
}

#[caps_bound(requires = PublicCap)]
enum PrivateEnum<C> {
    Variant(PhantomData<C>),
}

// =============================================================================
// 4. Impl Block Tests
// =============================================================================

pub struct Data<C>(PhantomData<C>);

#[caps_bound(requires = PublicCap)]
impl<C> Data<C> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C> Default for Data<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<C> Data<C> {
    pub fn method(&self) {}
}

// Test conflicting caps in impl block
#[derive(Capability)]
pub struct ConflictingCap;

#[caps_bound(requires = PublicCap, conflicts = ConflictingCap)]
impl<C> Data<C> {
    pub fn safe_method(&self) {}
}

#[test]
fn test_macros_rename() {
    type Set = caps![PublicCap];
    type NewSet = with![Set, PrivateCap];
    // This verifies that caps! and with! expand to types using the 'With' trait (which must exist)
    let _ = PhantomData::<NewSet>;
}

struct Wrapper<C>(PhantomData<C>);
type Doc<C> = Wrapper<C>;

#[caps_bound(requires = PublicCap, with = PrivateCap, transparent = true)]
fn magical_test(w: Doc) -> Doc<with![PrivateCap]> {
    // Generates: fn magical_test<__C>(w: Doc<__C>) -> Doc<<__C as With<PrivateCap>>::Out>
    // where __C: Evaluate<PublicCap>, __C: With<PrivateCap>
    Wrapper(PhantomData)
}

#[derive(Capability)] struct CapA;
#[derive(Capability)] struct CapB;

// Test flat syntax: CapB (!CapA implicit conflict), transparent
#[caps_bound(CapB, !CapA, transparent)]
fn verify_removal(w: Doc) {
   let _ = w;
}

// Test grouped syntax: without(CapA)
#[caps_bound(CapA & CapB, without(CapA), transparent)]
fn test_bound_without(w: Doc) -> Doc<without![CapA]> {
    // Input has A and B. Return type removes A.
    // The 'without = CapA' argument adds 'C: Without<CapA>' bound.
    Wrapper(PhantomData)
}

#[test]
fn test_without_macro() {
    // Initial set has A and B
    type SetAB = caps![CapA, CapB];
    let w = Wrapper::<SetAB>(PhantomData);

    // Call function that removes A
    let w_b = test_bound_without(w);

    // Verify w_b satisfies 'requires = CapB, conflicts = CapA'
    verify_removal(w_b);
}

// Arbitrary positional arguments test
#[caps_bound(CapA, CapB, !PrivateCap, transparent)]
fn verify_arbitrary_positional(doc: Doc) {
    let _ = doc;
}

#[test]
fn test_arbitrary_positional_support() {
    type Set = caps![CapA, CapB];
    let doc = Wrapper::<Set>(PhantomData);
    verify_arbitrary_positional(doc);
}

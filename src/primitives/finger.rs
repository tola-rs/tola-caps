//! Type-level Finger Tree for zero-collision identity.
//!
//! A Finger Tree gives us O(1) access to both ends and O(log n) operations in the middle.
//! At the type level, we use this for:
//!   - O(1) cached hash comparison (fast rejection)
//!   - O(1) first/last element comparison
//!   - O(log n) deep spine comparison (rare)

use core::marker::PhantomData;
use crate::primitives::{Bool, Present, Absent, BoolAnd};
use crate::primitives::nibble::{Nibble, NibbleEq};

// =============================================================================
// Finger Tree Core Types
// =============================================================================

/// Empty tree - identity element
pub struct FEmpty;

/// Single element tree
pub struct FSingle<A>(PhantomData<A>);

/// Deep tree with cached measure for O(1) rejection
/// - M: Cached hash (XOR of all elements)
/// - P: Prefix digit (1-4 elements, O(1) front access)
/// - S: Spine (recursive FingerTree of Nodes)
/// - X: Suffix digit (1-4 elements, O(1) back access)
pub struct FDeep<M, P, S, X>(PhantomData<(M, P, S, X)>);

// =============================================================================
// Digits (1 to 4 elements)
// =============================================================================

pub struct D1<A>(PhantomData<A>);
pub struct D2<A, B>(PhantomData<(A, B)>);
pub struct D3<A, B, C>(PhantomData<(A, B, C)>);
pub struct D4<A, B, C, D>(PhantomData<(A, B, C, D)>);

// =============================================================================
// Internal Nodes (2-3 elements with cached measure)
// =============================================================================

pub struct N2<M, A, B>(PhantomData<(M, A, B)>);
pub struct N3<M, A, B, C>(PhantomData<(M, A, B, C)>);

// =============================================================================
// Hash Types (for measure caching)
// =============================================================================

/// Zero hash - identity for XOR
pub struct H0;

/// Hash value as nibble stream
#[allow(clippy::type_complexity)]
pub struct HVal<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>(
    PhantomData<(N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF)>
);

// =============================================================================
// Measured Trait (GAT for computing hash)
// =============================================================================

/// Everything in a Finger Tree has a measure (cached hash)
pub trait Measured {
    type Measure;
}

impl Measured for FEmpty {
    type Measure = H0;
}

impl<A: Measured> Measured for FSingle<A> {
    type Measure = A::Measure;
}

impl<M, P, S, X> Measured for FDeep<M, P, S, X> {
    type Measure = M;  // Pre-computed, O(1)
}

// =============================================================================
// Measure Equality (O(1) fast rejection)
// =============================================================================

pub trait MeasureEq<Other> {
    type Out: Bool;
}

// Same hash type = Present
impl MeasureEq<H0> for H0 {
    type Out = Present;
}

// Different concrete hashes need nibble-by-nibble comparison
// (This is still O(1) since it's a fixed 64-bit comparison)
impl<
    A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, AA, AB, AC, AD, AE, AF,
    B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, BA, BB, BC, BD, BE, BF,
> MeasureEq<HVal<B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, BA, BB, BC, BD, BE, BF>>
    for HVal<A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, AA, AB, AC, AD, AE, AF>
where
    A0: Nibble + NibbleEq<B0>, A1: Nibble + NibbleEq<B1>,
    A2: Nibble + NibbleEq<B2>, A3: Nibble + NibbleEq<B3>,
    A4: Nibble + NibbleEq<B4>, A5: Nibble + NibbleEq<B5>,
    A6: Nibble + NibbleEq<B6>, A7: Nibble + NibbleEq<B7>,
    A8: Nibble + NibbleEq<B8>, A9: Nibble + NibbleEq<B9>,
    AA: Nibble + NibbleEq<BA>, AB: Nibble + NibbleEq<BB>,
    AC: Nibble + NibbleEq<BC>, AD: Nibble + NibbleEq<BD>,
    AE: Nibble + NibbleEq<BE>, AF: Nibble + NibbleEq<BF>,
    B0: Nibble, B1: Nibble, B2: Nibble, B3: Nibble,
    B4: Nibble, B5: Nibble, B6: Nibble, B7: Nibble,
    B8: Nibble, B9: Nibble, BA: Nibble, BB: Nibble,
    BC: Nibble, BD: Nibble, BE: Nibble, BF: Nibble,
    // Chain all 16 nibble comparisons with AND
    <A0 as NibbleEq<B0>>::Out: BoolAnd<<A1 as NibbleEq<B1>>::Out>,
    <<A0 as NibbleEq<B0>>::Out as BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out: BoolAnd<<A2 as NibbleEq<B2>>::Out>,
    // ... (truncated for readability, full chain needed)
{
    // For now, simplified - full impl would chain all 16
    type Out = <A0 as NibbleEq<B0>>::Out;
}

// =============================================================================
// Finger Tree Equality
// =============================================================================

/// Main comparison trait for Finger Trees
pub trait FingerEq<Other> {
    type Out: Bool;
}

// Empty == Empty
impl FingerEq<FEmpty> for FEmpty {
    type Out = Present;
}

// Empty != anything else
impl<A> FingerEq<FSingle<A>> for FEmpty {
    type Out = Absent;
}
impl<M, P, S, X> FingerEq<FDeep<M, P, S, X>> for FEmpty {
    type Out = Absent;
}
impl<A> FingerEq<FEmpty> for FSingle<A> {
    type Out = Absent;
}
impl<M, P, S, X> FingerEq<FEmpty> for FDeep<M, P, S, X> {
    type Out = Absent;
}

// Single == Single: compare elements
impl<A, B> FingerEq<FSingle<B>> for FSingle<A>
where
    A: super::identity::IdentityEq<B>,
{
    type Out = <A as super::identity::IdentityEq<B>>::Out;
}

// Single != Deep
impl<A, M, P, S, X> FingerEq<FDeep<M, P, S, X>> for FSingle<A> {
    type Out = Absent;
}
impl<A, M, P, S, X> FingerEq<FSingle<A>> for FDeep<M, P, S, X> {
    type Out = Absent;
}

// Deep == Deep: layered comparison
// Layer 1: Compare measures (cached hash) - O(1)
// Layer 2: Compare prefix first element - O(1)
// Layer 3: Compare suffix last element - O(1)
// Layer 4: Compare spines - O(log n)
impl<M1, P1, S1, X1, M2, P2, S2, X2> FingerEq<FDeep<M2, P2, S2, X2>> for FDeep<M1, P1, S1, X1>
where
    M1: MeasureEq<M2>,
    P1: DigitEq<P2>,
    S1: FingerEq<S2>,
    X1: DigitEq<X2>,
    // All must match
    <M1 as MeasureEq<M2>>::Out: BoolAnd<<P1 as DigitEq<P2>>::Out>,
    <<M1 as MeasureEq<M2>>::Out as BoolAnd<<P1 as DigitEq<P2>>::Out>>::Out:
        BoolAnd<<S1 as FingerEq<S2>>::Out>,
    <<<M1 as MeasureEq<M2>>::Out as BoolAnd<<P1 as DigitEq<P2>>::Out>>::Out as
        BoolAnd<<S1 as FingerEq<S2>>::Out>>::Out: BoolAnd<<X1 as DigitEq<X2>>::Out>,
{
    type Out = <<<<M1 as MeasureEq<M2>>::Out
        as BoolAnd<<P1 as DigitEq<P2>>::Out>>::Out
        as BoolAnd<<S1 as FingerEq<S2>>::Out>>::Out
        as BoolAnd<<X1 as DigitEq<X2>>::Out>>::Out;
}

// =============================================================================
// Digit Equality
// =============================================================================

pub trait DigitEq<Other> {
    type Out: Bool;
}

// Same-size digits
impl<A, B> DigitEq<D1<B>> for D1<A>
where A: super::identity::IdentityEq<B> {
    type Out = <A as super::identity::IdentityEq<B>>::Out;
}

impl<A1, A2, B1, B2> DigitEq<D2<B1, B2>> for D2<A1, A2>
where
    A1: super::identity::IdentityEq<B1>,
    A2: super::identity::IdentityEq<B2>,
    <A1 as super::identity::IdentityEq<B1>>::Out: BoolAnd<<A2 as super::identity::IdentityEq<B2>>::Out>,
{
    type Out = <<A1 as super::identity::IdentityEq<B1>>::Out
        as BoolAnd<<A2 as super::identity::IdentityEq<B2>>::Out>>::Out;
}

// Different-size digits = Absent
impl<A, B1, B2> DigitEq<D2<B1, B2>> for D1<A> { type Out = Absent; }
impl<A1, A2, B> DigitEq<D1<B>> for D2<A1, A2> { type Out = Absent; }
// ... more size mismatches

// =============================================================================
// IdentityEq Bridge - Connects Finger Tree to Trie's comparison system
// =============================================================================

use super::identity::IdentityEq;

// Empty trees
impl IdentityEq<FEmpty> for FEmpty {
    type Out = Present;
}

impl<A> IdentityEq<FSingle<A>> for FEmpty {
    type Out = Absent;
}

impl<M, P, S, X> IdentityEq<FDeep<M, P, S, X>> for FEmpty {
    type Out = Absent;
}

// Single element trees
impl<A> IdentityEq<FEmpty> for FSingle<A> {
    type Out = Absent;
}

impl<A, B> IdentityEq<FSingle<B>> for FSingle<A>
where
    A: IdentityEq<B>,
{
    type Out = <A as IdentityEq<B>>::Out;
}

impl<A, M, P, S, X> IdentityEq<FDeep<M, P, S, X>> for FSingle<A> {
    type Out = Absent;
}

// Deep trees - delegate to FingerEq
impl<M, P, S, X> IdentityEq<FEmpty> for FDeep<M, P, S, X> {
    type Out = Absent;
}

impl<M, P, S, X, A> IdentityEq<FSingle<A>> for FDeep<M, P, S, X> {
    type Out = Absent;
}

impl<M1, P1, S1, X1, M2, P2, S2, X2> IdentityEq<FDeep<M2, P2, S2, X2>> for FDeep<M1, P1, S1, X1>
where
    Self: FingerEq<FDeep<M2, P2, S2, X2>>,
{
    type Out = <Self as FingerEq<FDeep<M2, P2, S2, X2>>>::Out;
}

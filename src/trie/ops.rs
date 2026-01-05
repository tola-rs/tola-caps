//! Set operations on capability tries: Union, Intersect, SupersetOf, SetAnd
//!
//! These traits enable combining and comparing capability sets at the type level.

use crate::primitives::{Present, Absent, Bool};
use crate::primitives::stream::{StreamEq, DefaultMaxDepth};
use super::node::{Empty, Leaf, Node16};
use super::capability::Capability;
use super::insert::With;

// =============================================================================
// Set Operations Traits
// =============================================================================

/// Set Union: Merge two capability sets.
/// Returns a set containing all capabilities from both A and B.
pub trait SetUnion<Other> {
    type Out;
}

/// Set Intersection: Common capabilities between two sets.
/// Returns a set containing only capabilities present in both A and B.
pub trait SetIntersect<Other> {
    type Out;
}

/// SupersetOf: Check if Self contains all capabilities in Other.
/// Used for downcasting / forgetting extra capabilities.
pub trait SupersetOf<Other>: Sized {}

/// Structural set intersection (Cap1 & Cap2).
pub trait SetAnd<Other> {
    type Out;
}

// =============================================================================
// SetUnion Implementations
// =============================================================================

// Empty ∪ X = X
impl<T> SetUnion<T> for Empty {
    type Out = T;
}

// Leaf<A> ∪ Empty = Leaf<A>
impl<A> SetUnion<Empty> for Leaf<A> {
    type Out = Leaf<A>;
}

// Node16 ∪ Empty = Node16
#[macros::node16]
impl<_Slots_> SetUnion<Empty> for _Node16_ {
    type Out = Self;
}

// Leaf<A> ∪ Leaf<B> = Insert B into Leaf<A>
impl<A, B> SetUnion<Leaf<B>> for Leaf<A>
where
    B: Capability,
    Leaf<A>: With<B>,
{
    type Out = <Leaf<A> as With<B>>::Out;
}

// Leaf<A> ∪ Node16<...> = Insert A into Node16
#[macros::node16]
impl<A, _Slots_> SetUnion<_Node16_> for Leaf<A>
where
    A: Capability,
    _Node16_: With<A>,
{
    type Out = <_Node16_ as With<A>>::Out;
}

// Node16 ∪ Leaf<A> = Insert A into Node16
#[macros::node16]
impl<A, _Slots_> SetUnion<Leaf<A>> for _Node16_
where
    A: Capability,
    Self: With<A>,
{
    type Out = <Self as With<A>>::Out;
}

// =============================================================================
// SetIntersect Implementations
// =============================================================================

// Empty ∩ X = Empty
impl<T> SetIntersect<T> for Empty {
    type Out = Empty;
}

// X ∩ Empty = Empty
impl<A> SetIntersect<Empty> for Leaf<A> {
    type Out = Empty;
}

#[macros::node16]
impl<_Slots_> SetIntersect<Empty> for _Node16_ {
    type Out = Empty;
}

// Leaf<A> ∩ Leaf<B> = Leaf<A> if A == B, else Empty
impl<A, B> SetIntersect<Leaf<B>> for Leaf<A>
where
    A: Capability,
    B: Capability,
    A::Stream: StreamEq<B::Stream, DefaultMaxDepth>,
    <A::Stream as StreamEq<B::Stream, DefaultMaxDepth>>::Out: IntersectLeafHelper<A>,
{
    type Out = <<A::Stream as StreamEq<B::Stream, DefaultMaxDepth>>::Out as IntersectLeafHelper<A>>::Out;
}

/// Helper for conditional Leaf intersection result
pub trait IntersectLeafHelper<A> {
    type Out;
}

impl<A> IntersectLeafHelper<A> for Present {
    type Out = Leaf<A>;  // Same capability
}

impl<A> IntersectLeafHelper<A> for Absent {
    type Out = Empty;  // Different capabilities
}

// =============================================================================
// SupersetOf Implementations
// =============================================================================

// Everything is a superset of Empty
impl<T> SupersetOf<Empty> for T {}

// Leaf<A> is superset of Leaf<A> (identity)
impl<A> SupersetOf<Leaf<A>> for Leaf<A> {}

// Node16 is superset of Leaf<A> if it contains A
use super::evaluate::{Evaluate, Has};

#[macros::node16]
impl<A, _Slots_> SupersetOf<Leaf<A>> for _Node16_
where
    A: Capability,
    Self: Evaluate<Has<A>, Out = Present>,
{}

// =============================================================================
// SetAnd Implementations (Structural Intersection)
// =============================================================================

// Empty & Anything = Empty
impl<T> SetAnd<T> for Empty {
    type Out = Empty;
}

// Leaf & ... (dispatch based on RHS)
impl<C, Other> SetAnd<Other> for Leaf<C>
where
    Other: LeafAndDispatch<C>,
{
    type Out = <Other as LeafAndDispatch<C>>::Out;
}

pub trait LeafAndDispatch<LLeafCap> {
    type Out;
}

// Leaf & Empty = Empty
impl<C> LeafAndDispatch<C> for Empty {
    type Out = Empty;
}

// Leaf & Leaf = Leaf IF same cap, else Empty
impl<C1, C2> LeafAndDispatch<C1> for Leaf<C2>
where
    C1: Capability,
    C2: Capability,
    C1::Stream: StreamEq<C2::Stream, DefaultMaxDepth>,
{
    type Out = <<C1::Stream as StreamEq<C2::Stream, DefaultMaxDepth>>::Out as Bool>::If<Leaf<C1>, Empty>;
}

// Leaf & Node = Empty
#[macros::node16]
impl<C, _Slots_> LeafAndDispatch<C> for _Node16_ {
    type Out = Empty;
}

// Node & ... (dispatch based on RHS)
#[macros::node16]
impl<R, _Slots_> SetAnd<R> for _Node16_
where
    R: NodeAndDispatch<_Slots_>,
{
    type Out = <R as NodeAndDispatch<_Slots_>>::Out;
}

pub trait NodeAndDispatch<L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF> {
    type Out;
}

// Node & Empty = Empty
impl<L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF>
    NodeAndDispatch<L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF>
    for Empty
{
    type Out = Empty;
}

// Node & Leaf = Empty
impl<C, L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF>
    NodeAndDispatch<L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF>
    for Leaf<C>
{
    type Out = Empty;
}

// Node & Node = Node (Slot-by-slot structural intersection)
#[allow(clippy::type_complexity)]
impl<
    R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, RA, RB, RC, RD, RE, RF,
    L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF,
>
    NodeAndDispatch<L0, L1, L2, L3, L4, L5, L6, L7, L8, L9, LA, LB, LC, LD, LE, LF>
    for Node16<R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, RA, RB, RC, RD, RE, RF>
where
    L0: SetAnd<R0>, L1: SetAnd<R1>, L2: SetAnd<R2>, L3: SetAnd<R3>,
    L4: SetAnd<R4>, L5: SetAnd<R5>, L6: SetAnd<R6>, L7: SetAnd<R7>,
    L8: SetAnd<R8>, L9: SetAnd<R9>, LA: SetAnd<RA>, LB: SetAnd<RB>,
    LC: SetAnd<RC>, LD: SetAnd<RD>, LE: SetAnd<RE>, LF: SetAnd<RF>,
{
    type Out = Node16<
        <L0 as SetAnd<R0>>::Out, <L1 as SetAnd<R1>>::Out,
        <L2 as SetAnd<R2>>::Out, <L3 as SetAnd<R3>>::Out,
        <L4 as SetAnd<R4>>::Out, <L5 as SetAnd<R5>>::Out,
        <L6 as SetAnd<R6>>::Out, <L7 as SetAnd<R7>>::Out,
        <L8 as SetAnd<R8>>::Out, <L9 as SetAnd<R9>>::Out,
        <LA as SetAnd<RA>>::Out, <LB as SetAnd<RB>>::Out,
        <LC as SetAnd<RC>>::Out, <LD as SetAnd<RD>>::Out,
        <LE as SetAnd<RE>>::Out, <LF as SetAnd<RF>>::Out,
    >;
}

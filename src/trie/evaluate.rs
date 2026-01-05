//! Query types and evaluation logic
//!
//! Provides Evaluate trait and boolean query types (Has, And, Or, Not).

use core::marker::PhantomData;
use crate::primitives::Peano;
use crate::primitives::{Bool, Present, Absent, GetTail, BoolAnd, BoolOr, BoolNot};
use crate::primitives::stream::{S, D0};
use super::node::{Empty, Leaf, Node16};
use super::capability::Capability;

// =============================================================================
// Query Types
// =============================================================================

/// Query: Does the set contain capability Cap?
pub struct Has<Cap>(PhantomData<Cap>);

/// Conjunction: L AND R
pub struct And<L, R>(PhantomData<(L, R)>);

/// Disjunction: L OR R
pub struct Or<L, R>(PhantomData<(L, R)>);

/// Negation: NOT Q
pub struct Not<Q>(PhantomData<Q>);

// =============================================================================
// HList for All/Any
// =============================================================================

/// Empty HList
pub struct HNil;

/// HList cons cell
pub struct HCons<H, T>(PhantomData<(H, T)>);

/// All queries must be true (conjunction)
pub struct All<List>(PhantomData<List>);

/// At least one query must be true (disjunction)
pub struct Any<List>(PhantomData<List>);

// =============================================================================
// Evaluate (Main Entry Point)
// =============================================================================

/// Evaluate a boolean query on a capability set.
///
/// Returns `Present` (true) or `Absent` (false).
#[diagnostic::on_unimplemented(
    message = "Capability logic requirement evaluated to false or is invalid",
    label = "Logic '{Query}' is NOT satisfied by capability set '{Self}'",
    note = "Check if you are missing a required capability or possess a conflicting one."
)]
pub trait Evaluate<Query> {
    type Out: Bool;
    /// The boolean result of the evaluation as a constant.
    const RESULT: bool = <Self::Out as Bool>::VALUE;
}

// =============================================================================
// EvalAt - Internal depth-aware evaluation
// =============================================================================

/// Internal trait for evaluating queries at a specific depth
pub trait EvalAt<Query, Depth> {
    type Out: Bool;
}

impl<Cap, Depth> EvalAt<Has<Cap>, Depth> for Empty {
    type Out = Absent;
}

// Leaf: Two-tier matching (Identity perfect + Stream fallback)
// - Same Identity64 type → Stream also matches → Present (perfect!)
// - Different Identity64 → Stream comparison → Present/Absent (~10^-15 collision)
// Identity64 provides type documentation, Stream provides actual comparison
use crate::primitives::stream::{StreamEq, DefaultMaxDepth};
impl<QCap, StoredCap, Depth> EvalAt<Has<QCap>, Depth> for Leaf<StoredCap>
where
    QCap: Capability,
    StoredCap: Capability,
    QCap::Stream: StreamEq<StoredCap::Stream, DefaultMaxDepth>,
{
    type Out = <QCap::Stream as StreamEq<StoredCap::Stream, DefaultMaxDepth>>::Out;
}

#[macros::node16]
impl<QCap, Depth, _Slots_> EvalAt<Has<QCap>, Depth> for _Node16_
where
    QCap: Capability,
    Depth: Peano,
    QCap::Stream: GetTail<Depth>,
    Self: RouteQuery<QCap, Depth, QCap::At<Depth>>,
{
    type Out = <Self as RouteQuery<QCap, Depth, QCap::At<Depth>>>::Out;
}

// =============================================================================
// RouteQuery - 16-ary routing
// =============================================================================

use crate::primitives::nibble::{Nibble, X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF};

pub trait RouteQuery<Cap, Depth, Nib: Nibble> {
    type Out: Bool;
}

// 16-ary routing: each nibble Xi routes directly to slot Ni
// Generated 16 impls using #[node16(for_nibble)]
#[macros::node16(for_nibble)]
impl<Cap, Depth, _Slots_> RouteQuery<Cap, Depth, _Nibble_> for _Node16_
where
    Cap: Capability,
    Depth: Peano,
    _SlotN_: EvalAt<Has<Cap>, S<Depth>>,
{
    type Out = <_SlotN_ as EvalAt<Has<Cap>, S<Depth>>>::Out;
}

// =============================================================================
// Bucket Evaluation - Identity match + Stream fallback
// =============================================================================
//
// Strategy:
// 1. Check if QCap matches Head via Identity64 (perfect match)
// 2. If not, continue searching Tail
// 3. Stream routes, Identity verifies

use super::node::Bucket;

/// Bucket: Linear search with Identity+Stream hybrid matching
/// Checks head via Stream, returns Present if match OR continues to tail
impl<QCap, Head, Tail, Depth> EvalAt<Has<QCap>, Depth> for Bucket<Head, Tail>
where
    QCap: Capability,
    Head: Capability,
    Tail: EvalAt<Has<QCap>, Depth>,
    QCap::Stream: StreamEq<Head::Stream, DefaultMaxDepth>,
    <QCap::Stream as StreamEq<Head::Stream, DefaultMaxDepth>>::Out: BoolOr<<Tail as EvalAt<Has<QCap>, Depth>>::Out>,
{
    type Out = <<QCap::Stream as StreamEq<Head::Stream, DefaultMaxDepth>>::Out as BoolOr<<Tail as EvalAt<Has<QCap>, Depth>>::Out>>::Out;
    // Stream matches head OR tail contains QCap
}

/// Leaf: Exact Identity64 match (higher priority via &)
/// This impl is chosen when QCap == StoredCap (same Identity64 type).
impl<Cap, Depth> EvalAt<Has<Cap>, Depth> for &Leaf<Cap>
where
    Cap: Capability,
{
    type Out = Present;  // Same Identity64 type = exact match
}

// =============================================================================
// Evaluate implementations
// =============================================================================

/// Direct evaluation for any Capability type.
impl<Ctx, Cap> Evaluate<Cap> for Ctx
where
    Cap: Capability,
    Ctx: EvalAt<Has<Cap>, D0>,
{
    type Out = <Ctx as EvalAt<Has<Cap>, D0>>::Out;
}

// And<L, R>
impl<Ctx, L, R> Evaluate<And<L, R>> for Ctx
where
    Ctx: Evaluate<L> + Evaluate<R>,
    <Ctx as Evaluate<L>>::Out: BoolAnd<<Ctx as Evaluate<R>>::Out>,
{
    type Out = <<Ctx as Evaluate<L>>::Out as BoolAnd<<Ctx as Evaluate<R>>::Out>>::Out;
}

// Or<L, R>
impl<Ctx, L, R> Evaluate<Or<L, R>> for Ctx
where
    Ctx: Evaluate<L> + Evaluate<R>,
    <Ctx as Evaluate<L>>::Out: BoolOr<<Ctx as Evaluate<R>>::Out>,
{
    type Out = <<Ctx as Evaluate<L>>::Out as BoolOr<<Ctx as Evaluate<R>>::Out>>::Out;
}

// Not<Q>
impl<Ctx, Q> Evaluate<Not<Q>> for Ctx
where
    Ctx: Evaluate<Q>,
    <Ctx as Evaluate<Q>>::Out: BoolNot,
{
    type Out = <<Ctx as Evaluate<Q>>::Out as BoolNot>::Out;
}

// All<HNil>
impl<Ctx> Evaluate<All<HNil>> for Ctx {
    type Out = Present;
}

// All<HCons<H, T>>
impl<Ctx, H, T> Evaluate<All<HCons<H, T>>> for Ctx
where
    Ctx: Evaluate<H> + Evaluate<All<T>>,
    <Ctx as Evaluate<H>>::Out: BoolAnd<<Ctx as Evaluate<All<T>>>::Out>,
{
    type Out = <<Ctx as Evaluate<H>>::Out as BoolAnd<<Ctx as Evaluate<All<T>>>::Out>>::Out;
}

// Any<HNil>
impl<Ctx> Evaluate<Any<HNil>> for Ctx {
    type Out = Absent;
}

// Any<HCons<H, T>>
impl<Ctx, H, T> Evaluate<Any<HCons<H, T>>> for Ctx
where
    Ctx: Evaluate<H> + Evaluate<Any<T>>,
    <Ctx as Evaluate<H>>::Out: BoolOr<<Ctx as Evaluate<Any<T>>>::Out>,
{
    type Out = <<Ctx as Evaluate<H>>::Out as BoolOr<<Ctx as Evaluate<Any<T>>>::Out>>::Out;
}

// =============================================================================
// IsTrue / Require helpers
// =============================================================================

#[diagnostic::on_unimplemented(
    message = "Capability requirement failed: {Query}",
    label = "This capability set violates requirement '{Query}'",
    note = "Set: {Set}\nCheck if you are missing a required capability or possess a conflicting one."
)]
pub trait IsTrue<Set, Query: ?Sized> {}

impl<S, Q: ?Sized> IsTrue<S, Q> for Present {}

/// Wrapper trait to enforce a requirement.
/// Implemented only when `Evaluate<Q>::Out` is `Present`.
pub trait Require<Q> {}

impl<C, Q> Require<Q> for C
where
    C: Evaluate<Q>,
    <C as Evaluate<Q>>::Out: IsTrue<C, Q>,
{
}

// =============================================================================
// Macros
// =============================================================================

/// Build HList for All/Any queries
#[macro_export]
macro_rules! hlist {
    () => { $crate::trie::HNil };
    ($head:ty $(, $tail:ty)*) => {
        $crate::trie::HCons<$head, hlist![$($tail),*]>
    };
}

/// Macro for All query
#[macro_export]
macro_rules! all {
    ($($item:ty),* $(,)?) => {
        $crate::trie::All<hlist![$($item),*]>
    };
}

/// Macro for Any query
#[macro_export]
macro_rules! any {
    ($($item:ty),* $(,)?) => {
        $crate::trie::Any<hlist![$($item),*]>
    };
}

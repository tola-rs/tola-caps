//! # Capability System
//!
//! Zero-overhead compile-time capability checking with O(1) type-level lookups.
//!
//! ## Architecture
//!
//! This module uses a hash-routed 16-ary trie for capability management:
//! - **O(1) lookup**: Hash-based routing, not linear search
//! - **Infinite extensibility**: No central registry needed
//! - **Zero runtime overhead**: All checks at compile time
//!
//! ## How It Works
//!
//! ### Hash Stream
//! Each capability is assigned a unique path in a 16-ary trie (radix-16).
//! The path is derived from a hash of the type name, represented as an infinite
//! stream of nibbles (4-bit values, 0..15).
//!
//! ### Trie Structure
//! A capability set is stored as a trie where:
//! - `Empty` means no capability at this path
//! - `Leaf(C)` means capability C exists here
//! - `Node16` branches on the next nibble (16-way)
//!
//! Lookup walks the trie following the capability's hash stream.
//!
//! ### Macro-based Code Generation
//! We use recursive macros to generate all necessary trait implementations.
//! For example, generating impls for all distinct pairs (A, B) where A != B:
//!
//! 1. Base: empty list produces nothing
//! 2. Recurse: for list [head, ...tail], emit (head, t) and (t, head)
//!    for each t in tail, then recurse on tail
//!
//! This covers all 240 pairs (16 * 15) without repetition.
//!
//! ## Example
//!
//! ```ignore
//! use tola_vdom::capability::*;
//!
//! // Define capabilities with unique hash streams
//! struct CanRead;
//! impl_capability!(CanRead, ConstStream<X0>);
//!
//! struct CanWrite;
//! impl_capability!(CanWrite, ConstStream<X1>);
//!
//! // Build capability set
//! type MyCaps = capset![CanRead, CanWrite];
//!
//! // Query capabilities
//! fn process<C>(doc: Doc<Indexed, C>)
//! where
//!     C: Evaluate<Has<CanRead>, Out = Present>,
//! {
//!     // CanRead is checked at compile time
//! }
//!
//! // Boolean queries
//! fn complex_check<C>()
//! where
//!     C: Evaluate<And<Has<CanRead>, Not<Has<CanWrite>>>, Out = Present>,
//! { }
//! ```

use core::marker::PhantomData;

// =============================================================================
// Re-export proc-macros from macros crate
// =============================================================================

// (Proc-macros are re-exported from lib.rs)

// =============================================================================
// Boolean Logic
// =============================================================================

/// Type-level boolean trait
pub trait Bool: 'static {
    const VALUE: bool;
}

/// Type-level True
#[derive(Debug)]
pub struct Present;

/// Type-level False
#[derive(Debug)]
pub struct Absent;

impl Bool for Present {
    const VALUE: bool = true;
}

impl Bool for Absent {
    const VALUE: bool = false;
}

// =============================================================================
// Full Nibble System (X0-XF)
// =============================================================================

// Iteration Helper: Apply macro $M for each nibble X0..XF
macro_rules! for_each_nibble {
    ($mac:ident) => {
        $mac!(X0); $mac!(X1); $mac!(X2); $mac!(X3);
        $mac!(X4); $mac!(X5); $mac!(X6); $mac!(X7);
        $mac!(X8); $mac!(X9); $mac!(XA); $mac!(XB);
        $mac!(XC); $mac!(XD); $mac!(XE); $mac!(XF);
    };
}

// Iteration Helper: Apply macro $M to each (Nibble, SlotName) pair
macro_rules! for_each_nibble_pair {
    ($mac:ident) => {
        $mac!(X0, N0); $mac!(X1, N1); $mac!(X2, N2); $mac!(X3, N3);
        $mac!(X4, N4); $mac!(X5, N5); $mac!(X6, N6); $mac!(X7, N7);
        $mac!(X8, N8); $mac!(X9, N9); $mac!(XA, NA); $mac!(XB, NB);
        $mac!(XC, NC); $mac!(XD, ND); $mac!(XE, NE); $mac!(XF, NF);
    };
}

/// Type-level nibble (4-bit value, 0..15)
pub trait Nibble: 'static {}

// Define structs X0..XF and implement Nibble
macro_rules! define_nibble { ($n:ident) => { pub struct $n; impl Nibble for $n {} }; }
for_each_nibble!(define_nibble);

/// Type-level nibble equality
pub trait NibbleEq<Other: Nibble>: Nibble {
    type Out: Bool;
}

// Macro to generate NibbleEq implementations
// Macro to generate NibbleEq implementations
macro_rules! impl_nibble_eq_self {
    ($n:ident) => {
        impl NibbleEq<$n> for $n { type Out = Present; }
    };
}

macro_rules! impl_nibble_neq {
    ($a:ident, $b:ident) => {
        impl NibbleEq<$b> for $a { type Out = Absent; }
    };
}

// All equal cases (16 impls)
for_each_nibble!(impl_nibble_eq_self);

// Helper for symmetric pairs
macro_rules! impl_symmetric_pairs_all {
    ($mac:ident) => {
        impl_symmetric_pairs_all!(@recurse $mac, [X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF]);
    };
    (@recurse $mac:ident, [$head:ident, $($tail:ident),*]) => {
        $(
            $mac!($head, $tail);
            $mac!($tail, $head);
        )*
        impl_symmetric_pairs_all!(@recurse $mac, [$($tail),*]);
    };
    (@recurse $mac:ident, [$last:ident]) => {};
}

// All not-equal cases (240 impls)
impl_symmetric_pairs_all!(impl_nibble_neq);

// =============================================================================
// Recursive Hash Stream & GAT Accessor
// =============================================================================

/// Infinite stream of nibbles via recursive HList
pub trait HashStream: 'static {
    type Head: Nibble;
    type Tail: HashStream;
}

/// Helper to access stream at depth (Type Level Function)
pub trait GetTail<D> {
    type Out: HashStream;
}

impl<H: HashStream> GetTail<Z> for H {
    type Out = Self;
}

impl<H: HashStream, D> GetTail<S<D>> for H
where
    H::Tail: GetTail<D>,
{
    type Out = <H::Tail as GetTail<D>>::Out;
}

// --- Stream Implementations ---

/// Constant stream: N, N, N, N, ...
pub struct ConstStream<N>(PhantomData<N>);

impl<N: Nibble + 'static> HashStream for ConstStream<N> {
    type Head = N;
    type Tail = ConstStream<N>;
}

/// Alternating stream: A, B, A, B, ...
pub struct AltStream<A, B>(PhantomData<(A, B)>);

impl<A: Nibble + 'static, B: Nibble + 'static> HashStream for AltStream<A, B> {
    type Head = A;
    type Tail = AltStream<B, A>;
}

/// Cons cell for explicit streams
pub struct Cons<H, T>(PhantomData<(H, T)>);

impl<H: Nibble + 'static, T: HashStream> HashStream for Cons<H, T> {
    type Head = H;
    type Tail = T;
}

// =============================================================================
// Peano Numbers for Depth (D0-D64 for SHA-256)
// =============================================================================

/// Zero (base case)
pub struct Z;

/// Successor (`S<N>` = N + 1)
pub struct S<N>(PhantomData<N>);

pub type D0 = Z;
pub type D1 = S<D0>;
pub type D2 = S<D1>;
pub type D3 = S<D2>;
pub type D4 = S<D3>;
pub type D5 = S<D4>;
pub type D6 = S<D5>;
pub type D7 = S<D6>;
pub type D8 = S<D7>;
pub type D9 = S<D8>;
pub type D10 = S<D9>;
pub type D11 = S<D10>;
pub type D12 = S<D11>;
pub type D13 = S<D12>;
pub type D14 = S<D13>;
pub type D15 = S<D14>;
pub type D16 = S<D15>;
pub type D17 = S<D16>;
pub type D18 = S<D17>;
pub type D19 = S<D18>;
pub type D20 = S<D19>;
pub type D21 = S<D20>;
pub type D22 = S<D21>;
pub type D23 = S<D22>;
pub type D24 = S<D23>;
pub type D25 = S<D24>;
pub type D26 = S<D25>;
pub type D27 = S<D26>;
pub type D28 = S<D27>;
pub type D29 = S<D28>;
pub type D30 = S<D29>;
pub type D31 = S<D30>;
pub type D32 = S<D31>;
pub type D33 = S<D32>;
pub type D34 = S<D33>;
pub type D35 = S<D34>;
pub type D36 = S<D35>;
pub type D37 = S<D36>;
pub type D38 = S<D37>;
pub type D39 = S<D38>;
pub type D40 = S<D39>;
pub type D41 = S<D40>;
pub type D42 = S<D41>;
pub type D43 = S<D42>;
pub type D44 = S<D43>;
pub type D45 = S<D44>;
pub type D46 = S<D45>;
pub type D47 = S<D46>;
pub type D48 = S<D47>;
pub type D49 = S<D48>;
pub type D50 = S<D49>;
pub type D51 = S<D50>;
pub type D52 = S<D51>;
pub type D53 = S<D52>;
pub type D54 = S<D53>;
pub type D55 = S<D54>;
pub type D56 = S<D55>;
pub type D57 = S<D56>;
pub type D58 = S<D57>;
pub type D59 = S<D58>;
pub type D60 = S<D59>;
pub type D61 = S<D60>;
pub type D62 = S<D61>;
pub type D63 = S<D62>;
pub type D64 = S<D63>;

/// Default max depth for collision resolution (16 nibbles = 64 bits)
pub type DefaultMaxDepth = D16;

// =============================================================================
// Capability Trait (GAT Enhanced)
// =============================================================================

/// Core capability trait - defines a unique hash stream for each capability
///
/// # Example
///
/// ```ignore
/// struct MyCapability;
/// impl_capability!(MyCapability, ConstStream<X5>);
/// ```
pub trait Capability: 'static {
    /// The hash stream that uniquely identifies this capability
    type Stream: HashStream;

    /// GAT Accessor: Get nibble at depth D
    type At<D>: Nibble where Self::Stream: GetTail<D>;
}

/// Convenience macro to implement Capability with GAT
#[macro_export]
macro_rules! impl_capability {
    ($name:ident, $stream:ty) => {
        impl $crate::capability::Capability for $name {
            type Stream = $stream;
            type At<D> = <<Self::Stream as $crate::capability::GetTail<D>>::Out as $crate::capability::HashStream>::Head
            where Self::Stream: $crate::capability::GetTail<D>;
        }
    };
}

// =============================================================================
// Trie Structure (16-ary Trie)
// =============================================================================

/// Empty trie node (no capabilities)
pub struct Empty;

/// Leaf node containing a single capability
pub struct Leaf<Cap>(PhantomData<Cap>);

/// 16-ary internal node - one slot per nibble (X0..XF)
/// Each Ni corresponds to nibble Xi
#[allow(clippy::type_complexity)]
pub struct Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>(
    PhantomData<(N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF)>,
);

/// Type alias for an empty 16-ary node (all slots are Empty)
pub type EmptyNode16 = Node16<
    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
>;

// =============================================================================
// Stream Comparison
// =============================================================================

/// Compare two hash streams up to a depth limit
pub trait StreamEq<Other: HashStream, Limit> {
    type Out: Bool;
}

impl<A: HashStream, B: HashStream> StreamEq<B, Z> for A {
    type Out = Present;
}

impl<A, B, L> StreamEq<B, S<L>> for A
where
    A: HashStream,
    B: HashStream,
    A::Head: NibbleEq<B::Head>,
    <A::Head as NibbleEq<B::Head>>::Out: StreamEqDispatch<A::Tail, B::Tail, L>,
{
    type Out = <<A::Head as NibbleEq<B::Head>>::Out as StreamEqDispatch<A::Tail, B::Tail, L>>::Out;
}

pub trait StreamEqDispatch<TailA, TailB, Limit> {
    type Out: Bool;
}

impl<TailA, TailB, L> StreamEqDispatch<TailA, TailB, L> for Absent {
    type Out = Absent;
}

impl<TailA, TailB, L> StreamEqDispatch<TailA, TailB, L> for Present
where
    TailA: HashStream + StreamEq<TailB, L>,
    TailB: HashStream,
{
    type Out = <TailA as StreamEq<TailB, L>>::Out;
}

// =============================================================================
// Query Types
// =============================================================================

/// Query: Does the set contain capability Cap?
pub struct Has<Cap>(PhantomData<Cap>);

/// Internal trait for evaluating queries at a specific depth
pub trait EvalAt<Query, Depth> {
    type Out: Bool;
}

impl<Cap, Depth> EvalAt<Has<Cap>, Depth> for Empty {
    type Out = Absent;
}

impl<QCap, StoredCap, Depth> EvalAt<Has<QCap>, Depth> for Leaf<StoredCap>
where
    QCap: Capability,
    StoredCap: Capability,
    <QCap as Capability>::Stream: StreamEq<<StoredCap as Capability>::Stream, DefaultMaxDepth>,
{
    type Out = <<QCap as Capability>::Stream as StreamEq<<StoredCap as Capability>::Stream, DefaultMaxDepth>>::Out;
}

impl<QCap, Depth, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    EvalAt<Has<QCap>, Depth>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
where
    QCap: Capability,
    QCap::Stream: GetTail<Depth>,
    Self: RouteQuery<QCap, Depth, QCap::At<Depth>>,
{
    type Out = <Self as RouteQuery<QCap, Depth, QCap::At<Depth>>>::Out;
}

pub trait RouteQuery<Cap, Depth, Nib: Nibble> {
    type Out: Bool;
}

// 16-ary routing: each nibble Xi routes directly to slot Ni
macro_rules! impl_route_query_16 {
    ($nib:ident, $slot:ident) => {
        impl<Cap, Depth, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
            RouteQuery<Cap, Depth, $nib>
            for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
        where
            Cap: Capability,
            $slot: EvalAt<Has<Cap>, S<Depth>>,
        {
            type Out = <$slot as EvalAt<Has<Cap>, S<Depth>>>::Out;
        }
    };
}

// Generate all 16 routing implementations
for_each_nibble_pair!(impl_route_query_16);

// =============================================================================
// Boolean Logic Operators
// =============================================================================

/// Conjunction: L AND R
pub struct And<L, R>(PhantomData<(L, R)>);

/// Disjunction: L OR R
pub struct Or<L, R>(PhantomData<(L, R)>);

/// Negation: NOT Q
pub struct Not<Q>(PhantomData<Q>);

/// Type-level AND
pub trait BoolAnd<Other: Bool>: Bool {
    type Out: Bool;
}

impl<B: Bool> BoolAnd<B> for Absent {
    type Out = Absent;
}

impl BoolAnd<Absent> for Present {
    type Out = Absent;
}

impl BoolAnd<Present> for Present {
    type Out = Present;
}

/// Type-level OR
pub trait BoolOr<Other: Bool>: Bool {
    type Out: Bool;
}

impl<B: Bool> BoolOr<B> for Present {
    type Out = Present;
}

impl BoolOr<Present> for Absent {
    type Out = Present;
}

impl BoolOr<Absent> for Absent {
    type Out = Absent;
}

/// Type-level NOT
pub trait BoolNot: Bool {
    type Out: Bool;
}

impl BoolNot for Present {
    type Out = Absent;
}

impl BoolNot for Absent {
    type Out = Present;
}

// =============================================================================
// Evaluate (Main Entry Point)
// =============================================================================

/// Evaluate a boolean query on a capability set.
///
/// Returns `Present` (true) or `Absent` (false).
///
/// # Example
///
/// ```ignore
/// fn requires_read<C>()
/// where
///     C: Evaluate<Has<CanRead>, Out = Present>,
/// { }
/// ```
///
/// # Dependency Chain Validation
///
/// The type system automatically validates capability dependencies.
/// If `fn a()` requires A, and `fn b()` requires B but conflicts A,
/// calling `a()` then `b()` on the same capability set won't compile.
#[diagnostic::on_unimplemented(
    message = "Capability logic requirement evaluated to false or is invalid",
    label = "Logic '{Query}' is NOT satisfied by capability set '{Self}'",
    note = "Check if you are missing a required capability or possess a conflicting one."
)]
pub trait Evaluate<Query> {
    type Out: Bool;
}

// Cap (Direct implementation for any Capability)
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

// --- All / Any (variadic via HList) ---

/// Empty HList
pub struct HNil;

/// HList cons cell
pub struct HCons<H, T>(PhantomData<(H, T)>);

/// All queries must be true (conjunction)
pub struct All<List>(PhantomData<List>);

/// At least one query must be true (disjunction)
pub struct Any<List>(PhantomData<List>);

impl<Ctx> Evaluate<All<HNil>> for Ctx {
    type Out = Present;
}

impl<Ctx, H, T> Evaluate<All<HCons<H, T>>> for Ctx
where
    Ctx: Evaluate<H> + Evaluate<All<T>>,
    <Ctx as Evaluate<H>>::Out: BoolAnd<<Ctx as Evaluate<All<T>>>::Out>,
{
    type Out = <<Ctx as Evaluate<H>>::Out as BoolAnd<<Ctx as Evaluate<All<T>>>::Out>>::Out;
}

impl<Ctx> Evaluate<Any<HNil>> for Ctx {
    type Out = Absent;
}

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

// Helper trait for turning type equality into trait satisfaction.
// We include Set and Query to expose them in the diagnostic message.
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

/// Add a capability to a set.
#[diagnostic::on_unimplemented(
    message = "Cannot add capability {Cap} to set {Self}",
    label = "Failed to add {Cap} to {Self}",
    note = "Ensure {Self} is a valid capability set (Empty/Leaf/Node) and {Cap} is a Capability."
)]
pub trait With<Cap>: Sized {
    type Out;
}

impl<Ctx, Cap> With<Cap> for Ctx
where
    Cap: Capability,
    Ctx: InsertAt<Cap, D0>,
{
    type Out = <Ctx as InsertAt<Cap, D0>>::Out;
}

/// Remove a capability from a set
pub trait Without<Cap>: Sized {
    type Out;
}

impl<Ctx, Cap> Without<Cap> for Ctx
where
    Cap: Capability,
    Ctx: RemoveAt<Cap, D0>,
{
    type Out = <Ctx as RemoveAt<Cap, D0>>::Out;
}

pub trait RemoveAt<Cap, Depth> {
    type Out;
}

pub trait LeafRemove<IsMatch> {
    type Out;
}
pub trait InsertAt<Cap, Depth> {
    type Out;
}

impl<Cap, Depth> InsertAt<Cap, Depth> for Empty {
    type Out = Leaf<Cap>;
}

impl<Cap, Depth, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    InsertAt<Cap, Depth>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
where
    Cap: Capability,
    Cap::Stream: GetTail<Depth>,
    Self: NodeInsert<Cap, Depth, Cap::At<Depth>>,
{
    type Out = <Self as NodeInsert<Cap, Depth, Cap::At<Depth>>>::Out;
}

pub trait NodeInsert<Cap, Depth, Nib: Nibble> {
    type Out;
}

// 16-ary insertion: each nibble Xi inserts into slot Ni
macro_rules! impl_node_insert_16 {
    ($nib:ident, $slot:ident, [$($before:ident),*], [$($after:ident),*]) => {
        impl<Cap, Depth, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
            NodeInsert<Cap, Depth, $nib>
            for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
        where
            Cap: Capability,
            $slot: InsertAt<Cap, S<Depth>>,
        {
            type Out = Node16<$($before,)* <$slot as InsertAt<Cap, S<Depth>>>::Out, $($after),*>;
        }
    };
}

impl_node_insert_16!(X0, N0, [], [N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X1, N1, [N0], [N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X2, N2, [N0, N1], [N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X3, N3, [N0, N1, N2], [N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X4, N4, [N0, N1, N2, N3], [N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X5, N5, [N0, N1, N2, N3, N4], [N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X6, N6, [N0, N1, N2, N3, N4, N5], [N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X7, N7, [N0, N1, N2, N3, N4, N5, N6], [N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X8, N8, [N0, N1, N2, N3, N4, N5, N6, N7], [N9, NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(X9, N9, [N0, N1, N2, N3, N4, N5, N6, N7, N8], [NA, NB, NC, ND, NE, NF]);
impl_node_insert_16!(XA, NA, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9], [NB, NC, ND, NE, NF]);
impl_node_insert_16!(XB, NB, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA], [NC, ND, NE, NF]);
impl_node_insert_16!(XC, NC, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB], [ND, NE, NF]);
impl_node_insert_16!(XD, ND, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC], [NE, NF]);
impl_node_insert_16!(XE, NE, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND], [NF]);
impl_node_insert_16!(XF, NF, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE], []);

impl<NewCap, StoredCap, Depth> InsertAt<NewCap, Depth> for Leaf<StoredCap>
where
    NewCap: Capability,
    StoredCap: Capability,
    NewCap::Stream: GetTail<Depth>,
    StoredCap::Stream: GetTail<Depth>,
    Self: LeafInsert<NewCap, StoredCap, Depth, NewCap::At<Depth>, StoredCap::At<Depth>>,
{
    type Out = <Self as LeafInsert<NewCap, StoredCap, Depth, NewCap::At<Depth>, StoredCap::At<Depth>>>::Out;
}

pub trait LeafInsert<NewCap, StoredCap, Depth, NewNib: Nibble, StoredNib: Nibble> {
    type Out;
}

// =============================================================================
// LeafInsert for 16-ary Trie
// =============================================================================
//
// When inserting into a Leaf, there are two cases:
// 1. Same nibble (collision): Create a Node16 with both caps recursed into slot Xi
// 2. Different nibbles (diverge): Create a Node16 with NewCap in slot Xn, StoredCap in slot Xs

/// Helper trait to create a Node16 with a single leaf at position Xi
pub trait MakeNode16WithLeaf<Cap, Nib: Nibble> {
    type Out;
}

/// Helper trait to create a Node16 with two leaves at positions
pub trait MakeNode16WithTwoLeaves<NewCap, StoredCap, NewNib: Nibble, StoredNib: Nibble> {
    type Out;
}

// Macro to implement MakeNode16WithLeaf for each nibble position
macro_rules! impl_make_node16_with_leaf {
    ($nib:ident, [$($before:ident),*], [$($after:ident),*]) => {
        impl<Cap> MakeNode16WithLeaf<Cap, $nib> for () {
            type Out = Node16<$($before,)* Leaf<Cap>, $($after),*>;
        }
    };
}

impl_make_node16_with_leaf!(X0, [], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X1, [Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X2, [Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X3, [Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X4, [Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X5, [Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X6, [Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X7, [Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X8, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(X9, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(XA, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(XB, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty]);
impl_make_node16_with_leaf!(XC, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty]);
impl_make_node16_with_leaf!(XD, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty]);
impl_make_node16_with_leaf!(XE, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty]);
impl_make_node16_with_leaf!(XF, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], []);

// Same nibble collision: recurse BOTH caps into the same slot at NEXT depth
// The key insight: when two caps have the same nibble at current depth,
// we need to create a Node16 where the corresponding slot contains
// a subtree with both caps inserted at the next depth level.
macro_rules! impl_leaf_insert_same_16 {
    ($nib:ident, [$($before:ident),*], [$($after:ident),*]) => {
        impl<NewCap, StoredCap, Depth> LeafInsert<NewCap, StoredCap, Depth, $nib, $nib> for Leaf<StoredCap>
        where
            NewCap: Capability,
            StoredCap: Capability,
            // Recursively insert both caps into a subtree at next depth
            // First insert StoredCap into Empty at S<Depth>
            Empty: InsertAt<StoredCap, S<Depth>>,
            // Then insert NewCap into that result at S<Depth>
            <Empty as InsertAt<StoredCap, S<Depth>>>::Out: InsertAt<NewCap, S<Depth>>,
        {
            // Create Node16 with the recursive subtree in the correct slot
            type Out = Node16<
                $($before,)*
                <<Empty as InsertAt<StoredCap, S<Depth>>>::Out as InsertAt<NewCap, S<Depth>>>::Out,
                $($after),*
            >;
        }
    };
}

impl_leaf_insert_same_16!(X0, [], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X1, [Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X2, [Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X3, [Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X4, [Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X5, [Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X6, [Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X7, [Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X8, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(X9, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(XA, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(XB, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty, Empty]);
impl_leaf_insert_same_16!(XC, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty, Empty]);
impl_leaf_insert_same_16!(XD, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty, Empty]);
impl_leaf_insert_same_16!(XE, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], [Empty]);
impl_leaf_insert_same_16!(XF, [Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty], []);

// Different nibbles: create Node16 with both leaves in their respective slots
// This requires generating 240 impls (16*15)
macro_rules! impl_leaf_insert_diff_16 {
    // Generate impl for (new_nib, stored_nib) pair
    // The output is a Node16 with Leaf<NewCap> at new_nib slot and Leaf<StoredCap> at stored_nib slot
    ($new_nib:ident @ $new_idx:tt, $stored_nib:ident @ $stored_idx:tt) => {
        impl<NewCap, StoredCap, Depth> LeafInsert<NewCap, StoredCap, Depth, $new_nib, $stored_nib> for Leaf<StoredCap> {
            type Out = impl_leaf_insert_diff_16!(@make_node16 NewCap, StoredCap, $new_idx, $stored_idx);
        }
    };

    // Helper to generate Node16 type with two leaves
    (@make_node16 $new:ident, $stored:ident, $ni:tt, $si:tt) => {
        Node16<
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 0),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 1),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 2),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 3),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 4),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 5),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 6),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 7),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 8),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 9),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 10),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 11),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 12),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 13),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 14),
            impl_leaf_insert_diff_16!(@slot $new, $stored, $ni, $si, 15)
        >
    };

    // Slot selection: if idx == ni, Leaf<NewCap>; if idx == si, Leaf<StoredCap>; else Empty
    (@slot $new:ident, $stored:ident, $ni:tt, $si:tt, $idx:tt) => {
        impl_leaf_insert_diff_16!(@select $new, $stored, $ni, $si, $idx)
    };

    // Selection logic using tt comparison
    (@select $new:ident, $stored:ident, 0, 0, $idx:tt) => { compile_error!("same nibble") };
    (@select $new:ident, $stored:ident, $ni:tt, $si:tt, $idx:tt) => {
        impl_leaf_insert_diff_16!(@check_new $new, $stored, $ni, $si, $idx)
    };

    (@check_new $new:ident, $stored:ident, 0, $si:tt, 0) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 1, $si:tt, 1) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 2, $si:tt, 2) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 3, $si:tt, 3) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 4, $si:tt, 4) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 5, $si:tt, 5) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 6, $si:tt, 6) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 7, $si:tt, 7) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 8, $si:tt, 8) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 9, $si:tt, 9) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 10, $si:tt, 10) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 11, $si:tt, 11) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 12, $si:tt, 12) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 13, $si:tt, 13) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 14, $si:tt, 14) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, 15, $si:tt, 15) => { Leaf<$new> };
    (@check_new $new:ident, $stored:ident, $ni:tt, $si:tt, $idx:tt) => {
        impl_leaf_insert_diff_16!(@check_stored $new, $stored, $ni, $si, $idx)
    };

    (@check_stored $new:ident, $stored:ident, $ni:tt, 0, 0) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 1, 1) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 2, 2) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 3, 3) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 4, 4) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 5, 5) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 6, 6) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 7, 7) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 8, 8) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 9, 9) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 10, 10) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 11, 11) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 12, 12) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 13, 13) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 14, 14) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, 15, 15) => { Leaf<$stored> };
    (@check_stored $new:ident, $stored:ident, $ni:tt, $si:tt, $idx:tt) => { Empty };
}

// Helper macro to generate symmetric diff impls for a list of (Nibble, Index)
// Generates pairs (x, y) and (y, x) for all distinct x, y.
macro_rules! impl_leaf_insert_all_diff {
    ( $($nib:ident @ $idx:tt),* ) => {
        impl_leaf_insert_all_diff!(@recurse [$(($nib @ $idx))*]);
    };

    // Recursion: Head vs Tail expansion
    (@recurse [ $head:tt $($tail:tt)* ]) => {
        // Expand Head vs each element in Tail
        $(
            impl_leaf_insert_all_diff!(@emit_pair $head, $tail);
        )*
        // Recurse on Tail
        impl_leaf_insert_all_diff!(@recurse [ $($tail)* ]);
    };
    (@recurse []) => {};

    // Emit impls for (N1, N2) and (N2, N1) (Commutative cover)
    (@emit_pair ($n1:ident @ $i1:tt), ($n2:ident @ $i2:tt)) => {
         impl_leaf_insert_diff_16!($n1 @ $i1, $n2 @ $i2);
         impl_leaf_insert_diff_16!($n2 @ $i2, $n1 @ $i1);
    };
}

// Generate all 240 different-nibble impls (16*15)
impl_leaf_insert_all_diff!(
    X0 @ 0, X1 @ 1, X2 @ 2, X3 @ 3,
    X4 @ 4, X5 @ 5, X6 @ 6, X7 @ 7,
    X8 @ 8, X9 @ 9, XA @ 10, XB @ 11,
    XC @ 12, XD @ 13, XE @ 14, XF @ 15
);

// =============================================================================
// Convenience Type Aliases
// =============================================================================

/// Empty capability set
pub type CapSet0 = Empty;

/// Capability set with 1 capability
pub type CapSet1<A> = <Empty as With<A>>::Out;

/// Capability set with 2 capabilities
pub type CapSet2<A, B> = <<Empty as With<A>>::Out as With<B>>::Out;

/// Capability set with 3 capabilities
pub type CapSet3<A, B, C> = <<<Empty as With<A>>::Out as With<B>>::Out as With<C>>::Out;

/// Capability set with 4 capabilities
pub type CapSet4<A, B, C, D> = <<<<Empty as With<A>>::Out as With<B>>::Out as With<C>>::Out as With<D>>::Out;

// =============================================================================
// Convenience Macros
// =============================================================================

/// Build HList for All/Any queries
/// Usage: `hlist![Has<A>, Has<B>, Has<C>]`
#[macro_export]
macro_rules! hlist {
    () => { $crate::capability::HNil };
    ($head:ty $(, $tail:ty)*) => {
        $crate::capability::HCons<$head, hlist![$($tail),*]>
    };
}

/// Macro for All query
/// Usage: `all![Has<A>, Has<B>]`
#[macro_export]
macro_rules! all {
    ($($item:ty),* $(,)?) => {
        $crate::capability::All<hlist![$($item),*]>
    };
}

/// Macro for Any query
/// Usage: `any![Has<A>, Has<B>]`
#[macro_export]
macro_rules! any {
    ($($item:ty),* $(,)?) => {
        $crate::capability::Any<hlist![$($item),*]>
    };
}

/// Macro to build a CapSet from multiple capabilities
/// Usage: `capset![CapA, CapB, CapC]`
#[macro_export]
macro_rules! capset {
    () => { $crate::capability::Empty };
    ($cap:ty) => {
        <$crate::capability::Empty as $crate::capability::With<$cap>>::Out
    };
    ($cap:ty, $($rest:ty),+ $(,)?) => {
        <capset![$($rest),+] as $crate::capability::With<$cap>>::Out
    };
}

// =============================================================================
// Set Operations (Union, Intersect, SupersetOf)
// =============================================================================

/// Set Union: Merge two capability sets
/// Returns a set containing all capabilities from both A and B.
pub trait SetUnion<Other> {
    type Out;
}

/// Set Intersection: Common capabilities between two sets
/// Returns a set containing only capabilities present in both A and B.
pub trait SetIntersect<Other> {
    type Out;
}

/// SupersetOf: Check if Self contains all capabilities in Other
/// Used for downcasting / forgetting extra capabilities.
pub trait SupersetOf<Other>: Sized {}

// -----------------------------------------------------------------------------
// SetUnion Implementations
// -----------------------------------------------------------------------------

// Empty U X = X
impl<T> SetUnion<T> for Empty {
    type Out = T;
}

// Leaf<A> U Empty = Leaf<A>
impl<A> SetUnion<Empty> for Leaf<A> {
    type Out = Leaf<A>;
}

// Node16 U Empty = Node16
impl<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    SetUnion<Empty>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
{
    type Out = Self;
}

// Leaf<A> U Leaf<B> = Insert B into Leaf<A>
// (Uses existing Add semantics which handles identity/collision)
impl<A, B> SetUnion<Leaf<B>> for Leaf<A>
where
    B: Capability,
    Leaf<A>: With<B>,
{
    type Out = <Leaf<A> as With<B>>::Out;
}

// Leaf<A> U Node16<...> = Insert A into Node16
impl<A, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    SetUnion<Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>>
    for Leaf<A>
where
    A: Capability,
    Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>: With<A>,
{
    type Out = <Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF> as With<A>>::Out;
}

// Node16 U Leaf<A> = Insert A into Node16
impl<A, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    SetUnion<Leaf<A>>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
where
    A: Capability,
    Self: With<A>,
{
    type Out = <Self as With<A>>::Out;
}

// Node16 U Node16: not implemented (would need recursive slot-by-slot union)

// -----------------------------------------------------------------------------
// SetIntersect Implementations
// -----------------------------------------------------------------------------

// Empty n X = Empty
impl<T> SetIntersect<T> for Empty {
    type Out = Empty;
}

// X n Empty = Empty
impl<A> SetIntersect<Empty> for Leaf<A> {
    type Out = Empty;
}

impl<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    SetIntersect<Empty>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
{
    type Out = Empty;
}

// Leaf<A> n Leaf<A> = Leaf<A> (idempotent)
// Leaf<A> n Leaf<B> = Empty (when A != B)
// This requires stream comparison; simplified version here:
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

// -----------------------------------------------------------------------------
// SupersetOf Implementations
// -----------------------------------------------------------------------------

// Everything is a superset of Empty
impl<T> SupersetOf<Empty> for T {}

// Leaf<A> is superset of Leaf<A> (identity)
impl<A> SupersetOf<Leaf<A>> for Leaf<A> {}

// Node16 is superset of Leaf<A> if it contains A
impl<A, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    SupersetOf<Leaf<A>>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
where
    A: Capability,
    Self: Evaluate<Has<A>, Out = Present>,
{}

// =============================================================================
// Set Operation Macros
// =============================================================================

/// Macro to compute union of two capability sets
/// Usage: `union![SetA, SetB]`
#[macro_export]
macro_rules! union {
    ($a:ty, $b:ty) => {
        <$a as $crate::capability::SetUnion<$b>>::Out
    };
}

// =============================================================================
// RemoveAt Implementation
// =============================================================================

impl<Cap, Depth> RemoveAt<Cap, Depth> for Empty {
    type Out = Empty;
}

impl<S> LeafRemove<Present> for Leaf<S> {
    type Out = Empty;
}

impl<S> LeafRemove<Absent> for Leaf<S> {
    type Out = Leaf<S>;
}

impl<StoredCap, QCap, Depth> RemoveAt<QCap, Depth> for Leaf<StoredCap>
where
    QCap: Capability,
    StoredCap: Capability,
    Leaf<StoredCap>: EvalAt<Has<QCap>, Depth>,
    Leaf<StoredCap>: LeafRemove<<Leaf<StoredCap> as EvalAt<Has<QCap>, Depth>>::Out>,
{
    type Out = <Leaf<StoredCap> as LeafRemove<<Leaf<StoredCap> as EvalAt<Has<QCap>, Depth>>::Out>>::Out;
}

pub trait NodeRemove<Cap, Depth, Nib: Nibble> {
    type Out;
}

macro_rules! impl_node_remove_16 {
    ($nib:ident, $slot:ident, [$($before:ident),*], [$($after:ident),*]) => {
        impl<Cap, Depth, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
            NodeRemove<Cap, Depth, $nib>
            for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
        where
            Cap: Capability,
            $slot: RemoveAt<Cap, S<Depth>>,
        {
            type Out = Node16<$($before,)* <$slot as RemoveAt<Cap, S<Depth>>>::Out, $($after),*>;
        }
    };
}

impl_node_remove_16!(X0, N0, [], [N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X1, N1, [N0], [N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X2, N2, [N0, N1], [N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X3, N3, [N0, N1, N2], [N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X4, N4, [N0, N1, N2, N3], [N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X5, N5, [N0, N1, N2, N3, N4], [N6, N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X6, N6, [N0, N1, N2, N3, N4, N5], [N7, N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X7, N7, [N0, N1, N2, N3, N4, N5, N6], [N8, N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X8, N8, [N0, N1, N2, N3, N4, N5, N6, N7], [N9, NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(X9, N9, [N0, N1, N2, N3, N4, N5, N6, N7, N8], [NA, NB, NC, ND, NE, NF]);
impl_node_remove_16!(XA, NA, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9], [NB, NC, ND, NE, NF]);
impl_node_remove_16!(XB, NB, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA], [NC, ND, NE, NF]);
impl_node_remove_16!(XC, NC, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB], [ND, NE, NF]);
impl_node_remove_16!(XD, ND, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC], [NE, NF]);
impl_node_remove_16!(XE, NE, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND], [NF]);
impl_node_remove_16!(XF, NF, [N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE], []);

impl<Cap, Depth, N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
    RemoveAt<Cap, Depth>
    for Node16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF>
where
    Cap: Capability,
    Cap::Stream: GetTail<Depth>,
    Self: NodeRemove<Cap, Depth, Cap::At<Depth>>,
{
    type Out = <Self as NodeRemove<Cap, Depth, Cap::At<Depth>>>::Out;
}

/// Macro to compute intersection of two capability sets
/// Usage: `intersect![SetA, SetB]`
#[macro_export]
macro_rules! intersect {
    ($a:ty, $b:ty) => {
        <$a as $crate::capability::SetIntersect<$b>>::Out
    };
}

/// Macro to add a capability to a set (shorthand for `<Set as With<Cap>>::Out`)
/// Usage:
/// - `with![Set, Cap]` -> Single add
/// - `with![Set, A, B]` -> Chain add (A then B)
/// - `with![Cap]` -> Implicit `__C`
#[macro_export]
macro_rules! with {
    ($cap:ty) => {
        <__C as $crate::capability::With<$cap>>::Out
    };
    ($set:ty, $cap:ty) => {
        <$set as $crate::capability::With<$cap>>::Out
    };
    ($set:ty, $cap:ty, $($rest:ty),+) => {
        $crate::with![ $crate::with![$set, $cap], $($rest),+ ]
    };
}

/// Macro to remove a capability from a set
/// Usage:
/// - `without![Set, Cap]` -> Remove Cap from Set
/// - `without![Set, A, B]` -> Remove A then B
/// - `without![Cap]` -> Remove Cap from implicit `__C`
#[macro_export]
macro_rules! without {
    ($cap:ty) => {
        <__C as $crate::capability::Without<$cap>>::Out
    };
    ($set:ty, $cap:ty) => {
        <$set as $crate::capability::Without<$cap>>::Out
    };
    ($set:ty, $cap:ty, $($rest:ty),+) => {
        $crate::without![ $crate::without![$set, $cap], $($rest),+ ]
    };
}


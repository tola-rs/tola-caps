//! Hash stream types for capability routing.
//!
//! Provides infinite nibble streams and Peano numbers for depth tracking.

use core::marker::PhantomData;
use super::nibble::Nibble;

// =============================================================================
// Hash Stream trait
// =============================================================================

/// Infinite stream of nibbles via recursive type
pub trait HashStream: 'static {
    type Head: Nibble;
    type Tail: HashStream;
}

/// Helper to access stream at depth D
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

// =============================================================================
// Stream implementations
// =============================================================================

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
// Peano Numbers
// =============================================================================

/// Peano number trait
pub trait Peano {}

/// Zero (base case)
pub struct Z;
impl Peano for Z {}

/// Successor (S<N> = N + 1)
pub struct S<N>(PhantomData<N>);
impl<N: Peano> Peano for S<N> {}

// Generate D0..D64 using proc-macro
macros::peano!(64);

/// Default max depth for collision resolution (16 nibbles = 64 bits)
pub type DefaultMaxDepth = D16;

// =============================================================================
// Stream comparison
// =============================================================================

use super::bool::{Bool, Present, Absent};

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
    A::Head: super::nibble::NibbleEq<B::Head>,
    <A::Head as super::nibble::NibbleEq<B::Head>>::Out: StreamEqDispatch<A::Tail, B::Tail, L>,
{
    type Out = <<A::Head as super::nibble::NibbleEq<B::Head>>::Out as StreamEqDispatch<A::Tail, B::Tail, L>>::Out;
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
// Stream Intersection (Bitwise AND) - DISABLED: causes infinite compilation
// =============================================================================

// NOTE: StreamAnd is disabled because it has no depth limit and will cause
// infinite compilation when used with cyclic streams like HashStream16.
// If you need this functionality, add a depth limit parameter.

/*
/// Compute the bitwise AND of two capability streams.
pub trait StreamAnd<Other: HashStream>: HashStream {
    type Out: HashStream;
}

impl<A, B> StreamAnd<B> for A
where
    A: HashStream,
    B: HashStream,
    A::Head: super::nibble::HexAnd<B::Head>,
    A::Tail: StreamAnd<B::Tail>,
{
    type Out = Cons<
        <A::Head as super::nibble::HexAnd<B::Head>>::Out,
        <A::Tail as StreamAnd<B::Tail>>::Out
    >;
}
*/

// =============================================================================
// Const-to-Stream conversion (Stable Rust approach)
// =============================================================================

use super::nibble::{X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF};

/// Trait to select nibble type from const value
pub trait SelectNibble<const N: u8> {
    type Out: Nibble;
}

macro_rules! impl_select_nibble {
    ($($val:literal => $nib:ident),* $(,)?) => {
        $(
            impl SelectNibble<$val> for () {
                type Out = $nib;
            }
        )*
    };
}

impl_select_nibble!(
    0 => X0, 1 => X1, 2 => X2, 3 => X3,
    4 => X4, 5 => X5, 6 => X6, 7 => X7,
    8 => X8, 9 => X9, 10 => XA, 11 => XB,
    12 => XC, 13 => XD, 14 => XE, 15 => XF,
);

/// Build a hash stream from 16 const nibble values (for 64-bit hash)
/// Usage: HashStream16<{n0}, {n1}, ..., {n15}>
pub struct HashStream16<
    const N0: u8, const N1: u8, const N2: u8, const N3: u8,
    const N4: u8, const N5: u8, const N6: u8, const N7: u8,
    const N8: u8, const N9: u8, const N10: u8, const N11: u8,
    const N12: u8, const N13: u8, const N14: u8, const N15: u8,
>(PhantomData<()>);

impl<
    const N0: u8, const N1: u8, const N2: u8, const N3: u8,
    const N4: u8, const N5: u8, const N6: u8, const N7: u8,
    const N8: u8, const N9: u8, const N10: u8, const N11: u8,
    const N12: u8, const N13: u8, const N14: u8, const N15: u8,
> HashStream for HashStream16<N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, N10, N11, N12, N13, N14, N15>
where
    (): SelectNibble<N0> + SelectNibble<N1> + SelectNibble<N2> + SelectNibble<N3>
      + SelectNibble<N4> + SelectNibble<N5> + SelectNibble<N6> + SelectNibble<N7>
      + SelectNibble<N8> + SelectNibble<N9> + SelectNibble<N10> + SelectNibble<N11>
      + SelectNibble<N12> + SelectNibble<N13> + SelectNibble<N14> + SelectNibble<N15>,
{
    type Head = <() as SelectNibble<N0>>::Out;
    type Tail = HashStream16<N1, N2, N3, N4, N5, N6, N7, N8, N9, N10, N11, N12, N13, N14, N15, N0>;
}

// =============================================================================
// ByteStream128 - Disabled (requires nightly features for proper streaming)
// =============================================================================
//
// ByteStream128 with 64 nibble params would provide zero-collision identity,
// but implementing proper Tail rotation requires either:
// 1. generic_const_exprs (nightly)
// 2. Specialization (nightly)
// 3. Generating 64 wrapper types with complex dependency chains
//
// For now, we use HashStream16 (64-bit hash) which has negligible collision
// probability in practice, and any collision results in safe compilation failure.

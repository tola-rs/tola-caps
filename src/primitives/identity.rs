//! Identity primitives for Type Tuple system.

use core::marker::PhantomData;
use crate::primitives::nibble::{Nibble, NibbleEq};
use crate::primitives::stream::{StreamEq, DefaultMaxDepth};
use crate::primitives::stream::HashStream;
use crate::primitives::{Bool, Present, Absent};

/// A type-level character decomposed into nibbles for Stable comparison.
/// Uses 6 nibbles (24 bits) to support full Unicode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Char<N0, N1, N2, N3, N4, N5>(PhantomData<(N0, N1, N2, N3, N4, N5)>);

/// Compact type-level byte using only 2 nibbles (8 bits).
/// Used for ASCII path encoding - 3x more compact than Char.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Byte<N0, N1>(PhantomData<(N0, N1)>);

/// Unique Marker carrying a hash stream (legacy, being replaced by Finger Tree).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Marker<S>(PhantomData<S>);

/// Trait to check type-level equality of Identities (Tuples).
pub trait IdentityEq<Other: ?Sized> {
    type Out: Bool;
}

// =============================================================================
// Primitive Implementations
// =============================================================================

// Char comparison
impl<A0, A1, A2, A3, A4, A5, B0, B1, B2, B3, B4, B5> IdentityEq<Char<B0, B1, B2, B3, B4, B5>> for Char<A0, A1, A2, A3, A4, A5>
where
    A0: Nibble + NibbleEq<B0>, A1: Nibble + NibbleEq<B1>, A2: Nibble + NibbleEq<B2>,
    A3: Nibble + NibbleEq<B3>, A4: Nibble + NibbleEq<B4>, A5: Nibble + NibbleEq<B5>,
    B0: Nibble, B1: Nibble, B2: Nibble, B3: Nibble, B4: Nibble, B5: Nibble,
    <A0 as NibbleEq<B0>>::Out: crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>,
    <<A0 as NibbleEq<B0>>::Out as crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out: crate::primitives::BoolAnd<<A2 as NibbleEq<B2>>::Out>,
    <<<A0 as NibbleEq<B0>>::Out as crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out as crate::primitives::BoolAnd<<A2 as NibbleEq<B2>>::Out>>::Out: crate::primitives::BoolAnd<<A3 as NibbleEq<B3>>::Out>,
    <<<<A0 as NibbleEq<B0>>::Out as crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out as crate::primitives::BoolAnd<<A2 as NibbleEq<B2>>::Out>>::Out as crate::primitives::BoolAnd<<A3 as NibbleEq<B3>>::Out>>::Out: crate::primitives::BoolAnd<<A4 as NibbleEq<B4>>::Out>,
    <<<<<A0 as NibbleEq<B0>>::Out as crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out as crate::primitives::BoolAnd<<A2 as NibbleEq<B2>>::Out>>::Out as crate::primitives::bool::BoolAnd<<A3 as NibbleEq<B3>>::Out>>::Out as crate::primitives::bool::BoolAnd<<A4 as NibbleEq<B4>>::Out>>::Out: crate::primitives::bool::BoolAnd<<A5 as NibbleEq<B5>>::Out>,
{
    type Out = <<<<<
        <A0 as NibbleEq<B0>>::Out
        as crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out
        as crate::primitives::BoolAnd<<A2 as NibbleEq<B2>>::Out>>::Out
        as crate::primitives::BoolAnd<<A3 as NibbleEq<B3>>::Out>>::Out
        as crate::primitives::BoolAnd<<A4 as NibbleEq<B4>>::Out>>::Out
        as crate::primitives::BoolAnd<<A5 as NibbleEq<B5>>::Out>>::Out;
}


// Byte comparison - simple 2-nibble equality
impl<A0, A1, B0, B1> IdentityEq<Byte<B0, B1>> for Byte<A0, A1>
where
    A0: Nibble + NibbleEq<B0>,
    A1: Nibble + NibbleEq<B1>,
    B0: Nibble, B1: Nibble,
    <A0 as NibbleEq<B0>>::Out: crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>,
{
    type Out = <<A0 as NibbleEq<B0>>::Out as crate::primitives::BoolAnd<<A1 as NibbleEq<B1>>::Out>>::Out;
}

// Marker comparison via hash stream equality (legacy)
impl<S1: HashStream, S2: HashStream> IdentityEq<Marker<S2>> for Marker<S1>
where
    S1: StreamEq<S2, DefaultMaxDepth>,
{
    type Out = <S1 as StreamEq<S2, DefaultMaxDepth>>::Out;
}
// Unit comparison
impl IdentityEq<()> for () {
    type Out = Present;
}

// =============================================================================
// Tuple Implementations (Unrolled)
// =============================================================================

// 1 Element
impl<T0, U0> IdentityEq<(U0,)> for (T0,)
where T0: IdentityEq<U0> {
    type Out = <T0 as IdentityEq<U0>>::Out;
}

// Implement IdentityEq for Tuples of different arity (Always Absent)
macro_rules! impl_mismatch {
    (
        [$($T:ident),+], // Lhs params
        [$($U:ident),+]  // Rhs params
    ) => {
        impl< $($T),+, $($U),+ > IdentityEq< ($($U,)+) > for ($($T,)+) {
            type Out = Absent;
        }
    };
}

// We rely on the iterative macro I wrote in step 3033/3041 but adapted.
macro_rules! impl_tuple_ids {
    ($T0:ident $U0:ident, $T1:ident $U1:ident) => {
        impl<$T0, $T1, $U0, $U1> IdentityEq<($U0, $U1)> for ($T0, $T1)
        where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
        {
            type Out = <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out;
        }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident) => {
         impl<$T0, $T1, $T2, $U0, $U1, $U2> IdentityEq<($U0, $U1, $U2)> for ($T0, $T1, $T2)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
         {
             type Out = <<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out;
         }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident) => {
         impl<$T0, $T1, $T2, $T3, $U0, $U1, $U2, $U3> IdentityEq<($U0, $U1, $U2, $U3)> for ($T0, $T1, $T2, $T3)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
         {
             type Out = <<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out;
         }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $U0, $U1, $U2, $U3, $U4> IdentityEq<($U0, $U1, $U2, $U3, $U4)> for ($T0, $T1, $T2, $T3, $T4)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
         {
             type Out = <<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out;
         }
    };



    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $U0, $U1, $U2, $U3, $U4, $U5> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5)> for ($T0, $T1, $T2, $T3, $T4, $T5)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
         {
             type Out = <<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out;
         }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $U0, $U1, $U2, $U3, $U4, $U5, $U6> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
         {
             type Out = <<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out;
         }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
         {
             type Out = <<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out;
         }
    };



    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident, $T8:ident $U8:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>, $T8: IdentityEq<$U8>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
            <<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out: crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>,
         {
             type Out = <<<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out;
         }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident, $T8:ident $U8:ident, $T9:ident $U9:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>, $T8: IdentityEq<$U8>, $T9: IdentityEq<$U9>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
            <<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out: crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>,
            <<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out: crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>,
         {
             type Out = <<<<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out;
         }
    };



    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident, $T8:ident $U8:ident, $T9:ident $U9:ident, $TA:ident $UA:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>, $T8: IdentityEq<$U8>, $T9: IdentityEq<$U9>, $TA: IdentityEq<$UA>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
            <<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out: crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>,
            <<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out: crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>,
            <<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out: crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>,
         {
             type Out = <<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out;
         }
    };



    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident, $T8:ident $U8:ident, $T9:ident $U9:ident, $TA:ident $UA:ident, $TB:ident $UB:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $TB, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA, $UB> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA, $UB)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $TB)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>, $T8: IdentityEq<$U8>, $T9: IdentityEq<$U9>, $TA: IdentityEq<$UA>, $TB: IdentityEq<$UB>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
            <<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out: crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>,
            <<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out: crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>,
            <<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out: crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>,
            <<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out: crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>,
         {
             type Out = <<<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>>::Out;
         }
    };


    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident, $T8:ident $U8:ident, $T9:ident $U9:ident, $TA:ident $UA:ident, $TB:ident $UB:ident, $TC:ident $UC:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $TB, $TC, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA, $UB, $UC> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA, $UB, $UC)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $TB, $TC)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>, $T8: IdentityEq<$U8>, $T9: IdentityEq<$U9>, $TA: IdentityEq<$UA>, $TB: IdentityEq<$UB>, $TC: IdentityEq<$UC>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
            <<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out: crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>,
            <<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out: crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>,
            <<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out: crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>,
            <<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out: crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>,
            <<<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out as crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>>::Out: crate::primitives::BoolAnd<<$TC as IdentityEq<$UC>>::Out>,
         {
             type Out = <<<<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TC as IdentityEq<$UC>>::Out>>::Out;
         }
    };



    ($T0:ident $U0:ident, $T1:ident $U1:ident, $T2:ident $U2:ident, $T3:ident $U3:ident, $T4:ident $U4:ident, $T5:ident $U5:ident, $T6:ident $U6:ident, $T7:ident $U7:ident, $T8:ident $U8:ident, $T9:ident $U9:ident, $TA:ident $UA:ident, $TB:ident $UB:ident, $TC:ident $UC:ident, $TD:ident $UD:ident) => {
         impl<$T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $TB, $TC, $TD, $U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA, $UB, $UC, $UD> IdentityEq<($U0, $U1, $U2, $U3, $U4, $U5, $U6, $U7, $U8, $U9, $UA, $UB, $UC, $UD)> for ($T0, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $TA, $TB, $TC, $TD)
         where
            $T0: IdentityEq<$U0>, $T1: IdentityEq<$U1>, $T2: IdentityEq<$U2>, $T3: IdentityEq<$U3>, $T4: IdentityEq<$U4>, $T5: IdentityEq<$U5>, $T6: IdentityEq<$U6>, $T7: IdentityEq<$U7>, $T8: IdentityEq<$U8>, $T9: IdentityEq<$U9>, $TA: IdentityEq<$UA>, $TB: IdentityEq<$UB>, $TC: IdentityEq<$UC>, $TD: IdentityEq<$UD>,
            <$T0 as IdentityEq<$U0>>::Out: crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>,
            <<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out: crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>,
            <<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out: crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>,
            <<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out: crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>,
            <<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out: crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>,
            <<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out: crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>,
            <<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out: crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>,
            <<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out: crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>,
            <<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out: crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>,
            <<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out: crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>,
            <<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out: crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>,
            <<<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out as crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>>::Out: crate::primitives::BoolAnd<<$TC as IdentityEq<$UC>>::Out>,
            <<<<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out as crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>>::Out as crate::primitives::BoolAnd<<$TC as IdentityEq<$UC>>::Out>>::Out: crate::primitives::BoolAnd<<$TD as IdentityEq<$UD>>::Out>,
         {
             type Out = <<<<<<<<<<<<<<$T0 as IdentityEq<$U0>>::Out
                 as crate::primitives::BoolAnd<<$T1 as IdentityEq<$U1>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T2 as IdentityEq<$U2>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T3 as IdentityEq<$U3>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T4 as IdentityEq<$U4>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T5 as IdentityEq<$U5>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T6 as IdentityEq<$U6>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T7 as IdentityEq<$U7>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T8 as IdentityEq<$U8>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$T9 as IdentityEq<$U9>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TA as IdentityEq<$UA>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TB as IdentityEq<$UB>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TC as IdentityEq<$UC>>::Out>>::Out
                 as crate::primitives::BoolAnd<<$TD as IdentityEq<$UD>>::Out>>::Out;
         }
    };
}


impl_tuple_ids!(T0 U0, T1 U1);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7, T8 U8);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7, T8 U8, T9 U9);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7, T8 U8, T9 U9, TA UA);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7, T8 U8, T9 U9, TA UA, TB UB);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7, T8 U8, T9 U9, TA UA, TB UB, TC UC);
impl_tuple_ids!(T0 U0, T1 U1, T2 U2, T3 U3, T4 U4, T5 U5, T6 U6, T7 U7, T8 U8, T9 U9, TA UA, TB UB, TC UC, TD UD);




// =============================================================================
// Mismatched Length Implementations
// =============================================================================


// This is still awkward because params need to be expanded.

// I'll manually paste ~20 critical combinations or valid ranges.
// Or just handle 1..8 vs 1..8 robustly?

impl_mismatch!([T0], [U0, U1]);
impl_mismatch!([T0], [U0, U1, U2]);
impl_mismatch!([T0], [U0, U1, U2, U3]);
impl_mismatch!([T0], [U0, U1, U2, U3, U4]);

impl_mismatch!([T0, T1], [U0]);
impl_mismatch!([T0, T1], [U0, U1, U2]);
impl_mismatch!([T0, T1], [U0, U1, U2, U3]);
impl_mismatch!([T0, T1], [U0, U1, U2, U3, U4]);

impl_mismatch!([T0, T1, T2], [U0]);
impl_mismatch!([T0, T1, T2], [U0, U1]);
impl_mismatch!([T0, T1, T2], [U0, U1, U2, U3]);
impl_mismatch!([T0, T1, T2], [U0, U1, U2, U3, U4]);

impl_mismatch!([T0, T1, T2, T3], [U0]);
impl_mismatch!([T0, T1, T2, T3], [U0, U1]);
impl_mismatch!([T0, T1, T2, T3], [U0, U1, U2]);
impl_mismatch!([T0, T1, T2, T3], [U0, U1, U2, U3, U4]);

impl_mismatch!([T0, T1, T2, T3, T4], [U0]);
impl_mismatch!([T0, T1, T2, T3, T4], [U0, U1]);
impl_mismatch!([T0, T1, T2, T3, T4], [U0, U1, U2]);
impl_mismatch!([T0, T1, T2, T3, T4], [U0, U1, U2, U3]);


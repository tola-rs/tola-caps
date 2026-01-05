//! Packed primitive types for high-performance identity storage.
//!
//! We use `u128` "segments" to store 16 bytes of data at once. This reduces type depth
//! by 16x compared to storing individual characters.
//!
//! Combined with a flat Tuple structure, this keeps the compiler happy and fast.

use core::marker::PhantomData;
use crate::primitives::bool::{Bool, BoolAnd, Present};

// =============================================================================
// Segment - The "Bus" that carries 16 bytes (32 Nibbles)
// =============================================================================

/// A packed segment holding 16 bytes (32 Nibbles) of data.
/// We use Nibbles because they are comparable on Stable Rust, whereas const u128 is not.
#[allow(clippy::type_complexity)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Segment<
    N0, N1, N2, N3, N4, N5, N6, N7,
    N8, N9, N10, N11, N12, N13, N14, N15,
    N16, N17, N18, N19, N20, N21, N22, N23,
    N24, N25, N26, N27, N28, N29, N30, N31
>(PhantomData<(
    N0, N1, N2, N3, N4, N5, N6, N7,
    N8, N9, N10, N11, N12, N13, N14, N15,
    N16, N17, N18, N19, N20, N21, N22, N23,
    N24, N25, N26, N27, N28, N29, N30, N31
)>);



// =============================================================================
// Tuple Equality - Comparing the whole train
// =============================================================================

/// Trait to compare tuples of Segments.
///
/// We implement this for tuples of various sizes. The `Out` type will only be
/// `Present` if ALL segments match.
pub trait TupleEq<Other> {
    type Out: Bool;
}

// Empty tuple (base case)
impl TupleEq<()> for () {
    type Out = Present;
}

// Nested tuple support: treat last element as potential tail for recursive structures.
// This keeps type structure flat (O(log n) depth) instead of deep cons lists.

/// Generalized item equality supporting nested tuple comparison.
pub trait ItemEq<Other> {
    type Out: Bool;
}

// Manual impl for Segment using TupleEq of 4 8-tuples.
impl<
    // L-side generics
    L0, L1, L2, L3, L4, L5, L6, L7,
    L8, L9, L10, L11, L12, L13, L14, L15,
    L16, L17, L18, L19, L20, L21, L22, L23,
    L24, L25, L26, L27, L28, L29, L30, L31,
    // R-side generics
    R0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12, R13, R14, R15,
    R16, R17, R18, R19, R20, R21, R22, R23,
    R24, R25, R26, R27, R28, R29, R30, R31
> ItemEq<Segment<
    R0, R1, R2, R3, R4, R5, R6, R7,
    R8, R9, R10, R11, R12, R13, R14, R15,
    R16, R17, R18, R19, R20, R21, R22, R23,
    R24, R25, R26, R27, R28, R29, R30, R31
>> for Segment<
    L0, L1, L2, L3, L4, L5, L6, L7,
    L8, L9, L10, L11, L12, L13, L14, L15,
    L16, L17, L18, L19, L20, L21, L22, L23,
    L24, L25, L26, L27, L28, L29, L30, L31
>
where
    // Group 1
    (L0, L1, L2, L3, L4, L5, L6, L7): TupleEq<(R0, R1, R2, R3, R4, R5, R6, R7)>,
    // Group 2
    (L8, L9, L10, L11, L12, L13, L14, L15): TupleEq<(R8, R9, R10, R11, R12, R13, R14, R15)>,
    // Group 3
    (L16, L17, L18, L19, L20, L21, L22, L23): TupleEq<(R16, R17, R18, R19, R20, R21, R22, R23)>,
    // Group 4
    (L24, L25, L26, L27, L28, L29, L30, L31): TupleEq<(R24, R25, R26, R27, R28, R29, R30, R31)>,
{
    type Out = <
        <(L0, L1, L2, L3, L4, L5, L6, L7) as TupleEq<(R0, R1, R2, R3, R4, R5, R6, R7)>>::Out // G1
        as BoolAnd<
            <
                <(L8, L9, L10, L11, L12, L13, L14, L15) as TupleEq<(R8, R9, R10, R11, R12, R13, R14, R15)>>::Out // G2
                as BoolAnd<
                    <
                        <(L16, L17, L18, L19, L20, L21, L22, L23) as TupleEq<(R16, R17, R18, R19, R20, R21, R22, R23)>>::Out // G3
                        as BoolAnd<
                            <(L24, L25, L26, L27, L28, L29, L30, L31) as TupleEq<(R24, R25, R26, R27, R28, R29, R30, R31)>>::Out // G4
                        >
                    >::Out
                >
            >::Out
        >
    >::Out;
}

// Nibble ItemEq implementations (explicit for each nibble type)
// Using NibbleEq to determine equality
macro_rules! impl_nibble_item_eq {
    ($($N:ident),*) => {
        $(
            impl<Other: crate::primitives::nibble::Nibble> ItemEq<Other> for crate::primitives::nibble::$N
            where
                crate::primitives::nibble::$N: crate::primitives::nibble::NibbleEq<Other>,
            {
                type Out = <crate::primitives::nibble::$N as crate::primitives::nibble::NibbleEq<Other>>::Out;
            }
        )*
    };
}

impl_nibble_item_eq!(X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF);

// Now `impl_tuple_eq_recur` needs to use `ItemEq` instead of `SegmentEq`
// Updated macro below:

macro_rules! impl_tuple_eq_general {
    ($([$T:ident, $U:ident]),*) => {
        impl<$($T, $U),*> TupleEq<($($U,)*)> for ($($T,)*)
        where
            $($T: ItemEq<$U>),*  // Changed from SegmentEq to ItemEq
        {
            type Out = impl_tuple_eq_general!(@combine $($T::Out),*);
        }
    };

    (@combine $First:ty) => { $First };
    (@combine $First:ty, $($Rest:ty),+) => {
        <$First as BoolAnd<impl_tuple_eq_general!(@combine $($Rest),+)>>::Out
    };
}

// Re-generate with the general macro (up to 32 elements)
impl_tuple_eq_general!([T0, U0]);
impl_tuple_eq_general!([T0, U0], [T1, U1]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25], [T26, U26]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25], [T26, U26], [T27, U27]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25], [T26, U26], [T27, U27], [T28, U28]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25], [T26, U26], [T27, U27], [T28, U28], [T29, U29]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25], [T26, U26], [T27, U27], [T28, U28], [T29, U29], [T30, U30]);
impl_tuple_eq_general!([T0, U0], [T1, U1], [T2, U2], [T3, U3], [T4, U4], [T5, U5], [T6, U6], [T7, U7], [T8, U8], [T9, U9], [T10, U10], [T11, U11], [T12, U12], [T13, U13], [T14, U14], [T15, U15], [T16, U16], [T17, U17], [T18, U18], [T19, U19], [T20, U20], [T21, U21], [T22, U22], [T23, U23], [T24, U24], [T25, U25], [T26, U26], [T27, U27], [T28, U28], [T29, U29], [T30, U30], [T31, U31]);

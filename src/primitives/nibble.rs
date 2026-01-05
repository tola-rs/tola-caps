//! Type-level nibble system (4-bit values X0-XF).
//!
//! Nibbles are used for hash-based routing in the 16-ary trie.

use super::bool::{Bool, Present, Absent};

// =============================================================================
// Nibble iteration macros
// =============================================================================

/// Iterate over all 16 nibbles (X0..XF).
#[macro_export]
macro_rules! for_each_nibble {
    ($mac:ident) => {
        $mac!(X0); $mac!(X1); $mac!(X2); $mac!(X3);
        $mac!(X4); $mac!(X5); $mac!(X6); $mac!(X7);
        $mac!(X8); $mac!(X9); $mac!(XA); $mac!(XB);
        $mac!(XC); $mac!(XD); $mac!(XE); $mac!(XF);
    };
}

/// Iterate over all 16 (Nibble, SlotName) pairs.
#[macro_export]
macro_rules! for_each_nibble_pair {
    ($mac:ident) => {
        $mac!(X0, N0); $mac!(X1, N1); $mac!(X2, N2); $mac!(X3, N3);
        $mac!(X4, N4); $mac!(X5, N5); $mac!(X6, N6); $mac!(X7, N7);
        $mac!(X8, N8); $mac!(X9, N9); $mac!(XA, NA); $mac!(XB, NB);
        $mac!(XC, NC); $mac!(XD, ND); $mac!(XE, NE); $mac!(XF, NF);
    };
}

/// Generate impls for all distinct pairs (A, B) and (B, A) where A != B.
#[macro_export]
macro_rules! for_distinct_pairs {
    ($mac:ident) => {
        $crate::for_distinct_pairs!(@recurse $mac, [X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF]);
    };
    (@recurse $mac:ident, [$head:ident, $($tail:ident),*]) => {
        $(
            $mac!($head, $tail);
            $mac!($tail, $head);
        )*
        $crate::for_distinct_pairs!(@recurse $mac, [$($tail),*]);
    };
    (@recurse $mac:ident, [$last:ident]) => {};
}

// =============================================================================
// Nibble trait and types
// =============================================================================

/// Type-level nibble (4-bit value, 0..15)
pub trait Nibble: 'static {}

// Define structs X0..XF and implement Nibble
macro_rules! define_nibble {
    ($n:ident) => {
        pub struct $n;
        impl Nibble for $n {}
    };
}
for_each_nibble!(define_nibble);

// =============================================================================
// Const to Type Mapping (Map<N> -> Xn)
// =============================================================================

pub trait ToNibble {
    type Out: Nibble;
}

pub struct Map<const N: u8>;

impl ToNibble for Map<0> { type Out = X0; }
impl ToNibble for Map<1> { type Out = X1; }
impl ToNibble for Map<2> { type Out = X2; }
impl ToNibble for Map<3> { type Out = X3; }
impl ToNibble for Map<4> { type Out = X4; }
impl ToNibble for Map<5> { type Out = X5; }
impl ToNibble for Map<6> { type Out = X6; }
impl ToNibble for Map<7> { type Out = X7; }
impl ToNibble for Map<8> { type Out = X8; }
impl ToNibble for Map<9> { type Out = X9; }
impl ToNibble for Map<10> { type Out = XA; }
impl ToNibble for Map<11> { type Out = XB; }
impl ToNibble for Map<12> { type Out = XC; }
impl ToNibble for Map<13> { type Out = XD; }
impl ToNibble for Map<14> { type Out = XE; }
impl ToNibble for Map<15> { type Out = XF; }
// Default fallbacks handled by compiler error if N > 15 (unmatchable on stable unless we impl for all u8? No, just 0-15 is enough for nibbles).

// =============================================================================
// Nibble equality
// =============================================================================

/// Type-level nibble equality
pub trait NibbleEq<Other: Nibble>: Nibble {
    type Out: Bool;
}

// Self-equality: X == X → Present
macro_rules! impl_eq_self {
    ($($n:ident),*) => { $(impl NibbleEq<$n> for $n { type Out = Present; })* };
}
impl_eq_self!(X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF);

// Cross-inequality: X != Y → Absent
macro_rules! impl_neq { ($a:ident, $b:ident) => { impl NibbleEq<$b> for $a { type Out = Absent; } }; }
for_distinct_pairs!(impl_neq);

// =============================================================================
// HexAnd (Bitwise AND)
// =============================================================================

/// Bitwise AND of two Nibbles.
pub trait HexAnd<Other: Nibble>: Nibble { type Out: Nibble; }

/// Truth table for bitwise AND.
/// Each row: `Xa => [Xa&X0, Xa&X1, ..., Xa&XF]`
macro_rules! hex_and_table {
    ($($row:ident => [$($val:ident),*]),* $(,)?) => {
        $(
            hex_and_table!(@impl $row, [$($val),*], [X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF]);
        )*
    };
    (@impl $row:ident, [$($val:ident),*], [$($col:ident),*]) => {
        $(impl HexAnd<$col> for $row { type Out = $val; })*
    };
}

hex_and_table! {
    //        0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    X0 => [  X0, X0, X0, X0, X0, X0, X0, X0, X0, X0, X0, X0, X0, X0, X0, X0 ],
    X1 => [  X0, X1, X0, X1, X0, X1, X0, X1, X0, X1, X0, X1, X0, X1, X0, X1 ],
    X2 => [  X0, X0, X2, X2, X0, X0, X2, X2, X0, X0, X2, X2, X0, X0, X2, X2 ],
    X3 => [  X0, X1, X2, X3, X0, X1, X2, X3, X0, X1, X2, X3, X0, X1, X2, X3 ],
    X4 => [  X0, X0, X0, X0, X4, X4, X4, X4, X0, X0, X0, X0, X4, X4, X4, X4 ],
    X5 => [  X0, X1, X0, X1, X4, X5, X4, X5, X0, X1, X0, X1, X4, X5, X4, X5 ],
    X6 => [  X0, X0, X2, X2, X4, X4, X6, X6, X0, X0, X2, X2, X4, X4, X6, X6 ],
    X7 => [  X0, X1, X2, X3, X4, X5, X6, X7, X0, X1, X2, X3, X4, X5, X6, X7 ],
    X8 => [  X0, X0, X0, X0, X0, X0, X0, X0, X8, X8, X8, X8, X8, X8, X8, X8 ],
    X9 => [  X0, X1, X0, X1, X0, X1, X0, X1, X8, X9, X8, X9, X8, X9, X8, X9 ],
    XA => [  X0, X0, X2, X2, X0, X0, X2, X2, X8, X8, XA, XA, X8, X8, XA, XA ],
    XB => [  X0, X1, X2, X3, X0, X1, X2, X3, X8, X9, XA, XB, X8, X9, XA, XB ],
    XC => [  X0, X0, X0, X0, X4, X4, X4, X4, X8, X8, X8, X8, XC, XC, XC, XC ],
    XD => [  X0, X1, X0, X1, X4, X5, X4, X5, X8, X9, X8, X9, XC, XD, XC, XD ],
    XE => [  X0, X0, X2, X2, X4, X4, X6, X6, X8, X8, XA, XA, XC, XC, XE, XE ],
    XF => [  X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, XA, XB, XC, XD, XE, XF ],
}

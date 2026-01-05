//! Const-to-Type Bridge
//!
//! Converts compile-time constant values (u8, u128) to type-level representations.
//! This enables using `const fn` hash computations with type-level capability IDs.

use crate::primitives::nibble::*;

/// Converts a const `u8` (0-15) to a Nibble type.
///
/// Usage in type position:
/// ```ignore
/// type N = <() as ToNibble<{ (hash >> 4) & 0xF }>>::Out;
/// ```
pub trait ToNibble<const N: u8> {
    type Out: Nibble;
}

impl ToNibble<0> for () { type Out = X0; }
impl ToNibble<1> for () { type Out = X1; }
impl ToNibble<2> for () { type Out = X2; }
impl ToNibble<3> for () { type Out = X3; }
impl ToNibble<4> for () { type Out = X4; }
impl ToNibble<5> for () { type Out = X5; }
impl ToNibble<6> for () { type Out = X6; }
impl ToNibble<7> for () { type Out = X7; }
impl ToNibble<8> for () { type Out = X8; }
impl ToNibble<9> for () { type Out = X9; }
impl ToNibble<10> for () { type Out = XA; }
impl ToNibble<11> for () { type Out = XB; }
impl ToNibble<12> for () { type Out = XC; }
impl ToNibble<13> for () { type Out = XD; }
impl ToNibble<14> for () { type Out = XE; }
impl ToNibble<15> for () { type Out = XF; }

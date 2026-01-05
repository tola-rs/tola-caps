//! Type-level byte comparison using const generics.
//!
//! Uses const generics for exact byte comparison at the type level.

use crate::primitives::bool::{Bool, Present, Absent, BoolAnd};

/// Compare two const u8 values at type level.
pub trait ByteEq<const A: u8, const B: u8> {
    type Out: Bool;
}

// Generate all 65536 impls using a proc-macro
// DISABLED: This generates too many impls and may slow compilation.
// macros::impl_byte_eq!();

// Simplified: Only implement for same values (equality check)
impl<const V: u8> ByteEq<V, V> for () {
    type Out = Present;
}

// =============================================================================
// Type-level Path: A linked list of bytes
// =============================================================================

use core::marker::PhantomData;

/// A single byte in the path
pub struct B<const V: u8>;

/// Path cons cell: head byte + tail
pub struct PCons<Head, Tail>(PhantomData<(Head, Tail)>);

/// Empty path (end marker)
pub struct PNil;

// =============================================================================
// PathEq: Compare two paths for equality
// =============================================================================

/// Compare two type-level paths
pub trait PathEq<Other> {
    type Out: Bool;
}

// Empty == Empty -> Present
impl PathEq<PNil> for PNil {
    type Out = Present;
}

// Empty != Non-empty -> Absent
impl<const V: u8, Tail> PathEq<PCons<B<V>, Tail>> for PNil {
    type Out = Absent;
}

// Non-empty != Empty -> Absent
impl<const V: u8, Tail> PathEq<PNil> for PCons<B<V>, Tail> {
    type Out = Absent;
}

// Compare head bytes, then recurse on tails
// Note: Using different const names (VA, VB) to avoid conflict with type B<V>
impl<const VA: u8, const VB: u8, TailA, TailB> PathEq<PCons<B<VB>, TailB>> for PCons<B<VA>, TailA>
where
    (): ByteEq<VA, VB>,
    TailA: PathEq<TailB>,
    <() as ByteEq<VA, VB>>::Out: BoolAnd<<TailA as PathEq<TailB>>::Out>,
{
    type Out = <<() as ByteEq<VA, VB>>::Out as BoolAnd<<TailA as PathEq<TailB>>::Out>>::Out;
}

// =============================================================================
// PathIdentity: Wrapper for use as Capability::Identity
// =============================================================================

/// Identity type using type-level path encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathIdentity<Path>(PhantomData<Path>);

use crate::primitives::identity::IdentityEq;

impl<P1, P2> IdentityEq<PathIdentity<P2>> for PathIdentity<P1>
where
    P1: PathEq<P2>,
{
    type Out = <P1 as PathEq<P2>>::Out;
}

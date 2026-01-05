//! Const evaluation utilities

use core::marker::PhantomData;
use super::bool::{Bool, Present, Absent};

/// Get string length in a const context
pub const fn str_len(s: &str) -> usize {
    s.len()
}

/// Compare two strings for equality in a const context
pub const fn str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// FNV-1a 64-bit Hash for strings (const fn)
pub const fn fnv1a_64_str(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

// =============================================================================
// 512-bit Hash (4 × 128-bit)
// =============================================================================
//
// Cascaded hashing with different seeds for collision resistance.

/// FNV-1a 128-bit with custom seed
pub const fn fnv1a_128_seeded(s: &str, seed: u128) -> u128 {
    let bytes = s.as_bytes();
    let prime: u128 = 309485009821345068724781371;
    let mut hash: u128 = seed;
    let mut i = 0;
    while i < bytes.len() {
        hash = hash ^ (bytes[i] as u128);
        hash = hash.wrapping_mul(prime);
        i += 1;
    }
    hash
}

/// Generate 512-bit hash as 4 × u128
/// Each component uses different seed to ensure independence
pub const fn hash_512_h0(s: &str) -> u128 {
    fnv1a_128_seeded(s, 0xcbf29ce484222325cbf29ce484222325)
}

pub const fn hash_512_h1(s: &str) -> u128 {
    let h0 = hash_512_h0(s);
    fnv1a_128_seeded(s, 0x100000001b3100000001b3 ^ h0.rotate_left(64))
}

pub const fn hash_512_h2(s: &str) -> u128 {
    let h1 = hash_512_h1(s);
    fnv1a_128_seeded(s, 0x84222325cbf29ce484222325cbf29ce4 ^ h1.rotate_left(32))
}

pub const fn hash_512_h3(s: &str) -> u128 {
    let h2 = hash_512_h2(s);
    fnv1a_128_seeded(s, 0x1b3100000001b310000000100000001 ^ h2.rotate_left(96))
}

/// Extract nibble N (0-127) from 512-bit hash
pub const fn hash_512_nibble(s: &str, n: usize) -> u8 {
    let (hash, shift) = if n < 32 {
        (hash_512_h0(s), n * 4)
    } else if n < 64 {
        (hash_512_h1(s), (n - 32) * 4)
    } else if n < 96 {
        (hash_512_h2(s), (n - 64) * 4)
    } else {
        (hash_512_h3(s), (n - 96) * 4)
    };
    ((hash >> shift) & 0xF) as u8
}

/// Extract nibble N (0-15) from 64-bit FNV-1a hash
/// Used for HashStream16 generation from module paths
pub const fn hash_nibble(s: &str, n: u8) -> u8 {
    let hash = fnv1a_64_str(s);
    ((hash >> (n * 4)) & 0xF) as u8
}

// =============================================================================
// Raw Byte → Nibble extraction
// =============================================================================
//
// Extract nibbles directly from string bytes without hashing.

/// Get nibble at position N from string
/// N=0 → high nibble of byte 0, N=1 → low nibble of byte 0, etc.
pub const fn get_nibble(s: &str, n: u8) -> u8 {
    let bytes = s.as_bytes();
    let byte_idx = (n / 2) as usize;
    let is_high = n.is_multiple_of(2);

    if byte_idx < bytes.len() {
        if is_high {
            (bytes[byte_idx] >> 4) & 0xF
        } else {
            bytes[byte_idx] & 0xF
        }
    } else {
        0
    }
}

// =============================================================================
// Raw Byte Packing
// =============================================================================
//
// Pack string bytes directly into u128 without hashing.

/// Pack bytes [offset, offset+16) of string into u128
/// Pads with 0 if string is shorter
pub const fn pack_bytes_u128(s: &str, offset: usize) -> u128 {
    let bytes = s.as_bytes();
    let mut result: u128 = 0;
    let mut i = 0;
    while i < 16 {
        let idx = offset + i;
        let byte = if idx < bytes.len() { bytes[idx] } else { 0 };
        result |= (byte as u128) << (i * 8);
        i += 1;
    }
    result
}

/// Pack first 64 bytes of string into 4 × u128
/// For ByteStream<C0, C1, C2, C3>
pub const fn pack_c0(s: &str) -> u128 { pack_bytes_u128(s, 0) }
pub const fn pack_c1(s: &str) -> u128 { pack_bytes_u128(s, 16) }
pub const fn pack_c2(s: &str) -> u128 { pack_bytes_u128(s, 32) }
pub const fn pack_c3(s: &str) -> u128 { pack_bytes_u128(s, 48) }

/// FNV-1a 128-bit Hash
///
/// Parameters from FNV spec:
/// offset_basis: 144066263297769815596495629667062367629
/// prime: 309485009821345068724781371
pub const fn fnv1a_128(input: &[u8]) -> u128 {
    let prime: u128 = 309485009821345068724781371;
        let mut hash: u128 = 144066263297769815596495629667062367629;

    let mut i = 0;
    while i < input.len() {
        hash = hash ^ (input[i] as u128);
        hash = hash.wrapping_mul(prime);
        i += 1;
    }
    hash
}

/// FNV-1a 128-bit Hash for strings (const fn)
pub const fn fnv1a_128_str(s: &str) -> u128 {
    fnv1a_128(s.as_bytes())
}

/// Concatenate path and name and hash them
pub const fn hash_path_name(path: &str, name: &str) -> u128 {
    // We can't allocate a string, so we must feed bytes sequentially to hash.
    // Re-impl hashing to handle two slices.
    let prime: u128 = 309485009821345068724781371;
    let mut hash: u128 = 144066263297769815596495629667062367629;

    // Hash path
    let p = path.as_bytes();
    let mut i = 0;
    while i < p.len() {
        hash = hash ^ (p[i] as u128);
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    // Hash separator "::"
    hash = hash ^ (b':' as u128); hash = hash.wrapping_mul(prime);
    hash = hash ^ (b':' as u128); hash = hash.wrapping_mul(prime);

    // Hash name
    let n = name.as_bytes();
    i = 0;
    while i < n.len() {
        hash = hash ^ (n[i] as u128);
        hash = hash.wrapping_mul(prime);
        i += 1;
    }

    hash
}

/// Get the byte at index `i` of the concatenated `path + "::" + name` string.
/// Returns 0 if index is out of bounds.
pub const fn get_path_name_byte(path: &str, name: &str, idx: usize) -> u8 {
    let p = path.as_bytes();
    let n = name.as_bytes();
    let sep_len = 2; // "::"

    if idx < p.len() {
        return p[idx];
    }

    let idx_after_path = idx - p.len();
    if idx_after_path < sep_len {
        return b':';
    }

    let idx_after_sep = idx_after_path - sep_len;
    if idx_after_sep < n.len() {
        return n[idx_after_sep];
    }

    0 // Out of bounds / Padding
}

// =============================================================================
// PathIdentity
// =============================================================================

/// Pack 16 bytes of a string starting at offset into a u128.
/// Returns 0 for bytes beyond string length.
pub const fn pack_bytes_16(path: &str, name: &str, offset: usize) -> u128 {
    let mut result: u128 = 0;
    let mut i = 0;
    while i < 16 {
        let byte = get_path_name_byte(path, name, offset + i);
        result |= (byte as u128) << (i * 8);
        i += 1;
    }
    result
}

// =============================================================================
// UniqueId - Hybrid Identity (Type-Hash + Type-Body)
// =============================================================================

use crate::primitives::identity::IdentityEq;
use crate::primitives::pack::TupleEq;


/// Hybrid Unique Identity.
///
/// - `HASH`: A type-level tuple (32 nibbles) for O(1) matching.
/// - `BODY`: A type-level tuple of `Segment` (32 nibbles each) for full path verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueId<Hash, Body>(PhantomData<(Hash, Body)>);

// -----------------------------------------------------------------------------
// IdentityEq Logic with Lazy Gating
// -----------------------------------------------------------------------------

impl<H1, H2, B1, B2> IdentityEq<UniqueId<H2, B2>> for UniqueId<H1, B1>
where
    // 1. Compare Hash (Type-Level). Returns Present or Absent.
    H1: TupleEq<H2>,
    // 2. Gate the Body check based on Hash result.
    //    If HashResult is Absent, this trait implementation simply returns Absent
    //    WITHOUT resolving `B1: TupleEq<B2>`. True Lazy!
    <H1 as TupleEq<H2>>::Out: GateBodyEq<B1, B2>,
{
    type Out = <<H1 as TupleEq<H2>>::Out as GateBodyEq<B1, B2>>::Out;
}

/// Gate trait for lazy body evaluation.
pub trait GateBodyEq<B1, B2> {
    type Out: Bool;
}

// Case 1: Hash Mismatch (Absent) -> Short-circuit
impl<B1, B2> GateBodyEq<B1, B2> for Absent {
    type Out = Absent;
}

// Case 2: Hash Match (Present) -> Compare Bodies
impl<B1, B2> GateBodyEq<B1, B2> for Present
where
    B1: crate::primitives::pack::ItemEq<B2>,
{
    type Out = <B1 as crate::primitives::pack::ItemEq<B2>>::Out;
}

// =============================================================================
// ConstIdentity
// =============================================================================
//
// Stores up to 256 bytes of path data in 16 x u128 const generic parameters.
// Different const values = different types = exact comparison.

/// Pack 16 bytes of a string starting at offset into a u128.
/// Returns 0 for bytes beyond string length.
pub const fn pack_str_chunk(s: &str, offset: usize) -> u128 {
    let bytes = s.as_bytes();
    let mut result: u128 = 0;
    let mut i = 0;
    while i < 16 {
        let idx = offset + i;
        let byte = if idx < bytes.len() { bytes[idx] } else { 0 };
        result |= (byte as u128) << (i * 8);
        i += 1;
    }
    result
}

// =============================================================================
// CharIdentity - Type-Level Character Identity
// =============================================================================
//
// Tiered IList for exact string matching at compile time:
// - Tier 1-4: Full IList for strings ≤64 chars
// - Tier 5:   Smart sampling (head+mid+tail) for >64 chars

/// Character type with const generic
pub struct C<const CHAR: char>;

/// HList node: head + tail
pub struct IList<Head, Tail>(PhantomData<(Head, Tail)>);

/// HList terminator
pub struct INil;

/// Collision marker - triggers compile error if two >64 char strings collide.
#[allow(non_camel_case_types)]
pub struct COLLISION_DETECTED_INCREASE_SAMPLING_SIZE;

// Tiered wrapper types - each tier is a distinct type
pub struct IList8<T>(PhantomData<T>);      // ≤8 chars
pub struct IList16<T>(PhantomData<T>);     // ≤16 chars
pub struct IList32<T>(PhantomData<T>);     // ≤32 chars
pub struct IList64<T>(PhantomData<T>);     // ≤64 chars
pub struct IListSampled<T>(PhantomData<T>); // >64 chars (sampled)

// =============================================================================
// String → Identity Helper Functions
// =============================================================================

/// Smart sampling for strings > 64 chars
/// Strategy: head(32) + mid(16) + tail(16) = 64 samples
/// This maximizes differentiation for long strings
pub const fn sample_indices_64(len: usize) -> [usize; 64] {
    let mut indices = [0usize; 64];
    if len <= 64 {
        // Short string: take all chars
        let mut i = 0;
        while i < 64 {
            indices[i] = if i < len { i } else { len }; // Out of bounds → '\0'
            i += 1;
        }
    } else {
        // Long string: sample head(32) + mid(16) + tail(16)
        let mut i = 0;
        // Head: first 32 chars
        while i < 32 {
            indices[i] = i;
            i += 1;
        }
        // Mid: sample from middle 16 chars
        let mid_start = (len - 16) / 2;
        let mut j = 0;
        while j < 16 {
            indices[32 + j] = mid_start + j;
            j += 1;
        }
        // Tail: last 16 chars
        let mut k = 0;
        while k < 16 {
            indices[48 + k] = len - 16 + k;
            k += 1;
        }
    }
    indices
}

/// Extract character at position N from string
/// Returns '\0' for out-of-bounds positions
pub const fn str_char_at(s: &str, n: usize) -> char {
    let bytes = s.as_bytes();
    if n >= bytes.len() {
        return '\0';
    }

    // Simple ASCII for now (module paths are typically ASCII)
    // For Unicode, we'd need full UTF-8 decoding
    let byte = bytes[n];
    if byte < 128 {
        byte as char
    } else {
        // For non-ASCII, use the byte value as char (limited Unicode support)
        // This is safe for const fn context
        '?'  // Placeholder for non-ASCII
    }
}

/// Get effective length for identity (max 64 chars)
pub const fn identity_len(s: &str) -> usize {
    let len = s.len();
    if len > 64 { 64 } else { len }
}

/// DEPRECATED: Old TypeMarker approach (no longer used)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeMarker<T>(PhantomData<T>);

impl<T> IdentityEq<TypeMarker<T>> for TypeMarker<T> {
    type Out = Present;
}

// =============================================================================
// IdentityBytes
// =============================================================================
//
// Stores up to 64 bytes of module_path in 4 × u128 const generic parameters.
// Different const values = different types = exact identity comparison.

/// Identity type storing up to 64 bytes of path data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentityBytes<
    const B0: u128, const B1: u128, const B2: u128, const B3: u128
>(PhantomData<()>);

/// Const fn to pack 16 bytes of a string into a u128.
pub const fn pack_str_bytes(s: &str, offset: usize) -> u128 {
    let bytes = s.as_bytes();
    let mut result: u128 = 0;
    let mut i = 0;
    while i < 16 {
        let idx = offset + i;
        let byte = if idx < bytes.len() { bytes[idx] } else { 0 };
        result |= (byte as u128) << (i * 8);
        i += 1;
    }
    result
}

// IdentityEq implementation: Compare ANY two IdentityBytes types
//
// Uses SelectBool to convert const comparison to type-level bool.
// This allows Bucket traversal to work: different Identity -> Absent -> continue search.
//
// Note: This requires `generic_const_exprs` feature on nightly, OR we need a different approach.
// For now, we keep the simple impl and rely on hash uniqueness to avoid Bucket comparisons.
impl<
    const B0: u128, const B1: u128, const B2: u128, const B3: u128,
> IdentityEq<IdentityBytes<B0, B1, B2, B3>> for IdentityBytes<B0, B1, B2, B3>
{
    type Out = Present;
}

// For different IdentityBytes, we need a blanket impl that returns Absent.
// But Rust doesn't allow overlapping impls without specialization.
//
// WORKAROUND: Since we use full module_path in both Stream hash AND Identity,
// two capabilities with the same Stream hash MUST have the same Identity
// (because they're from the same module with the same name).
// Therefore, Bucket collision (same hash, different Identity) should NEVER happen.
//
// If it does happen (extremely rare 64-bit hash collision), compilation fails safely.

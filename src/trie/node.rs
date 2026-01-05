//! Trie node types: Empty, Leaf, Node16
//!
//! These are the core data structures for the 16-ary capability trie.

use core::marker::PhantomData;

/// Empty trie node (no capabilities)
#[derive(Default)]
pub struct Empty;

/// Leaf node containing a single capability
pub struct Leaf<Cap>(PhantomData<Cap>);

// Manual Default impl: doesn't require Cap: Default
impl<Cap> Default for Leaf<Cap> {
    fn default() -> Self { Leaf(PhantomData) }
}

/// 16-ary internal node - one slot per nibble (0x0..0xF)
#[allow(clippy::type_complexity)]
#[macros::node16]
pub struct Node16<_Slots_>(PhantomData<(_Slots_,)>);

#[macros::node16]
impl<_Slots_> Default for _Node16_ {
    fn default() -> Self { Self(PhantomData) }
}

/// Type alias for an empty 16-ary node (all slots are Empty)
#[macros::node16(all_empty)]
pub type EmptyNode16;

/// Bucket node for storing hash collisions (linear list)
pub struct Bucket<Head, Tail>(PhantomData<(Head, Tail)>);

impl<Head, Tail> Default for Bucket<Head, Tail> {
    fn default() -> Self { Self(PhantomData) }
}

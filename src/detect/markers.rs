//! Capability markers for standard traits.
//!
//! Each marker is a zero-sized type with a unique hash stream for trie routing.

use crate::primitives::{Cons, ConstStream, GetTail, HashStream};
use crate::primitives::{X0, X1, X2, X3, X4, X5};
use crate::primitives::const_utils::TypeMarker;

// Import Capability from trie (but trie isn't enabled yet, so use capability.rs)
use crate::capability::Capability;

// =============================================================================
// Capability Markers
// =============================================================================

/// Marker for Clone detection (ID 0)
pub struct IsClone;
impl Capability for IsClone {
    type Stream = Cons<X0, ConstStream<X0>>;
    type Identity = TypeMarker<Self>;
    type At<D> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
    where
        Self::Stream: GetTail<D>;
}

/// Marker for Copy detection (ID 1)
pub struct IsCopy;
impl Capability for IsCopy {
    type Stream = Cons<X1, ConstStream<X0>>;
    type Identity = TypeMarker<Self>;
    type At<D> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
    where
        Self::Stream: GetTail<D>;
}

/// Marker for Debug detection (ID 2)
pub struct IsDebug;
impl Capability for IsDebug {
    type Stream = Cons<X2, ConstStream<X0>>;
    type Identity = TypeMarker<Self>;
    type At<D> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
    where
        Self::Stream: GetTail<D>;
}

/// Marker for Default detection (ID 3)
pub struct IsDefault;
impl Capability for IsDefault {
    type Stream = Cons<X3, ConstStream<X0>>;
    type Identity = TypeMarker<Self>;
    type At<D> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
    where
        Self::Stream: GetTail<D>;
}

/// Marker for Send detection (ID 4)
pub struct IsSend;
impl Capability for IsSend {
    type Stream = Cons<X4, ConstStream<X0>>;
    type Identity = TypeMarker<Self>;
    type At<D> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
    where
        Self::Stream: GetTail<D>;
}

/// Marker for Sync detection (ID 5)
pub struct IsSync;
impl Capability for IsSync {
    type Stream = Cons<X5, ConstStream<X0>>;
    type Identity = TypeMarker<Self>;
    type At<D> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
    where
        Self::Stream: GetTail<D>;
}

/// Empty Stream (for types with no capabilities)
pub type EmptyStream = ConstStream<X0>;

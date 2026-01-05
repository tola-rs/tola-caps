

//! # Layer 1: Trie Core
//!
//! The core routing and storage logic of the capability system.
//!
//! - **Nodes**: `Node16` (routing), `Leaf` (storage), `Bucket` (collision handling).
//! - **Operations**: `InsertAt` (put), `RemoveAt` (take), `Evaluate` (has/get).
//!
//! This layer handles the $O(1)$ routing via HashStreams and robust collision resolution via NameStreams.

pub mod node;
pub mod capability;
pub mod evaluate;
pub mod insert;
pub mod ops;
pub mod inspect;
pub mod aliases;

// Re-export key types at trie level
pub use node::{Empty, Leaf, Node16, EmptyNode16};
pub use capability::Capability;
pub use evaluate::{
    Evaluate, EvalAt, RouteQuery,
    Has, And, Or, Not, All, Any, HNil, HCons,
    IsTrue, Require,
};
pub use insert::{
    InsertAt, RemoveAt, NodeInsert, LeafInsert, LeafRemove, NodeRemove,
    MakeNode16WithLeaf, With, Without,
};
pub use ops::{
    SetUnion, SetIntersect, SupersetOf, SetAnd,
    IntersectLeafHelper, LeafAndDispatch, NodeAndDispatch,
};
pub use inspect::Inspect;
pub use aliases::{CapSet0, CapSet1, CapSet2, CapSet3, CapSet4};

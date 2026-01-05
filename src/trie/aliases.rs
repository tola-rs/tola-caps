//! Convenience types and macros for capability sets
//!
//! Provides type aliases and macros for easier capability set construction.

use super::node::Empty;
use super::insert::With;

// =============================================================================
// Convenience Type Aliases
// =============================================================================

/// Empty capability set
pub type CapSet0 = Empty;

/// Capability set with 1 capability
pub type CapSet1<A> = <Empty as With<A>>::Out;

/// Capability set with 2 capabilities
pub type CapSet2<A, B> = <<Empty as With<A>>::Out as With<B>>::Out;

/// Capability set with 3 capabilities
pub type CapSet3<A, B, C> = <<<Empty as With<A>>::Out as With<B>>::Out as With<C>>::Out;

/// Capability set with 4 capabilities
pub type CapSet4<A, B, C, D> = <<<<Empty as With<A>>::Out as With<B>>::Out as With<C>>::Out as With<D>>::Out;

// =============================================================================
// Convenience Macros
// =============================================================================

// Note: hlist!, all!, any! are defined in evaluate.rs

/// Macro to build a CapSet from multiple capabilities
/// Usage: `capset![CapA, CapB, CapC]`
#[macro_export]
macro_rules! capset {
    () => { $crate::trie::Empty };
    ($cap:ty) => {
        <$crate::trie::Empty as $crate::trie::With<$cap>>::Out
    };
    ($cap:ty, $($rest:ty),+ $(,)?) => {
        <$crate::capset![$($rest),+] as $crate::trie::With<$cap>>::Out
    };
}

/// Macro to compute union of two capability sets
/// Usage: `union![SetA, SetB]`
#[macro_export]
macro_rules! union {
    ($a:ty, $b:ty) => {
        <$a as $crate::trie::SetUnion<$b>>::Out
    };
}

/// Macro to compute intersection of two capability sets
/// Usage: `intersect![SetA, SetB]`
#[macro_export]
macro_rules! intersect {
    ($a:ty, $b:ty) => {
        <$a as $crate::trie::SetIntersect<$b>>::Out
    };
}

/// Macro to add a capability to a set
/// Usage:
/// - `with![Set, Cap]` -> Single add
/// - `with![Set, A, B]` -> Chain add (A then B)
/// - `with![Cap]` -> Implicit `__C`
#[macro_export]
macro_rules! with {
    ($cap:ty) => {
        <__C as $crate::trie::With<$cap>>::Out
    };
    ($set:ty, $cap:ty) => {
        <$set as $crate::trie::With<$cap>>::Out
    };
    ($set:ty, $cap:ty, $($rest:ty),+) => {
        $crate::with![ $crate::with![$set, $cap], $($rest),+ ]
    };
}

/// Macro to remove a capability from a set
/// Usage:
/// - `without![Set, Cap]` -> Remove Cap from Set
/// - `without![Set, A, B]` -> Remove A then B
/// - `without![Cap]` -> Remove Cap from implicit `__C`
#[macro_export]
macro_rules! without {
    ($cap:ty) => {
        <__C as $crate::trie::Without<$cap>>::Out
    };
    ($set:ty, $cap:ty) => {
        <$set as $crate::trie::Without<$cap>>::Out
    };
    ($set:ty, $cap:ty, $($rest:ty),+) => {
        $crate::without![ $crate::without![$set, $cap], $($rest),+ ]
    };
}

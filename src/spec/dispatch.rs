//! Type-Level Dispatch System
//!
//! Provides zero-overhead method dispatch based on capabilities.
//! Uses pure type selection instead of `if` branches.
//!
//! ## Core Concepts
//!
//! 1. `SelectCap<Query, Then, Else>` - Selects `Then` or `Else` based on capability
//! 2. Uses `Bool::If<Then, Else>` for type-level selection
//! 3. Works with existing `Evaluate` trait
//!
//! ## Example
//!
//! ```ignore
//! // Define implementations as types
//! struct CloneImpl;
//! struct DefaultImpl;
//!
//! impl<T> MethodImpl<T> for CloneImpl { ... }
//! impl<T> MethodImpl<T> for DefaultImpl { ... }
//!
//! // Select at type level
//! type Selected = <Cap<MyType> as SelectCap<IsClone, CloneImpl, DefaultImpl>>::Out;
//! Selected::method(&value)  // No if branch!
//! ```

use crate::primitives::{Bool, Present, Absent};
use crate::trie::{Evaluate, And, Or, Not};

// Re-export select traits from detect module (only when detect feature is enabled)
#[cfg(feature = "detect")]
pub use crate::detect::{
    SelectClone, SelectCopy, SelectDebug, SelectDefault,
    SelectSend, SelectSync, SelectEq, SelectPartialEq,
    SelectOrd, SelectPartialOrd, SelectHash,
    SelectDisplay, SelectSized, SelectUnpin,
};

// =============================================================================
// SelectCap Trait - Core Type-Level Selector
// =============================================================================

/// Selects between two types based on a capability query.
///
/// Given a capability set `S`, a query `Q`, and two candidate types `Then`/`Else`:
/// - If `S: Evaluate<Q, Out = Present>`, returns `Then`
/// - If `S: Evaluate<Q, Out = Absent>`, returns `Else`
pub trait SelectCap<Q, Then, Else> {
    type Out;
}

impl<S, Q, Then, Else> SelectCap<Q, Then, Else> for S
where
    S: Evaluate<Q>,
    <S as Evaluate<Q>>::Out: Bool,
{
    type Out = <<S as Evaluate<Q>>::Out as Bool>::If<Then, Else>;
}

/// Call a static method based on capability selection.
///
/// This trait enables calling `StaticMethodImpl::call()` through type-level selection
/// without requiring the caller to prove that `SelectCap::Out` implements `StaticMethodImpl`.
///
/// `Then` and `Else` must implement `StaticMethodImpl<Output>`.
pub trait SelectStaticCall<Q, Then, Else, Output> {
    fn call() -> Output;
}

impl<S, Q, Then, Else, Output> SelectStaticCall<Q, Then, Else, Output> for S
where
    S: Evaluate<Q>,
    <S as Evaluate<Q>>::Out: Bool,
    Then: StaticMethodImpl<Output>,
    Else: StaticMethodImpl<Output>,
{
    #[inline(always)]
    fn call() -> Output {
        <S as Evaluate<Q>>::Out::static_dispatch::<Then, Else, Output>()
    }
}

/// Helper trait for calling static methods through Bool selection.
///
/// This trait is implemented for `Present` and `Absent` to enable
/// type-level branching for static method calls.
pub trait BoolStaticCall<Then, Else, Output> {
    fn call() -> Output;
}

impl<Then, Else, Output> BoolStaticCall<Then, Else, Output> for Present
where
    Then: StaticMethodImpl<Output>,
{
    #[inline(always)]
    fn call() -> Output {
        Then::call()
    }
}

impl<Then, Else, Output> BoolStaticCall<Then, Else, Output> for Absent
where
    Else: StaticMethodImpl<Output>,
{
    #[inline(always)]
    fn call() -> Output {
        Else::call()
    }
}

/// Wrapper type for chaining static method selections.
///
/// This type wraps a `SelectStaticCall` to itself implement `StaticMethodImpl`,
/// enabling nested selections.
pub struct StaticSelect<Cap, Q, Then, Else>(
    core::marker::PhantomData<(Cap, Q, Then, Else)>
);

impl<Cap, Q, Then, Else, Output> StaticMethodImpl<Output> for StaticSelect<Cap, Q, Then, Else>
where
    Cap: SelectStaticCall<Q, Then, Else, Output>,
{
    #[inline(always)]
    fn call() -> Output {
        <Cap as SelectStaticCall<Q, Then, Else, Output>>::call()
    }
}

// =============================================================================
// Compound Selectors (And, Or, Not)
// =============================================================================

/// Select based on multiple capabilities (AND)
pub trait SelectAnd<Q1, Q2, Then, Else> {
    type Out;
}

impl<S, Q1, Q2, Then, Else> SelectAnd<Q1, Q2, Then, Else> for S
where
    S: SelectCap<And<Q1, Q2>, Then, Else>,
{
    type Out = <S as SelectCap<And<Q1, Q2>, Then, Else>>::Out;
}

/// Select based on either capability (OR)
pub trait SelectOr<Q1, Q2, Then, Else> {
    type Out;
}

impl<S, Q1, Q2, Then, Else> SelectOr<Q1, Q2, Then, Else> for S
where
    S: SelectCap<Or<Q1, Q2>, Then, Else>,
{
    type Out = <S as SelectCap<Or<Q1, Q2>, Then, Else>>::Out;
}

/// Select based on absence of capability (NOT)
pub trait SelectNot<Q, Then, Else> {
    type Out;
}

impl<S, Q, Then, Else> SelectNot<Q, Then, Else> for S
where
    S: SelectCap<Not<Q>, Then, Else>,
{
    type Out = <S as SelectCap<Not<Q>, Then, Else>>::Out;
}

// =============================================================================
// Method Implementation Trait
// =============================================================================

/// Trait for method implementations that can be type-selected.
pub trait MethodImpl<T: ?Sized, Output = ()> {
    fn call(value: &T) -> Output;
}

/// Trait for static/associated function implementations (no self parameter).
pub trait StaticMethodImpl<Output = ()> {
    fn call() -> Output;
}

// =============================================================================
// Type Selection Trait (for associated types)
// =============================================================================

/// Selects between two types based on a capability query (for associated types).
pub trait SelectType<Q, Then, Else> {
    type Out;
}

impl<S, Q, Then, Else> SelectType<Q, Then, Else> for S
where
    S: Evaluate<Q>,
    <S as Evaluate<Q>>::Out: Bool,
{
    type Out = <<S as Evaluate<Q>>::Out as Bool>::If<Then, Else>;
}

/// Trait for type selectors - used by specialize! for associated types.
pub trait TypeSelector {
    type Out;
}

// =============================================================================
// NoImpl - Fallback for when no implementation matches
// =============================================================================

/// Marker type for when no specialized implementation is available.
pub struct NoImpl;

impl<T: ?Sized, Output: Default> MethodImpl<T, Output> for NoImpl {
    #[inline(always)]
    fn call(_value: &T) -> Output {
        Output::default()
    }
}

impl<Output: Default> StaticMethodImpl<Output> for NoImpl {
    #[inline(always)]
    fn call() -> Output {
        Output::default()
    }
}

impl TypeSelector for NoImpl {
    type Out = ();
}

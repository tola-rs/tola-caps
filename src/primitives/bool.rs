//! Type-level boolean logic.
//!
//! Core types: `Present` (true), `Absent` (false), `Bool` trait.

use crate::spec::dispatch::StaticMethodImpl;

/// Type-level boolean.
pub trait Bool: 'static {
    const VALUE: bool;
    /// Type-level conditional: If<Then, Else> (General Type Selector)
    type If<Then, Else>;

    /// Type-level boolean conditional: Then<T, E> where T, E are Bool.
    /// Returns a type guaranteed to implement Bool.
    type Elif<Then: Bool, Else: Bool>: Bool;

    /// Logical AND
    type And<Other: Bool>: Bool;

    /// Logical OR
    type Or<Other: Bool>: Bool;

    /// Call a static method based on this boolean value.
    /// If true (Present), calls Then::call().
    /// If false (Absent), calls Else::call().
    fn static_dispatch<Then, Else, Output>() -> Output
    where
        Then: StaticMethodImpl<Output>,
        Else: StaticMethodImpl<Output>;
}

/// Type-level True.
#[derive(Debug)]
pub struct Present;

/// Type-level False.
#[derive(Debug)]
pub struct Absent;

impl Bool for Present {
    const VALUE: bool = true;
    type If<Then, Else> = Then;
    type Elif<Then: Bool, Else: Bool> = Then;

    type And<Other: Bool> = Other;
    type Or<Other: Bool> = Present;

    #[inline(always)]
    fn static_dispatch<Then, Else, Output>() -> Output
    where
        Then: StaticMethodImpl<Output>,
        Else: StaticMethodImpl<Output>,
    {
        Then::call()
    }
}

impl Bool for Absent {
    const VALUE: bool = false;
    type If<Then, Else> = Else;
    type Elif<Then: Bool, Else: Bool> = Else;

    type And<Other: Bool> = Absent;
    type Or<Other: Bool> = Other;

    #[inline(always)]
    fn static_dispatch<Then, Else, Output>() -> Output
    where
        Then: StaticMethodImpl<Output>,
        Else: StaticMethodImpl<Output>,
    {
        Else::call()
    }
}

// Deprecated separate traits (kept for compatibility if needed, or remove?)
// Let's alias them to the new associated types to minimize breakage
pub trait BoolAnd<Other: Bool>: Bool {
    type Out: Bool;
}
impl<A: Bool, B: Bool> BoolAnd<B> for A {
    type Out = A::And<B>;
}

pub trait BoolOr<Other: Bool>: Bool {
    type Out: Bool;
}
impl<A: Bool, B: Bool> BoolOr<B> for A {
    type Out = A::Or<B>;
}




/// Type-level NOT.
pub trait BoolNot: Bool {
    type Out: Bool;
}

impl BoolNot for Present {
    type Out = Absent;
}

impl BoolNot for Absent {
    type Out = Present;
}

/// Convert const bool to type-level Bool.
pub trait SelectBool<const B: bool> {
    type Out: Bool;
}

impl SelectBool<true> for () {
    type Out = Present;
}

impl SelectBool<false> for () {
    type Out = Absent;
}

/// Conditional Type Alias
pub type If<const C: bool, T, E> = <<() as SelectBool<C>>::Out as Bool>::If<T, E>;

/// Strict Conditional Type Alias (Result is Bool)
pub type Elif<const C: bool, T, E> = <<() as SelectBool<C>>::Out as Bool>::Elif<T, E>;


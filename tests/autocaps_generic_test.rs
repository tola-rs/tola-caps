//! Comprehensive tests for AutoCaps with generic types and trait bounds.
//!
//! This test module is organized into the following sections:
//! 1. Helper macros for assertions
//! 2. Custom trait definitions (for testing `#[trait_autocaps]`)
//! 3. Type definitions (for testing `#[derive(AutoCaps)]`)
//! 4. Test modules grouped by functionality

use std::fmt::Debug;
use std::rc::Rc;
use tola_caps::prelude::*;
use tola_caps::{AutoCaps, trait_autocaps};

// ============================================================================
// PART 1: HELPER MACROS
// ============================================================================

/// Assert that a type implements the given trait(s).
/// Usage: `assert_caps!(Type: Trait)` or `assert_caps!(Type: Trait1 & Trait2)`
macro_rules! assert_caps {
    ($ty:ty : $($trait_expr:tt)+) => {
        assert!(
            caps_check!($ty: $($trait_expr)+),
            "Expected `{}` to satisfy: {}",
            stringify!($ty),
            stringify!($($trait_expr)+)
        );
    };
}

/// Assert that a type does NOT implement the given trait(s).
/// Usage: `assert_not_caps!(Type: Trait)` or `assert_not_caps!(Type: Trait1 & Trait2)`
macro_rules! assert_not_caps {
    ($ty:ty : $($trait_expr:tt)+) => {
        assert!(
            !caps_check!($ty: $($trait_expr)+),
            "Expected `{}` NOT to satisfy: {}",
            stringify!($ty),
            stringify!($($trait_expr)+)
        );
    };
}

// ============================================================================
// PART 2: CUSTOM TRAIT DEFINITIONS
// ============================================================================

// --- Simple custom traits ---

#[trait_autocaps]
trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

#[trait_autocaps]
trait Deserializable {
    fn deserialize(data: &[u8]) -> Self;
}

// --- Super trait (parent-child relationship) ---

#[trait_autocaps]
trait SerializableExt: Serializable {
    fn serialize_compressed(&self) -> Vec<u8>;
}

// --- Generic parameter traits ---

#[trait_autocaps]
trait Converter<T> {
    fn convert(&self) -> T;
}

#[trait_autocaps]
trait Transform<From, To> {
    fn transform(input: From) -> To;
}

// --- Lifetime traits ---

#[trait_autocaps]
trait Borrowable<'a> {
    fn borrow_data(&'a self) -> &'a [u8];
}

#[trait_autocaps]
trait Reference<'a, T: 'a> {
    fn get_ref(&'a self) -> &'a T;
}

// --- Where clause trait ---

#[trait_autocaps]
trait Processor<T>
where
    T: Clone + Send,
{
    fn process(&self, data: T) -> T;
}

// --- Associated type trait ---

#[trait_autocaps]
trait Container {
    type Item;
    fn get_item(&self) -> Self::Item;
}

// --- Const generic trait ---

#[trait_autocaps]
trait FixedBuffer<const N: usize> {
    fn buffer(&self) -> [u8; N];
}

// --- Multiple where clause constraints ---

#[trait_autocaps]
trait MultiConstraint<T, U>
where
    T: Clone + Send + Sync,
    U: Default + Debug,
{
    fn constrained_op(&self, t: T) -> U;
}

// ============================================================================
// PART 3: TYPE DEFINITIONS
// ============================================================================

// --- Basic generic types ---

#[derive(Clone, Debug, AutoCaps)]
struct Simple<T> {
    value: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct WithCloneBound<T: Clone> {
    value: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct WithMultipleBounds<T: Clone + Send + Sync> {
    value: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct MultiGeneric<T: Clone, U: Send> {
    t_value: T,
    u_value: U,
}

#[derive(Clone, Debug, AutoCaps)]
struct WithWhereClause<T>
where
    T: Clone + Send,
{
    value: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct WithLifetime<'a, T: Clone + 'a> {
    reference: &'a T,
    value: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct WithConstGeneric<T: Clone, const N: usize> {
    array: [T; N],
}

#[derive(Copy, Clone, Debug, AutoCaps)]
struct CopyGeneric<T: Copy> {
    value: T,
}

// --- Negative test types (missing derives) ---

#[derive(Debug, AutoCaps)] // NO Clone!
struct NoCloneGeneric<T> {
    value: T,
}

#[derive(Clone, AutoCaps)] // NO Debug!
struct NoDebugGeneric<T: Clone> {
    value: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct WithNonSend<T> {
    value: T,
}

// --- Custom trait implementation types ---

#[derive(Clone, Debug, AutoCaps)]
struct ImplementsSerializable<T> {
    data: T,
}

impl<T> Serializable for ImplementsSerializable<T> {
    fn serialize(&self) -> Vec<u8> {
        vec![1, 2, 3]
    }
}

#[derive(Clone, Debug, AutoCaps)]
struct NoSerializable<T> {
    data: T,
}

#[derive(Clone, Debug, AutoCaps)]
struct ConditionalSerializable<T> {
    data: T,
}

impl<T: Serializable> Serializable for ConditionalSerializable<T> {
    fn serialize(&self) -> Vec<u8> {
        self.data.serialize()
    }
}

// --- Super trait implementation ---

#[derive(Clone, Debug, AutoCaps)]
struct ExtendedSerializer<T> {
    data: T,
}

impl<T> Serializable for ExtendedSerializer<T> {
    fn serialize(&self) -> Vec<u8> {
        vec![1, 2, 3]
    }
}

impl<T> SerializableExt for ExtendedSerializer<T> {
    fn serialize_compressed(&self) -> Vec<u8> {
        vec![1]
    }
}

// --- Generic trait implementations ---

#[derive(Clone, Debug, AutoCaps)]
struct DualConverter<A, B> {
    a: A,
    b: B,
}

impl<A, B: Clone> Converter<B> for DualConverter<A, B> {
    fn convert(&self) -> B {
        self.b.clone()
    }
}

impl<A: Clone, B: Default> Transform<A, B> for DualConverter<A, B> {
    fn transform(_input: A) -> B {
        B::default()
    }
}

// --- Lifetime trait implementations ---

#[derive(Clone, Debug, AutoCaps)]
struct BorrowableWrapper<'a, T: 'a> {
    reference: &'a T,
    owned: Vec<u8>,
}

impl<'a, T: 'a> Borrowable<'a> for BorrowableWrapper<'a, T> {
    fn borrow_data(&'a self) -> &'a [u8] {
        &self.owned
    }
}

impl<'a, T: 'a> Reference<'a, T> for BorrowableWrapper<'a, T> {
    fn get_ref(&'a self) -> &'a T {
        self.reference
    }
}

// --- Where clause trait implementations ---

#[derive(Clone, Debug, AutoCaps)]
struct ComplexProcessor<T, U>
where
    T: Clone + Send + Sync,
    U: Clone,
{
    processor_data: T,
    output: U,
}

impl<T, U> Processor<T> for ComplexProcessor<T, U>
where
    T: Clone + Send + Sync,
    U: Clone,
{
    fn process(&self, data: T) -> T {
        data
    }
}

// --- Associated type trait implementations ---

#[derive(Clone, Debug, AutoCaps)]
struct VecContainer<T> {
    items: Vec<T>,
}

impl<T: Clone> Container for VecContainer<T> {
    type Item = T;
    fn get_item(&self) -> Self::Item {
        self.items[0].clone()
    }
}

// --- Const generic trait implementations ---

#[derive(Clone, Debug, AutoCaps)]
struct FixedArray<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> FixedBuffer<N> for FixedArray<T, N>
where
    T: Into<u8> + Copy,
{
    fn buffer(&self) -> [u8; N] {
        self.data.map(|x| x.into())
    }
}

// --- Nested generic types ---

#[derive(Clone, Debug, AutoCaps)]
struct NestedOption<T> {
    value: Option<T>,
}

#[derive(Clone, Debug, AutoCaps)]
struct NestedVec<T> {
    items: Vec<Option<T>>,
}

#[derive(Clone, Debug, AutoCaps)]
struct DeeplyNested<T> {
    data: Vec<Option<Box<T>>>,
}

// --- Empty generic parameter type ---

#[derive(Clone, Debug, AutoCaps)]
struct EmptyGeneric<T> {
    _phantom: std::marker::PhantomData<T>,
}

// --- Default type parameter ---

#[derive(Clone, Debug, AutoCaps)]
struct WithDefault<T = String> {
    value: T,
}

// --- Multiple where clause type ---

#[derive(Clone, Debug, AutoCaps)]
struct MultiWhereType<T, U>
where
    T: Clone + Send + Sync,
    U: Default + Debug,
{
    t: T,
    u: U,
}

impl<T, U> MultiConstraint<T, U> for MultiWhereType<T, U>
where
    T: Clone + Send + Sync,
    U: Default + Debug,
{
    fn constrained_op(&self, _t: T) -> U {
        U::default()
    }
}

// ============================================================================
// PART 4: TEST MODULES
// ============================================================================

/// Tests for basic generic type detection (standard traits)
mod standard_trait_tests {
    use super::*;

    #[test]
    fn simple_generic() {
        // Positive
        assert_caps!(Simple<String>: Clone);
        assert_caps!(Simple<String>: Debug);
        assert_caps!(Simple<i32>: Clone);
        assert_caps!(Simple<i32>: Debug);
        // Negative
        assert_not_caps!(Simple<i32>: Copy);
        assert_not_caps!(Simple<String>: Copy);
    }

    #[test]
    fn with_clone_bound() {
        // Positive
        assert_caps!(WithCloneBound<String>: Clone);
        assert_caps!(WithCloneBound<String>: Debug);
        // Negative
        assert_not_caps!(WithCloneBound<String>: Copy);
    }

    #[test]
    fn with_multiple_bounds() {
        // Positive
        assert_caps!(WithMultipleBounds<String>: Clone);
        assert_caps!(WithMultipleBounds<String>: Debug);
        assert_caps!(WithMultipleBounds<String>: Send);
        assert_caps!(WithMultipleBounds<String>: Sync);
        // Negative
        assert_not_caps!(WithMultipleBounds<String>: Copy);
    }

    #[test]
    fn multi_generic() {
        // Positive
        assert_caps!(MultiGeneric<String, i32>: Clone);
        assert_caps!(MultiGeneric<String, i32>: Debug);
        assert_caps!(MultiGeneric<String, i32>: Send);
        // Negative
        assert_not_caps!(MultiGeneric<String, i32>: Copy);
    }

    #[test]
    fn with_where_clause() {
        // Positive
        assert_caps!(WithWhereClause<String>: Clone);
        assert_caps!(WithWhereClause<String>: Debug);
        assert_caps!(WithWhereClause<String>: Send);
        // Negative
        assert_not_caps!(WithWhereClause<String>: Copy);
    }

    #[test]
    fn with_lifetime() {
        // Positive
        assert_caps!(WithLifetime<'_, String>: Clone);
        assert_caps!(WithLifetime<'_, String>: Debug);
        assert_caps!(WithLifetime<'static, i32>: Clone);
        // Negative
        assert_not_caps!(WithLifetime<'_, String>: Copy);
    }

    #[test]
    fn with_const_generic() {
        // Positive
        assert_caps!(WithConstGeneric<i32, 5>: Clone);
        assert_caps!(WithConstGeneric<i32, 5>: Debug);
        assert_caps!(WithConstGeneric<String, 10>: Clone);
        // Different sizes
        assert_caps!(WithConstGeneric<u8, 0>: Clone);
        assert_caps!(WithConstGeneric<u8, 1024>: Clone);
        // Negative
        assert_not_caps!(WithConstGeneric<i32, 5>: Copy);
    }

    #[test]
    fn copy_generic() {
        // Positive - Copy is implemented
        assert_caps!(CopyGeneric<i32>: Clone);
        assert_caps!(CopyGeneric<i32>: Copy);
        assert_caps!(CopyGeneric<i32>: Debug);
        // Different Copy types
        assert_caps!(CopyGeneric<u8>: Copy);
        assert_caps!(CopyGeneric<bool>: Copy);
        assert_caps!(CopyGeneric<char>: Copy);
    }

    #[test]
    fn nested_generic_types() {
        // NestedOption
        assert_caps!(NestedOption<String>: Clone);
        assert_caps!(NestedOption<String>: Debug);
        assert_not_caps!(NestedOption<String>: Copy);

        // NestedVec - Vec<Option<T>>
        assert_caps!(NestedVec<i32>: Clone);
        assert_caps!(NestedVec<i32>: Debug);
        assert_not_caps!(NestedVec<i32>: Copy);

        // DeeplyNested - Vec<Option<Box<T>>>
        assert_caps!(DeeplyNested<String>: Clone);
        assert_caps!(DeeplyNested<String>: Debug);
        assert_not_caps!(DeeplyNested<String>: Copy);
    }

    #[test]
    fn empty_generic_parameter() {
        // PhantomData type
        assert_caps!(EmptyGeneric<String>: Clone);
        assert_caps!(EmptyGeneric<String>: Debug);
        assert_caps!(EmptyGeneric<i32>: Clone);
        // Even with non-Clone type parameter (PhantomData is special)
        assert_caps!(EmptyGeneric<Rc<i32>>: Clone);
        assert_not_caps!(EmptyGeneric<i32>: Copy);
    }

    #[test]
    fn default_type_parameter() {
        // With default type
        assert_caps!(WithDefault: Clone);
        assert_caps!(WithDefault: Debug);
        // Explicit type
        assert_caps!(WithDefault<i32>: Clone);
        assert_caps!(WithDefault<Vec<u8>>: Clone);
        assert_not_caps!(WithDefault<String>: Copy);
    }
}

/// Tests for negative cases (missing trait implementations)
mod negative_tests {
    use super::*;

    #[test]
    fn no_clone_generic() {
        // Negative - NO Clone derive
        assert_not_caps!(NoCloneGeneric<String>: Clone);
        assert_not_caps!(NoCloneGeneric<i32>: Clone);
        // Positive - Debug is derived
        assert_caps!(NoCloneGeneric<String>: Debug);
        assert_caps!(NoCloneGeneric<i32>: Debug);
    }

    #[test]
    fn no_debug_generic() {
        // Negative - NO Debug derive
        assert_not_caps!(NoDebugGeneric<String>: Debug);
        assert_not_caps!(NoDebugGeneric<i32>: Debug);
        // Positive - Clone is derived
        assert_caps!(NoDebugGeneric<String>: Clone);
        assert_caps!(NoDebugGeneric<i32>: Clone);
    }

    #[test]
    fn send_detection_with_non_send_type() {
        // Rc<i32> is NOT Send
        assert_not_caps!(Rc<i32>: Send);
        // WithNonSend inherits Send from type parameter
        assert_not_caps!(WithNonSend<Rc<i32>>: Send);
        // But with Send type, it IS Send
        assert_caps!(WithNonSend<String>: Send);
        assert_caps!(WithNonSend<i32>: Send);
    }

    #[test]
    fn sync_detection_with_non_sync_type() {
        use std::cell::Cell;
        // Cell<i32> is NOT Sync
        assert_not_caps!(Cell<i32>: Sync);
        // WithNonSend inherits Sync from type parameter
        assert_not_caps!(WithNonSend<Cell<i32>>: Sync);
        // But with Sync type, it IS Sync
        assert_caps!(WithNonSend<String>: Sync);
    }

    #[test]
    fn copy_detection_critical() {
        // String: Clone but NOT Copy
        assert_caps!(String: Clone);
        assert_not_caps!(String: Copy);

        // CRITICAL: Simple<i32> where i32 IS Copy
        // But Simple didn't derive Copy, so it should NOT be Copy
        assert_caps!(Simple<i32>: Clone);
        assert_not_caps!(Simple<i32>: Copy);

        // Simple<String> also not Copy
        assert_not_caps!(Simple<String>: Copy);

        // CopyGeneric DID derive Copy
        assert_caps!(CopyGeneric<i32>: Copy);
    }
}

/// Tests for custom trait detection
mod custom_trait_tests {
    use super::*;

    #[test]
    fn trait_implemented() {
        // ImplementsSerializable unconditionally implements Serializable
        assert_caps!(ImplementsSerializable<String>: Serializable);
        assert_caps!(ImplementsSerializable<i32>: Serializable);
        assert_caps!(ImplementsSerializable<Vec<u8>>: Serializable);
    }

    #[test]
    fn trait_not_implemented() {
        // NoSerializable does NOT implement Serializable
        assert_not_caps!(NoSerializable<String>: Serializable);
        assert_not_caps!(NoSerializable<i32>: Serializable);
        // But has standard traits
        assert_caps!(NoSerializable<String>: Clone);
        assert_caps!(NoSerializable<String>: Debug);
    }

    #[test]
    fn conditional_implementation() {
        // Define a type that implements Serializable
        #[derive(Clone, Debug)]
        struct MyData;
        impl Serializable for MyData {
            fn serialize(&self) -> Vec<u8> {
                vec![42]
            }
        }

        // ConditionalSerializable<MyData> IS Serializable
        assert_caps!(ConditionalSerializable<MyData>: Serializable);
        // ConditionalSerializable<String> is NOT Serializable
        assert_not_caps!(ConditionalSerializable<String>: Serializable);
    }

    #[test]
    fn mixed_custom_and_standard() {
        // AND expression
        assert_caps!(ImplementsSerializable<String>: Clone & Serializable);
        assert_not_caps!(NoSerializable<String>: Clone & Serializable);

        // OR expression
        assert_caps!(NoSerializable<String>: Clone | Serializable);
        assert_caps!(ImplementsSerializable<String>: Clone | Serializable);

        // NOT expression
        assert_caps!(NoSerializable<String>: !Serializable);
        assert_not_caps!(ImplementsSerializable<String>: !Serializable);
    }
}

/// Tests for super trait detection
mod super_trait_tests {
    use super::*;

    #[test]
    fn super_trait_detection() {
        // ExtendedSerializer implements both
        assert_caps!(ExtendedSerializer<String>: Serializable);
        assert_caps!(ExtendedSerializer<String>: SerializableExt);
        // Combined
        assert_caps!(ExtendedSerializer<String>: Serializable & SerializableExt);
    }

    #[test]
    fn super_trait_negative() {
        // NoSerializable implements neither
        assert_not_caps!(NoSerializable<String>: Serializable);
        assert_not_caps!(NoSerializable<String>: SerializableExt);
        // ImplementsSerializable implements base but not ext
        assert_caps!(ImplementsSerializable<String>: Serializable);
        assert_not_caps!(ImplementsSerializable<String>: SerializableExt);
    }
}

/// Tests for generic trait parameters
mod generic_trait_tests {
    use super::*;

    #[test]
    fn single_generic_parameter() {
        // DualConverter implements Converter<B>
        assert_caps!(DualConverter<String, i32>: Converter<i32>);
        assert_caps!(DualConverter<Vec<u8>, String>: Converter<String>);
        // Negative - wrong type parameter
        assert_not_caps!(DualConverter<String, i32>: Converter<String>);
    }

    #[test]
    fn multiple_generic_parameters() {
        // DualConverter implements Transform<A, B>
        assert_caps!(DualConverter<String, i32>: Transform<String, i32>);
        assert_caps!(DualConverter<Vec<u8>, String>: Transform<Vec<u8>, String>);
        // Negative - wrong type parameters
        assert_not_caps!(DualConverter<String, i32>: Transform<i32, String>);
    }

    #[test]
    fn generic_trait_not_implemented() {
        // ExtendedSerializer does NOT implement Converter
        assert_not_caps!(ExtendedSerializer<String>: Converter<String>);
        assert_not_caps!(ExtendedSerializer<String>: Converter<i32>);
    }
}

/// Tests for lifetime trait detection
mod lifetime_trait_tests {
    use super::*;

    #[test]
    fn borrowable_trait() {
        // BorrowableWrapper implements Borrowable<'a>
        assert_caps!(BorrowableWrapper<'static, String>: Borrowable<'static>);
        assert_caps!(BorrowableWrapper<'static, i32>: Borrowable<'static>);
    }

    #[test]
    fn reference_trait() {
        // BorrowableWrapper implements Reference<'a, T>
        assert_caps!(BorrowableWrapper<'static, String>: Reference<'static, String>);
        assert_caps!(BorrowableWrapper<'static, i32>: Reference<'static, i32>);
    }

    #[test]
    fn lifetime_trait_negative() {
        // Simple does NOT implement Borrowable
        assert_not_caps!(Simple<String>: Borrowable<'static>);
        // NoSerializable does NOT implement Reference
        assert_not_caps!(NoSerializable<String>: Reference<'static, String>);
    }

    #[test]
    fn lifetime_trait_runtime_verification() {
        // Verify traits work at runtime
        let data = String::from("test");
        let owned = vec![1, 2, 3];
        let wrapper = BorrowableWrapper {
            reference: &data,
            owned: owned.clone(),
        };
        let _borrowed = wrapper.borrow_data();
        let _ref = wrapper.get_ref();

        // Standard traits still work
        assert_caps!(BorrowableWrapper<'static, String>: Clone);
        assert_caps!(BorrowableWrapper<'static, String>: Debug);
    }
}

/// Tests for where clause trait detection
mod where_clause_tests {
    use super::*;

    #[test]
    fn processor_trait() {
        // ComplexProcessor implements Processor<T>
        assert_caps!(ComplexProcessor<String, Vec<u8>>: Processor<String>);
        // Standard traits
        assert_caps!(ComplexProcessor<String, Vec<u8>>: Clone);
        assert_caps!(ComplexProcessor<String, Vec<u8>>: Debug);
    }

    #[test]
    fn processor_trait_negative() {
        // VecContainer does NOT implement Processor
        assert_not_caps!(VecContainer<i32>: Processor<i32>);
        // Simple does NOT implement Processor
        assert_not_caps!(Simple<String>: Processor<String>);
    }

    #[test]
    fn multi_constraint_trait() {
        // MultiWhereType implements MultiConstraint
        assert_caps!(MultiWhereType<String, i32>: MultiConstraint<String, i32>);
        // Standard traits
        assert_caps!(MultiWhereType<String, i32>: Clone);
        assert_caps!(MultiWhereType<String, i32>: Debug);
    }

    #[test]
    fn multi_constraint_negative() {
        // Simple does NOT implement MultiConstraint
        assert_not_caps!(Simple<String>: MultiConstraint<String, i32>);
        // ComplexProcessor does NOT implement MultiConstraint
        assert_not_caps!(ComplexProcessor<String, i32>: MultiConstraint<String, i32>);
    }
}

/// Tests for associated type trait detection
mod associated_type_tests {
    use super::*;

    #[test]
    fn container_trait() {
        // VecContainer implements Container
        assert_caps!(VecContainer<i32>: Container);
        assert_caps!(VecContainer<String>: Container);
        // Standard traits
        assert_caps!(VecContainer<i32>: Clone);
        assert_caps!(VecContainer<i32>: Debug);
    }

    #[test]
    fn container_trait_negative() {
        // ComplexProcessor does NOT implement Container
        assert_not_caps!(ComplexProcessor<String, Vec<u8>>: Container);
        // Simple does NOT implement Container
        assert_not_caps!(Simple<String>: Container);
    }
}

/// Tests for const generic trait detection
mod const_generic_trait_tests {
    use super::*;

    #[test]
    fn fixed_buffer_trait() {
        // FixedArray<u8, N> implements FixedBuffer<N>
        assert_caps!(FixedArray<u8, 16>: FixedBuffer<16>);
        assert_caps!(FixedArray<u8, 32>: FixedBuffer<32>);
        assert_caps!(FixedArray<u8, 0>: FixedBuffer<0>);
        assert_caps!(FixedArray<u8, 1024>: FixedBuffer<1024>);
    }

    #[test]
    fn fixed_buffer_standard_traits() {
        assert_caps!(FixedArray<u8, 16>: Clone);
        assert_caps!(FixedArray<u8, 16>: Debug);
        assert_not_caps!(FixedArray<u8, 16>: Copy);
    }

    #[test]
    fn fixed_buffer_negative() {
        // Simple does NOT implement FixedBuffer
        assert_not_caps!(Simple<u8>: FixedBuffer<16>);
        // VecContainer does NOT implement FixedBuffer
        assert_not_caps!(VecContainer<u8>: FixedBuffer<8>);
    }
}

/// Tests for complex boolean expressions
mod complex_expression_tests {
    use super::*;

    #[test]
    fn and_expressions() {
        assert_caps!(ExtendedSerializer<String>: Serializable & SerializableExt);
        assert_caps!(DualConverter<String, i32>: Converter<i32> & Transform<String, i32>);
        assert_caps!(ComplexProcessor<String, Vec<u8>>: Clone & Debug & Processor<String>);
    }

    #[test]
    fn or_expressions() {
        assert_caps!(NoSerializable<String>: Clone | Serializable);
        assert_caps!(ImplementsSerializable<String>: Serializable | Container);
    }

    #[test]
    fn not_expressions() {
        assert_caps!(NoSerializable<String>: Clone & !Serializable);
        assert_caps!(Simple<String>: Clone & !Copy);
    }

    #[test]
    fn parenthesized_expressions() {
        assert_caps!(
            ExtendedSerializer<String>: (Clone & Serializable) | (Debug & SerializableExt)
        );
        assert_caps!(
            ImplementsSerializable<String>: (Clone | Copy) & (Serializable | Container)
        );
    }

    #[test]
    fn complex_negative_expressions() {
        // Complex failure cases
        assert_not_caps!(NoSerializable<String>: Clone & Serializable);
        assert_not_caps!(Simple<String>: Clone & Copy);
        assert_not_caps!(NoSerializable<String>: (Serializable & Clone) | (SerializableExt & Debug));
    }
}

/// Cross-cutting tests combining multiple features
mod integration_tests {
    use super::*;

    #[test]
    fn all_standard_traits_combined() {
        // Test a type with all standard traits
        assert_caps!(WithMultipleBounds<String>: Clone & Debug & Send & Sync);
        assert_not_caps!(WithMultipleBounds<String>: Clone & Debug & Send & Sync & Copy);
    }

    #[test]
    fn custom_and_standard_combined() {
        assert_caps!(
            ImplementsSerializable<String>: Clone & Debug & Serializable & Send
        );
        assert_not_caps!(
            ImplementsSerializable<String>: Clone & Debug & Serializable & Copy
        );
    }

    #[test]
    fn generic_trait_complex() {
        assert_caps!(
            DualConverter<String, i32>: Clone & Debug & Converter<i32> & Transform<String, i32>
        );
        assert_not_caps!(
            DualConverter<String, i32>: Converter<i32> & Serializable
        );
    }

    #[test]
    fn lifetime_and_standard() {
        assert_caps!(
            BorrowableWrapper<'static, String>: Clone & Debug & Borrowable<'static> & Reference<'static, String>
        );
    }
}

//! Procedural macros for tola-caps capability system
//!
//! # Unified Macro API
//!
//! ## Core Macros (Recommended)
//!
//! | Macro | Target | Purpose |
//! |-------|--------|---------|
//! | `#[cap]` | trait | Register trait for caps system |
//! | `#[cap]` | struct/enum | Auto-detect std traits |
//! | `#[specialize]` | impl | Attribute-style specialization |
//! | `specialization!{}` | - | Block-style specialization |
//! | `caps![]` | - | Build capability set type |
//!
//! ## Example
//!
//! ```ignore
//! // 1. Define a trait with caps support
//! #[cap]
//! trait Serializable {
//!     fn serialize(&self) -> Vec<u8>;
//! }
//!
//! // 2. Use in specialization
//! specialization! {
//!     impl<T> Format for T {
//!         default fn format(&self) -> &'static str { "unknown" }
//!     }
//!     impl<T: Serializable> Format for T {
//!         fn format(&self) -> &'static str { "serializable" }
//!     }
//! }
//!
//! // 3. Check at compile time
//! if is_serializable::<MyType>() { ... }
//! ```

use proc_macro::TokenStream;
use syn::parse_macro_input;

// =============================================================================
// Module Declarations (Three-tier: inner / common / user)
// =============================================================================

mod inner;
mod common;
mod user;

// =============================================================================
// Internal Macros (inner/)
// =============================================================================

/// Generate Peano number type aliases D0..Dn.
///
/// # Usage
/// ```ignore
/// peano!(64);  // Generates D0 = Z, D1 = S<D0>, ..., D64 = S<D63>
/// ```
#[proc_macro]
pub fn peano(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as inner::peano::PeanoInput);
    inner::peano::expand_peano(input).into()
}

/// Generate HashStream impl for ByteStream128.
///
/// ByteStream128 stores 64 nibbles as const generic params.
/// This generates the impl that extracts Head (first nibble) and
/// creates Tail (rotated version of the params).
#[proc_macro]
pub fn impl_byte_stream_128(_input: TokenStream) -> TokenStream {
    inner::byte_stream::expand_impl_byte_stream_128().into()
}

/// Expand to: N0, N1, N2, N3, N4, N5, N6, N7, N8, N9, NA, NB, NC, ND, NE, NF
///
/// Use in generic parameters or tuple fields.
#[proc_macro]
pub fn node16_slots(_input: TokenStream) -> TokenStream {
    inner::node16::expand_slots().into()
}

/// Expand to: Node16<N0, N1, N2, ..., NF>
///
/// Use in impl blocks and type positions.
#[proc_macro]
pub fn node16_type(_input: TokenStream) -> TokenStream {
    inner::node16::expand_node16_type().into()
}

/// Generate all std trait detection infrastructure.
///
/// This generates:
/// - Capability markers (IsClone, IsCopy, etc.) with hash-based streams
/// - Fallback traits (CloneFallback, etc.)
/// - Select traits (SelectClone, etc.)
/// - AutoCaps trait with all IS_XXX consts
#[proc_macro]
pub fn define_std_traits(_input: TokenStream) -> TokenStream {
    inner::std_traits::expand_std_traits().into()
}

/// Generate the impl_auto_caps! macro.
#[proc_macro]
pub fn define_impl_auto_caps_macro(_input: TokenStream) -> TokenStream {
    inner::std_traits::expand_impl_auto_caps_macro().into()
}

/// Generate the impl_std_types! macro for primitives and core types.
#[proc_macro]
pub fn define_impl_std_types_macro(_input: TokenStream) -> TokenStream {
    inner::std_types::expand_impl_std_types_macro().into()
}

#[proc_macro]
pub fn define_impl_std_lib_types_macro(_input: TokenStream) -> TokenStream {
    inner::std_types::expand_impl_std_lib_types_macro().into()
}

/// Generate the impl_alloc_types! macro for alloc types.
#[proc_macro]
pub fn define_impl_alloc_types_macro(_input: TokenStream) -> TokenStream {
    inner::std_types::expand_impl_alloc_types_macro().into()
}


/// Internal: Convert a path string to a Finger Tree identity type.
/// Used by #[derive(Capability)] to create stable, reproducible identities.
/// The input must be: concat!(module_path!(), "::", stringify!(TypeName))
#[proc_macro]
pub fn __internal_make_identity(input: TokenStream) -> TokenStream {
    let _input_str = input.to_string();
    user::capability::expand_make_identity(input.into()).into()
}

/// Internal: Generate IdentityBytes type from module path string.
/// Uses const fn pack_str_bytes for const evaluation after concat!() expansion.
#[proc_macro]
pub fn make_identity_bytes(input: TokenStream) -> TokenStream {
    user::capability::expand_make_identity_bytes(input.into()).into()
}



/// Generate a type-level nibble stream from an identifier name.
///
/// Usage: `name_stream!(MyStruct)` expands to `Cons<X4, Cons<...> ...>`
#[proc_macro]
pub fn name_stream(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as inner::name_stream::NameStreamInput);
    inner::name_stream::expand_name_stream(input).into()
}

/// Internal: Compute routing hash stream from a full module path string.
/// Input must be a string literal (e.g. from `concat!`).
#[proc_macro]
pub fn make_routing_stream(input: TokenStream) -> TokenStream {
    // Just delegates to the same logic as identity, but returns Stream type tokens
    user::capability::expand_make_routing_stream(input.into()).into()
}

/// Implement `AutoCaps` and `AutoCapSet` for a type.
///
/// # Usage
/// ```ignore
/// impl_auto_caps!(MyType);
/// ```
#[proc_macro]
pub fn impl_auto_caps(input: TokenStream) -> TokenStream {
    inner::std_traits::expand_impl_auto_caps(input.into()).into()
}

/// Attribute macro for Node16 struct and impl blocks.
///
/// # Modes
///
/// - `#[node16]` - Basic placeholder replacement
/// - `#[node16(each_slot)]` - Per-slot expansion with `_Slot_` and `each(_Slots_)`
/// - `#[node16(for_nibble)]` - Generate 16 impls, one per nibble/slot pair
/// - `#[node16(all_empty)]` - Generate empty Node16 type alias
///
/// # Placeholders
///
/// - `_Slots_`  expands to N0, N1, ..., NF
/// - `_Node16_` expands to Node16<N0, N1, ..., NF>
/// - `each(_Slots_): Bound` expands to N0: Bound, ..., NF: Bound (each_slot mode)
/// - `_Slot_` in statements: repeated 16 times (each_slot mode)
/// - `_Nibble_` expands to X0, X1, ..., XF (for_nibble mode)
/// - `_SlotN_` expands to N0, N1, ..., NF (for_nibble mode, paired with _Nibble_)
///
/// # Examples
///
/// ```ignore
/// // Basic: struct and impl
/// #[macros::node16]
/// pub struct Node16<_Slots_>(PhantomData<(_Slots_,)>);
///
/// // Per-nibble impl (generates 16 impls)
/// #[macros::node16(for_nibble)]
/// impl<Cap, Depth, _Slots_> RouteQuery<Cap, Depth, _Nibble_> for _Node16_
/// where _SlotN_: EvalAt<Has<Cap>, S<Depth>>
/// {
///     type Out = <_SlotN_ as EvalAt<Has<Cap>, S<Depth>>>::Out;
/// }
/// ```
#[proc_macro_attribute]
pub fn node16(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr2: proc_macro2::TokenStream = attr.into();
    let item2: proc_macro2::TokenStream = item.into();
    let mode = inner::node16::parse_mode(attr2);
    inner::node16::expand_trie16_with_mode(mode, item2).into()
}

/// Generate all 65536 ByteEq impls for type-level byte comparison.
#[proc_macro]
pub fn impl_byte_eq(_input: TokenStream) -> TokenStream {
    inner::byte_eq::expand_byte_eq_impls().into()
}

// =============================================================================
// User-facing Macros (user/)
// =============================================================================

/// Unified capability constraint macro with boolean expression support.
///
/// # Usage
///
/// ```ignore
/// // Simple requirement
/// #[caps_bound(CanRead)]
/// fn read_doc<C>(doc: Doc<C>) { ... }
///
/// // Boolean logic
/// #[caps_bound(requires = CanRead & (CanWrite | CanAdmin), conflicts = CanGuest)]
/// fn secure_op<C>(doc: Doc<C>) { ... }
///
/// // Transparent mode (auto-inject generic)
/// #[caps_bound(CanRead, transparent)]
/// fn simple_read(doc: Doc) { ... }
/// ```
#[proc_macro_attribute]
pub fn caps_bound(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as user::CapsArgs);

    let item_clone = item.clone();
    if let Ok(func) = syn::parse::<syn::ItemFn>(item_clone.clone()) {
        return user::expand_caps_fn(args, func);
    }

    let item_clone = item.clone();
    if let Ok(item_struct) = syn::parse::<syn::ItemStruct>(item_clone.clone()) {
        return user::expand_caps_struct(args, item_struct);
    }

    let item_clone = item.clone();
    if let Ok(item_enum) = syn::parse::<syn::ItemEnum>(item_clone.clone()) {
        return user::expand_caps_enum(args, item_enum);
    }

    let item_clone = item.clone();
    if let Ok(item_impl) = syn::parse::<syn::ItemImpl>(item_clone) {
        return user::expand_caps_impl(args, item_impl);
    }

    syn::Error::new(
        proc_macro2::Span::call_site(),
        "caps_bound supports fn, struct, enum, or impl",
    )
    .to_compile_error()
    .into()
}

/// Create a capability set type from a list of capabilities.
///
/// # Usage
/// ```ignore
/// // Define capability set type
/// type MyCaps = caps![CanRead, CanWrite];
///
/// // Empty set
/// type NoCaps = caps![];
///
/// // Use in function signature
/// fn process<C: Evaluate<CanRead, Out = Present>>() { }
/// process::<caps![CanRead, CanWrite]>();
/// ```
#[proc_macro]
pub fn caps(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as user::CapsInput);
    let types: Vec<_> = input.types.into_iter().collect();

    if let Err(err) = user::check_duplicates(&types) {
        return err.to_compile_error().into();
    }

    user::build_capset(&types).into()
}

/// Create a capability set type (alias for `caps!`).
#[proc_macro]
pub fn cap_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as user::CapsInput);
    let types: Vec<_> = input.types.into_iter().collect();

    if let Err(err) = user::check_duplicates(&types) {
        return err.to_compile_error().into();
    }

    user::build_capset(&types).into()
}

/// Batch define capabilities with auto-generated `Cap` suffix.
///
/// # Usage
/// ```ignore
/// define_capabilities! {
///     LinksChecked => "Links have been checked",
///     LinksResolved => "Links have been resolved",
///     SvgOptimized => "SVG content has been optimized",
/// }
/// // Generates: LinksCheckedCap, LinksResolvedCap, SvgOptimizedCap
/// // Plus HasXxxCap and NotHasXxxCap traits for each
/// ```
#[proc_macro]
pub fn define_capabilities(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as user::DefineCapabilitiesInput);
    user::expand_define_capabilities(input).into()
}

/// Derive macro to automatically implement the `Capability` trait.
///
/// Computes a BLAKE3 hash of the struct name and generates a `HashStream` type.
///
/// # Usage
/// ```ignore
/// #[derive(Capability)]
/// struct CanRead;
///
/// #[derive(Capability)]
/// struct CanWrite;
///
/// // Now you can use:
/// type MyCaps = caps![CanRead, CanWrite];
/// ```
#[proc_macro_derive(Capability)]
pub fn derive_capability(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    user::expand_derive_capability(input).into()
}

/// Derive macro to auto-detect standard trait implementations.
///
/// This is the recommended way to enable `caps_check!` for user-defined types.
///
/// # Usage
/// ```ignore
/// #[derive(Clone, Debug, AutoCaps)]
/// struct MyType { data: String }
///
/// // Now you can check traits:
/// assert!(caps_check!(MyType: Clone));
/// assert!(caps_check!(MyType: Debug));
/// assert!(!caps_check!(MyType: Copy));
/// ```
#[proc_macro_derive(AutoCaps)]
pub fn derive_autocaps(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    user::expand_derive_autocaps(input).into()
}

/// Attribute macro to enable a trait for the caps system.
///
/// Use this on trait definitions to allow `caps_check!(T: TraitName)`.
///
/// # Usage
/// ```ignore
/// #[trait_autocaps]
/// trait Serializable {
///     fn serialize(&self) -> Vec<u8>;
/// }
///
/// // Now you can check:
/// assert!(caps_check!(MySerializableType: Serializable));
/// ```
#[proc_macro_attribute]
pub fn trait_autocaps(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr; // Unused for now
    user::expand_trait_autocaps(item)
}

/// Derive macro to create capability-tracked structs with PhantomData.
///
/// Automatically adds a `_caps: PhantomData<C>` field and conversion methods.
///
/// # Usage
/// ```ignore
/// #[derive(CapHolder)]
/// struct Doc {
///     content: String,
/// }
///
/// // Expands to:
/// // struct Doc<C = Empty> {
/// //     content: String,
/// //     _caps: PhantomData<C>,
/// // }
///
/// // Now use it with capabilities:
/// let doc: Doc<caps![Parsed]> = doc.with_caps();
/// ```
#[proc_macro_derive(CapHolder)]
pub fn derive_cap_holder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    user::capability::expand_derive_cap_holder(input).into()
}

/// #[specialize] attribute macro for distributed specialization across files.
///
/// Mark individual items as default with `#[specialize(default)]`.
/// Supports multi-generic constraints.
///
/// # Usage
/// ```ignore
/// trait MyTrait {
///     type Output;
///     fn describe(&self) -> Self::Output;
/// }
///
/// // Most general (default) implementation
/// #[specialize(default)]
/// impl<T> MyTrait for T {
///     type Output = ();
///     fn describe(&self) -> Self::Output { () }
/// }
///
/// // More specific - with constraint
/// #[specialize(T: Clone)]
/// impl<T> MyTrait for T {
///     type Output = T;
///     fn describe(&self) -> Self::Output { self.clone() }
/// }
///
/// // Multiple generics
/// #[specialize(T: Clone, U: Copy)]
/// impl<T, U> Pair<T, U> for (T, U) { ... }
///
/// // Most specific - concrete type
/// #[specialize]
/// impl MyTrait for String {
///     fn describe(&self) -> Self::Output { self.clone() }
/// }
/// ```
#[proc_macro_attribute]
pub fn specialize(attr: TokenStream, item: TokenStream) -> TokenStream {
    user::specialize::expand_specialize_attr(attr, item)
}

/// Nightly-like specialization block syntax on Stable Rust.
///
/// # Features
/// - `default fn` / `default type` - fine-grained control over what can be specialized
/// - Associated type specialization
/// - Multi-level specialization chains (A < B < C < ...)
/// - Custom trait-to-capability mapping via `#[map(MyTrait => IsMyTrait)]`
/// - Overlap detection with helpful error messages
///
/// # Usage
/// ```ignore
/// specialization! {
///     trait MyTrait {
///         type Output;
///         fn method(&self) -> Self::Output;
///     }
///
///     impl<T> MyTrait for T {
///         default type Output = ();
///         default fn method(&self) -> Self::Output { () }
///     }
///
///     impl<T: Clone> MyTrait for T {
///         type Output = T;
///         fn method(&self) -> Self::Output { self.clone() }
///     }
///
///     // Most specific
///     impl MyTrait for String {
///         fn method(&self) -> Self::Output { self.clone() }
///     }
/// }
/// ```
#[proc_macro]
pub fn specialization(input: TokenStream) -> TokenStream {
    user::specialize::expand_specialize_macro(input)
}

/// Nightly-like specialization for inherent impls.
///
/// # Usage
/// ```ignore
/// specialization_inherent! {
///     impl<T> MyStruct<T> {
///         default fn do_something(&self) { /* fallback */ }
///     }
///
///     impl<T: Clone> MyStruct<T> {
///         fn do_something(&self) { /* when T: Clone */ }
///     }
/// }
/// ```
#[proc_macro]
pub fn specialization_inherent(input: TokenStream) -> TokenStream {
    user::specialize::expand_specialize_inherent(input)
}

// Keep old names as aliases for backwards compatibility
/// Alias for `specialization!` (backwards compatibility)
#[proc_macro]
pub fn specialize_block(input: TokenStream) -> TokenStream {
    user::specialize::expand_specialize_macro(input)
}

/// Alias for `specialization_inherent!` (backwards compatibility)
#[proc_macro]
pub fn specialize_inherent(input: TokenStream) -> TokenStream {
    user::specialize::expand_specialize_inherent(input)
}

/// Legacy alias - use `#[specialize]` instead
#[proc_macro_attribute]
pub fn specialization_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    user::specialize::expand_specialize_attr(attr, item)
}

/// Check if types satisfy trait constraints with boolean expression support.
///
/// Returns `true` or `false` at runtime based on compile-time trait detection.
///
/// # Syntax: `caps_check!(Type: Expr, ...)`
///
/// Supports multiple checks in one call. All checks must pass for result to be true.
///
/// ```ignore
/// use std::fmt::Debug;
/// use tola_caps::caps_check;
///
/// // Single check
/// assert!(caps_check!(String: Clone));
/// assert!(!caps_check!(String: Copy));
///
/// // Boolean expressions
/// assert!(caps_check!(i32: Clone & Copy));
/// assert!(caps_check!(String: Clone | Copy));
/// assert!(caps_check!(String: Clone & !Copy));
/// assert!(caps_check!(i32: (Clone | Copy) & Debug));
///
/// // Multiple checks (all must pass)
/// assert!(caps_check!(String: Clone, i32: Copy));
/// assert!(caps_check!(String: Clone & !Copy, i32: Clone & Copy));
///
/// // Mix concrete and generic types
/// fn check<T: AutoCaps>() -> bool {
///     caps_check!(T: Clone, String: Debug)
/// }
///
/// // Custom traits work on concrete types
/// trait MyTrait {}
/// impl MyTrait for String {}
/// assert!(caps_check!(String: MyTrait));
/// ```
#[proc_macro]
pub fn caps_check(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CapsCheckInput);
    expand_caps_check(input).into()
}

/// Define a type capability marker.
///
/// # Usage
/// ```ignore
/// define_type_cap!(String);  // Generates TypeIsString marker struct
/// define_type_cap!(Vec);     // Generates TypeIsVec marker struct
///
/// // Use for type-based specialization
/// impl<T: TypeIsString> MyTrait for T { ... }
/// ```
#[proc_macro]
pub fn define_type_cap(input: TokenStream) -> TokenStream {
    user::define_type_cap(input)
}

/// Generate capability marker and detection for a custom trait.
///
/// For `derive_trait_cap!(MySerializable)` generates:
/// - `IsMySerializable` - capability marker
/// - `MySerializableFallback` - fallback trait
/// - Inherent const `IS_MY_SERIALIZABLE` for implementing types
///
/// # Example
/// ```ignore
/// trait MySerializable { fn serialize(&self) -> Vec<u8>; }
/// derive_trait_cap!(MySerializable);
///
/// // Use in specialize! - auto-detected
/// specialize! {
///     impl<T: MySerializable> Format for T { /* ... */ }
/// }
/// ```
#[proc_macro]
pub fn derive_trait_cap(input: TokenStream) -> TokenStream {
    user::derive_trait_cap(input)
}

/// Attribute macro to automatically implement AutoCaps and AutoCapSet for a type.
///
/// # Usage
/// ```ignore
/// #[auto_caps]
/// struct MyType {
///     data: String,
/// }
///
/// // Now you can use caps_check! on MyType
/// assert!(caps_check!(MyType, Clone));
/// ```
#[proc_macro_attribute]
pub fn auto_caps(attr: TokenStream, item: TokenStream) -> TokenStream {
    user::expand_auto_caps(attr, item)
}

/// Unified capability attribute macro.
///
/// # On Trait: Register trait for caps system
/// ```ignore
/// #[cap]
/// trait Serializable {
///     fn serialize(&self) -> Vec<u8>;
/// }
/// // Generates: IsSerializable marker, is_serializable::<T>() function
/// // Now usable in specialize! without #[map]
/// ```
///
/// # On Struct/Enum: Auto-detect standard traits
/// ```ignore
/// #[cap]
/// struct MyType { data: String }
/// // Enables trait detection via caps system
/// ```
///
/// **DEPRECATED**: Use `#[derive(AutoCaps)]` for types and `#[trait_autocaps]` for traits.
#[deprecated(note = "Use `#[derive(AutoCaps)]` for types and `#[trait_autocaps]` for traits instead")]
#[proc_macro_attribute]
pub fn cap(attr: TokenStream, item: TokenStream) -> TokenStream {
    user::expand_cap_attr(attr, item)
}

// =============================================================================
// caps_check! Implementation (Unified)
// =============================================================================

/// Single type check: `Type: Expr`
struct TypeCheck {
    ty: syn::Type,
    expr: common::BoolExpr,
}

impl syn::parse::Parse for TypeCheck {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: syn::Type = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let expr: common::BoolExpr = input.parse()?;
        Ok(TypeCheck { ty, expr })
    }
}

/// Input for caps_check! macro: one or more type checks
struct CapsCheckInput {
    checks: Vec<TypeCheck>,
}

impl syn::parse::Parse for CapsCheckInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut checks = Vec::new();

        // Parse first check (required)
        checks.push(input.parse()?);

        // Parse additional checks separated by commas
        while input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            if input.is_empty() {
                break;
            }
            checks.push(input.parse()?);
        }

        Ok(CapsCheckInput { checks })
    }
}

fn expand_caps_check(input: CapsCheckInput) -> proc_macro2::TokenStream {
    if input.checks.len() == 1 {
        let check = &input.checks[0];
        let ty = &check.ty;
        let inner = common::generate_unified_check(&check.expr, ty);

        // Reference user's type before internal imports to avoid unused import warnings
        quote::quote! {
            {
                fn __use_type<__T>(_: ::core::marker::PhantomData<__T>) {}
                __use_type::<#ty>(::core::marker::PhantomData);
                #inner
            }
        }
    } else {
        // Multiple checks - AND them together
        let type_refs: Vec<_> = input.checks.iter()
            .map(|c| {
                let ty = &c.ty;
                quote::quote! { __use_type::<#ty>(::core::marker::PhantomData); }
            })
            .collect();
        let check_exprs: Vec<_> = input.checks.iter()
            .map(|c| common::generate_unified_check(&c.expr, &c.ty))
            .collect();

        quote::quote! {
            {
                fn __use_type<__T>(_: ::core::marker::PhantomData<__T>) {}
                #(#type_refs)*
                (#(#check_exprs)&&*)
            }
        }
    }
}

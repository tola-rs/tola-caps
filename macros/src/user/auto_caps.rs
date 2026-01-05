//! Auto-detection macros for types and traits
//!
//! ## Unified Macro Names (all use `#[cap]` prefix)
//!
//! | Macro | Usage | Purpose |
//! |-------|-------|---------|
//! | `#[cap]` | on trait | Enable trait for caps system |
//! | `#[cap]` | on struct/enum | Auto-detect std traits |
//! | `#[derive(Capability)]` | on struct | Define capability marker |

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Ident, ItemTrait};

// =============================================================================
// #[cap] Unified Attribute Macro
// =============================================================================

/// Unified capability attribute macro.
///
/// # On Trait: Enable trait for caps system
/// ```ignore
/// #[cap]
/// trait MySerializable {
///     fn serialize(&self) -> Vec<u8>;
/// }
/// // Generates: IsMySerializable, is_my_serializable::<T>(), etc.
/// ```
///
/// # On Struct/Enum: Auto-detect standard traits
/// ```ignore
/// #[cap]
/// struct MyType { data: String }
/// // Enables: caps_check!(MyType: Clone), etc.
/// ```
pub fn expand_cap_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try parsing as trait first
    let item_clone = item.clone();
    if let Ok(trait_item) = syn::parse::<ItemTrait>(item_clone) {
        return expand_cap_on_trait(trait_item).into();
    }

    // Otherwise, parse as struct/enum (auto_caps behavior)
    let input = parse_macro_input!(item as DeriveInput);
    expand_cap_on_type(input).into()
}

/// Expand #[cap] on a type definition (struct/enum)
fn expand_cap_on_type(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    crate::inner::std_traits::expand_cap_on_type_impl(name, &input.generics)
}

/// Expand #[cap] on a trait definition
fn expand_cap_on_trait(trait_item: ItemTrait) -> proc_macro2::TokenStream {
    let trait_name = &trait_item.ident;
    let vis = &trait_item.vis;
    let generics = &trait_item.generics;

    // Generate names
    let cap_marker = format_ident!("Is{}", trait_name);
    let fallback_trait = format_ident!("{}Fallback", trait_name);
    let const_name = format_ident!("IS_{}", to_screaming_snake_case(&trait_name.to_string()));
    let detect_wrapper = format_ident!("__Detect_{}", trait_name);

    let cap_doc = format!("Capability marker for `{}` trait.", trait_name);
    let fallback_doc = format!("Fallback trait for `{}` detection.", trait_name);

    // Split generics for use in different contexts
    let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    // PERFECT SOLUTION: Make wrapper type carry ALL trait generic parameters!
    // For trait Converter<T>:
    //   struct __Detect_Converter<__TolaCapsDetectType_, T>(PhantomData<...>);
    // This way T is constrained by being part of the wrapper type itself.

    // Build wrapper generics: <__TolaCapsDetectType_, trait_generics...>
    let mut wrapper_params = syn::punctuated::Punctuated::new();
    let detect_param: syn::GenericParam = syn::parse_quote! { __TolaCapsDetectType_ };
    wrapper_params.push(detect_param);
    for param in generics.params.iter() {
        wrapper_params.push(param.clone());
    }

    let wrapper_generics = syn::Generics {
        lt_token: Some(Default::default()),
        params: wrapper_params.clone(),
        gt_token: Some(Default::default()),
        where_clause: None,
    };
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    // Build inherent impl generics with trait bound
    let mut inherent_params = wrapper_params.clone();
    // Find and add trait bound to __TolaCapsDetectType_
    if let Some(syn::GenericParam::Type(type_param)) = inherent_params.first_mut() {
        let trait_bound: syn::TypeParamBound = syn::parse_quote! { #trait_name #ty_generics };
        type_param.bounds.push(trait_bound);
    }

    let inherent_generics = syn::Generics {
        lt_token: Some(Default::default()),
        params: inherent_params,
        gt_token: Some(Default::default()),
        where_clause: generics.where_clause.clone(),
    };
    let (inherent_impl_generics, _, inherent_where_clause) = inherent_generics.split_for_impl();

    // Build PhantomData type for the wrapper struct
    // We need to use all generic parameters in PhantomData to avoid unused parameter errors
    let mut phantom_types = vec![quote! { __TolaCapsDetectType_ }];
    for param in generics.params.iter() {
        match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                phantom_types.push(quote! { #ident });
            }
            syn::GenericParam::Lifetime(lifetime_param) => {
                let lifetime = &lifetime_param.lifetime;
                phantom_types.push(quote! { &#lifetime () });
            }
            syn::GenericParam::Const(_const_param) => {
                // Const generics can't be used in types directly
            }
        }
    }

    quote! {
        // Original trait definition
        #trait_item

        // Capability marker
        #[doc = #cap_doc]
        #[derive(::tola_caps::Capability)]
        #vis struct #cap_marker;

        // Wrapper type carries ALL generic parameters (detect type + trait generics)
        #[doc(hidden)]
        #vis struct #detect_wrapper #wrapper_impl_generics (
            core::marker::PhantomData<(#(#phantom_types),*)>
        );

        // Fallback trait (no generics, returns false by default)
        #[doc = #fallback_doc]
        #[doc(hidden)]
        #vis trait #fallback_trait {
            const #const_name: bool = false;
        }

        // Implement fallback for all wrapper instances
        impl #wrapper_impl_generics #fallback_trait for #detect_wrapper #wrapper_ty_generics {}

        // Inherent impl for types that implement the trait
        // All generics are now properly constrained!
        impl #inherent_impl_generics #detect_wrapper #wrapper_ty_generics #inherent_where_clause {
            #vis const #const_name: bool = true;
        }

        // Use: __Detect_TraitName::<Type, GenericArgs...>::CONST_NAME
    }
}

/// Expand #[derive(AutoCaps)] on a struct/enum.
///
/// This is the recommended way to enable std trait detection for user types.
///
/// # Usage
/// ```ignore
/// #[derive(Clone, Debug, AutoCaps)]
/// struct MyType { data: String }
/// // Enables: caps_check!(MyType: Clone), etc.
/// ```
pub fn expand_derive_autocaps(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    crate::inner::std_traits::expand_cap_on_type_impl(name, &input.generics)
}

/// Expand #[trait_autocaps] on a trait definition.
///
/// This enables the trait for caps system detection.
///
/// # Usage
/// ```ignore
/// #[trait_autocaps]
/// trait MySerializable {
///     fn serialize(&self) -> Vec<u8>;
/// }
/// // Generates: IsMySerializable marker, caps_check!(T: MySerializable)
/// ```
pub fn expand_trait_autocaps(item: TokenStream) -> TokenStream {
    let trait_item = syn::parse::<ItemTrait>(item).expect("expected a trait definition");
    expand_cap_on_trait(trait_item).into()
}

// =============================================================================
// define_type_cap! Function Macro
// =============================================================================

/// Define a capability marker for a type.
///
/// Usage: `define_type_cap!(String);` generates `TypeIsString` marker struct.
pub fn define_type_cap(input: TokenStream) -> TokenStream {
    let ident = parse_macro_input!(input as Ident);
    let marker = format_ident!("TypeIs{}", ident);
    let doc = format!("Capability marker for type {}", ident);
    let expanded = quote! {
        #[doc = #doc]
        #[derive(::tola_caps::Capability)]
        pub struct #marker;
    };
    expanded.into()
}

// =============================================================================
// derive_trait_cap! Function Macro
// =============================================================================

/// Input for derive_trait_cap! macro
pub struct DeriveTraitCapInput {
    pub bounds: syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    pub alias: Option<syn::Ident>,
}

impl syn::parse::Parse for DeriveTraitCapInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse leading bounds: Bound + Bound + ...
        // We stop if we see `as`.
        let mut bounds = syn::punctuated::Punctuated::new();

        loop {
            // Check if we hit "as" (Alias keyword)
            if input.peek(syn::Token![as]) {
                break;
            }
            if input.is_empty() {
                break;
            }

            bounds.push(input.parse()?);

            // If next is +, consume it and continue
            if input.peek(syn::Token![+]) {
                 input.parse::<syn::Token![+]>()?;
            } else {
                 break;
            }
        }

        let alias = if input.peek(syn::Token![as]) {
             input.parse::<syn::Token![as]>()?;
             Some(input.parse::<syn::Ident>()?)
        } else {
             None
        };

        if bounds.is_empty() {
             return Err(input.error("Expected at least one trait bound"));
        }

        Ok(DeriveTraitCapInput {
            bounds,
            alias,
        })
    }
}

/// Generate capability marker and autoref detection for a custom trait.
///
/// This macro does everything needed to use a custom trait in `specialize!`:
///
/// 1. Creates `IsMyTrait` capability marker
/// 2. Creates `MyTraitFallback` trait with `IS_MY_TRAIT = false`
/// 3. Creates inherent impl with `IS_MY_TRAIT = true` for types implementing the trait
/// 4. Registers the mapping so `specialize!` auto-detects it
///
/// # Usage
///
/// ```ignore
/// // Define your trait
/// trait MySerializable {
///     fn serialize(&self) -> Vec<u8>;
/// }
///
/// // One line to enable specialization support!
/// derive_trait_cap!(MySerializable);
///
/// // Support for complex traits with alias:
/// derive_trait_cap!(Fn(u8) -> bool as FnU8);
///
/// // Now use it in specialize!
/// specialize! {
///     impl<T: MySerializable> Format for T { ... }
/// }
/// ```
pub fn derive_trait_cap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveTraitCapInput);
    expand_derive_trait_cap(input).into()
}

fn expand_derive_trait_cap(input: DeriveTraitCapInput) -> proc_macro2::TokenStream {
    // Reconstruct TraitModel to reuse naming logic (to_screaming_snake_case is also needed)
    // Since TraitModel is in common, we can reuse it if we had a constructor from bounds.
    // Instead of constructing full TraitModel, we reuse the naming logic here or partially implement it.
    // We already have `bounds` and `alias`.

    let alias = &input.alias;
    let bounds = &input.bounds;

    // Logic to determine name: Alias > First Trait Bound > Error
    let trait_ident = if let Some(a) = alias {
        a.clone()
    } else {
        // Find first TraitBound
        let mut found = None;
        for b in bounds {
            if let syn::TypeParamBound::Trait(tb) = b {
                if let Some(seg) = tb.path.segments.last() {
                    found = Some(seg.ident.clone());
                    break;
                }
            }
        }
        found.expect("derive_trait_cap: Cannot infer name. Consider using `as Alias`.")
    };

    let trait_name_str = trait_ident.to_string();

    let cap_marker = format_ident!("Is{}", trait_ident);
    let fallback_trait = format_ident!("{}Fallback", trait_ident);
    let const_name = format_ident!("IS_{}", to_screaming_snake_case(&trait_name_str));

    // Generate Alias Trait if alias is provided
    let alias_def = if alias.is_some() {
        quote! {
            #[allow(non_upper_case_globals)]
            pub trait #trait_ident: #bounds {}
            impl<T: ?Sized + #bounds> #trait_ident for T {}
        }
    } else {
        quote! {}
    };

    let cap_doc = format!("Capability marker for `{}` trait.", trait_name_str);
    let fallback_doc = format!("Fallback trait for `{}` detection.", trait_name_str);

    // Unique wrapper name to avoid collision
    let wrapper = format_ident!("__Detect_{}", trait_ident);

    // Use LOCAL wrapper to avoid cross-module collision.
    // Each call site gets its own wrapper, so different modules
    // defining the same trait name won't conflict.
    quote! {
        #alias_def

        #[doc = #cap_doc]
        #[derive(::tola_caps::Capability)]
        pub struct #cap_marker;

        // Local wrapper - avoids orphan rules and cross-module collision
        #[doc(hidden)]
        pub struct #wrapper<T: ?Sized>(core::marker::PhantomData<T>);

        #[doc = #fallback_doc]
        #[doc(hidden)]
        pub trait #fallback_trait {
            const #const_name: bool = false;
        }

        impl<T: ?Sized> #fallback_trait for #wrapper<T> {}

        impl<T: ?Sized + #bounds> #wrapper<T> {
            pub const #const_name: bool = true;
        }
    }
}

/// Convert CamelCase to SCREAMING_SNAKE_CASE
fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}


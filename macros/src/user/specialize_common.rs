//! Common utilities shared between specialize! function macro and #[specialize] attribute macro
//!
//! This module extracts the shared logic to improve maintainability and avoid code duplication.

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Generics, Ident, Path, TraitBound, Type};

// =============================================================================
// Trait-to-Capability Mapping
// =============================================================================

/// Built-in trait to capability mappings
///
/// This is the central registry for mapping standard Rust traits to their
/// corresponding tola-caps capability markers.
pub fn builtin_trait_map() -> HashMap<String, TokenStream2> {
    let mut map = HashMap::new();
    map.insert("Clone".to_string(), quote! { ::tola_caps::detect::IsClone });
    map.insert("Copy".to_string(), quote! { ::tola_caps::detect::IsCopy });
    map.insert("Debug".to_string(), quote! { ::tola_caps::detect::IsDebug });
    map.insert("Default".to_string(), quote! { ::tola_caps::detect::IsDefault });
    map.insert("Send".to_string(), quote! { ::tola_caps::detect::IsSend });
    map.insert("Sync".to_string(), quote! { ::tola_caps::detect::IsSync });
    map.insert("Eq".to_string(), quote! { ::tola_caps::detect::IsEq });
    map.insert("Ord".to_string(), quote! { ::tola_caps::detect::IsOrd });
    map.insert("Hash".to_string(), quote! { ::tola_caps::detect::IsHash });
    map.insert("Display".to_string(), quote! { ::tola_caps::detect::IsDisplay });
    map
}

/// Get the simple (last segment) name from a path
pub fn get_simple_trait_name(path: &Path) -> String {
    let full_path = path_to_string(path);
    full_path
        .split("::")
        .last()
        .unwrap_or(&full_path)
        .to_string()
}

/// Map a trait name to its capability marker
pub fn trait_to_capability(
    trait_name: &str,
    trait_map: &HashMap<String, TokenStream2>,
) -> Option<TokenStream2> {
    trait_map.get(trait_name).cloned()
}

/// Map a trait bound to its capability marker
pub fn bound_to_capability(
    bound: &TraitBound,
    trait_map: &HashMap<String, TokenStream2>,
) -> Option<TokenStream2> {
    let simple_name = get_simple_trait_name(&bound.path);
    trait_to_capability(&simple_name, trait_map)
}

/// Generate a default capability marker for unknown traits
/// Convention: Trait "Foo" maps to "IsFoo"
pub fn default_capability_for_trait(trait_name: &str) -> TokenStream2 {
    let marker = format_ident!("Is{}", trait_name);
    quote! { #marker }
}

/// Map a trait bound to capability, with fallback to default naming convention
pub fn bound_to_capability_with_fallback(
    bound: &TraitBound,
    trait_map: &HashMap<String, TokenStream2>,
) -> TokenStream2 {
    let simple_name = get_simple_trait_name(&bound.path);
    trait_map
        .get(&simple_name)
        .cloned()
        .unwrap_or_else(|| default_capability_for_trait(&simple_name))
}

// =============================================================================
// Specificity Computation
// =============================================================================

/// Compute specificity score for an impl block.
/// Lower score = more specific.
///
/// Rules based on RFC 1210:
/// - Concrete type: 0 (most specific)
/// - Type with many bounds: 100 - bound_count (more bounds = more specific)
/// - Generic with no bounds: 1000 (least specific, default fallback)
pub fn compute_specificity(generics: &Generics, bounds: &[TraitBound], _self_ty: &Type) -> u32 {
    let is_generic = !generics.params.is_empty();
    let bound_count = bounds.len() as u32;

    if !is_generic {
        // Concrete type like `impl Trait for String`
        0
    } else if bound_count > 0 {
        // Generic with bounds: more bounds = more specific
        100 - bound_count.min(99)
    } else {
        // Generic with no bounds: `impl<T> Trait for T`
        1000
    }
}

/// Represents the specificity level for quick comparison.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecificityLevel {
    /// Concrete type (most specific)
    Concrete,
    /// Generic with bounds
    BoundedGeneric { bound_count: u32 },
    /// Generic with no bounds (least specific)
    UnboundedGeneric,
}

#[allow(dead_code)]
impl SpecificityLevel {
    pub fn from_impl(generics: &Generics, bounds: &[TraitBound]) -> Self {
        let is_generic = !generics.params.is_empty();
        let bound_count = bounds.len() as u32;

        if !is_generic {
            SpecificityLevel::Concrete
        } else if bound_count > 0 {
            SpecificityLevel::BoundedGeneric { bound_count }
        } else {
            SpecificityLevel::UnboundedGeneric
        }
    }

    /// Convert to numeric specificity (lower = more specific)
    pub fn to_score(&self) -> u32 {
        match self {
            SpecificityLevel::Concrete => 0,
            SpecificityLevel::BoundedGeneric { bound_count } => 100 - (*bound_count).min(99),
            SpecificityLevel::UnboundedGeneric => 1000,
        }
    }
}

// =============================================================================
// Capability Expression Building
// =============================================================================

/// Build an AND expression from multiple capabilities
pub fn build_and_expression(capabilities: Vec<TokenStream2>) -> TokenStream2 {
    if capabilities.is_empty() {
        return quote! { ::tola_caps::Present };
    }
    if capabilities.len() == 1 {
        return capabilities.into_iter().next().unwrap();
    }

    let mut iter = capabilities.into_iter();
    let first = iter.next().unwrap();
    iter.fold(first, |acc, c| {
        quote! { ::tola_caps::capability::And<#acc, #c> }
    })
}

/// Build an OR expression from multiple capabilities.
#[allow(dead_code)]
pub fn build_or_expression(capabilities: Vec<TokenStream2>) -> TokenStream2 {
    if capabilities.is_empty() {
        return quote! { ::tola_caps::Absent };
    }
    if capabilities.len() == 1 {
        return capabilities.into_iter().next().unwrap();
    }

    let mut iter = capabilities.into_iter();
    let first = iter.next().unwrap();
    iter.fold(first, |acc, c| {
        quote! { ::tola_caps::capability::Or<#acc, #c> }
    })
}

/// Build capability conditions from trait bounds.
#[allow(dead_code)]
pub fn build_capability_conditions(
    bounds: &[TraitBound],
    trait_map: &HashMap<String, TokenStream2>,
) -> Vec<TokenStream2> {
    bounds
        .iter()
        .filter_map(|bound| bound_to_capability(bound, trait_map))
        .collect()
}

// =============================================================================
// Where Clause Bounds Generation
// =============================================================================

/// Generate AutoCapSet bound for a type parameter.
#[allow(dead_code)]
pub fn auto_cap_set_bound(ty: &TokenStream2) -> TokenStream2 {
    quote! { #ty: ::tola_caps::std_caps::AutoCapSet }
}

/// Generate capability evaluation bound for a type.
#[allow(dead_code)]
pub fn capability_eval_bound(ty: &TokenStream2, cap: &TokenStream2) -> TokenStream2 {
    quote! {
        ::tola_caps::std_caps::Cap<#ty>: ::tola_caps::capability::Evaluate<#cap, Out = ::tola_caps::Present>
    }
}

/// Generate standard capability bounds for dispatching
pub fn standard_capability_bounds() -> TokenStream2 {
    quote! {
        ::tola_caps::std_caps::Cap<T>: ::tola_caps::capability::Evaluate<::tola_caps::detect::IsClone>
            + ::tola_caps::capability::Evaluate<::tola_caps::detect::IsCopy>
            + ::tola_caps::capability::Evaluate<::tola_caps::detect::IsDebug>
            + ::tola_caps::capability::Evaluate<::tola_caps::detect::IsDefault>
            + ::tola_caps::capability::Evaluate<::tola_caps::detect::IsSend>
            + ::tola_caps::capability::Evaluate<::tola_caps::detect::IsSync>
    }
}

// =============================================================================
// Type and Path Utilities
// =============================================================================

/// Convert a syn::Path to a normalized string (no whitespace)
pub fn path_to_string(p: &Path) -> String {
    use quote::ToTokens;
    p.to_token_stream().to_string().replace(' ', "")
}

/// Convert a syn::Type to a normalized string (no whitespace)
pub fn type_to_string(t: &Type) -> String {
    use quote::ToTokens;
    t.to_token_stream().to_string().replace(' ', "")
}

/// Extract the last identifier from a type (for marker trait naming)
pub fn extract_type_ident(ty: &Type) -> Option<Ident> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.clone()),
        _ => None,
    }
}

/// Generate a marker trait name for a concrete type
pub fn marker_trait_name(ty: &Type) -> Ident {
    extract_type_ident(ty)
        .map(|ident| format_ident!("__TypeIs{}", ident))
        .unwrap_or_else(|| format_ident!("__TypeIsSpecial"))
}

// =============================================================================
// Bound Extraction
// =============================================================================

/// Extract all trait bounds from generics (both type params and where clause)
pub fn extract_all_bounds(generics: &Generics) -> Vec<TraitBound> {
    let mut bounds = Vec::new();

    // Extract bounds from type parameters
    for param in &generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            for bound in &type_param.bounds {
                if let syn::TypeParamBound::Trait(tb) = bound {
                    bounds.push(tb.clone());
                }
            }
        }
    }

    // Extract bounds from where clause
    if let Some(ref wc) = generics.where_clause {
        for pred in &wc.predicates {
            if let syn::WherePredicate::Type(type_pred) = pred {
                for bound in &type_pred.bounds {
                    if let syn::TypeParamBound::Trait(tb) = bound {
                        bounds.push(tb.clone());
                    }
                }
            }
        }
    }

    bounds
}

// =============================================================================
// Selection Chain Building (shared by function macro code generation)
// =============================================================================

/// Build a conditional type selection using SelectCap.
#[allow(dead_code)]
pub fn build_select_cap_chain(
    base: TokenStream2,
    condition: TokenStream2,
    if_true: TokenStream2,
    if_false: TokenStream2,
) -> TokenStream2 {
    quote! {
        <::tola_caps::std_caps::Cap<#base> as ::tola_caps::spec::dispatch::SelectCap<
            #condition,
            #if_true,
            #if_false
        >>::Out
    }
}

/// Build a conditional type selection using SelectType.
#[allow(dead_code)]
pub fn build_select_type_chain(
    base: TokenStream2,
    condition: TokenStream2,
    if_true: TokenStream2,
    if_false: TokenStream2,
) -> TokenStream2 {
    quote! {
        <::tola_caps::std_caps::Cap<#base> as ::tola_caps::spec::dispatch::SelectType<
            #condition,
            #if_true,
            #if_false
        >>::Out
    }
}

/// Build a StaticSelect wrapper for static method dispatch.
#[allow(dead_code)]
pub fn build_static_select(
    cap_base: TokenStream2,
    condition: TokenStream2,
    if_true: TokenStream2,
    if_false: TokenStream2,
) -> TokenStream2 {
    quote! {
        ::tola_caps::spec::dispatch::StaticSelect<
            #cap_base,
            #condition,
            #if_true,
            #if_false
        >
    }
}

// =============================================================================
// Code Generation Helpers
// =============================================================================

/// Generate impl struct name for method specialization
pub fn impl_struct_name(impl_idx: usize, method_name: &Ident) -> Ident {
    format_ident!("__SpecImpl{}_{}", impl_idx, method_name)
}

/// Generate type struct name for type specialization
pub fn type_struct_name(impl_idx: usize, type_name: &Ident) -> Ident {
    format_ident!("__SpecType{}_{}", impl_idx, type_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity_ordering() {
        // Concrete < BoundedGeneric < UnboundedGeneric
        assert!(SpecificityLevel::Concrete < SpecificityLevel::BoundedGeneric { bound_count: 5 });
        assert!(
            SpecificityLevel::BoundedGeneric { bound_count: 5 }
                < SpecificityLevel::BoundedGeneric { bound_count: 1 }
        );
        assert!(
            SpecificityLevel::BoundedGeneric { bound_count: 1 } < SpecificityLevel::UnboundedGeneric
        );
    }

    #[test]
    fn test_builtin_trait_map() {
        let map = builtin_trait_map();
        assert!(map.contains_key("Clone"));
        assert!(map.contains_key("Copy"));
        assert!(map.contains_key("Debug"));
        assert!(map.contains_key("Default"));
        assert!(map.contains_key("Send"));
        assert!(map.contains_key("Sync"));
    }
}

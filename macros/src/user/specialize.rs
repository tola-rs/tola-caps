//! Full-featured Specialization Macros for Stable Rust
//!
//! This module provides two equivalent ways to use specialization:
//!
//! ## 1. Attribute Macro: `#[specialize]`
//!
//! ```ignore
//! trait MyTrait {
//!     type Output;
//!     fn method(&self) -> Self::Output;
//! }
//!
//! // Default implementation (most general)
//! #[specialize]
//! impl<T> MyTrait for T {
//!     #[specialize(default)]
//!     type Output = ();
//!
//!     #[specialize(default)]
//!     fn method(&self) -> Self::Output { () }
//! }
//!
//! // More specific implementation
//! #[specialize]
//! impl<T: Clone> MyTrait for T {
//!     type Output = T;  // overrides default
//!     fn method(&self) -> Self::Output { self.clone() }
//! }
//!
//! // Most specific implementation
//! #[specialize]
//! impl MyTrait for String {
//!     fn method(&self) -> Self::Output { self.clone() }
//! }
//! ```
//!
//! ## 2. Function Macro: `specialize!`
//!
//! ```ignore
//! specialize! {
//!     trait MyTrait {
//!         type Output;
//!         fn method(&self) -> Self::Output;
//!     }
//!
//!     impl<T> MyTrait for T {
//!         default type Output = ();
//!         default fn method(&self) -> Self::Output { () }
//!     }
//!
//!     impl<T: Clone> MyTrait for T {
//!         type Output = T;
//!         fn method(&self) -> Self::Output { self.clone() }
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `default fn` / `default type` - fine-grained control over what can be specialized
//! - Associated type specialization
//! - Multi-level specialization chains (A < B < C < ...)
//! - Custom trait-to-capability mapping via `#[map(MyTrait => IsMyTrait)]`
//! - Overlap detection with helpful error messages
//! - Inherent impl specialization via `specialize_inherent!`

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, Expr, Generics, Ident, ItemImpl, Meta, Path, Token, TraitBound, Type,
    Visibility, WhereClause,
};
use std::collections::HashMap;

// Import shared utilities
use super::specialize_common::{
    builtin_trait_map, compute_specificity, path_to_string, type_to_string,
    extract_all_bounds, build_and_expression,
    impl_struct_name, type_struct_name, marker_trait_name,
    standard_capability_bounds, bound_to_capability, bound_to_capability_with_fallback,
};

// =============================================================================
// Part 1: AST Structures
// =============================================================================

/// Complete input for specialize! macro
pub struct SpecializeInput {
    /// Optional capability mappings: #[map(MyTrait => IsMyTrait)]
    pub mappings: Vec<CapabilityMapping>,
    /// The trait definition (optional for inherent impls)
    pub trait_def: Option<TraitDef>,
    /// All impl blocks
    pub impls: Vec<SpecImplBlock>,
}

/// Custom trait-to-capability mapping
#[derive(Clone)]
pub struct CapabilityMapping {
    pub trait_name: Path,
    pub capability: Path,
}

/// Trait definition within specialize!
pub struct TraitDef {
    pub vis: Visibility,
    pub name: Ident,
    pub generics: Generics,
    pub items: Vec<TraitItem>,
}

/// Items that can appear in a trait definition
pub enum TraitItem {
    Method(TraitMethodDef),
    Type(TraitTypeDef),
    Const(TraitConstDef),
}

/// Method definition in trait
pub struct TraitMethodDef {
    pub name: Ident,
    pub sig: MethodSignature,
    pub default_body: Option<TokenStream2>,
}

/// Associated type definition in trait
pub struct TraitTypeDef {
    pub name: Ident,
    pub bounds: Vec<TraitBound>,
    pub default: Option<Type>,
}

/// Associated const in trait
#[allow(dead_code)]
pub struct TraitConstDef {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

/// Method signature details
#[derive(Clone)]
#[allow(dead_code)]
pub struct MethodSignature {
    pub receiver: ReceiverKind,
    pub params: Vec<(Ident, Type)>,
    pub return_type: Option<Type>,
    pub where_clause: Option<WhereClause>,
}

/// Self receiver type
#[derive(Clone, Debug, PartialEq)]
pub enum ReceiverKind {
    None,
    SelfValue,
    SelfRef,
    SelfMutRef,
}

/// An impl block with specialization info
pub struct SpecImplBlock {
    /// The impl's generics
    pub generics: Generics,
    /// Trait bounds on type parameters
    pub bounds: Vec<TraitBound>,
    /// The trait being implemented (None for inherent impl)
    pub trait_path: Option<Path>,
    /// The Self type
    pub self_ty: Type,
    /// Items in this impl
    pub items: Vec<SpecImplItem>,
    /// Computed specificity (lower = more specific)
    pub specificity: u32,
}

/// Items that can appear in an impl block
pub enum SpecImplItem {
    Method(SpecMethodImpl),
    Type(SpecTypeImpl),
    #[allow(dead_code)]
    Const(SpecConstImpl),
}

/// Method implementation with default marker
pub struct SpecMethodImpl {
    pub is_default: bool,
    pub name: Ident,
    pub sig: MethodSignature,
    pub body: TokenStream2,
}

/// Associated type implementation with default marker
pub struct SpecTypeImpl {
    pub is_default: bool,
    pub name: Ident,
    pub ty: Type,
}

/// Associated const implementation
#[allow(dead_code)]
pub struct SpecConstImpl {
    pub is_default: bool,
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
}

// =============================================================================
// Part 2: Parsing Implementation
// =============================================================================

impl Parse for SpecializeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut mappings = Vec::new();
        let mut trait_def = None;
        let mut impls = Vec::new();

        // Parse optional #[map(...)] attributes at the top
        while input.peek(Token![#]) {
            let attrs = input.call(Attribute::parse_outer)?;
            for attr in attrs {
                if attr.path().is_ident("map") {
                    let mapping: CapabilityMapping = attr.parse_args()?;
                    mappings.push(mapping);
                }
            }
        }

        // Parse trait definition if present
        if input.peek(Token![pub]) || input.peek(Token![trait]) {
            trait_def = Some(input.parse()?);
        }

        // Parse impl blocks
        while !input.is_empty() {
            impls.push(input.parse()?);
        }

        Ok(SpecializeInput {
            mappings,
            trait_def,
            impls,
        })
    }
}

impl Parse for CapabilityMapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name: Path = input.parse()?;
        input.parse::<Token![=>]>()?;
        let capability: Path = input.parse()?;
        Ok(CapabilityMapping {
            trait_name,
            capability,
        })
    }
}

impl Parse for TraitDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis: Visibility = input.parse()?;
        input.parse::<Token![trait]>()?;
        let name: Ident = input.parse()?;
        let generics: Generics = input.parse()?;

        let content;
        braced!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }

        Ok(TraitDef {
            vis,
            name,
            generics,
            items,
        })
    }
}

impl Parse for TraitItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![fn]) {
            Ok(TraitItem::Method(input.parse()?))
        } else if lookahead.peek(Token![type]) {
            Ok(TraitItem::Type(input.parse()?))
        } else if lookahead.peek(Token![const]) {
            Ok(TraitItem::Const(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for TraitMethodDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let sig = parse_method_signature(&content)?;

        let return_type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        let where_clause = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let default_body = if input.peek(syn::token::Brace) {
            let body_content;
            braced!(body_content in input);
            Some(body_content.parse()?)
        } else {
            input.parse::<Token![;]>()?;
            None
        };

        Ok(TraitMethodDef {
            name,
            sig: MethodSignature {
                receiver: sig.receiver,
                params: sig.params,
                return_type,
                where_clause,
            },
            default_body,
        })
    }
}

impl Parse for TraitTypeDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![type]>()?;
        let name: Ident = input.parse()?;

        let mut bounds = Vec::new();
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            loop {
                if input.peek(Token![=]) || input.peek(Token![;]) {
                    break;
                }
                bounds.push(input.parse()?);
                if !input.peek(Token![+]) {
                    break;
                }
                input.parse::<Token![+]>()?;
            }
        }

        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![;]>()?;

        Ok(TraitTypeDef {
            name,
            bounds,
            default,
        })
    }
}

impl Parse for TraitConstDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![const]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;

        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<Token![;]>()?;

        Ok(TraitConstDef { name, ty, default })
    }
}

impl Parse for SpecImplBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![impl]>()?;

        let mut generics: Generics = input.parse()?;

        // Parse trait path and `for` keyword, or just self type for inherent impl
        let (trait_path, self_ty) = if input.peek2(Token![for]) {
            let path: Path = input.parse()?;
            input.parse::<Token![for]>()?;
            let ty: Type = input.parse()?;
            (Some(path), ty)
        } else {
            let ty: Type = input.parse()?;
            (None, ty)
        };

        // Parse where clause
        if input.peek(Token![where]) {
            generics.where_clause = Some(input.parse()?);
        }

        // Extract bounds using shared utility
        let bounds = extract_all_bounds(&generics);

        let content;
        braced!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }

        // Compute specificity using shared function
        let specificity = compute_specificity(&generics, &bounds, &self_ty);

        Ok(SpecImplBlock {
            generics,
            bounds,
            trait_path,
            self_ty,
            items,
            specificity,
        })
    }
}

impl Parse for SpecImplItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for `default` keyword
        let is_default = if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse().unwrap();
            if ident == "default" {
                input.parse::<Ident>()?; // consume "default"
                true
            } else {
                false
            }
        } else {
            false
        };

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![fn]) {
            let mut method: SpecMethodImpl = input.parse()?;
            method.is_default = is_default;
            Ok(SpecImplItem::Method(method))
        } else if lookahead.peek(Token![type]) {
            let mut ty: SpecTypeImpl = input.parse()?;
            ty.is_default = is_default;
            Ok(SpecImplItem::Type(ty))
        } else if lookahead.peek(Token![const]) {
            let mut c: SpecConstImpl = input.parse()?;
            c.is_default = is_default;
            Ok(SpecImplItem::Const(c))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for SpecMethodImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let sig = parse_method_signature(&content)?;

        let return_type = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        let where_clause = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let body_content;
        braced!(body_content in input);
        let body: TokenStream2 = body_content.parse()?;

        Ok(SpecMethodImpl {
            is_default: false,
            name,
            sig: MethodSignature {
                receiver: sig.receiver,
                params: sig.params,
                return_type,
                where_clause,
            },
            body,
        })
    }
}

impl Parse for SpecTypeImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![type]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(SpecTypeImpl {
            is_default: false,
            name,
            ty,
        })
    }
}

impl Parse for SpecConstImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![const]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(SpecConstImpl {
            is_default: false,
            name,
            ty,
            value,
        })
    }
}

/// Parse method signature from parenthesized content
fn parse_method_signature(input: ParseStream) -> syn::Result<MethodSignature> {
    let mut receiver = ReceiverKind::None;
    let mut params = Vec::new();

    if !input.is_empty() {
        // Check for self receiver
        if input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            if input.peek(Token![mut]) {
                input.parse::<Token![mut]>()?;
                input.parse::<Token![self]>()?;
                receiver = ReceiverKind::SelfMutRef;
            } else {
                input.parse::<Token![self]>()?;
                receiver = ReceiverKind::SelfRef;
            }
        } else if input.peek(Token![self]) {
            input.parse::<Token![self]>()?;
            receiver = ReceiverKind::SelfValue;
        }

        // Skip comma after self
        if receiver != ReceiverKind::None && input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        // Parse remaining parameters
        while !input.is_empty() {
            let param_name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;
            let param_ty: Type = input.parse()?;
            params.push((param_name, param_ty));

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
    }

    Ok(MethodSignature {
        receiver,
        params,
        return_type: None,
        where_clause: None,
    })
}

// =============================================================================
// Part 3: Specificity & Overlap Detection
// =============================================================================

// Note: compute_specificity is now in specialize_common module

/// Check for overlapping impls and report errors
pub fn check_overlaps(impls: &[SpecImplBlock]) -> syn::Result<()> {
    for i in 0..impls.len() {
        for j in (i + 1)..impls.len() {
            let impl_a = &impls[i];
            let impl_b = &impls[j];

            if impls_overlap(impl_a, impl_b) {
                // Check if one is strictly more specific
                if impl_a.specificity == impl_b.specificity {
                    return Err(syn::Error::new(
                        impl_b.self_ty.span(),
                        format!(
                            "Ambiguous specialization: impls have equal specificity.\n\
                             Neither impl is strictly more specific than the other.\n\
                             Add more trait bounds to one impl to disambiguate."
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Check if two impl blocks potentially overlap
fn impls_overlap(a: &SpecImplBlock, b: &SpecImplBlock) -> bool {
    // Different traits don't overlap
    match (&a.trait_path, &b.trait_path) {
        (Some(pa), Some(pb)) => {
            if path_to_string(pa) != path_to_string(pb) {
                return false;
            }
        }
        (None, Some(_)) | (Some(_), None) => return false,
        (None, None) => {
            // Both inherent impls - check self type
            if type_to_string(&a.self_ty) != type_to_string(&b.self_ty) {
                return false;
            }
        }
    }

    // Check if self types can unify
    types_can_unify(&a.self_ty, &b.self_ty, &a.generics, &b.generics)
}

/// Check if two types can potentially unify
fn types_can_unify(a: &Type, b: &Type, gen_a: &Generics, gen_b: &Generics) -> bool {
    let a_str = type_to_string(a);
    let b_str = type_to_string(b);

    // If either is a bare generic param, they can unify
    let a_is_generic = gen_a.params.iter().any(|p| {
        if let syn::GenericParam::Type(tp) = p {
            tp.ident.to_string() == a_str
        } else {
            false
        }
    });
    let b_is_generic = gen_b.params.iter().any(|p| {
        if let syn::GenericParam::Type(tp) = p {
            tp.ident.to_string() == b_str
        } else {
            false
        }
    });

    if a_is_generic || b_is_generic {
        true
    } else {
        // Both concrete: they unify only if equal
        a_str == b_str
    }
}

// =============================================================================
// Part 4: Code Generation
// =============================================================================

// Note: builtin_trait_map is now in specialize_common module

/// Main code generation function
pub fn expand_specialize(input: SpecializeInput) -> TokenStream2 {
    // Check for overlaps first
    if let Err(e) = check_overlaps(&input.impls) {
        return e.to_compile_error();
    }

    // Build trait-to-capability mapping using shared function
    let mut trait_map = builtin_trait_map();
    for mapping in &input.mappings {
        let key = path_to_string(&mapping.trait_name);
        let cap = &mapping.capability;
        trait_map.insert(key, quote! { #cap });
    }

    // Sort impls by specificity (most specific first)
    let mut sorted_impls = input.impls;
    sorted_impls.sort_by_key(|imp| imp.specificity);

    // Generate code
    let trait_def = input.trait_def.as_ref().map(|td| generate_trait_def(td));
    let impl_structs = generate_impl_structs(&sorted_impls, &trait_map);
    let dispatch_impl = generate_dispatch_impl(&sorted_impls, &trait_map, input.trait_def.as_ref());

    quote! {
        #trait_def
        #impl_structs
        #dispatch_impl
    }
}

fn generate_trait_def(td: &TraitDef) -> TokenStream2 {
    let vis = &td.vis;
    let name = &td.name;
    let generics = &td.generics;

    let items: Vec<_> = td.items.iter().map(|item| match item {
        TraitItem::Method(m) => {
            let method_name = &m.name;
            let receiver = match m.sig.receiver {
                ReceiverKind::None => quote! {},
                ReceiverKind::SelfValue => quote! { self },
                ReceiverKind::SelfRef => quote! { &self },
                ReceiverKind::SelfMutRef => quote! { &mut self },
            };
            let params: Vec<_> = m.sig.params.iter().map(|(n, t)| quote! { #n: #t }).collect();
            let ret = m.sig.return_type.as_ref().map(|r| quote! { -> #r }).unwrap_or_default();

            if let Some(body) = &m.default_body {
                quote! { fn #method_name(#receiver #(, #params)*) #ret { #body } }
            } else {
                quote! { fn #method_name(#receiver #(, #params)*) #ret; }
            }
        }
        TraitItem::Type(t) => {
            let type_name = &t.name;
            let bounds: Vec<_> = t.bounds.iter().map(|b| quote! { #b }).collect();
            let bounds_tokens = if bounds.is_empty() {
                quote! {}
            } else {
                quote! { : #(#bounds)+* }
            };
            if let Some(default) = &t.default {
                quote! { type #type_name #bounds_tokens = #default; }
            } else {
                quote! { type #type_name #bounds_tokens; }
            }
        }
        TraitItem::Const(c) => {
            let const_name = &c.name;
            let ty = &c.ty;
            if let Some(default) = &c.default {
                quote! { const #const_name: #ty = #default; }
            } else {
                quote! { const #const_name: #ty; }
            }
        }
    }).collect();

    quote! {
        #vis trait #name #generics {
            #(#items)*
        }
    }
}

fn generate_impl_structs(
    impls: &[SpecImplBlock],
    _trait_map: &HashMap<String, TokenStream2>,
) -> TokenStream2 {
    let structs: Vec<_> = impls.iter().enumerate().flat_map(|(idx, imp)| {
        imp.items.iter().filter_map(move |item| {
            match item {
                SpecImplItem::Method(m) => {
                    let struct_name = impl_struct_name(idx, &m.name);
                    let body = &m.body;
                    let ret = m.sig.return_type.as_ref()
                        .map(|r| quote! { #r })
                        .unwrap_or(quote! { () });

                    // Generate different impl based on whether method has receiver
                    if m.sig.receiver == ReceiverKind::None {
                        // Static method - use StaticMethodImpl
                        Some(quote! {
                            #[doc(hidden)]
                            #[allow(non_camel_case_types)]
                            pub struct #struct_name;

                            impl ::tola_caps::spec::dispatch::StaticMethodImpl<#ret> for #struct_name {
                                #[inline(always)]
                                fn call() -> #ret {
                                    #body
                                }
                            }
                        })
                    } else {
                        // Instance method - use MethodImpl
                        Some(quote! {
                            #[doc(hidden)]
                            #[allow(non_camel_case_types)]
                            pub struct #struct_name;

                            impl<T: ?Sized> ::tola_caps::spec::dispatch::MethodImpl<T, #ret> for #struct_name {
                                #[inline(always)]
                                fn call(_value: &T) -> #ret {
                                    #body
                                }
                            }
                        })
                    }
                }
                SpecImplItem::Type(t) => {
                    let struct_name = type_struct_name(idx, &t.name);
                    let ty = &t.ty;

                    Some(quote! {
                        #[doc(hidden)]
                        #[allow(non_camel_case_types)]
                        pub struct #struct_name;

                        impl ::tola_caps::spec::dispatch::TypeSelector for #struct_name {
                            type Out = #ty;
                        }
                    })
                }
                _ => None,
            }
        })
    }).collect();

    quote! { #(#structs)* }
}

fn generate_dispatch_impl(
    impls: &[SpecImplBlock],
    trait_map: &HashMap<String, TokenStream2>,
    trait_def: Option<&TraitDef>,
) -> TokenStream2 {
    let Some(td) = trait_def else {
        return quote! {};
    };

    let trait_name = &td.name;

    // Build selection chains for each method
    let method_chains: Vec<_> = td.items.iter().filter_map(|item| {
        if let TraitItem::Method(m) = item {
            let method_name = &m.name;
            let ret = m.sig.return_type.as_ref()
                .map(|r| quote! { #r })
                .unwrap_or(quote! { () });

            // Build chain from most general to most specific
            // We iterate in reverse since impls are sorted most-specific-first
            let mut selection = quote! { ::tola_caps::spec::dispatch::NoImpl };

            for (idx, imp) in impls.iter().enumerate().rev() {
                // Check if this impl has this method
                let has_method = imp.items.iter().any(|i| {
                    if let SpecImplItem::Method(m2) = i {
                        m2.name == *method_name
                    } else {
                        false
                    }
                });

                if has_method {
                    let impl_struct = impl_struct_name(idx, method_name);

                    // Build condition from bounds using shared utility
                    let conditions: Vec<_> = imp.bounds.iter()
                        .filter_map(|bound| bound_to_capability(bound, trait_map))
                        .collect();

                    if conditions.is_empty() && imp.specificity == 0 {
                        // Concrete type - unconditional selection
                        selection = quote! { #impl_struct };
                    } else if conditions.is_empty() {
                        // No conditions, this is the default
                        selection = quote! { #impl_struct };
                    } else {
                        // Build AND of all conditions using shared utility
                        let condition = build_and_expression(conditions);

                        let prev = selection;
                        selection = quote! {
                            <::tola_caps::std_caps::Cap<T> as ::tola_caps::spec::dispatch::SelectCap<
                                #condition,
                                #impl_struct,
                                #prev
                            >>::Out
                        };
                    }
                }
            }

            let receiver = match m.sig.receiver {
                ReceiverKind::None => quote! {},
                ReceiverKind::SelfValue => quote! { self },
                ReceiverKind::SelfRef => quote! { &self },
                ReceiverKind::SelfMutRef => quote! { &mut self },
            };
            let params: Vec<_> = m.sig.params.iter().map(|(n, t)| quote! { #n: #t }).collect();
            let _ret_decl = m.sig.return_type.as_ref().map(|r| quote! { -> #r }).unwrap_or_default();

            // Generate the call expression based on whether we have a receiver
            let call_expr = if m.sig.receiver == ReceiverKind::None {
                // For static methods, build a StaticSelect chain that implements StaticMethodImpl
                // This avoids the issue where Bool::If<Then, Else> can't be proven to impl StaticMethodImpl
                let mut static_selection = quote! { ::tola_caps::spec::dispatch::NoImpl };

                for (idx, imp) in impls.iter().enumerate().rev() {
                    let has_method = imp.items.iter().any(|i| {
                        if let SpecImplItem::Method(m2) = i {
                            m2.name == *method_name
                        } else {
                            false
                        }
                    });

                    if has_method {
                        let impl_struct = impl_struct_name(idx, method_name);

                        // Build conditions using shared utility
                        let conditions: Vec<_> = imp.bounds.iter()
                            .filter_map(|bound| bound_to_capability(bound, trait_map))
                            .collect();

                        if conditions.is_empty() {
                            // Unconditional - this is the default
                            static_selection = quote! { #impl_struct };
                        } else {
                            let condition = build_and_expression(conditions);

                            let prev = static_selection;
                            // Use StaticSelect wrapper which implements StaticMethodImpl
                            static_selection = quote! {
                                ::tola_caps::spec::dispatch::StaticSelect<
                                    ::tola_caps::std_caps::Cap<T>,
                                    #condition,
                                    #impl_struct,
                                    #prev
                                >
                            };
                        }
                    }
                }

                // Call via StaticMethodImpl
                quote! {
                    <#static_selection as ::tola_caps::spec::dispatch::StaticMethodImpl<#ret>>::call()
                }
            } else {
                // For methods with self, use the original selection approach
                quote! {
                    <#selection as ::tola_caps::spec::dispatch::MethodImpl<Self, #ret>>::call(&self)
                }
            };

            Some(quote! {
                #[inline(always)]
                fn #method_name(#receiver #(, #params)*) -> #ret {
                    #call_expr
                }
            })
        } else {
            None
        }
    }).collect();

    // Build selection chains for associated types
    let type_chains: Vec<_> = td.items.iter().filter_map(|item| {
        if let TraitItem::Type(t) = item {
            let type_name = &t.name;

            let mut selection = quote! { () };

            for (_idx, imp) in impls.iter().enumerate().rev() {
                let type_impl = imp.items.iter().find_map(|i| {
                    if let SpecImplItem::Type(t2) = i {
                        if t2.name == *type_name {
                            return Some(t2);
                        }
                    }
                    None
                });

                if let Some(ti) = type_impl {
                    let ty = &ti.ty;
                    // Build conditions using shared utility
                    let conditions: Vec<_> = imp.bounds.iter()
                        .filter_map(|bound| bound_to_capability(bound, trait_map))
                        .collect();

                    if conditions.is_empty() {
                        selection = quote! { #ty };
                    } else {
                        let condition = build_and_expression(conditions);

                        let prev = selection;
                        selection = quote! {
                            <::tola_caps::std_caps::Cap<T> as ::tola_caps::spec::dispatch::SelectType<
                                #condition,
                                #ty,
                                #prev
                            >>::Out
                        };
                    }
                }
            }

            Some(quote! { type #type_name = #selection; })
        } else {
            None
        }
    }).collect();

    // Standard bounds using shared utility
    let standard_bounds = standard_capability_bounds();

    quote! {
        impl<T: ::tola_caps::std_caps::AutoCapSet> #trait_name for T
        where
            #standard_bounds,
        {
            #(#type_chains)*
            #(#method_chains)*
        }
    }
}

// =============================================================================
// Part 5: Attribute Macro Support
// =============================================================================

/// Per-generic capability constraint
///
/// Represents `T: Clone & Debug` or `U: Copy | Default`
#[derive(Clone)]
pub struct GenericConstraint {
    /// The generic parameter name (T, U, V, etc.)
    pub generic_param: Ident,
    /// The capability expression (Clone & Debug, Copy | Default, etc.)
    pub capability_expr: crate::common::BoolExpr,
}

impl Parse for GenericConstraint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let generic_param: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let capability_expr = input.parse()?;
        Ok(GenericConstraint { generic_param, capability_expr })
    }
}

/// Attribute macro arguments for #[specialization(...)]
///
/// Supports:
/// - `#[specialization]` - basic specialization
/// - `#[specialization(default)]` - mark as default impl
/// - `#[specialization(T: Clone)]` - explicit generic constraint
/// - `#[specialization(T: Clone, U: Copy)]` - multiple generics with constraints
/// - `#[specialization(T: Clone & Debug | Default)]` - boolean expression constraints
#[allow(dead_code)]
pub struct SpecializeAttr {
    pub is_default: bool,
    /// Legacy bounds (for `for Clone + Debug` syntax, applies to T)
    pub bounds: Vec<TraitBound>,
    /// New per-generic constraints (for `T: Clone, U: Copy` syntax)
    pub constraints: Vec<GenericConstraint>,
    pub mappings: Vec<CapabilityMapping>,
}

impl Parse for SpecializeAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_default = false;
        let mut bounds = Vec::new();
        let mut constraints = Vec::new();
        let mut mappings = Vec::new();

        if input.is_empty() {
            return Ok(SpecializeAttr { is_default, bounds, constraints, mappings });
        }

        // Check for `default` keyword first
        if input.peek(Ident) {
            let ident: Ident = input.fork().parse()?;
            if ident == "default" {
                input.parse::<Ident>()?;
                is_default = true;
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }
        }

        // Try to parse new syntax: `T: Clone, U: Copy`
        // Check if next token is an Ident followed by `:`
        if input.peek(Ident) && !input.peek2(syn::token::Paren) {
            let fork = input.fork();
            if let Ok(ident) = fork.parse::<Ident>() {
                // Check if this looks like "T: ..." (new syntax)
                if fork.peek(Token![:]) {
                    // New syntax: parse comma-separated GenericConstraints
                    loop {
                        if input.is_empty() {
                            break;
                        }

                        // Check for map(...) or other meta items
                        if input.peek(Ident) {
                            let fork = input.fork();
                            if let Ok(id) = fork.parse::<Ident>() {
                                if id == "map" || id == "default" {
                                    break;
                                }
                            }
                        }

                        let constraint: GenericConstraint = input.parse()?;
                        constraints.push(constraint);

                        if input.peek(Token![,]) {
                            input.parse::<Token![,]>()?;
                        } else {
                            break;
                        }
                    }
                } else if ident == "for" || input.peek(Token![for]) {
                    // Legacy syntax: `for Clone + Debug`
                    // Fall through to legacy parsing below
                }
            }
        }

        // Legacy syntax: `for Trait + Trait2` (applies to first generic T)
        if constraints.is_empty() && input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            // Parse trait bounds separated by +
            loop {
                if input.is_empty() || input.peek(Token![,]) {
                    break;
                }
                bounds.push(input.parse()?);
                if input.peek(Token![+]) {
                    input.parse::<Token![+]>()?;
                } else {
                    break;
                }
            }
        }

        // Parse remaining items (map, etc.)
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        while !input.is_empty() {
            if let Ok(meta) = input.parse::<Meta>() {
                match meta {
                    Meta::Path(p) if p.is_ident("default") => {
                        is_default = true;
                    }
                    Meta::List(ml) if ml.path.is_ident("map") => {
                        let mapping: CapabilityMapping = syn::parse2(ml.tokens)?;
                        mappings.push(mapping);
                    }
                    _ => {}
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(SpecializeAttr { is_default, bounds, constraints, mappings })
    }
}

/// Expand #[specialize] attribute on an impl block
///
/// # Usage
///
/// ```ignore
/// // Default implementation (most general)
/// #[specialize(default)]
/// impl<T> MyTrait for Container<T> {
///     fn method(&self) { /* fallback */ }
/// }
///
/// // Specialized for Clone types
/// #[specialize(for Clone)]
/// impl<T> MyTrait for Container<T> {
///     fn method(&self) { /* optimized for Clone */ }
/// }
///
/// // Specialized for Clone + Debug
/// #[specialize(for Clone + Debug)]
/// impl<T> MyTrait for Container<T> {
///     fn method(&self) { /* even more specialized */ }
/// }
///
/// // Concrete type (most specific)
/// #[specialize]
/// impl MyTrait for Container<String> {
///     fn method(&self) { /* String-specific */ }
/// }
/// ```
pub fn expand_specialize_attr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as SpecializeAttr);
    let input = parse_macro_input!(item as ItemImpl);

    expand_specialize_attr_impl(args, input).into()
}

fn expand_specialize_attr_impl(args: SpecializeAttr, input: ItemImpl) -> TokenStream2 {
    let self_ty = &input.self_ty;
    let generics = &input.generics;
    let is_generic = !generics.params.is_empty();
    let trait_path = &input.trait_;

    // Get trait map for mapping bounds to capabilities
    let trait_map = builtin_trait_map();

    // Build where clause additions based on legacy `for` bounds (applies to T)
    let mut additional_bounds: Vec<_> = args.bounds.iter().map(|bound| {
        // Map trait names to capability markers using shared utility
        let cap_marker = bound_to_capability_with_fallback(bound, &trait_map);

        quote! {
            ::tola_caps::std_caps::Cap<T>: ::tola_caps::capability::Evaluate<#cap_marker, Out = ::tola_caps::Present>
        }
    }).collect();

    // Collect all generic parameters that need AutoCapSet bound
    let mut auto_cap_set_bounds = Vec::new();

    // For legacy bounds (applies to T)
    if !args.bounds.is_empty() {
        auto_cap_set_bounds.push(quote! { T: ::tola_caps::std_caps::AutoCapSet });
    }

    // For new per-generic constraints
    for constraint in &args.constraints {
        let generic_param = &constraint.generic_param;
        let cap_type = crate::common::bool_expr_to_capability_type(&constraint.capability_expr);

        // Add AutoCapSet bound for this generic
        auto_cap_set_bounds.push(quote! { #generic_param: ::tola_caps::std_caps::AutoCapSet });

        // Add the capability constraint
        additional_bounds.push(quote! {
            ::tola_caps::std_caps::Cap<#generic_param>: ::tola_caps::capability::Evaluate<#cap_type, Out = ::tola_caps::Present>
        });
    }

    // For concrete type: generate marker trait using shared utility
    if !is_generic {
        let marker_name = marker_trait_name(self_ty);

        let (impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
        let trait_tokens = trait_path.as_ref().map(|(bang, path, for_token)| {
            quote! { #bang #path #for_token }
        });

        let items = &input.items;
        let attrs = &input.attrs;

        quote! {
            /// Auto-generated marker trait for type specialization.
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub trait #marker_name {}
            impl #marker_name for #self_ty {}

            #(#attrs)*
            impl #impl_generics #trait_tokens #self_ty #where_clause {
                #(#items)*
            }
        }
    } else if !additional_bounds.is_empty() || !auto_cap_set_bounds.is_empty() {
        // Generic impl with capability bounds
        let (impl_generics, _ty_generics, existing_where) = generics.split_for_impl();
        let trait_tokens = trait_path.as_ref().map(|(bang, path, for_token)| {
            quote! { #bang #path #for_token }
        });

        let items = &input.items;
        let attrs = &input.attrs;

        // Combine all bounds: AutoCapSet bounds + capability bounds + existing where clause
        let all_bounds: Vec<_> = auto_cap_set_bounds.iter()
            .chain(additional_bounds.iter())
            .collect();

        // Merge existing where clause with new bounds
        let where_clause = if let Some(existing) = existing_where {
            let existing_preds = &existing.predicates;
            quote! {
                where #existing_preds, #(#all_bounds),*
            }
        } else {
            quote! {
                where #(#all_bounds),*
            }
        };

        quote! {
            #(#attrs)*
            impl #impl_generics #trait_tokens #self_ty #where_clause {
                #(#items)*
            }
        }
    } else {
        // Generic impl without extra bounds (default implementation)
        quote! { #input }
    }
}

// =============================================================================
// Part 6: Public Entry Points
// =============================================================================

/// Entry point for specialize! function macro
pub fn expand_specialize_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SpecializeInput);
    expand_specialize(input).into()
}

/// Entry point for inherent impl specialization
pub fn expand_specialize_inherent(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SpecializeInput);
    expand_specialize_inherent_impl(input).into()
}

fn expand_specialize_inherent_impl(input: SpecializeInput) -> TokenStream2 {
    // Check for overlaps
    if let Err(e) = check_overlaps(&input.impls) {
        return e.to_compile_error();
    }

    // Build trait map using shared function
    let mut trait_map = builtin_trait_map();
    for mapping in &input.mappings {
        let key = path_to_string(&mapping.trait_name);
        let cap = &mapping.capability;
        trait_map.insert(key, quote! { #cap });
    }

    // Sort impls by specificity
    let mut sorted_impls = input.impls;
    sorted_impls.sort_by_key(|imp| imp.specificity);

    // For inherent impls, generate conditional methods
    let mut methods = Vec::new();
    for (_idx, imp) in sorted_impls.iter().enumerate() {
        let self_ty = &imp.self_ty;
        let generics = &imp.generics;

        for item in &imp.items {
            if let SpecImplItem::Method(m) = item {
                let method_name = &m.name;
                let body = &m.body;
                let ret = m.sig.return_type.as_ref()
                    .map(|r| quote! { -> #r })
                    .unwrap_or_default();

                let receiver = match m.sig.receiver {
                    ReceiverKind::None => quote! {},
                    ReceiverKind::SelfValue => quote! { self },
                    ReceiverKind::SelfRef => quote! { &self },
                    ReceiverKind::SelfMutRef => quote! { &mut self },
                };

                // Build where clause from bounds using shared utility
                let bounds: Vec<_> = imp.bounds.iter()
                    .filter_map(|bound| {
                        bound_to_capability(bound, &trait_map).map(|cap| {
                            quote! { ::tola_caps::std_caps::Cap<Self>: ::tola_caps::capability::Evaluate<#cap, Out = ::tola_caps::Present> }
                        })
                    })
                    .collect();

                let where_clause = if bounds.is_empty() {
                    quote! {}
                } else {
                    quote! { where #(#bounds),* }
                };

                methods.push(quote! {
                    impl #generics #self_ty #where_clause {
                        #[inline(always)]
                        pub fn #method_name(#receiver) #ret {
                            #body
                        }
                    }
                });
            }
        }
    }

    quote! { #(#methods)* }
}

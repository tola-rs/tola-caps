//! Capability set construction and batch definition macros
//!
//! - `caps!` / `cap_set!` - build capability set types
//! - `define_capabilities!` - batch define capabilities with doc strings

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitStr, Token, Type,
};

// =============================================================================
// caps! / cap_set! Input Parser
// =============================================================================

pub struct CapsInput {
    pub types: Punctuated<Type, Token![,]>,
}

impl Parse for CapsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let types = Punctuated::parse_terminated(input)?;
        Ok(CapsInput { types })
    }
}

/// Check for duplicate capabilities in the list
pub fn check_duplicates(types: &[Type]) -> syn::Result<()> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for ty in types {
        let ty_str = ty.to_token_stream().to_string().replace(' ', "");
        if !seen.insert(ty_str.clone()) {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "duplicate capability `{}`\n\
                     \n\
                     Each capability should appear only once in a capability set.\n\
                     Duplicate capabilities cause type inference ambiguity.",
                    ty_str
                ),
            ));
        }
    }
    Ok(())
}

/// Build capset type: <<Empty as Add<C>>::Out as Add<B>>::Out as Add<A>>::Out
pub fn build_capset(types: &[Type]) -> TokenStream2 {
    if types.is_empty() {
        quote! { ::tola_caps::Empty }
    } else {
        let mut result = quote! { ::tola_caps::Empty };
        for ty in types.iter().rev() {
            result = quote! { <#result as ::tola_caps::With<#ty>>::Out };
        }
        result
    }
}

// =============================================================================
// define_capabilities! Input Parser
// =============================================================================

/// Single capability definition: `Name => "doc string"`
pub struct CapDef {
    pub name: Ident,
    pub doc: LitStr,
}

impl Parse for CapDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;
        let doc: LitStr = input.parse()?;
        Ok(CapDef { name, doc })
    }
}

/// Multiple capability definitions separated by commas
pub struct DefineCapabilitiesInput {
    pub caps: Punctuated<CapDef, Token![,]>,
}

impl Parse for DefineCapabilitiesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let caps = Punctuated::parse_terminated(input)?;
        Ok(DefineCapabilitiesInput { caps })
    }
}

// =============================================================================
// expand_define_capabilities
// =============================================================================

pub fn expand_define_capabilities(input: DefineCapabilitiesInput) -> TokenStream2 {
    let caps: Vec<_> = input.caps.into_iter().collect();

    // Collect all capability names for cross-recursive impls
    let cap_names: Vec<_> = caps.iter().map(|c| &c.name).collect();
    let cap_structs: Vec<_> = cap_names
        .iter()
        .map(|n| format_ident!("{}Cap", n))
        .collect();
    let has_traits: Vec<_> = cap_names
        .iter()
        .map(|n| format_ident!("Has{}Cap", n))
        .collect();
    let not_has_traits: Vec<_> = cap_names
        .iter()
        .map(|n| format_ident!("NotHas{}Cap", n))
        .collect();

    // Generate struct definitions, trait definitions, and base impls
    let struct_defs: Vec<_> = caps
        .iter()
        .zip(cap_structs.iter())
        .zip(has_traits.iter())
        .zip(not_has_traits.iter())
        .map(|(((cap, struct_name), has_trait), not_has_trait)| {
            let doc = &cap.doc;
            let name_str = cap.name.to_string();
            let struct_name_str = struct_name.to_string();

            // Pre-compute diagnostic messages as string literals
            let has_diag_message = format!(
                "capability `{}` is required but not available in `{{Self}}`",
                struct_name_str
            );
            let has_diag_label = format!("this transform requires `{}`", struct_name_str);
            let has_doc_trait = format!(
                "Trait to check if a capability set contains `{}`",
                struct_name_str
            );

            let not_has_diag_message = format!(
                "capability `{}` must NOT be present in `{{Self}}`",
                struct_name_str
            );
            let not_has_diag_label = format!(
                "this transform must run BEFORE `{}` is added",
                struct_name_str
            );
            let not_has_doc_trait = format!(
                "Trait to check if a capability set does NOT contain `{}`",
                struct_name_str
            );

            let doc_marker = format!("Marker: {}", doc.value());

            quote! {
                #[doc = #doc_marker]
                #[derive(Debug, Clone, Copy, Default)]
                pub struct #struct_name;

                impl sealed::Sealed for #struct_name {}

                impl Capability for #struct_name {
                    const NAME: &'static str = #name_str;
                }

                // HasXxxCap trait (presence check)
                #[doc = #has_doc_trait]
                #[diagnostic::on_unimplemented(
                    message = #has_diag_message,
                    label = #has_diag_label,
                    note = "try adding the appropriate Transform earlier in the pipeline"
                )]
                pub trait #has_trait: Capabilities {}

                // Base case: this cap at head
                impl<Rest: Capabilities> #has_trait for (#struct_name, Rest) {}

                // NotHasXxxCap trait (absence check)
                #[doc = #not_has_doc_trait]
                #[doc = ""]
                #[doc = "Use with `#[requires_not]` to enforce ordering constraints."]
                #[diagnostic::on_unimplemented(
                    message = #not_has_diag_message,
                    label = #not_has_diag_label,
                    note = "this transform must run earlier in the pipeline"
                )]
                pub trait #not_has_trait: Capabilities {}

                // Base case: empty set does not contain any cap
                impl #not_has_trait for () {}
            }
        })
        .collect();

    // Generate cross-recursive impls for HasXxxCap
    let has_cross_impls: Vec<_> = has_traits
        .iter()
        .enumerate()
        .flat_map(|(target_idx, target_trait)| {
            cap_structs
                .iter()
                .enumerate()
                .filter(move |(other_idx, _)| *other_idx != target_idx)
                .map(move |(_, other_struct)| {
                    quote! {
                        impl<Rest: #target_trait> #target_trait for (#other_struct, Rest) {}
                    }
                })
        })
        .collect();

    // Cross-recursive impls for NotHasXxxCap
    let not_has_cross_impls: Vec<_> = not_has_traits
        .iter()
        .enumerate()
        .flat_map(|(target_idx, target_trait)| {
            cap_structs
                .iter()
                .enumerate()
                .filter(move |(other_idx, _)| *other_idx != target_idx)
                .map(move |(_, other_struct)| {
                    quote! {
                        impl<Rest: #target_trait> #target_trait for (#other_struct, Rest) {}
                    }
                })
        })
        .collect();

    quote! {
        #(#struct_defs)*
        #(#has_cross_impls)*
        #(#not_has_cross_impls)*
    }
}

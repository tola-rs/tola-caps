//! Capability constraint macro: #[caps_bound]
//!
//! Add capability constraints to functions, structs, enums, and impl blocks.
//!
//! # Syntax (Unified)
//!
//! ```ignore
//! // Specify target generic with T: Caps style
//! #[caps_bound(C: Parsed & Validated)]
//! fn process<C>(doc: Doc<C>) { ... }
//!
//! // Add capabilities with 'with'
//! #[caps_bound(C: Parsed, with(Validated))]
//! fn validate<C>(doc: Doc<C>) -> Doc<with![C, Validated]> { ... }
//!
//! // Remove capabilities with 'without'
//! #[caps_bound(C: Admin, without(Admin))]
//! fn drop_admin<C>(user: User<C>) -> User<without![C, Admin]> { ... }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Ident, ItemFn, Token, Type,
};

use crate::common::{bool_expr_to_string, bool_expr_to_type, BoolExpr, peek_generic_constraint};

// Keywords excluded from generic constraint detection
const CAPS_BOUND_KEYWORDS: &[&str] = &["with", "without", "transparent", "requires", "conflicts", "target"];

// =============================================================================
// CapsArgs - Attribute Arguments Parser
// =============================================================================

pub struct CapsArgs {
    pub predicates: Vec<BoolExpr>,
    pub with_caps: Vec<Type>,
    pub without_caps: Vec<Type>,
    pub transparent: bool,
    pub target: Option<syn::Ident>,
}

impl Parse for CapsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut predicates = Vec::new();
        let mut with_caps = Vec::new();
        let mut without_caps = Vec::new();
        let mut transparent = false;
        let mut target = None;

        while !input.is_empty() {
            // 1. New unified syntax: T: BoolExpr (generic constraint)
            if peek_generic_constraint(input, CAPS_BOUND_KEYWORDS) {
                let t: Ident = input.parse()?;
                input.parse::<Token![:]>()?;
                let expr: BoolExpr = input.parse()?;
                target = Some(t);
                predicates.push(expr);
            }
            // 2. Legacy: key = value syntax (for backwards compatibility)
            else if input.peek(Ident) && input.peek2(Token![=]) {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                if key == "requires" {
                    predicates.push(input.parse()?);
                } else if key == "conflicts" {
                    let conflict: BoolExpr = input.parse()?;
                    predicates.push(BoolExpr::Not(Box::new(conflict)));
                } else if key == "with" {
                    with_caps.push(input.parse()?);
                } else if key == "without" {
                    without_caps.push(input.parse()?);
                } else if key == "transparent" {
                    let val: syn::LitBool = input.parse()?;
                    transparent = val.value;
                } else if key == "target" {
                    // Legacy target = C syntax - still supported
                    target = Some(input.parse()?);
                }
            }
            // 3. Check for grouping: with(...) / without(...)
            else if input.peek(Ident) && input.peek2(syn::token::Paren) && {
                let fork = input.fork();
                let key: Ident = fork.parse().unwrap();
                key == "with" || key == "without"
            } {
                let key: Ident = input.parse()?;
                let content;
                syn::parenthesized!(content in input);
                let types =
                    syn::punctuated::Punctuated::<Type, Token![,]>::parse_terminated(&content)?;

                if key == "with" {
                    with_caps.extend(types);
                } else if key == "without" {
                    without_caps.extend(types);
                }
            }
            // 4. Check for flags: transparent
            else if input.peek(Ident) && {
                let fork = input.fork();
                let key: Ident = fork.parse().unwrap();
                key == "transparent"
            } {
                let _: Ident = input.parse()?; // consume
                transparent = true;
            }
            // 5. Positional boolean expression (requires / !conflicts) - no target specified
            else {
                let expr: BoolExpr = input.parse()?;
                predicates.push(expr);
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(CapsArgs {
            predicates,
            with_caps,
            without_caps,
            transparent,
            target,
        })
    }
}

// =============================================================================
// Predicate Generation
// =============================================================================

/// Generate predicates from args (for structs, enums, impl blocks)
pub fn generate_predicates(args: &CapsArgs, bound_param: &syn::Ident) -> Vec<TokenStream2> {
    let mut output_tokens = Vec::new();

    for pred in &args.predicates {
        let type_expr = bool_expr_to_type(pred);
        output_tokens.push(quote! {
            #bound_param: ::tola_caps::Require<#type_expr>
        });
    }

    for cap in &args.with_caps {
        output_tokens.push(quote! {
            #bound_param: ::tola_caps::With<#cap>
        });
    }

    for cap in &args.without_caps {
        output_tokens.push(quote! {
            #bound_param: ::tola_caps::Without<#cap>
        });
    }

    output_tokens
}

/// Generate unique traits for each predicate to support custom diagnostic messages
pub fn generate_predicate_traits(
    args: &CapsArgs,
    bound_param: &syn::Ident,
    fn_name: &syn::Ident,
) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    let mut bounds = Vec::new();
    let mut definitions = Vec::new();

    for (i, pred) in args.predicates.iter().enumerate() {
        let type_expr = bool_expr_to_type(pred);
        let msg_str = bool_expr_to_string(pred);

        // Generate a unique name for this requirement trait
        let trait_name = format_ident!("__Req_{}_{}", fn_name, i);

        let message = format!("Capability requirement failed: {}", msg_str);
        let label = format!("This capability set violates requirement '{}'", msg_str);

        // Generate the definition
        definitions.push(quote! {
            #[allow(non_camel_case_types)]
            #[diagnostic::on_unimplemented(
                message = #message,
                label = #label,
                note = "Check if you are missing a required capability or possess a conflicting one."
            )]
            pub trait #trait_name {
                 const MSG: &'static str = #msg_str;
            }

            impl<C> #trait_name for C
            where
                C: ::tola_caps::Evaluate<#type_expr>,
                <C as ::tola_caps::Evaluate<#type_expr>>::Out: ::tola_caps::IsTrue<C, #type_expr>
            {}
        });

        // Generate the bound
        bounds.push(quote! {
            #bound_param: #trait_name
        });
    }

    (bounds, definitions)
}

// =============================================================================
// Expand Functions
// =============================================================================

/// Find the correct insertion position for __C in generic params
fn find_insert_position(
    params: &syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma>,
) -> usize {
    let mut pos = 0;
    for (i, param) in params.iter().enumerate() {
        match param {
            syn::GenericParam::Lifetime(_) => {
                pos = i + 1;
            }
            syn::GenericParam::Type(t) => {
                if t.default.is_some() {
                    return pos;
                }
                pos = i + 1;
            }
            syn::GenericParam::Const(_) => {
                return pos;
            }
        }
    }
    pos
}

pub fn expand_caps_fn(args: CapsArgs, mut func: ItemFn) -> TokenStream {
    let generic_param = format_ident!("__C");
    let fn_name = func.sig.ident.clone();

    if args.transparent {
        let insert_pos = find_insert_position(&func.sig.generics.params);
        func.sig
            .generics
            .params
            .insert(insert_pos, syn::parse_quote!(#generic_param));

        for arg in &mut func.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let Type::Path(type_path) = &mut *pat_type.ty {
                    if let Some(last_seg) = type_path.path.segments.last_mut() {
                        if last_seg.ident == "Doc" {
                            if let syn::PathArguments::None = last_seg.arguments {
                                last_seg.arguments = syn::PathArguments::AngleBracketed(
                                    syn::parse_quote!(<#generic_param>),
                                );
                            } else if let syn::PathArguments::AngleBracketed(ga) =
                                &mut last_seg.arguments
                            {
                                ga.args.push(syn::parse_quote!(#generic_param));
                            }
                        }
                    }
                }
            }
        }
    }

    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else if args.transparent {
        generic_param.clone()
    } else {
        func.sig
            .generics
            .params
            .iter()
            .filter_map(|p| {
                if let syn::GenericParam::Type(t) = p {
                    Some(t.ident.clone())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| format_ident!("C"))
    };

    // Generate predicate traits and bounds
    let (pred_bounds, pred_defs) = generate_predicate_traits(&args, &bound_param, &fn_name);

    if let Some(wc) = &mut func.sig.generics.where_clause {
        for b in pred_bounds {
            wc.predicates.push(syn::parse_quote!(#b));
        }
    } else {
        let mut wc: syn::WhereClause = syn::parse_quote!(where);
        for b in pred_bounds {
            wc.predicates.push(syn::parse_quote!(#b));
        }
        func.sig.generics.where_clause = Some(wc);
    }

    // Add With/Without bounds
    let where_clause = func.sig.generics.make_where_clause();

    for cap in &args.with_caps {
        where_clause
            .predicates
            .push(syn::parse_quote!(#bound_param: ::tola_caps::With<#cap>));
    }

    for cap in &args.without_caps {
        where_clause
            .predicates
            .push(syn::parse_quote!(#bound_param: ::tola_caps::Without<#cap>));
    }

    // Output: definitions + function
    quote! {
        #(#pred_defs)*
        #func
    }
    .into()
}

pub fn expand_caps_struct(args: CapsArgs, mut item: syn::ItemStruct) -> TokenStream {
    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else {
        item.generics
            .params
            .iter()
            .filter_map(|p| {
                if let syn::GenericParam::Type(t) = p {
                    Some(t.ident.clone())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| format_ident!("C"))
    };

    let predicates = generate_predicates(&args, &bound_param);
    let where_clause = item.generics.make_where_clause();
    for pred in predicates {
        where_clause.predicates.push(syn::parse_quote!(#pred));
    }

    item.into_token_stream().into()
}

pub fn expand_caps_enum(args: CapsArgs, mut item: syn::ItemEnum) -> TokenStream {
    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else {
        item.generics
            .params
            .iter()
            .filter_map(|p| {
                if let syn::GenericParam::Type(t) = p {
                    Some(t.ident.clone())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| format_ident!("C"))
    };

    let predicates = generate_predicates(&args, &bound_param);
    let where_clause = item.generics.make_where_clause();
    for pred in predicates {
        where_clause.predicates.push(syn::parse_quote!(#pred));
    }

    item.into_token_stream().into()
}

pub fn expand_caps_impl(args: CapsArgs, mut item: syn::ItemImpl) -> TokenStream {
    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else {
        item.generics
            .params
            .iter()
            .filter_map(|p| {
                if let syn::GenericParam::Type(t) = p {
                    Some(t.ident.clone())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| format_ident!("C"))
    };

    let predicates = generate_predicates(&args, &bound_param);
    let where_clause = item.generics.make_where_clause();
    for pred in predicates {
        where_clause.predicates.push(syn::parse_quote!(#pred));
    }

    item.into_token_stream().into()
}

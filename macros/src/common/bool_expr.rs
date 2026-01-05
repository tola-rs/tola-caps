// Boolean expression parsing and evaluation for capability constraints

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    Token, Type,
};

// =============================================================================
// Boolean Expression AST
// =============================================================================

#[derive(Clone, Debug)]
pub enum BoolExpr {
    Cap(Type),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
}

impl Parse for BoolExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_or(input)
    }
}

// Recursive descent parser: Or -> And -> Unary -> Primary

fn parse_or(input: ParseStream) -> syn::Result<BoolExpr> {
    let mut lhs = parse_and(input)?;

    while input.peek(Token![|]) {
        input.parse::<Token![|]>()?;
        let rhs = parse_and(input)?;
        lhs = BoolExpr::Or(Box::new(lhs), Box::new(rhs));
    }
    Ok(lhs)
}

fn parse_and(input: ParseStream) -> syn::Result<BoolExpr> {
    let mut lhs = parse_unary(input)?;

    while input.peek(Token![&]) {
        input.parse::<Token![&]>()?;
        let rhs = parse_unary(input)?;
        lhs = BoolExpr::And(Box::new(lhs), Box::new(rhs));
    }
    Ok(lhs)
}

fn parse_unary(input: ParseStream) -> syn::Result<BoolExpr> {
    if input.peek(Token![!]) {
        input.parse::<Token![!]>()?;
        let operand = parse_unary(input)?;
        Ok(BoolExpr::Not(Box::new(operand)))
    } else {
        parse_primary(input)
    }
}

fn parse_primary(input: ParseStream) -> syn::Result<BoolExpr> {
    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        content.parse()
    } else {
        // Parse a type (Capability)
        let ty: Type = input.parse()?;
        Ok(BoolExpr::Cap(ty))
    }
}

// =============================================================================
// BoolExpr Utilities
// =============================================================================

/// Convert BoolExpr to human-readable string
pub fn bool_expr_to_string(expr: &BoolExpr) -> String {
    match expr {
        BoolExpr::Cap(ty) => quote!(#ty).to_string().replace(' ', ""),
        BoolExpr::And(lhs, rhs) => {
            format!("({} & {})", bool_expr_to_string(lhs), bool_expr_to_string(rhs))
        }
        BoolExpr::Or(lhs, rhs) => {
            format!("({} | {})", bool_expr_to_string(lhs), bool_expr_to_string(rhs))
        }
        BoolExpr::Not(operand) => format!("!{}", bool_expr_to_string(operand)),
    }
}

/// Convert BoolExpr to type-level representation
pub fn bool_expr_to_type(expr: &BoolExpr) -> TokenStream {
    match expr {
        BoolExpr::Cap(ty) => quote! { #ty },
        BoolExpr::And(lhs, rhs) => {
            let l = bool_expr_to_type(lhs);
            let r = bool_expr_to_type(rhs);
            quote! { ::tola_caps::And<#l, #r> }
        }
        BoolExpr::Or(lhs, rhs) => {
            let l = bool_expr_to_type(lhs);
            let r = bool_expr_to_type(rhs);
            quote! { ::tola_caps::Or<#l, #r> }
        }
        BoolExpr::Not(operand) => {
            let o = bool_expr_to_type(operand);
            quote! { ::tola_caps::Not<#o> }
        }
    }
}

/// Convert BoolExpr to type-level representation with trait name to capability marker mapping.
/// Maps: Clone -> IsClone, Copy -> IsCopy, Debug -> IsDebug, etc.
pub fn bool_expr_to_capability_type(expr: &BoolExpr) -> TokenStream {
    match expr {
        BoolExpr::Cap(ty) => {
            // Map trait names to capability markers
            let ty_str = quote!(#ty).to_string();
            match ty_str.as_str() {
                "Clone" => quote! { ::tola_caps::detect::IsClone },
                "Copy" => quote! { ::tola_caps::detect::IsCopy },
                "Debug" => quote! { ::tola_caps::detect::IsDebug },
                "Default" => quote! { ::tola_caps::detect::IsDefault },
                "Send" => quote! { ::tola_caps::detect::IsSend },
                "Sync" => quote! { ::tola_caps::detect::IsSync },
                _ => {
                    // Custom trait: assume Is{TraitName} marker exists
                    let marker = quote::format_ident!("Is{}", ty_str);
                    quote! { #marker }
                }
            }
        }
        BoolExpr::And(lhs, rhs) => {
            let l = bool_expr_to_capability_type(lhs);
            let r = bool_expr_to_capability_type(rhs);
            quote! { ::tola_caps::And<#l, #r> }
        }
        BoolExpr::Or(lhs, rhs) => {
            let l = bool_expr_to_capability_type(lhs);
            let r = bool_expr_to_capability_type(rhs);
            quote! { ::tola_caps::Or<#l, #r> }
        }
        BoolExpr::Not(operand) => {
            let o = bool_expr_to_capability_type(operand);
            quote! { ::tola_caps::Not<#o> }
        }
    }
}

/// Convert trait name to SCREAMING_SNAKE_CASE for IS_* constant.
/// e.g., "Clone" -> "CLONE", "MyTrait" -> "MY_TRAIT"
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

/// Built-in traits supported by AutoCaps
const BUILTIN_TRAITS: &[&str] = &["Clone", "Copy", "Debug", "Default", "Send", "Sync"];

fn is_builtin_trait(name: &str) -> bool {
    BUILTIN_TRAITS.contains(&name)
}

/// Check if an expression contains ONLY built-in traits (no custom traits)
fn is_all_builtin(expr: &BoolExpr) -> bool {
    match expr {
        BoolExpr::Cap(trait_ty) => {
            let trait_str = quote!(#trait_ty).to_string();
            is_builtin_trait(&trait_str)
        }
        BoolExpr::And(lhs, rhs) => is_all_builtin(lhs) && is_all_builtin(rhs),
        BoolExpr::Or(lhs, rhs) => is_all_builtin(lhs) && is_all_builtin(rhs),
        BoolExpr::Not(operand) => is_all_builtin(operand),
    }
}

// =============================================================================
// Unified caps_check! Implementation (Autoref Fallback)
// =============================================================================

/// Generate unified check code using autoref-based dispatch.
///
/// For built-in traits:
///   Uses autoref dispatch: Inherent `AutoCaps` method (High Priority) > Trait method (Low Priority).
///
/// For custom traits:
///   Uses Probe pattern directly (since AutoCaps only covers built-in traits).
///
/// NOT Logic:
///   The NOT operator must be applied AFTER retrieving the capability status,
///   because a "Probe" failure (false) on a generic type simply means "unknown",
///   not necessarily "definitely not implemented".
pub fn generate_unified_check(expr: &BoolExpr, ty: &Type) -> TokenStream {
    // If expression contains custom traits, Probe is the only option
    // because Probe pattern only works with concrete types (not generic T)
    if !is_all_builtin(expr) {
        let probe_body = generate_probe_body(expr, ty);
        // Wrap with type reference to prevent "unused import" warnings
        // Use a let binding instead of const to allow generic type parameters
        return quote! {
            {
                // Reference user's type to prevent "unused import" warnings
                let _ = ::core::marker::PhantomData::<#ty>;
                #probe_body
            }
        };
    }

    // All built-in traits: use autoref dispatch for efficiency
    // Fallback: Probe only (for types without AutoCaps)
    let probe_only_body = generate_probe_body(expr, ty);
    // Inherent: Probe || AutoCaps (for types with AutoCaps)
    let combined_body = generate_combined_body_inherent(expr, ty);

    quote! {
        {
            use ::core::marker::PhantomData;

            struct __Wrapper<T: ?Sized>(PhantomData<T>);

            // Low priority: trait method on &Wrapper<T> (fallback for all T)
            // This is used when T does NOT implement AutoCaps
            // Uses Probe only - works for concrete types
            trait __Fallback { fn __check(&self) -> bool; }
            impl<T: ?Sized> __Fallback for &__Wrapper<T> {
                #[inline]
                fn __check(&self) -> bool { #probe_only_body }
            }

            // High priority: inherent method on Wrapper<T> where T: AutoCaps
            // Uses Probe || AutoCaps strategy
            impl<T: ?Sized + ::tola_caps::detect::AutoCaps> __Wrapper<T> {
                #[inline]
                fn __check(&self) -> bool { #combined_body }
            }

            // Method resolution: inherent method preferred over trait method
            (&__Wrapper::<#ty>(PhantomData)).__check()
        }
    }
}

/// Generate combined body for inherent impl (T: AutoCaps).
/// Uses Probe || AutoCaps strategy with correct NOT semantics.
///
/// For each atomic trait check X:
///   result_X = Probe<#ty, X> || AutoCaps<T>::IS_X
///
/// For NOT: !X becomes !result_X (NOT applied after combination)
/// For AND: X & Y becomes result_X && result_Y
/// For OR: X | Y becomes result_X || result_Y
fn generate_combined_body_inherent(expr: &BoolExpr, ty: &Type) -> TokenStream {
    match expr {
        BoolExpr::Cap(trait_ty) => {
            let trait_str = quote!(#trait_ty).to_string();
            if is_builtin_trait(&trait_str) {
                let const_name = format_ident!("IS_{}", to_screaming_snake_case(&trait_str));
                // Combined: Probe<#ty> || AutoCaps<T>
                quote! {
                    {
                        // Probe: detects if concrete type implements trait
                        let __probe = {
                            trait __ProbeFallback { const VAL: bool = false; }
                            struct __Probe<X: ?Sized>(::core::marker::PhantomData<X>);
                            impl<X: ?Sized> __ProbeFallback for __Probe<X> {}
                            impl<X: ?Sized + #trait_ty> __Probe<X> { const VAL: bool = true; }
                            __Probe::<#ty>::VAL
                        };
                        // AutoCaps: claimed capability (may be from generic bound)
                        let __autocaps = <T as ::tola_caps::detect::AutoCaps>::#const_name;
                        // Combine: either detection confirms the trait
                        __probe || __autocaps
                    }
                }
            } else {
                // Custom trait: use probe only
                generate_single_probe(trait_ty, ty)
            }
        }
        BoolExpr::And(lhs, rhs) => {
            let l = generate_combined_body_inherent(lhs, ty);
            let r = generate_combined_body_inherent(rhs, ty);
            quote! { (#l && #r) }
        }
        BoolExpr::Or(lhs, rhs) => {
            let l = generate_combined_body_inherent(lhs, ty);
            let r = generate_combined_body_inherent(rhs, ty);
            quote! { (#l || #r) }
        }
        BoolExpr::Not(operand) => {
            // CRITICAL: Apply NOT after the combined result!
            let o = generate_combined_body_inherent(operand, ty);
            quote! { (!#o) }
        }
    }
}

/// Generate Probe-based check body (for any trait)
fn generate_probe_body(expr: &BoolExpr, ty: &Type) -> TokenStream {
    match expr {
        BoolExpr::Cap(trait_ty) => generate_single_probe(trait_ty, ty),
        BoolExpr::And(lhs, rhs) => {
            let l = generate_probe_body(lhs, ty);
            let r = generate_probe_body(rhs, ty);
            quote! { (#l && #r) }
        }
        BoolExpr::Or(lhs, rhs) => {
            let l = generate_probe_body(lhs, ty);
            let r = generate_probe_body(rhs, ty);
            quote! { (#l || #r) }
        }
        BoolExpr::Not(operand) => {
            let o = generate_probe_body(operand, ty);
            quote! { (!#o) }
        }
    }
}

/// Generate a single probe check for one trait
fn generate_single_probe(trait_ty: &Type, ty: &Type) -> TokenStream {
    quote! {
        {
            trait __ProbeFallback { const VAL: bool = false; }
            struct __Probe<X: ?Sized>(::core::marker::PhantomData<X>);
            impl<X: ?Sized> __ProbeFallback for __Probe<X> {}
            impl<X: ?Sized + #trait_ty> __Probe<X> { const VAL: bool = true; }
            __Probe::<#ty>::VAL
        }
    }
}

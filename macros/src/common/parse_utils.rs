//! Common parsing utilities
//!
//! Shared parsing helpers for consistent syntax across macros.

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, Type,
};

use super::BoolExpr;

// =============================================================================
// Generic Constraint Parsing: `T: Expr`
// =============================================================================

/// A single generic constraint: `T: BoolExpr`
///
/// Used in:
/// - `#[caps_bound(T: Parsed & Validated)]`
/// - `#[specialize(T: Clone, U: Copy)]`
#[derive(Clone)]
#[allow(dead_code)]
pub struct GenericConstraint {
    pub name: Ident,
    pub expr: BoolExpr,
}

impl Parse for GenericConstraint {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let expr: BoolExpr = input.parse()?;
        Ok(GenericConstraint { name, expr })
    }
}

/// Check if next tokens look like `Ident: ...` (generic constraint)
/// Excludes keywords like 'with', 'without', 'default', etc.
pub fn peek_generic_constraint(input: ParseStream, exclude_keywords: &[&str]) -> bool {
    if input.peek(Ident) && input.peek2(Token![:]) {
        let fork = input.fork();
        if let Ok(ident) = fork.parse::<Ident>() {
            let name = ident.to_string();
            !exclude_keywords.contains(&name.as_str())
        } else {
            false
        }
    } else {
        false
    }
}

// =============================================================================
// Comma-separated List Parsing
// =============================================================================

/// Parse a comma-separated list of items
#[allow(dead_code)]
pub fn parse_comma_separated<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let items = Punctuated::<T, Token![,]>::parse_terminated(input)?;
    Ok(items.into_iter().collect())
}

/// Parse comma-separated types (e.g., in `with(A, B, C)`)
#[allow(dead_code)]
pub fn parse_type_list(input: ParseStream) -> syn::Result<Vec<Type>> {
    parse_comma_separated(input)
}

// =============================================================================
// Keyword Detection
// =============================================================================

/// Check if the next identifier is a specific keyword
#[allow(dead_code)]
pub fn peek_keyword(input: ParseStream, keyword: &str) -> bool {
    if input.peek(Ident) {
        let fork = input.fork();
        if let Ok(ident) = fork.parse::<Ident>() {
            return ident == keyword;
        }
    }
    false
}

/// Check if the next identifier is any of the given keywords
#[allow(dead_code)]
pub fn peek_any_keyword(input: ParseStream, keywords: &[&str]) -> bool {
    if input.peek(Ident) {
        let fork = input.fork();
        if let Ok(ident) = fork.parse::<Ident>() {
            let name = ident.to_string();
            return keywords.contains(&name.as_str());
        }
    }
    false
}

/// Consume an identifier if it matches the keyword
#[allow(dead_code)]
pub fn try_parse_keyword(input: ParseStream, keyword: &str) -> syn::Result<bool> {
    if peek_keyword(input, keyword) {
        let _: Ident = input.parse()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

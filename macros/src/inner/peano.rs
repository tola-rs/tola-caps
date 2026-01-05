//! Peano number generation macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, LitInt};

pub struct PeanoInput {
    pub max: usize,
}

impl Parse for PeanoInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit: LitInt = input.parse()?;
        let max = lit.base10_parse::<usize>()?;
        Ok(PeanoInput { max })
    }
}

pub fn expand_peano(input: PeanoInput) -> TokenStream {
    let max = input.max;

    // D0 = Z
    let mut types = vec![quote! { pub type D0 = Z; }];

    // D1..Dmax = S<D(n-1)>
    for n in 1..=max {
        let curr = syn::Ident::new(&format!("D{}", n), proc_macro2::Span::call_site());
        let prev = syn::Ident::new(&format!("D{}", n - 1), proc_macro2::Span::call_site());
        types.push(quote! { pub type #curr = S<#prev>; });
    }

    quote! { #(#types)* }
}

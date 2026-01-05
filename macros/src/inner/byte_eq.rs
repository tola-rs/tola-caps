//! Generate ByteEq impls for all 256*256 byte combinations.

use proc_macro2::TokenStream;
use quote::quote;

/// Generate all 65536 ByteEq impls
pub fn expand_byte_eq_impls() -> TokenStream {
    let mut impls = TokenStream::new();

    for a in 0u8..=255 {
        for b in 0u8..=255 {
            let result = if a == b {
                quote! { Present }
            } else {
                quote! { Absent }
            };

            let a_lit = proc_macro2::Literal::u8_unsuffixed(a);
            let b_lit = proc_macro2::Literal::u8_unsuffixed(b);

            impls.extend(quote! {
                impl ByteEq<#a_lit, #b_lit> for () {
                    type Out = #result;
                }
            });
        }
    }

    impls
}

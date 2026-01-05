//! Generate HashStream impl for ByteStream128
//!
//! ByteStream128 has 64 const u8 params representing nibbles (0-15 each).
//! To implement proper streaming (where Tail returns a different nibble),
//! we generate 64 wrapper types ByteStream128At0, ByteStream128At1, etc.
//! Each At_N_ type returns the N-th nibble as Head and At_{N+1} as Tail.

use proc_macro2::TokenStream;
use quote::quote;

/// Generate HashStream impls for ByteStream128.
///
/// Strategy: Create 64 "view" types that share the same const params but
/// track the current position. Each view's Tail is the next view.
///
/// ByteStream128At0<N0..N63> → Head = XN0, Tail = ByteStream128At1<N0..N63>
/// ByteStream128At1<N0..N63> → Head = XN1, Tail = ByteStream128At2<N0..N63>
/// ...
/// ByteStream128At63<N0..N63> → Head = XN63, Tail = ByteStream128At0<N0..N63> (cycle)
pub fn expand_impl_byte_stream_128() -> TokenStream {
    let const_params: Vec<_> = (0usize..64).map(|i| {
        let name = quote::format_ident!("N{}", i);
        quote! { const #name: u8 }
    }).collect();

    let const_names: Vec<_> = (0usize..64).map(|i| {
        quote::format_ident!("N{}", i)
    }).collect();

    // Generate 64 "AtN" wrapper types
    let at_types: Vec<_> = (0usize..64).map(|i| {
        let name = quote::format_ident!("ByteStream128At{}", i);
        quote! {
            pub struct #name<#(#const_params),*>(core::marker::PhantomData<()>);
        }
    }).collect();

    // Generate HashStream impls for each AtN type
    let at_impls: Vec<_> = (0usize..64).map(|i| {
        let current_name = quote::format_ident!("ByteStream128At{}", i);
        let next_i = (i + 1) % 64;
        let next_name = quote::format_ident!("ByteStream128At{}", next_i);
        let nibble_param = quote::format_ident!("N{}", i);
        let next_nibble_param = quote::format_ident!("N{}", next_i);

        quote! {
            impl<#(#const_params),*> HashStream for #current_name<#(#const_names),*>
            where
                (): SelectNibble<#nibble_param> + SelectNibble<#next_nibble_param>,
            {
                type Head = <() as SelectNibble<#nibble_param>>::Out;
                type Tail = #next_name<#(#const_names),*>;
            }
        }
    }).collect();

    // ByteStream128 delegates to ByteStream128At0
    quote! {
        // Define the 64 position-tracking wrapper types
        #(#at_types)*

        // ByteStream128 itself delegates to At0
        impl<#(#const_params),*> HashStream for ByteStream128<#(#const_names),*>
        where
            (): SelectNibble<N0>,
        {
            type Head = <() as SelectNibble<N0>>::Out;
            type Tail = ByteStream128At1<#(#const_names),*>;
        }

        // Implement HashStream for each AtN type
        #(#at_impls)*
    }
}
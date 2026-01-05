use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;


/// #[derive(Capability)] generates a call to the declarative macro bridge.
/// This allows module_path!() to be expanded before the proc-macro processes it.
///
/// The three-layer architecture:
/// 1. #[derive(Capability)] (proc-macro) -> generates __impl_capability! call
/// 2. __impl_capability! (decl-macro) -> passes concat!() to proc-macro
/// 3. make_routing_stream! / __internal_make_identity! (proc-macro) -> receives expanded string
pub fn expand_derive_capability(input: DeriveInput) -> TokenStream2 {
    let ident = &input.ident;
    let ident_str = ident.to_string();

    // Generate a call to the declarative macro bridge.
    // The concat!(module_path!(), ...) will be expanded BEFORE the inner proc-macros run.
    quote! {
        ::tola_caps::__impl_capability!(#ident, #ident_str);
    }
}

// Logic for make_routing_stream macro
pub fn expand_make_routing_stream(input: TokenStream2) -> TokenStream2 {
    // 1. Try to parse as string literal first (direct case)
    if let Ok(lit) = syn::parse2::<syn::LitStr>(input.clone()) {
        let s = lit.value();
        let hash = fnv1a_64(&s);

        // Build HashStream16 directly from hash
        let nibbles: Vec<u8> = (0..16).map(|i| {
            ((hash >> (i * 4)) & 0xF) as u8
        }).collect();

        return quote! {
            ::tola_caps::primitives::stream::HashStream16<
                #(#nibbles),*
            >
        };
    }

    // 2. If not a string literal, it's likely concat!(module_path!(), ...)
    //    Generate HashStream16 with hash computed at compile time using const fn
    quote! {
        ::tola_caps::primitives::stream::HashStream16<
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 0) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 1) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 2) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 3) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 4) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 5) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 6) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 7) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 8) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 9) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 10) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 11) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 12) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 13) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 14) },
            { ::tola_caps::primitives::const_utils::hash_nibble(#input, 15) },
        >
    }
}

// Helpers needed
fn fnv1a_64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Generate tiered IList Identity from string.
/// Tiering strategy:
/// - ≤8 chars:   IList8 (full)
/// - ≤16 chars:  IList16 (full)
/// - ≤32 chars:  IList32 (full)
/// - ≤64 chars:  IList64 (full)
/// - >64 chars:  IListSampled (head32 + mid16 + tail16)
///
/// Example:
/// "ab" (2 chars) → IList8<IList<C<'a'>, IList<C<'b'>, INil>>>
/// "very_long_module_path::VeryLongTypeName" (>64) → IListSampled<...sampled...>
pub fn expand_make_identity(input: TokenStream2) -> TokenStream2 {
    // Try to parse as string literal
    let s = if let Ok(lit) = syn::parse2::<syn::LitStr>(input.clone()) {
        lit.value()
    } else {
        // For concat!(), use const fn approach
        return generate_identity_const_fn(input);
    };

    generate_identity_tiered(&s)
}

/// Generate tiered IList based on string length
pub fn generate_identity_tiered(s: &str) -> TokenStream2 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    // Determine tier and chars to encode
    let (tier_type, sampled) = if len <= 8 {
        ("IList8", chars.clone())
    } else if len <= 16 {
        ("IList16", chars.clone())
    } else if len <= 32 {
        ("IList32", chars.clone())
    } else if len <= 64 {
        ("IList64", chars.clone())
    } else {
        // >64: Smart sampling (head32 + mid16 + tail16 = 64)
        let mut result = Vec::new();
        result.extend_from_slice(&chars[0..32]);        // Head 32
        let mid_start = (len - 16) / 2;
        result.extend_from_slice(&chars[mid_start..mid_start + 16]); // Mid 16
        result.extend_from_slice(&chars[len - 16..len]); // Tail 16
        ("IListSampled", result)
    };

    // Build IList from right to left
    let mut ilist = quote! { ::tola_caps::primitives::const_utils::INil };
    for ch in sampled.iter().rev() {
        ilist = quote! {
            ::tola_caps::primitives::const_utils::IList<
                ::tola_caps::primitives::const_utils::C<#ch>,
                #ilist
            >
        };
    }

    // Wrap with tier type
    let tier_ident = syn::Ident::new(tier_type, proc_macro2::Span::call_site());
    quote! {
        ::tola_caps::primitives::const_utils::#tier_ident<#ilist>
    }
}

/// Generate tiered IList using const fn for concat!() expressions.
fn generate_identity_const_fn(input: TokenStream2) -> TokenStream2 {
    let mut ilist = quote! { ::tola_caps::primitives::const_utils::INil };

    for i in (0usize..64).rev() {
        let idx = syn::Index::from(i);
        ilist = quote! {
            ::tola_caps::primitives::const_utils::IList<
                ::tola_caps::primitives::const_utils::C<{
                    let s = #input;
                    let indices = ::tola_caps::primitives::const_utils::sample_indices_64(s.len());
                    ::tola_caps::primitives::const_utils::str_char_at(s, indices[#idx])
                }>,
                #ilist
            >
        };
    }

    quote! {
        ::tola_caps::primitives::const_utils::IListSampled<#ilist>
    }
}

/// Generate IdentityBytes from string literal or concat!() expression.
pub fn expand_make_identity_bytes(input: TokenStream2) -> TokenStream2 {
    if let Ok(lit) = syn::parse2::<syn::LitStr>(input.clone()) {
        let s = lit.value();
        let bytes = s.as_bytes();

        let pack = |offset: usize| -> u128 {
            let mut result: u128 = 0;
            for i in 0..16 {
                let idx = offset + i;
                let byte = if idx < bytes.len() { bytes[idx] } else { 0 };
                result |= (byte as u128) << (i * 8);
            }
            result
        };

        let b0 = pack(0);
        let b1 = pack(16);
        let b2 = pack(32);
        let b3 = pack(48);

        return quote! {
            ::tola_caps::primitives::const_utils::IdentityBytes<
                #b0, #b1, #b2, #b3
            >
        };
    }

    // For concat!() expressions, use const fn approach
    //    Use const fn pack_str_bytes at compile time
    quote! {
        ::tola_caps::primitives::const_utils::IdentityBytes<
            { ::tola_caps::primitives::const_utils::pack_str_bytes(#input, 0) },
            { ::tola_caps::primitives::const_utils::pack_str_bytes(#input, 16) },
            { ::tola_caps::primitives::const_utils::pack_str_bytes(#input, 32) },
            { ::tola_caps::primitives::const_utils::pack_str_bytes(#input, 48) },
        >
    }
}

/// Derive macro that adds a phantom capability field to a struct.
pub fn expand_derive_cap_holder(input: syn::DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;

    // Get existing generics
    let mut generics = input.generics.clone();

    // Add C generic with default
    let cap_generic: syn::GenericParam = syn::parse_quote!(C = ::tola_caps::Empty);
    generics.params.push(cap_generic);

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    // Extract fields from struct
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(&input, "CapHolder can only be derived for structs")
                .to_compile_error();
        }
    };

    // Generate field definitions with added _caps field
    let (field_defs, field_names, _field_types): (Vec<_>, Vec<_>, Vec<_>) = match fields {
        syn::Fields::Named(named) => {
            let defs: Vec<_> = named.named.iter().map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                let vis = &f.vis;
                let attrs = &f.attrs;
                quote! { #(#attrs)* #vis #name: #ty }
            }).collect();
            let names: Vec<_> = named.named.iter().map(|f| f.ident.clone().unwrap()).collect();
            let types: Vec<_> = named.named.iter().map(|f| f.ty.clone()).collect();
            (defs, names, types)
        }
        _ => {
            return syn::Error::new_spanned(&input, "CapHolder only supports named fields")
                .to_compile_error();
        }
    };

    // Generate the conversion helper methods
    let field_copies: Vec<_> = field_names.iter().map(|n| quote! { #n: self.#n }).collect();

    quote! {
        #(#attrs)*
        #vis struct #name #impl_generics #where_clause {
            #(#field_defs,)*
            _caps: ::core::marker::PhantomData<C>,
        }

        impl<C> #name<C> {
            /// Convert to a new capability set (internal use).
            #[inline]
            pub fn with_caps<C2>(self) -> #name<C2> {
                #name {
                    #(#field_copies,)*
                    _caps: ::core::marker::PhantomData,
                }
            }
        }
    }
}

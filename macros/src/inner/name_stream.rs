use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::parse::{Parse, ParseStream};
use syn::Ident;

pub struct NameStreamInput {
    pub name: String,
}

impl Parse for NameStreamInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_str = if input.peek(syn::LitStr) {
            input.parse::<syn::LitStr>()?.value()
        } else {
            // Parse as path (e.g. std::option::Option)
            // We use quote! to turn path back into string, as ToTokens does
            let path: syn::Path = input.parse()?;
            quote::quote!(#path).to_string().replace(" ", "")
        };
        Ok(NameStreamInput { name: name_str })
    }
}

pub fn expand_name_stream(input: NameStreamInput) -> TokenStream {
    let name_str = input.name;
    let bytes = name_str.as_bytes();

    let mut nibbles = Vec::new();
    for byte in bytes {
        let high = (byte >> 4) & 0xF;
        let low = byte & 0xF;
        nibbles.push(high);
        nibbles.push(low);
    }

    build_stream_type(&nibbles)
}


fn build_stream_type(nibbles: &[u8]) -> TokenStream {
    if nibbles.is_empty() {
        return quote! { ::tola_caps::primitives::ConstStream<::tola_caps::primitives::nibble::X0> };
    }

    let head = nibble_to_ident(nibbles[0]);
    let tail_stream = build_stream_type(&nibbles[1..]);

    quote! {
        ::tola_caps::primitives::Cons<::tola_caps::primitives::nibble::#head, #tail_stream>
    }
}

fn nibble_to_ident(n: u8) -> Ident {
    format_ident!("X{:X}", n)
}

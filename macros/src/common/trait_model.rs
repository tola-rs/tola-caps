use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{parse_str, Ident, TypeParamBound, Token};
use syn::punctuated::Punctuated;

/// unified metadata model for generating trait detection logic.
/// Handles both std traits (global) and user traits (local).
pub struct TraitModel {
    /// The main traits to detect (e.g. Fn(u8) + Send).
    pub bounds: Punctuated<TypeParamBound, Token![+]>,

    /// true: global Detect<T> (std lib); false: local __Detect<T>.
    pub is_global_std: bool,
    pub alias: Option<Ident>,
}

impl TraitModel {
    /// Parse DSL string: "path::to::Trait<G> + Bounds" or "Bound as Alias"
    pub fn parse_desc(desc: &str, is_global_std: bool) -> Self {
        let parts: Vec<&str> = desc.split(" as ").collect();

        let (trait_part, alias) = if parts.len() > 1 {
            // Found alias
            let alias_str = parts[1].trim();
            // The trait part might be "Trait + Bound"
            (parts[0].trim(), Some(format_ident!("{}", alias_str)))
        } else {
            (desc.trim(), None)
        };

        // Parse trait string as Punctuated TypeParamBound.
        // Fallback to TypeTraitObject parsing if single bound parse fails (e.g. for "Trait + Send").
        let bounds = if trait_part.is_empty() {
             Punctuated::new()
        } else {
             match parse_str::<TypeParamBound>(trait_part) {
                 Ok(b) => {
                     let mut p = Punctuated::new();
                     p.push(b);
                     p
                 },
                 Err(_) => {
                     let dyn_str = format!("dyn {}", trait_part);
                     match parse_str::<syn::TypeTraitObject>(&dyn_str) {
                         Ok(obj) => obj.bounds,
                         Err(e) => panic!("Failed to parse bounds '{}': {}", trait_part, e),
                     }
                 }
             }
        };

        TraitModel {
            bounds,
            is_global_std,
            alias,
        }
    }


    /// Generate core detection logic (Fallback trait + Impl).
    /// `cfg` is applied to each generated item for proper conditional compilation.
    pub fn expand_detection(&self, cfg: &TokenStream) -> TokenStream {
        let name = self.name();
        let const_name = format_ident!("IS_{}", to_screaming_snake_case(&name.to_string()));
        let fallback_trait = format_ident!("{}Fallback", name);

        let bounds = &self.bounds;

        // Global vs Local wrapper selection
        let (wrapper, wrapper_def) = if self.is_global_std {
             (quote! { ::tola_caps::detect::Detect<T> }, quote! {})
        } else {
             (quote! { __Detect<T> }, quote! {
                 #cfg
                 #[doc(hidden)]
                 pub struct __Detect<T: ?Sized>(core::marker::PhantomData<T>);
             })
        };

        // Fallback: return false by default
        let fallback_impl = quote! {
            #cfg
            #[doc(hidden)]
            pub trait #fallback_trait {
                const #const_name: bool = false;
            }
            #cfg
            impl<T: ?Sized> #fallback_trait for #wrapper {}
        };

        let success_impl = if bounds.is_empty() {
             quote! {}
        } else {
             quote! {
                #cfg
                impl<T: #bounds> #wrapper {
                    pub const #const_name: bool = true;
                }
             }
        };

        quote! {
            #wrapper_def
            #fallback_impl
            #success_impl
        }
    }

    pub fn name(&self) -> Ident {
        if let Some(alias) = &self.alias {
            return alias.clone();
        }

        // Try to derive name from the first Trait bound
        for bound in &self.bounds {
            if let TypeParamBound::Trait(tb) = bound {
                if let Some(segment) = tb.path.segments.last() {
                    return segment.ident.clone();
                }
            }
        }

        // For std traits we assume valid traits.
        panic!("Cannot infer name for trait model without alias. Bounds: {:?}",
               self.bounds.to_token_stream().to_string());
    }
}

fn to_screaming_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 { result.push('_'); }
        result.push(c.to_ascii_uppercase());
    }
    result
}


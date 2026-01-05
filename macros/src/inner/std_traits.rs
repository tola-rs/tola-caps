//! Proc macro to generate std trait detection infrastructure.
//!
//! Single source of truth for all std traits. To add a trait, just add it here!

use proc_macro2::TokenStream;
use quote::{quote, format_ident};

// Std trait definitions using string DSL: "path::to::Trait<G> + Bounds"
// See `macros/src/common/trait_model.rs` for parsing logic.

/// Core library traits (always available).
pub const CORE_TRAITS: &[&str] = &[
    // ==================== Core Marker Traits ====================
    "core::clone::Clone",
    "core::marker::Copy",
    "core::marker::Send + ?Sized",
    "core::marker::Sync + ?Sized",
    "core::marker::Sized",
    "core::marker::Unpin + ?Sized",


    // ==================== Panic/Unwind Traits ====================
    "core::panic::RefUnwindSafe + ?Sized",
    "core::panic::UnwindSafe + ?Sized",

    // ==================== Common Traits ====================
    "core::default::Default",
    // Drop and Debug were duplicates here, removing them.
    // Any is removed to avoid static lifetime issues in detection.

    // ==================== Comparison Traits ====================
    "core::cmp::Eq + ?Sized",
    "core::cmp::PartialEq + ?Sized",
    "core::cmp::Ord + ?Sized",
    "core::cmp::PartialOrd + ?Sized",
    "core::hash::Hash + ?Sized",

    // ==================== Formatting Traits ====================
    "core::fmt::Debug + ?Sized",
    "core::fmt::Display + ?Sized",
    "core::fmt::Binary + ?Sized",
    "core::fmt::LowerExp + ?Sized",
    "core::fmt::LowerHex + ?Sized",
    "core::fmt::Octal + ?Sized",
    "core::fmt::Pointer + ?Sized",
    "core::fmt::UpperExp + ?Sized",
    "core::fmt::UpperHex + ?Sized",
    "core::fmt::Write as FmtWrite", // Alias to avoid conflict with std::io::Write
    "core::ops::Drop + ?Sized",

    // ==================== Iterator Traits ====================
    "core::iter::Iterator",
    "core::iter::IntoIterator",
    "core::iter::ExactSizeIterator",
    "core::iter::DoubleEndedIterator",
    "core::iter::FusedIterator",

    // ==================== Deref Traits ====================
    "core::ops::Deref + ?Sized",
    "core::ops::DerefMut + ?Sized",

    // ==================== Async ====================
    "core::future::Future + ?Sized",

    // ==================== Operator Traits ====================
    "core::ops::Add",
    "core::ops::Sub",
    "core::ops::Mul",
    "core::ops::Div",
    "core::ops::Rem",
    "core::ops::Neg",
    "core::ops::Not",
    "core::ops::BitAnd",
    "core::ops::BitOr",
    "core::ops::BitXor",
    "core::ops::Shl",
    "core::ops::Shr",

    // ==================== Assign Operator Traits ====================
    "core::ops::AddAssign",
    "core::ops::SubAssign",
    "core::ops::MulAssign",
    "core::ops::DivAssign",
    "core::ops::RemAssign",
    "core::ops::BitAndAssign",
    "core::ops::BitOrAssign",
    "core::ops::BitXorAssign",
    "core::ops::ShlAssign",
    "core::ops::ShrAssign",

    // ==================== Generic Traits (Markers Only) ====================
    // Traits with generic parameters (e.g. From<T>).
    // We generate markers (IsFrom) but not automatic `Detect::IS_FROM` constants,
    // as they require specific type parameters.
];

pub const GENERIC_TRAITS: &[&str] = &[
    "core::convert::From",
    "core::convert::Into",
    "core::any::Any + 'static + ?Sized", // Moved from CORE to avoid detection issues
    "core::convert::TryFrom",
    "core::convert::TryInto",
    "core::convert::AsRef",
    "core::convert::AsMut",
    "core::borrow::Borrow",
    "core::borrow::BorrowMut",
    "core::iter::FromIterator",
    "core::iter::Extend",
    "core::ops::Index",
    "core::ops::IndexMut",
];

pub const CORE_TRAITS_2: &[&str] = &[
    "core::str::FromStr",
];

/// Alloc library traits (requires "alloc" feature).
pub const ALLOC_TRAITS: &[&str] = &[
    "alloc::borrow::ToOwned + ?Sized",
    "alloc::string::ToString + ?Sized",
];

/// Standard library traits (requires "std" feature).
pub const STD_LIB_TRAITS: &[&str] = &[
    "std::error::Error + ?Sized",
    "std::io::Read",
    "std::io::Write as IoWrite", // Alias to avoid conflict with core::fmt::Write
    "std::io::Seek",
    "std::io::BufRead",
];



/// Convert nibble value (0-15) to Ident (X0-XF)
fn nibble_to_ident(nibble: u8) -> syn::Ident {
    format_ident!("X{:X}", nibble & 0x0F)
}

#[allow(dead_code)]
/// Generate hash stream type from trait name using BLAKE3.
/// Uses first 4 bytes (8 nibbles) for unique identification.
fn generate_hash_stream(name: &str) -> TokenStream {
    let nibbles = get_nibbles(name);
    build_hash_stream(&nibbles)
}

#[allow(dead_code)]
/// Build hash stream type from nibbles: Cons<X0, Cons<X1, ... ConstStream<XN>>>
fn build_hash_stream(nibbles: &[u8]) -> TokenStream {
    if nibbles.is_empty() {
        quote! { ConstStream<X0> }
    } else if nibbles.len() == 1 {
        let nib = nibble_to_ident(nibbles[0]);
        quote! { ConstStream<#nib> }
    } else {
        let head = nibble_to_ident(nibbles[0]);
        let tail = build_hash_stream(&nibbles[1..]);
        quote! { Cons<#head, #tail> }
    }
}

/// Generate the IS_XXX constant name from trait name
fn const_name(name: &str) -> syn::Ident {
    // Clone -> IS_CLONE, PartialEq -> IS_PARTIAL_EQ
    let upper = name.chars().enumerate().map(|(i, c)| {
        if c.is_uppercase() && i > 0 {
            format!("_{}", c)
        } else {
            c.to_uppercase().to_string()
        }
    }).collect::<String>();
    format_ident!("IS_{}", upper)
}

/// Generate marker struct name: Clone -> IsClone
fn marker_name(name: &str) -> syn::Ident {
    format_ident!("Is{}", name)
}

/// Generate fallback trait name: Clone -> CloneFallback
fn fallback_name(name: &str) -> syn::Ident {
    format_ident!("{}Fallback", name)
}

/// Generate select trait name: Clone -> SelectClone
fn select_name(name: &str) -> syn::Ident {
    format_ident!("Select{}", name)
}


/// FNV-1a 64-bit hash
fn fnv1a_64(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Generate 16-nibble hash stream for routing
fn build_hash_stream_64(hash: u64) -> TokenStream {
    let mut stream = quote! { ::tola_caps::ConstStream<::tola_caps::X0> };
    for i in 0..16 {
        let shift = i * 4;
        let nibble = ((hash >> shift) & 0xF) as u8;
        // Reuse module-level nibble_to_ident (returns syn::Ident which works in quote)
        let nib_ident = nibble_to_ident(nibble);
        if i == 0 {
             stream = quote! { ::tola_caps::ConstStream<::tola_caps::#nib_ident> };
        } else {
             stream = quote! { ::tola_caps::Cons<::tola_caps::#nib_ident, #stream> };
        }
    }
    stream
}

use crate::common::TraitModel;

/// Generate AutoCaps impl for#[cap] attribute (used in auto_caps.rs)
pub fn expand_cap_on_type_impl(ty: &syn::Ident, generics: &syn::Generics) -> TokenStream {
    let mut consts = Vec::new();
    let mut insert_chain_parts = Vec::new();

    // Helper to process a list
    fn process_list(
        list: &[&str],
        cfg: TokenStream,
        ty: &syn::Ident,
        generics: &syn::Generics,
        consts: &mut Vec<TokenStream>,
        chain: &mut Vec<(syn::Ident, syn::Ident, TokenStream)>,
        skip_consts: bool
    ) {
        let (_impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

        for desc in list {
            let model = TraitModel::parse_desc(desc, true);
            let name = model.name().to_string();
            let const_nm = const_name(&name);
            let marker = marker_name(&name);
            let fallback = fallback_name(&name);

            if !skip_consts {
                consts.push(quote! {
                    #cfg
                    #[allow(unused_imports)]
                    const #const_nm: bool = {
                        use ::tola_caps::detect::#fallback;
                        ::tola_caps::detect::Detect::<#ty #ty_generics>::#const_nm
                    };
                });
            }

            // Always add to chain if it's a marker we want to track
            chain.push((marker, const_nm, cfg.clone()));
        }
    }

    // Process trait lists
    process_list(CORE_TRAITS, quote!{}, ty, generics, &mut consts, &mut insert_chain_parts, false);
    process_list(CORE_TRAITS_2, quote!{}, ty, generics, &mut consts, &mut insert_chain_parts, false);
    process_list(GENERIC_TRAITS, quote!{}, ty, generics, &mut consts, &mut insert_chain_parts, true);
    process_list(ALLOC_TRAITS, quote!{ #[cfg(feature = "alloc")] }, ty, generics, &mut consts, &mut insert_chain_parts, false);
    process_list(STD_LIB_TRAITS, quote!{ #[cfg(feature = "std")] }, ty, generics, &mut consts, &mut insert_chain_parts, false);

    // Build the InsertIf chain
    // NOTE: For stability, the AutoCapSet Trie only includes unconditional (CORE) traits.
    // Feature-gated traits (alloc/std) are excluded from the structural type to avoid
    // conditional type definition complexities, but their IS_XXX constants work for direct checks.

    let mut insert_chain = quote! { ::tola_caps::trie::Empty };
    let (_, ty_generics, _) = generics.split_for_impl();

    // Only process CORE traits for the Trie chain
    let core_lists = [CORE_TRAITS, CORE_TRAITS_2, GENERIC_TRAITS];
    for list in core_lists {
         for desc in list {
            let model = TraitModel::parse_desc(desc, true);
            let name = model.name().to_string();
            let marker = marker_name(&name);
            let const_nm = const_name(&name);

            insert_chain = quote! {
                <#insert_chain as ::tola_caps::detect::InsertIf<
                    ::tola_caps::detect::#marker,
                    { <#ty #ty_generics as ::tola_caps::detect::AutoCaps>::#const_nm },
                >>::Out
            };
         }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // AutoCaps implementation - always generated
    let autocaps_impl = quote! {
         impl #impl_generics ::tola_caps::detect::AutoCaps for #ty #ty_generics #where_clause {
            #(#consts)*
        }
    };

    // AutoCapSet can only be implemented for concrete types (no generics) on stable Rust.
    // Generic types would require generic const expressions (T's capabilities in const context).
    let autocapset_impl = if generics.params.is_empty() {
        // Concrete type: generate full AutoCapSet with InsertIf chain
        quote! {
            impl #impl_generics ::tola_caps::detect::AutoCapSet for #ty #ty_generics #where_clause {
                type Out = #insert_chain;
            }
        }
    } else {
        // Generic type: use Empty trie (capabilities unknown at compile time)
        quote! {
            impl #impl_generics ::tola_caps::detect::AutoCapSet for #ty #ty_generics #where_clause {
                type Out = ::tola_caps::trie::Empty;
            }
        }
    };

    quote! {
         #autocaps_impl
         #autocapset_impl
    }
}

/// Generate all std trait detection code using TraitModel
pub fn expand_std_traits() -> TokenStream {
    let mut headers = Vec::new();       // Markers, Fallbacks, Selects, etc.
    let mut marker_idents = Vec::new(); // For with_all_std_traits!

    // Helper to process lists
    fn process_list_std(
        list: &[&str],
        cfg: TokenStream,
        headers: &mut Vec<TokenStream>,
        marker_idents: &mut Vec<TokenStream>,
        skip_detection: bool // If true, only generate Marker
    ) {
        for desc in list {
            let model = TraitModel::parse_desc(desc, true);
            let name = model.name().to_string();
            let marker = marker_name(&name);
            let select = select_name(&name);

            // 1. Generate Capability Marker (unique Identity)
            let hash_64 = fnv1a_64(&name);
            let stream_type = build_hash_stream_64(hash_64);
            // Generate tiered IList Identity
            let identity_type = super::super::user::capability::generate_identity_tiered(&name);

            headers.push(quote! {
                #cfg
                #[doc = concat!("Capability marker for `", #name, "` trait detection.")]
                pub type #marker = ::tola_caps::primitives::identity::Marker<#stream_type>;

                #cfg
                impl Capability for #marker {
                    type Stream = #stream_type;
                    type Identity = #identity_type;

                    type At<D: ::tola_caps::Peano> = <<Self::Stream as GetTail<D>>::Out as HashStream>::Head
                    where Self::Stream: GetTail<D>;
                }
            });

            // 2. Add to marker list
            marker_idents.push(quote! {
                #cfg
                crate::detect::#marker
            });

            if !skip_detection {
                // 3. Generate Detection Logic (Fallback + Const) via TraitModel
                // cfg is now passed to expand_detection to apply to each item
                let detection_code = model.expand_detection(&cfg);
                headers.push(detection_code);

                // 4. Generate Select Trait
                headers.push(quote! {
                    #cfg
                    #[doc = concat!("Select between two types based on `", #name, "` capability.")]
                    pub trait #select<Then, Else> {
                        type Out;
                    }

                    #cfg
                    impl<S, Then, Else> #select<Then, Else> for S
                    where
                        S: Evaluate<#marker>,
                        <S as Evaluate<#marker>>::Out: Bool,
                    {
                        type Out = <<S as Evaluate<#marker>>::Out as Bool>::If<Then, Else>;
                    }
                });
            }
        }
    }

    // Process all lists to generate content
    process_list_std(CORE_TRAITS, quote!{}, &mut headers, &mut marker_idents, false);
    process_list_std(CORE_TRAITS_2, quote!{}, &mut headers, &mut marker_idents, false);
    process_list_std(GENERIC_TRAITS, quote!{}, &mut headers, &mut marker_idents, true);
    process_list_std(ALLOC_TRAITS, quote!{ #[cfg(feature = "alloc")] }, &mut headers, &mut marker_idents, false);
    process_list_std(STD_LIB_TRAITS, quote!{ #[cfg(feature = "std")] }, &mut headers, &mut marker_idents, false);

    // Generate AutoCaps trait body with default constants
    let mut autocaps_defaults = Vec::new();
    fn process_list_defaults(
        list: &[&str],
        cfg: TokenStream,
        defaults: &mut Vec<TokenStream>,
        skip_detection: bool
    ) {
        if skip_detection { return; }
        for desc in list {
            let model = TraitModel::parse_desc(desc, true);
            let name = model.name().to_string();
            let const_nm = const_name(&name);
            defaults.push(quote! {
                #cfg
                const #const_nm: bool = false;
            });
        }
    }
    process_list_defaults(CORE_TRAITS, quote!{}, &mut autocaps_defaults, false);
    process_list_defaults(CORE_TRAITS_2, quote!{}, &mut autocaps_defaults, false);
    // Even if detection is skipped, we need default constants (IS_XXX = false)
    // because AutoCapSet Trie structure references them.
    process_list_defaults(GENERIC_TRAITS, quote!{}, &mut autocaps_defaults, false);
    process_list_defaults(ALLOC_TRAITS, quote!{ #[cfg(feature = "alloc")] }, &mut autocaps_defaults, false);
    process_list_defaults(STD_LIB_TRAITS, quote!{ #[cfg(feature = "std")] }, &mut autocaps_defaults, false);

    quote! {
        // Auto-generated std trait detection infrastructure.
        // Defines markers, fallback traits, and detection logic.

        use core::marker::PhantomData;
        use crate::primitives::{Cons, ConstStream, GetTail, HashStream, Bool, Peano};
        use crate::primitives::nibble::*;
        use crate::trie::{Capability, Evaluate};

        #[doc(hidden)]
        pub struct Detect<T: ?Sized>(PhantomData<*const T>);

        // All generated code blocks
        #(#headers)*

        /// Internal trait for types with automatically detected capabilities.
        #[doc(hidden)]
        pub trait AutoCaps {
             #(#autocaps_defaults)*
        }
    }
}

/// Generate impl AutoCaps + AutoCapSet for a type (proc macro version)
/// Generate impl AutoCaps + AutoCapSet for a type (proc macro version)
pub fn expand_impl_auto_caps(input: proc_macro2::TokenStream) -> TokenStream {
    let ty: syn::Type = match syn::parse2(input) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error(),
    };

    let mut autocaps_impl_consts = Vec::new();
    let mut insert_chain_parts = Vec::new();

    // Helper to process lists
    fn process_list_impl(
        list: &[&str],
        cfg: TokenStream,
        ty: &syn::Type,
        consts: &mut Vec<TokenStream>,
        chain: &mut Vec<(syn::Ident, syn::Ident, TokenStream)>,
        skip_consts: bool
    ) {
        for desc in list {
            let model = TraitModel::parse_desc(desc, true);
            let name = model.name().to_string();
            let const_nm = const_name(&name);
            let marker = marker_name(&name);
            let fallback = fallback_name(&name);

            if !skip_consts {
                consts.push(quote! {
                    #cfg
                    #[allow(unused_imports)]
                    const #const_nm: bool = {
                        use tola_caps::detect::#fallback;
                        tola_caps::detect::Detect::<#ty>::#const_nm
                    };
                });
            }

            // Always add to chain candidates
            chain.push((marker, const_nm, cfg.clone()));
        }
    }

    // Process all lists
    process_list_impl(CORE_TRAITS, quote!{}, &ty, &mut autocaps_impl_consts, &mut insert_chain_parts, false);
    process_list_impl(CORE_TRAITS_2, quote!{}, &ty, &mut autocaps_impl_consts, &mut insert_chain_parts, false);
    process_list_impl(GENERIC_TRAITS, quote!{}, &ty, &mut autocaps_impl_consts, &mut insert_chain_parts, true);
    process_list_impl(ALLOC_TRAITS, quote!{ #[cfg(feature = "alloc")] }, &ty, &mut autocaps_impl_consts, &mut insert_chain_parts, false);
    process_list_impl(STD_LIB_TRAITS, quote!{ #[cfg(feature = "std")] }, &ty, &mut autocaps_impl_consts, &mut insert_chain_parts, false);

    // Build the InsertIf chain
    // NOTE: We only include traits in the Trie (AutoCapSet) that are unconditionally available (CORE),
    // to avoid conditional compilation complexities in structural types.
    let mut insert_chain = quote! { tola_caps::trie::Empty };

    // Re-build chain using ONLY unconditional traits (empty cfg)
    for (marker, const_nm, cfg) in insert_chain_parts {
        if !cfg.is_empty() { continue; }

        insert_chain = quote! {
            <#insert_chain as tola_caps::detect::InsertIf<
                tola_caps::detect::#marker,
                { <#ty as tola_caps::detect::AutoCaps>::#const_nm },
            >>::Out
        };
    }

    quote! {
        impl tola_caps::detect::AutoCaps for #ty {
            #(#autocaps_impl_consts)*
        }

        impl tola_caps::detect::AutoCapSet for #ty {
            type Out = #insert_chain;
        }
    }
}

/// In-memory Trie structure for generating structural types
struct TrieNode {
    children: Vec<Option<Box<TrieNode>>>, // 16 slots
    traits: Vec<String>, // Trait names stored at this node (if it's a leaf/collision)
}

impl TrieNode {
    fn new() -> Self {
        TrieNode { children: (0..16).map(|_| None).collect(), traits: Vec::new() }
    }

    fn insert(&mut self, nibbles: &[u8], trait_name: String) {
        if nibbles.is_empty() {
            self.traits.push(trait_name);
            return;
        }
        let idx = nibbles[0] as usize;
        if self.children[idx].is_none() {
            self.children[idx] = Some(Box::new(TrieNode::new()));
        }
        self.children[idx].as_mut().unwrap().insert(&nibbles[1..], trait_name);
    }
}

/// Helper to generate the structural generic type for a TrieNode.
///
/// Generates a nested Node16 structure where:
/// - Internal nodes are Node16<...>
/// - Leaf nodes use Bool::If<Leaf<M>, Empty> - NO InsertIf to avoid deep recursion
/// - Single trait per leaf slot (no collision handling - hash is unique enough)
fn generate_trie_structure(node: &TrieNode) -> TokenStream {
    // Check if this is a pure leaf (no children, only traits)
    let has_children = node.children.iter().any(|c| c.is_some());
    let has_traits = !node.traits.is_empty();

    // Safety check: With fixed-depth hashing (4 nibbles), a node CANNOT have both
    // children (intermediate node) and traits (leaf node).
    if has_children && has_traits {
        panic!("Logic Error: Node has both children and traits! This is impossible with fixed-depth hashing.");
    }

    if !has_children && !has_traits {
        // Empty node
        return quote! { $crate::trie::Empty };
    }

    if has_children {
        // Internal node: generate Node16<...>
        let child_types = node.children.iter().map(|child| {
            match child {
                Some(c) => generate_trie_structure(c),
                None => quote! { $crate::trie::Empty },
            }
        });

        quote! { $crate::trie::Node16< #(#child_types),* > }
    } else {
        // Leaf node: generate conditional Leaf for each trait
        // Use Bool::If to select between Leaf<M> and previous result
        // <<() as SelectBool<{T::IS_XXX}>>::Out as Bool>::If<Leaf<M>, Prev>
        let mut result = quote! { $crate::trie::Empty };

        // Process traits in reverse so first trait is outermost
        for name in node.traits.iter().rev() {
            let marker = marker_name(name);
            let const_nm = const_name(name);

            // Use Bool::If to select between Leaf<M> and previous result
            result = quote! {
                <<() as $crate::primitives::SelectBool<
                    { <$T as $crate::detect::AutoCaps>::#const_nm }
                >>::Out as $crate::primitives::Bool>::If<
                    $crate::trie::Leaf<$crate::detect::#marker>,
                    #result
                >
            };
        }

        result
    }
}

/// Get hash nibbles for a trait name.
/// Uses FNV-1a hash, 4 nibbles depth for 万级 traits support.
fn get_nibbles(name: &str) -> Vec<u8> {
    let hash = {
        let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
        for b in name.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3); // FNV prime
        }
        h
    };
    // Take first 4 nibbles (2 bytes) for 16^4 = 65536 slots capacity
    hash.to_be_bytes()
        .iter()
        .take(2)
        .flat_map(|b| vec![(b >> 4) & 0x0F, b & 0x0F])
        .collect()
    }

#[allow(dead_code)]
/// Calculate FNV-1a 128-bit hash for ID
fn fnv1a_128(data: &str) -> u128 {
    let mut hash: u128 = 0x6c62272e07bb014262b821756295c58d;
    for b in data.as_bytes() {
        hash ^= *b as u128;
        hash = hash.wrapping_mul(0x1000000000000000000013b);
    }
    hash
}

/// Generate impl_auto_caps! macro.
///
/// Implements `AutoCaps` (constants) and `AutoCapSet` (Trie) for a type.
/// The Trie is a 4-layer Node16 structure sorting traits by hash nibbles.
pub fn expand_impl_auto_caps_macro() -> TokenStream {
    let mut autocaps_impl_consts = Vec::new();
    let mut trie_root = TrieNode::new();

    fn process_list_trie(
        list: &[&str],
        cfg: TokenStream,
        consts: &mut Vec<TokenStream>,
        trie: &mut TrieNode
    ) {
         for desc in list {
            let model = TraitModel::parse_desc(desc, true);
            let name = model.name().to_string();
            let fallback = fallback_name(&name);
            let const_nm = const_name(&name);

            consts.push(quote! {
                #cfg
                #[allow(unused_imports)]
                const #const_nm: bool = {
                    use $crate::detect::#fallback;
                    $crate::detect::Detect::<$T>::#const_nm
                };
            });

            // Insert into Trie if unconditional (no cfg)
            if cfg.is_empty() {
                 let nibbles = get_nibbles(&name);
                 trie.insert(&nibbles, name.clone());
            }
         }
    }

    // Process lists. EXCLUDE GENERIC_TRAITS from consts (they have no Detect const).
    process_list_trie(CORE_TRAITS, quote!{}, &mut autocaps_impl_consts, &mut trie_root);
    process_list_trie(CORE_TRAITS_2, quote!{}, &mut autocaps_impl_consts, &mut trie_root);
    // Skip GENERIC_TRAITS for both consts (no IS_XXX) and Trie (no capability to check without generic)

    process_list_trie(ALLOC_TRAITS, quote!{ #[cfg(feature = "alloc")] }, &mut autocaps_impl_consts, &mut trie_root);
    process_list_trie(STD_LIB_TRAITS, quote!{ #[cfg(feature = "std")] }, &mut autocaps_impl_consts, &mut trie_root);

    // Generate the layered Node16 Trie type
    let trie_type = generate_trie_structure(&trie_root);

    quote! {
        /// Implement `AutoCaps` and `AutoCapSet` for a concrete type.
        ///
        /// - `AutoCaps`: Provides IS_XXX constants for ALL std traits (for caps_check!)
        /// - `AutoCapSet::Out`: Layered Node16 Trie with O(log₁₆ N) depth
        ///
        /// Supports 10,000+ traits without compiler overflow.
        macro_rules! impl_auto_caps {
            // Concrete type: both AutoCaps and AutoCapSet
            ($T:ty) => {
                impl $crate::detect::AutoCaps for $T {
                    #(#autocaps_impl_consts)*
                }

                impl $crate::detect::AutoCapSet for $T {
                    type Out = #trie_type;
                }
            };
            // Generic type: AutoCaps only (AutoCapSet needs special handling)
            ( @generic_no_set [ $($G:tt)* ] $T:ty ) => {
                impl< $($G)* > $crate::detect::AutoCaps for $T {
                    #(#autocaps_impl_consts)*
                }
            };
            // Generic type with Trie: for containers like Vec<T>
            ( @generic [ $($G:tt)* ] $T:ty ) => {
                impl< $($G)* > $crate::detect::AutoCaps for $T {
                    #(#autocaps_impl_consts)*
                }

                impl< $($G)* > $crate::detect::AutoCapSet for $T {
                    type Out = #trie_type;
                }
            };
        }
        pub(crate) use impl_auto_caps;
    }
}

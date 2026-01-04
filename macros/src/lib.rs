//! Procedural macros for tola-caps capability system
//!
//! Provides:
//! - `#[derive(Capability)]` - auto-generate hash stream from type name
//! - `#[requires]` - require capabilities with boolean logic
//! - `#[conflicts]` - require capabilities to be absent
//! - `caps!` - build capability sets with duplicate checking

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use blake3;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Ident, ItemFn, LitStr, Token, Type,
};

// =============================================================================
// #[requires] attribute macro
// =============================================================================

/// Parsed form of `#[requires(C: Cap1, Cap2<T>, ...)]`
struct RequiresAttr {
    /// The type parameter name (e.g., `C`)
    target: Ident,
    /// The capability types
    caps: Punctuated<Type, Token![,]>,
}

impl Parse for RequiresAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target: Ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let caps = Punctuated::parse_terminated(input)?;
        Ok(RequiresAttr { target, caps })
    }
}

/// Attribute macro to add capability requirements to a function.
///
/// # Usage
///
/// ```ignore
/// #[requires(C: LinksCheckedCap, SvgOptimizedCap)]
/// fn my_transform<C>(doc: Doc<Indexed, C>) { ... }
/// ```
///
/// # How It Works
///
/// The macro generates bounds using the `Evaluate` trait:
/// - Single cap: `C: Evaluate<Has<Cap>, Out = True>`
/// - Multiple caps: `C: Evaluate<all![Has<A>, Has<B>], Out = True>`
///
/// All capabilities listed are required (AND semantics).
#[deprecated(since = "0.2.0", note = "Use `#[caps(requires = ...)]` instead")]
#[proc_macro_attribute]
pub fn requires(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as RequiresAttr);
    let func = parse_macro_input!(item as ItemFn);

    let type_param_ident = &attr.target;
    let caps: Vec<&Type> = attr.caps.iter().collect();

    if caps.is_empty() {
        return func.into_token_stream().into();
    }

    let func_name = &func.sig.ident;
    let func_vis = &func.vis;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;
    let func_generics = &func.sig.generics;
    let func_body = &func.block;
    let func_attrs = &func.attrs;

    let generic_params = &func_generics.params;
    let where_clause = &func_generics.where_clause;

    // Build the Evaluate bound
    let eval_bound = if caps.len() == 1 {
        let cap = caps[0];
        quote! {
            ::tola_caps::Evaluate<::tola_caps::Has<#cap>, Out = ::tola_caps::Present>
        }
    } else {
        // Multiple caps: use All<HList>
        let has_caps = caps.iter().map(|cap| {
            quote! { ::tola_caps::Has<#cap> }
        });
        quote! {
            ::tola_caps::Evaluate<
                ::tola_caps::All<::tola_caps::hlist![#(#has_caps),*]>,
                Out = ::tola_caps::Present
            >
        }
    };

    // Build the where clause
    let new_where = if let Some(wc) = where_clause {
        let existing_predicates = &wc.predicates;
        quote! {
            where #existing_predicates, #type_param_ident: #eval_bound
        }
    } else {
        quote! {
            where #type_param_ident: #eval_bound
        }
    };

    let output = quote! {
        #(#func_attrs)*
        #func_vis fn #func_name<#generic_params>(#func_inputs) #func_output #new_where
        #func_body
    };

    output.into()
}

/// Attribute macro to require capabilities to be ABSENT.
///
/// Use this to enforce that a transform runs BEFORE certain capabilities are added.
///
/// # Usage
///
/// ```ignore
/// // This transform must run BEFORE SvgOptimizedCap is added
/// #[requires_not(C: SvgOptimizedCap)]
/// fn raw_svg_transform<C>(doc: Doc<Indexed, C>) { ... }
/// ```
///
/// # How It Works
///
/// The macro generates bounds using the `Evaluate` trait with `Not`:
/// - Single cap: `C: Evaluate<Not<Has<Cap>>, Out = True>`
/// - Multiple caps: `C: Evaluate<all![Not<Has<A>>, Not<Has<B>>], Out = True>`
#[deprecated(since = "0.2.0", note = "Use `#[caps(conflicts = ...)]` instead")]
#[proc_macro_attribute]
pub fn requires_not(attr: TokenStream, item: TokenStream) -> TokenStream {
    conflicts_impl(attr, item)
}

/// Attribute macro to declare conflicting capabilities.
///
/// Alias for `#[requires_not]` with clearer semantics.
///
/// # Usage
///
/// ```ignore
/// #[conflicts(C: SvgOptimizedCap)]
/// fn must_run_before_svg<C>() { ... }
/// ```
#[deprecated(since = "0.2.0", note = "Use `#[caps(conflicts = ...)]` instead")]
#[proc_macro_attribute]
pub fn conflicts(attr: TokenStream, item: TokenStream) -> TokenStream {
    conflicts_impl(attr, item)
}

fn conflicts_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as RequiresAttr);
    let func = parse_macro_input!(item as ItemFn);

    let type_param_ident = &attr.target;
    let caps: Vec<&Type> = attr.caps.iter().collect();

    if caps.is_empty() {
        return func.into_token_stream().into();
    }

    let func_name = &func.sig.ident;
    let func_vis = &func.vis;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;
    let func_generics = &func.sig.generics;
    let func_body = &func.block;
    let func_attrs = &func.attrs;

    let generic_params = &func_generics.params;
    let where_clause = &func_generics.where_clause;

    // Build the Evaluate bound with Not
    let eval_bound = if caps.len() == 1 {
        let cap = caps[0];
        quote! {
            ::tola_caps::Evaluate<::tola_caps::Not<::tola_caps::Has<#cap>>, Out = ::tola_caps::Present>
        }
    } else {
        // Multiple caps: use All<HList> with Not<Has<...>> for each
        let not_has_caps = caps.iter().map(|cap| {
            quote! { ::tola_caps::Not<::tola_caps::Has<#cap>> }
        });
        quote! {
            ::tola_caps::Evaluate<
                ::tola_caps::All<::tola_caps::hlist![#(#not_has_caps),*]>,
                Out = ::tola_caps::Present
            >
        }
    };

    // Build the where clause
    let new_where = if let Some(wc) = where_clause {
        let existing_predicates = &wc.predicates;
        quote! {
            where #existing_predicates, #type_param_ident: #eval_bound
        }
    } else {
        quote! {
            where #type_param_ident: #eval_bound
        }
    };

    let output = quote! {
        #(#func_attrs)*
        #func_vis fn #func_name<#generic_params>(#func_inputs) #func_output #new_where
        #func_body
    };

    output.into()
}



// =============================================================================
// Unified #[caps] macro
// =============================================================================

/// Unified capability constraint macro with boolean expression support and auto-generic injection.
///
/// # Usage
///
/// ```ignore
/// #[caps_bound(requires = CanRead & (CanWrite | CanAdmin), conflicts = CanGuest)]
/// fn secure_op(doc: Doc) { ... }
/// ```
///
/// This expands to:
///
/// ```ignore
/// fn secure_op<C>(doc: Doc<C>)
/// where
///     C: Evaluate<And<Has<CanRead>, Or<Has<CanWrite>, Has<CanAdmin>>>, Out = True>,
///     C: Evaluate<Not<Has<CanGuest>>, Out = True>
/// { ... }
/// ```
///
/// # Features
///
/// 1. **Boolean Logic**: Supports `&` (AND), `|` (OR), `!` (NOT), `()` (grouping).
/// 2. **Auto-Generics**: Automatically injects `<C>` if `Doc` or other carrier types are present in args.
/// 3. **Supports**: Functions, Structs, Enums, and Impl blocks.
#[proc_macro_attribute]
pub fn caps_bound(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as CapsArgs);

    // Try to parse as different item types
    let item_clone = item.clone();
    if let Ok(func) = syn::parse::<ItemFn>(item_clone.clone()) {
        return expand_caps_fn(args, func);
    }

    let item_clone = item.clone();
    if let Ok(item_struct) = syn::parse::<syn::ItemStruct>(item_clone.clone()) {
        return expand_caps_struct(args, item_struct);
    }

    let item_clone = item.clone();
    if let Ok(item_enum) = syn::parse::<syn::ItemEnum>(item_clone.clone()) {
        return expand_caps_enum(args, item_enum);
    }

    let item_clone = item.clone();
    if let Ok(item_impl) = syn::parse::<syn::ItemImpl>(item_clone) {
        return expand_caps_impl(args, item_impl);
    }

    // Fallback error
    syn::Error::new(proc_macro2::Span::call_site(), "caps_bound supports fn, struct, enum, or impl")
        .to_compile_error()
        .into()
}

/// Check for duplicate capabilities in the list
#[proc_macro]
pub fn caps(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CapsInput);
    let types: Vec<_> = input.types.into_iter().collect();

    // Check for duplicates
    if let Err(err) = check_duplicates(&types) {
        return err.to_compile_error().into();
    }

    let output = build_capset(&types);
    output.into()
}

mod caps_impl {
    use super::*;

    use syn::{parse::{Parse, ParseStream}, Token, Type};

    // --- AST for Boolean Expressions ---

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

    // Recursive Descent Parser: Or -> And -> Unary -> Primary

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

    // --- Attributes Parsing ---

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
                // 1. Check for Named Arguments: key = value
                if input.peek(Ident) && input.peek2(Token![=]) {
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
                         // support transparent = true/false (legacy style)
                         if input.peek(syn::LitBool) {
                            let val: syn::LitBool = input.parse()?;
                            transparent = val.value;
                         } else {
                             // allow bare 'transparent' if used as flag elsewhere?
                             // But here we are in key=value block.
                             // Assuming transparent=true if only transparent is key?
                             // No, if key=value expected, value must be there.
                             // But wait, my previous parser handled `transparent` in positional section (flag).
                             // Here it is key=value.
                             // I will stick to what was there.
                             let val: syn::LitBool = input.parse()?;
                             transparent = val.value;
                         }
                    } else if key == "target" {
                        target = Some(input.parse()?);
                    }
                }
                // 2. Check for Grouping: with(...) / without(...)
                else if input.peek(Ident) && input.peek2(syn::token::Paren) && {
                    let fork = input.fork();
                    let key: Ident = fork.parse().unwrap();
                    key == "with" || key == "without"
                } {
                    let key: Ident = input.parse()?;
                    let content;
                    syn::parenthesized!(content in input);
                    let types = syn::punctuated::Punctuated::<Type, Token![,]>::parse_terminated(&content)?;

                    if key == "with" {
                        with_caps.extend(types);
                    } else if key == "without" {
                        without_caps.extend(types);
                    }
                }
                // 3. Check for Flags: transparent
                else if input.peek(Ident) && {
                    let fork = input.fork();
                    let key: Ident = fork.parse().unwrap();
                    key == "transparent"
                } {
                    let _: Ident = input.parse()?; // consume
                    transparent = true;
                }
                // 4. Positional Boolean Expression (Requires / !Conflicts)
                else {
                    let expr: BoolExpr = input.parse()?;
                    predicates.push(expr);
                }

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }

            Ok(CapsArgs { predicates, with_caps, without_caps, transparent, target })
        }
    }
}

use caps_impl::{CapsArgs, BoolExpr};

// Helper to pretty-print BoolExpr
fn bool_expr_to_string(expr: &BoolExpr) -> String {
    match expr {
        BoolExpr::Cap(ty) => quote::quote!(#ty).to_string().replace(" ", ""),
        BoolExpr::And(lhs, rhs) => format!("({} & {})", bool_expr_to_string(lhs), bool_expr_to_string(rhs)),
        BoolExpr::Or(lhs, rhs) => format!("({} | {})", bool_expr_to_string(lhs), bool_expr_to_string(rhs)),
        BoolExpr::Not(operand) => format!("!{}", bool_expr_to_string(operand)),
    }
}

// Helper to generate predicates from args (Legacy/Simple for Structs)
fn generate_predicates(args: &CapsArgs, bound_param: &syn::Ident) -> Vec<proc_macro2::TokenStream> {
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

// Generate unique traits for each predicate to support custom diagnostic messages
fn generate_predicate_traits(
    args: &CapsArgs,
    bound_param: &syn::Ident,
    fn_name: &syn::Ident
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut bounds = Vec::new();
    let mut definitions = Vec::new();

    for (i, pred) in args.predicates.iter().enumerate() {
        let type_expr = bool_expr_to_type(pred);
        let msg_str = bool_expr_to_string(pred);

        // Generate a unique name for this requirement trait
        // e.g. __Req_myfunc_0
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
                 // Bake the message into a const for potential introspection (optional)
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

fn expand_caps_fn(args: CapsArgs, mut func: ItemFn) -> TokenStream {
    let generic_param = format_ident!("__C");
    let fn_name = func.sig.ident.clone();

    if args.transparent {
        let insert_pos = find_insert_position(&func.sig.generics.params);
        func.sig.generics.params.insert(insert_pos, syn::parse_quote!(#generic_param));

        for arg in &mut func.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = arg {
                if let Type::Path(type_path) = &mut *pat_type.ty {
                    if let Some(last_seg) = type_path.path.segments.last_mut() {
                        if last_seg.ident == "Doc" {
                            if let syn::PathArguments::None = last_seg.arguments {
                                last_seg.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote!(<#generic_param>));
                            } else if let syn::PathArguments::AngleBracketed(ga) = &mut last_seg.arguments {
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
        func.sig.generics.params.iter()
            .filter_map(|p| if let syn::GenericParam::Type(t) = p { Some(t.ident.clone()) } else { None })
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

    // Add With/Without bounds (these don't need custom messages as much, or can use standard wrappers)
    // We can keep using standard implementation for them
    let where_clause = func.sig.generics.make_where_clause();

    for cap in &args.with_caps {
        where_clause.predicates.push(syn::parse_quote!(
            #bound_param: ::tola_caps::With<#cap>
        ));
    }

    for cap in &args.without_caps {
        where_clause.predicates.push(syn::parse_quote!(
            #bound_param: ::tola_caps::Without<#cap>
        ));
    }

    // Output: definitions + function
    quote! {
        #(#pred_defs)*
        #func
    }.into()
}

fn expand_caps_struct(args: CapsArgs, mut item: syn::ItemStruct) -> TokenStream {
    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else {
        item.generics.params.iter()
            .filter_map(|p| if let syn::GenericParam::Type(t) = p { Some(t.ident.clone()) } else { None })
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

fn expand_caps_enum(args: CapsArgs, mut item: syn::ItemEnum) -> TokenStream {
    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else {
        item.generics.params.iter()
            .filter_map(|p| if let syn::GenericParam::Type(t) = p { Some(t.ident.clone()) } else { None })
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

fn expand_caps_impl(args: CapsArgs, mut item: syn::ItemImpl) -> TokenStream {
    let bound_param = if let Some(target) = args.target.clone() {
        target
    } else {
        item.generics.params.iter()
            .filter_map(|p| if let syn::GenericParam::Type(t) = p { Some(t.ident.clone()) } else { None })
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

/// Find the correct insertion position for __C in generic params.
/// Returns index where __C should be inserted.
/// Order: lifetimes, then types (non-default), then __C, then types (with default), then consts.
fn find_insert_position(params: &syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma>) -> usize {
    let mut pos = 0;
    for (i, param) in params.iter().enumerate() {
        match param {
            syn::GenericParam::Lifetime(_) => {
                // Lifetimes come first, keep going
                pos = i + 1;
            }
            syn::GenericParam::Type(t) => {
                // If this type has a default, insert before it
                if t.default.is_some() {
                    return pos;
                }
                // Non-default type, keep going
                pos = i + 1;
            }
            syn::GenericParam::Const(_) => {
                // Const params come at end, insert before them
                return pos;
            }
        }
    }
    pos
}

fn bool_expr_to_type(expr: &BoolExpr) -> proc_macro2::TokenStream {
    match expr {
        BoolExpr::Cap(ty) => quote! { #ty },
        BoolExpr::And(lhs, rhs) => {
            let l = bool_expr_to_type(lhs);
            let r = bool_expr_to_type(rhs);
            quote! { ::tola_caps::And<#l, #r> }
        },
        BoolExpr::Or(lhs, rhs) => {
            let l = bool_expr_to_type(lhs);
            let r = bool_expr_to_type(rhs);
            quote! { ::tola_caps::Or<#l, #r> }
        },
        BoolExpr::Not(operand) => {
            let o = bool_expr_to_type(operand);
            quote! { ::tola_caps::Not<#o> }
        },
    }
}


/// Single capability definition: `Name => "doc string"`
struct CapDef {
    name: Ident,
    doc: LitStr,
}

impl Parse for CapDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;
        let doc: LitStr = input.parse()?;
        Ok(CapDef { name, doc })
    }
}

/// Multiple capability definitions separated by commas
struct DefineCapabilitiesInput {
    caps: Punctuated<CapDef, Token![,]>,
}

impl Parse for DefineCapabilitiesInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let caps = Punctuated::parse_terminated(input)?;
        Ok(DefineCapabilitiesInput { caps })
    }
}

/// Batch define capabilities with auto-generated `Cap` suffix and doc comments.
///
/// This macro generates:
/// 1. The capability struct (e.g., `LinksCheckedCap`)
/// 2. A unique `HasXxxCap` trait for searching (e.g., `HasLinksCheckedCap`)
/// 3. Base impl: `(XxxCap, Rest)` implements `HasXxxCap`
/// 4. Recursive impls: `(OtherCap, Rest)` implements `HasXxxCap` if `Rest` does
///
/// # Usage
///
/// ```ignore
/// define_capabilities! {
///     LinksChecked => "Links have been checked (existence validated)",
///     LinksResolved => "Links have been resolved (relative → absolute paths)",
/// }
/// ```
///
/// Expands to:
/// - `LinksCheckedCap`, `LinksResolvedCap` (structs)
/// - `HasLinksCheckedCap`, `HasLinksResolvedCap` (presence traits)
/// - `NotHasLinksCheckedCap`, `NotHasLinksResolvedCap` (absence traits)
#[proc_macro]
pub fn define_capabilities(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DefineCapabilitiesInput);
    let caps: Vec<_> = input.caps.into_iter().collect();

    // Collect all capability names for cross-recursive impls
    let cap_names: Vec<_> = caps.iter().map(|c| &c.name).collect();
    let cap_structs: Vec<_> = cap_names
        .iter()
        .map(|n| format_ident!("{}Cap", n))
        .collect();
    let has_traits: Vec<_> = cap_names
        .iter()
        .map(|n| format_ident!("Has{}Cap", n))
        .collect();
    let not_has_traits: Vec<_> = cap_names
        .iter()
        .map(|n| format_ident!("NotHas{}Cap", n))
        .collect();

    // Generate struct definitions, trait definitions, and base impls
    let struct_defs: Vec<_> = caps
        .iter()
        .zip(cap_structs.iter())
        .zip(has_traits.iter())
        .zip(not_has_traits.iter())
        .map(|(((cap, struct_name), has_trait), not_has_trait)| {
            let doc = &cap.doc;
            let name_str = cap.name.to_string();
            let struct_name_str = struct_name.to_string();

            // Pre-compute diagnostic messages as string literals
            let has_diag_message = format!(
                "capability `{}` is required but not available in `{{Self}}`",
                struct_name_str
            );
            let has_diag_label = format!("this transform requires `{}`", struct_name_str);
            let has_doc_trait = format!(
                "Trait to check if a capability set contains `{}`",
                struct_name_str
            );

            let not_has_diag_message = format!(
                "capability `{}` must NOT be present in `{{Self}}`",
                struct_name_str
            );
            let not_has_diag_label = format!(
                "this transform must run BEFORE `{}` is added",
                struct_name_str
            );
            let not_has_doc_trait = format!(
                "Trait to check if a capability set does NOT contain `{}`",
                struct_name_str
            );

            let doc_marker = format!("Marker: {}", doc.value());

            quote! {
                #[doc = #doc_marker]
                #[derive(Debug, Clone, Copy, Default)]
                pub struct #struct_name;

                impl sealed::Sealed for #struct_name {}

                impl Capability for #struct_name {
                    const NAME: &'static str = #name_str;
                }

                // === HasXxxCap trait (presence check) ===
                #[doc = #has_doc_trait]
                #[diagnostic::on_unimplemented(
                    message = #has_diag_message,
                    label = #has_diag_label,
                    note = "try adding the appropriate Transform earlier in the pipeline"
                )]
                pub trait #has_trait: Capabilities {}

                // Base case: this cap at head
                impl<Rest: Capabilities> #has_trait for (#struct_name, Rest) {}

                // === NotHasXxxCap trait (absence check) ===
                #[doc = #not_has_doc_trait]
                #[doc = ""]
                #[doc = "Use with `#[requires_not]` to enforce ordering constraints."]
                #[diagnostic::on_unimplemented(
                    message = #not_has_diag_message,
                    label = #not_has_diag_label,
                    note = "this transform must run earlier in the pipeline"
                )]
                pub trait #not_has_trait: Capabilities {}

                // Base case: empty set does not contain any cap
                impl #not_has_trait for () {}

                // NOTE: We do NOT impl NotHasXxxCap for (XxxCap, Rest)!
                // This is the key: if XxxCap is at head, the trait is NOT satisfied.
            }
        })
        .collect();

    // Generate cross-recursive impls for HasXxxCap
    let has_cross_impls: Vec<_> = has_traits
        .iter()
        .enumerate()
        .flat_map(|(target_idx, target_trait)| {
            cap_structs
                .iter()
                .enumerate()
                .filter(move |(other_idx, _)| *other_idx != target_idx)
                .map(move |(_, other_struct)| {
                    quote! {
                        impl<Rest: #target_trait> #target_trait for (#other_struct, Rest) {}
                    }
                })
        })
        .collect();

    // Cross-recursive impls for NotHasXxxCap:
    // (OtherCap, Rest) satisfies NotHasXxxCap if Rest does.
    // (XxxCap, Rest) has no impl, so it never satisfies NotHasXxxCap.
    let not_has_cross_impls: Vec<_> = not_has_traits
        .iter()
        .enumerate()
        .flat_map(|(target_idx, target_trait)| {
            cap_structs
                .iter()
                .enumerate()
                .filter(move |(other_idx, _)| *other_idx != target_idx)
                .map(move |(_, other_struct)| {
                    quote! {
                        impl<Rest: #target_trait> #target_trait for (#other_struct, Rest) {}
                    }
                })
        })
        .collect();

    let output = quote! {
        #(#struct_defs)*
        #(#has_cross_impls)*
        #(#not_has_cross_impls)*
    };

    output.into()
}

// =============================================================================
// caps! proc-macro
// =============================================================================

/// Parse a list of types for caps! macro
struct CapsInput {
    types: Punctuated<Type, Token![,]>,
}

impl Parse for CapsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let types = Punctuated::parse_terminated(input)?;
        Ok(CapsInput { types })
    }
}

/// Create a capability set type from a list of capabilities.
///
/// This macro converts a list of capabilities into the trie representation:
/// ```ignore
/// caps![A, B, C]  // expands to: <<Empty as Add<C>>::Out as Add<B>>::Out as Add<A>>::Out
/// caps![]         // expands to: Empty
/// ```
///
/// Useful for type annotations:
/// ```ignore
/// type FullyProcessed = caps![LinksResolvedCap, LinksCheckedCap, SvgOptimizedCap];
/// ```
///
/// # Compile Error on Duplicates
///
/// Duplicate capabilities are not allowed and will cause a compile error:
/// ```ignore
/// caps![CapA, CapA]  // ERROR: duplicate capability `CapA`
/// ```
#[proc_macro]
pub fn cap_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CapsInput);
    let types: Vec<_> = input.types.into_iter().collect();

    // Check for duplicates
    if let Err(err) = check_duplicates(&types) {
        return err.to_compile_error().into();
    }

    let output = build_capset(&types);
    output.into()
}

/// Check for duplicate capabilities in the list
fn check_duplicates(types: &[Type]) -> syn::Result<()> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for ty in types {
        let ty_str = ty.to_token_stream().to_string().replace(" ", "");
        if !seen.insert(ty_str.clone()) {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "duplicate capability `{}`\n\
                     \n\
                     Each capability should appear only once in a capability set.\n\
                     Duplicate capabilities cause type inference ambiguity.",
                    ty_str
                ),
            ));
        }
    }
    Ok(())
}

/// Build capset type: <<Empty as Add<C>>::Out as Add<B>>::Out as Add<A>>::Out
fn build_capset(types: &[Type]) -> proc_macro2::TokenStream {
    if types.is_empty() {
        quote! { ::tola_caps::Empty }
    } else {
        // Build from right to left: Add<C>, then Add<B>, then Add<A>
        // Result: <<Empty as Add<C>>::Out as Add<B>>::Out as Add<A>>::Out
        let mut result = quote! { ::tola_caps::Empty };
        for ty in types.iter().rev() {
            result = quote! { <#result as ::tola_caps::With<#ty>>::Out };
        }
        result
    }
}

// =============================================================================
// #[derive(Capability)] derive macro
// =============================================================================

/// Derive macro to automatically implement the `Capability` trait.
///
/// This macro computes a SHA-256 hash of the struct name and generates
/// a `HashStream` type that produces nibbles from this hash.
///
/// # Usage
///
/// ```ignore
/// use tola_caps::Capability;
///
/// #[derive(Capability)]
/// struct MyCustomCap;
///
/// // Automatically generates:
/// // impl Capability for MyCustomCap {
/// //     type Stream = Cons<X?, Cons<X?, ...>>;
/// //     type At<D> = ...;
/// // }
/// ```
///
/// # Hash Generation
///
/// The hash is computed from the struct's fully qualified name (as seen by the macro).
/// This ensures different modules/crates get different hashes even for same-named types.
#[proc_macro_derive(Capability)]
pub fn derive_capability(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    // Compute BLAKE3 hash of the type name
    let hash = blake3::hash(name_str.as_bytes());

    // Convert first 16 bytes (32 nibbles) to Cons chain
    // We use 16 bytes = 32 nibbles, which is enough for most cases
    let nibbles: Vec<_> = hash.as_bytes().iter()
        .take(16)
        .flat_map(|byte| {
            let high = (byte >> 4) & 0x0F;
            let low = byte & 0x0F;
            vec![high, low]
        })
        .collect();

    // Generate the stream type: Cons<X?, Cons<X?, ...ConstStream<X?>>>
    // Use the last nibble for ConstStream tail
    let stream_type = build_hash_stream(&nibbles);

    let output = quote! {
        impl ::tola_caps::Capability for #name {
            type Stream = #stream_type;
            type At<D> = <<Self::Stream as ::tola_caps::GetTail<D>>::Out as ::tola_caps::HashStream>::Head
            where Self::Stream: ::tola_caps::GetTail<D>;
        }
    };

    output.into()
}

/// Build hash stream type from nibbles: Cons<X0, Cons<X1, ... ConstStream<XN>>>
fn build_hash_stream(nibbles: &[u8]) -> proc_macro2::TokenStream {
    if nibbles.is_empty() {
        quote! { ::tola_caps::ConstStream<::tola_caps::X0> }
    } else if nibbles.len() == 1 {
        let nib = nibble_to_ident(nibbles[0]);
        quote! { ::tola_caps::ConstStream<::tola_caps::#nib> }
    } else {
        let head = nibble_to_ident(nibbles[0]);
        let tail = build_hash_stream(&nibbles[1..]);
        quote! { ::tola_caps::Cons<::tola_caps::#head, #tail> }
    }
}

/// Convert a nibble (0-15) to its type identifier (X0-XF)
fn nibble_to_ident(n: u8) -> proc_macro2::Ident {
    let name = match n {
        0 => "X0", 1 => "X1", 2 => "X2", 3 => "X3",
        4 => "X4", 5 => "X5", 6 => "X6", 7 => "X7",
        8 => "X8", 9 => "X9", 10 => "XA", 11 => "XB",
        12 => "XC", 13 => "XD", 14 => "XE", 15 => "XF",
        _ => "X0",
    };
    format_ident!("{}", name)
}

//! Trie16 attribute macro for elegant code generation.
//!
//! Unified macro that handles struct, impl, and type alias with various modes.
//!
//! # Modes
//! - `#[trie16]` - Basic mode, replaces `_Slots_` and `_Node16_`
//! - `#[trie16(empty)]` - Generates EmptyNode16 type alias
//! - `#[trie16(each)]` - Expands `each!{}` blocks and `each(_Slots_)` in where clauses
//!
//! # Placeholders
//! - `_Slots_`  expands to N0, N1, N2, ..., NF (generic params)
//! - `_Node16_` expands to Node16<N0, N1, ..., NF> (full type)
//! - `each(_Slots_): Bound` expands to N0: Bound, N1: Bound, ..., NF: Bound
//! - `each! { expr }` repeated 16x with _Slot_ replaced by each Ni

use proc_macro2::{TokenStream, TokenTree, Ident, Span, Group, Delimiter};
use quote::{format_ident, quote};

/// The 16 slot parameter names
pub const SLOTS: [&str; 16] = [
    "N0", "N1", "N2", "N3", "N4", "N5", "N6", "N7",
    "N8", "N9", "NA", "NB", "NC", "ND", "NE", "NF",
];

/// The 16 nibble names (X0..XF)
pub const NIBBLES: [&str; 16] = [
    "X0", "X1", "X2", "X3", "X4", "X5", "X6", "X7",
    "X8", "X9", "XA", "XB", "XC", "XD", "XE", "XF",
];

/// Generate slot idents: N0, N1, ..., NF
pub fn slot_idents() -> Vec<syn::Ident> {
    SLOTS.iter().map(|s| format_ident!("{}", s)).collect()
}

/// Expand to N0, N1, ..., NF token stream
pub fn expand_slots() -> TokenStream {
    let slots = slot_idents();
    quote! { #(#slots),* }
}

/// Expand to Node16<N0, N1, ..., NF> token stream
pub fn expand_node16_type() -> TokenStream {
    let slots = slot_idents();
    quote! { Node16<#(#slots),*> }
}

// =============================================================================
// Unified #[trie16] with modes
// =============================================================================

/// Parse the mode from attribute arguments
pub fn parse_mode(attr: TokenStream) -> Mode {
    let tokens: Vec<_> = attr.into_iter().collect();
    if tokens.is_empty() {
        return Mode::Basic;
    }

    // Check for keyword
    if let Some(TokenTree::Ident(ident)) = tokens.first() {
        let name = ident.to_string();
        match name.as_str() {
            "all_empty" => return Mode::Empty,
            "each_slot" => return Mode::Each,
            "for_nibble" => return Mode::ForNibble,
            "for_nibble_split" => return Mode::ForNibbleSplit,
            _ => {}
        }
    }

    Mode::Basic
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Basic,
    Empty,
    Each,
    ForNibble,
    ForNibbleSplit,
}

/// Main entry point for #[node16] with optional mode
pub fn expand_trie16_with_mode(mode: Mode, input: TokenStream) -> TokenStream {
    match mode {
        Mode::Basic => replace_placeholders(input),
        Mode::Empty => expand_empty_type_alias(input),
        Mode::Each => expand_with_each(input),
        Mode::ForNibble => expand_for_nibble(input),
        Mode::ForNibbleSplit => expand_for_nibble_split(input),
    }
}

/// ForNibble mode: generate 16 copies with _Nibble_/N0..NF pairs replaced
fn expand_for_nibble(input: TokenStream) -> TokenStream {
    let mut result = Vec::new();

    for i in 0..16 {
        let nibble = NIBBLES[i];
        let slot = SLOTS[i];

        // Replace _Nibble_ with Xi and _SlotN_ with Ni
        let replaced = replace_nibble_slot(input.clone(), nibble, slot);
        // Then do normal placeholder replacement
        let expanded = replace_placeholders(replaced);
        result.extend(expanded);
    }

    result.into_iter().collect()
}

/// Replace _Nibble_ and _SlotN_ placeholders
fn replace_nibble_slot(tokens: TokenStream, nibble: &str, slot: &str) -> TokenStream {
    let mut result = Vec::new();

    for token in tokens {
        match token {
            TokenTree::Ident(ident) if ident == "_Nibble_" => {
                result.push(TokenTree::Ident(Ident::new(nibble, Span::call_site())));
            }
            TokenTree::Ident(ident) if ident == "_SlotN_" => {
                result.push(TokenTree::Ident(Ident::new(slot, Span::call_site())));
            }
            TokenTree::Group(group) => {
                let inner = replace_nibble_slot(group.stream(), nibble, slot);
                result.push(TokenTree::Group(Group::new(group.delimiter(), inner)));
            }
            other => result.push(other),
        }
    }

    result.into_iter().collect()
}

/// ForNibbleSplit mode: like ForNibble but with _Before_ and _After_ placeholders
/// - `_Before_` expands to slots before current (e.g., for N2: N0, N1)
/// - `_After_` expands to slots after current (e.g., for N2: N3, N4, ..., NF)
fn expand_for_nibble_split(input: TokenStream) -> TokenStream {
    let mut result = Vec::new();

    for i in 0..16 {
        let nibble = NIBBLES[i];
        let slot = SLOTS[i];
        let before: Vec<&str> = SLOTS[..i].to_vec();
        let after: Vec<&str> = SLOTS[i+1..].to_vec();

        // Replace all placeholders
        let replaced = replace_nibble_split(input.clone(), nibble, slot, &before, &after);
        // Then do normal placeholder replacement
        let expanded = replace_placeholders(replaced);
        result.extend(expanded);
    }

    result.into_iter().collect()
}

/// Replace _Nibble_, _SlotN_, _Before_, _After_, _EmptyBefore_, _EmptyAfter_ placeholders
fn replace_nibble_split(tokens: TokenStream, nibble: &str, slot: &str, before: &[&str], after: &[&str]) -> TokenStream {
    let mut result = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match &token {
            TokenTree::Ident(ident) if ident == "_Nibble_" => {
                result.push(TokenTree::Ident(Ident::new(nibble, Span::call_site())));
            }
            TokenTree::Ident(ident) if ident == "_SlotN_" => {
                result.push(TokenTree::Ident(Ident::new(slot, Span::call_site())));
            }
            TokenTree::Ident(ident) if ident == "_Before_" => {
                // Expand to before slots with trailing comma if non-empty
                for s in before.iter() {
                    result.push(TokenTree::Ident(Ident::new(s, Span::call_site())));
                    result.push(TokenTree::Punct(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)));
                }
                // If _Before_ is empty and followed by comma, skip that comma
                if before.is_empty() {
                    if let Some(TokenTree::Punct(p)) = iter.peek() {
                        if p.as_char() == ',' {
                            iter.next(); // consume trailing comma
                        }
                    }
                }
            }
            TokenTree::Ident(ident) if ident == "_After_" => {
                // If _After_ non-empty, add leading comma then slots
                if !after.is_empty() {
                    for (j, s) in after.iter().enumerate() {
                        if j > 0 {
                            result.push(TokenTree::Punct(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)));
                        }
                        result.push(TokenTree::Ident(Ident::new(s, Span::call_site())));
                    }
                }
            }
            TokenTree::Ident(ident) if ident == "_EmptyBefore_" => {
                // Expand to Empty, Empty, ... (before.len() times) with trailing comma
                for _ in 0..before.len() {
                    result.push(TokenTree::Ident(Ident::new("Empty", Span::call_site())));
                    result.push(TokenTree::Punct(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)));
                }
                // If empty and followed by comma, skip that comma
                if before.is_empty() {
                    if let Some(TokenTree::Punct(p)) = iter.peek() {
                        if p.as_char() == ',' {
                            iter.next();
                        }
                    }
                }
            }
            TokenTree::Ident(ident) if ident == "_EmptyAfter_" => {
                // Expand to Empty, Empty, ... (after.len() times)
                if !after.is_empty() {
                    for j in 0..after.len() {
                        if j > 0 {
                            result.push(TokenTree::Punct(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)));
                        }
                        result.push(TokenTree::Ident(Ident::new("Empty", Span::call_site())));
                    }
                }
            }
            TokenTree::Group(group) => {
                let inner = replace_nibble_split(group.stream(), nibble, slot, before, after);
                result.push(TokenTree::Group(Group::new(group.delimiter(), inner)));
            }
            _ => result.push(token),
        }
    }

    // Post-process: remove ,, sequences (double commas)
    remove_double_commas(result.into_iter().collect())
}

/// Remove ,, sequences that may arise from empty placeholders
fn remove_double_commas(tokens: TokenStream) -> TokenStream {
    let tokens_vec: Vec<_> = tokens.into_iter().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens_vec.len() {
        // Check for leading comma followed by nothing useful (for <,...>)
        if i == 0 {
            if let TokenTree::Punct(p) = &tokens_vec[i] {
                if p.as_char() == ',' {
                    i += 1;
                    continue;
                }
            }
        }

        // Skip consecutive commas
        if let TokenTree::Punct(p1) = &tokens_vec[i] {
            if p1.as_char() == ',' && i + 1 < tokens_vec.len() {
                if let TokenTree::Punct(p2) = &tokens_vec[i + 1] {
                    if p2.as_char() == ',' {
                        i += 1; // skip one comma
                        continue;
                    }
                }
            }
        }

        // Recursively handle groups
        if let TokenTree::Group(group) = &tokens_vec[i] {
            let inner = remove_double_commas(group.stream());
            result.push(TokenTree::Group(Group::new(group.delimiter(), inner)));
        } else {
            result.push(tokens_vec[i].clone());
        }
        i += 1;
    }

    result.into_iter().collect()
}

/// Basic mode: replace _Slots_ and _Node16_
fn replace_placeholders(tokens: TokenStream) -> TokenStream {
    let mut result = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) if ident == "_Slots_" => {
                // Replace _Slots_ with N0, N1, ..., NF
                for (i, slot) in SLOTS.iter().enumerate() {
                    result.push(TokenTree::Ident(Ident::new(slot, Span::call_site())));
                    if i < 15 {
                        result.push(TokenTree::Punct(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)));
                    }
                }
            }
            TokenTree::Ident(ident) if ident == "_Node16_" => {
                // Replace _Node16_ with Node16<N0, N1, ..., NF>
                result.push(TokenTree::Ident(Ident::new("Node16", Span::call_site())));
                let inner = expand_slots();
                result.push(TokenTree::Punct(proc_macro2::Punct::new('<', proc_macro2::Spacing::Alone)));
                result.extend(inner);
                result.push(TokenTree::Punct(proc_macro2::Punct::new('>', proc_macro2::Spacing::Alone)));
            }
            TokenTree::Group(group) => {
                let inner = replace_placeholders(group.stream());
                let new_group = Group::new(group.delimiter(), inner);
                result.push(TokenTree::Group(new_group));
            }
            other => result.push(other),
        }
    }

    result.into_iter().collect()
}

/// Empty mode: generate EmptyNode16 = Node16<Empty, ...>
fn expand_empty_type_alias(input: TokenStream) -> TokenStream {
    // Parse as type alias - can be `type Foo;` or `type Foo = ();`
    // Try parsing as ItemType first
    if let Ok(item) = syn::parse2::<syn::ItemType>(input.clone()) {
        let vis = &item.vis;
        let ident = &item.ident;
        let attrs = &item.attrs;

        return quote! {
            #(#attrs)*
            #vis type #ident = Node16<
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty
            >;
        };
    }

    // Try parsing as "type Foo;" (without = part) - we need custom parsing
    // For now, just look for the pattern: vis? "type" ident ";"
    let tokens: Vec<_> = input.clone().into_iter().collect();
    let mut vis_tokens = Vec::new();
    let mut ident_token = None;
    let mut found_type = false;

    for token in tokens.iter() {
        match token {
            TokenTree::Ident(id) if id == "type" => {
                found_type = true;
            }
            TokenTree::Ident(id) if found_type && ident_token.is_none() => {
                ident_token = Some(id.clone());
            }
            TokenTree::Ident(id) if !found_type => {
                vis_tokens.push(token.clone());
            }
            _ => {}
        }
    }

    if let Some(ident) = ident_token {
        let vis: TokenStream = vis_tokens.into_iter().collect();
        return quote! {
            #vis type #ident = Node16<
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty
            >;
        };
    }

    // Fallback
    input
}

/// Each mode: expand each(_Slots_) in where clauses, then expand _Slot_ in statements
fn expand_with_each(tokens: TokenStream) -> TokenStream {
    // First pass: expand each(_Slots_) in where clauses
    let expanded_where = expand_each_where(tokens);
    // Second pass: expand statements containing _Slot_
    let expanded_slots = expand_slot_statements(expanded_where);
    // Final pass: normal placeholder replacement
    replace_placeholders(expanded_slots)
}

/// Expands statements containing _Slot_ by repeating them 16 times
fn expand_slot_statements(tokens: TokenStream) -> TokenStream {
    let mut result = Vec::new();

    for token in tokens {
        match token {
            TokenTree::Group(group) => {
                // For brace groups (function bodies), expand slot statements
                if group.delimiter() == Delimiter::Brace {
                    let inner = expand_slot_in_block(group.stream());
                    result.push(TokenTree::Group(Group::new(Delimiter::Brace, inner)));
                } else {
                    // Recursively process other groups
                    let inner = expand_slot_statements(group.stream());
                    result.push(TokenTree::Group(Group::new(group.delimiter(), inner)));
                }
            }
            other => result.push(other),
        }
    }

    result.into_iter().collect()
}

/// Expand statements in a block that contain _Slot_
fn expand_slot_in_block(tokens: TokenStream) -> TokenStream {
    let mut result = Vec::new();
    let mut current_stmt = Vec::new();

    for token in tokens {
        match &token {
            TokenTree::Punct(p) if p.as_char() == ';' => {
                // Statement ends
                current_stmt.push(token);

                // Check if this statement contains _Slot_
                if contains_slot(&current_stmt) {
                    // Repeat 16 times with _Slot_ replaced
                    for slot in SLOTS.iter() {
                        for t in &current_stmt {
                            result.extend(replace_single_slot_token(t.clone(), "_Slot_", slot));
                        }
                    }
                } else {
                    result.extend(current_stmt.drain(..));
                }
                current_stmt.clear();
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                // Nested block - check if current_stmt should be repeated
                if contains_slot(&current_stmt) {
                    // This is a control structure with _Slot_, repeat whole thing
                    current_stmt.push(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        expand_slot_in_block(group.stream())
                    )));
                } else {
                    current_stmt.push(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        expand_slot_in_block(group.stream())
                    )));
                }
            }
            _ => current_stmt.push(token),
        }
    }

    // Handle remaining tokens (no trailing semicolon)
    if !current_stmt.is_empty() {
        if contains_slot(&current_stmt) {
            for slot in SLOTS.iter() {
                for t in &current_stmt {
                    result.extend(replace_single_slot_token(t.clone(), "_Slot_", slot));
                }
            }
        } else {
            result.extend(current_stmt);
        }
    }

    result.into_iter().collect()
}

/// Check if a slice of tokens contains _Slot_
fn contains_slot(tokens: &[TokenTree]) -> bool {
    for token in tokens {
        match token {
            TokenTree::Ident(ident) if ident == "_Slot_" => return true,
            TokenTree::Group(group) => {
                if contains_slot(&group.stream().into_iter().collect::<Vec<_>>()) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Replace _Slot_ in a single token, returning potentially multiple tokens
fn replace_single_slot_token(token: TokenTree, placeholder: &str, replacement: &str) -> Vec<TokenTree> {
    match token {
        TokenTree::Ident(ident) if ident == placeholder => {
            vec![TokenTree::Ident(Ident::new(replacement, Span::call_site()))]
        }
        TokenTree::Group(group) => {
            let inner = replace_single_slot(group.stream(), placeholder, replacement);
            vec![TokenTree::Group(Group::new(group.delimiter(), inner))]
        }
        other => vec![other],
    }
}

/// Expand each(_Slots_): Bound to N0: Bound, N1: Bound, ..., NF: Bound
fn expand_each_where(tokens: TokenStream) -> TokenStream {
    let mut result = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match &token {
            TokenTree::Ident(ident) if ident == "each" => {
                // Check for each(...)
                if let Some(TokenTree::Group(group)) = iter.peek() {
                    if group.delimiter() == Delimiter::Parenthesis {
                        let group = iter.next().unwrap();
                        if let TokenTree::Group(g) = group {
                            // Expect `:` and bounds after the group
                            if let Some(TokenTree::Punct(p)) = iter.peek() {
                                if p.as_char() == ':' {
                                    iter.next(); // consume :
                                    // Collect bounds until , or where-ending
                                    let bounds = collect_bounds(&mut iter);

                                    // Generate N0: Bound, N1: Bound, ...
                                    for (i, slot) in SLOTS.iter().enumerate() {
                                        result.push(TokenTree::Ident(Ident::new(slot, Span::call_site())));
                                        result.push(TokenTree::Punct(proc_macro2::Punct::new(':', proc_macro2::Spacing::Alone)));
                                        result.extend(bounds.clone());
                                        if i < 15 {
                                            result.push(TokenTree::Punct(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone)));
                                        }
                                    }
                                    continue;
                                }
                            }
                            // Put back what we consumed
                            result.push(token);
                            result.push(TokenTree::Group(g));
                            continue;
                        }
                    }
                }
                result.push(token);
            }
            TokenTree::Group(group) => {
                let inner = expand_each_where(group.stream());
                let new_group = Group::new(group.delimiter(), inner);
                result.push(TokenTree::Group(new_group));
            }
            _ => result.push(token),
        }
    }

    result.into_iter().collect()
}

/// Collect bounds until a comma at the same level or end of where clause
fn collect_bounds(iter: &mut std::iter::Peekable<impl Iterator<Item = TokenTree>>) -> Vec<TokenTree> {
    let mut bounds = Vec::new();

    while let Some(token) = iter.peek() {
        match token {
            TokenTree::Punct(p) if p.as_char() == ',' => break,
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => break, // impl body
            _ => {
                bounds.push(iter.next().unwrap());
            }
        }
    }

    bounds
}

/// Replace a single placeholder with a specific slot
fn replace_single_slot(tokens: TokenStream, placeholder: &str, replacement: &str) -> TokenStream {
    let mut result = Vec::new();

    for token in tokens {
        match token {
            TokenTree::Ident(ident) if ident == placeholder => {
                result.push(TokenTree::Ident(Ident::new(replacement, Span::call_site())));
            }
            TokenTree::Group(group) => {
                let inner = replace_single_slot(group.stream(), placeholder, replacement);
                let new_group = Group::new(group.delimiter(), inner);
                result.push(TokenTree::Group(new_group));
            }
            other => result.push(other),
        }
    }

    result.into_iter().collect()
}

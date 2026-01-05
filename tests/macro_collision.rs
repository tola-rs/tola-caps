use tola_caps::Capability;

macro_rules! gen_struct {
    ($name:ident) => {
        #[derive(Capability)]
        pub struct $name;
    };
}

mod a {
    use super::*;
    gen_struct!(S);
}

mod b {
    use super::*;
    gen_struct!(S);
}

// Global detection
impl tola_caps::detect::AutoCapSet for a::S { type Out = tola_caps::trie::Leaf<a::S>; }
impl tola_caps::detect::AutoCapSet for b::S { type Out = tola_caps::trie::Leaf<b::S>; }

#[test]
fn test_macro_gen_collision() {
    // If Span hashing uses Definition Site span, these might collide.
    // If it uses Call Site span, they are distinct.

    // Check if Set{a::S} contains b::S
    // Should be FALSE
    let has_b_in_a = <tola_caps::trie::Leaf<a::S> as tola_caps::trie::Evaluate<b::S>>::RESULT;
    assert!(!has_b_in_a, "Macro generated structs collided! Span hashing failed.");
}

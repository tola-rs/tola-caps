//! Test runtime reflection via the Inspect trait

use tola_caps::prelude::*;
use tola_caps::capability::Inspect;

#[derive(Capability)]
struct CapA;

#[derive(Capability)]
struct CapB;

#[derive(Capability)]
struct CapC;

#[test]
fn test_inspect_empty() {
    let empty = tola_caps::capability::Empty;
    let mut names = Vec::new();
    empty.inspect(|n| names.push(n));
    assert!(names.is_empty());
}

#[test]
fn test_inspect_single_cap() {
    type Set = caps![CapA];
    let set = <Set as Default>::default();
    let mut names = Vec::new();
    set.inspect(|n| names.push(n));
    assert_eq!(names.len(), 1);
    assert!(names[0].ends_with("CapA"));
}

#[test]
fn test_inspect_multiple_caps() {
    type Set = caps![CapA, CapB, CapC];
    let set = <Set as Default>::default();
    let mut names = Vec::new();
    set.inspect(|n| names.push(n));
    // Should have 3 capabilities
    assert_eq!(names.len(), 3);
    // Names should contain CapA, CapB, CapC (in some order, based on hash routing)
    let names_str = names.join(", ");
    assert!(names_str.contains("CapA"), "Missing CapA in {}", names_str);
    assert!(names_str.contains("CapB"), "Missing CapB in {}", names_str);
    assert!(names_str.contains("CapC"), "Missing CapC in {}", names_str);
}

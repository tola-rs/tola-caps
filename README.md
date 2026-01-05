# tola-caps

**Type-level capability system for Rust**

Let the compiler track your state.

tola-caps is a **typestate** library: encode state into types, let the compiler verify transitions. Each capability is a zero-sized marker type, and a capability set is a type-level collection of these markers.

```
Type -> Hash -> Point in metric space -> Position in set
```

You define states (`Parsed`, `Validated`), the compiler enforces correct ordering. Zero runtime cost.

## Features

- **Typestate Pattern**: State transitions enforced by the type system
- **Zero Runtime Cost**: All checks happen at compile time
- **Extensible**: Define capabilities anywhere, no central registry
- **Boolean Logic**: Full `&`, `|`, `!`, `()` support
- **Readable Errors**: Shows `(A & !B)` instead of complex generics
- **Stable Specialization**: Nightly-like specialization on stable Rust
- **Trait Detection**: Compile-time check if any type implements any trait
- **no_std Compatible**: Works without std, optional alloc support

### Feature Flags

```toml
[dependencies]
tola-caps = "0.2"  # default: std + specialize + detect

# Or customize:
tola-caps = { version = "0.2", default-features = false, features = ["alloc"] }
```

| Feature | Description |
|---------|-------------|
| `std` (default) | Standard library support, includes `alloc` |
| `alloc` | Alloc types (Vec, Box, String...) without full std |
| `specialize` (default) | Enable `#[specialize]` and `specialization!` macros |
| `detect` (default) | Std trait detection (`caps_check!`, `AutoCaps`) |

## Use Cases

- **Protocol State Machines**: HTTP connections (Connected -> Authenticated -> Active)
- **Database Transactions**: Track transaction lifecycle (Started -> Modified -> Committed)
- **File Handles**: Enforce open -> read/write -> close ordering
- **Build Pipelines**: Ensure compile -> test -> deploy sequence
- **Access Control**: Type-safe permission tracking (Anonymous -> User -> Admin)

## Macro Overview

| Macro | Target | Purpose |
|-------|--------|---------|
| `#[derive(Capability)]` | struct | Define capability marker |
| `#[derive(AutoCaps)]` | struct/enum | Enable type for generic context detection |
| `#[trait_autocaps]` | trait | Enable trait for generic context detection |
| `#[caps_bound]` | fn/impl | Add capability constraints |
| `#[specialize]` | impl | Attribute-style specialization |
| `specialization!{}` | - | Block-style specialization syntax |
| `caps![]` | - | Build capability set type |
| `caps_check!` | - | Compile-time trait detection |

**Note**:
- `caps_check!` for concrete types works with ANY trait directly
- Generic contexts (`fn<T>`) need `T: AutoCaps` for both `caps_check!` and `specialization!`
- Standard library types already implement `AutoCaps`
- `#[derive(AutoCaps)]` enables custom types for generic detection and specialization
- `#[trait_autocaps]` enables custom traits for detection/specialization
## Quick Start

```rust
use tola_caps::prelude::*;

// 1. Define your states as capabilities
#[derive(Capability)] struct Parsed;
#[derive(Capability)] struct Validated;
#[derive(Capability)] struct Signed;

// 2. Use PhantomData to track capabilities at type level
pub struct Doc<C = caps![]> {
    content: String,
    _caps: std::marker::PhantomData<C>,
}

impl Doc {
    pub fn new(s: &str) -> Self {
        Doc { content: s.to_string(), _caps: std::marker::PhantomData }
    }
}

// 3. Add capability constraints to functions
fn parse(doc: Doc) -> Doc<caps![Parsed]> {
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

#[caps_bound(C: Parsed, with(Validated))]
fn validate<C>(doc: Doc<C>) -> Doc<with![C, Validated]> {
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

#[caps_bound(C: Parsed & Validated, with(Signed))]
fn sign<C>(doc: Doc<C>) -> Doc<with![C, Signed]> {
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

fn main() {
    let doc = Doc::new("hello");
    let doc = parse(doc);
    let doc = validate(doc);  // OK: Has Parsed
    let doc = sign(doc);      // OK: Has Parsed & Validated

    // sign(parse(Doc::new("x")));  // ERROR: Compile error: missing Validated
}
```

## Trait Detection

Check if a type implements any trait at compile time:

```rust
use tola_caps::caps_check;
use std::fmt::Debug;

// Concrete types - works with ANY trait, no setup needed
assert!(caps_check!(String: Clone));
assert!(!caps_check!(String: Copy));
assert!(caps_check!(String: Clone & !Copy));

// Custom traits also work on concrete types
trait MyTrait {}
impl MyTrait for String {}
assert!(caps_check!(String: MyTrait));
```

### Generic Context

For generic type parameters, add `AutoCaps` bound:

```rust
use tola_caps::{caps_check, detect::AutoCaps};

fn pick_strategy<T: AutoCaps>() -> &'static str {
    if caps_check!(T: Copy) { "memcpy" }
    else if caps_check!(T: Clone) { "clone" }
    else { "move" }
}
```

**Why `AutoCaps` is needed for generics:**
- Concrete types (e.g., `String`): The compiler knows all trait impls at the call site
- Generic `T`: The compiler doesn't know `T`'s traits until instantiation
- `AutoCaps` provides a type-level capability set that enables detection in generic contexts
- Standard library types (String, Vec, Box, etc.) implement `AutoCaps` automatically
- Custom types need `#[derive(AutoCaps)]`
```

Or use specialization for true type-level dispatch:

```rust
use tola_caps::specialization;

specialization! {
    trait Strategy {
        fn pick() -> &'static str;
    }

    impl<T> Strategy for T {
        default fn pick() -> &'static str { "move" }
    }

    impl<T: Clone> Strategy for T {
        default fn pick() -> &'static str { "clone" }
    }

    impl<T: Copy> Strategy for T {
        fn pick() -> &'static str { "memcpy" }
    }
}
```

## Specialization

tola-caps provides stable Rust specialization using two syntax styles. Both achieve the same goal: select different implementations based on trait bounds.

**How it works**: The `specialization!` macro transforms trait bounds into capability queries on `Cap<T>` (the type's capability set). Selection happens at type-level via `SelectCap<Condition, Then, Else>`. This requires `T: AutoCaps` to provide the capability set.

**Key insight**: `T: Clone` becomes `Cap<T>: Evaluate<IsClone>` - dispatching based on whether `IsClone` exists in `T`'s capability set.

### Block Syntax: `specialization!`

All-in-one macro with `default` keyword for overridable items:

```rust
use tola_caps::specialization;

specialization! {
    trait Encode {
        fn encode(&self) -> Vec<u8>;
    }

    impl<T> Encode for T {
        default fn encode(&self) -> Vec<u8> { vec![] }  // fallback
    }

    impl<T: Clone> Encode for T {
        default fn encode(&self) -> Vec<u8> { vec![0xC1] }  // Clone impl
    }

    impl<T: Copy> Encode for T {
        fn encode(&self) -> Vec<u8> { vec![0xC0] }  // Copy impl (most specific)
    }

    impl Encode for String {
        fn encode(&self) -> Vec<u8> { self.as_bytes().to_vec() }  // concrete override
    }
}

// Usage:
let v: Vec<u8> = 42i32.encode();  // picks Copy impl
let v: Vec<u8> = "hello".to_string().encode();  // picks String impl
```

### Attribute Syntax: `#[specialize]`

Distributed across files, with explicit constraint syntax:

```rust
use tola_caps::specialize;

trait Encode { fn encode(&self) -> Vec<u8>; }

#[specialize(default)]  // Fallback impl
impl<T> Encode for T {
    fn encode(&self) -> Vec<u8> { vec![] }
}

#[specialize(T: Clone)]  // When T: Clone
impl<T> Encode for T {
    fn encode(&self) -> Vec<u8> { vec![1] }
}

#[specialize(T: Clone, U: Copy)]  // Multiple bounds
impl<T, U> Pair<T, U> for (T, U) { /* ... */ }

#[specialize]  // Concrete type (highest priority)
impl Encode for String {
    fn encode(&self) -> Vec<u8> { self.as_bytes().to_vec() }
}
```

### Specialization Priority

Resolution order (highest to lowest priority):

1. **Concrete types**: `impl Encode for String`
2. **Multiple bounds**: `impl<T: Clone + Copy> Encode for T`
3. **Single bound**: `impl<T: Clone> Encode for T`
4. **Default**: `impl<T> Encode for T` with `default` keyword

### Safety Guarantees

tola-caps uses **type-level capability dispatch** (different from Rust's nightly `specialization`):

- **Mechanism**: Dispatches via `Cap<T>` capability set lookup, not trait impl ordering
- **Primary benefit**: No lifetime-dependent dispatch - the main source of nightly's unsoundness
- **Trade-off**: Requires `AutoCaps` on types, but more reliable on stable Rust

For detailed analysis and caveats, see the [Specialization Soundness](#specialization-soundness) section.

### Limitations

- **`AutoCaps` required for generic dispatch**:
  - Both `caps_check!` AND `specialization!` use `Cap<T>` internally
  - Concrete types: `caps_check!(String: Clone)` works directly (no generics)
  - Generic `T`: Needs `T: AutoCaps` for `Cap<T>` to exist
  - Standard library types already implement `AutoCaps`
  ```rust
  // Concrete types: no AutoCaps needed
  assert!(caps_check!(String: Clone));  // OK

  // Generic context: AutoCaps required for Cap<T>
  fn strategy<T: AutoCaps>() -> &'static str {
      if caps_check!(T: Clone) { "clone" } else { "move" }
  }

  // Custom type for generic specialization
  #[derive(Clone, AutoCaps)]
  struct MyType;
  assert_eq!(strategy::<MyType>(), "clone");
  ```
- **Custom traits**: Need `#[trait_autocaps]` for generic dispatch
  ```rust
  #[trait_autocaps]
  trait MyTrait { }  // Generates IsMyTrait capability marker
  ```
- **No lifetime-based specialization**: By design - lifetimes are erased at codegen
## More Examples

### Transaction Safety

```rust
#[derive(Capability)] struct TxStarted;
#[derive(Capability)] struct TxCommitted;

#[caps_bound(C: !TxStarted, with(TxStarted))]
impl<C> Conn<C> {
    fn begin(self) -> Conn<with![C, TxStarted]> { todo!() }
}

#[caps_bound(C: TxStarted & !TxCommitted, with(TxCommitted))]
impl<C> Conn<C> {
    fn commit(self) -> Conn<with![C, TxCommitted]> { todo!() }
}
```

### Dropping Capabilities

```rust
#[derive(Capability)] struct Admin;

#[caps_bound(C: Admin, without(Admin))]
fn drop_admin<C>(user: User<C>) -> User<without![C, Admin]> {
    todo!()
}
```

### Set Operations

```rust
type Combined = union![caps![A], caps![B]];
type Common = intersect![caps![A, B], caps![B, C]];
```

## Architecture

**The Challenge**: Type-level set membership when Rust's coherence rules prevent arbitrary type comparison.

**The Solution**: Hash-based routing + compile-time identity verification.

```
Capability Name  ->  FNV-1a Hash  ->  Trie Path  ->  Identity Check
    "Parsed"     ->    0x84...     ->  [8][4]...  ->    Unique
```

Each capability gets a deterministic address in a 16-ary trie. Membership is O(1) path lookup.

### Components

#### 1. Routing (Hash -> Path)

FNV-1a (64-bit) hashes the capability name into a `HashStream`. Each nibble (4 bits) selects a slot in the 16-ary `Node16` trie.

#### 2. Identity & Collision Resolution

Hash collisions are unavoidable with 64-bit FNV-1a. tola-caps uses a **two-tier identity system**:

**Tier 1: 64-bit Hash Routing**
- FNV-1a hash -> 16 nibbles -> Trie path
- Handles initial routing to leaf nodes
- Fast O(1) lookup
- Note: The codebase also has a 512-bit hash (4×128-bit) system for future collision reduction

**Tier 2: Full Path Identity (Guaranteed Uniqueness)**
- Complete source path: `name@module::path:file:line:col`
- Encoded as Finger Tree structure with tiered IList
- Each character stored as compile-time type
- For strings >64 chars: smart sampling (head32 + mid16 + tail16)

```rust
// Example identity type structure
type Identity = IList64<
    IList<C<'c'>, IList<C<'o'>, IList<C<'r'>, ...>>>
>;
```

**Collision Handling**: When two capabilities hash to the same trie slot:
1. Compare full path identities -> guaranteed unique
2. If paths match -> **compile error** (same capability defined twice)

This ensures O(1) routing with 100% collision resolution.

#### 3. Trait Detection (Autoref Fallback)

Rust prefers inherent methods over trait methods:

```rust
trait Fallback { const IS_CLONE: bool = false; }
impl<T> Fallback for T {}

struct Detect<T>(PhantomData<T>);
impl<T: Clone> Detect<T> {
    const IS_CLONE: bool = true;  // Inherent shadows trait
}
```

`Detect::<String>::IS_CLONE` finds the inherent const if `String: Clone`, else falls back to trait default.

#### 4. Specialization (Cap<T> Dispatch)

Specialization uses `Cap<T>` (the type's capability set) for type-level selection:

```rust
// Generated by specialization! macro:
type Selected = <Cap<T> as SelectCap<IsClone, CloneImpl, DefaultImpl>>::Out;

// SelectCap checks if IsClone exists in Cap<T>:
// - If T: Clone, IsClone is Present -> selects CloneImpl
// - Otherwise, IsClone is Absent -> selects DefaultImpl
```

This is pure type-level computation - no runtime dispatch.

### Complexity

| Operation | Cost |
|-----------|------|
| Routing | O(1) - fixed 16 levels |
| Collision resolution | O(1) in practice |
| Runtime | Zero - compile-time only |
| Memory | Zero - all `PhantomData` |

## Error Messages

**Human-readable errors** - shows your capability logic, not compiler internals:

```text
error[E0277]: Capability requirement failed: ((A | B) & !C)
   --> src/main.rs:44:19
    |
 44 |     complex_logic(doc_c);
    |     ^^^^^^^^^^^^^ requirement '((A | B) & !C)' not satisfied
    |
    = help: ensure the type has capabilities: (A OR B) AND NOT C
```

## Caveats

### Specialization Soundness

tola-caps uses **type-level capability dispatch**, which differs fundamentally from Rust's nightly `specialization` feature:

**Mechanism comparison:**

| Aspect | Nightly `specialization` | tola-caps |
|--------|-------------------------|----------|
| Dispatch | Trait impl ordering | `Cap<T>` capability set lookup |
| Decision time | Monomorphization | Type-level (compile-time) |
| Lifetime handling | Problematic (unsound) | Irrelevant (no lifetime dispatch) |
| Associated types | Can be unsound | Not affected |

**How tola-caps avoids nightly's unsoundness:**

1. **No lifetime-dependent dispatch**: Selection is based on capability markers (`IsClone`, `IsCopy`), not on lifetime bounds like `'static`. The SOUNDNESS.md documents that lifetime-dependent specialization is the primary source of unsoundness in nightly.

2. **No associated type normalization issues**: tola-caps generates separate impl structs and selects between them at type-level. There's no "which impl to use" decision that could differ between type-checking and codegen.

3. **Capability set is fixed per type**: `Cap<MyType>` is deterministic - it's built from `#[derive(AutoCaps)]` at definition time, not inferred from context.

**What works safely:**
```rust
// OK: Trait bound specialization
impl<T: Clone> Trait for T { }  // IsClone in Cap<T>
impl<T> Trait for T { }          // Fallback

// OK: Concrete type override
impl Trait for String { }        // Highest priority
```

**What is NOT supported (by design):**
```rust
// NOT supported: Lifetime-dependent dispatch
// tola-caps cannot distinguish T: 'static from T
impl<T: 'static> Trait for T { }
impl<T> Trait for T { }
```

**Caveats:**
- [VERIFIED] Sound for trait-bound-based dispatch (Clone, Copy, Debug, etc.)
- [VERIFIED] No lifetime erasure issues since capabilities don't encode lifetimes
- [LIMITATION] Cannot specialize on `'static` or other lifetime bounds
- [LIMITATION] Custom traits need `#[trait_autocaps]` for generic dispatch

### Practical Limitations

- **`AutoCaps` is the foundation**:
  - Both `caps_check!` and `specialization!` use `Cap<T>` for dispatch
  - Standard library types (String, Vec, i32, etc.) already implement `AutoCaps`
  - Custom types need `#[derive(AutoCaps)]` for generic contexts
  ```rust
  #[derive(AutoCaps)]
  struct MyType;  // Now works with caps_check! and specialization!
  ```

- **`#[trait_autocaps]` for custom traits**:
  - Registers trait in the capability system
  - Enables `caps_check!(T: MyTrait)` in generic contexts
  - Enables specialization based on `MyTrait` bound
  ```rust
  #[trait_autocaps]
  trait MyTrait { }  // Now IsMyTrait capability is available
  ```

## License

MIT

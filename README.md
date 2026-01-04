# tola-caps

**Capability system using 16-ary type-level trie with hash-stream routing**

## Features

- **O(1) Compile-time Lookup**: Type-level hash-based routing via 16-ary radix trie
- **Zero Runtime Overhead**: All capability checks happen at compile time
- **Infinite Extensibility**: Define capabilities anywhere, no central registry needed
- **Clean API**: No `_` placeholders or turbofish in function signatures
- **Boolean Logic**: Full support for `And(&)`, `Or(|)`, `Not(!)`, `All`, `Any`, `Group(())` queries
- **Friendly Diagnostics**: Custom error messages show `(A & !B)` instead of complex generic types
- **No Dependencies**: Pure type-level computation, no external crates


## Use Cases

This is a **type-state** library. It uses the type system to:

- Enforce operation ordering (parse → validate → sign)
- Prevent invalid state transitions (can't commit after rollback)
- Track resource lifecycle (lock → use → unlock)

It does NOT perform runtime checks or static analysis of your code.
The compiler itself becomes the checker.

## Design: Capabilities, not Roles

Define **fine-grained capabilities** (actions), not roles:

```rust
// ✓ Capabilities: what you CAN DO
#[derive(Capability)] struct CanRead;
#[derive(Capability)] struct CanWrite;
#[derive(Capability)] struct CanDelete;

// Roles are just type aliases for capability sets
type User = caps![CanRead];
type Moderator = caps![CanRead, CanWrite, CanDelete];
type Admin = caps![CanRead, CanWrite, CanDelete, CanBan];

// Functions require capabilities, not roles
#[caps_bound(CanDelete)]
fn delete_post<C>(doc: Doc<C>) { ... }  // Any role with CanDelete works
```

This avoids needing inheritance (`Admin > Moderator > User`). The set operations handle it naturally.

## Quick Start
```rust
use tola_caps::prelude::*;

// Define capabilities for a document processing pipeline
#[derive(Capability)] struct Parsed;       // Document has been parsed
#[derive(Capability)] struct Validated;    // Schema validated
#[derive(Capability)] struct Sanitized;    // XSS/injection cleaned
#[derive(Capability)] struct Signed;       // Cryptographically signed
#[derive(Capability)] struct Encrypted;    // Content encrypted

// Capability sets
type RawDoc = caps![];
type ParsedDoc = caps![Parsed];

// Document type carrying capability proof
pub struct Doc<C> {
    content: String,
    _caps: std::marker::PhantomData<C>,
}

impl Doc<RawDoc> {
    pub fn new(s: &str) -> Self {
        Doc { content: s.to_string(), _caps: std::marker::PhantomData }
    }
}

// First step: parse raw input (no requirements)
fn parse(doc: Doc<RawDoc>) -> Doc<ParsedDoc> {
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

// Next steps: Standard generics with explicit capability bounds.
// We use `target = C` to specify which generic carries the capabilities.

// Validation requires Parsed. Adds Validated.
#[caps_bound(Parsed, with(Validated), target = C)]
fn validate<C>(doc: Doc<C>) -> Doc<with![C, Validated]>
{
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

// Sanitization requires Parsed & Validated. Adds Sanitized.
#[caps_bound(Parsed & Validated, with(Sanitized), target = C)]
fn sanitize<C>(doc: Doc<C>) -> Doc<with![C, Sanitized]>
{
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

// Signing requires sanitization, conflicts with Encrypted. Adds Signed.
#[caps_bound(Sanitized, !Encrypted, with(Signed), target = C)]
fn sign<C>(doc: Doc<C>) -> Doc<with![C, Signed]>
{
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

// Encryption requires signing. Adds Encrypted.
#[caps_bound(Signed, with(Encrypted), target = C)]
fn encrypt<C>(doc: Doc<C>) -> Doc<with![C, Encrypted]>
{
    Doc { content: doc.content, _caps: std::marker::PhantomData }
}

// Final consumer: requires full safety chain
#[caps_bound(Parsed & Validated & Sanitized & Signed, target = C)]
fn publish<C>(doc: Doc<C>) {
    println!("Publishing secure document");
}

fn pipeline_example() {
    let raw = Doc::new("<script>alert('xss')</script>");
    let parsed = parse(raw);
    let validated = validate(parsed);
    let sanitized = sanitize(validated);
    let signed = sign(sanitized);
    publish(signed);  // ✓ Compiles

    // fail: encrypt(sanitized); // Missing Signed
}
```

## Dependency Chain Validation

Capability dependencies are checked at compile time:

```rust
// === Real-world scenario: Database transaction safety ===

#[derive(Capability)] struct TxStarted;
#[derive(Capability)] struct TxCommitted;
#[derive(Capability)] struct TxRolledBack;
#[derive(Capability)] struct Locked;
#[derive(Capability)] struct Unlocked;

// Initial state: No transaction active.
// Conflicts with Started/Committed/RolledBack.
#[caps_bound(!TxStarted & !TxCommitted & !TxRolledBack, with(TxStarted))]
impl<C> Conn<C> {
    fn begin_tx(self) -> Conn<with![C, TxStarted]> { todo!() }
}

// Active transaction operations
// Requires TxStarted, must NOT be committed/rolled back yet.
#[caps_bound(TxStarted, !TxCommitted & !TxRolledBack)]
impl<C> Conn<C> {
    #[caps_bound(with(TxCommitted))]
    fn commit(self) -> Conn<with![C, TxCommitted]> { todo!() }

    #[caps_bound(with(TxRolledBack))]
    fn rollback(self) -> Conn<with![C, TxRolledBack]> { todo!() }

    #[caps_bound(with(Locked))]
    fn acquire_lock(self) -> Conn<with![C, Locked]> { todo!() }
}

// Operations requiring a Lock
// Requires (Locked & TxStarted), !TxCommitted. adds Unlocked.
#[caps_bound(Locked & TxStarted & !TxCommitted, with(Unlocked))]
impl<C> Conn<C> {
    fn release_lock(self) -> Conn<with![C, Unlocked]> { todo!() }
}

// === Dropping Capabilities ===
// You can use `without!` to remove capabilities (e.g., dropping privileges)

#[derive(Capability)] struct RootPrivileges;

// 'without' argument generates Without<RootPrivileges> bound
// 'without!' macro computes the new set type by removing the capability
// naming 'without' group is also supported, e.g. without(RootPrivileges)
#[caps_bound(RootPrivileges, without(RootPrivileges), target = C)]
fn drop_root<C>(user: Doc<C>) -> Doc<without![C, RootPrivileges]> {
    // ... logic to drop privileges ...
    todo!()
}
```

## Set Operations

```rust
type Combined = union![caps![A], caps![B]];
type Common = intersect![caps![A, B], caps![B, C]];
```

## Compiler Diagnostics

Error messages show your original logic expression instead of internal types.

For example, with a requirement `(A | B) & !C` called on a set containing `C`:

```text
// Requirements: Must have (A OR B) AND (NOT C)
#[caps_bound((A | B) & !C)]
fn complex_logic<C>(doc: Doc<C>) { ... }

// Call with a set containing [C]
complex_logic(doc_c);
```

Compiler output:

```text
error[E0277]: Capability requirement failed: ((A | B) & !C)
   --> src/main.rs:44:19
    |
 44 |     complex_logic(doc_c);
    |     ^^^^^^^^^^^^^ This capability set violates requirement '((A | B) & !C)'
    |
    = note: Check if you are missing a required capability or possess a conflicting one.
```

Instead of `And<Or<A, B>, Not<C>>`, you see `((A | B) & !C)`.

### Why Unique Traits?

`#[diagnostic::on_unimplemented]` needs a static string literal. To show different messages for different requirements, we generate a unique trait per requirement at macro expansion time.

- **Functions**: We generate ephemeral traits like `__Req_myfunc_0` alongside the function.
- **Structs/Impls**: Uses the generic `Evaluate` trait error to avoid code bloat.


## Transparent Mode (Experimental)

The `transparent` flag rewrites code to simulate implicit capability context:

1. Injects a generic `__C`
2. Rewrites bare `Doc` to `Doc<__C>`
3. `with![Cap]` expands to `with![__C, Cap]`

### Example

```rust
#[caps_bound(Parsed, transparent)]
fn process(doc: Doc) -> Doc<with![Validated]> { ... }
```

Expands to:

```rust
fn process<__C>(doc: Doc<__C>) -> Doc<with![__C, Validated]>
where
    __C: Evaluate<Parsed, Out=Present>
{ ... }
```

### Drawbacks

Transparent mode is fragile:
- IDEs don't see `__C`
- Breaks if you have a generic named `__C`
- Obscures data flow

Prefer explicit generics (`fn foo<C>(d: Doc<C>)`) for production code.

## Architecture

**Hash stream** / **16-ary trie** / **Elastic depth**

We can't compare arbitrary types in Rust's trait system (coherence rules). So instead, we hash each capability's type name and use those hash bits to route through the trie.

Each capability gets a BLAKE3 hash, which we treat as a stream of 4-bit nibbles. The trie has 16 branches per node. To check if capability `Foo` is in a set, we just follow `Foo`'s hash nibbles down the trie.

If two capabilities share the same first nibble, the trie grows deeper at that branch until their hashes differ. In practice this almost never happens (BLAKE3 collision), so lookups are effectively O(1).

See [src/capability.rs](src/capability.rs) for details.

## Concurrency & Async

All capability types are zero-sized markers. They automatically derive `Send + Sync`.

```rust
// Works out of the box
async fn process<C>(doc: Doc<C>) -> Doc<with![C, Done]>
where C: Evaluate<Has<Started>, Out = Present>
{
    some_async_work().await;
    Doc { content: doc.content, _caps: PhantomData }
}

// Sharing across threads
fn spawn<C: Send + 'static>(doc: Doc<C>) {
    std::thread::spawn(move || use_doc(doc));
}
```

**Limitation**: The type system tracks capabilities per value, not across channels or shared state. If you need to pass capability changes between threads/tasks, you'll need to design your own synchronization (e.g., typed channels).

## License
MIT

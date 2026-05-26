# Fingerprint Grammar Reference

> The DSL for structural pattern-matching in antigen fingerprint declarations.
> Fingerprints live in the `#[antigen(fingerprint = r#"..."#)]` attribute field
> and drive passive detection: `cargo antigen scan` evaluates them against every
> item in the workspace and reports matches automatically, even on sites without
> an explicit `#[presents]` marker.

---

## Syntax overview

A fingerprint is a comma-separated list of constraints, all **AND'd** at the
top level:

```
item = enum,
name = matches("*Class"),
variants = 3..=8,
has_method("meet", "(Self, Self) -> Self")
```

This fingerprint matches an enum named with the `*Class` suffix, 3 to 8
variants, and a `meet` method with the given signature. All four constraints
must hold.

Fingerprints are written as raw strings to avoid having to escape double quotes:

```rust
fingerprint = r#"item = enum, name = matches("*Class"), variants = 3..=8"#
```

---

## Operators

### `item = <kind>`

Restricts the match to a specific Rust item kind.

```
item = struct
item = enum
item = trait
item = fn
item = impl
item = type
item = mod
```

**Behavior**: the scanner only evaluates the remaining constraints for items of
this kind. This is the most important performance lever: a fingerprint with
`item = enum` is only evaluated against `enum` definitions, not every item in
the workspace.

**Note on methods**: `item = fn` matches free functions only. Methods inside
`impl` blocks are matched via `item = impl` + `has_method`.

**Example**: match any struct:

```
item = struct
```

**Example**: match any `Drop` implementation:

```
item = impl, has_method("drop", "(& mut self)")
```

---

### `name = matches("<glob>")`

Matches the item's identifier against a glob pattern. `*` matches any
sequence of characters; `?` matches any single character. Case-sensitive.
Whole-name match (the pattern must match the full identifier, not just a
substring).

```
name = matches("*Class")       -- any name ending in "Class"
name = matches("Test*")        -- any name starting with "Test"
name = matches("*Cache*")      -- any name containing "Cache"
name = matches("ExpKernelState") -- exact match
```

**Example**: match enums whose name ends in `Class`:

```
item = enum, name = matches("*Class")
```

**What it does NOT match**: substrings without `*`. `name = matches("Class")`
only matches an item named exactly `Class`, not `DeterminismClass`.

---

### `variants = M..=N`

Matches an enum whose variant count is between M and N, inclusive. Only
meaningful with `item = enum` (or `any_of([item = enum, ...])`); has no effect
on non-enum items.

```
variants = 3..=8      -- 3 to 8 variants (inclusive)
variants = 2..=2      -- exactly 2 variants
variants = 1..=100    -- 1 to 100 variants
```

**Example**: match class enums with a typical number of variants:

```
item = enum, name = matches("*Class"), variants = 3..=8
```

---

### `has_method("<name>", "<signature>")`

Matches an `impl` block that has a method with the given name and
signature shape. Only `item = impl` sites are evaluated; this operator has
no effect on struct, enum, trait, fn, or other item kinds.

```
has_method("meet", "(self, Self) -> Self")   -- by-value receiver + one typed Self arg
has_method("drop", "(& mut self)")            -- mutable reference receiver
has_method("new", "() -> Self")               -- static constructor (no receiver)
```

**Receiver spacing** (ATK-W6a-013, resolved by ADR-010 Amendment 5): `proc_macro2`
renders receiver tokens with a space between `&` and `self`/`mut`. The engine
canonicalizes user-provided pattern strings through proc_macro2 at parse time, so
both the space-separated and compact forms work:

```
-- Both accepted (engine canonicalizes to same form):
has_method("is_valid", "(& self) -> bool")
has_method("is_valid", "(&self) -> bool")     -- also works post-Amendment 5
has_method("drop", "(& mut self)")
has_method("drop", "(&mut self)")             -- also works post-Amendment 5
```

*Historical note*: before ADR-010 Amendment 5 (2026-05-11), the compact form
`"(&mut self)"` produced silent zero matches because whitespace normalization could
not insert missing spaces. The engine fix canonicalizes both forms to the same
proc_macro2 token spacing at parse time. ATK-W6a-013 was inverted to assert the
corrected behavior.

**Receiver rendering reference** — how each receiver form appears in patterns:

| Rust source | Pattern form |
|---|---|
| `fn f(self, ...)` | `"(self, ...)"` |
| `fn f(&self, ...)` | `"(& self, ...)"` |
| `fn f(&mut self, ...)` | `"(& mut self, ...)"` |
| `fn f(a: T, b: T)` (no receiver) | `"(T, T)"` |

Typed argument names are dropped; only types appear. `fn meet(self, other: Self)`
renders as `"(self, Self)"`, not `"(Self, Self)"`.

**Signature matching**: whitespace is normalized before comparison. Extra
spaces collapse. The comparison is textual after normalization — it is not a
full Rust parser comparison. Use the canonical space-normalized form.

**Example**: match impl blocks containing a `drop` method:

```
item = impl, has_method("drop", "(& mut self)")
```

**Example**: the `PanickingInDrop` fingerprint (matches Drop impls with a
`panic!` or `unreachable!` call in the body):

```
all_of([
    item = impl,
    has_method("drop", "(& mut self)"),
    any_of([
        body_contains_macro("panic"),
        body_contains_macro("unreachable")
    ])
])
```

---

### `attr_present("<path>")`

Matches an item that has an outer attribute whose path matches `path`. The
match is against the last path segment OR the full path.

The path is the *attribute name* — the identifier before any `(...)` content.
For `#[derive(Serialize)]`, the attribute name is `derive`, not `Serialize`.
There is no operator to match against derive-macro arguments.

```
attr_present("repr")          -- matches #[repr(u8)], #[repr(C)], etc.
attr_present("test")          -- matches #[test]
attr_present("derive")        -- matches #[derive(...)], any derive
attr_present("cfg")           -- matches #[cfg(...)]
attr_present("clippy::panic") -- matches by full path
```

**Example**: match enums with `repr`:

```
item = enum, attr_present("repr")
```

**Example**: match test functions:

```
item = fn, attr_present("test")
```

---

### `doc_contains("<substring>")`

Matches an item whose doc-comment text contains the given substring.
Case-sensitive. Searches across all `///` doc comments on the item itself.

**Scope note**: reads the item's own doc attributes only — not doc comments on
fields inside a struct body, not doc comments on methods inside an `impl` block.
To surface items that *have a method* containing specific text, pair `item = impl`
with `has_method` rather than `doc_contains`.

```
doc_contains("lattice")
doc_contains("SAFETY:")
doc_contains("# Panics")
doc_contains("meet")
```

**Example**: match enums documented with "lattice" semantics:

```
item = enum, doc_contains("lattice")
```

**Example**: a richer polarity-class fingerprint using doc hints:

```
item = enum,
name = matches("*Class"),
variants = 3..=8,
has_method("meet", "(self, Self) -> Self"),
any_of([
    attr_present("repr"),
    doc_contains("strength"),
    doc_contains("lattice"),
    doc_contains("meet")
])
```

---

### `body_contains_macro("<name>")`

Matches a function or method body that contains a macro invocation whose path
last segment equals `name`.

```
body_contains_macro("panic")        -- matches panic!("...")
body_contains_macro("unreachable")  -- matches unreachable!()
body_contains_macro("todo")         -- matches todo!()
body_contains_macro("unimplemented") -- matches unimplemented!()
body_contains_macro("assert")       -- matches assert!(...)
body_contains_macro("println")      -- matches println!(...)
```

**Match rule**: the last segment of the macro path must match. `panic!(...)`,
`std::panic!(...)`, and `my_crate::panic!(...)` all match
`body_contains_macro("panic")`.

**Limitation**: this operator detects macro *invocations*, not method calls.
`.unwrap()` and `.expect(...)` are method calls, not macro invocations — they
do NOT match `body_contains_macro("panic")` even though they can panic. Use
explicit `#[presents]` markers for method-call panic paths.

**Example**: match functions that call `todo!`:

```
item = fn, body_contains_macro("todo")
```

---

## Composition operators

These combine constraints with logical operators.

### `all_of([...])`

Every child constraint must match. Useful for grouping multiple conditions that
must all hold together.

```
all_of([
    item = impl,
    has_method("drop", "(& mut self)"),
    body_contains_macro("panic")
])
```

At the top level, constraints are already AND'd implicitly. `all_of` adds
grouping when needed inside `any_of` or when structuring complex fingerprints.

`not` is only legal inside `all_of` (see below).

---

### `any_of([...])`

At least one child constraint must match.

```
any_of([
    body_contains_macro("panic"),
    body_contains_macro("unreachable"),
    body_contains_macro("todo")
])
```

**Example**: match functions or closures that call any of several macros:

```
item = fn,
any_of([
    body_contains_macro("panic"),
    body_contains_macro("unreachable"),
    body_contains_macro("unimplemented")
])
```

**Example**: match structs or enums (item kind either/or):

```
any_of([item = struct, item = enum]),
name = matches("*Config")
```

---

### `not(<constraint>)`

The child constraint must NOT match.

`not` is only legal **inside `all_of`**, and only as a sibling of at least one
positive matcher. This prevents the De Morgan promiscuity loophole where
`any_of([not(A), not(B)])` becomes `not(all_of([A, B]))` and effectively
creates a top-level negation that matches most of the codebase.

```
-- Legal: not inside all_of with a positive sibling
all_of([
    item = enum,
    not(name = matches("Test*"))
])

-- Illegal: not at top level
not(item = enum)              -- parse error

-- Illegal: not inside any_of
any_of([not(item = enum), item = struct])  -- parse error

-- Illegal: all_of with only not children (no positive sibling)
all_of([not(item = enum), not(item = struct)])  -- parse error
```

**Example**: match class enums that are NOT test fixtures:

```
item = enum,
name = matches("*Class"),
all_of([
    variants = 3..=8,
    not(name = matches("Test*"))
])
```

---

## Complete examples

### `PanickingInDrop` — Drop impls with explicit panic macros

```
all_of([
    item = impl,
    has_method("drop", "(& mut self)"),
    any_of([
        body_contains_macro("panic"),
        body_contains_macro("unreachable")
    ])
])
```

Matches: `impl Drop for T { fn drop(&mut self) { panic!("...") } }`

Does not match: `impl Drop for T { fn drop(&mut self) { self.cleanup().expect("...") } }`
(`.expect` is a method call, not a macro invocation)

---

### `PolarityInvertedClassMeet` — Class enums with meet methods

```
item = enum,
name = matches("*Class"),
variants = 3..=8,
has_method("meet", "(self, Self) -> Self"),
any_of([
    attr_present("repr"),
    doc_contains("strength"),
    doc_contains("lattice"),
    doc_contains("meet")
])
```

Matches: a `DeterminismClass` enum with 4 variants, `#[repr(u8)]`, and a
`meet(self, other: Self) -> Self` method. The `"(self, Self) -> Self"` pattern
matches a by-value receiver (`self`) plus one typed `Self` argument — the form
tambear's class enums use. Use `"(& self, Self) -> Self"` for reference-receiver
methods, or `"(Self, Self) -> Self"` for pure static methods with no receiver.

The `any_of` clause anchors to explicit documentation or representation
attributes, reducing false positives on arbitrary enums with `meet` methods
unrelated to lattice semantics.

---

### Free-function panic callers

```
item = fn,
any_of([
    body_contains_macro("panic"),
    body_contains_macro("unreachable"),
    body_contains_macro("unimplemented"),
    body_contains_macro("todo")
])
```

Matches any free function whose body calls one of the four "this shouldn't
happen" macros. Useful for auditing code that defers error handling.

---

### Structs whose names suggest a cache

```
any_of([item = struct, item = enum]),
name = matches("*Cache*")
```

Matches `LruCache`, `CacheEntry`, `ResponseCacheState`, etc.

---

## Constraints and limits

**Depth cap**: fingerprints may not nest deeper than 10 levels. A fingerprint
exceeding this depth is rejected at parse time with a diagnostic.

**Node count cap**: fingerprints may not exceed 256 total constraint nodes. A
fingerprint exceeding this count is rejected at parse time with a diagnostic.

**`not` placement**: only legal inside `all_of`, only as a sibling of at least
one positive matcher. Violating this rule produces a parse-time error.

**Empty fingerprint**: a fingerprint with no constraints is rejected. At least
one constraint is required.

---

## What scan does with fingerprints

When you run `cargo antigen scan`, the scanner:

1. Parses all antigen declarations and their fingerprints
2. Walks every `.rs` file in the workspace
3. For each item in each file, evaluates any fingerprints whose `item = <kind>`
   constraint matches the item's kind
4. Reports matches as fingerprint-match presentations (distinct from explicit
   `#[presents]` markers, but shown together in output)

Fingerprint matches appear in scan output as `[fingerprint match]`:

```
856 fingerprint match(es) — structurally similar to a declared antigen:

  ./src/resource.rs:14  PanickingInDrop on impl [fingerprint match]
  ./src/state.rs:42     PolarityInvertedClassMeet on enum [fingerprint match]
```

Fingerprint matches that are also explicitly `#[presents]`-annotated show as
one presentation (not two). Matches that are `#[antigen_tolerance]`-annotated
are filtered from the output.

---

## References

- [`docs/decisions.md`](decisions.md) — ADR-010 (fingerprint grammar v1) and
  amendments; performance invariants; De Morgan loophole rationale
- [`docs/tutorial.md`](tutorial.md) — step-by-step walkthrough using a
  real fingerprint
- [`docs/usage-patterns.md`](usage-patterns.md) — composition-boundary and
  tolerance patterns using fingerprints in context
- `antigen-fingerprint/src/lib.rs` — canonical grammar source; `Constraint`
  enum with all operators

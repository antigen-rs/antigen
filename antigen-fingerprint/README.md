# antigen-fingerprint

**The fingerprint grammar for [antigen](https://crates.io/crates/antigen):
parse, match, and serialize structural failure-class fingerprints.**

A *fingerprint* is the structural pattern in an `#[antigen(fingerprint = r#"..."#)]`
declaration — a small DSL that describes the *shape* of code a failure-class
exhibits (an enum whose name matches `*Class` with 3–8 variants; an `impl` that
discards an active argument; a serde struct that accepts unknown fields). This
crate owns three operations over that grammar:

- **parse** — `&str` → a `Fingerprint` AST (`Fingerprint::parse`).
- **match** — evaluate a `Fingerprint` against a scanned code item.
- **serialize** — `Fingerprint` → DSL string (`to_antigen_attr`), the parser's
  exact inverse.

Parsing and matching live in the same crate by design, so the macro path
(compile-time validation that a fingerprint parses) and the scan path (matching
against real code) cannot drift apart.

## Parse a fingerprint

```rust
use antigen_fingerprint::Fingerprint;

let fp = Fingerprint::parse(
    r#"item = enum, name = matches("*Class"), variants = 3..=8"#,
).unwrap();
```

The grammar is a custom DSL (combinators `all_of` / `any_of` / `not` over leaf
predicates like `item =`, `name = matches(...)`, `derives(...)`, `body_calls(...)`,
`is_unsafe`, `impl_of_trait(...)`). The full human-readable reference is
[`docs/fingerprint-grammar.md`](https://github.com/antigen-rs/antigen/blob/main/docs/fingerprint-grammar.md).

## Round-trip: serialize is the parser's exact inverse

`to_antigen_attr` renders a `Fingerprint` back into the attribute form a human
pastes — and for every parser-producible fingerprint, the text it emits parses
back to the same AST:

```rust
use antigen_fingerprint::{Fingerprint, to_antigen_attr};

let fp = Fingerprint::parse(
    r#"item = enum, name = matches("*Class"), variants = 3..=8"#,
).unwrap();

// Renders the full attribute, with the same inner grammar the parser reads:
assert_eq!(
    to_antigen_attr(&fp),
    r##"#[antigen(fingerprint = r#"item = enum, name = matches("*Class"), variants = 3..=8"#)]"##,
);
```

This is what lets tooling generate a fingerprint (for example, from
`cargo antigen propose`) and emit an attribute a developer can paste verbatim.

## Usage

Most users depend on [`antigen`](https://crates.io/crates/antigen), which uses
this crate internally. Depend on `antigen-fingerprint` directly when you need the
grammar itself — to parse, match, or render fingerprints outside the macro flow.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).

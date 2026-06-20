# antigen

**Structural memory of failure-classes for Rust. Make implicit immunity explicit.**

> **Status: working beta, published on crates.io.** The macros,
> witness types, and structural-recognition primitives below are real and
> exercised by a large test suite. The API is stabilizing toward `1.0` and may
> still change between beta releases.

## What this crate is

The core library of the [antigen project](https://github.com/antigen-rs/antigen).
It gives you the attribute macros for declaring failure-classes and marking code
against them, plus the scan/audit/witness machinery they feed. The companion
[`cargo-antigen`](https://crates.io/crates/cargo-antigen) subcommand is what
*reads* these declarations across a workspace.

The primitives you write by hand:

- `#[antigen(name = "...", fingerprint = r#"..."#)]` — declare a named
  failure-class with a structural fingerprint (the grammar lives in
  [`antigen-fingerprint`](https://crates.io/crates/antigen-fingerprint)).
- `#[presents(...)]` — mark code as exhibiting a failure-class pattern, with
  optional `requires = <predicate>` / `proof = <expr>` site-attached evidence.
- `#[defended_by(...)]` — register a test as the observed code-tier witness for a
  failure-class (ADR-029 observe-don't-declare).
- `#[descended_from(...)]` — propagate failure-class markers through structural
  derivation.
- `#[antigen_tolerance(...)]` — mark a deliberate, ratified exception.
- The marker family `#[dread]` / `#[aura]` / `#[red_flag]` — record a *felt*
  unease at a site so `cargo antigen propose` can later anti-unify a candidate
  fingerprint from the cluster.

That is the core; the crate re-exports a wider macro vocabulary (the clinical and
clonal families) — see the
[macro reference](https://github.com/antigen-rs/antigen/tree/main/docs) for the
full set.

## A first taste

Declare a failure-class with a structural fingerprint, then mark a site that
exhibits it:

```rust
use antigen::{antigen, presents};
use serde::Deserialize;

// A serde struct that silently accepts unknown fields — a real failure-class.
// Antigen names are kebab-case.
#[antigen(
    name = "deserialize-accepts-unknown-fields",
    fingerprint = r#"all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])"#
)]
pub struct DeserializeAcceptsUnknownFields;

#[derive(Deserialize)]
#[presents(DeserializeAcceptsUnknownFields)]
pub struct Config {
    pub port: u16,
}
```

The `fingerprint` string is the same grammar
[`antigen-fingerprint`](https://crates.io/crates/antigen-fingerprint) parses;
[`cargo antigen scan`](https://crates.io/crates/cargo-antigen) walks the workspace
to surface the marked sites and check that each has a witness.

## Why structural memory matters

When a bug is fixed, the test for THAT bug ships. But the *failure-class* the bug
was an instance of — the family of cases sharing its structural shape — usually
doesn't get captured. The lesson lives in commit messages, code comments,
developer memory. When a structurally-similar new type gets written later, **none
of the immunity transfers automatically**.

Antigen makes failure-class memory **structural and inheritable** — declared in
the type system, checked by tooling, propagated by composition.

## Honest scope

- A fingerprint match is a **candidate to inspect, not an audited verdict**.
  Antigen surfaces structure that *resembles* a known class; whether it is truly
  vulnerable is the reader's call (and the witness layer's job to refine).
- It catches *named* failure-classes. It does not detect novel logic errors.
- It composes existing tools (clippy, proptest, kani, prusti) rather than
  replacing them; it is not a substitute for tests, lints, or formal
  verification.
- Antigen never labels unmarked code clean.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).

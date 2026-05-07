# antigen

**Structural memory of failure-classes for Rust. Make implicit immunity explicit.**

> **Status: Design phase — `0.0.1` reserves the namespace.** Real macros, witness types,
> and structural recognition primitives are under active design at
> [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen).

## What this crate is (intended)

The core library of the [antigen project](https://github.com/antigen-rs/antigen). Will provide:

- `#[antigen(name = "...", fingerprint = "...")]` — declare a named failure-class
- `#[presents(antigen)]` — mark code as vulnerable to a known failure-class
- `#[immune(antigen, witness = ...)]` — declare immunity with proof requirement
- `#[descended_from(...)]` — propagate antigen markers through derivation

Together with the [`cargo-antigen`](https://crates.io/crates/cargo-antigen) extension,
these primitives compose existing Rust ecosystem tools (clippy, proptest, kani, prusti)
into a coherent immune-system surface for failure-class memory.

## Why structural memory matters

When a bug is fixed, the test for THAT bug ships. But the *failure-class* the bug was an
instance of — the family of cases sharing its structural shape — usually doesn't get
captured. The lesson lives in commit messages, code comments, developer memory.

When a structurally-similar new type gets created later, **none of the immunity transfers
automatically**.

Antigen makes failure-class memory **structural and inheritable** — declared in the type
system, checked by tooling, propagated by composition.

## What this is NOT

- Not a documentation system. Documentation drifts; antigen declarations are checked by tooling.
- Not a replacement for tests, lints, deprecations, or formal verification. Antigen *composes* them.
- Not a logic-bug catcher. It catches *named* failure-classes; it does not detect novel logic errors.

## Status

The published `0.0.1` reserves the crate name and signals intent. See the
[design documents](https://github.com/antigen-rs/antigen/tree/main/docs/expedition) for
the active design — including the 8-class first-principles failure taxonomy and the
biological-to-Rust constructs mapping.

## License

Dual-licensed under [MIT](https://github.com/antigen-rs/antigen/blob/main/LICENSE-MIT) or
[Apache-2.0](https://github.com/antigen-rs/antigen/blob/main/LICENSE-APACHE).

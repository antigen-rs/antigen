# Antigen

**Structural memory of failure-classes for Rust. Make implicit immunity explicit.**

> **Status: Design phase.** The `0.0.1` release reserves the namespace and signals intent.
> Active design happens here in `docs/expedition/`. Real macros and tooling under construction.

## The problem

When a bug is fixed, the test for THAT bug ships. But the *failure-class* the bug was an
instance of — the family of cases sharing its structural shape — usually doesn't get
captured. The lesson lives in commit messages, code comments, developer memory.

When a structurally-similar new type gets created later, **none of the immunity transfers
automatically**. The same failure shows up in a slightly different costume, and the team
re-derives the lesson from scratch.

This is the implicit-memory failure mode. Mainstream programming culture handles it through
mentorship and career-pain. AI-assisted coding amplifies it because agents lose context
between sessions, so the implicit memory has nowhere persistent to live.

## The shape of the answer

Antigen makes failure-class memory **structural and inheritable**. Three verbs:

- **Build** an antigen — declare a failure-class with a structural fingerprint
- **Give** an antigen — mark code as presenting (vulnerable to) a known failure-class
- **Find** antigens — let cargo tooling scan the codebase and flag undefended sites

The biological metaphor maps cleanly:

| Biology | Rust ecosystem |
|---|---|
| Pathogen Recognition Receptors | structural pattern matchers |
| MHC presentation | `#[presents(antigen)]` |
| T-cell receptors | named-failure-class fingerprints |
| Antibody | failing-as-passing test, structural-pattern proptest |
| B-cell memory | `#[antigen(name = "...")]` declarations persisting past specific bugs |
| Inheritance through composition | `#[descended_from]` propagation |
| Vaccination | `cargo antigen vaccinate` applying immunity to a structural family |
| Tolerance / autoimmunity check | distinguishing legitimate code from over-flagging |

## What this is NOT

- Not a documentation system. Documentation drifts; antigen declarations are checked by tooling.
- Not a replacement for tests, lints, deprecations, or formal verification. Antigen *composes* existing Rust ecosystem tools into a coherent immune-system surface.
- Not a logic-bug catcher. Antigen catches *failure-classes that have been named*; it does not detect novel logic errors.

## Project structure

```
antigen/                 — workspace root
├── antigen/             — core crate (macros, witness types, recognition primitives)
├── cargo-antigen/       — cargo subcommand (scan, new, vaccinate, audit)
└── docs/expedition/     — design documents
    ├── design-intent.md
    ├── api-shape.md
    ├── revolutionary-and-not.md
    └── team-briefing.md
```

## Why now

- **Post-COVID vocabulary**: "antigen" is everyday language. The biological metaphor is universally accessible.
- **Mature Rust ecosystem**: cargo extensions, proc-macros, custom diagnostics, and proptest are all stable.
- **AI-coding era**: agents lose context between sessions. Implicit memory of failure patterns is no longer a viable strategy. Memory must be structural.

## License

Dual-licensed under MIT or Apache-2.0.

## Contributing

The project is in active design. Issues and discussion welcome — see the GitHub repository.
Code contribution is premature until the design phase resolves.

## Status

Reserved on crates.io: [`antigen`](https://crates.io/crates/antigen), [`cargo-antigen`](https://crates.io/crates/cargo-antigen) — version `0.0.1` placeholders signaling intent.

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

## Quick start (Layer 1 — minimum viable)

Per ADR-009 (adoption gradient), antigen meets your project at whatever
discipline level it has. The smallest viable declaration uses just `name`
and `fingerprint`:

```rust,ignore
use antigen::{antigen, immune, presents};

#[antigen(
    name = "panicking-in-drop",
    fingerprint = "impl Drop with unwrap/expect/panic in body",
)]
pub struct PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for SomeType { /* could panic */ }

#[immune(PanickingInDrop, witness = no_panic_in_drop_test)]
impl Drop for SafeType { /* verified safe */ }
```

That's it — two required fields, one of three macros applied as needed.
No project-wide ADR registry required. No bureaucracy.

Then run `cargo antigen scan` to find unaddressed presentations across
your codebase, or `cargo antigen audit` to check that every immunity
claim has a working witness. Layer 2 adds `family`, `summary`, and
`references` (open-vocabulary cross-refs to URLs, issues, ADRs, CVEs,
or anything else). Layer 3 (planned) supports structured ADR registry
cross-references.

## What this is NOT

- Not a documentation system. Documentation drifts; antigen declarations are checked by tooling.
- Not a replacement for tests, lints, deprecations, or formal verification. Antigen *composes* existing Rust ecosystem tools into a coherent immune-system surface.
- Not a logic-bug catcher. Antigen catches *failure-classes that have been named*; it does not detect novel logic errors.

## Project structure

```
antigen/                              workspace root
├── antigen/                          core crate (macros, witness types, recognition primitives)
├── cargo-antigen/                    cargo subcommand (scan, new, vaccinate, audit)
├── docs/
│   ├── origin.md                     the WHY — post-mortem narrative motivating the project
│   ├── decisions.md                  ratified ADRs (foundational ADR-001 through ADR-008)
│   ├── process.md                    formal ADR lifecycle and governance
│   ├── glossary.md                   vocabulary anchor
│   └── expedition/
│       ├── design-intent.md          what antigen IS, what it ISN'T, why now
│       ├── api-shape.md              sketch of macros and cargo subcommands
│       ├── revolutionary-and-not.md  honest claims and limits
│       ├── failure-class-instances.md  real-world Rust ecosystem instances of the 8 classes
│       ├── ecosystem-composition.md  composition opportunities with existing Rust tools
│       ├── academic-context.md       relationship to existing academic work
│       ├── inheritance-from-tambear.md  disciplines and patterns inherited from tambear
│       ├── team-briefing.md          for the JBD team at spawn time
│       └── HANDOFF.md                pre-team scaffolding hand-off summary
├── CONTRIBUTING.md                   how to contribute (design phase guidelines)
├── CODE_OF_CONDUCT.md                Rust Code of Conduct adoption
├── SECURITY.md                       security disclosure policy
└── CHANGELOG.md                      version history (Keep-a-Changelog format)
```

## Read first

If you've never heard of antigen before, read in this order:

1. **[`docs/origin.md`](docs/origin.md)** — the post-mortem narrative. The story of the
   tambear `DeterminismClass` failure that healed once, the same failure showing up
   months later in `CommutativityClass`, and how that became this project.
2. **[`docs/expedition/design-intent.md`](docs/expedition/design-intent.md)** — what
   antigen IS, what it ISN'T, why now.
3. **[`docs/expedition/revolutionary-and-not.md`](docs/expedition/revolutionary-and-not.md)** —
   honest assessment of what's genuinely new vs. existing-tools-recomposed.

If you're an architect interested in API surface:

- **[`docs/expedition/api-shape.md`](docs/expedition/api-shape.md)** — sketch of macros, cargo subcommands, witness types
- **[`docs/expedition/ecosystem-composition.md`](docs/expedition/ecosystem-composition.md)** — how antigen delegates to existing Rust tools
- **[`docs/decisions.md`](docs/decisions.md)** — ratified ADRs
- **[`docs/process.md`](docs/process.md)** — formal ADR lifecycle and governance

If you're an academic or researcher:

- **[`docs/expedition/academic-context.md`](docs/expedition/academic-context.md)** — relationship to refinement types, design-by-contract, named-effect type systems, and the Rust verification cohort

## Why now

- **Post-COVID vocabulary**: "antigen" is everyday language. The biological metaphor is universally accessible.
- **Mature Rust ecosystem**: cargo extensions, proc-macros, custom diagnostics, and proptest are all stable.
- **AI-coding era**: agents lose context between sessions. Implicit memory of failure patterns is no longer a viable strategy. Memory must be structural.

## Contributing

The project is in active design. The most valuable contributions right now:

- **Design feedback** on the substrate documents
- **Prior-art surfacing** — tools and papers we should know about
- **Failure-class proposals** — real-world Rust failures that fit (or refine) the 8 classes
- **Antigen-stdlib candidates** — specific patterns to bundle in the eventual stdlib library

See **[`CONTRIBUTING.md`](CONTRIBUTING.md)** for detail. Code PRs against the placeholder
crates are unlikely to land until v0.1.

## License

Dual-licensed under MIT or Apache-2.0.

## Status

- Reserved on crates.io: [`antigen`](https://crates.io/crates/antigen), [`cargo-antigen`](https://crates.io/crates/cargo-antigen) — version `0.0.1` placeholders
- Repository: [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen)
- CI: cargo check + test + fmt + clippy + doc on every push and PR

## Acknowledgments

The originating insight came from the adversarial agent on the [tambear](https://github.com/tambear-rs/tambear)
project's 2026-05-06 cleanup expedition. The frame shift to immune-system architecture
came from the project lead. The naming, three-verb framing, taxonomy, and design substrate
emerged in pre-team scaffolding conversation. See [`docs/origin.md`](docs/origin.md) for
the full story.

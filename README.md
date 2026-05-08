# Antigen

**Structural memory of failure-classes for Rust. Make implicit immunity explicit.**

> **Status**: v0.1.0-rc.1 — release candidate. The substrate from Sweep A2
> (W1–W8: property tests, trybuild fixtures, item-identity scan,
> span-aware errors, structural witness detection, fingerprint grammar,
> witness-tier gradient with `WitnessTier`/`AuditHint`, `#[antigen_tolerance]`,
> and idiomatic-refinement pass) is shipped. Pinning `=0.1.0-rc.1` lets
> tambear and other early adopters validate before v0.1.0 final. See
> [`CHANGELOG.md`](CHANGELOG.md) for the full v0.1.0-rc.1 manifest.

---

## The gap antigen fills

The Rust safety story currently offers three structural properties:

- **Memory safety** — no use-after-free, no buffer overflow
- **Type safety** — the type system catches mismatches
- **Thread safety** — no data races

Antigen adds a fourth:

- **Domain-knowledge-memory safety** — the lessons learned about WHY classes of code
  fail persist structurally, propagate through inheritance, and survive across
  developer turnover, AI agent context cycling, time, and refactors.

That last property has historically been structurally unsafe. Lessons live in:

- The developer's mind (lost when they leave or context cycles)
- The current AI session's context window (lost the moment session compacts)
- One git commit message (decays into archive nobody reads)
- Code review comments (lost when the PR closes)
- A docstring that drifts from the code it described
- Post-mortems on platforms that may not exist in five years

None of those carriers are drift-resistant. Antigen makes domain-knowledge-memory
*safe* — the WHY lives in the type system, propagates through `#[descended_from]`
inheritance, and is structurally checkable by `cargo antigen scan`.

This is structurally as significant a gap as the gap testing-as-practice filled.
Before testing, code worked or didn't and lessons were tribal. After testing
became standard practice, an entire industry of frameworks and methodology grew
around it. Antigen is the first instantiation of structural failure-class memory
as standard practice for a programming language ecosystem.

---

## The shape of the answer

Three verbs:

- **Build** an antigen — declare a failure-class with a structural fingerprint
- **Give** an antigen — mark code as presenting (vulnerable to) a known failure-class
- **Find** antigens — let cargo tooling scan the codebase and flag undefended sites

Plus inheritance via `#[descended_from]` propagation, witness pluralism (test /
proptest / clippy / kani / prusti / verus / phantom-type), and audit-time
witness validation with a tier gradient (`WitnessTier`: Reachability / Execution
/ FormalProof, with BehavioralAlignment reserved).

The biological metaphor maps cleanly — and is **load-bearing, not decorative**:

| Biology | Rust ecosystem analog |
|---|---|
| Pathogen Recognition Receptors (PRRs) | structural pattern matchers in `cargo antigen scan` |
| MHC Class I/II presentation | `#[presents(antigen)]` |
| T-cell receptors | named failure-class fingerprints |
| Antibody | failing-as-passing test, structural-pattern proptest, phantom-type witness, lint reference |
| B-cell memory (pattern layer) | `#[antigen(name = "...")]` declarations persisting past specific bugs |
| Antibody titer (currency layer) | `verified_at` claims; ADR-016 territory |
| B-cell lineage (clonal expansion) | `#[descended_from]` propagation |
| Vaccination | `cargo antigen vaccinate` applying immunity to a structural family |
| Peripheral tolerance / Tregs | `#[antigen_tolerance]` for legitimate fingerprint matches |
| Innate vs adaptive immunity | passive surface (fingerprint scan) vs active surface (explicit markers) |

Five+ macros, two surfaces, one architecture. See [`docs/scope.md`](docs/scope.md)
for the comprehensive vision including future immune-system primitives the
project will instantiate as adoption surfaces them.

---

## Quick start (Layer 1 — minimum viable)

Per ADR-009 (adoption gradient), antigen meets your project at whatever
discipline level it has. The smallest viable declaration uses just `name`
and `fingerprint`:

```rust,ignore
use antigen::{antigen, immune, presents};

#[antigen(
    name = "panicking-in-drop",
    fingerprint = "item: impl, has_method('drop'), body_contains_macro('panic')",
)]
pub struct PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for SomeType { /* could panic */ }

#[immune(PanickingInDrop, witness = no_panic_in_drop_test)]
impl Drop for SafeType { /* verified safe */ }
```

That's it — two required fields, three macros applied as needed. No
project-wide ADR registry required. No bureaucracy.

Then run:

- `cargo antigen scan` — find unaddressed presentations across your codebase
- `cargo antigen audit` — verify every immunity claim has a working witness
  at the appropriate tier (Reachability / Execution / FormalProof)
- `cargo antigen vaccinate <antigen> <pattern>` — apply known immunity across
  a structural family (planned)

Layer 2 adds `family`, `summary`, and `references` (open-vocabulary
cross-references to URLs, issues, ADRs, CVEs, blog posts, anything). Layer 3
supports structured ADR registry cross-references for projects with formal
decision discipline.

---

## Adoption ergonomics

Like a linter: install the cargo subcommand, get value immediately. Default
behavior on a fresh codebase:

- `cargo antigen scan` runs against your whole workspace, returns clean if
  nothing matches (no antigens declared yet)
- Add `antigen-stdlib` as a dev-dependency (planned, post-v0.1) and gain
  immunity to common Rust failure-classes without authoring any antigens
  yourself — the way `clippy` ships default lints

Customization deepens the value:

- Author project-specific antigens for your domain's failure-classes
- Cross-reference your team's ADRs / DECs / GitHub issues via the
  open-vocabulary `references` field
- Configure `[package.metadata.antigen]` for severity, scope, and
  registry validation
- Future: rust-analyzer plugin surfaces fingerprint matches inline as you type

Low friction OOTB. Comprehensive when worked.

---

## What this is NOT

This is the part that matters most for understanding what's new:

- **Not "another testing tool"** or "more tests" or "TDD with extra steps." Tests
  verify *this code does X*; antigen captures *this class of code has historically
  failed in this structural way*. Different artifact, different lifecycle, different
  contribution to engineering practice.
- **Not "another linter"** in the sense of catching style or common mistakes.
  Clippy has that covered. Antigen catches *named failure-class patterns* with
  structural fingerprints + delegated witness validation. It composes WITH
  clippy (clippy lint references are valid witness types).
- **Not a documentation system.** Documentation drifts; antigen declarations
  are checked by cargo tooling. Drift fails the build at scan time.
- **Not a replacement for tests, lints, deprecations, or formal verification.**
  Antigen *composes* them — witness pluralism delegates to whichever tool
  proves immunity for a given antigen. Compose, don't compete (ADR-002).
- **Not a logic-bug catcher.** Antigen catches *failure-classes that have been
  named*; it does not detect novel logic errors. The library of antigens grows
  when humans + AI agents recognize failure-classes; the tooling cannot predict
  patterns nobody has named yet.
- **Not a fine-tuning or training-data alternative.** Lessons live in code,
  inspectable and version-controlled. They propagate to *any* AI model or human
  reading the codebase, not just to fine-tuned weights. A new LLM picks up
  antigen declarations like any other code; the lessons travel structurally.

---

## What's actually shipped

As of v0.1.0 substrate (release imminent):

- The four core macros: `#[antigen]`, `#[presents]`, `#[immune]`, `#[descended_from]`
- Plus `#[antigen_tolerance]` (ADR-011) for legitimate fingerprint matches
- `cargo antigen scan` with item-identity matching and the 5-state interaction
  matrix (marked + matched, passively detected, inconsistent, tolerated, stale-tolerance)
- `cargo antigen audit` with `WitnessTier` gradient — tier-honest reporting
  per ADR-005 Amendment 3 (no more is_well_formed boolean; tier-aware audit
  output)
- Fingerprint grammar v1: six item-level operators (`item`, `name`,
  `variants`, `has_method`, `attr_present`, `doc_contains`) plus
  composition (`all_of`, `any_of`, `not`)
- Phantom-type witness recognition (ADR-013): `Witnessed<T,W>`, `typewit::TypeEq`,
  hand-rolled `PhantomData<T>` shapes recognized at FormalProof tier
- Property tests over both parser surfaces; trybuild fixtures for compile-error paths
- Span-aware error messages that underline the offending literal

Future sweeps (per [`sweeps/`](sweeps/)):

- W6b: body-level fingerprint operators via ast-grep subprocess (ADR-015)
- W8: idiomatic refinement
- W9: v0.1.0-rc.1 → v0.1.0 release prep
- A3: cross-crate scan + `#[descended_from]` propagation across workspaces
- A4: composition rules + witness-type pluralism completion (kani/prusti/verus/creusot harness invocation)
- A5: `cargo antigen vaccinate` + audit-extension + antigen-stdlib v0.1 (10-20 stdlib antigens covering all 8 first-principles failure classes)
- A6: rust-analyzer plugin / IDE integration (real-time fingerprint match surfacing as you type)

---

## Project structure

```
antigen/                              workspace root
├── antigen/                          core crate (macros, scan, audit, fingerprint)
├── antigen-macros/                   proc-macro crate (re-exported through antigen)
├── antigen-fingerprint/              fingerprint grammar v1 (Path C: syn tokenizer + ParseBuffer)
├── cargo-antigen/                    cargo subcommand (scan, audit; new + vaccinate planned)
├── docs/
│   ├── origin.md                     the WHY — post-mortem narrative motivating the project
│   ├── scope.md                      comprehensive scope + the full vision
│   ├── decisions.md                  ratified ADRs (ADR-001 through ADR-016 + amendments)
│   ├── postures.md                   informational catalog of architectural postures
│   ├── process.md                    formal ADR lifecycle and governance
│   ├── glossary.md                   vocabulary anchor
│   └── expedition/                   design substrate (pre-team scaffolding through current sweeps)
├── sweeps/
│   ├── A1-design-ratification/       Sweep A1 closure narrative + the four empirical validations
│   └── A2-core-macros/               Sweep A2 scope-lock + closure (when sweep closes)
└── ...                               (CONTRIBUTING.md, LICENSE-MIT, LICENSE-APACHE, .github/, etc.)
```

---

## Read first

If you've never heard of antigen before:

1. **[`docs/origin.md`](docs/origin.md)** — the post-mortem narrative. The
   tambear `DeterminismClass` polarity-inversion incident (GAP-BIT-EXACT-1)
   and how the same failure pattern almost shipped a second time months later
   in `CommutativityClass`. The garden entry that named the pattern. The
   project that came out of it.
2. **[`docs/scope.md`](docs/scope.md)** — comprehensive scope: what antigen
   does today, what it's becoming, the full immune-system metaphor expansion,
   adoption flywheel, AI-industry implications.
3. **[`sweeps/A1-design-ratification/CLOSURE.md`](sweeps/A1-design-ratification/CLOSURE.md)** —
   the four empirical validations Sweep A1 produced (events, discipline,
   biology-as-search-heuristic, coordination-tier substrate-over-memory).

If you're an architect interested in API surface:

- **[`docs/decisions.md`](docs/decisions.md)** — ratified ADRs through ADR-016 + amendments
- **[`docs/postures.md`](docs/postures.md)** — architectural postures threaded through the ADRs
- **[`docs/process.md`](docs/process.md)** — formal ADR lifecycle and governance

If you're a researcher or practitioner:

- **[`docs/expedition/academic-context.md`](docs/expedition/academic-context.md)** —
  relationship to refinement types, design-by-contract, named-effect type
  systems, and the Rust verification cohort
- **[`docs/expedition/ecosystem-composition.md`](docs/expedition/ecosystem-composition.md)** —
  how antigen composes with kani, prusti, verus, creusot, clippy, proptest,
  cargo-mutants, miri, and the broader Rust verification ecosystem

---

## Why now

- **Post-COVID vocabulary**: "antigen," "antibody," "vaccination" are everyday
  language; the biological metaphor is universally accessible and emotionally
  grounded.
- **Mature Rust ecosystem**: cargo extensions, proc-macros, custom diagnostics,
  proptest, ast-grep, and the formal-verification cohort (kani/prusti/verus/creusot/flux)
  are all stable enough to compose with rather than reinvent.
- **AI-coding era**: agents lose context between sessions. Implicit memory of
  failure patterns is no longer viable; structural memory is required for
  AI-assisted development at scale.
- **Convergence across fields**: biological immunology, programming-language theory
  (Hoare 1969 → Eiffel 1992 → Liquid Haskell → Flux), pre-project AI gardening
  (March-April 2026 entries predicting frame-translation as a category), and 2026
  ML graph-memory research have all been converging on structural-memory-with-
  recognition-and-inheritance as a core architecture. Antigen is the Rust-domain
  instantiation that names it and ships it.

---

## Contributing

The project is in active build (sweep A2 in flight as of 2026-05-08). Most
valuable contributions right now:

- **Real-world failure-class proposals** — Rust failures that fit (or refine)
  the 8 first-principles classes
- **Antigen-stdlib candidates** — specific patterns to bundle in the eventual
  stdlib library (post-v0.1)
- **Witness type integrations** — kani/prusti/verus/creusot/flux harness
  recognition refinements
- **Adoption feedback** — once v0.1.0 lands, real-world adoption signal from
  Rust workspaces

See **[`CONTRIBUTING.md`](CONTRIBUTING.md)** for detail.

---

## License

Dual-licensed under MIT or Apache-2.0. See [`LICENSE-MIT`](LICENSE-MIT) and
[`LICENSE-APACHE`](LICENSE-APACHE).

---

## Status

- crates.io: [`antigen`](https://crates.io/crates/antigen),
  [`cargo-antigen`](https://crates.io/crates/cargo-antigen),
  [`antigen-macros`](https://crates.io/crates/antigen-macros),
  [`antigen-fingerprint`](https://crates.io/crates/antigen-fingerprint) —
  v0.1.0-rc.1 (release candidate; v0.1.0 final tracking after rc validation)
- Repository: [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen)
- CI: cargo check + test + fmt + clippy (-D warnings) + doc (-D warnings) on
  every push and PR
- Tests: 187 passing, 21 ignored across the workspace as of A2 (W1-W8) closure
- Tambear integration: live as of 2026-05-07 (commit `80a19b4` in tambear);
  exploratory adoption with three seed antigens

---

## Acknowledgments

The originating insight came from the adversarial agent on the
[tambear](https://github.com/tambear-rs/tambear) project's 2026-05-06 cleanup
expedition. The frame shift to immune-system architecture came from the project
lead. The naming, three-verb framing, taxonomy, and design substrate emerged in
pre-team scaffolding conversation. Sweep A1 (design ratification) and Sweep A2
(core macros + WitnessTier + fingerprint grammar v1) ratified five amendments,
ADR-015 (engine architecture), ADR-016 (temporal recognition surface), and
produced the four empirical validations + scale-invariance + colonization-ratio
empirical defenses captured in [`sweeps/A1-design-ratification/CLOSURE.md`](sweeps/A1-design-ratification/CLOSURE.md).
See [`docs/origin.md`](docs/origin.md) for the founding incident; [`docs/scope.md`](docs/scope.md)
for the full vision.

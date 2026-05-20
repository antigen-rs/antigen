# Antigen

**Structural memory of failure-classes for Rust. Make implicit immunity explicit.**

> **Status**: v0.1.0-rc.1 — release candidate. Core macros, fingerprint
> grammar, scan + audit, witness-tier gradient, cross-crate scanning,
> `#[descended_from]` propagation with diamond dedup all shipped.
> Pinning `=0.1.0-rc.1` lets early adopters validate before v0.1.0
> final. See [`CHANGELOG.md`](CHANGELOG.md) for the full manifest.

---

## What antigen is

When you fix a bug, you learn something about *why* a class of code
fails. That lesson lives in your head, in a commit message, in a Slack
thread, in a docstring that drifts as the code evolves. None of those
carriers are drift-resistant. Six months later, structurally identical
code appears in another module — and the lesson is gone.

Antigen makes the lesson **structural**: declared in code, propagated
through inheritance, checked by cargo tooling. The lesson survives
developer turnover, AI agent context cycling, time, and refactors —
because it lives in the type system, not in human memory.

```rust
#[antigen(
    name = "panicking-in-drop",
    fingerprint = r#"item = impl, has_method("drop", "(& mut self)"), body_contains_macro("panic")"#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
)]
pub struct PanickingInDrop;
```

That declaration is the failure-class memory. `cargo antigen scan`
finds every site in your codebase that structurally matches it.
`cargo antigen audit` verifies that every claimed immunity has a
working witness. The lesson is now durable substrate, not implicit
knowledge.

---

## What this is NOT

This is the part that matters most for understanding what's new:

- **Not "another testing tool"** or "more tests" or "TDD with extra steps."
  Tests verify *this code does X*; antigen captures *this class of code
  has historically failed in this structural way*. Different artifact,
  different lifecycle, different contribution to engineering practice.
- **Not "another linter"** in the sense of catching style or common
  mistakes. Clippy has that covered. Antigen catches *named failure-class
  patterns* with structural fingerprints + delegated witness validation.
  It composes WITH clippy (clippy lint references are valid witness
  types).
- **Not a documentation system.** Documentation drifts; antigen
  declarations are checked by cargo tooling. Drift fails the build at
  scan time.
- **Not a replacement for tests, lints, deprecations, or formal
  verification.** Antigen *composes* them — witness pluralism delegates
  to whichever tool proves immunity for a given antigen.
- **Not a logic-bug catcher.** Antigen catches *failure-classes that
  have been named*; it does not detect novel logic errors.
- **Not a fine-tuning or training-data alternative.** Lessons live in
  code, inspectable and version-controlled. They propagate to *any* AI
  model or human reading the codebase, not just to fine-tuned weights.

Antigen is a **third pillar** alongside testing and documentation. Both
of those filled gaps that structurally couldn't be filled with the
practices that preceded them. Antigen fills the gap of structural
failure-class memory, which testing and documentation have historically
tried to address as side-effects of their primary jobs.

---

## The gap antigen fills

The Rust safety story currently offers three structural properties:

- **Memory safety** — no use-after-free, no buffer overflow
- **Type safety** — the type system catches mismatches
- **Thread safety** — no data races

Antigen adds a fourth:

- **Domain-knowledge-memory safety** — the lessons learned about WHY
  classes of code fail persist structurally, propagate through
  inheritance, and survive across developer turnover, AI agent context
  cycling, time, and refactors.

That fourth property has historically been structurally unsafe. Lessons
live in:

- The developer's mind (lost when they leave or context cycles)
- The current AI session's context window (lost the moment session
  compacts)
- One git commit message (decays into archive nobody reads)
- Code review comments (lost when the PR closes)
- A docstring that drifts from the code it described
- Post-mortems on platforms that may not exist in five years

None of those carriers are drift-resistant. Antigen makes
domain-knowledge-memory *safe* — the WHY lives in the type system,
propagates through `#[descended_from]` inheritance, and is structurally
checkable by `cargo antigen scan`.

---

## Install and first scan

```sh
cargo install cargo-antigen
```

Run `cargo antigen scan` in any Rust project. On a fresh codebase with
no antigen declarations yet:

```
Scanning workspace: .

Scanned 0 files, found 0 antigen-related declarations.
```

Add antigen as a dependency:

```toml
[dependencies]
antigen = "=0.1.0-rc.1"
```

Now you can declare your first antigen. The full walkthrough lives in
[`docs/tutorial.md`](docs/tutorial.md) — your first 15 minutes,
end-to-end, with a real failure-class.

---

## The vocabulary

Five macros form antigen's core vocabulary:

- **`#[antigen(...)]`** — declare a named failure-class with a structural
  fingerprint
- **`#[presents(AntigenName)]`** — mark code as vulnerable to a known
  failure-class
- **`#[immune(AntigenName, witness = ...)]`** — claim the code is
  protected, naming the witness (test, proptest, formal proof, lint,
  phantom-type) that verifies it
- **`#[descended_from(Parent)]`** — declare structural inheritance
  between failure-classes
- **`#[antigen_tolerance(AntigenName, rationale = ...)]`** — explicitly
  tolerate a fingerprint match the team has reviewed

Plus two cargo subcommands:

- **`cargo antigen scan`** — find unaddressed presentations across your
  codebase
- **`cargo antigen audit`** — verify every immunity claim has a working
  witness at the appropriate tier (Reachability / Execution /
  FormalProof)

---

## The biology cognate

The biological metaphor is **load-bearing, not decorative**. The
project's design has consistently emerged from immunological structure:

| Biology | Rust ecosystem analog |
|---|---|
| Pathogen Recognition Receptors (PRRs) | structural pattern matchers in `cargo antigen scan` |
| MHC Class I/II presentation | `#[presents(antigen)]` |
| T-cell receptors | named failure-class fingerprints |
| Antibody | test, proptest, phantom-type witness, lint reference |
| B-cell memory (pattern layer) | `#[antigen(name = "...")]` declarations |
| Antibody titer (currency layer) | `verified_at` (ADR-016) |
| B-cell lineage (clonal expansion) | `#[descended_from]` propagation |
| Peripheral tolerance / Tregs | `#[antigen_tolerance]` for legitimate matches |
| Innate vs adaptive immunity | passive surface (fingerprint scan) vs active surface (explicit markers) |
| Antigenic drift / shift | version-boundary recognition (ADR-017) |

When the biology predicts a primitive, the project builds it. When the
biology breaks, the project names where and refines. See
[`docs/decisions.md`](docs/decisions.md) (ADR-003) for the discipline.

---

## Multi-component view

Antigen is not a single tool — it's a vocabulary that lets you compose
multiple kinds of structural immunity. Adopting at the floor gets you
one component (the linter); growing into deeper composition unlocks
more.

The seven components currently named (the enumeration is open and may
extend):

1. **Dev-in-the-loop** — you write antigens by hand based on judgment.
   Floor value: structural memory exists.
2. **Passive scan/lint/tool** — automated walks find antigens, audit
   verifies witnesses. Cargo subcommand. Linter-mode adoption.
3. **Test integration** — witnesses link to actual tests; test history
   becomes immune history; audit reports verification tier honestly.
4. **Knowledge-ecosystem integration** — references attach antigens to
   PRs, ADRs, CVEs, papers, post-mortems. The bridge to your team's
   knowledge substrate.
5. **Cross-version / lineage** — `#[descended_from]` chains, temporal
   recognition surface, version-boundary handling.
6. **Cross-crate / ecosystem** — antigens propagate across crate
   boundaries via cargo metadata; future `antigen-stdlib` will share
   ecosystem-level failure-class memory.
7. **Real-time / CI feedback** — PR-scope diff against scan baseline,
   inline annotations, recognition at the moment of change. Future
   sweep.

You can adopt at any component composition. A team using only
components 1 and 2 (dev + linter) gets real value. Adding component 3
(test integration) extends it. Each layer multiplies leverage without
requiring the others.

The deeper architectural framing lives in
[`docs/expedition/multi-component-immunity.md`](docs/expedition/multi-component-immunity.md)
(currently maturing as expedition substrate; expected to canonicalize
post-A3.5 sweep close).

---

## What's actually shipped in v0.1.0-rc.1

**Core macros**:
- `#[antigen]`, `#[presents]`, `#[immune]`, `#[descended_from]`,
  `#[antigen_tolerance]`
- `requires = <predicate>` parameter on `#[immune]` for substrate-witness
  predicates (ADR-019)
- `attested = (who, allowed_types, why, scope)` parameter on any macro
  for cross-cutting review attestation (ADR-020)

**Cargo subcommands**:
- **`cargo antigen scan`** — item-identity fingerprint matching,
  cross-crate scanning, cycle detection, diamond inheritance dedup
- **`cargo antigen audit`** — `WitnessTier` gradient
  (None / Reachability / Execution / FormalProof) with three-axis
  output: `WitnessTier × AuditHint × EvidenceKind` (ADR-019)
- **`cargo antigen attest`** — substrate-witness sidecar management:
  `scaffold`, `sign`, `check`, `delta`, `list`, `gc`
- **`cargo antigen tolerate`** — tolerance-ratification sidecar
  management: `scaffold`, `sign`, `check`, `list`

**Fingerprint grammar v1**: seven item-level operators (`item`,
`name`, `variants`, `has_method`, `attr_present`, `doc_contains`,
`body_contains_macro`) plus composition (`all_of`, `any_of`, `not`).
Full reference at [`docs/fingerprint-grammar.md`](docs/fingerprint-grammar.md).

**Substrate-witness predicate language** (ADR-019): closed combinator
grammar (`all_of`, `any_of`, `not`) over five sealed leaf primitives
(`ratified_doc`, `signers`, `signed_trailer`, `oracles_complete`,
`fresh_within_days`). JSON sidecars at `.attest/<Antigen>.json`.

**Anti-laundering safeguards**: `attest delta` enforces chain-depth
cap (default 3), minimum rationale character count, and tracks
cumulative-root fingerprint for drift detection.

**Phantom-type witness recognition** (ADR-013): turbofish syntax
recognized as FormalProof-tier witnesses.

**Cross-crate identity** at `name@version` granularity (ADR-017).

**`#[descended_from]` propagation** with tagged synthesis, diamond
dedup, and 7-state inheritance state matrix (ADR-018).

**495 tests passing** (31 ignored as pre-impl contracts) across the
workspace; property-test + trybuild + adversarial precision tests;
span-aware error messages.

Not yet shipped (hidden from CLI with honest stubs):

- `cargo antigen attest oracle` — oracle completion markers (blocked on
  ADR-021 OracleRef generalization ratification)
- `cargo antigen attest migrate` — resolves to no-op once ADR-021
  additive-only schema evolution ratifies
- `cargo antigen new` — scaffold an antigen declaration
- `cargo antigen vaccinate` — bulk-apply immunity across a structural family
- `antigen-stdlib` — ecosystem-wide failure-class library

See [`docs/roadmap.md`](docs/roadmap.md) for the planned trajectory
through v0.2+ and beyond.

---

## Where to go next

**If you're getting started:**

- [`docs/tutorial.md`](docs/tutorial.md) — your first 15 minutes with
  antigen, end-to-end
- [`docs/where-to-look-for-antigens.md`](docs/where-to-look-for-antigens.md) —
  conventions for locating antigen declarations in your project
- [`docs/usage-patterns.md`](docs/usage-patterns.md) — common patterns
  for applying antigen's vocabulary to real failure-classes

**If you want reference docs:**

- [`docs/macros.md`](docs/macros.md) — full reference for all five
  attribute macros with syntax, examples, and discipline notes
- [`docs/witness-tiers.md`](docs/witness-tiers.md) — WitnessTier
  gradient reference: what each tier means and when it applies
- [`docs/output-formats.md`](docs/output-formats.md) — scan/audit
  output reference: human-readable and JSON field-by-field
- [`docs/fingerprint-grammar.md`](docs/fingerprint-grammar.md) — full
  fingerprint DSL reference
- [`docs/glossary.md`](docs/glossary.md) — vocabulary anchor for every
  project term
- [`CHANGELOG.md`](CHANGELOG.md) — what shipped in this release

**If you want the architecture:**

- [`docs/origin.md`](docs/origin.md) — the post-mortem narrative that
  motivated the project (tambear's `DeterminismClass` /
  `CommutativityClass` incident)
- [`docs/scope.md`](docs/scope.md) — comprehensive vision; multi-paper
  publication trajectory; immune-system primitive map
- [`docs/decisions.md`](docs/decisions.md) — ratified ADRs (through
  ADR-018) and amendments
- [`docs/postures.md`](docs/postures.md) — architectural postures
  threaded through the ADRs
- [`docs/process.md`](docs/process.md) — formal ADR lifecycle and
  governance

**If you're a researcher or practitioner:**

- [`docs/expedition/`](docs/expedition/) — design substrate, including
  multi-component immunity, cross-domain architectural map (16+ academic
  fields), and the encounters discipline

---

## Adoption ergonomics

Like a linter: install the cargo subcommand, get value immediately.
Default behavior on a fresh codebase:

- `cargo antigen scan` runs against your whole workspace, returns clean
  if nothing matches (no antigens declared yet)
- Add `antigen-stdlib` as a dev-dependency (planned, post-v0.1) and
  gain immunity to common Rust failure-classes without authoring any
  antigens yourself

Customization deepens the value:

- Author project-specific antigens for your domain's failure-classes
- Cross-reference your team's ADRs / DECs / GitHub issues via the
  open-vocabulary `references` field
- Configure `[package.metadata.antigen]` for severity, scope, and
  registry validation
- Future: rust-analyzer plugin surfaces fingerprint matches inline as
  you type

Low friction OOTB. Comprehensive when worked.

---

## Why now

- **Post-COVID vocabulary**: "antigen," "antibody," "vaccination" are
  everyday language; the biological metaphor is universally accessible.
- **Mature Rust ecosystem**: cargo extensions, proc-macros, custom
  diagnostics, proptest, ast-grep, and the formal-verification cohort
  (kani/prusti/verus/creusot/flux) are all stable enough to compose
  with rather than reinvent.
- **AI-coding era**: agents lose context between sessions. Implicit
  memory of failure patterns is no longer viable; structural memory is
  required for AI-assisted development at scale.
- **Convergence across fields**: biological immunology, programming-
  language theory (Hoare 1969 → Eiffel 1992 → Liquid Haskell → Flux),
  pre-project AI gardening (March-April 2026 entries predicting
  frame-translation as a category), and 2026 ML graph-memory research
  have all been converging on structural-memory-with-recognition-and-
  inheritance as a core architecture. Antigen is the Rust-domain
  instantiation.

---

## Contributing

The project is in active build (Sweep A3.5 — Onboarding — in flight
as of 2026-05-11). Most valuable contributions right now:

- **Real-world failure-class proposals** — Rust failures that fit (or
  refine) the project's growing taxonomy
- **Antigen-stdlib candidates** — specific patterns to bundle in the
  eventual stdlib library (post-v0.1)
- **Witness type integrations** — kani/prusti/verus/creusot/flux
  harness recognition refinements
- **Adoption feedback** — once v0.1.0 lands, real-world adoption
  signal from Rust workspaces

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for detail.

---

## License

Dual-licensed under MIT or Apache-2.0. See [`LICENSE-MIT`](LICENSE-MIT)
and [`LICENSE-APACHE`](LICENSE-APACHE).

---

## Status

- crates.io: [`antigen`](https://crates.io/crates/antigen),
  [`cargo-antigen`](https://crates.io/crates/cargo-antigen),
  [`antigen-macros`](https://crates.io/crates/antigen-macros),
  [`antigen-fingerprint`](https://crates.io/crates/antigen-fingerprint)
  — v0.1.0-rc.1 (release candidate; v0.1.0 final tracking after rc
  validation and A3.5 onboarding sweep close)
- Repository:
  [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen)
- CI: cargo check + test + fmt + clippy (-D warnings) + doc
  (-D warnings) on every push and PR
- Tests: 235 passing, 27 ignored across the workspace as of A3 closure
- Tambear integration: live since 2026-05-07; first real adoption
  exercising the substrate and surfacing real bugs (see
  [`docs/expedition/tambear-adoption-log.md`](docs/expedition/tambear-adoption-log.md))

---

## Acknowledgments

The originating insight came from the adversarial agent on the
[tambear](https://github.com/tambear-rs/tambear) project's 2026-05-06
cleanup expedition. The frame shift to immune-system architecture came
from the project lead. The naming, three-verb framing, taxonomy, and
design substrate emerged in pre-team scaffolding conversation. Sweeps
A1 through A3 ratified eighteen ADRs and amendments, produced four
empirical validations (events, discipline, biology-as-instrument,
coordination-tier substrate-over-memory), surfaced the multi-component
immunity framing, and produced the encounters discipline currently
maturing as expedition substrate.

See [`docs/origin.md`](docs/origin.md) for the founding incident;
[`docs/scope.md`](docs/scope.md) for the full vision.

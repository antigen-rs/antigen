# Antigen — Vision Pitch

> **For Rust ecosystem maintainers, library authors, and tooling-aware engineers.** A
> 1500-word read explaining what antigen is, why it matters, and what we're asking
> the community to consider.

---

## The problem in one sentence

When a Rust project fixes a structural bug, the test for THAT bug ships — but the
*lesson* about the failure-class generally doesn't, so structurally-similar new code
gets bitten by the same pattern, and the team re-derives the lesson from scratch.

This is the implicit-memory failure mode. It hits every codebase, in every language.
But it hits AI-assisted Rust development especially hard, because agents lose
context between sessions; lessons that humans tacitly carry have nowhere persistent
to live. As AI-coding adoption grows, the gap between "we already learned this once"
and "we can prevent it next time" widens.

## A concrete instance from the project that motivated antigen

In April 2026, the [tambear](https://github.com/tambear-rs/tambear) project — a
Windows-native GPU-accelerated mathematical computing toolkit — discovered a polarity
inversion in its `DeterminismClass` enum's `meet` method. The discriminants were
ordered strongest-first; the lattice ordering is reverse-strictness; `meet =
std::cmp::min` therefore returned the *strongest* class instead of the weakest. The
bug was named GAP-BIT-EXACT-1 and fixed: `meet = max` is correct.

Two months later, an unrelated DEC introduced `CommutativityClass` — structurally
identical shape, independently designed, by different agents on a different team.
The polarity inversion shipped again with `meet = std::cmp::min`. The same illness,
re-derived from scratch, narrowly caught by adversarial pre-implementation
verification.

The healing didn't propagate. The lesson lived in the corrected `DeterminismClass`
file, in the GAP-BIT-EXACT-1 issue, and in dev memory. None of those reached
`CommutativityClass` until the team manually re-derived the lesson.

This is documented in [`docs/origin.md`](origin.md). It's one instance of a pattern
that recurs across every project that fixes structural bugs.

## What antigen does

Antigen makes failure-class memory **structural and inheritable** through:

- **`#[antigen(name = "...", fingerprint = "...")]`** — declare a named failure-class
  with a structural pattern that matches vulnerable code shapes.
- **`#[presents(antigen)]`** — mark code as exhibiting the structural pattern.
- **`#[immune(antigen, witness = ...)]`** — declare immunity, with a witness that
  proves the immunity by delegating to existing tools (proptest, clippy, kani,
  prusti, etc.).
- **`#[descended_from(parent)]`** — propagate antigen markers through derivation,
  copy-paste, or structural similarity.

Plus a cargo subcommand — `cargo antigen` — that scans for unaddressed presentations,
applies known immunity patterns across structural families (vaccination), and audits
overall coverage.

Concretely: had antigen existed during tambear's `DeterminismClass` fix, the team
would have declared:

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = "enum *Class with strongest-first discriminants and meet method",
    references = ["GAP-BIT-EXACT-1"],
)]
pub struct PolarityInvertedClassMeet;
```

When `CommutativityClass` was introduced months later, `cargo antigen scan` would
have flagged the structural match in CI. The pathmaker would have seen the
diagnostic, written the suggested witness proptest, watched it fail, and fixed the
polarity before the code merged.

The illness would have been cured before it appeared. The full case study is in
[`docs/expedition/case-study-determinism-class.md`](expedition/case-study-determinism-class.md).

## What's genuinely new vs synthesized from existing tools

Antigen is **architecturally a synthesis**, not a new verification technique. Most
of its primitives exist somewhere in the Rust ecosystem:

- The deprecation system handles memory of one specific kind of fix.
- clippy provides structural pattern recognition via lints.
- proptest, quickcheck, and cargo-mutants provide property-based and mutation
  witnesses.
- kani, prusti, creusot, verus provide formal verification witnesses.
- cargo-careful, miri, cargo-deny address adjacent concerns.

What antigen contributes — verified through the academic-context survey at
[`docs/expedition/academic-context.md`](expedition/academic-context.md) — is:

1. **Failure-class names as inherited first-class artifacts.** Existing tools detect
   patterns; antigen NAMES the failure class structurally and inherits the immunity
   declaration through `#[descended_from]`. This shape doesn't exist in any current
   Rust tool. Eiffel inherits predicates; CWE has names without inheritance; Koka
   inherits effects. None inherits *named failure-classes* through structural
   derivation with witness re-validation.

2. **Vaccination as a developer-facing bulk transform.** `cargo antigen vaccinate
   <antigen> <pattern>` applies known immunity across a structural family in one
   command. Closest analogs (cargo fix; Coq's `Hint Resolve`) are per-site or
   proof-internal; antigen's vaccinate is a bulk operation on the failure-class
   graph.

3. **Witness-shape pluralism under one vocabulary.** `#[immune(X, witness = ...)]`
   accepts proptest blocks, clippy lints, kani proofs, prusti annotations, phantom-
   type constructions, or test functions — all valid as witnesses for the same
   antigen. Why3's multi-prover architecture is the closest cousin but unifies under
   a single specification language; antigen unifies under failure-class names while
   leaving witness mechanisms heterogeneous.

The defensible novelty claim is *composition*, *inheritance*, *vaccination*, and
*ecosystem orchestration* — not new verification.

## Why this is timely for Rust

Three forces converge on Rust as the right substrate at the right time:

**Rich type system + mature procedural macros + cargo-extension pattern.** Antigen's
declarations are syn-based proc-macros. Its tooling is a `cargo` subcommand. The
infrastructure is stable and idiomatic.

**Strong safety culture, vibrant verification ecosystem.** Rust developers expect
tooling to enforce invariants. Adopting clippy, kani, prusti, miri without
resistance. Antigen threads this existing fabric — DELEGATES to it via witnesses —
rather than competing.

**The implicit/explicit boundary is increasingly load-bearing.** Lifetime variance,
async coloring, Send/Sync auto-traits, ownership boundaries — Rust has more
implicit-but-load-bearing structure than most languages. As Rust adoption broadens
to less Rust-fluent contributors AND as AI agents become major contributors,
making this implicit structure *legible at the failure-class level* matters more.

The "AI-coding amplifies the implicit-memory problem" claim isn't speculation. It's
visible in tambear's own development: agents pre-loading context lose access to
lessons that ratified DECs already encode. Antigen makes that knowledge structural,
which is the only viable strategy for AI-only or mixed-team development.

## Adoption pathway

We're not asking the Rust community to adopt antigen all at once. The pathway has
explicit phases:

**Phase 1 (current): namespace reservation + design substrate.** `antigen` and
`cargo-antigen` are reserved as `0.0.1` placeholders. ~25k words of design substrate
documents the architecture, ratified ADRs, and open questions. The
[antigen-rs/antigen](https://github.com/antigen-rs/antigen) repository is open for
design feedback.

**Phase 2: core macros + scan.** First functional release ships `#[antigen]`,
`#[presents]`, `#[immune]`, and `cargo antigen scan`. Early adopters write their own
antigens for project-specific failure classes. This phase is the "early Rust users"
stage — enthusiasts willing to be on the leading edge.

**Phase 3: antigen-stdlib.** A companion crate provides 20-50 ready-made antigens for
common Rust failure classes. Adoption barrier drops significantly because users get
value without writing antigens themselves. The [`docs/expedition/stdlib-seed-antigens.md`](expedition/stdlib-seed-antigens.md)
draft seeds the first 10.

**Phase 4: ecosystem composition matures.** Witness adapters for kani, prusti, verus,
cargo-mutants. IDE integration via rust-analyzer. Cross-crate antigen versioning
worked out.

**Phase 5: community library.** Projects publish their domain-specific antigens
(`tambear-antigens`, `tokio-antigens`, etc.) to crates.io. Cross-project failure-class
patterns become visible and shareable.

Each phase delivers value independently. Stalling at any phase is acceptable; the
substrate continues serving its consumers without forcing them to upgrade.

## What we're asking for

For Rust ecosystem maintainers and tooling-aware engineers:

1. **Read the design substrate** (start with [`origin.md`](origin.md) and
   [`design-intent.md`](expedition/design-intent.md), ~30 minutes total) and tell us
   where the design is wrong, over-claiming, or missing considerations.
2. **Surface prior art** we haven't covered. The
   [`academic-context.md`](expedition/academic-context.md) and
   [`ecosystem-composition.md`](expedition/ecosystem-composition.md) docs are the
   landing pages for survey gaps.
3. **Propose failure-classes** that should be in `antigen-stdlib`, with real-world
   instance evidence (not speculation). Issue templates at
   [`.github/ISSUE_TEMPLATE`](.github/ISSUE_TEMPLATE) accept these.
4. **Tell us if you'd be an early adopter.** Real adoption stories shape the project's
   priorities far more than maintainer guesses. Open a GitHub Discussion thread if
   antigen would address a pain point in your codebase.

For tool authors (clippy, kani, prusti, verus, cargo-mutants, etc.):

1. **Tell us your integration surface.** Antigen wants to delegate witnesses to
   your tool. The mechanics for `#[immune(X, witness = clippy::lint_name)]` need
   your input.
2. **Help us avoid friction at delegation boundaries.** When clippy adds new lints,
   antigen's witness adapters should track them automatically.

For AI-coding tool authors and AI-agent framework authors:

1. **Help us understand AI-coding-specific failure classes.** What patterns recur in
   agent-produced code that human-only code doesn't exhibit? These are antigen
   candidates.
2. **Consider antigen as a cross-session memory layer for your agents.** When agents
   declare `#[immune]` with witnesses, the immunity persists past their session
   boundaries. The substrate becomes shared memory.

## What we're NOT asking for

We're not asking for endorsement before evidence. We're not asking for adoption
before v0.1. We're not asking the Rust core team to adopt antigen as part of the
language. We're not competing with clippy, kani, prusti, or any existing tool —
antigen composes them.

The project is ambitious in shape but humble in claim. The composition is novel; the
underlying primitives are mostly familiar. Adoption depends entirely on engineering
quality, ergonomics, and the gradual proof that structural failure-class memory
delivers compounding value.

## In one phrase

**Antigen makes the lessons learned by one Rust project, automatically available to
every project that follows.** Not by writing better documentation. Not by mandatory
process. By moving the memory itself — from human heads, commit messages, and
documentation that drifts — into the type system, where the compiler and `cargo`
can enforce that the lessons stay applied.

The illness already healed once. Let's not heal it again next year, and the year
after, in every Rust project that ships a similar shape. Let's heal it once and
inoculate everyone.

---

## Where to read more

- **The story**: [`docs/origin.md`](origin.md) — the post-mortem narrative motivating
  the project.
- **The design**: [`docs/expedition/design-intent.md`](expedition/design-intent.md),
  [`api-shape.md`](expedition/api-shape.md),
  [`revolutionary-and-not.md`](expedition/revolutionary-and-not.md).
- **The architecture**: [`docs/decisions.md`](decisions.md) — 10 ratified ADRs.
- **The process**: [`docs/process.md`](process.md) — how decisions get drafted,
  reviewed, ratified, and govern downstream work.
- **The case study**: [`docs/expedition/case-study-determinism-class.md`](expedition/case-study-determinism-class.md)
  — full walkthrough of how antigen would have caught the originating bug pattern.
- **The seed catalog**: [`docs/expedition/stdlib-seed-antigens.md`](expedition/stdlib-seed-antigens.md)
  — 10 concrete antigens for the eventual `antigen-stdlib` v0.1.

If anything here resonates, please [open a Discussion](https://github.com/antigen-rs/antigen/discussions).

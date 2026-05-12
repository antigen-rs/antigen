# Antigen — Architectural Decision Records

> Ratified architectural decisions for the antigen project. Modeled on tambear's DEC
> registry. Every load-bearing decision should land here with a clear rationale, a
> resolves-clause, and an enforcement mechanism.
>
> **Convention**: ADR-NNN entries are added in numerical order. Each starts with a
> status (Draft / Ratified / Superseded), participants, related ADRs, finding,
> decision, mechanics, sweep-level consequences, enforcement, and resolves clauses.
> Drafts can be edited freely; ratified ADRs require explicit revision via amendment
> or supersession.

> **Note on Phase 1-8 status of ADR-001 through ADR-010** (foundational, pre-team):
> these ten ADRs were ratified by the team-lead during pre-team scaffolding (Tekgy +
> Claude in winrapids working directory, 2026-05-07) WITHOUT going through the full
> Phase 1-8 deconstruction process documented in [`process.md`](process.md). They are
> ratified-by-trust rather than ratified-by-discipline.
>
> **The JBD team's first sweep (Sweep A1)** explicitly covers Phase 1-8 deconstruction
> of these foundational ADRs by the aristotle role, with adversarial review,
> systems-research review, and scientist validation. See [`expedition/first-sweep-plan.md`](expedition/first-sweep-plan.md)
> for the concrete plan.
>
> **Implications for readers**:
> - These ADRs are TREATED as ratified for purposes of building substrate (the team
>   operates under them; downstream code can cite them; the process treats them as
>   load-bearing)
> - But they are MORE OPEN to amendment than ADRs that have been through full Phase 1-8
> - When the team's Phase 1-8 surfaces necessary refinements, expect amendments to
>   land readily (less ratification ceremony than a post-team ADR amendment)
> - The team should NOT defer to these as authoritative when their Phase 1-8 finds real
>   issues; the discipline of recognition-not-design (ADR-006) means the team's
>   findings supersede pre-team intuition

---

## Index

- [ADR-001 — Failure-class memory is structural, not documentary](#adr-001--failure-class-memory-is-structural-not-documentary)
  - [ADR-001 Amendment 1 — Carrier-strength hierarchy + passive/active surfaces + structural commitments C1–C8](#adr-001-amendment-1--carrier-strength-hierarchy--passiveactive-surfaces--structural-commitments-c1c8)
- [ADR-002 — Compose, don't compete](#adr-002--compose-dont-compete) *(amended by ADR-013, ADR-015)*
- [ADR-003 — Biological metaphor is load-bearing, not decorative](#adr-003--biological-metaphor-is-load-bearing-not-decorative)
- [ADR-004 — Implicit-to-explicit elevation as architectural posture](#adr-004--implicit-to-explicit-elevation-as-architectural-posture)
- [ADR-005 — Sub-clause F at every trust boundary](#adr-005--sub-clause-f-at-every-trust-boundary)
  - [ADR-005 Amendment 2 — Rationale-as-required-field as transverse sub-clause F discipline](#adr-005-amendment-2--rationale-as-required-field-as-transverse-sub-clause-f-discipline)
  - [ADR-005 Amendment 3 — Audit reports its own tier honestly](#adr-005-amendment-3--audit-reports-its-own-tier-honestly)
- [ADR-006 — Recognition, not design](#adr-006--recognition-not-design)
- [ADR-007 — Anti-YAGNI: structurally-guaranteed need](#adr-007--anti-yagni-structurally-guaranteed-need)
- [ADR-008 — Named-observer position as terminal stratum](#adr-008--named-observer-position-as-terminal-stratum)
  - [ADR-008 Amendment 1 — Multi-contributor workflow + scan severity defaults](#adr-008-amendment-1--multi-contributor-workflow--scan-severity-defaults)
- [ADR-009 — Adoption gradient: antigen meets consumers at any discipline level](#adr-009--adoption-gradient-antigen-meets-consumers-at-any-discipline-level)
- [ADR-010 — Fingerprint grammar v1: syn-based AST visitor pattern](#adr-010--fingerprint-grammar-v1-syn-based-ast-visitor-pattern) *(amended by ADR-012; partially superseded by ADR-015 §Mechanics §1)*
  - [ADR-010 Amendment 1 — Disambiguate the parsing path (Path C)](#adr-010-amendment-1--disambiguate-the-parsing-path-path-c)
  - [ADR-010 Amendment 2 — Fingerprint semver + MSRV policy](#adr-010-amendment-2--fingerprint-semver--msrv-policy)
  - [ADR-010 Amendment 3 — Scan semantics + first-stdlib operators + matcher-engine location + filter framing + invariants](#adr-010-amendment-3--scan-semantics--first-stdlib-operators--matcher-engine-location--filter-framing--invariants)
  - [ADR-010 Amendment 4 — Filter/proof framing as architectural principle](#adr-010-amendment-4--filterproof-framing-as-architectural-principle)
  - [ADR-010 Amendment 5 — `has_method` signature canonicalization via proc_macro2 (strict)](#adr-010-amendment-5--has_method-signature-canonicalization-via-proc_macro2-strict)
- [ADR-011 — `#[antigen_tolerance(...)]`: opt-out for legitimate fingerprint matches](#adr-011-antigen_tolerance-opt-out-for-legitimate-fingerprint-matches)
- [ADR-012 — ADR-010 Amendment 1: function-body patterns + match-context awareness](#adr-012-adr-010-amendment-1-function-body-patterns--match-context-awareness)
- [ADR-013 — ADR-002 Amendment 1: phantom-type witness recognition + witness-validity tier mapping](#adr-013-adr-002-amendment-1-phantom-type-witness-recognition--witness-validity-tier-mapping)
- [ADR-014 — `#[antigen_generates(...)]`: declaring antigens that proc-macros emit](#adr-014-antigen_generates-declaring-antigens-that-proc-macros-emit)
- [ADR-015 — Fingerprint engine: grammar-over-AST with per-fingerprint evaluator trait](#adr-015-fingerprint-engine-grammar-over-ast-with-per-fingerprint-evaluator-trait) *(partially supersedes ADR-010 §Mechanics §1)*
- [ADR-016 — Temporal recognition surface: provenance + freshness primitives](#adr-016-temporal-recognition-surface-provenance--freshness-primitives-for-stale-context-and-premature-abstraction)
- [ADR-017 — Antigen identity is canonical declaration site; cross-crate trust delegates to cargo](#adr-017-antigen-identity-is-canonical-declaration-site-cross-crate-trust-delegates-to-cargo)
- [ADR-018 — `#[descended_from]` propagation: tagged synthesis + diamond dedup + inheritance state matrix](#adr-018-descended_from-propagation-tagged-synthesis--diamond-dedup--inheritance-state-matrix)

---

## [ADR-001] Failure-class memory is structural, not documentary

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude (winrapids cwd, pre-team).

**Related**: ADR-002, ADR-004, ADR-008.

### Finding

When a bug is fixed in mainstream programming culture:
- The test for THAT bug ships.
- The lesson about the failure-CLASS the bug was an instance of lives in commit messages,
  developer memory, code comments, and at best a vague mentorship transmission.
- New code in structurally-similar territory does NOT inherit the lesson; the failure
  re-surfaces in a slightly different costume.

This is the implicit-memory failure mode. AI-coding agents amplify it because they lose
context between sessions, so the implicit memory has nowhere persistent to live.

Documentation is itself vulnerable to this — docstrings drift, README rot, blog posts
disappear. Documentation as the carrier of failure-class memory is a vulnerability,
not a solution.

### Decision

**Antigen makes failure-class memory structural and inheritable through the type system
and cargo tooling, not through documentation.**

The carriers of failure-class memory are:
- `#[antigen(name = "...", fingerprint = "...")]` declarations (B-cell memory)
- `#[presents(antigen)]` markers (MHC presentation)
- `#[immune(antigen, witness = ...)]` declarations (antibody specificity)
- `#[descended_from(...)]` propagation (lineage inheritance)

Each is a *checked* construct — the cargo tooling reads them, validates them, propagates
them, and enforces their integrity. Drift is detected at scan time, not at code-review
time.

### Mechanics

The witness requirement is load-bearing. A `#[immune(X, witness = Y)]` declaration
without a working `Y` is not a claim — `cargo antigen scan` flags it. This prevents
the documentation-rot pattern where claims in docstrings outlive their truth.

Cargo subcommands:
- `cargo antigen scan` — find unaddressed presentations
- `cargo antigen audit` — coverage and immunity-trend report
- `cargo antigen vaccinate` — apply known immunity to a structural family
- `cargo antigen new` — scaffold a new antigen

### Sweep-level consequences

- `antigen-core` ships the macros + witness primitives
- `cargo-antigen` ships the tooling that enforces structural integrity
- `antigen-stdlib` populates the 8 first-principles failure classes with concrete antigens
- Documentation in `docs/` is INFORMATIONAL; the source-of-truth lives in declarations

### Enforcement

- `cargo antigen scan` flags presentations without immunity
- `cargo antigen audit` enforces witness validity (witness function exists, runs, asserts)
- A repository `Cargo.toml` `[package.metadata.antigen]` `required = [...]` list causes
  CI failure on missing immunity

### Resolves

- The implicit-memory failure mode (per the originating insight from tambear adversarial)
- Documentation drift as a memory carrier (refined: ADR-001 doesn't reject
  documentation, it pushes memory upward in the carrier hierarchy — see Amendment 1)
- AI-coding-agent context-loss across sessions
- The "structural vs documentary" false binary (see Amendment 1 Change 1)
- The implicit conflation of "active marker" with "passive fingerprint" surfaces (see
  Amendment 1 Change 2)
- The unenumerated structural commitments C1-C8 (see Amendment 1 Change 3)

---

## ADR-001 Amendment 1 — Carrier-strength hierarchy + passive/active surfaces + structural commitments C1–C8

**Status**: Ratified 2026-05-08.

**Amends**: ADR-001.

**Reason**: Synthesizes aristotle's Phase 1-8 deconstruction findings A17, F1, F6,
scientist's validation pass F-RELATED-1, and adversarial's ATK-001-{1..5} into
structural refinements ADR-001 missed. The amendment is **structural-forcing**: the
project is committed to all C1–C8 commitments by other ADRs (007, 005, 010); ADR-001
should enumerate them rather than leaving them implicit. Aristotle's reciprocal
Phase 1-8 confirmed approval with no foundational objections; scientist validated.

**Related**: ADR-002, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009, ADR-010,
ADR-011, ADR-013.

### Change 1: Reframe Finding from "structural vs documentary" to "carrier-strength hierarchy"

Memory carriers exist on a **hierarchy of drift-resistance**:

```
  compile-time-checked   (type system, phantom-types, kani/prusti proofs)
          ↑
  scan-time-checked      (#[antigen], #[immune], #[presents], #[descended_from],
                         validated by `cargo antigen scan`/`audit`)
          ↑
  test-suite-checked     (proptest properties, regression tests, witness
                         functions invoked by `cargo test`)
          ↑
  review-discipline      (PR review checklists, mentorship, ADR cross-references)
          ↑
  documentation          (rustdoc, README, design docs, CHANGELOG)
          ↑
  commit-message         (commit log, issue tracker, post-mortems)
          ↑
  human/agent memory     (mentorship, in-flight conversation, in-context
                         working memory)
```

Each tier is more drift-resistant than the one below, but each costs more in
authoring effort and ergonomic friction. **Antigen's role is to push failure-class
memory upward in this hierarchy** — from human/agent memory to scan-time-checked
declarations whenever the failure-class admits structural recognition.

Some failure-class memory genuinely resists formalization (regulatory findings with
social context; post-mortems whose narrative is the lesson; compliance artifacts
whose authority comes from human signature). For these, antigen's `references`
field bridges to lower-tier carriers (ADR/DEC IDs, URLs, CVE numbers) without
claiming structural equivalence. The structural-vs-documentary distinction is
**not binary**; it's about *which carrier tier* a given failure-class belongs in.

### Change 2: Acknowledge passive (fingerprint scan) and active (explicit marker) surfaces

Antigen has two surfaces:

- **Active surface** — the developer explicitly marks code with attribute macros
  (`#[presents]`, `#[immune]`, `#[descended_from]`, `#[antigen_tolerance]`,
  `#[antigen_generates]`). Active markers are unambiguous, document intent, and
  survive refactoring as long as the marked items survive.
- **Passive surface** — the `fingerprint` field on `#[antigen]` declarations.
  `cargo antigen scan` walks the codebase and *recognizes* unmarked code that
  structurally matches a declared fingerprint. Passive scan finds vulnerable code
  that the original author did not mark — including code authored before the
  antigen was declared.

Both surfaces are load-bearing. Active markers carry intent; passive fingerprints
catch new sites that match known patterns, including in code authored by people
who don't know the antigen exists. v0.1 ships both surfaces.

**The 5-state interaction matrix** (audit reports each state separately):

1. **Marked + matched** — `#[presents(X)]` on a site that also matches X's
   fingerprint (intentional + recognized; audit reports doubly-marked).
2. **Passively detected** — no markers; matches X's fingerprint (scan reports
   needs `#[presents]` + `#[immune]` or `#[antigen_tolerance]`).
3. **Inconsistent** — `#[presents(X)]` on a site that does NOT match X's
   fingerprint (audit warns: marker wrong or fingerprint wrong).
4. **Tolerated** — `#[antigen_tolerance(X)]` on a site that matches X's
   fingerprint (legitimate match explicitly acknowledged).
5. **Stale tolerance** — `#[antigen_tolerance(X)]` on a site that doesn't match
   any fingerprint (the tolerance is dead weight; audit warns it can be removed
   — the descended_from-style stale-reference pattern applied to tolerances; per
   aristotle's reciprocal Phase 1-8 enhancement).

### Change 3: Enumerate structural commitments C1–C8

Ratifying ADR-001 commits the project to:

- **C1 — All four core carriers ship in v1.** `#[antigen]`, `#[presents]`,
  `#[immune]`, `#[descended_from]`. None deferrable. Per ADR-007 (anti-YAGNI).
- **C2 — Tooling-validated witness is non-negotiable.** A `witness = ...` field
  that doesn't resolve to a callable, runnable, asserting artifact is a bug, not
  a tolerated state. `cargo antigen audit` MUST detect dangling witnesses. Per
  ADR-005 sub-clause F.
- **C3 — Falsifiability is invariant.** No "trust me" mode. Every immunity claim
  is checkable. The audit's `Missing` and `NotFound` witness statuses are
  sub-clause F enforcement at the v0.0.x level.
- **C4 — Declarations live in source files, not in side-cars.** The carriers are
  macro attributes inside `.rs` files; not a separate `.antigen.toml` per
  failure-class. (Configuration in `[package.metadata.antigen]` is acceptable
  because it's project-level, not failure-class-level.)
- **C5 — Drift-detection happens at scan + audit time.** Not at compile time, not
  at runtime, not at code-review time. Scan time is the trust boundary at which
  witness validity is re-checked. Compile-time witnesses (phantom-type proofs)
  ARE compile-time safety; scan validates that the compile-time witness is
  reachable, not that it's "still" valid (the compiler answers that).
- **C6 — The carrier set is itself structural.** Adding a new primitive carrier
  (`#[antigen_tolerance]`, `#[antigen_generates]`, `#[exposes]`) requires an ADR
  amendment or new ADR, not just a new macro feature. Per ADR-006
  (recognition-not-design): each new carrier must recognize structure that the
  substrate already exhibits.
- **C7 — Cross-crate consumption is in-scope for v1+.** ADR-001 commits to
  antigens declared in one crate applying to consumers. Cross-crate trust-boundary
  mechanics defer to ADR-005's enforcement clauses and ADR-010 OQ1; the
  *commitment* is foundational here. ADR-009 governs how cross-crate references
  render at the named-observer stratum.
- **C8 — `[package.metadata.antigen]` is part of the structural memory.**
  Required-list, ADR registry pointer, audit strictness — all project-level
  structural. CI gates can read this metadata and enforce.

### Change 4: Witness-validity tier acknowledgment

The `witness` parameter accepts artifacts at multiple validity tiers:

- **Reachability tier**: the witness identifier resolves to a function/test that
  exists. (Floor; v0.0.x audit lives here.)
- **Execution tier**: the witness runs without panic and asserts a non-trivial
  property. (Sweep A2-A3 lift.)
- **Behavioral-alignment tier**: the witness exercises behavior that matches the
  antigen's structural fingerprint. (Sweep A4-A5 work; ADR-005 open question.)
- **Formal-proof tier**: the witness is a verified compile-time proof
  (phantom-type construction, kani/prusti/verus/creusot proof annotation). (Sweep
  A4+ via ADR-002 witness delegation.)

v0.1 ships the reachability tier; subsequent sweeps lift the bar. ADR-005
sub-clause F applies to whichever tier is current. JSON output includes a
`witness_tier` field for CI gates per ADR-013.

### Change 5: Add ergonomic-maintenance pressure as drift prevention

The prevention against declaration-drift is **ergonomic-maintenance pressure** —
making declarations cheaper to maintain than the docs they replace. Per ADR-008
(named-observer terminal stratum): scaffolding via `cargo antigen new`, IDE
annotations via the future rust-analyzer plugin, warn-don't-fail soft drift
detection, and amendment-suggestion when witnesses become stale. If maintaining
declarations costs more than maintaining docs, adoption fails — and ADR-001's
value-claim with it.

### Change 6: Related field expansion

Per scientist's F-RELATED-1 finding, ADR-001's original Related field was sparse.
Amended Related field listed in this Amendment's header above; full
cross-references include ADR-006 (recognition-not-design discipline grounding C6).

### Change 7: Resolves clause expansion

Three new entries added (the false-binary surfacing, the active/passive conflation,
and the C1-C8 unenumerated commitments) — all already integrated above in the
amended Resolves clause of ADR-001 itself.

### Resolves (this amendment)

- Aristotle Phase 1-8 finding A17 (memory carriers form a hierarchy, not a binary)
- Aristotle Phase 8 F1 (passive vs active spectrum was implicit)
- Aristotle Phase 5 commitments-enumeration (C1-C8 surfaced but not in ADR-001)
- Aristotle Phase 8 F6 (ergonomic-maintenance pressure was implicit)
- Adversarial ATK-001-2 (witness shape mismatch — the spectrum was unnamed)
- Adversarial ATK-001-5 (passive-vs-active surface confusion)
- Scientist F-RELATED-1 (ADR-001's Related field was sparse)

---

## [ADR-002] Compose, don't compete

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude.

**Related**: ADR-001 (memory mechanism), ADR-004 (elevation posture),
ADR-005 (sub-clause F validates the witness types this ADR defines),
ADR-012 (function-body matching extends compose-with-clippy posture),
ADR-013 (amends this ADR with phantom-type witness recognition).

### Finding

The Rust ecosystem already has many tools that handle pieces of the immune-system
shape:
- clippy (lints, structural pattern recognition)
- proptest, quickcheck (property-based testing)
- cargo-mutants (mutation testing)
- kani, prusti, creusot, verus (formal verification)
- miri (UB detection)
- the deprecation system (memory of one specific kind)
- RustSec / cargo-audit / cargo-deny (supply-chain awareness)

Each tool addresses a slice of the failure-class-memory problem. None composes them into
a coherent immune system. Antigen could either (a) reinvent these tools with
antigen-native versions, or (b) compose them under a shared vocabulary with shared
primitives.

Reinventing is wasteful and strategically wrong: it would fragment the ecosystem,
duplicate engineering, and miss the ecosystem-of-mature-tools advantage Rust already
has.

### Decision

**Antigen composes existing Rust ecosystem tools rather than competing with them. Witness
types DELEGATE to existing tools wherever possible.**

Witness mechanisms include:
- Tests (`#[test]`) — the immunity is verified by `cargo test`
- Property tests (`proptest!`, `quickcheck`) — same
- Formal verification harnesses (`kani::proof`, `prusti::trusted`, `verus::proof`,
  `creusot::ensures`) — antigen knows about them and treats them as valid witnesses
- Custom lints (clippy, dylint) — antigen treats lint enforcement as a witness
- Phantom-type proofs — for cases where a compile-time witness is feasible
- Antigen-native witnesses — only when no existing tool fits

When an existing tool covers a failure-class, antigen's antigen for that class delegates
to that tool. e.g., `#[immune(PanickingInDrop, witness = clippy::no_panic_in_drop)]`.

### Mechanics

The `witness` parameter on `#[immune(...)]` accepts:
- A test/proptest function name in the same module
- A path to a clippy lint identifier
- A path to a kani/prusti/verus/creusot proof annotation
- A path to a phantom-type construction proof
- An antigen-native witness type

`cargo antigen scan` validates each witness type by delegating to the underlying tool.

### Sweep-level consequences

- The team must thoroughly research existing Rust ecosystem tools (see
  `docs/expedition/ecosystem-composition.md`)
- Antigen's API must be pluggable so that future tools can become witness providers
- The first-version witness library prioritizes integration with widely-adopted tools
  (clippy, proptest) over deeper integration with niche tools

### Enforcement

- API design review: every new witness type must justify why it's not a thin delegation
  to an existing tool
- Documentation: every antigen in `antigen-stdlib` must specify which existing tool(s) it
  delegates to (if any) and which it competes with (if any — should be empty in v1)

### Resolves

- Ecosystem fragmentation risk
- "Yet-another-lint" criticism
- Reinventing-the-wheel engineering cost

---

## [ADR-003] Biological metaphor is load-bearing, not decorative

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude.

**Related**: ADR-006 (recognition), ADR-011 (autoimmunity prediction realized via tolerance carrier).

### Finding

The biological metaphor for antigen is rich:
- Antigen, antibody, vaccination
- B-cell memory, T-cell receptors, MHC presentation
- Lineage, clonal expansion
- Innate vs adaptive immunity
- Tolerance vs autoimmunity
- Cytokine signaling, inflammation
- Pathogen Recognition Receptors

When the metaphor predicts a primitive — e.g., "B-cell memory persists across
infections" — the Rust analog should also persist (across compile units, across
sessions, across crates). When the metaphor predicts inheritance — e.g., "antibodies
inherit through B-cell lineage" — the Rust analog should propagate (via
`#[descended_from]`).

The metaphor is a **thinking tool that has produced real architectural insights**. It
suggested the inheritance primitive (which doesn't exist in any current Rust tool). It
suggested the autoimmunity tolerance check (which protects against false-positive
flagging). It suggested vaccination as a development action (which becomes
`cargo antigen vaccinate`).

If we abandon the metaphor as decorative, we lose the predictive power. If we treat
metaphor-suggested primitives as suspect, we cripple the design.

### Decision

**The biological metaphor is preserved as load-bearing throughout the design. When the
metaphor breaks (predicts something that doesn't fit Rust naturally), name where and
refine — do not abandon. When the metaphor predicts something useful, build it.**

Specifically:
- The naturalist role on the antigen team has explicit responsibility for keeping the
  metaphor honest
- Every API decision considers the biological analog as a thinking tool
- Where biology rhymes (e.g., MHC presentation → `#[presents]`), the names align
- Where biology and Rust ecosystem standards differ (e.g., antibody vs witness), Rust
  ecosystem precision wins for API; biology preserves for documentation/pedagogy

### Mechanics

The naturalist role on the antigen JBD team owns this discipline. When they observe
metaphor-predicted primitives that haven't been built, they surface them. When the
metaphor breaks, they name where.

The `docs/glossary.md` anchors every term to its biological referent + Rust ecosystem
analog.

### Sweep-level consequences

- The naturalist role is non-optional in the antigen team
- The glossary is maintained as load-bearing artifact
- Design reviews include "does this break the metaphor?" as a checklist item
- Documentation can use biological language freely; API documentation uses precise
  Rust terms with biological analogies cross-referenced

### Enforcement

- Glossary updates required for every new term in design docs
- Naturalist review required for any API change that breaks an established metaphor
  mapping
- Every new antigen in `antigen-stdlib` includes its biological analog in documentation

### Resolves

- Metaphor-as-decoration anti-pattern (where biology is mentioned in docs but doesn't
  inform design)
- Vocabulary drift (where the biological referent is forgotten and only the Rust term
  survives, leading to imprecise reasoning)

---

## [ADR-004] Implicit-to-explicit elevation as architectural posture

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude. Inherited from tambear's DEC-029-impl + V4 work.

**Related**: ADR-001 (memory mechanism), ADR-008 (named-observer).

### Finding

Mainstream programming languages are dominated by **implicit structure that is
load-bearing**. Closures capture lexical environments implicitly. Type variance is
implicit in subtyping rules. Effect tracking is implicit in monad libraries.
Memoization invariants are implicit in cache implementations. Refactoring discipline is
implicit in mentorship.

When this implicit structure is wrong (the meet=min vs meet=max frame-translation in
tambear; the missing variance annotation in TypeScript that produces a runtime cast
error), the failure mode is invisible because the structure itself is invisible.

Tambear's expedition-level work showed that **making structural what is implicit is the
deepest fold operation a project can perform**. Each elevation (sequential→parallel,
value→reference, concrete→symbolic, single-axis→product-axis, implicit→explicit) makes
new work possible while elevating the boundary that was preventing it.

Antigen is one specific application of this fold: making **failure-class memory** —
which has been implicit in human/agent memory — structural and explicit in the type
system.

### Decision

**Antigen treats implicit-to-explicit elevation as its core architectural posture. Every
design decision is evaluated against: does this make implicit structure explicit, or
does it preserve implicit-mode obscurity?**

When the design forces work to flow through explicit declarations (`#[antigen]`,
`#[presents]`, `#[immune]`, `#[descended_from]`), it is doing the elevation correctly.
When the design accepts implicit conventions ("everyone knows this is fragile"), it is
falling back to implicit-mode.

The cost of explicit-mode is forced pacing, more typing, and slower velocity per-line.
The benefit is legibility — to future agents (Claude or human), to fresh-context teams,
to cross-project consumers, to the broader Rust ecosystem.

### Mechanics

The discipline is pre-loaded into the team via `team-briefing.md`. Every fresh agent
imports the explicit posture before doing work. The campsite logbook, the glossary, the
ADR registry are explicit-mode infrastructure.

The cost is real: an antigen team works slower per-token than an implicit-mode team. The
exchange is calibration: explicit-mode produces results that are CORRECT and LEGIBLE,
while implicit-mode produces results that are FAST and FRAGILE.

### Sweep-level consequences

- The antigen team's velocity is paced by explicit-mode discipline
- Premature optimization toward implicit-mode (skipping witness declarations, eliding
  `#[descended_from]`) is rejected
- Documentation reflects the elevation: the design docs walk through the implicit-mode
  baseline before describing the explicit-mode replacement
- Every ADR explicitly names the implicit pattern it replaces

### Enforcement

- ADR template includes "implicit pattern being elevated" as a required section
- Code review asks: "is this declaration replacing an implicit convention?"
- Onboarding for new antigen team members starts with this ADR

### Resolves

- The "implicit-skilled-fast vs explicit-discipline-slower-but-required" tension named
  by Tekgy + Claude
- Fresh-session amnesia where new agents revert to implicit-default (pre-loaded explicit
  imports prevent this)
- Cross-team communication failures (explicit declarations are inspectable by all)

---

## [ADR-005] Sub-clause F at every trust boundary

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude. Inherited from tambear DEC-022 sub-clause F.

**Related**: ADR-001 (witness mechanism), ADR-002 (composition).

### Finding

Tambear DEC-022 sub-clause F establishes: **every trust boundary requires a validation
check before trust is extended**. The pattern: an asserted claim must be canonicalized
and validated by the receiving system before it is acted upon.

Antigen has multiple trust boundaries:
- The boundary where `#[immune(X, witness = Y)]` claims immunity — must validate that
  Y exists and asserts what it claims
- The boundary where `#[descended_from(parent)]` propagates markers — must validate that
  the parent's markers still apply
- The boundary where `cargo antigen vaccinate` applies a pattern across a family — must
  validate that the pattern matches each target site
- The boundary where antigen-stdlib is consumed by downstream crates — must validate
  that imported antigens haven't been redefined incompatibly

If any of these boundaries skips validation, the immune system is poisoned. A claim
of immunity without a working witness becomes the new "trust me" comment. A propagated
inheritance without re-justification becomes a stale reference.

### Decision

**Every antigen trust boundary requires a sub-clause F validation check. The check is
implemented in tooling (cargo-antigen) and verified by CI integration.**

Specific boundaries and their checks:

1. **Immunity claim**: `cargo antigen scan` validates that `witness = Y` resolves to a
   real test/proptest/proof/lint and that it exercises behavior matching the antigen's
   structural fingerprint.

2. **Inheritance propagation**: `cargo antigen scan` walks `#[descended_from]` chains and
   re-checks that inherited witnesses still apply to descendants. Signature divergence
   or behavioral change invalidates the inheritance.

3. **Vaccination application**: `cargo antigen vaccinate` requires confirmation before
   applying patterns; the pattern's match against each target site is logged for audit.

4. **Cross-crate antigen consumption**: when crate A imports antigens from crate B, the
   imported declarations are checked for fingerprint compatibility (not just name
   collision). Incompatible redefinitions fail the build.

### Mechanics

`cargo antigen scan` and `cargo antigen audit` are the trust-boundary enforcers. Their
output is structured (JSON / SARIF) so that CI can fail builds on trust-boundary
violations.

### Sweep-level consequences

- Every cargo-antigen subcommand performs explicit validation; no "trust me" mode
- Documentation for every antigen in `antigen-stdlib` includes the witness validation
  steps
- IDE integration surfaces trust-boundary failures inline

### Enforcement

- CI gate: `cargo antigen audit --strict` fails build on any trust-boundary violation
- API: tooling functions return structured errors (not panics) for trust-boundary
  violations so consumers can handle them
- Documentation: every ADR amendment must describe its trust-boundary impact

### Resolves

- The "trust me" anti-pattern in immunity claims
- Stale inheritance after parent function changes
- Cross-crate antigen confusion (where two crates define `FrameTranslation` differently)

---

## ADR-005 Amendment 2 — Rationale-as-required-field as transverse sub-clause F discipline

**Status**: Ratified 2026-05-09.

**Amends**: ADR-005 (Sub-clause F at every trust boundary).

**Participants**: pathmaker (drafted; integration commit), navigator (full-body
spec), aristotle (Phase 1-8 in Track 1 bundle), team-lead (ratification).

**Related**: ADR-001 Amendment 1 Change 7 (originating observation —
rationale-as-required-field elevated to a structural commitment of the
carrier hierarchy); ADR-009 (`summary` field on Layer 1; `references` field
on Layer 2); ADR-011 (`rationale` on `#[antigen_tolerance]`); ADR-014
(`rationale` on `#[antigen_generates]`); ADR-005 Amendment 3 (sibling
amendment; complementary surface — Amendment 2 governs trust-extension
primitives at parse time, Amendment 3 governs audit's reporting surface
at runtime; per aristotle R-Conflict-Check, no content conflict).

### Finding

ADR-005 ratifies sub-clause F as a per-boundary discipline: every trust
boundary requires a validation check before trust is extended. The original
text enumerates three boundaries (immunity claim, lineage propagation,
cross-crate antigen consumption) and discusses each individually. What the
original text does *not* surface — and what the substrate exhibits across
nearly every primitive added since ratification — is that sub-clause F has
a *transverse* application: **every primitive that extends trust requires
an explicit justification field, not just a validation check on the trust
itself.**

The transverse principle was first named in ADR-001 Amendment 1 Change 7
("rationale-as-required-field is structurally a property of the carrier
set, not just a Layer-2 ergonomic"). What that change observed: the
*carriers* of antigen memory consistently grew justification fields as
they accumulated trust-extending power. This amendment promotes the
observation from a property-of-the-carriers (per ADR-001) to an
operational-discipline-of-trust-boundaries (per ADR-005). Different lens;
same fact.

The substrate makes the principle visible: every primitive that extends
trust currently carries a justification field, and removing any of them
would be a sub-clause F violation:

| Primitive | Trust-extension | Justification field |
|---|---|---|
| `#[antigen(name = "...", summary = "...")]` | "this name labels a real failure-class" | `summary` (Layer 1; ADR-009) — human-readable description of what the antigen is |
| `#[antigen(..., references = [...])]` | "this declaration is grounded in real-world evidence" | `references` (Layer 2; ADR-009) — open-vocabulary list of CVE/RFC/ADR/URL pointers |
| `#[immune(X, witness = Y)]` | "Y proves immunity to X" | `witness` (ADR-001/002/005) — executable rationale |
| `#[immune(X, witness = Y, rationale = "...")]` | "the witness is appropriate for THIS site" | `rationale` (ADR-001 Amendment 1 Change 7) — narrative justification supplementing the executable witness |
| `#[antigen_tolerance(X, rationale = "...")]` | "this site matches X's fingerprint by design, not by vulnerability" | `rationale` (ADR-011) — required field; tolerance without rationale is rejected at parse time |
| `#[antigen_generates(X, rationale = "...")]` | "this proc-macro emits sites presenting X" | `rationale` (ADR-014) — required field; the macro author justifies the generation pattern |

Five+ manifestations across five+ ADRs. The discipline propagates from
existing ADRs to new ADRs (ADR-011, ADR-014) without explicit
coordination — that's how a load-bearing principle should behave.

### Decision

**Sub-clause F applies recursively at the API level: when an ADR
introduces a trust-extending primitive, the justification-field
requirement applies by default unless explicitly waived with documented
reasoning.**

The principle, stated normatively:

> When a new ADR ratifies a primitive (attribute macro, configuration
> field, declaration form) that extends trust — i.e., causes downstream
> tooling, auditors, or consumers to act differently because the
> primitive is present — the primitive MUST carry a justification field
> (named `rationale`, `summary`, `references`, `witness`, or an
> ADR-specific equivalent) by default.
>
> Waivers require explicit ADR-level reasoning. A waiver is a
> structural commitment ("this primitive does not extend trust;
> justification would be decorative"); the absence of a justification
> field is itself a claim that consumers will read and rely on.

### Mechanics

The amendment operationalizes at three surfaces:

1. **Parse-time enforcement**: justification fields are typically
   `Option<String>` at the parser level (so they appear in syntax)
   but ADR-NNN may declare them required (`#[antigen_tolerance]`'s
   `rationale` is required at parse time per ADR-011). When required,
   omission produces a compile error from the proc-macro pointing at
   the missing field.

2. **Empty-rationale rejection**: where the field is required, the
   parser MUST reject empty strings (rationale-stuffing prevention).
   ADR-011's parser already does this; the principle generalizes.

3. **Future-ADR review checklist**: the ADR template (per
   `process.md`) gains an explicit prompt — "Does this ADR introduce
   a primitive that extends trust? If yes: name the justification
   field, OR document why one is not required." Ratification cannot
   proceed without an answer.

The cross-cutting consequence: when an ADR proposes a new primitive
that downstream consumers will rely on, reviewers ask "where is the
justification field?" rather than "why isn't there one?" The default
flips. New trust-extending primitives without justification fields
require active argument; primitives with justification fields are the
unmarked default.

### Sweep-level consequences

- **Sweep A2 (current)**: ADR-011 (`#[antigen_tolerance]`) and ADR-014
  (`#[antigen_generates]`) already operationalize the principle; this
  amendment ratifies what they already do. ADR-016 (temporal
  recognition surface) ships its `verified_at`/`evidence`/`stale_after`
  trio with the same discipline — `evidence` is the justification
  field for temporal trust extensions.
- **Future sweeps**: every ADR that proposes a new attribute, config
  field, or declaration form is reviewed against this discipline at
  Phase 1-8 time. Aristotle's deconstruction asks the question
  explicitly.

### Enforcement

- **Parse-time**: required justification fields enforced by the
  relevant proc-macro parsers (e.g., `antigen-macros::parse::ImmuneArgs::validate`,
  `antigen_tolerance` parser when ADR-011 ships).
- **Process**: `process.md`'s ADR template + Phase 1-8 review prompt
  ensures future amendments are reviewed against the discipline.
- **Adversarial**: ATK seeds against new primitives include
  "justification-field-missing" as a default attack pattern.

### Resolves

- The implicit-but-unnamed principle that the carrier set has
  consistently grown justification fields as trust-extending power
  accumulated.
- The asymmetry where ADR-005's original text named per-boundary
  validation (the trust check itself) but did not name the
  per-primitive justification (the rationale for extending trust in
  the first place).
- The future-ADR review gap: without this amendment, future trust-
  extending primitives could ship without justification fields and
  be discovered only at adversarial review. The amendment makes the
  question structural at ADR-template time.

---

## ADR-005 Amendment 3 — Audit reports its own tier honestly

**Status**: Ratified 2026-05-09.

**Amends**: ADR-005 (Sub-clause F at every trust boundary), specifically
Decision item 1 (Immunity-claim trust boundary).

**Participants**: naturalist (drafted; v1 absorbs aristotle's four
refinements: R-Tier-Granularity, R-Crash, R-Soft-MUST, R-Conflict-Check),
aristotle (Phase 1-8 verdict: ratify substantively with refinements),
adversarial (motivation ATKs filed A2-003/004/005/011/012; ATK-Am3 queue
surfaced via aristotle Phase 1-8), scientist (validation), team-lead
(ratification).

**Related**: ADR-001 Amendment 1 Change 4 (witness-validity tier model —
this amendment enforces tier-honesty at the output surface where the
model is read); ADR-005 §sub-clause F (the discipline being extended);
ADR-005 Amendment 2 (sibling amendment — complementary surfaces:
Amendment 2 governs trust-extension primitives, Amendment 3 governs
audit's reporting surface; per R-Conflict-Check, no content conflict);
ADR-006 (recognition-not-design — five confirmed instances cleared the
threshold; broader generalization to all recognition mechanisms held
below threshold per team-lead's scope decision); ADR-013 (phantom-type
witness recognition — also subject to the tier-honesty discipline).

### Finding

ADR-005 establishes sub-clause F at every trust boundary: an asserted
claim must be canonicalized and validated by the receiving system
before it is acted upon. Item 1 of the original Decision identifies the
immunity-claim trust boundary: `cargo antigen scan` validates that
`witness = Y` resolves to a real test/proptest/proof/lint.

What ADR-005 does not yet address: **the audit's own status output is
itself a trust boundary**. When `ImmunityAudit::is_well_formed()`
returns `true`, or when the JSON output reports `WitnessStatus::Resolved`,
downstream consumers (developers reading reports, CI gates, IDE
integrations) extend trust on the basis of the status word. That trust
extension is governed by sub-clause F: the status word must reflect
what the audit *actually verified*, not what it *could maximally
infer*.

A2 adversarial filings produced five confirmed classes of
under-verified immunity claims, each demonstrating the same failure
mode: the audit's status surface reports verification work at a
stronger tier than the audit actually performs. The pattern is
biology-derivable from B-cell affinity-maturation tier hierarchy and
corresponds directly to ADR-001 Amendment 1 Change 4's witness-validity
tier model (reachability / execution / behavioral-alignment /
formal-proof). The substrate evidence:

| ATK | Surface | Reported tier | Actual tier | Failure mode |
|---|---|---|---|---|
| **A2-003** | `audit.rs` (`validate_witness` + `is_well_formed`) | Execution (Resolved Function = "well-formed") | Reachability (identifier exists; function body asserts nothing) | `fn my_witness() {}` empty body passes audit |
| **A2-004** | `audit.rs` (`detect_external_tool`) | External delegation (well-formed; "trusts external tool") | Pattern-match-on-prefix (`clippy::` matched; lint name unverified) | `clippy::nonexistent_lint` passes audit |
| **A2-005** | `audit.rs` (`FunctionIndexVisitor`) | Resolved with confident `WitnessKind` | Resolved-to-one-of-N functions, filesystem-walk-order dependent | Same-name collision flips classification non-deterministically |
| **A2-011** | `audit.rs` (function-name extraction via `rsplit("::")`) | Resolved | Last-segment-name-lookup; full path discarded | Fabricated path resolves cleanly |
| **A2-012** | `audit.rs` (`detect_kind`) | Execution (Resolved Test = "cargo test ran it") | Reachability + attribute-presence (audit doesn't run cargo test) | `#[test] #[ignore]` witness reported as well-formed |

All five are sub-clause F violations at the audit-reporting surface.
The audit extends trust ("this immunity claim is well-formed") on the
basis of work that does not support the trust extension.
Recognition-not-design (ADR-006) threshold satisfied: substrate (five
`TODO(team)` markers in `antigen/src/` already enumerate the same
gaps), engineering (five adversarial-confirmed ATKs), biology (B-cell
affinity-maturation cognate names the pattern). Three windows; same
finding.

The biology-derivation itself instantiates the depth-shift discipline:
the visible question was "should `is_well_formed` return true for
empty bodies?" The load-bearing commitment was "the audit's reporting
surface IS a trust boundary" — once that commitment is named, the
empty-body case becomes one of five instances of the underlying
violation, and the amendment's scope follows from the deeper commitment
rather than the visible question.

**Implicit pattern elevated**: the audit's status word ("Resolved",
"well-formed") implicitly encodes a verification-tier claim. This
amendment makes the encoding *explicit* — the audit's reporting
surface must name the tier its verification work actually supports,
not the tier its status word historically implied.

### Decision

**ADR-005's sub-clause F applies at the audit-reporting surface. The
audit's status output (`is_well_formed()`, `WitnessStatus`,
`witness_tier` field in JSON) must report the tier its verification
work actually supports — never a stronger tier.**

The principle, stated normatively:

> **Audit reports its own tier honestly.** When `cargo antigen audit`
> emits a status word (`Resolved`, `External`, `well-formed`,
> `witness_tier`), the word must reflect the verification work the
> audit actually performed at that point. The audit cannot report
> tier-N+1 work while doing tier-N verification. Where verification
> at the claimed tier has not occurred, the audit MUST either (a)
> report at the lower tier its work actually supports (per W7's
> strict four-tier `WitnessTier` enum: `None | Reachability |
> Execution | FormalProof`), OR (b) emit a tier-honesty audit hint
> per W7's audit-hint mechanism so downstream consumers can
> distinguish "audit verified at the reported tier" from "audit
> recognized at this tier; stronger verification deferred or absent."

Two failure modes the principle covers:

- **Wrong-tier (silent false answer)**: audit reports
  `Resolved`/`well-formed` while doing strictly less verification
  work. Five confirmed instances above.
- **No-answer (crash variant)**: audit's recognition mechanism crashes
  on a legitimate-but-pathological input. This is a stronger
  sub-clause F violation — the audit extends trust *unconditionally*
  by failing to produce any answer at all. A crashing recognition
  mechanism cannot report any tier honestly.

### Mechanics

The amendment operationalizes at three surfaces:

1. **`is_well_formed()` semantics**: must not return `true` when the
   underlying verification work is at strictly lower tier than the
   audit's report language suggests. Per ADR-001 Amendment 1 Change 4's
   tier model, `Resolved (Function)` (no `#[test]` attribute, not
   external, not proptest) is at Reachability tier — `is_well_formed()`
   should return `true` only if the consumer is willing to accept
   Reachability-tier evidence. v0.1 ships with documented soft-language:
   status output explicitly names the tier so consumers can choose
   their threshold.

2. **`witness_tier` field in JSON output** (R-Tier-Granularity:
   aligned with W7's strict four-tier `WitnessTier` enum +
   audit-hint mechanism): per ADR-001 Amendment 1 Change 4, audit
   emits the witness-validity tier as an explicit field. The
   amendment requires this field to reflect actual verification
   work, not maximally-inferred tier. The tier is the strict W7
   `WitnessTier` enum (Ord-able for CI gating); per-case
   disambiguation rides on a parallel `audit_hint` field rather
   than tier-name compound forms:

   | Verified work | `witness_tier` | `audit_hint` |
   |---|---|---|
   | Identifier resolves to a function (no further check) | `Reachability` | `function-resolves` |
   | Function has `#[test]` attribute (audit did not invoke cargo test) | `Reachability` | `test-attribute-present-not-invoked` |
   | Function has `#[test]` AND `#[ignore]` (test will not run) | `Reachability` | `test-attribute-present-ignore-skipped` |
   | External-tool prefix matched (e.g., `clippy::`) | `Reachability` | `external-tool-prefix-recognized` |
   | External-tool reference verified by tool invocation (deferred to A3+) | `Execution` | `external-tool-invoked` |
   | Phantom-type witness shape recognized | `FormalProof` | `phantom-type-shape-recognized` |
   | Phantom-type witness construction validated (deferred to future ADR) | `FormalProof` | `phantom-type-construction-validated` |
   | Witness resolves to function in dependency crate (consuming workspace cannot execute via local cargo test) | `Reachability` | `cross-crate-witness-not-locally-executable` |

   The strict four-tier enum preserves W7's Ord-able CI gating;
   the audit_hint carries the additional information that
   compound names would have encoded.

   **Cross-crate witness tier defaulting** (per ADR-005 Amendment 3
   sub-amendment, 2026-05-10): when a witness identifier resolves to a
   function in a dependency crate (path qualified by `canonical_path`,
   or path prefix matches a dependency's crate name), the consuming
   workspace cannot execute the witness via local `cargo test`. The
   audit MUST report `Reachability` tier (identifier resolves) +
   `audit_hint: "cross-crate-witness-not-locally-executable"`, NOT
   `Execution` tier. Reporting `Execution` overstates the audit's
   verification work and violates sub-clause F at the audit surface.

   **Future-amendment door**: when antigen-stdlib + cross-crate test
   orchestration matures (A4-A5 behavioral-tier work), a dependency's
   test status may become locally introspectable (e.g., via a manifest
   field declaring which tests are antigen-witnesses + a CI signal from
   the dependency's release pipeline). At that point, cross-crate
   witnesses with verified-passing dependency tests may earn `Execution`
   tier + a different audit_hint. The current default reflects what the
   consuming workspace's audit actually verifies in v0.1; the door is
   preserved without commitment.

3. **Crash-resistance at recognition surfaces** (per R-Crash:
   in-domain input defined explicitly): `cargo antigen audit` MUST
   not crash on legitimate-but-pathological *in-domain* input.

   *In-domain input* is defined as: source files that parse via
   `syn::parse_file`; declarations whose attribute-arg parsers
   accept; `[package.metadata.antigen]` config that deserializes
   via serde. Out-of-domain inputs (parse failures at any layer)
   are caught earlier by the parsing trust boundary and are not
   subject to crash-resistance under this amendment.

   *In-domain pathological inputs* governed by crash-resistance:
   cycles in `#[descended_from]` chains; deep-recursion phantom-type
   construction walks; high-cardinality fingerprint-set matching.
   Audit MUST handle gracefully — bound depth, detect cycle, or
   fail with structured error naming the limit hit (rather than
   panic, abort, or hang).

### Sweep-level consequences

- **Sweep A2 W7**: `WitnessKind::PhantomType` recognition ships with
  `WitnessTier::FormalProof` + `phantom-type-shape-recognized` audit
  hint (per ADR-013); recognition is warn-and-emit-tier rather than
  silent acceptance. `#[ignore]` detection added to `detect_kind`;
  `#[test] #[ignore]` reports `WitnessTier::Reachability` +
  `test-attribute-present-ignore-skipped`.
- **Sweep A2 W9**: human-readable audit output language reviewed for
  tier-honesty — replace "structurally well-formed" with explicit
  tier-naming where current status word over-claims.
- **Sweep A3**: cycle detection in `#[descended_from]` walking is
  *required* for Amendment 3 compliance, not optional.
- **Future ADR territory**: extension of the tier-honesty principle
  beyond audit surfaces (to all antigen recognition mechanisms)
  waits for substrate confirmation outside audit. Per team-lead's
  ADR-006 invocation: this amendment is audit-specific only; the
  broader generalization stays in methodology/closure-narrative
  substrate until three independent instances surface outside audit.

### Enforcement

- **Tests for tier-honesty (existing motivation ATKs)**:
  ATK-A2-003/004/005/011/012 fixtures remain in
  `antigen/tests/atk_a2_adversarial.rs` as Amendment 3 regression
  guards. They PASS when the implementation reports honest tiers;
  FAIL when the audit silently over-claims.
- **JSON schema includes `witness_tier` + `audit_hint`**: CI gates
  for downstream consumers can read the tier field directly. Both
  fields MUST be present on every audit output; absence indicates
  tier-unknown which is itself a sub-clause F violation.
- **Crash-resistance gate (per ATK-Am3 queue)**:
  - **ATK-Am3-Crash-1**: cycle detection in `#[descended_from]`
    chain walking. Audit MUST detect cycle; structured error names
    the cycle.
  - **ATK-Am3-Crash-2**: depth-bound on phantom-type construction
    walks. Audit MUST bound depth; structured error names the
    depth limit hit.
  - **ATK-Am3-Crash-3**: memory-bound on high-cardinality
    fingerprint-set matching. Audit MUST bound memory or paginate;
    structured error names the limit hit.
  - **ATK-Am3-Tier-1**: `witness_tier` field absent in JSON output
    is itself a sub-clause F violation. Test verifies field
    presence on every audit output.
  - **ATK-A3-011**: cross-crate witness tier defaulting. Fixture:
    workspace W depends on crate C; C declares
    `#[immune(SomeAntigen, witness = some_test_in_C)]`; W scans with
    `--include-deps`. Audit output for the dep's immunity MUST report
    `Reachability` + `cross-crate-witness-not-locally-executable`, NOT
    `Execution`. Pre-implementation contract; filed in
    `atk_a3_fractal_preview.rs` with `#[ignore]` until A4 ships.

### Resolves

- ATK-A2-003 (empty witness body classified as well-formed).
- ATK-A2-004 (fabricated external tool reference classified as
  well-formed).
- ATK-A2-005 (same-name function collision classified
  non-deterministically).
- ATK-A2-011 (fabricated path prefix discarded; resolves as clean
  witness).
- ATK-A2-012 (`#[test] #[ignore]` classified equivalently to running
  test).
- The biology-cognate prediction (B-cell affinity-maturation tier
  hierarchy applied to witness validity) is now operationalized at
  the audit reporting surface, not just acknowledged in ADR-001
  Amendment 1 Change 4's tier model.

### Open questions deferred

1. **Tier-honesty for non-audit recognition mechanisms.** The
   amendment is audit-specific per team-lead's ADR-006 scope
   decision. Whether the principle generalizes to scan-time
   attribute parsing, schema versioning at the producer-consumer
   boundary, or cross-crate `descended_from` propagation stays
   open until three independent instances surface outside audit.
2. **Behavioral-alignment tier verification.** ADR-001 Amendment 1
   Change 4 names this tier; the amendment requires the audit to
   *report honestly that behavioral-alignment is unverified* when
   fingerprint-aware reasoning hasn't occurred. Behavioral-alignment
   verification is its own future-ADR territory.
3. **External-tool actual invocation**. Currently the audit
   recognizes external-tool prefixes without invoking the tool;
   `Reachability` + `external-tool-prefix-recognized` acknowledges
   this gap. `Execution` + `external-tool-invoked` comes when
   invocation lands (Sweep A3+).

---

## [ADR-006] Recognition, not design

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude. Inherited from tambear DEC-032 placeholder
("recognition-not-design") and naturalist's DEC-character finding.

**Related**: ADR-003 (metaphor), ADR-004 (elevation),
ADR-007 (anti-YAGNI is the structurally-guaranteed counterweight to recognition).

### Finding

Tambear's expedition surfaced a distinction between two kinds of architectural work:
- **Design DECs** — choosing among alternatives; ratifying a decision that wasn't
  predetermined
- **Recognition DECs** — naming structure that was already implicit in the substrate;
  ratifying a fact, not a choice

Antigen is fundamentally a **recognition** project. It does not invent failure-classes;
it recognizes patterns that already exist in real-world Rust codebases. It does not
design immunity; it recognizes proof-shapes that existing tools already produce
(witnesses).

Treating antigen as recognition rather than design has implications:
- The 8-class first-principles taxonomy is recognition of existing structural shapes,
  not invention
- Antigen-stdlib is recognition of existing common bug patterns, not invention
- The witness mechanism is recognition of existing proof types, not invention
- The vaccination operation is recognition of existing refactoring patterns, not invention

This reframing matters because it sets the right epistemic posture: when a proposed
antigen feels speculative, the question is "is there a real structural pattern this
recognizes?" not "should we add this to the design?"

### Decision

**Antigen operates with recognition-not-design epistemic posture. New antigens, new
witness types, new composition rules are added when they recognize existing structure
in the substrate — not when they extend the design speculatively.**

Specifically:
- Adding an antigen to `antigen-stdlib` requires showing it recognizes a real pattern
  with multiple instances in the wild
- Adding a witness type requires showing it integrates with an existing tool/proof system
- Adding a composition rule requires showing it captures behavior the substrate already
  exhibits

The opposite — adding speculative entries because "we might need it" — is rejected
unless ADR-007 (anti-YAGNI) explicitly grants the structural-guarantee.

### Mechanics

The naturalist role guards this discipline at design-review time. When a proposed
addition feels designed-not-recognized, naturalist asks: "what structure are you
recognizing? show me the instances."

The `docs/expedition/failure-class-instances.md` document is the recognition substrate:
every antigen in stdlib must have its source pattern documented there.

### Sweep-level consequences

- Antigen development is bottom-up (recognize patterns from real code) more than
  top-down (design from first principles)
- The 8-class first-principles taxonomy is the EXCEPTION — it's a recognition of
  observed structural shapes lifted to a complete taxonomy via Phase 1-8 first-principles
  thinking. Future taxonomies should follow the same lift-from-observation pattern
- Speculative API features are deferred until structural-guarantee is shown

### Enforcement

- Code review: every new antigen / witness type / composition rule requires a
  "recognition" section in its declaration explaining what it recognizes
- `docs/expedition/failure-class-instances.md` requires updates for every antigen-stdlib
  addition
- Design discussions explicitly ask: "are we recognizing or designing?"

### Resolves

- Speculative-feature drift in API design
- Top-down design anti-patterns where features get added without empirical grounding
- Conflation of "the design says X" with "we ratified X based on evidence"

---

## [ADR-007] Anti-YAGNI: structurally-guaranteed need

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude. Inherited from tambear standing constraints.

**Related**: ADR-001 (the memory carriers ADR-007 commits to building all of),
ADR-002 (composition), ADR-006 (recognition),
ADR-013 (completes ADR-007's witness-type commitment via phantom-type recognition).

### Finding

Mainstream software engineering culture preaches YAGNI ("You Aren't Gonna Need It") —
don't build features speculatively. This is correct in many contexts, but it has a
load-bearing inversion: when the project's *structural commitments* guarantee that a
feature will be needed, building it later (when the structure forces the issue) is
expensive.

Tambear's anti-YAGNI / YAWNI doctrine: "If the principles structurally guarantee we'll
need it, build it now."

Antigen's structural commitments include:
- All 8 first-principles failure classes (ADR-006 recognition; not all 8 will have
  immediate stdlib instances, but all 8 are guaranteed-needed by the taxonomy)
- All four witness types (test, proptest, formal-verification, lint) — no version that
  ships only some
- The full `#[descended_from]` propagation logic (not just the easy cases)
- The `cargo antigen vaccinate` operation (not deferrable to "user runs find/replace")

These are guaranteed-needed because the structure of the design commits to them. Shipping
without them creates retrofit cost when the structure forces the issue.

### Decision

**Antigen builds for structural guarantee, not speculative possibility. Features that
the design's principles guarantee will be needed are built upfront; features that are
merely "might be useful" are deferred.**

The test for "structurally guaranteed":
1. Does some other ratified ADR commit to this feature being present? (yes → build now)
2. Does the failure-class taxonomy require this feature for completeness? (yes → build now)
3. Does the composition with other tools (ADR-002) demand this feature? (yes → build now)
4. Is the feature merely "might be cool" without a structural commitment? (no → defer)

### Mechanics

The aristotle role on the antigen team owns the structurally-guaranteed-need analysis.
When a proposed feature is debated, aristotle's first question is: "what structural
commitment guarantees we need this?"

The contrarian/inversion role asks the opposite: "what would happen if we DIDN'T build
this?" If the answer is "the design works fine," the feature is YAGNI; if the answer is
"we'd violate ADR-X," the feature is structurally-guaranteed.

### Sweep-level consequences

- The first sweep of antigen development implements ALL 8 failure classes' core
  primitives, not just the easy ones (e.g., FrameTranslation + BoundaryViolation)
- The first witness library covers ALL four witness types, not just `#[test]`
- `#[descended_from]` ships with full propagation logic, not stubbed
- `cargo antigen vaccinate` ships in v1, not v2

### Enforcement

- Sweep planning: every feature must be tagged "structurally-guaranteed" or "speculative"
- Speculative features require explicit ADR-7-amendment to be added; cannot just slip in
- "Implementation gap" reports show the structurally-guaranteed features that haven't
  shipped yet

### Resolves

- YAGNI-induced design fragmentation (where structurally-required features get deferred
  and the design becomes incoherent)
- The retrofit cost of adding structurally-guaranteed features after the fact
- Conflation of "we don't need this yet" with "this isn't structurally needed"

---

## [ADR-008] Named-observer position as terminal stratum

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude. Inherited from tambear's vertical-to-horizontal terminal
pattern (P8-A) and named-observer convergence-pattern work.

**Related**: ADR-001 (the carriers the named observer authors),
ADR-004 (elevation), ADR-006 (recognition),
ADR-009 (adoption gradient is named-observer ergonomics applied to the API).

### Finding

Tambear's expedition revealed that every refinement-lattice has a **terminal stratum
where individual practitioners enact the protocol**. The lattice abstracts; the terminal
stratum embodies. Practitioners are the inhabitants of the terminal stratum.

Antigen's lattice — failure-class memory at structural level — has a terminal stratum
too: **the developer (human or AI) who actually writes `#[antigen(...)]`, `#[immune(...)]`,
`#[descended_from(...)]` declarations and runs `cargo antigen scan`**.

The cargo tooling, the macros, the witness validators are infrastructure — they
*serve* the practitioner. The named-observer position is where the immune system
actually runs. Architecture below this stratum is invisible to the practitioner;
architecture above is invisible to the practitioner-as-implementer.

This has design implications:
- Ergonomics at the named-observer position is non-negotiable (60-second declaration
  threshold)
- IDE integration matters because the named observer is editing code, not running CLI
  tools all day
- Error messages must speak in the named-observer's vocabulary, not in tooling-internal
  language
- Antigen's "for whom is this designed" question always resolves to: the practitioner
  writing or reading code with antigen markers

### Decision

**Antigen treats the named-observer (developer) position as the terminal stratum of its
architecture. Design decisions are evaluated against: does this serve the named observer
who is editing/reading/maintaining code with antigen markers?**

Specifically:
- Macros are designed for ergonomic typing; aggressive scaffolding via `cargo antigen
  new`
- Cargo subcommand output is designed for human readability first, machine consumption
  second (with `--format=json` for tooling)
- IDE integration (rust-analyzer plugin) is a top-priority deliverable post-v1, because
  named-observer ergonomics live there
- Documentation is written for the named observer, not for the tooling implementer
- The `team-briefing.md` for the antigen team explicitly names the practitioner-stratum
  as the architecture's terminus

### Mechanics

User-experience review is explicit at every API decision. Questions to ask:
1. How long does it take a named observer to declare an antigen for a known
   failure-class? (target: under 60 seconds with `cargo antigen new`)
2. How visible is the antigen state at the named observer's editing position?
   (target: inline IDE annotations within v1.5)
3. How understandable is the cargo-antigen output to someone who hasn't read the API
   docs? (target: scan output is self-explanatory; audit output points to specific
   actionable next steps)

### Sweep-level consequences

- Sweep A6 (ergonomics polish + IDE integration) is a high-priority sweep, not a
  "nice-to-have"
- Cargo subcommand output design is a real engineering investment, not boilerplate
- The naturalist + scientist roles on the antigen team have explicit responsibility for
  named-observer experience
- "How does this feel to a named observer?" is a standard design-review question

### Enforcement

- Every public API surface includes a "named observer experience" section in its docs
- IDE integration milestones are tracked in the sweep plan
- User-experience telemetry (when antigen ships) feeds back into ergonomics priorities

### Resolves

- Tooling-first anti-pattern (where the tool exists for its own sake, not for users)
- Vocabulary fragmentation between API docs and tooling output
- Implicit assumption that "clean architecture" matters more than "ergonomic to use"

---

## ADR-008 Amendment 1 — Multi-contributor workflow + scan severity defaults

**Status**: Ratified 2026-05-09.

**Amends**: ADR-008 (Named-observer position as terminal stratum).

**Participants**: pathmaker (drafted), aristotle (Phase 1-8: ratify
substantively as drafted with Open-questions Layer-1/3 differential
add), adversarial (ATK-008-1 surfaced multi-contributor friction;
amendment ATKs queued), scientist (validation), team-lead (ratification).

**Related**: ADR-001 (the carriers the named observer authors); ADR-003
(biological metaphor — immune response lag predicts the warn-not-error
default); ADR-004 (elevation); ADR-005 (severity defaults are
trust-boundary defaults); ADR-006 (recognition — multi-contributor
lag is observed reality); ADR-009 (adoption gradient — warn-not-error
IS Layer 1 ergonomics); ADR-011 (`#[antigen_tolerance]` is the sibling
escape valve to warn-not-error severity).

### Reason

ADR-008 frames the named-observer as a single practitioner editing
code with antigen markers, but the design is silent on
multi-contributor workflows. Adversarial ATK-008-1 surfaced the
concrete failure: Team A declares `#[presents(X)]` in their branch;
Team B is writing `#[immune(X, witness = ...)]` in a parallel branch.
With `cargo antigen scan --strict` in CI, Team A's branch permanently
fails until Team B merges. The lag is normal development flow, not a
real vulnerability — but the current design treats it as a hard
failure. Naturalist's biology framing reinforces this: immune response
lag is normal; antigen presentation and antibody production are
temporally separated in real immune systems.

This amendment also closes the gap scientist named in F-DOCS-2 (the
"named observer experience" sections claimed by ADR-008's enforcement
clause that don't exist on current API surfaces — Sweep A2 deliverable).

### Change 1: Acknowledge multi-contributor workflow as the default case

ADR-008 calibrates around a single practitioner editing code with
antigen markers. The team-development reality is N practitioners
across parallel branches with different markers in flight at different
stages of review. Per ADR-006 (recognition-not-design):
multi-contributor lag is the observed-substrate reality of how Rust
projects actually develop.

**Amended addition** (new subsection in "Decision" section, after the
existing ergonomic-threshold paragraph):

> #### Multi-contributor workflow as the default
>
> The named-observer is rarely working alone. In team development, the
> practitioner's branch may contain a `#[presents(X)]` marker whose
> matching `#[immune(X, witness = ...)]` lives in a parallel branch
> still under review. The temporal gap between presentation and
> immunity is normal development flow, not a vulnerability.
>
> Antigen's tooling treats this lag as expected:
>
> 1. `cargo antigen scan` defaults to **warn-not-error** for
>    unaddressed presentations. The default reports unaddressed sites
>    and exits with success (exit code 0). The named observer sees the
>    warning, the CI pipeline doesn't break.
> 2. `--strict` mode is opt-in for both the CLI flag and the
>    `[package.metadata.antigen] strict = true` Cargo.toml setting.
>    Strict mode escalates unaddressed presentations to errors (exit
>    code 1) — appropriate for release branches and consumer projects
>    that have completed their adoption phase.
> 3. CI configurations may invoke `cargo antigen scan --strict` only
>    on main/release branches, leaving feature branches in warn-mode.
>    This pattern is documented in `cargo-antigen`'s help text and the
>    project's CONTRIBUTING-style guides.
>
> The biological analog (per ADR-003): immune response lag is normal.
> Antigen presentation and antibody production are temporally
> separated in real immune systems. The system accommodates the lag
> rather than treating any gap as a crisis.

### Change 2: Configurable severity at the project level

ADR-008 lists "User-experience telemetry" as an enforcement clause but
doesn't specify the project-level configuration surface for severity.
This change adds explicit configuration semantics consistent with
ADR-009's `[package.metadata.antigen]` schema.

**Amended addition** (new sub-bullet in "Mechanics" section):

> 4. Severity defaults are configurable at the project level via
>    `[package.metadata.antigen]`:
>
>    ```toml
>    [package.metadata.antigen]
>    # Default severity for unaddressed presentations.
>    # "warn" — report and exit 0 (default; per ADR-008 amendment 1)
>    # "error" — report and exit 1 (equivalent to --strict)
>    severity = "warn"  # default
>
>    # Per-antigen override: list antigens that escalate to error.
>    required = ["FrameTranslation", "BoundaryViolation"]
>
>    # Per-antigen override: list antigens that demote to info (no flag).
>    advisory = ["StaleContext"]
>    ```
>
>    The CLI flag `--strict` overrides the configured severity for the
>    invocation. Per-antigen overrides (`required`, `advisory`) are
>    fine-grained: a consumer can warn-by-default for most antigens
>    while requiring strict immunity for a small set of critical
>    failure-classes.

### Change 3: "Named observer experience" documentation as a Sweep A2 deliverable

Scientist F-DOCS-2 finding: ADR-008's enforcement clause requires
"every public API surface includes a 'named observer experience'
section in its docs." Current `antigen/src/lib.rs` and
`cargo-antigen/src/main.rs` documentation is minimal. The enforcement
gap is real but is a Sweep A2 deliverable, not an ADR amendment per
se. This change explicitly names the A2 commitment so it doesn't slip.

**Amended addition** (new bullet in "Sweep-level consequences" section):

> - Sweep A2 W8 (idiomatic refinement) ships "named observer
>   experience" sections on every public API surface
>   (`antigen-macros::antigen`, `antigen-macros::presents`,
>   `antigen-macros::immune`, `antigen-macros::descended_from`,
>   `antigen-macros::antigen_tolerance` per ADR-011, `cargo-antigen
>   scan|audit|new|vaccinate`). Each section names: who the named
>   observer is at that surface, what their typical 60-second flow
>   looks like, what ergonomic affordances exist (scaffolding, error
>   message shape, IDE integration when available), and the
>   multi-contributor expectations (warn-not-error default, when to
>   escalate via `--strict`).

### Related field expansion

**Original**: ADR-008 lists Related: ADR-001, ADR-004, ADR-006, ADR-009.

**Amended Related field**:

> **Related**: ADR-001 (the carriers the named observer authors),
> ADR-003 (biological metaphor — immune response lag predicts the
> warn-not-error default), ADR-004 (elevation), ADR-005 (severity
> defaults are trust-boundary defaults), ADR-006 (recognition —
> multi-contributor lag is observed reality), ADR-009 (adoption
> gradient — warn-not-error IS Layer 1 ergonomics), ADR-011
> (`#[antigen_tolerance]` is the sibling escape valve to
> warn-not-error severity).

### Resolves

- Adversarial ATK-008-1 (multi-contributor workflow friction; the
  "Team A's branch permanently fails until Team B merges" failure
  case).
- Naturalist's Risk A5 entry on multi-contributor workflow friction.
- The conflict between ADR-008's named-observer ergonomics and
  ADR-009's Layer 1 minimum-friction promise.
- The configuration-surface ambiguity in ADR-008 around severity.
- Scientist F-DOCS-2 finding (named observer experience documentation
  gap).

### Open questions deferred to future ADRs

1. **Per-author tolerance scoping**: should tolerance markers track
   which contributor added them? Useful for review accountability;
   risks pseudo-blame patterns. Defer to adoption data.
2. **Severity escalation timing**: when a project transitions from
   warn-default to strict-default (e.g., post-v1.0 release), how does
   the migration work? Bulk tolerance? Per-antigen waiver flow? Defer
   to A5+ when stdlib + adoption data inform the question.
3. **CI-platform integration**: GitHub Actions checks API, GitLab
   merge-request annotations, etc. — should antigen ship recognized
   output formats (SARIF was named in ADR-001 mechanics but is not
   yet implemented)? Defer to A6+ (IDE/CI integration sweep).
4. **Layer 1 vs Layer 3 default-severity differential** (per
   aristotle's Phase 1-8 D2 finding): Layer 3 consumers (with ADR
   registries per ADR-009) are in discipline-mature territory and
   may default to severity = error rather than warn, since strict
   discipline is the operating mode they've already opted into.
   Defer to future ADR; expected to surface when the first Layer 3
   consumer (likely tambear) hits the question and the migration
   path becomes concrete.

---

## [ADR-009] Adoption gradient: antigen meets consumers at any discipline level

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude.

**Related**: ADR-002 (compose, don't compete),
ADR-005 (Layer 3 `adr` field validation is a trust-boundary decision),
ADR-006 (recognition-not-design),
ADR-008 (named-observer terminal stratum),
ADR-010 (fingerprint grammar — required field in Layer 1).

### Finding

Antigen's adoption depends on a hard question: how much architectural discipline does
a consuming project need to have before they can use antigen?

If antigen requires consumers to maintain ratified architectural decision records
(DECs/ADRs), structured changelogs, linked issue trackers, or other "mature project"
artifacts, adoption stalls at projects that already have those — which is a small
minority of Rust codebases.

If antigen can be adopted by a project with only a Cargo.toml, a README, and some
test files, adoption can be broad — early-stage projects, hobby projects, internal
tools, and large codebases without rigorous decision-record practices all become
candidates.

The forgotten-lesson failure mode (ADR-001's motivating problem) is universal. It
hits projects regardless of their architectural-record discipline. Antigen's value
proposition must be available regardless.

### Decision

**Antigen's API is layered into a minimum-viable, enriched, and richest-experience
gradient. Only the minimum-viable layer is required for the tool to function. Higher
layers add traceability and search affordances; none gate basic functionality.**

**Layer 1 — Minimum viable** (works for any project on day one):

```rust
#[antigen(name = "panicking-in-drop", fingerprint = "...")]
pub struct PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for MyType { ... }

#[immune(PanickingInDrop, witness = no_panic_in_drop_test)]
impl Drop for SafeType { ... }
```

Required fields:
- `#[antigen]`: `name` (string identifier), `fingerprint` (structural pattern, see ADR-010)
- `#[presents]`: the antigen type
- `#[immune]`: the antigen type + `witness` (test/proptest/clippy/kani/phantom-type
  reference)

That's it. Two required fields per macro. No internal-doc discipline required.

**Layer 2 — Enriched** (when the project has architectural records or rich context):

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    fingerprint = "...",
    family = "frame-translation",                          // optional class hierarchy
    summary = "Class enums with strongest-first ...",      // optional human description
    references = ["GAP-BIT-EXACT-1", "DEC-030 §1.1"],      // optional open-vocabulary list
)]
pub struct PolarityInvertedClassMeet;
```

Optional fields:
- `family`: maps to one of the 8 first-principles classes or a project-specific family
- `summary`: human-readable description for IDE hover, error messages, audit reports
- `references`: open-vocabulary list (URLs, ADR/DEC IDs, CVE numbers, RFC numbers,
  blog post URLs, internal Notion docs, issue tracker references — anything)

The `references` field's open vocabulary is load-bearing. It accommodates any
project's documentation discipline (or absence of one) without antigen prescribing a
specific schema.

**Layer 3 — Richest** (with project-side ADR/DEC integration when antigen-stdlib v0.2+
supports it):

```rust
#[antigen(
    name = "...",
    fingerprint = "...",
    adr = "ADR-NNN",   // explicit cross-reference to consumer's ADR registry
    family = "...",
)]
```

The `adr` field (and equivalent for tambear's `dec` etc.) is structured cross-reference.
When present, cargo-antigen tooling can validate that the named ADR exists in the
project's `decisions.md` (or configured equivalent), surface it in audit reports,
generate trace links from antigen presentations to ratified decisions, and provide
rich IDE integration (hover shows ADR text inline).

This layer is enrichment, not gating. Projects without ADR registries skip the field;
their experience is identical to Layer 2 minus the structured ADR cross-reference.

### Mechanics

The layers are implemented as **optional macro fields**. The proc-macro accepts both
`#[antigen(name, fingerprint)]` and `#[antigen(name, fingerprint, family, summary,
references, adr)]` and any subset between. Missing fields default to None and produce
no warnings.

The `references` field accepts any string or string array; cargo-antigen does not
validate URL syntax or doc-existence at compile time. Validation happens optionally
at `cargo antigen audit` time, with configurable strictness.

The `adr` field, when present, points to an identifier resolvable in
`Cargo.toml`'s `[package.metadata.antigen]` section:

```toml
[package.metadata.antigen]
adr_registry = "docs/decisions.md"   # or "docs/adrs/"; or omitted
adr_pattern = "ADR-(\\d+)"            # default; configurable for projects using DEC-N or similar
```

If `adr_registry` is configured, `cargo antigen audit` validates that referenced ADR
identifiers exist. If not configured, `adr` field references are stored but not
validated.

### Sweep-level consequences

- The macro design must support optional fields without surface-area warnings
- Cargo.toml metadata schema must include `[package.metadata.antigen]` for
  configuration
- `cargo antigen audit` strictness must be configurable (skip ADR validation for
  projects without registries)
- antigen-stdlib's antigens must work for consumers at all three layers
- Documentation must show the minimum viable example as the primary surface; enriched
  examples as secondary

### Enforcement

- API design review: every new optional field must have a clear default and produce
  no warnings when absent
- Documentation: README and getting-started materials lead with Layer 1 examples
- CI: `cargo antigen audit` on a project without `adr_registry` configured must
  succeed even with antigen presentations and immunities declared

### Resolves

- Adoption barrier for early-stage Rust projects without ADR discipline
- The "antigen requires you to be a tambear-class project" misperception
- Schema rigidity in cross-reference fields (open-vocabulary `references` accommodates
  any documentation practice)

### Open question deferred to future ADR

How does antigen handle CONFLICTING `references` across descended-from chains? e.g.,
parent function cites `ADR-005` but descendant cites `ADR-007` (a partial supersession).
Initial heuristic: cargo-antigen audit reports both; future ADR may refine.

---

## [ADR-010] Fingerprint grammar v1: syn-based AST visitor pattern

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude. Synthesizes ecosystem-composition research
(ast-grep, comby, clippy lint internals, dylint) with academic-context research
(refinement type specification grammars).

**Related**: ADR-001 (structural memory),
ADR-002 (compose, don't compete),
ADR-005 (cross-crate fingerprint inheritance is a trust-boundary decision),
ADR-012 (amends this ADR with function-body patterns + match-context awareness),
ADR-009 (adoption gradient).

### Finding

The `#[antigen(fingerprint = "...")]` field needs a grammar. The grammar specifies
what structural patterns `cargo antigen scan` matches against new code to identify
sites that should be flagged for the antigen.

The grammar's design space spans:
- **Free-text identifier patterns**: shortest path; brittle; cannot match structural
  shape, only names
- **Regex over source**: flexible but unprincipled; misses AST structure; sensitive
  to formatting
- **AST shape match via syn::parse2 + visitor pattern**: principled; matches actual
  Rust syntax; integrates with cargo-antigen's existing AST scanning
- **Tree-sitter based grammar**: cross-language; heavier; introduces tree-sitter as a
  dependency
- **Custom DSL**: full power; high implementation cost; introduces parser/grammar
  maintenance burden

The trade-offs are real. Surveyed ecosystem tools:
- **clippy** uses syn-internal AST visitors with hardcoded pattern matching per lint
- **ast-grep** uses tree-sitter for cross-language structural search
- **comby** uses its own template-based syntax for structural rewrites
- **dylint** allows external clippy-style lints via syn::Visit trait

For antigen's v1, the right balance is: principled enough to match real structural
patterns; light enough to ship quickly; extensible enough to grow into richer
grammars; aligned enough with Rust ecosystem norms (clippy-style) to feel native.

### Decision

**Antigen v1 fingerprints are described as structured Rust expressions, parsed via
`syn::parse2`, evaluated against target code via a visitor pattern over `syn::File`
ASTs. The grammar is Rust-syntax-shaped and compiled at antigen-declaration-load
time.**

The fingerprint surface accepts:
- **Type-name patterns**: glob-style (`*Class`, `Class*`, exact match)
- **Struct/enum/trait kind matchers**: filter by item kind
- **Attribute presence checks**: e.g., `has_attr("derive(PartialEq)")`
- **Field/variant shape matchers**: e.g., `enum_with_4_or_more_variants`,
  `struct_with_field("hi", "f64")`
- **Method-signature patterns**: e.g., `has_method("meet", "(Self, Self) -> Self")`
- **Composition operators**: `all_of`, `any_of`, `not`

Concrete syntax (subject to refinement during implementation):

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    fingerprint = "
        item: enum,
        name: matches('*Class'),
        variants: 3..=8,
        has_method('meet', '(self, Self) -> Self'),
        all_of([
            attr_present('repr(u8)'),
            doc_contains('strength')
        ])
    "
)]
pub struct PolarityInvertedClassMeet;
```

The fingerprint is **a structured expression**, not free text. The grammar is small
enough to learn in 30 minutes. It compiles to a syn-visitor that walks AST nodes
and reports matches.

### Mechanics

**Implementation surface** (lives in `antigen-fingerprint` workspace member, or
`antigen::fingerprint` module):

1. `syn::parse2` parses the fingerprint string into an internal AST
2. The internal AST has variants for each match operator (TypeNameGlob, ItemKind,
   AttrPresent, FieldShape, MethodSignature, Composition)
3. A visitor type implementing `syn::visit::Visit` walks target code's `syn::File`,
   evaluating each fingerprint AST node against AST positions
4. Matches return `Vec<MatchSite>` with file:line positions for `cargo antigen scan`
   output

**Performance**: the visitor pass is `O(n × m)` where `n` is target code AST size
and `m` is fingerprint complexity. For typical projects (10-100k lines, 10-50 active
antigens), scan time should be under 5 seconds. Cargo's incremental compilation and
fingerprint caching apply.

**Extensibility path** (v2+):
- Tree-sitter integration for cross-language fingerprints (when antigen extends
  beyond Rust)
- Pattern macros: shorthand for common patterns (`is_class_enum!()` expands to a
  full fingerprint clause)
- Auto-generation: from a sample failing site, antigen suggests a fingerprint that
  matches it

### Sweep-level consequences

- Sweep A2 (core macros) implements the basic fingerprint parser
- Sweep A3 (cargo-antigen scan) implements the visitor pattern walking target code
- Sweep A4 (composition rules + #[descended_from]) extends fingerprints to handle
  inheritance-aware matching
- Sweep A5 (vaccinate + audit + stdlib antigens) populates antigen-stdlib with real
  fingerprints exercising the grammar

### Enforcement

- Property tests verify each fingerprint operator's behavior against synthetic ASTs
- Adversarial sweep (per ADR-005 sub-clause F) tests fingerprint validation at
  `cargo antigen scan` time: malformed fingerprints fail loudly, not silently
- Documentation includes worked examples of each operator with input/output pairs

### Resolves

- The "what is the fingerprint grammar" open question from `api-shape.md`
- The structural-pattern matching gap identified in `ecosystem-composition.md`
- The need for principled-but-light grammar (vs free text vs full DSL vs heavyweight
  tree-sitter)

### Open questions deferred to future ADRs

1. **Cross-crate fingerprint inheritance**: when an antigen is imported from another
   crate, do its fingerprints re-evaluate against the consuming crate's AST? Or are
   matches cached at the source crate? (Future ADR; v0.2+ work.)

2. **Fingerprint versioning**: when an antigen ships v1.0 with fingerprint F1 and
   later ships v1.1 with refined fingerprint F2, do existing immunity declarations
   need re-validation? (Future ADR; tied to crates.io semver discipline.)

3. **Negative fingerprints**: should `not` operators be allowed at top level (e.g.,
   "match anything that's not X")? Risk: autoimmunity (over-flagging legitimate code).
   Initial position: top-level negation is rejected; `not` is composable inside
   `all_of` / `any_of` only. Future refinement possible.

4. **Performance bounds**: at what point does fingerprint complexity become
   pathological? Initial heuristic: cap fingerprint AST depth at 10; reject beyond.
   Empirical refinement during stdlib development.

These open questions become future ADR-NNNs as the team encounters concrete needs.

---

## ADR-010 Amendment 1 — Disambiguate the parsing path (Path C)

**Status**: Ratified 2026-05-09.

**Amends**: ADR-010 (Fingerprint grammar v1: syn-based AST visitor pattern).

**Participants**: pathmaker (drafted), aristotle (Phase 1-8 verdict:
ratify substantively as drafted), math-researcher (originating
systems-research review identifying Path C), adversarial (ATK pass),
scientist (validation), team-lead (ratification).

**Reason**: ADR-010's example fingerprint isn't valid Rust syntax (uses
single-quoted string-style operators that `syn::parse2::<Expr>`
rejects). The intended path is custom DSL parsing via
`syn::parse::ParseBuffer` peek/parse machinery — math-researcher's
"Path C". The ADR's prose said "parsed via `syn::parse2`" without
specifying that the `T` parameter is a custom DSL parser, not
`syn::Expr`.

### Change

Replace the "Decision" section's example fingerprint with a
tokens-valid version, and replace the prose with the tokens-machinery
specification.

The current ADR-010 Decision section example becomes:

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    fingerprint = r#"
        item = enum,
        name = matches("*Class"),
        variants = 3..=8,
        has_method("meet", "(self, Self) -> Self"),
        all_of([
            attr_present("repr(u8)"),
            doc_contains("strength")
        ])
    "#,
)]
pub struct PolarityInvertedClassMeet;
```

Differences from the pre-amendment example:
- `r#"..."#` raw-string outer wrapper (so inner double-quotes don't
  escape).
- `key = value` pairs (not `key: value`) — Rust idiom; `syn` parses
  cleanly.
- Inner string literals use `"..."` not `'...'` (single-quoted
  character literals in Rust mean `char`, not `&str`).
- Operator parameters are double-quoted.

The Mechanics section's prose:

> 1. `syn::parse2` parses the fingerprint string into an internal AST

Becomes:

> 1. The fingerprint string is tokenized via `syn`'s tokenizer; a
>    custom DSL parser (using `syn::parse::ParseBuffer` peek/parse
>    machinery) consumes the tokens into an internal `Fingerprint`
>    AST. This is Path C from the systems-research review — distinct
>    from `syn::parse2::<syn::Expr>`, which cannot accept the DSL
>    syntax above.

### W6 implementation note (per aristotle R-Q1.D — not part of ratified amendment text)

The fingerprint parser will tokenize the LitStr value via
`proc_macro2::TokenStream::from_str` then parse through Path C.
Spans inside the re-tokenized stream are relative to the inner
string; mapping back to outer `#[antigen(fingerprint = "...")]`
source positions requires span-bridging. This is sibling concern
with W4 (span-aware errors) — flagged in W6 implementation notes,
not blocker for this amendment's ratification.

### Resolves

- The parsing-path ambiguity that would have surfaced as a Sweep A2
  W6 implementation failure (the worked example as written cannot
  parse).
- ADR-005 sub-clause F at the parser boundary: the parsing path is
  now unambiguous; the trust extended to "the example compiles and
  runs" is honest.

---

## ADR-010 Amendment 2 — Fingerprint semver + MSRV policy

**Status**: Ratified 2026-05-09.

**Amends**: ADR-010 (Fingerprint grammar v1: syn-based AST visitor pattern).

**Participants**: pathmaker (drafted; v1 absorbs aristotle's two
refinements R-Q2-A deprecation-then-narrow workflow and R-Q2-B
named-companion-antigen pattern), aristotle (Phase 1-8 verdict),
adversarial, scientist, team-lead (ratification).

**Reason**: Once `antigen-stdlib` ships v0.1 to crates.io, every
fingerprint is part of a public API under semver. Without explicit
policy, breaking changes leak into minor versions silently. Clippy's
MSRV precedent ([clippy::msrv configuration]) is directly applicable.
This was ADR-010 open question 2; math-researcher's review elevated
it to "must land before stdlib publication."

[clippy::msrv configuration]: https://rust-lang.github.io/rust-clippy/master/index.html#/msrv

### Change

Add a "Semver and MSRV" subsection to ADR-010 between "Mechanics" and
"Sweep-level consequences":

> ### Semver and MSRV policy
>
> Once an antigen is published in a crate (whether `antigen-stdlib`,
> a project's local antigen catalog, or a third-party antigen
> library), its fingerprint is part of the crate's public API under
> semver:
>
> - **Broadening a fingerprint** (the new fingerprint matches strictly
>   more sites than the old): minor version bump. Existing consumers
>   may see new flagged sites; this is an additive expansion of
>   recognition.
> - **Narrowing a fingerprint** (the new fingerprint matches strictly
>   fewer sites than the old): **major version bump by default**.
>   Existing consumers may lose coverage they were relying on;
>   tolerance markers and immunity declarations may become orphaned
>   (per ADR-011 stale-tolerance detection).
> - **Deprecation-then-narrow workflow** (alternative to immediate
>   major bump; per aristotle R-Q2-A): narrowings MAY ship in minor
>   versions IF preceded by N..N+M minor versions of deprecation
>   warnings. During the deprecation cycle, audit emits a warning
>   when sites are matched by the broader form but would be
>   unmatched by the narrower form; consumers migrate during the
>   cycle. After the deprecation window, the narrower form ships in
>   the next minor version. Strict immediate narrowing remains
>   major-bump-required when no deprecation cycle is feasible
>   (e.g., when the broader form is actively unsound and consumers
>   should never rely on it). Substrate precedent: clippy and
>   cargo-deny both use deprecation-then-narrow patterns.
> - **Named-companion-antigen pattern** (recommended for material
>   narrowings; per aristotle R-Q2-B): for narrowings affecting >10%
>   of matched sites (heuristic, not formal threshold), introduce a
>   *named companion antigen* with the narrower fingerprint rather
>   than narrowing in-place. The original antigen persists at v1.0
>   shape; consumers opt into the narrower form by adding immune
>   declarations to the new antigen. Both antigens coexist; the
>   broader catches more candidates, the narrower catches the
>   precise pattern. Pattern preserves consumer trust while letting
>   the catalog grow more discriminating.
> - **Adding operators to the grammar**: minor version bump on the
>   `antigen-fingerprint` crate. Forward-compatible: older consumers
>   silently ignore unknown operators (per ADR-009 adoption-gradient
>   tolerance for unknown fields).
> - **Removing operators from the grammar**: major version bump on
>   `antigen-fingerprint`. Breaking; consumers using removed
>   operators stop parsing.
>
> The `#[antigen]` macro accepts an optional `msrv = "1.65"` field
> (parallel to Cargo.toml's `rust-version`):
>
> ```rust
> #[antigen(
>     name = "...",
>     fingerprint = r#"..."#,
>     msrv = "1.75",
> )]
> ```
>
> When present, `cargo antigen scan` skips antigens whose `msrv`
> exceeds the consuming crate's `rust-version` (read from the
> consumer's `Cargo.toml`). This mirrors clippy's MSRV-aware lint
> behavior. The skip is logged under `--verbose` so consumers can
> see which antigens they're missing due to MSRV.
>
> Antigens with no `msrv` field are scanned regardless of the
> consumer's `rust-version` (forward-compatible default).

### Resolves

- ADR-010 open question 2 (fingerprint versioning), before stdlib
  publication forces the issue.
- The substrate-trust vacuum where stdlib consumers had no contract
  for what semver bumps mean.
- The MSRV-induced false-positive class: an antigen using grammar
  operators added in `antigen-fingerprint` v0.3 firing against a
  consumer whose `rust-version = "1.65"` and pulling in
  `antigen-fingerprint` v0.2 transitively.
- The narrowing-without-warning anti-pattern: catalogs that update
  fingerprints in-place without semver discipline silently invalidate
  consumer expectations. The two refinement options
  (deprecation-then-narrow workflow + named-companion-antigen
  pattern) give catalog authors structurally-sound paths.

---

## ADR-010 Amendment 3 — Scan semantics + first-stdlib operators + matcher-engine location + filter framing + invariants

**Status**: Ratified 2026-05-09.

**Amends**: ADR-010 (Fingerprint grammar v1: syn-based AST visitor pattern).

**Participants**: pathmaker (drafted; v1 absorbs aristotle's three
refinements R-Q3 dead-code-elimination clarification, R-Q4
anti-laziness discipline, R-Q5 fourth performance invariant
node_kind dispatch), math-researcher (originating systems-research
review §16 + filter framing folding), aristotle (Phase 1-8 verdict),
adversarial, scientist, team-lead (ratification).

**Reason**: ADR-010 is silent on conditional compilation, macro
expansion, and body-level shape matching. The first canonical seed
antigen (`panicking-in-drop`, per origin.md) requires body-level
operators that v1 grammar as ratified doesn't provide. The matcher
engine's location is also unspecified, leading to circular-dependency
risk between `antigen-macros` (compile-time validation) and `antigen`
(scan-time matching). This amendment also folds in the filter-vs-proof
framing crystallized during A1 in scout's framing + math-researcher's
review §16.

### Clause A: scan semantics — pre-expansion only

> ### Scan semantics: what's scanned
>
> v1 fingerprints match **pre-expansion source** only. Macro-generated
> code is invisible to the scan — derive macros, declarative macros,
> and proc-macros all expand outside the scan's view. `cargo antigen
> scan` reports "N items in macro-generated code were not scanned"
> when macro expansion sites are detected (transparency, not blame).
> Coverage of macro-generated code is the structural-blindness pair
> addressed by ADR-014 (`#[antigen_generates]`) in a future sweep.

### Clause B: cfg handling — match-through by default

> ### Conditional compilation
>
> v1 fingerprints match **all items regardless of `#[cfg]` gates** by
> default. A site under `#[cfg(target_os = "linux")]` is matched
> whether the consumer's compile target is linux or not. The default
> is match-through because:
>
> 1. Failure-classes the antigen names exist whenever the source is
>    parseable, regardless of whether THIS build target compiles them.
> 2. Compile-target-specific scans miss vulnerabilities in
>    cross-platform code that other consumers' builds will execute.
> 3. Match-through aligns with the filter framing (Clause D below):
>    the scan is recall-tuned; let the witness layer reason about cfg.
>
> Consumers needing cfg-aware scans use `cargo antigen scan
> --respect-cfg` for strict-mode behavior. Per-target invocation is
> configurable in `[package.metadata.antigen]`.

### Clause C: first-stdlib operators

> ### First-stdlib operators (v1 grammar additions)
>
> v1 grammar adds:
>
> - **`body_contains_macro(name)`** — matches when the
>   function/method body contains a macro invocation whose name
>   matches `name`. Required for the canonical seed antigen
>   `panicking-in-drop` (which detects `panic!`, `unreachable!`,
>   `todo!`, `unimplemented!` in `Drop` impl bodies). Implemented as a
>   `syn::Block` walk for `syn::Macro` invocations. **Native syn
>   walker, NOT delegated to the body-pattern engine** (per ADR-015
>   §S2 + R-3).
>
> v1 grammar **gestures at** but does NOT ratify:
>
> - **`enum_discriminant_ordering(strongest_first | weakest_first |
>   unspecified)`** — needed for `polarity-inverted-class-meet`-shaped
>   antigens. Deferred to the first stdlib release sweep (likely A5)
>   because it requires analysis of explicit discriminant expressions
>   on `syn::Variant::discriminant` and a stable convention for what
>   "strongest" means when discriminants are user-supplied.
>   Recognition-driven (ADR-006): ratify when stdlib content
>   exercises it, not speculatively.
>
> Other operators ratified in the original ADR-010 (type-name
> patterns, item-kind matchers, attribute presence checks,
> field/variant shape matchers, method-signature patterns,
> composition operators) ship in v1 unchanged.

### Clause D: filter-vs-proof framing — folded in from scout/math-researcher §16.1

> ### Semantic posture: fingerprints filter, witnesses prove
>
> Fingerprints are **recall-tuned candidate filters**. They identify
> *sites that may exhibit the failure-class*, not *sites that
> definitely do*. The witness mechanism (per ADR-001 + ADR-002) is
> what proves immunity at each candidate site. This split is
> intentional:
>
> - `cargo antigen scan` finds candidates fast (recall-tuned, accept
>   moderate false-positive rate).
> - `cargo antigen audit` validates witnesses where they apply
>   (precision lives in the witness layer per ADR-002 composition).
> - `#[antigen_tolerance(...)]` (per ADR-011) marks sites that are
>   matches-by-design and not vulnerabilities.
>
> Consequences for grammar design:
>
> 1. Cheap syntax-level operators (item-kind, name pattern, attribute
>    presence) are sufficient for v1. Expensive operators (call-graph
>    analysis, HIR type resolution) are NOT structurally guaranteed by
>    ADR-007; they may land if the witness layer can't cover the
>    precision case, but the default is to push precision to the
>    witness.
> 2. False positives from the filter are EXPECTED and not failure
>    states. ADR-011 tolerance is the structural relief valve.
>    Without the filter framing,
>    autoimmunity-via-broad-fingerprint becomes a design crisis;
>    with the framing, it's load-bearing recall.
> 3. The grammar surface (which operators exist) is structurally
>    guaranteed by ADR-007. Each individual operator is
>    recognition-driven (ADR-006) — added when stdlib content
>    exercises it. The filter framing makes the staged rollout
>    coherent: ship the parser + dispatch + name-glob in A2; add
>    operators per stdlib need.
>
> **Anti-laziness discipline** (per aristotle R-Q4): recall-tuned
> does NOT mean unboundedly broad. Stdlib authors SHOULD calibrate
> fingerprints against a representative corpus
> (`docs/expedition/failure-class-instances.md`) before publishing.
> A fingerprint matching >50% of items in any item-kind bucket
> (heuristic, not formal threshold) signals over-broadness; such
> fingerprints either narrow before publishing or cite specific
> evidence justifying the breadth. ATK-001-1's audit-side
> autoimmunity-rate signal (deferred to A4+) is the systematic
> enforcement of this discipline; until it ships, code review is
> the discipline.

### Clause E: matcher engine location — workspace topology

> ### Implementation surface: `antigen-fingerprint` workspace member
>
> The fingerprint parser + visitor lives in a separate workspace
> member `antigen-fingerprint`, depended-on by both:
>
> - `antigen-macros` — for compile-time validation that
>   `#[antigen(fingerprint = "...")]` parses cleanly (the macro is
>   identity-transform-with-validation; rejecting at parse time
>   avoids shipping malformed fingerprints to crates.io).
> - `antigen` — for scan-time matching against target-code ASTs.
>
> Both `Fingerprint::parse(s)` AND `Fingerprint::matches(item)`
> live in `antigen-fingerprint` (per aristotle R-Q3).
> `antigen-macros` consumes parse-only (compile-time validation);
> `antigen` + `cargo-antigen` consume both. The matcher's
> per-consumer compile cost in `antigen-macros` is negligible — cargo
> dead-code elimination removes unreachable visitor code at the
> macro-crate compile boundary, so consuming the parser does not pay
> for the matcher.
>
> Without this split, `antigen-macros` (proc-macro = true) would have
> to re-implement the parser internally, and `antigen` would have to
> re-implement it again at scan time. The duplication was the
> substance of ATK-001-2 in pre-team scaffolding (the dual-parser
> drift bug). The workspace split makes the parser canonical.
>
> The `antigen-fingerprint` crate is itself a Sweep A2 W6 deliverable.
> Its public API is the parser + the `Fingerprint::matches(item:
> &syn::Item) -> bool` method. Internal state is private; future
> performance optimizations (incremental scan, caching) live behind
> the API.

### Performance invariants (load-bearing)

> ### Implementation invariants (load-bearing)
>
> The 5-second performance budget for typical workspaces (10-100k
> LoC, 10-50 antigens) requires four implementation invariants:
>
> 1. **Single-pass walks**: `cargo antigen scan` walks each `.rs`
>    file exactly once per invocation; all fingerprints are
>    evaluated on the same pass via fan-out at the visitor level.
> 2. **Pre-parsed pattern signatures**: signatures inside operators
>    like `has_method("meet", "(self, Self) -> Self")` are
>    canonicalized via proc_macro2 round-trip and stored as
>    canonical strings at fingerprint-load time, NOT re-parsed per
>    match site. The naive per-match-site re-parse is a documented
>    50× slowdown (math-researcher §4.1). See ADR-010 Amendment 5
>    for the canonicalization path; the `syn::Signature` AST
>    comparison remains a future upgrade target.
> 3. **Parse-time depth + node-count caps**: the fingerprint parser
>    rejects fingerprints with AST depth > 10 OR total node count >
>    256 (defaults; configurable in `[package.metadata.antigen]`).
>    This prevents pathological fingerprints from blowing up the
>    matcher.
> 4. **Node-kind dispatch at the visitor** (per aristotle R-Q5; also
>    named in ADR-015 §Performance invariants): the
>    `Fingerprint::matches` evaluator dispatches by `node_kind` at
>    the top of each visit. Only fingerprints with matching
>    `node_kind` are evaluated against the current AST node. For N
>    fingerprints sharing common node-kinds (e.g., 20 `item: enum`
>    fingerprints), the per-node cost is O(matching-fingerprints)
>    not O(N). Without this invariant, scan time scales linearly
>    with total fingerprint count regardless of relevance.
>
> Without these invariants, the 5-second budget blows out 5-10× per
> math-researcher §4. The invariants are required for v0.1.0 release.

### Sweep-level consequences additions

> - **Sweep A2 (W6)**: ships `antigen-fingerprint` workspace member;
>   ships parser + item-kind dispatch + name-glob operator; ships
>   `body_contains_macro` operator (native syn walker); ships `msrv`
>   macro-field; ships single-pass walks + pre-parsed signatures +
>   depth/node-count caps + node-kind dispatch. Defers other
>   operators per the filter framing.
> - **Sweep A3+**: incremental scan cache (per-file mtime +
>   content-hash keyed) — math-researcher §14 names this as
>   required for IDE integration; not load-bearing for v0.1.0
>   release but high-priority for v0.2+.
> - **Sweep A5 (stdlib)**: `enum_discriminant_ordering` operator
>   added when first stdlib antigen exercises it.

### Open-question tightenings

In addition to the substantive amendments, three of ADR-010's open
questions tighten without deferring further:

**Open Question 1 (cross-crate fingerprint inheritance)**: the larger
question stays deferred to a future ADR. Lock the static-fingerprint
invariant in ADR-010 itself:

> Fingerprints are static at the antigen's declaration site — they
> do not re-evaluate against the consuming crate's AST when the
> antigen is imported across crates. Cross-crate matching walks the
> consumer's AST against the source crate's parsed `Fingerprint` AST.

**Open Question 3 (negative fingerprints)**: tighten from "top-level
negation rejected" to:

> The `not` operator is valid only inside `all_of`, only as a
> sibling of at least one positive matcher. `not` as a direct child
> of `any_of` is rejected at parse time. This closes the De Morgan
> promiscuity loophole where `any_of([not(A), not(B)])` becomes
> `not(all_of([A, B]))` via De Morgan and re-creates top-level
> negation.

**Open Question 4 (depth cap)**: tighten from "cap fingerprint AST
depth at 10" to:

> Cap fingerprint AST depth AND total node count. Defaults: depth
> 10, total nodes 256. Both checked at parse time; both configurable
> in `[package.metadata.antigen]` (`fingerprint_max_depth`,
> `fingerprint_max_nodes`).

### Resolves

- The cfg-handling silence in original ADR-010 (Clause B).
- The macro-expansion silence (Clause A; structural-blindness pair
  to ADR-014).
- The body-level operator gap (Clause C; canonical seed antigen
  `panicking-in-drop` was structurally blocked).
- The matcher-engine-location ambiguity (Clause E; ATK-001-2
  dual-parser drift surfaced from this gap).
- Performance budget honesty (Invariants); the 5-second claim was
  speculative without these invariants ratified.
- Adversarial ATK-010-2 (math-researcher §11; performance estimate
  speculation acknowledged).

---

## ADR-010 Amendment 4 — Filter/proof framing as architectural principle

**Status**: Ratified 2026-05-09.

**Amends**: ADR-010 (Fingerprint grammar v1: syn-based AST visitor pattern).

**Participants**: pathmaker (drafted; integration commit), navigator
(full-body spec), aristotle (Track 1 P1-8 + bundle deconstruction
identifying the framing as elevation-worthy), team-lead (ratification).

**Related**: ADR-001 Amendment 1 (carrier-strength + witness-validity
tiers; the filter-vs-proof split is the operational form of the
tiered-substrate pattern); ADR-002 (compose-don't-compete; filter
framing is what makes composition with witnesses natural);
ADR-010 Amendment 3 Clause D (where the framing was first ratified
as part of the bundle); ADR-011 (`#[antigen_tolerance]` is the
structural relief valve for filter false-positives); ADR-012
(function-body patterns inherit the framing); ADR-015 (engine
architecture preserves filter/proof at the body-pattern delegation
boundary).

### Reason

Amendment 3 Clause D ratifies filter-vs-proof as part of a bundle of
mechanics changes, but the framing is *load-bearing across ADR-010 as
a whole*, not just a single clause inside Amendment 3. Subsequent
readers of ADR-010 looking for the operative semantic posture will
find it buried inside Amendment 3 rather than elevated to a top-level
named principle. This amendment promotes the framing from
clause-position to top-level-principle-position, naming it as the
operative semantic posture across the entire ADR.

The framing is not new content — Amendment 3 Clause D is the
canonical statement and remains so. Amendment 4 names the framing as
*the operative posture for ADR-010 readings going forward*: when
future amendments, future operators, or future implementation
decisions for ADR-010 surface, they are evaluated against the
filter/proof split as the architectural commitment.

### Decision

**Antigen's recognition surface is split into a filter layer
(fingerprints) and a proof layer (witnesses). The filter is
recall-tuned; the proof is precision-tuned. ADR-010's grammar serves
the filter layer; the proof layer composes via ADR-002 with external
witness mechanisms.**

The principle, stated normatively as the operative architectural
posture for ADR-010:

> **Fingerprints filter; witnesses prove.** The fingerprint grammar
> ratified by ADR-010 is recall-tuned: it identifies *candidates*
> that may exhibit the failure-class. Precision — the determination
> of whether a candidate is *actually* a vulnerability — lives in
> the witness layer, composed via ADR-002 with tests, proptests,
> formal-verification adapters, phantom-type proofs, or external
> tools. ADR-011 (`#[antigen_tolerance]`) is the structural relief
> valve for filter false-positives that are matches-by-design.

### Consequences for grammar design

The filter/proof split has three load-bearing consequences:

1. **Cheap syntax-level operators are sufficient for v1.**
   Item-kind, name pattern, attribute presence, body-macro presence
   — these are all O(syntactic-shape) operations. Expensive
   operators (call-graph analysis, HIR type resolution, MIR
   reasoning) are NOT structurally guaranteed by ADR-007: they may
   land when the witness layer cannot cover the precision case, but
   the default is to push precision to the witness rather than
   inflate the filter.

2. **False positives from the filter are EXPECTED and not failure
   states.** ADR-011's `#[antigen_tolerance]` is the structural
   relief valve: a site that matches a fingerprint by design (not
   by vulnerability) gets a tolerance marker with required
   rationale. Without the filter framing,
   autoimmunity-via-broad-fingerprint becomes a design crisis
   ("the scan reports too much; the grammar is broken"); with the
   framing, the same observation is load-bearing recall ("the
   scan finds candidates; the witness or tolerance proves the
   per-site disposition").

3. **The grammar surface is structurally guaranteed; individual
   operators are recognition-driven.** Per ADR-007, the project
   commits to recognition coverage of the 8-class failure
   taxonomy. Per ADR-006, individual operators are added when
   stdlib content exercises them. Filter/proof makes the staged
   rollout coherent: ship the parser + dispatch + a small set of
   operators in A2; expand the operator set as stdlib antigens
   surface needs.

### Mechanics

The amendment is principle-elevation, not new mechanics.
Implementation details remain in Amendment 3 Clause D (the original
ratification site). Future amendments to ADR-010 reference *this
amendment* as the architectural commitment when justifying
operator additions, performance trade-offs, or composition rules
involving the witness layer.

### Sweep-level consequences

- **Sweep A2 (current)**: W6 ships the filter layer (parser +
  dispatch + initial operators per Amendment 3 Clause C). W7
  ships the proof layer's witness-tier API (per ADR-013 + scout's
  W7 design). The two ship together because they are operational
  halves of the same architectural commitment.
- **Sweep A3+**: composition rules for the filter/proof boundary
  become explicit (when does a witness invalidate a filter match?
  When does a tolerance? When does a `descended_from` chain
  inherit either?).
- **Future ADRs**: when a future amendment proposes adding an
  expensive operator (HIR/MIR reasoning, cross-crate analysis),
  the review asks "can this live in the witness layer instead?"
  per filter/proof. The default is push-to-witness; expensive
  filter operators require structural argument why witness
  composition cannot suffice.

### Enforcement

- **Process**: ADR-template Phase 1-8 review prompt (per
  `process.md`) asks of every ADR-010 amendment: "Does this
  belong in the filter layer (fingerprint) or the proof layer
  (witness)? If filter, can it instead live in proof? If proof,
  why does this ADR amend ADR-010?"
- **Adversarial**: ATK seeds against new ADR-010 amendments
  include "filter/proof boundary violation" as a default attack
  pattern.

### Resolves

- The implicit-but-unnamed semantic posture across ADR-010 + its
  amendments. Pre-Amendment-4, future readers of ADR-010 had to
  chase Amendment 3 Clause D to find the operative principle;
  Amendment 4 makes the principle top-level.
- The architectural-posture-vs-clause asymmetry: Amendment 3
  ratified the framing as one clause among five; Amendment 4
  acknowledges that the framing is the operative posture for
  the ADR's architectural reading going forward.
- The future-ADR review gap: when a future amendment proposes
  expanding the filter, the question "does this belong in proof
  instead?" is now structural rather than ad-hoc.

---

## ADR-010 Amendment 5 — `has_method` signature canonicalization via proc_macro2 (strict)

**Status**: Ratified 2026-05-11.

**Amends**: ADR-010 (Fingerprint grammar v1), specifically Performance
Invariant 2 in ADR-010 Amendment 3.

**Participants**: scout (first author, A3.5 onboarding sweep); aristotle
(Phase 1-8 audit + small-push audit); adversarial (Stage 3 review +
ATK-W6a-017 Mechanism B negative test); pathmaker (OQ1 STRICT implementation
at bb22e56); team-lead (OQ1 STRICT adjudication + ratification).

**Related**: ADR-010 Amendment 3 (Performance Invariant 2 — pre-parsed
signature storage); ADR-015 (grammar-vs-vocabulary cut + per-operator
implementation principle); ADR-005 §1 sub-clause F (trust boundary
validation — grounds the strict-fail decision).

**Implements**:
- `00c35ed` — initial engine canonicalization via proc_macro2 round-trip (lenient form, superseded by bb22e56)
- `bb22e56` — OQ1 STRICT: `normalize_signature_canonical` → `Option<String>`, strict fail on proc_macro2 parse error
- `af4113c` — fingerprint-declaration corrections including ADR-010 ratified text at lines 1724 + 1844
- `cd33c96` — ATK-W6a-017: Mechanism B negative test, three assertions

### Finding

The `has_method` operator compares user-written pattern strings against
proc_macro2-rendered method signature strings. The comparison uses whitespace
normalization only, which is insufficient: token-level differences between
natural Rust syntax and proc_macro2-rendered output survive whitespace
normalization and cause silent mismatches (no error, no diagnostic — zero
fingerprint matches returned).

This failure-class — **user-pattern-string vs engine-rendered-string
tokenization asymmetry** — has two known sub-mechanisms with different
mitigations:

**Sub-mechanism A (whitespace/spacing asymmetry)**: natural Rust groups
tokens like `&mut` without intervening spaces; proc_macro2 renders them with
spaces (`"& mut"`). Whitespace normalization collapses multi-space runs but
cannot insert missing spaces. Example: `"(&mut self)"` → normalized →
`"(&mut self)"`; engine renders → `"(& mut self)"`. These are not equal.

**Sub-mechanism B (token-class distinction)**: `Self` (type alias, capital-S,
`Ident` token) and `self` (receiver keyword, lowercase, `Ident` token with
keyword semantics) are categorically different tokens. proc_macro2 preserves
the distinction; `Self` cannot be silently bridged to `self` without changing
semantics (a fingerprint targeting a static method taking two `Self`-typed
parameters would be incorrectly re-shaped).

Both sub-mechanisms were discovered during A3.5's Phase 3 coherence review
by cross-checking doc examples against actual matcher behavior.

**ADR-tier catch (V8 verifier-self-correction)**: the ratified ADR-010 text
at lines 1724 and 1844 carried `"(Self, Self) -> Self"` for the
`PolarityInvertedClassMeet` fingerprint for the entire post-ratification
period. This is the correct form for a static method taking two `Self` typed
parameters; the intended form (by-value receiver) is `"(self, Self) -> Self"`.
The error was invisible at spec-review level; only cross-checking concrete
engine behavior surfaced it. Fixed at `af4113c`.

**Recursive V8 meta-finding**: the amendment's own proposal draft overclaimed
scope — the "Failure-class eliminated" section listed sub-mechanism B as closed
by proc_macro2 round-trip, but round-trip does NOT bridge token-class
distinctions (Self/self are different tokens that remain different after
tokenization). Aristotle's Phase 1-8 caught this overclaim during the review.

### Decision

#### Clause A — Operative principle: pre-tokenize `has_method` pattern strings

User-provided signature strings in `has_method("name", "sig")` are
canonicalized through proc_macro2's tokenizer at fingerprint-parse time. The
canonical form is used for all comparisons against engine-rendered actual
signatures.

This is an upgrade to Performance Invariant 2 from ADR-010 Amendment 3.
PI-2 ratified *when* (load-time) and *what is produced* (pre-parsed
signatures). This clause specifies the normalization path: the user-provided
string is run through `proc_macro2::TokenStream::from_str` before whitespace
normalization, producing the same token spacing the matcher uses when rendering
actual `syn::Signature` instances.

**Strict fail on parse failure (OQ1, ratified strict, ADR-005 §1 sub-clause F)**:
if proc_macro2 cannot tokenize the user's string (malformed, unbalanced
delimiters), the engine returns an error — it does NOT fall back to plain
whitespace normalization. The lenient fallback would reintroduce the spacing
asymmetry on the degraded path (match-site always produces proc_macro2-canonical
form; pattern side on fallback would produce only whitespace-normalized form —
asymmetry preserved). This is an ADR-005 sub-clause F violation at the engine
trust boundary: a pattern that cannot be canonicalized cannot be compared
symmetrically against engine-rendered signatures. Valid v1 `has_method` signature
strings are always tokenizable by proc_macro2; inputs that fail tokenization are
already silently broken and must surface an error.

#### Clause B — Sub-mechanism A: whitespace/spacing asymmetry (engine-side fix)

Engine-side fix: proc_macro2 round-trip canonicalization (Clause A) closes
sub-mechanism A entirely. After round-trip, `"(&mut self)"` and `"(& mut self)"`
produce the same normalized form. Users may write either; the engine accepts both.

Implemented as `normalize_signature_canonical()` in
`antigen-fingerprint/src/lib.rs`. The fix is internal to the parse-time path;
no external API change; no match-site change; fully non-regressive on
previously-working fingerprints.

#### Clause C — Sub-mechanism B: token-class asymmetry (docs-and-declarations)

Engine cannot bridge this: `Self` → `self` auto-bridging would change semantics
(silently re-shape fingerprints targeting static methods into
receiver-shape fingerprints). Mitigation is documentation and correct
declarations.

Mitigations:
- **Receiver-rendering reference table** in `docs/fingerprint-grammar.md`:
  explicitly maps each receiver form (`self`, `&self`, `&mut self`, no receiver)
  to its pattern form.
- **Fingerprint-declaration corrections**: `PolarityInvertedClassMeet` in
  tambear's `antigens.rs`, `docs/expedition/stdlib-seed-antigens.md`, and
  the two ADR-010 ratified-text instances corrected from `"(Self, Self) -> Self"`
  to `"(self, Self) -> Self"`.

This distinction — `Self` (typed parameter) vs `self` (receiver keyword) — is
a Rust grammar-level distinction, not a fingerprint-grammar-level limitation.

#### Clause D — Triage discipline for future sub-mechanisms

The two known sub-mechanisms establish the triage protocol for any future
tokenization-asymmetry discovery:

1. **Can a pure tokenizer (no semantic position analysis) close this gap?** If
   the difference is surface formatting that tokenizes identically regardless of
   grammatical role (whitespace, spacing between punctuation and identifiers)
   → engine-bridgeable. If closing the gap requires knowing what grammatical
   role a token plays (receiver position vs typed-parameter position, keyword vs
   identifier in context) → semantic-distinction.

2. If **engine-bridgeable**: apply engine-side fix at parse time. Update
   `normalize_signature_canonical` or equivalent. Ship as patch (non-regressive
   on valid inputs).

3. If **semantic-distinction**: mitigation is docs and correct declarations.
   Update `docs/fingerprint-grammar.md` with the distinction; update affected
   fingerprint declarations. Do NOT auto-bridge in the engine.

### Mechanics

`normalize_signature_canonical` in `antigen-fingerprint/src/lib.rs`
(as of `bb22e56`):

```rust
pub(crate) fn normalize_signature_canonical(sig: &str) -> Option<String> {
    use std::str::FromStr;
    let stream = proc_macro2::TokenStream::from_str(sig).ok()?;
    Some(normalize_ws(&stream.to_string()))
}
```

Returns `None` when proc_macro2 rejects the input. Callers surface `None` as
a parse error with diagnostic context. Called from `parse_has_method()` in
`antigen-fingerprint/src/parser.rs` at fingerprint-parse time.

The match-site path (`render_inputs()`, `signature_matches()` in
`antigen-fingerprint/src/matcher.rs`) is unchanged. The fix is entirely
parse-time.

### Scope boundary

This amendment covers `has_method` signature strings only:

- `name = matches("<glob>")` — glob on identifier strings; single tokens; no spacing variation
- `attr_present("<path>")` — path-segment identity; no tokenization comparison
- `doc_contains("<substring>")` — substring search in doc text; no tokenization
- `body_contains_macro("<name>")` — last-segment match; no tokenization

Does not extend to semantic comparison (resolving types, understanding `Self`
in context). Comparison remains textual after normalization.

### ATK contracts

**ATK-W6a-013** (`antigen-fingerprint`): the `"(&self)"` / `"(&mut self)"`
spacing footgun. Clause B closes this for the spacing variant; the test was
inverted at `00c35ed` to assert corrected behavior.

**ATK-W6a-013b** (new): `Self` vs `self` token-class distinction. Separate
ATK number because the mitigation surface is different (docs, not engine).
Mitigation at `af4113c`.

**ATK-W6a-017** (Mechanism B negative test, `cd33c96`): three assertions:
(1) `"(Self, Self) -> Self"` does NOT match `fn meet(self, other: Self)` —
core guard for engine NOT bridging the distinction; (2) `"(self, Self) -> Self"`
DOES match `fn meet(self, other: Self)` — positive control; (3) `"(& self,
Self) -> Self"` DOES match `fn meet(&self, other: Self)` — reference-receiver
positive control.

### Amendment cascade (downstream consistency — not gating)

1. **ADR-010 Amendment 3 PI-2**: "stored as `syn::Signature` AST nodes" →
   "normalized via proc_macro2 round-trip at parse time; stored as canonical
   string." The `syn::Signature` AST comparison remains a future upgrade path;
   v1 contract is canonical string after proc_macro2 round-trip.

2. **`docs/fingerprint-grammar.md`**: the receiver-spacing caveat section
   ("Never matches (silent failure)" examples) should note that sub-mechanism A
   is now engine-bridged — users may write either spacing form.

3. **Tambear adoption log**: the PanickingInDrop fingerprint's `"(&mut self)"`
   fix committed at `7d9664a` is no longer load-bearing post-Amendment 5; kept
   for clarity.

---

## [ADR-011] `#[antigen_tolerance(...)]`: opt-out for legitimate fingerprint matches

**Status**: Ratified 2026-05-08.

**Participants**: pathmaker (drafted), aristotle (reciprocal Phase 1-8), adversarial
(ATK pass), scientist (validation pass).

**Related**: ADR-001 (carrier set; this is a C6 carrier addition), ADR-003
(autoimmunity prediction from biological metaphor), ADR-005 (sub-clause F at trust
boundaries; tolerance is itself a trust boundary), ADR-006 (recognition not design;
required-rationale enforces recognition), ADR-007 (anti-YAGNI: autoimmunity opt-out
structurally guaranteed), ADR-009 (adoption gradient: tolerance is a Layer-1 surface),
ADR-010 (fingerprint grammar; tolerance interacts with scan pass).

### Finding

The fingerprint mechanism (ADR-010) commits us to scanning unmarked code for
structural matches against declared antigen fingerprints. When `cargo antigen
scan` finds an unmarked site that matches an antigen's fingerprint, it flags the
site as needing `#[presents(X)]` + `#[immune(X, witness = ...)]`.

**Some matches are false-positives by design**: test fixtures that deliberately
exhibit the antigen pattern to test the witness; `examples/broken_witness.rs` (which
already ships) demonstrating audit catching the failure-class; code-generation sites
where the pattern is fine because the generation context is correct; legitimate
domain-specific exceptions; **autoimmunity protection** — the immune-system metaphor
(ADR-003) predicts that a recognition system without tolerance over-flags legitimate
code and adoption fails.

**The substrate already silently committed to a name for this.** The scan output
already directs users to `#[antigen_tolerance(...)]` (`cargo-antigen/src/main.rs:185`),
but no such attribute is defined anywhere. This is a sub-clause F violation: the
scan output extends trust to a mechanism whose enforcement is missing.

### Decision

**Define `#[antigen_tolerance(...)]` as a first-class macro alongside the four core
macros, with required justification and optional duration.**

```rust
#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "This test fixture deliberately constructs the failure pattern \
                 to verify the witness catches it.",
    until = "v1.0",  // optional; default = forever
    see = ["GAP-BIT-EXACT-1"],  // optional open-vocabulary
)]
fn test_polarity_inversion_caught() { ... }
```

**Required**: antigen type (positional), `rationale` (non-empty string).
**Optional**: `until` (non-empty if present, per aristotle reciprocal Phase 1-8),
`see` (open-vocabulary string array mirroring ADR-009's `references`).

A single item can stack multiple tolerances against different antigens. Tolerance
is **item-level only** in v1; module-level deferred to future ADR.

### Mechanics

- Macro expansion: identity transform with attribute-arg validation. No runtime code.
- Scan integration: when scan finds an unmarked fingerprint match, it checks for
  `#[antigen_tolerance(X)]` on the same item via item-identity matching (Sweep A2's
  W3). Matches with valid tolerance report under `tolerated`.
- Audit reports tolerances in three categories: **active** (`until` not expired),
  **expired** (`until` set and passed), **no-expiry** (forever-tolerances surfaced
  via `--list-tolerances`).
- `--strict` mode treats expired tolerances as failures.

**Trust-boundary check (ADR-005)**:
1. Empty rationale rejected at parse time.
2. Empty `until` rejected at parse time (per aristotle reciprocal Phase 1-8 —
   meaningless empty expiry indicates user error).
3. Antigen type must be discoverable in the workspace or imported from a crate.
4. Item-level placement only in v1.
5. **Tolerance dominates over `#[presents]` on the same item** (per aristotle
   ATK-011-3): site is reported as tolerated; the `#[presents]` marker is dead
   code; audit warns to remove one or the other.

### Sweep-level consequences

- **Sweep A2 (W6 fingerprint grammar)**: grammar recognizes `#[antigen_tolerance]`
  on items during AST walk; matches against fingerprints check tolerance presence.
- **Sweep A2 (W10 — added)**: implement the `antigen_tolerance` macro in
  `antigen-macros`. Add `TolerationDeclaration` to the `scan` module. Add
  `tolerated_count` to `AuditReport` and update human/JSON output.
- **Sweep A5 (audit completeness)**: `cargo antigen audit --list-tolerances`
  subcommand listing all tolerances with rationale + expiry.
- **antigen-stdlib v0.1**: stdlib examples demonstrate tolerance usage where
  appropriate.

### Enforcement

- `antigen_tolerance` macro shipped as part of v0.1.0 release.
- CI gate: `cargo antigen audit --strict` fails on expired tolerances.
- Trybuild fixtures: empty rationale rejected, missing antigen rejected,
  empty-until rejected, expired-until parsing, stacked tolerances accepted.

### Resolves

- The substrate's silent commitment in `cargo-antigen/src/main.rs:185`.
- The autoimmunity prediction from the biological metaphor (ADR-003).
- ADR-010's open question 3 (negative fingerprints / autoimmunity risk) — partial
  answer: tolerance-as-opt-out is the v1 mechanism; negative fingerprints stay
  deferred.
- The "false-positive flagging" risk from `revolutionary-and-not.md`.
- The bootstrap-blocker per aristotle ATK-011-5: the project's own
  `examples/broken_witness.rs` is the first auto-flag candidate when W6 lands;
  ADR-011 ratification is structurally urgent for that to ship coherent.

### Open questions deferred

1. File-level / module-level tolerance vs item-level (item-only in v1).
2. Tolerance inheritance via `#[descended_from]` (no inheritance in v1; each
   descendant re-justifies).
3. Cross-crate tolerance (yes; consumer-side context is the use case; mechanism
   in Sweep A3).
4. CI default-warn-not-fail vs strict on tolerance presence.
5. Bypass-detection for rationale-stuffing (no automated mechanism in v1; future
   ADR may add rationale-quality lint informed by naturalist's biology framing).

---

## [ADR-012] ADR-010 Amendment 1: function-body patterns + match-context awareness

**Status**: Ratified 2026-05-08. Implementation deferred to Sweep A4-A5.

**Amends**: ADR-010 (Fingerprint grammar v1).

**Participants**: pathmaker (drafted), math-researcher (systems-research), aristotle
(reciprocal Phase 1-8), adversarial (ATK pass), scientist (validation pass).

**Related**: ADR-001 [as amended] (C5 drift-detection-at-scan-time invariant),
ADR-002 (compose, don't compete; clippy pattern engine reuse), ADR-005 (sub-clause
F: structural blindness IS a sub-clause F violation), ADR-006 (recognition not
design; tambear adoption surfaced this), ADR-007 (anti-YAGNI: function-body
matching structurally guaranteed), ADR-010 (the ADR being amended), ADR-014
(sibling structural-blindness fix for macro-generated code).

### Finding

ADR-010's v1 grammar matches at the **item level** — declarations and signatures.
Tambear's adoption log (entry 2026-05-07, `UlpDistanceRolledByHand` antigen)
surfaced a real gap: two newly re-rolled ULP-distance functions in tambear escaped
detection because the existing pattern detector only catches the inline
single-expression form, not the multi-statement function-body form.

This is the structural sibling of adversarial's ATK-010-1 (macro-expansion
blindness): both are *structural blindness* where the failure-class exists in
executed code but not in the syntactic surface ADR-010's v1 grammar walks.

A failure-class can manifest as either signature-shape or body-shape. A v1-only
grammar catches the first and misses the second — and the failure-class memory
degrades silently because the audit reports "0 unaddressed presentations" while
the second form ships.

### Decision

**Extend ADR-010's grammar with body-level operators and match-context awareness in
v2 (target: Sweep A4-A5).**

New operators: `body_contains: ...`, `body_pattern: 'name'`, `expr_call: 'path'`,
`expr_macro: 'macro_name'`, `statement_count_in: M..=N`.

Match contexts surfaced via `MatchContext`: `kind: ItemMatch | BodyMatch |
GeneratedMatch`; `confidence: High | Medium | Low`.

### Mechanics

The v2 grammar is a **non-breaking extension** of v1: v1 fingerprints continue to
parse and run unchanged.

**Pre-parsed-pattern invariant** (per aristotle reciprocal Phase 1-8 +
math-researcher §4.1): body-level operators MUST be pre-parsed at fingerprint-load
time, not per-match-site. Without this invariant, body-level operators exhibit
the 50× constant-factor cost asymmetry math-researcher flagged for v1's
`has_method`.

Performance impact: `O(n × m × b)` where `b` is average body size. Realistic
estimate for tambear-scale (217 files): ~6s per scan. Borderline for CI; Sweep A5
should benchmark and may need parallelism or incremental scan caching.

### Sweep-level consequences

- **Sweep A4** extends fingerprint visitor for `#[descended_from]` walking; v2
  grammar is sibling extension of the same visitor.
- **Sweep A5** ships v2 + uses it for stdlib antigens needing body-level patterns.
- **antigen-stdlib v0.1+**: ships small `body_pattern` library
  (`sign-magnitude-distance`, `panic-in-drop-body`, `lock-after-await`).
- **Performance budget revision**: v1's "<5s for typical projects" raises to ~10s
  for v2; ATK-010-2 already noted v1 estimate was speculative.

### Enforcement

- Property tests for each v2 operator against synthetic ASTs.
- Adversarial sweep: malformed v2 fingerprints fail loudly.
- Tambear case study: `UlpDistanceRolledByHand` migrates to v2 grammar.

### Resolves

- The `UlpDistanceRolledByHand` adoption-log finding.
- The structural-blindness sibling of ATK-010-1.
- ADR-001's C5 drift-detection-at-scan-time invariant (without v2 grammar, scan
  has known structural blind spots).
- ADR-007 anti-YAGNI: recognizing failure-classes that recur structurally requires
  recognizing them in whatever syntactic form they recur.

### Open questions deferred

1. Body-pattern correctness validation (recursive recognition-discipline problem;
   future meta-witness ADR).
2. Cross-language body patterns (still Rust-only in v2).
3. Performance under workspace growth (>100k files).
4. Relationship to clippy's internal pattern DSL (per ADR-002 compose-don't-compete:
   yes if feasible; math-researcher systems-review for v2 should investigate).

---

## [ADR-013] ADR-002 Amendment 1: phantom-type witness recognition + witness-validity tier mapping

**Status**: Ratified 2026-05-08.

**Amends**: ADR-002 (Compose, don't compete).

**Participants**: pathmaker (drafted), aristotle (reciprocal Phase 1-8), scientist
(validation pass).

**Related**: ADR-001 [as amended] (C1: all named witness types ship; tier
acknowledgment Change 4), ADR-002 (the ADR being amended), ADR-005 (sub-clause F:
witness validity is the trust-boundary check), ADR-007 (anti-YAGNI: phantom-type
witnesses structurally guaranteed), ADR-010 (fingerprint grammar; phantom-type
witnesses interact with operator set), ADR-011 (tolerance-via-type-state alternative
noted in OQ3).

### Finding

ADR-002 lists witness mechanisms including phantom-type proofs ("for cases where a
compile-time witness is feasible"), but doesn't specify how cargo-antigen audit
recognizes them. The substrate currently has `WitnessKind = Test | Proptest |
Function` (audit.rs:65-76); phantom-type witnesses are not recognized at all.

ADR-007's anti-YAGNI commitment to "all four witness types" + ADR-002's enumeration
of five mechanisms produces a structural-completion requirement: ship recognition
of all named witness families, not 4-of-5.

A phantom-type witness expression is a typed path: `PolarityProof::<FrameTranslation>::established_by_construction`.
The existing `validate_witness` takes the LAST path segment, looks it up in the
function index, and classifies based on attributes. This is wrong for phantom-type
witnesses — the function index walk loses type-parameter context, and the audit
can't distinguish a real phantom-type witness whose construction encodes the proof
from a vacuous `fn () -> ()` reported as "Resolved."

This is a sub-clause F violation: the trust boundary at "audit reports witness as
well-formed" extends trust without checking the structural shape.

### Decision

**Extend `WitnessKind` with `PhantomType` variant. Refine `validate_witness` to
recognize the phantom-type pattern and classify with appropriate confidence.
Acknowledge witness-validity tiers in the audit's reporting surface.**

```rust
pub enum WitnessKind {
    Test,
    Proptest,
    Function,
    PhantomType {
        proof_type: String,         // e.g., "PolarityProof"
        type_params: Vec<String>,    // e.g., ["FrameTranslation"]
        constructor: String,         // e.g., "established_by_construction"
    },
}
```

`validate_witness` recognition order: external-tool delegation → phantom-type
detection → function-index lookup → `NotFound`.

### Mechanics

A phantom-type witness whose constructor exists is classified as `Resolved` with
`WitnessKind::PhantomType { ... }`. The audit reports phantom-type witnesses with
**higher confidence** than `Function`: the construction is compile-time-checked,
so if the code compiles, the proof holds.

**Recognize-and-warn for v0.1** (per OQ1 below): a phantom-type witness can be
constructed with deliberately-trivial bounds. Audit recognizes the *shape* but
cannot verify *meaning*. The audit emits a hint: "phantom-type witness — verify
the constructor encodes a real proof," not silent acceptance.

**Witness-validity tier mapping** (per ADR-001 Amendment 1 Change 4):

| Tier | Audit status mapping |
|---|---|
| Reachability | `Resolved (Function)` or `External` |
| Execution | `Resolved (Test \| Proptest)` (cargo test ran it) |
| Behavioral-alignment | `Resolved + AlignmentVerified` (deferred to ADR-005 OQ) |
| Formal-proof | `Resolved (PhantomType \| External=kani/prusti/verus/creusot)` |

JSON output includes `witness_tier` field for CI gates.

### Sweep-level consequences

- **Sweep A2 W7**: ships `WitnessKind::PhantomType` and basic detection. v0.1.0
  ships at recognize-and-warn level.
- **Sweep A3**: phantom-type witnesses imported from other crates (e.g.,
  antigen-stdlib's witness library) are recognized.
- **Sweep A4**: phantom-type witnesses survive `#[descended_from]` propagation
  when type parameters align.
- **Sweep A5**: at least one stdlib antigen ships with a phantom-type witness
  in `antigen-stdlib`.

### Enforcement

- Property tests verify `detect_phantom_type_witness` correctly identifies
  phantom-type paths.
- Trybuild fixture: phantom-type witness expression parses.
- Adversarial sweep: type-parameter mismatch
  (`#[immune(FrameTranslation, witness = PolarityProof::<BoundaryViolation>::built)]`)
  is flagged. Sub-clause F: witness type-parameter must align with antigen.

### Resolves

- ADR-007's anti-YAGNI commitment to "all four witness types."
- The audit's silent failure to distinguish phantom-type witnesses from vacuous
  functions.
- The witness-tier acknowledgment from ADR-001 Amendment 1 Change 4 (operationalized).
- The api-shape.md sketch's "advanced form" of witnesses becomes ratified.

### Open questions deferred

1. Trivial phantom-type construction validation (recognize-and-warn in v0.1; future
   ADR may add construction-validation, potentially via Flux delegation).
2. Phantom-type witness inheritance via `#[descended_from]` (Sweep A4 work).
3. "Tolerance via type-state" as alternative to ADR-011's attribute approach
   (keep ADR-011's attribute-based tolerance for v1; document type-state pattern
   as future-work).
4. Cross-language phantom-type analogs (Haskell GADTs, TypeScript branded types,
   Swift phantom-protocol witnesses; Rust-only in v1).

---

## [ADR-014] `#[antigen_generates(...)]`: declaring antigens that proc-macros emit

**Status**: Ratified 2026-05-08. Implementation deferred to Sweep A3-A4.

**Participants**: pathmaker (drafted from adversarial's ATK-010-1 finding),
aristotle (reciprocal Phase 1-8), scientist (validation pass).

**Related**: ADR-001 [as amended] (C6 carrier addition; this introduces a fifth
core macro), ADR-002 (compose, don't compete; consumer-side annotation question
in OQ1), ADR-005 (sub-clause F: scan's structural blindness for macro outputs is
a trust-boundary gap), ADR-007 (anti-YAGNI: macro-generated code is structurally
guaranteed-to-exist), ADR-010 (fingerprint grammar v1; generates interacts with
scan synthesis pass), ADR-011 (tolerance interaction at consumer call sites),
ADR-012 (sibling structural-blindness fix for function-body patterns).

### Finding

`cargo antigen scan` walks the source-level AST via `syn::parse_file`. It sees
the `#[derive(Foo)]` invocation but does NOT see the code that the `Foo` derive
macro generates. **Failure-classes that manifest in macro-generated code are
invisible to the scan.**

This is structurally guaranteed to bite real workspaces: derive macros are
ubiquitous in Rust (`Debug`, `Clone`, `Serialize`, `thiserror::Error`,
`tokio::main`, `async-trait`, custom domain derives). A scan blind to their
output misses a meaningful fraction of the failure-class surface.

The structural fix lives at the macro author's side: the macro author knows what
their macro generates. They can declare it.

### Decision

**Define `#[antigen_generates(antigen_type, ...)]` as a fifth core macro that
proc-macro and macro_rules authors apply to declare their macro emits code
presenting the named antigen.**

```rust
#[antigen_generates(
    PanickingInDrop,
    rationale = "This derive emits a Drop impl that may panic if the inner \
                 type's destructor panics; users should verify their inner \
                 types are panic-safe in Drop.",
)]
#[proc_macro_derive(SomeDerive)]
pub fn some_derive(input: TokenStream) -> TokenStream { ... }
```

**Required**: antigen type (positional), `rationale` (non-empty string, mirrors
ADR-011).
**Optional**: `witness_template` (path), `if_attr_present` (v2 conditional generation).

A macro can stack multiple `#[antigen_generates]` declarations.

### Mechanics

`cargo antigen scan` walks two passes:

1. **Source-level pass** (existing v1): collect `#[antigen]`/`#[presents]`/...
   /`#[antigen_generates]` declarations.
2. **Synthesis pass** (new v0.2+): for every macro invocation whose macro path
   resolves to one with `#[antigen_generates(X, ...)]`, emit a synthetic
   `Presentation { antigen_type: X, file: <invocation_file>, line:
   <invocation_line>, item_kind: "generated_<macro_name>" }`. Treated as
   `#[presents]` for matching.

**Macro path resolution**:
- Same workspace: scan walks the workspace and discovers `#[antigen_generates]`
  declarations directly.
- Cross-crate: requires cross-crate antigen-discovery (deferred to ADR-010 OQ1 /
  Sweep A3). v0.2.0 ships same-workspace; cross-crate awaits the discovery
  mechanism.

**Audit integration**: synthetic presentations are checked for an immunity
declaration on the same item (the macro INVOCATION, not the macro definition).
Per aristotle's reciprocal Phase 1-8, scan output surfaces consumer-side awareness:

```
src/lib.rs:42  PanickingInDrop on generated_SomeDerive (#[derive(SomeDerive)] expansion)
  note: this derive emits a Drop impl that presents PanickingInDrop;
        add #[immune(PanickingInDrop, witness = ...)] on the same item,
        OR mark with #[antigen_tolerance(PanickingInDrop, rationale = "...")].
```

**Absent declarations**: macros that don't declare are NOT silently exempt.
`cargo antigen audit --strict` may flag them as "unaudited macros" with a
separate category. The intent: absent declarations are a known unknown.

**Sub-clause F** (ADR-005): antigen type must be discoverable; rationale required;
expansion-validation deferred (v0.2+ trusts the author).

### Sweep-level consequences

- **Sweep A2** does NOT ship `#[antigen_generates]`. Deferred to A3-A4.
- **Sweep A3** wires the synthesis pass for same-workspace.
- **Sweep A4** extends to cross-crate via the antigen-discovery mechanism.
- **Sweep A5** populates antigen-stdlib with `#[antigen_generates]` on
  pattern-emitting macros (recursive use of antigen against itself).
- **Sweep A6** rust-analyzer surfaces synthetic presentations inline at the IDE.

### Enforcement

- Macro shipped as part of v0.2.0 release.
- CI gate: `cargo antigen audit --strict` reports unaddressed synthetic
  presentations.
- Trybuild fixtures: empty rationale rejected, missing antigen rejected, multiple
  generates accepted.

### Resolves

- Adversarial ATK-010-1 (macro-expansion blindness producing silent false-negatives).
- ADR-001's C6 (the carrier set is structural; new carrier requires ADR — this ADR).
- The structural-blindness pair with ADR-012 (function-body patterns).
- The third-party-derive blind spot in real Rust adoption.

### Open questions deferred

1. Third-party macros without `#[antigen_generates]`: consumer-side
   `#[antigen_generates_at(macro_path, antigen)]` annotation? (Future ADR; per
   ADR-002 composition discussion.)
2. Macro expansion validation (deeper structural check; deferred).
3. Conditional generation (`if_attr_present` v2 sketch; v1 unconditional).
4. Doc-comment surfacing in generated code (legibility vs pollution; deferred).

---

## [ADR-015] Fingerprint engine: grammar-over-AST with per-fingerprint evaluator trait

**Status**: Ratified 2026-05-09.

**Participants**: scout (engine-shift proposal; substrate evidence in
tambear `pattern.rs`), math-researcher (drafted v0 + v1 absorbing
self-deconstruction + substrate-correction addendum), aristotle (two
external Phase 1-8 cycles + revising addendum; recommendation:
ratify substantively as drafted with backend-choice deferred),
adversarial (ATK-015-1..7), scientist (validation), team-lead
(ratification).

**Related**:
- ADR-002 (compose, don't compete) — load-bearing; this ADR
  operationalizes ADR-002 at the body-pattern delegation surface.
- ADR-006 (recognition, not design) — drives substrate honesty.
- ADR-007 (anti-YAGNI / structurally-guaranteed need) — ADR-016
  dependency makes S3's evaluator trait structurally guaranteed;
  without ADR-016, S3 reduces to private scaffolding-until-second-backend.
- ADR-010 + Amendments 1-4 — partially superseded; see Supersedes.
- ADR-011 (`#[antigen_tolerance]`) — tolerance is the autoimmunity
  relief valve under filter-vs-proof framing.
- ADR-012 (function-body operators) — body-pattern operator family
  that delegates per ADR-015.
- ADR-013 (phantom-witness operators) — separate concern (witness
  side); coordinates via the same delegation-boundary discipline.
- ADR-016 (temporal recognition surface) — sibling ratification;
  structural guarantee for the evaluator trait going public.

**Supersedes (partial)**: **ADR-010 §Mechanics §1 only** ("syn::parse2
parses the fingerprint string into an internal AST"). The visitor
pattern, `MatchSite` reporting, performance characterization, and all
other §Mechanics content stand under this engine choice. ADR-010
Amendment 1 (Path C parsing) is the actual replacement for
§Mechanics §1.

### Finding

ADR-010 ratified a fingerprint grammar with `syn::parse2` +
`syn::visit::Visit` as the implementation engine. Three pieces of
substrate evidence surfaced after ratification that warrant amendment
to that single mechanics commitment:

1. **The body-pattern problem is real** (ADR-012, deferred from
   ADR-010 v1; tambear adoption-log entry 2026-05-07: simple
   syntactic patterns missed the multi-statement form of
   `UlpDistanceRolledByHand`). Body-level matching for arbitrary
   structural patterns is structurally guaranteed by the v0.1 stdlib
   commitments per ADR-007.

2. **The grammar-vs-vocabulary cut** (per aristotle's reciprocal
   Phase 1-8 of math-researcher's ADR-010 systems review): the
   *grammar* (node-kind × field-path × constraint-op + Boolean
   composition) is the load-bearing structural commitment; the
   *vocabulary* (named operators) is the projection surface. ADR-010
   ratified the vocabulary explicitly; the grammar was implicit.
   ADR-015 surfaces it.

3. **Engine choice and grammar shape are separable**. The grammar
   (predicates over Rust AST) can be evaluated by syn-visitor,
   ast-grep matcher, or future backends. The grammar's shape is
   independent of which engine runs it. ADR-010 conflated them;
   ADR-015 separates them.

**Substrate honesty**: tambear's `pattern.rs` is exploratory-not-
committed (per team-lead clarification). ast-grep itself is at
v0.42.1 (April 2026), pre-1.0; ast-grep-core docs explicitly state
"the Rust API is not stable yet" with the CLI as the recommended
primary path. Recognition-not-design (ADR-006) requires substrate
that has happened *and is committed*; tambear clears the first bar
but not the second. ast-grep-core clears existence but not stability.

This honesty constrains what ADR-015 can ratify. The
grammar-vs-vocabulary cut, the engine separability, and the
delegation-boundary discipline all stand on substrate-stable
evidence. The specific delegation backend (ast-grep-core library vs
ast-grep CLI subprocess vs deferred) does not — the substrate doesn't
yet justify ratifying a single backend.

### Decision

**ADR-015 ratifies four structural commitments and explicitly defers
the body-pattern delegation backend choice.**

#### Ratified structural commitments

**S1 — Grammar-vs-vocabulary cut**: the antigen fingerprint engine is
a *grammar over Rust AST predicates*. Operators in the grammar may
evaluate via different runtimes; the grammar is one. The vocabulary
(named operators users write) projects the grammar.

**S2 — Per-operator implementation principle**: typed-AST queries
(item-kind dispatch, name predicates, attribute presence,
field/variant shape, method-signature shape) evaluate against `syn`'s
typed AST directly. Metavariable / structural patterns at body level
delegate to a separate runtime via the evaluator interface (S3). The
grammar is one; the runtime is per-operator-decided.

This explicitly resolves: `body_contains_macro(name)` (Tier-1
vocabulary, per ADR-010 Amendment 3 Clause C) is a `syn::Block`
walker for `syn::Macro` invocations matching the name — implemented
natively in syn, **NOT** through a delegated body-pattern engine. The
native walker is faster for the common case of macro-name matching.
`body_pattern("<arbitrary structural pattern>")` (deferred to sibling
backend decision; see Deferred) is the general escape hatch for
arbitrary structural patterns at body level, when ratified.

**S3 — Per-fingerprint evaluator trait (private in v0.1)**:

```rust
trait Evaluator {
    fn evaluate(&self, fingerprint: &Fingerprint, file: &syn::File) -> Vec<MatchSite>;
}
```

Per-fingerprint granularity (not per-operator). The trait stays
**private** in v0.1 and goes public when a second evaluator backend
ratifies (or when ADR-016's temporal evaluator lands, whichever comes
first). The trait abstraction at the delegation boundary is what
makes "defer the backend choice" operationally clean: a sibling
decision on body-pattern backend can later add an `AstGrepEvaluator`
or `SubprocessEvaluator` that implements the trait without invasive
refactor.

**Coordination note**: S3's structurally-guaranteed-need argument
depends on ADR-016 ratifying its temporal-axis structural commitments
(the temporal evaluator becomes the second backend that justifies
S3's existence). If ADR-016 fails or is deferred to v0.2+, S3 reduces
to YAGNI under anti-YAGNI scrutiny — the private-until-second-backend
fallback is then primary mechanism.

**S4 — Supersession scope: ADR-010 §Mechanics §1 only**. The visitor
pattern, `MatchSite` reporting, performance characterization,
depth/node-count caps, per-file caching pattern, and witness adapter
independence (ADR-002 separation) all stand. Only the
implementation-detail "syn::parse2 parses the fingerprint string into
an internal AST" shifts. The Path C parsing decision (ADR-010
Amendment 1) is the actual replacement for §Mechanics §1.

The fingerprint AST is `#[derive(Serialize, Deserialize)]` so future
cross-language ADRs can emit alternative serialized forms from the
AST without parser rewrites.

#### Deferred — body-pattern delegation backend

The choice of *which* runtime evaluates body-pattern operators is
**deferred** to a sibling decision under W6b implementation pressure.
Three honest paths exist:

- **Path 1 — Library-level delegation** to ast-grep-core. Requires
  hardened version-pinning (explicit `rust-version` in
  antigen-fingerprint's Cargo.toml + ast-grep range as `>=X, <Y`; CI
  checks minor-version compat; major triggers amendment). Pre-1.0
  API instability is a load-bearing operational concern.

- **Path 2 — Subprocess-level delegation** to ast-grep CLI (binary).
  Drops the library dependency; invokes `ast-grep` as a subprocess
  and parses JSON output. Version-pinning concern transforms to
  **PATH/binary verification at audit time** (the cargo-deny /
  cargo-audit pattern). Restores consistency with the
  composition-surface taxonomy (clippy/kani/prusti/creusot/verus
  all compose at output-level for stability reasons).

- **Path 3 — Defer body-pattern entirely to v0.2+**. Ship v0.1 with
  item-shape-only fingerprints; document the body-pattern
  limitation honestly per ADR-010's already-stated honest-known-
  limitations posture.

**Both math-researcher (substrate-correction addendum) and aristotle
(revising addendum) independently recommend Path 2** when the backend
ratifies. Independent convergence on Path 2 from different
chains-of-reasoning is the recognition signal at the strongest tier.
The sibling decision should consider this as substrate, not as a
pre-commitment from ADR-015's body. **Pathmaker decides under W6b
implementation pressure.**

When the sibling decision lands:
- The S3 evaluator trait goes public, with the chosen backend
  implementing it.
- Version-pinning materializes per-path: Path 1 → MSRV+range
  discipline; Path 2 → binary verification; Path 3 → moot.
- ADR-012's deferred body-level operators unblock for W6b
  implementation.

### Mechanics

#### Implementation surface

`antigen-fingerprint` (workspace member):

1. **Parser** — hand-written `Parse` impl over
   `syn::parse::ParseBuffer`, per Path C. Tokenizes via `syn`'s
   tokenizer; parses the comma-separated key=value vocabulary into a
   `Fingerprint` AST.

2. **Fingerprint AST** — serializable Rust enum. Node kinds,
   constraints, composition. `#[derive(Serialize, Deserialize)]`.

3. **Evaluator trait (private)** — per-fingerprint granularity (S3);
   private API in v0.1; goes public when a second backend ratifies.

4. **Built-in syn evaluator** — implements the evaluator trait
   against `syn::visit::Visit`. Handles all Tier-1 typed-AST queries
   (item-kind dispatch, name predicates, attribute presence,
   field/variant shape, method-signature shape, `body_contains_macro`
   via native syn::Block walker).

5. **Body-pattern operator** — `body_pattern("<arbitrary structural
   pattern>")`. NOT shipped in v0.1 unless the sibling backend
   decision lands first. When shipped: the chosen backend implements
   the evaluator trait for body-pattern operators; the syn evaluator
   delegates body-pattern operators to that backend.

#### Span granularity

Three distinct span concerns disambiguated:

- **Scan reports**: file:line:col is mechanical and ships in v0.1.
- **Proc-macro `Span`** for fingerprint-parse compile errors: uses
  antigen-fingerprint's own Path C parser spans (not external
  substrate spans). Sibling work to W4.
- **Body-pattern report-back** (when a body-pattern backend
  ratifies): the backend's match coordinates round-trip to syn for
  span reporting; v0.1 fallback is file:line:col.

#### Crate dependencies

`antigen-fingerprint` adds (to its `Cargo.toml`):
- `syn` — already in scan.rs's deps; tokenizer + AST.
- `proc-macro2` — already transitive.
- `serde` — already in deps; for serializable AST invariant.
- **No body-pattern backend dependency in v0.1**. Added (under
  chosen Path) when sibling decision ratifies.

#### Performance invariants

Per math-researcher's ADR-010 review §3.2 + §4 + §15.Q2 + W6 advance
note, all unchanged from ADR-010 Amendment 3:

- Single-pass walking with `node_kind` dispatch (don't run N
  separate passes for N fingerprints).
- Pattern signatures compile *once* per fingerprint set per scan
  invocation, not per file or per item. Cost asymmetry is ~50×
  without this discipline.
- Per-file `target/antigen/` cache keyed on `(path, mtime,
  content_hash, fingerprint_set_hash)` (forward-compat path; not
  v0.1 surface).
- Depth + node-count caps at parse time (depth ≤ 10, total nodes ≤
  256) per ADR-010 OQ4.

#### Witness adapter independence

Witness adapters (per ADR-002) are unaffected by this engine choice.
Witnesses run via their respective tools (`cargo test`, `cargo kani`,
`cargo clippy`, `cargo prusti`, etc.) and report results. The engine
choice is *what fingerprints match*, not *what witnesses prove*.

### Sweep-level consequences

- **Sweep A2 (core macros + initial fingerprint engine)**: scope is
  item-level operators only. Body-pattern operator (W6b) blocks on
  sibling backend decision. The evaluator trait ships as private
  scaffolding in v0.1.
- **Sweep A3 (cargo-antigen scan)**: per-file caching pattern is
  ratified; implementation defers per ADR-010 stewardship.
- **Sweep A4 (composition rules)**: ADR-016's temporal evaluator
  (when ratified) is the second backend that justifies S3's public
  API; trait goes public during A4 if not earlier.
- **Sweep A5 (vaccinate + audit + stdlib antigens)**: stdlib
  antigens authored against this grammar. Tier-2/Tier-3 vocabulary
  additions sized when stdlib content surfaces the need.

### Enforcement

- **CI gate**: `cargo build` succeeds with `antigen-fingerprint`
  crate building against `syn` only (v0.1). `cargo test --workspace`
  covers parser unit tests + syn-evaluator integration tests
  against synthetic AST fixtures.
- **Adversarial sweep** (per ADR-005 sub-clause F): malformed
  fingerprint strings fail loudly at fingerprint-load time, not
  silently at scan time. `Path C` parser produces `syn::Error`
  pointing to the offending token's span.
- **Recognition substrate**: when the sibling backend decision
  ratifies, tambear's `UlpDistanceRolledByHand` adoption gap
  (multi-statement form) becomes the integration test target. v0.1
  ships honest documentation that body-pattern matching is
  deferred; the gap is named, not papered over.

### Resolves

- The implementation-detail tension in ADR-010 §Mechanics §1 that
  surfaced under math-researcher's review §16.6 + tambear
  adoption-log evidence.
- The B-vs-C question scout asked: this ADR makes the answer
  "neither B nor C as initially framed; it's *one grammar with
  separable runtime*." The runtime choice (library / subprocess /
  deferred) is sibling decision territory.
- Aristotle's grammar-vs-vocabulary cut: the grammar is what gets
  ratified (this ADR pins it); the vocabulary is what gets
  documented and grows with stdlib content.

### Open questions deferred to future ADRs

1. **Body-pattern backend choice (sibling decision)**. Path 1 / Path
   2 / Path 3 per Deferred section. Both reviewers independently
   recommend Path 2; the sibling decision should weigh that
   convergence against any new substrate.

2. **Path-relative predicates**: fingerprints expressing
   path-relative predicates ("an enum X *inside* a module named Y")
   aren't in v0.1 grammar. Future Tier-2/Tier-3 vocabulary
   extension if substrate surfaces the need.

3. **Cross-language readiness**. The serializable AST makes
   language-portability tractable; the actual cross-language ADR
   sequences in v0.2+ when stdlib content surfaces a non-Rust
   failure-class.

4. **Per-operator trait granularity** (deferred from S3).
   Per-fingerprint is the v0.1 grain. If cross-language work
   surfaces evidence that per-operator dispatch is structurally
   needed, future amendment to S3.

---

## [ADR-016] Temporal recognition surface: provenance + freshness primitives for stale-context and premature-abstraction

**Status**: Ratified 2026-05-09.

**Participants**: scout (T2 finding — stale-context and
premature-abstraction need temporal primitives), math-researcher
(drafted v0 + v1 absorbing aristotle's external Phase 1-8),
aristotle (external Phase 1-8: ratify with grid framing +
substrate-honesty refinement + R-5 PrematureAbstraction witness gap
closure), adversarial (ATK-T2-1..4 + ATK-016-1..5 + ATK-A2-1..6),
scientist (validation), team-lead (ratification).

**Sibling**: ADR-015 (engine grammar). Together they ratify the
analysis-level × temporality grid as recognition substrate. ADR-015
picks engines (the analysis-level axis); ADR-016 picks the temporal
level + freshness substrate (the temporality axis).

**Related**:
- ADR-001 + Amendment 1 — **orthogonal axes**, not extension.
  Carrier-strength is about drift-resistance of the memory carrier;
  temporality is about the memory's relationship to time.
- ADR-002 (compose-don't-compete) — load-bearing for v1
  implementation: cargo-audit / cargo-deny / Renovate are external
  temporal witnesses.
- ADR-005 sub-clause F — every trust boundary needs validation. The
  temporal trust boundary (was-verified-at-commit-X) is no
  exception.
- ADR-006 (recognition, not design) — the substrate exhibits
  temporal failure-classes; this ADR names them.
- ADR-007 (anti-YAGNI: structurally-guaranteed need) — the 8-class
  taxonomy commits us to all 8; classes 4 (stale-context) and 5
  (premature-abstraction) are temporal.
- ADR-009 (adoption gradient) — `references` field is the passive
  temporal carrier today; ADR-016 adds an active carrier.
- ADR-010 + Amendments — temporal level joins syn / HIR / MIR /
  runtime in the analysis-level field.
- ADR-013 + Amendments — witness-validity tier hierarchy.
  PrematureAbstraction's witness shape sits *below* Reachability
  tier; future-ADR placeholder for a possible Documentation tier.
- ADR-015 — engine-axis extensibility (the per-fingerprint
  evaluator trait private-in-v0.1) admits a temporal-evaluator
  backend when one ratifies.

### Finding

The 8-class failure taxonomy is **not analysis-level-uniform** and
**not temporality-uniform**. Two of the eight classes (stale-context
#4 and premature-abstraction #5) are *fundamentally temporal*: they
involve comparisons between *current state* and *past state* (or
past evidence). The substrate evidence:

- **Stale-context** instances (per
  `failure-class-instances.md` §4; 4 instances):
  `cratedepression-rustdecimal-typosquat`,
  `faster-log-async-println-supply-chain`,
  `rust-1.80-time-crate-stale-pin`,
  `openssl-sys-vendored-stale-build-cache`. Each instance has the
  shape: state X was true at time T; state X is no longer true at
  T'; consumer is using state X confidently.

- **Premature-abstraction** instances (per
  `failure-class-instances.md` §5; 5 instances):
  `mem-uninitialized`, `unsafe_destructor_blind_to_params`,
  `TrustedLen`, `pin!`, GATs. Each has the shape: abstraction A was
  made against evidence E at time T; evidence E' became available
  at T'; A is now load-bearing but doesn't fit E'.

ADR-010 (fingerprint grammar at scan time) and ADR-002 (witness
composition operating on current code/test/proof state) are
**temporally flat** — they see the codebase's *current state* only.
They cannot, in principle, recognize stale-context or
premature-abstraction without access to a temporal substrate.

ADR-007's anti-YAGNI clause commits us to all 8 classes. Aristotle's
P1-8 phase 8 surfaced the deeper structure: antigen v1 occupies one
cell of a 2D grid (analysis-level × temporality) and gestures at the
others via taxonomy + manually-presented antigens. This ADR ratifies
the grid as a recognition substrate.

### Decision

**Antigen ratifies temporality as a first-class axis of recognition,
orthogonal to analysis-level. Antigen declarations may carry a
`verified_at = "<commit-hash>"` field; `cargo antigen audit` walks
git history to check freshness; `cargo audit` / `cargo-deny` /
`Renovate` are accepted as external temporal witnesses via ADR-002
composition. Stdlib v0.1 ships at least one temporal antigen
(`OutdatedSecurityAdvisory` with cargo-audit witness).**

**Stdlib v0.1 commitments — sequencing**:
- **A2** ratifies the substrate (fields + grid).
- **A4** implements audit-checks.
- **A4-A5** stdlib populates.

**PrematureAbstraction's witness shape gap**: PrematureAbstraction
in v0.1 is **taxonomy-entry + Layer-2 references-only carrier** (per
ADR-009). Sites marked manually use `#[presents(PrematureAbstraction,
references = [...])]`. **Sites are NOT marked `#[immune(...)]`**
because the witness shape ("manual evidence review") is *below* the
Reachability tier of ADR-013's witness-validity hierarchy — it's not
a callable artifact. To explicitly tolerate a site that matches
PrematureAbstraction's shape, use `#[antigen_tolerance(...)]` per
ADR-011.

**Substrate-honesty refinement**: the A2 macro-parser must accept
the three new fields (`verified_at`, `evidence`, `stale_after`)
**with known-limitation note** ("verified_at field present but
audit-check landing in v0.2+"). NOT silent acceptance (sub-clause F
violation per ATK-A2-1); NOT field rejection (forward-compat block);
explicit accept-and-note. The pattern matches ADR-001 Amendment 1
Change 4's witness-tier-deferral discipline.

The two-axis grid:

```
                analysis level →
              syn   HIR   MIR   runtime
snapshot      X     .     .     .          (ADR-010 + ADR-015 cell)
longitudinal  .     .     .     X2         (ADR-016 cell)
```

**v0.1 cell coverage**:
- **(syn, snapshot)** — the ADR-010 + ADR-015 cell. Most v1 stdlib
  antigens. PrematureAbstraction lives here in v0.1 as taxonomy-only
  entry (Layer-2 references-only carrier; no `#[immune]` because
  witness shape sits below ADR-013 Reachability tier).
- **(runtime, longitudinal)** — X2 — stale-context via cargo-audit,
  cargo-deny, Renovate as witnesses. v0.1 stdlib commits at least
  one antigen here (`OutdatedSecurityAdvisory` +
  `StaleDependencyPin`).
- **(syn, longitudinal)** — *future cell, not v0.1*. Will carry
  premature-abstraction's full provenance substrate when it
  ratifies in v0.2+.
- **(HIR, snapshot)**, **(MIR, snapshot)**, **(HIR, longitudinal)**
  — deferred to v0.2+ via external-analyzer composition.

The grid is the recognition substrate; v0.1 populates two cells;
future ADRs populate others as substrate evidence accumulates.

### Mechanics

#### Field additions

`#[antigen(...)]`, `#[immune(...)]`, `#[presents(...)]` accept three
new optional fields (per substrate-honesty discipline: A2 accepts
these with known-limitation note; A4 implements the audit-side
checks):

- **`verified_at = "<commit-hash>"`** — the commit hash at which
  this declaration was last verified. Audit walks `git log` to
  determine whether HEAD is reachable from this commit and how far.
  Default: absent (no temporal claim).

- **`stale_after = <interval>`** — declaration becomes "stale" if
  HEAD is more than this many commits / days past `verified_at`.
  Default: absent (no decay; staleness only flagged on explicit
  external-tool failure). Interval syntax: `commits(N)` (commit-
  count distance), `days(N)` (calendar days), `version("X.Y.Z")`
  (semver-major-or-minor change in a watched dependency).

- **`evidence = ["<URL-or-commit-or-RFC>"]`** — the evidence that
  supports this declaration's claim. Audit checks evidence is still
  accessible. Decay is reported as warnings, not failures (link rot
  is normal).

#### Audit behavior

`cargo antigen audit` extends with three checks (A4 implementation):

1. **Freshness check** — for declarations carrying `verified_at`,
   walk `git log --first-parent` and compute commit distance from
   HEAD. **Calendar time is sourced at audit invocation** (system
   clock); audit explicitly does NOT use frozen-time substrates. Report:
   - Reachable + within threshold → green.
   - Reachable + over threshold → warning (not error; v0.1).
   - Unreachable → error.
   - Git unavailable (CI shallow clone) → graceful skip with note.

2. **Evidence check** — for declarations carrying `evidence`,
   attempt to verify each entry:
   - URL → HTTP HEAD (off by default to keep audit hermetic; opt-in
     via `cargo antigen audit --check-evidence`).
   - Commit → `git rev-parse` (always; no network).
   - Other (RFC, ADR ref) → format-validation only.

3. **External temporal-witness check** — for antigens declared with
   `temporal_witness = "cargo-audit" | "cargo-deny" | ...`, audit
   invokes the external tool and threads its output. Same composition
   pattern as ADR-002 for syn-snapshot witnesses, applied to the
   runtime-longitudinal cell.

#### Stdlib v0.1 commitments

- **`OutdatedSecurityAdvisory`** (runtime-longitudinal) —
  vulnerability declared in a workspace's lockfile is patched in a
  newer version. Witness: `cargo audit`. Audit fails when cargo-audit
  reports unpatched advisory.
- **`StaleDependencyPin`** (runtime-longitudinal) — declared minimum
  supported dependency version is N major versions behind crates.io
  latest. Witness: cargo-deny `bans` rules with version range.
- **`PrematureAbstraction`** (syn-snapshot, taxonomy entry only in
  v0.1) — declared via `#[presents(PrematureAbstraction, references
  = ["<RFC URL>", "<commit hash>"])]`. v0.1 has no automated
  detection; consumer marks manually. v0.2+ adds active provenance
  substrate. NOT marked `#[immune(...)]` because witness shape sits
  below Reachability tier.

#### Engine integration (sibling ADR-015 connection)

Per ADR-015's engine-axis extensibility (the per-fingerprint
evaluator trait private-in-v0.1, going public when a second backend
ratifies), the temporal evaluator is a backend. Operators in the
temporal cell:

- **`verified_at(<commit>)`** — predicate evaluated at audit time,
  not scan time.
- **`evidence_present(<URL>)`** — predicate evaluated on
  evidence-check pass.
- **`stale_relative_to(<interval>)`** — predicate over `git log` /
  dependency-graph state.

When ADR-016's temporal evaluator implementation lands (A4), it
becomes the second evaluator backend that justifies ADR-015 §S3's
evaluator trait going public. The two ADRs co-validate: ADR-015's S3
trait is structurally guaranteed by ADR-016's temporal evaluator;
ADR-016's temporal evaluator is operationally clean because ADR-015
§S3 abstracts the delegation boundary.

#### Crate dependencies

ADR-016 ratifies the *need* for git-graph walking. **A4
implementation chooses among**:
- `git2` — libgit2 binding for commit-graph walking.
- `gitoxide` — pure-Rust git library; rapidly maturing as of 2026.
- `git` subprocess invocation — composition-surface taxonomy
  consistency with cargo-audit/cargo-deny path; no library dep.

ADR-016 does not pin the git substrate at v0; A4 implementation pass
picks based on substrate evolution at the time. Graceful degradation
when git substrate is unavailable.

`semver` — version parsing for `stale_after = version("X.Y.Z")`.
Mature stable.

External witness invocation reuses the cargo subcommand machinery
already in `cargo-antigen`. cargo-audit / cargo-deny are not added
as dependencies; they are invoked as external processes (per
ADR-002).

### Sweep-level consequences

- **Sweep A2 (core macros)**: the macro arg-parser must accept the
  three new fields **with known-limitation note** per
  substrate-honesty refinement. Audit doesn't yet *act* on the
  fields in v0.1.0 (ADR-016 implementation is mostly Sweep A4
  territory).
- **Sweep A3 (cross-crate scan + descended_from)**: the
  `verified_at` field interacts with `#[descended_from]`: the
  inheritor must re-verify (filed as Eiffel rule D4 — substantive
  claims do not propagate without explicit re-statement).
- **Sweep A4 (composition rules)**: ADR-016's full implementation
  lands here. Audit gains `verified_at` walking; stdlib gets
  `OutdatedSecurityAdvisory` + `StaleDependencyPin`; cargo-audit /
  cargo-deny witness adapters land. A4 also surfaces ADR-015 §S3's
  evaluator trait going public if ADR-015's R6.4 fallback was
  active.
- **Sweep A5 (vaccinate + audit completeness)**: stdlib temporal
  vocabulary is mature enough to author complex stdlib temporal
  antigens. Premature-abstraction's full provenance substrate is
  the v0.2+ ADR.

### Enforcement

- **CI gate**: `cargo build` succeeds with `verified_at = "..."`
  declarations. `cargo antigen audit` reports staleness in
  machine-readable JSON.
- **Adversarial sweep** (ADR-005 sub-clause F): a `verified_at`
  field that the audit doesn't actually walk is *decoration*, not
  memory. ADR-005 forbids decorative trust extensions.
- **Recognition substrate**: tambear's CI workflow (after antigen
  v0.2+ ships) is the integration test target.

### Resolves

- The temporal flatness of ADR-010 + ADR-002.
- The 2-of-8 stdlib coverage gap in v1 (without ADR-016, classes 4
  and 5 have no v0.1 carriers).
- The implicit conflation of recognition surface with snapshot
  recognition (ADR-006 violation: substrate exhibits temporal
  recognition; v1 didn't name it).
- The asymmetry between `references` (Layer 2 ADR-009; passive
  carrier) and what stale-context actually needs (active carrier
  with audit-time check).
- The witness-shape gap for PrematureAbstraction: below-Reachability
  tier is now explicitly named; tolerance path via ADR-011; future
  Documentation tier as ADR placeholder.
- The accept-vs-reject vs accept-with-note dilemma at A2
  macro-parser boundary: substrate-honesty refinement closes
  ATK-A2-1.

### Open questions deferred to future ADRs

1. **Premature-abstraction full provenance** (v0.2+ ADR). Likely
   shape: `verified_at_evidence = [{commit, time, justification}]`.
   Requires git-graph integration, evidence-storage shape,
   abstraction-time recording.
2. **Documentation witness tier**. If multiple instances surface
   where "manual evidence review" or "documentation cross-reference"
   is structurally a witness, a Documentation tier *below*
   Reachability becomes structurally guaranteed by the substrate.
3. **Reverse-antigen primitive (`#[contingent_on(X)]`)**.
   Stale-context is structurally a *reverse antigen* — the antigen
   is "this code is correct WHEN X is current." May need its own
   primitive.
4. **Decay-rate per failure-class**. cargo-audit's advisory DB has
   daily decay; rust-1.80-time has yearly decay; mem-uninitialized
   had decade-scale decay. v0.1 ships single-default decay.
5. **Cross-version semver awareness as fingerprint operator**. v0.1
   ships interval syntax; the actual semver-watch logic may need
   its own ADR if it grows beyond cargo-deny composition.
6. **Inheritance interaction with verified_at**. v1 position:
   inheritors don't inherit `verified_at`. Eiffel rule D4.
7. **Audit hermeticity disclosure**. The `--check-evidence` flag is
   opt-in for network calls; system clock for calendar time is
   canonically non-hermetic. The exact disclosure-shape may need
   its own future ADR if audit grows additional non-hermetic checks.
8. **Git substrate choice** (deferred to A4 implementation).
   git2 / gitoxide / `git` subprocess.

---

## [ADR-017] Antigen identity is canonical declaration site; cross-crate trust delegates to cargo

**Status**: Ratified 2026-05-09.

**Participants**: aristotle (Phase 1-8 + draft), navigator (scope-lock,
substrate-currency, routing, divergence-as-signal sharpening, v5 stale-example
fix + precondition clarification), pathmaker (substrate verification), scout
(cross-crate discovery substrate; P1/P2/P5 empirical checks; canonical_path
semver format; precondition enforcement clarification), naturalist
(biology-layering rationale; earned-identity framing), adversarial
(orphaned_lineage_edges enforcement gap), team-lead + Tekgy (Approach
3-revised ratification).

**Related**:
- ADR-001 Amendment 1 Change 3 C7 (cross-crate consumption commitment)
- ADR-005 (sub-clause F at every trust boundary; cross-crate antigen consumption is named boundary item 4)
- ADR-005 Amendment 2 (rationale-as-required-field; the trust-extension justification carries via cargo's own checksum chain)
- ADR-006 (recognition-not-design; canonical-path is recognition of structure already partially present in `ItemTarget::Impl::target_type`)
- ADR-007 (anti-YAGNI structurally-guaranteed; cross-crate identity is forced by C7 + T7 + T13 from Phase 1-8)
- ADR-009 (adoption gradient; canonical_path stays Option to preserve Layer 1 minimum-viable path)
- ADR-010 Amendment 4 (filter/proof framing; canonical_path keys identity, fingerprints filter — orthogonal axes)
- ADR-018 (propagation semantics; sibling ADR depending on identity primitives this ADR ratifies)

**Implicit pattern elevated** (per ADR-004 Enforcement):

The current substrate keys antigen identity via bare type-name strings
(`AntigenDeclaration::type_name: String`, `LineageEdge::parent: String`,
`Presentation::antigen_type: String`, `Immunity::antigen_type: String`,
`Toleration::antigen_type: String`). For intra-workspace scanning this is
correct — antigen type-names are unique within a workspace by convention
(enforced by the compiler). For cross-crate scanning (ADR-001 C7, deferred
to A3), bare type-names are structurally ambiguous: `crate_a::PanickingInDrop`
and `crate_b::PanickingInDrop` are different antigens with the same type-name.
The implicit convention "type-name uniqueness = antigen identity" is a
workspace-scope assumption that fails at the cross-crate boundary. This ADR
elevates antigen identity from type-name-string to canonical-declaration-site.

### Finding

Cross-crate antigen consumption (ADR-001 C7, Sweep A3 scope) requires
antigen *identity* beyond the bare type-name. Two antigens in different
crates can share a type-name; immunity claims and lineage edges must
distinguish them.

Three identity-model approaches exist:

**Approach 1** (bare type-name): current substrate. Works intra-workspace;
fails at cross-crate boundary as described above.

**Approach 2** (user-declared qualified paths: `crate_a::PanickingInDrop`
in source code): requires users to write qualified paths in
`#[presents]`/`#[immune]` attributes. Conflicts with ADR-001 C4 (declarations
live in source files without side-cars) — the qualification becomes a
side-car embedded in the attribute.

**Approach 3-revised** (scanner-derived canonical_path field): the user never
writes the canonical_path. The scanner derives it from cargo metadata at
scan-time and stamps it on discovered declarations. User-facing macros are
unchanged. The identity is a derived property of the discovery mechanism,
not a user-declared property.

Phase 1-8 (aristotle) deconstructed all approaches and eight additional
alternatives on a gradient from user-burden to scanner-burden. Approach
3-revised wins on ADR-002 (compose, don't compete — delegates to cargo),
ADR-005 (sub-clause F — validates cross-crate trust via cargo's resolution
chain rather than re-implementing it), and ADR-006 (recognition, not design —
recognizes structure that cargo metadata already provides).

### Decision

**Antigen identity is the canonical declaration site: `(type_name,
canonical_path)` tuple where `canonical_path` is scanner-derived at
cross-crate scan time.**

A new field is added to five carrier types:

| Type | Field added | Semantics |
|---|---|---|
| `AntigenDeclaration` | `canonical_path: Option<String>` | crate name + version where this antigen was originally declared (e.g., `"crate_a@1.2.3"`); `None` for intra-workspace |
| `Presentation` | `canonical_path: Option<String>` | crate where the *antigen* originated, not where the presentation is — see Mechanics |
| `Immunity` | `canonical_path: Option<String>` | crate where the antigen originated |
| `Toleration` | `canonical_path: Option<String>` | crate where the antigen originated |
| `LineageEdge` | `parent_canonical_path: Option<String>` + `child_canonical_path: Option<String>` | crates of parent and child antigens (cross-crate lineage edges become first-class) |

All fields use `#[serde(default)]` for backward-compatible deserialization
of pre-A3 reports.

### Mechanics

#### canonical_path is set by the scanner, not by the user

The user never types `canonical_path` in source code. The field is set by
the cargo-metadata-driven scanner when the scan is cross-crate:

- Workspace-internal scan (`cargo antigen scan`): all `canonical_path` fields are `None`.
- Cross-crate scan (`cargo antigen scan --include-deps`): the scanner runs
  `cargo metadata`, identifies dependency crates, walks each to its
  `.cargo/registry/src/<index>/<crate>-<version>/`, and runs the same scanner
  with `canonical_path` set to the crate identity for each declaration
  discovered there.

The user's source code stays unchanged. The path information is derived,
not declared. This honors ADR-001 C4.

#### How the path is computed

**The canonical_path format is `"<crate-name>@<version>"`** — e.g.,
`"foo@1.2.3"`, `"serde@1.0.193"`. The `<version>` portion is the dependency's
exact resolved version string from `cargo metadata`'s `packages[].version`
field (semver-formatted; major.minor.patch with optional pre-release/build
metadata as cargo reports it).

**Why include the version** (scout's P5 nuance, empirically verified):
A workspace can depend on `foo v1.0` and `foo v2.0` simultaneously. With
crate-name-only canonical_path, the two versions' antigens become
observationally identical — sub-clause F violation. The `@<version>` suffix
makes the distinction explicit. Scout empirically verified: antigen's own
dep graph already has 4 crate names at multiple versions (`getrandom`,
`hashbrown`, `r-efi`, `wit-bindgen`). `name@version` is minimum-viable, not
future-proofing.

#### The `addresses()` semantics

`unaddressed_presentations()` is amended: the `i.file == p.file` guard is
replaced with a combined identity check:

```rust
let same_antigen_identity =
    i.antigen_type == p.antigen_type &&
    i.canonical_path == p.canonical_path;
let same_item = i.item_target.addresses(&p.item_target);
let same_locus =
    i.canonical_path.is_some() == p.canonical_path.is_some()
    && (i.canonical_path.is_none() && i.file == p.file
        || i.canonical_path.is_some() && i.canonical_path == p.canonical_path);
```

**Note on locus semantics**: a `Presentation` discovered in
`.cargo/registry/src/<crate-α>-1.2.3/` carries `canonical_path =
Some("crate-α@1.2.3")`. An `Immunity` declared in the consumer workspace that
references `crate_α::PanickingInDrop` carries `canonical_path =
Some("crate-α@1.2.3")` (the antigen's origin + version, not the consumer's
location). The two address each other because they reference the same
cross-crate identity including version.

#### Version-boundary semantics: re-attestation as a feature, not a limitation

With `"name@version"` as the identity, version upgrade is itself a
trust-boundary event. Sub-clause F applied across time: a child antigen
declaring `#[descended_from(ParentAntigen)]` against `foo@1.0.0::ParentAntigen`
made its claim at a specific declaration at a specific fingerprint at a
specific version. When `foo` upgrades to `1.1.0`, the parent declaration
may have changed.

**The orphan mechanism IS the re-verification prompt**:
- Lineage edges pointing at `foo@1.0.0::ParentAntigen` become orphans when
  the workspace upgrades to `foo@1.1.0` (`orphaned_lineage_edges()` surfaces
  the edge).
- Cross-version immunity claims appear as unaddressed presentations (the
  `addresses()` check returns false because canonical_paths differ).
- Both behaviors are correct — they enforce periodic re-verification of
  cross-crate claims at version boundaries.

**What v0.1 audit does NOT do** (and why): it does not auto-translate v1
immunities to v2; does not silence orphan warnings on version upgrade; does
not warn the user they upgraded a dep. The orphan warning is the feature.

**Open Question 1** (documented limitation): same-named crates from different
registries (`crates.io foo@1.0.0` vs alt-registry `foo@1.0.0`) produce the
same canonical_path string. Alt-registry users typically vendor or rename;
this limitation is acceptable for v0.1. Forward-compatible: a future
structured `CanonicalPath { crate_name, version, registry }` form requires
only a schema_version bump.

**A4+ candidate**: semver-range descent claims (`#[descended_from(Parent,
semver = "~1.0")]`). Future ADR; not A3 scope. Forward-compatible with
current `Option<String>` shape.

#### The trust delegation

**Both preconditions are satisfied by construction via `enumerate_dep_crate_roots`**,
not by runtime layout-check. The function is the only public mechanism for
enumerating cross-crate scan targets, and every path it returns is sourced
from `cargo metadata`'s output. Cargo verifies registry layout itself before
populating that output. Workspace-internal packages (`source: null` in cargo
metadata) are explicitly excluded from `enumerate_dep_crate_roots`'s output —
their antigens enter the scan via the workspace pass with `canonical_path =
None`, not via the cross-crate pass. The filter is
`package.source.is_some_and(|s| s.starts_with("registry+"))`.

**Do not add alternative path-discovery mechanisms that bypass
`enumerate_dep_crate_roots`** — doing so bypasses the sub-clause F delegation
and requires a separate trust argument. ATK-A3-007 enforces this discipline.

**Trust scope statement**: antigen's cross-crate trust model is predicated on
cargo metadata integrity. The trust-delegation guarantee in this ADR extends
*only* as far as cargo's own trust chain (cargo's checksum verification,
cargo's resolution graph, cargo's registry-layout enforcement). Attacks against
cargo itself are out of antigen's trust model:

- **CARGO_HOME override**: an attacker who sets `CARGO_HOME` to a prepared
  directory can substitute a malicious registry cache. `cargo metadata` will
  resolve against the tampered cache; antigen will trust that resolution.
- **Cargo.lock manipulation**: an attacker with write access to the workspace
  can rewrite `Cargo.lock` to pin different versions of a dependency, including
  versions with malicious antigen declarations.
- **Registry cache tampering**: an attacker with filesystem write access to
  `.cargo/registry/src/<index>/<crate-version>/` can modify source files
  post-fetch (cargo verifies the original fetch, not subsequent filesystem
  state).

These are cargo-level attacks. They violate cargo's own trust assumptions, and
antigen's trust delegation has no independent defense against them. Consumers
who need a stronger guarantee require a stronger cargo (or a different
dependency-management tier entirely); antigen's claim is *consistent with
cargo's*, not *stronger than cargo's*.

This is sub-clause F applied to the trust-delegation surface: antigen reports
the strength of trust it actually extends. The honest-boundary is registered as
an encounter-tier substrate (per Q7 honest-boundary-as-encounter-registration
discipline, when ratified).

The two preconditions (path reachable from cargo metadata's resolution graph;
path's parent directory matches registry layout) are both honored by
construction via `enumerate_dep_crate_roots`. The scanner does NOT re-verify
checksums or dependency authenticity — cargo's resolution chain is the authority.
Sub-clause F by delegation (ADR-002 compose-not-compete applied to the trust
boundary itself).

### Sweep-level consequences

- **A3**: `canonical_path` field lands on all five types + LineageEdge.
  `unaddressed_presentations()` amended. `scan_workspace` option A (caller
  stamps `canonical_path` post-scan via `enumerate_dep_crate_roots`). D3
  already shipped in commit `9b677c6`.
- **A4-A5**: behavioral re-validation of inherited presentations consumes
  canonical_path-aware identity for O(1) ancestor lookup (via ADR-018's
  `ProvenanceEntry`).
- **v0.1.0 public API**: `canonical_path: Option<String>` is a public field.
  Backward-compatible (serde default). Semver-minor bump policy applies.

### Enforcement

- `orphaned_lineage_edges()` and `dangling_child_lineage_edges()` MUST
  compare `(type_name, canonical_path)` tuples, not bare names. Detected
  by cross-crate fixture (ATK-A3-006): edge with `parent_canonical_path:
  Some("foo@1.0.0")` NOT satisfied by AntigenDeclaration with same
  `type_name` but different `canonical_path`.
- The propagation walk MUST use `(type_name, canonical_path)` tuple lookup
  for parent endpoint resolution. Detected by version-mismatch fixture:
  `#[descended_from(XVuln)]` pointing at `foo@1.0.0` when only `foo@1.1.0`
  is in scan produces orphaned-edge warning, NOT silently-wrong propagation.
- `enumerate_dep_crate_roots` is the only public mechanism for returning
  registry paths to scan (ATK-A3-007 fixture).

### Resolves

- Cross-crate antigen identity: bare type-name ambiguity at the cross-crate
  boundary.
- ADR-001 C7's cross-crate consumption commitment — now has a substrate
  mechanism.
- ADR-005's named trust-boundary item 4 (cross-crate antigen consumption)
  — now has a validation mechanism (cargo delegation).
- The orphan-detection gap for `orphaned_lineage_edges()` canonical_path
  comparison (adversarial enforcement review, 2026-05-09).
- ATK-A3-006 (cross-crate canonical_path-aware orphan detection).

### Open questions deferred

1. Alt-registry disambiguation (same name+version, different registry).
   **Trigger**: adoption feedback from alt-registry users.
2. Structured `CanonicalPath` type (name, version, registry fields).
   **Trigger**: schema_version bump migration tooling is available.
3. Behavioral validation of `#[descended_from]` claim (does the inheritance
   hold structurally?). Cross-listed with ADR-018 OQ4.
4. Re-export resolution (witness `crate_α::fn` re-exported as `crate_β::fn`).
   Out of scope — witness-resolution problem, not antigen identity problem.
5. Semver-range descent claims (`semver = "~1.0"` on `#[descended_from]`).
   **Trigger**: adoption feedback on re-attestation overhead at version boundaries.

---

## [ADR-018] `#[descended_from]` propagation: tagged synthesis + diamond dedup + inheritance state matrix

**Status**: Ratified 2026-05-09.

**Participants**: aristotle (Phase 1-8 + draft), pathmaker (substrate
verification), navigator (scope-lock, propagation question routing,
refinement coordination, v3 absorption), naturalist (biology cognate
validation: lineage-as-clonal-line), adversarial (BUG-A3-001 duplicate-edge
gap, BUG-A3-002 dangling-child case, enforcement review A18-01 through
A18-09), team-lead + Tekgy (sibling ratification with ADR-017;
`ProvenanceEntry` Option C ratification 2026-05-09).

**Related**:
- ADR-001 Amendment 1 Change 2 (5-state interaction matrix; this ADR amends to 7-state)
- ADR-005 §Decision item 2 (inheritance propagation trust boundary)
- ADR-005 Amendment 3 (audit reports its own tier honestly; AuditHint integration)
- ADR-007 (anti-YAGNI structurally-guaranteed; full propagation logic must ship)
- ADR-008 Amendment 1 (multi-contributor warn-not-error severity)
- ADR-011 (`#[antigen_tolerance]` is the escape valve for intentional inheritance)
- ADR-013 (`#[descended_from]` meaningful only on antigen-type declarations)
- ADR-017 (sibling ADR; identity primitives this ADR consumes)

**Implicit pattern elevated** (per ADR-004 Enforcement):

The current substrate has `#[descended_from]` declarations and lineage edges
collected in `ScanReport.lineage_edges`, but the propagation walk that consumes
edges to synthesize inherited presentations does not yet exist. The implicit
convention "lineage edges are collected; what they DO is decided later" is
load-bearing-by-omission. This ADR elevates the implicit-by-omission to
explicit tagged-synthesis with diamond dedup.

### Finding

A3's scope-lock commits to `#[descended_from]` propagation as Deliverable 1.
Pathmaker surfaced a question before implementing: what does the synthesis
pass actually DO with inherited presentations? Three readings:

1. **Literal scope-lock**: synthesize inherited Presentation records as-if-marked; audit treats them identically to explicit markers.
2. **ADR-005-strict**: scan collects lineage edges only; audit walks chain at audit-time.
3. **Pathmaker's recommendation**: synthesize inherited Presentations BUT mark them with `MatchKind::DescendedFrom { parent }` variant; audit emits re-attestation hint.

Phase 1-8 (aristotle) deconstructed all three plus seven additional approaches.
The depth-shift cut: ADR-005 §Decision item 2 says scan re-checks inherited
witnesses; P2 says behavioral re-check is A4-A5 work. A3's job is to surface
the inheritance state in a form A4-A5's behavioral-tier can consume. This is
Approach 4 + Approach 8 hybrid: no MatchKind change; provenance lives in a
separate `inherited_from` field.

### Decision

**Inheritance propagates as tagged Presentation records carrying provenance
in an `inherited_from: Option<Vec<ProvenanceEntry>>` field. `MatchKind` is
unchanged. Diamond inheritance dedupes by `(antigen_type, item_target,
canonical_path)` tuple, merging inherited_from chains by set-union. The audit
emits warn-level diagnostics for inherited+unaddressed presentations.**

### Mechanics

#### `ProvenanceEntry` struct and `inherited_from` field

```rust
/// Provenance entry: the identity of one ancestor antigen whose
/// presentations propagated to this descendant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    /// Antigen type name at the ancestor declaration site.
    pub antigen_type: String,
    /// Crate-name@version where the ancestor antigen was declared.
    /// `None` = ancestor is intra-workspace.
    pub canonical_path: Option<String>,
}
```

Field on `Presentation`:

```rust
#[serde(default)]
pub inherited_from: Option<Vec<ProvenanceEntry>>,
```

Semantics: `None` = direct match. `Some(chain)` = inherited via these ancestor
antigens, each fully identified by `(antigen_type, canonical_path)` tuple.
Order not significant; use `BTreeSet<ProvenanceEntry>` internally (requires
`Ord`), serialize as `Vec<ProvenanceEntry>` for JSON schema stability.
Empty Vec inside Some: forbidden — collapse to `None` (defensive normalization
at construction).

**Rationale for `ProvenanceEntry` over bare `String`** (ADR-004, ADR-006,
ADR-007):
- ADR-004 (implicit-to-explicit): bare strings hide canonical_path as implicit
  co-variance. ProvenanceEntry makes full identity explicit in the type system.
- ADR-006 (recognition-not-design): `(antigen_type, canonical_path)` is the
  identity tuple used by `unaddressed_presentations()`, diamond dedup, and all
  lineage-edge query methods. ProvenanceEntry is recognition of existing
  structure.
- ADR-007 (anti-YAGNI): A4-A5 behavioral re-validation is structurally
  guaranteed; bare strings force O(lineage_edges) traversal per lookup.
  ProvenanceEntry gives O(1). Field is pre-commit — no migration cost.

#### The synthesis algorithm

```
For each child antigen with at least one outgoing lineage edge:
    visited = HashSet::new()
    ancestors = transitive_closure(child, lineage_edges, visited)
    For each (ancestor_decl, edge) in ancestors:
        provenance = ProvenanceEntry {
            antigen_type: ancestor_decl.type_name,
            canonical_path: edge.parent_canonical_path,
        }
        For each presentation P on ancestor_decl:
            if exists explicit/fingerprint Presentation on child site
                with same (antigen_type, item_target, canonical_path):
                    update existing.inherited_from = set_union(existing.inherited_from, [provenance])
                    continue
            if exists inherited Presentation on child site
                with same (antigen_type, item_target, canonical_path):
                    existing.inherited_from = set_union(existing.inherited_from, [provenance])
                    continue
            append Presentation {
                antigen_type: P.antigen_type,
                file: child.declaration_file,
                line: child.declaration_line,
                item_kind: child.item_kind,
                item_target: child.item_target,
                match_kind: P.match_kind,   // preserve ancestor's
                inherited_from: Some([provenance]),
                canonical_path: P.canonical_path,
            }
```

The descendant inherits the ancestor's `match_kind`. Inheritance is provenance,
not match-kind.

**Mechanism note**: the `visited: HashSet<&str>` in the pseudocode is *scoped
per descendant DFS*, not global to the propagation walk. Each descendant's DFS
visits each transitive ancestor at most once (defense-in-depth against cycles
bypassed at the lineage-safety layer). Diamond dedup operates at a different
layer — the `(antigen_type, item_target, canonical_path)` dedup key on the
resulting Presentation records ensures that even if two descendants' DFS walks
reach the same ancestor presentation independently, the *Presentation* records
produced are merged by set-union of `inherited_from`. Per-DFS visited-set
prevents intra-descendant cycle traversal; per-Presentation dedup-key prevents
cross-descendant duplication. Both layers operate; neither replaces the other.

#### Diamond dedup

**Same-version true-diamond is the dedup case the algorithm handles directly**:
A→B→D and A→C→D where D's `canonical_path` is identical via both paths produces
ONE Presentation record on A with both B and C unioned into `inherited_from`.
The cross-version case below is *not* a diamond — it explicitly does NOT
collapse, because `canonical_path` differs between paths.

When a descendant has multiple paths to the same ancestor presentation
(diamond: A→B→D and A→C→D), the second visit hits the dedup branch and merges
inherited_from chains by set-union. Result: one Presentation record per
`(antigen, item, canonical_path)` tuple per descendant, with inherited_from
carrying ALL transitive ancestors.

The dedup key `(antigen, item, canonical_path)` corresponds to the three
components of `addresses()` (ADR-017 §addresses): `same_antigen_identity` +
`same_item` + `same_locus`. Consistent by construction.

**Cross-version diamond is not a diamond**: under ADR-017's `name@version`
format, `foo@1.0.0::D` and `foo@1.1.0::D` have different `canonical_path`
values and are distinct antigens. A child with lineage chains through both
versions inherits two distinct Presentations, each entering state 7 until
separately re-attested. Collapsing versions would skip a sub-clause F check —
the audit warnings for both records are the correct mechanism.

#### Edge-level dedup

Edge-level dedup runs before BOTH `detect_lineage_failures` AND the propagation
walk (BUG-A3-001). Edges are deduped by `(child, parent, child_canonical_path,
parent_canonical_path)` tuple. A user accidentally writing
`#[descended_from(A)] #[descended_from(A)]` produces two edges; dedup collapses
to one. Dedup MUST happen before cycle detection to avoid spurious cycle
detection on duplicated paths.

Implementation order in `scan_workspace`:
1. Edge collection (existing visitor pass).
2. **Edge-level dedup** (new).
3. Cycle detection (`detect_lineage_failures`) consumes the deduped edge set.
4. Propagation walk consumes the same deduped edge set.

#### Stale-lineage interaction

The propagation walk does NOT walk through orphaned edges (parent antigen not
in scan) or dangling-child edges (child antigen not in `self.antigens`).
Both produce no inherited presentations on the descendant; both surface via
query methods (`orphaned_lineage_edges()` and `dangling_child_lineage_edges()`).

Both query methods compare `(type_name, canonical_path)` tuples, not bare
names. An edge with `parent_canonical_path: Some("foo@1.0.0")` is satisfied
ONLY by an `AntigenDeclaration` with both matching fields.

#### The 7-state interaction matrix

ADR-001 Amendment 1 Change 2 ratified a 5-state matrix. This ADR amends to 7 states:

1. **Marked + matched** (existing): `#[presents(X)]` + matching `#[immune(X, witness=Y)]` on same item.
2. **Passively detected** (existing): no markers; matches X's fingerprint.
3. **Inconsistent** (existing): `#[presents(X)]` on item NOT matching X's fingerprint.
4. **Tolerated** (existing): `#[antigen_tolerance(X)]` on a matching site.
5. **Stale tolerance** (existing): `#[antigen_tolerance(X)]` on a site matching no fingerprint.
6. **Inherited + re-attested** (NEW): `inherited_from = Some(_)` AND descendant has explicit `#[immune]` or `#[antigen_tolerance]` addressing the same antigen on the same item.
7. **Inherited + unaddressed** (NEW): `inherited_from = Some(_)` AND descendant has neither immune nor tolerance for the same antigen on the same item. Audit emits warn diagnostic (default) or error (`--strict`).

**Anti-case for state 6**: an item with `#[presents(A)]` and an inherited
Presentation for B (different antigen) is state 1 for A and state 7 for B.
`#[presents(A)]` does NOT re-attest an inherited Presentation for a different
antigen. State 6 requires the explicit marker and inherited Presentation to be
for the **same antigen** on the same item.

**No state 8**: orphaned/dangling edges produce no Presentation records — they
surface at the report layer via query methods. A Presentation record for a
broken lineage would have no well-defined `inherited_from` content. Audit
can cross-reference ("broken lineage claim — see orphaned-edge warning") as a
rendering choice without substrate-state change.

#### Audit diagnostic text

Default (warn) format:

```
warning: inherited presentation: `<Antigen>` flowed from `<Ancestor>`
  (declared in <ancestor-source>) to `<Descendant>` via `#[descended_from]`;
  the witness inherited from the ancestor has not been re-attested on the
  descendant. Add `#[immune(<Antigen>, witness = ...)]` or
  `#[antigen_tolerance(<Antigen>, rationale = "...")]` on the descendant.
  --> <descendant-source>:<line>
```

State 7 AuditHint: `inherited-presentation-not-re-attested: behavioral-tier
audit (A4-A5) will validate that the ancestor's witness applies to descendant;
reachability-tier audit cannot perform this check.` SHOULD include reference
to ancestor immunity declarations to help the user evaluate re-attestation.

### Sweep-level consequences

- **A3 implementation** adds `ProvenanceEntry` struct + `inherited_from:
  Option<Vec<ProvenanceEntry>>` on `Presentation` (~15 lines struct + serde),
  propagation walk (~80 lines), audit diagnostic (~30 lines), tests (~80
  lines covering linear chain, diamond, deep chain, orphaned-edge non-walk,
  explicit-vs-inherited precedence, fingerprint-match-vs-inherited precedence,
  cross-crate inheritance, immunity-does-not-inherit, tolerance-covers-inherited).
- `ScanReport schema_version` bumps once for combined ADR-017 + ADR-018 changes.
- 7-state matrix amends ADR-001 Amendment 1 Change 2.
- **A4** scope unblocks: behavioral re-validation of inherited witnesses
  (state 7 → A4-A5 check); `ProvenanceEntry` provides O(1) ancestor identity
  lookup for that check.

### Enforcement

- The propagation walk MUST run after `detect_lineage_failures` passes clean.
  Detected by ordering test.
- `inherited_from = Some(empty_vec)` MUST never appear. Detected by serde
  round-trip property test.
- Diamond cases MUST produce exactly one Presentation per `(antigen, item,
  canonical_path)` tuple per descendant. Detected by diamond-fixture test.
- Explicit `#[presents]` + inheritance overlap MUST produce one Presentation
  with `MatchKind::ExplicitMarker` and `inherited_from = Some(_)`.
- Fingerprint-match + inheritance overlap MUST produce one Presentation with
  `MatchKind::FingerprintMatch` and `inherited_from = Some(_)`.
- Immunity MUST NOT auto-propagate. Detected by inheritance-fixture test.
- State 7 MUST emit warn-level diagnostic by default and error-level under
  `--strict`.
- Tolerance MUST cover inherited presentations (state 4 absorbs
  inherited+tolerated).
- Orphaned lineage edges MUST NOT be walked through.
- Dangling-child lineage edges MUST NOT be walked through (BUG-A3-002).
- Edge-level dedup MUST run before BOTH `detect_lineage_failures` AND the
  propagation walk (BUG-A3-001).
- Both `orphaned_lineage_edges()` and `dangling_child_lineage_edges()` MUST
  compare `(type_name, canonical_path)` tuples, not bare names.
- The propagation walk MUST use `(type_name, canonical_path)` tuple lookup
  for parent endpoint resolution, not bare-name lookup.
- Each `inherited_from` entry MUST identify the ancestor antigen with
  canonical_path fidelity — `(antigen_type, canonical_path)` ProvenanceEntry,
  not bare name. Detected by cross-crate diamond fixture.
- Anti-case fixture: `#[presents(A)]` + inherited Presentation for B →
  state 7 for B, not state 6.

### Resolves

- The propagation-semantics question pathmaker surfaced (Reading 1 vs 2 vs 3).
- ADR-005 §Decision item 2's "scan walks descended_from chains and re-checks"
  language gains an A3-tractable substrate.
- Diamond Phase 1-8 and propagation Phase 1-8 findings — ratified into substrate.
- ADR-001 Amendment 1 Change 2 5-state matrix — amended to 7-state.
- The implicit-by-omission "what does propagation DO?" elevated to explicit
  tagged-synthesis with diamond dedup.
- Adversarial BUG-A3-001 (edge-level dedup precondition for cycle detection).
- Adversarial BUG-A3-002 (dangling-child edge non-walk).
- Adversarial enforcement findings A18-01/04/09 (inherited_from canonical_path
  granularity — resolved by ProvenanceEntry).

### Open questions deferred

1. **Provisional inheritance via isotype-switching analog**: defer until stdlib
   antigens with shared lineage produce ergonomic-friction adoption signals.
2. **Lazy/audit-time synthesis**: defer until scan-side synthesis at ecosystem
   scale produces real performance pressure.
3. **Path-attribution beyond set-of-ancestors**: would consumers benefit from
   per-path provenance? Trigger: audit use case requiring path detail beyond
   inherited_from + lineage_edges cross-reference.
4. **Behavioral validation of `#[descended_from]` claim itself**. Cross-listed
   with ADR-017 Open Question 3.
5. **Witness re-validation across inheritance** (A4-A5 behavioral-tier work;
   surfaced by state 7 + AuditHint). Includes the multi-version case: with
   `ProvenanceEntry` carrying canonical_path, A4-A5 ancestor lookup is O(1) —
   this open question is substantially simplified vs bare-string Option A.

---

## Amendment template

When an ADR needs to be amended (not superseded), add an Amendment section:

```
## ADR-NNN Amendment N — [title]

**Status**: Ratified [date].
**Amends**: ADR-NNN.
**Reason**: [structural-forcing argument; pure refinement vs. expansion].
**Change**: [precise diff to the original ADR].
**Resolves**: [new findings since original ratification].
```

When an ADR is superseded (not amended), the new ADR's "Related" field references the
old, and the old ADR's status becomes "Superseded by ADR-MMM".

---

## Adding a new ADR

The full ADR lifecycle is documented in [`docs/process.md`](process.md). Quick
checklist:

1. Number sequentially. Skip numbers only with explicit reservation.
2. Open a campsite under `campsites/adr-NNN-<slug>` for the in-flight draft.
3. Use the section template above (Status, Participants, Related, Finding, Decision,
   Mechanics, Sweep-level consequences, Enforcement, Resolves). Within the Finding
   section, optionally include an **"Implicit pattern elevated"** sub-clause naming
   the implicit-mode convention this ADR replaces with explicit structure (per
   ADR-004's enforcement clause; F-TEMPLATE-1 from scientist's validation pass).
   Foundational ADR-001 through ADR-010 contain this analysis embedded in their
   Finding prose; future ADRs may surface it as a labeled sub-clause for clarity.
4. Run the draft through the full lifecycle: Phase 1-8 deconstruction (aristotle) →
   adversarial review → math/systems-research review → scientist validation →
   team-lead ratification.
5. After ratification:
   - Move the ratified text into this file (`decisions.md`)
   - Update the index at the top of this file
   - Update `docs/glossary.md` if the ADR introduces new vocabulary
   - Reference the ADR in any related code or other docs that act on its decisions
   - Mark the campsite `closed` with a final log entry

See [`docs/process.md`](process.md) for the complete process — including the
recursive insight that **ADRs are antigen-in-document-form** (the original
implementation of the structural-memory pattern that antigen-the-tool ships at the
code level).

---

## Convention notes

- **ADR vs. DEC**: this project uses "ADR" (Architecture Decision Record) following
  ecosystem convention. Tambear uses "DEC" (Decision Entry Container). Same shape,
  different naming for consistency with broader Rust/software-architecture practice.
- **Ratification authority**: ADRs at the foundational level (1-8) require team-lead
  ratification. ADRs above 8 (project ratifications during expeditions) follow the
  team's normal Phase 1-8 review and ratification process.
- **Pre-team ADRs (1-8)**: these were ratified by Tekgy + Claude in the pre-team
  scaffolding session. They are foundational and should not be casually amended; major
  amendments require explicit deconstruction by the antigen team.

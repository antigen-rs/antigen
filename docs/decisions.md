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
- [ADR-002 — Compose, don't compete](#adr-002--compose-dont-compete) *(amended by ADR-013)*
- [ADR-003 — Biological metaphor is load-bearing, not decorative](#adr-003--biological-metaphor-is-load-bearing-not-decorative)
- [ADR-004 — Implicit-to-explicit elevation as architectural posture](#adr-004--implicit-to-explicit-elevation-as-architectural-posture)
- [ADR-005 — Sub-clause F at every trust boundary](#adr-005--sub-clause-f-at-every-trust-boundary)
- [ADR-006 — Recognition, not design](#adr-006--recognition-not-design)
- [ADR-007 — Anti-YAGNI: structurally-guaranteed need](#adr-007--anti-yagni-structurally-guaranteed-need)
- [ADR-008 — Named-observer position as terminal stratum](#adr-008--named-observer-position-as-terminal-stratum)
- [ADR-009 — Adoption gradient: antigen meets consumers at any discipline level](#adr-009--adoption-gradient-antigen-meets-consumers-at-any-discipline-level)
- [ADR-010 — Fingerprint grammar v1: syn-based AST visitor pattern](#adr-010--fingerprint-grammar-v1-syn-based-ast-visitor-pattern) *(amended by ADR-012)*
- [ADR-011 — `#[antigen_tolerance(...)]`: opt-out for legitimate fingerprint matches](#adr-011-antigen_tolerance-opt-out-for-legitimate-fingerprint-matches)
- [ADR-012 — ADR-010 Amendment 1: function-body patterns + match-context awareness](#adr-012-adr-010-amendment-1-function-body-patterns--match-context-awareness)
- [ADR-013 — ADR-002 Amendment 1: phantom-type witness recognition + witness-validity tier mapping](#adr-013-adr-002-amendment-1-phantom-type-witness-recognition--witness-validity-tier-mapping)
- [ADR-014 — `#[antigen_generates(...)]`: declaring antigens that proc-macros emit](#adr-014-antigen_generates-declaring-antigens-that-proc-macros-emit)

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
        has_method('meet', '(Self, Self) -> Self'),
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

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
- [ADR-002 — Compose, don't compete](#adr-002--compose-dont-compete) *(amended by ADR-013, ADR-015, Amendment 2)*
  - [ADR-002 Amendment 2 — Compose where external expertise serves; compete where antigen cohesion serves](#adr-002-amendment-2--compose-where-external-expertise-serves-compete-where-antigen-cohesion-serves)
- [ADR-003 — Biological metaphor is load-bearing, not decorative](#adr-003--biological-metaphor-is-load-bearing-not-decorative) *(amended by Amendment 1)*
  - [ADR-003 Amendment 1 — Biology is BOTH teaching tool AND discovery framework](#adr-003-amendment-1--biology-is-both-teaching-tool-and-discovery-framework)
- [ADR-004 — Implicit-to-explicit elevation as architectural posture](#adr-004--implicit-to-explicit-elevation-as-architectural-posture)
- [ADR-005 — Sub-clause F at every trust boundary](#adr-005--sub-clause-f-at-every-trust-boundary)
  - [ADR-005 Amendment 2 — Rationale-as-required-field as transverse sub-clause F discipline](#adr-005-amendment-2--rationale-as-required-field-as-transverse-sub-clause-f-discipline)
  - [ADR-005 Amendment 3 — Audit reports its own tier honestly](#adr-005-amendment-3--audit-reports-its-own-tier-honestly)
- [ADR-006 — Recognition, not design](#adr-006--recognition-not-design) *(amended by Amendment 1)*
  - [ADR-006 Amendment 1 — Recognition discipline scoped to adopter-extension; stdlib growth is research-discipline](#adr-006-amendment-1--recognition-discipline-scoped-to-adopter-extension-stdlib-growth-is-research-discipline)
- [ADR-007 — Anti-YAGNI: structurally-guaranteed need](#adr-007--anti-yagni-structurally-guaranteed-need)
- [ADR-008 — Named-observer position as terminal stratum](#adr-008--named-observer-position-as-terminal-stratum)
  - [ADR-008 Amendment 1 — Multi-contributor workflow + scan severity defaults](#adr-008-amendment-1--multi-contributor-workflow--scan-severity-defaults)
- [ADR-009 — Adoption gradient: antigen meets consumers at any discipline level](#adr-009--adoption-gradient-antigen-meets-consumers-at-any-discipline-level)
  - [ADR-009 Amendment 1 — Fingerprint is required iff scan-locatable; verify-only antigens have no fingerprint](#adr-009-amendment-1--fingerprint-is-required-iff-scan-locatable-verify-only-antigens-have-no-fingerprint)
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
  - [ADR-017 Amendment 1 — Cross-crate `addresses()` resolution: the sub-clause-F clause at the cross-crate reference boundary](#adr-017-amendment-1--cross-crate-addresses-resolution-the-sub-clause-f-clause-at-the-cross-crate-reference-boundary)
- [ADR-018 — `#[descended_from]` propagation: tagged synthesis + diamond dedup + inheritance state matrix](#adr-018-descended_from-propagation-tagged-synthesis--diamond-dedup--inheritance-state-matrix)
  - [ADR-018 Amendment 1 — Inheritance is provenance, not substitutability](#adr-018-amendment-1--inheritance-is-provenance-not-substitutability)
- [ADR-019 — Substrate-witness predicate family](#adr-019-substrate-witness-predicate-family)
- [ADR-020 — Cross-cutting attestation primitive](#adr-020-cross-cutting-attestation-primitive)
- [ADR-021 — OracleRef generalization + additive-only schema evolution + oracle-as-artifact-class](#adr-021-oracleref-generalization--additive-only-schema-evolution--oracle-as-artifact-class)
- [ADR-022 — Stdlib-vs-Extension: Two Disciplines, One Public API](#adr-022--stdlib-vs-extension-two-disciplines-one-public-api)
- [ADR-023 — Deferred-Defense Family: Loudness-as-Discipline for Intentional Non-Immunity](#adr-023--deferred-defense-family-loudness-as-discipline-for-intentional-non-immunity)
- [ADR-024 — Three Sibling Families: Convergent Evidence + Recurrent Emergence + Prescriptive Work-Orchestration](#adr-024--three-sibling-families-convergent-evidence--recurrent-emergence--prescriptive-work-orchestration)
  - [ADR-024 Amendment 1 — `#[titer]` biology-grounding axis reassignment](#adr-024-amendment-1--titer-biology-grounding-axis-reassignment)
  - [ADR-024 Amendment 2 — Recurrent Emergence macro arg-signatures (shipped shapes from parse.rs)](#adr-024-amendment-2--recurrent-emergence-macro-arg-signatures-shipped-shapes-from-parsers)
  - [ADR-024 Amendment 3 — `from_itches` is class-specific (lineage-aware): the recurrence-anchor noticing-precondition](#adr-024-amendment-3--from_itches-is-class-specific-lineage-aware-the-recurrence-anchor-noticing-precondition)
- [ADR-025 — Supply-Chain Defense Family: Antigens for Dependency-Boundary Risk in the 2026+ Threat Landscape](#adr-025--supply-chain-defense-family-antigens-for-dependency-boundary-risk-in-the-2026-threat-landscape)
- [ADR-026 — VCS-Information-Loss Family: Structural Defense Against Git-History-Erasing Operations + Rollback-as-Triage Discipline](#adr-026--vcs-information-loss-family-structural-defense-against-git-history-erasing-operations--rollback-as-triage-discipline) *(amended by Amendments 1–4)*
  - [ADR-026 Amendment 1 — rollback-as-triage uses `#[triage_commit]`, not `#[orient]` extension](#adr-026-amendment-1--rollback-as-triage-uses-triage_commit-not-orient-extension)
  - [ADR-026 Amendment 2 — TriageDecision variant-semantic backfill + camp::triage connection-claim discipline](#adr-026-amendment-2--triagedecision-variant-semantic-backfill--camptriage-connection-claim-discipline)
  - [ADR-026 Amendment 3 — rollback detection algorithm (AUTHOR-DECLARATION) + structural enforcement verification requirement](#adr-026-amendment-3--rollback-detection-algorithm-author-declaration--structural-enforcement-verification-requirement)
  - [ADR-026 Amendment 4 — Rollback detection step-2 signal: commit-trailer not codebase-presence](#adr-026-amendment-4--rollback-detection-step-2-signal-commit-trailer-not-codebase-presence)
- [ADR-027 — Mucosal Boundary Taxonomy + Mapping Discipline](#adr-027--mucosal-boundary-taxonomy--mapping-discipline)
- [ADR-028 — Antigen-Category Taxonomy: Substrate-Alignment vs Functional-Correctness as First-Class Distinction](#adr-028--antigen-category-taxonomy-substrate-alignment-vs-functional-correctness-as-first-class-distinction)
  - [ADR-028 Amendment 6 — tier marker (object | description): relationship to own declaration](#adr-028-amendment-6--tier-marker-object--description-relationship-to-own-declaration)
  - [ADR-028 Amendment 7 — silence-generator witness-selection guidance + witness-locus split within SubstrateAlignment](#adr-028-amendment-7--silence-generator-witness-selection-guidance--witness-locus-split-within-substratalignment)
- [ADR-029 — Immunity Is Observed, Not Declared: `#[defended_by]` + `#[presents]` Evidence Extension](#adr-029--immunity-is-observed-not-declared-defended_by--presents-evidence-extension)
  - [ADR-029 Amendment 1 — Substrate-intent precedence: a failing `requires=` is not masked by a code witness](#adr-029-amendment-1--substrate-intent-precedence-a-failing-requires-is-not-masked-by-a-code-witness)
- [ADR-030 — Aggregate and Temporal Properties Are Audit-Observed](#adr-030-aggregate-and-temporal-properties-are-audit-observed)
- [ADR-031 — Negative Selection: `#[no_longer_presents(X)]` and Revocation-Staleness Observation](#adr-031-negative-selection-no_longer_presentsx-and-revocation-staleness-observation)
- [ADR-032 — Conjunction Witnesses: Required-All Defense Semantics](#adr-032-conjunction-witnesses-required-all-defense-semantics)
- [ADR-019 Amendment 1 — Witness Taxonomy: Two Kinds (Categorical ‖ Titer/Scalar), Each with Named Members + a Generic Escape-Hatch](#adr-019-amendment-1--witness-taxonomy-two-kinds-categorical--titerscalar-each-with-named-members--a-generic-escape-hatch)
- [ADR-033 — Prescriptive Work-Orchestration: Four Structural Shapes, Eight Clinical Names, the ADR-029 Spine Pointed at Work-Needs](#adr-033-prescriptive-work-orchestration-four-structural-shapes-eight-clinical-names-the-adr-029-spine-pointed-at-work-needs)
- [ADR-034 — The Report Is a Live Projection, Never a Stored Truth](#adr-034-the-report-is-a-live-projection-never-a-stored-truth)
- [ADR-035 — Cardinality Collapse at a Trust Boundary: the Three-Valued Type Law (a Self-Applying Antigen)](#adr-035-cardinality-collapse-at-a-trust-boundary-the-three-valued-type-law-a-self-applying-antigen)
- [ADR-036 — The Scan/Audit Orchestration Decomposition: a Thin Out-of-Band Coordinator Above the Detector Sequence (the SCRAM Host)](#adr-036-the-scanaudit-orchestration-decomposition-a-thin-out-of-band-coordinator-above-the-detector-sequence-the-scram-host)
- [ADR-037 — Antigen Is a Closed-Loop Regulator: Its Own Machinery Has Six Failure-Points (the Control-Loop Master-Frame)](#adr-037-antigen-is-a-closed-loop-regulator-its-own-machinery-has-six-failure-points-the-control-loop-master-frame)
- [ADR-038 — The Stdlib Taxonomy Grid: Three Divergence-Genera (One per Active Loop-Stage), Super-Family Parents, and Remedy-Shape as the Primary Sort](#adr-038-the-stdlib-taxonomy-grid-three-divergence-genera-one-per-active-loop-stage-super-family-parents-and-remedy-shape-as-the-primary-sort)
- [ADR-039 — The Confidence Dial, the Build Gate, and the Emit Seam: Three Admission Decisions and One Typed-Event Stream](#adr-039-the-confidence-dial-the-build-gate-and-the-emit-seam-three-admission-decisions-and-one-typed-event-stream)
- [ADR-040 — The Grammar Increment: Frame-Relative Matching, `body_calls`, and the Syntactic Absence Family (Leaf-Matcher Tier)](#adr-040-the-grammar-increment-frame-relative-matching-body_calls-and-the-syntactic-absence-family-leaf-matcher-tier)
- [ADR-041 — The Marked-Unknown Plane: a Declarable Three-Valued Bottom on Two Orthogonal Axes (Magnitude × Existence-Certainty), Surfaced at the Dial's Non-Gating Floor](#adr-041-the-marked-unknown-plane-a-declarable-three-valued-bottom-on-two-orthogonal-axes-magnitude--existence-certainty-surfaced-at-the-dials-non-gating-floor)
- [ADR-042 — The Usage-Discipline: Three Disciplines (Front-Line Liberal · Regulatory Sparing · Ranked Surfacing), and the `#[autoimmune]` Naming Reconciliation](#adr-042-the-usage-discipline-three-disciplines-front-line-liberal--regulatory-sparing--ranked-surfacing-and-the-autoimmune-naming-reconciliation)

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
  render at the named-observer stratum. **Scanner activation status:** v0.2's
  `cargo antigen scan --include-deps` scans each crate *independently* (per-crate
  `dep_reports`, `canonical_path` stamping per ADR-017) — it does NOT yet do
  cross-crate `addresses()` matching or fingerprint synthesis. The activation
  path that realizes this commitment in the scanner is tracked in
  [`roadmap.md`](roadmap.md) under "Cross-crate scan reachability (ADR-001 C7
  activation path)"; deferred from v0.1 by the Sweep A3 scope-lock.
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

## ADR-002 Amendment 2 — Compose where external expertise serves; compete where antigen cohesion serves

**Status**: Ratified 2026-05-22.

**Amends**: ADR-002 (Compose, don't compete) — ratified 2026-05-07; previously amended by ADR-013 (Amendment 1).

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named the amendment + locked 2026-05-21 night); naturalist (biology-grounding upgrade per Class-1 immune-system parallel-surface prediction); adversarial (B11 partial-lands; resolved via ADR-025 adopter-segmentation, not this amendment).

**Related**: ADR-002, ADR-003 (biology-as-discovery-framework strengthens this), ADR-013 (Amendment 1), NEW-ADR-022 (Stdlib-vs-Extension), NEW-ADR-025 (supply-chain compete decision).

### Finding

ADR-002's "default compose" became a project-wide constraint when its load-bearing reason (v0.1 ship discipline) had a narrower scope. The amended default is **substrate-grounded design choice**, not categorical-default. The Tekgy reframe identifies a clamping artifact: antigen needs its own surfaces at cohesion-critical points, exactly as the immune system evolved its own integrated vocabulary at surfaces where adjacent-system tools would have caused translation losses.

### Decision

**ADR-002 is amended to replace the "default compose" posture with a substrate-grounded design choice between composing and competing. The choice is made per primitive, per surface, per amendment — applying a distinguishing test, not a categorical default.**

The principle, stated normatively:

> When antigen considers a new primitive, witness type, tooling surface, or family, the design choice is between (a) COMPOSING with an existing tool (delegating the work + accepting the tool's vocabulary as part of antigen's contract) and (b) COMPETING by owning the equivalent surface (with antigen's opinions, vocabulary, integration, and cohesion). The choice is made by applying the **distinguishing test** below to the specific substrate; neither default is privileged.

**Compose when**:
- The external tool's specific expertise is hard to match
- Integration cost is low + maintenance burden low
- The tool's vocabulary doesn't conflict with antigen's
- Adopters benefit from access to the broader ecosystem

**Compete when**:
- Adopters in antigen's ecosystem benefit more from cohesion-within-antigen
- Antigen's opinions improve on the alternative
- The adjacent tool's vocabulary conflicts with antigen's (composing forces awkward translation)
- Owning the surface lets antigen evolve it in lockstep with the rest of the immune system

**The four-item substrate-citation requirement**: each compete decision requires substrate-citation naming (1) adjacent tools, (2) cohesion/opinion/vocabulary load-bearing reason, (3) measurable adopter-experience differential, and (4) that the alternative path is preserved.

### Biology grounding

The immune system has evolved its **own integrated vocabulary** at every surface where adjacent-system tools would have caused translation losses that break integrated response. The immune system did NOT compose with the endocrine system for cytokine signaling; it evolved its own. It did NOT borrow neural memory machinery; it evolved its own (B-cell/T-cell memory). It did NOT defer to stomach acid + skin barrier for antimicrobial chemistry; it evolved its own (defensins, complement, lysozyme).

This amendment's compose-vs-compete posture is **biology-predicted at the outcome level** (Class 1): knowing only immune biology, you would predict that antigen (committed to immune-system-as-tool) will need to own some surfaces. Biology is **silent** on the four-item substrate-citation mechanic itself (that's software-engineering invention); it grounds the structural necessity of competing at cohesion-critical surfaces.

### Mechanics

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| Four-item substrate-citation requirement | ADR-Phase-3 (during Strip) + ratification ceremony | process | hand-wavy citations caught at gates |
| Distinguishing-test application | ADR-Phase-3 + Phase-4 | process | future ADRs apply per-primitive; pattern reuse across ADR drafts |
| Alternative-path-preserved check | ADR-Phase-3 §What-this-ADR-does-NOT-do | process | ratified amendments explicitly state non-deprecation |

### Sweep-level consequences

- Biology cognate upgraded: cytokine signaling + memory machinery + antimicrobial chemistry as parallel-surfaces; substrate-cited
- Honest dual-axis grounding named: biology grounds outcome; software-engineering grounds process
- Each compete decision in a new ADR carries the four-item substrate-citation (verified by naturalist + adversarial gates)

### Resolves

- The "default compose" clamping artifact (per Tekgy lock 2026-05-21)
- The hand-wavy biology framing: Class-1 outcome-level grounding replaces "competition + cooperation coexist" generic claim
- The compose-default that fragmented supply-chain adopter experience (per ADR-025 compete decision)

### What this amendment does NOT do

- Does NOT abandon compose discipline; preserves it where external expertise + low integration cost warrant it
- Does NOT claim biology grounds the four-item substrate-citation mechanic (honest silence; software-engineering invention)
- Does NOT preclude future ADRs from surfacing additional compose-vs-compete decision mechanisms

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

## ADR-003 Amendment 1 — Biology is BOTH teaching tool AND discovery framework

**Status**: Ratified 2026-05-22.

**Amends**: ADR-003 (Biological metaphor is load-bearing, not decorative) — ratified 2026-05-07.

**Participants**: aristotle (draft + Phase 1-8); Tekgy (lock 2026-05-21 night); naturalist (biology-validation; gate passed, no major refinement needed).

**Related**: ADR-003, ADR-006 Amendment 1 (stdlib research-discipline), ADR-022 (Stdlib-vs-Extension).

### Finding

ADR-003 ratifies the biological metaphor as load-bearing for two roles: (1) teaching tool and (2) architectural-insight predictor. What the original text does NOT surface explicitly — and what the Tekgy lock makes clear — is a third role: **discovery framework for stdlib growth**.

The Tekgy lock (2026-05-21): "Each unused biological component is a research prompt. Mucosal immunity → input boundary defenses. MHC presentation → scope/visibility of declaration. Complement cascade → compound multi-step defenses. We've used ~10% of the metaphor. Each remaining component is a research arc."

### Decision

**ADR-003 is amended to make explicit that the biological metaphor is BOTH (1) a teaching tool for adopters AND (2) a discovery framework for stdlib growth. Coverage of the metaphor's immune-system components is the completeness check for stdlib comprehensiveness.**

**Discovery framework**: each immune-system component (T-cell, B-cell, macrophage, NK, dendritic, anergy, immunosuppress, vaccinate, complement, opsonize, Treg, autoimmune, lysozyme, mast, mucosal-IgA, etc.) is a research prompt. The naturalist role maintains `docs/expedition/biology-coverage-map.md` identifying which components have stdlib primitive equivalents and which are unmapped research-arc candidates. Stdlib completeness check: biological-metaphor coverage map IS the completeness measure.

**Teaching tool** (preserved from original ADR-003): documentation uses biological language freely; API documentation uses precise Rust terms with biological analogies cross-referenced; glossary anchors every term.

### Mechanics

ADR-003 §Mechanics extended:
- The naturalist role responsibility extends to maintaining `docs/expedition/biology-coverage-map.md`
- Stdlib research-arc planning consults the coverage map for candidate research prompts
- Stdlib ratification reviews include "where does this fit on the biology coverage map?" as a check
- Coverage map records trichotomy: MAPPED / UNMAPPED / INVESTIGATED-BUT-NOT-MAPPED

### Sweep-level consequences

- Biology coverage map becomes the discovery substrate for stdlib research arcs
- Naturalist role is BOTH metaphor-honesty guardian AND discovery-framework maintainer
- Stdlib completeness has a measurable target: coverage of the biological metaphor

### Enforcement

- New stdlib antigens cite their biological-component mapping in documentation
- Biology coverage map updated as part of stdlib research-arc drops
- Naturalist review at stdlib ratification time includes coverage-map consultation

### Resolves

- The implicit-discovery-role failure mode (biology was load-bearing but the discovery role wasn't explicit; research arcs were ad-hoc rather than systematic)
- The completeness question for stdlib: biology coverage map IS the measure
- The "what's a stdlib research arc?" question: unmapped immune-system component = candidate arc

### What this amendment does NOT do

- Does NOT abandon the teaching-tool role; preserves + strengthens it
- Does NOT require EVERY immune-system component to have a primitive (some components may not have software-engineering analogs; the coverage map names all three states)
- Does NOT make biology the ONLY discovery substrate; field knowledge, literature, postmortems, predictive analysis remain valid per ADR-006 Amendment 1

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
   or behavioral change invalidates the inheritance. (At reachability tier: topological
   re-attestation check, surfaces as state 7 `InheritedPresentationNotReAttested` hint
   per ADR-018. At behavioral tier: semantic witness re-validation against the descendant
   site — A4-A5 work, not yet implemented in v0.1.)

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

## ADR-006 Amendment 1 — Recognition discipline scoped to adopter-extension; stdlib growth is research-discipline

**Status**: Ratified 2026-05-22.

**Amends**: ADR-006 (Recognition, not design) — ratified 2026-05-07.

**Participants**: aristotle (draft + Phase 1-8); Tekgy (lock 2026-05-21 night); naturalist (biology-validation; gate passed); adversarial (no blocking findings on this amendment).

**Related**: ADR-003 Amendment 1 (biology-as-discovery-framework), ADR-006, ADR-007 (anti-YAGNI unchanged), ADR-022 (Stdlib-vs-Extension formalizes the separation this amendment creates).

### Finding

ADR-006 ratified "recognition-not-design" as a single discipline applied across ALL antigen development. This was structurally correct for the adopter-extension layer and structurally INCORRECT for the stdlib-growth layer.

The conflation arose because at ratification time (2026-05-07), antigen's scope was still being elaborated. The Tekgy reframe (2026-05-21): antigen ships a FULL immune system as core stdlib — every immune-component earns primitive status via **research + substrate-grounding from beyond direct encounter**.

The original framing — "recognize patterns that already exist in real-world Rust codebases" — is too narrow for two reasons: (1) biology metaphor actively predicts primitives no Rust codebase has yet encountered; (2) 2026+ dev contexts (agentic teams, vibe coders, AI-pair) produce failure-classes that wait-for-encounter cannot reach in safety-critical timelines.

### Decision

**ADR-006 is amended to apply recognition discipline at the ADOPTER EXTENSION layer; stdlib growth follows a separate RESEARCH discipline formalized in ADR-022.**

**Stdlib (research discipline)**:
- Trigger: research arc identifies a fail-class worth defending across all software development
- Source: biological-immune-component mapping; field knowledge (postmortems, RFCs, books, talks, training data, predictive analysis of 2026+ dev contexts); direct encounter is legitimate but NOT privileged
- Substrate-citability: every stdlib antigen carries non-empty `references = [...]` field at parse time
- Cadence: regular research-driven drops adding 10-30 antigens per arc
- Scope: comprehensive — aim at saturating the failure landscape per biological metaphor coverage check

**Adopter extension (recognition discipline, unchanged)**:
- Trigger: adopter project encounters domain-specific fail-class
- Source: direct encounter, bug history, lived team experience
- Cadence: organic per-project; their own versioning
- Scope: narrow — only what the domain needs

### Mechanics

ADR-006 §Mechanics amended:
- The naturalist role guards recognition discipline at the ADOPTER extension layer
- For stdlib additions: the discipline check is "what substrate grounds this antigen?" not "what direct encounter recognizes this?"
- Stdlib antigens MUST cite at least one substrate reference in `#[antigen(..., references = [...])]`; declaration without references is a parse-time error for `antigen-stdlib`

### Sweep-level consequences

- Stdlib aims at hundreds of antigens via research-arc drops, not organic 30-50 over years
- Biological metaphor coverage becomes the completeness check (per ADR-003 Amendment 1)
- Adopter-extension surface treated as first-class public API (per ADR-022)
- Code-review checklists branch on stdlib-vs-extension

### Enforcement

**Stdlib**: every antigen declaration requires non-empty `references = [...]` field at parse time; every antigen includes biological-component mapping in documentation.

**Adopter extension**: no mandatory references field; recognition-not-design check at adopter's code review per original ADR-006 enforcement.

### Resolves

- The over-restriction of antigen's stdlib growth (would have capped scope at ~30-50 antigens)
- The conflation of "extension recognition discipline" with "stdlib research discipline"
- The implicit assumption that stdlib grows organically from direct encounter
- The foreclosure of biology's discovery role (per ADR-003 Amendment 1)

### What this amendment does NOT do

- Does NOT abandon recognition discipline; preserves it at the adopter-extension layer
- Does NOT permit speculative stdlib additions; research-grounding is the substitute discipline
- Does NOT change ADR-007 anti-YAGNI commitment

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

## ADR-009 Amendment 1 — Fingerprint is required iff scan-locatable; verify-only antigens have no fingerprint

**Status**: Ratified 2026-05-27.

**Amends**: ADR-009 §Decision §Layer 1 Required fields.

**Related campsites**: `findings/verify-only-antigen-forced-fingerprint`.

**Reason**: ADR-009 ratified `fingerprint` as a required Layer-1 field for `#[antigen]`.
This was correct for the scan-locatable archetype (PanickingInDrop: located by source-structure
pattern). ADR-025 later ratified verify-only antigens (supply-chain family: UnpinnedDependency,
UnsandboxedBuildScript, etc.) whose detection-model is external-substrate (Cargo.lock, registry
metadata) — unreachable by the syn-based scanner. Forcing these antigens to carry a fingerprint
forces them to declare a scan-surface they do not have. The result: placeholder fingerprints
(`doc_contains("ADR-025")`) matching every file that mentions the ADR number, producing O(codebase
mentions) spurious scan presentations (~14,792 at measurement time; observer-confirmed). This is
a representation forced to diverge from substrate truth — the macro's required-fingerprint encoding
the false claim "I am scan-locatable" for antigens that are not.

**Aristotle Phase-1-8 verdict** (findings campsite story, 2026-05-27): PASS. F1 (R1+R3):
make fingerprint `Option<String>`; absent ⇒ verify-only (no scan-detection; audit evaluates via
`requires=`/witness only). This is recognition-not-design (ADR-006): the verify path already exists
in the audit layer; the macro is the only place forcing the scan-surface lie. Forced-rejection
confirmed: requiring fingerprint for all antigens would retroactively invalidate two ratified stdlib
families (ADR-025 supply-chain, ADR-026 VCS-info-loss). The required-fingerprint constraint must
yield, not the verify-only class.

**Detection-model axis (implicit → explicit, ADR-004)**: this finding surfaces an implicit axis
that ADR-009 encoded before verify-only antigens existed. The axis is:

```
detection_model ∈ { scan-locatable, verify-only, both }
```

- `scan-locatable`: antigen has a source-structure pattern the syn-scanner can match → `fingerprint`
  required.
- `verify-only`: antigen's condition is external substrate (Cargo.lock, registry, git history, sidecar)
  → no source-structure pattern exists → `fingerprint` absent.
- `both`: antigen has both a scan-surface AND a substrate-predicate → `fingerprint` required +
  `requires=`/witness also present.

This axis was IMPLICIT (fingerprint required = scan-locatable assumed universal). This amendment
makes it EXPLICIT via the optional/required fingerprint semantics.

### Change 1: `fingerprint` becomes `Option<String>` in `#[antigen]` — required iff scan-locatable

**ADR-009 §Decision §Layer 1 Required fields — amended:**

Before:
```
Required fields:
- `#[antigen]`: `name` (string identifier), `fingerprint` (structural pattern, see ADR-010)
```

After:
```
Required fields:
- `#[antigen]`: `name` (string identifier)
- `fingerprint` (structural pattern, see ADR-010): required when the antigen has a
  source-structure scan-surface; OMITTED for verify-only antigens whose detection is
  entirely via substrate-predicate (`requires=`) or external substrate checks
```

**Parse-time enforcement**: a `#[antigen]` without `fingerprint` is valid. The macro emits
no warning or error. The scan layer skips the antigen (no fingerprint = no scan-detection;
the antigen's only evaluation path is audit-time `requires=`/witness).

**Backward-compatibility**: existing antigens with `fingerprint` are unaffected. This is
additive relaxation only — no breaking change.

### Change 2: Supply-chain and VCS-info-loss stdlib antigens drop placeholder fingerprints

The 11 supply-chain antigens (`supply_chain.rs`) and all VCS-info-loss family antigens
(`vcs_info_loss.rs`) currently carry `fingerprint = doc_contains("ADR-025")` or equivalent
placeholder. Under this amendment: these antigens are `verify-only`; they drop the fingerprint
field. Their scan-detection output becomes zero by design (no fingerprint = no scan match),
and their audit evaluation proceeds entirely via `requires=`/witness.

This is the honest representation: these antigens DO NOT have source-structure detection-loci.
Removing their placeholder fingerprints removes ~14,792 spurious scan presentations and makes
`cargo antigen scan` output actionable.

### Companion clause for ADR-010

**ADR-010 scan-semantics addendum**: a `#[antigen]` declaration with no `fingerprint` field
is a **verify-only antigen**. The scan phase SKIPS it entirely (no fingerprint to match;
no scan output generated). The audit phase evaluates it via `requires=`/witness cross-reference.
This is consistent with ADR-010's existing scan-semantics: scan finds candidate presentations
by matching fingerprint patterns; if there is no pattern, there is nothing to match.

Zero-fingerprint antigens are NOT scan-invisible by accident — they are scan-invisible BY
DESIGN, because their detection-model does not include source-structure scanning.

### Enforcement

- `AntigenArgs` parse (`parse.rs:202-209`): relax fingerprint from required `String` to
  `Option<String>`. Error path for missing-fingerprint removed. Scan layer: short-circuit
  early when `fingerprint.is_none()`.
- Stdlib update: supply-chain + VCS-info-loss antigens drop placeholder `fingerprint` fields.
- `cargo antigen scan` output: verify-only antigens produce zero scan presentations (correct,
  not a regression).
- Future antigens that are purely verify-only MUST omit fingerprint (no placeholder fingerprints).
  New antigens with a source-structure surface MUST include fingerprint.

### Resolves

- ~14,792 spurious scan presentations from placeholder fingerprints on verify-only antigens
- The forced-lie surface: `#[antigen]` no longer forces any antigen to claim scan-locatability
  it doesn't have
- Names the detection-model axis (scan-locatable / verify-only / both) explicitly, elevating
  an implicit assumption (ADR-004)

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

## ADR-010 Amendment 6 — Three-Valued Predicate Evaluation (Match3)

**Status**: Ratified 2026-05-28.

**Amends**: ADR-010 Amendment 3 (`match_constraint` return type) and Amendment 4 (sub-item
domain extension prerequisite).

**Participants**: antigen-dx-dogfood team (scientist primary drafter; aristotle Phase-1-8
PASSED with strengthening clarification on two-level semantics; naturalist biology PASSED
with primary-source grounding; adversarial PASSED with all four ATK-FP tests traced through
Kleene-strong algebra; all four ceremony signers).

**Related campsites**: `forward/fingerprint-grammar-body-content-with-negation`;
`ceremony/ratify-adr-010-amendment-6-match3`.

### Problem

The fingerprint matcher's predicate evaluators return `bool`, conflating three distinct
meanings into two values:

- **Match** — predicate evaluated, condition present
- **NoMatch** — predicate evaluated, condition absent
- **Undefined** — predicate has no locus on this item-class (no body, no fields — the
  question is malformed here)

`body_contains_macro` (`matcher.rs:244`) returns `false` for non-fn/non-impl items via the
`_ => {}` arm at `matcher.rs:282`. When an adopter writes:

```
all_of([item = struct, not(body_contains_macro("panic!"))])
```

the evaluation is: `body_contains_macro("panic!") = false` (no body-locus, returns false),
`not(false) = true`, `all_of(Match, true) = Match`. Every struct matches, vacuously.

This is the adversarial ATK-FP-NOT-BODY-VACUOUS finding (committed `1b843d1`). The adopter
intended "structs without `panic!`" but gets all structs — the fingerprint fires everywhere
it was meant to filter.

**Why this is a type-level defect, not a documentation burden**: the `bool` type cannot
distinguish "no body here, predicate undefined" from "body searched, condition absent." `not()`
of an undefined-as-false produces vacuously-true. Any new sub-item predicate
(`field_attr_present`, `body_contains_call`) inherits this hazard on every item-class outside
its domain. Documenting the limitation per-predicate multiplies the hazard rather than
fixing it.

**The three-antigen convergence** (recognition-not-design criterion met, ADR-006): three
independent real-world fingerprint failures hit the same property — fingerprint grammar
reaching only top-level item structure — `#19 ScanVisitorDigestAssignmentOmission` (body
ordering constraint), `#22 SerdeDefaultMaskingStructLiteralBreak` (field-level attribute
unreachable), and the body-negation campsite (caller vs definer direction). Subject-2
(definedness semantics) must be fixed first; Subject-1 (domain coverage) rides on top
safely.

### Decision

**Match3: three-valued predicate evaluation.** Replace `bool` return types in all predicate
evaluators with `Match3`:

```rust
pub enum Match3 {
    Match,
    NoMatch,
    Undefined,  // predicate has no locus on this item-class
}
```

The `_ => {}` arm in `body_contains_macro` (`matcher.rs:282`) returns `Match3::Undefined`
instead of falling through to `false`. No separate domain-annotation is needed — the
evaluator already knows whether the locus exists (it is the `_ => {}` arm). The domain is
implicit in which item-classes the predicate's match arm visits.

**Match3 composition — two levels.** There are two distinct semantic levels (aristotle
Phase-1-8 PASSED, strengthening clarification 2026-05-27). They MUST NOT be conflated in
implementation or documentation, or future grammar extensions will accidentally collapse
`Undefined` into `NoMatch` in the leaf algebra.

*Level 1: Leaf-algebra (Kleene-strong three-valued logic).* Predicates and combinators
operate on `Match3` throughout:

```
not:   Match -> NoMatch
       NoMatch -> Match
       Undefined -> Undefined   (NOT true — kills vacuous-not at the type level)

and (all_of):
       Match ∧ Match = Match
       any NoMatch present => NoMatch (short-circuit on definite failure)
       else if any Undefined present => Undefined (preserves the definedness gap)

or (any_of):
       NoMatch ∨ NoMatch = NoMatch
       any Match present => Match (short-circuit on definite success)
       else if any Undefined present => Undefined
```

The key invariant: `Undefined` propagates through combinators. It does NOT collapse to
`NoMatch` inside `all_of`. This preserves the type-level distinction for callers reading
intermediate results (future audit-hints, debug output, "fingerprint X was undefined on N
items — domain mismatch?" advisories).

*Level 2: Fingerprint-fires projection (user-facing).* The fingerprint fires IFF the
top-level expression evaluates to `Match`. Both `NoMatch` and `Undefined` at the top level
project to "doesn't fire":

- `Match` → fingerprint fires (item flagged)
- `NoMatch` → fingerprint doesn't fire
- `Undefined` → fingerprint doesn't fire

At the audit level, the only question is whether the antigen flags this item. An item where
the fingerprint is undefined (e.g., a struct asked a body-content question) should not be
flagged — the predicate has no locus here. The semantic distinction is preserved at Level 1
for tooling; it is projected away at Level 2 for the user.

*Why the two-level distinction matters*: the Level-2 wording "Undefined-as-non-matching for
`all_of`" — which is correct at the fingerprint-fires level — must NOT be read as "the
`all_of` operator returns `NoMatch` when fed `Undefined`" at Level 1. That reading would
destroy the type-level distinction Match3 was built to preserve. The Kleene-strong leaf
algebra keeps `Undefined` propagating through combinators; the projection step at the top
evaluates "fires? only if Match."

**Impact on four ATK-FP tests** (adversarial gate, all four traced through Match3,
PASSED 2026-05-28):

1. **ATK-FP-NOT-BODY-VACUOUS** (struct + not(body_contains_macro(X))): `body_contains_macro`
   on struct = `Undefined`; `not(Undefined) = Undefined`; `all_of` evaluates to `Undefined`
   at leaf-algebra; fingerprint-fires projection: doesn't fire. Test currently asserts
   matches=true (broken-behavior pin). **Test MUST BE INVERTED** to assert matches=false
   after this amendment lands.
2. **ATK-FP-NOT-BODY-FN-CORRECT** (fn + not(body_contains_macro(X)), X absent):
   `body_contains_macro` on fn returns real `NoMatch`; `not(NoMatch) = Match`; fingerprint
   fires. Asserts matches=true; **unchanged**.
3. **not() directly under any_of rejected**: parse-time rule, unchanged by Match3.
   **Unchanged**.
4. **all_of containing only nots rejected**: parse-time rule, unchanged by Match3.
   **Unchanged**.

Match3 closes the vacuous-not hazard at the type level.

### Biology (PMID 11238607, primary-source grounded)

Three-valued predicate evaluation is biology-confirmed (naturalist gate PASSED 2026-05-27).
The assay-on-wrong-tissue cognate is exact and pubmed-grounded.

**Core principle**: absence of evidence is not evidence of absence when the evidence could
not have been generated. A negative result is only meaningful when the assay COULD have
returned positive. If the assay isn't validated for the substrate — wrong tissue, wrong
matrix, wrong question-shape — the result is UNDEFINED, not negative. Clinical immunology
has a dedicated category for this: indeterminate / not-evaluable / invalid / preanalytical
failure.

**Canonical instance — window-period HIV testing**: a negative HIV antibody test in the
seroconversion window (first ~2–4 weeks post-exposure, before antibody titer crosses
detection threshold) is NOT HIV-negative — it is window-period indeterminate. Treating
`not(positive) = negative` led to real transfusion-transmitted HIV before the indeterminate
category was clinically enforced. The clinical world enforces `not(Undefined) = Undefined`
under penalty of patient deaths.

**Receptor assay on wrong-tissue/wrong-matrix**: flow cytometry for surface markers on a
sample with no viable cells → INDETERMINATE, not negative. PCR for tissue-specific gene
expression run on the wrong tissue → "not detected" but interpreted as NON-INFORMATIVE, not
absence-of-gene.

**The antigen analog**: `body_contains_macro` applied to a struct (no body-locus) is
precisely "a receptor assay run on a substrate where the assay's preconditions don't hold."
The predicate is asking "does the function body of this item contain macro X," applied to
an item that has no function body. The question itself is malformed in this context.
Clinical immunology's answer: return UNDEFINED, not vacuous-false. Treating it as false is
the window-period-as-negative error in DSL form — same shape of harm.

**`not(Undefined) = Undefined` is required, not optional**: in clinical immunology, you
cannot negate an indeterminate result into a definite one. The negation of "we couldn't tell
whether HIV is present" is NOT "HIV is absent" — it's "we still couldn't tell." The
three-valued logic is closed under negation; collapsing to two-valued at the boolean-output
stage re-introduces the assay-on-wrong-tissue bug. So Match/NoMatch/Undefined with
`not(Undefined) = Undefined` throughout the eval is biology-required, not just structurally
tidy.

### Sub-item domain extension (Subject-1 — v0.3, gated by this amendment)

Three new predicate leaves are proposed for v0.3 once Match3 lands:

- `body_contains_call(path)`: caller leaf — any fn/impl body that calls the named path.
  Domain: fn/impl. Returns `Undefined` on bodyless items via `_ => Undefined` arm (safe by
  construction under Match3).
- `body_contains_call_before(a, b)`: ordering — call to `a` precedes call to `b` in the
  same body. Domain: fn/impl. `Undefined` on bodyless.
- `field_attr_present(path)`: field-level attribute — struct/enum field bears named attr.
  Domain: struct/enum with fields. `Undefined` on fn/impl/trait.

Each new predicate declares its domain implicitly via the `_ => Undefined` arm. Match3 makes
them safe by construction — without Match3, each would inherit the vacuous-not hazard on
every item-class outside its domain. **Subject-2 (Match3) gates Subject-1 (domain
extension)**: without the three-valued fix, extending the domain multiplies the hazard
across every new sub-item predicate. The order is Match3 first, then domain-extension lands
safely on top.

### Implementation path

1. `match_constraint()` in `matcher.rs`: return type `bool -> Match3`.
2. `body_contains_macro()`, `body_contains_macro_in_single_impl()`, `finder_fn()`: return
   `Match3` (the `_ => {}` arm returns `Match3::Undefined`).
3. `not()` combinator: `not(Undefined) = Undefined` (the type-level fix).
4. `all_of()` / `any_of()` combinators: Kleene-strong leaf-algebra (Undefined propagates,
   does NOT collapse to NoMatch).
5. Fingerprint-fires projection at `scan.rs` / `audit.rs`: fires iff `Match3::Match`; both
   `NoMatch` and `Undefined` produce "doesn't fire."
6. Propagate through `matches_item()` and `matches()`.
7. **Invert ATK-FP-NOT-BODY-VACUOUS test** (test 1): change assertion from matches=true to
   matches=false.

### Gate outcomes (ceremony complete)

**Aristotle Phase-1-8 — PASSED** (2026-05-27). Resolved: Match3 cardinality is correct
(three values for the definedness subject; Error is orthogonal infrastructure-layer concern;
PartialMatch is a different subject coverage-axis); `all_of` Kleene-strong leaf-algebra +
fingerprint-fires projection (two-level distinction explicit); `not(Undefined) = Undefined`
(root fix at the type level); Subject-2 (Match3) gates Subject-1 (domain extension); ADR-010
Amendment 6 territory (not a new ADR). Strengthening clarification (Kleene-strong two-level
semantics naming) folded into §Match3 composition.

**Naturalist (biology) — PASSED** (2026-05-27). Assay-on-wrong-tissue cognate confirmed,
pubmed-grounded (PMID 11238607; indeterminate-Western-blot clinical category). Three-valued
logic with `not(Undefined) = Undefined` is biology-required. Window-period HIV testing as
canonical instance.

**Adversarial — PASSED** (2026-05-28). All four ATK-FP tests traced through Kleene-strong
algebra; all confirmed closed. Test 1 inverts (broken behavior becomes correct assertion);
tests 2/3/4 unaffected.

**Scientist (consistency) — PASSED** (2026-05-28).

### What this amendment does NOT do

- Does NOT change the fingerprint grammar DSL syntax (only the evaluator internals and the
  fingerprint-fires projection).
- Does NOT introduce sub-item predicates yet — those are v0.3 follow-on work gated by this
  amendment.
- Does NOT change parse-time rejection rules (e.g., not() under any_of) — those are
  unchanged.

### Evidence citations

- ATK-FP-NOT-BODY-VACUOUS: adversarial finding committed `1b843d1` (four tests pinning
  the broken behavior)
- Substrate verification: `matcher.rs:244-285`, `body_contains_macro -> bool`, `_ => {}` arm
  at `matcher.rs:282`
- Three-antigen convergence: #19 ScanVisitorDigestAssignmentOmission, #22
  SerdeDefaultMaskingStructLiteralBreak, body-negation campsite
- PMID 11238607: Ludewig/Zinkernagel — rapid peptide turnover limits clone activation
  (predicate preconditions never crossed threshold = Undefined)
- Window-period HIV testing: clinical indeterminate category (standardized; transfusion-
  transmitted HIV before category enforced)
- Aristotle Phase-1-8: `forward/fingerprint-grammar-body-content-with-negation` campsite
  (2026-05-27); Kleene-strong two-level semantics clarification
- Naturalist biology gate: same campsite (2026-05-27); assay-on-wrong-tissue cognate
- Recognition-not-design: ADR-006 (3 independent real failures = grammar investment earns)

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

**Status**: Ratified 2026-05-08. **Same-workspace implementation landed v0.3**
(the `#[antigen_generates]` macro + scan generates-synthesis pass;
`antigen/tests/atk_antigen_generates.rs`). Cross-crate macro-output recognition
(§A4) remains deferred pending the cross-crate antigen-discovery mechanism.

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
- **Sweep A3 / v0.3 (DONE)** wires the synthesis pass for same-workspace: the
  `#[antigen_generates]` macro emits a discoverable `antigen:generates:v1:<X>`
  marker; the scan parses the source attribute into a `GeneratesDeclaration`,
  builds a `macro_name → [antigen_type]` index, and synthesizes presentations
  at matching `#[derive]` / attribute-macro / bang-macro invocation sites.
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

## ADR-017 Amendment 1 — Cross-crate `addresses()` resolution: the sub-clause-F clause at the cross-crate reference boundary

**Status**: Proposed 2026-06-01 (campsite: `infra/multi-crate-scan`; Type-A, design-first
§Mechanics-level amendment).

**Amends**: ADR-017 (antigen identity = canonical declaration site; cross-crate trust delegates
to cargo).

**Participants**: aristotle (the cross-crate ruling — `infra/multi-crate-scan` note `669d59de`;
the sub-clause-F clause); pathmaker (Layer-1 multi-crate scan substrate `e53f91d`; the Layer-2
scope + the §Decision-vs-§Mechanics question that this answers — §Mechanics → Type-A); navigator
(blocking note: the ruling must become a formal amendment block before pathmaker can sign).

**Related**:
- ADR-001 Amendment 1 Change 3 C7 (cross-crate consumption commitment + scanner-activation status —
  this amendment formalizes the trust-boundary clause C7 deferred to ADR-005's enforcement clauses).
- ADR-005 (sub-clause F at every trust boundary — cross-crate consumption is named boundary item 4;
  this amendment specifies the validation check for the cross-crate `addresses()` boundary).
- ADR-029 Amendment 1 (well-posedness: a verdict over an un-evaluable state is out-of-frame).
- `forward/three-valued-logic-api-boundary-layer` (the gem: an unresolvable cross-crate reference
  is the third value — out-of-frame, distinct from resolved-and-undefended).
- ADR-033 (the prescriptive cross-need references, VOID-4b, are the SAME structural shape — the
  resolution machinery this amendment specifies is reused there in v0.4).

**Campsite**: `infra/multi-crate-scan` (the ruling note `669d59de` is the witness).

### Finding

ADR-017 ratified cross-crate antigen identity (Approach 3-revised: scanner-derived
`canonical_path`) and stated cross-crate trust-boundary mechanics "defer to ADR-005's enforcement
clauses." ADR-001 C7 named the scanner-activation of cross-crate `addresses()` matching as
tracked-not-decided. Pathmaker's multi-crate scan Layer 1 (`e53f91d`) shipped the substrate
(member-aware scan, distinct `canonical_path` stamping per member, cross-member lineage). Layer 2 —
cross-crate `addresses()`/`defended_by` matching over the merged member-report (== ATK-A3-005,
closes `DelegateCrossCrateResolutionGap`) — is an **implementation-activation of already-ratified
architecture**, NOT a new architectural decision: the identity primitive is ratified (ADR-017
canonical_path), the commitment is ratified (ADR-001 C7), the trust delegates to cargo (ADR-017
§Decision). It requires exactly **one** clause to be specified: the sub-clause-F validation check at
the moment a cross-crate claim is honored.

### Decision

When a `defended_by` / `presents` in crate B addresses an antigen declared in crate A, the audit
issues a cross-crate verdict only under this sub-clause-F validation check:

1. **Resolution gate (the third value).** A cross-crate defense/presents claim is HONORED only when
   its `canonical_path` resolves to a real declaration in a scanned member or dependency. If it does
   NOT resolve, the verdict is **out-of-frame**, NOT **undefended**. An unresolvable cross-crate
   reference is the three-valued-logic third value (un-evaluable), categorically distinct from
   resolved-and-undefended. This is the asymmetric default: an unresolved cross-crate reference is a
   **loud GAP** (out-of-frame), never a silent pass and never a silent failure.
2. **Canonical-path-keyed trust (no same-name cross-satisfy).** Cross-crate trust is recorded
   `canonical_path`-keyed (name@version), so a same-type-name collision across crates
   (`crate_a::PanickingInDrop` vs `crate_b::PanickingInDrop`) does NOT silently cross-satisfy. This
   is the exact ADR-017 identity finding applied to the matching step: the bare-name-overclaim guard
   (`canonical_paths_match`: `None ≠ Some`, `Some(x) == Some(x)`) already does the right thing once
   members are stamped distinctly — this amendment makes the requirement explicit at the verdict
   boundary.

These are the only two clauses Layer 2 adds beyond the ratified architecture. With them on the
record, pathmaker's Layer-2 implementation has its trust-boundary check ratified before the code
signs.

### Why this is a Type-A §Mechanics amendment, not a new ADR

- **Identity is already ratified** (ADR-017 Approach 3-revised). The match keys on the existing
  canonical_path primitive — no new identity decision.
- **The commitment is already ratified** (ADR-001 C7). Layer 2 is C7's scanner-activation; the
  deferral was a SCOPE choice, not an open architectural question.
- **Trust already delegates to cargo** (ADR-017 §Decision). Doing the match invents no new trust
  mechanism.

The only genuinely new content is the sub-clause-F clause above — a §Mechanics-level specification of
the validation check ADR-005 requires at this boundary, which is precisely what a Type-A amendment
captures.

### What this amendment does NOT do

- Does NOT change ADR-017's identity model (canonical_path stays the key).
- Does NOT add cross-crate fingerprint *synthesis* (that re-runs synthesis over the merged whole — a
  separate, deliberately-deferred concern; Layer 1 already avoids double-counting intra-member
  synthesis).
- Does NOT resolve re-export witness chains (ADR-017 open-question 4 — out of scope, a
  witness-resolution problem, not an identity problem).
- Does NOT make `--workspace` the default scan mode (opt-in; flat scan stays default for
  backward-compatible output).

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

## ADR-018 Amendment 1 — Inheritance is provenance, not substitutability

**Status**: Ratified 2026-05-20.

**Amends**: ADR-018 §Decision "The descendant inherits the ancestor's match_kind.
Inheritance is provenance, not match-kind."

**Reason**: Observer NB017 (F24 peer review) surfaced a precision gap in the
ratified text: "not match-kind" correctly names what inheritance does NOT preserve
from a match-semantics perspective, but does not name the stronger claim that
`descended_from` makes no substitutability guarantee. ADR-024 (F24 aristotle
Phase 1-8 on substitutability vs provenance) established this as a load-bearing
distinction that future implementers and consumers need to read explicitly.

**Change**: Extend the Decision text from:

> "The descendant inherits the ancestor's `match_kind`. Inheritance is provenance,
> not match-kind."

to:

> "The descendant inherits the ancestor's `match_kind`. Inheritance is provenance,
> not match-kind **and not substitutability**. `#[descended_from(X)]` records that
> the declaring type is structurally related to X's failure-class by lineage; it
> does NOT assert that the descendant satisfies X's immunity witness, that the
> descendant's context is semantically equivalent to X's, or that the descendant
> can be substituted for X in any behavioral sense. Substitutability is the subject
> of A4-A5 behavioral-tier work (open question 5 above); A3's job is to surface the
> inheritance state in a form A4-A5 can consume."

**Resolves**: F24 (aristotle Phase 1-8 on substitutability vs provenance) finding
that the ratified text left implicit the non-substitutability guarantee. Explicit
statement prevents implementers from inferring substitutability from `descended_from`
declarations.

---

## ADR-026 Amendment 1 — rollback-as-triage uses `#[triage_commit]`, not `#[orient]` extension

**Status**: Ratified 2026-05-24.

**Amends**: ADR-026 §Decision (the sentence "rollback-as-triage discipline (via new `#[triage_commit]` primitive)") and §Sweep-level consequences bullet 2.

**Reason**: The orient-dual-signature analysis (campsite `fixup-orient-dual-signature`) established that `#[orient]` is passive-context-only: it annotates code with contextual understanding but carries no decisional or committal semantics. Rollback-as-triage is a DECISIONAL act (triage decision, rollback target, rationale, due-within) that requires a structurally distinct primitive. Extending `#[orient]` with decisional fields would violate ADR-023's semantic boundary. The `#[triage_commit]` macro (shipped in commit 94f088d) implements the five required fields.

**Change**: The §Decision text originally read (at ratification):

> "rollback-as-triage discipline (via extended `#[orient]` primitive)"

Amended to:

> "rollback-as-triage discipline (via new `#[triage_commit]` primitive)"

The inline code example was changed from `#[orient(...)]` to `#[triage_commit(...)]` with five REQUIRED fields: `triage_decision`, `rollback_target`, `triaged_by`, `rationale` (≥ 20 chars), `rollback_due_within_minutes` (> 0). §Sweep-level consequences bullet 2 updated to read "New `#[triage_commit]` primitive carries rollback-as-triage fields (Amendment 1: `#[orient]` NOT extended)".

**Resolves**: Orient-dual-signature campsite finding that decisional/committal fields require structural distinction from passive-context `#[orient]`. ADR-023 §orient semantics preserved unchanged.

---

## ADR-026 Amendment 2 — TriageDecision variant-semantic backfill + camp::triage connection-claim discipline

**Status**: Ratified 2026-05-24.

**Amends**: ADR-026 §Decision (Schema additions paragraph — backfills variant semantics into the ratified contract); ADR-026 §Sweep-level consequences (camp::triage connection-claim loosening from STRUCTURAL claim to conceptual-alignment-now / structural-v0.3+).

**Reason**: Outsider's connection-claim discipline analysis (`820a710a`, `fd7ff496`, `b095f9c3`) surfaced two related issues: (1) ADR-026 §Decision named `TriageDecision` as an enum (`Black | Red | Yellow | Green | White`) but did not define the variant semantics — those were documented only in `antigen/src/vcs.rs` doc-comments, creating a documentation-tier inversion where the implementation is more rigorous than the ratified contract; (2) the §Sweep-level "Connection to camp `triage` primitive" line was a STRUCTURAL claim with decorative-tier delivery — no cross-tool schema commitment exists today, and the honest state is "conceptual alignment now; structural alignment v0.3+." Per scientist's framing and outsider's resolve recommendation (b1 with v0.3 commitment named).

**Change 1 — TriageDecision variant semantics backfill (`fd7ff496`)**: The §Schema additions paragraph is extended with explicit variant semantics so the ADR itself is the authoritative source. The five variants carry the following semantics in the rollback-as-triage software-engineering use-case (analogous to but not identical with clinical field-triage protocols — see Amendment 2 Change 3 below on the connection-claim tier):

- `Black` — system-down / data-loss imminent / catastrophic regression confirmed; rollback is the immediate action.
- `Red` — vital-metric regression confirmed; rollback within tight time window (typical `rollback_due_within_minutes` ≤ 30).
- `Yellow` — concerning signal but not vital-metric-blocking; investigation pending; rollback decision deferred.
- `Green` — no functional regression detected; the `#[triage_commit]` carries the analysis chain attesting non-regression.
- `White` — out of scope for this triage event (e.g., the change is unrelated to the suspected regression); explicit non-action chart entry. **`White` is software-engineering-introduced** — clinical START protocols ship as 4-color (Black/Red/Yellow/Green); the 5th variant is added for the rollback-as-triage use-case where explicit-non-action chart entries carry their own audit value.

**Change 2 — camp::triage connection-claim loosening (`b095f9c3`)**: The §Sweep-level consequences bullet "Connection to camp `triage` primitive" is loosened from a structural-claim phrasing to:

> "Conceptual alignment with camp `triage` primitive — both classify state into 5-color taxonomy with rollback/treatment-discipline semantics. Cross-tool schema alignment (shared `TriageDecision` type across antigen + camp) is deferred to v0.3+ research arc; no schema commitment is made in v0.2."

This honors outsider's resolve recommendation (b1 with v0.3 commitment named) and avoids the documentation-tier inversion of claiming structural alignment that does not yet exist.

**Change 3 — START attribution loosening (`a87e4245`)**: The `#[triage_commit]` doc-comment and the `TriageDecision` doc-comment in `antigen/src/vcs.rs` previously claimed "Modeled on the START field-triage protocol" (IDENTITY-tier connection-claim per outsider's discipline taxonomy). Loosened to "Color-tagged analogously to clinical field-triage protocols (e.g., START — Simple Triage And Rapid Treatment, US emergency-medicine standard since 1983)" — RHYME-tier connection-claim. The substantive structural rhyme is preserved (5-color schema + treatment-decision discipline); the IDENTITY overclaim is removed (clinical START doesn't have `White`; the protocol's diagnostic protocol per se is not what `#[triage_commit]` implements). The corresponding doc-comment edits ship in the same commit as this amendment.

**Resolves**: Connection-claim discipline gaps surfaced by outsider's audit during the v02-completion-arc expedition. Adopters reading ADR-026 in isolation now have authoritative variant semantics in the ratified contract; the camp::triage connection-claim accurately reflects current state (conceptual alignment) and named future commitment (v0.3+ structural alignment); the START attribution preserves the substantive rhyme without IDENTITY overclaim.

---

## ADR-026 Amendment 3 — rollback detection algorithm (AUTHOR-DECLARATION) + structural enforcement verification requirement

**Status**: Ratified 2026-05-24.

**Amends**: ADR-026 §Finding (Detection model paragraph) and §Finding (Enforcement model paragraph); adds two audit hints to §Decision audit-hint vocabulary.

**Reason**: Adversarial ATK-VCS-1 (rollback detection algorithm not specified) and ATK-VCS-4 (structural enforcement verification not required at audit-time) identified held-implementation-spec-depth-gap instances. Aristotle Phase 1-8 (campsite `v02-impl-vcs-info-loss`) established the correct resolution for both. The ADR previously stated detection MUST be at commit-time (§D1) without specifying HOW the hook recognizes a rollback commit. The enforcement model accepted Structural mode declarations without verifying the remote configuration via substrate-witness.

**Change 1 — Rollback detection algorithm (ATK-VCS-1)**: Specifies AUTHOR-DECLARATION (Algorithm C) as the detection algorithm. The commit-time hook applies a three-step decision tree: (1) git-revert metadata present + no `Triage-Decision:` trailer → fire `RollbackWithoutTriageCommit`; (2) commit carries a `Triage-Decision: <value>` trailer → validate value is a `TriageDecision` enum variant; fire `vcs-rollback-triage-chain` witness check *(Amendment 4 corrects step 2 from codebase-presence to commit-trailer signal)*; (3) otherwise → audit defers. Residual risk from manual inverse cherry-picks (undetectable without author declaration) is NAMED and EXPLICIT per friction-only philosophy. Diff-similarity detection (Algorithm B) is opt-in via `cargo antigen vcs --diff-similarity-check` (v0.3+ experimental).

**Change 2 — Structural enforcement verification (ATK-VCS-4)**: When `ServerSideEnforcementMode::Structural` is declared, the audit pipeline MUST evaluate `vcs_server_side_enforcement_active(repo, antigen_name)` at audit-time. False return → emit `vcs-enforcement-structural-mode-declared-but-not-active` + demote to FrictionOnly for that antigen. Network error during evaluation → emit `vcs-server-config-check-failed` (distinct from structural-not-active). This witness is v0.2.1+ alongside Structural mode; v0.2 ships friction-only only.

**Change 3 — Two new audit hints**: `vcs-enforcement-structural-mode-declared-but-not-active` (emitted when Structural declared but remote config not verified) and `vcs-server-config-check-failed` (emitted on network error during structural verification). Total audit-hint count: 12 → 14.

**Resolves**: ATK-VCS-1 (rollback detection algorithm gap) and ATK-VCS-4 (structural enforcement unverified claim gap) from adversarial pre-attack pass on `v02-impl-vcs-info-loss` campsite. Observer's network-dependent-witness tier concern (1bb4f0c7) addressed by the two-error-mode split.

**What this amendment does NOT do**: Does not add a `vcs-rollback-coverage-partial` hint for step-3 commits (ATK-VCS-A1 partial residual — see adversarial campsite note). The hint would fire on every commit that falls through to step 3, making the partial-coverage zone visible at audit-time per ADR-005 Amendment 3 tier-honesty discipline. Deferred to v0.2.x alongside `install-hooks`/`install-server-hooks` CLI verbs, which provide the infrastructure through which the hint would be emitted.

---

## ADR-026 Amendment 4 — Rollback detection step-2 signal: commit-trailer not codebase-presence

**Status**: Ratified 2026-05-24.

**Amends**: ADR-026 Amendment 3 Change 1 §Decision-tree step 2 (inline parenthetical at §Finding Detection model paragraph).

**Reason**: ATK-VCS-A2 (adversarial post-ratification finding on Amendment 3, campsite `adr026-amendment-4-step2-commit-trailer`): Amendment 3 step 2 specified "commit declares `#[triage_commit]`" as the detection signal. This is codebase-presence semantics — does the codebase contain the attribute anywhere, not does THIS commit declare a triage decision. Three failure modes: (a) rollback commit on a codebase that already has `#[triage_commit]` elsewhere fires step 2 by coincidence; (b) rollback commit on a codebase with zero `#[triage_commit]` declarations falls silently to step 3; (c) two identical rollback commits on different codebases receive different audit behavior. The structural mechanism for commit-intent is already specified in ADR-026 §M3: the `Triage-Decision: <value>` git trailer on the commit itself.

**Change — Step-2 signal corrected**: Step 2 of the AUTHOR-DECLARATION decision tree is: commit carries a `Triage-Decision: <value>` trailer → validate value is a `TriageDecision` enum variant; fire `vcs-rollback-triage-chain` witness check. This is commit-intent semantics (does THIS commit declare its triage decision?), consistent with ADR-026 §M3 trailer schema.

**Scope**: This correction affects the `install-hooks` and `install-server-hooks` CLI verbs (which write the pre-commit/pre-push hook that executes the decision tree). It does NOT affect the witness evaluators (`vcs_trailer_present`, `vcs_rollback_triage_chain`, `vcs_attest_branch_deletion`, `vcs_server_side_enforcement_active`) or the observation-side CLI verbs (`scan`, `check-commit`, `attest`, `rollback-prepare`, `branch-archive`). Per aristotle F1 ratification (campsite `adr026-amendment-4-step2-commit-trailer`): witness layer + observation CLI proceed in parallel; hook-installation verbs defer until this amendment ratifies.

**Participants**: adversarial (ATK-VCS-A2 finding); aristotle (Phase 1-8 ratification of two-layer separation principle — witness independence from decision-tree text); scientist (Amendment 4 draft + inline correction).

**Related**: ADR-026 Amendment 3 Change 1 (the amended text); ADR-026 §M3 (`Triage-Decision:` trailer schema); ADR-010 (witness independence from scan-layer decisions); ADR-019 (substrate-witness reads substrate, not ADR text).

---

## ADR-027 Amendment 1 — Mucosal taxonomy disambiguation + delegate-kind-matching + tolerance primitive

**Status**: Ratified 2026-05-24.

**Amends**: ADR-027.

**Reason**: The v02-completion-arc team surfaced six spec-depth gaps in
ADR-027 before pathmaker could implement the mucosal family. The gaps are
independent in origin (naturalist biology-prediction, scout adversarial-test,
outsider naive-question) but converge on the same root: ADR-027 §Decision
specifies WHAT the mucosal primitives are without specifying HOW their
argument shape, variant inclusion, and audit logic resolve at impl time.
This amendment closes the gaps to unblock implementation. Filed as instance
(v) of held-implementation-spec-depth-gap pattern; absorbed into
`docs/process.md` §Standing Adversarial Checklist.

**Participants**: aristotle (Phase 1-8 deconstruction F1+F2); naturalist
(biology grounding refinement; `#[mucosal_tolerant]` primitive; amendment
text draft); scout (ATK-MUCOSAL-3 adversarial test, `c7ae5990` handled_by
typing); outsider (`1bb7c7b6` enum chaos dust-finding); adversarial
(ATK-MUCOSAL-3 pre-implementation test encoding the spec gap).

**Related**: ADR-001 Amendment 1 C6 (sealed enum amendment process);
ADR-003 Amendment 1 (biology grounding); ADR-016 (tolerance discipline);
ADR-019 (substrate-witness); ADR-028 (antigen-category);
`docs/process.md` §Standing Adversarial Checklist (sibling workstream).

### Change 1 — MucosalKind inclusion-discipline named

**Finding**: ADR-027 §Decision ships a sealed 15-variant MucosalKind enum
(per ADR-001 Amendment 1 C6) but does not state the inclusion criterion.
Adopters proposing new variants (e.g., 'WebhookEndpoint',
'PluginEntryPoint') have no decision rule; ADR amendment process is named
but amendment-criteria are not.

**Decision**: The MucosalKind axis is **type-of-data-crossing-boundary**.
A variant belongs in MucosalKind iff:

(a) it names a kind of data/control flow crossing a trust boundary at
runtime;
(b) the data-flow type is meaningfully distinct from other variants in
terms of sanitization vocabulary (no isomorphism with an existing variant);
(c) the boundary surfaces in a way `#[mucosal]` can attach to (a
function/method receiving the data).

Process events (PR creation, CI hook firing, code-review approval) are NOT
data-flow types — they are software-engineering-process events that occur
AROUND the boundary, not AT it. These belong in a sibling axis if needed
(deferred to v0.3+ research).

### Change 2 — PrBoundary removed from MucosalKind

**Finding**: 'PrBoundary' fails the Change 1 criterion: it is a process
event, not a runtime data-flow. `#[mucosal(kind = PrBoundary)]` has no
defined attachment site. Inclusion is incoherent.

**Decision**: Remove `PrBoundary` from the sealed v0.2 MucosalKind set.
PR-boundary concerns are addressed at the data-flow level through existing
variants (a PR-arrival webhook carries `ApiRequest` + `UserInput`; the PR
body content carries `UserInput`; etc).

### Change 3 — Import vs DependencyImport disambiguation

**Finding**: ADR-027 §Decision lists both `Import` and `DependencyImport`
as variants without disambiguation. 'Import' reads as `use foo::bar`
(in-language symbol import); 'DependencyImport' as `cargo dep`
(supply-chain dependency intake). The ADR is silent on the distinction.

**Decision**: `Import` is REMOVED from the v0.2 sealed set as
redundant/ambiguous. `DependencyImport` remains; its meaning is the
cargo-dep-intake boundary (data flow: 3rd-party crate code → workspace,
per ADR-025 supply-chain family). In-language `use` statements do NOT
cross trust boundaries in Rust's model.

After Changes 2 and 3, the v0.2 sealed MucosalKind set becomes **13
variants**: `ApiRequest, ApiResponse, McpInvocation, ExternalLink, Iframe,
DatabaseQuery, CrossService, SubprocessLaunch, DependencyImport,
UserInput, FilesystemPath, EnvironmentVariable, ShellArgument`.

### Change 4 — handled_by typed as syn::Path

**Finding**: ADR-027 §Decision says `handled_by = "..."` (string) but
does not specify the string format. Scout flagged (`c7ae5990`); aristotle
confirmed. Free-form string allows typos to silently produce broken
delegations.

**Decision**: `handled_by` parses as a Rust path expression (`syn::Path`),
not a string literal. Syntax:

```rust
#[mucosal_delegate(
    boundary = MucosalKind::UserInput,
    handled_by = crate::sanitize::user_input_sanitizer,
    rationale = "...",
)]
```

Path resolution at audit-time follows standard Rust visibility +
module-graph rules.

### Change 5 — Delegate kind-matching semantics

**Finding** (scout ATK-MUCOSAL-3): 'corresponding declaration' in
§Decision is ambiguous. Reading A (any `#[mucosal]` on target suffices)
is an attack surface: `#[mucosal_delegate(boundary = UserInput,
handled_by = sanitize_db)]` passes when `sanitize_db` carries only
`#[mucosal(kind = DatabaseQuery)]`.

**Decision**: Kind-matching semantics (Reading B) required. Audit logic
per `#[mucosal_delegate]`:

(a) Resolve `handled_by` path. If not found: emit `mucosal-discipline-delegate-target-missing`.
(b) If path resolves but no `#[mucosal]` present: emit `mucosal-discipline-delegate-target-not-mucosal`.
(c) If `#[mucosal]` present but no `kind = X` matches delegate's boundary kind: emit `mucosal-discipline-delegate-target-kind-mismatch` (NEW).

The three hints form a precise three-tier diagnosis: missing-handler →
handler-undefended → handler-wrong-defense. Hybrid handlers (one function
carrying multiple `#[mucosal(kind = X)]` declarations) satisfy
kind-matching via set-membership (NOT exact-equality).

### Change 6 — #[mucosal_tolerant] primitive added

**Finding** (naturalist `fab2b234`): biology distinguishes THREE response
states at mucosal sites — active defense, active tolerance (Tregs /
oral tolerance), and undecided. ADR-027 ships primitives for state 1 only;
states 2 and 3 are both treated as 'undefended'. Adopters with intentional
unauthenticated endpoints have no honest declaration option.

**Decision**: Ship `#[mucosal_tolerant]` in v0.2 alongside `#[mucosal]`
and `#[mucosal_delegate]`.

```rust
#[mucosal_tolerant(
    kind = MucosalKind::...,
    rationale = "<≥40-char string>",
    accepts = "<description of what passes through>",
    reviewed_by = "<role-or-name>",   // optional v0.2; required v0.2.1+
    until = "<RFC-3339 date>",         // optional; review-deadline
)]
```

Field requirements: `kind` REQUIRED; `rationale` REQUIRED ≥40 chars
(higher than `#[mucosal]`'s ≥20 — tolerance is the riskier declaration;
per ADR-005 Amendment 2 risk-proportionate length floors); `accepts`
REQUIRED non-empty; `reviewed_by` OPTIONAL v0.2; `until` OPTIONAL
RFC-3339 date.

`mucosal-map --undefended` EXCLUDES boundaries carrying `#[mucosal_tolerant]`.
New sub-flag: `mucosal-map --tolerant` lists all tolerance declarations.

Biology: three active states with distinct cellular substrates (active
tolerance = Treg-mediated antigen-specific suppression via tolerogenic
DCs — NOT absence of response). Parallel to ADR-016 `#[antigen_tolerance]`
at boundary tier rather than failure-class tier.

### Change 7 — Audit-hint vocabulary refresh

The full §Audit-hint vocabulary for the mucosal family becomes 11 hints:
`mucosal-boundary-undefended`, `mucosal-kind-mismatch`,
`mucosal-rationale-insufficient`, `mucosal-discipline-delegated`,
`mucosal-discipline-delegate-target-missing`,
`mucosal-discipline-delegate-target-not-mucosal`,
`mucosal-discipline-delegate-target-kind-mismatch` *(NEW)*,
`mucosal-tolerant-rationale-insufficient` *(NEW)*,
`mucosal-tolerant-past-review-date` *(NEW)*,
`mucosal-tolerant-accepts-empty` *(NEW)*,
`mucosal-tolerant-without-reviewer` *(NEW; v0.2.1+)*.

### Mechanics

- Sealed MucosalKind set: 15 → 13 variants (PrBoundary + Import removed).
- New primitive `#[mucosal_tolerant]` alongside `#[mucosal]` and `#[mucosal_delegate]`.
- `handled_by` typed as `syn::Path`, not string literal.
- §Audit logic per `#[mucosal_delegate]` gains three-tier diagnosis (Change 5).
- §Audit-hint vocabulary gains 5 new hints (Changes 5+6+7).

### Resolves

- Six spec-depth gaps surfaced by v02-completion-arc team (Changes 1-6).
- The 'corresponding declaration' ambiguity enabling silent miscategorization defenses.
- The 'no way to declare intentional tolerance' friction for adopters with public-intake endpoints.

### What this amendment does NOT do

- Does NOT introduce a sibling axis for process-event boundaries (PrBoundary class). Deferred v0.3+.
- Does NOT ship the three additional metaphor-predicted primitives from naturalist `dd2732e8` (M-cell, goblet, tolerance-breakdown). Deferred v0.3+.
- Does NOT make `reviewed_by` required in v0.2; that lands v0.2.1+ with a migration hint.
- Does NOT mechanically verify 'accepts' content matches actual runtime boundary behavior. Deferred v0.3+.

---

## ADR-028 Amendment 2 — predicate-leaf requirement applies to witness layer, not fingerprint scan-side

**Status**: Ratified 2026-05-24.

**Amends**: ADR-028 §Decision enforcement bullet: "`category = SubstrateAlignment` requires at least one substrate-witness predicate leaf".

**Reason**: Observer audit of the supply-chain category backfill (campsite `adr028-predicate-leaf-clarification`) surfaced an ambiguity: does "substrate-witness predicate leaf" apply to the FINGERPRINT scan-side pattern, or to the WITNESS evaluator? The two are structurally different. Fingerprint patterns locate declaration sites; witnesses evaluate substrate state at audit time. Requiring the fingerprint itself to be a substrate-witness leaf would prohibit valid scan-side patterns like `doc_contains("ADR-025")` that correctly locate antigen declarations but do not themselves read substrate state. Team-lead confirmed Interpretation 2: the requirement applies to the witness layer.

**Change**: The enforcement bullet previously read (implicitly):

> "`category = SubstrateAlignment` requires at least one substrate-witness predicate leaf [in the fingerprint]"

Clarified (inline annotation at §Decision, line 5780) to:

> The "substrate-witness predicate leaf" requirement applies to the WITNESS layer — either an audit-pipeline evaluator that reads substrate state directly (e.g., `DepPinnedState`, `ContentHashState`), or a fingerprint using substrate-witness leaves from ADR-019's grammar. It does NOT require the fingerprint scan-side pattern itself to be a substrate-witness leaf. Fingerprint finds the declaration sites; witness evaluates substrate state.

**Enforcement gap named** (per aristotle R3 on `adr028-predicate-leaf-clarification`): parse-time enforcement of the substrate-witness leaf requirement depends on the category-vs-predicate-type cross-check (ADR-028 §F1-R Hybrid miscategorization defense), tracked in campsite `v02-impl-category-witness-cross-check`. Until that cross-check ships, the requirement is advisory at parse-time and enforced only at audit-time. This is an honest disclosure per ADR-005 sub-clause F — the enforcement chain is traceable even where parse-time enforcement is not yet implemented.

**Resolves**: Ambiguity that would have required `doc_contains("ADR-025")` to be replaced with a substrate-witness leaf at scan time — an overconstrained requirement that conflates fingerprint location semantics with witness evaluation semantics.

---

## ADR-028 Amendment 3 — category-vs-predicate-type cross-check is audit-time, not parse-time

**Status**: Ratified 2026-05-24.

**Amends**: ADR-028 §Enforcement (G2 deliverable): "G2: category-vs-predicate-type cross-check."

**Reason**: The G2 campsite spec originally implied a parse-time check — `AntigenArgs::validate()` reading the category field and enforcing witness-type consistency at macro-expand time. aristotle's F1 finding on `v02-impl-category-witness-cross-check` identified the structural blocker: a single `#[antigen]` macro cannot see the `#[immune]` declarations that address it. Those declarations are on different items, potentially in different files. The antigen-immunity join only exists once `cargo antigen scan` assembles the full `ScanReport`. A parse-time check cannot be implemented without an inversion of the macro execution model.

**Change**: G2 is an AUDIT-TIME cross-check in `audit_category()`, not a parse-time check in `AntigenArgs::validate()`. Implementation: `audit_category()` joins each explicit-category `AntigenDeclaration` against every `Immunity` where `immunity.antigen_type == decl.type_name`. Witness type is read structurally from the immunity: `requires_predicate.is_some()` identifies a substrate-witness; a non-empty `witness` field identifies a code-witness. The mismatch hint (`AntigenCategoryClaimInconsistentWithPredicateType`) is advisory at audit time and CI-gateable per Amendment 2. Zero immunities is not flagged (orthogonal coverage gap).

This is consistent with Amendment 2's principle: the witness-layer requirement is evaluated at audit time where full scan context is available, not at parse time where only the single item is visible.

**Resolves**: The campsite `v02-impl-category-witness-cross-check` enforcement gap named in Amendment 2. The cross-check is now implemented at the correct architectural layer (commit `114af45`).

---

## ADR-028 Amendment 4 — enforcement-surface re-sync post G1/G2/G3

**Status**: Ratified 2026-05-24.

**Amends**: ADR-028 §Enforcement-Surface table + §Decision backward-compat paragraph + §Audit-hint vocabulary.

**Reason**: The G1→G2→G3 implementation arc (campsites `v02-impl-category-v01-discriminator`, `v02-impl-category-witness-cross-check`, `v02-impl-category-audit-hints`) shipped with two deliberate differences from the original ADR-028 text:
1. G1 ships the v0.1-carryover default as a **migration hint** (`antigen-category-defaulted-implicit-functional`), not a parse-time hard error. Hard error for NEW declarations is v0.2.x scope once the migration tool exists.
2. G2 ships the category-vs-witness-type cross-check at **audit-time only** (Amendment 3 records the structural reason). The §Enforcement-Surface table still said "parse-time (hint) + audit-time."
3. G3 ships two of the four named hints; the other two are deferred to v0.2.x with named blocking reasons. The audit-hint vocabulary paragraph listed all four without tiering.

**Change**:

*§Enforcement-Surface table* (lines 6195–6200 in decisions.md at Amendment-4 time) — replace rows with:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Status |
|---|---|---|---|
| `category` field on `#[antigen]` (v0.2+) | audit-time: migration hint for v0.1 carryover; hard error for new declarations (v0.2.x) | client | v0.2 ships hint; hard error deferred pending migration tool |
| Category-vs-witness-type cross-check | audit-time (ADVISORY; CI-gateable) | client + CI | v0.2 shipped (Amendment 3: structural reason audit-time is correct layer) |
| Hybrid incomplete-evidence | audit-time (partial coverage signal) | client + CI | v0.2 shipped |
| v0.1 backward-compat default | parse-time migration hint | client | v0.2 ships; v0.3+ deprecation removes default |

*§Audit-hint vocabulary* — tiered as shipped:

- **v0.2 (shipped)**: `antigen-category-defaulted-implicit-functional` (G1), `antigen-category-claim-inconsistent-with-predicate-type` (G2), `antigen-category-hybrid-incomplete-evidence` (G3).
- **v0.2.x (deferred — named blocking reasons)**:
  - `antigen-category-missing-explicit` — requires the v0.1/v0.2 migration-record discriminator (deferred in G1 aristotle F1 ruling).
  - `antigen-category-mismatch-witness-type` — advisory soft-smell layer atop claim-inconsistent; distinct softer signal; lands after claim-inconsistent proves in the field.

*§Decision backward-compat paragraph* inline annotation — remove the phrase "v0.2+ NEW declarations: `category` REQUIRED at parse-time; absence is hard-error" and replace with "v0.2+ NEW declarations: absence emits `antigen-category-defaulted-implicit-functional` migration hint; parse-time hard error is v0.2.x once migration tooling exists."

**Resolves**: Stale enforcement-surface table and hint vocabulary that described the original design rather than the shipped v0.2 state. Aristotle's finding on `v02-impl-category-audit-hints` campsite.

---

## ADR-028 Amendment 5 — encounter-status axis (vaccinated / encountered / affinity-matured)

**Status**: Ratified — navigator + naturalist signed (2/2). F7 per-leaf diagnostics gate cleared 2026-05-26. Tekgy confirmed team ratification authority.

**Amends**: ADR-028 §Decision (category metadata surface) + §Audit-hint vocabulary. Cross-references ADR-003 Amendment 1 (biology-as-discovery), ADR-006 Amendment 1 / ADR-022 (prospective vs retrospective growth discipline), ADR-016 (anergy / tolerance), ADR-023 (deferred-defense family).

**Reason**: The category-metadata surface (ADR-028) describes failure-class properties but omits a dimension the immune-system metaphor requires: whether a fail-class has been ENCOUNTERED in the wild yet. This distinction is already implicit in two ratified places — ADR-006-Am1/ADR-022 split stdlib growth into prospective (research-declared, no live instance required) and retrospective (adopter-extension, triggered by real encounter) — but it is purely policy-level, with no per-antigen typed field. The project's own preemptive-bias discipline (feedback memory `feedback_internal_tool_antigens_preemptive.md`) and the dogfood stdlib already contain vaccinated-state antigens (declared from shape-prediction, no live instance), making the implicit policy visible. Recognition-not-design (ADR-006): the encounter-status axis already exists as implicit policy + biology prediction; this amendment makes it EXPLICIT as typed metadata.

Biology grounding (naturalist ruling, 2026-05-26): The vaccination cognate (`cargo antigen vaccinate` verb, glossary:43) already encodes the "weakened antigen for advance memory" concept. A fail-class declared from forward-compat reasoning IS the germline-naive / vaccinated state. Anergy (ADR-016) presupposes encounter and is RESPONSE-POSTURE on a separate axis, not a fourth encounter-state. Biology is silent on how to witness a vaccinated antigen (no live input), which predicts the hard sub-clause-F constraint that Vaccinated antigens may not claim Behavioral witnesses.

**Change**:

*§Decision — add encounter_status field to antigen declaration:*

- Add `encounter_status: EncounterStatus` as an optional field on `#[antigen(...)].` Default: `Vaccinated` for research-stdlib antigens (the common case for preemptive declarations); explicit-required for adopter-extension antigens (where the encounter is the trigger per ADR-022).
- Sealed enum `EncounterStatus`: `Vaccinated` (declared from research/shape-prediction; no live instance), `Encountered` (first instance seen in production), `AffinityMatured` (witness refined post-encounter; `#[descended_from]` lineage typically present).
- Sub-clause F invariant 1 (parse-time hard): a `Vaccinated` antigen **MAY NOT** claim a `Behavioral` witness (`EvidenceKind::Behavioral`). No live input exists to behave on. Biology-mandated; violation is a structural contradiction.
- Sub-clause F invariant 2 (parse-time): response-posture attributes (`#[anergy]`, `#[immunosuppress]`, `#[poxparty]`, `#[orient]`) from the deferred-defense family (ADR-023) are only valid when `encounter_status != Vaccinated`. You cannot defer-respond to something never encountered.
- The deferred-defense family of four (ADR-023) is RECOGNIZED as the response-posture axis, gated on encounter. This amendment does NOT redefine them — it names their structural relationship to encounter-status as an axis constraint.

*§Audit-hint vocabulary — add encounter-axis hints:*

- `antigen-encounter-vaccinated-behavioral-witness` — emitted when `encounter_status = Vaccinated` AND a Behavioral witness is claimed. Hard block (sub-clause F invariant 1).
- `antigen-encounter-vaccinated-deferred-posture` — emitted when `encounter_status = Vaccinated` AND a deferred-defense macro is applied. Hard block (sub-clause F invariant 2).
- `antigen-encounter-status-coordinate` — informational; emitted in audit output as `[vaccinated|encountered|matured] × [None|Reachability|Execution|FormalProof]` so the full two-axis coordinate is visible. Not a gate; enables informed triage.

*Anergy cross-link (explicit naming):* `anergic = encountered + unresponsive-without-co-stimulation (ADR-016 tolerance)`. Anergy is receptor-level unresponsiveness — non-volitional and reversible, not deliberate suppression (that is `#[immunosuppress]`). This is a (EncounterStatus::Encountered, response_posture::Anergic) coordinate in the product space, not a separate encounter state. The cross-link from encounter-status to ADR-016 is now named rather than implicit.

**Resolves**: aristotle finding F6 (antigen-dx-dogfood expedition, 2026-05-26). The three-axis state-space — encounter-status × witness-tier × response-posture (posture gated on encounter) — was always implicit in the ratified surface; this amendment makes it explicit and sub-clause-F enforced. Naturalist biology-confirmed: silence on Behavioral-witness-for-vaccinated IS the constraint made structural; three-axis product-not-chain resolves the VecCardinalityMasqueradingAsSet error in aristotle's prior skeleton.

---

## ADR-028 Amendment 6 — tier marker (object | description): relationship to own declaration

**Status**: Ratified 2026-05-26. Ceremony: aristotle absorption → naturalist biology-check → adversarial gate → scientist validation (full four-stage ceremony; team ratification authority per generalized governance decision). Participants: aristotle (draft + stage-3 absorption of adversarial orthogonality finding), naturalist (biology-check + silence-derivation idiotype-network grounding), adversarial (three-attack gate; orthogonality partial snag absorbed by aristotle), scientist (stage-4 validation: falsifiable prediction confirmed by substrate survey; orthogonality edge cases probed; enforcement coherence confirmed).

**Amends**: ADR-028 §Decision (category metadata surface) + §Audit-hint vocabulary. Cross-references ADR-006 (recognition-not-design), ADR-010 (fingerprint matching), ADR-019 B1 (recognition-vs-evidence role distinction), and the antigen-dx-dogfood F10 fundamentality-test + the idiotype-network cognate (immune-system-primitive-map.md, naturalist). Sibling to Amendment 5 (encounter-status axis — relationship to the WILD). Separate ratification unit per aristotle's ruling: different subjects, bundling would be VecCardinalityMasqueradingAsSet at the amendment level.

**Reason**: The category-metadata surface (ADR-028) classifies a fail-class by WHAT it gets wrong (SubstrateAlignment vs FunctionalCorrectness) but omits a dimension the F10 fundamentality-test surfaced: a class's relationship TO ITS OWN DECLARATION. Some fail-classes are *about* the relation between a description and its referent (spec-vs-impl, fingerprint-vs-class-extension, declared-intent-vs-realized) — and for exactly those, the class's own declaration is itself a description-with-a-referent, so the declaration CAN exhibit the class. This "self-reach" is observable and was observed repeatedly during the dx-dogfood expedition (`RatifiedSpecDriftFromImpl` fired on its own docs; the `AntigenFingerprint` antigen's name committed its own error; the glossary entry would have committed the failure it documents). Recognition-not-design (ADR-006): the distinction already exists structurally and is observable; this amendment makes it EXPLICIT as typed metadata + a ratification-lane check.

**Change**:

*§Decision — add tier field to antigen declaration:*

- Add `tier: AntigenTier` as an optional field on `#[antigen(...)]`. Sealed enum `AntigenTier`: `Object` (the class's subject is a runtime/value property — a verb produces wrong output, a value violates an invariant; the declaration lives OUTSIDE the class's domain and cannot self-reach) and `Description` (the class's subject IS the description↔referent relation; the declaration is IN the domain and can exhibit the class).
- **Tier is recognition-lane-determined, author-recorded** (the structural difference from category + encounter-status, both purely author-set). Determination rule: `tier = Description` iff the SELF-REACH check passes — some artifact that declares the class (spec, name, fingerprint, glossary entry, the taxonomy row) can itself exhibit the class. Else `tier = Object`. Determined by the RECOGNITION LANE at ratification time; then recorded in the declaration by the author under that determination. Default (when omitted): `Object` — the conservative default, since self-reach is the positive diagnostic that must be affirmatively recognized.
- **Orthogonal to category** (sub-clause: tier ⊥ category). Both directions of the orthogonality are substrate-verified, NOT theoretical:
  - **FunctionalCorrectness + Description-tier exists**: `silent-intent-nullification` (verified dogfood.rs) is FunctionalCorrectness yet self-reaches — its DSL can omit the capability it names.
  - **SubstrateAlignment + Object-tier exists** — six counterexamples found in the stdlib by aristotle (stage-3 absorption) and independently confirmed by scientist (stage-4): `delegated-handler-kind-mismatch` (about `#[mucosal_delegate]` declarations, not antigen declarations), `scanner-boundary-false-negative` (about trust-boundary heuristics, not declarations), `unstable-hash-as-persisted-value` (about persisted-hash code, not declarations), `parallel-state-trackers-diverge` (about runtime trackers, not declarations), `agent-wake-without-substrate-delta-injection` (about agent-wake protocol, not declarations — subject involves context-vs-substrate gap but self-reach fails operationally because the declaration cannot "resume" and "fail to read a delta"), `delegate-cross-crate-resolution-gap` (about cross-crate resolution, not declarations). All SubstrateAlignment; none self-reach; all Object-tier.
  - **The load-bearing discriminator**: a class is Description-tier iff *the class's SUBJECT is the description↔referent relation*. Every class has a name (a descriptive artifact), but only a subset are *about* description-referent relations. The discriminator is the SUBJECT, not the artifacts involved. SubstrateAlignment classes whose subject is a specific code/protocol shape (boundaries, hashes, trackers, wake-protocols) are Object-tier; SubstrateAlignment classes whose subject is declarations/specs/fingerprints/biology-claims drifting from their referents are Description-tier. Category answers "what does it get wrong" (representation-vs-state vs verb-correctness); tier answers "is the class itself about description↔referent." Independent axes, both directions substrate-populated.
- **Self-reach is a NETWORK/TAXONOMY property, not single-declaration** (naturalist's idiotype-network biology, F10-(c)): the per-antigen tier marker (b) is the LOCAL SHADOW of the taxonomy-completeness-pressure (c). Biology basis: recognition-of-recognition exists at the NETWORK level (anti-idiotype antibodies bind *another* cell's receptor) but biology is SILENT on reflexive single-molecule self-binding — no antibody binds its own idiotope. That silence is instrument-not-argument: it LOCATES self-reach at the network/taxonomy tier rather than the single-declaration tier. A class reaches its own declaration only across the network (the taxonomy) that holds both class and declaration, never as a single declaration binding itself. So the per-antigen marker (b) is necessarily the local shadow of the network-level property (c). Part (c) is a standing pressure on stdlib growth (comprehensive-vision research-driven coverage), not a per-antigen field — named here as the biology grounding.

*§Audit-hint vocabulary — add tier hints (ADVISORY in v0.2):*

- `antigen-tier-description-self-reach-unchecked` — informational; emitted when a `Description`-tier antigen's OWN declaration artifacts (name, fingerprint, summary, glossary entry) have not been checked for self-exhibition. Prompts the recognition-lane self-check (does the name commit its own error? does the fingerprint under/over-cover? does the glossary entry assert an undeclared type?). NOT a gate in v0.2.
- `antigen-tier-coordinate` — informational; emitted as `[object|description]` so the tier is visible in audit output. Combines with Amendment-5 encounter-status coordinate and witness-tier for the full metadata picture.

**Enforcement-model honesty**: tier's enforcement differs structurally from category (parse-time author-assertion) and encounter-status (parse-time, defaulted). Tier is recognition-lane-determined; its v0.2 enforcement is ADVISORY because the self-reach check is a manual recognition-lane determination at ratification time, not a mechanized scan. Mechanizing it (a scan that checks whether a Description-tier class's own declaration artifacts exhibit the class) is itself `AntigenFingerprintDivergesFromClassExtension` territory and is deferred to v0.2.x — it requires comparing a class's fingerprint match-set against its own declaration. v0.2 ships: the tier FIELD + the advisory hints + the recognition-lane derivation discipline. v0.2 transparency gap acknowledged: no hint distinguishes "recognition lane checked and confirmed Object" from "defaulted without checking" — `antigen-tier-object-self-reach-unverified` is named as the v0.2.x upgrade companion to the mechanized self-reach checker. The mechanized checker is the v0.2.x upgrade path.

**With category + encounter-status, the `#[antigen]` metadata surface has three orthogonal axes**:
- **category**: what the class gets wrong (SubstrateAlignment | FunctionalCorrectness)
- **encounter_status**: relationship to the wild (Vaccinated | Encountered | AffinityMatured)
- **tier**: relationship to own declaration (Object | Description)

The product space, not a chain. Each axis is independent; all four corners of the category × tier subspace are substrate-populated.

**Resolves**: aristotle finding F10 part (b) (antigen-dx-dogfood, 2026-05-26). Part (a) (the fundamentality-test as a ratification heuristic) is a recognition-lane practice, not a typed surface — it is the DISCIPLINE that determines this field. Part (c) (taxonomy-completeness-pressure) is the network-level form, grounded in the idiotype-network cognate; named here as the biology grounding for why (b) is a local shadow. Sibling heuristic: `description-tier-grows-by-witness-split` (naturalist, dogfood/description-tier-grows-by-witness-split) — description-tier classes tend to grow as parent + witness-mechanism-split children rather than flat standalones; confirmed by scientist's stage-4 substrate survey (Fingerprint sibling pair is cleanest instance; Object-tier classes all flat).

---

## ADR-028 Amendment 7 — silence-generator witness-selection guidance + witness-locus split within SubstrateAlignment

**Status**: Ratified 2026-05-27.

**Amends**: ADR-028 §Decision. Adds (a) `§Witness-selection-guidance` subsection and (b) `§SubstrateAlignment witness-locus discriminator` subsection.

**Participants**: naturalist (silence-generator section draft; campsite `forward/silence-taxonomy-substrate-alignment`); aristotle (witness-locus split Phase-1-8 deconstruction; campsite `findings/category-witness-crosscheck-vs-fingerprint-only-stdlib`); scientist (consistency review: both additions validated; 2x2 ceiling prediction confirmed; convergence finding verified); outsider (convergence note: recursive irony — `ParallelStateTrackersDiverge` is itself a `ParallelStateTrackersDiverge` instance across the category/witness split, the medicine demonstrating the disease in the diagnosing organ); 4 independent roles converged on the witness-locus gap within ~24 hours of expedition work (recurrence-anchor threshold).

**Related**: ADR-028 Amendment 2 (substrate-witness predicate leaf requirement at the witness layer); ADR-028 Amendment 3 (category-vs-predicate-type cross-check); ADR-006 (recognition-not-design); ADR-019 B1 (recognition-vs-evidence role distinction); ADR-004 (implicit-to-explicit); ADR-029 (observed-not-declared: silence-generator guidance and ADR-029 are the same structural principle viewed from opposite ends — ADR-029 applies it to immune-state *verdicts*, this amendment applies it to witness-*selection*; naturalist synthesis 2026-05-27); `findings/category-witness-crosscheck-vs-fingerprint-only-stdlib`.

**Two-part reason:**

**Part A (silence-generator)**: ADR-028 made SubstrateAlignment vs FunctionalCorrectness a first-class category because the category predicts the witness *layer*. The antigen-dx-dogfood expedition surfaced a finer recurring structure *inside* SubstrateAlignment: these antigens differ by their **silence-generator** — the specific mechanism by which the failure fails to announce itself. Four generators recurred across the stdlib + dogfood antigens, each biologically distinct, and each implying a different witness shape. The silence-generator is therefore witness-selection guidance: name how the failure stays silent, and the witness that breaks the silence follows. This is recognition (ADR-006): the four generators were read off existing antigens, not invented. **Structural unity with ADR-029** (naturalist synthesis, 2026-05-27): ADR-029's observed-not-declared principle and this silence-generator guidance are the same structure seen from opposite ends — ADR-029 applies "do not declare a verdict the tool cannot observe" to immune-state (the *output* side); silence-generator guidance applies "name the mechanism that prevents observation" to witness-selection (the *input* side). They are the same discipline: where silence lives, what breaks it, and who gets to say so.

**Part B (witness-locus split)**: ADR-028 Amendment 2's "substrate-witness reads substrate state" correctly covers external-substrate-divergence (representation vs external sidecar/git/doc) but silently excludes in-repo-parity-divergence (two code artifacts in the same repo that must agree). `ParallelStateTrackersDiverge` is SubstrateAlignment by failure-KIND (representation-diverges-from-state), yet its correct witness is a bijection/parity code-test (not a substrate-predicate), because the "state" is an in-repo enum rather than external substrate. G2's cross-check faithfully implements Amendment 2 — the gap is the ADR's model, not G2's code. The fix: name the discriminator within SubstrateAlignment explicitly. Recognition-not-design (ADR-006): the distinction already exists in the substrate (the witness type itself carries the locus information); this amendment makes it explicit in the category model.

**Aristotelian convergence** (aristotle, `findings/category-witness-crosscheck-vs-fingerprint-only-stdlib`): three deconstructions this expedition cycle converged on one structural signature — *a typed model collapses a distinction the substrate already carries*: (1) the `witness` word in ADR-029-V2 collapses runtime-witness vs audit-time-predicate; (2) `EvalNode::passed():bool` collapses Passed/Failed/Indeterminate (e58627d5 fix); (3) `category=SubstrateAlignment` collapses external-substrate-divergence vs in-repo-parity-divergence. This is `VecCardinalityMasqueradingAsSet` at the model-type layer — the recurring shape where the typed verdict/category/word is coarser than the substrate distinction it claims to represent.

### Change A: §Witness-selection-guidance subsection

*Added to ADR-028 §Decision, after the category/encounter-status/tier three-axis summary:*

**Silence-generator: witness-selection guidance for SubstrateAlignment antigens**

SubstrateAlignment failures are, by definition, representations diverging from the state they model *without announcing the divergence*. Within SubstrateAlignment, the **silence-generator** classifies how the non-announcement happens. Naming the generator points directly at the witness shape required to break the silence.

**This is guidance, not a new declared field.** The silence-generator is the author's diagnostic question ("how does this failure stay silent?") whose answer points at a witness. No `#[antigen(silence = …)]` field is added — the guidance is operational. (Anti-YAGNI note: a stored field would need a consumer; the guidance has its consumer today — the antigen author choosing a witness.)

Four generators have cleared the recurrence threshold (each observed in multiple stdlib + dogfood antigens; biology cognates verified by naturalist):

**1. Silence-by-absence** — no enforcement mechanism exists; a comment substitutes for one. Nothing fires when representations drift because nothing was ever wired to fire.

- *Canonical instance*: `ParallelStateTrackersDiverge` — a "MUST match exactly" comment shadowing an enum's serde keys; the comment promises synchronization but enforces none.
- *Biology cognate*: no immune memory was ever formed for the class; the system has no receptor for this divergence.
- *Witness to reach for*: **parity / bijection test** that reads *both* representations and asserts they agree.
- *Critical refinement*: the absence-generator's true signature is NOT "two representations disagree at this instant" — that would false-positive on every legitimate deferred state (a pre-ratification ADR draft, a migration in progress, a CI parity test scheduled for next release). The fail-class is *gap + no-mechanism*: a divergence where the only thing promising closure is prose that schedules nothing. The absence-witness must assert *the closure-mechanism exists*, not merely that the two sides happen to agree right now.

**2. Silence-by-masking** — an enforcement-substitute *actively manufactures* a plausible-but-wrong value; the failure never reaches a detection surface. Detection = 0, but "all clear" signal emitted.

- *Canonical instance*: `SerdeDefaultMaskingStructLiteralBreak` — `#[serde(default)]` makes stale fixture JSON deserialize successfully into a default-filled value; the missing field never registers as a problem.
- *Biology cognate*: active immune suppression — regulatory T-cells, IL-10 / TGF-β, checkpoint inhibition. The danger signal is prevented from propagating.
- *Witness to reach for*: **reject-the-default-at-the-boundary** — assert that the construction/deserialization path cannot manufacture a plausible value for a slot that should have been explicitly migrated. Pair with `#[non_exhaustive]` for external-crate construction sites.

**3. Silence-by-missing-diagnostic** — the wrong value *is* produced and surfaces loudly downstream, but no per-leaf message traces the failure back to the boundary that introduced it. Detection = 1; localization = 0.

- *Canonical instance*: `SilentSemanticMismatchAtTrustBoundary` (dogfood #14), F7 meta-instance — `DisciplinePredicateFailed` fires at the tree level with no per-leaf expected/found, so the adopter cannot tell which boundary passed the wrong value through.
- *Biology cognate*: the signal fires (inflammation is present) but is not localized — the system knows something is wrong without knowing where.
- *Witness to reach for*: **per-leaf diagnostic emit** — surface expected-vs-found at each leaf so the failure is named where it was introduced.

**4. Silence-by-wrong-weighting** — the failure *is* detected and *is* surfaced, but at the wrong confidence / priority / urgency. The distinction that should change the downstream response is collapsed. Detection = 1; confidence-metadata = wrong.

- *Canonical instance*: `unaddressed_presentations()` treating a `FingerprintMatch` at equal urgency to an `ExplicitMarker` — the low-specificity inferred match and the high-specificity declared marker land in the same bucket, collapsing the `PresentationSource` gradient.
- *Biology cognate*: autoimmunity / cytokine-storm. The two-signal confidence-graded model (Matzinger danger model) deliberately down-weights innate detection (low-specificity, broad) relative to adaptive detection (affinity-matured, high-specificity). Collapsing that gradient — acting on a low-specificity match as if it were confirmed-specific — is the autoimmune pathology. `FingerprintMatch` IS the innate/germline-recall surface; `ExplicitMarker` IS the adaptive/affinity-matured declaration; the `PresentationSource` enum IS the innate/adaptive gradient, and the bug collapses it.
- *Witness to reach for*: **confidence-discrimination test** — assert the model outputs *distinct* values for distinct confidence tiers, not just that detection occurred.

**Summary table:**

| Silence-generator | What's broken | Biology cognate | Witness to reach for |
|---|---|---|---|
| absence | no mechanism; comment substitutes | no immune memory formed | parity / bijection test |
| masking | substitute manufactures plausible-wrong value (detection=0) | active suppression (Treg, IL-10) | reject-default-at-boundary + `#[non_exhaustive]` |
| missing-diagnostic | wrong value loud downstream, untraced (localization=0) | signal fires but unlocalized | per-leaf expected/found emit |
| wrong-weighting | detected but confidence gradient collapsed | autoimmunity (innate ≠ adaptive, collapsed) | confidence-discrimination test |

The four generators cleared the recurrence threshold as of 2026-05-27. A fifth may surface; amend this section when it does. The 2x2 structure (detection-survives? × confidence-correct?) provides a falsifiable ceiling: a 5th generator would require a third boolean axis (scientist, 2026-05-27).

### Change B: §SubstrateAlignment witness-locus discriminator

*Added to ADR-028 §Decision, as a subsection of the SubstrateAlignment category description:*

**SubstrateAlignment witness-locus: external-substrate vs in-repo-parity**

ADR-028 Amendment 2 established that SubstrateAlignment antigens should carry a substrate-witness predicate leaf. This is correct for **external-substrate-divergence** — where a code representation diverges from external state (sidecar, Cargo.lock, git history, remote registry). The substrate-predicate reads that external state.

It is *incomplete* for **in-repo-parity-divergence** — where two code artifacts in the same repository (a const, an enum, a doc-string, a test fixture) must agree, and the failure is their disagreement. For these, the defense is a **bijection/parity code-test** that reads both in-repo artifacts and asserts they agree. No substrate-predicate can be written for two same-crate artifacts — there is no external substrate to check.

Both are SubstrateAlignment by failure-KIND (representation-diverges-from-state). The discriminator is **witness-locus**:

| Witness-locus | Divergence type | Correct witness form | Example antigen |
|---|---|---|---|
| external-substrate | code representation vs external state (sidecar / Cargo.lock / git) | substrate-predicate (`requires =` on `#[presents]`) | `UnpinnedDependency` |
| in-repo-parity | two in-repo artifacts that must agree | bijection/parity code-test (`#[defended_by(X)]` on a test fn) | `ParallelStateTrackersDiverge` |

**G2 cross-check behavior**: G2 (`antigen-category-mismatch-witness-type`) fires when a SubstrateAlignment antigen carries a code-witness. This correctly catches external-substrate-divergence defended by a code-test (wrong). It SHOULD NOT fire for in-repo-parity-divergence defended by a bijection/parity code-test (correct, per this amendment). G2 should accept a code-witness as satisfying SubstrateAlignment WHEN the divergence is in-repo-parity.

**Near-term implementation** (v0.2.x): G2's cross-check accepts a code-witness for SubstrateAlignment antigens that carry an explicit in-repo-parity rationale (via `requires =` absent AND a declared `#[defended_by]` with a parity-test naming convention, or a new `witness_locus = in_repo_parity` advisory hint). Until mechanized: `ParallelStateTrackersDiverge` and `BiologyGroundingClaimDrift` carry known-advisory-flag comments documenting this as the correct in-repo-parity witness pattern (per navigator sequencing decision, `findings/category-witness-crosscheck-vs-fingerprint-only-stdlib`).

**Recursive irony** (outsider, 2026-05-27): `ParallelStateTrackersDiverge` (#18) is itself a `ParallelStateTrackersDiverge` instance across the category/witness split — `category=SubstrateAlignment` (the two-tracker claim in the declared model) diverges from `witness=code-bijection-test` (the evidence the implementation actually uses to defend it). The antigen about the failure-class is the poster-child for the gap in the rule that classifies it.

### Resolves

- `forward/silence-taxonomy-substrate-alignment` (absorbed; campsite closes as "absorbed-into-ADR-028-Amendment-7")
- `findings/category-witness-crosscheck-vs-fingerprint-only-stdlib` (witness-locus discriminator named; G2 behavior specified)
- The advisory-stopgap comments on `ParallelStateTrackersDiverge` and `BiologyGroundingClaimDrift` in dogfood.rs (the amendment formally ratifies what those comments named as pending)
- The implicit assumption that all SubstrateAlignment antigens require external-substrate predicates (witness-locus split makes the assumption explicit and corrects the over-constraint)

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

## [ADR-019] Substrate-witness predicate family

**Status**: Ratified 2026-05-19.

**Participants**: pathmaker (draft v0 + implementation + P3e test corpus), aristotle
(F10-F19 Phase 1-8 arc, 16 findings), adversarial (ATK-019 Stages 1-3, all pre-named
attack surfaces + FA-1/FA-3/FA-6 resolutions), naturalist (F3 scope-biology, F8
EvidenceKind biology, framing-call, notary-arc B6, biology corroboration audit),
observer (NB001-NB024, peer-review arc), scout (S1-S4, cross-domain pre-scouting),
scientist (Stage 5 validation + prose-polish + whitepaper chapter), academic-researcher
(Stage 4 prior-art alignment, 8 checks), navigator (coordinator, architectural call,
Stage 4 + Stage 5 runs), Tekgy (trust).

**Related**:
- ADR-001 (structural memory = this ADR's application domain)
- ADR-002 (compose-don't-compete; substrate-witness composes existing tooling)
- ADR-003 (biology metaphor; B1-B8 grounding sections)
- ADR-005 Amendment 2 (rationale-as-required-field; transverse sub-clause F)
- ADR-005 Amendment 3 (WitnessTier × AuditHint; this ADR adds EvidenceKind as third axis)
- ADR-006 (recognition-not-design; discipline-witnesses recognized from three-independent-instance evidence)
- ADR-007 (anti-YAGNI; all 5 sealed leaf primitives ship v0.1 — structurally guaranteed)
- ADR-009 (adoption gradient; Layer 1 works without sidecar infrastructure)
- ADR-011 (tolerance-ratification gap this ADR plugs via isomorphic schema)
- ADR-017 (antigen identity; `AntigenIdentifier` semantic name not Rust path)
- ADR-018 (`descended_from`; `weakened_from` + `weakening_rationale` schema fields for predicate-weakening)

### Finding

Antigen's witness vocabulary (ADR-001 + ADR-002 + ADR-013) covers code-side substrate
exclusively: `Test / IgnoredTest / Proptest / Function / PhantomType`. Each verifies
the code itself via syntactic recognition or compile-time proof. A structural gap exists
for **discipline failure-classes** — where the antigen is presented at a code site but
the carrier of immunity is substrate *other than the code being audited*: a ratified
discipline doc, team sign-off record, oracle-completion marker, signed git trailer.

This gap is not hypothetical:

1. **Direct adoption signal**: tambear independently sketched `witness = doc_attested(...)`
   from three antibody-tier methodology patterns crystallized in one day
   (2026-05-18) — independent design convergence with three architectural differences
   (tooling friction, PR review visibility, code-drift defense).
2. **ADR-011 open question**: `#[antigen_tolerance(X)]` without structured attestation
   was named as an open question in ADR-011's own text. No resolution primitive existed.
3. **F17 transverse principle**: six independent gap-instances share a unifying structure
   (every attestation must carry an explicit WHY-record the audit can surface), named
   as "rationale-as-visibility."

The substrate-witness reframe: witnesses currently check code-side substrate. They should
be extensible to check **substrate other than the code being audited** — as long as the
audit remains tier-honest about what was actually verified.

### Decision

**Antigen ships a substrate-witness predicate family in v0.1-rc**: a closed declarative
predicate language over typed substrate (predicate combinators + sealed leaf primitives),
a ratification schema (`antigen-attestation` crate; serde-derived), JSON sidecars
co-located with source under `.attest/` subfolders, and `cargo antigen attest` +
`cargo antigen tolerate` CLI families.

This is ONE ADR introducing ONE primitive — the substrate-witness predicate — covering
both immunity claims and tolerance ratifications via isomorphic schema.

The decision commits to:

**Three-axis tier-honest reporting** (extends ADR-005 Amendment 3): `WitnessTier ×
AuditHint × EvidenceKind`. `EvidenceKind` is a **parallel axis, NOT an ordered scale**
(FA-6 resolution per ATK-019 Stage 3): `TypeSystemProof | Behavioral | SubstrateState`
are different *types* of evidence, not different *strengths*. Per-kind ceilings:
`TypeSystemProof` → `FormalProof`; `Behavioral` → `Execution`; `SubstrateState` →
`Execution` (cannot reach `FormalProof`). `EvidenceKind::None` for pre-evaluation states
and vibes-grade tolerance. CI gate pattern: exact-kind matching or `any_of` composition;
never `>= Behavioral` (no ordering exists).

**Closed predicate grammar** (5 sealed leaf primitives v0.1; 3 combinators):
- `ratified_doc(path?, min_version?, anchor?, sibling_json?)` — doc exists + frontmatter check
- `signers(required, roles?, against?, signature_allow?, signature_prefer?)` — sidecar signers check; `signature_allow` is categorical allow-set per ABO/Rh biology (B6)
- `signed_trailer(key, role?, count?)` — git trailer via `git interpret-trailers`
- `oracles_complete(files)` — oracle completion markers exist
- `fresh_within_days(n)` — most recent current-fingerprint signer `.date` within N days (NFA-21: stale-fingerprint signer entries excluded)

Combinators: `all_of` (co-stimulation biology B3), `any_of` (redundant-pathway biology),
`not` (inhibitory-checkpoint biology). Schema rejects zero-leaf compositions at parse-time.

**4-point bright-line rule** for leaf primitives invoking external binaries: (1) binary
named in leaf source; (2) has own release process; (3) does NOT execute user-supplied
code; (4) invocation args fixed except for declared substrate-parameters. Only
`signed_trailer` invokes external tooling (`git interpret-trailers`); all four points
satisfied.

**Anti-laundering safeguards** on delta-attestation (`SignerBasis::DeltaFrom`): chain-depth
cap (default 3; hard-floor constants `HARD_DELTA_CHAIN_CAP_MAX = 10`,
`HARD_DELTA_CHAIN_CAP_MIN = 1`; enforced in v0.1 evaluator); cumulative-fingerprint
`cumulative_root_fingerprint` field (schema present; threshold check deferred to v0.2);
non-empty `rationale` schema-enforced. Chain-depth cap + non-empty rationale close the
primary laundering channel in v0.1.

**Tolerance-ratification via isomorphic schema**: same `Ratification` struct for
`#[immune]` and `#[antigen_tolerance]` via `RatificationKind ∈ {Immunity, Tolerance}`
discriminator. `tolerance-vibes-grade` hint surfaces the gap when consumers haven't
opted into `sidecar = true`. v0.1 limitation: `ToleranceVibesGrade` maps to
`AuditHint::NoneApplicable` at top-level API; named hint visible at substrate layer.
Full enum migration is A3+ work.

**Predicate weakening explicit-declaration requirement**: consumer crate's `#[descended_from(X)]`
redeclaration using a weaker predicate must declare `weakened_from: AntigenIdentifier` +
`weakening_rationale: String` (per Eiffel variance rule + ADR-018). Audit emits
`discipline-predicate-weakening-undeclared` on silent weakening.

**v0.1 fingerprint-under-scope semantic** (FA-3): `scope` is a metadata label in v0.1;
item-level fingerprints used regardless of scope. File-scope hash-of-all-items is v0.2+.

**Witness-provider-crate trust boundary**: v0.1 ships sealed leaf set. v0.2+ ADR MUST
specify actual enforcement mechanism (WASM sandboxing, `no_std` + restricted-deps, or
subprocess isolation) — documentation-trust insufficient (adversarial T5-R).

**Discipline-vs-machinery unification asymmetry**: substrate-witnesses and cross-crate
witnesses share discipline-level unification (tier-honesty; SubstrateState evidence kind;
Execution ceiling) but NOT machinery (separate parsers, separate recognition pipelines).
Enforced via in-code comment blocks + `atk_a3_unification_guardrail.rs` adversarial test.

### Mechanics

**Schema** (`antigen-attestation` crate; `schema.rs`):
`Ratification { schema_version, kind, antigen, source_file, items: Vec<ItemRatification> }`
where `ItemRatification { item_path, current_fingerprint, doc_ref?, signers, oracles, fresh_through?, extensions }`.
`Signer { name, role?, date, signed_against_fingerprint, basis: SignerBasis, strength: SignatureStrength, signature? }`.
`SignerBasis ∈ { Fresh, DeltaFrom { prior_fingerprint, cumulative_root_fingerprint, chain_depth, rationale } }`.
`SignatureStrength ∈ { TextStamp, GitTrust, CryptoSigned }` (ordinal; retains `PartialOrd+Ord` for weakest-link reporting).
Cross-language primitive: `.attest/` JSON sidecars, not proc-macro syntax.
Schema-governance: JSON Schema backward compatibility required on any `schema_version` bump.

**Sidecar location**: per-antigen-per-file under `.attest/` subfolder adjacent to source.
`src/numerics.rs` carrying `SignedZeroDiscipline` on `sinh` → `src/numerics.attest/SignedZeroDiscipline.json`.

**CLI families**:
- `cargo antigen attest scaffold [--minimal] [--file F] [--antigen A] [--item I]`
- `cargo antigen attest scaffold-anchor` — one-shot anchor (scaffold + sign + freeze)
- `cargo antigen attest sign [--strength TextStamp|GitTrust|CryptoSigned]`
- `cargo antigen attest check [--file F | --all]`
- `cargo antigen attest delta [--from FP] [--as NAME] [--rationale R]`
- `cargo antigen attest list [--orphan-scan]` — `--orphan-scan` walks `.attest/` independent of scan-side discovery
- `cargo antigen attest gc` — report-only in v0.1 (bidirectional orphan detection)
- `cargo antigen attest migrate --from v1 --to v2 [--acknowledge-scheme-migration] [--set-basis KIND]` — `--set-basis` defaults to `legacy_unknown` when prior sidecar lacked `basis` field (FA-1 tier-honest default)
- `cargo antigen tolerate scaffold/sign/check/list` — parallel tolerance family

### Enforcement

**E1** — `cargo antigen audit` evaluates substrate-witness predicates against on-disk
sidecars + git log + named docs. Failures produce `WitnessTier::None` or `Reachability`
with specific hints; CI gates fail on `--min-tier execution` when predicate doesn't pass.

**E2** — `Ratification` JSON sidecar validated against schema at parse-time. Empty
rationale on `DeltaFrom` rejected. Zero-leaf compositions rejected at macro-parse time
(trybuild fixture verifies error).

**E3** — `attest delta` CLI refuses entries exceeding chain-depth cap, or carrying empty
rationale. Distinct exit codes for CI routing.

**E4** — Closed-set tool bright-line at leaf-design review; documented in
`antigen-attestation/src/predicate.rs` module doc.

**E5** — Discipline-vs-machinery unification guardrail: in-code comment blocks +
`antigen-attestation/tests/atk_a3_unification_guardrail.rs` adversarial precision test.

**E6** — Tier-honesty ratchet: `EvidenceKind` ceiling per-kind enforced at derivation;
`SubstrateState` cannot reach `FormalProof`; `signature_strength` is `None` until
DSSE/Sigstore activate (v0.4+).

**E7** — Tambear-adoption smoke test (Phase 4): primitive exercises `SignedZeroDiscipline`
on sinh/cosh. Adoption findings feed v0.2 amendment planning.

### Resolves

- **Discipline failure-class gap**: failure-classes with substrate witnesses now have
  first-class declarative path (`requires = <predicate>` on `#[immune]`).
- **ADR-011 vibes-grade tolerance gap**: `tolerance-vibes-grade` hint surfaces absence
  at substrate layer; `WitnessTier::None` surfaces at top-level API.
- **ADR-005 Amendment 3 three-axis extension**: `WitnessTier × AuditHint × EvidenceKind`.
- **Delta-attestation laundering surface**: chain-depth cap + non-empty rationale enforced;
  cumulative-fingerprint threshold in schema (evaluator enforcement v0.2).
- **Closed-set tool boundary**: 4-point bright-line replaces vague "ecosystem tools."
- **Discipline-vs-machinery unification drift risk**: comment + adversarial test prevents
  silent shared-parser regression.
- **Reviewer-not-committer PR workflow**: `signed_trailer` leaf for v0.1; v0.4+
  crypto-signing via DSSE decouples from committer identity.

### Biology grounding

**B1** — Recognition vs evidence role-distinction: `requires = <predicate>` is recognition
(B-cell receptor); `.attest/*.json` sidecar is evidence of attestation (antibody /
cellular activation history). Biology mandates role-distinction; sidecar-as-evidence is
architecture (code-drift defense via fingerprint pinning).

**B2** — Somatic hypermutation → WHY-of-attestation: `SignerBasis::Fresh.reasoning`
records WHY the signer signed (F17 transverse principle unifies six independent gap-instances).

**B3** — Closed combinator grammar → immune-cell signal integration: `all_of` ↔
co-stimulation; `any_of` ↔ redundant pathways; `not` ↔ inhibitory checkpoints.

**B4** — Memory-cell waning → `fresh_within_days`: persistence + staleness doubly
justified (engineering + biology).

**B5** — `DeltaFrom` carry-forward → vaccination booster: re-attestation without
full fresh review, with recorded rationale.

**B6** — Civic notary 800-year arc → `SignatureStrength` escalation: `TextStamp` ↔
pre-institutional peer testimony; `GitTrust` ↔ civic notary (workspace-bounded,
institutionally accountable); `CryptoSigned` ↔ notary public with universal license
(DSSE + Sigstore identity-binding, v0.4+).

**B7** — MHC-presentation → `evidence_provenance`: structured data encoding ADR-006
three-instances threshold; audit can verify `n >= 3` for stdlib-promotion eligibility.

**B8** — Kinetic-proofreading minimum-across-signers: weakest-link aggregation when
multiple individually-passing signers contribute to a collective tier.

### Open questions carried to v0.2

- CODEOWNERS interop (`signers(required_role = "math-team", ...)`) — Amendment 1
- Leaf-provider extensibility with actual enforcement mechanism (WASM / `no_std` / subprocess) — v0.2 ADR
- Coarser-scope sidecar location (module/crate/workspace) — per F21 coordination-substrate
- Cumulative-fingerprint threshold evaluator enforcement — Amendment 1 or standalone
- Fresh.reasoning promotion from `Option<String>` to required — v0.3 after adoption confirms
- Compound-evidence list form (full list per item, not just `compound_evidence: bool`) — Amendment 1

---

## [ADR-020] Cross-cutting attestation primitive

**Status**: Ratified 2026-05-20.

**Participants**: navigator (F25 architectural call + OQ pre-rulings), aristotle (F26
Phase 1-8), adversarial (ATK-020, 10 findings), scientist (v0 + v1 + v2 draft arc),
naturalist (ABO/Rh cognate + notary-arc extension), Tekgy (v0.1-rc timing verdict).

**Related**:
- ADR-009 (adoption gradient — architecture-deciding constraint for the SPLIT decision)
- ADR-019 (substrate-witness predicate family — consumer of attestation data; ratified 03a36c0)
- ADR-003 (biological metaphor; ABO/Rh + notary-arc B6 grounding)
- ADR-005 Amendment 2 (rationale-as-required-field; explicit waiver for `why = optional`)
- F11 (multi-witness lattice discipline; independent-axes reporting)

**Implicit pattern elevated** (per ADR-004 Enforcement): the structural declaration
layer `#[immune(X, witness = fn)]` captures the verification layer but not the review
layer. A codebase where humans reviewed code before a test existed has no structural
record of that review. This ADR elevates the review layer from implicit to explicit.

### Finding

F25 established two irreducible kernels: **attestation** (declare who should attest,
static metadata at code-authoring time) and **substrate-witness predicate** (evaluate
whether recorded attestation satisfies a condition, at audit time). These are producer
and consumer of different kinds.

ADR-009 adoption gradient is the architecture-deciding constraint: embedding attestation
inside the predicate grammar would require sidecar infrastructure at Layer 1, violating
Layer 1's "works without sidecar infrastructure" commitment. SPLIT is the only
architecturally-honest choice. F26 confirms. ADR-020 IS the SPLIT.

### Decision

**`attested = (who, allowed_types, why, scope)` is a new macro parameter on any antigen
macro, parsed at proc-macro time, written into the compiled artifact as static attestation
metadata, and evaluated by `cargo antigen audit` independently of the sidecar predicate
grammar.**

| Field | Type | Required | Meaning |
|---|---|---|---|
| `who` | `[name, ...]` | YES | Named reviewers expected to attest this item |
| `allowed_types` | `[TextStamp \| GitTrust \| CryptoSigned]` | NO (absent = all) | Accepted signature strength categories |
| `why` | `"..."` | NO (recommended) | Human-readable rationale |
| `scope` | `site \| file \| package \| workspace` | NO (default `site`) | Attestation claim granularity |

**`who` field constraints** (ATK-020-1, ATK-020-8):
- Non-empty required
- Names trimmed at proc-macro parse; whitespace-only names are compile error
- Duplicate names are compile error
- Set semantics enforced at compile time

**`allowed_types` field constraints** (ATK-020-2):
- ABSENT = all types accepted (O-negative universal-donor default)
- EXPLICIT EMPTY `allowed_types = []` is a COMPILE ERROR — permanently unfulfillable;
  joins the `EmptySignersList`, `ZeroTrailerCount`, `EmptyOraclesList` class of
  vacuous-bypass guards

**`why` field — Amendment 2 waiver**: `why` is optional per ADR-005 Amendment 2 waiver.
Declarations are structural intent; review rationale lives at sidecar layer in
`SignerBasis::Fresh.reasoning` (F17/F20). Compile-time `why` would duplicate or conflict.
Recommended but not required; when present, static metadata only (no evaluation effect).

**`scope = workspace` in v0.1**: does NOT provide coverage propagation. Every site
requiring attestation coverage carries its own `attested = (...)`. Cross-site enforcement
defers to v0.2+ (F21 coordination-substrate guidance).

**Identity binding**: `who` list uses name-string equality. At TextStamp strength, anyone
can configure any name. `allowed_types = [GitTrust, CryptoSigned]` strongly recommended
alongside `who` where identity binding matters — `GitTrust` binds name to git commit
authorship; `CryptoSigned` binds to cryptographic key. Without this, `who = ["alice"]`
is documentation-quality intent, not a binding identity check.

### Mechanics

```rust
#[immune(SignedZeroDiscipline,
    witness = test_sinh_signed_zero,
    attested = (
        who = ["alice"],
        allowed_types = [GitTrust, CryptoSigned],
        why = "alice reviewed the sinh implementation against Higham 2002 §6.3",
    ),
)]
pub fn sinh(x: f64) -> f64 { ... }
```

**Audit tier mapping** (current-fp filtering required per ATK-020-7 / NFA-21 pattern):

| State | `WitnessTier` | `EvidenceKind` | `AuditHint` |
|---|---|---|---|
| Declared; no current-fp sidecar entry for any declared `who` | `Reachability` | `SubstrateState` | `attestation-declared-not-verified` |
| Declared + current-fp entries for `who` match `allowed_types` | `Execution` | `SubstrateState` | (per existing sidecar hint) |
| Declared; current-fp entry exists; signer strength NOT in `allowed_types` | `Reachability` | `SubstrateState` | `attestation-type-not-in-allowed-types` |
| Declared; signer NOT in `who` (via `--force`) | `Reachability` | `SubstrateState` | `attestation-signer-not-in-declared-who` |
| `scope = package \| workspace` in v0.1 | `Reachability` | `SubstrateState` | `attestation-scope-not-yet-enforced` |
| Not declared | (per `witness =`) | (per witness) | (no change) |

Stale signer entries (signed against prior fingerprint) do NOT satisfy the attestation
declaration. Current-fp filtering is the same NFA-18/19/20/21 discipline applied
throughout the evaluator.

**Multi-axis composition** (F11; F26): when both `attested` and `witness` are declared,
each reports independently. Audit emits `compound_evidence: true`. MUST NOT collapse to
composite tier. `conservative_tier = MIN(attestation_tier, witness_tier)` is the
recommended CI aggregation field for compound-evidence sites.

**CI enforcement**: gate on BOTH hints for `who` compliance:
```
attestation-declared-not-verified == 0
AND attestation-signer-not-in-declared-who == 0
```
Using only the first is insufficient — `--force` satisfies it while violating `who`.

### Adoption gradient

| Layer | Behavior | Infrastructure |
|---|---|---|
| 1 (naive) | Not declared | None |
| 1+ (light-touch) | `attested = (who = ["alice"])` | None |
| 2 (substrate) | `attested` + `cargo antigen attest sign` | Sidecar write |
| 3 (predicate) | `attested` + `requires = all_of([signers(...)])` | Sidecar + predicate |

### Biology grounding

**ABO/Rh** (naturalist): `allowed_types` is categorical set membership. Scope limitations:
no within-allow-list trust gradient; no emergency-override; no multi-recipient consensus.

**Structural vs nominal recognition**: `GitTrust`/`CryptoSigned` in `allowed_types` adds
structural binding analogous to epitope recognition. `TextStamp` without `allowed_types`
specifying higher tiers is like recognizing by name label rather than molecular structure.

**Notary arc (B6)**: cross-cutting domain-agnosticism grounds the "attest any macro"
property. Attestation (notary witnessing) and predicate evaluation (clinical titer test)
are complementary, not competing.

### Resolves

- "Humans reviewed this but there's no structural record" gap at Layer 1
- SPLIT vs. widen ADR-019 (SPLIT, per F25 + F26)
- `allowed_types` categorical-vs-ordinal (categorical, ABO/Rh)
- Adoption gradient tension (Layer 1 compatible; no sidecar required at compile time)
- `why` Amendment 2 compliance (waiver with explicit reasoning)
- Scope v0.1 behavior (syntactic acceptance + non-enforcement hint)
- `allowed_types = []` vacuous bypass (compile error)
- CI gate gap for `--force` (dual-hint gate requirement)
- Stale-fp entry inflating attestation tier (current-fp-only filtering)
- Missing type-mismatch hint (`attestation-type-not-in-allowed-types`)

---

## [ADR-021] OracleRef generalization + additive-only schema evolution + oracle-as-artifact-class

**Status**: Ratified 2026-05-20.

**Participants**: Tekgy (two architectural reframes + slice-e commitment), navigator
(F25-equivalent architectural call; R1/R2 reframes), aristotle (F27 Model B Phase 1-8;
F28 oracle CLI Phase 1-8), adversarial (ATK-021 1-10 initial; ATK-021-11 through -18
Model B), naturalist (B-021-1 through B-021-5, all Class 1), scientist (v0 through v4
draft arc).

**Related**:
- ADR-019 (substrate-witness predicate family; ratified 03a36c0; `oracles_complete` leaf
  consumer of this ADR's Oracle artifact-class)
- ADR-020 (cross-cutting attestation; ratified bb3b2b9; sign-time-validity principle
  shared discipline)
- ADR-007 (anti-YAGNI; oracle lifecycle CLI confirmed structurally-required)
- ADR-005 Amendment 2 (rationale-as-required-field; `Steward.authorization_basis` and
  `StateTransition.rationale` both inherit Amendment 2 discipline)
- ADR-009 (adoption gradient; Layer 1 preserved; oracle CLI is Layer-2+ machinery)

### Finding

**R1 (Tekgy)**: Audit NEVER reads oracle content. Substantive judgment ("does the code
satisfy what the oracle specifies?") is human/LLM work done at sign-time. Audit validates
structural well-formedness + oracle state + completion marker + version-pin.

**R2 (Tekgy)**: The current `OracleRef { path, status }` struct is insufficient. An oracle
is not a typed pointer carrying a completion marker — it is a **structurally distinguished
artifact-class** with its own state machine, dedicated stewards, provenance, and lifecycle
tracking. Without lifecycle structure, discipline degrades to convention — the exact failure
mode antigen exists to solve.

**Reframe composition** (aristotle F27): R1 (content-blindness) + R2 (richer metadata)
compose cleanly. Richer metadata + no content access = stronger schema-enforceable
discipline than content access + weaker metadata would produce.

### Decision

**D1 — OracleRef becomes a tagged union** (last breaking schema change before additive-only
commits):

```
OracleRef: LocalFile | Url | Doi | Arxiv | GitHubIssue | Other
```

All variants behave identically to the audit (content-blindness). Audit validates
structural well-formedness (URL parses, DOI matches format) + completion marker +
version-pin. No network calls.

**D2 — Additive-only schema evolution**: core fields invariant; new fields always optional
with `serde(default)` matching prior behavior; `extensions: BTreeMap<String, Value>`;
`schema_version` informational only; `Other { kind, reference }` escape hatch. `attest
migrate` verb: DROPPED (additive-only makes migration unnecessary; old `OracleRef { path,
status }` sidecars parsed via two-pass deserialization with `oracle-ref-needs-migration`
hint).

**D3 — Oracle as artifact-class** (Model B; aristotle F27; adversarial ATK-021-11 through -18):

Oracle struct: `id`, `reference: OracleRef`, `state: OracleState`, `stewards: Vec<Steward>`,
`created: Provenance`, `version: OracleVersion`, `transitions: Vec<StateTransition>`,
`extensions`.

State machine (monotonic; backward transitions prohibited):
- `Draft` — not yet authoritatively established; signers CANNOT attest against Draft oracles
- `Complete` — authoritatively established; signers may attest
- `Deprecated { superseded_by, reason }` — superseded; prior attestations honored at Execution
- `Retired { reason, retired_by }` — permanently gone; prior attestations honored at Execution
- `Revoked { reason, revoked_by, invalidates_prior_attestations: bool }` — compromised or
  incorrect; `invalidates_prior=true` retroactively demotes prior attestations to Reachability

`Steward` struct: `name`, `role?`, `authorization_basis: String` (REQUIRED non-empty per
Amendment 2 — WHY this person has steward authority).

`StateTransition` struct: `from`, `to`, `authorized_by` (must appear in `stewards[*].name`),
`at: NaiveDate`, `rationale: String` (REQUIRED non-empty per Amendment 2).

Minimum 2 stewards at creation (ATK-021-13: succession mitigation). GitTrust minimum for
state transitions (ATK-021-15). Steward set is APPEND-ONLY in v0.1 (F28-R1).

**D4 — Sign-time-validity principle** (elevated from implicit mechanics; ATK-021-11):

> An attestation's evidentiary tier is determined by the oracle's state AT SIGN TIME,
> not at audit time.

Oracle state changes AFTER a valid attestation produce audit hints only — MUST NOT degrade
the signer's attested tier. Exceptions: (1) `Revoked(invalidates_prior=true)` — explicit
steward decision to invalidate; demotes prior attestations to Reachability. (2) Fraudulent
state transition (TextStamp-authorized) — transition flagged but attestation preserved.

### Mechanics

**Oracle CLI subfamily** (all five slices ship in v0.1-rc per Tekgy):
- `cargo antigen oracle list` — workspace oracle inventory
- `cargo antigen oracle status <id>` — state + transitions + stewards + attestations
- `cargo antigen oracle declare --as steward --reference <ref> --rationale <r>` — create DRAFT
- `cargo antigen oracle complete --as steward --id <id> --version <v> --rationale <r>` — DRAFT→COMPLETE
- `cargo antigen oracle deprecate --as steward --id <id> --superseded-by <id?> --rationale <r>`
- `cargo antigen oracle retire --as steward --id <id> --rationale <r>`
- `cargo antigen oracle revoke --as steward --id <id> --reason <r> --invalidates-prior {true|false}`

**CLI is friction-layer; schema is enforcement-layer** (F28-R3). Hand-edited sidecars that
don't violate schema invariants are valid; CLI captures steward's git-trust identity at
invocation. Parallel to `attest sign` discipline.

**Naming disambiguation** (F28-R2): `cargo antigen attest oracle complete` (per-attestation:
marks signer reviewed the oracle) renamed → `cargo antigen attest oracle mark`. Distinct
from `cargo antigen oracle complete` (per-oracle state transition: steward moves DRAFT→COMPLETE).

**Draft state scope**: Draft-blocking applies ONLY to `oracles_complete(...)` predicate leaf.
`ratified_doc(...)`, `signers(...)`, and other leaves are not affected by oracle state machine.
Oracle curation (state) and document ratification are separate discipline layers.

**Adoption gradient**:
- Layer 1: `oracles_complete([...])` without sidecar → `oracle-no-sidecar-information` at Reachability
- Layer 1+: `oracle declare` one-time entry point → Layer 2 operational
- Layer 2+: full oracle lifecycle CLI active
- Layer 3: `requires = all_of([oracles_complete([...]), signers(...)])` per F11

### Enforcement

- Schema parse-time: minimum 2 stewards; `authorization_basis` non-empty; transition `rationale`
  non-empty; chronological monotonicity; authorized_by in stewards list; Draft blocks
  `oracles_complete` leaf evaluation
- Audit-time: sign-time-validity (D4) applied throughout; state-change hints emitted; tier
  degradation only on Revoked+invalidates=true
- CLI: steward git-trust identity captured at invocation; TextStamp-level transitions rejected

### Biology grounding

**B-021-1** (temporal axis — immune memory): tier persists independent of oracle reachability.
**B-021-2** (substrate axis — BCR direct recognition): uniform behavior across oracle kinds.
**B-021-3** (evolution axis — V(D)J recombination): additive-only schema.
**B-021-4** (role-separation axis — FDC stewardship): FDCs are of stromal origin, different
lineage from B-cells. Steward/signer structural separation is biology-predicted, not convention.
**B-021-5** (state-machine axis): pre-B cell gating (Draft-blocks-signers); germinal-center
exit decisions (authorized transitions); memory senescence (Retired preserves historical evidence).

### Resolves

- `oracles_complete` leaf restricted to local files → all substrate-addressable oracle
  references (file, URL, DOI, arXiv, GitHub, Other)
- Discipline-as-convention oracle lifecycle → structurally enforced lifecycle via state machine
  + stewardship
- Missing Retired/Revoked distinction → five-state machine with explicit trust-impact semantics
- `attest migrate` verb → DROPPED (additive-only makes migration unnecessary)
- `attest move` verb → DROPPED (discipline enforced by gc/audit)
- Orphaned-steward risk → minimum-2-stewards at creation + append-only steward set in v0.1
- Post-sign-time oracle state changes → sign-time-validity principle (D4)
- `attest oracle complete` naming collision → renamed to `attest oracle mark`

---

## [ADR-022] Stdlib-vs-Extension: Two Disciplines, One Public API

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8); Tekgy (lock 2026-05-21 night); naturalist (biology-validation: gate passed — biology-as-discovery feeds stdlib growth via ADR-003 Amendment 1); adversarial (no blocking findings).

**Related**: ADR-006 Amendment 1 (recognition discipline scoped to adopter); ADR-003 Amendment 1 (biology); ADR-007 (anti-YAGNI); ADR-009 (adoption gradient); ADR-021 (additive-only schema evolution).

**Implicit pattern elevated** (per ADR-004): the dual-discipline architecture is currently implicit in the project structure (stdlib crate separate from extension surfaces) but never named at the ADR layer.

### Finding

Antigen has two growth layers structurally:

1. **Stdlib growth** (`antigen-stdlib` crate): comprehensive coverage of the failure landscape; ships canonical primitives all adopters consume.
2. **Adopter extension growth** (per-consumer-crate): domain-specific antigens, witnesses, predicates, fingerprints declared against stdlib primitives.

These layers have DIFFERENT growth disciplines (per ADR-006 Amendment 1): stdlib = research; adopter = recognition. They have DIFFERENT substrate-grounding requirements. They have DIFFERENT cadences. But they SHARE a public API — the primitive machinery adopters consume. The contract BETWEEN stdlib and extension is itself part of the public API.

### Decision

**Antigen ships two architectural layers with distinct growth disciplines but a unified public API. The extension contract is first-class: changes to the stdlib primitive interface are semver breaks; adopters can rely on the contract across stdlib releases.**

**The stdlib layer ships**: (1) comprehensive PRIMITIVES (immune-system machinery: cells, signals, states, responses); (2) canonical antigens for failure-classes worth defending across all software development; (3) documentation of extension patterns; (4) extension tooling (cargo-antigen recognizes custom antigens identically to stdlib).

**The extension contract is part of the public API**:
- Primitive macro names (`#[antigen]`, `#[immune]`, etc.) are stable; rename = major-version-break
- Sidecar schema field-names + types are stable; change = semver-break
- Audit hint vocabulary is stable for adopter consumption; new variants ship additively (per ADR-021)
- Composition rules (how custom antigens compose with stdlib via `requires =` predicates) are stable

**The extension contract is NOT**: the specific stdlib antigens shipped (additive after v1.0); the internal evaluator implementation; the specific failure-classes covered (stdlib grows toward comprehensive coverage).

### Mechanics

**Crate organization**:
- `antigen` (macros + core types) — primitive machinery; semver-stable
- `antigen-attestation` (sidecar schema) — extension contract; semver-stable
- `antigen-stdlib` (canonical antigens) — research-discipline grown; additive after v1.0
- `antigen-extensions-<domain>` (adopter or community-ratified extension drops) — recognition-discipline grown

**Extension contract documentation**:
- `docs/extension-contract.md` — first-class doc enumerating what adopters can rely on
- `docs/extension-patterns/` — directory of canonical extension patterns

**Versioning**: `antigen` + `antigen-attestation` follow semver strictly; `antigen-stdlib` versions independently; `antigen-extensions-<domain>` crates version per maintainer.

### Sweep-level consequences

- Stdlib team plans research-arc drops (quarterly cadence)
- Extension-contract changes require ADR-level ratification
- Adopter crates can be authored with confidence that the primitive surface is stable
- Community extension crates (`antigen-async`, `antigen-embedded`, `antigen-wasm`, `antigen-ffi`, `antigen-sqlx`, `antigen-http`, `antigen-cloud`) become viable

### Enforcement

- Extension contract sections marked `[CONTRACT]` in docs/code-comments
- CI test: parse + recompile downstream adopter crate fixture against the contract; any breaking change fails the test
- Semver-checks at release time enforce the contract boundary

### Resolves

- The implicit-architecture failure mode (stdlib-vs-extension separation was load-bearing but never named at ADR layer)
- The "what can adopters rely on?" question (now: the contract; new stdlib antigens are not the contract)
- The growth-discipline question for stdlib (separate ADR-006 Amendment 1)
- The growth-discipline question for extensions (preserved per original ADR-006)

### What this ADR does NOT do

- Does NOT define the stdlib research discipline (that's ADR-006 Amendment 1)
- Does NOT enumerate the canonical extension patterns (that's `docs/extension-patterns/`)
- Does NOT permit extension-contract changes without semver-break ceremony

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

---

## [ADR-023] Deferred-Defense Family: Loudness-as-Discipline for Intentional Non-Immunity

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named the family); naturalist (biology grounding sound at per-primitive level; no major refinement); adversarial (5 BLOCKING + 3 non-blocking attacks, all absorbed).

**Related**: ADR-005 Amendments 2 & 3 (sub-clause F); ADR-006 Amendment 1 (stdlib); ADR-007 (anti-YAGNI); ADR-011 (`#[antigen_tolerance]` closest relative); ADR-022 (Stdlib-vs-Extension); ADR-023 deferred-defense family; ADR-026 (VCS — rollback-as-triage uses `#[orient]`-shape declarations).

**Implicit pattern elevated** (per ADR-004): deferred defenses have been captured via inline comments, `// TODO`, or tolerance-without-distinction. This ADR elevates the richer vocabulary of intentional non-immunity to structural primitives.

### Finding

ADR-011 ships `#[antigen_tolerance]` as the escape valve for intentional non-immunity. Biology has a richer vocabulary:

- **Anergy**: T-cell or B-cell encounters antigen but fails to respond due to lack of co-stimulation. Alive but unresponsive. Reversible if co-stimulation arrives.
- **Immunosuppression**: pharmacological/pathological reduction of immune response. Time-bounded; expected to be revisited.
- **Pox party**: pre-emptive controlled exposure to build immunity (chaos engineering, fault injection, red-team exercises).
- **Orient**: explicit orientation period during which systems acknowledge they LACK immunity.

These are STRUCTURALLY DISTINCT from tolerance:

| Mechanism | Posture | Loudness | Aging | Escalation | In ADR-023? |
|---|---|---|---|---|---|
| `#[antigen_tolerance]` | Compliance | Quiet | None | None | No (per ADR-011) |
| `#[anergy]` | Deferred-but-muted | Medium-loud | Yes | Yes — auto-re-engage | YES |
| `#[immunosuppress]` | Deliberately-muted | Loud | Yes | Yes — expires | YES |
| `#[poxparty]` | Pre-emptive-controlled-exposure | Loud | Bounded | Outcome-feedback | YES |
| `#[orient]` | Pre-immunity-acknowledged | Loud | Yes | Auto-escalate | YES |
| `#[vaccinate]` | Active-immunity-building | Loud at invocation | None after | N/A | NO (ADR-007 standalone) |

The unifying property: **loudness IS the discipline**. Silently-deferred defense becomes silently-broken defense.

### Decision

**Antigen ships a Deferred-Defense Family of FOUR declarative primitives — `#[anergy]`, `#[immunosuppress]`, `#[poxparty]`, `#[orient]` — each capturing a structurally distinct mode of intentional non-immunity. The family is governed by loudness-as-discipline: each primitive's audit visibility is structurally proportionate to its mode's risk profile. Aging + escalation mechanics prevent silent permanence. Cap-enforcement happens at PARSE-TIME. Poxparty has STRUCTURAL compile-time isolation.**

`#[vaccinate]` is REMOVED from this family — vaccinate is active-immunity-building, not deferred-defense. It remains in scope per ADR-007 commitment as a standalone primitive.

**`#[anergy(X, reason, expected_co_stimulation, until)]`**:
- `until` is REQUIRED (removed `?` from prior draft); anergy without time-bound degrades to tolerance
- `expected_co_stimulation` is ADVISORY ONLY — NOT machine-verified; field names the trigger for future-readers
- Audit emits `anergy-active` hint at Reachability tier; `anergy-co-stimulation-not-arrived` after `until`; `anergy-stale` past grace

**`#[immunosuppress(X, duration, rationale, signed_by)]`**:
- `duration` validated at PARSE TIME against workspace-config cap; default cap = 90d
- COMPILE ERROR emitted for `duration > cap`
- `rationale` minimum 20 characters; UTC date comparison
- Workspace config: `immunosuppress_duration_cap`

**`#[poxparty(X, exercise_type, rationale, signed_by)]`**:
- STRUCTURAL ISOLATION REQUIREMENT: code annotated with `#[poxparty]` MUST be inside `#[cfg(feature = "antigen-poxparty")]` scope OR `#[cfg(test)]`
- Proc-macro reads `CARGO_FEATURE_ANTIGEN_POXPARTY` env var at macro-expansion time; emits COMPILE ERROR if feature not active
- `antigen-poxparty` feature MUST NOT be in default feature set; production builds cannot accidentally enable it
- `exercise_type` field minimum 20-char; outcome-feedback required

**`#[orient(X, learning_path, until)]`**:
- `until` validated at parse time against `deferred_defense_max_horizon` (default 180d)
- Dates beyond `now + max_horizon` REJECTED at parse time; UTC comparison
- Cannot ship to production with `orient-active` past `until` per CI gate

### Mechanics

**Schema additions** (additive per ADR-021): `DeferredDefenseKind` enum: `Anergy | Immunosuppress | Poxparty | Orient` (vaccinate removed); per-kind metadata structs; `signed_by` per ADR-005 Amendment 2.

**Workspace config**: `immunosuppress_duration_cap`, `deferred_defense_max_horizon`, `deferred_defense_default_cap`.

**Audit-hint vocabulary** (cross-ADR substrate-grep verified clean):
- `anergy-active`, `anergy-co-stimulation-not-arrived`, `anergy-stale`
- `immunosuppress-active`, `immunosuppress-expired`, `immunosuppress-duration-cap-exceeded`
- `poxparty-active`, `poxparty-outcome-pending`, `poxparty-outcome-recorded`, `poxparty-outside-isolation`
- `orient-active`, `orient-pending-action-required`
- `deferred-defense-hint-suppressed-without-rationale`

**Hint-fatigue protection**: deferred-defense hint suppression in workspace config requires non-empty rationale; `cargo antigen defer status` provides separate deferred-defense report; adopters cannot silently filter deferred-defense hints.

**CLI additions** (cross-ADR substrate-grep clean): `cargo antigen defer {status, suppress, suppress-expire, anergy, anergy-clear, poxparty start, poxparty outcome, orient, orient-complete}`.

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| Duration cap on `#[immunosuppress]` | parse-time + audit-time | client | parse-time = compile error; audit-time = additional gate |
| `until` cap on `#[anergy]` / `#[orient]` | parse-time + audit-time | client | parse-time = compile error; max-horizon configurable per workspace |
| `#[poxparty]` cfg-gated isolation | parse-time (macro reads CARGO_FEATURE_ANTIGEN_POXPARTY env var) | client + CI | compile error when feature not active; feature absent from default feature set |
| Rationale 20-char minimum | parse-time | client | uniform across reason/rationale/learning_path/exercise_type |
| Aging escalation | parse-time + audit-time | client | compile-time progression; grace period configurable |
| UTC date comparison | parse-time + audit-time | client | timezone-independent; reproducible across CI runners |
| Hint-suppression-requires-rationale | parse-time + audit-time | client | per ADR-005 Amendment 2 |
| Solo-developer single-signer | NONE (named limitation) | — | culturally enforced; workspace `min_signers = 2` option |

### Known limitations

1. **Solo-developer single-signer**: `signed_by` reduces to "you committed to this explicitly" without independent review.
2. **`expected_co_stimulation` not machine-verified**: free-text; advisory only.
3. **Hint-fatigue at scale**: separate defer-status report + structural-suppression-with-rationale mitigates.

### Sweep-level consequences

- Stdlib gains FOUR new declarative primitives (vaccinate standalone in ADR-007)
- Parse-time enforcement of caps + rationale length + date horizons
- Structural cfg-gated isolation for poxparty (compile-error if violated)

### Resolves

- The implicit-deferred-defense failure mode (inline `// TODO` / comment-only suppression; invisible to audit)
- The single-shape-fits-all problem (tolerance was the only declarative primitive)
- The loudness-is-discipline architectural pattern (now structurally expressed)
- Workspace-config cap parse-time bypass (parse-time enforcement closes audit-only gap)
- Poxparty production isolation gap (structural cfg-gated isolation; CI gate)

### What this ADR does NOT do

- Does NOT replace `#[antigen_tolerance]` (compliance-acknowledged cases remain valid)
- Does NOT cover `#[vaccinate]` (moved to ADR-007 standalone commitment)
- Does NOT permit indefinite deferral (workspace caps + aging required; parse-time enforcement)
- Does NOT silently allow poxparty in production (structural cfg-gated; compile-error if violated)

---

## [ADR-024] Three Sibling Families: Convergent Evidence + Recurrent Emergence + Prescriptive Work-Orchestration

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named three families + temporal-arc framing in drill #74); naturalist (3 refinements: dual-axis grounding honesty; MHC routing-fix; temporal-arc forward-pointer); adversarial (3 HIGH attacks absorbed: WitnessClass, SeedKind, disambiguation table).

**Related**: ADR-002 Amendment 2 (compose-vs-compete per family); ADR-003 Amendment 1 (biology); ADR-006 Amendment 1 (stdlib); ADR-007 (anti-YAGNI); ADR-019 (substrate-witness); ADR-022 (Stdlib-vs-Extension); ADR-023 (Deferred-Defense — itch/anergy disambiguation); ADR-028 (antigen-category).

**Implicit pattern elevated** (per ADR-004): antigen's v0.1 vocabulary covers DECLARATION + CHECKING but doesn't structurally cover the TEMPORAL discipline of evidence-aggregation, recurrence-detection, or work-orchestration.

### Finding

Drill #74 substrate identified three primitive families covering the temporal arc:

**Family 1: Convergent Evidence (backward-looking)** — `#[diagnostic]`, `#[clonal]`, `#[igg]`, `#[crossreactive]`, `#[polyclonal]`, `#[monoclonal]`, `#[adcc]`

**Family 2: Recurrent Emergence (present-looking)** — `#[itch]`, `#[recurrence_anchor]`, `#[crystallize]`, `#[chronic]`, `#[saturate]`, `#[strand]`

**Family 3: Prescriptive Work-Orchestration (forward-looking)** — `#[panel]`, `#[ddx]`, `#[rx]`, `#[triage]`, `#[refer]`, `#[biopsy]`, `#[culture]`, `#[titer]`, `#[quarantine]`

### Decision

**Antigen ships three sibling primitive families as a single ratification per temporal-arc cohesion. WitnessClass enum + non-deterministic seed requirement + family disambiguation table address adversarial findings. Biology grounding is dual-axis (immunology + clinical-medicine + cognitive) per naturalist's refinement.**

**Convergent Evidence Family** ships v0.2 macros (6 members); `WitnessClass` enum for independence-checking; `SeedKind` enum for non-deterministic seed enforcement.

**Recurrent Emergence Family** ships v0.2 macros + camp integration; cross-substrate intelligence layers phased v0.3+.

**Prescriptive Work-Orchestration Family** ships v0.2 macros (9 members) + camp integration.

**WitnessClass enum** (per adversarial C1: `min_independent` = distinct CLASSES not distinct witnesses):

```rust
pub enum WitnessClass {
    StaticAnalysis, PropertyTest, FormalVerification,
    ManualReview, RuntimeFuzz, SubstrateWitness,
}
```

**SeedKind enum** (per adversarial C2: non-deterministic seed required for `#[clonal]`):

```rust
pub enum SeedKind {
    Random, EntropyFromCi, TimestampSeeded,
    Fixed(u64),  // REJECTED for #[clonal]; emits compile-time error or audit-time fail
}
```

**IgG identity-collapse limitation** (per adversarial C3): `#[igg]` source-independence is NOMINAL (different signer identity strings) not STRUCTURAL. Same limitation as ADR-025 dep_attested B6-C.

**Disambiguation table** (per adversarial C4; load-bearing — adopter-misuse defeats discipline):

| If your situation is... | Use... | Not... |
|---|---|---|
| "Multiple independent witnesses converge on confirming defense (backward evidence)" | `#[diagnostic]` | `#[panel]` |
| "Need to ORDER a battery of diagnostic tests with role-workflow" | `#[panel]` | `#[diagnostic]` |
| "Pattern noticed below threshold; no commitment yet" | `#[itch]` | `#[anergy]` (ADR-023) |
| "Intentionally not defending while waiting for upstream condition" | `#[anergy]` (ADR-023) | `#[itch]` |
| "Cross-substrate recurrence; threshold reached; want to surface for action" | `#[recurrence_anchor]` | `#[chronic]` |
| "Low-level persistent signal NOT cross-substrate" | `#[chronic]` | `#[recurrence_anchor]` |
| "Differential diagnosis: rule-out workflow" | `#[ddx]` | `#[panel]` |

**Biology grounding — dual-axis honesty** (per naturalist refinement 1; `#[titer]` reassigned per Amendment 1):
- **Immunology-proper** (7 primitives): `#[clonal]`, `#[igg]`, `#[crossreactive]`, `#[polyclonal]`, `#[monoclonal]`, `#[adcc]`, `#[chronic]`
- **Clinical-medicine** (11 primitives): `#[diagnostic]`, `#[panel]`, `#[ddx]`, `#[rx]`, `#[triage]`, `#[refer]`, `#[biopsy]`, `#[culture]`, `#[titer]`, `#[quarantine]`, `#[recurrence_anchor]`
- **Cognitive-organizational** (4 primitives): `#[itch]`, `#[saturate]`, `#[crystallize]`, `#[strand]`

**MHC routing-error correction** (per naturalist refinement 2): no primitive in ADR-024 maps to MHC. The MHC cognate is `#[presents]` in ADR-001.

**Temporal-arc forward-pointer** (per naturalist refinement 3): other principles may ground different family-groupings (specificity-arc, location-arc, duration-arc, layer-of-immune-response-arc). This ADR ratifies the temporal-arc grouping; future ADRs extend the recursion.

**Compose vs compete decision**: Convergent COMPOSES with proptest/kani/clippy; COMPETES on aggregation discipline. Recurrent COMPETES (no existing tool covers cross-substrate recurrence with antigen's structural vocabulary). Prescriptive COMPETES (Asana/Jira/Linear/Notion alternatives; antigen-cohesion serves antigen-adopters better).

**Antigen-category** (per ADR-028): Convergent = mostly FunctionalCorrectness; Recurrent = mostly SubstrateAlignment; Prescriptive = mostly SubstrateAlignment with some FunctionalCorrectness.

### Mechanics

**Schema additions** (additive per ADR-021): new family-tagged macro variants per member set; `WitnessClass` + `SeedKind` enums; new schema structs (`DiagnosticEvidence`, `RecurrenceAnchor`, `PanelDeclaration`, etc.); camp integration via new sidecar conventions for prescriptive family.

**Audit-hint vocabulary** (~30 new hints; cross-ADR substrate-grep verified; examples): `diagnostic-modality-insufficient`, `diagnostic-modalities-class-collapsed`, `clonal-fixed-seed-detected`, `igg-identity-collapse-warning`, `itch-noticed-not-anchored`, `recurrence-threshold-reached-no-action`, `panel-needs-unfulfilled`, `ddx-rule-out-pending`, `rx-treatment-not-applied`, `triage-decision-stale`, `quarantine-still-active-past-until`.

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| WitnessClass independence count | audit-time | client + CI | configurable; suppression requires rationale per ADR-005 Amendment 2 |
| `clonal-fixed-seed-detected` | parse-time OR audit-time | client | static analysis at parse/audit; flag if witness config has fixed seed |
| `#[igg]` temporal-independence | audit-time | client + CI | mechanical; signers must have ≥ min_span timestamps |
| `#[igg]` source-independence | NONE (named limitation) | — | nominal-only |
| Prescriptive role-authorization | audit-time | client + CI | per ADR-020 attested_by discipline |

### Sweep-level consequences

- v0.2 stdlib gains ~21 new family primitives (6 + 6 + 9)
- Camp gains substantial coordination vocabulary (prescriptive family)
- Three families together replace much of Slack/Asana/Jira/Linear/Notion translation tax
- `WitnessClass` and `SeedKind` enums added to antigen-core (new types)

### Resolves

- The temporal-arc gap in antigen's v0.1 vocabulary
- Independence-claim unenforceability (WitnessClass mechanizes the check)
- Fixed-seed bypass of `#[clonal]` (SeedKind enum + static analysis)
- Family overlap confusion (disambiguation table)

### What this ADR does NOT do

- Does NOT mechanically verify source-independence of `#[igg]` signers (nominal-only; known limitation)
- Does NOT claim biology proper grounds all 21 primitives uniformly (dual-axis honesty per naturalist)
- Does NOT claim temporal-arc is the only valid organizing principle

## ADR-024 Amendment 1 — `#[titer]` biology-grounding axis reassignment

**Status**: Ratified 2026-05-24.

**Amends**: ADR-024.

**Reason**: Outsider dust-finding (`3a3fada0`) surfaced a count drift in the
§Biology grounding block: header declared "(7 primitives)" for
immunology-proper but the list contained 8 entries. Naturalist (original
author of refinement 1 dual-axis honesty) evaluated the axis assignment for
the eighth entry, `#[titer]`, and judged it belongs in clinical-medicine,
not immunology-proper.

**Participants**: outsider (`3a3fada0` count-drift dust-finding); naturalist
(axis-assignment evaluation + amendment draft); aristotle (`aa805ca5`
process call: substantive axis-assignment changes warrant amendment, not
silent-fix).

**Related**: ADR-003 (biology metaphor); ADR-024 §Biology grounding
(parent dual-axis honesty refinement);
`docs/process.md` §Amendment-vs-fixup taxonomy.

### Finding

The §Biology grounding block carried a count drift since ratification
(2026-05-22): immunology-proper header declared 7 primitives but the list
contained 8 entries. The header count was correct; the LIST miscounted by
erroneously including `#[titer]`. Total counts cross-check: convergent (7)
+ recurrent (6) + prescriptive (9) = 22 primitives. After amendment:
immunology-proper (7) + clinical-medicine (11) + cognitive-organizational
(4) = 22. Pre-amendment sum was 8 + 10 + 4 = 22 — the sum-to-22 invariant
was preserved accidentally; the axis assignment for `#[titer]` was the
underlying error.

### Decision

**`#[titer]` is biology-grounded in clinical-medicine, not
immunology-proper.** Its operational discipline is prescriptive —
chart-documented monitoring of antibody concentration over time. Clinicians
ORDER titer tests to monitor specific patient cohorts. This is clinical-
medicine usage, not immunology-proper antibody-binding-affinity measurement.
The §Biology grounding lists at lines 5735–5737 are updated per this
amendment.

### What this amendment does NOT do

- Does NOT change `#[titer]`'s macro semantics, arg signature, or
  implementation.
- ~~Does NOT modify the family-grouping of `#[titer]` (still Family 3
  Prescriptive Work-Orchestration).~~ **KILLED by ADR-024 Amendment 3** — `#[titer]`
  is reclassified to the titer-witness kind; see §Titer reclassification below.
- Does NOT reassess axis assignments for other primitives.

---

## ADR-024 Amendment 2 — Recurrent Emergence macro arg-signatures (shipped shapes from parse.rs)

**Status**: Ratified 2026-05-27.

**Amends**: ADR-024 §Mechanics. Documents the concrete arg-signature shapes for the 6 Recurrent Emergence family macros as shipped in `antigen-macros/src/parse.rs`.

**Participants**: scientist (substrate read + amendment draft; navigator activity-log reference `936e678e`). Source of truth: `antigen-macros/src/parse.rs` struct definitions, NOT scout's narrower inference.

**Reason**: ADR-024 §Mechanics says "new schema structs (`DiagnosticEvidence`, `RecurrenceAnchor`, `PanelDeclaration`, etc.)" without documenting the concrete field shapes that shipped. The field-level arg signatures are load-bearing for adopters writing `#[itch]`, `#[recurrence_anchor]`, `#[crystallize]`, `#[chronic]`, `#[saturate]`, `#[strand]`. Documenting what shipped makes the ratified surface legible from the ADR alone.

### Change: document shipped arg shapes for Recurrent Emergence family

All six structs read from `antigen-macros/src/parse.rs` at HEAD:

**`#[itch(...)]`** — notice a potential recurrence pattern below the threshold for anchoring.
```
ItchArgs {
    name: Option<String>,         // human-readable label for this itch
    antigen: Option<syn::Path>,   // the antigen class this may be an instance of
    description: Option<String>,  // optional description of the observed pattern
    threshold: Option<String>,    // threshold hint (advisory; audit may use)
}
```
All fields optional. Positional leading antigen path accepted. Parse line: `parse.rs:2355`.

**`#[recurrence_anchor(...)]`** — cross-substrate recurrence threshold reached; surface for action.
```
RecurrenceAnchorArgs {
    antigen: Option<syn::Path>,   // positional-or-named antigen path
    instances: Option<u32>,       // count of observed instances
    since: Option<String>,        // date string (ISO 8601 recommended)
    rationale: Option<String>,    // why this constitutes a recurrence anchor
}
```
Leading positional antigen path accepted (peek-not-eq-sign detection). Parse line: `parse.rs:2476`.

**`#[crystallize(...)]`** — pattern has crystallized from itches into a named class.
```
CrystallizeArgs {
    name: Option<String>,                 // proposed class name
    from_itches: Vec<syn::Path>,          // the itch annotations that triggered this
    antigen: Option<syn::Path>,           // the antigen being crystallized toward
    summary: Option<String>,             // one-line summary of the class
}
```
Parse line: `parse.rs:2633`.

**`#[chronic(...)]`** — low-level persistent signal; NOT yet cross-substrate.
```
ChronicArgs {
    antigen: Option<syn::Path>,   // positional-or-named antigen path
    since: Option<String>,        // date string when first observed
    status: Option<String>,       // current management status
    managed_by: Option<String>,   // who/what is managing this chronic signal
}
```
Leading positional antigen path accepted. Parse line: `parse.rs:2762`.

**`#[saturate(...)]`** — a contributing signal that adds to a convergent-evidence cluster.
```
SaturateArgs {
    antigen: Option<syn::Path>,       // the antigen being saturated toward
    contributing_to: Option<String>,  // the cluster/anchor this contributes to
    description: Option<String>,      // description of this specific contribution
}
```
Parse line: `parse.rs:2874`.

**`#[strand(...)]`** — a named thread connecting multiple recurrence instances.
```
StrandArgs {
    name: Option<String>,           // strand name
    anchored_by: Vec<syn::Path>,    // the recurrence_anchor markers that anchor this strand
    description: Option<String>,    // description of the strand's through-line
}
```
Parse line: `parse.rs:2967`.

### What this amendment does NOT do

- Does NOT change any macro semantics, parse behavior, or audit behavior.
- Does NOT add new fields (pure documentation of shipped state).
- Does NOT supersede the §Mechanics paragraph in ADR-024 — it supplements it.

---

## ADR-024 Amendment 3 — `from_itches` is class-specific (lineage-aware): the recurrence-anchor noticing-precondition

**Status**: Proposed 2026-06-01 (campsite: `forward/adr024-from-itches-cross-class-ruling`).

**Amends**: ADR-024 §Mechanics (Recurrent Emergence family — the `#[recurrence_anchor]`
noticing-precondition). Resolves a question ADR-024 §Mechanics left silent, surfaced by an
adversarial follow-on and deferred from v0.2.

**Participants**: scientist (the well-posed question + the attack fixture + the scoped fix —
dogfood note `fd7c24c9`); adversarial (the follow-on that surfaced it on
`findings/recurrent-anchor-phantom-from-itches`); aristotle (the class-specific ruling +
lineage-aware refinement); navigator + team-lead (v0.3-non-blocking triage — the ATK-RECURRENT-7
phantom fix `8dfd4d5` stands regardless).

**Related**:
- ADR-024 §Mechanics (Recurrent Emergence — `#[itch]` / `#[recurrence_anchor]` /
  `#[crystallize]` temporal progression: notice → commit → crystallize).
- ADR-018 Amendment 1 (inheritance is provenance, not substitutability — grounds the lineage-aware
  refinement: a parent-class itch is legitimate upstream evidence for a child-class anchor).
- ADR-005 sub-clause F (the noticing-precondition is the validation check at the
  commitment-to-track trust boundary).
- `forward/three-valued-logic-api-boundary-layer` (the same cardinality-collapse shape: a
  cross-class itch is OUT-OF-FRAME — irrelevant evidence — not a weak-yes; collapsing it into
  satisfied-precondition is the gem at the precondition boundary).
- The dogfood `RatifiedSpecDriftFromImpl` class (this amendment realigns code with the
  doc-comment's already-stated intent — a substrate-alignment fix, not a new design choice).

**Campsite**: `forward/adr024-from-itches-cross-class-ruling` (the ruling note is the witness).

### Finding

`#[recurrence_anchor(X)]` carries a noticing-precondition: a commitment to track failure-class
`X`'s recurrence is only well-founded if `X` was previously *noticed* (an `#[itch(X)]` exists).
The audit fires `RecurrenceAnchorNoItchPrecondition` when no upstream noticing is found. ADR-024
§Mechanics was silent on whether a `from_itches` entry must name an itch for the anchor's **own**
antigen class. The shipped implementation (`audit.rs:3067-3072`) checks `from_itches` against the
**global** itch set (`itch_antigen_types`), so `from_itches = ["AntigenY"]` suppresses the
precondition for an `AntigenX` anchor whenever `AntigenY` has any itch anywhere — a **cross-class**
acceptance. The doc-comment for the audit-hint (`audit.rs:524-527`) already states the intended
semantics ("reference the same antigen type"); the implementation drifted wider than the doc.

### Decision

**`from_itches` is class-specific (lineage-aware).** A `from_itches` entry satisfies the
noticing-precondition for a `#[recurrence_anchor(X)]` if and only if it names `X` itself **or a
lineage ancestor of `X`** (via `#[descended_from]` `lineage_edges`), AND that named class has a
scan-resident `#[itch]`. A pure cross-class reference (`from_itches = ["AntigenY"]` where `AntigenY`
is neither `X` nor an ancestor of `X`) carries **no** precondition evidence and is treated as a
phantom — exactly as ATK-RECURRENT-7 treats non-scan-resident phantoms.

Grounds:

1. **The precondition's meaning is class-specific by construction.** "Noticing precedes
   commitment" means noticing *of the same failure-class*. Noticing `AntigenY` tells you nothing
   about whether `AntigenX` has recurred — it is not weaker evidence, it is *no* evidence for this
   anchor (a different failure-class entirely). This is the three-valued-logic gem at the
   precondition boundary: a cross-class itch is OUT-OF-FRAME (irrelevant), and the shipped code
   collapses out-of-frame-irrelevant into satisfied-precondition (the cardinality-collapse defect
   class).
2. **It realigns code with already-ratified intent.** The audit-hint doc-comment already says "the
   same antigen type"; the implementation drifted. This is fixing a `RatifiedSpecDriftFromImpl`
   substrate-alignment gap, not making a new design choice.
3. **Cross-class acceptance guts the precondition.** Under the shipped behavior, any single itch
   anywhere in the workspace suppresses the precondition for *every* anchor — reducing a per-anchor
   temporal-progression check to "does this workspace contain any itch at all?" (the vacuous-guard
   failure shape, `EmptySignersList` family).

**Lineage-aware refinement (the one legitimate "cross-class" case).** If the anchor is for a child
class and the itch is for a *parent* class, that is not cross-class — it is intra-lineage. Per
ADR-018 Amendment 1 (inheritance is provenance), noticing the parent recurring is legitimate
upstream evidence for committing to track the child (the child is a descended specialization;
parent-recurrence is evidence the lineage recurs). So the class match walks the lineage upward: an
ancestor-class itch satisfies. Pure cross-class (unrelated classes) never satisfies.

### Mechanics (the fix)

In `evaluate_recurrent_hints` (`audit.rs:~3066`), replace the global-set membership test with a
class-scoped one: `has_valid_from_itches` is true iff some `from_itches` entry equals the anchor's
`antigen_type` **or** a lineage ancestor of it (resolved from `report.lineage_edges`), **and** that
named class is in `itch_antigen_types`. `has_implicit_itch` (the anchor's own class has a
scan-resident itch) is unchanged. The precondition fires iff neither holds.

### Enforcement-tier / non-blocking status

- **Non-blocking for 0.2.0 stable** (navigator + team-lead confirmed): the ATK-RECURRENT-7 phantom
  fix (`8dfd4d5`) is correct regardless; this is a v0.3 *enhancement* (tighten cross-class to
  class-specific), not a correctness regression in shipped stable.
- Routes to pathmaker for the `audit.rs` change + the lineage-walk; the ruling + this amendment
  block are the design deliverable.

### What this amendment does NOT do

- Does NOT change `#[itch]` / `#[recurrence_anchor]` / `#[crystallize]` macro semantics or arg
  shapes (ADR-024 Amendment 2 shapes unchanged).
- Does NOT forbid multi-class recurrence *patterns* — it forbids one anchor's noticing-precondition
  being satisfied by an *unrelated* class's itch. Multi-class patterns are expressed as multiple
  anchors, each with its own class-specific (lineage-aware) precondition.
- Does NOT block 0.2.0 stable (v0.3 enhancement; the phantom fix already covers the stable-gate
  correctness).

---

## [ADR-025] Supply-Chain Defense Family: Antigens for Dependency-Boundary Risk in the 2026+ Threat Landscape

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named family; reframed from basophil/eosinophil to supply-chain in drill); adversarial (11 attacks, 4 BLOCKING absorbed); naturalist (cognate reframed to distributed-boundary-innate-immunity — NON-NEGOTIABLE); scout (supply-chain threat landscape research arc).

**Related**: ADR-001 Amendment 1 (structural memory); ADR-002 Amendment 2 (compose-or-compete); ADR-005 Amendments 2 & 3; ADR-009 (adoption gradient); ADR-019 (substrate-witness); ADR-020 (attestation); ADR-021 (oracle); ADR-022 (Stdlib-vs-Extension); ADR-027 (Mucosal boundary); ADR-028 (antigen-category).

**Implicit pattern elevated** (per ADR-004): the supply-chain trust boundary has been implicit in dev culture.

### Finding

**The 2026 supply-chain threat landscape**: chalk/debug/eslint-config incidents (2025) involved **content replacement at fixed version** — Cargo.lock pins VERSION but not CONTENT-HASH; lockfile pinning alone would not have prevented these. Proc-macro attack surface: proc macros execute at compile time with arbitrary code execution; compromised proc-macro is MORE dangerous than compromised regular dependency. AI-pair-generated `cargo add` adds dependencies without human review.

**Compose vs compete decision** (per AMEND-ADR-002 four-item substrate):
1. Adjacent tools: cargo-vet, cargo-deny, cargo-audit, cargo-crev, cargo-supply-chain, Sigstore, SLSA
2. Cohesion reason: unified failure-class-memory vocabulary vs fragmented cargo-vet/cargo-deny/cargo-audit translation
3. Adopter-experience differential segmented: (a) all-in antigen: 4-vs-1 differential; (b) antigen + cargo-audit: 2-vs-1; (c) non-antigen: irrelevant
4. Alternative path preserved: cargo-vet/cargo-deny/cargo-audit/cargo-crev continue for non-antigen adopters

**Decision**: COMPETE for segments (a) and (b); alternative preserved for (c).

### Decision

**Antigen ships a Supply-Chain Defense Family of 11 v0.2 stdlib antigens, a `cargo antigen verify` CLI subfamily, substrate-witness leaves over Cargo.toml/Cargo.lock/content-hash-registry/crates.io-metadata, and a tooling-phase progression from static-checks (v0.2) through behavioral-fingerprinting (v0.5+).**

**Eleven v0.2 stdlib antigens**:
1. `UnpinnedDependency` — Cargo.toml dep without exact-pin `=` version specifier
2. `UnpinnedTransitiveDependency` *(narrowed per B9-R)* — direct dep with `*`/`?` for its OWN dependencies (NOT "any transitive dep with non-exact pins" — ~100% false-positive avoided)
3. `UnattestedDependencyInclusion` — new dep added without team-attestation in commit history
4. `DependencyUpgradeWithoutDiffReview` — version bump without diff-reviewed attestation
5. `AutoDependencyChainWithoutPinning` — `?` or `*` anywhere in dependency tree
6. `MaintainerChangeWithoutReattestation` — crate ownership change; CI sequencing constraint: `verify maintainer-changes` MUST run BEFORE `cargo update`
7. `SuddenDependencyExpansion` — version bump with large LOC delta; complements `DependencyUpgradeWithoutDiffReview` for account-compromise defense
8. `UnsandboxedBuildScript` — `build.rs` from external dep not audited in sandbox
9. `UnsandboxedProcMacro` *(NEW per B3-R)* — external proc-macro dep not audited in sandbox; higher-risk than build.rs (runs in-rustc)
10. `PostInstallScriptInDependency` — external code running at install/build time
11. `ContentHashMismatch` *(NEW per B1-R, NON-NEGOTIABLE)* — content hash of published dep at recorded version differs from first-attestation hash. **This is the antigen for the chalk/debug/eslint-config attack**: Cargo.lock pins VERSION not CONTENT-HASH. Requires proactive first-attestation via `cargo antigen verify content-hash record` to activate.

**`cargo antigen verify` CLI subfamily** (cross-ADR substrate-grep verified clean): `deps`, `maintainer-changes`, `dep-attest <crate@version> --reviewable-artifact <PATH>` (REQUIRED), `dep-pin`, `content-hash <crate@version>`, `proc-macro-sandbox` (v0.4+), `sandbox` (v0.4+), `behavioral-diff` (v0.5+).

**Substrate-witness leaves** (additive per ADR-021): `dep_pinned(crate?)`, `dep_attested(crate, version, exact_version: bool = true)` *(default version-specific; requires non-empty `reviewable_artifact`)*, `maintainer_unchanged(crate, since_version)`, `content_hash_matches(crate, version)` *(NEW)*, `sandbox_clean(crate, sandbox_kind = "build" | "proc-macro")` *(NEW kind discriminator)*, `behavioral_diff_within` (v0.5+).

**Tooling-phase progression**:

| Phase | Version | Tooling |
|---|---|---|
| 1 | v0.2 | Static checks + content-hash recording/verification |
| 2 | v0.3 | Dependency-change diff display + sign-on-review + crates.io metadata |
| 3 | v0.4 | Sandboxed pre-update (build.rs + proc-macro) |
| 4 | v0.5+ | Behavioral fingerprinting + federated trust (Sigstore/SLSA) |

**Schema additions** (additive per ADR-021): `.attest/supply-chain/` for dep-attestation sidecars; `.attest/supply-chain/content-hash/<crate>@<version>.json`; `DepAttestation` schema with required `reviewable_artifact: PathBuf`; `ReviewScope ∈ { Full | Diff | BuildScriptOnly | ProcMacroOnly | MetadataOnly }`; `ContentHashRecord`.

**Audit-hint vocabulary** (15 total; cross-ADR substrate-grep verified): `unpinned-dependency`, `unattested-dependency-inclusion`, `dependency-upgrade-without-diff-review`, `maintainer-change-without-reattestation`, `maintainer-change-detected-after-cargo-update`, `sudden-dependency-expansion`, `unsandboxed-build-script`, `unsandboxed-proc-macro`, `post-install-script-in-dependency`, `content-hash-mismatch`, `content-hash-no-attestation`, `dep-attest-without-reviewable-artifact`, `crates-io-metadata-query-failed`, `dep-attestation-stale`, `auto-dependency-chain-without-pinning`.

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| `UnpinnedDependency` detection | scan-time | client | suppression via `#[antigen_tolerance]` requires rationale |
| `ContentHashMismatch` first-attestation | CLI-time (record) | client | requires proactive attestation; gap named |
| `ContentHashMismatch` verification | audit-time | client + CI | CI gate enforces |
| `MaintainerChangeWithoutReattestation` | audit-time | client + CI | CI sequencing constraint: must run BEFORE `cargo update` |
| `dep_attested` signing | CLI-time | client | requires `--reviewable-artifact`; rubber-stamp limitation named |
| `UnsandboxedBuildScript` + `UnsandboxedProcMacro` | audit-time (v0.4+) | client + CI | sandbox-detection limitations named |

**Biology grounding** (per naturalist reframe — NON-NEGOTIABLE): **Distributed-Boundary Innate-Immunity family** — multi-cell-type integrated system, NOT basophil/eosinophil (wrong shape). Per-primitive cognates: `UnpinnedDependency` ↔ PRR specificity discipline; `ContentHashMismatch` ↔ antigenic identity verification; `MaintainerChangeWithoutReattestation` ↔ transplant immunology re-attestation; `SuddenDependencyExpansion` ↔ Trojan-horse + MHC-I internal antigen presentation; `UnsandboxedBuildScript/ProcMacro` ↔ macrophage phagosome containment.

**Known limitations**: (1) rubber-stamp attestation; (2) solo-developer single-signer; (3) first-attestation gap for ContentHashMismatch; (4) sandbox-detection limitations (time-bomb attacks, environment-detection); (5) account-compromise without ownership change; (6) git-trust signing baseline; (7) maintainer-change detection timing; (8) dependency-confusion attacks (v0.3+ roadmap); (9) typosquatting partial (v0.3+ roadmap).

### Resolves

- The implicit-memory failure mode at the dependency boundary
- The supply-chain coverage gap in antigen-cohesion
- The AI-pair-generated `cargo add` failure pattern
- The content-replacement-at-fixed-version attack pattern (chalk/debug/eslint-config)
- The proc-macro attack surface gap

### What this ADR does NOT do

- Does NOT deprecate cargo-vet/cargo-deny/cargo-audit/cargo-crev
- Does NOT solve all supply-chain attacks (gaps acknowledged above)
- Does NOT enforce supply-chain discipline at Rust-compile time

---

## [ADR-026] VCS-Information-Loss Family: Structural Defense Against Git-History-Erasing Operations + Rollback-as-Triage Discipline

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named family + rollback-as-triage discipline in drill #74); adversarial (3 BLOCKING attacks absorbed: D1 commit-time detection, D2 force-with-lease coverage, D3 friction-vs-structural explicit choice); naturalist (cognate broadened to immune-memory-loss-mechanisms class; ForcePushErasingHistory ↔ Immune Amnesia centralized; dual-axis grounding for rollback-as-triage — NON-NEGOTIABLE).

**Related**: ADR-001 Amendment 1 (structural memory; preserving through VCS ops); ADR-002 Amendment 2 (compose-or-compete); ADR-005 Amendments 2 & 3; ADR-019 (substrate-witness, VCS-trailer-based); ADR-022 (Stdlib-vs-Extension); ADR-023 (rollback-as-triage uses `#[orient]`-shape); ADR-028 (antigen-category — most members substrate-alignment).

**Implicit pattern elevated** (per ADR-004): git operations that erase information have been governed by team-convention; the structural why-this-was-done lives in commit messages that get rewritten.

### Finding

Modern git workflows include force-push, branch-deletion, rebase, squash-merge, amend — each has legitimate use cases AND can erase load-bearing history.

**The central cognate: ForcePushErasingHistory ↔ Immune Amnesia (measles)** (Mina et al. 2015, Science): measles virus infects memory lymphocytes; post-measles patients show increased susceptibility to other pathogens for 2-3 years. CATASTROPHIC LOSS of MEMORY-CARRYING substrates with DOCUMENTED HARM and STRUCTURAL DEFENSE patterns. Biology PREDICTS the failure mode and defense pattern.

**Rollback-as-triage discipline** (per drill #74): rollback-as-treatment requires triage-commit-first discipline: commit triage decision THEN do rollback. **Dual-axis grounding** (per naturalist): this discipline is CLINICAL-MEDICINE grounded (informed consent + chart documentation). Immune biology has NO analog to "log rationale before acting." This honest dual-axis acknowledgement parallels ADR-024's grounding split.

**Detection model** (per adversarial D1): `RollbackWithoutTriageCommit` cannot be detected by post-hoc history inspection (`git reset --hard` removes traces). MUST operate at COMMIT-TIME via hooks.

*(Amendment 3 — 2026-05-24: rollback detection uses AUTHOR-DECLARATION (Algorithm C), not diff-similarity (Algorithm B). The commit-time hook applies a three-step decision tree: (1) commit message contains git-revert metadata (`This-reverts-commit-X` or `Revert-Of:` trailer) AND no `Triage-Decision:` trailer → fire `RollbackWithoutTriageCommit` hint; (2) commit carries a `Triage-Decision: <value>` trailer → validate value is a `TriageDecision` enum variant; fire `vcs-rollback-triage-chain` witness check *(Amendment 4 — 2026-05-24: corrected from "commit declares `#[triage_commit]`" — codebase-presence semantics; trailer-on-commit is the correct commit-intent signal per ADR-026 §M3)*; (3) otherwise → audit defers; residual risk is that manual inverse cherry-picks without any declaration are undetectable at commit-time. This residual risk is NAMED and EXPLICIT per friction-only philosophy: making bad behavior deliberate rather than impossible. Adopters requiring diff-similarity detection must opt in via `cargo antigen vcs --diff-similarity-check` (v0.3+ experimental path). Campsite: `v02-impl-vcs-info-loss`.)*

**Enforcement model** (per adversarial D3): client-side hooks are bypassable via plumbing commands. The ADR ships:
- **Friction-only mode** (default v0.2): client-side hooks + audit-time; makes bad behavior DELIBERATE rather than ACCIDENTAL; explicitly NOT preventive
- **Structural mode** (server-side; v0.2.1+): pre-receive hooks; requires adopter to control git remote

*(Amendment 3 — 2026-05-24: when `ServerSideEnforcementMode::Structural` is declared, the audit pipeline MUST evaluate `vcs_server_side_enforcement_active(repo, antigen_name)` at audit-time. If the witness returns false, emit `vcs-enforcement-structural-mode-declared-but-not-active` hint and demote the audit-tier to FrictionOnly for that antigen. Without this guard, Structural mode is an UNVERIFIED CLAIM. Network error during witness evaluation emits a separate `vcs-server-config-check-failed` hint (network-error != structural-not-active). This witness is v0.2.1+ alongside Structural mode itself; v0.2 ships friction-only only. Campsite: `v02-impl-vcs-info-loss`.)*

### Decision

**Antigen ships an 11-antigen VCS-Information-Loss Family + rollback-as-triage discipline (via new `#[triage_commit]` primitive) + git-trailer-based substrate-witnesses + commit-hook detection mechanism + `cargo antigen vcs` CLI subfamily. Detection is friction-only by default; server-side enforcement is the path to structural-mode.**

*(Amendment 1 — 2026-05-24: rollback-as-triage uses a dedicated `#[triage_commit]` macro, NOT an `#[orient]` extension. The orient-dual-signature analysis [see fixup-orient-dual-signature campsite] established that #[orient] is passive-context-only; decisional/committal fields require a structurally distinct primitive. ADR-023 §orient semantics are preserved unchanged.)*

**Eleven v0.2 stdlib antigens**: `RollbackWithoutTriageCommit`, `RefactorWithoutPreservationOfWhy`, `BranchDeletionWithoutAttestation`, `ForcePushErasingHistory` (covers both `--force` AND `--force-with-lease` per D2), `SquashMergeLosingIntermediateState`, `CherryPickLosingOriginalContext`, `RebaseRewritingHistoryWithoutLog`, `UnpushedBranchWithSubstantiveWork`, `StashedWorkAbandoned`, `MergeConflictResolutionWithoutAttestation`, `AmendedCommitWithoutOldHashPreservation`.

**Biology per-primitive cognates**: `ForcePushErasingHistory` ↔ **Immune amnesia (measles)** — central foundational family cognate; `RefactorWithoutPreservationOfWhy` ↔ Original antigenic sin; `SquashMergeLosingIntermediateState` ↔ Affinity maturation history loss; `CherryPickLosingOriginalContext` ↔ Class-switching context loss; `RebaseRewritingHistoryWithoutLog` ↔ V(D)J recombination without per-cell record; `StashedWorkAbandoned` ↔ Anergy/primed-without-activation.

**Rollback-as-triage primitive** (new `#[triage_commit]` macro; structurally distinct from `#[orient]` per Amendment 1):

```rust
#[triage_commit(
    triage_decision = TriageDecision::Red,
    rollback_target = "abc1234",
    triaged_by = "navigator",
    rationale = "vital metric regression confirmed via #84; rolling back to last-known-good",
    rollback_due_within_minutes = 30,
)]
fn _triage_marker_do_not_remove() {}
// Followed by rollback commit with: Triage-Decision: <triage-commit-sha>
```

**Schema additions** (additive per ADR-021): `TriageDecision` enum (`Black | Red | Yellow | Green | White`); `VcsAttestation`; `.attest/vcs/` convention; `ServerSideEnforcementMode` enum (`FrictionOnly | Structural`).

**Git-trailer-based substrate-witnesses** (per ADR-019): `vcs_trailer_present(trailer_name)`, `vcs_attest_branch_deletion(branch, by_role)`, `vcs_rollback_triage_chain(commit)` (commit-time; per D1), `vcs_server_side_enforcement_active(repo, antigen_name)` *(NEW — checks remote configuration)*.

**CLI subfamily** (cross-ADR substrate-grep clean): `cargo antigen vcs {scan, check-commit, attest, rollback-prepare, branch-archive, install-hooks, install-server-hooks}`.

**Audit-hint vocabulary** (14 new hints prefixed `vcs-`; Amendment 3 adds 2): `vcs-rollback-without-triage-commit`, `vcs-force-push-erased-substantive-history` (covers both --force and --force-with-lease), `vcs-enforcement-friction-only-no-server-hook`, `vcs-enforcement-structural-mode-declared-but-not-active` *(Amendment 3)*, `vcs-server-config-check-failed` *(Amendment 3)*, and others.

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| `RollbackWithoutTriageCommit` | commit-time (pre-commit hook) | client (friction) OR server (structural) | client-side bypassable via `git commit --no-verify` |
| `ForcePushErasingHistory` (incl. --force-with-lease) | push-time | client (friction) OR server (structural) | `receive.denyNonFastForwards = true` on remote for structural |
| `BranchDeletionWithoutAttestation` | branch-delete-time | client OR server | `git update-ref -d` bypasses client-side |
| Other 8 members | audit-time + commit-time | client (friction) + audit visibility | friction-only; audit-time hint surfaces loss |

### Sweep-level consequences

- v0.2 stdlib gains 11 VCS-info-loss antigens
- New `#[triage_commit]` primitive carries rollback-as-triage fields (Amendment 1: `#[orient]` NOT extended)
- Adopter rollback discipline: triage-commit-before-rollback becomes structural practice
- Conceptual alignment with camp `triage` primitive (5-color taxonomy + treatment-discipline semantics); cross-tool schema alignment is v0.3+ research arc (per Amendment 2)

### Resolves

- The git-information-loss failure mode (reflog + reviewer memory only)
- The rollback-without-structural-why pattern
- The detection-via-history-inspection fundamental flaw (commit-time hook required per D1)
- The --force vs --force-with-lease false distinction (both covered uniformly per D2)
- The client-side hook bypass overclaim (friction-only named explicitly + server-side path provided)

### What this ADR does NOT do

- Does NOT prevent VCS operations
- Does NOT replace `git reflog` for local recovery
- Does NOT enforce structural mode without adopter setup
- Does NOT overclaim — v0.2 default is friction-only, named explicitly

---

## [ADR-027] Mucosal Boundary Taxonomy + Mapping Discipline

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named boundary taxonomy + mapping discipline in drill); adversarial (E1 + E2 absorbed: `#[mucosal_delegate]` primitive + missing boundary types); naturalist (don't overclaim per-variant biology grounding — NON-NEGOTIABLE; trafficking-integration as v0.3+ research arc); scout (prior-art on boundary mapping).

**Related**: ADR-002 Amendment 2 (compete decision); ADR-003 Amendment 1 (biology); ADR-006 Amendment 1 (stdlib); ADR-019 (substrate-witness); ADR-022 (Stdlib-vs-Extension); ADR-025 (supply-chain — `DependencyImport` boundary); ADR-028 (antigen-category — most members substrate-alignment).

**Implicit pattern elevated** (per ADR-004): codebases have boundaries adopters don't enumerate. Unknown boundaries are where attacks land.

### Finding

**Boundary types in modern software** (v0.2 + v0.3+ scope marks): Imports/exports, APIs (HTTP/gRPC), MCPs, External links, iframes, Databases, Cross-service, Cross-computer executables, Untrusted 3rd party code (see ADR-025), PRs with external-boundary touch, User input — plus NEW per adversarial E1: **Filesystem/path construction** (path-traversal via user input), **Environment variables** (env-var injection), **Shell argument construction** (user-input → shell arg). WebSocket + CI/CD pipeline inputs deferred to v0.3+.

**The critical insight**: sanitization presence ≠ correctness. Discipline fires at boundary regardless.

**The split-defense problem** (per adversarial E2): `#[mucosal]` declared on the outermost function (the actual boundary), but sanitization performed by an inner callee. Without explicit delegation primitive, audit might falsely report the boundary as defended. Resolution: `#[mucosal_delegate]` primitive.

### Decision

**Antigen ships a Mucosal Boundary Taxonomy (15 variants in v0.2; 2 more documented for v0.3+) + `cargo antigen mucosal-map` discovery tool + `#[mucosal]` and `#[mucosal_delegate]` macros + per-boundary stdlib antigens. Per-variant tissue-mapping IS NOT biology-grounded (per naturalist); biology grounds the tier-claim + 4 functional disciplines.**

**`#[mucosal(kind = MucosalKind::..., rationale = "...")]`** — declarative marker for boundary defense.

**`#[mucosal_delegate(boundary = MucosalKind::..., handled_by = "...", rationale = "...")]`** *(NEW per E2-R)* — when boundary discipline is delegated to a callee. The delegate handler MUST itself carry `#[mucosal(...)]`. Audit emits `mucosal-discipline-delegate-target-missing` if handler doesn't exist; `mucosal-discipline-delegate-target-not-mucosal` if target lacks corresponding declaration.

**`MucosalKind` enum** (sealed v0.2 set; per ADR-001 Amendment 1 C6): `Import, ApiRequest, ApiResponse, McpInvocation, ExternalLink, Iframe, DatabaseQuery, CrossService, SubprocessLaunch, DependencyImport, PrBoundary, UserInput, FilesystemPath, EnvironmentVariable, ShellArgument` (15 variants; v0.3+ adds `WebSocketStream`, `CiCdPipelineInput`).

*(Amendment 1 — 2026-05-24: sealed set revised to 13 variants: `Import` removed (redundant/ambiguous vs `DependencyImport`); `PrBoundary` removed (process event, not runtime data-flow). Authoritative 13-variant set: `ApiRequest, ApiResponse, McpInvocation, ExternalLink, Iframe, DatabaseQuery, CrossService, SubprocessLaunch, DependencyImport, UserInput, FilesystemPath, EnvironmentVariable, ShellArgument`. Inclusion discipline: type-of-data-crossing-boundary axis; see structured Amendment 1 block for criteria.)*

**`cargo antigen mucosal-map`**: `mucosal-map`, `mucosal-map --kind <kind>`, `mucosal-map --undefended` — walks codebase; surfaces boundaries.

**Schema additions** (additive per ADR-021): `MucosalDeclaration { kind, rationale, handled_by? }`; `.attest/mucosal/map.json`.

**Audit-hint vocabulary** (cross-ADR substrate-grep verified): `mucosal-boundary-undefended`, `mucosal-kind-mismatch`, `mucosal-discipline-delegated`, `mucosal-discipline-delegate-target-missing`, `mucosal-discipline-delegate-target-not-mucosal`, `mucosal-rationale-insufficient`.

*(Amendment 1 — 2026-05-24: 5 new hints; handled_by typed as syn::Path; delegate audit logic gains three-tier kind-matching diagnosis; #[mucosal_tolerant] primitive added with 4 tolerance-specific hints + mucosal-map --tolerant flag. Full 11-hint vocabulary in Amendment 1 structured block.)*

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| `#[mucosal]` declaration | parse-time + audit-time | client + CI | absent declaration surfaces via `mucosal-map --undefended` |
| `#[mucosal_delegate]` delegate-target check | audit-time | client + CI | substrate-witness validates handler exists + is mucosal-declared |
| Boundary detection | scan-time | client | static analysis best-effort; missed boundaries = false-negatives |
| MucosalKind set | parse-time (enum check) | — | adding kinds requires ADR amendment per ADR-001 Amendment 1 C6 |

**Biology grounding** (per naturalist — NON-NEGOTIABLE): the 15-variant MucosalKind taxonomy is **software-engineering scope-selection**, NOT biology-grounded. Biology has ~5-7 anatomical mucosal sites organized by ANATOMICAL LOCATION; the software taxonomy is organized by DATA-FLOW TYPE. Per-variant tissue-mapping (GALT-to-DatabaseQuery, etc.) fails at every cell.

**What biology DOES ground (Class 1)**: (1) mucosal-tier-as-distinct discipline; (2) tolerogenic-by-default + selective-response; (3) prevention-at-boundary (secretory-IgA-style exclusion); (4) trafficking-integration insight (CCR9/CCR10 link GALT and respiratory immunity — opens v0.3+ research arc for cross-MucosalKind shared validation libraries).

**Compose vs compete decision**: COMPETE per AMEND-ADR-002 (cohesion + opinion + integrated vocabulary; alternative path preserved).

### Sweep-level consequences

- v0.2 stdlib gains `#[mucosal]` + `#[mucosal_delegate]` + `#[mucosal_tolerant]` macros (Amendment 1 adds `#[mucosal_tolerant]`)
- MucosalKind enum with 13 variants (Amendment 1: 15 → 13; Import + PrBoundary removed)
- `cargo antigen mucosal-map` CLI tool (Amendment 1 adds `--tolerant` sub-flag)
- Cross-reference to ADR-025 for DependencyImport boundary

### Resolves

- The missing-boundary-enumeration failure mode
- The false-positive defense claim pattern (per E2: explicit delegate primitive)
- The missing boundary types gap (per E1: filesystem/path, env vars, shell-args v0.2; WebSocket + CI/CD v0.3+)

### What this ADR does NOT do

- Does NOT claim per-variant biology grounding (honest silence about anatomy-vs-data-flow mismatch)
- Does NOT ship trafficking-integration in v0.2 (v0.3+ research arc)
- Does NOT claim sanitization-presence implies correctness

---

## [ADR-028] Antigen-Category Taxonomy: Substrate-Alignment vs Functional-Correctness as First-Class Distinction

**Status**: Ratified 2026-05-22.

**Participants**: aristotle (draft + Phase 1-8 + revision); Tekgy (named the distinction in drill); observer (substrate-alignment discipline source); naturalist (operational-substrate-primary correction; biology-as-documentation-cognate clarified — NON-NEGOTIABLE); adversarial (F1 + F2 absorbed: hybrid miscategorization defense + strict enforcement chosen).

**Related**: ADR-001 Amendment 1 (structural memory carriers); ADR-005 Amendment 2 (rationale as trust-extension); ADR-019 (substrate-witness vs code-witness already operationalizes this split); ADR-022 (Stdlib-vs-Extension); and all v0.2 family ADRs (ADR-023 through ADR-027 carry category metadata).

**Implicit pattern elevated** (per ADR-004): substrate-alignment vs functional-correctness has been implicit in antigen's architecture from day one — `cargo antigen scan` does substrate-alignment work, `cargo antigen audit` does functional-correctness work, observer-role catches substrate-alignment failures, adversarial+scientist+pathmaker roles catch functional-correctness failures.

### Finding

**The two categories**:
- **SubstrateAlignment**: antigen fires when a REPRESENTATION diverges from actual state. "This says X but actual state is Y." Witness checks the substrate. Example: `UnpinnedDependency` — Cargo.toml says `dep = "^1.0"` when it should say `dep = "=1.0.3"`.
- **FunctionalCorrectness**: antigen fires when a VERB produces the wrong output. "This claims to do X but produces Y." Witness exercises behavior. Example: `PanickingInDrop` — Drop impl panics under some inputs.

Hybrid: `CampsiteOpen` (sidecar must exist AND signatures must cryptographically validate); `UnsandboxedBuildScript` / `UnsandboxedProcMacro` (no sandbox attestation AND the code might actually do bad things).

### Decision

**Antigen declarations gain a REQUIRED `category` field carrying one or both of `AntigenCategory::SubstrateAlignment` / `AntigenCategory::FunctionalCorrectness`. The category is STRUCTURALLY ENFORCED (Option A STRICT): category determines minimum witness requirements. Hybrid antigens require BOTH witness types verified for full immunity.**

**Enforcement-model — Option A (STRICT) chosen** (per F2-R): advisory category makes "first-class metadata shaping witness type, audit layer, lifecycle phase, responder role" an overclaim.

- `category = SubstrateAlignment` requires at least one substrate-witness predicate leaf
- `category = FunctionalCorrectness` requires at least one code-witness predicate leaf
- Hybrid requires BOTH witness types
- Category mismatch vs predicates FAILS validation at parse-time / audit-time

*(Amendment 2 — 2026-05-24: The "substrate-witness predicate leaf" requirement above applies to the WITNESS layer — either an audit-pipeline evaluator that reads substrate state directly (e.g., `DepPinnedState`, `ContentHashState`), or a fingerprint using substrate-witness leaves from ADR-019's grammar. It does NOT require the fingerprint scan-side pattern itself to be a substrate-witness leaf. Fingerprint finds the declaration sites; witness evaluates substrate state. Concretely: `doc_contains("ADR-025")` is a valid scan-side fingerprint for a `SubstrateAlignment` antigen whose witness is the `dep_pinned()` audit-pipeline evaluator — the fingerprint locates the antigen declaration, not the vulnerability site. This interpretation was confirmed by team-lead 2026-05-24; campsite `adr028-predicate-leaf-clarification`. Enforcement of this requirement at parse-time depends on the category-vs-predicate-type cross-check (per §F1-R Hybrid miscategorization defense below), tracked in campsite `v02-impl-category-witness-cross-check`. Until that cross-check ships, the substrate-witness leaf requirement is advisory at parse-time and enforced only at audit-time.)*

**Hybrid miscategorization defense** (per F1-R): parse-time category-vs-witness-type cross-check emits `antigen-category-claim-inconsistent-with-predicate-type` if declared category doesn't match predicate type. Hybrid antigens require BOTH axes EVALUATED at audit time; missing axis = UNVERIFIED.

**Per-site `category_required` escape hatch is REMOVED** in this revision — escape hatches defeat the strict-enforcement value.

**v0.2 backward-compat**: v0.1 antigens lacking `category` field default to `vec![FunctionalCorrectness]` + emit `antigen-category-defaulted-implicit-functional` migration hint. v0.2+ NEW declarations: absence emits the same migration hint at audit time; parse-time hard error is v0.2.x once the migration tool (`cargo antigen migrate categories`) exists. *(Amendment 4 — 2026-05-24: corrected "absence is hard-error" to reflect shipped G1 behavior: migration hint, not hard error. Hard error deferred to v0.2.x.)*

**Macro syntax**:

```rust
#[antigen(
    name = "UnpinnedDependency",
    category = AntigenCategory::SubstrateAlignment,
    family = "supply-chain",
    references = [...],
)]

#[antigen(
    name = "UnsandboxedBuildScript",
    category = [AntigenCategory::SubstrateAlignment, AntigenCategory::FunctionalCorrectness],
    family = "supply-chain",
    references = [...],
)]
```

**Schema additions** (additive per ADR-021): `AntigenCategory` enum: `SubstrateAlignment | FunctionalCorrectness` (sealed; variants require ADR amendment per ADR-001 Amendment 1 C6); `Antigen.category: Vec<AntigenCategory>` (required; non-empty; parse-time validation).

**Audit-hint vocabulary** (cross-ADR substrate-grep verified): `antigen-category-defaulted-implicit-functional`, `antigen-category-missing-explicit`, `antigen-category-mismatch-witness-type`, `antigen-category-claim-inconsistent-with-predicate-type`, `antigen-category-hybrid-incomplete-evidence`.

**CLI integration**: `cargo antigen scan --category substrate-alignment`, `cargo antigen audit --category functional-correctness`, `cargo antigen migrate categories` (v0.2.1+).

**§Enforcement-Surface**:

| Mechanism | Enforcement-Tier | Enforcement-Scope | Status |
|---|---|---|---|
| `category` field on `#[antigen]` (v0.2+) | audit-time: migration hint for v0.1 carryover; hard error for new declarations (v0.2.x) | client | v0.2 ships hint; hard error deferred pending migration tool *(Amd 4)* |
| Category-vs-witness-type cross-check | audit-time (ADVISORY; CI-gateable) | client + CI | v0.2 shipped at audit layer (Amendment 3) *(Amd 3+4)* |
| Hybrid incomplete-evidence | audit-time (partial coverage signal) | client + CI | v0.2 shipped (G3) *(Amd 4)* |
| v0.1 backward-compat default | parse-time migration hint | client | v0.2 ships; v0.3+ deprecation removes default |

**Biology grounding** (per naturalist — NON-NEGOTIABLE): the category distinction is **OPERATIONALLY substrate-grounded**, NOT biology-grounded. Biology provides an approximate documentation cognate (Class 2-3): pattern-recognition (PRRs, BCRs, TCRs) ↔ substrate-alignment; effector-function (cytokine release, cell killing) ↔ functional-correctness. The biology cognate is documentation-aid, not load-bearing prediction. The OPERATIONAL substrate is: observer-role catches substrate-alignment; adversarial+scientist+pathmaker catch functional-correctness; substrate-witnesses vs code-witnesses (ADR-019) already operationalize this split.

### Sweep-level consequences

- v0.2+ new declarations REQUIRE explicit category (hard error at parse-time)
- v0.1 carryover backward-compat with migration hint + tool
- Hybrid antigens require both witness types verified at audit-time
- Category becomes the primary routing metadata for team roles and tooling

### Resolves

- The unnamed-but-load-bearing distinction between substrate-alignment and functional-correctness
- Hybrid miscategorization escape (per F1: category-vs-predicate cross-check)
- Advisory-vs-structural category enforcement choice (per F2: Option A STRICT chosen)

### What this ADR does NOT do

- Does NOT claim biology grounds the category mechanic as primary substrate (operational substrate is primary)
- Does NOT permit advisory-only category declarations
- Does NOT permit per-site `category_required` escape hatches (removed in revision)

---

## [ADR-029] Immunity Is Observed, Not Declared: `#[defended_by]` + `#[presents]` Evidence Extension

**Status**: Ratified 2026-05-27.

**Participants**: antigen-dx-dogfood team (scientist primary drafter; aristotle Phase-1-8
PASSED; naturalist biology-check PASSED; adversarial gate PASSED; scientist consistency
review COMPLETE; all four ceremony signers).

**Related**: ADR-004 (implicit-to-explicit elevation), ADR-005 (sub-clause F), ADR-013
(phantom-type witnesses), ADR-019 (substrate-witness predicate family), ADR-020 (attestation
primitive), ADR-006 (recognition-not-design).

**Supersedes**: `#[immune]` as a user-facing primitive (backward-compatible deprecation path).

**Ceremony campsite**: `ceremony/ratify-adr-029-immune-observed`.

### Finding

`#[immune(X, witness = fn)]` and `#[immune(X, requires = pred)]` — the two mutually-exclusive
channels — each bundle concerns that should be separate:

**Code-tier (`witness = fn`) bundles:**
1. **Immunity-claim** — "this site is immune to failure-class X." This is a verdict. Code
   sites don't hold verdicts; audit tools do. Encoding a verdict as an attribute makes
   it static (true when written; may not be true now) and removes the audit tool's role
   as the single authoritative verdict issuer.
2. **Witness registration** — the pointer from defense-claim to the test function that
   defends it. This is a registration of evidence, not a verdict.

**Substrate-tier (`requires = pred`) bundles:**
1. Same **immunity-claim** (the verdict that shouldn't live at the code site).
2. **Substrate-witness binding** — a contextual predicate that the audit must verify against
   substrate (sidecars, git, docs) to issue a verdict. This is NOT an immunity-claim; it
   is a constraint on what the audit must check.

Both channels also carry:

3. **Site identity** — the `(antigen, source-file, item-path)` triple that sidecars anchor
   to. This survives `#[immune]` deprecation by moving to the presents-site, which is already
   the natural failure-locus.

**The core principle (Tekgy, seed note):** Code declares structural facts. `cargo antigen
audit` declares verdicts. Code never claims "I am immune to X" — the tool reports:
"defended at tier T / not defended, gaps at sites A, B, C."

**The shift is ~80% already shipped** (aristotle T7): `#[immune]` today only *declares*
a defense claim+evidence-pointer; `cargo antigen audit` already *issues* the verdict
(`audit.rs:8`: "meaningful only if Y resolves"). This ADR's novel content is (a) the
verdict-language change and (b) the witness→test-side migration for code-tier sites.

**Aristotle Phase-1-8 outcome**: PASS WITH AMENDMENTS. Ship R5 not R4: two primitives,
not three. `#[site_binding]` dropped; `requires=` folds into `#[presents]`. Migration is
asymmetric: code-tier and substrate-tier sites migrate differently. Three voids seeded for
v0.3+ (V1 conjunctive defense, V2 two-evaluation-mode, V3 vulnerability layer).

### Decision

**Immunity is observed, not declared.**

The `#[immune]` macro is deprecated. Its two channels migrate to two primitives:

#### Primitive 1: `#[defended_by(X)]` on test/proptest functions (code-tier migration)

A test function (or proptest property) that defends against failure-class X annotates itself:

```rust
#[test]
#[defended_by(ParallelStateTrackersDiverge)]
fn bijection_test_audit_hints_const_matches_enum() {
    // exercises both sides of the parallel state
}
```

`cargo antigen audit` scans for these markers and cross-references to the presents-sites
they cover. The test declares *what it defends*; the audit determines *whether it defends it*.

**Why `#[defended_by(X)]` is declared, not computed** (aristotle discriminator, 2026-05-27):
Detection is structural — the vulnerability's shape IS the vulnerability; a fingerprint recovers
which failure-class a site presents because presentation is identity-with-structure. Defense is
semantic — a test's *intent toward a failure-mode* is not carried by its structure; coverage
recovers "this test touches X-presenting code," not "this test *exercises the failure-mode* X."
The §honest-semantic-gap proves this: a hollow-wrapper test and a genuine defense have the same
structural signature. `#[defended_by(X)]` declares the irreducibly-human input (intent,
unrecoverable from structure); the audit still issues the verdict (defended/undefended) by
checking whether the intent's circuit is wired. This is NOT `#[immune]`'s dust moved one node:
`#[immune]` asserted the verdict ("I am immune"), usurping the audit's job; `#[defended_by(X)]`
asserts the intent ("this test aims at X"), which is irreducibly human. The audit still owns
the verdict. Division of labor is exactly ADR-029's principle, correctly placed.

**Scope**: `#[defended_by(X)]` applies to **code-tier witnesses only** — `#[test]` functions
and proptest properties. Phantom-type witnesses (type-system-resident, `WitnessTier::FormalProof`)
do NOT use this attribute; see §ADR-013.

**Name rationale**: avoids collision with `#[igg]`'s `witnesses = [...]` field (a plural
field for re-attestation history). `witnesses` as a top-level attribute and a field inside
`#[igg]` are not a parser collision (different namespaces) but ARE a glossary violation —
two `witnesses` meanings in the same vocabulary. `#[defended_by(X)]` is unambiguous.

#### Primitive 2: site-attached evidence folds into `#[presents]` (substrate-tier and phantom-tier migration)

When a presents-site carries defense evidence that is NOT a test function — a substrate
predicate or a phantom-type proof — the evidence attaches directly to `#[presents]`:

```rust
// Substrate-predicate (was requires= in #[immune]):
#[presents(UnpinnedDependency, requires = "cargo-lock-committed")]
fn add_dependency(...) { ... }

// Phantom-type proof (was witness= phantom in #[immune]):
#[presents(DropPanicClass, proof = NonPanickingProof::<T>::verified)]
fn make_droppable() { ... }
```

**Discriminator**: evidence belongs WHERE it is. A substrate predicate or phantom proof IS
at the site — it lives on `#[presents]`. A test fn IS elsewhere — it annotates via
`#[defended_by]`. Both forms are site-attached evidence with no separate test to annotate.

The `requires =` predicate replaces `requires =` inside `#[immune]`; the `proof =`
expression replaces `witness = <phantom>` inside `#[immune]`. Both move from the
(now-deprecated) immune-site to the presents-site — already the natural failure-locus,
and already present at every target site (aristotle A7-verified against full workspace grep).

**Note**: `PresentsArgs` currently accepts only the antigen path (`parse.rs:108`). Adding
`requires =` and `proof =` are additive fields on an existing attribute — one parse path
extended, not a new proc-macro invented. Recognition-over-design (ADR-006): the existing
carrier already exists at every target site.

**`#[site_binding]` dropped**: the three-primitive R4 design (scientist's original draft)
invents a new attribute to carry what `#[presents]` can already hold. Aristotle Phase-1-8
ruling: adopt R5 (two primitives), not R4 (three). `#[site_binding]` is not ratified.

**Full R5 model (unified)**:
- `#[presents(X)]` — detection/locus (always)
- `#[presents(X, requires=P)]` — + substrate-predicate evidence (substrate-tier)
- `#[presents(X, proof=PhantomPath::<T>::ctor)]` — + type-system-proof evidence (phantom-tier)
- `#[defended_by(X)]` on `#[test]`/proptest fn — code-tier runtime evidence

#### Sidecar identity anchors to the presents-site (consequence, not a new primitive)

Sidecar files (`.attest/`) anchor to the `(antigen, source-file, item-path)` of the
`#[presents]` site, not of a separate `#[immune]` site. The presents-site IS the identity.
No new primitive needed; the identity surface was always the presents-site.

#### Audit verdict language

The audit produces structured verdicts per presents-site per antigen:

- `defended` — witness registered at tier T (T = Reachability / Execution / FormalProof)
- `undefended` — no witness found (code-tier: no `#[defended_by]` cross-reference;
  substrate-tier: no `requires=` predicate passes)
- `substrate-gap` — `requires=` predicate not satisfied in current substrate
- `partial` — witness registered at lower tier than `min_tier` declared on the site
  (requires `#[presents(X, ..., min_tier = Execution)]`; deferred to v0.3 — see §L3)

The audit NEVER says "immune to X." The verdict is always about *the state of the defense
circuit*, not about whether the failure mode can fire.

### Mechanics

#### What antigen-macros changes

- `#[immune]`: deprecated, still parses (no breaking change), emits compiler warning
  pointing to migration guide
- `#[defended_by(X)]`: new attribute on `#[test]` / proptest items (code-tier only)
- `#[presents]`: `PresentsArgs` (`parse.rs:108`) gains optional `requires = <predicate>`
  field, optional `proof = <expr>` field (phantom-tier evidence), and optional
  `min_tier = <tier>` field (deferred to v0.3; see §L3)

**`PresentsArgs` target shape (for implementer — pre-build assumption document)**:
```rust
pub struct PresentsArgs {
    pub antigen: Path,
    pub requires: Option<(RequiresExpr, Span)>,   // same type as ImmuneArgs.requires
    pub proof: Option<Expr>,                        // same type as ImmuneArgs.witness
    pub min_tier: Option<MacroWitnessTier>,         // new local mirror type (v0.3)
}
```

`MacroWitnessTier` mirrors `MacroAntigenCategory` pattern (`parse.rs:48-77`): proc-macro
crates cannot dep on antigen (circular), so a local mirror enum is required.

#### What cargo-antigen audit changes

- **Scan phase**: collect `#[defended_by(X)]` registrations alongside `#[presents(X)]` sites,
  `#[presents(X, requires=P)]` substrate-predicates, and `#[presents(X, proof=...)]`
  phantom-proof references
- **Cross-reference**: for each presents-site, find registered code-tier witnesses that cover
  it (via `#[defended_by]`) + evaluate substrate-tier predicate (via `requires=` if present)
  + recognize phantom-tier evidence (via `proof=` expression shape, same `audit.rs:95-98`
  logic as today)
- **Tier detection** (enum ordinals: `None=0`, `Reachability=1`, `Execution=2`,
  `BehavioralAlignment=3` reserved, `FormalProof=4`; `audit.rs:148-165`):
  - Phantom-type (type-system-resident, via `proof=` expression): `WitnessTier::FormalProof`
  - proptest / kani / prusti / verus: `WitnessTier::FormalProof` or `Execution` per tool
  - `#[test]` function with llvm-cov coverage confirmation: `WitnessTier::Execution`
  - `#[test]` function without coverage confirmation: `WitnessTier::Reachability`
  - `requires=` predicate (substrate-tier): `WitnessTier::Execution` when predicate passes
- **Verdict emission**: per presents-site, per antigen, structured verdict
- **Sidecar identity**: compute from presents-site triple, not immune-site triple

#### Migration guide (asymmetric — three paths)

**Code-tier migration: `witness = fn` → `#[defended_by(X)]` on the test**

```rust
// Before — In tests: no annotation
#[test]
fn bijection_test_audit_hints_const_matches_enum() { ... }

// Before — At the immune-site:
#[immune(ParallelStateTrackersDiverge, witness = bijection_test_audit_hints_const_matches_enum)]
const ADR025_AUDIT_HINTS: &[&str] = &[ /* serde keys */ ];

// After — In tests: witness declares what it defends
#[test]
#[defended_by(ParallelStateTrackersDiverge)]
fn bijection_test_audit_hints_const_matches_enum() { ... }

// After — At the presents-site: #[immune] removed; audit computes verdict by cross-reference
#[presents(ParallelStateTrackersDiverge)]
const ADR025_AUDIT_HINTS: &[&str] = &[ /* serde keys */ ];
```

**Substrate-tier migration: `requires = pred` → folds into `#[presents]`**

Substrate-tier sites have no test function to annotate. `#[defended_by(X)]` has nothing
to attach to at these sites. Migration is entirely site-side.

```rust
// Before:
#[presents(UnpinnedDependency)]
#[immune(UnpinnedDependency, requires = "cargo-lock-committed")]
fn add_dependency(...) { ... }

// After: requires= folds into #[presents]; #[immune] removed
#[presents(UnpinnedDependency, requires = "cargo-lock-committed")]
fn add_dependency(...) { ... }
```

Substrate-tier sites in scope (aristotle T2, L1 — all verified against workspace grep):
`supply_chain_unpinned.rs:44`, `vcs_info_loss.rs:82/122/160`, `triage_commit.rs:96`,
`substrate_witness.rs:118`, `delta_attestation.rs:87`, `agentic_coordination.rs:112`,
`antigen_category.rs:161`.

**Phantom-tier migration: `witness = <phantom>` → `proof =` folds into `#[presents]`**

Phantom-type witnesses have no test function — the type-system construction IS the proof.
Cannot use `#[defended_by]`. Migration is entirely site-side, symmetrically to substrate-tier.

```rust
// Before:
#[presents(DropPanicClass)]
#[immune(DropPanicClass, witness = NonPanickingProof::<T>::verified)]
fn make_droppable<T>() { ... }

// After: proof= folds into #[presents]; #[immune] removed
#[presents(DropPanicClass, proof = NonPanickingProof::<T>::verified)]
fn make_droppable<T>() { ... }
```

The audit reads the `proof=` expression, recognizes the phantom shape structurally (same
`audit.rs:95-98` logic as today), classifies `WitnessTier::FormalProof`. Carrier-swap only.

### The honest semantic gap (adversarial findings)

The `#[defended_by]` marker + coverage-tier check + structural registry cross-reference
closes the *lazy-abuse surface* (no witness = fails) and reduces the *honest-mistake
surface* (wrong tier, uncovered site). It does NOT close the *semantic gap*: whether a
structurally-connected witness actually exercises the failure mode.

Four attack surfaces remain open after this ADR:

1. **Hollow wrapper attack**: witness calls a high-level API that incidentally covers the
   presents-site lines via llvm-cov. Coverage tier granted; failure mode unexpercised.
2. **Symbol-touch attack** (SubstrateAlignment-specific): witness imports both parallel
   symbols but never asserts a relation between them. Reads ≠ updates.
3. **Tier-inflation attack**: witness test excluded from coverage report that grants it
   coverage credit. Per-test coverage attribution would close this; aggregate line coverage
   does not.
4. **Stale-cross-reference attack**: witness correct at declaration time; production code
   refactors; antigen-relevant branch moves to an uncovered sub-function; coverage still
   passes. Drift-blind at the semantic level.

**ADR posture**: documented openly. This ADR provides *structural verification of the
witness circuit* — the wire exists and has current — but NOT *semantic verification that
the wire carries the right signal*.

**Intermediate mitigation (Approach 1.5, future work):** Fingerprint-anchored coverage
join — audit verifies witness-fn's llvm-cov coverage includes ≥1 line within fingerprint-match
site span. Requires per-item span data + per-test-binary coverage; non-trivial CI setup.

**Full semantic verification (Approach 4, future ADR):** witness-contract DSL declaring
what behavioral invariant a witness exercises.

### Open issues — not ratification blockers

**L2 — Phantom-witness registration (RESOLVED)**:
Phantom-type witnesses (`phantom_witness.rs:76`: `witness = NonPanickingProof::<T>::verified`)
are code-tier but NOT `#[test]` functions — `#[defended_by]` applies to test/proptest
witnesses only. Resolution: `proof=` folds into `#[presents]` (aristotle F1). The
phantom-proof reference is defense-evidence of the same category as `requires=` — site-attached
evidence with no test to annotate. The audit reads the `proof=` expression, recognizes the
phantom shape structurally (unchanged — same `audit.rs:95-98` logic), classifies `FormalProof`.
No new attribute, no auto-detect-guessing, no sidecar. Issue-3 (sidecar identity) dissolves:
`#[presents]` is the universal site-attached carrier, always co-located by construction.

**L3 — `partial` verdict and `min_tier` carrier (OPEN — deferred to v0.3)**:
The `partial` verdict presupposes a per-site required tier that no current primitive carries.
Recommendation: option A — `min_tier=` field on `#[presents(X, min_tier=Execution)]`. The
site knows what tier of defense it requires; the witness just registers what it provides.
Until implemented, `partial` is not emitted; the verdict vocabulary (`defended` /
`undefended` / `substrate-gap`) is complete without it.

### Voids seeded for v0.3+ (aristotle Phase 8)

Not addressed by this ADR; seed campsites when scheduling v0.3 work.

**V1 — Conjunctive (AND) multi-channel defense**: the current `witness=` / `requires=`
EITHER/OR structure forbids a site that is both code-tier-tested AND substrate-gated.
Real defenses are often conjunctive. `#[defended_by]` + `requires=` co-present = conjunctive
verdict. Future ADR.

**V2 — Two-evaluation-mode distinction**: runtime-witness (test executes) vs audit-time-
predicate (substrate checked) are different enough that one `witness` word hides the
substrate-gap third state. Links to e58627d5 bool-layer conflation (EvalNode::passed
ignoring evaluated). Future ADR.

**V3 — Declared/observed axis applies to vulnerability too**: `#[presents]`/fingerprint
already implement "observed not declared" for vulnerability recognition. This ADR addresses
the immunity half; the vulnerability half is structurally parallel. The scaffold primitive
(observe-first-declare-second) is V3's precedent already shipped. Future ADR.

### Sweep-level consequences

**ADR-004 (implicit-to-explicit elevation)**: extended. `#[immune]` made the defense-claim
explicit — correct at the time. This ADR makes the defense-*circuit* explicit: the witness
declares what it covers; the presents-site declares what it presents + what predicate its
defense requires; the audit derives the verdict. The verdict moves from static-declared to
dynamically-computed.

**ADR-005 (sub-clause F)**: strengthened. The verdict is computed by the audit tool, not
asserted by the developer. The audit's cross-reference IS the validation check. Exception:
`requires = <predicate>` on `#[presents]` is still a developer assertion about substrate;
audit validates it against substrate — developer doesn't get the final say.

**ADR-019 (substrate-witness predicate family)**: the `requires =` predicate family migrates
from `ImmuneArgs` to `PresentsArgs`. Predicate semantics unchanged; carrier changes. ADR-019
amended to reference `#[presents]` as the new carrier.

**ADR-013 (phantom-type witness recognition)**: phantom-type witnesses (`WitnessTier::FormalProof`)
continue to work. The type-system encoding IS the witness. `#[defended_by(X)]` is NOT
required for phantom-type witnesses. The audit discovers phantom witnesses via the `proof=`
expression on `#[presents]` (L2 resolution).

**ADR-006 (recognition-not-design)**: R5 over R4. `#[site_binding]` would have been a new
attribute carrying something `#[presents]` already holds at every target site. Dropped.
The presents-site is the natural carrier; extending it (additive field) is recognition;
inventing a new attribute is design.

### Enforcement

- `#[immune]` emits a `#[deprecated]` compiler warning from the proc-macro, pointing to
  the migration guide.
- `cargo antigen audit` gates on verdict, not on presence of `#[immune]`. Any presents-site
  without a registered code-tier witness (`#[defended_by]`) OR a passing substrate-tier
  predicate (`requires=`) = `undefended`.
- The `antigen audit` CI gate fails on `undefended` sites at Execution or FormalProof
  (configurable per antigen via severity).

### Resolves

- Removes the false binary immunity-claim from code sites
- Separates verdict (audit-derived) from code-tier witness registration (`#[defended_by]`
  on tests) from site-attached evidence (`requires=` substrate-predicate and `proof=`
  phantom-proof, both on `#[presents]`)
- Maintains sidecar identity at the presents-site (natural failure-locus)
- Aligns with biology: effector output (verdict) is produced by the immune system (audit),
  not declared by the antigen-presenting cell (code site). `#[defended_by(X)]` = antibody/BCR
  effector (ratified: glossary:22 witness=antibody; specificity intrinsic to the witness
  per BCR-variable-region analog — confirms witness-side class-binding per R5).
  `requires=` on site = germinal-center-record-sensing (context-sensing, site-attached;
  glossary:1038).
- Maintains backward compatibility via deprecation path (not a breaking removal)
- Ships R5 (ADR-006 compliant): recognition of `#[presents]` as carrier, not design of
  a new `#[site_binding]` attribute

### What this ADR does NOT do

- Does NOT claim code sites can compute or declare immune verdicts
- Does NOT close the semantic gap (structurally-connected witnesses that don't exercise the
  failure mode — open research question, see §honest semantic gap above)
- Does NOT implement `min_tier=` or the `partial` verdict (deferred to v0.3, L3)
- Does NOT break existing `#[immune]` usage (deprecation warning only; migration guide above)

---

## ADR-029 Amendment 1 — Substrate-intent precedence: a failing `requires=` is not masked by a code witness

**Status**: Ratified 2026-05-31.

**Amends**: ADR-029 §Verdict-precedence (compute_presentation_verdicts).

**Participants**: navigator (amendment draft + code fix). Source of truth: campsite
`findings/pv-requires-masked-by-code-witness` + ATK test `atk_pv_requires_masked_by_code_witness`.

**Reason**: The original ADR-029 verdict precedence rule computes `best_tier =
max(code_tier, immune_tier, site_requires_tier, site_proof_tier)` and maps
`Some(tier)` to `Defended { tier }` and `None` to `SubstrateGap` or `Undefended`.
This means a failing `requires=` predicate (which evaluates to `site_requires_tier =
None`) is silently hidden when a `#[defended_by]` code witness exists — the code
witness's `Reachability` tier wins `max(...)` and the site is reported as
`Defended(Reachability)`. The substrate intent the developer explicitly declared is
invisible in the audit output. `requires=` effectively becomes decoration rather than a
CI gate — the point of declaring it.

**The invariant violated**: ADR-029 §Mechanics states that `#[presents(X, requires=P)]`
is a substrate-witness declaration — the developer asserts that a substrate predicate P
MUST pass for the site to be defended. When P fails, the site has declared substrate
intent that is not met. This is exactly `SubstrateGap` (intent present, substrate
drifted) — regardless of whether a code witness also exists. A code witness does not
resolve a broken substrate predicate; it evidences a different channel. The two channels
are independent, and both must pass for the site to be fully defended.

### Change: requires= failure takes precedence over code witness tier

**New rule**: When `site_requires_eval` is `Some(None)` — meaning `requires=` was
present AND its predicate evaluation returned `WitnessTier::None` (failed) —
`compute_presentation_verdicts` must emit `SubstrateGap` regardless of `code_tier` or
`immune_tier`. A failing substrate predicate declared by the developer is a
substrate-alignment gap, not a gap patched by code witnesses operating in a different
channel.

Formally:

```
verdict = match (site_requires_eval, best_tier_excluding_requires) {
    // requires= present AND failed → SubstrateGap, regardless of code/immune tier
    (Some(None), _) => SubstrateGap,
    // requires= present AND passing → folded into best_tier normally
    (Some(Some(_tier)), best) => Defended { tier: best.unwrap() },
    // no requires= → original precedence
    (None, Some(tier)) => Defended { tier },
    (None, None) => match immune_gap { ... SubstrateGap or Undefended }
}
```

**Implementation**: `compute_presentation_verdicts` in `audit.rs` must check
`site_requires_eval == Some(None)` as an early branch BEFORE consulting `best_tier`.
The existing `best_tier` computation includes `site_requires_tier` (which is `None`
when the predicate fails); by checking `site_requires_eval` directly we distinguish
"no requires=" (`site_requires_eval = None`) from "requires= present but failed"
(`site_requires_eval = Some(None)`).

**ATK test inverted**: `atk_pv_requires_masked_by_code_witness` currently asserts
`Defended(Reachability)` (the broken outcome). After this fix, it must assert
`SubstrateGap`.

**Interaction with existing rules**: This amendment does not change `SubstrateGap`
semantics — it only refines the precedence rule so that a failing `requires=` always
surfaces regardless of code witness. `Defended` still means "evidence exists and is
passing." The amendment adds the invariant: if substrate intent is declared and broken,
the site is NOT defended.

---

## [ADR-030] Aggregate and Temporal Properties Are Audit-Observed

**Status**: Ratified 2026-05-27.

**Participants**: antigen-dx-dogfood team (scientist primary drafter; aristotle Phase-1-8
PASSED; naturalist biology PASSED; adversarial gate PASSED; scientist consistency review
COMPLETE; all four ceremony signers).

**Related**: ADR-029 (per-site defense verdicts), ADR-023 (deferred-defense declarations),
ADR-024 (convergent/recurrent emergence).

**Ceremony campsite**: `ceremony/ratify-adr-030-aggregate-temporal-observed`.

### Problem

The Antigen vocabulary includes annotation sites that make claims whose truth drifts after
write-time, without the annotation changing. A developer writes `instances = 3` on a
`#[recurrence_anchor]` today; tomorrow a fourth instance exists; the annotation still says 3.
No compile error. No audit signal. The claim is silently false.

Five instances of this pattern have been identified and verified against substrate:

**A. `recurrence_anchor.instances`** — the author declares how many instances they know of.
The audit can compute the count independently (workspace-wide cross-reference of `#[itch]`
anchors to `#[crystallize]` sites). The declared count drifts silently as instances accumulate.
Exterior consulted by fix: workspace instance count.

**B. `chronic.status`** — the author writes a prose assessment ("this is chronic, status:
worsening"). No re-confirmation clock. The assessment can be written once and never revisited
while the referent evolves — the antigenic-drift shape (bytes unchanged, referent changed).
Exterior consulted by fix: wall-clock date of last status review.

**C. suppression density** — no individual site declares the workspace-level aggregate of
deferred-defense declarations. The tolerance budget is bidirectionally pathological:
too many suppressions = immunodeficiency (nobody notices the accumulation); too few = healthy
but possibly overly rigid. The audit can compute density per antigen and warn on threshold.
Exterior consulted by fix: per-antigen accumulation count across the workspace.

**D. `orient-until-date`** (REFERENCE IMPLEMENTATION — SHIPPED `bf60e5d`).
A `#[orient]` declaration asserts its deferral is warranted until a declared date. Time passes;
the date elapses; the assertion is now false. The annotation is unchanged. Fix: observe
current date, compare to `until`, emit `OrientPendingActionRequired` if elapsed.
This pattern is the reference implementation for all other instances.

**E. `immunosuppress.duration_cap`** — the author writes `duration_cap = 30` (days) to bound
how long a suppression may last. The claim is: "this suppression expires within 30 days from
`since`." Fix: add `duration_cap: Option<u64>` and `since: Option<String>` as typed fields to
`DeferredDefense`, populate during scan push, add duration-check in evaluator comparing
`(today - since_date).num_days()` to cap. Adversarial confirmed (`d72dacf`).
Status: SHIPPED (commit `ac75c10`) — typed fields landed in `scan.rs`; emission path wired in
`evaluate_deferred_defense_hint()`; `ImmunosuppressDurationCapExceeded` now has an active
emission path; ATK documentation tests inverted to assert correct behavior.

### Unifying Principle

**Any claim whose truth can drift after write-time AND HAS A CONSULTABLE EXTERIOR must be
audit-observed.**

Both conjuncts are required (aristotle OQ1 refinement, Phase-7 stability check): truth-drift
is NECESSARY but NOT SUFFICIENT for observation-eligibility. A consultable exterior is the
second conjunct. Without it, the principle demands observing the unobservable — some drift-prone
claims correctly stay declared because no exterior exists (Tier-3, see below). The §Mechanics
criteria already encode the fix; this principle reconciles with them.

The principle distinguishes three tiers of drift-prone claims (aristotle OQ1 three-tier
sharpening, substrate-verified against all 27 arg-structs):

- **Tier-1 (directly-observable)**: structured claim WITH a machine-consultable exterior.
  Exterior = wall-clock date or workspace-count. Examples: `orient.until`,
  `recurrence_anchor.instances`, `immunosuppress.duration_cap+since`. The audit consults the
  exterior directly. Must be observed.

- **Tier-2 (proxy-observable)**: prose claim WITH a temporal sibling. The prose's SEMANTIC
  TRUTH is not audit-consultable (no exterior judges whether a rationale is still accurate),
  BUT the temporal sibling gives a PROXY exterior: the staleness of the review-date is observable
  even when the prose truth is not. Examples: `chronic.status + status_reviewed_at` (proposed),
  `immunosuppress.rationale + until`. Prose is NEVER directly-observable; its only exterior is
  a review-date-staleness proxy. Observation targets the proxy, never the semantic content.

- **Tier-3 (stays-declared, no exterior)**: prose claim, NO temporal sibling, no semantic
  exterior exists. Examples: `saturate.description`, `itch.description`, `crystallize.summary`.
  These ARE drift-prone (a description can become false as the pattern evolves), but they have
  NO exterior the audit can consult — not even a proxy. By criterion-2 they correctly stay
  declared. They are NOT a hole in the axis — they are the axis correctly classifying
  drift-prone-but-unobservable claims as declaration-stable-by-necessity. A Tier-3 field can be
  PROMOTED to Tier-2 by adding a `reviewed_at` date sibling — a design decision per primitive,
  not forced by the axis.

The principle establishes: silence on a Tier-1 or Tier-2 claim is the failure class; Tier-3
claims remain declared and the distinction is honest (not a gap).

This is complementary to ADR-029's principle (immunity is OBSERVED not declared), applied
to the aggregate and temporal class:

| ADR-029 | ADR-030 |
|---------|---------|
| Per-site defense verdicts | Aggregate + temporal drift |
| "Is this site defended?" | "Is this claim still true?" |
| Defense intent declared; verdict observed | Claim declared; currency observed |
| Exterior: test registry (code-tier), sidecar (substrate-tier) | Exterior: wall-clock, workspace count, accumulation |

ADR-029 inverted one claim class (per-site immune verdict → observed by audit). ADR-030
covers the complementary emergent class: system-level and time-sensitive properties that
only the audit can observe.

**Locus-dispatch connection** (aristotle OQ1): the "exterior the audit consults" is the
WITNESS-LOCUS (wall-clock / workspace-scan / sidecar / git-metadata — the same loci aristotle
found in the detectability collapse-test and applied in ADR-031 OQ4). ADR-030's
observation-eligibility is the same structure as ADR-031's revocation-verification: both are
observed-not-declared AT A LOCUS; both have a residual class with NO locus (ADR-031:
`RevocationCannotBeVerified` near-empty residual; ADR-030: Tier-3 prose stays-declared).

Pathmaker's independent framing (camp notice `a89ac198`, 2026-05-27) arrived before this ADR
draft: "ANY claim whose truth can drift after write-time should be audit-OBSERVED, not
site-DECLARED." This independent convergence from the implementation side confirms the
unifying principle; the conjunct refines it to be precision-complete.

### Decision

**Aggregate and temporal properties must be audit-observed, not declared-authoritative.**

Concrete resolutions per instance:

**A. `recurrence_anchor.instances` observation**: Add audit cross-reference: for each
`#[recurrence_anchor]`, count actual `#[crystallize]` sites that reference it. If
`declared_instances != observed_count`, emit `RecurrenceAnchorInstanceCountMismatch` hint.
The declared count remains (it is the author's view at write-time; valuable as documentation
of what the author knew). The audit's count is authoritative for the verdict; mismatch is a
finding, not a build break.

**B. `chronic.status` staleness observation**: Add `status_reviewed_at = "YYYY-MM-DD"` to
`#[chronic]` args. Make it optional initially; audit emits `ChronicStatusReviewOverdue` when
`status_reviewed_at` is present and the date is older than a configurable threshold (default
90 days). When absent, audit emits `ChronicStatusNoReviewDate` (advisory). The prose status
stays declared (the author's assessment). The review date is the freshness signal the audit
observes.

**C. suppression density observation**: Add a workspace-level density check to
`audit_deferred_defenses()`: for each antigen, count active deferred-defense declarations;
if count exceeds configurable threshold (default 3 per antigen), emit `SuppressionDensityHigh`.
The threshold is workspace-configurable (`.antigen.toml` or similar). This closes the
tolerance-budget observability gap: the audit knows the aggregate; it must surface it.

**D. `orient-until-date` (ALREADY SHIPPED — reference implementation)**: `#[orient]` now
observes `until` against wall-clock date. `OrientPendingActionRequired` emits when elapsed.
This is the reference implementation for B and E.

**E. `immunosuppress.duration_cap` observation (SHIPPED — commit `ac75c10`)**: The three-step
fix landed during the ADR ceremony arc: (1) `duration_cap: Option<u64>` and `since: Option<String>`
added as typed fields to `DeferredDefense` (scan.rs); (2) typed fields populated during scan
push; (3) `evaluate_deferred_defense_hint()` computes `(today - since).num_days()` and compares
to `cap`. Emits `ImmunosuppressDurationCapExceeded` when exceeded. Adversarial documentation
tests were inverted to assert correct behavior. Instance E is now a second reference
implementation alongside D.

### Mechanics

**Observation criteria.** A property is OBSERVATION-ELIGIBLE when ALL of:

1. It is declared on an annotation site.
2. There exists an exterior the audit can consult without network access or user-supplied
   code (wall-clock, workspace scan, sidecar files, git metadata).
3. The exterior can be consulted at `cargo antigen audit` time with bounded cost.
4. Disagreement between the declaration and the exterior constitutes a meaningful finding.

Properties that fail any criterion stay declared.

**Placement criterion** (naturalist OQ2 finding, biology-grounded via PMID 12819486): The
audit should observe at the CHEAPEST point that catches the failure-class, not everywhere
an exterior exists:

- **Per-site observation**: appropriate when a site-local check suffices (each site is checked
  in isolation at bounded cost). Examples: `orient-until` per-site date check;
  `immunosuppress.duration_cap` per-site elapsed-days check.
- **Aggregate/census observation**: appropriate ONLY when the property is genuinely emergent
  (no single site holds it; a per-site check CANNOT capture it). Examples: suppression density
  (no tolerizing site perceives systemic immunosuppression level); recurrence_anchor.instances
  count (no single anchor knows the workspace count).

Observe per-site when a site-local check suffices; reserve census/aggregate for
genuinely-emergent properties. Biology never runs an aggregate census when a cheaper per-site
check catches the same failure (PMID 12819486 — central-once vs peripheral-continuous is the
biology's cost-bounded placement discipline).

**Hint posture.** All five instances produce ADVISORY hints (non-blocking by default). The
pattern matches `audit_deferred_defenses()` today: a finding is surfaced, not a build break.
Adopters can gate on hint presence via `--strict` for the hints they care about (future
extension).

**Tier-2 vs Tier-1 asymmetry** (adversarial OQ3 gate finding, 2026-05-27): Tier-2
proxy-observation (e.g. `status_reviewed_at` staleness) is structurally weaker than Tier-1
direct-observation (e.g. `orient.until` elapsed-date check). Tier-1 proves the CLAIM IS FALSE
(the deadline elapsed; the count diverged). Tier-2 proves the FORM of re-examination was
present (a date was recorded), NOT the SUBSTANCE (that review actually occurred). This is not
a flaw — it is the honest semantic gap documented per ADR-029 §honest semantic gap. The
advisory hint tier is the correct posture for Tier-2 findings PRECISELY because of this
weakness: an advisory surfaces the finding without asserting substance-of-review it cannot
verify. Adopters choosing to gate on Tier-2 hints via `--strict` should understand they are
gating on form, not substance.

### Gate outcomes (ceremony complete)

**OQ1 — PASSED** (aristotle Phase-1-8, 2026-05-27): The observation-eligible /
declaration-stable axis HOLDS across all 27 arg-structs. No primitive breaks it. Three
refinements supplied: (1) principle-conjunct fix (drift + consultable-exterior both required,
not drift alone); (2) three-tier taxonomy (direct/proxy/stays-declared); (3) Tier-3 → Tier-2
promotion mechanism (add a `reviewed_at` sibling to promote any Tier-3 prose field). Biology
grounding confirmed: the exterior-locus IS the witness-locus of ADR-029/031 — the three ADRs
share one locus-dispatch frame.

**OQ2 — PASSED** (naturalist biology, 2026-05-27): Wall-clock / accumulation / workspace-count
exterior families each have clean biology cognates (kinetics/FcRn-clock, tolerance-budget-as-fold,
clonal-census) with constraints. Additional finding: the observation PLACEMENT criterion
(cost-bounded per-site vs aggregate-census) is grounded by PMID 12819486 (central-once vs
peripheral-continuous tolerance). All 5 instances correctly placed.

**OQ3 — PASSED** (adversarial, 2026-05-27): Timestamp-washing attack acknowledged as the
Tier-2 proxy's known semantic gap — the audit observes the declared date, not that review
actually occurred. Posture: observation is better than silence; the semantic gap is documented
openly per ADR-029 §honest semantic gap. Three additional angles probed: (1) tier-promotion
racing: Tier-3 promoted to Tier-2 correctly gets proxy-observable, not Tier-1 strength;
(2) suppression density threshold gaming: splitting across annotations to stay under threshold
is an acknowledged limitation (advisory, configurable); (3) `since=None` in immunosuppress
silently skips duration_cap check (separate gap, separately documented). Advisory added to
§Mechanics hint posture: Tier-2 observation proves form, not substance.

### What this ADR does NOT do

- Does NOT make any aggregate property a build-breaking finding without explicit `--strict` opt-in
- Does NOT invent a "review workflow" — the audit observes exteriors; it does not enforce process
- Does NOT cover per-site defense verdicts (ADR-029's domain)
- Does NOT cover fingerprint-structural claims (ADR-009/ADR-010's domain)
- Does NOT claim to fully close the semantic gap (a declared `status_reviewed_at` proves the
  author set the date, not that they actually reviewed the status)

### Evidence citations

- Instance D reference implementation: `bf60e5d` (orient-until-date fix)
- Instance E shipped: `ac75c10` (ImmunosuppressDurationCapExceeded emission path + typed fields)
- Instance E adversarial confirmation: `d72dacf` (ATK-IMMUNOSUPPRESS-DURATION-CAP-UNREACHABLE)
- Pathmaker independent convergence: camp notice `a89ac198` (2026-05-27)
- Scientist validation: campsite `forward/adr030-aggregate-temporal-observed` (5 validated instances)
- Naturalist OQ2 biology gate: PMID 36726033 (Pyzik, *Nat Rev Immunol* 2023 — FcRn-controlled
  half-life); PMID 12819486 (Wekerle, *Transplantation* 2003 — central vs peripheral tolerance
  as distinct mechanisms)
- Adversarial OQ3 gate: camp note `8ca8ccf5` (2026-05-27 — timestamp-washing + tier-promotion-racing
  + density-gaming probes; PASSED)
- Locus-dispatch synthesis: camp notice `87bb2f0b` (navigator, 2026-05-27 — shared frame across
  ADR-029/030/031)

---

## [ADR-031] Negative Selection: `#[no_longer_presents(X)]` and Revocation-Staleness Observation

**Status**: Ratified 2026-05-27.

**Participants**: antigen-dx-dogfood team (scientist primary drafter; aristotle Phase-1-8
PASSED; naturalist biology PASSED; adversarial gate OQ5 PASSED; outsider naive-pass;
scientist consistency review COMPLETE; all four ceremony signers).

**Related**: ADR-018 (diamond dedup + inheritance state matrix), ADR-029 (observed-not-declared),
ADR-030 (aggregate/temporal drift observation).

**Ceremony campsite**: `ceremony/ratify-adr-031-no-longer-presents`.

### Problem

`#[descended_from(Parent)]` propagates the parent's full presentation-set to each descendant
by SET-UNION across ALL inheritance paths (`scan.rs:1469/2814/2918`). Once a site is inherited,
there is no mechanism to revoke it at the descendant level. A child type that no longer has the
structural shape that made its ancestor present `X` has no way to declare that state. It is
stuck presenting `X` forever — not because the vulnerability is real, but because the
declaration machinery has no delete operation.

Scout's empirical finding (`f02f40bf`) and aristotle's Phase-8 ruling converge on the same
gap. The biology cognate: **negative selection** in the thymus deletes self-reactive T-cell
clones before they can mount an autoimmune attack. Without it, inherited recognition propagates
to clones that the gate should have pruned — the autoimmune-shadow-discovery-engine shadow #6
prediction, confirmed empirically.

**The 2×2 structure** (aristotle + outsider gate finding, 2026-05-27): the revocation primitive
closes BOTH off-diagonals of a signal-1 × signal-2 matrix. Without the primitive, only one of
the two fail-classes is observable:

- signal-1 (structure matches) + signal-2 (affirmation absent): `RevocationContradictedByStructure`
- signal-1 (structure ceased) + signal-2 (affirmation absent):
  `InheritedPresentationStructurallyCeasedWithoutAffirmation` (NEW)

Both require the audit to observe signal-1 (structural fingerprint check) and signal-2
(affirmation presence). The declaration requirement ensures the audit can distinguish
(a) genuinely-fixed from (b) silently-drifted.

### Evidence

Aristotle triangulated the placement (item-level vs edge-level) three ways:

**(1) Biology (naturalist gate — passed, primary-source grounded).** Negative selection deletes
the self-reactive **clone** (item), not its lineage edge. A thymocyte is deleted because of
WHAT IT PRESENTS (self-reactivity), not because of WHO ITS PARENT WAS. Verified via PMID
12766760 (Palmer E, *Nat Rev Immunol* 2003): the deletion decision is clone-intrinsic, made on
the clone's current receptor. Falsifier hunt found zero biology cases where a lineage EDGE
carries the deletion signal rather than the clone — the predicted falsifier came up empty,
which is honest evidence the item-level placement is structurally correct.

**(2) Subject-check.** The subject of a revocation is "this item's presentation-set" — does
this item present X? Edge-level revocation (`reverted_for=[X]` on `#[descended_from(Parent)]`)
makes the subject the edge. Item-level makes it the item's presentation-set. The item is the
right subject. Confirmed by aristotle's Phase 1-8 (presentation-state MODIFIER, not a new
identity axis — see §Three surfaces below).

**(3) Diamond-union incompleteness — the load-bearing argument (Phase-7 structural guarantee).**
Inherited presentations are computed by SET-UNION across ALL diamond paths. If a child inherits
X from parent-A AND from parent-B via a diamond, X appears in the union. An edge-level
revocation on the A-path does not remove X — it still arrives via the B-path. To fully revoke
X via edge-level, one must mark `reverted_for=[X]` on EVERY ancestor edge that carries X.
Adding a new ancestor later silently re-introduces X. Item-level `#[no_longer_presents(X)]`
revokes X from the item's effective presentation-set AFTER the union, regardless of how many
edges carried it — the only shape that is complete under the existing diamond-union semantics.
Edge-level revocation cannot be made complete without redesigning the propagation model;
item-level is robust under the existing model. This is a STRUCTURAL GUARANTEE from set-union,
not a conjecture; it holds for every inheritance graph with a diamond.

### Decision

**Primitive**: `#[no_longer_presents(X)]`

```rust
#[no_longer_presents(ClassName)]
fn method_that_no_longer_has_this_vulnerability() { ... }
```

Semantics: the annotated item explicitly revokes a presentation that would otherwise be
inherited. The audit OBSERVES whether the revocation is warranted — if the item STILL matches
X's structural fingerprint (or other witness evidence; see fail-class below), the revocation
is false.

- **Placement**: item-level (on the descendant item, not on the `#[descended_from]` edge).
- **Scope**: revokes the named presentation from the item's effective presentation-set. Does
  not alter the inheritance graph or parent declarations.
- **Inheritance**: `#[no_longer_presents(X)]` is NOT inherited by the item's own descendants.
  Each descendant that no longer presents X must declare it independently. (The revocation
  decision is specific to the structural shape of THIS item; a child that is more like the
  original parent may genuinely re-present X.)

**Stand-alone (preventive) use.** `#[no_longer_presents(X)]` with no X anywhere in the item's
inheritance union has two cases:

- *Without a `preventive` flag* (default): emits `RevocationOfUnpresentedClass` advisory.
  Catches typos, stale class names, and revoking the wrong class — the most common stand-alone
  mistake.
- *With an explicit `preventive` flag* (`#[no_longer_presents(X, preventive)]`): accepted. The
  AIRE cognate (biology: thymic epithelial cells pre-tolerize against peripheral antigens
  T-cells would never otherwise encounter) confirms this is a real, grounded mechanism — not
  mere defensive programming. The audit suppresses `RevocationOfUnpresentedClass` for
  preventive declarations BUT continuous re-scanning remains armed:
  `RevocationContradictedByStructure` fires the moment a matching structure appears.
  Preventive revocation is NOT trust-once; it is observed-continuously-into-the-future. A
  preventive declaration whose referent never appears is never contradicted; one whose referent
  later appears converts immediately to a checked revocation.

**New fail-class: `RevocationContradictedByStructure` (witness-locus-relative family).** If an
item declares `#[no_longer_presents(X)]` but the audit's observation still finds X's
structural pattern present, the revocation is false. The declared intent and the observed
structure disagree. This is the observed-not-declared principle (ADR-029) applied to
revocations:

- The author DECLARES the revocation-intent: "I no longer present X."
- The audit OBSERVES whether the structure supports it (at the witness-locus of X).
- Disagreement is `RevocationContradictedByStructure` (advisory initially, escalatable via
  `--strict`).

**Witness-locus-relative verification** (aristotle OQ4 sharpening): how you observe a
revocation is determined by WHERE the antigen's witness reads from — the witness-locus. This
makes `RevocationContradictedByStructure` a FAMILY, one member per locus, not a fingerprint-only
check:

| Antigen type | Witness-locus | Revocation observed via |
|---|---|---|
| Has a fingerprint | In-repo-static | Fingerprint NON-match |
| Verify-only (`fingerprint: None`) | External-substrate | Predicate NON-satisfaction at the substrate |
| Code-witness (behavioral) | Runtime | Witness test passes for the revoked state |

`RevocationCannotBeVerified` is the near-empty residual advisory: emitted only when the antigen
has NO observable locus at all (neither fingerprint, evaluable predicate, nor witness). By
ADR-028 category-enforcement every antigen must have at least one witness-leaf, so this
residual should be structurally rare.

**`rationale=` escape-hatch**: `#[no_longer_presents(X, rationale = "...")]` silences the check
when the author can explain why the observed evidence is a false positive. The canonical case
(naturalist OQ2 enrichment) is **molecular mimicry** — a DIFFERENT antigen's look-alike
fingerprint coincidentally matches the item, producing a false positive that is NOT the
re-introduction of X. The escape-hatch is motivated by a real biology-grounded phenomenon, not
just defensive convenience.

**Why the affirmation is DECLARED (not auto-revoked on structural cessation).** The
**asymmetry of silence**: fingerprint non-match has THREE possible meanings: (a) genuinely
fixed; (b) imprecise fingerprint that missed a mutated form; (c) a refactor that WILL regress.
Auto-revoking on non-match alone would make the audit SILENTLY ASSUME (a) — manufactured
safety by choosing the reassuring interpretation of an ambiguous signal. This is
SILENCE-BY-MASKING (silence-taxonomy generator 2, applied to revocation).

**The declaration is costimulation** (aristotle + outsider gate finding, 2026-05-27; confirmed
ADR-028 §all_of = costimulation principle, decisions.md:5383): structural non-match =
signal-1; the author's affirmation = signal-2. Acting on signal-1 alone (auto-revoke) is
inappropriate-activation-without-costimulation — the exact pathology that B-cell activation
guards against via CD28/CD80-86. `#[no_longer_presents(X)]` is not optional because revocation
requires signal-2; only the author can supply the judgment that distinguishes (a)–(c) above.
The declaration IS the costimulation gate that prevents the audit from silently assuming-fixed.

**Mirror fail-class: `InheritedPresentationStructurallyCeasedWithoutAffirmation`.** A child
whose inherited X's fingerprint has STOPPED MATCHING but has NO `#[no_longer_presents(X)]` is
in an OBSERVABLE-BUT-UNAFFIRMED state. The structure changed (signal-1 fired) but no human
vouched (signal-2 absent). This is SILENT DRIFT — the audit CAN observe it and SHOULD surface
it.

The two off-diagonal fail-classes close a clean 2×2 matrix (signal-1: structure-matches? ×
signal-2: affirmed-revoked?):

| | signal-2 absent (no affirmation) | signal-2 present (affirmation declared) |
|---|---|---|
| **signal-1 matches** (structure still there) | Healthy: item still presents X | `RevocationContradictedByStructure` (vouched-but-false) |
| **signal-1 absent** (structure ceased) | `InheritedPresentationStructurallyCeasedWithoutAffirmation` (true-but-unvouched) | Healthy: cleanly revoked |

On-diagonals are healthy states. Off-diagonals are fail-classes. The audit observes both
signals; the human supplies signal-2; disagreement between them is the finding.

`InheritedPresentationStructurallyCeasedWithoutAffirmation` (advisory, verdict:
`UnaffirmedCessation`): emitted when an inherited presentation's fingerprint no longer matches
at this site AND no `#[no_longer_presents(X)]` exists at this item.

**Detection is two-stage**, each stage doing what its subject permits:

- *Stage 1 (scan-layer, fully decidable)*: is X in this item's inherited union? The diamond
  set-union (`scan.rs:1469/2814/2918`) already computes this. If X is NOT in the inherited
  union: state (a) never-vulnerable — NOT A FINDING. If X IS in the inherited union but
  fingerprint non-matches: state (b)–(c), unaffirmed cessation — route to Stage 2.
- *Stage 2 (verdict layer, the undecidable fork)*: "fingerprint non-match on an inherited
  site" has two irresolvable interpretations — (b) silently-fixed (vulnerability genuinely
  gone) and (c) fingerprint-stale (code diverged, still vulnerable, fingerprint too imprecise
  to catch the mutated form). The audit CANNOT distinguish (b) from (c): the fingerprint is
  the only structural instrument and its imprecision is the unknown. The verdict
  `UnaffirmedCessation` names this unresolved state and routes it to the human — it does NOT
  choose between (b) and (c).

The human resolves by ONE of two actions: affirm via `#[no_longer_presents(X)]` (asserts it
was (b) genuinely-fixed; `RevocationContradictedByStructure` guards if wrong) OR tighten the
fingerprint (if it was (c), a tighter fingerprint re-matches and the cessation was illusory;
the advisory clears on next scan).

The audit already has the adjacent `inherited_unaddressed` machinery (audit.rs:840); this is
an extension of that path.

**Verdict vs antigen distinction**: `UnaffirmedCessation` is the audit's verdict-name
(witness/audit-cross-reference surface). `UnaffirmedStructuralCessation` is the dogfood antigen
on the identity surface — a genuinely-fixed inherited vulnerability left unaffirmed,
indistinguishable from stale-imprecise to the next reader. Distinct surfaces per the
three-surface map.

**Biology cognate**: the equivalent of a self-reactive clone whose antigen stopped being
expressed (the self-antigen's expression pattern changed), but the autoreactive clone persists
in the repertoire. The antigen is absent (signal-1), but the clone's specificity remains (no
signal-2 confirmation from the thymic-equivalent mechanism that the clone is safe to keep).
If the antigen is re-expressed later, the clone immediately re-activates.

**`RevocationOfUnpresentedClass`**: emitted when `#[no_longer_presents(X)]` is present but X
is absent from both direct presentations and the inherited union, with no `preventive` flag.
Catches stale class names, typos, and wrong-class revocations.

**Revocation-staleness observation (per ADR-030 pattern).** A `#[no_longer_presents(X)]`
declaration is a claim that can drift from its referent. If code is later changed to
re-introduce the structural shape of X (the exact drift ADR-030 covers for temporal claims),
the revocation becomes stale-and-wrong without any annotation change. The audit's witness-locus
observation handles this continuously: every `cargo antigen audit` re-checks whether the
evidence still matches. No additional expiry mechanism is needed — the revocation is
re-validated on every scan, not trusted once from declaration time.

This is the "continuous tolerance" cognate from biology (central + peripheral tolerance, not
one-time thymic pruning — naturalist grounded via PMID 10227976 Laufer et al., *J Immunol*
1999: self-reactive T-cells escape thymic selection and are pathogenic in vivo; one-time
pruning is insufficient). The audit IS the continuous re-evaluation.

### Relationship to ADR-029 (observed-not-declared)

`#[no_longer_presents(X)]` follows the same declaration-intent / audit-observation split as
`#[presents]`:

| | `#[presents(X)]` | `#[no_longer_presents(X)]` |
|---|---|---|
| Author declares | "this site presents vulnerability X" | "this site no longer presents X" |
| Audit observes | does the witness-locus evidence match? | does the witness-locus evidence NOT match? |
| Disagreement | SubstrateGap (undefended site) | `RevocationContradictedByStructure` |

Both are claims the audit verifies at the antigen's witness-locus. Neither is self-authorizing.
The witness-locus-relative verification makes this the direct extension of ADR-029's
observed-not-declared principle along the time axis: a revocation's truth is continuously
observed, not declared once.

### Relationship to ADR-018 (diamond dedup + inheritance state matrix)

ADR-018 defined the inheritance state matrix (presented, inherited, suppressed states at
presentation sites). `#[no_longer_presents(X)]` adds a new state: **revoked** (present in the
inherited union, explicitly canceled at this item). The inheritance state matrix gains a
fourth column:

| State | Declaration | Meaning |
|---|---|---|
| Presented | `#[presents(X)]` | This item directly presents X |
| Inherited | (no annotation) | X is inherited from an ancestor |
| Suppressed | `#[anergy(X)]` | X is suppressed for a defined period |
| Revoked | `#[no_longer_presents(X)]` | X is canceled, structural non-match expected |

### Three surfaces — closed at three (aristotle Phase 1-8)

Aristotle's ruling closes the identity-surface question: `#[no_longer_presents(X)]` is a
PRESENTATION-SITE STATE MODIFIER, NOT a new `#[antigen]` identity axis. It operates on the
presents-site surface as negation/retraction, parallel to `#[anergy]` (Suppressed state). The
three surfaces now mapped:

- **ANTIGEN-identity**: category / encounter / tier (3 axes, closed)
- **PRESENTS-site-state**: Presented / Inherited / Suppressed / Revoked (4 states, extended by this ADR)
- **WITNESS**: selection (silence-generator) + locus (in-repo / external-substrate / runtime)

"Fourth identity axis" candidates keep sorting to presents-site or witness by subject-check.
The identity surface stays closed at three.

### Known open gaps

**Diamond-sibling surface (v0.3 direction)**: `#[no_longer_presents(X)]` is NOT inherited. In
a diamond, C descends from both A (presents X) and B (descends from A, revokes X). C inherits
X from A via set-union. B's revocation does NOT flow to C — intentional by design (revocation
is specific to B's structural shape). If C's fingerprint still matches X, nothing fires; C
validly presents X; B's intent ("the subtree lost X") is silently uncovered at C. The gap is
that the ADR does not (yet) produce an advisory for this "forgot-to-re-revoke in a diamond"
case. v0.3 direction: `RevocationUncoveredByDiamondSibling` advisory — when a presented item
has a revoked ancestor on one inheritance path but inherits-X on another.

**Preventive window + imprecise fingerprint**: a preventive revocation declared before any
matching structure exists cannot fire `RevocationContradictedByStructure` (no structure to
match at declaration time). If the structure later appears but stays JUST BELOW the
fingerprint threshold (imprecise-fingerprint attack), the preventive revocation silences a
real vulnerability during that window. This is the existing imprecise-fingerprint surface
(already in adversarial's scope), widened by the preventive window, NOT a new attack class.
Guard: continuous re-scan armed on structure-appearance; fingerprint precision is the root
mitigation.

**`rationale=` is semantic-only**: the escape-hatch accepts any string rationale with no
structural verification — same shape as `#[anergy(rationale=...)]` and
`#[immunosuppress(rationale=...)]`. Named for completeness.

**`InheritedPresentationStructurallyCeasedWithoutAffirmation` — v0.3 implementation**: the
mirror fail-class is NAMED and the audit path exists (`inherited_unaddressed` at
`audit.rs:840`), but the SPECIFIC advisory for "inherited-X fingerprint-ceased-without-revocation"
is not yet emitted. When ADR-031 implementation begins, this advisory should be added to the
inherited-presentation audit path alongside the existing unaddressed machinery.

**Known limitation: the (b)-vs-(c) distinction is not audit-decidable.** When the audit emits
`UnaffirmedCessation`, it observes that fingerprint non-match has occurred on an inherited
site (Stage 1 decidable) and that no affirmation exists (Stage 2). But it CANNOT determine
whether the cessation is (b) genuinely-fixed or (c) fingerprint-imprecise-still-vulnerable —
the fingerprint is the only structural instrument, and its imprecision is the unknown. Making
this distinction would require the audit to claim knowledge it does not have; it would
produce silence-by-masking (choosing the reassuring interpretation). This is an honest
semantic gap, parallel to the imprecise-fingerprint gap in ADR-029 §honest semantic gap. The
resolution is not in the audit but at the human's locus: affirm
(`#[no_longer_presents(X)]` asserts (b)) OR tighten the fingerprint (if (c), the re-match
self-corrects the advisory). The undecidable cut routes to the human, exactly as the
locus-dispatch frame requires.

### What this ADR does NOT do

- Does NOT alter the existing `#[descended_from]` propagation model (set-union is preserved).
- Does NOT add `reverted_for=` to `#[descended_from]` edges (aristotle's ruling: edge-level is
  incomplete under diamond-union semantics).
- Does NOT make `RevocationContradictedByStructure` a build-breaking finding by default.
- Does NOT inherit revocations to the item's own descendants (each descendant declares for
  itself).

### Gate outcomes (ceremony record)

**Aristotle (Phase 1-8) — PASS.** Diamond-union structural-guarantee confirmed (Phase-7
stable); item-level placement = presentation-state-modifier (not identity axis);
identity-surface-closed-at-three holds. OQ3: `RevocationOfUnpresentedClass` advisory +
explicit `preventive` flag (AIRE-grounded, continuous-rescan-armed). OQ4: revocation
verification is witness-locus-relative; `RevocationContradictedByStructure` is a locus-relative
family; `RevocationCannotBeVerified` is the near-empty residual.

**Naturalist (biology) — PASS.** OQ1: item-level confirmed via PMID 12766760; falsifier hunt
empty. OQ2: continuous-tolerance maps via PMID 10227976; staleness-observation is biologically
required. `RevocationContradictedByStructure` has a clean cognate (escaped self-reactive
clone). Molecular-mimicry false-positive named as canonical `rationale=` case.

**Adversarial (OQ5) — PASS WITH ONE ADDITION (incorporated).** Diamond-revocation escape: real
but intentional — non-inheritance is load-bearing; diamond-sibling surface documented in
§Known open gaps. `rationale=` gaming: same shape as existing escapes; not a new surface.

**Scientist (consistency review) — PASS.** All four gate findings incorporated. ADR-029 table
updated to reflect locus-relative framing. `RevocationContradictedByStructure` correctly
presented as a family. §Known open gaps names both remaining surfaces cleanly. No internal
consistency failures.

**Post-ceremony amendments** (incorporated by scientist, re-reviewed PASS): (1) Mirror
fail-class `InheritedPresentationStructurallyCeasedWithoutAffirmation` + costimulation
rationale for why affirmation is declared. (2) `UnaffirmedCessation` verdict name + two-stage
detection architecture + (b)-vs-(c) known-limitation clause + `UnaffirmedStructuralCessation`
dogfood antigen. Strengthening-only; item-level/diamond-union/locus-relative core unchanged.

### Evidence citations

- Aristotle ruling: `forward/descended-from-negative-selection-gap` (camp story, 2026-05-27)
- Scout empirical gap: field notice `f02f40bf` (inheritance revocation gap)
- Diamond-union semantics: `scan.rs:1469/2814/2918` (substrate-verified by aristotle)
- Autoimmune-shadow-discovery shadow #6 prediction + empirical convergence:
  `forward/autoimmune-shadow-discovery-engine`
- ADR-029 observed-not-declared: §ADR-029
- ADR-018 inheritance state matrix: §ADR-018
- PMID 12766760: Palmer E (2003). "Negative selection — clearing out the bad apples from the
  T-cell repertoire." *Nat Rev Immunol.*
- PMID 10227976: Laufer et al. (1999). *J Immunol* — self-reactive T-cells escape thymic
  selection and are pathogenic in vivo.
- PMID 39621313: *J Clin Invest* (2024) — impaired negative selection → diabetes.
- Aristotle outsider-gate response: camp note `20100db6` (2026-05-27 —
  `InheritedPresentationStructurallyCeasedWithoutAffirmation` mirror fail-class + signal-1/signal-2
  costimulation framing)
- Scout three-bucket finding: camp notice `30ccd565` (2026-05-27 — unaffirmed cessation as
  structural gap)
- ADR-028 costimulation confirmation: decisions.md:5383 (`all_of` = costimulation principle)

---

## [ADR-032] Conjunction Witnesses: Required-All Defense Semantics

**Status**: Ratified 2026-05-28.

**Participants**: antigen-dx-dogfood team (scout primary drafter; aristotle Phase-1-8
PASSED; naturalist biology PASSED; adversarial gate PASSED with three named gaps; scientist
consistency review COMPLETE; outsider naive-pass; all four ceremony signers).

**Related**: ADR-029 (per-site defense verdicts, `#[defended_by]` + `#[presents]` mechanics),
ADR-018 (defense semantics, diamond inheritance), ADR-028 (`all_of` = costimulation
principle), ADR-030 (locus-dispatch frame).

**Ceremony campsite**: `ceremony/ratify-adr-032-conjunction-witnesses`.

### Problem

The current audit verdict model uses **OR semantics** across all witnesses at a site
(`audit.rs:1381`): any single passing witness at the highest tier grants `Defended` status.
There is no way to declare: "this failure class requires BOTH a proptest witness AND a kani
proof to be considered Defended." One signal licenses the full response even when two were
required.

This gap was predicted by the autoimmune-shadow discovery engine as **shadow #3**:
costimulation collapse → conjunctive-defense void. T-cell activation requires signal-1 (TCR
binding) AND signal-2 (CD28/B7 costimulatory); signal-1 alone → anergy, not activation. The
OR semantics in antigen's audit correspond to a pathological permissiveness: any single
witness licenses Defended status when the failure class's immunity model genuinely requires
multiple independent confirmation channels.

The structural commitment to witness pluralism (ADR-029) makes this gap structurally
guaranteed to be encountered by any adopter working in a high-assurance domain (formal
verification, safety-critical code) where one tier of evidence is genuinely insufficient
alone.

**Concrete adoption scenario**: an adopter declares an antigen for a memory-safety failure
class. They want to require BOTH a proptest witness (Empirical tier: stochastic coverage)
AND a kani proof (FormalProof tier: exhaustive formal verification). Under current
semantics, `#[defended_by(kani_proof)]` alone grants Defended at FormalProof tier — the
adopter has no way to express "proptest coverage is ALSO required."

### Decision

**Syntax: `all_of` compositor on `#[defended_by]`.** Introduce an `all_of` compositor
following the fingerprint grammar precedent (ADR-010). Vocabulary consistency: the same
compositor name works at both layers.

```rust
// Require BOTH witnesses to consider this site Defended:
#[presents(MemorySafetyFailure)]
#[defended_by(all_of(PropTestCoverage, KaniProof))]
fn critical_operation() { ... }

// Existing single-witness form unchanged:
#[defended_by(SingleWitness)]

// Multiple-annotation form retains OR semantics (backward-compatible):
#[defended_by(WitnessA)]
#[defended_by(WitnessB)]   // WitnessA OR WitnessB suffices
```

`all_of(X, Y)` makes the conjunction members explicit and scope unambiguous, consistent
with existing compositor vocabulary in the DSL.

**Audit semantics: new `AuditVerdict` variant `ConjunctionIncomplete`** (aristotle Q2: NEW
VARIANT, not a `SubstrateGap` specialization).

```
ConjunctionIncomplete {
    passing: Vec<(AntigenType, WitnessTier)>,  // which conjunction members passed
    missing: Vec<AntigenType>,                  // which members are absent or failing
}
```

Subject-check: `SubstrateGap`'s subject is "witness present, predicate FAILED at the
substrate." `ConjunctionIncomplete`'s subject is "some witnesses entirely ABSENT — the
structural requirement isn't met." Different subjects, different resolution paths.

`ConjunctionIncomplete` FAILS under `--strict` (aristotle Q4): it IS an unmet defense
requirement. Strict-failing set extends to: `{Undefended, ConjunctionIncomplete,
SubstrateGap}`. A declared conjunction that is not fully satisfied is a defense shortfall.

`defense_addresses()`: a conjunctive defense is addressed only when EVERY member of the
`all_of` has a passing witness at the class level. The conjunction is a SINGLE defense intent
requiring ALL named witnesses.

**Tier interaction: two orthogonal axes, not a higher tier** (aristotle Q1).

The full state is `(tier, plurality)` — two orthogonal axes. `tier = max(members)` is
correct as a backward-compatible PROJECTION onto the tier axis, but not the full state.

- **Kind axis** (WitnessTier): what TYPE of evidence — reachability, empirical, formal.
  Subject: quality of epistemic warrant. Unchanged from ADR-029.
- **Plurality axis** (new): how MANY independent evidence kinds are required. Subject:
  quantity of independent confirmation channels.

```
WitnessState {
    tier: WitnessTier,      // max(members) — backward-compat tier projection
    plurality: Plurality,   // {Single | ConjunctionSatisfied | ConjunctionPartial{passing,missing}}
}
```

`Plurality::ConjunctionPartial` (not `ConjunctionIncomplete`) avoids collision with
`AuditVerdict::ConjunctionIncomplete`. The `Plurality` field describes the
conjunction-satisfaction STATE at data-collection time; the `AuditVerdict` names the
auditor's JUDGMENT — distinct semantic levels, distinct names. A site with
`plurality: ConjunctionPartial` deterministically yields `AuditVerdict::ConjunctionIncomplete`.

Backward-compat callers reading only `tier` see `max(members)` — correct AS a projection.
Callers reading the full `WitnessState` see the conjunction-satisfaction status on its own
axis.

**Alternative rejected**: introduce a tier above FormalProof for conjunction. Rejected
because conjunction is a defense STRUCTURE, not a witness KIND — the tier taxonomy is about
witness kinds.

**`WitnessState` as an extensible struct** (aristotle Phase-8 void): a third witness-layer
axis is structurally possible in future versions (COVERAGE per-test/per-span; FRESHNESS
staleness obligation per ADR-030; RECENCY-OF-LAST-RUN). `WitnessState` is an open struct
with named fields (not a tuple), so future axes grow additively without breaking the v0.3
representation.

### Biology (PMID 12670403, primary-source grounded)

The canonical two-signal model of T-cell activation (Bretscher–Cohn, refined by
Lafferty–Cunningham, Janeway, Matzinger). Signal-1 = TCR engagement of peptide-MHC
(antigen recognition); signal-2 = costimulatory engagement (CD28-B7 family is the
prototype; PMID 12670403 Appleman & Boussiotis, *Immunol Rev* 2003 — canonical review).

**Signal-1 without signal-2 produces ANERGY** (a distinct cellular state — hyporesponsive,
with characteristic gene-expression program, persistent, qualitatively unresponsive on
re-challenge). **Signal-1 + signal-2 produces FULL ACTIVATION** (proliferation,
effector-program, qualitatively responsive). These are NOT graded positions on one scale —
they are categorically different programs. Anergic cells are NOT "weakly activated";
activated cells are NOT "strongly anergic." The cellular machinery is distinct (anergy has
its own transcription factors, e.g. NFAT-without-AP-1; activation has the full
NFAT/AP-1/NFkB program).

**Why max-of-members-flat fails biologically** (naturalist on the record): max-of-members
would assert "activated-via-conjunction = max(activation_from_signal_1,
activation_from_signal_2) = signal_1's activation (since signal-2 alone barely activates)."
That is biology-wrong: signal-1 ALONE produces anergy, not "weak activation"; the
conjunction PRODUCES a state that NEITHER signal alone produces. The categorically-different
output is structural to the mechanism, not an artifact of measurement. Max-of-members-flat
would assert a state biology says doesn't exist. The new plurality axis is required, not
optional.

**Why (tier, plurality) validates as orthogonal axes**: TIER is the KIND of evidence
(epistemic quality). PLURALITY is whether one or multiple independent evidence-kinds are
required (epistemic quantity). In immunology: signal-1 = one binding event; signal-2 = an
INDEPENDENT binding event that must ALSO occur. Two independent inputs required → the
system produces a categorically different output. Quality-of-evidence (tier) and
number-of-independent-evidence-kinds-required (plurality) are orthogonal in immunology by
30+ years of mechanism.

**Connection to prior rulings** (naturalist): this confirms the same orthogonal-axis shape
as the silence-axis afferent/efferent split (ADR-028 Amendment 7) and the count-split
measurement-vs-parameter (ADR-030). Each time biology was asked, it produced an orthogonal
axis where the structural reasoning under-specified the joint — instrument-grade pattern
recognition per ADR-003.

### Relationship to existing surfaces

**ADR-029 (`#[defended_by]` mechanics)**: `all_of` is an extension to the `#[defended_by]`
syntax, not a replacement. The proc-macro parser accepts the compositor form; existing
single-witness and multi-annotation forms unchanged.

**ADR-018 (diamond inheritance + class-level defense)**: Conjunction requirements propagate
via class-level defense match — no new mechanism needed (aristotle Q5). A parent's
`#[defended_by(all_of(A, B))]` declares that A and B defend class X at the CLASS level.
Descendants inheriting the presentation of X (via `#[descended_from]`) receive the
conjunction check at the class level: the inherited site shows `ConjunctionIncomplete` iff
one of A/B is absent at the class level. Consequence: a descendant cannot unilaterally
satisfy a conjunction declared by a parent — class-level conjunction either passes or
doesn't for ALL sites presenting that class.

**ADR-028 (category ↔ witness-type cross-check)**: The G2 cross-check applies to individual
witness types within a conjunction. A conjunction of `[SubstrateWitness, CodeWitness]` on a
`SubstrateAlignment` antigen has the same G2 semantics as individual witnesses of those
types.

**`audit.rs:1381` (current OR semantics)**: The conjunction audit path is a separate
evaluation branch, not a replacement of the `max()` logic. Sites without `all_of` continue
using `max()` as today. Sites with `all_of` use the new conjunction evaluator. The existing
`max()` logic produces `WitnessState { tier: max_tier, plurality: Single }` for
backward-compat callers.

### Implementation notes

These are non-optional constraints on the v0.3 implementation. They live here (not only in
§Adversarial) so implementers find them without reading the ceremony record.

**Parse-time rejection of nested `all_of` (NON-OPTIONAL)**: The v0.3 proc-macro parser MUST
reject nested `all_of` in `#[defended_by]` at compile time with a clear error. Accepting
`#[defended_by(all_of(A, all_of(B, C)))]` silently would cause the audit to treat the inner
`all_of(B, C)` token as a member-name, yielding `ConjunctionIncomplete { missing:
["all_of(B,C)"] }` — a misleading error reporting a non-existent witness.

Required error message shape:
```
error: nested all_of in #[defended_by] is not yet supported — wrap all witnesses in a
single all_of([A, B, C]) instead
```

This follows the fingerprint grammar precedent (ADR-010): parse-time rejection is the safe
boundary when forward-compatibility for nesting is explicitly deferred.

**`Plurality::ConjunctionPartial` → `AuditVerdict::ConjunctionIncomplete` derivation**: When
a site's `WitnessState` has `plurality: ConjunctionPartial { passing, missing }`, the audit
MUST produce `AuditVerdict::ConjunctionIncomplete { passing, missing, tier: max(passing) }`.
The derivation is deterministic; the audit MUST NOT produce `Defended` when
`ConjunctionPartial` is the state. Any path that grants `Defended` despite
`ConjunctionPartial` is a bypass of the conjunction requirement.

### Adoption guidance

These notes are required in user-facing documentation for v0.3.

**Bypass vector: parallel plain `#[defended_by]` silences conjunction**: Once you declare a
conjunction requirement on a class (via `#[defended_by(all_of(A, B))]`), adding a parallel
plain `#[defended_by(C)]` at the same site silences the `ConjunctionIncomplete` signal via
OR semantics. The plain defense produces `Defended { plurality: Single }` which satisfies
`--strict`. The conjunction shortfall is invisible. This is the current deliberate behavior
(OR-over-all-defenses is the default for multi-annotation form). The `ConjunctionIntentDiluted`
advisory (v0.4+ research item) will surface this case explicitly; until then: **avoid adding
plain `#[defended_by]` at a site that also bears `all_of` unless you intend to bypass the
conjunction requirement.**

**Class-level vs site-level: conjunction is satisfied at the class level**: A class-level
conjunction requirement is satisfied at the class level — meaning A and B must be present
as class-level defenses, not as site-specific `#[defended_by]` annotations. A descendant
site cannot unilaterally satisfy a class-level conjunction by adding its own site-level
witness. Adding `#[defended_by(A)]` at a descendant site that presents X (where X's
class-level defense requires `all_of(A, B)`) will NOT satisfy the conjunction — the
descendant site will still emit `ConjunctionIncomplete { missing: [B] }`. **If you see
`ConjunctionIncomplete` despite adding a witness, check whether the requirement is class-
level (`#[defended_by(all_of(...))]` on the antigen declaration) rather than site-level.**

### Failure-class: `ConjunctiveDefenseVoid` (deferred to v0.4+)

Once the conjunction primitive ships, the fail-class that names its absence can be declared:
a site that presents a failure class requiring conjunction for rigorous defense but whose
`#[defended_by]` uses OR semantics, where the adopter INTENDED to require multiple
independent channels but had no syntax to express it. Shadow #3 crystallized as a named
failure class.

### Gate outcomes (ceremony complete)

**Aristotle Phase-1-8 — PASSED** (2026-05-27). Five questions resolved: Q1 (tier) new
orthogonal plurality axis + `WitnessState { tier, plurality }`; Q2 (verdict)
`ConjunctionIncomplete` = new `AuditVerdict` variant; Q3 (nesting) deferred as
forward-compatible additive extension; Q4 (--strict) `ConjunctionIncomplete` fails under
`--strict`; Q5 (inheritance) class-level composition, no new mechanism. Phase-8 void:
`WitnessState` as extensible struct.

**Naturalist (biology) — PASSED** (2026-05-27). Costimulation categorical-state confirmed,
primary-source grounded (PMID 12670403). (tier, plurality) orthogonal axes confirmed by
30+ years of two-signal mechanism. Max-of-members-flat biologically disproven.

**Adversarial — PASSED** (2026-05-27) with three named gaps incorporated: (a) bypass
vector via parallel plain defense → `ConjunctionIntentDiluted` advisory named as v0.4+
research item; (b) nested `all_of` parse-time error constraint (NON-OPTIONAL implementation
note); (c) class-level vs site-level documentation gap (adoption guidance).

**Scientist (consistency) — PASSED** (2026-05-28). Plurality::ConjunctionPartial rename
adopted, §Implementation notes and §Adoption guidance sections promoted from §Adversarial.
Outsider naive-pass addressed.

### What this ADR does NOT do

- Does NOT change OR semantics for the multi-annotation form (backward-compatible).
- Does NOT change the existing `max()` logic for sites without `all_of`.
- Does NOT introduce nesting (`all_of(X, all_of(Y, Z))`) — deferred as forward-compatible
  additive extension; v0.3 parser MUST reject with clear error.
- Does NOT declare `ConjunctiveDefenseVoid` as a dogfood antigen yet — deferred to v0.4+
  once the conjunction primitive is in use.
- Does NOT introduce `ConjunctionIntentDiluted` advisory enforcement — v0.4+ research item.

### Evidence citations

- Shadow #3 discovery: `forward/autoimmune-shadow-discovery-engine` (notice `4f07c1e7`)
- OR semantics substrate verification: `audit.rs:1381`
- ADR-029 witness pluralism commitment: §ADR-029
- Fingerprint `all_of` compositor precedent: ADR-010 (fingerprint grammar)
- PMID 12670403: Appleman & Boussiotis (2003). "T cell anergy and costimulation."
  *Immunol Rev* — canonical two-signal model, primary source
- Naturalist biology gate: campsite note `8418caca` (2026-05-27)
- Adversarial gate: campsite note `bc9cbb62` (2026-05-27)
- Aristotle Phase-1-8: `forward/adr032-conjunction-witness` campsite note (2026-05-27);
  five questions resolved; (tier, plurality) two-axis reformulation
- Scientist consistency + outsider-gap fixes: `forward/adr032-conjunction-witness` campsite
  notes (2026-05-28); Plurality::ConjunctionPartial rename; §Implementation notes; §Adoption
  guidance

---

## ADR-019 Amendment 1 — Witness Taxonomy: Two Kinds (Categorical ‖ Titer/Scalar), Each with Named Members + a Generic Escape-Hatch

**Status**: Proposed 2026-06-01 (ceremony: `prescriptive/family-adr`).

**Amends**: ADR-019 (substrate-witness predicate family).

**Participants**: aristotle (Phase 1-8 deconstruction of the Tekgy-co-designed
titer-family reframe — the W-series irreducible truths, the escape-hatch principle,
the forced-rejection voids); math-researcher (the categorical-algebra boundary that
*is* the family line; Bolotin 2020 / McIntosh 2015 method-relativity proof);
naturalist (serology/affinity-maturation/cross-reactivity biology; the limit-of-detection
third-state grounding); Tekgy (the report-not-verdict reframe that dissolved the titer fork).

**Related**:
- ADR-019 (the categorical predicate family this amendment splits the *taxonomy* of — the
  5 sealed leaves + 3 combinators become the **categorical kind**; this adds a **second kind**).
- ADR-006 Amendment 1 (recognition-not-design scoped to adopter-extension; stdlib growth is
  research-discipline — the escape-hatch is the stdlib's recognition *instrument*).
- ADR-022 (Stdlib-vs-Extension: two disciplines, one public API — the 3-rung gradient makes
  the stdlib/extension split *first-class* on the witness axis).
- ADR-003 / ADR-003 Amendment 1 (biology as discovery framework — serology predicted the
  seroconversion ‖ serotiter split *before* we named the witness-kind axis).
- ADR-029 / ADR-029 Amendment 1 (observe-don't-declare + well-posedness — titer is
  observe-don't-declare on a *continuous* axis; the un-measurable third value is the
  well-posedness frame at the measurement layer).
- ADR-024 Amendment 1 (titer's family-grouping — **this amendment supersedes its "still
  Family 3 Prescriptive" clause**; see §Titer reclassification below).
- ADR-007 (anti-YAGNI: both kinds + both rungs ship — structurally guaranteed by W1+W5).
- ADR-033 (the prescriptive family — titer *leaves* it for this taxonomy).
- The reflective/recurrence-evolve campsite (the graduation tracker is itself a titer —
  "antigen-of-antigens" made concrete).
- `adaptive-memory-bcell-persistent-recognition` campsite (graduation = affinity maturation;
  cross-reactivity = fingerprint-similarity).

### Finding

ADR-019 shipped a closed predicate grammar of **five sealed leaf primitives**
(`ratified_doc` / `signers` / `signed_trailer` / `oracles_complete` / `fresh_within_days`)
plus three combinators (`all_of` / `any_of` / `not`). Every leaf is **categorical**: it
asks a yes/no/indeterminate question and feeds a verdict (`defended` / `undefended` /
`substrate-gap`). `EvidenceKind` (`TypeSystemProof | Behavioral | SubstrateState`) is
already a *parallel axis, not an ordered scale* — a structural anticipation of what this
amendment makes explicit on a second front.

A failure-class can also have **telemetry**: a measured value read from a source —
*scan coverage = 70%*, *cyclomatic complexity = 14*, *fuzz-corpus size = 3,400*. The
`scan_coverage: Option<ScanCoverage>` field (`scan.rs`) already ships the unverdict'd
**value-floor** for the ignorance case, and its own doc-comment names it "the substrate for
ignorance detection… the audit/verdict layer is ADR scope; this field is the floor it stands
on." That value-floor has no home in the categorical grammar — and the prior attempt to give
it one (a `titer` macro whose satisfaction was `measure(M) >= threshold`) was correctly
rejected by math-researcher: the threshold that turns a value into a verdict is a
*method-relative clinical judgment* (Bolotin 2020 measles 120 mIU/mL; McIntosh 2015 SBA
titre ≥4/8 complement-source-relative), not a substrate fact. **The objection was true of
the verdict we were bolting on, not of the value underneath.**

### Decision

**The witness vocabulary is two KINDS, not one. Each kind ships named members AND a generic
escape-hatch.**

#### Two witness kinds

| Kind | Attests | Output | Members (v0.3) | Escape-hatch |
|---|---|---|---|---|
| **Categorical** (ADR-019 as shipped) | a verdict about substrate | `defended` / `undefended` / indeterminate | `ratified_doc`, `signers`, `signed_trailer`, `oracles_complete`, `fresh_within_days` (+ `all_of`/`any_of`/`not`) | the **substrate-predicate** leaf is the categorical escape-hatch (already present — this amendment makes it *symmetric* and *named-as-such*) |
| **Titer / Scalar** (new) | a measured *value* read from a source | a magnitude, **no verdict**; trend-trackable | `#[ignorance]` / scan-coverage = **member one** (retroactive recognition — no code change; the taxonomy names what `scan_coverage` already is) | raw **`#[titer(source = …)]`** — anyone, today |

A **categorical** witness attests a *verdict*; a **titer** witness attests a *value*. Both
are substrate-reads. This makes "witness" honest about what it attests. **Antigen attests the
value; the threshold-as-judgment lives downstream, outside antigen.** Antigen stays out of the
clinical-judgment business — exactly the boundary math-researcher's rigor drew. (The fork
between defer-to-v0.4 and ship-in-v0.3 is *dissolved*, not adjudicated: math-researcher was
right about the verdict, Tekgy was right about the value; report-not-verdict keeps both.)

**Biology predicts the split (ADR-003 doing predictive work).** Serology has two distinct
readings: sero*conversion* (binary — did antibodies develop? yes / no / *equivocal* — the third
value is literally a column in the lab report) and sero*titer* (continuous — how much, the highest
dilution still detectable?). Categorical-witness ‖ titer-witness *is* seroconversion ‖ serotiter.
The metaphor named the axis before we did — and the silence-test confirms it is instrument-grade,
not decoration: serology gives the titer value but says *nothing* about the protective threshold
(which titer = "immune enough"), a method-relative clinical judgment downstream of the assay.
Biology goes silent at exactly the seam where antigen does: antigen attests the value, the
threshold-judgment lives downstream (naturalist's silence-test, ADR-003).

**Why `#[ignorance]`/scan-coverage is member-one (not arbitrary).** Every categorical witness
antigen has ever shipped is a *per-site / depth* reading (this site: defended / undefended /
unreached). A titer witness is a *breadth / space-coverage* reading — a measured value over a SET
(coverage = the fraction of a space with a valid witness). Scan-coverage is therefore the FIRST
breadth reading antigen carries, which makes it the canonical titer member, not an arbitrary pick.
This split was independently predicted three weeks before the witness-taxonomy existed (scout,
2026-05-08: *instance-coverage* (depth, per-site) ‖ *taxonomy-coverage* (breadth, across-the-space)
— "a good metric covers a SPACE, not just INSTANCES within it"). Two independent derivations
landing on one joint — and SARIF's own vocabulary already carries it (per-site `result` rows =
categorical verdicts; rule-level / no-result aggregates = the breadth/titer reading), which is why
the live report's two sections (ADR-034) fall out as the two witness kinds for free.

#### The escape-hatch principle (the spine — applies to EVERY witness kind)

> **Every witness kind ships named members AND a generic escape-hatch.** Named members carry
> recognized meaning; the escape-hatch carries adopter freedom (bounded by the family
> contract) and serves as the recognition instrument for future members.

- **Three-rung gradient**: **stdlib-named** (ours, blessed, cross-cutting — `#[ignorance]`) /
  **adopter-named** (an extension crate names a member for *its* domain, same contract — the
  ADR-022 stdlib-vs-extension split made first-class) / **raw escape-hatch** (anyone, today —
  `#[titer(source = …)]`). Usage climbs the gradient; the stdlib is never the rung-one
  bottleneck.
- **Structurally-guaranteed need (W5)**: we *cannot* enumerate what an adopter's codebase will
  want to quantify, so "an unseeable ad-hoc need exists" is a certainty. The hatch is not
  designing-ahead — it is **recognizing the unseeable** (clears the ADR-006 worry: named
  members recognize what we *can* see; the hatch recognizes that we *can't see all of it*).
- **The hatch IS the recognition instrument**: in-the-wild `#[titer(source = …)]` usage *tells
  us* what is common enough to bless with a name. Remove it and recognition starves (we'd guess
  what to name next, and guess wrong). This is observe-don't-declare applied to the stdlib's
  *own growth*.
- **Guard (so it is not a dumping ground)**: freedom in the family's *domain*, bounded by the
  family *contract*. A `#[titer(source = …)]` MUST attach to a failure-class; it cannot become a
  free-floating `p99_latency` metrics sink (that is prometheus — compose-don't-compete, ADR-002).
  **Well-typedness predicate** (math-researcher Sharpening 3 — for parse-time enforceability):
  a titer-witness is well-typed IFF it is *bound to an antigen failure-class declaration* — the
  `source` reads a value *about that class's defended-state quantity*. The discriminator vs a
  prometheus metric is exact: is there a failure-class this magnitude is the titer *of*?
  `p99_latency` with no failure-class = ill-typed (reject at parse / audit-hint).
  `coverage` of `#[ignorance]` = well-typed. Pathmaker specifies this as a parse-time binding
  check, parallel to how categorical witnesses bind via `#[presents(requires = …)]`.

#### Titer witnesses are three-valued at the value layer (the gem at the measurement layer)

A titer-witness's reading is **three-valued**: `measured-value` / `below-threshold-or-floor` /
**`un-measurable`** (no source resolved, or the measurement instrument never reached a readable
signal). **`un-measurable` is NOT `below-threshold`** — it is out-of-frame (well-posedness,
ADR-029 Amendment 1). Naturalist's biology grounding makes this concrete: clinical serology
already treats *"below the assay's limit of detection"* as categorically distinct from
*"measured-and-below-protective-threshold"* — a below-LoD sample is "not detectable," NOT "zero
antibody" (the instrument couldn't see it). This is the measurement-layer instance of the
three-valued-logic invariant (the gem): the *could-not-evaluate* state is categorically distinct
from the *evaluated-and-low* state, and collapsing them is the silent-wrong-verdict bug class.
A titer-witness MUST keep `un-measurable` distinct from `below-threshold`.

**`un-measurable` is the lift's unit, not a titer special case (math-researcher).** A value-read
`measure : Substrate ⇀ Scalar` is a partial function exactly as a categorical predicate is; its
honest total form is the lift `Substrate → (Scalar + 1)`, and `un-measurable` is the *same* unit
`⊥` that the categorical kind's `substrate-gap` / `out-of-frame` is — the one terminal inhabitant
that injects into every lifted codomain (categorical or scalar). It is not special to titers; it is
what totalizing any partial substrate-read *is*. Report-not-verdict is then exactly "do not apply a
threshold function to `⊥`" — a threshold partitions `Scalar`, and `⊥` is not in `Scalar`, so a
verdict over `⊥` is a type error, not a measurement. The value layer obeys the coproduct the same
way the verdict layer does. (When the universal three-valued principle ratifies, this is one of its
instantiations — the value-read lift — not a titer-local rule.)

**Non-monotonicity as a structural argument for report-not-verdict** (math-researcher Sharpening
2): a value-witness is non-monotone in *both* directions — a re-measurement can rise OR fall
(unlike categorical attestation-leaves, which are monotone under substrate growth). Therefore
there is no `fulfilled` latch to revert; the only stable thing to attest about a both-ways
non-monotone quantity is its current value, recomputed at evaluation-time *t*. Report-not-verdict
is not only correct on the judgment-locus grounds (W2/W3) — it is *forced* by non-monotonicity.

#### Titer witnesses: staleness is provenance-relative (no silent-stale-value bug)

Value-witness staleness is **two-regime**, not uniform — split by value provenance:

- **Scan-derived titers** (`#[ignorance]` / scan-coverage, member-one): the value is a
  *live projection* of the current scan state (a pure derivation: `|scanned| / |enumerated|`,
  recomputed every scan run, carrying no stored fingerprint). It is **pin-free** — staleness is
  structurally impossible (there is no independent copy to compare a fingerprint against).
  Applying NFA-21 fingerprint-pinning to scan-derived titers is a category error. This is the
  direct corollary of W6 (the report is a live projection — never stored) applied to the
  value layer: scan-derived values *are* the report; they cannot be stale.

- **Source-read titers** (raw `#[titer(source = …)]` where the source is a file produced
  by an external tool out-of-band): the value IS stored independently and **CAN drift** from
  the code it claims to measure. These titers **must** be (i) fingerprint-pinned (NFA-21
  reuse — a value attested against fingerprint F is excluded when the code mutates to F'), and
  (ii) carry a sub-clause-F source-attestation (who produced the source file? is its generation
  attested? is the file current?). **Non-optional** for source-read titers — omitting either
  ships the silent-stale-value bug (the titer analog of silent-stale-review).

Collapsing the two regimes is a bug in both directions: pinning scan-derived titers is a
category error; leaving source-read titers unfingerprinted ships silent-stale-value. The
regime is determined at parse-time from the titer's declaration (member-one `#[ignorance]` vs
raw `#[titer(source = …)]`). Math-researcher Sharpening 1 (campsite note, 2026-06-01).

#### Graduation via adaptive memory (the recognition loop, tracking-only in v0.3)

Escape-hatch usages are tracked and nudged toward graduation (→ a named member). Biology lands
the whole shape: a generic-in-use is the immune system mounting a *broad* response to a novel
antigen it lacks a named antibody for; repeated exposure (used N×, lived M months) is the signal
to *commit adaptive memory* — form a named member (a memory B-cell); graduation = affinity
maturation; "recognize when their generic resembles something we built" = **cross-reactivity** (a
fingerprint-similarity query). **Home**: the already-seeded
`adaptive-memory-bcell-persistent-recognition` campsite — not new scope.

**The germinal center is the fleet of codebases (naturalist, ADR-003 break-test).** The variation
pool that selection acts on is NOT intra-lineage somatic hypermutation — biology is correctly
silent there (importing random-variation + Darwinian-death would be the wrong model for deliberate
recognition-naming). The variation pool is *inter-codebase*: different adopters independently name
slightly-different generics for the same quantity-shape (`coverage` / `reachability-ratio` /
`inspected-fraction`). Affinity = cross-codebase *recurrence*; the selection step = cross-reactivity
(fingerprint-similarity clustering picks the shape that recurs across many independent codebases);
memory-commitment = blessing it into the stdlib. The anti-YAGNI tell (ADR-007): the escape-hatch
recurrence-tracking is not a v0.3 convenience — it is the *manual prototype* of the germinal-center
selection substrate that a cross-codebase model runs at scale later (the innate-tier north-star).
Same loop, two scales: hand-crank now, industrial later. Three refinements that must hold:

1. **Loudness is calibrated or it backfires.** The hatch's whole job is frictionless self-service
   *and* it is our recognition instrument. If using it feels like shamed debt, adopters stop
   using it and we go blind. So: **always tracked (silent) → a one-line count + pointer in audit →
   genuinely LOUD only on a signal** (lived past a threshold, used past N×, or a strong
   cross-reactivity match). Framed as a *graduation opportunity*, never "you did wrong," never
   blocking.
2. **Some generics are correctly terminal.** A company-internal quantity that will never be
   stdlib must not be nagged forever. An **acknowledge-as-intentional** state (a scoped
   `#[allow]`-shaped sub-clause-F acknowledgment) stops the nudge — distinct from "unexamined."
   (Forced by W5's own loudness-calibration: without it the hatch becomes a nag → adopters stop →
   we go blind.)
3. **Similarity matching is three-valued** (the gem again): match / no-match / *not-sure* —
   suggest only on a definite match, stay silent on not-sure, always advisory. A confident-but-
   wrong "use `#[ignorance]` here" erodes trust faster than silence. The clippy-style autofix is
   opt-in and *later*.

**Recursive (W6):** the graduation tracker is *itself a titer* — it measures a magnitude (count
of un-promoted generics, how long they have lived) on a meta-failure-class ("un-named-quantity
debt"), reports it, no verdict. This is `reflective/recurrence-evolve` made concrete — the
taxonomy self-applies, a strong signal it carves reality at a joint. The *smart* recommendations
(fingerprint cross-reactivity, "you used this exact pattern 7×") float to v0.3.x.

### Titer reclassification (supersedes ADR-024 Amendment 1's family-grouping clause)

ADR-024 Amendment 1 stated `#[titer]` "Does NOT modify the family-grouping… (still Family 3
Prescriptive Work-Orchestration)." **This amendment supersedes that clause.** `#[titer]` is a
**titer-witness**, not a prescriptive work-need: a work-need is forward-looking *work someone must
do by a frame* (satisfaction = a who-attests); a titer *attests a measured value about a
failure-class* (no who, no deadline, no verdict). ADR-024 listed titer as prescriptive because the
witness-kind axis did not exist yet — with only the verdict-witness kind available, "a measured
quantity monitored over time" had nowhere to live but as a pseudo-work-need (the verdict-bolting
math-researcher rejected). ADR-024 Amendment 1's own words ("clinicians ORDER titer tests to
monitor… antibody concentration over time") already describe a *measurement-witness workflow*, not
a unit of code-site-local forward work. Consequence: **ADR-033 ships eight prescriptive work-need
macros; `#[titer]` moves here.** math-researcher (whose categorical-algebra boundary drew the
family line) ruled this explicitly (campsite `prescriptive/family-adr`, 2026-06-01): titer EXITS
the prescriptive family, single-home in the witness taxonomy, **not** dual-home — a titer is not a
work-need (no who-attests, no workflow, and the four-valued `WorkVerdict` does not even type over a
value). A titer *value* may be REFERENCED by a downstream judgment that gates a work-need, but that
reference is the adopter's composition (the WR8 cross-kind seam, downstream of antigen), never an
antigen macro re-bolting the threshold back in.

### What this amendment does NOT do

- Does NOT change the categorical grammar's 5 leaves / 3 combinators / parse-time
  zero-leaf-rejection (that family is the **categorical kind** unchanged).
- Does NOT bolt a threshold-verdict onto titer (the verdict-from-value is a downstream judgment,
  outside antigen — the boundary that dissolves the fork).
- Does NOT ship the *smart* graduation recommendations in v0.3 (fingerprint cross-reactivity,
  pattern-count) — tracking + one-line audit pointer only; smart recs float to v0.3.x.
- Does NOT introduce a stored member-registry that drifts — the graduation lifecycle is computed
  from substrate (usage sites + git provenance), never a parallel ledger (see ADR-034).
- Does NOT make `#[titer(source=…)]` a general metrics surface — it MUST attach to a failure-class
  (compose-don't-compete; prometheus owns free-floating metrics).

---

## [ADR-033] Prescriptive Work-Orchestration: Four Structural Shapes, Eight Clinical Names, the ADR-029 Spine Pointed at Work-Needs

**Status**: Proposed 2026-06-01 (ceremony: `prescriptive/family-adr`).

**Participants**: aristotle (Phase 1-8 deconstruction — the T-series irreducible truths, the
four-shape decomposition, the locality test, the four-valued verdict, the audit-is-the-board
forced-rejection); adversarial (Q9 spec-test corpus `atk_prescriptive_family_adr033.rs`, the
`TriageDecision` cross-check, the cardinality-collapse gate ATK-PRES-8); scientist (consistency
review + the `panel.needs ↔ filled_by` binding catch); naturalist (clinical-medicine grounding
per ADR-024); math-researcher (verdict-lattice isomorphism; titer-kind boundary); Tekgy (anchor
#3 camp-separation; the report-not-verdict reframe).

**Related**:
- ADR-024 (ratified the prescriptive family's NAMES, COMPETES decision, category, clinical-medicine
  biology). **This ADR EXTENDS ADR-024; it does not supersede it** — it specifies the arg-shapes
  ADR-024 left open and names the four-shape implementation discipline.
- ADR-024 Amendment 1 (titer biology-axis — **and** its family-grouping clause, superseded by
  ADR-019 Amendment 1; titer leaves this family for the titer-witness kind).
- ADR-019 + ADR-019 Amendment 1 (substrate-witness predicate family; the categorical kind is the
  satisfaction machinery this family reuses — no new mechanism).
- ADR-020 (cross-cutting attestation — the `who` role-ref `filled_by`/`reviewed_by`/etc. instantiate).
- ADR-029 + Amendment 1 (observe-don't-declare + well-posedness — the spine; `out-of-frame` is the
  un-evaluable third value).
- ADR-023 (deferred-defense loudness-as-discipline — the `overdue` verdict's loudness isomorphism).
- ADR-026 (`TriageDecision` VCS-rollback enum — a DISTINCT axis from `#[triage]`; disambiguated below).
- ADR-007 / ADR-006 Amendment 1 (all four shapes ship; recognize shapes, don't invent carriers).
- ADR-005 (sub-clause F at the work-need trust boundary).
- `forward/three-valued-logic-api-boundary-layer` (the four-valued verdict is the gem at the
  verdict boundary).

**Ceremony campsite**: `prescriptive/family-adr` (the Phase 1-8 F-finding notes are the witness).

### Finding

ADR-024 ratified Family 3 (Prescriptive Work-Orchestration) as named macros with the COMPETES
decision, the SubstrateAlignment+FunctionalCorrectness category, and clinical-medicine biology — but
did **not** specify arg-signatures (only Recurrent Emergence got shapes, in ADR-024 Amendment 2).
process.md records why: "§prescriptive shipped 9 macros with no arg-shape spec because adversarial
pressure did not reach them." The macros are **not yet implemented** (substrate-grep: no
`PanelArgs`/etc. in `antigen-macros/src/parse.rs`; no `pub fn panel`/etc. in `lib.rs`).

Three open design questions were carried in by the launch routing: (1) **witness-semantics** —
sidecar `signers()`, git commit-trailer, or both? (2) **different-things-from-camp** — what
code-site-local work-needs does antigen-prescriptive coordinate that is distinct from camp (anchor
#3: camp stays separate; antigen never depends on camp)? (3) **audit-output-is-the-board** — how
does "code IS the Asana board" become literal? Phase 1-8 resolved all three by **recognition
rather than design**, and surfaced a fourth finding: the family is not nine primitives — it is
**four structural shapes** with clinical names as vocabulary.

### Irreducible truths (Phase 3)

- **T1 — Locality.** Some work-needs are ABOUT a specific code site and only meaningful there; the
  need's identity IS its `(file, item-path)`. Move the code → the need moves. Delete the code → the
  need is moot.
- **T2 — Substrate-checkable satisfaction.** A work-need's satisfaction is a predicate over
  substrate — exactly the ADR-019 categorical-witness shape.
- **T3 — Temporal frame.** A work-need may carry an intrinsic frame (`due` / `response_due` /
  `re_triage_due` / `runs_until`). Forward-looking work has this in a way backward-looking defense
  does not.
- **T4 — Audit is the observer.** Code declares the need + condition + frame; the audit declares the
  verdict. Pure ADR-029 isomorphism.
- **T5 — Role-workflow.** A work-need may carry an *ordered* set of who-does-what; the roles are
  ADR-020 `who` role-refs, the ORDERING is the new content.
- **T6 — The team's work is different substrate.** Camp coordinates team/expedition-scoped work
  with a different locus (the expedition tree, not the code), granularity, and observer. T1's
  locality is precisely what camp lacks.

### Decision

#### 1. The family is FOUR structural shapes; the clinical names are vocabulary over them.

Antigen ships **four shape-parsers**, not nine bespoke parsers. The ADR-024 names are preserved as
adopter-facing vocabulary (clinical discoverability + biology grounding) and distribute across the
shapes:

| Shape | Structure | Satisfaction | Names |
|---|---|---|---|
| **S1 — Role-workflow** | ordered who-steps + optional frame + a need-set | each who-step attests (ADR-020 `who` + ADR-019 `signers`/`signed_trailer`), fingerprint-pinned | `panel`, `rx`, `refer`, `biopsy` |
| **S2 — Elimination** | a set of alternatives, each independently closeable | verdict = which alternatives survive (each rule-out carries a closing attestation) | `ddx` |
| **S3 — Ordering** | a priority total-order over a set, re-validatable | the order is attested (`triaged_by`); `re_triage_due` is a staleness frame | `triage` |
| **S4 — Frame-only** | a temporal window with a satisfaction/expiry, minimal/no who | until-passes (`quarantine`) / test-green-within-frame (`culture`) | `culture`, `quarantine` |

This is the ADR-029 move at the family scale: ADR-029 collapsed `#[immune]`'s two channels and
*dropped* `#[site_binding]`; here we recognize four shapes rather than inventing carriers. ADR-007
is satisfied (all four shapes ship). ADR-006 Amendment 1 is satisfied (recognize, don't invent).
ADR-024's named members are honored — adopters write `#[panel]`, `#[ddx]`, etc.; the implementation
routes each to its shape-parser. **Per-instance shape-fit was verified individually** against the
comprehensive-vision §7 field-sets (not by family-resemblance); each fits without lossy projection.

**`#[titer]` is NOT in this family.** Phase 1-8 found it misfiled: titer's satisfaction is a measured
*value*, not a who-attests-by-a-frame — it is a **titer-witness** (ADR-019 Amendment 1), not a
work-need. The prescriptive family ships **eight** work-need macros (S1 `panel`/`rx`/`refer`/`biopsy`,
S2 `ddx`, S3 `triage`, S4 `culture`/`quarantine`). (`#[ignorance]`/scan-coverage is titer member-one;
`#[titer(source=…)]` is the titer escape-hatch — both live in ADR-019 Amendment 1.)

#### 2. Witness-semantics: reuse the ADR-019/020 categorical spine; no new mechanism.

The "signers vs trailer vs both" question dissolves into recognition. The who-refs (`filled_by` /
`reviewed_by` / `ordered_by` / `triaged_by` / `investigator` / `deep_investigation_by`) are ADR-020
`who` role-refs; only the ORDERING (S1) is new content. "Is `reviewed_by = bob` satisfied?" =
`signers(required = [bob])` over the site's `.attest/` sidecar (TextStamp default) OR
`signed_trailer(key = "Reviewed-by", role = bob)` for GitTrust strength — **adopter's choice via
`allowed_types`**, exactly as for defense attestation. Satisfaction is **fingerprint-pinned**
(ADR-019 NFA-21): a review at fingerprint F stales when the code mutates to F'. **Non-optional** —
omitting it ships the silent-stale-review bug.

#### 3. The verdict vocabulary is four-valued.

Forward-looking work has an intrinsic frame (T3), so the prescriptive verdict set is distinct from
the defense verdict set:

- **pending** — declared, within frame, satisfaction not yet met (expected, not a failure).
- **fulfilled** — satisfaction met at current fingerprint.
- **overdue** — past the frame and unsatisfied. **Loud** (ADR-023 loudness isomorphism).
- **out-of-frame** — the satisfaction condition is un-evaluable in current substrate (an unknown
  who-ref, a missing source). ADR-029 Amendment 1 well-posedness: this is NOT "overdue"; it is
  outside the well-posedness frame. The three-valued-logic gem applied to prescriptive —
  `out-of-frame` is the third value the v0.2 cardinality-collapse bugs teach us to keep distinct.

**Verdict-lattice isomorphism (math-researcher):** the four-valued prescriptive verdict is NOT a new
lattice — it is the defense tri-state (`defended` / `undefended` / `substrate-gap`) with the
unsatisfied cell *temporally split* by the frame: `undefended` splits into `pending` (within frame)
+ `overdue` (past frame); `substrate-gap` maps to `out-of-frame`. **Implementation consequence:
REUSE the ADR-029 audit evaluator; do NOT fork a parallel prescriptive evaluator** — forking
re-introduces the cardinality-collapse the gem warns against (two evaluators drifting on the same
three-valued domain). One evaluator, one substrate read, a frame-aware projection at render time.

#### 4. Audit-output IS the board (a required section, not a new tool).

"Code IS the Asana board" is a **required first-class section of `cargo antigen audit`**, not a new
renderer/dashboard. Phase-8 forced rejection proves it: if audit does not surface prescriptive
verdicts, the family collapses back into the `// TODO` it replaces (inert). The audit groups
prescriptive verdicts by site — `{macro, need-text, verdict, who/what blocks,
frame-remaining-or-elapsed}` — with overdue sorted to top (loud). Rendered from substrate, single
source of truth, no external tool, no drift (consistent with ADR-034: the board is a live
projection, never a stored tracker).

#### 5. The antigen-prescriptive ↔ camp boundary is a falsifiable test, not a feature list.

Per anchor #3 (Tekgy, 2026-06-01): camp stays separate; antigen never depends on or reads camp. The
discriminator:

> **If this exact code site vanished, does the work-need vanish with it?**
> **YES → antigen-prescriptive** (code-site-local; satisfied-or-moot by the code's existence;
> observed by `cargo antigen audit`). **NO → camp** (the need survives the code: a team decision, an
> expedition arc, cross-file coordination; observed by camp).

Phase-8 VOID-3 proves it: remove locality (T1) from a work-need and it BECOMES a camp campsite — the
void left by removing locality IS camp. So antigen-prescriptive = "camp's work-need shape (need + who
+ frame + satisfaction) MINUS the team-locus PLUS the code-site-locus." Same shape, different anchor,
different observer. The boundary asserts itself structurally in the `#[triage]` shape itself: the
comprehensive-vision §7 sketch had `triage` ordering camp *campsites* — but a triage that orders camp
campsites would be reading camp state, which anchor #3 forbids. The locality test resolves it: a
triage that orders camp campsites is *camp's* job (the need survives the code); a triage that orders
**code sites** by priority is antigen's (the need is moot if the sites vanish). So `#[triage]`'s
`priority_order` entries are **code-site references**, not camp campsites — the `campsites` field is
dropped (see the §Proc-Macro-Surface S3 transcription correction). The one allowed cross-tool bridge
is camp-side ingestion of antigen's scan-JSON (camp pulls; antigen never pushes-into or reads-from
camp) — a separate, camp-side concern, NOT antigen's job.

### Mechanics

#### §Proc-Macro-Surface (Q1 — per-primitive arg-signatures)

All fields parse-time validated. Unknown-field errors emit the full enumerated set (process.md Q1).
Each `Args` struct ships a doc-comment citing this ADR. Source field-sets: comprehensive-vision §7.
`who`-typed fields are ADR-020 role-refs (`Vec<String>` or `String`); frame fields are ISO-8601 date
strings (advisory parse, audit-time evaluated).

**S1 — Role-workflow:**

| Primitive | Field | Type | Required | Default | Constraint |
|---|---|---|---|---|---|
| `panel` | `needs` | `Vec<String>` | YES | — | non-empty (empty = vacuous; compile error) |
| | `filled_by` | `Vec<String>` | NO | `[]` | who-ref; trimmed; no dup |
| | `reviewed_by` | `Vec<String>` | NO | `[]` | who-ref; trimmed; no dup |
| | `ordered_by` | `Option<String>` | NO | `None` | who-ref |
| | `due` | `Option<String>` | NO | `None` | ISO-8601 date |
| `rx` | `treatment` | `String` | YES | — | non-empty |
| | `diagnosis` | `Option<String>` | NO | `None` | opaque label (v0.3; backref to ddx not resolved — VOID-4b) |
| | `filled_by` | `Vec<String>` | NO | `[]` | who-ref |
| | `reviewed_by` | `Vec<String>` | NO | `[]` | who-ref |
| | `due` | `Option<String>` | NO | `None` | ISO-8601 |
| `refer` | `to` | `String` | YES | — | who-ref (external owner) |
| | `response_due` | `Option<String>` | NO | `None` | ISO-8601 |
| `biopsy` | `location` | `String` | YES | — | sub-site pointer (opaque label v0.3) |
| | `request_text` | `String` | YES | — | non-empty |
| | `deep_investigation_by` | `Option<String>` | NO | `None` | who-ref |

**S2 — Elimination:**

| Primitive | Field | Type | Required | Default | Constraint |
|---|---|---|---|---|---|
| `ddx` | `symptom` | `String` | YES | — | non-empty |
| | `rule_out` | `Vec<String>` | YES | — | non-empty (the alternative-set) |
| | `investigator` | `Option<String>` | NO | `None` | who-ref |
| | `reviewer` | `Option<String>` | NO | `None` | who-ref |

**S3 — Ordering:**

| Primitive | Field | Type | Required | Default | Constraint |
|---|---|---|---|---|---|
| `triage` | `priority_order` | `Vec<String>` | YES | — | code-site references (file/item-path), in priority order; non-empty |
| | `triaged_by` | `Option<String>` | NO | `None` | who-ref |
| | `re_triage_due` | `Option<String>` | NO | `None` | ISO-8601 (staleness frame, not deadline) |

> **Transcription correction (2026-06-01, post-ratification fixup — NOT a new
> decision):** the `campsites` field is **DROPPED** per the Tekgy ruling already part
> of this ceremony (campsite `prescriptive/family-adr` note `57e56ecc`: "DROP
> triage.campsites ENTIRELY. antigen's `#[triage]` triages CODE — code-local
> work-needs/sites by priority. NOT camp campsites"). The original §Proc-Macro-Surface
> table mistakenly carried `campsites` (inherited from the comprehensive-vision §7 sketch)
> despite the ruling to drop it; the adversarial spec-test corpus
> (`atk_prescriptive_family_adr033.rs`, ATK-PRES-14) correctly encoded the ruling. This
> fixup realigns the table to the ratified ruling and the oracle. `#[triage]` orders
> **code sites** (per the anchor-#3 locality test: a triage that triages camp campsites
> would be camp's job, not antigen's); `priority_order` entries are code-site references,
> resolved like any other cross-site reference (ADR-017 Amendment 1).

**S4 — Frame-only:**

| Primitive | Field | Type | Required | Default | Constraint |
|---|---|---|---|---|---|
| `culture` | `test_kind` | `String` | YES | — | non-empty |
| | `duration` | `Option<String>` | NO | `None` | duration string |
| | `runs_until` | `Option<String>` | NO | `None` | ISO-8601 |
| `quarantine` | `scope` | `String` | YES | — | isolated-region pointer |
| | `until` | `Option<String>` | NO | `None` | ISO-8601 |
| | `reason` | `String` | YES | — | non-empty (ADR-005 Amd2 rationale-as-required) |

(`#[titer]` arg-shape lives in ADR-019 Amendment 1, the titer-witness kind — `source` + the
three-valued reading. It is NOT a prescriptive macro.)

#### §Witness-binding (Q4 — `panel.needs ↔ filled_by` binding, scientist's catch RESOLVED)

`panel.needs` (the need descriptions) and `filled_by` / `reviewed_by` (who fulfills/reviews) bind by
**collective coverage over the need-set, attested per role-step, NOT 1:1 parallel-array**:

- A panel's `needs` is the *battery's checklist* — what the panel as a whole must cover. `filled_by`
  and `reviewed_by` are the *role-steps* that close the battery. Satisfaction = **each declared
  role-step has an attestation at the current fingerprint** (e.g. every name in `reviewed_by` has a
  `signers`/`signed_trailer` entry), AND the workflow order holds (a `reviewed_by` attestation is
  only counted if the corresponding `filled_by` step is itself attested — you cannot review what is
  not filled). The verdict is the **conjunction over role-steps**, not a per-need pairing.
- This rejects the silent-wrong-verdict the ambiguity invited: there is NO positional `filled_by[i]
  ↔ needs[i]` coupling (which would mis-fire when the arrays differ in length or order). `needs` is
  documentation of *what the battery is for*; the audit does not pair individual needs to individual
  people. Rationale: the biology-faithful reading of a clinical panel is an *ordered battery whose
  closure is attested by its reviewing clinician(s)*, not a row-by-row sign-off matrix — and the
  positional reading is brittle (re-ordering `needs` would silently re-pair people to different
  needs). If finer per-need closure is ever wanted, it is a *separate need decomposed into its own
  panels*, not a hidden parallel-array contract.
- `filled_by` / `reviewed_by` may be shorter than `needs` (some needs unfilled) — that yields
  `pending` (within frame) or `overdue` (past frame) for the *site*, never a parse error.

**Workflow-order in S1 — `reviewed_by` requires ALL `filled_by` attested (conjunction, not
disjunction)** (adversarial gap, camp question `ae2e3a2d`). "You cannot review what is not filled":
a `reviewed_by` attestation counts toward fulfillment only when **every** `filled_by` role-step is
itself attested at the current fingerprint. For a multi-member `filled_by`, the discipline is ALL,
not ANY — a reviewer attests the *completed battery*, not a partial one. So the site-level verdict
is the conjunction over the ordered chain (`ordered_by` → all of `filled_by` → all of `reviewed_by`);
`Pending` until the chain closes, `Overdue` if the frame elapses with the chain open, `OutOfFrame`
if any who-ref is unresolvable. (A `reviewed_by` attestation present while a `filled_by` step is
un-attested is not "partial fulfillment" — it is a reviewer attesting prematurely; the audit does
not credit it until the filled step it depends on is attested.)

**`ordered_by` is the opening attestation — it never alone fulfills.** `ordered_by` is the
*requester* step (who commissioned the work); `filled_by` / `reviewed_by` are the *closing* steps
(who did and reviewed the work). `Fulfilled` requires at least one closing step attested. A panel
with `ordered_by` but no `filled_by` is evaluable-and-unsatisfied (`Pending` or `Overdue`, depending
on the frame) — never `Fulfilled`. A chain with no closing steps at all (`filled_by` empty AND
`reviewed_by` empty) is `OutOfFrame` / `MissingWorkStep` (structurally un-evaluable). This
distinguishes (a) truly-open-work-need with only an opener from (b) truly-empty-chain (no who-refs
at all) — both are non-`Fulfilled`, but only (b) is `OutOfFrame`.

#### §Verdict semantics per shape (Q-gap — S3/S4 `Fulfilled` reachability, adversarial `ae2e3a2d`)

Not every shape reaches `Fulfilled` the same way; two need explicit verdict semantics so `Fulfilled`
is neither structurally-unreachable nor a bypass:

- **S3 (`triage`) is a standing re-validated ORDERING, not a terminal task.** Its natural states map
  onto `WorkVerdict` thus: **Fulfilled** = `triaged_by` attested AND within `re_triage_due` AND all
  `priority_order` code-site refs resolve (the ordering is current and well-posed); **Overdue** =
  `re_triage_due` elapsed (the ordering is stale — re-triage owed); **OutOfFrame** = a
  `priority_order` entry does not resolve to a real code site (ADR-017 Amd1 — never silent-satisfied);
  **Pending** = declared, not yet `triaged_by`-attested, within frame. So `Fulfilled` IS reachable
  for S3 — it means "the priority ordering is current and resolvable," re-earned each `re_triage_due`
  cycle. (`triaged_by` alone does NOT permanently fulfill — the `re_triage_due` frame makes
  fulfillment expire, which is what keeps a triage honest; this is the freshness discipline, not the
  bypass it guards against.)
- **S4 (`culture` / `quarantine`) Fulfilled requires a POSITIVE closure, never frame-expiry alone**
  (this is the `fresh_through`-bypass class, ATK-FT-1/2 — guard against it here too). `culture`:
  **Fulfilled** = the named `test_kind` is green at audit-time within `runs_until`; **Overdue** =
  `runs_until` elapsed without a green reading; never "Fulfilled because the date passed."
  `quarantine`: **Fulfilled** = the `scope` is released by a positive event (a release attestation OR
  the named upstream fix landing); **Overdue** = `until` elapsed with the scope still quarantined and
  no release; **Pending** = within `until`, scope still isolated (the expected state). Frame-expiry
  (`until`/`runs_until` passing) makes an un-closed S4 site **Overdue**, NOT Fulfilled — expiry is the
  frame elapsing, and a frame elapsing without closure is exactly what `Overdue` means. A site that
  is `Fulfilled` purely because its deadline passed would be the temporal analog of the
  forged-freshness bypass; the positive-closure requirement forbids it.

#### §titer (relocated)

`titer` is reclassified out of the prescriptive family into the **titer-witness kind** (ADR-019
Amendment 1). It ships in v0.3 as report-not-verdict (a measured value, no verdict, three-valued
reading). The prescriptive family ships eight work-need macros. The old "titer defers to v0.4 / ships
8/9" framing is **superseded** by the report-not-verdict reframe — titer ships *now*, just not as a
prescriptive macro.

#### §Enforcement-Surface (Q8)

| Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation |
|---|---|---|---|
| Empty `needs`/`rule_out`/`reason`/`priority_order` rejection | parse-time | client + CI | compile error; joins the vacuous-guard class (EmptySignersList et al.) |
| who-ref satisfaction (signers/trailer) | audit-time | client + CI | per ADR-020 `allowed_types`; TextStamp is documentation-quality unless GitTrust/CryptoSigned selected (named limitation, ADR-020) |
| fingerprint-pinned staleness | audit-time | client + CI | reuse ADR-019 NFA-21; stale satisfaction excluded mechanically |
| overdue gate | audit-time | client + CI | configurable per-macro via severity (ADR-008 Amd1 pattern); friction-only by default |
| `triage.priority_order` code-site resolution | audit-time | client + CI | entries are code-site refs (ADR-017 Amd1); an unresolvable ref = **out-of-frame**, never silent-satisfied (the gem) |
| `rx.diagnosis` / cross-need backref | NONE (named limitation, v0.3) | — | opaque label; dependency-graph resolution deferred to v0.4 (VOID-4b) |

**Friction-vs-structural disclosure:** this ADR enforces work-need satisfaction at friction-only
level by default (audit-time hints + client hooks). Friction-only makes overdue/unsatisfied work
*deliberate* (loud in audit) rather than *accidental*, but does not prevent a determined adopter
from ignoring the audit. Adopters requiring structural mode gate CI on `cargo antigen audit`
prescriptive verdicts (the overdue gate).

#### §Standing-Pressure-Audit (Q2/Q3/Q5/Q6/Q7/Q9)

- **Q2 (sealed enums):** introduces the `WorkVerdict` sealed enum `{Pending, Fulfilled, Overdue,
  OutOfFrame}`. Axis: the satisfaction-state of a forward-looking work-need within its frame.
  Inclusion: a state the audit can distinguish from substrate. Exclusion: no "partial" variant (a
  multi-step S1 panel reports per-step; the site-level verdict is the conjunction — `Pending` until
  all fulfilled). `Overdue` (frame elapsed, evaluable) vs `OutOfFrame` (un-evaluable) is the
  load-bearing distinction — **never collapse** (the cardinality-collapse class). Extension requires
  ADR amendment.
- **Q3 (controlled vocab):** the eight macro NAMES are Tier-1 sealed (ADR-024-ratified; extension =
  amendment). The four SHAPES are an implementation taxonomy, not adopter-facing vocab. who-ref
  values are Tier-3 adopter-open.
- **Q5 (cross-primitive interaction):** two distinct reference KINDS, resolved differently.
  Cross-**site** references — `triage.priority_order` entries (code sites) — ARE resolved at
  audit-time via the ADR-017 Amendment 1 machinery (resolvable → ordered; unresolvable →
  out-of-frame, never silent-satisfied). Cross-**need** references — `rx.diagnosis` → a `ddx`,
  `panel.blocks` → downstream needs (deferred field) — stay OPAQUE LABELS in v0.3 (the
  prescriptive dependency-graph, VOID-4b, is v0.4). The split is the locality test again: a code-site
  ref points at a thing the scanner can resolve; a cross-need ref points at another work-need's
  identity, which requires the dependency-graph primitive antigen has not yet built. Convergent-evidence interaction:
  comprehensive-vision §7 lists `#[panel]`/`#[ddx]` as "(also prescriptive)" under Convergent.
  **Substrate-grep correction:** `#[diagnostic]` IS implemented (the convergent sibling), but
  `#[panel]`/`#[ddx]` are NOT — the dual-listing was aspirational, never built. ADR-033 owns
  `panel`/`ddx` cleanly with no shipped conflict. The DESIGN principle holds (Phase-8 VOID-2: the
  family axis is a reading-direction, not a partition — a forward-read `#[panel]` asks "will the
  battery be filled by due?", a backward-read asks "did enough witnesses converge?"); a backward
  reading, if ever wanted, composes the existing `#[diagnostic]`.
- **Q6 (deprecation):** additive. No prior shape superseded. ADR-024's named macros gain their
  arg-signatures; no existing caller breaks (the macros were never implemented).
- **Q7 (named-surface check):** cross-ADR substrate-grep performed for all eight names — ADR-024-owned,
  no collision. `WorkVerdict` / `WorkShape`: substrate-grep confirms no collision with existing
  `AuditHint` / `WitnessTier` / `Plurality` / `TriageDecision`. **`TriageDecision` (ADR-026)** is the
  VCS-rollback classification (Black/Red/Yellow/Green/White) — a DISTINCT axis from `#[triage]`
  work-need priority ordering, and from `#[triage_commit]` (ADR-026). Disambiguate at definition:
  ADR-026 `TriageDecision` = VCS rollback classification; ADR-033 `#[triage]` = work-need priority
  ordering. Names rhyme; surfaces are unrelated (adversarial cross-check PASSED, ATK-PRES-10).
- **Q9 (spec-adversarial pre-impl tests):** the `#[ignore]`'d corpus is shipped in
  `antigen/tests/atk_prescriptive_family_adr033.rs` (adversarial gate). Critical guard: ATK-PRES-8 —
  `WorkVerdict::OutOfFrame` (un-evaluable: who-ref unknown) must NOT collapse to
  `WorkVerdict::Overdue` (frame elapsed) — the prescriptive analog of ATK-3V-4. Tests flip from
  `#[ignore]` to active when pathmaker ships the macros. (The Q9 corpus must drop the `titer` row
  per the titer-relocation to ADR-019 Amendment 1. On `triage`: the corpus's ATK-PRES-14 was
  CORRECT — `triage` triages code sites, not camp campsites — and the §Proc-Macro-Surface S3 table
  has been fixed (transcription correction) to match the corpus + the Tekgy ruling. ATK-PRES-14's
  flagged "priority_order non-resolution tier unspecified" is now resolved: unresolvable
  code-site ref = out-of-frame per ADR-017 Amendment 1, see §Enforcement-Surface.)

### Sweep-level consequences

- antigen-macros gains 8 macro entry points routing to 4 shape-parsers (`WorkShape::{RoleWorkflow,
  Elimination, Ordering, FrameOnly}` internal dispatch).
- antigen-core gains the `WorkVerdict` sealed enum + the prescriptive audit section.
- cargo-antigen audit gains the required board-rendering section (a live projection — ADR-034).
- Reuses (no new build): ADR-019 categorical substrate-witness evaluator, ADR-020 attestation parse,
  ADR-019 NFA-21 fingerprint-filtering.
- Seeds one v0.4 void: prescriptive-dependency-graph (cross-need resolution, VOID-4b).

### Resolves

- The arg-signature gap ADR-024 left open for the prescriptive family.
- The witness-semantics question (by recognition — reuse the ADR-019/020 categorical spine).
- The audit-is-the-board question (a required audit section, live-projected).
- The different-things-from-camp question (the locality test).
- The dual-family smell (panel/ddx are reading-directions over a shared object, not duplicates).
- The `panel.needs ↔ filled_by` binding ambiguity (scientist's catch — collective coverage, not
  positional pairing).
- The titer misfiling (titer is a witness kind, not a work-need — relocated to ADR-019 Amendment 1).

### What this ADR does NOT do

- Does NOT supersede ADR-024 (extends it).
- Does NOT include `#[titer]` (relocated to the titer-witness kind, ADR-019 Amendment 1).
- Does NOT resolve cross-**need** references (`rx.diagnosis`, `panel.blocks`) — opaque labels in
  v0.3; the prescriptive dependency-graph is deferred to v0.4. (Cross-**site** references —
  `triage.priority_order` code sites — ARE resolved at audit-time per ADR-017 Amendment 1.)
- Does NOT read or depend on camp (anchor #3).
- Does NOT close the semantic gap (a hollow attestation that doesn't reflect real review work — the
  same open research question as ADR-029 §honest-semantic-gap, inherited).

---

## [ADR-034] The Report Is a Live Projection, Never a Stored Truth

**Status**: Proposed 2026-06-01 (ceremony: `prescriptive/family-adr`, folded from the
titer-family-and-reporting-codesign).

**Participants**: Tekgy (the correction that a stored report is itself a parallel state tracker —
antigen committing the sin it exists to catch); aristotle (Phase 1-8 forced-rejection VOID-W6 — the
self-contradiction proof; the rhyme with the three-valued-logic type-law).

**Related**:
- ADR-029 (observe-don't-declare — this ADR is the same principle applied to the report's *storage*).
- ADR-019 Amendment 1 (titer trends + escape-hatch lifetimes are computed from substrate, never a
  stored ledger).
- ADR-033 (the prescriptive board is a live-projected audit section).
- `forward/three-valued-logic-api-boundary-layer` (VOID-W6's "a stored report is ill-typed" is the
  same structural law as "a 2-valued substrate-boundary is ill-typed" — substrate-relative things
  must not be frozen into a parallel copy).
- The dogfood `ParallelStateTrackersDiverge` antigen (the exact failure-class a stored report would
  commit).
- ADR-017 (canonical declaration site; the report's envelope carries the git SHA + version it was
  computed against).

### Finding

A natural instinct (the superseded `.antigen/reports/` persistence/versioning floor) is to *store*
the defense-posture report — a release-anchored snapshot. But **a stored report is itself a parallel
state tracker**: the moment it is stored it can drift from the code, which is
`ParallelStateTrackersDiverge` — antigen's *own* canonical failure-class. A tool that exists to catch
parallel-state-divergence cannot ship a stored report without committing its own sin (project-level
autoimmunity).

### Decision

**The report is a LIVE PROJECTION of the code, recomputed every run — never a stored truth.** Exactly
how clippy reflects current source on every invocation. The code is the source of truth; the report
is a *view*, never a copy.

- **Primary mode = recompute on current state, whenever/wherever.** `cargo antigen audit` / `scan`
  already read the current code — the report *is* their output. There is **no "report subsystem" to
  build**; scan/audit are already live and the report inherits it. Invoke on demand, in CI, in a
  **pre-commit hook (the v0.3 stopgap delivery)**, in rust-analyzer later — the way any lint runs.
  Always current because always recomputed.
- **Loudness, reframed:** console = one-line summary; `--output <file>` = full detail. But the file is
  **output-of-a-run** (a render, like a clippy SARIF dump), NOT stored state antigen reads back as
  authoritative.
- **The report envelope (genuinely new v0.3 work, small + additive).** A live-projection render that
  claims "this is the v0.3.0 posture" MUST carry what it was computed against, or it is
  unverifiable/unreproducible. The envelope = `{ antigen_version, git_sha, generated_at,
  schema_version }`, extending the stabilized scan-json (the `ScanReport` `#[serde(default)]` additive
  pattern). **Note: this is NOT recognition — `ScanReport` has no envelope today; pathmaker builds it
  fresh.** It is the only genuinely-new piece of the reporting model; everything else is assembly.
- **Release SBOM = a reproducible *render* of a tagged state, not a stored truth.** Running
  `cargo antigen audit` at the `v0.3.0` tag *is* the v0.3.0 defense-posture SBOM, regenerable any time
  by re-running antigen at that tag. Keep/attach the file if useful — antigen never reads it back as
  authoritative, so it cannot drift.
- **Git is the only memory.** Titer *trends* and escape-hatch *lifetimes* never need a stored
  report-trail: recompute at HEAD *and* at the prior git point and diff (`blame` / `log` on the usage
  site → first-commit, last-release value). Same engine as `recurrence-automation`'s git-mining.
  Nothing stored as truth; the code's own history is the memory.

### Why this is forced, not chosen (Phase-8 VOID-W6)

Reject "live projection" and accept a stored report: antigen then commits its OWN canonical
failure-class (`ParallelStateTrackersDiverge`) inside the very tool that exists to catch it — a
self-contradiction. The void's shape is the proof: a two-state report model (`stored | absent`) is
*ill-typed*; only live-projection is *well-typed*. This rhymes exactly with the three-valued-logic
gem's "a 2-valued substrate-boundary is ill-typed" — the report-storage question and the third-value
question are the **same structural law**: substrate-relative things must not be frozen into a parallel
copy that can lie.

### Scope

- **v0.3 floor:** live report output (audit/scan already recompute) — console summary + `--output`
  detail + the report envelope + basic recommendations (counts + "consider naming") + git-derived
  provenance/trend + a pre-commit hook (stopgap delivery).
- **Additive / float (v0.3.x–v0.4):** the per-release SBOM *render* wired into `release.yml` (a
  one-line `audit --output` at the tag); the lint-style autofix; a `cargo antigen suggest`
  upstream-feature-request channel. Each composes with something external — clean later-adds, not v0.3
  gates.

### What this ADR does NOT do

- Does NOT build a report-storage subsystem (the superseded `.antigen/reports/` floor) — that would
  commit `ParallelStateTrackersDiverge`.
- Does NOT make `--output` files authoritative — they are renders, never read back as truth.
- Does NOT require the envelope to encode anything antigen cannot recompute (version + SHA + timestamp
  + schema_version only).

---

## [ADR-035] Cardinality Collapse at a Trust Boundary: the Three-Valued Type Law (a Self-Applying Antigen)

**Status**: Ratified 2026-06-01 (ceremony: `forward/adr035-three-valued-type-law-ceremony`, 3/3
co-signers: aristotle, math-researcher, adversarial). The adversarial falsification gate (no
counterexample to the no-total-boundary lemma), the math-researcher coproduct-closure formal
sign-off, and the aristotle first-principles co-sign all landed as part of the ceremony. The
ceremony campsite reached 3/3 complete on 2026-06-01 (third night).

**Participants**: aristotle (the "forced, not found" necessity argument; the Phase-1-8 forced-rejection
of the two-valued boundary and of the atomic-`⊥`; the 13-instance catalog; the `⊥` notation-collision
catch; the self-applying-witness recognition); math-researcher (the coproduct closure — the third value
is the unit of the lift, distinct by the universal property; the no-base-case regress; the Σ(stages)
two-clause discriminator; the formal sign-off that the falsification partition is *exhaustive*);
adversarial (the falsification gate — examined every substrate-relative boundary in the audit pipeline
and found NO counterexample to the lemma); naturalist (the ignorance≠anergy biology grounding; the
"applied-at-the-load-boundary-but-not-propagated-to-the-leaves" granularity-gap framing); scientist
(the six-witness convergence manuscript framing).

**Related**:
- ADR-029 (observe-don't-declare). This ADR is the **type** of which ADR-029 is the **discipline**:
  observe-don't-declare says "don't claim a verdict you couldn't observe"; this law says "your verdict
  type *has a value* for 'couldn't observe.'" Same principle, two faces.
- ADR-034 (the report is a live projection). ADR-034 already names this identity: "a stored report
  (`stored | absent`) is ill-typed; only live-projection is well-typed" is the *same* structural law as
  "a 2-valued substrate-boundary is ill-typed."
- ADR-010 Amendment 6 (Match3 — three-valued predicate evaluation at the fingerprint leaf-algebra).
- ADR-024 (cross-crate reference resolution — an unresolvable reference is out-of-frame, distinct from
  resolved-and-undefended).
- ADR-033 (`WorkVerdict::OutOfFrame` — the prescriptive work-need un-evaluable value; ATK-PRES-8 guards
  `OutOfFrame !-> Overdue`).
- ADR-019 Amendment 1 (the titer/scalar value layer — `measure : Substrate ⇀ Scalar` lifts the same
  way; un-measurable is the lift's unit; report-not-verdict = "don't apply a threshold to `⊥`").
- The dogfood `ParallelStateTrackersDiverge` antigen (the storage-face instance, via ADR-034).

### Finding

Antigen kept re-deriving the same shape from independent entry points — the fingerprint leaf-algebra
(Match3), the cross-crate resolution gate (out-of-frame), the prescriptive verdict (`OutOfFrame`), the
coverage frontier (`UnreachedCause`), the titer value layer (un-measurable), the stored-report question
(ADR-034), the categorical verdict (`Indeterminate`), the temporal-freshness and version-parse leaves
(the FT-1/2/3 bugs). Each looked like a separate design choice; each landed a *third value* beside
pass/fail. The question this ADR answers is **WHY the third value appears at every trust boundary with
necessity, rather than just appearing a lot** — the difference between a habit (catch it case-by-case,
forever) and a law (ratify the necessity once; make a two-valued boundary a compile-class error).

The answer is that the third value is **forced, not found**, and the forcing is a theorem:

1. A substrate-relative trust boundary is an evaluation `eval : Substrate ⇀ {true, false}` — a
   **partial** function, undefined exactly where the read fails (substrate absent / unreachable /
   malformed). Partiality is not a defect to engineer away; it is the honest type.
2. The honest *total* form of a partial function is its **lift**: every `A ⇀ B` is exactly a total
   `A → (B + 1)`, where `+` is the coproduct and `1` is the unit `⊥` ("undefined here").
   Substituting `B = {true, false}` gives `eval_total : Substrate → ({true, false} + {⊥})` — a
   **three-element codomain**. The third value is not *added*; it is what totalizing a partial function
   *is*. It is provably distinct from both `true` and `false` by the coproduct's universal property —
   not a stipulation.
3. So `partial ⟹ 3-valued codomain` is **unconditional** (a definition). The entire empirical content
   collapses onto one lemma: **is every substrate-relative boundary genuinely partial?** This reduces
   to a regress with no base case — "S is guaranteed present" is itself a proposition read from some
   `S'`, which is partial; there is no bottom, because *knowing-here IS reading*. The evaluator cannot
   step outside what it evaluates to certify the evaluation will succeed; that certification is another
   read.

**The lemma was put to a falsification gate and confirmed.** Adversarial examined every
substrate-relative boundary in antigen's audit pipeline for a *total* one (no `⊥` branch, partiality
not paid upstream) and found NO counterexample. Math-researcher signed off that the partition is
**exhaustive**: every such boundary is either (a) a genuine eval-time substrate read, correctly lifted
to 3+ values (`load_sidecar`, `compute_presentation_verdicts`, `AuditHint`); or (b) a read of
already-materialized in-memory data, correctly 2-valued with partiality paid upstream at scan time (the
`&ScanReport` audit functions); or (c) a 2-valued internal helper where both false-reasons route the
*same* correct downstream outcome — an allowed lossy projection (`is_complete`,
`priority_order_ref_resolves`). (a)+(b)+(c) is exhaustive; there is no fourth cell, hence no room for a
counterexample. **The lemma is a theorem.**

### Decision

**A substrate-relative trust boundary whose verdict type has two inhabitants over a partial domain is
ill-typed. The honest codomain carries the third value, and the honest tool never discards it.** This
ratifies as a self-applying antigen — antigen detecting its own type-discipline violation. The
principle has **two layers**, with distinct forcing conditions.

**Layer 1 — `CardinalityCollapseAtTrustBoundary` (the silent-wrong-VERDICT; unconditionally forced).**
The honest codomain of a substrate-relative boundary is `B + 1`. Collapsing the third value into `B`
forces the un-evaluated case to be reported as pass or fail — both lies ("the check ran" when it
didn't). Pass-on-unevaluable is the dangerous lie (silent false-green); fail-on-unevaluable is the
noisy lie (false alarms drown the signal; the adopter disables the audit — antigen's own
adoption-failure / autoimmunity mode). Both are unacceptable, which is *why* the third value must be
not merely present but **loud and distinct**.

**Layer 2 — `SubCauseCollapseInTheUnit` (the silent-wrong-REMEDY; conditionally forced).** The third
value is not always atomic. When `eval` is a **composite of staged partials**
(`evaluate ∘ match ∘ parse ∘ enumerate`) and the failure-stages are observationally distinguishable,
the honest `⊥` has internal structure `Σᵢ 1ᵢ` — one unit per distinguishable failure-stage. The
forcing condition is a **two-clause discriminator**: Σ is forced iff (1) the stages route *distinct,
non-interchangeable remedies* AND (2) the consumer *cannot recover the sub-cause downstream* from the
boundary's output alone. If clause 2 fails (the consumer can recover the sub-cause), fusing is an
*allowed* lossy projection. The cardinality of `Σ` is **pipeline-relative** — "one unit per
distinguishable failure-stage," NOT "always three." `B + 1` is the `|Σ| = 1` single-read special case.
Layer 2 inherits Layer 1's forcing (there is no third value to sub-structure if the third value itself
is not forced).

**The allowed escape — downstream projection (C4).** Collapsing the boundary *type* is the bug.
Projecting `{true, false, ⊥} → {true, false}` *downstream*, at a consumer that has legitimately decided
how to treat indeterminate, is **fine** (e.g. ADR-033's frame-aware render, `Option::unwrap_or`, a CI
gate that treats "couldn't evaluate" as non-failing). A lossy projection *existing* says nothing about
the source's cardinality, the same way `Option::unwrap_or(false)` does not make `Option<bool>` into
`bool`. The boundary's honest type is the codomain of `eval_total`; what a consumer chooses to forget is
its own business. This law constrains the *boundary's type*, never the consumer's projection.

**Instantiations (one `1`, thirteen costumes — NOT thirteen findings, NOT thirteen ADRs).** Each cite
resolves to real substrate:
1. Categorical verdict — `CompositeVerdict` / `Indeterminate` (`antigen-attestation/src/evaluate.rs`).
2. Fingerprint leaf-algebra — `Match3 {True, False, Undefined}` (ADR-010 Amd 6).
3. Cross-crate reference resolution — out-of-frame (ADR-024).
4. Prescriptive work-need — `WorkVerdict::OutOfFrame` (ADR-033; `audit.rs`).
5. Coverage / reachability — `UnreachedCause {Barrier | SubThreshold | Cryptic}` (`audit.rs`) — the
   first built Layer-2 split.
6. Titer / scalar value layer — un-measurable (ADR-019 Amd 1).
7. Stored report — `stored | absent` ill-typed (ADR-034).
8. Observe-don't-declare — the discipline-face identity (ADR-029).
9. Similarity matching — match / no-match / not-sure.
10. Biological ignorance — ignorance ≠ anergy (the scan-coverage ignorance frontier).
11. Tier-honesty — the `partial` verdict / `min_tier` (deferred to v0.3 L3; the shape is present).
12. Temporal-freshness — `fresh_through` requires a current-fingerprint signer (ADR-019 NFA-21).
13. Version-parse — malformed version is a third value, not coerced-to-zero.

**The self-applying witness (the canonical example).** `eval_ratified_doc`
(`antigen-attestation/src/evaluate.rs`) still ships a live Layer-1 collapse: its `fail` closure hardcodes
`evaluated: true` on every path, so a doc whose version is *absent* or *malformed* (a read-failure, `⊥`)
is reported as definitively-below-floor (`⊥ → false`) rather than as un-evaluated. The codebase already
contains the correct pattern: the supply-chain leaf arm sets `evaluated: false` for the not-run case,
with a standing ATK assertion that "`passed: false, evaluated: true` implies the check ran." So the
buggy leaf violates antigen's *own* established `⊥`-at-leaf pattern. This is the law operating as a
detector on real shipping code, with an in-tree precedent for the fix — the strongest possible evidence
that `CardinalityCollapseAtTrustBoundary` is self-applying, not a hypothetical.

The *other* direction of this same leaf — a `u64`-overflowing declared `min_version` coercing to zero
and vacuously passing (`⊥ → true`) — **was a live collapse when this gem was first surfaced, and has
since been closed** (commit `7941dc6`, ATK-FT-3): `validate()` now rejects any `min_version` with a
non-`u64`-parseable component (`PredicateParseError::UnparseableMinVersion`), so an unobtainable floor
can never silently become `0`. That remedy is itself a worked instance of this law's exhaustiveness
partition: it pays the partiality **upstream at validate time** — cell (b), the in-memory-2-valued path
— so the eval-time leaf never sees the `⊥`. The law predicted the shape of its own fix. The remaining
`⊥ → false` direction is the leaf-sweep follow-on (tracked below), where the closure should set
`evaluated: false` on the doc-read-failure paths exactly as the supply-chain arm already does.

### Why this is forced, not chosen (Phase-8 void)

Reject the law and accept that a substrate-relative boundary may be two-valued. Then the un-evaluated
case — which is *common in the wild* (substrate is always missing something) and *rare in test fixtures*
(which tend to be complete) — is silently called pass or fail. The boundary passes review, ships, and
then lies in production: silently, because the lie is "I evaluated this" when it didn't, and an
un-observed case produces no failing test until someone constructs the partial-substrate fixture. So
antigen — the tool whose entire purpose is making the implicit explicit and failing loud not silent —
would ship the *silent-wrong-verdict it exists to catch* (project-level autoimmunity). The void's shape
is the proof: the only well-typed model is the three-valued one. This is the same void as ADR-034's
VOID-W6 (a stored report commits `ParallelStateTrackersDiverge`); the storage question and the
third-value question are one law seen from two angles.

### A notation note (a deconstructor catch)

The glyph `⊥` is overloaded in this document. **In this ADR, `⊥` is the lift unit of `B + 1`** — the
"could-not-evaluate" inhabitant. **In ADR-031 ("tier `⊥` category"), `⊥` denotes orthogonality** — two
independent, substrate-populated axes. These are structurally unrelated; the orthogonality `⊥` is *not*
a gem-instance and must not be cited as one. Same glyph, two meanings — itself exactly the kind of
implicit-mode obscurity antigen exists to surface.

### Scope

- **v0.3 floor (recognition, not new build):** ratify the law + both antigen names; record the
  instantiation catalog. The shipped boundaries (`CompositeVerdict`, `Match3`, `UnreachedCause`,
  `WorkVerdict::OutOfFrame`, the substrate-witness `AuditHint`) already *demonstrate* the law on
  antigen's own construction — adversarial's gate confirmed the implementation naturally factors into
  (eval-time reads → 3-valued) + (in-memory → 2-valued, paid upstream).
- **Tracked v0.3+ follow-ons (additive, NON-blocking):**
  - The `eval_ratified_doc` leaf fix — the remaining `⊥ → false` direction: set `evaluated: false` on
    the doc-side read-failure paths (doc-absent, no-parseable-frontmatter-version) so a read-failure is
    reported as un-evaluated, not as definitively-below-floor. (The sibling `⊥ → true` malformed-version
    overflow already closed at validate time in `7941dc6` / ATK-FT-3.) Owned by the freshness-bypass
    campsites. A *systematic leaf-sweep of `evaluate.rs`* (naturalist's prediction): every leaf asked
    "does it fold an un-anchored / un-parseable / empty-collection `⊥` into pass-or-fail?" — the
    gem-as-detector in operation.
  - `CoverageAuditReport::coverage_was_applicable() -> bool` (adversarial's type-discipline finding: a
    3-state domain behind the 2-valued `is_complete()` — an allowed C4 projection today, but the third
    state should be inspectable by library consumers).
  - The `WorkVerdict::OutOfFrame` sub-cause refinement (`OutOfFrameCause` sub-enum mirroring
    `UnreachedCause::remedy`, routing the remedy per cause) — the prescriptive Layer-2 unfold.
- **The audit detector (self-applying, v0.3+):** a structural check that flags a substrate-relative
  boundary whose verdict type has two inhabitants over a partial domain — antigen detecting its own
  Layer-1 violations.

### What this ADR does NOT do

- Does NOT forbid downstream two-valued projection (C4 is explicitly allowed; the law constrains the
  boundary type, never the consumer's choice to forget).
- Does NOT claim `Σ(stages)` is a theorem about partial functions in general — a bare composite forgets
  which stage failed; `Σ` is forced only under the two-clause discriminator (and is *met* in antigen
  because the remedy-routing commitment retains drop-provenance).
- Does NOT make the third value's *cardinality* a constant — `|Σ|` is pipeline-relative.
- Does NOT block the v0.3 floor on any follow-on — the leaf-sweep, the coverage method, and the
  OutOfFrame sub-cause are additive refinements, not ratification gates.
- Does NOT introduce a new runtime primitive — every instantiation already exists; this ADR *recognizes*
  the law they share (ADR-006 recognition-not-design, applied at the type-law scale).

---

## [ADR-036] The Scan/Audit Orchestration Decomposition: a Thin Out-of-Band Coordinator Above the Detector Sequence (the SCRAM Host)

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) — buildability-confirmed against the
real substrate by the pathmaker; **awaiting the notary** (Geological Society / Boat 4) for promotion to
Witnessed. This is a *claim* ("the decomposition is buildable as specified, behavior-preserving, and the
SCRAM seam is genuinely near-free"), not a self-witnessed verification. The build itself is the
Bushwhackers' **opening move** (Boat 3), before any other file touches `scan.rs`/`audit.rs`.

*("**SCRAM**" — borrowed from reactor engineering, the emergency shutdown that halts a runaway from outside
the reactor it stops — is used throughout as shorthand for the future cascade-governor's kill-switch: a
mechanism that can halt a runaway detection cascade from a layer the runaway cannot disable.)*

**Participants**: pathmaker (the buildability pressure-test against the real 8031-line `scan.rs` +
7853-line `audit.rs`; the two-orchestrator finding; the SCRAM-host design); value-finder (surfaced the
FEEDBACK-stage safety-rail need); expansionist (the control-loop grounding — orchestration *is* the loop,
SCRAM *is* the FEEDBACK out-of-band damper); dreamer (carried the captain's SCRAM lock onto the campsite);
captain (ruled the out-of-band requirement, 2026-06-02 — locking the *requirement*, explicitly not the
mechanism, with the honest caveat that the pathmaker confirms near-free or surfaces it as a finding).

**Related**:
- The LOOP-A regulator frame (forthcoming ADR — antigen-is-a-closed-loop-regulator). This decomposition
  is that frame's **first concrete artifact**: the orchestration layer is the loop's spine; the SCRAM
  kill-switch is the FEEDBACK stage's structural safety-rail, sitting where the runaway it governs cannot
  disable it. The decomposition does NOT depend on that frame being ratified — it stands as
  behavior-preserving infra on its own — but it is where the frame physically lands.
- ADR-005 (sub-clause F at every trust boundary) — the SCRAM host is a new coordination boundary; its
  validation check is specified in §Out-of-band invariant below.
- The dogfood antigen `AuditVerdictComputedButNotDelivered`
  (`audit-verdict-computed-but-not-delivered`, `stdlib/dogfood.rs`) — antigen's OWN immune memory of the
  exact failure-class this refactor must not reintroduce: a severed `audit_*` whose verdict never reaches
  the CLI render path. It is a build-gate witness for this ADR (see §The behavior-preservation contract).
- ADR-002 (compose, don't compete) — the decomposition reorganizes antigen's own internals; it composes
  no external tool, but it is the structural pre-condition for the modular sensor/family growth (each
  detector becomes an independently-addable module).

### Finding

`scan.rs` (8031 lines, 354 KB) and `audit.rs` (7853 lines, 371 KB) are two monoliths that every
parallel build front must touch. Three problems compound:

1. **File-collision risk under parallel build.** Two pathmakers adding two families cannot both edit a
   single 350 KB file without merge conflict. The voyage's parallel-by-scope model (non-touching scopes)
   is unenforceable while the detectors all live in two files.
2. **The orchestration is not a layer — it is smeared across `main.rs`.** Hands-on substrate check:
   `cargo-antigen/src/main.rs` calls each detector *individually* — `audit::audit(&report, root)`,
   `audit::audit_category(&report)`, `audit::audit_supply_chain(&report, root)`,
   `audit::audit_convergent_evidence(&report)`, `audit::audit_recurrent(&report)`,
   `audit::audit_mucosal(&report)`, `audit::audit_lineage_fidelity(&report)`,
   `audit::audit_coverage(&report)`, `audit::audit_prescriptive(&report, root)`,
   `audit::audit_deferred_defenses(&report, 30)` — *the CLI binary is the de-facto orchestrator.* There
   is no single place that owns "run the detector sequence." A future cascade-governor would have nowhere
   to live above the loop.
3. **There are TWO cascades, of TWO different shapes** (the pathmaker's finding, sharpened by the
   adversarial stress on substrate). The audit side (the `main.rs` detector fan-out) *is* a natural
   detector-sequence — an orchestrator-runs-a-list shape; SCRAM fits cleanly above it. But the scan side
   (`scan_workspace` at `scan.rs:3093`) is **not a detector loop** — it is a *pipeline of passes*: a
   `WalkDir` file-walk driving an inline `ScanVisitor` per file, then a lineage-safety pass
   (dedup + cycle/depth, already internally damped by `MAX_LINEAGE_DEPTH = 64` + a visited-set + iterative
   DFS), then `finalize_report` (fingerprint synthesis + propagation). There is no "sequence of small
   detectors" for an orchestrator to sit *above* on the scan side; the work is a walk + finalize passes.
   So "above the detector loop" is the right shape for audit and the *wrong* shape for scan — and, more
   decisively, the runaway the SCRAM is actually for (the future cascade-governor / alert-storm damper) is
   an **aggregate over the whole result population** that runs *after* scan+audit produce it, not a step
   inside either hot path. The correct home for SCRAM is therefore **above the whole scan → audit →
   (future aggregate) pipeline** — the command-orchestration layer — not inside the lib's scan/audit
   passes. The near-free property to bank *now* is consequently not "a loop with a kill-switch" but the
   **purity invariant** (each detector/pass is a pure-ish function of its input; no detector
   self-coordinates or controls the run), which is what makes an out-of-band governor *insertable later*.

The captain locked the *requirement* — "the orchestration layer MUST be a genuine out-of-band coordinator
capable of hosting a future kill-switch above the detector loop, never inside a detector it must be able
to stop" — and charged the pathmaker to confirm it is genuinely near-free in the real decomposition, or
surface it as a finding. **Verdict: the REQUIREMENT survives (sound control theory); the MECHANISM as
phrased — "above the detector loop" — is the degenerate case, and the substrate sharpens it to a different,
still-near-free shape.** The near-free thing to bank now is **detector purity**, not a literal
loop-with-a-kill-switch: if every detector/pass is a pure function of its input (already nearly true
today), a future cascade-governor can be inserted as a *separate pipeline stage* at the
command-orchestration layer that consumes the emitted typed-event population and can short-circuit the run
from *outside* any detector. Locking "a loop to sit above" would send a Bushwhacker to thread a kill-switch
into `scan_workspace`'s file-walk (wrong layer; scan has no detector loop) or to over-hook the nine audit
fns (wasted) — so this ADR locks the *purity invariant + the command-orchestration host*, which is what
actually makes SCRAM insertable later, and confirms *that* is near-free (the detectors are already ~pure).

**Honest cost calibration (adversarial's pre-commit refinement — surface it, don't bury it): near-free on
the AUDIT side, MODEST-not-zero on the SCAN side.** The audit side is already a sequence of independent
one-shot `audit_*` fns — wrapping it in a thin coordinator is near-free. The scan side is a *thin wrapper*
around the `WalkDir` pass, NOT a detector-loop rewrite and NOT a mid-walk interrupt of the visitor — the
honest cost there is the thin-wrapper + the one-line per-file `should_continue()` check, which is modest
(real work, not a redesign), not zero. The *don't-foreclose* lock (the stage-sequencing invariant below)
is the genuinely-near-free thing to bank now; the scan-side wrapping is cheap but not free, and the ADR
says so rather than overclaiming.

### Decision

Decompose `scan.rs` and `audit.rs` into a **sequence of small, single-purpose detector modules** run by
**two thin orchestration modules** (`scan::orchestrate`, `audit::orchestrate`) that are themselves
coordinated by **one pipeline coordinator** capable of hosting a future SCRAM kill-switch above the union
of both cascades.

**The module shape (behavior-preserving extraction):**

- `antigen/src/audit/` becomes a module directory. Each existing `pub fn audit_*` + its report struct(s)
  moves to its own file: `audit/supply_chain.rs`, `audit/convergent.rs`, `audit/recurrent.rs`,
  `audit/mucosal.rs`, `audit/category.rs`, `audit/lineage_fidelity.rs`, `audit/coverage.rs`,
  `audit/prescriptive.rs`, `audit/deferred.rs`, `audit/immunity.rs` (the inline immunity-verdict loop +
  `compute_presentation_verdicts`, lifted out of the current monolithic `audit()`). The shared verdict
  vocabulary (`WitnessStatus`, `WitnessTier`, `AuditHint`, `ImmuneVerdict`, `WorkVerdict`, `FrameState`,
  `PresentationVerdict`, `AuditReport`, …) moves to `audit/types.rs`. The detectors are siblings; none
  calls another.
- `antigen/src/scan/` becomes a module directory: `scan/walk.rs` (the `WalkDir` + `ScanVisitor` file
  collection), `scan/synthesis.rs` (fingerprint synthesis), `scan/lineage.rs` (dedup + cycle/depth +
  propagation), `scan/multi_crate.rs` (member enumeration + merge), `scan/types.rs` (`ScanReport`,
  `AntigenDeclaration`, `Presentation`, … and every other `pub` type), and `scan/finalize.rs`
  (`finalize_report`, the shared post-collection pass already extracted today).
- `antigen/src/audit/orchestrate.rs` and `antigen/src/scan/orchestrate.rs` are **thin**: each owns the
  *order* in which its detectors run and nothing else (no detection logic). `audit::orchestrate::run`
  runs the audit detector sequence; `scan::orchestrate::run` runs the scan pass sequence.
- A single **pipeline coordinator** sits above both `orchestrate::run`s and is the designated SCRAM host
  — the layer above the whole scan → audit → (future) aggregate pipeline (see §The out-of-band invariant).
  The current `main.rs` detector-by-detector fan-out is *replaced* by a call into this coordinator. Its
  locus (library-side `antigen/src/pipeline.rs` vs binary-side in `cargo-antigen`) is the one open
  mechanism choice — see §Open mechanism choice; either satisfies the out-of-band invariant.

**Public-path preservation (non-negotiable).** Every current `pub` path stays a `pub` path. `pub use`
re-exports at the `audit`/`scan` module roots preserve `antigen::audit::audit_supply_chain`,
`antigen::scan::scan_workspace`, `antigen::scan::ScanReport`, etc., byte-for-byte. Moving an item into a
submodule and re-exporting it is API-invisible. The decomposition changes *where code lives*, never *what
the crate exposes*. (`scan_workspace`, `scan_workspace_multi_crate`, `audit`, all nine `audit_*`, all
report/verdict types, `enumerate_dep_crate_roots`, `enumerate_workspace_member_roots`,
`CLONAL_ITERATIONS_DEFAULT_FLOOR`, `IGG_HISTORICAL_SPAN_DEFAULT_FLOOR`, `evidence_kind_from_status` — the
full surface enumerated from the real files — is preserved.)

### The out-of-band invariant (the SCRAM seam, sub-clause-F-checked)

The captain's requirement — *the governor must sit where the runaway it governs cannot disable it* — is
sound and survives. Stripped of the kill-switch/SCRAM imagery, the irreducible property (aristotle's
first-principles grounding) is the **single-conductor invariant**: *the authority to stop the run lives in
exactly ONE place, and that place is not any detector/pass the run might run away in.* A runaway is a unit
producing more work/signal than the loop can damp; if that unit also held stop-authority, the runaway would
disable its own brake. Its real near-free form on this substrate is **detector purity + a named host above
the whole pipeline**, not "a kill-switch threaded into a detector loop." The lock, in three parts:

1. **The PURITY invariant is the thing banked now (and it is already ~true) — locked AS the proxy for the
   single-conductor invariant.** Every detector/pass is a pure-ish function of its input report → output
   report/verdict: it does not self-coordinate, does not control whether the run continues, does not poll
   or own any interruption mechanism. Purity is *strictly stronger* than single-conductor (a detector
   could log a line and still hold zero stop-authority) — but the extra strength is **free** here
   (`scan.rs:2822` literally comments "scan_workspace function is a pure directory scanner"; each `audit_*`
   takes a `&ScanReport` and returns its own report) and, crucially, purity is **checkable** where "holds
   no stop-authority" is a fuzzy negative about control flow: a pure fn provably cannot influence control
   flow except through its return value, so a population of pure detectors provably leaves all stop-authority
   with whoever sequences them. **Lock purity, and state it AS the rationale** — "detectors are pure fns of
   input SO THAT stop-authority stays with the orchestrator" — because locking purity without the *why*
   invites a future reader to relax it ("a little logging is fine") and silently break the single-conductor
   invariant. **Do not let any extracted detector acquire a back-reference to the run/coordinator.**
   *(Purity also stands on its own hygiene merit, independent of any future governor: a pure detector is
   independently testable, trivially parallelizable, free of cross-detector order-dependence, and is exactly
   the property that makes this a safe behavior-preserving decomposition — the monolith's detector logic is
   already nearly pure, and keeping it pure is what guarantees no extraction introduces a hidden shared-state
   coupling. The future-governor enablement is a bonus on top of a refactor-correctness property worth
   locking regardless.)*
   - **FORCED stage-sequencing invariant (adversarial's pre-commit finding, has teeth): NO stage triggers
     the next — the coordinator owns sequencing.** Purity of a *detector* is necessary but not sufficient;
     the extra requirement is that **no stage (scan-pass, audit-detector) calls the next stage directly** —
     every stage *returns to the coordinator*, which decides what runs next. This is the single-conductor
     invariant applied to STAGE SEQUENCING, not just detector purity. It is OPEN today (scan → audit go
     through `main.rs`, not via a direct scan→audit call), and the *risk* is the decomposition accidentally
     welding two stages with a convenience direct-call. Why it has teeth: the FUTURE afferent runtime-sensor
     (charter) IS a sensor-in-a-loop (drains prod → re-triggers maturation → re-fires detection), so
     sensor-layer SCRAM stops being optional *for it* — and the door stays open for it **iff** every stage
     returns to the coordinator. **Lock: "no stage triggers the next; the coordinator owns sequencing +
     SCRAM + the future sensor-drain."**
2. **The SCRAM host is the command-orchestration layer, above scan → audit → (future) aggregate — NOT
   inside the lib's scan/audit passes.** The cascade-governor SCRAM is meant to damp an *aggregate over
   the result population* (alert-storm / sepsis-anaphylaxis), which by nature runs *after* scan and audit
   produce that population. So its home is the top-level pipeline coordinator (today smeared across
   `cargo-antigen/src/main.rs`, ~5488 LOC; this ADR gives it a home — see §Decision) — above the union of
   scan, audit, and the future aggregate stage. A future governor is inserted there as a *separate
   pipeline stage* that (a) consumes the emitted typed-event population and (b) can short-circuit the run
   from outside any detector. (The one place a runaway *could* live inside the hot path today —
   `#[descended_from]` propagation — is already correctly damped *in-band* by `MAX_LINEAGE_DEPTH = 64` + a
   visited-set + iterative DFS + cycle detection; an intrinsic depth cap is correct damping, distinct from
   a kill-switch, and stays where it is.)
3. **Sub-clause F (ADR-005) validation check for this boundary:** the out-of-band property holds iff
   **no detector module imports or references the coordinator / interruption mechanism** — dependency
   flows one way (coordinator → detectors; never the reverse). Enforce it as a build-time structural test
   now (and a candidate `cargo antigen audit` self-check later): a detector that reaches up to the run has
   made the governor a participant in the cascade it must be able to stop. This is the trust-boundary
   check the SCRAM seam introduces; it is exactly the *purity invariant* (1) viewed as a module-boundary
   direction.

**This ADR locks the purity invariant + the host location + the validation check. It does NOT build the
governor.** The cascade-governor / cytokine-storm damper is charter (a future expedition). What beta.2
ships is the *shape that makes it buildable later for ~free*: pure detectors with no back-reference to the
run, and a named command-orchestration layer that can host the future aggregate-governor stage above the
whole pipeline. Retrofitting purity after the detectors have been allowed to self-coordinate inside a
monolith would be ruinous; preserving it while we are already moving every detector into its own file is
the marginal cost of *how the move is done* (keep them pure), which is ~zero.

### The two seams converge on one locus (a structural bonus worth banking)

SEAM 2's SCRAM host (this ADR) and SEAM 1's emit-merge point (ADR-039 — "one typed Finding schema, both
stages emit, merged at the audit/command stage") **land at the same place: the population-complete
coordination layer.** This is not a coincidence. A governor damps an *aggregate over the population*
(sepsis/anaphylaxis = a whole-scan-result aggregate), and an aggregate can only be computed where the
population is complete; symmetrically, the emit-merge needs the last stage that sees both the scan-time
markers and the audit-time verdicts. Both require the population-complete vantage. So the locked
architecture is one shape serving both seams: **pure detectors/passes → emit typed Findings into one
population → a population-complete coordination layer that (a) merges the schema (SEAM 1) and (b) is the
sole holder of stop-authority where a future SCRAM lives (SEAM 2).** It is near-free because the command
layer (`cargo-antigen/main.rs`) already *is* that population-complete vantage — the decomposition gives it
a name and the purity discipline that keeps it the single conductor.

**Schema ownership (the cross-ADR boundary — prevents a two-schemas `ParallelStateTrackersDiverge`).**
**This ADR (036) does NOT define the `Finding`/event schema** — it owns only the *orchestration*: the thin
coordinator, the purity invariant, and the *merge-locus* (where the unified population is assembled = the
population-complete coordination layer). The schema itself — the `Finding` record and its full field-list
(`site`, `class-or-marker`, `tier`, `magnitude`, the mandatory `class_provenance`, `presentation`,
`trigger`, `cluster-key`, `timestamp`, `origin-stage`) — is **owned by ADR-039 §C**, which this ADR cites.
There is exactly ONE schema definition (ADR-039) and ONE merge-locus (this ADR); the decomposition builds the
coordinator that *lands ADR-039's schema* into one population, never a second schema of its own.

### The behavior-preservation contract

This is a **behavior-preserving refactor**: no verdict, no output, no JSON shape, no exit code changes.
The guards:

- **The full test suite (~1187 tests incl. the ATK adversarial suite) is the primary witness.** Green
  before, green after, with zero test edits that change an assertion (test *moves* to follow code are
  fine; assertion changes are a red flag the refactor altered behavior).
- **The gates stay green continuously**: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` (pedantic + nursery), `cargo test --workspace --all-targets`,
  `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`, and `cargo +1.95.0 check` (the MSRV gate
  — note: MSRV is **1.95** at HEAD `8ebea9a`, not the 1.85 some older docs cite).
- **The dogfood antigen `AuditVerdictComputedButNotDelivered` is a build-gate witness.** Its fix-shape —
  "every `pub fn audit_*` is paired with a `print_*`/`render_*` call site exercised in the CLI
  integration suite" — must hold after the decomposition. When the detector fan-out moves from `main.rs`
  into the coordinator, every `audit_* → print_*` pairing moves with it; the integration suite that
  exercises the render path is the check that none was severed. The refactor is, fittingly, governed by
  antigen's own immune memory of this exact class.
- **Commit in coherent chunks**, one detector-extraction per commit where practical, gates green at each.

### Why this is recognition, not design (ADR-006)

The decomposition does not invent a structure; it *names and separates* one that already exists implicitly.
The nine `audit_*` detectors are *already* independent functions with their own report types — the module
boundaries are recognition of a fan-out that the code already has. The orchestration *already* runs (in
`main.rs`); the decomposition gives it a home and a name. The out-of-band property is *already* latent in
"the coordinator runs the sequence"; the lock makes it explicit and checkable. This is implicit-to-explicit
elevation (ADR-004) applied to antigen's own internals: the orchestration was implicit (smeared across the
binary); the decomposition makes it explicit (a named layer with a validation check).

### What this ADR does NOT do

- Does NOT change any verdict, output, JSON shape, or public path — behavior-preserving, API-invisible.
- Does NOT build the cascade-governor or any SCRAM logic — it locks the *host shape* so the governor is
  buildable later for ~free; the governor itself is charter.
- Does NOT pre-decide whether the pipeline coordinator lives in `antigen` (`pipeline.rs`) or in
  `cargo-antigen` (the binary). The pathmaker's lean is `antigen`-side (so library consumers, not only
  the CLI, get the orchestrated entry point and the future SCRAM) — but the minimal behavior-preserving
  move keeps the coordinator CLI-side; this is a build-time choice for the Bushwhackers, recorded as the
  one open mechanism question, not a blocker (see §Open mechanism choice).
- Does NOT mandate a specific interruption primitive (an `AtomicBool` continue-flag, a `Result`-returning
  `should_continue()` the coordinator checks, a callback) — only that it lives in the coordinator and no
  detector depends on it. The Bushwhackers pick the primitive; the sub-clause-F check (no detector
  imports it) is the invariant, not the type.

### Open mechanism choice (for the Bushwhackers, non-blocking)

Where does the pipeline coordinator live — `antigen/src/pipeline.rs` (library-side, so the orchestrated +
SCRAM-capable entry point is available to all consumers) or `cargo-antigen` (binary-side, the minimal
behavior-preserving move)? The pathmaker recommends **library-side**: it makes the regulator's loop a
first-class part of the library (consistent with the LOOP-A frame), and a runtime-sensor or external
platform consumer (charter) would want the orchestrated entry point, not a re-implementation of `main.rs`'s
fan-out. But binary-side is a valid minimal move that satisfies the out-of-band invariant for the CLI. This
is the single open mechanism question; it does not gate the lock. If the Bushwhackers find library-side is
*not* near-free (e.g. a circular-dependency surprise), that is a finding to surface to the captain, not a
requirement to drop.

### Build-time supersede-note (Bushwhackers / pathmaker, 2026-06-03) — the scan side gains a `parse.rs` leaf

**What this supersedes (append-only; the original §Decision module-shape is preserved above, this overrules
only the implicit "the scan parsing engine lives in `walk.rs`").** The §Decision §The module shape lists the
scan-side files as `scan/walk.rs` (the `WalkDir` + `ScanVisitor` file collection), `scan/synthesis.rs`,
`scan/lineage.rs`, `scan/multi_crate.rs`, `scan/types.rs`, `scan/finalize.rs`, `scan/orchestrate.rs`. Building
it revealed that the `ScanVisitor` (the ~1200-line `syn::visit::Visit` parse engine) + the `render_path` /
`render_type` / `attr_is` / `extract_requires_predicate_from_attrs` helpers are a **shared parse layer that
`scan_workspace` (walk), `synthesis_pass` (synthesis), AND `finalize_report` (finalize) all consume.** Putting
that engine in `walk.rs` as the file-list implied creates a module **cycle**: `walk → finalize → synthesis →
(walk's `ScanVisitor`)`.

**The decision (superseded → with):** decision X = "the parsing engine lives in `walk.rs`" is **superseded by**
decision Y = "the parsing engine is its own dependency-free **leaf** module `scan/parse.rs`", **because Z** =
`ScanVisitor` calls **no pass fn** (verified — it is a pure AST→declarations walker), so a `parse` leaf breaks
the cycle and the scan pipeline layers acyclically: **`parse ← {synthesis, finalize, walk} ← multi_crate`**.
The scan-time attribute arg-parsers (`Scan*Args` + their `syn::parse::Parse` impls) **stay in the `scan`
module root** (their white-box field-reading unit tests live there, interleaved with proptests across ~3300
test lines — not a movable contiguous block); `parse.rs` imports them from `super` (a child module reads its
parent's private types + fields, so no field is widened to `pub`). `ScanVisitor` exposes a `pub const fn new`
constructor (rather than `pub` fields) so its internal bookkeeping stays encapsulated across the new module
boundary. This is **behavior-preserving** (no verdict/output change; the 1187-test suite is the witness) and
**API-invisible** (`parse` is a private module; only `pub(crate)` re-exports give the passes + the
`#[cfg(test)]` test module reach — the public `scan::` surface is unchanged). The original intent (smaller
single-purpose files, types-first, orchestration named + out-of-band-capable) is **honored, not weakened** —
the parse leaf is the cleaner expression of "each detector/pass is a pure fn of its input."

**Honest note on the scan side's nature (a finding, not a defect):** unlike the audit side (nine genuinely
independent `audit_*` detectors that fan out cleanly), the scan side is **one tightly-coupled pipeline** — every
pass depends on the parse layer and the finalize/synthesis core, and its tests are white-box-coupled to the
internals. So the scan decomposition is necessarily a *layered* split (leaf → passes → coordinator) threaded
with `pub(crate)` shared-helper re-exports, not the audit side's flat fan-out. The purity invariant (§The
out-of-band invariant) still holds: no pass holds stop-authority; `parse` is the purest (a pure AST walker).

---

## [ADR-037] Antigen Is a Closed-Loop Regulator: Its Own Machinery Has Six Failure-Points (the Control-Loop Master-Frame)

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) for the **regulator self-model** (use-1
below) — **awaiting the notary** for promotion to Witnessed. The frame's *self-model* use is
falsification-gate-CLEARED (adversarial), biology-coherence-RULED (naturalist, high confidence), and
first-principles-grounded (aristotle). Its *second* use — whether the control-loop FUNCTION is also a
citing axis of the stdlib family taxonomy (ADR-038) — was an open seam, now **RESOLVED via the
fidelity-vs-genus crosscut test** (aristotle + adversarial, converged independently): the crosscut found a
COMPARE-fidelity cell the original two-genera missed, resolved by adding a THIRD genus
(comparator-divergence), so the genus axis becomes complete-by-construction (info/comparator/effect =
SENSE/COMPARE/ACT, one per active stage) and ADR-038 does **not** cite a separate function axis — it carries
the three genera as a single sub-axis under ADR-028's category. This ADR locks use-1; the resolution detail
lives in §The two uses / §Open seam and ADR-038.

**Participants**: expansionist (the Phase-6 convergence-check — six independent cross-domain mappings tiled
one control loop, one-per-stage, cleanly under fix-sorting; the requisite-variety sizing law; the
setpoint prediction — the one confirmed cross-validated prediction that is the frame's load-bearing
anti-vacuity evidence);
adversarial (the falsification gate — ran the family-sort by fix-shape, found the regulator-vs-disturbance
boundary, located the two mislabeled-`families/*` re-file findings, cleared the frame for the regulator and
correctly-excluded it from the catalog); naturalist (the biology-coherence ruling — the regulatory
*machinery* is not the *catalog* of antigens; immunology's founding architecture-fact; the fail-direction
invariant is biologically literal — thymic selection defaults to apoptosis); aristotle (the first-principles
deconstruction + honest self-correction — one frame, two uses, not two loops); value-finder (legibility as
a spine need across the loop's three damping scales); pathmaker (the lock + the fidelity-vs-genus crosscut
framing of the open seam).

**Related**:
- ADR-007 (anti-YAGNI / structurally-guaranteed need). Requisite variety (Ashby 1956, V(C) ≥ V(D)) is the
  *sizing law* of this loop and the formal grounding of ADR-007: a regulator with less variety than its
  disturbances provably cannot regulate, so building variety **to match the real disturbance space** is a
  theorem, not prudence (**requisite, not maximal** — variety beyond the *recurring* disturbance space is
  the charter/YAGNI region; the theorem forces the **floor**, not the ceiling). This *strengthens*
  "build it all": the frames ratify the variety the recurring disturbances need; chartered dreams are
  variety for not-yet-recurring disturbances (built when they begin to recur), not speculative over-build.
- ADR-036 (the orchestration decomposition). The decomposition IS this loop's spine made concrete:
  orchestration = the loop's coordination; the SCRAM out-of-band governor = the FEEDBACK stage's
  safety-rail. ADR-036 is this frame's first physical artifact.
- ADR-028 (substrate-alignment vs functional-correctness). This frame describes the **substrate-alignment**
  object — antigen-the-regulator's own anatomy; the stdlib families (functional-correctness, the
  disturbances) are ADR-028 + ADR-038's object. The boundary between them is the regulator/catalog line.
- ADR-029 / ADR-035 (observe-don't-declare; the three-valued type law). The **fail-direction invariant**
  (every guard fails *safe*, toward not-Defended) is this frame's COMPARE-stage discipline; it collapses
  antigen's own silent-wrong-verdict bugs into one rule and lives in ADR-029/035 territory.
- The forthcoming `#[autoimmune]` primitive — the COMPARE-stage **reference**-failure (the setpoint itself
  wrong: Goodhart / recognition mis-targets self), the gap this frame *predicted* and biology had already
  partitioned into separate machinery.

### Finding

Six independent cross-domain mappings, each derived on its own (absence-as-signal, gate-collapse,
gradient-routing, graduation-as-evidence-updating, cascade-becomes-the-problem, requisite-variety),
**tiled a single control loop — one failure-point per stage (plus the loop's sizing law), with no member
smearing across two stages when sorted by FIX-shape.** (The no-overlap property is *conditional on
fix-not-symptom sorting*; the tiling is an *organizing* completeness — a checklist heuristic — not a proven
totality. See the calibrated falsification result below.):

| Control-loop stage | What it does | antigen's failure-point at that stage |
|---|---|---|
| **SENSE** | observe the substrate | **absence-as-signal** — can't sense what isn't there without a frame |
| **COMPARE** | check observation vs reference | **gate-collapse** — the comparator stuck vacuously at "OK" (the ⊥/forgery bugs) |
| **(reference / setpoint)** | what's expected | *predicted gap*: **setpoint-wrong** = `#[autoimmune]` (Goodhart; biology's separate machinery) |
| **ROUTE / DECIDE** | send the correction to a responder | **gradient-routing** — route by live gradient, not a stale table |
| **ACT** | grade the response | **graduation / the confidence dial** — how confident before flagging loud (a Bayesian posterior) |
| **FEEDBACK** | damp so the loop stays stable | **cascade-becomes-the-problem** — positive feedback, no damping → SCRAM out-of-band |
| **(sizing)** | enough variety to regulate at all | **requisite-variety** — V(C) ≥ V(D) or it can't regulate (Ashby) |

When independent searches tile a known structure cleanly (no member smearing across stages, under
fix-sorting), the tiling is a useful **organizing** result — but the load-bearing anti-vacuity evidence is
not the tiling itself (a frame fitted to N findings can manufacture a clean tiling); it is the **one
confirmed cross-validated prediction.** The frame is not a pure post-hoc arrangement: the setpoint stage was
*predicted* (control theory says a loop has a reference; none of the six findings was "the reference is
wrong"), and the prediction landed on a *pre-existing* biological partition — autoimmune (setpoint
mis-targets self) is distinct machinery from the per-effector self-toxicity shadow, exactly as the frame
requires. A prediction landing on a distinction biology had already made is the anti-vacuity evidence —
**but it is N=1** (one gap-then-fill, cross-validated), which clears "not pure retrofit" without
establishing "predictive" as a strong standing track record. So: an organizing frame that has earned **one**
predictive credit, not a proven-complete taxonomy.

The falsification gate (adversarial, a different authority than the expansionist who drew the loop —
satisfying `IndependentRecheckAuthority`) attacked the frame and could not find a regulator-mechanism that
fails to tile. **The frame survives as a description of antigen's own regulator.** What the gate *did* find
is the boundary in the next section.

**The gate's three-verdict discriminator (the recurrent-tangle has teeth here).** Because the loop is a
recurrent *tangle*, not a clean pipeline (see §What this ADR does NOT do), a candidate family that "won't
sort cleanly into ONE stage" is ambiguous between three cases, and the gate must distinguish them by
**fix-shape** or it both false-rejects and false-accepts:
- **(a) real break** — the family's fix matches *no* stage's fix-shape at all → the frame is incomplete →
  *extend or kill* the taxonomy. (This is what the gate hunts.)
- **(b) seam-family** — the fix is the *composition* of two stages' fixes (e.g. a ROUTE/ACT boundary
  failure fixed by repairing *both* the routing decision *and* the effector) → a legitimate **edge** of
  the tangle, *not* a break and *not* a new node. The tangle *predicts* these; finding one means you
  discovered an edge, and the frame *survives*. Rejecting the frame because a seam exists is a
  false-positive break.
- **(c) clean node** — the fix is exactly one stage's fix-shape.
Fix-shape is the discriminator throughout (the same rule the family-boundary work uses: distinct remedies
force distinct units; a *composed* remedy is an edge, not a unit). Worked instance: `comparator-divergence`
(ADR-038) was a case-(a)-resolved-to-(c) — it matched no stage of the *original* two genera (info/effect),
forcing the taxonomy to extend, and extended to its *own* clean node (COMPARE), because "tighten the guard"
is neither the info-fix (recover/retain the value) nor the effect-fix (reorder/recount/recompartment) nor
their composition. It survived the seam-vs-node discriminator, not merely the sort-vs-not one — the stronger
test. The notary check this licenses: confirm no family on the board is a *seam mislabeled as a node* (or as
a break).

### The decision: one frame, scoped to antigen-the-regulator

**Antigen is a closed-loop regulator. Its own machinery is a SENSE → COMPARE(-to-reference) → ROUTE → ACT →
FEEDBACK loop, sized by requisite variety: 5 STAGE-FAILURES + 1 SIZING-LAW.** (The setpoint/reference is the
COMPARE stage's *reference*-failure — `#[autoimmune]` — not a separate stage; the table brackets it as
"(reference/setpoint)" for that reason. So the honest count is five stage-failure-points — SENSE, COMPARE,
ROUTE, ACT, FEEDBACK — plus the requisite-variety sizing law, *not* seven independent points.) This is
antigen's **self-model**: the architecture from which the
decomposition (ADR-036), the SCRAM governor, the confidence dial (ADR-039), the fail-direction invariant
(COMPARE discipline), `#[autoimmune]` (the reference-failure), and requisite-variety-as-ADR-007-grounding
all descend as one coherent subsystem — not six independent macros.

**The frame is the taxonomy of the REGULATOR, not of the DISTURBANCES.** A failure-class antigen detects in
user code (async-soundness, semver-contract, deserialization, drop-panic, …) is the **disturbance** the
regulator regulates — the *input* to the loop, not a *stage* of it. Asking "which loop-stage senses this
disturbance?" answers "SENSE" for every disturbance (vacuous: you never classify pathogens by which stage of
your own immune system perceives them). So the control-loop frame does **not** become the stdlib taxonomy;
the stdlib stays organized by remedy-shape under ADR-028 + the super-family parents (ADR-038).

**This split is recognition, not design — the line was already drawn by requisite variety (ADR-007).** The
control-theoretic name for it: the loop is the **controller C** (antigen's regulator, with its six
failure-points); the stdlib families are the **disturbances D** that C's SENSE stage detects. V(C) ≥ V(D)
(Ashby) presupposes C and D as *distinct sets* — and **a stage of C is categorically not a member of D.**
That is the formal reason control-loop-stage cannot sort stdlib families: you would be asking which element
of C a member of D *is*, a category error, which is exactly why every family answers "SENSE" vacuously. The
C/D boundary is the same line ADR-007's grounding already required; this ADR names it, it does not invent it.
Biology states the same partition: the immune system's *regulatory machinery* (the controller C) is not the
*catalog of antigen-specificities* (the disturbance-repertoire D) — coupled (the catalog drives maturation
via the afferent arm) but never merged. Control theory (C/D) is the principle; biology (machinery/catalog)
is its instance; both draw one line.

### The boundary-discriminator (locks the two ADRs apart — adversarial's located finding)

A `families/*` campsite belongs to the **stdlib (ADR-038, a disturbance)** iff it catalogs a user-code
failure-class antigen SENSES. It belongs to **this ADR (LOOP-A, the regulator)** iff it describes antigen's
own sense/compare/route/act/feedback machinery. **The discriminator: is the fix a change to USER code
(→ stdlib disturbance-family) or a change to ANTIGEN's own regulator (→ this frame's mechanism)?**

Two seeded `families/*` campsites are mislabeled regulator-machinery and must be re-filed under this frame,
keeping only their genuine disturbance-half in the stdlib:
- `families/gradient-routing-chemokine-recruitment` — its ROUTE **primitive** ("antigen should have a
  gradient-routing primitive") is a regulator-mechanism (this ADR); only its disturbance-half
  (`RoutingTableStale` / stale CODEOWNERS) is a stdlib family.
- `families/cascade-becomes-the-problem-sepsis-anaphylaxis` — its **governor** (the SCRAM organ) is the
  FEEDBACK regulator-mechanism (this ADR + ADR-036); only its disturbance-half (a detectable
  workspace-aggregate alert-storm class) is a stdlib family.
- `families/setpoint-corruption-goodhart-autoimmune` is the canonical **BOTH** case and correctly already
  lives at the seam: the disturbance = `FingerprintGamedNotDefended` (user/agent behavior); the
  regulator-fix = `#[autoimmune]` as the COMPARE-reference recheck.

### The two uses (the seam is now RESOLVED)

The frame's two-ness — mis-stated earlier as "two loops" — is **one frame, two USES**:
1. **Use-1 (LOCKED here): the regulator self-model.** Antigen's own loop and its seven failure-points. This
   is what ADR-036 / the dial / the governor / fail-direction / `#[autoimmune]` descend from. Gate-cleared,
   biology-ruled, grounded. **Locked.**
2. **Use-2 (RESOLVED — see next section): the FUNCTION axis of the detected-family grid.** Whether a stdlib
   family's failure-genus is usefully classified by *which loop-fidelity it violates*.

### The crosscut test result (use-2 resolved — aristotle + adversarial converged)

The deciding test (does the loop-function axis ever sort two families into a different cell than the genus
axis?) was run *independently* by aristotle and adversarial, and they **converged**:
- **Rows that are info- or effect-divergence are 1:1 with the function axis** — information-divergence ↔
  SENSE/MEMORY-fidelity, effect-divergence ↔ ACT-fidelity. On these the function axis is genus-relabeled.
- **Two families cross-cut: `crypto-non-constant-time` (== on a secret) and `gate-collapse-in-user-code`
  (deser-flatten-defeats-deny-unknown-fields, a vacuously-true guard).** These are *neither* genus: no
  value is lost/stale (not information-divergence) and the order/count/context of effects is right (not
  effect-divergence) — the fault is that the user's *comparator/guard* computed the wrong verdict, and the
  fix is "tighten the guard," not "recover info" (SENSE) or "change the effect" (ACT). They land in a
  **COMPARE-fidelity** cell the two-genera plane had no home for.

**Resolution: the two genera become THREE.** The clean realization (adversarial's option X, which aristotle's
scoped-Position-B reduces to) is **not** a second orthogonal axis on every family — the cross-cut is at one
cell only, so a whole second axis is over-machinery. Instead, **complete the genus axis** with its missing
value: **information-divergence (SENSE/MEMORY) · comparator-divergence (COMPARE) · effect-divergence (ACT)**
— one divergence-genus per *active* loop-stage. This keeps the genus the single sort key, makes it
complete-by-construction against the loop's three active stages, and gives `crypto-non-constant-time` +
`gate-collapse-in-user-code` a real home. The function axis *earned its citation* not by being a separate
coordinate but by **finding the missing genus** (recognition, not relabel). A clean prediction falls out:
ROUTE and FEEDBACK have *no* user-code disturbance-genus (they are antigen-response-only stages), so the
user-code genus axis populates exactly the three stages where a program has fidelity to lose
(sense → check → act) and is empty at the two response-only stages — an asymmetry that is evidence the axis
tracks something real. **ADR-038 carries the three-genus taxonomy; this resolves use-2.** (One residual:
adversarial flagged the cross-cut rests on N=2 same-cell families — `comparator-divergence` is named on two
witnesses; it is the weakest-supported of the three genera and ADR-038 should hold it at the suspected
confidence-tier until more witnesses land. The captain's concurrence + adversarial's witness-attack are the
final seals; the structural resolution is converged.)

### What this ADR does NOT do

- Does NOT make the control loop the stdlib taxonomy — the stdlib is disturbances (ADR-028 + ADR-038), not
  stages of antigen's regulator.
- Does NOT build the cascade-governor, the gradient-router, or `#[autoimmune]` — it names them as this
  loop's stages so they are designed as one subsystem; the builds are sequenced elsewhere (some charter).
- Does NOT *re-open* use-2 (the function-axis-of-the-catalog): the crosscut test is RESOLVED here. ADR-038
  cites the loop-function axis **defined as FIDELITY-VIOLATED** (info → SENSE/MEMORY-fidelity, comparator →
  COMPARE, effect → ACT) — which *discriminates* (whereas "which stage *senses* it" is the vacuous framing).
  Because the three genera ARE the three fidelities, citing-the-function-axis-as-fidelity and
  having-the-genus-be-fidelity-defined are the **same single sort-axis** — not a parallel field. What ADR-038
  still owns is *building* that taxonomy + sealing the N=2 `comparator-divergence` genus with more witnesses.
- Does NOT claim the loop is a clean pipeline — it is a recurrent tangle (cross-talk between stages is where
  the next failure-class layer lives; "closed but not terminal"). The table is block-diagram-true, not a
  denial of the coupling.
- Does NOT claim the stage-set is COMPLETE/closed — completeness is a **checklist heuristic, not a proven
  totality.** A real control loop carries more structure than this frame names (observability,
  controllability, delay/latency, stability margins); any of those *may add a stage* on a later finding. The
  frame has earned **one** cross-validated prediction (setpoint → autoimmune, N=1) — enough to clear "pure
  retrofit," not enough to assert a closed taxonomy. An open-edge / may-be-incomplete frame is the honest
  status; a future stage discovery EXTENDS it (and is exactly what the gate's three-verdict discriminator —
  real-break vs seam vs node — is built to classify when it arrives).

---

## [ADR-038] The Stdlib Taxonomy Grid: Three Divergence-Genera (One per Active Loop-Stage), Super-Family Parents, and Remedy-Shape as the Primary Sort

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) — **awaiting the notary** for promotion
to Witnessed. The crosscut test that gated this ADR has cleared (aristotle + adversarial converged
independently — see ADR-037 §The crosscut test result). The **`comparator-divergence` genus rests on N=2
witnesses** and is held at the suspected confidence-tier (ADR-039) until more land; the captain's concurrence
+ adversarial's witness-attack are the final seals on that one genus. The rest is converged.

**Participants**: researcher (the two-genera capstone — information-divergence vs effect-divergence,
genus-per-stage; the three super-family unifier-parents; the demand-counts); naturalist (the three
super-families as biology-coherent PARENTS with the must-not-merge guardrail; the space/time decomposition of
substrate-alignment — Info-Loss = space, Staleness = time; the apoptosis super-family); value-finder (the
worth-sort — frames-not-features; the space/time axis as the highest-leverage taxonomy decision); aristotle
+ adversarial (the crosscut resolution — the third genus, `comparator-divergence`, located at the COMPARE
cell); scout (distinct-remedies → distinct-units, the family-vs-member boundary); pathmaker (the lock,
grounded on the shipped `AntigenCategory` axis).

**Related**:
- ADR-028 (Antigen-Category Taxonomy: Substrate-Alignment vs Functional-Correctness). This grid's **primary
  sort lives inside ADR-028's category line**: the shipped `AntigenCategory` (`category.rs:51` —
  `SubstrateAlignment` / `FunctionalCorrectness`) is the top of the grid; the three genera and the
  super-family parents refine it. This ADR does not replace ADR-028; it populates it.
- ADR-037 (the control-loop frame). The grid's genus axis is **one genus per active loop-stage**
  (information-divergence ↔ SENSE/MEMORY, comparator-divergence ↔ COMPARE, effect-divergence ↔ ACT); ADR-037's
  function axis *found* the third genus (the crosscut). The genus axis is the single sort key (not a separate
  function coordinate) — completed-by-construction against the loop's three active stages.
- ADR-035 / ADR-029 (the fail-direction invariant lives in COMPARE; `comparator-divergence` is the user-code
  dual of antigen's own gate-collapse).
- ADR-006 (recognition-not-design); the super-family parents and the genera *name* structure that already
  ships (VCS-Info-Loss = ADR-026 = the space face; mucosal stale-tolerance = ADR-027 Amd1 = literally "stale"
  = the time face). The taxonomy is recognition of half-built structure, not a proposal.

### Finding

The stdlib families need a taxonomy that (a) does not collapse them into one bucket, (b) keeps members with
*distinct remedies* distinct (so the audit prescribes the right fix), and (c) gives `cargo antigen gaps` a
*finite completeness check* instead of a hand-walk. The Cartographer sweep produced the pieces — the
researcher's genera, the naturalist's super-family parents, the space/time decomposition — and the crosscut
resolution (ADR-037) supplied the missing third genus. This ADR assembles them into one grid.

### Decision: the grid

**Top axis — ADR-028 category (shipped, unchanged):** `SubstrateAlignment` (a representation diverges from
the actual state) vs `FunctionalCorrectness` (a verb produces the wrong output). Every stdlib family already
carries this; the grid refines beneath it.

**Genus axis — THREE divergence-genera, one per ACTIVE loop-stage (the single sort key):**

| Genus | What diverges | Loop-fidelity (ADR-037) | Sub-axes / examples | Fix-direction |
|---|---|---|---|---|
| **information-divergence** | value ≠ truth | SENSE/MEMORY | **SPACE** = Info-Loss (discarded across a transfer/teardown edge) · **TIME** = Staleness (persisted but truth expired) | recover / retain / refresh the information |
| **comparator-divergence** | the guard/check itself is wrong | COMPARE | vacuous-or-defeated guard (deser-flatten-defeats-deny-unknown-fields) · leaky comparison (`==` on a secret) — *N=2, suspected-tier* | **tighten the guard** (the fail-direction invariant's user-code dual) |
| **effect-divergence** | execution ≠ intended | ACT | **ORDER** (iterator-laziness) · **MULTIPLICITY** (resource-leak / deferred-obligation) · **CONTEXT** (compartment-violation — async) | reorder / re-count / re-compartment the effect |

`comparator-divergence` is the genus the two-genera plane was missing; it is the home for
`crypto-non-constant-time` and `gate-collapse-in-user-code`, which are *neither* information- nor
effect-divergence (the value is correct and the effects are faithful; the *comparator* computed a wrong
verdict). Naming it is recognition — the COMPARE cell already had families. **Category pin:**
`comparator-divergence` is `category = hybrid` (the off-diagonal cell — a guard that leaks or passes
vacuously is both a representation-vs-state divergence and a verb-correctness one); per ADR-028's
min-witness rule its witness-type is the *stricter* of the two (substrate-witness). Recognition, not design
— it lands on ADR-028's existing `hybrid` value, not a new field.

**Why EXACTLY three genera (the anti-vacuity grounding, not just "one per stage"):** the three genera are
the three places a **computation step touches data** — it *reads* its inputs (SENSE: information-fidelity),
*tests/compares* them against a guard (COMPARE: comparator-fidelity), and *acts* on the result (ACT:
effect-fidelity). read → test → act is the anatomy of a computation step, so "exactly three user-code
genera" is forced by that anatomy, not fitted to the findings. **Clean prediction (a structural check, not
a claim):** ROUTE and FEEDBACK have *no* user-code genus (they are antigen-response-only stages, not things
a user's program *does* to its own data), so the user-code genus axis populates exactly the three
computation-step stages and is empty at the two response-only stages — an asymmetry that predicts *which*
cells populate, evidence the axis tracks something real rather than relabeling.

**Super-family parents (the remedy-shape unifiers, the within-genus organizing layer):**
- **Information-Loss** (immune amnesia; ADR-026 = the shipped space-face) — members: vcs / error / config /
  coordination / doc info-loss.
- **Staleness** (antigenic drift; ADR-027 Amd1 = the shipped time-face; TOCTOU = the zero-Δt limit).
- **Drop-Lifecycle** (apoptosis + its two failure modes: necrosis = drop-panic / failure-to-apoptose =
  resource-leak).
Each parent is a named shape that **predicts new instances**; members under it are kept **distinct by
distinct-remedies → distinct-units** (`#[descended_from]` the parent, never one merged fingerprint — merging
would prescribe the wrong fix for N−1 branches). Biology supplies both the generator AND the must-not-merge
guardrail (in each parent the members map to mechanisms with different, often opposite, remedies).

**The remedy-shape is the PRIMARY sort; the genus + parent are the coordinates.** A family's home is its
remedy-parent (under its ADR-028 category); its genus (which loop-fidelity it breaks) is the
completeness-coordinate. So `cargo antigen gaps` becomes a **finite check**: *is every permitted
(category × genus × super-family-parent) cell populated, with members kept distinct by remedy?* — concrete,
not a hand-walk. (The PHASE and SCALE axes from the expansionist's lifecycle-converge grid complete the full
multi-axis address for primitives where they apply; they are LOOP-A-scoped and refine, not re-sort.)

### Mechanics: the stdlib/LOOP-A boundary-discriminator (the same clause as ADR-037 §Mechanics)

This ADR catalogs **disturbances D** (the failure-classes antigen detects in USER code); ADR-037 catalogs
the **controller C** (antigen's own regulator). The membership test, identical in both ADRs so a
`families/*` item lands in exactly one home:
**a `families/*` item is STDLIB (a disturbance, this ADR) iff its fix changes USER code; it is LOOP-A
(a regulator-mechanism, ADR-037) iff its fix changes ANTIGEN's own regulator.** Fix-locus is the
discriminator (per the master-frame gate's three-verdict rule, ADR-037). Consequences carried here:
- the regulator-halves of `gradient-routing-chemokine-recruitment` (the ROUTE primitive) and
  `cascade-becomes-the-problem-sepsis-anaphylaxis` (the SCRAM governor) are **NOT** stdlib families — they
  re-file to ADR-037. Only their disturbance-halves stay: `RoutingTableStale` (stale CODEOWNERS) and the
  detectable workspace-aggregate alert-storm class.
- `setpoint-corruption-goodhart` is the canonical **BOTH**: the disturbance `FingerprintGamedNotDefended`
  (user/agent behavior) lives here; the regulator-fix (`#[autoimmune]` as the COMPARE-reference recheck)
  lives in ADR-037. A BOTH-family is split by fix-locus, one half per ADR — never duplicated.

### Why the genus axis is the single sort key — the function axis cited AS fidelity-violated

The A-vs-B fork (does ADR-038 cite the loop-function axis, or stay pure-remedy-shape?) dissolves on one
distinction (the harbor-master's steer): the function axis must be defined as **FIDELITY-VIOLATED, not
stage-that-senses.** "Which stage *perceives* the disturbance?" is vacuous — every disturbance answers
"SENSE" (you never classify pathogens by which stage of your own immune system perceives them); that
vacuity objection (adversarial + naturalist) targets the *stage-that-senses* framing and is correct about
it. But "which loop-FIDELITY does the disturbance *violate*?" **discriminates** — information-fidelity
(SENSE/MEMORY), comparator-fidelity (COMPARE), effect-fidelity (ACT) — and those fidelities are *named by
loop-functions*. The genus is "what KIND of correctness fails" — a property/fix-shape discriminator, the
exact family-boundary method used everywhere else — not a perception one. So the vacuity objection leaves
the *fidelity* framing standing, and **ADR-038's genus axis IS the loop-function axis defined as
fidelity-violated.**

This is why the grid stays single-sort-axis without losing the coordinate: the three genera *are* the three
fidelities, so citing the function-axis-as-fidelity and having-the-genus-be-fidelity-defined are the same
structure — not two parallel fields (which would be the `genus-vs-function double-field` /
`ParallelStateTrackersDiverge` risk). The crosscut test showed the original two genera missed the COMPARE
fidelity-cell; completing the genus axis with `comparator-divergence` *is* citing the function axis (the
function axis earned its keep by **finding** the missing genus — recognition, not a permanent second
coordinate). The payoff is the one Position A would drop: `cargo antigen gaps` becomes a **finite
checklist** ("is every permitted (category × fidelity-genus × super-family-parent) cell populated?"), which
needs the fidelity coordinate to be finite. So ADR-038 cites the loop-function axis (ADR-037), *defined as
fidelity-violated*, as its genus axis — one sort key, the function-as-fidelity coordinate folded in, not a
parallel classification field bolted on.

### What "done well" means (for the notary)

- Every shipped stdlib family has a (category, genus, super-family-parent) address, and members under a
  parent are kept distinct by remedy (a fixture proving two members of one parent get *different*
  prescribed fixes).
- `comparator-divergence` carries its N=2 witnesses (`crypto-non-constant-time`, `gate-collapse-in-user-code`)
  and is surfaced at the suspected tier (ADR-039) — not asserted as a fully-witnessed genus.
- `cargo antigen gaps` (when built) walks the finite (category × genus × parent) cell grid, not a hand list.

### What this ADR does NOT do

- Does NOT replace ADR-028 — it populates ADR-028's category line with genera + parents.
- Does NOT make the control-loop the sort key — the genus axis is the sort key; the loop frame (ADR-037) is
  cited for *why* there are three active-stage genera, not used as a parallel field.
- Does NOT merge a super-family's members into one fingerprint — distinct-remedies → distinct-units is
  load-bearing (merging prescribes the wrong fix for N−1 branches).
- Does NOT assert `comparator-divergence` as fully witnessed — N=2, suspected-tier, pending more witnesses +
  adversarial's witness-attack + captain concurrence.

---

## [ADR-039] The Confidence Dial, the Build Gate, and the Emit Seam: Three Admission Decisions and One Typed-Event Stream

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) — **awaiting the notary** for promotion
to Witnessed. The three-decisions split and the emit-seam locus are locked first-principles + substrate.
The **build-gate admission rule is RULED (Tekgy, permissive — supersedes the earlier crisp-gate): admission
≈ ARTICULABILITY (name/see/imagine it → admitted; no fiction-exclusion gate), justified by the cost
asymmetry (false-positive ≈ ~0, especially PASSIVE; false-negative = the silent failure). The crisp /
constructable / encountered material moves to the dial as an honest PROVENANCE ladder (decision-(b)); a new
PASSIVE [tooling] vs ACTIVE [user-macro] presentation axis keeps permissive admission free; honest
provenance-labeling is the one invariant (adversarial's fiction-intolerance re-homes onto verifying
provenance, not gating entry)** — see §The build gate (the superseded crisp-gate + the earlier fork +
deliberation preserved append-only for the trail).

**Participants**: value-finder (the flagship value-prop — the dial decides *who antigen is for*: the
silent-failure population; the survivor-bias argument for dropping recorded-breakage to a dial; legibility
as the third discipline); naturalist (the lysozyme grounding of the innate/suspected tier; the
naive-repertoire-is-built-on-shape biology that dissolves the gate fork; the affinity-maturation reading of
graduation); dreamer (carried the captain's emit-not-display lock onto the campsite); adversarial (the
emit-seam locus correction — the dial verdict is an audit-time value, so "land it in ScanReport" splits the
stream; the merge-at-audit fix); aristotle (substrate-confirmed the emit-split; the three-decisions
first-principles read); outsider (self-corrected the ScanReport-locus half; flagged the dial-vocabulary ↔
shipped `match_kind` mapping); pathmaker (the lock + substrate verification of the emit locus).

**Related**:
- ADR-037 (the control-loop frame). The confidence dial is the loop's **ACT-stage calibration** (how
  confident before flagging loud); the build gate is the loop's *repertoire-generation* rule (what gets a
  receptor at all). This ADR realizes two of ADR-037's stages.
- ADR-034 (the report is a live projection, never a stored truth). The emit-seam locus is constrained by
  ADR-034: emitting the dial verdict *into* `ScanReport` would force scan to re-derive an audit-time
  computation = a second source of truth = the exact divergence ADR-034 forbids. Merge-at-audit is the
  ADR-034-compatible locus.
- ADR-028 / ADR-006 (substrate-alignment categories; recognition-not-design). The innate/suspected tier is
  *recognition*: it is antigen's own **lysozyme** (constitutive, non-specific, no-memory structural-hygiene)
  — the tier already had this shape; biology names it.
- ADR-019 Amendment 1 (the dial-as-Bayesian-posterior gives the *graduation rule* — the titer/scalar value
  layer's `measure : Substrate ⇀ Scalar` lifts the same way; the dial is a posterior over "is this a real
  failure-class," graduating suspected → named on evidence).
- The downstream learning-loop organs (charter): the affinity-maturation engine, antigen-as-platform, and
  the cytokine-signaling-network all subscribe to *this ADR's typed-event stream*. The emit seam exists for
  them.

### Finding

Three distinct admission/calibration decisions were being conflated under one phrase, "the 2-of-3 build
gate," and the captain's "emit, don't display" lock had a locus error that the substrate exposed. Both are
resolved here.

**(A) The three decisions are distinct and must be named separately.** "2-of-3 gate" hides three different
judgments with different inputs and different consequences:
1. **The build gate** — does a candidate failure-class get a stdlib antigen *at all*? (admission)
2. **The dial tier** — at what *confidence* is an admitted antigen surfaced: innate/suspected (soft, "looks
   like X") vs named/confident (loud)? (calibration)
3. **The dread-declaration** — the `#[dread]`/`#[aura]` marker semantics (asserted-not-hypothetical,
   declared-not-detected, earned + lifecycle). (a distinct authored signal — see ADR-040 / the
   marked-unknown plane)

Collapsing these loses the load-bearing structure: the gate decides *existence*, the dial decides
*loudness*, the dread-marker is an *authored* (not inferred) high-magnitude signal. Each has its own rule.

**(B) The emit-seam locus is the AUDIT stage, not `ScanReport`.** The captain's lock ("the dial verdict +
`#[dread]`/`#[aura]` markers must be queryable as structured typed events, not merely rendered") is correct
and survives. But "the `ScanReport` is already structured, so land the verdict in it" is the degenerate
case — verified on substrate: `Presentation` (`scan.rs:1584`) carries only a binary `match_kind`
(`ExplicitMarker | FingerprintMatch`) and no tier/magnitude/verdict field; the dial's tier-verdict is an
**audit-time** computation (`WitnessTier` at `audit.rs:148`, `ImmuneVerdict` at `audit.rs:992`), produced by
cross-referencing `#[defended_by]` against `#[presents]` — which `scan_workspace` does not do. So the typed
payload is *inherently split*: the `#[dread]`/`#[aura]` half is scan-time (declarations), the dial-verdict
half is audit-time. "Land it in `ScanReport`" can only carry the scan-time half, or forces scan to duplicate
audit's cross-reference (the ADR-034 violation). The three downstream organs were promised *one* schema; a
physically-split stream defeats the seam's whole purpose (one subscribable signal).

### Decision

**(A) Lock three named admission decisions, not one gate.**

- **Admission ≈ ARTICULABILITY-AS-A-STRUCTURAL-TELL — no filtering gate, but antigen-hood is definitional
  (RULED, Tekgy permissive + the captain's cut; supersedes the crisp-gate above).** A candidate failure-class
  is admitted to the stdlib if it can be *articulated as a scannable structural pattern* — named,
  seen-as-a-mechanism, or even *imagined* — **provided it has a structural TELL (a scannable fingerprint).**
  The tell-requirement is **definitional, not a Goodhart gate**: an antigen *is* a fingerprint the scanner
  matches; no fingerprint = nothing to scan = it simply *isn't an antigen* (this is adversarial's structural
  mechanism = the tell IS the causal path; a narrative-only "concern" has no tell). So there is **no
  fiction-exclusion gate at admission** *and* **no separate Goodhart gate is needed** — narrative-only fictions
  cannot flood the *scannable* stdlib because they have no fingerprint to scan with; they route to `#[dread]`
  instead (the site-local marker, the macro-on-the-shelf — see below). The justification for permissive
  admission of everything-with-a-tell is a **cost asymmetry**: a false-positive antigen costs ≈ near-zero
  (especially when it is PASSIVE / tooling-side — see below — and imposes no user-macro burden), while a
  false-negative is exactly the silent failure antigen exists to prevent. When over-coverage is ~free and
  under-coverage is catastrophic, the regulator over-admits (the requisite-variety logic: cover the
  disturbance space; the variety is cheap on the passive side). So decision-(a) is **not a filter** — it is an
  *articulability(-with-a-tell) threshold + a TIERING*. The crisp / constructable / recurred material is **not
  discarded** — it moves to the dial as the provenance ladder (decision-(b)), where it states *which kind* of
  antigen this is rather than *whether* it is admitted.
  - **THREE destinations, by what kind of tell exists (the captain's resolved cut — replaces any earlier
    binary framing):**
    1. **VERIFIED-CORE stdlib** — a structural tell **+ a constructable, verifiably-failing demonstration**
       (the demo is WILD-found OR CONSTRUCTED-and-verified — a *constructed* demo suffices; rarity-tolerant,
       no survivor-bias). This is the crew's Form-3 in its authoritative form (naturalist-captured): the
       verified core of the scannable stdlib, provenance = encountered / constructable.
    2. **PASSIVE-HEURISTIC stdlib** — a **heuristic / correlational tell** (scannable, but NOT
       verifiably-constructable-to-failure — clippy-lint-style, "this shape *correlates* with the failure").
       Admits at a **PASSIVE, honestly-labeled *imagined*-tier** below the verified core (Tekgy's permissive
       ruling). It is a general *scannable class* (so it is stdlib, not dread), surfaced soft / dial-gated,
       and *honestly labeled* as heuristic — it cannot *claim* the verified-core tier it hasn't earned.
    3. **`#[dread]`** — a **narrative-only / site-local hunch** with NO scannable class at all (no
       fingerprint). It is a site-local declaration, not a stdlib class (next bullet).
    The honest-tier IS the Goodhart protection: a manufactured heuristic can ONLY claim the heuristic tier
    (a passive, soft, labeled-heuristic class), never the verified core (which needs a real constructable
    failing demo a fiction cannot produce) — so flooding buys the floor, not the loud center. Recurrence-COUNT
    promotes within the dial; it is not an admission bar.
  - **A tell-less / narrative-only concern is NOT excluded — it takes the `#[dread]` form** (ADR-041, the
    site-local marker). This is where adversarial's "narrative → dread" and Tekgy's "the user has macros
    available for what we imagined" *meet*: a thing you can *feel/narrate at a site* but cannot *articulate
    as a scannable class* has no fingerprint, so it cannot be a stdlib antigen — it is a dread declaration
    instead. (dread = feeling-at-a-site, no class; imagined-antigen = a class WITH a tell, no instance yet —
    the two compose, they do not collide.)
  - **Goodhart, closed STRUCTURALLY (record the objection as resolved-by-this, per adversarial — the reason
    must be durable, not rot).** The objection: "mechanism is infinitely manufacturable by an LLM agent
    (antigen's target user); a narrative-admission gate hands the stdlib's key to the exact optimizer antigen
    names as its deepest self-risk (`FingerprintGamedNotDefended` / Goodhart)." The resolution: admission is
    not by narrative — it is by **structural tell**, and a narrative has no tell, so it *cannot enter the
    scannable stdlib* (it routes to dread). Fiction-intolerance therefore does not live at the entry gate
    (there is none); it lives as the **provenance tier-VERIFIER** (don't *claim* a tier — encountered /
    constructable — you have not earned). The Goodhart hole is closed by the *definition of an antigen* (must
    have a tell) + *honest tier-labeling*, not by a bouncer that the optimizer would learn to talk past.
  - **PASSIVE-by-default for low-provenance + macros-available (the new spec distinction).** An *imagined*
    antigen defaults to **PASSIVE** presentation — it lives scan/tooling-side (a fingerprint the scanner
    carries), imposing **no user-macro burden** on anyone's code. The user-facing macros (`#[presents]`
    etc.) remain **available** for whoever *encounters* the imagined thing and chooses to mark their site:
    **PASSIVE [tooling-side, default for imagined/low-provenance] vs ACTIVE [user-macro, chosen by an
    encounterer]** is a first-class presentation axis. This is what makes permissive admission free: the
    vast field of imagined-but-never-triggered antigens sits passive (costing nothing) until someone
    actually meets one.
  - **Honest labeling is the ONE invariant** (it serves permissiveness; it does not limit it). Every
    admitted antigen carries a **mandatory provenance/tier label**, and an *imagined* one MUST be labeled
    imagined — never dressed up as encountered or constructable. Adversarial's fiction-intolerance
    **re-homes**: it is no longer an entry-bouncer (there is none); it is the **provenance VERIFIER** — it
    catches *false provenance* (a "this was encountered" claim that wasn't, a "constructable" tier with no
    constructable demo). This is observe-don't-declare (ADR-029) applied to admission itself: admit freely,
    but never claim a provenance you cannot observe. The Goodhart hole the crisp-gate worried about closes
    here, not at entry — a manufactured fiction is *admitted* (harmlessly, passive, low-provenance) but
    *cannot be labeled* "encountered/constructable" without a real demonstration, so it stays visibly
    low-provenance and passive, where it costs nothing and misleads no one.
- **The dial tier (calibration) — now an honest PROVENANCE ladder, not just loudness.** Since admission is
  permissive (decision-(a)), the dial carries the work the old gate's crisp-test did — as a **provenance
  tier** stating *how we know this failure-class exists at all*. The ladder has a **VERIFIED CORE** and two
  unverified tiers beneath it:
  - **VERIFIED CORE (Goodhart-safe — the crew's Form-3, honored):**
    - **encountered** (seen in real code — highest provenance), and
    - **constructable** (a minimal case can be *built that verifiably exhibits* the failure — wild-found is
      NOT required; a constructed-and-verified demo suffices). Both count as "shown"; this IS the
      naive-repertoire V(D)J insight — built on a structural possibility you can *instantiate*, not a
      narrative you can manufacture — which is precisely why the verified core is Goodhart-resistant.
    - **Two tier-honesty sub-clauses qualify a `constructable` claim (RULED, captain — they close the
      Goodhart line at the verified core):**
      - **Affinity-pair.** A `constructable` demo must be a **PAIR** — a failing case the fingerprint
        *binds* + a clean sibling it must *not* bind (positive + negative selection). This verifies the
        fingerprint's *affinity* (binds the bad, spares the good), not merely the class's existence — it
        closes the gem/graffiti-fingerprint divergence (a real demonstration can otherwise ship a
        fingerprint whose codomain is wider/narrower than the demonstrated mechanism — antigen's own
        ⊥-collapse class).
        - **Amendment 1 — the SPARES-NAMESAKE sub-test (RULED, harbor-master; the named-common-arm
          overclaim root-fix; Geological Society notary, 2026-06-04). Supersede-not-erase: the affinity-pair
          above stands; this *sharpens which* clean sibling the negative-selection case must be.** A
          **trivially-absent** sibling (a struct that calls nothing, or calls an unrelated method) passes the
          negative-selection half *vacuously* — it proves only that the fingerprint doesn't fire on
          *everything*, not that it spares the **namesake clean case**. So for any **NAMED** member whose
          fingerprint contains a **common-method-name leaf** (`body_calls("name")` / `derives("name")` where
          `name` is not a rare/std-specific self-anchor), the `constructable` affinity-pair's negative case
          MUST be the **same-method, clean-receiver namesake**: a call to *that exact method name* on a
          receiver/in a context where it is **correct**, which the fingerprint must **not** fire on. Rationale
          (the spine): **rarity of the co-presence ≠ anti-correlation with the clean case** — a leaf can be
          rare in the corpus yet still fire on the idiomatic-correct use of its own namesake (the Vec-grow
          `size_of`; the bounded-source `from_slice`). The bare "spares *a* sibling" test does not catch this;
          "spares the *namesake* sibling" does. **Per-leaf, three outcomes (RULED):**
          - **Spares the namesake clean case → the arm STAYS NAMED** (the genuinely-rare self-anchor:
            `transmute`, `assume_init`, `uninitialized`, `from_utf8_unchecked`, `get_unchecked` — no common
            namesake exists, so the negative case is satisfiable and the codomain is the defect population).
            The test is **"does a COMMON *SAFE* method of this name exist?"**, NOT "does the leaf fire on
            *some* receiver." `get_unchecked` illustrates the distinction: it fires on a *domain*
            `SafeGrid::get_unchecked` — but that is **not** a clean sibling, because a method *named*
            `get_unchecked` on any receiver invokes the *same* unchecked-access risk-class (the name carries
            the risk, the receiver-type does not change it). So there is no SAFE namesake → the codomain is
            still the defect population → STAYS NAMED. Contrast `from_slice`/`zeroed`, whose namesakes
            (`serde_json::from_slice` on a bounded buffer, `bytemuck::zeroed`) are the **safe/recommended**
            use of the *same* name — that is the DROP case.
          - **Fires on the namesake's RECOMMENDED-SAFE form (the fix) → DROP the arm at every tier** (the
            clean-sibling-collision: `from_slice` = the bounded-source fix for the streaming-DoS;
            `zeroed` = `bytemuck::zeroed` the safe API; `elapsed` = `Instant::elapsed` the SystemTime fix). A
            needle that flags the recommended remediation is worse than a recall hole — DROP, do not demote.
          - **Fires on a benign/unrelated namesake the discriminator can't separate at AST → DEMOTE to
            suspected** (the labeled recall hole is within-tier-honest at suspected). The worked instance is
            `size_of`-in-element-count (`all_of([copy_nonoverlapping, size_of])`): its own anti-correlated
            **fix** — `copy_nonoverlapping(s, d, n)` with an element count and no `size_of` — *is* spared (the
            `all_of` co-anchor needs both calls), so it is **un-correlated, not anti-correlated → DEMOTE, not
            DROP**; but it fires on two *correct* both-calls siblings (a copy by element count whose body
            separately computes `size_of`; the legitimate single-element byte-copy
            `copy_nonoverlapping(p, q, size_of::<u32>())` on `*u8` pointers), which are honest labeled-recall
            noise at suspected. **Charter the named-promotion only at the precision the discriminator actually
            needs — do not over-promise a syntactic leaf.** Mark it **permanent-suspected** iff the only
            discriminator is the **receiver TYPE** (`set_len`: `Vec` vs a domain buffer; `duration_since`:
            `SystemTime` vs `Instant` — neither exposed at macro/scan time, so no syntactic leaf can ever
            re-earn named); and mark the promotion **type-aware (not a near-term AST operator-leaf)** when the
            discriminator needs resolved types even though an arg-shape is *part* of it — `size_of`-in-count is
            the case: an arg-position leaf (`size_of` *in the count argument*) is **necessary but insufficient**,
            because the correct `*mut u8` byte-buffer idiom (`copy(dst: *mut u8, n * size_of::<T>())`) carries
            the same `n * size_of` shape and is spared only by the **pointee type** (`*u8` = a byte buffer), a
            resolved-type fact — so it graduates at the **v0.4 type-aware tier (arg-position AND pointee-type)**,
            never at a syntactic operator-leaf. *(Honest note on the GRADUATION mechanism, not the outcome:
            `size_of`-in-count IS the worked DEMOTE (outcome-3) member — it ships at suspected. What none of the
            worked members has is a *cheap syntactic operator-leaf* path back to named: `set_len`/`duration_since`
            are permanent-suspected (receiver-type), and `size_of`-in-count is type-aware (arg-position AND
            pointee-type). The pure-AST-feasible-charter *graduation* — re-earning named via a single new
            operator-leaf — is a sound forward-provision for a genuinely syntactic-discriminable arm, but no
            beta.2 member graduates that cheaply; SizeOf's own graduation is the type-aware tier.)*
          The discriminator between DROP and DEMOTE is **anti-correlation vs un-correlation**: an arm that
          fires on the namesake's *fix* is anti-correlated (DROP); an arm that fires on an *unrelated/benign*
          namesake is merely un-correlated (DEMOTE). This is the arm-level dual of the name-specificity tier
          rule — a member can be named-honest on its rare arm and over-claiming on a common arm in the same
          `any_of`. **Enforcement:** every named member with a common-method leaf ships an affinity-pair whose
          negative case is the namesake clean sibling (a runnable probe, not a doc assertion — the build crew's
          "all rare/std-specific" self-description is not the witness); a notary may demand the probe.
      - **Silent-class oracle.** For a **silent** class (no automated verifier *by definition* — tests
        green, behavior wrong is the point), "verifiably exhibits" has no mechanical check, so the demo
        must carry a **STATED ORACLE** — a differential reference / known-answer-vector, OR an ADR-020
        intent-attestation (`attested=(who, why)`) establishing the correct-vs-actual divergence. This makes
        the judgment **explicit + attestable**, not hidden inside "verifiably" where an LLM could
        manufacture the oracle.
    - **`encountered` requires AUTHORSHIP-INDEPENDENCE (RULED, captain — locked invariant + charter-pointer
      for the checkers).** A fingerprint-match alone does **not** qualify for the top `encountered` tier when
      the *same author* planted both the code and the fingerprint (instance-author ≠ antigen-author;
      **signer ≠ author** — the self-planting / inverted-trust hole; cf. `IndependentRecheckAuthority`). The
      *checkers* (git-blame provenance / cross-repo sighting / a second independent attestation) are
      **charter-deferred** to the future provenance-verifier organ (`charter-learning-core.md` →
      self-tolerance / negative-selection) — this ADR locks the **invariant**, not the machinery.
  - **heuristic / correlational** (below the verified core): a *scannable tell that suggests risk without a
    verifiable-constructable failure* — clippy-lint-style (the tell correlates with the failure but is not a
    verified causal demonstration). These **DO admit**, but at the heuristic tier: **PASSIVE-by-default,
    honestly labeled "heuristic" (correlational, NOT causal), below the verified tiers, user-dialed.** This
    is the tier a lint-shaped smell lives at.
  - **imagined / named** (lowest): articulated from shape or reasoning; a class with a tell but no
    constructable demo *yet* — PASSIVE-by-default, a standing request to construct the demo that would
    graduate it into the verified core.
  All four are *admitted*; the tier declares which, honestly. **The honest tier-label IS the Goodhart
  protection (the captain's load-bearing point):** a manufactured heuristic can *only ever* claim the
  heuristic tier — it can never claim `constructable` (it has no verifiable demo) or `encountered` (it was
  never seen), so it is transparently marked unproven, sits passive, and is dial-gated. Permissiveness and
  Goodhart-safety coexist *because of* the honest tier: admit everything-with-a-tell, but a fiction can only
  occupy the lowest, quietest, passive tiers — never masquerade as verified. (Adversarial is the
  tier-VERIFIER blocking false claims to `constructable`+; recurrence-COUNT is a *promotion* signal
  — imagined/heuristic → constructable → encountered → matured — never an admission bar.) This provenance ladder is distinct from but composes with
  the **confidence** reading (the Bayesian posterior over "is this a real failure-class *here*, at this
  site", ADR-019 Amd1) and the innate/suspected→named/confident **surfacing** tier (shape-only soft —
  antigen's *lysozyme*, constitutive/non-specific/no-memory — vs declared/loud): provenance = "how solid is
  the *class*", confidence = "how sure is *this instance*", surfacing = "how loud". Encounter/recurrence is
  the **maturation signal** that promotes provenance (imagined → constructable → encountered) and confidence
  — the dial doing its job, *never* a second admission criterion. Nothing real is rejected; low-provenance
  imagined antigens sit passive and quiet; style-opinions self-sort to the quiet tier without a bouncer.
  This is also the **anti-drowning surfacing rule**: liberal in *coverage*, **ranked** in *surfacing* (the
  third usage-discipline; see ADR on usage-discipline — front-line-liberal / regulatory-sparing /
  ranked-surfacing).
- **The dread-declaration** is specified in the marked-unknown ADR (the `#[dread]`/`#[aura]` plane); this
  ADR only fixes that it is a *third*, *authored*, high-magnitude signal distinct from the inferred dial
  tier, and that it emits into the same typed-event stream (§C).

**(B) Lock the dial-vocabulary ↔ shipped-marker mapping (outsider's flag, prevents a parallel-tracker
divergence).** The new dial tier-words (innate/suspected/named/confident) MUST be explicitly related in the
ADR to the *shipped* binary `match_kind` (`ExplicitMarker | FingerprintMatch`) — either as a refinement of
it or as a stated-orthogonal axis. Shipping two tier-vocabularies for one axis with no stated relation is a
`ParallelStateTrackersDiverge` risk (antigen's own dogfood class) and newcomer-confusion. The mapping:
`FingerprintMatch` (a shape-only structural match, no adopter declaration) is the *innate/suspected* tier's
scan-time substrate; `ExplicitMarker` (an authored declaration) is the *named* tier's scan-time substrate;
the dial tier is the audit-time *confidence grade* computed over these — so the relation is "match_kind is
the scan-time input; dial-tier is the audit-time grade," an orthogonal-but-related axis, stated explicitly.

**(C) Lock ONE typed Finding/Event schema, emitted by BOTH scan and audit, MERGED at the audit stage.** Not
"land it in `ScanReport`." One typed record:
- `site` (file + line) · `class-or-marker` · `timestamp` · `origin-stage {scan | audit}`;
- **`schema_version`** (monotonic) + the locked **forward-compat rule: every future field is additive +
  optional (`#[serde(default)]`); external consumers branch on `schema_version`** (adversarial's
  schema-sufficiency finding #12). The `Finding` record is the EXTERNAL platform contract (the
  antigen-as-platform organ consumes it from outside); an un-versioned external contract is our own
  `SemVer-contract-violation` family dogfooded against ourselves — and ADR-041 already grew the schema once
  (it added `existence-certainty`), proving it *will* grow. `ScanReport` already runs this exact
  additive-optional discipline (the pre-v0.2-deserialize-cleanly pattern); the contract inherits it.
  Recognition, not design.
- **`structural_digest`** (the item's FNV-1a structural fingerprint) — **scan ALREADY computes this**
  (`scan.rs:1621/4402`) for presented/defended/tolerated items and then *discards* it before emit, so
  carrying it is **negative-to-zero cost** (stop discarding a computed value). Without it the
  affinity-maturation engine — the flagship learning-organ that clusters marked-unknown sites and *diffs out
  the common tell* — must RE-PARSE every clustered site from `file+line`, which is exactly the reverse-parse
  the emit-seam exists to kill (just moved from text→location). Load-bearing for organ 1.
- **`cluster-key`** — **specified** (not an opaque label): `cluster-key = derived-from(structural_digest,
  class)`. This is what makes "cluster by shared structure" a field-lookup instead of a re-parse; an
  undefined field in a contract is itself a gap.
- **`class_provenance` {encountered | constructable | heuristic | imagined}** — a **mandatory typed enum**
  (not an `Option`, not a free string): the permissive admission (decision A) is trustworthy *only* because
  this label is always present and honest. This is the typed home of the provenance ladder from decision (B):
  a **verified core** (`encountered`, `constructable`) + two unverified tiers (`heuristic` =
  correlational-not-causal clippy-lint-style; `imagined` = a class with a tell but no demo yet). The honest
  tier IS the Goodhart protection: a manufactured fiction can only ever hold `heuristic`/`imagined` (it can
  never *be labeled* `constructable`/`encountered` without a real demonstration), so it stays passive +
  visibly unproven. The `constructable` value carries the old crisp-test's content as a *tier*, not a gate.
- **`presentation` {passive | active}** — the first-class presentation axis (decision A): an
  imagined/low-provenance antigen defaults `passive` (tooling/scan-side, no user-macro burden); `active` is
  the user-facing macro chosen by an encounterer. This is what makes permissive admission free.
- **`severity`/`magnitude` is a UNIVERSAL field on EVERY Finding** (adversarial finding #12, organ 3): not a
  marked-unknown-plane-only field — the cytokine-signaling organ routes by severity-as-priority, so a named
  dial-verdict Finding must also carry a severity or it cannot be routed. The dread/aura plane (ADR-041) adds
  `existence-certainty` *on top of* the universal `magnitude`; it does not own it.
- the dial `tier` (verdict reading) and, for a marked-unknown, the **required** `trigger` (ADR-041 guard 3 —
  non-`Option` at the declaration site) + `existence-certainty`.

The scan stage emits the `#[dread]`/`#[aura]` half into this record and the audit stage emits the
dial-verdict half, **unified at the audit stage** (audit is the last stage that sees both halves and already
computes the verdict; it merges the scan-time markers with its own verdicts into one population). The natural
home is a new shared `event.rs` / `finding.rs` in the library (or the merged population landing on the
`AuditReport`). This is the only locus that satisfies the seam's purpose (one wire-format for the three
downstream organs) without violating single-source-of-truth (ADR-034). The captain's emit-not-display
*requirement* is preserved exactly; the *locus* is corrected from "ScanReport" to "merge-at-audit" — a
buildability sharpen (adversarial surfaced it, substrate confirmed it), not a weakening.

**Schema ownership (the cross-ADR boundary — prevents a two-schemas `ParallelStateTrackersDiverge`).** **This
ADR (039) is the SOLE OWNER of the `Finding`/event schema** — the record type and its full field-list above.
ADR-036 (the decomposition) owns only the *orchestration* (the thin coordinator + purity invariant) and the
*merge-locus* (where the unified population is assembled), and **cites this ADR for the schema** — it defines
no `Finding` type of its own. So there is exactly ONE schema definition (here) and ONE merge-locus (ADR-036);
building two `Finding` types from two field-lists would be antigen's own `ParallelStateTrackersDiverge` class
at the schema level, which this boundary forecloses. (Cross-cited both ways: ADR-036 §The-two-seams-converge.)

*(Buildability: the full record — `class_provenance` mandatory-enum + `presentation` + the dial tier +
the required marked-unknown `trigger`/`existence-certainty` — was spiked to a compiling, running, serializing
struct (`roles/pathmaker/spikes/seam-spike`): both new typed fields land additively, serialize kebab-cased,
and the run demonstrates the passive-by-default-for-imagined rule (an imagined class emits
`class_provenance: imagined` + `presentation: passive`, an encountered one `active`). "The verdict carries
provenance + passive" is SEEn in a real struct, not asserted.)*

### The build gate: RULED (Tekgy, permissive) — the deliberation ladder, preserved append-only below

**THE RULING (Tekgy, permissive — FINAL, supersedes the crisp-gate below): admission ≈ ARTICULABILITY;
there is no filtering gate.** Name it / see-a-mechanism / *imagine* it → admitted. The crisp /
constructable / encountered distinction is **not a gate** — it is the dial's honest **provenance ladder**
(decision-(b)): encountered → constructable → imagined, all admitted, the tier stating which. Three things
make this sound (see Decision (A) for the locked form): **(1) cost asymmetry** — a false-positive antigen
costs ≈ near-zero (especially PASSIVE/tooling-side), a false-negative is the silent failure antigen exists
to prevent; when over-coverage is ~free and under-coverage catastrophic, the regulator over-admits
(requisite-variety). **(2) PASSIVE-by-default** — imagined antigens live scan/tooling-side with no
user-macro burden (the new PASSIVE [tooling] vs ACTIVE [user-macro] axis); the vast field of
imagined-but-never-triggered antigens costs nothing until someone *encounters* one. **(3) honest labeling =
the one invariant** — every antigen carries a mandatory provenance label; an imagined one MUST be labeled
imagined. Adversarial's fiction-intolerance **re-homes** from entry-bouncer to *provenance VERIFIER*: a
fiction is *admitted* (harmlessly, passive, low-provenance) but *cannot be labeled* encountered/constructable
without a real demonstration — so the Goodhart hole closes at *labeling*, not entry, and a fiction stays
visibly low-provenance/passive where it misleads no one. This is observe-don't-declare (ADR-029) applied to
admission: admit freely, never claim a provenance you cannot observe. Biology backs it cleanly (naturalist):
the naive B-cell repertoire is a vast field of never-triggered PASSIVE receptors — permissive-but-passive is
how immunity is *built*, a metaphor-confirmation for the organizing frame too.

---

*(Superseded ruling — the crisp-GATE (Tekgy override #1), kept append-only and **later further superseded by
the permissive override #2** above. It was right that fiction must be caught; the permissive ruling kept
that finding and moved it from entry-gate to provenance-label.)* **GATE = tell +
crisp mechanism, where crisp = a constructable + verifiably-failing demonstration.** Recurrence is *not* an
admission criterion (it is a dial-promotion signal). The
ruling is *closest to* the adversarial Form-3 shape (fiction-intolerant, rarity-tolerant) but lands the
anchor on **"constructable + verifiably-failing"** rather than "a shown wild instance" — a stronger,
self-contained anchor: you need not find the failure *in the wild*, only be able to *construct a minimal
case that actually exhibits it*. A fiction cannot be constructed-and-shown-to-fail, so this closes the
Goodhart hole (a narrative is manufacturable; a verifiably-failing demonstration is not) while admitting the
rare-but-real silent class on the first encounter (no recurrence needed). The deliberation that produced
this ruling is preserved below (append-only; the fork is superseded, not erased).

---

*(Superseded deliberation — the fork as it stood before the Tekgy override. Kept for the trail.)* The exact
admission rule had two candidate forms; the captain owned the cut.
- **Form 1 — "recurs AND tell"** (the vision §2 sketch): a stdlib antigen requires both a structural tell
  *and* evidence it recurs in real code. Keeps the stdlib bar high; lets the dial/dread carry the
  not-yet-recurring.
- **Form 2 — "tell + (recurs OR mechanism)"** (aristotle's first-principles refinement): a structural tell
  is necessary; generality is established by *either* recurrence *or* a known failure-mechanism — two routes
  to "this is general, not a curiosity." Does not re-exclude the explicable-but-not-yet-witnessed silent
  class.

**Naturalist's biology counsel (recorded as the recommended dissolution, NOT a ruling):** the fork conflates
two distinct immune events. *Building* the recognition capacity = the naive repertoire, generated
combinatorially on structural possibility *before* any encounter (recurrence plays no role). *Committing to
memory* = affinity maturation on encounter (recurrence drives durable memory). The build gate asks the
first; biology builds receptors on **shape/mechanism before encounter** — so a tell+mechanism class is a
naive-repertoire receptor that binds its target the first time. Requiring "recurs" first = requiring prior
infection before building the antibody, which contradicts adaptive immunity's founding mechanism and
re-imports the survivor-bias the gate explicitly rejected (the silent-first population is exactly where the
*first* encounter is the dangerous one — `UnboundedDeserialization` is not safer for being novel to your
code). **The fork dissolves:** tell+mechanism *admits* (held at the suspected tier, low frequency, costing
nothing); recurrence *promotes the tier* (the dial's graduation rule). Recurrence is not a second admission
criterion — it is a maturation signal. The one thing biology will not endorse: making encounter a
*precondition for existence* in the repertoire — that is the anticipatory-immunity property antigen's thesis
is built on. **Recommendation: Form 2** (tell necessary; recurs OR mechanism admits; recurrence promotes
tier). The stricter-bar worry (stdlib fills with curiosities) is answered by the *dial* keeping
unencountered classes at the soft tier, not by an admission bouncer.

**Adversarial's counsel (a THIRD position that splits the difference — and disagrees with the naturalist on
one point):** adversarial accepts the gate-vs-dial split (evidence-strength is a dial concern, not a gate
concern) but **rejects "OR mechanism" as a GATE criterion** — because "mechanism" is *infinitely
manufacturable*: an LLM agent (antigen's stated target user) can construct a plausible failure-mechanism
narrative for any pattern on demand, and this gets *worse* as the optimizer gets stronger (the
`FingerprintGamedNotDefended` / Goodhart self-risk). So "OR mechanism" hands the stdlib's admission key to
the exact adversary antigen names as its deepest self-risk, flooding the stdlib with
mechanism-justified-but-never-recurring speculation — the very noise the `#[dread]` discipline rejects.
Adversarial's resolution: **GATE = tell + at-least-one-SHOWN-real-instance** (one real instance shown, *not*
a recurrence-*count* threshold — so it is rarity-tolerant but fiction-intolerant; rarity is fine,
fictionality is not). Then *mechanism-only-without-any-shown-recurrence* does **not** enter the built stdlib
— but it is **not discarded**: it is exactly what `#[dread]`/`#[aura]` (ADR-041, the inside-out driver) is
*for* (a standing request to find the recurrence that would graduate it into a built family).

**The genuine disagreement for the captain to resolve:** the naturalist says *mechanism admits* (to the
suspected tier of the built stdlib); adversarial says *mechanism-only does NOT admit to the built stdlib —
it lives at the `#[dread]` frontier until one real instance is shown.* Both route mechanism-only candidates
to a low-confidence home that isn't a loud stdlib fingerprint; they differ on **whether that home is a
suspected-tier built fingerprint (naturalist) or a `#[dread]` marker (adversarial).** The Goodhart argument
is the load-bearing new input: it is *why* "shown instance" beats "mechanism" as the non-gameable anchor.
Three options were on the table — Form 1 (recurs AND tell), Form 2 (tell + recurs-OR-mechanism), Form 3
(adversarial: tell + at-least-one-shown-instance; mechanism-only → `#[dread]`). **RESOLVED by the Tekgy
override at the top of this section: tell + crisp-mechanism (constructable + verifiably-failing); recurrence
is a promotion signal, not an admission criterion.** The override refines Form 3's non-gameable anchor from
"a shown wild instance" to "a constructable verifiably-failing demonstration" — same fiction-intolerance,
stronger because it needs no wild sighting.

### What "done well" means (for the notary)

The dial demonstrably **admits a shape-only / silent-failure fingerprint at low confidence** (proving the
survivor-bias bouncer is gone) AND demonstrably **surfaces a style-opinion ("function too long") at the
quiet tier without a gate rejecting it** (proving the dial self-sorts noise instead of needing a bouncer).
The typed-event stream is demonstrably **one schema both stages emit into, queryable as structured output**
(proving the seam is an emit, not a render — a test that subscribes to the merged population and reads a
dread-marker AND a dial-verdict from one stream). If the built artifact cannot show all three, it did not
deliver the value this design claims.

### What this ADR does NOT do

- Does NOT leave the build-gate admission rule open — it is RULED (Tekgy override): tell + crisp-mechanism
  (constructable + verifiably-failing); recurrence is a dial-promotion signal, not an admission criterion.
- Does NOT specify the `#[dread]`/`#[aura]` marker semantics — that is the marked-unknown plane ADR; this
  ADR only fixes the three-way split and the shared emit stream.
- Does NOT build the downstream learning-loop organs — they are charter; the emit seam exists so they are
  buildable later for ~free (one schema to subscribe to, not a reverse-parse of rendered text).
- Does NOT land the dial verdict in `ScanReport` — the locus is merge-at-audit (ADR-034 single-source).

---

## [ADR-040] The Grammar Increment: Frame-Relative Matching, `body_calls`, and the Syntactic Absence Family (Leaf-Matcher Tier)

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) — **awaiting the notary** for promotion
to Witnessed. Buildability-confirmed against the real `antigen-fingerprint` crate by the pathmaker. This
ADR locks the **leaf-matcher tier** (Increments 1–2: new `Constraint` variants in the existing per-node
walk). The **per-type correlation tier** (Increment 3 / G4) is carved to its own sibling ADR (it is a
different machine); the **semantic tier** (resolved types, control-flow liveness) is **charter → v0.4**
(`ra_ap_syntax`, unblocked by the MSRV-1.95 raise).

**Participants**: researcher (the ranked, demand-counted, depth-split driver analysis — the grammar
recognition rule, N≥2 verified-classes-demand-it, the per-increment fan-out); value-finder (the
build-order-by-proven-fan-out worth-call — cheap + proven-demanded + closes-a-real-silent-gap leads);
naturalist (the NK missing-self biology — absence-detection-given-context = the anti-graffiti anchor rule,
biology predicted the existing constraint; the syntactic/semantic depth-split + the locality bend); scout
(the absence-grammar-IS-the-SENSE-organ reframe — MOVE-1 on three independent build-order axes); pathmaker
(the buildability lock — substrate verification of the `Constraint` enum + the shipped anchor rule + the
drop-panic silent gap).

**Related**:
- ADR-010 Amendment 3 OQ3 (the `not`-placement / anti-graffiti rule). **Already shipped** and load-bearing
  here: a bare `not` is a parse error; `not` is legal only inside `all_of` alongside ≥1 positive matcher
  (`antigen-fingerprint/src/parser.rs:332` `check_not_placement`). This ADR's absence family *depends on*
  that rule — every absence fingerprint is `all_of([positive_anchor, not(absent_credential)])`, never a
  bare absence. Biology predicted the rule (NK fires on missing-self *given context*, spares MHC-I-less red
  blood cells). The absence family is *recognition* of a dimension the anchor rule already constrains.
- ADR-006 (recognition-not-design), applied at the grammar level: a dimension is built when **N≥2 real,
  verified failure-classes demand it** — the same gate as families, one layer down. Every increment below
  clears it with verified demand-counts.
- ADR-037 (the control-loop frame). The absence/presence grammar is the regulator's **SENSE organ**: the
  ROUTE/ACT/FEEDBACK stages are dead without a working sensor, so this is foundational, not additive — the
  unambiguous MOVE-1 build (sense-organ × silent-dominant × syntactic-before-v0.4 all agree).
- ADR-009 Amendment 1 (fingerprint required iff scan-locatable). These new leaf matchers are all
  scan-locatable syntactic tells, so they extend the scan-side fingerprint vocabulary directly.

### Finding

The shipped fingerprint alphabet (ground truth, `antigen-fingerprint/src/lib.rs:120`) is:
`Item` · `NameMatches` · `Variants` · `HasMethod` · `AttrPresent` · `DocContains` · `BodyContainsMacro` ·
`AllOf` · `AnyOf` · `Not`. The Cartographer sweep hit the **same expressivity gaps repeatedly** across
blind veins, and each gap is demanded by N≥2 *verified* (recurs-in-real-code) failure-classes:

- **No body *call* matching** — only `body_contains_macro`. The shipped `PanickingInDrop` fingerprint
  matches `body_contains_macro("panic")` / `unreachable` / `todo` (verified: `examples/basic.rs:47–49`)
  and therefore **silently misses the call-shaped panic paths** — `.unwrap()`, `.expect()`, indexing,
  overflowing arithmetic — which are *method calls / operators, not macros*. This is a silent gap in
  antigen's *own* shipped stdlib: the highest-leverage instance of the missing dimension.
- **No attribute *absence*** — only `AttrPresent`. "X present but its required guard Y absent" is unsayable.
- **No trait-impl identity** — `HasMethod` cannot reach method-less marker/contract traits (`Send`, `Sync`,
  `Hash`, `Eq`); `Item` matches only the *kind* `impl`, not "does T impl Trait?".
- **No item qualifier** — `async` / `unsafe` / `const` presence-or-absence is not matchable.

The frame-relative insight that unifies the absence-detection family (NK missing-self) and the
compartment-violation family (async) is itself *recognition*: both reduce to "a property of an item is a
failure-class only read against its FRAME" = `all_of([frame_predicate, operation_matcher])`, combined via
the **existing** `all_of`, gated by the **already-shipped** anchor rule. So the grammar increment is not a
new combinator — it is the missing **frame-predicates and operation-matchers** (new leaf `Constraint`
variants) that the frame-relative pattern needs.

### Decision: lock two leaf-matcher increments (build-now), carve the rest

**Increment 1 (the keystone) — `body_calls(path)`.** A new `Constraint::BodyCalls(path-pattern)` matcher:
the twin of the shipped `BodyContainsMacro`, extended from macro invocations to fn/method calls. Lowest
design risk in the sweep (an existing primitive's shape, one token-kind wider) for the highest demand
(**≥11 verified classes**: `BlockingCallInAsyncFn`, the `.unwrap`/`.expect` sources `PanickingInDrop`
misses, `PanicInLibraryApi`, `SwallowedResult`/`.ok()`, `ErrorContextStripped`, `UnboundedDeserialization`
(`from_reader`/`from_slice`), `SizeOfInPtrCopyCount`, `UninitMemoryAssumedInit` (`assume_init`),
`TransmuteSizeOrLifetimeMismatch`, `UnvalidatedFromUtf8Unchecked`, `SpawnedFutureNotAwaited`). Includes the
arg-position sub-capability (e.g. `size_of` in the *count* arg, not a divisor) where a class needs it.
**This increment alone closes the shipped `PanickingInDrop` silent gap.**

**Increment 2 (the absence family / SENSE organ) — three leaf matchers:**
- **G1 — item qualifier presence/absence** (`is_async` / `is_unsafe` / `is_const`, usable inside `not(...)`
  for the absence case): demanded by `BlockingCallInAsyncFn` (async), `UnsafeSendSync` (unsafe impl),
  `RawPtrDerefInSafeFn` (unsafe-*absence*: a `pub fn` that derefs a raw pointer but is **not** `unsafe`),
  `MissingSafetyInvariantDoc`. **≥4 classes.**
- **G1b — attribute absence + derive/attr-arg introspection**: `attr_absent(path)` (the negation of the
  shipped `attr_present`) plus derive-list-member / attr-arg introspection (is `Debug` *in* the
  `#[derive(...)]` list; is `deny_unknown_fields` *in* the `#[serde(...)]` args). Demanded by
  `DeserializeWithoutDenyUnknownFields`, public-type-missing-`Debug`, `DenyUnknownFieldsDefeatedByFlatten`,
  `derive(Hash)`-on-float. Cheapest of the absence family (derives ARE attributes — no trait-path
  resolution). **≥4 classes.**
- **G3 — trait-impl identity, presence AND absence** (`impl_of_trait("Send")`,
  `not(impl_of_trait("Hash"))`): reads the `impl Trait for Type` trait-path — **syntactic, no
  rust-analyzer**. The only structural tell that can reach method-less marker/contract traits. Demanded by
  `UnsafeSendSync`, manual-`PartialEq`-without-`Hash`, manual-`Ord`-vs-`Eq`, public-type-missing-`Debug`;
  also lets `PanickingInDrop` assert the impl is *actually* `Drop` (today it only checks a method *named*
  `drop`). **≥4 classes.**

Every Increment-1/2 matcher is a **leaf addition** — a new `Constraint` variant in the existing per-node
walk (`match_constraint` in `matcher.rs`), parsed in `parser.rs`, evaluated in `Match3`. Low architectural
risk. Each must be usable inside `all_of([anchor, …])` and (for the absence cases) inside `not(...)` only
under the shipped anchor rule — no bare-absence fingerprint is expressible, by construction.

### The anchor-required invariant is already enforced (recognition, not new work)

The absence family does **not** require a new safety rule. ADR-010 Amendment 3 OQ3 already ships
`check_not_placement` (`parser.rs:332`): a bare `not` is rejected; `all_of` containing only `not` children
is rejected (≥1 positive matcher required); the De Morgan `any_of([not, not])` loophole is closed. So every
absence fingerprint this ADR enables is *structurally forced* into the meaningful form
`all_of([positive_anchor, not(absent_credential)])` — e.g. `all_of([holds_raw_pointer, not(impls(Drop))])`,
`all_of([item=struct, derives(Hash), not(impls(Eq))])`. Biology predicted this exact constraint (missing-self
*given context*); the grammar already had it. The absence family is recognition of a dimension the existing
anchor rule already makes safe.

### What is carved out (sibling ADR + charter)

- **Increment 3 / G4 — per-type correlation (a DIFFERENT machine): its own sibling ADR.** Matching the
  *relationship between items* (struct derives `Hash` AND a *separate* `impl PartialEq`; signature ↔ body
  correlation) requires the matcher to build a **per-type index across items** — a per-node → per-type-graph
  shift, the most architecturally significant unlock in the sweep and the most on-thesis ("topology carries
  information no single node states"). Its *syntactic* correlation slice is build-now-eligible (no type
  resolution) but it is **not a leaf matcher**, so it is locked in a separate ADR and pairs naturally with
  the ADR-036 scan decomposition (the per-type index wants the modular scanner). G5's *syntactic* cast-shape
  slice (literal `x as u32` between named primitives; `*const T` param) rides with it.
- **Charter → v0.4 (`ra_ap_syntax`, MSRV-1.95-unblocked): the semantic tier.** Resolved-type identity
  (G5 resolved-width: is `usize` 32 or 64 here?), resolved-field-type (G4 semantic), and body-liveness /
  control-flow (`LockHeldAcrossAwait` — typed binding live across a suspension point). These need
  rust-analyzer-grade analysis. The confidence dial (ADR-039) bridges the depths honestly: the syntactic
  slice ships at the **suspected** tier ("looks like missing-Drop, low confidence"); the semantic slice
  graduates it to **named**. Same fingerprint shape both depths; the dial makes the syntactic
  false-negatives honest rather than silent.

### What "done well" means (for the notary)

Each shipped leaf matcher points at **≥2 real families it unblocked** (proving it was not speculative), and
**`body_calls` demonstrably closes the shipped `PanickingInDrop` fingerprint's `.unwrap`/`.expect` silent
gap** (a fixture where the old macro-only fingerprint is silent and the new `body_calls` fingerprint fires).
Every absence fingerprint is structurally forced through the anchor rule (a bare-absence attempt is a parse
error — a compile-fail fixture confirms it).

### What this ADR does NOT do

- Does NOT add a new combinator — `all_of`/`any_of`/`not` are unchanged; this adds leaf *predicates*.
- Does NOT add a new anchor/safety rule — the anti-graffiti invariant already ships (ADR-010 Amd3 OQ3).
- Does NOT build the per-type correlation machine (G4) — that is a sibling ADR (different machine).
- Does NOT reach resolved types or control-flow — that is charter → v0.4 (`ra_ap_syntax`); the dial bridges
  the syntactic/semantic depth honestly via the suspected → named graduation.

---

## [ADR-041] The Marked-Unknown Plane: a Declarable Three-Valued Bottom on Two Orthogonal Axes (Magnitude × Existence-Certainty), Surfaced at the Dial's Non-Gating Floor

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) — **awaiting the notary** for promotion
to Witnessed. The plane structure, the two-guard earned-ness discipline, and the emit-seam are converged
(aristotle Phase-1-8 + value-finder worth + naturalist biology). **The third marker is RULED `#[red_flag]`**
(captain — closing the naming sub-fork). `#[red_flag]` is the clinical-red-flag sense (certain-enough-to-
escalate, names-nothing-specific = the inverse of pathognomonic) — no cross-domain collision (unlike
`#[sentinel]` = the watcher) and no within-family ambiguity. `#[alarmin]` was a viable lean (the
DAMP/act-now-danger fit is real) but set aside on a *mild level-mismatch* (alarmin is the **family-wide**
danger-signal substrate, slightly broad for one corner-marker — naming one marker after the genus-word).
**Correction for the notary (naturalist self-withdrew, substrate-checked):** the earlier "`#[alarmin]` =
`#[dread]`'s referent → within-family collision" objection was **false and is retracted** — `#[dread]`'s
cited referent is *angor animi*, not alarmin; there is no collision. `#[alarmin]` was set aside on the mild
level-mismatch only, not a collision. See §The third marker.

**Participants**: value-finder (the worth-proof — the keystone primitive: continuity-of-suspicion across
the handoff where software loses the most knowledge; the affect-is-the-gate); naturalist (the angor-animi /
clinical-gestalt biology — the patient/observer provenance split; the magnitude × certainty 2-D plane; the
alarmin referent); aristotle (the load-bearing correction — marked-unknown is OFF the dial's classification
axis at ⊥ with its OWN existence-certainty axis; the ⊥-attestation-with-lifecycle as the second structural
guard); outsider (the `#[sentinel]` naming-collision catch); pathmaker (the lock + the emit-seam alignment
with ADR-039).

**Related**:
- ADR-039 (the dial + emit seam). The marked-unknown markers are **scan-time declarations** that emit into
  the scan-time half of ADR-039's one typed Finding schema; they are surfaced **at the dial's non-gating
  floor** — same surfacing policy as the suspected tier, *different semantics* (they are at ⊥, off the
  classification axis, not low-on-it). The dial *classifies* nameable things; the marked-unknown is the
  *unnameable*.
- ADR-035 (the three-valued type law). The marked-unknown is the **declarable** form of ADR-035's ⊥: a
  site-verdict becomes three (failure / clean / marked-unknown), and the third is now an authored,
  first-class, surfaced object instead of a vanished feeling. This is ADR-035's ⊥ given a voice.
- ADR-020 (attestation — site + author + why). A marked-unknown is an **attestation of ⊥**: a named author's
  claim of an unnameable danger at a site, carrying the felt trigger. This makes it falsifiable (the
  lifecycle) and is the first of the two earned-ness guards.
- ADR-031 (revocation-staleness). An un-investigated marked-unknown is itself a substrate-smell
  ("⊥-declared here for N commits, untouched") — the second guard, making abuse self-surfacing.
- ADR-037 (the control-loop frame). The marked-unknown is the **inside-out research-driver** that
  complements the outside-in sweep: every marker is a standing request to grow the substrate/grammar/sensor
  that *would* name it — the input port of the future affinity-maturation engine (charter).

### Finding

The single most perishable piece of knowledge in software is the **felt-but-unnamed danger**: a developer
or agent looks at code, senses the floor is rotten, and has nowhere to put that. The unease evaporates on
context-switch / session-end (the dominant outcome for agents, who compact) or rots as a vague TODO. The
agentic era makes it *more* perishable. Antigen's other primitives cannot reach this corner — they all need
a nameable tell. The clinical analog is **angor animi** (the sense of impending doom), which good clinicians
*heed* precisely because it correlates with serious pathology *before* anything localizes — a low-certainty
/ high-magnitude alarm that saves lives by being investigated rather than dismissed.

The design risk is that a marker for "something is wrong, I can't name it" degenerates into `#[unknown]`
graffiti — exactly the speculative "this pattern *could* harbor a problem" noise antigen rejects. The
finding is that two structural properties (not merely cultural restraint) keep it honest, and that the
marked-unknown lives on a **2-D plane**, not a 1-D magnitude line.

### Decision: the plane (two orthogonal axes)

The marked-unknown is **off the dial's classification axis** (it is at ⊥ — the unnameable) and has its own
two axes:

1. **Magnitude** (smell → aura → dread): how loud the unease is.
2. **Existence-certainty** (might-be-something → sure-but-unnameable): how sure the author is that *anything*
   is wrong — **distinct from the dial's classification-certainty** (how sure we are *what* is wrong). The
   dial's certainty is about *which class*; this axis is about *whether there is a class at all*.

Three earned markers tile the relevant corners (the low-magnitude / low-certainty "passing smell" corner is
absorbed by `aura`, so the requisite variety is 2 × 2 − 1 = **3 markers**):

- **`#[aura]`** (the light sibling) — **low magnitude**: "something *may* be off here, can't name it, check
  later." (Also spellable `#[prodrome]`.)
- **`#[dread]`** — **high magnitude, low existence-certainty** (angor animi / patient-conviction-of-outcome):
  "something *is* wrong here, can't name it, look now." Scared-but-unsure.
- **`#[red_flag]`** (the third marker) — **high existence-certainty, unnameable** (clinical sense-of-alarm /
  observer-conviction-of-wrongness): "I'm *sure* something is wrong here, I can't name it, act now." The
  sure-but-unnameable corner; auto-escalates on first match. (A *red flag* is the clinical term for a sign
  demanding immediate evaluation — fits the `#[aura]`/`#[dread]` clinical family.)

All three are **DECLARED, not detected** (semantic intent-bearing declarations — they share the dial's
non-gating floor with machine fuzzy-suspicion but are sourced by a person/agent, not inferred) and
**ASSERTED, not hypothetical** (a felt observation, never "this pattern could theoretically harbor a
problem").

### The three structural earned-ness guards (not just the name's gravity)

1. **The affect-gravity of the name self-selects sparing use.** You can't spray `#[dread]` — the word's
   weight makes "the floor feels rotten" the only honest use. Strip the affect (a neutral `#[unknown]`) and
   it becomes rot-graffiti; keep it and it is an earned signal. (value-finder's "the affect IS the gate.")
2. **The marker is a typed, authored, lifecycle-tracked ⊥-attestation with a built-in staleness-smell.** It
   is an attestation (ADR-020: site + author + trigger), therefore falsifiable (declared → investigated →
   named | cleared), and an un-investigated one is itself a substrate-smell (ADR-031-flavored: "⊥-declared
   here for N commits, untouched"). So abuse is *self-surfacing*, not merely discouraged.
3. **The trigger field is REQUIRED, not optional** (outsider's load-bearing ask, routed via aristotle;
   ruling confirmed-current by the captain at transcription time). The vision §5 phrasing "ideally carrying
   the trigger" reads as *optional* — but an optional trigger is the hole through which the graffiti
   returns: a triggerless `#[dread]` is exactly the contentless "this seems off" mark the primitive exists
   to prevent. A marked-unknown without a stated trigger ("what did you see that made you feel this?") is
   not an asserted observation; it is the hypothetical speculation the asserted-not-hypothetical rule
   rejects. **Lock the trigger as a REQUIRED field (NOT `Option<String>`); a `#[dread]`/`#[aura]` with no
   recorded trigger is audit-flagged** — this is the **rationale-as-required-field transverse sub-clause-F
   discipline (ADR-005 Amendment 2), the same shape as `#[antigen_tolerance]`'s rationale (ADR-011) and
   `#[anergy]`'s reason (ADR-023's min-chars).** It is the third structural guard, and the one that makes
   guard (1)'s "asserted, not hypothetical" *checkable* rather than cultural.
   - **The honest boundary (so the guard does not over-claim — the sub-clause-F reconciliation):** what is
     enforced is the **PRESENCE of a trigger**, not its sincerity. Sincerity stays a **social /
     non-compiler-checked boundary, named as such** (observe-don't-declare, ADR-029 — antigen marks the
     shape of its own ignorance rather than pretending the edge of enforcement is the edge of what's real).
     A *lazy/abandoned* dread (no trigger, or an untouched marker past its lifecycle) is structurally
     catchable; an *insincere* one is not — and the ADR states that boundary outright. This is the
     validate-the-claims-MADE / force-nothing-un-tabled invariant: trigger-presence is the claim held to
     account, sincerity is un-tabled (not forced).

**Lock all three guards** — the name's gravity selects sparing use; the attestation-with-lifecycle makes
abuse visible; the required trigger makes "asserted, not hypothetical" a structural fact, not an honor system.

### Surfacing: floor of the dial, never gates, never nags

A marked-unknown surfaces on scan/audit at the **non-gating floor** of the dial, **never gates** (it cannot
fail a CI build) and **never nags** (it is not re-raised every run as an error). It carries its trigger. An
untouched marker surfaces as a *mild* substrate-smell, not an escalating alarm — except the third
(high-certainty) marker, which auto-escalates on first match (that is its whole point). This preserves the
captain's 3-marker ruling and makes it rigorous: the third marker is *forced* by the orthogonal
existence-certainty axis, not an arbitrary addition.

### Emit seam (aligned with ADR-039)

The markers are scan-time declarations → they emit into the **scan-time half** of ADR-039's one typed
Finding schema (`site` + `marker` + `magnitude` + `existence-certainty` + `trigger` + `cluster-key` +
`timestamp` + the mandatory `class_provenance` {encountered | constructable | heuristic | imagined} +
`presentation` {passive | active}). A marked-unknown is an authored mark at a site the author *encountered*, so it carries
`class_provenance = encountered` and is `presentation = active` (the author chose to mark their own site) —
distinct from the *imagined*/passive tooling-side antigens that default to no user-macro burden (ADR-039 §A).
For a marked-unknown the **`trigger` is populated (required at the declaration site, per guard
3) — not the generic-Finding "optional trigger"**: the maturation engine clusters on trigger-similarity, so
a triggerless mark is both graffiti (guard 3) and an un-clusterable event. **Existence-certainty MUST be a
first-class schema field**, *not* folded into the generic dial-tier — or the future affinity-maturation
engine cannot distinguish high-value sentinel/alarm-clusters (investigate-now) from low-priority
aura-clusters. The dial verdicts (classification tier) emit into the
audit-time half; both merge at audit (ADR-039 §C). The marked-unknown's emit-half is exactly what the
maturation engine clusters: N related dread-marks at related sites → a proposed fingerprint = the inside-out
research-driver mechanized.

### The third marker: `#[red_flag]` (RULED — captain, closing the naming sub-fork)

The high-certainty / act-now / unnameable marker is **`#[red_flag]`**. The deliberation, preserved:
- **`#[sentinel]` — OUT** (outsider, substrate-checked): "sentinel" is already spoken-for as (a) a textual
  sentinel *value* (`audit.rs:2116/5069/5151`) and (b) "sentinel sites" in the epidemiology map meaning
  *surveillance / watch-and-report* — almost the **opposite** of this marker's auto-escalate-on-first-match
  posture.
- **`#[alarmin]` — set aside (a viable lean, on a MILD level-mismatch — NOT a collision).** The DAMP /
  act-now-danger fit is real (an alarmin *is* the "act now, danger present" signal). It was set aside on a
  *mild level-mismatch*: alarmin is the **family-wide** danger-signal substrate (the whole marked-unknown
  sensing system's mechanism), so naming one corner-marker `#[alarmin]` is a genus-word-for-one-species
  slight (like naming one antibody "immunoglobulin"). **Correction (naturalist self-withdrew on substrate-
  check — recorded so the notary does not act on the false objection):** an *earlier* objection claimed
  "alarmin = `#[dread]`'s referent → within-family collision" — that is **FALSE and retracted**: `#[dread]`'s
  cited referent is *angor animi* (this ADR, §the markers), not alarmin (alarmin is the family substrate,
  not dread's specific referent). There is **no within-family collision**; `#[alarmin]` lost only on the
  mild level-mismatch, and `#[red_flag]` was preferred on the cleaner level-fit + exact clinical semantics.
- **`#[foreboding]` — OUT**: foreboding is **low-certainty** (a vague sense something *might* be coming) —
  which contradicts the third marker's defining property: **high-certainty / act-now**.
- **`#[red_flag]` — RULED IN**: it is the actual **clinical term** (a *red flag* is a sign that demands
  immediate evaluation), so it fits the clinical naming family (`#[aura]` = migraine aura, `#[dread]` =
  angor animi) AND is semantically exact for "high-certainty, unnameable, investigate NOW." This ADR locks
  the *corner* (high-certainty/unnameable/auto-escalate-on-first-match) **and the spelling: `#[red_flag]`.**

### What "done well" means (for the notary)

A marked-unknown declared on a real site (a) surfaces at the dial's floor on scan/audit, (b) **never gates
and never nags**, (c) carries its trigger, (d) an untouched marker is surfaced as a mild substrate-smell
("ignorant here for N commits"), and (e) existence-certainty is a distinct queryable field from
classification-tier (a fixture where folding them would mis-rank a high-certainty marker as low-priority
proves the field must be separate). If the built primitive nags, gates, or lets a marker rot invisibly, it
became the graffiti it was designed to prevent.

### What this ADR does NOT do

- Does NOT put the marked-unknown ON the dial's classification axis — it is at ⊥, off-axis, on its own
  magnitude × existence-certainty plane; it merely shares the dial's *non-gating floor surfacing policy*.
- Does NOT gate or nag — it is the inside-out research-driver, not an error.
- Does NOT lock the third marker's spelling — `#[sentinel]` vs `#[alarm]`/`#[alarmin]` is the naturalist's
  biology-fit cut + outsider naming-check.
- Does NOT build the affinity-maturation engine — that is charter; the first-class existence-certainty
  schema field is the cheap stub that keeps it buildable later (the engine subscribes to this emit-half).

---

## [ADR-042] The Usage-Discipline: Three Disciplines (Front-Line Liberal · Regulatory Sparing · Ranked Surfacing), and the `#[autoimmune]` Naming Reconciliation

**Status**: Locked design (Outfitters / beta.2 voyage, 2026-06-02) — **awaiting the notary** for promotion
to Witnessed. The three-discipline structure and the `#[autoimmune]` naming reconciliation are converged
(value-finder legibility-spine + outsider naming-catch + naturalist ruling, all substrate-checked). Mostly
**recognition** — the confidence dial (ADR-039) already exists; this names it as the anti-drowning surfacing
discipline and corrects a backwards-reading name before it ships.

**Participants**: value-finder (the legibility spine — surfacing as the third, currently-implicit discipline;
the drowning-in-both-directions refinement); expansionist (the cascade-governor as the surfacing-keeper's
storm half); outsider (the `#[autoimmune]`-reads-backwards catch, substrate-grepped); naturalist (the
thermostat-is-not-the-fever ruling — screen-mode vs site-marker direction-rule); pathmaker (the lock + the
ADR-037 cross-reconciliation).

**Related**:
- ADR-039 (the dial). The ranked-surfacing discipline IS the dial pointed at *output* (anti-drowning), not
  only at *admission*. This ADR names that third use of the dial explicitly.
- ADR-041 (the marked-unknown plane). Over-reassurance is surfaced as a marked-unknown-shaped signal
  (never-denied guards → suspected-no-ops).
- The glossary (`glossary.md:258`) + ADR-context (`decisions.md:5574`) — both anchor
  **autoimmunity = the failure-mode** (over-flagging legitimate code). This ADR preserves that and adds the
  screen-vs-marker direction-rule to the glossary.
- The **shipped** `#[antigen_tolerance]` (W6a) + `#[anergy]` (ADR-023) — the per-site self-tolerance
  primitives that already exist; the regulatory discipline composes them, not a new `#[autoimmune]` marker.
- ADR-037 (the control-loop frame). `#[autoimmune]` appears there as the COMPARE-reference-failure; this ADR
  reconciles that reference: the *failure-class* is `setpoint-corruption` / `FingerprintGamedNotDefended`;
  the *detector* is an audit-mode screen (`autoimmune-check`), not a site-marker (see §The reconciliation).

### Finding

The masterclass is, by construction, the **densest-marked codebase that will exist** (the legibility-spine
finding). If its audit output is an undifferentiated wall, the masterclass *anti-teaches* — it contradicts
the discipline it exists to demonstrate. Two distinct things were implicit and one name was backwards:

1. **There are THREE disciplines, not two.** Front-line (liberal coverage) and regulatory (sparing
   high-cost marks) were named; the **surfacing** discipline (rank what's shown so dense-correct marking ≠ a
   wall of noise) was implicit. And drowning runs in **both directions**: over-alarm (too many marks shown
   flat) and **over-reassurance** (rubber-stamp no-op guards making a wall of green that falsely teaches
   "green = safe").
2. **`#[autoimmune]` reads backwards.** "autoimmunity" means the *disease* (over-flagging) everywhere in the
   project, but the campsite used `#[autoimmune]` for the *regulator that detects over-flagging* — inverting
   the metaphor at the most-read surface (the macro name). There is no shipped `#[autoimmune]` macro yet, so
   this is the moment to get it right.

### Decision (A): lock the three disciplines

1. **Front-line — liberal in COVERAGE.** Mark broadly; the build gate (ADR-039) admits on tell + shown
   recurrence; nothing real is excluded. Coverage is generous.
2. **Regulatory — SPARING in high-cost marks.** The high-gravity / high-cost declarations (`#[dread]`, the
   deliberately-tolerated exceptions) are used sparingly; their cost self-selects restraint (ADR-041's
   earned-ness; `#[antigen_tolerance]`'s Treg-license).
3. **Surfacing — RANK × BUDGET × TRIAGE-STATE (three inputs, not two — the anti-drowning keeper, both
   directions).** *Ranking alone does NOT cure habituation* (adversarial's break, biology-forced): ranking
   reorders a single *snapshot*, but habituation is a function of the *delta* between what the reader saw
   last run and sees now — and rank has no reader-state coordinate. Worse, a stable high-confidence /
   high-blast **accepted-risk** item permanently occupies the top ranks and *fills the output budget*, so a
   genuinely-NEW finding lands below the cutoff and is never shown — **the anti-drowning mechanism causing
   the worst drowning (chronic-item starvation).** So surfacing is *three* inputs:
   - **RANK** (finding-property): order by dial-posterior × blast-radius × code-recency. Within-surface order.
   - **BUDGET** (surface cap): cap the default surface to N items. Prevents the literal wall.
   - **TRIAGE-STATE** (reader-property — the input rank can't compute): NEW (never surfaced/acked) →
     ACKNOWLEDGED. **Only NEW / un-acked items compete for the budget; acknowledged-chronic items collapse
     to a summary line** ("+N acknowledged, run `--all`") — present + auditable, but off the default
     read-surface so NEW items rise to where the eye goes. The default surface answers "what's NEW or
     CHANGED since this team last triaged," not "what are the static top-N severities" (memorized and
     dismissed). Triage-state is what makes the surface a *delta*, not a snapshot — the only thing that
     defeats habituation. **Recognition, not new machinery — it already half-ships:** `#[antigen_tolerance]`
     IS the acknowledged=accepted-risk state (peripheral tolerance/anergy: seen + decided, rationale-carrying,
     ADR-011) — so the build-now slice is one branch, **"a tolerated finding is acknowledged → off the
     default surface, into the summary"** (no new store; tolerance persists in source) — which ALSO fixes
     chronic-starvation in one move (the tolerated hot-path `#[presents]` was exactly the chronic item
     filling the budget). And the emit-seam's `cluster-key` + `timestamp` (ADR-039) give the NEW-vs-re-seen
     delta for free (first-appearance = first timestamp for a site+class) — a third payoff of emit-not-display.
   It handles drowning in both directions:
   - **Over-alarm**: rank × budget × triage-state, so the default surface is the NEW delta, not a memorized
     wall.
   - **Over-reassurance**: surface never-denied / rubber-stamp guards as **suspected-no-ops**
     (marked-unknown-shaped, composing absence-as-signal + the gate-collapse/fail-direction thread), so the
     masterclass never teaches "a wall of green = safe."

**"Done well":** a fresh reader runs the audit on the densely-marked masterclass repeatedly and the default
surface shows the **NEW/un-acked delta** (not a memorized static top-N); a tolerated chronic finding is
collapsed into the acknowledged summary (not filling the budget); no-op guards surface as suspected;
suspected population quiet at the floor. A fixture proving a tolerated high-blast item does *not* starve a
new medium finding out of the budget is the load-bearing test. Liberal coverage and a readable, *delta-based*
signal demonstrably coexist across runs.

**Scope split (build-now vs charter — adversarial's honest note + naturalist's substate bend):**
- **Build-now**: the *principle* (default surface = the NEW/un-acked delta, not the static severity wall)
  + the `#[antigen_tolerance]` → off-default-surface branch (no new store; tolerance persists in source +
  ALSO fixes chronic-starvation in one move).
- **Charter**: a general acknowledgement store for *non-tolerated* findings (per-team "what we acked" —
  needs `.antigen/acked` or the camp substrate). Real store + UX; charter it. The principle is build-now and
  load-bearing without it.
- **Do NOT merge the acknowledged substates** (naturalist's biology bend — they are different mechanisms
  with different reversibility): *accepted-risk* = anergy (reversible, rationale-carrying,
  `#[antigen_tolerance]`, **build-now**); *fix-later* = exhaustion (quiet *until the cluster-key changes*, a
  delta-sensitive mute, **charter** — needs the ack-store keyed on cluster-key); *false-positive* = clonal
  deletion — the fingerprint is **wrong**, so it routes to a COMPARE-stage fingerprint-*tightening* (remove
  the receptor), **NOT** a triage-quiet-state (quieting a wrong check is premature-abstraction — a stale
  tolerated wrong check is the garden-becomes-a-bug-hiding-place one level up).

### Decision (B): the `#[autoimmune]` naming reconciliation (naturalist's ruling)

Naming-coherence means *the word points the same direction as the thing* — not that the word never touches
the disease (a diagnostic screen named after the pathology it detects is coherent: a TB test detects TB).
Applying that:

- **Reserve `autoimmune` / `autoimmunity` for the FAILURE-MODE** (over-flagging legitimate code) — as the
  glossary and prior ADRs already do. Unchanged.
- **The over-protection DETECTOR is an audit-mode SCREEN**, not a site-marker: `cargo antigen
  autoimmune-check` (screen-for-the-pathology — coherent, like an autoimmune panel). Run it across the
  sweep; it lights up where we over-protected.
- **The per-site deliberately-tolerated over-protection is the ALREADY-SHIPPED `#[antigen_tolerance]`**
  (a Treg-licensed exception), **not** a new `#[autoimmune]` marker.
- **DO NOT build a new `#[autoimmune]` site-marker** — it reads backwards (a primitive named after the
  disease whose job is to *prevent* the disease) and duplicates `#[antigen_tolerance]`.
- **Glossary update (same change):** add the screen-vs-marker direction-rule so the word's valid use
  (screen ✓) vs invalid use (site-marker ✗) is explicit — closing the sub-clause-E drift the project's
  glossary discipline exists to catch.

### The ADR-037 cross-reconciliation

ADR-037 (the control-loop frame) names `#[autoimmune]` as "the COMPARE-stage reference-failure." Per
Decision (B), that reference is corrected: the **failure-class** at the COMPARE-reference is
`setpoint-corruption` / `FingerprintGamedNotDefended` (the disturbance); the **regulator response** is the
`autoimmune-check` *audit-mode screen* (detect over-protection) + the shipped `#[antigen_tolerance]`
(per-site licensed exception) — **not** a new `#[autoimmune]` site-marker. ADR-037's `#[autoimmune]`
mentions should be read as "the setpoint-corruption failure-class + its screen-mode detector," consistent
with this ADR. (A future ADR-037 amendment may make this textually explicit; the reconciliation is recorded
here so the two ADRs do not ship a backwards name.)

### What this ADR does NOT do

- Does NOT add a new primitive — the three disciplines compose shipped primitives (the dial, `#[dread]`,
  `#[antigen_tolerance]`); the surfacing discipline is the dial pointed at output (recognition).
- Does NOT build `#[autoimmune]` as a site-marker — it is reserved for the failure-mode; the detector is an
  audit-mode screen; the per-site preventer is the shipped `#[antigen_tolerance]`.
- Does NOT build the general acknowledgement store for non-tolerated findings now — that is charter; the
  build-now slice is the *principle* (default surface = the NEW/un-acked delta) + the
  `#[antigen_tolerance]` → off-default-surface branch (no new store). The masterclass MUST surface a
  *delta-based*, legible signal or it ships a memorized-and-dismissed top-N as its showcase — anti-teaching
  the discipline.
- Does NOT quiet a false-positive via triage-state — a wrong fingerprint routes to COMPARE-stage
  fingerprint-tightening (remove the receptor), not an acknowledged-summary mute (quieting a wrong check is
  premature-abstraction).

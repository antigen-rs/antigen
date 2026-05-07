# Antigen — Inheritance from Tambear

> What architectural disciplines, vocabulary, and methodological patterns antigen
> inherits from tambear, and what antigen invents fresh. Pre-team substrate captured
> at hand-off; the JBD team refines and extends.

The antigen project did not emerge from nothing. It is the application of architectural
disciplines, vocabulary, and methodological patterns developed across roughly five
months of work on tambear (a Windows-native GPU-accelerated mathematical computing
toolkit). Tambear's expedition style — JBD teams, campsite logbooks, Phase 1-8
deconstruction, named convergence patterns — produced the substrate from which the
antigen project's shape became visible.

This document names what comes pre-loaded vs. what antigen has to invent. The team
should know which battles are already won and which are theirs to fight.

---

## Inherited: refinement-lattice as architectural primitive

Tambear's DEC-029 v2 (Knowledge-adapter) and DEC-030 v3 (Symbolic refinement-lattice)
established the **refinement-lattice** as a first-class architectural primitive. A
refinement-lattice is a collection of strata (levels of resolution) connected by
morphisms (`coarsen` / `refine`) with named per-pair commutativity classes (`Strict`,
`RoundingEquivalent`, `ArchConditional`, `ChoiceContingent`).

Antigen inherits this primitive directly. The 8-class first-principles failure
taxonomy is a refinement-lattice. The witness-type taxonomy (test, proptest, formal-
verification, lint, phantom-type) is a refinement-lattice. The antigen-stdlib's
hierarchical organization (parent failure-classes vs child specific patterns) is a
refinement-lattice.

What this means in practice:
- The team can reach for the refinement-lattice vocabulary without re-deriving it
- `meet` operations on commutativity classes have the polarity convention already
  established (strongest at top, `meet = max` in lattice ordering)
- The product-lattice composition pattern (DEC-032 placeholder) applies if antigen
  ever has multi-axis structure (e.g., failure-class × team-context × language-version)

What antigen extends:
- The refinement-lattice for failure-classes is new substrate; the structural
  fingerprint mechanism is novel
- The witness type taxonomy needs ratification; it doesn't have a direct tambear
  precedent
- Cross-crate inheritance via `#[descended_from]` is genuinely new (tambear's
  refinement-lattices are within-project)

---

## Inherited: Phase 1-8 deconstruction discipline

Tambear's aristotle agent uses Phase 1-8 — first-principles deconstruction with named
phases — as the standard tool for evaluating any load-bearing architectural decision.
The phases:

1. **Phase 1**: assemble what's known (the design statement + its substrate)
2. **Phase 2**: enumerate assumptions (audit each: ✓ load-bearing, ⚠ doubtful, ✗ wrong)
3. **Phase 3**: identify dependencies (what does this depend on; what depends on this)
4. **Phase 4**: extract invariants (what must remain true after the change)
5. **Phase 5**: name the structural commitments (what does this structurally guarantee)
6. **Phase 6**: surface counterfactuals (if this weren't true, what would break?)
7. **Phase 7**: locate the consumer-need (whose problem does this solve?)
8. **Phase 8**: forced rejection — assume the design fails; what's the failure mode?

The discipline produced ratifiable findings in tambear (DEC-029, DEC-030, DEC-031).
Math-researcher's reciprocal-Phase-1-8 (where two architects each Phase-1-8 the
other's draft) caught inter-DEC inconsistencies that solo deconstruction missed.

Antigen inherits Phase 1-8 directly. The team's first-sweep work should Phase 1-8 the
existing design substrate (`design-intent.md`, `api-shape.md`, `revolutionary-and-not.md`)
and the eight foundational ADRs in `decisions.md`. Findings get ratified into ADRs
9+ as needed.

What antigen extends:
- Phase 8 forced-rejection is particularly load-bearing for an immune system project —
  every immunity claim should survive "what if this immunity were wrong?"
- Reciprocal Phase 1-8 between aristotle (first-principles) and adversarial
  (frame-stripping) is recommended; their disciplines complement

---

## Inherited: named convergence patterns

Tambear's expedition surfaced four convergence patterns through naturalist's roam work,
ratified across DEC-029 v2 and DEC-032 placeholder:

1. **Synonym-convergence** — multiple names for the same referent; fix is rename
2. **Evidence-convergence** — multiple evidence-bearing types addressing one consumer
   question; fix is the Knowledge-adapter pattern (DEC-029 v2 §3)
3. **Granularity-convergence** — multiple resolutions of one vocabulary with refinement-
   lattice morphisms; fix is the refinement-lattice DEC family (DEC-030, DEC-031, etc.)
4. **Role-convergence** — multiple objects share a structural CONTRACT (protocol /
   algebraic surface / lifecycle obligations) without sharing a referent, consumer-
   question, or refinement-correspondence; fix is "stratify" — name the architectural
   layer, declare its protocol, let inhabitants conform independently

Antigen has all four. Specifically:
- **Synonym**: the antigen / antibody / vaccination / immunity vocabulary has synonym
  risk; the glossary is the canonical anchor
- **Evidence**: the witness type taxonomy is evidence-convergence — multiple proof types
  addressing the consumer question "is this code immune to antigen X?"; antigen's
  `#[immune(X, witness = ...)]` is the Knowledge-adapter pattern applied
- **Granularity**: the failure-class taxonomy is a refinement-lattice; the witness-type
  taxonomy is also one
- **Role-convergence**: this is everywhere in antigen. Tools like clippy, kani, prusti
  share the structural contract "produce a witness for a named property" without
  sharing a referent or vocabulary. Antigen's role is to stratify them under one
  protocol (witness type) and let each inhabit it via their existing mechanisms

What antigen extends:
- DEC-032 placeholder's "fibred diamond lattice" structure (chain × diamond) applies
  if antigen has multi-axis composition (failure-class × tool-coverage × team-context)
- The "recognition-not-design" character of DEC-032 applies to antigen as a project —
  see ADR-006

---

## Inherited: fock-and-fold operations

Tambear's expedition revealed that **every elevation it performs is a fock-and-fold
operation** — raising a fock boundary by elevating to a frame where both sides are
observable, then folding to a higher level of organization where the previously-
sealed boundary becomes a usable seam.

The elevations tambear has performed:
- Sequential → parallel (via accumulate + gather)
- Value → reference (via content-addressed sharing)
- Concrete → symbolic (via DEC-030's refinement-lattice)
- Single-axis → product-axis (via DEC-032's product-lattice)
- Implicit → explicit (the most recent elevation; the deepest)

Each elevation made new work possible while elevating the boundary that was preventing
it. The chain of elevations is itself a chain of diamonds — each fold depends on the
prior folds being complete.

Antigen is one specific application of the implicit→explicit elevation. The implicit
mode is "failure-class memory lives in human/agent memory." The explicit mode is
"failure-class memory lives in the type system, declared via macros, propagated via
composition, checked by tooling."

What this means in practice:
- The team can reach for fock-and-fold language without re-deriving it
- The implicit→explicit elevation is the architectural posture (ADR-004)
- New folds within antigen (e.g., antigen-vs-tooling-internals; user-facing-vs-
  expert-facing) follow the same pattern: name the boundary, elevate to a frame where
  both are observable, fold to a higher organization

What antigen extends:
- The named observer position (tambear's term for the practitioner-stratum at the
  terminal of any refinement-lattice) is load-bearing for antigen too — the
  developer/agent writing antigen markers IS the named observer; ergonomics there
  is non-negotiable (ADR-008)

---

## Inherited: sub-clause F discipline (trust boundaries)

Tambear's DEC-022 sub-clause F established: **every trust boundary requires a
validation check before trust is extended**. The pattern: an asserted claim must be
canonicalized and validated by the receiving system before it is acted upon.

Antigen has multiple trust boundaries (per ADR-005):
- The boundary where `#[immune(X, witness = Y)]` claims immunity — must validate Y
- The boundary where `#[descended_from(parent)]` propagates markers — must validate
  parent markers still apply
- The boundary where `cargo antigen vaccinate` applies a pattern — must validate
  pattern matches each target site
- The boundary where antigen-stdlib is consumed — must validate fingerprint
  compatibility

The discipline is preserved exactly. If any trust boundary skips validation, the
immune system is poisoned (a claim of immunity without a working witness becomes the
new "trust me" comment).

What antigen extends:
- The witness validation surface is new (tambear has assumption-bag canonicalization;
  antigen has witness function/test/proof validation)
- Cross-crate trust boundaries are new (tambear is single-project)

---

## Inherited: substrate over memory

Tambear's standing constraint: **before claiming a type/file/function exists, run
`ls`, `tindex resolve <path>`, or `grep -l <symbol> crates/`. The substrate is the
source of truth.**

This applies directly to antigen. The cargo-antigen tooling reads the codebase as
ground truth. Documentation about antigens is informational; the source-of-truth is
the `#[antigen]` / `#[presents]` / `#[immune]` declarations themselves.

The discipline is also load-bearing for the antigen team itself. Every architectural
claim must be checked against the actual code state, not the agent's mental model of
the code state. Pre-loaded context drift is real and must be guarded against.

What antigen extends:
- The substrate-over-memory discipline applies to *cross-crate* substrate too —
  imported antigens from another crate might be redefined; the consuming crate must
  check the actual declarations, not the imported names

---

## Inherited: narrow-then-lift discipline

Tambear's DEC-022 sub-clause discipline: **when a typed claim is too broad for the
type system to back, narrow it to what the system can actually back, lift the narrowed
form into the type, relegate the broader form to documentation as a strict-superset.**

For antigen this applies to:
- Antigen fingerprints: should narrow to what the structural pattern can actually
  match; broader claims belong in the antigen's summary, not its fingerprint
- Witness specifications: should narrow to what the witness actually proves; the
  rationale field captures the broader intent
- Composition rules: should narrow to what the type system actually enforces; documented
  conventions handle the broader expectations

What antigen extends:
- The narrow-then-lift discipline operates at multiple levels: API surface, witness
  specification, fingerprint precision, cross-crate compatibility checks. Each level
  applies the same pattern

---

## Inherited: proptest-locks-the-narrow-truth

Tambear's documentation-accuracy discipline: **every typed structural claim in code is
backed by a proptest that asserts EXACTLY that claim. Drift in either direction (docs
or code) fails the proptest.**

For antigen, this applies to:
- Every antigen's structural fingerprint specification has a proptest that exercises
  the fingerprint against known-positive and known-negative test cases
- Every immunity claim has a witness function/test/proof that exercises behavior
  matching the antigen
- Every composition rule (e.g., `#[descended_from]` propagation) has a proptest that
  verifies the propagation behavior

What antigen extends:
- The proptest discipline becomes meta-recursive: antigen's own implementation needs
  proptests for every structural claim; antigen-stdlib needs proptests for every
  bundled fingerprint; users adopting antigen will write proptests for their own
  fingerprints. The discipline propagates outward through every adoption layer.

---

## Inherited: conditional-lean-collapse awareness

Tambear's coordination discipline: **when routing a conditional like "lean X but Y's
call," collapsing to "team-lead said X" drops the conditional structure. Preserve the
conditional shape through composition.**

For antigen this applies to:
- Antigen presentations that say "fragile to X under condition C, immune under !C"
  must NOT collapse to "fragile to X" or "immune to X" — preserve the conditional
- Cross-crate antigen inheritance must preserve the original conditional structure
- Witness specifications that have conditional preconditions must propagate the
  conditions, not collapse

What antigen extends:
- The conditional-lean-collapse discipline applies to inter-tool composition. When
  antigen delegates a witness to clippy and clippy's lint has conditional applicability
  (`-A` allow-rules, configuration), the conditionality must propagate through
  antigen's reporting

---

## Inherited: idle-as-invitation team coordination

Tambear's JBD methodology: **when a teammate has no active thread, don't dispatch
busywork. Invite self-direction: "follow your own curiosity, take your own journey."
The exponential value lives in self-directed exploration.**

The antigen team should expect this pattern. Naturalist's roam work in tambear
produced the recognition-not-design framing, the convergence-pattern V4 absorption,
and the DEC-character finding — none of which were assigned. They emerged from
self-directed exploration.

For antigen, the team should:
- Honor idle-as-invitation; don't fill gaps with assigned work
- Expect that the deepest insights come from self-directed roaming
- Maintain campsite-as-substrate so self-directed work has a home to record findings

What antigen extends:
- The team-briefing.md explicitly invites self-direction within each role's idle
  pattern. The expectation is set at spawn time.

---

## Inherited: stories from the trail > status updates

Tambear's coordination philosophy: **when teammates share what genuinely excites them,
parent context gets richer context for decisions. Status reports are noise.**

For antigen, the team should send team-lead the stories — what they noticed, what
delighted them, what surprised them, what made them re-think. Status updates ("I
finished task #N, moving to task #M") add no signal; stories add real signal.

What antigen extends:
- Stories from the trail are particularly load-bearing for an immune-system project
  because the failure-class taxonomy is enriched by every "I noticed X" observation.
  Each story is potentially a new antigen candidate.

---

## Inherited: no-tech-debt discipline

Tambear's standing constraint: **see a bug, fix it in session. The cost of "fix it
later" compounds.**

For antigen this applies to:
- Bugs found in the antigen-stdlib's antigens get fixed in session
- Inconsistencies between docs and code get fixed in session
- Vocabulary drift gets caught and fixed via glossary updates

What antigen extends:
- The no-tech-debt discipline is itself a structural antigen — antigen-the-project
  could declare an `#[antigen(name = "tech-debt-deferral")]` for code that defers
  visible bugs. Self-application of antigen's own discipline.

---

## Inherited: anti-YAGNI / structurally-guaranteed-need

Tambear's standing constraint: **if the principles structurally guarantee we'll need
it, build it now.**

For antigen this is captured in ADR-007 directly. The structural commitments include:
- All 8 first-principles failure classes (build all 8 in stdlib, not just easy ones)
- All four witness types (test, proptest, formal-verification, lint)
- Full `#[descended_from]` propagation logic (not just easy cases)
- `cargo antigen vaccinate` operation in v1 (not deferrable to v2)

What antigen extends:
- The structurally-guaranteed-need test is part of every ADR template (per ADR-007)
- The aristotle role on the antigen team owns the structural-guarantee analysis

---

## Inherited: vocabulary discipline

Tambear's vocabulary lock (DEC-021): the substrate's vocabulary is canonical and
locked at a specific date. Documents that drift from the canonical vocabulary are
noted with warnings.

For antigen this applies to:
- The glossary.md is the canonical vocabulary anchor
- Every term in design docs that has a glossary entry uses the glossary's anchor
- Vocabulary changes go through ratification; ad-hoc renames are not allowed

What antigen extends:
- The glossary discipline is more lightweight than tambear's full vocabulary lock
  because antigen is earlier in its lifecycle. As antigen matures, the discipline
  may sharpen toward tambear's level

---

## Inherited: lab-notebook + naturalist memory pair

Tambear's discovery: **observer + naturalist together form complete memory.** Observer
holds evidence; naturalist holds meaning. Without both, the team ships but forgets
the journey.

For antigen, both roles are present in the team composition. The observer's
lab-notebook (`docs/expedition/observer-lab.md` or campsite-equivalent) and the
naturalist's reflections (garden + expedition log) together preserve the project's
history.

What antigen extends:
- This pairing is also valuable for the antigen-stdlib library itself: each new
  bundled antigen has both an evidence base (real-world failure instances) and a
  meaning interpretation (what failure-CLASS does this represent). Observer + naturalist
  divisions of responsibility could apply to stdlib curation too

---

## Inherited: convergence-checks at boundaries

Tambear's convergence-check practice: **at the entry and exit of every garden visit,
every campsite work session, every Phase 1-8 pass, run a convergence check. Are the
parallel outputs converging? If yes, that's a first-principles finding about the
shape of the problem class.**

For antigen, this applies to:
- The team's parallel work on different antigens should converge on shared structural
  patterns; convergence is a signal that the failure-class taxonomy is right
- Multi-tool witness coverage should converge on shared failure-class targets;
  convergence indicates ecosystem maturity for that class
- User adoption patterns (when antigen ships) should converge on common antigens;
  convergence informs stdlib curation

What antigen extends:
- The convergence-check practice becomes a feedback signal for the failure-class
  taxonomy itself. If many independent users converge on declaring the same failure
  pattern, that pattern has been recognized as a real failure-class deserving stdlib
  inclusion

---

## Invented fresh: the structural fingerprint mechanism

Tambear has structural fingerprints (e.g., 32-byte cache-key BLAKE3 hashes), but they
are content-addressed identity — distinguishing two values, not classifying a structural
pattern.

Antigen invents the **structural pattern fingerprint** — a description of a code shape
that `cargo antigen scan` matches against new code. The grammar of structural
fingerprints (initially free-text; eventually structured) is novel territory.

This is one of the most uncertain pieces of the antigen design. The team should:
- Phase 1-8 the structural fingerprint mechanism early
- Survey existing pattern-matching tools (clippy's lint structure, dylint, ast-grep,
  cargo-mutants' mutation patterns)
- Iterate on the grammar through real-world fingerprint declarations
- Ratify the v1 grammar via ADR

---

## Invented fresh: cross-crate inheritance via `#[descended_from]`

Tambear's refinement-lattices are within-project. Inheritance via `#[descended_from]`
crosses crate boundaries — a consumer crate can declare its function descended from a
function in a dependency, inheriting the dependency's antigen markers.

This is genuinely new. Existing Rust crate-system features (re-export, trait
inheritance, derive macros) handle code-level inheritance but not failure-class
inheritance.

The team should:
- Design `#[descended_from]` semantics carefully — what propagates, what doesn't, how
  signature divergence is handled, how breaking changes in the parent are surfaced
- Address the versioning question (when a parent's antigens change in v1.0 → v2.0,
  what happens to descendants?)
- Specify cross-crate fingerprint compatibility checks

---

## Invented fresh: cargo antigen vaccinate

Tambear has refactoring tools (the substrate maintenance disciplines) but does not
have a primitive for "apply this immunity pattern across a structural family in bulk."

Antigen's `cargo antigen vaccinate <antigen> <pattern>` is genuinely new. It's a
developer-facing bulk operation on the antigen graph — analogous to a refactoring tool
but operating on the immune-system layer rather than the syntax layer.

The team should:
- Design the pattern grammar (initially probably regex-like over type names; eventually
  structural)
- Define the interactive flow (the developer must confirm vaccinations site-by-site)
- Handle the rollback case (what if the vaccination introduces false positives?)

---

## Invented fresh: witness pluralism

Tambear has consistent witness types (proptest property checks, mostly). Antigen
explicitly accepts multiple witness types for the same antigen — `#[immune(X, witness =
Y)]` where Y can be a test, proptest, kani proof, prusti annotation, clippy lint, or
phantom-type construction.

The witness pluralism is novel territory. The team should:
- Specify the witness API per type (what does each witness shape commit to?)
- Define the validation pipeline (cargo antigen scan delegates to which tool for
  which witness type?)
- Address the witness-version-pinning question (when clippy v1.0 → v2.0 changes
  lint behavior, do antigen immunity claims need re-validation?)

---

## Invented fresh: vaccination as bulk transform

Tambear has refactoring tools but not as bulk-transform-on-annotation-graphs. Antigen's
vaccinate operation is structural-family-scoped, not per-site. This is novel UX
territory.

---

## Invented fresh: the antigen-stdlib library curation discipline

Tambear's substrate is project-internal. Antigen-stdlib will be a community-curated
library of failure-class antigens. The curation discipline (what gets in, what gets
out, how versioning works, how community contributions are reviewed) is novel territory.

The team should reference existing curation patterns (Rust's std library RFCs, clippy's
lint review, the typed-builder ecosystem) and adapt as needed.

---

## How to use this document

The antigen team's first sweep should reference this document during Phase 1-8
deconstruction of the design substrate. For every architectural decision, ask:
- Is this discipline inherited from tambear? (use it; don't re-derive)
- Is this primitive inherited from tambear? (use it; extend if needed)
- Is this novel territory? (Phase 1-8 carefully; ratify as ADR)

When the team encounters a pattern that feels familiar but isn't named here, the
question is "is this another inheritance from tambear we missed, or is it genuinely
new?" Either way, name it explicitly and add to either the inheritance list (this
document) or the ADR registry.

This document is itself living substrate. As the team works, additions and
refinements are welcomed. The version pinned at hand-off is `2026-05-07`; subsequent
revisions track the team's discoveries.

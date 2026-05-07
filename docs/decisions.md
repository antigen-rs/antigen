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

---

## Index

- [ADR-001 — Failure-class memory is structural, not documentary](#adr-001--failure-class-memory-is-structural-not-documentary)
- [ADR-002 — Compose, don't compete](#adr-002--compose-dont-compete)
- [ADR-003 — Biological metaphor is load-bearing, not decorative](#adr-003--biological-metaphor-is-load-bearing-not-decorative)
- [ADR-004 — Implicit-to-explicit elevation as architectural posture](#adr-004--implicit-to-explicit-elevation-as-architectural-posture)
- [ADR-005 — Sub-clause F at every trust boundary](#adr-005--sub-clause-f-at-every-trust-boundary)
- [ADR-006 — Recognition, not design](#adr-006--recognition-not-design)
- [ADR-007 — Anti-YAGNI: structurally-guaranteed need](#adr-007--anti-yagni-structurally-guaranteed-need)
- [ADR-008 — Named-observer position as terminal stratum](#adr-008--named-observer-position-as-terminal-stratum)

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
- Documentation drift as a memory carrier
- AI-coding-agent context-loss across sessions

---

## [ADR-002] Compose, don't compete

**Status**: Ratified 2026-05-07 (foundational; pre-team).

**Participants**: Tekgy + Claude.

**Related**: ADR-001 (memory mechanism), ADR-004 (elevation posture).

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

**Related**: ADR-006 (recognition).

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

**Related**: ADR-003 (metaphor), ADR-004 (elevation).

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

**Related**: ADR-006 (recognition), ADR-002 (composition).

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

**Related**: ADR-004 (elevation), ADR-006 (recognition).

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

1. Number sequentially. Skip numbers only with explicit reservation.
2. Use the section template above (Status, Participants, Related, Finding, Decision,
   Mechanics, Sweep-level consequences, Enforcement, Resolves).
3. Update the index at the top of this file.
4. Reference the ADR in any related code or other docs that act on its decisions.
5. Update `docs/glossary.md` if the ADR introduces new vocabulary.

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

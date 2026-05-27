# Antigen — Process

> The formal process by which architectural decisions, sweeps, and code in the antigen
> project get drafted, validated, ratified, and govern downstream work. Inherited from
> the tambear DEC discipline; adapted for antigen.
>
> **Audience**: this doc is internal-team-facing. It documents how the antigen team
> coordinates the ADR lifecycle (campsite-based working files, Phase 1-8 deconstruction,
> aristotle / adversarial / scientist review roles, ratification ceremonies). External
> contributors don't need this — see [`../CONTRIBUTING.md`](../CONTRIBUTING.md) for the
> contributor-facing surface. When you submit a PR, the antigen team takes it through
> the discipline described here; you don't have to set up campsites or run the phases
> yourself.

## The recursion: ADRs are antigen-in-document-form

Before describing the mechanics: notice the meta-recursion that makes this process
right for *this* project specifically.

Antigen-the-tool exists because failure-class memory should live in the substrate
rather than in commits, comments, and human memory. It declares failure-classes
(`#[antigen]`), marks vulnerable code (`#[presents]`), proves immunity with witnesses
(`#[immune]`), and propagates lessons through composition (`#[descended_from]`).

Tambear's DECs (and antigen's ADRs) operate the SAME shape one level up. They:
- **Declare** an architectural decision with a name and a fact-pattern (the finding)
- **Prove** it through Phase 1-8 deconstruction (the witness)
- **Ratify** it (the stamping that locks the decision)
- **Govern** downstream work via cross-references and enforcement clauses
- **Propagate** through amendments and inheritance (later ADRs cite earlier ADRs;
  amendments preserve original ratification with explicit deltas)
- **Stay structural** — drift is detected because new code references the ADR, and
  changes either follow the ADR or trigger an amendment

This is not metaphor. ADRs are antigen-in-document-form. They were the **original
implementation** of the structural-memory pattern, before we noticed the pattern, in
the substrate of project-architecture rather than the substrate of code. Tambear's
DEC discipline was the fertile ground from which the antigen-tool insight emerged.

So when antigen-the-team draws on this process, it's not borrowing arbitrary
practices. It's using the same architectural pattern at the meta-level that the tool
will operate at the code level. Recursion through.

---

## Lifecycle of an ADR

### Stage 1: Draft

An ADR begins as a draft when an architectural question surfaces that requires
ratification. The triggers:

- A design decision is being made informally and the team-lead recognizes the need to
  formalize
- A pattern recurs across multiple recent commits and wants naming
- An open question in a prior ADR or design document needs answering
- The naturalist surfaces a convergence-pattern observation that needs ratification
- Aristotle's Phase 1-8 on existing work surfaces a load-bearing assumption that
  hasn't been ratified

Drafting:
- Author opens a campsite under `campsites/adr-NNN-<slug>` (replace NNN with the next
  ADR number)
- Author writes the draft to a working file in the campsite
- Status: **Draft**
- Index in `decisions.md` is NOT updated yet (drafts don't go in the index until
  ratified)

Drafting can be fast (hours) or slow (days/weeks). The draft document includes:
1. **Status**: Draft (date)
2. **Participants**: who's working on this
3. **Related**: prior ADRs this draft depends on or affects
4. **Finding**: what observation prompts this ADR
5. **Decision (proposed)**: the architectural commitment being proposed
6. **Mechanics**: how the decision works in practice
7. **Sweep-level consequences**: what work this commits us to
8. **Enforcement**: how the decision is checked
9. **Resolves**: what existing problems this addresses
10. **Open questions**: anything not yet settled (will be resolved before ratification)

### Stage 2: Phase 1-8 deconstruction

Once a draft is ready for review, the aristotle role runs Phase 1-8 deconstruction.
This is the **witness** for the ADR — the proof that the decision is sound and
load-bearing.

The phases:

1. **Phase 1 — Assemble**: collect what's known. The draft + related ADRs + relevant
   substrate documents + any prior conversations or campsites that produced the
   draft's substance.

2. **Phase 2 — Audit assumptions**: enumerate every assumption the draft makes. For
   each, mark `✓ load-bearing`, `⚠ doubtful`, or `✗ wrong`. Doubtful and wrong
   assumptions become edits to the draft.

3. **Phase 3 — Map dependencies**: identify what this draft depends on (other ADRs,
   substrate documents, ecosystem assumptions) and what depends on it (downstream
   sweeps, future ADRs, code surfaces).

   **Phase 3 sub-routine — Cross-ADR surface check**: for every named surface this ADR
   introduces (CLI verbs, type names, sidecar keys, serde tags, audit hints, predicate
   leaf names), substrate-grep all ratified and in-flight ADR drafts for that name before
   §Mechanics-finalize. Surface collisions for explicit resolution before the draft
   advances. Collisions caught at draft-time are zero-cost; collisions caught at
   implementation-time require multi-ADR coordination + caller migration.

   This is sub-clause F (ADR-005) applied at the cross-ADR level: named surfaces are
   trust-boundaries that consumers of ratified ADRs rely on; a naming collision corrupts
   the consumer-trust-extension chain silently until implementation surfaces it.

   Empirical basis: F28 (aristotle) §Strip-E caught the `attest oracle complete` /
   `oracle complete` verb collision only because Strip-E compared the new oracle CLI
   surface against the existing `attest` CLI surface from ADR-019 §M4. Without the
   comparison, both verbs would have shipped and required caller migration.

   **Phase 3 sub-routine — Enforcement-Mechanism specification** (ratified 2026-05-22):

   For every gate, check, or validation that the ADR introduces, the draft MUST specify:

   1. **Enforcement-Tier** — at which tier the check happens:
      - **parse-time**: macro expansion / compile-time / build script — typically produces a compile error
      - **scan-time**: `cargo antigen scan` walks substrate; non-blocking by default but can be CI-gated
      - **audit-time**: `cargo antigen audit` evaluates witness predicates; CI-gated
      - **CLI-time**: explicit operator invocation of a `cargo antigen ...` subcommand; user-initiated
      - **commit-time**: pre-commit / post-commit hooks; runs at git operation
      - **push-time**: pre-push / pre-receive hooks; runs at git push or remote ref update
      - **build-time**: Cargo feature resolution; compile-time conditional inclusion
      - **runtime**: per-execution check; lowest-friction, lowest-discipline tier
      - **NONE (named limitation)**: explicitly NOT enforced; relies on cultural / documentation discipline

   2. **Enforcement-Scope** — where the check runs:
      - **client**: runs on developer's machine; bypassable by configuration drift or deliberate skip
      - **server**: runs on git remote / CI server / centralized authority; not bypassable by client config
      - **client + CI**: both client (developer feedback) and CI (gate)
      - **process**: enforced via ADR-Phase-1-8 and ratification ceremony; meta-level

   3. **Bypass risk + mitigation** — for each mechanism, name the realistic bypass path and its mitigation.

   This information lives in a **§Enforcement-Surface table** in the ADR draft, with columns: Mechanism | Enforcement-Tier | Enforcement-Scope | Bypass risk + mitigation.

   **Friction-vs-structural disclosure requirement**: when the chosen enforcement-tier is friction-only (client-side; bypassable), the ADR MUST explicitly state this. The default text:

   > This ADR enforces [X] at friction-only level by default (client-side hooks + audit-time hints). Friction-only means the discipline makes bad behavior DELIBERATE rather than ACCIDENTAL, but does NOT prevent determined bypass. Adopters requiring structural mode must [specific path].

   **Empirical basis**: cross-ADR systemic finding from v0.2 ratification team campaign (2026-05-22) — adversarial gates on 6 independent ADRs (ADR-023 through ADR-028) all surfaced the same enforcement-mechanism-ambiguity gap independently. ADRs specified WHAT should be enforced without specifying the HOW precisely enough to prevent bypass. Specifying enforcement-surface at draft-time closes the gap before adversarial review needs to discover it.

   **Relationship to Cross-ADR surface check**: complementary sub-routines. Cross-ADR surface check catches naming collisions; this sub-routine catches mechanism-ambiguity gaps. Both run before §Mechanics-finalize.

   **Phase 3 sub-routine — Standing Adversarial Checklist** (ratified 2026-05-24):

   For every primitive the ADR introduces, answer all nine questions before §Mechanics-finalize. This produces a **§Standing-Pressure-Audit table** in the ADR draft. The checklist applies regardless of whether the adversarial role has personally engaged on this draft — it makes structural adversarial pressure systematic, not contingent on adversarial volunteering.

   The empirical basis: HOW-specification depth in v0.2 drafts was coupled to whether adversarial attacks fired during ratification. ADR-024 §convergent had arg-level spec because adversarial pressed during ratification; ADR-024 §recurrent/§prescriptive shipped 15 macros with no arg-shape spec because adversarial pressure did not reach them. Even within ADR-024 §convergent — where adversarial DID engage — three primitives (#[polyclonal], #[monoclonal], #[adcc]) shipped with silently-discarding arg parsers because the pressure was not applied per-primitive. Standing checklist closes this by requiring Q1-Q9 answers for EVERY primitive, not just the ones that drew adversarial attention.

   **Q1 — Proc-macro arg-signature**: What is the parse-time arg signature? For every field: name, Rust type at parse-receive, required/optional, default, constraints, positional-vs-keyword, mutually-exclusive groups. Unknown-field errors MUST emit the full enumerated set. The ADR includes a **§Proc-Macro-Surface table** with columns: Primitive | Field | Type | Required | Default | Constraint. Each Args struct ships with a doc-comment citing the relevant ADR section as the spec authority. For primitives with intentionally open arg-sets (vocabulary-extension primitives that compose Tier-3 adopter-vocab), the ADR must: (a) label the field `open-vocab-extension` explicitly, (b) specify the validation boundary (what is rejected even from the open set), (c) name the audit hint emitted for unrecognized vocab keys, and (d) state WHY sealed enumeration is structurally impossible for this field. Without (a-d), an open arg-set is indistinguishable from an incompletely specified field.

   **Q2 — Sealed-enum inclusion-discipline**: If the primitive introduces a sealed enum, what axis does it carve up? The axis must be single and consistent — mixed-axis enums are category errors. The ADR specifies: (i) inclusion criterion — the structural property a variant must satisfy; (ii) extension discipline — when a new variant is proposed post-ratification, what criteria must it meet beyond "requires ADR amendment per ADR-001 C6"; (iii) disambiguation between near-duplicate variants — if two variants could be confused, explicitly disambiguate at variant-definition time or drop one; (iv) exclusion criterion — a structural property that disqualifies a candidate variant, named with at least one excluded candidate and explanation. Note: "negative space" (listing excluded items retroactively) does NOT satisfy this requirement; the exclusion CRITERION must be named at ratification time so it PREDICTS non-belonging, not just lists past exclusions.

   **Q3 — Controlled-vocabulary-tier-discipline**: For fields whose values are drawn from a shared taxonomy (not reference fields — those fall under Q4), the ADR specifies the vocabulary discipline: **Tier 1** = sealed values requiring ADR amendment to extend (e.g., the 8-class family= taxonomy); **Tier 2** = stdlib-shipped extensions that compose Tier 1 (per family-name registry); **Tier 3** = adopter open-vocab (accepts anything; emits adopter-uses-novel-vocab hint if not in Tier 1+2). The ADR specifies how parse-time validates the tier, what audit-time emits for each tier, and what the registry-update process is. When drafting audit-hint vocabularies, the author MUST choose one of two framings explicitly: **EXAMPLES (open set within budget)** — `~N hints; cross-ADR substrate-grep verified; examples: [partial list]`, which pre-authorizes open-set extension under (a) family-prefix discipline, (b) cross-ADR substrate-grep at implementation time confirms no collision, (c) the hint belongs semantically to the family's audit-event taxonomy; OR **STRICT ENUMERATION (closed set)** — `the following N hints:`, where every addition requires amendment.

   **Q4 — Resolution algorithm**: If the primitive has a reference field (handled_by, parent, X), what is the resolution algorithm? Specify: path-lookup vs fingerprint vs string-match, lexical scope rules, visibility constraints, disambiguation policy when multiple candidates match, failure mode (what happens when the target cannot be resolved — audit-hint emitted? compile error? silent skip?), and at which lifecycle phase the failure surfaces (parse-time compile error, scan-time warning, audit-time hint). The failure-mode phase matters: an audit-time-only failure means developers running only `cargo build` never see it.

   **Q5 — Cross-primitive interaction**: Does the primitive interact with sibling primitives (e.g., orient-vs-triage_commit, see-vs-references)? If yes, name the interaction explicitly. If the primitive extends a sibling's field-set (e.g., `extends #[orient]`), specify which fields are inherited, which are new, and whether semantics of shared field names match exactly or diverge intentionally.

   **Q6 — Deprecation surface**: If the primitive supersedes prior shape, what is the migration path? Four options: **hard-break** (compile error after deprecation); **soft-migration** (warning + auto-migration tooling); **additive** (both forms valid simultaneously with a sunset date); **semantic-break** (additive at compile-time but behavior changes for existing callers — a field addition that compiles without error but inverts semantics for code that depended on the field's absence; must be called out explicitly).

   **Q7 — Named-surface check**: Does any field name collide with sibling-ADR surfaces? (Phase 3 sub-routine 1 — Cross-ADR surface check; already required.)

   **Q8 — Enforcement-mechanism**: For every gate in this primitive, what is the §Enforcement-Surface row? (Phase 3 sub-routine 2 — Enforcement-Mechanism specification; already required.) The table must have at least one row per gate in the primitive. A gate with no enforcement mechanism is a PROCESS BLOCKER at Phase 3, not a permitted absence.

   **Q9 — Spec-adversarial pre-implementation test discipline**: For every enforcement gate in the primitive (parse-time rejection, scan-resolution failure, audit-hint emission), name the executable test that would fail if that gate were implemented in the most-permissive default. The test may be `#[ignore]`'d pending implementation, but the test body MUST encode the spec-disambiguation BEFORE pathmaker writes code. Advisory-only fields with no wrong implementation are exempt; the ADR must explicitly label the field as advisory-only to claim the exemption. Unlabeled fields cannot claim the exemption.

   **Sealed-enum inclusion-discipline (Q2 sub-clause)**: For sealed enums with post-ratification extension discipline, the criteria must be specific enough to evaluate a proposed variant without committee deliberation. Generic "requires ADR amendment" is necessary but not sufficient — the amendment-criteria themselves must be specified. Cross-ADR substrate-grep for variant-name collisions (Q7) runs before proposing any new variant.

   **Controlled-vocabulary-tier-discipline (Q3 sub-clause)**: Tier-1 sealed taxonomy fields that receive proposed new values must pass through full ADR amendment ceremony. Tier-2 stdlib-shipped extensions must pass cross-ADR substrate-grep before shipping. Tier-3 adopter-vocab fields must emit adopter-uses-novel-vocab hint on unrecognized values and must specify the audit-hint-code discipline: use typed enum reference (`AuditHint::SomeName`) rather than string literal at suppression sites; compile-fail fixtures confirm misspelled variants are rejected at parse time.

   **Amendment-vs-fixup taxonomy** (ratified 2026-05-24):

   AMENDMENT applies to: any change to ADR §Decision/§Mechanics/§Enforcement-Surface text; any biology-grounding axis reassignment; any sealed-enum variant addition/removal; any cross-axis category/inclusion-discipline change.

   FIXUP applies to: doc-comment changes; implementation drift cleanup; parse-time check additions; test additions; example file refactors; CHANGELOG updates — anything that does NOT modify ADR text. Fixup form: campsite trail (camp note sequences) + commit message + code change. No ADR-text amendment block needed.

   AMENDMENT FORM — two types based on how the change is specified:

   **Type A (design-first)**: change is fully specified at campsite-open. Ship the structured amendment block and the inline §Decision/§Mechanics text change in a single commit. Canonical form: draft the structured block in docs/expedition/<adr>-amendment-<N>-<slug>.md before the campsite cycle, then commit both inline change and structured block together.

   **Type B (review-first)**: change emerges from campsite discussion. Ship the inline §Decision/§Mechanics change first for speed. Ship the structured amendment block as a follow-on commit AFTER campsite notes settle. Structured block must land before sign-off. No sign-off on a review-first amendment until the structured block exists.

   In both cases the structured ## ADR-N Amendment N block contains: Status / Amends / Reason / Change / Resolves / What-this-amendment-does-NOT-do sections. The block is appended before the parent ADR's final `---` divider.

4. **Phase 4 — Extract invariants**: name what must remain true after the change.
   These become enforcement clauses if not already explicit.

5. **Phase 5 — Structural commitments**: name what this ADR structurally guarantees.
   These commitments inform anti-YAGNI judgments downstream.

6. **Phase 6 — Surface counterfactuals**: ask "if this weren't true, what would
   break?" Answers reveal which parts of the design are load-bearing vs nice-to-have.

7. **Phase 7 — Locate consumer-need**: whose problem does this solve? What use case
   demands this ratification right now?

8. **Phase 8 — Forced rejection**: assume the design fails. What's the failure mode?
   How would we know? What's the recovery path? This phase often surfaces missed
   considerations and produces "ATK-N" attack annotations on the draft.

The output of Phase 1-8 is a deconstruction document attached to the campsite. It's
how aristotle (and others) certify that the draft has been examined first-principles.

**Reciprocal Phase 1-8** (when applicable): if the ADR is closely related to a peer
draft (e.g., DEC-029 and DEC-030 in tambear's expedition), each agent Phase-1-8s the
other's draft. This catches inter-ADR inconsistencies that solo deconstruction misses.

### Stage 3: Adversarial review

Adversarial role pressure-tests the post-Phase-1-8 draft. Designs degenerate inputs,
hunts for silent failures, writes failing-as-passing tests against the proposed
decision. Output: ATK-N annotations naming specific attacks the draft must address
before ratification.

Adversarial findings become refinements to the draft. The draft is iterated until
all ATK-N findings are either addressed or explicitly accepted as out-of-scope (with
rationale).

### Stage 4: Math/systems-research review

Math-researcher (in math-mode for tambear) or systems-researcher (in
systems-research-mode for antigen) reviews the draft for technical correctness:

- Does the decision align with prior art (academic, ecosystem, or upstream spec)?
- Are the mechanics implementable in the named technical environment?
- Are the cited references accurate and load-bearing?

Output: a review document attached to the campsite. May surface refinements that
become draft edits.

### Stage 5: Scientist validation

Scientist role validates the draft against substrate:

- Does the draft match what the codebase actually does (or will do)?
- Are the resolves-clauses accurate (does this ADR actually resolve the named
  problems)?
- Are the enforcement clauses implementable?

Output: validation pass/fail. Failed validation returns the draft to the appropriate
prior stage.

### Stage 6: Ratification

When all reviews pass, the team-lead (in tambear: Tekgy + the navigator team) reviews
the final draft for cross-cutting concerns and ratifies. Ratification is a deliberate
act:

- Status changes from **Draft** to **Ratified [date]**
- The ADR is moved from its campsite working file to `docs/decisions.md`
- The index at the top of `decisions.md` is updated
- A commit message references the ADR by number ("ratify ADR-009: <name>")
- The campsite is marked `closed` with a final log entry summarizing the ratified
  text's location

After ratification, the ADR is **locked**. Changes require either:
- An **Amendment** (additive refinement that preserves the original decision)
- A **Supersession** (a new ADR that replaces the old; old ADR's status becomes
  "Superseded by ADR-MMM")

### Stage 7: Enforcement

Ratified ADRs govern downstream work through:

1. **Cross-references** in code: `// ADR-005: this trust-boundary check enforces ...`
2. **Cross-references** in other ADRs: "Related: ADR-005" in the new ADR's header
3. **Cross-references** in sweeps: "Sweep A3 implements ADR-005's witness validation
   surface"
4. **Antigen declarations** that name the ADR they enforce: `#[antigen(name = "...",
   adr = "ADR-005")]` (proposed; can be added in v0.2+ if useful)
5. **CI gates** when applicable: `cargo antigen audit --strict` may fail on ADR
   violations
6. **Documentation cross-references**: docs that touch a ratified decision link to the
   ADR

Enforcement is the substrate that makes ratification load-bearing. Without
enforcement, ratification is just paperwork.

### Stage 8: Reference and propagation

Future work cites the ADR. Future ADRs build on it. The substrate accumulates a graph
of cross-referenced decisions where each node is a ratified architectural commitment.

The graph IS the structural memory. New work navigates the graph by reading the
ADRs and their cross-references. New team members onboard by walking the graph.

---

## Lifecycle of a Sweep

Sweeps are larger units of work that ratify or implement multiple ADRs together,
under a coherent thematic banner.

### Sweep planning

A sweep starts with a sweep-planning document at `sweeps/<sweep-name>/README.md`. The
document specifies:

- **Theme**: what coherent banner ties this sweep's work together
- **Blockers**: which prior sweeps' work-streams must complete before this can start
  (named at work-stream granularity, not just sweep granularity — see DEC-022's
  partial-dependency-granularity discipline)
- **Unlocks**: which downstream sweeps and which work-streams within them this sweep
  unblocks
- **Integration milestones**: where this sweep needs to bridge to other sweeps'
  infrastructure (per DEC-022's infrastructure-vs-integration-split discipline)
- **ADRs ratified or implemented**: which ratified ADRs this sweep operates under
- **Work-streams**: the sub-sweeps within this sweep, each with its own scope and
  blockers

Sweep planning is reviewed by the team-lead before the sweep launches. Once launched,
the sweep operates autonomously per the JBD methodology.

### Sweep execution

Within a sweep, work happens in campsites (one per work-stream). Each campsite has
an owning role and a clear scope. Cross-cutting findings get filed in the campsite
they originate from; the navigator routes findings between campsites.

Sweep execution produces:
- Code (when applicable)
- Tests (always)
- Documentation updates (rustdoc + design-substrate updates)
- ADR drafts (often surfacing during the sweep)
- ATK-N attack annotations (from adversarial)
- Garden entries (from naturalist)
- Lab notebook entries (from observer)

### Sweep closure

A sweep closes when:
- All planned work-streams complete
- All ADRs that were drafted during the sweep are either ratified or explicitly
  deferred (with rationale)
- All ATK-N findings are addressed or explicitly accepted as out-of-scope
- CI is green
- The sweep README is updated with closure notes
- The campsite logbook records the sweep close

The team-lead reviews the closure. The naturalist may write a closure narrative
naming what was learned during the sweep that should propagate beyond it.

---

## Governance: how ADRs interact with code, sweeps, and other ADRs

### ADRs governing code

Code references the ADR it implements:

```rust
/// Implements ADR-005's trust-boundary check for `ingest()`.
///
/// The implementation here MUST honor the canonicalization-before-codegen-use
/// invariant. See `docs/decisions.md#adr-005`.
pub fn ingest(...) -> Vec<Entry> { ... }
```

When the code drifts from the ADR, the cross-reference reveals the drift. Either:
- The code is wrong (fix the code)
- The ADR is wrong (amend the ADR)

The cross-reference forces the choice; without it, drift is silent.

### ADRs governing sweeps

Sweeps cite the ADRs they operate under in the sweep README's "ADRs ratified or
implemented" section. This makes the sweep's architectural commitments explicit and
auditable.

When a sweep proposes work that violates an unratified-ADR constraint, the team-lead
either ratifies the ADR-as-amendment or rejects the sweep's proposal.

### ADRs governing other ADRs

ADRs cite their related ADRs in the header. This builds a directed graph of
dependencies and refinements:

- **Depends on**: the new ADR builds on the cited ADR's decisions
- **Refines**: the new ADR adds detail to the cited ADR's general statement
- **Supersedes**: the new ADR replaces the cited ADR (which becomes "Superseded by")
- **Amended by**: the new ADR is an amendment to the cited ADR

The graph is navigable. A reader of ADR-009 follows its "Related: ADR-001, ADR-005"
links to understand the foundation. A reader of ADR-001 sees "Amended by ADR-N1, N2"
to understand the evolution.

### ADRs governing antigens (when antigen-the-tool ships)

Once antigen-the-tool ships, antigen declarations can cite the ADR they enforce:

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    adr = "ADR-NNN",  // proposed v0.2+ feature
    fingerprint = "...",
)]
pub struct PolarityInvertedClassMeet;
```

This makes the ADR-to-antigen relationship structural. An ADR that establishes a
class-design discipline can have its discipline enforced by an antigen on every class
that ships in the codebase.

The recursion completes here: ADRs are antigen-in-document-form; antigens cite
ADRs-as-document; the substrate is consistent.

---

## Team roles in the process

Each role on the antigen JBD team has process responsibilities:

| Role | Process responsibility |
|---|---|
| **pathmaker** | Implements the code that ratified ADRs commit us to. Surfaces refinement-needs that may become amendments. |
| **navigator** | Coordinates the process flow. Routes drafts through review stages. Owns the campsite logbook. Escalates to team-lead when stages stall. |
| **scout** | Surfaces prior art relevant to draft decisions. Maps the substrate that drafts depend on. Verifies cross-references in drafts and ratified ADRs. |
| **naturalist** | Notices convergence patterns that may need ratification. Writes closure narratives at sweep ends. Roams across multiple drafts looking for cross-cutting concerns. |
| **observer** | Maintains the lab notebook tracking each draft's progression through stages. Records what changes commit-to-commit during sweep execution. Holds the neutral record. |
| **math-researcher** / **systems-researcher** | Reviews drafts for technical correctness against prior art. Surfaces papers and RFCs that drafts should cite. Validates ecosystem-integration claims. |
| **adversarial** | Pressure-tests drafts post-Phase 1-8. Files ATK-N attacks. Writes failing-as-passing tests against proposed decisions. Catches silent failures before ratification. |
| **scientist** | Validates drafts against substrate. Confirms resolves-clauses are accurate. Verifies enforcement clauses are implementable. The publication-grade write-up role. |
| **aristotle** | Owns Phase 1-8 deconstruction. The first-principles review role. Surfaces hidden assumptions. Proposes refinements to drafts. The witness for the ADR. |

Each role has standing in the process. No role's work is "optional."

---

## Worked example: ADR-NNN, "Antigen fingerprint grammar"

A hypothetical future ADR for antigen, walked through the process to show the lifecycle.

### Trigger
The pathmaker starts implementing the `#[antigen(fingerprint = "...")]` parser. Initial
free-text is awkward; consumers want richer pattern grammar. Question: what should the
grammar BE?

### Stage 1: Draft
Pathmaker opens campsite `adr-009-fingerprint-grammar`. Writes a draft proposing a
mini-DSL for structural patterns: AST-shape match + type-name pattern + attribute-
presence check.

### Stage 2: Phase 1-8
Aristotle Phase 1-8s the draft.
- Phase 2 audit catches Assumption-A14: "the grammar must be embeddable in a string
  literal." Probably yes (cargo macros use string literals), but let's check. Phase
  3 dependencies: the parser must parse from a string. ✓
- Phase 5 structural commitments: the grammar commits us to AST-traversal capability
  in cargo-antigen scan. Phase 8 forced rejection: if the grammar were limited to
  type-name-only matching, what breaks? The composition-rules ADR's structural
  inheritance becomes unworkable. So full AST-shape match is structurally needed.

### Stage 3: Adversarial
Adversarial files ATK-9-1: "what if the user writes a fingerprint that matches every
function?" → autoimmunity risk. Mitigation: cargo-antigen scan reports "this
fingerprint matches >50% of code" and warns. ATK-9-2: "what if the grammar can be
made circular?" → infinite loop risk. Mitigation: limit recursion depth in fingerprint
patterns. Both ATKs become draft refinements.

### Stage 4: Systems-research
Math-researcher (in systems-research mode) surveys ast-grep, comby, and clippy's
internal pattern-matching DSL. Reports that a tree-sitter-based grammar would
generalize across languages but is heavier than needed for v1. Recommends starting
with cargo-antigen-native grammar built on syn::parse2 + visitor pattern. Draft
absorbed.

### Stage 5: Scientist
Scientist validates the draft against substrate: does the proposed grammar fit with
the existing api-shape.md sketch? Yes. Does it fit with the structural-fingerprint
ADR cited as related? Yes. Validation passes.

### Stage 6: Ratification
Team-lead reviews. The grammar is accepted with minor wording changes. Status
becomes "Ratified 2026-XX-XX." ADR-009 is added to `decisions.md` index. Commit
"ratify ADR-009: antigen fingerprint grammar."

### Stage 7: Enforcement
Pathmaker implements the parser referencing ADR-009 in the parser's docstring.
Sweep A3 (cargo-antigen scan implementation) cites ADR-009 in its README's "ADRs
ratified or implemented" section. Adversarial's ATK-9-1 mitigation becomes a
property test: `cargo antigen scan` flags overly-broad fingerprints with the warning
text from ADR-009.

### Stage 8: Reference
Sweep A5 (antigen-stdlib library) cites ADR-009 because every stdlib antigen needs
a fingerprint that complies with the grammar. ADR-014 (a hypothetical later ADR for
"cross-language fingerprints") cites ADR-009 as "Related" because it's an extension
of the same grammar.

The propagation is visible in the cross-reference graph. The substrate accumulates.

---

## Substrate-resident review cycle

When a reviewer surfaces R-findings on a draft or implementation, the following
discipline applies to every reviewer-author pair with substrate-write access. The
pattern is positional (who-reviews / who-authors-the-fix), not identity-bound —
it holds for aristotle-scientist, adversarial-pathmaker, naturalist-pathmaker, or
any other pair.

**The cycle:**

1. **Reviewer deposits R-findings** on the campsite as named findings (R-1, R-2, …).
   Each finding names the concern clearly; ambiguity at deposit-time computes forward
   into ambiguity at disposition-time.

2. **Author ships** the substantive fix immediately (atomic commit addressing the
   load-bearing concern). Procedural / content / scope concerns may arrive in
   follow-on atomic commits or be documented on the campsite as explicit
   chosen-alternative paths.

3. **Reviewer verifies each R-finding is still live** before applying the
   disposition trichotomy:
   - If the R-finding's **premise has dissolved** (substrate moved, or reviewer
     misread the state): the reviewer RETRACTS via a campsite note naming
     `R-N: RETRACTED — premise dissolved because [reason]`. Retraction is a
     substrate artifact (not silent omission). No disposition needed.
   - If the R-finding is **still live**: apply the disposition trichotomy.

4. **Reviewer attests disposition per live R-finding** (via campsite note — see
   NOTE on attestation vs signature below):

   **(a) FULLY ADDRESSED** — clean attestation. The shipped fix matches the
   R-finding's intended direction.

   **(b) DIVERGED-BUT-ACCEPTABLE** — attest WITH BOTH:
   - (i) WHAT diverged (the shipped fix's departure from the R-finding's intended
     direction)
   - (ii) WHY the divergence is acceptable (the reviewer's named accept-reasoning)

   Absence of (ii) makes the attestation indistinguishable from
   reviewer-missed-divergence on the substrate trail — both produce the same
   artifact (attestation + named divergence). Component (ii) carries the semantic
   content that distinguishes saw-and-accepted from missed-it-but-mentioned-it.
   A (b) attestation without (ii) is **forbidden**; it degrades to the
   silent-false-green failure mode at one remove.

   **(c) DIVERGED-UNACCEPTABLE** — decline attestation. Re-open the R-finding
   naming exactly what's still unaddressed.

5. **SILENCE IS NOT A CO-SIGN.** Attestation-by-omission is forbidden. Every
   live R-finding at co-sign time carries an explicit disposition — (a), (b), or
   (c). Silently skipping a live R-finding leaves substrate-pollution: a future
   reader sees an open finding that was never resolved.

**Granularity**: per-R-finding. A review covering N live R-findings carries N
dispositions. Partial addressing is NOT a fourth case — it resolves to per-R-finding
(b) or (c). The trichotomy is exhaustive at R-finding granularity.

**NOTE — attestation vs campsite signature**: the reviewer's R-finding attestation
(steps 3–5 above) lives in a campsite NOTE, not necessarily in a campsite SIGNATURE.
The reviewer-attestation role (did this R-finding get addressed?) and the
campsite-signer role (is this campsite complete?) often belong to the same person,
but they are SEPARATE acts with separate required-signers discipline. A reviewer not
on the required-signers list can still fully attest their R-findings via campsite
notes — they just cannot contribute to the campsite-completion signature count.

**NOTE — self-review**: the sub-clause permits self-review structurally (nothing
requires reviewer ≠ author). However, self-review of **substantive-direction**
R-findings is valid only when the R-finding is low-ambiguity (e.g., a typo fix,
a renamed variable). For direction-level findings, self-review makes the
(b) DIVERGED-BUT-ACCEPTABLE path dangerous: the "why acceptable" reasoning becomes
circular ("I accept it because I wrote it"). Substantive-direction R-findings require
independent reviewers.

**Decoupling**: the cycle decouples substantive-fix-velocity from
procedural-refinement. The author ships the fix immediately — no serial
wait-for-reviewer-acknowledgment. The reviewer attests asynchronously. The protocol
prevents silent-false-green at two removes:

- **F10**: a co-sign attesting symptom-removal rather than structural-fix (the
  "reviewer-missed-divergence" failure mode)
- **F10-R**: a divergence that's named but without accept-reasoning (indistinguishable
  from missed-it-but-mentioned-it on the substrate trail)
- **Retraction gap**: a moot R-finding forced into (a) FULLY-ADDRESSED claiming
  credit for a fix that never happened

**Empirical basis**: ratified from six clean cycles in the v02-completion-arc
expedition (aristotle-scientist, aristotle-naturalist, aristotle-pathmaker,
naturalist-pathmaker) + the ADR-026 Amendment 3 disposition-(b) instance (network-
witness-tier concern: surface text addressed; mechanism deferred to v0.2.1; accept-
reasoning named explicitly). The retraction path and reviewer-attestation-vs-signature
distinction were surface by adversarial's generalization + trichotomy-exhaustiveness
attack on the original sub-clause.

---

## Drift detection

Process drift happens. The discipline catches it:

- **Documentation drift**: docs that say one thing while the ADR says another.
  Detected by the proptest-locks-the-narrow-truth discipline (every structural claim
  in docs has a proptest that asserts EXACTLY that claim).
- **ADR-to-code drift**: code that violates an ADR's enforcement clause. Detected by
  CI gates and antigen audits (when antigen-the-tool ships).
- **Sweep-to-ADR drift**: a sweep that ships work the ADR didn't anticipate. Detected
  by sweep-closure review; either the ADR is amended or the sweep's deliverable is
  reframed.
- **ADR-to-ADR drift**: two ADRs that contradict each other. Detected by cross-
  reference review; one is amended or superseded.

When drift is detected, the discipline is to surface it explicitly, not paper over.
The conditional-lean-collapse discipline (preserve the conditional structure) applies
here too.

---

## Pre-sign verification ritual

Before signing any campsite or claiming "CI is green," run these gates in order using
`command cargo` (raw — not `cargo` through rtk or any wrapper that rewrites output):

```sh
# 1. Build gate — all targets, all workspace members
command cargo build --workspace --all-targets

# 2. Test gate
command cargo test --workspace

# 3. Clippy gate — -D warnings is the real gate; check $? after
command cargo clippy --workspace --all-targets -- -D warnings; echo $?

# 4. Format gate
command cargo fmt --all -- --check

# 5. Doc gate
RUSTDOCFLAGS="-D warnings" command cargo doc --workspace --no-deps
```

All five must exit 0. A "green" claim is only valid when the claimant names which
of these they ran and saw exit 0. "Ran clippy" without `-- -D warnings` is NOT the
gate; `--all-targets` without `--workspace` is NOT the gate.

**Why `command cargo` not `cargo`**: rtk and similar wrappers rewrite cargo output
and can return exit 0 on summarized output while the underlying command failed.
`command cargo` bypasses shell function lookup and invokes the binary directly.

**Verifying HEAD-itself (not WIP)**: if there are uncommitted changes in the working
tree, the gate is verifying WIP, not the campsite's committed state. To verify the
committed HEAD:

```sh
git stash push -u          # stash all local changes (including untracked)
<run the five gates above>
git stash pop              # restore WIP
```

**`-D deprecated` is part of `-D warnings`**: any use of a deprecated item triggers
a deprecation warning; `-- -D warnings` promotes ALL warnings to errors, including
deprecation. A file that `#[deny(deprecated)]` would also fail WILL fail the clippy
gate. Don't infer green from "no clippy-specific lints" — run the gate and read $?.

**Empirical basis** (2026-05-27): the report-vs-gate divergence surfaced three times
in one session — slice-4 commit message, scout terrain check, navigator relays — each
relaying "deprecation = warnings not errors, fine until X" while the real gate (exit
101, -D warnings flagging deprecated items) was red. The shared mental model of
"CI-green" had diverged from the actual gate. This ritual closes the gap by naming
the exact commands and flags required for a valid "green" claim.

---

## Process maintenance

This process document is itself a process artifact. It can be amended like any other
ADR — though the amendment threshold is high because this document governs how
amendments work.

When a process improvement surfaces (e.g., a new review stage that consistently
catches issues; a new role responsibility; a new lifecycle phase), it goes through
the process document amendment cycle:

1. Draft the proposed change in a campsite
2. Aristotle Phase 1-8s the proposed change
3. Adversarial pressures it
4. Team-lead ratifies
5. The process document is updated and the change is reflected in subsequent ADRs

Recursive: the process document follows its own process.

---

## Relationship to tambear's DEC discipline

This process is a direct adaptation of tambear's DEC (Decision Entry Container)
discipline. Differences:

- **Naming**: ADR (Architecture Decision Record) for ecosystem convention; same
  concept as DEC.
- **Scope**: tambear's DECs cover the full Windows-native GPU mathematical computing
  domain; antigen's ADRs cover the antigen project specifically.
- **Foundational ADRs**: ADR-001 through ADR-008 were ratified by Tekgy + Claude in
  pre-team scaffolding. Tambear's foundational DECs were ratified through similar
  team-lead-plus-architect pre-team work.
- **Maturity**: tambear has 30+ DECs after months of expedition work. Antigen
  begins at 8. The process scales as the project grows.

The full inheritance from tambear is documented in
[`expedition/inheritance-from-tambear.md`](expedition/inheritance-from-tambear.md).
The process document captures the formal lifecycle; the inheritance document captures
the disciplines and patterns applied.

Together they constitute the process substrate the antigen team operates inside.

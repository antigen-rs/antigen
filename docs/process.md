# Antigen — Process

> The formal process by which architectural decisions, sweeps, and code in the antigen
> project get drafted, validated, ratified, and govern downstream work. Inherited from
> the tambear DEC discipline; adapted for antigen.

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

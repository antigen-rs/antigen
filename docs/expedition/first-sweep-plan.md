# Antigen — First Sweep Plan (Sweep A1)

> Concrete plan for the antigen JBD team's first sweep. Removes day-one ambiguity by
> naming specific deliverables, ATK targets, role assignments, dependencies, and
> expected duration. The team can refine and adjust as they go; this is a starting
> contract, not a constraint.
>
> **Purpose**: when the team launches in a fresh Claude Code session at `R:\antigen`,
> the navigator and pathmaker should not have to invent the first sweep from scratch.

---

## Sweep A1 — Design ratification + scope-lock

**Theme**: deconstruct the pre-team design substrate via Phase 1-8, ratify amendments
or supersessions where the substrate fails first-principles review, lock the scope
for Sweep A2 (core macro implementation).

**Duration estimate**: 2-3 sessions (~3-5 hours total wall-clock; depends on Phase 1-8
depth). The Tambear DEC ratification process suggests 30-60 minutes per ADR
deconstruction; with 10 existing ADRs to review and 2-3 new ones likely surfacing,
the whole sweep fits in this window.

**Blockers**: none. Pre-team scaffolding is complete; substrate is on disk.

**Unlocks**: Sweep A2 (core macro implementation) requires the design substrate to
be ratified-or-amended. Sweep A3+ depends on A2.

---

## Deliverables

By the end of Sweep A1, the team has produced:

### 1. Phase 1-8 deconstructions of the 10 existing ADRs

Each foundational ADR (ADR-001 through ADR-010) gets a Phase 1-8 deconstruction
attached as a campsite document. The deconstructions identify:

- **Load-bearing assumptions** that need to be made explicit if not already
- **Doubtful or wrong assumptions** that need correction (likely amendments)
- **Cross-cutting dependencies** between ADRs that should be made explicit in the
  Related field
- **Open questions** that should be deferred to future ADRs vs. resolved now

Some ADRs may need amendments. ADR-009 (adoption gradient) and ADR-010 (fingerprint
grammar) are the newest and most likely to need refinement. ADR-001 through ADR-008
were ratified by team-lead in pre-team scaffolding and may pass review largely
intact.

**Owner**: aristotle (Phase 1-8 lead). Math-researcher provides reciprocal Phase 1-8
where appropriate.

### 2. Adversarial sweep on each ADR

Adversarial designs degenerate inputs and failure scenarios for each ADR. Output:
ATK-NNN-K annotations on each ADR (where NNN is the ADR number and K is the attack
index).

Expected attacks:
- ADR-001: what if a project's failure-class memory legitimately lives in
  documentation rather than code? (e.g., compliance/regulatory artifacts)
- ADR-002: what if a witness mechanism CONFLICTS with the antigen's own enforcement
  (e.g., clippy lint that's overly broad and causes false positives in cases the
  antigen wants to allow)?
- ADR-009: what's the failure mode when consumers MIX layers (some sites enriched,
  others minimal)? Does cargo-antigen audit handle gracefully?
- ADR-010: what happens when a fingerprint matches code that's intentionally
  declared `#[antigen_tolerance]`? Does the audit understand the exemption?

**Owner**: adversarial.

### 3. Math-researcher / systems-researcher review on ADR-010

ADR-010 (fingerprint grammar) is the most technically uncertain piece. The
systems-researcher reviews:

- syn::parse2 + visitor pattern feasibility for the proposed grammar shape
- Performance bounds for typical workspaces (10-100k LoC)
- Comparison points with ast-grep, comby, dylint, clippy internals
- Tree-sitter integration consideration for v2 (cross-language support)

Output: review document attached to ADR-010's campsite. May surface refinements that
become draft amendments.

**Owner**: math-researcher (in systems-research mode for antigen).

### 4. Scientist validation of all 10 ADRs

Scientist validates each ADR against the project substrate:

- Are the cross-references in each ADR's "Related" field accurate?
- Does each ADR's "Resolves" clause name actual problems documented elsewhere?
- Are the enforcement clauses implementable in the named technical environment?

Output: validation pass/fail per ADR. Failed validation returns the ADR to
appropriate prior stage (Phase 1-8, adversarial, or systems-research).

**Owner**: scientist.

### 5. ADR amendments and any new ADR drafts

Findings from the deconstruction, adversarial sweep, and reviews crystallize into:

- **Amendments** to existing ADRs (likely 2-5 amendments across the 10 ADRs)
- **New ADRs** for surfaced architectural questions (likely 2-3 new ADRs, e.g.,
  the "antigen-tolerance" mechanism for opt-out, the "cross-crate fingerprint
  versioning" design, the "antigen-as-trait" question for type-erased antigens)

**Owner**: pathmaker drafts amendments and new ADRs based on team findings;
aristotle Phase 1-8s the new drafts.

### 6. Sweep A2 scope-lock

The team produces a Sweep A2 README at `sweeps/A2-core-macros/README.md` that names:

- **Theme**: implement the core macro primitives (`#[antigen]`, `#[presents]`,
  `#[immune]`)
- **Blockers**: ADR-001, ADR-009, ADR-010 ratified (Sweep A1 closes them)
- **Unlocks**: Sweep A3 (cargo antigen scan) + Sweep A4 (descended_from + composition)
- **Work-streams**: macro definitions, test fixtures, basic visitor pattern,
  end-to-end smoke test
- **Integration milestones**: where Sweep A2 needs to bridge to Sweep A3's scan
  infrastructure
- **Estimated duration**: 1-2 weeks of pathmaker time

**Owner**: navigator + pathmaker collaborate on scope; team-lead reviews before
launch.

### 7. Closure narrative

Naturalist writes a sweep-closure narrative that names:
- What was learned during Sweep A1 that should propagate beyond it
- Which substrate documents need updating based on learnings (this list, glossary,
  inheritance-from-tambear)
- Predictions about Sweep A2 challenges based on Sweep A1 findings

**Owner**: naturalist.

---

## ATK targets

ATKs (Attack-N findings) are the adversarial output that tracks design weaknesses
and either gets resolved during the sweep or explicitly deferred. The expected
target for Sweep A1: **8-15 ATKs filed across the 10 ADRs**, with at least 80%
addressed in-sweep (resolved as amendments or accepted as out-of-scope with clear
rationale).

Examples:
- ATK-001-1: "what about projects with extremely large codebases (>1M LoC)? Does
  cargo-antigen scan timeout?"
- ATK-002-1: "if multiple witness mechanisms exist for one antigen, how does the
  consumer pick? Is there a fallback chain?"
- ATK-006-1: "the recognition-not-design discipline depends on having instances to
  recognize. What about projects with no historical bug record?"
- ATK-009-1: "if a consumer is at Layer 1 (minimum viable) and another at Layer 2
  (enriched), does the antigen-stdlib provide both? Does the consumer's Layer choice
  affect their experience?"
- ATK-010-1: "the fingerprint grammar's pattern-matching interacts with macro
  expansion. How do antigens behave on code generated by other macros?"

---

## Role assignments and dependencies

```
                                        team-lead (escalation)
                                              │
                                              ▼
                                        navigator (coordination)
                                       /    │    │    \
                                      /     │    │     \
                            aristotle  scientist  observer  naturalist
                            (Phase 1-8) (validate) (record) (close)
                                  │       │
                                  │       │
                            adversarial   pathmaker
                            (ATKs)         (drafts)
                                              │
                                              ▼
                                       math-researcher
                                       (ADR-010 review)
```

Critical-path dependency: aristotle Phase 1-8s before adversarial attacks; both
before scientist validation; all before scope-lock. The scout role does parallel
prior-art scanning (which reduced friction during pre-team scaffolding) — likely
to surface gaps the team should address in amendments.

The naturalist roams during the sweep, observing patterns across the deconstructions
and contributing to the closure narrative.

---

## Process invariants for Sweep A1

Per [`docs/process.md`](process.md), the lifecycle for Sweep A1 work is:

1. Each existing ADR gets a Phase 1-8 deconstruction document in its campsite
2. Each adversarial finding gets an ATK-NNN-K annotation
3. Each amendment gets a draft document in the campsite, then ratified through the
   normal lifecycle
4. Each new ADR gets a draft document in a new campsite, then ratified through the
   normal lifecycle
5. Sweep A2 scope is locked only after all Sweep A1 ratifications are complete
6. Closure narrative goes to naturalist's campsite + a brief summary in
   `sweeps/A1-design-ratification/CLOSURE.md`

---

## What Sweep A1 does NOT do

- **Implementation work**: no code changes to `antigen/src/lib.rs` or
  `cargo-antigen/src/main.rs` beyond what's needed to support deconstruction (e.g.,
  small example fixtures). Real macro implementation is Sweep A2's work.
- **Stdlib content**: the seed antigens at
  [`stdlib-seed-antigens.md`](stdlib-seed-antigens.md) stay as pseudocode; their
  implementation is Sweep A5's work.
- **External outreach**: the vision pitch at [`vision-pitch.md`](../vision-pitch.md)
  is not yet shared publicly; that happens after Sweep A2 ships and there's working
  code to point at.

---

## Closure criteria

Sweep A1 closes when:

- All 10 existing ADRs have Phase 1-8 deconstructions (✓ owned by aristotle)
- All adversarial ATKs are addressed or explicitly deferred with rationale (✓ owned
  by adversarial)
- ADR-010 systems-research review is complete (✓ owned by math-researcher)
- All ADRs pass scientist validation (✓ owned by scientist)
- Any amendments and new ADRs are ratified (full lifecycle through team-lead approval)
- Sweep A2 scope is locked in `sweeps/A2-core-macros/README.md`
- Naturalist's closure narrative is written
- CI is green (no implementation work means no test churn; should remain green)
- Documentation updates land if any substrate changed (glossary, inheritance,
  process docs) — update via PRs reviewed by team-lead

---

## After Sweep A1

Sweep A2 (core macros) launches with:
- A ratified ADR substrate
- A locked Sweep A2 README naming specific work-streams and dependencies
- The naturalist's closure narrative providing context
- The team's accumulated context on the design's strengths and weaknesses

The first sweep teaches the team how to operate. The second sweep applies the
lessons. By Sweep A3 the team is fluent in the project's specific shape and can
autonomously navigate the work.

---

## Why this plan exists

Without a concrete first-sweep plan, the team launches into a large pre-team
substrate without a clear starting point. They might:
- Spend the first session reading without producing
- Argue about what to do first (productive but slow)
- Default to implementation work before the design is ratified (premature)
- Skip Phase 1-8 and assume the substrate is correct (skip validation)

This plan removes that ambiguity. The team can deviate as they see fit — JBD energy
encourages deviation when warranted — but they have a clear day-one direction.

The plan is a contract between team-lead's pre-team thinking and the team's
autonomous work. The team owns the outcomes; the plan is starting context, not
constraint.

---

## References

- [`docs/process.md`](../process.md) — full ADR lifecycle and sweep process
- [`docs/decisions.md`](../decisions.md) — the 10 existing ADRs
- [`docs/expedition/team-briefing.md`](team-briefing.md) — spawn-time briefing for
  the JBD team
- [`docs/expedition/inheritance-from-tambear.md`](inheritance-from-tambear.md) —
  what disciplines come pre-loaded from tambear

# Deferred substrate — what's parked, why, and what unblocks it

> **Purpose**: durable index of substrate that is intentionally deferred,
> so items don't fall through cracks while attention is on something else.
>
> **Maintained by**: navigator. The team-lead and Tekgy read this to
> verify nothing is being lost. Side-substrate items not on this list
> may be drifting silently.
>
> **Discipline**: when work is deferred, the deferral is recorded HERE
> with status + why-deferred + what-unblocks. When the unblock condition
> is met, the item moves out of this index into active campsite work
> (and the row is removed). The index itself is the substrate-currency
> check at the team-coordination tier — the answer to "is anything being
> lost while we focus on X?"
>
> **Companion to**: `postures.md` V0+1 candidates section (which tracks
> patterns watched for posture-class promotion). This index tracks
> *work* deferred, not patterns watched. The two are different shapes.
>
> **Status**: V1 (2026-05-09, post-ADR-017+018 ratification). V0 authored
> during ratification-cycle pause; V1 updated when commit 3ef4b9a landed —
> D1.5 moved to active, ATK-A3-009/007 marked now-unblocked, encounters-
> proposal marked tracked in git.

---

## A3-immediate (post-ratification cleanup)

Items that unblock when ADR-017 + ADR-018 ratification commit lands.

**ADR-017 + ADR-018 ratified 2026-05-09, commit 3ef4b9a.** Items below
are now unblocked and awaiting teammate wake-up; D1.5 moved to active.

### ATK-A3-009 reframe

**What**: Original attack surface (name-only silent match) is structurally
eliminated by ADR-017's name@version canonical_path format. Contract needs
either reframing to the residual registry-collision risk (same-name@version
from different registries — Open Question 1 in ADR-017) OR explicit closure
with a note. ATK-A3-010 already absorbed the drift-vs-waning angle; this
reframe covers only the identity-format angle.

**Where it lives**: `antigen/tests/atk_a3_fractal_preview.rs` (the existing
#[ignore] contract); scout's structural insight that superseded ADR-017 v3
Option B; scope-lock §verification checklist (last entry).

**Unblocked by**: ADR-017 ratified (commit 3ef4b9a). Now active.

**Owner when active**: adversarial's call (scout proposed the reframe;
adversarial decides whether to reframe to registry-collision risk or close).

### ATK-A3-007 reframe

**What**: Scout found the trust boundary is at enumeration, not scan.
Test contract should assert `enumerate_dep_crate_roots` never returns
paths not in cargo metadata's graph (not: `scan_workspace` rejects fake
registry paths). Pre-impl contract update in `atk_a3_fractal_preview.rs`
or a new fixture file.

**Where it lives**: scout's findings during ADR-017 review; ADR-017
§Trust delegation ("Do not add alternative path-discovery mechanisms");
scope-lock §verification checklist.

**Unblocked by**: ADR-017 + ADR-018 ratified (commit 3ef4b9a). Now active.

**Owner when active**: adversarial's call.

### ~~D1.5 implementation (propagation walk + diamond dedup)~~

**MOVED TO ACTIVE** (2026-05-09, commit 3ef4b9a unblocked). Pathmaker
owns implementation. ProvenanceEntry struct + `inherited_from:
Option<Vec<ProvenanceEntry>>` on Presentation + propagation algorithm
are all ratified in ADR-018. Algorithm is in decisions.md §ADR-018 §Mechanics.
Removing from deferred index.

---

## A3 sweep deferral (parallel substrate)

Items that are A3-cadence but explicitly held while ratification cycle
runs.

### Encounters proposal — Phase 1-8

**What**: Proposal for a third vocabulary tier below `postures.md` V0+1
candidates, for first-encounter formal capture so subsequent encounters
recognize each other rather than being treated as fresh first-recognitions.
Scope expanded mid-draft to cover patterns + vocabulary + structural-moves
+ frames; carries tracing (graph relationships between encounters).

**Where it lives**: `docs/expedition/encounters-proposal.md` (now tracked
in git as of commit after 3ef4b9a; 446-line proposal).

**Unblocked by**: aristotle's bandwidth post-A3-ADR-ratification.
Scout was prepping Q3 (entry shape) sharpening before aristotle starts
Phase 1-8; that prep may already be done.

**Owner when active**: aristotle Phase 1-8 → team deconstruction →
ratification per process.md.

### Naturalist's deeper biology-cognate work

**What**: Multiple sparks emerged during A3 launch — version-boundary as
immune memory re-activation (corrected to drift/shift, not memory-waning),
declared-vs-earned identity rhyme, Approach 4 (fingerprint-keyed) and
Approach 8 (epitope-class lattice) as biology-cognate post-A6 territory.
Threads not yet woven into the manuscript trajectory or
`immune-system-primitive-map.md`.

**Where it lives**: naturalist's A3 campsite entries; `cross-domain-architectural-map.md` V1 (academic-researcher's A2 work).

**Unblocked by**: idle-as-invitation (naturalist's natural cadence);
manuscript drafting cycle when scientist re-engages.

**Owner when active**: naturalist + scientist (when scientist spawns).

---

## Vocabulary candidates — held below ratification thresholds

Watched, but explicitly not yet ratified per ADR-006. Promoted to
`postures.md` V0+1 only when shape stabilizes; promoted to ratified
posture only on full threshold clear.

### substrate-currency (two-axis observation)

**What**: Three-tier framing reframed as two-axis (mechanism × substrate-
domain) during A3 launch. Now durably in `postures.md` V0+1 candidates
section with evolution-as-inoculation preserved.

**Where it lives**: `postures.md` V0+1 substrate-currency entry (Path-1
substrate); aristotle's posture draft at
`campsites/antigen-design/20260507161107-manuscript/scientist/substrate-currency-posture-draft.md`.

**Unblocked by**: cross-session temporal independence + same-cell
repetition (same mechanism × same substrate-domain seen at least twice)
+ concept stops surprising trackers. All three required.

**Owner when active**: scientist (posture draft) → aristotle (Phase 1-8)
→ team ratifies.

### Halt-state drift (candidate fourth substrate-currency mechanism)

**What**: When Anthropic usage limit hits, all account activity halts
instantly; agents not killed but mid-flight work stalls and resumption
isn't always automatic. Captured as candidate fourth mechanism (alongside
tracker / reporter / claim-propagation / persistence-registry). Substrate-
domain: harness-substrate.

**Where it lives**: `~/.claude/projects/R--antigen/memory/feedback_usage_limit_hard_stop.md`
(role-memory tier, not project-substrate yet).

**Unblocked by**: same-cell repetition (another halt-state drift event in
harness-substrate). When recurrence happens, register as encounter for
substrate-currency tracking; promote to V0+1 when shape stabilizes.

**Owner when active**: navigator (notice + log); team-lead (route to
substrate-currency posture work).

### V0+1 candidates already in postures.md

Already durably surfaced in `postures.md` V0+1 candidates section:
- antigen-grammar / antigen-engine architectural cut
- filter / proof split
- accept-and-note discipline
- settling-time diagnostic

These are watched per their own thresholds; not duplicated here.

---

## Cross-sweep deferrals (multi-sweep horizon)

Substrate explicitly deferred to A4+ or post-A5 by ratified ADRs or
team-lead rulings.

### A4+ substrate accumulating

**What**: Cross-language tree-sitter scoping; cross-crate semver
discipline; `#[descended_from]` inheritance semantics across version
boundaries; doc-comment embedding path (verified-viable but ADR-001
amendment territory); Eiffel D1/D2/D4 invariants from math-researcher.

**Where it lives**: scout's seeds doc + various A2 campsite entries.

**Unblocked by**: A3 close → A4 scope-lock authoring.

**Owner when active**: navigator (scope-lock) → team Phase 1-8 → ratify.

### Post-A5 ADR territory

**What**: Static-emission via `#[cfg(doc)] pub static` (scout verified
the attribute matrix; ADR-001 amendment required); separate
`antigen.json` artifact path; `cargo-checkmate` integration; SARIF
output (scout's full design note with antigen → SARIF mapping table).

**Where it lives**: scout's A3 seeds doc.

**Unblocked by**: A5 scope-lock or no-source-access case actually biting
in adoption.

**Owner when active**: scout proposes; team Phase 1-8.

### W6b body-level fingerprint operators

**What**: ast-grep subprocess for body-level operators per ADR-015.

**Where it lives**: SESSION-HANDOFF-2026-05-09; ADR-015.

**Unblocked by**: deferred to v0.2 (per A2 close).

**Owner when active**: pathmaker.

---

## Release-cadence deferrals

Items held by Tekgy's no-rush posture (per `feedback_team_lead_no_rush_discipline.md`).

### v0.1.0-rc.1 tag

**What**: Tag is ready-to-cut; substrate validated; held per Tekgy's
no-rush posture. Will tag when Tekgy decides.

**Where it lives**: SESSION-HANDOFF-2026-05-09; release substrate
already prepared.

**Unblocked by**: Tekgy's call. Team-lead surfaces readiness; user owns
timing.

### Tambear migration to crates.io version-pin

**What**: After v0.1.0 final ships, tambear migrates from path-dep to
crates.io version-pin.

**Where it lives**: SESSION-HANDOFF-2026-05-09.

**Unblocked by**: v0.1.0 final tag pushed to crates.io.

**Owner when active**: tambear-side work; antigen team supports if
adoption friction surfaces.

### Multi-paper publication trajectory

**What**: One big paradigm-shift paper + many smaller venue-specific
papers across 15+ academic disciplines. "No need to fully pick now"
per Tekgy.

**Where it lives**: scientist's manuscript campsite; scope.md;
cross-domain-architectural-map.md.

**Unblocked by**: scientist re-engages when manuscript work calls;
substrate matures at different rates across windows.

**Owner when active**: scientist (when spawned); team supports.

---

## Maintenance discipline

When an item from this index is unblocked and active work begins, **remove
its row from this index**. The index records *deferred* substrate, not
*all* substrate.

When new substrate is deferred, **add it here** with the same shape:
what / where-it-lives / unblocked-by / owner-when-active.

The index is corrected when:
- Items mature past V0+1 thresholds and become postures-class
- ADRs ratify and unblock multiple items at once (as ADR-017 + ADR-018
  ratification will unblock A3-immediate cleanup)
- Cross-sweep boundaries move (e.g., a post-A5 item becomes A5-immediate
  when A4 closes)

This index ITSELF is a substrate-currency artifact at the team-
coordination tier — it makes deferral state explicit so future-instances
of the team don't drift on what's-being-watched-vs-what's-being-lost.

---

*V0 authored 2026-05-09 during antigen-A3 launch session by team-lead
in conversation with Tekgy. Tekgy flagged the risk of side-substrate
drift; index makes the deferral durable.*

*V1 updated 2026-05-09 by navigator: ADR-017 + ADR-018 ratified
(commit 3ef4b9a); D1.5 moved to active (row removed from deferred);
ATK-A3-009/007 marked now-unblocked; encounters-proposal marked tracked.*

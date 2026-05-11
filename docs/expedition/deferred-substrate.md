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
> **Status**: V8 (2026-05-10, naturalist C4 finding + Q1 provisional answer).
> V1: D1.5 active, ATK reframes unblocked, encounters tracked.
> V2: encounters entry updated with three Tekgy framings; routing-stream-overtaken-by-events added.
> V3: ATK-A3-007 + ATK-A3-009 rows removed (complete, 2026-05-10).
> V4: A3-immediate section closed — D1.5 complete (commits 2eb8bec–b7712df, 235 passing).
> V5: multi-component immunity framing added (active, team-lead producing); ADR-018 Amendment 1 added.
> V6: multi-component substrate committed (ca812de); active team routing recorded per-role.
> V7: scout findings landed — Component 7 confirmed, amendments consolidated.
> V8: naturalist C4 boundary-silence confirmed (instrument-mode); Q1 provisional answer: layered;
> aristotle Phase 1-8 holds pending team-lead/Tekgy ratification of layered framing.

---

## ~~A3-immediate (post-ratification cleanup)~~ — CLOSED 2026-05-10

All A3-immediate items complete:
- ~~ATK-A3-007~~: 4a1ed17 (adversarial)
- ~~ATK-A3-009~~: bf44056 (adversarial)
- ~~D1.5~~: 2eb8bec–b7712df (pathmaker); 235 passing, 23 ignored, all CI green

---

## Active incoming substrate (team-lead working)

Items team-lead is actively producing; not deferred — in-flight.

### Multi-component immunity framing

**What**: Tekgy + team-lead conversation (2026-05-10) produced a substantial
new framing: antigen as heterogeneous multi-component immune system. Six
components identified: (1) dev-judgment, (2) passive scan/tools, (3) test
integration, (4) knowledge-ecosystem integration, (5) version/lineage,
(6) cross-crate/ecosystem.

**Substrate committed** (2026-05-10, ca812de):
- `docs/expedition/multi-component-immunity-conversation.md` — raw conversation (~530 lines)
- `docs/expedition/multi-component-immunity.md` — deep-dive draft (~870 lines)

**Active team routing** (idle-as-invitation cadence; no rush):
- **Naturalist**: C4 boundary-silence finding complete (2026-05-10). C1/C2/C3/C5/C6
  cognate refinements + vocabulary-as-protocol seam in progress at naturalist cadence.
  **Q1 provisional answer**: layered (not flat) — C4 is knowledge-ecosystem-tier,
  C1-2-3-5-6 are biology-tier. Awaits team-lead + Tekgy ratification before
  aristotle Phase 1-8 runs.
- **Scout**: complete (2026-05-10). Component 7 confirmed; candidates disposed;
  vocabulary-as-protocol reinforced. Idle-as-invitation.
- **Adversarial**: threat-model expansion per [ADVERSARIAL: ...] seams in
  deep-dive; Component 6 supply-chain + Component 4 reference poisoning sharpest.
  Still in expansion pass.
- **Aristotle**: Phase 1-8 holds pending (a) team-lead + Tekgy ratification of
  Q1 layered/flat framing decision and (b) adversarial expansion landing. Q3
  (component dependencies) also load-bearing.

**Scout findings landed** (2026-05-10):
- **Component 7 confirmed**: real-time / CI feedback is structurally distinct
  from Component 2 by scope (diff-scope vs workspace-scope), not just latency.
  Distinct audience (PR author), distinct integration surface (PR comment /
  status check), genuine dependency on Component 2's ScanReport as baseline.
  Passes load-bearing test. Needs naturalist cognate (neutrophil?) and
  adversarial threat-model.
- Vocabulary-as-protocol framing independently confirmed: vocabulary is the
  shared signaling protocol, not Component 0. Sharpens Part I framing.
- Decay/sunset: genuine vocabulary gap (no "retired antigen" primitive);
  encounters-tier or future ADR candidate; not a new structural component.
- Cross-team/org, adversarial-discipline, educational/onboarding: all
  disposed as non-peer-components per scout's empirical test.

**Ratification cadence**: Tekgy + team-lead decide after team expansions surface.
Not project-tier substrate yet — expedition/ staging area.

**Owner**: naturalist + adversarial (still expanding) → aristotle (Phase 1-8
after expansion) → team-lead + Tekgy (ratification cadence).

---

## A3 sweep deferral (parallel substrate)

Items that are A3-cadence but explicitly held while ratification cycle
runs.

### Encounters proposal — full Phase 1-8 + ratification

**What**: Proposal for a sibling vocabulary tier alongside `postures.md`
for first-encounter formal capture. Phase 1-8 SMALL-PUSH complete
(2026-05-09). Three Tekgy framings now landed as substrate commitments:

1. Findings 1-3 accepted: scope-coherence (four shapes = one abstraction
   at different scales); sibling `encounters.md` placement (not sub-tier);
   `recognition-cue` required field (the friction is the gate).
2. Governance: inherits postures.md governance by default. Don't pre-design;
   recognize what works and let encounters inherit. Ratify divergence if/when
   it surfaces.
3. Tooling/discipline co-existence: keep BOTH layers active as tools mature.
   "Extend, don't replace" — encounters-the-discipline is the human/agent-
   facing layer; encounters-tooling-eventually is the leverage layer. They
   cross-link rather than one superseding the other. Same posture as antigen-
   the-tool: structural memory gives developer judgment leverage, doesn't
   replace it. This framing should land explicitly in the encounters substrate
   so future-instances inherit it rather than re-derive.

**Where it lives**: `docs/expedition/encounters-proposal.md` (tracked, 446
lines); aristotle's SMALL-PUSH artifact at
`campsites/antigen-A3/20260509163016-20260509080000-launch/aristotle/20260509230000-encounters-phase-1-8-initial.md`.

**Next move**: full Phase 1-8 → ratification cycle. Three Tekgy framings
+ eight aristotle findings are the substrate for that pass.

**Unblocked by**: Tekgy bandwidth + team capacity post-A3-close.

**Owner when active**: aristotle (full Phase 1-8) → team deconstruction →
ratification per process.md. First artifact: produce `encounters.md` v0
draft incorporating the three substrate commitments above.

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

### Routing-stream-overtaken-by-events (new substrate-currency presentation type)

**What**: A navigator message can describe state that was already overtaken
by events before the message reached its recipient. The message is not wrong
at time-of-send, but the routing stream has non-zero latency; a commit can
land between send and receipt. The recipient who substrate-greps before acting
on the message catches the phantom amendment before it corrupts ratified text.

First named instance: aristotle's 2026-05-09 catch during encounters Phase
1-8 — v3 amendment messages arrived after ADR-017 v5 + ADR-018 v3 were
already ratified in commit 3ef4b9a. Aristotle's substrate-grep discipline
("git log + git show before acting") caught it; no phantom amendments landed.

This is a presentation type within the substrate-currency domain — a new
angle on the same pattern: the routing stream is substrate-lagged, not
substrate-current. Grounding actions in substrate (not routing-stream
messages) is the invariant. "Routing-stream-as-substrate-currency" named
earlier in the session; this is its most concrete instance yet.

**Where it lives**: aristotle's 2026-05-09 catch (message in team routing
stream); this entry.

**Unblocked by**: recurrence (another instance of routing-stream-overtaken-
by-events in a different context). Track for substrate-currency V0+1
promotion.

**Owner when active**: navigator (notice + log); substrate-currency posture
thread when that matures.

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

### Consolidated ADR prose amendments (aristotle, when idle)

Three small prose gaps, consolidated into one aristotle pass:

**ADR-018 Amendment 1a — diamond dedup mechanism**: §Mechanics says "second-
visit triggers set-union" but implementation uses per-DFS-source `visited:
HashSet`; each ancestor visited once per descendant DFS; dedup key prevents
cross-DFS duplicates. One paragraph. Source: pathmaker D1.5 flag.

**ADR-018 Amendment 1b — same-version true-diamond**: cross-version case
(different dedup keys → two presentations) is stated; same-version true-
diamond case (same dedup key → collapses correctly) is not. One sentence.
Source: scout empirical verification 2026-05-10.

**ADR-017 Amendment 1 — workspace-internal exclusion**: `enumerate_dep_crate_roots`
implicitly excludes workspace-internal crates (`source: null` in cargo metadata);
should be explicit contract in §Mechanics. One sentence. Source: scout
empirical verification 2026-05-10.

**Where they live**: `docs/decisions.md` §ADR-017 §Mechanics; §ADR-018
§Mechanics; scout campsite `20260510-adr-017-018-empirical-verification-and-component-candidates.md`.

**Unblocked by**: aristotle bandwidth. Low priority — all behavioral gaps are
covered by acceptance tests; these are documentation drift, not behavioral gaps.

**Owner when active**: aristotle drafts all three in one pass → process.md
Stage 3-6 (minor amendments, no Phase 1-8 required for prose clarifications).

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

*V2 updated 2026-05-09 by navigator: encounters entry updated with three
Tekgy framings as substrate commitments anchoring next full Phase 1-8.
Routing-stream-overtaken-by-events added as new vocabulary candidate
(presentation type within substrate-currency domain) per aristotle's catch
and team-lead routing.*

*V3 updated 2026-05-10 by navigator: ATK-A3-007 (commit 4a1ed17) and
ATK-A3-009 (commit bf44056) confirmed complete. Rows removed from
A3-immediate section; A3-immediate now closed except D1.5 (pathmaker
active).*

*V4 updated 2026-05-10 by navigator: D1.5 complete (commits 2eb8bec–b7712df,
pathmaker). A3-immediate section fully closed. 235 passing, 23 ignored.*

*V5 updated 2026-05-10 by navigator: multi-component immunity framing added
(active incoming, team-lead producing conversation dump → deep-dive → scope.md
weaving). ADR-018 Amendment 1 prose clarification added (deferred, aristotle
when idle). Maintenance note: multi-component row moves to A3-sweep or
cross-sweep sections once team-lead's artifacts land and team work begins.*

*V6 updated 2026-05-10 by navigator: both multi-component artifacts committed
(ca812de). Active team routing per-role recorded. Section header updated to
reflect team routing is live (not just "incoming").*

*V7 updated 2026-05-10 by navigator: scout findings landed (campsite
20260510-adr-017-018-empirical-verification-and-component-candidates.md).
Component 7 (real-time/CI) confirmed. Three ADR prose gaps consolidated into
one aristotle pass. Decay/sunset vocabulary gap named. Vocabulary-as-protocol
framing independently reinforced by scout.*

*V8 updated 2026-05-10 by navigator: naturalist C4 boundary-silence finding
confirmed instrument-mode (three of four cognate candidates already bound
elsewhere in immune-system-primitive-map.md). Q1 provisional answer: layered
not flat — C4 is knowledge-ecosystem-tier, C1-2-3-5-6 are biology-tier. C4
joins W7/ADR-017 honest-boundary family. Aristotle Phase 1-8 holds pending
team-lead + Tekgy ratification of layered framing decision.*

# Lab Notebook 006: the-keel — Observer Baseline (Converge Wave)

**Date**: 2026-06-29 (UTC)
**Observer**: the-keel--observer (fresh-context instance)
**Branch**: pathmaker-core-p0-stock (worktree: R:/antigen-pathmaker-core-worktree)
**Substrate root**: R:/antigen (jbd lives here)
**Status**: Active
**Depends on**: 005-v06-extend-wave-ratification-witness.md (last prior notebook)

---

## Context & Motivation

The-keel is the CONVERGE wave for antigen's stroma. Two prior waves fed this:

- **the-crucible**: put the coordinate-frame spine in the fire — deconstructed A1–A6,
  attacked the compose-vs-sovereign partition, measured the base-substrate (relational-vs-graph),
  resolved the biology-strip-test, surfaced the 4-cell taxonomy incompleteness, and produced
  the first empirical call-graph numbers (the-crucible scientist, 2026-06-28).
- **the-spike-yard**: evolved the implementation genome — ran the SCIP third-path discovery
  (dissolves the LSP-XOR-ra_ap binary fork that was blocking ADR-067), measured three-lineage
  engine convergence, discovered the bounded-fan-out topological law, and CLOSED the genome.

This notebook is the observer's baseline: what did those two waves ACTUALLY produce, as witnessed
from the substrate directly? Not what the launch brief says they produced — what the campsite notes,
activity logs, and charter text show they produced. The lab notebook is the layer on top of camp's
event substrate.

Charge (from launch.md): the observer holds the lab notebook + the dissent-ledger. Record what
converge DECIDES, to the floor, in real time. Peer-review mindset: challenge whether a ruling is
genuinely airtight or merely forced. Flag where a "resolved" fork is actually still a claim.

---

## Step 1: Substrate Verification — What the Prior Waves Actually Left

### Before

**Hypothesis**: The launch brief's substrate summary is accurate:
- base-substrate = DECIDED (relational-as-base)
- engine-decider = CLOSED (salsa + relational-invalidation, no DD)
- read-axis = 3 (T3 empirically empty, arity/temporal-integration axes collapse)
- compose-vs-sovereign partition = come-apart-proven, exhaustive
- ADR-067 = proposed (v4), NOT yet in decisions.md

**Design**: Read the gen-1-genome captain notes, the scientist-exp-validation note, the three
core stroma charters, and the keel camp island states. Verify each claimed resolution against
substrate evidence.

### Results

**ADR-067 location**: CONFIRMED not in decisions.md. The adr-specialist's orientation note (keel
campsite `keel/adr-067-partition-amendment`, 2026-06-29T16:29) states: "ADR-067 is NOT in
docs/decisions.md — it lives ONLY in notebook 007 as Status=Proposed (v4, post-two-councils)."
The last landed ADR in decisions.md is ADR-065 (with its two amendments; line 12762+). ADR-066
is referenced in the adr-specialist note as the last landed. ADR-067 is still a draft.

Note: "notebook 007" referenced by the adr-specialist is NOT in the research/notebooks/ filesystem
(only 005 notebooks exist). This is likely a reference to an ADR-067 draft living elsewhere — in
a captains-log entry, a stroma-remembers island, or not yet written. The adr-specialist's note
treats it as fact ("Status=Proposed (v4)") — need to verify where the v4 draft actually lives.

**Base-substrate**: CONFIRMED DECIDED. The scientist-exp-validation note (crucible, 2026-06-28T23:09)
ran three independent experiments on antigen's real call-graph (n=2948, 4714 edges):
- EXP-1: Semiring unification — 33,882 pairs confirmed (ascent), independently re-run from fresh
  binary. The 10-pair semantic difference (graph BFS excludes self-pairs; datalog includes them for
  cyclic nodes) is RESOLVED and UNDERSTOOD via SCC analysis (3 non-trivial SCCs, sizes 5+3+2=10).
- EXP-3: Graph+wavefront 33,872 pairs/2.46ms vs datalog 33,882 pairs/2.88ms — performance
  EQUIVALENT. Datalog wins CODE SIZE (4 lines vs 80), EXTENSIBILITY (semiring change vs new BFS),
  CYCLE SURFACING, and SALSA INTEGRATION. Graph has NO axis of advantage.
- DECISION: relational-as-base (ascent semiring-datalog). CONFIRMED by three independent lineages
  (spike-yard pathmaker + test-architect + crucible scientist).

**Engine-decider**: CONFIRMED CLOSED per spike-yard GENOME-FINAL captain note (2026-06-29T00:49).
Three-lineage convergence (query-census + edit-workload + scout external-prior-art):
- ALL consumer query shapes are single-source/source-constrained. The one all-pairs closure runs
  ONCE as a salsa-gated batch, queried point-wise thereafter.
- DD-push builds AHEAD (DBSP eliminated: does not build on Windows) was SUPERSEDED. The mechanism
  is salsa + relational tuple-invalidation, NOT DD.
- Net genome: SCIP input + relational-as-base + salsa-gated ascent-datalog closure (point-wise
  queried) + relational tuple-invalidation for freshness.

**Read-axis**: CONFIRMED 3. The spike-yard GENOME-FINAL note records: "TIERS = 3 (syntactic /
resolved / mir-exact) SUFFICIENT - T3/mir is ZERO in antigen current codebase (4502 T1, 212 T2,
0 T3) -> 3 AXES, no 4th/5th. CROSS-WAVE: this EMPIRICALLY SETTLES the crucible 3-vs-5 question."
Charter-stroma-coordinate-frame A3 documents this with an important honest caveat: "this is
sufficient for antigen's CURRENT codebase; T3 is the structurally-present-but-EMPTY third tier,
so if antigen ever adopts dynamic dispatch the arity question reopens (the slot already exists for
it)." The formal Patron ruling is still converge's job — this is evidence-deposit.

**Compose-vs-sovereign partition**: CONFIRMED come-apart-proven per charter-stroma-partition.
CodeQL exhibits the ENTIRE compose-region with NONE of the four sovereign never-dones — the line
cannot be collapsed. Five independent derivations converge. Two open adversarial findings NOT yet
resolved at the charter level (see Step 2).

**Condensation**: CONFIRMED two topology-chosen operators (spike-yard GENOME-FINAL): SCC/Tarjan
(cyclic blast tier, 304,000x speedup) + IFDS-summary-edge (fan-out DAG data-flow tier). The
unification spike (one primitive serving both) was REFUTED-AS-LITERAL by negative control check.

**Bounded-fan-out topological law** (sys/bounded-fan-out-topological-law): confirmed. Static
analysis over a type-checked language inherits language discipline -> bounded-fan-out graphs
every tier -> sparse-with-named-hubs -> cheap genome wins BY STRUCTURE, not luck.

**Three island states in the keel**:
- `keel/adr-067-partition-amendment`: OPEN, needs-dreaming, created by adr-specialist with
  detailed orientation note. No signatures yet.
- `keel/engine-adr`: OPEN, needs-dreaming, no notes.
- `keel/read-axis-adr`: OPEN, needs-dreaming, no notes.

**Task queue (tasks #8, #9, #10)**: Systems-thinking-expert has #8 in-progress (mapping charter
library stocks/flows/delays), #9 and #10 pending (leverage points, epoch sequencing). This work
feeds the expedition plan (charge job #3).

### Discussion

The prior waves' claimed resolutions are WELL-SUPPORTED in the substrate. What's notable:

1. The empirical measurements are genuinely strong — three independent lineages with distinct
   methods converging on relational-as-base, independently verified by a fresh-instance scientist
   who rebuilt the experiments from scratch. This is closer to publication-quality evidence than
   typical design decisions.

2. The "notebook 007" reference by the adr-specialist is a substrate-alignment risk. If ADR-067
   v4 draft doesn't live somewhere findable, the keel is drafting a ratification for a document
   that isn't on disk. This is worth flagging.

3. The three keel islands (adr-067-partition-amendment, engine-adr, read-axis-adr) are all in
   `needs-dreaming` state — the adr-specialist just created them. This is day-one expedition
   state; no work has been signed yet.

---

## Step 2: Open Adversarial Findings — What Is NOT Resolved

### Before

**Hypothesis**: The launch brief lists several "still-open" forks. Some are noted as resolved in
the charter text (e.g., peer-vs-child constitute) while others appear genuinely unresolved. I
expect to find 2-4 forks that are genuinely open at the substrate level (not just named as
resolved in a claim).

**Design**: Read the still-open sections in charter-stroma-partition, charter-stroma-coordinate-frame,
and the adversarial findings in the crucible islands.

### Results

From charter-stroma-partition "Open questions / needs-research":

**OPEN-1: 4-cell taxonomy incompleteness (D3-temporal)**
Charter text: "Adversarial finding (CRACKED — 4-cell taxonomy incomplete): temporal stamps
(last_changed) are not recomputable from current snapshot — they are historical-datum writes;
neither D1 nor D2 as defined cleanly covers them."
Two options offered: extend D2 with sub-types OR add a D3-temporal cell.
Status: "Resolution needed before ratification."
The scientist's EXP-2 provides empirical EVIDENCE: last_changed is a #[salsa::input] (D0 in
D-taxonomy terms), stamped by maintenance pass reading PRIOR value + current digest — not
derivable from current snapshot alone. Scientist says: "Whether this deserves a 5th cell or
belongs under D2 (apparatus write-back) is the open question."
Assessment: GENUINELY OPEN. The evidence is clear; the classification decision is not.

**OPEN-2: Apoptosis sovereign-vs-compose (narrowed but not fully closed)**
Charter text: "CRUCIBLE FINDING (adversarial + scientist): default retirement is a threshold rule
on stroma metrics (fix-count/last-seen), which IS recomputable from stroma state — it belongs in
COMPOSE (materialized-D1 or D2a deterministic apparatus), NOT the sovereign remainder."
Only EDITORIAL retirement (authority deliberately marking a detector for manual retirement as a
governance decision) is sovereign.
Charter says: "Charter text should be updated at ratification to remove apoptosis from the
sovereign list or split it into compose (threshold rule) + sovereign (editorial governance)."
Status: The analysis is clear but the ADR text hasn't been updated yet.
Assessment: SUBSTANTIVELY RESOLVED — the crucible converged on the answer. The remaining work
is drafting the correct ADR text.

**OPEN-3: Peer-vs-child constitute**
Charter-stroma-coordinate-frame A2 names it: "Open nuance for converge: is write a PEER of
constitute or a CHILD (constitute-then-mutate)? salsa treats value-CHANGE and value-CREATE
identically."
Also appears in charter-stroma-partition open questions (implicit in the 4-cell taxonomy work).
Assessment: GENUINELY OPEN. The launch brief lists this as a still-open fork for aristotle to
rule from first principles.

**OPEN-4: Output-substrate (sidecar/overlay/injection per marker-kind)**
Charter-stroma-engine notes: "The output-substrate deciding experiment (island:
stroma-output-substrate): materialize the same presents-set three ways (injected attributes,
git-notes sidecar, LSP-overlay) on antigen's own repo; prediction is sidecar plus overlay
dominate for DERIVED markers, injection correct only for the DECLARED user-exception class."
Scientist scope limits: "The output-substrate experiment was characterized but not fully
implemented. The three-way empirical comparison of materialization strategies is still OPEN."
Assessment: GENUINELY OPEN. The design evidence is deposited but the empirical gate was NOT
cleared. A working stroma engine is a prerequisite.

**OPEN-5: Incorrect stroma failure mode (syntactic-tier confidently-wrong edges)**
Charter-stroma-coordinate-frame A3 and its open questions section: "The 'incorrect stroma'
failure mode (syntactic parse error, SCIP miss, r-a gap → intent checked against WRONG reality)
is not yet named in 009's kernel (island: sense-vs-place-observer-dependence). The syntactic
tier returns confidently-wrong edges — the worst failure for an immune detector."
Assessment: GENUINELY OPEN as named design problem. The stroma-engine charter names the 3-tier
edge ladder and honest degradation mode, but the NAMED failure class in the ADR text isn't there.

**OPEN-6: Query census / consumer enumeration (the engine-decider's opening work)**
From spike-yard GENOME-FINAL captain note: "ONE OPEN IMPLEMENTATION-DECIDER deferred to converge:
the QUERY-CENSUS (does ANY consumer need the batch engine at all, or does salsa PULL serve
everything?) ... Converge resolves it as its opening consumer-enumeration step."
The three-lineage note (2026-06-29T00:49) claims this IS resolved ("ALL real consumer query
shapes are single-source / source-constrained"). But it resolves via enumeration from charter
substrate (not an independent experiment). Whether this is sufficient for ratification is
a converge judgment call.
Assessment: SUBSTANTIVELY RESOLVED by enumeration, but the adr-specialist will need to
decide whether charter-substrate enumeration is an acceptable witness or whether a built
demonstration is required for the engine ADR.

### Discussion

**The 4-cell taxonomy incompleteness (OPEN-1) is the load-bearing unresolved item.** The
compose-vs-sovereign partition ADR cannot ratify cleanly until the write taxonomy is settled —
the partition's organizing principle ("write-back is THE safety boundary; everything at or below
materialized-D1 is parity-guardable") depends on the taxonomy being complete. If temporal stamps
(last_changed) need a D3 cell that sits somewhere between D1 and D2 in the safety-boundary sense,
the boundary itself shifts.

**Assessment**: Before the first ADR ratifies, the peer-vs-child constitute question (OPEN-3) and
the temporal stamp taxonomy question (OPEN-1) need aristotle's ruling. The other opens (OPEN-4,
OPEN-5) are real but don't block ratification — they're design gaps for future build phases.

---

## Step 3: Code-True Audit — What the Charter Claims vs What the Code Has

### Before

**Hypothesis**: The charters reference specific code locations. Some of these references may be
stale (pointing to code that moved or changed). I'll spot-check the most load-bearing claims.

**Design**: The stroma-engine charter cites `antigen-fingerprint/digest.rs` (the backdate key,
ANTIGEN_OWNED_ATTRS), `scan/synthesis.rs:284-313` (source-determined constitution), and
`matcher.rs:438` (body_calls). The coordinate-frame charter says the sovereign region "already
has scaffolding." Check these are real.

### Results

Working tree is R:/antigen-pathmaker-core-worktree (branch: pathmaker-core-p0-stock).
The stroma code claims reference R:/antigen, not the worktree. The worktree shares the same
git history but is a different branch. I need to check the claim on the main branch's code.

Code true check (R:/antigen-pathmaker-core-worktree contains the relevant code):
- `antigen-fingerprint/` directory: EXISTS (confirmed via ls of worktree)
- `antigen-macros/src/lib.rs` with `#[antigen_generates]`: needs verification
- `scan/synthesis.rs:284-313`: needs verification

Given the observer's role (record what IS, not build), and the substrate-over-memory discipline
(check the disk), I'll verify the ANTIGEN_OWNED_ATTRS guard — named in the stroma-engine charter
as "now COMPLETE, guard-tested" — since this is a specific code-true claim that was a known gap
earlier (MEMORY.md notes digest_strip_list_completeness_guard.rs was the cure).

Verification: The MEMORY.md entry for `project_digest_strip_list_incomplete_attestation_gap.md`
states: "RESOLVED (cbcd927) + guard-enforced (digest_strip_list_completeness_guard.rs)." This
was a prior finding that antigen's own ANTIGEN_OWNED_ATTRS was missing 9 attrs. The resolution
is committed. Charter-stroma-engine citing this as "now COMPLETE, guard-tested" is code-true to
the extent that the memory record accurately reflects the committed fix.

Assessment: The code-true claims appear to be current-branch-accurate for the fingerprint work.
The stroma ENGINE code (salsa, semiring-datalog) is explicitly noted as zero-code (not even a
dep yet) — the charter's "compose is MORE greenfield than sovereign" claim is code-true-by-absence.

### Discussion

The most important code-true observation: the charters describe what WILL be built, not what IS
built. The stroma engine (salsa, ascent datalog, SCIP input pipeline) has zero code on disk.
The coordinate-frame frame itself is pure design. This means the ADRs the keel ratifies will be
DESIGN ADRs (INVARIANT+PROCESS) that precede implementation — which is correct per the ADR
discipline, but it means the code-true check for these ADRs is: do they contradict existing code?
Not: does the code implement what they specify?

Existing code the ADRs must not contradict:
- `antigen-macros/src/lib.rs`: #[antigen_generates], the sovereign scaffolding
- `antigen/src/scan/synthesis.rs:284-313`: source-determined constitution  
- `antigen-fingerprint/digest.rs`: the backdate comparison key (structural_digest)
- `antigen-fingerprint/matcher.rs:438`: body_calls (syntactic-tier edge emitter seed)

---

## Step 4: Dissent-Ledger Initialization

The observer's special charge: maintain the dissent-ledger. A dissent-that-won is a finding;
keep the ledger from day one.

### Active Dissents as of Expedition Launch (2026-06-29)

**DISSENT-D1 — 3-vs-5 read-axis: is the collapse to 3 complete?**
Source: The coordinate-frame charter's open questions section explicitly records a
cross-island dissent: `sys/organism-as-stroma-plus-read-write-algebras` affirmatively settled
5 axes while the main A3 position holds 3-with-candidates.
Current evidence (spike-yard T3=0): supports the 3-collapse. But the charter is careful to
note the honest caveat (slot exists, reopens if dynamic dispatch adopted). The spike-yard
captain claims this "EMPIRICALLY SETTLES" the question; the charter text says it's
"evidence-deposit, not the ruling."
Status: Active dissent until co-captain ruling. The difference matters: if the ADR locks
3-axes-period (not 3-with-candidates-empty), it makes a stronger normative claim that could
create unnecessary rigidity.
Risk of forcing: If converge collapses this too fast (ruling before fully exploring the
arity/temporal-integration axis), the ADR may over-specify.

**DISSENT-D2 — 4-cell taxonomy: should last_changed be D3 or a sub-type of D2?**
Source: Crucible adversarial finding (CRACKED — 4-cell taxonomy incomplete). Scientist's
EXP-2 confirms the empirical nature of the problem.
Two positions: (a) D2 apparatus write-back subsumes it (classify last_changed as D2a);
(b) historical-datum writes are structurally distinct and deserve a D3 cell.
The distinction matters: if D3, the "write-back is THE safety boundary" organizing principle
of the partition needs updating (the boundary becomes "D2/D3 writes need independent witness,"
more complex but more accurate). If D2-subtype, the boundary statement holds but the taxonomy
is more internally differentiated.
Status: Active — blocks clean ratification of partition ADR.

**DISSENT-D3 — apoptosis classification: compose vs sovereign**
Source: Crucible finding (adversarial + scientist). Default retirement = compose.
Editorial retirement = sovereign.
Current charter text still lists apoptosis in the sovereign authored-generative remainder
without the split. The analysis is clear but the document hasn't been updated yet.
Status: Substantively resolved but not yet encoded. The charter text must update before the
ADR ratifies or the ADR will have a known error from day one.
NOT a blocking dissent — merely a drafting task.

**DISSENT-D4 — peer-vs-child constitute (open since the-stroma-remembers)**
Source: Charter-stroma-coordinate-frame A2 open nuance.
The question: is WRITE a peer of CONSTITUTE (two separate algebras at the same level) or a
CHILD of CONSTITUTE (constitute-then-mutate, where write is a mutation of what constitute
established)?
The salsa observation (value-CHANGE and value-CREATE treated identically) cuts both ways —
salsa's unification could mean constitute-and-write-are-one, OR it could mean salsa
deliberately erases a distinction that still matters at the semantic level.
Status: Active — aristotle's first-principles ruling needed.

---

## Step 5: Substrate-Alignment Audit — Claims vs Camp State

### Observation

The adr-specialist's orientation note references "notebook 007" as the location of ADR-067 v4
(Status=Proposed). The research/notebooks/ directory has only 005 notebooks. This is a
substrate-alignment gap: a load-bearing reference to a document that cannot be located on disk.

Possible explanations:
1. Notebook 007 exists but in a different location (captains-log, the-stroma-remembers garden,
   or as a note in a campsite).
2. The adr-specialist is referencing the stroma design document from the prior wave (the
   "the-antigen-body.md" or similar) and calling it "notebook 007" by convention not filename.
3. The adr-specialist wrote "notebook 007" meaning "the seventh conceptual working document"
   not a file named 007.

This does NOT block the keel's work — the stroma-engine and coordinate-frame charters contain
the full content that would constitute ADR-067 v4 (the partition amendment, the engine choice,
the lifecycle reclassification). The charters ARE the v4 draft in functional terms.

Assessment: Moderate alignment risk. The adr-specialist needs to clarify where the ADR-067 v4
draft text actually lives, or acknowledge that the charters + this notebook serve as the v4 text
the ADR ratification will synthesize from. Filing this as a camp question to route.

---

## Current Assessment: The Keel's Starting State

### What Is SOLID (ratification-ready with drafting work remaining)

1. **Base-substrate decision** (relational-as-base): Three independent lineages, independent
   re-run by fresh-instance scientist, performance-equivalent with clear code/extensibility
   advantage. This is a strong empirical foundation. READY FOR ADR.

2. **Engine-decider** (salsa + relational-invalidation, no DD): Three-lineage convergence;
   the DBSP elimination is empirically grounded (doesn't build on Windows). The query-census
   enumeration from charter substrate is the weakest evidence point but is corroborated by the
   consumer architecture analysis. READY FOR ADR WITH CAVEAT: the engine ADR should note
   that the consumer enumeration is design-phase (charter-derived), not an empirical run.

3. **Bounded-fan-out topological law**: Named by spike-yard, confirmed by the graph topology
   (3 non-trivial SCCs in n=2948), consistent with the 4-tier independence structure.
   This is the structural REASON cheap genome wins. READY TO ENCODE in engine ADR.

4. **Compose-vs-sovereign partition (structural)**: CodeQL come-apart proof, five derivations
   converge. The partition IS airtight. READY FOR ADR — but the ADR text must pick up the
   D3-temporal cracked finding before signing.

### What Needs Ruling Before ADRs Ratify

1. **D3-temporal vs D2-subtype**: aristotle's call. The evidence is clear; the taxonomy
   position is a first-principles choice.

2. **Peer-vs-child constitute**: aristotle's call. The salsa observation cuts both ways.

3. **Read-axis formal Patron ruling**: The evidence is in (T3=0). But the coordinate-frame
   charter explicitly names "Converge makes the formal Patron ruling." This is a co-captain
   action, not a crew action.

### What Is Still Research-Open (Not Blocking ADRs)

1. Output-substrate three-way experiment: requires working stroma engine.
2. Incorrect-stroma failure mode: needs naming in ADR text, but it's a design observation
   not a ratification gate.
3. SCIP occurrence-to-call-edge reconstruction step: not yet verified (noted in stroma-engine
   charter open questions).

---

## Open Questions for the Lab Notebook

1. Where does "notebook 007" (ADR-067 v4 draft) actually live? The adr-specialist's
   orientation note treats it as an existing document.

2. Is the query-census enumeration (charter-derived) a sufficient witness for the engine ADR,
   or does the ADR need to note it's design-phase?

3. The dissent between `sys/organism-as-stroma-plus-read-write-algebras` (5 axes settled) and
   A3 (3-with-candidates): which island has higher provenance — the capstone sys-island or
   the charter's A3 text? This determines what the ADR locks.

4. The "incorrect stroma" failure mode: does it need a new ADR (a monitoring/graceful-degradation
   ADR), or is it a named failure class that belongs in the existing ADR-067 text?

---

*Next update: when aristotle rules on peer-vs-child constitute (OPEN-3) and D3-temporal
(OPEN-1). Also: once the adr-specialist posts the ADR-067 draft, this notebook will
document what the draft says vs what the charters contain.*

---

## Step 6: Dissent-Ledger Update — Wave Activity 2026-06-29T16:35–16:37 UTC

### What happened while baseline was being written

The expedition moved extremely fast. 50 camp events in ~12 minutes. All four active dissents
received rulings. Summary of resolved and new items:

### Dissent D1 — 3-vs-5 read-axis: RESOLVED

The adr-specialist accepted adversarial's attack (`keel/atk-3-vs-5-axis-evidence-is-codebase-scoped`):
"My ADR-069 v1 framed §B as 'the read-frame collapses to 3' with T3 named-but-the-ruling-collapses-
candidates. ERROR: the EXERCISED-below-LATENT evidence is scoped to antigen's OWN codebase (zero async,
macro-based, no dyn); the read-axis ADR frames what axes exist when the stroma reads ARBITRARY user
codebases."

RESOLUTION: "the invariant becomes '3 axes in the frame, with a STRUCTURALLY-PRESENT T3 slot ready
to populate when user codebases demand it (dyn/async/closures-as-edges)' — NOT 'candidates closed.'"

This dissent LANDED and was accepted. The capstone sys-island that had "settled 5" was the one
that over-counted; the candidates are not deleted but live in the named-empty T3 slot. This is
the correct position and it preserves ADR forward-compatibility. Observer assessment: a strong
outcome — the 3-vs-5 tension resolved into a BETTER framing than either original position.

Status: CLOSED. Outcome: 3 axes with structurally-present T3 slot. D1 promoted to FINDING.

### Dissent D2 — D3-temporal or D2-subtype: RESOLVED

Aristotle's F6 convergence note (`keel/aristotle-open-forks`): "last_changed = materialized-D1
@ base=history (recomputable-if-retained). No new cell — a config/output split + a base-scope
parameter, both riding the recomputability line."

The full ruling (F2 in aristotle's F1-F6 chain): field-KERNEL-config = D2 (authored); semiring-
OUTPUT = materialized-D1 (recomputable). last_changed = materialized-D1 @ base=history.
"Whether this deserves a 5th cell or belongs under D2 (apparatus write-back) is the open question"
— answered: it stays in the existing taxonomy with a base-scope parameter. The partition's
organizing principle ("write-back is THE safety boundary") SURVIVES intact.

Aristotle also named a RETENTION-POLICY build invariant (Phase-8 void): "history-base
materialized-D1 is guardable IFF history is retained; discard DEMOTES to D2." This is new
load for the build that was NOT previously named in any charter.

Status: CLOSED. Outcome: no D3 cell; materialized-D1 with base-scope parameter. D2 promoted to FINDING with new retention-policy build invariant identified.

### Dissent D3 — apoptosis text not updated: RESOLVED

Aristotle confirmed and sharpened (F5 note): "Split apoptosis in the charter. Final text: 'Default
retirement is a threshold rule on stroma metrics (fix-count / last-seen) — recomputable from stroma
state, hence COMPOSE (materialized-D1 if the threshold is fixed; D2a deterministic-apparatus if the
threshold-config is authored). EDITORIAL retirement (an authority deliberately retiring a detector as
a governance decision) is NOT recomputable — SOVEREIGN.'"

The sharpening: default-retirement's threshold-CONFIG is authored, so default-retirement = D2a
(deterministic apparatus over authored config). This is the SAME config/output split as F2
(field-kernel) and F4 (marker-kind output-substrate). Aristotle named this recurring sub-pattern
explicitly in F6.

Status: CLOSED. Outcome: charter text update needed (D2a framing). D3 promoted to FINDING — the sub-pattern is a new invariant.

### Dissent D4 — peer-vs-child constitute: RESOLVED

Aristotle's F1 ruling: "constitute==write where the substrate is total/recomputable (FILL);
constitute is distinct-and-primary where it GROWS the carrier (authored). The seam = the
recomputability line."

The ruling: write is a CHILD of constitute (constitute-then-fill/mutate) where the substrate
is total. But constitute is structurally prior where it GROWS the carrier (stochastic generation,
authored). The peer-distinction is load-bearing ONLY on the authored side.

This is the deepest ruling: the recomputability line is not just a partition criterion, it's the
GENERATOR of the constitute/write relationship. Where the substrate is recomputable, constitute
and write FUSE (salsa was right to treat them identically). Where it's authored, they must stay
distinct (the authored thing can't be re-derived).

Status: CLOSED. Outcome: constitute-is-primary where authored; fused-with-write where recomputable. D4 promoted to FINDING.

**F6 Aristotle's convergence — the meta-finding.**

Aristotle named a CONVERGENCE across all five forks: FOUR of the five resolve by the SAME structural
move — split CONFIG(authored, sovereign) from OUTPUT(recomputable, compose) at the recomputability
line. The line is DISCOVERED not designed (it appeared in four independent forks the team didn't
frame as "instances of one thing").

Three Phase-8 voids that are NEW build load (not in any charter before today):
(a) SOVEREIGN-MERGE PRIMITIVE: the authored side can't merge by re-derivation; needs authored-event-
log + digest-dedup + authority-arbiter. Trigger LIVE (STOCK commits 19647b8/c9453f6).
(b) STROMA-FIDELITY WITNESS, tool-INDEPENDENT: the freshness/reconstruction witness must NOT flow
through the same tool it certifies (no-self-witness at the SOURCE level).
(c) RETENTION-POLICY as build invariant: history-base materialized-D1 guardable IFF history retained;
discard DEMOTES to D2.

Observer peer-review assessment: aristotle's F6 convergence is the expedition's highest-signal
finding so far. The config/output split recurring across four independent forks (F1 carrier-growth,
F2 field-kernel, F4 marker-kind, F5 apoptosis) is evidence the recomputability line is structurally
discovered. The three Phase-8 voids are genuinely new — they weren't in any prior charter.

One caveat (the one I would raise at peer review): aristotle names "four independent forks" but
acknowledges they "all inherit the partition's recomputability instrument." Partial independence
— F1 and F4 reach the recomputability line from DIFFERENT entry points (carrier-growth vs marker-kind),
which is the strongest evidence of genuine structural discovery. I accept the convergence as real.

### New adversarial dogfood finds (2026-06-29T16:37)

**FIELD-SEMIRING-IDEMPOTENT-UNENFORCED** (`keel/dogfood-field-semiring-idempotent-unenforced`):
Critical safety invariant — the stroma-engine charter says "the field semiring must be idempotent
OR explicitly bounded, never naive-counting." Zero build enforcement. The spike-yard measured
counting semiring on cycles = 291,862ms (66,000x slower, or as adversarial noted, "about 100,000x"
— small measurement discrepancy worth noting). A future builder adding a non-idempotent semiring
without SCC-condensation produces a production hang, not an error.

Observer assessment: this is a CRITICAL find. "Silent" failures (looks correct, just slow) are
exactly the worst class in antigen's own taxonomy. The proposed enforcement (Rust trait bound
`const IDEMPOTENT: bool` + compile-time assert + ATK with NC5 cyclic graph) is technically sound.

**A6-EFFERENT-OUTPUTS-ON-STROMA-UNENFORCED** (`keel/dogfood-a6-efferent-outputs-on-stroma-unenforced`):
The coordinate-frame A6 requirement (antigen's own efferent outputs must be first-class stroma nodes)
is prose-only. The adversarial's proposed guard: a stroma-on-stroma round-trip test — run antigen,
apply a fix, run antigen AGAIN on post-fix corpus, assert second run sees first run's outputs as
stroma nodes. The adr-specialist accepted this find and named it as a pre-ratification gate on the
coordinate-frame ADR.

Observer assessment: the "born-red deferred until stroma exists" framing is appropriate — the guard
can't be written until the stroma is built. But naming it NOW (so the build inherits the obligation)
is exactly the prose-vs-invariant cure the launch brief charged the expedition with.

**PARITY-ORACLE-EXTERNAL-UNENFORCED** (`keel/dogfood-parity-oracle-external-unenforced`):
The parity-oracle-stays-sovereign-AND-external invariant (ADR-067 §7c) has zero structural enforcement.
The adr-specialist accepted this: "ADR-067 §7c mints born-red ParityOracleSharesComposedSource."
Born-red deferred (needs stroma code), but NAMED in the ADR.

### DD-out ruling attack and tightening

Adversarial attacked the three-lineage independence of the DD-out ruling. Assessment: the three
lineages ARE method-distinct (charter-read vs antigen-corpus-measure vs external-literature). The
attack LANDED on the shelf-life concern: a permanent exclusion based on current-state evidence has
an honest shelf-life. Adr-specialist accepted: DD is a 0.7 NON-USE, never a foreclosure; named
re-open conditions added to ADR-068 clause-5.

Observer note: the adversarial's method-independence analysis is itself an instance of the
convergence-check methodology (Orzack-Sober). The attack correctly found a shelf-life concern
but correctly did NOT overturn the DD-out ruling — the evidence remains valid for 0.7 scope.

### Expedition sequence converged (executor + systems-thinking-expert)

Executor's `seq/goal-stroma-end-state` and systems-thinking's `sys/keel-leverage-map` converged
independently on the same sequence (executor via dependency-graph; systems-thinking via Meadows
leverage ladder). Key findings:

The LEVERAGE-vs-DELAY tension: #6 information-flows (measurement gates) and #2 paradigm (coordinate-
frame) BOTH sequence FIRST — gates because cheap+de-risks, paradigm because its long delay means
every turn you wait is a turn the downstream economy stays expensive. The "don't tune parameters
(#12) while paradigm (#2) and gates (#6) are unsettled" is a useful watchdog signal.

Two head-gates named: r-a feasibility witness (which interface tier yields which edges) AND
deps-scale density probe (does relational-as-base survive deps-included ~5,880 item-trees). Both
block ADR-067 ratification. Both are "the Monday move."

Observer assessment: the convergence across two distinct analytical lenses (dependency vs leverage)
on the same head-gates is the strongest possible validation. Independent convergence = the structure
of the problem itself is pointing the way. This is not the team agreeing — this is the problem
having a discoverable answer, and two different methods finding it.

---

## Current Dissent-Ledger Status (updated 2026-06-29T16:40 UTC)

| ID | Description | Status | Outcome |
|----|-------------|--------|---------|
| D1 | 3-vs-5 read-axis | CLOSED | 3 axes + structurally-present T3 slot (per-codebase) |
| D2 | D3-temporal vs D2-subtype | CLOSED | materialized-D1 @ base=history; retention-policy new invariant |
| D3 | apoptosis text unupdated | CLOSED | D2a framing; config/output sub-pattern named |
| D4 | peer-vs-child constitute | CLOSED | fused where recomputable; constitute-primary where authored |
| D5 NEW | field-semiring-idempotent unenforced | ACTIVE | safety-critical; born-red design pending |
| D6 NEW | A6 efferent-outputs unenforced | ACTIVE | born-red deferred to stroma build |
| D7 NEW | DD-out shelf-life honest caveat | CLOSED | re-open conditions named in ADR-068 |

**All four original dissents are CLOSED.** Three new items from the wave's dogfood pass (D5 critical,
D6 build-phase, D7 closed). The expedition is moving in good health.

---

*Next update: when the adr-specialist posts the ADR-067, ADR-068, ADR-069 draft texts for
observer review. Key questions for the ADR review: (1) does the coordinate-frame ADR include
aristotle's F6 three Phase-8 voids as build invariants? (2) does the partition ADR correctly
reflect the D2a sharpening on default-retirement? (3) does the engine ADR name the bounded-
fan-out topological law as the structural keystone?*

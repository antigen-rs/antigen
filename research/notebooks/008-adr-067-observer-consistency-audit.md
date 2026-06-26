# Lab Notebook 008: ADR-067 Observer Consistency + Supersession Audit

**Date**: 2026-06-24
**Authors**: observer (camp-voyage-observer role)
**Branch**: pathmaker-core-p0-stock (worktree: antigen-061-self-non-self)
**Status**: Complete
**Depends on**: notebook 007 (ADR-067 draft), notebook 006 (math-feeder verdicts), ADR-066 + ADR-002 Amd3 in `docs/decisions.md`

---

## Context & Motivation

Team-lead requested an independent observer pass on the ADR-067 draft before ceremony. This is the scientific-conscience role: assess internal consistency, supersession cleanliness, honest-scope, and ceremony-readiness. The observer does NOT redesign — only reports.

Sources read in this pass:
- `research/notebooks/007-adr-067-sovereign-stroma-builder.md` (the draft)
- `docs/decisions.md` — ADR-066 full text (lines 13107–13412), ADR-002 Amd3 (lines 530–630)
- `research/notebooks/006-adr-066-math-followon-verdicts.md` (the math-feeder output)
- Code baseline verification: `grep -rn "LineageEdge|stroma"` across `*.rs`

---

## Step 1: Internal Consistency of the 10 Decision Clauses

### Before
**Hypothesis**: The 10 clauses should cohere without contradiction. Three specific tension points were flagged for scrutiny: A1 "full-AST never coarsened" vs. persistence-as-outcome; C5 "wavefront = field-diffusion"; B4 induced-views vs. D7/D8 sheaf + edge-tests.

### Results

**A1 vs. persistence-as-outcome — tension exists, but it is correctly classified.**

A1 says "never coarsened at the base." The "Process not outcome" section correctly defers the *persistence strategy* (persist-full vs. compute-live-persist-bounded vs. recompute-on-demand) as outcome. The tension is: if persistence strategy drifts, can you persist a coarsened form? The ADR blocks this by making "the base is never coarsened" the *invariant* and "how it's persisted" the *outcome*. The key sentence that seals this: "Detail is dialed downstream, never by coarsening the base." This is clean — the invariant governs what is stored *in* the base; the outcome governs how the base is physically persisted. Not a contradiction.

**C5 "recompute wavefront rides the same diffusion the field uses" — this is the most structurally interesting claim.**

The claim: the "could-have-changed closure" = the dependency-closure = percolation's traversal. This is elegant but has a circularity risk: the field diffusion is *not yet built* (it is explicitly deferred as outcome and future ADR). So what does "rides the same diffusion" mean before the field exists?

Reading carefully: the ADR says the wavefront rides the *same graph traversal* the field will later use — forward along dependency edges for freshness, backward for detection. This is topologically sound even without the field scalar implemented. The wavefront IS the reachability closure on the graph. The field later will BE a scalar value assigned to each node via that same traversal. So "same diffusion" means same graph structure, not same implementation. This is correct in substance, but the phrasing "rides the same diffusion the field uses" implies the field is operational. The field is NOT built in this ADR. This is a **phrasing imprecision**, not a logical error.

**B4 induced-views vs. D7/D8 sheaf + edge-tests — the critical boundary question.**

B4 declares: structures (topology, sheaf, manifold, field) are *induced views*, not built into the base.
D7 says: "the sheaf lens is a detector" — node-local contracts are *local sections*, consistency conditions on edges.
D8 says: edge-tests / edge-bindings are "a new defense surface."

The question: do D7/D8 smuggle sheaf *requirements* back into the base?

The answer after careful reading: **they do not smuggle, but the language is ambiguous.** B4 says nodes carry "a local contract (what it provides / requires)." That local contract IS the section in the sheaf. So the base *does* carry something the sheaf needs — but this is in the *node attribute set* (open, extensible per A1), not in a built-in sheaf structure. The sheaf lens then reads those attributes and checks cross-edge consistency. This is the correct induced-view architecture: the data lives in the base attributes; the structure (sheaf consistency check) is a lens over that data.

D8's edge-tests are more subtle. They run "continuously as you code — LSP-grade companion." But the "live editor surface" is explicitly deferred to a follow-on ADR in "What this ADR does NOT do": "the live editor surface — their own follow-ons; this ADR ships the base." So D8's claim that edge-tests run live is a *capability the lattice enables*, not something this ADR builds. The ADR is describing what the edge-test *is* conceptually and that it runs live — but the live running is a future layer.

This is a **scope-bleed risk**. D8 reads as if it's shipping live edge-tests. The "What this ADR does NOT do" section correctly disclaims this, but the D8 clause itself does not caveat. A reader of D8 in isolation would conclude live edge-tests ship with this ADR.

**Other clause coherence**: The remaining clauses (A2 open-edge-alphabet, A3 node-lifecycle, B4 views-architecture, C6 bidirectional-selective-signal, D9 "am-I-OK?" lens, E10 sovereignty) are internally coherent. No contradictions found.

### Discussion

**What we learned**: Three tensions found. Two are minor imprecisions (C5 phrasing, D8 scope-bleed). One is correctly handled (A1 vs. persistence). No logical contradictions in the core invariant.

---

## Step 2: ADR-066 §2 Deferral — Does ADR-067 Cleanly Deliver?

### Before
**Hypothesis**: ADR-066 §2 deferred four specific items to the stroma-builder follow-on: collision-free identity, lifecycle, digest, graph-integrity. ADR-067 should address all four.

### Results

Reading ADR-066 §2 exactly:

> "Identity is the stroma-node in antigen's own sovereign stroma-builder (ADR-002 Amd3): fully-qualified, collision-free; node lifecycle (deletion/rename/split re-home or re-open bindings); it owns the digest + threat model... Graph-integrity is a faithfulness precondition for the field math (§4): the stroma-builder follow-on ADR MUST carry it."

Cross-checking against ADR-067 Decision:

- **Collision-free identity** → A1: `(qualified-path, structural digest)`, never bare item-name, never line number. DELIVERED.
- **Node lifecycle** → A3: create/delete/rename/split/merge as first-class events; lifecycle connects to `AntibodyRebindsAcrossLocalDisambiguator`. DELIVERED.
- **Digest** → A3: "commitment over canonical structural tokens; doc-comment-only edits don't re-open; semantic-equivalence-under-different-tokens is out-of-scope by the ceiling." DELIVERED with honest ceiling.
- **Graph-integrity** → A3: "Graph-integrity (edge-provenance, unfabricated edges) is load-bearing for the field-math's honesty — a poisoned neighborhood would manufacture false-quiet with mathematical credibility." DELIVERED, cross-references ADR-066 §2.

**Notebook 006 keystone claim**: "The graph is the keystone of both halves of ADR-066 — binding-identity soundness (§2) and the field-math (§4). One build unlocks both."

ADR-067 adopts this framing explicitly in its Finding section. The convergence claim is accurately represented.

**ADR-002 Amd3 clause-3 empirical rebuttal**: ADR-066 §2 says "its clause-3 empirical rebuttal lives in its follow-on ADR." ADR-067 E10 delivers this:

> "The Amd3 empirical rebuttal: roll-our-own is cheaper (the parse is sunk; the marginal cost is the graph) AND sovereign... composing rust-analyzer would be a full IDE engine doing far more than we need, delegating the very identity §2 rests on — it loses the cost/risk/liability rubric."

This matches ADR-002 Amd3 clause 3's requirement: "Scaffold BOTH paths... and decide against a cost/risk/liability rubric... with an author-distinct witness confirming the verdict."

**Critical finding**: The rebuttal is *present* in E10 — but it is *not empirically completed*. The ADR itself flags this: "Pending an author-distinct witness that `syn` + `cargo metadata` genuinely suffice for the call/data-flow graph before this ratifies."

Amd3 clause 3 is clear: the author-distinct witness is **required** for the rebuttal to count. The ADR honestly surfaces this in Open Seams. The rebuttal is *stated* and *structured* correctly but **not yet witnessed**. This is not a flaw in the ADR's internal logic — it is an honest open seam — but it means **the Amd3 rebuttal is a pre-ratification blocker**, not merely a seam.

### Discussion

ADR-066 §2 deferral items: all four delivered cleanly. Notebook 006 keystone convergence: accurately represented. The Amd3 clause-3 rebuttal: present and correctly structured, but the author-distinct witness is pending and is required by Amd3 before ratification.

---

## Step 3: Honest-Scope Assessment

### Before
**Hypothesis**: The biology Class-1 vs. honest-invention split, the "What this ADR does NOT do" section, and the Open Seams should be accurate and complete. Check for: quietly-built things that are disclaimed; real blockers hiding as seams.

### Results

**Biology Class-1 split — accurate.**

The ADR claims as Class-1: thymic stroma (AIRE / tolerance-induction as the self-presentation analog), tissue-as-manifold (immune cells traffic the stroma = field substrate), Matzinger danger model (respond to change/damage, not whole-body re-survey = change-driven incremental model), receptor selectivity (cytokine diffuses, only receptor-bearing cells respond = change-type × requirement match).

These are all genuine, well-established immunology. The mapping is tight and not overreached.

The ADR correctly puts into honest-invention (biology grounds none): `syn`/`cargo-metadata` extraction, the digest + integrity scheme, the incremental-update + full-rebuild-witness, the induced-views functor architecture, the sheaf/edge-test machinery.

This is the same split discipline ADR-066 used and passed ceremony with. No overclaiming found.

**"What this ADR does NOT do" — mostly truthful, one gap.**

Disclaimed correctly:
- Field representation, maths, membrane, live editor surface: all deferred to follow-ons. TRUTHFUL.
- Does not coarsen the base for perf. TRUTHFUL.
- Does not delegate load-bearing node identity to external tools. TRUTHFUL.
- Does not discard `LineageEdge` DAG (subsumes it). TRUTHFUL — code baseline confirms `LineageEdge` is the only graph today.
- Does not decree schema/field representation/persistence strategy. TRUTHFUL.

**Gap**: D8 describes edge-bindings and live edge-tests as a "new defense surface" with real-time LSP-grade behavior. The "does NOT do" section disclaims "the live editor surface." But D8 as written is a Decision clause, not a current-map/orientation clause — it reads as ratified invariant, not deferred orientation. If it IS ratified invariant, then the live edge-tests are committed to (even if built later). If it is orientation, it should not be in the Decision section. This is either a scope-bleed (D8 claims more than the base ships) or a structural placement issue (D8 should be in current-map, not Decision). The "does NOT do" disclaimer covers the *implementation*, but the *commitment* in D8 is architectural: "an edge-test is an antibody on an edge." That architectural commitment is what this ADR would be ratifying, and that seems intentional and appropriate. The ambiguity is in the *live-running* sub-claim of D8, not the architectural definition of edge-as-binding.

**Open seams — are these the real open questions, or are blockers hiding?**

Five seams listed:
1. Field representation not here. GENUINE seam — correctly named.
2. Persistence strategy. GENUINE seam — correctly named and classified as outcome.
3. Workspace / cross-crate scope. GENUINE seam — real design question.
4. Conductance model (type edges now, weight at field layer). GENUINE seam.
5. Verify Amd3 rebuttal with author-distinct witness. **THIS IS A BLOCKER, not a seam.**

Seam vs. blocker: a seam is something the *next ADR* handles; a blocker is something *this ADR cannot ratify without*. ADR-002 Amd3 clause 3 requires the author-distinct witness to constitute the empirical rebuttal. The ADR explicitly states E10 is "Pending an author-distinct witness... before this ratifies." That phrase "before this ratifies" is the tell — the authors know this is a ratification precondition, not a future design question.

The ADR's own Open Seams section calls it "Verify the Amd3 rebuttal with an author-distinct witness" — using the word "verify" which implies it could go either way. The witness might find that `syn` + `cargo-metadata` genuinely do NOT suffice for call/data-flow edges, in which case the sovereignty argument changes substantially (either a different build strategy or a potential compose decision emerges). That is a load-bearing unknown for E10's ratifiability.

**Classification**: This seam should be labeled BLOCKER, not seam.

### Discussion

Biology split: clean, no overclaiming. "Does NOT do" section: truthful, with the D8 live-edge clarification needed. Open Seams #5 is misclassified — it is a pre-ratification blocker per Amd3 clause 3's own requirement.

---

## Step 4: Process/Outcome Split

### Before
**Hypothesis**: The durable/drifting split in "Process not outcome" should correctly classify each item. Check specifically: field representation as outcome, persistence as outcome.

### Results

Reading the "Process not outcome" section:

**Invariant (durable)**: sovereign full-AST attributed base; fully-qualified collision-free identity; local contracts; directed typed provenanced edges; induced views over base (never coarsen); persisted as single-source-of-truth; conservative change-driven increment; witnessed by full-rebuild; bidirectional + selective signal; load-bearing identity never delegated. — All of these are structural commitments that should not change without superseding this ADR. CORRECTLY classified.

**Process (durable)**: built from sunk `syn` parse + `cargo-metadata`; change as danger signal; recompute-wavefront rides field's diffusion; sheaf re-checks only change-touched edges; edge-tests span lint-grade→reasoned. — These are method-level decisions. The `syn` + `cargo-metadata` path is somewhat contestable as "durable" (what if the build system changes?), but it is flagged by the pending Amd3 witness anyway.

**Outcome (must drift)**: node/edge schema; which edge-kinds; conductance weights; field representation (per-node scalar vs. richer); persistence strategy; digest canonicalization; materialization policy; live-editor surface shape. — These are all correctly classified as drifting.

One observation: "persist-as-single-source-of-truth" is in the *invariant*, but "persist-full vs. compute-live-persist-bounded vs. recompute-on-demand" is in *outcome*. This is the right split: the *fact of persistence as SSOT* is the invariant; the *how* is outcome. This is correctly handled.

**Field representation**: correctly classified as outcome. The ADR is careful not to decree "per-node scalar" — it names that as the open design question. Correct.

### Discussion

Process/outcome split is well-executed. No misclassifications found. The `syn`-as-process-durable entry is slightly fragile but is covered by the pending Amd3 witness seam.

---

## Step 5: The Two New Self-Antigens

### Before
**Hypothesis**: `GlobalConsistencyObstruction` and `StromaIncrementalDrift` should be well-motivated and born-red-constructable.

### Results

**`GlobalConsistencyObstruction`**: "A sheaf gluing-failure — a change breaks a dependent's requirement."

Motivation: Clear. If the sheaf lens is the detector (D7), then the antigen it detects is a gluing failure. Structurally demanded by the sheaf-as-detector claim. Well-motivated.

Born-red-constructable? In principle: construct a node with a contract (provides: X), an edge to a dependent requiring X, introduce a change that removes X — the gluing condition fails. The test is constructable. However: the sheaf machinery is not yet built (it is a follow-on). So the born-red test would test a *concept* that doesn't have substrate yet. The test would be red until the sheaf is built, which is the correct born-red posture. The test defines done for the sheaf implementation. APPROPRIATE.

**`StromaIncrementalDrift`**: "The incremental base diverges from a full rebuild — stale-stroma turned inward."

Motivation: Excellent. This is antigen applying its own detection mission to itself — the failure of incrementality (under-propagation → stale stroma → false-quiet) caught by full-rebuild. The analogy to `RatifiedSpecDriftFromImpl` is precise. Well-motivated.

Born-red-constructable? Yes, straightforwardly: run the incremental updater on a sequence of changes, then run a full rebuild, and assert they match. Make a change that the incremental updater would *miss* (e.g., a change not signaled by git-diff or mtime) — the test fires. This is a canonical property/invariant test. CONSTRUCTABLE.

Both antigens are well-motivated and born-red-constructable. No findings here.

---

## Summary: Findings Ranked

### BLOCKER

**B1 — Amd3 clause-3 witness is a ratification precondition, not a seam.**

Location: Open Seams item 5; E10 (the sovereignty clause).

The ADR itself says "before this ratifies" — the authors know. But it is listed as a seam (future design question) rather than a ratification blocker. ADR-002 Amd3 clause 3 requires both paths scaffolded AND an author-distinct witness confirming the verdict. The verdict here is: `syn` + `cargo-metadata` suffice for the call/data-flow graph. Until that witness runs, the E10 rationale rests on an asserted rather than empirically-rebutted cost/risk/liability rubric.

Actionable: Before ratifying, run the Amd3 clause-3 experiment — scaffold a minimal `syn` + `cargo-metadata` graph against the actual antigen codebase, verify it produces call/data-flow edges, and have an author-distinct reviewer confirm the finding. If it suffices, E10 ratifies cleanly. If it doesn't fully suffice, E10 needs amendment.

The witness might be cheap (the math-feeder noted "`cargo metadata` + `syn` already parsed" — this is a genuine sunk-cost argument that survives scrutiny IF the call-edge extraction actually works at the detail the ADR claims). The witness isn't a design fork; it's an empirical check.

---

### SHOULD-FIX

**S1 — D8 live-edge-tests: scope-bleed in placement.**

Location: Decision clause D8.

D8 describes edge-tests running "continuously as you code — LSP-grade companion." This is an implementation claim in the Decision section. The architectural commitment (edge-test is an antibody on an edge) belongs in Decision. The live-running behavior belongs in "current map" or "What this ADR does NOT do." The "does NOT do" section disclaims the live editor surface, but D8 as written conflates the architectural definition with the live behavior.

Actionable: Split D8 into (a) the invariant architectural statement — "an edge-test is a first-class antibody on an edge; the ADR-066 binding model extends to edge-bindings" — and (b) move the live-running / LSP-grade description to the current-map section under "The live editor surface." This preserves the architectural ratification while honestly deferred the live running.

**S2 — C5 phrasing implies the field is operational.**

Location: Decision clause C5: "the recompute wavefront rides the same diffusion the field uses."

The field is explicitly not built in this ADR. "The same diffusion the field uses" implies an operational field. The actual meaning is "the same graph-traversal topology that a future field implementation will use." This is correct in substance but misleading in phrasing.

Actionable: Rephrase to something like: "the recompute wavefront rides the same graph traversal the field will later run — the dependency-closure over the attributed graph." Removes the false-present-tense of "the field uses."

---

### NIT

**N1 — "Subsumes the sparse `LineageEdge` inheritance DAG as one edge-kind" is accurate but understated.**

Location: Implements/depends-from header; A2 (edge alphabet).

The code baseline confirms `LineageEdge` is antigen's only graph today. The ADR correctly says it subsumes this as one edge-kind. What it doesn't say: the `LineageEdge` struct carries `parent_canonical_path: Option<String>` and `child_canonical_path: Option<String>` — bare string paths, not the fully-qualified collision-free `(qualified-path, structural digest)` identity A1 mandates. So the existing `LineageEdge` is *not* directly usable as a stroma-edge under the new identity scheme — it will need to be migrated or wrapped. This is an implementation detail, but noting it prevents a future builder from assuming `LineageEdge` can be slotted in as-is.

Not a ceremony blocker — the ADR correctly classifies the schema as outcome-drifting. Just worth recording for the implementation.

---

## Supersession / Implementation-Consistency Note

**Clean, with one gap.**

ADR-067 correctly identifies itself as implementing ADR-066 §2's deferred sovereign dependency. The four §2 deferral items are all addressed. Notebook 006's keystone convergence is accurately represented. The ADR does not supersede any ADR (it is additive, sitting below ADR-066 as its required dependency).

The ADR-002 Amd3 relationship is correctly framed: ADR-067 is "landing Amd3's clause-3 empirical rebuttal." The gap is that the rebuttal is structured but not yet witnessed, which is itself an Amd3 process requirement. This is the BLOCKER above.

---

## One-Line Assessment

**Ceremony-ready pending one blocker: the Amd3 clause-3 author-distinct witness (syn + cargo-metadata call/data-flow sufficiency check) must complete before ratification; all else is sound.**

The architecture is coherent, the biology is honest, the process/outcome split is clean, the two new antigens are well-motivated. The draft demonstrates genuine design maturity — the "induced views, not built-in" reframe is structurally important and correctly argued. The only true blocker is the one the ADR itself names but mislabels.

---

## Artifacts

| Source | Role |
|--------|------|
| `research/notebooks/007-adr-067-sovereign-stroma-builder.md` | Subject of this audit |
| `docs/decisions.md` lines 13107–13412 | ADR-066 full text |
| `docs/decisions.md` lines 530–630 | ADR-002 Amd3 full text |
| `research/notebooks/006-adr-066-math-followon-verdicts.md` | Math-feeder verdicts |
| `antigen/src/scan/types.rs:1244` | `LineageEdge` baseline (code-true today) |

## Open Questions

- Does `syn` call-graph extraction at the detail ADR-067 needs actually work at scale on a 200k-line codebase, or does it require semantic analysis (macro expansion, trait dispatch) beyond what `syn` can provide syntactically? This is what the Amd3 witness would surface.
- The workspace/cross-crate scope seam (#3): percolation wants the full dependency closure; is this the same boundary as the `antigen-fingerprint` crate separation? Worth confirming that the stroma scope question doesn't silently reopen the multi-crate composition question from ADR-064.

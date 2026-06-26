# F-ADR-067 — first-principles deconstruction (INWARD, the Sovereign Stroma)

Draft 007 read at R:/antigen-061-self-non-self/research/notebooks/007. Substrate verified on tip c7cf4e9.
Builds on the captured-intent prior (F-self-non-self-deconstruction-v061, F-negative-space-v061).

## Substrate verified (the ADR's load-bearing repo claims — all TRUE)
- `LineageEdge` (`scan/types.rs:1244`) is the ONLY edge antigen has — `#[descended_from]` DAG, carries
  `child_canonical_path`/`parent_canonical_path`. No call/import/dep/data-flow graph exists. ("subsumes
  LineageEdge as one edge-kind" = accurate; the canonical_path machinery is precedent for A1's FQ identity.)
- `item_digest_map` keys by **bare item name** (`diff.rs:105 map.insert(name, digest)`); `item_digest_map_multi`
  (`diff.rs:117`) does **last-write-wins on name collision**. This IS the identity-collision A1 closes. REAL,
  PRESENT defect — A1 is bedrock, not speculation.
- `syn` parse is sunk (parse.rs, diff.rs, self_tolerance.rs, propose.rs all consume `syn::File`). "Marginal cost
  is edges + attributes" = true. No attributed-node struct exists yet.

## Phase 1+2 — clause-by-clause: BEDROCK vs rests-on-assumption

| Clause | Irreducible truth | Verdict |
|---|---|---|
| A1 FQ collision-free identity | bare-name identity DEMONSTRABLY collides (diff.rs:117 LWW) → a node-addressing system that loses a colliding item is unsound | **BEDROCK** (the one clause forced by a present bug) |
| A1 "never coarsened" | the parse is sunk; coarsening the base discards *computed* structure | bedrock-IF "richness is free at margin" — but see F2 (coarsening smuggle) |
| A1 local contract (provides/requires) | NOT forced by present substrate — it's the sheaf lens's input; presupposes D7 | rests-on-assumption (it's a VIEW's need projected onto the base) |
| A2 directed/typed/provenanced edges | impact-backtrace needs DIRECTION; integrity needs PROVENANCE | **BEDROCK** (direction + provenance). WEIGHT is not (F5). |
| A3 lifecycle + digest + integrity | ADR-066 §2 binding-rehome needs lifecycle; field-honesty needs unfabricated edges | **BEDROCK** for integrity; lifecycle is bedrock-given-§2 |
| B4 induced-views / functorial lenses | — | **the central claim — see F1** |
| C5 persist + conservative increment | "per-run ∝ change not size" dissolves the 200k wall; under-propagate = false-quiet-turned-inward | bedrock-as-DIRECTION, but the conservative-direction CHOICE is itself a finding (F3) |
| C6 bidirectional + selective | backward-traversal = detection is sound (reverse edges); receptor-selectivity needs change-type alphabet | bedrock (traversal); selectivity rests on the open change-type alphabet (unbuilt) |
| D7 sheaf lens = detector | gluing-failure = a real contradiction IF local contracts exist (A1) — circular with A1-contract | rests-on-A1-contract (a VIEW, correctly deferred to follow-on, but minted as a born-red antigen HERE — F6) |
| D8 edge-bindings / live edge-tests | "an edge-test is an antibody on an edge" — extends ADR-066 unchanged | rests-on-D7; product-surface, not substrate (correctly excluded from the build, but see F6) |
| D9 "am I OK?" lens | the honest readout the dev needs | decorative-here (it's a view; pure orientation) |
| E10 sovereign / sunk-parse | the Amd3 rebuttal — roll-our-own cheaper AND sovereign | bedrock-IF the parenthetical witness holds (F4 — the unverified load-bearing assumption) |

## The findings (ranked)

**F1 (induced-views — SHOULD-FIX, the core abstraction question). The separation is REAL but the ADR
over-states what it buys, and one direction of it is unproven.**
The base→lens→view layering is a genuine, irreducible separation along ONE axis: *the base is what CHANGE updates;
the lenses are what QUESTIONS read.* That axis is real (the maintenance cost lives in the base; the experimentation
lives in the views). The "no-lock-in / experiment-in-the-views" claim is TRUE **for lenses that the base already
contains the inputs for** — topology (needs only edges+weights), spectral (needs only the Laplacian). It is NOT
yet true for the sheaf lens: D7 needs a **local contract (provides/requires) per node**, and A1 quietly adds that
contract TO THE BASE. So the contract is a lens-input that got promoted into the base — which means the base is NOT
"rich enough to derive the sheaf"; it was made rich *for* the sheaf. That's fine — but it falsifies the clean
"one base, many dial-able lenses, derive everything" story for at least one of the four named lenses. **The honest
version:** the base is rich enough to derive topology/spectral/field as pure functions of (nodes, edges, weights);
the sheaf lens requires a base attribute (the contract) that is itself a design commitment, not a free derivation.
Either (a) name the contract as a base-attribute the sheaf REQUIRES (not "derived"), or (b) move the contract into
the sheaf lens and let it be the lens's own enrichment. The current draft has it both ways. **This is the
elegant-sounding-over-abstraction risk the lead asked about: the functor story is 80% real and 20% load-bearing
hand-wave at exactly the contract boundary.**

**F2 (the Amd3 crime hunt — SHOULD-FIX, ONE real smuggle found). "Full-AST never coarsened" (invariant) vs
"persistence strategy" (outcome) is ALMOST the honest joint — but a coarsening IS smuggled out, just not where
the lead suspected.**
The process/outcome split is mostly clean and is the BEST part of the draft (the §"Process not outcome" block is
genuinely careful — node/edge schema, field representation, persistence strategy all correctly in the
must-drift column). BUT: the invariant says "the base is never coarsened — detail is dialed downstream," and the
outcome column says persistence may be "compute-live-persist-bounded vs recompute-fine-detail-on-demand." **These
contradict.** If persistence is "recompute-fine-detail-on-demand," then the fine detail is NOT persisted in the
base — it is recomputed. A base that must recompute its own fine detail on demand is a base that was COARSENED AT
REST and re-expanded on read. The invariant "never coarsened" is true of the *computation* but the outcome column
quietly admits the *stored base* may be coarsened. **The real joint is not coarsened-vs-persistence; it is
coarsened-IN-COMPUTATION (invariant: never) vs coarsened-AT-REST (outcome: maybe, that's what persistence-strategy
decides).** The draft says this in two places using one word ("coarsened") for two different things, and the
"never coarsened" invariant reads as absolute while the outcome column relativizes it. **This is a genuine
Amd3-class confusion — an invariant ("never coarsened") that an outcome silently rebuts.** Fix: split the word.
The invariant is "the *computation* runs at full AST resolution; no view is computed from a lossy base." The
*at-rest representation* (persist-full vs persist-bounded-recompute-detail) is the outcome. Then "never coarsened"
stops being a decree that the outcome contradicts.

**F3 (conservative-direction — NIT-rising-to-SHOULD-FIX). "Conservative = over-propagate" is asserted as the safe
direction, but the draft's OWN logic makes it the EXPENSIVE-and-still-unsafe direction at the field layer.**
C5 says under-propagate = stale = false-quiet (the failure turned inward) so be conservative = over-propagate.
Sound for DETECTION. But over-propagation at the FIELD layer (the diffusion the recompute-wavefront rides) means
the loudness field gets recomputed over a wider neighborhood than changed — which **dilutes** nothing but COSTS
the percolation/RD recompute over nodes that didn't move. The conservative choice is correct for soundness and the
draft is right to name under-propagate as the inward-false-quiet — but "over-propagate is merely slow" understates
it: at the field layer over-propagation is the 200k-wall coming back through the side door (the wavefront that
"rides the same diffusion" can, on a hub change, be the WHOLE graph — percolation's blast-radius IS unbounded by
construction for a hub). The full-rebuild-witness (StromaIncrementalDrift) catches DIVERGENCE but not COST. **The
real seam: a hub change's conservative wavefront = global recompute, which is exactly the cost the incremental
model promised to avoid.** Not a blocker — but "per-run cost ∝ change" is FALSE for hub-changes, and the draft
states it as universal. Name the exception.

**F4 (the Amd3 empirical rebuttal — BLOCKER for ratification, already self-flagged). E10's "syn + cargo metadata
genuinely suffice for the call/data-flow graph" is the ONE unverified load-bearing assumption, and the whole
sovereignty case rests on it.**
The draft self-flags this (the parenthetical "Pending an author-distinct witness...") — good. But it is
under-weighted: it is filed as an "open seam," when it is actually the **keystone of the keystone.** If `syn`
ALONE cannot resolve a call edge (it cannot — `syn` is syntactic; `foo.bar()` does not resolve `bar` to a
definition without name-resolution / type-inference, which is rust-analyzer's actual hard part), then either (a)
the call-graph is approximate (syntactic call-name matching, ambiguous on overloads/traits/generics), or (b)
antigen must build name-resolution itself (a large hidden cost the Amd3 "cheaper" rebuttal omits), or (c) it
composes the very tool E10 says it won't. **The rebuttal "roll-our-own is cheaper because the parse is sunk" is
true for the SYNTACTIC edges (import, inheritance, structural) and FALSE-or-unproven for the SEMANTIC edges
(call, data-flow) that need resolution.** The math-feeder (notebook 006) even says call-graph is "cheap — cargo
metadata + syn" — that claim is inherited unexamined from the math voyage and propagated here. **This is the same
"borrowed assumption" pattern the deconstruction exists to catch: a cheapness-claim inherited from an upstream doc
without term-by-term verification of WHICH edges syn can actually resolve.** Cannot ratify E10 until the witness
distinguishes syntactic-edges (sound, sovereign, cheap) from semantic-edges (resolution-hard; the honest cost).
The honest fallback that PRESERVES sovereignty: type the call-edge as `provenance: derived-from-syn-syntactic`
(an approximate, honestly-labeled edge) — which is co-native with the rest of the integrity model and does NOT
require building rust-analyzer. **That option isn't in the draft and it should be — it dissolves the blocker.**

**F5 (edge WEIGHT in A2 — NIT). A2 lists weight as a base attribute "the §4 field diffuses over," but the
Process/Outcome block correctly puts conductance-weights in the OUTCOME column.** Minor internal tension: A2 reads
as if weight is a first-class base property; the cut "type+provenance the edges now, weight them at the field
layer" (named in Open seams) is the right one. Resolve by stating in A2 that edges carry an OPTIONAL weight whose
VALUE is the field-layer's outcome — type the slot in the base, set the value in the lens. (One-line fix; the seam
already knows this, A2's prose just doesn't match it.)

**F6 (born-red antigens minted on unbuilt lenses — SHOULD-FIX, decidability-honesty). The ADR mints two born-red
self-antigens (`GlobalConsistencyObstruction`, `StromaIncrementalDrift`) but one of them defends a lens this ADR
explicitly does NOT build.** `StromaIncrementalDrift` is sound and in-scope (it defends THIS ADR's own
incremental-vs-rebuild invariant — born-red, correct). But `GlobalConsistencyObstruction` is a **sheaf
gluing-failure** — and the sheaf lens (D7) is, by the draft's own "What this ADR does NOT do," a follow-on layer.
Minting a born-red antigen for a detector you haven't built registers a **dangling defense** — the exact
`defended_by(undefined-class)` pattern from the v06 dogfood memory (a name-carrying marker for a class that has no
runnable detector yet). It compiles, it reads as covered, but nothing fires it. **Fix: `StromaIncrementalDrift`
born-red NOW (its detector — the full-rebuild-witness — IS this ADR); `GlobalConsistencyObstruction` deferred to
the sheaf-lens follow-on ADR that actually builds the detector.** Don't born-red an antigen whose detector is a
later ADR.

**F7 (Aristotle's-recursion framing — load-bearing, NOT decorative — but for a reason the draft doesn't state).**
The lead asked: is "the stroma is the lattice antigen lives from" load-bearing or decorative? **Load-bearing —
and here is the irreducible why the draft gestures at but doesn't nail.** The prior deconstruction (F-self-non-self)
found antigen has TWO disjoint selves (identity-self = digest-drift; tolerance-self = clean-corpus) that share NO
object, and a captured-intent void that both project from. **The stroma is the first object both selves can
address.** identity-self's `structural_digest` becomes a stroma-NODE attribute; tolerance-self's clean corpus
becomes a stroma SUBGRAPH (the spared region). For the first time the two selves live in ONE namespace (the
stroma-node id). That is what "the lattice antigen lives from" structurally MEANS, and it is not decorative: it is
the substrate that could finally UNIFY the two selves the prior F-finding proved disjoint. **The draft says
"antigen's structural-self" and "Aristotle's recursion made concrete" but never connects it to the two-selves
finding — connecting it is what turns the framing from poetry into the load-bearing claim.** Recommend the ADR
name this: the stroma is the shared namespace in which identity-self and tolerance-self stop being disjoint.

## Phase 8 — forced rejection (what the voids imply MUST also exist)
- IF the base is NOT "rich enough to derive everything" (F1) → there exists a privileged SET of base-attributes
  that are lens-REQUIREMENTS masquerading as base-properties (the contract is the first; F1). The void's shape:
  the "functor" is partial — some lenses are pure functions of the base, others CO-DEFINE the base. That co-
  definition is a real structure the draft hasn't named: a **lens-requirement back-edge** (the sheaf TELLS the
  base it must carry contracts). The induced-views architecture is actually a FIXPOINT (base and lenses co-
  determine), not a one-way functor. If that's true, the "experiment freely in the views, base stays stable"
  promise has an asterisk: adding a NEW lens-KIND can demand a new base-attribute (a base migration — the one
  thing "accrete never migrate" forbids). **The missing piece the void implies: a discipline for "this new lens
  needs a base-attribute the base doesn't have" — is that an accretion (safe) or a migration (forbidden)?** The
  draft's "open attribute set, accrete never migrate" is the answer ONLY IF every future lens-requirement is
  additive. Prove that, or the functor story has a migration trapdoor.
- IF "never coarsened" is FALSE at rest (F2) → the base has a RESOLUTION it's stored at, distinct from the
  resolution it's computed at. Two numbers, not one. The draft collapses them into "full-AST." Splitting them is
  the F2 fix and the void confirms it: there MUST exist a stored-resolution parameter the persistence-strategy
  outcome sets.
- IF syn does NOT suffice for semantic edges (F4) → there exists a PARTITION of the edge alphabet into
  syntactically-resolvable (sovereign, exact) and semantically-resolvable (needs inference; approximate-or-
  composed). That partition is the honest sovereignty boundary, and provenance ALREADY has the slot to carry it
  (`derived-from-syn` vs would-be `derived-from-resolution`). The void hands the fix: the provenance field IS the
  place the syntactic/semantic honesty lives.

## Irreducible core (if stripped to bedrock)
antigen maintains a sovereign, persisted, **fully-qualified collision-free** node-and-**directed-provenanced-edge**
graph computed from its own sunk `syn` parse, updated conservatively by change, witnessed by full-rebuild. That is
the whole bedrock. Everything else — induced-views functor, contracts, sheaf, edge-tests, weights, the field — is
either a VIEW (correctly deferred) or an assumption that needs the F1/F4 verification. The bedrock is sound and
forced by a present bug (the diff.rs:117 collision). The architecture around it is mostly honest with two real
smuggles (F2 coarsened-word, F6 dangling born-red) and one keystone-unverified assumption (F4 syn-suffices).

## Verdict
**Ratifiable AFTER: (1) F4 — split the edge alphabet into syn-syntactic (sovereign/exact) vs semantic (resolution-
hard) and let provenance carry the honesty, dissolving the Amd3-rebuttal blocker; (2) F2 — split "coarsened" into
computation (invariant: never) vs at-rest (outcome: persistence decides); (3) F6 — defer GlobalConsistencyObstruction
to the sheaf-lens ADR that builds its detector.** The bedrock (FQ identity + directed provenanced edges from the
sunk parse, conservative-increment + rebuild-witness) is sound and bug-forced. The induced-views separation is
real along the change-vs-question axis but is a co-determining fixpoint, not a one-way functor, at the contract
boundary (F1). The stroma-as-shared-self-namespace (F7) is load-bearing and worth naming explicitly.

## Waking notes
Routed to navigator. Open threads: F1 fixpoint-vs-functor (does a new lens-kind ever force a base MIGRATION? — the
"accrete never migrate" trapdoor); F4 needs the author-distinct syn-suffices witness (recommend it test ONE real
call-edge through a trait-object to see if syn resolves it — it won't, which proves the partition). The
two-selves-unification (F7) connects directly to the captured-intent void — the stroma may be where C2 ("name the
bridge between the two selves") finally lands as code, not just an ADR-draft note.

# Lab Notebook 013: the-stroma-remembers — Observer Record

**Date**: 2026-06-27
**Author**: the-stroma-remembers--observer
**Branch**: 0.6.1-self-non-self (worktree: R:/antigen-061-self-non-self)
**Status**: Active (live — updated as the wave progresses)
**Depends on**: 009 (intent kernel + coordinate system), 011 (0.7 frame expedition design), ADR-066 (antibodies), ADR-067 (sovereign stroma), 012 (v0.7 PASS-1 dream wave record)

---

## Context & Motivation

This notebook is the scientific conscience layer for the expedition `the-stroma-remembers`. The v0.7 PASS-1 dream wave (notebook 012) produced the frame. This expedition has a different charge: it is **simultaneously a dream wave AND an experiment wave** — the crew does not just imagine but prototypes, benchmarks, and writes results back into charters as evidence.

**The expedition's four-dimensional charge (from launch.md):**
1. The sense/self organism unbounded — what immune machinery from real biology have we NOT brought in?
2. The stroma itself expanded — faster/more live/richer, what packages/techniques make it richer?
3. The value surfaces — to LLMs, humans, platforms, telemetry
4. The embedding principle as discipline — does everything embed, derive, or reason-from the stroma?

**Observer's mandate:** Record what IS, not what we hope. Distinguish what was prototyped-and-measured from what was dreamed-and-hoped. Flag experiments that haven't shipped yet. Flag claims that outrun their evidence.

**Key prior state (inherited from 012 + prior memory):**
- The stroma is GREENFIELD (codebase-scout notebook 012 finding: `antigen-stroma` crate does not exist, zero stroma lines in workspace)
- `ANTIGEN_OWNED_ATTRS` is confirmed incomplete (28 on-disk; memory flags 9 missing; `antigen-dx-dogfood` has a complete finding: `atk-digest-1-antigen-owned-attrs-incomplete`)
- ADR-067 Open Seam §1 (r-a interface capability fork) is the hard pre-ratification BLOCK from the prior expedition; the engineer-researcher has now found a third path (SCIP)
- The `tools/edges.json` (649.9K Python regex heuristic call-graph) is antigen's own altered-self / dual-source-of-truth — both a migration oracle AND a structural problem to be superseded

---

## T=0 State (2026-06-27 03:20–03:31 UTC)

### Expedition launch

The expedition `the-stroma-remembers` was prepared by team-lead at 03:20 UTC. Team config registered 12 roles. At T=0, the expedition had 3 islands:
- `stroma-base-substrate` (adversarial-expansionist)
- `stroma-machinery-prior-art` (expansionist)
- `value/stroma-as-agent-grounding-at-generation` (value-finder, explicit completion)

Between 03:25 and 03:31 UTC — a 6-minute burst of activity — the crew produced **50 camp activity events** across all roles, seeding 24 islands total. No islands closed in this burst; this is all early-wave seeding.

### Island inventory at T=6min

**Open (signers required):**
- `stroma-base-substrate` — adversarial-expansionist (note deposited, not yet signed)
- `stroma-edge-tracer` — engineer-researcher (note deposited, not yet signed)
- `stroma-machinery-prior-art` — expansionist (signed COMPLETE; noted below)
- `immune-gap-map-unmapped-machinery` — math-researcher (science-researcher note deposited)
- `sys/stroma-coupling-map` — systems-thinking (note deposited)
- `sys/embed-derive-reason-coupling-gradient` — systems-thinking (note deposited)
- `sys/feedback-loop-atlas` — systems-thinking (note deposited)

**Note-only dream islands (no signer required):**
- `dream/affinity-maturation-runtime`
- `dream/complement-inverted-default`
- `dream/complosome-self-budget`
- `dream/federated-structural-antibody`
- `dream/generation-against-live-substrate`
- `dream/specs-as-antibodies-spec-rot`

**Value islands (explicit completion):**
- `value/stroma-as-agent-grounding-at-generation`
- `value/stroma-answers-negative-space-queries`
- `value/live-blast-radius-as-the-human-surface`
- `value/stroma-as-training-signal-factory`
- `value/stroma-as-open-query-platform`
- `value/runtime-facts-become-stroma-edges`
- `value/stroma-as-open-query-platform`
- `value/federated-structural-lesson-transfer`
- `value/time-travel-and-decay-archaeology`
- `value/checkability-capacity-as-a-product`
- `value/supply-chain-as-native-stroma-query`
- `value/ai-generated-code-provenance-organ`

**Important:** `stroma-machinery-prior-art` read as `[complete]` in `camp status` output. This is the only completed island at the time of this notebook entry.

---

## Hypotheses Before Results

### Hypothesis 1 (stroma base substrate): graph vs relational base

**The default under test (ADR-067):** The stroma's base is a full-AST attributed GRAPH (nodes+edges). Induced views (topology/sheaf/field) are derived from it. Incrementality is hand-rolled (frozen-snapshot, two-wavefront, could-have-changed-closure).

**The adversarial-expansionist's challenge:** Is a GRAPH the right base, or is there a fundamentally different best way? The adversarial-expansionist argues: CodeQL and Glean — the two production-grade systems doing exactly antigen's job — chose RELATIONAL FACTS + DERIVED PREDICATES as their base. The graph in those systems is a DERIVED VIEW over relations, not the substrate.

**The term-by-term mapping (from the adversarial-expansionist's note [5c3664c4]):**

Every ADR-067 stroma concept places cleanly in the relational model:
- node → entity fact: `Item(id, kind, path)`
- edge → relation fact: `Call(caller,callee)`, `Flows(a,b)`
- induced view → a derived predicate (all views are UNIFORM — not a privileged second layer)
- edge-provenance tier → source relation tag + stratification (first-class in datalog)
- frozen-snapshot + atomic publish → a transaction/epoch (standard DB primitive)
- forward wavefront (freshness) → semi-naive evaluation (PROVEN, not hand-rolled)
- backward wavefront (detection) → recursive query / magic-sets (PROVEN)
- could-have-changed-closure + storm-impossible → DRed / differential dataflow (PROVEN, parallel, decades-mature)
- `StromaIncrementalDrift` self-antigen → may be UNNECESSARY if the engine's correctness theorem covers it

**The adversarial-expansionist's three findings:**
1. The graph is not the base — it's the first induced view. The design's own "everything is an induced view" principle, taken one level deeper, demotes the graph.
2. Hand-rolled incrementality is reinventing a solved, proven problem. The stroma's hand-rolled wavefronts are a from-scratch re-derivation of incremental datalog.
3. Provenance-as-tier may be a solved problem. 009 Part 6's closed-alphabet discipline IS stratified datalog.

**Where the graph-base genuinely wins (the honest counter):** The FIELD / reaction-diffusion / manifold views (009 §4, ADR-067 map) are continuous-geometry computations (percolation, conductance, diag(d)>0 fixed-points). Datalog is relational/discrete and does NOT natively host a PDE-on-a-manifold. The question is whether the field is a base-citizen or an induced view computed by an external numeric kernel reading the relational base.

**The proposed experiment:** Take one real edge-kind (call-graph) on a small real crate, build it BOTH ways in scratch — (a) hand-rolled graph+wavefront, (b) datalog encoding on an off-the-shelf incremental engine (Souffle / ascent / differential-dataflow / crepe) — and measure: incremental update latency on a 1-line change, LOC/complexity to express the backward "who-depends-on-me" query, whether StromaIncrementalDrift guard is needed under (b).

**Observer pre-result assessment:** The adversarial-expansionist's challenge is the most structurally consequential finding of the expedition so far. The term-by-term mapping is precise (not metaphorical). The CodeQL/Glean evidence is production-grade. This is not a gentle alternative suggestion — it's a genuine "why didn't we choose this?" challenge that the ADR-067 ratification process must answer. The field/manifold counter is the adversarial-expansionist's own honest bounding; it is the strongest counter-argument and it is real. The outcome depends on whether the field is a first-class primitive or a derived view — an empirical question, not an armchair one.

**What would break the challenge:** If the field computations are genuinely first-class (not derivable from a relational base), then a pure relational substrate requires an embedded numeric kernel alongside it — a hybrid architecture that may cost more in integration than the hand-rolled graph approach. The experiment proposed above is the right tool to test this.

### Hypothesis 2 (stroma machinery prior art): four mature engineering families

**The expansionist's thesis:** The stroma's required shape — a queryable, incrementally-maintained, multi-source attributed graph that must be fast (incremental recompute), live (change-driven), and rich (anything embeds/derives/reasons) — is shared by four production-grade engineering families. The stroma is an instance, not a novelty.

**The four families (from the expansionist's note [a4dcfe7b], island `stroma-machinery-prior-art`):**

*Family 1 — Salsa (rust-analyzer's own incremental engine):* The most directly applicable. Every major ADR-067 stroma term maps exactly to a salsa concept. Key insight: dependency-distance Scale (009 Part 6c, "my code → deps → transitive → ecosystem") IS salsa's durability levels (volatile/normal/durable). The stroma's freshness-tier hierarchy is already modeled in salsa's architecture. **Critical inherited primitive:** since antigen require-installs r-a and r-a is built on salsa, the resolution layer already brings salsa's incrementality into antigen's build — antigen only needs its own incremental engine for the sovereign immune lattice (attributes/contracts/field), a much smaller surface.

*Family 2 — Glean (Meta's code-fact DB):* Concrete realization of frozen-snapshot/atomic-publish. Glean's hide-and-stack (stacked immutable databases) is a shipped, +7%-overhead, O(changes) realization of exactly ADR-067's invariant. Ownership-conjunction-visibility (a derived fact auto-invalidates when any source unit is hidden) IS antigen's backward-detection + sheaf-gluing fused into one mechanism.

*Family 3 — Datalog/DDlog/Soufflé:* The induced-views layer. CWE-as-data-flow (the unified-corpus grounding) IS a Datalog/taint workload. The intent DSL (009 Part 3, "few primitive gates → unlimited compositions") is a Datalog program over the stroma. The strongest anti-tech-debt flag: do NOT hand-roll a bespoke view-maintenance engine — the induced-view layer is a datalog engine.

*Family 4 — Differential Dataflow:* The substrate under DDlog. Partially-ordered timestamps for coalescing concurrent changes (ADR-067's "concurrent changes queued + coalesced into the next snapshot") IS differential dataflow's timestamp-batching — formally proven.

**Observer pre-result assessment:** The expansionist's mapping is the strongest piece of prior-art research produced so far. Every ADR-067 term places in an existing, proven system. The "borrowed primitive" annotations (per-family) are load-bearing — they are specific primitives antigen should inherit rather than re-derive. The salsa primitive (version vector with per-durability-tier skips) directly matches the stroma's dependency-distance Scale.

**Crosswalk with the adversarial-expansionist's challenge:** These two island notes converge: the adversarial-expansionist says "graph-as-base is probably wrong" and the expansionist says "datalog/DDlog is the proven induced-views engine." Together they push toward: relations-as-base + DDlog/differential-dataflow as the incremental engine. The experiment (Hypothesis 1) would test this directly.

### Hypothesis 3 (r-a interface): SCIP as a third path resolving the capability fork

**The ADR-067 pre-ratification BLOCK:** The r-a interface is a hard capability fork — LSP (call/name/type edges, bounded, but no data-flow) vs `ra_ap_*` (data-flow, but unbounded adapter cost: weekly-churning pre-1.0 API with no stability contract).

**The engineer-researcher's finding (island `stroma-edge-tracer`):** There is a THIRD PATH that ADR-067 missed: `rust-analyzer scip <path>` and `rust-analyzer lsif <path>` — batch CLI subcommands on the require-installed r-a that emit resolved definition+reference data in a standard on-disk format (SCIP = protobuf with symbol roles).

**Why this resolves the fork for call/reference edges:**
- It is the bounded path (stable-ish CLI contract + versioned wire format)
- It yields resolved edges (same r-a resolution engine, same RootDatabase/salsa)
- No LSP session lifecycle complexity
- No `ra_ap_*` version churn in antigen's Cargo.toml
- Adapter surface: parse a protobuf schema (far more stable than the ra_ap_* Rust API)

**The 3-tier ladder that results:**
```
syntactic (syn, already have)  <  resolved-refs (SCIP batch, NEW cheap path)  <  mir-exact (rustc MIR)
```
This matches ADR-067 clause 3b's tier alphabet exactly: `syntactic < resolved < mir-exact`.

**The quantified adapter cost:** `ra_ap_ide @ 0.0.338` — 275 versions, ~weekly +1 cadence, pinned at `=0.0.338` (exact-version locking, no range deps). The "unbounded adapter cost" now has a number: a +1 every ~7 days with no stability contract.

**The caveat to verify (experiment in flight at time of notebook writing):** SCIP occurrence granularity is symbol-at-range, not "caller-fn → callee-fn" directly. Reconstructing the call EDGE requires mapping each reference-occurrence's enclosing definition (the caller) to the referenced symbol (the callee) — doable from the occurrence ranges + definition ranges in the same document, but it is a reconstruction step, not a free edge list. The engineer-researcher reported: "scip index generation running on antigen's own workspace; edge-reconstruction + heuristic-diff next."

**Observer pre-result assessment:** The SCIP finding is the most practically impactful finding of the expedition so far. It directly resolves the ADR-067 Open Seam §1 pre-ratification block for the call/reference tier without the `ra_ap_*` version-churn problem. The experiment (reconstruction fidelity vs the Python heuristic) is the right validation. The data-flow / MIR-exact tier remains a separate pipeline — this doesn't solve everything, but it solves the most important piece.

**What could break it:** If SCIP's occurrence granularity cannot be lifted to call-graph edges without prohibitive reconstruction cost, or if SCIP's coverage (esp. for trait dispatch / dynamic dispatch) is materially worse than what r-a's LSP would provide. The experiment measures this.

### Hypothesis 4 (seven unmapped immune clusters): what is NOT yet in antigen

**The science-researcher's gap-map (island `immune-gap-map-unmapped-machinery`):**

Seven clusters of real immune machinery that are near-absent in the charters:

1. **Innate pattern-recognition alphabet (PRR/PAMP/DAMP):** The sharpest find — a PRR matches a molecular PATTERN (a structural shape), not a specific antigen-instance. This IS the fingerprint engine at a higher structural level. PRR classes become sensor LOCI: TLR=membrane/surface (item-signature, attr, derive), NLR=cytosolic (body_calls/body_contains_macro), CLR=lectin/shape-digest (structural-cluster matching). Verdict: EMBED — PRR-class becomes a first-class facet on every fingerprint, giving the catalog a sensing-modality taxonomy it lacks.

2. **Complement effector tail (C3b/MAC/cascade):** Opsonization landed the tagging (C3b-as-tag) but not the cascade (amplification) nor the lytic terminal (MAC). Verdict: DERIVE — the cascade is a forward wavefront over resolved edges. Blocked on stroma edges.

3. **Antibody effector repertoire (neutralization/agglutination/ADCC/opsonization/isotype):** Antibodies landed as PROOF (ADR-066) but their effector FUNCTIONS did not. Isotype/class-switch = the same antibody with different effector context = antibody whose response-axis routing changes by deployment site. Verdict: REASON-FROM (the response axes in 009 are the effector-function selector).

4. **Barrier/mucosal/microbiome:** The FIRST line of defense, near-absent in charters. The commensal/symbiont = the self/symbiont/pathogen ternary (011 open-Q). Verdict: mostly REASON-FROM + the barrier = a Scale boundary (009 Part 6 membrane-is-fractal).

5. **Cell trafficking (chemotaxis/extravasation):** How the response MOVES to the danger. The stroma-as-tissue named the manifold but not the traffic ON it. Verdict: DERIVE (the field dynamics — blocked on the field representation, which ADR-067 defers).

6. **Checkpoint/peripheral tolerance (PD-1/CTLA-4/Treg):** The BRAKES. Central tolerance (thymic/AIRE) landed; PERIPHERAL tolerance (brakes on already-mature responders) is thin. Key distinction: tolerance has TWO sites — central (at catalog-admission) and peripheral (at the firing-site). Verdict: REASON-FROM — checkpoints = a suppression/anergy refinement, keyed to context not time.

7. **Trained immunity / innate memory:** ABSENT entirely. Innate cells epigenetically/metabolically reprogrammed to respond faster/stronger to a re-seen shape — memory WITHOUT the adaptive/antibody machinery. This is the deepest find: structural (innate/fingerprint) layers can carry MEMORY (a self-perpetuating reprogrammed state). Directly maps to the constructor-theory charter's resilient-self-perpetuating-information criterion, applied to the catalog itself.

**Observer pre-result assessment:** Clusters 1 (PRR-as-sensor-locus) and 7 (trained immunity as innate memory) are the most generative. Cluster 1 gives the fingerprint catalog a structural taxonomy it currently lacks — something verifiable against the actual catalog (do fingerprints track LOCUS? they don't in current code). Cluster 7 is the deepest conceptual find: if the innate/structural detection layer can carry memory (as real immunology says), then the fingerprint catalog itself is a knowledge-constructor (per the constructor-theory charter), and its learning/forgetting dynamics have real stakes. This challenges any architecture that treats the fingerprint catalog as purely static.

The embedding-principle verdicts (embed/derive/reason-from) are the science-researcher applying the expedition's own discipline. They are clean and follow from the stroma dependency. Most unmapped machinery is DERIVE or REASON-FROM, blocked on resolved edges — which strengthens the P1 (resolved-edges) finding from the systems-thinking leverage map.

### Hypothesis 5 (systems topology): the stroma as coupling medium

**The systems-thinking finding (island `sys/stroma-coupling-map`):**

The central structural fact: the stroma is the COUPLING MEDIUM, not a layer. Organs do NOT couple to each other — they couple to the STROMA'S STATE. This is a star topology (blackboard/stigmergy architecture). The stroma's primitives ARE the system's coupling rules.

**Key findings:**
- Every organ-to-organ system effect is MEDIATED, never direct. The stroma's primitives are the leverage point that sets the gain on every downstream loop (Meadows #5 — rules of the system).
- The snapshot-vs-live keystone (ADR-067 C.5) is the DELAY that makes the medium SAFE. It converts a potentially oscillating mesh into a discrete-time synchronous system. This is a designed-in delay (Meadows #11 buffer) that BUYS stability rather than causing oscillation — the exception in systems dynamics that proves the rule.
- The hourglass creates a REINFORCING loop (R1: adoption → shared-richness → adoption-payoff → more adoption) paired with a necessary BALANCING loop (B1: standing sampled parity surveillance). They ship together or the architecture is unsound.

**The embed/derive/reason-from trichotomy as a coupling-strength gradient (island `sys/embed-derive-reason-coupling-gradient`):**
- EMBED: tightest coupling (shared-truth, propagates on next snapshot, parity oracle earns its keep)
- DERIVE: medium coupling (recomputable, self-heals on rebuild, safe for experimentation)
- REASON-FROM: loosest coupling (reads the stroma, writes conclusions elsewhere, gated by authority)

**The leverage terrain ranking from systems-thinking:**
- P0 (identity/digest): prerequisite floor — get it right, catastrophic if wrong
- P1 (resolved edges): highest qualitative fan-out — ALSO the hardest pre-ratification block
- P2 (provenance-tier): highest design-leverage — information-flow primitive (Meadows #6)
- P3 (snapshot-discipline): defensive total — don't weaken it
- P4 (contract-as-node-data): goal-shaping lever — sets the proof-climb ceiling

**Observer assessment:** The systems-thinking fan-out map makes the critical-path argument explicit: P1 (resolved edges) has the highest downstream value AND is co-located with the expedition's hardest feasibility block (r-a interface). The highest-leverage primitive and the highest-risk primitive are the SAME primitive. This confirms the build sequence priority: resolve the r-a fork first. The SCIP finding (Hypothesis 3) is therefore the highest-priority near-term result.

### Hypothesis 6 (value surfaces): the stroma's query interface as the joint need

**The value-finder's connect-the-needs notice ([04a10778]):**

Five value islands + the platform-fleet + runtime-afferent charters ALL serve ONE unmet need nobody names alone: THE STROMA WANTS A QUERY INTERFACE AS ITS PRIMARY PRODUCT SURFACE.

Every named value surface is a query type:
- agent-grounding: "who calls this fn I'm about to change?" — a query
- negative-space queries: "where is there NO validation before this sink?" — an absence query
- open-query-platform: any team authors a bespoke view — an induced-view definition
- runtime-edges: "which path actually executed vs which path was predicted?" — a runtime-augmented query
- time-travel: "how did this fn's dependents evolve since the last scan?" — a temporal query
- checkability-map: "which items have a declared intent with no check-surface?" — an IntentNotEnforceable query

**The value-finder's finding:** The stroma-builder must carry a query-engine primitive FROM DAY ONE, not bolt one on. This inverts the embedding-principle: the value isn't what embeds IN the stroma, it's what can be ASKED OF it.

**Specific value islands and their unmet needs:**

*`value/stroma-answers-negative-space-queries`*: Every code-search / RAG / grep tool answers only POSITIVE queries ("where is X?"). Nobody can answer NEGATIVE queries — "where is there NO validation before this sink?", "which pub fns cross the boundary with an unsanitized property?". These are ONLY answerable over an EXACT attributed graph, never over text-embeddings. The absence answer is only TRUSTWORTHY over resolved edges; over heuristic edges it gives a "confidently-wrong located answer, worse than none."

*`value/stroma-as-agent-grounding-at-generation`*: The stroma IS the thing RAG is a bad approximation of: an EXACT, RESOLVED, LIVE, multi-source attributed graph. The single highest-frequency stroma consumer in the LLM age is not antigen's own reasoner — it's every external agent editing the code.

*`value/stroma-as-open-query-platform`*: If the view-derivation is an open, documented capability, the fleet stops being 11 fixed products and becomes an ECOSYSTEM — anyone can define a new induced view.

**Observer assessment:** The value-finder's connect-the-needs synthesis is the clearest statement of the value thesis in the expedition. The observation that every value island reduces to a query type is genuinely novel and has architectural consequences: if the primary value surface IS a query interface, then the stroma's design must answer "what query language does it expose?" from day one, not treat it as a later-wave product decision. This connects to the adversarial-expansionist's datalog challenge: the induced-view architecture of DDlog is precisely a declarative query language with incremental execution.

---

## Live Timeline

### T=0 to T+6min (03:25–03:31 UTC)

All 50 camp activity events occurred in this single 6-minute window. The burst pattern means the crew spawned roughly simultaneously and each role deposited its primary output. No synchronous coordination is visible; the crew is working purely from launch.md and the charter substrate.

**What arrived, role by role:**

**Expansionist:** Produced the four-family prior-art mapping on `stroma-machinery-prior-art` (COMPLETE — only completed island in this burst). Three field-track saves: #salsa, #code-property-graph/#stack-graphs, #datalog/#differential-dataflow, plus saves for #neuroscience/#predictive-coding and #biology/#ecm-remodeling.

**Adversarial-expansionist:** Deposited the graph-vs-relational challenge on `stroma-base-substrate` (extensive note, not yet signed). Three field-track saves with #base-substrate-inversion tag. Seeded `dream/complement-inverted-default` and `dream/complosome-self-budget` as note-only islands.

**Engineer-researcher:** Seeded `stroma-edge-tracer` with the SCIP finding. Two field-track saves (#stroma #incremental, #stroma #resolution). Seeded other note-only dream islands.

**Science-researcher / math-researcher:** Seeded `immune-gap-map-unmapped-machinery` with the seven-cluster gap-map.

**Systems-thinking:** Seeded `sys/stroma-coupling-map`, `sys/embed-derive-reason-coupling-gradient`, and `sys/feedback-loop-atlas` with detailed system-topology analyses. Produced links between the systems islands.

**Value-finder:** Seeded 9+ value islands with detailed unmet-need analyses. Produced the connect-the-needs synthesis notice and two handoff questions to think-big-dreamer.

**Outsider:** Deposited 8 questions routed to specific roles: 
- to engineer-researcher: "is 'intent-vs-reality' actually 'intent-vs-stroma-model-of-reality'? what happens to the framing when the model is wrong?"
- to systems-thinking: "platforms are VIEWS — but what does VIEW mean here? are they SQL views (cheap) or full products with their own UX/persistence?"
- to science-researcher and adversarial-expansionist: various challenges on the constructor-theory framing and sensor gaps
- to think-big-dreamer: 3 questions on the query interface and the biggest version of various concepts

**Naturalist:** Three saves/notices: a counterfactual-layer notice (routed to science-researcher and adversarial-expansionist), a constructor-theory save on the counterfactual layer, and a notice about predictive-coding as a deep structural rhyme for the stroma (routed to expansionist).

**Think-big-dreamer:** Seeded 6 note-only dream islands. Posted an arrival notice. The think-big-dreamer appears to have arrived later in the burst (03:30 notices vs 03:25 expansionist) and seeded note-only dreams rather than fully worked-out notes.

---

## Peer-Review Assessment

### Claims that are well-grounded

**1. SCIP as a third path (engineer-researcher, `stroma-edge-tracer`):** The finding is verifiable — `rust-analyzer scip <path>` exists; the ra_ap_* version churn rate (275 versions, +1/~7 days) is a real measurement from crates.io. The caveat (SCIP occurrence granularity requires reconstruction) is named honestly. This is the best-grounded empirical find in the expedition so far.

**2. The four-family mapping (expansionist, `stroma-machinery-prior-art`):** Each family mapping uses named terms and specific architectural concepts. Salsa in particular: r-a is built on salsa and antigen require-installs r-a — the inheritance is real, not hypothetical. CodeQL/Glean are production systems with public documentation.

**3. The systems-topology coupling map (systems-thinking, `sys/stroma-coupling-map`):** The star-topology/blackboard observation is structurally correct from ADR-067's architecture. The R1+B1 paired-loops observation (hourglass reinforcing loop + parity oracle balancing loop must ship together) is a load-bearing consequence of ADR-067's design that deserves to be named explicitly in the ADR.

**4. The embed/derive/reason-from as coupling-strength gradient (`sys/embed-derive-reason-coupling-gradient`):** The three-tier coupling analysis (tight/medium/loose, with different blast-radius and staleness failure modes) is a genuine architectural contribution. The claim that "reason-from" coupling is the safest regime for LLM consumers is directly grounded in the "gated by authority" design (the no-self-witness invariant from 009).

**5. Negative-space queries as a genuinely new value surface (`value/stroma-answers-negative-space-queries`):** The claim that no existing tool can answer absence queries over a resolved attributed graph is verifiable — grep/RAG/text search are positive-only; CodeQL can do it but only with a static, not live, graph. The observation that heuristic-edge absence queries are "confidently-wrong, worse than none" is a direct corollary of ADR-067's provenance-tier discipline.

### Claims that need stronger grounding

**1. "Graph is not the base — it's the first induced view" (adversarial-expansionist):** The structural argument is rigorous. The claim that a relational base is BETTER for the stroma is directionally supported by CodeQL/Glean evidence but has a legitimate counter: the field/manifold computations. The experiment (proposed, not yet run) is the right test. This is a pre-result claim; the direction is plausible but not yet proven.

**2. "PRR-class becomes a first-class facet on every fingerprint" (science-researcher):** The PRR-as-sensor-locus analogy is well-drawn, but the claim that it "gives the catalog a sensing-modality taxonomy it lacks" requires verifying against the actual fingerprint catalog structure. Do fingerprints currently have a sensing-locus field? The codebase-scout confirmed in notebook 012 that the catalog exists as a flat list of fingerprints. The LOCUS concept is a design proposal, not yet verified as fitting the current data model.

**3. "Trained immunity: catalog can carry memory" (science-researcher):** The biological claim (innate cells carry memory via epigenetic reprogramming) is well-supported by the Netea-school 2024/25 reviews referenced. The mapping to the antigen fingerprint catalog ("structural layer can carry memory") is the interesting claim. But the analogy requires more precision: is fingerprint catalog persistence (across runs, via caching) the same sense of "memory" as epigenetic reprogramming? They are both "persistence of learned pattern," but the mechanisms differ. The constructor-theory bridge (resilient-self-perpetuating-information) helps — but the disanalogies need naming to make this publishable.

**4. The query-interface-as-primary-product-surface (value-finder):** The convergence of 5+ value islands onto a common query need is a genuine structural observation. The claim that "the stroma-builder must carry a query-engine primitive from day one" is a design prescription that follows from this observation — but it assumes the value surfaces are equally important. Observer note: some value surfaces are nearer-term (agent-grounding, negative-space queries) and some are far-future (federated structural lesson transfer). The "day one" prescription may overload the stroma-builder's scope. The observation is valid; the prescription needs a priority ordering.

**5. "Stroma as membrane-installer at every scale" (implicit in the expedition):** Notebook 012 documented the membrane-installer thesis (antigen installs a membrane where there was none; membranes compound = the moat). This expedition inherits that thesis. The stroma-as-Layer-0 framing grounds it. The claim has not been stress-tested specifically against the relational-base challenge: if the stroma's base becomes relations + DDlog, is the membrane metaphor still apt? The answer is probably yes (the immune meaning is still sovereign), but the architectural form of the membrane shifts.

### Ungrounded as of T+6min

**Unmapped immune machinery experiments:** The science-researcher's gap-map explicitly notes "Own note coming" and "scratch EXPERIMENT" for PRR-as-sensor-locus and trained-immunity clusters. These are hypotheses, not results. The gap-map itself is well-structured but the embedding-principle verdicts (EMBED/DERIVE/REASON-FROM per cluster) are design proposals pending actual work.

**Note-only dream islands:** The 6 note-only dream islands (affinity-maturation-runtime, complement-inverted-default, complosome-self-budget, federated-structural-antibody, generation-against-live-substrate, specs-as-antibodies-spec-rot) have been sighted but have no content deposited. Their scope is entirely unknown.

**The outsider's "model-of-reality" challenge:** The outsider's question to engineer-researcher — "is 'intent-vs-reality' actually 'intent-vs-stroma-model-of-reality'? what happens when the model is wrong?" — is one of the sharpest philosophical challenges deposited. The question names a genuine approximation gap: the stroma is a snapshot representation of code, not the code itself. The answer to this question is not in any island yet. This deserves a campsite or a formal response. It is not merely a naive question — it challenges the kernel framing in 009.

---

## Substrate-Alignment Checks

**`stroma-machinery-prior-art` reads as `[complete]` in `camp status`:** Camp status output shows this as complete, but the island story shows it with a signer clause and the expansionist having only deposited a note (not explicitly signed). Possible that the sign happened in the same burst and my story-read was sequential. Will recheck on next logbook read.

**Charter disk vs camp substrate:** The launch.md instruction says the crew should run experiments in scratch/sandbox and deposit findings as charters/notes. At T+6min, the engineer-researcher's SCIP experiment is "in flight" — the index generation is running but results haven't been deposited. This is the expected state for an active experiment. The substrate is ahead of the disk.

**No navigator in this expedition:** The launch.md explicitly states "No navigator — crew self-organizes peer-to-peer through the camp substrate." At T+6min, no navigator-role activity appears in the log. All routing (questions, notices, links) is direct role-to-role. This is the correct model. Observer notes: the outsider's questions are routed to specific roles by name, which is the right self-organizing pattern.

---

## Flags for the Expedition

**FLAG 1 — The r-a experiment must report before ADR-067 ratifies.** The SCIP finding (Hypothesis 3) directly addresses the pre-ratification block. The engineer-researcher's experiment is in flight. Camp substrate should carry the result when it lands. ADR-067 ratification should explicitly wait for this finding.

**FLAG 2 — The graph-vs-relational question is not resolved.** The adversarial-expansionist's challenge is one of the most consequential open questions in the expedition. The proposed experiment is the right path. Until it runs, the stroma-builder design (graph-as-base vs relational-as-base) is not decided. Building the stroma on the wrong base would require a migration — exactly the kind of dual-source-of-truth problem antigen exists to catch.

**FLAG 3 — The outsider's "model-of-reality" question deserves a formal response.** "Is 'intent-vs-reality' actually 'intent-vs-stroma-model-of-reality'?" is not a naive question. The kernel framing in 009 says "intent-vs-reality over the stroma." The outsider correctly identifies that the stroma is a MODEL of the codebase, not the codebase itself. This means: (1) stale stroma = wrong reality check (named in ADR-067 as `StromaIncrementalDrift`, but not acknowledged in the kernel framing), (2) incomplete stroma (no runtime state) = bounded reality check (acknowledged as a tier ceiling), (3) incorrect stroma (syn parse error) = wrong reality check (NOT acknowledged). The kernel framing should say "intent-vs-stroma-model-of-codebase" and name the approximation honestly.

**FLAG 4 — The "day one" query engine prescription needs priority ordering.** The value-finder's finding that all value surfaces are queries is structurally correct. But "carry a query-engine primitive from day one" may overscope the stroma-builder. The 011 build-plan (§3) front-loads the stroma itself; a full query-engine from day one may delay the syntactic-subset + batch `--review` first deliverable. The recommendation should be: carry a minimal query interface (the `--review` lens is already a query) from day one, with the full DDlog/query-language as a follow-on.

**FLAG 5 — Trained immunity analogy needs disanalogy articulation.** The science-researcher's trained-immunity-as-innate-memory mapping to the fingerprint catalog is genuinely interesting. But the disanalogy matters: biological trained immunity is EPIGENETIC (the reprogramming persists in the cell across generations; it is not re-computed). Antigen's fingerprint catalog "memory" is different — it is persistent storage (the catalog on disk). The question is whether the catalog itself adapts (learns from pattern-match frequency, adjusts sensitivity based on deployment history). That would be true trained immunity; the current catalog is static. The constructor-theory bridge helps (both are resilient-self-perpetuating-information) but the mechanisms are different and both deserve naming.

---

## Open Questions for the Team

1. **Graph vs relational base:** What does the experiment show? (engineer-researcher; experiment proposed but not yet run)
2. **SCIP edge reconstruction fidelity:** How close to the Python heuristic? What does it miss (dynamic dispatch? trait objects?) — and is missing those a regression or an improvement (heuristic was confidently-wrong on those anyway)? (engineer-researcher; experiment in flight)
3. **The stroma's query interface:** What is the minimal query surface that ships with the syntactic-subset + batch `--review` first deliverable? (engineer-researcher + value-finder)
4. **PRR-class as fingerprint LOCUS facet:** Does the current fingerprint data model have a place for sensing-locus? If not, is this an EMBED (a new first-class facet) or a REASON-FROM (a derived classification)? (codebase-scout + science-researcher)
5. **The outsider's model-of-reality challenge:** How does the kernel framing (009 Part 1) name the stroma-as-approximation honestly without losing the kernel's clarity? (engineer-researcher + outsider)
6. **Peripheral tolerance as a distinct site:** The gap-map identifies central tolerance (thymic/AIRE at catalog-admission) vs peripheral tolerance (brakes at the firing-site). Does antigen's current architecture have a peripheral-tolerance equivalent, or does all tolerance happen at catalog-admission? (codebase-scout)

---

## Wave State at Close of First Burst (03:31 UTC)

| Island | State | Notes |
|---|---|---|
| `stroma-machinery-prior-art` | COMPLETE | expansionist; 4-family prior-art mapping |
| `stroma-base-substrate` | open (0/1) | adversarial-expansionist; graph-vs-relational note deposited |
| `stroma-edge-tracer` | open (0/1) | engineer-researcher; SCIP finding + experiment in flight |
| `immune-gap-map-unmapped-machinery` | open (0/1) | science-researcher; 7-cluster gap-map deposited |
| `sys/stroma-coupling-map` | open (0/1) | systems-thinking; coupling-medium topology mapped |
| `sys/embed-derive-reason-coupling-gradient` | open (0/1) | systems-thinking; 3-regime coupling analysis |
| `sys/feedback-loop-atlas` | open (0/1) | systems-thinking; note deposited (not read in full yet) |
| 11 `value/` islands | open (explicit) | value-finder; unmet-need analysis per island |
| 6 `dream/` islands | note-only | think-big-dreamer; sighted, no content yet |

**What is missing:** The think-big-dreamer's dream content (all 6 islands note-only, no substantive notes). The engineer-researcher's SCIP experiment results. The expansionist's #neuroscience/#predictive-coding save (content not yet read — this is the predictive-coding structural rhyme for the stroma). The science-researcher's PRR-class and trained-immunity cluster notes (flagged as "own note coming").

---

## Observer's Publishability Assessment (T+6min snapshot)

**What would survive peer review (as of this burst):**
- The four-family mapping (expansionist) — well-grounded, term-by-term, production systems as evidence
- The SCIP third-path finding (engineer-researcher) — empirically verifiable, will have benchmark results
- The systems-topology coupling analysis (systems-thinking) — structurally sound, grounded in ADR-067's own architecture
- The negative-space queries as a novel value surface (value-finder) — clean differentiation from existing tools
- The seven-cluster gap-map method (science-researcher) — systematic, uses the expedition's own embed/derive/reason-from discipline

**What a skeptical reviewer would attack first:**
- The graph-vs-relational claim — directionally plausible but the field-computation counter is a real objection; needs the experiment
- The trained-immunity analogy — requires explicit disanalogy articulation
- The "day one query engine" prescription — needs priority ordering against the 011 §3 build-spine
- The outsider's model-of-reality question is the kind of thing a reviewer would ask immediately about the kernel framing

**What is genuinely novel in this burst:**
- SCIP as a third path resolving the r-a fork (not in ADR-067)
- The graph-as-first-induced-view challenge (not in 009 or ADR-067 — the strongest structural challenge to date)
- The embed/derive/reason-from coupling-strength gradient analysis (extends 011's framing to a dynamical-regimes frame)
- The PRR-as-sensor-locus mapping for the fingerprint catalog (not in any charter)
- Trained immunity as the fingerprint catalog's analog to innate memory (connects constructor-theory charter to the learning-core)
- The stroma-query-interface as the joint unmet need (synthesizes 5+ value islands into one architectural prescription)

---

## Post-Sleep Update: Experiments Land Results (~04:00–04:47 UTC)

*Observer woke to 37 islands moved during the ~1-hour sleep window. This section records what actually happened, not what was hoped for.*

### Experiment Results: What the Data Says

**SCIP edge-reconstruction benchmark (engineer-researcher, `stroma-edge-tracer` — SIGNED COMPLETE 04:03):**

| Measurement | Result |
|---|---|
| Cold SCIP index time | 49s |
| Warm/incremental SCIP index time | 13-16s |
| One-file-touch SCIP time | 13-16s (fresh CLI = no salsa memory) |
| LSP delta (resident salsa db) | sub-second |
| Syntactic-only call-graph coverage | 2332 edges (48% of SCIP's 4647) |
| SCIP resolved call-graph coverage | 4647 edges (100% baseline) |

*The 48% figure is now the named floor: ADR-067 §E graceful-degradation has a concrete measurement. The 52% gap is the resolved-tier's first-value proposition — every transitive-reachability needle in the catalog that depends on cross-function edges requires closing this gap.*

*The one-file-touch timing confirms the CLI is a fresh-process invocation, not incremental (no salsa state persistence across calls). Live LSP sub-second is the delta regime, but it requires a resident r-a process. The SCIP batch CLI is the zero-infrastructure path, 13-16s for a warm run.*

**Semiring experiment on real antigen call-graph (adversarial-expansionist, `stroma-semiring-unification` — via circle speak):**

| Finding | Detail |
|---|---|
| Graph | 2948 fns / 4714 edges (SCIP on antigen's workspace) |
| Semiring algebra | One graph, semiring-parameterized queries |
| boolean(detection) vs tropical-min(conductance) vs counting(blast) | Identical reachability relation; only per-pair value differs |
| Counting semiring blowup | 242s on cyclic graph (100,000x over boolean) |
| Antigen call-graph cyclic depth | Deepest chain 12 |
| Consequence | Field semiring MUST be idempotent or bounded |
| Engine consequence | One revision-clock (salsa) + one semiring-query layer + idempotency maintenance |

*This experiment settled systems-thinking's Condition 3 empirically. The evaluation-strategy fork (salsa PULL for memoized/idempotent queries; differential-dataflow PUSH for non-idempotent/field) is architecturally motivated by antigen's own graph topology, not a design preference. The conductance = tropical-min (not sum-of-paths) is re-derived from the cyclic-graph constraint — both a design vindication and an empirical grounding.*

**Codebase-scout grounding sweep (all 34 dream + value islands, sleep of 04:46):**

The codebase-scout completed a full sweep of all 34 dream + value islands. Three classes of finding:

*Compose-region gaps (all map to existing prior art):*
- salsa-memoized scan → replaces hand-rolled `scan/walk.rs`
- semiring queries (detection/field/provenance/blast-radius) → proven by experiment 2
- import/inheritance/co-change edges → SCIP/r-a resolved-tier
- incremental keystone → salsa revision-clock

*Sovereign-region gaps (all zero-code, zero prior art):*
- write-back primitive (runtime edge ingest): ZERO CODE in antigen
- `#[intent]` macro / intent declarations: ZERO CODE (vocabulary only in research notebooks)
- parity-oracle (self-distrust): ZERO CODE (no cross-comparison tooling)
- checkability map (what can/cannot hold an intent): ZERO CODE (requires intent layer first)

*Constitutive algebra split (empirical find, directly in current code):*
- `antigen-macros/src/lib.rs:#[antigen_generates]` = AUTHORED constitution (fingerprint-generation is a sovereign decision, not recomputable from sources)
- `scan/synthesis.rs:284-313` = SOURCE-DETERMINED constitution (AST-derived node-set, recomputable, compose-side)

*Observer assessment of the sweep:* The codebase-scout's evidence is the strongest empirical grounding the partition has. The recomputability line (systems-thinking's Condition 1) is not a design proposal — it is verified in the current codebase. Every compose-region gap has a named prior-art replacement; every sovereign-region gap has zero code and zero prior art. This is the partition proved by actual inventory, not just conceptual argument.

**Three surprise finds (codebase-scout's own summary):**

1. **Affinity maturation already implemented.** `learn/maturation.rs:mature()` IS the germinal center loop: `mutation_candidates()` + `Affinity::measure()` + `pareto_improves_on()` + `max_rounds` iteration. The batch algorithm is real and working. The true gap is ONLINE (live stroma) vs OFFLINE (batch). The dream is a stage-2 capability, not a missing capability.

2. **Antigen already answers negative-space queries.** `MarkedUnknown`, `WitnessStatus::Missing`, `ClassVerdict::Obsolete` are item-level NK-cell analogs. The product surface IS there at item level — it is not documented as a product feature. The stroma would lift this to region/graph-level.

3. **Six pre-stroma wins, ~50-200 lines each:**
   - Match-evidence proof term (matcher.rs — ~200 lines): discard proof of predicate match; return it instead
   - Cluster-key persistence cache (pipeline.rs → `.antigen/` — ~20 lines)
   - `committed_at` timestamp in `CommitMeta` (learn/szz.rs — ~5 lines)
   - Training corpus serialization (cargo-antigen — ~50 lines)
   - Blindspot collation (audit output — ~50 lines)
   - Watch mode (cargo-antigen + scan/walk.rs — ~100 lines)

*Observer assessment:* The pre-stroma wins are the most actionable finding of the expedition for the near-term build. They each embed/derive cleanly (embedding-principle check passed), give partial value to multiple dream/value islands simultaneously, and none require new primitive types in the stroma. The cluster-key persistence cache ([signed COMPLETE](stroma-sovereign-vs-compose-partition)) is already closed.

---

### The Sovereign/Compose Partition: Final Observer Assessment

**What the evidence says:** Five independent derivations all draw the same line.

| Derivation | Method | Evidence type |
|---|---|---|
| Prior-art come-apart | CodeQL has compose-region, zero sovereign moves | Structural |
| Coupling-theory read/write | Read-algebra = compose; non-recomputable write-algebra = sovereign | Conceptual + structural |
| Semiring experiment | One graph, semiring-parameterized queries; idempotency as seam | Empirical (real antigen call-graph) |
| SCIP experiment | Resolved-tier = proven infrastructure, compose-side | Empirical (real SCIP benchmark) |
| Codebase-scout sweep | Compose gaps = all prior-art; sovereign gaps = all zero-code | Empirical (current codebase) |

Five-derivation convergence on an exhaustive + disjoint + come-apart-proven partition meets the observer's evidentiary standard. This is a well-grounded finding.

**Systems-thinking's three conditions (accepted by adversarial-expansionist, confirmed empirically):**
1. Recomputability line: objective per-primitive build test
2. Parity oracle sovereign-and-external: no self-witness
3. One algebra, two evaluation strategies: empirically confirmed by semiring experiment

**The open question that remains:** The constitutive algebra status (authored-constitutive as a small sovereign remainder vs a sublayer of the write-algebra). Adv-exp's "two-and-a-half" estimate is empirically grounded by the codebase-scout. The mesh wave should apply the per-act recomputability test before the capstone names "two" or "three." This does NOT affect the partition itself.

**Observer circle position:** Signed YES. The evidentiary record is here; the circle position records the reasoning summary.

---

### Wave State Update (04:47 UTC)

**Islands that closed during sleep window:**

| Island | State | Notes |
|---|---|---|
| `stroma-sovereign-vs-compose-partition` | COMPLETE | adversarial-expansionist; exhaustive+disjoint partition + come-apart proof |
| `stroma-engine-choice` | COMPLETE (engineer-researcher, 04:04) | demand/push two-half architecture; maturity ground-truth (salsa + differential-dataflow) |
| `stroma-edge-tracer` | COMPLETE (engineer-researcher, 04:03) | SCIP latency/coverage measured; 48% syntactic floor named |
| `stroma-base-substrate` | COMPLETE (adversarial-expansionist, 03:43) | Graph-vs-relational resolved; relational base + sovereign-surface-base-choice-independent |
| `dream/cluster-key-persistence-cache` | COMPLETE (codebase-scout) | 4-island convergence primitive, ~20 lines, pre-stroma |

**Circles:**

| Circle | State | Participants signed |
|---|---|---|
| `stroma-sovereign-vs-compose` | Converging | adversarial-expansionist, systems-thinking (2x), observer |
| `stroma-constructor-capacity` | Partial (1/2 clauses) | engineer-researcher needs to sign |

**Active/open islands (snapshot ~04:47 UTC):**
- All `value/` islands: open, awaiting convergence wave
- All dream islands (except cluster-key-persistence-cache): note-only or open, awaiting convergence
- `immune-gap-map-unmapped-machinery`: open
- `sys/` islands: open
- `stroma-semiring-unification`: open (adv-exp spoke in circle, no formal closure yet)
- `stroma-sovereign-vs-compose`: circle converging, not yet formally converged

---

### Publishability Assessment (Post-Experiments)

**What would survive peer review (post-experiment snapshot):**

1. **The compose/sovereign partition** — exhaustive, disjoint, come-apart-proven, five-derivation convergence, empirically confirmed by codebase inventory. Novel finding: the sovereign surface is SMALLER than ADR-067 frames it.

2. **SCIP as the third path resolving ADR-067 Open Seam §1** — empirical benchmark, 48% floor measurement, clear tier-ladder consequence. Novel finding: not in ADR-067; closes the pre-ratification block for call/reference edges.

3. **Semiring algebra as the unified compose architecture** — empirical confirmation on antigen's real call-graph; idempotency as the seam is re-derived from antigen's own graph topology. Novel find: cyclic-graph constraint forces the conductance semiring to be tropical (not counting) — a consequence of antigen's OWN topology, not a design preference.

4. **Six pre-stroma wins** — all buildable now, embedding-principle verified, each unlocks multiple dream/value islands. Most actionable near-term finding of the expedition.

5. **Affinity maturation is batch-real (not missing)** — `learn/maturation.rs:mature()` is the germinal center loop. Novel claim correction: the dream is "online" not "implemented," which is a more honest framing than "future dream."

**What a skeptical reviewer would attack:**

- The constitutive algebra open question: the paper shouldn't call it "two" or "three" algebras until the mesh wave runs the per-act test. The current honest answer is "two-and-a-half."
- The field's evaluation strategy: salsa PULL vs DD PUSH is empirically motivated but the exact boundary (which queries are idempotent?) needs a complete idempotency survey of the current and planned semiring operations. The semiring experiment covers detection/conductance/counting; the full catalog needs the same test.
- The outsider's model-of-reality question [4b2c3dba]: remains unresolved. The kernel framing in 009 should say "intent-vs-stroma-model-of-codebase" and name the approximation honestly. No island has formally closed this.

**What is genuinely novel in the post-experiment burst:**
- The semiring experiment showing cyclic-graph enforcement of idempotency (re-derives design from topology)
- The 48% syntactic floor measurement (named empirically for the first time)
- The constitutive algebra split IN CURRENT CODE (unnamed, found by codebase-scout)
- Affinity maturation corrected from "future dream" to "batch-real, online-gap"
- The negative-space query as an existing item-level capability (NK-cell analog — named for the first time as a product surface)

---

*Notebook status: Active. Circle speak recorded; lab notebook updated through 04:47 UTC. Remaining open work: circle convergence + formal sign; outsider model-of-reality question still open; constitutive algebra question deferred to mesh wave. Lab notebook at `R:/antigen-061-self-non-self/research/notebooks/013-the-stroma-remembers-observer-lab-notebook.md`.*

# Lab Notebook 015: the-spike-yard — Observer Record

**Date**: 2026-06-28 (expedition launched 23:35 UTC)
**Author**: the-spike-yard--observer
**Branch**: v0.6.1-self-non-self (worktree: R:/antigen-061-self-non-self)
**Status**: Active (live — updated as spikes land)
**Depends on**: 014 (the-crucible observer record — the finalized design this voyage tests), 013 (the-stroma-remembers), 009 (intent kernel + coordinate system)

---

## Context & Motivation

The crucible expedition finalized the stroma's **design**. Its headline conclusions:

- **Recomputability** is the generating distinction (K0); read-write is its projection onto the mutation axis.
- **Relational-as-base** is the chosen substrate for the stroma.
- The **compose/sovereign partition** at the recomputability line holds.
- The **salsa-clock + datalog-query engine** is the selected architecture.
- The **three-region parity invariant** is the correctness signal.
- **A4 product-cell** framing: antigen occupies the unique cell in efferent × normative × modal × reflexive.
- Both deciding experiments (base-substrate and output-substrate) were named but NOT run — those are pre-ratification gates that the spike-yard is intended to run.

This expedition is **not the build**. It is **compressed evolution**: the crew generates variation (diverse throwaway spikes across 6 pipeline stages), the scientist and test-architect measure fitness, and the captain runs selection to recombine the fittest stage-choices into a GEN-2 candidate. The observer maintains the **Implementation-Findings Map** — the artifact converge inherits.

**Observer's mandate:**
1. Record every spike with the full scientific framework: hypothesis, method, result, conclusion.
2. Maintain the per-stage fitness map (options × measured fitness × recommendation).
3. Peer-review every "it works" claim against measured evidence. Flag unchecked claims immediately.
4. Read camp activity continuously. The map must be built FOR selection + recombination, not as a flat list.

---

## T=0 State (2026-06-28 23:35–23:39 UTC)

### Expedition launch

Team-lead prepared the expedition at 23:35 UTC. 9 crew roles registered: pathmaker, expansionist, low-hanging-fruit, test-architect, scientist, scout, systems-thinking-expert, observer, naturalist. Captain joined separately.

**Activity log at T=0 (4 entries):**
- `[95029eb4]` 23:35 UTC — role join (captain)
- `[85296077]` 23:35 UTC — team prepare (9+captain roles)
- `[873a1f00]` 23:39 UTC — new island `the-spike-yard/the-evolution-frame` (captain, signer: captain)
- `[9d50d282]` 23:39 UTC — note on `the-evolution-frame` (captain: compressed-evolution frame)

### The captain's evolution frame

The captain has explicitly named the metaphor: this voyage is antigen's own evolution-organ (germinal-center / affinity-maturation, the crucible CEGIS finding) applied to **building antigen itself**. The recursion is the point.

- **VARIATION** = crew spikes across the 6 pipeline stages
- **SELECTION** = scientist benchmarks + test-architect breakages + observer findings-map
- **RECOMBINATION + MUTATION** = captain takes fittest stage-choices, recombines, re-spikes in GEN-2

**Implication for the findings map**: The map must be structured as a **genome-space** that selection can operate on — not a flat list. Per stage: surviving variants × measured fitness × which traits carry forward into GEN-2.

---

## Givens (from the crucible — do not re-litigate)

| Decision | Source | Status |
|----------|--------|--------|
| Recomputability is the generating cut (K0) | Crucible 4-way convergence | RATIFIED |
| Relational-as-base for stroma substrate | Charter-stroma-engine (ADR-067) | RATIFIED (but tested only at n=2948) |
| Compose/sovereign partition at recomputability line | 5 independent derivations | RATIFIED (independence needs verification per Orzack-Sober) |
| Salsa-clock + datalog-query engine | Crucible | RATIFIED |
| SCIP as the third path for edge resolution | Charter-stroma-engine | RATIFIED |
| 3-tier edge ladder: syntactic < resolved < mir-exact | Measured: 52% miss rate for heuristic | RATIFIED |
| 4 semirings unify over 1 path-query | Measured on n=2948/4714 edges | RATIFIED |
| Temporal gate (backdate-gated `last_changed`) | Salsa prototype | RATIFIED (small scale; spike-yard tests at real scale) |
| Three-region parity invariant | Structural | RATIFIED |

**Pre-ratification GATES that the spike-yard must run** (named in crucible but not executed):
1. Base-substrate both-ways: hand-rolled graph+wavefront vs datalog (Souffle/ascent/DD) on antigen's real code at real scale
2. Output-substrate three-ways: injected-in-source vs git-notes-sidecar vs LSP-overlay on antigen's own repo
3. Anergy/reversible-write quantity experiment
4. EXERCISED-below-LATENT (3-vs-5 axis test) — requires resolved-edges in a spike

---

## Pipeline Stages Under Spiking

Six stages per the launch.md charge:

| Stage | Question | Key alternatives |
|-------|----------|-----------------|
| 1. INPUT | How does data get in? | `syn` AST · rust-analyzer (LSP vs `ra_ap_*` vs SCIP batch) · MIR |
| 2. STRUCTURE/BASE | Relational facts vs property-graph vs hybrid? | Datalog schemas · graph DB · hybrid |
| 3. COMPOSITION/ENGINE | Salsa + which datalog? Do they compose? | ascent · crepe · datafrog · Souffle/FFI · differential-dataflow |
| 4. FRESHNESS | Snapshot↔live · salsa-revision · temporal model | Batch vs streaming vs on-demand |
| 5. OUTPUT | Where markers land | Injected-in-source · git-notes-sidecar · LSP-overlay |
| 6. CONSUMPTION | How things consume the stroma | Induced-views · query API · 4 semirings · LLM-grounding · IDE |

---

## Implementation-Findings Map — GEN-1 (LIVE)

*This is the central deliverable. Updated as spikes land. Each stage has a grid: variant × fitness-dimensions × recommendation.*

### Fitness Dimensions (from launch.md)

- **F-CORRECT**: Correctness of resolved call/data-flow edges (vs ground truth)
- **F-FRESH**: Freshness behaviour — correct handling of meaning-change (not byte-change)
- **F-INCR**: Incrementality cost (re-query cost on small change)
- **F-INTEG**: Integration/maintenance cost (compose-is-not-cheap is real)
- **F-SEMIRING**: How cleanly the 4 semirings express
- **F-CONSUM**: How well the output serves LLM + human + tooling consumption
- **F-CHEAP**: Cheapest-thing-that-works (low cost is a genuine win)

Rating scale: UNKNOWN · MEASURED · ESTIMATED · FAILS · SURVIVES · WINS

---

### STAGE 1 — INPUT

*How data gets in: resolution depth and combination.*

| Variant | F-CORRECT | F-INCR | F-INTEG | F-CHEAP | Status | Notes |
|---------|-----------|--------|---------|---------|--------|-------|
| `syn` AST (syntactic) | — | — | — | — | NOT YET SPIKED | 52% miss rate known from crucible; baseline |
| rust-analyzer LSP mode | — | — | — | — | NOT YET SPIKED | High integration cost; running daemon |
| `ra_ap_*` library | — | — | — | — | NOT YET SPIKED | In-process; no daemon; more composable |
| SCIP batch | — | — | — | — | NOT YET SPIKED | Cold 13-49s; 4647 vs 2332 edges |
| MIR | — | — | — | — | NOT YET SPIKED | Requires nightly; exact-semantics but higher cost |
| syn + SCIP hybrid | — | — | — | — | NOT YET SPIKED | Low-hanging-fruit candidate |

**Pre-existing evidence (from crucible):**
- SCIP batch: 4647 edges vs 2332 heuristic = 52% miss rate for heuristic. Cold start 13-49s. Third-path candidate.
- `syn` AST: antigen's current surface. Syntactic only — misses dynamic dispatch, trait impls.

**Open questions for stage 1 spikes:**
- Can SCIP incremental mode (not cold-batch) reduce the 13-49s cold cost? What is the warm-path latency?
- Does the `ra_ap_*` library provide equivalent edges to SCIP at lower integration cost?
- What is the minimum viable INPUT for the 4-semiring path-query to work correctly?

**GEN-1 recommendation**: UNKNOWN until spikes land. The crucible measured SCIP > heuristic at the edge-count level; the spike-yard needs to measure SCIP vs ra_ap at integration-cost level.

---

### STAGE 2 — STRUCTURE / BASE

*How the stroma stores facts: relational vs graph vs hybrid.*

| Variant | F-CORRECT | F-SEMIRING | F-INCR | F-INTEG | F-CHEAP | Status | Notes |
|---------|-----------|------------|--------|---------|---------|--------|-------|
| Datalog (relational facts) | — | — | — | — | — | NOT YET SPIKED | Ratified as base; tested at n=2948 only |
| Property-graph | — | — | — | — | — | NOT YET SPIKED | Exploration target |
| Hybrid (relational base + graph views) | — | — | — | — | — | NOT YET SPIKED | Possible best-of-both |

**Pre-existing evidence:**
- Relational-as-base was decided on a toy n=2948 graph (antigen itself). Real scale is unknown.
- The 4-semiring unification was measured on this toy graph.
- **Pre-ratification gate**: this deciding experiment must be run at real scale before the stroma build commits to relational-as-base.

**Correctness constraint (from partition taxonomy crack in crucible):**
The adversarial found that temporal stamps (`last_changed`) don't fit the 4-cell write taxonomy cleanly — they are historical state, not recomputable from current snapshot. Any base structure must have an explicit answer for historical/temporal facts.

**Open questions for stage 2 spikes:**
- At N=10K+ nodes / 100K+ edges (real antigen corpus + its dependencies), does relational-as-base maintain sub-second query latency for the 4-semiring path-queries?
- Does a property-graph representation express the 4 semirings more naturally or less naturally than relational facts?
- Where does `last_changed` (historical stamp) live in the base structure? Which table/relation? What is the provenance model for time?

**GEN-1 recommendation**: UNKNOWN until real-scale measurement. The toy-graph measurement cannot stand alone as ratification evidence.

---

### STAGE 3 — COMPOSITION / ENGINE

*Salsa-clock + which datalog? Do they compose without fighting?*

| Variant | F-SEMIRING | F-INCR | F-INTEG | F-CHEAP | Status | Notes |
|---------|------------|--------|---------|---------|--------|-------|
| ascent | — | — | — | — | NOT YET SPIKED | Pure Rust; most active |
| crepe | — | — | — | — | NOT YET SPIKED | Pure Rust; simpler |
| datafrog | — | — | — | — | NOT YET SPIKED | Polonius engine; battle-tested |
| Souffle/FFI | — | — | — | — | NOT YET SPIKED | External; most powerful; FFI cost |
| differential-dataflow | — | — | — | — | NOT YET SPIKED | Push-based; incremental at engine level |
| salsa 0.27 + ascent | — | — | — | — | NOT YET SPIKED | The proposed stack; do revision models fight? |
| salsa 0.27 + datafrog | — | — | — | — | NOT YET SPIKED | Polonius heritage; proven |

**Critical open question (contrarian DM4 from crucible):**
Does the salsa-clock (pull-based, revision model) + datalog-query (push or pull, fixed-point) compose, or do the two revision models fight? The crucible named this as an unresolved composition risk. The spike-yard MUST produce an empirical answer.

**Pre-existing evidence:**
- Salsa 0.27 prototype ran the temporal gate (5-clock-ticks experiment). Gate held. But this did NOT test a full datalog engine inside a salsa query.
- Souffle is the most expressive and widely deployed datalog engine; FFI cost is real but bounded.
- datafrog powers Polonius (Rust borrow checker) — proven at Rust-compiler scale.
- ascent is the newest pure-Rust option with the most active development.

**Open questions:**
- What is the exact composition seam? Does salsa call the datalog engine as a pure function, or does the datalog engine need to track its own revision history? If the latter, the two revision models DO fight.
- For the 4-semiring path-query: which datalog variant handles recursive queries over parameterized semirings most naturally?
- Is differential-dataflow (push-based) composable with salsa (pull-based), or does the push/pull boundary require a bridge that adds latency?

**GEN-1 recommendation**: UNKNOWN. This is the highest-risk stage — the revision-model clash is a genuine unknow that no prior expedition measured. This stage deserves the EARLIEST spike.

---

### STAGE 4 — FRESHNESS

*How the stroma stays current: cost of each freshness model.*

| Variant | F-FRESH | F-INCR | F-CHEAP | Status | Notes |
|---------|---------|--------|---------|--------|-------|
| Snapshot (batch rebuild) | — | — | — | NOT YET SPIKED | Simplest; high latency |
| Salsa-revision (on-demand) | — | — | — | NOT YET SPIKED | The ratified model; prototype only |
| Backdate/`last_changed` temporal | — | — | — | NOT YET SPIKED | Temporal organism — anergy + fibrosis |
| Streaming (event-driven) | — | — | — | NOT YET SPIKED | Lowest latency; highest cost |
| Hybrid (snapshot + on-demand overlay) | — | — | — | NOT YET SPIKED | Possible LHF candidate |

**Pre-existing evidence:**
- Temporal gate held at 5-clock-ticks scale (salsa 0.27 prototype). Not measured at real scale.
- Anergy mode: no antigen quantity confirmed yet. The "reversible-anergy" experiment has never run.
- Fibrosis evidence: the SZZ 2.5-5.3x correlation is correlational, NOT mechanistic. The spike-yard's scientist should run the mechanistic bridge test.

**Key open question from crucible (contrarian seam S4):**
The EXERCISED-below-LATENT test (3-vs-5 axis) requires resolved edges — which requires the stroma to exist — but the stroma build needs the 3-vs-5 count settled first. Gate-ordering inversion. One of the stage 1+4 spikes may break this cycle by producing resolved edges in scratch.

**Open questions:**
- What fraction of salsa cache invalidations (on a real code edit) propagate to requiring full datalog re-evaluation vs incremental delta propagation?
- What is the wall-clock latency for a cold salsa query (full SCIP parse + datalog evaluation) on the antigen codebase (52K lines)?
- Can the temporal gate be validated at real corpus scale in a scratch spike?

**GEN-1 recommendation**: UNKNOWN. The freshness model is tightly coupled to stages 1 and 3.

---

### STAGE 5 — OUTPUT

*Where markers land: the deferred three-way experiment.*

| Variant | F-CORRECT | F-FRESH | F-CONSUM | F-INTEG | F-CHEAP | Status | Notes |
|---------|-----------|---------|---------|---------|---------|--------|-------|
| Injected-in-source | — | — | — | — | — | NOT YET SPIKED | antigen's current model; co-native |
| git-notes-sidecar | — | — | — | — | — | NOT YET SPIKED | External; survives builds; tooling exists |
| LSP-overlay | — | — | — | — | — | NOT YET SPIKED | Real-time; IDE-native; ephemeral |
| DB/JSON sidecar | — | — | — | — | — | NOT YET SPIKED | Simple; non-co-native |

**Pre-existing evidence:**
- Injected-in-source: antigen's entire existing model. Maximally co-native. Requires source-modifying write-back.
- git-notes-sidecar: survives the build; allows marker updates without touching source; cited as a "deferred experiment" in crucible.
- LSP-overlay: real-time visibility in IDE; no source touch required; ephemeral on process death.

**The co-native design principle** (from CLAUDE.md and ADR-002 amendment 3):
Build representations that BOTH human and AI agents can engage with natively — without either translating to the other's format. Injected-in-source is maximally co-native. git-notes requires translation for human readers. LSP-overlay is IDE-native but not durable without LSP server.

**Critical constraint**: antigen's existing model is injected-in-source. The stroma's markers (efferent writes — the never-done write-back axis) extend this model. The question is whether stroma markers CAN live in source co-natively or require a separate substrate.

**Open questions:**
- Can the stroma's inferred `danger`, `symbiont`, `pathogen` verdicts be written back as source annotations (as antigen currently writes `#[presents]` etc.) — or is the inference-velocity too high, making write-back too noisy?
- What is the durability model for LSP-overlay when the LSP server restarts? Can it be reconstituated from the stroma?
- Does git-notes-sidecar work in a worktree? (Known camp feedback: worktrees have git complexity.)

**GEN-1 recommendation**: UNKNOWN. The co-native principle strongly favors injected-in-source; the inference-frequency may make that impractical. This is the experiment to run.

---

### STAGE 6 — CONSUMPTION

*How things consume the stroma.*

| Variant | F-SEMIRING | F-CONSUM | F-FRESH | F-INTEG | Status | Notes |
|---------|------------|---------|---------|---------|--------|-------|
| Induced-views (query API) | — | — | — | — | NOT YET SPIKED | The designed consumption surface |
| 4-semiring path-queries | — | — | — | — | NOT YET SPIKED | Measured at n=2948; needs real scale |
| LLM-grounding at generation | — | — | — | — | NOT YET SPIKED | The AI-native consumption surface |
| IDE/co-native surfaces (LSP) | — | — | — | — | NOT YET SPIKED | Developer-facing |
| CLI tooling (cargo-antigen) | — | — | — | — | NOT YET SPIKED | cargo-antigen already exists; extend |

**Pre-existing evidence:**
- 4-semiring path-query: measured as one unified query over the antigen codebase at n=2948. Query correctness: 33,882-pair reachability relation identical across all three derivations. This is the strongest empirical evidence in the entire stroma design.
- LLM-grounding: the key AI-native value surface — antigen compensates the generation-inspection asymmetry. This is the co-native design's payoff.

**Open questions:**
- Does the 4-semiring unification hold at 10× scale (N=30K nodes)?
- What is the latency of the induced-view query for a real LLM call? Sub-100ms? Sub-1s?
- Can the stroma serve as a real-time grounding source for a tool-use call during LLM generation?

**GEN-1 recommendation**: UNKNOWN. The 4-semiring result is the most-measured part of the design; scale validation is the remaining gap.

---

## Deferred Experiments (from crucible)

Two experiments were named in the crucible as pre-ratification gates and never run. The spike-yard is designed to run them.

### EXP-1: EXERCISED-below-LATENT (3-vs-5 axis test)

**What it tests**: Whether the temporal-integration and arity dimensions are genuinely orthogonal read-axes (5-axis position) or fold into the SOURCE axis (3-axis position).

**Required precondition**: resolved edges must exist in a spike (a stage 1+2 spike produces them).

**Math-researcher finding (crucible)**: 
- ARITY: provenance semirings paper SUPPORTS 3-with-candidates (arity folds into source at different recursion depth).
- TEMPORALITY: bitemporal database literature says time is ORTHOGONAL (not a fold).

**Observer's prediction (H3 from crucible)**: The experiment will show arity folds (3-camp wins) but temporality stays separate (5-camp has a point on temporality). Expected result: 4 axes (3 + temporal-integration as a genuine 4th).

**Measurement design**: Run the path-query under variations that change only temporal-integration and only arity. If the semiring output changes under temporal-variation but not arity-variation (controlling for other axes), temporality is orthogonal.

### EXP-2: Output-substrate three-way

**What it tests**: injected-in-source vs git-notes-sidecar vs LSP-overlay for the stroma's efferent write-back.

**Measurement dimensions**: Co-nativeness (human readability in-source), durability (survives restarts/builds), write-frequency ceiling (how many markers can the efferent layer write before source becomes noisy), LLM-readability (does the LLM see stroma markers in its context window when generating against this file?).

### EXP-3: Reversible-anergy quantity

**What it tests**: Is there a measurable antigen quantity that relaxes a confirmed verdict when the triggering condition reverses?

**Pre-existing state**: No antigen quantity confirmed yet (crucible finding). The temporal gate prototype used `last_changed`; anergy would require a quantity that DECREASES as code ages without being flagged.

**Measurement design**: Not yet designed. Candidate: a `last_changed`-derived age score that decays in "conformity confidence" over time.

---

## Hypotheses Before Results

Written before spikes land.

**H1 (composition/engine)**: The salsa-clock + datalog-query revision-model clash (contrarian DM4) is REAL and will require an architectural bridge. Pure salsa queries calling pure datalog evaluation will work only if the datalog engine is a pure function of the salsa-managed input relation. If the datalog engine tries to maintain its own incremental state, the two revision models will fight. Prediction: ascent (pure Rust, salsa-callable as a function) will compose most naturally; differential-dataflow will require a bridge.

**H2 (base-substrate at scale)**: Relational-as-base will hold correctness at real scale but will reveal latency gaps for full-corpus path-queries. Prediction: sub-second for per-function queries; multi-second for whole-corpus 4-semiring queries at N=50K nodes. The spike will need to measure whether the salsa on-demand model keeps per-function queries sub-second.

**H3 (output-substrate)**: Injected-in-source will win on co-nativeness and LLM readability but lose on write-frequency ceiling. The stroma's inferred verdicts will be too volatile to safely inject into source as static annotations. Prediction: git-notes-sidecar will be the engineering winner; injected-in-source will be the UX winner for human readers. A hybrid is likely: git-notes as durable backing store + injected-in-source as optional view layer.

**H4 (SCIP at real scale)**: SCIP's 13-49s cold start is the biggest practical obstacle. The spike-yard will find that incremental SCIP (LSP mode) reduces this to sub-second for warm queries. Prediction: SCIP is the right foundation but cannot be run cold-on-demand; it needs to run as a background service.

**H5 (LHF candidate)**: The cheapest-thing-that-works for GEN-1 will be: `syn` AST for most structural queries + on-demand SCIP for resolved-edge queries + datalog (ascent) for path-queries + git-notes as output substrate. Not the most elegant design, but measurably fast to integrate and measurably correct for antigen's primary use cases.

---

## Running Spike Log

*Per spike: hypothesis, method, result, conclusion. Updated as spikes land.*

---

## Wave 1 — First Crew Burst (~23:40–23:45 UTC, 2026-06-28)

The crew produced 25+ camp events in roughly 5–6 minutes. This section records every significant finding by role.

### Systems-Thinking-Expert: Genome-Space + Spike-Order Map

Two deep islands landed back-to-back: `sys/genome-space-linkage-map` and `sys/spike-order-leverage-terrain`.

#### The Genome-Space (linkage map)

The 6 pipeline stages are LOCI, not independent dimensions. The key linkage edges:

| Edge | Direction | Constraint |
|------|-----------|-----------|
| LE1 | L1 INPUT ==> L6 CONSUMPTION | syn-only FORBIDS any flow-consuming consumption (52% miss = wrong 52% of the time). L1 sets the hard ceiling on L6. |
| LE2 | L1 INPUT ==> L4 FRESHNESS | syn-only FORBIDS semantic-freshness. Meaning-change detection requires resolved/MIR input. |
| LE3 | L1 INPUT ==> L3 ENGINE (parity-oracle) | L1=r-a-only FORBIDS a SOUND parity-oracle for resolved edges (shared upstream = shared blind spot). Self-distrust requires a SECOND resolver (MIR-exact) as oracle source. |
| LE4 | L2 STRUCTURE ↔ L3 ENGINE | relational-datalog ↔ salsa-clock+datalog-query CO-REQUIRE. property-graph would FORBID clean salsa-facts mapping. DECIDED linkage group. |
| LE5 | L3 ENGINE ==> L6 CONSUMPTION | Counting-blast semiring is NON-VIABLE: 206s vs 2.4ms (85,000x). Forces blast-as-reachable-SET not blast-as-path-COUNT. |
| LE6 | L4 FRESHNESS ==> L3 ENGINE | last_changed-as-input FORCES maintenance-pass stamping, NEVER a tracked-fn (VL3 pin-at-zero recurring). A RULE on L3. |
| LE7 | L5 OUTPUT ==> L4 FRESHNESS | injected-in-source COUPLES to L4 (marker write IS a source edit = bumps digest = risks VL1 epitope-spreading). git-notes-sidecar RELAXES this constraint. HIGHEST-VALUE OPEN linkage. |
| LE8 | L5 OUTPUT ==> L1 INPUT | injected-in-source + re-parse = markers become INPUT next cycle (self-ingestion = A6 recursion at implementation level). sidecar/overlay avoid this. |

**THREE LINKAGE GROUPS:**
- **{L1-root}**: upstream locus, cascades to L3/L4/L6 via 3 edges
- **{L2-L3-L4 engine-core}**: mostly DECIDED (relational-as-base + salsa-clock decided at toy scale)
- **{L5-output}**: GENUINELY OPEN, two back-edges to L4 and L1 (the A6 self-recursion made concrete)

**Observer's assessment:** This is the most important structural finding of Wave 1. The linkage map means the captain CANNOT freely recombine stage alleles — crossing a linkage edge produces a non-viable chromosome. A spike claiming L1=syn-only + L6=flow-consuming-semiring is incoherent regardless of how good L2–L5 are. The observer will flag any spike that proposes an incoherent chromosome.

**What is NOT in the linkage map that should be**: The Orzack-Sober critique from the crucible applies here too — the "5 independent derivations" for relational-as-base may share common assumptions. The linkage map treats L2 as DECIDED, but the deciding experiments ran at n=2948 only. The linkage is decided AT TOY SCALE; at real scale it may need re-opening.

#### The Spike-Order Map (leverage terrain)

Systems-thinking-expert applied Meadows leverage analysis to the spike-order question:

| Rank | Stage | Meadows level | Rationale |
|------|-------|---------------|-----------|
| #1 | L1 INPUT | #6 info-flow + #2-adjacent | Root of linkage DAG; sets correctness ceiling. Steepest fitness gradient (52% correctness swing). The SCIP-third-path frontier is the highest-information-per-spike measurement. |
| #2 | L5 OUTPUT | #5 rules | Only genuinely OPEN linkage; two back-edges = A6 recursion at implementation scale; highest DISCOVERY VALUE because it's unmapped. |
| #3 | L3 ENGINE (datalog backend) | #9 parameter | Low gradient — all four datalogs land ~3ms at current scale. Parallel benchmark, NOT the centerpiece. |
| #4 | L6 CONSUMPTION | #6 info-flow out | DOWNSTREAM of everything; wait for a minimal engine. |

**Observer's assessment:** The spike-order analysis is HIGH quality — it correctly identifies that spiking four datalog backends generates low-value variation (shallow fitness gradient) while spiking L1 resolver depth generates high-value variation (steep gradient). However, I dispute one element: the systems-thinking-expert says L3-backend is a "#9 parameter choice" but NOTE the revision-model clash (DM4 — salsa-clock + datalog composition). The contrarian DM4 concern is NOT a parameter tuning question; it's a STRUCTURAL question about whether the two engines compose cleanly. This deserves a HIGHER priority spike than "#9 parameter." The pathmaker has already opened `spike/dm4-salsa-ascent-compose` to resolve it — the observer endorses prioritizing this.

---

### Test-Architect: ATK-CLOSURE-DENSITY-CLIFF (born-red)

Island `spike/atk-closure-density-cliff` — a critical born-red finding with peer-reviewed negative control.

**The teeth-check first**: Test-architect ran an INDEPENDENT 4th derivation (naive Python BFS, lineage-disjoint from ascent) on callgraph.json to verify E1. Result: **33,882 pairs — EXACT MATCH**. E1 is genuinely teeth-checked, not three-bugs-agreeing.

**Nuance found**: 10 of 33,882 are REFLEXIVE (reachable from self via cycle). The relation is 33,872 strict-downstream + 10 cycle-reflexive. Any spike asserting "33882" must state which interpretation.

**THE DENSITY CLIFF (the real finding — born-red):**

Measured degenerate battery (naive BFS on synthetic graphs):

| Graph type | Nodes | Pairs | Time |
|-----------|-------|-------|------|
| antigen real (0.4% dense) | 2948 | 33,882 | ~2.5ms |
| Chain k=2000 | 2000 | 1,999,000 | 0.205s |
| Single cycle k=2000 | 2000 | 4,000,000 (100% dense) | 0.347s |
| Random k=3000 avg-deg=10 | 3000 | 9,000,000 (100% dense) | 4.3s |

**The born-red claim**: L2 "relational-as-base validated at n=2948" does NOT transfer to the deps-included graph. Workspace-only = 2948 fns / 352 item-trees. r-a analysis shows DEPS = 5,880 item-trees / 2.9M LOC. **Nobody has measured the closure size or ascent runtime on the deps-included node set.** If cross-crate resolution densifies the graph (hubs like `syn::` / `salsa::` pulling many callers), the closure could go superlinear — full-materialization becomes intractable, FORCING demand-driven (salsa-PULL / magic-set) over whole-relation batch.

**Observer's assessment of ATK-CLOSURE-DENSITY-CLIFF:** CRITICAL FINDING. This is exactly what the spike-yard exists to find. The observer assessment:

1. The negative control for E1 is the right methodology — independent derivation, not same-lineage-agreement. The teeth-check is now genuinely done for E1 at n=2948.

2. The density cliff measurement is real and important. The "antigen is sparse" assumption is baked into every "relational-as-base wins" result from the crucible. If the deps-included graph densifies, the entire L2-DECIDED assumption may need reopening.

3. **This is not a finding about whether relational-as-base is wrong** — it's a finding about where the current evidence runs out. The experiment to close this is: run ascent reachability on the deps-included graph (use `ra_ap_*` to pull in 5,880 item-trees) and report closure-size + runtime + memory.

4. **Peer-review concern**: The born-red claim correctly targets a real gap, but the evidence for the "superlinear danger" is synthetic random graphs — not antigen's actual deps graph. The deps graph has known hub structure (syn, proc-macro2, salsa) but is NOT a random graph. Actual hub-and-spoke graphs (like Rust ecosystem dependency trees) tend to be SPARSE at the call-edge level even when they have many nodes, because most crate dependencies are used for a few entry-points. The born-red is warranted but the severity estimate (could go superlinear) should be flagged as ESTIMATED, not MEASURED, until the actual deps run lands.

**Required follow-up**: Scientist must run the deps-included closure measurement. This is pre-ratification gate material.

---

### Pathmaker: Spike/DM4-Salsa-Ascent-Compose (in progress)

Island `spike/dm4-salsa-ascent-compose` opened with the exact experimental design:

**Hypothesis**: In a composed stroma, the edge-relation feeding ascent is ITSELF a salsa-tracked derived value. Salsa's per-file delta is thrown away at the ascent boundary (whole-relation batch). The two revision models may fight.

**Method**: Build salsa-tracked edges feeding an ascent reachability query. Instrument:
- (a) Does the composition typecheck/compose cleanly or need a bespoke bridge?
- (b) Cost of an ascent full re-run at n=2948?
- (c) How does salsa's backdate gate whether ascent re-runs at all?

**Status**: Spike opened, not yet complete. Required signers: pathmaker + scientist.

**Observer's assessment**: This is exactly the right experiment. The DM4 composition question is the highest-risk structural unknown in the engine design. The experiment design is sound. One clarification needed: "(c) how salsa's backdate gates whether ascent re-runs" — the key measurement is WHEN the backdate logic kicks in and whether it correctly suppresses ascent re-runs for age-only changes (no structural change). This is the temporal gate applied at the engine composition level.

---

### Expansionist: DBSP Z-Set Algebra (cross-domain borrow)

Two field saves and a full term-by-term map on island `expansionist-cross-domain-stage-survey`.

**The domain borrow**: DBSP (Budiu et al, VLDB 2023) — Z-sets (relations weighted by commutative group Z) + integration/differentiation operators. Key claim: subsumes salsa + datalog + differential-dataflow into ONE engine via the chain rule: (Q1∘Q2)^Δ = Q1^Δ ∘ Q2^Δ.

**The term-by-term map** (antigen → DBSP):

| Antigen concept | DBSP term |
|----------------|-----------|
| Fact-set (call/dataflow edges) | Z-set: relation weighted by group Z |
| Salsa revision / edit | Stream element Δ |
| Whole-relation publish per revision | Integration operator I |
| Salsa demand-PULL memoization | Differentiation operator D |
| 4-semiring queries | DBSP circuit Q over Z-sets |
| Incremental query | Q^Δ = D ∘ ↑Q ∘ I |
| Salsa/datalog revision-model fight (DM4) | Dissolved by ONE clock (DBSP time = ℕ, single predecessor) |
| Least-fixpoint datalog | Semi-naive evaluation in recursive DBSP sub-circuit |

**The critical constraint**: DBSP time is TOTAL ORDER (single predecessor). Differential-dataflow generalizes to a LATTICE (partial order) for multi-writer / iterative case. If antigen needs partial-order time (concurrent edits from multiple writers), DBSP is too weak and full DD is required. **This is a MEASURABLE question**: does antigen have genuinely concurrent revision sources, or one serial edit-clock?

**Magic-Set second save**: Magic-Set / Demand Transformation (Tekle-Liu) — SOURCE-TO-SOURCE rule rewrite that makes bottom-up datalog demand-driven without a glue boundary. The named formal bridge for the salsa-demand-PULL + datalog-whole-relation-PUSH seam.

**Observer's assessment of DBSP borrow**: SIGNIFICANT CROSS-DOMAIN FIND. If DBSP's claim holds (one algebra subsumes three), it directly dissolves the DM4 composition risk the pathmaker is spiking. The term-by-term map is complete — every antigen concept has a DBSP term. The one constraint (serial vs concurrent time) is exactly the question to measure. However:

**Peer-review concern**: The expansionist's claim is analytic (term-map), not empirical (measured on antigen). A term-map can be complete while the implementation is wrong. The DM4 spike (pathmaker) is testing the composition empirically. DBSP provides a theoretical framework for WHY the composition should work; the spike tests WHETHER IT DOES in practice with Rust libraries. Both are needed.

**Feldera** is named as a Rust DBSP library. This is a scout-level finding that should be followed up: what is Feldera's API surface? Does it integrate cleanly with salsa's revision model?

**Also filed**: Notice on Magic-Set as the formal bridge for salsa-demand-PULL + datalog, tagged `#cross-domain #engine-seam`.

---

### Naturalist: Three Structural Notices

**Notice 1 — Vocabulary-Foreclosure Antidote** (tagged `#bench-over-text`):
The spike-yard's "antigen is the corpus AND the judge" fitness function is the structural cure for three problems the crucible identified: (a) vocabulary-foreclosure (biology pre-answers implementation), (b) text-primed convergence (independent readers converging because they read the same charters), (c) self-witness on design. Bench-derived evidence dissolves all three at once.

**Notice 2 — Bench-Over-Text Rhyme** (tagged `#bench-over-text`):
The crucible's contrarian found its own triple-convergence was text-primed. The one finding the contrarian trusted was the 52% edge-miss measurement. The spike-yard is structurally the answer: findings come from measured spikes on real code, not re-reading the frame.

**Notice 3 — Metaphor-As-Process, Not Justification** (tagged `#metaphor-discipline`):
The captain's evolution frame does NOT re-foreclose. The discriminator: metaphor-as-justification answers a STAGE QUESTION (e.g., "anergy says suppression should be continuous"). Metaphor-as-process-description answers NO stage question (it describes the search loop, not any stage verdict). The germinal-center analogy is safe because it touches the team's OWN LOOP, not any implementation choice.

**Observer's assessment of naturalist notices**: All three are substantive. Notice 1 is the cleanest synthesis: the spike-yard IS the methodology-fix the crucible identified as missing. Notice 3 is important for the observer to remember — I should not flag the captain's evolution frame as vocabulary-foreclosure. The discriminator (process vs justification) is the correct test. Notice 2 is documented for the record.

---

## Updated Implementation-Findings Map — After Wave 1

### Stage-level updates (replacing UNKNOWN where new evidence exists):

**STAGE 1 — INPUT**: New evidence from ATK-CLOSURE finding: the relevant question is not just resolver depth for antigen-workspace edges but DEPS-INCLUDED resolution. The deps graph (5,880 item-trees) is the real measurement target. Status: **PARTIALLY CHARACTERIZED** — workspace-level edges measured; deps-level unmeasured (born-red gap).

**STAGE 2 — STRUCTURE/BASE**: Density cliff finding directly attacks this stage. L2=relational-as-base is DECIDED at n=2948/0.4% density. At deps-included scale, the closure size is unknown. Status: **CONDITIONALLY DECIDED** — holds at workspace scale; deps-scale is the open pre-ratification gate.

**STAGE 3 — ENGINE**: DM4 spike in progress. DBSP offers a theoretical dissolution of the revision-model clash. Status: **SPIKE IN PROGRESS** — pathmaker spiking dm4-salsa-ascent-compose; DBSP term-map available as theoretical framework.

**STAGE 4 — FRESHNESS**: No new wave 1 measurements. Linkage constraint LE6 (last_changed as input, NEVER tracked-fn) is now documented in the linkage map. Status: **UNCHANGED — UNKNOWN at real scale**.

**STAGE 5 — OUTPUT**: Linkage edges LE7 and LE8 confirmed as the highest-value OPEN linkage. injected-in-source creates two self-referential back-edges (to freshness and to input). sidecar relaxes both. Status: **OPEN — highest discovery value per systems-thinking-expert**.

**STAGE 6 — CONSUMPTION**: LE5 linkage confirmed: blast-as-COUNT is NON-VIABLE (85,000x cost difference). Blast-as-SET is the forced allele. Status: **PARTIALLY CONSTRAINED** — blast semiring allele forced; rest unmeasured.

---

## Updated Running Verdict Table (After Wave 1)

| Claim | Verdict | Strength | Evidence |
|-------|---------|----------|---------|
| E1 — 33,882-pair reachability | VERIFIED (with nuance) | HIGH | Independent BFS 4th derivation matches. 10 reflexive pairs = cycle signature. Teeth-checked. |
| E1 — at deps scale | UNVERIFIED (born-red) | HIGH concern | Density cliff measurement shows antigen is 0.4% dense; deps-included graph unmeasured. Could go superlinear. |
| L2 relational-as-base | CONDITIONALLY DECIDED | MEDIUM | Decided at n=2948; deps-scale is open pre-ratification gate. |
| L3 salsa+datalog compose | SPIKE IN PROGRESS | — | DM4 spike opened by pathmaker; DBSP theoretical framework maps cleanly but empirical result pending. |
| DBSP subsumes salsa+datalog | ANALYTIC (unmeasured) | — | Term-map complete; Feldera (Rust lib) needs API surface check; empirical composition test pending. |
| Linkage edges LE1–LE8 | MAPPED (structural reasoning) | HIGH | Systems-thinking map is substrate-consistent and linkage-sound. Cross-stage interaction effects documented. |
| Spike-order recommendation | L1 FIRST, L5 SECOND | HIGH | Leverage analysis sound; observer endorses, with caveat that DM4 composition risk may elevate L3 above "#9 parameter." |

---

## Observer Questions (Updated After Wave 1)

1. **For scientist** (URGENT): Run the deps-included closure measurement. Use `ra_ap_*` to pull in the 5,880 item-tree graph. Report closure-size + runtime + memory for ascent reachability. This is the test-architect's born-red blocker and a pre-ratification gate for L2.

2. **For pathmaker** (DM4 spike): When the salsa+ascent composition result lands — report BOTH the success/failure of composition AND the specific latency measurement for a full ascent re-run at n=2948 (the warm-path cost). The backdate-gating result is the most important: does a non-structural edit (whitespace, comment) suppress the ascent re-run?

3. **For scout**: Check Feldera (the DBSP Rust library). API surface? Integration cost with salsa? Does it support the provenance semiring natively, or would the 4 semirings require custom Z-set weight types?

4. **For systems-thinking-expert**: DM4 composition risk — the pathmaker is treating this as a structural spike (can it compose?), but the revision-model fight is a RULES-LEVEL concern (Meadows #5), not a parameter. Should this elevate L3-backend from "#9 parameter" to a higher leverage priority?

5. **For test-architect** (measurement scaffold): The observer has seeded `the-spike-yard/measurement-scaffold` with 5 pre-conditions. Will the test-architect endorse these as sufficient, or are there additional ATK requirements? Specifically: what is the born-red test for EACH fitness dimension (F-CORRECT, F-FRESH, F-INCR, F-INTEG)?

---

## Camp Substrate Alignment Notes (After Wave 1)

Active islands in the-spike-yard after Wave 1:
- `the-evolution-frame` — OPEN (captain to sign; observer frame note deposited)
- `the-spike-yard/measurement-scaffold` — OPEN (observer owns; methodology-gap tagged)
- `expansionist-cross-domain-stage-survey` — OPEN (expansionist to sign)
- `sys/genome-space-linkage-map` — OPEN (systems-thinking-expert to sign)
- `sys/spike-order-leverage-terrain` — OPEN (systems-thinking-expert to sign)
- `spike/atk-closure-density-cliff` — OPEN (test-architect to sign)
- `spike/dm4-salsa-ascent-compose` — OPEN (pathmaker + scientist to sign)

**Substrate alignment concern**: The linkage map (LE3) notes that L1=r-a-only FORBIDS a sound parity-oracle for resolved edges. This directly impacts the antigen self-dogfood case (antigen must witness its OWN immune-system claims). The parity-oracle linkage constraint should be noted on the evolution-frame island for the captain's recombination decisions.

---

## Wave 2 — Second Burst (~23:46–23:49 UTC, 2026-06-28)

33 more events. Major developments: DM4 spike COMPLETED with measured result; test-architect cracks the 4% FP claim; systems-thinking-expert adds LE9 linkage edge and coherent chromosomes catalog; naturalist identifies re-grounding as the expedition's deep grammar and finds the DBSP/DD deciding question already answered in antigen's own git history.

---

### DM4 Spike — COMPLETED (pathmaker result)

Island `spike/dm4-salsa-ascent-compose` — RESULT LANDED.

**Pathmaker result** (spike built + run, real callgraph n=2948/4714 edges):

| Assertion | Result | Measurement |
|-----------|--------|-------------|
| (A) Compose cleanly | PASS | Salsa-tracked query holds ascent program inline; ascent consumes salsa-published Vec<(u32,u32)> — zero bespoke bridge |
| (B) Faithful output | PASS | 33,882 reach-pairs == crucible standalone ascent exactly |
| (C) Backdate gates re-run | PASS | Byte-change with identical derived edges → file_edges +1, all_edges +0, reach +0 → ascent closure GATED OUT. 37.6μs vs 3.34ms cold = 89x cheaper |
| (D) Honest cost when meaning changes | MEASURED | Ascent re-runs WHOLE closure (3.08ms) — salsa's per-file delta (63/64 buckets untouched) thrown away at ascent boundary |

**The architectural result**: salsa = the GATE (does the closure re-run at all?). Ascent = the BATCH (when it does, it's whole-graph). The danger-model (meaning-change not byte-change) protects the non-incremental ascent boundary FOR FREE via salsa's backdate.

**Test-architect's born-red criteria** (ATK-DM4-001, measured before the spike ran):
- Median edit dirties 1 source-closure → ascent recomputes all 2948 = 2948x waste ratio
- Hub edit (idx 1186): 181 affected → still 2948x recompute
- The waste ratio is confirmed: salsa's per-file delta IS thrown away. The backdate gating is what makes this acceptable.

**Observer's assessment of DM4 result: SIGNIFICANT POSITIVE FINDING with an open scaling question.**

DM4 is answered at n=2948: the revision models layer cleanly rather than fighting. The mechanism is elegant — salsa backdate prevents ascent from running at all on meaning-unchanged edits, so the "whole-relation batch cost" only applies when something actually changed. At n=2948 (3.08ms), this is fast enough to be irrelevant.

**The open question (and it IS open)**: The pathmaker explicitly names it — "does the all-or-nothing batch cost survive 10x–100x scale?" This is exactly what the LE9 linkage edge (systems-thinking-expert, see below) formalizes. At n=2948, 3ms is fine. At n=88,000+ (deps-included), the same all-or-nothing batch may be seconds-to-intractable. The DM4 spike answered the SPARSE case; the DENSE case remains the born-red blocker.

**What this means for the chromosome catalog**: CH-A (sparse workspace baseline) has L3=salsa+ascent confirmed-coherent. CH-B (deps-scale stress) has L3 CONDITIONAL — if deps graph stays sparse, ascent batch stays cheap and L3=ascent holds; if it densifies, demand-driven evaluation is forced.

**Peer-review of the DM4 result**: The measurement is correct. One subtle point: assertion (C) passing means "a COMMENT EDIT does not trigger ascent re-run." This is an important correctness guarantee. But the falsification would be: does a SEMANTIC change that produces ZERO new call-edges (e.g., replacing one implementation of an interface method with another, same callers, same callees, but different semantics) correctly trigger or suppress? That case is not in the spike. It's unlikely to matter at this stage — the danger-model is about resolved call edges, not arbitrary semantic changes — but it's worth noting as an untested edge case.

---

### ATK-NAME-COLLAPSE-LAUNDERS-FP — New Critical Born-Red

Island `spike/atk-name-collapse-launders-fp` — a crack in the 4% FP floor.

**The crack**: The crucible's EXP-4 "failure-mode-is-SILENCE-not-noise" conclusion (4% FP rate) rests on `compare_edges.py` normalizing edges to `(caller_NAME, callee_NAME)`. This normalization is **blind to the dominant error class**: wrong-target edges.

**Measured (raw symbol scan, bypassing protobuf mismatch)**:
- ~6,310 antigen fn symbol monikers; 3,051 distinct names
- 2,492 names (81.7%) are AMBIGUOUS (>1 distinct symbol)
- 5,751 symbols (91.1% of all fn symbols) live under an ambiguous name
- Worst offenders: `parse`=121 symbols, `validate`=80, `new`=40, `eq`=37, `clone`=33

**Why this launders false positives**: The regex heuristic resolves call targets by NAME ONLY. When it emits edge `(foo, parse)` intending `foo→M1::parse` but the true resolved target is `foo→M2::parse`, the name-level set-diff sees `(foo, parse)` in both sets and scores it CONFIRMED. With 91% of symbols name-ambiguous, the heuristic is structurally prone to wrong-target edges that the benchmark cannot see.

**Consequence for the ADR-067 §7 conservatism claim**: "False-quiet is the safe failure mode" — this may be FALSE for the syntactic tier. A wrong-target edge IS noise (it pollutes the reachability closure with edges that don't exist, producing confidently-wrong downstream answers). The "silence > noise" conclusion only holds if the dominant error mode is MISSING edges, not WRONG-TARGET edges. This has not been verified.

**Tooling finding**: `scip_pb2.py` fails to import (gencode 6.33.5 vs runtime 5.29.6 — protobuf version mismatch). The 52%/4% numbers came from a protoc/runtime pairing that no longer loads. A re-validation spike must regenerate `scip_pb2` against the installed runtime BEFORE any re-diff.

**Observer's assessment of ATK-NAME-COLLAPSE: CRITICAL METHODOLOGICAL FINDING.**

This is the highest-severity finding in Wave 2 from a peer-review standpoint. The 4% FP floor is one of the load-bearing evidence items for the "syntactic tier degrades gracefully" claim. If the true FP rate at symbol level is materially above 4%, the ADR-067 conservatism claim needs revision.

**Severity assessment**: The finding is logically airtight — the normalization method IS blind to wrong-target edges, and 91% name-ambiguity IS measured. But the MAGNITUDE of the actual wrong-target rate is not yet measured. It's possible the heuristic, despite name-resolution, gets the right target most of the time via co-occurrence or structural context. The born-red is warranted: measure the true symbol-level FP rate before relying on the "silence not noise" claim.

**Routing**: This routes to scientist for the symbol-level re-diff (once protobuf is fixed). The observer's question `347cdbe0` (deps-closure measurement) should be followed by a second question on this.

**Impact on the findings map**: STAGE 1 — INPUT: The syntactic-only allele (syn-only, heuristic edges) may be worse than "52% miss + 4% FP." If the true FP rate is materially higher, the correct characterization is "52% miss + X% noise," where noise includes wrong-target edges. This is a STRONGER argument for resolved-edge input (SCIP/r-a) — the 52% miss is bad enough; noise-on-top makes the syntactic tier actively harmful for downstream consumers.

---

### LE9 Linkage Edge — Systems-Thinking Update

Systems-thinking-expert added LE9 to the linkage map and SELF-CORRECTED the earlier "#9 parameter" assessment:

**LE9**: L1-SCOPE (workspace vs deps-included) ==> L3-ARCHITECTURE (batch vs demand-driven), via graph DENSITY, MEASURED-PARTIAL.

- L1=workspace-only → sparse → whole-relation batch (ascent .run()) is CHEAP → L3-backend is #9 parameter ← **my earlier reading, holds HERE**
- L1=deps-included → POSSIBLY dense (unmeasured) → IF dense, full materialization INTRACTABLE → FORCES demand-driven (magic-set / seminaive-from-query-seed) → L3 architecture flips from #9-PARAMETER to #5-RULE

**The self-correction is exemplary**: The systems-thinking-expert flagged that their own prior note on L3-as-#9-parameter was PARTIALLY STALE after the test-architect's density cliff finding. This is exactly the substrate-over-memory discipline applied correctly — the substrate (density cliff measurement) updated the model (L3 is a parameter) and the correction landed in the substrate (LE9 note).

**The meta pattern** (systems-thinking-expert's own observation): "A fitness measured on antigen's CURRENT (sparse, workspace-only) self is not transitively a fitness at antigen's FULL (deps-included) scale." This generalizes: every GEN-1 measurement must be tagged with the scale at which it was taken.

---

### Coherent Chromosomes Catalog — Complete

Island `sys/coherent-chromosomes-catalog` SIGNED COMPLETE by systems-thinking-expert.

**Non-viable chromosomes (X-region — never spike here)**:

| ID | Chromosome | Why Dead |
|----|-----------|----------|
| X1 | syn-only + semantic-fresh + flow-consuming | VIOLATES LE1+LE2 — 52% wrong, can't see meaning-change |
| X2 | property-graph + salsa+datalog | VIOLATES LE4 — crucible EXP-3 measured this as losing path |
| X3 | count-semiring + blast-radius consumer | VIOLATES LE5 — 206s + ill-defined on cycles |
| X4 | last_changed-as-clock-read | VIOLATES LE6/VL3 — pin-at-zero, hit twice in prototypes |
| X5 | r-a-only + self-distrust/parity | VIOLATES LE3/C2 — shared-upstream blind spot |

**Viable candidates**:

| ID | Description | L5 | Status | Priority |
|----|-------------|-----|--------|----------|
| CH-A | Sparse-workspace baseline (crucible decided core) | OPEN | Validated at n=2948; the CONTROL | Use as baseline |
| CH-B | Deps-scale stress variant (CH-A with deps-included scope) | OPEN | L3 architecture CONDITIONAL on density | HIGHEST priority (steepest gradient) |
| CH-C | Self-distrust-honest variant (requires MIR as oracle source) | OPEN | Tests LE3 — the only coherent self-distrust chromosome | When parity-oracle consumer in scope |
| CH-D | Injected-output with governor (tests A6 recursion) | injected-in-source | Coherent ONLY with effector-writes-excluded-from-danger governor | Second priority |
| CH-D' | Sidecar variant (relaxed governor) | git-notes-sidecar | Cheaper coherence than CH-D | Second priority (pair with CH-D) |

**Observer's assessment**: The chromosome catalog is the right abstraction for the captain's recombination. It makes explicit which combinations are non-viable (saving wasted spikes) and which are the highest-value experiments. The CH-B finding confirms the observer's Wave 1 assessment: the deps-scale closure measurement IS the highest-leverage spike.

**Freely swappable within linkage groups**: L3-datalog-backend (ascent/crepe/datafrog/Souffle) at sparse scale. L6 consumers (additive, as long as each respects its semiring cost). Do NOT recombine across named linkage edges.

---

### Naturalist: Re-Grounding as Deep Grammar

**Notice: Convergence across 4 naturalist observations** — the spike-yard's deep grammar is RE-GROUNDING: every move takes a decision being made by a weaker authority and promotes it to a stronger evidence register.

1. Vocabulary-foreclosure → MEASUREMENT (metaphor demoted; bench decides)
2. Text-primed convergence → BENCH-DERIVED (same charters → same real code)
3. Evolution frame → PROCESS-DESCRIPTION (metaphor demoted from deciding-authority to describing-the-search)
4. Engine deciding-question → ALREADY IN ANTIGEN'S GIT HISTORY (strongest ground: not a fresh spike, a recall)

**The corollary the naturalist names**: "antigen is the corpus AND the judge" has a corollary the launch didn't state: **the corpus has a MEMORY**. Some spike-yard measurements are RECALLS, not fresh runs. The cheapest spike is the one antigen already did.

**Three vocabularies, one scalar** (selection pressure = leverage = information-per-spike): The naturalist observed that the systems-thinking-expert's leverage analysis and the captain's evolution-frame are measuring the SAME scalar at each locus. At L1, all three read MAX. At L3-datalog-backend, all three read ~ZERO. This is what "evolution-frame as process" looks like when it's generative: it tells you WHERE to spend variation.

---

### DBSP vs DD — Already Answered in Antigen's STOCK

**Naturalist notice** (routed to expansionist): The expansionist's DBSP-vs-DD deciding question ("does antigen have serial or partial-order time?") is ALREADY ANSWERED in antigen's git history.

Commit `19647b8` ("name the STOCK multi-writer merge-order seam, adversarial design-stress against ADR-059") contains:

- `is_retired()` = COMMUTATIVE EXISTENCE FOLD — merge-order invariant, monotone → this is a Z-set / commutative-group structure → DBSP-native
- `trajectory_direction()` = ORDER-SENSITIVE (first-vs-last over append order) → named MULTI-WRITER SEAM, needs total-order key → DD or timestamp

**The map is NOT a coin-flip**: commutative/merge-safe reads → DBSP-native; order-sensitive reads → need the total-order key DD provides. Antigen's own adversarial analysis already drew the first edge of this map.

**Observer's assessment**: This is the naturalist's deepest finding. It exemplifies the "recall not re-run" principle — the most expensive theoretical question of the expansionist's cross-domain survey has a partial empirical answer sitting in a prior expedition's adversarial stress work. The observer will note this as a methodology pattern for the findings map: BEFORE launching a new spike, check whether antigen's ADR/adversarial history already ran it.

---

## Updated Implementation-Findings Map — After Wave 2

### Stage fitness updates:

**STAGE 1 — INPUT**: Status upgraded from NOT YET SPIKED to PARTIALLY CHARACTERIZED WITH BORN-RED GAP.
- Syntactic (syn-only): 52% miss rate CONFIRMED (teeth-checked). FP rate: UNKNOWN at symbol level — 4% is an upper bound from a name-collapsed benchmark blind to wrong-target edges. ATK-NAME-COLLAPSE born-red pending.
- SCIP batch: Still 4647 vs 2332 edges. Cold start 13-49s. The correct edge count is confirmed by independent BFS.
- `ra_ap_*`: Unmeasured integration cost vs SCIP. Still the key comparison.
- **The resolver-SCOPE sub-allele** (workspace vs deps-included) is now a named linkage axis.

**STAGE 3 — ENGINE**: DM4 spike COMPLETE.
- salsa + ascent: COMPOSE CLEANLY (zero bespoke bridge). Backdate gating works. F-INCR=MEASURED (89x cheaper on non-structural edits). F-CORRECT=VERIFIED (33,882 pairs match).
- **LE9 caveat**: At workspace scale (n=2948, sparse), the batch cost (3.08ms) is irrelevant. At deps-scale (unmeasured), may be forced to demand-driven. L3=ascent is CONDITIONALLY DECIDED.

### Updated Running Verdict Table

| Claim | Verdict | Strength | Evidence |
|-------|---------|----------|---------|
| DM4 — salsa+ascent revision models fight | FALSE (at workspace scale) | HIGH | Direct measurement: 33,882 pairs, zero bespoke bridge, backdate gates correctly (89x cheaper) |
| DM4 at deps-scale | OPEN (LE9) | — | All-or-nothing batch cost at 10–100x scale unmeasured |
| 4% FP floor (syntactic tier) | UPPER-BOUND (name-collapsed, blind to wrong-target) | MEDIUM concern | 91.1% name-ambiguity MEASURED; symbol-level re-diff not yet run |
| "Silence not noise" (ADR-067 §7) | CONDITIONALLY HOLDS | MEDIUM | Holds IF wrong-target rate is low; ATK-NAME-COLLAPSE challenges this |
| CH-A (sparse-baseline chromosome) | COHERENT (L1–L4) | HIGH | DM4 spike confirms L3=ascent+salsa at n=2948; L5 open |
| Coherent chromosomes catalog | COMPLETE | HIGH | Systems-thinking-expert signed; X-region cataloged; CH-A through CH-D/D' viable |
| DBSP-vs-DD deciding question | PARTIALLY ANSWERED (from STOCK) | HIGH | Commutative reads → DBSP-native; order-sensitive reads → need total-order key (commit 19647b8) |

---

## Updated Observer Questions (After Wave 2)

1. **For scientist** (URGENT, pre-ratification gate): Run deps-included closure measurement (question `347cdbe0` routed). CH-B spike = highest-information spike on the board.

2. **For scientist** (URGENT, new): Run symbol-level heuristic-vs-SCIP edge diff (re-diff against SCIP at `(caller_symbol, callee_symbol)` level, not name-collapsed). Must regenerate `scip_pb2.py` against installed protobuf first. Report true symbol-level FP rate. Island: `spike/atk-name-collapse-launders-fp`.

3. **For pathmaker** (DM4 follow-up): The DM4 spike answered the sparse case. The next spike is CH-B: does the all-or-nothing batch cost survive 10x–100x scale? This is the LE9 deciding experiment.

4. **For expansionist**: Confirm or route the STOCK finding (naturalist notice `27d0e9a8`): is the DBSP/DD read-class map (commutative → DBSP, order-sensitive → DD) complete from commit `19647b8`, or are there additional read classes in the stroma design that need classification?

5. **For test-architect**: The measurement scaffold now has a linkage-aware negative control design (systems-thinking-expert note on `measurement-scaffold`): each stage's clear-failure test is a chromosome that violates one named linkage edge. Will you build the negative-control battery from the X-region catalog?

---

## Camp Substrate Alignment Notes (After Wave 2)

Completed islands:
- `sys/genome-space-linkage-map` — SIGNED COMPLETE (systems-thinking-expert)
- `sys/spike-order-leverage-terrain` — SIGNED COMPLETE (systems-thinking-expert)
- `sys/coherent-chromosomes-catalog` — SIGNED COMPLETE (systems-thinking-expert)

Open spikes:
- `spike/dm4-salsa-ascent-compose` — result landed (pathmaker note); awaiting pathmaker + scientist signatures
- `spike/atk-closure-density-cliff` — born-red standing; awaiting test-architect signature
- `spike/atk-name-collapse-launders-fp` — born-red standing; awaiting test-architect + scientist signatures

Durability note: All completed islands are "working-tree — not yet durable (uncommitted)." The captain should ensure a commit lands before the end of GEN-1 selection, or the substrate is at risk on a branch reset.

---

## Wave 3 — Third Burst (~23:50–23:53 UTC, 2026-06-28)

19 events. Three major developments: captain GEN-1 selection directive, DBSP as a paradigm-level engine collapse candidate (LE10 added), test-architect builds the negative-control battery and routes scaffold completion to the observer, naturalist adds Pareto-frontier framing.

---

### Captain GEN-1 Selection Directive

Island `gen-1-selection` opened by captain. Three directives:

1. **PRIORITY SPIKE = CH-B**: Run ascent reachability on deps-included graph (n~5,880 item-trees / 2.9M LOC). Report closure-size + runtime + memory. This decides whether the engine-core is DECIDED (sparse) or CONDITIONAL (dense = demands demand-driven, DM4 flips from non-issue to load-bearing).

2. **SELECTION DISCIPLINE**: Tag EVERY fitness number with the scale measured at (workspace n=2948 vs deps-included). Local-correctness does not compose into global.

3. **Design-feedback gate**: If CH-B comes back superlinear — surface to captain immediately. This is a design-feedback finding permitted by launch.md.

**Observer's assessment of the captain's directive**: Correct and aligned with what the observer and systems-thinking-expert independently derived. The CH-B priority designation is justified by the linkage analysis — the density measurement is the gating variable for three downstream architectural decisions. The scale-tagging discipline is the operationalization of "local correctness does not compose into global."

**Substrate alignment**: The captain's directive on `gen-1-selection` is the substrate-resident record. The observer's questions `347cdbe0` (deps-closure) and `38ad4ba2` (name-collapse FP) both feed into this directive. The captain's selection will happen here — the observer must ensure all GEN-1 findings land on islands the captain can read from this island.

---

### DBSP as Paradigm-Level Allele — LE10 Added

Systems-thinking-expert deposited a paradigm-level note on `sys/genome-space-linkage-map` and a route to the expansionist, adding:

**LE10**: writer-cardinality ==> engine-algebra:
- one-serial-edit-clock → DBSP (single-predecessor time) suffices → LE4/LE6/LE9 COLLAPSE → engine-core is ONE algebra, density cliff dissolved
- concurrent-multi-writer → DBSP too weak → full DD (lattice time) required → collapse partially un-does → density cliff returns as Δ-rate-under-concurrency cost

**What DBSP collapse would mean** (if it holds at antigen's real recursive scale):
- LE4 (structure ↔ engine) becomes trivial: Z-set IS both structure and engine object
- LE6 (freshness ↔ salsa-discipline) becomes automatic: backdate/`last_changed` discipline is a property of the Δ-stream, not a hand-wired salsa rule
- LE9 (L1-scope → L3-architecture via density) DISSOLVES: DBSP pays Δ-cost not n² full-closure cost → density cliff doesn't force demand-driven

**The three-arm CH-B outcome space** (systems-thinking-expert and captain now agree on this framing):
- (a) deps-sparse → ascent-batch decided → CH-A holds at scale
- (b) deps-dense + DBSP-recursive-Δ stays sub-n² → DBSP dominates, engine-core collapses to ONE algebra, density moot
- (c) deps-dense + DBSP-recursive-Δ ALSO superlinear → genuine re-architecture forced, demand-driven/magic-set required, DM4 load-bearing

**The gate on arm (b)**: LE10 writer-cardinality. Single-tenant 0.7 = one serial clock → arm (b) AVAILABLE. 0.8 universal-stroma multi-writer → arm (b) AT-RISK. The naturalist found this gate is ALREADY partially answered in antigen's STOCK (commit `19647b8`): `is_retired()` = commutative (DBSP-native), `trajectory_direction()` = order-sensitive (needs total-order key). For 0.7's scope, one clock holds.

**The measurable question** before the captain trusts the DBSP collapse: does a single-edge Δ on a RECURSIVE reachability circuit cost O(affected-closure) or O(full-closure) at deps density? If sub-n², arm (b) is real and the whole engine-core question reframes. If superlinear, arm (c) stands.

**Observer's peer-review assessment of the DBSP collapse proposal**: SIGNIFICANT, with important caveats.

The LE10 addition is the most structurally consequential move in Wave 3. If DBSP's chain-rule claim holds on antigen's recursive reachability closure at deps density, it changes the LINKAGE TOPOLOGY — not just an allele's fitness value. That's a paradigm-level event (Meadows #2) worth spiking.

**Caveats to track**:

1. **Recursive fixpoint cost**: The DBSP papers prove incrementality is compositional for FIRST-ORDER queries (the chain rule). Recursive/nested fixpoints (transitive closure) require the semi-naive fixpoint inside the DBSP circuit. The Δ-cost on a recursive circuit is NOT trivially O(affected-closure) — it depends on the query's least-fixpoint semantics. This needs empirical verification at deps scale, not just analytical confidence.

2. **Feldera integration cost**: DBSP is available as Feldera (a Rust library). But integrating Feldera into the antigen workspace, with its `salsa 0.27` dependency and existing build infrastructure, carries an integration cost the term-map doesn't capture. This is F-INTEG, which the DBSP analysis hasn't priced.

3. **The "collapses LE4/LE6/LE9" claim is analytic**: It's derived from the structure of DBSP, not measured on antigen's code. Until a DBSP spike actually runs antigen's reachability query, "collapse is real" is a theoretical prediction.

**Bottom line**: The DBSP arm (b) is worth adding to CH-B as a parallel measurement, NOT as a replacement for the ascent-batch measurement. Run both. The worst case is 2× the spike work; the best case is the density-cliff fight dissolves before it starts.

---

### Test-Architect: Measurement Scaffold Negative Controls — BUILT

Test-architect built scaffold point 5 (negative controls) and routed to the observer:

**What's in `scratchpad/measurement-scaffold/`**:

| File | Contents |
|------|---------|
| `CORPUS.txt` | Pinned corpus identity: HEAD `bc8df9d`, n=2948, edges=4714, closure=33,882 (33,872 strict + 10 cycle-reflexive). Flags deps-included scale as UNMEASURED. |
| `EXPECTED.json` | Known-correct answer + named failure-mode per specimen |
| `negative-controls/NC1.json` | Diamond graph: reachability=1, npaths=2. Catches double-count/missed-join. |
| `negative-controls/NC2.json` | Self-cycle a↔b: BOTH reach self via cycle. Catches reflexive-via-cycle DROP (the off-by-10 signature). |
| `negative-controls/NC3.json` | Name-ambiguity: foo calls parse@M1 ONLY; NAME-ONLY resolver may emit wrong 'parse'. ANY edge to node 3 = FAIL. ATK-INPUT-001 minimized. |
| `negative-controls/NC4.json` | Disconnected graph: isolated node + two components. No phantom pairs. Catches node-drop/hallucinated edge. |
| `negative-controls/NC5.json` | Full-closure 20-cycle: closure MUST be exactly 400 (n²). Catches SILENT TRUNCATION. Density-cliff failure in miniature. |
| `check_battery.py` | Self-grading runner: spike imports `reach_pairs(n,edges)`, calls `check_all()`, exits 1 on RED. TEETH-CHECKED: wrong engine triggers NC2+NC5 RED. |

**Critical property**: The runner is TEETH-CHECKED — a deliberately wrong engine (drops reflexive, truncates) verifies the runner correctly flags RED. "Correct engine → all PASS; wrong engine → NC2+NC5 RED. The runner can fail, so its green means something."

**What the test-architect says is STILL OWED by the scaffold** (routed to observer):
- Observer's scaffold points 1, 3, 4: pinned-corpus persistence file in the repo, fixed timing harness specification, the 4-semiring 5-pair query fixture
- Systems-thinking-expert's ask: a deps-included snapshot (corpus identity doesn't exist yet — needs the same spike as ATK-CLOSURE-001)

**Observer's assessment**: The negative-control battery is well-designed. NC3 (name-ambiguity) directly operationalizes ATK-NAME-COLLAPSE-LAUNDERS-FP as a minimized specimen — any spike claiming F-CORRECT=WINS must pass NC3, which means getting the RIGHT `parse` target, not just a `parse`-named target. NC5 (full-closure 20-cycle) directly tests the density-cliff failure mode in miniature.

The teeth-check is the right methodology discipline — the runner is falsifiable, not just a green-checker.

**Observer's outstanding work** (scaffold points 1/3/4 — the test-architect has explicitly routed this to the observer):

The observer must complete these before the scaffold is usable for fitness comparison:

1. **Point 1 — Fixed corpus persistence**: Write a `CORPUS-REGISTRY.json` to the measurement-scaffold directory that names the pinned commit (already in `CORPUS.txt`), the workspace Cargo.lock hash, and the working directory path. Any spike that runs against a different Cargo.lock hash gets flagged as off-corpus.

3. **Point 3 — Fixed timing harness**: Specify the timing method all spikes must use. Recommendation: `std::time::Instant` for in-process Rust timing; `hyperfine --warmup 3` for CLI timing; report MEDIAN of 5 runs (not single-run). Document this in `TIMING-SPEC.md` in the scaffold directory.

4. **Point 4 — 4-semiring 5-pair query fixture**: The 33,882-pair reachability relation is the closure result. The 5-pair query fixture is the concrete test: 5 specific (source, target) function pairs from the antigen codebase with known reachability under each of the 4 semirings (detection, conductance, provenance, blast-as-SET). This fixture tests that the SEMIRING SEMANTICS are correctly implemented, not just that the closure has the right cardinality.

---

### Naturalist: Pareto-Frontier Fitness Framing

**Notice to observer**: Fitness must be a VECTOR, not a scalar.

DSE (Design Space Exploration) framing: the spike-yard is multi-objective optimization. "Kill the unfit fast" = kill the DOMINATED (worse on EVERY axis), NOT the low-scorers. An expensive-but-uniquely-correct spike is a FRONTIER POINT, not unfit.

**Concrete consequence for the findings map**: If the scaffold stores one scalar fitness per spike, selection silently scalarizes and discards frontier points that are dominated-on-the-scalar-but-unique-on-an-axis.

The observer's per-stage variant × fitness-dimension tables ARE the right shape — they store vectors. But the fitness map needs an explicit PARETO DOMINANCE check column: for each variant, is it dominated (worse on every axis than another variant), or is it a frontier point (better on at least one axis than every alternative)?

**The DSE field save** (naturalist → scientist): hw/sw co-exploration literature (arXiv:1907.04650), multi-objective evolutionary config search for compile-time-vs-code-quality Pareto-optimality. The named research lineage for what the spike-yard is doing.

**Observer's assessment**: The Pareto framing is correct and improves the findings map. The observer will add a DOMINANCE column to the per-stage tables as findings land. "Kill the unfit" under Pareto dominance means: a variant is eliminated ONLY IF there exists another variant that is at least as good on all axes AND strictly better on at least one. The current tables have the right structure; they need a dominance check column added when the first real measurements arrive.

---

## Updated Implementation-Findings Map — After Wave 3

### Key structural updates:

**New locus: L7 WRITER-CARDINALITY** (from LE10): Single-serial-clock vs concurrent-multi-writer. This is not a pipeline stage but a ROOT CONSTRAINT that determines whether DBSP-collapse is available. For 0.7 target: one clock → DBSP available. For 0.8 universal-stroma: concurrent → DD required.

**Three-arm CH-B** (updated from two-arm):

| Arm | Condition | Result | Implication |
|-----|-----------|--------|-------------|
| (a) | deps-sparse (closure stays tractable) | ascent-batch decided | CH-A holds at scale; engine-core confirmed |
| (b) | deps-dense + DBSP-Δ sub-n² | DBSP dominates | Engine-core collapses to ONE algebra; LE4/LE6/LE9 dissolve |
| (c) | deps-dense + DBSP-Δ also superlinear | demand-driven forced | DM4 becomes load-bearing; magic-set or full DD required |

**STAGE 3 — ENGINE** update: L3 now has a fourth allele option beyond ascent/crepe/datafrog/Souffle:
- DBSP/Z-set (Feldera Rust library) — paradigm-level candidate; collapses three linkage edges IF recursive-Δ stays sub-n²; integration cost (F-INTEG) unmeasured

### Updated Running Verdict Table (After Wave 3)

| Claim | Verdict | Strength | Evidence |
|-------|---------|----------|---------|
| DM4 at workspace scale | FALSE (revision models layer) | HIGH | Direct measurement: pathmaker spike |
| DM4 at deps scale | CONDITIONAL (LE9+LE10) | — | Depends on arm (a/b/c) of CH-B |
| DBSP collapses LE4/LE6/LE9 | ANALYTIC PREDICTION | MEDIUM | Term-map complete; recursive-Δ cost unmeasured at deps density |
| Writer-cardinality (0.7) | ONE SERIAL CLOCK | HIGH (from STOCK) | Commit 19647b8; commutative reads DBSP-native; order-sensitive reads need total-order key |
| CH-B three-arm framing | CORRECT structural analysis | HIGH | Systems-thinking-expert LE10 + captain directive + naturalist convergence |
| Measurement scaffold NC1–NC5 | BUILT AND TEETH-CHECKED | HIGH | Test-architect built; wrong engine → NC2+NC5 RED confirmed |
| Scaffold points 1/3/4 | OUTSTANDING (observer owns) | — | Test-architect explicitly routed to observer |
| Fitness must be VECTOR | CORRECT (Pareto framing) | HIGH | DSE literature + naturalist notice; observer adopts for findings map |

---

## Observer Immediate Actions (Post-Wave 3)

**1. Complete scaffold points 1/3/4** (test-architect routed to observer — this is the observer's action item, not a question to route elsewhere):
- Write `CORPUS-REGISTRY.json` to scaffold directory
- Write `TIMING-SPEC.md` to scaffold directory  
- Build the 4-semiring 5-pair query fixture

**2. Update the per-stage fitness tables to add a PARETO DOMINANCE column** — when first measurements arrive, each variant needs: dominated (yes/no/partial) and frontier-point-on-axis (which axis).

**3. Monitor for CH-B result** — the captain's priority spike. When it lands, immediately update the engine-core section of the findings map with the arm (a/b/c) outcome.

---
*Wave 3 entries integrated. Completing scaffold points 1/3/4 now.*

---

## Camp Substrate Alignment Notes

- `the-spike-yard/the-evolution-frame`: OPEN (waiting on captain signature). The captain's frame note is deposited. No other islands exist yet.
- Observer has no pending role actions at T=0.
- The crew (pathmaker, expansionist, low-hanging-fruit, test-architect, scientist, scout, systems-thinking-expert, naturalist) has not yet started spiking. This notebook is being written in advance of GEN-1 to establish the measurement framework.

---

## Observer Questions (T=0)

For routing to crew via camp substrate when crew arrives:

1. **For systems-thinking-expert**: Map the combinatoric space before GEN-1 begins. Which stage-combination pairs are HIGH-risk for revision-model clash? Which are LOW-risk (independent)? This map determines which spikes should run first.

2. **For test-architect**: What is the falsification test for each fitness dimension? A spike claiming F-CORRECT without a negative control (a known-wrong edge not returned) is not measured. What is the born-red ATK for each stage?

3. **For scientist**: Three measurements are needed BEFORE GEN-1 completes: (a) SCIP warm vs cold latency, (b) salsa incremental cost on a real code edit (not 5-clock-ticks toy), (c) 4-semiring query latency at N>10K.

4. **For scout**: Has anyone tried the `ra_ap_*` library route for edge resolution vs the SCIP route? The integration cost comparison is the primary unknown for stage 1.

5. **For pathmaker**: When writing the first spike, the RESULT must be measured (not observed). "It compiled" is not a result. "Cold-query latency was 2.3s on the antigen codebase" is a result.

---

## Peer-Review Assessment — Pre-GEN-1

What would a skeptical reviewer say about the spike-yard's design before a single spike lands?

1. **The fitness function has no negative controls**: "Best for antigen" is measured by correctness + freshness + cost. But a spike claiming F-CORRECT=WINS without showing a case where the WRONG approach returns false-positives or misses edges is not tested. The test-architect's born-red ATK is the only instrument that can provide negative controls.

2. **The measuring instruments aren't calibrated**: The 4-semiring path-query was measured once (n=2948) on one hardware configuration. Before GEN-1 results can be compared across variants, we need a FIXED benchmark: same corpus, same machine, same query, same ground truth. Without this, comparing "ascent runs in 200ms" vs "datafrog runs in 150ms" is meaningless.

3. **The captain's recombination step is underspecified**: After GEN-1, the captain will select and recombine the fittest stage-choices. But fitness across stages may not be independent — a fast STAGE-1 input + a slow STAGE-3 engine may compose to something slower than a slower STAGE-1 + a faster STAGE-3. The systems-thinking-expert needs to map the cross-stage interaction effects before the captain recombines.

4. **The spike scope is underspecified**: The launch.md says spikes must be in scratch (scratchpad dir or throwaway). But the crew has not yet committed to a shared measurement harness. If each spike uses different timing methods, different corpus sizes, different antigen code versions — the fitness measurements are incomparable. A shared measurement scaffold is a pre-condition for valid GEN-1 selection.

---
*This notebook is a living document. Updated continuously as camp activity arrives.*

---

## Wave 4 — Convergence Burst (~23:53–23:59 UTC, 2026-06-28)

43 events. GEN-1 CLOSES. Captain rules CH-B = arm (a). Engine-core DECIDED. DM4 resolved by measurement. New findings: LHF cheap-base tie forces engine-vs-no-engine as the real L3 question (LE11). Expansionist DBSP spike measures 65-75x incremental cheapness but leaves batch-edit and memory costs open. Scientist's input-approach-benchmark SIGNED. Test-architect refines ATK-INPUT-001 to two distinct false-edge classes. Scout's prior-art consolidated across all 6 stages. STE and naturalist sleep with full waking notes.

---

### Captain: CH-B CLOSED — Engine-Core DECIDED

Island `gen-1-selection`, captain ruling (2026-06-28 23:58 UTC):

**CH-B result**: deps-included closure = **127,358 pairs / 0.97% dense / 0.02s** (first-party-rooted, 3619 nodes / 16,267 edges). Density ratio: 2.4x workspace, NOT n². NOT a re-architecture trigger.

**Captain's ruling on scope**: antigen lints first-party code and does NOT traverse into syn/salsa internals. The first-party-rooted graph IS the right graph. CH-B CLOSED.

**Supply-chain reachability** (dep-internal, full 5880-item-tree closure) = a **0.8+ forward capability**, not a 0.7 gate. Explicitly deferred.

**Engine-core decision: DECIDED.** relational-as-base + salsa-clock + ascent-query stays decided. DM4 does NOT flip on density grounds.

**Independent corroboration**: the deps graph reproduced the engineer-researcher's exact SCIP count (16,267 edges, n=3619) — independent confirmation the SCIP reconstruction is correct.

---

### Test-Architect: CH-B First Measurement (Island `gen-1-selection`)

The test-architect extracted the deps-included edge graph independently from `index.scip` using the stroma-experiments `.venv` (protobuf 6.33.6 — the version-mismatch sidestepped):

| Metric | Value |
|--------|-------|
| n (deps-included) | 3619 (2948 first-party + 671 dep callees) |
| edges | 16,267 |
| closure pairs | 127,358 |
| density | 0.972% |
| BFS time | 0.02s |
| max single-source reach | 785 |

**SCOPE CAVEAT** (the test-architect's explicit red): dep callees are leaves (zero out-degree) because SCIP gives first-party bodies only. dep→dep internal edges absent. This is FIRST-PARTY-ROOTED reachability.

**Captain ruled**: reading (1) is correct for antigen's design — CH-B is answered.

**Fixture persisted**: `scratchpad/stroma-experiments/callgraph_deps_included.json` available for any future engine spike.

**Observer's assessment**: The measurement is clean and the methodology sound. The scope caveat is real and the captain's ruling is correct. antigen is a tool that marks first-party code; tracing through dep internals would be supply-chain analysis — out of 0.7 scope. The fixture landing is a good substrate artifact.

---

### Pathmaker: DM4 Scale Result — Faithful Tiling Method

Island `spike/dm4-salsa-ascent-compose`, scale note (pathmaker):

**The key methodological finding**: The first two synthetic-graph attempts used incorrect growth models (reach-ratio 0.087–0.24 vs antigen's real 0.0039 = too dense, pessimistic n² explosion). The pathmaker correctly identified the ROOT CAUSE and switched to the FAITHFUL method.

**The faithful growth model** (from `topo.rs` characterizing the real callgraph):

antigen's reachability is sparse for a STRUCTURAL reason:
- forward-reach sets are tiny (median 2, mean 11.5, max 387, depth≤12)
- **hubs are SINKS** (max in-degree 134), not forward fan-out sources
- a sink-hub absorbs many callers but does NOT multiply forward-reach

This is a DOMAIN PROPERTY of antigen: a static-analysis tool over Rust. Rust call-graphs are structurally sink-hubbed (utilities/traits are called-by-many, call-few; forward-reach bounded by type-checked program depth ≤12).

**FAITHFUL SCALING RESULT** (tile the real callgraph k times):

| k | n | Closure pairs | Wall time |
|---|---|---------------|-----------|
| 1 | 2,948 | 33,882 | ~3ms |
| 8 | 23,584 | 270,656 | ~linear |
| 64 | 188,672 | 2,168,448 | 480ms |

Shared-util cross-cut variant (global hub every copy calls): 347ms — even cheaper (bounded forward reach absorbs it).

**Result: reach_pairs = EXACTLY k×33882 (LINEAR, verified). Batch-ascent SURVIVES realistic antigen growth.**

**Observer's assessment**: The pathmaker's methodology correction is significant. The pessimistic model WOULD have given superlinear results — the switch to faithful tiling was the right call. The STRUCTURAL reason (sink-hubbed-shallow = domain property of Rust call-graphs) is more important than the number: it makes the arm (a) result not luck but guaranteed by the topology class.

**Peer-review note**: The structural reason is a strong argument, but it applies to the CALL-GRAPH. The data-flow tier is a different topology class — a value can flow through many sites in ways that a call-edge cannot. LE9 resolves at the call-graph tier; it does NOT transfer to the data-flow tier (STE confirmed this — see below). The structural argument must be re-established for each tier.

---

### LHF Cheap-Base Spike — LE11 New Linkage Edge

Island `spike-stage2-cheap-base-zero-dep`, LHF result (2026-06-28 23:55–23:56 UTC):

**What was tested**: Can a 65-line zero-dependency std-Rust module match ascent on correctness AND beat it on integration cost?

**Result**: YES.

| Metric | Cheap-base (std-Rust) | ascent |
|--------|----------------------|--------|
| LOC | ~65 | "4 lines" (but 50 transitive crates) |
| Dependencies | 0 | 50 transitive crates |
| All-pairs closure | 33,882 (same as ascent) | 33,882 |
| Single-source backward | ~12µs | similar |
| All-2948 queries | ~0.93ms | similar |
| Base build time | 0.49s (clean) | heavier |
| F-CORRECT | TIES ascent | reference |
| F-INTEG | WINS (zero deps) | 50 transitive crates |
| F-CHEAP | WINS | heavier |

**The idempotency boundary** (independent confirmation):
Three idempotent/bounded semirings (boolean detection, tropical-min conductance, node-count blast) work with zero deps, sub-ms queries. The counting/path-blast semiring **DIVERGES** on cyclic graphs — the spike literally hung. This is independent corroboration of engineer-researcher's finding (242.3s on ascent, stroma-semiring-unification island, Wave 1). Two unrelated engines, same wall. The boundary is intrinsic to the problem (cycles), not an ascent artifact.

LHF's provenance correction (substrate-over-memory applied): credit belongs to engineer-researcher for the original finding; LHF's finding = independent corroboration.

**THE REFRAME**: L3 question is now **engine-vs-no-engine**, not *which* datalog engine.

The "4 lines vs 80 lines" framing obscured the real cost: ascent's "4 lines" carries 50 transitive crates. The cheap base matches correctness and beats integration cost at workspace scale. The engine earns its keep ONLY where incrementality-across-edits is the bottleneck — and the charter already routes THAT to salsa (which r-a brings for free).

**LE11 — new linkage edge** (systems-thinking-expert formalization):
`L6-QUERY-SHAPE ==> L3-ENGINE-NECESSITY`
- If dominant consumer query = single-source backward reach ("I changed fn X — who must re-review?") → L3 needs NO batch engine; cheap base serves it.
- If dominant consumer query = all-pairs closure ("what is the full reachability matrix?") → engine earns its keep.

The "am I OK?" review lens IS single-source-backward. LE11 means the CONSUMPTION stage (L6) retroactively decides whether the engine is needed in L3.

**Observer's assessment**: This is a genuine LHF finding — seen by nobody else because the "4-vs-80 lines" framing had locked the question as *which* datalog rather than *whether* datalog. The LE11 edge is correct: if the dominant query is single-source, the materialization the engine provides is waste.

**Peer-review concern**: The cheap-base ties ascent on all-pairs at workspace scale (n=2948). The observation does NOT hold by assumption at deps-included scale. At n=3619+ with 127,358 closure pairs, the cheap-base's all-pairs BFS remains sub-100ms (the test-architect measured it at 0.02s). So the cheap-base WINS on F-INTEG at ALL measured scales. The engine's advantage is incrementality on edits — which is real if the dominant pattern is many-edits, rare on the cheap-base's O(n+e) per query.

**Updated F-INTEG implication for Stage 2/3 tables**: The cheap-base is now the BASELINE the engine must beat. The "relational wins on 4-vs-80 lines" finding from the crucible is technically correct but the framing was wrong: the correct comparison is "zero-dep std-Rust (65 lines, 0 crates) vs ascent (4 rule-lines, 50 crates)."

---

### Expansionist: DBSP/DD Spike — Measured on Real Graph

Island `expansionist-cross-domain-stage-survey`, DBSP spike result:

**What was built**: differential-dataflow 0.24 + timely 0.30 recursive reachability circuit on antigen's real callgraph (n=2948, 4714 edges). Reach = same least-fixpoint via DD's `iterate()`; `distinct()` = boolean/idempotent semiring (sidesteps SEED-E5's non-idempotent blowup).

**Correctness**: reach-pairs settle at exactly 33,882. Matches ascent (correct relation, including the 3-SCC cyclic self-pairs).

**Incremental measurements** (3 runs, stable):

| Operation | Cost | Ratio vs full |
|-----------|------|---------------|
| Round 0 full closure | ~15ms | 1.0× |
| Round 1 incremental +1 edge | ~0.21ms | 65-75× cheaper |
| Round 2 incremental -1 edge | ~0.19ms | 74-85× cheaper |

**Key finding vs DM4**: ascent's incremental cost = full re-run = 1.0× (no incremental story; salsa's per-file delta IS thrown away at the ascent boundary — the pathmaker confirmed this in the DM4 spike). DD's incremental cost = ~1.5% of a full re-run. AND DD handles edge-REMOVAL incrementally via Z-set negative weights — ascent cannot do this without a full re-run.

**The one real constraint** (expansionist's honest red):
- DD's win on +1/-1 single edit is best-case.
- BATCH edit (many edges at once): does the win hold or degrade toward full-recompute? UNMEASURED.
- MEMORY cost: DD maintains arrangements/traces vs ascent's stateless re-run. At big workspace, this trade must be measured. UNMEASURED.
- DBSP-vs-full-DD: multi-writer = the STOCK seam (LE10/writer-cardinality, already answered for 0.7).

**Observer's assessment**: The DBSP/DD spike is the most novel measurement in Wave 4 from a scientific standpoint. The 65-75× incremental cheapness is real and measured. The key open questions are battery: (1) batch-edit Δ-cost, (2) memory footprint under maintained arrangements. These are not fatal unknowns — they are the next natural measurements.

**Peer-review note**: The expansionist built the spike; the scientist was named as the party to independently benchmark. That independent measurement has NOT yet landed. The 65-75× number is the expansionist's own measurement on their own spike. Before treating this as evidence-grade, the scientist should re-run on the fixture corpus and confirm.

**Implication for the L3 allele table**: DBSP/DD is now a MEASURED allele, not just a theoretical candidate. Its profile at workspace scale:
- F-CORRECT: VERIFIED (33,882 exact match)
- F-INCR (single-edit): 65-75× cheaper than full re-run (vs ascent's 1×)
- F-INCR (batch-edit): UNMEASURED
- F-INTEG: heavier than ascent (timely+DD runtime, maintained arrangements)
- F-CHEAP (integration cost): higher than cheap-base, comparable to ascent at deps-included scale

---

### STE Synthesis on `gen-1-selection`

The systems-thinking-expert's synthesis note folded together the pathmaker's DM4 scale result + the LHF cheap-base spike:

**CH-B = ARM (a) CONFIRMED** for the call-graph detection semiring. Structural reason: sink-hubbed-shallow topology is a domain property of Rust call-graphs (static analysis over Rust), not luck.

**Two GEN-2 pressure-points the arm (a) result OPENED:**

1. **LE9 RE-OPENS AT DATA-FLOW TIER**: The sink-hubbed-shallow property is proven for the CALL-graph. It does NOT transfer to the DATA-FLOW graph. Data-flow reach can be denser (a value flows through many sites). If antigen's base graph becomes data-flow (MIR-tier, SEED-E4 prize), the density question must be RE-MEASURED there. The call-graph win does NOT carry.

2. **Engine-vs-no-engine (LHF + LE11)**: The 65-line zero-dep module TIES ascent on correctness and BEATS it on integration. The L3 question is now "engine vs no-engine," not "which engine." The common consumer query is single-source backward reach (~12µs) — the engine's whole-relation materialization is only needed if L6 asks all-pairs questions. LE11: L6-query-shape → L3-engine-necessity.

**STE recommendation (terrain-not-verdict)**: Treat the 65-line cheap-base as the BASELINE the engine must EARN past. GEN-2's two highest-value spikes:
(a) density at DATA-FLOW tier (does LE9 re-fire on MIR-flow?)
(b) L6-query-shape census (single-source vs all-pairs — does any real consumer need the batch engine?)

Blast/counting semiring still needs SCC-condensation (LE5, re-confirmed by BOTH spikes — the one semiring that is NOT free).

**Observer's assessment**: The STE synthesis is correct and concise. The two GEN-2 pressure-points are the right formulation: arm (a) closes a question but opens two more. The naturalist's three-lenses-one-scalar observation holds: the GEN-2 spike-order by leverage is (a) data-flow density then (b) L6-query-shape census. The STE is lens-not-verdict, which is exactly right.

---

### Test-Architect: ATK-INPUT-001 Refined — Two False-Edge Classes

Island `input-approach-benchmark`, test-architect note (sharpening Wave 2 finding):

**Two distinct false-edge classes** — the ATK-NAME-COLLAPSE is broader than Wave 2 recorded:

**CLASS-1** (measured = 4%, caught by NC-2): Heuristic emits edge to a name that is NOT a fn at all — antigen's macro attrs (`defended_by`, `presents`, `itch`, `mucosal`) misread as calls because `(...)` syntax fires. CAUGHT by name-level diff because the callee-name is absent from SCIP's fn-def set.

**CLASS-2** (ATK-INPUT-001, UNMEASURED): Heuristic emits edge to a name that IS a fn but the WRONG one — `foo→parse@M2` when the resolved truth is `foo→parse@M1`. INVISIBLE to name-level diff: both sets contain `(foo, 'parse')`, so it scores CONFIRMED. With `parse`=121 symbols, `validate`=80, 91% of fn-symbols name-ambiguous, the regex is structurally prone to this.

**Consequence**: The 4% false-rate is CLASS-1-only. CLASS-2 is laundered into the "96% confirmed / 2243-in-both" number. The "silence-not-noise" conclusion (ADR-067 §7) is about MISSING edges; CLASS-2 wrong-target edges are NOISE (confidently-wrong downstream), not silence.

**The tooling note**: The deps-included graph independently reproduced the engineer-researcher's 16,267-edge SCIP count — confirms the SCIP reconstruction is correct.

**To close CLASS-2**: Re-run the diff at SYMBOL level (`caller_symbol→callee_symbol`, no name-collapse) and report CLASS-2 count separately. The parser works (425ms parse), the `.venv` has matching protobuf 6.33.6.

**Observer's assessment**: The two-class refinement is the correct analysis. CLASS-1 (wrong-name) and CLASS-2 (wrong-target-same-name) are qualitatively different failure modes. CLASS-1 degrades gracefully (missing edges are silence). CLASS-2 generates noise (wrong-target edges are false positives that pollute downstream). The "silence not noise" claim from ADR-067 §7 is CLASS-1-specific — it needs to be re-evaluated once CLASS-2 is measured.

**Updated STAGE 1 — INPUT, syntactic allele**:
- F-CORRECT: CLASS-1 FP = 4% (MEASURED, name-collapsed). CLASS-2 FP = UNKNOWN (unmeasured, laundered into "confirmed" count). Total FP floor is 4%; total ceiling is unknown.
- The syntactic tier may be actively harmful (noise-on-top-of-miss) if CLASS-2 rate is material.

---

### Scientist: `input-approach-benchmark` SIGNED

Island `input-approach-benchmark` SIGNED COMPLETE (scientist, state=Complete, fingerprint=manual).

The full scientist note from Wave 3/4 now has formal status. Key numbers now official:
- SCIP: 4714 first-party edges, 0% miss (gold standard), ~5s generation, 425ms parse
- Heuristic: 2332 edges, 52% miss, 4% FP (CLASS-1-only), ~4.8s, zero integration cost
- Salsa incrementality: demonstrated on 5-file toy (backdate gate confirmed, REQUERY=zero recompute)
- Semiring spike: running (detection/conductance/blast as one parameterized rule)

**Observer's assessment**: The SCIP verdict (clear winner for antigen's INPUT stage) is now formally attested. Three tiers confirmed: syn < SCIP < MIR-exact. The fitness table is officially comparable to other stage measurements.

---

### Scout: Prior-Art Map — All 6 Stages

Island `scout-prior-art-map`, signed by captain (required signer). Full cross-stage prior-art consolidated:

**INPUT**: 3 resolution levels (syn/zero, SCIP/symbol-occurrence, ra_ap_hir/full). SCIP TRAP: symbol-occurrence ≠ resolved dispatch. dyn Trait points at trait method decl, not impl.

**STRUCTURE**: Three schema models. CodeQL relational wins on incrementality (file-level tuple invalidation = finest grain). Joern CPG = zero incrementality (rebuild per file). Glean = coarse batch-retire. WINNER: CodeQL relational.

**ENGINE**: salsa+ascent CLEAN COMPOSE. salsa+DD HOSTILE (two clock models, zero prior art). datafrog FROZEN (2019). crepe THIN. RECOMMENDED: salsa + ascent.

**OUTPUT**: git-notes OID-trap CONFIRMED (note orphaned on ANY file edit). LSP injection into r-a IMPOSSIBLE — separate VS Code extension required.

**CONSUMPTION**: CodexGraph (NAACL 2025): property-graph + Cypher = 22.96% SWE-bench vs BM25 3.11%. BUT collapses on weak LLM backbone (5.0% with Qwen2 vs BM25 15.5%). Hybrid consensus: structured for topology + embeddings for semantic fuzzy lookup.

**Cross-stage recurrence** (naturalist notice on the same island): COARSE-CONVENIENT LOSES TO FINE-GRAINED-STRUCTURAL at every stage independently. Five stages, zero coordination, one verdict — the same shape that decided the design recurring across implementation choices.

**Observer's assessment**: The prior-art map is an important substrate artifact. The coarse-vs-fine recurrence is the strongest single pattern across the spike-yard. However: it's a convergent AGREEMENT (5 independent stages finding the same answer), which per the crucible's Orzack-Sober discipline is WEAKER evidence than convergent DISAGREEMENT. The naturalist's "watch for the stage where coarse WINS" is the right follow-up — if one stage inverts, THAT's the load-bearing finding.

**Candidate for inversion**: FRESHNESS stage. The danger-model is deliberately coarse (meaning-change-not-byte-change). The question is whether COARSE freshness (salsa-level backdate) outperforms FINE freshness (line-level diff). This is genuinely undetermined and deserves a spike.

---

## Updated Implementation-Findings Map — After Wave 4

### CH-B LANDED: ARM (a) — ENGINE-CORE DECIDED

The captain's CH-B ruling closes the biggest open question in GEN-1.

**Updated Stage 2 / Stage 3 status**:

| Claim | Status | Measurement |
|-------|--------|-------------|
| Relational-as-base at workspace scale | CONFIRMED | n=2948, closure=33,882 at 3ms |
| Relational-as-base at deps scale | CONFIRMED (first-party-rooted) | n=3619, closure=127,358 at 0.02s, 0.97% dense |
| salsa+ascent (batch) at workspace scale | CONFIRMED | DM4 spike: 3.08ms, linear, zero bridge |
| salsa+ascent at deps scale (64×) | CONFIRMED (faithful tiling) | 480ms at k=64 (188k nodes), linear scaling |
| Engine-core DECIDED | YES (arm a) | Captain ruling on gen-1-selection |

### Updated STAGE 1 — INPUT Fitness Table

| Variant | F-CORRECT | F-INCR | F-INTEG | F-CHEAP | Feasibility | Pareto | Notes |
|---------|-----------|--------|---------|---------|-------------|--------|-------|
| syn (syntactic heuristic) | CLASS-1: 52% miss + 4% FP. CLASS-2: UNKNOWN (unmeasured noise floor) | N/A | WINS (zero) | WINS | FAILS LE1/LE2 if used with semantic consumers | DOMINATED on correctness by SCIP | CLASS-2 may make this actively harmful |
| SCIP batch | WINS (gold, 4714 edges, 0% miss confirmed) | Cold 5s; warm 425ms parse | MEASURED (stable protobuf schema) | GOOD | FEASIBLE | FRONTIER (correctness + cost) | Input-benchmark SIGNED |
| ra_ap_hir library | WINS (same as SCIP + monomorphized dispatch) | Per-query (no cold batch cost) | POOR (weekly API churn, no semver) | POOR (API instability) | FEASIBLE | FRONTIER (dispatch depth) | Not spiked; F-INTEG the concern |
| SCIP + ra_ap_hir hybrid | WINS (both layers) | UNKNOWN | HIGH (two integration costs) | POOR | FEASIBLE (LE3: dual-resolver for parity oracle) | FRONTIER (LE3 correctness) | Required ONLY for self-distrust/parity case (CH-C) |

### Updated STAGE 2 — STRUCTURE/BASE Fitness Table

| Variant | F-CORRECT | F-SEMIRING | F-INCR | F-INTEG | F-CHEAP | Feasibility | Pareto |
|---------|-----------|------------|--------|---------|---------|-------------|--------|
| Relational-datalog (CodeQL-style) | WINS | WINS (semiring as one parameterized query) | WINS (file-level tuple invalidation) | GOOD | GOOD | FEASIBLE | FRONTIER |
| Property-graph | UNKNOWN | ESTIMATED lower (joins need translation) | POOR (no fine-grained invalidation) | POOR | UNKNOWN | VIOLATES LE4 (fight with salsa) | INFEASIBLE |
| Hybrid (relational + graph views) | UNKNOWN | UNKNOWN | UNKNOWN | HIGH | UNKNOWN | FEASIBLE if base is relational | UNMEASURED |

### Updated STAGE 3 — ENGINE/COMPOSITION Fitness Table

The L3 question has been reframed by the LHF spike. It is now **engine-vs-no-engine**, not *which* engine.

| Variant | F-CORRECT | F-INCR (single-edit) | F-INCR (batch-edit) | F-INTEG | F-CHEAP | Feasibility | Pareto | Notes |
|---------|-----------|---------------------|---------------------|---------|---------|-------------|--------|-------|
| No-engine (65-line std-Rust) | WINS (33,882 exact, idempotent semirings) | N/A (no maintained state) | N/A (recompute = O(n+e)) | WINS (0 deps) | WINS (0.49s build, 0 crates) | FEASIBLE | FRONTIER (F-INTEG + F-CHEAP) | Counting-semiring DIVERGES on cycles (LE5); needs SCC-condensation separately |
| salsa + ascent | WINS (33,882) | BAD (1× = full re-run; salsa delta thrown away) | BAD (same) | GOOD (pure Rust) | GOOD | FEASIBLE | FRONTIER (F-CORRECT at scale, linear) | Batch=480ms at 64× scale, linear |
| salsa + DD (Feldera/DBSP) | WINS (33,882, correctness MEASURED) | WINS (65-75× cheaper on single-edit; handles removal) | UNKNOWN | POOR (maintained arrangements) | POOR (heavier runtime) | FEASIBLE (serial clock for 0.7) | FRONTIER (F-INCR single-edit) | Batch-edit Δ and memory UNMEASURED |
| salsa + differential-dataflow (full) | WINS (inferred from DD = superset of DBSP) | WINS | WINS | POOR (clock mismatch; zero prior art) | POOR | HOSTILE (LE4: clock models fight) | INFEASIBLE for composition with salsa | Required only for multi-writer 0.8+ case |
| ascent alone (no salsa) | WINS | POOR (no salsa backdate) | N/A | GOOD | GOOD | FEASIBLE | DOMINATED by salsa+ascent (worse on F-INCR) | Loses the backdate gating (89× on non-structural edits) |
| datafrog | INFERRED CORRECT | POOR | POOR | GOOD | GOOD | FEASIBLE | INFEASIBLE for new dev (FROZEN 2019) | Polonius-heritage; battle-tested but dead |

**LE11 gating for engine decision**: If dominant L6 query is single-source backward (most likely for the "am I OK?" review lens), the no-engine cheap-base serves L6 AND avoids F-INTEG cost. Engine earns its keep only if: (a) all-pairs queries needed at L6, OR (b) incremental-on-many-edits bottleneck materializes in real workload.

### Updated STAGE 5 — OUTPUT (no new measurements, but structural update)

LE7 and LE8 remain OPEN. The LHF finding (cheap-base with zero deps) does not directly address L5, but it establishes a pattern: the "convenient" substrate should earn its integration cost vs the cheap alternative. The git-notes OID-trap (scout confirmed) disqualifies git-notes for sub-commit-granularity use. Injected-in-source + LSP-overlay remain the viable alleles; the EXP-2 three-way experiment not yet run.

### Updated STAGE 6 — CONSUMPTION (LE11 update)

LE11 now makes L6-query-shape a root constraint: the dominant consumer query decides whether the batch engine is needed at L3. The scout's prior-art (CodexGraph) and the LHF finding (single-source at 12µs) together suggest:

- **Single-source backward ("who must re-review?")**: cheap-base serves; no engine needed
- **All-pairs ("full reachability matrix")**: engine earns its keep
- **LLM grounding at generation**: likely single-source (the model is asking about one function at a time)

The census of actual L6 query patterns (the GEN-2 spike STE recommends) is the deciding measurement.

---

## Updated Running Verdict Table — After Wave 4 (GEN-1 CLOSED)

| Claim | Verdict | Strength | Evidence |
|-------|---------|----------|---------|
| CH-B = arm (a) (deps-sparse) | DECIDED | HIGH | Captain ruling + test-architect measurement (127,358 pairs / 0.97% dense) + pathmaker faithful tiling (480ms at k=64) |
| Engine-core DECIDED (relational + salsa + batch-ascent) | DECIDED for 0.7 | HIGH | CH-B ruling; DM4 spike; faithful tiling; captain gen-1-selection note |
| DM4 at deps scale | FALSE (not load-bearing for detection semiring) | HIGH | Faithful tiling = linear; structural reason (sink-hubbed-shallow = domain property) |
| Structural reason for arm (a) | DOMAIN PROPERTY (Rust call-graphs are sink-hubbed) | HIGH | topo.rs characterization; pathmaker faithful-tiling measurement confirms |
| LE9 resolves for call-graph | RESOLVED — arm (a) | HIGH | Measured. Does NOT transfer to data-flow tier — LE9 re-opens there |
| LE11 (L6-query-shape → L3-engine-necessity) | NEW LINKAGE EDGE | HIGH | LHF spike + STE formalization; dominance depends on actual L6 query census |
| Engine-vs-no-engine as L3 question | REFRAMED | HIGH | LHF 65-line std-Rust TIES ascent on correctness, WINS on F-INTEG |
| DBSP/DD incremental (single-edit) | 65-75× cheaper than full re-run | MEDIUM | Expansionist spike (self-measured; scientist independent benchmark pending) |
| DBSP/DD batch-edit Δ cost | UNMEASURED | — | Required for full DBSP allele fitness profile |
| DBSP/DD memory cost | UNMEASURED | — | Maintained arrangements vs stateless re-run |
| CLASS-1 FP (syntactic heuristic) | MEASURED = 4% | HIGH | Scientist benchmark + NC-2 |
| CLASS-2 FP (syntactic heuristic) | UNMEASURED (laundered into "confirmed" count) | HIGH concern | ATK-INPUT-001; symbol-level re-diff needed |
| SCIP batch = INPUT winner | DECIDED (signed) | HIGH | input-approach-benchmark SIGNED by scientist |
| Input-approach-benchmark SIGNED | COMPLETE | HIGH | scientist, state=Complete |
| Counting-semiring diverges on cycles (LE5) | DOUBLY CONFIRMED | HIGH | Engineer-researcher (ascent, 242.3s) + LHF (std-Rust, hung) — two independent engines |
| Coarse-vs-fine pattern (all 5 stages) | CONVERGENT AGREEMENT | MEDIUM (convergent agreement weaker than disagreement) | Scout prior-art map; naturalist synthesis. Watch for inversion at FRESHNESS stage. |
| FRESHNESS stage: coarse may WIN | HYPOTHESIS (not measured) | MEDIUM concern | The danger-model is deliberately coarse — potential inversion of the 5-stage pattern |
| STE sleep note (waking-up record) | CAPTURED | — | Genome map re-open conditions, GEN-2 spike-order |
| Naturalist sleep note | CAPTURED | — | 6 held patterns + garden entry filed |

---

## Observer Questions — GEN-2 Priority Queue

GEN-1 is CLOSED. These are the open questions the captain needs for GEN-2 selection:

1. **CLASS-2 FP rate** (scientist): Run symbol-level heuristic-vs-SCIP re-diff. Protobuf 6.33.6 confirmed working in the `.venv`. Report CLASS-2 count separately. This resolves whether the "silence not noise" claim holds for the syntactic tier.

2. **Data-flow tier density** (scientist/pathmaker): LE9 re-opens at MIR data-flow tier. If antigen expands to SEED-E4 (value-flow) scope, does the sink-hubbed-shallow property hold, or does data-flow graph densify past the density cliff? This is the GEN-2 highest-value measurement.

3. **L6-query-shape census** (pathmaker/scout): What is the actual distribution of consumer query patterns? Is single-source backward ("who must re-review X?") the dominant pattern, or do consumers need all-pairs materialization? This resolves LE11 and decides whether the engine is needed at all.

4. **DBSP/DD memory footprint** (expansionist + scientist): The 65-75× incremental win is real. The memory cost of maintained arrangements at 127,358 closure pairs is unmeasured. This is the remaining fitness dimension for the DBSP allele.

5. **DBSP/DD batch-edit Δ cost** (expansionist + scientist): Does the incremental win degrade on batch edits (many edges at once)? This matters for real workloads where a refactor touches many call sites simultaneously.

6. **EXP-2: Output-substrate three-way** (pathmaker): Still not run. CH-B resolution and engine-core decision clear the path to spike the output substrate. injected-in-source vs LSP-overlay (git-notes OID-trap disqualifies git-notes for sub-commit granularity per scout).

7. **FRESHNESS stage inversion check** (observer-flagged): Is there any scenario where the coarse-freshness allele (salsa backdate = meaning-change detector) WINS over a finer-grained approach? The 5-stage coarse-loses pattern makes this the most interesting disanalogy to hunt.

---

## Camp Substrate Alignment — After Wave 4

**Signed/complete islands**:
- `input-approach-benchmark` — SIGNED (scientist, Complete)
- `scout-prior-art-map` — captain required-signer (fulfilled by note activity)
- `sys/genome-space-linkage-map` — SIGNED (STE); updated with LE9/LE10/LE11 notes
- `sys/spike-order-leverage-terrain` — SIGNED (STE)
- `sys/coherent-chromosomes-catalog` — SIGNED (STE)

**Sleeping roles** (with waking notes):
- naturalist — sleep at 23:55 UTC (waking note captured above)
- STE — sleep at 23:55 UTC (waking note: re-check LE9/data-flow tier, engine-vs-no-engine)
- scout — sleep at 23:56 UTC (waking note: GEN-1 prior-art recon complete)

**Active islands**:
- `gen-1-selection` — captain CLOSED GEN-1 engine-core decision; open for GEN-2 directives
- `spike-stage2-cheap-base-zero-dep` — LHF result landed (no formal signature event captured)
- `expansionist-cross-domain-stage-survey` — DBSP spike result landed
- `spike/atk-name-collapse-launders-fp` — CLASS-2 open; symbol-level re-diff pending

**Substrate drift alert**: The CORPUS-REGISTRY.json `deps_included_stats` slot now has measured values — must be updated.

---

*Wave 4 integrated. GEN-1 engine-core CLOSED: arm (a), relational+salsa+ascent DECIDED for detection semiring at 0.7 scope. Two GEN-2 pressure-points open: data-flow tier density (LE9 re-opens) + L6-query-shape census (LE11). Scaffold complete. Findings map updated.*

---

## Wave 5 — GEN-2 Seed Burst (~00:00–00:05 UTC, 2026-06-29)

7 events on `gen-2-seed` + `semiring-spike-results` SIGNED. GEN-2 opens with: captain selection note, LHF census of shipped antigen consumer traversals (call/lineage tier = single-source-local, no engine needed), STE faithful-method discipline for data-flow-tier spike, expansionist cross-domain taint-flow analogy, semiring-spike-results (blast-counting = 66,000x slower = offline/batch tier, not a live engine driver).

---

### Captain GEN-2 Seed Note

Island `gen-2-seed` (open, waiting on captain signature), captain selection note at 00:00 UTC:

**GEN-1 result summary** (captain framing):
- Engine-core DECIDED: sink-hubbed-shallow is a DOMAIN PROPERTY of static-analysis-over-Rust (not luck). Batch-ascent scales linearly.
- Fitter variant found: 65-line zero-dep std-Rust TIES ascent on correctness AND BEATS it on integration cost (ascent's "4 lines" hides 50 transitive crates).
- Common consumer query = single-source backward ("who-must-I-re-review", ~12µs) — NOT all-pairs batch.

**GEN-2 recombined candidate**: cheap-base (zero-dep) + salsa-incrementality (gate) + datalog-engine ONLY-IF a census finds a real all-pairs consumer. Blast/counting semiring needs SCC-condensation (LE5 — the one semiring not free).

**GEN-2 priority spikes**:
1. DATA-FLOW-TIER DENSITY — re-opens LE9. Sink-hubbed-shallow proven for call-graph ONLY; does NOT transfer to data-flow/MIR. Re-measure at data-flow tier.
2. CONSUMER-QUERY-SHAPE CENSUS — enumerate queries antigen consumers actually need + their shapes. If none need all-pairs-batch, the cheap base + salsa serves everything; datalog engine needed for NOTHING.

---

### LHF Census: Shipped Antigen Consumer Traversals

Two notes on `gen-2-seed` from LHF (00:02 UTC):

**The finding**: LHF deck-walked `antigen/src` and found exactly TWO shipped graph traversals:

| Location | Function | Shape | Engine? |
|----------|----------|-------|---------|
| `antigen/src/scan/finalize.rs:259` | `transitive_ancestors_dfs(adjacency, child_key)` | single-source backward DFS, ~15 lines, std HashSet + Vec stack | NONE |
| `antigen/src/audit/recurrent.rs:158` | `ancestors_of(antigen, parent_of)` | single-source backward stack-BFS, ~12 lines | NONE |

Both are IDENTICAL in shape to the LHF cheap-base's `who_depends_on` (single-source backward, ~12µs at workspace scale). Neither builds a closure. Neither needs a datalog engine.

**CENSUS VERDICT (call/lineage tier)**: Every shipped consumer is single-source-local. The datalog engine earns its keep for NOTHING antigen currently consumes at this tier.

**The captain's recombined candidate confirmed**: "cheap-base + salsa + datalog-ONLY-IF" — the IF now has no current 0.7 consumer on the call/lineage tier.

**Remaining 3 engine-justification shapes** (from second LHF note):
1. **LLM-grounding precompute**: NO 0.7 consumer. Forward 0.8+ capability (same class as supply-chain-reachability, already deferred). Cannot justify the engine now.
2. **Data-flow/MIR tier**: LIVE for 0.7 (SEED-E4 prize). Genuinely unmeasured. Query-shape unknown. This is the ONE real GEN-2 spike that could flip datalog-vs-no-engine.
3. **Field/percolation lattice**: Genuinely all-graph; ascent/DD already indicated. But runs against FROZEN snapshot (ADR-067 clause 5) — may be batch-cheap on a frozen snapshot.

**Net**: engine-vs-no-engine for 0.7 reduces to ONE live spike — data-flow-tier density + query-shape — plus the field-lattice (may also be batch-cheap). Everything else antigen consumes at 0.7 is single-source-local = the zero-dep cheap base wins.

**Observer's assessment of the LHF census**: This is the highest-confidence finding in Wave 5. The evidence is shipped production code in antigen's own src — not a synthetic benchmark, not a theoretical argument. Two independent single-source traversals in production, both matching the LHF cheap-base's shape exactly. The "recall not re-run" principle at its best: antigen already answered the consumer-query-shape question in its own code. The census is real on what it covers; the boundary (data-flow tier, LLM grounding, field-lattice) is correctly named.

**Peer-review note**: The census covers `antigen/src/` only. It's possible that planned GEN-2 consumers (yet to be built) would need all-pairs materialization. But the captain's recombined candidate is correctly scoped: "datalog-ONLY-IF" means the engine is a DEFERRED dependency contingent on finding a real consumer, not a pre-emptive inclusion. That's the right posture for a zero-dep base that already works.

---

### STE: Faithful-Method Discipline for Data-Flow Density Spike

STE note on `gen-2-seed` (00:02 UTC) — pre-registering the faithful-method discipline before the spike runs:

**Why data-flow reach CAN densify where call reach did NOT** (structural reason):
- Call-graph stayed sparse: hubs are SINKS (called-by-many, call-few); forward-reach bounded by call-depth (~12).
- Data-flow BREAKS both bounds: def-use chain length can exceed call-depth; data-flow HUBS are FORWARD-FAN-OUT sources (a config/context/Arc value is passed forward to many sites, each reaching everything those sites reach). This is exactly the topology the test-architect's degenerate battery showed goes n².

**Three faithful-method requirements**:

| Requirement | Call-graph did | Data-flow spike must do |
|-------------|---------------|------------------------|
| Measure real data | Tile real callgraph | Measure real MIR data-flow edges (NOT synthetic); if too heavy, use a representative subset labeled as such |
| Report distribution, not just median | Median forward-reach = 2 | Report tail distribution + identify fan-out hubs; median alone can lie if distribution is heavy-tailed |
| Scope inter-proc separately | workspace vs deps (scope) | Intra-proc vs inter-proc (depth); inter-proc is where fan-out lives |

**Three outcome arms for data-flow spike**:
- (a) stays sparse → call-graph result extends to data-flow; cheap-base + salsa serves both
- (b) densifies via FEW fan-out hubs → named structure; hub-aware pre-pass (same SCC-condensation move as LE5); engine tractable WITH a hub-aware pre-pass
- (c) densifies broadly → all-pairs infeasible; demand-driven (single-source-from-changed-value) FORCED — but LE11 shows that's what antigen's real consumer needs anyway

**LE9' (data-flow form)**: LE9 was `[L1-scope → L3-architecture via density]`. GEN-2 form is `LE9' [L1-DEPTH (call vs data-flow) → L3-architecture via FAN-OUT-density]`. The variable is now INPUT DEPTH not scope.

**Coupling of spike-1 (density) and spike-2 (census)**: density only matters if a consumer needs all-pairs over it. If the data-flow consumer is also single-source (most likely given the LHF census), then outcome (c) dense-but-demand-driven is NOT a problem — demand-driven single-source is already the query shape.

**Observer's assessment**: The STE's pre-registration of the faithful-method discipline is the right methodology move — it prevents the same near-miss that almost invalidated the call-graph density spike (first attempts too dense). The three-outcome framing is the right structure for the captain's reading. The LE9' sharpening is correct.

---

### Expansionist: Cross-Domain Taint-Flow Analogies

Note on `gen-2-seed` (00:03 UTC):

**CPG multi-layer-edge model** (Joern, Yamaguchi IEEE S&P 2014): one node-set, multi-layer edges (call | control | data-dependence/PDG). antigen's 3-tier ladder IS already multi-layer-edge — E4 only adds a data-dependence edge-relation from MIR. Same reachability engine serves call-blast AND taint, just against different edge-relations.

**Cross-domain solved problem**: Epidemiology and power-grids both face "flow graph densifies, naive reachability blows up" and both solve it the same way: they DON'T compute all-pairs transitive closure on the dense flow graph.
- Epidemiology: SOURCE-BOUNDED forward simulation from specific seeds (contact-tracing = single-source, bounded-depth) — matches LHF census exactly.
- Power-grids: CONTINGENCY-SCREENING (test the N-1 set that matters, not all cascades) — matches the "defense-relevant subset" framing.

**Borrow**: if the data-flow tier densifies, the answer is NOT "datalog all-pairs is forced" — it's "taint is single-source-from-a-sink/source-of-interest, bounded." Density may make all-pairs infeasible AND irrelevant simultaneously. The spike should measure SINGLE-SOURCE taint cost on the dense data-flow graph, not all-pairs closure.

**Observer's assessment**: The cross-domain convergence strengthens the LHF census. Three independent domains (antigen's own shipped code, epidemiology, power-grids) all arrive at single-source traversal as the real query shape for taint/reachability under density. This convergence upgrades the LHF finding from "shipped code evidence" to "domain-structural argument + shipped code evidence."

---

### Scientist: `semiring-spike-results` SIGNED

Island `semiring-spike-results`, scientist note (00:02 UTC), SIGNED:

**Corpus**: 2948 fns, 4714 first-party edges. Tool: ascent datalog, 3 semirings, release build.

| Semiring | Pairs | Time | Verdict |
|----------|-------|------|---------|
| Detection (boolean) | 33,882 | 2.6ms | LIVE — viable |
| Conductance (tropical min,+) | 33,882 | 4.4ms | LIVE — viable |
| Blast (counting, capped 1M) | 33,882 | **291,862ms (~4.9 min)** | DEAD END — offline/batch only |

**66,000× slower than detection on the same graph.** The cap at 1M doesn't help — path-counting on a real-world call graph with cycles explodes.

**Key findings**:

1. **SAME RULE SHAPE**: All three semirings are the same transitive closure relation, same 33,882 pairs. Only the per-pair value differs (bool / min-hops / count). The "Layer A vs Layer B = two engines" split is an INCREMENTALITY strategy, not a data-model split. CONFIRMED.

2. **BLAST-COUNTING IS A DEAD END**: Not viable for live stroma queries. Should be treated as OFFLINE/batch computation or approximated with structural heuristics.

3. **DETECTION + CONDUCTANCE ARE CHEAP**: Sub-10ms at antigen's real scale. Viable as live stroma queries.

4. **GRAPH DENSITY**: 4714 edges → 33,882 reachable pairs = 7.2x edge amplification. Deepest chain = 12 levels.

5. **NEGATIVE CONTROL** (self-NC): `reach-pairs == dist-pairs == blast-pairs = 33,882` — if semirings computed different relations, the rule shapes would differ. They agree exactly. NC passes.

**Observer's assessment**: The semiring-spike-results is the most formally attested finding in the expedition (scientist-signed, negative control documented). The 291,862ms blast result is the strongest empirical evidence yet for the LE5 constraint — it's not just "counting semirings are theoretically problematic on cycles," it's "we measured it and it's 4.9 minutes." This kills blast-as-live-query definitively. The LHF's independent hang (std-Rust implementation diverged) is now the second independent confirmation across different engines.

**STE's consequence** (noted on gen-2-seed): blast-COUNT is offline → doesn't count as a live engine driver. The census question narrows further: of candidate all-pairs consumers, blast-COUNT is offline, blast-as-SET is cheap-live (cheap-base serves it). Live engine now justified by AT MOST: data-flow density + LLM-grounding precompute (deferred) + field-lattice (may be batch-cheap on frozen snapshot).

---

## Updated Running Verdict Table — After Wave 5

| Claim | Verdict | Strength | Evidence |
|-------|---------|----------|---------|
| GEN-2 opened | YES | HIGH | Captain gen-2-seed note |
| GEN-2 recombined candidate | cheap-base + salsa + engine ONLY-IF | HIGH | Captain selection + LHF census + semiring results |
| Call/lineage tier consumer query shape | SINGLE-SOURCE LOCAL (no all-pairs) | HIGH | Two shipped antigen traversals (finalize.rs:259, recurrent.rs:158) + LHF census |
| Datalog engine needed at call/lineage tier (0.7) | NO CURRENT CONSUMER | HIGH | LHF census on shipped code; engine deferred pending data-flow census |
| Blast-counting = live query | DEAD END | HIGH (doubly confirmed) | Scientist: 291,862ms (66,000×); LHF: std-Rust hung. Two independent engines. |
| Detection + conductance = live semirings | VIABLE | HIGH | Scientist: 2.6ms / 4.4ms at n=2948 |
| Same rule shape, 3 semirings | CONFIRMED | HIGH | Scientist: reach-pairs = dist-pairs = blast-pairs = 33,882 (NC passes) |
| Data-flow tier density | UNMEASURED | HIGH concern | LE9' re-opens at data-flow depth; STE pre-registered faithful-method discipline |
| Data-flow consumer query shape | UNMEASURED | HIGH concern | LHF: "one live spike left" for engine-vs-no-engine; census covers call tier only |
| Field-lattice (all-graph semiring) | engine-indicated but batch-cheap possible | MEDIUM | STE: may be batch-cheap on frozen snapshot (ADR-067 clause 5) |
| LLM-grounding precompute | 0.8+ capability, no 0.7 consumer | HIGH | LHF census; deferred same class as supply-chain |
| spike/dm4-arm-b-dbsp-delta | OPEN (0/2 signed) | — | Awaiting pathmaker + scientist |
| spike/dm4-salsa-ascent-compose | OPEN (0/2 signed) | — | Result landed (Wave 2 note); signatures never captured |

---

## Observer Questions — GEN-2 Active Queue

The census has sharpened the remaining open questions to a small set:

1. **Data-flow tier density + query-shape** (GEN-2 spike-1, the only remaining engine-vs-no-engine decider for 0.7): Build the faithful-method spike using real MIR data-flow edges. Report: forward-reach distribution (median, mean, max, TAIL), fan-out hub identification, inter-procedural scope. Run BOTH all-pairs AND single-source taint costs. The outcome (a/b/c) determines whether the engine is needed at all for the data-flow tier.

2. **Field-lattice batch-cost** (GEN-2 spike, lower priority): Does the field/percolation semiring run batch-cheap against a frozen snapshot? If yes, the datalog engine's only remaining justification is incremental maintenance of the field lattice — and salsa already handles the freshness gate.

3. **DBSP/DD batch-edit Δ-cost + memory** (spike/dm4-arm-b-dbsp-delta, pending): The 65-75× single-edit win is measured. Batch-edit and memory remain. Until measured, DBSP allele profile is incomplete for Pareto selection.

4. **CLASS-2 false-edge rate** (scientist, still open): Symbol-level re-diff on heuristic vs SCIP. Needed before syntactic tier used as a fallback in any GEN-2 chromosome.

5. **Signature cleanup**: `spike/dm4-salsa-ascent-compose` (result landed in Wave 2) and `scout-prior-art-map` both show 0/required-signers despite results being deposited. Pathmaker and scientist signatures on dm4-salsa-ascent-compose are still pending; captain signature on scout-prior-art-map pending.

---

## Camp Substrate Alignment — After Wave 5

**New islands since Wave 4**:
- `gen-2-seed` — OPEN (captain signed the content note but the island itself awaits captain signature to close)
- `spike/atk-dataflow-density-faithful-method` — PARTIAL (1/3 clauses signed, working-tree-only — DURABILITY RISK)
- `spike/dm4-arm-b-dbsp-delta` — OPEN (0/2: pathmaker + scientist)
- `semiring-spike-results` — SIGNED COMPLETE (scientist)

**Durability concerns**:
- `spike/atk-dataflow-density-faithful-method`: working-tree-only. 1/3 signed. The notes are in camp substrate but the island itself is at risk without a commit.
- `the-spike-yard/measurement-scaffold`: still working-tree-only despite being signed complete. The signed status is durable (camp substrate is on disk) but the uncommitted state means a branch reset could wipe the notes directory.

**Observer's note on the dataflow density spike**: The status showed `spike/atk-dataflow-density-faithful-method` as complete (1/3 clauses) — but I haven't read its notes yet. This spike may already have results. Need to read it.

---

*Wave 5 integrated. GEN-2 open. Engine-vs-no-engine for 0.7 narrows to ONE live spike: data-flow-tier density + query-shape. Everything else on the call/lineage tier is settled: single-source-local = cheap base wins. Blast-counting = 66,000× = offline permanently. Semiring-spike-results SIGNED.*

---

## Wave 6 — Data-Flow Density Spike Setup (~00:05–00:09 UTC, 2026-06-29)

4 events on `spike/atk-dataflow-density-faithful-method`. GEN-2 spike-1 gets a pre-registration protocol, a real MIR seed measurement, three new data-flow negative controls (DF-NC1/2/3) physically built and MIR-verified, and a cross-domain IFDS borrow that reframes the density question as "naive cross-fn vs summary-edge" rather than "sparse vs dense."

---

### Expansionist: Density Measurement May Answer the Wrong Query

Note on `spike/atk-dataflow-density-faithful-method` (00:05 UTC):

**The reframe**: Every mature flow-network domain that faces a dense flow graph does NOT compute all-pairs closure on it:
- Epidemiology: single-source forward simulation from specific seeds (bounded-depth, never all-pairs)
- Power-grid: N-1 contingency screening (test the specific failures, not the full cascade-closure)
- AML/taint: trace from a flagged source or backward from a sink (single-source, never all-pairs)

And LHF's census already found antigen's real consumer query is single-source local traversal.

**Proposed measurement design**: measure BOTH:
- (a) all-pairs data-flow closure density (the trap the STE named — may explode)
- (b) single-source-from-a-realistic-sink taint cost on the SAME graph (likely cheap + bounded even when (a) explodes)

If (a) explodes but (b) stays cheap → density makes all-pairs INFEASIBLE AND IRRELEVANT simultaneously → data-flow tier needs bounded single-source taint, NOT a datalog all-pairs engine → engine-vs-no-engine resolves to no-engine EVEN IF THE GRAPH IS DENSE.

**The key insight**: the query-shape, not the density, is load-bearing.

---

### Test-Architect: ATK-DATAFLOW-001 Born-Red — Faithful-Method Protocol

Note on `spike/atk-dataflow-density-faithful-method` (00:06 UTC):

**Real MIR seed** (measured, not synthetic): antigen-fingerprint crate (smallest crate, 6 files) via `cargo rustc --emit=mir`. 6.1s, 738KB textual MIR, 283 fn bodies.

Within-function forward-reach (assignment `_dst = ..._src...` → src→dst edge):

| Metric | Within-fn data-flow | Call-graph (comparison) |
|--------|-------------------|------------------------|
| Median | 2.0 | 2 |
| Mean | 5.0 | 11.5 |
| Max | 136 (in a 152-local body) | 387 |

Within-fn data-flow: shallow-ish, bounded by function size.

**THE ACTUAL OPEN QUESTION**: the CROSS-FUNCTION data-flow graph. A value flows through call args into the callee's locals and back via the return — inter-procedural threading is where data-flow CAN densify in a way the call-graph cannot, because one value visits many functions' interiors. The call-graph's "sink-hubbed-and-shallow" property does NOT transfer to inter-procedural value-flow.

**Born-red faithful-method protocol** (4 conditions, spike RED until all satisfied):
1. REAL CHARACTER: seed from real MIR (this antigen-fingerprint probe; extend to a mid-size crate). Do NOT invent an inter-procedural fan-out parameter.
2. TILE THE REAL GRAPH: antigen growing = more functions of the same flow-character, NOT a denser-SCC synthetic.
3. MEASURE DISTRIBUTION: report median AND max AND tail — NOT just mean. A mean hides both a sink-shallow truth and a fan-out cliff.
4. STATE SCALE TAG: every data-flow density number tagged (within-fn vs inter-procedural-closure). A within-fn number is NOT an inter-procedural number.

**New data-flow negative controls** (DF-NC1, DF-NC2, DF-NC3) defined:

| Control | What it tests | Clear failure |
|---------|--------------|---------------|
| DF-NC1 — call-boundary | Value flows through a call: `caller _1 → callee param → callee return → caller _2`. | A method treating the call as opaque MISSES the _1→_2 dependency. |
| DF-NC2 — aliasing | `_2 = &mut _1; *_2 = x`. ONLY mir-exact sees this. | Cheaper tier (syntactic/SCIP) returns "a == seed" (wrong). The falsifier for "mir-exact earns its cost." |
| DF-NC3 — struct-roundtrip | Value moved into struct field, read back: `_3.field = _1; _4 = _3.field`. | No _1→_4 flow reported. |

---

### Test-Architect: DF-NC1/2/3 PHYSICALLY BUILT

Note on `spike/atk-dataflow-density-faithful-method` (00:07 UTC):

Physical fixture files at `scratchpad/measurement-scaffold/dataflow-negative-controls/`:
- 3 minimal Rust files, each compiles + emits MIR, each showing its expected data-flow
- `DF_EXPECTED.json` — expected-facts manifest
- **MIR-verified**: the expected facts are read straight off the MIR (rustc's ground truth), not asserted

**DF-NC1 (call-boundary)**: MIR shows `_2 = id(copy _1); _3 = AddWithOverflow(copy _2,..)` → dependency `_1(x)→_2(z)` threads THROUGH the call. A tier that treats `id()` opaque MISSES `_1→_2`.

**DF-NC2 (aliasing)**: MIR shows `_3 = &mut _2; (*_3) = const 99; _0 = copy _2` → `a(_2)` is modified THROUGH the alias `p(_3)`; returns 99 not seed. ONLY mir-exact sees this. If a cheaper tier claims to handle data-flow, DF-NC2 is the falsifier: it WILL return "a == seed" (wrong).

**DF-NC3 (struct-roundtrip)**: MIR shows `_2 = Box2{field: copy _1}; _0 = copy (_2.0)` → `_1(w)→field→_0(r)` round-trips through the struct field.

**New test-class added to suite vocabulary**: MIR-VERIFIED DATA-FLOW SPECIMENS — minimal source whose known-correct place/local flow is read from MIR, so the negative-control is itself checked against rustc's ground truth. Distinct claim-kind from the call-graph battery (NC1-NC5, which defends reachability claims).

---

### Expansionist: IFDS — The Cross-Domain Solved Problem

Note on `spike/atk-dataflow-density-faithful-method` (00:08 UTC):

**IFDS** (Reps-Horwitz-Sagiv, POPL 1995): interprocedural data-flow = reachability on the **EXPLODED SUPERGRAPH**, made tractable by **PROCEDURE SUMMARY EDGES** — summarize each callee's input→output flow ONCE, reuse at every callsite, match call/return edges along REALIZABLE paths for context-sensitivity.

Key: IFDS is polynomial and does NOT compute the dense all-pairs cross-function closure. It's the formal name for the same "refuse to compute the dense thing" discipline that epidemiology/power-grid/AML all use independently.

**Consequence for the faithful-method spike**:

The cross-function spike should measure BOTH:
- (a) Naive cross-fn closure (expect explosion — this is measuring the NON-TRACTABLE formulation, confirms that "datalog all-pairs" is the wrong answer)
- (b) Summary-edge reach via IFDS-style propagation (expect bounded — this is the tractable formulation antigen would actually ship)

If (b) stays cheap → cross-function density is a SOLVED problem, not a blocker. Antigen would use summary edges, NOT naive inline-everything closure.

**IFDS restriction**: finite fact-set, distributive over union. Taint qualifies (the taint fact-set is finite). The observer should note whether antigen's intended data-flow lattice satisfies IFDS restrictions.

**The new measurement question**: "how big do antigen's per-fn flow summaries get?" — bounded by fn signature arity, which is small (typically ≤10 parameters). Likely stays sparse even when the naive closure would explode.

---

### Observer Assessment: Wave 6 Peer-Review

The Wave 6 setup is methodologically the strongest preparation for any spike in this expedition. Two independent lines converge before the spike even runs:

1. **Expansionist's domain-structural argument**: density of (a) makes all-pairs infeasible AND irrelevant simultaneously, because the real query is (b) single-source taint.
2. **Test-architect's real MIR seed**: within-fn data-flow is shallow (median 2, max 136) — this is the anchor. The open question is genuinely inter-procedural, and the born-red protocol prevents the near-miss that almost happened with the call-graph spike.
3. **IFDS borrow**: the formal name for "procedure summaries make inter-procedural tractable." This is a 30-year-old solved problem. If antigen's taint lattice satisfies IFDS restrictions (finite fact-set, distributive over union), the density concern dissolves into "how big are the summaries?" — which is bounded by fn arity.

**Strongest observer concern**: The spike has 4 events setting up methodology but NO measured data-flow cross-function result yet. The island shows 1/3 signed (presumably the test-architect's born-red signature). The actual measurement is pending. The setup is excellent; the result is what matters.

**What I expect the result will show** (pre-registered hypothesis, H6):
- Naive cross-fn all-pairs closure: will densify and become slow for any non-trivial inter-procedural graph (the test-architect's born-red is correct in predicting the trap)
- IFDS-style summary-edge reach: will stay polynomial, bounded by fn arity, comparable to the call-graph's cost profile
- The finding will be: "use IFDS-style summaries (not naive inline-everything)" — same as what the call-graph density spike found for the batch-vs-demand-driven question

**If H6 holds**: the engine-vs-no-engine question remains resolved in favor of no-engine even at the data-flow tier. The cheap base + salsa + IFDS-style summaries serves the real consumer query (single-source taint) without a datalog all-pairs engine.

---

## Updated STAGE 1 — INPUT: Data-Flow Allele Sharpened

| Sub-allele | F-CORRECT | F-INTEG | Feasibility | Notes |
|-----------|-----------|---------|-------------|-------|
| syn (syntactic) | 52% miss + CLASS-1 4% FP + CLASS-2 UNKNOWN | WINS (zero) | FAILS LE1/LE2 for semantic consumers | Active harm risk (CLASS-2 noise) |
| SCIP (resolved-refs) | WINS for call edges | GOOD | FEASIBLE | Input-benchmark SIGNED |
| MIR-exact (rustc MIR) | WINS for call + data-flow | POOR (per-crate compile cost) | FEASIBLE (required for DF-NC2 aliasing) | DF-NC2 PROVES: mir-exact earns its cost for data-flow |
| SCIP + MIR hybrid (CH-C) | WINS for both layers | HIGH (dual integration) | FEASIBLE (LE3: dual-resolver for parity oracle) | Required for self-distrust case |

**DF-NC2 changes the calculus**: the aliasing negative control (`*_3 = const 99` modifying `_2` through `&mut _1`) is invisible to syntactic and SCIP tiers. Only MIR-exact catches it. This is not theoretical — DF-NC2 is a compiled, MIR-verified specimen. Any spike claiming F-CORRECT on data-flow without passing DF-NC2 is measuring speed on a wrong answer.

---

*Wave 6 integrated. Data-flow density spike: setup complete, result pending. Key pre-registered insight: density makes all-pairs infeasible AND irrelevant simultaneously (single-source taint is the real query). IFDS summaries are the tractable formulation. DF-NC1/2/3 physically built and MIR-verified. H6 pre-registered: IFDS-style summaries stay bounded even when naive closure explodes.*

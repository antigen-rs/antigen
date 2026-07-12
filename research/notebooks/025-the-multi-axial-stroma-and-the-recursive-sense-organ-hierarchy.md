# 025 — The Multi-Axial Stroma and the Recursive Sense-Organ Hierarchy

*Synthesis of a 3-agent research fan-out (2026-07-11) — three fresh minds on distinct facets (stroma
sources · 1st-order organs · the multi-order hierarchy DAG) — woven together, plus Tekgy's multi-adic /
recursive refinement (the concept×tier lattice). Successor to nb 022 (the two-winged body) / 023 / 024.
Tier-honest inline: **CONFIRMED** (grounded in the frame) / **DESIGN** / **CONJECTURE**.*

---

## §1 — The stroma is a MULTI-AXIAL space, not a fidelity ladder (DESIGN)

The "3-tier edge ladder" (`syntactic < resolved < mir`) is only **one axis** — fidelity (the same
call-edge at rising resolution = `ResolutionTier`). The real stroma is **9 orthogonal axes**: same nodes,
genuinely different question. **antigen currently populates ~1.5 of the 9; six are nearly empty.** "The
stroma HOLDS ALL" now has a concrete meaning: hold all nine axes at full fidelity.

| # | axis | what it adds | representative sources | status |
|---|---|---|---|---|
| 1 | **fidelity** | same edge at rising resolution | syntactic → SCIP → HIR → THIR → MIR → mono → LLVM-IR | partial (syntactic + SCIP) |
| 2 | **reachability-modality** | could / **did** / feasible / will-codegen — different reach-predicates | static graph · **coverage** · symbolic/Kani · mono-collector | ~empty (no *did*) |
| 3 | **guarantee-opt-out** | where Rust's guarantees lapse | `unsafe` map · FFI/ABI · `asm!` · build.rs-generated | empty |
| 4 | **aliasing / ownership** | who-aliases/owns/races whom | Polonius (`.facts`) · Miri borrows · TSAN | empty |
| 5 | **trait / type implication** | Send/Sync flow, blanket-impl reach — a DAG, not a call edge | trait solver · coherence · layout | empty |
| 6 | **the config FIBER** (multiplicative) | code is a *family* of graphs fibered over cfg × features × targets | cfg geometry · `cargo hack` powerset · target matrix | collapsed to one fiber |
| 7 | **temporal / labeled history** | same nodes indexed by time + human outcome | git · blame · churn · **co-change** · **SZZ** · CI · issues · reviews | barely (DETECT diff only) |
| 8 | **semver / public surface** | what consumers see; breaking-change facts | rustdoc-JSON · cargo-public-api · reverse-deps | empty |
| 9 | **cost / measurement** | numeric weights (perf, alloc, layout) | criterion · dhat · `-Zprint-type-sizes` | empty |

**Within** an axis, sources are *refinements* (redundant-as-fidelity — keep the highest available, tag the
tier). **Across** axes they are orthogonal — a fusion of two axes sees what neither can. **Axis 6 is
special: it is *multiplicative*** — every other axis is really a family of graphs fibered over
config-space, and antigen currently projects to a single fiber.

---

## §2 — The organ layer: receptors vs integrators; fusion = axis-crossing (DESIGN)

1st-order organs are antigen's **innate immune layer**, and they split like innate immunity does:
- **Receptors** (single-axis) — one raw source → one per-node field. Biology's pattern-recognition
  receptors (a TLR reads one PAMP class). The four semirings, SCC-condensation, public-surface,
  unsafe-census, coverage-field, churn-field.
- **Integrators** (fusion — *cross-axis*) — combine ≥2 axes into a verdict. Biology's dendritic
  cell / macrophage. **This is where the new failure-classes live**, because a failure spanning two axes
  can only be regulated by an organ with the variety to see both (Ashby's requisite variety). Every
  fusion organ *is* an axis-crossing:

| fusion organ | axes crossed | immune analog |
|---|---|---|
| **dead-guard** (could ∧ ¬did) | 1 × 2 | **anergy** — a lymphocyte present but never activated |
| **fragility-central** (churn × path-count) | 7 × 1 | chronic inflammation on a load-bearing joint |
| **antigenic-drift** (guarded-edited-after-witness) | 7 × defense-topology | **escape mutant** — antibody no longer binds the mutated antigen |
| **shared-mutable-boundary** (Polonius × boundary-edge) | 4 × 1 | tight-junction breach / gut-barrier permeability |
| **hidden-coupling / paracrine** (co-change × ¬edge) | 7 × ¬1 | paracrine signaling — coordination with no synapse |
| **scar-tissue / immune-memory** (SZZ bug-fix × node) | 7 | **memory B-cells** — remembered sites of past infection |

---

## §3 — The order=tier DAG: B2′ as a TYPE RULE (CONFIRMED-consistent)

Set **order = tier**: an order-k organ reads facts at tiers ≤ k and writes at `1 + max(input tiers)`.
This makes **B2′ a type rule, not a discipline** — a write can never land at-or-below a read, so cycles
are *unrepresentable* and the accumulate/cancel oscillator cannot form (sense lives at tiers 1–4, act at
5+; they can never write a shared tier). The trunk, with its emergence rationale:

- **order-0 · projection** (raw edges → per-node *path*-scalars via closure): the four semirings + SCC.
  New sight = "what a node inherits along all paths" — a property no single edge states.
- **order-1 · topology** (fold the scalar-*field* into an invariant): min-cut/root-cause, blast-region,
  the confidence barcode (tier-filtered path homology), field-lens Mapper communities. New sight =
  *relations between nodes' fields* (a cut is a property of a set; a homology class of a path-class) — no
  scalar holds it.
- **order-2 · criticality** (join *two* topologies): severity = how much blast a cut carries × how many
  *communities* it bridges; and the **phantom-guard** — a feature asserted at `Syntactic` that dissolves
  at `T3Mir` = trust on coarse evidence. New sight = a relation *between two topologies*.
- **order-3 · the INTEGRATOR** = the complete **"Is"** (per-locus criticality × defense-validity). Its
  peak cell: *critical site × invalid/phantom witness = max deficiency* — the product of two
  independently-climbed sub-trees.

**Two structural facts:** (a) **raw axes stay re-readable at every order** — a tier-0 channel is not
consumed; a high organ may re-read the raw call graph directly (the verification ladder's L4 does — it
sits at the top yet re-reads raw reachability to catch the vacuous-green). (b) **Fixpoint** at the
integrator (Knaster–Tarski + finite axes + strict tier ascent) — and *the fixpoint rises only when the
stroma gains a new axis.* Above it sits the **hinge** (COMPARE = Should − Is: the conformance-monitor,
reading the "Is" + the external `#[antigen]`/`#[presents]` "Should"), the one-way membrane that hands the
GAP field to the efferent wing. Variable-**input-arity** organs (affinity-maturation, all-organ
meta-sensing, cluster-arity-dependent folds) exit to the learn-core — litmus: **count the input ports,
not the payload.**

### The verification ladder as the canonical multi-order check
`L1a name-resolves (order-0, reads symbols) → L1b body-non-empty (order-0, reads spans) → L2 assertions
(order-1, +AST) → L3 passes (order-2, +exec) → L4 reaches-the-guarded-site (order-3, +raw call graph)`.
Every rung is a different organ at a different witness-tier, and **every rung freshly re-reads a NEW raw
channel** its predecessor never touched. Certification of "defended" is the **AND-fold at the TOP** —
so the **unverified-trust-extension** failure-class (trust extended at a coarse level without verifying
the fine level, nb / charter `antigen-unverified-trust-extension`) is **structurally uncertifiable by any
single organ.** Collapse any rung and reopen exactly one hole (drop L1b → dangling witness; L2 →
tautology witness; L3 → red-test-counted; L4 → vacuous-green).

---

## §4 — The RECURSIVE concept×tier lattice (Tekgy's refinement — the key new structure) (DESIGN)

§3's trunk reads as *distinct organs per tier*. The truer structure is **multi-adic**: the hierarchy is
a **CONCEPT × TIER lattice**, not a thin trunk.

- **A concept recurs across tiers.** *Fragility* does not live at one order — it has a version at each:
  - `fragility@0` — raw churn-field (Axis-7 scalar per node).
  - `fragility@1` — churn × path-count/blast (fuse Axis-7 × Axis-1 closure): fragility weighted by how
    much flows through the node.
  - `fragility@2` — `fragility@1` refined by community-structure (fragile **and** bridging two
    communities = worse): reads `fragility@1` + the community organ.
  - `fragility@3` — `fragility@2` refined by criticality + defense-validity (a fragile bridging node
    that is **also** undefended = max): reads `fragility@2` + criticality + the ladder.
  - Same *concept*, monotonically sharper — because a concept's *true* form often isn't computable until
    higher structure exists, but a crude first approximation exists early.

- **The refinement operator is tier-monotone → acyclic.** `C@k` reads `C@(k−1)` — a *self-reference*, but
  strictly up-tier (`k` reads `k−1`), so **B2′ and acyclicity hold.** This is **recursion in the CONCEPT
  dimension, realized as a monotone climb in the TIER dimension** — *not* a cycle in the tier-DAG. The
  distinction is load-bearing: self-similar concept ≠ self-referential tier.

- **Each tier is a MIX** (Tekgy: "1st and 2nd and new 3rd order inside the 3rd tier"): every tier's
  organs = {all lower concepts, **refined** by the newly-available structure} ∪ {**new** concepts that
  only emerge at this tier}. So 1st-order organs are re-used as 2nd-tier *inputs* alongside the
  uniquely-new 2nd-order concepts — much of the time. **The lattice THICKENS as it climbs** (each tier is
  wider, not a single new organ).

- **The recursion IS the expansion.** Climbing does three things at once: every existing concept gains a
  richer version; new concepts emerge; both feed further up. The concept converges to its *true* form at
  the fixpoint (integrator), passing through cruder versions on the way.

**Rhymes that confirm the shape:**
- **Renormalization / multigrid**: the same observable (a "coupling") recomputed at each scale, each
  scale informing the next. `C@k` = the concept at scale/order k; tracking `{C@k}` across k is a
  **persistence computation** — the same *parametrized family of shapes* as nb 022 §11.8 (λ ×
  Mapper-resolution), here parameterized by **order/tier**.
- **Multi-scale immunity**: the immune system reuses its own concepts across scales — *recognition*
  (molecular receptor-binding → cellular T-cell → tissue lymph-node → systemic), *memory* (molecular
  affinity → cellular B-cell → organismal), *tolerance* (central thymic → peripheral Treg). Each concept
  recurs, refined, at each scale. Multi-scale immunity **is** Tekgy's multi-tier fragility. (Biology is
  load-bearing, ADR-003 — it predicts the concept×tier lattice.)

**CONJECTURE (worth a falsifier):** every genuine antigen-concept is multi-adic — it admits a crude
`@0`/`@1` approximation that a monotone refinement operator sharpens up the tiers to a fixpoint form. If a
concept only exists at one tier with no cruder approximation below and no sharper form above, it is either
mis-placed or not a genuine concept. (Falsifier: find a load-bearing concept with a single admissible
tier.)

---

## §5 — Convergences (the three-way corroboration) + the strategic headline

**Three fresh minds landed on the same structures** (corroborate()-across-independent-sources):
- **did-vs-could / coverage** — the source-facet named it Axis 2; the organ-facet built the dead-guard
  organ on it; the hierarchy-facet put the raw-call-graph re-read at L4 for the vacuous-green. All three
  centered coverage as the ladder's load-bearing top rung.
- **VCS/temporal history = antigen's missing recognition engine** — the source-facet ranked co-change +
  SZZ Tier-S; the organ-facet invented "scar-tissue / immune-memory" (*git history **is** antigen's
  B-cell memory, un-harvested*). The most on-thesis finding of the fan-out: antigen is "structural memory
  of failure-classes," and the failure memory already exists in the git log, for free.
- **the unverified-trust / stale-witness class** — source (Axis-7 blame timestamps) × organ
  (antigenic-drift) × hierarchy (ladder-as-DAG, uncertifiable-by-one-organ) — four independent
  derivations converging on the class found the same morning.

**The strategic headline — enter new AXES, don't climb the fidelity ladder:**
1. **Coverage → the did-reach axis** — one test run, *on stable*; unblocks the ladder's L4 (defeats the
   verified vacuous-green hole). Best leverage-per-cost.
2. **Co-change + SZZ → the temporal axis** — pure git, no compile/exec: the recognition-not-design engine
   antigen is missing + coupling edges with no static shadow. Near-zero cost, deeply on-thesis.
3. **The config-fiber (Axis 6) is a decision to make *now*** — multiplicative, structurally-guaranteed
   need (ADR-007); retrofitting a fibered substrate later is the expensive path.

---

## §6 — Tier-honesty + what's next

- **DESIGN-tier:** the 9-axis stroma, the receptor/integrator organ layer, the concept×tier lattice — all
  design, not built. The current code holds Axis 1 (partial) + Axis 7 (DETECT diff only).
- **CONFIRMED-consistent:** order=tier makes B2′ a type rule; the fixpoint argument; the
  count-the-ports Fock litmus; the ladder-as-DAG defeating unverified-trust — all cohere with nb 022's
  ratified frame.
- **Next moves (candidate charters):** `coverage-did-reach-axis` (the L4 unlock) and
  `temporal-recognition-axis` (co-change/SZZ = the recognition engine) — the two cheapest new-axis
  entries; and a decision-record for the **config-fiber** substrate shape.
- **The multi-adic CONJECTURE (§4)** wants a falsifier before it hardens past DESIGN.

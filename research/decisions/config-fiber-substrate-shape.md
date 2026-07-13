# Decision-Record (DRAFT — pre-ceremony) — The config-fiber substrate shape

> **Status: DRAFT.** A design decision-record staged for the ADR lifecycle (`docs/process.md`) — NOT yet
> ratified. Drafted 2026-07-13. Source: nb 025 §1 (Axis 6, the config fiber) + the dissipative-adaptation
> resonance ([Te Brinke et al., *Nat. Nanotechnol.* 2018](https://www.nature.com/articles/s41565-018-0192-1)).

## Context

The stroma is a **multi-axial** space (nb 025). **Axis 6 — the config fiber — is *multiplicative***: the
codebase is not one graph but a *family* of graphs fibered over config-space (`cfg × features × targets`).
Every other axis (reachability, aliasing, temporal, …) is really fibered over this one. antigen currently
projects to a **single fiber** (the default config). Two facts make the substrate *shape* w.r.t. this
axis a **"decide now"** call:

- **Retrofit is expensive (ADR-007 structural-guaranteed-need).** antigen commits to flagging *every*
  recurrence site. A config-conditional item (`#[cfg(feature = "x")] fn …`) is a recurrence site **in
  another world** the single-config scan never visits. So the fibered substrate is *structurally
  guaranteed* — and threading a fiber index through a substrate that assumed one world later is a
  pervasive rewrite, not an addition.
- **Combination failures are real and single-fiber-invisible.** "`A` works, `B` works, `A+B` breaks"
  (feature-unification bugs) live *only* on this axis. A single-fiber stroma cannot even represent them.

## The decision

**What shape is the stroma with respect to config-space?**

| option | now | later | verdict |
|---|---|---|---|
| **1. Single-fiber** (project to default; retrofit if needed) | simplest | pervasive rewrite to add fibers; blind to combination-failures | rejected — the retrofit ADR-007 warns against |
| **2. Fibered-from-start** (materialize the full `cfg × feature × target` product) | correct variety | combinatorial blowup — most fibers never queried | over-build — YAGNI on the *materialization* (though NOT on the abstraction) |
| **3. Fiber-INDEXED abstraction + adaptive compartments** (recommended) | the substrate is fiber-indexed *by construction*, initially populated with one fiber; fibers added/split on demand | additive — no rewrite; combination-failures representable when a second fiber lands | **recommended** |

## Recommendation — fiber-indexed by construction, materialized lazily, compartments adaptive

**Commit to the fiber-indexed *abstraction* now; do NOT materialize the full product.** Every stroma
node/edge/fact carries a **fiber coordinate** (a config-world identifier) from day one, even while only
the default fiber is populated. This makes adding a fiber **additive** (populate a new coordinate) rather
than a rewrite — dodging exactly the retrofit ADR-007 forbids — without paying the combinatorial cost of
materializing worlds nobody queries. (This is the same discipline as the substrate/organ frame: the
*substrate* holds the fiber structure; *materialization* is the organ's lazy, on-demand projection.)

### The adaptive-compartment framing (the dissipative-adaptation resonance — DESIGN-tier)

The [self-dividing-fibrils paper](https://www.nature.com/articles/s41565-018-0192-1) reframes what a fiber
*is*, and it sharpens option 3. Its mechanism: **open compartments locally modulate a driven,
dissipative self-assembly by restricting energy absorption — and that compartmentalization is what
enables both adaptation and self-*division*.** Ported (RESONANCE, *measure before collapsing to identity*
— per the render-math discipline, a rhyme earns "import" status only when measured against the code):

- A config-fiber is a **compartment** — a world with its own local "fuel" (which code exists, which edges
  fire, which failures manifest under that config). Each compartment adapts *its own* stroma/organs to
  *its own* drive.
- **Fibers need not be a statically-enumerated product.** They can **self-divide**: start with one
  compartment; when a feature-world's failure-behavior *diverges* enough from the default (measured — a
  fiber-distance exceeds a threshold), the compartment **splits into a new fiber** that materializes
  independently. Fibers are born where the drive demands them, not pre-enumerated. This is
  recognition-not-design at the substrate level: the *drive* (config-conditional divergence) grows the
  fiber structure, we don't hand-enumerate `2^n` feature worlds.
- The default-fiber stays the "parent"; divergent worlds are "descendants" — the **same clonal-lineage /
  self-division shape** as `#[descended_from]` and "distinct remedies force distinct units," now at the
  substrate/config layer.

**Tier-honesty:** the fiber-indexed abstraction (the concrete recommendation) is ADR-007 structural need,
buildable, ceremony-ready. The **adaptive/self-dividing compartment** part is a **DESIGN-tier resonance**
— genuinely shaped like the physics, but it must be *measured* (does config-divergence actually cluster
into a few natural fibers, or is it a smooth continuum?) before it hardens past conjecture.

## Falsifier / open questions (for the ceremony)

- **The fiber-distance metric.** "Diverges enough to split" needs a measurable notion of fiber-distance
  (e.g. how much the reachability/failure structure differs between two configs). Undefined today.
- **Does divergence cluster?** The self-division framing assumes config-space partitions into a *few*
  natural compartments. If feature-interactions produce a smooth continuum (no natural cut), self-division
  degrades to the `2^n` product and option 3's adaptive part fails (falling back to fiber-indexed-lazy,
  which still holds). Measure on a real feature-rich crate.
- **Interaction with the temporal axis.** Fibers are spatial (config); the temporal axis is time. A fiber
  can be born *and* age — the two axes compose. Out of scope here; flag for the multi-axis synthesis.

## Recommendation summary

**Ratify: the stroma is fiber-indexed by construction (every fact carries a config-world coordinate),
materialized lazily (default fiber first, others on demand).** This is the ADR-007 structural need,
paid cheaply. **Scope as DESIGN-tier follow-on: adaptive/self-dividing compartments** (fibers born where
config-divergence demands them) — the dissipative-adaptation import, to be *measured* before it hardens.

---

## RESEARCH-REVIEW AMENDMENTS (2026-07-13) — AMEND, do not ratify as-is

*From a pre-ceremony research-review agent (attack + SPL/variability prior-art) + the dissipative-adaptation
and self-organization surveys. The draft's instinct (fibered, lazy, adaptive-later) is right and **~80%
already recognized in the shipped substrate** (`cfg` is in `StromaNodeId` and the salsa `Locator`), but four
things must land before ratification.*

- **A1 — the "single-fiber" premise is FALSE (substrate-over-memory correction).** The real state is not
  single-fiber; it is **cfg-partitioned IDENTITY + cfg-flattened SCAN + single-fiber POPULATOR**. `cfg`
  already rides in `StromaNodeId`. The draft's premise must be corrected to this three-part reality.
- **A2 — the primitive is a PRESENCE CONDITION (as a BDD), NOT a flat coordinate.** Formally settled: a
  variational value is a total function `C→A` (Choice Calculus, TOSEM 2011; Variational Data Structures,
  Onward! 2014) — a **coordinate addresses one input** (a single minterm), a **presence condition denotes an
  arbitrary sub-domain**. The coordinate cannot even represent this record's own combination-failure payoff
  (`A+B` breaks). Replace "fiber coordinate" with **presence-condition-as-BDD** throughout. This upgrades the
  ADR-002 compose argument from design-taste to *formal*.
- **A3 — COMPOSE the config-space math layer; BUILD only the Rust front-end (ADR-002).** Compose: a BDD lib +
  SAT + presence-condition abstraction + SPLLIFT-style lifting + **Dimovski's BDD-lifted abstract-interpretation
  domain** (size ≈ #distinct-results, *not* #configs — this is the answer to the blowup concern) + **Batory's
  feature-model→propositional-formula** (from `Cargo.toml`) + **rust-analyzer's own `CfgExpr`/`DnfExpr`**. Build
  only: extract PCs from `#[cfg]`/features/target-triples onto stroma facts. Rust's disciplined `#[cfg]`
  (attribute-on-syntactic-item) makes this *materially easier* than the C `#ifdef` product lines this
  literature was built to tame.
- **A4 — the two open questions are now answerable by composition:** **fiber-distance = BDD-inequivalence**
  (+ a Hamming/Jaccard baseline); **clustering = COMPACTNESS is empirically supported (NIST interaction rule /
  Kuhn 2004: faults ≤6-way, ≤3-way covers 89–99%), but DISJOINT PARTITION is NOT.** Compactness ≠ clean
  partition — so a *small set of representative PC-regions captures the family*, but config-space does **not**
  cleanly partition into k disjoint fibers.
- **A5 — self-division stays DESIGN-tier on measured-but-WEAKER ground, and is RE-GROUNDED off physics.** The
  self-dividing-compartment framing *assumes disjoint compartments*, which A4 shows is unmeasured (compactness,
  not partition). AND: the dissipative-adaptation survey returned a **RHYME, not import** verdict (no work, no
  dissipation functional, no β — it fails the render-math identity test). So do NOT ground self-division in the
  physics. Re-ground it in antigen's own **bimodal-cluster criterion** (decidable from the affinity module's
  frontier geometry): *split a class/fiber when no single Pareto-optimal fingerprint is both tight AND complete,
  but a partition into two lineages closes the recall–precision gap.* Physics stays a naming/intuition source.
- **A6 — independent support (Kauffman NK).** The config-fiber IS the NK "K" parameter — feature interactions
  are epistasis; single-fiber landscapes are misleadingly smooth, the real (rugged) landscape is only visible
  fibered. A second field arguing *for* fibering.

**Four MUST-FIX before ceremony (agent claims — VERIFY against substrate first, a flag is a claim not an
order):** (1) correct the single-fiber premise (A1); (2) PC-as-BDD not coordinate (A2); (3) a claimed
**`constitute` populator-overwrite bug** that would falsify the "additive, no rewrite" recommendation —
verify and fix; (4) reconcile the ratified **scan-match-through posture (ADR-010 Amd 3)** against the stroma's
identity-partition.

**Amended verdict:** the recommendation holds in *direction* (fibered, lazy, adaptive-later) but must be
**amended** (A1–A6) and the four must-fixes closed before the attack-annotate → ratify ceremony.

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

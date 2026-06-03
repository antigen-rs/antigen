# Charter — the Effector Arm (ACT: the two-speed response)

*The cluster that lets antigen DO something about a detected failure-class, not just name it. Two speeds:
slow-and-curative (repair) + fast-and-containing (neutralization). Gated on sub-clause-F for any auto-apply.*

**Tier:** deep-future (chartered expedition).

---

## Organ-identity (what this IS)

The effector arm of the immune system. Detection is the afferent/recognition half; this is the *response*
— and biology runs it at two speeds. **Slow effector (repair):** heal the wound — emit a fix. **Fast
effector (neutralization):** contain the blast-radius *without* fixing — opsonize/quarantine the danger so
it can't spread while the slow fix is prepared. The two compose: neutralize now, repair when ready.

## The dreams in this cluster

- **`dream/repair-tier`** — heal the wound (SLOW effector). **Near-slice: the SUGGEST floor** — emit a diff,
  the human ratifies (antigen proposes, the dev disposes — same identity-rule as the runtime arm: antigen
  emits, the dev decides the action). AUTO-APPLY is the deep version, and it **charters behind
  sub-clause-F** (a fix applied without a validation gate is a trust-boundary violation).
- **`dream/neutralization-containment-effector`** — contain blast-radius WITHOUT fixing (FAST effector;
  `#[opsonize]` / `#[quarantine]`). Mark the danger so downstream tooling routes around it while the repair
  is prepared.

## Dependencies (what unblocks this)

- **A detected failure-class to act ON** — so this charter is downstream of the build-now detection spine
  (families/grammar) AND the learning core (to act on *discovered* classes, not just authored ones).
- **Sub-clause-F (ADR-005) gates AUTO-APPLY** — the SUGGEST-floor needs no new trust boundary (the human
  ratifies); auto-apply introduces one and must specify its validation check.

## Could-combine-with

- The **ROUTE charter** — neutralization/containment is *where* the effector acts; gradient-routing is
  *which sites* to recruit it to. Route + act compose into the response.
- `families/setpoint-corruption` — the `autoimmune-check` screen (ADR-042) is a degenerate effector (it
  detects over-protection but doesn't auto-fix); the repair-tier's SUGGEST-floor is the general form.

## Buildability / effort scoping

- **SUGGEST-floor: MODEST** — antigen already knows the fix-direction per family (ADR-038 carries
  "fix-direction" per genus: recover-info / tighten-the-guard / reorder-the-effect). Emitting a *suggested*
  diff from a known fix-shape is a real but bounded codegen step; the human-ratifies gate keeps it safe.
- **AUTO-APPLY: HARD + trust-sensitive** — a tool that edits the adopter's code is a serious trust
  boundary; charter it last, behind an explicit sub-clause-F validation (the fix must be verifiably
  correct, not just plausible — the same constructable+verifiable discipline as the build gate, applied to
  the fix).
- **Neutralization markers: CHEAP** — `#[opsonize]`/`#[quarantine]` are declaration-markers (like the
  shipped `#[antigen_tolerance]`); the marker is cheap, the *routing-around* behavior is the work.

## Invitation to deepen

The two-speed effector is why biology survives fast pathogens: you can't always fix in time, so you
contain first. The dev cognate: a known-but-unfixed failure-class on a hot path should be *quarantined*
(downstream tooling warned to route around it) even before the repair lands. The deepest version — antigen
suggesting the fix for a class *it discovered* (learning core) and *forward-deploying* it as a runtime
guard (afferent arm) — is the whole loop acting on itself.

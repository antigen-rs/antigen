# Charter — Feedback / Homeostasis (FEEDBACK: keep the signal legible)

*The cluster that keeps antigen's own output from becoming the problem — the event-bus that makes the
organs one system, the governor that damps runaway, the curation that lets memory wane, the density-tuner
per place. The SCRAM host + the emit-bus are already build-now; the governing logic charters.*

**Tier:** deep-future (chartered expedition). **Substrate (emit-bus + SCRAM host) is build-now.**

---

## Organ-identity (what this IS)

Homeostasis — the regulatory machinery that keeps the immune response proportionate. Three failure-modes
this cluster prevents: the response amplifying until *it* is the failure (cytokine storm / sepsis), memory
accumulating until the signal is all noise (no forgetting), and uniform density ignoring that some places
(the unsafe core, a hot path, generated code) warrant different scrutiny than others.

## The dreams in this cluster

- **`dream/cytokine-signaling-network`** — the event-bus that makes the organs ONE system. **Its substrate
  IS the ADR-039 emit-seam (build-now)** — every organ subscribes to the typed Finding stream; the
  *inter-organ propagation logic* (routing a signal to the organs with receptors for it) is the charter.
- **`families/cascade-becomes-the-problem-sepsis-anaphylaxis`** (the GOVERNOR half, re-filed from families
  to LOOP-A per ADR-037) — the cascade-governor / SCRAM. **Its HOST is build-now (ADR-036's out-of-band
  command-orchestration layer + the stage-sequencing invariant); the governor LOGIC charters** (the
  aggregate-over-population damper that decides when to trip SCRAM). Only the disturbance-half (a detectable
  workspace-aggregate alert-storm class) is a stdlib family.
- **`dream/forgetting-curve-memory-curation`** — memory must WANE; the temporal governor (counter-intuitive
  but structurally forced — a fingerprint for an obsolete class is noise). Shares the runtime
  incident-signal with evasion (the obsolete-vs-evaded discriminator needs the afferent arm).
- **`dream/tissue-locality-immune-privilege`** — right density per PLACE (`#[mhc]` / `#[immunocompromised]`);
  the spatial twin of the dial. The unsafe core warrants max scrutiny; generated code warrants less.

## Dependencies (what unblocks this)

- **The ADR-039 emit-seam + the ADR-036 SCRAM host are SHIPPED (build-now)** — this charter's *substrate*
  is already laid; what charters is the *logic* riding on it (the governor's trip-decision, the cytokine
  routing, the forgetting schedule).
- **Forgetting needs the afferent arm** (the runtime incident-signal is the obsolete-vs-evaded
  discriminator — a class that stopped firing in prod is a candidate to forget; one that's being evaded is
  a candidate to broaden). So forgetting is most valuable after the runtime charter.

## Could-combine-with

- The **size/adapt charter** (`adversarial-evasion-red-queen`) — forgetting + evasion share the
  obsolete-vs-evaded discriminator; they are two faces of "is this class still real?".
- The **route/registry charters** — the same gradient-field that routes attention is the substrate the
  cytokine-bus propagates on.

## Buildability / effort scoping

- **The cytokine substrate: ALREADY BUILD-NOW** (the emit-seam). The propagation logic is MODERATE (a
  pub-sub over the Finding stream keyed on receptor-declarations).
- **The cascade-governor: HOST build-now, LOGIC moderate** — the SCRAM host is ADR-036; the
  aggregate-over-population trip-decision is a separate pipeline stage (per the stage-sequencing invariant)
  that reads the population and short-circuits — bounded once the host exists.
- **Forgetting: MODERATE** but needs the runtime signal; **tissue-locality: CHEAP** (a per-place density
  multiplier on the dial — `#[mhc]`/`#[immunocompromised]` are declaration-markers).

## Invitation to deepen

The deepest homeostatic insight: antigen's own success is its biggest risk. The denser and more
correct the marking, the louder the wall — so the legibility spine (ADR-042) and this charter are the same
need at escalating scales (codebase → cascade → registry). The governor that damps the storm, the curation
that lets memory wane, the per-place density — all of them are antigen keeping antigen from becoming the
noise it exists to cut through. The strange loop: the immune system needs an immune system for its own
over-response, and that is `#[autoimmune]`-as-screen pointed at the regulator itself.

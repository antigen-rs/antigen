# Charter — the Route Arm (ROUTE: recruit the response to where it's needed)

*The cluster that directs scarce attention/effort to the right sites by a live gradient, not a stale
table. Includes the regulator primitive re-filed from `families/*` (it's machinery, not a disturbance).*

**Tier:** deep-future (chartered expedition). One half — the dread-escalation HOOK — is **design-now**.

---

## Organ-identity (what this IS)

Chemotaxis / gradient routing — the immune system recruits responders to a site by following a chemical
gradient, not a fixed address book. antigen's cognate: route scarce developer/reviewer attention to the
risky delta by a *live* signal (convergent dread, blast-radius, recency), never a stale CODEOWNERS-style
table (which would itself be the `RoutingTableStale` failure-class). This is the ROUTE stage of the
control loop (ADR-037) — and the `gradient-routing` primitive is regulator-machinery (re-filed here from
`families/*`; only its disturbance-half, `RoutingTableStale`, stays a stdlib family).

## The dreams in this cluster

- **`families/gradient-routing-chemokine-recruitment`** (the PRIMITIVE half, re-filed from families to
  LOOP-A per ADR-037) — antigen's ROUTE-stage mechanism: recruit by live gradient.
- **`dream/convergent-dread-sentinel-network`** — N independent `#[dread]` marks at one locus
  auto-escalate (chemotaxis: many weak signals converging = a strong recruit). **DESIGN-NOW SLICE: the HOOK
  into beta.2's dread/aura.** Do not paint `#[dread]` as a solitary marker — leave the cluster-key +
  convergence-count in the Finding schema (already there, ADR-039 §C) so a future sentinel-network can
  subscribe. This is the one piece of this charter that touches beta.2.
- **`dream/epidemiology-contact-tracing-outbreak-map`** — trace the exposed cohort (copy-paste / dep /
  near-neighbor of a found failure). Near-slice: syntactic copy-paste cohort.
- **`dream/reviewer-side-immune-lens`** — `cargo antigen review <diff>`; route the scarce reviewer
  attention to the risky delta. The active twin of the human-attention sensor.

## Dependencies (what unblocks this)

- **The dread/aura marks (SHIPPED, ADR-041) + the Finding schema's cluster-key (SHIPPED, ADR-039 §C)** —
  the convergent-dread sentinel subscribes to exactly these. The HOOK is design-now precisely so this
  charter starts from a real signal (don't foreclose it by shipping dread solitary).
- **The registry/herd substrate** — gradient-routing's content-addressed field IS the registry's
  substrate (the expansionist's collapse); so ROUTE and POPULATION share a mechanism.

## Could-combine-with

- The **registry/herd charter** — same gradient-field substrate, one distance-metric. Likely ONE
  expedition.
- The **effector arm** — route decides *where*, the effector decides *what*; they compose into the response.

## Buildability / effort scoping

- **The dread-escalation HOOK: NEAR-FREE NOW** (design-only this wave) — the Finding schema already carries
  `cluster-key` + `site`; a sentinel-network is a downstream subscriber that counts dread-marks per
  cluster-key. Don't build it; just don't foreclose it (the schema already doesn't).
- **`cargo antigen review <diff>`: MODERATE** — antigen already scans a workspace; scoping a scan to a diff
  + ranking by the risky-delta is a bounded CLI extension (pairs with the ADR-036 modular scanner).
- **The gradient-field: HARD** (shared with the registry charter) — the content-addressed routing substrate
  is a real distributed component.

## Invitation to deepen

Stigmergy — ants route by pheromone trails, no central coordinator. The deepest version of gradient-routing
is the same: no server, just a field every codebase reads and writes, where the convergence of independent
dread-marks (from different devs, different agents, different sessions) at one structural locus IS the
signal that says "look here." That convergence is the convergent-evidence thesis (the shipped family)
applied to *attention* instead of *defense*.

# Charter — the Runtime Afferent Arm (SENSE: the deployed organism)

*The cluster that closes antigen's control loop into a true cycle: the return path from the deployed
organism back to the dev-time germinal center. Identity-SETTLED (Tekgy): production IS antigen's identity.*

**Tier:** deep-future (chartered expedition); **identity = SETTLED YES** (a timing call, not an identity fork).

---

## Organ-identity (what this IS)

The **afferent lymphatic circuit** — antigen samples the deployed organism (production), drains the signal
back to dev, where the germinal center (the learning core) matures the response. It is the open-loop →
closed-loop upgrade (Wiener): antigen stops being a classifier and becomes a *closed-loop regulator*.
**The bounding principle is identity-defining: SAMPLING ≠ ACTING.** Antigen *drains* prod (samples
incidents, traffics the sample to dev) but does NOT act in prod (no paging, no auto-rollback) — antigen
EMITS, the dev decides the effector action (incl. wiring into their own CI/CD). This is what keeps antigen
a dev-tool, not an APM. Dev stays the germinal center; prod is the tissue antigen patrols by sampling.

## The dreams + sensors in this cluster

- **`dream/runtime-sensor-deployed-organism`** — what-fired; silent→loud at runtime. Identity SETTLED YES.
  Organ = `#[titer]` (live intensity, shipped ADR-024) + `#[culture]` (duration) — the monitor cognate.
- **`sensors/afferent-drainage-runtime-feedback-loop`** — the return path itself (dendritic sampling →
  afferent drainage → germinal-center maturation; all already named in the naturalist's framework).
- **`sensors/tissue-resident-memory-runtime-guard`** (T_RM) — where antigen forward-deploys a *frozen,
  dev-ratified* fingerprint as a runtime guard. The line is central-decision-vs-forward-deployed-frozen,
  not sampling-vs-acting.
- **`sensors/one-signal-four-scales-titer`** — the single signal-type (`#[titer]`) across the four scales
  (intra-pass → inter-deployment → runtime-resident → species). The afferent (in) + efferent (out) arms
  are the four-scale ladder seen in two directions.
- **`sensors/efferent-arm-matured-response-export`** — the outward arm: matured fingerprints exported
  (memory-cell graduation / long-lived-plasma-cell always-on-scan / circulating-antibody runtime-guard).
- **`dream/environment-threat-landscape-sensor`** — the WORLD that redefines failure (code innocent, world
  moved; RUSTSEC-against-static-code is the near-slice).
- **`dream/human-attention-inspection-substrate`** + **`dream/conversation-reasoning-sensor`** — the
  where-inspected / where-decided sensors (the near-slice: read camp's own `activity.jsonl`).

## Dependencies (what unblocks this)

- **The dial/Finding schema (SHIPPED) leaves a `runtime-resident` tier slot** above `named` — the cheap
  stub the expansionist flagged. The `(tell, loc, ts, sev)` inward interface is the minimal afferent input.
- **The learning core** consumes what this arm drains (the afferent loop's whole point is to feed
  maturation) — so this charter is most valuable *after* (or alongside) the learning-core charter.

## Could-combine-with

- The **learning-core charter** — the afferent arm is the input port; maturation is what it feeds. They are
  the two halves of one cycle.
- **`sensors/coordination-transcript-substrate`** (the agentic-coordination sensor; v0.3-near slice in the
  ROADMAP) — the most thesis-true dogfood: antigen sensing its own agentic-coordination failures (read
  camp's own `activity.jsonl`; detect an unratified multi-agent decision).

## Buildability / effort scoping

- **The afferent STUB is cheap** — a `runtime-resident` tier on the dial + the `(tell, loc, ts, sev)`
  inward interface is a small additive surface (the Finding schema already carries site+severity+timestamp).
- **The real drainage is MODERATE-to-HARD** — it needs a place prod telemetry lands and a transport to dev.
  But SAMPLING-not-ACTING keeps it bounded: no prod-side actuator, just a sample-and-traffic path. Antigen
  emits; the dev wires the ingestion (their CI/CD/telemetry — their choice).
- **T_RM forward-deploy is HARD** — running a frozen fingerprint in prod is a real runtime component; charter
  it last, gated on the afferent stub proving value.

## Invitation to deepen

The lymphatic network routing detail (which the dreamer flagged as a live frontier). The
inter-deployment loop made whole (afferent in + efferent out = the four-scale ladder both directions).
And the deepest: when antigen drains a prod incident, matures a fingerprint, and forward-deploys it as a
T_RM guard, the loop has closed end-to-end — a failure that bit production once cannot bite it silently
again. That is the whole promise, made literal.

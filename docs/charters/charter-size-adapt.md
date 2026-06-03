# Charter — Size / Adapt (SIZE: track a moving, evolving target)

*The cluster that handles failure-classes which EVOLVE — to evade antigen's own success, or to cluster at
process-phases. The arms race that is the whole reason adaptive immunity exists.*

**Tier:** deep-future (chartered expedition). The thinnest of the deep-future clusters.

---

## Organ-identity (what this IS)

The Red Queen — "it takes all the running you can do to keep in the same place." Failure-classes are not
static: as antigen gets good at catching a class, code (or an AI agent) evolves to dodge the fingerprint
rather than avoid the failure (Goodhart — the setpoint-corruption self-risk). Adaptive immunity exists
*precisely because* pathogens evolve; this cluster is antigen's adaptation machinery pointed at its own
obsolescence.

## The dreams in this cluster

- **`dream/adversarial-evasion-red-queen`** — failure-classes evolve to evade antigen's OWN success; the
  arms race; **maturation pointed at evasion** (the learning-core engine, with a moving target). The
  obsolete-vs-evaded discriminator (shared with forgetting) decides: did this class stop mattering
  (forget it) or did it learn to dodge the fingerprint (broaden it)?
- **`dream/circadian-process-rhythm`** — failure-classes that cluster at process-PHASES (release-eve,
  post-fix window). **Thinnest dream — maybe a recurrence refinement, not its own organ.** A candidate to
  fold into the recurrent-emergence family (ADR-024) rather than charter standalone.

## Dependencies (what unblocks this)

- **The learning core (maturation) is the prerequisite** — evasion-tracking IS maturation with an
  adversarial moving target; there is no "adapt" without the engine that generates. This charter is a
  *sequel* to the learning-core charter, not a parallel.
- **The afferent runtime arm** supplies the obsolete-vs-evaded discriminator (the runtime incident-signal
  that distinguishes a class that stopped firing from one being dodged).

## Could-combine-with

- The **learning-core charter** — strongly. Red-queen evasion is maturation's adversarial mode; they may be
  one expedition (the engine + its adversarial extension).
- The **feedback-homeostasis charter** — forgetting + evasion share the obsolete-vs-evaded discriminator.

## Buildability / effort scoping

- **Gated on the learning core + the afferent arm** — this is the *last* of the loop-stage charters to
  become buildable (it needs both the engine and the runtime signal). Effort: MODERATE *given* those
  prerequisites; impossible without them.
- **`circadian-process-rhythm` is CHEAP-or-fold** — if it's real, it's a phase-tag on the recurrence
  detector (ADR-024), not a new organ. Flag for the next dreamer: confirm it's distinct before chartering
  it standalone, else fold into recurrent-emergence.

## Invitation to deepen

The Goodhart self-risk is antigen's deepest structural vulnerability (`FingerprintGamedNotDefended`): it
gets WORSE as the optimizer (the AI agent antigen serves) gets stronger. The arms race is not optional —
it is the structural consequence of antigen succeeding. The flagship manuscript framing: *antigen vs the
very optimizers it serves.* The mitigation is already partly shipped (convergent-evidence: multiple
uncorrelated fingerprints per class resist gaming) — this charter is where that becomes an active,
maturing defense rather than a static one.

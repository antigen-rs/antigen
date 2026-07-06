# DRAFT — ADR-067 Amendment: The Capability Expansion Law

**Status:** DRAFT (for the ratification ceremony — draft → deconstruct → adversarial-annotate → research →
ratify). Supersede-not-erase amendment to ADR-067 (The Stroma). Derived from notebook
[017](017-the-0.7-organ-council-boyd-ruling.md) and the design arc that closed it.

---

## Context

ADR-067 established the stroma as the substrate everything senses on. The surrounding 0.6.x charters carried an
implicit posture — **"enrich, never replace"**: ship flat floors now, add stroma-enriched ceilings later, keep
the flat path beside the stroma indefinitely. That posture was calibrated to *protect shipped value*.

Two facts retire it:

1. **There is no shipped value to protect.** antigen has ~0 external crates.io consumers (the two known users
   are the authors). The risk "enrich, never replace" insures against does not exist yet.
2. **A permanent dual substrate violates the stroma's reason to exist.** By Ashby's Law of Requisite Variety,
   the regulating organ must hold at least the variety of everything it regulates — so the stroma must *absorb*
   all function, not sit beside a parallel flat path. Carrying two sources of truth is precisely the
   dual-source-of-truth drift antigen exists to catch (`v0.7-presents-the-self/launch.md`: "Half-migrated = the
   dual-source-of-truth drift antigen exists to catch — so it's full, or it's dishonest").

## Decision

Retire "enrich, never replace." Adopt **the Capability Expansion Law** — one uniform operation, iterated over
every capability in the build sequence `[stroma, frame, organ₁…organₙ, capstone]`:

> **BUILD** the capability → **ABSORB** every beneficiary onto it (re-point each existing consumer, validated
> against the existing test suite as the known-good baseline) → **VALIDATE.**
> *Full, or it's dishonest.* No capability ships leaving its beneficiaries on the old path.

Four clauses:

1. **Expansion-before-consumer** (invariant): the signal a feature needs is built into the substrate *first*;
   no consumer is ever built on non-stroma substrate. The stroma migration is not a one-time phase — it is the
   ABSORB beat, and it fires for *every* capability (the stroma's own migration is merely its first and largest
   instance, because the stroma's beneficiary-set is "everything that already exists").
2. **Full-refactor-as-we-go** (process): existing machinery is refactored to substrate-native *as each feature
   reaches it* — build the signal in, refactor beneficiaries to consume it, drop the old path. Not enriched,
   not deferred to a later consolidation.
3. **Revert-and-rebuild** (safety net): a refactor that breaks a release is rolled back (or branch-replaced),
   root-caused (substrate signal *or* refactor plan), and rebuilt clean. This replaces dual-substrate insurance.
4. **Trigger** (the anti-rot clause): this posture holds *while external adoption is negligible*. When real
   external consumers appear, re-open the protect-shipped-value tradeoff — a deprecation discipline may
   legitimately return then. The law is "don't protect value that doesn't exist," not "never protect value."

## Consequences

- **Single substrate** (Ashby-complete): no permanent two-substrate seam.
- **Less complexity, not more**: deletes the enrich/defer machinery and the dual source of truth.
- **Short-term churn risk** absorbed by revert-and-rebuild (cheap at ~0 external consumers).
- Each capability node's definition-of-done now *includes* its beneficiary-absorb + known-good validation.

## Supersedes

The "enrich, never replace" clauses in the 0.6.x charters and the `pre-stroma-staircase` ceiling framing
("ship flat floors in 0.6.x, enrich in 0.8"). Those floors, where still useful, are rebuilt substrate-native.

## Open questions for the ceremony

- Does clause 4's trigger need a concrete metric (download count? named external adopter?) or stay judgment?
- Interaction with semver: a full-refactor release that reverts — is the reverted version yanked, or superseded?

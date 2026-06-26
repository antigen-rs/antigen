# F-arc: affinity-score calibration — v0.6-vs-v0.7 gate (INWARD)

Tip: 86cea89 (0.6-dev sealed-ready). Read-only via `git show`, no sink mutation.

## The discriminating fact — SETTLED: NO live cold-start-exposed irreversible-disposal consumer in v0.6.

Call-graph of the affinity SCORE (the `Affinity` (recall,precision) rate-vector):

- **Producer**: `affinity::Affinity::measure()` (reads substrate) + `maturation::mature()` writes a `trajectory: Vec<Affinity>` and `LifeEvent::Scored(Affinity)` into the life-record.
- **Disposal pipeline** (the only path to the single irreversible action `CurationAction::Forget`):
  `score_trajectory()` → `adwin::detect(trajectory, δ)` → `adwin::fuse_channels(drift, silent, defended)` (== `discriminator::fused_classify`) → `ClassVerdict` → `curate::curate(verdict)` → `curate::apply(Forget)` → `LifeEvent::Retired`.
- **Forget gate**: `curate()` emits `Forget` ONLY for `ClassVerdict::Obsolete`; `Obsolete` is reachable ONLY from `(SilentStatus::Obsolete, defended=false)` AND a non-blind ADWIN axis. `is_auto_forgettable()` is `true` for `Obsolete` alone (`discriminator.rs:117`).

### Two structural guards already cover the cold-start affinity case:

1. **Cold-start trajectory IS UnderPowered → RouteToHuman.** A cold-start / un-queried class has a short affinity-trajectory (n≈0–8). `adwin::detect` returns `DriftVerdict::UnderPowered` as the DEFAULT at antigen's scale (ADR-065; adwin.rs:21,613). `fuse_channels` half-1: `UnderPowered ⇒ RouteToHuman` UNCONDITIONALLY — "even the single forgettable cell (shape-gone-undefended) must HOLD" (adwin.rs:846-849). So a cold-start biased score CANNOT reach Forget. It is caught by the SAME conservatism-JOIN as #47, on the rate axis.
2. **The disposal pipeline has ZERO binary callers on 86cea89.** `fused_classify` non-test callers: 0. `curate()`/`apply()` callers outside their own test mod: 0. `mature()` live callers: 0. The binary (`cargo-antigen/src/main.rs`) wires ONLY `propose::propose()` (main.rs:4072), which emits a `ProposeOutcome` (draft suggestion OR rejection) and NEVER auto-disposes/forgets/asserts (observe-don't-declare, ADR-044/048).

### The binary-wired affinity-adjacent path does NOT consume the score:
`self_tolerance::evaluate`/`promote_if_safe` (the B-gate, binary-wired) is a BINARY set-predicate `spare_clean` over the clean corpus (`ToleranceVerdict::Spared` vs `BindsCleanItem`), NOT a rate-threshold over the affinity SCORE. The score (rate) is read only by the not-yet-wired maturation engine.

## Verdict on captain's hypothesis

- **H1 (structural rhyme with #47): CONFIRMED — and sharpened.** Cold-start calibration risk IS "blindness-mislabeled-as-confidence drives an irreversible action," one axis over (rate axis vs drift/near-miss axis). BUT the rhyme is so exact that v0.6 ALREADY closed it: the affinity SCORE only reaches a disposal via ADWIN-over-the-trajectory, and a cold-start trajectory is `UnderPowered` BY CONSTRUCTION → the existing conservatism-JOIN catches it. The #47 fix is not "one axis over, unguarded" — it's the SAME guard, already covering this axis. Convergence finding: the rate axis was never a separate hole; it routes through the same UnderPowered spine.
- **H2 (A/B split): CONFIRMED as a clean conceptual boundary, but A is currently a NO-OP on 86cea89** — there is no live consumer for guard-A to protect. The cold-start guard the captain describes (mark un-queried affinity under-powered, refuse confident disposal) is ALREADY the behavior, structurally, via `detect→UnderPowered→RouteToHuman`. So:
  - **Guard A is not needed as new code today** (the guard exists). At most: a born-red ATK of the #47 family that PINS the cold-start→UnderPowered→RouteToHuman path so a future maturation-wiring can't regress it. That ATK is cheap and v0.6-spirit (it defends an existing invariant), but it is the ONLY do-now item, and even it defends a path with no live consumer yet.
  - **Guard B (calibration methodology: isotonic/Platt, sampling-bias correction, active-learning) is v0.7** and the boundary is clean: B is about making the SCORE a calibrated probability; v0.6 never treats the score as a probability — it's a PartialOrd ranking with NO Ord (anti-scalar shape, ADR-061). There is no v0.6 surface that interprets the score AS a probability, so there is nothing for calibration to correct yet.

## ANSWER to the release decision
DEFER the whole affinity-calibration axis to v0.7. The cold-start irreversible-disposal risk is real as a CLASS but has NO live v0.6 consumer, and the one path that COULD reach disposal is already guarded by the UnderPowered conservatism-JOIN (the #47 family, generalized). Folding calibration into v0.6 would be building guard-A ahead of a v0.7 consumer (the live curation loop) — the exact "don't build ahead on a pending design fork" anti-pattern. The ONE cheap, in-spirit, optional do-now: a born-red ATK pinning `cold-start-trajectory → UnderPowered → RouteToHuman` so the v0.7 maturation-wiring inherits a red tripwire. Not required for the tag.

## Waking notes (next session)
- Phase-8 void to explore: the `Scored{affinity, cluster_size}` RESERVE (adwin.rs:833) — the recall-rate is denominator-free, which is WHY recall-Drift+Dormant routes to human. That denominator absence is a structural proxy for a missing first principle: the score carries no power/sample-count, so it CANNOT self-report its own under-power except via trajectory LENGTH. Is trajectory-length a faithful proxy for sampling-power? If a class is queried often in a BIASED region, the trajectory is long (not UnderPowered) but the score is still cold-start-biased. THAT is the genuine residual — and it IS v0.7 (needs the cluster_size denominator). Pull this thread next.
- Open: does any v0.7 design doc already name the live curation loop's caller of `fused_classify`? If so, guard-A's ATK should be authored against that named seam.

## Phase-8 residual RESOLVED (the void's shape)
The trajectory is built ONLY inside one `mature()` call (maturation.rs:240-290), one `Scored` per accepted mutation round, all measured against the SAME handed `cluster`+`clean_corpus`. Consequences:
1. Trajectory-length ∝ maturation ROUNDS, not wild query-frequency. The "long trajectory from a biased region" worry I floated does not arise — a cold-start class cannot accumulate a long trajectory through biased querying. UnderPowered-by-length holds.
2. THE GENUINE RESIDUAL (the void): `measure()` ALWAYS reads the corpus it was handed. ADWIN watches the trajectory for drift; it is STRUCTURALLY BLIND to whether the `clean_corpus` itself is representative of real clean siblings. The math-researcher's "sampling-bias in un-queried regions → confidently-wrong" is REAL but it is a property of the CORPUS, not the SCORE or the trajectory. No v0.6 organ can see it, because no v0.6 organ models corpus-representativeness — `measure` takes the corpus as ground truth. This is exactly what guard-B (calibration + sampling-bias correction + active-learning to query un-sampled regions) would build, and it requires a first principle v0.6 does not have: a model of corpus COVERAGE / a power-count on the sample. THAT first principle (the `cluster_size`/coverage denominator, adwin.rs:833 RESERVE) is the preliminary unit guard-B needs. Confirmed v0.7.

So the clean boundary is even sharper than the captain's A/B: the affinity SCORE's cold-start risk has TWO sub-axes — (a) trajectory-blindness (already guarded by UnderPowered) and (b) corpus-unrepresentativeness (unguardable in v0.6 because no organ models coverage; needs the coverage-denominator first principle). (a) is closed; (b) is v0.7 and the score itself is anti-scalar-shaped (no Ord) precisely so it never masquerades as the calibrated probability (b) would require. v0.6's anti-scalar shape is the HONEST placeholder for the calibration it deliberately does not yet do.

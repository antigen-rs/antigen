# 020 — Deconstruction Council: DRAFT ADR-067-amendment & ADR-071

*Record of the ratification-prep council run 2026-07-06. Scout (stroma real-state)
+ 4 deconstructors (aristotle + adversarial on each of the two drafts). Feeds the
live human ratification ceremony. Drafts under review: notebook 018 (ADR-067
amendment — the Capability Expansion Law) and notebook 019 (ADR-071 — organ-hood
+ loop-as-diagnostic-taxonomy). Council mechanism: direct-spawn (the Workflow
parser rejected the orchestration script; the council ran identically via
captain-held batons).*

---

## 0. Ground truth (stroma scout)

The stroma-closure claim is **TRUE and non-stale.** The `antigen-stroma` frame
layer is fully built and real (read/ node/ base/ constitute/ db.rs write.rs
fidelity.rs deferred.rs). The **one unbuilt node** is the reachability CLOSURE:

- `read/query.rs:25/35/47/61` — `reachable_from` / `field_at` / `provenance_of` /
  `blast_from` are `todo!()` (my earlier grep that found "zero todo!()" was
  wrong — path/encoding artifact; the scout read the files).
- The `#[salsa::tracked] fn reachability` using `ascent_run!` over `EdgeFacts`
  (4 semirings + SCC-condensation for blast) **does not exist** — only a
  pseudocode comment at `base/facts.rs:123-145`.
- `scip.rs:36` `ingest_scip()` is `todo!()`; `ResolvedEdge` empty.

Verdict: "the stroma-closure is the ONE node that gates everything" holds.
ADR-071's build-order rests on a real unbuilt node. ~0 external crates.io
consumers of the closure work.

---

## 1. ADR-067 amendment — verdict: **RATIFY-WITH-CHANGES**

Core thesis is **sound** (full-refactor-as-we-go while there's nothing to
protect; expansion-before-consumer forever — already latent in ADR-067 +
the 017 organ-hood law, so partly recognition-not-design). The failure is
**operationalizability, not principle.** The two lenses converge hard.

### Convergent BLOCKERS (two lenses, same spot)

**B1 — The trigger smuggles an OUTCOME; name its PREDICATE.**
(arist F2 + adv Break-2.) "This holds while external adoption is negligible" is
justified on *today's state* (~0 consumers) rather than encoding a queryable
condition. "Negligible" must resolve to a **predicate**, not a scalar/judgment:
*no crate outside the workspace has published a version with `antigen` in its
`Cargo.toml`* (a reverse-dep fact). Adversarial adds the **detection mechanism +
tripwire**: `cargo antigen audit` runs the crates.io reverse-dep check as a
pre-refactor gate in ABSORB mode; if it fires, the law must be re-opened via
amendment before ABSORB continues. An invariant with an unspecified
firing-condition "is a vibe with an ADR number."

**B2 — ABSORB-done against the old suite is CIRCULAR; require a born-red witness.**
(arist F4 + adv Break-3b.) The whole point of ABSORB is capability *expansion* —
the new true-positives are, by construction, cases the old suite never asserted.
"Old suite green" proves *no regression on the old capability*, never *the new
capability is correct*. This is antigen's own negative-control / born-red law
turned inward. **ABSORB-done = (old suite green: no regression) AND (a born-red
test that FAILS on the old flat path and PASSES only on the stroma path).**
"An amendment to the immune system that lets unwitnessed expansion pass is the
system failing its own dogfood at the ADR layer."

### Fixable (ratify with edits)

**F-1 — Cut the bundle: invariant vs process.** (arist F1.) BUILD→ABSORB→VALIDATE
is two durability-classes in one flat list. Clause 1 (*expansion-before-consumer*)
is a **permanent invariant, no trigger** — it must outlive the adoption gate.
Clauses 2+3+4 (full-refactor tempo + revert net + trigger) are a **scoped
process.** Separate them visibly so the invariant doesn't inherit the trigger's
expiry.

**F-2 — Scope the retirement of "enrich, never replace"; it protects two classes
the external trigger is blind to.** (arist F3 + adv Break-4.) Retiring enrich is
right for code-only refactors at ~0 consumers, but enrich was also guarding:
(a) **internal path-dep consumers** — tambear + the dogfood suite (invisible to
an *external*-adoption trigger, real to the breakage); and (b) **persisted
formats** — on-disk `ScanReport`/`schema_version` records; revert recovers code,
**not recorded data**. → Two guards, not one: an external adoption-gated guard
(drifts) beneath a **permanent internal/persisted-format guard** (never expires;
enrich-then-retire preserved for serialized-type changes).

**F-3 — Published-artifact supersession.** (adv Break-1.) Revert is cheap for the
working tree, not for a published crate (`0.4.0-beta.1` is live; yank ≠ delete).
"Rebuild" for a published release means a **new semver version** that supersedes
the broken one, not a rollback. Clause 3 must distinguish the two.

**F-4 — Define the terms + forbid absorb-thrash.** (adv Break-3.) "Beneficiary"
= any call-site / type-import / test-fixture exercising the replaced path,
identified by `cargo antigen scan` at BUILD-completion. "ABSORB-done" = the B2
born-red gate. **Absorb-thrash**: two ABSORBs on overlapping beneficiary sets can
satisfy neither; forbid parallel ABSORB on overlapping sets — strict serial per
beneficiary.

**F-5 — Wording: namespace + category.** (arist F5/F6.) "Enrich, never replace"
(the CHARTER posture — internal build strategy; lives only in 0.6.x charters +
two ATK files, never in decisions.md) must be disambiguated from the **ratified**
"enrichment, not gating" ADR (user-facing API layering — a *different axis*).
And reframe "ADR supersedes charter" as "ratifies an invariant that *overrides*
the non-ADR posture" (keeps ADR-lineage clean: ADRs supersede ADRs).

---

## 2. ADR-071 — verdict: **RATIFY-WITH-CHANGES, via ONE reframe**

### THE ONE MOVE (arist-071, Phase-5)

Decisions 1, 2, 3 are **not three decisions** — they are one invariant:

> **The control-loop INDEXES capabilities; it never INDIVIDUATES them.** A
> capability (organ *or* self-regulation) is individuated in BUILD-space
> (distinct cost + distinct blast-if-absent), witnessed by a distinct signal OR
> a distinct authorization-gate, consuming a distinct beneficiary-set. Its
> loop-stage is its **coordinate** (where it lives), not its **boundary** (what
> makes it one).

Adopt that as the single Decision; derive the rest:
- Decision 2 = the invariant applied to organs (don't cut organs along the loop).
- Decision 3 = the invariant applied to self-regulation (self-reg items are
  *indexed-by*, not *constituted-by*, stages) — so D2 and D3 stop being opposed
  ("not a WBS" / "a build-track") and become **one law, two instances.**
- The split-test = the invariant's contrapositive at finer grain.
- Decision 5 = the invariant's completeness-audit (enumerate every coordinate,
  even empty/on_hold ones).

This single reframe **discharges the two blockers below at once.**

### BLOCKERS

**B3 — Shared-substrate re-introduces loop-as-WBS / ROUTE has no arbitration.**
(arist-071 F5 + adv-071 Attack-3 — the SAME finding from two directions, the
sharpest on the board.) The organ track and control-plane track "share substrate
at ROUTE/ACT/FEEDBACK (build once, both consume)." But at those three stages the
organ track *is* being partly individuated by loop-stage (the leak D2 exists to
prevent), and "build once, both consume" gives builders **no ownership rule** —
and by ADR-037's own boundary-discriminator, ROUTE-as-control-plane
(antigen's self-attention) and ROUTE-as-organ (user-code workflow) are two
distinct fix-shapes on one artifact. **Fix (merges both lenses):** a shared node
is ONE build unit (individuated in build-space) occupying a coordinate consumed
by both tracks — *sharing a coordinate is not being individuated by it*; and the
**organ track owns the external interface contract, the control-plane hooks on
top** (not a co-owner). The INDEX-NOT-INDIVIDUATE invariant makes this natural.

**B4 — Beneficiary-absorb conjunct is the wrong criterion (two teeth).**
(arist-071 F2.) (i) **Cross-draft collision**: 019-conjunct-(b) says the capstone
has no beneficiary-absorb (correctly → not an organ), but 018-clause-1 lists the
capstone as a capability that MUST absorb. The two drafts disagree about the
capstone and go to the same ceremony. (ii) The conjunct **excludes greenfield
organs**: self/non-self (`1a`) re-points no existing consumer, so "an existing
consumer to re-point" would deny organ-hood to the canonical first organ. **Fix:**
conjunct (b) = "distinct beneficiary-SET (who consumes it)", NOT "an existing
consumer to re-point" — the latter is 018's ABSORB *lifecycle duty*, a different
law. Organ-hood needs distinct-beneficiaries (individuation); ABSORB is a separate
obligation. (This cleanly *connects* the two ADRs.)

**B5 — Individuation-law and split-test are fused but use different variable-sets.**
(arist-071 F3.) The forward organ-hood test uses {build, absorb, signal-OR-gate};
the backward split-test uses {gate, signal, absorb, belongs-iff} — "belongs-iff"
appears *only* in the split-test. A splitting rule should be the contrapositive
of the individuation rule (or an explicitly distinct REFINEMENT-LAW with its own
justification). Don't ship them fused in one sentence.

### Fixable (ratify with edits)

- **Re-warrant Decision 2 on build-space ≠ fault-space; demote the 9808 citation.**
  (arist-071 F4.) decisions.md:9808 is about the *user-code disturbance* genus
  being empty at ROUTE/FEEDBACK — a claim about the STDLIB taxonomy (D-vs-C), NOT
  about build-decomposition. The real reason the loop isn't the organ-WBS is the
  non-injective map between build-space and fault-space. Citing 9808 borrows
  ADR-037's authority for a claim it doesn't make.
- **"Distinct BUILD" is undefined and doing all the work** (arist-071 F1;
  strong-rec) → reframe as one criterion (distinct build) + two witness-classes
  in OR. Subsumes adv-071 Attack-1 (Fs failing the OR-gate).
- **Cite the two OR-witnesses** (arist-071 F7): effector-tiers = gate-only witness;
  self/non-self = signal-only witness → closes 019's open-question 1
  (authorization-gate is one of two OR-witnesses, not a sole axis).
- **Soften "known-complete"** (arist-071 F8): complete only w.r.t. ADR-037's
  *open* six-stage frame (ADR-037 itself refuses to claim stage-completeness).
- **Honest-scope: the map is a DESIGN decomposition, not a parallel BUILD one,
  until the closure ships** (adv-071 Attack-4). Only closure-independent nodes
  (Fs substrate, Rh schema-reservation, 1a self/non-self, marker-sovereignty
  ratify) advance concurrently with the closure; the rest serialize behind it.
- **`Sr` (reputation-update) horizon placement is unverified** (adv-071 Attack-2):
  it's placed 0.7.x on `provenance_of`, a `todo!()` lattice-JOIN semiring that may
  only land with the hard SCC tier → tag as a ceremony question (0.7.x vs on_hold
  pending closure-tier ordering).

### Emergent — the Phase-8 gold (arist-071 F9, sharpened by addendum)

Not "name one candidate" — a **theorem.** ADR-037 (9832-9838) names *exactly four*
open stages: **observability, controllability, delay/latency, stability-margin.**
Compose with Decision 3 (the loop indexes one self-regulation build-item per stage)
+ Ashby (V(C)≥V(D); antigen regulates *itself*): each of the four control-theory
properties predicts a **specific unbuilt self-regulation organ**, structurally
guaranteed by antigen's own self-model — the ADR-007 anti-YAGNI law turned inward.
The four as candidate v0.8+ control-plane organs:

- **OBSERVABILITY** — can antigen reconstruct its own loop STATE from what it logs?
  (distinct from FEEDBACK-legibility/SCRAM, which logs *events* — you can log
  everything and still not be observable).
- **CONTROLLABILITY** — can antigen MOVE its own setpoint, or only watch it drift?
  (`#[autoimmune]` setpoint-failure is the *disturbance*; the correction capability
  is unbuilt).
- **DELAY/LATENCY** — does antigen's own feedback arrive in time to damp its own
  cascade? (the SCRAM rail exists; whether it fires before irreversibility is a
  delay-margin nothing measures).
- **STABILITY-MARGIN** — how close is antigen's own loop to oscillating?

**Name all four in the on_hold map** ("candidate control-plane organs, per ADR-037's
open stage-edge; structurally-guaranteed by Ashby on antigen's self-regulation;
unbuilt this horizon"). Not a ratification blocker (spawned discoveries) — but
naming only one (or none) makes Decision 5 assert a completeness the project's own
self-model contradicts: the exact confident-completeness-on-an-open-premise antigen
exists to catch. (Thread seated at
`garden/2026-07/2026-07-06-the-coordinate-indexes-it-never-constitutes.md`.)

---

## 3. Emergent items (charters / homing / ceremony)

1. **Capstone cross-draft collision (B4-i)** — 018 and 019 disagree on whether the
   capstone absorbs. MUST be reconciled at the joint ceremony; the two ADRs must
   agree before either ratifies.
2. **Born-red ABSORB-witness discipline (B2)** — itself a test-class; homes to the
   testing-platform node (v076) / test-architect registry.
3. **crates.io reverse-dep gate in `cargo antigen audit` ABSORB-mode (B1)** — a
   concrete tooling build item; needs a home (audit/effector layer or new charter).
4. **Four candidate control-plane organs (071-F9)** — observability,
   controllability, delay/latency, stability-margin; structurally-guaranteed by
   Ashby on antigen's self-regulation; home to on_hold in the v07 map (v0.8+).
5. **Closure-tier ordering** — which semiring lands when (`reachable_from` vs
   `provenance_of` vs `blast_from`+SCC); `Sr`'s horizon (and others') depends on it.

---

## 4. The decisions the ceremony must make

**ADR-067:** (a) Is the trigger-predicate the crates.io reverse-dep check? (b) Does
ABSORB-done require the born-red witness? If yes to both, the remaining findings
are wording/scope.

**ADR-071:** (a) Adopt INDEX-NOT-INDIVIDUATE as the single Decision (dissolving
D2/D3 + the shared-substrate blocker at once)? (b) Resolve the capstone collision:
does the capstone absorb (018) or not (019)? (c) Fold the control-plane track in as
a *section* of this ADR, not a separate ADR (F6: it's the same invariant).

Neither draft is ratify-as-is; both are ratify-with-changes. No structural void in
either — every blocker is wording/scope surgery.

# 022 — The Two-Winged Body: antigen's derived stroma-shape (the spine)

*The derived structural prior for antigen, from a first-principles derivation council
(aristotle F1–F18 + adversarial + expansionist) and a long co-design with Tekgy,
2026-07-07/08. This is **the spine the idea-expansion homes onto** — every mined idea
maps to a wing / axis / `Op` / loop-position. Held as a **PRIOR** (mixed DESIGN+CODE tier,
will-update at closure-ship + at the falsifiers in §9), not a proof — which is itself the
[A5 discipline](021-capability-expansion-and-the-afferent-organ-hierarchy.md) turned on the
shape. Full derivation detail lives in the agents' garden notes
(`~/.claude/garden/2026-07/aristotle-stroma-semiring-generator-campsite.md`,
`…rhyme-antigen-semirings-are-tambear-folds-over-a-graph-functor.md`).*

---

## 0. The one-paragraph shape

**Antigen is a control loop with two asymmetric wings.** An **AFFERENT sensor wing**
(read code, monotone, no inverse) coupled through a **HINGE** (`COMPARE`) to an **EFFERENT
actuator wing** (suppress/tolerate, bounded, *deliberately not* a ring). The apparent
"four semirings" are **not the axes** — they are the `Op`-dial (the value you fold with);
the real structural frame is **`access × structure` + an observer**, which is *the same
frame as tambear's* `accumulate/gather` 2×2 + Tam. Antigen currently occupies the **fold
(descriptive) half**; the **unfold (generative) half is the frontier** — and the planned
**intent-substrate is the keystone** that opens it. The whole body **refuses one operation
— true cancellation** — because a real additive inverse (permanent deletion of a
self-signal) *is* the autoimmune pathology.

---

## 1. The re-layering (why "not a 2×2" was a mis-layer)

The derivation first read antigen as "3 imbalanced axes, not a 2×2" — because it put the
**four semirings** where a structural axis goes. They belong one layer down. Correct
layering:

- **The 2×2 frame** = **`access` (direction) × `structure` (fold/unfold)** + **observer**.
  Identical to tambear's `gather/scatter × fold/unfold` + Tam.
  - **`access` = direction = chirality.** `reachable_from` = "what reaches me" = *gather /
    toward-me*; `field_at`/`blast_from` = "what I reach" = *scatter / away-me*. Realized as
    a **graph transpose** (G↔Gᵀ) — orientation-reversing, a true reflection/**involution**
    (flip twice = identity). *(tambear's is an adjunction `Σ⊣f*⊣Π`; antigen's is a clean
    involution — same axis, different symmetry-type.)*
  - **`structure` = fold/unfold = `Is`/`Should`.** `Is` = fold (reduce the actual — "what
    happened"). `Should` = unfold (generate toward the target — "what should"). **Antigen
    holds only the fold pole today** (see §4).
  - **observer = Tam = the frame-exit** (§5). Antigen's is **rank-1** (raises one dual, the
    resolution/provenance tier) where Tam is many-dual.
- **The `Op`-dial** (below the frame) = the four semirings: **detection (boolean) ·
  conductance (tropical-min) · provenance (lattice-JOIN) · blast (counting)** — the value
  you fold with. `provenance`'s lattice-JOIN is *also* the tier-semilattice's algebra (§5,
  the observer's own law), which is why it "abstained from traversal." The **tropical-min**
  semiring is **bit-for-bit tambear's min-plus, independently derived** — Tier-3 convergent
  evidence, not rhyme.
- **`cyclic-safety`** (below, a compile-gate) = the `const IDEMPOTENT` wall + SCC-
  condensation the counting semiring forces = **tambear's totality-firewall** ("partiality
  enters through one door"); the `CondensedGraph` type-state == tambear's `fix`-contract.

## 2. The AFFERENT wing (sensor) — CODE+DESIGN tier

Monotone, inverse-free **by construction**: a reachability *fold* over a monotone-growing
fact set; the tier law is `max`-only lift-only (`tier.rs:104-112`, CODE-witnessed). You
cannot un-reach or un-corroborate. Occupies **fold × {gather, scatter}** — both access
directions, one structure pole. Acyclic DAG → **cannot oscillate**.

## 3. The EFFERENT wing (actuator) — DESIGN tier (chartered-not-built)

**The ring-hypothesis broke, and the break is the thesis.** The efferent primitives
(`#[anergy]`, `#[immunosuppress]`, tolerance, Treg) are **NOT ring-negation** — they are
**bounded** (parse-time `duration_cap`/`until`), **reversible** ("quiet-not-gone" —
`#[anergy]` auto-re-engages, `#[immunosuppress]` *expires and self-presents*), and
**governed** (signed + rationale'd + loudness∝risk). So the efferent algebra is a
**bounded-reversible-governed suppression — a decaying MEET-lattice, not a ring.** *A true
additive inverse — permanent deletion of a self-signal — is literally what autoimmunity is*
(`decisions.md:5538`, "collapsing the gradient IS the autoimmune pathology"). The historical
receipt: **`#[immune]` was the ring** ("this failure-class is cancelled/gone here" = an
*absence*-claim = a negation), and antigen **killed it in v0.0.1** → `#[defended_by]` (a
*presence*-witness). The one time antigen built the additive inverse, it deleted it. *(Open
honesty-check: confirm the recorded `#[immune]` kill-rationale reads as "can't claim
absence," not only the dangling-undefined-class bug — that's witnessed-vs-inferred.)*

**Its shape is a clean 2×2** (unlike the afferent stack): ADR-053's tolerance grid,
`{central, peripheral} × {prevent, delete}` (only central-delete = negative-selection built).
**The two wings have DIFFERENT SHAPES** — sensor = imbalanced fold-stack, actuator =
balanced tolerance-2×2. The body is *not* mirror-symmetric.

**Derived law:** *observer-dimensionality is a function of the wing's cyclicity.* Acyclic
afferent DAG → **rank-1** monotone observer (the tier-semilattice). Cyclic efferent loop →
**4-dim** Ashby control-observer (the F9 quartet: observability · controllability · delay ·
stability-margin — the last has *no afferent analog* because the DAG can't ring). The F9
four aren't an arbitrary list — they are *the efferent wing's control-observer, and they
exist because the efferent side has feedback.*

## 4. The HINGE (`COMPARE`) — and the one legitimate ring

Sensor lives in `Is`, actuator serves `Should`; the coupling is **`COMPARE = Should − Is`**
= the control-theory **error** = the setpoint gap. **The ring — the subtraction forbidden
everywhere else — appears exactly once, here, at the hinge.** It is permitted because it is
a **comparison-subtraction** (measures a gap → a number) not a **deletion-subtraction**
(annihilates a fact). Same operator `−`, opposite intent: `COMPARE` measures distance to
setpoint; `#[immune]` tried to zero a danger. Antigen permits the one, killed the other.
So aristotle's "missing hinge-transducer" is **not a new structure — it is the polarity
flip evaluated as a difference.**

**The keystone: the intent-substrate = the load-bearing `Should`.** Today `Should` is thin
(a declared fact you fold over), so `Should − Is` is nearly empty. The planned
**intent-substrate** makes `Should` a real structure, which does **four jobs at once**:
(1) makes the hinge-ring *bear load* (error becomes a genuine conformance-gap); (2) is the
**real setpoint** — the reference the efferent wing regulates toward (without it, antigen is
a sensor with a twitch, not a loop); (3) is the **seed `propose` unfolds from** (the
generative pole); (4) so intent-substrate + conformance-monitor + `propose` are **one move
— completing the `Should` pole.** *The shape-derivation (pure algebra) and the roadmap
(planned intent-substrate) converged on the same keystone from opposite ends.*

## 5. The symmetry-group frame + the frame-exit

The generator is **one fold on one frame, acted on by a symmetry group + a frame-exit** —
*duals of the things ON the semiring, not an inverse OF it*:
- **reflection / chirality** = the access-transpose (G↔Gᵀ), an involution;
- **polarity flip** = `Is`↔`Should` (ADR-069 `POLARITY`);
- **rotation** = `perspective` (which edge-kind lens);
- **frame-exit** = the observer / Fock-raise — the *one operation that exits the frame to
  observe it*, which is *why* it raises a dimension (exiting a frame = adding one).

**Two species of frame-exit** (from the chirality↔climb rhyme): a **reversible involution**
(reflection — walk back through the mirror) and an **irreversible monotone climb** (the
observer — one-way, because it carries the non-cancel at the tier-dimension). The
**Flatland lens**: a higher-D observer can perform an in-frame-impossible flip — *but*
antigen's lift is **irreversible** (B2′: no write-down), so it *can't set the flatlander
back through the same door.* That is **why there are two arms**: you rise on the afferent
(can't descend), so the only way back to reality is to *act* — the efferent. A reversible
geometry wouldn't need two arms; antigen's one-way lift forces them.

## 6. B2′ is the algebra-firewall (deeper than ratified)

`B2′` (ratified 2026-07-07) is the structural firewall keeping the **two incompatible
algebras** (monotone-accumulate + cancelling-suppression) from forming an **oscillator** (an
accumulate feeding a canceller feeding back = a sign-flipped loop = ringing). Autoimmunity /
cytokine-storm / loop-oscillation are the *same* failure: the efferent cancel mis-coupled
into the afferent fixpoint. B2′ decouples them; the **stability-margin** organ watches the
one place they legitimately touch (the ADR-037 loop). *We ratified the firewall without
knowing it firewalls two algebras.*

## 7. Three versions of one structure

antigen (code) · camp (relationships) · tambear (numeric arrays) are **convergent
instantiations of one structure** — `accumulate + gather + observer` over different
substrates, *and* the reasoning that builds each (dream→…→document) is the same triple
oscillating. **antigen built the descriptive (fold/sense) half; tambear built the generative
(unfold/compute) half** — and each is now reaching into the other's half (antigen via
`propose`/intent; tambear via observation). The `access × structure` frame + observer is
shared; the difference is *which half each has built.*

## 8. Backward-audit — gaps in ratified ADR-071 (for a future amendment)

- **B4** ("open-ended latent tree to the fixpoint") over-reaches a **rank-1, tier-bounded**
  observer (`ResolutionTier` has 3 values → tree depth ~2). Needs an **unbounded
  organ-output tier (ℕ)** distinct from `ResolutionTier`, or B4 should admit rank-1-shallow.
- **B2′** is **process-enforced, not type-enforced** — nothing prevents an organ writing at
  the tier it reads. Should be a compile-time **type wall** (a signal-algebra build
  requirement).

## 9. Falsifier-watchlist (live, cheap, gated on unbuilt organs)

1. **SENSE-charter** → does it reach for an afferent *count* (`blast_to` = exposure/attack-
   surface cardinality) or just boolean afferent? Count → Axis-B is a true 2-pole mirror;
   boolean → 1.5-pole (finding).
2. **signal-algebra ships** → does its write-higher check **reuse** `ResolutionTier`'s
   ordering (one-law confirmed CODE-tier) or invent a parallel tier (it was a rhyme →
   update)?
3. **mitigation-cancellation organ chartered** → does `field_at`/conductance stay
   tropical-min, or move to **signed-flow**? Signed → the ring leaks into the afferent side,
   B2′ softens.
4. **autoimmunity-pruner (peripheral-delete) built** → true-delete (ring-negation) or
   bounded-reversible mask? True-delete → the "quiet-not-gone" claim is wrong for that
   quadrant → update.
5. **`propose` matures** → the **unfold pole appears** (antigen leaves the fold-only half);
   watch whether it unfolds *from the intent-substrate* (keystone confirmed).

## 10. What this is FOR (the expansion spine)

Every mined idea, charter, ADR, and notebook homes onto: **which wing** (afferent-sensor /
efferent-actuator / the hinge), **which axis** (`access` / `structure`), **which `Op`**
(detection/conductance/provenance/blast), **which loop-position** (SENSE/COMPARE/ROUTE/ACT/
FEEDBACK/SIZE), and **which observer** (rank-1 tier vs the 4-dim control quartet). The empty
cells name the gap-charters; the unfold half names the growth. *Not a change of direction —
a lens that expands the understanding at every spot and shows where the atoms are missing.*

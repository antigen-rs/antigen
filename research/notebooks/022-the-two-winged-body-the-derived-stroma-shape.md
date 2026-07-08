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
**bounded-reversible-governed suppression — a decaying MEET-lattice, not a ring.** *(The
three legs are co-equal: **governed** — ADR-023 ships `#[immunosuppress]` as **Loud**,
loudness∝risk a first-class audit discipline — is NOT folded under "quiet-not-gone";
"quiet-not-gone" is shorthand for all three legs, not a claim that suppression is silent.)* *A true
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
- **rotation** = `perspective` (which edge-kind lens) — and **`access` itself factors**:
  `access = (rotation: which edge-kind lens) × (reflection: gather/scatter direction)`, with
  *each lens carrying its own transpose-involution*. So `perspective` and the direction-
  transpose are **orthogonal generators** of the access-frame, not competing framings —
  which sharpens §9 falsifier-1 to *"does the count survive on **every** lens, or only the
  call-graph?"*;
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

## 11. The math-witness — three co-pending render patents PROVE the design-tier priors

*Added 2026-07-08. The 022 shape was DERIVED (multi-lens council priors). Pith/tambear's
render-math patents — the same "lift" family that runs across every sibling — turn out to
formalize antigen's stroma-shape as **theorems**. This is the strongest possible witness: an
independent field, built for GPU rendering, proves the algebra we posited for immunity. The
priors became theorems. Below, each 022 claim ↔ its patent theorem.*

### 11.1 Liftability ⇒ the afferent wing is exactly the liftable class

The **Liftability Framework** (Theorem 16.1): an effect is *liftable* iff it is an additive
per-particle fold `E = F(Σᵢ f(pᵢ))` — a monotone accumulate over a commutative-additive
kernel. **That IS the afferent wing** (§2): the accumulate-semiring, catamorphic fold,
rank-1 observer. The liftable class and the afferent-sensor class are the same set.

### 11.2 The Operator Boundary (Theorem 22.1) ⇒ B2′, PROVEN

**Theorem 22.1**: *an effect requires non-additive (non-commutative) accumulation iff it is
order-dependent.* Proof: additive accumulation is permutation-invariant (`Σ` commutes), so
any order-dependent effect (which particle is in front) is unreachable by any additive
operator. **This is B2′** (§6, §8): the afferent monotone-accumulate algebra CANNOT compute
the order-dependent / cancelling efferent decision — not by discipline, by **algebra**. The
firewall we ratified process-side (§8 gap: "should be a type wall") is a *proven* boundary,
not a convention. The signal-algebra expedition can cite the theorem as its type-wall's
justification.

### 11.3 The depth-kernel ⇒ the HINGE, and λ is the dial between the wings

The **exponential depth kernel** (Nadaraya-Watson estimator, `Numerator/Denominator` with
per-pixel `z_min` reference) is COMPARE = Should − Is (§4), made precise:

- **λ is the temperature dial connecting the two wings.** `λ=0` → pure additive accumulate
  (afferent / volumetric / the density KDE). `λ→∞` → the **MIN operator** (nearest-surface
  selection = tropical-min = the *route/conductance* semiring). **The four semirings of §1
  are NOT independent axes — they are λ-limits of ONE family, joined by log-sum-exp**
  (`softmin_λ → min` as `λ→∞`; the standard tropical-limit homomorphism). This upgrades
  "semirings = the `Op`-dial" from an assertion to a *mechanism*: the dial is a temperature,
  and log-sum-exp is the semiring morphism. `(+,×)` and `(min,+)` are the two ends of one
  thermometer. **[DESIGN-TIER, not code — flagged by the 2026-07-08 code-lens sweep.]** In the
  current source, `field_at` and all four semirings are `todo!()`/comment-only
  (`antigen-stroma/src/read/query.rs`, `.../base/facts.rs:131-146`); the afferent fold that
  *is* built (`scan/walk.rs:128`, `read/tier.rs:104 corroborate`) is a plain monotone
  accumulate, not yet a `fold_λ`. So "the semirings are one λ-family" is a **falsifiable
  prior, not a code fact**: it holds iff a *single* `fold_λ` recovers the detection/blast count
  at `λ=0` AND `field_at`/conductance-min at `λ→∞` bit-for-bit (with `O(exp(-λΔ))` deformation
  on a λ-sweep). Today those are two separate reducers ⇒ the honest verdict is **"rhyme until a
  unifying `fold_λ` is built"**; the test is the *acceptance criterion* for building it (§9
  falsifier #3). The signal-algebra expedition owns proving or refuting it.
- **`z_min`-subtraction-before-exponentiation = COMPARE is a comparison-subtraction against a
  reference/setpoint** — "measure relative to the baseline," never a deletion. Exactly the
  §4 claim that the hinge is *the one legitimate ring* (a comparison-subtraction, not a
  deletion-subtraction). The reference `z_min` is the **setpoint** — i.e. the
  intent-substrate keystone (§4): the hinge is meaningless without the reference it subtracts
  against, which is *why* intent-substrate gates the efferent build.
- **`λ<0` flips the reference from `z_min` to `z_max` = the POLARITY flip** (Is/Should, §5)
  as a literal sign-of-λ, the reference-pole flipping to match. Polarity is the sign of the
  hinge's temperature. (Atmospheric-inversion in the patent; Should-vs-Is in antigen.)
- **z-buffer is not a separate algorithm — it is the `λ→∞` limit of one family** (patent
  Corollary). Rhymes with ADR-071's "the loop is edges, not nodes": the efferent selection
  isn't a different *kind* of thing bolted on, it is the afferent accumulate *at a different
  temperature*. Two wings, one thermometer.

### 11.4 Wave-optics ⇒ the INVERSE exists, it OSCILLATES, and the firewall is a channel-split

Substituting the **oscillating (complex Helmholtz) kernel** for the positive-definite
(Gaussian) one gives the accumulate-semiring a true **additive inverse** (phase → destructive
interference → cancellation). This is the sharpest witness of all:

- **inverse ⇒ oscillation, PROVEN.** The moment the afferent accumulate gains a real
  subtraction, it *interferes* (Airy fringes, diffraction). This is precisely the B2′
  oscillator-hazard (§6: "accumulate feeding a canceller feeding back = ringing"). Wave
  optics is what "the ring leaked into the afferent side" (§9 falsifier #3) *looks like* —
  and antigen's whole point is to NOT be an interferometer.
- **the cure is a channel-split, and it's the SAME architecture.** The complex field lives in
  **separate Re/Im render targets** (MRT), accumulated additively but *independently*, and
  the observer reads only the **modulus-squared** (`Re² + Im²`, non-negative). **That MRT
  split IS the B2′ firewall, physical**: you *may* have the Should−Is inverse — but only in a
  **separate channel** whose afferent-observable projection is non-negative (monotone).
  Complex where the observer can't see; monotone where they can — the exact "complex inside,
  simple at the surface" posture. The efferent (signed, cancelling) computation is legal
  *because it is a different render target*, not a different value in the same accumulator.
  This tells the signal-algebra expedition its type: **the efferent channel is a distinct
  output target, not a signed value in the afferent semiring.**
- **Fresnel dual-boundary = the self-aligning seam = recognition-not-design.** `F = D²/(λL)`
  governs BOTH the physical regime (near/far field) AND the computational regime
  (compact/screen-scale splats): the boundary where the *semantics* change coincides with the
  boundary where the *algebra* changes. Antigen's B4 is the same self-aligning seam — the
  tier where the *vocabulary* changes is the tier where the *type-wall* falls (§8). A dual
  boundary is never imposed; it is *recognized* where two independent things happen to
  coincide (ADR-006).

### 11.5 The Fock boundary ⇒ the fixed-organ latent tree vs the variable-population learn-core

Liftability's **Fock boundary**: effects whose *particle count itself varies* (population-
dependent, variable-N) are **non-liftable** — no fixed additive fold reaches them. Map:

- The **latent-organ tree** (ADR-071 B4: organs-sensing-from-organs to a fixpoint) is the
  **k-particle → N-body hierarchy at FIXED N** — a bounded tower of liftable folds. It stays
  inside the liftable class *as long as the organ-set is fixed*. This is why the capstone is
  a **recursive latent-organ tree**, not a new algebra: it climbs dimension (frame-exit,
  §5) without crossing the Fock boundary.
- The **learn-core** (`antigen::learn`: affinity-maturation) is the organ that *will* cross to
  the **N-varying** side — a germinal center holding a *variable antibody population*. **[The
  variable-N population is DESIGN-TIER, not built — flagged by the 2026-07-08 code-lens sweep.]**
  In the current source the learn-core matures **one draft at a time** (`learn/maturation.rs:248
  mature` climbs a single `current`; `learn/propose.rs` takes one cluster; `learn/life_record.rs`
  is one-record-per-class); there is *no* `Repertoire { clones: Vec<…> }` growing/pruning a live
  population across rounds. So the code today sits structurally on the **fixed-fold side**, and
  "grows/prunes clones" is the chartered dream, not the instantiated type. What is *design-true*:
  the learn-core is the ONE organ **positioned** to cross the Fock boundary (variable-N is its
  chartered shape), which is exactly why it is the keystone that couldn't be stamped stable with
  zero callers (CLAUDE.md status) — a Fock-crossing organ is categorically harder than a fixed
  fold. **The fixed latent tree and the (to-be-built) variable-N learn-core are the two sides of
  the Fock boundary** — the reason they must be different organs with different safety-governance,
  not one mechanism. The variable-N `Repertoire` type is a named build, not a claim of code.

### 11.6 What this witness CHANGES (live consequences, not just satisfaction)

1. **B2′'s §8 gap closes with a citation.** The type-wall the signal-algebra expedition owes
   (§8) has a proof (Theorem 22.1) and a *shape* (§11.4: separate output target, not a signed
   value). Both belong in that expedition's charter.
2. **The `Op`-dial gets a mechanism.** §1's "semirings = Op-dial" is now "λ-temperature
   family joined by log-sum-exp" (§11.3) — testable: does antigen's route/conductance
   `field_at` sit at `λ→∞` (tropical-min) of the same family the detection-count sits at
   (`λ=0`)? Falsifier #3 (§9) sharpens: signed-flow = letting `λ` go complex = interference.
3. **intent-substrate = the `z_min` reference.** §11.3 gives the keystone a mathematical
   role: the hinge is *definitionally* a subtraction against the setpoint; no setpoint, no
   COMPARE. Reinforces intent-substrate gating v073/v074.
4. **The capstone's boundary is named.** §11.5 draws the exact line the latent tree must not
   cross (fixed-N) and names the one organ that does (learn-core) — a real architectural
   constraint on ADR-071's B4, for the backward-audit amendment (§8).

*The convergence is now four-project AND cross-tier: antigen (code) · camp (relationships) ·
tambear (computation) · pith (prose) instantiate the shape; and tambear/pith's render-math
proves the shape's algebra. The priors we derived, an independent field proved. See sweep
notebook 023 for the full cross-project map.*

### 11.7 Tier-honesty ledger (what the math-witness IS vs ISN'T, per the code-lens)

*antigen's own self-distrust discipline, turned on this notebook: a witness is only as true as
its tier. The 2026-07-08 code-lens sweep graded each §11 claim against real source.*

| §11 claim | tier | witness / gap |
|---|---|---|
| afferent wing = monotone accumulate | **CODE** | `scan/walk.rs:128` (pure `+=`/`push`/`extend`), `read/tier.rs:104 corroborate` (`max`-only) |
| efferent = bounded-reversible-governed, not a ring | **CODE (one witness)** | `learn/curate.rs:71 Forget` (reversible KEEP/HOLD/ROUTE/ReArm + one gated irreversible); rest of efferent is frontier |
| B2′ = Thm 22.1 (the boundary is proven) | **PROOF** (math) + **CORE type-wall now BUILT** (2026-07-08) | `antigen-stroma/src/signal.rs`: `write_back` yields the successor `Up<R>` by construction, so an organ cannot write at ≤ the tier it reads (born-red ATK-SIGNAL-B2PRIME-WRITE-AT-READ-TIER, E0308). §8 gap CLOSED for the single-input core; **deferred**: the multi-input `MaxTier` (peano strict-above *relation* hits stable-Rust coherence overlap → successor-strictness instead). Separate from the `facts.rs` idempotency gate (still a comment). |
| hinge = COMPARE = Should − Is against a setpoint | **CORE BUILT + LIVE** (2026-07-08) | `antigen-stroma/src/intent.rs`: `Setpoint` (attested `(who, why)`, role-typed `Disposition`) + `compare()` = a three-valued conformance-gap (Conforms/Absent/Underdeveloped). Now **wired to REAL scan output** via `conformance.rs` (the conformance-monitor: `#[presents]`/`#[defended_by]` markers → observed `Strength` → per-locus gaps over an actual codebase). Rank-1 (attested). Deferred: canonical locus-resolution + tolerations, the what-was-meant AIRE-analog, the 4-dim observer. |
| semirings = one λ-temperature family | **DESIGN / falsifiable** | `field_at` + all 4 semirings `todo!()`; verdict "rhyme until `fold_λ` built" (§11.3) |
| learn-core = the variable-N Fock-crosser | **DESIGN** (positioned, not built) | matures one draft; no `Repertoire` population type (§11.5) |
| firewall = channel-split (efferent = separate target) | **CORE now BUILT** (2026-07-08) | `signal.rs`: `EfferentChannel<T>` is a distinct type the afferent `write_back` cannot consume (born-red ATK-SIGNAL-B2PRIME-EFFERENT-IN-AFFERENT, E0308) — a signed inverse cannot enter the afferent semiring. The MRT-shape as a type. pith's `suggest`-annotation-channel is the sibling prose-side proof (023 Part IV). Deferred: the efferent wing's own governance (a separate build). |

**The honest one-line:** the *afferent wing is compiled*; the *math-witness is a proof about
the algebra*; the *efferent wing, the hinge, the λ-family, and the Fock-crossing learn-core are
DESIGN-tier* — chartered, math-backed, and (for the channel-split) already built by a sibling,
but not yet antigen's code. The witness tells us *what to build and why it's right*, not *that
it's built*.

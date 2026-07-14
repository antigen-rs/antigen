# 027 — The Streaming Enriching Current (the afferent wing in its running form)

*Synthesis of a live co-design session (2026-07-13), building on nb 022 (two-winged body), nb 025
(multi-axial stroma + the concept×tier lattice), nb 026 (Ashby/Friston). Six threads —
imaginary-discriminant · N-modal root-portrait · tier-relative SNR · Fock-noise-to-info · residual-lifting
· enriching-current — resolved into one coherent streaming afferent wing. Tier-honest: **CONFIRMED** (rests
on ratified structure) / **INFERENCE** / **CONJECTURE** (under active enrichment sweeps).*

**Headline:** antigen's afferent wing is not a batch graph — it is a **salsa-driven streaming *enriching
current*.** The drive (new data) is the carrier; each interpreting organ appends its `(output, residual)`
to a monotone-accumulating bundle; residuals ride up and are *progressively explained* (noise→info); and
"the stroma HOLDS ALL" is upgraded from a design decision to a **theorem** — you cannot pre-classify
anything as noise, because signal/noise is tier- and Fock-relative.

---

## §1 — The current (INFERENCE, resting on nb 022 §11.2's afferent algebra)

The afferent wing is a **live dataflow**, not a graph you recompute on demand. Two layers:
- **The carrier / "electricity" = the drive** — the arrival of new data (commits, edits, coverage runs,
  scans). When the drive stops the current is quiescent; each new datum is a pulse.
- **What rides the carrier = signal + residual** — facts, organ-outputs, and un-explained residuals.

Each organ is an **interpreting transducer**: it reads the accumulated bundle, *decides / pattern-matches
/ thinks*, and forwards `drive + own-signal + own-residual` at a strictly-higher tier. **Why streaming and
not recompute:** the drive never stops — by the time you would recompute, the input has already changed —
so the only coherent model is *maintain-in-flight*, and the mechanism is **salsa** (the ratified
revision-clock, stroma-engine charter): a change-pulse propagates through the dependency graph and *only
the touched organs re-fire.* The recompute-vs-materialize fork dissolves: the residual is an
**incrementally-maintained live channel**, cache-backed (salsa memoization), not recomputed-from-scratch
and not durably-stored-and-diffed. The real axis was **static vs streaming**, and the wing is streaming.

## §2 — Tier-relative SNR = the renormalization relevant/irrelevant split (INFERENCE)

SNR is not a per-stream scalar — it is a **per-(stream × tier) field.** The signal/noise decomposition
depends on which organ is reading, and it *changes up the tiers* exactly as the **renormalization group**
reclassifies degrees of freedom (relevant = signal, survives coarse-graining; irrelevant = noise, averages
out) at each scale. So the concept×tier lattice (nb 025 §4) carries an SNR recursion: `fragility@0` and
`fragility@1` have different signal/noise. The sharp form: **the noise at tier-k is often the *signal* at
tier-(k+1)** — a node's coverage-*variance across runs* is noise to the coverage organ but *is the
flakiness signal* to the organ above; molecular motion is "noise" until temperature is the macro signal it
becomes.

## §3 — Fock positioning converts noise → info (INFERENCE, = nb 022 §11.5 through the SNR lens)

A **fixed-N afferent organ must discard variable-N structure as noise** (it can't hold a population beyond
its fixed arity — the overflow "doesn't fit"). The **variable-N learn-core reads that same structure as its
signal** (the population *is* the repertoire). Crossing the Fock boundary *is* the noise→info conversion.
This is the nb 022 §11.5 Fock boundary seen through SNR, with a literal grounding: in Fock space **vacuum
fluctuations (noise) become real correlated photons (signal)** under parametric down-conversion. And the
**dimensional-Nyquist / Marchenko-Pastur floor** (nb 022 §11.5) sets *where* the floor sits per resolution
— a signal above the floor at one dimension *aliases into noise* at another. Fock/dimensional positioning
is the knob that **moves the noise floor.**

## §4 — Residual-lifting: why higher organs exist (INFERENCE, strong — a 2nd justification)

nb 025 §3 justified higher organs by *emergence* (fold the field into invariants no scalar holds). There is
a **second, independent reason:** a tier-k organ emits `output` (what it explained) **+ `residual`** (what
it couldn't = its noise), and the tier-(k+1) organ lifts *both*. **A higher organ exists to explain what
the lower organ threw away** — because, by §2, that residual *is* a higher tier's signal. One structure,
three names:
- **Predictive coding (Friston, again).** Each layer sends its *prediction* down and its *prediction-error*
  (the residual) *up*; the layer above exists to explain the error. This IS the nb 026 §2 Friston hierarchy,
  with the residual as the up-going channel. (STRUCTURAL-MAP bordering on literal.)
- **Gradient boosting.** Fit a model, fit the next to its *residual*. The concept×tier lattice is a boosting
  stack: `fragility@(k+1)` fits `fragility@k`'s residual.
- **Renormalization (sharpest).** RG integrates out the high-frequency "noise" modes at a scale — but their
  effect *renormalizes the couplings at the next scale.* The discarded modes *become* the higher tier's
  signal via the coupling flow. The residual *is* the RG flow (nb 025 §4).

So the DAG input-types upgrade: a tier-(k+1) organ reads **`{raw axes} + {lower outputs} + {lower
RESIDUALS}`** — the residual is a *first-class input channel*, distinct from the output.

## §5 — Enrichment: the noise→info conversion made mechanical (INFERENCE)

The current is **enriching / monotone-accumulating**: nothing is removed; each organ only *appends*. So the
bundle **grows** up the tiers — raw passes through, and every tier adds its `(output, residual)`. A tier-k
organ sees the **whole accumulated enrichment** (raw + all lower outputs + all lower residuals) and taps
whatever *its* sense needs.
- **This IS the afferent monotone-accumulate wing (nb 022 §11.2), streaming.** "Only add, never subtract" =
  the accumulate law; "add at a strictly-higher tier" = B2′. The enriching bus and the afferent algebra were
  always the same object — this is its dataflow form. It is *why* raw + residuals are re-readable at every
  order (nb 025 §3): the bus accumulates, nothing dropped. The fixpoint (nb 022 B4) = **enrichment
  saturation** (a tier whose `(output, residual)` is already derivable adds nothing → the climb stops).
- **Enrichment is the noise→info mechanism.** tier-0 emits `residual_0` (noise); tier-1's read produces
  `output_1` that *explains part of `residual_0`* — the noise became signal *by being enriched*. So "does the
  twist persist?" = "does this residual stay *un*-explained all the way up, or does some tier enrich it into
  signal?" A residual explained-away is noise; one surviving every tier's enrichment is real structure.
- **Each organ interprets** (decides/thinks/pattern-matches) — an *agent on the bus*, not a mechanical
  filter. The **LLM-reasoner** (held out-of-scope) is just the *richest tap* — a tier-k agent reading the
  whole enrichment and adding its interpretation + residual. No special case; one more organ on the current.

## §6 — "Hold all" is a THEOREM, not a preference (INFERENCE — the loop this closes)

We enshrined "the stroma HOLDS ALL" as a founding decision (Ashby requisite variety,
[[project_substrate_holds_all_organs_project_selectivity]]). §2–§5 give it a **second, independent proof:**
**you cannot pre-classify anything as noise, because signal/noise is tier- and Fock-relative — what you'd
discard at tier-0 is the learn-core's signal at variable-N.** So the substrate *must* hold all — not by
preference, but because *there is no observer-independent noise to drop.* And "hold all" is **dynamic**: not
a store that contains everything, but **the live current carrying everything (signals + residuals) in
flight.** Substrate-holds-all + tier-relative-SNR + Fock-noise-to-info are **one principle**, reached from
three directions. Consequence: *discarding a residual at any tier below the Fock boundary is a bug* — the
current keeps it; the organ ignores it *without deleting it.*

## §7 — The root-portrait self-division: split / twist / poised (SWEEP RETURNED — framing STRIPPED, see §10b)

> **Superseded 2026-07-13:** the polynomial/discriminant/holonomy framing below was math-swept and mostly
> rejected (modes ≠ roots; the holonomy is a RHYME, 4/4 ingredients absent). What survives is a
> *frontier-native* trichotomy (SPLIT / LEAVE-WHOLE=overlapping-support / POISED→RouteToHuman) — see §10b and
> the [[root-portrait-self-division]] charter (corrected). The paragraphs below are retained as the record of
> what was tested.

A cluster's modal structure reads as a **polynomial root-portrait.** Two modes = a quadratic; the
discriminant selects THREE regimes (not two):
- **Δ > 0** (two real roots) → separable → **SPLIT**.
- **Δ < 0** (complex-conjugate pair) → real center + imaginary spread, no real separation → **LEAVE WHOLE,
  record the phase** — the *twist/kissing* state (cutting it severs a real coupling; tractography refuses to
  cut kissing fibers).
- **Δ = 0** (repeated root) → the **bifurcation / critical boundary** — a *local, per-fiber* criticality
  (NOT the global SOC that Bak-measurement falsified, nb 026 §4; the marginal-split edge).

The **imaginary part is proposed to be the twist / phase / holonomy** (nb 022 §11's Berry-phase / comma /
curvature) — literal-vs-structural under active math sweep. **N-modal** → a degree-N polynomial; the
split-decision is a **partition of the roots**: real singletons split off, **conjugate-pairs stay
entangled**, repeated roots poise. Crucial: *"count the bumps" is a lossy projection onto the real line that
erases the twist-pairs* — the true modal structure is the complex root-*portrait*, not its real shadow. This
is the same organ as the **crossing-node ODF** (nb 026 tractography): a crossing node's N-modal ODF *is* the
root-portrait, and the modal-partition is "which fibers separate vs stay bundled-with-a-twist." And the
twist verdict is **tier-relative + SNR-gated**: a low-SNR stream can manufacture a *spurious* twist, so the
twist is adjudicated by **persistence up the residual-climb** (does it stay un-explained through the tiers?)
— *preserved-then-resolved-by-persistence.*

## §8 — Everything is streaming persistence (INFERENCE)

SNR, twist, N-modal, self-division are all one shape: **persistence of a feature over a filtration** —
here, over the *flowing residual-stack up the tier-climb* — **re-adjudicated continuously because the drive
never stops.** A twist real last week may dissolve as new data arrives; the current catches it for free. The
confidence-graded-topology barcode (built, `engine/path_homology.rs`) is the first *tier-filtered* instance;
the streaming version is the general organ. This is the same "parametrized family of shapes / persistence"
motif as nb 022 §11.8 (λ × Mapper-resolution) — here the parameter is the tier-climb.

## §9 — Tier-honesty + the open enrichment sweeps

- **CONFIRMED-consistent** with ratified structure: the afferent accumulate algebra + B2′ (nb 022); salsa as
  the revision-clock (stroma-engine); the concept×tier lattice + raw-re-readable (nb 025); the Fock boundary
  + dimensional-Nyquist (nb 022 §11.5).
- **INFERENCE (design-tier):** the streaming-current form, tier-relative SNR = RG, residual-lifting =
  predictive-coding/boosting/RG, enrichment = noise→info, hold-all-as-theorem.
- **CONJECTURE (under active sweeps 2026-07-13):** (1) the **streaming-dataflow prior-art** — can salsa carry
  residual-channels + streaming persistence, or is a differential-dataflow layer needed? (2) the
  **SNR-per-(stream×tier)** noise models + first experiment. (3) the **imaginary/holonomy** verdict — is the
  twist *literally* a Berry-phase (an identity like B2′=Thm 22.1) or a structural-map? Their returns enrich
  this notebook (fittingly).
- **Concrete organ landed:** the tier-filtered barcode (`path_homology.rs`) — the first streaming-persistence
  primitive, built ahead of its consumer.

## §10 — Enrichment (2026-07-13 sweeps): the three verdicts folded in

Three research sweeps enriched this notebook. Tier-honest verdicts:

**(a) SNR machinery — buildable, mostly RECOGNITION.** `SNR_s(k)` is a *curve* over the tier index (§2 made
concrete): a per-axis **noise model** with concrete floor estimators (Axis-1 fidelity = syntactic
false-edge-rate vs SCIP, ~48% prior; Axis-7 co-change = PMI/lift; etc.) and three SNR-metric families
(edge-set = precision-ratio vs a higher-tier reference; verdict = stability-under-resampling; numeric = μ/σ).
**Two denoisers already SHIPPED:** the `TierBarcode` (per-stream tier-filter) + `corroborate`
(cross-orthogonal-axis — noise *multiplies* because the axes are independent). The genuinely-new organ: the
**residual-lift test** — a residual element *un*-explained on its own axis but *corroborated* on an
orthogonal one is the noise→info event (`¬signal(s₁) ∧ signal(s₂)` = the paracrine/hidden-coupling signal),
realized as a **cross-axis persistence barcode.** First experiment: Axis-1 false-edge-rate vs SCIP on
antigen+tambear (converts the prior to MEASURED + validates the barcode-as-denoiser).

**(b) Self-division — root-portrait/holonomy STRIPPED (measure-before-identity; 4th catch, cleanest).** The
polynomial read is FALSE as a density fact (modes ≠ roots — Duistermaat: 3 Gaussians → 4 modes;
critical-points are *transcendental*). The N-modal root-partition needs a 1-D projection = the Goodhart
scalar `affinity.rs` refuses (no `Ord`). The **holonomy is a RHYME** (4/4 ingredients absent — no bundle,
loop, connection, or observable phase; and the render-math witness §11.4 argues to *channel-split phase
AWAY*, since a phase in the afferent accumulator IS the B2′ oscillation pathology — antigen is *not an
interferometer*). What SURVIVES → the **frontier-native trichotomy**: SPLIT (two lineages jointly dominate a
frontier one can't reach) / **LEAVE-WHOLE = overlapping support, NOT a phase** (measurable) / **POISED =
the `disc=0` bifurcation → `RouteToHuman`** (the one earned discriminant-cognate; Ashby: eager Loop-2 is
instability). Decidable on shipped `affinity.rs` Pareto geometry. Details + the three assumption-flags:
[[root-portrait-self-division]].

**(c) The streaming compose/build map — salsa carries the CLOCK, not the SEMANTICS.** salsa (chosen,
ADR-068) carries the drive-tick clock + tier-monotone coarse-recompute; it carries NONE of the
residual/streaming/persistence semantics natively.
- **COMPOSE:** salsa + streaming-stats crates (Welford = noise-floor; t-digest = quantiles) + union-find +
  `lophat` (ordinary persistence).
- **BUILD (thin):** the never-stopping **drive loop** (salsa is pull-only); **residuals as tracked-return
  VALUES** (salsa's `Accumulator` is an output-only TRAP, not readable by higher queries); a per-organ
  **ε-cutoff brake** (floats wobble → without it the whole stack re-adjudicates every tick); a residual
  **whiteness stopping-test** (Kalman-innovation — a residual still autocorrelated ⇒ a higher tier *should*
  exist = a principled fixpoint criterion, sharpening §5's enrichment-saturation).
- **PORT:** **vineyards** (Dionysus2 C++ → Rust; no Rust impl; the stream-axis persistence).
- **DO NOT** put differential-dataflow beneath salsa — antigen is ~thousands of nodes, 4–6 orders below where
  DD pays; revisit only if measured past ~10⁵ nodes.

**Three structural results worth ratifying:**
1. **Tier ⟺ propagation-height ⟺ GLITCH-FREEDOM.** The strict tier-monotone order IS height-ordered reactive
   scheduling (FrTime/REScala): each organ reads only strictly-lower tiers → no organ observes a
   half-updated bundle. Tier-monotonicity is the *glitch-free consistency guarantee* — the prior-art
   justification for why B2′ must be *strict*.
2. **B2′ becomes a TRAIT BOUND.** Differential-dataflow's algebra: a diff is a commutative-monoid element;
   **negation lives in a separate `Abelian` trait.** Enforce the afferent-never-subtracts law as **`R:
   Monoid` but NOT `R: Abelian`** — the type-level "the afferent side can't form an oscillator" (prior art:
   positive Z-sets, Datalog-over-semirings). The most elegant compose-win of the survey.
3. **The residual-flow has a three-altitude referent** — Kalman-innovation (what a residual *is* + the
   whiteness test) + predictive-coding (the tier hierarchy) + gradient-boosting (the mechanism). Confirms
   §4's convergence, adds Kalman.

**The novel primitive (honest).** The dual-axis verdict — persistence *up the tier-climb* × persistence
*over the data-stream* — is unpublished; it is the PRODUCT of multiparameter-persistence (tier × λ, computed
as fibered 1-param slices for MVP) × **vineyards** (the stream is *time deforming the filtration*, NOT a
third filtration axis — treating it as one is a category error) × **ZIGZAG persistence** (the *unexplained-
residual* channel: a residual is *born* when it appears and *dies* when a higher organ *explains it away* —
so the accumulate-output channel is ordinary append-only persistence, but the **residual channel is
zigzag**; a real distinction the design should name). Correctness witness: property-tests vs batch-recompute
(a natural negative control). **Novelty = the composition, not the primitives** (= the project's own
"novelty = conjunction of axes, not single axis").

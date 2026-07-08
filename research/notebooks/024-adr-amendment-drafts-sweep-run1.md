# 024 — ADR Amendment Drafts (sweep run-1 → ratification ceremony)

> **What this is.** Three ADR amendment DRAFTS produced by the 2026-07-08 cross-project sweep
> (nb 023 Part IV convergent actionable set). These enter the ADR lifecycle (docs/process.md):
> draft (here) → author-distinct panel (aristotle deconstruct + adversarial attack; main-loop
> drafter excluded — no-self-witness) → certification (BLOCK or bless) → co-captain bless
> (Tekgy) → land into decisions.md (append-only, supersede-not-erase). **Nothing here is
> ratified until it lands in decisions.md through that ceremony.**
>
> Two of the three are **recognition** amendments (037-Am1, 071-Am1 B4′ — naming what ADR-071
> already implies); the third (068-Am1) is a **DESIGN-tier conjecture + its falsifier**, framed
> as process-not-outcome per "ADRs encode invariant + process, never an outcome."

---

## Draft 1 — ADR-037 Amendment 1: the four control-theory extras are latent ORGANS, not candidate stages

**Amends:** ADR-037 §"What this ADR does NOT do", the bullet at `decisions.md:9832-9838`.
**Kind:** recognition / correction (supersede-not-erase). **Warrant:** ADR-071 (RATIFIED
2026-07-07) B1 + B7.

**Draft text (to append to ADR-037, after the §"does NOT do" block):**

> **Amendment 1 (2026-07-08) — the four extras are latent organs (ADR-071 B7), not stages.**
> The bullet above hedged that "a real control loop carries more structure than this frame names
> (observability, controllability, delay/latency, stability margins); any of those *may add a
> stage* on a later finding." ADR-071 resolves those four precisely: by **B1** the loop is EDGES
> not NODES, and by **B7** the four — observability, controllability, delay/latency,
> stability-margin — are the structurally-guaranteed (Ashby) self-regulation **latent ORGANS**
> (nodes), named on_hold, *indexed-by* (not stages of) the loop.
> **Correction (supersede-not-erase):** the four named extras are NOT candidate *stages*; they
> are latent organs per ADR-071 B7. The open-edge honesty of this ADR stands — but it applies to
> genuinely-new **edges** (new signal-flow structure), and any future new-stage claim routes
> through the right route: any future new-**organ** claim routes through ADR-071 B1's
> individuation rule (build-cost + blast, edges-not-nodes); any genuinely-new **edge** (a real
> new stage) remains open under this ADR's open-edge honesty — never through this ADR's original
> "may add a stage" phrasing for the four named extras (which are organs, not stages). The one
> cross-validated
> prediction (setpoint → autoimmune, N=1) is untouched; only the mislabeling of the four extras
> as potential stages is corrected.

**Why now / why recognition:** ADR-071 B7 already *named the same four* as latent organs; this
amendment removes the surviving contradiction (the older ADR still calls them possible stages).
No new claim — it aligns two ratified ADRs.

---

## Draft 2 — ADR-071 Amendment 1: B4′, the Fock-boundary constraint on the latent tree

**Amends:** ADR-071 §Decision B4 (`decisions.md:14025`). **Kind:** recognition + math-import
(append B4′). **Warrant:** research nb 022 §11.5 (the liftability Fock boundary); retroactively
explains the zero-caller beta bar the project already lives by (ADR-006 recognition).

**Draft text (to append to ADR-071 §Decision, as B4′):**

> **B4′ (2026-07-08) · The Fock-boundary constraint on the latent tree.** B4 calls the capstone
> tree "open-ended, climbing the DAG to the fixpoint." The 2026-07-08 cross-project math-witness
> (nb 022 §11.5; the liftability framework's Fock boundary) sharpens "open-ended": the climb is
> open-ended in **tier-depth** but must stay **fixed-N in afferent breadth**. The recursive
> latent-organ tree must remain **fixed-N** — every latent organ senses from a *statically-fixed
> set* of lower organs (a fixed-arity afferent tuple), keeping the whole tree inside the
> **liftable class** (Liftability Theorem 16.1: a fixed-k hierarchy at any k is liftable, "with N
> replaced by kN"). The **learn-core is the one currently-chartered organ positioned to cross the
> Fock boundary** (variable-N — a population whose count depends on field state, the one
> non-liftable class); it therefore lives **outside** the latent tree's additive climb, gated by
> its own safety-governance, never wired as an ordinary afferent node in the fixpoint. *(No
> uniqueness proof is claimed — a future organ could also require variable-N; if one arises it
> takes the same outside-the-tree, separately-governed placement. What is forced is the
> **placement** of any Fock-crosser, not that there is exactly one.)* **This retroactively
> explains why the learn-core could not be stamped stable with zero callers** — a Fock-crossing
> organ is categorically harder than a fixed fold.
> **Typed corollary (a build directive to signal-algebra, feeds B5 sub-clause F — NOT yet built):**
> a latent organ's afferent input should be a fixed-arity tuple (liftable), not a variable-length
> population; the signal-algebra expedition's type-wall **must enforce** this — a `Vec`-shaped
> population input *would* flag the Fock-crosser at the type level. This enforcement does not exist
> today (the B2′ type-wall is itself unbuilt — nb 022 §11.7).
> **Companion — the depth-cap, conjectured form FOUND (2026-07-08 hunt), still DESIGN-tier:** B4's
> tier-*depth* is also bounded by the observer's rank — `ResolutionTier` has 3 values (rank-1,
> depth ~2), so "open-ended climb" needs either an unbounded organ-output tier (ℕ) distinct from
> `ResolutionTier`, or a **dimensional-Nyquist depth cap**. The cap's *form* is now recovered from
> pith (`well-tempered-expedition/expedition-log.md:227`, PR-3): the **information-theoretic
> octave**, `D ≈ log₂(N) + const` — each doubling of the base organs' *independent* signal adds ~1
> resolvable latent-organ tier; above that a higher organ falls below the **Marchenko-Pastur noise
> floor** and *aliases* (its output indistinguishable from noise). So the honest cap is "climb
> while information-gain clears the MP floor, ~`log₂(N_facts)` tiers," which even rationalizes
> `ResolutionTier`'s 3 values ⇔ ~4–8 independent base signal-classes. **[TIER: pith PR-3 is a
> PRE-REGISTERED PREDICTION, not a confirmed result — this imports the depth-cap's conjectured
> *form*, not a proven bound; it graduates conjecture→witnessed only when pith's experiment
> confirms PR-3.]** B4′ therefore names the depth-cap as a **conjectured `log₂(N)` form**, no
> longer formless-open.
> **Tier-honesty:** B4′'s fixed-N constraint is a math-backed DESIGN constraint; the variable-N
> learn-core population is itself DESIGN-tier — not built (nb 022 §11.5, §11.7): the code today
> matures one draft, with no `Repertoire` population type.

**Why now / why recognition:** the project *already* treats the learn-core as categorically
harder (the zero-caller beta bar). B4′ names *why* (the Fock boundary) and constrains B4's
"open-ended" so it can't be read as "variable-N afferent inputs," which would break liftability.

---

## Draft 3 — ADR-068 Amendment 1: the four semirings as a conjectured λ-temperature family (DESIGN-tier, falsifiable)

**Amends:** ADR-068 §Decision item 2 (`decisions.md:13673-13676`). **Kind:** DESIGN-tier
conjecture + falsifier (process, not outcome). **Warrant:** research nb 022 §11.3.

**Draft text (to append to ADR-068 §Decision item 2, as a sub-note):**

> **Amendment 1 (2026-07-08) — the four semirings as a conjectured λ-temperature family
> (DESIGN-tier).** Item 2 establishes as MEASURED that the four semirings are ONE reachability
> *relation* under four value-algebras (the identical 33,882-pair relation). The 2026-07-08
> math-witness (nb 022 §11.3) proposes a deeper unification *one tier below* the measured fact:
> the four value-algebras may themselves be **λ-limits of a single log-sum-exp temperature
> family** — `softmin_λ(z) = −(1/λ)·log Σ exp(−λ zᵢ) → min` as `λ→∞` — with detection/blast at
> the `λ=0` (additive) end and field-conductance at the `λ→∞` (tropical-min) end.
> **This is a DESIGN-tier CONJECTURE, not a ratified fact.** It does not overturn item 2's
> measured "one relation, four semirings"; it proposes that the four *value-algebras* are one
> thermometer. **The falsifier** (owned by the signal-algebra build, which pours the first real
> `field_at` — `query.rs:35` is `todo!()` today): a single `fold_λ` must recover the
> detection/blast count at `λ=0` AND the conductance-min at `λ→∞` *bit-for-bit*, with
> `O(exp(−λΔ))` deformation on a λ-sweep property test. **Pass** → the `Op`-dial is a real
> temperature-family (one fold, a knob). **Fail** (two structurally distinct reducers that merely
> happen to be a min and a sum) → the λ-family is a *rhyme*; a **follow-on amendment records the
> failure and supersedes this conjecture** (append-only per process.md — not a zero-ceremony
> auto-withdrawal). Either way the corpus stays true: what this amendment ratifies is the *test*
> (an invariant that holds regardless of the conjecture's fate), never the conjecture's outcome.
> The honest current standing verdict is **"rhyme until a unifying `fold_λ` is built."**

**Why now / why safe:** it encodes a conjecture + its acceptance-test as *process* (per "ADRs
encode invariant + process, never an outcome"); it explicitly defers the outcome to the build
and self-withdraws on failure — so ratifying it commits nothing false, only names a test.

---

## Ceremony checklist — COMPLETE (2026-07-08)

- [x] Author-distinct panel reviewed all three (aristotle deconstruct + adversarial attack); main-loop drafter excluded (no-self-witness).
- [x] Certification: aristotle SOUND ×3; adversarial D1/D3 PASS, D2 BLOCK (2 over-reaches) → fixed → **fresh** adversarial re-attest **CLEAR**.
- [x] Co-captain bless (Tekgy) — incl. the D3 "rhyme until" standing-verdict confirm ("agree on the rhyme, same feeling").
- [x] Landed into decisions.md (append-only, supersede-not-erase) as ADR-037 Am1, ADR-068 Am1, ADR-071 Am1. ToC stops at ADR-060 → no entry needed.
- [x] 023 Part IV "needs ceremony" items marked landed.

**Ceremony record:** BLOCK→fix→re-attest on D2 is the ceremony working as designed — the adversarial seat caught two real over-reaches (unwarranted "sole organ" exclusivity; present-tense enforcement for an unbuilt wall); both fixed and independently re-verified before landing. The Nyquist find (pith PR-3) *strengthened* D2's depth-cap (formless-open → conjectured `log₂(N)` octave) mid-ceremony, held honestly at pre-registered-prediction tier.

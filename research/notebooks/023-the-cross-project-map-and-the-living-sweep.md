# 023 — The Cross-Project Map & the Living Sweep

> **What this is.** A *living* notebook. On 2026-07-08 we (a) derived antigen's stroma-shape
> (the two-winged body, [nb 022]), (b) recognized it recurs across four sibling projects, and
> (c) found that tambear/pith's render-math *proves* the shape's algebra as theorems (022 §11).
> This notebook is the **map** of that convergence and the **ledger** of the sweep it triggers.
> It is designed to grow — every further exploration (unread pith docs, tambear's own n-body /
> dimensional-Nyquist math, new charters) appends here. Not a report; a **standing frontier**.

---

## Part I — The four-project convergence (one shape, four substrates)

Each project grew its **own stroma independently**, then we recognized they are the same
structure: `accumulate + gather + observer` over a different substrate — an afferent
(descriptive / fold / sense) wing and an efferent (generative / unfold / act) wing joined by a
COMPARE hinge (022 §0–§5).

| | substrate | afferent (Is / fold / sense) | efferent (Should / unfold / act) | the hinge (COMPARE) | observer |
|---|---|---|---|---|---|
| **antigen** | Rust codebases | scan → tier-tag failure-classes (built) | `propose` / effector / suppression (frontier) | intent-substrate setpoint (keystone, absent) | `ResolutionTier` (rank-1) → 4-dim control |
| **tambear** | numeric arrays / computation | `accumulate`(grouping, expr, op) | `gather` / unfold / JIT (the "door") | — (its lift is the fold half) | **Tam** the Observer (unbounded Fock-exit) |
| **camp** | relationships / orchestration | notes / notices (sense the team-state) | routes / handoffs (act on the team) | the campsite substrate as setpoint | the patron / helm |
| **pith** | prose / documents | topic-graph analysis (sense the text) | `suggest` / bounded edits (act on the text) | topic-templates + expectations (intent) | version-aware morph (the reader-frame) |

**The origin arc (phylogeny):** antigen was **born FROM tambear** — tambear kept hitting
*distributed-thinking failure-classes*, and antigen is the immune system that remembers them.
So the sibling relationship isn't analogy; it's **descent**. antigen built the *descriptive*
(fold / sense) half of the shared shape first; tambear built the *generative* (unfold /
compute) half first; each now reaches into the other's half (antigen via `propose`/intent,
tambear via observation). camp and pith are two further instantiations over their own
substrates.

**The recomputability spine (independently re-derived ×2).** ADR-067 §F.13's CONFIG/OUTPUT
split (authored+stable half vs recomputable+changing half) was **re-derived by pith** in its
`.pith` directory format (`document.md` = CONFIG, topic-graph/analysis = OUTPUT). Two projects
hit the same split from opposite substrates ⇒ recognition, not design (ADR-006).

---

## Part II — The math-witness (the priors became theorems)

Three co-pending render patents (the "lift" family that runs across every sibling) formalize
antigen's *derived* shape as **theorems**. Full mapping in **[nb 022 §11]**; summary here.

| antigen claim (022) | render-math theorem | what it upgrades |
|---|---|---|
| afferent wing = monotone accumulate | **Liftability** (Thm 16.1): liftable ⟺ additive per-particle fold | the afferent class *is* the liftable class |
| **B2′** (afferent algebra can't compute efferent) | **Operator Boundary** (Thm 22.1): non-additive ⟺ order-dependent | firewall proven, not conventional → cite in signal-algebra expedition |
| **the hinge** = COMPARE = Should − Is | **depth-kernel** (Nadaraya-Watson, `z_min`-referenced) | hinge = comparison-subtraction against a **setpoint** (= intent-substrate) |
| the four **semirings = the `Op`-dial** | **λ-temperature family**, `softmin_λ → min`, joined by log-sum-exp | dial gets a *mechanism*: `(+,×)` and `(min,+)` are two ends of one thermometer |
| **polarity flip** (Is/Should) | depth-kernel **sign of λ** (`z_min`↔`z_max` reference flip) | polarity = the sign of the hinge's temperature |
| **the inverse oscillates** (B2′ hazard) | **wave-optics**: complex kernel ⇒ interference | inverse ⇒ oscillation, *proven* |
| the firewall is a **channel-split** | wave-optics **MRT** (Re/Im targets, modulus readout) | efferent = a distinct output *target*, not a signed value in the afferent semiring |
| **capstone** = fixed recursive latent tree | **Fock boundary** (variable-N = non-liftable) | latent tree = fixed-N k-particle→N-body; **learn-core = the one Fock-crossing organ** |

**The single sharpest consequence:** the efferent Should−Is is *legal* — antigen is not
forbidden a subtraction — but only as a **separate channel** whose afferent-observable
projection is non-negative (the modulus). Complex inside, monotone at the surface. That is the
type the signal-algebra expedition must build.

---

## Part III — Cross-pollination threads (where the shared math surfaces)

Threads that appear in *multiple* siblings and will surface in antigen — watch-and-import:

1. **n-body / FMM** — the k-particle → N-body hierarchy (fast-multipole) is tambear's
   compute-tree, pith's render-tree (Fock boundary), AND antigen's **latent-organ tree**
   (ADR-071 B4). findmcp became the **F**ingerprint-**M**atrix-**M**odel classifier — a
   fingerprint FMM. *Import target:* the capstone's tree depth / summarization structure.
2. **dimensional-Nyquist** — (from tambear's math; not yet read in full) the sampling bound on
   how many dimensions a fold can resolve. *Predicted import:* the **rank-bound on the observer**
   (022 §8: B4 over-reaches a rank-1 tier). A Nyquist bound would say *how many* latent-organ
   tiers are resolvable before aliasing — a hard cap on the tree, not a soft one.
3. **folds / seams oscillation** — (from tambear's math + wave-optics interference) the ringing
   that appears at a fold-seam. *Import target:* B2′ — the seam between the two wings is exactly
   where oscillation threatens; the **stability-margin** organ (022 §6) watches it. Wave-optics
   proves the seam *is* an interference boundary.
4. **"lift" everywhere** — tambear's core verb, pith's render primitive, antigen's tier-raise
   (afferent → observer). The frame-exit / dimensional-raise (022 §5) is one "lift" seen four
   ways.

*These are OPEN threads. As tambear's own math notebooks are read, this section accretes the
actual theorems (dimensional-Nyquist statement, the folds/seams recurrence) and their antigen
import-targets.*

---

## Part IV — The bidirectional sweep (design + ledger)

The sweep combs antigen's substrate (and, at chosen breadth, the siblings') through the fully
enriched lens = [nb 022] + the math-witness (Part II) + the four-project frame (Part I). It runs
in **two directions**:

**(A) ABSORB (siblings → antigen).** What pith/tambear built that antigen's frontier needs:
- pith's **conformance / contradiction-detection** → antigen's `conformance-monitor` charter
  (COMPARE made concrete: text-vs-expectation = code-vs-setpoint).
- pith's **topic-templates + expectations** → antigen's **intent-substrate** shape (the setpoint
  the hinge subtracts against).
- pith's **Tier 0–3 collaboration ladder** → antigen's resolution-tier build-order.
- the **recomputability spine** (Part I) → confirm ADR-067 §F.13 against pith's `.pith` format.

**(B) IMPORT-PROOFS (tambear/pith math → antigen).** What the render-math proves that antigen
should *cite / enforce*:
- **Thm 22.1 → B2′ type-wall** (022 §8 gap): the signal-algebra expedition cites the theorem and
  builds the channel-split (Part II).
- **λ-family / log-sum-exp → the `Op`-dial** (022 §11.3): test whether antigen's route
  `field_at` sits at `λ→∞` of the same family as the detection-count at `λ=0`.
- **dimensional-Nyquist → the observer rank-cap** (Part III #2): a hard bound on latent-tree depth.
- **Fock boundary → latent-tree-vs-learn-core** (022 §11.5): the fixed-N / variable-N line as a
  real constraint on ADR-071 B4.

### Sweep ledger (findings land here)

*(empty — populated by the sweep run below; each finding: `[lens] · substrate-location ·
alignment|gap|import|tension · one-line`.)*

---

## Part V — Living frontier (what still wants exploring)

- [ ] read the remaining pith vision docs (any not yet seen) for further absorb-targets
- [ ] read **tambear's own math notebooks** — the *actual* dimensional-Nyquist statement + the
      folds/seams recurrence (Part III #2, #3 are placeholders until then)
- [ ] the kernel-substitution ("Application A") patent — the full kernel *library* (the `Op`-dial
      as an actual interchangeable-kernel set)
- [ ] does findmcp's FMM classifier share the learn-core's variable-N population? (Fock-side rhyme)
- [ ] ADR-037 T1 drift (stale "may-add-a-stage" vs 071-B7 Ashby-forcing) — captured for amendment

---

*Home notebook for the 2026-07-08 cross-project convergence. Grows by accretion. See [nb 022]
for the derived shape and its §11 math-witness; see the gap-charters (jbd/charters/) and the
intent-substrate expedition for where the empty cells became work.*

[nb 022]: ./022-the-two-winged-body-the-derived-stroma-shape.md
[nb 022 §11]: ./022-the-two-winged-body-the-derived-stroma-shape.md#11-the-math-witness--three-co-pending-render-patents-prove-the-design-tier-priors

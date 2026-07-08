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

### Sweep ledger — run 1 (2026-07-08, 6 read-only lenses over antigen's substrate)

*Antigen-only breadth (cross-project reach deferred, Part V). Each finding: `location ·
[tag] · one-line`. Six lenses: ADR · Charter · Notebook · Code · Absorb · Import-proof.*

**Code-lens (the grounding — what's real vs claimed):**
- `scan/walk.rs:128`, `read/tier.rs:104 corroborate` · [CODE-true] · afferent fold is built + monotone (`+=`/`push`/`extend`; `max`-only). The sensor wing is compiled.
- `learn/curate.rs:71 Forget` · [CODE-true] · the one built efferent organ, and it IS the bounded-reversible-governed shape (not a ring). §3 confirmed.
- `read/query.rs:{21,35,47,62}`, `base/facts.rs:131-146` · [gap] · `field_at` + all four semirings are `todo!()`/comment-only ⇒ **the λ-family witness is DESIGN-tier, unfalsifiable against current code** (→ corrected 022 §11.3).
- `learn/maturation.rs:248`, `learn/life_record.rs:155` · [gap] · learn-core matures ONE draft; no `Repertoire{clones:Vec}` ⇒ **Fock-crossing is a doc-claim, not a type** (→ corrected 022 §11.5). Biggest lens-vs-substrate gap.
- `write.rs:34-54` · [gap, §8 confirmed] · type-enforces only overlay-vs-injection; **no tier-write-ordering wall.** B2′ remains process-enforced.
- `scip.rs:36` · [gap] · SCIP feeder unbuilt ⇒ today's fold runs at `Syntactic`/`dread` tier only.

**ADR-lens (decisions.md):**
- 037 §"does NOT do" L9833-9834 · [tension, CONFIRMED] · "may add a stage" for observability/controllability/delay/stability-margin **contradicts ratified 071-B7** (those four = Ashby-forced latent *organs*, not stages). Narrow amendment.
- 071-B2′ L14023 · [import] · B2′-in-prose = Thm 22.1; cite the proof + build the type-wall (§8 gap).
- 071-B4 L14025 · [tension, CONFIRMED] · "open-ended latent tree" over-reaches rank-1 `ResolutionTier` (3 values → depth ~2). Needs fixed-N/Fock distinction + depth cap.
- 068 §2 L13673 · [import] · "four semirings" asserted as a list → λ-family mechanism (log-sum-exp) absent; cite it.
- 002-Amd3 L530-569, 069 POLARITY L13772, 070 fractal-spine L13882 · [alignment] · sovereign/non-cascade = channel-split rhyme; POLARITY = sign-of-λ; fractal CONFIG/OUTPUT = monotone-afferent law.

**Notebook-lens (research/notebooks/):**
- nb 009 (intent-kernel) · [CONFIRM, strongest prior] · reached `COMPARE = Should−Is` + intent-substrate-setpoint + polarity-as-1-bit a full cycle BEFORE the math proved it. Recognition, then witness.
- nb 019/021 §B2′ · [CONFIRM→PROVEN] · the acyclicity invariant is now Thm 22.1; §8 gap closes with a citation + shape.
- nb 019 §B5/B6, 021 §B "open-ended latent tree" · [CONTRADICT/stale] · Fock boundary + Nyquist cap say fixed-N/tier-bounded. Clearest stale claim → the B4 amendment.
- nb 006 (four maths) · [REVISE] · quasispecies/percolation/Hill/reaction-diffusion as *independent* borrows → subsumed into one λ-family; membrane-crossing = channel-split reached from 3 maths converging.
- nb 017 (Boyd ruling) · [CONFIRM+revise-reason] · FEEDBACK-stage emptiness Boyd read as "correctly thin" is now PROVEN (only the efferent wing has feedback; the DAG can't ring).

**Absorb-lens (pith → antigen frontier):**
- pith `suggest` = capability-gated annotation channel · [absorb, sharpest] · the MRT channel-split as a concrete API: `suggest`→annotation (never direct edit), promoted by explicit `resolve`, every op `{author,type,content,resolved,resolvedBy,timestamp}`. **Answers "what type is the efferent channel."**
- pith topic-templates/expectations · [absorb] · intent-substrate's schema = expected-defense-set per structural locus, per-element role-typed (present/missing/underdeveloped) ⇒ COMPARE is **three-valued** (matches antigen's three-valued trust law).
- pith contradiction-detection · [absorb] · COMPARE mechanism = extract declared-relation (setpoint) + observed-relation (scan), key by class-node, fire on opposite polarity. `Should−Is≠0` made discrete.
- pith Tier 0-3 ladder · [absorb] · build-order: sense→observe-only-COMPARE→act→multi-mind, each tier additive (never mutates lower algebra = B2′ as a build-rule). Confirms intent-substrate+conformance gate `propose`.
- pith `.pith` CONFIG/OUTPUT · [CONFIRM-strong] · bit-for-bit independent re-derivation of ADR-067 §F.13; adds content-hash cache-key as recompute-invalidation trigger.

**Charter-lens (jbd/charters/):**
- All 10 gap-charters + older charters place cleanly onto (wing·axis·Op·loop-pos·observer). No orphans.
- `b2-prime-signal-algebra-type-wall` · [gap, sharpest] · cites 022 §6/§8/§9 (pre-witness) but **zero** mention of Thm 22.1 / channel-split / MRT. Most-owing charter omits the witness.
- **No Fock-boundary/capstone charter exists** · [gap] · the fixed-N-tree-vs-variable-N-learn-core line lives only as a scattered horizon-tag. §11.5's core constraint has no charter home.
- λ-family falsifier · [gap] · §11.6 #2 test (`field_at` at λ→∞ vs count at λ=0) has no home; add to route-arm/cytokine-field.
- route-arm+cytokine-field+registry-herd · [merge-adjacent, keep-split] · share one gradient-field substrate, distinct payoffs (magnitude/character/aggregate) — "secretly-one-substrate, three-organs," already self-noted.
- `peripheral-delete-autoimmunity-pruner` · [alignment, exemplary] · charters the true-delete-vs-reversible-mask fork + pins §9 falsifier #4.

**Import-proof-lens (theorems → concrete enforcement):**
- Thm 22.1 → B2′ type-wall · [ready] · sealed `StrictlyAbove` tier-lift trait + no-public-ctor `OrganOutput<T,Tier>` (template = `tier.rs PresentsVerdict`); efferent = distinct `EfferentChannel`, never a signed `AfferentFact`. Buildable today.
- λ-family → falsifiable test · [ready] · one `fold_λ` must recover count@λ=0 AND field_at@λ→∞ bit-for-bit, `O(exp(-λΔ))` on a λ-sweep. Likely verdict "rhyme until `fold_λ` built" — itself a de-risking finding.
- Fock → B4′ clause · [ready] · one-sentence ADR-071 amendment: latent tree stays fixed-N (fixed-arity afferent tuples, liftable at kN); learn-core sole crosser, outside the tree, own safety-gate. Retroactively explains the zero-caller bar.
- dimensional-Nyquist → hard depth cap · [needs-tambear-math] · the ONE import yielding a hard number, gates ratified B4; theorem unread. **Top cross-project follow-on.**

### Convergent actionable set (ranked; every lens lands here)

**In-bounds substrate (notebooks + charters — done / doing in run-1 pass):**
1. ✅ Correct 022 §11.3 (λ-family design-tier) + §11.5 (learn-core Fock design-tier) + add §11.7 tier-honesty ledger.
2. ✅ This ledger (023 Part IV).
3. Amend `b2-prime-signal-algebra-type-wall` charter to carry Thm 22.1 + the channel-split type-shape (the ready `OrganOutput<T,Tier>`/`StrictlyAbove`/`EfferentChannel` design).
4. Create the missing **Fock-boundary/capstone** charter (fixed-N tree vs variable-N learn-core).
5. Home the λ-family falsifier (`fold_λ` sweep-test) in route-arm/cytokine-field.

**Needs ADR ceremony (draft-only; team ratifies per process.md):**
- ADR-037 T1 drift amendment (the four Ashby extras = latent organs, not open stages).
- ADR-071 **B4′** Fock-boundary clause (Import-lens drafted it).
- ADR-068 §2 λ-family mechanism (turn the list into a thermometer).

**Cross-project follow-on (blocked on tambear math — Part V):**
- dimensional-Nyquist → hard latent-tree depth cap (gates B4; the one hard-number import).

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

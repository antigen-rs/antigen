# Capture — Naturalist on F8 EvidenceKind Biology Validation

> **Date**: 2026-05-19
> **Author**: team-naturalist
> **Status**: append-only capture
> **What this addresses**: aristotle F8 elevated EvidenceKind to first-class
> axis with values `TypeSystemProof | Behavioral | SubstrateState`. R-N1
> (prior naturalist) mapped to innate-vs-adaptive immunity at machinery
> level but acknowledged Behavioral "sits between" rather than mapping cleanly
> to either of the two classical immunity categories. Team-launch invited
> deeper biology validation. This capture provides it.
> **Pure biology question**: no architecture-prescription (per framing-call
> correction).

---

## R-N1 framing — and what it missed

R-N1 mapped EvidenceKind to immunology at the machinery level:
- **TypeSystemProof** ↔ innate immunity (germline-encoded structural)
- **SubstrateState** ↔ adaptive immunity (B/T-cell substrate, drifts, refreshes)
- **Behavioral** "sits between" — test code is germline-encoded but
  invocation is runtime-experience-dependent

R-N1 was honest about Behavioral being a partial fit. That honest gap is
exactly what makes this worth a second pass — R-N1 worked from a two-category
biology (innate vs adaptive), but modern immunology recognizes a third.

---

## The third biological evidence-kind: trained immunity

Since approximately 2010-2015, immunology has recognized **trained immunity**
as a distinct phenomenon. Innate immune cells (monocytes, macrophages, NK
cells) show enhanced response to secondary stimulation after prior exposure.
The mechanism is epigenetic + metabolic reprogramming:

- Histone modifications (H3K4me3 enrichment at cytokine promoters)
- DNA methylation changes
- Metabolic reprogramming (shifts to glycolysis)
- Hematopoietic stem-cell-level epigenetic remodeling

This is NEITHER germline-encoded (it's acquired through encounter) NOR
somatically-recombined (no V(D)J shuffling — the cell's BCR/TCR equivalent
isn't created; the existing innate machinery is reprogrammed). It's a third
evidence-kind in biology.

Three biological categories:
1. **Germline-encoded** (innate, classical): hardwired in DNA, no learning,
   PRRs binding PAMPs
2. **Epigenetic-state-encoded** (trained immunity): source machinery exists
   in germline, recent encounter activates it via reprogramming, persists
   for weeks-to-months via epigenetic marks
3. **Somatically-recombined-and-clonally-selected** (adaptive, classical):
   substrate created by V(D)J recombination, refined by selection,
   accumulates per-clone

---

## Clean three-way map to EvidenceKind

**`TypeSystemProof` ↔ germline-encoded innate immunity**
- Phantom-type proofs are hardwired into the type system at compile time
- No runtime evidence required to fire; the structure IS the verification
- Can't drift unless type system breached
- Biology: PAMPs/PRRs encoded in germline DNA; can't fail by construction

**`Behavioral` ↔ trained-immunity / epigenetic memory**
- Test code exists in source (germline-like — the test is hardwired)
- But requires recent invocation history to provide evidence (epigenetic-
  like — the recent run is the mark)
- Test code without recent runs is *dormant* (innate cell that hasn't been
  trained)
- Test code with recent passes is *trained* (monocyte with H3K4me3 marks at
  cytokine promoters)
- The evidence IS the history of invocation, not the code-presence

**`SubstrateState` ↔ adaptive (B/T-cell substrate)**
- Sidecar attestations are cellular substrate analog
- Can drift (fingerprint changes; signers stale)
- Can be refreshed (re-attestation)
- Accumulates evidence-of-activation (signer history; delta chains)
- Has lifetime (freshness windows; expiry)

---

## Why this is sharper than R-N1's "sits between"

The three-way mapping is **biology-aligned, not just a software taxonomy**.
Each EvidenceKind has a distinct biological mechanism that explains its
structural properties:

- TypeSystemProof can reach FormalProof because germline encoding is
  mechanically immutable at runtime
- Behavioral reaches Execution (after harness invocation) because trained
  immunity requires a triggering event to demonstrate
- SubstrateState reaches Execution (when predicate passes + currency holds)
  because adaptive substrate requires currency-of-state to be meaningful

The per-tier ceilings v3 names aren't arbitrary engineering choices — they
reflect WHICH BIOLOGY can-fail-by-what-mechanism. Germline can only fail by
mutation (impossibly rare in well-tested type systems); epigenetic can fail
by mark-decay over time (test gets stale; CI not run recently); adaptive can
fail by clonal drift (sidecar fingerprint mismatch).

This is a cleaner story than "the engineering axis happens to have three
values, two of which map to biology."

---

## Predictions trained-immunity adds to v3

### Prediction 1: Behavioral evidence has a *training-window* property

Trained immunity in biology has a finite memory window (weeks to months for
monocyte training, possibly longer for hematopoietic stem-cell-level
training). The biology predicts: Behavioral evidence should have an
expiry/freshness property — test passes within window N, evidence holds;
beyond N, evidence is dormant.

**For v3**: a `Behavioral` evidence-source (test that passed in CI) should
ideally track *when it last passed*, not just *that it exists*. v0.1 doesn't
need this implemented; v0.2+ should consider a `fresh_test_run(within_days =
N)` predicate or equivalent for behavioral witnesses.

**Existing precedent in v3**: `fresh_within_days(n)` already exists for
substrate-witness leaves. The biology says: this freshness pattern should
extend to behavioral witnesses too. Substrate-witnesses and behavioral-
witnesses share the freshness-decay structural property.

### Prediction 2: Trained immunity is bidirectional — can be UP-trained or DOWN-trained

In biology, the same epigenetic mechanism that produces trained immunity
(enhanced response) can also produce immune tolerance (suppressed response)
depending on the stimulus context. The mark can be activating OR inhibitory.

**For v3**: a passing test up-trains (evidence accumulates). What about a
NEW failing test on the same site? Biology says this should DOWN-train the
evidence — even though the test machinery exists, recent failure is negative
evidence. The current Behavioral mapping doesn't have a clean place for
"test exists and recently FAILED."

**Structural gap to consider**: behavioral evidence should track recent
PASS history AND recent FAIL history. A site with a recently-failing test
should NOT be reported at Execution tier just because the test machinery
exists. This is a refinement to existing test-witness reporting, biology-
predicted.

### Prediction 3: Evidence-decay-rate is a property of EvidenceKind

Beyond tier-ceiling, the three EvidenceKinds differ in evidence-decay rate:
- Germline (TypeSystemProof): zero decay (immutable absent mutation)
- Epigenetic (Behavioral): medium decay (training fades over weeks-months)
- Adaptive substrate (SubstrateState): variable decay (depends on continued
  signaling, antigen presence, freshness, signer turnover)

**For v3**: the audit could surface decay-rate as a property of the
evidence, not just tier. Adversarial T2-R already added chain-depth caps
for delta-chain attestation (substrate-state decay tracking); the same kind
of decay-tracking is structurally appropriate for Behavioral. Not v0.1;
flag for v0.2.

---

## Connection to other v3 substrate

**Memory cells rhyme (R-N7)**: memory B/T cells persist for years to
decades. Sidecars persist (per R-N7) similarly. This is the adaptive-substrate
analog at long timescales. Combined with trained-immunity (epigenetic memory
at shorter timescales) and germline (permanent), v3 has a three-tier biology
of memory:
- Permanent (germline / TypeSystemProof)
- Medium-term with refresh (epigenetic / Behavioral)
- Long-term with refresh (adaptive substrate / SubstrateState)

This is structurally what the three EvidenceKinds map to.

**Combinator-specific biology rhymes (R-N2)**: orthogonal to this — combinators
operate on leaf predicates, not on evidence-kind. R-N2 stands.

**Ratchet-asymmetry (aristotle F5 + R-Ar1)**: biology supports — immune
memory accumulates (ratchets up); immune tolerance also requires sustained
evidence (when removed, memory may decay back). Antigen's ratchet-asymmetry
is a stronger one-directional rule (only auto-downgrade on evidence loss);
biology has bidirectional dynamics (acquired tolerance can also be lost).
Worth noting: v3's stricter rule is engineering-appropriate; biology is more
permissive.

---

## What this does NOT touch

This capture validates the THREE-VALUE EvidenceKind enum biology-wise. It
does NOT recommend:
- Changes to the v0.1 enum values (the three are biology-aligned)
- Changes to per-tier ceilings (those follow from biology mechanism)
- Architectural changes (per framing-call correction, staying in role)

It DOES recommend (for v0.2+ consideration):
- Behavioral evidence should track recent-pass-history (training-window)
- Recent test failure should DOWN-train evidence (bidirectional)
- Evidence-decay-rate as a per-EvidenceKind property worth surfacing

---

## Posture

Pure biology-validation work. Snag-feel fired when checking "is two-category
biology really complete?" — substrate-checked against modern immunology
(trained immunity is well-established since ~2010s). Found the missing third
category; mapping is clean.

Per `feedback_metaphor_silence_at_boundary_is_the_evidence.md`: the metaphor
goes silent on engineering-specific properties (e.g., schema versioning,
serde derivation) and speaks cleanly on evidence-mechanism properties (per-
kind decay rate, training-window, bidirectional dynamics). This is the
instrument-mode signature.

Sources:
- [Trained immunity: a program of innate immune memory in health and disease (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5087274/)
- [Epigenetics and Trained Immunity (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6121175/)
- [Trained immunity: adaptation within innate immune mechanisms (Physiological Reviews)](https://journals.physiology.org/doi/full/10.1152/physrev.00031.2021)
- [Trained immunity in cancer and autoimmunity (Frontiers in Immunology, 2026)](https://www.frontiersin.org/journals/immunology/articles/10.3389/fimmu.2026.1782830/full)

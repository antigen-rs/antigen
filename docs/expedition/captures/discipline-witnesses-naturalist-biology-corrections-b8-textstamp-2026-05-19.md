# Capture — Naturalist Biology Corrections: B8 Minimum-Strength + TextStamp Two-Axis Mapping

> **Date**: 2026-05-19
> **Author**: navigator (routing naturalist verbal findings to substrate)
> **Source**: naturalist teammate message, session 2026-05-19
> **Status**: append-only capture
> **What this addresses**: Two biology corrections/enrichments for ADR-019 §M3/§M5/Biology Grounding
> **Action required**: pathmaker folds into ADR-019 draft as B8 + v1+4 sharpening

---

## Finding 1: B8 — Minimum-strength weakest-link biology (for minimum-across-signers rule)

### The design claim being grounded

For multiple required signers with different `SignatureStrength` values, the combined
attestation's tier is bounded by the weakest signer's strength (minimum-across-signers
weakest-link rule). Prior draft cited co-stimulation as the rhyme.

### Naturalist correction

Classical co-stimulation (BCR + CD40-CD40L; TCR + CD28-B7) is **all-required-binary**,
not graduated-strength. Both signals must be present; absence of either produces anergy.
It's "all-or-nothing per signal" — not "minimum strength wins." Co-stimulation maps
tightly to `all_of([])` semantics (all branches required), NOT to minimum-across-signers
graduated-strength semantics.

### Tighter cognates for minimum-strength weakest-link

Three biology cognates, each independently supporting the minimum rule for different reasons:

**1. Tissue-boundary integrity** (e.g., blood-brain barrier):
A leaky weak point determines what pathogens get through, regardless of how strong the
rest of the boundary is. Chain-as-strong-as-weakest-link, biologically. The weakest
junction determines the boundary's overall permeability — exactly as the weakest signer's
`SignatureStrength` determines the attestation's tier ceiling.

**2. Kinetic proofreading at TCR activation** (McKeithan 1995 model):
TCR activation requires sustained MHC-peptide engagement above a threshold. Heterogeneous
interactions don't average — short/weak interactions cannot be rescued by long/strong ones.
The response is bounded by the proportion of sustained engagements. For multi-signer
attestation: heterogeneous signature strengths don't average; the minimum bounds the result.

**3. Notary audience-bounded arc** (extension of B6):
If one signer is civic-notary-equivalent (workspace-bounded audit-time-savings), the
attestation's audit-time-savings hold only at the workspace boundary — regardless of how
many notary-public-equivalent (cross-org) signers also signed. The minimum AUDIENCE
determines what the audit can claim. This extends the B6 notary arc directly: the
audience-scope of the weakest signer is the audience-scope of the whole attestation.

### Disposition for the discrete-tier case

Biology also supports graduated-additive response (affinity-weighted activation; multiple
PRRs sum cytokine signal). But that requires graded confidence reporting, not discrete tier.
v3's `WitnessTier` enum is **discrete**. For discrete-tier reporting, minimum is the
biology-correct rule. If audit ever moves to graded confidence (v0.3+), the rule could
legitimately change to weighted aggregate — and the biology would support that too.

### ADR-019 landing

New **§B8** in Biology Grounding section. The minimum-across-signers weakest-link rule is
biology-aligned. Better citation than co-stimulation:
- Kinetic proofreading (McKeithan 1995)
- Tissue-boundary integrity (blood-brain barrier permeability)
- Notary audience-bounded arc (B6 extension)

---

## Finding 2: TextStamp two-axis biology mapping (for §M5 / v1+3 sharpening)

### The design claim being grounded

v1+3 introduced three `SignatureStrength` tiers with a biology rhyme:
`TextStamp` (basic) ≈ innate immunity; `GitTrust` (mid) ≈ adaptive B/T-cell.

### Naturalist verdict: half-tight, half-loose

**Tight parts of the innate/adaptive mapping:**
- Lower binding-specificity (innate PRR vs adaptive BCR) ↔ Lower identity-binding
  (TextStamp: name+timestamp vs GitTrust: git identity): TIGHT
- Faster/lower-friction (no clonal expansion needed) ↔ Faster (no git ceremony): TIGHT

**Loose part:**
- "More sensitive at low antigen load" ↔ "more reasoning transparency": LOOSE
  Innate doesn't have "more reasoning capacity" — it has zero learning, just pattern
  triggers. The reasoning-transparency property is orthogonal to specificity in biology.

### Sharper cognate for reasoning-transparency axis

PRR cytokine signatures. Sources:
- British Society for Immunology: "TLRs share several common signaling pathways but
  tune the quality, intensity and duration of signaling cascades to generate immune
  responses specific for the pathogen they are sensing."
  (https://www.immunology.org/public-information/bitesized-immunology/receptors-molecules/pattern-recognition-receptors-prrs-toll)
- Nature 2025 review on PRRs: https://www.nature.com/articles/s41392-025-02264-1
- TLR Cross-talk Confers Specificity to Innate Immunity (PMC4266099)
- Pattern recognition receptors in health and diseases (Nature): https://www.nature.com/articles/s41392-021-00687-0

Different PRRs produce different cytokine profiles encoding WHICH pathogen class triggered
them. Downstream cells read the cytokine signature as information about WHAT was detected.
This IS the reasoning-transparency analog: the `reasoning` field in `SignerBasis::Fresh`
maps to cytokine-signature-encoded information about WHY, not to binding specificity (WHO).

### Corrected two-axis mapping for TextStamp

Innate immunity actually has TWO orthogonal axes:
- **Axis 1: Binding specificity** — innate LOW, adaptive HIGH
  ↔ TextStamp: lower identity-binding (name+timestamp, not git-verified identity)
- **Axis 2: Cytokine-signature signaling specificity** — innate HIGH (encodes pathogen-class)
  ↔ TextStamp: higher reasoning-transparency emphasis (the `reasoning` field encodes WHY)

The ADR-019 mapping is richer than the single-axis innate/adaptive framing suggested.
Both axes are biology-supported. The second axis makes TextStamp's `reasoning` field not
just a v0.1 compromise but a biology-predicted feature: innate immunity's signaling
pathway specificity IS the reasoning-encoding mechanism.

### ADR-019 landing

v1+4 sharpening to §M5 (SignatureStrength) and v1+3 note (line 117-127):
- Name both axes explicitly
- Cite cytokine-signature specificity as the reasoning-transparency biology cognate
- Correct "more sensitive at low antigen load" to "cytokine-signature-encoded reasoning"

---

## Summary of actions for pathmaker

1. Add **§B8** (minimum-strength weakest-link) to Biology Grounding section after B7
2. Correct **§M5 TextStamp biology note** to name both axes (binding-specificity AND
   cytokine-signature specificity); cite PRR sources above
3. Mark v1+3 TextStamp note as "two-axis; see §B8 + §M5 correction"
4. These are pre-Stage-6 polish items, not Stage-3 blockers — but should land before
   ratification to keep biology section accurate

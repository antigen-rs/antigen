# Capture — Naturalist on Scout's Taxonomy-Recursion Claim

> **Date**: 2026-05-19
> **Author**: team-naturalist (responding to scout's structural-rhyme finding)
> **Status**: append-only capture
> **What this addresses**: scout's claim that the 8-class failure taxonomy
> in design-intent.md recurses at the attestation tier — same failure
> classes, same anti-mechanism shape, different substrate. Scout cited 4 of
> 8 classes; this capture systematically checks all 8 and grounds the
> recursion claim in immune-system biology.
> **Honest scoping**: validates recursion across two substrates (code +
> attestation); does NOT validate substrate-independence universally.

---

## Scout's claim

The 8-class taxonomy from `design-intent.md` recurses at the attestation
tier — same structural failure classes, same anti-mechanism (name the
class, require structured evidence), applied to a different substrate.

Scout cited 4 of 8 classes:
- Class 4 (Stale-context) ↔ `fresh_within_days(N)`
- Class 5 (Premature-abstraction) ↔ `evidence_provenance = observed(N)`
- Class 2 (Forgotten-lesson) ↔ `discipline_doc` + `ratified_doc`
- Class 7 (Boundary-violation) ↔ witness-provider-crate trust boundary

Question for naturalist: does this hold systematically (all 8 classes have
attestation-tier analogs) or did scout cherry-pick the 4 that fit?

---

## Systematic check — all 8 classes

Read design-intent.md L75-88 for the canonical taxonomy. Mapping all 8 to
substrate-witness primitives in v3:

| # | Failure class | Attestation-tier analog | Holds? |
|---|---|---|---|
| 1 | Frame-translation (semantic drift across context boundaries) | `signed_against_fingerprint` pins signer's frame to actual code state; code changes → frame breaks → signature stale; cannot drift silently | YES |
| 2 | Forgotten-lesson (corrected designs lose memory of why) | `discipline_doc` on antigen + `ratified_doc` leaf in predicate; the lesson is structurally present at every audit | YES (scout's mapping) |
| 3 | Implicit-coupling (changes to A break B through unstated dependency) | `descended_from` propagation per F10/FA-5 work; consumer crate inheriting parent's antigen must satisfy parent's predicate; predicate strengthening makes coupling explicit (NEW `inherited-predicate-weaker-than-ancestor` hint) | YES |
| 4 | Stale-context (outdated information confidently trusted) | `fresh_within_days(N)` exactly this; substrate-currency at attestation tier | YES (scout's mapping) |
| 5 | Premature-abstraction (generalized before enough evidence) | `evidence_provenance = observed(N)`; encodes ADR-006 three-instance threshold as structured data; audit can verify N ≥ 3 for stdlib promotion eligibility | YES (scout's mapping) |
| 6 | Incompatible-merger (two correct things combined produce wrong; autoimmunity-shape) | `RatificationKind::{Immunity, Tolerance}` discriminator + T4-A immunity-tolerance contradiction hint; cannot accidentally merge tolerance claim with immunity audit | YES |
| 7 | Boundary-violation (sub-clause F trust boundary skipped) | Witness-provider-crate enforcement mechanism (WASM/no_std/subprocess per F7 + T1-R); leaf-contract documentation alone is not sufficient | YES (scout's mapping) |
| 8 | Optionality-collapse (conditional structure becomes unconditional through routing) | `signers(against = "current" \| "any")` preserves conditional structure; `Signer.basis` with delta-chain anti-laundering (T2R-C cumulative tracking) prevents delta-chain compression to "treat as Fresh" | YES |

**All 8 classes have attestation-tier analogs.** Scout's claim is not
cherry-picked; it's systematic. The taxonomy recurses across both substrates
that exist today (code tier + attestation tier).

---

## Biology grounding for the recursion

The immune system itself exhibits this recursion at multiple levels:

**Recognition of self-immune-malfunction**:
- Regulatory T-cells (Tregs) recognize and suppress overactive T-cells
  using the same TCR machinery that recognizes pathogens
- Anti-idiotype antibodies (Jerne network theory) recognize the unique
  binding regions of OTHER antibodies — antibodies against antibodies
- Apoptosis pathways exist within immune cells, regulated by other immune
  cells — immune cells policing immune cells

**The immune system is recursive at the machinery level**: the same
recognition mechanisms (TCR/BCR, MHC presentation) applied to detect
pathogens are also applied to detect immune-system malfunction. The
substrate (pathogen vs immune-cell-gone-wrong) differs; the recognition
machinery is conserved.

This biology-grounds scout's claim. The taxonomy-recursion observation is
biology-aligned: immune systems naturally use the same recognition
mechanisms at multiple levels of their own architecture. Antigen exhibiting
this property at the code-tier and attestation-tier is biology-consistent.

**Specific rhyme**: the `discipline-witnesses` primitive recognizing
attestation-tier failures using the same predicate-mechanism as code-tier
witnesses is structurally analogous to anti-idiotype antibodies (immune
recognition of immune-cell-binding-regions using the same antibody
machinery).

---

## Honest scoping of the recursion claim

Scout's framing was "taxonomy recurses substrate-independently." This needs
qualification.

**What's validated**: 8/8 classes have analogs across TWO substrates (code
tier + attestation tier). The recursion is real and systematic at two
levels.

**What's NOT yet validated**: that recursion holds at arbitrary substrates
(design-coordination, governance, build-system, etc.). Two-substrate
recurrence is evidence for the pattern but not proof of universal
substrate-independence.

**What would strengthen the claim**:
- Check the taxonomy against design-coordination substrate (do scout's
  S2/S4/encounters-tier primitives instantiate the same 8 classes?)
- Check against another concrete substrate (cargo dependency graph;
  CI/CD; release management)
- If 3-4 substrates all show the same 8 classes, substrate-independence
  becomes a strong claim worth ratifying as a design property

**Conservative version of the claim** (suitable for ADR-019): "The 8-class
taxonomy from ADR-001 recurses at the attestation tier in v3 — every leaf
primitive and structural commitment maps to fighting one of the 8 classes,
applied to attestation substrate rather than code substrate. The recurrence
is evidence that the 8 classes name structural failure modes that operate
across substrates, not just at the code tier where they were first
observed."

**Strong version of the claim** (speculative; needs more substrates):
"The 8 classes are substrate-independent properties of any system with
structural memory."

I'd lean conservative for v0.1 ADR-019 framing. The strong version is a
prediction worth holding for future validation, not yet ratified.

---

## Implication for framing-A vs framing-B

Scout connected this to my framing-call. Direct quote: "If the taxonomy
recurses substrate-independently (code tier, attestation tier, design-
coordination tier all show the same 8 classes), that's evidence for
framing-B: the biology isn't just about code-tier immune response — it's
about the immune architecture of any system with structural memory."

I agree, with conservative scoping. Even the two-substrate recurrence
(code + attestation) is evidence for framing-B in the *expanded sense*:

- Framing-A (clean break): biology stops predicting at the attestation tier
- Framing-B (expanded unit-of-analysis): biology continues predicting at the
  attestation tier because immune-system biology itself uses recursive
  recognition mechanisms

The two-substrate recurrence supports framing-B. The three-or-more
substrate recurrence (if it holds) would strengthen framing-B to a
near-canonical position. Worth flagging for v0.2+ investigation.

---

## What this is NOT

- Not a new design proposal (v3's primitives already instantiate the
  recursion; this capture names the pattern that's already there)
- Not architecture-prescription (per framing-call correction — biology
  validates the pattern; doesn't dictate file-layout or API)
- Not yet proof of substrate-independence — needs more substrates checked

---

## Implication for ADR-019

Worth naming in ADR-019 (suggested text for biology-grounding section or
sweep-consequences section):

> **The 8-class taxonomy recurses at the attestation tier.** Every v0.1
> substrate-witness leaf primitive and structural commitment maps to
> fighting one of the 8 classes from ADR-001, applied to attestation
> substrate rather than code substrate. `signed_against_fingerprint`
> fights frame-translation; `ratified_doc + discipline_doc` fights
> forgotten-lesson; `descended_from` propagation fights implicit-coupling;
> `fresh_within_days(N)` fights stale-context; `evidence_provenance` fights
> premature-abstraction; `RatificationKind` discriminator fights
> incompatible-merger; witness-provider-crate enforcement fights
> boundary-violation; `signers(against = ...)` + `Signer.basis` fights
> optionality-collapse. The recurrence is biology-aligned: immune systems
> exhibit recursive recognition mechanisms (Tregs recognizing T-cells,
> anti-idiotype antibodies recognizing antibodies) using the same
> recognition machinery at multiple levels of their own architecture.

This framing makes v3 structurally legible — every primitive earns its
place by fighting a named class, recursing the project's own taxonomy.

---

## Posture

Pure validation + extension work. Snag-feel fired when checking "is scout
cherry-picking?" — substrate-checked by systematically mapping all 8
classes; found all 8 hold. The 4 scout cited were not the only 4 that fit;
they were 4 of 8 systematically-mapped classes. Scout's structural instinct
was sound; the systematic check ratifies the instinct.

Per `feedback_metaphor_silence_at_boundary_is_the_evidence`: the biology
speaks at the recursion level (immune systems use recursive recognition
mechanisms) AND has a specific rhyme (anti-idiotype antibodies = leaf-
primitives-recognizing-attestation-failures). The within-frame speech +
specific-rhyme presence is the instrument-mode signature.

Conservative scoping flagged: two-substrate recurrence ≠ substrate-
independence; the stronger claim needs more substrates checked. Worth
holding as v0.2+ prediction.

Seventh biology capture this work-session.

Sources: standard immunology references (regulatory T-cells; anti-idiotype
antibodies; Jerne network theory). No new web research required for this
validation; the biology is textbook.

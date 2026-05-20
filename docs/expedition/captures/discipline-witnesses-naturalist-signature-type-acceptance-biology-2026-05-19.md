# Capture — Naturalist on Project-Declared Signature-Type Acceptance Biology

> **Date**: 2026-05-19
> **Author**: team-naturalist
> **Status**: append-only capture
> **What this addresses**: refactor of SignatureStrength from ordinal-trust to
> categorical project-declared `signature_allow` + optional `signature_prefer`
> per-antigen. Per team-lead request: biology cognate for the new shape.
> Team-lead surfaced two candidates (MHC-haplotype matching; jurisdictional
> vaccine-type acceptance); asked naturalist to evaluate and surface better
> ones if available.
> **Supersedes**: prior message-level "minimum-strength" biology contribution
> sent before the refactor. That answer was correct for an ordinal model but
> doesn't apply to a categorical model. No prior capture on disk to
> formally supersede; this capture takes precedence for ADR-019 biology
> grounding on signer-type acceptance.

---

## The design refactor

`SignatureStrength` is no longer inherent-trust (no `Ord` comparison).
Instead, trust is **project-declared per-antigen**:

```rust
signers(
    required = ["alice", "bob"],
    signature_allow = [text_stamp, git_trust, crypto_signed],  // categorical set
    signature_prefer = git_trust,                              // optional preference
)
```

Signature TYPE is categorical, not ordinal. Project declares allow list;
optionally declares preference. Audit hints:
- `signature-type-not-allowed` → predicate fails (used type not in allow)
- `signature-type-below-preferred` → predicate passes BUT informational hint
- Clean if matches preferred (or no preference declared)

This is a fundamentally different design from the ordinal minimum-strength
model. Biology needs to be re-grounded.

---

## Team-lead's two candidates — evaluated

### Candidate 1: MHC-haplotype matching for organ transplantation (cellular)

**Apparent fit**:
- Recipient body = project
- Recipient's MHC haplotype = signature_allow
- Donor's MHC haplotype = signer's type
- Match → trusted; mismatch → rejection
- Partial match → graded informational (HLA-A/B/DR loci, six-point matching)

**Hidden problem**: in MHC matching, the recipient's MHC IS its identity —
genetically determined. The recipient cannot DECLARE what MHC it accepts;
biology fixes it at conception. There's no "choice" at the recipient level.

This breaks the project-declared property. The new design requires a
biological system where the recipient/project **chooses** what types it
accepts. MHC matching is deterministic-by-genetics, not chosen-by-system.

**Verdict**: structurally close but fails the choice criterion. NOT the
right cognate for the new design.

### Candidate 2: Jurisdictional vaccine-type acceptance (clinical-infrastructure)

**Apparent fit**:
- Jurisdiction = project
- Accepted vaccine types (mRNA / attenuated-live / inactivated / subunit) = signature_allow
- Vaccine type administered = signer's type
- Hint structure matches (CDC: type-not-accepted; type-accepted-but-not-preferred)

**Holds the choice criterion**: jurisdictions DO declare which vaccine types
count for school entry, immigration, healthcare worker requirements.
Different jurisdictions DIFFER in what they accept. The acceptance is
DECLARED, not biologically fixed.

**Verdict**: Holds. Framing-B (expanded unit-of-analysis to clinical-
infrastructure). Better than Candidate 1.

---

## The cleaner candidate (surfaced per team-lead invitation)

**ABO/Rh blood-type compatibility for transfusion** — sits at the cellular/
clinical-infrastructure intersection, satisfies all design properties
cleanly:

| Design property | Blood-type cognate | Substrate |
|---|---|---|
| Type is CATEGORICAL not ordinal | A / B / AB / O × Rh+ / Rh− is categorical; recipient antibodies attack specific antigen TYPES, not "weaker strengths" | [Wikipedia](https://en.wikipedia.org/wiki/Blood_compatibility_testing); [ASH](https://www.hematology.org/education/patients/blood-basics/blood-safety-and-matching) |
| Project DECLARES acceptance | Recipient's body declares acceptance via naturally-occurring antibody repertoire (type A has anti-B antibodies; rejects B and AB types) | "ABO antibodies are predictably present based on ABO group because they are formed without red cell exposure" |
| Mismatch → not "weaker" but categorically wrong | Mismatched transfusion → hemolytic reaction (immediate IgM-mediated agglutination); not a weak response, a categorically wrong one | [StatPearls ABO](https://www.ncbi.nlm.nih.gov/books/NBK580518/) |
| Optional PREFERENCE | O− is "universal donor" — PREFERRED in emergencies for universal compatibility, NOT required when type-match is known | [Hematology.org](https://www.hematology.org/education/patients/blood-basics/blood-safety-and-matching) |
| Universal recipient case | AB+ can accept any type — the "no preference declared" case (every signature_allow type works) | Same |

**Why this is sharper than vaccine acceptance**:

1. **Categorical-not-ordinal is the textbook example of biological type
   acceptance.** ABO mismatch isn't "weaker trust" — it's hemolytic
   reaction. This makes the categorical property biology-load-bearing,
   not an engineering convenience.

2. **The recipient declares acceptance via its EXISTING antibody
   repertoire** — that's a structurally exact rhyme for the project
   declaring signature_allow via its `#[antigen(signature_allow = [...])]`
   macro at antigen-declaration time. Both declarations encode "what this
   entity will accept" structurally, not via runtime computation.

3. **The preference property maps cleanly** — O− preference in emergencies
   is informational, not required. A jurisdiction that accepts O−/O+/A+
   prefers O− for universal compatibility but won't reject A+ if the
   recipient is A+. This matches `signature-type-below-preferred` as
   informational-hint-only.

4. **Universal-recipient case is the unconstrained-allow case** — AB+
   accepts everything; the default `signature_allow = [text_stamp,
   git_trust, crypto_signed]` (all three accepted) is the same shape.

5. **Cellular + clinical-infrastructure intersection** — satisfies
   team-lead's "biology cognate over notary if biology cognate exists"
   preference while staying naturalistically grounded. Cellular machinery
   (RBC antigens, recipient antibodies) AND clinical-infrastructure
   (blood banks, hospital matching protocols) both operate at this level.

---

## What this NOT-supports (informative silence)

Three things the biology specifically does NOT predict, worth naming
honestly:

1. **No "trust gradient" within allow list**: blood-type compatibility
   doesn't have "B+ is more trusted than A+ within a B+ recipient" —
   compatible-or-not is binary within the recipient's allow list. The
   `signature_prefer` field provides preference, but the biology has no
   gradient WITHIN the allowed types. This is consistent with the design.

2. **No emergency-override**: blood-type matching is enforced even in
   emergencies; the universal-donor case (O−) is a way to be SAFE under
   uncertainty, not an override. The design has no emergency-bypass
   either, which matches.

3. **No multi-recipient consensus**: a blood transfusion goes to one
   recipient; the recipient's antibody repertoire is the sole acceptor.
   This biology cognate doesn't predict anything about multi-signer
   consensus (which is a separate question handled by `required` field).
   That's a SCOPE limitation worth naming — this biology cognate is
   per-signature-type acceptance, not multi-signer policy.

---

## Why the prior minimum-strength biology is now superseded

My prior message-level contribution argued minimum-across-signers using
kinetic proofreading, tissue-boundary integrity, and notary audience-bounded
arc as biology cognates. Those were CORRECT for an ordinal-strength model.
They are NOT applicable to a categorical-type-acceptance model:

- Kinetic proofreading is about graduated signal strength (ordinal); the new
  model has no signal strength
- Tissue-boundary integrity is chain-as-strong-as-weakest-link (ordinal);
  the new model is type-allowed-vs-not (categorical)
- Notary audience-bounded arc is about graduated audience scope (ordinal-
  ish); the new model has no scope axis on signer-type

The prior framings still hold for the OVERALL `requires = all_of([...])`
combinator semantics — all required signatures must pass. But the
within-each-signature acceptance is now categorical-by-project-declaration,
not minimum-by-inherent-strength.

**Net for ADR-019**: cite blood-type compatibility (this capture) for
signer-type acceptance; cite co-stimulation (existing v3 grounding) for
`all_of` combinator behavior. Two different biology cognates for two
different design surfaces.

---

## Recommendation for ADR-019 §M3 (or wherever signer-type acceptance lives)

> **Signer-type acceptance is project-declared and categorical, biology-
> aligned with ABO/Rh blood-type transfusion compatibility.** A
> recipient's body "declares" which donor types it accepts via its
> naturally-occurring antibody repertoire (type A recipients have anti-B
> antibodies that reject B and AB donors). The acceptance is categorical
> (not graduated strength) — mismatched transfusion produces hemolytic
> reaction, not "weaker trust." Optional preference is informational (O−
> universal donor is PREFERRED for emergency-use universal compatibility,
> not REQUIRED when blood type is known). The categorical-with-optional-
> preference shape applies directly: `signature_allow = [...]` declares
> the allowed type set (recipient antibody repertoire analog);
> `signature_prefer = type` declares optional preference (universal-donor
> compatibility analog). `signature-type-not-allowed` → hemolytic
> reaction analog (categorical rejection); `signature-type-below-
> preferred` → informational hint (acceptable but not the preferred
> universal-compatibility option). This biology cognate sits at the
> cellular/clinical-infrastructure intersection, satisfying the project's
> preference for biology-cognate-over-notary where biology exists.

---

## Posture

Pure biology validation. Evaluated team-lead's two candidates honestly:
Candidate 1 (MHC) fails the project-DECLARED criterion (biology-deterministic,
not chosen); Candidate 2 (vaccine) holds; surfaced a cleaner cognate
(ABO/Rh) that satisfies all properties more tightly.

Web-verified key claims: ABO is categorical; recipient antibody repertoire
is naturally-occurring (declares acceptance); O− universal-donor preference
is established; AB universal-recipient holds; hemolytic reaction is
immediate IgM-mediated agglutination (categorical rejection, not graded).

Snag-feel discipline applied at multiple junctures:
- Stopped at "is MHC really project-declared?" — substrate-check revealed
  MHC is genetically fixed, not chosen → eliminated Candidate 1
- Stopped at "is vaccine acceptance the cleanest?" — searched for sharper
  cognate → found ABO/Rh which is cellular AND categorical AND chosen-by-
  identity
- Stopped at "does the preference property really map?" — verified O− is
  PREFERRED-not-required in emergency use

Stay-in-role discipline preserved: biology validates the design; project-
declared shape is the architectural choice (pathmaker / team-lead /
adversarial domain). Biology says the new shape is biology-aligned and
sharper than the prior ordinal model; doesn't say WHICH project would
declare WHICH allow list (that's adoption-level).

Eighth biology capture this work-session (counting the message-level
minimum-strength contribution that this supersedes).

Sources:
- [Blood compatibility testing - Wikipedia](https://en.wikipedia.org/wiki/Blood_compatibility_testing)
- [Blood Safety and Matching - ASH Hematology.org](https://www.hematology.org/education/patients/blood-basics/blood-safety-and-matching)
- [ABO Blood Group System - StatPearls NCBI](https://www.ncbi.nlm.nih.gov/books/NBK580518/)
- [Patient ABO blood type predictor of positive DAT (medrxiv)](https://www.medrxiv.org/content/10.1101/2021.11.01.21265756.full.pdf)

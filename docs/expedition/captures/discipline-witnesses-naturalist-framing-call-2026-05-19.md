# Capture — Naturalist Framing Call on Discipline-Witnesses (Inline vs Sidecar)

> **Date**: 2026-05-19
> **Author**: team-naturalist (post team-launch; building on prior single-instance
> naturalist self-pass [discipline-witnesses-naturalist-self-pass-2026-05-18.md])
> **Status**: append-only capture
> **What this resolves**: navigator's Phase-1 pivot question (campsite note
> 18:24:20) — "inline-vs-sidecar IS framing-A vs framing-B at concrete level.
> Does biology treat 'memory' as persisting in the lymph node (sidecar) or as
> the antibody itself (inline)?"
> **Trigger**: tambear adoption log entry 2026-05-18 sketched inline
> `witness = doc_attested(doc, attested_by, at, rationale)` shape. Team
> converged on framing-A vs framing-B as the underlying architectural call.
> Naturalist was asked for the biology answer.

---

## The call

**Framing-B (expanded unit-of-analysis) is correct, and the biology is more
specific than the prior naturalist pass acknowledged.**

The biology does not just predict that record-trust exists in
immunology-as-clinically-embedded. It predicts the **separation between
recognition-machinery and evidence-of-attestation** as a fundamental design
property.

**Inline-vs-sidecar is not a choice; it is a TYPE distinction. Both
presentations are necessary; collapsing them violates the biology.**

---

## Why the prior framing was incomplete

Prior naturalist self-pass (R-N6) leaned framing-B over framing-A on the grounds
that biology-as-clinically-embedded has central registries (immunization records,
vaccination boosters as manual re-attestation). That's correct as far as it
goes — but it left the inline-vs-sidecar question underspecified, because it
treated "biology has records" as a global property without asking *where in the
cellular machinery the records actually live*.

When the question shifted from "is biology aware of attestation?" to "WHERE
does biology put attestation relative to recognition?", a sharper answer
became available.

---

## Biology check — does the metaphor go silent or speak cleanly?

Per `feedback_metaphor_silence_at_boundary_is_the_evidence.md`: a metaphor
operating as instrument produces dense predictions inside its domain and
silence at the boundary. Here, the metaphor produced a dense, specific,
biology-substrate-grounded answer with a sharp prediction — *and* it produced
silence at the boundary I tried to extend it to (inline-degenerate form). Both
the speaking and the silence are diagnostic.

### Memory B-cell vs plasma cell — the load-bearing structural distinction

A memory B-cell:
- Carries the BCR (B-cell receptor) on its membrane
- The BCR encodes *what it binds* — specificity
- Is long-lived and quiescent
- Re-activates on antigen re-encounter

A plasma cell:
- Secretes free antibody (soluble, circulating)
- The secreted antibody is the *evidence-of-prior-activation*
- Has its own lifetime (most short-lived; LLPCs in bone marrow long-lived)
- Cannot itself respond to secondary exposure — that's the memory cell's job

**These are physically and functionally distinct structures:**
- Recognition specificity lives on the cell surface (BCR)
- Evidence-of-activation lives in soluble form (secreted Ig) or as cellular
  history (somatic hypermutations in DNA, surface markers like CD27,
  class-switched constant region)
- Recognition machinery is NEVER colocated with attestation-evidence inside
  the same molecular structure

This separation is biologically load-bearing. It's why you can re-activate
without re-engineering the BCR; why multiple activation events can produce
distinct evidence; why the cell's identity is stable across many activations.

### Mapping to inline vs sidecar

**Inline `requires = ...` attribute** = **BCR on memory cell**
- IS the cell's recognition identity
- Defines what specificity the code claims
- Structurally bound to the code (changing the immune claim requires changing
  source)
- Visible in PR diff because it IS source
- Doesn't drift; doesn't expire; same predicate evaluated each time

**Adjacent `.attest/` sidecar** = **secreted antibody / cellular history**
- IS the evidence-of-attestation
- Carries who/when/against-what/rationale per attestation event
- Has its own lifetime (signatures stale; fingerprints drift; cumulative-diff
  caps)
- Can be re-issued (re-attest) without changing recognition specificity
- Multiple signers can accumulate independently

---

## Substrate-grep result (per discipline)

Per `feedback_grep_decisions_before_design_answer.md`: before routing a clean
biology answer, grep `docs/decisions.md` for the type being designed.

- `grep -nE "(sidecar|adjacent|inline|locality)" docs/decisions.md` returns
  no ratified architectural commitment on inline-vs-sidecar.
- The germinal-center reframe (capture 2026-05-18 turn 7, Tekgy-driven)
  ratified **code-locality** (sidecar adjacent to code, not in central
  registry). It did NOT settle inline-vs-sidecar within code-locality.
- v3 draft is the substrate where sidecar-pattern lives, but v3 is explicitly
  pre-ratification.

**Substrate is silent on inline-vs-sidecar within code-locality.** Biology is
in prediction-mode, not argument-mode. Snag-feel fired (and was honored) at the
moment of considering "is biology really qualified to answer this?" — the
substrate-grep confirmed it is.

---

## The inline-degenerate form (and why biology rejects it)

I considered proposing a third form: an inline-degenerate sidecar for tambear's
case (one signer, frozen, historical attestation lives directly in the
attribute):

```rust
// PROPOSED THEN REJECTED:
#[immune(X, witness = doc_attested(
    doc = "...",
    attested_by = "math-researcher",
    at = "2026-05-18",
    rationale = "audited under Sub-pattern 5.11..."
))]
```

This is tambear's actual sketch. It's appealing because:
1. Single-attestation case is simple
2. PR-diff visibility is immediate
3. Avoids the sidecar file overhead for one-shot cases

**Biology check before routing**: does biology have an inline-degenerate
attestation form?

Web-checked memory B-cell vs plasma cell biology
([Wikipedia](https://en.wikipedia.org/wiki/Memory_B_cell);
[Technology Networks](https://www.technologynetworks.com/immunology/articles/b-cells-memory-b-cells-and-plasma-cells-b-cell-activation-development-and-the-b-cell-receptor-384316);
[Frontiers in Immunology](https://www.frontiersin.org/journals/immunology/articles/10.3389/fimmu.2019.01787/full)).

**No. Biology never colocates attestation-data with recognition-machinery.**

Even long-lived plasma cells (LLPCs) — which are the cellular analog of "one-shot
historical attestation" — are structurally separate from the recognition
machinery. They secrete antibody for decades from a single activation event,
but they are SEPARATE CELLS from the memory B-cell carrying the BCR. The
attestation-evidence is in soluble form (secreted Ig in serum, deposited in
bone marrow niches); the recognition machinery is on the memory B-cell membrane.

This is the metaphor speaking, not going silent. The biology has a *specific
no* to inline-attestation: separation between specificity-encoding and
evidence-encoding is biologically load-bearing. It enables re-activation
without re-engineering; it enables multiple activation events with separate
evidence; it keeps cell identity stable across many activations.

**Predicted failure modes if inline-attestation is adopted:**
1. Can't re-attest without rewriting the recognition-claim (analog: can't
   re-activate without re-engineering the BCR)
2. Can't have multiple attestations (analog: can't have multiple plasma cells
   producing different antibody batches)
3. Conflates cell-identity with cell-history (analog: would force re-deriving
   the BCR each time a new activation event happens)

---

## Resolution for tambear's use case

Tambear's `doc_attested` use case is REAL. The workflow is right. Only the
inline shape is structurally wrong.

The biology-aligned resolution:

```rust
// Inline (the cell's identity): recognition specificity
#[immune(X, requires = all_of([
    ratified_doc(path = "docs/methodology/sub-pattern-5-11.md"),
    signers(required = ["math-researcher"]),
]))]
fn anchored_recipe(...) { ... }
```

Combined with CLI ergonomics for one-shot anchor case:

```sh
# One CLI invocation creates the sidecar with rationale + signer + date,
# pinning to current fingerprint. No editing required after declaration.
cargo antigen attest scaffold-anchor \
    --file src/recipes.rs --antigen X --item anchored_recipe \
    --signer math-researcher \
    --doc docs/methodology/sub-pattern-5-11.md \
    --rationale "audited under Sub-pattern 5.11 + Pattern 23 Type-1; \
                 dispatch shoulder verified Chebyshev-optimal at x=5; \
                 antibody battery includes envelope-max canary"
```

The `scaffold-anchor` subcommand (NEW; not in current v3 CLI surface) is the
ergonomic shortcut. It does the work of `scaffold + sign + (optionally) freeze`
in one command. Adoption-friction is comparable to inline-attestation; the
sidecar file lives where biology says it should.

---

## Observer's "rationale-visibility gap" — resolved under this frame

Observer flagged (campsite note 18:25:07): inline rationale is visible in PR
diff; sidecar rationale is auditable but not source-diff-visible.

Under biology-aligned frame, this is NOT a gap — it's a **discoverability
question**:

- Sidecar files DO live in the PR diff. They're regular files; PRs touching
  source files touch sidecars; reviewers see both.
- The "rationale not visible at the call site" is true but mirrors biology
  exactly: an antibody's binding rationale (somatic hypermutation history) is
  not visible at the BCR — you have to look at the cell's lineage record.
  Biology accepts this separation because separation enables independence.
- If reviewer-cognitive-load is the real concern (which it might be), the
  resolution is CLI/IDE ergonomics (hover-shows-sidecar-content; PR-diff
  rendering shows sidecar inline-with-source) — NOT moving the rationale into
  the source attribute.

---

## Implications for v3 → ADR-019

**Locked architectural commitments (post-naturalist-call)**:

1. **Recognition specificity is inline** — `requires = ...` predicate on
   `#[immune(...)]` attribute. Structurally bound to source. Defines what the
   code claims immunity to.

2. **Evidence-of-attestation is sidecar** — `.attest/X.json` adjacent to
   source. Carries who/when/against-what/rationale. Evolves through
   re-attestation events.

3. **There is no inline-attestation form.** No leaf primitive accepts
   attestation data (signer-name, date, rationale, doc-link) as inline
   arguments. The biology specifically forbids this colocation.

4. **CLI ergonomics make one-shot anchor cases painless.** `attest
   scaffold-anchor` (NEW — propose for v0.1) combines scaffold + sign +
   pre-populated rationale in one invocation. This is the workflow tambear
   needs; the sidecar is where the data should live.

**Framing-B (expanded unit-of-analysis) is sharpened**:

The prior framing-B statement ("biology-as-clinically-embedded has record-trust")
was correct but underspecified. The sharpened version:

> Biology-as-clinically-embedded has record-trust, AND biology specifically
> separates recognition-machinery from evidence-of-attestation as a fundamental
> design property. Both presentations exist; they are not alternatives but
> distinct verification surfaces with different lifetimes, different update
> semantics, and different visibility properties. Antigen inherits this
> separation; collapsing it violates the metaphor.

**ADR-019 text changes**:

ADR-019 should add a section (or expand the "Mechanics" section) explicitly
naming the separation. Suggested heading: "Recognition-specificity-inline,
evidence-of-attestation-sidecar: a biology-aligned type distinction." Cite
this capture; cite memory-B-cell vs plasma-cell biology; mark this as a
load-bearing architectural commitment in the same tier as code-locality
(which it extends, not replaces).

**Tambear adoption-log followup**: the inline `doc_attested(...)` proposal
should be re-expressed in the next tambear-adoption-log entry as the canonical
sidecar form + `attest scaffold-anchor` CLI invocation. The semantic intent is
preserved; the mechanical shape changes.

---

## What's still open

- **R-N5 (peripheral tolerance backup)** — biology has a backup mechanism for
  trust-once at thymic-education layer. Software analog: ongoing dependency
  audit. This was already in the prior naturalist pass; it's not affected by
  this framing call, but it remains as substrate for the cross-crate
  `descended_from` discussion (T8).
- **Memory-cell rhyme deepening (R-N7)** — properties of memory B/T cells
  (long-lived, isotype switching, rapid re-activation) may predict properties
  of antigen sidecars beyond persistence. Worth a separate pass; not blocking
  the framing call.
- **F3 scope biology** — does biology have a discipline-scope axis
  (cellular/tissue/organ/systemic)? Aristotle flagged this for naturalist
  pass. Worth a separate pass; not blocking.
- **F8 EvidenceKind biology validation** — innate-vs-adaptive immunity mapping
  at the machinery level (R-N1). Worth deeper validation; not blocking the
  framing call.
- **Notary-institution 800-yr arc** (scout S4) — naturalist should verify
  against actual notary history. Worth separate pass; not blocking.

---

## Posture

This call is appropriate to land in ADR-019 text and in v3 as a structural
refinement. It does NOT require a new draft version; it's an in-place
sharpening of the architectural commitments v3 already carries (per the
forward-only update convention in INDEX).

Per `feedback_clean_without_snag_is_argument_mode.md`: snag-feel fired at the
right moments (when extending the metaphor to inline-degenerate; when
considering whether biology actually predicts this). Both times, substrate-
verification confirmed the snag was correct. Biology speaks in prediction-mode
here, not argument-mode.

---

## Correction — appended 2026-05-19 after observer NB003 + NB004

Observer (NB003) caught a structural conflation in this capture: it treats
framing-A/B (biology-positioning question) and inline-vs-sidecar
(information-architecture question) as the same question. They are not.

Re-evaluating my own capture:
- **Claim that biology distinguishes recognition-machinery from
  evidence-of-attestation**: this survives. Web-verified; load-bearing;
  correct.
- **Claim that biology therefore mandates separate sidecar location**: this
  overreaches. The biology speaks at the role-distinction level (recognition
  vs evidence), not at the file-layout level. Multiple architectural options
  carry the role-distinction; sidecar is one, but a hybrid (sidecar carries
  signers/fingerprints/dates + macro carries inline `rationale`) ALSO carries
  the role-distinction with rationale colocated at the call-site for PR
  visibility.

The biology call (corrected):
> **Framing-B (expanded unit-of-analysis) is correct.** The biology predicts
> a role-distinction between recognition-specificity and evidence-of-
> attestation. This distinction is load-bearing for the design.
>
> **Biology does NOT prescribe the file-layout architecture** that carries
> this distinction. Whether evidence lives inline, in sidecar, or hybrid is
> an information-architecture question owned by pathmaker (syntax
> feasibility), adversarial (bypass analysis), aristotle (schema analysis).
> The biology constrains the *kind* of evidence (per-attestation-event,
> evolving, with lifetime) but not its *storage location*.

The architectural commitment I proposed ("recognition-specificity-inline,
evidence-of-attestation-sidecar") is one valid implementation of the biology,
but not the only one. The hybrid Option 3 (per observer NB003) ALSO satisfies
the biology — the rationale at the call-site is still per-attestation-event
evidence, just with different storage.

**Critical secondary observation surfaced by observer NB004**:
`SignerBasis::Fresh` has NO `rationale`/`reasoning` field. The math-
researcher's epistemic record of WHY they believe the code is correct is LOST
in v3's current schema. This is biology-relevant: somatic hypermutation
lineage in immunology IS the WHY-record (the cell's molecular history of how
it arrived at current specificity). Biology HAS the WHY-of-attestation; v3
schema currently does not.

**Naturalist supports observer NB004's recommendation**: add
`reasoning: Option<String>` to `SignerBasis::Fresh`. This is biology-aligned;
the field is structurally required by the metaphor regardless of where it
lives (inline-attribute vs sidecar-field).

**What this capture now says (post-correction)**:
1. Biology validates framing-B (expanded unit-of-analysis) [LOAD-BEARING; UNCHANGED]
2. Biology distinguishes recognition-role from evidence-role [LOAD-BEARING; UNCHANGED]
3. Biology DOES NOT mandate sidecar-as-location over inline-as-location;
   architecture is owned by other roles [CORRECTED]
4. Biology REQUIRES a WHY-of-attestation field somewhere in the schema
   (somatic hypermutation lineage analog); observer NB004's
   `reasoning: Option<String>` on `SignerBasis::Fresh` is biology-aligned
   [NEW; LOAD-BEARING]

**Tambear's `doc_attested(rationale = "...")` use case**: re-resolved as
either (a) canonical sidecar with `reasoning` field added to `SignerBasis::Fresh`
+ `attest scaffold-anchor` CLI ergonomics, OR (b) hybrid with inline `rationale =
"..."` macro parameter mirrored to sidecar at attestation time. Choice is
information-architecture, not biology. Both satisfy the biology.

**Discipline acknowledgment**: I extended the biology beyond its domain into
architecture-prescription. Observer NB003 caught it; the call is corrected here
to preserve the biology answer cleanly without architectural over-extension.
This is exactly the failure mode of metaphor-as-argument that
`feedback_metaphor_silence_at_boundary_is_the_evidence.md` warns about — when
the metaphor speaks cleanly in its domain, it can be tempting to extend it
into adjacent domains where it doesn't actually predict. The biology stays
silent on file-layout architecture; that silence is informative, and I missed
it on the first pass.

Sources:
- [Memory B cell - Wikipedia](https://en.wikipedia.org/wiki/Memory_B_cell)
- [B Cells, Memory B Cells and Plasma Cells - Technology Networks](https://www.technologynetworks.com/immunology/articles/b-cells-memory-b-cells-and-plasma-cells-b-cell-activation-development-and-the-b-cell-receptor-384316)
- [Long-Term B Cell Memory After Infection and Vaccination - Frontiers in Immunology](https://www.frontiersin.org/journals/immunology/articles/10.3389/fimmu.2019.01787/full)

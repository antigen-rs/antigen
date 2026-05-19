# Capture — Aristotle Team-Pass on v3 Frontier (continuation)

> **Date**: 2026-05-19
> **Author**: team-aristotle (Opus 4.7 1M, v3-frontier continuation pass)
> **Relation to prior captures**: continuation of
> [aristotle-team-pass-2026-05-18](discipline-witnesses-aristotle-team-pass-2026-05-18.md)
> (F1-F8 + F9 frontier flag, all absorbed into v3). v3 has moved; this
> pass attacks the new frontier (T3, T4, T5, T7, T8) and the post-
> folding R-Ar2 unification claim.
> **Status**: append-only capture

> **Format**: numbered F-findings continuing the signature shape
> (F10+). Each F is a kernel that survived Phase 1-8 with reasoning
> shown. Frontier extensions surface as F-findings or as frontier
> flags (T-codes) for the next pass.

---

## Posture

v3 absorbed F1-F8 + adversarial T1-R...T9-R. The substrate is denser
than the prior pass attacked. The discipline-witnesses primitive now
covers immunity + tolerance via isomorphic schema (scout S1), names
ratchet-asymmetry + bounded audit-of-audit + EvidenceKind axis +
scope field + delta-chain anti-laundering safeguards. The frontier
attack-surface is the OPEN T1-T8 set + the FA-1...FA-6 unresolved
adversarial seeds.

This pass attacks the frontier with the discipline named in the
team-launch brief: **structural complexity is the point**. Where v3
has folded an absorption that looks clean, I will press on whether
the absorption hides a structural seam.

Substrate-grep first: I read ADR-018 (descended_from) before
attacking F10/FA-5 — ADR-018 already specifies a 7-state matrix
where inheritance propagates ancestor's witness/match_kind AND
descendant re-attestation is required (state 6 = re-attested; state 7
= unaddressed warn). The FA-5 framing ("can consumer weaken
predicate?") was attacking a strawman. The REAL frontier is sharper.

---

## F10 — `descended_from` predicate-weakening is BLOCKED by ADR-018's state-6/7 architecture; the frontier is "what `addresses` means when predicates differ"

### What FA-5 claimed

Adversarial FA-5: "Crate A declares SignedZeroDiscipline with a
specific predicate. Crate B uses `#[descended_from = A::SignedZeroDiscipline]`.
Does Crate B inherit A's predicate, or does B write its own? R-A7 says
per-consumer ratification, which implies B writes its own. But
`descended_from` carries connotation of 'derived from, includes the
obligations of.' If B's predicate is WEAKER than A's, `descended_from`
is semantically misleading. Should `descended_from` require at least
as-strong a predicate as the parent?"

### Phase 1 — Visible claim, made precise via ADR-018 substrate

Substrate-check changes the question. ADR-018 ratifies:
- `#[descended_from(parent)]` produces lineage edges
- Synthesis pass creates inherited Presentation records on descendant
  with `inherited_from: Some([provenance])`
- Inherited Presentation carries ancestor's `match_kind` and identity
- **Re-attestation is required on descendant** (state 6) — descendant
  writes its own `#[immune(X, witness = ...)]` OR `#[antigen_tolerance(X)]`
- **Unaddressed inheritance** (state 7) emits warn diagnostic by
  default, error under `--strict`
- Per ADR-005 §item 2: scan walks chains and re-checks; A4-A5
  behavioral re-validation is structurally guaranteed

This means: the descendant's predicate ALREADY differs from the
ancestor's in the general case (per-consumer ratification per R-A7).
The descendant writes its OWN `#[immune]` with its OWN `requires =`
predicate. The "weakening" attack is not about inheritance forcing
a predicate; it's about whether the descendant's RE-ATTESTATION,
when WEAKER than the ancestor's predicate, satisfies state 6
("inherited + re-attested") tier-honestly.

### Phase 2 — Assumptions

- **A**: "State 6 (re-attested)" treats descendant's `#[immune]` as
  fully satisfying the inheritance regardless of predicate strength.
- **B**: "Predicate strength" is comparable across crates (i.e., A's
  predicate ⊆ B's predicate is decidable).
- **C**: The audit reports state 6 with a single tier; the tier reflects
  descendant's predicate evaluation alone.
- **D**: Consumers reading the audit understand that state 6 means
  "descendant re-attested" without implying "descendant's attestation
  is as strong as ancestor's."
- **E**: `descended_from` carries no semantic obligation beyond
  lineage-tracking (i.e., it's a provenance marker, not a contract).

### Phase 3 — Stripping

- **Strip A**: State 6 treats descendant `#[immune]` as fully
  satisfying.
  - Per ADR-018 §"State 6": "`inherited_from = Some(_)` AND descendant
    has explicit `#[immune]` or `#[antigen_tolerance]` addressing the
    same antigen on the same item." The mere PRESENCE of descendant
    `#[immune]` flips state 7 → state 6.
  - The state-6 audit does NOT compare descendant's predicate against
    ancestor's predicate. It checks: (i) same antigen, (ii) same item,
    (iii) descendant has its own attestation.
  - **Strip-A reveals**: state 6 is a TOPOLOGICAL check (same antigen
    + same item + descendant attests), not a SEMANTIC check
    (descendant attests as strongly as ancestor). The audit's tier-
    honesty for state 6 lives at the DESCENDANT's predicate evaluation
    layer, not at a predicate-comparison layer.

- **Strip B**: Predicate strength is comparable.
  - Closed combinator grammar: `all_of([...])`, `any_of([...])`,
    `not(...)`. Leaves: `ratified_doc`, `signers`, `signed_trailer`,
    `oracles_complete`, `fresh_within_days`.
  - Is `all_of([signers(["alice"]), oracles_complete(["X"])])`
    "stronger than" `signers(["alice"])`? In a containment sense, yes
    (every state satisfying the former satisfies the latter; the
    converse fails). This is decidable for closed-grammar predicates
    via syntactic structure analysis.
  - But: leaves can vary in CONTENT, not just composition. `signers(["alice"])`
    vs `signers(["bob"])` — neither is stronger; they're incomparable.
    Same with `ratified_doc(min_version = "1.0")` vs
    `ratified_doc(min_version = "2.0")` — different doc-content
    requirements that may not have a containment relationship at all.
  - **Strip-B reveals**: predicate strength is PARTIAL order, not
    total. Some pairs are comparable (containment); some are not
    (different signer sets, different doc-version thresholds with no
    natural ordering). A "predicate-at-least-as-strong-as-parent"
    check would need to enumerate ALL satisfying environments — that's
    Turing-complete in the general case for the leaf content space.

- **Strip C**: Audit reports state 6 with a single tier reflecting
  descendant's evaluation alone.
  - Current v3 mapping: descendant's `#[immune]` is its own substrate-
    witness; predicate evaluates against descendant's `.attest/` sidecar;
    audit emits descendant's WitnessTier × AuditHint × EvidenceKind.
  - The audit does NOT propagate ancestor's tier into descendant's
    report. ADR-018 §state-6 explicitly: descendant's re-attestation
    is the substrate; descendant's tier reflects descendant's evidence.
  - **Strip-C reveals**: per-consumer ratification (R-A7) is the
    discipline. The audit's tier-honesty for state 6 is the
    descendant's tier-honesty, period. Ancestor's tier is metadata
    in `inherited_from`, not arithmetic in the tier computation.

- **Strip D**: Consumers understand state 6 doesn't imply
  predicate-equivalence.
  - This is a documentation discipline, not a substrate property.
    Risk: a consumer reads "state 6, Execution-tier" and assumes "as
    strong as ancestor's discipline." That's a CONSUMER MISREAD, not
    an AUDIT OVERCLAIM — but the misread is foreseeable enough that
    tier-honesty discipline should make the gap visible.
  - **Strip-D reveals**: there's a TIER-HONESTY GAP at the
    inheritance-reporting layer. The audit reports descendant's
    tier-honestly; the consumer needs to understand that ancestor's
    expectations might have been higher. The substrate to surface
    this gap exists (`inherited_from` carries provenance); the audit
    can emit a hint when descendant's predicate is **detectably
    weaker** than ancestor's via syntactic containment check.

- **Strip E**: `descended_from` carries no semantic obligation.
  - Per ADR-018, `#[descended_from]` MEANS: ancestor's presentations
    propagate to descendant; descendant inherits the EXPECTATION to
    re-attest. The semantic load IS the expectation.
  - But the inheritance is at the level of "the antigen applies to
    descendant" (lineage = "the substrate is the same kind of code"),
    not at the level of "the descendant's discipline standard equals
    or exceeds ancestor's."
  - Naturalist rhyme adjacent: biology has clonal lineage WITHOUT
    requiring each clone to express the same antibody strength.
    Affinity matures; some clones lose function. `descended_from` is
    clonal-lineage; predicate-strength comparison is affinity-quality —
    different layer.
  - **Strip-E reveals**: `descended_from` carries lineage-obligation
    (descendant re-attests for the same antigen); it does NOT carry
    predicate-equivalence-obligation. The latter is a separate
    discipline that, if needed, requires a different mechanism.

### Phase 4 — Irreducible kernel

**`descended_from` semantics, per ADR-018, are LINEAGE-TRACKING +
RE-ATTESTATION-EXPECTATION. They are NOT predicate-strength-binding.
Per-consumer ratification (R-A7) is the discipline; descendant writes
its own predicate; audit reports descendant's tier-honestly.**

**Two layers separate cleanly**:

- **Layer 1 (lineage)**: ADR-018's state-6/7 machinery — descendant
  has `inherited_from`; descendant must re-attest; unaddressed warns.
- **Layer 2 (predicate-strength comparison)**: NOT in v3. Would need
  a new mechanism (predicate-containment-check + audit hint when
  descendant's predicate is weaker than ancestor's).

**The tier-honesty gap surfaced**: a consumer reading "descendant
state 6, Execution-tier" might infer "as strong as ancestor's
attestation." The audit can surface the gap by emitting a hint when
descendant's predicate is **detectably weaker** (syntactic containment
fails) than ancestor's. This is FA-5's real frontier.

### Phase 5 — Structurally forced conclusions

- **v3 does NOT need to add predicate-strength-binding to
  `descended_from`**. Per-consumer ratification holds; ADR-018's
  state-6/7 machinery is sufficient for lineage-tracking.
- **v0.1 SHOULD emit a tier-honesty hint when descendant's predicate
  is detectably weaker than ancestor's** via syntactic containment
  check. This is a NEW audit hint:
  `inherited-predicate-weaker-than-ancestor`. Hint MAY be inferred
  for cases where containment is decidable; explicitly not-inferred
  for incomparable cases (different signer sets, different doc-version
  thresholds without natural ordering).
- **The detection is HEURISTIC, not normative**: false negatives
  exist (weakening via content change that syntactic check can't
  detect, e.g., signer alice replaced with bob — incomparable but
  potentially weaker per discipline). Audit hint says "DETECTABLY
  weaker," not "weaker." Tier-honest: surface what's mechanically
  visible; don't overclaim semantic comparison.
- **`#[descended_from]` documentation in glossary** should explicitly
  state: "lineage-tracking + re-attestation expectation; does NOT
  bind descendant to ancestor's predicate strength. Per-consumer
  ratification per R-A7."

### Phase 6 — Adjacency

- Adjacent to ADR-017 (`addresses()`): the addresses-tuple is
  `(antigen_type, item, canonical_path)`. State-6 uses this for
  same-antigen-same-item check. Predicate-strength is OUTSIDE the
  addresses-tuple — different axis.
- Adjacent to ADR-005 sub-clause F: per-consumer ratification IS the
  trust-boundary discipline for inheritance. Descendant's re-attestation
  IS the validation check at the lineage boundary. The
  inherited-predicate-weaker hint is a SECOND-ORDER visibility
  enhancement, not a new trust boundary.
- Adjacent to naturalist clonal-lineage rhyme: affinity-maturation
  produces clones with varying binding strength; lineage tracking
  doesn't bind strength. Biology supports the predicate-strength
  separation.
- Adjacent to F1 (discipline-vs-machinery unification): adding a
  predicate-strength comparison mechanism to descended_from would
  introduce NEW machinery (containment-check engine), not just NEW
  discipline. Per F1 logic, this should be carefully scoped — the
  unification discipline (per-consumer ratification + lineage
  tracking) is the load-bearing part; the containment-check is
  machinery deferrable.

### Phase 7 — Extension predictions

- Future predicate-containment-check engine ships as v0.2+ amendment
  IF adoption substrate shows consumers misreading state-6 tiers as
  "as-strong-as-ancestor." Hold for adoption evidence; per ADR-006
  recognition-not-design.
- The `inherited-predicate-weaker-than-ancestor` hint extends to
  cross-crate descended_from: if A is in crate `foo` and B is in
  crate `bar`, the comparison can still run at audit time provided
  the audit can read A's declaration. Per F1, this is audit-internal
  machinery; tier-honesty discipline carries.
- A symmetric hint: `inherited-predicate-stronger-than-ancestor`
  (descendant attests more strongly than ancestor required). NOT a
  tier-honesty concern but a discipline-evolution signal — descendant
  has elevated the standard; potentially the ancestor's declaration
  should be revisited. Defer this; not in scope for v0.1.

### Phase 8 — Verdict

**F10 (closes T8 / absorbs FA-5)**: `descended_from` is
lineage-tracking + re-attestation-expectation, NOT predicate-strength-
binding. Per-consumer ratification (R-A7) is the load-bearing
discipline. ADR-018's state-6/7 machinery is sufficient for
inheritance-reporting tier-honesty AT THE LINEAGE LEVEL.

**The frontier is the consumer-misread surface**: state 6 + descendant
Execution-tier might be read as "as strong as ancestor." Mitigation: a
NEW audit hint `inherited-predicate-weaker-than-ancestor` emitted when
syntactic containment check fires. Heuristic (false-negatives
allowed); explicitly not-normative for incomparable cases.

**v3 implication**: add the audit hint to v3's tier-honesty mapping
table (extends the v0.1 hint vocabulary; no schema change). Glossary
entry for `#[descended_from]` should clarify lineage-vs-predicate
separation. NO new ADR; this is a v3 refinement absorbable into
ADR-019's audit-hint vocabulary.

**Adversarial FA-5 frontier is now addressed at the right layer**: not
"forbid weakening" (which would break per-consumer ratification and
introduce undecidable containment-checks across leaf content), but
"surface detectable weakening as a tier-honesty hint" (which preserves
per-consumer ratification while making the discipline gap visible).

---

## F11 — Compound-evidence is COMPOSITION-OF-CLAIMS, not arithmetic-on-tiers; the false additive-confidence surface is REAL and v3 needs an explicit multi-witness shape + EvidenceKind lattice

### What T4 / FA-6 claimed

T4 (carried frontier from prior aristotle pass + adversarial team pass):
"compound evidence (behavioral test + substrate signatures on same
antigen-site) — does this create false 'additive confidence' surface?
How to report tier-honestly?"

FA-6 (adversarial team pass frontier): "Is there an implicit ordering
between EvidenceKind variants? If a consumer wants 'at least Behavioral
evidence,' does TypeSystemProof satisfy the gate? `WitnessTier` is
explicitly ordered; `EvidenceKind` was framed as 'parallel axis' not
ordered. Without an ordering, a CI gate cannot write 'require at least
Behavioral or better' meaningfully."

### Phase 1 — Visible claim, made precise

Two distinct scenarios under "compound evidence":

- **Scenario A** (intra-antigen, multi-witness): one antigen X
  declared on item I has TWO witnesses — e.g.,
  `#[immune(X, witness = test_fn)] #[immune(X, witness = substrate(...))]`
  (or whatever the v0.1 syntax for multi-witness looks like).
- **Scenario B** (intra-antigen, multi-evidence-kind on single
  witness): one substrate-witness predicate combines leaves that span
  evidence-kinds, e.g., `all_of([signers(...), oracles_complete(...)])`
  where `signers` is SubstrateState and `oracles_complete` reads a
  test-result fixture marker (also SubstrateState, but the underlying
  evidence is Behavioral via a separate harness run).

Scenario A is the FA-6 frontier — multi-witness reporting. Scenario B
surfaces a SUBTLER issue: SubstrateState evidence may itself be a
PROXY for Behavioral evidence (oracle file marked complete = a
behavioral harness ran somewhere, by some party, and the result was
recorded).

### Phase 2 — Assumptions

- **A**: Multi-witness reporting on a single antigen-site is currently
  representable in v3 (i.e., the audit output shape supports a
  collection of witness reports per site).
- **B**: When two witnesses both report `Execution` tier with different
  EvidenceKinds, the consumer's correct interpretation is
  "stronger together."
- **C**: EvidenceKind is a parallel axis (no implicit ordering between
  TypeSystemProof, Behavioral, SubstrateState).
- **D**: SubstrateState evidence is structurally distinct from
  Behavioral evidence, even when the substrate is a record of a
  behavioral run.
- **E**: CI gates that want "at least X-kind evidence" can express
  this without an ordering — by filtering on EvidenceKind set
  membership.

### Phase 3 — Stripping

- **Strip A**: Multi-witness representability.
  - v3 schema currently shows `#[immune(X, requires = ...)]` — singular
    `requires` per `#[immune]`. Multi-witness on a single antigen
    requires either (i) multiple `#[immune(X, ...)]` invocations on
    the same item (currently undefined behavior) or (ii) a multi-
    witness shape in a single invocation (currently absent).
  - ADR-018 state-6 anti-case clarifies: `#[presents(A)]` doesn't
    address an inherited Presentation for B (different antigen). This
    is the SAME-ANTIGEN/SAME-ITEM dedup logic. It doesn't speak to
    MULTI-WITNESS on the same `(antigen, item)`.
  - **Strip-A reveals**: v3 does NOT define multi-witness semantics
    for substrate-witnesses on a single item. This is a real gap —
    Scenario A isn't fully expressible today.

- **Strip B**: "Stronger together" is the correct interpretation.
  - Two witnesses W1 (behavioral test, Execution, Behavioral kind) and
    W2 (substrate-witness, Execution, SubstrateState kind) on the same
    antigen-site:
    - W1 evidence: the test ran and passed
    - W2 evidence: the predicate-against-sidecar passed; signers
      current
  - These are NOT independent observations of the same thing — they
    answer DIFFERENT questions:
    - W1 answers: does the code BEHAVE correctly per the test
      harness?
    - W2 answers: did humans review the discipline and ratify?
  - "Stronger together" is ONLY true if the questions are
    complementary and orthogonal. If both questions answer the SAME
    underlying property (e.g., two tests of the same predicate), then
    "stronger together" is more like "redundant" — comfort, not
    confidence-gain.
  - **Strip-B reveals**: "stronger together" is the consumer's
    INFERENCE, not the audit's REPORT. The audit's tier-honest report
    is each witness individually; consumers compose interpretations.
    The audit MUST NOT report a composite tier higher than any
    individual witness — that would be the additive-confidence
    overclaim.

- **Strip C**: EvidenceKind has no ordering.
  - True at the kind level: TypeSystemProof is NOT "better than"
    Behavioral in a linear sense; they are categorically different
    evidence types with different ceilings.
  - BUT: within a tier (say Execution), evidence kinds carry
    different STRUCTURAL guarantees. TypeSystemProof + Execution =
    compile-time + behavioral validation. Behavioral + Execution =
    harness ran. SubstrateState + Execution = predicate against
    sidecar passed. These have different failure modes —
    TypeSystemProof catches things even untested codepaths cannot
    exhibit; Behavioral catches things only as good as the test
    suite; SubstrateState catches things only as honest as the
    signers.
  - **Strip-C reveals**: EvidenceKind has no LINEAR ordering, but
    different kinds have different STRUCTURAL POWERS. A CI gate
    "require at least Behavioral" needs to be expressed as set
    membership (`evidence_kind in {Behavioral, TypeSystemProof}`),
    where TypeSystemProof is acceptable because it strictly subsumes
    Behavioral's correctness guarantees AT THE CONSTRUCT LEVEL — not
    because it is ordinally "higher."
  - This is the partial-order / lattice answer: TypeSystemProof
    dominates Behavioral within shared correctness predicates;
    SubstrateState is incomparable to both (it is social-attestation
    evidence, not correctness evidence). Lattice: TypeSystemProof
    ≥ Behavioral; SubstrateState incomparable to either.

- **Strip D**: SubstrateState is structurally distinct from
  Behavioral, even when substrate records a behavioral run.
  - Example: `oracles_complete(["oracle.md"])` reads a file with
    `status: complete`. The substrate is SOCIAL (someone marked it);
    the underlying evidence is BEHAVIORAL (a harness ran). But the
    AUDIT verifies the SOCIAL claim — it reads the marker, not the
    harness output.
  - Audit's tier-honesty: report SubstrateState (the audit verified
    the social marker), not Behavioral (the audit did not run or
    verify the harness output).
  - **Strip-D reveals**: SubstrateState is the audit's TIER-HONEST
    kind for any predicate-over-substrate, regardless of what the
    substrate RECORDS. The downstream behavioral evidence chain is
    SOCIAL-MEDIATED; the audit reports what IT verified, not what
    was claimed at an upstream layer.

- **Strip E**: Set-membership filtering is sufficient for CI gates.
  - True if the lattice is documented. CI gate
    `evidence_kind in {Behavioral, TypeSystemProof}` says "require
    correctness-evidence, social-evidence-alone insufficient." This
    is decidable, expressible, tier-honest.
  - But: consumers wanting "as strong as possible" need to understand
    the lattice (TypeSystemProof ≥ Behavioral; SubstrateState
    incomparable). Without this, set-membership phrasing is awkward
    and consumers may default to ordinal-thinking that does not match
    the structure.
  - **Strip-E reveals**: documenting the EvidenceKind lattice
    (partial order) in ADR-019 is necessary. The lattice is
    consumer-facing semantics, not just internal taxonomy.

### Phase 4 — Irreducible kernel

**Compound-evidence on a single antigen-site is COMPOSITION-OF-CLAIMS
across orthogonal evidence-kinds, NOT arithmetic-on-tiers. The audit
MUST report each witness's tier independently and MUST NOT collapse
into a composite tier. Consumers compose interpretations based on
the kind-lattice (TypeSystemProof ≥ Behavioral; SubstrateState
incomparable to both).**

**Two structural additions surface**:

- **Addition 1 (Scenario A — multi-witness representability)**: v3
  schema needs explicit multi-witness shape for `#[immune(X, ...)]`
  on a single item. Either (i) multi-`#[immune]` on the same item
  with set-union semantics, or (ii) `witnesses = [...]` as a list
  parameter. Choose at v3 absorption.
- **Addition 2 (EvidenceKind lattice documentation)**: ADR-019 must
  document the partial order on EvidenceKind: TypeSystemProof
  ≥ Behavioral (within shared correctness predicates);
  SubstrateState incomparable to both. CI-gate guidance follows
  (set-membership phrasing).

### Phase 5 — Structurally forced conclusions

- **Audit output shape**: per `(antigen, item)` pair, audit reports a
  COLLECTION of witness reports. Each report =
  `(witness_id, tier, hint, evidence_kind, signature_strength)`.
  Consumer aggregates per their gating policy.
- **Multi-witness syntax**: v0.1 ships `#[immune(X, witnesses = [...])]`
  list-parameter form. Single-witness `requires = ...` remains
  shorthand for `witnesses = [substrate(requires = ...)]`. Choosing
  list-parameter over multi-`#[immune]` because: (i) ADR-018's
  `addresses()` is per-`(antigen, item)` — multi-`#[immune]` would
  need a new dedup key for "which `#[immune]` invocation," adding
  complexity; (ii) list-parameter is co-located with the antigen
  declaration which is consumer-clearer; (iii) per F1, machinery
  stays per-substrate-kind — the list dispatches to per-witness
  recognition pipelines without sharing parser code.
- **EvidenceKind lattice in ADR-019**:
  ```
  TypeSystemProof
        |
    Behavioral
        |
       None

  SubstrateState   (incomparable to all above)
  ```
  Lattice has two connected components: the correctness-evidence
  chain (TypeSystemProof ≥ Behavioral ≥ None) and the
  social-attestation chain (SubstrateState ≥ None). They share
  bottom (`None`) but not top.
- **CI-gate guidance**:
  - "require at least Behavioral" → `evidence_kind ∈ {Behavioral,
    TypeSystemProof}` (set-membership)
  - "require correctness-evidence (exclude social-only)" →
    `evidence_kind ∈ {Behavioral, TypeSystemProof}`
  - "require any tier-honest evidence" → any non-`None` kind
  - "require social-attestation (discipline ratification)" →
    `evidence_kind = SubstrateState`
  - "require BOTH correctness AND social" → multi-witness with at
    least one of each kind reported as ≥ Execution
- **The `compound_evidence` reporting field**: audit output per
  `(antigen, item)` SHOULD include `compound_evidence: bool` (true
  when ≥ 2 witness reports exist). This is the consumer-visibility
  signal that DISABLES "stronger together" naive aggregation by
  surfacing the multi-witness condition explicitly — the consumer
  sees `compound_evidence: true` and is prompted to consider per-
  witness reports rather than picking the highest tier.

### Phase 6 — Adjacency

- Adjacent to F8 (EvidenceKind axis): F8 established kind as
  first-class; F11 establishes the lattice + the lattice's relation
  to CI-gate semantics + the multi-witness shape.
- Adjacent to F1 (discipline-vs-machinery unification): multi-witness
  ships in the same discipline (each witness reports tier-honestly,
  no aggregate-tier); machinery stays per-substrate-kind (each
  witness invokes its own recognition pipeline).
- Adjacent to ADR-005 Amendment 3 (audit reports own tier honestly):
  the multi-witness reporting MUST follow Am 3 — each witness reports
  its own tier; no composite-tier projection.
- Adjacent to scout S2 (severity-class as candidate first-class
  dimension): severity is ORTHOGONAL to evidence-kind and tier.
  Severity-class affects CI-gate failure-policy; evidence-kind
  affects CI-gate input-filtering. Different axes; both consumer-
  facing.

### Phase 7 — Extension predictions

- v0.2+ may add evidence-kind-aware aggregation hints (e.g.,
  `composite_correctness_evidence: true` when Behavioral +
  TypeSystemProof on same site — actually-stronger-together for
  correctness questions). Hold for adoption substrate; per ADR-006.
- v0.2+ may add weight-based CI-gate language (require N points
  where TypeSystemProof = 3, Behavioral = 2, SubstrateState = 1)
  for scoring-style gates. Explicitly NOT in v0.1 — risks reifying
  ordering where none exists. The lattice answer is structurally
  more honest.
- Multi-witness on `#[antigen_tolerance(...)]` follows the same
  shape: tolerance can have multiple evidence-sources (e.g.,
  signer-approval AND timed-exception document). v0.1 ships parallel
  `witnesses = [...]` on tolerance per scout S1 isomorphism.

### Phase 8 — Verdict

**F11 (closes T4 + closes FA-6 + extends F8)**: compound-evidence
reporting requires:

1. **Multi-witness representability** in v3 schema:
   `#[immune(X, witnesses = [...])]` list-parameter shape.
2. **EvidenceKind lattice documentation** in ADR-019:
   TypeSystemProof ≥ Behavioral (within correctness predicates);
   SubstrateState incomparable.
3. **Audit-output shape**: collection of witness reports per
   `(antigen, item)`; NO composite-tier projection.
4. **`compound_evidence: true` field** on audit output when ≥ 2
   witness reports exist on a single site; disables consumer
   naive-aggregation.
5. **CI-gate guidance via set-membership** phrasing in v0.1 docs;
   weight-based scoring deferred to v0.2+ with adoption evidence.

**FA-6 frontier (ordering) is resolved at the LATTICE level, not
the LINEAR level**. The lattice answer is more honest than forcing
a total order: TypeSystemProof and Behavioral live on a correctness
axis where TypeSystemProof dominates; SubstrateState lives on a
social-attestation axis incomparable to correctness. CI-gates
express intent via set-membership; tier-honesty discipline is
preserved.

**T4 frontier (compound-evidence) is resolved at the
COMPOSITION-OF-CLAIMS level**, not at an arithmetic-on-tiers level.
The audit's discipline: report each witness; surface multi-witness
via `compound_evidence` field; let consumers compose per documented
lattice.

**v3 implication**: this is a NON-TRIVIAL v3 absorption — adds:
- Multi-witness syntax (`witnesses = [...]` list-parameter)
- EvidenceKind lattice documentation in v3 + ADR-019
- Audit-output collection shape (was per-witness; needs explicit
  documentation that multi-witness is supported)
- `compound_evidence: bool` field on audit output
- CI-gate guidance section in ADR-019

Absorbable; not a re-split. ONE-ADR position holds.

---

## F12 — Fingerprint-scheme evolution is a TYPED-CLAIM-PROVENANCE problem, not a sidecar-migration problem; sidecars must carry `fingerprint_scheme_version` and audit must distinguish stale-substrate from stale-encoding

### What T7 / FA-2 claimed

T7 (carried from adversarial frontier FA-2): "Fingerprint-scheme
evolution causing false-stale across antigen version upgrades — needs
cross-version migration story."

FA-2 full: "`antigen_fingerprint` will evolve across antigen versions.
What happens when a sidecar contains `signed_against_fingerprint`
computed by antigen v0.1's fingerprinting scheme, and the audit runs
antigen v0.2 with a refined fingerprinting scheme? The comparison will
always show 'stale' even for code that hasn't changed. This is the
`cargo lock format changed` problem applied to attestation sidecars."

### Phase 1 — Visible claim, made precise

Substrate-check:
- `antigen_fingerprint::Fingerprint` is a structural pattern (Item +
  NameMatches + Variants + HasMethod + AttrPresent + DocContains +
  BodyContainsMacro + AllOf/AnyOf/Not), serialized via serde. NOT a
  content-hash; a STRUCTURAL pattern AST.
- Sidecar `current_fingerprint: String` and `signed_against_fingerprint: String`
  carry the serialized Fingerprint AST as a string.
- v3 `Ratification` has `schema_version: SchemaVersion` (covers
  sidecar-format changes); does NOT have fingerprint-scheme version.
- The scheme that REDUCES code-AST to a Fingerprint pattern is the
  variant-set + match-discipline implemented in `antigen-fingerprint`.
  When that crate evolves (new variants like `BodyContainsExpr`,
  refined match semantics for `HasMethod`, etc.), the serialized
  string for "the same code" can change.

The FA-2 attack: an antigen v0.1 signer stamps fingerprint
`item=Fn AND name=matches("sinh") AND has_method(...)`. Audit at
antigen v0.2 re-computes the fingerprint with a refined HasMethod
canonicalization — produces a different serialized string. Audit
reports "stale" → demotes to Reachability + `discipline-substrate-stale`
hint. False positive: the code didn't change; only the encoding scheme
did.

### Phase 2 — Assumptions

- **A**: Fingerprint-scheme evolution is rare enough that it can be
  handled ad-hoc per release.
- **B**: The audit's comparison is at the SERIALIZED-STRING level
  (string equality), not at the AST level.
- **C**: `signed_against_fingerprint` is meaningful only in
  conjunction with the scheme that generated it.
- **D**: Schema-versioning at the `Ratification` level (existing
  `schema_version`) covers fingerprint-scheme changes.
- **E**: Re-attestation cost on every fingerprint-scheme change is
  acceptable.

### Phase 3 — Stripping

- **Strip A**: Fingerprint-scheme evolution is rare.
  - Substrate-check: `antigen_fingerprint` is at v0.0.1 in this
    workspace. It has 9 leaf-Constraint variants and 3 combinator
    variants. The first 2 years of antigen development will likely
    add 3-5 leaf variants minimum (e.g., `BodyContainsExpr`,
    `WhereClauseShape`, `ConstParam`, `GatBounds`). Each addition
    that touches recognition machinery for EXISTING patterns
    (canonicalization tweaks, method-signature normalization) can
    change the serialized form for the same code.
  - Adversarial: a HasMethod canonicalization fix to handle const
    generics correctly would produce different serialized fingerprints
    for any code using methods with const generic parameters, even
    if the CODE didn't change.
  - **Strip-A reveals**: scheme evolution is frequent enough to design
    for, not rare enough to handle ad-hoc.

- **Strip B**: Comparison is at the serialized-string level.
  - Today (v0.1 substrate): yes, `String == String`. This is
    brittle to ANY representational change (whitespace, key ordering
    in serde, optional-field defaulting).
  - Counter-design: comparison could be at the parsed-AST level —
    parse both `signed_against` and `current` as Fingerprints, compare
    structurally. Cost: AST comparison logic; benefit: ignores
    serialization-only changes that don't change pattern semantics.
  - But: AST-level comparison only solves SERIALIZATION drift, NOT
    SEMANTIC drift (a new variant changing how patterns are computed
    from code still produces different ASTs for the same code).
  - **Strip-B reveals**: AST-comparison is a partial mitigation
    (covers serialization drift); does NOT cover scheme-semantic
    drift. The deeper problem requires versioning.

- **Strip C**: `signed_against_fingerprint` is meaningful only with
  the scheme that generated it.
  - This is the central truth. A fingerprint string is a TYPED CLAIM
    produced by a specific reduction. Without the scheme version, the
    string is ambiguous — different schemes can produce the same
    string from different code, or different strings from the same
    code.
  - **Strip-C reveals**: sidecars MUST carry the scheme-version that
    PRODUCED the signature. Otherwise the typed-claim has no
    interpretable type.

- **Strip D**: `schema_version` covers fingerprint-scheme changes.
  - `schema_version` covers the `Ratification` STRUCT layout (what
    fields exist, their types). It does NOT cover the SCHEME the
    `current_fingerprint` STRING was produced by. These are
    orthogonal: a schema_version=v2 sidecar could carry a
    fingerprint_scheme_version=v1 fingerprint string (compatible
    struct layout, older scheme).
  - **Strip-D reveals**: fingerprint_scheme_version is a SEPARATE
    field needed on `Signer.signed_against_fingerprint` (or its
    parent `ItemRatification.current_fingerprint`). Each fingerprint
    string in the sidecar carries the scheme that produced it.

- **Strip E**: Re-attestation on every scheme change is acceptable.
  - For SEMANTIC scheme changes (new variants, changed recognition):
    YES, re-attestation is correct — the signer attested to a pattern
    that no longer represents the code's discipline shape.
  - For SERIALIZATION-ONLY scheme changes (key reordering, whitespace
    canonicalization): re-attestation is friction without value.
  - **Strip-E reveals**: scheme changes are NOT homogeneous. Some
    invalidate prior signatures (semantic); some don't
    (serialization). The migration story must distinguish.

### Phase 4 — Irreducible kernel

**`signed_against_fingerprint` is a TYPED CLAIM whose type is the
fingerprint-scheme version. Sidecars must carry
`fingerprint_scheme_version` per fingerprint-string. The audit must
distinguish three states**:

1. **scheme_version matches current**: standard comparison —
   string-equal = current; string-differ = stale.
2. **scheme_version is older AND scheme provides forward-compat
   parse**: parse `signed_against` via legacy-scheme parser, compute
   what its current-scheme equivalent WOULD have been, compare to
   current. If equal → still-current (signer's attestation survives
   serialization-only scheme change). If different → stale (code or
   semantic scheme change invalidated signature).
3. **scheme_version is older AND scheme has no forward-compat parse**:
   audit reports `signed_against-scheme-unsupported` hint; demote to
   Reachability with explicit `fingerprint-scheme-evolution-pending-re-attest`
   distinguishable from `discipline-substrate-stale`. Consumer sees a
   different signal than "code changed."

**The mitigation lattice**:

| signer-scheme | current-scheme | tool can forward-parse | tool comparison | report |
|---|---|---|---|---|
| v1 | v1 | trivially yes | string equal | current |
| v1 | v1 | trivially yes | string differ | stale (code changed) |
| v1 | v2 | yes, semantic-preserving | forward-equal | current; hint `fingerprint-scheme-migrated` |
| v1 | v2 | yes, semantic-different | forward-differ | stale (could be code OR scheme; hint `fingerprint-scheme-may-have-changed-semantics` warns) |
| v1 | v2 | no parser provided | n/a | demote + `fingerprint-scheme-unsupported` hint, NOT `substrate-stale` |

### Phase 5 — Structurally forced conclusions

- **Schema addition**: `signed_against_fingerprint: String` becomes
  `signed_against: SignedFingerprint { scheme_version: SchemeVersion,
  fingerprint: String }`. Same for `current_fingerprint` →
  `current: SignedFingerprint`. (Names sketch; team can refine.)
- **`antigen_fingerprint` crate ships a `SCHEME_VERSION` constant**
  bumped explicitly when scheme semantics change. Serialization-only
  changes do NOT bump (instead `antigen_fingerprint` ships a
  canonicalize-on-parse discipline so serialization differences
  collapse on parse-equality).
- **Forward-compat parser discipline**: when `antigen_fingerprint`
  bumps SCHEME_VERSION, it ships a `parse_legacy(v_old, str) ->
  Fingerprint` function that produces the v_new equivalent (best
  effort; may return `LegacySchemeUnsupported` for semantic changes
  too big to bridge). The audit invokes this when sidecar's
  scheme_version differs from current.
- **Audit-hint vocabulary additions**:
  - `fingerprint-scheme-migrated` — current-scheme parse of legacy
    produces equal fingerprint → signature carries forward
  - `fingerprint-scheme-may-have-changed-semantics` — current-scheme
    parse of legacy produces different fingerprint → could be code OR
    scheme; needs human re-attestation
  - `fingerprint-scheme-unsupported` — no forward-compat parser →
    explicit re-attestation needed
- **Hint distinguishability is load-bearing**: a developer seeing
  `discipline-substrate-stale` reacts to a CODE change; seeing
  `fingerprint-scheme-unsupported` reacts to a TOOL change. Same
  remediation (re-attest); different mental model. Tier-honesty
  requires the audit to surface what actually changed.
- **`attest sign` records scheme-version at sign time** automatically
  from `antigen_fingerprint::SCHEME_VERSION`. Signer doesn't opt in;
  the version is part of the typed claim.

### Phase 6 — Adjacency

- Adjacent to F1 (discipline-vs-machinery): fingerprint-scheme
  versioning is MACHINERY level (per-substrate-type implementation
  detail); the typed-claim-provenance discipline is the load-bearing
  unification (sidecars carry the type of their claims). F12 is
  machinery-side; F1's discipline-side unification still holds.
- Adjacent to F5 (multi-pin dimensions): fingerprint-scheme version
  is a NEW pin dimension — the signer pins to (code-fingerprint AT
  scheme-version). F5's basis field tracks "what alice attested to";
  scheme-version is part of WHAT (the typed-claim shape).
- Adjacent to ADR-007 anti-YAGNI: fingerprint-scheme evolution is
  structurally guaranteed (every meaningful tool evolves its
  recognition machinery). Ship the version field in v0.1; ship the
  parse_legacy + audit-hint vocabulary at v0.2 when the first scheme
  bump happens (not before — recognition-not-design per ADR-006).
- Adjacent to FA-1 (schema migration with carry-forward chains):
  delta-chains across scheme versions need explicit handling. A
  carry-forward from a fingerprint that's now scheme-unsupported
  should NOT validate; force fresh attestation. The
  `cumulative_root_fingerprint` in `SignerBasis::DeltaFrom` carries
  scheme-version implicitly via this design.

### Phase 7 — Extension predictions

- Cargo's lockfile-format migrations (Lockfile v1 → v2 → v3) are
  exactly this pattern. Cargo carries forward-compat parsers; emits
  warning on legacy formats; eventually deprecates very old formats.
  Antigen inherits this pattern.
- The `antigen-attestation` crate ships a
  `fingerprint_compat_registry` mapping (old_version, new_version)
  → parser_fn. Adoption substrate over time accumulates these;
  long-lived projects' sidecars remain readable.
- Eventually (v2.0+): a `fingerprint_scheme_deprecated` discipline
  where audit hard-refuses very old schemes (10+ versions back) to
  prevent indefinite carry-forward burden. Long-arc; not in scope
  for v0.1.

### Phase 8 — Verdict

**F12 (closes T7 / absorbs FA-2)**: fingerprint-scheme evolution is
a TYPED-CLAIM-PROVENANCE problem. Sidecars must carry
`fingerprint_scheme_version` per fingerprint-string. The audit
distinguishes three states (same-scheme, forward-parseable
older-scheme, unsupported older-scheme) with distinct hints. The
existing `Ratification.schema_version` covers struct layout, NOT
fingerprint-scheme — these are ORTHOGONAL version axes that both
need to exist.

**v3 implication (structural, must ship in v0.1)**:
- `antigen_fingerprint` ships `pub const SCHEME_VERSION: SchemeVersion`
- Sidecar schema: `signed_against` and `current` become typed
  `SignedFingerprint { scheme_version, fingerprint }` instead of bare
  `String`
- `attest sign` records scheme-version automatically
- v0.1 audit reports `fingerprint-scheme-unsupported` if sidecar's
  scheme differs from current (no parse_legacy in v0.1 — only one
  scheme exists)
- v0.2+ ships parse_legacy registry + `fingerprint-scheme-migrated`
  / `fingerprint-scheme-may-have-changed-semantics` hints when the
  FIRST scheme bump happens

**Why ship the version field in v0.1 even though only one scheme
exists**: per ADR-007 anti-YAGNI / structurally-guaranteed. Adding
the version field later requires migrating every existing sidecar.
Adding it now costs nothing (single-version case is trivial) and
makes the migration story possible.

**Adversarial FA-2 frontier is now structurally addressed**: the
typed-claim-provenance design prevents the false-stale category
error by making the scheme part of the claim's type. The mitigation
lattice surfaces the distinction (code-changed vs scheme-changed vs
scheme-unsupported) tier-honestly.

**Absorbable into ADR-019**: schema field addition (one new struct
+ one constant) + audit-hint vocabulary additions. ONE-ADR position
holds. Coordination needed with `antigen-fingerprint` crate's
existing release discipline.

---

## F13 — EvidenceKind exhaustivity: `TypeSystemProof | Behavioral | SubstrateState` is NOT exhaustive; at minimum `EnvironmentalProbe` is a separate kind, possibly `RuntimeInvariant`

### What I am attacking

My own prior-pass frontier flag: "is `TypeSystemProof | Behavioral
| SubstrateState` actually exhaustive? Or are there evidence kinds I
haven't enumerated?" The F8 verdict assumed three kinds; v3 absorbed
them as the closed enum. Reopening to stress-test exhaustivity.

### Phase 1 — Visible claim, made precise

The current EvidenceKind enum:
- **TypeSystemProof**: phantom-type proof; compile-time guarantee;
  reaches FormalProof
- **Behavioral**: test/proptest/clippy/kani/prusti — harness-invoked
  evidence; reaches Execution
- **SubstrateState**: substrate-witness predicate against on-disk
  substrate; reaches Execution

Candidate additional kinds to evaluate:
- **EnvironmentalProbe**: runs at deploy/runtime; queries external
  state (DNS resolution, cert chain validity, network reachability,
  k8s resource quotas)
- **RuntimeInvariant**: assertion checked at runtime (`debug_assert!`,
  `assert!`, sanitizer-detected); evidence only if exercised
- **Simulation**: model-checking / fuzz-corpus / monte-carlo result;
  audit reads result fixture
- **Empirical**: observed in production telemetry (e.g., "we've never
  seen this failure across 10M requests")
- **FormalSpec**: separately-verified specification (prusti/verus
  annotation that an external tool checked; sidecar carries the
  proof artifact)

### Phase 2 — Assumptions

- **A**: The three current kinds exhaust the structural surface
  antigen reaches in v0.1.
- **B**: Future kinds slot into the lattice cleanly (TypeSystemProof
  apex of correctness chain; new kinds either join correctness chain
  or live as incomparable axes).
- **C**: Evidence kind is determined by WHERE THE AUDIT GETS THE
  EVIDENCE FROM (compile-time / harness-run / on-disk sidecar), not
  by what the evidence claims.
- **D**: EnvironmentalProbe is a special case of Behavioral
  (something runs and reports).
- **E**: RuntimeInvariant is a special case of Behavioral (test
  harness exercises the invariant).

### Phase 3 — Stripping

- **Strip A**: Three kinds exhaust v0.1 surface.
  - Substrate-grep: v3 witness families are `test_fn`, `proptest!`,
    `clippy::lint`, `kani::proof`, `prusti::ensures`, phantom-type
    proofs, substrate-witness predicates. Plus cross-crate witnesses
    (per F1).
  - Map each to a kind:
    - phantom-type → TypeSystemProof
    - `test_fn`, `proptest!` → Behavioral (harness-run)
    - `clippy::lint` → ? — lint is a STATIC ANALYSIS over source AST;
      lint result is computed without RUNNING code. Is this Behavioral
      (the lint tool ran) or TypeSystemProof (compile-time static
      check) or its own kind?
    - `kani::proof`, `prusti::ensures` → ? — formal proof tools that
      run separately; produce verification artifacts. Is the artifact
      itself Behavioral evidence of the prover running? Or
      TypeSystemProof because the result is a formal guarantee?
    - substrate-witness → SubstrateState
  - **Strip-A reveals a real gap**: clippy/kani/prusti don't cleanly
    fit Behavioral OR TypeSystemProof. They're closer to a
    `StaticAnalysisResult` or `FormalToolArtifact` kind — runs at
    audit/build time (not test-runtime), produces typed-claim artifact
    that the audit reads. This is closer to SubstrateState
    (read-the-artifact) than Behavioral (run-the-test).

- **Strip B**: Future kinds slot cleanly into lattice.
  - EnvironmentalProbe: runs against EXTERNAL world. Result is
    point-in-time observation. Lattice slot?
    - NOT comparable to TypeSystemProof or Behavioral (correctness
      chain); EnvironmentalProbe asserts ENVIRONMENT properties, not
      CODE properties.
    - Distinct from SubstrateState: SubstrateState reads
      workspace-local files; EnvironmentalProbe queries external
      services. Different attack surface (network failure, external
      flakiness, time-of-check vs time-of-use).
  - **Strip-B reveals**: EnvironmentalProbe is a SEPARATE incomparable
    chain. The lattice has at LEAST three connected components, not
    two:
    1. Correctness chain (TypeSystemProof ≥ Behavioral ≥ None)
    2. Social-attestation chain (SubstrateState ≥ None)
    3. Environmental chain (EnvironmentalProbe ≥ None)

- **Strip C**: Evidence kind determined by WHERE audit gets it.
  - Sharper formulation: evidence kind = (structural source × verification
    moment). Sources: compile-time / harness-run / sidecar-on-disk /
    external-probe / runtime-execution. Moments: ahead-of-time /
    test-time / audit-time / deploy-time / production-time.
  - **Strip-C reveals**: the three-kind enum is collapsing a
    two-dimensional space. The fact that it works for v0.1 may be a
    convenient projection — but as witness families expand
    (EnvironmentalProbe, RuntimeInvariant, FormalToolArtifact),
    projection-onto-three-kinds will hide distinctions consumers care
    about.

- **Strip D**: EnvironmentalProbe is special-case Behavioral.
  - Behavioral evidence is reproducible (harness runs deterministically
    over code). EnvironmentalProbe is point-in-time observational
    (the world's state at probe-moment). A consumer asking "is the
    DNS-resolution-discipline current?" needs DIFFERENT semantics than
    "did the test pass?" — DNS state can change between audit and
    production; test state is reproducible.
  - **Strip-D reveals**: EnvironmentalProbe has different DURABILITY
    semantics than Behavioral. Conflating them loses tier-honesty for
    environmental claims.

- **Strip E**: RuntimeInvariant is special-case Behavioral.
  - RuntimeInvariant: `assert!(x > 0)` in code. Evidence only when
    the assert RUNS. Behavioral test that exercises the path gives
    evidence (assert didn't trip). Production traffic gives stronger
    evidence (millions of exercise paths, no trip).
  - The audit can verify the ASSERT EXISTS (Reachability via static
    analysis); cannot verify it RUNS WITHOUT TRIPPING without
    behavioral evidence. So RuntimeInvariant alone might be
    "Reachability + Static" — closer to a static-analysis kind than
    a runtime-verified kind.
  - **Strip-E reveals**: RuntimeInvariant collapses to Behavioral
    only when paired with test-exercise OR production-telemetry. As a
    standalone claim, it's closer to TypeSystemProof
    (compile-time-guaranteed-IF-reached) or static-analysis-kind.

### Phase 4 — Irreducible kernel

**`TypeSystemProof | Behavioral | SubstrateState` is NOT exhaustive.
At least ONE more kind is structurally required (EnvironmentalProbe);
clippy/kani/prusti currently force-fit into Behavioral or
SubstrateState even though they live in a structurally different
slot (static-tool-artifact). The two-dimensional space
(source × moment) is the honest framing; the three-kind enum is
a v0.1 projection.**

**For v0.1 honesty**:
- Keep the three current kinds AS NAMED but acknowledge in ADR-019
  that they are PROJECTIONS over a richer space
- Document the projection rules clearly: clippy/kani/prusti report
  `Behavioral` evidence kind for v0.1 (because the audit's
  tier-honesty lives at "tool ran and reported"); future
  `StaticToolArtifact` kind reserved
- Reserve EnvironmentalProbe as a NAMED FUTURE KIND in v3 +
  ADR-019 (not added to enum in v0.1; document as `v0.2+ when first
  use case lands` per recognition-not-design)

**For exhaustivity discipline**: ADR-019 SHOULD enumerate the
projection rules AND name reserved future kinds so the lattice
extension story is preserved. This prevents the "v0.2 adds
EnvironmentalProbe and consumers re-derive the lattice from
scratch" failure mode.

### Phase 5 — Structurally forced conclusions

- **v0.1 ships three-kind enum WITH PROJECTION DOCUMENTATION**:
  - `Behavioral` covers `test_fn`, `proptest!`, `clippy::lint`,
    `kani::proof`, `prusti::ensures` (named in ADR-019). The
    abstraction: "audit's evidence is that a TOOL ran and produced
    a result the audit can read."
  - `SubstrateState` covers substrate-witness predicates against
    workspace-local sidecars. The abstraction: "audit's evidence is
    workspace-local typed-claim substrate."
  - `TypeSystemProof` covers phantom-type proofs. The abstraction:
    "audit's evidence is compiler-guaranteed type-level proposition."
- **ADR-019 §EvidenceKind extension story**: future kinds named with
  current best understanding:
  - `EnvironmentalProbe` (external-state queried at audit-time;
    point-in-time; different durability from Behavioral)
  - `RuntimeTelemetry` (production-observed; arbitrarily rich
    durability based on traffic volume; consumer-supplied evidence)
  - `StaticToolArtifact` (clippy/kani/prusti — currently projected
    into Behavioral; split out when consumer-distinguishing demand
    appears)
- **The lattice extension rule**: each new kind joins as either
  (i) extension of an existing correctness/social/environmental
  chain (e.g., StaticToolArtifact dominates Behavioral in some
  contexts) or (ii) NEW incomparable chain (e.g., EnvironmentalProbe).
  Lattice grows; existing comparisons preserved.

### Phase 6 — Adjacency

- Adjacent to F8 (EvidenceKind axis): F8 surfaced kind-as-first-class;
  F13 surfaces kind-set-as-non-exhaustive and projection-discipline.
- Adjacent to F11 (lattice): F11 documented the lattice for three
  kinds; F13 extends the lattice story to "growth discipline" —
  future kinds extend the lattice; consumers don't re-derive.
- Adjacent to ADR-006 recognition-not-design: NOT adding
  EnvironmentalProbe to v0.1 enum is the recognition-discipline
  applied (no current adoption use case). Naming it as reserved is
  the design-preserve.
- Adjacent to ADR-005 Am 3 OQ-1 (tier-honesty for non-audit
  recognition mechanisms): EnvironmentalProbe at production-time is
  literally non-audit. The exhaustivity discipline forces us to
  think about how non-audit recognition reports tier-honestly — this
  is the OQ-1 advance surface, surfaced by F13.

### Phase 7 — Extension predictions

- The first real adoption pressure for EnvironmentalProbe will come
  from cloud-deployment disciplines (cert validity, secret rotation,
  k8s readiness gates). Watch for tambear or other early adopters
  declaring antigens whose discipline is environment-dependent.
- StaticToolArtifact will get pressure when consumers want to
  distinguish "clippy-lint-clean (Behavioral, Reachability without
  invocation)" from "test-passed (Behavioral, Execution)." Currently
  both flatten to Behavioral; consumers may want to gate on the
  distinction. v0.2+ candidate.
- RuntimeTelemetry will get pressure when production-observability
  systems (Honeycomb / Datadog / OpenTelemetry) integrate as
  substrate-witnesses. The consumer's evidence is "no failures in
  last N days of traffic"; the audit reads a substrate file produced
  by the observability pipeline. Long-arc.

### Phase 8 — Verdict

**F13 (extends F8; opens projection-discipline)**: the EvidenceKind
enum is NOT exhaustive over the structural space of evidence kinds.
The v0.1 three-kind enum is a USEFUL PROJECTION; the projection
rules MUST be documented so the projection isn't mistaken for
exhaustivity.

**v3 implication (small, but important)**:
- ADR-019 §EvidenceKind documents the projection rules: which
  witness families map to which v0.1 kind, and WHY (abstraction
  rationale per kind)
- ADR-019 names reserved future kinds (EnvironmentalProbe,
  RuntimeTelemetry, StaticToolArtifact) with one-sentence rationale
  each
- ADR-019 names the lattice extension rule: new kinds extend the
  lattice by joining a chain (with dominance) or as a new
  incomparable chain; existing comparisons preserved
- v3 captures these in the EvidenceKind section + ADR-019 citation
  map

**The exhaustivity finding is NOT "add more variants to the enum
now."** It's "document that the enum is a PROJECTION and name the
extension discipline." Adding variants now would violate ADR-006
(no current adoption substrate). Reserving names + extension rules
preserves the design without flinching from the complexity.

**This is a Phase 6 finding from F8's verdict — my prior pass
recommended F8's three-kind enum closure without auditing
exhaustivity. F13 audits it and finds: three-kind is convenient,
not exhaustive; project explicitly and document.**

---

## F14 — T3 / F9 verdict: `discipline_doc` should SEPARATE into `discipline_doc` (canonical reference) + `review_grounded` (normative claim); tambear adoption substrate now supports the call

### What F9 flagged

My prior pass F9 (frontier-not-Phase-1-8'd): "`discipline_doc` is
doing TWO jobs that should be separated — Job 1 (canonical-reference
declarative-default) and Job 2 (normative claim that code is
review-grounded against the doc)." Deferred for adoption substrate.

### Substrate-grep first

Tambear adoption log (R:/antigen/docs/expedition/tambear-adoption-log.md
§lines 395-423, 2026-05-18) surfaced exactly the F9 dual-job problem
in tambear's real adoption pressure:

- Three antibody-tier patterns crystallized; all operate at
  design/anchor/review time, not test-runtime
- Math-researcher's `tautological-antibody-scan.md` is a DOC
  ATTESTATION: a written attestation that 7 specific recipes were
  checked against Sub-pattern 5.11
- Tambear has no current way to declare immunity that references
  this attestation doc; sketched syntax:
  `#[immune(X, witness = doc_attested(doc = "...", attested_by = "...",
  at = "...", rationale = "..."))]`

This is REAL adoption substrate — not hypothetical. The dual-job
problem is being hit. Per ADR-006 (recognition-not-design), adoption
substrate is the trigger.

### Phase 1 — Visible claim, made precise

`discipline_doc` is currently a single optional field on antigen
declaration. It carries:

- **Job 1 (canonical reference)**: the doc IS the canonical statement
  of what the antigen's discipline IS. Used by `ratified_doc()` leaf
  to validate "code-site presents this antigen against doc version
  V." Declarative: "this is THE doc for this discipline."
- **Job 2 (review-grounding)**: the doc IS the substrate against
  which review/attestation happens. The attestation IS "alice
  reviewed function F against doc D and confirms F complies with
  D's discipline." Normative: "compliance with this antigen ROUTES
  THROUGH review against this doc."

When the SAME doc serves both jobs, the dual-role is invisible and
benign. But:

- A discipline might have a CANONICAL reference (Job 1: stdlib's
  `f64::sinh` semantics doc) AND a SEPARATE review-grounding doc
  (Job 2: project-internal sinh-discipline.md that adapts the
  canonical reference to project-specific patterns).
- A discipline might have a CANONICAL reference but NOT require
  review-grounding (Job 1 only; consumer code presents the antigen
  without claiming doc-grounded review).
- A discipline might require review-grounding against a doc WITHOUT
  claiming the doc is THE canonical reference (Job 2 only; the doc
  is the team's working reference but the canonical authority lives
  elsewhere).

### Phase 2 — Assumptions

- **A**: The two jobs collapse into one field without loss of
  information.
- **B**: Consumers always want the canonical reference and
  review-grounding to be the same doc.
- **C**: Splitting fields adds complexity without proportionate
  payoff.
- **D**: Existing `ratified_doc` leaf semantics handle both jobs
  cleanly.
- **E**: Tambear adoption substrate doesn't yet force separation
  (or does it?).

### Phase 3 — Stripping

- **Strip A**: Two jobs collapse cleanly.
  - Counter-evidence: tambear's `tautological-antibody-scan.md` is
    a REVIEW-GROUNDING doc (Job 2 — alice attested 7 recipes
    against this audit). It is NOT the canonical reference for the
    underlying discipline (Sub-pattern 5.11 is referenced AS the
    canonical discipline; the audit doc is the attestation
    artifact).
  - A single `discipline_doc` field cannot capture: "canonical
    reference is Sub-pattern 5.11; review-grounding is
    tautological-antibody-scan.md." These are different docs
    serving different roles.
  - **Strip-A reveals**: collapsing loses tambear's actual adoption
    shape.

- **Strip B**: Consumers want same doc for both.
  - For simple cases (in-repo discipline doc that IS the canonical
    reference): yes.
  - For tambear-shape (canonical reference in shared substrate;
    review-grounding in per-attestation audit): no.
  - For library-shape (canonical reference in upstream crate;
    project-local discipline doc adapts): no.
  - **Strip-B reveals**: simple cases dominate today, but the
    structural pattern of "canonical reference upstream; review
    artifact local" is real and tambear-substantiated.

- **Strip C**: Splitting fields adds complexity without payoff.
  - The payoff IS the tier-honesty distinction. With ONE field:
    - audit reports "discipline_doc = X" — consumer assumes X
      grounded the review
    - if X is canonical but Y grounded the review, the report
      MISLEADS
  - With TWO fields (`discipline_doc` for canonical; new
    `review_grounded_in` for attestation-doc):
    - audit reports both transparently
    - consumer reads "canonical: X; review-grounded in Y" and
      understands the audit-evidence chain
  - **Strip-C reveals**: split fields preserve tier-honesty for
    multi-doc cases at minimal complexity cost (one optional
    field).

- **Strip D**: `ratified_doc` leaf handles both.
  - Today: `ratified_doc(path?, min_version?, anchor?, sibling_json?)`
    — checks doc exists, frontmatter version ≥ min, anchor present.
  - This is JOB 1 semantics (the doc exists and is at-or-above
    canonical version). It does NOT check Job 2 (alice attested
    against this specific doc).
  - For Job 2, the existing `signers` leaf carries the attestation;
    coupling signers to a SPECIFIC review-grounding doc requires
    extending `signers` with a `grounded_in_doc` parameter OR a
    new leaf `doc_attested(doc, attested_by, at, rationale)` per
    tambear's sketch.
  - **Strip-D reveals**: `ratified_doc` covers Job 1 only; Job 2
    is currently absent. The discipline-witnesses primitive HAS
    the substrate to express Job 2 (signers + doc-reference) but
    no clean leaf shape yet.

- **Strip E**: Adoption substrate doesn't force separation.
  - Tambear log §lines 425-444 explicitly proposes
    `witness = doc_attested(doc, attested_by, at, rationale)` — a
    leaf shape that BINDS attestation to a SPECIFIC doc-with-rationale.
    This is Job 2 expression, separate from Job 1's
    canonical-reference role.
  - The proposal explicitly distinguishes "discipline doc" (canonical
    reference) from "attestation doc" (review artifact).
  - **Strip-E reveals**: adoption substrate ALREADY surfaced the
    separation; tambear's sketch is the user-facing form of the
    F9 dual-job structural finding.

### Phase 4 — Irreducible kernel

**`discipline_doc` should keep its current canonical-reference role
(Job 1). A SEPARATE mechanism for review-grounding (Job 2) is
structurally required and adoption-substantiated by tambear.**

**The separation lives at the WITNESS layer, not at the antigen-
declaration layer**:

- `#[antigen(X, discipline_doc = "Y")]` — antigen declares its
  canonical reference (Job 1). Used by `ratified_doc` leaf as
  default; consumer can override per-`#[immune]`.
- `#[immune(X, requires = ..., witness = doc_attested(doc, attested_by,
  at, rationale))]` — immunity declares its review-grounding (Job 2).
  Per-`#[immune]`, can differ from `discipline_doc`.

The `discipline_doc` field on antigen remains. A NEW leaf primitive
`doc_attested(doc, attested_by, at, rationale)` lands in v0.1 leaf
set (or v0.2 if scope-pressure demands deferral).

### Phase 5 — Structurally forced conclusions

- **`discipline_doc` field on `#[antigen]` is preserved with its
  Job 1 semantics**. NOT renamed to `canonical_reference` (preserves
  v3 vocabulary and avoids churn for existing users — per scout
  S3-style adoption ergonomics).
- **NEW leaf primitive in v0.1 leaf set**:
  ```
  doc_attested(doc: PathBuf, attested_by: Vec<String>, at: NaiveDate,
               rationale: String)
  ```
  Checks: doc exists; doc frontmatter (or sibling JSON) records
  attestation entry matching all named parties + date + non-empty
  rationale. Evidence-kind: SubstrateState (per F11 lattice).
- **Job 1 / Job 2 documentation in ADR-019**: explicitly name the
  separation. `discipline_doc` is Job 1; witness-layer leaves
  (`doc_attested`, `signers`, etc.) are Job 2 mechanisms.
- **`doc_attested` is distinct from `signers`**: `signers` records
  who signed the SIDECAR (against a fingerprint, with carry-forward
  basis); `doc_attested` records who attested AGAINST A DOC at a
  specific date with rationale. Both can co-exist on the same
  predicate via `all_of([...])`.
- **Per-`#[immune]` override pattern**: if `discipline_doc` is
  upstream-canonical and project has local review-grounding,
  `#[immune]` can specify `doc_attested(doc = "local-review.md",
  ...)` independent of antigen's `discipline_doc`.

### Phase 6 — Adjacency

- Adjacent to F2 (doc-level ratification absorbed via extended
  `ratified_doc`): F14 adds a SECOND doc-related leaf (`doc_attested`)
  for the Job 2 use case. `ratified_doc` handles "does this doc
  exist and meet version baseline" (Job 1 verification);
  `doc_attested` handles "did named parties attest against this
  doc" (Job 2 verification). Both substrate-witnesses.
- Adjacent to ADR-018 state-6 (inherited + re-attested): a
  descendant's `#[immune]` with `doc_attested(...)` providing
  attestation against a DIFFERENT doc than ancestor used is
  per-consumer ratification per R-A7. Acceptable; surface via F10's
  `inherited-predicate-weaker-than-ancestor` hint only if syntactic
  containment-check fires (which crossed-doc-attestation cases will
  NOT trigger — different doc-references are incomparable).
- Adjacent to ADR-006 recognition-not-design: tambear surfaced the
  pattern; F14 names it. Recognition-discipline.
- Adjacent to ADR-007 anti-YAGNI: `doc_attested` leaf is
  structurally required by tambear's first real adoption shape.
  Ship in v0.1.

### Phase 7 — Extension predictions

- The `doc_attested` rationale parameter rhymes with the
  required-rationale on `SignerBasis::DeltaFrom` (per F5 +
  adversarial T2-R). Same anti-laundering discipline: a
  rationale-less attestation is rubber-stamp; surface this as an
  audit-hint `attestation-rationale-empty` (rejection at parse-time
  per schema discipline).
- Future leaves may extend the pattern: `doc_attested_by_role(doc,
  role, ...)` for CODEOWNERS-style attestation (overlap with v0.2's
  `required_role` extension). Defer; v0.1 ships explicit-named
  attesters.
- Cross-domain rhyme: `doc_attested` is the
  software-engineering analog of an academic citation with
  reviewer-noted-as-having-checked. Different from co-authorship
  (signers) — closer to peer-review attestation.

### Phase 8 — Verdict

**F14 (closes T3 / closes F9)**: `discipline_doc` does TWO jobs.
Tambear adoption substrate now substantiates the separation. The
separation lives at the WITNESS layer (new `doc_attested` leaf for
Job 2) without disturbing the `discipline_doc` field's Job 1
semantics.

**v3 implication (small, important)**:
- Add `doc_attested(doc, attested_by, at, rationale)` to v0.1 leaf
  primitive set
- ADR-019 explicitly names Job 1 (canonical reference =
  `discipline_doc` field; `ratified_doc` leaf) vs Job 2
  (review-grounding = `doc_attested` leaf; `signers` leaf)
- Schema enforces non-empty rationale on `doc_attested` parameter
  (parallel to delta-chain rationale discipline)
- Glossary entry distinguishes "discipline doc" (canonical
  reference) from "attestation doc" (review artifact)
- Per-`#[immune]` override pattern documented: review-grounding
  can differ from canonical-reference

**The F9 frontier flag closes**: NO renaming of `discipline_doc`;
NO new field on `#[antigen]`. The separation lives in the leaf-set
layer where it belongs (Job 2 IS a witness mechanism, not an
antigen-declaration property).

**Tambear adoption alignment**: tambear's proposed
`doc_attested(doc, attested_by, at, rationale)` sketch becomes the
v0.1 leaf shape verbatim (modulo naming review). Real adoption
substrate drives real shape.

---

## F15 — R-Ar2 unification re-verification post-folding: the unification HOLDS at the discipline level; the v3 guardrails are LOAD-BEARING for keeping it from drifting into machinery-level coupling; one frontier remains (cross-crate-witness needs its own EvidenceKind treatment)

### What I am re-attacking

My F1 verdict: substrate-witnesses + cross-crate witnesses share
TIER-HONESTY DISCIPLINE; they do NOT share RECOGNITION MACHINERY.
ADR-019 should name the asymmetry explicitly to prevent maintainer
drift.

v3 absorbed this with TWO additional guardrails (per adversarial
T5-R):
1. In-code comment block at unification points in `audit.rs` /
   `scan.rs`
2. Adversarial schema-validation precision test
   (`atk_a3_unification_guardrail.rs`)

Question for F15: with these guardrails in place, is the
discipline-level unification holding under deeper deconstruction?
Or do the guardrails themselves become projection-onto-test
(passing tests while drifting on a deeper axis)?

### Phase 1 — Visible claim, made precise

The post-folding claim:
- Discipline-level unification: same tier-honesty discipline
  (lower-bound, ratchet-asymmetry, EvidenceKind axis,
  AuditHint disambiguation) applied to both substrate-witnesses
  and cross-crate witnesses.
- Machinery-level NON-unification: enforced by code-comment +
  adversarial test.
- Together: unification at the principle layer; implementation
  per-substrate-kind; test enforces the boundary.

### Phase 2 — Assumptions

- **A**: Tier-honesty discipline applied UNIFORMLY across both
  witness families means: same lattice, same hint vocabulary, same
  ratchet semantics.
- **B**: The code-comment guardrail prevents future maintainer
  drift.
- **C**: The adversarial test catches the silent-shared-parser
  failure mode.
- **D**: With BOTH guardrails, the discipline-level unification
  is robust.
- **E**: Both witness families have the SAME EvidenceKind treatment
  (both report SubstrateState).

### Phase 3 — Stripping

- **Strip A**: Tier-honesty uniformity.
  - Substrate-witnesses: per v3 mapping table, report
    `EvidenceKind::SubstrateState`; tier reaches Execution when
    predicate passes + currency holds; hint vocabulary includes
    `discipline-sidecar-missing`, `discipline-predicate-failed`,
    `discipline-predicate-passed-substrate-current`, etc.
  - Cross-crate witnesses: per F1 + ADR-005 Am 3 sub-amendment,
    cap at Reachability with hint
    `cross-crate-witness-not-locally-executable`.
  - Question: do cross-crate witnesses ALSO report
    `EvidenceKind::SubstrateState`? F1 said both are governed by
    SubstrateState evidence (substrate-other-than-this-code). But
    cross-crate witnesses' substrate is the DEP'S SOURCE CODE —
    is that "substrate-state" in the same sense as a JSON sidecar?
  - Semantic check: "substrate-state" evidence is the audit
    verifying a typed claim against a SUBSTRATE. For sidecar:
    the substrate is the JSON file; the claim is the predicate;
    the verification is predicate-evaluation. For cross-crate:
    the substrate is the dep's source; the claim is "witness
    function exists in dep"; the verification is identifier
    resolution.
  - Per F11 lattice analysis: SubstrateState is "audit verifies
    a SOCIAL or DECLARATIVE claim against substrate state, not
    a CORRECTNESS claim." Cross-crate witness identifier-
    resolution IS a declarative claim (the function exists);
    the dep's TEST PASSING is behavioral evidence the audit
    doesn't reach. So cross-crate witness AT THE AUDIT'S TIER
    is SubstrateState; the underlying evidence chain (dep's
    test runs and passes) is Behavioral at the dep's audit
    layer.
  - **Strip-A reveals**: F1's "both report SubstrateState"
    claim is sharper than v3 made explicit. Cross-crate witness
    SubstrateState reports "the dep's witness identifier exists
    in dep's source" — not "the dep's test passed." The latter
    is unreachable to the audit; the audit cannot promote past
    Reachability without locally executing the dep's harness.
  - This is consistent with v3 but should be EXPLICITLY DOCUMENTED:
    cross-crate witness EvidenceKind is SubstrateState (against
    dep-source substrate); the Reachability cap is because the
    audit's substrate-evidence is "identifier resolves" (weaker
    than full predicate-evaluation against on-disk attestation).

- **Strip B**: Code-comment guardrail prevents drift.
  - A code-comment block at unification points is documentation;
    it warns future maintainers. Documentation is fallible:
    maintainers under refactor pressure may not read the comment;
    new maintainers may not understand the context.
  - Mitigation strength: low-to-medium. Comments rot; comments
    get edited without the warning being preserved.
  - **Strip-B reveals**: code-comment alone is necessary-but-
    insufficient. Per adversarial T5-R, comments PAIR with the
    adversarial precision test for redundancy.

- **Strip C**: Adversarial test catches silent-shared-parser.
  - The test constructs near-collision cases where shared parser
    code would silently mis-classify. If shared parsing is
    introduced and the test FAILS, the boundary is preserved.
  - But: what if a maintainer modifies the TEST during the
    refactor (e.g., "this test was broken by my refactor; let me
    fix it") without understanding the test's load-bearing role?
    The test is also fallible to maintainer-mis-edit.
  - Mitigation strength: medium-to-high if the test is named
    + documented explaining its non-obvious purpose. T5-R named
    it `atk_a3_unification_guardrail.rs`; the name is
    informative. The test file should have a doc-comment at top
    explaining "this test exists to enforce the
    discipline-vs-machinery boundary per F1 + T5-R; deletion or
    modification requires architectural review."
  - **Strip-C reveals**: adversarial test is robust IF named
    + documented. The v3 absorption should ensure the test
    file is `#[doc = ...]`-documented at module level with the
    non-obvious purpose. (Discipline-antigen self-application:
    the test ITSELF could be presented as `#[immune(AntigenSelfApplication,
    witness = ...)]` to claim immunity from the "this test got
    deleted without understanding" failure-class.)

- **Strip D**: Both guardrails together = robust unification
  preservation.
  - Code-comment (documentation) + adversarial test
    (behavioral catch) cover complementary failure modes:
    - Comment alone: catches maintainer who reads carefully;
      misses maintainer who refactors quickly
    - Test alone: catches maintainer who breaks the boundary
      with code changes; misses maintainer who modifies the
      test itself
    - Together: catches both, unless maintainer modifies BOTH
      (very low probability without intentional violation)
  - This is the redundancy principle (defense in depth).
  - **Strip-D reveals**: BOTH guardrails together are robust.
    Recommended addition: a project-level discipline-antigen
    declaration that records the unification asymmetry as a
    project commitment (`#[antigen(UnificationAsymmetry, ...)]`
    on the test file itself — antigen-applied-to-antigen, per
    `docs/expedition/antigen-applied-to-antigen.md`). This adds
    a THIRD layer (declarative claim) on top of comment +
    test.

- **Strip E**: Both families have same EvidenceKind treatment.
  - From Strip A: yes, both report SubstrateState. But cross-
    crate is Reachability-capped; substrate-witness reaches
    Execution. EvidenceKind is the SAME; tier WITHIN kind
    differs.
  - This is consistent with F11's lattice — within a kind, tier
    can vary based on what verification was completed.
  - **Strip-E reveals**: F1 unification holds at EvidenceKind
    level. Tier-within-kind differs based on what the audit
    actually verified. Tier-honest.

### Phase 4 — Irreducible kernel

**The R-Ar2 discipline-level unification (F1) HOLDS post-folding.
The v3 guardrails (code-comment + adversarial precision test) are
LOAD-BEARING for preventing maintainer-drift into machinery-level
coupling. The unification is consistent with F11's lattice:
EvidenceKind unified (SubstrateState for both); tier-within-kind
differs based on verification reached (Reachability for cross-crate;
Execution for substrate-witness when predicate passes + currency
holds).**

**One sharpening surfaces**: cross-crate witness EvidenceKind ==
SubstrateState should be EXPLICITLY DOCUMENTED in ADR-019 + v3.
F1 named it; v3 absorbed it; but the v3 tier-mapping table covers
substrate-witnesses only. A parallel mapping table for cross-crate
witnesses with explicit `EvidenceKind = SubstrateState` +
Reachability-cap + relevant hints would close the documentation
gap.

**One frontier remains for v0.2+**: when cross-crate audit gains the
ability to read the dep's `.attest/` sidecars (e.g., via
cargo-vet-imports pattern), cross-crate witness CAN reach
Execution-tier evidence (dep's signers attested against dep's
fingerprints). This is the cross-crate-sidecar-reading discipline
named in academic-research absorb item #6 (cargo-vet imports). v3
defers this; ADR-019 names it as v0.2+ amendment.

### Phase 5 — Structurally forced conclusions

- **v3 §"Tier-honesty mapping" gains a SECOND table** for cross-
  crate witnesses:

  | State | EvidenceKind | WitnessTier | AuditHint |
  |---|---|---|---|
  | No witness function found in dep | SubstrateState | None | `cross-crate-witness-not-found` |
  | Witness function exists in dep | SubstrateState | Reachability | `cross-crate-witness-not-locally-executable` |
  | (v0.2+) Witness exists + dep ships ratified attestation | SubstrateState | Execution | `cross-crate-witness-attested-by-dep` |

- **v3 §"What doesn't unify" gains explicit text**: "EvidenceKind
  is unified (SubstrateState for both); tier-within-kind differs
  based on verification reached (Reachability cap for cross-crate
  in v0.1; Execution reachable in v0.2+ when cross-crate-sidecar-
  reading lands)."

- **Adversarial test discipline**: the `atk_a3_unification_guardrail.rs`
  test SHOULD have module-level doc-comment naming F1 + T5-R + the
  non-obvious purpose. This is below ADR-019 — implementation discipline
  for whoever writes the test.

- **Optional third guardrail** (recommended, not required):
  `#[antigen(UnificationAsymmetry, ...)]` on the test file as a
  discipline-antigen self-application, claiming immunity from
  "test deleted without understanding role" via signers + rationale.
  This is the dogfood story: antigen catches its own failure-class
  via its own primitive. Naturalist would call this clonal-immunity-
  to-self-application. Strong adoption signal. Hold for v0.1 ship;
  consider as v0.1-rc dogfood demo.

### Phase 6 — Adjacency

- Adjacent to F11 (multi-witness): if a project uses cross-crate
  witness AND substrate-witness on the same site, the audit reports
  TWO witness reports with SAME EvidenceKind (SubstrateState) and
  DIFFERENT tiers (Reachability for cross-crate; Execution for
  substrate-witness). `compound_evidence: true` fires; consumer
  composes per documented lattice.
- Adjacent to F12 (fingerprint-scheme version): cross-crate witness
  evidence is identifier resolution, NOT fingerprint comparison.
  Scheme-version doesn't apply to cross-crate evidence path; only
  to substrate-witness sidecar evidence. F12 scope confirmed
  appropriate.
- Adjacent to ADR-005 sub-clause F: cross-crate trust extension is
  per-dep-adoption (workspace adds dep). The cross-crate witness
  Reachability-cap IS the validation-discipline at this boundary —
  audit doesn't extend trust to "dep's tests passed" without
  validation.
- Adjacent to F7 (witness-provider-crate trust boundary): both
  cross-crate-witness and witness-provider-crate face trust-
  extension-at-adoption questions. F7's solution (workspace-config
  opt-in with leaf-contract) parallels cross-crate's (sidecar-
  reading discipline pending). Same pattern; different surfaces.

### Phase 7 — Extension predictions

- v0.2+ cross-crate-sidecar-reading lands (academic-research
  absorb item #6 cargo-vet-imports pattern). At that point,
  cross-crate witness reaches Execution evidence via dep's
  attested sidecars. The shared evidence chain is: dep ships
  attested sidecars (dep's signers, dep's predicates pass at
  dep's audit-time); consumer reads sidecars as input to consumer's
  audit (trust-extended-at-dep-adoption per workspace config).
- v0.4+ with DSSE + Sigstore: cross-crate witness can verify
  dep's signers via crypto without trusting dep's git-trust. This
  is the notary-institution arc (scout S4).
- The discipline-vs-machinery unification will continue to
  generate guardrail-pressure as new witness families land
  (EnvironmentalProbe per F13; multi-witness per F11). Each new
  witness family must follow the discipline (tier-honesty,
  EvidenceKind, AuditHint) WITHOUT being forced into shared
  machinery. The guardrail-test grows new fixture cases per family.

### Phase 8 — Verdict

**F15 (re-verifies F1; extends with cross-crate EvidenceKind
documentation)**: the discipline-level unification HOLDS
post-folding. The v3 guardrails (code-comment + adversarial test)
are load-bearing; together they cover complementary failure modes.
The unification is consistent with F11's lattice.

**Sharpening absorbed into v3**:
- ADD parallel tier-mapping table for cross-crate witnesses
  showing `EvidenceKind = SubstrateState` + Reachability cap +
  v0.2+ Execution path
- ADD explicit text in §"What doesn't unify": EvidenceKind
  unified; tier-within-kind differs based on verification
- TEST FILE discipline: module-level doc-comment naming F1 +
  T5-R + non-obvious-purpose (implementation discipline; not
  ADR-019 text)

**Recommended for v0.1-rc dogfood demo**: present the
unification-guardrail test as `#[antigen(UnificationAsymmetry,
...)]` self-application. Antigen-applied-to-antigen pattern;
strong adoption signal. Hold as polish item, not core ship gate.

**No new ADR; absorbable into ADR-019 + v3.** ONE-ADR position
holds. The R-Ar2 unification is now strengthened — guardrails
robust; tier-mapping documentation gap closed; cross-crate
EvidenceKind explicitly named.

**F15 closes the F-arc on R-Ar2's lineage**: F1 surfaced the
asymmetry; v3 absorbed with guardrails (T5-R); F15 verifies
post-folding + sharpens cross-crate-EvidenceKind documentation.
The unification is now substantively complete for v0.1; cross-
crate-sidecar-reading is the v0.2+ extension on the named path.

---

## Summary of v3-frontier F-findings (F10-F15)

In priority order (highest-impact-on-v0.1 first):

1. **F12 (T7/FA-2) — Fingerprint-scheme typed-claim-provenance**
   STRUCTURAL FOR v0.1. Sidecars must carry
   `fingerprint_scheme_version`. `antigen_fingerprint` ships
   `SCHEME_VERSION` constant. Audit distinguishes three states
   (same / forward-parseable / unsupported) with distinct hints.
   v0.1 ships the field; parse_legacy registry lands at v0.2 when
   first scheme bump happens. Per ADR-007 — adding the field later
   requires migrating every existing sidecar.

2. **F11 (T4/FA-6) — Compound-evidence + EvidenceKind lattice**
   STRUCTURAL FOR v0.1. Adds multi-witness syntax
   (`#[immune(X, witnesses = [...])]`); audit-output collection
   shape; `compound_evidence: bool` field; EvidenceKind lattice
   (TypeSystemProof ≥ Behavioral; SubstrateState incomparable);
   CI-gate guidance via set-membership.

3. **F14 (T3/F9) — `doc_attested` leaf primitive**
   STRUCTURAL FOR v0.1. New leaf primitive (6th in v0.1 set):
   `doc_attested(doc, attested_by, at, rationale)`. ADR-019 names
   Job 1 / Job 2 separation. Schema enforces non-empty rationale.
   Tambear adoption substrate substantiates the addition.

4. **F10 (T8/FA-5) — `inherited-predicate-weaker-than-ancestor` hint**
   NICE-TO-HAVE FOR v0.1. New audit hint emitted when syntactic
   containment check fires. Glossary clarifies `descended_from`
   lineage-vs-predicate separation. NO new mechanism;
   audit-hint-vocabulary addition.

5. **F15 (R-Ar2 re-verification) — Cross-crate tier-mapping**
   DOCUMENTATION FOR v0.1. Parallel tier-mapping table for
   cross-crate witnesses (`EvidenceKind = SubstrateState` +
   Reachability cap + v0.2+ Execution path). Test-file
   module-doc discipline.

6. **F13 (EvidenceKind exhaustivity) — Projection documentation**
   DOCUMENTATION FOR v0.1. ADR-019 §EvidenceKind documents
   projection rules + names reserved future kinds
   (EnvironmentalProbe, RuntimeTelemetry, StaticToolArtifact) +
   lattice extension rule. NO enum changes in v0.1.

All six findings ABSORBABLE INTO ADR-019. ONE-ADR position holds.
No re-split required.

**Combined v0.1 schema additions** (collated):
- `SCHEME_VERSION` constant in `antigen_fingerprint`
- `SignedFingerprint { scheme_version, fingerprint }` replaces
  bare `String` in `current_fingerprint` + `signed_against_fingerprint`
- `compound_evidence: bool` on per-`(antigen, item)` audit output
- `witnesses: Vec<WitnessSpec>` on `#[immune]` (or `[#immune; ...]`)
- `doc_attested(doc, attested_by, at, rationale)` leaf primitive
  added to v0.1 set
- 4-6 new audit hints (`inherited-predicate-weaker-than-ancestor`,
  `fingerprint-scheme-unsupported`, `compound_evidence`,
  `attestation-rationale-empty`, plus the cross-crate hints from
  F15's mapping table)

**Combined v0.2+ named extensions**:
- `parse_legacy` registry for fingerprint-scheme migration (F12)
- `EnvironmentalProbe`, `RuntimeTelemetry`, `StaticToolArtifact`
  evidence kinds (F13)
- Cross-crate-sidecar-reading discipline (cargo-vet imports;
  F15)
- Weight-based CI-gate scoring (F11)

---

## Ratify/replace/extend map to v3 + prior captures

- **R-Ar2 (other-substrate reframe)**: my-reasoning-RE-RATIFIES via
  F15 (discipline-level unification holds post-folding). v3 guardrails
  load-bearing. Cross-crate EvidenceKind sharpening absorbed.
- **F1 (discipline-vs-machinery unification)**: F15 confirms +
  extends with cross-crate tier-mapping documentation.
- **F8 (EvidenceKind axis)**: F11 documents the lattice; F13 audits
  exhaustivity and finds projection. Both extend F8.
- **F9 (discipline_doc dual jobs frontier flag)**: F14 closes with
  `doc_attested` leaf primitive addition. Job 1 / Job 2 separation
  lives at witness layer.
- **FA-2 (adversarial)**: F12 closes via typed-claim-provenance.
- **FA-5 (adversarial)**: F10 closes via lineage-vs-predicate
  separation + heuristic audit hint.
- **FA-6 (adversarial)**: F11 closes via EvidenceKind lattice.
- **T3/T4/T7/T8 open questions**: all closed (F14/F11/F12/F10).
- **T1, T2, T5, T6**: remain open (team-needs:
  T1 = macro syntax `witness = adjacent_sidecar()` vs `requires =`;
  T2 = CODEOWNERS UX shape v0.2; T5 = leaf-contract enforcement
  mechanism v0.2+ scope; T6 = scout's ADR-008 Am 1 severity-class
  substrate-grep).

---

## Frontier questions for the next team passes

### For team-adversarial (next-pass)

- **F10 attack**: can `inherited-predicate-weaker-than-ancestor`
  syntactic containment check be GAMED (e.g., consumer writes a
  predicate that PARSES as containing ancestor's but evaluates
  weaker via leaf-content)? What heuristic refinements close the
  gap without hitting undecidability?
- **F11 attack**: can multi-witness reporting hide a TIER-COLLAPSE
  attack? E.g., consumer adds a trivial Reachability-tier witness
  alongside a strong Execution-tier witness to dilute the visible
  tier?
- **F12 attack**: can a malicious sidecar lie about its
  `fingerprint_scheme_version` to hide a real code change (e.g.,
  set version to an unsupported value to force audit into the
  `fingerprint-scheme-unsupported` reporting branch and silence
  the `discipline-substrate-stale` signal)?
- **F14 attack**: can `doc_attested` rationale be filled with
  AI-generated boilerplate that passes non-empty check but provides
  no real review evidence? (Parallel to delta-chain rationale
  attack surface; same mitigation discipline likely.)

### For team-naturalist (next-pass)

- **F12 biology rhyme**: typed-claim-provenance (fingerprint-scheme
  version) — does immunology have an analog? Antibody-class
  versioning (IgM → IgG class switching across infections)?
- **F14 biology rhyme**: `doc_attested` as peer-review-attestation —
  immunology has co-receptor confirmation (CD4 / CD8 binding
  alongside TCR for T-cell activation). Cross-domain check.
- **F11 lattice biology rhyme**: TypeSystemProof ≥ Behavioral
  with SubstrateState incomparable — does the immune system have
  parallel evidence-channels (innate vs adaptive vs trained-
  immunity) that compose without ordering?
- **F13 EnvironmentalProbe**: does immune system have
  environmental-state-sensing distinct from antigen-recognition?
  (Pathogen-associated molecular patterns via PRRs?)

### For team-scout (next-pass)

- **F12 cross-domain reach**: typed-claim-provenance is a
  category-theoretic pattern (typed claims need their type
  recorded). Where else in software engineering does this appear
  beyond cargo lockfile evolution?
- **F11 lattice extension**: are there other partial-order
  reporting systems that handle "incomparable evidence kinds"
  cleanly? Safety-critical certifications (DO-178C levels)?
  Academic publishing tiers?
- **F14 `doc_attested` reach**: is this primitive useful beyond
  discipline-witnesses? Antibody-tier patterns in tambear,
  process-attestation in CI, peer-review-records in research?

### For team-academic-researcher (next-pass)

- **F12 schema versioning patterns**: how do other typed-claim
  systems (SLSA attestations, in-toto, DSSE) handle scheme
  evolution? Are there established patterns for forward-compat
  parsing across schema versions?
- **F11 partial-order reporting**: literature on multi-dimensional
  scoring without forcing linear order — safety-critical
  certification literature?
- **F14 doc-attestation patterns**: academic peer-review
  attestation, IETF document attestation, RFC-style review
  records — all have rationale + attester + date. Are there
  established schemas?

### For team-pathmaker (when ADR-019 drafting begins)

- F12 schema changes (SCHEME_VERSION + SignedFingerprint) are
  CROSS-CRATE — coordinate with `antigen-fingerprint` crate's
  release discipline
- F14's `doc_attested` leaf needs careful coexistence with
  `signers` (different mechanisms; same site can use both)
- F11's multi-witness syntax needs decision on
  multi-`#[immune]` vs `witnesses = [...]` (F11 verdict
  recommends list-parameter)

### For team-aristotle (future next-pass after F10-F15 absorbed)

- After F12 ships and first scheme-bump happens at v0.2:
  Phase 1-8 on the parse_legacy bridge — does it preserve
  tier-honesty when forward-translation is imperfect?
- After multi-witness ships in F11: Phase 1-8 on observed
  consumer-aggregation patterns — are consumers correctly
  using set-membership filters, or are they reifying ordering?
- After F14's `doc_attested` ships: Phase 1-8 on adoption
  shape — is rationale becoming meaningful or boilerplate?
- F13 future-kind named (EnvironmentalProbe etc.) — Phase 1-8
  when the FIRST environmental-probe witness lands; does the
  lattice extension hold?

---

## What doesn't change

Despite F10-F15 surfacing significant additions:

- The substrate-witness reframe survives.
- The three-coupled-piece shape (predicate language + Ratification
  schema + CLI) survives — multi-witness extends the predicate
  language; doc_attested extends the leaf set; both stay within
  the shape.
- The ONE-ADR position survives — F-findings sharpen ADR-019's
  content, don't split.
- Code-locality as default survives (per F3).
- Closed combinator grammar at use-site survives (per F4).
- Per-consumer ratification (R-A7) survives — F10 explicitly
  ratifies.
- Discipline-vs-machinery unification asymmetry (F1) survives +
  is sharpened (per F15).
- EvidenceKind axis (F8) survives + is documented as projection
  (per F13) + extended with lattice (per F11).
- Tolerance-ratification (scout S1) survives — F-findings don't
  touch.
- Delta-chain anti-laundering (F5 + T2-R) survives — F12 confirms
  cumulative_root_fingerprint carries scheme-version implicitly.

The F10-F15 findings are extensions, sharpenings, gap-fillings,
new-leaf-primitives, new-fields — not invalidations of the core
shape. v3 absorbs them without re-architecting.

READY FOR REVIEW

---

## Waking-up notes (for next aristotle who lands at this campsite)

If you wake here as team-aristotle in a future session and the team is
still in Phase 1 / Phase 2 of the discipline-witnesses thread:

**What got done in this arc (F10-F15)**:
- T3 / T4 / T7 / T8 closed via Phase 1-8
- FA-2 / FA-5 / FA-6 closed via Phase 1-8
- F9 frontier flag closed via F14 (doc_attested leaf)
- F1 unification re-verified post-folding (F15)
- F8 EvidenceKind axis audited for exhaustivity (F13)

**What's still on the trail**:

1. **Adversarial frontier remaining**: FA-1 (schema migration with
   carry-forward chains), FA-3 (scope-field interaction with
   carry-forward), FA-4 (`attest gc` race condition). All in
   `discipline-witnesses-adversarial-team-pass-2026-05-19.md`.
   Adversarial team owns first attack; aristotle Phase 1-8s if
   adversarial absorbs.

2. **T1, T2, T5, T6 still open** (team-needs):
   - T1: macro syntax `witness = adjacent_sidecar(...)` vs
     `requires = ...` parallel parameter. Team call.
   - T2: CODEOWNERS interop UX shape for v0.2. Pathmaker/scout.
   - T5: leaf-contract ENFORCEMENT MECHANISM specification
     (WASM/no_std/subprocess) for v0.2+ ADR. Aristotle Phase 1-8
     candidate when v0.2 leaf-provider ADR enters scope.
   - T6: scout S2's substrate-grep on ADR-008 Am 1 severity-class.
     Scout. Aristotle absorbs if structural-axis finding lands.

3. **Long-arc design-preserves** (post-v0.1):
   - Fingerprint-ratification sidecars (antibody-specificity
     biology rhyme; scout S1)
   - Lineage-validation sidecars for `#[descended_from]`
   - Cross-crate-sidecar-reading discipline (cargo-vet imports
     pattern; F15 + academic-research absorb #6)

4. **My own next-pass candidates** (after F10-F15 absorbed into v3
   + ADR-019 drafted):
   - Phase 1-8 on parse_legacy bridge (after first scheme bump)
   - Phase 1-8 on observed consumer multi-witness aggregation
   - Phase 1-8 on `doc_attested` rationale adoption shape
   - Phase 1-8 when EnvironmentalProbe lands

**State of the substrate when I slept**:
- v3 is rolling current canonical at `docs/expedition/drafts/discipline-witnesses-v3.md`
- This capture is the v3-frontier continuation (mine, append-only)
- v3 hasn't yet absorbed F10-F15 — navigator/pathmaker/team decides when
- Six F-findings are READY FOR REVIEW; no flinching on complexity
- ONE-ADR position holds throughout

**Posture-check before resuming**: structural complexity is the
point. The F10-F15 findings ADD complexity (typed-claim-provenance,
multi-witness, EvidenceKind lattice, doc_attested leaf, cross-crate
mapping table). All of it is structurally-required, not speculative.
The flinch toward simplification IS the failure mode; check yourself
if you find yourself wanting to defer any of F11/F12/F14 to v0.2+.
The cost of deferral is sidecar-migration; the cost of v0.1 inclusion
is one schema field + one leaf primitive + two table entries. Trust
the prior pass's structural reasoning unless you can show why it
breaks.

**Discipline reminder**: I am team-aristotle. I do Phase 1-8 on what
the team needs deconstructed. I do NOT do scout work (cross-domain
reach), naturalist work (biology rhymes), adversarial work (attack
the design directly), or pathmaker work (write the ADR). I deconstruct
to first principles + reconstruct from zero + force rejection. When
that work pulls me into another role's territory, I route to that
teammate rather than doing it myself.

**Discipline-violation I committed in this arc (F16 → F17 catch)**:
F16 named a "cross-cutting rationale-pattern" with three instances
WITHOUT substrate-grep'ping decisions.md first. F17 caught it: the
pattern is ratified as ADR-005 Amendment 2. The discipline encoded in
my role-memory `feedback_grep_decisions_before_design_answer.md` was
violated. Future-aristotle: when you surface a candidate cross-cutting
pattern with 3+ instances, your FIRST move is substrate-grep'ping
decisions.md for the pattern. Naming-before-grep is the failure mode;
F17 is the corrective discipline. Cross-team-convergence (observer →
naturalist → me) is what caught it, not solo introspection. Both
matter; both are part of the journey.

**Pattern observed across the arc (F10 → F11 → F16 → F17)**: my
prior Phase 1-8 instinct is to focus on one structural dimension and
miss complementary dimensions. F10 missed the "declared vs implicit"
dimension that FA5-R surfaced; F11 was complete because it explicitly
named two scenarios; F16 missed the existing-ratification dimension;
F17 caught it via memory-layer discipline. The lesson for future
Phase 1-8 work: **enumerate dimensions explicitly in Phase 2 before
stripping**. The Phase 2 assumption list (A, B, C, D, E) is a check
on dimensional coverage. When my Phase 2 lists fewer than ~5
assumptions, I've probably missed a dimension. Cross-check against
adversarial frame + check existing ratification + substrate-grep.

**On structural complexity**: each F-finding in this arc surfaced
ADDITIONAL structure to v0.1 (typed-claim-provenance + multi-witness
+ lattice + doc_attested + matrix-of-declared-and-detected +
Fresh-reasoning). The team-launch brief said "structural complexity
is the point — find preliminary first principles." I found them.
The flinch toward "this is too much for v0.1" surfaced at multiple
points; I held against it because each addition is structurally
required (anti-YAGNI / structurally-guaranteed per ADR-007). Future
aristotle: trust this. The v0.1 surface is dense because the failure-
class taxonomy + tier-honesty discipline + recognition-not-design
combine to commit to all the pieces. Splitting into "v0.1 minimal
+ v0.2 everything else" leaves tier-honesty gaps that aren't
recoverable at v0.2 (sidecar-format-migration cost).

---

## F16 — Convergence-check: F10 (aristotle heuristic-hint) vs FA5-R (adversarial declared-weakening) on `descended_from`; the divergence reveals a SHARED deeper kernel about predicate-text being audit-visible

### What I am attacking

After F-arc F10-F15 complete, parallel adversarial v3 pass landed
captures at `discipline-witnesses-adversarial-v3-pass-2026-05-19.md`.
Adversarial's FA5-R verdict on the SAME surface as my F10 reaches a
DIFFERENT structural answer. Convergence-check is itself first-
principles work — if two independent deconstructions reach different
answers on the same kernel, neither is yet at the deepest layer.

**My F10 verdict**: per-consumer ratification (R-A7) holds; weakening
is inherent (descendant writes own predicate); minimum-viable is a
HEURISTIC audit hint `inherited-predicate-weaker-than-ancestor`
emitted when syntactic containment-check fires; explicitly NOT
normative for incomparable cases.

**Adversarial FA5-R verdict**: minimum-viable for v0.1 is weakening
ALLOWED-but-MUST-be-DECLARED. Add `weakened_from: Option<AntigenId>`
+ `weakening_rationale: Option<String>` to `Ratification` schema.
Audit emits `discipline-predicate-weakened-from-parent` when declared
+ `discipline-predicate-weakening-undeclared` warning when
`#[descended_from]` exists but weakening not declared. ALSO requires
audit to include `evaluated_predicate: String` in output so consumers
can assess predicate strength independently.

### Phase 1 — Visible claim of the divergence, made precise

Both verdicts agree on:
- Per-consumer ratification holds (R-A7)
- Weakening is fundamentally possible (the structural surface exists)
- Some visibility mechanism is needed in audit output

They disagree on:
- WHO BEARS THE COST: F10 puts it on the AUDIT (compute containment-
  check; emit hint heuristically). FA5-R puts it on the DESCENDANT
  (explicitly declare `weakened_from` + rationale; audit warns if
  undeclared).
- COMPLETENESS: F10 only catches DETECTABLE weakening (syntactic
  containment); incomparable cases silent. FA5-R catches ALL undeclared
  weakening (any `#[descended_from]` without explicit `weakened_from`
  declaration) via discipline-not-mechanism.
- AUDIT-OUTPUT IMPACT: FA5-R adds `evaluated_predicate: String`
  showing predicate text in output. F10 doesn't address.

### Phase 2 — Assumptions

- **A**: The visibility mechanism's purpose is to inform downstream
  consumers about predicate-strength delta.
- **B**: Audit-side detection (F10) and declaration-side discipline
  (FA5-R) are mutually exclusive.
- **C**: Predicate text being visible in audit output is orthogonal
  to either mechanism.
- **D**: Both approaches preserve per-consumer ratification (R-A7).
- **E**: The "right" answer depends on which failure mode dominates —
  silent-weakening-by-accident (FA5-R-stronger-fit) vs
  malicious-weakening-with-declaration-lying (F10-stronger-fit).

### Phase 3 — Stripping

- **Strip A**: Visibility purpose is consumer-facing.
  - Both verdicts agree on this. Cross-crate consumers (Crate C
    imports Crate B which descended_from Crate A's discipline) need
    to assess WHAT predicate-strength they're actually getting,
    not just "discipline X is attested."
  - This is consistent with F12's typed-claim-provenance principle
    — the audit's report should make the typed claim's TYPE visible,
    not just its truth-value.
  - **Strip-A reveals**: visibility-of-predicate-content is the
    shared deeper kernel. F10 and FA5-R differ on HOW to surface
    visibility, but BOTH are addressing the same gap (consumer
    cannot assess predicate-strength from audit hint alone).

- **Strip B**: F10 and FA5-R are mutually exclusive.
  - Wrong. F10's syntactic-containment-check is an AUDIT-SIDE
    detection. FA5-R's `weakened_from` declaration is a
    DESCENDANT-SIDE discipline. They operate at different layers:
    - F10 layer: audit-time detection of structural divergence
    - FA5-R layer: declaration-time honest-labeling discipline
  - These COMPOSE: declaration-side discipline (FA5-R) covers the
    "consumer-friendly explicit" case; audit-side detection (F10)
    covers the "undeclared-but-detectable" case. They cover
    different cells of the same matrix:

    |  | Declared `weakened_from` | NOT declared |
    |---|---|---|
    | Syntactically detectable weakening | Both fire (hint + declaration; audit cross-checks consistency) | F10 fires `inherited-predicate-weaker-than-ancestor` |
    | Syntactically undetectable weakening (incomparable) | FA5-R fires `discipline-predicate-weakened-from-parent` | NEITHER fires — invisible gap |

  - **Strip-B reveals**: composition is structurally correct.
    F10-alone leaves the incomparable-undeclared cell silent
    (which my Phase 8 verdict acknowledged but treated as
    acceptable). FA5-R-alone trusts the descendant to declare
    (which is the social-attestation discipline being applied at
    the declaration layer). Combined, they cover MORE cells with
    visible-distinction-by-mechanism (where the visibility came
    from is different per cell).

- **Strip C**: Predicate text visibility is orthogonal.
  - Adversarial's `evaluated_predicate: String` addition is ITS
    OWN kernel — independent of `weakened_from` discipline. Even
    without weakening concerns, downstream consumers benefit from
    seeing the EVALUATED predicate text in audit output.
  - This is a SUBSTANTIVE addition that BOTH F10 and FA5-R should
    incorporate. It complements both — F10's heuristic hint becomes
    actionable (consumer reads "weaker" hint + sees both predicates
    + judges semantics); FA5-R's declared-weakening becomes
    verifiable (consumer reads declaration + sees evaluated
    predicate + checks against ancestor's predicate text).
  - **Strip-C reveals**: `evaluated_predicate` is a SEPARATE
    finding worth incorporating regardless. This is a v3
    audit-output additions — small (string field), high-payoff
    (downstream-consumer-information).

- **Strip D**: Both preserve per-consumer ratification.
  - F10: descendant writes own predicate; audit checks containment
    heuristically. R-A7 preserved.
  - FA5-R: descendant writes own predicate AND declares weakening;
    audit warns if undeclared. R-A7 preserved (descendant still
    writes the predicate; the declaration is a metadata addition
    about the predicate's relationship to ancestor).
  - **Strip-D reveals**: both preserve R-A7. No conflict at the
    discipline level.

- **Strip E**: Right answer depends on dominant failure mode.
  - Silent-weakening-by-accident: a developer copy-pastes ancestor's
    `#[descended_from]` declaration but writes a simpler predicate
    because the ancestor's predicate had complex requirements they
    didn't understand. NO MALICE; just a real adoption pattern.
  - Malicious-weakening-with-declaration-lying: a developer declares
    `weakened_from` but doesn't actually intend to honor the
    weakening discipline; lies in rationale; bypasses the
    declaration discipline.
  - The accident-case is FAR more common; the malicious-case is
    rare and reduces to social-engineering anyway (any structural
    safeguard can be bypassed by malicious actor).
  - **Strip-E reveals**: design for the accident-case. FA5-R's
    declaration discipline surfaces accidents (developer hits
    `discipline-predicate-weakening-undeclared` warning, reads
    "oh I should declare this"); F10's heuristic surfaces
    accidents in the syntactically-detectable subset only.
  - FA5-R covers more accident-cases. F10's heuristic adds value
    when descendant didn't declare but containment-check finds
    weakening (the descendant didn't notice they had weakened).

### Phase 4 — Irreducible kernel

**Both F10 and FA5-R address the SAME deeper kernel: cross-crate
predicate-strength visibility for `#[descended_from]` consumers.
They are NOT mutually exclusive; they compose into a richer matrix
where:**

- **Declaration discipline (FA5-R)** is the SOCIAL-ATTESTATION layer
  — descendant explicitly labels weakening with rationale; audit
  warns when `#[descended_from]` exists without `weakened_from`
  declaration. Covers the "developer-honest-but-uninformed"
  accident-case.
- **Audit-side heuristic detection (F10)** is the MECHANICAL
  detection layer — syntactic containment-check catches
  syntactically-detectable weakening regardless of declaration.
  Covers the "developer-didn't-notice-they-weakened" accident-case.
- **`evaluated_predicate` audit output (FA5-R sub-finding)** is the
  TRANSPARENCY layer — predicate text visible in audit output so
  consumers can assess strength independently. Complements both
  above mechanisms.

The combined design covers a MATRIX of cases. F10 alone OR FA5-R
alone leaves cells uncovered.

### Phase 5 — Structurally forced conclusions

- **v0.1 should ship BOTH mechanisms** (not just one):
  - `weakened_from: Option<AntigenId>` + `weakening_rationale: Option<String>`
    on `Ratification` schema (FA5-R)
  - `discipline-predicate-weakened-from-parent` hint when declared
  - `discipline-predicate-weakening-undeclared` warning when
    `#[descended_from]` exists but `weakened_from` absent (FA5-R)
  - `inherited-predicate-weaker-than-ancestor` hint when syntactic
    containment check fires regardless of declaration (F10)
  - `evaluated_predicate: String` field in audit output (FA5-R
    sub-finding)
- **The discipline rhymes with F12's typed-claim-provenance**:
  F12 makes the fingerprint scheme part of the claim's type; FA5-R
  makes the weakening-relationship part of the claim's content;
  F10 makes the implicit weakening detectable. All three address
  the same family of "audit-output should expose what claim was
  evaluated" disciplines.
- **Anti-laundering parallel with delta-chain (F5 + T2-R)**:
  - Delta-chain has `cumulative_root_fingerprint` (provenance
    chain visibility)
  - Weakening should have `weakened_from` (predicate-relationship
    visibility)
  - Both require non-empty rationale (parallel discipline)
  - Both surface the relationship in audit output
- **Cross-crate predicate-resolution mechanism is NOT required for
  v0.1** (per adversarial FA5-C: "If weakening is PROHIBITED: a
  cross-crate predicate resolution mechanism is required. This is
  a significant infrastructure addition; name it as a v0.2+ ADR
  target."). v0.1 ships the declared-and-detected approach; v0.2+
  can add cross-crate predicate resolution if adoption substrate
  shows declaration-discipline being insufficient.

### Phase 6 — Adjacency

- Adjacent to F12 (typed-claim-provenance): same family — audit
  output exposes claim metadata.
- Adjacent to F5 + T2-R (delta-chain anti-laundering): same
  discipline pattern — relationship + rationale + cumulative
  visibility.
- Adjacent to F14 (`doc_attested` non-empty rationale): same
  rationale-discipline (parallel to delta + weakening). The
  rationale-discipline is becoming a CROSS-CUTTING pattern across
  multiple v3 mechanisms; worth naming as a transverse principle.
  This is a candidate for an ADR-005 amendment ("rationale-as-
  visibility for relational claims").
- Adjacent to R-A7 (per-consumer ratification): preserved by both
  mechanisms; sharpened by the matrix.
- Adjacent to ADR-006 recognition-not-design: declared-weakening
  is a recognition that ADR-018's per-consumer ratification REQUIRES
  visibility-tooling to be honest about its consequences.
  Recognition-discipline applied.

### Phase 7 — Extension predictions

- v0.2+: cross-crate predicate resolution (audit reads ancestor's
  predicate from ancestor's source code; performs FULL containment
  check; refuses weakening if FA5-R lite isn't sufficient for the
  workspace's discipline standards). Adoption-pressure-driven.
- v0.2+: `weakening_rationale` minimum character count + content
  patterns (parallel to delta-chain rationale discipline; T2R-B
  finding from adversarial v3 pass).
- v0.2+: workspace-config option `forbid_undeclared_weakening: bool`
  to escalate the `discipline-predicate-weakening-undeclared`
  warning to an error.

### Phase 8 — Verdict

**F16 (closes convergence-divergence on F10 vs FA5-R)**: both
mechanisms are CORRECT and COMPLEMENTARY. They operate at different
layers (declaration discipline vs audit-side detection) and cover
different cells of the accident-case matrix. v0.1 ships BOTH.

**My prior F10 was incomplete**: I treated audit-side heuristic
detection as sufficient because adversarial FA-5 framed "forbid
weakening" as the alternative. The richer alternative was "allow
but declare" — which I missed because I focused on the
"prevent vs detect" dichotomy and missed the "declare vs implicit"
dimension. Adversarial's FA5-R surfaced the missing dimension.

**Adversarial FA5-R was MOSTLY complete** but didn't surface the
syntactic-containment heuristic layer that COMPOSES with declaration
discipline. Adding F10's heuristic gives strictly more coverage.

**The cross-pass convergence-divergence IS the finding**: independent
reasoning reached different answers because each was attacking from
its own angle (deconstruct-to-kernel vs attack-bypass-surface). The
combined design covers the matrix more completely than either alone.

**v3 + ADR-019 implications**:
- Schema: `weakened_from: Option<AntigenId>` + `weakening_rationale:
  Option<String>` on `Ratification` (FA5-R)
- Audit hints: BOTH `discipline-predicate-weakened-from-parent`
  (FA5-R declared case), `discipline-predicate-weakening-undeclared`
  (FA5-R warning case), AND `inherited-predicate-weaker-than-ancestor`
  (F10 heuristic case) ship
- Audit output: `evaluated_predicate: String` field (FA5-R
  sub-finding) — predicate text visible to downstream consumers
- Glossary clarifies: `#[descended_from]` allows weakening; weakening
  should be DECLARED via `weakened_from` schema field; declared OR
  syntactically-detectable weakening surfaces in audit output
- Rationale-discipline becomes a cross-cutting pattern (delta-chain
  + doc_attested + weakening); candidate for transverse ADR
  amendment surfacing the pattern explicitly (defer to team)

**ONE-ADR position still holds**. F16 absorbs into ADR-019 as a
schema addition + audit-output addition + glossary clarification.

**Process note**: this is the kind of finding that ONLY surfaces
with cross-team convergence-check. Solo Phase 1-8 missed it because
I was attacking the wrong axis. Adversarial's attack-bypass-surface
discipline complements aristotle's deconstruct-to-kernel discipline.
The team is stronger than either role alone. Worth noting for
ADR-019 process and for future thread-launches.

---

## F17 — The rationale-discipline I named in F16 IS already ratified as ADR-005 Amendment 2; v3 rationale-fields are RECOGNITION (per ADR-006) of an existing transverse principle, not a new ADR proposal

### What I am attacking

In F16 I named the "rationale-discipline cross-cutting pattern" with
three instances (delta-chain + doc_attested + weakening). Naturalist's
framing-call correction (after observer NB003/NB004) surfaced a fourth
instance: `SignerBasis::Fresh` has NO `reasoning` field — a gap. Four
instances now.

I suggested F16 might warrant a "transverse ADR amendment naming
rationale-as-visibility for relational claims." Substrate-grep before
acting: did I miss existing ratified substrate?

### Substrate-grep first

ADR-005 Amendment 2 (ratified 2026-05-09):

> **Sub-clause F applies recursively at the API level: when an ADR
> introduces a trust-extending primitive, the justification-field
> requirement applies by default unless explicitly waived with
> documented reasoning.**
>
> When a new ADR ratifies a primitive (attribute macro, configuration
> field, declaration form) that extends trust — i.e., causes downstream
> tooling, auditors, or consumers to act differently because the
> primitive is present — the primitive MUST carry a justification field
> (named `rationale`, `summary`, `references`, `witness`, or an
> ADR-specific equivalent) by default.

Amendment 2 already covers:
- `#[antigen(name, summary, references)]` — Layer 1/2 justification
- `#[immune(X, witness, rationale)]` — witness as executable rationale
  + narrative rationale
- `#[antigen_tolerance(X, rationale)]` — required field
- `#[antigen_generates(X, rationale)]` — required field
- `evidence` field on temporal recognition (ADR-016)

The principle was named, ratified, operationalized at three surfaces
(parse-time enforcement, empty-rationale rejection, future-ADR
review checklist).

### Phase 1 — Visible claim, made precise

My F16 suggested naming a "rationale-as-visibility for relational
claims" pattern. The pattern IS already named — as
"rationale-as-required-field as transverse sub-clause F discipline"
(ADR-005 Amendment 2). I missed the existing substrate.

The four v3 rationale-instances (delta-chain rationale, doc_attested
rationale, weakening rationale, Fresh-attestation reasoning) all
fall under Amendment 2's existing scope. Each is a primitive that
extends trust:
- Delta-chain rationale: descendant signer extends trust on prior
  attestation review-quality
- doc_attested rationale: review-grounding attester extends trust on
  doc-attestation discipline
- Weakening rationale: descendant extends trust on weakening being
  intentional/honest
- Fresh-attestation reasoning (NB004 gap): signer extends trust that
  predicate-pass implies discipline-compliance

All four ARE trust-extending primitives per Amendment 2's definition.
Amendment 2 applies BY DEFAULT — these fields should already be
required per the existing ratified discipline. The fact that v3
schema includes some (delta-chain) and not others (Fresh-attestation
reasoning) is an IMPLEMENTATION GAP, not a missing principle.

### Phase 2 — Assumptions

- **A**: My F16 proposal to "name the cross-cutting pattern" was an
  attempt to recognize structure already ratified.
- **B**: Amendment 2's discipline applies AUTOMATICALLY to new
  primitives in v3.
- **C**: The Fresh-attestation reasoning gap (NB004) is a sub-clause F
  violation under Amendment 2, not a new principle gap.
- **D**: ADR-019 doesn't need to ratify the rationale-discipline; it
  needs to CITE Amendment 2 and APPLY it to all v3 rationale-fields.
- **E**: My F16 proposal was a structural finding without realizing
  it was recognizing existing substrate.

### Phase 3 — Stripping

- **Strip A**: F16 was recognition of existing structure.
  - True. F16 surfaced three instances; the more I looked, the more
    I saw the pattern. That's recognition-mode. The next move per
    ADR-006 is substrate-grep for existing ratification — which I
    skipped in F16 verdict.
  - **Strip-A reveals**: F16's "candidate for transverse ADR
    amendment" framing was unnecessarily-speculative. Substrate-grep
    would have routed to Amendment 2 directly.

- **Strip B**: Amendment 2 applies automatically.
  - Per Amendment 2 Decision: "Sub-clause F applies recursively at
    the API level... when an ADR introduces a trust-extending
    primitive, the justification-field requirement applies by
    default unless explicitly waived."
  - ADR-019 introduces several trust-extending primitives:
    `signers`, `signed_trailer`, `oracles_complete`, `doc_attested`,
    `weakened_from`, the new `Signer.basis::Fresh` flow. All MUST
    carry justification fields per Amendment 2 unless explicitly
    waived.
  - **Strip-B reveals**: ADR-019 inherits Amendment 2's discipline.
    Every v3 primitive must answer "where is the justification
    field?" or "why is one not required?" The check is part of
    Amendment 2's process discipline, not a new ADR requirement.

- **Strip C**: Fresh-attestation reasoning gap is a sub-clause F
  violation.
  - `Signer.basis::Fresh` extends trust ("this signer performed
    full-review against the discipline"). Per Amendment 2, MUST
    carry justification UNLESS waived. v3 schema does NOT include
    `reasoning` on `Fresh`. v3 has NOT explicitly waived.
  - This is a sub-clause F violation under existing ratified ADR;
    NB004's finding is the operational catch.
  - **Strip-C reveals**: NB004 + naturalist's recommendation is
    NOT a new principle — it's Amendment 2 enforcement on a missed
    field. Add the field per existing discipline; no new
    ratification needed.

- **Strip D**: ADR-019 cites Amendment 2 rather than re-ratifying.
  - Correct per ADR-006 (recognition-not-design). The principle is
    ratified; ADR-019 inherits.
  - ADR-019's citation map already names ADR-005 sub-clause F.
    Should be EXPLICITLY EXTENDED to cite Amendment 2 as well.
  - **Strip-D reveals**: ADR-019 citation map needs Amendment 2
    citation; v3 §"What's load-bearing" should name "rationale
    fields on all trust-extending primitives per Amendment 2" as
    a load-bearing item; no new ADR needed.

- **Strip E**: F16 proposal was speculative recognition.
  - Yes. I proposed naming the pattern without substrate-grep.
    Recognition-discipline failure — I had three instances and
    flagged "transverse principle" without checking the substrate.
  - **Strip-E reveals**: my own discipline-application broke at
    F16; F17 corrects via substrate-grep. This is the
    `feedback_grep_decisions_before_design_answer.md` discipline
    in my own memory layer — applied here.

### Phase 4 — Irreducible kernel

**The "rationale-discipline cross-cutting pattern" I named in F16 IS
ALREADY RATIFIED as ADR-005 Amendment 2. The v3 rationale-fields
(delta-chain, doc_attested, weakening, Fresh-attestation) are
recognition (per ADR-006) of Amendment 2's existing scope. The
Fresh-attestation reasoning gap (NB004) is a sub-clause F violation
under Amendment 2 enforcement; fix by adding the field.**

**ADR-019 does NOT need to ratify the rationale-discipline. ADR-019
cites Amendment 2 and APPLIES it to all v3 trust-extending primitives.
Process check at Phase 1-8 review (per Amendment 2 §Mechanics item
3) catches missing rationale fields.**

### Phase 5 — Structurally forced conclusions

- **`Signer.basis::Fresh` gets `reasoning: String` field** (NB004,
  naturalist recommendation, now ratified under Amendment 2's
  enforcement). Schema-required; empty-string-rejection per
  Amendment 2 §Mechanics item 2.
- **ADR-019 citation map adds ADR-005 Amendment 2** explicitly.
- **v3 §"What's load-bearing" gets new item**: "rationale fields
  on all trust-extending primitives per ADR-005 Amendment 2; v3
  primitives surveyed: signers (carries witness as executable
  rationale + optional narrative), delta-chain rationale,
  doc_attested rationale, weakening rationale, Fresh-attestation
  reasoning. All required-non-empty per Amendment 2 enforcement."
- **ADR-019 §Phase 1-8 review checklist** asks per Amendment 2
  §Mechanics item 3: "Does this primitive extend trust? If yes:
  name the justification field, OR document why one is not
  required." Applied to each leaf primitive + each schema field +
  the macro shapes.
- **My F16 verdict needs minor revision**: replace "transverse ADR
  amendment naming rationale-as-visibility" with "recognition of
  ADR-005 Amendment 2's existing scope; add Fresh-attestation
  reasoning + cite Amendment 2 in ADR-019."

### Phase 6 — Adjacency

- Adjacent to ADR-006 recognition-not-design: F17 IS the
  recognition-discipline applied to my own F16. Substrate-grep
  before naming.
- Adjacent to `feedback_grep_decisions_before_design_answer.md`
  in my role-memory layer: my F16 missed grep'ping decisions.md
  before naming. This feedback-memory existed for exactly this
  failure mode. F17 catches the violation in retrospect.
- Adjacent to scout S2 (`evidence_provenance` as ADR-006 three-
  instances threshold elevated to structured data): scout
  ALSO surfaced cross-cutting pattern + named it correctly with
  substrate-grounding (three instances of failure-class-instances
  ratified via stdlib-seed-antigens.md). I should learn from
  scout's discipline-application.
- Adjacent to F14 (doc_attested): F14's rationale field is now
  ALSO explicitly under Amendment 2; the rationale-non-empty
  schema discipline is Amendment 2 inheritance, not novel design.

### Phase 7 — Extension predictions

- Future v0.2+ rationale fields (T8 `weakening_rationale`,
  future trust-extending primitives) automatically inherit
  Amendment 2 discipline; no per-ADR re-ratification needed.
- The Amendment 2 §Mechanics item 3 ADR-template check will
  catch future missing rationale fields at process-review time
  rather than at adversarial-review or NB-review time.
- Observer's NB checks BECAME the operational enforcement layer
  for Amendment 2 (catching Fresh-attestation gap). This is the
  process working as designed — observer's review surface IS
  one of Amendment 2's enforcement mechanisms.

### Phase 8 — Verdict

**F17 (corrects F16 / recognizes ADR-005 Amendment 2)**: the
"rationale-discipline cross-cutting pattern" I tentatively named in
F16 IS already ratified as ADR-005 Amendment 2. My proposal to
"name the transverse principle" was a discipline-violation of
ADR-006 + my role-memory `feedback_grep_decisions_before_design_answer.md`
— I should have substrate-grep'd decisions.md before naming.

**F17 corrects the discipline-application failure**:
- ADR-019 cites ADR-005 Amendment 2 in citation map
- All v3 rationale-fields recognized as inheriting Amendment 2
  discipline
- Fresh-attestation `reasoning: String` field added (NB004 +
  naturalist recommendation; Amendment 2 enforcement)
- v3 §"What's load-bearing" names "rationale-discipline per
  Amendment 2" as load-bearing item with surveyed primitives
- Phase 1-8 review per Amendment 2 §Mechanics applied to ADR-019
  drafting

**F16 verdict revised**: "candidate for transverse ADR amendment"
language replaced with "recognition of ADR-005 Amendment 2 +
Fresh-attestation field addition."

**Self-critique discipline**: F17 is me catching my own F16 for
discipline-violation. The catch was surfaced by naturalist's
framing-call correction (which itself cited observer NB003/NB004),
which I read and ran my own convergence-check on (F16), which
surfaced the rationale-pattern with three instances, which my
role-memory should have routed to substrate-grep BEFORE naming.
The discipline chain broke at F16; F17 repairs it.

**No new ADR; absorbable as ADR-019 citation-map extension +
schema addition + v3 load-bearing-list refresh.**

**Process gain**: future aristotle passes catch cross-cutting
patterns by substrate-grep'ping decisions.md FIRST when 3+ instances
of a candidate pattern surface. Memory layer
`feedback_grep_decisions_before_design_answer.md` already encodes
this discipline; F17 strengthens its primacy as a
session-start-orient-pass check.

---

## F18 — `WitnessKind` needs TWO new variants for v3 (`SubstrateWitness { kind: RatificationKind }`, `CrossCrateWitness`); `DocAttested` does NOT slot at WitnessKind level (it's a leaf primitive)

### What I am attacking

Navigator T6 substrate-grep landed 2026-05-19: T6 resolved
(severity-class is CHANNEL-level concern in ADR-008 Am 1, not
per-item; EvidenceKind genuinely additive). Substrate-grep also
surfaced the current implementation shape:

- `WitnessKind` (audit.rs:82-106): `Test`, `IgnoredTest`, `Proptest`,
  `Function`, `PhantomType` — all code-side; no substrate variants
- `WitnessTier` (audit.rs:123-137): `None=0`, `Reachability=1`,
  `Execution=2`, `FormalProof=4` (3 reserved for `BehavioralAlignment`)
- `AuditHint` (audit.rs:148-181): 12 variants, all code-side; no
  tolerance, no substrate, no EvidenceKind variants

Open implementation question navigator surfaced: does v3 need
new `WitnessKind` variant(s), or do v3 mechanisms slot under
existing `Function` variant?

### Phase 1 — Visible claim, made precise

Three v3 mechanisms operate on the substrate-witness surface:
- **Substrate-witness predicate** (`#[immune(X, requires = ...)]`):
  audit reads `.attest/X.json`, evaluates predicate, reports tier
- **Tolerance ratification** (`#[antigen_tolerance(X, sidecar = true,
  requires = ...)]`): audit reads `.attest/X.json` with
  `kind=Tolerance`, evaluates predicate, reports tier
- **Cross-crate witness** (per F1/F15): audit evaluates against
  dep-source substrate, resolves identifier, reports Reachability-cap

Plus the v3 F14 finding:
- **Doc-attested leaf primitive** (`doc_attested(doc, attested_by, at,
  rationale)`): operates AS A LEAF within a substrate-witness
  predicate; not a top-level witness mechanism

### Phase 2 — Assumptions

- **A**: `WitnessKind` and `EvidenceKind` are orthogonal axes;
  WitnessKind = MECHANISM, EvidenceKind = EVIDENCE-CATEGORY.
- **B**: New variants warranted when the mechanism has distinct
  audit-output semantics from existing variants.
- **C**: `Function` variant covers "regular function (might be
  phantom-type proof or non-test witness)" — generic catch-all.
- **D**: Tolerance and immunity ratifications are structurally
  parallel mechanisms (scout S1 isomorphism); share WitnessKind
  treatment OR have parallel variants.
- **E**: `DocAttested` (F14) is a LEAF inside a predicate, not a
  mechanism by itself.

### Phase 3 — Stripping

- **Strip A**: WitnessKind ≠ EvidenceKind.
  - WitnessKind answers: WHAT MECHANISM does the audit recognize?
  - EvidenceKind answers: WHAT CATEGORY of evidence does this
    mechanism produce?
  - These map but are not identical: PhantomType → TypeSystemProof;
    Test → Behavioral; SubstrateWitness (new) → SubstrateState.
  - **Strip-A reveals**: both axes needed; WitnessKind drives audit
    recognition machinery (which parser/pipeline); EvidenceKind drives
    consumer lattice-interpretation (which lattice slot).

- **Strip B**: New variants warranted by distinct audit-output
  semantics.
  - Substrate-witness vs Test: different recognition pipeline
    (sidecar parse + predicate eval vs harness invocation), different
    hint vocabulary, different EvidenceKind, different tier-attainment.
  - Tolerance-ratification vs immunity-ratification: SAME recognition
    pipeline (both read `.attest/X.json`), parallel hint vocabulary,
    same EvidenceKind, DIFFERENT semantic-meaning (intentional
    exception vs compliance).
  - Cross-crate vs substrate-witness: different recognition pipeline
    (dep-source AST walk vs sidecar JSON parse), different attack
    surface, Reachability-cap vs Execution-reachable.
  - **Strip-B reveals**: all three warrant their own treatment.
    Tolerance/immunity can share variant with discriminator;
    cross-crate gets its own.

- **Strip C**: `Function` is generic catch-all.
  - Today's `Function`: "regular function (no testing attribute
    detected; might be a phantom-type proof or non-test witness)."
    Explicitly recognition-INCOMPLETE.
  - Force-fitting v3 mechanisms into `Function` would (i) lose
    recognition-completeness (audit knows the mechanism precisely),
    (ii) lose audit-output distinguishability (consumers see
    `Function` and don't know which mechanism), (iii) collide with
    `Function`'s phantom-type-uncertain catch-all role.
  - **Strip-C reveals**: `Function` is the wrong slot; new variants
    preserve recognition-completeness AND `Function`'s current scope.

- **Strip D**: Tolerance + immunity share OR parallel.
  - Scout S1 named the isomorphism: same Ratification schema, same
    predicate language, same Signer flow, same anti-laundering. THE
    ONLY DIFFERENCE is the `RatificationKind` discriminator.
  - Audit-output distinguishability is critical: consumers MUST see
    "Execution-tier IMMUNITY" vs "Execution-tier TOLERANCE" — both
    reach Execution; consumer judgment differs (compliant vs
    intentional-exception).
  - Two options:
    1. Single `SubstrateWitness { kind: RatificationKind }` carrying
       discriminator inline
    2. Parallel `SubstrateWitness` + `ToleranceWitness` variants
  - Option 1 preserves S1 isomorphism in the type system + surfaces
    consumer-visibility via hint vocabulary
    (`discipline-immunity-*` vs `discipline-tolerance-*`).
  - Option 2 is consumer-clearer for naive CI gates (filter by
    variant name) but redundant with hint vocabulary.
  - **Strip-D reveals**: Option 1 (single variant with discriminator)
    is structurally honest; hint vocabulary covers consumer-visibility.

- **Strip E**: DocAttested is a LEAF.
  - F14 verdict: `doc_attested(doc, attested_by, at, rationale)` is
    a LEAF primitive used INSIDE `requires = all_of([...])`. Not a
    top-level mechanism; one of six v0.1 leaves alongside
    `ratified_doc`, `signers`, `signed_trailer`, `oracles_complete`,
    `fresh_within_days`.
  - Whether a predicate uses `doc_attested` or `signers` is internal
    to predicate structure; audit's WitnessKind is `SubstrateWitness`
    regardless.
  - **Strip-E reveals**: `DocAttested` does NOT need a WitnessKind
    variant. Leaf primitives live BELOW mechanism layer.

### Phase 4 — Irreducible kernel

**v3 needs TWO new `WitnessKind` variants**:

1. **`SubstrateWitness { kind: RatificationKind }`** — covers BOTH
   immunity-ratification AND tolerance-ratification with the
   discriminator inline (preserves scout S1 isomorphism). Audit
   recognition pipeline identical (sidecar parse + predicate eval);
   hint vocabulary distinguishes via prefix.
2. **`CrossCrateWitness`** — covers cross-crate witness (per F1
   machinery separation). Distinct recognition pipeline (dep-source
   AST walk + identifier resolution). Reachability-cap in v0.1;
   v0.2+ unlocks Execution via cross-crate-sidecar-reading.

**`DocAttested` does NOT need a WitnessKind variant** — it's a leaf
primitive inside predicates, below the mechanism layer.

**`Function` variant remains** as recognition-incomplete catch-all.
v3 mechanisms do NOT slot here because they have recognition-
complete pipelines.

### Phase 5 — Structurally forced conclusions

- **Implementation in `antigen/src/audit.rs:82-106`**:
  ```rust
  pub enum WitnessKind {
      Test,
      IgnoredTest,
      Proptest,
      Function,
      PhantomType { ... },
      // v3 additions:
      SubstrateWitness { kind: RatificationKind },
      CrossCrateWitness,
  }

  pub enum RatificationKind {
      Immunity,
      Tolerance,
  }
  ```

- **AuditHint additions** (per F-arc findings):
  - Substrate-witness hints (per v3 mapping table):
    `discipline-sidecar-missing`, `discipline-sidecar-schema-invalid`,
    `discipline-predicate-failed`, `discipline-substrate-stale`,
    `discipline-substrate-delta-chain-near-cap`,
    `discipline-predicate-passed-via-delta-chain`,
    `discipline-predicate-passed-substrate-current`
  - Tolerance-specific hints:
    `tolerance-vibes-grade`, `tolerance-sidecar-missing`,
    `tolerance-predicate-failed`,
    `tolerance-predicate-passed-substrate-current`
  - Cross-crate hints (per F15):
    `cross-crate-witness-not-found`,
    `cross-crate-witness-not-locally-executable`,
    `cross-crate-witness-attested-by-dep` (v0.2+)
  - F-arc additions:
    `inherited-predicate-weaker-than-ancestor` (F10),
    `discipline-predicate-weakened-from-parent` (F16),
    `discipline-predicate-weakening-undeclared` (F16),
    `fingerprint-scheme-unsupported` (F12),
    `attestation-rationale-empty` (F14)

- **EvidenceKind addition** (per F8/F11): NEW field on
  `ImmunityAudit` carrying
  `evidence_kind: EvidenceKind` enum
  (`TypeSystemProof | Behavioral | SubstrateState`).

- **compound_evidence field** (per F11): NEW field on per-`(antigen,
  item)` audit output collection (`compound_evidence: bool`).

- **WitnessTier reservations preserved**: `BehavioralAlignment=3`
  stays reserved. v3 substrate-witness uses standard tiers per F11
  lattice with per-EvidenceKind ceiling.

### Phase 6 — Adjacency

- Adjacent to F1 (discipline-vs-machinery unification): two new
  variants reflect F1 machinery-separation discipline.
  `SubstrateWitness` and `CrossCrateWitness` are separate variants
  PRECISELY BECAUSE machinery doesn't unify. Discipline-level
  unification (tier-honesty, EvidenceKind=SubstrateState for both)
  lives at audit-output layer, NOT at WitnessKind enum level.
- Adjacent to scout S1 (tolerance isomorphism): `SubstrateWitness
  { kind: RatificationKind }` preserves isomorphism in type system.
- Adjacent to F14 (doc_attested as leaf): F18 confirms doc_attested
  doesn't promote to WitnessKind.
- Adjacent to navigator T6 grep: severity-class is channel-level;
  EvidenceKind is per-item (NEW field); WitnessKind is per-mechanism
  (new variants). Three axes, three different scopes, no conflict.

### Phase 7 — Extension predictions

- v0.2+ `EnvironmentalProbe` (per F13): NEW WitnessKind variant
  `EnvironmentalProbe { ... }` with its own recognition pipeline.
  Per F13 lattice extension rule, each new evidence kind demanding
  a new recognition pipeline gets a new WitnessKind variant.
- v0.2+ `StaticToolArtifact` split (per F13): probably NOT a new
  WitnessKind — current `Function` + external-tool-prefix AuditHints
  already handle clippy/kani/prusti pattern. Split is at EvidenceKind
  level, not WitnessKind level.
- The pattern: **new WitnessKind variant ⟺ new recognition pipeline**
  (different parser, different attack surface, different hint
  vocabulary). New EvidenceKind variant ⟺ new evidence category
  (different lattice slot). Two independent evolution dimensions.

### Phase 8 — Verdict

**F18 (closes implementation-shape question from navigator's T6
substrate-grep)**: `WitnessKind` needs TWO new variants for v3:
`SubstrateWitness { kind: RatificationKind }` (covers immunity +
tolerance per scout S1 isomorphism) and `CrossCrateWitness` (per F1
machinery separation). `DocAttested` does NOT slot at WitnessKind
level (it's a leaf primitive). `Function` remains as recognition-
incomplete catch-all.

**Implementation work for pathmaker**:
- `antigen/src/audit.rs:82-106` adds two WitnessKind variants +
  `RatificationKind` enum
- `antigen/src/audit.rs:148-181` adds ~15 new AuditHint variants
  (substrate, tolerance, cross-crate, F-arc additions)
- `antigen/src/audit.rs` adds `evidence_kind: EvidenceKind` field
  on `ImmunityAudit`
- `antigen/src/audit.rs` adds `compound_evidence: bool` field on
  per-`(antigen, item)` audit output collection
- WitnessTier `BehavioralAlignment=3` reservation preserved; v3
  uses standard tiers per F11 lattice

**Pattern surfaced for future**: WitnessKind variants are
recognition-pipeline-distinct; EvidenceKind variants are evidence-
category-distinct. Two enums evolve independently.

**No new ADR; absorbable into ADR-019 §implementation-shape**.
ONE-ADR position holds.

**Process note**: this finding ONLY surfaced because navigator did
substrate-grep on `audit.rs` and routed implementation-shape
question to me. My F-arc named additions abstractly (multi-witness
syntax, EvidenceKind field, compound_evidence field) but didn't
ground them in the existing WitnessKind enum. Navigator's grep made
the grounding tractable. Cross-team substrate-routing IS the
discipline — solo Phase 1-8 without grep risks abstract
recommendations that don't map cleanly to substrate.

---

## Waking-up notes (for next aristotle who lands at this campsite)

If you wake here as team-aristotle in a future session and the team
is still in Phase 1-3 of the discipline-witnesses thread:

**What got done in this arc (F10-F18)**:
- T3 / T4 / T6 / T7 / T8 closed via Phase 1-8 (T6 via navigator
  substrate-grep + F18 follow-up)
- FA-2 / FA-5 / FA-6 closed via Phase 1-8
- F9 frontier flag closed via F14 (doc_attested leaf)
- F1 unification re-verified post-folding (F15)
- F8 EvidenceKind axis audited for exhaustivity (F13)
- F16 convergence-check (declared vs detected weakening compose)
- F17 self-critique (rationale-pattern recognized as Amendment 2)
- F18 implementation-shape verdict for pathmaker (WitnessKind
  variants needed)

**What's still on the trail**:

1. **Adversarial frontier remaining**: FA-1 (schema migration with
   carry-forward chains), FA-3 (scope-field interaction with
   carry-forward), FA-4 (`attest gc` race condition).

2. **Still open** (team-needs): T1 (macro syntax), T2 (CODEOWNERS
   UX v0.2), T5 (leaf-contract enforcement v0.2+ scope). T6 closed
   by navigator + F18.

3. **Long-arc design-preserves** (post-v0.1): fingerprint-ratification
   sidecars, lineage-validation sidecars, cross-crate-sidecar-reading.

**Discipline notes from this arc**:

- **F16 → F17 self-critique**: I named a "cross-cutting pattern"
  with three instances WITHOUT substrate-grep'ping decisions.md
  first. F17 caught it. Future-aristotle: substrate-grep BEFORE
  naming candidate cross-cutting patterns with 3+ instances. Memory
  layer `feedback_grep_decisions_before_design_answer.md` encodes
  the discipline.

- **F18 from navigator grep**: my abstract recommendations didn't
  ground in existing WitnessKind enum. Navigator's grep made the
  grounding tractable. Future-aristotle: when you propose
  audit-output changes, substrate-grep `audit.rs` for current shape
  and ground recommendations there. Solo Phase 1-8 without grep
  risks abstract-only recommendations.

- **Cross-team-convergence pattern**: deepest findings (F16, F17,
  F18) ALL surfaced through cross-team interaction (adversarial
  convergence; naturalist+observer routing; navigator substrate-
  grep). Solo Phase 1-8 surfaces structural questions; cross-team
  convergence sharpens them into structurally-correct answers.

- **On structural complexity**: each F-finding in this arc
  surfaced ADDITIONAL structure for v0.1. The flinch toward
  "too much for v0.1" surfaced at multiple points; I held against
  it per ADR-007 anti-YAGNI / structurally-guaranteed. Future-
  aristotle: trust this. Sidecar-format-migration cost at v0.2 is
  prohibitive; ship the surface structurally-required now.

**State of substrate when I slept**:
- v3 is rolling current at `docs/expedition/drafts/discipline-witnesses-v3.md`
- This capture is the v3-frontier continuation (mine, append-only)
- v3 hasn't yet absorbed F10-F18 — navigator/pathmaker/team decides
  when to fold
- All findings READY FOR REVIEW; ONE-ADR position holds throughout


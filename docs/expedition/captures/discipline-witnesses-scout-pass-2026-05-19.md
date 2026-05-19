# Capture — Scout Pass on Discipline-Witnesses F2/F3/F8 Frontier Questions

> **Date**: 2026-05-19
> **Author**: scout (Claude Sonnet 4.6)
> **Relation to prior captures**: this is the scout pass explicitly requested by
> aristotle's team-pass F-findings. The three frontier questions are starting
> material; the wandering follows whatever catches attention from there.
> **Source substrate read**:
> - `discipline-witnesses-v2.md` (canonical draft)
> - `discipline-witnesses-2026-05-18.md` (design conversation capture)
> - `discipline-witnesses-aristotle-team-pass-2026-05-18.md` (F1-F9)
> - `discipline-witnesses-academic-research-2026-05-18.md` (14-system landscape map)
> - `docs/decisions.md` (ADR index, headers)
> - `docs/expedition/stdlib-seed-antigens.md`
> - `docs/expedition/failure-class-instances.md` (opening)
> - `docs/expedition/multi-component-immunity.md` (opening)
> - `docs/expedition/deferred-substrate.md` (opening)
> - `antigen/src/audit.rs` (current implementation shape)
> **Status**: append-only capture

---

## Orientation — what I'm walking into

Aristotle surfaced F1-F9. For scout specifically:

- **F2 absorption-pattern generalization**: typed-JSON-sidecar-adjacent-to-substrate is
  a pattern. Where else in antigen does it apply? Could it replace special-case mechanisms?
- **F3 scope-as-first-class**: discipline-scope is one such dimension. What OTHER first-class
  dimensions are implicit-but-should-be-explicit?
- **F8 EvidenceKind reach**: where does evidence-kind matter BEYOND witnesses? Does the
  antigen declaration itself have an evidence-kind?

Plus: pull in cross-domain rhymes the prior passes missed. Naturalist covered immunology;
aristotle covered first-principles; academic-research covered attestation systems. What's
left?

My instinct is to wander rather than answer the frontier questions in order. Start with
F2 because it's the one with codebase terrain to walk, then F3, then F8, then the
cross-domain tangent.

---

## S1 — Walking the codebase for F2 absorption candidates

The F2 pattern: **typed JSON sidecar carrying typed claim about adjacent substrate;
substrate is any file the audit can read**.

Aristotle's framing was about doc-level ratification absorbing into this pattern. But the
claim is broader: this pattern generalizes beyond discipline-witnesses. Let me walk the
terrain.

### What the current audit reads

From `antigen/src/audit.rs`, the audit currently:
- Walks `.rs` files looking for antigen declarations, presentations, immunity claims
- Resolves witness identifiers to functions in the workspace
- Reports `WitnessStatus` per immunity claim

The audit is `.rs`-centric. The substrate it reads is Rust source. The witness validation
it performs is identifier-resolution (does this identifier resolve to a real function/test?).

### The `#[antigen]` declaration itself as a sidecar candidate

Here's the tangent I wasn't expecting to find:

The `#[antigen(...)]` macro currently carries:
- `name`, `family`, `summary`, `fingerprint`, `references`

These live IN the `.rs` source file. But consider what the `fingerprint` field actually is:
a string DSL describing "what code shape triggers this." That's a typed claim about a class
of code sites — structurally identical to what a sidecar JSON would carry. The declaration
is an inline sidecar.

**The absorption question**: could `#[antigen]` declarations themselves be expressed as
typed JSON sidecars adjacent to any source file (`.rs` OR other), with the macro becoming
a thin pointer (`#[antigen(ref = "antigens/SignedZeroDiscipline.json")]`)? The sidecar
carries the rich declaration; the in-code macro is lightweight.

This is the F2 generalization applied to the OTHER direction: not just sidecars adjacent
to `.rs` files, but the declaration layer itself moving to sidecars.

**Why this matters**: in `stdlib-seed-antigens.md`, each antigen is declared on a `pub
struct`. The struct IS the declaration site. But for ecosystem-level antigens (antigen-stdlib),
the declaration IS the artifact — there's no "adjacent code" being annotated; there's
just the declaration. The sidecar-pattern fits naturally: `antigens/PolarityInvertedClassMeet.json`
carrying the full declaration, with `pub struct PolarityInvertedClassMeet` in source
holding nothing but the name.

I'm not recommending this for v0.1. But the structural rhyme is there: **the sidecar
pattern appears at BOTH the attestation layer (`.attest/` sidecars for immunity claims)
AND the declaration layer (JSON sidecars for antigen definitions)**. Two ends of the
antigen lifecycle sharing the same primitive shape.

### The `#[antigen_tolerance]` annotation as a sidecar candidate (ADR-011)

ADR-011 introduces `#[antigen_tolerance(...)]` for opt-out of fingerprint matches.
The current shape: an attribute on the code site with an inline rationale.

The discipline gap: rationale lives in the attribute; the audit reports it; but there's
no structured record of "who approved this tolerance, when, against what review". It's
a vibes-grade escape valve dressed up as structured opt-out.

This is EXACTLY the substrate-witness problem at the tolerance layer. Apply the
F2-pattern: a `.attest/`-adjacent sidecar for tolerance ratifications. `#[antigen_tolerance(ToleranceName)]`
with a JSON sidecar in `.attest/` carrying who approved, when, against what fingerprint,
with a freshness window.

The two-sentence version: **tolerance-without-attestation is a tier-honesty gap.** The
discipline-witness substrate-witness pattern plugs it, IF tolerance sidecars follow the
same schema.

**Concrete implication**: `cargo antigen attest` CLI should include tolerance-scaffold
commands parallel to immunity-scaffold commands. Tolerance ratification is a discipline
claim, same as immunity. The schema is isomorphic.

### The `#[descended_from]` inheritance graph as a sidecar candidate (ADR-018)

ADR-018 introduces `#[descended_from]` for antigen lineage inheritance.

The lineage graph is currently implicit in the declarations: you read all the
`#[descended_from]` attributes and reconstruct the graph. But the audit-time claim "this
antigen is descended from that antigen with these inherited properties" is itself a
typed claim. It could be a sidecar.

More specifically: when an antigen crate declares `#[descended_from(ParentAntigen)]`,
there is an implicit trust-extension happening (ADR-005 sub-clause F: every trust boundary
requires validation). The child is claiming to inherit the failure-class shape of the
parent. That claim should have attestation: who validated that the inheritance relationship
is appropriate? Is `LockOrderInversionVariant` genuinely a descendant of `LockOrderInversion`
or just structurally similar? A human (or review process) decided. That decision is
discipline-claim territory.

I'm NOT saying every `#[descended_from]` needs a sidecar in v0.1. I'm saying the
**lineage-validation claim has the same structural shape as the immunity claim**, and
the substrate-witness pattern applies to both. Long-term, `#[descended_from]` may want
attestation sidecars for the "this inheritance was reviewed" claim.

### The fingerprint grammar itself as a sidecar candidate

ADR-010 through ADR-015 build the fingerprint grammar — the DSL for describing what
code shape an antigen matches. The fingerprint is currently a string field inside the
`#[antigen]` macro.

But fingerprints can be wrong. A fingerprint that's too broad generates false positives;
too narrow misses true positives. Someone has to validate that the fingerprint actually
matches the intended failure pattern. That validation is a discipline claim.

The `stdlib-seed-antigens.md` document is honest about this: "Pseudocode declarations.
Syntax conforms to ADR-009 and ADR-010. The team refines and ratifies during sweep A5."
"Refines and ratifies" is exactly the discipline-witness process. Fingerprint ratification
sidecars.

This one has a biological rhyme the prior passes didn't name: **antibody specificity
validation**. In immunology, an antibody is validated not just by "does it bind?" but
"does it bind ONLY what we want?" (no cross-reactivity). Fingerprint specificity validation
is exactly this — the fingerprint must match the right sites and not the wrong ones.

### Synthesis for F2

The F2 absorption pattern is more general than aristotle's framing suggested:

1. **Immunity claim attestation** — the v2 draft's primary use case (`.attest/` sidecars)
2. **Tolerance ratification** — ADR-011's opt-out, currently vibes-grade; sidecar pattern
   fixes it
3. **Lineage validation** — ADR-018's `#[descended_from]`, eventually; inheritance
   relationships are discipline claims
4. **Fingerprint ratification** — ADR-010/015's fingerprint grammar; "this fingerprint
   actually matches the right sites" is a discipline claim

The sidecar pattern is the **universal typed-claim-about-adjacent-substrate primitive**.
It's not just discipline-witnesses — it's anywhere antigen needs to record "a human (or
process) reviewed and ratified this claim, against this substrate, at this time."

**For v0.1**: cases 1 and 2 are in scope. Cases 3 and 4 are structurally guaranteed
eventually but don't block the rc. The schema for case 2 (tolerance sidecars) is isomorphic
to case 1; shipping one schema that covers both is the minimum-additional-complexity move.

---

## S2 — Walking the ADR substrate for F3: other implicit-uniform dimensions

Aristotle named scope (site/file/module/crate/workspace). Let me walk the ADR substrate
for other dimensions.

### Dimension candidate: Certainty-class / Evidence-provenance of the antigen declaration itself

From `failure-class-instances.md` and `stdlib-seed-antigens.md`: some antigens are
declared based on N real-world observed failure instances (empirical); others are declared
based on structural reasoning about what COULD go wrong (theoretical prediction).

ADR-006 (recognition-not-design) establishes the discipline: "adding antigens to the
stdlib requires showing real-world instances." But the declaration doesn't carry the
provenance: was this antigen added because of 3 confirmed instances, or because it seemed
like a thing that could go wrong?

This is the F8 angle applied to the declaration layer: **the antigen declaration itself
has an evidence-kind**. Two distinct provenance shapes:
- `observed(n)` — "this failure class is grounded in N independently confirmed instances"
- `predicted(rationale)` — "this failure class is structurally guaranteed by the
  following reasoning; no observed instances yet"

The distinction matters for adoption: an antigen with `observed(7)` carries more weight
than one with `predicted(structural_reasoning)`. Teams scanning for "high-confidence
antigens to adopt" want this provenance information.

**ADR-006's recognition-not-design discipline implies this dimension exists but doesn't
explicitly structure it.** The "three independent instances" threshold for stdlib promotion
is an implicit evidence-floor; `evidence_provenance` on the declaration makes it explicit.

This is an F3-type finding: a first-class dimension implicitly assumed uniform (all
antigens are treated as equally well-grounded) that should be explicit.

### Dimension candidate: Severity-class / Risk-tier

The `stdlib-seed-antigens.md` notes different antigen families but doesn't name severity.
From the failure class taxonomy:
- `panicking-in-drop` (process abort on double-panic) — catastrophic
- `nan-comparison-trap` (wrong results) — correctness bug
- `optional-dependency-implicit-feature` (API surface pollution) — maintainability
- `hash-collision-iteration-order` (non-determinism) — depends heavily on context

ADR-008 Amendment 1 names "scan severity defaults" — there IS a severity dimension in
the scan machinery. But is it carried explicitly on the antigen declaration, or inferred
from family?

Walking the ADR index: ADR-008 "Named-observer position as terminal stratum" with
"Amendment 1 — Multi-contributor workflow + scan severity defaults." This needs a closer
read, but from the index, severity is already at least partially in the design.

What's NOT in the design (from what I can see): severity as a consumer-side GATING
dimension independent of witness-tier. A team might say "block on any antigen of
security-class severity; warn on any antigen of maintainability-class severity, regardless
of witness tier." That's a two-dimensional gate: `(severity, witness_tier)`. Currently
the machinery offers only `witness_tier` as the gating axis.

**F3 finding (weak; needs substrate-grep to confirm)**: severity-class may be an
implicit-uniform dimension that CI consumers need to vary. If ADR-008 Am 1 covers this,
it's already explicit. If it covers it only in scan (not in audit output), the gap is
at the consumer-gating surface.

### Dimension candidate: Lifetime of the discipline claim

Some disciplines are permanent: "a function in a `Drop` impl must never panic" is a
forever-invariant. The fingerprint matches forever; the witness requirement is ongoing.

Other disciplines are temporal: "the Cargo.lock must be reviewed monthly by the security
team." The fingerprint is a workspace-level invariant; the witness requirement cycles.

Still others are transitional: "until the migration to async is complete, all sync-Mutex
sites must have this review." Once the migration completes, the antigen itself becomes
obsolete.

The current discipline-witnesses design treats all antigens as permanent. The
`fresh_within_days(N)` leaf handles the temporal aspect of witness currency, but the
ANTIGEN ITSELF has no lifetime. An antigen cannot declare "I am a transitional discipline
that should be retired when condition X is met."

**First-class lifetime dimension**:
- `permanent` — default; the failure class exists forever
- `temporal(review_cadence)` — the discipline is ongoing periodic work
- `transitional(retirement_condition)` — the antigen expects to be retired

This rhymes with something in software engineering literature: **technical debt tracking**.
Debt items have expected retirement conditions. Antigens are the structural version of
debt declarations; lifetime is the structural version of expected-retirement. ADR-004
(implicit-to-explicit elevation) would predict: make the lifetime explicit, not implicit.

### Dimension candidate: Presentation-density expectations

Some antigens are expected to be RARE (presenting more than once is alarming):
`lock-order-inversion` should ideally present zero times; every presentation is a finding
to investigate.

Other antigens are expected to be COMMON (presenting many times is normal):
`signed-zero-discipline` might present at every floating-point function in a numerics
crate.

The current scan machinery treats all antigens uniformly: presents = sites to investigate.
But scan reports would be more actionable if the antigen declaration carried:
- `expected_density: rare | occasional | common | universal`

A rare-density antigen with 47 presentations is alarming. A universal-density antigen with
2 presentations out of 200 eligible sites is alarming in the opposite direction (why are
only 2 sites declared?).

This is more scan-machinery than discipline-witness territory, but it's an F3-type
implicit-uniform dimension.

### Summary for F3

Additional first-class dimensions beyond scope (aristotle's F3):

1. **Evidence-provenance** on the antigen declaration (`observed(n)` vs `predicted(rationale)`)
2. **Severity-class** as a gating axis independent of witness-tier (may already be partially
   in ADR-008 Am 1; need substrate-grep to confirm gap vs overlap)
3. **Lifetime** of the discipline claim (`permanent` vs `temporal` vs `transitional`)
4. **Presentation-density expectations** (`rare` vs `common`) for scan actionability

All four are currently implicit-uniform: every antigen is treated as if it has the same
provenance quality, same severity, permanent lifetime, and no expected density. The
implicit-to-explicit discipline (ADR-004) predicts all four should eventually be explicit.

For v0.1 rc: evidence-provenance and severity-class are the highest-value additions
because they affect the antigen declaration itself (not just the sidecar). The other two
are scan-layer concerns that don't block the discipline-witnesses rc.

---

## S3 — Walking for F8: where else does evidence-kind matter?

Aristotle surfaced EvidenceKind as a structured axis on audit output. The scout question:
where else in antigen does evidence-kind appear implicitly?

### The antigen declaration's own evidence-kind

Already named in S2 as "evidence-provenance." The same axis aristotle identified for
witness output applies to the antigen declaration itself.

Concretely:
- `PolarityInvertedClassMeet` is grounded in a real case study from tambear
  (empirical, observed instance)
- `LockOrderInversion` is a well-known pattern with documented instances (empirical,
  multi-source)
- Some of the `stdlib-seed-antigens.md` candidates are structural predictions ("this
  could go wrong") without confirmed instances

The antigen declaration is itself a claim: "this failure class exists and appears in real
Rust codebases." That claim has an evidence-kind. Currently it's implicit in the `references`
field — references to real bugs is empirical evidence; references to docs or reasoning is
predicted evidence.

Elevating this: `EvidenceKind` on the declaration is `Empirical(n_instances) | Predicted(reasoning_ref)`.
This maps cleanly to the audit-side `EvidenceKind = TypeSystemProof | Behavioral | SubstrateState`:
both are the same question ("how strong is the evidence?") applied at different layers.

### The `#[antigen_tolerance]` escape valve's evidence-kind

When a site has `#[antigen_tolerance(X, rationale = "...")]`, the rationale is evidence
that the site has CHOSEN to present the antigen despite knowing the failure class. That
choice has an evidence-kind:
- `Behavioral` — "we have a test that exercises the path and verifies the specific failure
  mode doesn't occur in our context"
- `SubstrateState` — "the discipline doc says this case is excluded from the pattern"
- `None` — "rationale is a string with no verifiable backing"

The current tolerance mechanism is all `None`-evidence. The sidecar-attestation for
tolerances (from S1) would elevate this to `SubstrateState` evidence.

This surfaces a potential tier-honesty gap: tolerances without attestation are currently
reported as "tolerated" with a rationale string. A tier-honest audit should report WHAT
KIND of evidence backs the tolerance claim, same as it reports what kind of evidence backs
the immunity claim. A bare-string rationale is `EvidenceKind::None` tolerance evidence;
that should be visible in the audit output.

### The `#[descended_from]` inheritance's evidence-kind

When an antigen is declared `#[descended_from(ParentAntigen)]`, the inheritance relationship
is asserted. The evidence for that assertion:
- Was it structurally validated? (the failure-class instances of the child ARE provably
  instances of the parent failure class)
- Was it empirically observed? (real cases where the child manifested showed the parent
  failure class shape)
- Was it designed? (someone decided the inheritance made sense without validation)

Inheritance-without-validation is a classification error waiting to happen. The
taxonomy coherence of antigen's stdlib depends on the evidence-kind of inheritance claims.

### Synthesis for F8

EvidenceKind isn't just an audit-output axis. It's a **design-wide axis** that applies to:
1. Witness outputs (aristotle's primary finding)
2. Antigen declarations themselves (observed vs predicted failure class)
3. Tolerance claims (is the tolerance backed by something verifiable?)
4. Inheritance relationships (was this lineage relationship validated?)

The most actionable for v0.1 rc: #1 (already in aristotle's recommendations) and #3
(tolerance sidecar, already in S1). Items #2 and #4 are longer-arc work.

---

## S4 — Cross-domain tangent: territories the prior passes didn't visit

The prior passes covered:
- Naturalist: immunology, biological metaphor
- Aristotle: first principles, structural necessity
- Academic-researcher: 14 attestation/supply-chain systems (in-toto, SLSA, Sigstore, TUF,
  DSSE, PASETO, cargo-deny, cargo-vet, OPA/Rego, Salsa, CODEOWNERS, Dependabot, npm/GHA,
  GUAC)

Territories NOT covered: economics of trust, history of notary/witness institutions,
anthropology of ritual certification, distributed-systems consensus literature,
software-ergonomics literature on annotation fatigue.

### Notary institutions and the witness problem

The notary public institution (dating to Roman scribes, formalized in medieval Europe)
is one of the oldest human solutions to the "how do we make attestation portable and
trusted?" problem. It's been running for ~800 years. What does that track record teach?

**The notary's key insight**: attestation is trusted not because the notary's word is
inherently credible, but because the notary is a BOUNDED PARTY WITH KNOWN INCENTIVE
ALIGNMENT. The notary has professional liability if they attest falsely; there's a
certification body that can revoke their commission; they're personally identifiable.

Antigen's `git-trust as default` (v2 R2) is the no-notary-friction version: anyone with
git commit rights can sign a sidecar entry. The notary insight says: **trust without
accountability structures is fragile**. The accountability structure (professional
liability, revocable certification) is what makes the notary system work at scale.

What's the antigen equivalent of notary accountability?

- **Near-term analog**: role-in-CODEOWNERS. A `math-researcher` signer who's in CODEOWNERS
  for the relevant path has implicit accountability — the forge knows who they are, they
  can lose their CODEOWNERS status.
- **Stronger analog**: Sigstore OIDC-bound signatures (from academic-research §3). The
  signing event is publicly logged; the signing identity is OIDC-verified; tampering with
  the log is detectable. This is notary-accountability without the professional licensing.

The notary lens predicts: antigen's tier-honesty discipline is structurally right, but
tier-Execution attestation from git-trust-only signers will be contested once antigen
serves high-stakes domains. The accountability structure (OIDC, transparency log, role
verification) is what makes Execution-tier attestation defensible externally.

**For v0.1**: the current design is right. The notary insight is a long-arc design
prediction, not a v0.1 blocker. But it's worth naming in ADR-019 as the design arc:
"git-trust is the floor; the accountability escalation path is OIDC + transparency
log."

### Economics of trust: signaling theory

From signaling theory (Spence, 1973 — job market signaling; extended to many domains):
a credible signal must be COSTLY TO FAKE. An easy-to-fake signal provides no information.

Antigen's `signers(required = ["alice"])` where alice is git-trust: how costly is it to
fake? Alice can sign a sidecar entry by running `cargo antigen attest sign` with her git
config set appropriately. If alice is a bad actor with commit rights, she can fake the
entire attestation trail. The signal is cheap if the signer has commit rights anyway.

The signaling lens predicts: discipline-witness attestation is only a credible signal IF
the cost of faking it exceeds the cost of actually doing the discipline work. For
compliance/regulatory contexts (the main use case for "math-researcher must review"), the
cost of faking is reputational-and-legal; git-trust captures enough identity to establish
accountability.

But there's a subtler signaling failure: **rubber-stamp attestation**. Alice signs every
sidecar entry as `math-researcher` without actually doing the review. No tier-honesty
violation per se — the predicate passes. But the signal is meaningless.

The sidecar schema currently has no way to distinguish "alice read the function carefully
against the discipline doc" from "alice ran `cargo antigen attest sign` without looking."

This is the signaling-theory equivalent of credential inflation. What prevents it?

**What the prior passes missed**: cargo-vet's approach to this is worth surfacing. cargo-vet
uses `delta audits` as the anti-rubber-stamp primitive. A delta audit explicitly says "I
reviewed the diff from version X to Y" — which requires actually LOOKING at what changed.
Aristotle noted `--carry-forward-from X` as a parallel. But the anti-rubber-stamp
discipline in cargo-vet is also behavioral: the community can observe "this organization
audited 3000 crates in one day" and discount those audits as rubber-stamps.

For antigen: the `audit_frequency` signal. If the same signer has signed 200 sidecars
in a day, that's suspicious. `cargo antigen attest list --signed-by alice --since 24h`
would surface the signal. This is a social enforcement mechanism, not a technical one.
Worth naming in ADR-019 as the intended discipline rather than assuming it's encoded.

### Anthropology of ritual: why certification ceremonies work

Anthropologically, certification ceremonies (PhD defenses, bar exams, medical licensing
boards, religious ordination) work not because they perfectly verify competence but
because they COORDINATE EXPECTATIONS and create COMMUNITY ACCOUNTABILITY. The PhD committee
knows that if they wave through poor work, their department's credibility suffers. The
ceremony creates shared stakes.

The antigen sidecar schema has the mechanics of a certification ceremony (who signed,
when, against what) but lacks the social coordination layer. The social layer is:
**who can see this attestation, who can challenge it, what happens when it's challenged?**

For v0.1, the answer is: git history shows who signed, PR reviewers can challenge, and
the audit reports stale-on-change. That's a minimal ceremony. For high-stakes use cases
(regulatory compliance, safety-critical software), the ceremony needs to be richer: a
review board, challenge period, public registry of attestations.

This isn't a v0.1 design change — it's the prediction about where antigen goes as it
matures. The design should accommodate escalation of ceremony without requiring a rewrite.
The `Signer.signature: Option<Signature>` slot is the right shape: no signature = git
ceremony; with signature = cryptographic ceremony. The ceremony escalation path is already
designed in; this anthropological lens just confirms it's the right shape.

### Distributed-systems consensus: what antigen can learn from Raft/Paxos

In distributed systems, CONSENSUS is the problem of getting a set of nodes to agree on
a value despite failures. The interesting parallel to antigen's multi-signer attestation:

`signers(required = ["alice", "bob", "carol"])` is NOT consensus — it's quorum
ratification. All three must sign; if any one refuses, the predicate fails. This is the
strict interpretation.

Raft's approach: leader election, majority agreement, committed entries. The majority
might not include ALL nodes but it's ENOUGH to make the log persistent (won't be lost).

For antigen: TUF's threshold signatures (from academic-research §4) are the
distributed-systems-inspired version: `k-of-n` rather than `all-of-n`. Aristotle noted
this; the academic-research pass noted it. What the prior passes didn't name:

**The partition-tolerance tradeoff in attestation**. In Raft: you can have consistency or
availability during a partition; you can't have both (CAP theorem). In antigen attestation:
- **Consistency**: predicate fails if ANY required signer is unavailable (full `all_of`
  quorum)
- **Availability**: predicate passes with k-of-n (threshold), surviving even if some
  signers are unreachable (gone from the team, etc.)

Antigen's v0.1 design (`signers(required = [...])` with n-of-n implied) chooses
consistency over availability. The design prediction: once antigen is used by larger teams
with personnel turnover, n-of-n will break adoption when a required signer leaves the
org. The k-of-n amendment (TUF-style) is the CAP-theorem-motivated availability addition.

Aristotle mentioned this briefly (TUF threshold). The distributed-systems lens gives
it firmer grounding: **k-of-n threshold signing is the CAP theorem's availability clause
applied to attestation.**

### Software-ergonomics literature: annotation fatigue

From software engineering research (not in any of the prior passes): annotation-heavy
systems suffer annotation fatigue. JML (Java Modeling Language), Dafny specifications,
ClangSA annotations — all show the same pattern: initial adoption enthusiasm, followed by
annotation attrition as the annotation burden mounts faster than its perceived value.

The key finding from the literature: annotation systems survive adoption when they have
TOOLING THAT REDUCES THE ANNOTATION BURDEN faster than the annotation burden grows. Clippy
did this by automating most lints so the human only intervenes when clippy can't catch it.
JML did not do this (mostly manual annotations) and largely failed outside academia.

What this predicts for antigen:

1. **`cargo antigen attest scaffold` is not enough.** Scaffolding creates the template;
   humans fill it in. For annotation fatigue to be prevented, there should be tooling
   that proposes WHO should sign (based on CODEOWNERS or git blame), proposes WHICH
   oracles need completion (based on the fingerprint match), and REMINDS signers when
   their previous signatures are going stale.

2. **IDE integration matters more than CLI tooling for adoption.** The annotation-fatigue
   literature consistently shows that IDE-integrated tooling (inline warnings, quick-fix
   suggestions) has dramatically higher adoption than CLI-gated tooling. The CLI is the
   right v0.1 surface. IDE integration (rust-analyzer plugin, VS Code extension) is the
   v0.2 adoption multiplier.

3. **The "nag keeps coming up in scan/audit" property Tekgy named** (from the design
   conversation, Turn 4) is an annotation-fatigue double-edged sword. It creates urgency
   to resolve outstanding attestations. But if the nag-frequency exceeds the team's
   capacity to respond, it becomes background noise — the same failure mode as CI gates
   that are always red. **Antigen's design should account for this**: a team that presents
   20 discipline antigens with 0 signed should still get actionable signal, not a wall
   of red that they learn to ignore.

4. **Adoption gradient (ADR-009)** is the right structural response to annotation fatigue.
   The antigen-the-vocabulary entry point (just name a failure class) has zero annotation
   cost. Each adoption level adds more annotation cost; teams level up when the value
   exceeds the cost. This is exactly what the ergonomics literature says is needed.

The annotation-fatigue lens suggests: **`cargo antigen attest list --pending` needs to be
opinionated about PRIORITY, not just COMPLETENESS.** Rather than listing all pending
attestations equally, it should surface the highest-impact ones first. "Here are the 3
presentations where the fingerprint matches highly-confident antigens with existing
witnesses; fixing these 3 covers your highest-risk sites." The prioritization is the
ergonomics work.

---

## S5 — A structural rhyme I wasn't looking for but noticed

Walking the `multi-component-immunity.md` (Part I): antigen is described as an "emergent
practice" where components (dev-judgment, passive scan, test integration, knowledge
ecosystem, version/lineage, cross-crate) participate in a shared vocabulary.

Here's the rhyme: **discipline-witnesses are Component 1.5** in this taxonomy.

Component 1 is "dev-judgment" — the human making a decision about a code site. Component
2 is "passive scan/tools" — automated detection. Discipline-witnesses are neither: they're
human judgment STRUCTURED as substrate that the passive scan can verify. They're the
mechanical interface between C1 and C2.

This is more than a rhyme — it's a missing slot in the multi-component taxonomy. The
taxonomy names 7 components; discipline-witnesses bridge C1 and C2 in a way none of the
7 components explicitly describes. They're the **attestation-mediated-judgment component**.

Why does this matter? If the multi-component-immunity framing shapes how antigen's
components are designed and composed, discipline-witnesses need a slot in that framing.
The current framing may be treating them as an extension of C1 (dev-judgment expressed
in a new format) or C2 (passive verification of new substrate types). But they're
structurally intermediate: disciplined-human-judgment-made-machine-verifiable. The
middle position is load-bearing.

**For v3 or the multi-component-immunity doc**: add the attestation-mediated-judgment
component explicitly. It may be the bridge component that explains why antigen's
architecture is coherent across all 7 components — the shared vocabulary works precisely
because discipline-witnesses create a machine-verifiable bridge between human judgment and
automated verification.

---

## S6 — The F2 absorption pattern and the scan/audit pipeline asymmetry

Something I noticed while walking the audit.rs source that nobody named:

The **scan** and the **audit** have a structural asymmetry in how they treat substrate:

- **Scan** walks `.rs` files looking for antigen presentations, fingerprint matches. Its
  substrate is Rust source.
- **Audit** validates immunity claims — witness identifier resolution. Its substrate is
  also Rust source (looking up function definitions).

Both are `.rs`-centric. Discipline-witnesses require the audit to read `.attest/` JSON
sidecars. That's a qualitative change in what the audit reads — it moves from
"reads Rust source" to "reads typed JSON adjacent to Rust source."

The observation: **there's no corresponding scan extension for non-`.rs` substrate.** The
scan doesn't walk `Cargo.toml`, doesn't walk `.md` docs, doesn't walk `.proto` or IDL
files. But `stdlib-seed-antigens.md` Antigen 10 (`optional-dependency-implicit-feature`)
is EXPLICITLY a `Cargo.toml` pattern. The fingerprint for it is `cargo_toml_pattern: '...'`.

This means scan already conceptually reaches outside `.rs` files — but the implementation
may not. The F2 absorption pattern (typed JSON sidecar adjacent to any substrate) requires
the audit to read non-`.rs` substrate. The parallel question for scan: what substrates
does scan need to walk that it doesn't currently?

Candidates from stdlib-seed-antigens.md:
- `Cargo.toml` — Antigen 10 (optional-dependency-implicit-feature)
- `build.rs` + `.proto` — generated code discipline
- `Cargo.lock` — workspace-level antigens (Cargo.lock review discipline)

The F2 pattern generalizes the audit's substrate-reading; the SCAN has a parallel
generalization question. Neither is explicitly addressed in v2 (which focuses on the
attestation side of discipline-witnesses, not the scan side of non-`.rs` substrate).

**Finding**: the discipline-witnesses design implicitly extends the audit's substrate
reading. A parallel design question is whether the scan's substrate walking also needs
extension. For `optional-dependency-implicit-feature` to ship in antigen-stdlib v0.1 with
any mechanical detection (not just declaration), the scan needs to walk `Cargo.toml`.
This is a gap between stdlib-seed-antigens.md's ambition and the current scan engine's
scope.

---

## S7 — The geography of a v3 and what would make antigen MORE USEFUL

Aristotle's recommendations for v3 (in priority order):
1. F8 — EvidenceKind first-class axis
2. F3 — `scope:` field on discipline-antigen declarations
3. F1 — Name the discipline-level vs machinery-level unification asymmetry
4. F2 — Absorb doc-level ratification into existing primitive
5. F4 — Refine predicate-language ceiling
6. F5 — `Signer.basis` field + multi-pin `against` parameters
7. F7 — Defer witness-provider-crate trust boundary with explicit scope
8. F6 — Name audit-of-audit bounded-regress structure

From the scout pass, additional items that would make antigen MORE USEFUL (prioritized by
adoption impact):

**S-A (high-value, v0.1 rc candidate)**: Tolerance sidecar schema. `#[antigen_tolerance]`
without attestation is a tier-honesty gap. Isomorphic schema to immunity sidecars; ships
in v0.1 as a discipline extension, not a new primitive. If the schema ships, tolerance
claims join the audit's verified substrate.

**S-B (high-value, v0.1 rc candidate)**: `evidence_provenance` field on antigen
declarations. Simple addition: `observed_instances: Option<u32>` and `predicted_by:
Option<String>` (rationale reference). Not a gate — purely informational. But it makes
the difference between `LockOrderInversion` (5+ known instances) and a hypothetical
antigen with no observed instances visible in the audit output and scan output.

**S-C (medium-value, v0.2 candidate)**: `attest sign --carry-forward-from X` with diff
inspection (aristotle's F5). The anti-annotation-fatigue version of re-attestation. Teams
that run rustfmt should not need to re-sign every sidecar when the formatting changes.

**S-D (medium-value, v0.2 candidate)**: k-of-n threshold signing (`signers(required_count
= K, candidates = [...])`). The CAP-theorem-motivated availability addition. Required for
teams with > 10 people in CODEOWNERS rotation.

**S-E (adoption multiplier, v0.2 candidate)**: `cargo antigen attest list --pending
--prioritized`. Outputs pending attestations ordered by: (1) antigen evidence-provenance
strength, (2) fingerprint confidence, (3) presentation count. The ergonomics research
finding: prioritized actionable lists prevent annotation fatigue. Completeness lists
become wallpaper.

**S-F (long-arc design, v0.3+ candidate)**: attestation-mediated-judgment as explicit
Component 8 in the multi-component-immunity framing. Not a v0.1 or v0.2 implementation
concern; a framing clarification that pays off when the docs and marketing tell the story.

---

## What I'd recommend folding into v3

In roughly priority order, based on the full scout pass:

**Fold from F2 generalization (S1)**:
1. Tolerance ratification schema — same shape as immunity sidecar; plugs the tolerance
   tier-honesty gap. Concrete addition: `#[antigen_tolerance(X, sidecar = true)]` opt-in
   plus schema extension. The value is tier-honest tolerance claims vs vibes-grade rationale
   strings.
2. Reserve fingerprint ratification as a long-arc extension. Not v3 but design-preserve:
   the sidecar-adjacent-to-substrate pattern should explicitly not rule out applying to
   fingerprint declarations.

**Fold from F3 additions (S2)**:
1. `evidence_provenance` field on `#[antigen]` declaration. Minimal schema change.
   `observed_instances: Option<u32>` captures the recognition-not-design discipline check
   (ADR-006 "three instances threshold") as structured data rather than implicit practice.
2. Flag severity-class as already-in-or-not: do an ADR-008 Am 1 substrate-grep to confirm
   whether severity is already first-class in scan output. If it is in scan but not in
   audit output, closing that gap is the action.
3. Lifetime of discipline claim: flag as v0.2+ — `permanent | temporal(cadence) | transitional(condition)`.
   Not v3 but should be on the roadmap.

**Fold from F8 extension (S3)**:
1. `EvidenceKind::None` for tolerance claims in audit output — low-cost addition that
   makes tolerance-without-attestation visible. Pairs with S1's tolerance sidecar work.
2. EvidenceKind on antigen declarations: flag as companion to `evidence_provenance`. They're
   the same question at different layers.

**Fold from cross-domain tangent (S4)**:
1. Name the accountability escalation path in ADR-019: git-trust → OIDC + transparency
   log (Sigstore). The notary-institution lens confirms this arc; naming it explicitly
   sets the design direction for compliance-grade use cases.
2. `cargo antigen attest list --pending --prioritized`: annotation-fatigue-aware output
   ordering. Flag for v0.2 CLI extension.
3. k-of-n threshold signing: flag for v0.2 amendment. The CAP-theorem framing gives it
   principled grounding beyond just "it's useful."

**Fold from structural rhyme (S5)**:
1. Name discipline-witnesses as "attestation-mediated-judgment" component in
   multi-component-immunity doc. Low-cost framing clarification; high-value for the
   project's self-understanding.

**Fold from scan/audit asymmetry (S6)**:
1. Explicitly call out in ADR-019 that discipline-witnesses extend the audit's substrate
   reading, and that a parallel question exists for the scan (Cargo.toml, etc.).
   Don't design the scan extension here; name the structural parallel so it doesn't
   get lost.

---

## What doesn't need to change

The three-piece shape (predicate language + Ratification schema + CLI) survives the
scout pass. Code-locality survives. Closed combinator grammar survives. Tier-honesty
discipline survives and is reinforced by the cross-domain perspectives.

The notary lens, signaling theory, ergonomics literature, and distributed-systems
consensus framing all CONFIRM the design directions already chosen, while adding
vocabulary for the escalation paths and failure modes to watch.

Nothing from the scout pass invalidates v2's core design. The additions are
sharpenings, gap-fillings, and long-arc predictions — not structural objections.

---

READY FOR REVIEW

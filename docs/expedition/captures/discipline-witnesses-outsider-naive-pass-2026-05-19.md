# Discipline-Witnesses — Outsider Naive Pass (2026-05-19)

> **Author**: outsider (first outsider pass on the discipline-witnesses
> thread). **Audience**: pathmaker drafting ADR-019; team for Phase-2
> readiness; future outsiders inheriting this trail.
>
> **Method**: read v3 + launch-brief + code surface as a fresh Rust
> developer landing on antigen v0.1-rc with zero prior context. Stripped
> all prior-capture context. Asked "why" of every framing, every
> term-of-art, every "everyone knows." Where v3 says "absorbed per
> X-finding" treated that as inherited assumption to question fresh, not
> as resolved. Walked code with `grep` to verify substrate-vs-promise
> alignment.
>
> **Posture**: outsider doesn't write to v3 directly. Findings are
> routed to navigator + persisted here for pathmaker, executor, and
> whoever does the Phase-2 ADR-019 drafting. Team decides which findings
> are dust to remove vs inherited assumptions to crystallize explicitly.
>
> **Disposition signal** at end of each finding:
> - **DUST?** — possible removal/simplification
> - **JURISDICTION** — citation graph or authority issue
> - **SUBSTRATE-GAP** — v3 promises what code doesn't deliver
> - **VOCAB** — naming/term-overload
> - **USER-FRICTION** — concrete adoption pain
> - **META** — about the team or process itself
>
> See [INDEX.md](../INDEX.md) for the substrate trail.

---

## Top-line: the headline finding

**OUT-17 is the load-bearing one. v3's macro examples will not compile
against the current parser.** Everything else is texture; OUT-17 is the
gap that turns v3 from "design substrate" into "design substrate
mis-representing itself as implementation-ready." Routing produced
tasks #22/#23/#24/#25 after this finding landed.

If pathmaker reads only one section of this capture, read OUT-17.

---

## Findings (OUT-1 through OUT-19)

Grouped: vocabulary/framing dust (OUT-1...OUT-11), substrate-vs-promise
gaps (OUT-12...OUT-17), jurisdictional citation (OUT-18), team
composition meta (OUT-19).

---

### OUT-1 — "discipline" as a term is doing fuzzy work [VOCAB]

v3 line 122: "discipline antigens stop being a special category and
become *ordinary antigens*." Then uses "discipline-witness" /
"discipline failure-classes" / "discipline-substrate" / "discipline_doc"
pervasively throughout. The word "discipline" is doing at least three
jobs:

1. A *kind of failure-class* (one verified by non-code substrate)
2. A *property of attestation* (chain-depth caps + rationales = "the
   discipline")
3. A *human practice* (the team's review process)

A fresh user can't tell which sense applies in a given sentence.

**Sharpest question**: if discipline-antigens really aren't a special
category, why does the audit emit hints prefixed `discipline-*` (e.g.,
`discipline-predicate-passed-substrate-current`) rather than something
uniform like `substrate-witness-passed`? The hint prefix is the
asymmetry made permanent. Either the category IS special (and v3 should
say so), or the hint prefix is residue from earlier framing.

---

### OUT-2 — "substrate-witness" vocabulary chain a fresh user can't parse [VOCAB]

"Witness" is already a domain-specific word (the W7 enum). "Substrate"
is project-specific jargon. "Substrate-witness" requires understanding
both, then composing them. The doc never defines "substrate" inline.

A naive Rust developer landing on `#[immune(X, requires = all_of([...]))]`
will think "this is a *requires* clause" — which is what the macro
literally says. But the audit-side hints call it "substrate-witness."
The doc calls it "predicate language." **Three names for one thing.**

**Question**: why isn't the predicate-side vocabulary just
"requirements" or "preconditions"? Those are universal Rust/SE
vocabulary. The biology-side vocabulary can stay biology; the
developer-facing vocabulary could be plain.

---

### OUT-3 — `.attest/` directory choice is unexplained in v3 [USER-FRICTION + DUST?]

v3 mentions `.attest/` in passing (line 222, 353) but never says *why*
not `.antigen/` or `.witness/` or just inline in the source file. The
launch-brief flags this as a "v1 capture turn 9-10" decision; a fresh
user reading v3 alone has no context.

**Concrete user friction**: when I tell my team "add `.attest/` to
gitignore patterns and CI artifact ignore lists," they will ask why. If
I say "because the antigen tool puts attestation sidecars there," they
will ask "why isn't it under `.antigen/` like everything else?" I don't
have an answer from v3.

**Action**: either v3 should carry the one-line justification, or the
choice is dust.

---

### OUT-4 — Three-axis output (`WitnessTier × AuditHint × EvidenceKind`) is ceremonial in v0.1 [DUST?]

A fresh user's first question: "When I run `cargo antigen audit`, what
do I look at first?" v3 doesn't say. There's no recommended scanning
order, no "the headline number is X, the details are Y." The audit
hint table has 7 row-categories for immunity + 4 for tolerance = 11
distinct hint values just in v0.1.

**Key observation**: looking at the v3 tier-mapping tables,
`EvidenceKind` is *always* `SubstrateState` for substrate-witnesses
(except tolerance-vibes-grade = `None`). So in v0.1 the `EvidenceKind`
axis has 2 effective values for this primitive. The "first-class third
axis" framing may be over-credited for the v0.1 surface — it carries
one bit of info (substrate vs not). The team's projected future use of
the axis (TypeSystemProof / Behavioral) may justify it, but as shipped
in v0.1 the axis looks ceremonial.

---

### OUT-5 — Biology metaphor is doing two different jobs and might not earn both [VOCAB]

Job A: design heuristic (naturalist asks "what does biology predict
here?" → produces real architectural insight, e.g., memory-cell
persistence → sidecar persistence). Job B: user-facing communication
(the README, the ADR, the rustdoc strings).

Job A is clearly load-bearing for the team. Job B is questionable: a
Rust developer who has never opened an immunology textbook reads "MHC
presentation → typed sidecar; T-cell+B-cell co-stimulation → all_of"
and processes it as decorative jargon.

**Question**: are the biology rhymes in v3 §"Biology grounding" written
for the team (Job A) or for adopters (Job B)? If A, the section belongs
in a design substrate, not in the ADR. If B, the rhymes need to be
earnable by readers who weren't on the journey.

---

### OUT-6 — The 4-point bright-line rule is under-defended at point 1 [DUST?]

v3 line 169-180: "binary named in leaf source — no runtime tool-name
resolution." Why? The reason isn't given inline. A fresh user can
reason their way to the threat (supply-chain attack: malicious binary
on PATH), but the bright-line *as stated* doesn't reference that
threat. Compare point 3 ("does not execute user-supplied code") which
is self-justifying. Points 1, 2, 4 require security-context to
motivate.

**Question**: where's the threat model the bright-line is responding to?
Without it, point 1 looks like aesthetic preference and a future
maintainer might soften it.

---

### OUT-7 — `evidence_provenance = observed(7)` is self-reported vibes-grade [DUST?]

v3 line 308: `evidence_provenance = observed(7)` "encodes ADR-006's
three-instances threshold as structured data; audit can verify N ≥ 3
for stdlib promotion eligibility."

**Fresh-user question**: who counts? If I'm declaring a new antigen, do
I scan the world for instances and put the number in the field? Is `7`
self-attested? Can I write `observed(999)` to inflate?

The field as designed is *self-reported provenance*, which is exactly
the kind of vibes-grade attestation the rest of v3 carefully avoids.
ADR-006 is about a real human-substrate practice (the team has seen N
instances before adding the antigen); encoding the count as a number in
the source file moves the practice into the type system but loses the
verifiability of the practice.

**Action**: should `evidence_provenance` instead be a substrate-witness
predicate itself, where the audit verifies the references? Or is
"self-attested provenance metadata" honest enough as long as the audit
reports it as `EvidenceKind::SelfReport` or similar?

---

### OUT-8 — Tolerance-as-isomorphic-schema feels too tidy [META]

v3 leans hard on "tolerance and immunity share the same Ratification
schema, just `kind = Tolerance` vs `Immunity`." Elegant but suspicious.

**Naive question**: if tolerance is "I know this code violates the
discipline and I'm accepting that," and immunity is "I claim this code
complies with the discipline," then the *signers attesting fundamentally
different things*. Same schema, different semantics. The
`signers[].role` field is the same and the `signed_against_fingerprint`
is the same — but the meaning of "I signed" depends on
`RatificationKind`.

This is a discriminator-controlled semantics, which is exactly what
ATK-v3-TOL attacked. **Question**: did ATK-v3-TOL examine whether the
schema-isomorphism creates a *human* footgun even if the machine-
discriminator is well-defended? Putting them in the same CLI surface
(`attest sign` vs `tolerate sign`) may not be enough psychological
separation.

---

### OUT-9 — ADR-019 cites 8 other ADRs; standalone-comprehensibility cost [JURISDICTION]

v3 lines 678-708: ADR-019 cites ADR-002, ADR-004, ADR-005 (sub-clause F
AND Amendment 3), ADR-006, ADR-007, ADR-008, ADR-011. That's 8
citations.

**Naive question**: if I'm an ecosystem implementer reading ADR-019 to
build a competing tool or a compatible reader, do I need to absorb all
8 cited ADRs first? If yes, ADR-019 has high prerequisite mass. If no,
the citations could be moved to footnotes/appendices.

The team's instinct ("everything is structurally connected") may be
right, but the cost is that ADR-019 isn't standalone-comprehensible —
and standalone-comprehensibility is what a fresh ecosystem participant
needs. **Sharpened version: see OUT-18.**

---

### OUT-10 — "New-user first sidecar" walkthrough is missing from v3 [USER-FRICTION]

v3 documents macro, schema, CLI subcommands, hint table, citation map.
It does not document: *what does a developer do, from `cargo init` to
first green audit*? The doc is structured as architecture-spec, not as
user-journey.

A fresh user will not know whether they: (a) write the macro first and
the CLI scaffolds the sidecar; (b) run a CLI scaffold and then write
the macro; (c) some other ordering.

**Risk**: v0.1-rc is supposed to ship to actual humans. Implicit-mode
docs ship with the implicit-mode reflexes the developer brings, not the
reflexes antigen wants.

---

### OUT-11 — `signers(against = "any")` is an undocumented security hole [DUST?]

v3 line 152: `signers(required, roles?, against?)` with default
`against = "current"`.

**Question**: under what circumstance would someone set
`against = "any"`? The doc doesn't motivate this case. Setting `"any"`
accepts a signature against a stale fingerprint, which appears to
defeat the whole ratchet. The legitimate use case "I want to accept the
signature even though the code has changed in a non-substantive way" is
exactly what `attest delta` is for — and `attest delta` has the
anti-laundering safeguards that `against = "any"` would bypass.

**Action**: either remove `against = "any"` from v0.1, or document the
specific use case it serves + how it interacts with delta safeguards.

---

### OUT-12 — `antigen-attestation` crate exists but is orphaned [SUBSTRATE-GAP]

`antigen-attestation/` is in `Cargo.toml` workspace members (line 5) and
has `schema.rs` (602 lines), `predicate.rs`, `evaluate.rs`, `tier.rs`,
`lib.rs` all real. **But no other crate depends on it.** Neither
`antigen` (the lib) nor `cargo-antigen` (the CLI) imports it.

v3's "load-bearing schema + evaluator" is built as a workspace island.
The launch-brief says it's to be "new" in Phase 3 ("New crate:
`antigen-attestation` (separate per v2 R4)"), which suggests the brief
is stale — the crate is already there.

**Action**: known in-flight state? Or someone built crate and forgot to
wire it? Either way, INDEX/launch-brief should be updated.

---

### OUT-13 — Two `WitnessTier` enums exist; two `AuditHint` enums coming [SUBSTRATE-GAP]

`antigen/src/audit.rs:123` defines `WitnessTier`;
`antigen-attestation/src/tier.rs:36` defines another `WitnessTier`.
Same name, two crates, possibly different shapes. Same story brewing
for `AuditHint`.

This is exactly the "machinery-level unification asymmetry" guardrail
concern from aristotle F1 + adversarial T5-R — except the discipline-
side concern was about *shared parsers*, and here we have *shared type
names*.

**Action**: when `antigen` finally depends on `antigen-attestation`,
which crate's `WitnessTier` wins? T5-R's in-code comment guardrail and
adversarial precision test should probably cover *type-name-collision*
in addition to *parser-code-sharing*.

---

### OUT-14 — `EvidenceKind` exists in attestation but NOT in audit.rs [SUBSTRATE-GAP]

v3 claims (line 415) "EvidenceKind is the third axis on audit output."
But `antigen/src/audit.rs`'s `ImmunityAudit` struct (line 194) doesn't
have an `evidence_kind` field. So in current code, audit output is
**still two-axis** (`WitnessTier × AuditHint`). The third axis is built
in the attestation crate but not threaded through the audit-call-graph.

**Concrete user impact**: if I run `cargo antigen audit` today, I get
two-axis output. v3 says I'll get three. The gap between substrate-
state and v3-promise is the kind of thing that confuses adoption.

---

### OUT-15 — `cargo antigen attest` / `cargo antigen tolerate` CLI not built [SUBSTRATE-GAP]

`grep "attest\|tolerate"` in cargo-antigen/src/ lit up only the
`antigen-attestation` crate name. v3 documents the full CLI surface
(`attest scaffold/sign/delta/oracle/check/list/move/migrate/gc` +
`tolerate scaffold/sign/check/list`).

If the CLI isn't built, v3 is documenting a future, not a present.
That's fine for design substrate — but launch-brief Phase 3 is
`in_progress` and v3 reads as if it's describing what ships.

**Question**: is there a clear distinction in v3 between "shipping in
v0.1-rc" and "shipping in v0.2+" that I'm missing, or has the "what's
in v0.1" boundary blurred?

---

### OUT-16 — TaskList phase-ordering broken [META]

TaskList #1 (Phase 1 frontier-attack) was `in_progress` simultaneously
with #3 (Phase 3 implementation). Per launch brief, Phase 3 follows
Phase 2 which follows Phase 1. So implementation was happening **before
ADR-019 drafts**, while frontier-attack on v3 was still in flight.

A fresh outsider would ask: "How can we be implementing the schema if
the ADR ratifying it isn't drafted yet?"

If the answer is "the schema is exploratory until the ADR ratifies,"
then `antigen-attestation` crate should be marked unstable /
experimental in Cargo.toml or lib.rs preamble. Nothing currently marks
it as pre-ratification.

*Note: navigator confirmed Phase 1 closed as of 2026-05-19; the
ordering question stands but the specific concurrency observed at
report-time has resolved.*

---

### OUT-17 — **HEADLINE: v3 macro examples will NOT compile against current parser** [SUBSTRATE-GAP]

Hard evidence from `antigen-macros/src/parse.rs`:

**`#[antigen]`** currently accepts only (lib.rs:116):
`name, fingerprint, family, summary, references`

v3 line 304-315 promises (in a code block formatted as if it works):
```rust
#[antigen(
    SignedZeroDiscipline,
    scope = "site",
    discipline_doc = "docs/sinh.md",
    evidence_provenance = observed(7),
    canonical_reference = "...",
)]
```
**`scope`, `discipline_doc`, `evidence_provenance`, `canonical_reference`
— none of these are accepted by the current parser.** A user copy-
pasting this gets:
`error: unknown #[antigen] field 'scope'; expected one of: name, fingerprint, family, summary, references`

**`#[immune]`** currently accepts (parse.rs:212-228): `witness, rationale`

v3 line 184:
```rust
#[immune(SignedZeroDiscipline, requires = all_of([...]))]
```
**`requires` not accepted.** And `witness` IS required by `validate()`
(parse.rs:244). So v3's example fails on TWO counts: unrecognized
`requires` AND missing required `witness`.

**`#[antigen_tolerance]`** currently accepts (parse.rs:316):
`rationale, until, see`

v3 line 196:
```rust
#[antigen_tolerance(SignedZeroDiscipline, sidecar = true, requires = all_of([...]))]
```
**`sidecar` and `requires` not accepted.** And `rationale` (required) is
missing.

**Severity**: this turns v3 from "design substrate" into "design
substrate that mis-represents itself as implementation-ready." A fresh
user reading the launch-brief ("substrate is at full-team development
launch ready"), opening v3, and trying the macros will hit immediate
compile errors and lose trust in the whole pipeline.

**Hypothesis**: this drift accumulated because v3 evolved through 7
captures (design substrate) while macros evolved through the Sweep A2
implementation track (separate substrate). The two tracks both moved
forward, but their cross-validation point (macros accept what v3
designs) wasn't ratified.

**Action**: either extend macros (task #22 in flight) OR mark v3
examples as proposed-syntax-not-yet-shipped. Until one of these lands,
pathmaker drafting ADR-019 with these examples will publish a syntax
that doesn't compile.

---

### OUT-18 — ADR-005 Am 3 doesn't authorize the EvidenceKind axis [JURISDICTION]

Read ADR-005 Am 3 directly (decisions.md:784-998). What's ratified is
the **two-axis** tier-honesty model: `WitnessTier` (strict 4-tier enum:
None/Reachability/Execution/FormalProof) + `audit_hint` (free-form
disambiguation). Decision (line 866-871) and Mechanics §2 (line
913-936) name these two surfaces explicitly.

v3 cites ADR-005 Am 3 as authority for a **three-axis** model:
> v3 line 684: "ADR-005 Amendment 3 (audit-tier-honesty): ... ratchet-
> asymmetry named; **EvidenceKind adds third orthogonal field**;
> bounded-recursion of audit-of-audit named explicitly"

The word "adds" is doing a lot of work. ADR-005 Am 3 was ratified
2026-05-09 with TWO axes. v3's third axis (`EvidenceKind`) is a NEW
design move per aristotle F8.

**Jurisdictional gap**: ADR-005 Am 3 doesn't AUTHORIZE the third axis,
it just doesn't FORBID it. ADR-019 needs to do the authorizing work
itself, not borrow it from a precedent that doesn't say what v3 implies.

**Action for pathmaker**: cite ADR-005 Am 3 as the *principle* (tier-
honesty exists; audit reports lower bound) but do the EvidenceKind
authorization *fresh in ADR-019*, with its own three-instances
threshold under ADR-006. The aristotle F8 finding is the right
substrate for that authorization; bring it forward into ADR-019's body
rather than implying ADR-005 Am 3 already did the work.

Alternative: draft ADR-005 Amendment 4 to add the third axis, then have
ADR-019 cite that.

**Why a naive reader catches this**: insider-vision sees "Am 3 said
tier-honesty, EvidenceKind serves tier-honesty, therefore Am 3
authorizes EvidenceKind." Outsider sees "Am 3 explicitly ratified two
specific axes; EvidenceKind is a third axis; jurisdictional gap." The
conflation happens because everyone on the team already believes the
third axis is right, and the citation reads as agreement rather than as
load-bearing logical step.

---

### OUT-19 — Team composition asymmetry [META]

Counted captures in `docs/expedition/captures/` at time of pass: 12
captures across 5 roles:
- naturalist: 4
- aristotle: 3
- adversarial: 3
- scout: 1
- academic-research: 1
- **outsider: 0 (before this capture)**
- **pathmaker: 0**
- **observer: 0**
- **executor: 0**

The substrate that v3 absorbed was generated by **5 critique-from-
inside roles** doing **12 passes**. The build/user-empathy/coordination
roles produced **0 captures** before v3 was declared "ready for full-
team launch."

**Naive question**: a substrate built by 12 passes of internal critique
without a single user-empathy or build-validation pass — is that
substrate *deeply self-coherent* or *self-reinforcing without ground-
truth contact*? Both are possible from the same evidence. The critique
roles validated each other's findings; nobody asked "would a real user
understand this?" or "does this build?"

OUT-12 through OUT-17 are exactly the kind of thing a build-validation
pass would have caught at any point in the 12-pass cycle. They didn't
get caught because no pass had that as its job.

**Structural implication**: if launch-readiness is judged by capture-
density, critique roles will *always* dominate the readiness signal. A
team where naturalist + aristotle + adversarial keep finding refinements
will *always* look "substrate is rich, almost ready." Roles whose job
is "does it work for the user / does it build / does it ship" produce
*integration*, which shows up as code working, not as documents.

**Question for navigator**: deliberate sequencing ("design first,
integrate second") or emergent bias ("critique reports more visibly")?
If deliberate, the structural risk above is named and accepted. If
emergent, there's a feedback loop worth interrupting: integration-roles
silence reads as "nothing to add" when it may be "we haven't been
invited yet."

---

## Routing summary (recommended)

| Finding | Primary owner | Disposition |
|---|---|---|
| OUT-17 | pathmaker + executor | **Headline. Block ADR-019 final or annotate v3 examples.** Tasks #22-25 in flight. |
| OUT-18 | pathmaker | Citation rework in ADR-019 draft. |
| OUT-12, OUT-13, OUT-14, OUT-15 | executor | Wire `antigen-attestation` through `antigen` + `cargo-antigen`. |
| OUT-1, OUT-2, OUT-5 | naturalist + pathmaker | Vocabulary audit before ADR-019 final prose. |
| OUT-3, OUT-10 | pathmaker | User-facing prose / getting-started. |
| OUT-7, OUT-11 | adversarial | Re-attack `evidence_provenance` self-report + `against="any"` hole. |
| OUT-6 | adversarial | Threat-model write-up for 4-point bright-line. |
| OUT-8 | adversarial | Human-footgun examination of tolerance/immunity ceremony separation. |
| OUT-4, OUT-9 | pathmaker | ADR-019 prose discipline: headline first; citation appendix. |
| OUT-16, OUT-19 | navigator / team-lead | Meta — process and team-composition. |

---

## Closing note

19 findings is a lot for one pass. The expected disposition is **not**
"each one becomes a ratified amendment." The expected disposition is:

- Some collapse to "we questioned that already in capture X; here's the
  answer, let's add a one-line crystallization to v3 so the next
  outsider doesn't ask again."
- Some are real dust to remove (suspects: OUT-6 point 1 motivation,
  OUT-11 `against = "any"`, parts of OUT-4 ceremonial axis).
- Some require ADR work (OUT-17 macro extension, OUT-18 citation
  rework).
- Some are meta-observations for the team to digest (OUT-19).

Either way, after this pass the inherited assumptions in v3 are
*explicit*. That's the outsider's deliverable. Whether the assumptions
hold up is the team's call.

— outsider, 2026-05-19

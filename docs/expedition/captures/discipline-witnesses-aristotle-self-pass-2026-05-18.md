# Capture — Aristotle Self-Pass on Discipline-Witnesses v2 + Adversarial + Naturalist Refinements

> **Date**: 2026-05-18
> **Author**: single-instance Claude, after v2 + adversarial-self-attack + naturalist-self-pass
> **Relation to prior captures**: runs Phase 1-8 first-principles
> deconstruction on the load-bearing principles in v2 (and the
> refinements from adversarial/naturalist captures). Goal is to find
> the irreducible kernel of each principle so the team-aristotle pass
> works at the frontier rather than rederiving foundations.
> **Honest caveat**: aristotle's Phase 1-8 deconstruction is the
> cognitive mode hardest to do single-instance — team-aristotle's
> prompting specifically activates it. This pass is an *approximation*;
> team-aristotle should attack what's here, not inherit-without-attack.
> **Status**: append-only capture

> **What this is for**: when the team aristotle agent runs against v2 +
> adversarial + naturalist captures, they should attack the kernel
> claims surfaced below, not rederive them. The capture also surfaces
> several NEW insights that emerged from Phase 1-8 application — the
> Principle 2 reframe in particular (substrate-witness/cross-crate-
> witness unification under "this-code's-substrate vs other substrate"
> axis) was not visible from outside this mode.

---

## Posture

Approximating Phase 1-8 with my best understanding of the discipline:

- **Phase 1**: What's the visible claim?
- **Phase 2**: What inherited assumptions does the claim make?
- **Phase 3**: What does the claim look like with each assumption stripped?
- **Phase 4**: What's the irreducible kernel after stripping?
- **Phase 5**: What conclusions are structurally forced from the kernel?
- **Phase 6**: How does the kernel relate to adjacent kernels?
- **Phase 7**: What does the kernel predict about extension cases?
- **Phase 8**: Verdict — ratify the claim, refine it, or replace with kernel-derived version

Full deconstruction on 4 most load-bearing principles. Phase 1-8 lite
on 2 supporting principles. Refinements (R-Ar1 ... R-Ar?) for team
folding; frontier questions for team-aristotle.

---

## Principle 1 — Tier-honesty mandates audit-side enforcement

### Phase 1 — Visible claim
The audit must report the tier its verification work actually
supports, never stronger. Per ADR-005 Amendment 3.

### Phase 2 — Inherited assumptions
- **A**: Trust is "extended" by consumers on the basis of the status word
- **B**: The status word IS the trust-extension surface (not, e.g.,
  the JSON schema docs, or a separate human review)
- **C**: Tier names are meaningful — consumers distinguish them and
  make different decisions accordingly
- **D**: Truth-of-claim verification IS the developer's responsibility,
  not the audit's
- **E**: The audit's own work is verifiable (otherwise the audit could
  lie about what it did)

### Phase 3 — Stripping
- Strip A: if consumers don't rely on the status word, the audit is
  informational, not a trust boundary — but ADR-005 sub-clause F
  ratifies the audit AS a trust boundary. So A is structurally
  necessary.
- Strip B: if trust-extension lives elsewhere (e.g., human PR review),
  the audit becomes advisory — contradicts the trust-boundary role.
  Structurally necessary.
- Strip C: if tier names aren't meaningful, the 4-tier enum collapses
  to pass/fail. Loses ability to gate on Reachability-vs-Execution.
  Structurally necessary for the enum to exist.
- Strip D: if the audit verifies truth, it becomes a complete
  verification system — theorem prover or similar. Contradicts
  compose-don't-compete (ADR-002). Structurally necessary.
- Strip E: if the audit can lie about its work, tier-honesty is
  unverifiable. The audit's code must be itself tested — atk_w7
  contracts exist for this. Structurally necessary; meta-recursion
  (audit-of-audit).

### Phase 4 — Irreducible kernel
**The audit's status word is a trust-extension surface; consumers act
on it; the work behind the word is finite and structured; therefore the
word must reflect the work, lower-bound only.**

### Phase 5 — Structurally forced conclusions
- Multi-tier system (multiple distinguishable depths of work) → 4-tier
  WitnessTier enum
- Audit_hint dimension (multiple kinds of work at same depth) → parallel
  hint axis (already in audit.rs)
- **"Reports lower bound, never upper bound"** rule → ratchet
  asymmetry (this is implicit in v2 but never named as a ratchet)
- Audit's own implementation must be tested (audit-of-audit recursion)
- Schema-as-contract at producer-consumer boundary → tier-honesty
  extends backwards to inputs (sidecar schema validation IS this)

### Phase 6 — Adjacency
- Adjacent to recognition-not-design (ADR-006): tier-honesty is
  *recognition* of what the audit did, not *design* of what it might
  have done
- Adjacent to compose-don't-compete (ADR-002): tier-honesty is what
  makes composition tractable — consumers can chain audits without
  losing fidelity
- Adjacent to substrate-over-memory: tier-honesty applied inward —
  audit reports its own substrate state, not its memory

### Phase 7 — Extension predictions
- Tier-honesty extends to substrate-witnesses ✓ (v2 does this)
- Tier-honesty extends to cross-crate witness evaluation ✓ (ADR-005 Am
  3 sub-amendment)
- Tier-honesty extends to schema-versioning during migration (open
  question; v2 deferred via R-A2 / Attack-B)
- **Prediction**: any future witness family where the audit completes
  encoded verification work reaches Execution tier; families requiring
  external execution stay at Reachability until A4-A5

### Phase 8 — Verdict
Original claim survives intact. Kernel correctly identified
(status-word-as-trust-surface). Predictions land cleanly. **Refinement**:
explicitly name the **ratchet-asymmetry** property (lower-bound-only),
which v2 has implicitly but never articulated. The ratchet IS the
discipline; without it, "tier-honesty" is just "be honest sometimes."

**Refinement R-Ar1**: name the ratchet-asymmetry explicitly in v3 —
"the audit reports lower-bound; promotions require evidence; downgrades
are automatic when evidence falters." Symmetric to the substrate-stale
hint from R-A3 (predicate-passes-but-pin-stale demotes from Execution
to Reachability).

---

## Principle 2 — Substrate-witnesses are ordinary antigens with non-code substrate

### Phase 1 — Visible claim
Discipline-antigens aren't a special category; they're ordinary
antigens whose witness predicates evaluate against non-`.rs` substrate.

### Phase 2 — Inherited assumptions
- **A**: There's a single "antigen" abstraction that encompasses both
  code and non-code failure-classes
- **B**: Witnesses can be categorized by what substrate they evaluate
  against (code vs non-code)
- **C**: The existing antigen machinery extends naturally to non-code
  substrate without forking the data model
- **D**: "Discipline" failure-class is meaningfully a sub-class, not a
  parallel system

### Phase 3 — Stripping
- Strip A: if discipline-antigens are NOT antigens, antigen-the-project
  doesn't cover discipline; fragments the failure-class memory.
  Discipline IS a structural failure-class per ADR-001's 8-class
  taxonomy. Structurally necessary.
- Strip B: if witnesses can't be categorized by substrate kind, the
  categorization must use verification-kind (behavior vs structure vs
  attestation) — but verification-kind is downstream of substrate-kind.
  B is the right axis.
- Strip C: if existing data model can't extend, either (a) it forks or
  (b) it was wrong from the start. The existing `#[immune(X, witness =
  Y)]` with Y as an expression accommodates compound substrate-predicates
  without fork. Structurally sound.
- Strip D: if discipline isn't a sub-class, what is it? Stripping reveals
  it's just "failure-class where verification depends on non-code
  substrate." That's the definition. Structurally sound after strip.

### Phase 4 — Irreducible kernel
**A failure-class can be presented at a code site even when its
verification depends on substrate other than that code site. The antigen
abstraction is broad enough to cover this if the witness primitive is
broad enough to evaluate against the relevant substrate.**

### Phase 5 — Structurally forced conclusions
- Witness primitive must be extensible to non-code substrate types
- Predicate language is the right shape (declarative over substrate of
  any kind)
- No new top-level antigen abstraction needed
- Tier-honesty applies uniformly
- "Discipline" is a USAGE category, not a structural category

### Phase 6 — Adjacency
- Adjacent to ADR-001's 8-class taxonomy: discipline failure-classes
  ARE one or more of the 8 classes (likely Engineering Practice /
  Process)
- Adjacent to recognition-not-design: substrate-witness reframe
  RECOGNIZES that witnesses are predicates over substrate; the "code"
  restriction was contingent, not structural

### Phase 7 — Extension predictions
- Future witness families (simulation-witness, fuzz-coverage-witness,
  ML-verification-witness) can be added via substrate-witness pattern
- **Cross-crate witnesses fit here too**: cross-crate code IS code,
  but it's not THIS code being audited

### Phase 8 — Verdict + NEW INSIGHT
Original claim survives but framing needs sharpening. **The "non-code
substrate" axis is contingent; the deeper axis is "this-code's-substrate
vs other substrate."**

Cross-crate witnesses (currently handled via cross-crate audit_hint
per ADR-005 Am 3 sub-amendment) are a special case of substrate-witnesses
where the "other substrate" is code in a different crate. By the
substrate-witness pattern, the consuming workspace's audit can read
the dep's source (substrate), parse it, find the witness function
(predicate evaluation), and report Reachability + hint —
*structurally identical* to substrate-witnesses reading sidecar JSON
and reporting Reachability + hint.

This UNIFIES two seemingly-separate witness families:
- **Substrate-witnesses over JSON sidecars** (discipline-antigens)
- **Cross-crate witnesses over dep's source** (existing v0.1 mechanism)

Both are substrate-witnesses where substrate is "other than this code."
Both follow tier-honesty discipline. Both could share predicate-language
infrastructure if the abstraction is taken seriously.

**Refinement R-Ar2**: reframe v3's substrate-witness section to use
"other substrate" axis rather than "non-code substrate." This
generalizes the framework and unifies substrate-witnesses with
cross-crate witnesses. Both are evaluations of predicates over substrate
that isn't the code being audited.

**Frontier question for team-aristotle**: does this unification hold
under deeper deconstruction, or are there structural differences
between sidecar-substrate and dep-source-substrate that prevent the
unification? If it holds, ADR-019 has wider scope than v2 acknowledged
— the substrate-witness primitive covers both new (discipline) and
existing (cross-crate) cases.

---

## Principle 3 — Closed combinator grammar is the right ceiling

### Phase 1 — Visible claim
Predicate language uses closed combinator grammar (`all_of` / `any_of`
/ `not`) with sealed leaf primitives; no user-defined functions, no
conditionals.

### Phase 2 — Inherited assumptions
- **A**: Predicates need mechanical evaluation by the audit
- **B**: Closed grammar prevents Turing-tarpit
- **C**: Boolean composition is sufficient for substrate-witness needs
- **D**: User-defined-fn or conditionals would re-introduce
  trust-the-witness problem
- **E**: Leaf set is sufficient (or extensible only via well-defined paths)

### Phase 3 — Stripping
- Strip A: if predicates don't need mechanical evaluation, they're
  decorative. The whole point is mechanical verification.
  Structurally necessary.
- Strip B: if grammar is open (user-defined operations), predicate
  evaluator becomes general programming language interpreter; halting
  problem; verification undecidable. Structurally necessary.
- Strip C: 3 combinators (`all_of` / `any_of` / `not`) give full
  propositional logic — XOR, implication, IFF all expressible.
  Stripping reveals: even fewer combinators (just `all_of`)
  insufficient (can't express alternatives or negation); 3 is the
  structurally minimal complete set.
- Strip D: user-defined-fn means audit trusts the function. Same
  failure mode as `witness = trust_me`. Structurally necessary.
- Strip E: if leaf set is permanently sealed, future discipline needs
  blocked. Tier-3 witness-provider crates provide adoption-time
  extensibility (already in v2). Use-site is sealed; adoption-time is
  extensible. Structurally sound.

### Phase 4 — Irreducible kernel
**Predicates must be (a) mechanically evaluatable by the audit, (b)
terminating, (c) verifiable without invoking arbitrary code. Closed
propositional-logic composition over a sealed-but-extensible leaf set
is the simplest grammar satisfying all three.**

### Phase 5 — Structurally forced conclusions
- 3 combinators (`all_of`, `any_of`, `not`) span propositional logic —
  nothing simpler suffices, nothing more is needed
- Leaf set sealed at use-site; extensible at adoption-time via
  witness-provider crates
- Each leaf must be terminating and side-effect-free
- New combinators (temporal logic, weighted counting) are LEAVES with
  internal structure, NOT new combinators (e.g., `k_of_n([...])` is a
  leaf that internally counts; `signing_order(...)` is a leaf that
  internally checks order)

### Phase 6 — Adjacency
- Adjacent to TCR-diversity-from-fixed-segments biology rhyme (per
  naturalist R-N2): vast expressive power from compositions of FIXED
  primitives
- Adjacent to compose-don't-compete: closed grammar enables composition
  without forking
- Adjacent to ADR-007 anti-YAGNI: ship the closed-grammar even though
  restrictive; structure forces good usage

### Phase 7 — Extension predictions
- Future predicate needs are met by NEW LEAVES (not new combinators)
- Witness-provider crates contribute leaves under a published
  trait/contract — needs leaf-trait specification (deferred to v0.2+)
- Naming conventions for leaves: verb-based (`signers`, `oracles_complete`,
  `fresh_within_days`)
- Audit must support depending on witness-provider crates' leaves at
  audit-time (cargo dep graph walking)

### Phase 8 — Verdict
Original claim survives intact. **Refinement**: explicitly distinguish
**use-site sealing** (no user-defined-fn) from **adoption-time
extensibility** (witness-provider crates ship new leaves). v2 has this
implicitly but the first-principles distinction is sharper —
extensibility lives at a different layer than expressiveness.

**Refinement R-Ar3**: name the use-site-sealed / adoption-time-extensible
distinction explicitly in v3. This isn't a contradiction; they're
different layers. Use-site users have a fixed set; adoption-time
authors (cargo manifest editors) can pull in more.

---

## Principle 4 — Code-locality (not doc-locality)

### Phase 1 — Visible claim
Sidecars live with the code that presents the antigen, not with the doc.

### Phase 2 — Inherited assumptions
- **A**: Sidecars need a location somewhere
- **B**: Co-locating with code means PR review naturally covers them
- **C**: Each antigen presentation is independently ratifiable (per-site)
- **D**: Doc-locality would centralize ratifications, which is bad

### Phase 3 — Stripping
- Strip A: trivially false to strip; substrate-witnesses need persistent
  state somewhere
- Strip B: if PR review doesn't naturally cover sidecars, code-locality
  loses its UX advantage. But PR review DOES cover sidecars in the
  same diff. Contingently true; load-bearing for adoption.
- Strip C: if each presentation isn't independently ratifiable, why
  per-presentation? Because each site has different context (different
  file, function, consumers). Different context can mean different
  appropriate signers, freshness, oracles. Structurally sound.
- Strip D: doc-locality WOULD centralize (one sidecar per doc regardless
  of how many presenting sites the doc covers). Centralization erases
  per-site context. Structurally sound for *specific failure mode*
  (loss of per-site granularity).

### Phase 4 — Irreducible kernel
**Per-site discipline requires per-site substrate. Co-locating substrate
with the presenting site is the only structure that preserves per-site
granularity. Code-locality follows from per-site discipline.**

### Phase 5 — Structurally forced conclusions
- Per-antigen-per-file granularity (multiple sidecars per file possible)
- Per-item granularity within sidecar (items[] array)
- PR-review workflow alignment (sidecar in same diff as code)
- Refactor tooling needed (sidecars move with code) — `attest move`
  per v2 / R-A6

### Phase 6 — Adjacency
- Adjacent to MHC presentation per-cell biology rhyme (per-cell antigen
  processing)
- Adjacent to germinal-center biology rhyme (distributed substrate,
  local validation)
- BUT per naturalist R-N5: clinical-infrastructure DOES use central
  registries (medical records). The biological-narrow rhyme is what's
  load-bearing here; clinical analog rhymes with *something else*
  (see below)

### Phase 7 — Extension predictions
- Cross-crate sidecars live in CONSUMING crate (per R-A7)
- Macro-invocation-site is the input layer (per R-A8)
- Coarser-grained disciplines (package-wide, workspace-wide) follow
  same pattern (sidecar at scope of presentation, per R-A9)
- Generated-code-output is OUT OF SCOPE — discipline at input layer
- **NEW**: doc-level discipline (ratification of the DOC ITSELF by team
  X) might be appropriate as a *separate primitive* from
  code-presenting-discipline. v2/v3 don't currently distinguish; this
  might be a real gap.

### Phase 8 — Verdict + NEW INSIGHT
Original claim survives. Kernel sharpens to: **per-site discipline →
per-site substrate**. The deeper claim makes "code-locality" a
*consequence*, not a primary commitment. If discipline isn't per-site,
location can be elsewhere.

**Doc-level discipline ratification (ratification OF THE DOC itself,
not of code-presenting-the-doc) might be a separate primitive.** Use
case: "this discipline doc has been ratified by team X with version
1.0; downstream consumers can rely on it being canonical." This is
DOC-LEVEL ratification — about the doc, not about any specific code
site that presents the doc.

v2/v3 substrate-witnesses cover code-locality (code presents antigen;
sidecar lives with code). They don't cover doc-locality (the doc
itself is ratified). The latter might be the *natural* home for
doc-side metadata that v1 turn 7 reframed away from for code-sites
but could exist as a parallel primitive.

**Refinement R-Ar4**: surface to team-aristotle the possibility of
**doc-level discipline ratification as a parallel primitive** —
sidecar adjacent to the doc, asserting the doc's own ratification
state. This is DIFFERENT from code-presenting-the-doc sidecars.
Could be deferred to v0.2+ or even a separate ADR; flagging as visible
structural gap that emerged from first-principles deconstruction.

---

## Principle 5 — Recognition-not-design ratio is healthy (Phase 1-8 lite)

### Visible claim
Leaves recognize existing substrate (markdown, git, JSON); combinators
+ schema + CLI are designed. Ratio is design-small + recognition-vast.

### Quick deconstruction
This is a **posture-level claim**, not a structural commitment.
Contingent property of choices made. Could be tilted toward design
(invent more) or recognition (use fewer custom shapes). Healthy ratio
is a posture per ADR-006, not an irreducible kernel.

### Verdict
Not deeply load-bearing in the first-principles sense. Posture-level;
team-aristotle can attack the ratio claim but won't find a deeper
kernel. **No refinement needed.**

---

## Principle 6 — Three pieces ship together (Phase 1-8 lite)

### Visible claim
Predicate language + Ratification schema + CLI ship together.

### Quick deconstruction
Structurally forced by mutual dependence: predicate needs schema needs
CLI needs predicate. Each alone fails (predicate without schema has
nothing to evaluate; schema without CLI fails on adoption; CLI without
predicate is empty scaffolding). The kernel is just "mutual dependence
forces co-shipping" — no deeper structure to find.

### Verdict
Structurally forced. **No refinement needed.** Team-aristotle can
verify the mutual-dependence claim but no kernel below it.

---

## Refinements that survive — to fold into v3

Listed in priority order:

### R-Ar1 — Name the ratchet-asymmetry property explicitly
Add to v3 tier-honesty section: **"The audit reports lower-bound;
promotions require evidence; downgrades are automatic when evidence
falters."** This is the ratchet that IS the tier-honesty discipline.
Without it, "tier-honesty" reduces to "be honest sometimes." Symmetric
to the substrate-stale hint from R-A3 (predicate-passes-but-pin-stale
demotes from Execution to Reachability).

### R-Ar2 — Reframe "non-code substrate" as "other substrate"
This is the **biggest insight from this pass**. Replace v2's "witnesses
that evaluate against non-`.rs` substrate" with **"witnesses that
evaluate against substrate other than the code being audited."** This
generalizes the framework and **unifies substrate-witnesses with
cross-crate witnesses** — both are evaluations of predicates over
substrate that isn't this code. ADR-019 has wider scope than v2
acknowledged. Worth checking with team-aristotle whether the
unification holds under deeper deconstruction (it might not — there
might be structural differences between sidecar-substrate and
dep-source-substrate that prevent unification).

### R-Ar3 — Use-site sealed vs adoption-time extensible distinction
Add to v3 predicate-language section: **the closed combinator grammar
is sealed at use-site (no user-defined-fn) but extensible at
adoption-time (witness-provider crates ship new leaves).** These are
different layers, not a contradiction. Use-site users have a fixed
set; adoption-time authors (cargo manifest editors) can pull in more.

### R-Ar4 — Surface doc-level discipline ratification as parallel primitive
Flag to team-aristotle: **doc-level discipline ratification** (the
doc itself is ratified by team) might be a separate primitive from
code-presenting-discipline (function in code is ratified per-site).
v2/v3 don't currently distinguish. Use case: "this discipline doc has
been ratified by team X with version 1.0." Could be deferred to v0.2+
or a separate ADR. Visible structural gap that emerged from
first-principles deconstruction.

---

## What doesn't change

All four major principles survive Phase 1-8 deconstruction with their
core claims intact:
- Tier-honesty audit-side enforcement — survives; refinement names the
  ratchet
- Substrate-witnesses are ordinary antigens — survives; refinement
  generalizes the axis
- Closed combinator grammar — survives; refinement names use-site/
  adoption-time layers
- Code-locality — survives; refinement surfaces doc-level as parallel
  primitive

No principle was invalidated. Three refinements are sharpenings;
one (R-Ar2) is a structural insight that genuinely widens the
framework. One (R-Ar4) is a gap surfaced for team consideration.

---

## What the team aristotle pass should attack

With these refinements absorbed (and the adversarial + naturalist
refinements):

1. **R-Ar2 unification claim**: does the substrate-witness/cross-crate-
   witness unification hold under deeper deconstruction? If yes,
   ADR-019 scope widens significantly. If no, name the structural
   distinction that prevents unification.

2. **R-Ar4 doc-level ratification gap**: is this a real gap requiring
   a parallel primitive, or is it absorbed by existing antigen
   declaration mechanism (`#[antigen(..., discipline_doc = ...)]`
   with the doc itself implicitly ratified by its presence in repo
   + git history)?

3. **The kernel "per-site discipline → per-site substrate"** (Principle
   4): is this universally true, or are there discipline types that are
   inherently NOT per-site? Workspace-wide invariants ("all code in this
   workspace uses Result, never Option for fallible operations") might
   not be per-site. Where do they live?

4. **The kernel "predicates must be mechanically evaluatable +
   terminating + verifiable without invoking arbitrary code"**
   (Principle 3): is the "verifiable without invoking arbitrary code"
   condition exactly right? What about leaves that DO need to invoke
   arbitrary code (e.g., a leaf that runs `cargo check` to verify a
   workspace compiles)? Does this break the kernel or fit within it
   (the leaf invokes specific known tools, not user-defined-fn)?

5. **The ratchet-asymmetry property** (R-Ar1): does it have edge
   cases where automatic downgrade is wrong? E.g., a phantom-type
   witness whose constructor changed — does the audit auto-downgrade,
   or does it require explicit re-attestation?

6. **The audit-of-audit recursion** (Principle 1, structurally
   forced): is this actually bounded, or is there an infinite regress
   risk? (The audit's tests need their own audit-honesty, which needs
   tests, which need... ?)

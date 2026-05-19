# Capture — Aristotle Team-Pass on Discipline-Witnesses v2 + Self-Pass Refinements

> **Date**: 2026-05-18
> **Author**: team-aristotle (Opus 4.7 1M, solo team pass)
> **Relation to prior captures**: this is the *real* Phase 1-8 pass — the
> aristotle-self-pass capture explicitly flagged itself as approximation
> and invited attack on its kernels. This pass attacks them.
> **Discipline**: I do NOT inherit the self-pass findings. Where my
> reasoning ratifies them, that's my-reasoning-ratifies (named so the
> ratification is independent evidence). Where my reasoning replaces
> them, I name what the self-pass missed. Where my reasoning extends
> beyond what the self-pass reached, those are NEW kernels (F-findings,
> in my signature shape).
> **Status**: append-only capture

> **Format**: numbered structural findings (F1, F2, ...) as my signature
> shape. Each F is a kernel that survived Phase 1-8 with reasoning shown.
> Where a finding ratifies/replaces/extends a self-pass refinement, named
> explicitly. Final section: frontier questions for the next team passes.

---

## Posture

Phase 1-8 done seriously means: the kernel I extract must survive
hostile interrogation, and the chain of necessity from kernel to
implementation must be visible. "Structurally necessary" is a strong
claim — I will name what would have to be false in the world for the
claim to fail, not just "my intuition says load-bearing."

Also: I am running this in the discipline named by the campsite —
**structural complexity is the point**. Where the self-pass flattened a
gradient (held a position because it seemed feasible), I will keep the
fuller gradient even if the more elaborate end pushes back into
preliminary-first-principle territory.

Eight major Phase 1-8 deconstructions below + frontier extensions. Where
prior pass surfaced a frontier question, I attack it head-on.

---

## F1 — The R-Ar2 unification claim DOES hold, but for a sharper reason than self-pass named

### What self-pass claimed

R-Ar2 reframed "non-code substrate" → "other substrate," and asserted
that substrate-witness-over-JSON-sidecar AND cross-crate-witness-over-
dep-source are *both* "substrate-witnesses over substrate-other-than-
this-code." Self-pass invited team-aristotle to attack: does the
unification hold, or are there structural differences that prevent it?

### Phase 1 — Visible claim, made precise

Two witness families:
- **Substrate-witness over JSON sidecar**: audit reads
  `src/foo.attest/X.json`, parses as `Ratification`, evaluates predicate
  against on-disk substrate.
- **Cross-crate witness over dep source**: audit reads dep's `.rs`
  files (substrate), resolves `witness = dep::test_fn` identifier,
  reports Reachability + `cross-crate-witness-not-locally-executable`.

R-Ar2 claim: both are predicate-evaluations against substrate-not-this-
code; same primitive.

### Phase 2 — Assumptions

- **A**: "Substrate" is a uniform abstraction across JSON files and
  Rust source files.
- **B**: "Predicate evaluation" is a uniform operation across (i)
  JSON-schema validation + boolean composition over typed leaves and
  (ii) AST walk + identifier resolution.
- **C**: The audit's work in both cases is structurally the same kind
  of work.
- **D**: Unification is a *gain* (shared infrastructure, shared
  tier-honesty discipline), not a loss (collapses meaningful
  distinctions).
- **E**: "Other substrate" is a meaningful structural axis (not just a
  loose family resemblance).

### Phase 3 — Stripping

- **Strip A**: "Substrate" as a uniform abstraction. Without A, the
  two cases are different types of input to different recognition
  pipelines; the "unification" is just a rhetorical reframe with no
  shared infrastructure. **A is contingently true**: any byte-string
  on disk is substrate from a sufficiently abstract POV. But this is
  too abstract to do work. The work-doing claim is sharper: *both
  substrates carry typed claims that the audit reads and validates
  structurally*. JSON sidecar carries typed `Ratification` claims;
  dep source carries typed `#[immune(...)]` claims that the audit
  parses via `syn`. Both are typed-claim-substrates. **Strip-A
  reveals this stronger claim.**

- **Strip B**: "Predicate evaluation" as a uniform operation.
  - Sidecar case: audit reads JSON, validates against schema, evaluates
    closed combinator predicate over leaf primitives.
  - Cross-crate case: audit reads `.rs`, parses via `syn`, walks AST,
    matches witness identifier against function index, reports
    resolution.
  - Are these uniform? They share structure: *parse substrate →
    extract typed claim → evaluate against expected shape → report
    tier*. They differ in the recognition machinery (JSON parser vs
    Rust parser; combinator-predicate vs identifier-resolution).
  - **Strip B reveals**: uniformity is at the *workflow* level, NOT
    at the recognition-machinery level. The "unification" buys shared
    workflow discipline (tier-honesty, ratchet-asymmetry, audit-hint
    parallel axis), not shared parser code.

- **Strip C**: Audit's work is the same kind of work.
  - Sidecar case: the audit is *the verifier* — it completes the full
    predicate-evaluation the witness encodes.
  - Cross-crate case: the audit is *not the verifier* — the
    verification work the cross-crate witness encodes is "test runs
    and passes," which the audit cannot do locally (per the
    `cross-crate-witness-not-locally-executable` hint). The audit
    completes only identifier-resolution; reports Reachability.
  - **Strip C reveals a structural asymmetry**: substrate-witnesses
    over sidecar reach Execution because the audit completes the
    encoded verification work; cross-crate witnesses cap at
    Reachability because the audit cannot complete the encoded
    verification work. **The "unification" does NOT mean both reach
    the same tier ceiling**. It means both follow the same
    tier-honesty discipline (report only what was done).

- **Strip D**: Unification is a gain. Without D, the reframe is a
  category error that erases a real distinction. **Strip D fails**
  in the sense that the cases ARE structurally different at the
  machinery level (Strip B) AND at the tier-ceiling level (Strip C).
  But unification at the *discipline level* (tier-honesty,
  ratchet-asymmetry) is still a gain — these disciplines were already
  applied uniformly per ADR-005 Am 3; the reframe just names that
  uniformity.

- **Strip E**: "Other substrate" is the structural axis. Without E,
  the cases are united by something else (or nothing). **Strip E
  fails**: at the discipline level, "is the substrate something
  other than this code" is what determines whether the audit can
  complete the encoded verification work via local mechanisms. For
  sidecar substrate (on disk, in this workspace, schema-validatable),
  the audit completes the work. For dep-source substrate (in another
  workspace, with its own test harness), the audit cannot complete
  the work. The axis matters; it's not arbitrary.

### Phase 4 — Irreducible kernel

**There are two distinct claims, both true, that the self-pass
conflated**:

- **Kernel A (discipline-level unification, HOLDS)**: substrate-
  witnesses (over sidecars) and cross-crate witnesses (over dep source)
  are both governed by the same tier-honesty discipline applied
  uniformly to "audit verifies its own work, lower-bound only,
  reports actually-completed verification work." This unification IS
  ADR-005 Amendment 3 + its cross-crate sub-amendment. The reframe
  names existing unity.

- **Kernel B (machinery-level unification, DOES NOT HOLD)**: the
  recognition machinery (JSON parser + combinator evaluator vs Rust
  parser + AST walker + identifier resolver) is NOT unified and should
  not be unified. They share workflow shape (parse → validate →
  report) but no shared code beyond the audit's tier-emission helpers.

### Phase 5 — Structurally forced conclusions

- ADR-019's *scope* does NOT widen to include cross-crate witness
  mechanism — that mechanism already exists per ADR-005 Am 3 sub-
  amendment and ships under different recognition pipelines.
- ADR-019 SHOULD explicitly position substrate-witnesses as *another
  instance* of the tier-honesty discipline already applied to
  cross-crate witnesses. The discipline is the unification; the
  mechanism is parallel.
- The "other substrate" reframe is useful as a *teaching frame* but
  must not be used to argue for shared implementation infrastructure.
- The R-Ar2 reframe should be ABSORBED into v3 with the asymmetry
  named explicitly, NOT as the widening of ADR-019's scope.

### Phase 6 — Adjacency

- Adjacent to OQ-1 of ADR-005 Am 3: "tier-honesty for non-audit
  recognition mechanisms" — needs three instances outside audit.
  Substrate-witnesses are *inside* audit (per v2 §"Connection to
  ADR-005 Am 3 OQ-1") — they don't promote OQ-1. **But cross-crate
  witnesses also live inside audit.** Both are audit-internal
  recognition extensions. So the discipline-level unification (kernel
  A) is *intra-audit* unification — already implicit in Am 3 + its
  sub-amendment, just unnamed.
- Adjacent to ADR-002 compose-don't-compete: each recognition pipeline
  composes with substrate-specific parsers (serde_json for sidecars,
  syn for Rust source). Composition argues *against* machinery
  unification — share the discipline, not the parser code.

### Phase 7 — Extension predictions

- Future witness families that read substrate-not-this-code
  (e.g., simulation-result-witness, fuzz-corpus-witness,
  ML-eval-witness) will each ship their own recognition pipeline AND
  inherit the tier-honesty discipline. The pattern recurs at the
  discipline level; the machinery is per-substrate-kind.
- Specifically: a "fuzz-corpus-witness" would read corpus state
  (substrate-other-than-this-code), have its own parser
  (corpus-format-specific), and earn Execution-tier-when-corpus-
  current OR Reachability-when-corpus-stale. Same tier-honesty
  workflow; different machinery.

### Phase 8 — Verdict

**Self-pass R-Ar2 was 80% right and 20% wrong.** The 80%: the
discipline IS unified; calling it out explicitly improves v3. The 20%:
the unification is at the discipline level, NOT the machinery level,
and ADR-019 scope does NOT widen to include cross-crate witnesses (they
already ship under different recognition pipelines via ADR-005 Am 3
sub-amendment).

**F1 (irreducible kernel, my-reasoning-ratifies and refines R-Ar2)**:
substrate-witnesses (over sidecars) and cross-crate witnesses (over
dep source) share *tier-honesty discipline* applied uniformly, not
*recognition machinery*. The discipline-level unification is real and
should be named in v3; the machinery-level unification is a category
error to avoid. ADR-019 should cite this asymmetry explicitly to
prevent future passes from re-deriving the wrong unification.

---

## F2 — The R-Ar4 doc-level ratification claim is ABSORBED, not parallel

### What self-pass claimed

R-Ar4 surfaced "doc-level discipline ratification" as a possible
parallel primitive — sidecar adjacent to the doc, asserting the doc's
own ratification state (e.g., "this discipline doc has been ratified by
team X with version 1.0"). Self-pass flagged this as a visible
structural gap and invited team-aristotle to decide.

### Phase 1 — Visible claim

Doc-level ratification is a different primitive from
code-presenting-the-doc ratification. The former is about the doc
itself ("team X ratified version 1.0 of this discipline"); the latter
is about a code site that presents the doc as its discipline reference.

### Phase 2 — Assumptions

- **A**: Docs need their own ratification state separate from any code
  site that references them.
- **B**: Doc ratification claims something different from
  code-presents-discipline-doc claims.
- **C**: Doc ratification can't be expressed via the existing antigen
  declaration machinery.
- **D**: Code-presenting-discipline-doc claims do NOT cover what
  doc-ratification claims would cover.

### Phase 3 — Stripping

- **Strip A**: docs don't need ratification state.
  - Counter-evidence: discipline-antigens cite `discipline_doc`; if
    the doc is mutable and unratified, the citation is brittle.
  - But this is *exactly* what `ratified_doc(min_version, anchor)`
    leaf primitive checks. Strip-A is not strippable because the
    need exists, BUT the need is already met by an existing leaf.

- **Strip B**: doc ratification ≠ code-presenting-discipline-doc.
  - Doc ratification asserts: "doc D is ratified at version V by
    signers S."
  - Code-presenting-discipline-doc asserts: "code site C presents
    discipline D, witnessed by `ratified_doc(D, min_version=V) AND
    signers(required=S)`."
  - The two claims are equivalent if S is the doc-level signers and V
    is the doc-level version. **The doc-level claim is what the code-
    level claim's `ratified_doc` leaf evaluates against.** Strip-B
    reveals: doc-level ratification is the *substrate* the
    code-level claim's predicate reads.

- **Strip C**: doc ratification can't be expressed via antigen.
  - This is where the structural question really lives. Can antigen
    declare "doc D itself has a discipline that says it must be
    ratified by team X at version V"?
  - Three ways to do this within existing antigen:
    1. **Doc-as-presenting-site**: treat the doc as if it presents
       a "DocRatificationDiscipline" antigen. Sidecar lives next to
       the doc (`docs/foo.md` → `docs/foo.attest/DocRatificationDiscipline.json`).
       The substrate-witness primitive already supports this — there's
       no rule that says antigens must be presented at `.rs` sites.
       But the *scan machinery* currently walks `.rs` files only.
       Extending the scan to also walk `.md` files for antigen
       presentation is a non-trivial change.
    2. **Doc-ratification-as-a-leaf**: ship a leaf primitive
       `doc_ratified(path, min_version, signers, ...)` that reads
       the doc's frontmatter / a sibling JSON file / a manifest
       entry, returns boolean. Used inside code-level predicates.
       This is essentially `ratified_doc` extended with signer
       checking — and the v2 schema's `Signer` mechanism is
       designed exactly to live in JSON sidecars, which are
       file-adjacent-substrate; nothing prevents a JSON sidecar
       from being adjacent to a `.md` file rather than a `.rs`
       file.
    3. **Workspace-level ratification doc**: a special file (e.g.,
       `docs/ratifications.json`) that the audit reads. Doc
       ratification becomes a workspace-state query, not a per-doc
       sidecar.

  - **Option 2 is the absorption path.** The substrate-witness
    primitive already supports "JSON sidecar adjacent to substrate;
    audit reads, validates, evaluates predicate against substrate
    state." The substrate can be a `.md` file just as easily as a
    `.rs` file. The current scan limitation (walks `.rs` only) is
    an implementation detail of where the antigen *presents*, not of
    where the sidecar can live.

- **Strip D**: code-presenting-discipline-doc claims do not cover
  what doc-ratification claims would cover.
  - Code-presenting claim: "code site C complies with discipline D
    per code-level evidence (signers, oracles, doc-ratification
    state)."
  - Doc-ratification claim: "doc D itself is ratified."
  - These are at different layers. Code-presenting claim *depends
    on* doc-ratification state (via `ratified_doc` leaf). But the
    doc-ratification state itself, if relevant on its own (e.g.,
    "this doc is a canonical reference for downstream consumers"),
    is a different kind of claim.

  - Crucial substrate-grep: **is "doc D is canonical" a
    discipline-failure-class antigen could express?** In antigen's
    8-class taxonomy, "Documentation Drift" is one of the classes
    (per ADR-001's failure taxonomy). A discipline-antigen for
    "DocumentationDrift" could be presented AT THE DOC SITE with a
    witness predicate like
    `signers(required=team_X) AND fresh_within_days(365)`. This is
    just substrate-witness over a `.md` substrate.

  - **Strip-D conclusion**: code-presenting claims do NOT cover
    doc-ratification claims; the latter is its own legitimate use
    case. BUT — and this is the load-bearing point — the substrate-
    witness primitive ALREADY ENCODES the doc-ratification use case;
    it just hasn't been exercised yet because the scan walks `.rs`
    only.

### Phase 4 — Irreducible kernel

**Doc-level ratification is NOT a parallel primitive — it's the same
primitive (substrate-witness over sidecar) applied to non-`.rs`
substrate. The structural question is whether the SCAN machinery
walks `.md` files to find `#[antigen]`-equivalent presentation sites
on docs.**

Two sub-cases:
- **Doc-side antigen presentation** (frontmatter or sibling JSON
  declares "this doc presents DocumentationDrift antigen with witness
  ..."): requires scan-walks-`.md`-files. New mechanism.
- **Doc-ratification-state-as-a-leaf** (code-side predicate evaluates
  doc state via a leaf primitive that reads `.md` frontmatter + a
  sibling JSON): requires only an extended `ratified_doc` leaf with
  signer-checking. No new mechanism; entirely within v0.1 substrate-
  witness scope.

The latter is the absorption path. The former is a future v0.2+
extension (scan-walks-`.md`) for cases where the doc itself is the
presentation site.

### Phase 5 — Structurally forced conclusions

- **v0.1 ships the absorption path**: extend `ratified_doc` leaf to
  optionally read sibling JSON (e.g., `docs/foo.md` + `docs/foo.ratify.json`)
  carrying signers/version. This is a single leaf-primitive evolution,
  not a new primitive. **NO new ADR needed for v0.1**; covered by
  ADR-019.
- **v0.2+ may add the scan-walks-`.md` mechanism** if a real adoption
  use case for doc-side antigen presentation emerges. Until then,
  the absorption path covers all known use cases.
- **R-Ar4's "parallel primitive" framing was wrong**: doc-level
  ratification is not parallel; it's the same primitive at a
  different substrate-site.

### Phase 6 — Adjacency

- Adjacent to ADR-006 recognition-not-design: doc-ratification-as-
  leaf RECOGNIZES that sidecars-adjacent-to-substrate is the pattern,
  applied uniformly. The scan-walks-`.md` extension would be DESIGN
  (inventing a new presentation site). Recognition path is preferred
  per ADR-006.
- Adjacent to ADR-007 anti-YAGNI: structurally-required is
  doc-ratification-as-leaf (used by every discipline-antigen with a
  ratified doc requirement). Scan-walks-`.md` is NOT structurally
  required by v0.1; defer.

### Phase 7 — Extension predictions

- The same absorption logic applies to other substrate types
  (oracle files, IDL files, test corpus state). Each is "substrate
  with a sibling JSON carrying ratification state, read by an
  extended leaf primitive."
- This is more powerful than v2 acknowledged: substrate-witnesses
  are not just for `.rs`-presented antigens with `.attest/` sidecars
  next to `.rs` files. They're for *any* substrate the audit can
  read, with a sibling JSON carrying the typed claim.
- The `.attest/` convention is *one instance* of the broader pattern
  "typed JSON sidecar carries the typed claim about adjacent
  substrate."

### Phase 8 — Verdict

**F2 (replaces R-Ar4)**: doc-level ratification is ABSORBED into the
substrate-witness primitive, not a parallel primitive. The absorption
path is: extend `ratified_doc` leaf to optionally read sibling JSON
carrying signers/version. NO new ADR; covered by ADR-019. The
scan-walks-`.md` mechanism is a v0.2+ extension reserved for cases
where the doc itself is the antigen presentation site (no current use
case demands it).

**Self-pass overshot**: R-Ar4's "parallel primitive" framing
introduced complexity that wasn't structurally required. The cleaner
absorption resolves the gap with a leaf-primitive evolution, not a
new ADR.

---

## F3 — "Per-site discipline → per-site substrate" is NOT universally true; workspace-wide invariants need a sibling primitive

### What self-pass claimed

Self-pass surfaced kernel for Principle 4 (code-locality) as **"per-
site discipline → per-site substrate."** Frontier question 3 asked
whether this holds universally or whether workspace-wide invariants
(e.g., "all code in this workspace uses Result not Option for
fallible operations") are NOT per-site, and where they live.

### Phase 1 — Visible claim

The kernel claims that discipline-antigens are per-presentation-site,
and their substrate (sidecars carrying signers, oracles, freshness)
is per-presentation-site.

### Phase 2 — Assumptions

- **A**: All disciplines decompose to per-site claims.
- **B**: Substrate-currency follows discipline-presentation-currency.
- **C**: "Site" is a meaningful unit-of-discipline (not too granular,
  not too coarse).
- **D**: Workspace-wide invariants either (i) decompose to per-site
  presentations or (ii) are a different kind of thing than antigens.

### Phase 3 — Stripping

- **Strip A**: Some disciplines don't decompose to per-site.
  - Example: "no `unsafe` blocks anywhere in this crate without a
    safety-doc per block." This decomposes: each `unsafe` block is a
    site presenting `UnsafeWithoutSafetyDoc` antigen, witnessed by a
    sibling sidecar. Decomposes cleanly.
  - Counter-example: "the workspace's Cargo.lock has been reviewed
    by the security team this month." There is NO `.rs` site that
    "presents" this. The discipline is at the *workspace* level.
    - Could express as a single top-level antigen presented on
      `Cargo.lock` (but that's not Rust; scan doesn't walk it). Or
      on the workspace root `Cargo.toml`. Or on a designated dummy
      site (`src/workspace_invariants.rs` with an `#[immune]`
      statement). All of these are workarounds for the fact that
      the discipline is fundamentally workspace-scoped, not site-
      scoped.
  - **Strip-A reveals**: some disciplines are inherently NOT
    per-site. The kernel doesn't cover them.

- **Strip B**: Substrate-currency doesn't follow presentation-currency.
  - If you accept Strip A, then for workspace-wide disciplines, the
    sidecar lives at the *workspace* (not per-site), and substrate-
    currency follows discipline-scope (workspace-currency, not
    site-currency). The fingerprint-pin is over workspace state,
    not function body.
  - This works — but it's a DIFFERENT shape than v2's per-site
    sidecars. Same primitive at different granularity is acceptable
    (self-pass R-A9 / R-Ar9 acknowledges granularity follows
    presentation-scope), BUT the **fingerprint** dimension is the
    catch: workspace-wide fingerprints are
    (a) not yet defined by `antigen_fingerprint` (which fingerprints
        individual items),
    (b) inherently coarser (workspace structure, all dep versions,
        all `Cargo.lock` content?).

- **Strip C**: "Site" is meaningful.
  - "Site" = function/struct/impl-block in v2. Coarser sites
    (file, module, crate, workspace) are all expressible as
    presentation-sites if `#[immune]` is allowed there. Cargo
    metadata `[package.metadata.antigen]` could be a presentation
    site for crate-level antigens. Workspace `Cargo.toml`
    `[workspace.metadata.antigen]` for workspace-level antigens.
  - **Strip-C reveals**: site is meaningful but multi-granular. The
    kernel needs the granularity dimension named explicitly.

- **Strip D**: Workspace-wide invariants decompose OR are different.
  - The honest answer is: SOME decompose to per-site presentations
    (per-`unsafe`-block discipline); SOME require workspace-scope
    presentation (Cargo.lock-monthly-review discipline). The
    primitive needs to handle both.

### Phase 4 — Irreducible kernel (revised)

**Discipline-antigens have a *scope* — site, file, module, crate, or
workspace — and substrate-currency follows discipline-scope. The
sidecar lives at the substrate that captures the discipline's scope.
The fingerprint dimension depends on the scope: per-site fingerprints
exist today (item fingerprints); coarser-scope fingerprints (file
fingerprint = hash of all relevant items; workspace fingerprint =
?) need definition as scope coarsens.**

The "per-site discipline → per-site substrate" kernel from self-pass
is a SPECIAL CASE of the broader kernel "discipline-scope determines
substrate-scope." It's the most common case, but not universal.

### Phase 5 — Structurally forced conclusions

- **Scope is a first-class dimension of discipline-antigens**, not
  implicit in the presentation site.
- **Sidecar location follows scope**, not just presentation site.
  - Site-scope → `.attest/` adjacent to the source file (as v2)
  - File-scope → `.attest/` at the file (single sidecar covers file)
  - Module-scope → `.attest/` at the mod's source file
  - Crate-scope → `.attest/` at the package root
  - Workspace-scope → `.attest/` at the workspace root
- **Fingerprint scope follows discipline scope**:
  - Site-scope: existing `antigen_fingerprint::Fingerprint` (item-level)
  - File-scope: hash-of-items-in-file (forced for compose)
  - Crate-scope: hash-of-crate-API + crate-config (Cargo.toml relevant
    bits)
  - Workspace-scope: hash-of-workspace-state (Cargo.lock + members'
    Cargo.toml relevant bits)
- **Open implementation question**: does antigen ship coarser-scope
  fingerprints in v0.1, or defer? Position: **defer to v0.2** for
  coarser-than-file fingerprints; v0.1 ships site-scope and file-scope
  only. Workspace-scope discipline ships in v0.1 only if the
  fingerprint can be defined trivially (e.g., hash of relevant
  workspace files); otherwise reserved.
- **v2 does NOT name scope as a dimension**. v2 implicitly assumes
  per-site (via the `.attest/` adjacent-to-source-file convention).
  R-A9 mentions coarser scopes ("sidecar follows presentation
  location") but doesn't elevate scope to a first-class dimension of
  the discipline-antigen's *shape*.

### Phase 6 — Adjacency

- Adjacent to ADR-001 8-class failure taxonomy: different classes may
  prefer different scopes (Process-class disciplines may be inherently
  workspace-scoped; Engineering Practice may be per-site).
- Adjacent to ADR-005 sub-clause F: trust-boundary at the scope-level
  — workspace-scope discipline's "is the workspace state current?"
  is the trust-boundary check, parallel to per-site
  "is this item's fingerprint current?"

### Phase 7 — Extension predictions

- Crate-level disciplines (e.g., "all public APIs ratified by API
  review team") naturally live at crate-scope; sidecar at package
  root.
- Workspace-level disciplines (e.g., "Cargo.lock reviewed monthly,"
  "no advisories from rustsec for current dep set") at workspace
  scope; sidecar at workspace root.
- Cross-cutting disciplines that span multiple sites but aren't fully
  workspace-wide are best modeled as multiple per-site presentations
  with a shared discipline doc (per the absorption pattern in F2).

### Phase 8 — Verdict

**F3 (extends Principle 4 kernel; replaces self-pass framing)**: the
self-pass kernel "per-site discipline → per-site substrate" is the
common case but not universal. The structurally honest kernel is
**discipline-scope determines substrate-scope; substrate-scope
determines sidecar-location, fingerprint-scope, and currency-check
shape.**

**v0.1 implication**: scope is a first-class dimension of
discipline-antigens; v2's per-site implicit assumption needs to be
made explicit. v0.1 ships site-scope and file-scope; coarser scopes
ship as their respective fingerprint mechanisms ratify. R-Ar9 (self-
pass) was directionally right but understated the depth of the
change: scope is not just "where the sidecar lives" but a structural
dimension of the antigen.

**Concrete consequence for ADR-019**: add a `scope:` field to
discipline-antigen declarations (default `site`); enumerate the
allowable scopes (`site | file | module | crate | workspace`);
specify the sidecar-location and fingerprint-mechanism per scope.

---

## F4 — The "verifiable without invoking arbitrary code" condition is ALMOST right; needs refinement to "verifiable without invoking author-defined code"

### What self-pass claimed

Self-pass kernel for Principle 3: predicates must be (a) mechanically
evaluatable, (b) terminating, (c) verifiable without invoking arbitrary
code. Frontier question 4 asked whether (c) is exactly right or
whether some leaves legitimately need to invoke specific known tools.

### Phase 1 — Visible claim

The closed combinator grammar prevents Turing-tarpit; the sealed leaf
set prevents user-defined-fn / trust-the-witness escape. Leaves
themselves should not invoke arbitrary code.

### Phase 2 — Assumptions

- **A**: "Invoking code" is a single category — code invocation IS
  the failure mode.
- **B**: Predicates that invoke code violate the verification
  discipline.
- **C**: All leaves can be pure-function evaluators over substrate
  state.
- **D**: External tools (clippy, kani, prusti, git binary) are not
  "code" in the relevant sense.

### Phase 3 — Stripping

- **Strip A**: "Invoking code" is one category.
  - Wrong: there are at least three distinct categories.
    1. **Author-defined code** (user-written Rust function passed as
       witness) — the trust-the-witness problem; this is what the
       closed grammar prevents.
    2. **Closed-set ecosystem tools** (cargo, git, clippy, kani,
       prusti) — these are bounded, externally specified, and the
       audit invokes them via known interfaces. NOT a trust-the-
       witness problem because the audit chooses which tool to
       invoke, not the witness author.
    3. **Closed-set audit primitives** (read JSON, parse syn,
       compute SHA, walk filesystem) — pure or near-pure operations
       the audit performs directly.
  - **Strip-A reveals**: the prohibition is specifically on
    category 1, not on 2 or 3.

- **Strip B**: Predicates that invoke code violate verification.
  - Wrong if we accept Strip A. The existing witness families
    already invoke code:
    - `test_fn` witness → `cargo test` invokes the test (category 2)
    - `clippy::lint` witness → `cargo clippy` invokes the lint
    - `kani::proof` witness → `cargo kani` invokes the prover
  - These invocations are deferred to A4-A5 (per ADR-005 Am 3's
    "test-attribute-present-not-invoked" hint), but they're
    structurally fine. The audit invokes the tool via a known
    interface; the tool's output is the verification evidence.
  - **The closed-set-tool invocation is consistent with tier-
    honesty** — the audit reports the work it did + the tool's
    response. Tier-honesty doesn't prohibit tool invocation; it
    prohibits over-reporting.

- **Strip C**: All leaves can be pure-function evaluators.
  - Mostly true for v0.1 leaves. `ratified_doc`, `signers`,
    `oracles_complete`, `fresh_within_days` are pure functions over
    on-disk state.
  - `signed_trailer` requires invoking `git interpret-trailers` or
    equivalent (per Attack-E from adversarial self-attack). That's
    a category-2 invocation. Pure-function-over-state is the
    workflow shape; the git binary is the parser.
  - Future leaves likely need category-2 invocations (e.g.,
    `tool_clean(cargo_audit)` to verify no advisories;
    `binary_passes(my_oracle_binary)` if oracle is a binary).
  - **Strip-C reveals**: leaves CAN invoke ecosystem tools; the
    constraint is that the tool is *chosen by the leaf author* (or
    by the audit), not by the witness use-site author.

- **Strip D**: External tools aren't "code" in the relevant sense.
  - True at the trust-boundary level: the audit's trust extension is
    over the tool's INTERFACE, not the tool's source. Tools have
    their own trust boundaries (their own tests, their own version
    discipline).
  - **Strip-D reveals**: trust is per-tool, extended at tool-adoption
    time; the discipline rhymes with the thymic-education / generator-
    trust pattern from v2's "What this is NOT" section.

### Phase 4 — Irreducible kernel (revised)

**The closed grammar prohibits *witness-author-defined code* (because
it re-introduces the trust-the-witness problem). It does NOT prohibit
*closed-set ecosystem tool invocation* by leaf primitives (which are
themselves bounded, audit-chosen, and tier-honest reportable).**

The self-pass condition "verifiable without invoking arbitrary code"
should be sharpened to "verifiable without invoking
witness-author-defined code." Leaves can invoke closed-set tools
(git, cargo audit, format validators) under the same tier-honesty
discipline (report what the tool actually verified).

### Phase 5 — Structurally forced conclusions

- The v0.1 leaf set's `signed_trailer` already implicitly invokes git
  (via parsing git-log output); this is OK and should be made
  explicit in the leaf's documentation.
- Future leaves MAY invoke closed-set tools. The constraint:
  - The tool must be deterministically chosen by the leaf
    (`signed_trailer` uses git; `cargo_audit_clean` uses
    `cargo audit`)
  - The tool must be specified in the leaf's contract (which tool,
    what version range, what output is consumed)
  - The audit must report tier-honestly what the tool verified
    (e.g., `cargo audit` returning clean = Execution-tier evidence
    for "no known advisories at audit time")
- The tool invocation is a per-leaf design choice, not a use-site
  escape valve. Use-site authors compose closed-grammar predicates;
  they do not author tool-invocation code.
- **Trust boundary**: each closed-set-tool leaf extends trust at
  *leaf-shipping-time*, not at use-site. Antigen's tool-leaf authors
  vet the tools; antigen ships the leaf; users use the leaf. Trust
  is extended where the leaf is defined, not where it's invoked.

### Phase 6 — Adjacency

- Adjacent to ADR-002 compose-don't-compete: leaves that invoke
  closed-set tools COMPOSE with those tools rather than reinventing
  their functionality.
- Adjacent to the thymic-education / generator-trust rhyme
  (from v1 turn 13 / v2 §"What this is NOT"): trust extended at
  adoption/integration time; leaf-shipping-time is the leaf's
  adoption point.
- Adjacent to ADR-007 anti-YAGNI: tool-invoking leaves are
  structurally required by use cases like `cargo audit` integration;
  reserve the pattern in v0.1 even if specific leaves ship later.

### Phase 7 — Extension predictions

- Witness-provider crates (Tier-3, deferred) can ship tool-invoking
  leaves; each leaf's contract specifies which tools, version
  range, output format.
- The audit's invocation framework needs to handle tool-not-found,
  tool-version-mismatch, tool-error in tier-honest ways (defer to
  v0.2+; not v0.1 scope).
- Naming convention emerges: `<tool>_<assertion>` (e.g.,
  `cargo_audit_clean`, `git_trailer_present`).

### Phase 8 — Verdict

**F4 (refines self-pass kernel of Principle 3)**: the "verifiable
without invoking arbitrary code" condition is too broad. The kernel
is sharpened to **"verifiable without invoking witness-author-defined
code; closed-set audit-chosen tool invocation is permitted under the
same tier-honesty discipline."**

This UNBLOCKS a class of future leaves the v2 draft implicitly
disallowed via the too-broad framing. It also makes explicit what
`signed_trailer` already does (invokes git). The closed-set is
extensible at adoption-time via witness-provider crates (per F4
contract specifications), not at use-site.

**v3 implication**: rewrite the predicate-language ceiling section to
distinguish (a) use-site sealing of grammar/combinators/leaf-set vs
(b) leaf-internal tool invocation under closed-set constraints. The
self-pass R-Ar3 (use-site sealed / adoption-time extensible) is
correct but doesn't cover this third layer (leaf-internal tool
invocation).

---

## F5 — The ratchet-asymmetry (R-Ar1) holds, but there's a "fingerprint-pin recovery" edge case the self-pass missed

### What self-pass claimed

R-Ar1 named the ratchet-asymmetry: "audit reports lower-bound;
promotions require evidence; downgrades are automatic when evidence
falters." Self-pass frontier question 5 asked whether the
automatic-downgrade has edge cases where it's wrong.

### Phase 1 — Visible claim

The ratchet IS the tier-honesty discipline. Promotions require new
evidence; demotions are immediate when evidence falters (e.g.,
`signed_against_fingerprint` stale → demote from Execution to
Reachability).

### Phase 2 — Assumptions

- **A**: Demotion is always the right answer when evidence falters.
- **B**: Evidence-faltering has a single mechanism (pin stale).
- **C**: Re-attestation is the only path to re-promote.
- **D**: The downgrade-and-re-attest cycle is desirable (forces
  re-engagement with the discipline).

### Phase 3 — Stripping

- **Strip A**: Demotion always right when evidence falters.
  - Edge case 1: alice signs against fingerprint X. Code refactor
    changes formatting but not semantics (rustfmt run; comment
    rewording; doc-comment addition). New fingerprint Y. Alice's
    signature pinned to X is now stale.
    - Is the discipline-judgment alice exercised against X still
      valid against Y? In the most common case (semantic preserving
      change), YES — alice's review of the function's discipline
      compliance doesn't lose validity because of whitespace.
    - But the audit cannot distinguish semantic-preserving vs
      semantic-changing refactors via SHA comparison alone.
    - **Strip-A reveals**: demotion is right *given the available
      evidence* (SHA differs → no proof of semantic equivalence),
      but it can produce false-positive demotions in the
      semantic-preserving case.

  - Edge case 2: the discipline doc was updated (new
    `min_version`), but alice's prior signature is still valid for
    the doc's actual content (the version bump was for clarification
    only). Pin reads `signed_against_fingerprint = X`
    (function-fingerprint); doc-ratification leaf checks
    `min_version` against current doc. If doc bumped from 1.0 to
    1.1 with no breaking changes, alice's signature against function
    X under doc 1.0 should still satisfy "predicate requires
    doc>=1.1."
    - The leaf evaluates against current doc state, not against the
      doc state at signing time. **Discipline gap**: signers don't
      pin to doc version, only to function fingerprint.

- **Strip B**: Single faltering mechanism (pin stale).
  - Multiple mechanisms:
    1. Function fingerprint changed (the v2 case)
    2. Doc version bumped past pin (no current pin to doc!)
    3. Oracle file marked incomplete (status flipped)
    4. Signer left (revocation; not in v2)
    5. Discipline doc deprecated (not in v2)
    6. Freshness expired (date check)
  - v2 mostly addresses (1) and (6); (2)-(5) need predicate-language
    or schema support.

- **Strip C**: Re-attestation only re-promotes.
  - Alternative: explicit "this fingerprint change is semantic-
    preserving; signature carries forward" — a "blessing" command
    (e.g., `cargo antigen attest bless --as alice
    --carry-forward-from X`). Distinct from re-signing; alice
    asserts "I reviewed the diff and it preserves compliance."
  - This re-introduces a flexibility surface that could be abused.
    But it's a real workflow need (rustfmt runs shouldn't require
    full re-review).

- **Strip D**: Re-attest cycle desirable.
  - For semantic changes: YES, re-engagement is the discipline
    working as intended.
  - For non-semantic changes (formatting, doc-comment additions):
    re-engagement is friction without value. Adoption killer.

### Phase 4 — Irreducible kernel

**The ratchet-asymmetry kernel HOLDS: the audit must report what's
verified, and currency-failure means lower-tier reporting. BUT the
mechanism for "what counts as evidence-faltering" is sharper than
v2/self-pass acknowledged:**

- Fingerprint mismatch is *one* faltering mechanism, not the only one
- Function-fingerprint-pin is *one* pin shape; doc-version-pin and
  others are absent from v2
- "Carry forward" (alice blesses a semantic-preserving refactor)
  is a legitimate workflow that doesn't violate the ratchet IF
  alice's blessing IS itself recorded as new evidence (a new
  signature with `against = new_fingerprint, basis =
  "carry_forward_from X with review of diff"`).

The ratchet remains symmetric: every promotion records new evidence.
The "carry forward" case records a different KIND of evidence (alice
reviewed diff Y-X) than a fresh full-review (alice reviewed function
Y). Both promote; both have recorded basis.

### Phase 5 — Structurally forced conclusions

- **The `Signer` schema needs a `basis` field** (or similar) that
  records what alice attested to:
  - `full_review` — alice reviewed the function against the
    discipline doc
  - `carry_forward_from(X)` — alice reviewed the diff from X to
    current and asserts it preserves compliance
  - `doc_version_carry_forward(V_old, V_new)` — alice attested
    the doc's V_old→V_new bump preserves the compliance she signed
    for
- **The default for `attest sign` should remain `full_review`**;
  `--carry-forward-from X` is an explicit alternative requiring the
  prior fingerprint.
- **Doc-version pin**: `Signer` should also carry the doc version
  alice signed against (not just the function fingerprint), so doc
  bumps can be tier-honestly assessed.
- **Self-pass R-Ar1 was directionally right but missed the
  pin-multiplicity dimension** — the ratchet covers multiple pins,
  not just function fingerprint.

### Phase 6 — Adjacency

- Adjacent to R-A3 (`against = "current"|"any"`): the `against`
  parameter captures whether the predicate cares about
  function-fingerprint-currency. Need parallel parameters for
  doc-version-currency, oracle-completion-currency, freshness-
  currency.
- Adjacent to the affinity-maturation biology rhyme: germinal-center
  refinement doesn't necessarily replace prior antibody fully;
  partial-affinity-preserving mutations exist. The "carry forward"
  workflow rhymes with affinity-maturation-with-some-prior-binding-
  preserved, not just full replacement.

### Phase 7 — Extension predictions

- Future pins: oracle-state-pin, dep-version-pin (for cross-crate
  cases), workspace-state-pin (for workspace-scope disciplines per
  F3). Each is a pin dimension; each can stale; each demotes when
  stale.
- The `Signer.basis` field becomes the central record of WHAT alice
  attested to — the more pins, the more bases.
- An adversarial pass should attack: can `carry_forward_from X` be
  used to launder bad changes? Mitigation: `attest sign --carry-
  forward-from X` requires git diff to show ONLY changes alice
  asserts are compliance-preserving; refuses if diff is empty or
  too large (>N lines per discipline rule).

### Phase 8 — Verdict

**F5 (extends and refines R-Ar1)**: the ratchet-asymmetry kernel
holds — promotions need recorded evidence; demotions auto-apply when
evidence falters. The refinement is **what counts as evidence is
multi-dimensional**: function fingerprint, doc version, oracle state,
freshness date, and basis-of-signature (full-review vs carry-forward
vs doc-version-carry-forward).

**v3 implication**: schema needs `Signer.basis` field; predicate
language needs per-leaf `against` parameters (or a unified pin-currency
mechanism); CLI needs `--carry-forward-from X` flag on `attest sign`
with structured restrictions (diff-size, time-since-last-sign).

**Self-pass missed**: the multiplicity of pin dimensions and the
workflow need for non-semantic-change re-promotion without full
re-review. Both surface from Phase 1-8 stripping of the
single-faltering-mechanism assumption.

---

## F6 — The audit-of-audit recursion IS bounded; the structure that bounds it is the tier collapse from FormalProof down to Reachability + audit_hint

### What self-pass claimed

Self-pass Principle 1 (tier-honesty) noted: "the audit's own
implementation must be itself tested — atk_w7 contracts exist for this.
Structurally necessary; meta-recursion (audit-of-audit)." Frontier
question 6 asked whether the recursion is bounded or whether there's
an infinite regress (the audit's tests need their own audit-honesty,
which needs tests, which need...).

### Phase 1 — Visible claim

Tier-honesty applies to the audit. The audit's own tests must be
themselves verified to be tier-honest. Those tests' verifiers must
be verified. Infinite regress?

### Phase 2 — Assumptions

- **A**: Tier-honesty is meaningful only when applied transitively.
- **B**: Verification of N requires verification of N+1, ad infinitum.
- **C**: The chain has no natural stopping point.
- **D**: Other verification systems don't have this problem (or
  they do and we should look at how they resolve it).

### Phase 3 — Stripping

- **Strip A**: Transitivity.
  - Without transitivity, the audit can be lying and tier-honesty
    is just a claim, not a verifiable property. But the assertion
    "audit reports honestly" is a *first-order* claim about the
    audit's behavior; it can be verified empirically (run audit on
    test fixtures; check reported tier against known-correct
    answer). The verification of "the test fixtures are correct"
    is a *human judgment* (the test author looked at the antigen
    declaration and the witness function and reasoned about what
    tier the audit should report).
  - **Strip-A reveals**: the chain stops at human judgment. Test
    authors make ground-truth claims; we trust their judgment under
    a different discipline (code review, PR sign-off, multiple
    reviewers).

- **Strip B**: Verification of N requires verification of N+1.
  - This is true mechanically (the verifier itself can be
    incorrect) but practically bounded by:
    1. Verifier-of-verifier becomes increasingly simple at each
       level (eventually trivial)
    2. Verification reduces to comparison-against-ground-truth-claim
       (a human-judgment claim)
    3. The ground-truth claim is itself subject to discipline (PR
       review, multiple reviewers, established practice)
  - This is the classic verification regress in software engineering;
    it's bounded by tooling-trust-extension (we trust `rustc`'s
    type-checker; we trust `cargo test`'s pass/fail report; we
    don't audit those at each level).
  - **Strip-B reveals**: the regress is mechanically infinite but
    practically bounded by accepted-trust-boundaries.

- **Strip C**: No natural stopping point.
  - The natural stopping point is **the tier collapse**: at each
    level of the regress, the EVIDENCE KIND collapses. The
    top-level audit reports tier-Execution against substrate it
    evaluated. The audit's tests verify the audit reports correctly
    — but the tests are at a *lower* evidence-kind tier: they are
    behavioral evidence (cargo test ran them and they passed).
    Verifying-the-tests-themselves would have to be at YET LOWER
    evidence-kind tier (manual review = code-review-tier; not even
    Reachability).
  - **The regress is bounded because evidence-kind monotonically
    decreases**: Execution → Behavioral-via-test → Code-review →
    Human-judgment. At Human-judgment, the verification chain
    naturally terminates because there's no mechanical verification
    below human judgment in software engineering practice.
  - **Strip-C reveals**: the chain HAS a natural stopping point —
    where mechanical verification ends and human-judgment / PR-
    review-discipline begins.

- **Strip D**: Other verification systems.
  - Type systems: trusted because compiler is widely-used,
    well-tested, formally verified for some subsets. Trust-extension
    at compiler-adoption.
  - Test frameworks: trusted because the framework's tests are run,
    framework is widely-used. Trust-extension at framework-adoption.
  - Formal verifiers (kani, prusti): trusted because verification
    algorithm is published, peer-reviewed, implementation is
    open-source. Trust-extension at adoption.
  - All have the bounded-by-adoption-trust pattern. Antigen's audit
    has the same shape.

### Phase 4 — Irreducible kernel

**Audit-of-audit recursion is bounded because evidence-kind
monotonically decreases at each level until it reaches
human-judgment-via-established-discipline (PR review, multiple
maintainers, etc.). The chain naturally terminates at the boundary
where mechanical verification ends.**

The pattern is the same as every other verification system in
software engineering: trust-extension at adoption boundary, then
verification within the adopted tool's domain. The audit is no
exception. The audit's own tests run under cargo test
(adoption-trust); cargo test's results are trusted (cargo-adoption-
trust); rustc's compilation is trusted (rustc-adoption-trust);
human review of the audit's test claims is trusted (peer-review
discipline).

### Phase 5 — Structurally forced conclusions

- The audit IS tested via cargo test + adversarial test fixtures
  (atk_a2_adversarial.rs); this is the bottom of the formal
  verification chain.
- Tests assert known-correct tier answers; their correctness is
  human-judgment + peer-review (project discipline).
- "Audit-of-audit" is therefore NOT an unbounded regress; it's
  bounded at the cargo-test + peer-review layer, like every other
  verifier in Rust ecosystem.
- v3 should NAME this boundary explicitly to prevent future
  confusion: tier-honesty applies transitively WITHIN the formal
  verification chain; the chain bottoms out at human-judgment-via-
  established-discipline, which is the inherited trust-boundary of
  the whole development practice.

### Phase 6 — Adjacency

- Adjacent to the thymic-education / generator-trust pattern: trust
  extended at adoption boundary; per-use re-checking is not the
  discipline.
- Adjacent to peripheral-tolerance rhyme (per naturalist R-N5):
  ongoing audit catches regressions; doesn't re-derive trust from
  scratch each time.
- Adjacent to ADR-005 sub-clause F: every trust boundary requires
  validation; the cargo-test / peer-review boundary is one such
  boundary, validated by project process (CI runs tests; PRs are
  reviewed; multiple maintainers required for landing).

### Phase 7 — Extension predictions

- The same bounded-regress structure applies to any future audit
  mechanism (cross-crate witness audit, substrate-witness audit,
  workspace-scope-audit). Each terminates at cargo-test + peer-review.
- The discipline antigen "audit's tests are reviewed by maintainer
  team and pass cargo test" could itself be a discipline-antigen
  (recursive!) — `#[immune(AuditCorrectness, witness = ...)]` on
  the audit module itself. But this is decorative; the practical
  trust extension is at cargo-test + peer-review.

### Phase 8 — Verdict

**F6 (resolves self-pass frontier question 6)**: the audit-of-audit
recursion IS bounded. Bounded by evidence-kind monotonic decrease
plus the inherited adoption-trust of cargo-test + peer-review.

The bounding structure is the same as every verifier in Rust
ecosystem: trust extended at adoption-boundary; within-adopted-tool
verification proceeds normally. Antigen is not special; it inherits
the standard discipline.

**v3 implication**: explicitly name the bounded-regress structure in
ADR-019 (or as an amendment to ADR-005 Am 3): tier-honesty is
transitively applied within the formal chain; chain bottoms out at
the project's established review discipline (which is itself a
discipline-antigen at the meta-level, but not recursively in a
problematic way).

---

## F7 — NEW kernel surfaced: the WITNESS-PROVIDER-CRATE trust boundary is currently UNSPECIFIED and would re-introduce the trust-the-witness problem if shipped without sub-clause F

### What self-pass missed

The self-pass treated Tier-3 witness-provider crates as a deferred
extension point (per v2's open-questions resolutions). It did not
deconstruct what HAPPENS structurally when third-party crates ship
new leaf primitives. R-Ar3 named "use-site-sealed / adoption-time-
extensible" but did not push on what the adoption-time extension
mechanism MUST look like to preserve tier-honesty.

This is a NEW frontier surface, not present in any prior pass.

### Phase 1 — Visible claim

Tier-3 ambition: `antigen-witnesses-research-rigor` ships
`peer_reviewed_and_replicated` leaf; user's antigen.toml lists it as
dep; user's `#[immune]` macros can use it. Audit walks dep graph;
discovers + invokes the leaf.

### Phase 2 — Assumptions

- **A**: Third-party leaves can be safely invoked by antigen's audit.
- **B**: Tier-honesty of the audit's report is preserved across
  third-party leaf use.
- **C**: The leaf author's reported tier IS the tier the leaf
  earned.
- **D**: The audit's trust extension to the leaf is appropriate.

### Phase 3 — Stripping

- **Strip A**: Safe invocation.
  - Third-party code invoked at audit-time is, by definition, code
    the audit author didn't write. If the leaf misbehaves (panics,
    hangs, lies), the audit's tier-honesty surface is corrupted.
  - The mechanism that would prevent this: the leaf trait is
    sealed/structured/sandboxed somehow. v2 doesn't specify.
  - **Strip-A reveals**: ungoverned leaf invocation = unbounded
    risk.

- **Strip B**: Tier-honesty preserved across third-party.
  - Wrong unless the audit imposes contract-checking on the leaf:
    - Leaf must declare what tier its result earns (e.g., "this
      leaf earns Execution-tier evidence when it returns true")
    - Leaf must be deterministic for fixed-input substrate
    - Leaf must terminate within bounded time/memory
    - Leaf must not have side effects beyond its declared substrate
      reads
  - Without contract-checking, the audit's tier-honesty surface is
    only as honest as the most-honest third-party leaf author.
  - **Strip-B reveals**: tier-honesty REQUIRES contract enforcement
    at leaf-invocation boundary — sub-clause F for leaf calls.

- **Strip C**: Author-reported tier IS earned tier.
  - This is exactly the trust-the-witness problem at the leaf
    level. A malicious or careless leaf author could ship a leaf
    that returns `true` for all inputs and declares it earns
    Execution-tier. The audit ships the over-claim.
  - The mechanism that prevents this:
    - Audit checks the leaf's declared tier against the audit's
      ability to verify the leaf's claim (meta-circular; doesn't
      work in general)
    - Audit caps third-party leaves at a lower tier (Reachability
      + `third-party-leaf-not-validated` hint) by default
    - Project explicitly opts in to higher-tier third-party leaves
      via a workspace-config trust list
  - **Strip-C reveals**: the structurally sound answer is the third
    option — explicit trust opt-in at workspace-config level.

- **Strip D**: Audit trust extension appropriate.
  - Audit extends trust to the leaf because the workspace included
    its parent crate as a dep. Trust extension at dep-adoption,
    rhymes with thymic-education pattern.
  - But adding a leaf-providing crate to deps should not silently
    grant Execution-tier reporting for that leaf's outputs. The
    trust extension should be EXPLICIT — workspace config names
    which third-party leaves are trusted to which tier.
  - **Strip-D reveals**: trust extension to third-party leaves
    requires explicit workspace-config opt-in (similar to
    cargo-audit's `allow` list for advisories), not implicit
    dep-adoption.

### Phase 4 — Irreducible kernel

**Third-party leaf primitives re-introduce the trust-the-witness
problem at the leaf level. The mitigation must be (a) leaf-contract
specification (deterministic, terminating, side-effect-free, declared
tier), (b) default cap at Reachability for third-party leaves, (c)
explicit workspace-config opt-in for higher tiers per-leaf.**

Without these, the witness-provider-crate ambition is a
tier-honesty failure waiting to ship.

### Phase 5 — Structurally forced conclusions

- **ADR-019 must defer witness-provider crates explicitly** until
  the leaf-contract is specified (probably a separate ADR or
  amendment).
- **v0.1 ships sealed leaf set** (no third-party leaves) — per v2.
  This is now structurally required, not just conservative.
- **v0.2+ leaf-provider-crate ADR** must specify:
  - Leaf trait/contract (terminate, deterministic, side-effect-
    bounded, declared-tier)
  - Default tier cap at Reachability for non-opted-in leaves
  - Workspace-config mechanism for opt-in to higher tiers
  - Audit invocation framework (bounded-time, sandboxed if
    necessary)
- **The third-party leaf trust boundary is a sub-clause F surface**:
  every workspace's adoption of a leaf-provider crate must
  explicitly extend trust per the validation discipline.

### Phase 6 — Adjacency

- Adjacent to thymic-education trust-extension: but THERE'S A
  REFINEMENT — generator-trust extends to the generator's WHOLE
  output (you're using the generator FOR its output). Leaf-trust
  is FINER-GRAINED — the leaf is one of many; trusting one leaf
  doesn't transfer to others.
- Adjacent to ADR-005 sub-clause F: every trust boundary requires
  validation. The leaf-invocation boundary is one such; v0.1's
  sealed-leaf-set avoids the boundary entirely. v0.2+'s opt-in
  mechanism IS the validation.
- Adjacent to ADR-007 anti-YAGNI: the leaf-contract is
  structurally required by the leaf-provider ambition. Ship the
  contract spec when the ambition ships, not before.

### Phase 7 — Extension predictions

- A leaf-provider crate publishes leaves with contract metadata
  (declared tier, version range it's stable in, what substrate it
  reads).
- Workspace's `cargo-antigen.toml` opts in with per-leaf entries:
  `trusted_leaves = { "antigen-witnesses-research-rigor::peer_reviewed_and_replicated" = "Execution" }`.
- Audit refuses to report higher than the opted-in tier for that
  leaf.
- Naturalist rhyme prediction: this is the **vaccine-development
  workflow** — new vaccines (leaves) ship after trials (leaf author
  verification), then receive regulatory approval (opt-in by
  workspace) for use at a given population scope.

### Phase 8 — Verdict

**F7 (NEW kernel, not in self-pass)**: the witness-provider-crate
ambition re-introduces the trust-the-witness problem at the leaf
level. v0.1's sealed-leaf-set avoids the boundary; v0.2+'s extension
mechanism MUST specify leaf-contract + default-tier-cap + workspace-
config opt-in.

**v3 implication**: ADR-019 deferral of witness-provider crates is
not just "feature deferred" — it's "trust-boundary deferred pending
sub-clause F design." Flag the structural commitment explicitly:
v0.2+ leaf-provider ADR is on the critical path for Tier-3, not just
"we'll get to it."

**This is a kernel the self-pass did not surface** because the
self-pass treated Tier-3 as already-resolved-by-deferral. Phase 1-8
stripping of "third-party-leaf invocation is safe" reveals the gap.

---

## F8 — NEW kernel surfaced: the "EVIDENCE-KIND" axis (per naturalist R-N1) is more load-bearing than v2/self-pass acknowledged, and reframes WitnessTier itself

### What self-pass missed

Naturalist R-N1 reframed v2's B-cell-vs-T-cell tier-cap argument as
"evidence-kind" ceiling — three categorically different evidence
kinds, each with its own structural ceiling. Self-pass cited R-N1
but did NOT do Phase 1-8 on it. Doing so reveals the evidence-kind
axis is more load-bearing than v2 acknowledged.

### Phase 1 — Visible claim (sharpened from R-N1)

Three evidence kinds:
- **Type-system-proof** (phantom-type) → reaches FormalProof
- **Behavioral** (test/proptest, harness-executed) → reaches Execution
- **Substrate-state** (substrate-witness) → reaches Execution
- (Tests not yet run reach Reachability — incomplete behavioral)
- (External tool prefix recognized but not invoked → Reachability)

Each evidence kind has a structural ceiling determined by what the
verification produces.

### Phase 2 — Assumptions

- **A**: WitnessTier is a sufficient axis for tier-honesty reporting.
- **B**: Evidence-kind is downstream of WitnessTier (witness families
  carry tiers; evidence-kind is the witness family's nature).
- **C**: A single tier-axis is consumer-comfortable (Ord-able for CI
  gating).
- **D**: AuditHint sufficiently distinguishes evidence kinds within
  a tier.

### Phase 3 — Stripping

- **Strip A**: WitnessTier sufficient.
  - Consider: two witnesses report `Execution`. Witness 1 is a
    test-fn that was harness-invoked (behavioral evidence). Witness
    2 is a substrate-witness whose predicate-passes-and-current
    (substrate-state evidence).
  - Both report Execution; both are tier-honest. A CI gate that
    requires "at least Execution" passes both.
  - But a more discerning consumer (e.g., a regulatory body, a
    safety-critical reviewer) may want different thresholds for
    different evidence kinds: "behavioral OR formal-proof at
    Execution+, accept; substrate-state at any tier, REJECT (must
    have behavioral covering it)."
  - WitnessTier-alone cannot express this. AuditHint can carry the
    distinction but it's not Ord-able; CI gates that distinguish
    evidence-kind would need to parse hints.
  - **Strip-A reveals**: WitnessTier alone is sufficient for
    *coarse* gating, but insufficient for evidence-kind-aware
    gating. The current AuditHint mechanism is a workaround.

- **Strip B**: Evidence-kind is downstream.
  - Wrong: evidence-kind is upstream. The witness family chooses its
    evidence kind by what verification it can structurally produce
    (phantom-type → type-system-proof; test-fn → behavioral;
    substrate-witness → substrate-state). The TIER is downstream of
    the evidence kind (each kind has its ceiling).
  - **Strip-B reveals**: evidence-kind is a FIRST-CLASS axis upstream
    of WitnessTier.

- **Strip C**: Single tier-axis consumer-comfortable.
  - True for v0.1 — most consumers want a single comparable score.
  - But the more general framing is: **two-axis tier reporting**
    (evidence-kind × tier-of-evidence-kind). A consumer can choose
    to filter on evidence-kind first, then tier within kind.
  - **Strip-C reveals**: single-axis is contingently sufficient
    today but the more honest model is two-axis.

- **Strip D**: AuditHint sufficient for kind distinction.
  - Current AuditHint values are domain-specific (`function-resolves`,
    `phantom-type-shape-recognized`, etc.) — they don't enumerate
    evidence-kinds as a structured field.
  - To make evidence-kind a structured axis, it should be a
    first-class field, not an inferred-from-hint property.
  - **Strip-D reveals**: AuditHint can carry the info but doesn't
    structure it as a kind-axis.

### Phase 4 — Irreducible kernel

**Tier-honesty has TWO axes: evidence-kind and tier-within-kind.
WitnessTier currently flattens both into one ordinal scale. The
flattening works for coarse CI gating but loses fidelity for
discerning consumers (regulatory, safety-critical, multi-evidence
disciplines).**

The honest model: each witness emits `evidence_kind: TypeSystemProof
| Behavioral | SubstrateState` AND `tier_within_kind: None |
Reachability | Execution | FormalProof` (where the upper bound is
kind-dependent: TypeSystemProof can reach FormalProof; Behavioral
and SubstrateState cap at Execution).

The current single-WitnessTier-axis is a *projection* of the
two-axis model that loses the kind dimension.

### Phase 5 — Structurally forced conclusions

- **v0.1 should ship `evidence_kind` as a first-class field on
  audit output** parallel to `witness_tier` and `audit_hint`.
- The `witness_tier` field becomes the projection-onto-single-axis
  (for backward compat + coarse CI gating).
- The `evidence_kind` field is new and supports evidence-kind-aware
  gating.
- The closed enum `EvidenceKind = TypeSystemProof | Behavioral |
  SubstrateState` is locked at this level; future evidence kinds
  (e.g., `Simulation`, `MLEval`) added via amendment if needed.
- **This is a refinement of ADR-005 Am 3, not just ADR-019**:
  Am 3 introduced AuditHint as parallel axis to WitnessTier;
  ADR-019 (or an amendment) adds EvidenceKind as a third parallel
  axis.

### Phase 6 — Adjacency

- Adjacent to naturalist R-N1 biology rhyme (innate vs adaptive
  immunity at machinery level): EvidenceKind axis is the
  software-engineering analog of immune-system-level distinctions
  (germline-encoded vs somatic-encoded; structural vs behavioral
  vs substrate-mediated).
- Adjacent to ADR-005 Am 3 OQ-1 (tier-honesty for non-audit
  mechanisms): if EvidenceKind becomes first-class, the discipline
  extension is sharper (tier-honesty *per evidence kind* in
  non-audit contexts).
- Adjacent to F1 (discipline-level unification of substrate-witness
  + cross-crate-witness): the unification is at the *discipline*
  level; EvidenceKind makes the discipline crisper.

### Phase 7 — Extension predictions

- Future evidence kinds will add to the EvidenceKind enum.
  Candidates: `Simulation` (audit reads sim results), `MLEval`
  (audit reads ML eval scores), `FormalSpec` (audit reads
  prusti/verus annotation that's verified externally).
- Each evidence kind has its own ceiling. Document the ceiling per
  kind.
- Audit's reporting surface becomes:
  ```
  witness_tier: Execution
  audit_hint: discipline-substrate-validated-and-current
  evidence_kind: SubstrateState
  ```
- Consumers can gate on any combination.

### Phase 8 — Verdict

**F8 (NEW kernel, extending naturalist R-N1 via Phase 1-8)**:
EvidenceKind is a first-class axis parallel to WitnessTier and
AuditHint. The self-pass cited R-N1 but didn't surface that R-N1
implies a *new structured axis*, not just a *reframing*. Phase 1-8
on R-N1 reveals the axis.

**v3 implication**: ADR-019 (or sibling amendment to ADR-005 Am 3)
adds `EvidenceKind` enum + `evidence_kind` JSON field on audit
output. v0.1 ships three kinds (TypeSystemProof, Behavioral,
SubstrateState). The single-WitnessTier axis remains as backward-
compat projection; new field is parallel.

**Self-pass missed** because R-N1's framing-shift was treated as
"better metaphor" rather than "structural axis surfaced." Phase 1-8
stripping reveals the axis was always implicit; making it explicit
is the elevation move per ADR-004.

---

## F9 — Frontier kernel: the `discipline_doc` field is doing TWO jobs that should be separated (recommended for discussion, not Phase 1-8'd to closure)

### What I'm flagging

v2 §"Load-bearing item 3" introduces `discipline_doc` as optional
default field on antigen declaration. The self-pass Phase 1-8'd it
under Principle 4. But there's a structural sub-claim that's worth
surfacing for a future pass:

- **Job 1**: `discipline_doc` declares the *canonical reference*
  the discipline is grounded in (used by `ratified_doc` leaf as
  default path).
- **Job 2**: `discipline_doc` implicitly *binds* code-presenting-
  sites to that doc (every `#[immune]` for this antigen presumes
  doc-grounded review).

These are subtly different. Job 1 is a declarative-default
mechanism. Job 2 is a normative claim about review discipline.

A future pass should consider: should there be a separate
`canonical_reference: PathBuf` field (Job 1) and a separate
`review_grounded: bool` field (Job 2), so the two claims can vary
independently?

Not Phase 1-8'd here because I don't have enough substrate to
strip the assumption set without more evidence from real adoption.
Flagged as **frontier-question for next-pass** rather than F-finding.

---

## What I ratify from self-pass, with my own reasoning

I want to be explicit about which self-pass findings I'm absorbing as
ratified vs. replacing:

**R-Ar1 (ratchet-asymmetry)**: my-reasoning-ratifies AND extends via
F5 (multi-pin dimensions + carry-forward workflow). Self-pass
direction is right; F5 sharpens what counts as evidence and what
counts as faltering.

**R-Ar2 (other-substrate reframe)**: my-reasoning-PARTIALLY-ratifies
via F1 (discipline-level unification holds; machinery-level
unification does not). Self-pass overshot on scope.

**R-Ar3 (use-site sealed / adoption-time extensible)**: my-reasoning-
ratifies AND extends via F4 (third layer — leaf-internal closed-set
tool invocation is permitted) AND F7 (adoption-time extension
requires sub-clause F validation).

**R-Ar4 (doc-level discipline ratification)**: my-reasoning-REPLACES
via F2 (absorbed into existing primitive; not parallel). Self-pass
overshot.

**R-A1 (tier framing "fraction of work")**: absorbed by F8 (sharper
framing via EvidenceKind axis).

**R-A2 (every tier verifies structure-of-evidence, not truth)**:
ratified; orthogonal to F-findings.

**R-A3 (`signers(against = "current"|"any")`)**: ratified AND
extended by F5 (multi-pin dimensions need per-pin `against` params).

**R-A4 (only `attest sign` writes fingerprint)**: ratified.

**R-A5 (`signature_strength` field)**: ratified; orthogonal to F8's
EvidenceKind axis (signature_strength is sub-attribute of evidence;
EvidenceKind is which kind of evidence overall).

**R-A6 (schema rejects zero-leaf compositions)**: ratified.

**R-A7 (per-consumer ratification for cross-crate)**: ratified.

**R-A8 (macro-invocation site is input layer)**: ratified.

**R-A9 (granularity follows presentation)**: ratified AND extended
by F3 (scope is first-class dimension; granularity is a consequence,
not the primary commitment).

**R-N1..N7 (naturalist refinements)**: R-N1 absorbed and extended
by F8; R-N2 through R-N7 are biology-grounding and not directly
attacked by Phase 1-8 — defer to team naturalist's follow-up.

---

## Frontier questions for the next team passes

### For team-adversarial (next-pass)

- **F7 attack**: can a malicious leaf-provider crate compromise
  audit honesty even with the v0.2+ contract spec from F7? What
  enforcement mechanism is actually robust (sandboxing? compile-
  time checks? runtime bounds?)?
- **F5 attack**: can `--carry-forward-from X` be abused to launder
  changes? What restrictions (diff-size cap? time-cap? required
  doc-reference?) make it tier-honest?
- **F8 attack**: does the EvidenceKind axis introduce a new
  surface where compound-evidence claims can be over-reported?
  E.g., a discipline-antigen with BOTH a behavioral test AND
  substrate signatures — does reporting both kinds of evidence
  create a false "stronger together" impression?
- **F1 attack**: with the machinery-level non-unification kept,
  what's the failure mode if a future maintainer mistakes the
  discipline-level unification for machinery-level unification and
  tries to share parser code?

### For team-naturalist (next-pass)

- **F8 deepening**: does the EvidenceKind axis have a deeper
  biology rhyme than R-N1's innate-vs-adaptive (which was at the
  immune-system level)? E.g., does evidence-kind rhyme with
  modality-of-sensory-perception (vision vs touch vs
  proprioception)? Cross-domain extension.
- **F3 deepening**: discipline-scope as a first-class dimension —
  does biology have a scope-axis for immune-discipline? Cellular
  vs tissue vs organ vs systemic immunity is one candidate.
  Validate.
- **F7 vaccine-development rhyme**: does this hold up against
  actual immunology research? Vaccine adoption / population-scope
  approval pattern.

### For team-academic-researcher

- **F4 closed-set tool invocation**: how do other verification
  frameworks (cargo-audit, SLSA, in-toto, Sigstore) handle the
  closed-set-tool-invocation question? Are there established
  patterns for "third-party leaf with declared contract"?
- **F8 multi-axis tier reporting**: how do other code-attestation
  systems report multi-kind evidence (cargo-audit reports
  advisories with severity + status + version-range; SLSA
  reports build provenance with attestation predicates)? Do they
  use single-axis or multi-axis?
- **F7 leaf-contract design**: are there published trait-contract
  designs for sandboxed plugin systems (WASM modules, Lua scripts
  in databases, NPM plugins) that inform the leaf-contract
  shape?

### For team-scout

- **F2 absorption pattern**: this pattern (typed JSON sidecar
  carrying typed claim about adjacent substrate; substrate is any
  file the audit can read) GENERALIZES well beyond
  discipline-witnesses. Where else in antigen does this pattern
  apply? Could replace several special-case mechanisms with the
  unified sidecar-substrate pattern.
- **F3 scope-as-first-class**: are there OTHER first-class
  dimensions of antigens that v2 implicitly assumes uniform but
  should be explicit? Lifetime-scope? Severity-class?
- **F8 EvidenceKind reach**: where else in antigen does
  evidence-kind matter beyond witnesses? Does the antigen
  declaration itself have an evidence-kind (e.g., "this antigen
  is documented based on N observed failure instances vs based on
  theoretical prediction")?

### For team-aristotle next-pass (after team incorporates F-findings)

- **F9 surfacing**: does `discipline_doc` doing two jobs need
  separation? After more adoption substrate accumulates.
- **The EvidenceKind enum closure**: is `TypeSystemProof |
  Behavioral | SubstrateState` actually exhaustive? Or are there
  evidence kinds I haven't enumerated?
- **The doc-level absorption vs scan-walks-`.md` decision (F2)**:
  if a v0.2 use case demands doc-side antigen presentation, what
  exactly triggers the upgrade?

---

## What doesn't change

Despite F1-F8 surfacing significant refinements:

- The substrate-witness reframe itself (witnesses over non-code
  substrate) survives.
- The three-coupled-piece shape (predicate language + Ratification
  schema + CLI) survives.
- The ONE-ADR position survives — the F-findings sharpen what's IN
  ADR-019, not split it.
- Code-locality as default survives (per Principle 4 / F3 — it's
  the most common scope, just not the only scope).
- Closed combinator grammar at use-site survives (per F4 — sharpened
  to permit leaf-internal closed-set tool invocation).
- The tier-honesty discipline survives and is strengthened (per F6
  bounded-regress + F8 evidence-kind axis).
- Per-consumer ratification for cross-crate (R-A7) survives intact.

The F-findings are sharpenings, gap-fillings, and new-axis-surfacings —
not invalidations of the core shape.

---

## Summary of recommended v3 changes

In priority order (highest-impact first):

1. **F8 — Add EvidenceKind as first-class axis** parallel to
   WitnessTier and AuditHint. v0.1 ships three kinds (TypeSystemProof,
   Behavioral, SubstrateState). New field on audit output.
2. **F3 — Add `scope:` field to discipline-antigen declarations**.
   v0.1 ships site-scope and file-scope; coarser scopes when
   fingerprint mechanisms ratify.
3. **F1 — Name the discipline-level vs machinery-level unification
   asymmetry**. Substrate-witnesses + cross-crate witnesses share
   tier-honesty discipline; do NOT share recognition machinery.
   Cite explicitly in ADR-019.
4. **F2 — Absorb doc-level ratification into existing primitive**
   via extended `ratified_doc` leaf with optional sibling JSON.
   Drop R-Ar4's "parallel primitive" framing.
5. **F4 — Refine predicate-language ceiling** to permit
   leaf-internal closed-set tool invocation under tier-honesty
   discipline. Make explicit what `signed_trailer` already does.
6. **F5 — Add `Signer.basis` field + multi-pin `against` parameters**
   to support carry-forward workflow without losing ratchet.
7. **F7 — Defer witness-provider-crate trust boundary to v0.2+
   ADR** with explicit scope: leaf-contract, default-tier-cap,
   workspace-opt-in. Name this on the critical path, not
   "deferred extension."
8. **F6 — Name the audit-of-audit bounded-regress structure**
   explicitly in ADR-019 (or amendment to Am 3).

All eight changes are absorbable into the ONE-ADR position (some
land in ADR-019 directly; some land as sibling amendments to
ADR-005 Am 3). None forces a re-split.

---

READY FOR REVIEW

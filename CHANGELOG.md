# Changelog

All notable changes to the antigen project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — `body_calls("<name>")` fingerprint leaf (ADR-040 grammar increment 1)

- New fingerprint operator `body_calls("<name>")` — the call-shaped twin of
  `body_contains_macro`. Matches a function/method body that *calls* the named
  function or method, in both shapes Rust spells calls: free/path calls
  (`foo()`, `std::process::exit(1)` — matched on the callee path's **last
  segment**) and method calls (`x.unwrap()`, `r.expect(..)` — matched on the
  **method identifier**). Same partial domain as the macro twin: definite
  Match/NoMatch for bodied items (`fn`, `impl` methods), `Undefined` for
  bodyless item-classes (so `not(body_calls(X))` inside `all_of` stays sound,
  ADR-010 Amd6). Closes the silent `.unwrap()`/`.expect()` gap a macro-only
  match misses (e.g. the `PanickingInDrop` fingerprint).

### Added — `is_async` / `is_unsafe` / `is_const` qualifier leaves (ADR-040 grammar increment 2, G1)

- New value-less fingerprint operators `is_async`, `is_unsafe`, `is_const` —
  item-qualifier presence checks. `is_async` / `is_const` read a function's
  `async` / `const` qualifier (`fn` locus); `is_unsafe` reads `unsafe` on both
  loci that carry it (`unsafe fn` and `unsafe impl`). Partial-domain like the
  body leaves: `Undefined` on item-classes with no locus for the qualifier (e.g.
  `is_async` on a `struct`), so `not(is_async)` stays sound inside `all_of`
  (ADR-010 Amd6) — never a vacuous match. These unblock the call/qualifier
  family members (e.g. `all_of([is_async, body_calls(...)])` for
  `BlockingCallInAsyncFn`, `all_of([item = fn, not(is_unsafe)])` for
  `RawPtrDerefInSafeFn`).

### Added — `derives("<name>")` / `serde_arg("<name>")` attribute-introspection leaves (ADR-040 grammar increment 2, G1b)

- New fingerprint operators `derives("<name>")` (is `name` in a `#[derive(...)]`
  list on the item) and `serde_arg("<name>")` (is `name` an argument in a
  `#[serde(...)]` attribute, e.g. `deny_unknown_fields` — matched whether bare or
  `= value`). Both are syntactic last-ident membership (no path resolution — a
  user type also named `Hash` is indistinguishable here, the honest false-positive
  the confidence dial carries) and full-domain like `attr_present` (absent =
  definite `NoMatch`), so the anchored absence form is the tell:
  `all_of([item = struct, derives("Hash"), not(derives("Eq"))])` for the
  `derive(Hash)`-without-`Eq` class;
  `all_of([derives("Deserialize"), not(serde_arg("deny_unknown_fields"))])` for
  `DeserializeWithoutDenyUnknownFields`. (`attr_absent` is not a new operator —
  the anchored `not(attr_present(...))` already expresses attribute absence.)

### Added — `impl_of_trait("<name>")` trait-impl identity leaf (ADR-040 grammar increment 2, G3)

- New fingerprint operator `impl_of_trait("<name>")` — matches an
  `impl <Trait> for <Type>` whose trait path's last segment equals `name`
  (syntactic last-segment, so `impl std::ops::Drop for V` matches
  `impl_of_trait("Drop")`). An inherent `impl V {}` is a definite `NoMatch`;
  a non-`impl` item is `Undefined` (partial domain — `not(impl_of_trait(X))`
  stays sound). Reads ONE impl item's own trait path; the cross-item "does
  `Type` impl X *anywhere*" question is a separate (G4 / charter) concern. This
  lets a fingerprint assert an impl is *actually* the trait it claims — e.g.
  `all_of([item = impl, impl_of_trait("Drop")])` distinguishes a real `Drop`
  impl from an inherent impl with a method merely *named* `drop` (which the
  shipped `PanickingInDrop` fingerprint cannot), and `UnsafeSendSync` anchors on
  `impl_of_trait("Send")` + `is_unsafe`.

### Changed — `body_contains_macro` / `body_calls` now reject unmatchable names (fail-direction fix)

- **Behavior change (tiny compat surface; surfaced here per our own
  practice-what-we-preach discipline).** Both the call/macro-target leaves now
  **reject at parse time** a name that is not a single bare identifier — a
  path-spelled (`"std::process::exit"`), `!`-bearing (`"panic!"`), dotted
  (`".unwrap"`), parenthesized (`"unwrap()"`), or whitespace-padded
  (`" unwrap"`) name — with a helpful message naming the fix. Previously such a
  name *parsed OK and silently matched nothing* — a **named-but-silent
  false-coverage miss**, the exact failure-class antigen exists to surface,
  found in antigen's own keystone grammar leaf by the tests-first pass. The fix
  is a single shared `validate_target_ident_name` gate both leaves route
  through (DRY). Every real fingerprint already uses bare names (`"panic"`,
  `"unreachable"`, `"todo"`, `"unimplemented"`, …), so the compat surface is
  empty in practice; a fingerprint that *relied on* the silent miss was already
  a no-op. Migration: use the bare name (`body_calls("exit")`, not
  `body_calls("std::process::exit")`).

## [0.3.0-beta.1] — 2026-06-01

_First public v0.3 prerelease. The v0.3 surface is the prescriptive/work-orchestration
family, the titer/scalar witness kind, the live-projection reporting model, multi-crate
scan, and the reachability-audit frontier._

### Added — prescriptive work-orchestration family (ADR-033)

Eight new macros encoding code-site-local work-needs directly in the type system —
the "code IS the Asana board" family. Each work-need declares its satisfaction
condition, optional temporal frame, and who-refs; `cargo antigen audit` renders
verdicts (`Pending` / `Fulfilled` / `Overdue` / `OutOfFrame`) as a live-projected
board section.

- `#[panel(...)]` — ordered review workflow (ordered\_by → filled\_by → reviewed\_by)
- `#[ddx(...)]` — open differential (competing hypotheses eliminated at the code site)
- `#[rx(...)]` — treatment prescription (what must be done before the site ships)
- `#[triage(...)]` — priority/care-level decision with temporal re-triage deadline
- `#[refer(...)]` — referral to an external owner, anchored at the site needing the look
- `#[biopsy(...)]` — deep investigation request
- `#[culture(...)]` — time-bounded observation ("watch this for N days")
- `#[quarantine(...)]` — site isolation until a named condition lifts

Verdict lattice is isomorphic to the defense tri-state with the false-cell temporally
partitioned by `OutOfFrame` (un-evaluable) vs `Pending` (within frame) vs
`Overdue` (past deadline, loud). Reuses the ADR-029 evaluator — no parallel
evaluator, no cardinality collapse.

Satisfaction uses the same witness leaves as defense (`signers()` / `signed_trailer()`
via `allowed_types`, fingerprint-pinned via NFA-21). Step-presence is verified
(order-agnostic for v0.3; `ordered_all_of` seeded for v0.4).

### Added — witness taxonomy: two kinds (ADR-019 Amendment 1)

The witness taxonomy now has two first-class kinds, each with named members and a
generic escape-hatch:

- **Categorical witnesses** — verdict-producers: `test` / `proptest` / `clippy` /
  `kani` / `prusti` / `verus` / `phantom` / the five supply-chain leaves (ADR-025).
  Attest a verdict (yes / no / indeterminate). Ten boolean leaves at HEAD.
- **Titer witnesses (scalar)** — magnitude-reporters: attest a *measured value*,
  no verdict, trend-trackable. `#[ignorance]` / scan-coverage is retroactively
  recognized as **member-one** (the ignorance frontier computed fresh every scan run).
  Raw escape-hatch: `#[titer(source=...)]`.

Staleness is provenance-relative: scan-derived members are pin-free (live
projection, structurally cannot be stale); source-read members are fingerprint-pinned
(NFA-21) + carry a sub-clause-F source-attestation.

Titer witnesses are three-valued at the value layer: measured / below-threshold /
un-measurable (instrument couldn't reach a reading — the limit-of-detection
third state, distinct from measured-and-low).

The escape-hatch gradient: stdlib-named → adopter-named → raw `#[titer(source=)]`.
In-the-wild raw usage drives recognition of what to name next (the recognition
instrument). Adopters' raw-hatch escape-hatch usages are tracked toward graduation.

### Added — live-projection reporting model (ADR-034)

The report is never a stored truth — it is a live projection of the code,
recomputed every run (like `clippy` reflects current source). Storing a report
would commit `ParallelStateTrackersDiverge` — antigen's own failure-class —
making the stored report a parallel state tracker that can drift.

- `cargo antigen scan/audit` gain `--output <file>`: writes a full enveloped
  render (antigen version, git SHA, timestamp, `report_schema_version`). The file
  is a render-of-a-run, overwritten each time, never read back as authoritative.
- Running `cargo antigen audit` at a release tag *is* that release's reproducible
  defense-posture SBOM — regenerable any time by re-running at the tag.
- A `hooks/pre-commit` example delivers lint-like commit-time feedback (opt-in,
  never default, never writes `.git/`).

### Added — member-aware multi-crate scan (v0.3 cornerstone)

`cargo antigen scan --workspace` scans each workspace member separately, stamps
each with its `name@version` canonical path, and merges into one `ScanReport`.
Cross-member `#[descended_from]` lineage now resolves correctly across crate
boundaries without collapsing member identity.

`ScanCoverage` records which members were enumerated vs scanned — the
**ignorance frontier**: members enumerated by cargo but not walked by the scanner.
The frontier is a set (deduplicated); `unscanned_members()` surfaces it; the audit
produces `UnreachedSite { cause, remedy }` verdicts from it (see below).

### Added — reachability-audit frontier (ignorance mechanism)

`audit_coverage(report)` emits per-site verdicts for the ignorance frontier —
sites the scanner *should* have evaluated but did not. Three causes, each with a
distinct remedy, partitioned by scanner pipeline stage:

- **`Barrier`** — member never enumerated (remedy: extend coverage). Detector live
  from `ScanCoverage::unscanned_members()`.
- **`SubThreshold`** — site reached but recognition heuristic didn't fire (remedy:
  widen recall / add `#[presents]`). Detector composes with multi-crate Layer-2.
- **`Cryptic`** — site present but in a form the scanner cannot parse — macro body,
  hidden `impl Trait` (remedy: pre-process / macro-expand before scanning). Detector
  composes with multi-crate Layer-2.

Barrier verdicts are member-granular (sites within an unscanned member are
unknowable — claiming per-site would assert knowledge never acquired).
SubThreshold/Cryptic verdicts are site-granular (the site was reached; a resolvable
reference points into it).

### Added — `#[antigen_generates(...)]`: macro-output recognition (ADR-014)

`#[antigen_generates(MacroName, emits = [AntigenType, ...])]` declares that a
proc-macro emits antigens the scanner cannot see in the macro body. The macro's
name travels from declaration site to invocation site — the first antigen marker
where declaration and effect live in different places connected only by a name.

### Added — typed `OutOfFrameCause` — sub-cause Layer-2 (ADR-033 / ADR-035)

`WorkVerdict::OutOfFrame` now carries a structured `OutOfFrameCause` sub-enum
distinguishing the four un-evaluable cases: `MissingWhoRef` / `UnresolvableRef` /
`NoApplicableFrame` / `RequiresPreconditionViolated`. Each routes a distinct remedy
rather than collapsing all un-evaluable paths to a single `OutOfFrame` unit — the
ADR-035 Layer-2 (`SubCauseCollapseInTheUnit`) applied to the prescriptive pipeline.
`OutOfFrameCause::remedy()` surfaces the per-cause corrective action.

### Added — `coverage_was_applicable()` — 3-state coverage discriminator (ADR-035)

`CoverageAuditReport::coverage_was_applicable() -> bool` is the discriminator that
makes the 3-state coverage domain readable from a 2-valued `is_complete()`. Before
this, `is_complete() == true` was ambiguous across two structurally distinct
situations: a member-aware scan where every member was reached (verified-complete)
and a flat scan where no member set existed (not applicable). The `(is_complete,
coverage_was_applicable)` pair now distinguishes all three states unambiguously:
`(true, true)` = verified-complete; `(false, true)` = incomplete; `(true, false)` =
not applicable. An allowed C4 downstream projection per ADR-035.

### Ratified — ADR-035: Cardinality Collapse at a Trust Boundary (Three-Valued Type Law)

The Three-Valued Type Law is ratified as a self-applying antigen — antigen detecting
its own type-discipline violation. The law names two layers: `CardinalityCollapseAtTrustBoundary`
(the silent-wrong-verdict; unconditionally forced at every substrate-relative boundary)
and `SubCauseCollapseInTheUnit` (the silent-wrong-remedy; conditionally forced when
failure-stages are distinguishable and route non-interchangeable remedies). The
ceremony (`forward/adr035-three-valued-type-law-ceremony`) was co-signed by
aristotle, math-researcher, and adversarial after the falsification gate confirmed
no counterexample to the no-total-boundary regress lemma.

### Fixed — correctness hardening (ATK suite)

- **Three-valued logic boundary** (ATK-3V-4): `immune_audit_is_substrate_gap()`
  no longer conflates `DisciplinePredicateDeferred` with `SubstrateGap`. A deferred
  supply-chain predicate is `Indeterminate`, not failed. Guard:
  `audit_hint != AuditHint::DisciplinePredicateDeferred`.
- **Scan dedup** (ATK-COV-2): byte-identical `FingerprintMatch` presentations at
  one site deduplicated; ignorance frontier is a set (unscanned members appear once).
- **Serde-validate sidecar** (ATK-SD-*): the sidecar schema is validated at load
  time so a malformed `.attest/` JSON does not silently produce a passing verdict.
- **Immune-stacked same-item gap mask** (ATK-IS-*): stacked `#[immune]` on the
  same item no longer masks a substrate gap on one declaration with a passing
  witness on another.
- **Freshness/version bypass closes** (ATK-FT-1/2/3): three silent false-green
  paths in `antigen-attestation` closed. ATK-FT-1: `fresh_through` active even when
  the sidecar names no current-fingerprint signer; ATK-FT-2: a `fresh_through` site
  with no `through=` date was treated as permanently fresh; ATK-FT-3: a `min_version`
  with a non-`u64`-parseable component coerced to `0` (vacuously passing any floor)
  — `validate()` now rejects with `PredicateParseError::UnparseableMinVersion`,
  paying the partiality upstream so the eval-time leaf never sees the `⊥`.
- **Qualified `priority_order` ref resolution** (ATK-PRES-14b): `priority_order`
  entries in `#[triage]` that use fully-qualified paths (`crate::Module::Variant`)
  are matched precisely by canonical path rather than by bare identifier suffix,
  preventing phantom-resolution false-greens.
- **Signature `allow` against any-strength bypass**: a `#[defended_by(allow_if=...)]`
  clause now requires a matching strength-tier witness; a weak witness no longer
  satisfies a site that requires a stronger attestation tier.

### Changed — cross-crate trust boundary (ADR-017 Amendment 1)

When a `#[defended_by]` / `#[presents]` in crate B addresses an antigen declared
in crate A, the audit honors the claim only when:

1. The `canonical_path` resolves to a real declaration in a scanned member
   (else: `out-of-frame`, the three-valued third value — not silently undefended).
2. Trust is keyed by `canonical_path` (`name@version`): same-type-name across
   crates does not cross-satisfy.

### Changed — `from_itches` is class-specific (ADR-024 Amendment 3)

A `from_itches` entry on a `#[recurrence_anchor]` satisfies the noticing-precondition
only if it names the anchor's own antigen type (or a lineage ancestor). A pure
cross-class itch reference is a phantom — it provides no evidence that *this*
failure-class has been noticed recurring. Realigns code with the doc-comment's
already-stated intent; fixes the vacuous-guard failure shape adversarial found.

### Added — multi-crate scan Layer 2: cross-crate `addresses()` resolution

The cross-crate matching that ADR-017 Amendment 1 specified is now implemented.
`scan_workspace_multi_crate` runs `resolve_cross_member_addresses` over the merged
member report: a `#[presents]` / `#[defended_by]` / `#[immune]` / `#[antigen_tolerance]`
whose addressed antigen is declared in a *different* member is re-stamped to that
declaring member's `canonical_path`, so a legitimate cross-crate defense matches
(closing `DelegateCrossCrateResolutionGap`). An antigen declared in no member leaves
the reference out-of-frame (never a silent cross-satisfy); a same-name collision
across ≥2 members is reported, never guessed. Sibling of the existing cross-member
lineage-parent resolution (identical rule).

### Added — `cargo antigen vcs recurrence`: git-mining for the recurrent family

Mines git history for the three recurrent-emergence stdlib failure-classes and
surfaces recurrence counts — the passive→active loop. `MsrvCreepAfterMajorVersionBump`
(commits changing a `rust-version` line), `GitignorePatternDriftOverReleases`
(commits touching `.gitignore`), `LockfileChurnFromUnpinnedTooling` (commits touching
`Cargo.lock`). Detection only: the *verdict* (anchor it?) stays the adopter's call.
Degrades honestly — git unavailable reports `observable: false` (not a misleading
zero) and exits 0 (never blocks an audit).

### Added — `cargo antigen verify dep-pin --write`: in-place manifest rewrite

The mutation half of `verify dep-pin`. Rewrites `Cargo.toml` IN PLACE via `toml_edit`
(format-preserving — comments, layout, and sibling keys survive), pinning each
unpinned dep to its resolved `=<version>` from `Cargo.lock`. Opt-in by design
(`--write` is never the default — rewriting the adopter's manifest is an
outward-facing mutation); a dep with no resolved lockfile version is never guessed.

### Added — `cargo antigen verify content-hash check --live`: registry-served-hash verification

Additionally verifies the recorded content hash against the hash crates.io *actually
serves* (the sparse-index `cksum`, which is the `.crate` tarball SHA-256) — a
substitution / yank-and-republish detector. Three-valued by construction: `Verified`
(served matches) / `Mismatch` (served differs — loud) / `Unverifiable` (registry
unreachable — `⊥`, never blocks, never escalates). A mismatch escalates the exit
only under `--strict`; the local check stays authoritative.

### Fixed — `ordered_by` never alone fulfills (prescriptive S1, witness-forgery)

`eval_role_workflow` credited a bare `#[panel(ordered_by = ...)]` (no `filled_by`)
as `Fulfilled` once the orderer attested — an opening witness forged as a closing
one. The satisfaction predicate now requires a genuine closing step (≥1 `filled_by`);
a bare-orderer site is `Pending` (awaiting fill), never `Fulfilled`. The
witness-forgery sibling of the three-valued gem (tighten the predicate, not widen
the codomain).

### Fixed — ADR-035 leaf-sweep: `⊥` read-failures lift to `evaluated: false`

Four substrate-absent / input-unreadable arms in `antigen-attestation` reported
`evaluated: true` ("I ran this check and it failed") when no check actually ran —
the `⊥→false` collapse the Three-Valued Type Law forbids. Now `evaluated: false`
(could-not-evaluate): `eval_ratified_doc` doc-not-found / no-parseable-version /
unparseable-found-version (a new `version_is_parseable` gate before `compare_versions`,
the eval-time mirror of ATK-FT-3); `eval_oracles_complete` splits the fused
"missing OR not-complete" arm so an absent oracle is `⊥` while a present-but-incomplete
one stays a genuine fail. Genuine evaluated-and-failed paths (version-below-min,
present-but-incomplete oracle, signer-absent) are unchanged.

### Known v0.3.0-beta limitations (deferred to stable)

- **`cargo antigen --help` audit subcommand text references `#[immune]`**: the
  `--help` text on the `audit` subcommand still describes the deprecated v0.1
  `#[immune(...)]` declaration form. This is a CLI text issue only — the evaluator
  and docs are fully migrated. Will be corrected before the 0.3.0 stable tag.

## [0.2.0] — 2026-05-31

**First stable release of the v0.2 line.** Promotes `0.2.0-beta.1` to stable after a
correctness-hardening and documentation-masterclass pass. No new stdlib surface
beyond beta.1 — this release closes a family of silent-wrong-verdict audit bugs and
brings the documentation fully onto the ADR-029 observe-don't-declare idiom.

### Fixed — silent-wrong-verdict audit bugs

A family of audit bugs that shared one root cause: two-valued boolean logic applied
to a domain that is actually three-valued (definite-yes / definite-no /
not-evaluable). Each silently collapsed "could not evaluate" into a passing verdict.

- **ADR-029 Amendment 1 (verdict precedence)** — a failing `requires=` substrate
  predicate now takes `SubstrateGap` precedence over a passing code witness, so
  `requires=` is a real CI gate rather than decoration.
- **Match3 (ADR-010 Amendment 6)** — three-valued fingerprint evaluation at the
  type level; the audit layer no longer collapses "not evaluable" into a pass.
- Specific cases hardened: zero-threshold convergent-evidence (ATK-CE-5),
  lineage-fidelity child without item kind (ATK-LF-6), phantom recurrence anchor
  (ATK-RECURRENT-7), empty `all_of` via serde (ATK-SC-7), mucosal same-name-fn
  delegation ambiguity, canonical-path equality.
- `#[immune]` multi-stack in associated-const position (named-const fix); malformed
  `until=` now escalates loudly instead of silently granting `Active`;
  immune-channel `requires=` gate extension.

### Changed — documentation

- Full migration of the documentation corpus onto the ADR-029 `#[defended_by]` /
  `#[presents(requires=)]` idiom (concepts, macros, tutorial, for-llm-collaborators,
  adoption, examples-guide, glossary, vision-pitch). Deprecated `#[immune]` is
  retained only in clearly-marked deprecation/historical contexts.

## [0.2.0-beta.1] — 2026-05-28

**First public release of the v0.2 line** — the first published to crates.io
since `v0.1.0-rc.3`, consolidating the internal `alpha.1 → alpha.4` development
arc into one coherent, feature-complete surface. The v0.2 feature set is complete
for this beta; further surface lands **additively (non-breaking)** in later betas
en route to `0.2.0`. Shipped: five stdlib families (deferred-defense,
supply-chain, convergent-evidence, recurrent-emergence, mucosal-boundary) plus the
agentic-coordination and dogfood families; the ADR-028 `AntigenCategory` taxonomy
with G1/G2/G3 enforcement; the ADR-029 observe-don't-declare model
(`#[defended_by]` / `#[presents(requires=)]`, with `#[immune]` deprecated) and its
ADR-030/031/032 follow-ons; and antigen's own codebase dogfooding the primitives
at real failure sites.

### Added — stdlib families

- **Deferred-Defense Family (ADR-023)** — loudness-as-discipline for intentional
  non-immunity; all four primitives ratified by ADR-023.
  - `#[anergy(reason, until, ...)]` — deferred-but-muted posture; `until` REQUIRED
    (A5: anergy without time-bound degrades to silent tolerance); `reason` ≥20 chars;
    optional advisory `expected_co_stimulation`. Hints: `anergy-active` /
    `anergy-co-stimulation-not-arrived` / `anergy-stale`.
  - `#[immunosuppress(rationale, until, ...)]` — surgical silencing with hard
    duration cap enforced at **parse time** (default 90d; workspace config
    `immunosuppress_duration_cap`); `rationale` ≥20 chars. Hints:
    `immunosuppress-active` / `immunosuppress-expired`.
  - `#[poxparty(exercise_type, until, ...)]` — intentional exposure with structural
    isolation via the `antigen-poxparty` Cargo feature (not in the default set);
    `exercise_type` ≥20 chars. Hints: `poxparty-active` / `poxparty-outcome-pending`
    / `poxparty-outside-isolation`. (See Known limitations.)
  - `#[orient(see, adr, attestation_optional)]` — lightest-weight deferred-defense
    primitive; all fields optional; bare `#[orient]` valid. Hint: `orient-active`.
  - `ScanReport::deferred_defenses` (additive `#[serde(default)]`),
    `DeferredDefenseKind` enum, `DeferredDefense` struct, 16 deferred-defense
    `AuditHint` variants, `audit_deferred_defenses()` (UTC-date aging; feeds
    `cargo antigen defer status`), four worked examples.

- **Supply-Chain Defense Family (ADR-025)** — 11 stdlib antigens for
  dependency-boundary risk in the 2026+ threat landscape. Biology cognate:
  Distributed-Boundary Innate-Immunity.
  - `ContentHashMismatch` (**NON-NEGOTIABLE** — content-replacement-at-fixed-version;
    Cargo.lock pins VERSION not CONTENT-HASH; proactive first-attestation via
    `cargo antigen verify content-hash record <crate@version>`),
    `UnsandboxedProcMacro` (in-rustc; higher risk than build-script),
    `UnpinnedDependency`, `UnpinnedTransitiveDependency` (NARROW per B9-R — direct
    dep with `*`/`?` for its own deps), `UnattestedDependencyInclusion`,
    `DependencyUpgradeWithoutDiffReview`, `AutoDependencyChainWithoutPinning`,
    `SuddenDependencyExpansion`, `UnsandboxedBuildScript`,
    `PostInstallScriptInDependency`, `MaintainerChangeWithoutReattestation`.
  - `audit_supply_chain()` with combinator-aware predicate evaluation (`AnyOf`
    discharges failing siblings when a branch passes — ATK-SC-AUDIT-1 fix);
    17 supply-chain `AuditHint` variants; substrate-witness runtime
    (`schema` / `witness` / `evaluate` / hand-rolled `manifest` scanner, no toml
    dep per ADR-002 Amendment 2); 5 new `antigen_attestation::Leaf` variants
    (`DepPinned`, `DepAttested`, `MaintainerUnchanged`, `ContentHashMatches`,
    `SandboxClean`; sealed-set exhaustivity 5→10); `antigen::stdlib::supply_chain`
    re-importable members; 3 examples.

- **Convergent-Evidence Family (ADR-024)** — 7 macros for backward-looking evidence
  aggregation.
  - `#[diagnostic]` (clinical differential-diagnosis grounding; counts distinct
    WitnessClass CATEGORIES per C1), `#[clonal]` (`SeedKind::Fixed(_)` is a COMPILE
    ERROR per C2), `#[igg]` (unique signer count enforced per ATK-CE-3-B),
    `#[crossreactive]`, `#[polyclonal]`, `#[monoclonal]`, `#[adcc]`.
  - `antigen::WitnessClass` (6 variants), `antigen::SeedKind` (4 variants),
    11 convergent-evidence `AuditHint` variants, `audit_convergent_evidence()` +
    report types, `ScanReport::convergent_evidences` (additive), 3 examples,
    trybuild compile-fail fixtures (CE-1, CE-2).

- **VCS-Information-Loss Family (ADR-026)** — 11 stdlib antigens for
  git-history-erasing operations. Biology cognate: Immune Amnesia
  (`ForcePushErasingHistory` ↔ measles memory-lymphocyte depletion; Mina et al.
  2015). Includes `RollbackWithoutTriageCommit` (AUTHOR-DECLARATION detection,
  Algorithm C), `RefactorWithoutPreservationOfWhy`, `BranchDeletionWithoutAttestation`,
  `SquashMergeLosingIntermediateState`, `CherryPickLosingOriginalContext`,
  `RebaseRewritingHistoryWithoutLog`, `UnpushedBranchWithSubstantiveWork`,
  `StashedWorkAbandoned`, `MergeConflictResolutionWithoutAttestation`,
  `AmendedCommitWithoutOldHashPreservation`.
  - `#[triage_commit]` decisional macro (rollback-as-triage; distinct from passive
    `#[orient]`; required `triage_decision` / `rollback_target` / `triaged_by` /
    `rationale` ≥20 chars / `rollback_due_within_minutes` >0). `TriageDecision` enum
    (`Black|Red|Yellow|Green|White`; `mandates_rollback()`, `parse_decision()`).
    `ServerSideEnforcementMode` enum (`FrictionOnly` default | `Structural` v0.2.1+).
    14 `vcs-` audit hint variants.

- **Recurrent-Emergence Family (ADR-024 §Family 2)** — scan + audit + stdlib +
  worked example. `MsrvCreepAfterMajorVersionBump`,
  `GitignorePatternDriftOverReleases`, `LockfileChurnFromUnpinnedTooling`. Six
  recurrent-declaration kinds: `#[itch]`, `#[recurrence_anchor]`, `#[crystallize]`,
  `#[chronic]`, `#[saturate]`, `#[strand]`. Hints: `ItchNoticedNotAnchored`,
  `RecurrenceThresholdReachedNoAction`, `RecurrenceAnchorNoItchPrecondition`,
  `ChronicManagedByRequired`, `ChronicSinceNotADate`, `CrystallizeWithoutSource`.

- **Mucosal Boundary Family (ADR-027 + Amendment 1)** — scan + audit + stdlib +
  `cargo antigen mucosal-map` CLI. `#[mucosal]`, `#[mucosal_delegate]`,
  `#[mucosal_tolerant]`; 13-variant `MucosalKind`; three-tier delegate
  kind-mismatch diagnosis. Stdlib: `UndefendedTrustBoundary`,
  `DelegatedDefenseWithoutMatchingHandler`, `ToleratedBoundaryWithoutReview`.

- **Agentic-Coordination Family** — coordination-layer failure-classes from
  multi-session / multi-agent workflows. `AgentWakeWithoutSubstrateDeltaInjection`
  (agent resumes without reading the substrate delta accumulated while idle;
  stale context → wrong routing) and `DelegateCrossCrateResolutionGap` (mucosal
  handler-kinds index is intra-crate only; cross-crate delegates false-positive
  as target-missing — residual risk at v0.2; multi-crate scan is v0.3+). Kept in
  v0.2 on team-readiness judgment; further coordination-layer fail-classes are a
  v0.3 research arc.

- **Dogfood Family** — antigen's own codebase carries live markers at real
  failure sites (the Layer-1 dogfood). 24 stdlib antigens:
  `AntigenDeclarationMissingCategory`, `DelegatedHandlerKindMismatch`,
  `WitnessClaimWithoutImplementation`, `VecCardinalityMasqueradingAsSet`,
  `AuditHintWithNoUpstreamPreconditionCheck`, `RatifiedSpecDriftFromImpl`,
  `UnvalidatedSealedEnumAcceptance`, `FingerprintStringWithoutDslValidation`,
  `FingerprintDigestWithoutFormatValidation`, `SilentIntentNullification`,
  `ActiveArgumentDiscard`, `ScannerBoundaryFalseNegative`,
  `BiologyGroundingClaimDrift`, `UnstableHashAsPersistedValue`,
  `AuditFingerprintSelfReferential`, `SilentSemanticMismatchAtTrustBoundary`,
  `DeclaredCapabilityWithNoProductionPath`, `CapabilityOmissionAtLowering`,
  `AntigenFingerprintDivergesFromClassExtension`, `ParallelStateTrackersDiverge`,
  `ScanVisitorDigestAssignmentOmission`, `FailingTestWithoutIgnorePin`,
  `MarkerStructDeadCodeInBinary`, `SerdeDefaultMaskingStructLiteralBreak`. Live in-source markers
  (scan.rs): `#[presents(VecCardinalityMasqueradingAsSet)]` on `AntigenDeclaration`,
  `#[presents(ScannerBoundaryFalseNegative)]` on `scan_workspace`,
  `#[presents(ScanVisitorDigestAssignmentOmission)]` on `ScanVisitor`.
  (audit.rs): `#[presents(ParallelStateTrackersDiverge)]` on `AuditHint`,
  `#[presents(DeclaredCapabilityWithNoProductionPath)]` on `WitnessStatus`,
  `#[immune(AuditFingerprintSelfReferential)]` (witness → fingerprint override test),
  `#[immune(AuditHintWithNoUpstreamPreconditionCheck)]` (witness → adversarial fixture),
  `#[presents(DelegateCrossCrateResolutionGap)]` on `audit_mucosal`.

### Added — AntigenCategory taxonomy + enforcement (ADR-028)

- **`AntigenCategory` enum** — `SubstrateAlignment | FunctionalCorrectness`
  (sealed; variants require an ADR amendment). Optional `category` field on
  `#[antigen]`; hybrid antigens accept both variants. `MacroAntigenCategory` is
  the proc-macro-side mirror (avoids an `antigen` ↔ `antigen-macros` cycle).

- **G1 — v0.1-carryover migration hint** — absent `category` emits
  `antigen-category-defaulted-implicit-functional` at scan/audit time (soft default
  to `FunctionalCorrectness`). Per the ratified G1 decision the v0.2 enforcement is
  **scan-time-only**; the parse-time hard error for new declarations + the v0.1/v0.2
  migration-record discriminator are **deferred to v0.2.x** (ADR-028 Amendment 4).
  17 example-site hits confirmed.

- **G2 — category-vs-witness-type cross-check at AUDIT time** (ADR-028 Amendment 3
  records why audit-time, not parse-time: a single `#[antigen]` cannot see its
  separately-declared `#[immune]`s at macro-expand time; the antigen-immunity join
  only exists once the scan report assembles). `audit_category()` reads the witness
  type structurally from each immunity (`requires_predicate.is_some()` =
  substrate-witness; non-empty `witness` = code-witness) and fires
  `antigen-category-claim-inconsistent-with-predicate-type` when a single-axis
  category has no matching witness (or a hybrid has zero axes witnessed). Zero
  immunities is not flagged (orthogonal coverage gap).

- **G3 — hybrid-incomplete-evidence + `--category` CLI filter.** A hybrid
  `[SubstrateAlignment, FunctionalCorrectness]` with exactly one axis witnessed
  emits `antigen-category-hybrid-incomplete-evidence` (partial coverage, distinct
  from the full-violation `claim-inconsistent`). `cargo antigen scan --category` /
  `cargo antigen audit --category <substrate-alignment|functional-correctness>`
  filter by category (hybrid matches either; absent-category defaults to
  functional-correctness; unrecognized value exits 2).

- **AntigenCategory audit-hint tiering** (per ADR-028 Amendment 4): shipped in
  v0.2 — `antigen-category-defaulted-implicit-functional`,
  `antigen-category-claim-inconsistent-with-predicate-type`,
  `antigen-category-hybrid-incomplete-evidence`. Deferred to v0.2.x with named
  dependencies — `antigen-category-missing-explicit` (needs the v0.1/v0.2
  migration-record discriminator) and `antigen-category-mismatch-witness-type`
  (advisory soft-smell layer; lands after the hard-violation hint proves out).

### Fixed

- **ATK-RECURRENT-2** (`dd51d4b`) — `RecurrenceAnchor` audit arm checked the
  downstream action (`acted_on`) but not the upstream precondition (a matching
  `#[itch]`). Added `AuditHint::RecurrenceAnchorNoItchPrecondition` + threaded
  `itch_antigen_types` into `evaluate_recurrent_hints`; positive + clearing cases
  tested in the adversarial fixture.
- **ATK-VCS-5** (whitespace-only field silent acceptance) — `#[triage_commit]` with
  `rollback_target = "   "` / `triaged_by = "   "` parsed silently; now rejected at
  parse time (`Some("")` guard widened to `Some(s) if s.trim().is_empty()`).
- **ATK-SC-1-A** (rubber-stamp bypass), **ATK-SC-2-A** (sidecar-corruption
  downgrade — malformed sidecar must NOT silently become `NoAttestation`),
  **ATK-SC-AUDIT-1** (`any_of` semantics), **ATK-CE-3-B** (IgG raw-count bypass →
  unique signer count enforced).

### Documentation (ADR amendments ratified this arc)

- **ADR-024 Amendment 1** — `#[titer]` biology-grounding axis reassignment;
  operational substrate primary, biology approximate cognate.
- **ADR-026 Amendments 1–3** — rollback-as-triage uses `#[triage_commit]` not an
  `#[orient]` extension (1); `TriageDecision` variant-semantic backfill + `camp::triage`
  RHYME-tier connection + START-attribution honesty (2); AUTHOR-DECLARATION
  (Algorithm C) rollback detection + `vcs_server_side_enforcement_active()` guard (3).
- **ADR-027 Amendment 1** — mucosal taxonomy disambiguation (15→13 `MucosalKind`;
  type-of-data-crossing-boundary axis; `handled_by` as `syn::Path`; delegate
  three-tier audit; `#[mucosal_tolerant]`; 6→11 hints).
- **ADR-028 Amendment 2** — predicate-leaf requirement applies to the WITNESS layer,
  NOT the fingerprint scan-side pattern (`doc_contains(...)` is a valid scan-side
  fingerprint for a SubstrateAlignment antigen whose witness reads substrate).
- **ADR-028 Amendment 3** — the category-vs-predicate-type cross-check is
  AUDIT-time, not parse-time (the antigen-immunity join only exists once the scan
  report assembles); resolves the G2 campsite.
- **ADR-028 Amendment 4** — §Enforcement-Surface re-sync post G1/G2/G3: table row 1
  corrected from "parse-time HARD ERROR" to "v0.2 migration hint; hard error v0.2.x";
  cross-check row → audit-time-only; audit-hint vocabulary tiered (v0.2 shipped vs
  v0.2.x deferred with named deps); inline backward-compat annotation fixed.

### Known limitations

- **Poxparty isolation (A3)** — Cargo does not reliably propagate `CARGO_FEATURE_*`
  to proc-macro expansion environments. The env-var check is best-effort; the
  load-bearing isolation is the `#[cfg(feature = "antigen-poxparty")]` gate (items
  inside an inactive cfg block never reach macro expansion). Tracked for a future
  ADR amendment when Cargo's propagation behavior stabilizes.
- **Cross-crate mucosal delegates** — `DelegateCrossCrateResolutionGap`: the
  handler-kinds index is intra-crate only; cross-crate delegates false-positive as
  target-missing. Residual risk at v0.2; the multi-crate scan pass is v0.3+.
- **ADR-029 multi-channel verdict precedence** — a presents-site that carries
  *both* a `requires =` substrate-witness predicate that fails (sidecar
  absent/stale) *and* a passing `#[defended_by]` code witness is currently
  reported as `Defended`, masking the substrate gap. Because the verdict resolves
  to the highest available tier, the `SubstrateGap` signal — the whole point of
  declaring `requires =` to catch drift at CI time — is never surfaced for that
  site. The ADR-029 verdict matrix is silent on the simultaneous-multi-channel
  case; resolution (likely surfacing `SubstrateGap` alongside or in precedence
  over `Defended`) is queued as an ADR-029 amendment for a later beta. Tracked as
  `pv-requires-masked-by-code-witness`.

---

## [0.1.0-rc.3] — 2026-05-22

Small CLI patch: expose `--version` (and `-V`) on the `cargo antigen`
subcommand. The flag is what camp v0.1.1's version-mismatch warning
sub-step depends on — without it, camp cannot introspect the installed
`cargo-antigen` version from a subprocess invocation.

### Added

- `cargo antigen --version` / `cargo antigen -V` print the workspace-
  pinned package version (clap's standard `version` attribute on both the
  `cargo-antigen` parser and the `antigen` sub-parser).
- `atk_version_flag.rs` integration test locks down the contract: exit 0
  and stdout contains the workspace version string.

## [0.1.0-rc.2] — 2026-05-20

Hotfix release: wire the substrate-witness pipeline end-to-end. ADR-019's
`#[immune(X, requires = <predicate>)]` form parsed and emitted a JSON
marker at macro-expansion time, but scan walks **written source** via
`syn::parse_file` and never saw the post-expansion doc marker. Every
substrate-witness immunity reported `tier = None, hint = NoneApplicable`
("missing witness identifier") — even the shipped
`antigen/examples/substrate_witness.rs` example. Surfaced via the camp/
dogfood (`camp/` Rust crate now tracked as canonical dogfood content per
the updated `.gitignore`).

### Fixed

- **Substrate-witness pipeline wiring**: scan now parses
  `requires = <predicate>` directly from `#[immune]` /
  `#[antigen_tolerance]` source attributes via a shared parser. The doc-
  marker channel survives as a fallback for rc.1-compiled code, but
  discovery no longer depends on macro expansion. (Token-level diff:
  audit on `antigen/examples` now reports `tier = None, hint =
  DisciplineSidecarMissing` for substrate-witness sites without sidecars,
  routing correctly through `audit_substrate_witness` instead of
  falling through to the code-witness branch.)
- **`RequiresExpr::to_json` wire format**: rc.1 hand-rolled JSON with the
  shape `{"kind":"leaf","leaf":{...}}` which `Predicate` serde rejected
  as schema-invalid. rc.2 routes through the real `Predicate` type so
  the JSON is byte-identical to what the audit evaluator deserializes
  (locked by `parser::requires_json_tests::json_shape_is_flat_not_nested`).
- **`AuditHint` collapse**: rc.1 mapped every substrate-witness hint
  variant to `NoneApplicable` / `ExternalToolPrefixRecognized`, hiding
  the substrate-pipeline diagnosis from the user. rc.2 surfaces 14 new
  variants 1:1 with `antigen_attestation::SubstrateAuditHint`, so the
  user-facing hint names the actual state (sidecar-missing,
  predicate-failed, substrate-stale, etc.).

### Added

- New `parser` feature on `antigen-attestation` exposes the
  source-attribute parser; off by default (runtime crate stays syn-free).
  Both `antigen-macros` and `antigen` turn it on.
- `antigen_attestation::parser::RequiresExpr::to_predicate()` returns
  the runtime `Predicate` directly (the new load-bearing lowering).
- `atk_a3_substrate_witness_pipeline.rs` — regression test that pins
  the three pipeline wirings (scan capture, audit routing, hint
  surfacing). Would have caught the rc.1 bug at scan-write time.

### Internal

- `Option::expect` is const since Rust 1.83; helper `fn sample_date()`
  test fixtures in `antigen-attestation/tests/*` lifted to `const fn`
  (clippy 1.95 `missing_const_for_fn`).
- `f64::midpoint` used in `tolerance_attested.rs` example
  (clippy 1.95 `manual_midpoint`).
- `Option::is_none_or` replaces `Option::map_or(true, ...)` per clippy
  1.95 `unnecessary_map_or`.

## [0.1.0-rc.1] — 2026-05-20

First release candidate. Consolidates A2 (core macros + scan + audit completion)
+ A3.5 (onboarding sweep) + the discipline-witnesses arc (ADR-019, ADR-020,
ADR-021) into a single shipped rc. The earlier 2026-05-08 `[0.1.0-rc.1]` entry
below documents preliminary substrate that was planned but never shipped to
crates.io — its content is absorbed here.

### Discipline-witnesses arc (NEW — 2026-05-19/20 session)

#### ADR-019 — Substrate-witness predicate family (RATIFIED)

Extends witness vocabulary beyond code-side substrate (test_fn / proptest! /
clippy:: / phantom-type) to **substrate other than the code being audited**:
ratified docs, sign-off records, signed git trailers, oracle completion
markers, attestation sidecars. Closed combinator grammar (`all_of` / `any_of`
/ `not`) over sealed leaf primitives (`signers`, `ratified_doc`,
`signed_trailer`, `oracles_complete`, `fresh_within_days`). Tier-honesty
preserved via three-axis output (`WitnessTier × AuditHint × EvidenceKind`).
ADR-005 Amendment 3 extended to substrate-witness recognition surface;
ratchet-asymmetry property named explicitly; bounded audit-of-audit recursion
named explicitly.

#### ADR-020 — Cross-cutting attestation primitive (RATIFIED)

`attested = (who, allowed_types, why, scope)` available as a macro parameter
on any antigen-related macro (`#[antigen]`, `#[immune]`, `#[antigen_tolerance]`,
possibly `#[descended_from]`). Distinct from `requires =` substrate-witness
predicates — attestation is the *declaration* of who attests; substrate-witness
predicates *evaluate* against that declaration. Layer 1 adoption gradient
(ADR-009) compliance: light-touch attestation reaches every adopter without
requiring the full predicate language. Notary-arc biology grounding (B6 from
naturalist work).

#### ADR-021 — OracleRef generalization + Oracle artifact-class (RATIFIED)

Oracle as **structurally distinguished artifact** (Model B per Tekgy decision)
rather than typed pointer. Five-state lifecycle:
`Draft → Complete → {Deprecated, Retired, Revoked}`. Dedicated stewardship
role separate from signers. State transitions are steward-authorized events
with provenance trail. Signers cannot attest against DRAFT oracles
(`oracle_state_at_attestation` field enforces). OracleRef as tagged union
covers LocalFile, Url, Doi, Arxiv, GitHubIssue, Other — same structural
treatment regardless of physical location. Audit validates metadata +
completion-marker + version-pin but **never reads/interprets oracle content**
(substantive judgment lives at sign-time human/LLM work; tier-honesty caps
oracles at Execution tier). Additive-only schema evolution discipline ratified
(no migration framework needed). Five Class-1 biology cognates including
immune-memory + V(D)J recombination.

#### Tolerance-ratification (scout S1 — plugs ADR-011 vibes-grade gap)

`#[antigen_tolerance(X, sidecar = true)]` opt-in enables structured
attestation for tolerance claims; schema **isomorphic to immunity sidecars**.
Audit emits new `tolerance-vibes-grade` hint with `EvidenceKind::None` for
unattested tolerance — makes the tier-honesty gap visible.

#### Three signature tiers

`SignatureStrength = TextStamp | GitTrust | CryptoSigned`. Categorical, NOT
ordinal — trust is project-declared per-antigen, not inherent in the enum.
TextStamp (name + timestamp; no infra required) opens adoption to LLM agents
and reviewers without git config. GitTrust (git config user.name/email +
fingerprint pin) is the v0.1 default for human teams. CryptoSigned slot
reserved for v0.4+ DSSE envelope + Sigstore identity-bound activation path.

#### Delta-attestation with anti-laundering safeguards

`cargo antigen attest delta` records `SignerBasis::DeltaFrom { ... }` carrying
chain-depth cap (default 3, hard max enforced), cumulative-fingerprint
tracking (to last Fresh-basis signature), and required non-empty rationale
(minimum char count enforced at CLI + schema). Closes the laundering surface
where small carry-forwards could smuggle substantive change.

#### Process discipline: cross-ADR substrate-grep sub-routine

`docs/process.md` amended with Phase 3 cross-ADR surface check — prevents
naming collisions (e.g., F28-R2 where `attest oracle complete` would have
collided with `oracle complete` lifecycle verb). Caught at draft-time rather
than ship-time.

### Implementation (v0.1-rc shipping)

#### New crate

- `antigen-attestation` — Ratification schema + substrate-witness predicate
  evaluator. Separate workspace member; `serde_json` + `chrono` deps only.
  Includes Oracle schema (5-state lifecycle), SignerBasis enum (Fresh /
  DeltaFrom with anti-laundering fields), OracleRef tagged union (6 variants),
  Provenance struct, StateTransition event log. v0.0.1 name reserved on
  crates.io 2026-05-20 prior to this ship.

#### CLI families

- `cargo antigen attest scaffold | sign | check | delta | list | gc` — full
  immunity-sidecar lifecycle. `attest delta` enforces anti-laundering caps
  + rationale minimum at CLI layer.
- `cargo antigen tolerate scaffold | sign | check | list` — parallel family
  for tolerance ratifications via isomorphic schema.
- `cargo antigen oracle list | status | declare | complete | deprecate |
  retire | revoke` — full Oracle artifact-class lifecycle CLI (slice e per
  ADR-021).
- Removed: `attest migrate` (additive-only schema discipline obviates), `attest
  move` (error-path enforcement via gc + scan/audit yelling provides discipline
  through consequences; convenience verb unneeded).

#### Audit output extensions

- `EvidenceKind` enum (TypeSystemProof | Behavioral | SubstrateState) as
  third audit-output axis.
- `signature_strength` field per signer on audit output (git-trust default;
  text-stamp + crypto-signed as Tekgy verdict 2026-05-20).
- New hints: `discipline-predicate-passed-substrate-current`,
  `discipline-substrate-stale`, `discipline-predicate-passed-via-delta-chain`,
  `discipline-substrate-delta-chain-near-cap`, `tolerance-vibes-grade`,
  `oracle-in-draft`, `oracle-completion-attested`, `oracle-reference-malformed`,
  + others. Tier-honesty mapping documented in `docs/witness-tiers.md`.

#### Tambear adoption (Phase 4 shipped)

Tambear's sinh/cosh signed-zero discipline declared and substrate-witnessed
end-to-end against the v0.1-rc primitives. First-user adoption arc closed
against the originating motivation.

### A3.5 Onboarding sweep

#### Documentation (new)

- `docs/tutorial.md` — five-step narrative (declare → presents → scan → immune → audit);
  real scan/audit output throughout; teaching point on `.expect()` vs `body_contains_macro`
- `docs/fingerprint-grammar.md` — all 10 operators documented with behavior, examples,
  and receiver-rendering reference table; explicit tokenization-asymmetry warning
- `docs/troubleshooting.md` — all observable error categories from live scan+audit output;
  856 fingerprint match count explained; 39 parse failure categories; quick diagnostic table

#### Documentation (updated)

- `docs/fingerprint-grammar.md` — receiver-rendering reference table; `has_method` item-kind
  scope clarified (impl sites only); `"(self, Self)"` corrected throughout (was `"(Self, Self)"`)
- `docs/decisions.md` — ADR-010 concrete example + ratified declaration corrected to
  `"(self, Self) -> Self"` (Receiver token renders as `"self"`, not `"Self"`)
- `docs/tutorial.md`, `docs/scope.md`, `docs/where-to-look-for-antigens.md`,
  `docs/expedition/stdlib-seed-antigens.md`, `docs/expedition/case-study-determinism-class.md`
  — same receiver-form correction
- `README.md` — full narrative deep-draft replacing terse status block; what/what-not/
  vocabulary/workflow/architecture/tambear-origin/v0.1.0-scope/setup/license
- `docs/usage-patterns.md` — `#[antigen_tolerance]` decision tree + good/weak rationale
  examples + `until` field usage
- All four crate-level doc-comments improved; per-macro ADR references; stale "future"
  references removed; `antigen-fingerprint` positioned as canonical-implementation crate

#### Examples (new)

- `antigen/examples/descended_from.rs` — inheritance chain; scan produces state-7 inherited
  Presentation on `UseAfterFreeClass` with `inherited_from`
- `antigen/examples/antigen_tolerance.rs` — opt-out pattern; tolerance absorbs cross-reactive
  match; `until = "v0.2"` flags re-evaluation
- `antigen/examples/phantom_witness.rs` — phantom-type witness; audit classifies as
  `WitnessTier::FormalProof` with `PhantomTypeShapeRecognized` hint; `--format json` shows tier

#### Examples (updated)

- `antigen/examples/broken_witness.rs` — fingerprint narrowed to `matches("Looks*")`;
  workspace-wide cross-reactivity eliminated; docstring teaches the lesson

#### Engine

- `antigen-fingerprint`: symmetric canonicalization of `has_method` signature strings via
  proc_macro2 round-trip at parse time. User-natural Rust syntax works: `"(&mut self)"`,
  `"(& mut self)"`, and sloppy-whitespace variants all canonicalize to the same form and
  match the same signatures. Pre-A3.5 the engine required the spaced form `"(& self, ...)"`;
  that footgun is eliminated. (ATK-W6a-013 / ATK-W6a-013b; first real instance:
  tambear's PanickingInDrop, surfaced during A3.5 onboarding cross-check)
- `normalize_signature_canonical` now returns `Option<String>`; strict fail on malformed
  signature string (proc_macro2 parse error → `None`, not silent fallback to plain
  `normalize_ws(raw)`). Grounds: ADR-005 §1 sub-clause F — lenient fallback reintroduces
  the spacing asymmetry the fix exists to eliminate. Malformed patterns surface a
  compile-time parse error anchored at the offending literal. (Amendment 5 OQ1 ratified
  strict; bb22e56)

#### CLI

- `cargo-antigen`: `new` and `vaccinate` subcommands hidden from `--help` (not yet
  implemented; surface when A5 ships them)

#### Audit output

- `cargo antigen audit` human-readable output now distinguishes `FormalProof` and
  `Reachability` witnesses. Option A: per-tier sub-counts in the resolved summary
  ("N formal-proof", "N execution", "N declared (Reachability)"). Option B:
  confirmed-claims section parallel to warnings block, listing Execution+ tier claims
  with tier name and audit hint. Phantom-type witnesses now produce explicit positive
  feedback in human output. (ATK-A3-019)

#### Tests

- 240 passing, 31 ignored (up from 187/18-suites at rc.1); 21 suites
- ATK-W6a-013 inverted: was "must NOT match" (documenting bug); now "must match" (fix verified)
- ATK-W6a-013b added: tambear footgun — `has_method("drop", "(&mut self)")` now matches
  across natural/canonical/sloppy whitespace variants
- ATK-W6a-017 added: Self/self token-class distinction guard — `"(Self, Self) -> Self"` must
  NOT match `fn meet(self, other: Self)`; two positive controls included (receiver pattern matches
  receiver sig; static pattern matches static sig) (cd33c96)
- ATK-W6a-018 added: four malformed-signature cases verify the strict `None` path — unbalanced
  open paren, extra closing paren, unterminated string literal, raw backtick (bb22e56)
- ATK-A3-019 activated (was `#[ignore]`): asserts human audit output contains both
  "FormalProof" (Option B confirmed-claims) and "formal-proof" (Option A summary)

## [0.1.0-rc.1] — 2026-05-08

First functional release candidate. Sweep A2 (core macros + scan + audit
completion) closed with 187 passing tests across 18 suites; clippy + doc gates
clean. Cuts the substrate the JBD team built across A1 (10 ratified ADRs +
4 amendments) and A2 (W1-W8 implementation work-streams).

### Added

#### Macros (`antigen-macros`, re-exported from `antigen`)

- `#[antigen(name, fingerprint, family, summary, references)]` — declare a
  named failure-class with a structural fingerprint per ADR-001 + ADR-010
- `#[presents(antigen_type)]` — mark code as exhibiting an antigen's
  structural pattern
- `#[immune(antigen, witness, rationale?)]` — declare immunity with a
  witness reference; required-witness enforcement at parse time
- `#[descended_from(parent)]` — propagate antigen markers through
  structural derivation (cross-crate walking is A3 work)
- `#[antigen_tolerance(antigen, rationale, until?, see?)]` — mark
  fingerprint matches that are deliberate-not-vulnerable (ADR-011);
  rationale required and non-empty, `until` non-empty if present
- All five macros emit token-precise error spans per ADR-008 (W4); errors
  anchor at the offending literal or the macro's argument list, never at
  call_site

#### Fingerprint grammar (`antigen-fingerprint`)

- v1 DSL parser per ADR-010 Amendment 1 (Path C: custom `syn::ParseBuffer`,
  not `syn::parse2::<Expr>`)
- Seven item-level operators per ADR-010 Amendment 3 Clause C:
  - `item: <kind>` — struct / enum / trait / fn / impl / type / mod
  - `name: matches("<glob>")` — bespoke 20-line `glob_match_ident` with
    `*` and `?` metachars (no external glob dep)
  - `variants: M..=N` — inclusive enum variant-count range
  - `has_method("<name>", "<sig>")` — signature pre-parsed at load time
    per Performance Invariant 2 (cached as `Option<String>` on
    `MethodPattern::normalized_signature`)
  - `attr_present("<path>")` — outer attribute path matcher
  - `doc_contains("<substring>")` — case-sensitive doc-text search
  - `body_contains_macro("<name>")` — native syn::Block walker for
    `panic!`/`unreachable!`/etc. (Clause C, NOT delegated to a body engine)
- Composition: `all_of([...])`, `any_of([...])`, `not(...)` with
  `not`-only-inside-`all_of`-with-positive-sibling enforcement per OQ3
- Performance invariants honored: single-pass walks, pre-parsed sigs,
  depth ≤ 10 + node-count ≤ 256 caps at parse time, node-kind dispatch
- Compile-time DSL validation: `#[antigen]` rejects malformed fingerprints
  at macro-compile time per Clause E

#### Scan (`antigen::scan` + `cargo antigen scan`)

- `scan_workspace` walks the source tree, collects explicit declarations,
  then synthesizes fingerprint-match `Presentation`s with
  `MatchKind::FingerprintMatch` per ADR-001 Amendment 1 Change 2
- Item-identity matching via `ItemTarget` (W3) — replaces the pre-W3
  proximity heuristic; methods inside `impl` blocks carry their enclosing
  trait + target type so two `drop` methods on different types don't
  collide
- Tolerance recognition: `#[antigen_tolerance]` markers acknowledge
  fingerprint matches; `unaddressed_presentations` consults tolerances
- `ScanReport::orphaned_tolerances()` flags tolerances referencing antigens
  not declared in the workspace (ATK-A2-009)
- Span-precise line tracking via `syn::spanned::Spanned::span().start().line`
  (requires `proc-macro2` `span-locations` feature)
- Path-qualified attribute name handling (`#[antigen::antigen]`,
  `#[my_alias]` after `use ... as`)
- 5-state CLI output: explicit / fingerprint match / tolerated / unaddressed
  / immunity claims, with remediation guidance

#### Audit (`antigen::audit` + `cargo antigen audit`)

- Four-tier `WitnessTier { None=0, Reachability=1, Execution=2,
  FormalProof=4 }` per ADR-005 Amendment 3 (discriminant 3 reserved for
  future BehavioralAlignment)
- Parallel `AuditHint` axis with structured variants for per-case
  disambiguation (FunctionResolves / TestAttributePresentNotInvoked /
  TestAttributePresentIgnoreSkipped / ExternalToolPrefixRecognized /
  PhantomTypeShapeRecognized / AmbiguousResolution / ...)
- `WitnessKind` extended with `IgnoredTest` (anergic-B-cell cognate),
  `PhantomType { proof_type, type_params, constructor }` per ADR-013
- `WitnessStatus::Ambiguous { candidates }` for collision detection;
  `FunctionIndex` tracks all candidate locations rather than silently
  picking one
- `is_well_formed() = meets_tier(Execution)` — informational only; not
  wired to `--strict` at v0.1 (see ATK-CLI-003 below)
- `audit --strict` gates on `all_meet_tier(Reachability)` — exits 1 for
  `Missing`/`NotFound`/`Ambiguous` witnesses (`WitnessTier::None`);
  `Execution`-tier gating arrives in A3 with `cargo test` integration
  (ATK-CLI-003: previously always-exited-1, training users to disable)
- Phantom-type witness recognition for `Type::<Args>::ctor` shapes
- Structural `proptest!` witness detection (W5) — replaces the pre-W5
  textual `source.contains("proptest!")` sentinel that over-classified
  every function in any file mentioning the macro
- `#[ignore]` distinction in `detect_kind` (W7 + ATK-A2-012)

#### Workspace + tooling

- `antigen-fingerprint` workspace member per ADR-010 Amendment 3 Clause E
  — both `antigen-macros` (compile-time validation) and `antigen`
  (scan-time matching) consume it
- `cargo antigen scan|audit|new|vaccinate` subcommand binary; `new` and
  `vaccinate` are scaffolded with helpful "design phase" messages
  (real implementations land A3+/A5)
- Workspace-level `[lints.rust]` `unsafe_code = forbid` and
  `missing_docs = warn`; `[lints.clippy]` pedantic + nursery at warn level
  with explicit ergonomic allow-list

### Known v0.1.0 limitations (deferred to later sweeps)

- **No function-body fingerprint patterns** (ADR-012 amendment) — v1
  grammar matches at the item level. `body_contains_macro` is the one
  body-level operator shipped (native syn walker). General `body_pattern`
  awaits the W6b ast-grep subprocess decision per ADR-015.
- **No macro-output recognition** (ADR-014 `#[antigen_generates]`) —
  derives, declarative macros, and proc-macros expand outside the scan's
  view. v1 scans pre-expansion source only per ADR-010 Amendment 3 Clause A.
- **No cross-crate `#[descended_from]` propagation** — A3 sweep work.
- **No witness execution** — audit reports Reachability for `#[test]`
  resolution; promotion to Execution requires actual `cargo test`
  invocation (A3+).
- **No external-tool invocation** — `clippy::`/`kani::` prefixes get
  Reachability + `ExternalToolPrefixRecognized` hint until A3+ runs the
  tools.
- **No `cargo antigen vaccinate`** — A5 work; requires fingerprint grammar
  v1 and witness library to be stable (both ship in 0.1.0).
- **No `antigen-stdlib`** — A5 populates the 8 first-principles failure
  classes. `panicking-in-drop` exercises body_contains_macro end-to-end
  in `antigen/examples/basic.rs` as a standing demo.
- **No fabricated-path-prefix detection** — `nonexistent::module::real_fn`
  silently drops the prefix at this layer; the underlying tier-honesty
  catches it (ATK-A2-011), but full module-graph resolution is A3 work.
- **Same-name proptest+free-fn collision is `Ambiguous`, not silently
  picked** — by design (ATK-W5-007 reframe under W7); the user resolves
  by renaming or qualifying. Consistent with ATK-A2-005's discipline.

### Reserved / placeholders

- `antigen-fingerprint` and `antigen-macros` are workspace-internal
  crates published to crates.io alongside `antigen` (because Cargo
  requires them to be) but documented as "use `antigen` instead." The
  evaluator-trait public-vs-private question (ADR-015 §S3) is reserved
  for the second-backend ratification.

## [0.0.1] — 2026-05-07

### Added

- Workspace scaffolding (Cargo.toml, dual MIT/Apache-2.0 license)
- `antigen` crate placeholder (lib) with module-doc explaining design phase
- `cargo-antigen` crate placeholder (bin) with reserved subcommand notice
- Design substrate documents:
  - `docs/origin.md` — narrative post-mortem motivating the project
  - `docs/expedition/design-intent.md` — what antigen IS, what it ISN'T
  - `docs/expedition/api-shape.md` — sketch of API surface
  - `docs/expedition/revolutionary-and-not.md` — honest claims and limits
  - `docs/expedition/team-briefing.md` — for the JBD team at spawn time
  - `docs/expedition/failure-class-instances.md` — real-world Rust ecosystem instances
    of the 8 first-principles failure classes
  - `docs/expedition/ecosystem-composition.md` — composition opportunities with existing
    Rust tools
  - `docs/expedition/academic-context.md` — relationship to existing academic work
  - `docs/expedition/inheritance-from-tambear.md` — disciplines and patterns inherited
    from the tambear project
- Foundational ADRs (ADR-001 through ADR-008) ratified by Tekgy + Claude in pre-team
  scaffolding
- `docs/glossary.md` — vocabulary anchor
- `docs/process.md` — formal ADR lifecycle and governance (how decisions get drafted,
  reviewed, ratified, and govern downstream work; inherited from tambear DEC discipline
  and adapted for antigen)
- `docs/vision-pitch.md` — 1500-word ecosystem-outreach pitch
- `docs/expedition/case-study-determinism-class.md` — pseudocode walkthrough of how
  antigen would have caught the originating bug pattern (closes the loop origin.md
  opens)
- `docs/expedition/stdlib-seed-antigens.md` — 10 concrete antigen declarations for
  the eventual `antigen-stdlib` v0.1 catalog
- `docs/expedition/first-sweep-plan.md` — concrete plan for Sweep A1 (design
  ratification + scope-lock for Sweep A2)
- `docs/expedition/risk-register.md` — adversarial-perspective catalog of what
  could kill the project
- `docs/expedition/conventions.md` — naming, file layout, witness type abbreviations
- ADR-009 (Adoption gradient — antigen meets consumers at any discipline level) and
  ADR-010 (Fingerprint grammar v1 — syn-based AST visitor pattern) ratified into
  `docs/decisions.md`
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md` — open-source hygiene
- `.github/workflows/ci.yml` — CI scaffolding (cargo check + test + fmt + clippy + doc)
- `.github/workflows/release.yml` — release workflow (git-tag-triggered crates.io
  publish + GitHub release)
- GitHub issue templates and PR template
- 9 starter campsites for the future antigen JBD team

### Reserved

- Crate name `antigen` on crates.io
- Crate name `cargo-antigen` on crates.io
- Org name `antigen-rs` on github.com
- Repository name `antigen-rs/antigen` on GitHub

[Unreleased]: https://github.com/antigen-rs/antigen/compare/v0.1.0-rc.1...HEAD
[0.1.0-rc.1]: https://github.com/antigen-rs/antigen/releases/tag/v0.1.0-rc.1
[0.0.1]: https://github.com/antigen-rs/antigen/releases/tag/v0.0.1

# Sweep A2 — Core Macros + Scan/Audit Completion

> **Status**: Scope-locked draft. Pending team-lead ratification.
> **Owner**: pathmaker (with adversarial pressure-tests + scientist validation).
> **Predecessor**: Sweep A1 (design ratification).
> **Successors**: A3 (cross-crate scan + descended_from), A4 (composition rules),
> A5 (vaccinate + audit completeness + stdlib antigens), A6 (ergonomics + IDE).

---

## Theme

**Bring the v0.1 substrate from "functional skeleton" to "release-grade core"**:
property tests over the macros, idiomatic refinement of scan/audit, the four
load-bearing TODO(team) corrections, structural commitments from ADR-007 that
were stubbed in pre-team scaffolding, and a publishable v0.1 milestone that
both the project itself and tambear can stand on.

**This sweep is NOT greenfield.** Pre-team scaffolding + Sweep A1 in-flight
work shipped:
- Four macros (`#[antigen]`, `#[presents]`, `#[immune]`, `#[descended_from]`)
  as identity transforms with attribute-arg validation via syn
  (`antigen-macros/src/lib.rs` + `parse.rs`)
- `scan` module with `syn::visit::Visit` walker, item-kind awareness, and
  proximity-based presentation-immunity matching. **Sweep A1 ATK-001-2 fix
  landed**: scan-time arg parsing migrated from string-manipulation
  (`parse_kv`/`split_top_level_commas`) to `syn::parse2` with
  `ScanAntigenArgs` / `ScanImmuneArgs` parsers using `syn::LitStr` —
  correctly handles inner-quoted fingerprint content (`antigen/src/scan.rs`).
- `audit` module with witness validation, function-index walk, and external-
  tool delegation detection (`antigen/src/audit.rs`). `WitnessKind` enum
  recognizes Test/Proptest/Function; external-tool detection covers clippy,
  kani, prusti, creusot, verus, cargo-mutants. **Phantom-type witnesses are
  not yet recognized** (gap → ADR-013 amendment draft).
- `cargo antigen scan` + `cargo antigen audit` subcommands with human + JSON
  output, `--strict` exit codes (`cargo-antigen/src/main.rs`). Output already
  references `#[antigen_tolerance(...)]` as a remediation path even though
  the macro is undefined (gap → ADR-011 draft).
- Two examples (`basic.rs` + `broken_witness.rs`); 10 unit tests across scan
  + audit; CI gates (fmt, clippy -D warnings + pedantic + nursery, doc, test)
- Tambear adopting antigen as path dep with three seed antigens — including
  `UlpDistanceRolledByHand` (active, 2 sites cleaned up). Adoption log
  surfaces a v1 grammar gap: function-body patterns aren't expressible
  (gap → ADR-012 amendment draft for v2 grammar in Sweep A4-A5).

A2 sharpens what's there, completes structural commitments that ADR-007
guarantees, absorbs the Sweep A1 amendments + new ADR drafts (ADR-011,
ADR-013), and produces a v0.1.0 release. Two structural-blindness ADRs
(ADR-012, ADR-014) are deferred to Sweep A4+ but explicitly named so v0.1
ships with honest known limitations.

---

## Blockers

A2 launches when Sweep A1 closes. Specifically:

- **ADR-001 amendment 1** ratified — carrier-strength hierarchy, passive/active
  surfaces, structural commitments C1-C8, witness-validity tiers,
  ergonomic-maintenance pressure, expanded Related field. The amendment makes
  implicit structural commitments explicit and is the spine A2 implements
  against.
- **ADR-002 amendment 1 (= ADR-013 draft)** ratified — phantom-type witness
  recognition + witness-validity tier reporting. A2's W7 (witness pluralism
  completion) requires this.
- **ADR-005** ratified-as-amended (per adversarial ATK-005-1 phasing) —
  sub-clause F at trust boundaries is the invariant `audit` enforces;
  amendment phases enforcement claims to match actual implementation state.
- **ADR-007** ratified-as-amended (per adversarial ATK-007-1 termination
  criterion) — anti-YAGNI commitments determine which witness types A2 must
  ship; amendment names which features are required for Layer 1 adoption
  vs. v2+.
- **ADR-009** ratified-as-amended (per adversarial ATK-009-1, ATK-009-2) —
  adoption gradient; amendments cover mixed-layer audit semantics +
  ADR-registry validation behavior.
- **ADR-010** ratified-as-amended (per adversarial ATK-010-2) — fingerprint
  grammar v1 decision; amendment removes speculative performance estimates,
  acknowledges the structural-blindness pair (ADR-012 + ADR-014) deferred
  to A4-A5.
- **ADR-011** ratified — `#[antigen_tolerance(...)]` opt-out. A2's W6 grammar
  ships the macro alongside the parser; cargo-antigen output already names
  the macro as a remediation path so the substrate has already silently
  committed.

A1 surfaced four new/amendment ADR drafts. A2 absorbs ADR-011 and ADR-013;
defers ADR-012 (function-body grammar) and ADR-014
(`#[antigen_generates]`) to Sweep A4-A5 with honest documentation of the
v0.1 limitation.

**Non-blocker but informative**: A1's adversarial ATKs on ADR-005 and ADR-009
will likely surface witness-validation edge cases A2 must address (e.g.,
witnesses inside `proptest!` blocks, witnesses that are `impl` methods rather
than free functions). These become work-stream sub-items, not new sweeps.

---

## Unlocks

When A2 closes, the following become possible:

- **A3 — Cross-crate scan + `#[descended_from]` propagation**: A2 ships the
  per-item match story, A3 lifts it across crates and through inheritance
  chains. A3's blocker is A2's W3 (item-identity matching) landing.
- **A4 — Composition rules + witness-type pluralism completion**: if A2 ships
  the four witness types as recognized-but-stubbed-where-needed, A4 wires the
  full delegations (clippy, kani, prusti, proptest harness invocation).
- **A5 — `cargo antigen vaccinate` + audit-extension + stdlib antigens v0.1
  with all 8 first-principles classes**: A2 stabilizes the macro arg surface
  so stdlib declarations are stable; A5 populates.
- **v0.1.0 release on crates.io** (anti-squat refresh + first real version):
  A2 closes with the version bump, changelog cut, and tag.

---

## ADRs ratified or implemented

A2 implements (does not ratify) the following:
- ADR-001 (structural memory mechanism — macros + scan + audit are the carrier)
- ADR-002 (compose, don't compete — the witness external-tool detection)
- ADR-005 (sub-clause F — audit's well-formedness check)
- ADR-007 (anti-YAGNI — all four witness kinds shipped, not just `#[test]`)
- ADR-008 (named-observer terminus — error messages, scan/audit ergonomics)
- ADR-009 (adoption gradient — Layer 1/2 fields parsed, Layer 3 deferred to A3
  if cross-crate ADR registry validation lands there)

A2 may *surface* new ADR drafts (e.g., for a `#[antigen_tolerance]` opt-out
mechanism if adversarial's A1 attacks demand one, or for proptest witness-
detection rules if the heuristic in `audit.rs` proves brittle). New ADR drafts
go through the lifecycle in `docs/process.md`.

---

## Work-streams

Each work-stream is independently committable. Naming follows W*N* per
sweep convention; cross-stream dependencies named explicitly.

### W1 — Property tests over both parser surfaces

**What**: Both `antigen-macros::parse::AntigenArgs/ImmuneArgs/...` and
`antigen::scan::ScanAntigenArgs/ScanImmuneArgs` are now first-class
parsers (the latter landed in ATK-001-2 fix, replacing the brittle
string-manipulation path). The two MUST produce identical results for the
same input — they share the attribute grammar, with the proc-macro side
emitting compile errors and the scan side aggregating into reports.

**Property tests** (cross-parser equivalence):
- For any valid attribute body, `AntigenArgs::parse(body) ==
  ScanAntigenArgs::parse(body)` semantically (same name, fingerprint,
  family, summary).
- For any input that one parser rejects, the other must reject too (the
  parsers cannot disagree on validity without producing scan/macro drift).

**Property tests** (per-parser):
- Round-trip: `Args::parse(args.to_string())` reconstructs equivalent args
  for any valid input
- Order-invariance: argument order doesn't affect parse result
- Required-field enforcement: any input missing `name` or `fingerprint`
  produces a `syn::Error` whose message names the missing field
- Kebab-case validation: `is_kebab_case` accepts/rejects the right strings
- Reject-unknown-field on macro side: `#[antigen(unknown = "x")]` produces
  an error with "unknown" + suggested-field message
- Tolerate-unknown-field on scan side: scan parser silently consumes
  unknown fields (forward-compat for new fields added to the grammar)

**How**: `proptest` dev-dependency. Files: `antigen-macros/tests/parse_props.rs`
(per-parser); `antigen/tests/parser_equivalence.rs` (cross-parser equivalence).

**Why now**: ADR-005's sub-clause F applies to both parsers. ADR-001 amended
C5 makes drift-detection load-bearing — and parser drift between macro and
scan would silently corrupt audit reports (which is exactly what
ATK-001-2 was). Property tests lock the equivalence in code.

**Adversarial check**: After W1 lands, adversarial generates parser inputs
designed to break invariants (Unicode in names, nested macros in fingerprints,
extremely long string literals, malformed array literals, kanji in
references field, `\u{0000}` null bytes in strings). Findings filed as
ATK-W1-N annotations and fed back into W1.

**Estimated**: 1-2 sessions.

---

### W2 — trybuild fixtures for proc-macro errors

**What**: Add `trybuild` test harness exercising the macros' compile-error
paths. Fixtures in `antigen-macros/tests/ui/`:

- `empty_name.rs` + `.stderr` — `#[antigen(name = "", fingerprint = "x")]`
  must reject with "name cannot be empty"
- `non_kebab_case_name.rs` — `#[antigen(name = "FooBar", fingerprint = "x")]`
  must reject with kebab-case-required message
- `missing_fingerprint.rs` — `#[antigen(name = "x")]` must reject
- `non_unit_struct_target.rs` — `#[antigen(...)] pub struct X(u32);` must
  reject with "must be applied to a unit struct"
- `immune_without_witness.rs` — `#[immune(X)]` (no witness) must reject
  with the witness-required message
- `unknown_antigen_field.rs` — `#[antigen(name = "x", fingerprint = "y",
  bogus = "z")]` must reject with "unknown" + suggested-field message

**How**: `trybuild` dev-dependency on `antigen-macros`. New
`antigen-macros/tests/compile_fail.rs` discovering `tests/ui/*.rs` fixtures.

**Why now**: `testing-patterns.md` describes the trybuild pattern; macros
ship without fixtures today. Error messages are the named-observer-stratum
(ADR-008) experience for misuse — they must be tested as carefully as
correct-path behavior.

**Cross-cutting**: When W4 (span-aware errors) lands, W2's fixtures will
need `.stderr` regenerations. Order: W2 first (capture current error
shapes), W4 second (refine, regenerate).

**Estimated**: 1 session.

---

### W3 — Item-identity matching in scan

**What**: Replace the 20-line proximity heuristic in
`ScanReport::unaddressed_presentations` with structural item-identity
matching. The current code matches presentations to immunities by file +
line proximity, which is loose and brittle (multi-attribute stacking,
nested impl blocks, multiple impls in one file all stress it).

Concrete changes:

- Augment `Presentation` and `Immunity` with `item_target: ItemTarget` (an
  enum: `Impl(impl_target_path)`, `Fn(fn_path)`, `Struct(struct_name)`,
  `Trait(trait_name)`, `ImplFn(impl_target_path, fn_name)`)
- During AST walk, capture the item target: for `impl Drop for VulnerableType`,
  record `Impl("VulnerableType", "Drop")`; for free `fn meet`, record
  `Fn("meet")`; etc.
- `unaddressed_presentations` matches by `item_target` + `antigen_type`
  rather than by file/line proximity
- Backwards-compat: `line` field stays for diagnostic output; matching no
  longer depends on it

**Why now**: TODO(team) marker at `scan.rs:127`. Real workspaces will hit
the proximity heuristic's failure modes before A2 closes (multi-impl files
exist in tambear). The fix is mechanical and substantially improves accuracy.

**Adversarial check**: ATK targets — what about (a) trait method impls
where presents lives on the trait method but immune on the impl method?
(b) generic impls with multiple instantiations? (c) cfg-conditional impls?
The structural matching should handle (a) cleanly, surface (b) as a
known gap (ATK to A3 cross-crate work), and (c) requires test fixture.

**Estimated**: 2-3 sessions (matching is per-item-kind; visitor needs
augmenting in 5 places).

---

### W4 — Span-aware error messages in macro parser

**What**: Replace `Span::call_site()` with token-precise spans in
`AntigenArgs::validate` and `ImmuneArgs::validate` so error squiggles
underline the offending literal, not the whole macro invocation.

Concrete changes:

- `AntigenArgs` and `ImmuneArgs` carry `Span`s on each parsed field (e.g.,
  `name_span: Span`, not just `name: String`)
- `validate()` produces errors anchored to those spans
- TODO(team) marker at `parse.rs:92` is removed when this lands

**Why now**: ADR-008 (named-observer terminus). Bad error spans are the
single biggest first-impression hit for a proc-macro. Cheap to fix;
expensive UX cost while broken.

**Sequencing**: After W2 (trybuild fixtures exist; we know what error text
looks like) so we don't lose `.stderr` snapshots. Before v0.1.0 release.

**Estimated**: 1 session.

---

### W5 — `proptest!` witness detection (audit)

**What**: The current `FunctionIndexVisitor::detect_kind` uses textual
`source.contains("proptest!")` as a sentinel, which over-matches (any file
mentioning `proptest!` in a comment or doctest gets every function classified
as `Proptest`). Replace with structural detection: walk macro invocations
during the function-index pass, identify `proptest! { ... #[test] fn name }`
expansions, and tag `name` as `WitnessKind::Proptest`.

Concrete changes:

- New `FunctionIndexVisitor::visit_macro` (or per-item walk) recognizing
  `proptest!` invocations
- Track which functions live inside a `proptest!` body during the walk
- Resolve `WitnessKind::Proptest` per-function based on enclosing-macro
  context, not file-level text presence

**Why now**: TODO(team) marker at `audit.rs:245`. ADR-002 (compose, don't
compete) commits us to recognizing proptest as a first-class witness
mechanism. Mis-classification is silent failure (audit reports a
test-derived witness as a proptest when both work; reports a generic fn as
proptest when neither does).

**Cross-cutting**: A new ADR may be needed if we discover `proptest!` macro
hygiene makes structural detection unreliable. Path: file ATK during
adversarial review; if structural detection can't be made robust on
stable Rust, propose ADR for "witness hint via attribute" (e.g.,
`#[antigen_witness(proptest)]`) as opt-in disambiguation.

**Estimated**: 2 sessions (proptest macro shape walk is non-trivial).

---

### W6 — Fingerprint grammar v1 + `#[antigen_tolerance]` (ADR-011)

**What**: Ship the v1 fingerprint grammar parser per ADR-010, **and** ship
`#[antigen_tolerance(...)]` per ADR-011, **and** ship the
recognition-not-yet-marked synthesis pass.

ADR-010 specifies a syn-based AST visitor pattern with: type-name patterns
(glob), struct/enum/trait kind matchers, attribute presence checks,
field/variant shape matchers, method-signature patterns, composition
operators (`all_of`, `any_of`, `not`).

A1's outcome decided **Option B (ship in A2)**: deferring leaves the
audit/scan story incoherent, and ADR-007 anti-YAGNI commits us. Honest v1
limitations are explicitly documented (no function-body patterns — that's
ADR-012 v2 grammar in A4-A5; no macro-output recognition — that's ADR-014
in A4-A5).

**Concrete changes**:
- New module `antigen::fingerprint` with `Fingerprint` struct,
  `parse_fingerprint(&str) -> syn::Result<Fingerprint>`, and
  `Fingerprint::matches(item: &syn::Item) -> bool`.
- Six operators implemented: `item: <kind>`, `name: matches(<glob>)`,
  `variants: <range>`, `has_method(<name>, <signature>)`,
  `attr_present(<path>)`, `doc_contains(<substring>)`, plus `all_of`,
  `any_of`, `not` composition.
- Scan pass extended: in addition to collecting explicit
  `#[antigen]`/`#[presents]`/`#[immune]` declarations, walk every item
  and check against every declared antigen's `fingerprint`. Emit
  synthetic `Presentation` for matches with `match_kind: ItemMatch`
  (per ADR-001 amendment 1 Change 2 — passive surface).
- `#[antigen_tolerance(X, rationale = "...", until = "...")]` macro added
  to `antigen-macros` with arg validation (rationale required, antigen
  type required); identity transform.
- Scan recognizes tolerance markers as explicit acknowledgments of
  fingerprint matches; `cargo antigen scan` reports tolerated matches in
  a separate category.
- `print_human_report` updated: separate counts for "explicit
  presentations" / "fingerprint matches (passive)" / "tolerated matches"
  / "unaddressed" — the four states from ADR-001 amendment 1 Change 2.

**Why now**: ADR-007 commits us; ADR-010 + ADR-011 ratified during A1;
the tooling already names `#[antigen_tolerance]` as remediation in
`main.rs:185` — the substrate has the commitment but lacks the
implementation.

**Adversarial check**: ATKs to expect — over-broad fingerprints causing
autoimmunity at scale (per ATK-001-1; mitigation: scan warns when
fingerprint matches >X% of items); circular fingerprints (ATK-fingerprint-
recursion); rationale-stuffing on tolerance (ATK-tolerance-1).

**Estimated**: 4-6 sessions. The largest individual work-stream in A2;
budget upfront.

---

### W7 — Witness pluralism completeness (ADR-007 + ADR-013 amendment)

**What**: Implements ADR-013 (ADR-002 amendment 1 — phantom-type witness
recognition + witness-validity tier reporting). Today, audit recognizes:

- `#[test]` — yes (`WitnessKind::Test`)
- `proptest!` — yes (textual heuristic; W5 makes it structural)
- Clippy lints — partial (`detect_external_tool` recognizes `clippy::`)
- Kani — partial (`kani::`)
- Prusti — partial (`prusti::`)
- Creusot — partial (`creusot::`)
- Verus — partial (`verus::`)
- Cargo-mutants — partial (`mutants::`)
- **Phantom-type witnesses** — *not recognized at all*

**Concrete changes** (per ADR-013):

- Extend `WitnessKind` with `PhantomType { proof_type, type_params,
  constructor }` — captures the structure of phantom-type witness paths.
- New helper `detect_phantom_type_witness(expr) -> Option<PhantomTypeRef>`
  recognizes the `Path::<TypeParams>::constructor` shape.
- `validate_witness` priority order: external-tool delegation → phantom-
  type detection → function-index lookup → `NotFound`.
- Audit emits witness-validity tier per ADR-001 amendment 1 Change 4:
  Reachability / Execution / Behavioral-alignment (deferred) / Formal-proof.
  JSON output includes `witness_tier` field for CI gates.
- Recognize-and-warn for phantom-type witnesses: audit reports them as
  `Resolved` but adds a hint ("phantom-type witness — verify the
  constructor encodes a real proof").
- New example file `antigen/examples/phantom_witness.rs` demonstrating
  the construction-encodes-proof pattern.

**Adversarial check**: ATK targets — phantom-type witnesses can be
constructed with deliberately-impossible bounds that still compile (e.g.,
where a type parameter is unused). Audit must distinguish "trivially
constructible phantom-type witness" (red flag) from "construction-encodes-
proof" (the real pattern). For v0.1.0, recognize-but-warn is acceptable
(per ADR-013 §Open Question 1); construction-validation is a future ADR.
Witness type-parameter mismatch (witness `PolarityProof::<FrameTranslation>`
on antigen `BoundaryViolation`) gets flagged.

**Estimated**: 2-3 sessions.

---

### W8 — Idiomatic refinement pass

**What**: One pass through the codebase looking for clippy pedantic +
nursery findings that were silenced or worked-around during pre-team
scaffolding, plus general Rust idiomaticity improvements:

- `String` → `&str` where ownership isn't needed
- `Vec<String>` → `Vec<Cow<'_, str>>` where transient
- Replace `unwrap_or` calls that produce `String::new()` with `unwrap_or_default`
- `match` blocks that should be `if let` (and vice versa)
- `pub fn` that should be `pub const fn`
- Doc-comments that should link via intra-doc links (`[\`Type\`]`)
- Module-level `#![allow]`s that should be expression-level
- Re-export shape: should `antigen-macros` be a workspace member exposed
  to users, or strictly an internal? (Currently exposed but not published
  separately; decide.)

**How**: Run `cargo clippy --workspace --all-targets -- -D warnings -W
clippy::pedantic -W clippy::nursery -W clippy::cargo` and address each
finding. Same for `cargo doc --workspace --no-deps -D warnings`.

**Why now**: ADR-008 (named-observer terminus). Code that ships to
crates.io must be idiomatic; idiomaticity drift is a bug like any other.
A2 closes with v0.1.0 release — last chance to scrub before the API
surface stabilizes.

**Constraint**: This work-stream does NOT add features. Any refactor that
crosses module boundaries gets discussed with team-lead before commit.

**Estimated**: 1-2 sessions.

---

### W9 — v0.1.0 release prep

**What**: At sweep close, cut the v0.1.0 release:

- Bump `Cargo.toml` `[workspace.package].version` from `0.0.1` to `0.1.0`
- Update `CHANGELOG.md` with the v0.1.0 section naming the four macros,
  scan, audit, examples, and known gaps (descended_from semantics, cross-
  crate scan, vaccinate, stdlib antigens) as future-sweep work
- README revision pass (current README references "0.0.1 placeholder";
  update to "0.1.0 — first release of the core macros + scan + audit")
- Tag-driven release workflow at `.github/workflows/release.yml` runs on
  `v0.1.0` tag (verify the workflow exists and is correct)
- Verify both `antigen` and `cargo-antigen` publish cleanly with
  `cargo publish --dry-run`

**Why now**: A2 is the substrate-completion sweep; releasing v0.1.0 is
the externalization of that completion. Tambear and other early consumers
benefit from a stable version pin (currently they path-dep, which works
locally but doesn't scale to other adopters).

**Estimated**: 1 session, mostly mechanical.

---

## Integration milestones

The work-streams converge at three checkpoints. Pathmaker calls these out
to navigator at the relevant inflection points; navigator routes
adversarial/scientist for cross-cutting checks.

### Milestone A — Property-test floor (after W1 + W2)

The property-test + trybuild infrastructure is in place. Subsequent work
(W3 onward) is gated by "must not regress W1's properties or W2's
fixtures." Adversarial gets a dedicated session at this point to attack
the parser before more code rests on it.

**Exit criteria**: 10+ properties on macro parser (5+ each on `AntigenArgs`,
`ImmuneArgs`); 6+ trybuild fixtures covering the named error paths; CI
gate: property tests + trybuild run on every push.

### Milestone B — Item-identity + span correctness (after W3 + W4 + W5)

The scan and audit modules now reason about *items* and *spans*, not
proximity and call_site. The "structural memory" claim (ADR-001) becomes
load-bearing: scan/audit now perform the structural reasoning the
declarations describe.

**Exit criteria**: Existing 10 unit tests + new property tests still pass;
scan correctly handles 3 fixture cases (multi-impl file, nested traits,
impl method witnesses); audit correctly classifies a real `proptest!`
block in fixture; tambear-adoption-log entry recording the W3-W5
upgrade against tambear's real codebase (217 files).

### Milestone C — Witness pluralism + fingerprint grammar (after W6 + W7)

The witness story is ADR-007-complete (all four witness families
recognized); fingerprint grammar v1 is shipped (recognition-not-yet-marked
half of scan exists). v0.1.0 is releasable after W8 idiomatic pass + W9
release prep.

**Exit criteria**: All four witness types have a worked example in
`antigen/examples/`; fingerprint parser shipped with 6+ documented
operators; scan flags 3+ fingerprint-matching unmarked sites in a
constructed fixture; tambear-adoption-log entry recording fingerprint
matching against tambear's class enums (expected: 0 matches today,
because tambear's class enums don't have meet methods, but the scan
output should *report 0 matches with confidence*, demonstrating the
recognition pass works).

---

## Estimated duration

15-20 sessions of pathmaker time, spread across 3-4 weeks of
calendar-walked work. Adversarial pressure-tests, scientist validation,
and naturalist roams happen in parallel and don't extend the critical
path.

Sessions per work-stream:
- W1: 1-2
- W2: 1
- W3: 2-3
- W4: 1
- W5: 2
- W6: 3-5 (recommend Option B; budget upfront)
- W7: 2
- W8: 1-2
- W9: 1
- Adversarial passes: 1-2 across the sweep
- Scientist validation passes: 1-2 across the sweep
- Naturalist roams + closure narrative: 1

Total: ~17-22 sessions. The wide range reflects W6's decision point.

---

## What A2 does NOT do

Explicit out-of-scope items, deferred to later sweeps:

- **Cross-crate `#[descended_from]` propagation** → A3. The semantics of
  walking inheritance chains across crate boundaries (signature divergence
  detection, witness re-validation) is a sweep of its own.
- **`cargo antigen vaccinate`** → A5. Bulk pattern application across
  structural families requires the fingerprint grammar to be stable and
  the witness library to be complete. Both happen in A2; vaccinate
  consumes them in A5.
- **Cross-crate antigen import + ADR-009 Layer 3 (`adr` field validation)**
  → A3. Cross-crate antigen consumption requires the fingerprint grammar
  to be defined (W6 in A2) but the cross-crate semver/registry semantics
  is its own design problem.
- **Function-body fingerprint patterns (ADR-012 amendment)** → A4-A5. The
  v1 grammar is item-shape-only. Body-level patterns (`body_contains`,
  `body_pattern`) require deeper visitor work and a body-pattern library;
  deferred. v0.1.0 ships with documented limitation: scan misses
  body-shaped failure-class re-emergences (per tambear's
  `UlpDistanceRolledByHand` adoption-log finding).
- **`#[antigen_generates]` for proc-macro authors (ADR-014)** → A3 (same-
  workspace) or A4 (cross-crate). The macro-output structural-blindness
  pair with ADR-012; v0.1.0 ships without it. Workspaces using third-
  party derives have a known blind spot until v0.2+.
- **Witness behavioral-alignment validation (ADR-001 amendment 1
  Change 4 + ADR-005 OQ)** → future ADR + Sweep A4-A5. v0.1 audit lives
  at the reachability tier; behavioral alignment requires reasoning about
  what the witness's body asserts and is a deeper design problem.
- **Stdlib antigens crate (`antigen-stdlib`) population** → A5. The
  scaffolding (Cargo.toml, workspace member) may land in A2 if free; the
  population of all 8 first-principles classes is A5 work.
  **Named v0.1 stdlib candidates** (per recognition discipline of ADR-006;
  three or more independent real-world instances confirmed):
  - **`frame-translation` / `polarity-inverted-class-meet`** — the
    failure-class motivating the project (per origin.md). Three independent
    instances now documented: tambear `DeterminismClass` (GAP-BIT-EXACT-1),
    tambear `CommutativityClass` (DEC-030 ATK-DEC030-2), and ULP-CANON-1
    (per tambear adoption log entry 2026-05-07). The most empirically-
    grounded antigen the project has; A5 ratifies its v0.1 stdlib
    inclusion. A2 prepares the substrate (W6 fingerprint grammar — note
    the discriminant-ordering operator this antigen needs is explicitly
    deferred to A5 per ADR-010 amendment 3 Clause C).
  - **`panicking-in-drop`** — canonical seed antigen demonstrating the
    `body_contains_macro` operator (which A2 W6 ships per ADR-010
    amendment 3 Clause C). Exercises the operator end-to-end before A5
    populates the full taxonomy.
- **rust-analyzer plugin / IDE integration** → A6. Sweep A2 ships ergonomic
  CLI; A6 ships ergonomic editor.
- **`cargo antigen test`** (run only witness functions) → A6 or later.
  Useful for antigen-library development; not load-bearing for v0.1.0.
- **Anti-squatting version-bump cadence** → process concern, addressed
  via release workflow pattern, not in-sweep work.

---

## Closure criteria

A2 closes when:

- All work-streams W1-W9 land (any deferred items have explicit defer-
  ratification with rationale)
- Property tests + trybuild fixtures run green in CI
- Item-identity matching replaces proximity heuristic; tambear scan
  results re-validated against the new matching
- Span-aware error messages ship; trybuild fixtures regenerated
- Witness pluralism complete (4 families recognized)
- Fingerprint grammar v1 shipped (Option B) OR explicitly deferred to A3
  with an ADR-010 amendment ratifying the deferral (Option A — not
  recommended)
- Idiomatic pass complete; clippy/doc gates clean
- `v0.1.0` tagged and published to crates.io
- Tambear adoption log updated with v0.1.0 upgrade entry
- Naturalist closure narrative written
- Sweep A3 scope-lock README begun (drafting starts when A2's W3+W6
  surface what A3 needs to absorb)

---

## Risk register

Surfaced risks with mitigation strategies:

1. **W6 expands scope.** Mitigation: hard-cap fingerprint grammar at
   ADR-010's v1 surface. Function-body patterns and tree-sitter integration
   are explicitly v2 (deferred to A4 amendment process).

2. **trybuild snapshots regenerate frequently during W4.** Mitigation:
   sequence W2 before W4; W2 captures current snapshots, W4 regenerates
   under controlled conditions (clear "regenerated for span work" commit).

3. **ADR-007 phantom-type witness recognition is harder than expected.**
   Mitigation: scope W7 to recognize-and-warn shape (don't try to validate
   phantom-type proof correctness in v0.1.0). If audit can't distinguish
   trivial from real phantom-type witnesses, file as known-gap and lift
   in A3+ via ADR.

4. **Adversarial findings during A1 produce ADR amendments that break
   A2 assumptions.** Mitigation: A2 doesn't launch until A1 closes;
   amendments are absorbed into this scope-lock document at A1 closure
   before A2 launch.

5. **CI takes too long with property tests + trybuild + 50+ fixtures.**
   Mitigation: parallelize via cargo nextest; aim for <90s CI. If
   property tests exceed budget, sample-down for CI and run full in a
   nightly job.

6. **v0.1.0 release breaks tambear's path-dep adoption.** Mitigation:
   tambear migrates from `path = "..."` to `version = "0.1.0"` as
   smoke-test of the release. Pre-release tag (`v0.1.0-rc.1`) lets
   tambear validate before the proper v0.1.0 tag.

---

## References

- [`docs/decisions.md`](../../docs/decisions.md) — the 10 ADRs A2 implements
- [`docs/process.md`](../../docs/process.md) — sweep + ADR lifecycle
- [`docs/expedition/first-sweep-plan.md`](../../docs/expedition/first-sweep-plan.md) —
  Sweep A1 (predecessor)
- [`docs/expedition/api-shape.md`](../../docs/expedition/api-shape.md) —
  pre-team API sketch (substrate, not authority)
- [`docs/expedition/design-intent.md`](../../docs/expedition/design-intent.md) —
  8-class taxonomy + biological mapping
- [`docs/expedition/tambear-adoption-log.md`](../../docs/expedition/tambear-adoption-log.md) —
  ground-truth adoption signal (especially the W6 grammar gap finding)
- [`antigen/src/scan.rs`](../../antigen/src/scan.rs) — current scan implementation
- [`antigen/src/audit.rs`](../../antigen/src/audit.rs) — current audit implementation
- [`antigen-macros/src/lib.rs`](../../antigen-macros/src/lib.rs) +
  [`parse.rs`](../../antigen-macros/src/parse.rs) — current macro
  implementation
- [`cargo-antigen/src/main.rs`](../../cargo-antigen/src/main.rs) — current
  CLI entrypoint

---

## Open scope-lock questions for navigator + team-lead

Before A2 launches, the following need agreement:

1. **W6: Option A or Option B?** Pathmaker recommends B (ship grammar in
   A2). Team-lead decision.
2. **`antigen-macros` publication independence**: should A2 publish
   `antigen-macros` as a separate crate on crates.io (currently only
   workspace-internal), or keep it as a re-export from `antigen`? Adoption
   gradient question.
3. **Pre-release tag for v0.1.0 (`v0.1.0-rc.1`)?** Recommended for tambear
   smoke-test; team-lead may prefer direct `v0.1.0`.
4. **A1 amendments that affect A2**: at A1 closure, navigator + pathmaker
   walk this scope-lock and absorb any A1 amendments before A2 launch
   sign-off.

---

*Drafted: 2026-05-07.*
*Status: scope-locked draft, pending A1 closure + team-lead ratification.*
*Owner: pathmaker.*

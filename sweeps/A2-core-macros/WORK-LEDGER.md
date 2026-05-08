# A2 Work-Stream Ledger

> Substrate-grounded record of what each A2 work-stream actually shipped,
> commit-by-commit. Pathmaker's contribution to the closure-narrative
> substrate; naturalist's full closure narrative drafts after W9 ships
> (per team-lead's assignment, recorded at
> [`closure-narrative-seed.md`](../../campsites/antigen-design/20260508120021-20260508170000-naturalist-a2/naturalist/20260508-closure-narrative-seed.md)).
>
> This ledger answers: *what does the substrate now contain that it didn't
> at sweep launch?* For each W-stream: the ratifying commit(s), the
> substrate-grounded check that proves it shipped, and the on-disk artifacts.

---

## W1 — Property tests over both parser surfaces

**Ratifying commits**: `6c3cafe` (cross-parser equivalence fixtures
landed earlier in `9ea922e`).

**Substrate-grounded check**:
- `cargo test -p antigen-macros --lib parser_props` → property tests for
  AntigenArgs round-trip, order-invariance, required-field enforcement,
  unknown-field rejection, references array round-trip, ImmuneArgs
  witness, ImmuneArgs missing-witness validation.
- `cargo test -p antigen scan_parser_props` → mirror on the scan side.

**Artifacts**:
- `antigen-macros/src/parse.rs` — `parser_props` mod with 8 proptest!
  invariants
- `antigen/src/scan.rs::tests::scan_parser_props` — mirror module
- `ANTIGEN_PARSER_FIXTURES` shared fixture table (canonical 6 cases)

**Adversarial seed substrate**: `antigen-macros/tests/atk_parser_adversarial.rs`
+ `antigen/tests/atk_parser_scan_adversarial.rs` (filed as part of
adversarial's clippy cleanup pass `5ca45b2`).

---

## W2 — trybuild fixtures for proc-macro errors

**Ratifying commit**: `e321907`.

**Substrate-grounded check**:
- `cargo test -p antigen-macros --test compile_fail` → 10 .stderr fixtures
  green (5 from W2, 1 already existed for `non_unit_struct_target`,
  4 added later by W6a for `#[antigen_tolerance]` paths).

**Artifacts**:
- `antigen-macros/tests/ui/empty_name.rs` + `.stderr`
- `antigen-macros/tests/ui/non_kebab_case_name.rs` + `.stderr`
- `antigen-macros/tests/ui/missing_fingerprint.rs` + `.stderr`
- `antigen-macros/tests/ui/non_unit_struct_target.rs` + `.stderr`
- `antigen-macros/tests/ui/immune_without_witness.rs` + `.stderr`
- `antigen-macros/tests/ui/unknown_antigen_field.rs` + `.stderr`
- `antigen-macros/tests/compile_fail.rs` discovery harness

---

## W3 — Item-identity matching in scan

**Ratifying commit**: `1b4e04a`.

**Substrate-grounded check**:
- `cargo test -p antigen --test atk_w3_item_identity` → 17 tests
  exercising the structural matcher across multi-impl files, nested
  traits, type aliases, and trait/impl-method bridging.

**Artifacts**:
- `antigen/src/scan.rs::ItemTarget` enum (Struct / Enum / Trait / Fn /
  TypeAlias / Impl / ImplFn / TraitFn / Unknown)
- `ItemTarget::addresses()` — the structural-matching invariant
- ScanVisitor `impl_stack` + `trait_stack` for nested-context tracking
- `antigen/tests/atk_w3_item_identity.rs` — full coverage

**Hotfixes that landed alongside**:
- `b9440b2` — extract_presents path-split (ATK-A2-001) + enum no-op
  recording (ATK-A2-007)
- `b358c3f` — path-qualified attr names + adversarial test substrate
  with `#[ignore]` annotations

---

## W4 — Span-aware error messages in macro parser

**Ratifying commit**: `38cff21`.

**Substrate-grounded check**:
- `cargo test -p antigen-macros --test atk_w4_span_contracts` → 6 tests
  reading regenerated `.stderr` fixtures and asserting column-precise
  anchors (was `#[ignore]` panicking shells pre-W4).

**Artifacts**:
- `antigen-macros/src/parse.rs::AntigenArgs` gains `name_span`,
  `fingerprint_span`, `args_span` fields
- `MetaPair::expect_string_spanned()` helper
- `ImmuneArgs::validate()` uses `new_spanned(&self.antigen)` for the
  immune-without-witness path (anchors at the antigen path)
- 4 trybuild .stderr fixtures regenerated under `TRYBUILD=overwrite`
- 6 ATK-W4 contracts pin column-precise anchors as regression guards

**The discipline**: token-precise spans where a token exists; `args_span`
(the macro arg-list span captured at parse time) for missing-required-
field errors where there is no offending token. Consistently better than
`Span::call_site()`.

---

## W5 — `proptest!` witness detection (audit)

**Ratifying commit**: `e472b66`.

**Substrate-grounded check**:
- `cargo test -p antigen --test atk_w5_proptest_contracts` → 5 tests
  pinning W5's behavior including `atk_w5_007_proptest_function_collision_with_free_fn_is_ambiguous`
  (reframed under W7 — see W7 note).

**Artifacts**:
- `antigen/src/audit.rs::FunctionIndexVisitor::visit_macro` — structural
  detection via macro-path + token-walking the proptest! body for
  `fn IDENT` patterns
- `extract_proptest_fn_names()` — the token walker
- `macro_path_last_is()` helper — handles bare `proptest!` and
  path-qualified `proptest::proptest!`
- Removed the pre-W5 textual sentinel `source.contains("proptest!")`
  that over-classified every function in any file mentioning the
  macro (including doc comments)

**The discipline**: structural detection at the macro-invocation level,
not textual scan of the source string.

---

## W6a — Fingerprint grammar item-level operators + `#[antigen_tolerance]`

**Ratifying commits** (5 steps + 1 polish):
- `af4209c` — step 1: `antigen-fingerprint` workspace member + parser
  + matcher
- `e1f3d3c` — step 2: `#[antigen_tolerance]` macro per ADR-011
- `ba44d01` — step 3: compile-time DSL validation in `#[antigen]` +
  substrate migration of examples to DSL form
- `01b7da8` — adversarial pre-impl contracts for W6a (substrate
  bundled mid-stream)
- `a7d22d3` — step 4: scan synthesis pass + tolerance recognition +
  5-state CLI output
- `408a380` — step 5: tolerance-acknowledges-presentation + orphan
  detection (ATK-A2-009 transitions to passing)
- `e5dd2d8` — polish: pre-normalize `has_method` signature at parse
  time per ADR-010 Amendment 3 Performance Invariant 2 (navigator's
  substrate-check)

**Substrate-grounded check**:
- `cargo test -p antigen-fingerprint` → 77 tests pass (parser, matcher,
  glob with proptests, W6a contracts)
- `cargo test -p antigen --test atk_w6a_synthesis` → 2 tests confirming
  synthesis emits + dedupes against explicit markers

**Artifacts**:
- New `antigen-fingerprint/` workspace member with `Fingerprint` AST,
  Path C parser, syn evaluator, glob matcher
- `antigen-macros::antigen_tolerance` macro re-exported from
  `antigen::antigen_tolerance`
- `antigen::scan::MatchKind { ExplicitMarker, FingerprintMatch }`
- `antigen::scan::Toleration` + `ScanReport::tolerances`
- `ScanReport::orphaned_tolerances()` for ATK-A2-009
- `synthesis_pass()` runs after explicit collection with node-kind
  dispatch + dedup
- `cargo-antigen` CLI emits 5-state report (antigen / explicit /
  fingerprint / tolerated / unaddressed)

**Substrate migration in step 3**: examples + ATK fixtures + parser
proptest fuzzers that used prose fingerprints (`fingerprint = "x"`,
`fingerprint = "impl Drop with..."`) migrated to real DSL form per
ADR-010 Amendment 1. The cascade was correct — Amendment 1 ratifies
the DSL form; the prose strings were always going to need migration.

**The performance invariants honored**:
- PI-1 single-pass: scan caches parsed files between pass-1 and pass-2
- PI-2 pre-parsed signatures: `MethodPattern::normalized_signature` Some
  populated at parse time; matcher reads cache
- PI-3 depth/node-count caps: parser rejects depth > 10 or nodes > 256
- PI-4 node-kind dispatch: synthesis pass skips fingerprints whose
  top-level `item:` constraint can't match the current AST node

---

## W6b — Body-level operators (deferred)

**Status**: deferred per ADR-015 §Deferred. Path 2 (ast-grep CLI
subprocess) is the recommendation from both math-researcher's
substrate-correction addendum and aristotle's revising addendum.

**Substrate**: `body_contains_macro` is implemented natively in
`antigen-fingerprint` (no body engine needed) per ADR-015 §S2 and ADR-010
Amendment 3 Clause C. The general `body_pattern("...")` operator awaits
the sibling backend decision.

**v0.1 known limitation**: `UlpDistanceRolledByHand`-shape patterns
(multi-statement function bodies) are documented as not detectable in v1.
Honest documentation per the project's known-limitations posture.

---

## W7 — Witness pluralism + phantom-type recognition + WitnessTier gradient

**Ratifying commits**:
- `ce6b150` — main W7 implementation
- `12210c9` — hotfix: scan-side span precision via `syn::spanned::Spanned`
  (ATK-A2-002 transitioned alongside)
- `97a2b85` — hotfix: phantom-type nested-generic guard in
  `detect_phantom_type_witness`

**Substrate-grounded check**:
- `cargo test -p antigen --test atk_w7_tier_contracts` → 16 tests pinning
  Amendment 3's mapping table (tier ordering, discriminants, status →
  (tier, hint) derivation, serde round-trip, ambiguous collision)
- 6 ATK-A2 transitions from `#[ignore]` to passing: 002 (line_of_attr),
  003 (empty function), 004 (fabricated external), 005 (ambiguous),
  010 (phantom mismatch — recognize-and-warn per ADR-013 OQ1),
  011 (fabricated path prefix — saved by tier honesty), 012 (ignored
  test → IgnoredTest hint)

**Artifacts** (per aristotle's Phase 6 corrected design, honoring
ADR-005 Amendment 3):
- `WitnessTier { None=0, Reachability=1, Execution=2, FormalProof=4 }`
  (3 reserved for BehavioralAlignment)
- `AuditHint` parallel axis with structured variants
- `WitnessKind::IgnoredTest` (anergic-B-cell cognate per naturalist's
  W7 metaphor-fidelity check)
- `WitnessKind::PhantomType { proof_type, type_params, constructor }`
- `WitnessStatus::Ambiguous { candidates }` for collision detection
- `FunctionIndex` → `HashMap<String, Vec<FunctionEntry>>` so collisions
  surface rather than silently picking one
- `ImmunityAudit::is_well_formed() = meets_tier(Execution)` — the
  honest definition (Reachability is not well-formed)
- `detect_kind` distinguishes `#[test] #[ignore]`
- `detect_phantom_type_witness` recognizes turbofish shapes; nested-
  generic `Foo::<Bar<Baz>>::new` constructor extraction guarded

**The reframe**: scout's pre-Amendment-3 design carried 5 silent
deviations. Aristotle's Phase 1-8 ruling caught all 5; pathmaker
implemented the corrected design verbatim. ATK-W5-007 reframed: under
W7's collision detection, proptest-vs-free-fn name shadow is correctly
`Ambiguous`, not "Proptest wins precedence."

**ATK-W5-007 Proptest-priority refinement** (deferred): navigator
flagged a refinement to the `many` arm of `validate_witness` —
Proptest > Test > Function priority when one candidate is structurally
stronger. Not yet shipped; tracked for follow-up.

---

## W8 — Idiomatic refinement pass

**Ratifying commits**:
- `a519d53` — chore: `#[must_use]` on `keyword`/`matches`/`contains` in
  antigen-fingerprint
- `7449eba` — W8: idiomatic refinement pass (scan_workspace `# Errors`
  doc; redundant module-level allow removed)

**Substrate-grounded check**:
- `cargo clippy --workspace --all-targets -- -D warnings` → exits 0
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` → exits 0
- `cargo test --workspace` → 183 passing, 20 ignored

**Findings closed**:
- 3 `must_use_candidate` annotations in antigen-fingerprint
- `scan_workspace` missing `# Errors` doc — added an honest section
  documenting that the function never returns Err in v0.1 (IO failures
  silently skip; parse failures land in `parse_failures`)
- Redundant `#![allow(clippy::module_name_repetitions)]` in
  antigen-fingerprint removed (workspace `[lints]` already allows it)

**Allowed-by-design** (per workspace `[lints.clippy]`):
- `uninlined_format_args` and `needless_pass_by_value` remain `allow`
  per the workspace ergonomic decisions; they fire under explicit
  `-W clippy::pedantic` but are not part of the CI gate.

**Re-export decision** (per W8 contract): `antigen-macros` and
`antigen-fingerprint` stay workspace-internal, re-exported from
`antigen`. The evaluator-trait public-vs-private question (per
ADR-015 §S3) is reserved for the second-backend ratification.

---

## W9 — v0.1.0 release prep

**Status**: pending. Closure criteria: tag `v0.1.0`, `cargo publish`
clean for both `antigen` and `cargo-antigen`, README revision pass,
CHANGELOG, tambear adoption-log entry.

**Pre-conditions met**: W1-W8 all shipped; CI gates clean; example
crates compile against DSL fingerprints.

---

## Cross-stream artifacts

These shipped during A2 but aren't strictly bound to a single W-stream:

- `docs/postures.md` (V0, 7 entries) — aristotle's authored substrate;
  depth-shift discipline as posture #7 with the recognition-recursion
  pattern documented
- `docs/glossary.md` updates — `WitnessTier`, `AuditHint`, `IgnoredTest`,
  `MatchKind`, `Toleration`, "depth-shift discipline"
- `docs/decisions.md` amendments ratified in `817afd0` (A2 ratification
  commit): ADR-001 Amendment 1 + ADR-005 Amendment 3 + ADR-008
  Amendment 1 + ADR-010 Amendments 1-4 + ADR-011-016
- ATK-A3 fractal preview substrate: `atk_a3_fractal_preview.rs` —
  pre-implementation contracts for A3 territory (cycle detection,
  re-export resolution, stale parent reference, proc-macro generated
  witness)

---

## Test count trajectory

| Milestone | Passing | Ignored | Notes |
|---|---|---|---|
| Pre-A2 (sweep launch) | 10 | 0 | scan + audit unit tests |
| W1 land | ~25 | 0 | property tests added |
| W2 land | ~31 | 0 | trybuild fixtures |
| W3 land | ~48 | 0 | item-identity coverage |
| W4 land | 70 | 29 | W4 contracts transitioned |
| W7 land | 101 | 18 | 6 ATK-A2 transitions, +17 W7 contracts |
| W6a complete | 183 | 20 | +77 antigen-fingerprint, +scan synthesis |
| W8 land | 183 | 20 | no test changes; only lint/doc fixes |

The trajectory is monotone-up on the passing axis and monotone-down on
the ignored axis — the pre-implementation-contract discipline is doing
real work. Each W-stream landed transitioned its own pre-impl
contracts; no stream introduced new perma-ignored tests.

---

## Substrate notes for naturalist's closure narrative drafter

When A2 actually closes (after W9 ships), the closure-narrative drafter
inherits this ledger as ground truth for *what shipped*. The seed at
`closure-narrative-seed.md` is the framing spine. The two together are
the closure narrative's substrate.

A few points worth threading into the narrative that may not be obvious
from the seed alone:

1. **The W6a step-3 substrate cascade** is a clean case study in
   substrate-honest design. Making compile-time DSL validation strict
   in `#[antigen]` broke examples + ATK fixtures + the parser fuzz
   harness — and the fix in every case was to migrate to the ratified
   DSL form, not to soften the validation. The cascade was the right
   shape; it forced the migration ADR-010 Amendment 1 had ratified to
   actually land in the substrate.

2. **The ATK-W5-007 reframe** under W7 is the cleanest substrate
   demonstration of the colonization property. The W5 contract
   originally specified Proptest-precedence; W7's structural collision
   detection makes the better answer `Ambiguous`. Same source contract,
   colonized into different correct behavior under the more honest
   recognition mechanism. ADR-005 Amendment 3's tier-honesty discipline
   colonized W5's contract scope.

3. **The synthesis pass + node-kind dispatch combination** is the
   structural moment where ADR-010 Amendment 3 Performance Invariant 4
   stops being aspiration and becomes operational substrate. The
   8-tier-deep-fingerprint-against-200-files cost calculation lives in
   `synthesize_fingerprint_matches` as `Option<ItemKind> != Some(actual)
   => continue` — that line is the invariant's operational form.

4. **The hotfixes that rode along the W-streams** (ATK-A2-002,
   nested-generic phantom guard, ATK-A2-001 path-split, ATK-A2-007 enum
   no-op) are evidence for the no-fixed-point property at fix-as-you-go
   discipline scope. None of them were on the planned W-stream list;
   all of them shipped because pathmaker noticed during the in-flight
   work and fixed in session per the project's "no tech debt" posture.

---

*Drafted: 2026-05-08, A2 day-2 evening, after W8 shipped.*
*Pathmaker.*

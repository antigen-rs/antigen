//! ATK-A2 adversarial failing tests.
//!
//! Each test asserts a property that SHOULD be true but the current implementation
//! violates. Tests here FAIL until the bug is fixed.
//!
//! Fixture files live in `tests/fixtures/atk_a2_*/`. Scan is run against the
//! fixture directory (no filesystem mutation at test time — fixtures are static).
//!
//! When a bug is fixed and a test passes, add a comment recording which ATK
//! it covers. Do NOT remove passing tests — they are regressions guards.

use antigen::audit::{audit, AuditHint, WitnessKind, WitnessStatus, WitnessTier};
use antigen::scan::{scan_workspace, Immunity, MatchKind, ScanReport};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// ============================================================================
// ATK-A2-001: extract_presents uses raw string-split, not syn::parse2
//
// A #[presents(my_crate::PanickingInDrop)] declared with a qualified path
// must extract "PanickingInDrop" as the antigen_type — not the token-rendered
// string "my_crate :: PanickingInDrop" (with spaces inserted by ToTokens).
// ============================================================================

#[test]
fn atk_a2_001_path_qualified_presents_extracts_last_segment() {
    let report = scan_workspace(&fixture("atk_a2_001_presents_qualified_path"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "should find one presentation"
    );
    let p = &report.presentations[0];
    assert_eq!(
        p.antigen_type, "PanickingInDrop",
        "antigen_type must be the last path segment, not the full token string;\n\
         got: {:?}\n\
         ATK-A2-001: extract_presents uses raw string-split on the token rendering.\n\
         quote::ToTokens renders 'my_crate::PanickingInDrop' as 'my_crate :: PanickingInDrop'\n\
         (spaces around '::'). split(\"::\") then finds no \"::\" separator and returns\n\
         the whole string. Fix: use syn::Path parsing to extract the last segment.",
        p.antigen_type
    );
}

// ============================================================================
// ATK-A2-003: Empty-body function witness passes audit as well-formed
//
// Naturalist PREDICTION 1: "the temptation will be to ship reachability-only
// and call it execution-tier." This test catches that drift.
//
// A function with no #[test] attribute and an empty body must NOT be treated
// as a well-formed immunity claim by audit. At minimum the audit must
// distinguish this from a real test witness at the tier level (ADR-001
// amendment 1 Change 4).
// ============================================================================

#[test]
fn atk_a2_003_empty_function_witness_is_reachability_tier() {
    let fixture_root = fixture("atk_a2_003_empty_witness");
    let scan = scan_workspace(&fixture_root, None).unwrap();
    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // The witness resolves (function exists) but is not a test.
    if let WitnessStatus::Resolved {
        ref witness_kind, ..
    } = a.witness_status
    {
        assert_ne!(
            *witness_kind,
            WitnessKind::Test,
            "ATK-A2-003: empty_witness has no #[test] attribute and must not be \
             classified as WitnessKind::Test"
        );
    } else {
        panic!(
            "ATK-A2-003: expected Resolved status for empty_witness; got {:?}",
            a.witness_status
        );
    }

    // Post-W7 tier check: a Function-kind witness with no test attribute and
    // an empty body sits at Reachability — the function exists but asserts
    // nothing. Below Execution tier, fails --strict gates correctly.
    assert_eq!(
        a.witness_tier,
        WitnessTier::Reachability,
        "ATK-A2-003: empty Function-kind witness must map to Reachability, not higher",
    );
    assert_eq!(a.audit_hint, AuditHint::FunctionResolves);
    assert!(
        !a.is_well_formed(),
        "ATK-A2-003: Reachability-tier witness must not be well-formed",
    );
    assert!(!a.meets_tier(WitnessTier::Execution));
}

// ============================================================================
// ATK-A2-004: Fabricated external witness passes audit unconditionally
//
// Any string starting with "clippy::" is classified External and is_well_formed()
// returns true — including made-up lint names that clippy doesn't know about.
// This is a sub-clause F violation: no validation at the external trust boundary.
// ============================================================================

#[test]
fn atk_a2_004_fabricated_external_witness_is_reachability_tier() {
    // Build the scan report manually to avoid filesystem dependency for this unit.
    let immunity = Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "clippy::nonexistent_lint_i_made_up_completely_4a2".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "impl".to_string(),
        // W3: structural item-identity. Synthetic record for audit-only test;
        // any value works since this test doesn't exercise the matcher.
        item_target: antigen::scan::ItemTarget::Unknown { line: 10 },
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    // Use the fixture root as an (empty-ish) workspace root for audit's fn walk.
    let root = fixture("atk_a2_003_empty_witness"); // any existing dir works
    let audit_report = audit(&report, &root);

    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    assert!(
        matches!(a.witness_status, WitnessStatus::External { .. }),
        "clippy:: prefix must produce External status — that part is correct"
    );

    // Post-W7 tier check (per ADR-005 Amendment 3): an external-tool reference
    // whose tool has NOT been invoked sits at Reachability + the
    // ExternalToolPrefixRecognized hint. The fabricated lint name is
    // structurally indistinguishable from a real one at this layer; both stay
    // Reachability until A3+ runs the tool to promote to Execution.
    assert_eq!(
        a.witness_tier,
        WitnessTier::Reachability,
        "ATK-A2-004: external-tool prefix only = Reachability tier, not Execution",
    );
    assert_eq!(a.audit_hint, AuditHint::ExternalToolPrefixRecognized);
    assert!(
        !a.is_well_formed(),
        "ATK-A2-004: Reachability-tier external witness must not be well-formed",
    );
    assert!(!a.meets_tier(WitnessTier::Execution));
}

// ============================================================================
// ATK-A2-006: witness = my_fn() (call expression) → NotFound for existing fn
//
// The ScanImmuneArgs renders `my_test_fn()` as `"my_test_fn ()"` via ToTokens.
// validate_witness does trim_end_matches("()") which does NOT strip " ()" (the
// space before the paren is in the string). The lookup then fails.
// ============================================================================

#[test]
fn atk_a2_006_witness_call_expression_resolves_to_existing_test() {
    let fixture_root = fixture("atk_a2_006_witness_call_expr");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(
        scan.immunities.len(),
        1,
        "fixture must have one immunity; found: {:?}",
        scan.parse_failures
    );

    // Check what the scan recorded for the witness string — this reveals the
    // token-rendering behavior.
    let recorded_witness = &scan.immunities[0].witness;
    // Token rendering via quote::ToTokens adds spaces: "my_test_fn ()"
    // This is the intermediate evidence of the bug:
    eprintln!("ATK-A2-006: scan recorded witness = {:?}", recorded_witness);

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    assert!(
        matches!(a.witness_status, WitnessStatus::Resolved { .. }),
        "ATK-A2-006: witness = my_test_fn() (call expression form) must resolve\n\
         to the existing #[test] function my_test_fn.\n\
         Currently FAILING: token rendering produces {:?} (with space before paren).\n\
         trim_end_matches(\"()\") fails to strip \" ()\" because the space breaks\n\
         the exact-suffix match. The function lookup then fails.\n\
         Fix: after rsplit on segments, trim whitespace before stripping parens,\n\
         OR parse the witness expression structurally to extract the function path\n\
         without depending on token-string rendering.\n\
         Got: {:?}",
        recorded_witness,
        a.witness_status
    );
}

// ============================================================================
// ATK-A2-007: #[antigen] on an enum is silently discarded by scan
//
// The visit_item_enum handler does `let _ = attr` — a no-op. The comment says
// "we record it for diagnostic" but nothing is recorded. Developer gets silence.
// ============================================================================

#[test]
fn atk_a2_007_antigen_on_enum_produces_feedback_not_silence() {
    let fixture_root = fixture("atk_a2_007_antigen_on_enum");
    let report = scan_workspace(&fixture_root, None).unwrap();

    // The scan should either record a diagnostic or record the antigen declaration.
    // It must NOT produce zero output — that's the diagnostic black hole.
    assert!(
        !report.parse_failures.is_empty() || !report.antigens.is_empty(),
        "ATK-A2-007: #[antigen] applied to an enum must not be silently discarded.\n\
         The visit_item_enum handler does `let _ = attr` (a no-op).\n\
         Expected: either a parse_failure diagnostic explaining the limitation,\n\
         OR an AntigenDeclaration recorded with type_name = \"DeterminismClass\".\n\
         Got: antigens={:?}  parse_failures={:?}\n\
         Fix: in visit_item_enum, when an antigen attribute is found, either:\n\
         (a) push to report.parse_failures with a helpful message, or\n\
         (b) record an AntigenDeclaration (and let cargo antigen scan explain\n\
             that enums cannot directly be antigen-bearing in v0.1).",
        report.antigens,
        report.parse_failures
    );
}

// ============================================================================
// ATK-A2-005 (reframed): Scope-unaware witness lookup — ambiguous name must
// not silently resolve.
//
// The flat FunctionIndex cannot distinguish two functions with the same name
// in different scopes. When a witness declaration is ambiguous (two functions
// share the name), the audit must NOT silently resolve to whichever function
// the filesystem walk happened to index last.
//
// Fixture (tests/fixtures/atk_a2_005_scope_cross_reactive/):
//   tests.rs       — `#[test] fn verify_boundary() { assert!(...) }`  (intended)
//   utils.rs       — `fn verify_boundary() {}`                         (collision)
//   immune_site.rs — `#[immune(X, witness = verify_boundary)]`
//
// THE PREMISE ISSUE WITH THE ORIGINAL TEST: the original test asserted that
// the resolution MUST be Test-kind (i.e., the right function wins). That's
// the wrong assertion — it only tests for a specific tie-breaking rule, not
// for the real bug. A "prefer Test over Function on collision" fix would pass
// that test while still being wrong: it leaves the result dependent on which
// functions happen to share a name, producing fragile immunity claims that
// break if the naming changes.
//
// THE CORRECT ASSERTION: when two functions share a name, the audit must NOT
// silently pick one. The correct fix is one of:
//   (a) WitnessStatus that flags ambiguity (WitnessStatus::Ambiguous or
//       equivalent), OR
//   (b) require a qualified path — `verify_boundary` is rejected; the user
//       must write `tests::verify_boundary` to be unambiguous.
//
// In both cases: `is_well_formed()` must return false for an ambiguous
// unqualified witness name. A developer writing `witness = verify_boundary`
// in a codebase where that name is shared gets a NOT-well-formed result
// that tells them to qualify the path — not a silent pass.
//
// PASS CONDITION for W7: `is_well_formed()` returns false for an unqualified
// witness name that resolves to multiple functions in the workspace. The
// resolution either produces a new WitnessStatus variant (Ambiguous) or
// WitnessStatus::NotFound with an ambiguity message.
// ============================================================================

#[test]
fn atk_a2_005_ambiguous_witness_name_is_not_silently_resolved() {
    let fixture_root = fixture("atk_a2_005_scope_cross_reactive");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // Post-W7: same-named functions in the workspace must NOT silently resolve
    // to one of them. The audit emits WitnessStatus::Ambiguous + WitnessTier::None
    // + AuditHint::AmbiguousResolution. The developer is forced to qualify the
    // path or rename one candidate.
    match &a.witness_status {
        WitnessStatus::Ambiguous { candidates } => {
            assert!(
                candidates.len() >= 2,
                "ATK-A2-005: expected at least two candidate paths; got {candidates:?}",
            );
        }
        other => panic!(
            "ATK-A2-005: expected WitnessStatus::Ambiguous; got {:?}",
            other
        ),
    }
    assert_eq!(a.witness_tier, WitnessTier::None);
    assert_eq!(a.audit_hint, AuditHint::AmbiguousResolution);
    assert!(!a.is_well_formed());
    assert_eq!(audit_report.ambiguous_count, 1);
}

// ============================================================================
// ATK-A2-009 (pre-implementation contract): Stale tolerance orphan detection
//
// Biological analog: peripheral suppression (Tregs) continuing after the
// antigen it suppressed is no longer present — dysregulation.
//
// When W6 ships #[antigen_tolerance], audit must detect tolerance markers
// whose named antigen no longer exists in the workspace.
//
// Status: #[ignore] until W6 ships. Remove ignore, verify test FAILS,
// then fix the audit implementation to make it pass.
// Substrate check: `grep "ATK-A2-009" antigen/tests/atk_a2_adversarial.rs`
// ============================================================================

#[test]
fn atk_a2_009_stale_tolerance_orphan_is_flagged_by_scan() {
    // Per ADR-011 §Mechanics + ATK-A2-009 (naturalist's biology cognate
    // "peripheral suppression continuing after the antigen it suppressed is
    // no longer present"). A tolerance marker for an antigen that isn't
    // declared in the scanned workspace must surface as an orphan.
    //
    // For v0.1 the orphan check lives on the scan side
    // (ScanReport::orphaned_tolerances). Cross-crate antigens are deferred
    // to A3 — a tolerance referencing an antigen imported from another
    // crate would surface as an orphan in this v0.1 model. That's the
    // documented v0.1 limitation.
    let fixture_root = fixture("atk_a2_009_orphaned_tolerance");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(
        scan.tolerances.len(),
        1,
        "fixture has one #[antigen_tolerance] declaration",
    );
    assert!(
        scan.antigens.is_empty(),
        "fixture must NOT declare OldAntigen — that's the orphan setup",
    );

    let orphans = scan.orphaned_tolerances();
    assert_eq!(
        orphans.len(),
        1,
        "ATK-A2-009: tolerance for undeclared OldAntigen must surface as orphan",
    );
    assert_eq!(orphans[0].antigen_type, "OldAntigen");
    assert!(
        !orphans[0].rationale.is_empty(),
        "orphan tolerance carries the original rationale forward for diagnostic context",
    );
}

// ============================================================================
// ATK-A2-010 (pre-implementation contract): Phantom-type witness type-param
// mismatch — ADR-013 OQ1 enforcement gate
//
// Biological analog: anergy / non-functional B-cell — the receptor binds
// something, but the proof content is vacuous (wrong epitope).
//
// When W7 ships phantom-type witness recognition (WitnessKind::PhantomType),
// audit must detect witnesses whose type parameters don't structurally encode
// a proof for the claimed antigen.
//
// Status: #[ignore] until W7 ships. Remove ignore, verify test FAILS,
// then fix W7 implementation to make it pass.
// Substrate check: `grep "ATK-A2-010" antigen/tests/atk_a2_adversarial.rs`
// ============================================================================

#[test]
fn atk_a2_010_phantom_witness_type_param_mismatch_is_flagged() {
    // W7 shipped WitnessKind::PhantomType and recognize-and-warn per ADR-013 §OQ1.
    //
    // Contract (v0.1 / recognize-and-warn):
    //   - PolarityProof::<WrongClass> resolves as PhantomType (structural shape matches)
    //   - WitnessTier = FormalProof (shape recognized — ADR-013 §OQ1 accept)
    //   - AuditHint = PhantomTypeShapeRecognized (with the hint "verify constructor sealed")
    //   - is_well_formed() = true (FormalProof >= Execution)
    //
    // KNOWN LIMITATION (ADR-013 §OQ1 deferred): v0.1 does NOT check whether the
    // type parameter WrongClass structurally matches the antigen's failure-class.
    // A phantom-type witness with wrong type params is accepted at FormalProof tier.
    // The safeguard is the AuditHint: users see "phantom-type — verify constructor
    // is sealed" and must manually confirm correctness.
    //
    // The pre-implementation contract expected is_well_formed() == false for mismatched
    // phantoms. ADR-013 §OQ1 explicitly defers that to a future ADR. This test
    // documents the actual v0.1 behavior and serves as a regression guard.
    //
    // When the future ADR ships type-param matching, this test must be updated to
    // assert is_well_formed() == false for mismatched type params.

    let immunity = Immunity {
        antigen_type: "frame-translation".to_string(),
        witness: "PolarityProof::<WrongClass>".to_string(),
        file: PathBuf::from("lib.rs"),
        line: 1,
        item_kind: "impl".to_string(),
        item_target: antigen::scan::ItemTarget::Unknown { line: 0 },
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    // Use a temporary directory as workspace root — no actual files needed
    // because detect_phantom_type_witness fires BEFORE the function index lookup.
    let tmp = std::env::temp_dir();
    let audit_report = audit(&report, &tmp);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // Shape is recognized as phantom-type (has turbofish).
    match &a.witness_status {
        WitnessStatus::Resolved { witness_kind, .. } => {
            assert!(
                matches!(witness_kind, WitnessKind::PhantomType { .. }),
                "ATK-A2-010: PolarityProof::<WrongClass> must resolve as PhantomType, got {:?}",
                witness_kind
            );
        }
        other => panic!(
            "ATK-A2-010: expected Resolved(PhantomType), got {:?}",
            other
        ),
    }

    // v0.1 behavior: FormalProof tier (shape recognized, construction not validated).
    assert_eq!(
        a.witness_tier,
        WitnessTier::FormalProof,
        "ATK-A2-010: phantom-type witness is FormalProof tier in v0.1 (recognize-and-warn)"
    );

    // Hint signals the user must verify the constructor manually.
    assert_eq!(
        a.audit_hint,
        AuditHint::PhantomTypeShapeRecognized,
        "ATK-A2-010: phantom-type witness hint must be PhantomTypeShapeRecognized"
    );

    // KNOWN GAP (future ADR): is_well_formed() returns true even for wrong type params.
    // This is the v0.1 limitation — documented here as a regression guard for the
    // day this gets tightened by type-param matching.
    assert!(
        a.is_well_formed(),
        "ATK-A2-010: v0.1 reports phantom-type witness as well-formed (recognize-and-warn);\n\
         future ADR will tighten this to check type-param match against antigen shape"
    );
}

// ============================================================================
// ATK-A2-002: line_of_attr returns first-occurrence line for every declaration
//
// When a file contains two antigen declarations, line_of_attr("antigen") finds
// the FIRST "#[antigen" in the source for BOTH calls. The second declaration
// reports the line number of the first.
//
// Fixture: two antigen declarations in one file.
//   Line 13: #[antigen(...)] pub struct PanickingInDrop;
//   Line 19: #[antigen(...)] pub struct FrameTranslation;
//
// Expected: antigens[0].line == 13, antigens[1].line == 19 (or vice versa by
//   type_name, since scan walks in AST order).
// Actual: both report line 13 (the first match of "#[antigen" in the file).
//
// Impact: audit output reports the wrong source location for every declaration
// after the first of each kind. Developer follows line 13 to find the wrong struct.
// Compounds with W3 item-identity work: if line numbers are wrong, diagnostic
// output for unaddressed presentations cites wrong locations.
// ============================================================================

#[test]
fn atk_a2_002_second_antigen_in_file_reports_correct_line() {
    let report = scan_workspace(&fixture("atk_a2_002_multi_antigen_same_file"), None).unwrap();

    assert_eq!(
        report.antigens.len(),
        2,
        "fixture has two antigen declarations"
    );

    // Find by type name to be order-independent.
    // Note: the antigen attribute goes on the struct, so the type_name is the
    // struct name: PanickingInDropAntigen and FrameTranslationAntigen.
    let panicking = report
        .antigens
        .iter()
        .find(|a| a.type_name == "PanickingInDropAntigen")
        .expect("PanickingInDropAntigen antigen must be found");
    let frame = report
        .antigens
        .iter()
        .find(|a| a.type_name == "FrameTranslationAntigen")
        .expect("FrameTranslationAntigen antigen must be found");

    // In the fixture file:
    //   Line 7:  #[antigen(   ← PanickingInDropAntigen attribute start
    //   Line 13: #[antigen(   ← FrameTranslationAntigen attribute start
    // The heuristic finds line 7 for BOTH calls (first match of "#[antigen").
    assert_eq!(
        panicking.line, 7,
        "PanickingInDropAntigen attribute starts at line 7 in the fixture"
    );
    assert_ne!(
        frame.line, panicking.line,
        "ATK-A2-002: FrameTranslationAntigen (line 13) must not report the same\n\
         line as PanickingInDropAntigen (line 7).\n\
         line_of_attr finds the FIRST occurrence of '#[antigen' in the entire file\n\
         for every call — so every declaration after the first reports line 7.\n\
         Fix: pass the &syn::Attribute to line_of_attr and use proc_macro2::Span\n\
         byte offsets to find the true line of each specific invocation.\n\
         Got: PanickingInDropAntigen.line={}, FrameTranslationAntigen.line={}",
        panicking.line, frame.line
    );
    assert_eq!(
        frame.line, 13,
        "FrameTranslationAntigen attribute starts at line 13 in the fixture;\n\
         ATK-A2-002: got {} instead",
        frame.line
    );
}

// ============================================================================
// ATK-A2-011: validate_witness discards the entire module path prefix (TC-7)
//
// `rsplit("::").next()` keeps only the last segment and throws away the prefix.
// A witness written as `nonexistent_crate::nonexistent_module::real_fn` resolves
// cleanly if `real_fn` exists anywhere in the workspace — even though the path
// as written points nowhere coherent.
//
// This is TC-7 from naturalist's tier-confusion roam (20260508-tier-confusion-roam.md).
// Distinct from ATK-A2-005 (same-name collision): TC-7 is about a path that doesn't
// point to the right place at all, not about two functions sharing a name.
//
// Biology cognate (naturalist): auto-immune cross-reactivity — antibody recognizes
// a peptide motif on the wrong tissue. Same segment shape, wrong molecular context.
// ============================================================================

#[test]
fn atk_a2_011_fabricated_path_prefix_does_not_pass_strict_gate() {
    let fixture_root = fixture("atk_a2_011_path_discard_witness");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    let recorded = &scan.immunities[0].witness;
    eprintln!("ATK-A2-011: scan recorded witness = {:?}", recorded);

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // Post-W7: the audit's path-prefix discarding is still a v0.1 limitation;
    // `validate_witness` extracts the last segment via `rsplit("::")`. The
    // fabricated prefix `nonexistent_crate::nonexistent_module` is silently
    // dropped and the audit lands on the underlying `#[test] fn
    // real_function_name`. BUT — under W7's tier honesty, that resolution is
    // Reachability (test exists, audit didn't invoke `cargo test`), NOT
    // Execution. The --strict CI gate now correctly catches this even though
    // the fabricated-prefix check itself is deferred to A3+.
    //
    // The structural fix for fabricated-prefix detection requires module-graph
    // resolution (A3+ scope). For v0.1, the tier-honesty check is the load-
    // bearing safety net.
    assert_eq!(a.witness_tier, WitnessTier::Reachability);
    assert!(
        !a.is_well_formed(),
        "ATK-A2-011: even though the fabricated path prefix is silently dropped \
         (A3+ gap), the underlying #[test] resolves at Reachability tier — \
         not Execution — so --strict CI gates correctly fail this case.",
    );
    assert!(!a.meets_tier(WitnessTier::Execution));
}

// ============================================================================
// ATK-A2-012: #[test] #[ignore] witness classified as Test-kind (well-formed)
//
// TC-6 from naturalist's tier-confusion roam. A witness with BOTH #[test] AND
// #[ignore] attributes never runs by default (cargo test skips ignored tests).
// detect_kind checks for #[test] presence but not #[ignore] — so a test that
// is explicitly marked as "do not run" passes audit as WitnessKind::Test.
//
// ADR-001 Amendment 1 Change 4 mapping: Resolved(Test) = "Execution because
// cargo test ran it" — but audit doesn't run cargo test, and for #[ignore]
// tests cargo test explicitly wouldn't run it even if invoked.
//
// Sub-cases of TC-6 already covered: #[test] fn witness() { return; } →
// covered by ATK-A2-003's empty/trivial body assertion. The distinct sub-case
// here is the #[ignore] attribute specifically, which is a developer-intentional
// "this test doesn't count yet" signal that audit currently ignores.
// ============================================================================

#[test]
fn atk_a2_012_ignored_test_witness_is_reachability_with_distinct_hint() {
    let fixture_root = fixture("atk_a2_012_ignored_test_witness");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // Post-W7: detect_kind distinguishes #[test] #[ignore] from a runnable
    // #[test]. Both produce Reachability tier in v0.1 (no cargo test invocation),
    // but the audit_hint disambiguates: `TestAttributePresentIgnoreSkipped`
    // tells the user the test is explicitly opt-out from default runs, while
    // `TestAttributePresentNotInvoked` says the test is runnable but unrun.
    //
    // This is the orthogonal-axes design: tier carries the strength gate
    // (--min-tier execution fails both); hint carries the diagnostic detail.
    if let WitnessStatus::Resolved {
        ref witness_kind, ..
    } = a.witness_status
    {
        assert_eq!(
            *witness_kind,
            WitnessKind::IgnoredTest,
            "ATK-A2-012: #[test] #[ignore] must be classified as IgnoredTest, not Test",
        );
    } else {
        panic!(
            "ATK-A2-012: expected Resolved status for not_yet_ready_witness; got {:?}",
            a.witness_status
        );
    }

    assert_eq!(a.witness_tier, WitnessTier::Reachability);
    assert_eq!(a.audit_hint, AuditHint::TestAttributePresentIgnoreSkipped);
    assert!(!a.is_well_formed());
    assert!(!a.meets_tier(WitnessTier::Execution));
}

// ============================================================================
// ATK-W7-002: detect_phantom_type_witness fires on ANY turbofish string —
//             including completely fabricated/nonexistent types
//
// `detect_phantom_type_witness` runs BEFORE the workspace function-index lookup.
// Any witness string containing "::<" triggers phantom-type recognition and
// returns WitnessStatus::Resolved { PhantomType } — without checking whether
// the type actually exists in the workspace.
//
// This is ADR-013 §OQ1's "recognize-and-warn" behavior, BUT the tier assigned
// is WitnessTier::FormalProof — the HIGHEST tier. A completely fabricated
// `Nonexistent::<Fake>::construct()` gets FormalProof and is_well_formed() true.
//
// ADR-005 sub-clause F violation: the trust boundary is "construction encodes
// proof" but the audit extends trust based only on the syntactic presence of
// turbofish — no verification that the type exists, that the constructor is
// sealed, or that the construction is non-trivial.
//
// Impact tier: MEDIUM (not silent false-positive like A2-011, but
// FormalProof-tier false-confidence is arguably worse than Reachability).
// ============================================================================

#[test]
fn atk_w7_002_fabricated_phantom_type_gets_formal_proof_tier() {
    // A witness with turbofish but no real type behind it.
    // detect_phantom_type_witness fires on "::<" — it doesn't check existence.
    let immunity = Immunity {
        antigen_type: "frame-translation".to_string(),
        witness: "CompletelyFabricatedType::<AlsoFake>::construct".to_string(),
        file: PathBuf::from("lib.rs"),
        line: 1,
        item_kind: "impl".to_string(),
        item_target: antigen::scan::ItemTarget::Unknown { line: 0 },
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    let tmp = std::env::temp_dir();
    let audit_report = audit(&report, &tmp);
    let a = &audit_report.audits[0];

    // Currently: FormalProof — the highest tier — for a fabricated type.
    // This is the ADR-013 §OQ1 limitation documented here as a regression guard.
    // The expected long-term behavior (future ADR): existence check before
    // granting FormalProof.
    //
    // NOTE: this test currently PASSES because FormalProof IS what W7 returns.
    // The test documents the known gap, not a bug to fix now.
    // Future ADR will change this to Reachability or NotFound for nonexistent types.
    assert!(
        matches!(
            a.witness_status,
            WitnessStatus::Resolved {
                witness_kind: WitnessKind::PhantomType { .. },
                ..
            }
        ),
        "ATK-W7-002: fabricated turbofish witness resolves as PhantomType (syntactic match)"
    );
    assert_eq!(
        a.witness_tier,
        WitnessTier::FormalProof,
        "ATK-W7-002: KNOWN GAP — fabricated phantom type gets FormalProof tier \
         because existence is not checked. Future ADR must add existence verification."
    );
    assert_eq!(a.audit_hint, AuditHint::PhantomTypeShapeRecognized);
    // is_well_formed() returns true — documented as a v0.1 limitation.
    assert!(
        a.is_well_formed(),
        "ATK-W7-002: v0.1 reports fabricated phantom-type as well-formed (recognize-and-warn);\n\
         this is the ADR-013 §OQ1 deferred limitation"
    );
}

// ============================================================================
// ATK-W7-003: detect_phantom_type_witness nested generics fall through cleanly
//
// `Foo::<Bar<Baz>>::new` contains a nested generic in the type-param region.
// The balanced-bracket guard in detect_phantom_type_witness detects the
// unmatched `<` in params_raw and returns None, so the witness falls through
// to function-index lookup (producing NotFound, not a malformed FormalProof).
//
// Pre-fix behavior (v0.1 before hotfix): returned Resolved(PhantomType) with
// malformed type_params="Bar<Baz" (dangling open bracket). That would silently
// corrupt future type-param matching. The fix was caught by this test.
//
// Post-fix behavior: None → function-index → NotFound. Honest: audit reports
// the witness cannot be resolved rather than fabricating a FormalProof tier
// for a parse that didn't succeed.
// ============================================================================

#[test]
fn atk_w7_003_nested_generic_in_phantom_witness_falls_through_to_not_found() {
    let immunity = Immunity {
        antigen_type: "frame-translation".to_string(),
        witness: "Foo::<Bar<Baz>>::new".to_string(),
        file: PathBuf::from("lib.rs"),
        line: 1,
        item_kind: "impl".to_string(),
        item_target: antigen::scan::ItemTarget::Unknown { line: 0 },
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
    };
    let mut report = ScanReport::default();
    report.immunities.push(immunity);

    let tmp = std::env::temp_dir();
    let audit_report = audit(&report, &tmp);
    let a = &audit_report.audits[0];

    // Nested generic must NOT produce a (malformed) PhantomType result.
    // The balanced-bracket guard returns None; function-index finds no
    // function named "new" in the empty temp dir, so we get NotFound.
    assert!(
        matches!(a.witness_status, WitnessStatus::NotFound { .. }),
        "ATK-W7-003: nested generic must fall through to NotFound, not fabricate FormalProof; got {:?}",
        a.witness_status,
    );
    assert_eq!(a.witness_tier, WitnessTier::None);
    assert!(!a.is_well_formed());
}

// ============================================================================
// ATK-A2-ENUM-VARIANT: #[presents] on an enum variant is silently ignored
//
// `ScanVisitor` has `visit_item_enum` which calls `check_attrs` on the enum
// itself and then delegates to `syn::visit::visit_item_enum`. The syn
// visitor traverses variants, but because there is no `visit_variant`
// override in `ScanVisitor`, `check_attrs` is NEVER called on
// `variant.attrs`. A `#[presents(SomeAntigen)]` on an enum variant
// compiles cleanly (the proc-macro layer accepts it) but produces zero
// scan output — the presentation is invisible to failure-class memory.
//
// STATUS: FAILING — scanner has no visit_variant override
// BUG: No ItemTarget::EnumVariant exists; scanner ignores variant-level attrs
// ============================================================================

#[test]
fn atk_a2_enum_variant_presents_is_not_silently_ignored() {
    let report =
        scan_workspace(&fixture("atk_a2_enum_variant_presents"), None).expect("scan completes");

    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-ENUM-VARIANT: #[presents(BoundaryViolation)] on the `External` enum variant \
         must produce exactly one presentation record in the scan output. \
         Instead the scanner silently ignored it — no visit_variant override in ScanVisitor \
         means check_attrs is never called on variant.attrs. \
         Found presentations: {:?}. \
         Fix: add visit_variant to ScanVisitor and add ItemTarget::EnumVariant(enum_name, variant_name).",
        report.presentations,
    );

    let p = &report.presentations[0];
    assert_eq!(
        p.antigen_type, "BoundaryViolation",
        "ATK-A2-ENUM-VARIANT: presentation antigen_type must be 'BoundaryViolation'; got {:?}",
        p.antigen_type,
    );
}

// ============================================================================
// ATK-A2-ENUM-VARIANT-MULTI: two enum variants with different #[presents]
// annotations share the same structural fingerprint (the enclosing enum's
// digest). A consumer using structural_fingerprint as a unique key would
// silently deduplicate or conflate the two presentations.
//
// This tests that BOTH presentations survive in the scan report with distinct
// antigen_type fields — i.e., the shared fingerprint does not cause one to
// shadow the other in the report.
// ============================================================================

#[test]
fn atk_a2_enum_variant_multi_presents_both_survive_in_report() {
    let report =
        scan_workspace(&fixture("atk_a2_enum_variant_multi_presents"), None).expect("scan");

    assert_eq!(
        report.presentations.len(),
        2,
        "ATK-A2-ENUM-VARIANT-MULTI: two enum variants with different #[presents] annotations \
         must both appear in scan output. Both share the enclosing enum's structural fingerprint \
         (the enclosing-enum digest stands in for variants per scan.rs comment at ~4330). \
         A consumer using structural_fingerprint as a unique key would silently lose one. \
         Got {} presentations: {:?}.",
        report.presentations.len(),
        report.presentations,
    );

    let types: Vec<&str> = report
        .presentations
        .iter()
        .map(|p| p.antigen_type.as_str())
        .collect();
    assert!(
        types.contains(&"BoundaryViolation"),
        "ATK-A2-ENUM-VARIANT-MULTI: BoundaryViolation presentation missing; got {:?}",
        types,
    );
    assert!(
        types.contains(&"CapabilityEscape"),
        "ATK-A2-ENUM-VARIANT-MULTI: CapabilityEscape presentation missing; got {:?}",
        types,
    );

    // Structural fingerprints for enum variants share the enclosing enum's digest.
    // If they're EQUAL, that's by design (comment says "enclosing-enum digest stands in
    // for each variant") — but it must be DOCUMENTED and not silently cause deduplication.
    let fp0 = &report.presentations[0].structural_fingerprint;
    let fp1 = &report.presentations[1].structural_fingerprint;
    assert_eq!(
        fp0, fp1,
        "ATK-A2-ENUM-VARIANT-MULTI: two variants of the same enum SHOULD share the enum's \
         structural fingerprint (the enclosing-enum digest stands in for each variant per \
         scan.rs comment). If this fails, the design changed — update the test and docs.",
    );
}

// ============================================================================
// ATK-A2-IMPL-CONST: #[presents] on an impl-block const is silently ignored
//
// `ScanVisitor` has `visit_impl_item_fn` but NO `visit_impl_item_const`
// override. When `syn::visit::visit_item_impl` delegates to the visitor's
// impl-item traversal, `ImplItemConst` nodes pass through with their attrs
// unchecked. A `#[presents(SomeAntigen)]` on an associated constant
// compiles cleanly but produces zero scan output — identical blindspot to
// the enum variant case.
//
// This is the same class of bug as ATK-A2-ENUM-VARIANT (no visit_* override
// for the item kind), now on an impl-block const.
//
// STATUS: FAILING — scanner has no visit_impl_item_const override
// BUG: ImplItemConst.attrs never routed through check_attrs
// ============================================================================

#[test]
fn atk_a2_impl_const_presents_is_not_silently_ignored() {
    let report =
        scan_workspace(&fixture("atk_a2_impl_const_presents"), None).expect("scan completes");

    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-IMPL-CONST: #[presents(BoundaryViolation)] on `Parser::MAX_INPUT_BYTES` \
         impl-block const must produce exactly one presentation record in scan output. \
         Instead the scanner silently ignored it — no visit_impl_item_const override \
         in ScanVisitor means ImplItemConst.attrs are never routed through check_attrs. \
         Found presentations: {:?}. \
         Fix: add visit_impl_item_const to ScanVisitor that calls check_attrs on \
         ImplItemConst.attrs with an appropriate ItemTarget (ImplFn pattern for the \
         containing impl's type + const name, or a new ItemTarget::ImplConst variant).",
        report.presentations,
    );

    let p = &report.presentations[0];
    assert_eq!(
        p.antigen_type, "BoundaryViolation",
        "ATK-A2-IMPL-CONST: presentation antigen_type must be 'BoundaryViolation'; got {:?}",
        p.antigen_type,
    );
}

// ============================================================================
// ATK-A2-TOPLEVEL-CONST: #[presents] on a top-level const is silently ignored
//
// `ScanVisitor` has `visit_item_fn` and `visit_item_struct` but NO
// `visit_item_const` override. A `#[presents(X)]` on a top-level `const`
// compiles cleanly (the proc-macro passes through any item) but the scanner
// never calls `check_attrs` on the const's attrs — same gap as the enum
// variant and impl-const cases.
//
// Top-level constants are real vulnerability sites: `const MAX_SIZE: usize`
// that bounds buffer allocation, `const TIMEOUT_SECS: u64` at a network
// boundary — these are legitimate presentation sites for overflow/timeout
// failure classes.
//
// STATUS: FAILING — scanner has no visit_item_const override
// BUG: ItemConst.attrs never routed through check_attrs
// ============================================================================

#[test]
fn atk_a2_toplevel_const_presents_is_not_silently_ignored() {
    let report =
        scan_workspace(&fixture("atk_a2_toplevel_const_presents"), None).expect("scan completes");

    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-TOPLEVEL-CONST: #[presents(BoundaryViolation)] on top-level const \
         MAX_REQUEST_SIZE must produce exactly one presentation record in scan output. \
         Instead the scanner silently ignored it — no visit_item_const override in \
         ScanVisitor means ItemConst.attrs are never routed through check_attrs. \
         Found presentations: {:?}. \
         Fix: add visit_item_const to ScanVisitor that calls check_attrs with \
         ItemTarget::Fn(const_name) or a new ItemTarget::Const(name) variant.",
        report.presentations,
    );

    let p = &report.presentations[0];
    assert_eq!(
        p.antigen_type, "BoundaryViolation",
        "ATK-A2-TOPLEVEL-CONST: antigen_type must be 'BoundaryViolation'; got {:?}",
        p.antigen_type,
    );
}

// ============================================================================
// ATK-A2-PRES-FP: extract_presentation always emits empty structural_fingerprint
//
// `extract_immune` (the immunity path) correctly uses `self.current_item_digest`
// for `structural_fingerprint`. But `extract_presentation` always emits
// `String::new()` — even for structs, functions, and impls where the visitor
// sets `self.current_item_digest` before calling `check_attrs`.
//
// This is a silent failure: the fingerprint is the drift-detection baseline.
// An empty fingerprint means:
//   1. Audit cannot detect that a #[presents]-marked item changed shape.
//   2. `cargo antigen scan --format json` output misleadingly shows `""` for
//      every explicit presentation — adopters inspecting the JSON see no
//      fingerprint data even when the item IS fingerprintable.
//   3. The schema lock's fingerprint field is effectively dead for presentations.
//
// Root cause: `extract_presentation` in scan.rs does not read
// `self.current_item_digest`. The fix: change `String::new()` to
// `self.current_item_digest.clone()` in `extract_presentation`, matching what
// `extract_immune` already does.
//
// STATUS: FAILING — extract_presentation always emits String::new() for
//   structural_fingerprint, even for structs/fns/impls where current_item_digest
//   is populated by the visitor before check_attrs is called.
// ============================================================================

#[test]
fn atk_a2_pres_fp_struct_explicit_marker_has_non_empty_fingerprint() {
    // Fixture: #[presents] on an impl block (the 001 fixture reuses an impl,
    // which has current_item_digest computed in visit_item_impl before check_attrs).
    // Any item kind with a visitor that sets current_item_digest before calling
    // check_attrs exposes this bug.
    //
    // We use the toplevel const fixture here because it's the simplest item kind
    // that calls check_attrs. The bug is universal: ALL item kinds produce empty
    // fingerprints for presentations. We use a struct fixture to show the bug on
    // an item that definitely HAS a computable fingerprint.
    //
    // Fixture for struct: create a dedicated fixture that is a struct with #[presents].
    // For now, use atk_a2_001 (impl with #[presents]) — impls have fingerprints.
    let report = scan_workspace(&fixture("atk_a2_001_presents_qualified_path"), None)
        .expect("scan completes");

    assert_eq!(
        report.presentations.len(),
        1,
        "fixture must have exactly one presentation; got: {:?}",
        report.presentations,
    );

    let p = &report.presentations[0];

    assert!(
        !p.structural_fingerprint.is_empty(),
        "ATK-A2-PRES-FP: #[presents] on an impl block must produce a non-empty \
         structural_fingerprint in the scan output. The impl visitor sets \
         self.current_item_digest via antigen_fingerprint::structural_digest(item) \
         BEFORE calling check_attrs — but extract_presentation always emits \
         String::new() instead of self.current_item_digest.clone(). \
         Immunity records correctly use self.current_item_digest (see extract_immune). \
         Fix: change extract_presentation structural_fingerprint from String::new() \
         to self.current_item_digest.clone(). \
         Got structural_fingerprint = {:?}",
        p.structural_fingerprint,
    );
}

// ATK-A2-PRES-FP-CONST: #[presents] on const has WRONG fingerprint (contamination).
// extract_presentation now uses current_item_digest — but visit_item_const never
// SETS current_item_digest. It gets whatever the previous item set. Two consts
// with different values get the same fingerprint (the preceding struct's digest).
//
// STATUS: FAILING — two different consts produce identical fingerprints because
// neither visit_item_const, visit_item_static, nor visit_impl_item_const sets
// self.current_item_digest before calling check_attrs. The digest from a prior
// item bleeds into subsequent const/static presentations.
// Fix: add structural_digest calls in visit_item_const, visit_item_static,
// visit_impl_item_const, visit_trait_item_const, and the enum variant loop.
// Requires adding HasAttributes impls for these types in antigen-fingerprint.
#[test]
fn atk_a2_pres_fp_two_consts_with_different_values_have_different_fingerprints() {
    // Fixture has two consts: SMALL_LIMIT=1024 and LARGE_LIMIT=65536.
    // If fingerprints are contaminated (using the preceding struct's digest),
    // both fingerprints will be IDENTICAL — the struct digest bleeds into both.
    // If fingerprints correctly capture each const's own content, they must DIFFER.
    let report =
        scan_workspace(&fixture("atk_a2_const_fp_contamination"), None).expect("scan completes");

    assert_eq!(
        report.presentations.len(),
        2,
        "fixture must have exactly two presentations (SMALL_LIMIT + LARGE_LIMIT); got: {:?}",
        report.presentations,
    );

    let fp0 = &report.presentations[0].structural_fingerprint;
    let fp1 = &report.presentations[1].structural_fingerprint;

    assert_ne!(
        fp0, fp1,
        "ATK-A2-PRES-FP-CONST: two const items with different values must produce DIFFERENT \
         structural_fingerprints. Both got {:?} — proving fingerprint contamination: \
         visit_item_const does not set self.current_item_digest, so the preceding \
         struct's digest bleeds into all subsequent const presentations in the file. \
         Fix: (1) add HasAttributes for syn::ItemConst/ItemStatic/ImplItemConst/TraitItemConst \
         in antigen-fingerprint/src/digest.rs; (2) set self.current_item_digest = \
         antigen_fingerprint::structural_digest(item) in each new visitor before \
         calling check_attrs.",
        fp0,
    );
}

// ============================================================================
// ATK-A2-FINGERPRINT-MISS: ActiveArgumentDiscard fingerprint misses its own
// declared instance because doc_contains only searches `///` doc attributes,
// not `//` regular comments.
//
// The ActiveArgumentDiscard antigen (dogfood.rs) declares fingerprint:
//   `all_of([item = impl, doc_contains("forward compat")])`
//
// The real instance in antigen-macros/src/parse.rs has:
//   impl Parse for PolyclonalArgs {
//       // Accept arbitrary trailing args for forward compat — discard them.
//       while !input.is_empty() { ... }
//   }
//
// The text "forward compat" appears in a `//` regular comment, NOT in a
// `///` doc attribute. `doc_contains` searches `#[doc = "..."]` attributes
// (desugared from `///`). Regular `//` comments are invisible to syn's AST
// representation — they're discarded by the tokenizer.
//
// Impact: the antigen's own fingerprint doesn't fire on its own named instance.
// Passive detection is broken for this family. Any adopter scanning a codebase
// with the discard-loop pattern will NOT see it flagged unless they add an
// explicit #[presents(ActiveArgumentDiscard)] marker.
//
// Fix options:
//   1. Change the fingerprint to a body-structural predicate:
//      body_contains_macro("parse") won't work. Use signature-level matching:
//      has_method("parse") + item = impl would be more robust.
//   2. Add `/// forward compat` doc comment to the impl block so doc_contains fires.
//   3. Use body_contains_macro or another predicate that matches the actual pattern.
//
// STATUS: FAILING — scanning antigen-macros produces 0 ActiveArgumentDiscard
//   fingerprint matches for PolyclonalArgs::parse, MonoclonalArgs::parse,
//   AdccArgs::parse.
// ============================================================================

#[test]
fn atk_a2_fingerprint_miss_active_argument_discard_matches_its_instance() {
    // Scan the workspace root so the fingerprints declared in antigen/ are
    // visible. The synthesis pass uses fingerprints found in the scanned tree;
    // scanning antigen-macros alone cannot see ActiveArgumentDiscard (declared
    // in antigen/src/stdlib/dogfood.rs). Filter results to files under
    // antigen-macros/ to confirm the match fires on the intended instance.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let macros_prefix = workspace_root.join("antigen-macros");

    let report = scan_workspace(workspace_root, None).expect("workspace scan completes");

    let found_active_arg_discard = report
        .presentations
        .iter()
        .any(|p| p.antigen_type == "ActiveArgumentDiscard" && p.file.starts_with(&macros_prefix));

    assert!(
        found_active_arg_discard,
        "ATK-A2-FINGERPRINT-MISS: scanning the workspace should find at least one \
         ActiveArgumentDiscard fingerprint match in antigen-macros/ \
         (PolyclonalArgs::parse, MonoclonalArgs::parse, or AdccArgs::parse — they all \
         use the discard-loop). Got 0 matches in antigen-macros/. \
         The fingerprint 'all_of([item = impl, has_method(\"parse\", \"(ParseStream) -> \
         syn::Result<Self>\")])' should match these impl Parse blocks. \
         Fingerprint scanning is workspace-level: the antigen declaration (in antigen/) \
         must be visible alongside the instance (in antigen-macros/) for synthesis to fire.",
    );
}

// ============================================================================
// ATK-A2-FINGERPRINT-MISS-2: CapabilityOmissionAtLowering fingerprint misses
// its own declared instance for the same reason as ActiveArgumentDiscard.
//
// The CapabilityOmissionAtLowering antigen (dogfood.rs) declares:
//   fingerprint = r#"doc_contains("to_leaf")"#
//
// The real instance is `LeafExpr::to_leaf()` in antigen-attestation/src/parser.rs.
// The impl block at line 299 has:
//   /// Convert to the runtime [`crate::Leaf`]. Mirror of ...
//
// That doc string does NOT contain the literal "to_leaf" — it says "to the
// runtime" — so doc_contains("to_leaf") produces 0 matches.
//
// The antigen describes instances where a lowering step (`to_leaf`) hardcodes
// defaults instead of threading parsed values through. The text "to_leaf"
// appears only in // comments inside test bodies, not in any #[doc = ...] attr.
//
// Impact: identical to ActiveArgumentDiscard — passive fingerprint detection
// is broken. This is the SECOND instance of the same `doc_contains` vs `//`
// comment class miss (AntigenFingerprintDivergesFromClassExtension).
//
// STATUS: FAILING — scanning antigen-attestation produces 0
//   CapabilityOmissionAtLowering fingerprint matches.
// ============================================================================

#[test]
fn atk_a2_fingerprint_miss_capability_omission_at_lowering_matches_its_instance() {
    // Scan the workspace root so the fingerprints declared in antigen/ are
    // visible alongside the instance in antigen-attestation/. Filter to files
    // under antigen-attestation/ to confirm the fingerprint fires there.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let attestation_prefix = workspace_root.join("antigen-attestation");

    let report = scan_workspace(workspace_root, None).expect("workspace scan completes");

    let found_capability_omission = report.presentations.iter().any(|p| {
        p.antigen_type == "CapabilityOmissionAtLowering" && p.file.starts_with(&attestation_prefix)
    });

    assert!(
        found_capability_omission,
        "ATK-A2-FINGERPRINT-MISS-2: scanning the workspace should find at least one \
         CapabilityOmissionAtLowering fingerprint match in antigen-attestation/ \
         (LeafExpr::to_leaf in parser.rs is the declared instance). Got 0 matches \
         in antigen-attestation/. \
         The fingerprint 'all_of([item = impl, has_method(\"to_leaf\", \
         \"(&self) -> crate::Leaf\")])' should match the impl LeafExpr block. \
         Fingerprint scanning is workspace-level: the antigen declaration (in antigen/) \
         must be visible alongside the instance (in antigen-attestation/) for synthesis \
         to fire.",
    );
}

// ============================================================================
// ATK-A2-MACRO-RULES: #[presents] on a macro_rules! item is silently ignored.
//
// The ScanVisitor overrides visit_item_struct, visit_item_impl, visit_item_const,
// visit_item_static, visit_item_fn, visit_impl_item_fn, visit_impl_item_const,
// visit_item_trait, visit_trait_item_fn, visit_trait_item_const, visit_item_type,
// and visit_item_enum — but NOT visit_item_macro (macro_rules! items).
//
// When syn walks a file and encounters a `macro_rules!` declaration, it calls
// the default visit_item_macro which does nothing with attributes. The
// ScanVisitor never calls check_attrs for macro items, so any #[presents(X)]
// on a macro_rules! block is silently dropped.
//
// This is the same blind-spot class as the enum-variant and impl-const fixes
// (the scanner has a default-visitng gap for an item kind it doesn't handle).
//
// PROOF: Scan a fixture with #[presents(SilentIntentNullification)] on a
// macro_rules! item. The report must contain one presentation. If it contains
// zero, the blind spot is confirmed.
//
// STATUS: FAILING — visit_item_macro is not overridden; macro_rules! attrs
// are never routed through check_attrs.
//
// Fix: add visit_item_macro override to ScanVisitor that routes item.attrs
// through check_attrs with an ItemTarget::Macro (or falls back to an existing
// target kind). Add ItemTarget::Macro variant if not present.
// ============================================================================

#[test]
fn atk_a2_macro_rules_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_macro_rules_presents"), None).unwrap();

    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-MACRO-RULES: scanning a file with #[presents(SilentIntentNullification)] \
         on a macro_rules! item must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor overrides visit_item_struct/impl/const/static/fn/etc \
         but does NOT override visit_item_macro. syn's default walker does not call \
         check_attrs for macro_rules! items — the #[presents] attribute is silently \
         dropped. Same blind-spot class as enum-variant (fixed) and impl-const (fixed). \
         Fix: add visit_item_macro override to ScanVisitor that routes item.attrs \
         through check_attrs with an ItemTarget::Macro(name) target.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-USE-ITEM: #[presents] on a `use` (re-export) item is silently ignored.
//
// ScanVisitor does not override visit_item_use. A #[presents(X)] on a `use`
// item (e.g., a dangerous re-export that exposes a capability at a trust
// boundary) is silently dropped — 0 presentations found.
//
// This is the same scanner blind-spot class as macro_rules! (above), enum
// variant, and impl const. Every item kind without an explicit visit_item_*
// override is invisible to the scanner.
//
// STATUS: FAILING — visit_item_use is not overridden.
//
// Fix: add visit_item_use override to ScanVisitor that routes item.attrs
// through check_attrs with an ItemTarget naming the re-exported path.
// ============================================================================

#[test]
fn atk_a2_use_item_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_use_presents"), None).unwrap();

    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-USE-ITEM: scanning a file with #[presents(SilentIntentNullification)] \
         on a `use` declaration must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor does NOT override visit_item_use. A dangerous \
         re-export marked with #[presents] (e.g., a module that surfaces an \
         unsafe capability at a trust boundary) is invisible to the scanner. \
         Same blind-spot class as macro_rules! (ATK-A2-MACRO-RULES), enum-variant \
         (fixed), and impl-const (fixed). \
         Fix: add visit_item_use override to ScanVisitor that routes item.attrs \
         through check_attrs.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-UNION: #[presents] on a `union` declaration is silently ignored.
//
// ScanVisitor does not override visit_item_union. Union declarations with
// #[presents] are invisible to the scanner — same blind-spot class as
// macro_rules! and use items (both fixed) but not yet addressed for unions.
//
// Union declarations are high-risk (unsafe memory reinterpretation) and
// exactly the kind of site that should carry explicit failure-class markers.
//
// STATUS: FAILING — visit_item_union is not overridden.
// ============================================================================

#[test]
fn atk_a2_union_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_union_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-UNION: scanning a file with #[presents(BoundaryViolation)] \
         on a union declaration must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor does NOT override visit_item_union. Union \
         declarations (which are inherently unsafe memory-reinterpretation) \
         with #[presents] attrs are silently dropped. \
         Same blind-spot class as macro_rules! and use-item. \
         Fix: add visit_item_union override to ScanVisitor.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-FOREIGN-MOD: #[presents] on an `extern` block is silently ignored.
//
// ScanVisitor does not override visit_item_foreign_mod. An `extern "C" { ... }`
// block annotated with #[presents] to mark the FFI boundary as presenting a
// trust-boundary failure class is invisible to the scanner.
//
// FFI extern blocks are high-priority mucosal boundary sites — exactly where
// BoundaryViolation and similar antigens should fire.
//
// STATUS: FAILING — visit_item_foreign_mod is not overridden.
// ============================================================================

#[test]
fn atk_a2_foreign_mod_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_foreign_mod_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-FOREIGN-MOD: scanning a file with #[presents(BoundaryViolation)] \
         on an extern block must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor does NOT override visit_item_foreign_mod. FFI \
         extern blocks annotated with #[presents] (e.g., marking an entire C \
         interface as a mucosal trust-boundary site) are silently dropped. \
         Fix: add visit_item_foreign_mod override to ScanVisitor.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-MOD: #[presents] ON a `mod` declaration itself is silently ignored.
//
// ScanVisitor does not override visit_item_mod. While items INSIDE a mod{}
// block are reached by recursion (syn::visit::visit_item_mod descends), the
// attribute ON the mod declaration itself is never routed through check_attrs.
//
// STATUS: FAILING — visit_item_mod is not overridden for the mod's own attrs.
// ============================================================================

#[test]
fn atk_a2_mod_declaration_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_mod_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-MOD: scanning a file with #[presents(BoundaryViolation)] \
         on a mod declaration must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor does NOT override visit_item_mod. While \
         items inside the mod are recursively visited, the attribute ON the \
         mod declaration itself is never routed through check_attrs. \
         Fix: add visit_item_mod override to ScanVisitor that checks item.attrs \
         before calling syn::visit::visit_item_mod for recursion.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-EXTERN-CRATE: #[presents] on an `extern crate` declaration is silently
// ignored.
//
// ScanVisitor does not override visit_item_extern_crate. An `extern crate foo;`
// annotated with #[presents] to mark an external dependency as a known risk site
// is invisible to the scanner.
//
// STATUS: FAILING before fix — visit_item_extern_crate not overridden.
// ============================================================================

#[test]
fn atk_a2_extern_crate_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_extern_crate_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-EXTERN-CRATE: scanning a file with #[presents(ExternalDependencyRisk)] \
         on an extern crate declaration must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor did not override visit_item_extern_crate. \
         Fix: add visit_item_extern_crate override to ScanVisitor.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-TRAIT-ALIAS: #[presents] on a trait alias declaration is silently
// ignored.
//
// ScanVisitor does not override visit_item_trait_alias. A trait alias annotated
// with #[presents] (e.g., marking a capability-narrowing alias as a boundary site)
// is invisible to the scanner. Fixture uses nightly syntax; syn parses it
// regardless of stable/nightly (syn supports the full Rust grammar).
//
// STATUS: FAILING before fix — visit_item_trait_alias not overridden.
// ============================================================================

#[test]
fn atk_a2_trait_alias_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_trait_alias_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-TRAIT-ALIAS: scanning a file with #[presents(AliasCapabilityLeak)] \
         on a trait alias declaration must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor did not override visit_item_trait_alias. \
         Fix: add visit_item_trait_alias override to ScanVisitor.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-IMPL-ITEM-TYPE: #[presents] on an impl-block associated type is
// silently ignored.
//
// ScanVisitor overrides visit_impl_item_fn and visit_impl_item_const, but has
// no visit_impl_item_type override. A `#[presents(X)] type Foo = Bar;` inside
// an impl block is silently dropped — associated types are real code sites that
// can present failure classes (e.g., a type alias that narrows or widens a
// capability boundary).
//
// STATUS: FAILING — visit_impl_item_type is not overridden.
// ============================================================================

#[test]
fn atk_a2_impl_item_type_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_impl_type_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-IMPL-ITEM-TYPE: scanning a file with #[presents(NullabilityViolation)] \
         on an impl-block associated type must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor does NOT override visit_impl_item_type. \
         Associated type items (`type Foo = Bar;` inside impl blocks) annotated \
         with #[presents] are silently dropped — same blind-spot class as \
         impl_item_const before its fix. \
         Fix: add visit_impl_item_type override to ScanVisitor.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-TRAIT-ITEM-TYPE: #[presents] on a trait associated type declaration
// is silently ignored.
//
// ScanVisitor overrides visit_trait_item_fn and visit_trait_item_const, but has
// no visit_trait_item_type override. A `#[presents(X)] type Item;` inside a
// trait body is silently dropped — trait associated type declarations are real
// code sites (especially mucosal boundary contracts like Iterator::Item).
//
// STATUS: FAILING — visit_trait_item_type is not overridden.
// ============================================================================

#[test]
fn atk_a2_trait_item_type_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_trait_type_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-TRAIT-ITEM-TYPE: scanning a file with #[presents(AssociatedTypeViolation)] \
         on a trait associated type declaration must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor does NOT override visit_trait_item_type. \
         Trait associated type items (`type Item;`) annotated with #[presents] are \
         silently dropped — same blind-spot class as trait_item_const before its fix. \
         Fix: add visit_trait_item_type override to ScanVisitor.",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-IMPL-TYPE-FP-CONTAMINATION: proactive contamination guard.
//
// Proactive guard: visit_impl_item_type MUST assign current_item_digest before
// check_attrs, or the preceding item's digest bleeds into the associated type's
// fingerprint (same contamination class as visit_item_const, fixed in fe6a3a0).
// ============================================================================

#[test]
fn atk_a2_impl_item_type_digest_not_contaminated_by_preceding_item() {
    let report = scan_workspace(&fixture("atk_a2_impl_type_fp_contamination"), None).expect("scan");

    assert_eq!(
        report.presentations.len(),
        2,
        "fixture must find exactly 2 presentations (type A + type B); got: {:?}",
        report.presentations,
    );

    let fp0 = &report.presentations[0].structural_fingerprint;
    let fp1 = &report.presentations[1].structural_fingerprint;

    assert_ne!(
        fp0, fp1,
        "ATK-A2-IMPL-TYPE-FP: two associated types with different bodies must produce \
         DIFFERENT structural fingerprints. Both got {:?} — digest contamination: \
         visit_impl_item_type was added WITHOUT self.current_item_digest assignment, \
         so the preceding item's digest bleeds into both associated-type presentations. \
         Fix: add 'self.current_item_digest = antigen_fingerprint::structural_digest(item);' \
         as the FIRST line of visit_impl_item_type.",
        fp0,
    );
}

// ============================================================================
// ATK-A2-TRAIT-TYPE-FP-CONTAMINATION: proactive contamination guard.
//
// Proactive guard: visit_trait_item_type MUST assign current_item_digest before
// check_attrs, or the preceding item's digest bleeds into the associated type's
// fingerprint (same contamination class as visit_item_const and impl_item_type).
// Two trait associated types with different bounds must produce DIFFERENT
// structural fingerprints.
// ============================================================================

#[test]
fn atk_a2_trait_item_type_digest_not_contaminated_by_preceding_item() {
    let report =
        scan_workspace(&fixture("atk_a2_trait_type_fp_contamination"), None).expect("scan");

    assert_eq!(
        report.presentations.len(),
        2,
        "fixture must find exactly 2 presentations (Output + Error types); got: {:?}",
        report.presentations,
    );

    let fp0 = &report.presentations[0].structural_fingerprint;
    let fp1 = &report.presentations[1].structural_fingerprint;

    assert_ne!(
        fp0, fp1,
        "ATK-A2-TRAIT-TYPE-FP: two associated types with different bounds must produce \
         DIFFERENT structural fingerprints. Both got {:?} — digest contamination: \
         visit_trait_item_type was added WITHOUT self.current_item_digest assignment \
         before check_attrs, so the preceding item's digest bleeds into the \
         associated-type presentation. \
         Fix: verify 'self.current_item_digest = antigen_fingerprint::structural_digest(item);' \
         is the FIRST statement of visit_trait_item_type, before check_attrs.",
        fp0,
    );
}

// ============================================================================
// ATK-A2-SCAN-NONEXISTENT-PATH: scan_workspace on a nonexistent path returns
// Ok with an empty report (0 files scanned, no error signal).
//
// scan_workspace uses WalkDir::new(root) and silently skips all walk errors
// via `let Ok(entry) = entry else { continue }`. This means a typo in the
// scan root produces a successful-looking empty report. An adopter who calls
// `cargo antigen scan --root /typo/path` sees zero presentations with no
// indication that nothing was scanned. The scan SHOULD distinguish between
// "scanned, found nothing" and "couldn't scan at all."
//
// This test documents the current behavior (Ok empty report) and asserts
// that files_scanned == 0 — callers must check this field to detect a
// misconfigured root. A future fix could return Err or warn in parse_failures.
// ============================================================================

#[test]
fn atk_a2_scan_nonexistent_path_returns_empty_report_silently() {
    let nonexistent = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("__this_path_does_not_exist__");
    assert!(
        !nonexistent.exists(),
        "test precondition: path must not exist"
    );

    let result = scan_workspace(&nonexistent, None);

    assert!(
        result.is_ok(),
        "ATK-A2-SCAN-NONEXISTENT-PATH: scan_workspace on a nonexistent path currently \
         returns Ok (not Err). This test documents the CURRENT behavior — if this assertion \
         fails, scan_workspace now returns Err for nonexistent paths, which is BETTER behavior. \
         Update this test to verify the Err is descriptive."
    );

    let report = result.unwrap();
    assert_eq!(
        report.files_scanned, 0,
        "ATK-A2-SCAN-NONEXISTENT-PATH: an empty-path scan must yield files_scanned == 0. \
         Callers must check this field to distinguish 'scanned and found nothing' from \
         'path was invalid and nothing was scanned'. Got files_scanned = {}.",
        report.files_scanned,
    );
    assert!(
        report.presentations.is_empty(),
        "ATK-A2-SCAN-NONEXISTENT-PATH: nonexistent path scan must produce no presentations. \
         Got: {:?}",
        report.presentations,
    );
    assert!(
        report.parse_failures.is_empty(),
        "ATK-A2-SCAN-NONEXISTENT-PATH: current behavior emits no parse_failure for a \
         nonexistent root — walk errors are silently discarded (line 2452 of scan.rs: \
         'let Ok(entry) = entry else {{ continue }}'). \
         A future improvement could add a parse_failure entry for 'root path not found'. \
         Got: {:?}",
        report.parse_failures,
    );
}

// ============================================================================
// ATK-A2-IMPL-ITEM-MACRO: #[presents] on an impl-block macro invocation is
// silently ignored.
//
// ScanVisitor overrides visit_impl_item_fn, visit_impl_item_const, and
// visit_impl_item_type, but has no visit_impl_item_macro override. A macro
// invocation inside an impl block annotated with #[presents(X)] (e.g.,
// `#[presents(MacroExpansionHazard)] forward_to_inner!()`) is silently
// dropped — the attrs field on ImplItemMacro is never routed through
// check_attrs.
//
// STATUS: FAILING until visit_impl_item_macro override is added.
// ============================================================================

#[test]
fn atk_a2_impl_item_macro_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_impl_macro_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-IMPL-ITEM-MACRO: scanning a file with #[presents(MacroExpansionHazard)] \
         on an impl-block macro invocation must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor has no visit_impl_item_macro override — ImplItemMacro \
         attrs are never routed through check_attrs. \
         Fix: add visit_impl_item_macro override to ScanVisitor (parallel to \
         visit_impl_item_fn/const/type).",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-TRAIT-ITEM-MACRO: #[presents] on a trait-body macro invocation is
// silently ignored.
//
// ScanVisitor overrides visit_trait_item_fn, visit_trait_item_const, and
// visit_trait_item_type, but has no visit_trait_item_macro override. A macro
// invocation inside a trait body annotated with #[presents(X)] (e.g.,
// `#[presents(TraitContractViolation)] blanket_requirements!()`) is silently
// dropped — the attrs field on TraitItemMacro is never routed through
// check_attrs.
//
// STATUS: FAILING until visit_trait_item_macro override is added.
// ============================================================================

#[test]
fn atk_a2_trait_item_macro_presents_is_not_silently_ignored() {
    let report = scan_workspace(&fixture("atk_a2_trait_macro_presents"), None).unwrap();
    assert_eq!(
        report.presentations.len(),
        1,
        "ATK-A2-TRAIT-ITEM-MACRO: scanning a file with #[presents(TraitContractViolation)] \
         on a trait-body macro invocation must find exactly 1 presentation. \
         Got {} presentations. \
         Root cause: ScanVisitor has no visit_trait_item_macro override — TraitItemMacro \
         attrs are never routed through check_attrs. \
         Fix: add visit_trait_item_macro override to ScanVisitor (parallel to \
         visit_trait_item_fn/const/type).",
        report.presentations.len()
    );
}

// ============================================================================
// ATK-A2-CONST-SYNTHESIS-MISS: fingerprint synthesis silently skips const items.
//
// synthesis_pass() calls item_kind_and_target(syn_item) for each top-level item.
// For syn::Item::Const the function returns None (caught by `_ => None`), causing
// `continue` — the const is never evaluated against any fingerprint.
//
// Consequence: a fingerprint without an `item = <kind>` pin (e.g. only
// `name = matches("SENTINEL_*")`) should fire for BOTH struct and const items
// matching the glob. It fires for the struct (ItemTarget::Struct goes through
// item_kind_and_target), but NOT for the const (silently skipped).
//
// This is the ParallelStateTrackersDiverge anti-pattern at the scanner's own
// design level: Pass 1 (attribute scanning) has visit_item_const; Pass 2
// (fingerprint synthesis) lacks the matching arm.
//
// STATUS: FAILING — synthesis produces 1 match (struct) but must produce 2
// (struct + const). THREE-WAY gap: (1) item_kind_and_target must handle Const,
// (2) item_kind_for_dispatch must map Item::Const to ItemKind::Const, AND
// (3) item_name() in matcher.rs must return the const ident. All three must land.
// ============================================================================

#[test]
fn atk_a2_const_synthesis_fingerprint_miss_is_silent() {
    let report = scan_workspace(&fixture("atk_a2_const_synthesis_miss"), None).unwrap();

    // The fixture declares SentinelSilentMiss with fingerprint `name = matches("SENTINEL_*")`.
    // Two items match: SENTINEL_StructSite (struct) and SENTINEL_CONST_SITE (const).
    // Both should appear as FingerprintMatch presentations — but synthesis skips consts.
    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();

    assert_eq!(
        fp_matches.len(),
        2,
        "ATK-A2-CONST-SYNTHESIS-MISS: fingerprint `name = matches(\"SENTINEL_*\")` \
         must fire for BOTH SENTINEL_StructSite (struct) AND SENTINEL_CONST_SITE (const). \
         Got {} FingerprintMatch presentations. \
         THREE-WAY gap — all three must be fixed together: \
         (1) item_kind_and_target() must return Some for syn::Item::Const; \
         (2) item_kind_for_dispatch block in synthesis_pass must map Item::Const to ItemKind::Const; \
         (3) item_name() in antigen-fingerprint/src/matcher.rs must return const ident. \
         Currently gap (3) causes name=matches() to return None for const items even \
         after gaps (1) and (2) are patched. ParallelStateTrackersDiverge at the scanner's \
         own design level: three separately-maintained const-handling sites with no \
         compile-time enforcement that they stay in sync.",
        fp_matches.len()
    );
}

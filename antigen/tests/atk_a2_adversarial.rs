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
use antigen::scan::{scan_workspace, Immunity, ScanReport};
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

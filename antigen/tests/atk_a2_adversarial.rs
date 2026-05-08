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

use antigen::audit::{audit, WitnessKind, WitnessStatus};
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
#[ignore = "blocks on W7 tier-aware audit — Function-kind witnesses must \
    not be well-formed without a tier downgrade per ADR-001 Amendment 1 \
    Change 4. Architectural fix; remove ignore when W7 lands."]
fn atk_a2_003_empty_function_witness_is_not_well_formed() {
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
        // Verify the kind is correct (not misclassified as Test).
        assert_ne!(
            *witness_kind,
            WitnessKind::Test,
            "ATK-A2-003: empty_witness has no #[test] attribute and must not be \
             classified as WitnessKind::Test"
        );

        // The critical assertion: a non-test Function-kind witness must NOT pass
        // is_well_formed(). An empty function body asserts nothing. Treating it
        // as well-formed is the "reachability-only called execution-tier" failure
        // the naturalist predicted in the A1 closure.
        assert!(
            !a.is_well_formed(),
            "ATK-A2-003: a Function-kind witness (no #[test], empty body) must NOT \
             be reported as well-formed by is_well_formed().\n\
             Currently FAILING: is_well_formed() returns true for any Resolved{{..}},\n\
             including witnesses that assert nothing.\n\
             Fix: WitnessKind::Function should produce a weaker well-formedness claim\n\
             (or a separate WitnessStatus variant) per ADR-001 amendment 1 Change 4\n\
             tiered witness-validity model. The audit must surface this to the user."
        );
    } else {
        panic!(
            "ATK-A2-003: expected Resolved status for empty_witness; got {:?}",
            a.witness_status
        );
    }
}

// ============================================================================
// ATK-A2-004: Fabricated external witness passes audit unconditionally
//
// Any string starting with "clippy::" is classified External and is_well_formed()
// returns true — including made-up lint names that clippy doesn't know about.
// This is a sub-clause F violation: no validation at the external trust boundary.
// ============================================================================

#[test]
#[ignore = "blocks on W7 tier-aware audit — External witnesses must not \
    be unconditionally well-formed; sub-clause F validation at the external \
    trust boundary is W7 territory. Remove ignore when W7 lands."]
fn atk_a2_004_fabricated_external_witness_is_not_well_formed() {
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

    // ATK-A2-004: External should NOT be unconditionally well-formed.
    // A fabricated lint name is not validated at all. --strict mode passing
    // on this is a sub-clause F violation (ADR-005): trust extended without
    // a validation check at the trust boundary.
    assert!(
        !a.is_well_formed(),
        "ATK-A2-004: a fabricated external witness (clippy::nonexistent_lint_i_made_up_completely_4a2)\n\
         must not pass is_well_formed().\n\
         Currently FAILING: is_well_formed() returns true for External{{..}} unconditionally.\n\
         Fix: External should be 'present but unvalidated' — below Resolved{{Test}} in the\n\
         tier model. is_well_formed() for External should return false until the external\n\
         tool has been actually invoked and confirmed."
    );
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
#[ignore = "pre-implementation contract for W7 — ambiguous witness names must \
    not silently resolve. Remove ignore when W7 ships FunctionIndex with \
    qualified-path resolution or WitnessStatus::Ambiguous."]
fn atk_a2_005_ambiguous_witness_name_is_not_silently_resolved() {
    let fixture_root = fixture("atk_a2_005_scope_cross_reactive");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // THE REAL FAILURE MODE: the witness `verify_boundary` is ambiguous —
    // two functions share this name in the workspace. The audit must surface
    // this ambiguity rather than silently resolving to one of them.
    //
    // Currently FAILING: audit silently picks whichever file was indexed last
    // (utils.rs, alphabetically after tests.rs) and reports Resolved{Function},
    // which is_well_formed() treats as structurally sound. The developer has
    // no indication their witness points at a non-test function.
    //
    // W7 fix: either
    //   (a) is_well_formed() returns false when the witness name is ambiguous, OR
    //   (b) a new WitnessStatus::Ambiguous variant is emitted, which is also
    //       not well-formed.
    // Either way: an unqualified witness that matches multiple functions in the
    // workspace must NOT produce a clean is_well_formed() result.
    assert!(
        !a.is_well_formed(),
        "ATK-A2-005 (reframed): witness `verify_boundary` is ambiguous — two functions\n\
         share this name in the fixture workspace:\n\
         - tests.rs: #[test] fn verify_boundary()   (intended reference)\n\
         - utils.rs: fn verify_boundary() {{}}        (collision, asserts nothing)\n\
         \n\
         The audit must NOT report is_well_formed() = true for an ambiguous unqualified\n\
         witness name. The developer cannot know which function the audit resolved to.\n\
         \n\
         Current behavior: {:?} — is_well_formed() = {}\n\
         \n\
         Fix (W7): FunctionIndex must detect name collisions and either:\n\
         (a) emit WitnessStatus::Ambiguous (not well-formed), requiring the user\n\
             to qualify the path (e.g., `tests::verify_boundary`), OR\n\
         (b) emit WitnessStatus::NotFound with a message explaining the ambiguity.\n\
         Silently picking one function by filesystem walk order is not acceptable.",
        a.witness_status,
        a.is_well_formed()
    );
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
#[ignore = "pre-implementation contract for W6/ADR-011; remove ignore when #[antigen_tolerance] and orphan detection ship"]
fn atk_a2_009_stale_tolerance_orphan_is_flagged_by_audit() {
    // Contract: a scan report containing a tolerance marker for an antigen
    // that has been renamed or removed must NOT silently pass audit.
    // AuditReport must expose orphaned tolerances as non-well-formed.
    //
    // Naturalist prediction (A1 closure): "don't ship ADR-011 without at
    // least a count + top-N rationale strings per antigen in audit output."
    // Orphan detection is the enforcement gate for that prediction.
    //
    // When W6 ships:
    //   1. ScanReport gains `tolerances: Vec<Tolerance>` field
    //   2. AuditReport gains `orphaned_tolerances: Vec<OrphanedTolerance>`
    //   3. Scan fixture: #[antigen_tolerance(OldAntigen, rationale="x")]
    //      with no AntigenDeclaration for OldAntigen in the workspace
    //   4. Assert: audit_report.orphaned_tolerances.len() == 1
    //   5. Assert: audit_report.all_valid() == false (orphans invalidate)
    //
    // TODO(adversarial): fill in test body when W6 extends ScanReport/AuditReport.
    panic!("pre-implementation contract — remove #[ignore] when W6 ships tolerance tracking");
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
#[ignore = "pre-implementation contract for W7/ADR-013; remove ignore when phantom-type witness detection ships"]
fn atk_a2_010_phantom_witness_type_param_mismatch_is_flagged() {
    // Contract: #[immune(FrameTranslation, witness = PolarityProof::<WrongClass>)]
    // must NOT be treated as semantically validated.
    //
    // The construction compiles (phantom types don't constrain at runtime),
    // but the type parameter WrongClass ≠ FrameTranslation means the witness
    // encodes a proof for a different antigen. The audit should warn.
    //
    // When W7 ships:
    //   1. WitnessKind::PhantomType { proof_type, type_params } exists
    //   2. audit resolves PolarityProof::<WrongClass> as PhantomType
    //   3. audit compares type_params against the antigen's structural shape
    //   4. Mismatch → WitnessStatus::Resolved with a warning, or a distinct
    //      variant — but NOT silently well-formed
    //   5. is_well_formed() for a mismatched phantom returns false
    //
    // ADR-013 §OQ1: "trivially constructible phantom-type witness (red flag)
    // vs construction-encodes-proof (the real pattern)." This test forces W7
    // to address OQ1 explicitly.
    //
    // TODO(adversarial): fill in test body when W7 ships WitnessKind::PhantomType.
    panic!("pre-implementation contract — remove #[ignore] when W7 ships phantom-type detection");
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
#[ignore = "blocks on W4 span-aware error messages — `line_of_attr` returns \
    the first occurrence of the attribute name in the file rather than the \
    span of the specific invocation. W4's span-threading work is the natural \
    home for the fix on the macro side; the scan-side line story lands in a \
    follow-up using `syn::spanned::Spanned::span().start().line`."]
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
#[ignore = "blocks on W7 tier-aware audit — cross-reactive (fabricated-prefix) \
    witness paths require sub-clause F validation at the witness-resolution \
    trust boundary. Architectural fix; remove ignore when W7 + the witness \
    path-resolution work land."]
fn atk_a2_011_fabricated_path_prefix_resolves_as_clean_witness() {
    let fixture_root = fixture("atk_a2_011_path_discard_witness");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    // Verify what the scan recorded for the witness path.
    let recorded = &scan.immunities[0].witness;
    eprintln!("ATK-A2-011: scan recorded witness = {:?}", recorded);

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // The witness path `nonexistent_crate::nonexistent_module::real_function_name`
    // is incoherent as written. The function exists in the crate root, not in
    // `nonexistent_crate::nonexistent_module`. The audit must NOT treat this as
    // a clean Resolved — the path prefix was fabricated.
    //
    // Currently FAILING: rsplit("::").next() extracts "real_function_name",
    // finds it in the workspace, and reports Resolved{Test} as well-formed.
    // The fabricated prefix is silently discarded without any validation.
    //
    // Fix direction (A3): parse witness as syn::Path, resolve against the
    // module graph, validate that the resolved function's actual path matches
    // the written path. Until then, at minimum: if the witness contains "::"
    // and the prefix segments don't resolve to any known module, emit a warning.
    assert!(
        !a.is_well_formed(),
        "ATK-A2-011: witness `nonexistent_crate::nonexistent_module::real_function_name`\n\
         must NOT be reported as well-formed.\n\
         The prefix path is fabricated; `real_function_name` exists in the crate root\n\
         with no connection to the claimed module path.\n\
         Currently FAILING: validate_witness does rsplit('::').next(), discards the\n\
         entire prefix, finds 'real_function_name' in the index, and reports Resolved.\n\
         Got: {:?}",
        a.witness_status
    );
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
#[ignore = "blocks on W7 tier-aware audit — #[ignore] test witnesses must \
    surface as a weaker tier (not equivalent to a running #[test]). Same \
    architectural pattern as ATK-A2-003/004/005. Remove ignore when W7 lands."]
fn atk_a2_012_ignored_test_witness_is_not_equivalent_to_running_test() {
    let fixture_root = fixture("atk_a2_012_ignored_test_witness");
    let scan = scan_workspace(&fixture_root, None).unwrap();

    assert_eq!(scan.immunities.len(), 1, "fixture must have one immunity");

    let audit_report = audit(&scan, &fixture_root);
    assert_eq!(audit_report.audits.len(), 1);
    let a = &audit_report.audits[0];

    // The witness `not_yet_ready_witness` has both #[test] AND #[ignore].
    // It never runs in standard CI. The immunity claim is backed by a test
    // that the developer has explicitly said "don't run this yet."
    //
    // Expected: either WitnessKind::IgnoredTest (not yet implemented), or
    // is_well_formed() returns false for ignored tests, or at minimum the
    // audit output flags the #[ignore] attribute to the user.
    //
    // Currently FAILING: detect_kind finds #[test], returns WitnessKind::Test,
    // and is_well_formed() returns true — treating a deliberately-skipped test
    // as equivalent to a running test.
    if let WitnessStatus::Resolved {
        ref witness_kind, ..
    } = a.witness_status
    {
        // If resolved as Test, audit must have checked for #[ignore] and must
        // NOT treat it as well-formed.
        assert!(
            !a.is_well_formed(),
            "ATK-A2-012: a #[test] #[ignore] witness must NOT be reported as well-formed.\n\
             The #[ignore] attribute means `cargo test` skips this test by default.\n\
             An immunity claim backed by a test that doesn't run in standard CI is\n\
             weaker than one backed by a test that runs.\n\
             Currently FAILING: detect_kind returns WitnessKind::Test for any function\n\
             with #[test] regardless of #[ignore], and is_well_formed() returns true.\n\
             Fix: detect_kind must check for #[ignore] attribute presence and return\n\
             a distinct kind (WitnessKind::IgnoredTest) or set a flag on the resolved\n\
             status. Minimum for v0.1: audit output warns on #[test] #[ignore] witnesses.\n\
             Got witness_kind={:?}",
            witness_kind
        );
    } else {
        panic!(
            "ATK-A2-012: expected Resolved status for not_yet_ready_witness; got {:?}",
            a.witness_status
        );
    }
}

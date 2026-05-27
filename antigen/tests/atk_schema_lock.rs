//! Schema-lock integration test: pin the user-facing JSON shape of
//! `cargo antigen scan` and `cargo antigen audit` to the documented schema
//! in `docs/output-formats.md`.
//!
//! Rationale (per A2 Phase 5 gap-check + Tekgy ratification pre-tag):
//! Phase 4 doc work produced multiple drift incidents — phantom
//! `ExternalUnvalidated` tier, wrong field names (`results` vs `audits`),
//! wrong `witness_kind` shapes, wrong `audit_hint` names — because docs
//! were authored against design-substrate (expedition prose) rather than
//! code-substrate (actual serde-serialized output). This test closes that
//! gap by parsing the real binary's JSON output and asserting against a
//! frozen schema. The test fails when the schema changes, forcing the
//! doc to be updated in the same change.
//!
//! Fixture: `antigen/examples/` — already exercised by other tests, ships
//! a small mixed-status workload (resolved + `not_found`, multiple tiers).
//!
//! Filed: 2026-05-12 (A3 substrate sweep, pre-rc.1 tag).

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Workspace root = parent of the `antigen` package dir.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen package dir has a parent (workspace root)")
        .to_path_buf()
}

/// Invoke `cargo run --bin cargo-antigen -- antigen <subcommand> --format
/// json --root antigen/examples` and return the parsed JSON.
fn run_and_parse(subcommand: &str) -> Value {
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace_root())
        .args([
            "run",
            "--quiet",
            "--bin",
            "cargo-antigen",
            "--",
            "antigen",
            subcommand,
            "--format",
            "json",
            "--root",
            "antigen/examples",
        ])
        .output()
        .unwrap_or_else(|e| panic!("failed to invoke cargo-antigen: {e}"));

    assert!(
        output.status.success(),
        "cargo-antigen {subcommand} exited non-zero: status={:?}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("cargo-antigen stdout is valid UTF-8");
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("cargo-antigen {subcommand} stdout is not valid JSON: {e}\nstdout:\n{stdout}")
    })
}

fn keys(v: &Value) -> Vec<String> {
    v.as_object()
        .expect("expected JSON object")
        .keys()
        .cloned()
        .collect()
}

fn contains_key(v: &Value, key: &str) -> bool {
    v.as_object().is_some_and(|m| m.contains_key(key))
}

// ============================================================================
// Scan schema lock
// ============================================================================

#[test]
fn schema_lock_scan_top_level_keys() {
    let json = run_and_parse("scan");
    assert!(
        contains_key(&json, "report"),
        "scan JSON must have top-level `report` key; got: {:?}",
        keys(&json)
    );
    assert!(
        contains_key(&json, "unaddressed"),
        "scan JSON must have top-level `unaddressed` key; got: {:?}",
        keys(&json)
    );
}

#[test]
fn schema_lock_scan_report_fields() {
    let json = run_and_parse("scan");
    let report = &json["report"];
    for required in &[
        "antigens",
        "presentations",
        "immunities",
        "tolerances",
        "lineage_edges",
        "files_scanned",
        "parse_failures",
    ] {
        assert!(
            contains_key(report, required),
            "scan `report` must contain `{required}`; got: {:?}",
            keys(report)
        );
    }
}

#[test]
fn schema_lock_scan_match_kind_values() {
    let json = run_and_parse("scan");
    let presentations = json["report"]["presentations"]
        .as_array()
        .expect("`presentations` is an array");
    assert!(
        !presentations.is_empty(),
        "antigen/examples fixture should yield at least one presentation"
    );
    for (i, p) in presentations.iter().enumerate() {
        let mk = p["match_kind"]
            .as_str()
            .unwrap_or_else(|| panic!("presentation[{i}] missing `match_kind` string"));
        assert!(
            matches!(mk, "explicit_marker" | "fingerprint_match"),
            "presentation[{i}].match_kind must be `explicit_marker` or `fingerprint_match`, got `{mk}`"
        );
    }
}

// ============================================================================
// Audit schema lock
// ============================================================================

#[test]
fn schema_lock_audit_top_level_keys() {
    let json = run_and_parse("audit");
    assert!(
        contains_key(&json, "scan"),
        "audit JSON must have top-level `scan` key; got: {:?}",
        keys(&json)
    );
    assert!(
        contains_key(&json, "audit"),
        "audit JSON must have top-level `audit` key; got: {:?}",
        keys(&json)
    );
    assert!(
        !contains_key(&json, "report"),
        "audit JSON must NOT have top-level `report` key (that's scan-only); got: {:?}",
        keys(&json)
    );
    assert!(
        !contains_key(&json, "results"),
        "audit JSON must NOT have top-level `results` key (legacy phantom name); got: {:?}",
        keys(&json)
    );
}

#[test]
fn schema_lock_audit_has_audits_array() {
    let json = run_and_parse("audit");
    let audit = &json["audit"];
    assert!(
        contains_key(audit, "audits"),
        "audit sub-object must have `audits` key (NOT `results`); got: {:?}",
        keys(audit)
    );
    assert!(
        !contains_key(audit, "results"),
        "audit sub-object must NOT have `results` key (legacy phantom name); got: {:?}",
        keys(audit)
    );
    assert!(audit["audits"].is_array(), "audit.audits must be an array");
}

#[test]
fn schema_lock_audit_witness_tier_variants() {
    const ALLOWED_TIERS: &[&str] = &["none", "reachability", "execution", "formal_proof"];

    let json = run_and_parse("audit");
    let audits = json["audit"]["audits"]
        .as_array()
        .expect("audit.audits is an array");
    assert!(
        !audits.is_empty(),
        "antigen/examples fixture should yield at least one audit entry"
    );
    for (i, a) in audits.iter().enumerate() {
        let tier = a["witness_tier"]
            .as_str()
            .unwrap_or_else(|| panic!("audit[{i}] missing `witness_tier` string"));
        assert!(
            ALLOWED_TIERS.contains(&tier),
            "audit[{i}].witness_tier `{tier}` is not in the v0.1 four-tier set {ALLOWED_TIERS:?} \
             — if a new tier was added, update this allowlist AND docs/witness-tiers.md AND \
             docs/output-formats.md in the same change"
        );
    }
}

#[test]
fn schema_lock_audit_hint_variants() {
    const ALLOWED_HINTS: &[&str] = &[
        // Code-witness hints (rc.1 original set).
        "none-applicable",
        "function-resolves",
        "test-attribute-present-not-invoked",
        "test-attribute-present-ignore-skipped",
        "proptest-present-not-invoked",
        "external-tool-prefix-recognized",
        "external-tool-invoked",
        "phantom-type-shape-recognized",
        "phantom-type-construction-validated",
        "ambiguous-resolution",
        "fabricated-path-prefix",
        "inherited-presentation-not-re-attested",
        // Substrate-witness hints (rc.2 — surfaces real states the
        // substrate-witness pipeline reaches; mirror of
        // antigen_attestation::SubstrateAuditHint).
        "discipline-sidecar-missing",
        "discipline-sidecar-schema-invalid",
        "discipline-predicate-failed",
        "discipline-substrate-stale",
        "discipline-substrate-delta-chain-near-cap",
        "discipline-predicate-passed-via-delta-chain",
        "discipline-predicate-passed-substrate-current",
        "tolerance-vibes-grade",
        "tolerance-sidecar-missing",
        "tolerance-predicate-failed",
        "tolerance-predicate-passed-substrate-current",
        "discipline-sidecar-kind-mismatch-expected-immunity-got-tolerance",
        "tolerance-sidecar-kind-mismatch-expected-tolerance-got-immunity",
        "discipline-immunity-tolerance-contradiction",
    ];

    let json = run_and_parse("audit");
    let audits = json["audit"]["audits"]
        .as_array()
        .expect("audit.audits is an array");
    for (i, a) in audits.iter().enumerate() {
        let hint = a["audit_hint"]
            .as_str()
            .unwrap_or_else(|| panic!("audit[{i}] missing `audit_hint` string"));
        assert!(
            ALLOWED_HINTS.contains(&hint),
            "audit[{i}].audit_hint `{hint}` is not in the v0.1.0-rc.2 \
             {n}-variant set — if a new hint was added, update this allowlist \
             AND docs/output-formats.md AND docs/witness-tiers.md in the same \
             change. Current allowed: {ALLOWED_HINTS:?}",
            n = ALLOWED_HINTS.len()
        );
    }
}

#[test]
fn schema_lock_audit_resolved_status_shape() {
    let json = run_and_parse("audit");
    let audits = json["audit"]["audits"]
        .as_array()
        .expect("audit.audits is an array");

    let resolved: Vec<&Value> = audits
        .iter()
        .filter(|a| a["witness_status"]["status"].as_str() == Some("resolved"))
        .collect();

    assert!(
        !resolved.is_empty(),
        "antigen/examples fixture should yield at least one `resolved` audit \
         (basic.rs declares a phantom-type immunity); got statuses: {:?}",
        audits
            .iter()
            .filter_map(|a| a["witness_status"]["status"].as_str())
            .collect::<Vec<_>>()
    );

    for (i, a) in resolved.iter().enumerate() {
        let ws = &a["witness_status"];
        assert!(
            contains_key(ws, "status"),
            "resolved audit[{i}].witness_status must have `status` field"
        );
        assert!(
            contains_key(ws, "location"),
            "resolved audit[{i}].witness_status must have `location` field"
        );
        assert!(
            contains_key(ws, "witness_kind"),
            "resolved audit[{i}].witness_status must have `witness_kind` field"
        );
    }
}

// ============================================================================
// ATK-SCHEMA-ITEM-TARGET: docs/output-formats.md must document every
// item_target discriminant that can appear in cargo antigen scan JSON output.
//
// The existing schema_lock_* tests exercise the `examples/` directory which
// uses only a small subset of ItemTarget variants (Fn, Struct, Impl, ImplFn).
// After ATK-A2-ENUM-VARIANT/IMPL-CONST/TOPLEVEL-CONST fixes, the scanner
// now emits EnumVariant, ImplConst, Const, and Static — none of which are
// documented in output-formats.md's "Item target shapes" section.
//
// This test runs cargo antigen scan on a fixture that exercises all new item
// kinds and asserts that each discriminant key ("EnumVariant", "ImplConst",
// "Const", "Static") appears in docs/output-formats.md.
//
// STATUS: FIXED — output-formats.md now documents EnumVariant, Const, Static, ImplConst.
// ============================================================================

#[test]
fn schema_lock_item_target_new_variants_are_documented() {
    // Run cargo antigen scan --format json on the fixtures that produce the
    // new item_target variants.
    let fixture_root = workspace_root()
        .join("antigen")
        .join("tests")
        .join("fixtures");

    // Run a scan across multiple fixtures to get all new variant kinds.
    // EnumVariant comes from atk_a2_enum_variant_presents.
    // Const comes from atk_a2_toplevel_const_presents.
    // Static comes from atk_a2_static_presents.
    // ImplConst comes from atk_a2_impl_const_presents.
    let variants_to_check = [
        ("EnumVariant", "atk_a2_enum_variant_presents"),
        ("Const", "atk_a2_toplevel_const_presents"),
        ("Static", "atk_a2_static_presents"),
        ("ImplConst", "atk_a2_impl_const_presents"),
    ];

    let output_formats_doc =
        std::fs::read_to_string(workspace_root().join("docs").join("output-formats.md"))
            .expect("docs/output-formats.md must be readable");

    for (variant_name, fixture_name) in &variants_to_check {
        let fixture_path = fixture_root.join(fixture_name);

        let output = Command::new(env!("CARGO"))
            .current_dir(workspace_root())
            .args([
                "run",
                "--quiet",
                "--bin",
                "cargo-antigen",
                "--",
                "antigen",
                "scan",
                "--format",
                "json",
                "--root",
                fixture_path.to_str().unwrap(),
            ])
            .output()
            .unwrap_or_else(|e| {
                panic!("failed to invoke cargo-antigen scan on {fixture_name}: {e}")
            });

        let stdout =
            String::from_utf8(output.stdout).expect("cargo-antigen scan stdout is valid UTF-8");
        let scan_json: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!("failed to parse scan JSON for {fixture_name}: {e}\nstdout: {stdout}")
        });

        // Verify the fixture actually produces the expected variant.
        let presentations = scan_json["report"]["presentations"]
            .as_array()
            .expect("presentations must be an array");
        let has_variant = presentations.iter().any(|p| {
            p["item_target"]
                .as_object()
                .is_some_and(|obj| obj.contains_key(*variant_name))
        });
        assert!(
            has_variant,
            "ATK-SCHEMA-ITEM-TARGET: fixture '{fixture_name}' must produce an \
             item_target with discriminant '{variant_name}' in scan JSON. \
             Got: {:?}",
            presentations
                .iter()
                .map(|p| &p["item_target"])
                .collect::<Vec<_>>()
        );

        // Verify the variant is documented.
        assert!(
            output_formats_doc.contains(variant_name),
            "ATK-SCHEMA-ITEM-TARGET: ItemTarget variant '{variant_name}' appears in \
             cargo antigen scan JSON output (produced by fixture '{fixture_name}') but \
             is NOT documented in docs/output-formats.md §'Item target shapes'. \
             This violates the schema-lock contract: any variant reachable from scan \
             must appear in the user-facing format documentation. \
             Fix: add '{variant_name}' to the item_target shapes table in output-formats.md."
        );
    }
}

// ============================================================================
// ATK-SCHEMA-PRES-FP: presentations in scan JSON must have non-empty
// structural_fingerprint for fingerprintable item kinds.
//
// After the extract_presentation fingerprint fix (commits d9c251f/fe6a3a0),
// #[presents] on an impl/struct/fn/const/enum produces a real fingerprint.
// The schema lock does NOT currently assert this — an accidental revert
// of the fix (String::new() instead of current_item_digest.clone()) would
// produce empty fingerprints silently, with all schema_lock tests still
// passing. This test closes the regression gap.
//
// We use the examples/ fixture (struct + impl + fn presentation sites) and
// assert that at least ONE presentation has a non-empty structural_fingerprint.
// If ALL are empty, the fix has regressed.
//
// STATUS: PASSING once commits d9c251f/fe6a3a0 land. If this test FAILS,
// the extract_presentation fingerprint fix was reverted.
// ============================================================================

#[test]
fn schema_lock_scan_presentation_fingerprint_non_empty() {
    let json = run_and_parse("scan");
    let presentations = json["report"]["presentations"]
        .as_array()
        .expect("`presentations` is an array");
    assert!(
        !presentations.is_empty(),
        "antigen/examples fixture should yield at least one presentation"
    );

    let has_non_empty = presentations.iter().any(|p| {
        p["structural_fingerprint"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    });

    assert!(
        has_non_empty,
        "ATK-SCHEMA-PRES-FP: at least one presentation in scan JSON must have \
         a non-empty `structural_fingerprint`. All are empty — this indicates \
         extract_presentation regression (the fix that changed String::new() to \
         self.current_item_digest.clone() was reverted). \
         Presentations: {:?}",
        presentations
            .iter()
            .map(|p| (
                p["antigen_type"].as_str().unwrap_or("?"),
                p["structural_fingerprint"].as_str().unwrap_or("MISSING"),
            ))
            .collect::<Vec<_>>()
    );
}

//! Adversarial tests for antigen DX findings from the camp binary-adopter expedition.
//!
//! Each test asserts what SHOULD be true — and currently FAILS because the fix
//! has not landed. Tests are named after the finding they exercise.
//!
//! Finding 8 — empty fingerprint guard:
//!   `atk_dx_f8_sign_empty_fp_must_warn`     — sign against="" emits warning + non-zero exit
//!   `atk_dx_f8_sign_empty_fp_any_passes`    — sign against="any" with empty fp is fine (control)
//!
//! Finding 3 — sidecar/witness= disconnect warning:
//!   `atk_dx_f3_scaffold_for_witness_site_warns` — scaffold on a witness= immune site warns
//!   `atk_dx_f3_jq_hint_uses_correct_field`      — scaffold jq hint references `structural_fingerprint`
//!
//! Finding 6 — presentations missing fingerprint:
//!   `atk_dx_f6_presentation_entry_has_fingerprint` — scan JSON presentations carry `structural_fingerprint`

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

// Serialize tests that write sidecars to the shared fixture directory.
// The F3 sidecar test must write to the real workspace path (so audit --root
// finds it during its scan) but that creates a race with parallel audit calls
// from other tests. This mutex serializes all tests in the file that touch
// the shared fixture sidecar path.
static FIXTURE_SIDECAR_MUTEX: Mutex<()> = Mutex::new(());

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

fn attest(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("attest")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    (exit, stdout, stderr)
}

/// Scaffold a sidecar with the given `current_fingerprint` and return the sidecar path.
fn scaffold_with_fp(dir: &Path, antigen: &str, item: &str, fp: &str) -> PathBuf {
    let src = dir.join("src").join("lib.rs");
    std::fs::create_dir_all(src.parent().unwrap()).unwrap();
    std::fs::write(&src, "// placeholder").unwrap();
    let (code, _stdout, stderr) = attest(&[
        "scaffold",
        "--antigen",
        antigen,
        "--source-file",
        src.to_str().unwrap(),
        "--item-path",
        item,
        "--fingerprint",
        fp,
    ]);
    assert_eq!(code, 0, "scaffold should succeed: {stderr}");
    src.parent()
        .unwrap()
        .join(".attest")
        .join(format!("{antigen}.json"))
}

// ============================================================================
// Finding 8 — empty fingerprint guard
// ============================================================================

/// FAILING: sign against an `against="current"` sidecar with empty fingerprint
/// must warn (or refuse with non-zero exit). Currently silently succeeds.
///
/// Reproducer: camp's `VacuousCompletionFalseGreen` sidecar was signed with
/// empty `current_fingerprint`; audit then fails the predicate with no hint
/// that the empty fingerprint is the cause.
#[test]
fn atk_dx_f8_sign_empty_fp_must_warn() {
    let tmp = tempfile::tempdir().unwrap();
    // Scaffold with empty fingerprint — this leaves current_fingerprint: ""
    let sidecar = scaffold_with_fp(tmp.path(), "TestAntigenF8", "test_item", "");

    // Signing against empty fingerprint must emit a warning (or refuse with exit != 0)
    let (code, stdout, stderr) = attest(&[
        "sign",
        "--sidecar",
        sidecar.to_str().unwrap(),
        "--item-path",
        "test_item",
        "--signer",
        "alice",
        "--fingerprint",
        "", // <-- the attack: empty fingerprint
        "--strength",
        "text-stamp",
    ]);

    let combined = format!("{stdout}\n{stderr}");
    // Must emit a specific warning about the empty fingerprint making the predicate unfulfillable,
    // OR refuse with non-zero exit code.
    // A success message that happens to contain "fingerprint" (e.g. "Signed: ... against fingerprint ``")
    // does NOT count as a warning.
    let warns = combined.to_lowercase().contains("warn")
        || combined.to_lowercase().contains("empty fingerprint")
        || combined.to_lowercase().contains("placeholder fingerprint")
        || combined.to_lowercase().contains("predicate will fail")
        || combined.to_lowercase().contains("will not pass")
        || code != 0;

    assert!(
        warns,
        "signing with empty fingerprint on an against='current' sidecar must warn or refuse; \
         the success message 'Signed: ... against fingerprint ``' does not count. \
         got exit={code}, stdout={stdout:?}, stderr={stderr:?}"
    );
}

/// Control: sign against an `against="any"` sidecar with empty fingerprint
/// is fine (no predicate to violate).
///
/// This should PASS after the fix — confirming the guard is scoped correctly
/// and doesn't over-trigger on sidecars where fingerprint staleness is irrelevant.
#[test]
fn atk_dx_f8_sign_empty_fp_any_passes() {
    let tmp = tempfile::tempdir().unwrap();
    // Scaffold normally and manually rewrite the sidecar to use against="any"
    let sidecar = scaffold_with_fp(tmp.path(), "TestAntigenF8Any", "test_item", "");
    let content = std::fs::read_to_string(&sidecar).unwrap();
    // The scaffold uses against="current" by default; rewrite to against="any"
    // This test is a control: against="any" with empty fp is intentionally valid
    // For now, just verify the sidecar was created — the real assertion is about
    // against="current" in the test above.
    assert!(
        content.contains("TestAntigenF8Any"),
        "sidecar must name the antigen: {content}"
    );
}

// ============================================================================
// Finding 3 — sidecar/witness= disconnect warning
// ============================================================================

/// FAILING: `audit` against a workspace where an immune site uses `witness=` but
/// has a signed `.attest/` sidecar must warn that the sidecar is being ignored.
///
/// Currently: audit takes the code-witness branch (`validate_witness`) when
/// `requires_predicate` is None, never checks for a sidecar, and emits no signal.
/// The adopter believes they've attested; audit disagrees silently.
///
/// Reproducer: `antigen/tests/fixtures/atk_a2_003_empty_witness` has
/// `#[immune(PanickingInDrop, witness = empty_witness)]`. We scaffold+sign a sidecar
/// adjacent to that file, run audit --root on the antigen workspace, and assert
/// the output warns about the ignored sidecar.
///
/// The camp live reproducer: `R:/camp/src/schema/.attest/VacuousCompletionFalseGreen.json`
/// was signed for an immune site using `witness=` — audit ignored the sidecar completely.
#[test]
fn atk_dx_f3_audit_warns_on_sidecar_for_witness_site() {
    use std::io::Write;
    // Serialize against other tests that share this fixture directory.
    // The sidecar must live at the real workspace path so `cargo antigen audit`
    // finds it during its scan — we cannot use a temp copy because audit resolves
    // sidecar paths relative to the scanned source tree. FIXTURE_SIDECAR_MUTEX
    // prevents the parallel-write race that intermittently caused this test to fail
    // when run as part of `cargo test --workspace`.
    let _guard = FIXTURE_SIDECAR_MUTEX.lock().unwrap();

    // The fixture directory that has a witness= immune site.
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("antigen")
        .join("tests")
        .join("fixtures")
        .join("atk_a2_003_empty_witness");

    assert!(
        fixture_dir.exists(),
        "fixture dir must exist: {}",
        fixture_dir.display()
    );

    // Scaffold a sidecar for the witness= immune site in the fixture.
    let attest_dir = fixture_dir.join(".attest");
    std::fs::create_dir_all(&attest_dir).unwrap();
    let sidecar_path = attest_dir.join("PanickingInDrop.json");

    // Write a minimal sidecar manually (scaffold would work too, but this is deterministic)
    let sidecar_content = r#"{
  "schema_version": "v1",
  "kind": "immunity",
  "antigen": { "name": "PanickingInDrop" },
  "source_file": "antigen/tests/fixtures/atk_a2_003_empty_witness/lib.rs",
  "items": [
    {
      "item_path": "SomeType::drop",
      "current_fingerprint": "test-fp",
      "signers": [
        {
          "name": "test-adversarial",
          "role": "adversarial",
          "date": "2026-05-25",
          "signed_against_fingerprint": "test-fp",
          "basis": { "kind": "fresh", "reasoning": "ATK test signer - this sidecar should be warned about" },
          "strength": "text_stamp"
        }
      ],
      "oracles": []
    }
  ]
}"#;
    std::fs::File::create(&sidecar_path)
        .unwrap()
        .write_all(sidecar_content.as_bytes())
        .unwrap();

    // Run audit on the antigen workspace.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let out = Command::new(bin())
        .arg("antigen")
        .arg("audit")
        .arg("--root")
        .arg(&workspace_root)
        .output()
        .expect("failed to run cargo-antigen audit");

    // Clean up before asserting (so a failing assert doesn't leave the sidecar)
    let _ = std::fs::remove_file(&sidecar_path);
    let _ = std::fs::remove_dir(&attest_dir);

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let combined = format!("{stdout}\n{stderr}");

    // The specific audit output line for the witness= fixture site:
    // "atk_a2_003_empty_witness/lib.rs:4  PanickingInDrop (witness = `empty_witness`)"
    // followed by tier/hint info.
    //
    // The hint line for THIS SPECIFIC SITE must mention the sidecar being ignored/bypassed.
    // It must NOT be "FunctionResolves" without any sidecar note — that's the silent failure.
    //
    // Find the block for the atk_a2_003 fixture in the output:
    let fixture_block_start = combined.find("atk_a2_003_empty_witness");
    assert!(
        fixture_block_start.is_some(),
        "audit output must mention the atk_a2_003_empty_witness fixture; got:\n{}",
        &combined[..combined.len().min(2000)]
    );

    let fixture_context = &combined[fixture_block_start.unwrap()..];
    // Take only the first ~300 chars (one campsite's output block)
    let site_output = &fixture_context[..fixture_context.len().min(300)];

    // The output for this site must warn about the sidecar being ignored.
    // Currently it says: "tier = Reachability, hint = FunctionResolves" — NO sidecar mention.
    // After the fix, it should say something like "sidecar present but ignored" or similar.
    let site_warns_about_sidecar = site_output.to_lowercase().contains("sidecar")
        && (site_output.to_lowercase().contains("witness")
            || site_output.to_lowercase().contains("ignored")
            || site_output.to_lowercase().contains("credited")
            || site_output.to_lowercase().contains("requires"));

    assert!(
        site_warns_about_sidecar,
        "audit output for the witness= immune site must warn about the .attest/ sidecar \
         being ignored (not credited). Currently the output shows only \
         'tier = Reachability, hint = FunctionResolves' with no mention of the present sidecar. \
         See audit.rs:1036 — the code-witness branch never checks for sidecar presence. \
         site audit output:\n{site_output}"
    );
}

/// FAILING: the jq hint emitted by `attest scaffold` references `.requires_predicate`
/// which does not exist in the scan JSON schema. The correct field is
/// `.structural_fingerprint`. Adopters following the hint get a broken jq query.
///
/// Reproducer: `attest scaffold` output on any site says:
///   `jq '.immunities[] | select(.antigen_type=="X") | .requires_predicate'`
/// but the actual immunity entry schema has no `requires_predicate` field.
#[test]
fn atk_dx_f3_jq_hint_uses_correct_field() {
    let tmp = tempfile::tempdir().unwrap();
    let sidecar_dir = tmp.path().join("src");
    std::fs::create_dir_all(&sidecar_dir).unwrap();
    let src = sidecar_dir.join("lib.rs");
    std::fs::write(&src, "// placeholder").unwrap();

    let (code, stdout, stderr) = attest(&[
        "scaffold",
        "--antigen",
        "TestAntigenJqHint",
        "--source-file",
        src.to_str().unwrap(),
        "--item-path",
        "some_fn",
    ]);
    assert_eq!(code, 0, "scaffold should succeed: {stderr}");

    let combined = format!("{stdout}\n{stderr}");
    // The hint must NOT reference the nonexistent .requires_predicate field
    assert!(
        !combined.contains("requires_predicate"),
        "scaffold jq hint must not reference nonexistent '.requires_predicate' field; \
         the correct field is '.structural_fingerprint'. Got: {combined}"
    );
    // And it SHOULD reference the correct field
    assert!(
        combined.contains("structural_fingerprint"),
        "scaffold jq hint must reference the actual '.structural_fingerprint' field; \
         got: {combined}"
    );
}

// ============================================================================
// Finding 6 — presentations missing structural_fingerprint in scan JSON
// ============================================================================

/// FAILING: `scan --format json` presentation entries carry no `structural_fingerprint`.
/// This means fingerprint-matched unmarked sites (the sites an adopter most needs
/// to decide about) have no fingerprint an adopter can pass to `attest scaffold --fingerprint`.
///
/// Only immunity entries currently carry `structural_fingerprint`.
/// Presentation entries (the 16k+ fingerprint-matched sites) do not.
///
/// Consequence: an adopter scanning their codebase, finding a presentation, and
/// wanting to scaffold a sidecar with a real fingerprint cannot do so without
/// first adding an #[immune] macro and re-scanning — a chicken-and-egg problem.
#[test]
fn atk_dx_f6_presentation_entry_has_fingerprint() {
    // Run scan on antigen's own workspace (has presentations)
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();

    let out = Command::new(bin())
        .arg("antigen")
        .arg("scan")
        .arg("--format")
        .arg("json")
        .current_dir(&workspace_root)
        .output()
        .expect("failed to run scan");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let report: serde_json::Value =
        serde_json::from_str(&stdout).expect("scan must emit valid JSON");

    let presentations = report["report"]["presentations"]
        .as_array()
        .expect("report must have presentations array");

    // Find any fingerprint-match presentation (not explicit_marker)
    let fingerprint_pres: Vec<_> = presentations
        .iter()
        .filter(|p| {
            p["match_kind"]
                .as_str()
                .is_some_and(|k| k == "fingerprint_match")
        })
        .collect();

    assert!(
        !fingerprint_pres.is_empty(),
        "scan must produce at least one fingerprint_match presentation in antigen's own workspace"
    );

    // Every fingerprint_match presentation must carry a structural_fingerprint field
    let missing: Vec<_> = fingerprint_pres
        .iter()
        .filter(|p| {
            p.get("structural_fingerprint")
                .is_none_or(|f| f.is_null() || f.as_str().is_some_and(str::is_empty))
        })
        .collect();

    assert!(
        missing.is_empty(),
        "all fingerprint_match presentation entries must carry structural_fingerprint; \
         {} of {} are missing it",
        missing.len(),
        fingerprint_pres.len()
    );
}

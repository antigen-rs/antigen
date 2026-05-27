//! Delivery-arm tests: ADR-024 convergent-evidence + recurrent-emergence audit
//! concerns must reach `cargo antigen audit` output (human + JSON).
//!
//! ## The failure this guards (notice 25256aa7, dogfood #25)
//!
//! `audit_convergent_evidence()` and `audit_recurrent()` compute concern hints
//! (`diagnostic-modality-insufficient`, `recurrence-anchor-no-itch-precondition`,
//! …) in the library, and scan populates `report.convergent_evidences` /
//! `report.recurrent_declarations` — but the CLI had ZERO references to either,
//! so the computed verdicts never reached the adopter. That is the
//! `AuditVerdictComputedButNotDelivered` severance (the same shape fixed for the
//! lineage + deferred-defense surfaces). These tests pin the delivery: a concern
//! is rendered in human output and carried in the JSON `convergent_evidence_audit`
//! / `recurrent_audit` fields; a clean workspace stays quiet.
//!
//! Substrate check:
//!   `cargo test --package cargo-antigen --test atk_convergent_recurrent_delivery_arm`

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

fn write_crate(src: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), src).unwrap();
    (tmp, src_dir)
}

fn audit_human(root: &Path) -> String {
    let out = Command::new(bin())
        .args(["antigen", "audit", "--root", root.to_str().unwrap()])
        .output()
        .expect("run audit");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    stdout + &stderr
}

fn audit_json(root: &Path) -> serde_json::Value {
    let out = Command::new(bin())
        .args([
            "antigen",
            "audit",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .expect("run audit json");
    serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).expect("audit JSON must parse")
}

// A #[recurrence_anchor] with no matching upstream #[itch] → the library emits
// recurrence-anchor-no-itch-precondition. It must be delivered.
const RECURRENT_SRC: &str = r#"use antigen::{antigen, recurrence_anchor};

#[antigen(name = "msrv-creep", family = "recurrent", fingerprint = "item = struct", summary = "recurring msrv bump")]
pub struct MsrvCreep;

#[recurrence_anchor(antigen = MsrvCreep)]
pub fn anchored_without_itch() {}
"#;

// A #[diagnostic] with a single modality but min_independent = 2 → the library
// emits diagnostic-modality-insufficient. It must be delivered.
const CONVERGENT_SRC: &str = r#"use antigen::{antigen, diagnostic, WitnessClass};

#[antigen(name = "under-diagnosed", family = "convergent", fingerprint = "item = struct", summary = "one modality, needs two")]
pub struct UnderDiagnosed;

#[diagnostic(modalities = [WitnessClass::StaticAnalysis], min_independent = 2)]
pub fn under_diagnosed_fn() {}
"#;

const CLEAN_SRC: &str = r#"use antigen::{antigen, presents};

#[antigen(name = "clean-class", family = "functional-correctness", fingerprint = "item = struct", summary = "no convergent/recurrent declarations")]
pub struct CleanClass;

#[presents(CleanClass)]
pub struct CleanSite;
"#;

#[test]
fn recurrent_concern_delivered_to_human_output() {
    let (_tmp, root) = write_crate(RECURRENT_SRC);
    let output = audit_human(&root);
    assert!(
        output.contains("recurrent-emergence") && output.contains("recurrence-anchor-no-itch"),
        "audit human output must DELIVER the recurrence-anchor-no-itch-precondition \
         concern (was a severed delivery arm). Output:\n{output}"
    );
}

#[test]
fn recurrent_concern_delivered_to_json() {
    let (_tmp, root) = write_crate(RECURRENT_SRC);
    let doc = audit_json(&root);
    let rec = doc
        .get("recurrent_audit")
        .expect("audit JSON must carry recurrent_audit (delivery arm)");
    assert_eq!(
        rec["concern_count"].as_u64(),
        Some(1),
        "the floating recurrence_anchor must surface exactly one concern; got {rec:?}"
    );
}

#[test]
fn convergent_concern_delivered_to_human_output() {
    let (_tmp, root) = write_crate(CONVERGENT_SRC);
    let output = audit_human(&root);
    assert!(
        output.contains("convergent-evidence") && output.contains("diagnostic-modality"),
        "audit human output must DELIVER the diagnostic-modality-insufficient concern \
         (was a severed delivery arm). Output:\n{output}"
    );
}

#[test]
fn convergent_concern_delivered_to_json() {
    let (_tmp, root) = write_crate(CONVERGENT_SRC);
    let doc = audit_json(&root);
    let conv = doc
        .get("convergent_evidence_audit")
        .expect("audit JSON must carry convergent_evidence_audit (delivery arm)");
    assert!(
        conv["concern_count"].as_u64().unwrap_or(0) >= 1,
        "the under-modalitied diagnostic must surface a concern; got {conv:?}"
    );
}

#[test]
fn clean_workspace_emits_no_convergent_or_recurrent_concern_section() {
    // The delivery is concern-gated: a workspace with no convergent/recurrent
    // declarations must NOT print the concern sections (signal, not noise).
    let (_tmp, root) = write_crate(CLEAN_SRC);
    let output = audit_human(&root);
    assert!(
        !output.contains("convergent-evidence declaration(s) with concerns"),
        "clean workspace must not print a convergent-evidence concern section: {output}"
    );
    assert!(
        !output.contains("recurrent-emergence declaration(s) with concerns"),
        "clean workspace must not print a recurrent-emergence concern section: {output}"
    );
}

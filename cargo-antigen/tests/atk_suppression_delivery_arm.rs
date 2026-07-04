//! Adversarial tests: suppression delivery arm -- anergy + immunosuppress must
//! be LOUD, never silent.
//!
//! ## The failure this guards (forward/suppression-loud-must-be-removed)
//!
//! `audit_deferred_defenses()` in antigen/src/audit.rs correctly computes
//! `DeferredDefenseAuditReport` -- active/expired/stale counts, per-declaration
//! `AuditHint` values. But `cargo antigen audit` NEVER calls this function. The
//! result is NEVER included in human or JSON output. The delivery arm is
//! completely severed.
//!
//! Team-lead reframe (2026-05-27): both #[anergy] and #[immunosuppress] are
//! intentional dev permissions to proceed with a known defense gap. The audit
//! must keep them LOUD -- always announce them prominently so they cannot
//! become silent accumulated debt. They must NOT block the build, but they
//! must NOT be invisible either.
//!
//! ## What these tests assert
//!
//! (1) Human output of `cargo antigen audit` does NOT mention anergy/immunosuppress
//!     (documents the gap: delivery arm severed, LOUD is NOT implemented).
//!
//! (2) JSON output of `cargo antigen audit` has NO `deferred_defense_audit` field
//!     (documents the gap: computed data never reaches CLI output).
//!
//! (3) An anergy-annotated workspace has the same audit output shape as one without
//!     anergy (delivery arm severed: no observable difference from outside).
//!
//! (4) A stale immunosuppress hides a defense gap from the audit -- the
//!     adversarial scenario team-lead named explicitly.
//!
//! Tests (1)-(4) PASS as documentation tests confirming current broken behavior.
//! When the delivery arm is fixed, the assertion senses must be inverted.
//!
//! ## Fix direction
//!
//! Wire `antigen::audit::audit_deferred_defenses(&scan_report, 30)` into
//! `run_audit()` in main.rs. Human output: always print a prominent section for
//! active anergy/immunosuppress declarations. JSON output: include the
//! `DeferredDefenseAuditReport` as a top-level field `deferred_defense_audit`.
//! Key invariant: zero active deferred-defense declarations = silent. One or
//! more = loudly announced.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Stage a crate with an explicit #[anergy] declaration on a presents-site.
fn staged_with_anergy() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"use antigen::{antigen, presents, anergy};

#[antigen(
    name = "known-gap",
    family = "functional-correctness",
    fingerprint = "item = struct",
    summary = "A known failure class with an anergy-suppressed site."
)]
pub struct KnownGap;

#[presents(KnownGap)]
#[anergy(reason = "no energy for this post-v0.3", until = "2999-01-01")]
pub struct AnergySite;
"#,
    )
    .unwrap();

    (tmp, src_dir)
}

/// Stage a crate with an #[immunosuppress] declaration.
fn staged_with_immunosuppress() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"use antigen::{antigen, presents, immunosuppress};

#[antigen(
    name = "suppressed-gap",
    family = "functional-correctness",
    fingerprint = "item = struct",
    summary = "A failure class whose defense is temporarily suppressed."
)]
pub struct SuppressedGap;

#[presents(SuppressedGap)]
#[immunosuppress(rationale = "mid-refactor defense landing in PR42")]
pub struct ImmunoPresentsSite;
"#,
    )
    .unwrap();

    (tmp, src_dir)
}

/// Stage a clean crate with no anergy or immunosuppress.
fn staged_clean() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"use antigen::{antigen, presents};

#[antigen(
    name = "clean-class",
    family = "functional-correctness",
    fingerprint = "item = struct",
    summary = "A failure class with no suppression."
)]
pub struct CleanClass;

#[presents(CleanClass)]
pub struct CleanSite;
"#,
    )
    .unwrap();

    (tmp, src_dir)
}

fn audit_human(root: &Path) -> (i32, String) {
    let out = Command::new(bin())
        .args(["antigen", "audit", "--root", root.to_str().unwrap()])
        .output()
        .expect("run audit");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    // audit writes progress to stderr; combine both for the check
    (out.status.code().unwrap_or(-1), stdout + &stderr)
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
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(&stdout).expect("audit JSON must parse")
}

// ============================================================================
// ATK-suppression-delivery-arm: human output is silent
// ============================================================================

#[test]
fn atk_suppression_human_output_silent_about_anergy() {
    // DELIVERY ARM WIRED (pathmaker): human output now LOUDLY announces the
    // active anergy declaration — naming the keyword, the reason, and a
    // deferred-defense section. The dev gets a clear signal the gap exists.
    let (_tmp, root) = staged_with_anergy();
    let (_exit, output) = audit_human(&root);

    assert!(
        output.to_lowercase().contains("anergy"),
        "ATK-suppression-delivery-arm: human audit output must LOUDLY mention the \
        active #[anergy] declaration. Output:\n{}",
        output
    );
    assert!(
        output.contains("no energy for this"),
        "ATK-suppression-delivery-arm: the anergy reason must be surfaced (LOUD). \
        Output:\n{}",
        output
    );
    assert!(
        output.to_lowercase().contains("deferred"),
        "ATK-suppression-delivery-arm: output must carry the deferred-defense \
        section header. Output:\n{}",
        output
    );
}

#[test]
fn atk_suppression_human_output_silent_about_immunosuppress() {
    // DOCUMENTS THE GAP: human output says nothing about the immunosuppress
    // declaration. An author reading `cargo antigen audit` output has no
    // signal that an active suppression exists on a presents-site.
    let (_tmp, root) = staged_with_immunosuppress();
    let (_exit, output) = audit_human(&root);

    // DELIVERY ARM WIRED: human output now announces the active immunosuppress.
    // NOTE: "suppressed" alone is ambiguous (fixture struct is "SuppressedGap");
    // assert on the keyword "immunosuppress" + the rationale text the audit emits.
    let mentions_immunosuppress =
        output.to_lowercase().contains("immunosuppress") && output.contains("mid-refactor");

    assert!(
        mentions_immunosuppress,
        "ATK-suppression-delivery-arm: human audit output must LOUDLY announce the \
        active #[immunosuppress] (keyword + rationale 'mid-refactor'). Output:\n{}",
        output
    );
}

// ============================================================================
// ATK-suppression-delivery-arm: JSON output missing deferred_defense_audit
// ============================================================================

#[test]
fn atk_suppression_json_output_has_no_deferred_defense_audit_field() {
    // DELIVERY ARM WIRED: the JSON audit output now carries the
    // `deferred_defense_audit` field (the DeferredDefenseAuditReport), with the
    // anergy declaration reflected in the counts.
    let (_tmp, root) = staged_with_anergy();
    let doc = audit_json(&root);

    let deferred = doc
        .get("deferred_defense_audit")
        .expect("audit JSON must carry deferred_defense_audit (the delivery arm)");
    assert!(
        deferred["audits"].as_array().is_some_and(|a| !a.is_empty()),
        "deferred_defense_audit.audits must contain the anergy declaration; got {deferred:?}"
    );
    assert_eq!(
        deferred["active_count"].as_u64(),
        Some(1),
        "the active anergy declaration must be counted active; got {deferred:?}"
    );
}

// ============================================================================
// ATK-suppression-delivery-arm: anergy workspace indistinguishable from clean
//
// The most alarming consequence of the severed delivery arm: a workspace with
// active anergy declarations produces audit output that contains no anergy
// mention -- identical silence to a clean workspace.
// ============================================================================

#[test]
fn atk_suppression_anergy_workspace_is_silent_in_audit_output() {
    // Confirm that even with an anergy declaration present, the audit output
    // says nothing about it. Both workspace types are equally silent.
    let (_tmp_anergy, root_anergy) = staged_with_anergy();
    let (_tmp_clean, root_clean) = staged_clean();

    let (_exit_a, output_anergy) = audit_human(&root_anergy);
    let (_exit_c, output_clean) = audit_human(&root_clean);

    // DELIVERY ARM WIRED: the anergy workspace is now DISTINGUISHABLE from clean.
    // The anergy output announces the suppression; the clean output stays silent
    // (the LOUD invariant: zero deferred defenses = silent, ≥1 = announced).
    assert!(
        output_anergy.to_lowercase().contains("anergy"),
        "ATK-suppression-delivery-arm: anergy workspace audit output must announce \
        the anergy declaration. Output:\n{}",
        output_anergy
    );
    assert!(
        !output_clean.to_lowercase().contains("anergy"),
        "ATK-suppression-delivery-arm: clean workspace (no deferred defenses) must \
        stay silent — LOUD only when present, not noise when absent: {}",
        output_clean
    );
}

// ============================================================================
// ATK-suppression-hide-behind-old-immunosuppress:
//
// The adversarial scenario team-lead named: can you hide a defense-gap behind
// an old immunosuppress? Yes -- trivially. The audit never surfaces the
// immunosuppress at all, so a stale one (author long gone, PR never landed) is
// indistinguishable from a fresh one in progress.
// ============================================================================

#[test]
fn atk_suppression_old_immunosuppress_hides_gap_from_audit() {
    // Stage a workspace with #[immunosuppress] whose rationale is stale.
    // The PR referenced was closed without landing; the suppression has
    // accumulated as invisible debt. The audit gives no signal.
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        src_dir.join("lib.rs"),
        r#"use antigen::{antigen, presents, immunosuppress};

#[antigen(
    name = "hidden-gap",
    family = "functional-correctness",
    fingerprint = "item = struct",
    summary = "A gap hidden behind a stale immunosuppress."
)]
pub struct HiddenGap;

#[presents(HiddenGap)]
#[immunosuppress(rationale = "temp fix from 2020 PR1 closed without landing")]
pub struct StaleImmunoPresentsSite;
"#,
    )
    .unwrap();

    let (_exit, output) = audit_human(&src_dir);

    // DELIVERY ARM WIRED: the stale immunosuppress can no longer hide. The audit
    // LOUDLY surfaces it (keyword + rationale), so accumulated old suppressions
    // are visible debt, not silent. (team-lead's named adversarial scenario —
    // "can you hide a gap behind an old immunosuppress?" — now answered NO.)
    let mentions_suppression =
        output.to_lowercase().contains("immunosuppress") && output.contains("temp fix");

    assert!(
        mentions_suppression,
        "ATK-suppression-hide: audit must surface the stale immunosuppress (keyword \
        + rationale) so it cannot hide a defense gap as silent debt. Output:\n{}",
        output
    );
}

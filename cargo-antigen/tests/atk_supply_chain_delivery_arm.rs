//! Adversarial test: supply-chain delivery arm -- `audit_supply_chain()` result
//! must reach `cargo antigen audit` output (human + JSON).
//!
//! ## The failure this guards (forward/audit-delivery-completeness-antigen,
//!    dogfood antigen #25 `AuditVerdictComputedButNotDelivered`)
//!
//! `audit_supply_chain()` in antigen/src/audit.rs correctly computes a
//! `SupplyChainAuditReport` (per-leaf `dep_pinned` / `dep_attested` / etc. hints
//! with pass/fail counts). But `cargo antigen audit` has ZERO references to
//! `audit_supply_chain` or `SupplyChainAuditReport` in main.rs. The function
//! is computed, exercised by unit tests, fully wired in the library -- and then
//! the result is silently dropped. The delivery arm is severed.
//!
//! The existing `cargo antigen verify` subcommand evaluates supply-chain leaves
//! per-dep using the lower-level `evaluate_*` fns directly -- it is NOT the same
//! pipeline. `audit_supply_chain()` walks `#[immune(requires=<pred>)]` immunity
//! declarations and evaluates the predicate tree (with combinator semantics) as
//! an aggregate workspace-level report. This workspace-scoped verdict is the one
//! that is severed.
//!
//! ## What these tests assert (current broken behavior as documentation tests)
//!
//! (1) `cargo antigen audit --format json` on a workspace with a supply-chain
//!     `#[immune(X, requires = dep_pinned())]` does NOT include a
//!     `supply_chain_audit` key in the JSON -- the computed verdict is dropped.
//!
//! (2) Human output of `cargo antigen audit` says nothing about the
//!     `dep_pinned` result -- the adopter cannot see whether their supply-chain
//!     immunity claim passed or failed.
//!
//! Tests (1) and (2) PASS as documentation tests confirming current broken behavior.
//! When the delivery arm is fixed, the assertion senses must be INVERTED:
//!   - JSON test: change `is_none()` to `expect("supply_chain_audit (delivery arm)")`.
//!   - Human test: change `!mentions_supply_chain` to `assert!(mentions_supply_chain, ...)`.
//!
//! ## Fix direction (per pathmaker note in forward/audit-delivery-completeness-antigen)
//!
//! Wire `antigen::audit::audit_supply_chain(&scan_report, &args.root)` into
//! `run_audit()` in main.rs alongside the existing audit calls. Add a
//! `supply_chain_audit` field to `JsonAuditReport`. Human render: show a
//! pass/fail summary section (model on the deferred-defense section). Design
//! consideration: the UX relationship with `cargo antigen verify` (per-dep
//! drill-down) vs `cargo antigen audit` (workspace-aggregate) must not create
//! two competing supply-chain surfaces. Recommended shape: audit shows aggregate
//! counts + flagged entries; verify shows per-dep details.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Stage a crate with an `#[immune]` declaration using a supply-chain
/// `dep_pinned()` predicate. This is the minimal case that causes
/// `audit_supply_chain()` to produce a non-empty `SupplyChainAuditReport`.
fn staged_with_supply_chain_immune() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src_dir = tmp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // Declare an antigen + an immunity with dep_pinned substrate-witness.
    // dep_pinned() will evaluate to NotAllPinned / AllPinned depending on whether
    // the staged workspace has a Cargo.toml with pinned deps. There is no
    // Cargo.toml here, so evaluate_dep_pinned() will fail gracefully. What matters
    // is that audit_supply_chain() PRODUCES an entry -- the delivery gap is that
    // the entry is never rendered, not that it passes/fails.
    std::fs::write(
        src_dir.join("lib.rs"),
        "use antigen::{antigen, immune};\n\
         \n\
         #[antigen(\n\
             name = \"unpinned-dep\",\n\
             family = \"supply-chain\",\n\
             fingerprint = \"item = fn\",\n\
             summary = \"Dependency is not exact-version pinned in Cargo.toml.\"\n\
         )]\n\
         pub struct UnpinnedDep;\n\
         \n\
         #[immune(UnpinnedDep, requires = dep_pinned())]\n\
         pub fn dep_pinned_site() {}\n",
    )
    .unwrap();

    (tmp, src_dir)
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

fn audit_human(root: &Path) -> String {
    let out = Command::new(bin())
        .args(["antigen", "audit", "--root", root.to_str().unwrap()])
        .output()
        .expect("run audit");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    stdout + &stderr
}

// ============================================================================
// ATK-supply-chain-delivery-arm: JSON output missing supply_chain_audit field
//
// audit_supply_chain() computes a SupplyChainAuditReport and then drops it.
// The --format json output has no supply_chain_audit key. The adopter running
// CI against JSON audit output cannot programmatically check supply-chain state
// via the audit command.
// ============================================================================

#[test]
fn atk_supply_chain_audit_json_field_absent() {
    // DELIVERY ARM SEVERED: audit_supply_chain() is computed but never attached
    // to JsonAuditReport. The supply_chain_audit key is absent from JSON output.
    //
    // FIX: add `supply_chain_audit: &audit::SupplyChainAuditReport` to
    // JsonAuditReport in main.rs. When fixed, invert this assertion:
    //   `doc.get("supply_chain_audit").expect("supply_chain_audit (delivery arm)")`
    let (_tmp, root) = staged_with_supply_chain_immune();
    let doc = audit_json(&root);

    // Documents the gap: the field is absent from JSON output.
    assert!(
        doc.get("supply_chain_audit").is_none(),
        "ATK-supply-chain-delivery-arm: when this test STARTS FAILING, the delivery \
         arm has been fixed -- invert this assertion to assert the field is PRESENT \
         and non-empty. (The fix is correct; the test shape was documenting the gap.) \
         supply_chain_audit field found: {:?}",
        doc.get("supply_chain_audit")
    );
}

// ============================================================================
// ATK-supply-chain-delivery-arm: human output silent about supply-chain results
//
// An adopter running `cargo antigen audit` to check whether their dep_pinned()
// immunity claims passed gets no signal. The audit runs, computes, drops.
// ============================================================================

#[test]
fn atk_supply_chain_audit_human_output_silent() {
    // DELIVERY ARM SEVERED: the audit human output says nothing about supply-chain
    // immunity evaluation. The adopter cannot see whether dep_pinned() passed.
    //
    // FIX: add a supply-chain section to the human render path in run_audit().
    // When fixed, invert this assertion to assert the output CONTAINS supply-chain
    // result information (e.g., "supply-chain" or "dep_pinned" in the output).
    let (_tmp, root) = staged_with_supply_chain_immune();
    let output = audit_human(&root);

    // Documents the gap: the human output is silent about supply-chain results.
    let mentions_supply_chain = output.to_lowercase().contains("supply-chain audit")
        || output.to_lowercase().contains("dep_pinned audit")
        || output.to_lowercase().contains("supply chain audit");

    assert!(
        !mentions_supply_chain,
        "ATK-supply-chain-delivery-arm: when this test STARTS FAILING, the delivery \
         arm has been fixed -- invert this assertion to assert supply-chain RESULTS \
         ARE mentioned. Output:\n{}",
        output
    );
}

// ============================================================================
// ATK-supply-chain-delivery-arm: spec for the fixed output shape
//
// This test is currently IGNORED (would fail if supply_chain_audit is absent).
// It documents what the fixed output SHOULD contain, as a specification for
// pathmaker implementing the fix. Once fixed, un-ignore and delete the gap-
// documenting tests above.
// ============================================================================

#[test]
#[ignore = "specification for the supply-chain delivery arm fix -- un-ignore when fixed"]
fn atk_supply_chain_audit_json_field_present_after_fix() {
    // When fixed, `cargo antigen audit --format json` must:
    // (1) Include `supply_chain_audit` key at the top level.
    // (2) `supply_chain_audit.audits` must be an array.
    // (3) `supply_chain_audit.pass_count` + `fail_count` must be present.
    // (4) With our dep_pinned() immunity and no Cargo.toml, fail_count >= 1
    //     (dep_pinned() evaluates to a failure when no Cargo.toml exists).
    let (_tmp, root) = staged_with_supply_chain_immune();
    let doc = audit_json(&root);

    let sc = doc
        .get("supply_chain_audit")
        .expect("supply_chain_audit must be in JSON audit output after fix");

    assert!(
        sc.get("audits").and_then(|a| a.as_array()).is_some(),
        "supply_chain_audit.audits must be an array; got {:?}",
        sc
    );
    assert!(
        sc.get("pass_count").is_some() && sc.get("fail_count").is_some(),
        "supply_chain_audit must have pass_count and fail_count; got {:?}",
        sc
    );
    // dep_pinned() with no Cargo.toml produces a failure entry.
    assert!(
        sc["fail_count"].as_u64().unwrap_or(0) >= 1,
        "dep_pinned() with no Cargo.toml must fail; got supply_chain_audit: {:?}",
        sc
    );
}

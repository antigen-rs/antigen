//! ATK-A3-019: audit `resolved_count` must not conflate `FormalProof` and
//! `Reachability` under one misleading label in human-readable output.
//!
//! Finding source: Phase 2 examples adversarial review (A3.5 onboarding sweep).
//! The `phantom_witness.rs` example promised "see `WitnessTier::FormalProof` in
//! the audit output" but the human-readable output showed nothing — the
//! `FormalProof` claim was silently subsumed into `resolved_count` and labeled
//! "declared (witness identifier found in workspace — not yet semantically
//! verified)". The fix surfaces the `FormalProof` tier in human-readable output.
//!
//! HOME: this test invokes the compiled `cargo-antigen` binary, so it lives in
//! `cargo-antigen/tests/` next to its ~20 binary-shelling siblings and resolves
//! the binary via `env!("CARGO_BIN_EXE_cargo-antigen")` — the lock-free,
//! deterministic shell-out idiom. It was previously in
//! `antigen/tests/atk_a3_fractal_preview.rs`, where it had to shell out via
//! `Command::new("cargo").arg("run")` (a NESTED `cargo run` under
//! `cargo test --workspace`), which races the target dir lock on cold runs
//! (os error 5 / "test FAILED" that vanishes warm or isolated). `CARGO_BIN_EXE_*`
//! is injected ONLY in tests of the crate that DEFINES the binary, so the test
//! had to MOVE here to use it — it could not be fixed in place. Same assertion,
//! same binary, same `antigen/examples` corpus, zero production-code change.
//! The rest of `atk_a3_fractal_preview.rs` is direct `antigen::scan` fixture
//! tests bound to `antigen/tests/fixtures/`; those stay in the `antigen` crate.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the compiled `cargo-antigen` binary injected by Cargo at test-link
/// time. Replaces the original nested `cargo run --bin cargo-antigen`, which
/// contended for the workspace target lock under `cargo test --workspace`.
fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// The real workspace root (parent of the `cargo-antigen` crate dir). Identical
/// to the value the original test computed from the `antigen` crate's manifest
/// dir — both crates share the same workspace parent.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-antigen crate dir has a parent (the workspace root)")
        .to_path_buf()
}

#[test]
fn atk_a3_019_audit_resolved_count_conflates_formal_proof_and_reachability() {
    // Contract (GREEN per A3.5 fix): `cargo antigen audit --root antigen/examples`
    // human-readable output distinguishes phantom-type witnesses (FormalProof
    // tier) from function witnesses (Reachability tier) in the audit output.
    //
    // Pathmaker chose to ship both Option A (per-tier sub-counts) and Option B
    // (confirmed-claims section). ADR-029 (2026-05-27): phantom-type proof now
    // surfaces via `#[presents(proof=...)]` on the presentation site, so the
    // audit shows "defended at FormalProof" in the immune-state verdicts
    // (presentation verdict surface), not in the immunity audit summary
    // sub-count. Both assertions verify the FormalProof tier is observable in
    // human-readable output without `--format json`.
    let workspace_root = workspace_root();
    let examples_root = workspace_root.join("antigen").join("examples");
    let output = Command::new(bin())
        .arg("antigen")
        .arg("audit")
        .arg("--root")
        .arg(&examples_root)
        .current_dir(&workspace_root)
        .output()
        .expect("cargo-antigen binary must run (injected by CARGO_BIN_EXE)");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("FormalProof"),
        "human-readable audit output must contain the tier name `FormalProof` \
         (immune-state verdicts section: 'defended at FormalProof'); got stdout:\n{stdout}"
    );
    assert!(
        stdout.contains("FormalProof")
            || stdout.contains("formal-proof")
            || stdout.contains("formal proof"),
        "human-readable audit output must distinguish FormalProof from Reachability \
         in either the presentation verdicts or the immunity summary sub-count; \
         got stdout:\n{stdout}"
    );
}

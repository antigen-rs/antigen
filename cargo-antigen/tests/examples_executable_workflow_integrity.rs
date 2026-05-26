//! Executable-workflow integrity: an example's own documented operator
//! workflow MUST actually be runnable end-to-end and reach the tier the
//! example claims.
//!
//! ## Why this test exists (the failure-class it defends)
//!
//! `antigen/examples/substrate_witness.rs` documents a four-step operator
//! workflow (scan → scaffold → sign → audit) and claims the immunity climbs
//! to the `Execution` tier once the sidecar is signed. But documentation
//! drifts from implementation silently: at one point the example's `#[immune]`
//! predicate referenced `docs/disciplines/ieee754-odd-functions.md` — a doc
//! that did not exist — so the `ratified_doc` leaf could never pass and the
//! documented `Execution`-tier outcome was *unreachable*. The example read as
//! correct; only running its own workflow exposed the gap. (That was instance
//! #5 of "example can't satisfy its own predicate"; the doc-gap is now fixed.)
//!
//! This is the CLASS defense aristotle + naturalist seeded under the
//! `examples-ci-executable-workflow-integrity` campsite: a test that EXECUTES
//! the example's documented workflow and asserts the claimed tier, so a future
//! drift (renamed flag, removed doc, changed predicate, regressed fingerprint
//! producer) FAILS HERE at CI time instead of waiting for an adopter to trip on
//! it. The example becomes its own witness — living documentation that checks
//! itself.
//!
//! ## What the workflow exercises (each step pins a real dependency)
//!
//! - `scan --format json` must emit an obtainable `structural_fingerprint`
//!   (the F6 producer) — the scaffold/sign steps need it.
//! - `attest scaffold` must create the sidecar at the immune site.
//! - `attest sign --signer alice --role math-researcher` must satisfy the
//!   `signers(required = ["alice"], roles = {alice = "math-researcher"})` leaf
//!   (the F5 `roles=` DSL).
//! - The `ratified_doc(path = "docs/disciplines/ieee754-odd-functions.md",
//!   min_version = "1.0")` leaf must resolve a real doc with a `version >= 1.0`
//!   frontmatter.
//! - `fresh_within_days(180)` must pass against the just-written signature date.
//! - `audit` must then report `tier = Execution`.
//!
//! ## A real integrity constraint this test encodes
//!
//! The audit resolves the `ratified_doc` path with `std::fs::read_to_string`
//! directly — i.e. relative to the PROCESS WORKING DIRECTORY, not relative to
//! `--root`. So the documented workflow only reaches `Execution` when run from
//! the workspace root (where `docs/disciplines/...` resolves). The test
//! reproduces exactly that condition: a temp workspace whose layout mirrors the
//! real relative paths, with the binary's CWD set to the temp root. If a future
//! change makes the doc path `--root`-relative instead, this test keeps passing
//! only if the example's documented invocation keeps matching reality.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the compiled `cargo-antigen` binary injected by Cargo.
fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Path to the real workspace root (parent of the `cargo-antigen` crate dir).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cargo-antigen crate dir has a parent (the workspace root)")
        .to_path_buf()
}

/// Run the binary with a given working directory; return `(exit_code, stdout, stderr)`.
fn run_in(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Build a temp workspace mirroring the real relative layout the example's
/// documented workflow needs:
///
/// ```text
/// <tmp>/
///   antigen/examples/substrate_witness.rs    (copied from the real example)
///   docs/disciplines/ieee754-odd-functions.md (copied from the real doc)
/// ```
///
/// Returns the temp dir handle (kept alive by the caller) and its root path.
fn staged_workspace() -> (tempfile::TempDir, PathBuf) {
    let src_root = workspace_root();
    let example_src = src_root.join("antigen/examples/substrate_witness.rs");
    let doc_src = src_root.join("docs/disciplines/ieee754-odd-functions.md");

    assert!(
        example_src.exists(),
        "the substrate_witness example must exist in the real tree: {}",
        example_src.display()
    );
    assert!(
        doc_src.exists(),
        "the ratified discipline doc must exist in the real tree (this is the \
         doc-gap fix that unblocked the example's workflow): {}",
        doc_src.display()
    );

    let tmp = tempfile::tempdir().expect("create temp workspace");
    let root = tmp.path().to_path_buf();

    let example_dst = root.join("antigen/examples/substrate_witness.rs");
    std::fs::create_dir_all(example_dst.parent().unwrap()).unwrap();
    std::fs::copy(&example_src, &example_dst).expect("copy example into temp workspace");

    let doc_dst = root.join("docs/disciplines/ieee754-odd-functions.md");
    std::fs::create_dir_all(doc_dst.parent().unwrap()).unwrap();
    std::fs::copy(&doc_src, &doc_dst).expect("copy discipline doc into temp workspace");

    (tmp, root)
}

/// Extract the obtainable `structural_fingerprint` for the
/// `SignedZeroDiscipline` *immunity* (not the antigen declaration) from
/// `scan --format json`. This is the F6 producer the scaffold/sign steps
/// depend on; a regression that drops the field (or the immunity) fails here
/// with a clear message rather than producing a silently-broken workflow.
///
/// We must key on the immunity entry specifically — `SignedZeroDiscipline`
/// appears in the JSON as both an antigen `type_name` (which carries the
/// *fingerprint grammar string*, NOT a `structural_fingerprint` hash) and as
/// an immunity `antigen_type` (which carries the obtainable
/// `structural_fingerprint` the workflow needs). String-scanning for the bare
/// name lands on the antigen block first and grabs an unrelated immunity's
/// fingerprint — so parse structurally.
fn fingerprint_from_scan(cwd: &Path) -> String {
    let (code, stdout, stderr) = run_in(
        cwd,
        &[
            "antigen",
            "scan",
            "--root",
            "antigen/examples",
            "--format",
            "json",
        ],
    );
    assert_eq!(code, 0, "scan --format json must exit 0: {stderr}");

    let doc: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("scan --format json must emit valid JSON ({e}):\n{stdout}"));
    let immunities = doc["report"]["immunities"]
        .as_array()
        .unwrap_or_else(|| panic!("scan JSON must have report.immunities array:\n{stdout}"));

    let immunity = immunities
        .iter()
        .find(|i| i["antigen_type"] == "SignedZeroDiscipline")
        .unwrap_or_else(|| panic!("scan must capture a SignedZeroDiscipline immunity:\n{stdout}"));

    let fp = immunity["structural_fingerprint"]
        .as_str()
        .unwrap_or_else(|| {
            panic!(
                "the SignedZeroDiscipline immunity must carry an obtainable \
                 structural_fingerprint (F6 producer); immunity = {immunity}"
            )
        });
    assert!(
        fp.contains(':'),
        "structural_fingerprint should be `<algo>:<hex>`, got `{fp}`"
    );
    fp.to_string()
}

/// Locate the `SignedZeroDiscipline` block in human-format audit output and
/// return the `tier = ...` token reported for it.
///
/// Human audit prints, for each immunity:
/// ```text
///   <file>:<line>  SignedZeroDiscipline (witness = ``)
///     tier = Execution, hint = DisciplinePredicatePassedSubstrateCurrent
/// ```
fn audit_tier_for_signed_zero(audit_stdout: &str) -> String {
    let mut lines = audit_stdout.lines();
    while let Some(line) = lines.next() {
        if line.contains("SignedZeroDiscipline") && line.contains("witness") {
            let tier_line = lines.next().unwrap_or_else(|| {
                panic!("SignedZeroDiscipline header had no following tier line")
            });
            // tier_line looks like: `    tier = Execution, hint = ...`
            let after_tier = tier_line
                .split("tier =")
                .nth(1)
                .unwrap_or_else(|| panic!("expected `tier =` on line: {tier_line}"));
            let tier = after_tier
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            return tier;
        }
    }
    panic!("audit output did not contain a SignedZeroDiscipline immunity block:\n{audit_stdout}");
}

// ============================================================================
// The integrity test: run the documented workflow, assert Execution tier.
// ============================================================================

/// Execute the `substrate_witness.rs` documented four-step workflow against a
/// staged workspace and assert the immunity reaches `Execution` tier — exactly
/// as the example's doc-comment claims. This is the positive case: the example
/// is satisfiable end-to-end.
#[test]
fn substrate_witness_example_workflow_reaches_execution_tier() {
    let (_tmp, root) = staged_workspace();

    // STEP 1 (scan): obtain the structural fingerprint the workflow needs.
    let fingerprint = fingerprint_from_scan(&root);

    let sidecar = root.join("antigen/examples/.attest/SignedZeroDiscipline.json");

    // STEP 2 (scaffold): create the sidecar at the immune site.
    let (code, _out, stderr) = run_in(
        &root,
        &[
            "antigen",
            "attest",
            "scaffold",
            "--antigen",
            "SignedZeroDiscipline",
            "--source-file",
            "antigen/examples/substrate_witness.rs",
            "--item-path",
            "signed_zero_preserving_sinh",
            "--fingerprint",
            &fingerprint,
        ],
    );
    assert_eq!(code, 0, "scaffold must succeed: {stderr}");
    assert!(sidecar.exists(), "scaffold must create the sidecar file");

    // STEP 3 (sign): record WHO reviewed, with the role the predicate requires.
    // The `--role math-researcher` is what satisfies the
    // `roles = {alice = "math-researcher"}` clause (F5 roles= DSL).
    let (code, _out, stderr) = run_in(
        &root,
        &[
            "antigen",
            "attest",
            "sign",
            "--sidecar",
            sidecar.to_str().unwrap(),
            "--item-path",
            "signed_zero_preserving_sinh",
            "--signer",
            "alice",
            "--role",
            "math-researcher",
            "--fingerprint",
            &fingerprint,
            "--reasoning",
            "reviewed sinh body: explicit x == 0.0 short-circuit preserves the sign bit",
        ],
    );
    assert_eq!(code, 0, "sign must succeed: {stderr}");

    // STEP 4 (audit): the documented Execution-tier outcome must now be real.
    let (code, stdout, stderr) = run_in(&root, &["antigen", "audit", "--root", "antigen/examples"]);
    assert_eq!(code, 0, "audit must exit 0: {stderr}");

    let tier = audit_tier_for_signed_zero(&stdout);
    assert_eq!(
        tier, "Execution",
        "the substrate_witness example documents that its signed sidecar climbs to \
         the Execution tier. If this is not `Execution`, the example's own workflow \
         no longer reaches the tier it claims — a doc↔impl drift the example can no \
         longer be trusted to demonstrate. Full audit:\n{stdout}"
    );

    // The Execution-tier outcome must be the substrate-current predicate-pass,
    // not some unrelated tier coincidence. The hint is printed on the tier line.
    assert!(
        stdout.contains("DisciplinePredicatePassedSubstrateCurrent"),
        "Execution tier here must come from the substrate predicate passing \
         (all three leaves: signers+role, ratified_doc, fresh_within_days). \
         Audit:\n{stdout}"
    );
}

/// Negative control: BEFORE the sidecar is signed, the immunity must NOT be at
/// Execution tier (it should report sidecar-missing). This proves the
/// positive test's `Execution` result is *caused by* running the workflow, not
/// a tier the example reports unconditionally — so a regression that makes
/// every claim falsely report Execution would be caught.
#[test]
fn substrate_witness_example_is_not_execution_tier_without_signing() {
    let (_tmp, root) = staged_workspace();

    // No scaffold, no sign — just audit the freshly-staged example.
    let (code, stdout, stderr) = run_in(&root, &["antigen", "audit", "--root", "antigen/examples"]);
    assert_eq!(code, 0, "audit must exit 0 even with no sidecar: {stderr}");

    let tier = audit_tier_for_signed_zero(&stdout);
    assert_ne!(
        tier, "Execution",
        "without a signed sidecar the substrate-witness immunity must NOT report \
         Execution tier — that would mean the tier is unconditional and the \
         positive test proves nothing. Audit:\n{stdout}"
    );
    assert!(
        stdout.contains("DisciplineSidecarMissing"),
        "the unsigned example must report the sidecar-missing diagnostic (the real \
         next-step prompt for the operator). Audit:\n{stdout}"
    );
}

//! ATK — `cargo antigen propose` (Island 3, the keystone-goes-live surface), BORN-RED.
//!
//! Island 3 wires the FIRST production caller of the learning core: `cargo antigen
//! propose` takes a cluster of marked sites + an operator-supplied clean corpus,
//! routes them through `propose()` (anti-unify → GATE-G), and RENDERS the outcome.
//! It is greenfield (`command grep` 2026-06-12: no `Propose` variant / `run_propose`
//! / `ProposeArgs` / `--clean-root` in `cargo-antigen/src/main.rs`; `AntigenSubcommand`
//! @ :69 carries scan/audit/new/vaccinate). This file lands the test-classes that
//! DEFINE done before the pathmaker wires the caller (tests-first).
//!
//! # The spine: the CLI is PLUMBING; the GATE is SAFETY
//!
//! Every class below guards ONE invariant (the team-lead's framing): **wiring a
//! caller onto the gate must not move any safety decision OUT of the gate and INTO
//! the CLI.** The live `cargo antigen propose` never tries to *be* the safety — it
//! (1) *surfaces* the gate's route-to-human as a render, never manufactures a
//! promote; (2) *passes through* the operator's corpus, never auto-labels
//! unmarked=clean (the gate's spare-clean does the real check); (3) renders a
//! ratifiable suggestion typed on the token, never auto-asserts; (4) plumbs even a
//! *contaminated* corpus to the gate, which catches it — the CLI does not
//! pre-validate cleanliness. A safety decision that migrates into the CLI is the
//! failure this whole file defends against.
//!
//! Four claim-kinds the keystone-goes-live surface must defend:
//!
//! 1. **route-to-human regression (CLI level).** The library level is pinned in
//!    `antigen/tests/dogfood_honesty_guard.rs` (the real read-loop twins route-to-
//!    human, NOT promote). At the CLI, the same outcome must RENDER as a route-to-
//!    human suggestion (a `NotCorpusWitnessable` specimen for a human to ratify),
//!    never a silent promote — so a future change can't quietly flip the dogfood
//!    into a false promote at the CLI boundary.
//!
//! 2. **operator-supplied clean corpus (`--clean-root`).** The captain's CLEAN-
//!    CORPUS-SOURCE ruling: the clean corpus is OPERATOR-SUPPLIED, never auto-
//!    derived/auto-labeled. Antigen auto-labeling "unmarked = clean" IS the
//!    mislabeled-clean residual (ATK-047-4 — the gate trusting its own label =
//!    unsafe). So `propose` REQUIRES an explicit operator corpus source; there is no
//!    "scan the whole tree and assume the rest is clean" default.
//!
//! 3. **observe-don't-declare (ADR-044) — the keystone-goes-live safety line.** The
//!    render is a ratifiable SUGGESTION typed on the `PromotedDraft` capability
//!    token (ADR-048), NEVER an auto-`#[presents]` mark or an auto-named class. The
//!    machine supplies the syntactic half; the human/incident ratifies the semantic
//!    half. `cargo antigen propose` MUST NOT write a `#[presents]`/`#[antigen]` into
//!    the source — it emits a suggestion, full stop.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo antigen propose <args>` and capture (exit-code, stdout, stderr).
fn propose(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("propose")
        .args(args)
        .output()
        .expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Stage an empty cluster + clean dir pair (no marks). Enough to exercise the
/// arg-parse + dir-validation contract (a ≥2 cluster needs a COMPILED fixture so the
/// `#[dread]` macro emits the marked-unknown the scan reads — see the
/// `#[ignore]`'d render tests). Returns `(tempdir, cluster_root, clean_root)`.
fn staged_dirs() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cluster_src = tmp.path().join("cluster/src");
    let clean_src = tmp.path().join("clean/src");
    std::fs::create_dir_all(&cluster_src).unwrap();
    std::fs::create_dir_all(&clean_src).unwrap();
    // A clean sibling so --clean-root is non-empty (avoids the empty-corpus exit-2).
    std::fs::write(
        clean_src.join("lib.rs"),
        "impl Drop for CleanGuard { fn drop(&mut self) { let _ = flush().ok(); } }\n",
    )
    .unwrap();
    // Compute the returned roots BEFORE moving `tmp` into the tuple (borrow-after-move).
    let cluster_root = tmp.path().join("cluster");
    let clean_root = tmp.path().join("clean");
    (tmp, cluster_root, clean_root)
}

// ───────────────────────────────────────────────────────────────────────────
// Class 1 — route-to-human regression (CLI level)
// ───────────────────────────────────────────────────────────────────────────

/// ADR-047/051 — `propose_cli_renders_route_to_human_not_a_promote`. When the
/// cluster routes-to-human (`NotCorpusWitnessable` — the settled dogfood outcome),
/// the CLI RENDERS a needs-human-ratification suggestion, exit-code legible, and
/// does NOT report a promote. Pins the honest outcome at the CLI boundary so a
/// silent flip-to-false-promote is NOTICED here too.
/// VERIFIED-PARTIAL: the no-cluster informational path renders cleanly (exit 0, names
/// the ≥2-cluster requirement, never a false promote) — testable now without marks.
/// The FULL route-to-human RENDER (a ≥2 cluster that anti-unifies + routes-to-human)
/// needs a COMPILED fixture crate: `run_propose` re-acquires the cluster via the scan's
/// marked-unknown plane, which reads the `#[dread]` MACRO's emitted marker — a raw
/// `#[dread(...)]` attribute in an uncompiled tempdir is read as a fingerprint-match,
/// not a marked-unknown, so it never clusters. That render half stays `#[ignore]`'d
/// pending a compiled-fixture harness (see `propose_route_to_human_render_compiled`).
#[test]
fn propose_no_cluster_renders_informationally_never_a_false_promote() {
    let (tmp, cluster, clean) = staged_dirs();
    let (code, stdout, _stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    // An empty cluster is HONEST, not an error (exit 0): there is nothing to
    // anti-unify; the render says so and NEVER reports a promote.
    assert_eq!(
        code, 0,
        "an empty cluster is honest, not an error; stdout={stdout}"
    );
    assert!(
        stdout.contains("cluster") && stdout.contains("anti-unify"),
        "the no-cluster render must name the ≥2-cluster requirement; stdout={stdout}"
    );
    assert!(
        !stdout.to_lowercase().contains("promoted"),
        "an empty cluster must NEVER render a promote (no false-promote on no-input); \
         stdout={stdout}"
    );
    drop(tmp);
}

/// The FULL route-to-human RENDER on a real ≥2 cluster — needs a COMPILED fixture so
/// the `#[dread]` macro emits the marked-unknown the scan clusters on. Un-ignore when
/// a compiled-fixture harness lands (a fixture crate with a `Cargo.toml` depending on
/// `antigen`, built then scanned), OR the pathmaker adds a raw-attribute cluster path.
#[test]
#[ignore = "needs a COMPILED fixture crate (raw #[dread] in a tempdir is not a \
            marked-unknown until macro-expanded); un-ignore with a compiled-fixture harness"]
fn propose_route_to_human_render_compiled() {
    // SPEC (un-ignore with a compiled fixture):
    //   // A built crate with 2 shape-identical #[dread] Drop impls (the twins) +
    //   // a separate --clean-root with a .ok() sibling. The twins draft is many-
    //   // conjunct (no any_of), so NO near-miss exists → route-to-human.
    //   let (code, stdout, _) = propose(&["--cluster-root", built, "--clean-root", clean]);
    //   assert!(stdout.contains("route") || stdout.contains("ratif"),
    //       "a route-to-human cluster renders as needs-ratification, not a promote");
    //   assert!(!stdout.to_lowercase().contains("promoted to a named"),
    //       "the CLI must NOT report a promote for a route-to-human cluster");
    unimplemented!("compiled-fixture harness for the route-to-human render not yet built");
}

// ───────────────────────────────────────────────────────────────────────────
// Class 2 — operator-supplied clean corpus (--clean-root); no auto-label
// ───────────────────────────────────────────────────────────────────────────

/// CLEAN-CORPUS-SOURCE (captain's ruling) — `propose_requires_an_operator_clean_corpus`.
/// `propose` REQUIRES an explicit operator-supplied clean corpus (`--clean-root` or
/// equivalent); it must NOT auto-derive "everything unmarked is clean" (that is the
/// ATK-047-4 mislabeled-clean residual — the gate trusting its own label). Invoking
/// `propose` with NO clean-corpus source is a usage error, not a silent
/// auto-labeling.
/// VERIFIED against the live CLI (`a9396ad`+`run_propose`). A missing `--clean-root`
/// is a clap usage error (exit 2) — there is NO auto-derived "the rest of the tree is
/// clean" default. antigen never labels unmarked code clean (ADR-044/047, ATK-047-4).
#[test]
fn propose_requires_an_operator_clean_corpus() {
    let (tmp, cluster, _clean) = staged_dirs();
    let cluster = cluster.to_str().unwrap();
    // No --clean-root → clap REQUIRES it → exit 2, with the missing-arg named.
    let (code, _stdout, stderr) = propose(&["--cluster-root", cluster]);
    assert_eq!(
        code, 2,
        "propose without an operator clean corpus must be a usage error (exit 2), \
         never a silent auto-labeled run; stderr={stderr}"
    );
    assert!(
        stderr.contains("clean-root") && stderr.to_lowercase().contains("required"),
        "the usage error must name the missing --clean-root (ATK-047-4: antigen must \
         not auto-label unmarked code as clean); stderr={stderr}"
    );
    drop(tmp);
}

/// CLEAN-CORPUS-SOURCE — `propose_does_not_auto_label_unmarked_as_clean`. The
/// affirmative dual: the clean corpus comes SOLELY from `--clean-root`, never from
/// "scan the rest of the tree". CODE-TRUE assertion against the live `run_propose`
/// (cargo-antigen/src/main.rs): the clean corpus is `collect_clean_corpus(&args
/// .clean_root)` and NOTHING else — no `scan_workspace`-minus-marks auto-derivation.
/// (A grep-checkable invariant the CODE-TRUE audit owns; pinned here so a future
/// "convenience" auto-clean path is a test failure.)
#[test]
fn propose_does_not_auto_label_unmarked_as_clean() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("read cargo-antigen main.rs");
    // Locate run_propose's body and assert the clean corpus is built ONLY from
    // --clean-root (collect_clean_corpus(&args.clean_root)).
    let run_propose = src
        .split("fn run_propose(")
        .nth(1)
        .expect("run_propose exists");
    let body = &run_propose[..run_propose.find("\nfn ").unwrap_or(run_propose.len())];
    assert!(
        body.contains("collect_clean_corpus(&args.clean_root)"),
        "the clean corpus must come from --clean-root (collect_clean_corpus(&args.clean_root))"
    );
    // The anti-auto-label invariant: run_propose must NOT derive the clean corpus by
    // scanning the cluster root / the whole workspace and subtracting marks. If a
    // future edit adds such a path, this trips.
    assert!(
        !body.contains("scan_workspace")
            && !body.contains("collect_clean_corpus(&args.cluster_root)"),
        "run_propose must NOT auto-derive the clean corpus (no scan_workspace / cluster-root \
         clean-derivation) — the operator labels clean, antigen does not (ATK-047-4)"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// Class 3 — observe-don't-declare (ADR-044): the keystone-goes-live safety line
// ───────────────────────────────────────────────────────────────────────────

/// ADR-044 — `propose_emits_a_suggestion_never_an_auto_presents`. The
/// keystone-goes-live safety assertion: `cargo antigen propose` RENDERS a ratifiable
/// `PromotedDraft` suggestion; it MUST NOT write a `#[presents]` mark or a named
/// `#[antigen]` into any source file. The machine supplies the syntactic half; the
/// human/incident ratifies the semantic half. A `propose` run leaves the source tree
/// BYTE-UNCHANGED (it suggests, it does not declare).
/// VERIFIED against the live CLI. A `propose` run leaves the source tree
/// BYTE-UNCHANGED — it never auto-writes a `#[presents]`/`#[antigen]`. This holds on
/// EVERY path (even the no-cluster informational path), because observe-don't-declare
/// is structural: `run_propose` only READS + RENDERS, it has no source-writing path
/// at all. (The render-of-a-promoted-suggestion is exercised by the `#[ignore]`'d
/// compiled-fixture tests; the no-auto-write invariant is testable now and is the
/// load-bearing safety half — the machine never declares.)
#[test]
fn propose_emits_a_suggestion_never_an_auto_presents() {
    let (tmp, cluster, clean) = staged_dirs();
    // Snapshot the cluster + clean trees before the run.
    let before = snapshot_tree(tmp.path());
    let _ = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    let after = snapshot_tree(tmp.path());
    assert_eq!(
        before, after,
        "propose MUST NOT auto-write a #[presents]/#[antigen] into any source file — \
         it emits a ratifiable suggestion (observe-don't-declare, ADR-044); the human \
         ratifies the semantic half. The source tree must be byte-unchanged."
    );
    drop(tmp);
}

/// A (path → bytes) snapshot of every `.rs` file under `root`, for the
/// byte-unchanged assertion (observe-don't-declare).
fn snapshot_tree(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.insert(p.clone(), std::fs::read(&p).unwrap_or_default());
            }
        }
    }
    out
}

// ───────────────────────────────────────────────────────────────────────────
// Class 4 — plumbing-vs-safety division (the team-lead's spine): the CLI plumbs,
// the GATE decides. The unifying invariant: wiring a caller onto the gate must not
// move any safety decision OUT of the gate and INTO the CLI.
// ───────────────────────────────────────────────────────────────────────────

/// PLUMBING-VS-SAFETY — `propose_cli_plumbs_a_dirty_corpus_the_gate_catches_it`.
/// The corpus-cleanliness check lives in the GATE (spare-clean), NOT the CLI. If the
/// operator supplies a `--clean-root` that is *actually contaminated* (it contains a
/// marked/defect site the draft binds), the CLI does NOT pre-validate cleanliness and
/// reject up-front — it PLUMBS the corpus to `promote_if_safe`, and the GATE catches
/// it (`BindsCleanItem` — the draft binds a "clean" item → autoimmune → refused). This
/// pins WHERE the safety decision lives: a builder who "helpfully" adds corpus-
/// cleanliness validation to the CLI would be moving a safety decision out of the gate
/// (the wrong organ — ATK-047-4 is the gate's job, the gate trusting the operator's
/// label and checking it via spare-clean, not the CLI second-guessing the label).
/// CODE-TRUE half VERIFIED now; behavioral half needs a compiled fixture. The
/// plumbing-vs-safety invariant: `run_propose` has NO corpus-cleanliness validation of
/// its own — it `collect_clean_corpus`es the operator's `--clean-root` and plumbs it to
/// `propose()` (the gate's spare-clean is the sole cleanliness check). A builder who
/// added CLI-side cleanliness pre-validation would move a safety decision OUT of the
/// gate. This asserts the absence of any such CLI-side check (CODE-TRUE).
#[test]
fn propose_cli_does_not_pre_validate_corpus_cleanliness() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("read cargo-antigen main.rs");
    let run_propose = src
        .split("fn run_propose(")
        .nth(1)
        .expect("run_propose exists");
    let body = &run_propose[..run_propose.find("\nfn ").unwrap_or(run_propose.len())];
    // The corpus is plumbed straight to propose(); the gate decides safety. The CLI
    // must NOT call a cleanliness-validator on the corpus (no is_clean / spare_clean /
    // validate_corpus in run_propose — those live in the gate, the right organ).
    assert!(
        body.contains("propose::propose(&cluster, &clean_corpus)")
            || body.contains("propose(&cluster, &clean_corpus)"),
        "run_propose must plumb the corpus to the gate via propose() — the gate owns safety"
    );
    assert!(
        !body.contains("spare_clean")
            && !body.contains("is_clean")
            && !body.contains("validate_corpus"),
        "run_propose must NOT pre-validate corpus cleanliness at the CLI — the GATE's \
         spare-clean is the sole cleanliness check (don't move safety into the wrong organ)"
    );
}

/// The BEHAVIORAL half — a contaminated `--clean-root` (it contains a site the draft
/// binds) is caught by the GATE (`BindsCleanItem`), not pre-rejected by the CLI. Needs
/// a compiled fixture (same marked-unknown-needs-macro constraint). Un-ignore with the
/// compiled-fixture harness.
#[test]
#[ignore = "needs a COMPILED fixture crate (the contaminated corpus + the cluster both \
            need real marked-unknowns); un-ignore with a compiled-fixture harness"]
fn propose_cli_plumbs_a_dirty_corpus_the_gate_catches_it() {
    // SPEC (un-ignore with a compiled fixture):
    //   // --clean-root contains a Drop impl the draft BINDS (operator mislabeled clean):
    //   let (code, stdout, _) = propose(&["--cluster-root", built, "--clean-root", dirty]);
    //   assert!(stdout.contains("binds") || stdout.to_lowercase().contains("autoimmune"),
    //       "a contaminated corpus is caught by the GATE's spare-clean (BindsCleanItem), \
    //        not pre-validated by the CLI — the cleanliness check lives in the gate");
    unimplemented!("compiled-fixture harness for the dirty-corpus gate-catch not yet built");
}

/// VERIFIED — dir validation: a nonexistent `--cluster-root` / `--clean-root` is a
/// usage error (exit 2), named by flag. (`run_propose` validates both supplied paths.)
#[test]
fn propose_rejects_a_nonexistent_root() {
    let (tmp, _cluster, clean) = staged_dirs();
    let missing = tmp.path().join("does-not-exist");
    let (code, _stdout, stderr) = propose(&[
        "--cluster-root",
        missing.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "a nonexistent --cluster-root must be a usage error; stderr={stderr}"
    );
    assert!(
        stderr.contains("cluster-root") && stderr.contains("does not exist"),
        "the error must name the nonexistent --cluster-root; stderr={stderr}"
    );
    drop(tmp);
}

//! ATK — `cargo antigen propose` (Island 3, the keystone-goes-live surface).
//!
//! Island 3 wired the FIRST production caller of the learning core: `cargo antigen
//! propose` takes a cluster of marked sites + an operator-supplied clean corpus,
//! routes them through `propose()` (anti-unify → GATE-G), and RENDERS the outcome.
//! The caller is LIVE (`run_propose` / `ProposeArgs` / `--clean-root`, landed
//! `cd46dab`). These test-classes began born-red (tests-first — DEFINING done before
//! the wire) and now verify the live surface against the real binary: all three GATE-G
//! renders (route-to-human / promote→suggestion / autoimmune) on raw-text `#[dread]`
//! fixtures, plus the `--marker` parameterization — all green, none `#[ignore]`'d.
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
//! 2. **operator-supplied clean corpus (`--clean-root`).** The operator's CLEAN-
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

/// Stage an empty cluster + clean dir pair (no marks). Exercises the arg-parse +
/// dir-validation contract and the honest no-cluster render. For a real ≥2 cluster use
/// `staged_twins` (raw-text `#[dread]` twins — the scan reads the attribute
/// syntactically, so no compiled crate is needed). Returns
/// `(tempdir, cluster_root, clean_root)`.
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

/// The two `#[dread]`-marked fn twins: IDENTICAL bodies (same local idents) so their
/// name-insensitive `shape_digest`s match → the scan clusters them; the body carries a
/// real behavioral signal (a swallow-the-read-error directory walk) so the anti-unified
/// draft is NOT a bare-structural over-binder (it survives the C-side non-degeneracy
/// guard, ADR-056). Parsed-as-text by the scan — does NOT compile, does not need the
/// macro crate (the scan reads `#[dread]` SYNTACTICALLY from source; cf.
/// `antigen/tests/fixtures/marked_unknown_*`). This is what overturned the earlier
/// "needs a compiled fixture crate" premise: a raw `#[dread]` attribute clusters fine.
const TWINS: &str = r#"
#[dread(trigger = "this directory walk swallows a read error and continues, silently skipping entries")]
fn scan_dir_a(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            out.push(e.path().display().to_string());
        }
    }
    out
}

#[dread(trigger = "this directory walk also swallows the read error and continues; same shape")]
fn scan_dir_b(root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            out.push(e.path().display().to_string());
        }
    }
    out
}
"#;

/// `#[red_flag]` twins — a DIFFERENT marker-class than [`TWINS`] (red-flag, not dread),
/// identical bodies so they cluster. Used to prove `--marker` selects the marker-class
/// and the cluster mechanism is marker-agnostic. Raw-text; does not compile.
const RED_FLAG_TWINS: &str = r#"
#[red_flag(trigger = "this auth check is bypassable when the header is absent; high-certainty")]
fn auth_a(req: &Req) -> bool {
    let mut ok = false;
    if let Some(h) = req.header("authorization") {
        ok = verify(h);
    }
    ok
}

#[red_flag(trigger = "this auth check is bypassable when the header is absent; same shape")]
fn auth_b(req: &Req) -> bool {
    let mut ok = false;
    if let Some(h) = req.header("authorization") {
        ok = verify(h);
    }
    ok
}
"#;

/// Stage an arbitrary `cluster_src` + `clean_src` as a `cluster/`-and-`clean/` pair.
/// Returns `(tempdir, cluster_root, clean_root)`.
fn staged_cluster(cluster_src: &str, clean_src: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cluster_dir = tmp.path().join("cluster/src");
    let clean_dir = tmp.path().join("clean/src");
    std::fs::create_dir_all(&cluster_dir).unwrap();
    std::fs::create_dir_all(&clean_dir).unwrap();
    std::fs::write(cluster_dir.join("lib.rs"), cluster_src).unwrap();
    std::fs::write(clean_dir.join("lib.rs"), clean_src).unwrap();
    let cluster_root = tmp.path().join("cluster");
    let clean_root = tmp.path().join("clean");
    (tmp, cluster_root, clean_root)
}

/// `staged_cluster` with the canonical `#[dread]` [`TWINS`] cluster — the gate-outcome
/// tests all share it; the `clean_src` selects the outcome: unrelated code →
/// route-to-human (no near-miss); a near-miss sibling → promote; a near-miss + a binding
/// site → autoimmune (`BindsCleanItem`). (near-miss is gated BEFORE spare-clean, so the
/// autoimmune verdict requires a near-miss in the corpus too.)
fn staged_twins(clean_src: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    staged_cluster(TWINS, clean_src)
}

// ───────────────────────────────────────────────────────────────────────────
// Class 1 — route-to-human regression (CLI level)
// ───────────────────────────────────────────────────────────────────────────

/// ADR-047/051 — `propose_cli_renders_route_to_human_not_a_promote`. When the
/// cluster routes-to-human (`NotCorpusWitnessable` — the settled dogfood outcome),
/// the CLI RENDERS a needs-human-ratification suggestion, exit-code legible, and
/// does NOT report a promote. Pins the honest outcome at the CLI boundary so a
/// silent flip-to-false-promote is NOTICED here too.
/// VERIFIED here: the no-cluster informational path renders cleanly (exit 0, names the
/// ≥2-cluster requirement, never a false promote) — an EMPTY cluster is the honest
/// no-marks case. The FULL route-to-human / promote / autoimmune renders on a real ≥2
/// cluster are covered by `propose_route_to_human_render`,
/// `propose_promote_renders_a_ratifiable_suggestion_not_an_auto_presents`, and
/// `propose_cli_plumbs_a_dirty_corpus_the_gate_catches_it` — each staged with a
/// RAW-TEXT twins fixture (`staged_twins`). The scan reads `#[dread]` SYNTACTICALLY
/// from source, so no compiled crate is needed to produce a clustering marked-unknown.
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

/// The FULL route-to-human RENDER on a real ≥2 cluster (raw-text twins fixture). The
/// twins anti-unify into a behavioral draft, but an UNRELATED clean corpus holds NO
/// near-miss (no clean sibling is one discriminating constraint from binding the draft)
/// → `NotCorpusWitnessable` → the CLI renders a needs-ratification suggestion routed to
/// a human, NOT a promote. Pins the settled dogfood outcome at the CLI boundary so a
/// silent flip-to-false-promote is NOTICED here.
#[test]
fn propose_route_to_human_render() {
    // Unrelated clean code: shares no behavioral conjunct with the swallow-error draft,
    // so nothing is a near-miss → route-to-human.
    let (tmp, cluster, clean) = staged_twins(
        "fn add(a: i32, b: i32) -> i32 { a + b }\n\
         fn greet(n: &str) -> String { format!(\"hi {n}\") }\n",
    );
    let (code, stdout, _stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "route-to-human is an honest outcome (exit 0); stdout={stdout}"
    );
    let lower = stdout.to_lowercase();
    assert!(
        lower.contains("route") && lower.contains("ratif"),
        "a route-to-human cluster must render as routed-to-a-human-ratifier, not a \
         promote; stdout={stdout}"
    );
    assert!(
        !lower.contains("autoimmune") && !lower.contains("binds"),
        "route-to-human is the no-near-miss verdict, NOT the autoimmune one; stdout={stdout}"
    );
    drop(tmp);
}

/// `--marker` selects the marker-CLASS (`dread` / `aura` / `red-flag`); the cluster
/// mechanism is identical across all three (`assemble_marked_cluster` filters
/// `m.marker == marker` + groups by `shape_digest` — no marker-specific branching). The
/// keystone's gate-outcome tests all exercise the default `dread`; this pins the
/// `--marker` parameterization itself, BOTH ways: `#[red_flag]` twins cluster +
/// route-to-human under `--marker red-flag`, and the SAME marks produce NO cluster under
/// the default `--marker dread` (so the filter genuinely isolates by class — `--marker`
/// is honored, not cosmetic). propose is for the three marked-unknown markers ONLY, not
/// the declaration/defense macros (`#[antigen]`/`#[presents]`/`#[defended_by]`).
#[test]
fn propose_marker_arg_selects_the_marker_class() {
    let unrelated = "fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let (tmp, cluster, clean) = staged_cluster(RED_FLAG_TWINS, unrelated);
    let cluster = cluster.to_str().unwrap();
    let clean = clean.to_str().unwrap();

    // (1) red-flag twins under --marker red-flag → cluster + route-to-human.
    let (code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster,
        "--clean-root",
        clean,
        "--marker",
        "red-flag",
    ]);
    assert_eq!(
        code, 0,
        "red-flag route-to-human is exit 0; stdout={stdout}"
    );
    let lower = stdout.to_lowercase();
    assert!(
        lower.contains("red-flag") && lower.contains("route") && lower.contains("ratif"),
        "red-flag twins must cluster + route-to-human under --marker red-flag (the \
         mechanism is marker-agnostic); stdout={stdout}"
    );

    // (2) the SAME marks under the default --marker dread → NO cluster: the filter
    //     isolates by marker-class, so a red-flag site is invisible to a dread run.
    let (code2, stdout2, _e2) = propose(&["--cluster-root", cluster, "--clean-root", clean]);
    assert_eq!(code2, 0, "no-cluster is an honest exit 0; stdout={stdout2}");
    assert!(
        stdout2.contains("dread") && stdout2.contains("found 0"),
        "red-flag marks must NOT cluster under --marker dread — the marker filter \
         genuinely isolates by class (--marker is honored, not cosmetic); stdout={stdout2}"
    );
    drop(tmp);
}

/// PROMOTE → ratifiable SUGGESTION render (the keystone's payoff path, raw-text
/// fixture). When the cluster has discriminating diversity AND the corpus holds a
/// near-miss (witnessable) without a binding site, the gate promotes — and the CLI
/// renders the `PromotedDraft` as a ratifiable SUGGESTION (observe-don't-declare): it
/// names a candidate fingerprint and says "ratify by hand", and it MUST NOT auto-write
/// a `#[presents]`/`#[antigen]` (byte-unchanged tree). NOTE: this is a SYNTHETIC
/// fixture proving the render PATH — it is NOT a claim that antigen immunized itself
/// (the dogfood on antigen's OWN marks routes-to-human; the self-immunization payoff
/// is the v0.6 abstract-recall frontier).
#[test]
fn propose_promote_renders_a_ratifiable_suggestion_not_an_auto_presents() {
    // A near-miss-only clean corpus: the `?`-propagating sibling is one discriminating
    // constraint from the swallow-error draft (witnessable) and the draft does not bind
    // it (spares clean) → promote.
    let (tmp, cluster, clean) = staged_twins(
        "fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {\n\
        \x20   let mut out = Vec::new();\n\
        \x20   for e in std::fs::read_dir(root)? {\n\
        \x20       out.push(e?.path().display().to_string());\n\
        \x20   }\n\
        \x20   Ok(out)\n\
         }\n",
    );
    let before = snapshot_tree(tmp.path());
    let (code, stdout, _stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    let after = snapshot_tree(tmp.path());
    assert_eq!(
        code, 0,
        "a promote renders a suggestion (exit 0); stdout={stdout}"
    );
    let lower = stdout.to_lowercase();
    assert!(
        lower.contains("suggestion") || lower.contains("ratifiable") || lower.contains("candidate"),
        "a promoted draft must render as a ratifiable SUGGESTION (a candidate \
         fingerprint to ratify by hand); stdout={stdout}"
    );
    assert!(
        !lower.contains("routed to a human"),
        "with a near-miss in the corpus the gate PROMOTES — it must not render \
         route-to-human; stdout={stdout}"
    );
    assert_eq!(
        before, after,
        "even on a PROMOTE, propose MUST NOT auto-write a #[presents]/#[antigen] — it \
         emits a ratifiable suggestion; the human ratifies (observe-don't-declare, ADR-044)"
    );
    drop(tmp);
}

// ───────────────────────────────────────────────────────────────────────────
// Class 2 — operator-supplied clean corpus (--clean-root); no auto-label
// ───────────────────────────────────────────────────────────────────────────

/// CLEAN-CORPUS-SOURCE — `propose_requires_an_operator_clean_corpus`.
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
/// (cargo-antigen/src/main.rs): the clean corpus is collected from a `clean_root`
/// bound from `args.clean_root` (the operator-supplied `--clean-root`) and NOTHING
/// else — no `scan_workspace`-minus-marks auto-derivation. (A grep-checkable
/// invariant the CODE-TRUE audit owns; pinned here so a future "convenience"
/// auto-clean path is a test failure.)
///
/// Note: `--clean-root` is `Option<PathBuf>` (optional ONLY for the `--list-clusters`
/// dry-run, which never runs the gate); the gate path binds the local
/// `let Some(clean_root) = args.clean_root` and passes `clean_root` to
/// `collect_clean_corpus`. The invariant pinned here is the SOURCE of the clean
/// corpus, not the binding's spelling.
#[test]
fn propose_does_not_auto_label_unmarked_as_clean() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("read cargo-antigen main.rs");
    // Locate run_propose's body and assert the clean corpus is built ONLY from
    // --clean-root: the gate binds `clean_root` from `args.clean_root` and feeds it
    // to collect_clean_corpus.
    let run_propose = src
        .split("fn run_propose(")
        .nth(1)
        .expect("run_propose exists");
    let body = &run_propose[..run_propose.find("\nfn ").unwrap_or(run_propose.len())];
    assert!(
        body.contains("collect_clean_corpus(clean_root)"),
        "the clean corpus must come from --clean-root (collect_clean_corpus(clean_root))"
    );
    assert!(
        body.contains("let Some(clean_root) = args.clean_root"),
        "the gate's clean_root must be bound from --clean-root (args.clean_root), not derived"
    );
    // The anti-auto-label invariant: run_propose must NOT derive the clean corpus by
    // scanning the cluster root / the whole workspace and subtracting marks. The
    // --list-clusters preview DOES scan the cluster root, but it never builds a clean
    // corpus (the gate doesn't run), so a `scan_workspace` in the body is no longer a
    // proxy for auto-derivation — assert the precise auto-clean shapes instead.
    assert!(
        !body.contains("collect_clean_corpus(&args.cluster_root)")
            && !body.contains("collect_clean_corpus(cluster_root)"),
        "run_propose must NOT auto-derive the clean corpus from the cluster root — the \
         operator labels clean, antigen does not (ATK-047-4)"
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

/// The BEHAVIORAL half (raw-text twins fixture): a contaminated `--clean-root` is
/// caught by the GATE, not pre-rejected by the CLI. A subtle GATE-G fact makes this
/// precise — near-miss is gated BEFORE spare-clean, so a corpus with ONLY a binding
/// site routes-to-human (no near-miss). To actually reach the autoimmune verdict the
/// dirty corpus must ALSO hold a near-miss: then the CLI plumbs to the gate, near-miss
/// passes, spare-clean finds the binding site → `BindsCleanItem` (autoimmune, refused).
/// The CLI never pre-validates cleanliness; the GATE makes the call.
#[test]
fn propose_cli_plumbs_a_dirty_corpus_the_gate_catches_it() {
    // Dirty clean-root: (a) a near-miss `?`-sibling (passes the witness gate) PLUS
    // (b) an IDENTICAL swallow-error site the operator mislabeled clean (the draft
    // BINDS it → spare-clean catches it).
    let (tmp, cluster, clean) = staged_twins(
        "fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {\n\
        \x20   let mut out = Vec::new();\n\
        \x20   for e in std::fs::read_dir(root)? {\n\
        \x20       out.push(e?.path().display().to_string());\n\
        \x20   }\n\
        \x20   Ok(out)\n\
         }\n\
         fn walk_clean(root: &std::path::Path) -> Vec<String> {\n\
        \x20   let mut out = Vec::new();\n\
        \x20   if let Ok(entries) = std::fs::read_dir(root) {\n\
        \x20       for e in entries.flatten() {\n\
        \x20           out.push(e.path().display().to_string());\n\
        \x20       }\n\
        \x20   }\n\
        \x20   out\n\
         }\n",
    );
    let (code, stdout, _stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "the gate's autoimmune refusal is an honest outcome (exit 0); stdout={stdout}"
    );
    assert!(
        stdout.to_lowercase().contains("binds") || stdout.to_lowercase().contains("autoimmune"),
        "a contaminated corpus must be caught by the GATE's spare-clean (BindsCleanItem / \
         autoimmune), NOT pre-validated by the CLI — the cleanliness check lives in the \
         gate (the right organ); stdout={stdout}"
    );
    drop(tmp);
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

// ───────────────────────────────────────────────────────────────────────────
// Class 6 — STREAM A1 observability: --list-clusters (dry-run preview) +
// --exit-code (CI-routable outcome categories). The verb SHOWS its work.
// These are the teeth for the observability cheap-wins: the render-boundary
// reasoning (the by_shape grouping) is surfaced, and CI can distinguish outcomes
// WITHOUT the human-facing default ever grading a first-class outcome a "failure".
// ───────────────────────────────────────────────────────────────────────────

/// `--list-clusters` previews the `by_shape` grouping and STOPS (the dry-run). On the
/// real ≥2 twins cluster it shows ONE chosen group with 2 sites; the gate never runs
/// (no clean corpus consulted). Exit 0 by default.
#[test]
fn list_clusters_previews_the_chosen_twins_group() {
    let (tmp, cluster, clean) = staged_twins("fn unrelated() {}\n");
    let (code, stdout, _stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--list-clusters",
    ]);
    assert_eq!(
        code, 0,
        "list-clusters is a preview, exit 0; stdout={stdout}"
    );
    assert!(
        stdout.contains("cluster landscape") && stdout.contains("dry-run"),
        "the preview must name itself a dry-run landscape; stdout={stdout}"
    );
    // The twins share a shape → exactly one ≥2 group, marked chosen (`<==`).
    assert!(
        stdout.contains("<==") && stdout.contains('2'),
        "the chosen ≥2 group must be marked with its site count; stdout={stdout}"
    );
    // A dry-run NEVER renders a promote/refuse verdict (the gate didn't run).
    assert!(
        !stdout.to_lowercase().contains("ratifiable suggestion")
            && !stdout.to_lowercase().contains("refused"),
        "the dry-run must not render a gate verdict; stdout={stdout}"
    );
    drop(tmp);
}

/// `--list-clusters` does NOT require `--clean-root` (a preview never consults the
/// clean corpus). Omitting it must succeed, not error — the Option made the corpus
/// optional ONLY on this path.
#[test]
fn list_clusters_does_not_require_clean_root() {
    let (tmp, cluster, _clean) = staged_twins("fn unrelated() {}\n");
    let (code, stdout, stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--list-clusters",
    ]);
    assert_eq!(
        code, 0,
        "list-clusters without --clean-root must succeed; stderr={stderr}"
    );
    assert!(
        stdout.contains("cluster landscape"),
        "the preview must render; stdout={stdout}"
    );
    drop(tmp);
}

/// Omitting `--clean-root` on the GATE path (no --list-clusters) is a usage error
/// (exit 2) — the operator must supply + label the clean corpus; antigen never
/// auto-derives it (ATK-047-4). The Option must not weaken the gate's requirement.
#[test]
fn missing_clean_root_on_the_gate_path_is_a_usage_error() {
    let (tmp, cluster, _clean) = staged_twins("fn unrelated() {}\n");
    let (code, _stdout, stderr) = propose(&["--cluster-root", cluster.to_str().unwrap()]);
    assert_eq!(
        code, 2,
        "the gate path requires --clean-root (usage error); stderr={stderr}"
    );
    assert!(
        stderr.contains("clean-root") && stderr.contains("required"),
        "the error must name the missing --clean-root requirement; stderr={stderr}"
    );
    drop(tmp);
}

/// `--exit-code` opts CI into a distinct code per outcome. Route-to-human (the
/// settled twins-vs-unrelated dogfood outcome) → code 10 WITH the flag, but 0
/// WITHOUT it (the human default never grades route-to-human a failure). The stdout
/// is identical either way — only the exit code differs.
#[test]
fn exit_code_distinguishes_route_to_human_only_when_opted_in() {
    let unrelated = "fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let (tmp, cluster, clean) = staged_twins(unrelated);
    let cluster_s = cluster.to_str().unwrap();
    let clean_s = clean.to_str().unwrap();

    // WITHOUT --exit-code: route-to-human renders, exit 0 (first-class, not a failure).
    let (code_default, stdout_default, _e1) =
        propose(&["--cluster-root", cluster_s, "--clean-root", clean_s]);
    assert_eq!(
        code_default, 0,
        "route-to-human is exit 0 by default; stdout={stdout_default}"
    );
    assert!(
        stdout_default.contains("routed to a human"),
        "the settled twins-vs-unrelated outcome is route-to-human; stdout={stdout_default}"
    );

    // WITH --exit-code: the SAME outcome → code 10 (route-to-human category).
    let (code_flag, stdout_flag, _e2) = propose(&[
        "--cluster-root",
        cluster_s,
        "--clean-root",
        clean_s,
        "--exit-code",
    ]);
    assert_eq!(
        code_flag, 10,
        "route-to-human is category code 10 with --exit-code; stdout={stdout_flag}"
    );
    // The render is identical — only the exit code is gated by the flag.
    assert_eq!(
        stdout_default, stdout_flag,
        "the --exit-code flag changes only the exit code, never the render"
    );
    drop(tmp);
}

/// `--exit-code` maps the no-cluster outcome to code 13 (distinct from route-to-human
/// 10), while the default stays 0. CI can tell "nothing to anti-unify" from "a
/// candidate needs a human" — the distinction the all-0 status quo erased.
#[test]
fn exit_code_distinguishes_no_cluster_from_route_to_human() {
    let (tmp, cluster, clean) = staged_dirs(); // empty cluster → no-cluster
    let cluster_s = cluster.to_str().unwrap();
    let clean_s = clean.to_str().unwrap();

    let (code_default, _o1, _e1) = propose(&["--cluster-root", cluster_s, "--clean-root", clean_s]);
    assert_eq!(code_default, 0, "no-cluster is exit 0 by default");

    let (code_flag, _o2, _e2) = propose(&[
        "--cluster-root",
        cluster_s,
        "--clean-root",
        clean_s,
        "--exit-code",
    ]);
    assert_eq!(
        code_flag, 13,
        "no-cluster is category code 13 with --exit-code (distinct from route-to-human 10)"
    );
    drop(tmp);
}

/// `--list-clusters --format json` emits a machine-readable landscape: the cluster
/// list with shape digests + site counts + the `chosen` flag, and `has_cluster`.
#[test]
fn list_clusters_json_is_machine_readable() {
    let (tmp, cluster, _clean) = staged_twins("fn unrelated() {}\n");
    let (code, stdout, _stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--list-clusters",
        "--format",
        "json",
    ]);
    assert_eq!(code, 0, "json list-clusters preview, exit 0");
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("list-clusters --format json emits valid JSON");
    assert_eq!(v["outcome"], "cluster-landscape");
    assert_eq!(v["has_cluster"], true, "the twins form a ≥2 cluster");
    let clusters = v["clusters"].as_array().expect("clusters is an array");
    assert!(
        clusters
            .iter()
            .any(|c| c["chosen"] == true && c["site_count"] == 2),
        "exactly one chosen 2-site cluster; got {clusters:?}"
    );
    drop(tmp);
}

// ───────────────────────────────────────────────────────────────────────────
// Class 7 — STREAM A2 observability: --explain (the GATE-G reasoning the render
// hides). Turns the gate from oracle into teacher. Pure additive output — it
// NEVER changes the verdict or the exit code (GATE-G holds).
// ───────────────────────────────────────────────────────────────────────────

/// The dirty corpus that reaches the autoimmune (`BindsCleanItem`) verdict: a near-miss
/// `?`-sibling (passes the witness gate) PLUS an IDENTICAL swallow-error site the
/// operator mislabeled clean (the draft BINDS it → spare-clean catches it). Mirrors
/// `propose_cli_plumbs_a_dirty_corpus_the_gate_catches_it`.
const DIRTY_CLEAN_CORPUS: &str = "fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {\n\
    \x20   let mut out = Vec::new();\n\
    \x20   for e in std::fs::read_dir(root)? {\n\
    \x20       out.push(e?.path().display().to_string());\n\
    \x20   }\n\
    \x20   Ok(out)\n\
     }\n\
     fn walk_clean(root: &std::path::Path) -> Vec<String> {\n\
    \x20   let mut out = Vec::new();\n\
    \x20   if let Ok(entries) = std::fs::read_dir(root) {\n\
    \x20       for e in entries.flatten() {\n\
    \x20           out.push(e.path().display().to_string());\n\
    \x20       }\n\
    \x20   }\n\
    \x20   out\n\
     }\n";

/// `--explain` on AUTOIMMUNE refusal names the BOUND clean twin — the known-good item
/// the draft would have flagged (the autoimmunity made concrete). Without --explain the
/// render names only the index; with it, the item identity.
#[test]
fn explain_on_autoimmune_names_the_bound_twin() {
    let (tmp, cluster, clean) = staged_twins(DIRTY_CLEAN_CORPUS);
    let (code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--explain",
    ]);
    assert_eq!(code, 0, "autoimmune refusal is an honest outcome (exit 0)");
    assert!(
        stdout.to_lowercase().contains("binds") || stdout.to_lowercase().contains("autoimmune"),
        "the verdict is still autoimmune; stdout={stdout}"
    );
    assert!(
        stdout.contains("--explain"),
        "the --explain block must render; stdout={stdout}"
    );
    // The bound twin is the mislabeled `walk_clean` fn the draft binds.
    assert!(
        stdout.contains("fn walk_clean"),
        "--explain must name the BOUND clean twin (fn walk_clean); stdout={stdout}"
    );
    drop(tmp);
}

/// `--explain` on ROUTE-TO-HUMAN renders WHY there is no near-miss (the path to a YES:
/// add a sibling one constraint from the defect). The reasoning IS the absence.
#[test]
fn explain_on_route_to_human_renders_why_no_near_miss() {
    // Unrelated clean code → no near-miss → route-to-human.
    let (tmp, cluster, clean) = staged_twins(
        "fn add(a: i32, b: i32) -> i32 { a + b }\n\
         fn greet(n: &str) -> String { format!(\"hi {n}\") }\n",
    );
    let (code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--explain",
    ]);
    assert_eq!(code, 0, "route-to-human is first-class (exit 0)");
    assert!(
        stdout.contains("routed to a human"),
        "the verdict is still route-to-human; stdout={stdout}"
    );
    assert!(
        stdout.contains("--explain") && stdout.to_lowercase().contains("near-miss"),
        "--explain must render the why-no-near-miss reasoning; stdout={stdout}"
    );
    drop(tmp);
}

/// **The load-bearing invariant: `--explain` is PURE OUTPUT.** It changes neither the
/// verdict text's core nor the exit code — only APPENDS reasoning. Run the same
/// route-to-human case with and without `--explain`; the exit code is identical and the
/// non-explain output is a PREFIX of the explain output (nothing removed/changed, only
/// added).
#[test]
fn explain_never_changes_the_verdict_or_exit_code() {
    let unrelated = "fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let (tmp, cluster, clean) = staged_twins(unrelated);
    let c = cluster.to_str().unwrap();
    let cl = clean.to_str().unwrap();

    let (code_plain, stdout_plain, _e1) = propose(&["--cluster-root", c, "--clean-root", cl]);
    let (code_explain, stdout_explain, _e2) =
        propose(&["--cluster-root", c, "--clean-root", cl, "--explain"]);

    assert_eq!(
        code_plain, code_explain,
        "--explain must NOT change the exit code (pure output)"
    );
    // The plain render is fully contained in the explain render (only appended to).
    let plain_trimmed = stdout_plain.trim_end();
    assert!(
        stdout_explain.contains(plain_trimmed),
        "--explain must only APPEND reasoning, never alter the verdict render"
    );
    assert!(
        stdout_explain.len() > stdout_plain.len(),
        "--explain must add reasoning lines"
    );
    drop(tmp);
}

/// `--explain --format json` attaches an `explain` object carrying the near-miss /
/// bound-twin identities (machine-readable). On autoimmune the `bound_twin` is named.
#[test]
fn explain_json_attaches_the_reasoning_object() {
    let (tmp, cluster, clean) = staged_twins(DIRTY_CLEAN_CORPUS);
    let (code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--explain",
        "--format",
        "json",
    ]);
    assert_eq!(code, 0, "json autoimmune, exit 0");
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("--explain --format json emits valid JSON");
    assert_eq!(v["outcome"], "refused-autoimmune");
    assert!(
        v["explain"].is_object(),
        "the explain object must be attached; got {v}"
    );
    assert_eq!(
        v["explain"]["bound_twin"], "fn walk_clean",
        "the JSON explain must name the bound twin; got {v}"
    );
    drop(tmp);
}

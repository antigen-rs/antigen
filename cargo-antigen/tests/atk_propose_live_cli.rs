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
//! Three claim-kinds the keystone-goes-live surface must defend:
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
/// (The subcommand does not exist yet; the `#[ignore]`'d tests below specify its
/// contract — un-ignore as the pathmaker wires `AntigenSubcommand::Propose`.)
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

// ───────────────────────────────────────────────────────────────────────────
// Class 1 — route-to-human regression (CLI level)
// ───────────────────────────────────────────────────────────────────────────

/// ADR-047/051 — `propose_cli_renders_route_to_human_not_a_promote`. When the
/// cluster routes-to-human (`NotCorpusWitnessable` — the settled dogfood outcome),
/// the CLI RENDERS a needs-human-ratification suggestion, exit-code legible, and
/// does NOT report a promote. Pins the honest outcome at the CLI boundary so a
/// silent flip-to-false-promote is NOTICED here too.
#[test]
#[ignore = "born-red Island-3: cargo antigen propose is greenfield; un-ignore when \
            run_propose + the route-to-human render land"]
fn propose_cli_renders_route_to_human_not_a_promote() {
    // SPEC (un-ignore + stage a real cluster when the surface lands):
    //   let (code, stdout, _) = propose(&["--cluster-root", cluster, "--clean-root", clean]);
    //   // The render names the route-to-human outcome (a ratifiable specimen), NOT
    //   // "promoted" — the NotCorpusWitnessable verdict surfaces, not a bare nothing.
    //   assert!(stdout.contains("route") || stdout.contains("ratif"),
    //       "a route-to-human cluster must render as needs-ratification, not a promote");
    //   assert!(!stdout.contains("promoted to a named"),
    //       "the CLI must NOT report a promote for a route-to-human cluster");
    let _ = propose(&[]);
    unimplemented!("Island-3 cargo antigen propose route-to-human render not yet built");
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
#[test]
#[ignore = "born-red Island-3: ProposeArgs greenfield; un-ignore when --clean-root \
            (operator-supplied corpus) lands"]
fn propose_requires_an_operator_clean_corpus() {
    // SPEC (un-ignore when ProposeArgs lands):
    //   // No clean-corpus source → a clear usage error (exit 2), NEVER an
    //   // auto-labeled "the rest of the tree is clean" run:
    //   let (code, _, stderr) = propose(&["--cluster-root", some_cluster]);
    //   assert_eq!(code, 2, "propose without an operator clean corpus must be a usage error");
    //   assert!(stderr.contains("clean") && stderr.contains("required"),
    //       "the error must name the missing operator-supplied clean corpus (ATK-047-4: \
    //        antigen must not auto-label unmarked code as clean)");
    let _ = propose(&[]);
    unimplemented!("Island-3 ProposeArgs --clean-root not yet built");
}

/// CLEAN-CORPUS-SOURCE — `propose_does_not_auto_label_unmarked_as_clean`. Even with
/// a `--cluster-root`, antigen must not treat the *rest of the scanned tree* as the
/// clean corpus by default — the operator labels clean, antigen does not. (The
/// affirmative dual of the required-corpus test: it is not enough to error on a
/// missing flag; the surface must have NO "auto-clean" code path at all.)
#[test]
#[ignore = "born-red Island-3: clean-corpus source greenfield; un-ignore when the \
            operator-supplied-only contract lands"]
fn propose_does_not_auto_label_unmarked_as_clean() {
    // SPEC (un-ignore when the surface lands):
    //   // grep/inspect: run_propose's clean corpus comes SOLELY from --clean-root,
    //   // never from "scan_workspace minus the marked sites". A code-level assertion
    //   // (the CODE-TRUE audit) + a behavioral one: a cluster whose ONLY clean
    //   // sibling is OUTSIDE --clean-root must route-to-human (no near-miss in the
    //   // supplied corpus), NOT promote against an auto-derived corpus.
    let _ = propose(&[]);
    unimplemented!("Island-3 operator-supplied-only clean corpus not yet built");
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
#[test]
#[ignore = "born-red Island-3: cargo antigen propose greenfield; un-ignore when the \
            suggestion-render lands (ADR-044 observe-don't-declare)"]
fn propose_emits_a_suggestion_never_an_auto_presents() {
    // SPEC (un-ignore + stage a real cluster when the surface lands):
    //   // Snapshot the staged source tree, run propose (even on a PROMOTING cluster),
    //   // and assert NO source file changed — propose suggests, never declares.
    //   let before = hash_tree(&staged);
    //   let _ = propose(&["--cluster-root", cluster, "--clean-root", clean]);
    //   let after = hash_tree(&staged);
    //   assert_eq!(before, after,
    //       "propose MUST NOT auto-write a #[presents]/#[antigen] — it emits a \
    //        ratifiable suggestion (observe-don't-declare, ADR-044); the human ratifies");
    //   // And the rendered suggestion is typed on PromotedDraft (the only assertable
    //   // generalization, ADR-048) — surfaced as a suggestion, not asserted.
    let _ = propose(&[]);
    unimplemented!("Island-3 propose suggestion-render (observe-don't-declare) not yet built");
}

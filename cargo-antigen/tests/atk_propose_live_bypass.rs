//! ATK — `cargo antigen propose` BYPASS-ATTEMPTS (Island 3, keystone-goes-live),
//! BORN-RED. The ADVERSARIAL counterpart to `atk_propose_live_cli.rs`.
//!
//! `atk_propose_live_cli.rs` holds the CONTRACT classes (the positive controls — what
//! "done" looks like: route-to-human renders, operator corpus required, suggestion
//! emitted not declared). THIS file holds the **bypass-attempts those contracts are
//! the positive controls FOR** — the specific forge/launder/institutionalize moves
//! that must be **structurally impossible**, not merely "not done by the happy path."
//! The seam (captain's ruling, 2026-06-12): the test-architect owns CONTRACT; the
//! adversarial owns ATTACK. A contract says "propose leaves the tree byte-unchanged";
//! an attack says "there is NO code path — no flag, no env var, no config — that makes
//! propose write a `#[presents]`."
//!
//! Greenfield (`command grep` 2026-06-12: no `Propose`/`run_propose`/`ProposeArgs`/
//! `--clean-root` in `cargo-antigen/src/main.rs`). Each test is `#[ignore]`'d; its body
//! is the ATTACK CONTRACT (the 051-seal doc-spec pattern). Un-ignore + wire as the
//! pathmaker lands `AntigenSubcommand::Propose`.
//!
//! Standing smell (the build-wave adversary's): **attack the BOUNDARY, not the type.**
//! The `PromotedDraft` capability-token holds across five surfaces; the propose-CLI
//! adds a SIXTH — the render/emit boundary — where a forged or auto-derived suggestion
//! could leak past the gate the token guards. These four attacks probe that boundary.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo antigen propose <args>` → (exit-code, stdout, stderr). The subcommand
/// does not exist yet; the `#[ignore]`'d attacks below specify the bypasses it must
/// refuse — un-ignore as the pathmaker wires `run_propose`.
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
// ATTACK 1 — AUTO-ASSERT: is there ANY way to make propose WRITE a #[presents]?
//   (the bypass `propose_emits_a_suggestion_never_an_auto_presents` is the +control for)
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `no_flag_or_env_makes_propose_auto_write_a_presents`. The contract test
/// asserts the happy-path `propose` leaves the tree byte-unchanged. The ATTACK is
/// stronger: enumerate every plausible "apply it for me" affordance — a `--apply` /
/// `--write` / `--accept` flag, an `ANTIGEN_AUTO_APPLY` env var, a config key — and
/// assert NONE of them causes a `#[presents]`/`#[antigen]` to be written. Auto-applying
/// a machine suggestion IS the observe-don't-declare violation (ADR-044): the machine
/// must never cross the syntactic→semantic line. If any such affordance exists, that is
/// the real residual — a CLI path to a declared class the human never ratified.
#[test]
#[ignore = "born-red Island-3 ATTACK: cargo antigen propose greenfield; un-ignore when \
            run_propose lands — then enumerate every apply-affordance and assert none writes a mark"]
fn no_flag_or_env_makes_propose_auto_write_a_presents() {
    // SPEC (un-ignore + stage a PROMOTING cluster when the surface lands):
    //   let before = hash_tree(&staged);
    //   // Try every plausible auto-apply affordance — each MUST be either rejected
    //   // (no such flag) or a no-op on the source tree:
    //   for attack in [
    //       &["--cluster-root", c, "--clean-root", k, "--apply"][..],
    //       &["--cluster-root", c, "--clean-root", k, "--write"][..],
    //       &["--cluster-root", c, "--clean-root", k, "--accept"][..],
    //   ] {
    //       let _ = propose(attack);                       // with ANTIGEN_AUTO_APPLY=1 set too
    //       assert_eq!(hash_tree(&staged), before,
    //           "NO affordance may make propose write a #[presents]/#[antigen] — the \
    //            machine suggests, the human ratifies (ADR-044 observe-don't-declare)");
    //   }
    //   // CODE-TRUE half: grep run_propose — there is NO call that writes a marker
    //   // attribute to a source file (only a render to stdout / a ratification specimen).
    let _ = propose(&[]);
    unimplemented!("Island-3 propose auto-apply affordances not yet built");
}

// ───────────────────────────────────────────────────────────────────────────
// ATTACK 2 — LAUNDER: forge a suggestion the gate did NOT mint.
//   (probes the render/emit boundary — the sixth capability-token surface)
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `propose_render_surfaces_only_gate_minted_suggestions`. The render must emit
/// a suggestion ONLY for a draft the gate actually minted (`Ok(PromotedDraft)` or a
/// `NotCorpusWitnessable` route-to-human specimen). The ATTACK: is there a code path
/// where the CLI renders a "suggestion" for a draft that did NOT pass `promote_if_safe`
/// — e.g. it renders the raw `anti_unify` output on a parse/IO error, or a fallback
/// "here's the draft anyway" on the `Err` arm that isn't route-to-human? Such a path
/// launders an un-gated `Fingerprint` into the suggestion stream (the ADR-048 bypass at
/// the render boundary — the sixth surface). The residual-probe: *is there ANY rendered
/// suggestion whose provenance is NOT a gate verdict?* If none, the boundary holds.
#[test]
#[ignore = "born-red Island-3 ATTACK: propose render greenfield; un-ignore when the \
            render lands — assert every rendered suggestion carries a gate-verdict provenance"]
fn propose_render_surfaces_only_gate_minted_suggestions() {
    // SPEC (un-ignore when the render lands):
    //   // (a) a cluster that FAILS the gate (e.g. autoimmune BindsCleanItem, or a
    //   //     degenerate bare-structural draft C refuses) renders NO suggestion — it
    //   //     renders the legible refusal reason, never the raw draft as a suggestion.
    //   let (_, stdout, _) = propose(&["--cluster-root", autoimmune_cluster, "--clean-root", k]);
    //   assert!(!stdout.contains("suggest") || stdout.contains("refused"),
    //       "a gate-REFUSED cluster must not surface a suggestion — no un-gated draft \
    //        in the suggestion stream (ADR-048 at the render boundary)");
    //   // (b) CODE-TRUE: run_propose's render takes ONLY the propose() Result; there is
    //   //     no path that renders an anti_unify Fingerprint that bypassed promote_if_safe.
    let _ = propose(&[]);
    unimplemented!("Island-3 propose render-provenance not yet built");
}

// ───────────────────────────────────────────────────────────────────────────
// ATTACK 3 — INSTITUTIONALIZE: auto-label unmarked = clean (ATK-047-4 forbidden).
//   (the bypass `propose_requires_an_operator_clean_corpus` is the +control for)
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `propose_has_no_auto_clean_corpus_path_anywhere`. The contract test asserts a
/// missing `--clean-root` is a usage error. The ATTACK is the stronger structural
/// claim: there is NO code path — no `--all`, no `--auto-clean`, no "scan the tree and
/// treat unmarked items as the clean corpus" default, no env var — that DERIVES a clean
/// corpus antigen did not have an operator label. Auto-labeling unmarked=clean
/// institutionalizes the ATK-047-4 mislabeled-clean residual (the gate trusting its own
/// label) — and aristotle proved it computability-FORBIDDEN (a machine cannot decide
/// "this unmarked code is clean"). The probe: enumerate every corpus-source affordance;
/// each must require an explicit operator label.
#[test]
#[ignore = "born-red Island-3 ATTACK: clean-corpus source greenfield; un-ignore when \
            ProposeArgs lands — enumerate corpus-source affordances, assert all operator-explicit"]
fn propose_has_no_auto_clean_corpus_path_anywhere() {
    // SPEC (un-ignore when the surface lands):
    //   // Every plausible "figure out the clean corpus for me" affordance must be
    //   // ABSENT or require an explicit operator source — never auto-derive:
    //   for attack in [
    //       &["--cluster-root", c, "--all"][..],            // "use the whole tree"
    //       &["--cluster-root", c, "--auto-clean"][..],     // "infer the clean set"
    //       &["--cluster-root", c][..],                     // "default to unmarked = clean"
    //   ] {
    //       let (code, _, stderr) = propose(attack);
    //       assert_ne!(code, 0,
    //           "NO affordance may auto-derive the clean corpus — antigen must not label \
    //            unmarked code clean (ATK-047-4, computability-forbidden); got a success");
    //       // and if the flag is unknown, that's also fine (no such affordance exists).
    //   }
    //   // CODE-TRUE: run_propose's clean corpus is SOLELY parsed from --clean-root;
    //   // grep finds no "scan_workspace minus marked" or "assume_clean" path.
    let _ = propose(&[]);
    unimplemented!("Island-3 auto-clean-corpus absence not yet built");
}

// ───────────────────────────────────────────────────────────────────────────
// ATTACK 4 — narrow/persist RE-MINT stress (queued for ADR-051).
//   The CLI-level twin of the 051 library folds (narrow_output_is_producer_normalized
//   + tampered_nested_autoimmune_draft_is_caught_by_re_mint, atk_051_ratification_seal).
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `propose_ratify_narrow_re_mint_is_producer_independent_and_catches_forgery`.
/// When `cargo antigen ratify --narrow` (ADR-051) reconstructs a fingerprint and re-mints
/// it through the gate, the re-mint MUST be producer-independent (Fix B recursive
/// canonical-form) AND catch a NESTED-wrapped autoimmune forgery. At the CLI a ratifier
/// hand-edits a persisted suggestion (the on-disk `PersistedSpecimen`) into a nested
/// autoimmune draft; the re-mint on the next run MUST refuse it, not render it as a
/// re-validated suggestion. This is the CLI surface of the shape-fragility seam — the
/// place a user-editable fingerprint re-PARSES to arbitrary nesting and reaches the gate.
#[test]
#[ignore = "born-red Island-3/051 ATTACK: cargo antigen ratify greenfield; un-ignore \
            when narrow/persist re-mint + the recursive canonical-form both land"]
fn propose_ratify_narrow_re_mint_is_producer_independent_and_catches_forgery() {
    // SPEC (un-ignore when ratify --narrow + re-mint land; rides ADR-047 Amd2):
    //   // (a) a narrow that reconstructs via the all_of(..) surface re-gates to the
    //   //     SAME verdict as the flat form (recursive canonical-form, Fix B Hole-II).
    //   // (b) a hand-edited persisted specimen mutated to a NESTED autoimmune draft is
    //   //     CAUGHT by re-mint (Err / refused render), never re-validated as a suggestion.
    //   let tampered_specimen = edit_persisted(&pending, wrap_in_redundant_all_of(autoimmune));
    //   let (code, stdout, _) = ratify(&["--accept", &tampered_specimen.id]);
    //   assert_ne!(code, 0, "a nested autoimmune forgery must NOT survive re-mint at the CLI");
    //   assert!(!stdout.contains("accepted"), "the forged suggestion must not be accepted");
    let _ = propose(&[]);
    unimplemented!("Island-3/051 ratify narrow/persist re-mint not yet built");
}

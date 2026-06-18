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
//! Three of these four attacks are now WIRED against the live CLI (`run_propose` /
//! `ProposeArgs` / `--clean-root` landed `cd46dab`; the render landed in the same
//! commit, `render_propose_outcome`). ATTACKS 1-3 are un-ignored and real; the 4th
//! (`cargo antigen ratify` narrow/persist re-mint, ADR-051) targets a surface that does
//! NOT exist yet and stays `#[ignore]`'d — born-red on its future surface.
//!
//! Standing smell (the build-wave adversary's): **attack the BOUNDARY, not the type.**
//! The `PromotedDraft` capability-token holds across five surfaces; the propose-CLI
//! adds a SIXTH — the render/emit boundary — where a forged or auto-derived suggestion
//! could leak past the gate the token guards. These four attacks probe that boundary.
//!
//! Each wired attack has TWO halves, deliberately. The BEHAVIORAL half runs the real
//! binary against a staged fixture and asserts the bypass is refused (the tree is
//! byte-unchanged / the affordance is rejected / no un-gated suggestion surfaces) — it
//! catches a bypass that exists at runtime. The CODE-TRUE half reads
//! `cargo-antigen/src/main.rs` and asserts the bypass code-path is STRUCTURALLY ABSENT
//! (no `fs::write` in the propose region / `render_fingerprint` fires only on a
//! gate-minted token / the clean corpus is solely from `--clean-root`) — it catches a
//! future "convenience" edit that adds the path back. The behavioral half proves "not
//! done today"; the CODE-TRUE half proves "no code path that COULD do it" — the stronger
//! structural claim the attack contract demands.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo antigen propose <args>` → (exit-code, stdout, stderr).
fn propose(args: &[&str]) -> (i32, String, String) {
    propose_with_env(args, &[])
}

/// Run `cargo antigen propose <args>` with extra environment variables set →
/// (exit-code, stdout, stderr). Used to probe env-var "auto-apply" affordances.
fn propose_with_env(args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut cmd = Command::new(bin());
    cmd.arg("antigen").arg("propose").args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// ── Shared fixtures (raw-text `#[dread]` — the scan reads marks SYNTACTICALLY, so no
//    compiled crate is needed; mirrors `atk_propose_live_cli.rs`'s now-green pattern). ──

/// Two `#[dread]`-marked fn twins with IDENTICAL bodies (same local idents) so their
/// shape digests match → the scan clusters them; the swallow-the-read-error body
/// carries a real behavioral signal so the anti-unified draft survives the C-side
/// non-degeneracy guard (it is not a bare-structural over-binder). Parsed-as-text.
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

/// A near-miss-only clean corpus: a `?`-propagating sibling that is one discriminating
/// constraint from the swallow-error draft (witnessable) and is NOT bound by the draft
/// (spares clean) → the gate PROMOTES. This is the staging that yields an `Ok(token)`
/// render — the path where a forged/un-gated suggestion would have to leak if it could.
const NEAR_MISS_CLEAN: &str = "fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {\n\
    \x20   let mut out = Vec::new();\n\
    \x20   for e in std::fs::read_dir(root)? {\n\
    \x20       out.push(e?.path().display().to_string());\n\
    \x20   }\n\
    \x20   Ok(out)\n\
     }\n";

/// An AUTOIMMUNE-forcing clean corpus: the near-miss `?`-sibling PLUS an identical
/// swallow-error site mislabeled clean (the draft BINDS it → `BindsCleanItem` →
/// the gate REFUSES). The path where a gate-REFUSED draft must surface NO suggestion.
const AUTOIMMUNE_CLEAN: &str = "fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {\n\
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

/// Stage `cluster_src` + `clean_src` as a `cluster/`-and-`clean/` pair under a tempdir.
/// Returns `(tempdir, cluster_root, clean_root)`.
fn staged(cluster_src: &str, clean_src: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
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

/// A `(path → bytes)` snapshot of every `.rs` file under `root` — the substrate for the
/// byte-unchanged assertion (observe-don't-declare: a `propose` run writes NO mark).
fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
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

/// The body of `run_propose` plus its render helpers — the propose REGION of
/// `cargo-antigen/src/main.rs` the CODE-TRUE halves grep for a structurally-absent
/// bypass path. Spans `fn run_propose(` through the end of `render_propose_json`'s
/// successor boundary (the next free `fn ` after the render fns), so a write/forge
/// path added ANYWHERE in the propose pipeline is in scope.
fn propose_region() -> String {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("read cargo-antigen main.rs");
    let start = src
        .find("fn run_propose(")
        .expect("run_propose exists in main.rs");
    // The propose pipeline ends at `fn run_new(` (the next subcommand handler after the
    // propose render helpers). If that anchor ever moves, the slice still includes the
    // whole render surface; widen here if a new propose helper lands after run_new.
    let region = &src[start..];
    let end = region.find("\nfn run_new(").unwrap_or(region.len());
    let region = region[..end].to_string();
    // SELF-VALIDATE the slice so a moved/renamed anchor goes RED instead of letting the
    // CODE-TRUE probes pass VACUOUSLY on a truncated region (the toothless-generator
    // trap: a probe that greps an empty/partial region "passes" by finding nothing). The
    // region MUST contain the whole propose pipeline — the gate call AND both render
    // surfaces (the render fns are exactly where a launder/auto-write bypass would hide).
    for landmark in [
        "fn run_propose(",
        "propose::propose(",
        "fn render_propose_outcome(",
        "fn render_propose_json(",
        "fn render_fingerprint(",
    ] {
        assert!(
            region.contains(landmark),
            "propose_region() lost `{landmark}` — the slice anchors (fn run_propose .. \
             fn run_new) drifted, so the CODE-TRUE probes would grep a TRUNCATED region \
             and pass vacuously. Re-anchor the slice before trusting any CODE-TRUE half."
        );
    }
    region
}

// ───────────────────────────────────────────────────────────────────────────
// ATTACK 1 — AUTO-ASSERT: is there ANY way to make propose WRITE a #[presents]?
//   (the bypass `propose_emits_a_suggestion_never_an_auto_presents` is the +control for)
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `no_flag_or_env_makes_propose_auto_write_a_presents`. The contract test
/// asserts the happy-path `propose` leaves the tree byte-unchanged. The ATTACK is
/// stronger: enumerate every plausible "apply it for me" affordance — a `--apply` /
/// `--write` / `--accept` flag, an `ANTIGEN_AUTO_APPLY` env var — and assert NONE of
/// them causes a `#[presents]`/`#[antigen]` to be written, even on a PROMOTING cluster
/// (the only path where there is a draft worth applying). Auto-applying a machine
/// suggestion IS the observe-don't-declare violation (ADR-044): the machine must never
/// cross the syntactic→semantic line. If any such affordance exists, that is the real
/// residual — a CLI path to a declared class the human never ratified.
#[test]
fn no_flag_or_env_makes_propose_auto_write_a_presents() {
    // Stage the PROMOTING cluster: this is the ONLY outcome that mints a draft a
    // bypass could "apply". If even a promote leaves the tree byte-unchanged under
    // every apply-ish affordance, no weaker outcome can write a mark either.
    let (tmp, cluster, clean) = staged(TWINS, NEAR_MISS_CLEAN);
    let c = cluster.to_str().unwrap();
    let k = clean.to_str().unwrap();
    let before = snapshot_tree(tmp.path());

    // BEHAVIORAL — every plausible auto-apply affordance, each also with the env var
    // set. An unknown flag is clap-REJECTED (the affordance does not exist); a flag
    // that DID exist must be a no-op on the source tree. Either way: NO mark written.
    for extra in [
        &["--apply"][..],
        &["--write"][..],
        &["--accept"][..],
        &["--apply", "--yes"][..],
        &[][..], // the bare promote run, with the env var set, must also not apply
    ] {
        let mut argv = vec!["--cluster-root", c, "--clean-root", k];
        argv.extend_from_slice(extra);
        let _ = propose_with_env(
            &argv,
            &[("ANTIGEN_AUTO_APPLY", "1"), ("ANTIGEN_APPLY", "1")],
        );
        assert_eq!(
            snapshot_tree(tmp.path()),
            before,
            "NO affordance (flag or env var) may make propose write a \
             #[presents]/#[antigen] — the machine suggests, the human ratifies \
             (ADR-044 observe-don't-declare); affordance tried: {extra:?}"
        );
    }

    // CODE-TRUE — the propose region has NO source-file-writing path at all. `propose`
    // only READS (scan / parse / collect) and RENDERS (println / eprintln). A future
    // "apply" edit would have to add a write call here; assert their absence so it
    // trips. (We intern the literals so this test does not flag ITSELF.)
    let region = propose_region();
    for write_call in [
        concat!("fs", "::write"),
        "write_all",
        "OpenOptions",
        concat!("File", "::create"),
        "create_new",
    ] {
        assert!(
            !region.contains(write_call),
            "the propose region must not write to any file — observe-don't-declare is \
             STRUCTURAL (run_propose only reads + renders). Found a `{write_call}` call: \
             a source-writing path is a CLI route to a declared class the human never \
             ratified (ADR-044)"
        );
    }
    drop(tmp);
}

// ───────────────────────────────────────────────────────────────────────────
// ATTACK 2 — LAUNDER: forge a suggestion the gate did NOT mint.
//   (probes the render/emit boundary — the sixth capability-token surface)
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `propose_render_surfaces_only_gate_minted_suggestions`. The render must emit
/// a suggestion ONLY for a draft the gate actually minted (`Ok(PromotedDraft)`). The
/// ATTACK: is there a code path where the CLI renders a "suggestion" for a draft that
/// did NOT pass the gate — e.g. it renders a raw `anti_unify` `Fingerprint` on the
/// `Err` arm, or a fallback "here's the draft anyway" on a refusal? Such a path
/// launders an un-gated `Fingerprint` into the suggestion stream (the ADR-048 bypass at
/// the render boundary — the sixth surface). The residual-probe: *is there ANY rendered
/// fingerprint whose provenance is NOT a gate-minted token?* If none, the boundary holds.
#[test]
fn propose_render_surfaces_only_gate_minted_suggestions() {
    // BEHAVIORAL (a) — a gate-REFUSED (autoimmune) cluster renders the legible refusal,
    // NEVER a candidate suggestion. The draft the gate refused must not leak into the
    // suggestion stream as if it had been minted.
    let (tmp, cluster, clean) = staged(TWINS, AUTOIMMUNE_CLEAN);
    let (code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "the autoimmune refusal is an honest outcome; stdout={stdout}"
    );
    let lower = stdout.to_lowercase();
    assert!(
        lower.contains("refused") && (lower.contains("autoimmune") || lower.contains("binds")),
        "a gate-refused cluster must render the autoimmune REFUSAL reason; stdout={stdout}"
    );
    // The refusal must NOT dress the un-gated draft up as a ratifiable candidate: the
    // "candidate ... ratifiable suggestion" header fires ONLY on `Ok(token)`.
    assert!(
        !(lower.contains("candidate") && lower.contains("ratifiable")),
        "a gate-REFUSED cluster must NOT surface a ratifiable candidate suggestion — \
         no un-gated draft in the suggestion stream (ADR-048 at the render boundary); \
         stdout={stdout}"
    );
    drop(tmp);

    // BEHAVIORAL (b) — a route-to-human cluster renders the routed specimen, also NOT a
    // promoted/ratifiable-candidate suggestion (the gate did not mint a token).
    let (tmp2, cluster2, clean2) = staged(
        TWINS,
        "fn add(a: i32, b: i32) -> i32 { a + b }\nfn greet(n: &str) -> String { format!(\"hi {n}\") }\n",
    );
    let (_c2, stdout2, _e2) = propose(&[
        "--cluster-root",
        cluster2.to_str().unwrap(),
        "--clean-root",
        clean2.to_str().unwrap(),
    ]);
    let lower2 = stdout2.to_lowercase();
    assert!(
        lower2.contains("route") && lower2.contains("ratif"),
        "the route-to-human render names the human-ratification route; stdout={stdout2}"
    );
    assert!(
        !lower2.contains("candidate failure-class fingerprint (ratifiable suggestion)"),
        "route-to-human is NOT a minted-token suggestion — it must not render the \
         `Ok(token)` ratifiable-candidate header; stdout={stdout2}"
    );
    drop(tmp2);

    // CODE-TRUE — every `render_fingerprint(` call in the propose region takes a
    // `token.fingerprint()` (a gate-minted `PromotedDraft`), NEVER a bare `Fingerprint`
    // from `anti_unify`/the draft. If a future edit renders a fingerprint outside an
    // `Ok(token)` arm, this trips. The probe: count render_fingerprint( call-sites and
    // assert each is immediately fed `token.fingerprint()`.
    let region = propose_region();
    let mut call_sites = 0usize;
    for (idx, _) in region.match_indices("render_fingerprint(") {
        // Skip the DEFINITION (`fn render_fingerprint(`) — only count CALL sites.
        let preceding = &region[..idx];
        if preceding.ends_with("fn ") {
            continue;
        }
        call_sites += 1;
        // The argument must be the gate-minted token's fingerprint — the text
        // immediately after the open paren must BEGIN with `token.fingerprint(`, never
        // `draft`, `anti_unify`, or a bare `&fp` not sourced from the token. (We check
        // the prefix, not a paren-split — `token.fingerprint()` carries its own parens.)
        let after = region[idx + "render_fingerprint(".len()..].trim_start();
        assert!(
            after.starts_with("token.fingerprint(") || after.starts_with("token .fingerprint("),
            "render_fingerprint must be fed a GATE-MINTED token's fingerprint \
             (`token.fingerprint()`), never an un-gated draft — a rendered fingerprint \
             whose provenance is not a gate verdict launders the ADR-048 token at the \
             render boundary. Found arg starting: `{}`",
            &after[..after.len().min(40)]
        );
    }
    assert!(
        call_sites >= 2,
        "expected the human + json `Ok(token)` arms to each render the minted \
         fingerprint (≥2 call sites); found {call_sites} — did the render surface move? \
         (a moved render must keep the token-only-provenance invariant under attack)"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// ATTACK 3 — INSTITUTIONALIZE: auto-label unmarked = clean (ATK-047-4 forbidden).
//   (the bypass `propose_requires_an_operator_clean_corpus` is the +control for)
// ───────────────────────────────────────────────────────────────────────────

/// ATK — `propose_has_no_auto_clean_corpus_path_anywhere`. The contract test asserts a
/// missing `--clean-root` is a usage error. The ATTACK is the stronger structural
/// claim: there is NO code path — no `--all`, no `--auto-clean`, no "scan the tree and
/// treat unmarked items as the clean corpus" default — that DERIVES a clean corpus
/// antigen did not have an operator label. Auto-labeling unmarked=clean institutionalizes
/// the ATK-047-4 mislabeled-clean residual (the gate trusting its own label) — and
/// aristotle proved it computability-FORBIDDEN (a machine cannot decide "this unmarked
/// code is clean"). The probe: enumerate every corpus-source affordance; each must
/// require an explicit operator label.
#[test]
fn propose_has_no_auto_clean_corpus_path_anywhere() {
    // A real cluster root so the ONLY thing missing/forbidden is the auto-clean source.
    let (tmp, cluster, _clean) = staged(TWINS, NEAR_MISS_CLEAN);
    let c = cluster.to_str().unwrap();

    // BEHAVIORAL — every "figure out the clean corpus for me" affordance must be
    // refused: an UNKNOWN flag is a clap error (the affordance does not exist), and the
    // bare run with NO --clean-root is the REQUIRED-arg usage error. There is no
    // success path that auto-derives a corpus.
    for attack in [
        &["--cluster-root", c, "--all"][..], // "use the whole tree"
        &["--cluster-root", c, "--auto-clean"][..], // "infer the clean set"
        &["--cluster-root", c, "--clean-root", c, "--auto-clean"][..], // even WITH a root, no infer flag
        &["--cluster-root", c][..], // "default to unmarked = clean" — must be a usage error
    ] {
        let (code, _stdout, stderr) = propose(attack);
        assert_ne!(
            code, 0,
            "NO affordance may auto-derive the clean corpus — antigen must not label \
             unmarked code clean (ATK-047-4, computability-forbidden); attack {attack:?} \
             succeeded (exit 0). stderr={stderr}"
        );
    }
    // And specifically: the bare-no-clean-root case names the missing REQUIRED arg
    // (it is a usage error, not a silent auto-labeled run).
    let (code, _o, stderr) = propose(&["--cluster-root", c]);
    assert_eq!(
        code, 2,
        "a missing clean corpus source is a usage error (exit 2), not a silent \
         auto-derivation; stderr={stderr}"
    );
    assert!(
        stderr.contains("clean-root"),
        "the usage error must name the missing --clean-root; stderr={stderr}"
    );
    drop(tmp);

    // CODE-TRUE — the clean corpus is built SOLELY from `--clean-root`, never
    // auto-derived. `--clean-root` is `Option<PathBuf>` (optional ONLY for the
    // `--list-clusters` dry-run, which never gates), so the gate path binds the local
    // `let Some(clean_root) = args.clean_root` and feeds `clean_root` to
    // `collect_clean_corpus`. The invariant pinned here is the SOURCE of the corpus
    // (the operator-explicit `--clean-root`), not the binding's spelling. A future edit
    // that scans the cluster root / the whole workspace and subtracts marks would add a
    // forbidden path here; assert its absence so it trips. This is the
    // affordance-enumeration twin of the CLI file's `propose_does_not_auto_label_*`
    // (that one pins the positive source; this attack pins the NEGATIVE — no derivation).
    let region = propose_region();
    assert!(
        region.contains("collect_clean_corpus(clean_root)"),
        "the clean corpus must come from --clean-root (collect_clean_corpus(clean_root)) \
         — the operator-explicit source"
    );
    assert!(
        region.contains("let Some(clean_root) = args.clean_root"),
        "the gate's clean_root must be BOUND from --clean-root (args.clean_root), with an \
         `else` that refuses the gate when it is absent — not derived from anything else"
    );
    // No auto-derivation: not from the cluster root, not from a workspace scan, not from
    // an "assume clean / unmarked = clean" inference. (NOTE: `scan_workspace(&args
    // .cluster_root` IS now present in run_propose — the --list-clusters dry-run scans the
    // cluster root to PREVIEW the by_shape grouping — but that path never builds a clean
    // corpus, so it is not an auto-clean derivation. The forbidden shapes are the ones
    // that would feed a DERIVED corpus into the gate: collect_clean_corpus from the
    // cluster root, or an explicit assume/unmarked/auto-clean inference.)
    for forbidden in [
        "collect_clean_corpus(&args.cluster_root)",
        "collect_clean_corpus(cluster_root)",
        "assume_clean",
        "unmarked_is_clean",
        "auto_clean",
    ] {
        assert!(
            !region.contains(forbidden),
            "run_propose must NOT auto-derive the clean corpus — found `{forbidden}`. \
             Antigen never labels unmarked code clean (ATK-047-4); the operator supplies \
             + labels the corpus via --clean-root, antigen adds nothing"
        );
    }
}

/// ATK — `only_the_list_clusters_affordance_exits_zero_without_a_corpus`. The
/// `--clean-root → Option` change (so the `--list-clusters` dry-run can preview WITHOUT
/// a corpus) opened a precise bypass risk: a corpus-less success path on the GATE side.
/// The invariant this pins (the adversarial's ruling): **only the explicit
/// list/preview affordance may exit 0 without `--clean-root`; the bare run and every
/// gate path must still refuse.** Made the Option weaken nothing on the gate.
///
/// Three probes on the SAME real ≥2 cluster (so the only variable is the corpus
/// affordance):
/// - `--cluster-root c` (bare, no corpus, no preview) → exit 2, names `clean-root`
///   (the gate is REQUESTED; the missing required corpus is a usage error).
/// - `--cluster-root c --list-clusters` (preview, no corpus) → exit 0 (the ONLY
///   sanctioned corpus-less success: it never gates).
/// - `--cluster-root c --clean-root c` (gate, WITH corpus) → reaches the gate (NOT a
///   usage error): exit != 2 (the corpus IS supplied; whatever GATE-G then rules is a
///   real verdict, not a missing-arg error).
#[test]
fn only_the_list_clusters_affordance_exits_zero_without_a_corpus() {
    let (tmp, cluster, clean) = staged(TWINS, NEAR_MISS_CLEAN);
    let c = cluster.to_str().unwrap();
    let cl = clean.to_str().unwrap();

    // (a) Bare gate request, no corpus → usage error (exit 2), names --clean-root.
    let (bare_code, _o, bare_err) = propose(&["--cluster-root", c]);
    assert_eq!(
        bare_code, 2,
        "the bare run (gate requested, no --clean-root) must be a usage error, NOT a \
         corpus-less success; stderr={bare_err}"
    );
    assert!(
        bare_err.contains("clean-root"),
        "the usage error must name the missing --clean-root; stderr={bare_err}"
    );

    // (b) The preview affordance is the ONLY sanctioned corpus-less exit 0 — it never
    //     gates, so no corpus is needed.
    let (preview_code, preview_out, _e) = propose(&["--cluster-root", c, "--list-clusters"]);
    assert_eq!(
        preview_code, 0,
        "--list-clusters is the sanctioned corpus-less preview (exit 0); stdout={preview_out}"
    );

    // (c) WITH a corpus the gate is actually reached — the absent-corpus usage error
    //     (exit 2) must NOT fire when --clean-root IS supplied. (Whatever GATE-G rules
    //     from there is a real verdict, asserted by the gate-outcome tests; here we only
    //     pin that supplying the corpus clears the missing-arg refusal.)
    let (gate_code, _o, _e) = propose(&["--cluster-root", c, "--clean-root", cl]);
    assert_ne!(
        gate_code, 2,
        "supplying --clean-root must clear the missing-corpus usage error (the gate is \
         reached); a 2 here would mean the corpus was ignored"
    );
    drop(tmp);
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

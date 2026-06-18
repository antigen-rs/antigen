//! ATK — ACT, the effector SUGGEST-floor (`the-spared-sibling-is-the-patch`).
//!
//! v0.6 grows the organism's effector arm to its honest floor: when the gate
//! PROMOTES a draft (it found a spared clean sibling — a near-miss that witnessed the
//! generalization), `cargo antigen propose --suggest` shows that sibling as a
//! SUGGESTED FIX — "your defect sites carry <the discriminating signal>; this spared
//! clean sibling `<name>` shows the safe SHAPE without it." A retrieve-then-adapt
//! suggestion at the lowest trust rung (`Applicability::MaybeIncorrect` — suggest,
//! NEVER auto-apply), routed to a human ratifier.
//!
//! # The honest-scope (the load-bearing safety line — adversarial STRESS #5)
//!
//! "the spared sibling IS the patch" discharges only ONE of a fix's two primitives:
//! (1) a TARGET shape (what should this become?) — YES, the spared twin shows it; but
//! NOT (2) intent-preservation (it's the SAME code, made safe). The nearest spared
//! sibling is a DIFFERENT function with DIFFERENT intent that merely shares the
//! fingerprint-keyed conjuncts — a `Drop` that `.unwrap()`s the flush vs one that
//! `.ok()`s it are NOT the same code (one may REQUIRE observing the error; the other
//! silently drops it). So the SUGGEST-floor MUST NOT claim correctness — it shows the
//! safe SHAPE and says "ratify by hand; intent-preservation is YOUR call." That honest
//! scope is exactly why the floor is human-ratify and needs no matured draft (it acts
//! on GATE-G's spared-clean output, not on a maturation result).
//!
//! # The no-twin case (sub-clause-F honest)
//!
//! When there is NO spared sibling (route-to-human — the corpus holds no near-miss),
//! ACT must NOT fabricate a twin. It falls back to the GENUS fix-direction (ADR-038:
//! recover-info / tighten-the-guard / reorder-the-effect) and SAYS there is no
//! in-corpus example. Suggesting a "fix" with no evidence is the autoimmune failure of
//! the effector — the floor refuses it.
//!
//! These tests are BORN-RED (tests-first — they DEFINE done before `--suggest` lands)
//! and exercise the live binary against raw-text `#[dread]` fixtures.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `cargo antigen propose <args>` → (exit-code, stdout, stderr).
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

/// The two `#[dread]`-marked swallow-the-read-error twins (identical shape → they
/// cluster; a real behavioral signal so the draft is not bare-structural). Raw-text;
/// the scan reads `#[dread]` syntactically, so no compiled crate is needed.
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

/// A NEAR-MISS clean sibling: the `?`-propagating safe shape — one discriminating
/// constraint from the swallow-error draft (it does NOT swallow), so it is spared AND
/// witnesses the generalization → PROMOTE. This is the spared twin ACT suggests.
const NEAR_MISS_CLEAN: &str = "fn scan_dir_safe(root: &std::path::Path) -> std::io::Result<Vec<String>> {\n\
    \x20   let mut out = Vec::new();\n\
    \x20   for e in std::fs::read_dir(root)? {\n\
    \x20       out.push(e?.path().display().to_string());\n\
    \x20   }\n\
    \x20   Ok(out)\n\
     }\n";

/// Unrelated clean code: shares no behavioral conjunct with the swallow-error draft,
/// so NOTHING is a near-miss → route-to-human (the no-twin case).
const UNRELATED_CLEAN: &str = "fn add(a: i32, b: i32) -> i32 { a + b }\n\
                               fn greet(n: &str) -> String { format!(\"hi {n}\") }\n";

/// Stage `cluster_src` + `clean_src` as a `cluster/`-and-`clean/` pair.
fn staged(cluster_src: &str, clean_src: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cluster_dir = tmp.path().join("cluster/src");
    let clean_dir = tmp.path().join("clean/src");
    std::fs::create_dir_all(&cluster_dir).unwrap();
    std::fs::create_dir_all(&clean_dir).unwrap();
    std::fs::write(cluster_dir.join("lib.rs"), cluster_src).unwrap();
    std::fs::write(clean_dir.join("lib.rs"), clean_src).unwrap();
    // Compute the returned roots BEFORE moving `tmp` into the tuple (borrow-after-move).
    let cluster_root = tmp.path().join("cluster");
    let clean_root = tmp.path().join("clean");
    (tmp, cluster_root, clean_root)
}

/// Recursively hash the file tree (path → bytes) so a test can assert the source is
/// BYTE-UNCHANGED after a `--suggest` run (the floor suggests, never auto-applies).
fn walk(dir: &std::path::Path, map: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, map);
        } else {
            map.insert(p.clone(), std::fs::read(&p).unwrap());
        }
    }
}

fn snapshot_tree(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut map = std::collections::BTreeMap::new();
    walk(root, &mut map);
    map
}

// ───────────────────────────────────────────────────────────────────────────
// CLASS 1 — the SUGGEST renders the spared-twin fix (on a PROMOTE).
// ───────────────────────────────────────────────────────────────────────────

/// `propose --suggest` on a PROMOTE (a near-miss spared twin exists) renders the twin
/// as a suggested fix: it NAMES the spared sibling, shows it as the safe SHAPE, and
/// labels the suggestion at the lowest trust rung (suggest / maybe-incorrect), never
/// auto-apply.
#[test]
fn suggest_renders_the_spared_twin_as_a_fix_on_a_promote() {
    let (tmp, cluster, clean) = staged(TWINS, NEAR_MISS_CLEAN);
    let (code, stdout, stderr) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--suggest",
    ]);
    assert_eq!(code, 0, "a promote with --suggest exits 0; stderr={stderr}");
    let lower = stdout.to_lowercase();
    // It names the spared twin (the retrieval target) by its identity.
    assert!(
        stdout.contains("scan_dir_safe"),
        "--suggest must name the spared clean sibling (the safe shape to adapt toward); \
         stdout={stdout}"
    );
    // It frames it as a SUGGESTED fix at the lowest trust rung — suggest, not auto-apply.
    assert!(
        lower.contains("suggest"),
        "--suggest must frame the twin as a SUGGESTED fix; stdout={stdout}"
    );
    assert!(
        lower.contains("maybe-incorrect")
            || lower.contains("maybeincorrect")
            || lower.contains("ratify"),
        "the suggestion must be at the human-ratify trust floor (MaybeIncorrect / \
         ratify-by-hand), never auto-apply; stdout={stdout}"
    );
    drop(tmp);
}

/// The honest-scope caveat is PRESENT and load-bearing: the suggestion must say the
/// spared twin is a DIFFERENT function sharing the fingerprint — it shows the safe
/// SHAPE, NOT a verified fix; intent-preservation is the human's call. Without this
/// the floor over-claims correctness (the adversarial's STRESS #5).
#[test]
fn suggest_carries_the_intent_preservation_caveat() {
    let (tmp, cluster, clean) = staged(TWINS, NEAR_MISS_CLEAN);
    let (_code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--suggest",
    ]);
    let lower = stdout.to_lowercase();
    assert!(
        lower.contains("intent"),
        "the suggestion MUST name the intent-preservation caveat — the twin shows the \
         SHAPE, not a verified fix; the human ratifies intent. stdout={stdout}"
    );
    assert!(
        lower.contains("shape") || lower.contains("different") || lower.contains("not a verified"),
        "the caveat must clarify the twin is a DIFFERENT function / shows the safe SHAPE, \
         never claim it is a drop-in correct fix; stdout={stdout}"
    );
    drop(tmp);
}

// ───────────────────────────────────────────────────────────────────────────
// CLASS 2 — the no-twin case falls back to the genus, never fabricates.
// ───────────────────────────────────────────────────────────────────────────

/// `--suggest` on a ROUTE-TO-HUMAN (no spared sibling in the corpus) must NOT
/// fabricate a twin. It falls back to the GENUS fix-direction (ADR-038) and SAYS there
/// is no in-corpus example. A suggestion with no evidence is the effector's autoimmune
/// failure — refused (sub-clause-F honest).
#[test]
fn suggest_with_no_twin_falls_back_to_the_genus_not_a_fabrication() {
    let (tmp, cluster, clean) = staged(TWINS, UNRELATED_CLEAN);
    let (code, stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--suggest",
    ]);
    assert_eq!(code, 0, "route-to-human with --suggest is honest (exit 0)");
    let lower = stdout.to_lowercase();
    // It must say there is NO in-corpus twin (never invent one).
    assert!(
        lower.contains("no")
            && (lower.contains("twin") || lower.contains("sibling") || lower.contains("example")),
        "with no spared sibling, --suggest must SAY there is no in-corpus twin (never \
         fabricate one); stdout={stdout}"
    );
    // It must NOT name a clean sibling as the fix (there is none to name).
    assert!(
        !stdout.contains("scan_dir_safe"),
        "the route-to-human corpus has NO near-miss twin — --suggest must not name one; \
         stdout={stdout}"
    );
    drop(tmp);
}

// ───────────────────────────────────────────────────────────────────────────
// CLASS 3 — observe-don't-declare holds: --suggest never writes the source.
// ───────────────────────────────────────────────────────────────────────────

/// A `--suggest` run leaves the source tree BYTE-UNCHANGED — it is a SUGGEST-floor, not
/// an auto-fix. The retrieve-then-adapt suggestion is rendered to stdout; the human
/// applies it (or not). Pins the observe-don't-declare invariant at the effector.
#[test]
fn suggest_never_writes_the_source_tree() {
    let (tmp, cluster, clean) = staged(TWINS, NEAR_MISS_CLEAN);
    let before = snapshot_tree(tmp.path());
    let (_code, _stdout, _e) = propose(&[
        "--cluster-root",
        cluster.to_str().unwrap(),
        "--clean-root",
        clean.to_str().unwrap(),
        "--suggest",
    ]);
    let after = snapshot_tree(tmp.path());
    assert_eq!(
        before, after,
        "--suggest is a SUGGEST-floor: it renders a fix suggestion but MUST NOT write the \
         source tree (observe-don't-declare; the human applies)"
    );
    drop(tmp);
}

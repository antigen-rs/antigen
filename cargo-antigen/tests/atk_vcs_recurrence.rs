//! `cargo antigen vcs recurrence` — the recurrent-emergence git-mining observer
//! (`infra/recurrence-automation`, v03-vision-buildout).
//!
//! The passive→active loop for the recurrent family: today the three recurrent
//! stdlib antigens (`MsrvCreepAfterMajorVersionBump`,
//! `GitignorePatternDriftOverReleases`, `LockfileChurnFromUnpinnedTooling`) are
//! passive — the adopter manually marks `#[recurrence_anchor]`/`#[itch]`. This
//! command mines git history to DETECT how many times each pattern fired and
//! surfaces the counts; the VERDICT (anchor it?) stays the adopter's call (the
//! structural seam — mining detects, the adopter recognizes).
//!
//! These tests build a hermetic git repo in a tempdir with KNOWN commits and
//! assert the mined counts, then assert the honest-degradation path (a
//! non-repo directory reports unobservable, not zero — exit 0 either way, the
//! observer must never block an audit).

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

/// Run `git <args>` in `dir`, panicking on failure. Used to build the fixture
/// history; the test environment always has git (it built the workspace).
fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH for these tests");
    assert!(
        status.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&status.stderr)
    );
}

/// Commit `file` (relative to `dir`) with `content` and message `msg`.
fn commit_file(dir: &Path, file: &str, content: &str, msg: &str) {
    let path = dir.join(file);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    git(dir, &["add", file]);
    git(dir, &["commit", "-q", "-m", msg]);
}

/// Initialize a hermetic git repo (local identity, no signing) in `dir`.
fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "test@antigen.local"]);
    git(dir, &["config", "user.name", "antigen-test"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
}

/// Run `cargo-antigen antigen vcs recurrence <args>` in `dir`; return
/// `(exit_code, stdout, stderr)`.
fn recurrence(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("vcs")
        .arg("recurrence")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run cargo-antigen");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn recurrence_counts_each_pattern_from_known_history() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    init_repo(dir.path());

    // Seed a known history:
    //   - 2 commits that CHANGE a rust-version line in Cargo.toml (MSRV creep)
    //   - 3 commits touching .gitignore (gitignore drift)
    //   - 1 commit touching Cargo.lock (lockfile churn)
    // Plus an unrelated commit that must NOT be counted by any detector.
    commit_file(
        dir.path(),
        "Cargo.toml",
        "[package]\nname=\"x\"\nrust-version = \"1.70\"\n",
        "seed Cargo.toml with rust-version",
    );
    commit_file(
        dir.path(),
        "Cargo.toml",
        "[package]\nname=\"x\"\nrust-version = \"1.74\"\n",
        "bump rust-version (msrv creep #1)",
    );
    commit_file(dir.path(), ".gitignore", "/target\n", "gitignore #1");
    commit_file(dir.path(), ".gitignore", "/target\n*.tmp\n", "gitignore #2");
    commit_file(
        dir.path(),
        ".gitignore",
        "/target\n*.tmp\n*.log\n",
        "gitignore #3",
    );
    commit_file(dir.path(), "Cargo.lock", "# lockfile v1\n", "lockfile #1");
    commit_file(dir.path(), "README.md", "unrelated\n", "unrelated change");

    let (code, stdout, _stderr) = recurrence(dir.path(), &["--format", "json", "--depth", "50"]);
    assert_eq!(
        code, 0,
        "the observer must exit 0 (it never blocks an audit)"
    );

    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["observable"], serde_json::json!(true));
    let obs = v["observations"].as_array().expect("observations array");

    let count_for = |antigen: &str| -> i64 {
        obs.iter()
            .find(|o| o["antigen"] == serde_json::json!(antigen))
            .and_then(|o| o["recurrence_count"].as_i64())
            .unwrap_or_else(|| panic!("no observation for {antigen}: {obs:?}"))
    };

    // The seed Cargo.toml commit ADDS a rust-version line; the bump CHANGES it.
    // `git log -Grust-version` matches a diff that adds OR removes a matching
    // line, so BOTH count — 2 commits touched a rust-version line.
    assert_eq!(
        count_for("MsrvCreepAfterMajorVersionBump"),
        2,
        "two commits changed a rust-version line (seed + bump): {obs:?}"
    );
    assert_eq!(
        count_for("GitignorePatternDriftOverReleases"),
        3,
        "three commits touched .gitignore: {obs:?}"
    );
    assert_eq!(
        count_for("LockfileChurnFromUnpinnedTooling"),
        1,
        "one commit touched Cargo.lock: {obs:?}"
    );
}

#[test]
fn recurrence_human_output_names_the_anchor_macro() {
    // The human output's value is teaching the adopter the recognition move:
    // it must name the macro to use, framed as observation-not-verdict.
    let dir = tempfile::TempDir::new().expect("tempdir");
    init_repo(dir.path());
    commit_file(dir.path(), ".gitignore", "/target\n", "gitignore");

    let (code, stdout, _stderr) = recurrence(dir.path(), &[]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("#[recurrence_anchor"),
        "human output must point at the anchor macro the adopter would use: {stdout}"
    );
    assert!(
        stdout.contains("OBSERVATIONS, not verdicts"),
        "human output must frame counts as observations, preserving the recognition seam: {stdout}"
    );
}

#[test]
fn recurrence_degrades_honestly_outside_a_git_repo() {
    // A non-repo directory: git log fails. The observer must report UNOBSERVABLE
    // (not zero — absence of a mine is not evidence of absence) and still exit 0
    // (it must never block an audit just because git is unavailable).
    let dir = tempfile::TempDir::new().expect("tempdir");
    // Deliberately NOT a git repo.

    let (code, stdout, _stderr) = recurrence(dir.path(), &["--format", "json"]);
    assert_eq!(
        code, 0,
        "honest degradation is not an error verdict — exit 0 so the audit is not blocked"
    );
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("valid JSON even when degraded");
    assert_eq!(
        v["observable"],
        serde_json::json!(false),
        "outside a repo the mine is unobservable, never a misleading zero: {stdout}"
    );
}

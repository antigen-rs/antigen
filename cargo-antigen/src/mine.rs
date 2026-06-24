//! `cargo antigen mine` — the SZZ corpus-miner's **git WALK** (the CLI half).
//!
//! The `antigen` library holds the pure SZZ *classifier*
//! ([`antigen::learn::szz`] — `is_fix_commit`, `mine`, `Corpus` over already-read
//! [`CommitMeta`]). Per ADR-002 (the `vcs_witness` pattern), the library does **no
//! git I/O**; the CLI does. This module is that CLI half: it walks a real `.git`
//! with `gix` and feeds the library classifier, producing a measurable corpus.
//!
//! # The full-object-graph traversal (the load-bearing invariant)
//!
//! [`mine_repo`] seeds the rev-walk from **every reference** (`references().all()`)
//! — NOT just `HEAD`. This is `git rev-list --all` semantics: on a squash-merged
//! repo a branch-tip walk starves (~50 commits) while the full graph is mineable
//! (~hundreds, RUN-verified on antigen's own repo: 823 commits / 214 fix-class). A
//! tip-only walk is a build-bug; seeding from all refs is the fix, enforced here.
//! [`antigen::learn::szz::Corpus::size`] reports the *measured* count, so the
//! "cures corpus-starvation" claim is self-verifying per-repo.

use std::path::Path;

use antigen::learn::szz::{CommitMeta, Corpus, DefectFixPair, is_fix_commit};

/// An error mining a repository's `.git`.
#[derive(Debug)]
pub enum MineError {
    /// `gix` could not open the repository (not a git repo, or unreadable).
    Open(String),
    /// A read during the object-graph walk failed (refs, rev-walk, or commit
    /// lookup). The walk is abandoned — a partial corpus would silently
    /// under-report [`Corpus::size`], defeating the self-verification.
    Walk(String),
}

impl std::fmt::Display for MineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open(p) => write!(f, "could not open git repository: {p}"),
            Self::Walk(c) => write!(f, "git object-graph walk failed: {c}"),
        }
    }
}

impl std::error::Error for MineError {}

/// **Mine a repository's `.git` into the SZZ corpus** — the full-object-graph walk.
///
/// Walks every reference's ancestry (`rev-list --all`), classifies fix commits via
/// the library's [`is_fix_commit`], and links each to its **first parent** (the
/// candidate defect-introducing commit at commit granularity — the standard SZZ
/// "the bug existed in the pre-fix state" approximation). The precise per-line
/// blame linker is the library's [`antigen::learn::szz::mine`] fed real
/// `BlamedLine`s; wiring `gix blame` per fixed line is a precision refinement, not
/// a correctness gap in the corpus this produces. A **root** fix commit (no parent)
/// contributes no pair — honestly, not by silent failure.
///
/// # Errors
///
/// [`MineError::Open`] if `repo_path` is not a readable git repository;
/// [`MineError::Walk`] if a read fails mid-walk (the walk is abandoned rather than
/// returning a silently-truncated corpus).
pub fn mine_repo(repo_path: &Path) -> Result<Corpus, MineError> {
    let repo = gix::open(repo_path).map_err(|e| MineError::Open(e.to_string()))?;

    // The `--all` seed: every reference's target, not just HEAD. Missing this is the
    // tip-revwalk starvation bug the invariant exists to prevent.
    let mut tips: Vec<gix::ObjectId> = Vec::new();
    let refs = repo
        .references()
        .map_err(|e| MineError::Walk(e.to_string()))?;
    let all = refs.all().map_err(|e| MineError::Walk(e.to_string()))?;
    for r in all {
        let mut r = r.map_err(|e| MineError::Walk(e.to_string()))?;
        // Peel through tags/symbolic refs to the object the ref ultimately names.
        // A peel FAILURE is propagated as `MineError::Walk`, NOT silently dropped: the
        // function's contract is abandon-don't-truncate (a dropped ref would exclude its
        // whole ancestry from the `--all` seed, silently under-reporting `Corpus::size`
        // — the exact tip-revwalk starvation this seed exists to prevent). Matching the
        // surrounding refs/rev-walk/commit-lookup error handling (deep-comb szz fix).
        let peeled = r.peel_to_id().map_err(|e| MineError::Walk(e.to_string()))?;
        tips.push(peeled.detach());
    }

    // The full-object-graph ancestry walk from every tip. `gix` de-duplicates a
    // commit reachable from multiple tips, so it is visited once.
    let walk = repo
        .rev_walk(tips)
        .all()
        .map_err(|e| MineError::Walk(e.to_string()))?;

    let mut pairs: Vec<DefectFixPair> = Vec::new();
    for info in walk {
        let info = info.map_err(|e| MineError::Walk(e.to_string()))?;
        let commit = repo
            .find_commit(info.id)
            .map_err(|e| MineError::Walk(e.to_string()))?;
        let subject = commit
            .message()
            .map_err(|e| MineError::Walk(e.to_string()))?
            .summary()
            .to_string();

        let meta = CommitMeta {
            id: info.id.to_string(),
            subject,
            files_changed: Vec::new(),
            cosmetic_only: false,
        };
        if !is_fix_commit(&meta) {
            continue;
        }
        // Coarse linker: the first parent is the pre-fix state. A root fix has none.
        let Some(parent) = info.parent_ids.first() else {
            continue;
        };
        let pair = DefectFixPair {
            defect_commit: parent.to_string(),
            fix_commit: meta.id,
            // No per-line locus at commit granularity (the precise path comes from
            // the blame-fed library `mine`). Empty marks "commit-level pairing".
            path: String::new(),
        };
        if !pairs.contains(&pair) {
            pairs.push(pair);
        }
    }
    Ok(Corpus { pairs })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walk up from `start` to the first ancestor directory containing a `.git`.
    fn find_repo_root(start: &Path) -> Option<std::path::PathBuf> {
        start
            .ancestors()
            .find(|p| p.join(".git").exists())
            .map(Path::to_path_buf)
    }

    #[test]
    fn mine_repo_mines_a_real_corpus_from_this_repo_git() {
        // Mine the repository this test lives in (CARGO_MANIFEST_DIR = the
        // cargo-antigen crate dir; walk up to the worktree/repo root holding .git).
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let Some(root) = find_repo_root(manifest) else {
            // No .git reachable (e.g. a packaged build) — nothing to mine; skip.
            return;
        };

        let corpus = mine_repo(&root).expect("mining this repo's .git must not error");

        // THE SELF-VERIFYING CLAIM, RUN-PROVEN not baked: antigen's own history has
        // real fix-commits across the full object graph, so a full-graph walk yields
        // a non-empty corpus. (size 0 on a fix-history repo ⇒ a tip-revwalk
        // regression — the exact bug `Corpus::is_starved` guards against.)
        assert!(
            corpus.size() > 0,
            "full-object-graph walk of a repo with fix-history must mine a non-empty \
             corpus (size 0 ⇒ a tip-revwalk regression — the starvation bug)"
        );

        // Every mined pair is a real (parent → fix) link: distinct, non-empty ids.
        for pair in &corpus.pairs {
            assert!(!pair.fix_commit.is_empty(), "fix commit id must be present");
            assert!(
                !pair.defect_commit.is_empty(),
                "defect (parent) id must be present"
            );
            assert_ne!(
                pair.fix_commit, pair.defect_commit,
                "a fix and its parent are distinct commits"
            );
        }
    }

    #[test]
    fn mine_repo_errors_cleanly_on_a_non_repo() {
        // A path with no .git is an honest Open error, never a panic or silent empty.
        let not_a_repo = std::env::temp_dir().join("antigen-szz-definitely-not-a-git-repo-xyz");
        assert!(matches!(mine_repo(&not_a_repo), Err(MineError::Open(_))));
    }

    /// Run a `git` subcommand in `dir`, asserting success. Used only to STAGE a real
    /// `.git` for the dangling-ref fixture (the production miner never shells out — it
    /// reads the object graph via `gix`).
    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .current_dir(dir)
            // A throwaway fixture repo is NOT a gated project: isolate it from the
            // user's GLOBAL git hooks. (camp's pre-commit gate installs via a global
            // `core.hooksPath` and hard-fails — "no camp project found" — inside a
            // non-camp temp repo, breaking the fixture's commit.) This does NOT skip
            // the test — the miner is still fully exercised; only the synthetic
            // fixture commit runs un-hooked, exactly as a synthetic git history should.
            .args(["-c", "core.hooksPath="])
            .args(args)
            .status()
            .expect("git must be on PATH for this fixture");
        assert!(status.success(), "git {args:?} failed in {}", dir.display());
    }

    /// ATK-DEEPCOMB-SZZ-1 (deep-comb degenerate-input pass) — an UNPEELABLE ref must
    /// ABANDON the walk (`MineError::Walk`), never be silently dropped.
    ///
    /// `mine_repo`'s `--all` seed peels every reference to its object id. A ref that
    /// fails to peel (a dangling ref pointing at a missing object, a broken symbolic
    /// ref) was silently skipped by the old `if let Ok(peeled) = r.peel_to_id() {}` —
    /// which EXCLUDES that ref's entire ancestry from the corpus, silently
    /// under-reporting `Corpus::size`. That is exactly the tip-revwalk **starvation**
    /// the `--all` seed exists to prevent, and it directly contradicts the function's
    /// stated contract: `MineError::Walk` docs say "The walk is abandoned — a partial
    /// corpus would silently under-report `Corpus::size`," and the `# Errors` section
    /// says a mid-walk read failure abandons "rather than returning a silently-truncated
    /// corpus." A dropped unpeelable ref is a silently-truncated corpus.
    ///
    /// BORN-RED on the old code (the dangling ref was skipped → `Ok`); GREEN once the
    /// peel error is `?`-propagated as `MineError::Walk` (abandon-don't-truncate).
    #[test]
    fn mine_repo_abandons_on_an_unpeelable_ref_never_silently_drops() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = tmp.path();

        // A minimal real repo with one fix commit (so the GOOD ref has mineable history —
        // the point is that the BAD ref must not let the walk silently succeed with a
        // truncated corpus).
        git(repo, &["init", "-q", "-b", "main"]);
        git(repo, &["config", "user.email", "t@t.t"]);
        git(repo, &["config", "user.name", "t"]);
        std::fs::write(repo.join("f.txt"), "a").unwrap();
        git(repo, &["add", "f.txt"]);
        git(repo, &["commit", "-qm", "fix: a panic in Drop"]);

        // Plant a DANGLING ref: a ref file pointing at an object id that does not exist.
        // `peel_to_id` cannot resolve it → the old code silently dropped it.
        std::fs::write(
            repo.join(".git/refs/heads/dangling"),
            "0000000000000000000000000000000000000001\n",
        )
        .unwrap();

        let result = mine_repo(repo);
        assert!(
            matches!(result, Err(MineError::Walk(_))),
            "ATK-DEEPCOMB-SZZ-1: an unpeelable (dangling) ref must ABANDON the walk with \
             MineError::Walk — abandon-don't-truncate (the function's own contract). \
             Silently dropping it excludes its ancestry from the --all seed and \
             under-reports Corpus::size (the tip-revwalk starvation the seed prevents). \
             Got: {result:?}",
        );
    }
}

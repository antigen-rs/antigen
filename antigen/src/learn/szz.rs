//! The **SZZ corpus-miner** (v0.6) — MATURE's input-corpus, mined from `.git`.
//!
//! antigen's own learned marks are *singletons* in shape-space (three `#[dread]`
//! marks, no cluster) — the maturation organ (MATURE) provably starves on N=1 input.
//! The cure is a real corpus of `(defect, fix)` pairs, and that corpus is **already
//! present** in every repository's git history: 20 years of Mining-Software-
//! Repositories research (the **SZZ** algorithm — Śliwerski/Zimmermann/Zeller, MSR
//! 2005, *"When Do Changes Induce Fixes?"*) is exactly the method for recovering it.
//! This module is the SZZ heuristic, as a *reader* over git substrate (ADR-006,
//! recognition-not-design — the corpus is recomputable from `.git`, not generated).
//!
//! # Where this sits in the MATURE pipeline (a THIRD substrate, not the life-record)
//!
//! ```text
//!   [SZZ (defect,fix) corpus]  ──▶  MATURE  ──▶  [affinity-trajectory]  +  [story/outcome]
//!     this module (INPUT)                          the life-record (OUTPUT side)
//!     recomputable from .git                       un-back-fillable, turn-zero
//! ```
//!
//! The SZZ corpus is MATURE's **raw material**, at the *opposite* end of the pipeline
//! from the life-record (the organism's self-history). It is **recomputable** from
//! `.git` (zero lock-in, zero schema-urgency) — so it is a distinct substrate, never
//! a record-kind of the life-record. (Conflating the two would couple a recomputable
//! input to an un-back-fillable output schema; they are kept apart by identity.)
//!
//! # Library purity (ADR-002) — the lib classifies, the CLI walks `.git`
//!
//! Per the same discipline as [`vcs_witness`](crate::vcs_witness): the `antigen`
//! library does **NOT** shell out to `git` or depend on a git crate. This module
//! takes **already-read commit substrate** — a commit's subject, its changed files,
//! its diff hunks, and (for the fix-linking step) blame results — as plain data
//! ([`CommitMeta`], [`BlamedLine`]). The `cargo antigen` CLI performs the actual git
//! walk and feeds the results here. This keeps the lib subprocess-free,
//! deterministically testable on synthetic histories, and dependency-light.
//!
//! # The full-object-graph traversal invariant (a CONTRACT on the CLI walk)
//!
//! The do-now "this cures corpus-starvation day-one" justification holds **only**
//! under a **full-object-graph** traversal (`git rev-list --all` — all refs,
//! reflog-inclusive), NOT a single-tip revwalk (`git log` / the `gix`/`git2` default
//! `Revwalk`-from-`HEAD`). RUN-verified on antigen's own repo (2026-06-17): a
//! branch-tip view sees ~50 commits / ~7 fix-class (**starvation persists**), while
//! the full object graph sees 823 commits / 214 fix-class (a **real** mineable
//! corpus, ~30× larger — the history was squash-merged, so the mineable commits live
//! off-tip). A tip-revwalk miner is a **build-bug**, not a corpus-absence.
//!
//! This module cannot enforce the traversal (it does not walk git) — but it does
//! **not bake the corpus size**: [`Corpus::size`] reports the *measured* count of
//! whatever the CLI fed it, so the cure-claim is **self-verifying per-repo** (a
//! starved corpus reports a small size, surfacing a tip-revwalk bug instead of
//! silently failing). The CLI adapter carries the `rev-list --all` contract.

use serde::{Deserialize, Serialize};

/// Already-read metadata for one commit — the CLI's git walk fills this; the lib
/// classifies it. No git handle, no subprocess: plain data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMeta {
    /// The commit's identity (a short or full SHA — opaque to the lib; used only as
    /// a key in `(defect, fix)` pairs).
    pub id: String,
    /// The commit's subject line (first line of the message) — the B-SZZ
    /// fix-classification keyword surface.
    pub subject: String,
    /// The paths the commit touched. Used by the AG-SZZ cosmetic filter (a commit
    /// touching only non-source paths is not a code fix) and as fix-locus context.
    pub files_changed: Vec<String>,
    /// `true` iff every hunk in the commit's diff is whitespace-/formatting-only
    /// (no semantic line changed). The CLI computes this from the diff; the lib uses
    /// it for the AG-SZZ noise filter (a cosmetic commit is never a real fix, even if
    /// its subject says "fix formatting"). Defaults to `false` (assume semantic) when
    /// the CLI cannot determine it — the conservative direction (we would rather
    /// admit a borderline commit than silently drop a real fix).
    #[serde(default)]
    pub cosmetic_only: bool,
}

/// One blamed line of a fix-commit's change — the SZZ fix-linking input.
///
/// For each line a fix-commit *modified*, the CLI runs `git blame` on the **pre-fix**
/// version and reports which commit last touched that line = the candidate
/// bug-introducing change. The lib pairs them; it does not run blame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlamedLine {
    /// The commit the pre-fix blame attributes this line to — the candidate
    /// bug-introducing commit.
    pub introducing_commit: String,
    /// The source path the line belongs to (fix-locus; carried for corpus context).
    pub path: String,
}

/// A mined `(defect-introducing, fix)` commit pair — one SZZ corpus entry.
///
/// The atom MATURE consumes: the defect commit located the failure-shape, the fix
/// commit carries the `(defect → clean)` diff (the effector's clean-twin corpus,
/// downstream). The lib produces the *linkage*; the structural-fingerprint shaping of
/// each pair is a later MATURE-side step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefectFixPair {
    /// The bug-introducing commit (from the pre-fix blame).
    pub defect_commit: String,
    /// The fix commit (B-SZZ-classified, AG-SZZ-filtered).
    pub fix_commit: String,
    /// The source path the pairing was established on (the blamed line's locus).
    pub path: String,
}

/// The mined SZZ corpus — the collection of `(defect, fix)` pairs + a **measured**
/// size self-report.
///
/// [`size`](Self::size) is the per-repo self-verification of the corpus-starvation
/// cure: it reports the count actually mined, never a baked-in number. A starved
/// corpus (a tip-revwalk bug) reports a small size, surfacing the build-bug instead
/// of silently failing the do-now justification.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Corpus {
    /// The mined `(defect, fix)` pairs (deduplicated, deterministic order).
    pub pairs: Vec<DefectFixPair>,
}

impl Corpus {
    /// The **measured** number of mined `(defect, fix)` pairs — the self-verifying
    /// corpus-size (never a baked number). The "cures starvation day-one" claim is
    /// true for *this* repo iff this is large; a tip-revwalk bug shows up here as a
    /// near-zero count.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.pairs.len()
    }

    /// `true` iff the corpus is too small to feed MATURE (below `min_pairs`).
    ///
    /// The honest day-one tripwire: a caller mining its own `.git` checks this to
    /// distinguish "this repo genuinely has little fix-history" from "I walked the
    /// wrong graph" (the tip-revwalk build-bug). The threshold is the caller's — a
    /// cluster needs ≥2 members to exhibit diversity, so `min_pairs` is realistically
    /// a handful, not one.
    #[must_use]
    pub const fn is_starved(&self, min_pairs: usize) -> bool {
        self.size() < min_pairs
    }
}

/// Fix-commit subject keywords (B-SZZ, the field-standard fix-classification surface).
/// Lower-cased substring match — the canonical SZZ keyword set (`fix`, `bug`, `patch`,
/// `revert`) plus the conventional-commit `fix(` / `fix:` prefixes a substring catches.
const FIX_KEYWORDS: [&str; 5] = ["fix", "bug", "patch", "revert", "hotfix"];

/// **B-SZZ fix-commit classification** (+ the AG-SZZ cosmetic filter).
///
/// `true` iff `meta` is a *real* fix commit: its subject carries a fix-class keyword
/// (or an issue reference like `#1234`) AND it is not a cosmetic-only change.
///
/// - **B-SZZ (the 2005 original):** a fix commit is identified by message keywords /
///   issue-tracker links. We match the canonical `FIX_KEYWORDS` (a module-private
///   set) as lower-cased substrings and an issue reference (`#` followed by a digit).
/// - **AG-SZZ (the noise refinement):** a whitespace-/formatting-only commit is
///   excluded even if its subject says "fix formatting" — a cosmetic change never
///   induced a defect, so it is not a corpus fix (`meta.cosmetic_only`).
///
/// Scope (honest): this is the *simplest field-standard* heuristic (B-SZZ + AG-SZZ).
/// The squash-merge-aware PR-SZZ recovery and the refactoring-aware RA-SZZ filter are
/// later refinements — they raise precision, they do not change this contract.
#[must_use]
pub fn is_fix_commit(meta: &CommitMeta) -> bool {
    if meta.cosmetic_only {
        return false;
    }
    let subject = meta.subject.to_lowercase();
    let has_keyword = FIX_KEYWORDS.iter().any(|kw| subject.contains(kw));
    has_keyword || references_issue(&meta.subject)
}

/// `true` iff `subject` contains an issue reference: a `#` immediately followed by an
/// ASCII digit (`#1234`, `fixes #42`). A bare `#` (a Markdown heading) does not count.
fn references_issue(subject: &str) -> bool {
    let bytes = subject.as_bytes();
    bytes
        .iter()
        .enumerate()
        .any(|(i, &b)| b == b'#' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
}

/// **Mine the SZZ corpus** from already-read substrate: classify fix commits, then
/// link each fix's blamed lines back to their bug-introducing commits.
///
/// The lib half of the SZZ pipeline (the CLI supplies `commits` from a
/// **full-object-graph** walk and `blame_of` from per-fix pre-fix blame):
/// 1. **classify** — keep the commits for which [`is_fix_commit`] holds (B-SZZ +
///    AG-SZZ);
/// 2. **link** — for each fix commit, look up its blamed lines via `blame_of` and
///    emit one [`DefectFixPair`] per `(introducing_commit, fix_commit, path)`,
///    skipping a line a fix blames on *itself* (not a prior defect);
/// 3. **dedup** — collapse identical pairs (a fix touching many lines of one defect
///    yields one pair), in a deterministic order.
///
/// `blame_of` is a closure the CLI provides: `fix_commit_id → its blamed lines`. A
/// fix with no recorded blame (an empty slice / absent key) contributes no pair —
/// honestly, not by silent failure (it simply linked nothing).
pub fn mine<'a, F>(commits: &'a [CommitMeta], blame_of: F) -> Corpus
where F: Fn(&'a str) -> &'a [BlamedLine] {
    let mut pairs: Vec<DefectFixPair> = Vec::new();
    for fix in commits.iter().filter(|c| is_fix_commit(c)) {
        for line in blame_of(&fix.id) {
            // A line a fix blames on itself is not a prior defect — skip it (a fix
            // that adds a new line has no bug-introducing predecessor for it).
            if line.introducing_commit == fix.id {
                continue;
            }
            let pair = DefectFixPair {
                defect_commit: line.introducing_commit.clone(),
                fix_commit: fix.id.clone(),
                path: line.path.clone(),
            };
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }
    Corpus { pairs }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(id: &str, subject: &str) -> CommitMeta {
        CommitMeta {
            id: id.to_string(),
            subject: subject.to_string(),
            files_changed: vec!["src/lib.rs".to_string()],
            cosmetic_only: false,
        }
    }

    fn blamed(introducing: &str, path: &str) -> BlamedLine {
        BlamedLine {
            introducing_commit: introducing.to_string(),
            path: path.to_string(),
        }
    }

    // --- B-SZZ classification ---

    #[test]
    fn fix_keywords_classify_fix_commits() {
        assert!(is_fix_commit(&commit("a", "fix: panic in Drop")));
        assert!(is_fix_commit(&commit("b", "Fix the off-by-one")));
        assert!(is_fix_commit(&commit("c", "bug: wrong index")));
        assert!(is_fix_commit(&commit("d", "revert the bad merge")));
        assert!(is_fix_commit(&commit("e", "hotfix for prod")));
        assert!(is_fix_commit(&commit("f", "patch the leak")));
    }

    #[test]
    fn non_fix_commits_are_not_classified() {
        assert!(!is_fix_commit(&commit("a", "feat: add the affinity type")));
        assert!(!is_fix_commit(&commit("b", "docs: clarify the README")));
        assert!(!is_fix_commit(&commit("c", "refactor module layout")));
    }

    #[test]
    fn issue_reference_classifies_a_fix() {
        assert!(is_fix_commit(&commit("a", "close #1234")));
        assert!(is_fix_commit(&commit("b", "resolves #42 cleanly")));
        // A bare `#` (markdown heading) is NOT an issue reference.
        assert!(!is_fix_commit(&commit("c", "# Heading, not an issue")));
    }

    // --- AG-SZZ cosmetic filter ---

    #[test]
    fn cosmetic_only_commit_is_filtered_even_if_subject_says_fix() {
        let mut c = commit("a", "fix formatting");
        c.cosmetic_only = true;
        // A whitespace-only change never induced a defect — not a corpus fix.
        assert!(!is_fix_commit(&c));
    }

    // --- the mine pipeline ---

    #[test]
    fn mines_defect_fix_pairs_from_classified_fixes_and_blame() {
        let commits = [
            commit("d1", "feat: introduce the guard"), // the defect (not a fix)
            commit("f1", "fix: guard panics on Drop"), // the fix
            commit("x", "docs: unrelated"),            // neither
        ];
        // f1 fixed two lines, both introduced by d1 — one defect → one (deduped) pair.
        let f1_blame = [blamed("d1", "src/lib.rs"), blamed("d1", "src/lib.rs")];
        let corpus = mine(&commits, |id| if id == "f1" { &f1_blame[..] } else { &[] });
        // The duplicate (d1, f1, src/lib.rs) pair collapses to ONE.
        assert_eq!(corpus.size(), 1);
        assert_eq!(
            corpus.pairs[0],
            DefectFixPair {
                defect_commit: "d1".to_string(),
                fix_commit: "f1".to_string(),
                path: "src/lib.rs".to_string(),
            }
        );
    }

    #[test]
    fn a_fix_blaming_itself_yields_no_pair() {
        let commits = [commit("f1", "fix: add a brand-new guard line")];
        // The fixed line was ADDED by f1 itself — no prior defect commit.
        let self_blame = [blamed("f1", "src/lib.rs")];
        let corpus = mine(
            &commits,
            |id| if id == "f1" { &self_blame[..] } else { &[] },
        );
        assert_eq!(corpus.size(), 0);
    }

    #[test]
    fn a_non_fix_commit_with_blame_contributes_nothing() {
        // Even with blame data, a commit that isn't a fix is not mined.
        let commits = [commit("feat1", "feat: shiny new module")];
        let blame = [blamed("d0", "src/lib.rs")];
        let corpus = mine(&commits, |id| if id == "feat1" { &blame[..] } else { &[] });
        assert_eq!(corpus.size(), 0);
    }

    // --- corpus self-verification (the starvation tripwire) ---

    #[test]
    fn size_is_the_measured_count_never_baked() {
        let corpus = Corpus {
            pairs: vec![
                DefectFixPair {
                    defect_commit: "d1".into(),
                    fix_commit: "f1".into(),
                    path: "a.rs".into(),
                },
                DefectFixPair {
                    defect_commit: "d2".into(),
                    fix_commit: "f2".into(),
                    path: "b.rs".into(),
                },
            ],
        };
        assert_eq!(corpus.size(), 2);
    }

    #[test]
    fn is_starved_is_the_tip_revwalk_tripwire() {
        let empty = Corpus::default();
        // A tip-revwalk on a squash-merged repo mines ~nothing → starved.
        assert!(empty.is_starved(5));
        let healthy = Corpus {
            pairs: (0..10)
                .map(|i| DefectFixPair {
                    defect_commit: format!("d{i}"),
                    fix_commit: format!("f{i}"),
                    path: "src/lib.rs".into(),
                })
                .collect(),
        };
        assert!(!healthy.is_starved(5));
    }

    #[test]
    fn corpus_serde_roundtrips() {
        let corpus = Corpus {
            pairs: vec![DefectFixPair {
                defect_commit: "d1".into(),
                fix_commit: "f1".into(),
                path: "src/lib.rs".into(),
            }],
        };
        let json = serde_json::to_string(&corpus).expect("serialize");
        let back: Corpus = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(corpus, back);
    }
}

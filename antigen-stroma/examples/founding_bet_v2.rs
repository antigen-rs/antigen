//! The founding-bet defect-concentration test, v2 — SYMBOL granularity, the real measurement.
//!
//! **What v1 (`founding_bet_test.rs`) found.** A FILE-granular join of structural signals against
//! SZZ-mined fix-history came back a clean NULL — but it was killed by a size confound (a bigger
//! file trivially accumulates more of everything: more symbols, more edges, more commits, more
//! fixes). The bet is stated at SYMBOL level, not file level; a file-level test cannot falsify or
//! confirm it.
//!
//! **The unlock.** `Finding`/defect sites carry `file:line`; `ingest_scip` already computes exact
//! body-span containment (a definition occurrence's `enclosing_range` is that definition's OWN full
//! span — settled by `probe_bodyspan`/`probe_enclosing`, definition semantics, not "the fn enclosing
//! a reference"). So a git diff hunk's touched `file:line` joins to its CONTAINING SYMBOL the same
//! way `ingest_scip` joins a call-site to its caller: the innermost span whose `[start, end]`
//! contains the line. `ingest_scip` restricts that containment to fn-shaped definitions (its own
//! purpose — naming a call's caller — only needs function bodies); this module generalizes the
//! IDENTICAL containment rule across every definition kind that carries an `enclosing_range`
//! (function AND struct/enum/enum-variant), because the structural signal under test
//! (`efferent::risky_to_change_findings`'s DREAD grade) fires ONLY on construct-target symbols —
//! which SCIP's `ScipKind::is_constructor` restricts to `Struct` / `Enum` / `EnumMember` — never on
//! a plain function. A fn-only containment table would make every DREAD symbol structurally
//! unattributable by construction, which is not a scope choice this test is entitled to make quietly.
//!
//! **The two signals joined.**
//! 1. **Structural risk** — `efferent::risky_to_change_findings` (the REAL organ, called here, not
//!    re-implemented) grades a symbol DREAD when BOTH its construct-in-degree (how depended-on its
//!    shape is) and its propagation-heat (how hot the code that builds it is) clear the top-quartile
//!    threshold — the cross-kind fight `emit_risky_to_change` first lit up.
//! 2. **Real defect sites** — `antigen::learn::szz::is_fix_commit` (REUSED, not reimplemented)
//!    classifies every commit in a full-object-graph `git log --all` walk; each fix (and, separately,
//!    every commit) contributes its `@@ -a,b +c,d @@` new-side touched-line ranges, one `git log -p`
//!    invocation for the whole history (not one process per commit).
//!
//! **The join.** Every touched line is attributed to the innermost containing symbol span (or
//! counted as unmappable residue when no span contains it — an honest, measured gap, never papered
//! over). Per symbol this yields `fix_lines` (distinct lines ever touched by a FIX commit) and
//! `all_lines` (distinct lines ever touched by ANY commit) — both bounded by `span_size` by
//! construction (a set of distinct lines inside a span can never exceed the span's length).
//!
//! **The two controls — both load-bearing, for different reasons.**
//! - **Control 1 (SIZE):** `fix-density = fix_lines / span_size`. Kills "big symbols catch more
//!   fixes because they have more lines" — the exact confound that killed v1.
//! - **Control 2 (CENTRALITY), the real test:** `fix-fraction = fix_lines / all_lines` — of the
//!   lines that changed AT ALL, what share were fixes. A symbol that is merely CHANGED a lot (churn)
//!   without a corresponding structural-risk story would show elevated `all_lines` but an ORDINARY
//!   fix-fraction; only a symbol whose changes are disproportionately CORRECTIVE clears this bar.
//!   This is the number the module doc of the brief calls load-bearing, precisely because raw
//!   fix-COUNT is tautologically confounded: "risky-to-change" already means "changing it is
//!   high-blast", and a fix IS a change — so of course risky symbols get touched by fixes. The
//!   fix-FRACTION asks the sharper question: controlled for how much a symbol changes at all, does
//!   a DISPROPORTIONATE share of its change-history consist of corrections.
//!
//! **Three informative outcomes** (framed before running, not hunted after):
//! - risky (DREAD) fix-fraction ≫ base → **SUPPORTED** (structural risk predicts real bugs beyond
//!   mere churn).
//! - risky fix-fraction ≪ base → **INVERSION** (the periphery carries the bugs; the risky center is
//!   the well-defended one — the very fact that changing it is high-blast may be WHY it gets more
//!   care).
//! - flat after both controls → **NULL** (structure doesn't predict bug-density here).
//!
//! **The honest scope-caveat this module's own analysis surfaces (not hand-waved).** Because
//! `is_constructor` restricts Construct-edge destinations to `Struct`/`Enum`/`EnumMember`, EVERY
//! DREAD symbol is a TYPE, never a function — `risky_to_change_findings`'s cross-kind fight requires
//! high construct-in-degree (a construct-edge property, i.e. a type) AND high heat, and a type's own
//! call-blast is structurally ~0 (types don't call), so its heat comes entirely from
//! `dependent_heat` (its builders' call-blast). This test therefore measures a specific, real
//! sub-question: do fixes concentrate INSIDE a risky type's OWN declaration span (field
//! additions/removals/retypes needed to correct a bug in the shape itself) — not "do fixes
//! concentrate in the (hot) functions that BUILD a risky type", which is a different, complementary
//! question this module does not test.
//!
//! Usage: `cargo run -p antigen-stroma --example founding_bet_v2 -- [path/to.scip] [repo_root]`
//! (defaults: `target/antigen.scip`, `.`; the `.scip` is regenerated via `rust-analyzer scip .` only
//! if missing, per the same convention `founding_bet_test.rs` uses).

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use antigen::finding::{FindingBody, Magnitude};
use antigen::learn::szz::{CommitMeta, is_fix_commit};
use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::efferent::{NodeSignal, risky_to_change_findings};
use antigen_stroma::engine::ReachabilityRelation;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::scip::{ingest_scip, populate_from_reconstructions};
use protobuf::Message;
use scip::types::{Index, SymbolRole};

// ── Small shared helpers (duplicated across examples by this crate's own convention — no
// shared examples-support crate exists, per `founding_bet_test.rs`'s precedent) ─────────────────

/// The 75th-percentile ("top-quartile floor") of a set of counts — IDENTICAL to
/// `emit_risky_to_change`'s threshold rule (computed over the nonzero values only, so a graph full
/// of zeros doesn't drag the cutoff to nothing).
fn top_quartile(mut nonzero: Vec<usize>) -> usize {
    if nonzero.is_empty() {
        return usize::MAX; // nothing is "high" -> the marker never fires
    }
    nonzero.sort_unstable();
    nonzero[nonzero.len() * 3 / 4]
}

/// Normalize a path to forward slashes — `rust-analyzer scip` and `git` both emit repo-relative
/// paths, but a Windows `rust-analyzer` run can emit backslashes; without this the join silently
/// finds nothing (every "file" a distinct key on each side) rather than erroring, so it is applied
/// unconditionally (`founding_bet_test.rs`'s established fix for the same hazard).
fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// Regenerate the SCIP index only if it is missing — never re-derive a fresh one over a present
/// (possibly team-generated) index.
fn ensure_scip_index(scip_path: &Path, repo_root: &Path) {
    if scip_path.exists() {
        return;
    }
    println!(
        "no SCIP index at {} — generating via `rust-analyzer scip .` (this can take a while)…",
        scip_path.display()
    );
    let status = Command::new("rust-analyzer")
        .args(["scip", "."])
        .arg("--output")
        .arg(scip_path)
        .current_dir(repo_root)
        .status()
        .expect("rust-analyzer must be on PATH to generate the SCIP index");
    assert!(status.success(), "rust-analyzer scip generation failed");
}

/// `(scip_path, repo_root)` from argv, with the documented defaults.
fn parse_args() -> (PathBuf, PathBuf) {
    let mut args = std::env::args().skip(1);
    let scip_path = Path::new(
        &args
            .next()
            .unwrap_or_else(|| "target/antigen.scip".to_string()),
    )
    .to_path_buf();
    let repo_root = args.next().map_or_else(
        || Path::new(".").to_path_buf(),
        |s| Path::new(&s).to_path_buf(),
    );
    (scip_path, repo_root)
}

// ── Structural risk: the REAL organ, called (not re-implemented) ───────────────────────────────

/// Parse the raw SCIP protobuf for every definition occurrence's `symbol -> (file, start_line)` —
/// the exact `definition_locations` helper `emit_risky_to_change` uses to build `NodeSignal`s
/// (duplicated verbatim; these are independent example binaries, same convention as v1).
fn definition_locations(scip_path: &Path) -> HashMap<String, (String, usize)> {
    let mut map = HashMap::new();
    let Ok(bytes) = std::fs::read(scip_path) else {
        return map;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return map;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if is_def && !occ.symbol.is_empty() && !occ.symbol.starts_with("local ") {
                let line = usize::try_from(occ.range.first().copied().unwrap_or(0)).unwrap_or(0);
                map.entry(occ.symbol.clone())
                    .or_insert_with(|| (doc.relative_path.clone(), line));
            }
        }
    }
    map
}

/// The DREAD classification, bundled so `main` doesn't re-thread six separate maps.
struct RiskGraph {
    /// Every symbol `risky_to_change_findings` graded DREAD (both signals high — the cross-kind
    /// fight: a depended-on shape built by hot code).
    dread: HashSet<String>,
    /// How many symbols entered the classification at all (had a def-location + >=1 signal).
    node_count: usize,
    /// The two convergence thresholds actually used (top-quartile of each signal's nonzero values).
    construct_hi: usize,
    heat_hi: usize,
}

/// Classify DREAD symbols EXACTLY as `emit_risky_to_change` does (identical construct-in-degree +
/// propagation-heat computation, identical top-quartile thresholds), then feed them through the REAL
/// organ — `risky_to_change_findings` — rather than re-deriving the corroboration rule in parallel.
/// `risky_to_change_findings` emits one `Finding` per signaling node IN ORDER (a single filter-then-
/// push loop, undocumented as a public contract but load-bearing here — the trailing `assert_eq!`
/// fails loud if that ever changes rather than silently mis-joining).
fn build_risk_graph(scip_path: &Path) -> RiskGraph {
    let recons = ingest_scip(scip_path);
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    let snap = SnapshotHandle::open(&db);
    let edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();

    let mut construct_in: HashMap<String, usize> = HashMap::new();
    for e in &edges {
        if e.kind == EdgeKind::Construct {
            *construct_in.entry(e.dst.fq_path.path.clone()).or_default() += 1;
        }
    }
    let call_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Call)
        .cloned()
        .collect();
    let rel_call = ReachabilityRelation::from_edges(&call_edges);
    let mut own_blast: HashMap<String, usize> = HashMap::new();
    for i in 0..rel_call.len() {
        let reached = rel_call.reach(i, true).nodes.len();
        own_blast.insert(rel_call.node(i).fq_path.path.clone(), reached);
    }
    let mut dependent_heat: HashMap<String, usize> = HashMap::new();
    for e in &edges {
        if e.kind == EdgeKind::Construct {
            let src_blast = own_blast.get(&e.src.fq_path.path).copied().unwrap_or(0);
            let slot = dependent_heat
                .entry(e.dst.fq_path.path.clone())
                .or_default();
            *slot = (*slot).max(src_blast);
        }
    }

    let def_loc = definition_locations(scip_path);
    let mut universe: Vec<String> = construct_in
        .keys()
        .chain(own_blast.keys())
        .chain(dependent_heat.keys())
        .cloned()
        .collect();
    universe.sort_unstable();
    universe.dedup();
    let heat_of = |path: &str| {
        own_blast
            .get(path)
            .copied()
            .unwrap_or(0)
            .max(dependent_heat.get(path).copied().unwrap_or(0))
    };
    let nodes: Vec<NodeSignal> = universe
        .iter()
        .filter_map(|path| {
            let (file, line) = def_loc.get(path)?;
            Some(NodeSignal {
                path: path.clone(),
                construct_in: construct_in.get(path).copied().unwrap_or(0),
                heat: heat_of(path),
                file: file.clone(),
                line: *line,
            })
        })
        .collect();

    let construct_hi = top_quartile(construct_in.values().copied().filter(|&v| v > 0).collect());
    let heat_hi = top_quartile(nodes.iter().map(|n| n.heat).filter(|&v| v > 0).collect());

    let findings = risky_to_change_findings(&nodes, construct_hi, heat_hi);
    let mut dread: HashSet<String> = HashSet::new();
    let mut fi = 0usize;
    for n in &nodes {
        let hi_construct = n.construct_in >= construct_hi && construct_hi > 0;
        let hi_heat = n.heat >= heat_hi && heat_hi > 0;
        if !(hi_construct || hi_heat) {
            continue; // risky_to_change_findings emits nothing for a zero-signal node
        }
        let f = &findings[fi];
        fi += 1;
        if let FindingBody::MarkedUnknown { magnitude, .. } = &f.body {
            if *magnitude == Magnitude::Dread {
                dread.insert(n.path.clone());
            }
        }
    }
    assert_eq!(
        fi,
        findings.len(),
        "risky_to_change_findings must emit exactly one Finding per signaling node, in order — a \
         change to that contract needs this join updated, not silently mis-aligned"
    );

    RiskGraph {
        dread,
        node_count: nodes.len(),
        construct_hi,
        heat_hi,
    }
}

// ── Body-span containment, generalized across the FULL symbol universe ─────────────────────────

/// Per-symbol body-span + per-file candidate-span table — the generalization of `ingest_scip`'s own
/// containment mechanism across every definition kind carrying an `enclosing_range` (function AND
/// struct/enum/enum-variant), not just the fn-only subset `ingest_scip` needs for naming a call's
/// caller. `enclosing_range` on ANY definition occurrence is that definition's own full span
/// (definition semantics — settled by `probe_enclosing`), so the identical exact-containment rule
/// generalizes cleanly to non-fn symbols.
struct ScipSpans {
    /// symbol -> (defining file, span start line, span end line) — the symbol's OWN body span.
    span_of: HashMap<String, (String, i32, i32)>,
    /// file -> candidate spans `(start, end, symbol)` in that file — the containment lookup table.
    file_spans: HashMap<String, Vec<(i32, i32, String)>>,
    /// How many spans came from a fn-shaped symbol vs. a non-fn (type/ctor) symbol — measured, not
    /// assumed: the diagnostic that confirms non-fn spans actually exist to attribute a DREAD type's
    /// own declaration lines to (DREAD is type-only by construction — see the module doc).
    fn_spans: usize,
    non_fn_spans: usize,
}

/// `true` for a Method-descriptor-shaped SCIP symbol string (the `).`-suffix convention `ingest_scip`
/// / the `probe_*` examples all use) — duplicated locally rather than exposed from `scip.rs`.
fn is_fn_symbol(s: &str) -> bool {
    !s.starts_with("local ") && s.ends_with(").")
}

/// Extract the start line from a SCIP range encoding (duplicated from `scip.rs`'s private helper).
fn range_start_line(r: &[i32]) -> i32 {
    r.first().copied().unwrap_or(0)
}

/// Extract the end line from a SCIP range encoding: `[sl, sc, el, ec]` (multi-line, end at index 2)
/// or `[sl, sc, ec]` (single-line, ends on its own start line) — duplicated from `scip.rs`.
fn range_end_line(r: &[i32]) -> i32 {
    if r.len() >= 4 {
        r[2]
    } else {
        r.first().copied().unwrap_or(0)
    }
}

/// Parse the SCIP index once for every definition occurrence carrying a non-empty `enclosing_range`
/// — the generalized body-span table (any symbol kind, unlike `ingest_scip`'s fn-only restriction).
/// A symbol defined in more than one document keeps its FIRST-seen span (harmless for this
/// aggregate; the same convention `founding_bet_test.rs`'s `symbol_to_file_map` uses).
fn parse_scip_spans(scip_path: &Path) -> ScipSpans {
    let mut span_of: HashMap<String, (String, i32, i32)> = HashMap::new();
    let mut file_spans: HashMap<String, Vec<(i32, i32, String)>> = HashMap::new();
    let (mut fn_spans, mut non_fn_spans) = (0usize, 0usize);

    let Ok(bytes) = std::fs::read(scip_path) else {
        return ScipSpans {
            span_of,
            file_spans,
            fn_spans,
            non_fn_spans,
        };
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return ScipSpans {
            span_of,
            file_spans,
            fn_spans,
            non_fn_spans,
        };
    };

    for doc in &index.documents {
        let file = normalize_path(&doc.relative_path);
        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if !is_def
                || occ.symbol.is_empty()
                || occ.symbol.starts_with("local ")
                || occ.enclosing_range.is_empty()
            {
                continue;
            }
            let start = range_start_line(&occ.enclosing_range);
            let end = range_end_line(&occ.enclosing_range);
            if is_fn_symbol(&occ.symbol) {
                fn_spans += 1;
            } else {
                non_fn_spans += 1;
            }
            span_of
                .entry(occ.symbol.clone())
                .or_insert_with(|| (file.clone(), start, end));
            file_spans
                .entry(file.clone())
                .or_default()
                .push((start, end, occ.symbol.clone()));
        }
    }
    ScipSpans {
        span_of,
        file_spans,
        fn_spans,
        non_fn_spans,
    }
}

// ── SZZ: full-object-graph line-level mining, ONE process spawn for the whole history ───────────

/// Per-file distinct touched-line footprints, mined from a SINGLE `git log --all --unified=0 -p`
/// walk (not one process spawn per commit — 1046 commits parse in under a second from one stream,
/// the same single-invocation discipline `founding_bet_test.rs`'s `--name-only` walk uses,
/// generalized from file-lists to line-ranges).
struct GitLineTouches {
    /// file -> the set of DISTINCT line numbers EVER touched by ANY commit (the change baseline).
    all_touched: HashMap<String, HashSet<i32>>,
    /// file -> the set of DISTINCT line numbers EVER touched by a FIX-classified commit.
    fix_touched: HashMap<String, HashSet<i32>>,
    total_commits: usize,
    total_fix_commits: usize,
}

/// Mine `GitLineTouches` via `antigen::learn::szz::is_fix_commit` (REUSED — the fix-commit
/// definition here is IDENTICAL to `cargo antigen mine`'s, not a parallel heuristic), classifying
/// every commit reached by a full-object-graph `--all` walk.
///
/// Parsing scheme: `--pretty=format:%x01%H%x02%s` prefixes each commit with a SOH/STX-delimited
/// header (bytes unlikely in a commit subject); `-p --unified=0` emits each file's diff with
/// ZERO-context hunks, so a hunk's `@@ -a,b +c,d @@` header names EXACTLY the touched lines (no
/// surrounding context lines to mis-attribute). A merge commit shows no diff under plain `git log
/// -p` (needs `-m`/`--cc`, not requested here) — an honest, small coverage gap, unchanged from v1.
fn mine_git_line_touches(repo_root: &Path) -> GitLineTouches {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["log", "--all", "--unified=0", "-p", "--format=%x01%H%x02%s"])
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git log --all -p failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);

    let mut all_touched: HashMap<String, HashSet<i32>> = HashMap::new();
    let mut fix_touched: HashMap<String, HashSet<i32>> = HashMap::new();
    let mut total_commits = 0usize;
    let mut total_fix_commits = 0usize;

    for chunk in text.split('\u{1}').filter(|c| !c.trim().is_empty()) {
        let (head, body) = chunk.split_once('\n').unwrap_or((chunk, ""));
        let Some((id, subject)) = head.split_once('\u{2}') else {
            continue;
        };
        total_commits += 1;
        let meta = CommitMeta {
            id: id.to_string(),
            subject: subject.to_string(),
            files_changed: Vec::new(), // not used by is_fix_commit; skipped for speed
            cosmetic_only: false,      // never computed here — same honest gap as v1
        };
        let is_fix = is_fix_commit(&meta);
        if is_fix {
            total_fix_commits += 1;
        }

        let mut current_file: Option<String> = None;
        for line in body.lines() {
            if let Some(rest) = line.strip_prefix("+++ ") {
                current_file = parse_plus_plus_path(rest);
                continue;
            }
            let Some(rest) = line.strip_prefix("@@ ") else {
                continue;
            };
            let Some((new_start, new_len)) = parse_hunk_new_range(rest) else {
                continue;
            };
            if new_len == 0 {
                continue; // a pure-deletion hunk — nothing NEW at this file to attribute
            }
            let Some(file) = &current_file else { continue };
            let touched = all_touched.entry(file.clone()).or_default();
            for l in new_start..new_start + new_len {
                touched.insert(l);
            }
            if is_fix {
                let touched_fix = fix_touched.entry(file.clone()).or_default();
                for l in new_start..new_start + new_len {
                    touched_fix.insert(l);
                }
            }
        }
    }
    GitLineTouches {
        all_touched,
        fix_touched,
        total_commits,
        total_fix_commits,
    }
}

/// The file a `+++ ` patch header names — `b/path` -> `Some(path)` (forward-slash normalized),
/// `/dev/null` (a deleted file) -> `None` (nothing at HEAD to attribute a touch to).
fn parse_plus_plus_path(rest: &str) -> Option<String> {
    rest.trim_end().strip_prefix("b/").map(normalize_path)
}

/// The new-side `(start_line, line_count)` from a `@@ -a,b +c,d @@ trailer` hunk header (the part
/// AFTER the `@@ ` this function is called with) — `d` defaults to `1` when the `+c` token carries
/// no comma (unified-diff's single-line shorthand, confirmed present in this repo's own history).
fn parse_hunk_new_range(rest: &str) -> Option<(i32, i32)> {
    let plus_tok = rest.split_whitespace().find(|t| t.starts_with('+'))?;
    let spec = plus_tok.strip_prefix('+')?;
    let (start_str, len_str) = spec.split_once(',').unwrap_or((spec, "1"));
    Some((start_str.parse().ok()?, len_str.parse().ok()?))
}

/// Attribute each touched line to its INNERMOST containing symbol span — the identical exact-
/// containment rule `ingest_scip` uses for call-edge reconstruction, applied here to git-diff-
/// touched lines instead of SCIP reference occurrences. Returns per-symbol distinct touched-line
/// counts + how many touches mapped vs. fell inside no span at all (the honest unmappable residue).
fn attribute_touches(
    touched: &HashMap<String, HashSet<i32>>,
    file_spans: &HashMap<String, Vec<(i32, i32, String)>>,
) -> (HashMap<String, usize>, usize, usize) {
    let mut per_symbol: HashMap<String, usize> = HashMap::new();
    let (mut mapped, mut unmapped) = (0usize, 0usize);
    for (file, lines) in touched {
        let spans = file_spans.get(file);
        for &l in lines {
            let found = spans.and_then(|sp| {
                sp.iter()
                    .filter(|&&(s, e, _)| s <= l && l <= e)
                    .min_by_key(|&&(s, e, _)| e - s)
            });
            match found {
                Some((_, _, sym)) => {
                    *per_symbol.entry(sym.clone()).or_insert(0) += 1;
                    mapped += 1;
                },
                None => unmapped += 1,
            }
        }
    }
    (per_symbol, mapped, unmapped)
}

// ── Symbol rows: the join of structural risk + real defect history ─────────────────────────────

/// One symbol's analysis unit: is it DREAD, how big is its span, how many of its lines were EVER
/// touched by a fix vs. by any commit. Every field a measured aggregate.
struct SymbolRow {
    is_dread: bool,
    span_size: f64,
    fix_lines: f64,
    all_lines: f64,
}

/// Build one `SymbolRow` per symbol carrying a body-span (the analysis universe — a symbol with no
/// span cannot be joined to a git line, so it cannot be part of a structure-vs-defect comparison).
fn build_rows(
    span_of: &HashMap<String, (String, i32, i32)>,
    dread: &HashSet<String>,
    fix_by_symbol: &HashMap<String, usize>,
    all_by_symbol: &HashMap<String, usize>,
) -> Vec<SymbolRow> {
    span_of
        .iter()
        .map(|(sym, &(_, start, end))| SymbolRow {
            is_dread: dread.contains(sym),
            span_size: f64::from((end - start + 1).max(1)),
            fix_lines: fix_by_symbol.get(sym).copied().unwrap_or(0) as f64,
            all_lines: all_by_symbol.get(sym).copied().unwrap_or(0) as f64,
        })
        .collect()
}

// ── Statistics: risky-vs-base comparison + Pearson collinearity ─────────────────────────────────

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

/// Pearson correlation coefficient — `None` when either series has zero variance (undefined, not
/// zero: reporting `0.0` would misleadingly read as "no relationship" rather than "not computable").
fn pearson(xs: &[f64], ys: &[f64]) -> Option<f64> {
    if xs.is_empty() || xs.len() != ys.len() {
        return None;
    }
    let (mx, my) = (mean(xs), mean(ys));
    let (mut cov, mut vx, mut vy) = (0.0, 0.0, 0.0);
    for (x, y) in xs.iter().zip(ys) {
        let (dx, dy) = (x - mx, y - my);
        cov = dx.mul_add(dy, cov);
        vx = dx.mul_add(dx, vx);
        vy = dy.mul_add(dy, vy);
    }
    (vx > 0.0 && vy > 0.0).then(|| cov / (vx.sqrt() * vy.sqrt()))
}

/// One control's risky-vs-base comparison: `metric`'s mean over DREAD rows vs. over ALL eligible
/// rows. The base INCLUDES the risky subset (the same "top group is a subset of the base"
/// convention `founding_bet_test.rs`'s `enrichment` uses, kept for methodological consistency across
/// the two versions of this test).
struct Comparison {
    risky_mean: f64,
    risky_n: usize,
    base_mean: f64,
    base_n: usize,
}

impl Comparison {
    fn ratio(&self) -> Option<f64> {
        (self.base_mean > 0.0).then_some(self.risky_mean / self.base_mean)
    }
}

/// Compare `metric` over `{rows filtered by eligible}` (base) vs. `{those rows that are also DREAD}`
/// (risky).
fn compare(
    rows: &[SymbolRow],
    eligible: impl Fn(&SymbolRow) -> bool,
    metric: impl Fn(&SymbolRow) -> f64,
) -> Comparison {
    let base_vals: Vec<f64> = rows.iter().filter(|r| eligible(r)).map(&metric).collect();
    let risky_vals: Vec<f64> = rows
        .iter()
        .filter(|r| r.is_dread && eligible(r))
        .map(&metric)
        .collect();
    Comparison {
        risky_mean: mean(&risky_vals),
        risky_n: risky_vals.len(),
        base_mean: mean(&base_vals),
        base_n: base_vals.len(),
    }
}

fn print_comparison(c: &Comparison) {
    match c.ratio() {
        Some(r) => println!(
            "  DREAD (n={}) mean={:.4}   BASE (n={}) mean={:.4}   enrichment ×{r:.2}",
            c.risky_n, c.risky_mean, c.base_n, c.base_mean
        ),
        None => println!(
            "  BASE mean is zero — cannot compute a ratio (DREAD n={}, BASE n={})",
            c.risky_n, c.base_n
        ),
    }
}

/// The three-way verdict, driven by Control 2 (fix-fraction) — the load-bearing number per the
/// module doc: raw fix-count/density is expected-confounded by "risky = high-blast = changed a lot",
/// but fix-fraction asks whether the SHARE of change that is corrective is elevated.
fn verdict(fraction_ratio: Option<f64>) -> &'static str {
    match fraction_ratio {
        Some(r) if r >= 1.2 => {
            "SUPPORTED — DREAD symbols carry a higher fix-FRACTION than base: structural risk \
             predicts real bugs beyond mere churn."
        },
        Some(r) if r <= 1.0 / 1.2 => {
            "INVERSION — DREAD symbols carry a LOWER fix-fraction than base: the periphery carries \
             the bugs, not the risky center — the very centrality that makes a symbol risky-to-\
             change may be why it gets more care."
        },
        Some(_) => {
            "NULL — fix-fraction is flat between DREAD and base: structure doesn't predict \
                    bug-density here, controlled or not."
        },
        None => {
            "INDETERMINATE — the base fix-fraction population is empty (no span-bearing symbol \
                 was ever touched by any commit)."
        },
    }
}

fn print_caveats() {
    println!(
        "\n== Honest caveats ==\n\
         - HISTORICAL LINE DRIFT (the sharpest threat to validity): a hunk's new-side line number \
         is relative to the file's state AT THAT COMMIT, but every symbol span is read from HEAD's \
         SCIP index. On a file with deep history, intervening edits shift line numbers — an old \
         commit's touched-line-12 may not be line-12's symbol today. This is un-corrected (no \
         historical re-blame is run); it adds NOISE to the join, most on long-lived, heavily-edited \
         files, and is the honest reason this is called \"H1-lite\" rather than a full resolver.\n\
         - DREAD IS TYPE-ONLY BY CONSTRUCTION (see module doc): this test asks whether fixes \
         concentrate INSIDE a risky type's own declaration span, not whether fixes concentrate in \
         the (hot) functions that BUILD it — a real, different, complementary question this module \
         does not test.\n\
         - `cosmetic_only` is never computed (nor by `cargo antigen mine` itself) — every commit is \
         assumed non-cosmetic.\n\
         - Merge commits contribute no line-touches under plain `git log -p` (no `-m`/`--cc`) — an \
         honest, uncorrected gap, unchanged from v1.\n\
         - Single corpus, single repo (antigen's own history) — no cross-project replication."
    );
}

// ── Main (orchestration only) ────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)] // one cohesive end-to-end measurement reads best as one unit
fn main() {
    let (scip_path, repo_root) = parse_args();
    println!(
        "== founding-bet v2: SYMBOL-level defect-concentration (DREAD vs. real SZZ history) =="
    );
    println!(
        "Hypothesis: real SZZ-mined defect LINES concentrate inside DREAD (risky-to-change) \
         symbols' own body-spans, above the base rate — WITH both the size confound AND the \
         centrality confound controlled for.\n"
    );

    ensure_scip_index(&scip_path, &repo_root);

    let risk = build_risk_graph(&scip_path);
    println!(
        "stroma risk classification: {} signaling nodes, DREAD={} (construct-in>={}, heat>={})",
        risk.node_count,
        risk.dread.len(),
        risk.construct_hi,
        risk.heat_hi
    );

    let spans = parse_scip_spans(&scip_path);
    println!(
        "body-spans (any definition w/ enclosing_range): {} symbols ({} fn-shaped, {} non-fn/type-shaped)",
        spans.span_of.len(),
        spans.fn_spans,
        spans.non_fn_spans
    );
    let dread_with_span = risk
        .dread
        .iter()
        .filter(|d| spans.span_of.contains_key(*d))
        .count();
    println!(
        "DREAD symbols carrying an attributable span: {dread_with_span}/{}",
        risk.dread.len()
    );

    let git = mine_git_line_touches(&repo_root);
    println!(
        "\nSZZ (re-derived, full-object-graph `git log --all`): {} fix-classified / {} commits walked",
        git.total_fix_commits, git.total_commits
    );

    let (fix_by_symbol, fix_mapped, fix_unmapped) =
        attribute_touches(&git.fix_touched, &spans.file_spans);
    let (all_by_symbol, all_mapped, all_unmapped) =
        attribute_touches(&git.all_touched, &spans.file_spans);
    println!(
        "fix-touched lines: {fix_mapped} mapped to a containing symbol, {fix_unmapped} unmappable \
         (no span at that file:line)"
    );
    println!(
        "all-touched lines: {all_mapped} mapped to a containing symbol, {all_unmapped} unmappable"
    );

    let rows = build_rows(&spans.span_of, &risk.dread, &fix_by_symbol, &all_by_symbol);
    let risky_n = rows.iter().filter(|r| r.is_dread).count();
    println!(
        "\nanalysis universe: {} symbols with a body-span, {risky_n} of them DREAD",
        rows.len()
    );

    println!("\n== Control 1 (SIZE): fix-density = fix_lines / span_size ==");
    let density = compare(&rows, |_| true, |r| r.fix_lines / r.span_size);
    print_comparison(&density);

    println!(
        "\n== Control 2 (CENTRALITY, the load-bearing test): fix-fraction = fix_lines / all_lines =="
    );
    let fraction = compare(&rows, |r| r.all_lines > 0.0, |r| r.fix_lines / r.all_lines);
    print_comparison(&fraction);

    let risky_indicator: Vec<f64> = rows
        .iter()
        .map(|r| f64::from(u8::from(r.is_dread)))
        .collect();
    let span_sizes: Vec<f64> = rows.iter().map(|r| r.span_size).collect();
    let all_lines_vals: Vec<f64> = rows.iter().map(|r| r.all_lines).collect();
    println!("\n== Collinearity (is DREAD just re-measuring size or churn?) ==");
    println!(
        "  r(DREAD, span_size) = {}",
        pearson(&risky_indicator, &span_sizes)
            .map_or_else(|| "n/a".to_string(), |v| format!("{v:.2}"))
    );
    println!(
        "  r(DREAD, all_lines)  = {}",
        pearson(&risky_indicator, &all_lines_vals)
            .map_or_else(|| "n/a".to_string(), |v| format!("{v:.2}"))
    );

    println!("\n== VERDICT ==\n  {}", verdict(fraction.ratio()));
    print_caveats();
}

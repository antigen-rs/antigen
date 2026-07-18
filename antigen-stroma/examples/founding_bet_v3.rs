//! The founding-bet defect-concentration test, v3 — the PROPAGATION slice.
//!
//! **What v2 measured, and the question it left open.** v2 (`founding_bet_v2.rs`) tests whether SZZ
//! fix-lines concentrate inside a DREAD (risky-to-change) TYPE's own declaration span — but
//! `risky_to_change_findings` grades DREAD on construct-in-degree (a Construct-edge property, i.e. a
//! type) AND propagation-heat (`dependent_heat`: the max call-blast of the FUNCTIONS that build that
//! type). A type's own body is almost always small and rarely touched for its own sake; the DREAD
//! grade's entire "why is this risky" story is about the hot code that BUILDS the type, not the type's
//! declaration. v2's own module doc names this as a real, untested, complementary question. v3 is that
//! question.
//!
//! **The propagation hypothesis.** If structural risk propagates the way the founding bet predicts,
//! fixes should concentrate not in a risky type's own (small, rarely-touched) declaration, but in the
//! constructor/builder FUNCTIONS that build it — the actual call-sites where a shape change, a bad
//! invariant, or a wrong default would be introduced or repaired. So: for every DREAD type, the target
//! span is redefined as the UNION of the def-spans of every function that is the SOURCE of a
//! Construct-edge into that type (`e.kind == Construct && e.dst == the type`, `e.src` = a builder).
//!
//! **The plumbing is a direct generalization of v2, not a new pipeline.** Everything upstream is
//! identical and reused verbatim: the same `risky_to_change_findings` DREAD classification (same
//! organ, same thresholds), the same `is_fix_commit`-driven SZZ line mining (same one-process
//! `git log --all -p` walk), the same exact-containment fix-line/all-line attribution per symbol
//! (`attribute_touches`, unchanged — it already attributes every touched line to its INNERMOST
//! containing span regardless of symbol kind, function or type). The ONLY change is which spans get
//! pooled into a "row": v2 pools ONE span per DREAD type (its own declaration); v3 pools the UNION of
//! several builder-function spans per DREAD type, summing each builder's already-attributed
//! `fix_lines`/`all_lines`/`span_size` (no re-derivation of the containment join — same numbers v2
//! would compute for those builder symbols, just re-grouped by which type(s) they build).
//!
//! **Row population (the base, so DREAD is a comparable subset of it, per v2's convention).** Every
//! type `T` that is EVER the destination of a Construct-edge from a function-shaped symbol AND has at
//! least one such builder with an attributable body-span gets one row: `is_dread = T is DREAD-graded`,
//! `span_size = Σ builder span sizes`, `fix_lines = Σ builder fix_lines`, `all_lines = Σ builder
//! all_lines` (builders de-duplicated per type — a builder that constructs the SAME type at multiple
//! call-sites contributes only once, per type; see harness-verification output for the honest disclosure
//! of the DIFFERENT, cross-row overlap: a builder that constructs TWO DREAD types contributes to both
//! types' rows, because it genuinely does propagate risk to both — measured and reported, not hidden).
//!
//! **The two controls — identical in spirit to v2, same load-bearing role.**
//! - Control 1 (SIZE): fix-density = `fix_lines` / `span_size`. Kills "more builder-code -> mechanically
//!   more fixes".
//! - Control 2 (CENTRALITY, load-bearing): fix-fraction = `fix_lines` / `all_lines`. Of the lines that
//!   changed AT ALL inside a DREAD type's builders, what share were CORRECTIVE. This is the number
//!   that answers the propagation question: not "do risky types' builders change a lot" (tautological
//!   — high-blast code changes a lot by definition) but "is a disproportionate SHARE of that change
//!   corrective".
//!
//! **Three informative outcomes** (same framing as v2, aimed at the builder-union instead of the
//! type's own declaration):
//! - risky-builders fix-fraction ≫ base -> SUPPORTED (risk propagates to callers: the functions that
//!   build a risky type carry a disproportionate share of real bugs).
//! - risky-builders fix-fraction ≪ base -> INVERSION (the builders of a risky type are, if anything,
//!   the well-defended ones — consistent with v2's own inversion, now tested on the complementary
//!   slice).
//! - flat -> NULL.
//!
//! Usage: `cargo run -p antigen-stroma --example founding_bet_v3 -- [path/to.scip] [repo_root]`
//! (defaults identical to v2: `target/antigen.scip`, `.`).

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

// ── Small shared helpers (identical to v2 — duplicated across examples by this crate's own
// convention, no shared examples-support crate exists) ─────────────────────────────────────────────

/// The 75th-percentile ("top-quartile floor") of a set of counts — IDENTICAL to
/// `emit_risky_to_change`'s threshold rule.
fn top_quartile(mut nonzero: Vec<usize>) -> usize {
    if nonzero.is_empty() {
        return usize::MAX;
    }
    nonzero.sort_unstable();
    nonzero[nonzero.len() * 3 / 4]
}

/// Normalize a path to forward slashes — `rust-analyzer scip` and `git` both emit repo-relative
/// paths, but a Windows run can emit backslashes.
fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// Regenerate the SCIP index only if it is missing.
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

/// Parse the raw SCIP protobuf for every definition occurrence's `symbol -> (file, start_line)`.
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

/// The DREAD classification, PLUS every Construct edge `(builder_src, type_dst)` — v3 needs the raw
/// edge list (v2 only needed the aggregated in-degree count) to resolve, per DREAD type, WHICH
/// functions build it.
struct RiskGraph {
    /// Every symbol `risky_to_change_findings` graded DREAD.
    dread: HashSet<String>,
    /// How many symbols entered the classification at all.
    node_count: usize,
    /// The two convergence thresholds actually used.
    construct_hi: usize,
    heat_hi: usize,
    /// Every Construct edge as `(src, dst)` symbol-path pairs — the builder->type relation v3 joins
    /// against body-spans.
    construct_edges: Vec<(String, String)>,
}

/// Classify DREAD symbols EXACTLY as `emit_risky_to_change` does, then feed them through the REAL
/// organ — `risky_to_change_findings` — identical to v2's `build_risk_graph`, plus capturing the raw
/// Construct edge list for v3's builder resolution.
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
    let mut construct_edges: Vec<(String, String)> = Vec::new();
    for e in &edges {
        if e.kind == EdgeKind::Construct {
            *construct_in.entry(e.dst.fq_path.path.clone()).or_default() += 1;
            construct_edges.push((e.src.fq_path.path.clone(), e.dst.fq_path.path.clone()));
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
            continue;
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
        "risky_to_change_findings must emit exactly one Finding per signaling node, in order"
    );

    RiskGraph {
        dread,
        node_count: nodes.len(),
        construct_hi,
        heat_hi,
        construct_edges,
    }
}

// ── Body-span containment, generalized across the FULL symbol universe (identical to v2) ──────

struct ScipSpans {
    span_of: HashMap<String, (String, i32, i32)>,
    file_spans: HashMap<String, Vec<(i32, i32, String)>>,
    fn_spans: usize,
    non_fn_spans: usize,
}

/// `true` for a Method-descriptor-shaped SCIP symbol string — the signal v3 uses to restrict
/// "builder" to FUNCTION-shaped Construct-edge sources (the task's "constructor/builder functions"),
/// excluding the rare non-fn edge source (e.g. a `const` initializer) so the propagation slice targets
/// exactly the callable code the founding bet is about.
fn is_fn_symbol(s: &str) -> bool {
    !s.starts_with("local ") && s.ends_with(").")
}

/// Heuristic `true` if a builder symbol is TEST code — the PROJECTION-time filter (tag-not-filter /
/// hold-all: the substrate KEEPS test code; this organ projects to PRODUCTION builders only). Two signals,
/// both from what the SCIP index already gives us:
/// - **file** under `tests/` / `benches/` (integration tests + benches live in their own dirs);
/// - **symbol path** containing a `tests`/`test` module descriptor — a `#[cfg(test)] mod tests { … }` unit
///   test compiles into a `…/tests/fn().` symbol even though its file is a normal `src/` file.
///
/// First-cut and honest about its ceiling: it cannot see a bare `#[cfg(test)] fn` outside a conventionally-
/// named module (that wants the syn instrument's cfg attributes, not yet wired). The count of excluded
/// builders is DISCLOSED, never silent.
fn is_test_symbol(symbol: &str, file: &str) -> bool {
    let f = normalize_path(file);
    if f.starts_with("tests/")
        || f.contains("/tests/")
        || f.starts_with("benches/")
        || f.contains("/benches/")
    {
        return true;
    }
    symbol.contains("/tests/") || symbol.contains("/test/")
}

fn range_start_line(r: &[i32]) -> i32 {
    r.first().copied().unwrap_or(0)
}

fn range_end_line(r: &[i32]) -> i32 {
    if r.len() >= 4 {
        r[2]
    } else {
        r.first().copied().unwrap_or(0)
    }
}

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

// ── SZZ: full-object-graph line-level mining, ONE process spawn for the whole history (identical
// to v2) ─────────────────────────────────────────────────────────────────────────────────────────

struct GitLineTouches {
    all_touched: HashMap<String, HashSet<i32>>,
    fix_touched: HashMap<String, HashSet<i32>>,
    total_commits: usize,
    total_fix_commits: usize,
}

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
            files_changed: Vec::new(),
            cosmetic_only: false,
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
                continue;
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

fn parse_plus_plus_path(rest: &str) -> Option<String> {
    rest.trim_end().strip_prefix("b/").map(normalize_path)
}

fn parse_hunk_new_range(rest: &str) -> Option<(i32, i32)> {
    let plus_tok = rest.split_whitespace().find(|t| t.starts_with('+'))?;
    let spec = plus_tok.strip_prefix('+')?;
    let (start_str, len_str) = spec.split_once(',').unwrap_or((spec, "1"));
    Some((start_str.parse().ok()?, len_str.parse().ok()?))
}

/// Attribute each touched line to its INNERMOST containing symbol span — unchanged from v2. Works
/// identically for function spans and type spans; v3 reuses it to get per-BUILDER (function) fix/all
/// line counts, then re-groups those by which DREAD type(s) each builder constructs.
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

// ── The propagation join: per DREAD-TYPE union-of-builder-spans (the v3-specific step) ────────

/// One row per CONSTRUCTED TYPE (not per symbol, unlike v2) — `is_dread` flags whether the type
/// itself is DREAD-graded; the three metrics are SUMS over that type's distinct FUNCTION-shaped
/// Construct-edge sources ("builders"), each builder's `fix_lines`/`all_lines`/`span_size` computed by
/// the SAME containment join v2 uses (`attribute_touches`), never re-derived.
struct SymbolRow {
    is_dread: bool,
    span_size: f64,
    fix_lines: f64,
    all_lines: f64,
}

/// Harness-verification counters — printed so the propagation join's honesty is checked, not assumed.
struct BuilderJoinDiagnostics {
    /// Distinct types that are EVER the destination of a Construct-edge from a function-shaped src.
    types_total: usize,
    /// Of those, how many have at least one builder with an attributable body-span (the actual row
    /// population — a type whose every builder lacks a span cannot be measured and is DROPPED, not
    /// zero-filled).
    types_with_rows: usize,
    /// How many DREAD types exist at all (from `RiskGraph::dread`).
    dread_total: usize,
    /// Of the DREAD types, how many have >=1 Construct-edge from ANY source (fn or not).
    dread_with_any_builder_edge: usize,
    /// Of those, how many survive into a row (>=1 builder with an attributable, function-shaped
    /// span) — the population Control-1/Control-2 actually measure the "risky" side over.
    dread_with_rows: usize,
    /// Construct-edge sources that were NOT function-shaped (e.g. a const/static initializer) —
    /// excluded from "builder" by construction (the task's "constructor/builder FUNCTIONS").
    non_fn_edge_sources_skipped: usize,
    /// Paranoia check: a builder whose resolved span is IDENTICAL to its type's own span (would mean
    /// "builder" and "type" collapsed onto the same definition — should be 0; a nonzero value would
    /// mean the union is NOT distinct from v2's own-declaration span and the propagation slice would
    /// not actually be testing a different question).
    builder_span_equals_type_span: usize,
    /// Distinct builder symbols that are shared across MORE THAN ONE DREAD type's builder set — an
    /// honest, disclosed non-independence (that builder's fix/all lines are summed into BOTH types'
    /// rows, because it genuinely builds both), not a within-row double-count.
    builders_shared_across_dread_types: usize,
    /// Distinct TEST-shaped builder symbols EXCLUDED at projection time (hold-all: the graph kept them;
    /// this organ filters them out — test fixtures construct many types and would dilute the
    /// propagation signal). Disclosed so the filter's reach is measured, not assumed.
    test_builders_excluded: usize,
}

fn build_builder_rows(
    dread: &HashSet<String>,
    construct_edges: &[(String, String)],
    span_of: &HashMap<String, (String, i32, i32)>,
    fix_by_symbol: &HashMap<String, usize>,
    all_by_symbol: &HashMap<String, usize>,
) -> (Vec<SymbolRow>, BuilderJoinDiagnostics) {
    // type -> distinct function-shaped builder symbols (dedups a builder that constructs the SAME
    // type at multiple call-sites — the within-row double-count guard).
    let mut builders_of_type: HashMap<String, HashSet<String>> = HashMap::new();
    let mut non_fn_edge_sources_skipped = 0usize;
    for (src, dst) in construct_edges {
        if src == dst {
            continue; // a type never constructs itself; defensive, not expected to fire
        }
        if !is_fn_symbol(src) {
            non_fn_edge_sources_skipped += 1;
            continue;
        }
        builders_of_type
            .entry(dst.clone())
            .or_default()
            .insert(src.clone());
    }

    let types_total = builders_of_type.len();
    let dread_with_any_builder_edge = builders_of_type
        .keys()
        .filter(|t| dread.contains(*t))
        .count();

    let mut builder_span_equals_type_span = 0usize;
    let mut builder_dread_type_count: HashMap<String, usize> = HashMap::new();
    // PROJECTION-time test filter (tag-not-filter / hold-all): the graph KEEPS test code; this organ
    // projects to PRODUCTION builders only — test fixtures construct many struct-literals and would
    // otherwise dilute the propagation signal. An organ's choice to exclude, not a partition of the
    // substrate. Applied uniformly to DREAD and BASE rows so the comparison stays fair; count disclosed.
    let mut test_builders_excluded: HashSet<String> = HashSet::new();
    let mut rows = Vec::new();

    for (type_path, builders) in &builders_of_type {
        let is_dread = dread.contains(type_path);
        let builders_with_span: Vec<&String> = builders
            .iter()
            .filter(|b| span_of.contains_key(*b))
            .filter(|b| {
                let is_test = span_of
                    .get(*b)
                    .is_some_and(|(file, ..)| is_test_symbol(b, file));
                if is_test {
                    test_builders_excluded.insert((*b).clone());
                }
                !is_test
            })
            .collect();
        if builders_with_span.is_empty() {
            continue; // no attributable PRODUCTION builder span for this type — DROPPED, not zero-filled
        }
        if let Some(type_span) = span_of.get(type_path) {
            for b in &builders_with_span {
                if span_of.get(*b) == Some(type_span) {
                    builder_span_equals_type_span += 1;
                }
            }
        }
        if is_dread {
            for b in &builders_with_span {
                *builder_dread_type_count.entry((*b).clone()).or_default() += 1;
            }
        }

        let mut span_size = 0.0;
        let mut fix_lines = 0.0;
        let mut all_lines = 0.0;
        for b in &builders_with_span {
            let (_, start, end) = span_of[*b];
            span_size += f64::from((end - start + 1).max(1));
            fix_lines += fix_by_symbol.get(*b).copied().unwrap_or(0) as f64;
            all_lines += all_by_symbol.get(*b).copied().unwrap_or(0) as f64;
        }
        rows.push(SymbolRow {
            is_dread,
            span_size,
            fix_lines,
            all_lines,
        });
    }

    let types_with_rows = rows.len();
    let dread_with_rows = rows.iter().filter(|r| r.is_dread).count();
    let builders_shared_across_dread_types = builder_dread_type_count
        .values()
        .filter(|&&c| c > 1)
        .count();

    let diag = BuilderJoinDiagnostics {
        types_total,
        types_with_rows,
        dread_total: dread.len(),
        dread_with_any_builder_edge,
        dread_with_rows,
        non_fn_edge_sources_skipped,
        builder_span_equals_type_span,
        builders_shared_across_dread_types,
        test_builders_excluded: test_builders_excluded.len(),
    };
    (rows, diag)
}

// ── Statistics: risky-vs-base comparison + Pearson collinearity (identical to v2) ──────────────

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

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

fn verdict(fraction_ratio: Option<f64>) -> &'static str {
    match fraction_ratio {
        Some(r) if r >= 1.2 => {
            "SUPPORTED — DREAD types' BUILDERS carry a higher fix-FRACTION than base: structural risk \
             propagates to the functions that build a risky type, beyond mere churn."
        },
        Some(r) if r <= 1.0 / 1.2 => {
            "INVERSION — DREAD types' BUILDERS carry a LOWER fix-fraction than base: the builders of a \
             risky type are, if anything, the well-defended ones — consistent with v2's own-declaration \
             inversion, now on the complementary propagation slice."
        },
        Some(_) => {
            "NULL — fix-fraction is flat between DREAD-type builders and base: structure doesn't \
             predict bug-density in the builder-union either, controlled or not."
        },
        None => {
            "INDETERMINATE — the base fix-fraction population is empty (no builder-bearing type row \
             was ever touched by any commit)."
        },
    }
}

fn print_diagnostics(diag: &BuilderJoinDiagnostics) {
    println!("\n== Harness verification (builder-union join) ==");
    println!(
        "  constructed types total (>=1 fn-shaped builder edge): {}",
        diag.types_total
    );
    println!(
        "  constructed types with >=1 attributable builder span (the row population): {}",
        diag.types_with_rows
    );
    println!(
        "  DREAD types total: {}   with >=1 builder edge at all: {}   surviving into a row: {}",
        diag.dread_total, diag.dread_with_any_builder_edge, diag.dread_with_rows
    );
    println!(
        "  non-fn-shaped Construct-edge sources skipped (not a \"builder function\"): {}",
        diag.non_fn_edge_sources_skipped
    );
    println!(
        "  builder span == type's own span (should be 0 — else builder-union collapses onto v2's \
         own-declaration slice): {}",
        diag.builder_span_equals_type_span
    );
    println!(
        "  builders shared across >1 DREAD type (disclosed cross-row overlap, not a within-row \
         double-count): {}",
        diag.builders_shared_across_dread_types
    );
    println!(
        "  TEST-shaped builders excluded at projection (tag-not-filter: graph kept them, organ \
         projects to production): {}",
        diag.test_builders_excluded
    );
}

fn print_caveats() {
    println!(
        "\n== Honest caveats ==\n\
         - HISTORICAL LINE DRIFT (same threat as v2): a hunk's new-side line number is relative to \
         the file's state AT THAT COMMIT, but every symbol span is read from HEAD's SCIP index. \
         Un-corrected; adds noise, worst on long-lived heavily-edited files.\n\
         - CROSS-ROW OVERLAP: a builder function that constructs TWO DREAD types contributes its \
         fix_lines/all_lines to BOTH types' rows (see harness verification count above). This is a \
         real, disclosed non-independence between rows, not a within-row double-count (which is \
         guarded against via per-type builder de-duplication) — it means the DREAD row population is \
         not a partition of code, some builder code is counted more than once across DIFFERENT DREAD \
         types it genuinely serves.\n\
         - A TYPE WITH NO ATTRIBUTABLE BUILDER SPAN IS DROPPED, not zero-filled — this can shrink the \
         DREAD population below `risk.dread.len()` (see harness verification): a DREAD type all of \
         whose builders are, e.g., in a doc-comment-only stub or unresolved macro expansion contributes \
         no row rather than a misleading zero.\n\
         - `cosmetic_only` is never computed (nor by `cargo antigen mine` itself) — every commit is \
         assumed non-cosmetic.\n\
         - Merge commits contribute no line-touches under plain `git log -p` (no `-m`/`--cc`).\n\
         - Single corpus per run — cross-corpus replication is the point of running this on ripgrep \
         and zellij, not a single antigen-only claim.\n\
         - COMPLEMENTARY, NOT A REPLACEMENT: this tests whether risk propagates to a DREAD type's \
         BUILDERS; it says nothing about the type's own declaration span (that is v2's question) — \
         the two can legitimately disagree."
    );
}

// ── Main (orchestration only) ────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn main() {
    let (scip_path, repo_root) = parse_args();
    println!(
        "== founding-bet v3: PROPAGATION-slice defect-concentration (DREAD types' BUILDERS vs. real \
         SZZ history) =="
    );
    println!(
        "Hypothesis: real SZZ-mined defect LINES concentrate inside the constructor/builder \
         FUNCTIONS of a DREAD (risky-to-change) type — not the type's own declaration (v2's slice) — \
         above the base rate, WITH both the size confound and the centrality confound controlled \
         for.\n"
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
    println!("Construct edges total: {}", risk.construct_edges.len());

    let spans = parse_scip_spans(&scip_path);
    println!(
        "body-spans (any definition w/ enclosing_range): {} symbols ({} fn-shaped, {} non-fn/type-shaped)",
        spans.span_of.len(),
        spans.fn_spans,
        spans.non_fn_spans
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
        "fix-touched lines: {fix_mapped} mapped to a containing symbol, {fix_unmapped} unmappable"
    );
    println!(
        "all-touched lines: {all_mapped} mapped to a containing symbol, {all_unmapped} unmappable"
    );

    let (rows, diag) = build_builder_rows(
        &risk.dread,
        &risk.construct_edges,
        &spans.span_of,
        &fix_by_symbol,
        &all_by_symbol,
    );
    print_diagnostics(&diag);

    let risky_n = rows.iter().filter(|r| r.is_dread).count();
    println!(
        "\nanalysis universe: {} constructed-type rows (builder-union), {risky_n} of them DREAD",
        rows.len()
    );

    println!("\n== Control 1 (SIZE): fix-density = fix_lines / span_size (builder-union) ==");
    let density = compare(&rows, |_| true, |r| r.fix_lines / r.span_size);
    print_comparison(&density);

    println!(
        "\n== Control 2 (CENTRALITY, the load-bearing test): fix-fraction = fix_lines / all_lines \
         (builder-union) =="
    );
    let fraction = compare(&rows, |r| r.all_lines > 0.0, |r| r.fix_lines / r.all_lines);
    print_comparison(&fraction);

    let risky_indicator: Vec<f64> = rows
        .iter()
        .map(|r| f64::from(u8::from(r.is_dread)))
        .collect();
    let span_sizes: Vec<f64> = rows.iter().map(|r| r.span_size).collect();
    let all_lines_vals: Vec<f64> = rows.iter().map(|r| r.all_lines).collect();
    println!("\n== Collinearity (is DREAD just re-measuring builder-union size or churn?) ==");
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

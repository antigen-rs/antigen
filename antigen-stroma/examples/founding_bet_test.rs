//! The founding-bet defect-concentration test — antigen's first measurement against REAL defect
//! history, not a structural proxy.
//!
//! **The bet.** antigen's founding bet: failure-classes concentrate at structural
//! DISCONTINUITIES/hubs, not uniformly across a codebase. `light_the_wing` measured the
//! blast-radius field is fat-tailed on real code; `singular_locus.rs` tested whether
//! "interesting" structural proxies (constructor-shape, dispatch-shape, module/crate boundaries)
//! concentrate at the field's discontinuities — and closed with: "a better ground truth [than a
//! structural proxy] would be … running the singular locus against SZZ-identified defect sites …
//! the next, much stronger, test of this same hypothesis." This is that test: real defect sites
//! (mined from `.git` via SZZ) vs. structural signals (construct-in-degree, call-blast,
//! singular-locus membership), on antigen's own repo.
//!
//! **Step 0 — what `cargo antigen mine` actually produces (measured, not assumed).**
//! Ran `cargo build --release -p cargo-antigen` then
//! `cargo-antigen antigen mine --root . --format json` on this repo (2026-07-16): **276
//! `(defect, fix)` pairs, every single one with `path == ""`.** Reading
//! `cargo-antigen/src/mine.rs::mine_repo` explains why: the CLI's git walk (`gix`) links each
//! classified fix commit to its **first parent** at COMMIT granularity — `path: String::new()`
//! is hard-coded (mine.rs:120) because the precise per-line blame linker
//! ([`antigen::learn::szz::mine`], the library half, which DOES carry a `path` when fed real
//! `BlamedLine`s) is not yet wired into the CLI walk. So: **the shipped tool's output cannot be
//! joined to anything file-level or symbol-level today** — not a design choice we're working
//! around, a measured gap. (The library's classifier, [`antigen::learn::szz::is_fix_commit`], is
//! exactly what we need though — reused below rather than reimplemented, so the fix-commit
//! definition here is IDENTICAL to `cargo antigen mine`'s, not a parallel heuristic.)
//!
//! **The join problem, stated honestly.** Structural signals here are PER-SYMBOL
//! (`AuditHint` constructed 395×, per `light_the_wing`). SZZ (as shipped) gives commit-level
//! `(defect, fix)` pairs with no path. A true symbol-level join needs a source-location→symbol
//! resolver that does not exist yet (the "H1" gap noted in team discussion: stroma nodes are SCIP
//! `FqPath` symbols; a fix's diff hunks are `file:line` ranges — turning a `file:line` into "which
//! symbol's body contains this line" is unbuilt as a reusable primitive, though `ingest_scip`'s own
//! `enclosing_range` containment logic — see its module doc — is the closest existing machinery
//! and would be the natural base for it).
//!
//! So **this test runs at FILE granularity**, on two independently-mined per-file counts:
//! 1. Structural signals, aggregated per file by mapping each graph SYMBOL to its DEFINING file —
//!    SCIP's own `Document.relative_path`, read directly from the raw index here (neither
//!    `ingest_scip` nor `ingest_scip_symbols` carry a file field today; this module parses the
//!    same `.scip` protobuf a second time, purely for this join, rather than growing either
//!    production function's contract for a one-off measurement).
//! 2. SZZ fix-history, aggregated per file by re-deriving file-linkage `cargo antigen mine` doesn't
//!    yet produce: `git log --all --name-only` (the same full-object-graph seed `mine_repo` uses),
//!    each commit classified by the REUSED `is_fix_commit`, each fix's changed-file list attributed
//!    a fix-count. (A merge commit's diff is not shown by plain `git log --name-only`; 33 of 1041
//!    commits on this repo are merges — an honest, small coverage gap, the same kind of "abandon
//!    a case rather than fake it" honesty `mine_repo` itself practices for root fix commits.)
//!
//! **The size confound — the control this project has been bitten by omitting before.** A
//! big/central file trivially accumulates more of EVERYTHING: more symbols, more edges, more
//! commits touching it, more of any signal. So every signal below is reported THREE ways:
//! - **RAW** — top-quartile-by-raw-signal files' mean fix-count ÷ the all-files mean fix-count.
//! - **SIZE-ONLY baseline** — the identical computation ranking by `symbol_count` (file size)
//!   instead of the signal. If a structural signal's RAW enrichment merely matches this baseline,
//!   the signal is riding on size, not contributing anything structural.
//! - **NORMALIZED (size-controlled)** — rank by signal DENSITY (`raw / symbol_count`), measure FIX
//!   DENSITY (`fix_count / symbol_count`) as the outcome. Dividing both sides by file size removes
//!   the confound from both the ranking and the outcome at once — this is the number that answers
//!   "does the STRUCTURE matter, independent of how big the file is".
//!
//! A Pearson correlation between each raw signal and `symbol_count` is also reported — a direct
//! collinearity readout, independent of the quartile-enrichment numbers.
//!
//! Usage: `cargo run -p antigen-stroma --example founding_bet_test -- [path/to.scip] [repo_root]`
//! (defaults: `target/antigen.scip`, `.`; the `.scip` is regenerated via `rust-analyzer scip .`
//! only if missing).

// A stats-display example: usize/i32→f64 precision loss on graph/commit counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use antigen::learn::szz::{CommitMeta, is_fix_commit};
use antigen_stroma::base::facts::{EdgeFact, EdgeKind};
use antigen_stroma::db::StromaDb;
use antigen_stroma::engine::ReachabilityRelation;
use antigen_stroma::read::query::blast_from;
use antigen_stroma::read::{DetectionGrade, SnapshotHandle};
use antigen_stroma::scip::{ingest_scip, populate_from_reconstructions};

// ── SCIP: symbol → defining file (the join key neither production function carries) ───────────

/// Normalize a path to forward slashes — `rust-analyzer scip` and `git` both emit repo-relative
/// paths, but a Windows `rust-analyzer` run can emit backslashes; without this the join silently
/// finds nothing (every "file" a distinct key on each side) rather than erroring, so it is applied
/// unconditionally rather than only where a backslash was observed.
fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

/// Parse the raw SCIP protobuf a second time (independent of [`ingest_scip`]/
/// [`antigen_stroma::scip::ingest_scip_symbols`], neither of which carries a file field) to build
/// `SCIP symbol string → defining file`, from every `Document`'s `relative_path` + its `Definition`
/// occurrences. A symbol defined in more than one document (rare; e.g. a generic instantiated in
/// two compilation units) keeps its FIRST-seen file — harmless for this aggregate analysis.
fn symbol_to_file_map(scip_index_path: &Path) -> HashMap<String, String> {
    use protobuf::Message;
    use scip::types::{Index, SymbolRole};

    let mut map = HashMap::new();
    let Ok(bytes) = std::fs::read(scip_index_path) else {
        return map;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return map;
    };
    for doc in &index.documents {
        let file = normalize_path(&doc.relative_path);
        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if is_def && !occ.symbol.is_empty() && !occ.symbol.starts_with("local ") {
                map.entry(occ.symbol.clone())
                    .or_insert_with(|| file.clone());
            }
        }
    }
    map
}

/// Regenerate the SCIP index only if it is missing — never re-derive a fresh one over a present
/// (possibly team-generated) index, so a stale-but-present index is used as-is per the brief.
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

// ── SZZ: per-file fix counts, re-derived (the file-linkage `cargo antigen mine` doesn't produce) ─

/// Mine per-file SZZ fix-counts via a full-object-graph `git log --all --name-only`, classifying
/// each commit with antigen's OWN production classifier ([`is_fix_commit`]) — reused, not
/// reimplemented, so "fix commit" means exactly what `cargo antigen mine` means by it. Returns
/// `(fix_count_by_file, total_fix_commits, total_commits_walked)`.
///
/// Parsing scheme: `--pretty=format:%x01%H%x02%s` prefixes each commit with a SOH/STX-delimited
/// header (bytes unlikely to appear in a commit subject), `--name-only` appends its changed files
/// (blank for a merge commit — git shows no diff for merges by default; 33/1041 commits here are
/// merges, a small honest coverage gap this test does not try to paper over).
fn mine_fix_counts_by_file(repo_root: &Path) -> (HashMap<String, usize>, usize, usize) {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args([
            "log",
            "--all",
            "--name-only",
            "--pretty=format:%x01%H%x02%s",
        ])
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git log --all failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);

    let mut fix_counts: HashMap<String, usize> = HashMap::new();
    let mut total_commits = 0usize;
    let mut total_fix = 0usize;

    for chunk in text.split('\u{1}').filter(|c| !c.trim().is_empty()) {
        let (head, files_block) = chunk.split_once('\n').unwrap_or((chunk, ""));
        let Some((id, subject)) = head.split_once('\u{2}') else {
            continue;
        };
        total_commits += 1;
        let files: Vec<String> = files_block
            .lines()
            .filter(|l| !l.is_empty())
            .map(normalize_path)
            .collect();
        let meta = CommitMeta {
            id: id.to_string(),
            subject: subject.to_string(),
            files_changed: files.clone(),
            cosmetic_only: false,
        };
        if is_fix_commit(&meta) {
            total_fix += 1;
            for f in &files {
                *fix_counts.entry(f.clone()).or_insert(0) += 1;
            }
        }
    }
    (fix_counts, total_fix, total_commits)
}

// ── The stroma graph (identical constitution to `light_the_wing`/`singular_locus`) ─────────────

fn ingest(path: &Path) -> StromaDb {
    let recons = ingest_scip(path);
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    db
}

/// The blast-radius field — one `blast_from` count per node (ported from `singular_locus.rs`'s
/// `blast_field`; duplicated rather than shared because these are independent example binaries
/// with no shared examples-support crate).
fn blast_field(snap: &SnapshotHandle, rel: &ReachabilityRelation) -> Vec<usize> {
    (0..rel.len())
        .map(|i| {
            blast_from(snap, rel.node(i))
                .read_at_least(DetectionGrade::Dread)
                .map_or(0, Vec::len)
        })
        .collect()
}

/// The distinct structural edge set: deduplicated `(src, dst)` index pairs, self-loops dropped
/// (ported from `singular_locus.rs` — a self-loop has zero gradient on every field by construction).
fn distinct_edges(rel: &ReachabilityRelation) -> Vec<(usize, usize)> {
    let mut set = std::collections::BTreeSet::new();
    for u in 0..rel.len() {
        for &(v, _tier) in rel.forward_adj(u) {
            if u != v {
                set.insert((u, v));
            }
        }
    }
    set.into_iter().collect()
}

/// The top-decile cutoff of a `usize` distribution, by RANK (ported from `singular_locus.rs`'s
/// `decile_cutoff`, degenerate-collapse guard included) — used ONLY for the edge-gradient decile
/// that defines "singular locus", distinct from the per-FILE quartile split below.
fn decile_cutoff(values: &[usize]) -> Option<usize> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let raw_cutoff = sorted[(sorted.len() - 1) * 9 / 10];
    if raw_cutoff > 0 {
        return Some(raw_cutoff);
    }
    let mut positive: Vec<usize> = values.iter().copied().filter(|&v| v > 0).collect();
    if positive.is_empty() {
        return None;
    }
    positive.sort_unstable();
    Some(positive[(positive.len() - 1) * 9 / 10])
}

/// Construct-in-degree per SCIP symbol string — the data-model-hub signal (`AuditHint` constructed
/// 395× is exactly this count, keyed by the FULL symbol string so it joins against
/// [`symbol_to_file_map`], unlike `light_the_wing`'s display-only `short()`-keyed version).
fn construct_in_degree_by_symbol(edges: &[EdgeFact]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for e in edges {
        if e.kind == EdgeKind::Construct {
            *map.entry(e.dst.fq_path.path.clone()).or_insert(0) += 1;
        }
    }
    map
}

// ── Per-file rows: the join of structural signals + SZZ ground truth ───────────────────────────

/// One file's structural signals + its SZZ ground truth — the analysis unit. Every field a
/// **measured aggregate**, never a guess: `symbol_count` counts graph nodes whose defining file
/// this is; `construct_indeg`/`call_blast`/`singular_touch` sum/count over exactly those symbols;
/// `fix_count` is the independently-mined SZZ count for this same path string.
struct FileRow {
    construct_indeg: usize,
    call_blast: usize,
    singular_touch: usize,
    symbol_count: usize,
    fix_count: usize,
}

impl FileRow {
    fn fix_density(&self) -> f64 {
        self.fix_count as f64 / self.symbol_count.max(1) as f64
    }
}

/// Build one [`FileRow`] per file that defines at least one graph symbol — the analysis universe.
/// A file with SZZ fix-history but NO symbol in the graph (docs, `Cargo.toml`, non-Rust files, or
/// a crate excluded from the `.scip` generation) is out of scope by construction — it cannot carry
/// a structural signal, so it cannot be part of a structure-vs-defect correlation.
#[allow(clippy::too_many_arguments)]
fn build_file_rows(
    rel: &ReachabilityRelation,
    edges: &[EdgeFact],
    blast: &[usize],
    structural_edges: &[(usize, usize)],
    singular_idx: &[usize],
    sym_file: &HashMap<String, String>,
    fix_by_file: &HashMap<String, usize>,
) -> Vec<FileRow> {
    let ctor_indeg = construct_in_degree_by_symbol(edges);

    let file_of = |i: usize| -> Option<&str> {
        sym_file
            .get(rel.node(i).fq_path.path.as_str())
            .map(String::as_str)
    };

    let mut symbol_count: HashMap<&str, usize> = HashMap::new();
    let mut construct_sum: HashMap<&str, usize> = HashMap::new();
    let mut blast_sum: HashMap<&str, usize> = HashMap::new();
    // `i` indexes THREE independent things (`file_of`, `blast`, and `rel.node`), not just one
    // slice — an enumerate()-over-one-slice rewrite would still need the others by index anyway.
    #[allow(clippy::needless_range_loop)]
    for i in 0..rel.len() {
        let Some(f) = file_of(i) else { continue };
        *symbol_count.entry(f).or_insert(0) += 1;
        *blast_sum.entry(f).or_insert(0) += blast[i];
        let sym = rel.node(i).fq_path.path.as_str();
        if let Some(&c) = ctor_indeg.get(sym) {
            *construct_sum.entry(f).or_insert(0) += c;
        }
    }

    let mut singular_touch: HashMap<&str, usize> = HashMap::new();
    for &edge_idx in singular_idx {
        let (u, v) = structural_edges[edge_idx];
        let mut files: Vec<&str> = [file_of(u), file_of(v)].into_iter().flatten().collect();
        files.sort_unstable();
        files.dedup();
        for f in files {
            *singular_touch.entry(f).or_insert(0) += 1;
        }
    }

    symbol_count
        .into_iter()
        .map(|(f, symbol_count)| {
            let fix_count = fix_by_file.get(f).copied().unwrap_or(0);
            FileRow {
                construct_indeg: construct_sum.get(f).copied().unwrap_or(0),
                call_blast: blast_sum.get(f).copied().unwrap_or(0),
                singular_touch: singular_touch.get(f).copied().unwrap_or(0),
                symbol_count,
                fix_count,
            }
        })
        .collect()
}

// ── Statistics: quartile split, enrichment ratio, Pearson r ────────────────────────────────────

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

/// The top-quartile cutoff of an `f64` distribution, by RANK (the value at the 75th-percentile
/// rank — ties at the cutoff are all included). No degenerate-collapse fallback: a cutoff of 0.0
/// here just means "the top quartile is every file with any signal at all", an honest outcome
/// (not a machinery failure the way a zero EDGE-gradient decile would be — see [`decile_cutoff`]).
fn quartile_cutoff(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Some(sorted[(sorted.len() - 1) * 3 / 4])
}

/// One signal's base-vs-top-quartile enrichment: `base_mean` = `outcome`'s mean over ALL rows;
/// `top_mean` = `outcome`'s mean over the rows in the top quartile of `rank_by`.
struct EnrichmentResult {
    base_mean: f64,
    top_mean: f64,
    top_n: usize,
    total_n: usize,
}

impl EnrichmentResult {
    fn ratio(&self) -> Option<f64> {
        (self.base_mean > 0.0).then_some(self.top_mean / self.base_mean)
    }
}

fn enrichment(
    rows: &[FileRow],
    rank_by: impl Fn(&FileRow) -> f64,
    outcome: impl Fn(&FileRow) -> f64,
) -> EnrichmentResult {
    let ranks: Vec<f64> = rows.iter().map(&rank_by).collect();
    let cutoff = quartile_cutoff(&ranks);
    let base_vals: Vec<f64> = rows.iter().map(&outcome).collect();
    let base_mean = mean(&base_vals);

    let top_vals: Vec<f64> = cutoff.map_or_else(Vec::new, |c| {
        rows.iter()
            .zip(&ranks)
            .filter(|&(_, &r)| r >= c)
            .map(|(row, _)| outcome(row))
            .collect()
    });
    let top_n = top_vals.len();
    EnrichmentResult {
        base_mean,
        top_mean: mean(&top_vals),
        top_n,
        total_n: rows.len(),
    }
}

/// Pearson correlation coefficient — `None` when either series has zero variance (undefined, not
/// zero: a constant series correlates with nothing, including itself, and reporting 0.0 would read
/// as "no relationship" rather than "not computable").
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

// ── Reporting ───────────────────────────────────────────────────────────────────────────────────

fn print_enrichment_line(label: &str, e: &EnrichmentResult) {
    match e.ratio() {
        Some(r) => println!(
            "  {label:<42} base={:.3}  top-quartile(n={}/{})={:.3}  enrichment ×{r:.2}",
            e.base_mean, e.top_n, e.total_n, e.top_mean
        ),
        None => println!("  {label:<42} base rate is zero — cannot compute a ratio"),
    }
}

/// Report one structural signal's enrichment RAW, NORMALIZED (size-controlled), and the
/// raw-signal↔file-size Pearson r, then a verdict per the module doc's three-way test.
fn report_signal(
    name: &str,
    rows: &[FileRow],
    raw: impl Fn(&FileRow) -> f64 + Copy,
    density: impl Fn(&FileRow) -> f64 + Copy,
) {
    let raw_result = enrichment(rows, raw, |r| r.fix_count as f64);
    let normalized = enrichment(rows, density, FileRow::fix_density);

    let sizes: Vec<f64> = rows.iter().map(|r| r.symbol_count as f64).collect();
    let raw_vals: Vec<f64> = rows.iter().map(raw).collect();
    let r = pearson(&raw_vals, &sizes);

    println!("\n-- {name} --");
    println!(
        "  size↔signal Pearson r (raw signal vs. symbol_count): {}",
        r.map_or_else(|| "n/a".to_string(), |v| format!("{v:.2}"))
    );
    print_enrichment_line("RAW (uncontrolled)", &raw_result);
    print_enrichment_line("NORMALIZED (density-ranked, density-outcome)", &normalized);

    let verdict = match (raw_result.ratio(), normalized.ratio()) {
        (_, Some(nr)) if nr >= 1.2 => {
            format!("REAL enrichment, survives size-control (×{nr:.2} normalized).")
        },
        (Some(rr), Some(nr)) if rr >= 1.2 => format!(
            "SIZE-CONFOUND NULL: raw enrichment ×{rr:.2} COLLAPSES to ×{nr:.2} once file size is \
             controlled for — this signal was riding on file size, not structure."
        ),
        _ => "NULL: no meaningful enrichment, controlled or not.".to_string(),
    };
    println!("  verdict: {verdict}");
}

// ── Main (orchestration only — each stage's own print is a named helper, keeping `main` short) ──

/// `(scip_path, repo_root)` from argv, with the documented defaults.
fn parse_args() -> (std::path::PathBuf, std::path::PathBuf) {
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

/// Everything downstream needs from the constituted stroma graph — bundled so `main` doesn't
/// re-thread five separate values.
struct Graph {
    rel: ReachabilityRelation,
    edges: Vec<EdgeFact>,
    blast: Vec<usize>,
    structural_edges: Vec<(usize, usize)>,
    singular_idx: Vec<usize>,
}

/// Constitute the stroma from `scip_path` (identical to `light_the_wing`/`singular_locus`), then
/// compute the blast field + the singular-locus edge set on it. `None` (having printed why) for an
/// empty graph — nothing downstream can run.
fn build_graph(scip_path: &Path) -> Option<Graph> {
    let db = ingest(scip_path);
    let snap = SnapshotHandle::open(&db);
    let edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    let rel = ReachabilityRelation::from_edges(&edges);
    if rel.is_empty() {
        println!("(empty graph — check the .scip path / that it generated)");
        return None;
    }
    println!(
        "stroma constituted: {} nodes, {} edges",
        rel.len(),
        edges.len()
    );

    let blast = blast_field(&snap, &rel);
    let structural_edges = distinct_edges(&rel);
    let grad: Vec<usize> = structural_edges
        .iter()
        .map(|&(u, v)| blast[u].abs_diff(blast[v]))
        .collect();
    let cutoff = decile_cutoff(&grad);
    let singular_idx: Vec<usize> = cutoff.map_or_else(Vec::new, |c| {
        (0..grad.len()).filter(|&i| grad[i] >= c).collect()
    });
    println!(
        "singular locus: {}/{} structural edges at/above the top-decile blast-gradient",
        singular_idx.len(),
        structural_edges.len()
    );
    Some(Graph {
        rel,
        edges,
        blast,
        structural_edges,
        singular_idx,
    })
}

/// Print the SZZ re-derivation's headline count + the cross-check against the shipped tool's
/// separately-run JSON output (the Step-0 finding, stated once in the module doc; re-stated here
/// as a live number so a re-run's `total_fix` can be visually compared against it).
fn print_szz_summary(total_fix: usize, total_commits: usize) {
    println!(
        "\nSZZ (re-derived, full-object-graph `git log --all`, {total_commits} commits walked): \
         {total_fix} fix-classified commits (`cargo antigen mine`'s shipped JSON reported 276 \
         commit-granularity pairs on this repo on 2026-07-16, ALL with path==\"\" — the two counts \
         are expected to match modulo root-fix commits `mine_repo` excludes for having no parent)."
    );
}

/// Print the analysis-universe coverage: how many files carry a structural signal, how many of
/// those were also touched by a mined fix — the honest denominator every enrichment number below
/// is computed against.
fn print_universe_summary(rows: &[FileRow], total_fix: usize) {
    let files_with_any_fix = rows.iter().filter(|r| r.fix_count > 0).count();
    let total_fixes_in_universe: usize = rows.iter().map(|r| r.fix_count).sum();
    println!(
        "analysis universe: {} files define ≥1 graph symbol; {files_with_any_fix} of them were \
         touched by ≥1 SZZ fix commit ({total_fixes_in_universe} fix-touches total; the gap to \
         {total_fix} fix commits is fixes that touched only files outside this universe — docs, \
         `Cargo.toml`, non-indexed crates, or merge commits `git log --name-only` shows no diff \
         for).",
        rows.len()
    );
}

/// The closing honest-caveats block — printed once, verbatim, regardless of what the numbers
/// above came out as (a real/null verdict does not change what limits it).
fn print_caveats(node_count: usize) {
    println!(
        "\n== Honest caveats ==\n\
         - Granularity is FILE-level, not symbol-level: the H1 gap (a `file:line` → SCIP-symbol \
         resolver) does not exist yet. A symbol-level test needs that resolver PLUS `cargo antigen \
         mine`'s CLI walk wired to real per-line blame (the library classifier already carries a \
         `path` when fed `BlamedLine`s — only the CLI-side blame plumbing is missing).\n\
         - Single corpus, single repo (antigen's own ~{node_count} node graph) — no cross-project \
         replication.\n\
         - `cosmetic_only` is never computed here (nor by `cargo antigen mine` itself) — every \
         commit is assumed non-cosmetic; a whitespace-only \"fix:\" commit would be misclassified \
         the same way in both this test and the shipped tool.\n\
         - Merge commits (33/1041 here) contribute no file-linkage under plain `git log \
         --name-only` — an honest, uncorrected gap, not silently patched over."
    );
}

fn main() {
    let (scip_path, repo_root) = parse_args();

    println!("== The founding-bet defect-concentration test ==");
    println!(
        "Hypothesis: real SZZ-mined defect sites concentrate at structural signals \
         (construct-in-degree / call-blast / singular-locus membership), above the base rate — \
         WITH the size confound controlled for.\n"
    );

    ensure_scip_index(&scip_path, &repo_root);

    let Some(graph) = build_graph(&scip_path) else {
        return;
    };
    let sym_file = symbol_to_file_map(&scip_path);

    let (fix_by_file, total_fix, total_commits) = mine_fix_counts_by_file(&repo_root);
    print_szz_summary(total_fix, total_commits);

    let rows = build_file_rows(
        &graph.rel,
        &graph.edges,
        &graph.blast,
        &graph.structural_edges,
        &graph.singular_idx,
        &sym_file,
        &fix_by_file,
    );
    print_universe_summary(&rows, total_fix);

    // The size-only baseline — how much enrichment does file SIZE ALONE produce.
    let size_only = enrichment(&rows, |r| r.symbol_count as f64, |r| r.fix_count as f64);
    println!("\n== Size-only baseline (rank purely by symbol_count, no structural signal) ==");
    print_enrichment_line("SIZE-ONLY", &size_only);
    if let Some(r) = size_only.ratio() {
        println!(
            "  (any structural signal below whose RAW enrichment is close to ×{r:.2} is likely \
             just re-measuring file size)"
        );
    }

    // The three structural signals, each RAW + size-NORMALIZED + a verdict.
    println!("\n== Per-signal enrichment (RAW vs. size-NORMALIZED) ==");
    report_signal(
        "construct-in-degree (the data-model-hub signal)",
        &rows,
        |r| r.construct_indeg as f64,
        |r| r.construct_indeg as f64 / r.symbol_count.max(1) as f64,
    );
    report_signal(
        "call-blast (sum of blast_from over symbols in the file)",
        &rows,
        |r| r.call_blast as f64,
        |r| r.call_blast as f64 / r.symbol_count.max(1) as f64,
    );
    report_signal(
        "singular-locus membership (top-decile blast-gradient edges touching the file)",
        &rows,
        |r| r.singular_touch as f64,
        |r| r.singular_touch as f64 / r.symbol_count.max(1) as f64,
    );

    print_caveats(graph.rel.len());
}

//! Sheaf cohomology of the two-instrument stroma on antigen's own call graph — the sparse measurement.
//!
//! nb 028 ratified that the stroma is a **sheaf**: H⁰ = the assignments every instrument agrees with,
//! H¹ = the edge-data no vertex-assignment can explain. This example computes both on the real graph.
//!
//! The construction measured here:
//! - base: UNDIRECTED graph, vertices = fn-def symbols, edges = resolved `EdgeKind::Call` edges, deduped,
//!   self-loops dropped;
//! - `F(v) = ℝ²` = `(syn_confidence, scip_confidence)`, `F(e) = ℝ`;
//! - restriction: BOTH endpoints of an edge use the SAME 1×2 row `R_e` — `[0.5, 0.5]` when both instruments
//!   assert the edge, `[1.0, 0.0]` when only syn does, `[0.0, 1.0]` when only SCIP does.
//!
//! The dense coboundary is `(|E|·d_e) × (|V|·d_v)` f64 — on this graph roughly a gigabyte — while each row
//! holds only `2·d_v = 4` nonzeros. Hence the sparse path, and hence the component split (`δ⁰` is
//! block-diagonal over components, so ranks and dimensions ADD).
//!
//! Two edge sets are measured, because the restriction rule anticipates a syn-only case that the SCIP-only
//! base graph cannot contain:
//! 1. **base** — the resolved SCIP Call edges exactly (syn-only count is structurally 0 here);
//! 2. **union** — base plus the syn-only pairs whose two bare names each resolve to exactly ONE antigen
//!    symbol (the rough-set *lower* residue; a colliding bare name cannot be lifted to a vertex pair).
//!
//! Rank over floats is a THRESHOLDING DECISION, so every dimension is reported with its tolerance and the
//! run repeats at three tolerances.
//!
//! Usage: `cargo run --release -p antigen-stroma --example sheaf_conflict -- [path/to.scip]`

// Graph dimensions here are ~10⁴ — far from i64 wrap or f64 mantissa limits.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use antigen_stroma::sheaf::Sheaf;
use antigen_stroma::syn_ingest::{bare_fn_name, syn_call_pairs_from_dir};
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const CRATE_DIRS: &[&str] = &[
    "antigen/src",
    "antigen-stroma/src",
    "antigen-fingerprint/src",
    "antigen-macros/src",
    "antigen-attestation/src",
    "cargo-antigen/src",
];

/// Swept wide on purpose. If the rank never moves across twelve orders of magnitude, "tolerance-insensitive"
/// is a MEASURED property of this construction, not an assumption — and the last entries probe where it
/// finally does move, which is the only way to show the thresholding path is live rather than inert.
const TOLERANCES: &[f64] = &[1e-12, 1e-9, 1e-7, 1e-3, 1e-1, 0.4, 0.6];

/// Which instruments assert an edge — this is exactly what picks the restriction row.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Asserted {
    Both,
    SynOnly,
    ScipOnly,
}

impl Asserted {
    /// The 1×2 row `R_e`, used on BOTH endpoints of the edge.
    const fn row(self) -> [f64; 2] {
        match self {
            Self::Both => [0.5, 0.5],
            Self::SynOnly => [1.0, 0.0],
            Self::ScipOnly => [0.0, 1.0],
        }
    }
}

/// `bare_name -> { full SCIP symbols }` over every callable DEFINITION (the syn indiscernibility classes).
fn fn_classes(scip_path: &Path) -> HashMap<String, HashSet<String>> {
    let mut classes: HashMap<String, HashSet<String>> = HashMap::new();
    let Ok(bytes) = std::fs::read(scip_path) else {
        return classes;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return classes;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) != 0 {
                if let Some(name) = bare_fn_name(&occ.symbol) {
                    classes
                        .entry(name.to_string())
                        .or_default()
                        .insert(occ.symbol.clone());
                }
            }
        }
    }
    classes
}

/// Assemble the sheaf from a symbol-level undirected edge list plus each edge's instrument set.
fn build_sheaf(edges: &[((String, String), Asserted)]) -> Sheaf {
    fn idx(index: &mut HashMap<String, usize>, s: &str) -> usize {
        let n = index.len();
        *index.entry(s.to_string()).or_insert(n)
    }
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut pairs = Vec::with_capacity(edges.len());
    let mut restrictions = Vec::with_capacity(edges.len());
    for ((u, v), a) in edges {
        let (iu, iv) = (idx(&mut index, u), idx(&mut index, v));
        pairs.push((iu, iv));
        let r = a.row().to_vec();
        restrictions.push((r.clone(), r));
    }
    Sheaf::new(index.len(), pairs, 2, 1, restrictions)
}

/// Cross-check the sparse path against the dense [`rank`] on a REAL subgraph.
///
/// The unit tests do this on toy sheaves; this does it on the actual measured construction, at the largest
/// size the dense matrix still fits. Disagreement here means one implementation is wrong, which outranks
/// every number this example prints.
fn cross_check_against_dense(edges: &[((String, String), Asserted)], n_edges: usize, tol: f64) {
    let sub: Vec<((String, String), Asserted)> = edges.iter().take(n_edges).cloned().collect();
    let sheaf = build_sheaf(&sub);
    let dense = antigen_stroma::sheaf::rank(&mut sheaf.coboundary(), tol);
    let sparse_flat = antigen_stroma::sheaf::sparse_rank(&sheaf.sparse_coboundary(), tol).rank;
    let sparse_split = sheaf.sparse_rank_total(tol).rank;
    assert_eq!(
        dense, sparse_flat,
        "sparse rank disagrees with dense on the real subgraph"
    );
    assert_eq!(
        dense, sparse_split,
        "component-split rank disagrees with dense on the real subgraph"
    );
    println!(
        "\ncross-check on a REAL subgraph ({} edges, {} vertices, dense {}x{}): dense = sparse = sparse-split = {dense}  [tol {tol:.0e}]",
        sheaf.edge_count(),
        sheaf.node_count(),
        sheaf.edge_count(),
        2 * sheaf.node_count()
    );
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

/// Reconcile this example's edge count with `hodge_betti`'s, so the two numbers are never read as competing
/// measurements of one quantity.
///
/// `hodge_betti` dedups ORIENTED edges and keeps self-loops; this example dedups UNDIRECTED simple edges and
/// drops self-loops. Same vertices, same components — the whole difference is those two choices, and the gap
/// in `β₁` must equal the gap in `|E|` exactly, because `β₁ = E − V + C`.
fn reconcile_with_hodge_betti(recons: &[EdgeReconstruction], undirected_simple: usize) {
    let mut oriented: HashSet<(&str, &str)> = HashSet::new();
    for r in recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                oriented.insert((e.src_symbol.as_str(), e.dst_symbol.as_str()));
            }
        }
    }
    let loops = oriented.iter().filter(|(a, b)| a == b).count();
    let mutual = oriented
        .iter()
        .filter(|(a, b)| a != b && oriented.contains(&(*b, *a)))
        .count()
        / 2;
    println!(
        "  reconciliation with hodge_betti: oriented-deduped {} = undirected-simple {undirected_simple} + self-loops {loops} + reciprocal back-edges {mutual}  (delta {})",
        oriented.len(),
        oriented.len() - undirected_simple
    );
    assert_eq!(
        oriented.len(),
        undirected_simple + loops + mutual,
        "the two edge counts must reconcile exactly"
    );
}

/// Degenerate baselines: re-label EVERY edge with one single restriction row, keeping the graph identical.
///
/// This is the negative control for the whole construction. If a heterogeneous instrument-labelling produced
/// the same rank as a uniform one, then distinguishing the instruments contributes nothing and the measured
/// H⁰/H¹ carry no information beyond the base graph's topology. Reported as MEASURED ranks, not derived ones.
fn degenerate_baselines(edges: &[((String, String), Asserted)], tol: f64) {
    println!("\n  DEGENERATE BASELINES (same graph, every edge forced to ONE restriction row):");
    for (name, forced) in [
        ("all [0.5, 0.5] (both)", Asserted::Both),
        ("all [1.0, 0.0] (syn)", Asserted::SynOnly),
        ("all [0.0, 1.0] (SCIP)", Asserted::ScipOnly),
    ] {
        let relabelled: Vec<((String, String), Asserted)> =
            edges.iter().map(|(p, _)| (p.clone(), forced)).collect();
        let sheaf = build_sheaf(&relabelled);
        let r = sheaf.sparse_rank_total(tol).rank;
        let (v, e) = (sheaf.node_count(), sheaf.edge_count());
        println!(
            "    {name:<24} rank {r:5}   dim H^0 {:5}   dim H^1 {:6}",
            2 * v - r,
            e - r
        );
    }
}

/// Measure one edge set end to end: census, ranks at each tolerance, Euler self-check, cost.
/// How much of the `Both` census is real corroboration versus bare-name coincidence?
///
/// syn matches call pairs by BARE NAME, so it cannot tell two same-named symbols apart. An edge labelled
/// `Both` because syn "agrees" is only a verified same-symbol match when neither endpoint's bare name is
/// shared with another antigen symbol. Where it is shared, syn's corroboration is a COINCIDENCE of names,
/// not evidence about that edge — so the census must never be narrated as plain "instrument agreement"
/// without this qualifier. Changes no rank/H⁰/H¹ number; it changes what the census MEANS.
fn both_edges_ambiguity(
    edges: &[((String, String), Asserted)],
    classes: &HashMap<String, HashSet<String>>,
) {
    let colliding =
        |sym: &str| bare_fn_name(sym).is_some_and(|b| classes.get(b).is_some_and(|c| c.len() >= 2));
    let (mut ambiguous, mut both_total) = (0usize, 0usize);
    for ((u, v), a) in edges {
        if *a == Asserted::Both {
            both_total += 1;
            if colliding(u) || colliding(v) {
                ambiguous += 1;
            }
        }
    }
    if both_total == 0 {
        return;
    }
    println!(
        "    ^ AMBIGUITY CAVEAT: {ambiguous} of {both_total} `Both` edges ({:.1}%) have >=1 ENDPOINT whose bare\n\
         \x20     name is shared by >=2 callable DEFINITIONS in this index -- so syn's corroboration there is a\n\
         \x20     name coincidence, not a verified same-symbol match. Only {} ({:.1}%) are unambiguous.\n\
         \x20     (Counted PER EDGE over DEFINITION-symbols; a per-endpoint count, or a universe including\n\
         \x20     non-definition occurrences, gives a materially different figure -- state the convention\n\
         \x20     whenever this number is quoted. Changes no rank/H0/H1 value; it changes what the census MEANS.)",
        pct(ambiguous, both_total),
        both_total - ambiguous,
        pct(both_total - ambiguous, both_total)
    );
}

fn measure(
    label: &str,
    edges: &[((String, String), Asserted)],
    classes: &HashMap<String, HashSet<String>>,
) {
    let (mut both, mut syn_only, mut scip_only) = (0usize, 0usize, 0usize);
    for (_, a) in edges {
        match a {
            Asserted::Both => both += 1,
            Asserted::SynOnly => syn_only += 1,
            Asserted::ScipOnly => scip_only += 1,
        }
    }

    let t0 = Instant::now();
    let sheaf = build_sheaf(edges);
    let build_ms = t0.elapsed().as_secs_f64() * 1e3;

    let v = sheaf.node_count();
    let e = sheaf.edge_count();
    let comps = sheaf.components();
    let n_comp = comps.iter().copied().max().map_or(0, |m| m + 1);
    let mut comp_sizes = vec![0usize; n_comp];
    for &c in &comps {
        comp_sizes[c] += 1;
    }
    comp_sizes.sort_unstable();

    println!("\n================ {label} ================");
    println!("  |V| (fn-def symbols)      = {v}");
    println!("  |E| (undirected, deduped) = {e}");
    println!(
        "  components                = {n_comp}   (largest {}, 2nd {})",
        comp_sizes.last().copied().unwrap_or(0),
        comp_sizes.iter().rev().nth(1).copied().unwrap_or(0)
    );
    println!("\n  EDGE-TYPE CENSUS (which instruments assert the edge → which restriction row):");
    println!(
        "    both syn AND SCIP  [0.5, 0.5] : {both:6}  ({:.1}%)",
        pct(both, e)
    );
    println!(
        "    syn ONLY           [1.0, 0.0] : {syn_only:6}  ({:.1}%)",
        pct(syn_only, e)
    );
    println!(
        "    SCIP ONLY          [0.0, 1.0] : {scip_only:6}  ({:.1}%)",
        pct(scip_only, e)
    );
    both_edges_ambiguity(edges, classes);

    let dense_bytes = (e * 2 * v * 8) as f64;
    println!(
        "\n  dense delta^0 would be {e} x {} f64 = {:.2} GB  (not materialized)",
        2 * v,
        dense_bytes / 1e9
    );

    println!("\n  SPARSE COHOMOLOGY (component-split; rank and dims ADD over components):");
    println!(
        "    {:<10} {:>8} {:>10} {:>10} {:>7} {:>10} {:>9}",
        "tol", "rank", "dim H^0", "dim H^1", "Euler", "fill", "wall"
    );
    for &tol in TOLERANCES {
        let t = Instant::now();
        let rep = sheaf.sparse_rank_total(tol);
        let secs = t.elapsed().as_secs_f64();
        let (h0, h1) = (2 * v - rep.rank, e - rep.rank);
        let euler_ok = h0 as i64 - h1 as i64 == 2 * v as i64 - e as i64;
        assert!(euler_ok, "Euler identity must hold: it is free and exact");
        println!(
            "    {tol:<10.0e} {:>8} {:>10} {:>10} {:>7} {:>9.2}x {:>8.2}s",
            rep.rank,
            h0,
            h1,
            "ok",
            rep.fill_factor(),
            secs
        );
        if (tol - 1e-9).abs() < f64::EPSILON {
            println!(
                "      [tol=1e-9 detail] input nnz {} -> peak retained nnz {} (~{:.1} MB), widest pivot row {}",
                rep.input_nnz,
                rep.peak_nnz,
                rep.peak_nnz as f64 * 16.0 / 1e6,
                rep.widest_pivot_row
            );
        }
    }
    degenerate_baselines(edges, 1e-9);
    println!("  (sheaf assembly {build_ms:.0} ms)");
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let scip_path = Path::new(&scip_path_s);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent (the workspace root)");

    println!("== Sheaf cohomology of the two-instrument stroma on antigen's call graph ==");

    let recons = ingest_scip(scip_path);
    if recons.is_empty() {
        println!("(no SCIP reconstructions from {scip_path_s})");
        return;
    }

    // --- SCIP side: undirected, deduped, self-loops dropped ---
    let mut scip_undirected: HashSet<(String, String)> = HashSet::new();
    let mut self_loops = 0usize;
    let mut directed = 0usize;
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                directed += 1;
                if e.src_symbol == e.dst_symbol {
                    self_loops += 1;
                    continue;
                }
                let (a, b) = if e.src_symbol <= e.dst_symbol {
                    (e.src_symbol.clone(), e.dst_symbol.clone())
                } else {
                    (e.dst_symbol.clone(), e.src_symbol.clone())
                };
                scip_undirected.insert((a, b));
            }
        }
    }
    println!(
        "\nresolved directed Call edges {directed}  ->  self-loops dropped {self_loops}  ->  undirected simple {}",
        scip_undirected.len()
    );
    reconcile_with_hodge_betti(&recons, scip_undirected.len());

    // --- syn side: bare-name call pairs ---
    let src_roots: Vec<PathBuf> = CRATE_DIRS.iter().map(|d| workspace_root.join(d)).collect();
    let syn_pairs = syn_call_pairs_from_dir(&src_roots);
    println!("syn bare-name call pairs      {}", syn_pairs.len());

    // An undirected SCIP edge is syn-asserted if syn saw the bare pair in EITHER orientation.
    let syn_asserts = |u: &str, v: &str| -> bool {
        let (Some(bu), Some(bv)) = (bare_fn_name(u), bare_fn_name(v)) else {
            return false;
        };
        syn_pairs.contains(&(bu.to_string(), bv.to_string()))
            || syn_pairs.contains(&(bv.to_string(), bu.to_string()))
    };

    // Sorted, so the vertex numbering, the cross-check subgraphs, and every printed number are reproducible.
    let mut scip_sorted: Vec<&(String, String)> = scip_undirected.iter().collect();
    scip_sorted.sort_unstable();
    // ⚠ READ THIS BEFORE INTERPRETING THE CENSUS. This loop iterates over SCIP-RESOLVED edges, so every
    // edge here is SCIP-asserted BY CONSTRUCTION and `Asserted::SynOnly` is UNREACHABLE in the base set.
    // The printed "syn-only = 0 (0.0%)" is therefore a fact about THIS CODE PATH, not about antigen's
    // instrument coverage. (It was briefly mis-reported as a finding; caught by adversarial review.)
    // Edge set 2 (the union) is the only one where all three labels can occur — read the census there.
    let base: Vec<((String, String), Asserted)> = scip_sorted
        .iter()
        .map(|(u, v)| {
            let a = if syn_asserts(u, v) {
                Asserted::Both
            } else {
                Asserted::ScipOnly
            };
            ((u.clone(), v.clone()), a)
        })
        .collect();
    for n in [200, 800, 2000] {
        cross_check_against_dense(&base, n, 1e-9);
    }
    // The bare-name indiscernibility classes — needed by the census caveat (how much of `Both` is real
    // corroboration vs name coincidence) as well as by the syn-only lifting further down.
    let classes = fn_classes(scip_path);
    measure(
        "EDGE SET 1: base (resolved SCIP Call edges only)",
        &base,
        &classes,
    );

    let union = extend_with_syn_only(base, scip_path, &scip_undirected, &syn_pairs);
    measure(
        "EDGE SET 2: union (base + liftable syn-only edges)",
        &union,
        &classes,
    );
}

/// Add the syn-only pairs that CAN be lifted to vertex pairs: both bare names must resolve to exactly ONE
/// antigen symbol. A colliding bare name is syn-indiscernible and has no single vertex to attach to.
fn extend_with_syn_only(
    mut union: Vec<((String, String), Asserted)>,
    scip_path: &Path,
    scip_undirected: &HashSet<(String, String)>,
    syn_pairs: &HashSet<(String, String)>,
) -> Vec<((String, String), Asserted)> {
    let classes = fn_classes(scip_path);
    let scip_bare: HashSet<(String, String)> = scip_undirected
        .iter()
        .filter_map(|(u, v)| Some((bare_fn_name(u)?.to_string(), bare_fn_name(v)?.to_string())))
        .collect();
    let mut lifted = 0usize;
    let (mut unliftable_collision, mut unliftable_external) = (0usize, 0usize);
    let mut seen: HashSet<(String, String)> = scip_undirected.clone();
    let mut syn_sorted: Vec<&(String, String)> = syn_pairs.iter().collect();
    syn_sorted.sort_unstable();
    for (su, sv) in syn_sorted {
        if scip_bare.contains(&(su.clone(), sv.clone()))
            || scip_bare.contains(&(sv.clone(), su.clone()))
        {
            continue; // already an edge of the base graph
        }
        let (Some(cu), Some(cv)) = (classes.get(su), classes.get(sv)) else {
            unliftable_external += 1;
            continue;
        };
        if cu.len() != 1 || cv.len() != 1 {
            unliftable_collision += 1;
            continue;
        }
        let (u, v) = (
            cu.iter().next().expect("singleton").clone(),
            cv.iter().next().expect("singleton").clone(),
        );
        if u == v {
            continue; // self-loop
        }
        let key = if u <= v {
            (u.clone(), v.clone())
        } else {
            (v.clone(), u.clone())
        };
        if seen.insert(key.clone()) {
            union.push((key, Asserted::SynOnly));
            lifted += 1;
        }
    }
    println!(
        "\nsyn-only pairs: lifted to vertex pairs {lifted}  |  unliftable: bare-name COLLISION {unliftable_collision}, callee/caller EXTERNAL to antigen {unliftable_external}"
    );
    union
}

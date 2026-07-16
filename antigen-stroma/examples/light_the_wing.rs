//! Light the wing — constitute the stroma from antigen's OWN SCIP graph and run the
//! afferent organs (blast-radius, tier-barcode, min vertex-cut) on real data for the
//! first time. Until now the whole engine has only ever run on hand-built fixtures.
//!
//! Usage: `cargo run -p antigen-stroma --example light_the_wing -- [path/to.scip]`
//! (defaults to `target/antigen.scip`, produced by `rust-analyzer scip . --output …`).

// A stats-display example: usize→f64 precision loss on graph counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]

use std::path::Path;

use antigen_stroma::base::facts::{EdgeFact, EdgeKind};
use antigen_stroma::db::StromaDb;
use antigen_stroma::engine::ReachabilityRelation;
use antigen_stroma::engine::mincut::min_vertex_cut;
use antigen_stroma::engine::path_homology::TierBarcode;
use antigen_stroma::node::StromaNodeId;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::read::query::blast_from;
use antigen_stroma::read::tier::DetectionGrade;
use antigen_stroma::scip::{ingest_scip, populate_from_reconstructions};

/// The tail descriptor of a SCIP symbol (they are long, space-delimited scheme + descriptors).
fn short(id: &StromaNodeId) -> &str {
    let p = id.fq_path.path.as_str();
    p.rsplit(' ').next().unwrap_or(p)
}

/// Print the Call-vs-Construct split, the top-10 most-constructed types, and the node-count delta
/// (split out of `main` to stay under the line budget — the pattern `dual_tier_barcode.rs` uses).
///
/// Every `Construct` edge here is a reference `ingest_scip`'s old `is_fn_symbol`-only gate used to
/// DROP entirely (SCIP's own symbol-kind table is what recovers them; `syn` cannot tell `Some(x)`
/// from a call).
fn print_construct_stats(edges: &[EdgeFact], node_count: usize) {
    let construct_count = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Construct)
        .count();
    let call_count = edges.iter().filter(|e| e.kind == EdgeKind::Call).count();
    println!("== Call vs. Construct (the recovered construction edges) ==");
    println!(
        "  Call:      {call_count:6}\n  Construct: {construct_count:6}  ({:.1}% of all edges)\n  \
         [pre-fill baseline would have been {} edges, Call-only]",
        100.0 * construct_count as f64 / edges.len() as f64,
        edges.len() - construct_count,
    );

    let mut ctor_in_degree: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for e in edges {
        if e.kind == EdgeKind::Construct {
            *ctor_in_degree.entry(short(&e.dst)).or_default() += 1;
        }
    }
    let mut ctor_ranked: Vec<(&str, usize)> = ctor_in_degree.into_iter().collect();
    ctor_ranked.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    println!("\n  Top 10 most-CONSTRUCTED types (highest in-degree over Construct edges):");
    for (sym, n) in ctor_ranked.iter().take(10) {
        println!("    {n:5}  {sym}");
    }

    // Node-count delta: destination nodes reachable ONLY via a Construct edge — the data-model
    // types the pre-fill (Call-only) graph never had a node for at all.
    let non_construct_nodes: std::collections::HashSet<&StromaNodeId> = edges
        .iter()
        .filter(|e| e.kind != EdgeKind::Construct)
        .flat_map(|e| [&e.src, &e.dst])
        .collect();
    let construct_only_nodes: std::collections::HashSet<&StromaNodeId> = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Construct)
        .flat_map(|e| [&e.src, &e.dst])
        .filter(|n| !non_construct_nodes.contains(n))
        .collect();
    println!(
        "\n  new (Construct-only) nodes: {}  of {node_count} total — the rest were already \
         reachable via Call",
        construct_only_nodes.len()
    );
    println!();
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!("== Lighting the wing on {path} ==\n");

    // 1. Ingest the real SCIP graph + constitute the stroma.
    let recons = ingest_scip(Path::new(&path));
    println!("SCIP edge-reconstructions: {}", recons.len());

    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    let snap = SnapshotHandle::open(&db);

    let edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    let rel = ReachabilityRelation::from_edges(&edges);
    println!(
        "stroma constituted: {} nodes, {} edges\n",
        rel.len(),
        edges.len()
    );
    if rel.is_empty() {
        println!("(empty — check the .scip path / that it generated)");
        return;
    }

    // 1b. Construction vs. Call — the EdgeKind::Construct fill.
    print_construct_stats(&edges, rel.len());

    // 2. Blast-radius over every node — the fat-tail shape on real code.
    let mut blasts: Vec<(usize, usize)> = (0..rel.len())
        .map(|i| {
            let node = rel.node(i).clone();
            let n = blast_from(&snap, &node)
                .read_at_least(DetectionGrade::Dread)
                .map_or(0, Vec::len);
            (i, n)
        })
        .collect();
    blasts.sort_by_key(|&(_, n)| std::cmp::Reverse(n));

    println!("== Top 12 blast-radius nodes (# nodes downstream) ==");
    for &(i, n) in blasts.iter().take(12) {
        println!("  {n:5}  {}", short(rel.node(i)));
    }
    let mut counts: Vec<usize> = blasts.iter().map(|&(_, n)| n).collect();
    counts.sort_unstable();
    let max = *counts.last().unwrap();
    let mean = counts.iter().sum::<usize>() as f64 / counts.len() as f64;
    let median = counts[counts.len() / 2];
    let reach_any = counts.iter().filter(|&&n| n > 0).count();
    println!(
        "\n  blast: max={max}  mean={mean:.1}  median={median}  ({reach_any}/{} reach anything)",
        counts.len()
    );
    println!(
        "  → {}\n",
        if max as f64 > 5.0 * mean.max(1.0) {
            "FAT-TAILED (max >> mean) — the tail-not-mean shape, measured on real code"
        } else {
            "roughly flat"
        }
    );

    // 3. Confidence-graded topology — directed H1 cycles per fidelity tier.
    let bc = TierBarcode::from_relation(&rel);
    println!("== Confidence-graded topology (directed H1 cycles per fidelity tier) ==");
    println!(
        "  G3 (proof-tier only):  vertices={:5} edges={:6} H0={:4} H1={}",
        bc.g3.vertices, bc.g3.edges, bc.g3.h0, bc.g3.h1
    );
    println!(
        "  G2 (+Resolved):        vertices={:5} edges={:6} H0={:4} H1={}",
        bc.g2.vertices, bc.g2.edges, bc.g2.h0, bc.g2.h1
    );
    println!(
        "  G1 (all, +Syntactic):  vertices={:5} edges={:6} H0={:4} H1={}",
        bc.g1.vertices, bc.g1.edges, bc.g1.h0, bc.g1.h1
    );
    println!(
        "  → H1 growth G3→G1 = entanglement cycles that exist only through lower-fidelity edges\n"
    );

    // 4. Min vertex-cut — the smallest set of nodes whose removal severs a real path.
    let mut blast_of = vec![0usize; rel.len()];
    for &(i, n) in &blasts {
        blast_of[i] = n;
    }
    let src = blasts[0].0;
    let ds = blast_from(&snap, &rel.node(src).clone());
    if let Some(reached) = ds.read_at_least(DetectionGrade::Dread) {
        // A deep, load-bearing target: the reachable node with its own largest blast.
        if let Some(tgt) = reached
            .iter()
            .filter_map(|id| rel.index_of(id))
            .filter(|&t| t != src)
            .max_by_key(|&t| blast_of[t])
        {
            let cut = min_vertex_cut(&rel, &[src], tgt);
            println!("== Min vertex-cut (fewest nodes whose removal severs src → tgt) ==");
            println!("  source: {}", short(rel.node(src)));
            println!("  target: {}", short(rel.node(tgt)));
            println!("  cut size: {} node(s)", cut.cut_size);
            for &c in cut.cut_nodes.iter().take(8) {
                println!("    · {}", short(rel.node(c)));
            }
            println!();
        }
    }

    println!("== The wing is alive on real data. ==");
}

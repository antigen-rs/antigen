//! Hodge exploration (dep-free) — where do antigen's feedback cycles LIVE?
//!
//! `hodge_betti` (`163cff3`) gave β₁ = 13731 = the harmonic dimension on the bare graph, but that is the
//! trivial leg (the whole cycle space; nb 030 design round). Before the real Hodge build (which needs a
//! sparse-linalg dep for the 2-complex nullspace), this localizes the cyclic structure with pure graph
//! algorithmics: the **strongly-connected components** are where directed feedback lives (a node can reach
//! itself iff it is in a non-trivial SCC). This is the substrate the future harmonic permutation-test targets
//! (adversary design: "harmonic support vs SCC loci beating a random baseline"). No linear algebra, no deps.
//!
//! Usage: `cargo run -p antigen-stroma --example hodge_scc -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};

/// Kosaraju SCC (iterative — no recursion, safe at depth) over a directed graph given as adjacency lists.
fn strongly_connected(n: usize, adj: &[Vec<usize>], radj: &[Vec<usize>]) -> Vec<usize> {
    // Pass 1: post-order (finish) order via iterative DFS.
    let mut visited = vec![false; n];
    let mut order = Vec::with_capacity(n);
    for s in 0..n {
        if visited[s] {
            continue;
        }
        let mut stack = vec![(s, false)];
        while let Some((u, processed)) = stack.pop() {
            if processed {
                order.push(u);
                continue;
            }
            if visited[u] {
                continue;
            }
            visited[u] = true;
            stack.push((u, true));
            for &v in &adj[u] {
                if !visited[v] {
                    stack.push((v, false));
                }
            }
        }
    }
    // Pass 2: assign components in reverse finish order, walking the transpose.
    let mut comp = vec![usize::MAX; n];
    let mut next = 0usize;
    for &s in order.iter().rev() {
        if comp[s] != usize::MAX {
            continue;
        }
        let mut stack = vec![s];
        comp[s] = next;
        while let Some(u) = stack.pop() {
            for &v in &radj[u] {
                if comp[v] == usize::MAX {
                    comp[v] = next;
                    stack.push(v);
                }
            }
        }
        next += 1;
    }
    comp
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!(
        "== Hodge exploration: where do antigen's feedback cycles live? (SCC localization) ==\n"
    );

    let recons = ingest_scip(Path::new(&scip_path_s));
    if recons.is_empty() {
        println!("(no SCIP reconstructions from {scip_path_s})");
        return;
    }

    // Index nodes; build directed adjacency + transpose over resolved Call edges.
    let mut idx: HashMap<&str, usize> = HashMap::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                let n = idx.len();
                let s = *idx.entry(&e.src_symbol).or_insert(n);
                let n = idx.len();
                let d = *idx.entry(&e.dst_symbol).or_insert(n);
                edges.push((s, d));
            }
        }
    }
    let n = idx.len();
    let (mut adj, mut radj) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    for &(s, d) in &edges {
        adj[s].push(d);
        radj[d].push(s);
    }

    let comp = strongly_connected(n, &adj, &radj);
    let mut sizes: HashMap<usize, usize> = HashMap::new();
    for &c in &comp {
        *sizes.entry(c).or_default() += 1;
    }
    let n_scc = sizes.len();
    let mut nontrivial: Vec<usize> = sizes.values().copied().filter(|&s| s >= 2).collect();
    nontrivial.sort_unstable_by(|a, b| b.cmp(a));
    let in_cycles: usize = nontrivial.iter().sum();
    let largest = nontrivial.first().copied().unwrap_or(0);

    println!("  nodes: {n}   Call edges: {}", edges.len());
    println!(
        "  SCCs: {n_scc}   non-trivial (size ≥ 2, i.e. contain feedback): {}",
        nontrivial.len()
    );
    println!(
        "  nodes inside a feedback SCC: {in_cycles}  ({:.1}% of nodes)",
        in_cycles as f64 / n as f64 * 100.0
    );
    println!(
        "  largest SCC: {largest} nodes  ({:.1}% of nodes)",
        largest as f64 / n as f64 * 100.0
    );
    println!(
        "  next-largest non-trivial SCCs: {:?}",
        nontrivial.iter().skip(1).take(8).collect::<Vec<_>>()
    );
    println!(
        "\n  => the directed feedback (a node reaching itself) concentrates in {} SCC(s); the giant SCC holds\n\
         \x20    {:.1}% of nodes. The bare-graph β₁=13731 counts UNDIRECTED cycles (broader), but these SCCs are\n\
         \x20    where the genuine loops live — the loci the real Hodge build's harmonic permutation-test will\n\
         \x20    score against (does harmonic support localize here beyond a random baseline?). Dep-free.",
        nontrivial.len(),
        largest as f64 / n as f64 * 100.0
    );
}

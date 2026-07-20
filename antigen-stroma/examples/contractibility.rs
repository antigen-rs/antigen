//! How CONTRACTIBLE is the stroma? Counts non-branching (degree-2) chains in the call graph.
//!
//! A degree-2 node is a *pass-through* — in code terms a delegation link (`a → b → c` where `b` only
//! forwards). Collapsing runs of them (PATRICIA/radix-style, or skeletal-formula-style) shrinks the graph
//! while preserving branch structure; for a sheaf the restriction maps COMPOSE along the collapsed chain.
//! This measures whether that would pay here, before any representation is designed for it.
//!
//! Usage: `cargo run -p antigen-stroma --example contractibility -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::curvature::{UndirectedGraph, index_pairs};
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};

fn main() {
    let p = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let recons = ingest_scip(Path::new(&p));
    let mut sp: Vec<(&str, &str)> = Vec::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                sp.push((e.src_symbol.as_str(), e.dst_symbol.as_str()));
            }
        }
    }
    if sp.is_empty() {
        println!("(no call edges from {p})");
        return;
    }
    let (idx, ip) = index_pairs(&sp);
    let g = UndirectedGraph::from_pairs(idx.len(), &ip);
    let n = g.node_count();
    let mut hist = [0usize; 5]; // deg 0,1,2,3,4+
    for v in 0..n {
        hist[g.degree(v).min(4)] += 1;
    }
    println!("== contractibility of the call graph ==\n");
    println!("  nodes {n}   undirected edges {}", g.edges().len());
    for (d, c) in hist.iter().enumerate() {
        let label = if d == 4 {
            "4+".to_string()
        } else {
            d.to_string()
        };
        println!(
            "  degree {label:<3}: {c:>6}  ({:.1}%)",
            *c as f64 / n as f64 * 100.0
        );
    }
    let deg2 = hist[2];
    println!(
        "\n  => {deg2} pass-through (degree-2) nodes = {:.1}% of the graph.",
        deg2 as f64 / n as f64 * 100.0
    );
    println!(
        "     Contracting every maximal degree-2 chain removes at most that many nodes AND that many\n\
         \x20    edges, so beta1 is unchanged. Payoff is bounded by this number: a small share means the\n\
         \x20    representation idea is elegant but does not pay HERE (a claim about antigen, not in general)."
    );
}

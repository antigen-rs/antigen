//! Hodge β₁ — the first Hodge number of antigen's call graph (the Hodge frontier, first slice).
//!
//! An organ ≡ a local homology over the stroma (nb 028); the flow side of that (nb 030 §7, the
//! decompositions-and-flows charter) is the Helmholtz-Hodge decomposition `gradient ⊕ curl ⊕ harmonic`,
//! where the **harmonic component IS the homology** and `dim(harmonic) = β₁ = dim H₁`. On the BARE call
//! graph (a 1-complex, no 2-simplices filled) every cyclic flow is harmonic, so the harmonic dimension is
//! just `β₁ = E − V + C` (oriented edges − vertices + weakly-connected components) — computable by counting,
//! no linear solve. This is "harmonic = homology" at the dimension level; `β₁` of a call graph is also
//! exactly the aggregate `McCabe` cyclomatic complexity = the number of independent feedback cycles.
//!
//! Usage: `cargo run -p antigen-stroma --example hodge_betti -- [path/to.scip]`

#![allow(clippy::cast_possible_wrap)] // β₁ ≥ 0 for a graph; the counts are ~10⁴, far from i64 wrap

use std::collections::{HashMap, HashSet};
use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};

/// Disjoint-set union over string node ids (path-halving find).
#[derive(Default)]
struct UnionFind {
    parent: HashMap<String, String>,
}

impl UnionFind {
    fn add(&mut self, x: &str) {
        self.parent
            .entry(x.to_string())
            .or_insert_with(|| x.to_string());
    }

    fn find(&mut self, x: &str) -> String {
        let mut root = x.to_string();
        while self.parent[&root] != root {
            let grand = self.parent[&self.parent[&root]].clone();
            self.parent.insert(root.clone(), grand.clone());
            root = grand;
        }
        root
    }

    fn union(&mut self, a: &str, b: &str) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent.insert(ra, rb);
        }
    }
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!("== Hodge β₁ of antigen's call graph (dim harmonic = dim H₁) ==\n");

    let recons = ingest_scip(Path::new(&scip_path_s));
    if recons.is_empty() {
        println!("(no SCIP reconstructions from {scip_path_s})");
        return;
    }

    // Oriented Call edges (deduped), the node set, and weakly-connected components.
    let mut edges: HashSet<(String, String)> = HashSet::new();
    let mut nodes: HashSet<String> = HashSet::new();
    let mut uf = UnionFind::default();
    let mut self_loops = 0usize;
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                if e.src_symbol == e.dst_symbol {
                    self_loops += 1;
                }
                edges.insert((e.src_symbol.clone(), e.dst_symbol.clone()));
                for n in [&e.src_symbol, &e.dst_symbol] {
                    nodes.insert(n.clone());
                    uf.add(n);
                }
                uf.union(&e.src_symbol, &e.dst_symbol);
            }
        }
    }

    let v = nodes.len();
    let e = edges.len();
    let components: HashSet<String> = nodes.iter().map(|n| uf.find(n)).collect();
    let c = components.len();
    // β₁ = E − V + C (cellular homology of the oriented call graph as a 1-complex).
    let beta1 = e as i64 - v as i64 + c as i64;

    println!("  V (nodes)                    = {v}");
    println!("  E (oriented call edges)      = {e}");
    println!("  C (weakly-connected comps)   = {c}");
    println!("  direct self-recursion edges  = {self_loops}");
    println!("\n  β₁ = E − V + C = {beta1}");
    println!(
        "\n  => dim(harmonic space) = dim H₁ = {beta1} independent feedback cycles in antigen's call graph\n\
         \x20    (= the aggregate cyclomatic complexity). These are the loops NO spanning tree can break —\n\
         \x20    the irreducible circulation the gradient/trend component cannot explain. The full\n\
         \x20    Helmholtz split (gradient potential via a Laplacian solve → gradient ⊕ cyclic, then the\n\
         \x20    2-simplex fill → curl ⊕ harmonic representatives) is the next slice."
    );
}

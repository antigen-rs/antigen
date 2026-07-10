//! SCC-condensation (Tarjan) — the pre-pass the blast (counting) semiring requires (ADR-068 R4).
//!
//! A non-idempotent counting closure diverges on cycles. Condensing each strongly-connected component
//! to a single super-node yields a DAG on which the counting closure (and blast reachability)
//! terminates. Producing a [`Condensation`] is the proof-of-pre-pass that
//! [`super::semiring::run_condensed`] consumes via [`super::semiring::Condensed`].

use super::closure::ReachabilityRelation;
use super::semiring::{Condensed, Semiring};

/// The condensation of a graph: SCC membership + the acyclic super-node DAG.
pub struct Condensation {
    /// `scc_of[node_index]` = the id of the SCC that node belongs to.
    pub scc_of: Vec<usize>,
    /// `sccs[scc_id]` = the member node indices of that SCC.
    pub sccs: Vec<Vec<usize>>,
    /// `dag_adj[scc_id]` = the successor SCC ids (deduplicated, self-loops removed) — an acyclic DAG.
    pub dag_adj: Vec<Vec<usize>>,
}

impl Condensation {
    /// Mint the [`Condensed`] proof-of-pre-pass for a non-idempotent (counting) closure over this
    /// condensation's acyclic DAG — the ONLY way to obtain a `Condensed<S>` (R4). The proof carries the
    /// DAG, so [`run_condensed`](super::semiring::run_condensed) provably runs on a condensed graph.
    #[must_use]
    pub fn proof<S: Semiring>(&self) -> Condensed<S> {
        Condensed::from_condensed_dag(self.dag_adj.clone())
    }
}

/// Condense a relation's SCCs into a DAG via Tarjan's algorithm.
#[must_use]
pub fn condense(rel: &ReachabilityRelation) -> Condensation {
    let n = rel.len();
    let mut state = Tarjan {
        rel,
        index: vec![usize::MAX; n],
        lowlink: vec![0; n],
        on_stack: vec![false; n],
        stack: Vec::new(),
        next_index: 0,
        scc_of: vec![usize::MAX; n],
        sccs: Vec::new(),
    };
    for v in 0..n {
        if state.index[v] == usize::MAX {
            state.strong_connect(v);
        }
    }

    // Build the super-node DAG: an edge scc(u) -> scc(v) for every base edge u -> v across components.
    let scc_count = state.sccs.len();
    let mut dag_adj: Vec<Vec<usize>> = vec![Vec::new(); scc_count];
    for u in 0..n {
        for &(v, _tier) in rel.forward_adj(u) {
            let (su, sv) = (state.scc_of[u], state.scc_of[v]);
            if su != sv && !dag_adj[su].contains(&sv) {
                dag_adj[su].push(sv);
            }
        }
    }

    Condensation {
        scc_of: state.scc_of,
        sccs: state.sccs,
        dag_adj,
    }
}

/// Tarjan's SCC working state (iterative recursion over the relation's forward adjacency).
struct Tarjan<'a> {
    rel: &'a ReachabilityRelation,
    index: Vec<usize>,
    lowlink: Vec<usize>,
    on_stack: Vec<bool>,
    stack: Vec<usize>,
    next_index: usize,
    scc_of: Vec<usize>,
    sccs: Vec<Vec<usize>>,
}

impl Tarjan<'_> {
    fn strong_connect(&mut self, v: usize) {
        self.index[v] = self.next_index;
        self.lowlink[v] = self.next_index;
        self.next_index += 1;
        self.stack.push(v);
        self.on_stack[v] = true;

        for i in 0..self.rel.forward_adj(v).len() {
            let w = self.rel.forward_adj(v)[i].0;
            if self.index[w] == usize::MAX {
                self.strong_connect(w);
                self.lowlink[v] = self.lowlink[v].min(self.lowlink[w]);
            } else if self.on_stack[w] {
                self.lowlink[v] = self.lowlink[v].min(self.index[w]);
            }
        }

        // v is a root of an SCC — pop the component.
        if self.lowlink[v] == self.index[v] {
            let scc_id = self.sccs.len();
            let mut members = Vec::new();
            loop {
                let w = self.stack.pop().expect("tarjan stack underflow");
                self.on_stack[w] = false;
                self.scc_of[w] = scc_id;
                members.push(w);
                if w == v {
                    break;
                }
            }
            self.sccs.push(members);
        }
    }
}

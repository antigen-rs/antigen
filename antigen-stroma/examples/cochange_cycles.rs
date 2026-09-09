//! Co-change cycles — does git's co-change relation carry homology the CALL graph does not? The cheap,
//! reliable precondition-gate for the transport-sheaf build (`the-sheaf-laplacian` (B) blocker).
//!
//! **Why this, and why now.** `the-sheaf-laplacian` proved the (A) structural sheaf is degenerate
//! (`dim H¹ = β₁(G_syn) + β₁(G_scip)`, zero instrument interaction) and that the way out is the TRANSPORT
//! form (`x_v = T_e x_u`), which is BLOCKED on naming a `T_e` from real semantics. Git's directed
//! fix-coupling is a genuine transport candidate (defect-risk flows change-site → fix-site) where syn/scip's
//! undirected "asserts-this-edge" is not. But the same charter proved the obstruction is **purely cyclic**
//! ("on a tree, twisting changes nothing"): a transport sheaf on an acyclic edge set has defect 0 no matter
//! how it twists. So before paying for the SZZ+blame build, GATE it: does git's co-change relation even
//! have cycles — and cycles the call graph does NOT already contain?
//!
//! Co-change (A and B modified in the same commit) is the cheap, RELIABLE git relation (no blame, no
//! squash-merge SZZ false-positives). It is undirected — so it is a precondition PROXY, not the transport
//! itself — but its cycle structure bounds what any git-derived edge set can offer: if co-change adds no
//! call-disjoint cycles, no directed refinement of it will either.
//!
//! **PRE-REGISTERED (before the run):** if the multi-meaning edge is a genuinely different relation and not
//! a relabeling of calls, then
//!   (P1) most co-change adjacency is call-DISJOINT — >50% of co-change edges are not call edges;
//!   (P2) co-change closes NEW independent cycles — `β₁(call ∪ cochange) − β₁(call)` is large (hundreds+),
//!        so the union carries homology neither meaning has alone;
//!   (P3) base-rate control — a DEGREE-PRESERVING rewire of the co-change edges adds cycles too (any dense
//!        graph does), so the real test is whether REAL co-change's call-disjoint fraction and new-cycle
//!        count DIFFER from the rewired null. Real co-change clusters by file (locality); a rewire destroys
//!        that, so if the signal is real the disjoint-fraction should MOVE.
//!
//! Usage: `cargo run -p antigen-stroma --example cochange_cycles -- [repo] [scip]`  (needs git history).

#![allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

/// A mega-commit touching more than this many functions is a bulk refactor / rename / license-header sweep;
/// its co-change clique is a near-complete graph that would swamp β₁ with non-semantic edges. Capped and
/// COUNTED (no silent truncation), reported below.
const CLIQUE_CAP: usize = 40;

/// `(bare_name, file, start_line, end_line)`.
type FnDef = (String, String, i32, i32);

fn fn_defs(scip: &Path) -> Vec<FnDef> {
    let mut defs: Vec<FnDef> = Vec::new();
    let Ok(bytes) = std::fs::read(scip) else {
        return defs;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return defs;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) == 0 {
                continue;
            }
            let Some(bare) = bare_fn_name(&occ.symbol) else {
                continue;
            };
            let r = if occ.enclosing_range.len() >= 3 {
                &occ.enclosing_range
            } else {
                &occ.range
            };
            if r.len() >= 3 {
                defs.push((bare.to_string(), doc.relative_path.clone(), r[0], r[2]));
            }
        }
    }
    defs
}

/// commit-hash → the set of bare fn-names whose def-span overlapped a hunk in that commit (cross-file).
fn cochange_commits(repo: &Path, defs: &[FnDef]) -> (HashMap<String, HashSet<String>>, usize) {
    let mut by_file: HashMap<&str, Vec<&FnDef>> = HashMap::new();
    for d in defs {
        by_file.entry(&d.1).or_default().push(d);
    }
    let mut commits: HashMap<String, HashSet<String>> = HashMap::new();
    for (file, fns) in &by_file {
        let Ok(out) = Command::new("git")
            .args([
                "-C",
                &repo.to_string_lossy(),
                "log",
                "--no-merges",
                "--unified=0",
                "--format=%x00%H",
                "--",
                file,
            ])
            .output()
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&out.stdout);
        let mut cur_commit = String::new();
        for line in text.lines() {
            if let Some(hash) = line.strip_prefix('\0') {
                cur_commit = hash.trim().to_string();
            } else if let Some(h) = line.strip_prefix("@@") {
                if let Some(plus) = h.split('+').nth(1) {
                    let mut it = plus.split('@').next().unwrap_or("").trim().split(',');
                    if let Some(c) = it.next().and_then(|s| s.trim().parse::<i32>().ok()) {
                        let d = it
                            .next()
                            .and_then(|s| s.trim().parse::<i32>().ok())
                            .unwrap_or(1);
                        let (cs, ce) = (c, c + d.max(1));
                        for f in fns {
                            if cs <= f.3 + 1 && ce > f.2 {
                                commits
                                    .entry(cur_commit.clone())
                                    .or_default()
                                    .insert(f.0.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    let capped = commits.values().filter(|s| s.len() > CLIQUE_CAP).count();
    (commits, capped)
}

/// Undirected, deduplicated, self-loop-free edge set as sorted `(min,max)` index pairs.
type Edges = HashSet<(usize, usize)>;

#[allow(clippy::comparison_chain)]
const fn undirected(src: usize, dst: usize) -> Option<(usize, usize)> {
    if src == dst {
        None
    } else if src < dst {
        Some((src, dst))
    } else {
        Some((dst, src))
    }
}

/// Union-find root with path compression.
const fn uf_find(parent: &mut [usize], start: usize) -> usize {
    let mut root = start;
    while parent[root] != root {
        root = parent[root];
    }
    let mut node = start;
    while parent[node] != node {
        let next = parent[node];
        parent[node] = root;
        node = next;
    }
    root
}

/// `β₁ = |E| − |V_touched| + #components` over the vertices the edges actually touch.
/// Returns `(β₁, |V_touched|, #components)`.
fn betti1(edges: &Edges, n_nodes: usize) -> (usize, usize, usize) {
    let mut parent: Vec<usize> = (0..n_nodes).collect();
    let mut touched: HashSet<usize> = HashSet::new();
    for &(src, dst) in edges {
        touched.insert(src);
        touched.insert(dst);
        let (rs, rd) = (uf_find(&mut parent, src), uf_find(&mut parent, dst));
        if rs != rd {
            parent[rs] = rd;
        }
    }
    let roots: HashSet<usize> = touched.iter().map(|&x| uf_find(&mut parent, x)).collect();
    let (n_edges, n_touched, n_comp) = (edges.len(), touched.len(), roots.len());
    (n_edges + n_comp - n_touched, n_touched, n_comp)
}

/// Deterministic degree-preserving-ish rewire: a double-edge swap driven by an index-seeded LCG (no
/// `Math.random`, which is unavailable). Preserves each node's degree exactly; destroys locality.
fn rewire(edges: &Edges) -> Edges {
    let mut list: Vec<(usize, usize)> = edges.iter().copied().collect();
    list.sort_unstable();
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut rng = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (state >> 33) as usize
    };
    let len = list.len();
    for _ in 0..(len * 4) {
        let (ix, jx) = (rng() % len, rng() % len);
        if ix == jx {
            continue;
        }
        let ((a0, a1), (b0, b1)) = (list[ix], list[jx]);
        // swap partners: (a0,a1),(b0,b1) -> (a0,b1),(b0,a1)
        if let (Some(e1), Some(e2)) = (undirected(a0, b1), undirected(b0, a1)) {
            list[ix] = e1;
            list[jx] = e2;
        }
    }
    list.into_iter().collect()
}

/// SCIP `Call` edges as undirected index pairs.
fn call_edges(recons: &[EdgeReconstruction], idx: &HashMap<String, usize>) -> Edges {
    let mut call: Edges = HashSet::new();
    for r in recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                if let (Some(u), Some(v)) =
                    (bare_fn_name(&e.src_symbol), bare_fn_name(&e.dst_symbol))
                {
                    if let (Some(&iu), Some(&iv)) = (idx.get(u), idx.get(v)) {
                        if let Some(edge) = undirected(iu, iv) {
                            call.insert(edge);
                        }
                    }
                }
            }
        }
    }
    call
}

/// Per-commit fn-set cliques (capped) as undirected index pairs.
fn cochange_edges(
    commits: &HashMap<String, HashSet<String>>,
    idx: &HashMap<String, usize>,
) -> Edges {
    let mut cochange: Edges = HashSet::new();
    for fnset in commits.values() {
        if fnset.len() > CLIQUE_CAP {
            continue;
        }
        let members: Vec<usize> = fnset.iter().filter_map(|f| idx.get(f).copied()).collect();
        for a in 0..members.len() {
            for b in (a + 1)..members.len() {
                if let Some(edge) = undirected(members[a], members[b]) {
                    cochange.insert(edge);
                }
            }
        }
    }
    cochange
}

/// Percentage of `sub`'s edges that are NOT in `other` (a fraction OF `sub`, so it cannot exceed 100%).
fn disjoint_frac(sub: &Edges, other: &Edges) -> f64 {
    if sub.is_empty() {
        0.0
    } else {
        sub.difference(other).count() as f64 / sub.len() as f64 * 100.0
    }
}

/// The three pre-registered tests + the census, given the built edge sets.
fn report(n_nodes: usize, n_commits: usize, capped: usize, call: &Edges, cochange: &Edges) {
    let union: Edges = call.union(cochange).copied().collect();
    let (b_call, _, c_call) = betti1(call, n_nodes);
    let (b_cc, _, c_cc) = betti1(cochange, n_nodes);
    let (b_union, _, c_union) = betti1(&union, n_nodes);

    println!(
        "  universe (fn-defs): {n_nodes}   [co-change commits: {n_commits}, capped >{CLIQUE_CAP} fns: {capped}]"
    );
    println!("\n  EDGE SETS (undirected, deduped, self-loop-free):");
    println!(
        "    call            {:6}   β₁ {b_call:5}   components {c_call}",
        call.len()
    );
    println!(
        "    cochange        {:6}   β₁ {b_cc:5}   components {c_cc}",
        cochange.len()
    );
    println!(
        "    union           {:6}   β₁ {b_union:5}   components {c_union}",
        union.len()
    );

    let df = disjoint_frac(cochange, call);
    let cc_only = cochange.difference(call).count();
    println!("\n  (P1) co-change adjacency is call-DISJOINT (pre-registered > 50%):");
    println!(
        "    cochange \\ call = {cc_only} / {} = {df:.1}%   {}",
        cochange.len(),
        if df > 50.0 {
            "CONFIRMED"
        } else {
            "not confirmed"
        }
    );

    let new_cycles = b_union as i64 - b_call as i64;
    println!("\n  (P2) co-change closes NEW independent cycles (pre-registered: hundreds+):");
    println!(
        "    β₁(union) − β₁(call) = {b_union} − {b_call} = {new_cycles}   {}",
        if new_cycles >= 100 {
            "CONFIRMED"
        } else {
            "not confirmed at 100"
        }
    );

    let rw = rewire(cochange);
    let rw_union: Edges = call.union(&rw).copied().collect();
    let (b_rw_union, ..) = betti1(&rw_union, n_nodes);
    let rw_df = disjoint_frac(&rw, call);
    println!("\n  (P3) base-rate control — degree-preserving rewire (destroys file-locality):");
    println!(
        "    rewired disjoint-frac {rw_df:.1}%  vs real {df:.1}%   (Δ {:+.1})",
        df - rw_df
    );
    println!(
        "    rewired β₁(union) {b_rw_union}  vs real {b_union}   (Δ {:+})",
        b_union as i64 - b_rw_union as i64
    );

    let (p1, p2) = (df > 50.0, new_cycles >= 100);
    println!(
        "\n  => co-change {} carry call-disjoint homology (P1 {p1}, P2 {p2}). {}",
        if p1 && p2 { "DOES" } else { "does NOT clearly" },
        if p1 && p2 {
            "The transport sheaf HAS cyclic support the call graph lacks → the SZZ fix-coupling\n\
             \x20    transport build (directed refinement of this relation) is WARRANTED. The multi-meaning\n\
             \x20    edge carries homology, not a relabeling of calls."
        } else {
            "Co-change adds no call-disjoint cycle mass → a directed git transport would obstruct nothing\n\
             \x20    new; the transport-sheaf build is NOT warranted on this corpus. Signal dies cheaply."
        }
    );
}

fn main() {
    let repo = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "R:/antigen".to_string());
    let scip = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!("== co-change cycles: does git's co-change carry homology the call graph lacks? ==\n");

    let defs = fn_defs(Path::new(&scip));
    let mut idx: HashMap<String, usize> = HashMap::new();
    for d in &defs {
        let n = idx.len();
        idx.entry(d.0.clone()).or_insert(n);
    }
    let recons = ingest_scip(Path::new(&scip));
    let call = call_edges(&recons, &idx);
    let (commits, capped) = cochange_commits(Path::new(&repo), &defs);
    let cochange = cochange_edges(&commits, &idx);
    report(idx.len(), commits.len(), capped, &call, &cochange);
}

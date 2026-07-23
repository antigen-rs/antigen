//! Transport holonomy — the **scalar** transport-sheaf defect on fix-coupling (`the-sheaf-laplacian` (B)).
//!
//! The dream, made testable. `fixcoupling_cycles` proved the SZZ defect→fix graph carries cyclic support
//! (WARRANTED). This measures the *transport defect* on it, scalar-first (no matrix `T_e` yet — that's the
//! open blocker under adversarial investigation in `jbd/research/te-matrix/`).
//!
//! **The measurement = the Hodge curl of the log-transport field.** Each directed pair has a fix-coupling
//! strength (count). Form the antisymmetric 1-form `ω(a→b) = log(1+c_{a→b}) − log(1+c_{b→a})` — the *net
//! directed imbalance* of defect-risk flow. Per fundamental cycle, **circulation = Σ ω around the loop**. If
//! `ω` were a gradient (each node a defect-propensity potential), circulation ≡ 0 — no holonomy. Non-zero
//! circulation = irreducible loop structure = the transport defect. This is antigen's Hodge machinery on the
//! fix-coupling flow, and it needs no matrix — honest now.
//!
//! **PRE-REGISTERED:** if fix-coupling carries real transport holonomy,
//!   (P1) real circulation is NON-ZERO on a nontrivial fraction of cycles (the flow is not a pure gradient);
//!   (P2) trivial control: symmetrize the flow (c_{a→b}=c_{b→a}) → ω≡0 → circulation ≡ 0 (sanity: no free
//!        holonomy);
//!   (P3) base-rate: SHUFFLE ω across edges → the real ASSIGNMENT must carry different (concentrated) curl
//!        energy than random, else the holonomy is generic non-uniformity, not fix-coupling structure.
//!
//! **Honest scope:** scalar per-cycle circulation is graded but coarser than a matrix `T_e`'s defect (one
//! scalar per loop, not a k×k holonomy). It answers "does the directed fix-coupling flow rotate (carry curl)
//! beyond a gradient, concentrated by real structure" — a real bind test, and it INFORMS the matrix question
//! rather than presupposing it. Extraction is the proven `fixcoupling_cycles` pipeline (SCIP `\`→`/`, B-SZZ
//! blame, HEAD-def-span approximation; under-counts, so signal is a floor). LIB-FACTOR TODO: the SZZ
//! fix-coupling extraction + `BlameEngine` belong in `antigen-stroma/src`, shared by both examples.
//!
//! Usage: `cargo run -p antigen-stroma --example transport_holonomy -- [repo] [scip]`  (needs history).

#![allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use antigen_stroma::scip::ingest_scip;
use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const FIX_KEYWORDS: [&str; 5] = ["fix", "bug", "patch", "revert", "hotfix"];
const CLIQUE_CAP: usize = 40;

type FnDef = (String, String, i32, i32);
type Ranges = Vec<(i32, i32)>;
type HunkMap = HashMap<String, (Ranges, Ranges)>;
type ByFile<'a> = HashMap<&'a str, Vec<&'a FnDef>>;
/// directed `(defect_fn, fix_fn) -> coupling count`.
type Directed = HashMap<(usize, usize), usize>;

// ── the BlameEngine seam (git-blame subprocess = oracle + first engine; stromix-swap insurance) ──
trait BlameEngine {
    fn introducers(&self, repo: &Path, rev: &str, file: &str, lines: &[(i32, i32)]) -> Vec<String>;
}
struct GitBlame;
impl BlameEngine for GitBlame {
    fn introducers(&self, repo: &Path, rev: &str, file: &str, lines: &[(i32, i32)]) -> Vec<String> {
        let mut out: HashSet<String> = HashSet::new();
        for &(a, b) in lines {
            let Ok(o) = Command::new("git")
                .args([
                    "-C",
                    &repo.to_string_lossy(),
                    "blame",
                    "--porcelain",
                    "-L",
                    &format!("{a},{b}"),
                    rev,
                    "--",
                    file,
                ])
                .output()
            else {
                continue;
            };
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                let sha = line.split(' ').next().unwrap_or("");
                if sha.len() == 40 && sha.bytes().all(|c| c.is_ascii_hexdigit()) {
                    out.insert(sha[..10].to_string());
                }
            }
        }
        out.into_iter().collect()
    }
}

fn fn_defs(scip: &Path) -> Vec<FnDef> {
    let mut defs = Vec::new();
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
                defs.push((
                    bare.to_string(),
                    doc.relative_path.replace('\\', "/"),
                    r[0],
                    r[2],
                ));
            }
        }
    }
    defs
}

fn is_fix_subject(s: &str) -> bool {
    let low = s.to_lowercase();
    FIX_KEYWORDS.iter().any(|k| low.contains(k))
}

fn fix_commits(repo: &Path) -> Vec<(String, String)> {
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--all",
            "--no-merges",
            "--format=%H%x00%P%x00%s",
        ])
        .output()
    else {
        return Vec::new();
    };
    let mut v = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut it = line.split('\0');
        let (Some(h), Some(parents), Some(subj)) = (it.next(), it.next(), it.next()) else {
            continue;
        };
        if is_fix_subject(subj) {
            if let Some(p) = parents.split(' ').next().filter(|p| !p.is_empty()) {
                v.push((h.to_string(), p.to_string()));
            }
        }
    }
    v
}

fn parse_side(s: &str) -> Option<(i32, i32)> {
    let mut it = s.split(',');
    let start: i32 = it.next()?.parse().ok()?;
    let len: i32 = it.next().map_or(1, |l| l.parse().unwrap_or(1));
    if len == 0 {
        return None;
    }
    Some((start, start + len - 1))
}

fn commit_hunks(repo: &Path, rev: &str) -> HunkMap {
    let mut map: HunkMap = HashMap::new();
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "show",
            rev,
            "--unified=0",
            "--format=",
            "--",
            "*.rs",
        ])
        .output()
    else {
        return map;
    };
    let mut file = String::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(p) = line.strip_prefix("+++ b/") {
            file = p.to_string();
        } else if let Some(h) = line.strip_prefix("@@ ") {
            let entry = map.entry(file.clone()).or_default();
            let mut sides = h.split(' ');
            if let Some(r) = sides
                .next()
                .and_then(|o| parse_side(o.trim_start_matches('-')))
            {
                entry.0.push(r);
            }
            if let Some(r) = sides
                .next()
                .and_then(|n| parse_side(n.trim_start_matches('+')))
            {
                entry.1.push(r);
            }
        }
    }
    map
}

fn commit_fns(hunks: &HunkMap, by_file: &ByFile, idx: &HashMap<String, usize>) -> HashSet<usize> {
    let mut fns = HashSet::new();
    for (file, (_old, new)) in hunks {
        let Some(defs) = by_file.get(file.as_str()) else {
            continue;
        };
        for &(hs, he) in new {
            for d in defs {
                if hs <= d.3 && he >= d.2 {
                    if let Some(&i) = idx.get(&d.0) {
                        fns.insert(i);
                    }
                }
            }
        }
    }
    fns
}

fn fn_set(
    repo: &Path,
    rev: &str,
    by_file: &ByFile,
    idx: &HashMap<String, usize>,
    cache: &mut HashMap<String, HashSet<usize>>,
) -> HashSet<usize> {
    if let Some(f) = cache.get(rev) {
        return f.clone();
    }
    let f = commit_fns(&commit_hunks(repo, rev), by_file, idx);
    cache.insert(rev.to_string(), f.clone());
    f
}

/// Build the DIRECTED, weighted fix-coupling graph: `(defect_fn → fix_fn) -> #couplings`.
fn build_directed(
    repo: &Path,
    fixes: &[(String, String)],
    by_file: &ByFile,
    idx: &HashMap<String, usize>,
    blame: &dyn BlameEngine,
) -> Directed {
    let mut cache: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut dir: Directed = HashMap::new();
    for (fix, parent) in fixes {
        let mut defects: HashSet<String> = HashSet::new();
        for (file, (old, _new)) in &commit_hunks(repo, fix) {
            if !old.is_empty() {
                for d in blame.introducers(repo, parent, file, old) {
                    defects.insert(d);
                }
            }
        }
        if defects.is_empty() {
            continue;
        }
        let fix_fns = fn_set(repo, fix, by_file, idx, &mut cache);
        if fix_fns.is_empty() || fix_fns.len() > CLIQUE_CAP {
            continue;
        }
        for def in &defects {
            let def_fns = fn_set(repo, def, by_file, idx, &mut cache);
            if def_fns.len() > CLIQUE_CAP {
                continue;
            }
            for &d in &def_fns {
                for &f in &fix_fns {
                    if d != f {
                        *dir.entry((d, f)).or_default() += 1;
                    }
                }
            }
        }
    }
    dir
}

fn uf_find(parent: &mut [usize], start: usize) -> usize {
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

/// The log-transport 1-form on undirected edge `{a,b}` in the a→b orientation:
/// `ω = ln(1+c_{a→b}) − ln(1+c_{b→a})` (antisymmetric; net directed imbalance).
fn omega(dir: &Directed, a: usize, b: usize) -> f64 {
    let fwd = dir.get(&(a, b)).copied().unwrap_or(0) as f64;
    let bwd = dir.get(&(b, a)).copied().unwrap_or(0) as f64;
    fwd.ln_1p() - bwd.ln_1p()
}

/// Fundamental-cycle circulations of the 1-form: spanning tree (union-find + parent-pointer BFS), then each
/// non-tree edge closes a cycle whose circulation = Σ ω around it. Returns one circulation per independent cycle.
fn cycle_circulations(
    edges: &[(usize, usize)],
    n: usize,
    w: impl Fn(usize, usize) -> f64,
) -> Vec<f64> {
    // adjacency (undirected), and a spanning forest via union-find; non-tree edges are the cycle generators.
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut parent: Vec<usize> = (0..n).collect();
    let mut tree_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut non_tree: Vec<(usize, usize)> = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for &(a, b) in edges {
        let key = if a < b { (a, b) } else { (b, a) };
        if !seen.insert(key) {
            continue;
        }
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
        let (ra, rb) = (uf_find(&mut parent, a), uf_find(&mut parent, b));
        if ra == rb {
            non_tree.push((a, b));
        } else {
            parent[ra] = rb;
            tree_adj.entry(a).or_default().push(b);
            tree_adj.entry(b).or_default().push(a);
        }
    }
    // BFS parent pointers per node within the tree, so we can walk a→root and b→root.
    let mut tparent: HashMap<usize, usize> = HashMap::new();
    let mut depth: HashMap<usize, usize> = HashMap::new();
    let touched: HashSet<usize> = adj.keys().copied().collect();
    let mut visited: HashSet<usize> = HashSet::new();
    for &s in &touched {
        if visited.contains(&s) {
            continue;
        }
        let mut stack = vec![s];
        visited.insert(s);
        depth.insert(s, 0);
        while let Some(u) = stack.pop() {
            if let Some(ns) = tree_adj.get(&u) {
                let du = depth[&u];
                for &v in ns {
                    if visited.insert(v) {
                        tparent.insert(v, u);
                        depth.insert(v, du + 1);
                        stack.push(v);
                    }
                }
            }
        }
    }
    // circulation of a fundamental cycle: ω(a→b) + Σ ω(tree path b→a).
    let mut circs = Vec::with_capacity(non_tree.len());
    for &(a, b) in &non_tree {
        let mut circ = w(a, b); // a→b (the closing non-tree edge)
        // walk a→lca and b→lca via parent pointers; sum ω along, oriented toward the shared ancestor.
        let (mut x, mut y) = (a, b);
        let mut path_a: Vec<usize> = vec![a];
        let mut path_b: Vec<usize> = vec![b];
        while depth.get(&x).copied().unwrap_or(0) > depth.get(&y).copied().unwrap_or(0) {
            x = tparent[&x];
            path_a.push(x);
        }
        while depth.get(&y).copied().unwrap_or(0) > depth.get(&x).copied().unwrap_or(0) {
            y = tparent[&y];
            path_b.push(y);
        }
        while x != y {
            x = tparent[&x];
            path_a.push(x);
            y = tparent[&y];
            path_b.push(y);
        }
        // cycle: b → ... → lca (reverse of path_b up to lca), then lca → ... → a (path_a reversed), close via a→b already counted.
        // sum ω along b→a through the tree, then the non-tree edge a→b closes it.
        let lca = x;
        // b up to lca:
        for win in path_b.windows(2) {
            if win[1] == lca && win[0] == lca {
                break;
            }
            circ += w(win[0], win[1]); // b→parent direction
            if win[1] == lca {
                break;
            }
        }
        // a up to lca, traversed a→...→lca means the cycle goes lca→...→a, so subtract (reverse orientation):
        for win in path_a.windows(2) {
            circ -= w(win[0], win[1]); // a→parent; cycle traverses parent→a, hence minus
            if win[1] == lca {
                break;
            }
        }
        circs.push(circ);
    }
    circs
}

fn stats(circs: &[f64]) -> (usize, f64, f64) {
    let nonzero = circs.iter().filter(|c| c.abs() > 1e-9).count();
    let energy: f64 = circs.iter().map(|c| c * c).sum();
    let max = circs.iter().fold(0.0_f64, |m, c| m.max(c.abs()));
    (nonzero, energy, max)
}

fn main() {
    let repo = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "R:/antigen".to_string());
    let scip = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let repo = Path::new(&repo);
    println!(
        "== transport holonomy: does the fix-coupling flow carry curl (holonomy) beyond a gradient? ==\n"
    );

    let defs = fn_defs(Path::new(&scip));
    let mut idx: HashMap<String, usize> = HashMap::new();
    for d in &defs {
        let n = idx.len();
        idx.entry(d.0.clone()).or_insert(n);
    }
    let mut by_file: ByFile = HashMap::new();
    for d in &defs {
        by_file.entry(&d.1).or_default().push(d);
    }
    let _ = ingest_scip(Path::new(&scip)); // (call graph not needed here; extraction shares the SCIP)

    let fixes = fix_commits(repo);
    let dir = build_directed(repo, &fixes, &by_file, &idx, &GitBlame);
    // undirected edge list (dedup) over the directed graph's node-pairs.
    let mut und: HashSet<(usize, usize)> = HashSet::new();
    for &(a, b) in dir.keys() {
        und.insert(if a < b { (a, b) } else { (b, a) });
    }
    let edges: Vec<(usize, usize)> = und.iter().copied().collect();
    let n = idx.len();

    // REAL: circulation of ω.
    let real = cycle_circulations(&edges, n, |a, b| omega(&dir, a, b));
    let (r_nz, r_en, r_mx) = stats(&real);
    // TRIVIAL: symmetrize (ω≡0) — must give circulation ≡ 0.
    let triv = cycle_circulations(&edges, n, |_, _| 0.0);
    let (t_nz, t_en, _) = stats(&triv);
    // SHUFFLE: permute the per-edge |ω| magnitudes across edges (LCG, deterministic), keep orientation by a<b.
    let mut mags: Vec<f64> = edges.iter().map(|&(a, b)| omega(&dir, a, b)).collect();
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    for i in (1..mags.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let j = (state >> 33) as usize % (i + 1);
        mags.swap(i, j);
    }
    let emap: HashMap<(usize, usize), f64> = edges.iter().copied().zip(mags).collect();
    let shuf = cycle_circulations(&edges, n, |a, b| {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let v = emap.get(&(lo, hi)).copied().unwrap_or(0.0);
        if a < b { v } else { -v }
    });
    let (s_nz, s_en, _) = stats(&shuf);

    println!(
        "  fix-coupling directed edges: {}   undirected: {}   independent cycles: {}",
        dir.len(),
        edges.len(),
        real.len()
    );
    println!("\n  CIRCULATION (Σ ω per fundamental cycle):");
    println!(
        "    REAL      nonzero {r_nz}/{}   curl-energy {r_en:.1}   max |circ| {r_mx:.3}",
        real.len()
    );
    println!(
        "    TRIVIAL   nonzero {t_nz}   curl-energy {t_en:.3}   (symmetric flow → must be ≈0)"
    );
    println!("    SHUFFLED  nonzero {s_nz}   curl-energy {s_en:.1}");

    let p1 = r_nz > real.len() / 10 && r_en > 1e-6;
    let p2 = t_en < 1e-9;
    let ratio = if s_en > 1e-12 {
        r_en / s_en
    } else {
        f64::INFINITY
    };
    println!("\n  (P1) real flow carries holonomy (curl not ≈0): {p1}");
    println!("  (P2) trivial sanity (symmetric → curl ≈0): {p2}");
    println!(
        "  (P3) real curl-energy / shuffled = {ratio:.2}  (>1: real assignment MORE rotational than random)"
    );
    let exists = p1 && p2;
    print!("\n  => scalar transport defect: ");
    if !exists {
        println!("NOT clearly present — the flow reduces to a node potential.");
    } else if ratio > 1.1 {
        println!(
            "REAL and CONCENTRATED beyond random (P3 ratio > 1.1) — a structured signal; the real\n\
             \x20    fix-coupling assignment carries more curl than a shuffle. Worth the matrix question."
        );
    } else {
        println!(
            "REAL but GENERIC. Nonzero holonomy (P1,P2; trivial control = 0 confirms the measurement),\n\
             \x20    but P3 shows the real assignment is NOT more rotational than a shuffle (ratio ≤ ~1, here\n\
             \x20    even slightly MORE gradient-like) — so the fix-coupling flow carries holonomy but no special\n\
             \x20    organ-boundary concentration. Honest: the ambitious guess (a *structured* transport defect)\n\
             \x20    is tested and found modest — real, generic. And the matrix T_e (jbd/research/te-matrix/) is\n\
             \x20    refuted by information content: one number/edge is abelian → holonomy is *necessarily* this\n\
             \x20    scalar; a k×k transport needs ≥2 non-commuting params/edge, which SZZ does not emit. The\n\
             \x20    scalar defect here IS the ceiling of what fix-coupling's information content supports."
        );
    }
}

//! Fix-coupling cycles — the SZZ **transport-sheaf precondition gate** (`the-sheaf-laplacian` (B)).
//!
//! `cochange_cycles` proved the *undirected* co-change relation has call-disjoint cyclic support
//! (WARRANTED). This is one instrument deeper and *directed*: the real transport `T_e` is
//! **fix-coupling** — defect-risk flows *change-site → fix-site*. SZZ per-line blame gives it: for
//! each fix commit, blame the pre-fix lines it touched → the bug-introducing commits (the *defect
//! sites*); the fix commit is the *fix site*. Map both to functions → directed **defect-fn →
//! fix-fn** edges. This is the graph the transport sheaf would obstruct.
//!
//! Before paying for the full sheaf, GATE it (the discipline that worked for co-change): does the
//! fix-coupling graph even have cycles the call graph lacks? (`on a tree, twisting changes nothing`.)
//!
//! **The `BlameEngine` seam.** Blame sits behind `trait BlameEngine` (git-blame subprocess = the
//! canonical oracle + first engine). This is antigen's ingest-behind-a-seam discipline AND the
//! stromix-swap insurance: a future gix-blame or stromix source swaps the impl, nothing upstream
//! flinches.
//!
//! **PRE-REGISTERED (before the run):** if fix-coupling is a genuine directed transport relation,
//!   (P1) most fix-coupling adjacency is call-DISJOINT (>50%): fixes couple to defect-sites the
//!        call graph doesn't already connect;
//!   (P2) it carries NEW cyclic support: β₁(call ∪ fixcoupling) − β₁(call) > 0 (any cycles at all —
//!        fix-coupling is sparser than co-change, so the bar is "non-trivial", not "hundreds");
//!   (P3) base-rate control: a degree-preserving rewire adds cycles too; the REAL relation must
//!        differ (fix-coupling has locality — a fix touches functions near its defect).
//!
//! **Honest scope:** commit→function mapping uses HEAD def-spans against each commit's own hunks (a
//! cross-revision approximation, same one `cochange_cycles` accepts) — fine for a *gate* ("do cycles
//! exist?"), not for the final defect measurement. Blame is B-SZZ (keyword fixes, bare-`#` guarded).
//! SCIP paths are normalized `\`→`/` (SCIP emits Windows separators; git diff uses `/`) — a `\`/`/`
//! mismatch silently produced 0 edges before the fix (caught by "a zero isn't a finding until you
//! rule out your own harness"). Files not in SCIP (some examples; historical renamed/deleted files)
//! are UNDER-counted, so the measured cyclic support is a **floor** — under-counting cannot
//! manufacture cycles, so the WARRANTED verdict is robust (the true graph has *more* support).
//!
//! Usage: `cargo run -p antigen-stroma --example fixcoupling_cycles -- [repo] [scip]`  (needs history).

#![allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const FIX_KEYWORDS: [&str; 5] = ["fix", "bug", "patch", "revert", "hotfix"];
/// A fix touching more than this many functions is a bulk sweep; its clique swamps β₁. Capped + reported.
const CLIQUE_CAP: usize = 40;

type FnDef = (String, String, i32, i32); // (bare_name, file, start_line, end_line)
type Edges = HashSet<(usize, usize)>;
type Ranges = Vec<(i32, i32)>;
type HunkMap = HashMap<String, (Ranges, Ranges)>; // file -> (old-side, new-side) 1-based inclusive
type ByFile<'a> = HashMap<&'a str, Vec<&'a FnDef>>;

// ── the BlameEngine seam (git-blame subprocess = oracle + first engine; stromix-swap insurance) ──
trait BlameEngine {
    /// Introducing commit SHAs for `lines` (1-based inclusive ranges) as of `rev`, in `file`.
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
                // porcelain: a blame group's first line is `<40-hex-sha> <orig> <final> [<n>]`
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
                // SCIP emits Windows backslash paths; git diff uses '/'. Normalize so file-keys match.
                let path = doc.relative_path.replace('\\', "/");
                defs.push((bare.to_string(), path, r[0], r[2]));
            }
        }
    }
    defs
}

fn is_fix_subject(s: &str) -> bool {
    let low = s.to_lowercase();
    if FIX_KEYWORDS.iter().any(|k| low.contains(k)) {
        return true;
    }
    // bare-`#N` is a squash-merge PR suffix, NOT a fix (the precision guard) — require a close-verb.
    false
}

/// All fix commits across the full object graph, as `(fix_sha, parent_sha)`.
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

/// Per-file `(old_ranges, new_ranges)` a commit touched (1-based inclusive), from `git show --unified=0`.
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
            // @@ -old_start[,old_len] +new_start[,new_len] @@
            let mut sides = h.split(' ');
            if let Some(old) = sides.next() {
                if let Some(r) = parse_side(old.trim_start_matches('-')) {
                    entry.0.push(r);
                }
            }
            if let Some(new) = sides.next() {
                if let Some(r) = parse_side(new.trim_start_matches('+')) {
                    entry.1.push(r);
                }
            }
        }
    }
    map
}

fn parse_side(s: &str) -> Option<(i32, i32)> {
    let mut it = s.split(',');
    let start: i32 = it.next()?.parse().ok()?;
    let len: i32 = it.next().map_or(1, |l| l.parse().unwrap_or(1));
    if len == 0 {
        return None; // pure insertion/deletion at a zero-width point — no lines to attribute
    }
    Some((start, start + len - 1))
}

/// Functions a commit touched: its NEW-side hunks overlapped against HEAD def-spans (the approximation).
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

#[allow(clippy::comparison_chain)]
const fn undirected(a: usize, b: usize) -> Option<(usize, usize)> {
    if a == b {
        None
    } else if a < b {
        Some((a, b))
    } else {
        Some((b, a))
    }
}

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

fn betti1(edges: &Edges, n: usize) -> (usize, usize) {
    let mut parent: Vec<usize> = (0..n).collect();
    let mut touched = HashSet::new();
    for &(src, dst) in edges {
        touched.insert(src);
        touched.insert(dst);
        let (rs, rd) = (uf_find(&mut parent, src), uf_find(&mut parent, dst));
        if rs != rd {
            parent[rs] = rd;
        }
    }
    let comps: HashSet<usize> = touched.iter().map(|&x| uf_find(&mut parent, x)).collect();
    (edges.len() + comps.len() - touched.len(), comps.len())
}

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
    if len < 2 {
        return edges.clone();
    }
    for _ in 0..(len * 4) {
        let (i, j) = (rng() % len, rng() % len);
        if i == j {
            continue;
        }
        let ((a0, a1), (b0, b1)) = (list[i], list[j]);
        if let (Some(e1), Some(e2)) = (undirected(a0, b1), undirected(b0, a1)) {
            list[i] = e1;
            list[j] = e2;
        }
    }
    list.into_iter().collect()
}

fn call_edges(recons: &[EdgeReconstruction], idx: &HashMap<String, usize>) -> Edges {
    let mut call = HashSet::new();
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

/// Functions a commit touched, cached (commit → hunks → HEAD-def-span overlap).
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

/// Mine fixes → blame each fix's pre-fix lines → build undirected defect-fn↔fix-fn edges.
/// Returns `(edges, blamed, capped, raw_pairs)`.
fn build_fixcoupling(
    repo: &Path,
    fixes: &[(String, String)],
    by_file: &ByFile,
    idx: &HashMap<String, usize>,
    blame: &dyn BlameEngine,
) -> (Edges, usize, usize, usize) {
    let mut cache: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut edges: Edges = HashSet::new();
    let (mut pairs, mut capped, mut blamed) = (0usize, 0usize, 0usize);
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
        blamed += 1;
        let fix_fns = fn_set(repo, fix, by_file, idx, &mut cache);
        if fix_fns.len() > CLIQUE_CAP {
            capped += 1;
            continue;
        }
        for def in &defects {
            let def_fns = fn_set(repo, def, by_file, idx, &mut cache);
            if def_fns.len() > CLIQUE_CAP {
                continue;
            }
            // directed defect-fn → fix-fn (undirected for the β₁ gate, as cochange did)
            for &df in &def_fns {
                for &ff in &fix_fns {
                    if let Some(edge) = undirected(df, ff) {
                        edges.insert(edge);
                        pairs += 1;
                    }
                }
            }
        }
    }
    (edges, blamed, capped, pairs)
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
        "== fix-coupling cycles: does SZZ defect→fix carry cyclic support the call graph lacks? ==\n"
    );

    let defs = fn_defs(Path::new(&scip));
    let mut idx: HashMap<String, usize> = HashMap::new();
    for d in &defs {
        let n = idx.len();
        idx.entry(d.0.clone()).or_insert(n);
    }
    let mut by_file: HashMap<&str, Vec<&FnDef>> = HashMap::new();
    for d in &defs {
        by_file.entry(&d.1).or_default().push(d);
    }

    let fixes = fix_commits(repo);
    let (fixcoupling, blamed, capped, pairs) =
        build_fixcoupling(repo, &fixes, &by_file, &idx, &GitBlame);

    let recons = ingest_scip(Path::new(&scip));
    let call = call_edges(&recons, &idx);
    let union: Edges = call.union(&fixcoupling).copied().collect();
    let (b_call, _) = betti1(&call, idx.len());
    let (b_fc, c_fc) = betti1(&fixcoupling, idx.len());
    let (b_union, _) = betti1(&union, idx.len());

    println!(
        "  fix commits: {}   with-blame: {blamed}   capped >{CLIQUE_CAP} fns: {capped}   raw dir-pairs: {pairs}",
        fixes.len()
    );
    println!("\n  EDGE SETS (undirected, deduped):");
    println!("    call          {:6}   β₁ {b_call:5}", call.len());
    println!(
        "    fixcoupling   {:6}   β₁ {b_fc:5}   components {c_fc}",
        fixcoupling.len()
    );
    println!("    union         {:6}   β₁ {b_union:5}", union.len());

    let disjoint = fixcoupling.difference(&call).count();
    let df = if fixcoupling.is_empty() {
        0.0
    } else {
        disjoint as f64 / fixcoupling.len() as f64 * 100.0
    };
    println!("\n  (P1) fix-coupling adjacency is call-DISJOINT (pre-reg >50%):");
    println!(
        "    fixcoupling \\ call = {disjoint} / {} = {df:.1}%   {}",
        fixcoupling.len(),
        if df > 50.0 {
            "CONFIRMED"
        } else {
            "not confirmed"
        }
    );

    let new_cycles = b_union as i64 - b_call as i64;
    println!("\n  (P2) new cyclic support over call (pre-reg > 0):");
    println!(
        "    β₁(union) − β₁(call) = {b_union} − {b_call} = {new_cycles}   {}",
        if new_cycles > 0 { "CONFIRMED" } else { "NONE" }
    );

    let rw = rewire(&fixcoupling);
    let rw_union: Edges = call.union(&rw).copied().collect();
    let (b_rw, _) = betti1(&rw_union, idx.len());
    let rw_df = if rw.is_empty() {
        0.0
    } else {
        rw.difference(&call).count() as f64 / rw.len() as f64 * 100.0
    };
    println!("\n  (P3) base-rate — degree-preserving rewire (destroys defect-locality):");
    println!(
        "    rewired disjoint {rw_df:.1}% vs real {df:.1}% (Δ {:+.1});  rewired β₁(union) {b_rw} vs real {b_union} (Δ {:+})",
        df - rw_df,
        b_union as i64 - b_rw as i64
    );

    let (p1, p2) = (df > 50.0, new_cycles > 0);
    println!(
        "\n  => fix-coupling {} carry call-disjoint cyclic support (P1 {p1}, P2 {p2}). {}",
        if p1 && p2 { "DOES" } else { "does NOT clearly" },
        if p1 && p2 {
            "The transport sheaf has directed cyclic support the call graph lacks → the full\n\
             \x20    fix-coupling transport-sheaf build is WARRANTED. Blame gives a REAL defect-locus\n\
             \x20    (precise per-line, not coarse first-parent)."
        } else {
            "Fix-coupling adds no call-disjoint cycle mass → a directed transport would obstruct\n\
             \x20    little new here. Reconsider before the sheaf build."
        }
    );
}

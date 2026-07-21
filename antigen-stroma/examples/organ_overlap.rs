//! Organ overlap — the first PER-ORGAN measurement (`substrate-node-identity` / nb 028 resolution).
//!
//! The theory: organs are content-signatures over the same containers, and they OVERLAP measurably. The
//! substrate is flat and near-DAG; the interesting structure is in how organ domains intersect. This computes
//! several content-defined organ domains on antigen's fn-defs and their overlap, base-rate controlled:
//!   - TOPOLOGY   — participates in ≥1 resolved Call edge (the SCIP-graph organ);
//!   - CORROBORATED — participates in ≥1 edge BOTH syn and SCIP assert (the agreement organ);
//!   - PHANTOM    — participates in ≥1 syn-only edge (the disagreement / F organ);
//!   - CHURN      — touched by ≥1 commit (the git organ); CURSED = churn ≥ the 90th percentile.
//!
//! PRE-REGISTERED (before the run): if these are real, independent organs whose overlap is structured (not
//! random), then (a) CURSED ∩ PHANTOM exceeds base rate — fragile code is where the instruments disagree —
//! and (b) CORROBORATED functions have LOWER mean churn — agreement marks the stable core. Two organs from
//! two unrelated instruments (git, syn/SCIP) correlating is structure the flat substrate cannot show.
//!
//! Usage: `cargo run -p antigen-stroma --example organ_overlap -- [repo] [scip] [src ...]`  (needs history).

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use antigen_stroma::syn_ingest::{bare_fn_name, syn_call_pairs_from_dir};
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const ANTIGEN_SRC: &[&str] = &[
    "antigen/src",
    "antigen-stroma/src",
    "antigen-fingerprint/src",
    "antigen-macros/src",
    "antigen-attestation/src",
    "cargo-antigen/src",
];

/// One fn-def's identity + location: `(bare_name, file, start_line, end_line)`.
type FnDef = (String, String, i32, i32);

/// Every fn-def as `(bare, file, start, end)`, and the `bare -> #scip-def-symbols` collision classes.
fn fn_defs(scip: &Path) -> (Vec<FnDef>, HashMap<String, usize>) {
    let (mut defs, mut classes): (Vec<FnDef>, HashMap<String, usize>) =
        (Vec::new(), HashMap::new());
    let Ok(bytes) = std::fs::read(scip) else {
        return (defs, classes);
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return (defs, classes);
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) == 0 {
                continue;
            }
            let Some(bare) = bare_fn_name(&occ.symbol) else {
                continue;
            };
            *classes.entry(bare.to_string()).or_insert(0) += 1;
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
    (defs, classes)
}

/// Per-file changed-line hunks (new side) per commit, from `git log --unified=0`.
fn file_hunks(repo: &Path, file: &str) -> Vec<Vec<(i32, i32)>> {
    let Ok(out) = Command::new("git")
        .args([
            "-C",
            &repo.to_string_lossy(),
            "log",
            "--no-merges",
            "--unified=0",
            "--format=%x00",
            "--",
            file,
        ])
        .output()
    else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut commits: Vec<Vec<(i32, i32)>> = Vec::new();
    for line in text.lines() {
        if line.starts_with('\0') {
            commits.push(Vec::new());
        } else if let Some(h) = line.strip_prefix("@@") {
            if let Some(plus) = h.split('+').nth(1) {
                let mut it = plus.split('@').next().unwrap_or("").trim().split(',');
                if let Some(c) = it.next().and_then(|s| s.trim().parse::<i32>().ok()) {
                    let d = it
                        .next()
                        .and_then(|s| s.trim().parse::<i32>().ok())
                        .unwrap_or(1);
                    if let Some(cur) = commits.last_mut() {
                        cur.push((c, c + d.max(1)));
                    }
                }
            }
        }
    }
    commits
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

/// Per-fn churn = distinct commits whose hunks overlap the fn's def-span.
fn compute_churn(repo: &Path, defs: &[FnDef]) -> HashMap<String, usize> {
    let mut by_file: HashMap<&str, Vec<&FnDef>> = HashMap::new();
    for d in defs {
        by_file.entry(&d.1).or_default().push(d);
    }
    let mut churn: HashMap<String, usize> = HashMap::new();
    for (file, fns) in &by_file {
        for hunks in &file_hunks(repo, file) {
            for d in fns {
                if hunks.iter().any(|&(cs, ce)| cs <= d.3 + 1 && ce > d.2) {
                    *churn.entry(d.0.clone()).or_default() += 1;
                }
            }
        }
    }
    churn
}

fn mean_churn(s: &HashSet<String>, churn: &HashMap<String, usize>) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    s.iter()
        .map(|k| churn.get(k).copied().unwrap_or(0))
        .sum::<usize>() as f64
        / s.len() as f64
}

/// The two pre-registered tests + the domain census.
fn report(
    u: usize,
    doms: &[(&str, &HashSet<String>)],
    cursed: &HashSet<String>,
    phantom: &HashSet<String>,
    corroborated: &HashSet<String>,
    touched: &HashSet<String>,
    churn: &HashMap<String, usize>,
) {
    println!("  universe (fn-def bare names): {u}\n  ORGAN DOMAINS:");
    for (name, s) in doms {
        println!("    {name:<12} {:5}  ({:.1}%)", s.len(), pct(s.len(), u));
    }
    let inter = cursed.iter().filter(|k| phantom.contains(*k)).count();
    let p_phantom = phantom.len() as f64 / u as f64;
    let p_cond = if cursed.is_empty() {
        0.0
    } else {
        inter as f64 / cursed.len() as f64
    };
    let lift = if p_phantom > 0.0 {
        p_cond / p_phantom
    } else {
        0.0
    };
    println!("\n  (a) FRAGILE-IS-DISAGREEMENT (pre-registered lift > 1.3):");
    println!(
        "    P(phantom) {p_phantom:.3}   P(phantom|cursed) {p_cond:.3}  ({inter}/{} cursed)",
        cursed.len()
    );
    println!(
        "    => lift {lift:.2}  {}",
        if lift > 1.3 {
            "CONFIRMED: fragile code is where instruments disagree."
        } else {
            "not confirmed at 1.3."
        }
    );
    let non_corr: HashSet<String> = touched.difference(corroborated).cloned().collect();
    let (mc, mn) = (
        mean_churn(corroborated, churn),
        mean_churn(&non_corr, churn),
    );
    println!("\n  (b) AGREEMENT-IS-STABLE (pre-registered: corroborated churn < rest):");
    println!(
        "    mean churn corroborated {mc:.2}  vs  non-corroborated {mn:.2}  {}",
        if mc < mn {
            "CONFIRMED."
        } else {
            "not confirmed."
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
    let src: Vec<String> = {
        let r: Vec<String> = std::env::args().skip(3).collect();
        if r.is_empty() {
            ANTIGEN_SRC.iter().map(|s| (*s).to_string()).collect()
        } else {
            r
        }
    };
    let repo = Path::new(&repo);
    println!("== organ overlap: content-defined organ domains and their intersection ==\n");

    let (defs, _classes) = fn_defs(Path::new(&scip));
    let recons = ingest_scip(Path::new(&scip));

    // scip Call edges (bare-name), and per-node membership in a Call edge.
    let mut scip_bare: HashSet<(String, String)> = HashSet::new();
    let mut topology: HashSet<String> = HashSet::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                if let (Some(u), Some(v)) =
                    (bare_fn_name(&e.src_symbol), bare_fn_name(&e.dst_symbol))
                {
                    topology.insert(u.to_string());
                    topology.insert(v.to_string());
                    scip_bare.insert((u.to_string(), v.to_string()));
                }
            }
        }
    }
    // syn edges → CORROBORATED (both assert) vs PHANTOM (syn-only) node membership.
    let syn = syn_call_pairs_from_dir(&src);
    let (mut corroborated, mut phantom) = (HashSet::<String>::new(), HashSet::<String>::new());
    for (u, v) in &syn {
        let both = scip_bare.contains(&(u.clone(), v.clone()))
            || scip_bare.contains(&(v.clone(), u.clone()));
        let set = if both {
            &mut corroborated
        } else {
            &mut phantom
        };
        set.insert(u.clone());
        set.insert(v.clone());
    }
    // CHURN per bare name (git hunks × def spans), and the CURSED cutoff at P90.
    let churn = compute_churn(repo, &defs);
    let mut cvals: Vec<usize> = churn.values().copied().collect();
    cvals.sort_unstable();
    let p90 = cvals.get(cvals.len() * 9 / 10).copied().unwrap_or(1).max(2);
    let cursed: HashSet<String> = churn
        .iter()
        .filter(|(_, &c)| c >= p90)
        .map(|(k, _)| k.clone())
        .collect();
    let touched: HashSet<String> = churn.keys().cloned().collect();

    let universe: HashSet<String> = defs.iter().map(|d| d.0.clone()).collect();
    let doms: Vec<(&str, &HashSet<String>)> = vec![
        ("topology", &topology),
        ("corroborated", &corroborated),
        ("phantom", &phantom),
        ("churn>0", &touched),
        ("cursed>=P90", &cursed),
    ];
    report(
        universe.len(),
        &doms,
        &cursed,
        &phantom,
        &corroborated,
        &touched,
        &churn,
    );

    println!(
        "\n  => two organs from two UNRELATED instruments (git, syn/SCIP) correlating is structure the\n\
         \x20    flat substrate cannot show -- the first per-organ (content-signature) measurement."
    );
}

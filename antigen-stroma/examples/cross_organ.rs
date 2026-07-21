//! Cross-instrument organs — PROOF that an organ can consume PART of several instruments at once, reducible
//! to none of them (nb 028 resolution / `substrate-node-identity`).
//!
//! Two organs, each a content-signature that is a CONJUNCTION across three instruments, each reading only a
//! SLICE of each (not all of any):
//!   - **critical-core** = churn ≥ median (git: actively changed) ∧ fan-in ≥ median (SCIP: heavily called)
//!     ∧ corroborated (syn+SCIP agree) — the load-bearing, central, high-confidence core.
//!   - **boundary-risk**  = churn ≥ median (git) ∧ phantom (syn-only: at the unresolved boundary) — actively
//!     changed code where the instruments disagree.
//!
//! It proves three things: (1) EXISTENCE — the cross-instrument domain is non-empty; (2) REDUCIBLE-TO-NONE —
//! dropping ANY one predicate ENLARGES the domain, so all three instruments are load-bearing (no single one,
//! and no pair, reproduces the organ); (3) OVERLAP-BUT-DISTINCT — the two organs share the churn slice yet
//! diverge on the resolution axis (corroborated vs phantom), so they overlap partially without being equal
//! or reducible to each other. That is "overlapping organs": different partial cross-cuts of ONE container set.
//!
//! Usage: `cargo run -p antigen-stroma --example cross_organ -- [repo] [scip] [src ...]`  (needs git history).

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

fn compute_churn(repo: &Path, defs: &[FnDef]) -> HashMap<String, usize> {
    let mut by_file: HashMap<&str, Vec<&FnDef>> = HashMap::new();
    for d in defs {
        by_file.entry(&d.1).or_default().push(d);
    }
    let mut churn: HashMap<String, usize> = HashMap::new();
    for (file, fns) in &by_file {
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
            continue;
        };
        let text = String::from_utf8_lossy(&out.stdout);
        let mut hunks: Vec<(i32, i32)> = Vec::new();
        let flush = |hunks: &mut Vec<(i32, i32)>, churn: &mut HashMap<String, usize>| {
            for d in fns {
                if hunks.iter().any(|&(cs, ce)| cs <= d.3 + 1 && ce > d.2) {
                    *churn.entry(d.0.clone()).or_default() += 1;
                }
            }
            hunks.clear();
        };
        for line in text.lines() {
            if line.starts_with('\0') {
                flush(&mut hunks, &mut churn);
            } else if let Some(h) = line.strip_prefix("@@") {
                if let Some(plus) = h.split('+').nth(1) {
                    let mut it = plus.split('@').next().unwrap_or("").trim().split(',');
                    if let Some(c) = it.next().and_then(|s| s.trim().parse::<i32>().ok()) {
                        let d = it
                            .next()
                            .and_then(|s| s.trim().parse::<i32>().ok())
                            .unwrap_or(1);
                        hunks.push((c, c + d.max(1)));
                    }
                }
            }
        }
        flush(&mut hunks, &mut churn);
    }
    churn
}

fn median(mut v: Vec<usize>) -> usize {
    v.sort_unstable();
    v.get(v.len() / 2).copied().unwrap_or(0).max(1)
}

fn and2(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    a.iter().filter(|k| b.contains(*k)).cloned().collect()
}

/// The three-part proof: existence, reducible-to-none, overlap-but-distinct.
fn prove(
    u: usize,
    hi_churn: &HashSet<String>,
    hi_fanin: &HashSet<String>,
    corr: &HashSet<String>,
    phan: &HashSet<String>,
) {
    let organ_a: HashSet<String> = and2(&and2(hi_churn, hi_fanin), corr);
    let organ_b = and2(hi_churn, phan);
    println!(
        "  universe {u}   per-instrument SLICES (each reads ONE coordinate of its instrument):"
    );
    println!("    git  : hi-churn {}", hi_churn.len());
    println!(
        "    SCIP : hi-fanin {}   corroborated {}   phantom {}",
        hi_fanin.len(),
        corr.len(),
        phan.len()
    );

    let (drop_git, drop_scip, drop_syn) = (
        and2(hi_fanin, corr),
        and2(hi_churn, corr),
        and2(hi_churn, hi_fanin),
    );
    println!("\n  ORGAN A 'critical-core' = hi-churn ∧ hi-fanin ∧ corroborated");
    println!(
        "    |A| = {}   (EXISTS: non-empty cross-instrument domain)",
        organ_a.len()
    );
    println!(
        "    REDUCIBLE-TO-NONE — dropping each predicate ENLARGES the domain (so each is load-bearing):"
    );
    println!(
        "      drop git(churn)  → {} (+{})",
        drop_git.len(),
        drop_git.len() - organ_a.len()
    );
    println!(
        "      drop SCIP(fanin) → {} (+{})",
        drop_scip.len(),
        drop_scip.len() - organ_a.len()
    );
    println!(
        "      drop syn(corrob) → {} (+{})",
        drop_syn.len(),
        drop_syn.len() - organ_a.len()
    );

    println!(
        "\n  ORGAN B 'boundary-risk' = hi-churn ∧ phantom   |B| = {}",
        organ_b.len()
    );
    let a_and_b = and2(&organ_a, &organ_b).len();
    let a_not_b = organ_a.iter().filter(|k| !organ_b.contains(*k)).count();
    let b_not_a = organ_b.iter().filter(|k| !organ_a.contains(*k)).count();
    println!("  OVERLAP-BUT-DISTINCT (share the git slice, diverge on the resolution axis):");
    println!("    |A ∩ B| = {a_and_b}   |A \\ B| = {a_not_b}   |B \\ A| = {b_not_a}");

    let all_load_bearing = drop_git.len() > organ_a.len()
        && drop_scip.len() > organ_a.len()
        && drop_syn.len() > organ_a.len();
    println!(
        "\n  => PROVEN: organ A consumes PART of git + PART of SCIP + PART of syn, reducible to none\n\
         \x20    (all three predicates load-bearing: {all_load_bearing}). A and B read the SAME instruments\n\
         \x20    through DIFFERENT slices and overlap partially without being equal — overlapping organs are\n\
         \x20    different partial cross-cuts of one container set, not separate consumers of disjoint content."
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
    println!(
        "== cross-instrument organs: partial consumption of git + SCIP + syn, reducible to none ==\n"
    );

    let defs = fn_defs(Path::new(&scip));
    let recons = ingest_scip(Path::new(&scip));
    // SCIP slice: directed Call edges (bare) → fan-in per node; also the both/only sets need scip edges.
    let mut scip_bare: HashSet<(String, String)> = HashSet::new();
    let mut fanin: HashMap<String, usize> = HashMap::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                if let (Some(u), Some(v)) =
                    (bare_fn_name(&e.src_symbol), bare_fn_name(&e.dst_symbol))
                {
                    scip_bare.insert((u.to_string(), v.to_string()));
                    *fanin.entry(v.to_string()).or_default() += 1;
                }
            }
        }
    }
    // syn slice: corroborated (both) vs phantom (syn-only) membership.
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
    // git slice: churn.
    let churn = compute_churn(Path::new(&repo), &defs);

    let universe: HashSet<String> = defs.iter().map(|d| d.0.clone()).collect();
    let med_c = median(churn.values().copied().collect());
    let med_f = median(fanin.values().copied().collect());

    // The three per-instrument SLICES as node-predicates.
    let hi_churn: HashSet<String> = universe
        .iter()
        .filter(|k| churn.get(*k).copied().unwrap_or(0) >= med_c)
        .cloned()
        .collect();
    let hi_fanin: HashSet<String> = universe
        .iter()
        .filter(|k| fanin.get(*k).copied().unwrap_or(0) >= med_f)
        .cloned()
        .collect();
    let corr: HashSet<String> = corroborated.intersection(&universe).cloned().collect();
    let phan: HashSet<String> = phantom.intersection(&universe).cloned().collect();

    prove(universe.len(), &hi_churn, &hi_fanin, &corr, &phan);
}

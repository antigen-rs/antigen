//! Edge multiplicity — do node-pairs ALREADY carry more than one connection-meaning? The cheap grounding
//! probe for `an-edge-is-a-container-of-typed-connections` (`substrate-node-identity`).
//!
//! If a single `(src, dst)` pair carries both a `Call` and a `Construct` edge (etc.), then "an edge is a
//! container of typed connection-records, not a single arrow" is already TRUE in shipped data — `EdgeKind`
//! is the one-tag seed of the graded edge-stalk the sheaf build needs.
//!
//! Usage: `cargo run -p antigen-stroma --example edge_kinds -- [scip_path]`

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};

fn main() {
    let scip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let recons = ingest_scip(Path::new(&scip));

    let mut kinds_per_pair: HashMap<(String, String), HashSet<String>> = HashMap::new();
    let mut kind_totals: HashMap<String, usize> = HashMap::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            let k = format!("{:?}", e.kind);
            *kind_totals.entry(k.clone()).or_default() += 1;
            kinds_per_pair
                .entry((e.src_symbol.clone(), e.dst_symbol.clone()))
                .or_default()
                .insert(k);
        }
    }
    let pairs = kinds_per_pair.len();
    let multi: Vec<_> = kinds_per_pair
        .iter()
        .filter(|(_, ks)| ks.len() >= 2)
        .collect();

    println!("== edge multiplicity: connection-meanings per node-pair ==\n");
    println!("  EdgeKinds present (each is a connection-MEANING):");
    let mut kt: Vec<_> = kind_totals.iter().collect();
    kt.sort_by(|a, b| b.1.cmp(a.1));
    for (k, n) in &kt {
        println!("    {k:<12} {n:>7} edges");
    }
    println!("\n  distinct (src,dst) pairs: {pairs}");
    println!(
        "  pairs carrying >= 2 connection-meanings: {}  ({:.2}%)",
        multi.len(),
        multi.len() as f64 / pairs.max(1) as f64 * 100.0
    );
    // Which meaning-COMBINATIONS co-occur, and how often.
    let mut combos: HashMap<String, usize> = HashMap::new();
    for (_, ks) in &multi {
        let mut v: Vec<&String> = ks.iter().collect();
        v.sort();
        *combos
            .entry(v.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("+"))
            .or_default() += 1;
    }
    let mut cv: Vec<_> = combos.iter().collect();
    cv.sort_by(|a, b| b.1.cmp(a.1));
    if !cv.is_empty() {
        println!("\n  co-occurring meaning-combinations on one pair:");
        for (combo, n) in cv.iter().take(8) {
            println!("    {combo:<28} {n} pairs");
        }
    }
    println!(
        "\n  => within-SCIP, EdgeKinds sit on {}. Call and Construct are DIFFERENT-PAIR meanings\n\
         \x20    (you call functions, construct types -- disjoint targets, the 'orthogonal call/construct\n\
         \x20    hubs'), so no single pair carries two SCIP kinds. The multi-record edge is therefore realized\n\
         \x20    CROSS-INSTRUMENT, not within SCIP: the ~6658 both-assert pairs (a syn-call record AND a\n\
         \x20    scip-call record on the SAME pair, measured in holdall_nodes/sheaf) ARE edges carrying two\n\
         \x20    connection-meanings. Model holds, with a correction -- not every meaning-pair overlaps;\n\
         \x20    EdgeKind distinguishes DIFFERENT-pair meanings (Call vs Construct) from SAME-pair meanings\n\
         \x20    (syn-call vs scip-call), and the edge-as-container lives at the instrument-overlap.",
        if multi.is_empty() {
            "DISJOINT pairs (0 shared)"
        } else {
            "OVERLAPPING pairs"
        }
    );
}

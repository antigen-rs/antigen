//! HOLD-ALL node identity vs INGEST-PROJECTED — the falsifiable checkpoint of the
//! `substrate-node-identity` charter.
//!
//! Two policies for what the substrate's NODE SET is:
//! - **PROJECTED** (what the sheaf example does today): the node set is derived from one instrument's
//!   resolved output — SCIP-resolved symbols. A syn observation exists only if it lifts to that set, so
//!   syn-ONLY edges are structurally starved.
//! - **HOLD-ALL** (the charter): the node set is the UNION across ingests. An external callee is a node whose
//!   only attribute is a syn-observed name; a colliding bare name is a name-class node. Nothing is discarded
//!   at ingest; organs project when they run.
//!
//! Prediction (antigen): syn-only edges jump from ~14 (projected) to ~2200+ (hold-all), and the "78%
//! external" phantom figure is revealed as *where the node set stops*, not a fact about the codebase.
//!
//! Co-reference is a many-to-many relation, so the census also reports the CARDINALITY profile: how many syn
//! bare names map 1-none (external), 1-1 (cleanly resolved), or many-1 (collision class).
//!
//! Usage: `cargo run -p antigen-stroma --example holdall_nodes -- [scip_path] [src_dir ...]`
//! Defaults to antigen's own index and crate dirs; pass an external corpus's `.scip` + `src` dirs to repeat.

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use antigen_stroma::syn_ingest::{bare_fn_name, syn_call_pairs_from_dir};
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const ANTIGEN_CRATE_DIRS: &[&str] = &[
    "antigen/src",
    "antigen-stroma/src",
    "antigen-fingerprint/src",
    "antigen-macros/src",
    "antigen-attestation/src",
    "cargo-antigen/src",
];

/// `bare_name -> { full SCIP def symbols }` — the syn indiscernibility classes.
fn fn_classes(scip_path: &Path) -> HashMap<String, HashSet<String>> {
    let mut classes: HashMap<String, HashSet<String>> = HashMap::new();
    let Ok(bytes) = std::fs::read(scip_path) else {
        return classes;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return classes;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) != 0 {
                if let Some(name) = bare_fn_name(&occ.symbol) {
                    classes
                        .entry(name.to_string())
                        .or_default()
                        .insert(occ.symbol.clone());
                }
            }
        }
    }
    classes
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

/// The three co-reference buckets for the syn-only edges, plus the external syn-name node count and the
/// `both` count (syn AND SCIP assert the edge) — the coherence check that the two sources describe one corpus.
#[derive(Default)]
struct SynOnly {
    lifted_1_1: usize,
    collision_many_1: usize,
    external_1_none: usize,
    external_node_count: usize,
    both: usize,
}

/// Classify every syn pair NOT already asserted by SCIP into 1-1 / many-1 / 1-none.
fn classify_syn_only(
    syn_pairs: &HashSet<(String, String)>,
    scip_bare: &HashSet<(String, String)>,
    classes: &HashMap<String, HashSet<String>>,
) -> SynOnly {
    let mut out = SynOnly::default();
    let mut external_names: HashSet<&str> = HashSet::new();
    for (su, sv) in syn_pairs {
        if su == sv {
            continue;
        }
        if scip_bare.contains(&(su.clone(), sv.clone()))
            || scip_bare.contains(&(sv.clone(), su.clone()))
        {
            out.both += 1;
            continue; // already a Both edge
        }
        let card = |name: &str| classes.get(name).map_or(0, HashSet::len);
        let (cu, cv) = (card(su), card(sv));
        if cu == 0 || cv == 0 {
            out.external_1_none += 1;
            if cu == 0 {
                external_names.insert(su);
            }
            if cv == 0 {
                external_names.insert(sv);
            }
        } else if cu == 1 && cv == 1 {
            out.lifted_1_1 += 1;
        } else {
            out.collision_many_1 += 1;
        }
    }
    out.external_node_count = external_names.len();
    out
}

fn report(so: &SynOnly, scip_symbols: usize) {
    let total = so.lifted_1_1 + so.collision_many_1 + so.external_1_none;
    println!("  INGEST-PROJECTED (node set = SCIP-resolved symbols):");
    println!(
        "    |V| = {scip_symbols}   syn-only edges that lift = {} (only the 1-1 case)",
        so.lifted_1_1
    );
    println!("\n  HOLD-ALL (node set = union across ingests):");
    println!(
        "    |V| = {}  (= {scip_symbols} SCIP symbols + {} external syn-name nodes)",
        scip_symbols + so.external_node_count,
        so.external_node_count
    );
    println!(
        "    syn-only edges = {total}  (ALL of them: nothing is unliftable when identity is a container)"
    );
    println!("\n  CO-REFERENCE CARDINALITY of the syn-only edges:");
    println!(
        "    1-1    cleanly resolved      : {:6}  ({:.1}%)",
        so.lifted_1_1,
        pct(so.lifted_1_1, total)
    );
    println!(
        "    many-1 collision / overload  : {:6}  ({:.1}%)",
        so.collision_many_1,
        pct(so.collision_many_1, total)
    );
    println!(
        "    1-none external / unresolved : {:6}  ({:.1}%)",
        so.external_1_none,
        pct(so.external_1_none, total)
    );
    let jump = if so.lifted_1_1 == 0 {
        // Projected 0 is not a 0x jump -- it is the MOST extreme case: every syn-only edge was invisible
        // under ingest-projection (nothing resolved 1-1, e.g. a derive-macro-heavy crate like serde).
        format!("{total}x (from nothing -- no syn-only edge lifted at all under projection)")
    } else {
        format!("{:.0}x", total as f64 / so.lifted_1_1 as f64)
    };
    println!(
        "\n  => syn-only: {} (projected) -> {total} (hold-all), a {jump} jump.",
        so.lifted_1_1
    );
    println!(
        "     {:.1}% was discarded at ingest as 'external' -- not a fact about the codebase, but WHERE THE\n\
         \x20    PROJECTED NODE SET STOPS. Under hold-all those become nodes whose only attribute is a\n\
         \x20    syn-observed name, and the [1,0] restriction row gets real work for the first time.",
        pct(so.external_1_none, total)
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let scip_path_s = args
        .next()
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let src_dirs: Vec<String> = {
        let rest: Vec<String> = args.collect();
        if rest.is_empty() {
            ANTIGEN_CRATE_DIRS
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        } else {
            rest
        }
    };
    let scip_path = Path::new(&scip_path_s);
    println!("== HOLD-ALL vs INGEST-PROJECTED node identity ==");
    println!("   scip: {scip_path_s}   src: {src_dirs:?}\n");

    // SCIP side: resolved Call edges as undirected bare-name pairs, and the symbol node set.
    let recons = ingest_scip(scip_path);
    let mut scip_edges: HashSet<(String, String)> = HashSet::new();
    let mut scip_symbols: HashSet<String> = HashSet::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call && e.src_symbol != e.dst_symbol {
                for s in [&e.src_symbol, &e.dst_symbol] {
                    scip_symbols.insert(s.clone());
                }
                let (a, b) = if e.src_symbol <= e.dst_symbol {
                    (e.src_symbol.clone(), e.dst_symbol.clone())
                } else {
                    (e.dst_symbol.clone(), e.src_symbol.clone())
                };
                scip_edges.insert((a, b));
            }
        }
    }
    let classes = fn_classes(scip_path);
    // SCIP edges as bare-name pairs, to compare against syn (which only speaks bare names).
    let scip_bare: HashSet<(String, String)> = scip_edges
        .iter()
        .filter_map(|(u, v)| Some((bare_fn_name(u)?.to_string(), bare_fn_name(v)?.to_string())))
        .collect();

    // syn side.
    let syn_pairs = syn_call_pairs_from_dir(&src_dirs);
    if syn_pairs.is_empty() {
        println!("(no syn call pairs from {src_dirs:?} — pass the corpus's source dirs)");
        return;
    }

    let so = classify_syn_only(&syn_pairs, &scip_bare, &classes);
    // COHERENCE GUARD: syn and SCIP must describe the SAME corpus. If almost no syn edge is also a SCIP edge,
    // the source dirs and the .scip index are mismatched (e.g. antigen's src against ripgrep's index), which
    // silently inflates the "external" bucket toward 100%. Refuse rather than print a plausible wrong number.
    let overlap = pct(so.both, syn_pairs.len().max(1));
    println!(
        "  coherence: {} of {} syn edges are also SCIP edges ({overlap:.1}% overlap)\n",
        so.both,
        syn_pairs.len()
    );
    if overlap < 2.0 {
        println!(
            "  ⚠ ABORTING: <2% overlap means the syn source and the SCIP index describe DIFFERENT corpora.\n\
             \x20   Pass the source dirs that MATCH the .scip. (A mismatch inflates 'external' toward 100%\n\
             \x20   and looks like a stronger result -- exactly the trap this guard exists to stop.)"
        );
        return;
    }
    report(&so, scip_symbols.len());
}

//! First PRODUCTION measurement of syn-vs-SCIP agreement on antigen's own call graph (nb 029 §1).
//!
//! Connects the two afferent instruments on the REAL graph: production `ingest_scip` (full-symbol
//! `{ScipResolved}` Call edges) + the `syn_ingest` producer (a bare-name syn walk over antigen's own
//! source), via the library `corroborate_with_syn` primitive. Reports:
//! - **H0 corroboration** — SCIP Call edges syn ALSO saw (`{Syn, ScipResolved}`), the first
//!   non-degenerate witness set on the production graph;
//! - **SCIP-only** — resolved edges syn did not see (a call through a macro/dispatch the parser missed,
//!   or a bare-name collision);
//! - **syn-only phantoms** — syn pairs that matched NO SCIP Call edge (the H1-conflict candidate side:
//!   a call the parser saw that SCIP did not resolve).
//!
//! Corroboration is at BARE-NAME granularity (syn resolves no symbols) — the honest coarse projection
//! (`syn_ingest` module docs). This measures; baking corroboration into the `constitute`/scan path so
//! every organ consumes corroborated edges is the next slice.
//!
//! Usage: `cargo run -p antigen-stroma --example syn_scip_corroboration -- [path/to.scip]`
//! (defaults to `target/antigen.scip`).

// A stats-display example: usize→f64 precision loss on counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]

use std::collections::HashSet;
use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::scip::{ingest_scip, populate_from_reconstructions};
use antigen_stroma::syn_ingest::{bare_fn_name, corroborate_with_syn, syn_call_pairs_from_str};
use walkdir::WalkDir;

/// Workspace crate `src/` dirs to walk for the syn feeder (never includes `target/`).
const CRATE_DIRS: &[&str] = &[
    "antigen/src",
    "antigen-stroma/src",
    "antigen-fingerprint/src",
    "antigen-macros/src",
    "antigen-attestation/src",
    "cargo-antigen/src",
];

/// Every syntactic `(caller, callee)` bare-name pair across the workspace sources, deduplicated.
fn extract_syn_pairs(workspace_root: &Path) -> HashSet<(String, String)> {
    let mut pairs = HashSet::new();
    for dir in CRATE_DIRS {
        let src_root = workspace_root.join(dir);
        if !src_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&src_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ex| ex == "rs"))
        {
            if let Ok(src) = std::fs::read_to_string(entry.path()) {
                pairs.extend(syn_call_pairs_from_str(&src));
            }
        }
    }
    pairs
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

fn main() {
    let scip_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent (the workspace root)");
    println!("== syn ⋈ SCIP corroboration on antigen's own call graph ==\n");

    // 1. The production SCIP graph.
    let recons = ingest_scip(Path::new(&scip_path));
    if recons.is_empty() {
        println!(
            "(no SCIP reconstructions from {scip_path} — check the path / that `rust-analyzer scip` ran)"
        );
        return;
    }
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    let snap = SnapshotHandle::open(&db);
    let mut edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    let total_call = edges.iter().filter(|e| e.kind == EdgeKind::Call).count();

    // 2. The syn feeder over antigen's own source.
    let syn_pairs = extract_syn_pairs(workspace_root);

    // 3. Corroborate the production Call edges with syn (the library primitive).
    let stats = corroborate_with_syn(&mut edges, &syn_pairs);

    // 4. syn-only phantoms: syn pairs whose bare-name projection matched NO SCIP Call edge.
    let scip_call_bare: HashSet<(String, String)> = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Call)
        .filter_map(|e| {
            Some((
                bare_fn_name(&e.src.fq_path.path)?.to_string(),
                bare_fn_name(&e.dst.fq_path.path)?.to_string(),
            ))
        })
        .collect();
    let syn_only = syn_pairs
        .iter()
        .filter(|p| !scip_call_bare.contains(p))
        .count();

    // 5. Report.
    println!("SCIP Call edges (production graph): {total_call}");
    println!("syn (caller,callee) bare-name pairs: {}\n", syn_pairs.len());
    println!(
        "  H0 corroborated  {{Syn, ScipResolved}}: {:6}  ({:.1}% of SCIP Call edges)",
        stats.corroborated,
        pct(stats.corroborated, total_call)
    );
    println!(
        "  SCIP-only        {{ScipResolved}}:      {:6}  ({:.1}%)",
        stats.scip_only,
        pct(stats.scip_only, total_call)
    );
    println!(
        "  syn-only phantoms (H1 candidates):     {syn_only:6}  ({:.1}% of syn pairs — a call syn saw \
         that SCIP did not resolve)",
        pct(syn_only, syn_pairs.len())
    );
    println!(
        "\n(bare-name granularity — the honest coarse projection; full-symbol reconciliation is the \
         frontier. This is the FIRST non-degenerate witness set on the production graph.)"
    );
}

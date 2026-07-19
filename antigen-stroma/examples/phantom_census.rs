//! Phantom census — what ARE the syn-only "conflict" candidates? (F-axis discovery, nb 029 §5)
//!
//! The corroboration measurement found ~28% of syn call pairs are phantoms: syn saw the call, SCIP had
//! no matching resolved `Call` edge. But **phantom ≠ conflict.** A call to a `std`/dependency function is
//! syn seeing *past antigen's own boundary* — SCIP's antigen-scoped graph legitimately has no edge for a
//! target that isn't an antigen symbol. That's *out of scope*, neither resolved-absence (F) nor ignorance
//! (I). This census splits the phantoms by whether the callee name is even *defined* anywhere in antigen:
//! - **external** — callee name never defined in antigen → a call leaving antigen's universe (not a conflict);
//! - **in-scope** — callee name IS an antigen fn → the real conflict / F candidates (cfg-branch, macro,
//!   genuine SCIP-miss, or bare-name collision).
//!
//! Measure before identity: this tells us how much of the 28% is *genuine* in-antigen disagreement worth
//! designing F around, vs syn simply reaching past the boundary.
//!
//! Usage: `cargo run -p antigen-stroma --example phantom_census -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::scip::{ingest_scip, populate_from_reconstructions};
use antigen_stroma::syn_ingest::{bare_fn_name, syn_call_pairs_from_dir};
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const CRATE_DIRS: &[&str] = &[
    "antigen/src",
    "antigen-stroma/src",
    "antigen-fingerprint/src",
    "antigen-macros/src",
    "antigen-attestation/src",
    "cargo-antigen/src",
];

/// Bare names of every callable (method/fn) DEFINITION in the SCIP index — the set of antigen functions
/// a call could resolve to. Non-callable defs (types, modules) reduce to `None` and are skipped.
fn antigen_fn_names(scip_path: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    let Ok(bytes) = std::fs::read(scip_path) else {
        return names;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return names;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if is_def {
                if let Some(name) = bare_fn_name(&occ.symbol) {
                    names.insert(name.to_string());
                }
            }
        }
    }
    names
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let scip_path = Path::new(&scip_path_s);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent (the workspace root)");
    println!("== Phantom census: what ARE the syn-only conflict candidates? ==\n");

    // SCIP graph → resolved Call bare-pairs.
    let recons = ingest_scip(scip_path);
    if recons.is_empty() {
        println!("(no SCIP reconstructions from {scip_path_s})");
        return;
    }
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    let snap = SnapshotHandle::open(&db);
    let edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
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

    // syn call pairs over live source; phantoms = syn saw, SCIP has no matching bare Call edge.
    let src_roots: Vec<PathBuf> = CRATE_DIRS.iter().map(|d| workspace_root.join(d)).collect();
    let syn_pairs = syn_call_pairs_from_dir(&src_roots);
    let phantoms: Vec<&(String, String)> = syn_pairs
        .iter()
        .filter(|p| !scip_call_bare.contains(p))
        .collect();

    // Split by whether the callee name is defined anywhere in antigen.
    let antigen_fns = antigen_fn_names(scip_path);
    let (mut external, mut in_scope) = (0usize, 0usize);
    let mut in_scope_samples: Vec<&(String, String)> = Vec::new();
    let mut callee_counts: HashMap<&str, usize> = HashMap::new();
    for p in &phantoms {
        *callee_counts.entry(p.1.as_str()).or_default() += 1;
        if antigen_fns.contains(&p.1) {
            in_scope += 1;
            if in_scope_samples.len() < 20 {
                in_scope_samples.push(p);
            }
        } else {
            external += 1;
        }
    }

    println!("total syn-only phantoms: {}\n", phantoms.len());
    println!(
        "  EXTERNAL target (callee never DEFINED in antigen — a call LEAVING antigen's universe, e.g.\n\
         \x20   std/dep methods; NOT a conflict, out of scope):        {external:5}  ({:.1}%)",
        pct(external, phantoms.len())
    );
    println!(
        "  IN-SCOPE (callee IS an antigen fn — the real conflict / F candidates: cfg-branch, macro,\n\
         \x20   genuine SCIP-miss, or bare-name collision):            {in_scope:5}  ({:.1}%)",
        pct(in_scope, phantoms.len())
    );

    let mut top: Vec<(&str, usize)> = callee_counts.into_iter().collect();
    top.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    println!(
        "\n== Top phantom callee names (the shape of what syn calls that SCIP doesn't hold) =="
    );
    for (name, c) in top.iter().take(20) {
        let scope = if antigen_fns.contains(*name) {
            "in-scope"
        } else {
            "external"
        };
        println!("  {c:5}  {name:<26} [{scope}]");
    }

    println!("\n== Sample IN-SCOPE phantoms (genuine conflict/F candidates to eyeball) ==");
    for p in in_scope_samples.iter().take(20) {
        println!("  {} → {}", p.0, p.1);
    }
}

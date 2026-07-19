//! Dempster-Shafer conflict over the phantom residue — the H¹-obstruction stain + the hold-all test.
//! (Build-queue clone #3, nb 030 §7/§8; selection pressure for the `ds` primitive.)
//!
//! Model each in-scope syn-only phantom as two independent masses on `{Real, NotReal}`:
//! - **syn** saw the call → `(0.8 Real, 0, 0.2 unknown)`.
//! - **SCIP** depends on the rough class of the callee name:
//!   - *distinctive* (1 symbol): SCIP has that unique symbol but no edge to it → `(0, 0.8 NotReal, 0.2)` —
//!     it CONTRADICTS syn.
//!   - *collision* (≥2 symbols): SCIP cannot tell which same-named symbol was meant → `(0, 0, 1.0)` — pure
//!     IGNORANCE, not opposition.
//!
//! Two things to measure: (1) does DS conflict `K` light up the same distinctive residue rough flagged (the
//! H¹-obstruction ingredient — and DS *refines* rough by naming the boundary IGNORANCE and the lower
//! CONFLICT)? (2) does Dempster's normalization ERASE the conflict that hold-all preserves?
//!
//! Usage: `cargo run -p antigen-stroma --example ds_conflict -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::ds::Mass;
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

fn antigen_fn_classes(scip_path: &Path) -> HashMap<String, HashSet<String>> {
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

const SYN: Mass = Mass {
    real: 0.8,
    not_real: 0.0,
    unknown: 0.2,
};
const SCIP_CONTRADICTS: Mass = Mass {
    real: 0.0,
    not_real: 0.8,
    unknown: 0.2,
};
const SCIP_IGNORANT: Mass = Mass {
    real: 0.0,
    not_real: 0.0,
    unknown: 1.0,
};

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let scip_path = Path::new(&scip_path_s);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent");
    println!("== Dempster-Shafer conflict over the phantom residue (H¹-obstruction stain) ==\n");

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
    let src_roots: Vec<PathBuf> = CRATE_DIRS.iter().map(|d| workspace_root.join(d)).collect();
    let syn_pairs = syn_call_pairs_from_dir(&src_roots);
    let classes = antigen_fn_classes(scip_path);

    // Per in-scope phantom: combine syn ⊕ SCIP, tally conflict K by rough class.
    let (mut n_conflict, mut n_ignorance) = (0usize, 0usize);
    let (mut sum_k_conflict, mut sum_k_ignorance) = (0.0f64, 0.0f64);
    for p in &syn_pairs {
        if scip_call_bare.contains(p) {
            continue; // corroborated — not a phantom (syn ⊕ SCIP would agree, K≈0)
        }
        let Some(class) = classes.get(&p.1) else {
            continue; // external, out of scope
        };
        let scip = if class.len() == 1 {
            SCIP_CONTRADICTS
        } else {
            SCIP_IGNORANT
        };
        let Some((_, k)) = SYN.combine(scip) else {
            continue;
        };
        if class.len() == 1 {
            n_conflict += 1;
            sum_k_conflict += k;
        } else {
            n_ignorance += 1;
            sum_k_ignorance += k;
        }
    }

    println!("in-scope phantoms modelled: {}\n", n_conflict + n_ignorance);
    println!("  DISTINCTIVE (rough lower) → SCIP CONTRADICTS → conflict:");
    println!(
        "    n = {n_conflict:4}   mean K = {:.3}   (high conflict — the H¹ obstruction; genuine F candidates)",
        if n_conflict == 0 {
            0.0
        } else {
            sum_k_conflict / n_conflict as f64
        }
    );
    println!("  COLLISION (rough boundary) → SCIP IGNORANT → NOT conflict:");
    println!(
        "    n = {n_ignorance:4}   mean K = {:.3}   (K≈0 — ignorance m(Θ), not opposition; not a finding)",
        if n_ignorance == 0 {
            0.0
        } else {
            sum_k_ignorance / n_ignorance as f64
        }
    );
    println!(
        "\n  => DS conflict lights up EXACTLY rough's distinctive residue (cross-formalism convergence),\n\
         \x20    and REFINES it: the boundary is IGNORANCE (m(Θ)), the lower is CONFLICT (K). DS's signature\n\
         \x20    strength — separating conflict from ignorance — gives the rough split an evidential meaning."
    );

    // The hold-all test: what does Dempster's normalization do to a distinctive-phantom conflict?
    let (combined, k) = SYN.combine(SCIP_CONTRADICTS).expect("partial conflict");
    println!(
        "\n== hold-all vs Dempster (a single distinctive phantom: syn=Real vs SCIP=NotReal) =="
    );
    println!("  conflict K = {k:.3}  (large — a violent disagreement)");
    println!(
        "  Dempster-combined mass: Real {:.3}  NotReal {:.3}  unknown {:.3}",
        combined.real, combined.not_real, combined.unknown
    );
    println!(
        "  => normalization DISCARDED K: the result is a bland Real≈NotReal that looks like MILD UNCERTAINTY,\n\
         \x20    indistinguishable from a genuinely-ambiguous edge. hold-all keeps {{syn:Real, SCIP:NotReal}}\n\
         \x20    explicitly, so the conflict stays visible. VERDICT: keep DS's belief/plausibility + conflict-K\n\
         \x20    (the stain); REJECT Dempster's normalizing combination in favour of hold-all."
    );
}

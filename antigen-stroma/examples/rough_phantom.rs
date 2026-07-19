//! Rough-set decomposition of the phantom residue — does rough-set theory BIND, or is it a rhyme?
//! (Build-queue clone #1, nb 030; selection pressure for the `rough` primitive.)
//!
//! The phantom census (nb 029 §5) found the in-scope syn-only "conflict" is inflated by **bare-name
//! collisions**: syn sees a callee name like `read`, but antigen defines several `read`s, so syn cannot
//! tell WHICH it meant — and a non-corroboration might just be syn matching the wrong one. That is exactly
//! an *indiscernibility relation* (nodes sharing a bare name are syn-indiscernible), so the phantom set
//! should decompose the Pawlak way:
//!
//! - **lower / distinctive residue** — callee name maps to exactly ONE antigen symbol: syn's resolution is
//!   certain, so a syn-only pair here is a GENUINE SCIP-miss candidate (the real F residue).
//! - **boundary / collisions** — callee name maps to ≥2 symbols: undiscernible at the syn tier; the
//!   non-corroboration is indeterminate, not a finding.
//!
//! The BIND TEST: if the boundary is a large share of the in-scope phantoms and the distinctive residue is
//! a clean small set, rough sets have *explained* the census's collision inflation with a principled name.
//! If the split is uninformative, it stays a naming rhyme and we prune it (generate-then-select).
//!
//! Usage: `cargo run -p antigen-stroma --example rough_phantom -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::rough::approximate;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip, populate_from_reconstructions};
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

/// `bare_name -> { full SCIP symbols }` for every callable DEFINITION — the syn indiscernibility classes.
/// A singleton value is a distinctive name; a multi-member value is a collision class.
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

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

/// The bind test proper: split the in-scope phantoms into distinctive (lower) vs collision (boundary).
fn decompose_and_report(
    phantoms: &[&(String, String)],
    classes: &HashMap<String, HashSet<String>>,
) {
    let (mut distinctive, mut collision) = (0usize, 0usize);
    let mut distinctive_names: BTreeSet<&str> = BTreeSet::new();
    let mut collision_samples: Vec<(&str, usize)> = Vec::new();
    let mut seen_collision: HashSet<&str> = HashSet::new();
    for p in phantoms {
        let Some(class) = classes.get(&p.1) else {
            continue; // external — callee not defined in antigen, out of scope (census's external bucket)
        };
        if class.len() == 1 {
            distinctive += 1;
            distinctive_names.insert(&p.1);
        } else {
            collision += 1;
            if seen_collision.insert(&p.1) && collision_samples.len() < 15 {
                collision_samples.push((&p.1, class.len()));
            }
        }
    }
    let in_scope = distinctive + collision;

    println!("total syn-only phantoms:            {}", phantoms.len());
    println!("in-scope (callee defined in antigen): {in_scope}\n");
    println!(
        "  IN-SCOPE ROUGH DECOMPOSITION (the census's 'inflated by collision' made principled):"
    );
    println!(
        "    lower / DISTINCTIVE residue (callee → 1 symbol; genuine SCIP-miss F candidate): {distinctive:5}  ({:.1}% of in-scope)",
        pct(distinctive, in_scope)
    );
    println!(
        "    boundary / COLLISION (callee → ≥2 symbols; undiscernible at syn tier, not a finding): {collision:5}  ({:.1}% of in-scope)",
        pct(collision, in_scope)
    );
    println!(
        "    rough accuracy = |lower|/|in-scope| = {:.3}",
        if in_scope == 0 {
            1.0
        } else {
            distinctive as f64 / in_scope as f64
        }
    );
    println!(
        "\n  Sample COLLISION callee names (boundary — syn can't tell which target it meant):"
    );
    for (name, k) in &collision_samples {
        println!("    {name:<26} collides across {k} antigen symbols");
    }
    println!("\n  Sample DISTINCTIVE residue (lower — the clean F candidates worth eyeballing):");
    for name in distinctive_names.iter().take(15) {
        println!("    {name}");
    }
}

/// Global lens via the primitive: syn's whole bare-name view of the call graph, as one rough approximation.
fn global_lens(classes: &HashMap<String, HashSet<String>>, call_targets: &BTreeSet<String>) {
    let all_fn_symbols: Vec<String> = classes.values().flatten().cloned().collect();
    let approx = approximate(
        all_fn_symbols.iter().map(String::as_str),
        call_targets,
        |s| bare_fn_name(s).unwrap_or_default().to_string(),
    );
    println!(
        "\n== Global lens: syn's bare-name view of the whole call graph (rough approximation) =="
    );
    println!(
        "  universe (antigen fn def symbols): {}   concept (resolved Call targets): {}",
        all_fn_symbols.len(),
        call_targets.len()
    );
    println!(
        "  |lower| {}   |upper| {}   |boundary| {}   Pawlak accuracy {:.3}",
        approx.lower.len(),
        approx.upper.len(),
        approx.boundary.len(),
        approx.accuracy()
    );
    println!(
        "  (boundary = call-target names that COLLIDE with a non-target of the same name — the part of\n\
         \x20  syn's view that bare-name resolution structurally cannot separate.)"
    );
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let scip_path = Path::new(&scip_path_s);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent (the workspace root)");
    println!("== Rough-set decomposition of the phantom residue (bind test) ==\n");

    let recons = ingest_scip(scip_path);
    if recons.is_empty() {
        println!("(no SCIP reconstructions from {scip_path_s})");
        return;
    }

    // SCIP resolved Call bare-pairs (to find phantoms) + resolved Call TARGET symbols (the concept).
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
    let call_targets: BTreeSet<String> = recons
        .iter()
        .filter_map(|r| match r {
            EdgeReconstruction::Resolved(e) if e.kind == EdgeKind::Call => {
                Some(e.dst_symbol.clone())
            },
            _ => None,
        })
        .collect();

    // Phantoms = syn saw the call, SCIP has no matching bare Call edge.
    let src_roots: Vec<PathBuf> = CRATE_DIRS.iter().map(|d| workspace_root.join(d)).collect();
    let syn_pairs = syn_call_pairs_from_dir(&src_roots);
    let phantoms: Vec<&(String, String)> = syn_pairs
        .iter()
        .filter(|p| !scip_call_bare.contains(p))
        .collect();

    let classes = antigen_fn_classes(scip_path);
    decompose_and_report(&phantoms, &classes);
    global_lens(&classes, &call_targets);
}

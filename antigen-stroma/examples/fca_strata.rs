//! FCA clone #4 — the stratification-poset bind test (A-real, non-circular).
//! (nb 030 §4/§7; hardened by the 2026-07-19 adversarial design round.)
//!
//! Builds the formal context `fn-def symbols × ~8 RAW SCIP observables` on `target/antigen.scip` — the rough
//! `distinctive`/`collision` class is deliberately WITHHELD from the attributes (else "FCA re-derives rough"
//! is the `ds_conflict` circularity). Then measures three falsifiable things:
//!   1. non-degeneracy: `|concepts|` vs the `2^|M|` Boolean cube + the attribute-implication set (empty ⇒
//!      free Boolean ⇒ FCA found no structure ⇒ prune);
//!   2. the poset: lattice height (max intent size among concepts);
//!   3. non-circular DISCOVERY: max Jaccard of any concept's extent vs the WITHHELD rough strata — ≥0.7
//!      means FCA carved a known stratum from raw attributes ALONE (BIND); low across all concepts means the
//!      stratification is not recoverable from these observables (keep the descriptive lattice, prune the
//!      discovery claim).
//!
//! Usage: `cargo run -p antigen-stroma --example fca_strata -- [path/to.scip]`

#![allow(clippy::cast_precision_loss, clippy::similar_names)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::fca::Context;
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use antigen_stroma::syn_ingest::bare_fn_name;
use protobuf::Message;
use scip::types::{Index, SymbolRole};

const ATTRS: &[&str] = &[
    "call_source",    // 0: src of ≥1 Call edge
    "call_target",    // 1: dst of ≥1 Call edge
    "ctor_target",    // 2: dst of ≥1 Construct edge
    "fanout_hi",      // 3: out-degree ≥ median (among sources)
    "fanin_hi",       // 4: in-degree ≥ median (among targets)
    "self_recursive", // 5: has a self-loop Call edge
    "in_stroma",      // 6: defined in the antigen-stroma crate
    "in_core",        // 7: defined in the antigen (core) crate
];

/// Full fn-def symbols + the bare-name classes (for the WITHHELD rough strata), from the SCIP index.
fn fn_defs_and_classes(scip_path: &Path) -> (Vec<String>, HashMap<String, HashSet<String>>) {
    let mut defs: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut classes: HashMap<String, HashSet<String>> = HashMap::new();
    let Ok(bytes) = std::fs::read(scip_path) else {
        return (defs, classes);
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return (defs, classes);
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            if occ.symbol_roles & (SymbolRole::Definition as i32) != 0 {
                if let Some(name) = bare_fn_name(&occ.symbol) {
                    classes
                        .entry(name.to_string())
                        .or_default()
                        .insert(occ.symbol.clone());
                    if seen.insert(occ.symbol.clone()) {
                        defs.push(occ.symbol.clone());
                    }
                }
            }
        }
    }
    defs.sort();
    (defs, classes)
}

fn median(mut xs: Vec<usize>) -> usize {
    xs.sort_unstable();
    xs.get(xs.len() / 2).copied().unwrap_or(0)
}

/// SCIP symbol scheme is `rust-analyzer cargo <crate> <version> <descriptor>` — the crate is token 2.
fn crate_of(symbol: &str) -> Option<&str> {
    symbol.split(' ').nth(2)
}

fn jaccard(extent: &[usize], stratum: &HashSet<usize>) -> f64 {
    let inter = extent.iter().filter(|o| stratum.contains(o)).count();
    let union = extent.len() + stratum.len() - inter;
    if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    }
}

/// Build the `objects × 8 raw observables` context (rough class NOT among the attributes).
fn build_context(defs: &[String], recons: &[EdgeReconstruction]) -> Context {
    let (mut out_deg, mut in_deg) = (HashMap::<&str, usize>::new(), HashMap::<&str, usize>::new());
    let (mut ctor, mut selfrec) = (HashSet::<&str>::new(), HashSet::<&str>::new());
    for r in recons {
        if let EdgeReconstruction::Resolved(e) = r {
            match e.kind {
                EdgeKind::Call => {
                    *out_deg.entry(&e.src_symbol).or_default() += 1;
                    *in_deg.entry(&e.dst_symbol).or_default() += 1;
                    if e.src_symbol == e.dst_symbol {
                        selfrec.insert(&e.src_symbol);
                    }
                },
                EdgeKind::Construct => {
                    ctor.insert(&e.dst_symbol);
                },
                _ => {},
            }
        }
    }
    let med_out = median(out_deg.values().copied().collect());
    let med_in = median(in_deg.values().copied().collect());

    let mut incidence: Vec<(usize, usize)> = Vec::new();
    for (o, sym) in defs.iter().enumerate() {
        let s = sym.as_str();
        let (od, idg) = (
            out_deg.get(s).copied().unwrap_or(0),
            in_deg.get(s).copied().unwrap_or(0),
        );
        let mut has = |a: usize| incidence.push((o, a));
        if od > 0 {
            has(0);
        }
        if idg > 0 {
            has(1);
        }
        if ctor.contains(s) {
            has(2);
        }
        if od > 0 && od >= med_out {
            has(3);
        }
        if idg > 0 && idg >= med_in {
            has(4);
        }
        if selfrec.contains(s) {
            has(5);
        }
        match crate_of(s) {
            Some("antigen-stroma") => has(6),
            Some("antigen") => has(7),
            _ => {},
        }
    }
    Context::new(
        defs.to_vec(),
        ATTRS.iter().map(|a| (*a).to_string()).collect(),
        &incidence,
    )
}

fn intent_names(m: u64) -> String {
    (0..ATTRS.len())
        .filter(|i| m & (1 << i) != 0)
        .map(|i| ATTRS[i])
        .collect::<Vec<_>>()
        .join("+")
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    let scip_path = Path::new(&scip_path_s);
    println!("== FCA #4: stratification-poset bind test (A-real, rough class WITHHELD) ==\n");

    let recons = ingest_scip(scip_path);
    let (defs, classes) = fn_defs_and_classes(scip_path);
    if recons.is_empty() || defs.is_empty() {
        println!("(no SCIP data from {scip_path_s})");
        return;
    }
    let ctx = build_context(&defs, &recons);

    // 1. Non-degeneracy.
    let concepts = ctx.concepts();
    let cube = 1usize << ATTRS.len();
    let imps = ctx.pair_implications();
    println!(
        "  objects (fn-defs): {}   attributes: {}",
        defs.len(),
        ATTRS.len()
    );
    println!(
        "  |concepts| = {}   vs Boolean cube 2^{} = {}   (ratio {:.3})",
        concepts.len(),
        ATTRS.len(),
        cube,
        concepts.len() as f64 / cube as f64
    );
    println!(
        "  attribute implications a→b: {} {}",
        imps.len(),
        if imps.is_empty() {
            "(EMPTY = free Boolean = PRUNE)"
        } else {
            "(structure present)"
        }
    );
    for &(a, b) in imps.iter().take(8) {
        println!("      {} → {}", ATTRS[a], ATTRS[b]);
    }
    println!(
        "  lattice height (max intent size): {}",
        concepts
            .iter()
            .map(|c| c.intent.count_ones())
            .max()
            .unwrap_or(0)
    );

    // 3. Non-circular discovery: WITHHELD rough strata, by object index.
    let bare_class = |sym: &str| {
        classes
            .get(bare_fn_name(sym).unwrap_or(""))
            .map_or(0, HashSet::len)
    };
    let stratum = |keep: &dyn Fn(usize) -> bool| -> HashSet<usize> {
        defs.iter()
            .enumerate()
            .filter(|(_, s)| keep(bare_class(s)))
            .map(|(i, _)| i)
            .collect()
    };
    let distinctive = stratum(&|n| n == 1);
    let collision = stratum(&|n| n >= 2);
    let n_obj = defs.len();
    let base = |st: &HashSet<usize>| st.len() as f64 / n_obj as f64;
    // Base-rate control (the adversary's demand): exclude the ∅-extent and the UNIVERSAL concept, whose
    // Jaccard with a stratum S is exactly |S|/N (the base rate) — a base-rate artifact, not a discovery.
    let best = |st: &HashSet<usize>| {
        concepts
            .iter()
            .filter(|c| !c.extent.is_empty() && c.extent.len() < n_obj)
            .map(|c| (jaccard(&c.extent, st), c.intent))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap_or((0.0, 0))
    };
    let (jd, jd_i) = best(&distinctive);
    let (jc, jc_i) = best(&collision);
    let show = |name: &str, st: &HashSet<usize>, j: f64, intent: u64| {
        let b = base(st);
        println!(
            "    {name:<11} ({} objs, base {b:.3}): best non-trivial Jaccard {j:.3}  lift {:.2}  at [{}]",
            st.len(),
            j / b.max(f64::EPSILON),
            intent_names(intent)
        );
    };
    println!("\n  WITHHELD-stratum discovery (best NON-TRIVIAL concept, base-rate controlled):");
    show("distinctive", &distinctive, jd, jd_i);
    show("collision", &collision, jc, jc_i);
    // A real discovery clears 0.7 AND beats its base rate with genuine lift; otherwise it is an artifact.
    let discovered = (jd >= 0.7 && jd >= 1.3 * base(&distinctive))
        || (jc >= 0.7 && jc >= 1.3 * base(&collision));
    println!(
        "\n  => non-degenerate: {}; base-rate-controlled discovery: {} — {}",
        concepts.len() < cube && !imps.is_empty(),
        discovered,
        if discovered {
            "FCA carved a rough stratum from raw attributes alone (BIND-as-discovery)."
        } else {
            "raw observables do NOT recover the rough strata (no concept clears 0.7 with lift) → BIND-as-poset only; discovery PRUNED, exactly as the adversary predicted."
        }
    );
}

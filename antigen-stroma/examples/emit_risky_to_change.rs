//! The first heartbeat — the two wings connect. The afferent stroma senses "risky-to-change" and
//! emits `Finding`s that flow through antigen's existing efferent pipeline (they serialize into the
//! `scan --format json` contract). Sense → (grade) → act, end to end, on antigen's own graph.
//!
//! The signal is CROSS-KIND (the heartbeat taught us the single-node fight-quadrant is ~empty —
//! construct targets are TYPES, call-blast lives on FUNCTIONS). So: construct-in-degree = "how
//! depended-on is this SHAPE" (from Construct edges), and propagation-heat = "how hot is the code
//! that a change would disturb" = a node's own call-blast OR (for a type) the max call-blast of the
//! functions that BUILD it (across the construct edge). A node high on BOTH is the cross-kind fight
//! (a depended-on shape built by hot code) → a Dread marker; one → an Aura marker (convergence grades it).
//!
//! Usage: `cargo run -p antigen-stroma --example emit_risky_to_change -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::path::Path;

use antigen::finding::{FindingBody, Magnitude};
use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::efferent::{NodeSignal, risky_to_change_findings};
use antigen_stroma::engine::ReachabilityRelation;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::scip::{ingest_scip, populate_from_reconstructions};
use protobuf::Message;
use scip::types::{Index, SymbolRole};

/// The 75th-percentile ("top-quartile floor") of a set of counts — the "high" threshold. Computed
/// over the NONZERO values so a graph full of zeros doesn't drag the cutoff to nothing.
fn top_quartile(mut nonzero: Vec<usize>) -> usize {
    if nonzero.is_empty() {
        return usize::MAX; // nothing is "high" → the marker never fires
    }
    nonzero.sort_unstable();
    nonzero[nonzero.len() * 3 / 4]
}

fn short(path: &str) -> &str {
    path.rsplit(' ').next().unwrap_or(path)
}

#[allow(clippy::too_many_lines)] // one cohesive end-to-end demo of the sense→act loop
fn main() {
    let scip_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!(
        "== The first heartbeat: stroma senses risky-to-change → emits Findings ({scip_path}) ==\n"
    );

    // 1. Constitute the graph.
    let recons = ingest_scip(Path::new(&scip_path));
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    let snap = SnapshotHandle::open(&db);
    let edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    if edges.is_empty() {
        println!("(empty — check the .scip path)");
        return;
    }

    // 2. Construct-in-degree per node (keyed by symbol path) — from Construct edges only.
    let mut construct_in: HashMap<String, usize> = HashMap::new();
    for e in &edges {
        if e.kind == EdgeKind::Construct {
            *construct_in.entry(e.dst.fq_path.path.clone()).or_default() += 1;
        }
    }

    // 3. Own call-blast per node — forward reach over a CALL-ONLY relation (signals stay orthogonal).
    let call_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Call)
        .cloned()
        .collect();
    let rel_call = ReachabilityRelation::from_edges(&call_edges);
    let mut own_blast: HashMap<String, usize> = HashMap::new();
    for i in 0..rel_call.len() {
        let reached = rel_call.reach(i, true).nodes.len();
        own_blast.insert(rel_call.node(i).fq_path.path.clone(), reached);
    }

    // 3b. THE CROSS-KIND SIGNAL: a type's "propagation heat" = the hottest of the FUNCTIONS that build
    // it (max own-call-blast over the sources of Construct edges into it). The single-node fight was
    // ~empty because construct-targets-are-types / call-blast-is-functions; the real risk crosses the
    // construct edge — a depended-on SHAPE whose BUILDERS are on hot paths.
    let mut dependent_heat: HashMap<String, usize> = HashMap::new();
    for e in &edges {
        if e.kind == EdgeKind::Construct {
            let src_blast = own_blast.get(&e.src.fq_path.path).copied().unwrap_or(0);
            let slot = dependent_heat
                .entry(e.dst.fq_path.path.clone())
                .or_default();
            *slot = (*slot).max(src_blast);
        }
    }

    // 4. Definition locations (symbol → file:line) — the marker's site (SCIP definition occurrences).
    let def_loc = definition_locations(&scip_path);

    // 5. Assemble per-node signals. `heat` = max(own call-blast, dependent-heat): a hot function is
    // hot itself; a type is hot when its builders are. This is what makes the cross-kind fight fire.
    let mut universe: Vec<String> = construct_in
        .keys()
        .chain(own_blast.keys())
        .chain(dependent_heat.keys())
        .cloned()
        .collect();
    universe.sort_unstable();
    universe.dedup();
    let heat_of = |path: &str| {
        own_blast
            .get(path)
            .copied()
            .unwrap_or(0)
            .max(dependent_heat.get(path).copied().unwrap_or(0))
    };
    let nodes: Vec<NodeSignal> = universe
        .iter()
        .filter_map(|path| {
            let (file, line) = def_loc.get(path)?;
            Some(NodeSignal {
                path: path.clone(),
                construct_in: construct_in.get(path).copied().unwrap_or(0),
                heat: heat_of(path),
                file: file.clone(),
                line: *line,
            })
        })
        .collect();

    // 6. Thresholds (top-quartile of each signal's nonzero values).
    let c_hi = top_quartile(construct_in.values().copied().filter(|&v| v > 0).collect());
    let h_hi = top_quartile(nodes.iter().map(|n| n.heat).filter(|&v| v > 0).collect());
    println!(
        "nodes located: {}   thresholds: construct-in≥{c_hi}, propagation-heat≥{h_hi}\n",
        nodes.len()
    );

    // 7. THE BRIDGE — sense → act.
    let findings = risky_to_change_findings(&nodes, c_hi, h_hi);
    let (mut dread, mut aura) = (0usize, 0usize);
    for f in &findings {
        if let FindingBody::MarkedUnknown { magnitude, .. } = &f.body {
            match magnitude {
                Magnitude::Dread => dread += 1,
                Magnitude::Aura => aura += 1,
                Magnitude::Smell => {},
            }
        }
    }
    println!(
        "== {} risky-to-change Findings ==\n  DREAD (both signals — depended-on shape on a hot path): {dread}\n  AURA  (one signal): {aura}\n",
        findings.len()
    );

    // Order DREAD (the cross-kind fights) first — the interesting ones.
    let mut ordered: Vec<&_> = findings.iter().collect();
    ordered.sort_by_key(|f| std::cmp::Reverse(f.severity == antigen::finding::Severity::High));

    println!(
        "== Sample DREAD markers — the cross-kind fight (depended-on shape built by hot code) =="
    );
    for f in ordered
        .iter()
        .filter(|f| f.severity == antigen::finding::Severity::High)
        .take(8)
    {
        if let FindingBody::MarkedUnknown { trigger, .. } = &f.body {
            println!("  {}:{}  {}", short(&f.file), f.line, trigger);
        }
    }

    // 8. THE PROOF — one emitted DREAD Finding serialized into the pipeline's `--format json` contract.
    if let Some(f) = ordered.first() {
        println!("\n== One Finding, in the pipeline's Finding-JSON contract (the lift, proven) ==");
        println!(
            "  {}",
            serde_json::to_string(f).unwrap_or_else(|_| "<serialize failed>".to_string())
        );
    }
    println!("\n== The two wings are connected. ==");
}

/// Parse the SCIP index for every definition occurrence's `symbol → (file, start_line)`.
fn definition_locations(scip_path: &str) -> HashMap<String, (String, usize)> {
    let mut map = HashMap::new();
    let Ok(bytes) = std::fs::read(Path::new(scip_path)) else {
        return map;
    };
    let Ok(index) = Index::parse_from_bytes(&bytes) else {
        return map;
    };
    for doc in &index.documents {
        for occ in &doc.occurrences {
            let is_def = occ.symbol_roles & (SymbolRole::Definition as i32) != 0;
            if is_def && !occ.symbol.is_empty() && !occ.symbol.starts_with("local ") {
                let line = usize::try_from(occ.range.first().copied().unwrap_or(0)).unwrap_or(0);
                map.entry(occ.symbol.clone())
                    .or_insert_with(|| (doc.relative_path.clone(), line));
            }
        }
    }
    map
}

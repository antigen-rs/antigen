//! Curvature clone — the TOPOLOGY-lever bind test on antigen's own call graph.
//! (charter `curvature-as-perversity`; borrowed from Attali et al. curvature-constrained MPNN.)
//!
//! Three measurements, only the second of which is a genuine (non-definitional) bind test:
//!   1. **Triangle density + curvature distribution** — descriptive, and the honesty check: augmented Forman
//!      `4 − deg(u) − deg(v) + 3m` counts only TRIANGLES. If antigen's call graph is triangle-poor (likely —
//!      callers rarely call each other, and our reconvergence structure is 4-cycle DIAMONDS, not triangles),
//!      then Forman degenerates toward pure `−degree` and is a degree heuristic, NOT curvature. Say so.
//!   2. **The bind test (non-circular):** Forman claims "negative ⇒ bridge". Graph theory has an EXACT,
//!      independent notion — **cut edges** (removal disconnects), found by Tarjan. Do the most-negative edges
//!      actually coincide with real cut edges, BEYOND base rate? Lift ≤ ~1 ⇒ the bridge claim is not earning
//!      anything on this graph (the FCA base-rate lesson, applied up front).
//!   3. **Corroboration geometry** — triangle-bearing vs triangle-free curvature. Partly DEFINITIONAL (the
//!      `+3m` term mechanically lifts triangle-bearing edges), so it is reported as description, not a bind.
//!
//! Usage: `cargo run -p antigen-stroma --example curvature_strata -- [path/to.scip]`

#![allow(clippy::cast_precision_loss)]

use std::path::Path;

use antigen_stroma::base::facts::EdgeKind;
use antigen_stroma::curvature::{UndirectedGraph, index_pairs, mean_curvature};
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};

type Curvatures = Vec<((usize, usize), i64)>;

/// Descriptive pass: triangle density + curvature distribution. Returns the triangle-bearing fraction.
fn report_distribution(g: &UndirectedGraph, cs: &Curvatures, edges: &[(usize, usize)]) -> f64 {
    let tri_bearing = edges
        .iter()
        .filter(|(u, v)| g.triangles_on(*u, *v) > 0)
        .count();
    let total_tri: usize = edges
        .iter()
        .map(|(u, v)| g.triangles_on(*u, *v))
        .sum::<usize>()
        / 3;
    let frac = tri_bearing as f64 / edges.len() as f64;
    let neg = cs.iter().filter(|(_, c)| *c < 0).count();
    let pos = cs.iter().filter(|(_, c)| *c > 0).count();
    println!(
        "  nodes {}   undirected edges {}",
        g.node_count(),
        edges.len()
    );
    println!(
        "  triangles: {total_tri}   edges on ≥1 triangle: {tri_bearing} ({:.1}%)",
        frac * 100.0
    );
    println!(
        "  Forman curvature: mean {:.2}   negative {neg} ({:.1}%)   positive {pos} ({:.1}%)",
        mean_curvature(cs),
        neg as f64 / edges.len() as f64 * 100.0,
        pos as f64 / edges.len() as f64 * 100.0
    );
    frac
}

/// THE bind test: do the most-negative edges coincide with REAL cut edges beyond base rate? Returns lift.
fn bind_test(g: &UndirectedGraph, cs: &Curvatures, edges: &[(usize, usize)]) -> f64 {
    let cuts = g.cut_edges();
    let base = cuts.len() as f64 / edges.len() as f64;
    let mut sorted = cs.clone();
    sorted.sort_by_key(|(_, c)| *c); // ascending ⇒ most negative first
    let k = cuts.len().min(sorted.len()).max(1);
    let hits = sorted[..k].iter().filter(|(e, _)| cuts.contains(e)).count();
    let precision = hits as f64 / k as f64;
    let lift = if base > 0.0 { precision / base } else { 0.0 };
    let pick = |want: bool| -> Curvatures {
        cs.iter()
            .filter(|(e, _)| cuts.contains(e) == want)
            .copied()
            .collect()
    };
    println!("\n  BIND TEST — does 'negative curvature' find REAL bridges (cut edges)?");
    println!("    real cut edges: {} (base rate {base:.3})", cuts.len());
    println!(
        "    of the {k} most-negative edges, {hits} are real cut edges → precision {precision:.3}, lift {lift:.2}"
    );
    let (cut_mean, non_cut_mean) = (mean_curvature(&pick(true)), mean_curvature(&pick(false)));
    println!("    mean curvature: cut edges {cut_mean:.2}  vs  non-cut {non_cut_mean:.2}");
    if cut_mean > non_cut_mean {
        println!(
            "    ⚠ INVERTED: real bridges are the LEAST negative edges. Diagnosis — augmented Forman is\n\
             \x20     DEGREE-DOMINATED on a hub-heavy call graph, so it measures HUB-ness, not bottleneck-ness;\n\
             \x20     and a hub edge is never a cut edge (hubs have many routes) while genuine cut edges hang\n\
             \x20     off low-degree peripheral nodes. The measure is anti-correlated with the thing it claims."
        );
    }
    lift
}

/// Descriptive (DEFINITIONAL — the `+3m` term mechanically lifts triangle-bearing edges).
fn report_corroboration(g: &UndirectedGraph, cs: &Curvatures) {
    let pick = |want: bool| -> Curvatures {
        cs.iter()
            .filter(|((u, v), _)| (g.triangles_on(*u, *v) > 0) == want)
            .copied()
            .collect()
    };
    println!("\n  corroboration geometry (definitional, descriptive):");
    println!(
        "    mean curvature: triangle-bearing {:.2}  vs  triangle-free {:.2}",
        mean_curvature(&pick(true)),
        mean_curvature(&pick(false))
    );
}

fn main() {
    let scip_path_s = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!("== Curvature clone: the topology-lever on antigen's call graph ==\n");

    let recons = ingest_scip(Path::new(&scip_path_s));
    if recons.is_empty() {
        println!("(no SCIP reconstructions from {scip_path_s})");
        return;
    }
    let mut spairs: Vec<(&str, &str)> = Vec::new();
    for r in &recons {
        if let EdgeReconstruction::Resolved(e) = r {
            if e.kind == EdgeKind::Call {
                spairs.push((e.src_symbol.as_str(), e.dst_symbol.as_str()));
            }
        }
    }
    let (idx, ipairs) = index_pairs(&spairs);
    let g = UndirectedGraph::from_pairs(idx.len(), &ipairs);
    let (curvatures, edges) = (g.curvatures(), g.edges());
    if edges.is_empty() {
        println!("(no undirected edges)");
        return;
    }

    let tri_frac = report_distribution(&g, &curvatures, &edges);
    let lift = bind_test(&g, &curvatures, &edges);
    report_corroboration(&g, &curvatures);

    println!("\n  => VERDICT:");
    if tri_frac < 0.10 {
        println!(
            "     TRIANGLE-POOR graph ({:.1}% of edges on a triangle) ⇒ augmented Forman degenerates toward\n\
             \x20    −(deg u + deg v): a DEGREE heuristic, not curvature. Our reconvergence structure is 4-cycle\n\
             \x20    diamonds, which this measure cannot see. The corroboration-geometry claim needs a\n\
             \x20    4-cycle-aware variant (Balanced Forman / Ollivier) before it can be tested at all.",
            tri_frac * 100.0
        );
    }
    println!(
        "     bridge-detection: lift {lift:.2} ⇒ {}",
        if lift >= 1.3 {
            "negative curvature DOES find real bridges beyond base rate (risk-sensor leg BINDS)."
        } else {
            "negative curvature does NOT beat base rate at finding real bridges here (risk-sensor leg does not bind)."
        }
    );
}

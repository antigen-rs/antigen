//! Singular-locus stratification — the first empirical test of antigen's founding bet.
//!
//! **The idea being tested.** An organ computes a LOCAL invariant at each node — here,
//! blast-radius (`blast_from`, the downstream-affected-set size `light_the_wing` measured
//! fat-tailed on this same corpus). Such a field induces an INTRINSIC STRATIFICATION of the
//! graph: strata are regions where the field is ~constant; the **singular locus** is the edges
//! where the field CHANGES SHARPLY. The research prediction — failure-classes concentrate at
//! structural discontinuities, not at raw extremes — says "interesting" sites should concentrate
//! at the singular locus, ABOVE the base rate. This is a test of whether reading a field
//! TOPOLOGICALLY (where it changes) beats reading it FLATLY (its extremes).
//!
//! **Methodology, stated up front so the numbers below are auditable:**
//! - The graph is antigen's own SCIP call-graph (identical constitution to `light_the_wing`).
//! - Edges are DEDUPLICATED to distinct `(src, dst)` structural pairs and self-loops are DROPPED
//!   (a self-loop has zero gradient on every field here by construction — including it would
//!   dilute every base rate without contributing a possible singular edge).
//! - The "singular locus" cutoff is the TOP-DECILE gradient magnitude, by RANK (the value at the
//!   90th-percentile rank of `|field(u) − field(v)|` over all structural edges), not by an
//!   arbitrary fixed value. Ties at the cutoff are all included, so the realized fraction can run
//!   a little above 10% — reported honestly rather than forced to exactly 10%.
//! - **Degenerate-collapse guard**: if the naive rank-90 cutoff over the FULL gradient
//!   distribution is 0 (typical when most edges tie — e.g. every same-SCC edge below, or a
//!   shallow leaf-heavy call graph), a "top-decile" cutoff of 0 would call almost every edge
//!   "singular" and discriminate nothing. When that fires, the decile is recomputed over the
//!   STRICTLY-POSITIVE subset instead, and the collapse is reported — a real methodological
//!   finding, not something to sweep under the rug.
//! - Each "interesting" proxy reports an ENRICHMENT RATIO (rate at the singular locus ÷ base rate
//!   over ALL edges), never a raw percentage alone — a raw percentage alone is meaningless (this
//!   project has been bitten by exactly that mistake before).
//!
//! **A caveat that must be stated before the results, not after**: the SCC-boundary proxy below
//! is a TAUTOLOGY CHECK, not an independent finding. `blast_from` computes the SAME downstream
//! count for every member of one strongly-connected component (§ see `print_proxies`'s doc), so
//! any edge INSIDE an SCC has gradient exactly 0 — it can never be singular. The singular locus is
//! therefore GUARANTEED to be enriched for SCC-boundary edges by the field's own definition, not
//! by anything the corpus's structure independently confirms. It is included as a sanity control
//! (a proxy that SHOULD read near-100%, and a check that our machinery is not silently broken),
//! not as evidence for the hypothesis.
//!
//! Usage: `cargo run -p antigen-stroma --example singular_locus -- [path/to.scip]`
//! (defaults to `target/antigen.scip`, produced by `rust-analyzer scip . --output …`).

// A stats-display example: usize→f64 precision loss on graph counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use antigen_stroma::base::facts::ScipKind;
use antigen_stroma::db::StromaDb;
use antigen_stroma::engine::{Condensation, ReachabilityRelation, condense};
use antigen_stroma::read::query::blast_from;
use antigen_stroma::read::{DetectionGrade, ResolutionTier, SnapshotHandle};
use antigen_stroma::scip::{ingest_scip, ingest_scip_symbols, populate_from_reconstructions};

// ── Node-identity helpers (SCIP symbol string parsing) ─────────────────────────────────────────

/// A node's resolved-tier SCIP symbol string, or `None` for a non-resolved node. This corpus's
/// `ingest_scip` stamps every edge `Resolved` (no syntactic feeder here, unlike
/// `dual_tier_barcode`), but the accessor stays honest to the possibility rather than assuming it.
fn resolved_path(rel: &ReachabilityRelation, i: usize) -> Option<&str> {
    let id = rel.node(i);
    (id.fq_path.tier == ResolutionTier::Resolved).then_some(id.fq_path.path.as_str())
}

/// Split a SCIP symbol string into its `(package, descriptor)` halves.
///
/// Format: `<scheme> <manager> <package> <version> <descriptor>` (space-delimited); the
/// descriptor itself never contains a space in this corpus's Rust encoding (identifiers don't).
fn split_symbol(path: &str) -> Option<(&str, &str)> {
    let mut it = path.splitn(4, ' ');
    let _scheme = it.next()?;
    let _manager = it.next()?;
    let package = it.next()?;
    let (_version, descriptor) = it.next()?.split_once(' ')?;
    Some((package, descriptor))
}

/// The within-crate "module path" of a descriptor — the namespace segments before the final
/// type/fn segment (e.g. `engine/closure/ReachabilityRelation#from_edges().` → `engine/closure`;
/// a crate-root item with no namespace segment, e.g. `main().`, maps to `""`).
fn module_of(descriptor: &str) -> &str {
    descriptor.rsplit_once('/').map_or("", |(m, _)| m)
}

/// The tail descriptor of a SCIP symbol, for display (ported from `light_the_wing`'s `short`).
fn short(path: &str) -> &str {
    path.rsplit(' ').next().unwrap_or(path)
}

// ── Fields ──────────────────────────────────────────────────────────────────────────────────

/// The blast-radius field — one `blast_from` count per node (the fat-tail local invariant
/// `light_the_wing` measured on this same corpus).
fn blast_field(snap: &SnapshotHandle, rel: &ReachabilityRelation) -> Vec<usize> {
    (0..rel.len())
        .map(|i| {
            blast_from(snap, rel.node(i))
                .read_at_least(DetectionGrade::Dread)
                .map_or(0, Vec::len)
        })
        .collect()
}

/// The in-degree field — the cheap comparison field (no traversal, straight off the relation).
fn indegree_field(rel: &ReachabilityRelation) -> Vec<usize> {
    (0..rel.len()).map(|i| rel.backward_adj(i).len()).collect()
}

/// The distinct structural edge set: deduplicated `(src, dst)` index pairs, self-loops dropped.
fn distinct_edges(rel: &ReachabilityRelation) -> Vec<(usize, usize)> {
    let mut set = BTreeSet::new();
    for u in 0..rel.len() {
        for &(v, _tier) in rel.forward_adj(u) {
            if u != v {
                set.insert((u, v));
            }
        }
    }
    set.into_iter().collect()
}

/// The per-edge gradient `|field(u) − field(v)|` over a fixed structural edge set.
fn gradient(edges: &[(usize, usize)], field: &[usize]) -> Vec<usize> {
    edges
        .iter()
        .map(|&(u, v)| field[u].abs_diff(field[v]))
        .collect()
}

// ── Decile cutoff (with the degenerate-collapse guard) ─────────────────────────────────────────

/// The top-decile cutoff of a `usize` distribution — the value at the 90th-percentile RANK, so
/// ties at the boundary are all included rather than split. Returns `(None, _)` only when even the
/// strictly-positive subset is empty (the distribution is flat everywhere — nothing to stratify).
///
/// The `bool` reports whether the degenerate-collapse fallback fired: the naive rank-90 cutoff
/// over the FULL distribution was 0 (a majority of values tie at zero), so the decile was
/// recomputed over the strictly-positive subset instead.
fn decile_cutoff(values: &[usize]) -> (Option<usize>, bool) {
    if values.is_empty() {
        return (None, false);
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let raw_cutoff = sorted[(sorted.len() - 1) * 9 / 10];
    if raw_cutoff > 0 {
        return (Some(raw_cutoff), false);
    }
    let mut positive: Vec<usize> = values.iter().copied().filter(|&v| v > 0).collect();
    if positive.is_empty() {
        return (None, true);
    }
    positive.sort_unstable();
    (Some(positive[(positive.len() - 1) * 9 / 10]), true)
}

/// The edge indices at/above a decile cutoff (empty when the field was flat — `cutoff` is `None`).
fn singular_indices(grad: &[usize], cutoff: Option<usize>) -> Vec<usize> {
    cutoff.map_or_else(Vec::new, |c| {
        (0..grad.len()).filter(|&i| grad[i] >= c).collect()
    })
}

// ── Proxy accounting: base rate vs singular-locus rate, with an honest ELIGIBLE denominator ────

/// One proxy's hit-rate over some edge subset — `eligible` excludes edges the proxy could not
/// judge (missing SCIP-kind data, etc.), so a proxy's coverage gap never silently counts as a
/// miss.
struct EligibleRate {
    hits: usize,
    eligible: usize,
}

impl EligibleRate {
    fn rate(&self) -> Option<f64> {
        (self.eligible > 0).then(|| self.hits as f64 / self.eligible as f64 * 100.0)
    }
}

/// Accumulate hits/eligible over an edge-index subset for one classifier. `classify` returns
/// `None` for an edge this proxy cannot judge — excluded from both the numerator and denominator.
fn rate_over(
    edges: &[(usize, usize)],
    subset: &[usize],
    mut classify: impl FnMut(usize, usize) -> Option<bool>,
) -> EligibleRate {
    let mut hits = 0;
    let mut eligible = 0;
    for &i in subset {
        let (u, v) = edges[i];
        if let Some(hit) = classify(u, v) {
            eligible += 1;
            if hit {
                hits += 1;
            }
        }
    }
    EligibleRate { hits, eligible }
}

/// Print one proxy's base-vs-singular rate + enrichment ratio (or an "insufficient data" note).
///
/// `0.0 / 0.0` (zero hits in BOTH populations) is "no signal", not "infinite enrichment" — a naive
/// `if base == 0.0 { INFINITY }` guard would print `×inf` for a proxy that found NOTHING anywhere,
/// which reads as a strong result when it is the opposite. Only a zero base WITH a nonzero
/// singular rate is a genuine appearance-from-nothing.
fn print_proxy(name: &str, base: &EligibleRate, singular: &EligibleRate) {
    match (base.rate(), singular.rate()) {
        (Some(br), Some(sr)) => {
            let enrichment = match (br, sr) {
                (0.0, 0.0) => "n/a (zero hits at both)".to_string(),
                (0.0, _) => "×inf (zero base, nonzero singular)".to_string(),
                _ => format!("×{:.2}", sr / br),
            };
            println!(
                "  {name:<46} base={br:5.1}% (n={:>4}/{:<5})  singular={sr:5.1}% (n={:>3}/{:<4})  enrichment {enrichment}",
                base.hits, base.eligible, singular.hits, singular.eligible
            );
        },
        _ => println!("  {name:<46} — too few eligible edges to compute a rate (skipped)"),
    }
}

/// Compute + print one proxy's base and singular rates; return both for the closing verdict.
fn report_proxy(
    name: &str,
    edges: &[(usize, usize)],
    all_idx: &[usize],
    singular_idx: &[usize],
    classify: impl FnMut(usize, usize) -> Option<bool>,
) -> (EligibleRate, EligibleRate) {
    let mut classify = classify;
    let base = rate_over(edges, all_idx, &mut classify);
    let singular = rate_over(edges, singular_idx, &mut classify);
    print_proxy(name, &base, &singular);
    (base, singular)
}

// ── The "what's AT the singular locus" survey ──────────────────────────────────────────────────

/// Everything a proxy needs to classify an edge — bundled so `run_proxies` doesn't trip
/// `clippy::too_many_arguments`.
struct ProxyContext<'a> {
    rel: &'a ReachabilityRelation,
    edges: &'a [(usize, usize)],
    all_idx: &'a [usize],
    singular_idx: &'a [usize],
    kinds: &'a HashMap<&'a str, ScipKind>,
    cond: &'a Condensation,
    blast: &'a [usize],
    node_blast_cutoff: Option<usize>,
}

/// Run every "interesting" proxy over the base rate vs the singular locus, printing each and
/// returning `(name, base, singular)` triples for the closing verdict.
///
/// SCC-boundary is a TAUTOLOGY CHECK, not an independent finding (see the module doc): `blast_from`
/// derives every SCC member's downstream count as `|union of successor-SCC members| + (scc_size −
/// 1)`, which is IDENTICAL for every member — so an intra-SCC edge has gradient exactly 0 and can
/// never be singular by construction. It is kept as a sanity control (this proxy SHOULD read
/// near-100% enrichment; if it doesn't, the machinery itself is broken), separated from the
/// genuine candidates in the verdict.
fn run_proxies<'a>(ctx: &ProxyContext<'a>) -> Vec<(&'a str, EligibleRate, EligibleRate)> {
    let mut out = Vec::new();

    let (b, s) = report_proxy(
        "dst is constructor-shaped (enum-variant/struct/enum)",
        ctx.edges,
        ctx.all_idx,
        ctx.singular_idx,
        |_u, v| {
            resolved_path(ctx.rel, v)
                .and_then(|p| ctx.kinds.get(p))
                .map(|k| k.is_constructor())
        },
    );
    out.push(("dst is constructor-shaped", b, s));

    let (b, s) = report_proxy(
        "dst is dispatch-shaped (method/trait-method/static)",
        ctx.edges,
        ctx.all_idx,
        ctx.singular_idx,
        |_u, v| {
            resolved_path(ctx.rel, v)
                .and_then(|p| ctx.kinds.get(p))
                .map(|k| {
                    matches!(
                        k,
                        ScipKind::Method | ScipKind::TraitMethod | ScipKind::StaticMethod
                    )
                })
        },
    );
    out.push(("dst is dispatch-shaped", b, s));

    let (b, s) = report_proxy(
        "crosses a within-crate module boundary",
        ctx.edges,
        ctx.all_idx,
        ctx.singular_idx,
        |u, v| {
            let mu = resolved_path(ctx.rel, u)
                .and_then(split_symbol)
                .map(|(_, d)| module_of(d))?;
            let mv = resolved_path(ctx.rel, v)
                .and_then(split_symbol)
                .map(|(_, d)| module_of(d))?;
            Some(mu != mv)
        },
    );
    out.push(("crosses a module boundary", b, s));

    let (b, s) = report_proxy(
        "crosses a workspace-crate boundary",
        ctx.edges,
        ctx.all_idx,
        ctx.singular_idx,
        |u, v| {
            let pu = resolved_path(ctx.rel, u)
                .and_then(split_symbol)
                .map(|(p, _)| p)?;
            let pv = resolved_path(ctx.rel, v)
                .and_then(split_symbol)
                .map(|(p, _)| p)?;
            Some(pu != pv)
        },
    );
    out.push(("crosses a crate boundary", b, s));

    if let Some(cutoff) = ctx.node_blast_cutoff {
        let (b, s) = report_proxy(
            "src is itself a top-decile-blast orchestrator",
            ctx.edges,
            ctx.all_idx,
            ctx.singular_idx,
            |u, _v| Some(ctx.blast[u] >= cutoff),
        );
        out.push(("src is a high-blast orchestrator", b, s));
    }

    let (b, s) = report_proxy(
        "crosses an SCC boundary [TAUTOLOGY CHECK, not a finding]",
        ctx.edges,
        ctx.all_idx,
        ctx.singular_idx,
        |u, v| Some(ctx.cond.scc_of[u] != ctx.cond.scc_of[v]),
    );
    out.push(("SCC boundary (tautology check)", b, s));

    out
}

// ── Distribution + summary printing ────────────────────────────────────────────────────────────

/// Print one field's node-value distribution (max/mean/median/nonzero-count).
fn print_field_stats(name: &str, values: &[usize]) {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let max = *sorted.last().unwrap_or(&0);
    let mean = if sorted.is_empty() {
        0.0
    } else {
        sorted.iter().sum::<usize>() as f64 / sorted.len() as f64
    };
    let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0);
    let nonzero = sorted.iter().filter(|&&v| v > 0).count();
    println!(
        "  {name:<10} max={max:<6} mean={mean:<8.1} median={median:<6} nonzero={nonzero}/{}",
        sorted.len()
    );
}

/// Print one field's gradient distribution + the decile cutoff it yields (or the flat/collapse
/// notes when the naive cutoff needed the degenerate-collapse guard, or found nothing at all).
fn print_gradient_summary(
    name: &str,
    grad: &[usize],
    cutoff: Option<usize>,
    collapsed: bool,
    singular_count: usize,
) {
    let total = grad.len().max(1);
    let nonzero = grad.iter().filter(|&&g| g > 0).count();
    print!(
        "  {name} gradient: {} edges, {nonzero} nonzero ({:.1}%)",
        grad.len(),
        nonzero as f64 / total as f64 * 100.0
    );
    match cutoff {
        None => println!(" — FLAT: every edge ties. No singular locus exists on this field."),
        Some(c) => {
            println!(
                " — decile cutoff={c}{}, singular locus = {singular_count} edges ({:.1}%)",
                if collapsed {
                    " [degenerate-collapse: recomputed over the strictly-positive subset]"
                } else {
                    ""
                },
                singular_count as f64 / total as f64 * 100.0
            );
        },
    }
}

/// The Jaccard overlap between two fields' singular-locus edge sets — do independent local
/// invariants agree on WHERE the discontinuities are, or is "singular locus" field-specific?
fn print_cross_field_overlap(blast_idx: &[usize], indeg_idx: &[usize]) {
    let a: BTreeSet<usize> = blast_idx.iter().copied().collect();
    let b: BTreeSet<usize> = indeg_idx.iter().copied().collect();
    let overlap = a.intersection(&b).count();
    let union = a.union(&b).count().max(1);
    let jaccard = overlap as f64 / union as f64 * 100.0;
    println!(
        "\ncross-field check: blast-singular ∩ indegree-singular = {overlap}/{union} (Jaccard {jaccard:.1}%) — {}",
        if overlap == 0 {
            "the two fields pick out DISJOINT edges (singular locus is field-specific here)"
        } else if jaccard > 50.0 {
            "the two fields substantially AGREE on where the discontinuities sit"
        } else {
            "partial overlap — related but not the same locus"
        }
    );
}

/// The closing verdict: the strongest genuine (non-tautology) proxy's enrichment, the honest
/// biggest threat to validity, and what a better ground-truth would look like.
fn print_verdict(
    results: &[(&str, EligibleRate, EligibleRate)],
    singular_n: usize,
    total_n: usize,
) {
    println!("\n== Verdict ==");
    let best = results
        .iter()
        .filter(|(name, ..)| !name.contains("tautology"))
        .filter_map(|(name, base, singular)| {
            Some((
                *name,
                base.rate()?,
                singular.rate()?,
                singular.rate()? / base.rate()?.max(f64::MIN_POSITIVE),
            ))
        })
        .max_by(|a, b| a.3.total_cmp(&b.3));

    println!(
        "  singular locus: {singular_n}/{total_n} edges ({:.1}%) — the edges the blast-radius field changes sharply across.",
        singular_n as f64 / total_n.max(1) as f64 * 100.0
    );
    match best {
        Some((name, base, singular, enrichment)) if enrichment > 1.2 => println!(
            "  STRONGEST genuine signal: \"{name}\" — {base:.1}% base rate → {singular:.1}% at the \
             singular locus (×{enrichment:.2}). The stratification lens has SOME teeth on this corpus: \
             reading the field topologically beats reading it flatly, at least for this proxy."
        ),
        Some((name, base, singular, enrichment)) => println!(
            "  Best genuine signal (\"{name}\": {base:.1}% → {singular:.1}%, ×{enrichment:.2}) is WEAK \
             or flat. On this run, this proxy set does not show the singular locus concentrating \
             'interesting' structure above the base rate — a real null, not a failure to look."
        ),
        None => println!(
            "  No genuine (non-tautology) proxy had enough eligible data to compute an enrichment ratio."
        ),
    }
    println!(
        "\n  Biggest threats to validity: (1) every 'interesting' signal here is a STRUCTURAL PROXY \
         (constructor-kind, dispatch-shape, module/crate boundary), not a real failure-class \
         incidence — none of them is ground truth for 'a bug lives here'. (2) single corpus \
         (antigen's own ~3.9k-node call graph), single primary field (blast-radius); the fat tail \
         `light_the_wing` measured means a handful of orchestrator nodes could be doing most of the \
         enriching. (3) the SCC-boundary proxy is a tautology by the field's own definition (see the \
         module doc) — it is NOT independent evidence, it's a sanity control. (4) a 'src is a \
         top-decile-blast orchestrator' finding has its OWN mechanical-artifact risk: on a \
         fat-tailed field, the biggest |field(u) − field(v)| gradients are arithmetically far more \
         likely to touch an already-extreme value (two mid-range nodes can rarely produce a \
         top-decile gap; an extreme one paired with almost anything can) — so a high enrichment \
         there is real but partly an ARTIFACT of subtracting across a fat tail, not purely \
         independent evidence that orchestrators sit at architectural discontinuities. A better \
         ground truth: antigen already has an SZZ corpus-miner (`antigen/src/learn/szz.rs`, wired \
         through `cargo-antigen`'s `mine` subcommand) that finds bug-INTRODUCING commits by \
         git-blaming bug-FIXING ones — running the singular locus against SZZ-identified defect \
         sites (a real historical-defect ground truth, not a structural proxy) would be the next, \
         much stronger, test of this same hypothesis."
    );
}

// ── Orchestration (split out of `main` to stay under the line budget) ─────────────────────────

/// Ingest antigen's own SCIP index into a fresh `StromaDb` (identical to `light_the_wing`).
fn ingest(path: &str) -> StromaDb {
    let recons = ingest_scip(Path::new(path));
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    db
}

/// The node fields, their per-edge gradients, and the blast-field's singular-locus indices —
/// bundled so the later "what's at the locus" pass doesn't need every intermediate re-threaded.
struct FieldAnalysis {
    blast: Vec<usize>,
    blast_grad: Vec<usize>,
    blast_singular_idx: Vec<usize>,
}

/// Compute both fields, their gradients, and the singular loci; print the distribution + overlap
/// sections. Returns `None` (having already printed the null) when the primary (blast) field is
/// flat across every structural edge — there is nothing left to test.
fn analyze_fields(
    snap: &SnapshotHandle,
    rel: &ReachabilityRelation,
    edges: &[(usize, usize)],
) -> Option<FieldAnalysis> {
    let blast = blast_field(snap, rel);
    let indeg = indegree_field(rel);
    println!("== Node fields ==");
    print_field_stats("blast", &blast);
    print_field_stats("indegree", &indeg);

    let blast_grad = gradient(edges, &blast);
    let indeg_grad = gradient(edges, &indeg);
    let (blast_cutoff, blast_collapsed) = decile_cutoff(&blast_grad);
    let (indeg_cutoff, indeg_collapsed) = decile_cutoff(&indeg_grad);
    let blast_singular_idx = singular_indices(&blast_grad, blast_cutoff);
    let indeg_singular_idx = singular_indices(&indeg_grad, indeg_cutoff);

    println!("\n== Singular locus (top-decile gradient magnitude) ==");
    print_gradient_summary(
        "blast",
        &blast_grad,
        blast_cutoff,
        blast_collapsed,
        blast_singular_idx.len(),
    );
    print_gradient_summary(
        "indegree",
        &indeg_grad,
        indeg_cutoff,
        indeg_collapsed,
        indeg_singular_idx.len(),
    );
    if !indeg_singular_idx.is_empty() || !blast_singular_idx.is_empty() {
        print_cross_field_overlap(&blast_singular_idx, &indeg_singular_idx);
    }

    if blast_cutoff.is_none() {
        println!(
            "\nblast-gradient is FLAT across every structural edge on this corpus — there is no \
             singular locus to test enrichment against. That is itself a real, reportable null: on \
             this run, the stratification lens has nothing to grab onto."
        );
        return None;
    }
    Some(FieldAnalysis {
        blast,
        blast_grad,
        blast_singular_idx,
    })
}

/// Print the top singular-locus edges by blast gradient — the concrete sites the verdict talks
/// about, named rather than left as a count.
fn print_top_singular_edges(
    rel: &ReachabilityRelation,
    edges: &[(usize, usize)],
    fa: &FieldAnalysis,
) {
    println!("\n== Top singular-locus edges (largest blast gradient) ==");
    let mut by_grad: Vec<usize> = fa.blast_singular_idx.clone();
    by_grad.sort_by_key(|&i| std::cmp::Reverse(fa.blast_grad[i]));
    for &i in by_grad.iter().take(10) {
        let (u, v) = edges[i];
        println!(
            "  Δ{:<5} {} ({}) → {} ({})",
            fa.blast_grad[i],
            short(rel.node(u).fq_path.path.as_str()),
            fa.blast[u],
            short(rel.node(v).fq_path.path.as_str()),
            fa.blast[v]
        );
    }
}

/// The actual test: what's at the singular locus vs. the base rate, the verdict, and the concrete
/// top edges — everything downstream of a non-flat [`FieldAnalysis`].
fn run_singular_locus_test(
    rel: &ReachabilityRelation,
    edges: &[(usize, usize)],
    path: &str,
    fa: &FieldAnalysis,
) {
    let symbols = ingest_scip_symbols(Path::new(path));
    let kinds: HashMap<&str, ScipKind> = symbols
        .iter()
        .map(|s| (s.symbol.as_str(), s.kind))
        .collect();
    let cond = condense(rel);
    let all_idx: Vec<usize> = (0..edges.len()).collect();
    let (node_blast_cutoff, _) = decile_cutoff(&fa.blast);

    println!("\n== What's at the singular locus (base rate vs. singular-locus rate) ==");
    let ctx = ProxyContext {
        rel,
        edges,
        all_idx: &all_idx,
        singular_idx: &fa.blast_singular_idx,
        kinds: &kinds,
        cond: &cond,
        blast: &fa.blast,
        node_blast_cutoff,
    };
    let results = run_proxies(&ctx);

    print_verdict(&results, fa.blast_singular_idx.len(), edges.len());
    print_top_singular_edges(rel, edges, fa);
}

// ── Main ────────────────────────────────────────────────────────────────────────────────────────

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!(
        "== Singular-locus stratification: does 'interesting' concentrate where a local field \
         CHANGES SHARPLY, or is that just where it's high? =="
    );
    println!(
        "Hypothesis under test (antigen's founding bet): failure-classes/interesting-sites \
         concentrate at structural DISCONTINUITIES in a local-invariant field, above the base \
         rate — reading a field TOPOLOGICALLY beats reading it FLATLY (its extremes).\n"
    );

    // 1. Constitute the stroma from antigen's own SCIP graph (identical to `light_the_wing`).
    let db = ingest(&path);
    let snap = SnapshotHandle::open(&db);
    let edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    let rel = ReachabilityRelation::from_edges(&edges);
    if rel.is_empty() {
        println!("(empty — check the .scip path / that it generated)");
        return;
    }
    let structural_edges = distinct_edges(&rel);
    println!(
        "stroma constituted: {} nodes, {} edge-facts ({} distinct structural, self-loops excluded)\n",
        rel.len(),
        edges.len(),
        structural_edges.len()
    );

    // 2-3. The fields, their gradients, and the singular loci.
    let Some(fa) = analyze_fields(&snap, &rel, &structural_edges) else {
        return;
    };

    // 4. What's AT the singular locus — the actual test.
    run_singular_locus_test(&rel, &structural_edges, &path, &fa);
}

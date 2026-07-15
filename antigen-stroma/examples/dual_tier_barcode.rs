//! Dual-tier barcode — feed the stroma BOTH a syntactic (syn) extraction and the real SCIP
//! resolution of antigen's own codebase, tier-stamped by AGREEMENT, so [`TierBarcode`] actually
//! DISCRIMINATES.
//!
//! `light_the_wing`'s `ingest_scip` alone stamps every SCIP edge `Resolved` — there is no
//! `Syntactic`-only edge in that graph, so `G3` is empty and `G2 == G1`: the confidence-grading
//! never fires. Here a second, independent feeder (a `syn`-walk over the same source) supplies
//! the lower tier, and an edge is tier-stamped `Resolved` only when SCIP itself confirms it —
//! `Syntactic` otherwise (a syntactic-only phantom: a call the parser sees but SCIP could not, or
//! would not, resolve to a real enclosing/target function).
//!
//! Node identity here is deliberately coarsened to the **bare function name** (not the full SCIP
//! symbol or module path) — the projection both feeders can agree on, so a syntactic call-edge
//! and a SCIP-resolved call-edge can land on the SAME node and actually be compared.
//!
//! The syntactic extractor (`CallVisitor`, `extract_call_name`, `syn_edges_from_file`) and the
//! SCIP bare-name reducer (`symbol_fn_name`) are ported from
//! `antigen-stroma-experiment/src/main.rs` (the syntactic-vs-SCIP false-edge-rate falsifier) —
//! tested and measured there; reused here rather than reinvented.
//!
//! Usage: `cargo run -p antigen-stroma --example dual_tier_barcode -- [path/to.scip]`
//! (defaults to `target/antigen.scip`, produced by `rust-analyzer scip . --output …`).

// A stats-display example: usize→f64 precision loss on graph/edge counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]

use std::collections::{BTreeSet, HashSet};
use std::path::Path;

use antigen_stroma::base::facts::{EdgeFact, EdgeFacts, EdgeKind, WitnessSource};
use antigen_stroma::db::StromaDb;
use antigen_stroma::engine::ReachabilityRelation;
use antigen_stroma::engine::path_homology::{TierBarcode, phantom_cycles_from_relation};
use antigen_stroma::node::id::FqPath;
use antigen_stroma::node::{CfgSet, IdentityDigest, StromaNodeId};
use antigen_stroma::read::{ResolutionTier, SnapshotHandle};
use antigen_stroma::scip::{EdgeReconstruction, ingest_scip};
use syn::visit::Visit;
use walkdir::WalkDir;

// ── Syntactic extraction (ported from antigen-stroma-experiment/src/main.rs) ──────────────────

/// Visitor that records `(caller_fn, callee_name)` syntactic call-edges — the falsifier's
/// `CallVisitor`, unchanged.
struct CallVisitor {
    /// Current enclosing function name stack (for nested fns/closures).
    fn_stack: Vec<String>,
    /// Collected edges: (`caller_fn`, `callee_name`).
    edges: Vec<(String, String)>,
}

impl CallVisitor {
    const fn new() -> Self {
        Self {
            fn_stack: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn current_fn(&self) -> Option<&str> {
        self.fn_stack.last().map(String::as_str)
    }
}

impl<'ast> Visit<'ast> for CallVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let (Some(caller), Some(callee)) = (self.current_fn(), extract_call_name(&node.func)) {
            self.edges.push((caller.to_string(), callee));
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if let Some(caller) = self.current_fn() {
            self.edges
                .push((caller.to_string(), node.method.to_string()));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Extract the bare function name from a call expression's function part (ported, unchanged).
fn extract_call_name(expr: &syn::Expr) -> Option<String> {
    match expr {
        // `foo()`, `Foo::bar()`, `crate::mod::fn()` — last segment is the name.
        syn::Expr::Path(ep) => ep.path.segments.last().map(|s| s.ident.to_string()),
        // `self.field()` (method via field access) — use the field name.
        syn::Expr::Field(ef) => match &ef.member {
            syn::Member::Named(n) => Some(n.to_string()),
            syn::Member::Unnamed(_) => None,
        },
        // Closures, macro calls, etc. — not extractable as a name.
        _ => None,
    }
}

/// Extract syntactic `(caller_fn, callee_name)` call-edges from one Rust source file (ported).
fn syn_edges_from_file(path: &Path) -> Vec<(String, String)> {
    let Ok(src) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(parsed) = syn::parse_file(&src) else {
        return Vec::new();
    };
    let mut visitor = CallVisitor::new();
    visitor.visit_file(&parsed);
    visitor.edges
}

/// Walk the workspace's crate `src/` dirs (skipping `target/` implicitly — it is never in the
/// list) and collect every syntactic `(caller, callee)` name pair, deduplicated.
fn extract_syn_pairs(workspace_root: &Path) -> HashSet<(String, String)> {
    const CRATE_DIRS: &[&str] = &[
        "antigen/src",
        "antigen-stroma/src",
        "antigen-fingerprint/src",
        "antigen-macros/src",
        "antigen-attestation/src",
        "cargo-antigen/src",
    ];
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
            pairs.extend(syn_edges_from_file(entry.path()));
        }
    }
    pairs
}

// ── SCIP bare-name reduction (ported from antigen-stroma-experiment/src/main.rs) ──────────────

/// Extract the bare function name from a SCIP symbol string (ported, unchanged) — e.g.
/// `rust-analyzer cargo antigen 0.6.0 scan/scan_file().` → `scan_file`.
fn symbol_fn_name(symbol: &str) -> Option<&str> {
    let without_suffix = symbol.strip_suffix(").")?;
    let paren_pos = without_suffix.rfind('(')?;
    let before_paren = &without_suffix[..paren_pos];
    let segment = before_paren
        .rsplit(['/', '#'])
        .next()
        .filter(|s| !s.is_empty())?;
    // Strip a backtick-escaped type-qualifier prefix: `[`TypeName`]method_name` → `method_name`.
    if let Some(bracket_end) = segment.rfind(']') {
        let after_bracket = &segment[bracket_end + 1..];
        if !after_bracket.is_empty() {
            return Some(after_bracket);
        }
    }
    // Strip backtick escaping from a simple identifier: `` `name` `` → `name`.
    if segment.starts_with('`') && segment.ends_with('`') && segment.len() > 2 {
        return Some(&segment[1..segment.len() - 1]);
    }
    Some(segment)
}

/// Reduce every `Resolved` SCIP reconstruction to a bare-name `(caller, callee)` pair — the
/// stroma's own tier-honest ingestion already refused to call an `Ambiguous`/macro-site edge
/// `Resolved`, so this is exactly the SCIP-confirmed set.
fn scip_resolved_pairs(recons: &[EdgeReconstruction]) -> HashSet<(String, String)> {
    let mut pairs = HashSet::new();
    for recon in recons {
        let EdgeReconstruction::Resolved(edge) = recon else {
            continue;
        };
        if let (Some(caller), Some(callee)) = (
            symbol_fn_name(&edge.src_symbol),
            symbol_fn_name(&edge.dst_symbol),
        ) {
            pairs.insert((caller.to_string(), callee.to_string()));
        }
    }
    pairs
}

// ── Node construction ───────────────────────────────────────────────────────────────────────

/// A node id built from a bare function name — mirrors `scip.rs`'s private `node_from_symbol`,
/// but keyed on the coarser bare-NAME identity so a syntactic occurrence and a SCIP-resolved
/// occurrence of the SAME function land on the SAME node.
///
/// `fq_path.tier` is fixed at `Syntactic` for every node regardless of which edges reference
/// it: a bare name (no module qualification) is itself a name-level, not symbol-level, locator —
/// that is a property of the PATH, independent of which tier the specific edge that connects it
/// was reconstructed at (`EdgeFact.tier`, not the node's `fq_path.tier`, is what `TierBarcode`
/// filters on). Stamping every node the same tier is what lets the same bare name merge into one
/// node across both feeders — the whole point of this coarsening.
fn node_from_name(name: &str) -> StromaNodeId {
    StromaNodeId {
        fq_path: FqPath {
            path: name.to_string(),
            tier: ResolutionTier::Syntactic,
        },
        identity_digest: IdentityDigest::of_tokens(name.as_bytes()),
        cfg_set: CfgSet::default(),
    }
}

// ── Printing (split out of `main` to stay under the line budget) ──────────────────────────────

/// Print the syntactic-vs-SCIP alignment section. Returns the `(confirmed, phantom)` pair counts
/// — `phantom` is exactly the count of node-name pairs the union will stamp `Syntactic`.
fn print_alignment(
    syn_pairs: &HashSet<(String, String)>,
    scip_pairs: &HashSet<(String, String)>,
) -> (usize, usize) {
    let confirmed = syn_pairs.intersection(scip_pairs).count();
    let phantom = syn_pairs.len() - confirmed;
    let agreement = if syn_pairs.is_empty() {
        0.0
    } else {
        confirmed as f64 / syn_pairs.len() as f64 * 100.0
    };
    println!("== Syntactic-vs-SCIP alignment (of the syntactic set) ==");
    println!("  SCIP-confirmed syntactic pairs: {confirmed}");
    println!("  syntactic-only (phantom) pairs: {phantom}");
    println!(
        "  agreement rate: {confirmed} / {} = {agreement:.1}%\n",
        syn_pairs.len()
    );
    (confirmed, phantom)
}

/// Build the tier-stamped union over bare-name node identity — in the SCIP set → `Resolved`,
/// syntactic-only → `Syntactic` (a phantom that never got SCIP confirmation) — and print the
/// breakdown. `phantom` is threaded in from [`print_alignment`] (same count, different denominator).
fn build_tier_edges(
    syn_pairs: &HashSet<(String, String)>,
    scip_pairs: &HashSet<(String, String)>,
    phantom: usize,
) -> Vec<EdgeFact> {
    let union: HashSet<&(String, String)> = syn_pairs.union(scip_pairs).collect();
    let mut edges: Vec<EdgeFact> = Vec::with_capacity(union.len());
    for pair in &union {
        // HOLD the witness set — who saw it — rather than pre-collapsing it to a tier at the door.
        // `{Syn, ScipResolved}` (both instruments agree) and `{ScipResolved}` (resolution saw what
        // parsing cannot) both PROJECT to Resolved, but they are not the same evidence.
        let mut w = BTreeSet::new();
        if syn_pairs.contains(*pair) {
            w.insert(WitnessSource::Syn);
        }
        if scip_pairs.contains(*pair) {
            w.insert(WitnessSource::ScipResolved);
        }
        edges.push(EdgeFact::new(
            node_from_name(&pair.0),
            node_from_name(&pair.1),
            EdgeKind::Call,
            w,
        ));
    }
    let resolved_edges = union.len() - phantom;
    println!("== Tier-stamped union fed to the stroma ==");
    println!("  union (distinct node-name-pair edges): {}", union.len());
    println!(
        "    tier Resolved  (SCIP-confirmed):  {resolved_edges:5}  ({:.1}%)",
        resolved_edges as f64 / union.len() as f64 * 100.0
    );
    println!(
        "    tier Syntactic (SCIP-unconfirmed): {phantom:5}  ({:.1}%)\n",
        phantom as f64 / union.len() as f64 * 100.0
    );
    edges
}

/// Print the confidence-graded topology — the three nested filtration levels.
fn print_barcode(bc: &TierBarcode) {
    println!("== Confidence-graded topology (directed H1 cycles per fidelity tier) ==");
    println!(
        "  G3 (proof-tier only):  vertices={:5} edges={:6} H0={:4} H1={}",
        bc.g3.vertices, bc.g3.edges, bc.g3.h0, bc.g3.h1
    );
    println!(
        "  G2 (+Resolved):        vertices={:5} edges={:6} H0={:4} H1={}",
        bc.g2.vertices, bc.g2.edges, bc.g2.h0, bc.g2.h1
    );
    println!(
        "  G1 (all, +Syntactic):  vertices={:5} edges={:6} H0={:4} H1={}\n",
        bc.g1.vertices, bc.g1.edges, bc.g1.h0, bc.g1.h1
    );
}

/// Print the H1(G1)−H1(G2) key line plus the honest read of whether the tiering discriminates.
fn print_read(bc: &TierBarcode, phantom: usize, union_len: usize) {
    let phantom_cycles = bc.g1.h1.saturating_sub(bc.g2.h1);
    println!(
        "  H1(G1) − H1(G2) = {phantom_cycles}  → entanglement cycles that exist ONLY through \
         syntactic (SCIP-unconfirmed) edges — {phantom} phantom edge(s) total, {:.1}% of the union.",
        phantom as f64 / union_len as f64 * 100.0
    );
    println!();

    println!("== Read ==");
    if bc.g2.edges == bc.g1.edges {
        println!(
            "  G2 == G1 (edges): SCIP confirmed EVERY syntactic edge on this codebase — the \
             tiering is still monochrome here, not because it can't discriminate but because \
             this run found zero phantoms to discriminate against."
        );
    } else if phantom_cycles > 0 {
        println!(
            "  G2 < G1 AND H1 grows going down the ladder: the tiering DISCRIMINATES — there are \
             real entanglement cycles in the syntactic-only graph that a SCIP-confirmed reading \
             does not support. Grading now means something."
        );
    } else {
        println!(
            "  G2 < G1 (edges) but H1(G1) == H1(G2): the phantom edges exist but happen not to \
             close any NEW cycle on this codebase's actual call topology — the vertex/edge counts \
             discriminate even though H1 doesn't, on this run."
        );
    }
}

// ── Main ────────────────────────────────────────────────────────────────────────────────────

fn main() {
    let scip_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    // antigen-stroma's own CARGO_MANIFEST_DIR is `<workspace>/antigen-stroma`; its parent is the
    // workspace root regardless of the cwd `cargo run` was invoked from.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("antigen-stroma has a parent directory (the workspace root)");
    println!("== Dual-tier barcode: syntactic ⊔ SCIP on antigen's own codebase ==\n");

    // 1. Syntactic edges — a syn-walk over the workspace's crate sources.
    let syn_pairs = extract_syn_pairs(workspace_root);
    println!(
        "syntactic (syn-walk) (caller,callee) pairs: {}",
        syn_pairs.len()
    );

    // 2. SCIP edges — the real resolved-feeder wiring, reduced to bare names.
    let recons = ingest_scip(Path::new(&scip_path));
    if recons.is_empty() {
        println!(
            "\n(no SCIP reconstructions from {scip_path} — check the path, or that \
             `rust-analyzer scip . --output {scip_path}` ran)"
        );
        return;
    }
    let scip_pairs = scip_resolved_pairs(&recons);
    println!(
        "SCIP-resolved (caller,callee) pairs:         {}\n",
        scip_pairs.len()
    );

    let (_confirmed, phantom) = print_alignment(&syn_pairs, &scip_pairs);

    // 3. Tier-stamped union over bare-NAME node identity.
    let edges = build_tier_edges(&syn_pairs, &scip_pairs, phantom);
    let union_len = edges.len();

    // 4. Constitute + barcode — through the real salsa input surface (mirrors
    //    `populate_from_reconstructions`), then the confidence-graded topology.
    let mut db = StromaDb::default();
    let edge_facts = EdgeFacts::new(&db, edges);
    db.set_edge_facts(edge_facts);
    let snap = SnapshotHandle::open(&db);
    let stored_edges = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    let rel = ReachabilityRelation::from_edges(&stored_edges);
    println!(
        "stroma constituted: {} nodes, {} edges\n",
        rel.len(),
        stored_edges.len()
    );

    let bc = TierBarcode::from_relation(&rel);
    print_barcode(&bc);
    print_read(&bc, phantom, union_len);
    print_phantom_generators(&rel);
}

/// Name the phantom (syntactic-only) entanglement generators — the F1 generator-level companion to the
/// dimension barcode: WHICH loops witness the phantom H₁, not just how many. The cokernel dimension can
/// differ from the naive `H1(G1) − H1(G2)` delta when some resolved cycles die under new G₁ 2-paths.
fn print_phantom_generators(rel: &ReachabilityRelation) {
    let gens = phantom_cycles_from_relation(rel);
    println!("\n== Phantom generators — WHICH loops, not just how many (the F1 pairing) ==");
    println!(
        "  cokernel dim = {} independent syntactic-only entanglement loop(s)",
        gens.len()
    );
    let mut by_len: Vec<&Vec<usize>> = gens.iter().collect();
    by_len.sort_by_key(|c| c.len());
    println!("  shortest suspect tangles (each closes only via a SCIP-unconfirmed edge):");
    for cyc in by_len.iter().take(6) {
        let names: Vec<&str> = cyc
            .iter()
            .map(|&i| {
                let p = rel.node(i).fq_path.path.as_str();
                p.rsplit(' ').next().unwrap_or(p)
            })
            .collect();
        println!(
            "    [{} hop] {}",
            cyc.len().saturating_sub(1),
            names.join(" → ")
        );
    }
}

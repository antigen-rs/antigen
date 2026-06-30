//! STEP 4 — the `ScanReport -> NodeFacts/EdgeFacts` lowering (the integration seam).
//!
//! The existing scan (`antigen::scan::scan_workspace -> ScanReport`) is the input feeder. This
//! adapter lowers `ScanReport`'s per-attribute Vecs (presentations, immunities, ...) into the keyed
//! relational base. The SCIP ingestion (resolved edges) is a SECOND feeder into `EdgeFacts`,
//! tier-stamped resolved.
//!
//! CONVERGE-WAVE FLAG (do not let this go silent): does `ScanReport` stay the wire format with the
//! stroma INDUCED from it, or does the stroma become primary and `ScanReport` a projection? The
//! implementer's lean: stroma primary, `ScanReport` a backward-compatible projection (the genome's
//! "`ScanReport` is an induced view" reading). This is a converge-wave call — named here so the
//! builder doesn't decide it implicitly.
//!
//! ## The two digests at frame epoch
//!
//! Both are computed from ONE parse of the item source (see `digests_at_line`):
//! - `IdentityDigest` (BLAKE3) routes through the [`canonical_identity_tokens`] SEAM (§4.3): it
//!   strips PURE-annotation antigen attrs but KEEPS load-bearing ones, so identity is tamper-evident
//!   on a forged `#[presents]` yet stable under a toggled `#[diagnostic]`. The strip decision lives
//!   in that ONE seam (`node::digest`), NOT duplicated here — this adapter only calls it.
//! - `ShapeDigest` (FNV-1a) routes through `ShapeDigest::of_item`, which delegates to
//!   [`antigen_fingerprint::structural_shape_digest`] (name-INSENSITIVE — the clustering/backdate key).
//!
//! ## `structural_fingerprint` on `ScanReport` items CANNOT be reused
//!
//! The `structural_fingerprint` on `Presentation`/`Immunity`/etc. is the name-SENSITIVE FNV digest
//! (`structural_digest`, not `structural_shape_digest`). Reusing it as `ShapeDigest` would make
//! `ShapeDigest` name-sensitive, defeating the clustering/backdate property. Always recompute from
//! source using `ShapeDigest::of_item()`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::spanned::Spanned;

use crate::base::facts::{EdgeFact, NodeFact};
use crate::node::cfg::CfgSet;
use crate::node::digest::{IdentityDigest, ShapeDigest, canonical_identity_tokens};
use crate::node::id::{FqPath, StromaNodeId};
use crate::node::path::syntactic_fq_path;

// ── Item extraction ───────────────────────────────────────────────────────────────────────────────

/// Compute BOTH digests of an item from a file, identified by its 1-based line number.
///
/// Parses the file with `syn::parse_file` ONCE, finds the item whose `Spanned::span().start().line`
/// matches `target_line`, and computes:
/// - the `IdentityDigest` via the [`canonical_identity_tokens`] SEAM (§4.3 — strips PURE-annotation
///   antigen attrs but KEEPS load-bearing ones, so identity is tamper-evident on a forged `#[presents]`),
/// - the `ShapeDigest` via the item's rendered source (name-INSENSITIVE FNV, the clustering key).
///
/// Returns `None` if the file can't be read/parsed or no item matches the line. The 90% case for
/// antigen's own codebase (no inline non-test `mod` blocks); the SCIP symbol refines at engine-epoch.
fn digests_at_line(file: &Path, target_line: usize) -> Option<(IdentityDigest, ShapeDigest)> {
    let src = std::fs::read_to_string(file).ok()?;
    let parsed = syn::parse_file(&src).ok()?;

    for item in &parsed.items {
        if item.span().start().line == target_line {
            // Identity: BLAKE3 over the §4.3 canonical preimage (pure-stripped, load-bearing + name kept).
            let identity = IdentityDigest::of_tokens(&canonical_identity_tokens(item));
            // Shape: name-insensitive FNV. of_item takes source text and re-normalizes the ident; we
            // hand it the item's own rendered tokens (a canonical, parseable rendering of THIS item).
            let shape = ShapeDigest::of_item(&item.to_token_stream().to_string());
            return Some((identity, shape));
        }
    }
    None
}

/// The traceable placeholder digests for a node whose source is unreadable (file gone / item not at
/// the expected line). Identity = `BLAKE3(fq_path)` — WRONG-BUT-TRACEABLE: it does NOT represent the
/// item's real content and will NOT match a re-constituted scan once the file is readable, which is
/// the correct behavior (a gap in source evidence means the stale stroma SHOULD disagree with a fresh
/// one). Shape = a name-sensitive degraded key.
fn gap_digests(fq_path: &str) -> (IdentityDigest, ShapeDigest) {
    (
        IdentityDigest::of_tokens(fq_path.as_bytes()),
        ShapeDigest(format!("raw-gap:{fq_path}")),
    )
}

// ── Module-path derivation ────────────────────────────────────────────────────────────────────────

/// Derive the module chain from a file path relative to `source_root`.
///
/// Strips the `src/` prefix and `.rs` extension, handles `mod.rs` and `lib.rs` edge cases, and
/// splits the remainder on path separators. The 90% case for antigen's own codebase (no inline
/// non-test `mod` blocks, no `#[path]` attrs). The SCIP symbol (engine-epoch) supersedes this.
///
/// Examples:
/// - `src/lib.rs`         → `[]` (the crate root — module chain is empty)
/// - `src/node/mod.rs`    → `["node"]`
/// - `src/node/locator.rs`→ `["node"]` (the file IS the `node::locator` module, but the module
///   chain for items IN `locator.rs` is `["node", "locator"]` — see below)
///
/// NOTE: items in a file live in the module that FILE is. For `src/foo/bar.rs`, items are in
/// `crate::foo::bar::item`. The chain returned here is the chain for items in this file.
fn module_chain_from_path(file: &Path, source_root: &Path) -> Vec<String> {
    // Make relative to source_root.
    let rel = file.strip_prefix(source_root).unwrap_or(file);
    // Strip `src/` prefix (the common Rust convention).
    let rel = rel.strip_prefix("src").unwrap_or(rel);
    // Drop the `.rs` extension.
    let stem = rel.with_extension("");
    // Split on path separators and collect segments.
    let segments: Vec<String> = stem
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        return vec![];
    }

    // `lib.rs` → items are in the crate root — no module prefix.
    if segments.len() == 1 && segments[0] == "lib" {
        return vec![];
    }
    // `main.rs` → same as lib.rs for the crate-root module.
    if segments.len() == 1 && segments[0] == "main" {
        return vec![];
    }
    // `mod.rs` as the last segment → the module is the parent dir, strip the `mod` leaf.
    if segments.last().is_some_and(|s| s == "mod") {
        return segments[..segments.len() - 1].to_vec();
    }
    // Every other file: the full path-without-extension is the module chain.
    // e.g. `src/node/locator.rs` → ["node", "locator"]
    segments
}

// ── Key type for node deduplication ──────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Hash)]
struct NodeKey {
    file: PathBuf,
    item_target_str: String,
}

impl NodeKey {
    fn new(file: &Path, item_target: &antigen::scan::ItemTarget) -> Self {
        Self {
            file: file.to_path_buf(),
            item_target_str: format!("{item_target:?}"),
        }
    }
}

// ── The lowering pass ─────────────────────────────────────────────────────────────────────────────

/// Lower a scan report into the base fact PAYLOADS.
///
/// Returns `(Vec<NodeFact>, Vec<EdgeFact>)` — the plain value tuples. The caller (`constitute()`)
/// wraps these with `NodeFacts::new(&mut db, nodes)` / `EdgeFacts::new(&mut db, edges)` to produce
/// the salsa inputs. This function is `&mut db`-free so it is testable without a live db.
///
/// Each unique `(file, item_target)` pair in the report's per-attribute vecs becomes one
/// `NodeFact`. Edges come from `lineage_edges` (a `Lineage`-kind authored edge per
/// `#[descended_from]`; the SCIP call-graph is engine-epoch). `CfgSet` is passed through to the
/// `Locator` key so identical items under different cfg are DISTINCT nodes (ADR-070 §4.5 cfg-aware
/// identity; for frame epoch `cfg_set = CfgSet::default()`).
pub fn lower_scan_report(
    report: &antigen::scan::ScanReport,
    source_root: &Path,
    crate_name: &str,
    cfg_set: &CfgSet,
) -> (Vec<NodeFact>, Vec<EdgeFact>) {
    // --- STEP 1: collect all (file, line, item_target) triples from every attribute vec ----------
    // ScanReport has 14+ vecs; we visit the ones that carry item_target (the node-bearing records).
    // `lineage_edges` are visited separately for edges.
    let mut node_map: HashMap<NodeKey, NodeFact> = HashMap::new();

    // Helper: intern one (file, line, item_target) into node_map.
    let mut intern = |file: &Path, line: usize, item_target: &antigen::scan::ItemTarget| {
        let key = NodeKey::new(file, item_target);
        if node_map.contains_key(&key) {
            return; // already seen this (file, item_target) — dedup
        }

        let module_chain = module_chain_from_path(file, source_root);
        let item_name = item_target_name(item_target);

        // Syntactic FQ path (the floor-tier locator — SCIP supersedes at engine epoch).
        let fq: FqPath = syntactic_fq_path(crate_name, &module_chain, item_name);

        // Both digests, computed from ONE parse of the source: IdentityDigest via the §4.3
        // canonical_identity_tokens seam (tamper-evident, load-bearing-kept), ShapeDigest via the
        // name-insensitive FNV path. Falls back to traceable gap-digests if the source is unreadable.
        let abs_file = source_root.join(file);
        let (identity_digest, shape_digest) =
            digests_at_line(&abs_file, line).unwrap_or_else(|| gap_digests(&fq.path));

        let id = StromaNodeId {
            fq_path: fq,
            identity_digest: identity_digest.clone(),
            cfg_set: cfg_set.clone(),
        };

        node_map.insert(
            key,
            NodeFact {
                id,
                kind: item_target.clone(),
                identity_digest,
                shape_digest,
            },
        );
    };

    // Visit each attribute vec that carries item_target (the node-bearing records).
    for p in &report.presentations {
        intern(&p.file, p.line, &p.item_target);
    }
    for im in &report.immunities {
        intern(&im.file, im.line, &im.item_target);
    }
    for t in &report.tolerances {
        intern(&t.file, t.line, &t.item_target);
    }
    for dd in &report.deferred_defenses {
        intern(&dd.file, dd.line, &dd.item_target);
    }
    for ce in &report.convergent_evidences {
        intern(&ce.file, ce.line, &ce.item_target);
    }
    for rd in &report.recurrent_declarations {
        intern(&rd.file, rd.line, &rd.item_target);
    }
    for md in &report.mucosal_declarations {
        intern(&md.file, md.line, &md.item_target);
    }
    for pd in &report.prescriptive_declarations {
        intern(&pd.file, pd.line, &pd.item_target);
    }
    for df in &report.defenses {
        intern(&df.file, df.line, &df.item_target);
    }
    // `MarkedUnknown` carries `file` + `line` but NOT `item_target` (it's a marker-on-item, not
    // an item-identity record — it has structural_digest + shape_digest but no ItemTarget field).
    // Engine epoch: wire marked_unknowns via a `(file, line)` → `StromaNodeId` reverse-lookup.

    // --- STEP 2: edges ----------------------------------------------------------------------------
    // Call-graph edges (Call/Import/TypeUse) are engine-epoch (SCIP feeder — not wired here).
    //
    // `ScanReport::lineage_edges` carry antigen-declaration-level lineage (child/parent ANTIGEN
    // TYPES by name, not code-item `ItemTarget` pairs), so they don't map cleanly onto
    // `EdgeKind::Lineage` (which connects `StromaNodeId`-keyed items). Frame epoch: no edges.
    // Engine epoch: wire `lineage_edges` via the antigen-declaration lookup + SCIP call-edges.
    let edges: Vec<EdgeFact> = Vec::new();

    let nodes: Vec<NodeFact> = node_map.into_values().collect();
    (nodes, edges)
}

/// Extract the string name from an `ItemTarget`.
const fn item_target_name(target: &antigen::scan::ItemTarget) -> &str {
    match target {
        antigen::scan::ItemTarget::Struct(n)
        | antigen::scan::ItemTarget::Enum(n)
        | antigen::scan::ItemTarget::Trait(n)
        | antigen::scan::ItemTarget::Fn(n)
        | antigen::scan::ItemTarget::TypeAlias(n) => n.as_str(),
        // Non-name-bearing variants (impl, macro invocation) — use a placeholder.
        _ => "__impl__",
    }
}

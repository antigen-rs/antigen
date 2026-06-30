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
        let fq: FqPath = syntactic_fq_path(crate_name, &module_chain, &item_name);

        // Both digests, computed from ONE parse of the source: IdentityDigest via the §4.3
        // canonical_identity_tokens seam (tamper-evident, load-bearing-kept), ShapeDigest via the
        // name-insensitive FNV path. Falls back to traceable gap-digests if the source is unreadable.
        let abs_file = source_root.join(file);
        let (identity_digest, shape_digest) =
            digests_at_line(&abs_file, line).unwrap_or_else(|| gap_digests(&fq.path));

        let id = StromaNodeId {
            fq_path: fq,
            identity_digest,
            cfg_set: cfg_set.clone(),
        };

        node_map.insert(
            key,
            NodeFact {
                id,
                kind: item_target.clone(),
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

/// The subject of an impl block: `<Type as Trait>` for a trait impl, bare `Type` for an inherent one.
/// Shared by the `Impl` / `ImplFn` / `ImplConst` arms so the trait-qualification renders one way.
fn impl_subject(target_type: &str, trait_path: Option<&str>) -> String {
    trait_path.map_or_else(
        || target_type.to_string(),
        |tr| format!("<{target_type} as {tr}>"),
    )
}

/// Derive a DISTINGUISHING path-segment name from an `ItemTarget`.
///
/// The simple name-bearing variants yield their bare ident. The impl-family variants do NOT — they
/// carry `target_type`/`trait_path`/`fn_name` instead of a single ident, and collapsing them all to
/// one placeholder (`"__impl__"`) would re-import the bare-name collision the frame exists to close
/// (two `impl` blocks in one module → the SAME `fq_path`, distinguishable only by body digest). So
/// each impl-family variant is rendered to a path segment that preserves its distinguishing parts:
/// `<Type as Trait>` for a trait impl, `Type` for an inherent impl, `Type::method` / `Type::CONST`
/// for impl members, `Trait::method` for a trait method, `Enum::Variant` for a variant. (`::` inside
/// a segment is fine — the segment is a leaf identifier within `crate::mod::<here>`, never re-split.)
fn item_target_name(target: &antigen::scan::ItemTarget) -> String {
    use antigen::scan::ItemTarget as IT;
    match target {
        IT::Struct(n)
        | IT::Enum(n)
        | IT::Trait(n)
        | IT::Fn(n)
        | IT::TypeAlias(n)
        | IT::Const(n)
        | IT::Static(n)
        | IT::Union(n) => n.clone(),
        // A trait impl: `<Type as Trait>`; an inherent impl: just the `Type`.
        IT::Impl {
            trait_path,
            target_type,
        } => impl_subject(target_type, trait_path.as_deref()),
        // An impl method: `<subject>::method`.
        IT::ImplFn {
            trait_path,
            target_type,
            fn_name,
        } => format!(
            "{}::{fn_name}",
            impl_subject(target_type, trait_path.as_deref())
        ),
        // An impl const: `<subject>::CONST`.
        IT::ImplConst {
            trait_path,
            target_type,
            const_name,
        } => format!(
            "{}::{const_name}",
            impl_subject(target_type, trait_path.as_deref())
        ),
        // A trait method declaration: `Trait::method`.
        IT::TraitFn {
            trait_name,
            fn_name,
        } => format!("{trait_name}::{fn_name}"),
        // An enum variant: `Enum::Variant`.
        IT::EnumVariant {
            enum_name,
            variant_name,
        } => format!("{enum_name}::{variant_name}"),
        // Genuinely unidentifiable (a macro invocation, an unparsed item): a stable placeholder. The
        // body-digest still distinguishes two such nodes in the same module via `identity_digest`.
        IT::Unknown { .. } => "__unknown__".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use antigen::scan::ItemTarget as IT;

    use super::item_target_name;

    // REGRESSION (survey find): the impl-family variants must NOT all collapse to one placeholder —
    // that would re-import the bare-name collision the frame exists to close (two impls in one module
    // → the SAME fq_path). Each impl-family variant must yield a DISTINGUISHING path segment.

    #[test]
    fn two_inherent_impls_get_distinct_names() {
        let foo = IT::Impl {
            trait_path: None,
            target_type: "Foo".into(),
        };
        let bar = IT::Impl {
            trait_path: None,
            target_type: "Bar".into(),
        };
        assert_ne!(
            item_target_name(&foo),
            item_target_name(&bar),
            "impl Foo and impl Bar collapsed to the same path segment — the bare-name defect re-imported."
        );
    }

    #[test]
    fn trait_impl_distinguished_from_inherent_and_by_trait() {
        let inherent = IT::Impl {
            trait_path: None,
            target_type: "Foo".into(),
        };
        let as_drop = IT::Impl {
            trait_path: Some("Drop".into()),
            target_type: "Foo".into(),
        };
        let as_clone = IT::Impl {
            trait_path: Some("Clone".into()),
            target_type: "Foo".into(),
        };
        // `impl Foo`, `impl Drop for Foo`, `impl Clone for Foo` are THREE distinct nodes.
        assert_ne!(item_target_name(&inherent), item_target_name(&as_drop));
        assert_ne!(item_target_name(&as_drop), item_target_name(&as_clone));
        assert_ne!(item_target_name(&inherent), item_target_name(&as_clone));
    }

    #[test]
    fn methods_on_different_types_get_distinct_names() {
        let foo_run = IT::ImplFn {
            trait_path: None,
            target_type: "Foo".into(),
            fn_name: "run".into(),
        };
        let bar_run = IT::ImplFn {
            trait_path: None,
            target_type: "Bar".into(),
            fn_name: "run".into(),
        };
        // Two `run` methods on different types must not collide (the ImplFn bare-name case).
        assert_ne!(
            item_target_name(&foo_run),
            item_target_name(&bar_run),
            "Foo::run and Bar::run collapsed — methods are not distinguished by their owning type."
        );
    }

    #[test]
    fn enum_variants_distinguished_within_and_across_enums() {
        let a_x = IT::EnumVariant {
            enum_name: "A".into(),
            variant_name: "X".into(),
        };
        let a_y = IT::EnumVariant {
            enum_name: "A".into(),
            variant_name: "Y".into(),
        };
        let b_x = IT::EnumVariant {
            enum_name: "B".into(),
            variant_name: "X".into(),
        };
        assert_ne!(item_target_name(&a_x), item_target_name(&a_y)); // A::X ≠ A::Y
        assert_ne!(item_target_name(&a_x), item_target_name(&b_x)); // A::X ≠ B::X
    }

    #[test]
    fn identical_items_collide_the_teeth() {
        // NC: two IDENTICAL targets MUST yield the SAME name (so the dedup/identity is deterministic,
        // not over-disambiguating). Proves the above tests reject only genuine distinctions.
        let a = IT::Fn("handle".into());
        let b = IT::Fn("handle".into());
        assert_eq!(item_target_name(&a), item_target_name(&b));
        let m1 = IT::ImplFn {
            trait_path: Some("Drop".into()),
            target_type: "Foo".into(),
            fn_name: "drop".into(),
        };
        let m2 = IT::ImplFn {
            trait_path: Some("Drop".into()),
            target_type: "Foo".into(),
            fn_name: "drop".into(),
        };
        assert_eq!(item_target_name(&m1), item_target_name(&m2));
    }
}

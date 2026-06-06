//! Post-collection finalize pass — fingerprint synthesis + lineage propagation.
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). `finalize_report` is the single source of truth
//! for the post-collection pass ORDER (fingerprint + generates synthesis, then
//! `#[descended_from]` inheritance propagation), shared by both `scan_workspace`
//! and `scan_workspace_multi_crate` so the two callers cannot drift. It drives
//! the synthesis pass (the `parse` leaf underneath that) + `synthesize_inherited_presentations`
//! (with its `transitive_ancestors_dfs` / `propagate_ancestors_to_descendant`
//! helpers, already cycle/depth-guarded in-band). A pure mutation of its input
//! report; it holds no stop-authority (single-conductor invariant, ADR-036).
//!
//! API-invisible: crate-internal; `finalize_report` + `synthesize_inherited_presentations`
//! are re-exported `pub(crate)` at the scan root (the walk + multi-crate passes
//! drive them).

use std::path::PathBuf;

use super::{
    AntigenDeclaration, ItemTarget, ParseFailure, Presentation, ProvenanceEntry, ScanReport,
    generates_synthesis_pass, locus_matches, synthesis_pass,
};

/// Run the post-collection passes that turn a raw explicit-collection
/// [`ScanReport`] into a finished one: fingerprint synthesis + lineage
/// propagation, in the ADR-mandated order.
///
/// Extracted from [`scan_workspace`](crate::scan::scan_workspace) so the **single source of truth** for
/// the pass ordering is shared with [`scan_workspace_multi_crate`](crate::scan::scan_workspace_multi_crate)'s
/// merged-report finalize. The two callers differ only in *what* they feed
/// in — one a single crate's tree, the other the unioned member reports —
/// but the synthesis/propagation semantics must stay identical, so they
/// route through here.
///
/// Pre-conditions the caller must establish first:
/// - `report.lineage_edges` is deduped (ADR-018 §Edge-level dedup).
/// - cycle/depth detection has run (diagnostics already in `parse_failures`).
/// - every record's `canonical_path` is in its final state (member-aware
///   stamping + cross-member parent re-resolution already applied for the
///   multi-crate caller; `None` for the intra-workspace single-crate caller).
///
/// `parsed_files` is the `(path, syn::File)` cache from the collection walk,
/// reused by the synthesis pass so it never re-reads or re-parses a file.
pub fn finalize_report(report: &mut ScanReport, parsed_files: &[(PathBuf, syn::File)]) {
    // ---- Fingerprint synthesis pass ----
    //
    // After explicit-collection, walk every file again and emit synthetic
    // `Presentation { match_kind: FingerprintMatch }` records for items that
    // match a declared antigen's fingerprint but weren't explicitly annotated.
    //
    // Only antigens with a parseable fingerprint participate. Parse failures
    // are appended to `report.parse_failures` as non-fatal diagnostics —
    // a malformed fingerprint never silently suppresses all matching.
    //
    // Deduplication: an item that already has an explicit `#[presents(X)]`
    // gets no synthetic match for antigen X — `addresses()` is the bridge.

    // Build the set of parseable fingerprints once, before the file re-walk.
    // Collect parse failures separately to avoid aliasing `report` inside the
    // iterator (immutable borrow on `report.antigens` + mutable push on
    // `report.parse_failures` would conflict at borrow-check time).
    let mut fp_parse_failures: Vec<ParseFailure> = Vec::new();
    let fingerprints: Vec<(String, antigen_fingerprint::Fingerprint)> = report
        .antigens
        .iter()
        .filter_map(|ag| {
            let raw = ag.fingerprint.as_deref()?;
            match antigen_fingerprint::Fingerprint::parse(raw) {
                Ok(fp) => Some((ag.type_name.clone(), fp)),
                Err(e) => {
                    fp_parse_failures.push(ParseFailure {
                        file: ag.file.clone(),
                        error: format!(
                            "antigen `{}`: fingerprint failed to re-parse during synthesis: {e}",
                            ag.type_name
                        ),
                    });
                    None
                },
            }
        })
        .collect();
    report.parse_failures.extend(fp_parse_failures);

    if !fingerprints.is_empty() {
        // Build declaration-site set for self-match suppression (DX finding 4).
        let declaration_sites: std::collections::HashSet<(String, PathBuf)> = report
            .antigens
            .iter()
            .map(|ag| (ag.type_name.clone(), ag.file.clone()))
            .collect();
        synthesis_pass(parsed_files, &fingerprints, &declaration_sites, report);
    }

    // ---- Generates-synthesis pass (ADR-014) ----
    //
    // For every macro INVOCATION whose macro name matches an
    // `#[antigen_generates(X, ...)]` declaration on a macro DEFINITION, emit a
    // synthetic Presentation at the invocation site. Same-workspace only
    // (§A3); cross-crate macro-output recognition (§A4) is deferred. Runs after
    // fingerprint synthesis so generated presentations dedup against any
    // co-located explicit `#[presents]`/fingerprint match.
    if !report.generates_declarations.is_empty() {
        generates_synthesis_pass(parsed_files, report);
    }

    // ---- Lineage propagation pass (ADR-018) ----
    //
    // Runs AFTER cycle detection (Ok ⇒ lineage_edges is a DAG by
    // construction) AND after fingerprint synthesis (so that inherited
    // presentations can dedup against fingerprint matches too).
    //
    // The pass walks transitive closure of lineage edges per child
    // antigen, attaching ancestor presentations as inherited Presentations
    // on the descendant. Diamond inheritance (two paths to the same
    // ancestor) collapses to one Presentation per (antigen, item,
    // canonical_path) tuple with set-unioned `inherited_from` chain.
    //
    // Orphaned + dangling edges are not walked through (ADR-018
    // §Stale-lineage interaction).
    synthesize_inherited_presentations(report);
}

/// Walk transitive closure of `#[descended_from]` lineage edges and
/// attach ancestor presentations as inherited Presentations on each
/// descendant. ADR-018 §"The synthesis algorithm".
///
/// Pre-conditions assumed by caller:
/// - `report.lineage_edges` has been deduped (ADR-018 §Edge-level dedup).
/// - Cycle detection has run clean (the graph is a DAG).
///
/// Defense-in-depth: a per-source-node `visited` `HashSet` guards against
/// any cycle the upstream check might have missed (ADR-018 Finding 4 —
/// "trust the upstream cycle detection for correctness; this visited set
/// is defense-in-depth against refactor accidents, not a correctness
/// dependency").
///
/// Algorithm overview (per descendant antigen as DFS source):
///   1. Build a `(type_name, canonical_path)` -> [`AntigenDeclaration`]
///      index for parent/child endpoint validation.
///   2. Build adjacency `child_key → Vec<parent_key>` from the deduped
///      lineage edge set, *skipping* orphaned edges (parent not in
///      antigen index) and dangling-child edges (child not in antigen
///      index). The propagation walk never traverses those.
///   3. Build a `(antigen_type, canonical_path) → Vec<presentation_idx>`
///      index over a snapshot of `report.presentations`.
///   4. For each `AntigenDeclaration` with at least one outgoing
///      adjacency entry, collect transitive ancestor identities via
///      iterative DFS (per-call `visited` `HashSet`, defense-in-depth).
///   5. For each ancestor's presentation, either:
///      - merge `ProvenanceEntry` into an existing Presentation's
///        `inherited_from` via set-union (diamond dedup, keyed on the
///        ADR-018 three-tuple `(antigen_type, item_target, canonical_path)`),
///      - or append a new inherited Presentation at the descendant's
///        site, preserving the ancestor's `match_kind`.
pub fn synthesize_inherited_presentations(report: &mut ScanReport) {
    use std::collections::HashMap;

    // Build (type_name, canonical_path) -> AntigenDeclaration index.
    let antigen_by_key: HashMap<AntigenKey, AntigenDeclaration> = report
        .antigens
        .iter()
        .map(|a| ((a.type_name.clone(), a.canonical_path.clone()), a.clone()))
        .collect();

    // Build adjacency: child antigen → list of parent antigen keys.
    // Skip dangling-child edges (child not in antigen index) — the
    // descendant has no record for inheritance to flow into.
    // Skip orphaned edges (parent not in antigen index) — the propagation
    // walk does not walk through unknown ancestors (ADR-018 §Stale-lineage).
    let mut adjacency: LineageAdjacency = LineageAdjacency::new();
    for e in &report.lineage_edges {
        let child_key = (e.child.clone(), e.child_canonical_path.clone());
        let parent_key = (e.parent.clone(), e.parent_canonical_path.clone());
        if !antigen_by_key.contains_key(&child_key) || !antigen_by_key.contains_key(&parent_key) {
            continue;
        }
        adjacency.entry(child_key).or_default().push(parent_key);
    }

    // Index of existing presentations by (antigen_type, canonical_path)
    // for fast ancestor-presentation lookup. Cloned (immutable snapshot)
    // — we'll modify report.presentations during the walk, and reading
    // from a snapshot keeps the source-of-truth stable.
    let presentations_snapshot: Vec<Presentation> = report.presentations.clone();
    let mut presentations_by_antigen: HashMap<AntigenKey, Vec<usize>> = HashMap::new();
    for (idx, p) in presentations_snapshot.iter().enumerate() {
        presentations_by_antigen
            .entry((p.antigen_type.clone(), p.canonical_path.clone()))
            .or_default()
            .push(idx);
    }

    // For each child antigen with outgoing edges, walk transitive
    // ancestors and propagate their presentations.
    //
    // Iteration order: process antigens in declaration order for
    // determinism. (HashMap iteration order is randomised.)
    for child_decl in report.antigens.clone() {
        let child_key = (
            child_decl.type_name.clone(),
            child_decl.canonical_path.clone(),
        );
        if !adjacency.contains_key(&child_key) {
            continue;
        }
        let ancestors_in_order = transitive_ancestors_dfs(&adjacency, &child_key);
        propagate_ancestors_to_descendant(
            report,
            &child_decl,
            &ancestors_in_order,
            &presentations_snapshot,
            &presentations_by_antigen,
        );
    }
}

/// Antigen identity key used by the propagation walk: bare type name +
/// `canonical_path`. Mirrors the ADR-017 `(type_name, canonical_path)`
/// identity tuple.
type AntigenKey = (String, Option<String>);

/// Adjacency map from a child antigen key to its parent antigen keys, used
/// during the propagation walk. Built from the (already-deduped) lineage
/// edge set after orphan + dangling-child edges are filtered out.
type LineageAdjacency = std::collections::HashMap<AntigenKey, Vec<AntigenKey>>;

/// DFS over the lineage adjacency, returning transitive ancestor keys in
/// discovery order. Defense-in-depth `visited` `HashSet` per call (ADR-018
/// Finding 4) catches any cycle the upstream check might have missed.
fn transitive_ancestors_dfs(
    adjacency: &LineageAdjacency,
    child_key: &AntigenKey,
) -> Vec<AntigenKey> {
    use std::collections::HashSet;
    let mut visited: HashSet<AntigenKey> = HashSet::new();
    let mut stack: Vec<AntigenKey> = adjacency.get(child_key).cloned().unwrap_or_default();
    let mut ancestors_in_order: Vec<AntigenKey> = Vec::new();
    while let Some(node) = stack.pop() {
        if !visited.insert(node.clone()) {
            continue;
        }
        ancestors_in_order.push(node.clone());
        if let Some(parents) = adjacency.get(&node) {
            for parent in parents.iter().rev() {
                if !visited.contains(parent) {
                    stack.push(parent.clone());
                }
            }
        }
    }
    ancestors_in_order
}

/// Attach each ancestor's presentations to the descendant antigen, either
/// merging provenance into an existing Presentation record (diamond dedup)
/// or appending a new inherited Presentation. ADR-018 §"The synthesis
/// algorithm" — the per-descendant body.
///
/// The descendant's item identity is its declaration site: antigens are
/// unit-struct declarations per ADR-009 / ADR-010, so the synthesized
/// Presentations land on `ItemTarget::Struct(type_name)`.
fn propagate_ancestors_to_descendant(
    report: &mut ScanReport,
    child_decl: &AntigenDeclaration,
    ancestors_in_order: &[AntigenKey],
    presentations_snapshot: &[Presentation],
    presentations_by_antigen: &std::collections::HashMap<AntigenKey, Vec<usize>>,
) {
    use std::collections::BTreeSet;
    let descendant_item_target = ItemTarget::Struct(child_decl.type_name.clone());
    let descendant_item_kind = "struct".to_string();

    for ancestor_key in ancestors_in_order {
        let provenance = ProvenanceEntry {
            antigen_type: ancestor_key.0.clone(),
            canonical_path: ancestor_key.1.clone(),
        };
        let Some(ancestor_pres_indices) = presentations_by_antigen.get(ancestor_key) else {
            continue;
        };
        for &ancestor_pres_idx in ancestor_pres_indices {
            let ancestor_pres = &presentations_snapshot[ancestor_pres_idx];

            // Three-tuple dedup key per ADR-018 §"Diamond dedup":
            // (antigen_type, item_target, canonical_path). Linear scan
            // of `report.presentations` — fine at v0.1 fixture sizes
            // (deepest fixture has ~10 entries). If realistic workspaces
            // grow large lineage graphs, this is the spot to introduce
            // an `(antigen_type, item_target_key, canonical_path)`
            // index keyed by descendant antigen. Performance pressure
            // is the recognition trigger (per ADR-006); no premature
            // optimisation.
            let existing_idx = report.presentations.iter().position(|p| {
                p.antigen_type == ancestor_pres.antigen_type
                    && p.canonical_path == ancestor_pres.canonical_path
                    && p.item_target.addresses(&descendant_item_target)
                    && locus_matches(
                        p.file.as_path(),
                        p.canonical_path.as_deref(),
                        child_decl.file.as_path(),
                        child_decl.canonical_path.as_deref(),
                    )
            });

            if let Some(idx) = existing_idx {
                let existing = &mut report.presentations[idx];
                let mut chain: BTreeSet<ProvenanceEntry> = existing
                    .inherited_from
                    .take()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                chain.insert(provenance.clone());
                existing.inherited_from = Some(chain.into_iter().collect());
            } else {
                report.presentations.push(Presentation {
                    antigen_type: ancestor_pres.antigen_type.clone(),
                    file: child_decl.file.clone(),
                    line: child_decl.line,
                    item_kind: descendant_item_kind.clone(),
                    item_target: descendant_item_target.clone(),
                    match_kind: ancestor_pres.match_kind.clone(),
                    canonical_path: ancestor_pres.canonical_path.clone(),
                    inherited_from: Some(vec![provenance.clone()]),
                    structural_fingerprint: ancestor_pres.structural_fingerprint.clone(),
                    // Site-attached evidence (ADR-029) propagates with the
                    // inherited presentation: if the ancestor's presents-site
                    // carried `requires=`/`proof=`, the descendant inherits the
                    // same evidence claim. State-7 re-attestation still applies.
                    requires_predicate: ancestor_pres.requires_predicate.clone(),
                    proof: ancestor_pres.proof.clone(),
                });
            }
        }
    }
}

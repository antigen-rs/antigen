//! Lineage safety pass — `#[descended_from]` edge dedup + cycle/depth detection.
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). One pass of the scan pipeline: `dedupe_lineage_edges`
//! collapses duplicate edges (ADR-018 four-tuple key), `detect_lineage_failures`
//! flags cycles + over-depth chains (already correctly damped IN-BAND by
//! `MAX_LINEAGE_DEPTH` + a visited-set + iterative DFS — an intrinsic depth cap,
//! distinct from a SCRAM kill-switch; left as-is per ADR-036 §The out-of-band
//! invariant). A pure fn of its input edges; it holds no stop-authority.
//!
//! API-invisible: these passes are crate-internal (`scan_workspace` /
//! `scan_workspace_multi_crate` drive them); the scan module root re-exports them
//! `pub(crate)` for the passes + the test module.

use std::path::PathBuf;

use super::{LineageEdge, ParseFailure};

/// Deduplicate lineage edges by the ADR-018 four-tuple key and emit one
/// [`ParseFailure`] per collapsed duplicate group. BUG-A3-001 fix +
/// ADR-018 §"Edge-level dedup".
///
/// The dedup key is `(child, parent, child_canonical_path,
/// parent_canonical_path)`. Same-name edges at different
/// `canonical_path` values are structurally distinct and NOT duplicates
/// (a workspace depending on `foo@1.0.0::P` and `foo@2.0.0::P`
/// legitimately has both edges).
///
/// Two `#[descended_from(B)]` attributes on the same struct `A` produce
/// two identical `LineageEdge` entries. Without this pre-pass the DFS
/// in [`detect_lineage_failures`] would silently swallow the second one
/// (black-skip path), so duplicates would never reach the user. Per
/// ADR-004 implicit-to-explicit elevation, dedup surfaces collapsed
/// duplicates as explicit diagnostics on the `parse_failures` channel.
///
/// Returns the deduped edge `Vec` and the failure list. Both
/// [`detect_lineage_failures`] (cycle/depth detection) AND the
/// propagation walk (D1.5 commit 4) consume the deduped output —
/// dedup is structurally upstream of both per ADR-018 §"Implementation
/// order in `scan_workspace`".
pub fn dedupe_lineage_edges(edges: &[LineageEdge]) -> (Vec<LineageEdge>, Vec<ParseFailure>) {
    use std::collections::{HashMap, HashSet};

    // Four-tuple key: (child, parent, child_canonical_path, parent_canonical_path).
    // Borrow the inner string values; the lifetime of the returned
    // Vec<LineageEdge> is independent (we clone on insert).
    type DedupKey<'a> = (&'a str, &'a str, Option<&'a str>, Option<&'a str>);
    fn key_of(edge: &LineageEdge) -> DedupKey<'_> {
        (
            edge.child.as_str(),
            edge.parent.as_str(),
            edge.child_canonical_path.as_deref(),
            edge.parent_canonical_path.as_deref(),
        )
    }

    let mut counts: HashMap<DedupKey<'_>, usize> = HashMap::new();
    for edge in edges {
        *counts.entry(key_of(edge)).or_insert(0) += 1;
    }

    // Walk edges in source order: emit the first occurrence per key into
    // the deduped slice, flag duplicates as parse_failures (one per
    // duplicate group, anchored at the first occurrence).
    let mut emitted: HashSet<DedupKey<'_>> = HashSet::new();
    let mut deduped: Vec<LineageEdge> = Vec::with_capacity(edges.len());
    let mut failures: Vec<ParseFailure> = Vec::new();
    for edge in edges {
        let key = key_of(edge);
        let count = counts.get(&key).copied().unwrap_or(0);
        if emitted.insert(key) {
            deduped.push(edge.clone());
            if count > 1 {
                failures.push(ParseFailure {
                    file: edge.file.clone(),
                    error: format!(
                        "duplicate #[descended_from({})] declarations on `{}` \
                         (first at line {}); structural lies surface as \
                         diagnostics rather than being silently collapsed \
                         (ADR-004 implicit-to-explicit elevation)",
                        edge.parent, edge.child, edge.line
                    ),
                });
            }
        }
    }
    (deduped, failures)
}

/// Detect circular and over-deep `#[descended_from]` chains.
///
/// ATK-A3-002. Iterative DFS with white/gray/black coloring on the lineage
/// graph (`child → parent` edges). Stack frames carry `(node, child_index)`
/// so the algorithm is iterative — no recursion → no stack-overflow risk on
/// pathological inputs.
///
/// Coloring discipline:
/// - **white** (absent from `color`): not yet visited.
/// - **gray** (`= 1`): on the current DFS path. Re-encountering a gray node
///   closes a cycle.
/// - **black** (`= 2`): fully processed. Re-encountering a black node is a
///   shortcut — its subtree was already proven cycle-free in this scan.
///
/// Returns one [`ParseFailure`] per discovered cycle (cycle anchored at the
/// first edge that closed it) and one per chain that exceeded `max_depth`.
/// The chain text is preserved in the `error` string — the structured-enum
/// representation of `ParseFailure` is an open question (see scope-lock §5
/// and aristotle's pending Phase 1-8 ruling).
pub fn detect_lineage_failures(edges: &[LineageEdge], max_depth: usize) -> Vec<ParseFailure> {
    use std::collections::HashMap;

    // BUG-A3-001 + ADR-018 §"Edge-level dedup": this function ASSUMES edges
    // are already deduped (caller invariant). `scan_workspace` runs
    // `dedupe_lineage_edges()` before calling here; unit-test callers that
    // pass raw edges with duplicates may observe silent black-skip on the
    // dup pair — that's by design at this layer. The dedup contract is
    // tested separately against `dedupe_lineage_edges` directly.
    let mut failures: Vec<ParseFailure> = Vec::new();

    // Build adjacency: child → list of (parent, edge-index). The edge-index
    // lets us recover the source location (file + line) of the closing edge
    // when a cycle is reported, which matters for human-readable diagnostics.
    let mut adjacency: HashMap<&str, Vec<(&str, usize)>> = HashMap::new();
    for (idx, edge) in edges.iter().enumerate() {
        adjacency
            .entry(edge.child.as_str())
            .or_default()
            .push((edge.parent.as_str(), idx));
    }

    let mut color: HashMap<&str, u8> = HashMap::new();
    // Seen-cycle set keyed by the canonicalised cycle (smallest rotation of
    // the node sequence) so we don't report the same loop multiple times
    // when entered from different start nodes.
    let mut reported_cycles: std::collections::HashSet<Vec<String>> =
        std::collections::HashSet::new();

    // For deterministic output (tests, diff stability) iterate roots in the
    // order edges were discovered rather than HashMap iteration order.
    let mut roots_in_order: Vec<&str> = Vec::new();
    let mut seen_roots: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for edge in edges {
        let c = edge.child.as_str();
        if seen_roots.insert(c) {
            roots_in_order.push(c);
        }
    }

    for &root in &roots_in_order {
        if color.contains_key(root) {
            continue;
        }
        // Stack frame: (node, next-child-index, file-of-edge-into-node).
        // The path vector is maintained alongside so cycles can render the
        // full chain text on closure. file-of-edge is `None` for the root.
        let mut stack: Vec<(&str, usize)> = Vec::new();
        let mut path: Vec<&str> = Vec::new();

        color.insert(root, 1);
        stack.push((root, 0));
        path.push(root);

        while let Some(&mut (node, ref mut idx)) = stack.last_mut() {
            // Hard depth guard — per ADR-005 Amendment 3 sibling to cycle
            // detection. Path length includes the current node, so a chain
            // a -> b -> c at this frame has path.len() == 3.
            if path.len() > max_depth {
                // Anchor the diagnostic at the edge that pushed us over —
                // the most recent edge in the path.
                let leaf = *path.last().unwrap_or(&node);
                let anchor = adjacency
                    .get(leaf)
                    .and_then(|v| v.first())
                    .and_then(|(_, edge_idx)| edges.get(*edge_idx))
                    .map_or_else(PathBuf::new, |e| e.file.clone());
                failures.push(ParseFailure {
                    file: anchor,
                    error: format!(
                        "#[descended_from] chain exceeds maximum depth ({max_depth}) at \
                         `{leaf}`; chain: {}",
                        path.join(" -> ")
                    ),
                });
                // Mark the leaf black and pop so the rest of the graph is
                // still examined for other failures.
                color.insert(node, 2);
                stack.pop();
                path.pop();
                continue;
            }

            let children = adjacency.get(node).map_or(&[][..], Vec::as_slice);
            if *idx >= children.len() {
                // All children processed — paint black and unwind one level.
                color.insert(node, 2);
                stack.pop();
                path.pop();
                continue;
            }

            let (child, edge_idx) = children[*idx];
            *idx += 1;

            match color.get(child).copied().unwrap_or(0) {
                0 => {
                    // White — descend into it.
                    color.insert(child, 1);
                    path.push(child);
                    stack.push((child, 0));
                }
                1 => {
                    // Gray — closing a cycle. Capture the chain from the
                    // first occurrence of `child` in `path` to the current
                    // node, then back to `child`.
                    let cycle_start = path.iter().position(|n| *n == child).unwrap_or(0);
                    let bare_refs: Vec<&str> = path[cycle_start..].to_vec();
                    let mut cycle_chain: Vec<String> =
                        bare_refs.iter().map(|s| (*s).to_string()).collect();
                    cycle_chain.push(child.to_string());

                    // Canonicalise (smallest rotation of the bare cycle,
                    // excluding the duplicated tail) for dedup.
                    let canonical = canonicalise_cycle(&bare_refs);
                    if reported_cycles.insert(canonical) {
                        let edge = edges.get(edge_idx);
                        let file = edge.map_or_else(PathBuf::new, |e| e.file.clone());
                        let line = edge.map_or(0, |e| e.line);
                        failures.push(ParseFailure {
                            file,
                            error: format!(
                                "#[descended_from] forms a cycle (closing edge at line \
                                 {line}): {}",
                                cycle_chain.join(" -> ")
                            ),
                        });
                    }
                    // Don't descend into the gray child — that would loop.
                    // Continue with the next child of `node`.
                }
                _ => {
                    // Black — already proven cycle-free in this scan; skip.
                }
            }
        }
    }

    failures
}

/// Canonicalise a cycle as the lexicographically smallest rotation of its
/// node sequence, so cycles entered from different start nodes deduplicate.
///
/// Input is the bare cycle `[a, b, c]` (without the repeated tail node) —
/// `[a, b, c]` and `[b, c, a]` are the same cycle and produce the same
/// canonical form `[a, b, c]` here.
pub fn canonicalise_cycle(bare: &[&str]) -> Vec<String> {
    if bare.is_empty() {
        return Vec::new();
    }
    let n = bare.len();
    let mut best_start = 0;
    for start in 1..n {
        // Compare rotation starting at `start` vs current best.
        for i in 0..n {
            let a = bare[(start + i) % n];
            let b = bare[(best_start + i) % n];
            if a < b {
                best_start = start;
                break;
            } else if a > b {
                break;
            }
        }
    }
    (0..n)
        .map(|i| bare[(best_start + i) % n].to_string())
        .collect()
}

//! The collection walk — `scan_workspace` (pass 1 of the scan pipeline).
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). `scan_workspace` is the `WalkDir` file-walk that
//! drives one `parse::ScanVisitor` per `.rs` file (the explicit-collection pass),
//! then runs the lineage safety pass + the shared `finalize` pass. It is a
//! pure-ish directory scanner — the purity invariant ADR-036 banks (each
//! pass/detector a fn of its input; no pass holds stop-authority). Per ADR-036
//! §The out-of-band invariant + the build-time supersede-note, SCRAM is NOT
//! threaded into this walk (scan has no detector loop for it; the cascade-governor
//! lives at the command-orchestration layer, above the whole pipeline) — what is
//! banked here is the walk staying pure so a governor is insertable later.
//!
//! API-invisible: `scan_workspace` is re-exported from the scan root via `pub use`
//! exactly as before.

use std::path::{Path, PathBuf};

use antigen_macros::{antigen_tolerance, presents};
use syn::visit::Visit;
use walkdir::WalkDir;

use super::{
    dedupe_lineage_edges, detect_lineage_failures, finalize_report, ParseFailure, ScanReport,
    ScanVisitor, MAX_LINEAGE_DEPTH,
};

/// Scan a directory tree, reading every `.rs` file and extracting antigen
/// declarations.
///
/// `excluded_dirs` is a list of directory names (not full paths) to skip during
/// the walk; the default is `["target", ".git", "node_modules"]` if `None` is
/// passed.
///
/// **Mucosal boundary detection scope**: this scan ONLY finds explicitly
/// declared `#[mucosal]` / `#[mucosal_delegate]` / `#[mucosal_tolerant]`
/// annotations. Trust-boundary sites that lack an explicit annotation are
/// not surfaced — the scan cannot infer implicit boundaries from parameter
/// types or call sites. See
/// [`crate::stdlib::dogfood::ScannerBoundaryFalseNegative`].
///
/// # Errors
///
/// Currently never returns `Err` — IO errors during the walk (unreadable
/// files, permission denied, etc.) are silently skipped, and parse errors
/// are recorded in `ScanReport::parse_failures` rather than aborting the
/// scan. The `std::io::Result` return type reserves space for future
/// failure modes (e.g., a `--strict` mode that fails the walk on the first
/// unreadable file, or an out-of-memory cap on `parsed_files` cache size).
/// Callers should treat any `Err` as a hard scan failure and surface the
/// error to the user.
#[presents(ScannerBoundaryFalseNegative)]
#[antigen_tolerance(
    ScannerBoundaryFalseNegative,
    rationale = "Accepted v0.2 limitation: the scan is a static-heuristic walk that surfaces only \
                 explicitly-declared #[mucosal]/#[presents] sites — it cannot infer implicit trust \
                 boundaries from parameter types or call sites, by design (ADR-006 recognition-not-design: \
                 the scan recognizes declared structure, it does not guess). Adopters mark boundaries \
                 explicitly; the false-negative on unmarked sites is the honest cost of not guessing.",
    until = "v0.3"
)]
pub fn scan_workspace(root: &Path, excluded_dirs: Option<&[&str]>) -> std::io::Result<ScanReport> {
    let default_exclusions = ["target", ".git", "node_modules"];
    let exclusions = excluded_dirs.unwrap_or(&default_exclusions);

    let mut report = ScanReport::default();

    // Cache parsed files between pass 1 (collect explicit declarations) and
    // pass 2 (synthesize fingerprint matches) to avoid re-parsing every .rs.
    let mut parsed_files: Vec<(PathBuf, syn::File)> = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|x| *x == name)
            } else {
                true
            }
        })
    {
        let Ok(entry) = entry else { continue };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        match syn::parse_file(&content) {
            Ok(file) => {
                let file_path = entry.path().to_path_buf();
                let mut visitor = ScanVisitor::new(file_path.clone(), &mut report);
                visitor.visit_file(&file);
                report.files_scanned += 1;
                // Cache for the synthesis pass — avoids re-reading + re-parsing.
                parsed_files.push((file_path, file));
            }
            Err(e) => {
                report.parse_failures.push(ParseFailure {
                    file: entry.path().to_path_buf(),
                    error: e.to_string(),
                });
            }
        }
    }

    // ---- Lineage safety pass ----
    //
    // ATK-A3-002 — `#[descended_from]` chains require two hard entry guards
    // (ADR-005 Amendment 3 crash-resistance, both required) before any
    // propagation walk reads the edge graph:
    //
    //   1. Cycle detection — a `child → parent → ... → child` chain would
    //      cause a propagation walker to recurse indefinitely. Every cycle
    //      surfaces as one `ParseFailure` with the full chain text so the
    //      user sees which edges form the loop.
    //
    //   2. Depth limit (default 64) — bounds pathological-linear chains
    //      that aren't cyclic but blow the stack. Reports the offending
    //      child + observed depth.
    //
    // Both are emitted into `parse_failures` because they prevent correct
    // scan completion (channel taxonomy: structural error, not semantic
    // warning — the latter is `orphaned_lineage_edges()`).
    //
    // ADR-018 §"Implementation order in scan_workspace": edge-level dedup
    // (BUG-A3-001) MUST run before cycle detection AND propagation walk.
    // The deduped edge set feeds both downstream consumers; the duplicate
    // diagnostic accumulates into parse_failures alongside cycle/depth
    // failures.
    let (deduped_edges, dedup_failures) = dedupe_lineage_edges(&report.lineage_edges);
    report.lineage_edges = deduped_edges;
    report.parse_failures.extend(dedup_failures);
    let lineage_failures = detect_lineage_failures(&report.lineage_edges, MAX_LINEAGE_DEPTH);
    report.parse_failures.extend(lineage_failures);

    finalize_report(&mut report, &parsed_files);

    Ok(report)
}

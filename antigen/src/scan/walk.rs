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

use antigen_macros::{antigen_tolerance, dread, presents};
use syn::visit::Visit;
use walkdir::WalkDir;

use super::{
    MAX_LINEAGE_DEPTH, ParseFailure, ScanReport, ScanVisitor, dedupe_lineage_edges,
    detect_lineage_failures, finalize_report_with_catalog,
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
    scan_workspace_inner(root, excluded_dirs, BundledCatalog::None)
}

/// Whether (and how) to inject the bundled stdlib catalog into a scan's
/// synthesis pass (v0.4 E0).
///
/// The bundled catalog closes the **zero-hits-cliff**: a crate with zero in-tree
/// antigen declarations produces an empty `fingerprints` set, so `synthesis_pass`
/// never runs and the scan reports a false all-clear. Auto-detect injects the
/// catalog only when the crate has no in-tree antigens (the consumer-crate case);
/// `Always` injects regardless (the catalog augments any in-tree repertoire).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BundledCatalog {
    /// Do not inject (the default `scan_workspace` behaviour, unchanged).
    None,
    /// Inject only when the scan found zero in-tree antigen declarations
    /// (the `--bundled-catalog` auto-detect path).
    AutoDetect,
    /// Inject unconditionally (augment in-tree antigens with the bundled catalog).
    Always,
}

/// Scan with the bundled stdlib catalog injected (v0.4 E0).
///
/// Identical to [`scan_workspace`] except that, per `auto_detect`, antigen's
/// flagship stdlib fingerprints are merged into the synthesis pass so a
/// zero-declaration consumer crate still gets real fingerprint-match
/// presentations (closing the zero-hits-cliff). The synthesized matches are
/// tagged so the caller can carry the catalog's authored
/// [`Provenance`](crate::finding::Provenance) into the claim-scoped render
/// (ADR-043 Amendment 1 / ADR-044).
///
/// # Errors
/// Propagates the `scan_workspace` IO error (a hard scan failure).
pub fn scan_workspace_bundled_catalog(
    root: &Path,
    excluded_dirs: Option<&[&str]>,
    auto_detect: bool,
) -> std::io::Result<ScanReport> {
    let mode = if auto_detect {
        BundledCatalog::AutoDetect
    } else {
        BundledCatalog::Always
    };
    scan_workspace_inner(root, excluded_dirs, mode)
}

// The `io::Result` mirrors the public `scan_workspace` contract (it reserves
// space for future hard-failure modes — see that fn's `# Errors`); this private
// inner fn currently never returns `Err`, hence the allow.
//
// Dogfood mark (v0.4 keystone): this walk swallows a file's read error with
// `let Ok(content) = read_to_string(..) else { continue };` and proceeds — an
// unreadable file (permissions, non-UTF-8) is silently skipped, so the scan
// reports a CLEAN result over an INCOMPLETE corpus. That is antigen's own
// silent-failure class (a clean verdict that means "found nothing" indistinguishable
// from "couldn't look"). A genuinely-felt site, twinned with `collect_function_index`
// in audit/immunity.rs (same WalkDir + read-or-continue + parse-or-skip shape).
#[dread(
    trigger = "scan_workspace_inner silently `continue`s past a file it cannot read \
               (read_to_string Err -> skip), so an unreadable file lowers coverage \
               without lowering the all-clear verdict: 'reported clean' conflates \
               'found nothing' with 'could not look'. No counter, no surfaced skip."
)]
#[allow(clippy::unnecessary_wraps)]
fn scan_workspace_inner(
    root: &Path,
    excluded_dirs: Option<&[&str]>,
    bundled: BundledCatalog,
) -> std::io::Result<ScanReport> {
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
            },
            Err(e) => {
                report.parse_failures.push(ParseFailure {
                    file: entry.path().to_path_buf(),
                    error: e.to_string(),
                });
            },
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

    // v0.4 E0 — bundled stdlib catalog injection. Decide whether to merge the
    // compile-in catalog fingerprints into the synthesis pass. AutoDetect only
    // injects when there are no in-tree antigens (the consumer-crate zero-hits-
    // cliff case); Always injects unconditionally. The catalog fingerprints are
    // appended to the in-tree set finalize builds, so a zero-declaration crate
    // still gets fingerprint-match presentations against antigen's flagships.
    let inject_catalog = match bundled {
        BundledCatalog::None => false,
        BundledCatalog::AutoDetect => report.antigens.is_empty(),
        BundledCatalog::Always => true,
    };
    let catalog_fingerprints: Vec<(String, antigen_fingerprint::Fingerprint)> = if inject_catalog {
        crate::stdlib::catalog::stdlib_catalog()
    } else {
        Vec::new()
    };

    finalize_report_with_catalog(&mut report, &parsed_files, &catalog_fingerprints);

    Ok(report)
}

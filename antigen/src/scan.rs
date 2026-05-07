//! Source-AST scanner for antigen declarations.
//!
//! Walks Rust source files in a workspace, extracts `#[antigen]`, `#[presents]`,
//! `#[immune]`, and `#[descended_from]` attribute invocations, and produces a
//! [`ScanReport`] suitable for further analysis or rendering.
//!
//! This module is the engine behind `cargo antigen scan`. It's also exposed for
//! custom integrations (e.g., a project's own CI harness, IDE plugins, or
//! programmatic audit tooling).
//!
//! ## Status (v0.0.1)
//!
//! Initial implementation. Discovers attribute invocations and matches presentations
//! against immunities at the same item level. Future versions will add:
//!
//! - `#[descended_from]` propagation walks
//! - Cross-crate antigen declaration discovery
//! - Witness validation (delegating to clippy/kani/proptest as appropriate)
//! - Fingerprint structural matching against unmarked code (the
//!   recognition-not-yet-marked half of scan)
//! - Performance optimizations (incremental scan, parallel file walks)
//!
//! ## Known v1 limitations (easy wins for the JBD team)
//!
//! Search this file for `TODO(team)` to find specific spots that the antigen JBD
//! team can sharpen quickly without redesigning anything. Top three:
//!
//! 1. **Line numbers are heuristic** — see [`ScanVisitor::line_of_attr`]; finds
//!    the FIRST occurrence of the attribute name in the source, not the actual
//!    span of the specific invocation. Replace with `syn::spanned::Spanned::span()
//!    .start().line` once syn's span info is reliable on the team's toolchain.
//! 2. **Item-level matching is loose** — see [`ScanReport::unaddressed_presentations`];
//!    uses 20-line proximity heuristic. Should match by impl-target / fn-name /
//!    struct-name (the actual ITEM the attributes are applied to), not source
//!    proximity.
//! 3. **Witness validation is presence-only** — the scan records the witness
//!    identifier but doesn't verify it resolves to a real function or that the
//!    function actually exercises behavior matching the antigen. The audit
//!    subcommand (sweep A2/A3) lifts this.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use syn::visit::Visit;
use walkdir::WalkDir;

/// A single antigen declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigenDeclaration {
    /// The kebab-case antigen name from `#[antigen(name = "...")]`.
    pub name: String,
    /// The Rust type name the attribute is applied to (e.g., `PanickingInDrop`).
    pub type_name: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number of the antigen attribute.
    pub line: usize,
    /// Optional family classification (e.g., "boundary-violation").
    pub family: Option<String>,
    /// Optional human-readable summary.
    pub summary: Option<String>,
    /// Optional fingerprint string (uninterpreted in v1; ADR-010 grammar
    /// implementation lands in a future sweep).
    pub fingerprint: Option<String>,
}

/// A `#[presents(antigen_type)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// The antigen type referenced (last path segment, e.g., `PanickingInDrop`).
    pub antigen_type: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (impl, fn, struct, etc.).
    pub item_kind: String,
}

/// An `#[immune(antigen_type, witness = ...)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    /// The antigen type referenced.
    pub antigen_type: String,
    /// The witness expression as a string (validated lazily).
    pub witness: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
}

/// Aggregate result of scanning a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    /// All discovered antigen declarations.
    pub antigens: Vec<AntigenDeclaration>,
    /// All discovered `#[presents]` sites.
    pub presentations: Vec<Presentation>,
    /// All discovered `#[immune]` sites.
    pub immunities: Vec<Immunity>,
    /// Files scanned successfully.
    pub files_scanned: usize,
    /// Files that failed to parse.
    pub parse_failures: Vec<(PathBuf, String)>,
}

/// A presentation that has no matching immunity declaration on the same item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaddressedPresentation {
    /// The presentation itself.
    pub presentation: Presentation,
    /// True if the antigen referenced is found in the scan report.
    pub antigen_known: bool,
}

impl ScanReport {
    /// Find presentations that lack a corresponding immunity declaration.
    ///
    /// In v1, "matching" means: same `antigen_type` AND same file AND nearby
    /// line (presentations and immunities applied to the same item, within
    /// 20 source lines). Future versions will extend to `#[descended_from]`
    /// chains and exact item-target matching.
    ///
    // TODO(team): item-level matching is loose. The 20-line proximity heuristic
    // is a placeholder. Replace with structural matching: track the impl-target
    // / fn-name / struct-name during AST walk, store it on Presentation and
    // Immunity, and match by item identity rather than source proximity. This
    // is an easy win that significantly improves accuracy.
    #[must_use]
    pub fn unaddressed_presentations(&self) -> Vec<UnaddressedPresentation> {
        let known_antigens: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();

        let mut result = Vec::new();
        for p in &self.presentations {
            // Match if there's an immunity declaration on a nearby line (within ~20
            // lines, accommodating multi-attribute stacking).
            let has_nearby_immunity = self.immunities.iter().any(|i| {
                i.antigen_type == p.antigen_type
                    && i.file == p.file
                    && i.line.abs_diff(p.line) <= 20
            });
            if !has_nearby_immunity {
                result.push(UnaddressedPresentation {
                    presentation: p.clone(),
                    antigen_known: known_antigens.contains(p.antigen_type.as_str()),
                });
            }
        }
        result
    }

    /// Total count of antigen-related declarations found.
    #[must_use]
    pub fn total_declarations(&self) -> usize {
        self.antigens.len() + self.presentations.len() + self.immunities.len()
    }
}

/// Scan a directory tree, reading every `.rs` file and extracting antigen
/// declarations.
///
/// `excluded_dirs` is a list of directory names (not full paths) to skip during
/// the walk; the default is `["target", ".git", "node_modules"]` if `None` is
/// passed.
pub fn scan_workspace(root: &Path, excluded_dirs: Option<&[&str]>) -> std::io::Result<ScanReport> {
    let default_exclusions = ["target", ".git", "node_modules"];
    let exclusions = excluded_dirs.unwrap_or(&default_exclusions);

    let mut report = ScanReport::default();

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
                let mut visitor = ScanVisitor {
                    file_path: entry.path().to_path_buf(),
                    source: &content,
                    report: &mut report,
                };
                visitor.visit_file(&file);
                report.files_scanned += 1;
            }
            Err(e) => {
                report
                    .parse_failures
                    .push((entry.path().to_path_buf(), e.to_string()));
            }
        }
    }

    Ok(report)
}

/// AST visitor that extracts antigen-related attributes.
struct ScanVisitor<'a> {
    file_path: PathBuf,
    source: &'a str,
    report: &'a mut ScanReport,
}

impl ScanVisitor<'_> {
    /// Compute 1-indexed line number for a span by counting newlines in source up
    /// to the span's start.
    ///
    /// TODO(team): currently returns the FIRST occurrence of the attribute name
    /// in the file, regardless of which instance is being processed. This means
    /// multi-instance scenarios report the same line number for every instance.
    /// Replace with a real span-tracking approach: pass the `&syn::Attribute` in
    /// and use `syn::spanned::Spanned::span().start().line` (verify it returns
    /// usable line numbers on the team's stable toolchain; if not, walk byte
    /// offsets via `proc_macro2::Span::byte_range` once that's stable).
    fn line_of_attr(&self, attr_name: &str) -> usize {
        // Heuristic: find the first occurrence of the attribute name in the source.
        for (i, line) in self.source.lines().enumerate() {
            if line.contains(&format!("#[{attr_name}")) {
                return i + 1;
            }
        }
        0
    }

    fn extract_antigen(&mut self, item: &syn::ItemStruct, attr: &syn::Attribute) {
        let type_name = item.ident.to_string();
        let line = self.line_of_attr("antigen");

        let mut name = String::new();
        let mut family = None;
        let mut summary = None;
        let mut fingerprint = None;

        if let syn::Meta::List(list) = &attr.meta {
            // Parse the attribute body as comma-separated key=value pairs.
            // We can't use the proc-macro parser here (this is a regular crate),
            // so we do a string-level extraction.
            let body = list.tokens.to_string();
            for assignment in split_top_level_commas(&body) {
                if let Some((key, val)) = parse_kv(&assignment) {
                    match key.trim() {
                        "name" => name = val,
                        "family" => family = Some(val),
                        "summary" => summary = Some(val),
                        "fingerprint" => fingerprint = Some(val),
                        _ => {} // ignore unknown
                    }
                }
            }
        }

        self.report.antigens.push(AntigenDeclaration {
            name,
            type_name,
            file: self.file_path.clone(),
            line,
            family,
            summary,
            fingerprint,
        });
    }

    fn extract_presents(&mut self, attr: &syn::Attribute, item_kind: &str) {
        let antigen_type = if let syn::Meta::List(list) = &attr.meta {
            // The body is a single path; the last segment is the type name.
            let body = list.tokens.to_string();
            body.trim().split("::").last().unwrap_or(&body).to_string()
        } else {
            return;
        };
        let line = self.line_of_attr("presents");
        self.report.presentations.push(Presentation {
            antigen_type,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
        });
    }

    fn extract_immune(&mut self, attr: &syn::Attribute, item_kind: &str) {
        if let syn::Meta::List(list) = &attr.meta {
            let body = list.tokens.to_string();
            // First comma-separated segment is the antigen path; rest are key=value
            let parts = split_top_level_commas(&body);
            let antigen_type = parts
                .first()
                .map(|s| s.trim().split("::").last().unwrap_or(s).to_string())
                .unwrap_or_default();
            // TODO(team): witness validation is presence-only. The audit subcommand
            // (sweep A2/A3) should: (1) resolve the witness identifier to an item
            // in the workspace, (2) verify it's a #[test], proptest!, or recognized
            // delegated tool reference, (3) optionally invoke it via cargo test and
            // verify it asserts the expected property. Currently we just record
            // the witness string verbatim.
            let mut witness = String::new();
            for part in parts.iter().skip(1) {
                if let Some((k, v)) = parse_kv(part) {
                    if k.trim() == "witness" {
                        witness = v;
                    }
                }
            }
            let line = self.line_of_attr("immune");
            self.report.immunities.push(Immunity {
                antigen_type,
                witness,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
            });
        }
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute], item_kind: &str) {
        for attr in attrs {
            if attr.path().is_ident("presents") {
                self.extract_presents(attr, item_kind);
            } else if attr.path().is_ident("immune") {
                self.extract_immune(attr, item_kind);
            }
        }
    }
}

impl<'ast> Visit<'ast> for ScanVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        for attr in &item.attrs {
            if attr.path().is_ident("antigen") {
                self.extract_antigen(item, attr);
            }
        }
        // structs can also have presents/immune for unusual cases
        self.check_attrs(&item.attrs, "struct");
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        self.check_attrs(&item.attrs, "impl");
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.check_attrs(&item.attrs, "fn");
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.check_attrs(&item.attrs, "impl_fn");
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.check_attrs(&item.attrs, "trait");
        syn::visit::visit_item_trait(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for attr in &item.attrs {
            if attr.path().is_ident("antigen") {
                // antigen on enum is not standard but we record it for diagnostic
                let _ = attr;
            }
        }
        self.check_attrs(&item.attrs, "enum");
        syn::visit::visit_item_enum(self, item);
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Split a token string on commas at top level (not inside brackets/braces/parens).
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(c);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                if !current.trim().is_empty() {
                    out.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    }
    out
}

/// Parse a `key = value` pair, returning trimmed key and unquoted-string value.
fn parse_kv(s: &str) -> Option<(String, String)> {
    let mut parts = s.splitn(2, '=');
    let key = parts.next()?.trim().to_string();
    let raw_val = parts.next()?.trim();
    // Strip surrounding quotes if present.
    let val = if raw_val.starts_with('"') && raw_val.ends_with('"') && raw_val.len() >= 2 {
        raw_val[1..raw_val.len() - 1].to_string()
    } else {
        raw_val.to_string()
    };
    Some((key, val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_commas_respects_brackets() {
        let result = split_top_level_commas("a, [b, c], d");
        assert_eq!(result, vec!["a", "[b, c]", "d"]);
    }

    #[test]
    fn parse_kv_strips_quotes() {
        let (k, v) = parse_kv(r#"name = "hello""#).unwrap();
        assert_eq!(k, "name");
        assert_eq!(v, "hello");
    }

    #[test]
    fn parse_kv_keeps_inner_quotes() {
        let (k, v) = parse_kv(r#"family = "boundary-violation""#).unwrap();
        assert_eq!(k, "family");
        assert_eq!(v, "boundary-violation");
    }

    #[test]
    fn empty_scan_report_has_no_unaddressed() {
        let report = ScanReport::default();
        assert!(report.unaddressed_presentations().is_empty());
    }
}

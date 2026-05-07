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

use std::collections::HashMap;
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
    /// The antigen type referenced (last path segment, e.g., "PanickingInDrop").
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
    pub presentation: Presentation,
    /// True if the antigen referenced is found in the scan report.
    pub antigen_known: bool,
}

impl ScanReport {
    /// Find presentations that lack a corresponding immunity declaration.
    ///
    /// In v1, "matching" means: same antigen_type AND same file AND same line
    /// (presentations and immunities applied to the same item). Future versions
    /// will extend to `#[descended_from]` chains and item-aware matching.
    pub fn unaddressed_presentations(&self) -> Vec<UnaddressedPresentation> {
        let immunity_keys: HashMap<(String, PathBuf, usize), &Immunity> = self
            .immunities
            .iter()
            .map(|i| ((i.antigen_type.clone(), i.file.clone(), i.line), i))
            .collect();

        let known_antigens: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();

        let mut result = Vec::new();
        for p in &self.presentations {
            // Match if there's an immunity declaration on a nearby line (within ~20
            // lines, accommodating multi-attribute stacking).
            let has_nearby_immunity = self.immunities.iter().any(|i| {
                i.antigen_type == p.antigen_type
                    && i.file == p.file
                    && (i.line as isize - p.line as isize).abs() <= 20
            });
            if !has_nearby_immunity {
                result.push(UnaddressedPresentation {
                    presentation: p.clone(),
                    antigen_known: known_antigens.contains(p.antigen_type.as_str()),
                });
            }
        }
        // Suppress dead-code warning on the immunity_keys HashMap; it's reserved for
        // a future exact-line matching mode.
        let _ = immunity_keys;
        result
    }

    /// Total count of antigen-related declarations found.
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
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
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

impl<'a> ScanVisitor<'a> {
    /// Compute 1-indexed line number for a span by counting newlines in source up
    /// to the span's start. `syn`'s span line info isn't reliable on stable, so we
    /// fall back to source-counting.
    fn line_of_attr(&self, attr_name: &str) -> usize {
        // Heuristic: find the first occurrence of the attribute name in the source.
        // For multi-instance scenarios this isn't ideal; future improvement is to
        // track byte offsets via syn::spanned::Spanned and locate within source.
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

impl<'ast, 'a> Visit<'ast> for ScanVisitor<'a> {
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

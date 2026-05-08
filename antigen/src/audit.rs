//! Witness validation and immunity audit.
//!
//! The audit module operates a layer above [`crate::scan`]: where scan finds
//! antigen-related declarations as syntactic facts, audit reasons about whether
//! the immunity claims are actually backed by working witnesses.
//!
//! This is the "trust-boundary check" required by ADR-005 (sub-clause F at every
//! trust boundary). A declaration of `#[immune(X, witness = Y)]` is meaningful
//! only if `Y` resolves to a real function, test, lint reference, or proof that
//! demonstrates immunity. A marker without a working witness is not a claim.
//!
//! ## What audit checks (v0.0.1)
//!
//! - Witness identifiers resolve to a function/test in the workspace
//! - Witness functions have a recognized testing attribute (`#[test]`, recognizable
//!   `proptest!` invocation, or known external delegations like `clippy::lint_name`)
//!
//! ## What audit doesn't check (yet)
//!
//! - **Witness execution**: doesn't actually run the test/proptest. The team
//!   should add `cargo test` integration in sweep A3+.
//! - **Witness semantics**: doesn't verify the witness asserts the antigen's
//!   specific failure pattern. That requires fingerprint-aware reasoning.
//! - **External tool delegation**: clippy/kani/prusti adapters are stubbed with
//!   "external; manual validation required" status. Sweep A3+ adds adapters.
//! - **Cross-crate witnesses**: a witness that lives in a dependency isn't
//!   followed. v0.0.1 audit is workspace-local only.
//!
//! Search this file for `TODO(team)` for specific easy-win extension points.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::scan::{Immunity, ScanReport};

/// The status of a single witness validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WitnessStatus {
    /// Witness identifier resolves to a function with a recognized testing
    /// attribute in the workspace.
    ///
    /// **Important**: "resolved" means the identifier was found — it does NOT
    /// mean the witness was executed or that it asserts immunity to this specific
    /// failure class. Semantic verification requires fingerprint-aware reasoning
    /// (ADR-010, planned for Sweep A3+).
    Resolved {
        /// Where the witness function was found.
        location: PathBuf,
        /// What kind of witness was detected.
        witness_kind: WitnessKind,
    },
    /// Witness identifier appears to reference an external tool (clippy lint,
    /// kani proof, prusti annotation, etc.); deferred to that tool's validator.
    External {
        /// Best-effort guess at the external tool.
        tool_hint: String,
    },
    /// Witness identifier could not be resolved in the workspace.
    NotFound {
        /// Reason the witness wasn't found (e.g., "no matching function in any
        /// .rs file under the scan root").
        reason: String,
    },
    /// The immunity declaration didn't include a witness identifier at all.
    Missing,
}

/// What kind of witness mechanism was detected.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WitnessKind {
    /// A function with a `#[test]` attribute.
    Test,
    /// A `proptest!` macro invocation.
    Proptest,
    /// A regular function (no testing attribute detected; might be a phantom-type
    /// proof or non-test witness).
    Function,
}

/// Result of auditing a single immunity declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmunityAudit {
    /// The original immunity declaration.
    pub immunity: Immunity,
    /// What we determined about its witness.
    pub witness_status: WitnessStatus,
}

impl ImmunityAudit {
    /// True if the audit considers the immunity claim well-formed.
    ///
    /// `Resolved` and `External` are both well-formed (we trust external tools
    /// to enforce the claim). `NotFound` and `Missing` are NOT well-formed.
    #[must_use]
    pub const fn is_well_formed(&self) -> bool {
        matches!(
            self.witness_status,
            WitnessStatus::Resolved { .. } | WitnessStatus::External { .. }
        )
    }
}

/// Aggregate audit report for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditReport {
    /// Per-immunity audit results.
    pub audits: Vec<ImmunityAudit>,
    /// Number of immunities whose witness resolved cleanly.
    pub resolved_count: usize,
    /// Number of immunities whose witness defers to an external tool.
    pub external_count: usize,
    /// Number of immunities whose witness was not found.
    pub broken_count: usize,
    /// Number of immunities with no witness identifier at all.
    pub missing_count: usize,
}

impl AuditReport {
    /// True if all witnesses validated (no broken or missing).
    #[must_use]
    pub const fn all_valid(&self) -> bool {
        self.broken_count == 0 && self.missing_count == 0
    }

    /// Returns audits whose witness status indicates a problem.
    #[must_use]
    pub fn problematic_audits(&self) -> Vec<&ImmunityAudit> {
        self.audits.iter().filter(|a| !a.is_well_formed()).collect()
    }
}

/// Run audit against a [`ScanReport`].
///
/// For each immunity declaration, attempts to validate the witness identifier
/// by walking the workspace looking for the function it names.
///
/// `workspace_root` is used to look for witness functions; passing the same
/// path used for [`crate::scan::scan_workspace`] is typical.
///
/// Files that fail to parse during the function-index walk are silently
/// skipped (matching `scan_workspace`'s behavior); this function does not
/// itself surface IO errors to the caller.
#[must_use]
pub fn audit(report: &ScanReport, workspace_root: &Path) -> AuditReport {
    let workspace_functions = collect_function_index(workspace_root);

    let mut audits = Vec::new();
    for immunity in &report.immunities {
        let status = validate_witness(&immunity.witness, &workspace_functions);
        audits.push(ImmunityAudit {
            immunity: immunity.clone(),
            witness_status: status,
        });
    }

    let mut audit_report = AuditReport {
        audits,
        ..AuditReport::default()
    };
    for a in &audit_report.audits {
        match &a.witness_status {
            WitnessStatus::Resolved { .. } => audit_report.resolved_count += 1,
            WitnessStatus::External { .. } => audit_report.external_count += 1,
            WitnessStatus::NotFound { .. } => audit_report.broken_count += 1,
            WitnessStatus::Missing => audit_report.missing_count += 1,
        }
    }

    audit_report
}

/// Index of function name → (file path, kind) for the workspace.
///
/// TODO(team): this is a flat name index; doesn't handle:
/// - Module-qualified paths (`crate::foo::bar` would need to find the function
///   `bar` in module `foo`)
/// - Function ambiguity (two functions with the same name in different modules)
/// - Functions inside `impl` blocks (these are method names, not free functions;
///   currently we record both but matching is name-only)
///
/// Sweep A3 should: parse witness as a full path, resolve against the workspace's
/// module graph, and disambiguate methods vs free functions.
type FunctionIndex = std::collections::HashMap<String, (PathBuf, WitnessKind)>;

fn collect_function_index(root: &Path) -> FunctionIndex {
    use syn::visit::Visit;
    use walkdir::WalkDir;

    let exclusions = ["target", ".git", "node_modules"];
    let mut index = FunctionIndex::new();

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

        if let Ok(file) = syn::parse_file(&content) {
            let mut visitor = FunctionIndexVisitor {
                file_path: entry.path().to_path_buf(),
                source: &content,
                index: &mut index,
            };
            visitor.visit_file(&file);
        }
    }

    index
}

struct FunctionIndexVisitor<'a> {
    file_path: PathBuf,
    /// Source text of the file being walked. Carried for symmetry with
    /// `scan::ScanVisitor` and for future span-anchored diagnostics; the
    /// pre-W5 textual `source.contains("proptest!")` sentinel was removed
    /// when `visit_macro` took over proptest classification.
    #[allow(dead_code, reason = "reserved for span-anchored diagnostic work \
        that mirrors scan::ScanVisitor::source")]
    source: &'a str,
    index: &'a mut FunctionIndex,
}

impl FunctionIndexVisitor<'_> {
    /// Classify a function by its own attributes.
    ///
    /// W5 (sweep A2): the prior heuristic — `self.source.contains("proptest!")`
    /// — over-classified every function in any file mentioning the string
    /// `proptest!` (including doc comments) as `WitnessKind::Proptest`.
    /// Replaced by structural detection: `visit_macro` registers
    /// proptest-internal function names with `WitnessKind::Proptest` directly,
    /// so by the time `visit_item_fn` runs `detect_kind` the function is
    /// already correctly tagged for proptest cases. The remaining job here
    /// is to pick `Test` when `#[test]` appears, and `Function` otherwise.
    fn detect_kind(attrs: &[syn::Attribute]) -> WitnessKind {
        for attr in attrs {
            if attr.path().is_ident("test") {
                return WitnessKind::Test;
            }
        }
        WitnessKind::Function
    }
}

/// Extract top-level `fn IDENT` names from a `proptest! { ... }` macro body.
///
/// `proptest!` is a function-like macro that takes a sequence of test-shaped
/// declarations:
///
/// ```ignore
/// proptest! {
///     #[test]
///     fn name(args in strategy) { body }
///     ...
/// }
/// ```
///
/// The body's tokens contain `fn IDENT` at the top level for each test;
/// nested function definitions live inside `Group` tokens (the body block of
/// each fn) which a top-level token-iterator does not descend into. So a
/// linear walk that yields `name` whenever it sees `fn` followed by an
/// identifier captures exactly the proptest test names — no more, no less.
///
/// Why not parse with `syn` directly? `proptest!`'s grammar (`fn name(args
/// in strategy)`) is not a valid Rust function signature: the `in` keyword
/// inside the parameter list is custom syntax. `syn::ItemFn::parse` rejects
/// the body. The token walk below is grammar-aware enough for our purpose
/// (extracting names) without committing to parsing the strategy expressions.
fn extract_proptest_fn_names(tokens: &proc_macro2::TokenStream) -> Vec<String> {
    use proc_macro2::TokenTree;
    let mut names = Vec::new();
    let mut iter = tokens.clone().into_iter();
    while let Some(tt) = iter.next() {
        if let TokenTree::Ident(i) = &tt {
            if i == "fn" {
                if let Some(TokenTree::Ident(name)) = iter.next() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

/// Whether a macro path's last segment is `name`. Mirrors the
/// `attr_is`-style test in `scan.rs`: matches both `#[proptest!(...)]`-style
/// bare names and `proptest::proptest!(...)` path-qualified forms.
fn macro_path_last_is(path: &syn::Path, name: &str) -> bool {
    path.segments
        .last()
        .is_some_and(|s| s.ident == name)
}

impl<'ast> syn::visit::Visit<'ast> for FunctionIndexVisitor<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        // Only insert if not already present — preserves any prior tagging
        // (e.g., `WitnessKind::Proptest` from a `visit_macro` pass that
        // walked an enclosing `proptest! { ... }` block before the visitor
        // descended to the inner function items, in case the visitor order
        // ever changes). For W5 the proptest macro is a leaf so this is
        // defence-in-depth.
        self.index
            .entry(name)
            .or_insert_with(|| (self.file_path.clone(), kind));
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        // Only insert if not already present; free functions take precedence over
        // method names in the flat index.
        self.index
            .entry(name)
            .or_insert_with(|| (self.file_path.clone(), kind));
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        // W5: structural proptest! detection. When the macro path's last
        // segment is `proptest`, walk its tokens for `fn IDENT` patterns
        // and register each name with `WitnessKind::Proptest`. Inserting
        // BEFORE the visitor descends into the macro tokens (which it
        // doesn't actually do for unparsed-token macros, but we insert
        // ahead of any `visit_item_fn` that might happen for adjacent code
        // anyway) means a same-name function inside a proptest! block
        // wins the entry over a same-name free function elsewhere only
        // when it appears first in walk order. Acceptable since the
        // typical case is one or the other, not both.
        if macro_path_last_is(&mac.path, "proptest") {
            for name in extract_proptest_fn_names(&mac.tokens) {
                self.index
                    .entry(name)
                    .or_insert_with(|| (self.file_path.clone(), WitnessKind::Proptest));
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Determine the witness status for a single witness identifier string.
fn validate_witness(witness: &str, index: &FunctionIndex) -> WitnessStatus {
    let trimmed = witness.trim();
    if trimmed.is_empty() {
        return WitnessStatus::Missing;
    }

    // Detect external-tool delegations.
    if let Some(tool) = detect_external_tool(trimmed) {
        return WitnessStatus::External {
            tool_hint: tool.to_string(),
        };
    }

    // Resolve as a workspace-local function. The witness might be a path
    // (`module::function`); take the last segment as the function name.
    let function_name = trimmed
        .rsplit("::")
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches("()")
        .trim();

    if let Some((location, kind)) = index.get(function_name) {
        WitnessStatus::Resolved {
            location: location.clone(),
            witness_kind: kind.clone(),
        }
    } else {
        WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        }
    }
}

/// Detect whether the witness references an external tool we recognize.
fn detect_external_tool(witness: &str) -> Option<&'static str> {
    let lower = witness.to_ascii_lowercase();
    if lower.starts_with("clippy::") || lower.contains("clippy_") {
        Some("clippy")
    } else if lower.starts_with("kani::") || lower.contains("kani_proof") {
        Some("kani")
    } else if lower.starts_with("prusti::") {
        Some("prusti")
    } else if lower.starts_with("creusot::") {
        Some("creusot")
    } else if lower.starts_with("verus::") {
        Some("verus")
    } else if lower.starts_with("mutants::") {
        Some("cargo-mutants")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_clippy_external_tool() {
        assert_eq!(
            detect_external_tool("clippy::no_panic_in_drop"),
            Some("clippy")
        );
    }

    #[test]
    fn detect_kani_external_tool() {
        assert_eq!(
            detect_external_tool("kani::proof_drop_safety"),
            Some("kani")
        );
    }

    #[test]
    fn detect_no_tool_for_local_function() {
        assert_eq!(detect_external_tool("safe_type_drop_no_panic_test"), None);
    }

    #[test]
    fn validate_witness_strips_path_prefix() {
        let mut idx = FunctionIndex::new();
        idx.insert(
            "my_test".to_string(),
            (PathBuf::from("src/lib.rs"), WitnessKind::Test),
        );

        let status = validate_witness("module::path::my_test", &idx);
        assert!(matches!(status, WitnessStatus::Resolved { .. }));
    }

    #[test]
    fn validate_witness_reports_missing_when_empty() {
        let idx = FunctionIndex::new();
        let status = validate_witness("", &idx);
        assert_eq!(status, WitnessStatus::Missing);
    }

    #[test]
    fn validate_witness_reports_not_found_for_unknown() {
        let idx = FunctionIndex::new();
        let status = validate_witness("nonexistent_test", &idx);
        assert!(matches!(status, WitnessStatus::NotFound { .. }));
    }

    // ========================================================================
    // W5 — structural proptest! witness detection.
    //
    // Pre-W5, `detect_kind` did `self.source.contains("proptest!")` as a
    // sentinel — if the source string contained that text anywhere, every
    // function in the file was tagged `WitnessKind::Proptest`. Doc comments
    // mentioning the macro for explanatory purposes triggered the same
    // over-classification.
    //
    // W5 lifts this to structural detection via `visit_macro` + token-walking
    // the macro body for `fn IDENT` patterns. These tests are the contract
    // pinning the W5 behavior without needing a full filesystem fixture.
    // ========================================================================

    /// Run the function-index walk against an in-memory source string.
    /// Mirrors what `collect_function_index` does per-file but without
    /// touching disk — gives the W5 unit tests a tight feedback loop.
    fn index_from_str(source: &str) -> FunctionIndex {
        use syn::visit::Visit;
        let file = syn::parse_file(source).expect("source must parse");
        let mut index = FunctionIndex::new();
        let mut visitor = FunctionIndexVisitor {
            file_path: PathBuf::from("<test>.rs"),
            source,
            index: &mut index,
        };
        visitor.visit_file(&file);
        index
    }

    #[test]
    fn w5_proptest_inner_fns_are_classified_proptest() {
        let src = r"
            proptest! {
                #[test]
                fn first_proptest(x in 0u32..100) {
                    assert!(x < 100);
                }

                #[test]
                fn second_proptest(x in 0u32..100, y in 0u32..100) {
                    assert!(x + y < 200);
                }
            }
        ";
        let idx = index_from_str(src);
        let (_, k1) = idx.get("first_proptest").expect("first_proptest indexed");
        let (_, k2) = idx.get("second_proptest").expect("second_proptest indexed");
        assert_eq!(*k1, WitnessKind::Proptest);
        assert_eq!(*k2, WitnessKind::Proptest);
    }

    #[test]
    fn w5_proptest_path_qualified_macro_is_recognized() {
        // The fixture canonical form is `proptest::proptest!`, matching how
        // the `proptest` crate is typically imported. The W5 helper
        // `macro_path_last_is` checks the LAST segment, so any path ending
        // in `proptest` matches.
        let src = r"
            proptest::proptest! {
                #[test]
                fn qualified_form_proptest(x in 0u32..100) {
                    assert!(x < 100);
                }
            }
        ";
        let idx = index_from_str(src);
        let (_, k) = idx
            .get("qualified_form_proptest")
            .expect("qualified_form_proptest indexed");
        assert_eq!(*k, WitnessKind::Proptest);
    }

    #[test]
    fn w5_test_function_outside_proptest_is_classified_test() {
        // A regular `#[test]` outside any proptest! block must remain
        // `WitnessKind::Test`. The pre-W5 sentinel would have over-classified
        // this as Proptest if the file contained the string `proptest!`
        // anywhere; this test exercises the negative case directly.
        let src = r"
            // Doc-style comment mentioning proptest! for explanation purposes.
            // Pre-W5 this string in the source was sufficient to flag every
            // function in the file as Proptest. W5 must not regress to that.
            #[test]
            fn plain_test() {
                assert_eq!(2 + 2, 4);
            }

            proptest! {
                #[test]
                fn proptest_one(x in 0u32..10) {
                    assert!(x < 10);
                }
            }
        ";
        let idx = index_from_str(src);
        let (_, k_plain) = idx.get("plain_test").expect("plain_test indexed");
        assert_eq!(
            *k_plain,
            WitnessKind::Test,
            "plain_test outside proptest! must be Test, not Proptest, even when \
             the same file contains a proptest! invocation",
        );
        let (_, k_prop) = idx.get("proptest_one").expect("proptest_one indexed");
        assert_eq!(*k_prop, WitnessKind::Proptest);
    }

    #[test]
    fn w5_doc_comment_mentioning_proptest_does_not_over_classify() {
        // The exact regression the pre-W5 textual sentinel had: a doc
        // comment containing the literal string `proptest!` would tag
        // every function in the file as Proptest. W5's structural detection
        // only fires on actual macro invocations, so this `#[test]` stays Test.
        let src = r"
            /// This function has nothing to do with proptest! — the macro
            /// is named here only for documentation.
            #[test]
            fn doc_comment_only_test() {
                assert!(true);
            }
        ";
        let idx = index_from_str(src);
        let (_, k) = idx
            .get("doc_comment_only_test")
            .expect("doc_comment_only_test indexed");
        assert_eq!(*k, WitnessKind::Test, "doc-comment mention must not trigger Proptest");
    }

    #[test]
    fn w5_plain_function_is_classified_function() {
        let src = r"
            fn no_attribute_function() {}
        ";
        let idx = index_from_str(src);
        let (_, k) = idx
            .get("no_attribute_function")
            .expect("no_attribute_function indexed");
        assert_eq!(*k, WitnessKind::Function);
    }

    #[test]
    fn w5_extract_proptest_fn_names_skips_nested() {
        // Nested function definitions inside a fn body live in a Group token;
        // the top-level token walk should not descend into them. This locks
        // the "nested fn doesn't get registered as a proptest test" invariant.
        use proc_macro2::TokenStream;
        let tokens: TokenStream = r"
            #[test]
            fn outer(x in 0u32..10) {
                fn nested_helper() {}
                assert!(x < 10);
            }
        "
        .parse()
        .unwrap();
        let names = extract_proptest_fn_names(&tokens);
        assert_eq!(names, vec!["outer".to_string()]);
    }

    #[test]
    fn w5_macro_path_last_is_handles_qualified_paths() {
        let bare: syn::Path = syn::parse_str("proptest").unwrap();
        let qualified: syn::Path = syn::parse_str("proptest::proptest").unwrap();
        let unrelated: syn::Path = syn::parse_str("other_crate::other_macro").unwrap();
        assert!(macro_path_last_is(&bare, "proptest"));
        assert!(macro_path_last_is(&qualified, "proptest"));
        assert!(!macro_path_last_is(&unrelated, "proptest"));
    }
}

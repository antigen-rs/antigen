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
//!   followed. v0.0.1 audit is workspace-local only — A3 sweep extends this
//!   via cross-crate source walking (per scope-lock).

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
    /// failure class. Semantic verification (does the witness actually assert
    /// the antigen's failure mode?) is behavioral-tier work tracked as
    /// the `BehavioralAlignment` witness tier; planned for A4-A5 sweeps
    /// (ADR-001 Amendment 1 Change 4 + ADR-013 phantom-type witness pluralism).
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
    /// Witness identifier resolves to multiple functions in the workspace
    /// (ATK-A2-005). The caller must qualify the path or rename one
    /// candidate. Audit reports `WitnessTier::None` because no single
    /// resolution was confirmed.
    Ambiguous {
        /// Locations of all candidate functions sharing this name.
        candidates: Vec<PathBuf>,
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
    /// A function with a `#[test]` attribute (and not `#[ignore]`).
    Test,
    /// A function with `#[test]` AND `#[ignore]` — `cargo test` skips it
    /// by default. Audit treats this as Reachability tier, not Execution,
    /// per ADR-005 Amendment 3 (ATK-A2-012).
    IgnoredTest,
    /// A `proptest!` macro invocation.
    Proptest,
    /// A regular function (no testing attribute detected; might be a phantom-type
    /// proof or non-test witness).
    Function,
    /// A phantom-type witness: a path like `Path::<TypeParams>::constructor`
    /// where construction itself is the proof. Recognized structurally per
    /// ADR-013; the audit reports a hint to verify the constructor is sealed.
    PhantomType {
        /// The base path (e.g., `PolarityProof`).
        proof_type: String,
        /// Type parameters if any (e.g., `["FrameTranslation"]`).
        type_params: Vec<String>,
        /// Constructor function name if present (e.g., `verified` in
        /// `PolarityProof::<FrameTranslation>::verified()`).
        constructor: Option<String>,
    },
}

/// The strength of evidence a witness provides for an immunity claim.
///
/// Per ADR-005 Amendment 3: this enum reports work the audit *actually
/// performed* at the validation point — never potential-maximum evidence.
/// Per-case disambiguation lives on the parallel [`AuditHint`] axis.
///
/// Ordered: higher ordinal = stronger evidence. Stable discriminants
/// reserve room for `BehavioralAlignment` to insert at 3 in a future ADR.
///
/// # CI gating
///
/// `cargo antigen audit --min-tier execution` fails if any immunity claim
/// is below Execution tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum WitnessTier {
    /// No witness or unresolved witness. Immunity asserted without evidence.
    None = 0,
    /// Witness identifier resolves but no execution-level verification
    /// happened. Evidence: "this code path / tool reference exists."
    Reachability = 1,
    /// Witness was executed: a test or proptest function whose run was
    /// confirmed (A3+ feature; not yet emitted by v0.1 audit).
    Execution = 2,
    // BehavioralAlignment = 3, reserved per ADR-005 OQ
    /// Compile-time proof: phantom-type construction whose construction is
    /// the proof, or formal-verification tool with confirmed passing proof
    /// (A3+).
    FormalProof = 4,
}

/// Per-case verification-work disambiguation, parallel to [`WitnessTier`].
/// Per ADR-005 Amendment 3 Mechanics §2.
///
/// Two witnesses can carry the same [`WitnessTier`] but different
/// `AuditHint` — for example, an unrun `#[test]` and an external clippy
/// reference both sit at `Reachability` (zero confirmed assertions about
/// this site) but the disambiguation tells the user how to upgrade.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditHint {
    /// No hint applicable (status is Missing or `NotFound`).
    NoneApplicable,
    /// Identifier resolves to a function; no further check.
    FunctionResolves,
    /// Function has `#[test]`, audit did not invoke `cargo test`.
    TestAttributePresentNotInvoked,
    /// Function has `#[test]` AND `#[ignore]`; `cargo test` would skip it.
    TestAttributePresentIgnoreSkipped,
    /// `proptest!` macro invocation found; harness not invoked.
    ProptestPresentNotInvoked,
    /// External-tool prefix recognized (`clippy::`, `kani::`, ...);
    /// tool not invoked.
    ExternalToolPrefixRecognized,
    /// External tool actually invoked; deferred to A3+.
    ExternalToolInvoked,
    /// Phantom-type witness shape recognized; constructor not validated.
    PhantomTypeShapeRecognized,
    /// Phantom-type witness construction validated; deferred to future ADR.
    PhantomTypeConstructionValidated,
    /// Witness name matches more than one function in the workspace
    /// (ATK-A2-005). Caller should qualify the path.
    AmbiguousResolution,
    /// Witness path's module prefix does not exist in the workspace
    /// (ATK-A2-011). The last segment was found but in an unrelated location.
    FabricatedPathPrefix,
    /// Inherited Presentation lacks re-attestation on the descendant site
    /// (state 7 of the 7-state matrix, ADR-018 `§"AuditHint integration"`).
    /// Behavioral re-validation that the ancestor's witness applies to
    /// the descendant is A4-A5 work; reachability-tier audit cannot
    /// perform this check. The descendant should declare its own
    /// `#[immune]` or `#[antigen_tolerance]`.
    InheritedPresentationNotReAttested,
}

/// Result of auditing a single immunity declaration.
///
/// Two structured fields express what the audit found:
/// - [`witness_tier`](Self::witness_tier): Ord-able strength of evidence,
///   what CI gates check (`--min-tier`)
/// - [`audit_hint`](Self::audit_hint): per-case verification-work
///   disambiguation, what humans read in reports
///
/// Both are derived from [`witness_status`](Self::witness_status) at audit
/// time per ADR-005 Amendment 3 §Mechanics §2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmunityAudit {
    /// The original immunity declaration.
    pub immunity: Immunity,
    /// What we determined about its witness.
    pub witness_status: WitnessStatus,
    /// Strength of evidence the witness provides, derived from
    /// `witness_status` per ADR-005 Amendment 3.
    pub witness_tier: WitnessTier,
    /// Per-case verification-work disambiguation; carries the signal that
    /// the tier ordinal alone cannot.
    pub audit_hint: AuditHint,
}

impl ImmunityAudit {
    /// True if the witness provides any evidence (tier > None).
    #[must_use]
    pub const fn has_witness(&self) -> bool {
        !matches!(self.witness_tier, WitnessTier::None)
    }

    /// True if the witness meets a minimum evidence tier. Used by `--strict`
    /// mode and CI gates.
    #[must_use]
    pub fn meets_tier(&self, minimum: WitnessTier) -> bool {
        self.witness_tier >= minimum
    }

    /// True if the audit considers the immunity claim well-formed.
    ///
    /// Per ADR-005 Amendment 3: well-formed requires execution-tier evidence
    /// or stronger. `Reachability`-tier witnesses (e.g., a `#[test]` that
    /// hasn't been run, or a fabricated `clippy::` lint) are NOT well-formed.
    /// This is the post-W7 honest definition; the pre-W7 `is_well_formed`
    /// returned true for any `Resolved`/`External`, which is the bug
    /// ATK-A2-003/004/005/011/012 named.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.meets_tier(WitnessTier::Execution)
    }
}

impl WitnessTier {
    /// Derive the tier from a [`WitnessStatus`] per the Amendment 3 mapping
    /// table. The audit reports the work it actually performed — never
    /// potential maximum evidence.
    #[must_use]
    pub const fn from_status(status: &WitnessStatus) -> Self {
        match status {
            WitnessStatus::Missing
            | WitnessStatus::NotFound { .. }
            | WitnessStatus::Ambiguous { .. } => Self::None,
            WitnessStatus::External { .. } => Self::Reachability,
            WitnessStatus::Resolved { witness_kind, .. } => match witness_kind {
                // v0.1 audit does not invoke cargo test or proptest harness;
                // witness presence means "this code path exists" — Reachability.
                // Execution tier requires confirmed invocation (A3+ work).
                WitnessKind::Test
                | WitnessKind::IgnoredTest
                | WitnessKind::Proptest
                | WitnessKind::Function => Self::Reachability,
                WitnessKind::PhantomType { .. } => Self::FormalProof,
            },
        }
    }
}

impl AuditHint {
    /// Derive the audit hint from a [`WitnessStatus`] per the Amendment 3
    /// mapping table.
    #[must_use]
    pub const fn from_status(status: &WitnessStatus) -> Self {
        match status {
            WitnessStatus::Missing | WitnessStatus::NotFound { .. } => Self::NoneApplicable,
            WitnessStatus::Ambiguous { .. } => Self::AmbiguousResolution,
            WitnessStatus::External { .. } => Self::ExternalToolPrefixRecognized,
            WitnessStatus::Resolved { witness_kind, .. } => match witness_kind {
                WitnessKind::Test => Self::TestAttributePresentNotInvoked,
                WitnessKind::IgnoredTest => Self::TestAttributePresentIgnoreSkipped,
                WitnessKind::Proptest => Self::ProptestPresentNotInvoked,
                WitnessKind::Function => Self::FunctionResolves,
                WitnessKind::PhantomType { .. } => Self::PhantomTypeShapeRecognized,
            },
        }
    }
}

/// A diagnostic for state 7 of the 7-state interaction matrix:
/// inherited Presentation that lacks immune or tolerance re-attestation
/// on the descendant site. ADR-018 §"Audit diagnostic text".
///
/// Emitted at warn level by default; `--strict` promotes to error.
/// The descendant inherited a presentation from one or more ancestors
/// via `#[descended_from]` propagation; ADR-005 sub-clause F requires
/// the descendant to re-attest the witness rather than silently
/// extending the ancestor's trust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritedUnaddressed {
    /// The inherited presentation that lacks re-attestation.
    pub presentation: crate::scan::Presentation,
    /// The behavioral-tier audit hint per ADR-018 `§"AuditHint integration"`:
    /// `inherited-presentation-not-re-attested`. Behavioral re-validation
    /// (does the ancestor's witness actually apply to descendant?) is
    /// A4-A5 work; reachability-tier audit cannot perform this check.
    pub audit_hint: AuditHint,
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
    /// Number of immunities whose witness name resolves ambiguously
    /// (multiple workspace functions share the name). Per ATK-A2-005.
    pub ambiguous_count: usize,
    /// Number of immunities whose witness was not found.
    pub broken_count: usize,
    /// Number of immunities with no witness identifier at all.
    pub missing_count: usize,
    /// Inherited Presentations on a descendant that have no matching
    /// Immunity or Toleration on the same site (state 7 of the 7-state
    /// interaction matrix, ADR-018). Audit emits warn-level diagnostics
    /// for each; `--strict` promotes to error.
    #[serde(default)]
    pub inherited_unaddressed: Vec<InheritedUnaddressed>,
}

impl AuditReport {
    /// True if all immunity claims meet at least Execution tier
    /// (per `is_well_formed`). Per ADR-005 Amendment 3, a Reachability-tier
    /// witness is NOT a well-formed claim — it has zero confirmed evidence.
    #[must_use]
    pub fn all_valid(&self) -> bool {
        self.audits.iter().all(ImmunityAudit::is_well_formed)
    }

    /// True if all immunity claims meet the given minimum tier. Used by
    /// `cargo antigen audit --min-tier <tier>` for CI gating.
    #[must_use]
    pub fn all_meet_tier(&self, minimum: WitnessTier) -> bool {
        self.audits.iter().all(|a| a.meets_tier(minimum))
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
        let witness_tier = WitnessTier::from_status(&status);
        let audit_hint = AuditHint::from_status(&status);
        audits.push(ImmunityAudit {
            immunity: immunity.clone(),
            witness_status: status,
            witness_tier,
            audit_hint,
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
            WitnessStatus::Ambiguous { .. } => audit_report.ambiguous_count += 1,
            WitnessStatus::NotFound { .. } => audit_report.broken_count += 1,
            WitnessStatus::Missing => audit_report.missing_count += 1,
        }
    }

    // State 7 detection (ADR-018 §"7-state interaction matrix"): an
    // inherited Presentation (inherited_from = Some(_)) without matching
    // Immunity or Toleration on the descendant site is unaddressed.
    // `unaddressed_presentations()` already encodes the "no matching
    // immune/tolerance" check; we filter its output to the inherited
    // subset.
    for u in report.unaddressed_presentations() {
        if u.presentation.inherited_from.is_some() {
            audit_report
                .inherited_unaddressed
                .push(InheritedUnaddressed {
                    presentation: u.presentation,
                    audit_hint: AuditHint::InheritedPresentationNotReAttested,
                });
        }
    }

    audit_report
}

/// One entry in the function index — a single (path, kind) pair for a name.
#[derive(Debug, Clone)]
struct FunctionEntry {
    location: PathBuf,
    kind: WitnessKind,
}

/// Index of function name → all (file path, kind) pairs sharing that name.
///
/// W7 (A2) extends the flat name index to track *all* candidates for a name,
/// so `validate_witness` can detect ambiguity (ATK-A2-005). When more than one
/// function shares a name, the witness resolves to `WitnessStatus::Ambiguous`
/// rather than silently picking whichever was indexed last.
///
/// Cross-cutting limitations remaining for A3+:
/// - Module-qualified paths (`crate::foo::bar` parsing) require module-graph
///   resolution; for v0.1 we detect ambiguity and require the user to qualify
///   the witness (e.g., rename one of the conflicting functions, or use a
///   path that is unique).
/// - Functions inside `impl` blocks (method names, not free functions);
///   currently recorded with the same shape — matching is name-only.
type FunctionIndex = std::collections::HashMap<String, Vec<FunctionEntry>>;

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
    #[allow(
        dead_code,
        reason = "reserved for span-anchored diagnostic work \
        that mirrors scan::ScanVisitor::source"
    )]
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
    /// proptest-internal function names with `WitnessKind::Proptest` directly.
    ///
    /// W7 (sweep A2): distinguish `#[test] #[ignore]` from a running `#[test]`.
    /// Per ADR-005 Amendment 3 and ATK-A2-012, an ignored test is weaker
    /// evidence than a runnable test — `cargo test` skips it by default.
    /// We tag it as `WitnessKind::IgnoredTest` so the audit can emit the
    /// `TestAttributePresentIgnoreSkipped` hint.
    fn detect_kind(attrs: &[syn::Attribute]) -> WitnessKind {
        let has_test = attrs.iter().any(|a| a.path().is_ident("test"));
        let has_ignore = attrs.iter().any(|a| a.path().is_ident("ignore"));
        match (has_test, has_ignore) {
            (true, true) => WitnessKind::IgnoredTest,
            (true, false) => WitnessKind::Test,
            (false, _) => WitnessKind::Function,
        }
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
    path.segments.last().is_some_and(|s| s.ident == name)
}

impl FunctionIndexVisitor<'_> {
    /// Push a candidate entry for `name`. W7: every distinct (location, kind)
    /// pair is preserved so `validate_witness` can detect ambiguity. The
    /// pre-W7 behavior (silent first-wins) is the bug ATK-A2-005 named.
    fn push(&mut self, name: String, kind: WitnessKind) {
        self.index.entry(name).or_default().push(FunctionEntry {
            location: self.file_path.clone(),
            kind,
        });
    }
}

impl<'ast> syn::visit::Visit<'ast> for FunctionIndexVisitor<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        self.push(name, kind);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        self.push(name, kind);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        // W5: structural proptest! detection. When the macro path's last
        // segment is `proptest`, walk its tokens for `fn IDENT` patterns
        // and register each name with `WitnessKind::Proptest`.
        if macro_path_last_is(&mac.path, "proptest") {
            for name in extract_proptest_fn_names(&mac.tokens) {
                self.push(name, WitnessKind::Proptest);
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Determine the witness status for a single witness identifier string.
///
/// Resolution priority per ADR-013 + ADR-005 Amendment 3:
/// 1. Empty witness → `Missing`
/// 2. External-tool prefix (`clippy::`, `kani::`, ...) → `External`
/// 3. Phantom-type witness shape → `Resolved { PhantomType }`
/// 4. Workspace function lookup → `Resolved` / `Ambiguous` / `NotFound`
fn validate_witness(witness: &str, index: &FunctionIndex) -> WitnessStatus {
    // Normalize whitespace: the scan path records witnesses via ToTokens, which
    // emits spaced token form (`clippy :: no_panic_in_drop`, `PolarityProof :: < T > :: verified`).
    // Collapse all spacing around `::` and `<>` so every downstream detector
    // works on compact form regardless of source (hand-written or scan-path).
    let normalized_owned: String = {
        let collapsed = witness.split_whitespace().collect::<Vec<_>>().join(" ");
        collapsed
            .replace(" :: ", "::")
            .replace(":: ", "::")
            .replace(" ::", "::")
            .replace("< ", "<")
            .replace(" >", ">")
    };
    let trimmed = normalized_owned.trim();
    if trimmed.is_empty() {
        return WitnessStatus::Missing;
    }

    // Detect external-tool delegations.
    if let Some(tool) = detect_external_tool(trimmed) {
        return WitnessStatus::External {
            tool_hint: tool.to_string(),
        };
    }

    // Detect phantom-type witness shapes (ADR-013): `Path::<Args>::ctor` or
    // `Path::<Args>` or `Path` with trailing `()`. The shape recognition is
    // structural — we don't validate that the type exists.
    if let Some(phantom) = detect_phantom_type_witness(trimmed) {
        return WitnessStatus::Resolved {
            location: PathBuf::new(),
            witness_kind: phantom,
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

    let candidates = index.get(function_name);
    let Some(candidates) = candidates else {
        return WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        };
    };

    match candidates.as_slice() {
        [] => WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        },
        [only] => WitnessStatus::Resolved {
            location: only.location.clone(),
            witness_kind: only.kind.clone(),
        },
        many => WitnessStatus::Ambiguous {
            candidates: many.iter().map(|e| e.location.clone()).collect(),
        },
    }
}

/// Recognize a phantom-type witness shape per ADR-013.
///
/// Matches: `Type`, `Type::ctor`, `Type::<Args>`, `Type::<Args>::ctor`,
/// optionally with trailing `()`. The `<Args>` group, when present, contains
/// comma-separated type parameters.
///
/// We deliberately accept *any* type-name shape (capital-leading identifier
/// path) here because v0.1's audit has no symbol table — we cannot tell
/// whether `PolarityProof` refers to a real type. The `audit_hint` carries
/// the warning to verify the constructor is sealed; that's the recognize-
/// and-warn discipline ADR-013 §OQ1 specifies.
///
/// Returns `None` when the witness looks more like a function path than a
/// phantom-type construction (lowercase final segment with no type-param
/// list and no trailing `()`-after-`::`-segment ambiguity). The heuristic:
/// if the path contains a `<...>` segment, OR the last segment starts with
/// an uppercase letter (typical Rust type-name convention) AND there are
/// no trailing `()` (which would indicate a function call), treat it as
/// a phantom-type witness candidate.
fn detect_phantom_type_witness(witness: &str) -> Option<WitnessKind> {
    // Input is pre-normalized by validate_witness — compact token spacing guaranteed.
    let trimmed = witness.trim().trim_end_matches("()").trim();
    let has_turbofish = trimmed.contains("::<");
    if !has_turbofish {
        // No turbofish = not a phantom-type witness shape we recognize. The
        // bare-type-name shape (`Foo`) is indistinguishable from a function
        // path at this layer; we let the function-index path handle it.
        return None;
    }

    // Split into pre-turbofish, type-params, post-turbofish-ctor.
    let (before, after) = trimmed.split_once("::<")?;
    let (params_raw, ctor_part) = after.split_once('>')?;

    // Guard: nested generics like `Foo::<Option<Bar>, Baz>::new` make
    // split_once('>') fire at the inner `>`, leaving params_raw with
    // unbalanced `<`. Return None rather than emit FormalProof for a
    // garbled parse — let it fall through to function-index (NotFound).
    let open_count = params_raw.chars().filter(|&c| c == '<').count();
    if open_count > 0 {
        return None;
    }

    let proof_type = before.trim().to_string();
    let type_params: Vec<String> = params_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // Strip any remaining closing `>`s and the `::` separator left by
    // split_once('>') on nested-generic inputs like `Foo::<Bar<Baz>>::new`.
    let constructor = ctor_part
        .trim_start_matches(['>', ':'])
        .trim()
        .trim_end_matches("()")
        .trim();
    let constructor = if constructor.is_empty() {
        None
    } else {
        Some(constructor.to_string())
    };

    Some(WitnessKind::PhantomType {
        proof_type,
        type_params,
        constructor,
    })
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
            vec![FunctionEntry {
                location: PathBuf::from("src/lib.rs"),
                kind: WitnessKind::Test,
            }],
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

    /// Helper for tests that expect a single index entry for a name.
    /// Panics with a clear message if the name is unindexed or ambiguous.
    fn unique_kind(idx: &FunctionIndex, name: &str) -> WitnessKind {
        let entries = idx.get(name).unwrap_or_else(|| panic!("{name} indexed"));
        assert_eq!(
            entries.len(),
            1,
            "expected single index entry for {name}, got {entries:?}",
        );
        entries[0].kind.clone()
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
        assert_eq!(unique_kind(&idx, "first_proptest"), WitnessKind::Proptest);
        assert_eq!(unique_kind(&idx, "second_proptest"), WitnessKind::Proptest);
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
        assert_eq!(
            unique_kind(&idx, "qualified_form_proptest"),
            WitnessKind::Proptest,
        );
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
        assert_eq!(
            unique_kind(&idx, "plain_test"),
            WitnessKind::Test,
            "plain_test outside proptest! must be Test, not Proptest, even when \
             the same file contains a proptest! invocation",
        );
        assert_eq!(unique_kind(&idx, "proptest_one"), WitnessKind::Proptest);
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
        assert_eq!(
            unique_kind(&idx, "doc_comment_only_test"),
            WitnessKind::Test,
            "doc-comment mention must not trigger Proptest",
        );
    }

    #[test]
    fn w5_plain_function_is_classified_function() {
        let src = r"
            fn no_attribute_function() {}
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "no_attribute_function"),
            WitnessKind::Function,
        );
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

    #[test]
    fn detect_phantom_nested_generic_returns_none() {
        // `Witnessed::<Option<MyType>, MyWitness>::try_new` has a nested `<>`
        // inside the type-param region. split_once('>') fires at the inner `>`,
        // producing malformed fields. The balanced-bracket guard must return None
        // so audit falls through to function-index (NotFound), not FormalProof.
        assert_eq!(
            detect_phantom_type_witness("Witnessed::<Option<MyType>, MyWitness>::try_new"),
            None,
        );
        // Simple non-nested shape must still work.
        assert!(matches!(
            detect_phantom_type_witness("PolarityProof::<FrameTranslation>::verified"),
            Some(WitnessKind::PhantomType { .. }),
        ));
    }
}

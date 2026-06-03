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
//! ## Status (v0.1.0-rc.1)
//!
//! Initial implementation. Discovers attribute invocations, matches presentations
//! against immunities at the same item level, synthesizes fingerprint matches
//! against unmarked code (W6a), and collects `#[descended_from]` lineage edges
//! with cycle detection (A3 D1+D2). Future versions will add:
//!
//! - `#[descended_from]` propagation (synthesizing inherited presentations on
//!   descendants) — lineage edges + cycle/depth guards already land in
//!   [`ScanReport::lineage_edges`] (A3 D1+D2); the propagation step depends on
//!   the ADR-005 sub-clause F ruling on inherited-witness re-verification
//! - Cross-crate antigen declaration discovery (A3 D3)
//! - Witness validation (delegating to clippy/kani/proptest as appropriate)
//! - Performance optimizations (incremental scan, parallel file walks)
//!
//! ## Known v1 limitations
//!
//! 1. **Witness validation is presence-only at scan time** — the scan records
//!    the witness identifier verbatim. Validity classification (`Test`,
//!    `Proptest`, `PhantomType`, `Function`, `External`) and tier mapping
//!    (`Reachability`, `Execution`, `FormalProof`) are the [`crate::audit`]
//!    module's job (shipped in W7 per ADR-001 Amendment 1 + ADR-013).
//!
//! W3 (sweep A2) replaced the prior 20-line proximity heuristic in
//! [`ScanReport::unaddressed_presentations`] with structural item-identity
//! matching via [`ItemTarget`] + [`ItemTarget::addresses`]. See those types.

use std::path::{Path, PathBuf};

use antigen_macros::{antigen_tolerance, presents};
use syn::parse::Parse;
use syn::visit::Visit;
use walkdir::WalkDir;

// ============================================================================
// Scan-side attribute argument parsers
//
// These mirror the proc-macro-side parsers in antigen-macros but live in a
// regular (non-proc-macro) crate so they can be used at scan time. Both must
// produce identical results for the same input — the canonical representation
// is `syn::LitStr::value()`, which correctly unescapes string literal content.
//
// The proc-macro crate cannot be re-exported as a library (proc-macro = true
// crates export only their macro entry points), so we duplicate the parsing
// logic here. Any change to the attribute grammar must be reflected in both.
// ============================================================================

/// Scan-time parse of `#[antigen(name = "...", fingerprint = "...", ...)]`.
struct ScanAntigenArgs {
    name: String,
    fingerprint: Option<String>,
    family: Option<String>,
    summary: Option<String>,
    /// Category strings parsed from `category = AntigenCategory::X` or
    /// `category = [AntigenCategory::X, ...]` (ADR-028). Stored as strings
    /// for forward-compat; callers map to `AntigenCategory` via
    /// `AntigenCategory::parse_category`.
    category: Vec<String>,
}

impl Parse for ScanAntigenArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Ident, LitStr, Token};
        let mut name: Option<String> = None;
        let mut fingerprint: Option<String> = None;
        let mut family: Option<String> = None;
        let mut summary: Option<String> = None;
        let mut category: Vec<String> = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name = Some(lit.value());
                }
                "fingerprint" => {
                    let lit: LitStr = input.parse()?;
                    fingerprint = Some(lit.value());
                }
                "family" => {
                    let lit: LitStr = input.parse()?;
                    family = Some(lit.value());
                }
                "summary" => {
                    let lit: LitStr = input.parse()?;
                    summary = Some(lit.value());
                }
                "references" => {
                    // Consume the array without storing (not used in scan output yet).
                    let _arr: syn::ExprArray = input.parse()?;
                }
                "category" => {
                    // Parse path expression(s): single or array form.
                    fn path_to_string(expr: &syn::Expr) -> Option<String> {
                        if let syn::Expr::Path(p) = expr {
                            let segs: Vec<String> = p
                                .path
                                .segments
                                .iter()
                                .map(|s| s.ident.to_string())
                                .collect();
                            Some(segs.join("::"))
                        } else {
                            None
                        }
                    }
                    let val: syn::Expr = input.parse()?;
                    match &val {
                        syn::Expr::Array(arr) => {
                            for elem in &arr.elems {
                                if let Some(s) = path_to_string(elem) {
                                    category.push(s);
                                }
                            }
                        }
                        single => {
                            if let Some(s) = path_to_string(single) {
                                category.push(s);
                            }
                        }
                    }
                }
                _ => {
                    // Unknown field: consume the value expression and continue.
                    let _: syn::Expr = input.parse()?;
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name: name.unwrap_or_default(),
            fingerprint,
            family,
            summary,
            category,
        })
    }
}

/// Scan-time parse of `#[presents(AntigenType, [requires = <predicate>],
/// [proof = <expr>])]` (ADR-029 R5 — site-attached evidence folds onto
/// `#[presents]`). Mirrors `ScanImmuneArgs`'s forward-compat posture: unknown
/// fields are consumed silently (the macro side is the strict enforcer).
struct ScanPresentsArgs {
    antigen_type: String,
    /// Substrate-witness predicate from `requires = <predicate>` (ADR-029).
    requires_predicate: Option<String>,
    /// Phantom-type proof expression from `proof = <expr>`, rendered as its
    /// token string.
    proof: Option<String>,
}

impl Parse for ScanPresentsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use antigen_attestation::parser::RequiresExpr;
        use quote::ToTokens;
        use syn::{Ident, Path, Token};

        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut requires_predicate: Option<String> = None;
        let mut proof: Option<String> = None;
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "requires" => {
                    let pred: RequiresExpr = input.parse()?;
                    requires_predicate = Some(pred.to_json());
                }
                "proof" => {
                    let expr: syn::Expr = input.parse()?;
                    proof = Some(expr.to_token_stream().to_string());
                }
                _ => {
                    // Forward-compat: consume unknown values silently (the macro
                    // side rejects them). Recall-tuned scan per ADR-010.
                    let _: syn::Expr = input.parse()?;
                }
            }
        }

        Ok(Self {
            antigen_type,
            requires_predicate,
            proof,
        })
    }
}

/// Scan-time parse of `#[immune(AntigenType, witness = expr, ...)]`.
struct ScanImmuneArgs {
    antigen_type: String,
    witness: String,
    /// Substrate-witness predicate parsed straight from the source
    /// attribute (ADR-019 §P3b). When present, scan threads this JSON to
    /// the audit evaluator directly — independent of macro expansion.
    /// `None` when the declaration uses code-tier `witness = ...` only.
    requires_predicate: Option<String>,
}

impl Parse for ScanImmuneArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use antigen_attestation::parser::RequiresExpr;
        use syn::{Ident, Path, Token};
        // First token is the antigen path.
        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut witness = String::new();
        let mut requires_predicate: Option<String> = None;
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witness" => {
                    // Render the witness expression as its token string — this is the
                    // identifier or path the user wrote, e.g. `my_test_fn` or
                    // `clippy::no_panic_in_drop`. We use `quote::ToTokens` to get
                    // a canonical rendering without depending on string heuristics.
                    use quote::ToTokens;
                    let val: syn::Expr = input.parse()?;
                    witness = val.to_token_stream().to_string();
                }
                "requires" => {
                    // Substrate-witness predicate (ADR-019). Reuse the shared
                    // parser from antigen-attestation so the JSON the scan
                    // layer ships is byte-identical to what the macro side
                    // would emit. Failing to parse here is a hard error: a
                    // malformed predicate is silent suppression of immunity
                    // intent, which is exactly the failure-class ADR-005
                    // sub-clause F was built to catch.
                    let pred: RequiresExpr = input.parse()?;
                    requires_predicate = Some(pred.to_json());
                }
                _ => {
                    // Unknown / rationale / other fields: consume the value
                    // silently. Forward-compat per ADR-009 adoption gradient.
                    let _: syn::Expr = input.parse()?;
                }
            }
        }

        Ok(Self {
            antigen_type,
            witness,
            requires_predicate,
        })
    }
}

/// Scan-time parse of `#[antigen_tolerance(antigen, rationale = "...",
/// until = "...", see = [...], requires = <predicate>)]`. Per ADR-011 +
/// ADR-019 (tolerance-side substrate-witness predicates).
struct ScanToleranceArgs {
    antigen_type: String,
    rationale: String,
    until: Option<String>,
    see: Vec<String>,
    /// Tolerance-side substrate-witness predicate (ADR-019 tolerance tier),
    /// parsed straight from the source attribute. Same rationale as
    /// `ScanImmuneArgs::requires_predicate`.
    requires_predicate: Option<String>,
}

impl Parse for ScanToleranceArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use antigen_attestation::parser::RequiresExpr;
        use syn::{Expr, Ident, Lit, LitStr, Path, Token};
        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut rationale = String::new();
        let mut until: Option<String> = None;
        let mut see: Vec<String> = Vec::new();
        let mut requires_predicate: Option<String> = None;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = lit.value();
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until = Some(lit.value());
                }
                "see" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            see.push(s.value());
                        }
                    }
                }
                "requires" => {
                    let pred: RequiresExpr = input.parse()?;
                    requires_predicate = Some(pred.to_json());
                }
                _ => {
                    // Unknown field: consume silently (forward-compat per
                    // ADR-009 adoption-gradient tolerance).
                    let _: Expr = input.parse()?;
                }
            }
        }

        Ok(Self {
            antigen_type,
            rationale,
            until,
            see,
            requires_predicate,
        })
    }
}

/// Scan-side parser for `#[antigen_generates(antigen_type, rationale = "...")]`
/// (ADR-014). Mirrors the macro-side `GeneratesArgs` but parses straight from
/// the source attribute. Unknown fields are consumed silently for forward-compat.
struct ScanGeneratesArgs {
    antigen_type: String,
    rationale: String,
}

impl Parse for ScanGeneratesArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitStr, Path, Token};
        let antigen_path: Path = input.parse()?;
        let antigen_type = antigen_path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        let mut rationale = String::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = lit.value();
                }
                _ => {
                    // Unknown field (witness_template, if_attr_present, future):
                    // consume silently for adoption-gradient forward-compat.
                    let _: Expr = input.parse()?;
                }
            }
        }

        Ok(Self {
            antigen_type,
            rationale,
        })
    }
}

// ============================================================================
// Deferred-Defense Family scan-time parsers (ADR-023)
//
// These mirror the macro-side parsers in antigen-macros/src/parse.rs but live
// here for scan-time source walking. Unknown fields are consumed silently for
// forward-compat (adoption-gradient per ADR-009). Required-field validation is
// intentionally lenient on the scan side — the macro side is the parse-time
// enforcer; the scan side records what it finds.
// ============================================================================

/// Scan-time loose capture for all six recurrent-emergence primitives
/// (ADR-024 §Family 2). Mirrors `ScanAntigenArgs`'s forward-compat posture:
/// every field is optional; per-kind required-field validation is the audit
/// layer's job (scan is recall-tuned per ADR-010). `from_itches` /
/// `anchored_by` arrays capture path-expression idents as final segments.
#[derive(Default)]
struct ScanRecurrentArgs {
    name: Option<String>,
    antigen_type: Option<String>,
    description: Option<String>,
    instances: Option<u64>,
    since: Option<String>,
    rationale: Option<String>,
    from_itches: Vec<String>,
    anchored_by: Vec<String>,
    managed_by: Option<String>,
    contributing_to: Option<String>,
}

impl Parse for ScanRecurrentArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitInt, LitStr, Path, Token};
        let mut out = Self::default();

        // Optional leading positional antigen-type path (recurrence_anchor,
        // chronic accept it positionally; others use `antigen = ...`).
        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            let path: Path = input.parse()?;
            out.antigen_type = path.segments.last().map(|s| s.ident.to_string());
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    out.name = Some(lit.value());
                }
                "antigen" => {
                    let path: Path = input.parse()?;
                    out.antigen_type = path.segments.last().map(|s| s.ident.to_string());
                }
                "description" | "summary" => {
                    let lit: LitStr = input.parse()?;
                    out.description = Some(lit.value());
                }
                "instances" => {
                    let lit: LitInt = input.parse()?;
                    out.instances = lit.base10_parse::<u64>().ok();
                }
                "since" => {
                    let lit: LitStr = input.parse()?;
                    out.since = Some(lit.value());
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    out.rationale = Some(lit.value());
                }
                "from_itches" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Path(p) = elem {
                            if let Some(seg) = p.path.segments.last() {
                                out.from_itches.push(seg.ident.to_string());
                            }
                        }
                    }
                }
                "anchored_by" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Path(p) = elem {
                            if let Some(seg) = p.path.segments.last() {
                                out.anchored_by.push(seg.ident.to_string());
                            }
                        }
                    }
                }
                "managed_by" => {
                    let lit: LitStr = input.parse()?;
                    out.managed_by = Some(lit.value());
                }
                "contributing_to" => {
                    let lit: LitStr = input.parse()?;
                    out.contributing_to = Some(lit.value());
                }
                // Forward-compat: known-but-not-captured fields (threshold,
                // status) + any unknown field are consumed silently per the
                // ADR-009 adoption gradient. Audit handles required-field
                // validation; scan is recall-tuned (ADR-010).
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(out)
    }
}

/// Scan-time loose capture for all three mucosal-boundary primitives
/// (ADR-027 + Amendment 1). Every field optional; per-kind required-field
/// validation is the audit layer's job. `kind`/`boundary` both populate
/// `boundary_kind` (final path segment); `handled_by` captures the
/// delegate handler's final path segment.
#[derive(Default)]
struct ScanMucosalArgs {
    boundary_kind: Option<String>,
    rationale: Option<String>,
    handled_by: Option<String>,
    accepts: Option<String>,
    reviewed_by: Option<String>,
    until: Option<String>,
}

impl Parse for ScanMucosalArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitStr, Path, Token};
        let mut out = Self::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "kind" | "boundary" => {
                    // MucosalKind::X path expression → final segment.
                    let path: Path = input.parse()?;
                    out.boundary_kind = path.segments.last().map(|s| s.ident.to_string());
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    out.rationale = Some(lit.value());
                }
                "handled_by" => {
                    // syn::Path per Amendment 1 Change 4 → final segment.
                    let path: Path = input.parse()?;
                    out.handled_by = path.segments.last().map(|s| s.ident.to_string());
                }
                "accepts" => {
                    let lit: LitStr = input.parse()?;
                    out.accepts = Some(lit.value());
                }
                "reviewed_by" => {
                    let lit: LitStr = input.parse()?;
                    out.reviewed_by = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    out.until = Some(lit.value());
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(out)
    }
}

/// Scan-time loose capture for all eight prescriptive work-orchestration
/// primitives (ADR-033). Every field optional; per-kind required-field
/// validation is the macro's (parse-time) + audit's job — scan is recall-tuned
/// (ADR-010). The capture maps each macro's per-shape field NAMES onto the
/// shared [`PrescriptiveDeclaration`] slots:
/// - list slot (`items`): `needs` | `rule_out` | `priority_order`
/// - fill who-refs (`filled_by`): `filled_by` | `to` | `deep_investigation_by`
///   | `investigator` | `triaged_by`
/// - review who-refs (`reviewed_by`): `reviewed_by` | `reviewer`
/// - `ordered_by`; `frame`: `due` | `response_due` | `re_triage_due` |
///   `runs_until` | `until`
/// - `need_text`: `treatment` | `request_text` | `symptom` | `test_kind` |
///   `reason`; `label`: `diagnosis` | `location` | `scope`
#[derive(Default)]
struct ScanPrescriptiveArgs {
    items: Vec<String>,
    filled_by: Vec<String>,
    reviewed_by: Vec<String>,
    ordered_by: Option<String>,
    frame: Option<String>,
    need_text: Option<String>,
    label: Option<String>,
}

/// Collect the string-literal elements of an `[ "a", "b" ]` array expression
/// (non-string elements are skipped — scan is recall-tuned). Free helper so it
/// is not an item-after-statement inside the parse loop.
fn prescriptive_str_array(input: syn::parse::ParseStream) -> syn::Result<Vec<String>> {
    let arr: syn::ExprArray = input.parse()?;
    let mut v = Vec::new();
    for elem in &arr.elems {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = elem
        {
            v.push(s.value());
        }
    }
    Ok(v)
}

impl Parse for ScanPrescriptiveArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitStr, Token};
        let str_array = prescriptive_str_array;
        let mut out = Self::default();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                // The shape's required list (exactly one is meaningful per kind).
                "needs" | "rule_out" | "priority_order" => out.items = str_array(input)?,
                // Fill who-refs (across shapes).
                "filled_by" => out.filled_by = str_array(input)?,
                "to" | "deep_investigation_by" | "investigator" | "triaged_by" => {
                    let lit: LitStr = input.parse()?;
                    out.filled_by.push(lit.value());
                }
                // Review who-refs.
                "reviewed_by" => out.reviewed_by = str_array(input)?,
                "reviewer" => {
                    let lit: LitStr = input.parse()?;
                    out.reviewed_by.push(lit.value());
                }
                "ordered_by" => {
                    let lit: LitStr = input.parse()?;
                    out.ordered_by = Some(lit.value());
                }
                // Temporal frame (across shapes).
                "due" | "response_due" | "re_triage_due" | "runs_until" | "until" => {
                    let lit: LitStr = input.parse()?;
                    out.frame = Some(lit.value());
                }
                // Primary free-text content.
                "treatment" | "request_text" | "symptom" | "test_kind" | "reason" => {
                    let lit: LitStr = input.parse()?;
                    out.need_text = Some(lit.value());
                }
                // Secondary opaque label.
                "diagnosis" | "location" | "scope" => {
                    let lit: LitStr = input.parse()?;
                    out.label = Some(lit.value());
                }
                // Forward-compat: unknown field consumed silently (recall-tuned).
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(out)
    }
}

struct ScanAnergyArgs {
    antigen_type: Option<String>,
    reason: String,
    until: String,
    expected_co_stimulation: Option<String>,
    signed_by: Option<String>,
}

impl Parse for ScanAnergyArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitStr, Path, Token};
        let mut antigen_type: Option<String> = None;
        let mut reason = String::new();
        let mut until = String::new();
        let mut expected_co_stimulation: Option<String> = None;
        let mut signed_by: Option<String> = None;

        // Optional leading positional antigen type path
        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            let path: Path = input.parse()?;
            antigen_type = path.segments.last().map(|s| s.ident.to_string());
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "reason" => {
                    let lit: LitStr = input.parse()?;
                    reason = lit.value();
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until = lit.value();
                }
                "expected_co_stimulation" => {
                    let lit: LitStr = input.parse()?;
                    expected_co_stimulation = Some(lit.value());
                }
                "signed_by" => {
                    let lit: LitStr = input.parse()?;
                    signed_by = Some(lit.value());
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            antigen_type,
            reason,
            until,
            expected_co_stimulation,
            signed_by,
        })
    }
}

struct ScanImmunosuppressArgs {
    antigen_type: Option<String>,
    rationale: String,
    until: String,
    since: Option<String>,
    duration_cap: Option<u64>,
    signed_by: Option<String>,
}

impl Parse for ScanImmunosuppressArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitInt, LitStr, Path, Token};
        let mut antigen_type: Option<String> = None;
        let mut rationale = String::new();
        let mut until = String::new();
        let mut since: Option<String> = None;
        let mut duration_cap: Option<u64> = None;
        let mut signed_by: Option<String> = None;

        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            let path: Path = input.parse()?;
            antigen_type = path.segments.last().map(|s| s.ident.to_string());
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = lit.value();
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until = lit.value();
                }
                "since" => {
                    let lit: LitStr = input.parse()?;
                    since = Some(lit.value());
                }
                "duration_cap" => {
                    let lit: LitInt = input.parse()?;
                    duration_cap = lit.base10_parse::<u64>().ok();
                }
                "signed_by" => {
                    let lit: LitStr = input.parse()?;
                    signed_by = Some(lit.value());
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            antigen_type,
            rationale,
            until,
            since,
            duration_cap,
            signed_by,
        })
    }
}

struct ScanPoxpartyArgs {
    antigen_type: Option<String>,
    exercise_type: String,
    until: String,
    name: Option<String>,
    rationale: Option<String>,
    signed_by: Option<String>,
}

impl Parse for ScanPoxpartyArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitStr, Path, Token};
        let mut antigen_type: Option<String> = None;
        let mut exercise_type = String::new();
        let mut until = String::new();
        let mut name: Option<String> = None;
        let mut rationale: Option<String> = None;
        let mut signed_by: Option<String> = None;

        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            let path: Path = input.parse()?;
            antigen_type = path.segments.last().map(|s| s.ident.to_string());
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "exercise_type" => {
                    let lit: LitStr = input.parse()?;
                    exercise_type = lit.value();
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until = lit.value();
                }
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name = Some(lit.value());
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = Some(lit.value());
                }
                "signed_by" => {
                    let lit: LitStr = input.parse()?;
                    signed_by = Some(lit.value());
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            antigen_type,
            exercise_type,
            until,
            name,
            rationale,
            signed_by,
        })
    }
}

struct ScanOrientArgs {
    antigen_type: Option<String>,
    see: Vec<String>,
    adr: Option<String>,
    #[allow(dead_code)]
    attestation_optional: bool,
}

impl Parse for ScanOrientArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, Lit, LitStr, Path, Token};
        let mut antigen_type: Option<String> = None;
        let mut see: Vec<String> = Vec::new();
        let mut adr: Option<String> = None;
        let mut attestation_optional = false;

        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            // Check if bare `attestation_optional` flag
            let fork = input.fork();
            let ident: Ident = fork
                .parse()
                .unwrap_or_else(|_| Ident::new("_", proc_macro2::Span::call_site()));
            if ident == "attestation_optional" && (fork.is_empty() || fork.peek(Token![,])) {
                let _: Ident = input.parse()?;
                attestation_optional = true;
            } else {
                let path: Path = input.parse()?;
                antigen_type = path.segments.last().map(|s| s.ident.to_string());
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }

        while !input.is_empty() {
            if input.peek(Ident) {
                let fork = input.fork();
                let ident: Ident = fork
                    .parse()
                    .unwrap_or_else(|_| Ident::new("_", proc_macro2::Span::call_site()));
                if ident == "attestation_optional" && (fork.is_empty() || fork.peek(Token![,])) {
                    let _: Ident = input.parse()?;
                    attestation_optional = true;
                    if !input.is_empty() {
                        let _ = input.parse::<Token![,]>();
                    }
                    continue;
                }
            }

            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "see" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            see.push(s.value());
                        }
                    }
                }
                "adr" => {
                    let lit: LitStr = input.parse()?;
                    adr = Some(lit.value());
                }
                "attestation_optional" => {
                    let lit: syn::LitBool = input.parse()?;
                    attestation_optional = lit.value();
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            antigen_type,
            see,
            adr,
            attestation_optional,
        })
    }
}

// ============================================================================
// Convergent-Evidence Family scan-side arg parsers (ADR-024)
// ============================================================================

struct ScanDiagnosticArgs {
    modality_classes: Vec<String>,
    min_independent: Option<u64>,
}

impl Parse for ScanDiagnosticArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, LitInt, Token};
        let mut modality_classes = Vec::new();
        let mut min_independent: Option<u64> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "modalities" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Path(p) = elem {
                            if let Some(seg) = p.path.segments.last() {
                                modality_classes.push(seg.ident.to_string());
                            }
                        }
                    }
                }
                "min_independent" => {
                    let lit: LitInt = input.parse()?;
                    min_independent = lit.base10_parse::<u64>().ok();
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            modality_classes,
            min_independent,
        })
    }
}

struct ScanClonalArgs {
    witness: Option<String>,
    iterations: Option<u64>,
    seed_kind: Option<String>,
}

impl Parse for ScanClonalArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use quote::ToTokens;
        use syn::{Expr, Ident, LitInt, Token};
        let mut witness: Option<String> = None;
        let mut iterations: Option<u64> = None;
        let mut seed_kind: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witness" => {
                    let e: Expr = input.parse()?;
                    witness = Some(e.to_token_stream().to_string());
                }
                "iterations" => {
                    let lit: LitInt = input.parse()?;
                    iterations = lit.base10_parse::<u64>().ok();
                }
                "seed" => {
                    let e: Expr = input.parse()?;
                    if let Expr::Path(p) = &e {
                        if let Some(seg) = p.path.segments.last() {
                            seed_kind = Some(seg.ident.to_string());
                        }
                    } else if let Expr::Call(c) = &e {
                        if let Expr::Path(p) = &*c.func {
                            if let Some(seg) = p.path.segments.last() {
                                seed_kind = Some(seg.ident.to_string());
                            }
                        }
                    }
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            witness,
            iterations,
            seed_kind,
        })
    }
}

struct ScanIggArgs {
    witnesses: Vec<String>,
    historical_span: Option<u64>,
    min_reattestations: Option<u64>,
}

impl Parse for ScanIggArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use quote::ToTokens;
        use syn::{Expr, Ident, LitInt, Token};
        let mut witnesses = Vec::new();
        let mut historical_span: Option<u64> = None;
        let mut min_reattestations: Option<u64> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witnesses" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        witnesses.push(elem.to_token_stream().to_string());
                    }
                }
                "historical_span" => {
                    let lit: LitInt = input.parse()?;
                    historical_span = lit.base10_parse::<u64>().ok();
                }
                "min_reattestations" => {
                    let lit: LitInt = input.parse()?;
                    min_reattestations = lit.base10_parse::<u64>().ok();
                }
                _ => {
                    let _: Expr = input.parse()?;
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self {
            witnesses,
            historical_span,
            min_reattestations,
        })
    }
}

struct ScanCrossreactiveArgs {
    fingerprints: Vec<String>,
}

impl Parse for ScanCrossreactiveArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::{Expr, Ident, Lit, LitStr, Token};
        let mut fingerprints = Vec::new();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "fingerprints" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            fingerprints.push(s.value());
                        }
                    }
                }
                _ => {
                    if input.peek(LitStr) {
                        let _: LitStr = input.parse()?;
                    } else {
                        let _: Expr = input.parse()?;
                    }
                }
            }
            if !input.is_empty() {
                let _ = input.parse::<Token![,]>();
            }
        }
        Ok(Self { fingerprints })
    }
}

mod types;
// The public scan data model — exactly the `pub` surface scan.rs exposed before
// the decomposition (API-invisible). Explicit (not a glob) so the crate-internal
// matching helpers below are NOT widened to the public API.
pub use types::{
    AntigenDeclaration, ConvergentEvidence, ConvergentEvidenceKind, Defense, DeferredDefense,
    DeferredDefenseKind, GeneratesDeclaration, Immunity, ItemTarget, LineageEdge, MatchKind,
    MucosalDeclaration, MucosalKindTag, ParseFailure, PartitionedPresentations,
    PrescriptiveDeclaration, PrescriptiveKind, Presentation, ProvenanceEntry, RecurrentDeclaration,
    RecurrentKind, ScanCoverage, ScanReport, Toleration, UnaddressedPresentation, WorkShape,
};
// Crate-internal shared matching rule + the lineage depth cap. `pub` inside the
// private `types` module (so they are crate-bounded, not public API) and brought
// here `pub(crate)` so the scan passes + the audit cross-checks reach them via
// `crate::scan::{...}` exactly as before — NOT part of the public surface.
pub(crate) use types::{
    canonical_paths_match, defense_addresses, locus_matches, MAX_LINEAGE_DEPTH,
};

mod lineage;
pub(crate) use lineage::{dedupe_lineage_edges, detect_lineage_failures};
// `canonicalise_cycle` is a private lineage helper the scan test module exercises
// directly; re-export it into the scan namespace under `#[cfg(test)]` only (test
// reach, not public API) so the test module's `use super::*` keeps resolving it.
#[cfg(test)]
pub(crate) use lineage::canonicalise_cycle;

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

mod finalize;
pub(crate) use finalize::{finalize_report, synthesize_inherited_presentations};

mod multi_crate;
pub use multi_crate::{
    enumerate_dep_crate_roots, enumerate_workspace_member_roots, scan_workspace_multi_crate,
    CrateOrigin, DepCrateRoot, WorkspaceMemberRoot,
};
// Two cross-member resolution passes the scan test module exercises directly;
// re-export under `#[cfg(test)]` only (test reach, not public API).
#[cfg(test)]
pub(crate) use multi_crate::{
    resolve_cross_member_addresses, resolve_cross_member_lineage_parents,
};

mod synthesis;
pub(crate) use synthesis::{generates_synthesis_pass, synthesis_pass};

mod parse;
// The parsing engine is crate-internal (the scan passes drive it). Re-export the
// pieces the synthesis pass + the scan_workspace walk share — the visitor and the
// syn-render helpers — `pub(crate)` at the scan root, NOT to the public API.
pub(crate) use parse::{attr_is, render_path, render_type, ScanVisitor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scan_report_has_no_unaddressed() {
        let report = ScanReport::default();
        assert!(report.unaddressed_presentations().is_empty());
    }

    #[test]
    fn antigen_args_parses_name_and_fingerprint() {
        let tokens: proc_macro2::TokenStream = r#"
            name = "frame-translation",
            fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")"
        "#
        .parse()
        .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "frame-translation");
        // The fingerprint must be correctly unescaped — not contain raw backslash-quote.
        let fp = args.fingerprint.unwrap();
        assert!(
            fp.contains("has_method(\"meet\""),
            "fingerprint should contain unescaped double-quotes, got: {fp:?}"
        );
        assert!(
            !fp.contains(r#"\""#),
            "fingerprint must not contain raw backslash-quote escape sequences, got: {fp:?}"
        );
    }

    #[test]
    fn antigen_args_parses_optional_fields() {
        let tokens: proc_macro2::TokenStream =
            r#"name = "panicking-in-drop", fingerprint = "impl Drop", family = "boundary-violation", summary = "Drop impl can panic""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "panicking-in-drop");
        assert_eq!(args.family.as_deref(), Some("boundary-violation"));
        assert_eq!(args.summary.as_deref(), Some("Drop impl can panic"));
    }

    #[test]
    fn immune_args_parses_antigen_type_and_witness() {
        let tokens: proc_macro2::TokenStream = r"PanickingInDrop, witness = no_panic_in_drop_test"
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanImmuneArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type, "PanickingInDrop");
        assert_eq!(args.witness, "no_panic_in_drop_test");
    }

    #[test]
    fn immune_args_parses_path_witness() {
        let tokens: proc_macro2::TokenStream =
            r"FrameTranslation, witness = clippy :: no_panic_in_drop"
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanImmuneArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type, "FrameTranslation");
        // witness is the token-stream rendering of the expression
        assert!(args.witness.contains("no_panic_in_drop"));
    }

    // ========================================================================
    // Cross-parser equivalence fixtures
    //
    // These fixtures must stay in sync with `antigen-macros::parse::tests::
    // ANTIGEN_PARSER_FIXTURES`. The invariant: for any input the proc-macro
    // parser accepts as valid, the scan parser must produce equivalent semantic
    // content for the four overlapping fields (name, fingerprint, family,
    // summary). The two parsers live in different crates by structural
    // necessity (proc-macro = true crates can't be linked as libraries) and
    // their drift was the substance of ATK-001-2 in pre-team scaffolding.
    //
    // When adding a fixture here, add the matching one in antigen-macros's
    // parse.rs ANTIGEN_PARSER_FIXTURES table. Field-additions to the antigen
    // attribute grammar must add fixtures in BOTH crates to lock the
    // equivalence.
    // ========================================================================

    // (input, expected_name, expected_fingerprint, expected_family, expected_summary)
    type ScanFixture = (
        &'static str,
        &'static str,
        &'static str,
        Option<&'static str>,
        Option<&'static str>,
    );

    const SCAN_PARSER_FIXTURES: &[ScanFixture] = &[
        (
            r#"name = "panicking-in-drop", fingerprint = "impl Drop with panic""#,
            "panicking-in-drop",
            "impl Drop with panic",
            None,
            None,
        ),
        (
            r#"name = "frame-translation", fingerprint = "class enum + meet", family = "semantic-drift", summary = "Polarity inverts at the frame boundary""#,
            "frame-translation",
            "class enum + meet",
            Some("semantic-drift"),
            Some("Polarity inverts at the frame boundary"),
        ),
        (
            r#"name = "x", fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")""#,
            "x",
            r#"item: enum, has_method("meet", "(Self, Self) -> Self")"#,
            None,
            None,
        ),
        (
            r#"summary = "S", family = "F", fingerprint = "FP", name = "n""#,
            "n",
            "FP",
            Some("F"),
            Some("S"),
        ),
        (
            r#"name = "x", fingerprint = "y", references = ["GAP-1", "DEC-2"]"#,
            "x",
            "y",
            None,
            None,
        ),
        (
            "name = \"multi-line\",\n\tfingerprint = \"shape\",\n\tfamily = \"family\"",
            "multi-line",
            "shape",
            Some("family"),
            None,
        ),
    ];

    #[test]
    fn scan_parser_accepts_all_macro_fixtures() {
        for (input, exp_name, exp_fp, exp_family, exp_summary) in SCAN_PARSER_FIXTURES {
            let tokens: proc_macro2::TokenStream = input
                .parse()
                .unwrap_or_else(|e| panic!("fixture failed to tokenize: {input:?}: {e}"));
            let args = syn::parse2::<ScanAntigenArgs>(tokens)
                .unwrap_or_else(|e| panic!("scan parser rejected fixture {input:?}: {e}"));
            assert_eq!(&args.name, exp_name, "name mismatch for fixture: {input:?}");
            assert_eq!(
                args.fingerprint.as_deref(),
                Some(*exp_fp),
                "fingerprint mismatch for fixture: {input:?}"
            );
            assert_eq!(
                args.family.as_deref(),
                *exp_family,
                "family mismatch for fixture: {input:?}"
            );
            assert_eq!(
                args.summary.as_deref(),
                *exp_summary,
                "summary mismatch for fixture: {input:?}"
            );
        }
    }

    #[test]
    fn scan_parser_tolerates_unknown_fields() {
        // Asymmetry from macro side: scan tolerates unknown fields silently
        // (forward-compat for fields added later that this scan binary doesn't
        // know yet). Macro side rejects them strictly.
        let tokens: proc_macro2::TokenStream =
            r#"name = "x", fingerprint = "y", future_field = "irrelevant""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "x");
        assert_eq!(args.fingerprint.as_deref(), Some("y"));
    }

    #[test]
    fn scan_parser_tolerates_missing_required_fields() {
        // Asymmetry from macro side: scan defaults to empty rather than
        // erroring. Malformed declarations get aggregated into parse_failures
        // upstream rather than blocking the scan.
        let tokens: proc_macro2::TokenStream = r#"name = "only-name""#.parse().unwrap();
        let args = syn::parse2::<ScanAntigenArgs>(tokens).unwrap();
        assert_eq!(args.name, "only-name");
        assert_eq!(args.fingerprint, None);
    }

    // ========================================================================
    // Property tests (W1) — proptest invariants over the scan-side parser.
    //
    // These are the scan-side half of the cross-parser equivalence story.
    // Their invariants mirror the macro-side proptests in
    // `antigen-macros/src/parse.rs::parser_props`. Both sides share the same
    // input strategies (literal-copied; if you change one, change the other
    // in the same commit) — this is the by-construction substitute for
    // running both parsers in one binary, which the proc-macro crate
    // separation forbids.
    //
    // Cross-parser invariants asserted (macro-side P1-P8 mirror):
    //
    //   I1 (P1 mirror) — round-trip on intersection: any input the macro
    //        side accepts, the scan side accepts and produces equivalent
    //        semantic content for name/fingerprint/family/summary.
    //
    //   I2 (P2 mirror) — order-invariance: shuffling fields doesn't change
    //        the parsed result on the scan side.
    //
    //   I3 (asymmetry) — scan tolerates what macro rejects:
    //        - unknown fields: macro errors, scan silently consumes
    //        - missing required: macro errors, scan defaults to empty
    //        These asymmetries are intentional (forward-compat + non-blocking
    //        scan progress on partial workspaces) and are documented as
    //        properties so they don't accidentally regress.
    //
    // The macro-side parser reports were tested under `parser_props` in the
    // sister crate. Together, the two test modules form the W1 floor that
    // ADR-001 Amendment 1 C5 (drift-detection-at-scan-time) makes
    // load-bearing.
    // ========================================================================

    mod parser_props {
        use super::super::*;
        use proc_macro2::TokenStream;
        use proptest::prelude::*;

        /// Rust strict + reserved keywords that cannot appear as path
        /// segments. Kept in sync with `antigen-macros/src/parse.rs::
        /// parser_props::RUST_KEYWORDS` (literal-shared by convention; if
        /// you change one, change the other in the same commit).
        const RUST_KEYWORDS: &[&str] = &[
            "as", "async", "await", "box", "break", "const", "continue", "crate", "do", "dyn",
            "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
            "macro", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
            "struct", "super", "trait", "true", "type", "union", "unsafe", "use", "where", "while",
            "yield", "abstract", "become", "final", "override", "priv", "try",
        ];

        // --- Strategies (kept literally identical to macro side; see
        //     antigen-macros/src/parse.rs::parser_props). ---

        fn valid_kebab() -> impl Strategy<Value = String> {
            proptest::collection::vec(
                (
                    proptest::char::range('a', 'z'),
                    proptest::collection::vec(
                        prop_oneof![
                            proptest::char::range('a', 'z'),
                            proptest::char::range('0', '9'),
                        ],
                        0..8usize,
                    ),
                )
                    .prop_map(|(first, rest)| {
                        let mut s = String::with_capacity(rest.len() + 1);
                        s.push(first);
                        for c in rest {
                            s.push(c);
                        }
                        s
                    }),
                1..5usize,
            )
            .prop_map(|segments| segments.join("-"))
        }

        fn valid_text(max_len: usize) -> impl Strategy<Value = String> {
            proptest::collection::vec(
                prop_oneof![
                    proptest::char::range(' ', '~').prop_filter("excluded chars", |c| {
                        *c != '\\' && *c != '"' && *c != '\0'
                    }),
                ],
                1..=max_len,
            )
            .prop_map(|chars| chars.into_iter().collect())
        }

        fn lit(s: &str) -> String {
            format!("{s:?}")
        }

        fn render_antigen_body(
            name: &str,
            fingerprint: &str,
            family: Option<&str>,
            summary: Option<&str>,
        ) -> String {
            let mut parts = vec![
                format!("name = {}", lit(name)),
                format!("fingerprint = {}", lit(fingerprint)),
            ];
            if let Some(f) = family {
                parts.push(format!("family = {}", lit(f)));
            }
            if let Some(s) = summary {
                parts.push(format!("summary = {}", lit(s)));
            }
            parts.join(", ")
        }

        proptest! {
            // I1 — equivalence-on-intersection: any input the macro side
            // accepts (i.e., any input render_antigen_body produces from
            // valid_kebab + valid_text), the scan side accepts and produces
            // matching semantic content. The macro-side counterpart asserts
            // its own acceptance + identical extraction; together they lock
            // cross-parser drift.
            #[test]
            fn scan_parser_round_trip_on_macro_inputs(
                name in valid_kebab(),
                fingerprint in valid_text(64),
                family in proptest::option::of(valid_text(32)),
                summary in proptest::option::of(valid_text(64)),
            ) {
                let body = render_antigen_body(&name, &fingerprint, family.as_deref(), summary.as_deref());
                let tokens: TokenStream = body.parse().expect("body must tokenize");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan must accept macro-acceptable input");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
                prop_assert_eq!(args.family.as_deref(), family.as_deref());
                prop_assert_eq!(args.summary.as_deref(), summary.as_deref());
            }

            // I2 — order-invariance: scan side, like macro side, must not
            // depend on field order.
            #[test]
            fn scan_parser_order_invariant(
                name in valid_kebab(),
                fingerprint in valid_text(48),
                family in valid_text(24),
                summary in valid_text(48),
            ) {
                let canonical = format!(
                    "name = {}, fingerprint = {}, family = {}, summary = {}",
                    lit(&name), lit(&fingerprint), lit(&family), lit(&summary),
                );
                let reversed = format!(
                    "summary = {}, family = {}, fingerprint = {}, name = {}",
                    lit(&summary), lit(&family), lit(&fingerprint), lit(&name),
                );
                let a: ScanAntigenArgs = syn::parse2(canonical.parse::<TokenStream>().unwrap()).unwrap();
                let b: ScanAntigenArgs = syn::parse2(reversed.parse::<TokenStream>().unwrap()).unwrap();
                prop_assert_eq!(&a.name, &b.name);
                prop_assert_eq!(&a.fingerprint, &b.fingerprint);
                prop_assert_eq!(&a.family, &b.family);
                prop_assert_eq!(&a.summary, &b.summary);
            }

            // I3a (asymmetry) — scan tolerates unknown fields. For ANY
            // valid base input plus an arbitrary unknown field, scan still
            // parses and recovers name/fingerprint correctly.
            #[test]
            fn scan_parser_tolerates_arbitrary_unknown_field(
                name in valid_kebab(),
                fingerprint in valid_text(32),
                unknown in "[a-z][a-z_]{2,12}".prop_filter(
                    "must not collide with known fields or Rust keywords",
                    |s| {
                        !matches!(s.as_str(), "name" | "fingerprint" | "family" | "summary" | "references")
                            && !RUST_KEYWORDS.contains(&s.as_str())
                    },
                ),
                unknown_val in valid_text(16),
            ) {
                let body = format!(
                    "name = {}, fingerprint = {}, {} = {}",
                    lit(&name), lit(&fingerprint), unknown, lit(&unknown_val),
                );
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan tolerates unknown fields");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
            }

            // I3b (asymmetry) — scan tolerates missing required fields:
            // an input with only `name` (or only `fingerprint`) parses,
            // with the other field as `None` / empty. Macro side errors.
            #[test]
            fn scan_parser_tolerates_missing_fingerprint(
                name in valid_kebab(),
            ) {
                let body = format!("name = {}", lit(&name));
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan tolerates missing fingerprint");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint, None);
            }

            // I4 — references field is consumed silently (not stored in
            // the scan output today, but must not error). Macro side
            // stores into Vec<String>.
            #[test]
            fn scan_parser_consumes_references_array(
                name in valid_kebab(),
                fingerprint in valid_text(32),
                refs in proptest::collection::vec(valid_text(24), 0..6usize),
            ) {
                let refs_lit: Vec<String> = refs.iter().map(|s| lit(s)).collect();
                let body = format!(
                    "name = {}, fingerprint = {}, references = [{}]",
                    lit(&name), lit(&fingerprint), refs_lit.join(", "),
                );
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanAntigenArgs>(tokens).expect("scan parses references");
                prop_assert_eq!(&args.name, &name);
                prop_assert_eq!(args.fingerprint.as_deref(), Some(fingerprint.as_str()));
            }

            // I5 — ScanImmuneArgs: any (path, witness-path) parses and
            // exposes the last segment as antigen_type. Mirrors the
            // macro-side P7 property.
            //
            // Identifier strategies skip Rust keywords: even though our
            // regex generates legal-looking strings like "as" or "fn",
            // syn rejects them as path segments. Filtering the keyword
            // set out is the by-construction property — "valid path
            // segments parse" — rather than the false stronger one
            // "any [a-z][a-z_0-9]* parses".
            #[test]
            fn scan_immune_extracts_last_path_segment(
                antigen in "[A-Z][A-Za-z0-9]{0,16}",
                witness_segments in proptest::collection::vec(
                    "[a-z][a-z_0-9]{0,8}".prop_filter(
                        "must not be a Rust keyword",
                        |s| !RUST_KEYWORDS.contains(&s.as_str()),
                    ),
                    1..4usize,
                ),
            ) {
                let witness = witness_segments.join("::");
                let body = format!("{antigen}, witness = {witness}");
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanImmuneArgs>(tokens).expect("body parses");
                prop_assert_eq!(args.antigen_type.as_str(), antigen.as_str());
                // witness_segments.last() is the trailing identifier; the
                // scan parser canonicalises whitespace via quote::ToTokens,
                // so the rendered witness contains all segments separated
                // by " :: ".
                let last = witness_segments.last().unwrap();
                prop_assert!(args.witness.contains(last.as_str()),
                    "rendered witness {:?} should contain trailing segment {:?}", args.witness, last);
            }

            // I6 — ScanImmuneArgs: a qualified-path antigen extracts only
            // the last segment as the antigen_type (the matching surface
            // antigen scan/audit reasons against). This is a regression
            // anchor for ATK-A2-001 — the path-split corruption that the
            // adversarial pass surfaced.
            #[test]
            fn scan_immune_qualified_antigen_path_extracts_last_segment(
                module_segs in proptest::collection::vec(
                    "[a-z][a-z_0-9]{0,6}".prop_filter(
                        "must not be a Rust keyword",
                        |s| !RUST_KEYWORDS.contains(&s.as_str()),
                    ),
                    1..3usize,
                ),
                antigen in "[A-Z][A-Za-z0-9]{0,12}",
                witness in "[a-z][a-z_0-9]{0,12}".prop_filter(
                    "must not be a Rust keyword",
                    |s| !RUST_KEYWORDS.contains(&s.as_str()),
                ),
            ) {
                let qualified = format!("{}::{}", module_segs.join("::"), antigen);
                let body = format!("{qualified}, witness = {witness}");
                let tokens: TokenStream = body.parse().expect("body tokenizes");
                let args = syn::parse2::<ScanImmuneArgs>(tokens).expect("body parses");
                prop_assert_eq!(args.antigen_type.as_str(), antigen.as_str(),
                    "qualified antigen path {:?} must yield bare last-segment antigen_type", qualified);
            }
        }
    }

    // ========================================================================
    // Recurrent-Emergence Family scan-side parsing (ADR-024 §Family 2)
    // ========================================================================

    #[test]
    fn scan_recurrent_itch_captures_name_and_description() {
        let tokens: proc_macro2::TokenStream =
            r#"name = "drop-rhyme", description = "noticed Drop panics rhyme with unwrap-in-cleanup""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.name.as_deref(), Some("drop-rhyme"));
        assert!(args.description.as_deref().unwrap().contains("Drop panics"));
    }

    #[test]
    fn scan_recurrent_anchor_captures_positional_antigen_and_instances() {
        let tokens: proc_macro2::TokenStream = r#"MsrvCreep, instances = 3, since = "v0.1.0", rationale = "MSRV crept thrice across major bumps""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type.as_deref(), Some("MsrvCreep"));
        assert_eq!(args.instances, Some(3));
        assert_eq!(args.since.as_deref(), Some("v0.1.0"));
    }

    #[test]
    fn scan_recurrent_anchor_extracts_qualified_antigen_last_segment() {
        let tokens: proc_macro2::TokenStream = r#"antigen = crate::antigens::MsrvCreep, instances = 2, since = "v1", rationale = "twenty-char rationale text""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type.as_deref(), Some("MsrvCreep"));
    }

    #[test]
    fn scan_recurrent_crystallize_captures_from_itches_idents() {
        let tokens: proc_macro2::TokenStream =
            r#"name = "x", from_itches = [DropPanicItch, CleanupUnwrapItch], summary = "crystallized from two""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.from_itches, vec!["DropPanicItch", "CleanupUnwrapItch"]);
        // `summary` maps to the shared `description` capture.
        assert!(args
            .description
            .as_deref()
            .unwrap()
            .contains("crystallized"));
    }

    #[test]
    fn scan_recurrent_chronic_captures_managed_by() {
        let tokens: proc_macro2::TokenStream =
            r#"FlakeyStep, since = "v0.2.0", managed_by = "ci-team""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.antigen_type.as_deref(), Some("FlakeyStep"));
        assert_eq!(args.managed_by.as_deref(), Some("ci-team"));
    }

    #[test]
    fn scan_recurrent_strand_captures_anchored_by() {
        let tokens: proc_macro2::TokenStream = r#"name = "vcs-thread", anchored_by = [ForcePushItch, SquashItch], description = "history-loss thread""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.anchored_by, vec!["ForcePushItch", "SquashItch"]);
    }

    #[test]
    fn scan_recurrent_tolerates_unknown_and_missing_fields() {
        // Scan is recall-tuned (ADR-010): unknown fields consumed, missing
        // required fields tolerated; audit handles required-field validation.
        let tokens: proc_macro2::TokenStream =
            r#"name = "x", threshold = "5", bogus_future_field = "ignored""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.name.as_deref(), Some("x"));
    }

    #[test]
    fn scan_recurrent_saturate_captures_contributing_to() {
        let tokens: proc_macro2::TokenStream =
            r#"description = "evidence accumulating", contributing_to = "msrv-creep-anchor""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanRecurrentArgs>(tokens).unwrap();
        assert_eq!(args.contributing_to.as_deref(), Some("msrv-creep-anchor"));
    }

    // ========================================================================
    // Mucosal Boundary Family scan-side parsing (ADR-027 + Amendment 1)
    // ========================================================================

    #[test]
    fn scan_mucosal_captures_kind_and_rationale() {
        let tokens: proc_macro2::TokenStream =
            r#"kind = MucosalKind::UserInput, rationale = "public form; sanitized at render""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanMucosalArgs>(tokens).unwrap();
        assert_eq!(args.boundary_kind.as_deref(), Some("UserInput"));
        assert!(args.rationale.as_deref().unwrap().contains("sanitized"));
    }

    #[test]
    fn scan_mucosal_delegate_captures_boundary_and_handled_by_last_segment() {
        let tokens: proc_macro2::TokenStream =
            r#"boundary = MucosalKind::UserInput, handled_by = crate::sanitize::user_input, rationale = "delegated to central sanitizer""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanMucosalArgs>(tokens).unwrap();
        assert_eq!(args.boundary_kind.as_deref(), Some("UserInput"));
        // handled_by path → final segment.
        assert_eq!(args.handled_by.as_deref(), Some("user_input"));
    }

    #[test]
    fn scan_mucosal_tolerant_captures_accepts_reviewed_until() {
        let tokens: proc_macro2::TokenStream = r#"kind = MucosalKind::ApiRequest, rationale = "internal admin endpoint behind VPN; trusted network", accepts = "admin form posts", reviewed_by = "security-team", until = "2026-12-31""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ScanMucosalArgs>(tokens).unwrap();
        assert_eq!(args.boundary_kind.as_deref(), Some("ApiRequest"));
        assert_eq!(args.accepts.as_deref(), Some("admin form posts"));
        assert_eq!(args.reviewed_by.as_deref(), Some("security-team"));
        assert_eq!(args.until.as_deref(), Some("2026-12-31"));
    }

    #[test]
    fn scan_mucosal_tolerates_unknown_fields() {
        let tokens: proc_macro2::TokenStream =
            r#"kind = MucosalKind::Iframe, rationale = "embedded trusted widget context", future_field = "ignored""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ScanMucosalArgs>(tokens).unwrap();
        assert_eq!(args.boundary_kind.as_deref(), Some("Iframe"));
    }

    // ========================================================================
    // A3: lineage edge cycle detection (ATK-A3-002)
    // ========================================================================

    fn edge(child: &str, parent: &str) -> LineageEdge {
        LineageEdge {
            child: child.to_string(),
            parent: parent.to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            parent_canonical_path: None,
            child_canonical_path: None,
        }
    }

    #[test]
    fn lineage_no_edges_no_failures() {
        let failures = detect_lineage_failures(&[], MAX_LINEAGE_DEPTH);
        assert!(failures.is_empty());
    }

    #[test]
    fn lineage_acyclic_chain_no_failures() {
        // C -> B -> A (deepest first declared)
        let edges = vec![edge("C", "B"), edge("B", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert!(
            failures.is_empty(),
            "acyclic chain must produce no failures, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_self_loop_detected() {
        // A -> A
        let edges = vec![edge("A", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "self-loop must report exactly one failure"
        );
        assert!(
            failures[0].error.contains("cycle"),
            "self-loop error must mention cycle, got: {}",
            failures[0].error
        );
        assert!(
            failures[0].error.contains("A -> A"),
            "self-loop error must contain chain `A -> A`, got: {}",
            failures[0].error
        );
    }

    #[test]
    fn lineage_two_node_cycle_detected() {
        // A -> B -> A
        let edges = vec![edge("A", "B"), edge("B", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "2-cycle must report one failure, got: {failures:?}"
        );
        let err = &failures[0].error;
        assert!(err.contains("cycle"), "must mention cycle, got: {err}");
        assert!(
            err.contains("A -> B -> A") || err.contains("B -> A -> B"),
            "must contain full cycle chain, got: {err}"
        );
    }

    #[test]
    fn lineage_three_node_cycle_detected() {
        // A -> B -> C -> A
        let edges = vec![edge("A", "B"), edge("B", "C"), edge("C", "A")];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "3-cycle must report exactly one failure, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_cycle_dedup_across_entry_points() {
        // A -> B -> C -> A (same cycle reachable from B and from C)
        // Adding extra non-cyclic edges should still produce one failure.
        let edges = vec![
            edge("A", "B"),
            edge("B", "C"),
            edge("C", "A"),
            edge("D", "B"), // D enters the cycle through B
            edge("E", "C"), // E enters the cycle through C
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            1,
            "same cycle entered from multiple roots must dedup, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_two_disjoint_cycles_both_reported() {
        // A -> B -> A (cycle 1) and X -> Y -> X (cycle 2)
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("X", "Y"),
            edge("Y", "X"),
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            2,
            "two disjoint cycles must produce two failures, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_diamond_no_cycle() {
        // A descended_from B, A descended_from C, B descended_from D, C descended_from D
        // (a DAG diamond — no cycle even though D is reached via two paths)
        let edges = vec![
            edge("A", "B"),
            edge("A", "C"),
            edge("B", "D"),
            edge("C", "D"),
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert!(
            failures.is_empty(),
            "DAG diamond must not be reported as cycle, got: {failures:?}"
        );
    }

    #[test]
    fn lineage_depth_limit_fires_on_long_linear_chain() {
        // 10-node linear chain with a depth limit of 5 fires depth-exceeded.
        let edges: Vec<LineageEdge> = (0..10)
            .map(|i| edge(&format!("N{i}"), &format!("N{}", i + 1)))
            .collect();
        let failures = detect_lineage_failures(&edges, 5);
        assert!(
            failures.iter().any(|f| f.error.contains("maximum depth")),
            "depth limit must fire on long linear chain, got: {failures:?}"
        );
    }

    // ATK-LINEAGE-BOUNDARY: characterize exact boundary behavior of the depth limit.
    //
    // MAX_LINEAGE_DEPTH = 64. The check is `path.len() > max_depth`, where
    // path.len() is the number of NODES (not edges). So:
    //   - N+1 nodes (N edges): path.len() = N+1. Fires when N+1 > 64, i.e. N >= 64.
    //   - A chain of exactly 64 edges (65 nodes) fires the limit.
    //   - A chain of exactly 63 edges (64 nodes) is accepted.
    //
    // The constant is named MAX_LINEAGE_DEPTH but the effective limit is
    // MAX_LINEAGE_DEPTH-1 edges — naming and semantics are off by one.
    //
    // This test pins the actual behavior. If this test passes, the boundary
    // is as described. If it fails (both assertions fail together), the
    // implementation changed.
    #[test]
    fn lineage_depth_limit_boundary_exactly_at_max() {
        // Chain of MAX_LINEAGE_DEPTH nodes (MAX_LINEAGE_DEPTH-1 edges) — ACCEPTED.
        let accepted_edges: Vec<LineageEdge> = (0..MAX_LINEAGE_DEPTH - 1)
            .map(|i| edge(&format!("N{i}"), &format!("N{}", i + 1)))
            .collect();
        let failures = detect_lineage_failures(&accepted_edges, MAX_LINEAGE_DEPTH);
        assert!(
            failures.is_empty(),
            "ATK-LINEAGE-BOUNDARY: a chain of {}-1={} edges ({} nodes) must be \
             accepted by the depth limit (path.len()={} is NOT > {}). \
             Got failures: {failures:?}",
            MAX_LINEAGE_DEPTH,
            MAX_LINEAGE_DEPTH - 1,
            MAX_LINEAGE_DEPTH,
            MAX_LINEAGE_DEPTH,
            MAX_LINEAGE_DEPTH,
        );

        // Chain of MAX_LINEAGE_DEPTH edges (MAX_LINEAGE_DEPTH+1 nodes) — REJECTED.
        let rejected_edges: Vec<LineageEdge> = (0..MAX_LINEAGE_DEPTH)
            .map(|i| edge(&format!("M{i}"), &format!("M{}", i + 1)))
            .collect();
        let failures = detect_lineage_failures(&rejected_edges, MAX_LINEAGE_DEPTH);
        assert!(
            failures.iter().any(|f| f.error.contains("maximum depth")),
            "ATK-LINEAGE-BOUNDARY: a chain of {} edges ({} nodes) must fire the \
             depth limit (path.len()={} IS > {}). \
             Got: {failures:?}",
            MAX_LINEAGE_DEPTH,
            MAX_LINEAGE_DEPTH + 1,
            MAX_LINEAGE_DEPTH + 1,
            MAX_LINEAGE_DEPTH,
        );
    }

    #[test]
    fn lineage_canonicalise_cycle_basic() {
        // Three rotations of the same cycle produce the same canonical form.
        let a = canonicalise_cycle(&["A", "B", "C"]);
        let b = canonicalise_cycle(&["B", "C", "A"]);
        let c = canonicalise_cycle(&["C", "A", "B"]);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn lineage_canonicalise_cycle_distinguishes_distinct() {
        // Different cycles canonicalise to different forms.
        let a = canonicalise_cycle(&["A", "B"]);
        let b = canonicalise_cycle(&["A", "C"]);
        assert_ne!(a, b);
    }

    // ========================================================================
    // A3: orphaned lineage edges query (ATK-A3-003)
    // ========================================================================

    fn antigen_decl(type_name: &str) -> AntigenDeclaration {
        AntigenDeclaration {
            name: type_name.to_lowercase(),
            type_name: type_name.to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            family: None,
            summary: None,
            fingerprint: None,
            canonical_path: None,
            category: Vec::new(),
        }
    }

    #[test]
    fn orphaned_lineage_edges_empty_report_returns_empty() {
        let report = ScanReport::default();
        assert!(report.orphaned_lineage_edges().is_empty());
    }

    #[test]
    fn orphaned_lineage_edges_known_parent_not_orphan() {
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Parent"));
        report.antigens.push(antigen_decl("Child"));
        report.lineage_edges.push(edge("Child", "Parent"));
        assert!(report.orphaned_lineage_edges().is_empty());
    }

    #[test]
    fn orphaned_lineage_edges_unknown_parent_is_orphan() {
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Child"));
        report.lineage_edges.push(edge("Child", "MissingParent"));
        let orphans = report.orphaned_lineage_edges();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].parent, "MissingParent");
    }

    // ========================================================================
    // Multi-crate (member-aware) scan — Layer 1 unit coverage.
    //
    // The integration coverage (real workspace enumeration + per-member
    // stamping + cross-member lineage end-to-end) lives in
    // antigen/tests/atk_multi_crate_scan.rs. These unit tests pin the
    // structural pieces in isolation: canonical-path formatting, the merge
    // union, and — the heart of cross-crate `#[descended_from]` —
    // `resolve_cross_member_lineage_parents`.
    // ========================================================================

    /// `antigen_decl` variant that stamps a member canonical path, so a test
    /// can model a declaration living in a specific workspace member.
    fn antigen_decl_in(type_name: &str, crate_id: &str) -> AntigenDeclaration {
        let mut a = antigen_decl(type_name);
        a.canonical_path = Some(crate_id.to_string());
        a
    }

    #[test]
    fn member_root_canonical_path_is_name_at_version() {
        let m = WorkspaceMemberRoot {
            package_name: "antigen-fingerprint".to_string(),
            version: "0.3.0-alpha.1".to_string(),
            crate_root: PathBuf::from("/ws/antigen-fingerprint"),
        };
        assert_eq!(m.canonical_path(), "antigen-fingerprint@0.3.0-alpha.1");
    }

    #[test]
    fn merge_unions_all_record_vectors_and_sums_counts() {
        let mut a = ScanReport {
            files_scanned: 3,
            ..ScanReport::default()
        };
        a.antigens.push(antigen_decl_in("AlphaA", "crate-a@1.0.0"));
        a.lineage_edges.push(edge("ChildA", "AlphaA"));

        let mut b = ScanReport {
            files_scanned: 5,
            ..ScanReport::default()
        };
        b.antigens.push(antigen_decl_in("BetaB", "crate-b@1.0.0"));
        b.parse_failures.push(ParseFailure {
            file: PathBuf::from("b.rs"),
            error: "boom".to_string(),
        });

        a.merge(b);
        assert_eq!(a.antigens.len(), 2, "antigen vectors union");
        assert_eq!(a.lineage_edges.len(), 1, "edges carry over");
        assert_eq!(a.parse_failures.len(), 1, "parse failures union");
        assert_eq!(a.files_scanned, 8, "file counts sum");
    }

    #[test]
    fn cross_member_parent_reresolves_to_declaring_member() {
        // Parent `Shared` lives in crate-b; Child bears `#[descended_from(Shared)]`
        // in crate-a. Per-member stamping (modeled here) leaves the edge's
        // parent endpoint pointing at crate-a (the child's member). The
        // re-resolution pass must re-stamp it to crate-b so the propagation
        // walk's `(parent, canonical_path)` lookup resolves.
        let mut report = ScanReport::default();
        report
            .antigens
            .push(antigen_decl_in("Child", "crate-a@1.0.0"));
        report
            .antigens
            .push(antigen_decl_in("Shared", "crate-b@1.0.0"));
        let mut e = edge("Child", "Shared");
        e.child_canonical_path = Some("crate-a@1.0.0".to_string());
        e.parent_canonical_path = Some("crate-a@1.0.0".to_string()); // wrong on purpose
        report.lineage_edges.push(e);

        resolve_cross_member_lineage_parents(&mut report);

        assert_eq!(
            report.lineage_edges[0].parent_canonical_path.as_deref(),
            Some("crate-b@1.0.0"),
            "parent endpoint must re-resolve to the member that declares `Shared`"
        );
        assert!(
            report
                .parse_failures
                .iter()
                .all(|f| !f.error.contains("ambiguous")),
            "unambiguous cross-member parent must not produce an ambiguity diagnostic"
        );
        // And the edge is no longer orphaned — the propagation walk can use it.
        assert!(
            report.orphaned_lineage_edges().is_empty(),
            "re-resolved cross-member edge must not be orphaned"
        );
    }

    #[test]
    fn cross_member_parent_ambiguous_name_collision_is_diagnosed_not_guessed() {
        // `Dup` is declared in TWO members. A `#[descended_from(Dup)]` cannot be
        // resolved to one member without guessing; the pass must leave the parent
        // endpoint conservative AND emit an explicit ambiguity diagnostic
        // (ADR-004 implicit-to-explicit: surface the collision, don't silently
        // pick a member).
        let mut report = ScanReport::default();
        report
            .antigens
            .push(antigen_decl_in("Dup", "crate-a@1.0.0"));
        report
            .antigens
            .push(antigen_decl_in("Dup", "crate-b@1.0.0"));
        report
            .antigens
            .push(antigen_decl_in("Child", "crate-c@1.0.0"));
        let mut e = edge("Child", "Dup");
        e.child_canonical_path = Some("crate-c@1.0.0".to_string());
        e.parent_canonical_path = Some("crate-c@1.0.0".to_string());
        report.lineage_edges.push(e);

        resolve_cross_member_lineage_parents(&mut report);

        // Parent endpoint left as the conservative intra-member value.
        assert_eq!(
            report.lineage_edges[0].parent_canonical_path.as_deref(),
            Some("crate-c@1.0.0"),
            "ambiguous parent must NOT be silently re-stamped to one member"
        );
        let ambiguity: Vec<_> = report
            .parse_failures
            .iter()
            .filter(|f| f.error.contains("ambiguous across the workspace"))
            .collect();
        assert_eq!(
            ambiguity.len(),
            1,
            "ambiguous cross-member name must produce exactly one diagnostic; got: {:?}",
            report.parse_failures
        );
        assert!(
            ambiguity[0].error.contains("crate-a@1.0.0")
                && ambiguity[0].error.contains("crate-b@1.0.0"),
            "ambiguity diagnostic must name both colliding members"
        );
    }

    #[test]
    fn cross_member_parent_unknown_name_left_orphan() {
        // Parent declared in NO member — the edge must stay unchanged and
        // surface as an orphan downstream (existing channel discipline).
        let mut report = ScanReport::default();
        report
            .antigens
            .push(antigen_decl_in("Child", "crate-a@1.0.0"));
        let mut e = edge("Child", "Ghost");
        e.child_canonical_path = Some("crate-a@1.0.0".to_string());
        e.parent_canonical_path = Some("crate-a@1.0.0".to_string());
        report.lineage_edges.push(e);

        resolve_cross_member_lineage_parents(&mut report);

        assert_eq!(
            report.lineage_edges[0].parent_canonical_path.as_deref(),
            Some("crate-a@1.0.0"),
            "unknown parent edge is left unchanged"
        );
        assert!(
            report.parse_failures.is_empty(),
            "unknown parent is not an ambiguity — no diagnostic from this pass"
        );
        let orphans = report.orphaned_lineage_edges();
        assert_eq!(orphans.len(), 1, "unknown parent surfaces as orphan");
        assert_eq!(orphans[0].parent, "Ghost");
    }

    // ------------------------------------------------------------------------
    // Layer-2 addresses() resolution unit coverage
    // (`resolve_cross_member_addresses` — the ADR-017-Amd1 address-pass sibling
    // of `resolve_cross_member_lineage_parents`).
    //
    // Integration coverage lives in antigen/tests/atk_multi_crate_scan.rs
    // (cross_member_presents_resolves_to_declaring_member,
    // cross_member_defended_by_resolves_and_addresses). These unit tests pin the
    // structural corner-cases in isolation.
    // ------------------------------------------------------------------------

    /// Helper: build a minimal `Presentation` with a given antigen type and
    /// canonical path — the minimum fields `restamp_family!` reads and writes.
    fn presentation_in(antigen_type: &str, crate_id: &str) -> Presentation {
        Presentation {
            antigen_type: antigen_type.to_string(),
            file: PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: ItemTarget::Fn(format!("site_in_{}", crate_id.replace('@', "_"))),
            match_kind: MatchKind::ExplicitMarker,
            canonical_path: Some(crate_id.to_string()),
            inherited_from: None,
            structural_fingerprint: String::new(),
            requires_predicate: None,
            proof: None,
        }
    }

    #[test]
    fn cross_member_addresses_ambiguous_name_is_diagnosed_not_guessed() {
        // Two members both declare the same antigen type name (`Shared`).
        // A `#[presents(Shared)]` in a third member should NOT be silently
        // re-stamped to either declaring member — the resolution is ambiguous.
        // The pass must:
        //   (a) leave the record keyed to its own member (conservative assumption),
        //   (b) emit exactly one ParseFailure naming the collision.
        //
        // This mirrors the lineage-parent ambiguity test
        // (`cross_member_parent_ambiguous_name_collision_is_diagnosed_not_guessed`)
        // for the addresses()-resolution pass.
        let mut report = ScanReport::default();
        // Two antigens with the same bare name in different members.
        report
            .antigens
            .push(antigen_decl_in("Shared", "crate-a@1.0.0"));
        report
            .antigens
            .push(antigen_decl_in("Shared", "crate-b@1.0.0"));
        // A presentation in a third member referencing the ambiguous name.
        report
            .presentations
            .push(presentation_in("Shared", "crate-c@1.0.0"));

        resolve_cross_member_addresses(&mut report);

        // (a) The presentation must stay keyed to its own member — NOT silently
        // guessed to crate-a or crate-b.
        assert_eq!(
            report.presentations[0].canonical_path.as_deref(),
            Some("crate-c@1.0.0"),
            "ambiguous addresses() must NOT re-stamp to either declaring member; \
             got {:?}",
            report.presentations[0].canonical_path
        );
        // (b) Exactly one ParseFailure naming the collision.
        let ambiguity: Vec<_> = report
            .parse_failures
            .iter()
            .filter(|f| f.error.contains("ambiguous across the workspace"))
            .collect();
        assert_eq!(
            ambiguity.len(),
            1,
            "ambiguous same-name cross-member addresses() must produce exactly one \
             diagnostic; got: {:?}",
            report.parse_failures
        );
        assert!(
            ambiguity[0].error.contains("crate-a@1.0.0")
                && ambiguity[0].error.contains("crate-b@1.0.0"),
            "ambiguity diagnostic must name both colliding members; got: {}",
            ambiguity[0].error
        );
    }

    #[test]
    fn cross_member_addresses_unknown_antigen_is_left_unchanged() {
        // An antigen type declared in NO member — the reference stays keyed to
        // its own member. No ParseFailure (unknown is out-of-frame downstream,
        // not an ambiguity — parallel to `cross_member_parent_unknown_name_left_orphan`).
        let mut report = ScanReport::default();
        // Only one antigen in the workspace, and it's not "Ghost".
        report
            .antigens
            .push(antigen_decl_in("Other", "crate-a@1.0.0"));
        report
            .presentations
            .push(presentation_in("Ghost", "crate-b@1.0.0"));

        resolve_cross_member_addresses(&mut report);

        assert_eq!(
            report.presentations[0].canonical_path.as_deref(),
            Some("crate-b@1.0.0"),
            "unknown antigen must leave the presentation keyed to its own member"
        );
        assert!(
            report.parse_failures.is_empty(),
            "unknown antigen is not an ambiguity — no diagnostic from this pass; \
             got: {:?}",
            report.parse_failures
        );
    }

    // ------------------------------------------------------------------------
    // ScanCoverage — the ignorance-frontier substrate.
    // ------------------------------------------------------------------------

    fn coverage(enumerated: &[&str], scanned: &[&str]) -> ScanCoverage {
        ScanCoverage {
            enumerated_members: enumerated.iter().map(|s| (*s).to_string()).collect(),
            scanned_members: scanned.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn coverage_complete_when_all_enumerated_are_scanned() {
        let c = coverage(&["a@1", "b@1"], &["a@1", "b@1"]);
        assert!(
            c.is_complete(),
            "every enumerated member scanned ⇒ complete"
        );
        assert!(
            c.unscanned_members().is_empty(),
            "complete coverage has an empty ignorance frontier"
        );
    }

    #[test]
    fn coverage_unscanned_member_is_the_ignorance_frontier() {
        // `c@1` is enumerated but never scanned — its sites are UNSEEN.
        let c = coverage(&["a@1", "b@1", "c@1"], &["a@1", "b@1"]);
        assert!(
            !c.is_complete(),
            "an unscanned member ⇒ incomplete coverage"
        );
        assert_eq!(
            c.unscanned_members(),
            vec!["c@1"],
            "the frontier is exactly the enumerated-minus-scanned set"
        );
    }

    #[test]
    fn coverage_empty_workspace_is_vacuously_complete() {
        let c = coverage(&[], &[]);
        assert!(c.is_complete());
        assert!(c.unscanned_members().is_empty());
    }

    // ========================================================================
    // ATK-A3: adversarial edge cases.
    // Two are FAILING bug contracts (atk_a3_dup, atk_a3_orphan_child).
    // Two are PASSING positive-controls verifying dedup correctness.
    // ========================================================================

    #[test]
    fn atk_a3_dup_duplicate_lineage_edge_is_diagnosed_not_silent() {
        // ATK-A3-DUP / BUG-A3-001: two `#[descended_from(B)]` on the same
        // struct A produce two identical lineage edges. Without the dedup
        // pass, the DFS in `detect_lineage_failures` silently swallows the
        // duplicate (black-skip path). Per ADR-004 (implicit-to-explicit
        // elevation), the dedup pass surfaces collapsed duplicates as
        // explicit parse_failures.
        //
        // ADR-018 §"Implementation order in scan_workspace" ratifies that
        // edge-level dedup runs as a separate pass before cycle detection
        // AND the propagation walk. This test exercises the dedup
        // function directly; the integration is verified in
        // `scan_workspace` end-to-end via the BUG-A3-001 fixture (see
        // atk_a3_fractal_preview.rs).
        let edges = vec![
            edge("A", "B"),
            edge("A", "B"), // exact duplicate (same canonical_path = None)
        ];
        let (deduped, failures) = dedupe_lineage_edges(&edges);
        assert_eq!(
            deduped.len(),
            1,
            "duplicate edges must collapse to one in the deduped output"
        );
        assert!(
            !failures.is_empty(),
            "duplicate lineage edge (A->B twice) must produce at least one \
             diagnostic; got: {failures:?}"
        );
    }

    #[test]
    fn dedupe_distinguishes_edges_by_canonical_path() {
        // ADR-018 §"Edge-level dedup": dedup key is (child, parent,
        // child_canonical_path, parent_canonical_path). Same-name edges
        // with different canonical_paths are NOT duplicates (a workspace
        // depending on `foo@1.0.0::P` and `foo@2.0.0::P` legitimately has
        // both edges pointing at different identities).
        let edge_v1 = LineageEdge {
            child: "Child".to_string(),
            parent: "Parent".to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            parent_canonical_path: Some("foo@1.0.0".to_string()),
            child_canonical_path: None,
        };
        let edge_v2 = LineageEdge {
            child: "Child".to_string(),
            parent: "Parent".to_string(),
            file: PathBuf::from("test.rs"),
            line: 2,
            parent_canonical_path: Some("foo@2.0.0".to_string()),
            child_canonical_path: None,
        };
        let (deduped, failures) = dedupe_lineage_edges(&[edge_v1, edge_v2]);
        assert_eq!(
            deduped.len(),
            2,
            "edges differing in parent_canonical_path are distinct identities, \
             not duplicates"
        );
        assert!(
            failures.is_empty(),
            "no dedup failure should fire for cross-version edges; got: {failures:?}"
        );
    }

    #[test]
    fn atk_a3_shared_two_cycles_sharing_a_node_both_reported() {
        // ATK-A3-SHARED: A->B->A forms cycle 1; A->C->A forms cycle 2.
        // Node A participates in both. The canonicalise_cycle dedup must NOT
        // suppress cycle 2 because it shares node A with cycle 1 — the cycles
        // are structurally distinct ({A,B} vs {A,C}).
        //
        // This is a positive-control test verifying the dedup logic does not
        // over-suppress. Expected: 2 failures.
        let edges = vec![
            edge("A", "B"),
            edge("B", "A"),
            edge("A", "C"),
            edge("C", "A"),
        ];
        let failures = detect_lineage_failures(&edges, MAX_LINEAGE_DEPTH);
        assert_eq!(
            failures.len(),
            2,
            "two distinct cycles sharing node A must both be reported; \
             got: {failures:?}"
        );
        let texts: Vec<&str> = failures.iter().map(|f| f.error.as_str()).collect();
        assert!(
            texts.iter().any(|t| t.contains('A') && t.contains('B')),
            "one failure must name the A-B cycle; texts: {texts:?}"
        );
        assert!(
            texts.iter().any(|t| t.contains('A') && t.contains('C')),
            "one failure must name the A-C cycle; texts: {texts:?}"
        );
    }

    #[test]
    fn atk_a3_combined_cycle_and_depth_exceeded_both_reported() {
        // ATK-A3-COMBINED: a graph with BOTH a cycle (X->Y->X) and a long chain
        // (N0->...->N5) exceeding a small depth limit. Both failure types must
        // appear — the pass must not short-circuit after the first failure type.
        //
        // Contract: at least one "cycle" failure AND at least one "maximum depth"
        // failure must be present.
        let depth = 3_usize;
        let mut edges = vec![edge("X", "Y"), edge("Y", "X")];
        for i in 0..=(depth + 2) {
            edges.push(edge(&format!("N{i}"), &format!("N{}", i + 1)));
        }
        let failures = detect_lineage_failures(&edges, depth);
        assert!(
            failures.iter().any(|f| f.error.contains("cycle")),
            "cycle X->Y->X must be detected even when long chain is also present; \
             all failures: {failures:?}"
        );
        assert!(
            failures.iter().any(|f| f.error.contains("maximum depth")),
            "depth limit must fire on long chain even when cycle is also present; \
             all failures: {failures:?}"
        );
    }

    #[test]
    fn atk_a3_orphan_child_without_antigen_declaration_is_surfaced() {
        // ATK-A3-ORPHAN-CHILD (adversarial BUG-A3-002): lineage edge where the
        // CHILD has no corresponding `#[antigen]` declaration. The user wrote
        // `#[descended_from(Parent)]` on a struct that is NOT itself an antigen.
        //
        // `orphaned_lineage_edges()` only checks if the PARENT is in the
        // known-antigens set; this is its dual case. A child-without-antigen
        // is structurally incoherent: it claims inheritance into the antigen
        // system without being a participant.
        //
        // Contract: this must be surfaced via SOME query channel —
        // `dangling_child_lineage_edges()` (the chosen channel),
        // `orphaned_lineage_edges()`, or `parse_failures`. Pathmaker chose
        // a separate `dangling_child_lineage_edges()` method (parallel to
        // `orphaned_lineage_edges`) because the channel separation
        // is structurally cleaner per ADR-006 (recognition-not-design).
        //
        // The ADR-018 propagation walk (D1.5) skips edges flagged by either
        // query method — both produce the same effect on inheritance
        // resolution.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Parent"));
        // OrphanChild is NOT in antigens — only in lineage_edges.
        report.lineage_edges.push(edge("OrphanChild", "Parent"));

        let orphans = report.orphaned_lineage_edges();
        let dangling = report.dangling_child_lineage_edges();
        assert!(
            !orphans.is_empty() || !dangling.is_empty() || !report.parse_failures.is_empty(),
            "lineage edge whose child has no #[antigen] declaration must be \
             surfaced via orphaned_lineage_edges, dangling_child_lineage_edges, or \
             parse_failures; got orphans: {orphans:?}, dangling: {dangling:?}"
        );

        // Specific assertion: pathmaker chose dangling_child_lineage_edges as the
        // channel. The orphan-channel must NOT also surface this case
        // (parent IS in antigens, so it's not a parent-orphan).
        assert!(
            orphans.is_empty(),
            "child-missing case must NOT appear in orphaned_lineage_edges \
             (that channel is for parent-missing); got: {orphans:?}"
        );
        assert_eq!(
            dangling.len(),
            1,
            "child-missing must appear in dangling_child_lineage_edges, exactly one"
        );
        assert_eq!(dangling[0].child, "OrphanChild");
        assert_eq!(dangling[0].parent, "Parent");
    }

    // ========================================================================
    // A3: stamp_canonical_path (ADR-017 Option A — caller stamps post-scan)
    // ========================================================================

    #[test]
    fn stamp_canonical_path_sets_none_to_some() {
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Foo"));
        report.lineage_edges.push(edge("Child", "Parent"));
        report.stamp_canonical_path("crate-a@1.0.0");
        assert_eq!(
            report.antigens[0].canonical_path.as_deref(),
            Some("crate-a@1.0.0"),
            "antigens with canonical_path: None must be stamped"
        );
        assert_eq!(
            report.lineage_edges[0].parent_canonical_path.as_deref(),
            Some("crate-a@1.0.0")
        );
        assert_eq!(
            report.lineage_edges[0].child_canonical_path.as_deref(),
            Some("crate-a@1.0.0")
        );
    }

    #[test]
    fn stamp_canonical_path_does_not_overwrite_some() {
        // ADR-017 Option A: stamp is non-overwriting. A record already
        // stamped with `Some(_)` (e.g., during a nested scan) must NOT
        // be silently re-keyed by a subsequent stamp call.
        let mut a = antigen_decl("Foo");
        a.canonical_path = Some("crate-a@1.0.0".to_string());
        let mut report = ScanReport::default();
        report.antigens.push(a);
        report.stamp_canonical_path("crate-b@2.0.0");
        assert_eq!(
            report.antigens[0].canonical_path.as_deref(),
            Some("crate-a@1.0.0"),
            "pre-stamped Some(_) must NOT be overwritten by a later stamp call"
        );
    }

    #[test]
    fn stamp_canonical_path_is_idempotent() {
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("Foo"));
        report.stamp_canonical_path("crate-a@1.0.0");
        let after_first = report.clone();
        report.stamp_canonical_path("crate-a@1.0.0");
        // Same stamp twice: identical output.
        assert_eq!(
            report.antigens[0].canonical_path, after_first.antigens[0].canonical_path,
            "stamping with same crate_id twice must be idempotent"
        );
    }

    // ========================================================================
    // ADR-029: #[defended_by] code-tier witness registration discovery
    // ========================================================================

    /// Parse `src` as a Rust file and run the scan visitor over it, returning
    /// the assembled report. In-module helper (`ScanVisitor` is private); the
    /// integration-test path is `scan_workspace` against fixture dirs.
    fn scan_source(src: &str) -> ScanReport {
        use syn::visit::Visit;
        let file = syn::parse_file(src).expect("test source parses");
        let mut report = ScanReport::default();
        let mut visitor = ScanVisitor::new(std::path::PathBuf::from("test.rs"), &mut report);
        visitor.visit_file(&file);
        report
    }

    #[test]
    fn scan_discovers_defended_by_registration() {
        // The genuinely-new ADR-029 primitive: a #[test] fn declares which
        // failure-class it defends. Scan records the registration; the verdict
        // (does it actually defend a #[presents] site?) is audit-time work.
        let report = scan_source(
            r"
            #[test]
            #[defended_by(ParallelStateTrackersDiverge)]
            fn bijection_audit_hints_const_matches_enum() {}
            ",
        );
        assert_eq!(
            report.defenses.len(),
            1,
            "exactly one #[defended_by] registration expected; got: {:?}",
            report.defenses
        );
        let d = &report.defenses[0];
        assert_eq!(d.antigen_type, "ParallelStateTrackersDiverge");
        assert_eq!(d.item_kind, "fn");
        // The recorded item_target is the WITNESS function, not a defended site
        // — the cross-reference is computed at audit time (ADR-029).
        assert_eq!(
            d.item_target,
            ItemTarget::Fn("bijection_audit_hints_const_matches_enum".to_string())
        );
    }

    #[test]
    fn scan_defended_by_accepts_qualified_path_uses_last_segment() {
        // Like #[presents], the antigen is recorded by its last path segment so
        // a qualified `crate::antigens::Foo` and a bare `Foo` register identically.
        let report = scan_source(
            r"
            #[test]
            #[defended_by(crate::antigens::DropPanicClass)]
            fn no_panic_in_drop() {}
            ",
        );
        assert_eq!(report.defenses.len(), 1);
        assert_eq!(report.defenses[0].antigen_type, "DropPanicClass");
    }

    #[test]
    fn scan_bare_defended_by_without_antigen_is_parse_failure_not_ghost() {
        // A bare #[defended_by] (no antigen) declares a witness for nothing.
        // Surface it as a parse failure rather than recording a ghost defense
        // with an empty antigen_type (ADR-005 sub-clause F: a registration
        // without a subject is not a registration).
        let report = scan_source(
            r"
            #[defended_by]
            fn witness_for_nothing() {}
            ",
        );
        assert!(
            report.defenses.is_empty(),
            "bare #[defended_by] must not record a ghost defense; got: {:?}",
            report.defenses
        );
        assert!(
            report
                .parse_failures
                .iter()
                .any(|f| f.error.contains("#[defended_by] requires an antigen type")),
            "expected a parse failure naming the missing antigen type; got: {:?}",
            report.parse_failures
        );
    }

    // ========================================================================
    // ADR-029 R5: #[presents] site-attached evidence (requires= / proof=)
    // ========================================================================

    #[test]
    fn scan_presents_captures_requires_predicate() {
        // ADR-029 R5: a substrate-tier predicate folds onto #[presents]; scan
        // must capture it (the substrate-witness migration target). Source-attr
        // is the primary discovery channel.
        let report = scan_source(
            r#"
            #[presents(UnpinnedDependency, requires = ratified_doc(path = "docs/x.md"))]
            fn add_dependency() {}
            "#,
        );
        assert_eq!(report.presentations.len(), 1);
        let p = &report.presentations[0];
        assert_eq!(p.antigen_type, "UnpinnedDependency");
        assert!(
            p.requires_predicate.is_some(),
            "the requires= predicate must be captured on the presents-site; got: {p:?}"
        );
        assert!(p.proof.is_none());
    }

    #[test]
    fn scan_presents_captures_proof_expression() {
        // ADR-029 R5: a phantom-tier proof folds onto #[presents]; scan captures
        // its token-string form (the audit recognizes the phantom shape).
        let report = scan_source(
            r"
            #[presents(DropPanicClass, proof = NonPanickingProof::<T>::verified)]
            fn make_droppable() {}
            ",
        );
        assert_eq!(report.presentations.len(), 1);
        let p = &report.presentations[0];
        assert_eq!(p.antigen_type, "DropPanicClass");
        assert!(
            p.proof
                .as_deref()
                .is_some_and(|s| s.contains("NonPanickingProof")),
            "the proof= expression must be captured on the presents-site; got: {p:?}"
        );
        assert!(p.requires_predicate.is_none());
    }

    #[test]
    fn scan_bare_presents_has_no_site_evidence() {
        // Back-compat: a plain #[presents(X)] carries no site-attached evidence.
        let report = scan_source(
            r"
            #[presents(PanickingInDrop)]
            fn might_panic() {}
            ",
        );
        assert_eq!(report.presentations.len(), 1);
        assert!(report.presentations[0].requires_predicate.is_none());
        assert!(report.presentations[0].proof.is_none());
    }

    // ========================================================================
    // ADR-009 Amd-1: fingerprint-omission silent behavior (ATK-ADR009-AMD1)
    //
    // Scout probe surface (2026-05-27 field notice 032da904): when an antigen
    // is declared without a fingerprint, the synthesis_pass skips it silently.
    // This is CORRECT behavior (verify-only antigens have no scan-surface per
    // ADR-009 Amd-1), but it creates an invisible failure mode for authors who
    // INTEND to write a scan-locatable antigen and accidentally omit the fingerprint.
    //
    // Three claims to verify:
    //   (a) zero-fingerprint antigen produces no FingerprintMatch presentations
    //   (b) no false-positive presentations from other fingerprints (silence is clean)
    //   (c) explicit #[presents] markers are still captured (fingerprint omission
    //       does not break the explicit-marker path)
    //   (d) no diagnostic is emitted (parse_failures is empty for the omission case)
    //
    // (d) is the design gap scout identified: the author who INTENDED a fingerprint
    // gets exactly the same behavior as one who deliberately omitted it. There is no
    // "did you mean to add a fingerprint?" lint.
    // ========================================================================

    #[test]
    fn atk_adr009_amd1_no_fingerprint_antigen_produces_no_fingerprint_match_presentations() {
        // (a): An antigen with fingerprint=None must not generate any FingerprintMatch
        // presentations, even if source items would match a doc_contains pattern.
        //
        // The synthesis_pass skips antigens with no fingerprint (filter_map on
        // fingerprint.as_deref() at scan.rs:2734). This tests that the skip is clean.
        let src = r#"
            #[antigen(
                name = "verify-only-class",
                category = AntigenCategory::SubstrateAlignment,
                summary = "A verify-only antigen with no fingerprint."
            )]
            pub struct VerifyOnlyClass;

            /// verify-only-class: this function mentions the antigen by name in a doc comment.
            /// If fingerprint were `doc_contains("verify-only-class")`, this would match.
            pub fn a_function_that_would_match() {}
        "#;
        let report = scan_source(src);
        // Run synthesis_pass directly on the parsed content.
        // Antigen has fingerprint=None — filter_map drops it — fingerprints vec is empty.
        let fingerprints: Vec<(String, antigen_fingerprint::Fingerprint)> = report
            .antigens
            .iter()
            .filter_map(|ag| {
                let raw = ag.fingerprint.as_deref()?;
                antigen_fingerprint::Fingerprint::parse(raw)
                    .ok()
                    .map(|fp| (ag.type_name.clone(), fp))
            })
            .collect();
        assert!(
            fingerprints.is_empty(),
            "ATK-ADR009-AMD1(a): no-fingerprint antigen must produce empty fingerprints vec; \
             got: {fingerprints:?}"
        );
        // No FingerprintMatch presentations — synthesis_pass was never called.
        let fingerprint_matches: Vec<_> = report
            .presentations
            .iter()
            .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
            .collect();
        assert!(
            fingerprint_matches.is_empty(),
            "ATK-ADR009-AMD1(a): no-fingerprint antigen must produce zero FingerprintMatch \
             presentations; got {}: {fingerprint_matches:?}",
            fingerprint_matches.len()
        );
    }

    #[test]
    fn atk_adr009_amd1_no_fingerprint_does_not_suppress_explicit_presents() {
        // (c): A no-fingerprint antigen must not block explicit #[presents] markers.
        // The fingerprint-omission affects only synthesis_pass (inferred sites);
        // explicit markers flow through the attribute-walker path independently.
        let src = r#"
            #[antigen(
                name = "verify-only-class",
                summary = "No fingerprint."
            )]
            pub struct VerifyOnlyClass;

            #[presents(VerifyOnlyClass)]
            pub fn explicitly_marked_site() {}
        "#;
        let report = scan_source(src);
        // The explicit #[presents] must be captured.
        let explicit: Vec<_> = report
            .presentations
            .iter()
            .filter(|p| p.match_kind == MatchKind::ExplicitMarker)
            .collect();
        assert_eq!(
            explicit.len(),
            1,
            "ATK-ADR009-AMD1(c): a no-fingerprint antigen must not suppress explicit \
             #[presents] markers; got {} explicit sites: {:?}",
            explicit.len(),
            explicit
        );
        assert_eq!(
            explicit[0].antigen_type, "VerifyOnlyClass",
            "ATK-ADR009-AMD1(c): explicit site must name the correct antigen"
        );
        // No FingerprintMatch presentations (no fingerprint = no synthesis).
        assert!(
            report
                .presentations
                .iter()
                .all(|p| p.match_kind == MatchKind::ExplicitMarker),
            "ATK-ADR009-AMD1(c): all presentations must be ExplicitMarker; no synthesis \
             occurred (no fingerprint)"
        );
    }

    #[test]
    fn atk_adr009_amd1_no_diagnostic_for_accidental_omission() {
        // (d): The design gap — no diagnostic for accidental fingerprint omission.
        //
        // An author who INTENDED a scan-locatable antigen (and would write
        // `fingerprint = r#"doc_contains("my-class")"#`) but accidentally omitted
        // the field gets exactly the same behavior as an intentional verify-only
        // antigen declaration. No parse_failure, no warning, no lint.
        //
        // This is the silent failure scout flagged: the two cases are indistinguishable
        // from the tool's perspective. The mitigation direction per ADR-009 Amd-1
        // §Enforcement is a future lint: "antigen X has no fingerprint and no
        // detection_model=verify_only classification — consider adding one."
        //
        // This test DOCUMENTS the current behavior (no diagnostic) as a regression
        // anchor. When the lint is implemented, this test must be updated.
        let src = r#"
            #[antigen(
                name = "accidentally-no-fingerprint",
                summary = "Author intended a fingerprint but forgot it."
            )]
            pub struct AccidentallyNoFingerprint;
        "#;
        let report = scan_source(src);
        assert_eq!(
            report.antigens.len(),
            1,
            "antigen declaration must be scanned"
        );
        assert!(
            report.antigens[0].fingerprint.is_none(),
            "no fingerprint in declaration"
        );
        // CURRENT BEHAVIOR: no parse failure, no warning for the omission.
        // The tool cannot distinguish "intentionally verify-only" from
        // "accidentally omitted the fingerprint".
        assert!(
            report.parse_failures.is_empty(),
            "ATK-ADR009-AMD1(d) documented gap: no diagnostic for accidental fingerprint \
             omission; parse_failures is empty even when an author forgot to write a \
             fingerprint. Mitigation direction: a future lint warning for antigens with \
             no fingerprint and no explicit detection_model=verify_only annotation."
        );
    }

    // ========================================================================
    // ADR-023: #[immunosuppress(since=, duration_cap=)] → typed DeferredDefense
    // fields (since/duration_cap), so the audit can enforce the cap. Previously
    // these lived only as unparsed see[] string tags — the
    // ImmunosuppressDurationCapExceeded-unreachable root cause (d72dacf).
    // ========================================================================

    #[test]
    fn scan_immunosuppress_captures_since_and_duration_cap_as_typed_fields() {
        let report = scan_source(
            r#"
            #[immunosuppress(rationale = "mid-refactor, defense lands in PR42", since = "2020-01-01", duration_cap = 30)]
            fn suppressed_site() {}
            "#,
        );
        assert_eq!(report.deferred_defenses.len(), 1);
        let d = &report.deferred_defenses[0];
        assert_eq!(d.kind, crate::scan::DeferredDefenseKind::Immunosuppress);
        assert_eq!(
            d.since.as_deref(),
            Some("2020-01-01"),
            "since must be captured as a typed field, not a see[] string tag"
        );
        assert_eq!(
            d.duration_cap,
            Some(30),
            "duration_cap must be captured as a typed field"
        );
        // The old string-tag encoding must be gone — see[] should not carry
        // since/duration_cap anymore (the audit reads the typed fields).
        assert!(
            !d.see
                .iter()
                .any(|s| s.starts_with("since:") || s.starts_with("duration_cap:")),
            "since/duration_cap must NOT be stuffed into see[] as string tags; got: {:?}",
            d.see
        );
    }
}

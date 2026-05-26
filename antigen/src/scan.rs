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

use antigen_macros::presents;
use serde::{Deserialize, Serialize};
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

/// A single antigen declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[presents(VecCardinalityMasqueradingAsSet)]
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
    /// Optional fingerprint string in the [`antigen_fingerprint`] grammar
    /// (ADR-010, W6a). Parsed at scan time during the synthesis pass to
    /// emit synthetic [`Presentation`] records for unmarked items that
    /// match the structural pattern. `None` for antigens declared without
    /// a fingerprint (Layer 1 minimum-viable form, ADR-009).
    pub fingerprint: Option<String>,
    /// Canonical declaration site of this antigen, in the
    /// `"<crate-name>@<version>"` form (e.g., `"serde@1.0.193"`).
    ///
    /// ADR-017 (canonical declaration site identity). `None` for
    /// intra-workspace declarations — the default for the workspace-only
    /// scan path. Set by the cargo-metadata-driven `--include-deps`
    /// pipeline after scanning a dependency crate root. The full identity
    /// tuple at the cross-crate boundary is `(type_name, canonical_path)`.
    #[serde(default)]
    pub canonical_path: Option<String>,
    /// Category variants from `category = AntigenCategory::X` (ADR-028).
    ///
    /// Empty vec means absent (v0.1 backward-compat; audit tool emits
    /// `antigen-category-defaulted-implicit-functional` migration hint).
    /// Single-element = pure substrate-alignment or functional-correctness.
    /// Two elements = hybrid antigen requiring both witness types.
    #[serde(default)]
    pub category: Vec<crate::category::AntigenCategory>,
}

/// Identity of the Rust item that an antigen-related attribute is applied to.
///
/// W3 (sweep A2): replaces the old proximity heuristic in
/// `unaddressed_presentations` with structural matching. `Presentation` and
/// `Immunity` carry an `item_target` that names the *item they live on*; two
/// declarations address each other if and only if their item targets are
/// equal (and they're in the same file and reference the same antigen).
///
/// The variants mirror the visitor entry points:
/// - `Struct`, `Enum`, `Trait`: top-level type declarations
/// - `Fn`: a free function
/// - `Impl`: an `impl ... for ...` or inherent `impl ...` block
/// - `ImplFn`: a method inside an impl block (with its enclosing impl
///   target captured so two methods named `drop` on different types
///   don't collide)
/// - `Unknown`: visitor fallback for shapes we don't yet model (e.g.,
///   free constants); kept rather than asserted so scans never panic on
///   third-party code with novel item shapes.
///
/// `trait_path` on `Impl`/`ImplFn` is the trait being implemented (e.g.,
/// `Drop` from `impl Drop for X`); `None` for inherent impls. The path is
/// captured as a string after canonical rendering — full-path equality is
/// W3's invariant, but A3 cross-crate matching may need richer
/// representation later.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemTarget {
    /// A top-level struct declaration. Holds the struct identifier.
    Struct(String),
    /// A top-level enum declaration. Holds the enum identifier.
    Enum(String),
    /// A top-level trait declaration. Holds the trait identifier.
    Trait(String),
    /// A free function. Holds the function identifier.
    Fn(String),
    /// A type alias declaration (`type Foo = ...;`). Holds the alias
    /// identifier. ATK-W3-005: without this, type aliases fall back to
    /// `Unknown`, and two unrelated Unknown items collide on equality.
    TypeAlias(String),
    /// An `impl ... for ...` or inherent `impl ...` block. The trait
    /// portion is `None` for inherent impls and `Some(rendered_path)`
    /// otherwise. Methods inside the impl carry an `ImplFn` target that
    /// references the same `target_type` and `trait_path`.
    Impl {
        /// The trait being implemented, rendered to its canonical token
        /// string. `None` for inherent impls (no trait).
        trait_path: Option<String>,
        /// The implementing type, rendered to its canonical token string.
        target_type: String,
    },
    /// A method inside an impl block. `target_type` and `trait_path`
    /// mirror the enclosing `Impl` target so that two methods with the
    /// same name on different types do not collide for matching purposes.
    ImplFn {
        /// Trait of the enclosing impl, if any.
        trait_path: Option<String>,
        /// Type of the enclosing impl.
        target_type: String,
        /// The method name.
        fn_name: String,
    },
    /// A method declared inside a `trait` definition. Pairs with
    /// `ImplFn { trait_path: Some(trait_name), fn_name, .. }` — the
    /// presents-on-trait-method + immune-on-impl-method pattern is one
    /// of the W3 README's adversarial cases. Holds the trait name and
    /// method name; matching bridges `TraitFn` ↔ `ImplFn` explicitly.
    TraitFn {
        /// The enclosing trait identifier.
        trait_name: String,
        /// The method name.
        fn_name: String,
    },
    /// An enum variant carrying its own attribute (e.g. `#[presents]` on the
    /// `External` variant of `enum RequestKind`). Holds the enclosing enum
    /// identifier and the variant identifier so two variants of the same name
    /// on different enums do not collide. ATK-A2-ENUM-VARIANT: without a
    /// `visit_variant` override the scanner silently ignored variant-level
    /// attributes — a presentation invisible to failure-class memory.
    EnumVariant {
        /// The enclosing enum identifier.
        enum_name: String,
        /// The variant identifier.
        variant_name: String,
    },
    /// An associated `const` inside an `impl` block (e.g.
    /// `#[presents]` on `impl Parser { const MAX_INPUT_BYTES … }`). Mirrors
    /// [`Self::ImplFn`]: carries the enclosing impl's trait (if any) + type +
    /// the const name. ATK-A2-IMPL-CONST: without a `visit_impl_item_const`
    /// override the scanner silently ignored impl-const attributes — the same
    /// blind-spot class as [`Self::EnumVariant`].
    ImplConst {
        /// Trait of the enclosing impl, if any.
        trait_path: Option<String>,
        /// Type of the enclosing impl.
        target_type: String,
        /// The associated-const name.
        const_name: String,
    },
    /// A free-standing (top-level or module-level) `const` item carrying its own
    /// attribute (e.g. `#[presents] const MAX_REQUEST_SIZE: usize = …`). Holds
    /// the const identifier. ATK-A2-TOPLEVEL-CONST: same scanner blind-spot
    /// class as [`Self::EnumVariant`] / [`Self::ImplConst`] — a missing
    /// `visit_item_const` override let the attribute pass unscanned.
    Const(String),
    /// A free-standing `static` item carrying its own attribute (e.g.
    /// `#[presents] static GLOBAL_LIMIT: usize = …`). Distinct from
    /// [`Self::Const`] so a `static` and a `const` of the same name do not
    /// collide. Closed preemptively alongside the const cases (ADR-007:
    /// the same scanner blind-spot class — a missing `visit_item_static`
    /// override would otherwise let the attribute pass unscanned).
    Static(String),
    /// Visitor fallback for shapes we don't yet model (e.g., free
    /// constants, modules with attribute-bearing macro-expansion).
    /// Kept rather than asserted so scans never panic on third-party
    /// code with novel item shapes. Carries the source line so that two
    /// Unknown items at different positions are not falsely equal —
    /// ATK-W3-005 caught the previous unit-variant form colliding on
    /// equality across unrelated items. The line is a best-effort
    /// discriminator; perfect identity for unhandled shapes requires
    /// per-shape visitor methods (deferred).
    Unknown {
        /// Best-effort line number; used as a tie-breaker for equality.
        line: usize,
    },
}

impl ItemTarget {
    /// Best-effort short name for diagnostic output. Not used for matching.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Struct(n)
            | Self::Enum(n)
            | Self::Trait(n)
            | Self::Fn(n)
            | Self::TypeAlias(n)
            | Self::Const(n)
            | Self::Static(n) => n.clone(),
            Self::Impl {
                trait_path: Some(t),
                target_type,
            } => format!("impl {t} for {target_type}"),
            Self::Impl {
                trait_path: None,
                target_type,
            } => format!("impl {target_type}"),
            Self::ImplFn {
                trait_path: Some(t),
                target_type,
                fn_name,
            } => format!("<{target_type} as {t}>::{fn_name}"),
            Self::ImplFn {
                trait_path: None,
                target_type,
                fn_name,
            } => format!("{target_type}::{fn_name}"),
            Self::TraitFn {
                trait_name,
                fn_name,
            } => format!("trait {trait_name}::{fn_name}"),
            Self::EnumVariant {
                enum_name,
                variant_name,
            } => format!("{enum_name}::{variant_name}"),
            Self::ImplConst {
                trait_path: Some(t),
                target_type,
                const_name,
            } => format!("<{target_type} as {t}>::{const_name}"),
            Self::ImplConst {
                trait_path: None,
                target_type,
                const_name,
            } => format!("{target_type}::{const_name}"),
            Self::Unknown { line } => format!("<unknown at line {line}>"),
        }
    }

    /// Whether this item-target addresses another for the purposes of the
    /// presents+immune match. The relation is reflexive and symmetric.
    ///
    /// W3 (sweep A2) — the "addresses" relation is wider than strict
    /// equality, per the A2 README's matching rules:
    ///
    /// - Same kind, same name (Struct/Enum/Trait/Fn/TypeAlias) → match.
    /// - Two `Impl` blocks for the same base type (regardless of trait
    ///   being implemented) → match. Generics are normalised away so
    ///   `Container<T>` and `Container<i32>` share a base type.
    /// - Two `ImplFn` items on the same base type with the same method
    ///   name → match (regardless of whether the impls implement the
    ///   same trait).
    /// - `TraitFn(T, f)` ↔ `ImplFn { trait_path: Some(T), fn_name: f, .. }`
    ///   → match. Handles the README's case (a): presents on a trait
    ///   method, immune on the impl method.
    /// - `Unknown` never matches anything — never a false negative on
    ///   unclassified items (per ATK-W3-005's premise).
    /// - Heterogeneous variants don't match.
    ///
    /// The relaxation is intentional: false positives in the matcher
    /// surface as unaddressed presentations the user can investigate;
    /// false negatives silently green-light a vulnerability. Err on the
    /// side of matching legitimate presents+immune pairings.
    #[must_use]
    #[allow(
        clippy::match_same_arms,
        reason = "the explicit `Unknown` arm is the load-bearing invariant — \
                  Unknown items must NEVER match anything, including each other. \
                  Keeping it explicit (even though it duplicates the `_` wildcard's \
                  body) makes the invariant readable and refactor-safe."
    )]
    pub fn addresses(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unknown { .. }, _) | (_, Self::Unknown { .. }) => false,
            (Self::Struct(a), Self::Struct(b))
            | (Self::Enum(a), Self::Enum(b))
            | (Self::Trait(a), Self::Trait(b))
            | (Self::Fn(a), Self::Fn(b))
            | (Self::TypeAlias(a), Self::TypeAlias(b))
            | (Self::Const(a), Self::Const(b))
            | (Self::Static(a), Self::Static(b)) => a == b,
            (
                Self::Impl {
                    target_type: t1, ..
                },
                Self::Impl {
                    target_type: t2, ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2),
            (
                Self::ImplFn {
                    target_type: t1,
                    fn_name: f1,
                    ..
                },
                Self::ImplFn {
                    target_type: t2,
                    fn_name: f2,
                    ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2) && f1 == f2,
            (
                Self::TraitFn {
                    trait_name,
                    fn_name: tf,
                },
                Self::ImplFn {
                    trait_path: Some(t),
                    fn_name: imf,
                    ..
                },
            )
            | (
                Self::ImplFn {
                    trait_path: Some(t),
                    fn_name: imf,
                    ..
                },
                Self::TraitFn {
                    trait_name,
                    fn_name: tf,
                },
            ) => trait_name == t && tf == imf,
            (
                Self::TraitFn {
                    trait_name: t1,
                    fn_name: f1,
                },
                Self::TraitFn {
                    trait_name: t2,
                    fn_name: f2,
                },
            ) => t1 == t2 && f1 == f2,
            (
                Self::EnumVariant {
                    enum_name: e1,
                    variant_name: v1,
                },
                Self::EnumVariant {
                    enum_name: e2,
                    variant_name: v2,
                },
            ) => e1 == e2 && v1 == v2,
            (
                Self::ImplConst {
                    target_type: t1,
                    const_name: c1,
                    ..
                },
                Self::ImplConst {
                    target_type: t2,
                    const_name: c2,
                    ..
                },
            ) => normalize_type_name(t1) == normalize_type_name(t2) && c1 == c2,
            _ => false,
        }
    }
}

/// Strip generic parameters from a `quote::ToTokens`-rendered type name.
/// `"Container < T >"` → `"Container"`. Used for impl-block matching so
/// that `impl<T> Container<T>` and `impl Container<i32>` share an
/// addressable identity at the type level.
fn normalize_type_name(rendered: &str) -> String {
    let s = rendered.trim();
    s.find('<')
        .map_or_else(|| s.to_string(), |idx| s[..idx].trim().to_string())
}

/// How a [`Presentation`] was discovered.
///
/// Per ADR-001 Amendment 1 Change 2 (the 5-state matrix): explicit
/// `#[presents]` markers and synthetic fingerprint matches share the
/// `Presentation` shape but differ in provenance. Audit and CLI output
/// distinguish the two — passive (synthetic) matches are the structural
/// surface ADR-010's recognition-not-yet-marked half exposes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    /// `#[presents(X)]` was written on this item.
    #[default]
    ExplicitMarker,
    /// The item was not marked but matches an antigen's fingerprint per
    /// ADR-010. Surfaced by the synthesis pass after explicit collection.
    FingerprintMatch,
}

/// Provenance entry: the identity of one ancestor antigen whose
/// presentations propagated to a descendant via `#[descended_from]`.
///
/// ADR-018 (propagation semantics). Each [`Presentation`] inherited via
/// the lineage walk carries one [`ProvenanceEntry`] per transitive
/// ancestor it inherited from. The entry fully identifies the ancestor
/// via the same `(antigen_type, canonical_path)` tuple that
/// [`unaddressed_presentations`](ScanReport::unaddressed_presentations)
/// uses for antigen identity.
///
/// `Ord` is derived so a `BTreeSet<ProvenanceEntry>` can be used
/// internally during propagation for O(log n) set-union; the serialised
/// form is `Vec<ProvenanceEntry>` for JSON schema stability.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    /// Antigen type name at the ancestor declaration site.
    pub antigen_type: String,
    /// Crate identity (`"<crate-name>@<version>"`) where the ancestor
    /// antigen was originally declared. `None` if the ancestor is
    /// intra-workspace.
    pub canonical_path: Option<String>,
}

/// A `#[presents(antigen_type)]` declaration or synthetic fingerprint match
/// discovered in source.
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
    /// Item identity for structural matching against `Immunity`. W3 (sweep A2).
    pub item_target: ItemTarget,
    /// How this presentation was discovered: explicit marker vs fingerprint
    /// match. W6a (sweep A2). Defaults to `ExplicitMarker` for backwards
    /// compatibility with serialized reports from before W6a.
    #[serde(default)]
    pub match_kind: MatchKind,
    /// Canonical declaration site of the *antigen* referenced by this
    /// presentation (not the presentation's own location). ADR-017.
    /// `None` for intra-workspace antigens; `Some("<crate>@<version>")`
    /// for cross-crate antigens (set by the `--include-deps` driver
    /// after scanning the dependency crate root).
    #[serde(default)]
    pub canonical_path: Option<String>,
    /// Provenance chain of ancestor antigens this presentation was
    /// inherited from. ADR-018 (propagation semantics).
    ///
    /// - `None` = direct presentation (explicit marker or fingerprint match).
    /// - `Some(chain)` = synthesized via the propagation walk; the chain
    ///   names every transitive ancestor antigen whose presentation
    ///   propagated here (set-union across diamond paths). Empty `Vec`
    ///   inside `Some` is forbidden — normalised to `None` at construction.
    ///
    /// Audit emits a warn-level diagnostic for presentations with
    /// `inherited_from = Some(_)` that lack a re-attested immunity or
    /// tolerance on the descendant site (state 7 of the 7-state matrix).
    #[serde(default)]
    pub inherited_from: Option<Vec<ProvenanceEntry>>,
    /// FNV-1a structural digest of the presented item at scan time.
    /// Populated for `FingerprintMatch` presentations; empty string for
    /// `ExplicitMarker` presentations and inherited presentations where the
    /// ancestor was an explicit marker. Allows adopters to pass this value
    /// directly to `attest scaffold --fingerprint` without needing an
    /// `#[immune]` marker first (DX finding 6).
    #[serde(default)]
    pub structural_fingerprint: String,
}

/// An `#[immune(antigen_type, witness = ...)]` declaration discovered in source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Immunity {
    /// The antigen type referenced.
    pub antigen_type: String,
    /// The witness expression as a string (validated lazily).
    /// Empty string when `requires_predicate` is set (substrate-witness path).
    pub witness: String,
    /// Substrate-witness predicate JSON, present when the immunity was
    /// declared with `requires = <predicate>` (ADR-019 §P3b). The JSON
    /// matches `serde_json::to_string(&antigen_attestation::Predicate)`.
    /// Mutually exclusive with a non-empty `witness`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_predicate: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural matching against `Presentation`. W3 (sweep A2).
    pub item_target: ItemTarget,
    /// Canonical declaration site of the *antigen* referenced by this
    /// immunity claim (not where the immunity is declared). ADR-017.
    /// `None` for intra-workspace antigens.
    #[serde(default)]
    pub canonical_path: Option<String>,
    /// Structural digest of the defended item's source, computed via
    /// [`antigen_fingerprint::structural_digest`]. This is the value an
    /// adopter signs against (`signed_against_fingerprint`); audit recomputes
    /// it to detect drift for `against = "current"` / `fresh_within_days`
    /// (ADR-019). Distinct from the antigen *pattern* fingerprint — this is a
    /// per-item content hash of the immune site. Empty only on the legacy
    /// deserialization path (pre-this-field reports); always populated by scan.
    #[serde(default)]
    pub structural_fingerprint: String,
}

/// An `#[antigen_tolerance(antigen, rationale = "...", until = "...", see = [...])]`
/// declaration discovered in source. Per ADR-011.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    /// The antigen type referenced (last path segment).
    pub antigen_type: String,
    /// The rationale string from the macro args (required, non-empty per
    /// ADR-011).
    pub rationale: String,
    /// Optional expiry tag (e.g., `"v1.0"`); `None` for forever-tolerance.
    pub until: Option<String>,
    /// Optional open-vocabulary references list (mirrors ADR-009's `references`
    /// field shape).
    pub see: Vec<String>,
    /// Substrate-witness sidecar predicate JSON, present when the tolerance
    /// was declared with `requires = <predicate>` (ADR-019 tolerance tier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_predicate: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural matching against fingerprint matches.
    pub item_target: ItemTarget,
    /// Canonical declaration site of the *antigen* this tolerance
    /// addresses. ADR-017. `None` for intra-workspace antigens.
    #[serde(default)]
    pub canonical_path: Option<String>,
    /// Structural digest of the tolerated item's source — the
    /// `signed_against_fingerprint` value for substrate-witness tolerance
    /// sidecars (ADR-019 tolerance tier). Mirrors [`Immunity::structural_fingerprint`].
    #[serde(default)]
    pub structural_fingerprint: String,
}

// ============================================================================
// Deferred-Defense Family output types (ADR-023)
// ============================================================================

/// Which of the four deferred-defense postures was declared.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeferredDefenseKind {
    /// `#[anergy]` — deferred-but-muted; until required.
    Anergy,
    /// `#[immunosuppress]` — surgical silencing with duration cap.
    Immunosuppress,
    /// `#[poxparty]` — intentional controlled exposure; cfg-gated.
    Poxparty,
    /// `#[orient]` — see-also context; lightest-weight.
    Orient,
}

/// A deferred-defense declaration discovered in source (ADR-023).
///
/// Covers all four primitives: `#[anergy]`, `#[immunosuppress]`,
/// `#[poxparty]`, `#[orient]`. The `kind` field distinguishes them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredDefense {
    /// Which deferred-defense posture was declared.
    pub kind: DeferredDefenseKind,
    /// Antigen type referenced, if a positional argument was provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antigen_type: Option<String>,
    /// Primary text field: `rationale` (immunosuppress), `reason` (anergy),
    /// `exercise_type` (poxparty), or empty string (orient).
    /// For anergy: `reason`; for immunosuppress: `rationale`;
    /// for poxparty: `exercise_type`; for orient: empty string.
    pub text: String,
    /// Expiry date string (`until` field), if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    /// Optional co-stimulation hint (anergy only; advisory).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_co_stimulation: Option<String>,
    /// Optional signer identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by: Option<String>,
    /// See-also references (orient; also poxparty name field stored here).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub see: Vec<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (fn, impl, struct, etc.).
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

// ============================================================================
// Convergent-Evidence Family output types (ADR-024)
// ============================================================================

/// Which of the seven convergent-evidence primitives was declared.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConvergentEvidenceKind {
    /// `#[diagnostic(modalities = [...], min_independent = N)]`.
    Diagnostic,
    /// `#[clonal(witness = ..., iterations = N, seed = SeedKind::...)]`.
    Clonal,
    /// `#[igg(witnesses = [...], historical_span = N, min_reattestations = N)]`.
    Igg,
    /// `#[crossreactive(fingerprints = [...])]`.
    Crossreactive,
    /// `#[polyclonal]` marker.
    Polyclonal,
    /// `#[monoclonal]` marker.
    Monoclonal,
    /// `#[adcc]` marker.
    Adcc,
}

/// A convergent-evidence declaration discovered in source (ADR-024).
///
/// Covers all seven primitives. The `kind` field distinguishes them; the
/// rest of the fields are loosely-typed string captures shared across
/// kinds for forward-compat with the adoption gradient (per ADR-009).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergentEvidence {
    /// Which convergent-evidence primitive was declared.
    pub kind: ConvergentEvidenceKind,
    /// `#[diagnostic]` modality classes — the final segment of each
    /// `WitnessClass::*` path, e.g., `"StaticAnalysis"`. Empty for
    /// other kinds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modality_classes: Vec<String>,
    /// `#[diagnostic]` `min_independent` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_independent: Option<u64>,
    /// `#[clonal]` `witness` identifier (token string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
    /// `#[clonal]` `iterations` value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u64>,
    /// `#[clonal]` `seed` final ident (e.g., `"Random"`, `"Fixed"`).
    /// `Fixed` here is itself a bug-signal — the proc-macro rejects it
    /// at parse time, but a scan over older source can still surface it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_kind: Option<String>,
    /// `#[igg]` historical span in days.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical_span: Option<u64>,
    /// `#[igg]` minimum re-attestations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_reattestations: Option<u64>,
    /// `#[igg]` witness identifier strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<String>,
    /// `#[crossreactive]` fingerprint strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fingerprints: Vec<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated (fn, impl, struct, etc.).
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

/// Which recurrent-emergence primitive was declared (ADR-024 §Family 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecurrentKind {
    /// `#[itch]` — below-threshold noticing (cognitive-organizational).
    Itch,
    /// `#[recurrence_anchor]` — cross-substrate recurrence formally anchored
    /// (clinical-medicine).
    RecurrenceAnchor,
    /// `#[crystallize]` — itch-cluster promotion to named failure-class
    /// (cognitive-organizational).
    Crystallize,
    /// `#[chronic]` — low-level persistent NON-cross-substrate signal
    /// (immunology-proper).
    Chronic,
    /// `#[saturate]` — accumulating saturation evidence
    /// (cognitive-organizational).
    Saturate,
    /// `#[strand]` — thread of related noticing (cognitive-organizational).
    Strand,
}

/// A recurrent-emergence declaration discovered in source (ADR-024 §Family 2).
///
/// Covers all six primitives. The `kind` field distinguishes them; the rest
/// are loosely-typed optional captures shared across kinds for forward-compat
/// with the adoption gradient (per ADR-009), mirroring [`ConvergentEvidence`].
/// All members are antigen-category `SubstrateAlignment` per ADR-028.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrentDeclaration {
    /// Which recurrent primitive was declared.
    pub kind: RecurrentKind,
    /// `name` slug — `#[itch]`, `#[crystallize]`, `#[strand]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Antigen-type path final segment — `#[recurrence_anchor]`,
    /// `#[chronic]`, optional on `#[itch]`/`#[crystallize]`/`#[saturate]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antigen_type: Option<String>,
    /// `description` / `summary` text — the human-facing noticing/rationale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// `#[recurrence_anchor]` instance count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instances: Option<u64>,
    /// `since` date-or-version — `#[recurrence_anchor]`, `#[chronic]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// `#[recurrence_anchor]` rationale text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// `#[crystallize]` `from_itches` ident strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub from_itches: Vec<String>,
    /// `#[strand]` `anchored_by` ident strings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchored_by: Vec<String>,
    /// `#[chronic]` `managed_by` role/team.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_by: Option<String>,
    /// `#[saturate]` `contributing_to` slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributing_to: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

/// Which mucosal-boundary primitive was declared (ADR-027 + Amendment 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MucosalKindTag {
    /// `#[mucosal]` — active boundary defense.
    Mucosal,
    /// `#[mucosal_delegate]` — defense delegated to a named handler.
    MucosalDelegate,
    /// `#[mucosal_tolerant]` — boundary intentionally permitted.
    MucosalTolerant,
}

/// A mucosal-boundary declaration discovered in source (ADR-027 + Amendment 1).
///
/// Covers all three primitives. The `tag` distinguishes them; the rest are
/// loosely-typed optional captures shared across kinds (forward-compat per
/// ADR-009), mirroring [`RecurrentDeclaration`]. `boundary_kind` holds the
/// final segment of the `MucosalKind::X` path (`"UserInput"` etc.).
/// All members are antigen-category `SubstrateAlignment` per ADR-028.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MucosalDeclaration {
    /// Which primitive was declared.
    pub tag: MucosalKindTag,
    /// `MucosalKind::X` final segment — the boundary kind (`kind` on
    /// `#[mucosal]`/`#[mucosal_tolerant]`, `boundary` on `#[mucosal_delegate]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boundary_kind: Option<String>,
    /// `rationale` text (all three primitives).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// `#[mucosal_delegate]` `handled_by` path rendered to its final segment
    /// (the handler function name). Audit-time kind-matching (Change 5)
    /// resolves this against the workspace function index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by: Option<String>,
    /// `#[mucosal_tolerant]` `accepts` description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepts: Option<String>,
    /// `#[mucosal_tolerant]` `reviewed_by`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    /// `#[mucosal_tolerant]` `until` review-deadline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    /// Source file path.
    pub file: PathBuf,
    /// Line number.
    pub line: usize,
    /// Item kind that was annotated.
    pub item_kind: String,
    /// Item identity for structural cross-referencing.
    pub item_target: ItemTarget,
}

/// A file that failed to parse during a scan, with the associated error.
///
/// Serializes as `{"file": "...", "error": "..."}` — named fields, consistent
/// with every other collection in [`ScanReport`]. (`Vec<(PathBuf, String)>`
/// would serialize as positional JSON arrays, breaking JSON consumers.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseFailure {
    /// Path to the file that failed.
    pub file: PathBuf,
    /// Human-readable parse error.
    pub error: String,
}

/// A `#[descended_from(parent)]` lineage edge discovered during scan.
///
/// A3 (sweep) — every `#[descended_from]` site contributes one edge with
/// `child` = the bearing antigen type's name and `parent` = the last segment
/// of the path supplied as the attribute argument. Edges are collected during
/// the visitor pass and consumed afterwards by:
///
/// - cycle detection (ATK-A3-002 — required safety guard before propagation)
/// - the propagation walk (ADR-013 — child inherits parent's presentations)
/// - [`ScanReport::orphaned_lineage_edges`] (ATK-A3-003 — semantic warning
///   parallel to [`ScanReport::orphaned_tolerances`] for declarations whose
///   parent is no longer present in the scan)
///
/// `#[descended_from]` is meaningful only on antigen-type declarations
/// (unit `struct` and class-shaped `enum`). The visitor surfaces other
/// placements as `parse_failures`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Bare type name of the antigen bearing `#[descended_from]` (the child).
    pub child: String,
    /// Last path segment of the `#[descended_from]` argument (the parent
    /// antigen type), stored as the bare type name. Cross-crate identity
    /// at the parent endpoint lives in [`Self::parent_canonical_path`].
    pub parent: String,
    /// Source file path.
    pub file: PathBuf,
    /// Line number of the `#[descended_from]` attribute.
    pub line: usize,
    /// Canonical declaration site of the *parent* antigen (the
    /// `#[descended_from]` argument), `"<crate-name>@<version>"`.
    /// ADR-017. `None` for intra-workspace ancestors.
    ///
    /// Two `parent_canonical_path` fields make cross-crate lineage edges
    /// first-class: an intra-workspace child can declare descent from a
    /// cross-crate parent, or vice-versa. The full lineage edge identity
    /// is `(child, parent, child_canonical_path, parent_canonical_path)`.
    #[serde(default)]
    pub parent_canonical_path: Option<String>,
    /// Canonical declaration site of the *child* antigen (the bearer of
    /// `#[descended_from]`). ADR-017. `None` for intra-workspace.
    #[serde(default)]
    pub child_canonical_path: Option<String>,
}

/// Aggregate result of scanning a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    /// All discovered antigen declarations.
    pub antigens: Vec<AntigenDeclaration>,
    /// All discovered `#[presents]` sites + synthetic fingerprint matches.
    /// Distinguish the two via [`Presentation::match_kind`].
    pub presentations: Vec<Presentation>,
    /// All discovered `#[immune]` sites.
    pub immunities: Vec<Immunity>,
    /// All discovered `#[antigen_tolerance]` sites. W6a (sweep A2).
    #[serde(default)]
    pub tolerances: Vec<Toleration>,
    /// All discovered `#[descended_from]` edges. A3.
    ///
    /// `#[serde(default)]` so reports serialized before A3 deserialize
    /// cleanly with an empty edge list (additive change, not breaking).
    #[serde(default)]
    pub lineage_edges: Vec<LineageEdge>,
    /// All discovered deferred-defense declarations: `#[anergy]`,
    /// `#[immunosuppress]`, `#[poxparty]`, `#[orient]`. ADR-023.
    ///
    /// `#[serde(default)]` so pre-v0.2 reports deserialize cleanly.
    #[serde(default)]
    pub deferred_defenses: Vec<DeferredDefense>,
    /// All discovered convergent-evidence declarations: `#[diagnostic]`,
    /// `#[clonal]`, `#[igg]`, `#[crossreactive]`, `#[polyclonal]`,
    /// `#[monoclonal]`, `#[adcc]`. ADR-024.
    ///
    /// `#[serde(default)]` so pre-v0.2 reports deserialize cleanly.
    #[serde(default)]
    pub convergent_evidences: Vec<ConvergentEvidence>,
    /// All discovered recurrent-emergence declarations: `#[itch]`,
    /// `#[recurrence_anchor]`, `#[crystallize]`, `#[chronic]`,
    /// `#[saturate]`, `#[strand]`. ADR-024 §Family 2.
    ///
    /// `#[serde(default)]` so pre-recurrent reports deserialize cleanly.
    #[serde(default)]
    pub recurrent_declarations: Vec<RecurrentDeclaration>,
    /// All discovered mucosal-boundary declarations: `#[mucosal]`,
    /// `#[mucosal_delegate]`, `#[mucosal_tolerant]`. ADR-027 + Amendment 1.
    ///
    /// `#[serde(default)]` so pre-mucosal reports deserialize cleanly.
    #[serde(default)]
    pub mucosal_declarations: Vec<MucosalDeclaration>,
    /// Files scanned successfully.
    pub files_scanned: usize,
    /// Files that failed to parse.
    pub parse_failures: Vec<ParseFailure>,
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
    /// W3 (sweep A2) — structural item-identity matching. A `Presentation`
    /// and an `Immunity` "address each other" when:
    ///
    /// - they reference the same `antigen_type`, AND
    /// - they're in the same source file, AND
    /// - their `item_target` values are equal (i.e., they're applied to
    ///   the same Rust item).
    ///
    /// This replaces the pre-W3 20-line proximity heuristic, which produced
    /// false positives in multi-impl files (immunity on `impl X` matched
    /// presentation on `impl Y` if their attributes happened to be within
    /// 20 lines) and false negatives when long doc-comments separated paired
    /// declarations on the same item.
    ///
    /// Cross-file matching remains out of scope here — different items can
    /// share names across modules, and the structural identity of an
    /// "item" extends to its containing module path. That's A3 territory
    /// (cross-crate scan + `#[descended_from]` propagation).
    #[must_use]
    pub fn unaddressed_presentations(&self) -> Vec<UnaddressedPresentation> {
        // ADR-017 §addresses() semantics: known-antigen lookup uses the
        // canonical_path-aware tuple `(type_name, canonical_path)`.
        let known_antigens: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();

        let mut result = Vec::new();
        for p in &self.presentations {
            let has_matching_immunity =
                self.immunities.iter().any(|i| addresses_for_immunity(i, p));
            // W6a: tolerance acknowledges a presentation per ADR-011
            // §Mechanics. A site with `#[antigen_tolerance(X, ...)]` for
            // the same antigen on the same item is reported under
            // "tolerated", not "unaddressed".
            let has_matching_tolerance = self
                .tolerances
                .iter()
                .any(|t| addresses_for_tolerance(t, p));
            if !has_matching_immunity && !has_matching_tolerance {
                result.push(UnaddressedPresentation {
                    presentation: p.clone(),
                    antigen_known: known_antigens
                        .contains(&(p.antigen_type.as_str(), p.canonical_path.as_deref())),
                });
            }
        }
        result
    }

    /// Tolerances whose named antigen is no longer declared in the scanned
    /// workspace. Per ADR-011 §Mechanics + ATK-A2-009 (the stale-tolerance
    /// orphan check, naturalist's biology cognate "peripheral suppression
    /// continuing after the antigen it suppressed is no longer present").
    ///
    /// Cross-crate antigens are deferred to A3 — for v0.1, an "orphan" is a
    /// tolerance whose antigen `type_name` doesn't appear in any
    /// `AntigenDeclaration` in the same scan. Consumers using cross-crate
    /// antigens may produce false positives here; that's the recognized
    /// v0.1 limitation.
    #[must_use]
    pub fn orphaned_tolerances(&self) -> Vec<&Toleration> {
        // ADR-017 + ADR-018 §Enforcement: orphan checks compare
        // `(type_name, canonical_path)` tuples, NOT bare names.
        // Two crates each declaring `Foo` would have the same `type_name`
        // but distinct `canonical_path` values; a tolerance for
        // `foo@1.0.0::Foo` is orphaned when only `foo@2.0.0::Foo` is in
        // scope, even though "Foo" appears in `self.antigens`.
        let known: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();
        self.tolerances
            .iter()
            .filter(|t| !known.contains(&(t.antigen_type.as_str(), t.canonical_path.as_deref())))
            .collect()
    }

    /// Lineage edges whose parent antigen is not present in the scan.
    ///
    /// A3 / ATK-A3-003 — parallel to [`ScanReport::orphaned_tolerances`].
    ///
    /// A `#[descended_from(Parent)]` declaration whose `Parent` is no
    /// longer declared in the scanned workspace (rename, removal, or
    /// — for v0.1 — a parent that lives in a not-yet-scanned crate) is
    /// a *semantic warning*, not a structural error: the scan completed
    /// correctly, but the declaration references something that isn't
    /// there. Surfaced via this query method rather than emitted into
    /// `parse_failures` so callers (CLI, audit tooling, IDE plugins)
    /// choose the severity, the same channel discipline used for
    /// orphaned tolerances.
    ///
    /// Cross-crate antigens are deferred to A3+ — for v0.1, an "orphan"
    /// is a lineage edge whose `parent` doesn't appear as a `type_name`
    /// in any [`AntigenDeclaration`] in the same scan. Consumers using
    /// cross-crate antigens may produce false positives here; that's
    /// the recognized v0.1 limitation.
    ///
    /// See also [`ScanReport::dangling_child_lineage_edges`] for the dual case
    /// (child missing rather than parent missing).
    #[must_use]
    pub fn orphaned_lineage_edges(&self) -> Vec<&LineageEdge> {
        // ADR-017 + ADR-018 §Enforcement: orphan check compares
        // `(type_name, canonical_path)` tuples. An edge with
        // `parent_canonical_path: Some("foo@1.0.0")` is satisfied ONLY by
        // an AntigenDeclaration with matching `type_name` AND matching
        // `canonical_path`. Bare-name equality alone allows cross-crate
        // name collision to silently mask orphans (ATK-A3-006).
        let known: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();
        self.lineage_edges
            .iter()
            .filter(|e| !known.contains(&(e.parent.as_str(), e.parent_canonical_path.as_deref())))
            .collect()
    }

    /// Lineage edges whose CHILD has no [`AntigenDeclaration`] in the scan.
    ///
    /// BUG-A3-002 fix (adversarial 2026-05-09). The dual of
    /// [`ScanReport::orphaned_lineage_edges`] — `orphaned` checks the
    /// parent endpoint, `dangling` checks the child endpoint.
    ///
    /// A struct or enum bearing `#[descended_from(Parent)]` *without* its
    /// own `#[antigen]` declaration is structurally incoherent: it claims
    /// to inherit into the antigen system without being a participant
    /// itself. The propagation walk (D1.5) cannot meaningfully attach
    /// inherited presentations to a non-antigen child — the descendant
    /// has no record in [`ScanReport::antigens`] for inheritance to flow
    /// into.
    ///
    /// Surfaced as a *semantic warning*, not a `parse_failure` — the
    /// declaration is structurally well-formed; only the relationship
    /// to the antigen registry is missing. Caller (CLI, audit tooling)
    /// chooses severity, mirroring the `orphaned_tolerances` /
    /// `orphaned_lineage_edges` channel discipline.
    ///
    /// The propagation walk skips edges flagged by this query the same
    /// way it skips edges flagged by `orphaned_lineage_edges`.
    #[must_use]
    pub fn dangling_child_lineage_edges(&self) -> Vec<&LineageEdge> {
        // ADR-017 + ADR-018 §Enforcement: canonical_path-aware
        // comparison. Symmetric to `orphaned_lineage_edges` — the child
        // endpoint check uses the same tuple key.
        let known: std::collections::HashSet<(&str, Option<&str>)> = self
            .antigens
            .iter()
            .map(|a| (a.type_name.as_str(), a.canonical_path.as_deref()))
            .collect();
        self.lineage_edges
            .iter()
            .filter(|e| !known.contains(&(e.child.as_str(), e.child_canonical_path.as_deref())))
            .collect()
    }

    /// Stamp `canonical_path` (and `parent_canonical_path` /
    /// `child_canonical_path` on lineage edges) on every record in this
    /// report that does not already have one.
    ///
    /// ADR-017 (Option A — caller stamps post-scan). Called by the
    /// cargo-metadata-driven `--include-deps` driver after running
    /// [`scan_workspace`] on a dependency crate root: the driver knows
    /// the dependency's canonical path (`"<crate-name>@<version>"`), but
    /// the directory scanner doesn't, so the driver stamps the canonical
    /// path on every record post-scan.
    ///
    /// **Idempotent + non-overwriting**: records whose `canonical_path`
    /// (or relevant lineage-edge endpoint) is already `Some(_)` are
    /// left unchanged. This protects records that were stamped during
    /// an earlier (e.g., nested) scan from being silently re-keyed.
    ///
    /// `crate_id` is expected to be in the ADR-017 format
    /// `"<crate-name>@<version>"` (e.g., `"serde@1.0.193"`); the method
    /// does not validate the format — that's the driver's responsibility.
    pub fn stamp_canonical_path(&mut self, crate_id: &str) {
        for a in &mut self.antigens {
            if a.canonical_path.is_none() {
                a.canonical_path = Some(crate_id.to_string());
            }
        }
        for p in &mut self.presentations {
            if p.canonical_path.is_none() {
                p.canonical_path = Some(crate_id.to_string());
            }
        }
        for i in &mut self.immunities {
            if i.canonical_path.is_none() {
                i.canonical_path = Some(crate_id.to_string());
            }
        }
        for t in &mut self.tolerances {
            if t.canonical_path.is_none() {
                t.canonical_path = Some(crate_id.to_string());
            }
        }
        for e in &mut self.lineage_edges {
            // Both endpoints are stamped to the same crate_id when missing —
            // they're both intra-crate by construction at this point
            // (cross-crate edges land later when D1.5's propagation walk
            // discovers them). Each endpoint is independently None-guarded.
            if e.parent_canonical_path.is_none() {
                e.parent_canonical_path = Some(crate_id.to_string());
            }
            if e.child_canonical_path.is_none() {
                e.child_canonical_path = Some(crate_id.to_string());
            }
        }
    }

    /// Total count of antigen-related declarations found.
    #[must_use]
    pub fn total_declarations(&self) -> usize {
        self.antigens.len()
            + self.presentations.len()
            + self.immunities.len()
            + self.tolerances.len()
    }
}

/// Whether two records share the ADR-017 "same locus" identity.
///
/// Implements the combined locus check from ADR-017
/// `§addresses()` semantics (decisions.md lines 3637-3645):
///
/// - intra-workspace (both `canonical_path` are `None`): same source file
/// - cross-crate (both `Some`): same `canonical_path`
/// - mixed (one `Some`, one `None`): NOT a match — different scan modalities
fn locus_matches(
    a_path: &std::path::Path,
    a_canonical: Option<&str>,
    b_path: &std::path::Path,
    b_canonical: Option<&str>,
) -> bool {
    match (a_canonical, b_canonical) {
        (None, None) => a_path == b_path,
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Does this `Immunity` address this `Presentation`?
///
/// ADR-017 `§addresses()` — combined check of identity (`antigen_type` +
/// `canonical_path`) + item (`ItemTarget::addresses`) + locus.
fn addresses_for_immunity(i: &Immunity, p: &Presentation) -> bool {
    i.antigen_type == p.antigen_type
        && i.canonical_path == p.canonical_path
        && i.item_target.addresses(&p.item_target)
        && locus_matches(
            i.file.as_path(),
            i.canonical_path.as_deref(),
            p.file.as_path(),
            p.canonical_path.as_deref(),
        )
}

/// Does this `Toleration` address this `Presentation`?
fn addresses_for_tolerance(t: &Toleration, p: &Presentation) -> bool {
    t.antigen_type == p.antigen_type
        && t.canonical_path == p.canonical_path
        && t.item_target.addresses(&p.item_target)
        && locus_matches(
            t.file.as_path(),
            t.canonical_path.as_deref(),
            p.file.as_path(),
            p.canonical_path.as_deref(),
        )
}

/// Hard depth limit for `#[descended_from]` lineage chains.
///
/// ADR-005 Amendment 3 (crash-resistance) — bounds pathological-linear
/// chains that exceed reasonable inheritance depth. Default 64; longer chains
/// surface as `parse_failures` rather than letting the propagation walk
/// recurse without bound. The limit is a sibling guard to cycle detection;
/// both are required entry conditions before propagation.
///
/// The constant is internal for v0.1; per the scope-lock document, it will
/// become configurable via `[package.metadata.antigen]` in a follow-up.
pub(crate) const MAX_LINEAGE_DEPTH: usize = 64;

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
fn dedupe_lineage_edges(edges: &[LineageEdge]) -> (Vec<LineageEdge>, Vec<ParseFailure>) {
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
fn detect_lineage_failures(edges: &[LineageEdge], max_depth: usize) -> Vec<ParseFailure> {
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
fn canonicalise_cycle(bare: &[&str]) -> Vec<String> {
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
                let mut visitor = ScanVisitor {
                    file_path: file_path.clone(),
                    report: &mut report,
                    impl_stack: Vec::new(),
                    trait_stack: Vec::new(),
                    current_item_digest: String::new(),
                };
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
                }
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
        synthesis_pass(
            &parsed_files,
            &fingerprints,
            &declaration_sites,
            &mut report,
        );
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
    synthesize_inherited_presentations(&mut report);

    Ok(report)
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
fn synthesize_inherited_presentations(report: &mut ScanReport) {
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
                });
            }
        }
    }
}

// ============================================================================
// Cross-crate enumeration (A3 D3)
//
// Per the A3 scope-lock and navigator's 2026-05-09 ruling: cross-crate scope
// in v0.1 is enumeration + per-crate scanning, NOT merged cross-crate matching.
// The `addresses()` relation stays file-scoped; module-path-qualified
// `ItemTarget` is an ADR-class decision (ATK-A3-005) deferred until aristotle
// rules + an ADR sentence drafts.
//
// Empirical substrate findings (pre-flight P1/P2/P5, 2026-05-09):
//   P1: `cargo metadata --format-version 1` returns `manifest_path` already
//       resolved per-package — no need to construct paths from cargo home +
//       index hash + crate-version suffix. Path-deps, workspace-internal,
//       and registry deps share the same shape.
//   P2: `~/.cargo/registry/src/index.crates.io-<hash>/<crate>-<version>/`
//       hosts multiple co-existing versions of the same crate. The
//       `cargo metadata`-driven approach avoids the multi-version problem
//       entirely because cargo dedupes by version per package.
//   P5: zero `#[antigen(...)]` instances in the wild across the registry
//       (sample: this workspace's 96 reg deps + tambear's 227 reg deps).
//       The collision question is hypothetical until antigen-stdlib lands;
//       Approach 2 vs 3-revised ruling can absorb after D3 ships.
//
// Sub-clause F (ADR-005): cross-crate antigen declarations are trusted
// inputs; the trust anchor is cargo's own checksum verification chain.
// The trust-model ADR sentence is in flight with aristotle.
// ============================================================================

/// How a [`DepCrateRoot`] was sourced — the `cargo metadata` `source` field
/// classified into the buckets the scan tooling cares about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrateOrigin {
    /// `source: null` — workspace-internal package or path-dep (cross- or
    /// in-workspace). Source already lives at `manifest_path`'s parent.
    PathOrWorkspace,
    /// `source: "registry+..."` — a crates.io or alt-registry dependency
    /// downloaded into `~/.cargo/registry/src/<index>/<crate>-<version>/`.
    Registry,
    /// `source: "git+..."` — a git dependency cloned into
    /// `~/.cargo/git/checkouts/`. Captures `manifest_path` directly without
    /// path-construction.
    Git,
    /// Anything else cargo returns we don't classify yet (sparse registries,
    /// alternative registry indices, future cargo source kinds). The raw
    /// source string is preserved so consumers can decide to scan it or not.
    Other(String),
}

impl CrateOrigin {
    fn from_source(source: Option<&str>) -> Self {
        match source {
            None => Self::PathOrWorkspace,
            Some(s) if s.starts_with("registry+") => Self::Registry,
            Some(s) if s.starts_with("git+") => Self::Git,
            Some(s) => Self::Other(s.to_string()),
        }
    }
}

/// A single dependency crate's enumerated source root.
///
/// Returned by [`enumerate_dep_crate_roots`]. The `crate_root` directory is
/// the parent of the package's `Cargo.toml`; passing it to [`scan_workspace`]
/// scans the crate's full source tree. The `package_name` and `version`
/// pair uniquely identifies the dep across the workspace's resolved graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepCrateRoot {
    /// Cargo package name (e.g., `"serde"`, `"antigen-fingerprint"`).
    pub package_name: String,
    /// Cargo package version (e.g., `"1.0.219"`).
    pub version: String,
    /// Directory containing the package's `Cargo.toml` — i.e., the crate
    /// root suitable for [`scan_workspace`].
    pub crate_root: PathBuf,
    /// Where this crate came from. See [`CrateOrigin`].
    pub origin: CrateOrigin,
}

/// Enumerate dependency crates resolved by cargo for the workspace at
/// `workspace_root`.
///
/// Runs `cargo metadata --format-version 1 --manifest-path <workspace>/Cargo.toml`
/// in a subprocess, parses the JSON, and returns one [`DepCrateRoot`] per
/// non-workspace-member package. Workspace-internal members are excluded:
/// when [`scan_workspace`] is called on the workspace root, it already
/// covers them.
///
/// `include_path_workspace` controls whether `CrateOrigin::PathOrWorkspace`
/// dependencies (cross-workspace path-deps) are returned. The default for
/// CLI consumers is `false` — these path-deps usually live alongside the
/// workspace and are scanned independently. Set `true` to opt in.
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - the `cargo` binary cannot be invoked (`PATH` or executable issue),
/// - `cargo metadata` exits non-zero (manifest parse error, lock-file out
///   of date, network failure on first resolve, etc.),
/// - the JSON output cannot be parsed.
///
/// In all error cases, the error message preserves the underlying cause
/// (cargo's stderr or the JSON parse error) for diagnostic surfacing.
///
/// # Sub-clause F note (ADR-005 / ADR-017 trust delegation)
///
/// Cross-crate antigen declarations are trusted inputs — the trust anchor
/// is cargo's own checksum verification of registry sources + git revision
/// pinning. The ADR-017 (draft) trust delegation model requires two
/// preconditions before extending trust to a registry path:
///
/// 1. The path is reachable from `cargo metadata`'s resolution graph as
///    a transitive dependency of the consumer workspace.
/// 2. The path's parent directory matches the registry's expected layout
///    (`<index>/<crate>-<version>/`).
///
/// **Both preconditions are satisfied by construction here**: this function
/// is the only public mechanism for enumerating cross-crate scan targets,
/// and every path it returns is sourced from `cargo metadata`'s output.
/// Cargo verifies registry layout itself before populating that output;
/// we inherit cargo's verification rather than re-implementing it.
///
/// **Discipline for future contributors**: do NOT add a non-cargo-metadata
/// path discovery mechanism (e.g., recursive walking of
/// `~/.cargo/registry/src/`) without explicitly adding the layout-matching
/// and reachability checks. Such a path would extend trust outside cargo's
/// resolution chain. Adversarial ATK-A3-007 (in
/// `antigen/tests/atk_a3_fractal_preview.rs`) is the green-test for that
/// scenario.
pub fn enumerate_dep_crate_roots(
    workspace_root: &Path,
    include_path_workspace: bool,
) -> std::io::Result<Vec<DepCrateRoot>> {
    use std::process::Command;

    let manifest_path = workspace_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "failed to invoke `cargo metadata` at `{}`: {e} \
                     (is cargo on PATH?)",
                    manifest_path.display()
                ),
            )
        })?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "`cargo metadata` exited with status {} for manifest `{}`: {}",
            output.status,
            manifest_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        std::io::Error::other(format!("failed to parse `cargo metadata` JSON output: {e}"))
    })?;

    // Identify workspace-member package IDs so we can exclude them — running
    // scan_workspace on the workspace root already covers these.
    let workspace_members: std::collections::HashSet<String> = metadata
        .get("workspace_members")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let packages = metadata
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            std::io::Error::other(
                "`cargo metadata` output missing `packages` array — unexpected schema",
            )
        })?;

    let mut roots: Vec<DepCrateRoot> = Vec::new();
    for pkg in packages {
        let id = pkg.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        if workspace_members.contains(id) {
            continue;
        }

        let package_name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let version = pkg
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let source = pkg.get("source").and_then(|v| v.as_str());
        let manifest_str = pkg.get("manifest_path").and_then(|v| v.as_str());

        let Some(manifest_str) = manifest_str else {
            // No manifest_path — defensive guard. Skip rather than panic;
            // future cargo schemas may surface unexpected shapes.
            continue;
        };
        let manifest = PathBuf::from(manifest_str);
        let Some(crate_root) = manifest.parent().map(Path::to_path_buf) else {
            continue;
        };

        let origin = CrateOrigin::from_source(source);

        // Path-or-workspace deps: `source` is null. Some are workspace
        // members (already excluded above by id); the rest are path-deps to
        // sibling workspaces (e.g., a consuming crate's path-dep to a
        // separately-maintained antigen workspace checkout). Skip by default
        // — those workspaces are normally scanned on their own — but allow
        // opt-in for full transitive coverage.
        if matches!(origin, CrateOrigin::PathOrWorkspace) && !include_path_workspace {
            continue;
        }

        roots.push(DepCrateRoot {
            package_name,
            version,
            crate_root,
            origin,
        });
    }

    Ok(roots)
}

/// Emit synthetic `FingerprintMatch` presentations for items that match a
/// declared antigen fingerprint but weren't explicitly annotated.
///
/// Called from [`scan_workspace`] after the explicit-collection walk. Uses the
/// cached `(path, syn::File)` pairs from pass 1 — no re-reading or re-parsing.
/// Only top-level items are checked (`syn::File::items`); descent into `impl`
/// methods and `trait` methods is deferred to W6b/A3.
///
/// `declaration_sites` is the set of `(type_name, file)` pairs identifying
/// antigen declaration structs themselves. These are suppressed from
/// fingerprint-match reports — a declaration's own struct always matches its
/// own `doc_contains` fingerprint, producing noise with no signal (DX finding 4).
fn synthesis_pass(
    parsed_files: &[(PathBuf, syn::File)],
    fingerprints: &[(String, antigen_fingerprint::Fingerprint)],
    declaration_sites: &std::collections::HashSet<(String, PathBuf)>,
    report: &mut ScanReport,
) {
    for (file_path, parsed) in parsed_files {
        for syn_item in &parsed.items {
            let Some((kind_str, item_target)) = item_kind_and_target(syn_item) else {
                continue;
            };

            // Node-kind dispatch: skip fingerprints whose top-level item
            // constraint can't match this item's kind — cheap O(1) filter
            // per ADR-010 Amendment 3 Performance Invariant 4.
            let item_kind_for_dispatch = match syn_item {
                syn::Item::Struct(_) => Some(antigen_fingerprint::ItemKind::Struct),
                syn::Item::Enum(_) => Some(antigen_fingerprint::ItemKind::Enum),
                syn::Item::Trait(_) => Some(antigen_fingerprint::ItemKind::Trait),
                syn::Item::Fn(_) => Some(antigen_fingerprint::ItemKind::Fn),
                syn::Item::Impl(_) => Some(antigen_fingerprint::ItemKind::Impl),
                syn::Item::Type(_) => Some(antigen_fingerprint::ItemKind::Type),
                syn::Item::Mod(_) => Some(antigen_fingerprint::ItemKind::Mod),
                _ => None,
            };

            for (antigen_type, fp) in fingerprints {
                // Node-kind dispatch: if the fingerprint pins a required kind,
                // skip evaluation when this item's kind doesn't match.
                if let Some(required_kind) = fp.node_kind() {
                    if item_kind_for_dispatch != Some(required_kind) {
                        continue;
                    }
                }

                if !fp.matches(syn_item) {
                    continue;
                }

                // Self-match suppression: skip when the item IS the antigen's
                // own declaration struct (DX finding 4). The struct that carries
                // #[antigen] always matches its own fingerprint; this match has
                // no signal. Only suppress the exact struct, not other items in
                // the same file that legitimately match the fingerprint.
                let is_self_decl = matches!(&item_target, ItemTarget::Struct(s) if s == antigen_type)
                    && declaration_sites.contains(&(antigen_type.clone(), file_path.clone()));
                if is_self_decl {
                    continue;
                }

                // Deduplication: skip if an explicit #[presents] already covers
                // this (antigen_type, file, item) triple, OR if a tolerance
                // acknowledges the match — tolerated sites belong in the
                // "tolerated" state, not "fingerprint match" (5-state matrix,
                // ADR-001 Amendment 1 Change 2).
                let already_covered = report.presentations.iter().any(|p| {
                    p.match_kind == MatchKind::ExplicitMarker
                        && p.antigen_type == *antigen_type
                        && p.file == *file_path
                        && p.item_target.addresses(&item_target)
                }) || report.tolerances.iter().any(|t| {
                    t.antigen_type == *antigen_type
                        && t.file == *file_path
                        && t.item_target.addresses(&item_target)
                });
                if already_covered {
                    continue;
                }

                // Compute line from the item's first attribute or item span.
                let line = item_line(syn_item);

                let structural_fingerprint = match syn_item {
                    syn::Item::Struct(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Enum(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Trait(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Fn(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Type(i) => antigen_fingerprint::structural_digest(i),
                    syn::Item::Impl(i) => antigen_fingerprint::structural_digest(i),
                    _ => String::new(),
                };

                report.presentations.push(Presentation {
                    antigen_type: antigen_type.clone(),
                    file: file_path.clone(),
                    line,
                    item_kind: kind_str.to_string(),
                    item_target: item_target.clone(),
                    match_kind: MatchKind::FingerprintMatch,
                    canonical_path: None,
                    inherited_from: None,
                    structural_fingerprint,
                });
            }
        }
    }
}

/// Build a `(kind_str, ItemTarget)` pair from a top-level `syn::Item`.
/// Returns `None` for item kinds we don't model (macros, extern crates, etc.).
fn item_kind_and_target(item: &syn::Item) -> Option<(&'static str, ItemTarget)> {
    match item {
        syn::Item::Struct(s) => Some(("struct", ItemTarget::Struct(s.ident.to_string()))),
        syn::Item::Enum(e) => Some(("enum", ItemTarget::Enum(e.ident.to_string()))),
        syn::Item::Trait(t) => Some(("trait", ItemTarget::Trait(t.ident.to_string()))),
        syn::Item::Fn(f) => Some(("fn", ItemTarget::Fn(f.sig.ident.to_string()))),
        syn::Item::Type(t) => Some(("type", ItemTarget::TypeAlias(t.ident.to_string()))),
        syn::Item::Impl(i) => {
            let trait_path = i.trait_.as_ref().map(|(_, path, _)| render_path(path));
            let target_type = render_type(&i.self_ty);
            Some((
                "impl",
                ItemTarget::Impl {
                    trait_path,
                    target_type,
                },
            ))
        }
        // `mod` items are not yet modeled in `ItemTarget`; skip for synthesis.
        _ => None,
    }
}

/// Best-effort line number for a top-level `syn::Item` (line of its first
/// attribute if any, else the item's own span start).
fn item_line(item: &syn::Item) -> usize {
    use syn::spanned::Spanned;
    item.span().start().line
}

/// AST visitor that extracts antigen-related attributes.
#[presents(ScanVisitorDigestAssignmentOmission)]
struct ScanVisitor<'a> {
    file_path: PathBuf,
    report: &'a mut ScanReport,
    /// Context stack for nested items. The current top of stack is the
    /// enclosing-impl context for any `visit_impl_item_fn` call — so that
    /// a method's `ItemTarget::ImplFn` knows which impl block it lives in.
    /// W3 (sweep A2): structural item-identity tracking.
    impl_stack: Vec<(Option<String>, String)>,
    /// Context stack for nested traits — analogous to `impl_stack`, but
    /// for `visit_trait_item_fn` so trait methods carry the enclosing
    /// trait identifier in `ItemTarget::TraitFn`.
    trait_stack: Vec<String>,
    /// Structural digest of the item currently being visited, set by each
    /// `visit_item_*` before it calls [`Self::check_attrs`] so that
    /// `extract_immune` / `extract_tolerance` can stamp the defended item's
    /// digest onto the substrate-witness record without threading it through
    /// every `check_attrs` call site. Empty between items.
    current_item_digest: String,
}

impl ScanVisitor<'_> {
    /// Compute 1-indexed line number for a span by counting newlines in source up
    /// to the span's start.
    ///
    /// Resolve the source line of a specific `#[attr]` invocation via
    /// `syn::spanned::Spanned::span().start().line`. Each per-instance call
    /// reports the line of *that* invocation rather than the first match in
    /// the file (the pre-fix heuristic that broke ATK-A2-002 for multi-
    /// instance scenarios).
    ///
    /// Falls back to `0` only if the span info is unavailable (which on
    /// stable rustc with `proc-macro2`'s default features is rare; a 0
    /// return means "we don't know," which is honest).
    fn line_of_attr(attr: &syn::Attribute) -> usize {
        use syn::spanned::Spanned;
        attr.span().start().line
    }

    fn extract_antigen(&mut self, item: &syn::ItemStruct, attr: &syn::Attribute) {
        let type_name = item.ident.to_string();
        let line = Self::line_of_attr(attr);

        if let syn::Meta::List(list) = &attr.meta {
            match syn::parse2::<ScanAntigenArgs>(list.tokens.clone()) {
                Ok(args) => {
                    let category: Vec<crate::category::AntigenCategory> = args
                        .category
                        .iter()
                        .filter_map(|s| crate::category::AntigenCategory::parse_category(s))
                        .collect();
                    self.report.antigens.push(AntigenDeclaration {
                        name: args.name,
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: args.family,
                        summary: args.summary,
                        fingerprint: args.fingerprint,
                        canonical_path: None,
                        category,
                    });
                }
                Err(_) => {
                    // Malformed attribute: record with empty name so scan output
                    // surfaces the file for investigation rather than silently skipping.
                    self.report.antigens.push(AntigenDeclaration {
                        name: String::new(),
                        type_name,
                        file: self.file_path.clone(),
                        line,
                        family: None,
                        summary: None,
                        fingerprint: None,
                        canonical_path: None,
                        category: Vec::new(),
                    });
                }
            }
        }
    }

    fn extract_presents(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        let antigen_type = if let syn::Meta::List(list) = &attr.meta {
            // Parse the body as a `syn::Path` rather than splitting the
            // `quote::ToTokens` rendering on `::`. The string form contains
            // whitespace artifacts (`" my_crate :: Foo "`) that the prior
            // tactical-fix code recovered from with a `.trim()` — but the
            // structural fix is to never produce the string in the first
            // place. ATK-A2-001's pre-W3 hotfix landed in commit b9440b2;
            // this is the W3 structural form. Path's last segment is the
            // bare type name regardless of qualifier.
            match syn::parse2::<syn::Path>(list.tokens.clone()) {
                Ok(path) => path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default(),
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[presents] attribute: {e}"),
                    });
                    return;
                }
            }
        } else {
            return;
        };
        let line = Self::line_of_attr(attr);
        self.report.presentations.push(Presentation {
            antigen_type,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
            match_kind: MatchKind::ExplicitMarker,
            canonical_path: None,
            inherited_from: None,
            structural_fingerprint: self.current_item_digest.clone(),
        });
    }

    fn extract_immune(
        &mut self,
        attr: &syn::Attribute,
        all_attrs: &[syn::Attribute],
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            // Scan records the witness expression verbatim; validity
            // classification (Test, Proptest, PhantomType, Function, External)
            // and behavioral verification (cargo test invocation) are the
            // audit module's responsibility. ADR-005 sub-clause F: the
            // trust boundary at "immunity claim" is checked by audit, not
            // by scan — scan provides the substrate, audit decides validity.
            let args = match syn::parse2::<ScanImmuneArgs>(list.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    // Malformed #[immune] args: record a parse failure rather
                    // than silently inserting a ghost immunity record with empty
                    // antigen_type and witness. A ghost record would pass
                    // WitnessStatus::Missing detection only if the empty-string
                    // check fires, and would produce a misleading "0 unaddressed
                    // presentations" result. ADR-005: every trust boundary requires
                    // a validation check; malformed immunity claims are not claims.
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[immune] attribute: {e}"),
                    });
                    return;
                }
            };
            // ADR-019 §P3b: substrate-witness discovery has two channels.
            // The primary channel parses `requires = <predicate>` directly
            // from the source attribute (`args.requires_predicate`). The
            // fallback channel reads the `antigen:requires:v1:<json>` doc
            // marker the macro emits — useful when scanning crates already
            // compiled with rc.1 macros, or in any case the source attribute
            // didn't survive a build-script rewrite. Source wins because the
            // doc marker is only present POST macro expansion, and `syn`
            // parses the WRITTEN source. This is the rc.2 fix: rc.1 relied
            // exclusively on the doc-marker channel, which never engaged
            // because scan walks written source.
            let requires_predicate = args
                .requires_predicate
                .clone()
                .or_else(|| extract_requires_predicate_from_attrs(all_attrs));
            let line = Self::line_of_attr(attr);
            self.report.immunities.push(Immunity {
                antigen_type: args.antigen_type,
                witness: args.witness,
                requires_predicate,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                canonical_path: None,
                structural_fingerprint: self.current_item_digest.clone(),
            });
        }
    }

    fn extract_tolerance(
        &mut self,
        attr: &syn::Attribute,
        all_attrs: &[syn::Attribute],
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanToleranceArgs>(list.tokens.clone()) {
                Ok(args) => args,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[antigen_tolerance] attribute: {e}"),
                    });
                    return;
                }
            };
            // Per ADR-011 §Mechanics §1: rationale required + non-empty.
            // Scan side enforces the same boundary the macro enforces — a
            // tolerance without rationale is silent suppression.
            if args.rationale.is_empty() {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: "#[antigen_tolerance] requires non-empty rationale".to_string(),
                });
                return;
            }
            // Same two-channel discovery as immunity (source-attr primary,
            // doc-marker fallback). See `extract_immune` for the full
            // rationale — this branch is the tolerance-side mirror.
            let requires_predicate = args
                .requires_predicate
                .clone()
                .or_else(|| extract_requires_predicate_from_attrs(all_attrs));
            let line = Self::line_of_attr(attr);
            self.report.tolerances.push(Toleration {
                antigen_type: args.antigen_type,
                rationale: args.rationale,
                until: args.until,
                see: args.see,
                requires_predicate,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
                canonical_path: None,
                structural_fingerprint: self.current_item_digest.clone(),
            });
        }
    }

    // ============================================================================
    // Deferred-Defense Family extraction methods (ADR-023)
    // ============================================================================

    fn extract_anergy(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanAnergyArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[anergy] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.deferred_defenses.push(DeferredDefense {
                kind: DeferredDefenseKind::Anergy,
                antigen_type: args.antigen_type,
                text: args.reason,
                until: if args.until.is_empty() {
                    None
                } else {
                    Some(args.until)
                },
                expected_co_stimulation: args.expected_co_stimulation,
                signed_by: args.signed_by,
                see: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_immunosuppress(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanImmunosuppressArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[immunosuppress] attribute: {e}"),
                    });
                    return;
                }
            };
            let mut see = Vec::new();
            if let Some(since) = &args.since {
                // Store since as a see-ref for audit correlation
                see.push(format!("since:{since}"));
            }
            if let Some(cap) = args.duration_cap {
                see.push(format!("duration_cap:{cap}d"));
            }
            let line = Self::line_of_attr(attr);
            self.report.deferred_defenses.push(DeferredDefense {
                kind: DeferredDefenseKind::Immunosuppress,
                antigen_type: args.antigen_type,
                text: args.rationale,
                until: if args.until.is_empty() {
                    None
                } else {
                    Some(args.until)
                },
                expected_co_stimulation: None,
                signed_by: args.signed_by,
                see,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_poxparty(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanPoxpartyArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[poxparty] attribute: {e}"),
                    });
                    return;
                }
            };
            let mut see = Vec::new();
            if let Some(name) = &args.name {
                see.push(format!("exercise:{name}"));
            }
            if let Some(rationale) = &args.rationale {
                see.push(format!("rationale:{rationale}"));
            }
            let line = Self::line_of_attr(attr);
            self.report.deferred_defenses.push(DeferredDefense {
                kind: DeferredDefenseKind::Poxparty,
                antigen_type: args.antigen_type,
                text: args.exercise_type,
                until: if args.until.is_empty() {
                    None
                } else {
                    Some(args.until)
                },
                expected_co_stimulation: None,
                signed_by: args.signed_by,
                see,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_orient(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        // #[orient] with no args (bare attribute) is valid — acknowledge
        // orientation period with zero configuration.
        match &attr.meta {
            syn::Meta::List(list) => {
                let args = match syn::parse2::<ScanOrientArgs>(list.tokens.clone()) {
                    Ok(a) => a,
                    Err(e) => {
                        self.report.parse_failures.push(ParseFailure {
                            file: self.file_path.clone(),
                            error: format!("malformed #[orient] attribute: {e}"),
                        });
                        return;
                    }
                };
                let line = Self::line_of_attr(attr);
                let mut adr_see = args.see.clone();
                if let Some(adr) = &args.adr {
                    adr_see.push(format!("adr:{adr}"));
                }
                self.report.deferred_defenses.push(DeferredDefense {
                    kind: DeferredDefenseKind::Orient,
                    antigen_type: args.antigen_type,
                    text: String::new(),
                    until: None,
                    expected_co_stimulation: None,
                    signed_by: None,
                    see: adr_see,
                    file: self.file_path.clone(),
                    line,
                    item_kind: item_kind.to_string(),
                    item_target,
                });
            }
            syn::Meta::Path(_) => {
                // Bare `#[orient]` — valid, record with empty fields.
                let line = Self::line_of_attr(attr);
                self.report.deferred_defenses.push(DeferredDefense {
                    kind: DeferredDefenseKind::Orient,
                    antigen_type: None,
                    text: String::new(),
                    until: None,
                    expected_co_stimulation: None,
                    signed_by: None,
                    see: Vec::new(),
                    file: self.file_path.clone(),
                    line,
                    item_kind: item_kind.to_string(),
                    item_target,
                });
            }
            syn::Meta::NameValue(_) => {
                // `#[orient = value]` is not a valid orient invocation; ignore.
            }
        }
    }

    fn extract_descended_from(&mut self, attr: &syn::Attribute, item_target: &ItemTarget) {
        // ADR-013: `#[descended_from]` is meaningful only on antigen-type
        // declarations (unit `struct` and class-shaped `enum`). Other
        // placements — impl blocks, free functions, traits, methods —
        // surface as parse_failures so the user sees what got dropped
        // rather than the visitor silently no-op'ing them.
        let child = match item_target {
            ItemTarget::Struct(name) | ItemTarget::Enum(name) => name.clone(),
            other => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!(
                        "#[descended_from] on `{}` is not a type declaration; \
                         this attribute is meaningful only on `struct` and `enum` \
                         antigen declarations",
                        other.label()
                    ),
                });
                return;
            }
        };

        let syn::Meta::List(list) = &attr.meta else {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "malformed #[descended_from] attribute: expected `(parent)`".to_string(),
            });
            return;
        };

        // Body is a single positional `syn::Path`, mirroring
        // `extract_presents`. Last segment becomes the bare parent type
        // name — module-path qualification is an A3+ ADR-class question
        // (ATK-A3-005), so for now we keep names bare.
        let parent = match syn::parse2::<syn::Path>(list.tokens.clone()) {
            Ok(path) => path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            Err(e) => {
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!("malformed #[descended_from] attribute: {e}"),
                });
                return;
            }
        };

        if parent.is_empty() {
            self.report.parse_failures.push(ParseFailure {
                file: self.file_path.clone(),
                error: "#[descended_from] requires a parent path argument".to_string(),
            });
            return;
        }

        let line = Self::line_of_attr(attr);
        self.report.lineage_edges.push(LineageEdge {
            child,
            parent,
            file: self.file_path.clone(),
            line,
            parent_canonical_path: None,
            child_canonical_path: None,
        });
    }

    fn check_attrs(&mut self, attrs: &[syn::Attribute], item_kind: &str, item_target: &ItemTarget) {
        for attr in attrs {
            if attr_is(attr, "presents") {
                self.extract_presents(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "immune") {
                self.extract_immune(attr, attrs, item_kind, item_target.clone());
            } else if attr_is(attr, "antigen_tolerance") {
                self.extract_tolerance(attr, attrs, item_kind, item_target.clone());
            } else if attr_is(attr, "descended_from") {
                self.extract_descended_from(attr, item_target);
            // Deferred-Defense Family (ADR-023)
            } else if attr_is(attr, "anergy") {
                self.extract_anergy(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "immunosuppress") {
                self.extract_immunosuppress(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "poxparty") {
                self.extract_poxparty(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "orient") {
                self.extract_orient(attr, item_kind, item_target.clone());
            // Convergent-Evidence Family (ADR-024)
            } else if attr_is(attr, "diagnostic") {
                self.extract_diagnostic(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "clonal") {
                self.extract_clonal(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "igg") {
                self.extract_igg(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "crossreactive") {
                self.extract_crossreactive(attr, item_kind, item_target.clone());
            } else if attr_is(attr, "polyclonal") {
                self.extract_convergent_marker(
                    attr,
                    item_kind,
                    item_target.clone(),
                    ConvergentEvidenceKind::Polyclonal,
                );
            } else if attr_is(attr, "monoclonal") {
                self.extract_convergent_marker(
                    attr,
                    item_kind,
                    item_target.clone(),
                    ConvergentEvidenceKind::Monoclonal,
                );
            } else if attr_is(attr, "adcc") {
                self.extract_convergent_marker(
                    attr,
                    item_kind,
                    item_target.clone(),
                    ConvergentEvidenceKind::Adcc,
                );
            } else {
                // v0.2 families (recurrent-emergence + mucosal-boundary)
                // dispatch in a sibling helper to keep check_attrs concise.
                self.check_v02_family_attr(attr, item_kind, item_target);
            }
        }
    }

    /// Dispatch the v0.2 recurrent-emergence + mucosal-boundary attribute
    /// families (ADR-024 §Family 2, ADR-027). Split out of `check_attrs` so
    /// the primary attribute matcher stays readable.
    fn check_v02_family_attr(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: &ItemTarget,
    ) {
        // Recurrent-Emergence Family (ADR-024 §Family 2)
        if attr_is(attr, "itch") {
            self.extract_recurrent(attr, item_kind, item_target.clone(), RecurrentKind::Itch);
        } else if attr_is(attr, "recurrence_anchor") {
            self.extract_recurrent(
                attr,
                item_kind,
                item_target.clone(),
                RecurrentKind::RecurrenceAnchor,
            );
        } else if attr_is(attr, "crystallize") {
            self.extract_recurrent(
                attr,
                item_kind,
                item_target.clone(),
                RecurrentKind::Crystallize,
            );
        } else if attr_is(attr, "chronic") {
            self.extract_recurrent(attr, item_kind, item_target.clone(), RecurrentKind::Chronic);
        } else if attr_is(attr, "saturate") {
            self.extract_recurrent(
                attr,
                item_kind,
                item_target.clone(),
                RecurrentKind::Saturate,
            );
        } else if attr_is(attr, "strand") {
            self.extract_recurrent(attr, item_kind, item_target.clone(), RecurrentKind::Strand);
        // Mucosal Boundary Family (ADR-027 + Amendment 1)
        } else if attr_is(attr, "mucosal") {
            self.extract_mucosal(
                attr,
                item_kind,
                item_target.clone(),
                MucosalKindTag::Mucosal,
            );
        } else if attr_is(attr, "mucosal_delegate") {
            self.extract_mucosal(
                attr,
                item_kind,
                item_target.clone(),
                MucosalKindTag::MucosalDelegate,
            );
        } else if attr_is(attr, "mucosal_tolerant") {
            self.extract_mucosal(
                attr,
                item_kind,
                item_target.clone(),
                MucosalKindTag::MucosalTolerant,
            );
        }
    }

    /// Scan-extract a mucosal-boundary declaration (ADR-027 + Amendment 1).
    /// All three primitives share the loosely-typed `ScanMucosalArgs`
    /// capture; per-primitive required-field + delegate-kind-matching
    /// validation is the audit layer's job (Change 5 three-tier diagnosis).
    fn extract_mucosal(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        tag: MucosalKindTag,
    ) {
        let line = Self::line_of_attr(attr);
        let args = match &attr.meta {
            syn::Meta::List(list) => match syn::parse2::<ScanMucosalArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed mucosal-boundary attribute: {e}"),
                    });
                    return;
                }
            },
            syn::Meta::Path(_) => ScanMucosalArgs::default(),
            syn::Meta::NameValue(_) => return,
        };
        self.report.mucosal_declarations.push(MucosalDeclaration {
            tag,
            boundary_kind: args.boundary_kind,
            rationale: args.rationale,
            handled_by: args.handled_by,
            accepts: args.accepts,
            reviewed_by: args.reviewed_by,
            until: args.until,
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
        });
    }

    /// Scan-extract a recurrent-emergence declaration (ADR-024 §Family 2).
    ///
    /// All six primitives share the loosely-typed `ScanRecurrentArgs` capture
    /// (mirroring `ScanAntigenArgs`'s forward-compat posture per ADR-009).
    /// The `kind` discriminant is supplied by the dispatch site; per-kind
    /// required-field validation is the audit layer's job, not scan's
    /// (scan is recall-tuned per ADR-010; precision lives in audit).
    fn extract_recurrent(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        kind: RecurrentKind,
    ) {
        let line = Self::line_of_attr(attr);
        let args = match &attr.meta {
            syn::Meta::List(list) => match syn::parse2::<ScanRecurrentArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed recurrent-emergence attribute: {e}"),
                    });
                    return;
                }
            },
            // Bare `#[chronic]` etc. without args — recall it with empty
            // fields; audit surfaces the missing-required-field condition.
            syn::Meta::Path(_) => ScanRecurrentArgs::default(),
            syn::Meta::NameValue(_) => return,
        };
        self.report
            .recurrent_declarations
            .push(RecurrentDeclaration {
                kind,
                name: args.name,
                antigen_type: args.antigen_type,
                description: args.description,
                instances: args.instances,
                since: args.since,
                rationale: args.rationale,
                from_itches: args.from_itches,
                anchored_by: args.anchored_by,
                managed_by: args.managed_by,
                contributing_to: args.contributing_to,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
    }

    fn extract_diagnostic(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanDiagnosticArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[diagnostic] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Diagnostic,
                modality_classes: args.modality_classes,
                min_independent: args.min_independent,
                witness: None,
                iterations: None,
                seed_kind: None,
                historical_span: None,
                min_reattestations: None,
                witnesses: Vec::new(),
                fingerprints: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_clonal(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanClonalArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[clonal] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Clonal,
                modality_classes: Vec::new(),
                min_independent: None,
                witness: args.witness,
                iterations: args.iterations,
                seed_kind: args.seed_kind,
                historical_span: None,
                min_reattestations: None,
                witnesses: Vec::new(),
                fingerprints: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_igg(&mut self, attr: &syn::Attribute, item_kind: &str, item_target: ItemTarget) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanIggArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[igg] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Igg,
                modality_classes: Vec::new(),
                min_independent: None,
                witness: None,
                iterations: None,
                seed_kind: None,
                historical_span: args.historical_span,
                min_reattestations: args.min_reattestations,
                witnesses: args.witnesses,
                fingerprints: Vec::new(),
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    fn extract_crossreactive(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
    ) {
        if let syn::Meta::List(list) = &attr.meta {
            let args = match syn::parse2::<ScanCrossreactiveArgs>(list.tokens.clone()) {
                Ok(a) => a,
                Err(e) => {
                    self.report.parse_failures.push(ParseFailure {
                        file: self.file_path.clone(),
                        error: format!("malformed #[crossreactive] attribute: {e}"),
                    });
                    return;
                }
            };
            let line = Self::line_of_attr(attr);
            self.report.convergent_evidences.push(ConvergentEvidence {
                kind: ConvergentEvidenceKind::Crossreactive,
                modality_classes: Vec::new(),
                min_independent: None,
                witness: None,
                iterations: None,
                seed_kind: None,
                historical_span: None,
                min_reattestations: None,
                witnesses: Vec::new(),
                fingerprints: args.fingerprints,
                file: self.file_path.clone(),
                line,
                item_kind: item_kind.to_string(),
                item_target,
            });
        }
    }

    /// Common extractor for the three marker primitives (no required
    /// args): `#[polyclonal]`, `#[monoclonal]`, `#[adcc]`. Records the
    /// site with `kind = <kind>` and all other fields default.
    fn extract_convergent_marker(
        &mut self,
        attr: &syn::Attribute,
        item_kind: &str,
        item_target: ItemTarget,
        kind: ConvergentEvidenceKind,
    ) {
        let line = Self::line_of_attr(attr);
        self.report.convergent_evidences.push(ConvergentEvidence {
            kind,
            modality_classes: Vec::new(),
            min_independent: None,
            witness: None,
            iterations: None,
            seed_kind: None,
            historical_span: None,
            min_reattestations: None,
            witnesses: Vec::new(),
            fingerprints: Vec::new(),
            file: self.file_path.clone(),
            line,
            item_kind: item_kind.to_string(),
            item_target,
        });
    }
}

/// Render a `syn::Type` to its canonical token-stream string. Used to
/// extract a string identifier for `impl Trait for Type` blocks. The
/// rendering normalizes whitespace via `quote::ToTokens`. For W3 we only
/// need a stable string for equality matching — A3 cross-crate work will
/// likely want a richer canonical form (e.g., resolved module paths).
fn render_type(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string()
}

/// Whether an attribute's path matches a given antigen attribute name.
///
/// `syn::Path::is_ident("X")` only returns true for single-segment paths.
/// Path-qualified attribute forms — `#[antigen::immune(...)]`,
/// `#[crate::presents(...)]`, `#[my::module::antigen(...)]` — produce
/// multi-segment paths that `is_ident` rejects, causing the scan to
/// silently drop them. The fix: an attribute's path matches `name`
/// either when it's the bare ident, OR when its *last segment* is the
/// ident.
///
/// This is the path-segment-aware analog of `is_ident` and is the only
/// matcher used inside `ScanVisitor`. Using last-segment equality is
/// cheap and the same heuristic Rust itself uses to find the macro
/// being invoked — name resolution happens elsewhere.
fn attr_is(attr: &syn::Attribute, name: &str) -> bool {
    let path = attr.path();
    path.is_ident(name) || path.segments.last().is_some_and(|s| s.ident == name)
}

/// Extract the `antigen:requires:v1:<json>` predicate from a sibling doc attr.
///
/// The `#[immune(requires = ...)]` macro (P3b) emits:
///   `#[doc = " antigen:requires:v1:<json>"]`
/// as a sibling attribute on the annotated item. Scan finds it by looking
/// for a doc attribute whose string value starts with the marker prefix.
fn extract_requires_predicate_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    const MARKER_PREFIX: &str = "antigen:requires:v1:";
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &nv.value
            {
                let val = s.value();
                let trimmed = val.trim();
                if let Some(json) = trimmed.strip_prefix(MARKER_PREFIX) {
                    return Some(json.to_string());
                }
            }
        }
    }
    None
}

/// Render a `syn::Path` similarly. Used for the trait portion of
/// `impl Trait for Type` so that `Drop` and `core::ops::Drop` produce
/// distinct strings (which is correct — they're different items in
/// Rust's name resolution, even when they alias).
fn render_path(path: &syn::Path) -> String {
    use quote::ToTokens;
    path.to_token_stream().to_string()
}

impl<'ast> Visit<'ast> for ScanVisitor<'_> {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        for attr in &item.attrs {
            if attr_is(attr, "antigen") {
                self.extract_antigen(item, attr);
            }
        }
        let target = ItemTarget::Struct(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "struct", &target);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let trait_path = item.trait_.as_ref().map(|(_, path, _)| render_path(path));
        let target_type = render_type(&item.self_ty);
        let target = ItemTarget::Impl {
            trait_path: trait_path.clone(),
            target_type: target_type.clone(),
        };
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl", &target);
        // Push impl context so visit_impl_item_fn can build ImplFn targets.
        self.impl_stack.push((trait_path, target_type));
        syn::visit::visit_item_impl(self, item);
        self.impl_stack.pop();
    }

    fn visit_item_const(&mut self, item: &'ast syn::ItemConst) {
        // ATK-A2-TOPLEVEL-CONST: route a free-standing const's attrs through
        // check_attrs so `#[presents]` on a top-level/module const is not
        // silently ignored (same blind-spot class as enum variants + impl
        // consts).
        let target = ItemTarget::Const(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "const", &target);
        syn::visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast syn::ItemStatic) {
        // Same blind-spot class as visit_item_const: route a free-standing
        // `static`'s attrs through check_attrs so `#[presents]` on it is not
        // silently ignored. Closed preemptively (ADR-007) — the fixture
        // atk_a2_static_presents proves the need.
        let target = ItemTarget::Static(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "static", &target);
        syn::visit::visit_item_static(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let target = ItemTarget::Fn(item.sig.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "fn", &target);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |(trait_path, target_type)| ItemTarget::ImplFn {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_fn", &target);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        // ATK-A2-IMPL-CONST: route an associated const's attrs through
        // check_attrs so `#[presents]` on an impl-block const is not silently
        // ignored (the same blind-spot as enum variants). Falls back to a bare
        // Fn target if somehow visited outside an impl (shouldn't happen).
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(item.ident.to_string()),
            |(trait_path, target_type)| ItemTarget::ImplConst {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                const_name: item.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_const", &target);
        syn::visit::visit_impl_item_const(self, item);
    }

    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        // ATK-A2-IMPL-ITEM-TYPE: an impl-block associated type
        // (`type Foo = Bar;`) carries attrs too — `#[presents]` on it was
        // silently dropped (same blind-spot class as impl_item_const). Route it
        // through check_attrs. Target is the associated-type name (reusing
        // TypeAlias rather than minting a near-duplicate variant, mirroring how
        // visit_trait_item_const reuses ImplConst).
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_type", &target);
        syn::visit::visit_impl_item_type(self, item);
    }

    fn visit_impl_item_macro(&mut self, item: &'ast syn::ImplItemMacro) {
        // ATK-A2-IMPL-ITEM-MACRO: route a macro invocation inside an impl block
        // through check_attrs so `#[presents]` on patterns like
        // `#[presents(X)] delegate!()` is not silently ignored.
        // Same blind-spot class as impl_item_fn/const/type — the attrs field
        // exists and is valid, but without this override it is never visited.
        let mac_name = item
            .mac
            .path
            .segments
            .last()
            .map_or_else(|| "(macro)".to_string(), |s| s.ident.to_string());
        let target = self.impl_stack.last().map_or_else(
            || ItemTarget::Fn(mac_name.clone()),
            |(trait_path, target_type)| ItemTarget::ImplConst {
                trait_path: trait_path.clone(),
                target_type: target_type.clone(),
                const_name: mac_name.clone(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "impl_macro", &target);
        syn::visit::visit_impl_item_macro(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let target = ItemTarget::Trait(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait", &target);
        // Push trait context so visit_trait_item_fn produces TraitFn targets
        // identifying the enclosing trait.
        self.trait_stack.push(item.ident.to_string());
        syn::visit::visit_item_trait(self, item);
        self.trait_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Fn(item.sig.ident.to_string()),
            |trait_name| ItemTarget::TraitFn {
                trait_name: trait_name.clone(),
                fn_name: item.sig.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_fn", &target);
        syn::visit::visit_trait_item_fn(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        // Same blind-spot class as the impl/top-level const cases: route a
        // trait-associated const's attrs through check_attrs. Reuses
        // ItemTarget::ImplConst with the trait as the target type (an
        // associated const on a named type/trait) to avoid a near-duplicate
        // variant — label renders as `Trait::CONST`.
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Const(item.ident.to_string()),
            |trait_name| ItemTarget::ImplConst {
                trait_path: None,
                target_type: trait_name.clone(),
                const_name: item.ident.to_string(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_const", &target);
        syn::visit::visit_trait_item_const(self, item);
    }

    fn visit_trait_item_type(&mut self, item: &'ast syn::TraitItemType) {
        // ATK-A2-TRAIT-ITEM-TYPE: a trait associated-type declaration
        // (`type Item;`) carries attrs too — `#[presents]` on it was silently
        // dropped (same blind-spot class as trait_item_const). These are real
        // contract sites (e.g. a mucosal boundary like `Iterator::Item`). Route
        // through check_attrs with the associated-type name as target.
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_type", &target);
        syn::visit::visit_trait_item_type(self, item);
    }

    fn visit_trait_item_macro(&mut self, item: &'ast syn::TraitItemMacro) {
        // ATK-A2-TRAIT-ITEM-MACRO: route a macro invocation inside a trait body
        // through check_attrs so `#[presents]` on trait-body macro expansions
        // (blanket-impl helpers, proc-macro trait-body generators) is not silently
        // ignored. Same blind-spot class as trait_item_fn/const/type.
        let mac_name = item
            .mac
            .path
            .segments
            .last()
            .map_or_else(|| "(macro)".to_string(), |s| s.ident.to_string());
        let target = self.trait_stack.last().map_or_else(
            || ItemTarget::Fn(mac_name.clone()),
            |trait_name| ItemTarget::TraitFn {
                trait_name: trait_name.clone(),
                fn_name: mac_name.clone(),
            },
        );
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait_macro", &target);
        syn::visit::visit_trait_item_macro(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast syn::ItemType) {
        // Type aliases (`type Foo = ...;`) carry attributes too. ATK-W3-005:
        // without this handler, attributes on type aliases would fall back
        // to ItemTarget::Unknown, and two unrelated Unknown items collide
        // on equality. Tracking the alias name keeps each alias as its own
        // distinct match target.
        let target = ItemTarget::TypeAlias(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "type_alias", &target);
        syn::visit::visit_item_type(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        for attr in &item.attrs {
            if attr_is(attr, "antigen") {
                // ATK-A2-007: silently dropping #[antigen] on enums eats the
                // class-enum pattern (the frame-translation antigen's primary
                // use case). Surface the situation as a parse_failure so the
                // user sees it, rather than the previous `let _ = attr` no-op.
                // The macro itself still rejects non-unit structs at compile
                // time; this scan-side diagnostic catches enum cases that
                // wouldn't reach the macro (e.g., a user investigating "why
                // doesn't my class enum scan as an antigen?").
                self.report.parse_failures.push(ParseFailure {
                    file: self.file_path.clone(),
                    error: format!(
                        "#[antigen] on enum `{}` is not supported in v0.1; \
                         antigen declarations must be unit structs (e.g., \
                         `pub struct {};`). Enum-shaped failure-classes are \
                         tracked by ADR-010 Amendment 1's class-enum operator \
                         in a future grammar version.",
                        item.ident, item.ident
                    ),
                });
            }
        }
        let target = ItemTarget::Enum(item.ident.to_string());
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "enum", &target);

        // ATK-A2-ENUM-VARIANT: descend into variants so a variant-level
        // attribute (e.g. `#[presents(X)]` on one variant) is not silently
        // ignored. `syn::visit::visit_item_enum` walks the variants but never
        // routes their attrs through `check_attrs`, so without this loop the
        // presentation is invisible to failure-class memory. The enclosing-enum
        // digest stands in for each variant (a variant has no independent
        // structural digest of its own).
        let enum_name = item.ident.to_string();
        for variant in &item.variants {
            let variant_target = ItemTarget::EnumVariant {
                enum_name: enum_name.clone(),
                variant_name: variant.ident.to_string(),
            };
            self.check_attrs(&variant.attrs, "enum_variant", &variant_target);
        }

        syn::visit::visit_item_enum(self, item);
    }

    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        // ATK-A2-MACRO-RULES: route a macro_rules! item's attrs through
        // check_attrs so #[presents] on a macro definition is not silently
        // ignored. Same blind-spot class as enum variants and impl consts.
        // ItemTarget::Const reuses an existing string-carrying target variant;
        // the name is the macro identifier or "(anonymous)" for unnamed macros.
        let name = item
            .ident
            .as_ref()
            .map_or_else(|| "(anonymous)".to_string(), ToString::to_string);
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "macro", &target);
        syn::visit::visit_item_macro(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        // ATK-A2-USE-ITEM: route a use/re-export item's attrs through check_attrs
        // so #[presents] on a use declaration (e.g. a dangerous capability re-export
        // at a trust boundary) is not silently ignored. Same blind-spot class as
        // macro_rules! (above), enum variants, and impl consts.
        use quote::ToTokens;
        let path_str = item.tree.to_token_stream().to_string();
        let target = ItemTarget::Const(path_str);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "use", &target);
        syn::visit::visit_item_use(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "extern crate", &target);
        syn::visit::visit_item_extern_crate(self, item);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast syn::ItemForeignMod) {
        use quote::ToTokens;
        let abi_str = item.abi.to_token_stream().to_string();
        let target = ItemTarget::Const(abi_str);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "foreign mod", &target);
        syn::visit::visit_item_foreign_mod(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "mod", &target);
        syn::visit::visit_item_mod(self, item);
    }

    fn visit_item_trait_alias(&mut self, item: &'ast syn::ItemTraitAlias) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "trait alias", &target);
        syn::visit::visit_item_trait_alias(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        let name = item.ident.to_string();
        let target = ItemTarget::Const(name);
        self.current_item_digest = antigen_fingerprint::structural_digest(item);
        self.check_attrs(&item.attrs, "union", &target);
        syn::visit::visit_item_union(self, item);
    }
}

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
}

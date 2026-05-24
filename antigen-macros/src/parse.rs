//! Argument parsing for the antigen attribute macros.
//!
//! ## Span discipline (W4)
//!
//! Validation errors point at the offending token, not the whole macro
//! invocation. Each parsed field carries its own `proc_macro2::Span` (the
//! span of the *value* literal, e.g., the `""` in `name = ""`). For
//! missing-required-field errors there is no offending token — those errors
//! are anchored at `args_span`, the span of the macro's argument list. This
//! is consistently better than `Span::call_site()`, which points at the
//! whole `#[antigen(...)]` invocation.

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, Lit, LitStr, Path, Token};

// ============================================================================
// RequiresExpr is now defined in antigen-attestation behind the `parser`
// feature so the scan layer can re-use the same parser without depending on
// macro expansion. See `antigen_attestation::parser` for the implementation
// and the JSON-format contract.
//
// The macro side re-exports the types so existing call-sites compile
// unchanged; both sides round-trip through `serde_json` and the runtime
// `antigen_attestation::Predicate` type.
// ============================================================================

pub use antigen_attestation::parser::RequiresExpr;

// LeafExpr is only used by the depth-guard regression tests below; pull it
// in there via `super::*` rather than re-exporting from the macro crate's
// public surface (we keep the re-export of RequiresExpr because the
// ImmuneArgs / ToleranceArgs structs expose `requires: Option<(RequiresExpr,
// Span)>`).
#[cfg(test)]
use antigen_attestation::parser::LeafExpr;

// ============================================================================
// MacroAntigenCategory — local mirror of antigen::AntigenCategory (ADR-028)
//
// proc-macro crates cannot depend on the `antigen` library crate (circular
// dependency), so we maintain a local parse-time mirror. The two enums stay
// in sync; extending either requires an ADR amendment per ADR-001 C6.
// ============================================================================

/// Parse-time antigen-category variant (mirrors `antigen::AntigenCategory`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroAntigenCategory {
    SubstrateAlignment,
    FunctionalCorrectness,
}

impl MacroAntigenCategory {
    /// Parse from path-style expression strings and common aliases.
    fn from_path_str(s: &str) -> Option<Self> {
        match s {
            "SubstrateAlignment"
            | "AntigenCategory::SubstrateAlignment"
            | "substrate-alignment"
            | "substrate_alignment" => Some(Self::SubstrateAlignment),
            "FunctionalCorrectness"
            | "AntigenCategory::FunctionalCorrectness"
            | "functional-correctness"
            | "functional_correctness" => Some(Self::FunctionalCorrectness),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubstrateAlignment => "substrate-alignment",
            Self::FunctionalCorrectness => "functional-correctness",
        }
    }
}

/// Arguments to `#[antigen(...)]`.
#[allow(dead_code)]
// family/summary/references are captured for validation but
// not currently used in macro expansion. They will be used
// when the macro emits richer #[doc] forwards or registers
// declarations for cross-crate discovery (future ADRs).
#[derive(Debug)]
pub struct AntigenArgs {
    pub name: String,
    pub fingerprint: String,
    pub family: Option<String>,
    pub summary: Option<String>,
    pub references: Vec<String>,
    /// Antigen-category (ADR-028). Optional at parse time for backward-compat
    /// with v0.1 antigens; absence emits `antigen-category-defaulted-implicit-functional`
    /// at audit time. v0.2+ new declarations SHOULD supply this explicitly.
    pub category: Vec<MacroAntigenCategory>,
    /// Span of the `name`'s string literal value.
    /// `None` only when the field was missing — see [`AntigenArgs::validate`].
    pub name_span: Option<Span>,
    /// Span of the `fingerprint`'s string literal value.
    /// `None` only when the field was missing — see [`AntigenArgs::validate`].
    pub fingerprint_span: Option<Span>,
    /// Span of the macro's argument list as a whole. Used as the fallback
    /// anchor for missing-required-field errors (no offending token).
    pub args_span: Span,
}

/// Arguments to `#[presents(antigen_type)]`.
pub struct PresentsArgs {
    #[allow(dead_code)]
    pub antigen: Path,
}

/// Arguments to `#[immune(antigen_type, witness = ..., [rationale = ...])]`.
///
/// Accepts EITHER `witness = <expr>` (code-tier immunity) OR
/// `requires = <predicate>` (substrate-witness predicate, ADR-019).
/// Providing both or neither is a compile error.
pub struct ImmuneArgs {
    pub antigen: Path,
    pub witness: Option<Expr>,
    /// Substrate-witness predicate (ADR-019). Mutually exclusive with `witness`.
    pub requires: Option<(RequiresExpr, Span)>,
    #[allow(dead_code)]
    pub rationale: Option<String>,
}

/// Arguments to `#[descended_from(parent_path)]`.
pub struct DescendedFromArgs {
    #[allow(dead_code)]
    pub parent: Path,
}

/// Arguments to `#[antigen_tolerance(antigen, rationale = "...", until = "...", see = [...])]`.
///
/// Per ADR-011: positional antigen, required `rationale` (non-empty),
/// optional `until` (non-empty if present), optional `see` (open-vocab string array),
/// optional `requires = <predicate>` (substrate-witness sidecar predicate, ADR-019).
pub struct ToleranceArgs {
    #[allow(dead_code)]
    pub antigen: Path,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub until: Option<String>,
    pub until_span: Option<Span>,
    #[allow(dead_code)]
    pub see: Vec<String>,
    /// Optional substrate-witness sidecar predicate (ADR-019 tolerance tier).
    pub requires: Option<(RequiresExpr, Span)>,
    pub args_span: Span,
}

// ============================================================================
// AntigenArgs parsing
// ============================================================================

impl Parse for AntigenArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();

        let mut name: Option<String> = None;
        let mut name_span: Option<Span> = None;
        let mut fingerprint: Option<String> = None;
        let mut fingerprint_span: Option<Span> = None;
        let mut family: Option<String> = None;
        let mut summary: Option<String> = None;
        let mut references: Vec<String> = Vec::new();
        let mut category: Vec<MacroAntigenCategory> = Vec::new();

        let pairs: Punctuated<MetaPair, Token![,]> =
            input.parse_terminated(MetaPair::parse, Token![,])?;

        for pair in pairs {
            match pair.key.to_string().as_str() {
                "name" => {
                    let (s, span) = pair.expect_string_spanned()?;
                    name = Some(s);
                    name_span = Some(span);
                }
                "fingerprint" => {
                    let (s, span) = pair.expect_string_spanned()?;
                    fingerprint = Some(s);
                    fingerprint_span = Some(span);
                }
                "family" => family = Some(pair.expect_string()?),
                "summary" => summary = Some(pair.expect_string()?),
                "references" => references = pair.expect_string_array()?,
                "category" => category = pair.expect_antigen_category()?,
                other => {
                    return Err(syn::Error::new(
                        pair.key.span(),
                        format!(
                            "unknown #[antigen] field `{other}`; expected one of: \
                                 name, fingerprint, family, summary, references, category"
                        ),
                    ))
                }
            }
        }

        let name =
            name.ok_or_else(|| syn::Error::new(args_span, "#[antigen] requires `name = \"...\"`"))?;
        let fingerprint = fingerprint.ok_or_else(|| {
            syn::Error::new(args_span, "#[antigen] requires `fingerprint = \"...\"`")
        })?;

        Ok(Self {
            name,
            fingerprint,
            family,
            summary,
            references,
            category,
            name_span,
            fingerprint_span,
            args_span,
        })
    }
}

impl AntigenArgs {
    pub fn validate(&self) -> syn::Result<()> {
        if self.name.is_empty() {
            return Err(syn::Error::new(
                self.name_span.unwrap_or(self.args_span),
                "#[antigen] `name` cannot be empty",
            ));
        }
        if !is_kebab_case(&self.name) {
            return Err(syn::Error::new(
                self.name_span.unwrap_or(self.args_span),
                format!(
                    "#[antigen] `name = \"{}\"` must be kebab-case (lowercase with hyphens)",
                    self.name
                ),
            ));
        }
        if self.fingerprint.is_empty() {
            return Err(syn::Error::new(
                self.fingerprint_span.unwrap_or(self.args_span),
                "#[antigen] `fingerprint` cannot be empty",
            ));
        }
        // W6a: per ADR-010 Amendment 3 Clause E, the fingerprint string is
        // parsed at macro-compile time so malformed fingerprints don't ship.
        // Re-anchor any Path-C parser error to the fingerprint literal's span
        // so the user sees the squiggle on the offending text.
        if let Err(parse_err) = antigen_fingerprint::Fingerprint::parse(&self.fingerprint) {
            let anchor = self.fingerprint_span.unwrap_or(self.args_span);
            return Err(syn::Error::new(
                anchor,
                format!(
                    "#[antigen] `fingerprint` does not parse: {parse_err}\n\
                     (per ADR-010 Amendment 1 Path C — DSL syntax, not raw Rust expressions)"
                ),
            ));
        }
        Ok(())
    }
}

// ============================================================================
// PresentsArgs parsing
// ============================================================================

impl Parse for PresentsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let antigen: Path = input.parse()?;
        Ok(Self { antigen })
    }
}

// ============================================================================
// ImmuneArgs parsing
// ============================================================================

impl Parse for ImmuneArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let antigen: Path = input.parse()?;
        let mut witness: Option<Expr> = None;
        let mut requires: Option<(RequiresExpr, Span)> = None;
        let mut rationale: Option<String> = None;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            let key_span = key.span();
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witness" => {
                    witness = Some(input.parse()?);
                }
                "requires" => {
                    let pred: RequiresExpr = input.parse()?;
                    requires = Some((pred, key_span));
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[immune] field `{other}`; expected one of: \
                             witness, requires, rationale"
                        ),
                    ));
                }
            }
        }

        Ok(Self {
            antigen,
            witness,
            requires,
            rationale,
        })
    }
}

impl ImmuneArgs {
    pub fn validate(&self) -> syn::Result<()> {
        match (&self.witness, &self.requires) {
            (None, None) => Err(syn::Error::new_spanned(
                &self.antigen,
                "#[immune] requires either `witness = ...` (code-tier: a test, proptest, \
                 lint reference, formal-verification proof, or phantom-type construction) \
                 or `requires = <predicate>` (substrate-witness predicate, ADR-019). \
                 A marker without proof is not a claim.",
            )),
            (Some(_), Some((_, span))) => Err(syn::Error::new(
                *span,
                "#[immune] accepts either `witness = ...` or `requires = ...`, not both. \
                 For compound evidence across code-tier and substrate-tier, \
                 use `witnesses = [...]` (multi-witness syntax, ADR-019 §F11).",
            )),
            (_, Some((pred, span))) => pred.validate(*span),
            (Some(_), None) => Ok(()),
        }
    }

    /// If `requires` is set, return the JSON string for the predicate.
    /// The scan layer reads this from the `antigen:requires:v1:` doc marker.
    pub fn requires_json(&self) -> Option<String> {
        self.requires.as_ref().map(|(pred, _)| pred.to_json())
    }
}

// ============================================================================
// DescendedFromArgs parsing
// ============================================================================

impl Parse for DescendedFromArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parent: Path = input.parse()?;
        Ok(Self { parent })
    }
}

// ============================================================================
// ToleranceArgs parsing (ADR-011)
// ============================================================================

impl Parse for ToleranceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let antigen: Path = input.parse()?;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;
        let mut until: Option<String> = None;
        let mut until_span: Option<Span> = None;
        let mut see: Vec<String> = Vec::new();
        let mut requires: Option<(RequiresExpr, Span)> = None;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            let key_span = key.span();
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until_span = Some(lit.span());
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
                        } else {
                            return Err(syn::Error::new_spanned(
                                elem,
                                "expected a string literal in `see` array",
                            ));
                        }
                    }
                }
                "requires" => {
                    let pred: RequiresExpr = input.parse()?;
                    requires = Some((pred, key_span));
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[antigen_tolerance] field `{other}`; expected one of: \
                             rationale, until, see, requires",
                        ),
                    ));
                }
            }
        }

        Ok(Self {
            antigen,
            rationale,
            rationale_span,
            until,
            until_span,
            see,
            requires,
            args_span,
        })
    }
}

impl ToleranceArgs {
    /// Trust-boundary checks per ADR-011 Mechanics:
    /// - rationale required and non-empty (claim without rationale is not a claim)
    /// - until non-empty if present (empty string indicates user error)
    /// - requires predicate valid if present (semantic invariants per ADR-019 R-A6)
    pub fn validate(&self) -> syn::Result<()> {
        let Some(rationale) = self.rationale.as_deref() else {
            return Err(syn::Error::new_spanned(
                &self.antigen,
                "#[antigen_tolerance] requires `rationale = \"...\"`. \
                 A tolerance without rationale is not a claim — it's a silent suppression.",
            ));
        };
        if rationale.is_empty() {
            return Err(syn::Error::new(
                self.rationale_span.unwrap_or(self.args_span),
                "#[antigen_tolerance] `rationale` must not be empty",
            ));
        }
        if let Some(until) = self.until.as_deref() {
            if until.is_empty() {
                return Err(syn::Error::new(
                    self.until_span.unwrap_or(self.args_span),
                    "#[antigen_tolerance] `until = \"\"` rejected — \
                     an empty expiry indicates user error. Use `until = \"v1.0\"` \
                     (or similar) or omit the field entirely for forever-tolerance.",
                ));
            }
        }
        if let Some((pred, span)) = &self.requires {
            pred.validate(*span)?;
        }
        Ok(())
    }

    /// If `requires` is set, return the JSON string for the predicate.
    pub fn requires_json(&self) -> Option<String> {
        self.requires.as_ref().map(|(pred, _)| pred.to_json())
    }
}

// ============================================================================
// AnegyArgs parsing (ADR-023)
// ============================================================================

/// Arguments to `#[anergy(antigen_type, reason = "...", until = "...", ...)]`.
///
/// Per ADR-023: deferred-but-muted posture with aging escalation.
/// - `reason` required, minimum 20 characters
/// - `until` REQUIRED — anergy without time-bound degrades to tolerance (A5 absorbed)
/// - `expected_co_stimulation` advisory-only free text; NOT machine-verified
/// - `signed_by` optional signer identifier
#[derive(Debug)]
pub struct AnergyArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub reason: Option<String>,
    pub reason_span: Option<Span>,
    pub until: Option<String>,
    pub until_span: Option<Span>,
    #[allow(dead_code)]
    pub expected_co_stimulation: Option<String>,
    #[allow(dead_code)]
    pub signed_by: Option<String>,
    pub args_span: Span,
}

impl Parse for AnergyArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut reason: Option<String> = None;
        let mut reason_span: Option<Span> = None;
        let mut until: Option<String> = None;
        let mut until_span: Option<Span> = None;
        let mut expected_co_stimulation: Option<String> = None;
        let mut signed_by: Option<String> = None;

        // Optional leading positional antigen type path
        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            antigen = Some(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "reason" => {
                    let lit: LitStr = input.parse()?;
                    reason_span = Some(lit.span());
                    reason = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until_span = Some(lit.span());
                    until = Some(lit.value());
                }
                "expected_co_stimulation" => {
                    let lit: LitStr = input.parse()?;
                    expected_co_stimulation = Some(lit.value());
                }
                "signed_by" => {
                    let lit: LitStr = input.parse()?;
                    signed_by = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[anergy] field `{other}`; expected one of: \
                             reason, until, expected_co_stimulation, signed_by"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            antigen,
            reason,
            reason_span,
            until,
            until_span,
            expected_co_stimulation,
            signed_by,
            args_span,
        })
    }
}

impl AnergyArgs {
    /// Trust-boundary checks per ADR-023:
    /// - `reason` required and minimum 20 characters
    /// - `until` REQUIRED (A5 absorbed: anergy without time-bound = silent tolerance)
    /// - `until` must be non-empty
    pub fn validate(&self) -> syn::Result<()> {
        // reason required + 20-char minimum
        match self.reason.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[anergy] requires `reason = \"...\"`. \
                     Anergy without a stated reason is a silent suppression.",
                ));
            }
            Some(r) if r.len() < 20 => {
                return Err(syn::Error::new(
                    self.reason_span.unwrap_or(self.args_span),
                    format!(
                        "#[anergy] `reason` must be at least 20 characters \
                         (got {}); per ADR-023 loudness-as-discipline.",
                        r.len()
                    ),
                ));
            }
            _ => {}
        }

        // until REQUIRED (A5)
        match self.until.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[anergy] requires `until = \"YYYY-MM-DD\"`. \
                     Anergy without a time-bound degrades to silent tolerance. \
                     Per ADR-023 A5: `until` is not optional.",
                ));
            }
            Some("") => {
                return Err(syn::Error::new(
                    self.until_span.unwrap_or(self.args_span),
                    "#[anergy] `until` must not be empty. \
                     Use a date string, e.g. `until = \"2026-12-31\"`.",
                ));
            }
            _ => {}
        }

        Ok(())
    }
}

// ============================================================================
// ImmunosuppressArgs parsing (ADR-023)
// ============================================================================

/// Arguments to `#[immunosuppress(antigen_type, rationale = "...", until = "...", ...)]`.
///
/// Per ADR-023: surgical family-of-checks silencing with hard duration cap.
/// - `rationale` required, minimum 20 characters
/// - `until` required
/// - `since` optional ISO-8601 date; defaults to "now" for cap calculation
/// - `duration_cap` optional override (days); defaults to workspace 90d cap
/// - `signed_by` optional
/// - Compile error if implied duration exceeds cap (A4 absorbed)
#[derive(Debug)]
pub struct ImmunosuppressArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub until: Option<String>,
    pub until_span: Option<Span>,
    #[allow(dead_code)]
    pub since: Option<String>,
    pub duration_cap: Option<u64>,
    pub duration_cap_span: Option<Span>,
    #[allow(dead_code)]
    pub signed_by: Option<String>,
    pub args_span: Span,
}

impl Parse for ImmunosuppressArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        use syn::LitInt;
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;
        let mut until: Option<String> = None;
        let mut until_span: Option<Span> = None;
        let mut since: Option<String> = None;
        let mut duration_cap: Option<u64> = None;
        let mut duration_cap_span: Option<Span> = None;
        let mut signed_by: Option<String> = None;

        // Optional leading positional antigen type path
        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            antigen = Some(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until_span = Some(lit.span());
                    until = Some(lit.value());
                }
                "since" => {
                    let lit: LitStr = input.parse()?;
                    since = Some(lit.value());
                }
                "duration_cap" => {
                    let lit: LitInt = input.parse()?;
                    duration_cap_span = Some(lit.span());
                    duration_cap = Some(lit.base10_parse::<u64>()?);
                }
                "signed_by" => {
                    let lit: LitStr = input.parse()?;
                    signed_by = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[immunosuppress] field `{other}`; expected one of: \
                             rationale, until, since, duration_cap, signed_by"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            antigen,
            rationale,
            rationale_span,
            until,
            until_span,
            since,
            duration_cap,
            duration_cap_span,
            signed_by,
            args_span,
        })
    }
}

/// Default immunosuppression duration cap per ADR-023 (90 days).
pub const IMMUNOSUPPRESS_DEFAULT_CAP_DAYS: u64 = 90;

impl ImmunosuppressArgs {
    /// Trust-boundary checks per ADR-023:
    /// - `rationale` required and minimum 20 characters
    /// - `until` required
    /// - implied duration (until - since) must not exceed cap; COMPILE ERROR if exceeded (A4)
    pub fn validate(&self) -> syn::Result<()> {
        // rationale required + 20-char minimum
        match self.rationale.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[immunosuppress] requires `rationale = \"...\"`. \
                     Immunosuppression without rationale is not a claim.",
                ));
            }
            Some(r) if r.len() < 20 => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    format!(
                        "#[immunosuppress] `rationale` must be at least 20 characters \
                         (got {}); per ADR-023 loudness-as-discipline.",
                        r.len()
                    ),
                ));
            }
            _ => {}
        }

        // until required
        match self.until.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[immunosuppress] requires `until = \"YYYY-MM-DD\"`. \
                     Suppression without a deadline is indefinite suppression.",
                ));
            }
            Some("") => {
                return Err(syn::Error::new(
                    self.until_span.unwrap_or(self.args_span),
                    "#[immunosuppress] `until` must not be empty.",
                ));
            }
            _ => {}
        }

        // Duration cap enforcement (A4 absorbed): parse-time COMPILE ERROR
        // if until - since > cap days. This closes the audit-only gap.
        let cap = self.duration_cap.unwrap_or(IMMUNOSUPPRESS_DEFAULT_CAP_DAYS);
        // Use i64 throughout to avoid cast_possible_wrap: cap is workspace-configured
        // and guaranteed small (default 90); casting is safe but use i64 directly.
        let cap_i64 = i64::try_from(cap).unwrap_or(i64::MAX);
        if let Some(until_str) = self.until.as_deref() {
            if let Ok(until_date) = parse_iso_date(until_str) {
                let since_date = self
                    .since
                    .as_deref()
                    .and_then(|s| parse_iso_date(s).ok())
                    .unwrap_or_else(today_utc);
                let duration_days = (until_date - since_date).num_days();
                if duration_days > cap_i64 {
                    return Err(syn::Error::new(
                        self.until_span
                            .unwrap_or_else(|| self.duration_cap_span.unwrap_or(self.args_span)),
                        format!(
                            "#[immunosuppress] duration {duration_days}d exceeds cap {cap_i64}d. \
                             Per ADR-023: duration cap enforced at parse-time. \
                             Reduce the `until` date or set `duration_cap = N` (workspace \
                             default is {IMMUNOSUPPRESS_DEFAULT_CAP_DAYS}d)."
                        ),
                    ));
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// PoxpartyArgs parsing (ADR-023)
// ============================================================================

/// Arguments to `#[poxparty(antigen_type, exercise_type = "...", until = "...", ...)]`.
///
/// Per ADR-023: intentional exposure with structural compile-time isolation.
///
/// CRITICAL (A3 absorbed): the proc-macro checks `CARGO_FEATURE_ANTIGEN_POXPARTY`
/// env var at macro-expansion time and emits a COMPILE ERROR if the feature is
/// not active. This closes the production-isolation gap — poxparty code cannot
/// compile in a build where the `antigen-poxparty` Cargo feature is absent.
///
/// The `antigen-poxparty` feature MUST NOT be in the default feature set.
///
/// - `exercise_type` required, minimum 20 characters
/// - `until` required
/// - `name` optional descriptive name
/// - `rationale` optional additional context
/// - `signed_by` optional
#[derive(Debug)]
pub struct PoxpartyArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub exercise_type: Option<String>,
    pub exercise_type_span: Option<Span>,
    pub until: Option<String>,
    pub until_span: Option<Span>,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub rationale: Option<String>,
    #[allow(dead_code)]
    pub signed_by: Option<String>,
    pub args_span: Span,
}

impl Parse for PoxpartyArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut exercise_type: Option<String> = None;
        let mut exercise_type_span: Option<Span> = None;
        let mut until: Option<String> = None;
        let mut until_span: Option<Span> = None;
        let mut name: Option<String> = None;
        let mut rationale: Option<String> = None;
        let mut signed_by: Option<String> = None;

        // Optional leading positional antigen type path
        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            antigen = Some(input.parse()?);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "exercise_type" => {
                    let lit: LitStr = input.parse()?;
                    exercise_type_span = Some(lit.span());
                    exercise_type = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until_span = Some(lit.span());
                    until = Some(lit.value());
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
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[poxparty] field `{other}`; expected one of: \
                             exercise_type, until, name, rationale, signed_by"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            antigen,
            exercise_type,
            exercise_type_span,
            until,
            until_span,
            name,
            rationale,
            signed_by,
            args_span,
        })
    }
}

impl PoxpartyArgs {
    /// Trust-boundary checks per ADR-023:
    /// - `exercise_type` required and minimum 20 characters
    /// - `until` required
    ///
    /// Note: structural isolation is two-layer — primary via
    /// `#[cfg(feature = "antigen-poxparty")]` on the containing module (cfg
    /// gate prevents expansion when feature absent), secondary via the
    /// `CARGO_FEATURE_ANTIGEN_POXPARTY` env var check in the entry point.
    /// Neither check is in the parser.
    pub fn validate(&self) -> syn::Result<()> {
        // exercise_type required + 20-char minimum
        match self.exercise_type.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[poxparty] requires `exercise_type = \"...\"`. \
                     Per ADR-023: describes the controlled exposure exercise.",
                ));
            }
            Some(et) if et.len() < 20 => {
                return Err(syn::Error::new(
                    self.exercise_type_span.unwrap_or(self.args_span),
                    format!(
                        "#[poxparty] `exercise_type` must be at least 20 characters \
                         (got {}); per ADR-023 loudness-as-discipline.",
                        et.len()
                    ),
                ));
            }
            _ => {}
        }

        // until required
        match self.until.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[poxparty] requires `until = \"YYYY-MM-DD\"`. \
                     A pox party without a deadline runs indefinitely.",
                ));
            }
            Some("") => {
                return Err(syn::Error::new(
                    self.until_span.unwrap_or(self.args_span),
                    "#[poxparty] `until` must not be empty.",
                ));
            }
            _ => {}
        }

        Ok(())
    }
}

// ============================================================================
// OrientArgs parsing (ADR-023)
// ============================================================================

/// Arguments to `#[orient(see = [...], adr = "...", attestation_optional)]`.
///
/// Per ADR-023: see-also context without antigen claim. The lightest-weight
/// deferred-defense primitive — acknowledges orientation period during which
/// immunity is explicitly absent. All fields are optional.
///
/// - `see` optional array of references (URLs, ADR IDs, etc.)
/// - `adr` optional ADR reference string
/// - `attestation_optional` optional boolean flag
#[derive(Debug)]
pub struct OrientArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    #[allow(dead_code)]
    pub see: Vec<String>,
    #[allow(dead_code)]
    pub adr: Option<String>,
    #[allow(dead_code)]
    pub attestation_optional: bool,
    #[allow(dead_code)]
    pub args_span: Span,
}

impl Parse for OrientArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut see: Vec<String> = Vec::new();
        let mut adr: Option<String> = None;
        let mut attestation_optional = false;

        // Optional leading positional antigen type path
        if !input.is_empty() && input.peek(Ident) && !input.peek2(Token![=]) {
            // Could be `attestation_optional` flag (bare ident without `=`)
            // or an antigen type path — disambiguate by checking if it's a
            // bare ident followed by a comma or end-of-input (flag) vs
            // followed by `::` or end-of-args (path).
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "attestation_optional" && (fork.is_empty() || fork.peek(Token![,])) {
                // Consume from real stream
                let _: Ident = input.parse()?;
                attestation_optional = true;
            } else {
                antigen = Some(input.parse()?);
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        while !input.is_empty() {
            // Check for bare `attestation_optional` flag (no `=`)
            if input.peek(Ident) {
                let fork = input.fork();
                let ident: Ident = fork.parse()?;
                if ident == "attestation_optional" && (fork.is_empty() || fork.peek(Token![,])) {
                    let _: Ident = input.parse()?;
                    attestation_optional = true;
                    if !input.is_empty() {
                        input.parse::<Token![,]>()?;
                    }
                    continue;
                }
            }

            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "see" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            see.push(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                elem,
                                "expected a string literal in `see` array",
                            ));
                        }
                    }
                }
                "adr" => {
                    let lit: LitStr = input.parse()?;
                    adr = Some(lit.value());
                }
                "attestation_optional" => {
                    // Support `attestation_optional = true` form too
                    let lit: syn::LitBool = input.parse()?;
                    attestation_optional = lit.value();
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[orient] field `{other}`; expected one of: \
                             see, adr, attestation_optional"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            antigen,
            see,
            adr,
            attestation_optional,
            args_span,
        })
    }
}

impl OrientArgs {
    /// `#[orient]` has no required fields — all fields optional per ADR-023
    /// (lightest-weight deferred-defense primitive).
    ///
    /// Returns `Ok(())` always. The `syn::Result<()>` return type is kept for
    /// API uniformity with the other deferred-defense validators.
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub const fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Convergent-Evidence Family argument parsers (ADR-024)
// ============================================================================

/// Arguments to `#[diagnostic(modalities = [...], min_independent = N)]`.
///
/// Per ADR-024 §Decision + adversarial C1:
/// - `modalities` is a list of `WitnessClass::X` paths
/// - `min_independent` is REQUIRED and measured in distinct CLASSES
#[derive(Debug)]
pub struct DiagnosticArgs {
    /// The witness-class paths, captured as their final ident only.
    /// E.g., `WitnessClass::StaticAnalysis` → `"StaticAnalysis"`.
    pub modality_classes: Vec<String>,
    /// The list of modality spans for span-anchored error messages.
    pub modality_span: Option<Span>,
    /// Required minimum distinct classes.
    pub min_independent: Option<u64>,
    /// Span anchor for `min_independent` for span-anchored error messages.
    pub min_independent_span: Option<Span>,
    /// Span of the macro's argument list.
    pub args_span: Span,
}

impl Parse for DiagnosticArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use syn::LitInt;
        let args_span = input.span();
        let mut modality_classes: Vec<String> = Vec::new();
        let mut modality_span: Option<Span> = None;
        let mut min_independent: Option<u64> = None;
        let mut min_independent_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "modalities" => {
                    let arr: syn::ExprArray = input.parse()?;
                    modality_span = Some(arr.bracket_token.span.join());
                    for elem in &arr.elems {
                        if let Expr::Path(p) = elem {
                            if let Some(seg) = p.path.segments.last() {
                                modality_classes.push(seg.ident.to_string());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            elem,
                            "expected a `WitnessClass::*` path in `modalities` array",
                        ));
                    }
                }
                "min_independent" => {
                    let lit: LitInt = input.parse()?;
                    min_independent_span = Some(lit.span());
                    min_independent = Some(lit.base10_parse::<u64>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[diagnostic] field `{other}`; expected: modalities, min_independent"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            modality_classes,
            modality_span,
            min_independent,
            min_independent_span,
            args_span,
        })
    }
}

impl DiagnosticArgs {
    /// Per ADR-024 + adversarial C1: `min_independent` is REQUIRED;
    /// `modalities` must be non-empty; `min_independent` must not exceed
    /// the number of distinct modality classes (otherwise the claim is
    /// vacuously unsatisfiable).
    pub fn validate(&self) -> syn::Result<()> {
        if self.modality_classes.is_empty() {
            return Err(syn::Error::new(
                self.modality_span.unwrap_or(self.args_span),
                "#[diagnostic] requires non-empty `modalities = [WitnessClass::X, ...]` \
                 (ADR-024 §Decision; an empty modality set has no evidence to converge).",
            ));
        }
        let Some(min) = self.min_independent else {
            return Err(syn::Error::new(
                self.args_span,
                "#[diagnostic] requires `min_independent = N` (ADR-024 §Decision; \
                 N counts distinct WitnessClass categories per adversarial C1).",
            ));
        };
        if min == 0 {
            return Err(syn::Error::new(
                self.min_independent_span.unwrap_or(self.args_span),
                "#[diagnostic] `min_independent` must be at least 1 (a 0-floor is \
                 vacuously true and structurally meaningless).",
            ));
        }
        // Count distinct classes (per C1: classes, not raw count)
        let mut distinct: Vec<&String> = Vec::new();
        for c in &self.modality_classes {
            if !distinct.contains(&c) {
                distinct.push(c);
            }
        }
        if u64::try_from(distinct.len()).unwrap_or(u64::MAX) < min {
            return Err(syn::Error::new(
                self.min_independent_span.unwrap_or(self.args_span),
                format!(
                    "#[diagnostic] `min_independent = {min}` exceeds the number of \
                     distinct WitnessClass categories supplied ({}). Per ADR-024 \
                     adversarial C1, min_independent counts CLASSES not witnesses — \
                     duplicate classes don't add independence. Increase distinct \
                     modalities or lower the floor.",
                    distinct.len()
                ),
            ));
        }
        Ok(())
    }
}

/// Arguments to `#[clonal(witness = ..., iterations = N, seed = SeedKind::...)]`.
///
/// Per ADR-024 §Decision + adversarial C2:
/// - `witness` is REQUIRED — references the per-iteration function/test
/// - `iterations` is REQUIRED — explicit non-zero count
/// - `seed = SeedKind::Fixed(_)` is REJECTED at parse time
#[derive(Debug)]
pub struct ClonalArgs {
    #[allow(dead_code)]
    pub witness: Option<Expr>,
    pub witness_span: Option<Span>,
    pub iterations: Option<u64>,
    pub iterations_span: Option<Span>,
    /// The final ident of the `seed = SeedKind::X(...)` path. `None` if
    /// no seed argument supplied (defaults to `Random`). Captured for
    /// scan-side introspection / future audit-hint refinement.
    #[allow(dead_code)]
    pub seed_kind: Option<String>,
    pub seed_span: Option<Span>,
    /// True if the seed expression was `SeedKind::Fixed(...)`.
    pub seed_is_fixed: bool,
    pub args_span: Span,
}

impl Parse for ClonalArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use syn::LitInt;
        let args_span = input.span();
        let mut witness: Option<Expr> = None;
        let mut witness_span: Option<Span> = None;
        let mut iterations: Option<u64> = None;
        let mut iterations_span: Option<Span> = None;
        let mut seed_kind: Option<String> = None;
        let mut seed_span: Option<Span> = None;
        let mut seed_is_fixed = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witness" => {
                    let e: Expr = input.parse()?;
                    witness_span = Some(syn::spanned::Spanned::span(&e));
                    witness = Some(e);
                }
                "iterations" => {
                    let lit: LitInt = input.parse()?;
                    iterations_span = Some(lit.span());
                    iterations = Some(lit.base10_parse::<u64>()?);
                }
                "seed" => {
                    let e: Expr = input.parse()?;
                    seed_span = Some(syn::spanned::Spanned::span(&e));
                    // Extract the final ident: SeedKind::Random / SeedKind::Fixed(...).
                    if let Expr::Path(p) = &e {
                        if let Some(seg) = p.path.segments.last() {
                            let name = seg.ident.to_string();
                            seed_kind = Some(name);
                        }
                    } else if let Expr::Call(c) = &e {
                        if let Expr::Path(p) = &*c.func {
                            if let Some(seg) = p.path.segments.last() {
                                let name = seg.ident.to_string();
                                if name == "Fixed" {
                                    seed_is_fixed = true;
                                }
                                seed_kind = Some(name);
                            }
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[clonal] field `{other}`; expected: witness, iterations, seed"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            witness,
            witness_span,
            iterations,
            iterations_span,
            seed_kind,
            seed_span,
            seed_is_fixed,
            args_span,
        })
    }
}

impl ClonalArgs {
    /// Trust-boundary checks per ADR-024:
    /// - `witness` REQUIRED
    /// - `iterations` REQUIRED and > 0
    /// - `seed = SeedKind::Fixed(_)` REJECTED with COMPILE ERROR (C2)
    pub fn validate(&self) -> syn::Result<()> {
        if self.witness.is_none() {
            return Err(syn::Error::new(
                self.witness_span.unwrap_or(self.args_span),
                "#[clonal] requires `witness = <identifier>` referencing the \
                 per-iteration function/test (ADR-024 §Decision).",
            ));
        }
        let Some(iters) = self.iterations else {
            return Err(syn::Error::new(
                self.args_span,
                "#[clonal] requires `iterations = N` (ADR-024 §Decision; \
                 explicit iteration count is the structural memory of how \
                 much independence the witness is claiming).",
            ));
        };
        if iters == 0 {
            return Err(syn::Error::new(
                self.iterations_span.unwrap_or(self.args_span),
                "#[clonal] `iterations` must be > 0 (zero iterations means \
                 no evidence).",
            ));
        }
        if self.seed_is_fixed {
            return Err(syn::Error::new(
                self.seed_span.unwrap_or(self.args_span),
                "#[clonal] rejects `seed = SeedKind::Fixed(_)` — a fixed seed \
                 makes 'iterations' a misnomer (every iteration replays the same \
                 RNG state). Use SeedKind::Random, SeedKind::EntropyFromCi, or \
                 SeedKind::TimestampSeeded. \
                 Per ADR-024 adversarial C2: COMPILE-TIME enforcement.",
            ));
        }
        Ok(())
    }
}

/// Arguments to `#[igg(witnesses = [...], historical_span = N, min_reattestations = N)]`.
///
/// Per ADR-024 §Decision + adversarial C3:
/// - `witnesses` REQUIRED non-empty
/// - `historical_span` REQUIRED (days)
/// - `min_reattestations` REQUIRED
/// - Source-independence is NOMINAL — different signer-identity strings
///   don't structurally prove independent sources (named limitation)
#[derive(Debug)]
pub struct IggArgs {
    #[allow(dead_code)]
    pub witnesses: Vec<Expr>,
    pub witnesses_span: Option<Span>,
    pub historical_span: Option<u64>,
    pub historical_span_span: Option<Span>,
    pub min_reattestations: Option<u64>,
    pub min_reattestations_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for IggArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use syn::LitInt;
        let args_span = input.span();
        let mut witnesses: Vec<Expr> = Vec::new();
        let mut witnesses_span: Option<Span> = None;
        let mut historical_span: Option<u64> = None;
        let mut historical_span_span: Option<Span> = None;
        let mut min_reattestations: Option<u64> = None;
        let mut min_reattestations_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "witnesses" => {
                    let arr: syn::ExprArray = input.parse()?;
                    witnesses_span = Some(arr.bracket_token.span.join());
                    for elem in &arr.elems {
                        witnesses.push(elem.clone());
                    }
                }
                "historical_span" => {
                    let lit: LitInt = input.parse()?;
                    historical_span_span = Some(lit.span());
                    historical_span = Some(lit.base10_parse::<u64>()?);
                }
                "min_reattestations" => {
                    let lit: LitInt = input.parse()?;
                    min_reattestations_span = Some(lit.span());
                    min_reattestations = Some(lit.base10_parse::<u64>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[igg] field `{other}`; expected: \
                             witnesses, historical_span, min_reattestations"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            witnesses,
            witnesses_span,
            historical_span,
            historical_span_span,
            min_reattestations,
            min_reattestations_span,
            args_span,
        })
    }
}

impl IggArgs {
    pub fn validate(&self) -> syn::Result<()> {
        if self.witnesses.is_empty() {
            return Err(syn::Error::new(
                self.witnesses_span.unwrap_or(self.args_span),
                "#[igg] requires non-empty `witnesses = [...]` (ADR-024 §Decision).",
            ));
        }
        let Some(span) = self.historical_span else {
            return Err(syn::Error::new(
                self.args_span,
                "#[igg] requires `historical_span = N` (days; ADR-024 §Decision).",
            ));
        };
        if span == 0 {
            return Err(syn::Error::new(
                self.historical_span_span.unwrap_or(self.args_span),
                "#[igg] `historical_span` must be > 0 days.",
            ));
        }
        let Some(min) = self.min_reattestations else {
            return Err(syn::Error::new(
                self.args_span,
                "#[igg] requires `min_reattestations = N` (ADR-024 §Decision).",
            ));
        };
        if min == 0 {
            return Err(syn::Error::new(
                self.min_reattestations_span.unwrap_or(self.args_span),
                "#[igg] `min_reattestations` must be > 0.",
            ));
        }
        Ok(())
    }
}

/// Arguments to `#[crossreactive(fingerprints = [...])]`.
///
/// Declares one defense addresses multiple related antigens (cross-
/// reactive immune response).
#[derive(Debug)]
pub struct CrossreactiveArgs {
    #[allow(dead_code)]
    pub fingerprints: Vec<String>,
    pub fingerprints_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for CrossreactiveArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut fingerprints: Vec<String> = Vec::new();
        let mut fingerprints_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "fingerprints" => {
                    let arr: syn::ExprArray = input.parse()?;
                    fingerprints_span = Some(arr.bracket_token.span.join());
                    for elem in &arr.elems {
                        if let Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(s), ..
                        }) = elem
                        {
                            fingerprints.push(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                elem,
                                "expected a string literal fingerprint in `fingerprints` array",
                            ));
                        }
                    }
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown #[crossreactive] field `{other}`; expected: fingerprints"),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            fingerprints,
            fingerprints_span,
            args_span,
        })
    }
}

impl CrossreactiveArgs {
    pub fn validate(&self) -> syn::Result<()> {
        if self.fingerprints.is_empty() {
            return Err(syn::Error::new(
                self.fingerprints_span.unwrap_or(self.args_span),
                "#[crossreactive] requires non-empty `fingerprints = [...]` \
                 (a defense that covers no related antigens isn't crossreactive).",
            ));
        }
        for f in &self.fingerprints {
            if f.is_empty() {
                return Err(syn::Error::new(
                    self.fingerprints_span.unwrap_or(self.args_span),
                    "#[crossreactive] `fingerprints` entries must be non-empty strings.",
                ));
            }
        }
        Ok(())
    }
}

/// Arguments to `#[polyclonal]` — many independent lineages.
/// No required fields per ADR-024 §Decision.
#[derive(Debug)]
pub struct PolyclonalArgs {
    #[allow(dead_code)]
    pub args_span: Span,
}

impl Parse for PolyclonalArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        // Accept arbitrary trailing args for forward compat — discard them.
        while !input.is_empty() {
            let _: proc_macro2::TokenTree = input.parse()?;
        }
        Ok(Self { args_span })
    }
}

impl PolyclonalArgs {
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub const fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}

/// Arguments to `#[monoclonal]` — single independent lineage. No required
/// fields per ADR-024 §Decision.
#[derive(Debug)]
pub struct MonoclonalArgs {
    #[allow(dead_code)]
    pub args_span: Span,
}

impl Parse for MonoclonalArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        while !input.is_empty() {
            let _: proc_macro2::TokenTree = input.parse()?;
        }
        Ok(Self { args_span })
    }
}

impl MonoclonalArgs {
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub const fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}

/// Arguments to `#[adcc]` — antibody + cellular effector (multi-mechanism).
/// No required fields per ADR-024 §Decision.
#[derive(Debug)]
pub struct AdccArgs {
    #[allow(dead_code)]
    pub args_span: Span,
}

impl Parse for AdccArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        while !input.is_empty() {
            let _: proc_macro2::TokenTree = input.parse()?;
        }
        Ok(Self { args_span })
    }
}

impl AdccArgs {
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub const fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Date helpers for ADR-023 parse-time enforcement
// ============================================================================

/// Parse an ISO-8601 date string (`YYYY-MM-DD`) into a `chrono::NaiveDate`.
/// Returns `Err` if the string is not a valid date — callers treat parse
/// failure as "cannot validate; skip cap check" to avoid false compile errors
/// on non-date `until` strings (e.g., version tags like `"v2.0"`).
///
/// UTC mandate per ADR-023 §Enforcement-Surface.
fn parse_iso_date(s: &str) -> Result<chrono::NaiveDate, ()> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| ())
}

/// Return today's UTC date for cap calculations when `since` is absent.
fn today_utc() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

// ============================================================================
// Helpers
// ============================================================================

struct MetaPair {
    key: Ident,
    value: Expr,
}

impl Parse for MetaPair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(Self { key, value })
    }
}

impl MetaPair {
    fn expect_string(&self) -> syn::Result<String> {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &self.value
        {
            Ok(s.value())
        } else {
            Err(syn::Error::new_spanned(
                &self.value,
                format!("expected a string literal for `{}`", self.key),
            ))
        }
    }

    /// Like [`Self::expect_string`] but also returns the span of the string
    /// literal so validation errors can point at the literal itself.
    fn expect_string_spanned(&self) -> syn::Result<(String, Span)> {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &self.value
        {
            Ok((s.value(), s.span()))
        } else {
            Err(syn::Error::new_spanned(
                &self.value,
                format!("expected a string literal for `{}`", self.key),
            ))
        }
    }

    fn expect_string_array(&self) -> syn::Result<Vec<String>> {
        if let Expr::Array(arr) = &self.value {
            let mut out = Vec::new();
            for elem in &arr.elems {
                if let Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) = elem
                {
                    out.push(s.value());
                } else {
                    return Err(syn::Error::new_spanned(
                        elem,
                        "expected a string literal in references array",
                    ));
                }
            }
            Ok(out)
        } else {
            Err(syn::Error::new_spanned(
                &self.value,
                format!("expected a string array for `{}`", self.key),
            ))
        }
    }

    /// Parse `category = AntigenCategory::X` (single) or
    /// `category = [AntigenCategory::X, AntigenCategory::Y]` (hybrid).
    ///
    /// Accepts path expressions like `AntigenCategory::SubstrateAlignment` or
    /// plain idents like `SubstrateAlignment`. String literals are NOT
    /// accepted — the category must be a path expression for compile-time
    /// discoverability.
    fn expect_antigen_category(&self) -> syn::Result<Vec<MacroAntigenCategory>> {
        fn parse_single(expr: &Expr) -> syn::Result<MacroAntigenCategory> {
            // Convert the expression to a string representation and then match
            let s = match expr {
                Expr::Path(p) => {
                    // Reconstruct the path string: "AntigenCategory::SubstrateAlignment"
                    // or just "SubstrateAlignment"
                    let segments: Vec<String> = p
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect();
                    segments.join("::")
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        expr,
                        "expected `AntigenCategory::SubstrateAlignment` or \
                         `AntigenCategory::FunctionalCorrectness`",
                    ));
                }
            };
            MacroAntigenCategory::from_path_str(&s).ok_or_else(|| {
                syn::Error::new_spanned(
                    expr,
                    format!(
                        "unknown AntigenCategory `{s}`; expected \
                         `AntigenCategory::SubstrateAlignment` or \
                         `AntigenCategory::FunctionalCorrectness`"
                    ),
                )
            })
        }

        match &self.value {
            Expr::Array(arr) => {
                let mut out = Vec::new();
                for elem in &arr.elems {
                    out.push(parse_single(elem)?);
                }
                if out.is_empty() {
                    return Err(syn::Error::new_spanned(
                        &self.value,
                        "`category` array must not be empty",
                    ));
                }
                Ok(out)
            }
            single => Ok(vec![parse_single(single)?]),
        }
    }
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

// The `requires_json_tests` module previously lived here; it moved with the
// parser into `antigen_attestation::parser` (run via
// `cargo test -p antigen-attestation --features parser`). Keeping the tests
// next to the implementation prevents drift and removes the
// circular-dependency-by-test-coupling that used to live in this file.

// ============================================================================
// Cross-parser equivalence fixtures
//
// These fixtures define the invariant: for any input the macro side accepts as
// valid, the scan side must produce equivalent semantic content for the four
// overlapping fields (name, fingerprint, family, summary). The same fixture
// table appears in `antigen/src/scan.rs` (ScanAntigenArgs tests) — keeping the
// inputs and expected outputs literally identical is what makes the
// equivalence inspectable.
//
// ATK-001-2 lesson: the brittle string-manipulation parser corrupted
// fingerprints with inner double-quotes silently. Property tests over both
// parsers prevent that class of drift from re-emerging.
//
// When adding a fixture here, add the matching one to scan.rs. When adding a
// new field to the antigen attribute grammar, add fixtures here AND to scan.rs
// to lock the equivalence.
// ============================================================================

/// Fixture tuple shape: `(input, expected_name, expected_fingerprint,
/// expected_family, expected_summary)`.
#[cfg(test)]
type AntigenFixture = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
);

#[cfg(test)]
const ANTIGEN_PARSER_FIXTURES: &[AntigenFixture] = &[
    // 1. Smoke test: just the two required fields.
    (
        r#"name = "panicking-in-drop", fingerprint = "impl Drop with panic""#,
        "panicking-in-drop",
        "impl Drop with panic",
        None,
        None,
    ),
    // 2. All four fields populated.
    (
        r#"name = "frame-translation", fingerprint = "class enum + meet", family = "semantic-drift", summary = "Polarity inverts at the frame boundary""#,
        "frame-translation",
        "class enum + meet",
        Some("semantic-drift"),
        Some("Polarity inverts at the frame boundary"),
    ),
    // 3. Inner-quoted fingerprint (the ATK-001-2 regression case).
    (
        r#"name = "x", fingerprint = "item: enum, has_method(\"meet\", \"(Self, Self) -> Self\")""#,
        "x",
        r#"item: enum, has_method("meet", "(Self, Self) -> Self")"#,
        None,
        None,
    ),
    // 4. Reordered fields (order-invariance check).
    (
        r#"summary = "S", family = "F", fingerprint = "FP", name = "n""#,
        "n",
        "FP",
        Some("F"),
        Some("S"),
    ),
    // 5. References array present (macro stores; scan ignores; both must
    //    accept without error).
    (
        r#"name = "x", fingerprint = "y", references = ["GAP-1", "DEC-2"]"#,
        "x",
        "y",
        None,
        None,
    ),
    // 6. Multi-line whitespace (tab + newline) — common rustfmt output shape.
    (
        "name = \"multi-line\",\n\tfingerprint = \"shape\",\n\tfamily = \"family\"",
        "multi-line",
        "shape",
        Some("family"),
        None,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    #[test]
    fn antigen_parser_accepts_all_fixtures() {
        for (input, exp_name, exp_fp, exp_family, exp_summary) in ANTIGEN_PARSER_FIXTURES {
            let tokens: TokenStream = input
                .parse()
                .unwrap_or_else(|e| panic!("fixture failed to tokenize: {input:?}: {e}"));
            let args = syn::parse2::<AntigenArgs>(tokens)
                .unwrap_or_else(|e| panic!("macro parser rejected fixture {input:?}: {e}"));
            assert_eq!(&args.name, exp_name, "name mismatch for fixture: {input:?}");
            assert_eq!(
                &args.fingerprint, exp_fp,
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
    fn antigen_parser_rejects_missing_name() {
        let tokens: TokenStream = r#"fingerprint = "x""#.parse().unwrap();
        assert!(syn::parse2::<AntigenArgs>(tokens).is_err());
    }

    #[test]
    fn antigen_parser_rejects_missing_fingerprint() {
        let tokens: TokenStream = r#"name = "x""#.parse().unwrap();
        assert!(syn::parse2::<AntigenArgs>(tokens).is_err());
    }

    #[test]
    fn antigen_parser_rejects_unknown_field() {
        let tokens: TokenStream = r#"name = "x", fingerprint = "y", bogus = "z""#.parse().unwrap();
        match syn::parse2::<AntigenArgs>(tokens) {
            Ok(_) => panic!("expected parse to reject unknown field `bogus`"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("unknown") && msg.contains("bogus"),
                    "expected unknown-field error mentioning `bogus`, got: {msg}"
                );
            }
        }
    }

    /// Construct an `AntigenArgs` with the given name + a valid DSL fingerprint.
    /// Used by direct-construction tests that bypass `Parse` to exercise
    /// name-validation paths in `validate()`. Tests that need to exercise
    /// fingerprint-validation paths build their own `AntigenArgs` literal
    /// with the specific fingerprint they want to assert against.
    fn args_with(name: &str, fingerprint: &str) -> AntigenArgs {
        AntigenArgs {
            name: name.to_string(),
            fingerprint: fingerprint.to_string(),
            family: None,
            summary: None,
            references: Vec::new(),
            category: Vec::new(),
            name_span: Some(proc_macro2::Span::call_site()),
            fingerprint_span: Some(proc_macro2::Span::call_site()),
            args_span: proc_macro2::Span::call_site(),
        }
    }

    /// Minimal DSL fingerprint string accepted by the W6a parser. Tests that
    /// don't care about fingerprint content but DO want `validate()` to succeed
    /// use this to keep their assertions focused on name validation.
    const VALID_DSL: &str = r#"name = matches("*")"#;

    #[test]
    fn validate_rejects_empty_name() {
        assert!(args_with("", VALID_DSL).validate().is_err());
    }

    #[test]
    fn validate_rejects_non_kebab_name() {
        assert!(args_with("FooBar", VALID_DSL).validate().is_err());
    }

    #[test]
    fn validate_accepts_kebab_name_with_digits() {
        assert!(args_with("frame-2-translation", VALID_DSL)
            .validate()
            .is_ok());
    }

    #[test]
    fn validate_rejects_name_with_double_hyphen() {
        assert!(args_with("frame--translation", VALID_DSL)
            .validate()
            .is_err());
    }

    #[test]
    fn validate_rejects_name_starting_with_hyphen() {
        assert!(args_with("-frame", VALID_DSL).validate().is_err());
    }

    #[test]
    fn validate_rejects_malformed_dsl_fingerprint() {
        let args = args_with("ok-name", "this is not the dsl");
        let err = args.validate().unwrap_err().to_string();
        assert!(err.contains("fingerprint"), "got: {err}");
    }

    #[test]
    fn immune_parser_requires_witness() {
        let tokens: TokenStream = r"PanickingInDrop".parse().unwrap();
        let args = syn::parse2::<ImmuneArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn immune_parser_accepts_witness_path() {
        let tokens: TokenStream = r"PanickingInDrop, witness = my::module::test_fn"
            .parse()
            .unwrap();
        let args = syn::parse2::<ImmuneArgs>(tokens).unwrap();
        assert!(args.witness.is_some());
        assert!(args.validate().is_ok());
    }

    #[test]
    fn immune_parser_accepts_rationale() {
        let tokens: TokenStream = r#"X, witness = my_test, rationale = "checked manually""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ImmuneArgs>(tokens).unwrap();
        assert_eq!(args.rationale.as_deref(), Some("checked manually"));
    }

    #[test]
    fn requires_expr_depth_guard_rejects_excessive_nesting() {
        // Build a RequiresExpr at depth MAX_DEPTH+1 programmatically (bypassing
        // the proc-macro parse path, which would stack-overflow for truly pathological
        // depth). The validate() path runs the depth check post-parse.
        const MAX_DEPTH: usize = 64;
        let leaf = RequiresExpr::Leaf(LeafExpr::FreshWithinDays { days: 90 });
        let mut pred = leaf;
        // Wrap in MAX_DEPTH+1 levels of Not — one too many.
        for _ in 0..=MAX_DEPTH {
            pred = RequiresExpr::Not(Box::new(pred));
        }
        let err = pred
            .validate(proc_macro2::Span::call_site())
            .expect_err("depth exceeding MAX_DEPTH must be rejected by validate()");
        assert!(
            err.to_string().contains("depth") || err.to_string().contains("nesting"),
            "error must mention depth/nesting: {err}"
        );
    }

    #[test]
    fn requires_expr_depth_guard_accepts_at_max_depth() {
        // A predicate at exactly MAX_DEPTH nesting must be accepted.
        const MAX_DEPTH: usize = 64;
        let leaf = RequiresExpr::Leaf(LeafExpr::FreshWithinDays { days: 90 });
        let mut pred = leaf;
        for _ in 0..MAX_DEPTH {
            pred = RequiresExpr::Not(Box::new(pred));
        }
        assert!(
            pred.validate(proc_macro2::Span::call_site()).is_ok(),
            "predicate at exactly MAX_DEPTH must be accepted"
        );
    }

    // ========================================================================
    // AntigenCategory parsing tests (ADR-028)
    // ========================================================================

    #[test]
    fn antigen_parser_accepts_category_single_substrate_alignment() {
        let tokens: TokenStream =
            r#"name = "x", fingerprint = "item = fn", category = AntigenCategory::SubstrateAlignment"#
                .parse()
                .unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        assert_eq!(
            args.category,
            vec![MacroAntigenCategory::SubstrateAlignment]
        );
    }

    #[test]
    fn antigen_parser_accepts_category_single_functional_correctness() {
        let tokens: TokenStream =
            r#"name = "x", fingerprint = "item = fn", category = AntigenCategory::FunctionalCorrectness"#
                .parse()
                .unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        assert_eq!(
            args.category,
            vec![MacroAntigenCategory::FunctionalCorrectness]
        );
    }

    #[test]
    fn antigen_parser_accepts_category_hybrid_array() {
        let tokens: TokenStream = r#"name = "x", fingerprint = "item = fn", category = [AntigenCategory::SubstrateAlignment, AntigenCategory::FunctionalCorrectness]"#
            .parse()
            .unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        assert_eq!(
            args.category,
            vec![
                MacroAntigenCategory::SubstrateAlignment,
                MacroAntigenCategory::FunctionalCorrectness
            ]
        );
    }

    #[test]
    fn antigen_parser_accepts_bare_path_without_type_prefix() {
        // Accept `SubstrateAlignment` without `AntigenCategory::` prefix
        let tokens: TokenStream =
            r#"name = "x", fingerprint = "item = fn", category = SubstrateAlignment"#
                .parse()
                .unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        assert_eq!(
            args.category,
            vec![MacroAntigenCategory::SubstrateAlignment]
        );
    }

    #[test]
    fn antigen_parser_accepts_absent_category_for_compat() {
        // v0.1 backward-compat: absent category is fine at parse time
        let tokens: TokenStream = r#"name = "x", fingerprint = "item = fn""#.parse().unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        assert!(
            args.category.is_empty(),
            "absent category should yield empty vec"
        );
    }

    #[test]
    fn antigen_parser_rejects_unknown_category_variant() {
        let tokens: TokenStream =
            r#"name = "x", fingerprint = "item = fn", category = AntigenCategory::Unknown"#
                .parse()
                .unwrap();
        let result = syn::parse2::<AntigenArgs>(tokens);
        assert!(
            result.is_err(),
            "unknown category variant should be rejected"
        );
    }

    #[test]
    fn antigen_parser_rejects_empty_category_array() {
        let tokens: TokenStream = r#"name = "x", fingerprint = "item = fn", category = []"#
            .parse()
            .unwrap();
        let result = syn::parse2::<AntigenArgs>(tokens);
        assert!(result.is_err(), "empty category array should be rejected");
    }

    #[test]
    fn macro_antigen_category_as_str_roundtrip() {
        for (variant, expected) in [
            (
                MacroAntigenCategory::SubstrateAlignment,
                "substrate-alignment",
            ),
            (
                MacroAntigenCategory::FunctionalCorrectness,
                "functional-correctness",
            ),
        ] {
            assert_eq!(variant.as_str(), expected);
            assert_eq!(
                MacroAntigenCategory::from_path_str(variant.as_str()),
                Some(variant)
            );
        }
    }
}

// ============================================================================
// Property tests (W1) — proptest invariants over the macro-side parser surface.
//
// These proptests are the macro-side half of the cross-parser equivalence
// story. The matching scan-side proptests live in
// `antigen/src/scan.rs::tests::scan_parser_props`. The two test modules share:
//
//   - the same `valid_*` strategies (literal-copied; if you change one,
//     change the other in the same commit); and
//   - the same expected-outcome assertions for inputs both parsers accept.
//
// Because `proc-macro = true` crates cannot be linked as libraries, the two
// parsers cannot be invoked from a single test binary. The fixture-table
// approach in the same file (`ANTIGEN_PARSER_FIXTURES`) provides
// by-construction cross-parser checks at six concrete points; the proptest
// strategies fuzz the input space around the same grammar shape from each
// side independently. Drift between the two manifests as one side accepting
// inputs the other rejects — caught here on the macro side, caught there on
// the scan side.
//
// Cross-parser invariants asserted (per ADR-001 Amendment 1 C5
// drift-detection-at-scan-time, and the ATK-001-2 lesson):
//
//   I1 — equivalence-on-intersection: for any input the macro side accepts,
//        the scan side accepts and produces equivalent semantic content for
//        name/fingerprint/family/summary. (Macro side checks "I accept";
//        scan side checks "I accept and the result matches what I'd render
//        back into the macro grammar.")
//
//   I2 — strict-superset-of-rejection: the macro side strictly rejects more
//        than the scan side (asymmetric by design — see scan.rs comments on
//        unknown-field tolerance and missing-required-field tolerance).
//        Rejecting more is fine; accepting where the macro rejects is not.
//
// Adversarial input shapes worth fuzzing (per W1's adversarial-pass plan):
// Unicode in names, nested macros / inner-quoted strings in fingerprints,
// extremely long string literals, malformed array literals, multi-line
// rustfmt output, all-whitespace edge cases, kebab-case boundary inputs.
// ============================================================================

#[cfg(test)]
mod parser_props {
    use super::*;
    use proc_macro2::TokenStream;
    use proptest::prelude::*;

    // Rust reserved words that cannot appear as path segments. Generated by
    // strategy `[a-z][a-z_0-9]{0,8}` without this filter, causing syn to reject
    // inputs that are otherwise syntactically correct `#[immune]` bodies.
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "box", "break", "const", "continue", "crate", "do", "dyn", "else",
        "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "macro",
        "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super",
        "trait", "true", "type", "union", "unsafe", "use", "where", "while", "yield", "abstract",
        "become", "final", "override", "priv", "try",
    ];

    // --- Strategies (shared shape with antigen/src/scan.rs::tests) ---

    /// Generate a kebab-case name: `[a-z][a-z0-9]*(-[a-z0-9]+)*`. The
    /// substrate's `is_kebab_case` rule rejects leading/trailing hyphens
    /// and consecutive double-hyphens; this strategy generates only legal
    /// shapes so we can lock the validate-accepts side. (Rejection of
    /// non-kebab is tested by the existing fixture tests.)
    fn valid_kebab() -> impl Strategy<Value = String> {
        // 1-4 segments, each 1-8 chars from [a-z0-9] and starting with [a-z].
        proptest::collection::vec(
            (
                proptest::char::range('a', 'z'),
                proptest::collection::vec(
                    prop_oneof![
                        proptest::char::range('a', 'z'),
                        proptest::char::range('0', '9')
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

    /// Generate a non-empty string suitable for use as a fingerprint /
    /// summary / family content. Avoids characters that would close the
    /// outer `"..."` literal at the token level (since these end up
    /// embedded in a Rust source-text string literal we synthesize). We
    /// allow inner content that includes escaped quotes via the
    /// fixture-style escape `\"` — the serialization layer handles
    /// escaping.
    fn valid_text(max_len: usize) -> impl Strategy<Value = String> {
        // Keep characters in a printable-ASCII range that can be safely
        // round-tripped through `Debug` formatting (which is how we emit
        // string literals into the synthetic source). Excludes: backslash
        // (escape complications), null bytes, raw quotes (the encoder will
        // escape them anyway, but we keep the strategy simple here).
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

    /// Encode a Rust string literal: wrap in double-quotes and escape
    /// inner quotes/backslashes via `format!("{:?}", s)`, which is the
    /// canonical Debug-encoding for `String` and matches what
    /// `syn::LitStr` accepts/produces.
    fn lit(s: &str) -> String {
        format!("{s:?}")
    }

    /// Render a `(name, fingerprint, family, summary)` tuple as the
    /// canonical `#[antigen(...)]` body in name-first order.
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
        // P1 — round-trip: any valid set of args parses, and re-rendering it
        // produces a body that re-parses to the same args.
        #[test]
        fn antigen_parser_round_trip(
            name in valid_kebab(),
            fingerprint in valid_text(64),
            family in proptest::option::of(valid_text(32)),
            summary in proptest::option::of(valid_text(64)),
        ) {
            let body = render_antigen_body(&name, &fingerprint, family.as_deref(), summary.as_deref());
            let tokens: TokenStream = body.parse().expect("body must tokenize");
            let args = syn::parse2::<AntigenArgs>(tokens).expect("body must parse");
            prop_assert_eq!(&args.name, &name);
            prop_assert_eq!(&args.fingerprint, &fingerprint);
            prop_assert_eq!(args.family.as_deref(), family.as_deref());
            prop_assert_eq!(args.summary.as_deref(), summary.as_deref());
            // W6a: validate() now invokes antigen_fingerprint::Fingerprint::parse,
            // which rejects arbitrary text. The round-trip property is about
            // parse-render-parse idempotency for the value fields, not about
            // DSL validity; drop the validate() assertion here. A separate
            // proptest with a valid_dsl() strategy is future work.

            // Round-trip: re-render the parsed args and re-parse. Result
            // must be identical (idempotency under the canonical rendering).
            let re_rendered = render_antigen_body(&args.name, &args.fingerprint, args.family.as_deref(), args.summary.as_deref());
            let re_tokens: TokenStream = re_rendered.parse().expect("re-rendered body must tokenize");
            let args2 = syn::parse2::<AntigenArgs>(re_tokens).expect("re-rendered body must parse");
            prop_assert_eq!(&args.name, &args2.name);
            prop_assert_eq!(&args.fingerprint, &args2.fingerprint);
            prop_assert_eq!(args.family, args2.family);
            prop_assert_eq!(args.summary, args2.summary);
        }

        // P2 — order-invariance: shuffling the order of valid fields does
        // not change the parsed result.
        #[test]
        fn antigen_parser_order_invariant(
            name in valid_kebab(),
            fingerprint in valid_text(48),
            family in valid_text(24),
            summary in valid_text(48),
        ) {
            // Two orderings: name-first (canonical) and reversed.
            let canonical = format!(
                "name = {}, fingerprint = {}, family = {}, summary = {}",
                lit(&name), lit(&fingerprint), lit(&family), lit(&summary),
            );
            let reversed = format!(
                "summary = {}, family = {}, fingerprint = {}, name = {}",
                lit(&summary), lit(&family), lit(&fingerprint), lit(&name),
            );
            let a: AntigenArgs = syn::parse2(canonical.parse::<TokenStream>().unwrap()).unwrap();
            let b: AntigenArgs = syn::parse2(reversed.parse::<TokenStream>().unwrap()).unwrap();
            prop_assert_eq!(&a.name, &b.name);
            prop_assert_eq!(&a.fingerprint, &b.fingerprint);
            prop_assert_eq!(&a.family, &b.family);
            prop_assert_eq!(&a.summary, &b.summary);
        }

        // P3 — kebab-case validator accepts every kebab-case string our
        // generator produces. (Negative shapes are tested by the fixture
        // tests — `validate_rejects_*` — already.)
        #[test]
        fn is_kebab_case_accepts_generator(name in valid_kebab()) {
            prop_assert!(is_kebab_case(&name), "is_kebab_case rejected generator output: {name:?}");
        }

        // P4 — required-field enforcement: any input missing `name` is
        // rejected with an error mentioning `name`. Same for `fingerprint`.
        #[test]
        fn antigen_parser_requires_name(
            fingerprint in valid_text(32),
            family in proptest::option::of(valid_text(16)),
        ) {
            let mut parts = vec![format!("fingerprint = {}", lit(&fingerprint))];
            if let Some(f) = &family {
                parts.push(format!("family = {}", lit(f)));
            }
            let body = parts.join(", ");
            let tokens: TokenStream = body.parse().expect("body tokenizes");
            let result = syn::parse2::<AntigenArgs>(tokens);
            prop_assert!(result.is_err(), "expected parser to reject input missing `name`: {body:?}");
            let err = result.unwrap_err().to_string();
            prop_assert!(err.contains("name"), "error must mention `name`, got: {err:?}");
        }

        #[test]
        fn antigen_parser_requires_fingerprint(
            name in valid_kebab(),
            family in proptest::option::of(valid_text(16)),
        ) {
            let mut parts = vec![format!("name = {}", lit(&name))];
            if let Some(f) = &family {
                parts.push(format!("family = {}", lit(f)));
            }
            let body = parts.join(", ");
            let tokens: TokenStream = body.parse().expect("body tokenizes");
            let result = syn::parse2::<AntigenArgs>(tokens);
            prop_assert!(result.is_err(), "expected parser to reject input missing `fingerprint`: {body:?}");
            let err = result.unwrap_err().to_string();
            prop_assert!(err.contains("fingerprint"), "error must mention `fingerprint`, got: {err:?}");
        }

        // P5 — unknown-field rejection (macro-side strictness; the scan
        // side tolerates these — that's the documented asymmetry).
        #[test]
        fn antigen_parser_rejects_unknown_field(
            name in valid_kebab(),
            fingerprint in valid_text(32),
            // Generate an unknown field name that doesn't collide with any
            // of the known field names.
            unknown in "[a-z][a-z_]{2,12}".prop_filter(
                "must not collide with known fields or Rust keywords",
                |s| {
                    !matches!(s.as_str(), "name" | "fingerprint" | "family" | "summary" | "references")
                        && !RUST_KEYWORDS.contains(&s.as_str())
                },
            ),
        ) {
            let body = format!(
                "name = {}, fingerprint = {}, {} = \"x\"",
                lit(&name), lit(&fingerprint), unknown,
            );
            let tokens: TokenStream = body.parse().expect("body tokenizes");
            let result = syn::parse2::<AntigenArgs>(tokens);
            prop_assert!(result.is_err(), "expected unknown field rejection for: {body:?}");
            let err = result.unwrap_err().to_string();
            prop_assert!(
                err.contains("unknown") && err.contains(&unknown),
                "error must name the unknown field. got: {err:?}",
            );
        }

        // P6 — references array round-trips: any list of valid strings in
        // the references array parses without error and we record them.
        #[test]
        fn antigen_parser_accepts_references_array(
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
            let args = syn::parse2::<AntigenArgs>(tokens).expect("body parses");
            prop_assert_eq!(&args.references, &refs);
        }

        // P7 — ImmuneArgs: any valid (path, witness-path) pair parses
        // and validate() accepts.
        //
        // Strategy note: `[a-z][a-z_0-9]{0,8}` can generate Rust reserved
        // words (fn, if, in, let, mod, …). syn rejects reserved words as
        // path segments, so we filter them out. The filter does not weaken the
        // property: the invariant is about VALID witness paths, and keywords
        // are not valid path segments.
        #[test]
        fn immune_parser_accepts_witness(
            antigen in "[A-Z][A-Za-z0-9]{0,16}",
            witness_segments in proptest::collection::vec(
                "[a-z][a-z_0-9]{0,8}".prop_filter("must not be a Rust keyword", |s| {
                    !RUST_KEYWORDS.contains(&s.as_str())
                }),
                1..4usize,
            ),
        ) {
            let witness = witness_segments.join("::");
            let body = format!("{antigen}, witness = {witness}");
            let tokens: TokenStream = body.parse().expect("body tokenizes");
            let args = syn::parse2::<ImmuneArgs>(tokens).expect("body parses");
            prop_assert!(args.witness.is_some());
            prop_assert!(args.validate().is_ok());
        }

        // P8 — ImmuneArgs: missing witness => validate() errors with
        // the witness-required message. (The Parse impl accepts a bare
        // antigen path; validate() is the trust-boundary check.)
        #[test]
        fn immune_parser_validate_rejects_missing_witness(
            antigen in "[A-Z][A-Za-z0-9]{0,16}",
        ) {
            let tokens: TokenStream = antigen.parse().expect("antigen tokenizes");
            let args = syn::parse2::<ImmuneArgs>(tokens).expect("bare path parses");
            prop_assert!(args.witness.is_none());
            let err = args.validate().unwrap_err().to_string();
            prop_assert!(err.contains("witness"), "validate must mention `witness`, got: {err:?}");
        }
    }
}

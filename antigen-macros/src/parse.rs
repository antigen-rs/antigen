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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacroAntigenCategory {
    SubstrateAlignment,
    FunctionalCorrectness,
}

impl MacroAntigenCategory {
    /// Parse from path-style expression strings produced by the proc-macro parser.
    ///
    /// Accepted forms match what a developer can write in `category = <value>`:
    /// - `SubstrateAlignment` (Pascal ident, unqualified import)
    /// - `AntigenCategory::SubstrateAlignment` (fully-qualified path)
    ///
    /// Kebab and snake are NOT accepted: they are not valid Rust path tokens and
    /// no proc-macro input source produces them.
    fn from_path_str(s: &str) -> Option<Self> {
        match s {
            "SubstrateAlignment" | "AntigenCategory::SubstrateAlignment" => {
                Some(Self::SubstrateAlignment)
            }
            "FunctionalCorrectness" | "AntigenCategory::FunctionalCorrectness" => {
                Some(Self::FunctionalCorrectness)
            }
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

/// Arguments to `#[presents(antigen_type, [requires = <predicate>], [proof = <expr>])]`.
///
/// ADR-029 (R5): site-attached defense evidence folds into `#[presents]`. A
/// presents-site may optionally carry:
/// - `requires = <predicate>` — substrate-tier evidence (the predicate the audit
///   must verify against `.attest/` sidecars; same grammar as the deprecated
///   `#[immune(requires = ...)]`). The substrate-witness migration target.
/// - `proof = <expr>` — phantom-tier evidence (a type-system construction whose
///   existence is the proof, e.g. `NonPanickingProof::<T>::verified`). The
///   phantom-witness migration target.
///
/// Both are *site-attached* evidence (the evidence IS at the site). Code-tier
/// evidence (a test elsewhere) registers via `#[defended_by]` on the test, not
/// here — the R5 discriminator: evidence belongs where it is.
#[derive(Debug)]
pub struct PresentsArgs {
    #[allow(dead_code)]
    pub antigen: Path,
    /// Substrate-tier evidence (ADR-029 R5; substrate-witness predicate, ADR-019).
    pub requires: Option<(RequiresExpr, Span)>,
    /// Phantom-tier evidence (ADR-029 R5; type-system construction).
    #[allow(dead_code)]
    pub proof: Option<Expr>,
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

/// Arguments to `#[defended_by(antigen_type)]` (ADR-029).
///
/// Code-tier witness registration: a `#[test]` / proptest function declares
/// *what failure-class it defends*. The cross-reference to the presents-sites
/// it covers — and whether it actually defends them — is computed by
/// `cargo antigen audit`, not asserted here. Immunity is observed, not declared.
///
/// Single positional argument: the antigen type path (e.g.
/// `ParallelStateTrackersDiverge` or `crate::antigens::Foo`).
#[derive(Debug)]
pub struct DefendedByArgs {
    #[allow(dead_code)]
    pub antigen: Path,
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
        // ADR-028: category is a SET (SubstrateAlignment | FunctionalCorrectness | both).
        // Vec<MacroAntigenCategory> must not contain duplicate entries — duplicates
        // look like hybrid antigens (len == 2) but are not.
        {
            let mut seen = std::collections::HashSet::new();
            for cat in &self.category {
                if !seen.insert(cat) {
                    return Err(syn::Error::new(
                        self.args_span,
                        "duplicate AntigenCategory variant in `category` array; \
                         each category variant may appear at most once",
                    ));
                }
            }
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
        let mut requires: Option<(RequiresExpr, Span)> = None;
        let mut proof: Option<Expr> = None;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            let key_span = key.span();
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "requires" => {
                    let pred: RequiresExpr = input.parse()?;
                    requires = Some((pred, key_span));
                }
                "proof" => {
                    proof = Some(input.parse()?);
                }
                "witness" => {
                    // Reject `witness =` on #[presents] with a clear migration message.
                    // Code-tier witnesses register via `#[defended_by(X)]` on the test
                    // function, not `witness =` on the presents-site (ADR-029 R5
                    // discriminator: evidence belongs where it is).
                    let _: Expr = input.parse()?; // consume the value
                    return Err(syn::Error::new(
                        key_span,
                        "`witness = ...` is not valid on `#[presents]`. \
                         Code-tier witnesses register via `#[defended_by(X)]` on the \
                         test/proptest function, not on the presents-site. \
                         For substrate-tier evidence use `requires = ...` here. \
                         For phantom-tier evidence use `proof = ...` here.",
                    ));
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[presents] field `{other}`; expected one of: \
                             requires, proof"
                        ),
                    ));
                }
            }
        }

        Ok(Self {
            antigen,
            requires,
            proof,
        })
    }
}

impl PresentsArgs {
    pub fn validate(&self) -> syn::Result<()> {
        // Validate the requires predicate if present.
        if let Some((pred, span)) = &self.requires {
            pred.validate(*span)?;
        }
        Ok(())
    }

    /// If `requires` is set, return the JSON string for the predicate.
    /// The scan layer reads this from the `antigen:requires:v1:` doc marker.
    pub fn requires_json(&self) -> Option<String> {
        self.requires.as_ref().map(|(pred, _)| pred.to_json())
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
                 For compound evidence across code-tier and substrate-tier (e.g., \
                 hybrid antigens per ADR-028), stack two `#[immune]` attributes on \
                 the same item — one with `witness = ...`, one with `requires = ...`. \
                 Audit treats stacked `#[immune]` attributes as independent coverage \
                 entries.",
            )),
            (_, Some((pred, span))) => pred.validate(*span),
            (Some(witness), None) => {
                // A witness must be a bare identifier or path to the verifying
                // item (a test fn, lint reference, proof, or phantom-type
                // construction). A STRING LITERAL (`witness = "my_test"`) is
                // accepted by `Expr` parsing but silently never resolves: the
                // audit-time resolver compares fn names against the witness
                // token, and a string literal carries its quote characters, so
                // it searches for a fn literally named `"my_test"` and always
                // reports "broken: no function found." Reject it loudly here —
                // the macro is the earliest, clearest place to catch the
                // mis-shape (ADR-005 sub-clause F: a trust-boundary input
                // accepted in an unhonorable shape is not a valid claim).
                if let Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = witness
                {
                    return Err(syn::Error::new_spanned(
                        witness,
                        format!(
                            "#[immune] `witness` must be a bare identifier or path to the \
                             verifying item (e.g. `witness = my_test`), not a string literal. \
                             `witness = \"{}\"` silently never resolves at audit time because \
                             the resolver matches function names against the token, and a \
                             string literal carries its quotes. Drop the quotes: \
                             `witness = {}`.",
                            s.value(),
                            s.value()
                        ),
                    ));
                }
                Ok(())
            }
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
// DefendedByArgs parsing (ADR-029)
// ============================================================================

impl Parse for DefendedByArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let antigen: Path = input.parse()?;
        if !input.is_empty() {
            // `#[defended_by]` is a pure code-tier witness registration: it
            // carries only the failure-class it defends. Site-attached evidence
            // (`requires=`, `proof=`) folds into `#[presents]` instead (ADR-029
            // R5 discriminator: evidence belongs where it is). Reject trailing
            // tokens loudly so a developer who reaches for `#[defended_by(X,
            // witness = ...)]` (the old `#[immune]` shape) is pointed at the
            // right primitive rather than having the extra args silently dropped.
            return Err(syn::Error::new(
                input.span(),
                "#[defended_by] takes exactly one positional argument: the antigen \
                 type it defends, e.g. `#[defended_by(ParallelStateTrackersDiverge)]`. \
                 It registers a code-tier witness (a test/proptest); it does NOT carry \
                 `witness =`/`requires =`/`proof =`. Site-attached evidence folds into \
                 `#[presents]` (ADR-029).",
            ));
        }
        Ok(Self { antigen })
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
            Some(r) if r.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.reason_span.unwrap_or(self.args_span),
                    "#[anergy] `reason` must not be whitespace-only; \
                     a blank reason bypasses the loudness-as-discipline \
                     requirement (ADR-023).",
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

        // Past-date rejection: an expired anergy window means the suppression
        // should have been re-evaluated but wasn't. Parallels OrientArgs (53d2bab).
        if let Some(until_str) = self.until.as_deref() {
            match parse_iso_date(until_str) {
                Ok(until_date) => {
                    let horizon_days = (until_date - today_utc()).num_days();
                    if horizon_days < 0 {
                        return Err(syn::Error::new(
                            self.until_span.unwrap_or(self.args_span),
                            format!(
                                "#[anergy] `until` ({until_str}) is {} day(s) in the past — \
                                 an already-expired anergy window is silent suppression with \
                                 no accountability. Set a future `until` date or remove the \
                                 #[anergy] marker.",
                                -horizon_days
                            ),
                        ));
                    }
                }
                Err(()) => {
                    return Err(syn::Error::new(
                        self.until_span.unwrap_or(self.args_span),
                        format!(
                            "#[anergy] `until` value {until_str:?} is not a valid ISO-8601 \
                             date. Use YYYY-MM-DD format, e.g. `until = \"2026-12-31\"`."
                        ),
                    ));
                }
            }
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

/// Default `#[orient]` orientation-period horizon per ADR-023
/// (`deferred_defense_max_horizon`, 180 days). `until` dates beyond
/// `now + this` are rejected at parse time.
pub const ORIENT_DEFAULT_MAX_HORIZON_DAYS: i64 = 180;

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
            Some(r) if r.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    "#[immunosuppress] `rationale` must not be whitespace-only; \
                     a blank rationale bypasses the loudness-as-discipline \
                     requirement (ADR-023).",
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
            match parse_iso_date(until_str) {
                Ok(until_date) => {
                    let since_date = self
                        .since
                        .as_deref()
                        .and_then(|s| parse_iso_date(s).ok())
                        .unwrap_or_else(today_utc);
                    let duration_days = (until_date - since_date).num_days();
                    if duration_days < 0 {
                        return Err(syn::Error::new(
                            self.until_span.unwrap_or(self.args_span),
                            format!(
                                "#[immunosuppress] `until` ({}) is {} day(s) in the past — \
                                 an expired suppression window is silent check-disabling with \
                                 no accountability. Set a future `until` date or remove the \
                                 #[immunosuppress] marker.",
                                until_str, -duration_days
                            ),
                        ));
                    }
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
                Err(()) => {
                    return Err(syn::Error::new(
                        self.until_span.unwrap_or(self.args_span),
                        format!(
                            "#[immunosuppress] `until` value {until_str:?} is not a valid \
                             ISO-8601 date. Use YYYY-MM-DD format, e.g. `until = \"2026-12-31\"`."
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
            Some(et) if et.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.exercise_type_span.unwrap_or(self.args_span),
                    "#[poxparty] `exercise_type` must not be whitespace-only; \
                     a blank exercise type bypasses the loudness-as-discipline \
                     requirement (ADR-023).",
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

        // Past-date rejection: an expired pox-party window is stale controlled
        // exposure with no accountability. Parallels OrientArgs (53d2bab).
        if let Some(until_str) = self.until.as_deref() {
            match parse_iso_date(until_str) {
                Ok(until_date) => {
                    let horizon_days = (until_date - today_utc()).num_days();
                    if horizon_days < 0 {
                        return Err(syn::Error::new(
                            self.until_span.unwrap_or(self.args_span),
                            format!(
                                "#[poxparty] `until` ({until_str}) is {} day(s) in the past — \
                                 an expired pox-party exercise is stale controlled exposure with \
                                 no accountability. Set a future `until` date or remove the \
                                 #[poxparty] marker.",
                                -horizon_days
                            ),
                        ));
                    }
                }
                Err(()) => {
                    return Err(syn::Error::new(
                        self.until_span.unwrap_or(self.args_span),
                        format!(
                            "#[poxparty] `until` value {until_str:?} is not a valid ISO-8601 \
                             date. Use YYYY-MM-DD format, e.g. `until = \"2026-12-31\"`."
                        ),
                    ));
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// OrientArgs parsing (ADR-023)
// ============================================================================

/// Arguments to `#[orient(antigen, learning_path, until)]`.
///
/// Per ADR-023 §Decision + aristotle's `orient-field-optionality-ruling`
/// (Option A, hard break): orient acknowledges a pre-immunity learning period
/// and `learning_path` + `until` are **both REQUIRED** — they are the
/// accountability fields the primitive exists to impose. Making them optional
/// would collapse `#[orient]` to structurally-identical-to-silent-tolerance,
/// the exact A5 failure the deferred-defense family was built to prevent
/// (loudness-as-discipline). A bare `#[orient]` is therefore a parse error.
///
/// The pre-restoration drift-form fields (`see`, `adr`, `attestation_optional`)
/// are NOT in the ADR-023 spec — they were the drift — and are REMOVED
/// entirely (a hard parse error, not a deprecation window).
///
/// Decisional rollback-as-triage uses are handled by `#[triage_commit]`
/// (sibling primitive per ADR-026 Amendment 1), NOT by `#[orient]`.
///
/// - `<antigen-path>` (optional positional) — failure-class antigen path
/// - `learning_path = "..."` (REQUIRED, 20-char-min, rationale-class) — the
///   explicit path forward out of the learning period
/// - `until = "YYYY-MM-DD"` (REQUIRED) — orientation-period horizon; non-empty,
///   UTC, parse-validated, rejected if beyond `now + deferred_defense_max_horizon`
///   (180d default)
#[derive(Debug)]
pub struct OrientArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    #[allow(dead_code)]
    pub learning_path: Option<String>,
    /// Span of the `learning_path` value, for anchoring validation errors.
    #[allow(dead_code)]
    pub learning_path_span: Option<Span>,
    #[allow(dead_code)]
    pub until: Option<String>,
    /// Span of the `until` value (for anchoring validation errors).
    #[allow(dead_code)]
    pub until_span: Option<Span>,
    #[allow(dead_code)]
    pub args_span: Span,
}

impl Parse for OrientArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut learning_path: Option<String> = None;
        let mut learning_path_span: Option<Span> = None;
        let mut until: Option<String> = None;
        let mut until_span: Option<Span> = None;

        // Optional leading positional antigen type path.
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
                "learning_path" => {
                    let lit: LitStr = input.parse()?;
                    learning_path_span = Some(lit.span());
                    learning_path = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until_span = Some(lit.span());
                    until = Some(lit.value());
                }
                // The drift-form fields (see/adr/attestation_optional) are NOT
                // in the ADR-023 spec; restoration removes them. A clear error
                // names the migration target rather than silently accepting.
                "see" | "adr" | "attestation_optional" => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "#[orient] field `{key}` was removed in the ADR-023 restoration \
                             (it was never in the spec). The orient spec is \
                             `#[orient(<antigen>, learning_path = \"...\", until = \"YYYY-MM-DD\")]`. \
                             Migrate `see`/`adr` context into the `learning_path` text (or \
                             `references = [...]` on the antigen declaration); \
                             `attestation_optional` inverted loudness-as-discipline and has no \
                             replacement."
                        ),
                    ));
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[orient] field `{other}`; expected: learning_path, until"
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
            learning_path,
            learning_path_span,
            until,
            until_span,
            args_span,
        })
    }
}

impl OrientArgs {
    /// Trust-boundary checks per ADR-023 §Decision + the Option-A
    /// (hard-break) ruling:
    /// - `learning_path` REQUIRED, minimum 20 characters (rationale-class).
    /// - `until` REQUIRED, non-empty, UTC-parseable, and within
    ///   `now + ORIENT_DEFAULT_MAX_HORIZON_DAYS` (180d) — a horizon beyond that
    ///   is a parse-time COMPILE ERROR.
    ///
    /// A bare `#[orient]` (neither field) fails on the first check. Optionality
    /// is rejected on first principles: an orient without an explicit path-out
    /// and a time-bound is silent deferred non-immunity — exactly tolerance,
    /// which this loud primitive exists to be distinct from.
    pub fn validate(&self) -> syn::Result<()> {
        // learning_path required + 20-char minimum.
        match self.learning_path.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[orient] requires `learning_path = \"...\"`. An orientation period \
                     without an explicit path forward is silent deferred non-immunity \
                     (= tolerance); orient exists to be loud about it (ADR-023).",
                ));
            }
            Some(p) if p.len() < 20 => {
                return Err(syn::Error::new(
                    self.learning_path_span.unwrap_or(self.args_span),
                    format!(
                        "#[orient] `learning_path` must be at least 20 characters (got {}); \
                         per ADR-023 loudness-as-discipline (rationale-class field).",
                        p.len()
                    ),
                ));
            }
            Some(p) if p.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.learning_path_span.unwrap_or(self.args_span),
                    "#[orient] `learning_path` must not be whitespace-only; \
                     a blank learning path bypasses the loudness-as-discipline \
                     requirement (ADR-023).",
                ));
            }
            _ => {}
        }

        // until required + non-empty.
        let until_str = match self.until.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[orient] requires `until = \"YYYY-MM-DD\"`. An orientation period \
                     without a time-bound is indefinite; orient must escalate at a horizon.",
                ));
            }
            Some("") => {
                return Err(syn::Error::new(
                    self.until_span.unwrap_or(self.args_span),
                    "#[orient] `until` must not be empty.",
                ));
            }
            Some(s) => s,
        };

        // Horizon enforcement: until must parse as a UTC date and must not be
        // beyond now + max_horizon (180d default). Parse-time COMPILE ERROR.
        match parse_iso_date(until_str) {
            Err(()) => {
                return Err(syn::Error::new(
                    self.until_span.unwrap_or(self.args_span),
                    format!(
                        "#[orient] `until` must be an ISO-8601 date (YYYY-MM-DD); got `{until_str}`."
                    ),
                ));
            }
            Ok(until_date) => {
                let horizon_days = (until_date - today_utc()).num_days();
                // A past `until` is an already-expired orientation window: it
                // says "we never addressed this failure-class AND the
                // accountability horizon has already passed" — silent expired
                // non-immunity, the exact thing orient's loudness prevents. The
                // too-far-future check below uses `> MAX`, which is false for
                // negative values, so a past date would otherwise pass silently.
                if horizon_days < 0 {
                    return Err(syn::Error::new(
                        self.until_span.unwrap_or(self.args_span),
                        format!(
                            "#[orient] `until` ({until_str}) is {} day(s) in the past — an \
                             already-expired orientation window. Set a future horizon, or if \
                             the period has genuinely lapsed, resolve the failure-class \
                             (declare immunity/tolerance) rather than leave an expired orient.",
                            -horizon_days
                        ),
                    ));
                }
                if horizon_days > ORIENT_DEFAULT_MAX_HORIZON_DAYS {
                    return Err(syn::Error::new(
                        self.until_span.unwrap_or(self.args_span),
                        format!(
                            "#[orient] `until` is {horizon_days}d out, beyond the \
                             {ORIENT_DEFAULT_MAX_HORIZON_DAYS}d orientation-period horizon \
                             (deferred_defense_max_horizon). An orientation period this long \
                             is not orientation — shorten `until` or address the failure-class."
                        ),
                    ));
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// MacroTriageDecision — local mirror of antigen::TriageDecision (ADR-026)
//
// proc-macro crates cannot depend on the `antigen` library crate (circular
// dependency), so we maintain a local parse-time mirror — same pattern as
// MacroAntigenCategory for ADR-028. The two enums stay in sync; extending
// either requires an ADR amendment per ADR-001 Amendment 1 C6.
// ============================================================================

/// Parse-time triage-decision variant (mirrors `antigen::vcs::TriageDecision`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroTriageDecision {
    /// System-down / data-loss imminent / catastrophic regression confirmed.
    Black,
    /// Vital-metric regression confirmed; tight-time-window rollback.
    Red,
    /// Concerning signal; investigation pending; rollback decision deferred.
    Yellow,
    /// No functional regression; analysis chain attests non-regression.
    Green,
    /// Out of scope for this triage event; explicit non-action chart entry.
    White,
}

impl MacroTriageDecision {
    /// Parse from path-style expression strings and common aliases.
    fn from_path_str(s: &str) -> Option<Self> {
        match s {
            "Black" | "TriageDecision::Black" | "black" => Some(Self::Black),
            "Red" | "TriageDecision::Red" | "red" => Some(Self::Red),
            "Yellow" | "TriageDecision::Yellow" | "yellow" => Some(Self::Yellow),
            "Green" | "TriageDecision::Green" | "green" => Some(Self::Green),
            "White" | "TriageDecision::White" | "white" => Some(Self::White),
            _ => None,
        }
    }

    /// Kebab-case string form mirroring `antigen::vcs::TriageDecision::as_str()`.
    ///
    /// Reserved for future audit-hint emission paths — kept on the parse-time
    /// mirror so the proc-macro crate doesn't need to import `antigen`
    /// (circular dep) when hint detail strings grow.
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::Red => "red",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::White => "white",
        }
    }
}

// ============================================================================
// TriageCommitArgs parsing (ADR-026 §Rollback-as-triage primitive)
//
// `#[triage_commit]` is the rollback-as-triage primitive ratified by ADR-026
// §Decision. Per aristotle's fixup-orient-dual-signature F1-F5 resolution
// (camp note 55a161e7), `#[triage_commit]` is a SIBLING primitive to
// `#[orient]`, NOT an extension — orient names a failure-class (acknowledged
// pre-immunity); triage_commit names a system-state-classification + commit
// to a rollback action within a time-bound. The clinical-medicine grounding
// (chart documentation + informed consent before procedure) is dual-axis-
// acknowledged per ADR-024 + ADR-026 dual-axis discipline.
// ============================================================================

/// Arguments to `#[triage_commit(triage_decision = ..., rollback_target = ...,
/// triaged_by = ..., rationale = ..., rollback_due_within_minutes = ...)]`.
///
/// All five fields are REQUIRED per ADR-026 §Decision rollback-as-triage
/// primitive. Per ADR-026 §Rollback-as-triage discipline (NON-NEGOTIABLE
/// per naturalist): the chart-documentation cognate demands the rationale +
/// `triaged_by` + time-bound be present before the action commits, NOT after.
///
/// Loudness-as-discipline (ADR-023 central pattern applied here): missing
/// fields are compile-time errors so a triage commit that elides documentation
/// cannot reach `git push`.
#[derive(Debug)]
pub struct TriageCommitArgs {
    pub triage_decision: Option<MacroTriageDecision>,
    /// Span of the `triage_decision` value (path expression). Reserved for
    /// future audit-hint emission paths that need to point error reports at
    /// the variant token; presently unused.
    #[allow(dead_code)]
    pub triage_decision_span: Option<Span>,
    pub rollback_target: Option<String>,
    pub rollback_target_span: Option<Span>,
    pub triaged_by: Option<String>,
    pub triaged_by_span: Option<Span>,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub rollback_due_within_minutes: Option<u32>,
    pub rollback_due_within_minutes_span: Option<Span>,
    pub args_span: Span,
}

/// Parse `triage_decision = TriageDecision::X` from a path expression.
/// Extracted from `Parse for TriageCommitArgs` to keep that impl under 100 lines.
fn parse_triage_decision_expr(
    input: syn::parse::ParseStream,
    expr: &Expr,
) -> syn::Result<(MacroTriageDecision, Span)> {
    let expr_span = match expr {
        Expr::Path(p) => p
            .path
            .segments
            .first()
            .map_or_else(|| input.span(), |s| s.ident.span()),
        _ => input.span(),
    };
    let s = match expr {
        Expr::Path(p) => {
            let segs: Vec<String> = p
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect();
            segs.join("::")
        }
        _ => {
            return Err(syn::Error::new_spanned(
                expr,
                "expected `TriageDecision::Black`, `TriageDecision::Red`, \
                 `TriageDecision::Yellow`, `TriageDecision::Green`, or \
                 `TriageDecision::White`",
            ));
        }
    };
    let decision = MacroTriageDecision::from_path_str(&s).ok_or_else(|| {
        syn::Error::new_spanned(
            expr,
            format!(
                "unknown TriageDecision `{s}`; expected one of: Black, Red, \
                 Yellow, Green, White"
            ),
        )
    })?;
    Ok((decision, expr_span))
}

impl Parse for TriageCommitArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut triage_decision: Option<MacroTriageDecision> = None;
        let mut triage_decision_span: Option<Span> = None;
        let mut rollback_target: Option<String> = None;
        let mut rollback_target_span: Option<Span> = None;
        let mut triaged_by: Option<String> = None;
        let mut triaged_by_span: Option<Span> = None;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;
        let mut rollback_due_within_minutes: Option<u32> = None;
        let mut rollback_due_within_minutes_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "triage_decision" => {
                    let expr: Expr = input.parse()?;
                    let (decision, span) = parse_triage_decision_expr(input, &expr)?;
                    triage_decision = Some(decision);
                    triage_decision_span = Some(span);
                }
                "rollback_target" => {
                    let lit: LitStr = input.parse()?;
                    rollback_target_span = Some(lit.span());
                    rollback_target = Some(lit.value());
                }
                "triaged_by" => {
                    let lit: LitStr = input.parse()?;
                    triaged_by_span = Some(lit.span());
                    triaged_by = Some(lit.value());
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                "rollback_due_within_minutes" => {
                    let lit: syn::LitInt = input.parse()?;
                    rollback_due_within_minutes_span = Some(lit.span());
                    rollback_due_within_minutes = Some(lit.base10_parse::<u32>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[triage_commit] field `{other}`; expected one of: \
                             triage_decision, rollback_target, triaged_by, rationale, \
                             rollback_due_within_minutes"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            triage_decision,
            triage_decision_span,
            rollback_target,
            rollback_target_span,
            triaged_by,
            triaged_by_span,
            rationale,
            rationale_span,
            rollback_due_within_minutes,
            rollback_due_within_minutes_span,
            args_span,
        })
    }
}

impl TriageCommitArgs {
    /// Trust-boundary checks per ADR-026 §Decision rollback-as-triage:
    /// - `triage_decision` REQUIRED — the loud-acknowledgment IS the discipline
    /// - `rollback_target` REQUIRED, non-empty — sha pointer to last-known-good
    /// - `triaged_by` REQUIRED, non-empty — informed-consent author identity
    /// - `rationale` REQUIRED, minimum 20 characters — chart-documentation
    ///   discipline per naturalist's clinical-medicine grounding
    ///   (parallel to `#[anergy]` 20-char floor)
    /// - `rollback_due_within_minutes` REQUIRED, positive — tight time-bound
    ///   carrier; 0 would mean immediate-or-never and degrades discipline
    pub fn validate(&self) -> syn::Result<()> {
        if self.triage_decision.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[triage_commit] requires `triage_decision = TriageDecision::X` \
                 (one of Black, Red, Yellow, Green, White). Per ADR-026 \
                 §Rollback-as-triage, the loud-acknowledgment IS the discipline.",
            ));
        }
        match self.rollback_target.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[triage_commit] requires `rollback_target = \"<sha>\"` \
                     (commit sha pointing to last-known-good state).",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.rollback_target_span.unwrap_or(self.args_span),
                    "#[triage_commit] `rollback_target` cannot be empty or whitespace-only. \
                     A rollback without a target is not a rollback.",
                ));
            }
            Some(_) => {}
        }
        match self.triaged_by.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[triage_commit] requires `triaged_by = \"<role|name>\"`. \
                     Per ADR-026 §Rollback-as-triage clinical-medicine \
                     grounding, informed-consent requires an authoring identity.",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.triaged_by_span.unwrap_or(self.args_span),
                    "#[triage_commit] `triaged_by` cannot be empty or whitespace-only.",
                ));
            }
            Some(_) => {}
        }
        match self.rationale.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[triage_commit] requires `rationale = \"...\"` \
                     (chart-documentation; minimum 20 characters). Per ADR-026 \
                     §Rollback-as-triage: rationale before action, not after.",
                ));
            }
            Some(s) if s.len() < 20 => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    format!(
                        "#[triage_commit] `rationale` must be at least 20 \
                         characters (got {}); per ADR-023 loudness-as-discipline \
                         applied to clinical-medicine chart-documentation.",
                        s.len()
                    ),
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    "#[triage_commit] `rationale` must not be whitespace-only; \
                     a blank rationale bypasses chart-documentation discipline \
                     (ADR-026 §Rollback-as-triage: rationale before action, not after).",
                ));
            }
            Some(_) => {}
        }
        match self.rollback_due_within_minutes {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[triage_commit] requires `rollback_due_within_minutes = N` \
                     (positive integer; e.g., 30 for a Red triage per ADR-026 example).",
                ));
            }
            Some(0) => {
                return Err(syn::Error::new(
                    self.rollback_due_within_minutes_span
                        .unwrap_or(self.args_span),
                    "#[triage_commit] `rollback_due_within_minutes` must be > 0. \
                     A zero deadline degrades to no-deadline; per ADR-026 \
                     §Rollback-as-triage the time-bound carries discipline.",
                ));
            }
            Some(_) => {}
        }
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

/// The `WitnessClass` variant names this crate recognizes for `#[diagnostic]`
/// `modalities`. Mirrors `antigen::convergent::WitnessClass` (the sealed enum).
///
/// Held as a LOCAL allowlist because `antigen-macros` cannot depend on
/// `antigen` (circular: `antigen` depends on `antigen-macros`), so the macro
/// crate can't name the real enum at parse time. If `WitnessClass` gains a
/// variant, this list must grow in lockstep — itself a
/// `ParallelStateTrackersDiverge` site (two hand-maintained copies of one
/// variant set); the divergence surfaces the first time a new variant is used
/// in `modalities` and rejected here.
const KNOWN_WITNESS_CLASSES: &[&str] = &[
    "StaticAnalysis",
    "PropertyTest",
    "FormalVerification",
    "ManualReview",
    "RuntimeFuzz",
    "SubstrateWitness",
];

impl DiagnosticArgs {
    /// Per ADR-024 + adversarial C1: `min_independent` is REQUIRED;
    /// `modalities` must be non-empty; `min_independent` must not exceed
    /// the number of distinct modality classes (otherwise the claim is
    /// vacuously unsatisfiable). Each modality must also name a real
    /// `WitnessClass` variant — a sealed enum accepted as an arbitrary ident
    /// is `UnvalidatedSealedEnumAcceptance` (sub-clause F: validate the
    /// trust-boundary input against the closed set it claims to be).
    pub fn validate(&self) -> syn::Result<()> {
        if self.modality_classes.is_empty() {
            return Err(syn::Error::new(
                self.modality_span.unwrap_or(self.args_span),
                "#[diagnostic] requires non-empty `modalities = [WitnessClass::X, ...]` \
                 (ADR-024 §Decision; an empty modality set has no evidence to converge).",
            ));
        }
        // Sub-clause F: reject idents that don't name a real WitnessClass
        // variant. Without this, `modalities = [WitnessClass::Nonsense]` (or
        // any `Foo::Bar`) is silently captured as a bogus class string and the
        // distinct-count math runs on garbage.
        for class in &self.modality_classes {
            if !KNOWN_WITNESS_CLASSES.contains(&class.as_str()) {
                return Err(syn::Error::new(
                    self.modality_span.unwrap_or(self.args_span),
                    format!(
                        "#[diagnostic] `modalities` entry `{class}` is not a known \
                         WitnessClass variant; expected one of: {}. \
                         (A modality that names no real witness class is dead evidence — \
                         it can never converge.)",
                        KNOWN_WITNESS_CLASSES.join(", ")
                    ),
                ));
            }
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
// Recurrent-Emergence Family argument parsers (ADR-024 + scientist HOW-spec
// cf2a2317 + aristotle Reading-A pre-authorization 744471a3)
//
// Six present-looking primitives per ADR-024 §Family 2: #[itch],
// #[recurrence_anchor], #[crystallize], #[chronic], #[saturate], #[strand].
// Cognitive-organizational grounding for itch/saturate/crystallize/strand;
// immunology-proper for chronic; clinical-medicine for recurrence_anchor.
// All members declare antigen-category = SubstrateAlignment per ADR-028
// (representation of recurring pattern diverges from actual state).
// ============================================================================

/// Arguments to `#[itch(name, antigen?, description, threshold?)]`.
///
/// Cognitive-organizational primitive: "pattern noticed below threshold;
/// no commitment yet" per ADR-024 §Disambiguation table. Distinct from
/// `#[anergy]` (ADR-023, intentional non-defense while waiting): itch is
/// a pre-commitment NOTICING, anergy is a deliberate DEFER.
#[derive(Debug)]
pub struct ItchArgs {
    pub name: Option<String>,
    pub name_span: Option<Span>,
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub description: Option<String>,
    pub description_span: Option<Span>,
    #[allow(dead_code)]
    pub threshold: Option<String>,
    pub args_span: Span,
}

impl Parse for ItchArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut name: Option<String> = None;
        let mut name_span: Option<Span> = None;
        let mut antigen: Option<syn::Path> = None;
        let mut description: Option<String> = None;
        let mut description_span: Option<Span> = None;
        let mut threshold: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name_span = Some(lit.span());
                    name = Some(lit.value());
                }
                "antigen" => {
                    antigen = Some(input.parse()?);
                }
                "description" => {
                    let lit: LitStr = input.parse()?;
                    description_span = Some(lit.span());
                    description = Some(lit.value());
                }
                "threshold" => {
                    let lit: LitStr = input.parse()?;
                    threshold = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[itch] field `{other}`; expected one of: \
                             name, antigen, description, threshold"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name,
            name_span,
            antigen,
            description,
            description_span,
            threshold,
            args_span,
        })
    }
}

impl ItchArgs {
    /// Per scientist HOW-spec: name + description required; description ≥10 chars.
    pub fn validate(&self) -> syn::Result<()> {
        match self.name.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[itch] requires `name = \"<slug>\"` (kebab-case identifier).",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.name_span.unwrap_or(self.args_span),
                    "#[itch] `name` cannot be empty.",
                ));
            }
            Some(_) => {}
        }
        match self.description.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[itch] requires `description = \"...\"` (what is being noticed; \
                     minimum 10 characters).",
                ));
            }
            Some(s) if s.len() < 10 => {
                return Err(syn::Error::new(
                    self.description_span.unwrap_or(self.args_span),
                    format!(
                        "#[itch] `description` must be at least 10 characters \
                         (got {}); per cognitive-organizational discipline a noticing \
                         needs enough text to be useful next-time.",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[recurrence_anchor(antigen, instances, since, rationale)]`.
///
/// Clinical-medicine primitive: "cross-substrate recurrence; threshold
/// reached; want to surface for action" per ADR-024 §Disambiguation. The
/// recurrence-anchor commits to FORMAL recognition of a pattern that has
/// crossed the substrate-evidence threshold — analogous to a clinical
/// diagnosis after recurrent symptoms.
#[derive(Debug)]
pub struct RecurrenceAnchorArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub instances: Option<u32>,
    pub instances_span: Option<Span>,
    pub since: Option<String>,
    pub since_span: Option<Span>,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for RecurrenceAnchorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut instances: Option<u32> = None;
        let mut instances_span: Option<Span> = None;
        let mut since: Option<String> = None;
        let mut since_span: Option<Span> = None;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;

        // Optional leading positional antigen path
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
                "antigen" => {
                    antigen = Some(input.parse()?);
                }
                "instances" => {
                    let lit: syn::LitInt = input.parse()?;
                    instances_span = Some(lit.span());
                    instances = Some(lit.base10_parse::<u32>()?);
                }
                "since" => {
                    let lit: LitStr = input.parse()?;
                    since_span = Some(lit.span());
                    since = Some(lit.value());
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[recurrence_anchor] field `{other}`; expected \
                             one of: antigen, instances, since, rationale"
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
            instances,
            instances_span,
            since,
            since_span,
            rationale,
            rationale_span,
            args_span,
        })
    }
}

impl RecurrenceAnchorArgs {
    /// Per scientist HOW-spec: antigen + instances (> 0) + since + rationale
    /// (≥20 chars) all REQUIRED.
    pub fn validate(&self) -> syn::Result<()> {
        if self.antigen.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[recurrence_anchor] requires an `antigen` path (the failure-class \
                 being formally anchored as a cross-substrate recurrence).",
            ));
        }
        match self.instances {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[recurrence_anchor] requires `instances = N` (positive \
                     integer; how many recurrences have been observed).",
                ));
            }
            Some(0) => {
                return Err(syn::Error::new(
                    self.instances_span.unwrap_or(self.args_span),
                    "#[recurrence_anchor] `instances` must be > 0; an anchor at \
                     zero observed instances is structurally premature.",
                ));
            }
            Some(_) => {}
        }
        match self.since.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[recurrence_anchor] requires `since = \"<date-or-version>\"` \
                     (first detected instance anchor).",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.since_span.unwrap_or(self.args_span),
                    "#[recurrence_anchor] `since` cannot be empty.",
                ));
            }
            Some(_) => {}
        }
        match self.rationale.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[recurrence_anchor] requires `rationale = \"...\"` \
                     (≥20 characters; why this recurrence warrants action).",
                ));
            }
            Some(s) if s.len() < 20 => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    format!(
                        "#[recurrence_anchor] `rationale` must be at least 20 \
                         characters (got {}); clinical-diagnosis-grade rationale \
                         per ADR-024 clinical-medicine grounding.",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[crystallize(name, from_itches?, antigen?, summary)]`.
///
/// Cognitive-organizational primitive: "itch cluster crosses threshold
/// into named failure-class." The promotion event from below-threshold
/// noticing to formal recognition; parallel to `crystallize` in the camp
/// field-track substrate.
#[derive(Debug)]
pub struct CrystallizeArgs {
    pub name: Option<String>,
    pub name_span: Option<Span>,
    #[allow(dead_code)]
    pub from_itches: Vec<syn::Path>,
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub summary: Option<String>,
    pub summary_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for CrystallizeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut name: Option<String> = None;
        let mut name_span: Option<Span> = None;
        let mut from_itches: Vec<syn::Path> = Vec::new();
        let mut antigen: Option<syn::Path> = None;
        let mut summary: Option<String> = None;
        let mut summary_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name_span = Some(lit.span());
                    name = Some(lit.value());
                }
                "from_itches" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Path(p) = elem {
                            from_itches.push(p.path.clone());
                        } else {
                            return Err(syn::Error::new_spanned(
                                elem,
                                "expected a path expression in `from_itches` array",
                            ));
                        }
                    }
                }
                "antigen" => {
                    antigen = Some(input.parse()?);
                }
                "summary" => {
                    let lit: LitStr = input.parse()?;
                    summary_span = Some(lit.span());
                    summary = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[crystallize] field `{other}`; expected \
                             one of: name, from_itches, antigen, summary"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name,
            name_span,
            from_itches,
            antigen,
            summary,
            summary_span,
            args_span,
        })
    }
}

impl CrystallizeArgs {
    /// Per scientist HOW-spec: name + summary required; summary ≥10 chars.
    pub fn validate(&self) -> syn::Result<()> {
        match self.name.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[crystallize] requires `name = \"<slug>\"`.",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.name_span.unwrap_or(self.args_span),
                    "#[crystallize] `name` cannot be empty.",
                ));
            }
            Some(_) => {}
        }
        match self.summary.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[crystallize] requires `summary = \"...\"` (≥10 characters).",
                ));
            }
            Some(s) if s.len() < 10 => {
                return Err(syn::Error::new(
                    self.summary_span.unwrap_or(self.args_span),
                    format!(
                        "#[crystallize] `summary` must be at least 10 characters \
                         (got {}); per cognitive-organizational discipline a \
                         crystallization event needs enough text to be useful.",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[chronic(antigen, since, status?, managed_by?)]`.
///
/// Immunology-proper primitive: "low-level persistent signal NOT
/// cross-substrate" per ADR-024 §Disambiguation. Distinct from
/// `#[recurrence_anchor]` (cross-substrate, threshold reached): chronic
/// is a SUSTAINED single-substrate signal — analogous to a chronic
/// inflammatory state without acute recurrence.
#[derive(Debug)]
pub struct ChronicArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    pub since: Option<String>,
    pub since_span: Option<Span>,
    #[allow(dead_code)]
    pub status: Option<String>,
    #[allow(dead_code)]
    pub managed_by: Option<String>,
    pub args_span: Span,
}

impl Parse for ChronicArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut since: Option<String> = None;
        let mut since_span: Option<Span> = None;
        let mut status: Option<String> = None;
        let mut managed_by: Option<String> = None;

        // Optional leading positional antigen path
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
                "antigen" => {
                    antigen = Some(input.parse()?);
                }
                "since" => {
                    let lit: LitStr = input.parse()?;
                    since_span = Some(lit.span());
                    since = Some(lit.value());
                }
                "status" => {
                    let lit: LitStr = input.parse()?;
                    status = Some(lit.value());
                }
                "managed_by" => {
                    let lit: LitStr = input.parse()?;
                    managed_by = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[chronic] field `{other}`; expected one of: \
                             antigen, since, status, managed_by"
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
            since,
            since_span,
            status,
            managed_by,
            args_span,
        })
    }
}

impl ChronicArgs {
    /// Per scientist HOW-spec: antigen + since both REQUIRED.
    pub fn validate(&self) -> syn::Result<()> {
        if self.antigen.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[chronic] requires an `antigen` path (the failure-class \
                 being marked chronic).",
            ));
        }
        match self.since.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[chronic] requires `since = \"<date-or-version>\"` (when \
                     the chronic signal was first observed).",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.since_span.unwrap_or(self.args_span),
                    "#[chronic] `since` cannot be empty.",
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[saturate(antigen?, contributing_to?, description)]`.
///
/// Cognitive-organizational primitive: "accumulating saturation evidence
/// toward a recurrence threshold." A saturation event names a contribution
/// to a recognized recurrence pattern; without a `contributing_to` target
/// the audit emits `saturate-no-anchor`.
#[derive(Debug)]
pub struct SaturateArgs {
    #[allow(dead_code)]
    pub antigen: Option<syn::Path>,
    #[allow(dead_code)]
    pub contributing_to: Option<String>,
    pub description: Option<String>,
    pub description_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for SaturateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut antigen: Option<syn::Path> = None;
        let mut contributing_to: Option<String> = None;
        let mut description: Option<String> = None;
        let mut description_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "antigen" => {
                    antigen = Some(input.parse()?);
                }
                "contributing_to" => {
                    let lit: LitStr = input.parse()?;
                    contributing_to = Some(lit.value());
                }
                "description" => {
                    let lit: LitStr = input.parse()?;
                    description_span = Some(lit.span());
                    description = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[saturate] field `{other}`; expected one of: \
                             antigen, contributing_to, description"
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
            contributing_to,
            description,
            description_span,
            args_span,
        })
    }
}

impl SaturateArgs {
    /// Per scientist HOW-spec: description required (≥10 chars).
    pub fn validate(&self) -> syn::Result<()> {
        match self.description.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[saturate] requires `description = \"...\"` (≥10 characters; \
                     what evidence is saturating).",
                ));
            }
            Some(s) if s.len() < 10 => {
                return Err(syn::Error::new(
                    self.description_span.unwrap_or(self.args_span),
                    format!(
                        "#[saturate] `description` must be at least 10 characters \
                         (got {}).",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[strand(name, anchored_by?, description)]`.
///
/// Cognitive-organizational primitive: "thread of related noticing; may
/// spawn `#[itch]` or `#[recurrence_anchor]`." A strand groups noticings
/// across multiple substrates that share a structural rhyme but haven't
/// yet crystallized.
#[derive(Debug)]
pub struct StrandArgs {
    pub name: Option<String>,
    pub name_span: Option<Span>,
    #[allow(dead_code)]
    pub anchored_by: Vec<syn::Path>,
    pub description: Option<String>,
    pub description_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for StrandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut name: Option<String> = None;
        let mut name_span: Option<Span> = None;
        let mut anchored_by: Vec<syn::Path> = Vec::new();
        let mut description: Option<String> = None;
        let mut description_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => {
                    let lit: LitStr = input.parse()?;
                    name_span = Some(lit.span());
                    name = Some(lit.value());
                }
                "anchored_by" => {
                    let arr: syn::ExprArray = input.parse()?;
                    for elem in &arr.elems {
                        if let Expr::Path(p) = elem {
                            anchored_by.push(p.path.clone());
                        } else {
                            return Err(syn::Error::new_spanned(
                                elem,
                                "expected a path expression in `anchored_by` array",
                            ));
                        }
                    }
                }
                "description" => {
                    let lit: LitStr = input.parse()?;
                    description_span = Some(lit.span());
                    description = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[strand] field `{other}`; expected one of: \
                             name, anchored_by, description"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            name,
            name_span,
            anchored_by,
            description,
            description_span,
            args_span,
        })
    }
}

impl StrandArgs {
    /// Per scientist HOW-spec: name + description both REQUIRED (≥10 chars).
    pub fn validate(&self) -> syn::Result<()> {
        match self.name.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[strand] requires `name = \"<slug>\"`.",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.name_span.unwrap_or(self.args_span),
                    "#[strand] `name` cannot be empty.",
                ));
            }
            Some(_) => {}
        }
        match self.description.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[strand] requires `description = \"...\"` (≥10 characters; \
                     what threads of noticing this strand groups).",
                ));
            }
            Some(s) if s.len() < 10 => {
                return Err(syn::Error::new(
                    self.description_span.unwrap_or(self.args_span),
                    format!(
                        "#[strand] `description` must be at least 10 characters \
                         (got {}).",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

// ============================================================================
// MacroMucosalKind — local mirror of antigen::MucosalKind (ADR-027 + Amd 1)
//
// proc-macro crates cannot depend on the `antigen` library crate (circular
// dependency); local parse-time mirror — same pattern as MacroAntigenCategory
// (ADR-028) and MacroTriageDecision (ADR-026). Sealed 13-variant set; extending
// requires an ADR amendment per ADR-001 Amendment 1 C6.
// ============================================================================

/// Parse-time mucosal-kind variant (mirrors `antigen::mucosal::MucosalKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroMucosalKind {
    ApiRequest,
    ApiResponse,
    McpInvocation,
    ExternalLink,
    Iframe,
    DatabaseQuery,
    CrossService,
    SubprocessLaunch,
    DependencyImport,
    UserInput,
    FilesystemPath,
    EnvironmentVariable,
    ShellArgument,
}

impl MacroMucosalKind {
    /// Parse from path-style expression strings and common aliases. Strips
    /// an optional `MucosalKind::` prefix.
    fn from_path_str(s: &str) -> Option<Self> {
        let bare = s.strip_prefix("MucosalKind::").unwrap_or(s);
        match bare {
            "ApiRequest" | "api-request" | "api_request" => Some(Self::ApiRequest),
            "ApiResponse" | "api-response" | "api_response" => Some(Self::ApiResponse),
            "McpInvocation" | "mcp-invocation" | "mcp_invocation" => Some(Self::McpInvocation),
            "ExternalLink" | "external-link" | "external_link" => Some(Self::ExternalLink),
            "Iframe" | "iframe" => Some(Self::Iframe),
            "DatabaseQuery" | "database-query" | "database_query" => Some(Self::DatabaseQuery),
            "CrossService" | "cross-service" | "cross_service" => Some(Self::CrossService),
            "SubprocessLaunch" | "subprocess-launch" | "subprocess_launch" => {
                Some(Self::SubprocessLaunch)
            }
            "DependencyImport" | "dependency-import" | "dependency_import" => {
                Some(Self::DependencyImport)
            }
            "UserInput" | "user-input" | "user_input" => Some(Self::UserInput),
            "FilesystemPath" | "filesystem-path" | "filesystem_path" => Some(Self::FilesystemPath),
            "EnvironmentVariable" | "environment-variable" | "environment_variable" => {
                Some(Self::EnvironmentVariable)
            }
            "ShellArgument" | "shell-argument" | "shell_argument" => Some(Self::ShellArgument),
            _ => None,
        }
    }
}

/// Shared helper: parse a `kind = MucosalKind::X` value (path expr, not
/// string literal) into a `MacroMucosalKind`. Used by all three mucosal
/// parsers per ADR-027 Amendment 1 path-expression discipline.
fn parse_mucosal_kind_expr(expr: &Expr) -> syn::Result<MacroMucosalKind> {
    let s = match expr {
        Expr::Path(p) => p
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => {
            return Err(syn::Error::new_spanned(
                expr,
                "expected a `MucosalKind::X` path expression (e.g., \
                 `MucosalKind::UserInput`), not a string literal",
            ));
        }
    };
    MacroMucosalKind::from_path_str(&s).ok_or_else(|| {
        syn::Error::new_spanned(
            expr,
            format!(
                "unknown MucosalKind `{s}`; expected one of the 13 sealed-set \
                 variants (ApiRequest, ApiResponse, McpInvocation, ExternalLink, \
                 Iframe, DatabaseQuery, CrossService, SubprocessLaunch, \
                 DependencyImport, UserInput, FilesystemPath, EnvironmentVariable, \
                 ShellArgument). Note: PrBoundary + Import were removed in ADR-027 \
                 Amendment 1."
            ),
        )
    })
}

// ============================================================================
// Mucosal Boundary Family argument parsers (ADR-027 + Amendment 1)
//
// Three primitives: #[mucosal], #[mucosal_delegate], #[mucosal_tolerant].
// All declare antigen-category = SubstrateAlignment per ADR-028.
// ============================================================================

/// Arguments to `#[mucosal(kind, rationale)]`.
///
/// Declares a trust boundary is actively defended at this site. Per ADR-027
/// the biology grounds the tier-claim (mucosal surfaces are a distinct
/// immune tier) + the prevention-at-boundary discipline (secretory-IgA-style
/// exclusion).
#[derive(Debug)]
pub struct MucosalArgs {
    pub kind: Option<MacroMucosalKind>,
    #[allow(dead_code)]
    pub kind_span: Option<Span>,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for MucosalArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut kind: Option<MacroMucosalKind> = None;
        let mut kind_span: Option<Span> = None;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "kind" => {
                    let expr: Expr = input.parse()?;
                    kind_span = Some(input.span());
                    kind = Some(parse_mucosal_kind_expr(&expr)?);
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[mucosal] field `{other}`; expected one of: \
                             kind, rationale"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            kind,
            kind_span,
            rationale,
            rationale_span,
            args_span,
        })
    }
}

impl MucosalArgs {
    /// Per ADR-027: kind + rationale REQUIRED; rationale ≥20 chars.
    pub fn validate(&self) -> syn::Result<()> {
        if self.kind.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[mucosal] requires `kind = MucosalKind::X` (the boundary type \
                 being defended).",
            ));
        }
        match self.rationale.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[mucosal] requires `rationale = \"...\"` (≥20 characters; \
                     why this boundary is defended).",
                ));
            }
            Some(s) if s.len() < 20 => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    format!(
                        "#[mucosal] `rationale` must be at least 20 characters \
                         (got {}); per ADR-027 boundary-defense discipline.",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[mucosal_delegate(boundary, handled_by, rationale)]`.
///
/// Declares the boundary discipline is delegated to a named handler. Per
/// ADR-027 Amendment 1 Change 4, `handled_by` is a `syn::Path` (not a
/// string), so typos are caught at parse-time and resolution follows
/// standard Rust visibility rules at audit-time. Per Change 5, the handler
/// must carry a `#[mucosal(kind = X)]` where X matches `boundary`.
#[derive(Debug)]
pub struct MucosalDelegateArgs {
    pub boundary: Option<MacroMucosalKind>,
    #[allow(dead_code)]
    pub boundary_span: Option<Span>,
    #[allow(dead_code)]
    pub handled_by: Option<syn::Path>,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub args_span: Span,
}

impl Parse for MucosalDelegateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut boundary: Option<MacroMucosalKind> = None;
        let mut boundary_span: Option<Span> = None;
        let mut handled_by: Option<syn::Path> = None;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "boundary" => {
                    let expr: Expr = input.parse()?;
                    boundary_span = Some(input.span());
                    boundary = Some(parse_mucosal_kind_expr(&expr)?);
                }
                "handled_by" => {
                    // ADR-027 Amendment 1 Change 4: handled_by is a path
                    // expression, not a string literal.
                    handled_by = Some(input.parse()?);
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[mucosal_delegate] field `{other}`; expected \
                             one of: boundary, handled_by, rationale"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            boundary,
            boundary_span,
            handled_by,
            rationale,
            rationale_span,
            args_span,
        })
    }
}

impl MucosalDelegateArgs {
    /// Per ADR-027 Amendment 1: boundary + `handled_by` + rationale REQUIRED;
    /// rationale ≥20 chars. Kind-matching against the handler's `#[mucosal]`
    /// is an audit-time check (Change 5), not parse-time.
    pub fn validate(&self) -> syn::Result<()> {
        if self.boundary.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[mucosal_delegate] requires `boundary = MucosalKind::X` (the \
                 boundary kind being delegated).",
            ));
        }
        if self.handled_by.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[mucosal_delegate] requires `handled_by = <path>` (the handler \
                 function that carries the matching `#[mucosal(kind = X)]`). Per \
                 ADR-027 Amendment 1 this is a path expression, not a string.",
            ));
        }
        match self.rationale.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[mucosal_delegate] requires `rationale = \"...\"` (≥20 chars).",
                ));
            }
            Some(s) if s.len() < 20 => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    format!(
                        "#[mucosal_delegate] `rationale` must be at least 20 \
                         characters (got {}).",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        Ok(())
    }
}

/// Arguments to `#[mucosal_tolerant(kind, rationale, accepts, reviewed_by?,
/// until?)]`.
///
/// Declares a boundary is INTENTIONALLY permitted — active tolerance, not
/// absence of defense (ADR-027 Amendment 1 Change 6). The biology cognate is
/// Treg-mediated active tolerance (oral tolerance, fetal-maternal interface).
/// Parallel to ADR-016 `#[antigen_tolerance]` but at the boundary tier. The
/// rationale floor is ≥40 chars (higher than `#[mucosal]`'s ≥20) — tolerance
/// errors are silent/latent (no acute signal catches a bad tolerance decision),
/// so the up-front declaration carries a higher loudness floor to compensate
/// for the detection asymmetry.
#[derive(Debug)]
pub struct MucosalTolerantArgs {
    pub kind: Option<MacroMucosalKind>,
    #[allow(dead_code)]
    pub kind_span: Option<Span>,
    pub rationale: Option<String>,
    pub rationale_span: Option<Span>,
    pub accepts: Option<String>,
    pub accepts_span: Option<Span>,
    #[allow(dead_code)]
    pub reviewed_by: Option<String>,
    #[allow(dead_code)]
    pub until: Option<String>,
    pub args_span: Span,
}

impl Parse for MucosalTolerantArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args_span = input.span();
        let mut kind: Option<MacroMucosalKind> = None;
        let mut kind_span: Option<Span> = None;
        let mut rationale: Option<String> = None;
        let mut rationale_span: Option<Span> = None;
        let mut accepts: Option<String> = None;
        let mut accepts_span: Option<Span> = None;
        let mut reviewed_by: Option<String> = None;
        let mut until: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "kind" => {
                    let expr: Expr = input.parse()?;
                    kind_span = Some(input.span());
                    kind = Some(parse_mucosal_kind_expr(&expr)?);
                }
                "rationale" => {
                    let lit: LitStr = input.parse()?;
                    rationale_span = Some(lit.span());
                    rationale = Some(lit.value());
                }
                "accepts" => {
                    let lit: LitStr = input.parse()?;
                    accepts_span = Some(lit.span());
                    accepts = Some(lit.value());
                }
                "reviewed_by" => {
                    let lit: LitStr = input.parse()?;
                    reviewed_by = Some(lit.value());
                }
                "until" => {
                    let lit: LitStr = input.parse()?;
                    until = Some(lit.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown #[mucosal_tolerant] field `{other}`; expected \
                             one of: kind, rationale, accepts, reviewed_by, until. \
                             (For failure-class-tier tolerance see \
                             `#[antigen_tolerance]`; this is the BOUNDARY-tier \
                             tolerance primitive.)"
                        ),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            kind,
            kind_span,
            rationale,
            rationale_span,
            accepts,
            accepts_span,
            reviewed_by,
            until,
            args_span,
        })
    }
}

impl MucosalTolerantArgs {
    /// Per ADR-027 Amendment 1 Change 6: kind + rationale (≥40) + accepts
    /// (non-empty) REQUIRED; `reviewed_by` + until optional v0.2.
    pub fn validate(&self) -> syn::Result<()> {
        if self.kind.is_none() {
            return Err(syn::Error::new(
                self.args_span,
                "#[mucosal_tolerant] requires `kind = MucosalKind::X`. (For \
                 failure-class-tier tolerance see `#[antigen_tolerance]`; this \
                 is the BOUNDARY-tier tolerance primitive.)",
            ));
        }
        match self.rationale.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[mucosal_tolerant] requires `rationale = \"...\"` (≥40 \
                     characters — tolerance errors are silent/latent, so the \
                     up-front declaration carries a higher loudness floor than \
                     #[mucosal]'s ≥20 to compensate for the detection asymmetry).",
                ));
            }
            Some(s) if s.len() < 40 => {
                return Err(syn::Error::new(
                    self.rationale_span.unwrap_or(self.args_span),
                    format!(
                        "#[mucosal_tolerant] `rationale` must be at least 40 \
                         characters (got {}); per ADR-027 Amendment 1 the floor \
                         is higher than #[mucosal]'s ≥20 because tolerance errors \
                         are silent/latent — no acute signal catches a bad \
                         tolerance decision.",
                        s.len()
                    ),
                ));
            }
            Some(_) => {}
        }
        match self.accepts.as_deref() {
            None => {
                return Err(syn::Error::new(
                    self.args_span,
                    "#[mucosal_tolerant] requires `accepts = \"...\"` (non-empty; \
                     description of what the boundary accepts as legitimate input).",
                ));
            }
            Some(s) if s.trim().is_empty() => {
                return Err(syn::Error::new(
                    self.accepts_span.unwrap_or(self.args_span),
                    "#[mucosal_tolerant] `accepts` cannot be empty. Audit emits \
                     mucosal-tolerant-accepts-empty.",
                ));
            }
            Some(_) => {}
        }
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

    // Witness for #[immune(UnvalidatedSealedEnumAcceptance, ...)] on
    // DiagnosticArgs::validate — the sealed WitnessClass set is enforced at
    // parse/validate time; a non-variant ident must be rejected, not silently
    // counted as a valid modality.

    #[test]
    fn diagnostic_validate_rejects_unknown_witness_class() {
        // `WitnessClass::Nonsense` is not one of the 6 ratified variants.
        // Before the fix, this was captured as the string "Nonsense" and
        // counted toward the distinctness check (UnvalidatedSealedEnumAcceptance).
        let tokens: TokenStream = r"modalities = [WitnessClass::Nonsense], min_independent = 1"
            .parse()
            .unwrap();
        let args = syn::parse2::<DiagnosticArgs>(tokens).unwrap();
        let err = args
            .validate()
            .expect_err("an unknown WitnessClass variant must be rejected")
            .to_string();
        assert!(
            err.contains("Nonsense") && err.contains("WitnessClass"),
            "error must name the offending ident + the valid set: {err}"
        );
    }

    #[test]
    fn diagnostic_validate_rejects_non_witness_class_path() {
        // Any `Foo::Bar` path is captured as its last ident; it must not pass
        // as a modality just because it parses as a path.
        let tokens: TokenStream = r"modalities = [Foo::Bar], min_independent = 1"
            .parse()
            .unwrap();
        let args = syn::parse2::<DiagnosticArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "a non-WitnessClass path must be rejected, not counted as a modality"
        );
    }

    #[test]
    fn diagnostic_validate_accepts_all_known_witness_classes() {
        // The 6 ratified variants must all pass — the allowlist must not be
        // narrower than the real WitnessClass enum (a too-strict allowlist
        // would reject valid modalities).
        let tokens: TokenStream = "modalities = [WitnessClass::StaticAnalysis, \
             WitnessClass::PropertyTest, WitnessClass::FormalVerification, \
             WitnessClass::ManualReview, WitnessClass::RuntimeFuzz, \
             WitnessClass::SubstrateWitness], min_independent = 6"
            .parse()
            .unwrap();
        let args = syn::parse2::<DiagnosticArgs>(tokens).unwrap();
        args.validate()
            .expect("all 6 ratified WitnessClass variants must validate");
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
    fn macro_antigen_category_as_str() {
        assert_eq!(
            MacroAntigenCategory::SubstrateAlignment.as_str(),
            "substrate-alignment"
        );
        assert_eq!(
            MacroAntigenCategory::FunctionalCorrectness.as_str(),
            "functional-correctness"
        );
    }

    #[test]
    fn macro_antigen_category_from_path_str_accepts_path_forms() {
        // from_path_str handles macro-input path tokens (Pascal or qualified path).
        // Kebab is NOT valid Rust path syntax — no macro author writes it.
        for s in ["SubstrateAlignment", "AntigenCategory::SubstrateAlignment"] {
            assert_eq!(
                MacroAntigenCategory::from_path_str(s),
                Some(MacroAntigenCategory::SubstrateAlignment),
                "expected SubstrateAlignment from {s:?}"
            );
        }
        for s in [
            "FunctionalCorrectness",
            "AntigenCategory::FunctionalCorrectness",
        ] {
            assert_eq!(
                MacroAntigenCategory::from_path_str(s),
                Some(MacroAntigenCategory::FunctionalCorrectness),
                "expected FunctionalCorrectness from {s:?}"
            );
        }
        // Kebab and snake are rejected.
        for s in [
            "substrate-alignment",
            "substrate_alignment",
            "functional-correctness",
            "functional_correctness",
        ] {
            assert_eq!(
                MacroAntigenCategory::from_path_str(s),
                None,
                "expected None (not a valid macro-input path token) for {s:?}"
            );
        }
    }

    // -------------------------------------------------------------------------
    // Adversarial tests (added by adversarial role)
    // -------------------------------------------------------------------------

    #[test]
    fn antigen_parser_duplicate_category_in_array_is_rejected() {
        // [SubstrateAlignment, SubstrateAlignment] must be rejected by validate():
        // duplicate entries look like hybrid (len == 2) but contain only one
        // distinct variant. Fixed by dedup-check in AntigenArgs::validate() per
        // aristotle Phase 1-8 ratification (ADR-028).
        let tokens: TokenStream = r#"name = "x", fingerprint = "item = fn", category = [AntigenCategory::SubstrateAlignment, AntigenCategory::SubstrateAlignment]"#
            .parse()
            .unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        let err = args.validate().unwrap_err().to_string();
        assert!(
            err.contains("duplicate") && err.contains("category"),
            "validate() must reject duplicate category entries; got: {err:?}"
        );
    }

    #[test]
    fn antigen_parser_three_element_category_array_with_duplicate_is_rejected() {
        // [SA, FC, SA] must be rejected: SA appears twice.
        // Even though FC is present (making it look hybrid), the duplicate SA
        // is structurally incorrect.
        let tokens: TokenStream = r#"name = "x", fingerprint = "item = fn", category = [AntigenCategory::SubstrateAlignment, AntigenCategory::FunctionalCorrectness, AntigenCategory::SubstrateAlignment]"#
            .parse()
            .unwrap();
        let args = syn::parse2::<AntigenArgs>(tokens).unwrap();
        let err = args.validate().unwrap_err().to_string();
        assert!(
            err.contains("duplicate") && err.contains("category"),
            "validate() must reject duplicate category entries; got: {err:?}"
        );
    }

    #[test]
    fn antigen_parser_rejects_string_literal_as_category() {
        // String literals are NOT accepted as category values; path expressions
        // are required for compile-time discoverability (per expect_antigen_category).
        let tokens: TokenStream =
            r#"name = "x", fingerprint = "item = fn", category = "substrate-alignment""#
                .parse()
                .unwrap();
        let result = syn::parse2::<AntigenArgs>(tokens);
        assert!(
            result.is_err(),
            "string literal category should be rejected; got Ok"
        );
    }

    #[test]
    fn antigen_parser_rejects_integer_as_category() {
        // Non-path non-array expressions (integers, etc.) should be rejected.
        let tokens: TokenStream = r#"name = "x", fingerprint = "item = fn", category = 42"#
            .parse()
            .unwrap();
        let result = syn::parse2::<AntigenArgs>(tokens);
        assert!(
            result.is_err(),
            "integer category should be rejected; got Ok"
        );
    }

    // -------------------------------------------------------------------------
    // #[triage_commit] tests (ADR-026 §Rollback-as-triage primitive)
    // -------------------------------------------------------------------------

    fn valid_triage_commit_tokens() -> TokenStream {
        r#"triage_decision = TriageDecision::Red,
           rollback_target = "abc1234",
           triaged_by = "navigator",
           rationale = "vital metric regression confirmed via #84; rolling back",
           rollback_due_within_minutes = 30"#
            .parse()
            .unwrap()
    }

    #[test]
    fn triage_commit_parser_accepts_canonical_form() {
        let args = syn::parse2::<TriageCommitArgs>(valid_triage_commit_tokens()).unwrap();
        assert_eq!(args.triage_decision, Some(MacroTriageDecision::Red));
        assert_eq!(args.rollback_target.as_deref(), Some("abc1234"));
        assert_eq!(args.triaged_by.as_deref(), Some("navigator"));
        assert_eq!(args.rollback_due_within_minutes, Some(30));
        args.validate().unwrap();
    }

    #[test]
    fn triage_commit_parser_accepts_bare_variant_ident() {
        let tokens: TokenStream = r#"triage_decision = Black,
            rollback_target = "deadbeef",
            triaged_by = "oncall",
            rationale = "system-down: payment processor pod crashlooping",
            rollback_due_within_minutes = 5"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert_eq!(args.triage_decision, Some(MacroTriageDecision::Black));
        args.validate().unwrap();
    }

    #[test]
    fn triage_commit_parser_accepts_all_five_variants() {
        for (variant_text, expected) in [
            ("TriageDecision::Black", MacroTriageDecision::Black),
            ("TriageDecision::Red", MacroTriageDecision::Red),
            ("TriageDecision::Yellow", MacroTriageDecision::Yellow),
            ("TriageDecision::Green", MacroTriageDecision::Green),
            ("TriageDecision::White", MacroTriageDecision::White),
        ] {
            let src = format!(
                r#"triage_decision = {variant_text},
                   rollback_target = "abc1234",
                   triaged_by = "navigator",
                   rationale = "twenty-character-rationale-text-here",
                   rollback_due_within_minutes = 30"#
            );
            let tokens: TokenStream = src.parse().unwrap();
            let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
            assert_eq!(args.triage_decision, Some(expected));
            args.validate().unwrap();
        }
    }

    #[test]
    fn triage_commit_parser_rejects_unknown_variant() {
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Purple,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let result = syn::parse2::<TriageCommitArgs>(tokens);
        assert!(result.is_err(), "unknown variant should be rejected");
    }

    #[test]
    fn triage_commit_parser_rejects_string_literal_triage_decision() {
        let tokens: TokenStream = r#"triage_decision = "red",
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let result = syn::parse2::<TriageCommitArgs>(tokens);
        assert!(
            result.is_err(),
            "string literal triage_decision should be rejected"
        );
    }

    #[test]
    fn triage_commit_validate_rejects_missing_triage_decision() {
        let tokens: TokenStream = r#"rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn triage_commit_validate_rejects_empty_rollback_target() {
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn triage_commit_validate_rejects_empty_triaged_by() {
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn triage_commit_validate_rejects_short_rationale() {
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "too short",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn triage_commit_validate_rejects_zero_minutes() {
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 0"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn triage_commit_parser_rejects_unknown_field() {
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text-here",
            rollback_due_within_minutes = 30,
            bogus = "value""#
            .parse()
            .unwrap();
        let result = syn::parse2::<TriageCommitArgs>(tokens);
        assert!(result.is_err(), "unknown field should be rejected");
    }

    #[test]
    fn macro_triage_decision_as_str_roundtrip() {
        for (variant, expected) in [
            (MacroTriageDecision::Black, "black"),
            (MacroTriageDecision::Red, "red"),
            (MacroTriageDecision::Yellow, "yellow"),
            (MacroTriageDecision::Green, "green"),
            (MacroTriageDecision::White, "white"),
        ] {
            assert_eq!(variant.as_str(), expected);
        }
    }

    // -------------------------------------------------------------------------
    // Adversarial tests for #[triage_commit] (added by adversarial role)
    // -------------------------------------------------------------------------

    #[test]
    fn triage_commit_validate_rationale_exactly_20_chars_passes() {
        // Boundary case: rationale of EXACTLY 20 chars must pass (>= 20, not > 20).
        // "20characterrational!" is exactly 20 chars.
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "20characterrationale",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert_eq!(
            args.rationale.as_deref().map(str::len),
            Some(20),
            "fixture must be exactly 20 chars"
        );
        assert!(
            args.validate().is_ok(),
            "exactly 20 chars should pass rationale validation"
        );
    }

    #[test]
    fn triage_commit_validate_rationale_19_chars_fails() {
        // Just below the 20-char minimum: must fail.
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "19characterrational",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert_eq!(
            args.rationale.as_deref().map(str::len),
            Some(19),
            "fixture must be exactly 19 chars"
        );
        assert!(
            args.validate().is_err(),
            "19-char rationale should fail validation"
        );
    }

    #[test]
    fn triage_commit_validate_absurdly_large_deadline_is_currently_accepted() {
        // ADVERSARIAL FINDING: rollback_due_within_minutes = u32::MAX is accepted.
        // Semantically absurd (5.3 million hours = 600 years). The validate()
        // method only checks for 0; there is no upper cap. This test PINS the
        // current behavior. A future check could warn/error on deadline > 10080
        // (one week in minutes) per operational discipline.
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Yellow,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text",
            rollback_due_within_minutes = 4294967295"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert_eq!(args.rollback_due_within_minutes, Some(4_294_967_295));
        assert!(
            args.validate().is_ok(),
            "u32::MAX deadline currently accepted (no upper cap); this pins that behavior"
        );
    }

    #[test]
    fn triage_commit_validate_whitespace_only_rollback_target_is_rejected() {
        // ATK-VCS-5 fix: rollback_target = "   " (whitespace-only) is now
        // caught by trim().is_empty() check. A whitespace-only string is not
        // a valid git ref.
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "   ",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert_eq!(args.rollback_target.as_deref(), Some("   "));
        let err = args.validate().unwrap_err();
        assert!(
            err.to_string().contains("rollback_target"),
            "error should mention rollback_target field"
        );
    }

    #[test]
    fn triage_commit_validate_whitespace_only_rationale_of_20_chars_should_fail() {
        // ATK-TRIAGE-WHITESPACE-RATIONALE: the rationale length check uses
        // s.len() < 20 — a 20-space string has len == 20 and passes. But 20
        // spaces contains zero meaningful content. The rationale check should
        // also verify that the string is not all-whitespace (trim().is_empty()).
        //
        // This is the same gap as the pre-fix parse_doc_contains: is_empty() vs
        // trim().is_empty(). A blank rationale bypasses the chart-documentation
        // discipline (ADR-026 §Rollback-as-triage: rationale before action, not after).
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "                    ",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert_eq!(
            args.rationale.as_deref().map(str::len),
            Some(20),
            "fixture must be exactly 20 spaces"
        );
        assert!(
            args.validate().is_err(),
            "ATK-TRIAGE-WHITESPACE-RATIONALE: rationale of 20 spaces must fail validation. \
             The 20-char length check (s.len() < 20) passes for '                    ' because \
             len==20. But trim().is_empty() reveals it contains no meaningful content. \
             A blank rationale bypasses chart-documentation discipline. \
             Fix: after the len < 20 check, add trim().is_empty() check — same pattern as \
             rollback_target's whitespace guard."
        );
    }

    #[test]
    fn triage_commit_validate_non_sha_rollback_target_is_currently_accepted() {
        // ADVERSARIAL FINDING: rollback_target accepts arbitrary non-SHA strings.
        // ADR-026 says "commit sha" but no format validation is enforced.
        // "not-a-sha-at-all" is accepted. This is intentional forward-compat
        // (refs like branch names may also be valid rollback targets), but
        // the gap is that completely arbitrary text passes.
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Red,
            rollback_target = "not-a-sha-at-all-just-text",
            triaged_by = "navigator",
            rationale = "twenty-character-rationale-text",
            rollback_due_within_minutes = 30"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(
            args.validate().is_ok(),
            "non-SHA rollback_target currently accepted; pins that no format validation exists"
        );
    }

    #[test]
    fn triage_commit_validate_green_triage_with_tight_deadline_is_accepted() {
        // Green means "no regression detected" — no rollback planned.
        // A tight rollback_due_within_minutes = 1 on a Green triage is
        // semantically odd (no rollback needed but one is mandated within 1 min).
        // The validator does NOT check for this semantic inconsistency.
        let tokens: TokenStream = r#"triage_decision = TriageDecision::Green,
            rollback_target = "abc1234",
            triaged_by = "navigator",
            rationale = "no regression detected in twenty chars",
            rollback_due_within_minutes = 1"#
            .parse()
            .unwrap();
        let args = syn::parse2::<TriageCommitArgs>(tokens).unwrap();
        assert!(
            args.validate().is_ok(),
            "Green triage with tight deadline currently accepted; no semantic inconsistency check"
        );
    }

    // -------------------------------------------------------------------------
    // #[orient] tests (ADR-023 + fixup-orient-dual-signature STEP 2-3 fields)
    // -------------------------------------------------------------------------

    // Helper: an `until` date safely inside the 180d horizon (today + ~90d),
    // computed at test time so the suite never goes stale as the clock advances.
    fn orient_until_within_horizon() -> String {
        (today_utc() + chrono::Duration::days(90))
            .format("%Y-%m-%d")
            .to_string()
    }

    #[test]
    fn orient_parser_bare_form_is_a_validate_error() {
        // Option-A hard break: bare #[orient] is invalid — learning_path + until
        // are REQUIRED (an orient without them is silent deferred non-immunity).
        let tokens: TokenStream = "".parse().unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "bare #[orient] must fail validation (learning_path + until required)"
        );
    }

    #[test]
    fn orient_parser_accepts_canonical_adr023_form() {
        // ADR-023 §Decision spec: #[orient(antigen, learning_path, until)],
        // both fields REQUIRED. learning_path >= 20 chars; until within horizon.
        let until = orient_until_within_horizon();
        let tokens: TokenStream = format!(
            r#"PanickingInDrop,
            learning_path = "Audit Drop impls before alpha tag",
            until = "{until}""#
        )
        .parse()
        .unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(args.antigen.is_some());
        assert_eq!(
            args.learning_path.as_deref(),
            Some("Audit Drop impls before alpha tag")
        );
        assert_eq!(args.until.as_deref(), Some(until.as_str()));
        args.validate().unwrap();
    }

    #[test]
    fn orient_parser_requires_learning_path() {
        let until = orient_until_within_horizon();
        let tokens: TokenStream = format!(r#"until = "{until}""#).parse().unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "until without learning_path must fail validation"
        );
    }

    #[test]
    fn orient_parser_requires_until() {
        let tokens: TokenStream = r#"learning_path = "Audit Drop impls before the alpha tag""#
            .parse()
            .unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "learning_path without until must fail validation"
        );
    }

    #[test]
    fn orient_parser_learning_path_min_20_chars() {
        let until = orient_until_within_horizon();
        let tokens: TokenStream = format!(r#"learning_path = "too short", until = "{until}""#)
            .parse()
            .unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "learning_path under 20 chars must fail (rationale-class minimum)"
        );
    }

    #[test]
    fn orient_parser_rejects_until_beyond_horizon() {
        // until far beyond now + 180d is a parse-time (validate) error.
        let far = (today_utc() + chrono::Duration::days(400))
            .format("%Y-%m-%d")
            .to_string();
        let tokens: TokenStream =
            format!(r#"learning_path = "Audit Drop impls before the alpha tag", until = "{far}""#)
                .parse()
                .unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "until beyond the 180d orientation horizon must fail validation"
        );
    }

    #[test]
    fn orient_parser_rejects_until_in_the_past() {
        // A past `until` date is an already-expired orientation window — it
        // silently represents "we never dealt with this failure-class AND the
        // accountability horizon already passed."  validate() must reject it.
        //
        // ATK: horizon_days = (past_date - today) < 0 — the existing check is
        // `if horizon_days > 180` which is false for negative values, so a past
        // date passes validation silently.  This test FAILS until the check
        // `if horizon_days < 0` is added.
        let yesterday = (today_utc() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let tokens: TokenStream = format!(
            r#"learning_path = "Audit Drop impls before the alpha tag", until = "{yesterday}""#
        )
        .parse()
        .unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-ORIENT-PAST-DATE: until in the past must fail validation — \
             an already-expired orientation period represents hidden unresolved \
             deferred non-immunity with no accountability; got Ok(()) for until={yesterday}"
        );
    }

    #[test]
    fn orient_parser_drift_fields_are_parse_errors() {
        // The pre-restoration drift-form fields (see/adr/attestation_optional)
        // are removed entirely — a hard parse error, not a deprecation warning.
        for drift in [
            r#"see = ["ADR-023"]"#,
            r#"adr = "ADR-023""#,
            "attestation_optional = true",
        ] {
            let tokens: TokenStream = drift.parse().unwrap();
            assert!(
                syn::parse2::<OrientArgs>(tokens).is_err(),
                "drift-form field must be a parse error: {drift}"
            );
        }
    }

    #[test]
    fn orient_parser_rejects_unknown_field() {
        let tokens: TokenStream = r#"bogus = "value""#.parse().unwrap();
        let result = syn::parse2::<OrientArgs>(tokens);
        assert!(result.is_err(), "unknown field should still be rejected");
    }

    // -------------------------------------------------------------------------
    // Recurrent-Emergence Family tests (ADR-024 + scientist HOW-spec cf2a2317)
    // -------------------------------------------------------------------------

    #[test]
    fn itch_parser_accepts_minimal() {
        let tokens: TokenStream =
            r#"name = "drop-panic-rhyme", description = "noticed Drop panics rhyming with unwrap-in-cleanup""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ItchArgs>(tokens).unwrap();
        assert_eq!(args.name.as_deref(), Some("drop-panic-rhyme"));
        args.validate().unwrap();
    }

    #[test]
    fn itch_validate_rejects_missing_name() {
        let tokens: TokenStream = r#"description = "noticed something worth ten chars""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ItchArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn itch_validate_rejects_short_description() {
        let tokens: TokenStream = r#"name = "x", description = "short""#.parse().unwrap();
        let args = syn::parse2::<ItchArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn itch_parser_rejects_unknown_field() {
        let tokens: TokenStream = r#"name = "x", description = "long enough text", bogus = 1"#
            .parse()
            .unwrap();
        assert!(syn::parse2::<ItchArgs>(tokens).is_err());
    }

    #[test]
    fn recurrence_anchor_parser_accepts_canonical() {
        let tokens: TokenStream = r#"MsrvCreep, instances = 3, since = "v0.1.0", rationale = "MSRV crept three times across major bumps""#
            .parse()
            .unwrap();
        let args = syn::parse2::<RecurrenceAnchorArgs>(tokens).unwrap();
        assert!(args.antigen.is_some());
        assert_eq!(args.instances, Some(3));
        args.validate().unwrap();
    }

    #[test]
    fn recurrence_anchor_validate_rejects_zero_instances() {
        let tokens: TokenStream =
            r#"X, instances = 0, since = "v1", rationale = "twenty character rationale here""#
                .parse()
                .unwrap();
        let args = syn::parse2::<RecurrenceAnchorArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn recurrence_anchor_validate_rejects_missing_antigen() {
        let tokens: TokenStream =
            r#"instances = 2, since = "v1", rationale = "twenty character rationale here""#
                .parse()
                .unwrap();
        let args = syn::parse2::<RecurrenceAnchorArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn recurrence_anchor_validate_rejects_short_rationale() {
        let tokens: TokenStream = r#"X, instances = 2, since = "v1", rationale = "too short""#
            .parse()
            .unwrap();
        let args = syn::parse2::<RecurrenceAnchorArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn crystallize_parser_accepts_with_from_itches() {
        let tokens: TokenStream = r#"name = "drop-panic", from_itches = [DropPanicItch, CleanupUnwrapItch], summary = "crystallized from two itches""#
            .parse()
            .unwrap();
        let args = syn::parse2::<CrystallizeArgs>(tokens).unwrap();
        assert_eq!(args.from_itches.len(), 2);
        args.validate().unwrap();
    }

    #[test]
    fn crystallize_validate_rejects_missing_summary() {
        let tokens: TokenStream = r#"name = "x""#.parse().unwrap();
        let args = syn::parse2::<CrystallizeArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn chronic_parser_accepts_canonical() {
        let tokens: TokenStream = r#"FlakeyCiStep, since = "v0.2.0", managed_by = "ci-team""#
            .parse()
            .unwrap();
        let args = syn::parse2::<ChronicArgs>(tokens).unwrap();
        assert!(args.antigen.is_some());
        args.validate().unwrap();
    }

    #[test]
    fn chronic_validate_rejects_missing_since() {
        let tokens: TokenStream = "X".parse().unwrap();
        let args = syn::parse2::<ChronicArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn chronic_validate_rejects_missing_antigen() {
        let tokens: TokenStream = r#"since = "v1""#.parse().unwrap();
        let args = syn::parse2::<ChronicArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn saturate_parser_accepts_minimal() {
        let tokens: TokenStream =
            r#"description = "evidence accumulating toward MSRV-creep anchor""#
                .parse()
                .unwrap();
        let args = syn::parse2::<SaturateArgs>(tokens).unwrap();
        args.validate().unwrap();
    }

    #[test]
    fn saturate_validate_rejects_short_description() {
        let tokens: TokenStream = r#"description = "short""#.parse().unwrap();
        let args = syn::parse2::<SaturateArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn strand_parser_accepts_with_anchored_by() {
        let tokens: TokenStream = r#"name = "vcs-loss-thread", anchored_by = [ForcePushItch, SquashMergeItch], description = "thread of history-loss noticings""#
            .parse()
            .unwrap();
        let args = syn::parse2::<StrandArgs>(tokens).unwrap();
        assert_eq!(args.anchored_by.len(), 2);
        args.validate().unwrap();
    }

    #[test]
    fn strand_validate_rejects_missing_name() {
        let tokens: TokenStream = r#"description = "thread of related noticings here""#
            .parse()
            .unwrap();
        let args = syn::parse2::<StrandArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    // -------------------------------------------------------------------------
    // Mucosal Boundary Family tests (ADR-027 + Amendment 1)
    // -------------------------------------------------------------------------

    #[test]
    fn mucosal_parser_accepts_canonical() {
        let tokens: TokenStream =
            r#"kind = MucosalKind::UserInput, rationale = "public comment form; XSS sanitized at render""#
                .parse()
                .unwrap();
        let args = syn::parse2::<MucosalArgs>(tokens).unwrap();
        assert_eq!(args.kind, Some(MacroMucosalKind::UserInput));
        args.validate().unwrap();
    }

    #[test]
    fn mucosal_parser_accepts_bare_variant() {
        let tokens: TokenStream =
            r#"kind = DatabaseQuery, rationale = "parameterized queries only at this layer""#
                .parse()
                .unwrap();
        let args = syn::parse2::<MucosalArgs>(tokens).unwrap();
        assert_eq!(args.kind, Some(MacroMucosalKind::DatabaseQuery));
        args.validate().unwrap();
    }

    #[test]
    fn mucosal_validate_rejects_missing_kind() {
        let tokens: TokenStream = r#"rationale = "twenty character rationale here padding""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn mucosal_validate_rejects_short_rationale() {
        let tokens: TokenStream = r#"kind = MucosalKind::UserInput, rationale = "short""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn mucosal_parser_rejects_removed_pr_boundary_variant() {
        // PrBoundary was removed in ADR-027 Amendment 1.
        let tokens: TokenStream =
            r#"kind = MucosalKind::PrBoundary, rationale = "twenty character rationale text""#
                .parse()
                .unwrap();
        assert!(syn::parse2::<MucosalArgs>(tokens).is_err());
    }

    #[test]
    fn mucosal_parser_rejects_string_literal_kind() {
        let tokens: TokenStream =
            r#"kind = "user-input", rationale = "twenty character rationale text""#
                .parse()
                .unwrap();
        assert!(syn::parse2::<MucosalArgs>(tokens).is_err());
    }

    #[test]
    fn mucosal_delegate_parser_accepts_canonical() {
        let tokens: TokenStream = r#"boundary = MucosalKind::UserInput, handled_by = crate::sanitize::user_input, rationale = "delegated to central sanitizer module""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalDelegateArgs>(tokens).unwrap();
        assert_eq!(args.boundary, Some(MacroMucosalKind::UserInput));
        assert!(args.handled_by.is_some());
        args.validate().unwrap();
    }

    #[test]
    fn mucosal_delegate_validate_rejects_missing_handled_by() {
        let tokens: TokenStream =
            r#"boundary = MucosalKind::UserInput, rationale = "twenty character rationale text""#
                .parse()
                .unwrap();
        let args = syn::parse2::<MucosalDelegateArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn mucosal_delegate_rejects_string_handled_by() {
        // handled_by is a path expression, not a string (Amendment 1 Change 4).
        // A string literal where a syn::Path is expected fails to parse.
        let tokens: TokenStream = r#"boundary = MucosalKind::UserInput, handled_by = "sanitize_fn", rationale = "twenty character rationale text""#
            .parse()
            .unwrap();
        assert!(
            syn::parse2::<MucosalDelegateArgs>(tokens).is_err(),
            "string-literal handled_by must be rejected (path expression required)"
        );
    }

    #[test]
    fn mucosal_tolerant_parser_accepts_canonical() {
        let tokens: TokenStream = r#"kind = MucosalKind::UserInput, rationale = "public firehose intake endpoint accepts anonymous submissions by design", accepts = "anonymous JSON payloads up to 64KB""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalTolerantArgs>(tokens).unwrap();
        assert_eq!(args.kind, Some(MacroMucosalKind::UserInput));
        args.validate().unwrap();
    }

    #[test]
    fn mucosal_tolerant_validate_rejects_rationale_under_40() {
        // Tolerance floor is ≥40 (vs #[mucosal]'s ≥20).
        let tokens: TokenStream = r#"kind = MucosalKind::UserInput, rationale = "twenty-five char rationale", accepts = "anything""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalTolerantArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "25-char rationale must fail the ≥40 tolerance floor"
        );
    }

    #[test]
    fn mucosal_tolerant_validate_rejects_empty_accepts() {
        let tokens: TokenStream = r#"kind = MucosalKind::UserInput, rationale = "this rationale is definitely longer than forty characters", accepts = "   ""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalTolerantArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn mucosal_tolerant_validate_rejects_missing_accepts() {
        let tokens: TokenStream = r#"kind = MucosalKind::UserInput, rationale = "this rationale is definitely longer than forty characters""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalTolerantArgs>(tokens).unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn mucosal_tolerant_accepts_optional_reviewed_by_and_until() {
        let tokens: TokenStream = r#"kind = MucosalKind::ApiRequest, rationale = "internal admin endpoint behind VPN; trusted-network assumption documented", accepts = "admin-panel form posts", reviewed_by = "security-team", until = "2026-12-31""#
            .parse()
            .unwrap();
        let args = syn::parse2::<MucosalTolerantArgs>(tokens).unwrap();
        assert_eq!(args.reviewed_by.as_deref(), Some("security-team"));
        assert_eq!(args.until.as_deref(), Some("2026-12-31"));
        args.validate().unwrap();
    }

    // -------------------------------------------------------------------------
    // Adversarial tests for #[immunosuppress] and #[anergy] — past until date
    // -------------------------------------------------------------------------

    #[test]
    fn immunosuppress_validate_rejects_until_in_the_past() {
        // ATK-IMMUNOSUPPRESS-PAST-DATE: #[immunosuppress] with `until` in the
        // past should fail validation — the suppression window has expired.
        //
        // FIXED: validate() now rejects past dates (duration_days < 0 check added).
        let yesterday = (today_utc() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let tokens: TokenStream = format!(
            r#"PanickingInDrop,
            rationale = "Suppressing while we wait for proptest infrastructure build",
            until = "{yesterday}""#
        )
        .parse()
        .unwrap();
        let args = syn::parse2::<ImmunosuppressArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-IMMUNOSUPPRESS-PAST-DATE: #[immunosuppress] with until in the past must fail \
             validation — expired suppression is silent check-disabling with no accountability. \
             Got Ok(()) for until={yesterday}. \
             Fix: add duration_days < 0 check in ImmunosuppressArgs::validate() at ~line 856."
        );
    }

    #[test]
    fn poxparty_validate_rejects_until_in_the_past() {
        // ATK-POXPARTY-PAST-DATE: #[poxparty] with `until` in the past should fail
        // validation — an expired pox-party window means a controlled-exposure exercise
        // is still referenced in code but its deadline has passed. Silent expired
        // poxparty is the same class as expired immunosuppress and anergy.
        //
        // FIXED: validate() now rejects past dates (parse_iso_date() + horizon_days < 0 check added).
        let yesterday = (today_utc() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let tokens: TokenStream = format!(
            r#"PanickingInDrop,
            exercise_type = "Trigger deliberate Drop panic and measure detection lag",
            until = "{yesterday}""#
        )
        .parse()
        .unwrap();
        let args = syn::parse2::<PoxpartyArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-POXPARTY-PAST-DATE: #[poxparty] with until in the past must fail validation — \
             an expired pox-party exercise represents stale controlled-exposure with no accountability. \
             Got Ok(()) for until={yesterday}. \
             Fix: add parse_iso_date() + horizon_days < 0 check to PoxpartyArgs::validate()."
        );
    }

    #[test]
    fn anergy_validate_rejects_until_in_the_past() {
        // ATK-ANERGY-PAST-DATE: #[anergy] with `until` in the past should fail
        // validation — an already-expired anergy window means the suppression
        // should have been re-evaluated but wasn't. Silent expired anergy is
        // worse than orient: it silently suppresses checks for a failure-class
        // on an item where the anergy contract has lapsed.
        //
        // FIXED: validate() now rejects past dates (parse_iso_date() + horizon_days < 0 check added).
        let yesterday = (today_utc() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let tokens: TokenStream = format!(
            r#"PanickingInDrop,
            reason = "Suppressing until we audit all Drop impls in the codebase",
            until = "{yesterday}""#
        )
        .parse()
        .unwrap();
        let args = syn::parse2::<AnergyArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-ANERGY-PAST-DATE: #[anergy] with until in the past must fail validation — \
             an expired anergy window represents silent check-suppression with no accountability. \
             Got Ok(()) for until={yesterday}. \
             Fix: add parse_iso_date() + horizon_days < 0 check to AnergyArgs::validate() \
             parallel to OrientArgs commit 53d2bab."
        );
    }

    #[test]
    fn anergy_validate_rejects_invalid_date_format() {
        // ATK-ANERGY-INVALID-DATE: #[anergy] with a syntactically invalid `until`
        // date silently passes validation. When parse_iso_date returns Err(()),
        // the `if let Ok(...)` arm is not entered and no error is produced.
        //
        // A user who writes `until = "not-a-date"` or `until = "v2.0"` gets a
        // silently accepted macro — the anergy window is unbounded (no parseable
        // date means no past-date check, no expiry). This is worse than a past date.
        //
        // validate() must reject unparseable `until` values explicitly.
        let tokens: TokenStream =
            r#"PanickingInDrop, reason = "Suppressing until we audit all Drop impls in the codebase", until = "not-a-date""#
                .parse()
                .unwrap();
        let args = syn::parse2::<AnergyArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-ANERGY-INVALID-DATE: #[anergy] with until = 'not-a-date' must fail validation. \
             parse_iso_date returns Err(()), so the if-let-Ok arm is skipped — no error returned. \
             An unparseable until date is an unbounded anergy window (no expiry ever fires). \
             Fix: after the parse_iso_date call, also return Err if parsing FAILED — \
             invalid dates should be rejected, not silently accepted."
        );
    }

    #[test]
    fn immunosuppress_validate_rejects_invalid_date_format() {
        // ATK-IMMUNOSUPPRESS-INVALID-DATE: same gap as anergy — unparseable `until`
        // silently passes. ImmunosuppressArgs::validate() uses the same if-let-Ok pattern.
        let tokens: TokenStream =
            r#"PanickingInDrop, rationale = "Suppressing this check until we audit all Drop impls", until = "2999-13-01""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ImmunosuppressArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-IMMUNOSUPPRESS-INVALID-DATE: #[immunosuppress] with until = '2999-13-01' \
             (invalid month 13) must fail validation. parse_iso_date returns Err(()), \
             so the if-let-Ok arm is skipped — no error returned. \
             An unparseable until date is an unbounded suppression window. \
             Fix: treat parse_iso_date failure as a validation error."
        );
    }

    #[test]
    fn poxparty_validate_rejects_invalid_date_format() {
        // ATK-POXPARTY-INVALID-DATE: same gap as anergy/immunosuppress. PoxpartyArgs
        // uses the same if-let-Ok(parse_iso_date) pattern — invalid dates silently pass.
        let tokens: TokenStream =
            r#"PanickingInDrop, exercise_type = "Trigger deliberate Drop panic and measure detection lag", until = "v2.0-release""#
                .parse()
                .unwrap();
        let args = syn::parse2::<PoxpartyArgs>(tokens).unwrap();
        assert!(
            args.validate().is_err(),
            "ATK-POXPARTY-INVALID-DATE: #[poxparty] with until = 'v2.0-release' \
             must fail validation. parse_iso_date returns Err(()), so the if-let-Ok \
             arm is skipped — no error returned. A version tag as until date creates \
             an unbounded poxparty window that never expires. \
             Fix: treat parse_iso_date failure as a validation error."
        );
    }

    // -------------------------------------------------------------------------
    // ATK-WHITESPACE-RATIONALE: deferred-defense macros (anergy/immunosuppress/
    // poxparty/orient) have the same whitespace-stuffing gap as triage_commit did
    // (which was fixed at line 1640). The len() < 20 check passes for all-space
    // strings. These tests will FAIL until the trim().is_empty() guards are added
    // to the four deferred-defense validate() methods.
    //
    // The systemic root: ADR-023 loudness-as-discipline implemented as min-length
    // but whitespace-stuffing was not considered. triage_commit got fixed first
    // (via the hook in response to adversarial's failing test); these four remain.
    // -------------------------------------------------------------------------

    #[test]
    fn anergy_validate_rejects_whitespace_only_reason_of_20_chars() {
        let tokens: TokenStream =
            r#"PanickingInDrop, reason = "                    ", until = "2099-12-31""#
                .parse()
                .unwrap();
        let args = syn::parse2::<AnergyArgs>(tokens).unwrap();
        assert_eq!(args.reason.as_deref().map(str::len), Some(20));
        assert!(
            args.validate().is_err(),
            "ATK-WHITESPACE-RATIONALE: #[anergy] reason of 20 spaces must fail validation. \
             AnergyArgs::validate() line 643 uses r.len() < 20 but not trim().is_empty(). \
             Fix: add 'Some(r) if r.trim().is_empty() => Err(...)' arm to the reason match, \
             parallel to triage_commit's fix at line 1640."
        );
    }

    #[test]
    fn immunosuppress_validate_rejects_whitespace_only_rationale_of_20_chars() {
        let tokens: TokenStream =
            r#"PanickingInDrop, rationale = "                    ", until = "2099-12-31""#
                .parse()
                .unwrap();
        let args = syn::parse2::<ImmunosuppressArgs>(tokens).unwrap();
        assert_eq!(args.rationale.as_deref().map(str::len), Some(20));
        assert!(
            args.validate().is_err(),
            "ATK-WHITESPACE-RATIONALE: #[immunosuppress] rationale of 20 spaces must fail. \
             ImmunosuppressArgs::validate() line 843 uses r.len() < 20 without trim(). \
             Fix: add trim().is_empty() arm parallel to triage_commit fix at line 1640."
        );
    }

    #[test]
    fn poxparty_validate_rejects_whitespace_only_exercise_type_of_20_chars() {
        let tokens: TokenStream =
            r#"PanickingInDrop, exercise_type = "                    ", until = "2099-12-31""#
                .parse()
                .unwrap();
        let args = syn::parse2::<PoxpartyArgs>(tokens).unwrap();
        assert_eq!(args.exercise_type.as_deref().map(str::len), Some(20));
        assert!(
            args.validate().is_err(),
            "ATK-WHITESPACE-RATIONALE: #[poxparty] exercise_type of 20 spaces must fail. \
             PoxpartyArgs::validate() line 1062 uses et.len() < 20 without trim(). \
             Fix: add trim().is_empty() arm parallel to triage_commit fix at line 1640."
        );
    }

    #[test]
    fn orient_validate_rejects_whitespace_only_learning_path_of_20_chars() {
        let until = (today_utc() + chrono::Duration::days(90))
            .format("%Y-%m-%d")
            .to_string();
        let src = format!(
            r#"PanickingInDrop, learning_path = "                    ", until = "{until}""#
        );
        let tokens: TokenStream = src.parse().unwrap();
        let args = syn::parse2::<OrientArgs>(tokens).unwrap();
        assert_eq!(args.learning_path.as_deref().map(str::len), Some(20));
        assert!(
            args.validate().is_err(),
            "ATK-WHITESPACE-RATIONALE: #[orient] learning_path of 20 spaces must fail. \
             OrientArgs::validate() line 1269 uses p.len() < 20 without trim(). \
             Fix: add trim().is_empty() arm parallel to triage_commit fix at line 1640."
        );
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

    // ========================================================================
    // DefendedByArgs (ADR-029) — code-tier witness registration
    // ========================================================================

    #[test]
    fn defended_by_parses_bare_antigen_path() {
        let tokens: proc_macro2::TokenStream = "ParallelStateTrackersDiverge".parse().unwrap();
        let args = syn::parse2::<DefendedByArgs>(tokens).expect("bare path parses");
        assert_eq!(
            args.antigen.segments.last().unwrap().ident.to_string(),
            "ParallelStateTrackersDiverge"
        );
    }

    #[test]
    fn defended_by_parses_qualified_path() {
        let tokens: proc_macro2::TokenStream = "crate::antigens::DropPanicClass".parse().unwrap();
        let args = syn::parse2::<DefendedByArgs>(tokens).expect("qualified path parses");
        assert_eq!(
            args.antigen.segments.last().unwrap().ident.to_string(),
            "DropPanicClass"
        );
    }

    #[test]
    fn defended_by_rejects_trailing_witness_args() {
        // The old #[immune] shape (`X, witness = fn`) must NOT silently parse:
        // #[defended_by] carries only the failure-class. Site-attached evidence
        // folds into #[presents] (ADR-029 R5). Reject loudly with guidance.
        let tokens: proc_macro2::TokenStream =
            "DropPanicClass, witness = some_test".parse().unwrap();
        let err = syn::parse2::<DefendedByArgs>(tokens)
            .expect_err("trailing args must be rejected")
            .to_string();
        assert!(
            err.contains("exactly one positional argument"),
            "error must explain the single-arg shape; got: {err:?}"
        );
    }

    #[test]
    fn defended_by_rejects_empty() {
        // A bare `#[defended_by]` (no antigen) is a witness for nothing.
        let tokens = proc_macro2::TokenStream::new();
        assert!(
            syn::parse2::<DefendedByArgs>(tokens).is_err(),
            "empty #[defended_by] body must be rejected"
        );
    }

    // ========================================================================
    // PresentsArgs (ADR-029) — site-attached evidence folding
    // ========================================================================

    #[test]
    fn presents_parses_bare_antigen_still() {
        // Back-compat: the v0.1 single-positional form still parses.
        let tokens: proc_macro2::TokenStream = "PanickingInDrop".parse().unwrap();
        let args = syn::parse2::<PresentsArgs>(tokens).expect("bare presents parses");
        assert!(args.requires.is_none());
        assert!(args.proof.is_none());
        assert!(args.validate().is_ok());
    }

    #[test]
    fn presents_parses_requires_predicate() {
        // ADR-029: substrate-tier evidence folds onto #[presents].
        let tokens: proc_macro2::TokenStream =
            r#"UnpinnedDependency, requires = ratified_doc(path = "docs/x.md")"#
                .parse()
                .unwrap();
        let args = syn::parse2::<PresentsArgs>(tokens).expect("requires folds in");
        assert!(args.requires.is_some());
        assert!(args.validate().is_ok());
        // The emitted marker JSON must be present for scan discovery.
        assert!(args.requires_json().is_some());
    }

    #[test]
    fn presents_parses_proof_expression() {
        // ADR-029: phantom-tier evidence folds onto #[presents].
        let tokens: proc_macro2::TokenStream =
            "DropPanicClass, proof = NonPanickingProof::<T>::verified"
                .parse()
                .unwrap();
        let args = syn::parse2::<PresentsArgs>(tokens).expect("proof folds in");
        assert!(args.proof.is_some());
        // proof= is recognized structurally by audit; no requires marker.
        assert!(args.requires_json().is_none());
    }

    #[test]
    fn presents_rejects_unknown_field() {
        // A code-tier `witness =` on #[presents] is a misuse — it belongs on
        // #[defended_by]. Reject with guidance (R5 discriminator).
        let body = "DropPanicClass, witness = some_test";
        let tokens: proc_macro2::TokenStream = body.parse().unwrap();
        let err = syn::parse2::<PresentsArgs>(tokens)
            .expect_err("witness= on presents is rejected")
            .to_string();
        assert!(
            err.contains("defended_by"),
            "error should point at #[defended_by] for code-tier evidence; got: {err:?}"
        );
    }
}

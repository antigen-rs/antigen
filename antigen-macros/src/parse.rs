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
use syn::{Expr, Ident, Lit, LitBool, LitInt, LitStr, Path, Token};

// ============================================================================
// RequiresExpr — local mirror of antigen_attestation::Predicate
//
// proc-macro crates cannot depend on antigen-attestation (circular dep risk
// if antigen-attestation ever transitively imports antigen-macros; also
// proc-macro crates can't link arbitrary libs at expansion time). Instead,
// we mirror the predicate AST here and lower it to the same JSON serde
// format that antigen_attestation::Predicate produces, so the scan layer
// can round-trip via `serde_json::from_str::<Predicate>()`.
//
// JSON format (must match antigen_attestation::Predicate serde):
//   AllOf  -> {"kind":"all_of","children":[...]}
//   AnyOf  -> {"kind":"any_of","children":[...]}
//   Not    -> {"kind":"not","child":{...}}
//   Leaf   -> {"kind":"leaf","leaf":{"name":"<leaf_name>",...leaf_fields...}}
//
// Macro syntax parsed:
//   requires = signers(required = ["alice"])
//   requires = all_of([signers(required = ["alice"]), fresh_within_days(days = 90)])
//   requires = any_of([...])
//   requires = not(...)
// ============================================================================

/// Local predicate AST — mirrors `antigen_attestation::Predicate` without
/// creating a crate dependency.
#[derive(Debug, Clone)]
pub enum RequiresExpr {
    Leaf(LeafExpr),
    AllOf(Vec<Self>),
    AnyOf(Vec<Self>),
    Not(Box<Self>),
}

/// Local leaf AST — mirrors `antigen_attestation::Leaf`.
#[derive(Debug, Clone)]
pub enum LeafExpr {
    RatifiedDoc {
        path: Option<String>,
        min_version: Option<String>,
        anchor: Option<String>,
        sibling_json: bool,
    },
    Signers {
        required: Vec<String>,
        against: SignerCurrencyExpr,
    },
    SignedTrailer {
        key: String,
        role: Option<String>,
        count: u32,
    },
    OraclesComplete {
        files: Vec<String>,
    },
    FreshWithinDays {
        days: u32,
    },
}

/// Local mirror of `antigen_attestation::SignerCurrency`.
#[derive(Debug, Clone, Copy, Default)]
pub enum SignerCurrencyExpr {
    #[default]
    Current,
    Any,
}

impl RequiresExpr {
    /// Lower to the JSON string consumed by the scan layer.
    ///
    /// Format matches `serde_json::to_string(&antigen_attestation::Predicate)`.
    pub fn to_json(&self) -> String {
        match self {
            Self::AllOf(children) => {
                let kids: Vec<String> = children.iter().map(Self::to_json).collect();
                format!("{{\"kind\":\"all_of\",\"children\":[{}]}}", kids.join(","))
            }
            Self::AnyOf(children) => {
                let kids: Vec<String> = children.iter().map(Self::to_json).collect();
                format!("{{\"kind\":\"any_of\",\"children\":[{}]}}", kids.join(","))
            }
            Self::Not(child) => {
                format!("{{\"kind\":\"not\",\"child\":{}}}", child.to_json())
            }
            Self::Leaf(leaf) => {
                format!("{{\"kind\":\"leaf\",\"leaf\":{}}}", leaf.to_json())
            }
        }
    }

    /// Validate semantic invariants at parse time (mirrors
    /// `antigen_attestation::Predicate::validate`).
    pub fn validate(&self, span: Span) -> syn::Result<()> {
        match self {
            Self::AllOf(children) if children.is_empty() => Err(syn::Error::new(
                span,
                "requires: `all_of([])` is a semantic no-op (R-A6); add at least one child",
            )),
            Self::AnyOf(children) if children.is_empty() => Err(syn::Error::new(
                span,
                "requires: `any_of([])` is a semantic no-op (R-A6); add at least one child",
            )),
            Self::AllOf(children) | Self::AnyOf(children) => {
                for child in children {
                    child.validate(span)?;
                }
                Ok(())
            }
            Self::Not(child) => child.validate(span),
            Self::Leaf(leaf) => leaf.validate(span),
        }
    }
}

impl LeafExpr {
    fn to_json(&self) -> String {
        match self {
            Self::RatifiedDoc {
                path,
                min_version,
                anchor,
                sibling_json,
            } => {
                let mut fields = vec![r#""name":"ratified_doc""#.to_string()];
                if let Some(p) = path {
                    fields.push(format!("\"path\":{}", json_string(p)));
                }
                if let Some(v) = min_version {
                    fields.push(format!("\"min_version\":{}", json_string(v)));
                }
                if let Some(a) = anchor {
                    fields.push(format!("\"anchor\":{}", json_string(a)));
                }
                if *sibling_json {
                    fields.push("\"sibling_json\":true".to_string());
                }
                format!("{{{}}}", fields.join(","))
            }
            Self::Signers { required, against } => {
                let req_arr: Vec<String> = required.iter().map(|s| json_string(s)).collect();
                let mut s = format!(
                    "{{\"name\":\"signers\",\"required\":[{}]",
                    req_arr.join(",")
                );
                match against {
                    SignerCurrencyExpr::Any => {
                        s.push_str(",\"against\":\"any\"");
                    }
                    SignerCurrencyExpr::Current => {
                        // "current" is the serde default — skip_serializing_if omits it
                    }
                }
                s.push('}');
                s
            }
            Self::SignedTrailer { key, role, count } => {
                let mut fields = vec![
                    r#""name":"signed_trailer""#.to_string(),
                    format!("\"key\":{}", json_string(key)),
                ];
                if let Some(r) = role {
                    fields.push(format!("\"role\":{}", json_string(r)));
                }
                if *count != 1 {
                    // 1 is the serde default_trailer_count — omit when default
                    fields.push(format!("\"count\":{count}"));
                }
                format!("{{{}}}", fields.join(","))
            }
            Self::OraclesComplete { files } => {
                let arr: Vec<String> = files.iter().map(|f| json_string(f)).collect();
                format!(
                    "{{\"name\":\"oracles_complete\",\"files\":[{}]}}",
                    arr.join(",")
                )
            }
            Self::FreshWithinDays { days } => {
                format!("{{\"name\":\"fresh_within_days\",\"days\":{days}}}")
            }
        }
    }

    fn validate(&self, span: Span) -> syn::Result<()> {
        match self {
            Self::Signers { required, .. } if required.is_empty() => Err(syn::Error::new(
                span,
                "requires: `signers(required = [])` is a semantic no-op (NFA-7); \
                 add at least one required signer name",
            )),
            Self::OraclesComplete { files } if files.is_empty() => Err(syn::Error::new(
                span,
                "requires: `oracles_complete(files = [])` is a semantic no-op (NFA-8); \
                 add at least one oracle file path",
            )),
            Self::SignedTrailer { count: 0, .. } => Err(syn::Error::new(
                span,
                "requires: `signed_trailer(count = 0)` is vacuously true (NFA-9); \
                 use count >= 1 (default is 1)",
            )),
            _ => Ok(()),
        }
    }
}

/// Escape a string as a JSON string literal (double-quoted, inner
/// quotes and backslashes escaped). No other escapes needed for the
/// ASCII-safe content in predicate fields (paths, signer names, etc.).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ============================================================================
// RequiresExpr parsing
// ============================================================================

impl Parse for RequiresExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();
        let span = name.span();

        match name_str.as_str() {
            "all_of" => {
                let content;
                syn::parenthesized!(content in input);
                let inner;
                syn::bracketed!(inner in content);
                let children = parse_predicate_list(&inner)?;
                if children.is_empty() {
                    return Err(syn::Error::new(
                        span,
                        "all_of([]) is a semantic no-op (R-A6); add at least one child",
                    ));
                }
                Ok(Self::AllOf(children))
            }
            "any_of" => {
                let content;
                syn::parenthesized!(content in input);
                let inner;
                syn::bracketed!(inner in content);
                let children = parse_predicate_list(&inner)?;
                if children.is_empty() {
                    return Err(syn::Error::new(
                        span,
                        "any_of([]) is a semantic no-op (R-A6); add at least one child",
                    ));
                }
                Ok(Self::AnyOf(children))
            }
            "not" => {
                let content;
                syn::parenthesized!(content in input);
                let child: Self = content.parse()?;
                Ok(Self::Not(Box::new(child)))
            }
            leaf_name => {
                let leaf = parse_leaf(leaf_name, span, input)?;
                Ok(Self::Leaf(leaf))
            }
        }
    }
}

fn parse_predicate_list(input: ParseStream) -> syn::Result<Vec<RequiresExpr>> {
    let mut children = Vec::new();
    while !input.is_empty() {
        children.push(input.parse::<RequiresExpr>()?);
        if input.is_empty() {
            break;
        }
        input.parse::<Token![,]>()?;
    }
    Ok(children)
}

fn parse_leaf(name: &str, span: Span, input: ParseStream) -> syn::Result<LeafExpr> {
    match name {
        "ratified_doc" => parse_ratified_doc(span, input),
        "signers" => parse_signers(span, input),
        "signed_trailer" => parse_signed_trailer(span, input),
        "oracles_complete" => parse_oracles_complete(span, input),
        "fresh_within_days" => parse_fresh_within_days(span, input),
        other => Err(syn::Error::new(
            span,
            format!(
                "unknown substrate-witness predicate leaf `{other}`; \
                 v0.1 sealed set: ratified_doc, signers, signed_trailer, \
                 oracles_complete, fresh_within_days"
            ),
        )),
    }
}

/// Parse a `key = value` pair inside a leaf's parentheses, returning the key
/// ident and advancing past the `=`.
fn parse_kv_key(input: ParseStream) -> syn::Result<Ident> {
    let key: Ident = input.parse()?;
    input.parse::<Token![=]>()?;
    Ok(key)
}

/// Parse a `[str, str, ...]` bracketed string array (for `required`, `files`).
fn parse_string_array(input: ParseStream) -> syn::Result<Vec<String>> {
    let inner;
    syn::bracketed!(inner in input);
    let mut out = Vec::new();
    while !inner.is_empty() {
        let s: LitStr = inner.parse()?;
        out.push(s.value());
        if inner.is_empty() {
            break;
        }
        inner.parse::<Token![,]>()?;
    }
    Ok(out)
}

fn parse_ratified_doc(_span: Span, input: ParseStream) -> syn::Result<LeafExpr> {
    let mut path: Option<String> = None;
    let mut min_version: Option<String> = None;
    let mut anchor: Option<String> = None;
    let mut sibling_json = false;

    // ratified_doc may be bare (no parens) or have named args
    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        while !content.is_empty() {
            let key = parse_kv_key(&content)?;
            match key.to_string().as_str() {
                "path" => {
                    let s: LitStr = content.parse()?;
                    path = Some(s.value());
                }
                "min_version" => {
                    let s: LitStr = content.parse()?;
                    min_version = Some(s.value());
                }
                "anchor" => {
                    let s: LitStr = content.parse()?;
                    anchor = Some(s.value());
                }
                "sibling_json" => {
                    let b: LitBool = content.parse()?;
                    sibling_json = b.value();
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown ratified_doc field `{other}`; \
                             expected: path, min_version, anchor, sibling_json"
                        ),
                    ));
                }
            }
            if content.is_empty() {
                break;
            }
            content.parse::<Token![,]>()?;
        }
    }

    Ok(LeafExpr::RatifiedDoc {
        path,
        min_version,
        anchor,
        sibling_json,
    })
}

fn parse_signers(span: Span, input: ParseStream) -> syn::Result<LeafExpr> {
    let mut required: Vec<String> = Vec::new();
    let mut against = SignerCurrencyExpr::Current;

    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        while !content.is_empty() {
            let key = parse_kv_key(&content)?;
            match key.to_string().as_str() {
                "required" => {
                    required = parse_string_array(&content)?;
                }
                "against" => {
                    let s: LitStr = content.parse()?;
                    against = match s.value().as_str() {
                        "current" => SignerCurrencyExpr::Current,
                        "any" => SignerCurrencyExpr::Any,
                        other => {
                            return Err(syn::Error::new(
                                s.span(),
                                format!(
                                    "unknown signers `against` value `{other}`; \
                                     expected \"current\" or \"any\""
                                ),
                            ))
                        }
                    };
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown signers field `{other}`; expected: required, against"
                        ),
                    ));
                }
            }
            if content.is_empty() {
                break;
            }
            content.parse::<Token![,]>()?;
        }
    }

    if required.is_empty() {
        return Err(syn::Error::new(
            span,
            "signers(required = [...]) must include at least one name (NFA-7)",
        ));
    }

    Ok(LeafExpr::Signers { required, against })
}

fn parse_signed_trailer(span: Span, input: ParseStream) -> syn::Result<LeafExpr> {
    let mut key: Option<String> = None;
    let mut role: Option<String> = None;
    let mut count: u32 = 1;

    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        while !content.is_empty() {
            let k = parse_kv_key(&content)?;
            match k.to_string().as_str() {
                "key" => {
                    let s: LitStr = content.parse()?;
                    key = Some(s.value());
                }
                "role" => {
                    let s: LitStr = content.parse()?;
                    role = Some(s.value());
                }
                "count" => {
                    let n: LitInt = content.parse()?;
                    count = n.base10_parse::<u32>()?;
                }
                other => {
                    return Err(syn::Error::new(
                        k.span(),
                        format!(
                            "unknown signed_trailer field `{other}`; \
                             expected: key, role, count"
                        ),
                    ));
                }
            }
            if content.is_empty() {
                break;
            }
            content.parse::<Token![,]>()?;
        }
    }

    let key = key.ok_or_else(|| {
        syn::Error::new(span, "signed_trailer requires `key = \"...\"` (the trailer key, e.g., \"Discipline-Verified-By\")")
    })?;

    if count == 0 {
        return Err(syn::Error::new(
            span,
            "signed_trailer `count = 0` is vacuously true (NFA-9); use count >= 1",
        ));
    }

    Ok(LeafExpr::SignedTrailer { key, role, count })
}

fn parse_oracles_complete(span: Span, input: ParseStream) -> syn::Result<LeafExpr> {
    let mut files: Vec<String> = Vec::new();

    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        while !content.is_empty() {
            let key = parse_kv_key(&content)?;
            match key.to_string().as_str() {
                "files" => {
                    files = parse_string_array(&content)?;
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown oracles_complete field `{other}`; expected: files"),
                    ));
                }
            }
            if content.is_empty() {
                break;
            }
            content.parse::<Token![,]>()?;
        }
    }

    if files.is_empty() {
        return Err(syn::Error::new(
            span,
            "oracles_complete(files = [...]) must include at least one file path (NFA-8)",
        ));
    }

    Ok(LeafExpr::OraclesComplete { files })
}

fn parse_fresh_within_days(span: Span, input: ParseStream) -> syn::Result<LeafExpr> {
    let days = if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        if content.peek(Ident) && content.peek2(Token![=]) {
            // named: fresh_within_days(days = 90)
            let key: Ident = content.parse()?;
            if key != "days" {
                return Err(syn::Error::new(
                    key.span(),
                    format!("unknown fresh_within_days field `{key}`; expected: days"),
                ));
            }
            content.parse::<Token![=]>()?;
        }
        // positional or named — both end with a u32 literal
        let n: LitInt = content.parse()?;
        n.base10_parse::<u32>()?
    } else {
        return Err(syn::Error::new(
            span,
            "fresh_within_days requires a day count: `fresh_within_days(90)` or \
             `fresh_within_days(days = 90)`",
        ));
    };

    Ok(LeafExpr::FreshWithinDays { days })
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
                other => {
                    return Err(syn::Error::new(
                        pair.key.span(),
                        format!(
                            "unknown #[antigen] field `{other}`; expected one of: \
                                 name, fingerprint, family, summary, references"
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
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
}

// ============================================================================
// RequiresExpr JSON lowering tests
//
// These tests lock the JSON format that RequiresExpr::to_json() produces
// against the format that antigen_attestation::Predicate serde produces.
// The canonical format is documented at:
//   antigen-attestation/src/predicate.rs (serde attributes)
//   antigen-attestation/src/predicate.rs:525 (leaf JSON form: {"kind":"leaf","leaf":{...}})
//
// When adding a new leaf or changing serde attributes, update these tests
// AND verify the format matches via the predicate.rs round-trip test.
// ============================================================================

#[cfg(test)]
mod requires_json_tests {
    use super::*;
    use proc_macro2::TokenStream;

    fn parse_requires(input: &str) -> RequiresExpr {
        let tokens: TokenStream = input.parse().expect("tokenize");
        syn::parse2::<RequiresExpr>(tokens).expect("parse RequiresExpr")
    }

    #[test]
    fn leaf_fresh_within_days_positional_json() {
        let expr = parse_requires("fresh_within_days(90)");
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"fresh_within_days","days":90}}"#
        );
    }

    #[test]
    fn leaf_fresh_within_days_named_json() {
        let expr = parse_requires("fresh_within_days(days = 180)");
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"fresh_within_days","days":180}}"#
        );
    }

    #[test]
    fn leaf_signers_current_against_omitted_in_json() {
        // "current" is the serde default — must be omitted (skip_serializing_if)
        let expr = parse_requires(r#"signers(required = ["alice", "bob"])"#);
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"signers","required":["alice","bob"]}}"#
        );
    }

    #[test]
    fn leaf_signers_any_against_present_in_json() {
        let expr = parse_requires(r#"signers(required = ["alice"], against = "any")"#);
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"signers","required":["alice"],"against":"any"}}"#
        );
    }

    #[test]
    fn leaf_ratified_doc_bare_json() {
        let expr = parse_requires("ratified_doc");
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"ratified_doc"}}"#
        );
    }

    #[test]
    fn leaf_ratified_doc_with_path_json() {
        let expr = parse_requires(r#"ratified_doc(path = "docs/discipline.md")"#);
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"ratified_doc","path":"docs/discipline.md"}}"#
        );
    }

    #[test]
    fn leaf_ratified_doc_sibling_json_flag() {
        let expr = parse_requires("ratified_doc(sibling_json = true)");
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"ratified_doc","sibling_json":true}}"#
        );
    }

    #[test]
    fn leaf_oracles_complete_json() {
        let expr = parse_requires(r#"oracles_complete(files = ["a.md", "b.md"])"#);
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"oracles_complete","files":["a.md","b.md"]}}"#
        );
    }

    #[test]
    fn leaf_signed_trailer_default_count_omitted_json() {
        // count=1 is the serde default — must be omitted
        let expr = parse_requires(r#"signed_trailer(key = "Discipline-Verified-By")"#);
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"signed_trailer","key":"Discipline-Verified-By"}}"#
        );
    }

    #[test]
    fn leaf_signed_trailer_non_default_count_present_json() {
        let expr = parse_requires(r#"signed_trailer(key = "Verified-By", count = 2)"#);
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"leaf","leaf":{"name":"signed_trailer","key":"Verified-By","count":2}}"#
        );
    }

    #[test]
    fn combinator_all_of_json() {
        let expr = parse_requires(
            r#"all_of([fresh_within_days(90), signers(required = ["alice"])])"#,
        );
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"all_of","children":[{"kind":"leaf","leaf":{"name":"fresh_within_days","days":90}},{"kind":"leaf","leaf":{"name":"signers","required":["alice"]}}]}"#
        );
    }

    #[test]
    fn combinator_any_of_json() {
        let expr = parse_requires(
            r"any_of([fresh_within_days(30), fresh_within_days(90)])",
        );
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"any_of","children":[{"kind":"leaf","leaf":{"name":"fresh_within_days","days":30}},{"kind":"leaf","leaf":{"name":"fresh_within_days","days":90}}]}"#
        );
    }

    #[test]
    fn combinator_not_json() {
        let expr = parse_requires(r"not(fresh_within_days(90))");
        let json = expr.to_json();
        assert_eq!(
            json,
            r#"{"kind":"not","child":{"kind":"leaf","leaf":{"name":"fresh_within_days","days":90}}}"#
        );
    }

    #[test]
    fn json_string_escapes_quotes_and_backslash() {
        // Signer names with special chars must be safely JSON-escaped.
        let expr = parse_requires(r#"signers(required = ["alice\"bob"])"#);
        let json = expr.to_json();
        // The name "alice\"bob" in Rust source is the string alice"bob,
        // which in JSON is "alice\"bob".
        assert!(json.contains(r#"\"alice\\\"bob\""#) || json.contains(r#"alice\"bob"#));
    }

    #[test]
    fn requires_expr_validate_rejects_empty_all_of() {
        // all_of([]) is caught at parse time (not validate time) but the
        // validate() method must also reject it if somehow constructed.
        let span = proc_macro2::Span::call_site();
        let expr = RequiresExpr::AllOf(vec![]);
        assert!(expr.validate(span).is_err());
    }

    #[test]
    fn requires_expr_validate_rejects_empty_signers() {
        let span = proc_macro2::Span::call_site();
        let expr = RequiresExpr::Leaf(LeafExpr::Signers {
            required: vec![],
            against: SignerCurrencyExpr::Current,
        });
        assert!(expr.validate(span).is_err());
    }

    #[test]
    fn requires_expr_validate_passes_well_formed() {
        let span = proc_macro2::Span::call_site();
        let expr = RequiresExpr::AllOf(vec![
            RequiresExpr::Leaf(LeafExpr::Signers {
                required: vec!["alice".to_string()],
                against: SignerCurrencyExpr::Current,
            }),
            RequiresExpr::Leaf(LeafExpr::FreshWithinDays { days: 90 }),
        ]);
        assert!(expr.validate(span).is_ok());
    }
}

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

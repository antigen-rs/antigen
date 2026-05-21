//! Source-level parser for `#[immune(..., requires = <predicate>)]` and
//! `#[antigen_tolerance(..., requires = <predicate>)]` macro arguments.
//!
//! ## Why this lives here (and not in `antigen-macros`)
//!
//! The substrate-witness predicate AST ([`crate::Predicate`]) is the runtime
//! type the audit evaluator consumes. The macro-side AST that parses
//! `requires = all_of([...])` syntax was originally a local mirror inside
//! `antigen-macros`, and the scan layer rediscovered the JSON via a
//! `#[doc = " antigen:requires:v1:<json>"]` marker emitted by the macro.
//!
//! That round-trip was load-bearing: scan walks **written source**, not
//! post-expansion source. The doc-marker only exists after macro expansion,
//! so scan never saw it. The substrate-witness pipeline silently failed
//! (`antigen audit` reported `tier = None, hint = NoneApplicable` for every
//! `requires = ...` site, even the shipped example).
//!
//! Fix: co-locate the source-attribute parser with the predicate type, gate
//! it behind a `parser` feature so the runtime crate stays syn-free by
//! default. Both the proc-macro crate and the scan layer turn the feature
//! on and use a single parser. The macro continues to emit the doc marker
//! for forward-compatibility, but discovery now happens via source-attr
//! parsing — independent of macro expansion.
//!
//! ## Format contract
//!
//! [`RequiresExpr::to_json`] produces the exact JSON that
//! `serde_json::to_string(&crate::Predicate)` produces. The scan layer
//! round-trips via `serde_json::from_str::<crate::Predicate>(&json)` to
//! feed the audit evaluator. The format is locked by the test fixtures
//! at the bottom of this module; any divergence between the macro-side
//! AST and the runtime [`crate::Predicate`] serde representation is a
//! bug.
//!
//! ## Grammar parsed (mirrors `Predicate` grammar, ADR-019 §M2)
//!
//! ```text
//! requires = leaf
//!         | all_of([predicate, ...])
//!         | any_of([predicate, ...])
//!         | not(predicate)
//!
//! leaf := ratified_doc[(path=..., min_version=..., anchor=..., sibling_json=...)]
//!       | signers(required = [...], against = "current"|"any")
//!       | signed_trailer(key = "...", role = "...", count = N)
//!       | oracles_complete(files = ["...", ...])
//!       | fresh_within_days(N) | fresh_within_days(days = N)
//! ```

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitBool, LitInt, LitStr, Token};

// ============================================================================
// RequiresExpr — local mirror of crate::Predicate for compile-time parsing
//
// JSON format (must match crate::Predicate serde):
//   AllOf  -> {"kind":"all_of","children":[...]}
//   AnyOf  -> {"kind":"any_of","children":[...]}
//   Not    -> {"kind":"not","child":{...}}
//   Leaf   -> {"kind":"leaf","leaf":{"name":"<leaf_name>",...leaf_fields...}}
// ============================================================================

/// Local predicate AST — mirrors [`crate::Predicate`] without forcing
/// `proc-macro2`/`syn` on every consumer of the runtime crate.
#[derive(Debug, Clone)]
pub enum RequiresExpr {
    /// A single leaf primitive.
    Leaf(LeafExpr),
    /// `all_of([...])` combinator (every child must pass).
    AllOf(Vec<Self>),
    /// `any_of([...])` combinator (at least one child must pass).
    AnyOf(Vec<Self>),
    /// `not(...)` combinator (child must not pass).
    Not(Box<Self>),
}

/// Local leaf AST — mirrors [`crate::Leaf`].
#[derive(Debug, Clone)]
pub enum LeafExpr {
    /// `ratified_doc(path=..., min_version=..., anchor=..., sibling_json=...)`.
    RatifiedDoc {
        /// Optional doc path (relative to workspace root).
        path: Option<String>,
        /// Optional minimum frontmatter version (semver-cmp).
        min_version: Option<String>,
        /// Optional anchor string the doc body must contain.
        anchor: Option<String>,
        /// Whether to require a sibling JSON file at the same path.
        sibling_json: bool,
    },
    /// `signers(required = [...], against = "current"|"any")`.
    Signers {
        /// Required signer names (non-empty per NFA-7).
        required: Vec<String>,
        /// Whether to require currency against the live fingerprint.
        against: SignerCurrencyExpr,
    },
    /// `signed_trailer(key = "...", role = "...", count = N)`.
    SignedTrailer {
        /// The trailer key, e.g. `"Discipline-Verified-By"`.
        key: String,
        /// Optional role token filter.
        role: Option<String>,
        /// Required count (>= 1; default 1).
        count: u32,
    },
    /// `oracles_complete(files = ["a.md", "b.md"])`.
    OraclesComplete {
        /// Oracle file paths that must each report `status: complete`.
        files: Vec<String>,
    },
    /// `fresh_within_days(N)` / `fresh_within_days(days = N)`.
    FreshWithinDays {
        /// Maximum age in days for the most recent signature.
        days: u32,
    },
}

/// Local mirror of [`crate::predicate::SignerCurrency`].
#[derive(Debug, Clone, Copy, Default)]
pub enum SignerCurrencyExpr {
    /// Signers must be current against the live fingerprint (serde default).
    #[default]
    Current,
    /// Signers may be against any historical fingerprint.
    Any,
}

impl RequiresExpr {
    /// Convert to the runtime [`crate::Predicate`] AST.
    ///
    /// This is the load-bearing lowering: scan and audit talk to each
    /// other via JSON, and the JSON shape is whatever
    /// `serde_json::to_string(&crate::Predicate)` produces. Going through
    /// the real Predicate type — rather than hand-rolling the JSON — keeps
    /// the wire format and the runtime type locked in step automatically.
    ///
    /// rc.1 history: the macro-side AST hand-rolled JSON that did NOT
    /// match the runtime Predicate serde format (used `{"leaf": {...}}`
    /// instead of flat fields), so the round-trip silently failed and
    /// every substrate-witness audit reported sidecar-schema-invalid.
    /// rc.2 routes through the real type, eliminating the class.
    #[must_use]
    pub fn to_predicate(&self) -> crate::Predicate {
        match self {
            Self::AllOf(children) => crate::Predicate::AllOf {
                children: children.iter().map(Self::to_predicate).collect(),
            },
            Self::AnyOf(children) => crate::Predicate::AnyOf {
                children: children.iter().map(Self::to_predicate).collect(),
            },
            Self::Not(child) => crate::Predicate::Not {
                child: Box::new(child.to_predicate()),
            },
            Self::Leaf(leaf) => crate::Predicate::Leaf(leaf.to_leaf()),
        }
    }

    /// Lower to the JSON string consumed by the scan layer.
    ///
    /// Format matches `serde_json::to_string(&crate::Predicate)` — by
    /// construction, because we go through the real type. The
    /// `serde_json::to_string` call is infallible for these inputs (no
    /// floats, no NaN, no maps with non-string keys), but we use
    /// `unwrap_or_else` with a hard-failure marker rather than `unwrap()`
    /// so a panic would surface a debuggable string in audit output
    /// instead of crashing the proc-macro.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.to_predicate()).unwrap_or_else(|e| {
            format!("{{\"kind\":\"leaf\",\"name\":\"_to_json_failed\",\"error\":\"{e}\"}}")
        })
    }

    /// Validate semantic invariants at parse time (mirrors
    /// [`crate::Predicate::validate`]).
    ///
    /// # Errors
    ///
    /// Returns [`syn::Error`] anchored at `span` if any of:
    /// - empty `all_of`/`any_of` combinator (R-A6)
    /// - nesting depth exceeds [`crate::predicate::MAX_PREDICATE_DEPTH`]
    ///   (NFA-11)
    /// - any leaf fails its per-leaf validation (NFA-7, NFA-8, NFA-9,
    ///   NFA-14, NFA-15)
    pub fn validate(&self, span: Span) -> syn::Result<()> {
        // Depth + node-count guard (mirrors crate::Predicate::validate).
        // NOTE: these run AFTER the parse tree is constructed; they do not prevent
        // a stack overflow during parse of a truly pathological input. In practice,
        // proc-macro stack depth is generous enough that parse-time overflow requires
        // thousands of nesting levels — far beyond any legitimate predicate.
        self.check_depth(0, span)?;
        self.validate_inner(span)
    }

    fn check_depth(&self, depth: usize, span: Span) -> syn::Result<()> {
        if depth > crate::predicate::MAX_PREDICATE_DEPTH {
            return Err(syn::Error::new(
                span,
                format!(
                    "requires: predicate nesting depth exceeds maximum of {}; \
                     deeply-nested predicates are rejected (mirrors antigen_attestation \
                     MAX_PREDICATE_DEPTH guard)",
                    crate::predicate::MAX_PREDICATE_DEPTH
                ),
            ));
        }
        match self {
            Self::AllOf(children) | Self::AnyOf(children) => {
                for c in children {
                    c.check_depth(depth + 1, span)?;
                }
                Ok(())
            }
            Self::Not(child) => child.check_depth(depth + 1, span),
            Self::Leaf(_) => Ok(()),
        }
    }

    fn validate_inner(&self, span: Span) -> syn::Result<()> {
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
                    child.validate_inner(span)?;
                }
                Ok(())
            }
            Self::Not(child) => child.validate_inner(span),
            Self::Leaf(leaf) => leaf.validate(span),
        }
    }
}

impl LeafExpr {
    /// Convert to the runtime [`crate::Leaf`]. Mirror of
    /// [`RequiresExpr::to_predicate`] at the leaf level.
    fn to_leaf(&self) -> crate::Leaf {
        match self {
            Self::RatifiedDoc {
                path,
                min_version,
                anchor,
                sibling_json,
            } => crate::Leaf::RatifiedDoc {
                path: path.as_ref().map(std::path::PathBuf::from),
                min_version: min_version.clone(),
                anchor: anchor.clone(),
                sibling_json: *sibling_json,
            },
            Self::Signers { required, against } => crate::Leaf::Signers {
                required: required.clone(),
                roles: std::collections::BTreeMap::new(),
                against: match against {
                    SignerCurrencyExpr::Current => crate::predicate::SignerCurrency::Current,
                    SignerCurrencyExpr::Any => crate::predicate::SignerCurrency::Any,
                },
                signature_allow: Vec::new(),
                signature_prefer: None,
            },
            Self::SignedTrailer { key, role, count } => crate::Leaf::SignedTrailer {
                key: key.clone(),
                role: role.clone(),
                count: *count,
            },
            Self::OraclesComplete { files } => crate::Leaf::OraclesComplete {
                files: files.iter().map(std::path::PathBuf::from).collect(),
            },
            Self::FreshWithinDays { days } => crate::Leaf::FreshWithinDays { days: *days },
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
            Self::RatifiedDoc {
                anchor: Some(a), ..
            } if a.is_empty() => Err(syn::Error::new(
                span,
                "requires: `ratified_doc(anchor = \"\")` is vacuously true (NFA-14); \
                 str::contains(\"\") always succeeds — specify a non-empty anchor string",
            )),
            Self::RatifiedDoc {
                min_version: Some(v),
                ..
            } if v.is_empty() => Err(syn::Error::new(
                span,
                "requires: `ratified_doc(min_version = \"\")` is vacuously true (NFA-15); \
                 any versioned doc passes an empty floor — specify a non-empty version string",
            )),
            _ => Ok(()),
        }
    }
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
                        format!("unknown signers field `{other}`; expected: required, against"),
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

// ============================================================================
// RequiresExpr round-trip tests
//
// rc.2 contract: `RequiresExpr::to_json()` always produces JSON that
// deserializes as `crate::Predicate`. The earlier rc.1 contract — hand-rolled
// JSON literals matching a hypothesized shape — produced wrong output and
// silently broke the substrate-witness pipeline.
//
// We assert two invariants:
// 1. Round-trip equality: `parse → to_predicate` and `parse → to_json →
//    from_str` produce structurally-equivalent Predicates.
// 2. Structural-shape spot checks: per-leaf checks that the JSON contains
//    the `"kind":"leaf"` discriminator and the `"name":"<leaf>"` tag at the
//    top level (not nested in a wrapper field — the rc.1 bug).
// ============================================================================

#[cfg(test)]
mod requires_json_tests {
    use super::*;
    use proc_macro2::TokenStream;

    fn parse_requires(input: &str) -> RequiresExpr {
        let tokens: TokenStream = input.parse().expect("tokenize");
        syn::parse2::<RequiresExpr>(tokens).expect("parse RequiresExpr")
    }

    fn round_trip(input: &str) -> crate::Predicate {
        let expr = parse_requires(input);
        let predicate_direct = expr.to_predicate();
        let json = expr.to_json();
        let predicate_via_json: crate::Predicate = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("JSON `{json}` from `{input}` did not deserialize: {e}"));
        assert_eq!(
            predicate_direct, predicate_via_json,
            "round-trip mismatch for `{input}`:\n  direct = {predicate_direct:?}\n  via_json = {predicate_via_json:?}"
        );
        predicate_via_json
    }

    #[test]
    fn leaf_fresh_within_days_positional_round_trips() {
        let p = round_trip("fresh_within_days(90)");
        assert!(matches!(
            p,
            crate::Predicate::Leaf(crate::Leaf::FreshWithinDays { days: 90 })
        ));
    }

    #[test]
    fn leaf_fresh_within_days_named_round_trips() {
        let p = round_trip("fresh_within_days(days = 180)");
        assert!(matches!(
            p,
            crate::Predicate::Leaf(crate::Leaf::FreshWithinDays { days: 180 })
        ));
    }

    #[test]
    fn leaf_signers_current_round_trips() {
        round_trip(r#"signers(required = ["alice", "bob"])"#);
    }

    #[test]
    fn leaf_signers_any_against_round_trips() {
        round_trip(r#"signers(required = ["alice"], against = "any")"#);
    }

    #[test]
    fn leaf_ratified_doc_bare_round_trips() {
        round_trip("ratified_doc");
    }

    #[test]
    fn leaf_ratified_doc_with_path_round_trips() {
        round_trip(r#"ratified_doc(path = "docs/discipline.md")"#);
    }

    #[test]
    fn leaf_ratified_doc_sibling_json_round_trips() {
        round_trip("ratified_doc(sibling_json = true)");
    }

    #[test]
    fn leaf_oracles_complete_round_trips() {
        round_trip(r#"oracles_complete(files = ["a.md", "b.md"])"#);
    }

    #[test]
    fn leaf_signed_trailer_default_count_round_trips() {
        round_trip(r#"signed_trailer(key = "Discipline-Verified-By")"#);
    }

    #[test]
    fn leaf_signed_trailer_non_default_count_round_trips() {
        round_trip(r#"signed_trailer(key = "Verified-By", count = 2)"#);
    }

    #[test]
    fn combinator_all_of_round_trips() {
        round_trip(r#"all_of([fresh_within_days(90), signers(required = ["alice"])])"#);
    }

    #[test]
    fn combinator_any_of_round_trips() {
        round_trip(r"any_of([fresh_within_days(30), fresh_within_days(90)])");
    }

    #[test]
    fn combinator_not_round_trips() {
        round_trip(r"not(fresh_within_days(90))");
    }

    #[test]
    fn json_shape_is_flat_not_nested() {
        // rc.1 bug regression: the macro emitted `{"kind":"leaf","leaf":{...}}`
        // but the real Predicate serde produces `{"kind":"leaf","name":"signers",...}`.
        // The deserializer rejected the wrapped form, so the audit always reported
        // sidecar-schema-invalid. This test pins the flat shape.
        let json = parse_requires(r#"signers(required = ["alice"])"#).to_json();
        assert!(
            json.contains(r#""kind":"leaf""#),
            "JSON must carry the predicate discriminator: {json}"
        );
        assert!(
            json.contains(r#""name":"signers""#),
            "leaf discriminator must be at the same level as kind, not nested: {json}"
        );
        assert!(
            !json.contains(r#""leaf":{"#),
            "rc.1 bug: leaf fields must NOT be wrapped in a `leaf` field: {json}"
        );
    }

    #[test]
    fn json_string_escapes_quotes_and_backslash() {
        // Signer names with special chars must be safely JSON-escaped via the
        // real serde path. This used to be a custom escape function; now we
        // rely on serde_json to handle it correctly.
        let expr = parse_requires(r#"signers(required = ["alice\"bob"])"#);
        let json = expr.to_json();
        let p: crate::Predicate = serde_json::from_str(&json).expect("round-trip ok");
        if let crate::Predicate::Leaf(crate::Leaf::Signers { required, .. }) = p {
            assert_eq!(required, vec!["alice\"bob".to_string()]);
        } else {
            panic!("expected Signers leaf");
        }
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
    fn requires_expr_validate_rejects_empty_anchor_nfa14() {
        let span = proc_macro2::Span::call_site();
        let expr = RequiresExpr::Leaf(LeafExpr::RatifiedDoc {
            path: Some("docs/d.md".to_string()),
            min_version: None,
            anchor: Some(String::new()), // empty anchor — NFA-14
            sibling_json: false,
        });
        assert!(expr.validate(span).is_err());
    }

    #[test]
    fn requires_expr_validate_rejects_empty_min_version_nfa15() {
        let span = proc_macro2::Span::call_site();
        let expr = RequiresExpr::Leaf(LeafExpr::RatifiedDoc {
            path: Some("docs/d.md".to_string()),
            min_version: Some(String::new()), // empty min_version — NFA-15
            anchor: None,
            sibling_json: false,
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

    #[test]
    fn json_round_trips_through_predicate_serde() {
        // The contract: every JSON we produce must deserialize as the runtime
        // Predicate type. This locks the two ASTs together — any divergence
        // (renamed variant, new field, changed serde tag) breaks the test.
        let cases = [
            r#"signers(required = ["math-researcher"])"#,
            r#"all_of([signers(required = ["alice"]), fresh_within_days(180)])"#,
            r"any_of([fresh_within_days(30), fresh_within_days(90)])",
            r"not(fresh_within_days(90))",
            r#"ratified_doc(path = "docs/discipline.md", min_version = "1.0")"#,
            r#"oracles_complete(files = ["a.md"])"#,
            r#"signed_trailer(key = "Verified-By", count = 2)"#,
        ];
        for input in cases {
            let json = parse_requires(input).to_json();
            let _: crate::Predicate = serde_json::from_str(&json).unwrap_or_else(|e| {
                panic!("JSON `{json}` from `{input}` failed Predicate round-trip: {e}")
            });
        }
    }
}

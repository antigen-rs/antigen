//! Path C parser for the fingerprint DSL.
//!
//! Per ADR-010 Amendment 1: NOT raw `syn::parse2::<syn::Expr>` — that path
//! cannot accept the DSL syntax. We hand-roll a `Parse` impl over
//! `syn::parse::ParseBuffer`, peek/parse machinery driving a
//! [`Constraint`]-shaped tree.

use proc_macro2::Span;
use quote::ToTokens;
use syn::bracketed;
use syn::ext::IdentExt;
use syn::parenthesized;
use syn::parse::ParseStream;
use syn::{Ident, LitInt, LitStr, Token};

use crate::{
    Constraint, GlobPattern, ItemKind, MAX_DEPTH, MAX_NODES, MethodPattern, QualifierKind,
    VariantRange,
};

/// Parse the top-level constraint list (comma-separated, optional trailing
/// comma). The caller wraps the result in [`crate::Fingerprint`] and runs
/// [`validate`] before returning.
pub fn parse_top_level(input: ParseStream) -> syn::Result<Vec<Constraint>> {
    let mut constraints = Vec::new();
    while !input.is_empty() {
        constraints.push(parse_constraint(input)?);
        if input.is_empty() {
            break;
        }
        input.parse::<Token![,]>()?;
    }
    if constraints.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "fingerprint must contain at least one constraint",
        ));
    }
    Ok(constraints)
}

fn parse_constraint(input: ParseStream) -> syn::Result<Constraint> {
    let lookahead_ident: Ident = input.fork().parse()?;
    let key = lookahead_ident.to_string();
    match key.as_str() {
        "item" => parse_item(input),
        "name" => parse_name(input),
        "variants" => parse_variants(input),
        "has_method" => parse_has_method(input),
        "attr_present" => parse_attr_present(input),
        "doc_contains" => parse_doc_contains(input),
        "body_contains_macro" => parse_body_contains_macro(input),
        "body_calls" => parse_body_calls(input),
        "is_async" | "is_unsafe" | "is_const" => parse_qualifier(input),
        "impl_of_trait" => parse_impl_of_trait(input),
        "derives" => parse_derives(input),
        "serde_arg" => parse_serde_arg(input),
        "all_of" => parse_all_of(input),
        "any_of" => parse_any_of(input),
        "not" => parse_not(input),
        other => Err(syn::Error::new(
            lookahead_ident.span(),
            format!(
                "unknown fingerprint operator `{other}`; expected one of: \
                 item, name, variants, has_method, attr_present, doc_contains, \
                 body_contains_macro, body_calls, is_async, is_unsafe, is_const, \
                 impl_of_trait, derives, serde_arg, all_of, any_of, not",
            ),
        )),
    }
}

fn parse_item(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "item"
    input.parse::<Token![=]>()?;
    // Item-kind keywords (`enum`, `struct`, `fn`, etc.) are reserved Rust
    // keywords; `Ident::parse` rejects them. `parse_any` accepts keywords.
    let kind_ident: Ident = Ident::parse_any(input)?;
    let kind_str = kind_ident.to_string();
    let kind = ItemKind::from_ident(&kind_str).ok_or_else(|| {
        syn::Error::new(
            kind_ident.span(),
            format!(
                "unknown item kind `{kind_str}`; expected one of: \
                 struct, enum, trait, fn, impl, type, mod"
            ),
        )
    })?;
    Ok(Constraint::Item(kind))
}

fn parse_name(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "name"
    input.parse::<Token![=]>()?;
    let matches_kw: Ident = input.parse()?;
    if matches_kw != "matches" {
        return Err(syn::Error::new(
            matches_kw.span(),
            "expected `matches(\"<glob>\")` after `name =`",
        ));
    }
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let pattern = lit.value();
    if pattern.is_empty() {
        return Err(syn::Error::new(
            lit.span(),
            "glob pattern must not be empty (use `*` to match any name)",
        ));
    }
    Ok(Constraint::NameMatches(GlobPattern(pattern)))
}

fn parse_variants(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "variants"
    input.parse::<Token![=]>()?;
    let min_lit: LitInt = input.parse()?;
    let min: usize = min_lit.base10_parse()?;
    input.parse::<Token![..=]>()?;
    let max_lit: LitInt = input.parse()?;
    let max: usize = max_lit.base10_parse()?;
    if max < min {
        return Err(syn::Error::new(
            max_lit.span(),
            format!("variants range upper bound {max} is below lower bound {min}"),
        ));
    }
    Ok(Constraint::Variants(VariantRange { min, max }))
}

fn parse_has_method(input: ParseStream) -> syn::Result<Constraint> {
    let kw: Ident = input.parse()?; // "has_method"
    let content;
    parenthesized!(content in input);
    let name_lit: LitStr = content.parse()?;
    let name = name_lit.value();
    if name.trim().is_empty() {
        return Err(syn::Error::new(
            name_lit.span(),
            "has_method name must not be empty or whitespace-only",
        ));
    }
    content.parse::<Token![,]>()?;
    let sig_lit: LitStr = content.parse()?;
    let signature = sig_lit.value();
    // Pre-parse the signature shape at load time. We accept any non-empty
    // string for v1; the matcher does textual signature comparison after
    // normalizing via syn. A hard syn-parse here would over-reject the
    // shorthand `(Self, Self) -> Self` which isn't a valid `syn::Signature`
    // by itself (no `fn name`).
    if signature.trim().is_empty() {
        return Err(syn::Error::new(
            sig_lit.span(),
            "has_method signature must not be empty",
        ));
    }
    let _ = kw; // silence unused-warning; the keyword position carries the diagnostic span via the lit.
    // ADR-010 Amendment 3 Performance Invariant 2: normalize the signature
    // pattern ONCE at parse time so the matcher does not re-normalize per
    // match site. This is the "pre-parsed signature" the invariant names.
    //
    // Canonicalization beyond whitespace collapse: route the user-provided
    // signature through proc_macro2's tokenizer so user-natural `&mut self`
    // matches the `& mut self` spacing the matcher produces when rendering
    // the actual `syn::Signature`. See `normalize_signature_canonical` for
    // the full rationale.
    //
    // Amendment 5 OQ1 STRICT: when proc_macro2 cannot tokenize the
    // signature string (unbalanced parens, unterminated string,
    // etc.), surface a fingerprint parse error rather than silently
    // falling back to plain `normalize_ws` (which would produce
    // asymmetric normalization vs the strict-tokenized match-site
    // path — exactly the spacing bug this canonicalization exists
    // to eliminate).
    let tokenized = crate::normalize_signature_canonical(&signature).ok_or_else(|| {
        syn::Error::new(
            sig_lit.span(),
            format!(
                "has_method signature `{signature}` is not a valid Rust token stream \
                 (unbalanced delimiters, unterminated string, or invalid character); \
                 the canonical form cannot be derived and matching cannot proceed"
            ),
        )
    })?;
    // Strip parameter names from the normalized pattern so that `(input: ParseStream)`
    // matches the same form as `(ParseStream)`. `render_inputs` in the matcher strips
    // names from the actual signature; we must do the same for the pattern to keep
    // comparison symmetric. Wrap in `fn __pat__` to parse as a syn::Signature, then
    // rebuild with only types (no ident: prefix). Falls back to raw tokenized form on
    // parse failure (e.g. shorthand `Self` types that aren't valid syn::Type alone).
    let normalized_signature = strip_param_names_in_sig_pattern(&tokenized).unwrap_or(tokenized);
    Ok(Constraint::HasMethod(MethodPattern {
        name,
        signature,
        normalized_signature: Some(normalized_signature),
    }))
}

fn parse_attr_present(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "attr_present"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let path = lit.value();
    if path.trim().is_empty() {
        return Err(syn::Error::new(
            lit.span(),
            "attr_present path must not be empty",
        ));
    }
    Ok(Constraint::AttrPresent(path))
}

fn parse_doc_contains(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "doc_contains"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let needle = lit.value();
    if needle.trim().is_empty() {
        return Err(syn::Error::new(
            lit.span(),
            "doc_contains substring must not be empty or whitespace-only",
        ));
    }
    Ok(Constraint::DocContains(needle))
}

/// Shared name-validation gate for the call/macro-target leaves
/// (`body_calls` + `body_contains_macro`).
///
/// **Fail-LOUD, never silent-miss.** Both leaves match against a single bare
/// identifier — `body_calls` against a call's LAST path segment / method ident,
/// `body_contains_macro` against a macro path's LAST segment. So a `name` that is
/// NOT a single identifier (`"std::process::exit"`, `".unwrap"`, `"panic!"`,
/// `"unwrap()"`, `" unwrap"`, `"unwrap "`) can never fire — the stored string
/// would equal no single ident → a fingerprint that *silently matches nothing*,
/// the exact named-but-silent (false-coverage) failure-class antigen exists to
/// surface. This gate rejects such names at PARSE time with a message that names
/// the fix, rather than shipping a no-op fingerprint. (DRY: one place to be
/// honest about names — both leaves route through here, and the shipped
/// `body_contains_macro` gets the same fix.)
///
/// `syn::parse_str::<Ident>` accepts exactly one identifier — Unicode XID idents
/// and raw idents (`r#fn`) included — and rejects paths, dots, parens, and `!`.
/// But `parse_str` tolerates *surrounding* whitespace (so `" unwrap"` would parse
/// as `Ident("unwrap")` while the stored name keeps its space → still a silent
/// miss), so we reject ANY whitespace outright BEFORE the ident-parse — the
/// stored name is then exactly what the matcher compares.
fn validate_target_ident_name(op: &str, name: &str, span: Span) -> syn::Result<()> {
    if name.trim().is_empty() {
        return Err(syn::Error::new(
            span,
            format!("{op} name must not be empty or whitespace-only"),
        ));
    }
    if name.contains(char::is_whitespace) || syn::parse_str::<Ident>(name).is_err() {
        return Err(syn::Error::new(
            span,
            format!(
                "{op}(\"{name}\") is not a single identifier; {op} matches against a bare \
                 last-segment / method / macro name, so a path-spelled (`a::b`), dotted, \
                 `!`-bearing, parenthesized, or whitespace-padded name would silently never \
                 fire (a named-but-silent miss). Use the bare name — e.g. \
                 `{op}(\"exit\")`, not `{op}(\"std::process::exit\")`."
            ),
        ));
    }
    Ok(())
}

fn parse_body_contains_macro(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "body_contains_macro"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let name = lit.value();
    validate_target_ident_name("body_contains_macro", &name, lit.span())?;
    Ok(Constraint::BodyContainsMacro(name))
}

fn parse_body_calls(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "body_calls"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let name = lit.value();
    validate_target_ident_name("body_calls", &name, lit.span())?;
    Ok(Constraint::BodyCalls(name))
}

/// Parse a value-less item-qualifier leaf (`is_async` / `is_unsafe` / `is_const`,
/// ADR-040 G1). These are BARE keywords — no `= <value>`, no `(...)` argument —
/// so parsing consumes just the keyword and maps it to its [`QualifierKind`].
fn parse_qualifier(input: ParseStream) -> syn::Result<Constraint> {
    let kw: Ident = input.parse()?;
    let kind = QualifierKind::from_ident(&kw.to_string()).ok_or_else(|| {
        // Unreachable in practice (the dispatch only routes the three known
        // keywords here), but keep the parser total rather than panicking.
        syn::Error::new(
            kw.span(),
            "unknown item qualifier; expected `is_async`, `is_unsafe`, or `is_const`",
        )
    })?;
    Ok(Constraint::Qualifier(kind))
}

fn parse_impl_of_trait(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "impl_of_trait"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let name = lit.value();
    // A trait name is matched against an impl's trait-path LAST segment — a bare
    // identifier — so the same well-formedness gate applies (a path-spelled
    // `"std::ops::Drop"` or padded name would silently never fire; fail loud).
    validate_target_ident_name("impl_of_trait", &name, lit.span())?;
    Ok(Constraint::ImplOfTrait(name))
}

fn parse_derives(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "derives"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let name = lit.value();
    // A derive name is matched against a literal derive-list ident (`Hash`),
    // so the same bare-identifier gate applies (fail loud on a padded/path name).
    validate_target_ident_name("derives", &name, lit.span())?;
    Ok(Constraint::Derives(name))
}

fn parse_serde_arg(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "serde_arg"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let name = lit.value();
    // A serde arg name is matched against a literal `#[serde(...)]` arg ident
    // (`deny_unknown_fields`), so the same bare-identifier gate applies.
    validate_target_ident_name("serde_arg", &name, lit.span())?;
    Ok(Constraint::SerdeArg(name))
}

fn parse_all_of(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?;
    let children = parse_paren_bracket_list(input)?;
    if children.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "all_of([...]) must contain at least one constraint",
        ));
    }
    Ok(Constraint::AllOf(children))
}

fn parse_any_of(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?;
    let children = parse_paren_bracket_list(input)?;
    if children.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "any_of([...]) must contain at least one constraint",
        ));
    }
    Ok(Constraint::AnyOf(children))
}

fn parse_not(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?;
    let content;
    parenthesized!(content in input);
    let inner = parse_constraint(&content)?;
    Ok(Constraint::Not(Box::new(inner)))
}

/// Parse `( [ c, c, c ] )` — the wrapper used by `all_of`/`any_of`.
fn parse_paren_bracket_list(input: ParseStream) -> syn::Result<Vec<Constraint>> {
    let outer;
    parenthesized!(outer in input);
    let inner;
    bracketed!(inner in outer);
    let mut out = Vec::new();
    while !inner.is_empty() {
        out.push(parse_constraint(&inner)?);
        if inner.is_empty() {
            break;
        }
        inner.parse::<Token![,]>()?;
    }
    Ok(out)
}

// ============================================================================
// Validation: depth + node-count caps + `not` placement
// ============================================================================

pub fn validate(fp: &crate::Fingerprint) -> syn::Result<()> {
    let mut nodes = 0usize;
    for c in &fp.constraints {
        // Top-level constraints are at depth 1.
        check_depth_and_count(c, 1, &mut nodes)?;
        check_not_placement(c, /* in_all_of_with_positive_sibling = */ false)?;
    }
    Ok(())
}

fn check_depth_and_count(c: &Constraint, depth: usize, nodes: &mut usize) -> syn::Result<()> {
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("fingerprint exceeds total node count ({MAX_NODES})"),
        ));
    }
    if depth > MAX_DEPTH {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("fingerprint exceeds maximum depth ({MAX_DEPTH})"),
        ));
    }
    match c {
        Constraint::AllOf(children) | Constraint::AnyOf(children) => {
            for child in children {
                check_depth_and_count(child, depth + 1, nodes)?;
            }
        },
        Constraint::Not(child) => {
            check_depth_and_count(child, depth + 1, nodes)?;
        },
        // Leaf constraints contribute one node (already counted).
        _ => {},
    }
    Ok(())
}

/// Per ADR-010 Amendment 3 OQ3: `not` is only legal inside `all_of`, AND
/// only as a sibling of at least one positive matcher. This closes the De
/// Morgan promiscuity loophole where `any_of([not(A), not(B)])` becomes
/// `not(all_of([A, B]))` and re-creates top-level negation.
fn check_not_placement(c: &Constraint, in_legal_all_of: bool) -> syn::Result<()> {
    match c {
        Constraint::Not(_) if !in_legal_all_of => Err(syn::Error::new(
            Span::call_site(),
            "`not` is only legal inside `all_of([...])` alongside at least one positive matcher \
             (per ADR-010 Amendment 3 OQ3)",
        )),
        Constraint::Not(child) => {
            // Recurse into the child with the legal-context flag reset (a
            // `not` inside a `not` is not in `all_of`).
            check_not_placement(child, false)
        },
        Constraint::AllOf(children) => {
            // Among children, at least one must be a positive matcher (not a
            // bare `not`). Then each child gets recursed with the all_of
            // legal-context flag set (so a `not` inside is OK at this level).
            let has_positive = children.iter().any(|c| !matches!(c, Constraint::Not(_)));
            if !has_positive {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "`all_of` containing only `not` children is rejected (per ADR-010 Amendment 3 \
                     OQ3 — at least one positive matcher required as a sibling)",
                ));
            }
            for child in children {
                check_not_placement(child, true)?;
            }
            Ok(())
        },
        Constraint::AnyOf(children) => {
            // `not` directly under `any_of` is rejected (the OQ3 De Morgan
            // loophole). check_not_placement on each child with the all_of
            // legal flag set to `false`.
            for child in children {
                check_not_placement(child, false)?;
            }
            Ok(())
        },
        _ => Ok(()),
    }
}

/// Strip parameter names from a `has_method` signature pattern so that user-written
/// `(input: ParseStream)` matches the same canonical form as `(ParseStream)`.
///
/// `render_inputs` in the matcher strips names when rendering the actual `syn::Signature`
/// (typed args emit only their type). The pattern must go through the same transformation
/// at parse time to keep comparison symmetric.
///
/// Strategy: wrap the pattern in `fn __pat__<placeholder>` + parse as `syn::Signature`,
/// then rebuild inputs using only types (no `ident :` prefix), matching `render_inputs`.
/// Falls back to `sig` unchanged if the wrapped string doesn't parse (e.g. shorthand
/// `Self` types, or pattern syntax that is valid tokens but not a valid sig — the caller
/// in `parse_has_method` then uses the raw tokenized form).
fn strip_param_names_in_sig_pattern(sig: &str) -> Option<String> {
    use std::str::FromStr;

    let wrapped = format!("fn __pat__{sig}");
    let tokens = proc_macro2::TokenStream::from_str(&wrapped).ok()?;
    let parsed: syn::Signature = syn::parse2(tokens).ok()?;

    let parts: Vec<String> = parsed
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Receiver(r) => r.to_token_stream().to_string(),
            syn::FnArg::Typed(pt) => pt.ty.to_token_stream().to_string(),
        })
        .collect();
    let inputs_rendered = parts.join(", ");
    let output_rendered = match &parsed.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
    };
    let rebuilt = if output_rendered.is_empty() {
        format!("({inputs_rendered})")
    } else {
        format!("({inputs_rendered}) -> {output_rendered}")
    };
    crate::normalize_signature_canonical(&rebuilt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Fingerprint;

    fn parse(s: &str) -> syn::Result<Fingerprint> {
        Fingerprint::parse(s)
    }

    #[test]
    fn parses_item_kind() {
        let fp = parse("item = enum").unwrap();
        assert_eq!(fp.constraints, vec![Constraint::Item(ItemKind::Enum)]);
    }

    #[test]
    fn parses_name_glob() {
        let fp = parse(r#"name = matches("*Class")"#).unwrap();
        assert_eq!(
            fp.constraints,
            vec![Constraint::NameMatches(GlobPattern("*Class".to_string()))]
        );
    }

    #[test]
    fn parses_variants_range() {
        let fp = parse("variants = 3..=8").unwrap();
        assert_eq!(
            fp.constraints,
            vec![Constraint::Variants(VariantRange { min: 3, max: 8 })]
        );
    }

    #[test]
    fn parses_has_method() {
        let fp = parse(r#"has_method("meet", "(Self, Self) -> Self")"#).unwrap();
        // PartialEq on MethodPattern is on (name, signature) — the
        // normalized cache is a derived field, not part of equality.
        assert_eq!(
            fp.constraints,
            vec![Constraint::HasMethod(MethodPattern {
                name: "meet".to_string(),
                signature: "(Self, Self) -> Self".to_string(),
                normalized_signature: None,
            })]
        );
        // PI-2 substrate check: the parser populated the cache.
        if let Constraint::HasMethod(p) = &fp.constraints[0] {
            assert!(
                p.normalized_signature.is_some(),
                "parser must populate normalized_signature at load time per ADR-010 Am3 PI-2",
            );
            // Canonical form post-proc_macro2-tokenization: punctuation
            // tokens (commas, ->) get whitespace inserted by proc_macro2's
            // token rendering. The matcher routes its own output through
            // the same canonicalization, so the comparison stays symmetric.
            assert_eq!(
                p.normalized_signature.as_deref(),
                Some("(Self , Self) -> Self"),
                "post-tokenization canonical form: proc_macro2 inserts space \
                 around `,` and other punctuation tokens; matcher applies the \
                 same canonicalization so comparison is symmetric",
            );
        }
    }

    #[test]
    fn has_method_normalize_collapses_whitespace_at_parse_time() {
        // Pattern with sloppy whitespace; the normalized cache should be
        // the canonical (proc_macro2-tokenized) form.
        let fp = parse(r#"has_method("meet", "(Self,   Self)  ->   Self")"#).unwrap();
        let Constraint::HasMethod(p) = &fp.constraints[0] else {
            panic!("expected HasMethod");
        };
        assert_eq!(
            p.normalized_signature.as_deref(),
            Some("(Self , Self) -> Self"),
        );
    }

    #[test]
    fn parses_attr_present() {
        let fp = parse(r#"attr_present("repr")"#).unwrap();
        assert_eq!(
            fp.constraints,
            vec![Constraint::AttrPresent("repr".to_string())]
        );
    }

    #[test]
    fn parses_doc_contains() {
        let fp = parse(r#"doc_contains("strength")"#).unwrap();
        assert_eq!(
            fp.constraints,
            vec![Constraint::DocContains("strength".to_string())]
        );
    }

    #[test]
    fn parses_body_contains_macro() {
        let fp = parse(r#"body_contains_macro("panic")"#).unwrap();
        assert_eq!(
            fp.constraints,
            vec![Constraint::BodyContainsMacro("panic".to_string())]
        );
    }

    #[test]
    fn parses_body_calls_bare_ident() {
        // A single bare identifier parses — including Unicode XID + raw idents.
        for ok in ["unwrap", "exit", "名前", "r#fn"] {
            let src = format!(r#"body_calls("{ok}")"#);
            let fp = parse(&src).unwrap_or_else(|e| panic!("body_calls({ok:?}) must parse: {e}"));
            assert_eq!(fp.constraints, vec![Constraint::BodyCalls(ok.to_string())]);
        }
    }

    #[test]
    fn rejects_body_calls_non_ident_name_loudly() {
        // The well-formedness gate: a name the matcher can never fire on (because
        // it matches by last segment / method ident) is rejected at PARSE time —
        // fail-loud, not a silent never-fires miss (the named-but-silent class
        // antigen exists to surface). Empty/whitespace + path/dotted/parenthesized
        // /padded names all ERR.
        for bad in [
            "",                   // empty
            "   ",                // whitespace-only
            "std::process::exit", // path — use the last segment "exit"
            ".unwrap",            // leading dot
            "unwrap()",           // parens
            " unwrap",            // leading space
            "unwrap ",            // trailing space
            "a b",                // internal space
        ] {
            let src = format!(r#"body_calls("{bad}")"#);
            assert!(
                parse(&src).is_err(),
                "body_calls({bad:?}) must be REJECTED at parse — the matcher matches by \
                 last-segment/method-ident, so this name would silently never fire (a \
                 named-but-silent miss). It must fail loud, not ship a no-op fingerprint."
            );
        }
    }

    #[test]
    fn rejects_body_contains_macro_non_ident_name_loudly() {
        // The SHIPPED twin gets the same fail-loud gate:
        // body_contains_macro matches a macro path's LAST segment, so a path/`!`/
        // dotted/padded name silently never fires — the same named-but-silent
        // class. The shared `validate_target_ident_name` gate closes it for both
        // leaves. Bare names (the only shapes any real fingerprint uses — `panic`,
        // `unreachable`, `todo`, `unimplemented`, `recurse_marker`) still parse.
        assert!(parse(r#"body_contains_macro("panic")"#).is_ok());
        for bad in [
            "",
            "  ",
            "std::panic", // path — use the last segment "panic"
            "panic!",     // the `!` is not part of the macro NAME
            "panic ",     // trailing space
            " panic",     // leading space
            "panic()",    // parens
        ] {
            let src = format!(r#"body_contains_macro("{bad}")"#);
            assert!(
                parse(&src).is_err(),
                "body_contains_macro({bad:?}) must now be REJECTED at parse (a deliberate \
                 fail-direction fix to the shipped leaf — a path/!/padded macro name would \
                 silently never fire). Bare `panic` still parses."
            );
        }
    }

    #[test]
    fn parses_all_of() {
        let fp = parse(r#"all_of([attr_present("repr"), doc_contains("x")])"#).unwrap();
        match &fp.constraints[..] {
            [Constraint::AllOf(children)] => assert_eq!(children.len(), 2),
            other => panic!("expected single AllOf, got {other:?}"),
        }
    }

    #[test]
    fn parses_any_of() {
        let fp = parse("any_of([item = struct, item = enum])").unwrap();
        match &fp.constraints[..] {
            [Constraint::AnyOf(children)] => assert_eq!(children.len(), 2),
            other => panic!("expected single AnyOf, got {other:?}"),
        }
    }

    #[test]
    fn parses_not_inside_all_of() {
        let fp = parse(r#"all_of([item = enum, not(name = matches("Test*"))])"#).unwrap();
        match &fp.constraints[..] {
            [Constraint::AllOf(children)] => assert_eq!(children.len(), 2),
            other => panic!("expected AllOf, got {other:?}"),
        }
    }

    #[test]
    fn rejects_top_level_not() {
        let err = parse(r"not(item = enum)").unwrap_err().to_string();
        assert!(err.contains("not"), "got: {err}");
    }

    #[test]
    fn rejects_not_inside_any_of() {
        let err = parse(r"any_of([not(item = enum), item = struct])")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not"), "got: {err}");
    }

    #[test]
    fn rejects_all_of_with_only_not_children() {
        let err = parse(r"all_of([not(item = enum), not(item = struct)])")
            .unwrap_err()
            .to_string();
        assert!(err.contains("positive"), "got: {err}");
    }

    #[test]
    fn parses_multi_constraint_top_level() {
        let fp = parse(r#"item = enum, name = matches("*Class"), variants = 3..=8"#).unwrap();
        assert_eq!(fp.constraints.len(), 3);
    }

    #[test]
    fn rejects_unknown_operator() {
        let err = parse(r#"frobnicate("x")"#).unwrap_err().to_string();
        assert!(err.contains("unknown fingerprint operator"));
    }

    #[test]
    fn rejects_empty_glob() {
        let err = parse(r#"name = matches("")"#).unwrap_err().to_string();
        assert!(err.contains("empty"));
    }

    #[test]
    fn rejects_inverted_variant_range() {
        let err = parse("variants = 5..=2").unwrap_err().to_string();
        assert!(err.contains("below"));
    }

    #[test]
    fn rejects_unknown_item_kind() {
        let err = parse("item = wibble").unwrap_err().to_string();
        assert!(err.contains("unknown item kind"));
    }

    #[test]
    fn rejects_empty_fingerprint() {
        let err = parse("").unwrap_err().to_string();
        assert!(err.contains("at least one constraint"));
    }

    #[test]
    fn enforces_max_depth() {
        // Build all_of([all_of([all_of([...])])]) up to MAX_DEPTH+2 — should reject.
        let mut s = String::from("item = enum");
        for _ in 0..MAX_DEPTH + 2 {
            s = format!("all_of([{s}])");
        }
        let err = parse(&s).unwrap_err().to_string();
        assert!(err.contains("depth"), "got: {err}");
    }

    // ATK-FP-MAX-NODES: a wide all_of with MAX_NODES+1 leaves must be rejected.
    // Attack: instead of DEEP nesting (which hits MAX_DEPTH), use WIDE all_of to
    // create many nodes at the same depth — bypasses MAX_DEPTH but hits MAX_NODES.
    #[test]
    fn enforces_max_nodes() {
        // MAX_NODES=256. Build all_of([item=struct, item=struct, ...]) with 260 leaves.
        // Each leaf is 1 node; the all_of wrapper is 1 node; total = 261 > 256.
        let leaves: Vec<String> = (0..260).map(|_| "item = struct".to_string()).collect();
        let s = format!("all_of([{}])", leaves.join(", "));
        let err = parse(&s).unwrap_err().to_string();
        assert!(
            err.contains("node"),
            "ATK-FP-MAX-NODES: fingerprint with 260 all_of leaves must hit MAX_NODES limit. \
             Got: {err}"
        );
    }

    // ATK-FP-MAX-NODES-BOUNDARY: exactly MAX_NODES nodes must be accepted.
    // Verify the limit is exclusive (> MAX_NODES rejected, == MAX_NODES allowed).
    // MAX_NODES=256: the root has 1 node (the all_of), plus 255 leaves = 256 total.
    #[test]
    fn accepts_exactly_max_nodes() {
        // all_of with 255 leaves: 1 (all_of node) + 255 (leaves) = 256 = MAX_NODES.
        let leaves: Vec<String> = (0..255).map(|_| "item = struct".to_string()).collect();
        let s = format!("all_of([{}])", leaves.join(", "));
        // This should PARSE successfully (exactly at the limit, not over).
        assert!(
            parse(&s).is_ok(),
            "ATK-FP-MAX-NODES-BOUNDARY: exactly {MAX_NODES} nodes must be accepted. \
             all_of with 255 leaves = 256 total nodes."
        );
    }

    #[test]
    fn node_kind_dispatch_top_level() {
        let fp = parse("item = enum, name = matches(\"*Class\")").unwrap();
        assert_eq!(fp.node_kind(), Some(ItemKind::Enum));
    }

    #[test]
    fn node_kind_inside_all_of() {
        let fp = parse(r#"all_of([item = struct, attr_present("repr")])"#).unwrap();
        assert_eq!(fp.node_kind(), Some(ItemKind::Struct));
    }

    #[test]
    fn node_kind_none_when_unconstrained() {
        let fp = parse(r#"name = matches("*")"#).unwrap();
        assert_eq!(fp.node_kind(), None);
    }

    #[test]
    fn rejects_double_negation_not_inside_not() {
        // ATK-FP-DOUBLE-NOT: not(not(X)) is equivalent to X but the ADR requires
        // explicit positive form. Per check_not_placement line 342: not inside not
        // is NOT in legal all_of context — rejected.
        let err = parse(r"all_of([not(not(item = enum)), item = struct])")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("not"),
            "ATK-FP-DOUBLE-NOT: not(not(item = enum)) inside all_of should be rejected. Got: {err}"
        );
    }

    #[test]
    fn rejects_whitespace_only_has_method_name() {
        // ATK-FP-WHITESPACE-HM: has_method name guard uses is_empty() (line 129)
        // not trim().is_empty() — a whitespace-only method name " " would be accepted.
        // Consistent with other parsers (attr_present, body_contains_macro use trim()).
        // Practically: matching a method named " " would never fire (no such method
        // exists), making this a silent miss rather than a false positive. Still wrong.
        //
        // NOTE: This test verifies the inconsistency exists. If it PASSES, the fix
        // landed. If it fails with "called unwrap_err on Ok", the gap is real.
        let result = parse(r#"has_method(" ", "() -> ()")"#);
        assert!(
            result.is_err(),
            "ATK-FP-WHITESPACE-HM: has_method(' ', '() -> ()') with whitespace-only name \
             should be rejected (consistent with attr_present/body_contains_macro which use \
             trim().is_empty()). parse_has_method line 129 uses is_empty() not trim().is_empty(). \
             Accepted: {:?}",
            result.ok()
        );
    }

    #[test]
    fn rejects_whitespace_only_doc_contains() {
        // ATK-FP-WHITESPACE-DOC: a whitespace-only doc_contains pattern (e.g.,
        // " " or "\t") passes the non-empty check but acts as a near-universal
        // matcher — most doc strings contain spaces, so doc_contains(" ") matches
        // almost every struct/fn with a doc comment. This is an adversarial input
        // that produces a fingerprint with effectively zero specificity, silently
        // raising false positives for every site that has any doc comment.
        //
        // The parser accepts empty-string and rejects it (line 212). It must also
        // reject strings that are all-whitespace after trim().
        let err_space = parse(r#"doc_contains(" ")"#).unwrap_err().to_string();
        assert!(
            err_space.contains("empty") || err_space.contains("whitespace"),
            "ATK-FP-WHITESPACE-DOC: doc_contains(' ') (single space) must be rejected. \
             A whitespace-only needle matches any doc string with a space, making the \
             fingerprint a near-universal matcher with zero specificity. \
             Parser should reject needles that are all-whitespace (trim().is_empty()). \
             Got error: {err_space}"
        );
        let err_tab = parse(r#"doc_contains("\t")"#).unwrap_err().to_string();
        assert!(
            err_tab.contains("empty") || err_tab.contains("whitespace"),
            "ATK-FP-WHITESPACE-DOC: doc_contains('\\t') (tab) must also be rejected. \
             Got error: {err_tab}"
        );
    }

    #[test]
    fn rejects_empty_all_of() {
        // ATK-FP-EMPTY-ALL-OF: all_of([]) would vacuously match EVERY item
        // (empty iterator + .all(...) = true), flooding synthesis with false
        // positives for every struct/fn/impl in the workspace.
        let err = parse("all_of([])").unwrap_err().to_string();
        assert!(
            err.contains("at least one") || err.contains("empty"),
            "ATK-FP-EMPTY-ALL-OF: all_of([]) must be rejected — vacuously \
             matches every item. Got: {err}"
        );
    }

    #[test]
    fn rejects_empty_any_of() {
        // ATK-FP-EMPTY-ANY-OF: any_of([]) would vacuously fail for EVERY item
        // (empty iterator + .any(...) = false), producing a fingerprint that
        // never fires — a silent always-false matcher with no diagnostic.
        let err = parse("any_of([])").unwrap_err().to_string();
        assert!(
            err.contains("at least one") || err.contains("empty"),
            "ATK-FP-EMPTY-ANY-OF: any_of([]) must be rejected — vacuously \
             fails for every item, producing a dead fingerprint. Got: {err}"
        );
    }
}

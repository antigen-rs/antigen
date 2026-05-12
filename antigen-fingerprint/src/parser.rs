//! Path C parser for the fingerprint DSL.
//!
//! Per ADR-010 Amendment 1: NOT raw `syn::parse2::<syn::Expr>` — that path
//! cannot accept the DSL syntax. We hand-roll a `Parse` impl over
//! `syn::parse::ParseBuffer`, peek/parse machinery driving a
//! [`Constraint`]-shaped tree.

use proc_macro2::Span;
use syn::bracketed;
use syn::ext::IdentExt;
use syn::parenthesized;
use syn::parse::ParseStream;
use syn::{Ident, LitInt, LitStr, Token};

use crate::{Constraint, GlobPattern, ItemKind, MethodPattern, VariantRange, MAX_DEPTH, MAX_NODES};

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
        "all_of" => parse_all_of(input),
        "any_of" => parse_any_of(input),
        "not" => parse_not(input),
        other => Err(syn::Error::new(
            lookahead_ident.span(),
            format!(
                "unknown fingerprint operator `{other}`; expected one of: \
                 item, name, variants, has_method, attr_present, doc_contains, \
                 body_contains_macro, all_of, any_of, not",
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
    if name.is_empty() {
        return Err(syn::Error::new(
            name_lit.span(),
            "has_method name must not be empty",
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
                // the actual `syn::Signature`. A3.5 onboarding sweep — see
                // `normalize_signature_canonical` for the full rationale.
                //
                // Amendment 5 OQ1 STRICT: when proc_macro2 cannot tokenize the
                // signature string (unbalanced parens, unterminated string,
                // etc.), surface a fingerprint parse error rather than silently
                // falling back to plain `normalize_ws` (which would produce
                // asymmetric normalization vs the strict-tokenized match-site
                // path — exactly the spacing bug this canonicalization exists
                // to eliminate).
    let normalized_signature =
        crate::normalize_signature_canonical(&signature).ok_or_else(|| {
            syn::Error::new(
                sig_lit.span(),
                format!(
                    "has_method signature `{signature}` is not a valid Rust token stream \
                 (unbalanced delimiters, unterminated string, or invalid character); \
                 the canonical form cannot be derived and matching cannot proceed"
                ),
            )
        })?;
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
    if needle.is_empty() {
        return Err(syn::Error::new(
            lit.span(),
            "doc_contains substring must not be empty",
        ));
    }
    Ok(Constraint::DocContains(needle))
}

fn parse_body_contains_macro(input: ParseStream) -> syn::Result<Constraint> {
    let _kw: Ident = input.parse()?; // "body_contains_macro"
    let content;
    parenthesized!(content in input);
    let lit: LitStr = content.parse()?;
    let name = lit.value();
    if name.trim().is_empty() {
        return Err(syn::Error::new(
            lit.span(),
            "body_contains_macro name must not be empty",
        ));
    }
    Ok(Constraint::BodyContainsMacro(name))
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
        }
        Constraint::Not(child) => {
            check_depth_and_count(child, depth + 1, nodes)?;
        }
        // Leaf constraints contribute one node (already counted).
        _ => {}
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
        }
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
        }
        Constraint::AnyOf(children) => {
            // `not` directly under `any_of` is rejected (the OQ3 De Morgan
            // loophole). check_not_placement on each child with the all_of
            // legal flag set to `false`.
            for child in children {
                check_not_placement(child, false)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
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
            // A3.5 onboarding sweep canonicalization upgrade.
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
}

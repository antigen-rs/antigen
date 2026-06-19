//! The Fingerprint → DSL serializer: the parser's exact inverse (ADR-063).
//!
//! `antigen-fingerprint` ships [`Fingerprint::parse`] (`&str` → AST). This
//! module ships the inverse half — the AST → DSL-text rendering — completing
//! the bijection the type claims. The DSL is the **privileged** rendering of
//! the three the `Constraint` AST has (serde-JSON, `Debug`, DSL): it is the
//! UNIQUE one that is ALSO the parser's input grammar, so it is the only
//! rendering for which `parse ∘ serialize` can be the identity, and therefore
//! the only co-native one (ADR-058) — the same text a human reads/edits/pastes
//! is the same text the parser consumes is the same text the `#[antigen(…)]`
//! macro compiles.
//!
//! ## The round-trip type-law (ADR-063 — the executable correctness contract)
//!
//! > For every parser-producible `fp`: `Fingerprint::parse(fp.to_string()) ==
//! > Ok(fp)`.
//!
//! [`Display`](std::fmt::Display) emits the comma-joined top-level constraint list (NO
//! surrounding `#[antigen(…)]`) — exactly the inner form the parser's top-level entry
//! (`parser::parse_top_level`) reads.
//! [`to_antigen_attr`] wraps that in `#[antigen(fingerprint = r#"…"#)]` for the
//! scaffold consumer.
//!
//! ## Completeness is a compiler guarantee, not a test hope (ADR-063 T6)
//!
//! The [`Constraint`] alphabet is CLOSED (ADR-047 Amendment 3). The `match` in
//! [`Constraint`]'s `Display` is therefore **EXHAUSTIVE with no wildcard `_`
//! arm** — a new operator added to the grammar fails to compile this module
//! until an arm is written. This turns serializer-completeness from a
//! coverage-hope into a compiler guarantee. **Adding a wildcard `_` arm
//! re-opens the silent-variant-drop class and is a rejectable finding against
//! ADR-063** (the standing invariant joining the closed-alphabet family —
//! ADR-047 Amd3, ADR-051 Amd1).
//!
//! ## The three hazards this module defends (ADR-063 §3)
//!
//! - **(a) String escaping** — string-payload leaves sit inside a `LitStr` the
//!   parser reads. We COMPOSE `proc_macro2::Literal::string` (ADR-064: compose
//!   the witness/IO layer) to render an exact Rust string-literal body rather
//!   than hand-rolling an escaper. See [`render_str_lit`].
//! - **(b) `HasMethod` raw signature** — `MethodPattern: PartialEq` compares
//!   the RAW `signature`, NOT the `normalized_signature` perf cache
//!   (`lib.rs:395`). We emit the raw `signature`; emitting normalized would
//!   silently break round-trip by the type's own equality law.
//! - **(c) Scoped domain** — the round-trip law holds for every
//!   parser-producible fp; a hand-built fp that violates parser well-formedness
//!   (empty glob, empty `all_of`, misplaced `not`) serializes faithfully to a
//!   form the parser then correctly REJECTS. The serializer never silently
//!   repairs an invalid fp; it emits what the AST says and lets `parse` be the
//!   judge.

use std::fmt;

use crate::{Constraint, Fingerprint};

impl fmt::Display for Fingerprint {
    /// Emit the comma-joined top-level constraint list — the exact inner form
    /// `parse_top_level` consumes (NO surrounding
    /// `#[antigen(…)]`). This is the round-trip oracle's target: `parse` reads
    /// this list back into the same AST.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for c in &self.constraints {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            fmt::Display::fmt(c, f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Constraint {
    /// Emit a single constraint in the parser's exact surface form.
    ///
    /// EXHAUSTIVE no-wildcard match over **every** `Constraint` variant — the
    /// closed-alphabet completeness guarantee (ADR-063 T6). Do NOT add a `_`
    /// arm: a new variant MUST force a compile error here until its inverse is
    /// written.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // `item = <kind>` — BARE keyword via the shipped reverse map
            // (`ItemKind::keyword`, the inverse of `ItemKind::from_ident`).
            Self::Item(kind) => write!(f, "item = {}", kind.keyword()),

            // `name = matches("<glob>")` — the glob is a string payload.
            Self::NameMatches(glob) => {
                write!(f, "name = matches({})", render_str_lit(&glob.0))
            },

            // `variants = M..=N` — bare ints with the inclusive-range token.
            Self::Variants(range) => write!(f, "variants = {}..={}", range.min, range.max),

            // `has_method("<name>", "<signature>")` — BOTH payloads are strings;
            // emit the RAW `signature` (hazard b — `PartialEq` compares raw,
            // NOT `normalized_signature`).
            Self::HasMethod(pat) => write!(
                f,
                "has_method({}, {})",
                render_str_lit(&pat.name),
                render_str_lit(&pat.signature),
            ),

            // The bare-string-payload leaves: `op("<payload>")`.
            Self::AttrPresent(s) => write!(f, "attr_present({})", render_str_lit(s)),
            Self::DocContains(s) => write!(f, "doc_contains({})", render_str_lit(s)),
            Self::BodyContainsMacro(s) => {
                write!(f, "body_contains_macro({})", render_str_lit(s))
            },
            Self::BodyCalls(s) => write!(f, "body_calls({})", render_str_lit(s)),
            Self::ImplOfTrait(s) => write!(f, "impl_of_trait({})", render_str_lit(s)),
            Self::Derives(s) => write!(f, "derives({})", render_str_lit(s)),
            Self::SerdeArg(s) => write!(f, "serde_arg({})", render_str_lit(s)),

            // `is_async` / `is_unsafe` / `is_const` — BARE keyword, NO parens,
            // via the shipped reverse map (`QualifierKind::keyword`).
            Self::Qualifier(kind) => f.write_str(kind.keyword()),

            // The recursive combinators. `all_of`/`any_of` take a
            // bracketed, comma-joined child list; `not` takes a single child.
            Self::AllOf(children) => write_combinator_list(f, "all_of", children),
            Self::AnyOf(children) => write_combinator_list(f, "any_of", children),
            Self::Not(inner) => write!(f, "not({inner})"),
        }
    }
}

/// Render a string payload as a valid Rust string-literal body — quotes
/// included — with exact Rust-literal escaping (hazard a).
///
/// COMPOSES `proc_macro2::Literal::string` (ADR-064 — compose the witness/IO
/// layer at the leaf) rather than hand-rolling an escaper: `Literal::string`
/// produces precisely the escaping `syn::LitStr` reads back, so `\`, `"`,
/// control chars, and Unicode are all handled by the same tokenizer the parser
/// trusts. The returned string already carries its surrounding `"`.
fn render_str_lit(payload: &str) -> String {
    proc_macro2::Literal::string(payload).to_string()
}

/// Render a combinator: `<op>([<child>, <child>, …])`.
///
/// The bracketed list is the `parse_paren_bracket_list` inverse. An empty
/// `children` (only reachable on a hand-built, non-parser-producible fp)
/// renders `<op>([])`, which the parser then correctly REJECTS — the faithful
/// scoped-domain behavior (hazard c), never a silent repair.
fn write_combinator_list(
    f: &mut fmt::Formatter<'_>,
    op: &str,
    children: &[Constraint],
) -> fmt::Result {
    write!(f, "{op}([")?;
    let mut first = true;
    for c in children {
        if !first {
            f.write_str(", ")?;
        }
        first = false;
        fmt::Display::fmt(c, f)?;
    }
    f.write_str("])")
}

/// Wrap a fingerprint's DSL rendering in the `#[antigen(fingerprint = r#"…"#)]`
/// attribute form the scaffold consumer pastes above an item (ADR-063 §2 — the
/// cosmetic attribute wrapper).
///
/// The inner DSL is [`Fingerprint`]'s [`fmt::Display`]; this only adds the
/// macro-attribute envelope. Because the inner text round-trips
/// (`parse(serialize(fp)) == fp`), the pasted attribute parses back to the SAME
/// fingerprint the tool computed — paste-and-compile, not paste-and-rewrite.
///
/// A raw string (`r#"…"#`) is used so the DSL's own `"` (from string-payload
/// leaves) need no further escaping inside the attribute envelope.
#[must_use]
pub fn to_antigen_attr(fp: &Fingerprint) -> String {
    format!("#[antigen(fingerprint = r#\"{fp}\"#)]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GlobPattern, ItemKind, MethodPattern, QualifierKind, VariantRange};

    /// The round-trip oracle: parse → serialize → parse must yield the SAME
    /// AST. Targets [`Fingerprint::parse`] (ADR-063 T1 — the parser is the
    /// grammar; the serializer is sound only relative to it).
    fn assert_round_trips(dsl: &str) {
        let fp = Fingerprint::parse(dsl)
            .unwrap_or_else(|e| panic!("fixture DSL must parse: {dsl:?}: {e}"));
        let serialized = fp.to_string();
        let reparsed = Fingerprint::parse(&serialized).unwrap_or_else(|e| {
            panic!("serialized form must re-parse: {serialized:?} (from {dsl:?}): {e}")
        });
        assert_eq!(
            fp, reparsed,
            "round-trip diverged:\n  original DSL: {dsl}\n  serialized:   {serialized}",
        );
    }

    // ── §4.1 Per-variant matrix (ADR-063) ──────────────────────────────────
    // One fixture per Constraint variant → parse → serialize → parse → equal.
    // The readable enumeration of the contract; the bare-form rows catch the
    // quote/paren traps.

    #[test]
    fn round_trip_item() {
        // Every ItemKind keyword, bare (no quotes).
        for kw in [
            "struct", "enum", "trait", "fn", "impl", "type", "mod", "const", "static", "union",
        ] {
            assert_round_trips(&format!("item = {kw}"));
        }
    }

    #[test]
    fn round_trip_name_matches() {
        assert_round_trips(r#"name = matches("*Class")"#);
        assert_round_trips(r#"name = matches("?foo*")"#);
    }

    #[test]
    fn round_trip_variants() {
        assert_round_trips("variants = 3..=8");
        assert_round_trips("variants = 0..=0");
    }

    #[test]
    fn round_trip_has_method() {
        assert_round_trips(r#"has_method("meet", "(Self, Self) -> Self")"#);
    }

    #[test]
    fn round_trip_string_leaves() {
        assert_round_trips(r#"attr_present("repr")"#);
        assert_round_trips(r#"doc_contains("strength")"#);
        assert_round_trips(r#"body_contains_macro("panic")"#);
        assert_round_trips(r#"body_calls("unwrap")"#);
        assert_round_trips(r#"impl_of_trait("Drop")"#);
        assert_round_trips(r#"derives("Hash")"#);
        assert_round_trips(r#"serde_arg("deny_unknown_fields")"#);
    }

    #[test]
    fn round_trip_qualifiers() {
        // Bare keyword, NO parens.
        assert_round_trips("is_async");
        assert_round_trips("is_unsafe");
        assert_round_trips("is_const");
    }

    #[test]
    fn round_trip_combinators() {
        assert_round_trips(r#"all_of([attr_present("repr"), doc_contains("x")])"#);
        assert_round_trips("any_of([item = struct, item = enum])");
        // `not` only legal inside `all_of` alongside a positive sibling.
        assert_round_trips(r#"all_of([item = enum, not(name = matches("Test*"))])"#);
    }

    #[test]
    fn round_trip_multi_constraint_top_level() {
        assert_round_trips(r#"item = enum, name = matches("*Class"), variants = 3..=8"#);
    }

    // ── §4.2 Nested/composite (ADR-063) ────────────────────────────────────
    // A deep fixture exercising the recursive serializer + the only-legal Not
    // position (under all_of, with a positive sibling).

    #[test]
    fn round_trip_deep_nested() {
        assert_round_trips(
            r#"all_of([item = struct, derives("Hash"), not(derives("Eq")), any_of([body_calls("unwrap"), body_calls("expect")])])"#,
        );
    }

    // ── §4.3 Escaping (ADR-063 hazard a — the load-bearing one) ─────────────

    #[test]
    fn round_trip_escaping_quote_backslash_newline() {
        // A doc_contains payload with `"`, `\`, and a newline — the named
        // hazard-(a) fixture. Build it from the AST directly so the payload is
        // exactly those bytes (a DSL fixture would need its own escaping).
        let fp = Fingerprint {
            constraints: vec![Constraint::DocContains("a\"b\\c\nd".to_string())],
        };
        let serialized = fp.to_string();
        let reparsed = Fingerprint::parse(&serialized)
            .unwrap_or_else(|e| panic!("escaped form must re-parse: {serialized:?}: {e}"));
        assert_eq!(fp, reparsed, "escaping round-trip diverged: {serialized}");
    }

    #[test]
    fn render_str_lit_escapes_via_proc_macro2() {
        // The composed escaper renders a valid quoted Rust string-literal body.
        assert_eq!(render_str_lit("plain"), "\"plain\"");
        assert_eq!(render_str_lit("a\"b"), "\"a\\\"b\"");
        assert_eq!(render_str_lit("a\\b"), "\"a\\\\b\"");
    }

    // ── §4.3b Hazard b — HasMethod emits RAW signature, not normalized ──────

    #[test]
    fn has_method_emits_raw_signature_not_normalized() {
        // Non-canonical spacing: the parser stores the RAW `signature` as
        // written and a normalized cache separately. The serializer MUST emit
        // the raw form (PartialEq compares raw) — emitting normalized would
        // diverge the round-tripped signature from the original.
        let raw = "(&  mut  self,  T)  ->  U";
        let fp = Fingerprint {
            constraints: vec![Constraint::HasMethod(MethodPattern {
                name: "f".to_string(),
                signature: raw.to_string(),
                normalized_signature: Some("(& mut self , T) -> U".to_string()),
            })],
        };
        let serialized = fp.to_string();
        assert!(
            serialized.contains(raw),
            "serializer must emit the RAW signature {raw:?}, got: {serialized}",
        );
        let reparsed = Fingerprint::parse(&serialized)
            .unwrap_or_else(|e| panic!("raw-sig form must re-parse: {serialized:?}: {e}"));
        assert_eq!(fp, reparsed, "raw-signature round-trip diverged");
    }

    // ── §4.3c Hazard c — scoped domain: invalid fp serializes, parser ERRORS ─

    #[test]
    fn invalid_fp_serializes_then_parser_rejects() {
        // A hand-built fp with an empty all_of is NOT parser-producible. The
        // serializer faithfully emits `all_of([])`; parse then correctly ERRORS
        // — never a silent repair (ADR-063 hazard c).
        let fp = Fingerprint {
            constraints: vec![Constraint::AllOf(vec![])],
        };
        let serialized = fp.to_string();
        assert_eq!(serialized, "all_of([])");
        assert!(
            Fingerprint::parse(&serialized).is_err(),
            "an empty all_of must be REJECTED on re-parse (scoped domain), got Ok",
        );
    }

    // ── to_antigen_attr (the cosmetic wrapper) ──────────────────────────────

    #[test]
    fn to_antigen_attr_wraps_and_round_trips_inner() {
        let fp = Fingerprint {
            constraints: vec![
                Constraint::Item(ItemKind::Enum),
                Constraint::Variants(VariantRange { min: 3, max: 8 }),
            ],
        };
        let attr = to_antigen_attr(&fp);
        assert_eq!(
            attr,
            "#[antigen(fingerprint = r#\"item = enum, variants = 3..=8\"#)]",
        );
        // The inner DSL still round-trips (the wrapper is cosmetic).
        let reparsed = Fingerprint::parse(&fp.to_string()).unwrap();
        assert_eq!(fp, reparsed);
    }

    // ── Direct keyword-map smoke (the reused reverse maps) ──────────────────

    #[test]
    fn qualifier_keyword_map_round_trips() {
        for k in [
            QualifierKind::Async,
            QualifierKind::Unsafe,
            QualifierKind::Const,
        ] {
            let c = Constraint::Qualifier(k);
            assert_round_trips(&c.to_string());
        }
    }

    #[test]
    fn glob_with_question_metachar_round_trips() {
        let c = Constraint::NameMatches(GlobPattern("a?b*c".to_string()));
        assert_round_trips(&c.to_string());
    }
}

//! The Fingerprint → DSL serializer round-trip PROPTEST oracle (ADR-063 §4.4).
//!
//! This is the contract's WITNESS: a strategy generates arbitrary **well-formed**
//! (parser-producible) `Fingerprint`s and asserts the round-trip type-law
//!
//! > `Fingerprint::parse(serialize(fp)) == Ok(fp)`
//!
//! holds for ALL of them. Where the per-variant matrix in `serialize.rs`'s unit
//! tests ENUMERATES the contract, this SEARCHES for the fp that breaks it — the
//! audit that the enumeration is complete (ADR-063 aristotle C2: proptest =
//! per-arm correctness by search; the exhaustive no-wildcard match =
//! coverage-by-compiler). It is also the ADR-063 §Standing-Pressure-Audit Q9
//! (the adversarial pre-implementation test) turned into a standing oracle.
//!
//! ## Why the strategy generates only WELL-FORMED fps
//!
//! The round-trip law is SCOPED to parser-producible fps (ADR-063 hazard c). A
//! generator that emitted ill-formed fps (empty globs/lists, misplaced `not`)
//! would test the WRONG law — the serializer faithfully emits those and the
//! parser correctly REJECTS them (covered by the `serialize.rs` scoped-domain
//! unit test). So the strategy constructs only fps the parser accepts, by
//! construction:
//!
//! - string-payload leaves whose parser gates a bare identifier
//!   (`derives`/`serde_arg`/`body_calls`/`body_contains_macro`/`impl_of_trait`)
//!   draw from a bare-ident pool;
//! - free-form string leaves (`attr_present`/`doc_contains`/`name` glob/
//!   `has_method`) draw payloads that survive the parser's non-empty +
//!   non-whitespace-only gates AND tokenize (the `has_method` signature must be
//!   a valid token stream);
//! - `all_of`/`any_of` are always non-empty;
//! - `not` is generated ONLY as an `all_of` child, always paired with a
//!   positive sibling (the only legal `not` position, ADR-010 Amd3 OQ3);
//! - depth and node-count stay well under `MAX_DEPTH` (10) / `MAX_NODES` (256).

use antigen_fingerprint::{
    Constraint, Fingerprint, GlobPattern, ItemKind, MethodPattern, QualifierKind, VariantRange,
};
use proptest::collection::vec;
use proptest::prelude::*;

/// A small pool of valid bare identifiers for the gated string leaves.
fn bare_ident() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("unwrap".to_string()),
        Just("expect".to_string()),
        Just("exit".to_string()),
        Just("panic".to_string()),
        Just("Hash".to_string()),
        Just("Eq".to_string()),
        Just("Drop".to_string()),
        Just("deny_unknown_fields".to_string()),
        Just("transparent".to_string()),
        Just("r#fn".to_string()),
    ]
}

/// A free-form payload that survives the parser's non-empty + non-whitespace
/// gates (for `attr_present` / `doc_contains`). Includes the escaping hazards
/// (`"`, `\`, newline) so the proptest stresses the composed escaper, not just
/// the unit fixture. Always starts with a non-whitespace char so trim-gates
/// pass.
fn free_payload() -> impl Strategy<Value = String> {
    // Build "X" + arbitrary tail; the leading non-space defeats whitespace-only
    // rejection while the tail can carry any escaping hazard.
    "[A-Za-z][ -~\\n\\\\\"]{0,12}"
}

/// A glob pattern: non-empty, with optional `*`/`?` metachars.
fn glob_payload() -> impl Strategy<Value = String> {
    "[A-Za-z*?][A-Za-z0-9*?_]{0,10}"
}

/// A `has_method` (name, signature) pair the parser accepts: a bare-ish name
/// and a signature that tokenizes. We keep the signature to canonical-ish
/// forms the parser's `normalize_signature_canonical` accepts.
fn has_method_pair() -> impl Strategy<Value = (String, String)> {
    let name = prop_oneof![
        Just("meet".to_string()),
        Just("join".to_string()),
        Just("clone".to_string()),
    ];
    let sig = prop_oneof![
        Just("(Self, Self) -> Self".to_string()),
        Just("(&self) -> bool".to_string()),
        Just("(&mut self, T) -> U".to_string()),
        Just("()".to_string()),
    ];
    (name, sig)
}

/// A leaf constraint (no combinators). The full alphabet of leaves — the
/// non-recursive `Constraint` variants.
fn leaf() -> impl Strategy<Value = Constraint> {
    prop_oneof![
        prop_oneof![
            Just(ItemKind::Struct),
            Just(ItemKind::Enum),
            Just(ItemKind::Trait),
            Just(ItemKind::Fn),
            Just(ItemKind::Impl),
            Just(ItemKind::Type),
            Just(ItemKind::Mod),
            Just(ItemKind::Const),
            Just(ItemKind::Static),
            Just(ItemKind::Union),
        ]
        .prop_map(Constraint::Item),
        glob_payload().prop_map(|g| Constraint::NameMatches(GlobPattern(g))),
        (0usize..50, 0usize..50).prop_map(|(a, b)| {
            let (min, max) = if a <= b { (a, b) } else { (b, a) };
            Constraint::Variants(VariantRange { min, max })
        }),
        has_method_pair().prop_map(|(name, signature)| {
            Constraint::HasMethod(MethodPattern {
                name,
                signature,
                normalized_signature: None,
            })
        }),
        free_payload().prop_map(Constraint::AttrPresent),
        free_payload().prop_map(Constraint::DocContains),
        bare_ident().prop_map(Constraint::BodyContainsMacro),
        bare_ident().prop_map(Constraint::BodyCalls),
        bare_ident().prop_map(Constraint::ImplOfTrait),
        bare_ident().prop_map(Constraint::Derives),
        bare_ident().prop_map(Constraint::SerdeArg),
        prop_oneof![
            Just(QualifierKind::Async),
            Just(QualifierKind::Unsafe),
            Just(QualifierKind::Const),
        ]
        .prop_map(Constraint::Qualifier),
    ]
}

/// A recursively-built constraint, depth-bounded well under `MAX_DEPTH`.
///
/// `Not` is generated ONLY inside `all_of`, ALWAYS paired with at least one
/// positive (non-`not`) sibling — the only legal `not` position. We build the
/// `all_of` child list as `[positive_leaf, maybe_not(child), more...]`, which
/// guarantees a positive sibling by construction.
fn constraint() -> impl Strategy<Value = Constraint> {
    leaf().prop_recursive(
        3,  // max recursion depth (well under MAX_DEPTH=10)
        24, // target max total nodes (well under MAX_NODES=256)
        4,  // children per collection node
        |inner| {
            prop_oneof![
                // all_of: a leading positive sibling guarantees `not` legality,
                // then 0..3 more children that MAY be `not(child)`.
                (leaf(), vec(maybe_not(inner.clone()), 0..3)).prop_map(|(head, tail)| {
                    let mut children = vec![head];
                    children.extend(tail);
                    Constraint::AllOf(children)
                }),
                // any_of: non-empty, NO bare `not` directly under it (illegal).
                vec(inner, 1..4).prop_map(Constraint::AnyOf),
            ]
        },
    )
}

/// A child that may be a `not(child)` or a plain child — used ONLY inside
/// `all_of`, where `not` is legal alongside the guaranteed positive sibling.
fn maybe_not(inner: BoxedStrategy<Constraint>) -> impl Strategy<Value = Constraint> {
    prop_oneof![
        3 => inner.clone(),
        1 => inner.prop_map(|c| Constraint::Not(Box::new(c))),
    ]
}

/// A well-formed top-level fingerprint: 1..4 top-level constraints. A bare
/// `not` is illegal at top level, so top-level constraints are drawn from a
/// strategy that never yields a bare `not` (our `constraint()` only emits `not`
/// as an `all_of` child, so this holds by construction).
fn fingerprint_strategy() -> impl Strategy<Value = Fingerprint> {
    vec(constraint(), 1..4).prop_map(|constraints| Fingerprint { constraints })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    /// THE round-trip oracle: for every generated well-formed fp,
    /// `parse(serialize(fp)) == fp`.
    #[test]
    fn parse_serialize_roundtrips(fp in fingerprint_strategy()) {
        let serialized = fp.to_string();
        let reparsed = Fingerprint::parse(&serialized);
        prop_assert!(
            reparsed.is_ok(),
            "serialized form must re-parse but errored.\n  fp: {fp:?}\n  serialized: {serialized}\n  err: {:?}",
            reparsed.err(),
        );
        let reparsed = reparsed.unwrap();
        prop_assert_eq!(
            &fp, &reparsed,
            "round-trip diverged.\n  original:   {:?}\n  serialized: {}\n  reparsed:   {:?}",
            fp, serialized, reparsed,
        );
    }
}

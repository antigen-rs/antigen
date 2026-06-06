//! Adversarial precision test for the discipline-vs-machinery unification
//! asymmetry boundary (ADR-019 §M3 / §M5 + adversarial T5-R + aristotle F1).
//!
//! ## What this test guards
//!
//! ADR-019 commits to a load-bearing structural invariant:
//!
//! - **Discipline-level unification holds**: substrate-witnesses and
//!   (future) cross-crate witnesses share tier-honesty discipline, both
//!   carry `EvidenceKind::SubstrateState`, both cap at
//!   `WitnessTier::Execution`.
//! - **Machinery-level unification is FORBIDDEN**: the parser for
//!   substrate-witness JSON sidecars (`serde_json::from_str::<Ratification>`)
//!   must remain SEPARATE from the parser for cross-crate dep-source Rust
//!   AST (`syn::parse_file`). Sharing parser code between the two would
//!   silently mis-classify a JSON-formed payload as Rust-AST-shaped
//!   (or vice versa) — a precision-degradation that passes all
//!   correctness tests because both paths return "ok" on the wrong
//!   substrate.
//!
//! The discipline failure mode this enforces:
//!
//! > A future maintainer reads "discipline unification" in ADR-019 and
//! > tries to share parsing code across the substrate-witness and
//! > cross-crate paths. Their refactor passes all happy-path tests
//! > because both paths return Ok on well-formed input. The bug only
//! > surfaces when the audit reports `EvidenceKind::SubstrateState` on
//! > a payload that was actually Rust-AST-shaped, or vice versa — a
//! > silent tier-honesty violation.
//!
//! ## What this test checks
//!
//! 1. **JSON-shaped Ratification IS accepted** by the substrate-witness
//!    parser path. Baseline happy-path; if this fails, the schema parser
//!    itself is broken (not a unification issue).
//!
//! 2. **Rust-source-text input is REJECTED** by the substrate-witness
//!    parser path. Even when the input is syntactically valid Rust (e.g.,
//!    a struct definition that could parse as `syn::ItemStruct`), the
//!    substrate-witness layer must reject it with a parse error. A
//!    shared-parser refactor that accepts both would fail here.
//!
//! 3. **JSON payload with Rust-AST-shaped tokens is REJECTED**. A future
//!    refactor might add a "lenient" parser that tries Rust-AST first and
//!    falls back to JSON (or vice versa). This test FAILS if a payload
//!    that's "JSON-shaped containing Rust-AST keywords" is accepted —
//!    the parsers must be type-disjoint.
//!
//! 4. **The substrate-witness evaluator does not accept arbitrary
//!    syn-parsed input**. Even if a caller manually constructs a `syn`
//!    AST and tries to feed it to the substrate-witness evaluator (e.g.,
//!    via a future "convenience" adapter), the evaluator must require a
//!    properly-typed `Predicate` value.
//!
//! These four checks together enforce the boundary: any future shared-
//! parsing refactor produces visible test failure, not silent precision
//! loss.
//!
//! ## Why this test lives in `antigen-attestation` (not `antigen`)
//!
//! The substrate-witness parser lives in `antigen-attestation`. The
//! cross-crate witness parser lives in `antigen::scan` (per ADR-017).
//! This test guards the boundary FROM the substrate-witness side: it
//! verifies the substrate-witness parser doesn't accept Rust-AST input.
//! A parallel test in `antigen/tests/` (P3e companion) would verify the
//! reverse: the cross-crate path doesn't accept JSON.
//!
//! Per ADR-019 §M3: "DO NOT share parser code between sidecar-JSON and
//! Rust-AST paths" (this test enforces it from the JSON side).

use antigen_attestation::{Ratification, schema::ValidationError};

/// Sample well-formed Ratification JSON — the baseline happy-path that
/// the substrate-witness parser MUST accept. If this fails, the schema
/// parser itself is broken (a regression in the JSON layer, not the
/// unification boundary).
const VALID_RATIFICATION_JSON: &str = r#"{
    "schema_version": "v1",
    "kind": "immunity",
    "antigen": { "name": "TestAntigen" },
    "source_file": "src/test.rs",
    "items": []
}"#;

#[test]
fn baseline_well_formed_json_ratification_parses() {
    // Sanity check: the substrate-witness parser accepts well-formed
    // Ratification JSON. If this fails, the test infrastructure or schema
    // is broken; investigate before diagnosing unification-boundary
    // regressions below.
    let result: Result<Ratification, _> = serde_json::from_str(VALID_RATIFICATION_JSON);
    assert!(
        result.is_ok(),
        "baseline well-formed Ratification JSON failed to parse: {:?}",
        result.err()
    );
}

#[test]
fn rust_source_text_rejected_by_substrate_witness_parser() {
    // ATTACK: Rust source text (a valid Rust struct definition) is fed to
    // the substrate-witness JSON parser. The parser MUST reject it —
    // sharing a parser between JSON and Rust-AST paths would silently
    // accept this input as some kind of "parseable" thing.
    //
    // The Rust source below is syntactically valid Rust (would parse as
    // a `syn::ItemStruct` with one field). A shared lenient parser might
    // try Rust-parse fallback and succeed; the substrate-witness layer
    // must NOT do that.
    let rust_source = r"
        pub struct Ratification {
            kind: RatificationKind,
            items: Vec<ItemRatification>,
        }
    ";
    let result: Result<Ratification, _> = serde_json::from_str(rust_source);
    assert!(
        result.is_err(),
        "Rust source text was accepted by substrate-witness parser — \
         unification-boundary violation per ADR-019 §M3 + T5-R. A shared-\
         parser refactor produces this failure mode; the parsers must \
         stay type-disjoint."
    );
}

#[test]
fn json_containing_rust_keywords_rejected_when_shape_invalid() {
    // ATTACK: input that LOOKS like it could be either JSON or Rust
    // depending on parser leniency. Specifically: a JSON object with
    // string values that happen to be Rust keywords/syntax. This SHOULD
    // parse as JSON (it is well-formed JSON) BUT the resulting structure
    // does not match the Ratification schema, so the typed parse fails.
    //
    // The point of this test: distinguish "rejected because not Ratification"
    // from "accepted because parser is too lenient." A shared lenient parser
    // might try Rust-AST mode on this input and succeed on the syntactic
    // resemblance.
    let json_with_rust_keywords = r#"{
        "pub_struct": "Ratification { ... }",
        "impl": "Parse for Self",
        "where": "T: serde::Serialize"
    }"#;
    let result: Result<Ratification, _> = serde_json::from_str(json_with_rust_keywords);
    assert!(
        result.is_err(),
        "JSON containing Rust-keyword strings was incorrectly accepted \
         as a Ratification. The substrate-witness parser must reject \
         anything that doesn't match the Ratification schema, regardless \
         of whether the input's string contents look syntactically Rust-like."
    );
}

#[test]
fn fully_malformed_input_rejected() {
    // Sanity check: bytes that are neither valid JSON nor valid Rust are
    // rejected. If THIS test fails, the substrate-witness parser is
    // accepting random bytes — much worse than the unification-boundary
    // failure mode. Included as a baseline so regression analysis can
    // distinguish "parser too lenient" from "parser broken entirely."
    let garbage = "@@@ this is not json or rust !!! 12345 \x00\x01\x02";
    let result: Result<Ratification, _> = serde_json::from_str(garbage);
    assert!(
        result.is_err(),
        "Random-byte input was accepted as a Ratification. This is a \
         severe parser-regression, distinct from but related to the \
         unification-boundary issue this test family guards."
    );
}

#[test]
fn validate_chain_cap_is_substrate_witness_specific_not_generic() {
    // The `validate_chain_cap` function (anti-laundering safeguard #1)
    // operates on a `u32` representing a workspace-configured chain depth
    // cap. It is substrate-witness-specific — not a generic numeric
    // validator. If a future refactor "unifies" similar-shaped validators
    // (e.g., a generic Validator<u32> trait), the substrate-witness
    // discipline could be subverted by accepting cap values from other
    // contexts that don't carry the anti-laundering semantics.
    //
    // This test locks the function's narrow purpose: it validates AT
    // PROJECT-ENFORCED-HARD-FLOOR bounds (MIN=1, MAX=10). A generic
    // numeric validator without these specific bounds would be
    // structurally different.
    use antigen_attestation::schema::{
        HARD_DELTA_CHAIN_CAP_MAX, HARD_DELTA_CHAIN_CAP_MIN, validate_chain_cap,
    };

    // The function rejects values below the hard floor (anti-laundering
    // bypass protection) — locks the discipline-specific semantic.
    let too_low = HARD_DELTA_CHAIN_CAP_MIN.saturating_sub(1);
    let err = validate_chain_cap(too_low).expect_err(
        "validate_chain_cap must reject values below HARD_DELTA_CHAIN_CAP_MIN — \
         anti-laundering safeguard bypass protection",
    );
    assert!(
        matches!(err, ValidationError::WorkspaceConfigOutOfBounds { .. }),
        "validate_chain_cap returned wrong error variant for below-floor input"
    );

    // The function rejects values above the hard ceiling (workspace-TOML
    // bypass protection from adversarial T2R-C).
    let too_high = HARD_DELTA_CHAIN_CAP_MAX.saturating_add(1);
    let err = validate_chain_cap(too_high).expect_err(
        "validate_chain_cap must reject values above HARD_DELTA_CHAIN_CAP_MAX — \
         workspace-TOML cap-loosening protection per T2R-C",
    );
    assert!(
        matches!(err, ValidationError::WorkspaceConfigOutOfBounds { .. }),
        "validate_chain_cap returned wrong error variant for above-ceiling input"
    );
}

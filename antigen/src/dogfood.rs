//! Antigen dogfood declarations — the antigen codebase applying v0.2
//! primitives to itself.
//!
//! These declarations encode failure-classes that adversarial found during the
//! v0.2-completion-arc expedition: sites in the antigen codebase that present
//! known failure patterns and need structural memory of that fact.
//!
//! Per ADR-006 (recognition-not-design): each declaration below was surfaced
//! from a real adversarial attack that found a real gap. None are hypothetical.
//! Campsite substrate: `dogfood-layer1-antigen-self-application`.

use antigen_macros::antigen;

// ============================================================================
// UnvalidatedSealedEnumAcceptance
//
// A sealed enum field (like WitnessClass) is parsed from an ident path but
// the ident is never validated against the closed set of ratified variants.
// Any custom ident passes silently. The downstream logic (e.g., min_independent
// distinctness check) then operates on the unvalidated string as if it were a
// valid variant — a silent false-positive on a 'known defense.'
//
// Empirical basis: DiagnosticArgs::parse (antigen-macros/src/parse.rs ~1543)
// extracts WitnessClass::X as a String but does not check it is one of the
// 6 ratified variants. Adversarial ATK-PROCESS-5 convergent-HG3.
// ============================================================================

#[antigen(
    name = "unvalidated-sealed-enum-acceptance",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("WitnessClass")])"#,
    family = "functional-correctness",
    summary = "A sealed enum field is parsed as a string ident without validating against the closed variant set; arbitrary idents pass silently, defeating the sealed-enum discipline.",
    references = ["ADR-024", "ADR-028"]
)]
pub struct UnvalidatedSealedEnumAcceptance;

// ============================================================================
// FingerprintStringWithoutDslValidation
//
// A field that accepts fingerprint syntax strings does not call
// antigen_fingerprint::Fingerprint::parse() to validate them, while a
// sibling field at a different site does. This creates an inconsistent trust
// boundary: one parse site enforces the DSL contract, the other silently
// accepts malformed fingerprints. An adopter writing a valid-looking but
// malformed fingerprint string gets no compile-time signal.
//
// Empirical basis: CrossreactiveArgs::validate() checks non-empty but does
// not call Fingerprint::parse(); AntigenArgs::validate() does call it.
// Adversarial ATK-PROCESS-5 convergent-HG2.
// ============================================================================

#[antigen(
    name = "fingerprint-string-without-dsl-validation",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("fingerprint")])"#,
    family = "functional-correctness",
    summary = "A fingerprint-accepting field validates only non-emptiness, not DSL correctness, while sibling sites call Fingerprint::parse(). Malformed fingerprints pass silently at the inconsistent site.",
    references = ["ADR-024"]
)]
pub struct FingerprintStringWithoutDslValidation;

// ============================================================================
// SilentArgumentDiscard
//
// A proc-macro Parse impl consumes and discards all input tokens in a loop
// without examining them. Adopters who pass structured arguments (thinking
// they declare constraints) receive no error and no effect — their intent is
// silently nullified. The macro looks like it accepted meaningful input.
//
// Empirical basis: PolyclonalArgs, MonoclonalArgs, AdccArgs in
// antigen-macros/src/parse.rs all use the discard-loop pattern. The
// doc-comment says 'forward compat' but the forward-compat window is itself
// undeclared — when args are eventually added, every existing call site that
// passed arguments will silently stop having their old arguments honored.
// Adversarial ATK-PROCESS-5 convergent-HG1.
// ============================================================================

#[antigen(
    name = "silent-argument-discard",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("forward compat")])"#,
    family = "functional-correctness",
    summary = "A proc-macro Parse impl discards all arguments in a loop; adopter-supplied arguments are silently nullified with no error, making the macro appear to accept constraints it ignores.",
    references = ["ADR-024"]
)]
pub struct SilentArgumentDiscard;

// ============================================================================
// VecCardinalityMasqueradingAsSet
//
// A Vec field semantically represents a set (where duplicates are meaningless
// or harmful), but no deduplication or duplicate-rejection happens at parse
// time. A caller supplying [SA, SA] gets a cardinality-2 result that implies
// set membership (SA + FC hybrid) without actually being one. Downstream logic
// that counts Vec length as a proxy for set membership produces false results.
//
// Empirical basis: AntigenDeclaration.category: Vec<AntigenCategory> can hold
// duplicates. [SubstrateAlignment, SubstrateAlignment] has len 2, which looks
// like a hybrid (SA + FC) but is not. Production fix exists (dedup at audit
// time) but the #[presents] declaration was missing, so future drift at this
// site would be undetected. Adversarial ATK fa85a1e5 era.
// ============================================================================

#[antigen(
    name = "vec-cardinality-masquerading-as-set",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"item = struct, attr_present("derive")"#,
    family = "functional-correctness",
    summary = "A Vec is used where a set is intended; duplicates produce a cardinality count that implies set-membership semantics without enforcing them, causing downstream length-checks to misclassify.",
    references = ["ADR-028"]
)]
pub struct VecCardinalityMasqueradingAsSet;

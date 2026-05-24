//! # Antigen-Internal Dogfood Antigens
//!
//! Antigen eating its own cooking — failure-classes sourced from direct
//! observation of antigen's own development and coordination substrate during
//! the v0.2 completion arc (2026-05-24 expedition).
//!
//! Three classes declared here were each witnessed live as substrate-claim
//! mismatches during sign-pass audits and coordination substrate checks. They
//! are encoded preemptively per `feedback-internal-tool-antigens-preemptive`:
//! the cost of encoding after occurrence is asymmetrically higher than
//! encoding from shape-prediction.
//!
//! ## Category (ADR-028)
//!
//! - `AntigenDeclarationMissingCategory` — `SubstrateAlignment`: the
//!   parse-time representation (valid compilation) diverges from the
//!   audit-time representation (hint: category absent).
//! - `DelegatedHandlerKindMismatch` — `SubstrateAlignment`: the
//!   `#[mucosal_delegate]` declaration's `handled_by` path resolves at
//!   parse-time, but the handler's `#[mucosal(kind = X)]` kind-matching is
//!   deferred to audit-time. Two-phase representation of the same contract.
//! - `WitnessClaimWithoutImplementation` — `FunctionalCorrectness`: an
//!   `#[immune]` declaration names a witness that does not execute a
//!   meaningful verification; the immunity claim and the actual verification
//!   state diverge.
//!
//! ## Biology grounding
//!
//! These are software-engineering failure-classes observed in antigen's own
//! coordination substrate — no biological cognates are claimed. The antigen
//! project's biology grounding discipline applies to the stdlib's boundary
//! families (mucosal, supply-chain) per ADR-027 NON-NEGOTIABLE; it does not
//! require biology analogues for internal-tooling dogfood antigens.

use crate::antigen;

// ============================================================================
// 1. AntigenDeclarationMissingCategory
// ============================================================================

/// An `#[antigen]` declaration with no `category = AntigenCategory::X` field.
///
/// The parse-time layer (proc-macro, `cargo check`) accepts the declaration as
/// valid. The audit-time layer (`cargo antigen audit`) emits the
/// `antigen-category-defaulted-implicit-functional` hint. These two
/// representations of the same declaration's completeness diverge silently:
/// parse says "complete," audit says "category is missing."
///
/// **Observed instance** (2026-05-24, v0.2 completion arc): the G1/G2/G3
/// workstreams were open — new `#[antigen]` declarations could be written
/// without `category` and compile cleanly. The absence was only surfaced by
/// running `cargo antigen audit`. The divergence window between compile-clean
/// and audit-flagged is structurally predictable whenever parse-time
/// enforcement is softer than audit-time enforcement.
///
/// **Defense**: add `category = AntigenCategory::X` to every `#[antigen]`
/// declaration. The `cargo antigen audit` hint fires until this is done.
/// v0.2.x will add parse-time hard-error for new declarations via the G1
/// migration-record slice.
///
/// **Category**: `SubstrateAlignment` — two representations of the same
/// declaration's completeness state (parse-valid vs audit-flagged) diverge.
#[antigen(
    name = "antigen-declaration-missing-category",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-028")"#,
    family = "dogfood",
    summary = "An #[antigen] declaration omits `category = AntigenCategory::X`; parse-time accepts it, audit-time flags it — two representations of the same completeness state diverge.",
    references = ["ADR-028", "ADR-028#G1"]
)]
pub struct AntigenDeclarationMissingCategory;

// ============================================================================
// 2. DelegatedHandlerKindMismatch
// ============================================================================

/// A `#[mucosal_delegate]` declaration where the handler's boundary kind
/// does not match the delegated `boundary` kind.
///
/// The `handled_by` path is resolved at parse-time (typos are compile
/// errors per ADR-027 Amendment 1 Change 4). But kind-matching — verifying
/// that the handler function itself carries `#[mucosal(kind = X)]` where X
/// matches the `boundary` field — is an audit-time check (Change 5).
///
/// The result is a two-phase representation of the same contract: the
/// delegation is parse-valid (the path exists) while the kind-match may be
/// violated (the handler defends a different boundary kind). The contract
/// expressed by `#[mucosal_delegate(boundary = MucosalKind::ApiRequest, ...)]`
/// has a different meaning depending on which phase you are in.
///
/// **Observed shape** (2026-05-24): during the sign-pass for
/// `v02-impl-mucosal-boundary`, observer confirmed that `handled_by` is
/// `syn::Path` (enforced at parse-time) but kind-matching is explicitly
/// deferred to audit via the three-tier delegate diagnosis. The gap between
/// the two enforcement phases is structurally load-bearing — any new
/// `#[mucosal_delegate]` usage sits in this window until `cargo antigen
/// mucosal-map` runs.
///
/// **Defense**: `cargo antigen mucosal-map --delegates` runs the three-tier
/// audit-time diagnosis. Mark delegation sites with `#[mucosal_delegate]`
/// and run mucosal-map in CI to close the enforcement gap.
///
/// **Category**: `SubstrateAlignment` — the delegation contract's kind-match
/// is represented at parse-time (path resolves) and differently at audit-time
/// (kind may not match). Two representations, potentially divergent.
#[antigen(
    name = "delegated-handler-kind-mismatch",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-027")"#,
    family = "dogfood",
    summary = "`#[mucosal_delegate]` path resolves at parse-time but handler kind-matching is deferred to audit-time; the delegation contract has two representations that can diverge.",
    references = ["ADR-027", "ADR-027#Amendment-1-Change-4", "ADR-027#Amendment-1-Change-5"]
)]
pub struct DelegatedHandlerKindMismatch;

// ============================================================================
// 3. WitnessClaimWithoutImplementation
// ============================================================================

/// An `#[immune]` declaration names a witness that does not actually verify.
///
/// The witness is a stub, a TODO, or a function that always returns true
/// without testing the actual failure-class invariant.
///
/// The `#[immune(X, witness = my_test)]` declaration claims that `my_test`
/// provides evidence of immunity to `X`. `cargo antigen audit` checks that
/// `my_test` exists and is not ignored. But it cannot verify that `my_test`
/// actually tests what `X` names — a stub that `assert!(true)` satisfies the
/// structural check while providing no real evidence.
///
/// The immunity claim (at the `#[immune]` declaration site) and the actual
/// verification state (in the test body) diverge: the declaration says
/// "defended," the test says "untested."
///
/// **Category**: `FunctionalCorrectness` — the verb (`#[immune]`) claims a
/// correct outcome (verified defense) but the implementation (test body)
/// does not produce that outcome. This is distinct from `SubstrateAlignment`
/// (representation vs state) because the failure is in what the witness
/// actually executes, not in which layer names the work-unit.
///
/// **Defense**: antigen's own witness-review audit (`cargo antigen audit
/// --witness-review`) surfaces witnesses flagged as stubs or missing
/// predicate conditions. Encode substantive test bodies; use proptest /
/// kani / prusti witnesses where the invariant is non-trivial.
#[antigen(
    name = "witness-claim-without-implementation",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("ADR-005")"#,
    family = "dogfood",
    summary = "An #[immune] declaration names a witness that does not execute a meaningful verification — the immunity claim and actual verification state diverge.",
    references = ["ADR-005", "ADR-005#Amendment-3", "ADR-019"]
)]
pub struct WitnessClaimWithoutImplementation;

// ============================================================================
// 4. VecCardinalityMasqueradingAsSet
// ============================================================================

/// A `Vec<T>` field that models a *set* (each variant meaningful at most once),
/// where duplicate elements produce a cardinality count that implies membership
/// it does not have.
///
/// `category` on `#[antigen]` is semantically a set: `SubstrateAlignment`,
/// `FunctionalCorrectness`, or both (hybrid). It is *represented* as
/// `Vec<MacroAntigenCategory>`. Before the dedup-check,
/// `category = [SubstrateAlignment, SubstrateAlignment]` parsed to a 2-element
/// Vec — and downstream code that reads `len() == 2` to mean "hybrid antigen"
/// was fooled: two identical variants look like a hybrid (SA + FC) but carry
/// only one distinct category.
///
/// **Observed instance** (2026-05-24, fixed in `30e10e6`): adversarial pinned
/// the pre-fix behavior (`[SA, SA]` and `[SA, FC, SA]` were accepted at
/// parse-time). `AntigenArgs::validate()` now rejects duplicate entries with a
/// `duplicate AntigenCategory variant` error — the regression test
/// `antigen_parser_duplicate_category_in_array_is_rejected` pins the fix.
///
/// **Defense**: enforce uniqueness at the parse/validate boundary for any
/// set-semantics `Vec` (dedup-check, or model it as a true set type). Where a
/// cardinality count drives a downstream branch (`len() == N` meaning a
/// distinct configuration), the count must be a count of *distinct* elements.
///
/// **Category**: `FunctionalCorrectness` — `validate()` accepted input that
/// produced a wrong downstream interpretation (false-hybrid). The failure is in
/// what the code computes from the Vec, not in a representation-vs-state layer
/// split, so this is correctness, not substrate-alignment.
#[antigen(
    name = "vec-cardinality-masquerading-as-set",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("ADR-028")"#,
    family = "dogfood",
    summary = "A Vec field modeling a set permits duplicate elements; a cardinality count (len == N) then implies a distinct configuration (e.g. hybrid) the data does not actually carry.",
    references = ["ADR-028", "ADR-028#Schema"]
)]
pub struct VecCardinalityMasqueradingAsSet;

// ============================================================================
// 5. AuditHintWithNoUpstreamPreconditionCheck
// ============================================================================

/// An audit arm that verifies one direction of a temporal progression (the
/// downstream action occurred) without verifying the upstream precondition
/// (the action was reachable from a valid prior state).
///
/// The recurrent-emergence family models a temporal progression: `#[itch]`
/// (early signal) → `#[recurrence_anchor]` (the recurrence is acknowledged) →
/// crystallize (it becomes a tracked work-unit). The `RecurrenceAnchor` audit
/// arm originally checked the *downstream* edge (`acted_on`) but not the
/// *upstream* edge: a "floating" anchor with no `#[itch]` declaration
/// referencing the same antigen type bypassed the `itch → anchor` step with no
/// audit signal.
///
/// **Observed instance** (2026-05-24, fixed in `dd51d4b`): ATK-RECURRENT-2.
/// The fix added `AuditHint::RecurrenceAnchorNoItchPrecondition`, threaded
/// `itch_antigen_types: HashSet<&str>` into `evaluate_recurrent_hints`, and
/// fires when an anchor has no `from_itches` entries AND no `#[itch]` in the
/// report shares the same `antigen_type`. The positive case (hint clears when a
/// matching itch exists) is verified alongside the negative.
///
/// **Defense**: when an audit arm validates a step in a multi-stage temporal
/// progression, check *both* edges — the upstream precondition that makes the
/// step reachable and the downstream action that the step enables. A
/// one-directional check leaves a silent bypass at the unchecked edge.
///
/// **Category**: `FunctionalCorrectness` — the audit arm produced a wrong
/// result (silent pass) for a real violation (floating anchor). The hint logic
/// computed the wrong answer; nothing about representation-vs-state diverged.
#[antigen(
    name = "audit-hint-with-no-upstream-precondition-check",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("ADR-024")"#,
    family = "dogfood",
    summary = "An audit arm checks the downstream action of a temporal progression but not the upstream precondition, leaving a silent bypass at the unchecked edge.",
    references = ["ADR-024", "ADR-024#Family-2"]
)]
pub struct AuditHintWithNoUpstreamPreconditionCheck;

// ============================================================================
// 6. RatifiedSpecDriftFromImpl
// ============================================================================

/// A ratified ADR or spec surfaces one contract; the code that realizes it
/// ships a different (usually narrower) contract — and no structural check
/// catches the divergence at landing time.
///
/// This is a **spec-vs-realization** divergence. The spec is the authoritative
/// representation of intent; the implementation is the realization. When they
/// diverge silently, the spec becomes aspirational documentation rather than
/// a binding contract.
///
/// **Two substrate-grounded instances from the v0.2 completion arc
/// (2026-05-24)**:
///
/// 1. *orient drift (aristotle F4)*: ADR-023 §Decision specified
///    `#[orient(antigen, learning_path, until)]` with parse-time horizon
///    validation + CI gate. Commit `49a11eb` shipped
///    `#[orient(antigen, see, adr, attestation_optional)]` — two of three
///    enforcement mechanisms unrealized at the landing commit. Spec-vs-code.
///
/// 2. *G2 spec-drift*: ADR-028 Amendment 2 clarified that the
///    substrate-witness-leaf requirement does NOT apply to the fingerprint
///    predicate tree (Interpretation 2). A downstream campsite spec (written
///    citing aristotle's F2 before Amendment 2 corrected the layer) said "walk
///    the fingerprint predicate tree at parse-time" — directly contradicting
///    the amended ADR. Pathmaker caught it at impl-time via substrate-grep.
///    Spec-vs-spec (amendment vs downstream-spec).
///
/// **The generalization**: spec-vs-realization drift occurs at multiple
/// spec-layers (ADR vs code, amendment vs downstream-spec, process-doc vs
/// macro-arg-surface). The fail-class generalizes across layers; the
/// subcategory is the distance between where intent was expressed and where
/// realization was verified.
///
/// **Sibling fail-class**: `ParallelStateTrackersDiverge` (two trackers of the
/// same runtime state diverge). Together these form a **substrate-divergence**
/// parent family: `RatifiedSpecDriftFromImpl` = intent-representation vs
/// realization diverge; `ParallelStateTrackersDiverge` = two runtime
/// state-trackers diverge. Both are `SubstrateAlignment` — the substrate
/// (spec, tracker) does not match the actual (code, other-tracker).
///
/// **The mechanization path**: when ADRs carry structured §Proc-Macro-Surface
/// tables (aristotle's process-amendment F2 sub-clause), the spec becomes
/// machine-checkable: `cargo antigen audit` can compare the declared surface
/// against the scanned source. v0.2 ships this antigen with an advisory
/// substrate-witness (doc-contains); v0.2.x mechanizes it once structured
/// spec-surfaces land.
///
/// **Category**: `SubstrateAlignment` — the spec is a representation of intent;
/// the implementation is the actual state. Divergence is a representation-vs-
/// state split, not a computation-correctness failure.
#[antigen(
    name = "ratified-spec-drift-from-impl",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("ADR-")"#,
    family = "dogfood",
    summary = "A ratified ADR or spec expresses one contract; the implementation ships a narrower or different contract with no structural check catching the divergence at landing time.",
    references = ["ADR-023", "ADR-028", "ADR-028#Amendment-2"]
)]
pub struct RatifiedSpecDriftFromImpl;

// ============================================================================
// 7. UnvalidatedSealedEnumAcceptance
// ============================================================================

/// A sealed-enum field accepts any ident without validating the closed set.
///
/// The ident is parsed from a path but never checked against the ratified
/// variants, so a custom ident passes silently and downstream logic (e.g. a
/// `min_independent` distinctness check) treats it as a valid variant.
///
/// **Observed instance** (2026-05-24, ATK-PROCESS-5 convergent-HG3):
/// `DiagnosticArgs::parse` (antigen-macros/src/parse.rs ~1543) extracts
/// `WitnessClass::X` as a `String` but does not check it is one of the 6
/// ratified variants. `WitnessClass::CustomThing` passes silently and is
/// counted as valid by the distinctness check — a false-positive on a "known
/// defense."
///
/// **Defense**: validate the extracted ident against the sealed variant set at
/// parse-time; reject unknown idents with a `syn::Error` naming the valid set.
///
/// **Category**: `FunctionalCorrectness` — the parser accepts input that
/// produces a wrong downstream result (a non-variant counted as a variant).
#[antigen(
    name = "unvalidated-sealed-enum-acceptance",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("WitnessClass")])"#,
    family = "dogfood",
    summary = "A sealed enum field is parsed as a string ident without validating against the closed variant set; arbitrary idents pass silently, defeating the sealed-enum discipline.",
    references = ["ADR-024", "ADR-028"]
)]
pub struct UnvalidatedSealedEnumAcceptance;

// ============================================================================
// 8. FingerprintStringWithoutDslValidation
// ============================================================================

/// A fingerprint-accepting field skips DSL validation that a sibling site runs.
///
/// One parse site calls `antigen_fingerprint::Fingerprint::parse()`; the other
/// only checks non-emptiness. The result is an inconsistent trust boundary
/// where malformed fingerprints pass silently at the unguarded site.
///
/// **Observed instance** (2026-05-24, ATK-PROCESS-5 convergent-HG2):
/// `CrossreactiveArgs::validate()` checks non-emptiness but does not call
/// `Fingerprint::parse()`; `AntigenArgs::validate()` does. An adopter writing a
/// valid-looking but malformed fingerprint at the unguarded site gets no
/// compile-time signal.
///
/// **Defense**: route every fingerprint-accepting field through the same
/// `Fingerprint::parse()` validation. Cross-site consistency at a trust
/// boundary is sub-clause F (ADR-005) applied to the DSL contract.
///
/// **Category**: `FunctionalCorrectness` — a validation that should reject
/// malformed input silently accepts it, diverging from the sibling site's
/// contract.
#[antigen(
    name = "fingerprint-string-without-dsl-validation",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("fingerprint")])"#,
    family = "dogfood",
    summary = "A fingerprint-accepting field validates only non-emptiness, not DSL correctness, while sibling sites call Fingerprint::parse(). Malformed fingerprints pass silently at the inconsistent site.",
    references = ["ADR-005", "ADR-024"]
)]
pub struct FingerprintStringWithoutDslValidation;

// ============================================================================
// 9. SilentArgumentDiscard
// ============================================================================

/// A proc-macro `Parse` impl discards all input tokens without examining them.
///
/// Adopters who pass structured arguments — thinking they declare constraints —
/// receive no error and no effect; their intent is silently nullified while the
/// macro appears to have accepted meaningful input.
///
/// **Observed instance** (2026-05-24, ATK-PROCESS-5 convergent-HG1):
/// `PolyclonalArgs`, `MonoclonalArgs`, `AdccArgs` in antigen-macros/src/parse.rs
/// all use the discard-loop pattern. The doc-comment says "forward compat" but
/// the forward-compat window is itself undeclared — when args are eventually
/// added, every existing call site that passed arguments will silently stop
/// having its old arguments honored.
///
/// **Defense**: either reject unexpected tokens at parse-time (strict), or
/// document the forward-compat contract explicitly so the discard behavior is a
/// named, bounded decision rather than a silent footgun.
///
/// **Category**: `FunctionalCorrectness` — the macro produces a result (no
/// constraints) that contradicts the adopter's declared intent (constraints
/// supplied).
#[antigen(
    name = "silent-argument-discard",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("forward compat")])"#,
    family = "dogfood",
    summary = "A proc-macro Parse impl discards all arguments in a loop; adopter-supplied arguments are silently nullified with no error, making the macro appear to accept constraints it ignores.",
    references = ["ADR-024"]
)]
pub struct SilentArgumentDiscard;

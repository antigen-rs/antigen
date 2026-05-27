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

use crate::{antigen, descended_from};

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
/// **Four substrate-grounded instances from the v0.2 completion arc
/// (2026-05-24)** — each caught by substrate-grep-before-implementing, none by
/// a test:
///
/// 1. *orient drift (aristotle F4)*: ADR-023 §Decision specified
///    `#[orient(antigen, learning_path, until)]` with parse-time horizon
///    validation + CI gate. Commit `49a11eb` shipped
///    `#[orient(antigen, see, adr, attestation_optional)]` — two of three
///    enforcement mechanisms unrealized at the landing commit. Spec-vs-code.
///
/// 2. *G2 fingerprint spec-drift*: ADR-028 Amendment 2 clarified that the
///    substrate-witness-leaf requirement does NOT apply to the fingerprint
///    predicate tree (Interpretation 2). A downstream campsite spec (written
///    citing aristotle's F2 before Amendment 2 corrected the layer) said "walk
///    the fingerprint predicate tree at parse-time" — directly contradicting
///    the amended ADR. Caught at impl-time via substrate-grep. Spec-vs-spec
///    (amendment vs downstream-spec).
///
/// 3. *G3 §Enforcement-Surface-table drift*: ADR-028's §Enforcement-Surface
///    table row 1 said "category missing → parse-time HARD ERROR," but the
///    later G1 ratification softened that to scan-time-only-for-v0.2. The table
///    was never re-synced, so the G3 spec inherited a hint vocabulary that
///    contradicted the ratified G1/G2 decisions. Caught at impl-time;
///    re-synced in ADR-028 Amendment 4. Spec-vs-spec (table vs later
///    amendment).
///
/// 4. *agent-identity collision*: during this expedition, a coordination-layer
///    instance — an agent's post-compaction self-model diverged from its
///    registered team identity (the substrate `team-config.json`), producing a
///    duplicated implementation of one failure-class across two modules. The
///    representation (who-the-agent-thinks-it-is) diverged from the actual
///    state (the roster). Caught by substrate-check (team-config + a verbatim
///    message-loop). This is the failure-class one layer out — drift between a
///    self-model and the substrate that defines it.
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
    references = ["ADR-023", "ADR-028", "ADR-028#Amendment-2", "ADR-028#Amendment-4"]
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

/// A field holding a structural DIGEST is accepted without format-validation
/// that a sibling site runs — the digest-format sibling of
/// [`FingerprintStringWithoutDslValidation`].
///
/// Both share the parent shape: *a fingerprint-accepting field skips the
/// format-validation a sibling site runs, so malformed values pass silently*
/// (cross-site trust-boundary inconsistency, sub-clause F). They split by the
/// **kind of validation** missing — the same witness-structure discriminator
/// that separates `ActiveArgumentDiscard` (behavioral) from
/// `CapabilityOmissionAtLowering` (structural):
///
/// - [`FingerprintStringWithoutDslValidation`] — the field holds the fingerprint
///   **DSL grammar** (`item = fn`, `all_of([...])`); the missing check is
///   `Fingerprint::parse()`.
/// - `FingerprintDigestWithoutFormatValidation` (this one) — the field holds a
///   structural **digest** (`fnv1a64:<16 hex>`, e.g. from `cargo antigen scan
///   --format json`); the missing check is the digest *shape*
///   (`fnv1a64:`-prefix + 16 lowercase-hex). `Fingerprint::parse()` cannot apply
///   — a hash is not grammar — so DSL-validation is the wrong defense here.
///
/// **Observed instance** (2026-05-26): the `cargo antigen attest`
/// `--fingerprint` args hold digests. The `sign` site format-guarded them
/// (`looks_like_structural_digest`) but `scaffold`/`delta`/`check` did not — a
/// 1-of-N cross-site spread where a malformed digest at the unguarded sites was
/// recorded as `signed_against_fingerprint` and silently never matched the
/// item's real digest at audit (dead-on-arrival). Closed by running the guard at
/// every digest-accepting verb.
///
/// **Defense**: route every digest-accepting field through the same format check
/// (`fnv1a64:` + 16 lowercase-hex). Cross-site consistency at a trust boundary
/// is sub-clause F (ADR-005) applied to the digest contract — the same discipline
/// the DSL sibling applies to the grammar contract.
///
/// **Category**: `FunctionalCorrectness` — a validation that should reject
/// malformed input silently accepts it, diverging from the sibling site's
/// contract (identical to its DSL sibling).
#[antigen(
    name = "fingerprint-digest-without-format-validation",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, doc_contains("fnv1a64")])"#,
    family = "dogfood",
    summary = "A field holding a structural digest (fnv1a64:<hex>) is accepted without format-validation that a sibling site runs; malformed digests pass silently at the inconsistent site and never match at audit. The digest-format sibling of FingerprintStringWithoutDslValidation (same cross-site-inconsistency parent, split by validation-kind: digest-format vs DSL-grammar).",
    references = ["ADR-005", "ADR-019"]
)]
pub struct FingerprintDigestWithoutFormatValidation;

// ============================================================================
// 9. SilentIntentNullification (parent) + ActiveArgumentDiscard (child)
// ============================================================================

/// Parent of the intent-nullification family: a surface appears to accept or
/// honor an adopter's declared intent but does not realize it.
///
/// The adopter supplies something meaningful (arguments, a field, a constraint)
/// and the surface neither errors nor effects it — the intent is silently
/// nullified between declaration and realization, while the surface *looks* like
/// it accepted the input. Per aristotle's F5 ratification the family has two
/// children distinguished by **witness-structure** (not by loud-vs-silent —
/// both are silent at their layer):
///
/// - [`ActiveArgumentDiscard`] (#9, below): the parse step actively swallows
///   tokens in a discard-loop without examining them. Witness is BEHAVIORAL —
///   feed unexpected args and assert a compile-error (which it currently does
///   NOT produce; that is the defect).
/// - [`CapabilityOmissionAtLowering`] (#16): the parse step accepts the value
///   but the lowering omits it (no AST arm + hardcoded default). Witness is
///   STRUCTURAL — the parse-arm/target-field parity test.
///
/// **Category**: `FunctionalCorrectness` — the produced result (no effect)
/// contradicts the adopter's declared intent (effect supplied). The summary
/// tier of both children.
#[antigen(
    name = "silent-intent-nullification",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("silently nullified")"#,
    family = "dogfood",
    summary = "A surface appears to accept or honor an adopter's declared intent but does not realize it; the intent is silently nullified between declaration and effect. Parent of ActiveArgumentDiscard (parse-side, behavioral witness) and CapabilityOmissionAtLowering (lowering-side, structural witness), which differ by witness-structure — both are silent.",
    references = ["ADR-024", "ADR-007"]
)]
pub struct SilentIntentNullification;

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
/// **Witness-structure** (the F5 sibling discriminator): this child *silently
/// swallows* tokens at parse (`while !input.is_empty() { input.parse()? }`) —
/// it does NOT loudly reject. Its witness is BEHAVIORAL: feed unexpected args,
/// assert a compile-error. Its sibling [`CapabilityOmissionAtLowering`] silently
/// drops at lowering and is caught by a STRUCTURAL parity test. Both silent;
/// they differ by witness-structure.
///
/// **Defense**: either reject unexpected tokens at parse-time (strict), or
/// document the forward-compat contract explicitly so the discard behavior is a
/// named, bounded decision rather than a silent footgun.
///
/// **Fingerprint note** (this antigen ate its own dog food — it was an instance
/// of [`AntigenFingerprintDivergesFromClassExtension`] (#17)). The original
/// fingerprint `doc_contains("forward compat")` could not match ANY of its own
/// instances: the "forward compat" text lives in a `//` comment in
/// `PolyclonalArgs`/`MonoclonalArgs`/`AdccArgs::parse`, and `syn` discards `//`
/// comments — only `///` doc-attrs reach `doc_contains`. A fingerprint fitted to
/// an instance-artifact (an expected doc string) under-covered its own class
/// (the F4/under-coverage direction). The fix applies the F8 recall/precision
/// split: a BROAD structural recall-fingerprint (`item = impl` + a `parse`
/// method of the `Parse`-impl shape) catches the class; the precise
/// discriminator (is the body actually a token-discard-loop?) is the WITNESS's
/// job, not the fingerprint's. The discard-loop body itself is not expressible
/// in the v0.2 predicate grammar (no body-expression predicate;
/// `body_contains_macro` doesn't fire — the loop has no macro), which is exactly
/// why precision must live in the witness rather than a doc-string proxy.
///
/// **Category**: `FunctionalCorrectness` — the macro produces a result (no
/// constraints) that contradicts the adopter's declared intent (constraints
/// supplied).
#[descended_from(SilentIntentNullification)]
#[antigen(
    name = "active-argument-discard",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = impl, has_method("parse", "(ParseStream) -> syn::Result<Self>")])"#,
    family = "dogfood",
    summary = "A proc-macro Parse impl actively swallows all arguments in a discard-loop; adopter-supplied arguments are silently nullified with no error, making the macro appear to accept constraints it ignores. Behavioral-witness child of SilentIntentNullification; sibling of CapabilityOmissionAtLowering (parse-swallow vs lowering-drop, both silent). Fingerprint is broad-recall (Parse-impl shape); precision lives in the witness (F8 recall/precision split).",
    references = ["ADR-024", "ADR-010"]
)]
pub struct ActiveArgumentDiscard;

// ============================================================================
// 10. ScannerBoundaryFalseNegative
// ============================================================================

/// A static-heuristic boundary scanner misses a trust boundary because the
/// boundary expression uses a non-standard input type.
///
/// Antigen's mucosal scanner (`cargo antigen mucosal-map --undefended`) detects
/// trust boundaries by matching function parameter types against a set of
/// recognized patterns (e.g., `actix_web` `Path<T>`, axum `Json<T>`,
/// `HttpRequest`, etc.). A handler that uses a *custom* request type — one the
/// heuristic does not recognize — will not be surfaced as an undefended boundary.
/// The `--undefended` flag then reports "0 undefended boundaries" when there is
/// actually 1 (or more).
///
/// **Why this is structural, not a bug**: static-analysis-based boundary
/// detection can never be complete for a Turing-complete language. The heuristic
/// can be extended (adding recognized patterns), but it cannot be made exact.
/// This is not a bug to fix; it is a residual risk to document and monitor.
///
/// **Observed instance** (ATK-MUCOSAL-6, 2026-05-24): the adversarial fixture
/// for this case is `#[ignore]` pending mucosal scanner maturation. The test
/// itself correctly documents that the expected behavior is "0 detected
/// boundaries" for a custom type — i.e., the scanner's representation of
/// boundary sites diverges from the actual set.
///
/// **Defense**: document the heuristic's scope explicitly (which types are
/// recognized). Provide a way for adopters to annotate custom boundary types as
/// `#[mucosal]` directly so the scanner can include them. The ATK-MUCOSAL-6
/// fixture, when un-ignored, pins the expected-gap behavior so regressions are
/// not silent.
///
/// **Category**: `SubstrateAlignment` — the scanner's representation (detected
/// boundary set) diverges from the actual substrate state (all trust-boundary
/// sites in the codebase). The divergence is not a runtime error; it is a silent
/// false negative in the scan output.
#[antigen(
    name = "scanner-boundary-false-negative",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("mucosal-map")"#,
    family = "dogfood",
    summary = "Static-heuristic boundary scanner misses trust boundaries expressed with non-standard input types; --undefended reports 0 when boundaries exist. Structural residual risk, not a fixable bug.",
    references = ["ADR-027", "ADR-027#Amendment-1"]
)]
pub struct ScannerBoundaryFalseNegative;

// ============================================================================
// 11. BiologyGroundingClaimDrift
// ============================================================================

/// A biology-grounding doc-comment claims an axis or a claim-tier that diverges
/// from the ratified ADR §Biology grounding for that primitive.
///
/// Antigen's biological metaphor is load-bearing, not decorative (ADR-003): the
/// 17 `# Biology grounding` doc-comment blocks in `antigen-macros` plus the
/// per-family stdlib module headers each assert a specific **grounding axis**
/// (immunology-proper / clinical-medicine / cognitive-organizational /
/// software-engineering) and a specific **claim-tier** on the
/// IDENTITY → MODELED-ON → ANALOGOUS → RHYMES ladder. When an ADR amendment
/// changes the ratified axis or tier, the doc-comments can drift out of sync
/// with the ADR, and nothing fires: rustdoc builds clean, tests pass.
///
/// **Observed instance** (2026-05-24, caught at sign-time on
/// `adr026-amendment-1-triage-commit-naming`): ADR-026 Amendment 1 loosened the
/// START attribution from IDENTITY-tier ("modeled on") to RHYME-tier
/// ("analogous to"). The loosening was applied at the `TriageDecision` enum
/// doc-comment but MISSED at the module-level doc-comment in the same file
/// (`vcs.rs:10` still read "modeled on the START field-triage protocol"). No
/// tool caught it; only naturalist attention at sign-time did. The site has
/// since been corrected — this antigen locks in the now-clean state so a future
/// re-drift fires.
///
/// **Sibling fail-class**: `RatifiedSpecDriftFromImpl`. Both are
/// `SubstrateAlignment` members of the **substrate-divergence** family, but they
/// are siblings, not the same antigen, because their witnesses have DIFFERENT
/// STRUCTURE (the discriminator per aristotle's recognition ruling):
/// `RatifiedSpecDriftFromImpl`'s witness is a SET-COMPARISON (does the realized
/// field-set equal the ratified field-set? binary-per-element).
/// `BiologyGroundingClaimDrift`'s witness is an AXIS-MATCH plus an
/// ORDINAL TIER-COMPARISON (is the claim on the right axis AND not exceeding the
/// ratified tier on the IDENTITY→RHYMES ladder?). A set-comparison witness sees
/// "biology paragraph present = present" and PASSES — it cannot catch the
/// IDENTITY-vs-RHYME overclaim that was the lived failure. Different witness
/// structure = sibling.
///
/// **One antigen, two hints**: the two failure modes (axis-mismatch and
/// tier-overclaim) share the witness entry-point — read the biology-grounding
/// block, extract axis + tier, compare to the ratified ADR — and branch only at
/// emission (`biology-grounding-axis-mismatch` vs
/// `biology-grounding-claim-tier-overclaim`). Same shape as the G3
/// claim-inconsistent-vs-hybrid-incomplete cross-check: one entry-point,
/// branching emission.
///
/// **The mechanization path**: v0.2 ships this antigen with an advisory,
/// recall-tuned substrate-witness — a tier-honest-verb check that catches the
/// gross case (a biology block using only IDENTITY-tier verbs where the ADR
/// ratified a RHYME-tier claim), which is exactly the lived failure. Two named
/// v0.2.x gaps (per adversarial ATK-BIOLOGY-1/2): sentence-structure overclaim
/// (a RHYME verb up front smuggling an IDENTITY claim in a subordinate clause)
/// and dual-axis false positives (a correctly-stated multi-axis claim must be
/// checked per claim-sentence, not per block). Full mechanization — machine
/// comparison against the ratified axis/tier — lands when ADRs carry a
/// structured §Biology-grounding table (axis + claim-tier per primitive), the
/// same process-amendment substrate that mechanizes the whole
/// substrate-divergence family. Same recall-scan / precision-witness split as
/// ADR-010.
///
/// **Category**: `SubstrateAlignment` — the doc-comment is a representation of
/// the ratified biology-grounding claim; the ADR §Biology grounding is the
/// actual ratified state. Divergence is a representation-vs-state split, not a
/// computation-correctness failure.
#[antigen(
    name = "biology-grounding-claim-drift",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("Biology grounding")"#,
    family = "dogfood",
    summary = "A biology-grounding doc-comment claims an axis or a claim-tier (IDENTITY/MODELED-ON/ANALOGOUS/RHYMES) that diverges from the ratified ADR §Biology grounding; load-bearing metaphor (ADR-003) drifts silently when an amendment changes the ratified axis/tier.",
    references = ["ADR-003", "ADR-024", "ADR-024#Amendment-1", "ADR-026", "ADR-026#Amendment-1", "ADR-027"]
)]
pub struct BiologyGroundingClaimDrift;

// ============================================================================
// 12. UnstableHashAsPersistedValue
// ============================================================================

/// A value that is *persisted and later compared for equality* is produced by
/// an unstable hash.
///
/// "Unstable" means the hash carries no cross-run / cross-machine /
/// cross-version stability guarantee — most commonly
/// [`std::hash::DefaultHasher`] or any `Hash`-derived digest whose algorithm
/// the standard library is free to change.
///
/// The value looks correct the moment it is written: the producer runs, emits
/// bytes, the bytes are stored. The divergence is *temporal* — it appears only
/// when the value is recomputed under a different toolchain, a different
/// platform, or a future stdlib release, and the recomputed digest no longer
/// equals the stored one. Every equality check that was meant to mean "this is
/// the same thing I recorded" silently flips to "different," with no
/// compile-time or write-time signal.
///
/// **Observed instance** (2026-05-26, this expedition): the substrate-witness
/// `signed_against_fingerprint` / `current_fingerprint` machinery (ADR-019)
/// compares a *persisted* item digest to a freshly recomputed one for
/// `against = "current"` / `fresh_within_days`. The producer
/// (`antigen-fingerprint::digest::structural_digest`) was authored to hash with
/// FNV-1a *specifically* to avoid this class: `DefaultHasher`'s output is
/// documented as unstable across Rust versions, so a digest signed today could
/// stop matching after a toolchain bump — silently breaking every signature.
/// The digest also carries a `fnv1a64:` version prefix so a future algorithm
/// change can be detected and migrated rather than producing a silent
/// mismatch.
///
/// **Defense**: for any hash whose output is persisted and later compared, use
/// a fixed-specification algorithm (FNV-1a, a SHA family, blake3) — never
/// `DefaultHasher` or an unversioned `Hash`-derived value. Prefix the stored
/// value with an algorithm tag so an algorithm change is recognizable, not
/// silent. The defending producer (`antigen_fingerprint::digest::fnv1a_64`) lives
/// in `antigen-fingerprint` which cannot depend on `antigen` (circular dep), so
/// no `#[immune]` attribute can reach it directly. The structural evidence is
/// `antigen_fingerprint::digest::fnv1a_known_vector` (pins FNV-1a to its
/// published vector) and `digest_is_deterministic` (pins cross-run stability).
/// Call-sites in `antigen::scan` that invoke `structural_digest` carry the
/// downstream evidence.
///
/// **Category**: `SubstrateAlignment` — the persisted digest is a
/// *representation* of an item's structure; an unstable hash makes that
/// representation diverge from what recomputation produces, even though the
/// item itself never changed. The failure is representation-vs-state drift over
/// time, not a wrong computation at a single point.
#[antigen(
    name = "unstable-hash-as-persisted-value",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("DefaultHasher")"#,
    family = "dogfood",
    summary = "A persisted-and-later-compared value is produced by a hash with no cross-run/version stability guarantee (e.g. DefaultHasher); the stored digest silently stops matching recomputation under a different toolchain — representation drifts from state over time.",
    references = ["ADR-019", "ADR-005"]
)]
pub struct UnstableHashAsPersistedValue;

// ============================================================================
// 13. AuditFingerprintSelfReferential
// ============================================================================

/// A staleness-detection mechanism compares a stored value against *itself*
/// rather than against the actual current state.
///
/// The classic instance: an audit reads `current_fingerprint` from a stored
/// sidecar and compares each signer's `signed_against_fingerprint` to it — but
/// both sides come from the *same file*. A signer who signed against an old
/// fingerprint is compared to that same old fingerprint, so the staleness check
/// always clears. The mechanism looks like it detects drift; it structurally
/// cannot, because nothing in the comparison ever touches the live code.
///
/// This is distinct from [`UnstableHashAsPersistedValue`]: there the *hash* is
/// the problem (an unstable algorithm makes a stable comparison wrong over
/// time); here the *comparison topology* is the problem (a self-referential
/// comparison is wrong even with a perfectly stable hash). It is also a cousin
/// of [`AuditHintWithNoUpstreamPreconditionCheck`] — both check only one side
/// of a relationship — but the witness shape differs: that one omits an
/// upstream edge; this one folds both ends of the comparison onto one source.
///
/// **Observed instance** (2026-05-26, this expedition; resolved same day): the
/// substrate-witness audit (`audit_substrate_witness` in `antigen/src/audit.rs`)
/// originally passed the sidecar's stored `current_fingerprint` into the
/// predicate evaluator, so `signed_against_fingerprint == current_fingerprint`
/// compared the sidecar to itself — "Audit-SF-1." Real code drift was never
/// detected. The fix (A3) feeds the *scan-recomputed* `structural_fingerprint`
/// (from `antigen_fingerprint::structural_digest`, which reads the item on disk)
/// instead, so a signer who signed against stale code is now correctly rejected.
///
/// **Defense**: a staleness or drift check must compare the stored value
/// against a value *recomputed from the live source*, never against a sibling
/// field of the same stored record. Audit it: if both operands of the
/// comparison trace back to the same file with no disk read between, the check
/// is self-referential and vacuous.
///
/// **Category**: `FunctionalCorrectness` — the audit produces a *wrong result*
/// (staleness always clears, false-green), not a representation-vs-state layer
/// split. The verb's output is incorrect; that is correctness, not
/// substrate-alignment.
#[antigen(
    name = "audit-fingerprint-self-referential",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("Audit-SF-1")"#,
    family = "dogfood",
    summary = "A staleness-detection mechanism reads the comparison's 'current' value from the same stored record as the signed-against value (rather than recomputing from live code), so the check compares a value against itself and staleness always clears — false-green.",
    references = ["ADR-019", "ADR-005"]
)]
pub struct AuditFingerprintSelfReferential;

// ============================================================================
// 14. SilentSemanticMismatchAtTrustBoundary
// ============================================================================

/// Accept→bind-wrong→succeed→fail-downstream: trust-boundary input silently bound to the wrong slot.
///
/// A trust-boundary input is accepted (no parse or validation error) but is
/// silently bound to a slot where it produces no semantic effect — or the wrong
/// one — with failure surfaced only much later as an opaque downstream error.
///
/// The defining shape: **accept → bind-wrong → succeed → fail-downstream**.
///
/// 1. The tool accepts the input at the trust boundary (scaffold/sign/compile).
/// 2. The input is bound to a slot where the semantic contract is different from
///    what the adopter intended (e.g., a role string bound to a signer-name slot,
///    an empty string bound to a fingerprint slot where the predicate requires a
///    real digest, a field name that doesn't exist in the schema).
/// 3. The immediate verb reports success — no error, no warning.
/// 4. Audit (or a later read) fails with an opaque error
///    (`DisciplinePredicateFailed`, `NoSignersRequired`, `SignerNotFound`) that
///    names the downstream symptom, not the root cause.
///
/// This is ADR-005 sub-clause F applied recursively: trust must not be extended
/// at a boundary before the input is validated into an *honorable shape*. An
/// input that PARSES but carries the *wrong semantic meaning for the intended
/// slot* is not yet in an honorable shape.
///
/// **Five observed instances in this expedition (aristotle convergence, 2026-05-26)**:
/// - **Finding 3**: a signed `.attest/` sidecar is accepted for a `witness=`
///   immune site; the sidecar binds to a slot that audit's code-witness branch
///   never reads, so the attestation is silently uncredited.
/// - **Finding 5**: `signers(required = ["math-researcher"])` — the role string
///   is bound to the signer-name slot; audit matches by name, not role, so the
///   predicate always fails with no hint that the name/role confusion is the cause.
/// - **Finding 6**: `attest scaffold --fingerprint ""` — empty string accepted;
///   the predicate later fails because an empty fingerprint can never match a
///   real digest, with no diagnostic linking the cause.
/// - **Finding 7** (meta-instance): `attest check` reports only
///   `DisciplinePredicateFailed` at the tree level; per-leaf `expected/found`
///   is absent, so adopters cannot distinguish which of the above cases is their
///   root cause. This is the *diagnostic gap* that makes the other four silent.
/// - **Finding 8**: `attest sign --fingerprint ""` — empty fingerprint accepted,
///   the signer entry is written; a subsequent `against="current"` predicate
///   will always fail because the stored digest cannot match.
///
/// **Why F7 is the meta-instance**: the other four failures produce WRONG DATA
/// at the boundary; F7 produces MISSING DIAGNOSTICS at audit time. F7 built
/// first turns the others from "20-minute source dive" into "5-second read" —
/// `signers(required=[X]): FAIL — no signer named X (found: [...]; X is a role?)`
/// is the kind of per-leaf message that makes the root cause self-describing.
///
/// **Defense posture**:
/// 1. **Validate the semantic contract at the trust boundary**, not just the
///    parse-level shape. An empty fingerprint is syntactically valid; it is
///    semantically wrong for any `against="current"` slot.
/// 2. **Emit a per-leaf expected-vs-found diagnostic** at audit time so the
///    failure is named where it was introduced, not buried in a tree-level fail.
/// 3. **Sub-clause F at every new slot**: when adding a field that binds
///    user-supplied input, enumerate the honorable shapes and reject everything
///    outside that set at acceptance time, not at audit time.
///
/// **Category**: `FunctionalCorrectness` — the tool produces an incorrect
/// result (silent success, then opaque failure), not a representation-alignment
/// failure. The input binding is wrong; the downstream failure is the observable
/// consequence.
#[antigen(
    name = "silent-semantic-mismatch-at-trust-boundary",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([item = fn, doc_contains("trust boundary")])"#,
    family = "dogfood",
    summary = "A trust-boundary input is accepted (no parse error) but bound to a semantically wrong slot; the immediate verb succeeds, and failure surfaces only downstream as an opaque DisciplinePredicateFailed or similar, with no link back to the root cause at the boundary.",
    references = ["ADR-005", "ADR-019", "ADR-024"]
)]
pub struct SilentSemanticMismatchAtTrustBoundary;

// ============================================================================
// 15. DeclaredCapabilityWithNoProductionPath
// ============================================================================

/// A capability is declared in the type system or documentation but no code path can ever make it fire.
///
/// The structural shape: **declare → no-impl → silent-void**. Something is named and typed as
/// a real capability (a witness, an audit hint, a DSL field, a documented behavior), but the
/// implementation path that would exercise it either was never written or was silently removed.
/// The declaration compiles, tools report no error, and callers proceed under the assumption
/// that the capability is live — but it never fires.
///
/// This is antigen's own generation-inspection asymmetry turned inward: antigen's type surface
/// grows faster than the production paths that exercise it, exactly as agentic dev generates
/// code faster than teams can inspect it. The fail-class antigen exists to prevent IN ADOPTERS
/// is the fail-class most likely to hit antigen's own codebase through the same mechanism.
///
/// **Four observed instances (pathmaker structural rhyme, 2026-05-26)**:
/// - **Never-resolving witness**: `#[immune(X, witness = "fn_name")]` compiles; audit can
///   never resolve a string to a function reference. Silent void from compile-time forward.
/// - **Never-credited sidecar**: a signed `.attest/` sidecar exists for a `witness=` site;
///   the audit code-witness branch never reads sidecars, so the attestation is never credited.
/// - **Never-reachable DSL field**: `Leaf::Signers` had `signature_allow` and `signature_prefer`
///   fields; `parse_signers()` didn't expose them, so no adopter expression could ever set them.
///   (This instance is shared with [`CapabilityOmissionAtLowering`] (#16): the *same* defect seen
///   from two layers — #15 read-side "the field is never *reachable* from the surface" ∩ #16
///   write-side "a parsed value never *reaches* the runtime because lowering omits it". One defect,
///   two class-memberships — the F10 fundamentality-test's predicted multi-membership for
///   description-tier classes; evidence both classes are real, not redundant.)
/// - **Never-emitted `AuditHint` variant**: `PolyclonalInsufficientLineages` and
///   `AdccSingleMechanismOnly` exist in the `AuditHint` enum but are never constructed anywhere
///   in the codebase; the rustdoc described them as real behavior.
///
/// **Defense posture**:
/// 1. **Dead-declaration sweep**: at audit time, verify every `AuditHint` variant is constructed
///    somewhere; every DSL field is reachable from the public macro surface; every witness
///    identifier resolves to a real fn; every sidecar that exists for an immune site is read.
/// 2. **No-overclaim in rustdoc**: when a feature is planned but not emitted, say so explicitly
///    ("planned — not yet emitted at v0.2"); do not describe future behavior as present behavior.
/// 3. **Production-path test**: for every declared capability, a test must exercise the code
///    path that makes it fire (not just that it compiles).
///
/// **Category**: `SubstrateAlignment` — the representation (type declaration, rustdoc, `AuditHint`
/// variant) diverges from the actual production state (never fires). The declared surface and the
/// executed surface are out of alignment.
///
/// **Fingerprint v0.2 limit (recall is via the explicit marker only)**:
/// `doc_contains("planned")` is a recall PLACEHOLDER, not a working structural
/// matcher for this class — same v0.2-DSL limit as
/// [`ScanVisitorDigestAssignmentOmission`] (#19). Verified via `cargo antigen
/// scan`: ZERO independent fingerprint matches; the one match (the `AuditHint`
/// enum at `audit.rs`) is the explicit `#[presents]` marker, which is the
/// correct + only placement. Two reasons the per-variant instances
/// (`PolyclonalInsufficientLineages`, `AdccSingleMechanismOnly`) are
/// unreachable: (a) `doc_contains` is case-sensitive (`matcher.rs` —
/// `doc_text().contains(needle)`) and the annotations use "Planned" (capital);
/// (b) more fundamentally, `doc_text` reads only TOP-LEVEL item docs and
/// synthesis dispatches on top-level `syn::Item` kinds, so enum-VARIANT docs are
/// never reached regardless of case. So coverage of this class rests on the
/// explicit `#[presents]` marker on the enclosing TYPE, not fingerprint
/// synthesis. A grammar that could reach variant-level / body-content recall is
/// tracked at `forward/fingerprint-grammar-body-content-with-negation`; refine
/// when it lands.
#[antigen(
    name = "declared-capability-with-no-production-path",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("planned")"#,
    family = "dogfood",
    summary = "A capability (witness, DSL field, audit hint, documented behavior) is declared in the type system or docs but no code path can ever make it fire; the declaration compiles and reports no error while the capability is permanently void.",
    references = ["ADR-004", "ADR-006", "ADR-019"]
)]
pub struct DeclaredCapabilityWithNoProductionPath;

// ============================================================================
// 16. CapabilityOmissionAtLowering
// ============================================================================

/// A surface field is accepted but silently dropped at lowering.
///
/// The layer that translates the surface form into the runtime form hardcodes a
/// default instead of threading the parsed value through, so a field that
/// parses and type-checks never reaches the engine.
///
/// The shape: **surface accepts → lowering discards → runtime sees the default**.
/// Distinct from [`DeclaredCapabilityWithNoProductionPath`] (#15), which is the
/// read-side "a declared capability can never *fire*"; this is the write-side
/// "a parsed value never *reaches* the runtime because the lowering omits it."
/// The adopter's input vanishes between the surface and the engine, with no
/// parse error and no warning — the most insidious form because the surface
/// *looks* like it accepted the value.
///
/// Per aristotle's F5 ratification this is a child of [`SilentIntentNullification`],
/// distinguished from its sibling [`ActiveArgumentDiscard`] by witness-structure:
/// `ActiveArgumentDiscard` *silently swallows tokens at parse* (behavioral witness:
/// feed args, assert compile-error); this one *silently drops at lowering*
/// (structural witness: the parity test). Both are silent — they differ by
/// witness-structure, not by loud-vs-silent.
///
/// **Observed instance** (2026-05-25, this expedition; fixed in commit c237101):
/// the `signers()` substrate-witness DSL did not parse `signature_allow` /
/// `signature_prefer`, and `to_leaf()` hardcoded `signature_allow = Vec::new()`
/// / `signature_prefer = None` regardless of input (parser.rs:317-318).
/// decisions.md:5085 RATIFIES the grammar as `signers(required, roles?,
/// against?, signature_allow?, signature_prefer?)` — five fields — so two
/// ratified fields were silently inexpressible: a `RatifiedSpecDriftFromImpl`
/// against decisions.md:5085 AND this capability-omission-at-lowering. The fix
/// added the parse arms + threaded both fields through the lowering.
///
/// **Defense**: a parity guard — construct a runtime value with EVERY field set
/// to a non-default, render it to the surface form, parse + lower it back, and
/// assert equality. Any field the lowering drops breaks the test at the parser,
/// not at an adopter whose value vanished. The witness is
/// `atk_dsl_signers_every_field_reachable_and_lowered_no_omission` in
/// `antigen-attestation::parser` tests.
///
/// **Category**: `FunctionalCorrectness` — the lowering *computes a wrong
/// (lossy) value*: it produces a runtime form that does not faithfully represent
/// the parsed surface. This is a wrong-output failure, not a
/// representation-vs-state layer split.
#[descended_from(SilentIntentNullification)]
#[antigen(
    name = "capability-omission-at-lowering",
    category = AntigenCategory::FunctionalCorrectness,
    // Structural recall-fingerprint: the lowering site is the impl block that
    // defines `to_leaf` (e.g. `impl LeafExpr { fn to_leaf(&self) -> Leaf }`).
    // The earlier `doc_contains("to_leaf")` under-covered the class — "to_leaf"
    // lives in the method name, not in any /// doc attr, so it matched nothing
    // (its own AntigenFingerprintDivergesFromClassExtension, caught by ATK).
    fingerprint = r#"all_of([item = impl, has_method("to_leaf", "(&self) -> crate::Leaf")])"#,
    family = "dogfood",
    summary = "A surface field parses + type-checks but the lowering step hardcodes a default instead of threading the parsed value through; the adopter's input is silently dropped between surface and runtime, with no parse error. Child of SilentIntentNullification (F5), sibling of ActiveArgumentDiscard — both silent, differing by witness-structure (parse-swallow/behavioral vs lowering-drop/structural).",
    references = ["ADR-007", "ADR-019", "decisions.md:5085"]
)]
pub struct CapabilityOmissionAtLowering;

// ============================================================================
// 17. AntigenFingerprintDivergesFromClassExtension
// ============================================================================

/// A failure-class fingerprint is fitted to the shape of the first observed
/// instance rather than to the full extension of the class.
///
/// When a fingerprint is authored on the *self-application path* — the path
/// the author already walks — it is shaped to the instances that path reaches.
/// The fingerprint binds those instances precisely, while silently missing
/// every instance in the *geometric complement* of the author's path. The
/// match-set (what the fingerprint selects) diverges from the class's true
/// extension (every site that genuinely presents the fail-class).
///
/// Two divergence directions, both observed live:
///
/// - **Under-coverage (too narrow)**: `ActiveArgumentDiscard`'s fingerprint
///   (`all_of([item = impl, doc_contains("forward compat")])`) was fitted to
///   the `*Args::parse` discard-loop. It cannot match `parse_signers()` — a
///   real second instance of the same class — because `parse_signers` is a
///   `fn` item with no "forward compat" in its docs.
///
/// - **Over-coverage (too broad)**: `RatifiedSpecDriftFromImpl`'s fingerprint
///   (`doc_contains("ADR-")`) matches all 21 examples regardless of drift-risk.
///   It is fitted to the doc-mention, not the drift-risk, so it fires as noise
///   at the 20 non-drifted examples and the adopter cannot distinguish signal
///   from recall.
///
/// **The two directions are NOT equally severe** (the F8 severity-asymmetry, the
/// one thing the symmetric name "Diverges" must not flatten): under-coverage is a
/// **false negative — HIGH severity**. A real instance escapes the fingerprint and
/// ships undefended; a missed fail-class shipping silently is the exact thing
/// antigen exists to prevent. Over-coverage is a **false positive — advisory
/// severity**. It is noise (flag for tightening) and costs adopter trust, but it
/// is *safe* — nothing ships undefended. Audit support should rank under-coverage
/// above over-coverage accordingly (direction-discriminated hints, under = HIGH /
/// over = advisory).
///
/// The biological cognate is **original antigenic sin**: a recognition receptor
/// imprinted on the first-seen antigen strain binds that strain with high
/// affinity while under-binding the variant. The immune system's answer is
/// *affinity maturation* under exposure to multiple strains; antigen's answer
/// is the same — recruit a second body (a structurally different adopter, an
/// adversarial sweep, a coverage pass) whose incidental finds are an unbiased
/// sample of the class's true reach.
///
/// **Why this is hard to catch on the self-application path**: the fingerprint
/// author's fluency and the fingerprint's blind spot share a body — you cannot
/// see the geometric complement from inside. See `docs/testing-patterns.md §
/// Fingerprint authoring discipline` for the discipline.
///
/// **Observed instances** (2026-05-26, antigen-dx-dogfood expedition, camp's
/// binary-adopter coverage sweep): both directions surfaced when a second body
/// (camp's coverage sweep) reached instances the self-application path had
/// never walked. The under-coverage case (`ActiveArgumentDiscard`) was found
/// by the scout's systematic coverage pass; the over-coverage case
/// (`RatifiedSpecDriftFromImpl`) was identified by outsider's signal-vs-noise
/// analysis of scan output.
///
/// **Defense**: author fingerprints to the *class's extension* (the full set
/// of sites that present the class), not to the first instance. Use a broad
/// recall-fingerprint + precise witness (the structural parity-test that does
/// the exact discrimination) rather than a precise fingerprint that mistakes
/// instance-shape for class-shape. Recruit a second body for coverage review.
///
/// **Category**: `SubstrateAlignment` — the fingerprint is a *representation*
/// of the class's extension; a fingerprint shaped to instance-#1 misrepresents
/// the extension (it says "these and only these sites present the class" when
/// the actual extension is larger or differently bounded). The mismatch is
/// representation-vs-state, not a computation-correctness failure.
#[antigen(
    name = "antigen-fingerprint-diverges-from-class-extension",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("fingerprint")"#,
    family = "dogfood",
    summary = "A failure-class fingerprint is fitted to the shape of the first observed instance rather than to the full extension of the class, producing silent under-coverage (too narrow) or noisy over-coverage (too broad). The match-set (what the fingerprint selects) diverges from the class's true extension.",
    references = ["ADR-006", "ADR-010", "docs/testing-patterns.md"]
)]
pub struct AntigenFingerprintDivergesFromClassExtension;

// ============================================================================
// 18. ParallelStateTrackersDiverge
// ============================================================================

/// Two (or more) hand-maintained representations of the same fact are kept "in
/// sync" by convention rather than by a mechanism, and they drift.
///
/// One source of truth is duplicated into a second tracker — a mirrored enum, a
/// string-const shadowing an enum's serde keys, a version string copied across
/// docs, a doc cross-reference pointing at content that must exist elsewhere —
/// with a comment promising the two "must stay in sync." But a comment enforces
/// nothing. Nothing fails when they diverge; the drift is silent until a reader
/// trusts the stale copy.
///
/// **Observed instances** (2026-05-26, antigen-dx-dogfood expedition; the
/// recurrence that motivated declaring this class):
///
/// - **const ↔ enum (the canonical instance)**: `ADR025_AUDIT_HINTS` (a
///   hand-typed `&[&str]` of supply-chain hint strings) shadowed the `AuditHint`
///   enum's serde keys with only a "MUST match … exactly" comment. It had
///   *already* drifted — the const listed 15 entries while the enum had 16
///   supply-chain variants — and no test caught it (the prior tests checked the
///   const against itself). The fix is this antigen's witness (below).
/// - **enum ↔ enum**: `antigen::audit::WitnessTier` is hand-mirrored from
///   `antigen_attestation::tier::WitnessTier` (deliberate, to keep the on-disk
///   audit format serde-stable while the attestation crate evolves) — `tier.rs`
///   even names "the `ParallelStateTrackersDiverge` shape." A test
///   (`atk_witness_tier_parity`) is what actually enforces it; the comment alone
///   did not (the two had drifted on the `Hash` derive).
/// - **doc-string ↔ docs**: a version string hand-copied across README /
///   quickstart / tutorial drifted (README pinned `rc.2` while the others said
///   `rc.3`); a glossary cross-reference promised category-mapping content the
///   target entry did not yet carry.
///
/// **The shared shape**: a comment ("must stay in sync", "kept in lock-step",
/// "see X for Y") substitutes for a mechanism. The comment is a *promise of
/// enforcement that delivers none*. Antigen's whole posture is moving failure-
/// class memory from comment-tier (drift-prone) to structural-tier (enforced) —
/// so a comment-enforced mirror in antigen's own substrate is the project
/// failing to eat its own cooking. That recursion is why it earns declaration.
///
/// **Defense (the witness)**: replace the hope-comment with a test that reads
/// *both* sides and asserts they agree. The canonical witness is
/// `adr025_audit_hints_const_matches_enum_serde_keys` (a serde-key *bijection*:
/// every enum variant's key is in the const, every const entry maps to a real
/// variant, and the lengths are coupled) — a rename, a missing entry, or a dead
/// entry now FAILS the test instead of drifting silently. The general defense:
/// derive the second tracker from the first where a derive macro can express the
/// relation, or — where the relation needs human classification a derive can't
/// capture — a bijection/parity test that reads both sides. A "MUST stay in
/// sync" comment with no such test is the un-defended shape.
///
/// **Category**: `SubstrateAlignment` — each tracker is a *representation*; when
/// the copies diverge, one representation no longer matches the fact the other
/// asserts. The failure is representation-vs-state divergence, not a
/// computation-correctness error. Recognized fingerprint-only (like
/// `BiologyGroundingClaimDrift`): the "in sync / lock-step" comment is the
/// recall surface; the bijection/parity test is the precision witness.
#[antigen(
    name = "parallel-state-trackers-diverge",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("lock-step")"#,
    family = "dogfood",
    summary = "Two hand-maintained representations of the same fact (a mirrored enum, a const shadowing an enum's serde keys, a version string copied across docs, a doc cross-reference) are kept in sync by a comment rather than a mechanism, and drift silently. A comment promising 'must stay in sync' enforces nothing; only a test reading both sides catches the divergence.",
    references = ["ADR-004", "ADR-028"]
)]
pub struct ParallelStateTrackersDiverge;

// ============================================================================
// 19. ScanVisitorDigestAssignmentOmission
// ============================================================================

/// A `ScanVisitor` `visit_*` method calls `check_attrs` without first setting
/// `self.current_item_digest`, causing the preceding item's digest to bleed into
/// the new item's fingerprint.
///
/// `ScanVisitor` maintains a `current_item_digest: String` field that holds the
/// structural digest of the item currently being visited. Every `visit_*` method
/// that routes attributes through `check_attrs` MUST set this field first —
/// `self.current_item_digest = antigen_fingerprint::structural_digest(item)` —
/// or the scan output carries the PREVIOUS item's digest, producing non-empty but
/// wrong fingerprints that silently pass all schema checks.
///
/// **Observed instances** (2026-05-26, antigen-dx-dogfood expedition, fe6a3a0):
/// Three new visitor methods (`visit_item_const`, `visit_item_static`,
/// `visit_impl_item_const`) were added to cover previously-blind item kinds.
/// Each correctly called `check_attrs` but omitted the mandatory
/// `self.current_item_digest = …` prefix. The bug produced contaminated
/// fingerprints — wrong, not empty — that passed every downstream assertion.
/// No compile error, no test failure at addition time; only an explicit
/// adversarial ATK pass detected the contamination.
///
/// **Why preemptive**: this antigen is declared before the next occurrence, not
/// after. The visitor-extension pattern is structurally guaranteed to recur:
/// `ScanVisitor` adds a new item kind → developer copies the `visit_*` template →
/// the `check_attrs` call comes from the template, but the digest assignment
/// requires knowing the invariant. The coupling between "calls `check_attrs`" and
/// "must first set `current_item_digest`" is an ordering constraint with no
/// compile-time enforcement. A proc-macro or derive-helper approach could enforce
/// it structurally; until then, this antigen is the class's memory.
///
/// **The fingerprint — and its v0.2 limit (recall is via the explicit marker
/// only)**: `doc_contains("check_attrs")` is a recall PLACEHOLDER, not a working
/// structural matcher for this class. Verified against `cargo antigen scan` (the
/// right gate): it produces ZERO independent fingerprint matches — the one match
/// on `ScanVisitor` is the explicit `#[presents]` marker (the doc string
/// `check_attrs` lives in field/method docs that `doc_text()` does not read, and
/// the `check_attrs` call sites are method *bodies*, not item docs). So coverage
/// of this class rests entirely on the explicit `#[presents]` marker on
/// `ScanVisitor`, NOT on fingerprint synthesis. That is honest for v0.2: the real
/// recall target — "a `visit_*` method that calls `check_attrs` *without first*
/// calling `structural_digest`" — is a body-content-with-ordering-and-negation
/// pattern the v0.2 fingerprint DSL cannot express (the leaves are item-shape +
/// `has_method` + `doc_contains`, with no `body_contains_call` or call-ordering
/// predicate). `has_method("check_attrs")` would not help — it matches the single
/// *definer*, not the ~14 *callers*. The grammar extension that would make
/// structural recall real is tracked at
/// `forward/fingerprint-grammar-body-content-with-negation`; refine this
/// fingerprint when it lands. The witness
/// (`atk-digest-1-antigen-owned-attrs-incomplete`, closed) verified the
/// contamination MECHANISM directly: two visitor methods with and without the
/// digest assignment, with an ATK fixture exercising both, confirmed the bleed.
///
/// **Internal-tooling discipline**: per `feedback_internal_tool_antigens_preemptive`,
/// internal-tooling antigens are declared preemptively from analog patterns and
/// predictable-from-shape fail-classes, not from post-occurrence recovery. No
/// biology analogue is required (see dogfood module header: "internal-tooling
/// dogfood antigens rely on structural prediction rather than biological metaphor").
///
/// **Category**: `FunctionalCorrectness` — the visitor produces wrong output
/// (contaminated fingerprint) rather than diverging representation-vs-state.
/// The error is a computation-correctness failure at scan time, not a
/// substrate-alignment gap.
#[antigen(
    name = "scan-visitor-digest-assignment-omission",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("check_attrs")"#,
    family = "dogfood",
    summary = "A ScanVisitor visit_* method calls check_attrs without first setting self.current_item_digest, causing the preceding item's digest to contaminate the new item's fingerprint. The ordering constraint (digest-then-attrs) has no compile enforcement; the bug produces wrong but non-empty fingerprints that pass schema checks silently.",
    references = ["ADR-010"]
)]
pub struct ScanVisitorDigestAssignmentOmission;

// ============================================================================
// 20. FailingTestWithoutIgnorePin
// ============================================================================

/// A failing adversarial test committed to `HEAD` without `#[ignore]`, breaking
/// `cargo test --workspace` for the whole team until the fix lands.
///
/// **The discipline**: adversarial tests document blind spots by asserting what
/// SHOULD be true — and intentionally failing until the gap is closed. The
/// correct commit pattern is one of:
///
/// 1. `#[ignore]` at commit time, `#[ignore]` removed with the fix commit.
/// 2. Test + fix committed atomically (single commit, no intermediate red state).
///
/// Committing a failing test without `#[ignore]` produces a red HEAD that blocks
/// every other team member's `cargo test --workspace` until the fix lands — even
/// if the fix is in a dirty working tree or in-flight on a different agent.
///
/// **Observed instances** (2026-05-26, antigen-dx-dogfood expedition, three
/// consecutive P0s):
///
/// 1. `89f8108` — enum-variant presents blind spot. Test committed FAILING;
///    fixed at `d97c204`.
/// 2. `6a17036` — impl/trait-item-macro presents blind spot. Both
///    `atk_a2_impl_item_macro` and `atk_a2_trait_item_macro` committed FAILING;
///    fixed at `931ae89`.
/// 3. `477aeef` — const-synthesis fingerprint miss. Test committed FAILING;
///    fix was in the dirty working tree; fixed in the same session.
///
/// All three are the same shape: the adversarial intent is correct ("pin the
/// gap"), the mechanism is wrong ("break CI for the team"). The pattern recurred
/// three times in a single expedition despite all three being avoidable with the
/// same one-line fix (`#[ignore]`).
///
/// **Category**: `SubstrateAlignment` — the committed substrate (`git log`) claims
/// `cargo test --workspace` is green; it is not. The representation (committed
/// test suite) diverges from the actual verification state (broken workspace).
///
/// **Fingerprint**: `doc_contains("STATUS: FAILING")` — adversarial tests that
/// follow the ATK convention of documenting their intended-failing state in the
/// doc comment are surfaced. Combined with manual review of whether `#[ignore]`
/// is present, this provides partial passive recall.
///
/// **Fingerprint limitation** (v0.2): the ideal fingerprint is
/// `all_of([doc_contains("STATUS: FAILING"), not(has_attribute("ignore"))])` —
/// but `not(has_attribute(...))` is not yet in the v0.2 DSL. The current
/// fingerprint fires on ALL "STATUS: FAILING" tests, including correctly
/// `#[ignore]`d ones. Correctly-handled sites should carry
/// `#[immune(FailingTestWithoutIgnorePin, witness = ...)]`. The
/// `not(has_attribute)` predicate is the v0.3 fingerprint tightening path.
///
/// **Internal-tooling discipline**: per `feedback-internal-tool-antigens-preemptive`,
/// declared preemptively from the three confirmed instances rather than waiting
/// for a fourth.
#[antigen(
    name = "failing-test-without-ignore-pin",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("STATUS: FAILING")"#,
    family = "dogfood",
    summary = "A failing adversarial test is committed to HEAD without #[ignore], \
               breaking cargo test --workspace for the team until the fix lands. \
               The correct discipline is #[ignore] at commit time, removed atomically \
               with the fix, or test+fix in a single atomic commit.",
    references = ["ADR-028"]
)]
pub struct FailingTestWithoutIgnorePin;

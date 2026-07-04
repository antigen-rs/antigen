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
//! - `MarkerStructDeadCodeInBinary` — `FunctionalCorrectness`: an `#[antigen]`
//!   marker struct trips `dead_code` in a binary-crate adopter because `pub`
//!   does not exempt never-constructed declaration tokens there; the macro's
//!   `const _: fn()` use-token emit is the corrected verb behavior.
//!
//! ## Proc-macro-crate markability boundary
//!
//! A structural fact several of these antigens surface: `antigen-macros` is a
//! proc-macro crate that **cannot carry `#[antigen]` / `#[presents]` /
//! `#[immune]` markers** — it does not (and cannot) depend on `antigen` (the
//! reverse dependency would be a cycle), so the marker types are unreachable,
//! and a proc-macro crate cannot self-apply its own attribute macros. The crate
//! that *implements* a defense (e.g. the `MarkerStructDeadCodeInBinary`
//! use-token in `antigen-macros/src/lib.rs`) is therefore the crate that
//! *cannot mark* it. Coverage for macros-crate sites lives at the
//! fingerprint-recall layer instead: the dogfood declarations here carry
//! fingerprints (e.g. `ActiveArgumentDiscard`'s `Parse`-impl recall) that
//! `cargo antigen scan` matches against the macros-crate source passively, with
//! no explicit marker required at the site.
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
    summary = "A defense declaration names a witness that does not execute a meaningful verification — the declared defense and the actual verification state diverge.",
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
// Fingerprint-recall note (dogfood, INV — this class ate its own dog food): the
// original `doc_contains("silently nullified")` matched ONLY this class's own
// definition doc — it could not recall ANY real instance (the F4/under-coverage
// direction, an instance of AntigenFingerprintDivergesFromClassExtension #19). The
// real extension says "silent-miscalibration" (the ADWIN δ-clamp / UnderPowered
// guard in learn/adwin.rs::detect — a blind-axis intent silently collapsed into a
// confident NoDrift is exactly this class). So the fingerprint widens to a
// broad-RECALL `any_of` over the surface phrasings the class actually uses; precision
// stays in the witness (the F8 recall/precision split this family preaches). The
// authoritative carrier is the explicit `#[presents(SilentIntentNullification)]` on
// adwin::detect; this fingerprint is the recall net for the as-yet-undeclared.
#[antigen(
    name = "silent-intent-nullification",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"any_of([doc_contains("silently nullified"), doc_contains("silent-miscalibration"), doc_contains("silently miscalibrate")])"#,
    family = "dogfood",
    summary = "A surface appears to accept or honor an adopter's declared intent but does not realize it; the intent is silently nullified between declaration and effect. Parent of ActiveArgumentDiscard (parse-side, behavioral witness) and CapabilityOmissionAtLowering (lowering-side, structural witness), which differ by witness-structure — both are silent. Recall covers the silent-miscalibration phrasing (a blind detector axis collapsed into a confident verdict — the ADWIN instance).",
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
// ADVISORY: when the canonical axis-match + tier-comparison witness lands as an
// in-repo immunity, G2 will fire antigen-category-claim-inconsistent — same
// locus as ParallelStateTrackersDiverge. The canonical witness is a code-test
// (reads the biology block + compares to the ratified ADR axis/tier), not an
// external-substrate predicate. ADR-028 Amendment 2 doesn't model this locus.
// Pending ADR-028 Amendment (v0.2.x): SubstrateAlignment splits on
// witness-locus = {external-substrate (predicate) | in-repo-parity (test)}.
// See findings/category-witness-crosscheck-vs-fingerprint-only-stdlib.
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
/// detected. The fix feeds the *scan-recomputed* `structural_fingerprint`
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
// ADVISORY: G2 fires antigen-category-claim-inconsistent because the canonical
// witness is a bijection/parity test (in-repo-parity code-witness), not an
// external-substrate predicate. ADR-028 Amendment 2 defines substrate-witness
// as reading external substrate state; it did not anticipate in-repo
// representation-divergence defended by a code-test. G2 is correct per the
// ratified ADR; the ratified ADR is incomplete on this locus.
// Pending ADR-028 Amendment (v0.2.x): SubstrateAlignment splits on
// witness-locus = {external-substrate (predicate) | in-repo-parity (test)}.
// See findings/category-witness-crosscheck-vs-fingerprint-only-stdlib.
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
/// **Fingerprint**: `all_of([doc_contains("STATUS: FAILING"), not(attr_present("ignore"))])` —
/// fires only on tests that are documented as failing (the ATK convention of
/// recording intended-failing state in the doc comment) AND are *not* pinned with
/// `#[ignore]`. The `not(attr_present("ignore"))` leaf is the precision that the
/// earlier broad `doc_contains("STATUS: FAILING")` form lacked: correctly-`#[ignore]`d
/// tests no longer false-positive, so no manual review of the `#[ignore]` state is
/// needed. (`not` + `attr_present` are both v0.2 DSL operators; `attr_present`
/// matches `#[ignore]` as a top-level item attribute on the test fn.) A site that
/// must remain failing long-term even when correctly handled should carry
/// `#[immune(FailingTestWithoutIgnorePin, witness = ...)]`.
///
/// **Internal-tooling discipline**: per `feedback-internal-tool-antigens-preemptive`,
/// declared preemptively from the three confirmed instances rather than waiting
/// for a fourth.
#[antigen(
    name = "failing-test-without-ignore-pin",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"all_of([doc_contains("STATUS: FAILING"), not(attr_present("ignore"))])"#,
    family = "dogfood",
    summary = "A failing adversarial test is committed to HEAD without #[ignore], \
               breaking cargo test --workspace for the team until the fix lands. \
               The correct discipline is #[ignore] at commit time, removed atomically \
               with the fix, or test+fix in a single atomic commit.",
    references = ["ADR-028"]
)]
pub struct FailingTestWithoutIgnorePin;

// ============================================================================
// 21. MarkerStructDeadCodeInBinary
// ============================================================================

/// An `#[antigen]` marker struct trips `dead_code` when an adopter declares it
/// in a *binary* crate, because the marker is a declaration token that antigen
/// never constructs and `pub` does not exempt it.
///
/// A marker struct (`#[antigen] pub struct Foo;`) carries no data and is used
/// purely as a failure-class identity token — its `TypeId` / name is the value,
/// never an instance. In a library crate `pub` exempts it from `dead_code`
/// (it is part of the external API surface). In a *binary* crate there is no
/// external API surface, so `pub` does not exempt it, and the never-constructed
/// marker trips the `dead_code` lint — which is `-D warnings` in any adopter
/// running a strict clippy/CI gate. The adopter did nothing wrong; the macro's
/// emitted output broke their build.
///
/// **Observed instance** (2026-05-22 → 2026-05-24, camp's adoption of antigen):
/// camp is a binary crate. Its first `#[antigen]` declarations tripped
/// `dead_code` on every marker struct — Finding 1 of camp's hardcore-adoption
/// dogfood. The fix lives in the `#[antigen]` macro itself
/// (`antigen-macros/src/lib.rs`): the expansion emits a zero-cost use-token
/// alongside the marker —
///
/// ```ignore
/// const _: fn() = || { let _antigen_use_token: Foo; };
/// ```
///
/// The `const _` is anonymous (no namespace pollution), always compiled (not
/// `#[cfg(test)]`-gated), and the closure body is never invoked — the binding
/// only references the type, so the marker is genuinely "used" from the
/// compiler's view at zero runtime cost. This is strictly better than
/// `#[allow(dead_code)]`, which would also mask legitimate dead-code elsewhere
/// on the item.
///
/// **Category**: `FunctionalCorrectness` — the `#[antigen]` macro is a verb
/// (code generator); the fail-class is the verb producing output that breaks
/// the adopter's build (a `dead_code` warning under `-D warnings`). The
/// use-token emit is the corrected verb behavior. This is a generated-code
/// correctness property, not a representation-divergence.
///
/// **Defense / witness**: the use-token in the macro expansion. The defending
/// site is the `expanded` quote block in the `antigen()` proc-macro at
/// `antigen-macros/src/lib.rs`. That site **cannot** carry an explicit
/// `#[immune]` marker — `antigen-macros` is a proc-macro crate that cannot
/// depend on `antigen` (the reverse dependency would be a cycle), so the
/// marker type is unreachable there, and a proc-macro crate cannot self-apply
/// its own attribute macros (see the "Proc-macro-crate markability boundary"
/// note in this module's header). Coverage is therefore the declaration here
/// plus passive fingerprint-recall: the use-token site's doc-comment mentions
/// `dead_code`, which this antigen's `doc_contains("dead_code")` fingerprint
/// matches. The witness-in-spirit is the UI/build test that a binary-crate
/// adopter's marker compiles clean under `-D warnings`.
///
/// **Internal-tooling discipline**: per `feedback-internal-tool-antigens-preemptive`,
/// declared from the confirmed camp-adoption instance — the fix shipped in the
/// macro before this declaration existed, leaving the defending code
/// unmarkable. This declaration closes that declaration-layer gap.
#[antigen(
    name = "marker-struct-dead-code-in-binary",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("dead_code")"#,
    family = "dogfood",
    summary = "An #[antigen] marker struct trips dead_code in a binary-crate adopter \
               because pub does not exempt never-constructed declaration tokens there. \
               The macro emits a zero-cost `const _: fn()` use-token so the marker is \
               genuinely used from the compiler's view, fixing the adopter's build.",
    references = ["ADR-003"]
)]
pub struct MarkerStructDeadCodeInBinary;

// ============================================================================
// 22. SerdeDefaultMaskingStructLiteralBreak
// ============================================================================

/// A `pub` struct gains a `#[serde(default)]` field; serde callers keep
/// compiling while struct-literal constructors break, hiding the migration.
///
/// `#[serde(default)]` provides backward-compatibility for ONE construction
/// path: deserialization from data that predates the field. It does nothing for
/// the OTHER construction path: Rust struct-literal expressions
/// (`Foo { a, b }`) that name every field exhaustively. When a field is added,
/// every direct struct-literal site — typically test fixtures, internal
/// builders, and `..` -less constructors — fails to compile, while every serde
/// site sails through. The author who added the field sees serde tests pass and
/// may not realize the literal sites exist; whoever wrote those sites long ago
/// is the only one who "remembers" they need migration.
///
/// **Observed instance** (2026-05-26, v0.2 completion arc): `AntigenDeclaration`
/// (`antigen/src/scan.rs`) gained `category: Vec<AntigenCategory>` with
/// `#[serde(default)]` (ADR-028). JSON sidecar deserialization of v0.1
/// declarations kept working; but the direct struct-literal sites
/// (`scan.rs` synthesis tests, `antigen-macros/src/parse.rs` test fixtures,
/// `antigen/tests/atk_a3_fractal_preview.rs`) all needed `category: Vec::new()`
/// added by hand. Scout (`1c861b3a`) caught two sites; the migration touched
/// every direct constructor. The struct also has `#[serde(default)]` on
/// `canonical_path` — same shape, an earlier instance of the same class. This
/// recurs by construction: the comprehensive-vision metadata roadmap will keep
/// adding fields to `AntigenDeclaration` / `Presentation` / `Immunity` /
/// `Ratification`, and each one re-opens the asymmetry.
///
/// **Category**: `SubstrateAlignment` — two representations of the same
/// construction contract diverge. The serde-path representation says
/// "backward-compatible, all callers handled"; the struct-literal-path
/// representation says "every exhaustive constructor is broken." The
/// `#[serde(default)]` attribute is the source of the divergence: it aligns one
/// representation while leaving the other silently out of sync.
///
/// **Defense / witness**: two complementary moves.
/// 1. `#[non_exhaustive]` on the struct forces all *external-crate* construction
///    through `..` / `Default` / a builder, eliminating the struct-literal break
///    for downstream adopters. It is necessary but **not sufficient** —
///    `#[non_exhaustive]` does not affect *same-crate/workspace* construction,
///    so internal test fixtures still break (which is exactly where the v0.2
///    instance bit).
/// 2. A substrate-witness predicate that asserts struct/constructor parity: when
///    a `#[serde(default)]` field is added to a `pub` struct, all direct
///    struct-literal sites in the workspace either use `..` rest-syntax or have
///    been migrated. This is the audit-time check that closes the internal gap
///    `#[non_exhaustive]` leaves open.
///
/// **Fingerprint** (v0.2): `all_of([item = struct, attr_present("serde")])` is
/// the broad-recall shape — a struct carrying a `serde` attribute is a
/// candidate. Precision (is there actually a `#[serde(default)]` field AND an
/// unmigrated exhaustive constructor somewhere?) lives in the witness, not the
/// fingerprint — the same F8 recall/precision split [`ActiveArgumentDiscard`]
/// uses, because the v0.2 predicate grammar cannot express "a field-level
/// attribute on one of the struct's fields" nor "an exhaustive struct-literal
/// exists elsewhere." Note `attr_present` matches *item-level* attributes; a
/// struct whose only `serde` usage is field-level `#[serde(default)]` (with no
/// item-level `#[derive(...)]`/`#[serde(...)]`) will not recall — a known v0.2
/// fingerprint-grammar limit (field-level attribute predicate absent), tracked
/// with the body-content-negation grammar gap.
///
/// **Internal-tooling discipline**: per `feedback-internal-tool-antigens-preemptive`,
/// named from the confirmed v0.2 `AntigenDeclaration` instance rather than
/// waiting for the next metadata field to break the build. Raised as a naive
/// question by outsider (v02-completion-arc); the instance was at hand, the
/// recurrence is structurally guaranteed, so it earns a declaration.
#[antigen(
    name = "serde-default-masking-struct-literal-break",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"all_of([item = struct, attr_present("serde")])"#,
    family = "dogfood",
    summary = "A pub struct gains a field with #[serde(default)]; serde-deserialization callers keep \
               compiling but Rust struct-literal constructors (test fixtures, internal builders) break. \
               The serde-default aligns one construction path while leaving the other silently out of \
               sync. Defense is #[non_exhaustive] (covers external callers only) plus a struct/constructor \
               parity substrate-witness (covers the internal gap). Broad-recall fingerprint on serde \
               structs; precision lives in the witness (F8 split).",
    references = ["ADR-028", "ADR-007"]
)]
pub struct SerdeDefaultMaskingStructLiteralBreak;

// ============================================================================
// 23. PathTraversalViaUnvalidatedComponent
// ============================================================================

/// A `pub` path-building primitive composes a caller-supplied component into a
/// filesystem path without validating it, so a traversal sequence escapes root.
///
/// A path-builder whose contract is "produce a path inside root R" silently
/// breaks that contract when an unvalidated component carries `..` or a
/// separator: the composed path resolves outside R, and a subsequent read/write
/// touches an attacker-chosen location. The representation (the builder's
/// in-root contract) diverges from the actual state (an escaped path).
///
/// **Observed instance** (2026-05-26, supply-chain path-traversal guard,
/// `d635d21`): `dep_attest_path` / `content_hash_path` / `maintainer_path` in
/// `antigen/src/supply_chain/evaluate.rs` each compose `crate_name` (and
/// `version`) into a sidecar path under `supply_chain_root`. Callers
/// (`evaluate_dep_attested`, `load_content_hash_record`) reached the builders
/// without pre-validating, so a traversal `crate_name` would have escaped the
/// `.attest/supply-chain/` root. The fix validates at the builder: an invalid
/// component resolves to the bare in-root directory (a safe miss) instead of
/// being joined.
///
/// **Why this is NOT a sibling of the validation-gap family**
/// ([`FingerprintStringWithoutDslValidation`] /
/// [`FingerprintDigestWithoutFormatValidation`], #7/#8): those are
/// *receptor-disagreement* — N known call-sites validate inconsistently, and
/// the remedy is harmonize-the-sites. This is *barrier-breach* — a `pub`
/// primitive with UNBOUNDED external callers trusts its input, and the remedy
/// is validate-at-the-primitive-boundary (you can never enumerate every caller,
/// so site-consistency is unachievable). Different remedy-structure = different
/// class. (Per naturalist's recognition ruling, 2026-05-26: barrier-integrity /
/// innate-immune arm vs adaptive-receptor arm — biologically distinct defense
/// classes, so standalone, not sibling.)
///
/// **Biology cognate**: barrier-integrity failure — untrusted input escaping a
/// containment boundary is a pathogen breaching the epithelial barrier (skin,
/// gut lining) through an unvalidated entry point. An INNATE-barrier failure
/// (the boundary itself fails to contain), distinct from the adaptive-arm
/// receptor-disagreement of the validation-gap family.
///
/// **Category**: `SubstrateAlignment` — the path-builder's contract represents
/// the composed path as in-root/safe; an unvalidated component makes the actual
/// path escape. Representation-vs-state divergence (a security-flavored
/// substrate-alignment; barrier-integrity IS a security primitive in biology).
///
/// **Known advisory hint** (`antigen-category-claim-inconsistent-with-predicate-type`):
/// this antigen is `SubstrateAlignment` by recognition (representation-vs-state),
/// but its witness is the behavioral traversal test — a *code-witness*, not a
/// `requires = ...` *substrate-witness* predicate, which ADR-028 STRICT wants for
/// a `SubstrateAlignment` claim. The v0.2 substrate-witness DSL has no predicate
/// that expresses "the composed path is contained within root," so a code-witness
/// is the only available instrument; the audit hint fires advisory (audit still
/// exits 0). This is the same accepted state as [`ParallelStateTrackersDiverge`]
/// (also `SubstrateAlignment` witnessed by a bijection/behavioral test). A
/// path-containment substrate-witness predicate is a v0.3 DSL enrichment that
/// would let the witness-type match the category.
///
/// **Defense**: validate the component at the PRIMITIVE, not the call-site, when
/// the primitive is `pub` (unbounded callers). This is the generalizable
/// discipline and exactly what distinguishes this from the validation-gap
/// family: site-level validation cannot guard a primitive whose callers you
/// cannot enumerate, so the boundary must guard itself (defense-in-depth,
/// sub-clause F at the primitive). The witness is behavioral: feed traversal
/// input to the builder and assert the result stays within the containment root
/// and carries no `..` component.
///
/// **Internal-tooling discipline**: per `feedback-internal-tool-antigens-preemptive`,
/// declared from the confirmed supply-chain instance — it recurs (this is THE
/// canonical web/CLI security shape: every `pub` path-builder taking external
/// components has it), there is a real instance plus an executable witness, so
/// it earns durable structural memory that guards every future `pub`
/// path-builder.
#[antigen(
    name = "path-traversal-via-unvalidated-component",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("path-traversal")"#,
    family = "dogfood",
    summary = "A pub path-building primitive composes a caller-supplied component into a filesystem path \
               without validating it, so a traversal sequence escapes the containment root. Barrier-breach \
               (innate-immune arm), NOT a sibling of the validation-gap family (receptor-disagreement, \
               adaptive arm): the remedy is validate-at-the-unbounded-primitive, not harmonize-known-call-\
               sites. SubstrateAlignment: the builder's in-root contract diverges from the escaped path.",
    references = ["ADR-005", "ADR-025"]
)]
pub struct PathTraversalViaUnvalidatedComponent;

// ============================================================================
// 24. AffordanceTrapInAttestationDSL
// ============================================================================

/// A DSL field accepts a value whose underlying type is shared by a
/// semantically-distinct sibling field, so a wrong-slot binding parses
/// successfully and fails only at evaluation time.
///
/// When two slots in one DSL constructor share the same underlying type
/// (both `String`, both `Ident`) but carry distinct semantics (one means
/// a signer *name*, the other a *role*), the type system cannot enforce
/// slot-membership. An author who means a role can type it into the name
/// slot; the parser accepts it silently and the predicate fails only at
/// `cargo antigen audit` time — a temporal displacement from write to
/// evaluation.
///
/// **Observed instance** (2026-05-26, `dogfood.rs:799`, aristotle F1):
/// `signers(required = ["math-researcher"])` — author intended a *role*
/// slot but used the *name* slot. Parse succeeded; audit found the
/// signer name "math-researcher" never appeared on any attestation record.
///
/// **Generalization**: any DSL constructor with 2+ `String`/`Ident` slots
/// of distinct semantics has this trap. Confirmed candidates in the closed
/// predicate grammar: `signed_trailer(key, role?)` (key vs role), and
/// `ratified_doc(path?, anchor?)` (path vs anchor). Every new DSL
/// predicate-leaf added to the grammar (ADR-022 open-grammar discipline)
/// re-introduces the trap unless the author wraps slots in distinct types.
///
/// **Fix shape**: tagged constructors
/// `name("alice") / role("math-researcher")` so name-slot and role-slot
/// are distinct at the DSL grammar level; typing a role into a name slot
/// becomes a parse error. Alternatively, newtype-wrapped strings
/// (`SignerName(String)` vs `SignerRole(String)`) enforce at the
/// type level. Both make the fail-class unrepresentable — antigen's
/// canonical move.
///
/// **Silence-generator**: silence-by-absence (taxonomy generator 1) —
/// no enforcement mechanism at the binding moment; the field name
/// "promises" the semantic but enforces nothing. The witness is a
/// type-presence check: does every multi-string-slot DSL constructor
/// wrap slots in distinct types?
///
/// **Biology cognate** (naturalist routing): molecular mimicry /
/// cross-reactivity — a receptor that binds the wrong ligand because the
/// binding site doesn't discriminate. The DSL slot is the receptor; the
/// semantic value is the ligand; same shape (String) misleads into the
/// wrong binding.
///
/// **Internal-tooling discipline**: per
/// `feedback-internal-tool-antigens-preemptive`, declared preemptively
/// from the first confirmed instance. The predicate grammar is open (any
/// new DSL constructor with `String` slots re-introduces the trap); the
/// cost of naming the class now is low; the cost of re-deriving it at
/// the next occurrence is high.
///
/// **Category**: `FunctionalCorrectness` — a DSL binding that should
/// reject a wrong-slot value silently accepts it, producing a predicate
/// that the audit cannot satisfy (the failure is behavioral: the declared
/// signer is never found on attestation records).
///
/// **Description-tier** (ADR-028 Amd6): the subject of this antigen IS
/// the relation between a declaration (the DSL value) and its referent
/// (the semantic slot). Self-reach applies: the antigen DSL grammar itself
/// can exhibit the trap.
#[antigen(
    name = "affordance-trap-in-attestation-dsl",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("affordance-trap")"#,
    family = "dogfood",
    summary = "A DSL constructor accepts a value in the wrong semantic slot \
               (e.g., a role typed into a name field) because both slots share \
               the same underlying type. The wrong binding parses silently and \
               fails only at audit evaluation time. Fix: tagged constructors or \
               newtype-wrapped slots so wrong-slot binding is a parse error.",
    references = ["ADR-005", "ADR-022"]
)]
pub struct AffordanceTrapInAttestationDSL;

// ============================================================================
// 25. AuditVerdictComputedButNotDelivered
// ============================================================================

/// An audit computation produces a correct verdict (or a structured report).
///
/// But no code path delivers that verdict to a human-readable output surface,
/// so the verdict is informationally identical to never having been computed.
///
/// This is the **delivery-arm severance** fail-class: the audit pipeline has
/// three stages — recognize (collect markers), decide (compute verdicts), and
/// deliver (render to output). A severed delivery stage makes the decide stage
/// invisible to adopters; the tool produces no actionable signal for a class
/// of findings it knows about. The failure stays silent not because nothing
/// fired, but because nothing reached the adopter.
///
/// **Observed instances** in antigen itself (aristotle, 2026-05-27):
/// Five of eight `AuditReport` families are computed by public `audit_*`
/// functions and exercised by tests, but have zero CLI render paths in
/// `cargo-antigen/src/main.rs`:
/// - `audit_deferred_defense()` → `DeferredDefenseAudit` — no `print_deferred_defense`
/// - `audit_supply_chain()` → `SupplyChainAudit` — no `print_supply_chain_audit`
/// - `audit_convergent_evidence()` → `ConvergentEvidenceAudit` — no render
/// - `audit_recurrent_emergence()` → `RecurrentEmergenceAudit` — no render
/// - `audit_mucosal_boundary()` → `MucosalAudit` — no render
///
/// Three of eight families DO render: immune-state (ADR-029 implementation),
/// antigen-category (scan output), and witness (audit hints). The severed
/// five are computed + tested but never displayed.
///
/// **Also observed at scan layer** (observer, scout, 2026-05-27):
/// `orphaned_lineage_edges()` and `dangling_child_lineage_edges()` in
/// `scan.rs` are computed but have no CLI output paths — the same delivery-arm
/// severance at the scan stage for `#[descended_from]` structural verification
/// (campsite `forward/descended-from-structural-verification`).
///
/// **Silence-generator**: silence-by-absence — the delivery mechanism was
/// never wired, so the verdict never reaches a detection surface. The enforcer
/// (the CLI render path) was never created.
///
/// **Why the 2x2 silence-taxonomy ceiling holds** (aristotle): delivery-arm
/// severance is NOT a fifth silence-generator on the acute evasion axis.
/// The 2x2 (silence-by-absence / silence-by-masking / silence-by-missing-diagnostic /
/// silence-by-wrong-weighting) classifies how a FAILURE stays silent; delivery-arm
/// severance is a different subject — how a CORRECT VERDICT stays silent after
/// the tool has already done the right thing. Different subject, different axis:
/// the regulatory meta-arm (suppression-density) and memory meta-arm (staleness)
/// are the other orthogonal axes; delivery-arm severance is efferent-execution
/// silence within the acute axis.
///
/// **Biology cognate**: leukocyte adhesion deficiency (LAD) — neutrophils
/// compute the right response (recognize, activate, degranulate) but
/// cannot adhere and migrate to the infection site, so the response never
/// arrives. The immune machinery fires correctly; the delivery fails.
///
/// **Fix shape**: for each severed `audit_*` function, wire its output into
/// the CLI render path. The witness is: `pub fn audit_X()` → `print_X_audit()`
/// called in `main.rs`; CI exercises the render path.
///
/// **Fingerprint**: a `pub fn audit_*` that produces a structured report
/// type (`*Audit`, `*Verdict`, etc.) and has test coverage in the integration
/// test suite, but has no corresponding `print_*` or `render_*` call site
/// in the binary's main output path.
///
/// **Internal-tooling discipline**: per
/// `feedback-internal-tool-antigens-preemptive`, declared from confirmed
/// instances (5 severed report families + 2 scan-layer outputs). The pattern
/// is structural — any new `audit_*` family added without a paired render path
/// silently reintroduces this fail-class.
///
/// **Category**: `FunctionalCorrectness` — the tool claims to audit a class
/// but never surfaces the verdict; the adopter observes nothing, which is
/// functionally indistinguishable from the audit not being implemented at all.
#[antigen(
    name = "audit-verdict-computed-but-not-delivered",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("delivery-arm")"#,
    family = "dogfood",
    summary = "A `pub fn audit_*` function correctly computes a verdict but no CLI \
               render path exists, so the verdict is invisible to adopters. Silence \
               of a correct output is informationally identical to never computing it. \
               Fix: pair every audit_* function with a print_*() render path that is \
               exercised end-to-end in the CLI integration test suite.",
    references = ["ADR-005", "ADR-028"]
)]
pub struct AuditVerdictComputedButNotDelivered;

// ============================================================================
// 26. AbsentErrorCollapse
// ============================================================================

/// A match or parse point conflates the **error case** with the **intentionally-absent
/// case**, producing silent failures where diagnostics should fire.
///
/// The general shape: code has an optional value `Option<T>`. When the value is
/// `None` (absent — intentionally not set), the correct behavior is to skip the
/// check (no assumption violated). When the value is `Some(bad-string)` (present
/// but malformed — a declaration error), the correct behavior is to emit a diagnostic.
/// Both cases collapse to the same "skip" behavior at the match point, so a malformed
/// declaration is indistinguishable from an absent one. The author who wrote the
/// bad value believes they set it correctly; the audit never corrects them.
///
/// **Three confirmed shapes across five sites in antigen/cargo-antigen**
/// (adversarial, 2026-05-27, camp notice):
///
/// **(1) None-arm parse collapse** (`if let Some(x) = parse_optional(raw)` where
/// `raw = Some(bad-string)` → `parse_optional` returns `None` → outer `if let`
/// arm not entered):
/// - `audit.rs:1081`: `orient.until = Some("2099/01/01")` (slash-format typo) →
///   silently grants `OrientActive` forever (campsite `findings/orient-unparseable-until-silent-green`)
/// - `audit.rs:1072`: `immunosuppress.since = Some("bad-date")` → silently skips
///   the `duration_cap` exceeded check, granting indefinite `ImmunosuppressActive`
///   (adversarial commit `fd3e387`)
///
/// **(2) Bare-name match without canonical path** (error: wrong crate's entity;
/// absent: no entity; both treated as "found" or "not-found" without disambiguation):
/// - `scan.rs:2076`: `d.antigen_type == p.antigen_type` silently credits a defense
///   from a different crate's `Foo` against this crate's `Foo` (ATK-ADR029-21)
/// - `audit.rs:1335` (verdict path): same bare-name match in `compute_presentation_verdicts`
/// - `audit.rs:3100` (G2 path): same bare-name match in `audit_category` G2 check (ATK-G2-22)
///
/// **(3) if-let-Ok silent skip** (parse-error arm absent; failure treated as empty-file):
/// - `main.rs:1738`: `oracle list` silently skips corrupt `.json` files, yielding
///   "No oracle records found" with exit 0 and no parse-error diagnostic
///   (campsite `findings/oracle-corrupt-json-silent-skip`)
/// - `main.rs:3712`: `attest gc` silently skips sidecars with corrupt JSON
///
/// **Why the pattern recurs**: each site was written to handle the "intentionally
/// absent" gracefully (skip or return-empty). The author did not add a separate
/// arm for "present but malformed" because both initially produce the same
/// computational result (nothing to work with). The discriminator between "absent"
/// and "malformed" is never installed.
///
/// **Fix direction** (invariant across all three shapes): split the absent arm from
/// the error arm at the decision point. The absent arm retains its silent/skip behavior
/// (intentional, correct). The error arm emits a diagnostic (present-but-malformed
/// is a declaration error, not optional data).
///
/// **Silence-generator**: silence-by-absence — the error arm (the one that emits a
/// diagnostic) was never installed, so malformed declarations are never flagged.
///
/// **Biology cognate**: **tolerance by deletion** vs **tolerance by ignorance** —
/// the immune system tolerates self-antigens EITHER because self-reactive clones
/// were deleted (absent arm: the clone doesn't exist; no response is correct) OR
/// because self-antigens were never displayed (ignorance arm: the antigen is not
/// accessible; no response fires). `AbsentErrorCollapse` is the failure when these
/// two arms are not distinguished: an antigen IS present (malformed declaration =
/// malformed antigen displayed), but the response-gating collapses it into the
/// "never-displayed" ignorance path, silently failing to respond.
///
/// **Internal-tooling discipline**: per
/// `feedback-internal-tool-antigens-preemptive`, declared from 5+ confirmed
/// instances across 3 structural shapes. The pattern is structural — any new
/// match point that handles `None` (absent) without a separate arm for
/// `Some(parse_failure)` is a potential site.
///
/// **Category**: `FunctionalCorrectness` — the tool's diagnostic-emission function
/// is broken at the error-path: adopters who write malformed declarations get no
/// correction; the tool appears to accept the malformed input silently.
#[antigen(
    name = "absent-error-collapse",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("absent-error-collapse")"#,
    family = "dogfood",
    summary = "A match or parse point conflates the error case (present-but-malformed \
               declaration) with the absent case (value not set), silently skipping a \
               diagnostic that should fire. The absent arm's 'skip' behavior is correct; \
               the error arm's identical 'skip' is the failure. Fix: split the two arms \
               at the decision point so malformed declarations emit diagnostics.",
    references = ["ADR-005"]
)]
pub struct AbsentErrorCollapse;

// ============================================================================
// 27. AuditIndexKeyCollision
// ============================================================================

/// Audit index keyed on insufficient identity; audit silently evaluates the
/// **wrong item's data**.
///
/// An audit function builds an index (or selects via `.first()`) keyed on K.
/// Two or more items are logically distinct but share K (same position in a
/// list / same bare name / same bare type name). When the second item is
/// audited, the lookup returns the FIRST item's data. The
/// audit passes or fails based on the wrong item's properties. No error is
/// surfaced; the verdict is silently wrong.
///
/// The fix is always the same: **expand the key to include the distinguishing
/// dimension**. The distinguishing dimension is always present in the data
/// (item path, file path, canonical crate path) but was omitted because the
/// index was designed for the common case (one item per sidecar, one function
/// with that name, one crate with that type name). The omission is correct in
/// the common case; the bug manifests only in the expanded case.
///
/// **Three confirmed instances** (scout, 2026-05-28, antigen-dx-dogfood
/// expedition; three instances = naming threshold per the project's
/// recurrence-gate):
///
/// **(1) Sidecar first-item shortcut** (`audit.rs:1597`):
/// `audit_substrate_witness()` calls `sidecar.items.first()` to select the
/// evaluation target. When a sidecar covers multiple items, item N's immunity
/// is evaluated against item 0's `ItemRatification` — using item 0's
/// fingerprint and item 0's signers. Key = list position (always 0); the
/// distinguishing dimension is item path. Campsite:
/// `findings/sidecar-first-item-wrong-audit`.
///
/// **(2) Mucosal same-name function collision** (`audit.rs:2966`):
/// `audit_mucosal()` builds `handler_kinds: HashMap<&str, HashSet<&str>>`
/// keyed by bare function name. Two `#[mucosal]` functions with the same name
/// in different files merge their kind-sets under one key. A delegate pointing
/// at that name gets the union of BOTH files' kinds and may silently pass the
/// kind-check using the wrong file's kinds. Key = bare fn name; the
/// distinguishing dimension is source file. Fix shape: detect same-name
/// ambiguity at index-build time and emit
/// `MucosalDisciplineDelegateTargetAmbiguous` (keying by `(file, fn_name)`
/// requires `handled_by` to carry file info, which it does not). Campsite:
/// `findings/mucosal-same-name-fn-collision`.
///
/// **(3) Cross-crate defense bare-type match** (pre-fix; fixed at `03e1c99`):
/// `defense_addresses()` used `d.antigen_type == p.antigen_type` alone — bare
/// type name without `canonical_path`. Crate A's `Foo` and crate B's `Foo`
/// shared one key, so a defense from crate A silently credited against crate
/// B's presentation. Key = bare type name; the distinguishing dimension is
/// canonical crate path. Fix: `d.antigen_type == p.antigen_type &&
/// d.canonical_path == p.canonical_path`. (ATK-ADR029-21, `scan.rs:2368`.)
///
/// **Structural shape across all three**:
/// - Index keyed on K (position / bare name / bare type name).
/// - Items logically distinct but sharing K.
/// - Audit returns the wrong item's data when K collides.
/// - Verdict is silently wrong — no error, no diagnostic.
/// - Fix: expand the key to include the distinguishing dimension (item path /
///   file / canonical path). The distinguishing dimension was in the data all
///   along; it was simply omitted from the key.
///
/// **Connection to `LookupKeyInsufficientIdentity` as a dogfood antigen
/// candidate** (garden, 2026-05-28): this class is named from the structural
/// shape of the fix (key collision → expand key), not from the silence type.
/// The silence-generator is silence-by-absence: the lookup's ambiguity arm was
/// never installed. The class is distinct from
/// [`ParallelStateTrackersDiverge`] (representation drift between two copies
/// of the same fact) and from [`AuditVerdictComputedButNotDelivered`]
/// (correct verdict never reaching the output surface). Here the
/// *computation itself is wrong* because the index resolves to the wrong item.
///
/// **Fingerprint**: `doc_contains("items.first")` is a recall fingerprint for
/// the position-key shape — `"items.first()"` in a doc comment signals the
/// forward-pointer that the author KNEW the key was narrower than it should be
/// (a comment acknowledging that later work would match by `item_path`). The
/// other two shapes (bare fn name / bare type name) are
/// caught by the explicit `#[presents]` markers on the two blocked campsites'
/// index-build sites. A fingerprint grammar capable of expressing "a
/// `HashMap` keyed by `&str` where the value type has a second `&str`
/// field not used as the key" is a v0.3 predicate-language enrichment; for
/// v0.2 the recall-fingerprint + explicit markers provide the coverage.
///
/// **Category**: `FunctionalCorrectness` — the audit's index-selection
/// function returns the wrong item; the verdict is a wrong output (not a
/// representation-vs-state layer split). The key-selection is the computation
/// being audited; computing it with insufficient identity is a
/// computation-correctness failure.
///
/// **Internal-tooling discipline**: per
/// `feedback-internal-tool-antigens-preemptive`, declared from three confirmed
/// instances. All three were found empirically in the same expedition arc;
/// the structural shape is predictable to recur wherever an index is built
/// without incorporating the distinguishing dimension.
#[antigen(
    name = "audit-index-key-collision",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("items.first")"#,
    family = "dogfood",
    summary = "An audit lookup/index is keyed on insufficient identity (bare name, list \
               position, or bare type name) so two logically distinct items share the \
               same key; the audit silently evaluates the wrong item's data. Three \
               instances: sidecar items.first() shortcut (audit.rs:1597), mucosal \
               handler_kinds bare-fn-name HashMap (audit.rs:2966), cross-crate defense \
               bare-type-name match (pre-fix, 03e1c99). Fix: expand the key to include \
               the distinguishing dimension (item_path, source file, canonical_path).",
    references = ["ADR-005", "ADR-030"]
)]
pub struct AuditIndexKeyCollision;

// ============================================================================
// 28. TristateCollapseToBinary
// ============================================================================

/// A predicate or function that has three real return states represents only
/// two, collapsing the absent-precondition state into one of the valid pair.
///
/// The three real states: **Match** (predicate evaluated, condition present),
/// **NoMatch** (predicate evaluated, condition absent), **Undefined**
/// (predicate cannot evaluate — the precondition for evaluation is absent).
/// When the representation carries only two states, `Undefined` is forced into
/// either `NoMatch` (false negative: "not found" when the truth is "can't
/// check") or `Match` (vacuous match: "found" when the truth is "predicate
/// has no locus here"). Both produce silently wrong verdicts.
///
/// The fix is always to **represent the third state explicitly** and build
/// downstream handling that uses it correctly — conservative (non-match, but
/// not `NoMatch`) in most audit contexts.
///
/// **Six confirmed instances** (scout + navigator, 2026-05-28,
/// antigen-dx-dogfood expedition; six instances = well past the naming
/// threshold):
///
/// **(1) Fingerprint `not(body_contains_macro)` on a struct** (pre-fix;
/// fixes by ADR-010 Amendment 6): `body_contains_macro` applied to a struct
/// returns `false` (no body). `not(false) = true`, so ALL structs vacuously
/// match `all_of([item=struct, not(body_contains_macro(X))])`. The honest
/// return is `Undefined` — structs don't have bodies; the predicate has no
/// locus. Match3 (`{Match, NoMatch, Undefined}` with Kleene-strong algebra)
/// closes this: `not(Undefined) = Undefined`, which does not contribute to a
/// positive match result. Campsite:
/// `forward/fingerprint-grammar-body-content-with-negation`.
///
/// **(2) Orient `until` date — collapse: None vs Some(malformed-date)**
/// (`audit.rs:1081`): when `until = Some("2026/01/01")` (slash-format typo),
/// `parse_optional` returns `None`; the outer `if let Some(x)` arm is not
/// entered; the audit silently grants `OrientActive` forever. The malformed
/// case is indistinguishable from "no until date." Fixed by the Orient-style
/// split: `None` (absent) retains silent skip; `Some(bad-date)` escalates
/// to a diagnostic. Campsite: `findings/orient-unparseable-until-silent-green`.
///
/// **(3) Anergy/Immunosuppress `until` date — same collapse** (`audit.rs:1119,
/// 1151, 1158`): `unwrap_or("")` collapses `None` (absent) and
/// `Some("not-a-date")` (present but malformed) to the same empty string, so
/// a typo in `until=` silently grants indefinite `AnergicActive` /
/// `ImmunosuppressActive`. Campsite:
/// `findings/deferred-until-malformed-silent-active`.
///
/// **(4) Supply-chain predicate evaluation — no-sidecar vs failed**
/// (pre-fix): a predicate returning `false` (checked, fails) was
/// indistinguishable from "no sidecar found" (unchecked, precondition
/// absent). Fixed by surfacing the sidecar-absence state explicitly in the
/// audit verdict.
///
/// **(5) `AuditVerdict` `Defended vs Undefended`** (pre-`SubstrateGap`): two
/// verdict states could not represent "witness present but predicate
/// UNSATISFIED at the substrate" — that case collapsed into `Undefended`.
/// `SubstrateGap` is the explicit third value added by ADR-029.
///
/// **(6) `WitnessTier` in `requires=` context — tier-present vs
/// tier-absent vs precondition-missing**: a `requires=` predicate that
/// references a signer role that never signed produces the same audit outcome
/// as a predicate whose evaluation precondition is simply not met. Both
/// collapse to the same non-pass state without distinguishing "checked and
/// found absent" from "unchecked because the substrate couldn't be reached."
///
/// **Biology cognate** (naturalist gate required before ratification; scout
/// recommendation for routing): **anergy** — T-cell anergy is the cellular
/// program that runs when a T-cell receptor engages its antigen but the
/// costimulatory signal (CD28/B7, signal-2) is absent. The cell enters a
/// persistent non-responsive program that is CATEGORICALLY DISTINCT from
/// "clone absent" and from "TCR not engaged." The immune system represents all
/// three states explicitly; confusing anergy with deletion (wrong third-state
/// collapse) produces tolerance failures with massive clinical consequences.
/// The antigen analog: `Undefined` (evaluator can't apply the predicate) is
/// distinct from `NoMatch` (evaluated, condition absent); collapsing them
/// produces the vacuous-not hazard (instance 1) and the malformed-as-absent
/// silences (instances 2, 3).
///
/// **Silence-generator**: silence-by-absence — the third state's arm was
/// never installed, so the collapsed value is the only output available.
///
/// **Fix**: represent `Undefined` as a first-class return value. Use
/// Kleene-strong algebra for composition: `not(Undefined) = Undefined`;
/// `all_of` where any child is `NoMatch` = `NoMatch`; `all_of` with at least
/// one `Undefined` and no `NoMatch` = `Undefined`. Project to bool at the
/// outermost layer only (the fingerprint-fires boundary). This is the exact
/// design of Match3 in ADR-010 Amendment 6.
///
/// **Category**: `FunctionalCorrectness` — the predicate or function produces
/// wrong output (a false negative or vacuous match) by collapsing the
/// evaluator-can't-evaluate state into a definite verdict.
///
/// **Distinction from [`AbsentErrorCollapse`]** (#26): `AbsentErrorCollapse`
/// is specifically about a MATCH POINT confusing a present-but-malformed
/// VALUE with an absent value. `TristateCollapseToBinary` is broader: the
/// precondition can be structurally absent (no body exists; no sidecar;
/// costimulation unavailable) — not just malformed. Different cause
/// (structural-absence vs malformed-presence), different fix-shape
/// (three-valued algebra vs error-arm split).
///
/// **Internal-tooling discipline**: per
/// `feedback-internal-tool-antigens-preemptive`, declared from six confirmed
/// instances. The pattern is universal — any code that evaluates a predicate
/// against a substrate that might not exist is a potential site. The
/// fingerprint targets the cases where the author already named the third
/// value (`Undefined`, `Undefined` in an enum name); explicit `#[presents]`
/// markers cover the pre-fix instances that never named it.
///
/// **NOTE**: naturalist biology gate is recommended before ratification of
/// this antigen into the stdlib (ADR-027 biology-grounding discipline for
/// non-dogfood families). The biology is already well-mapped (anergy vs
/// deletion vs inactivation-by-absence), but the formal gate produces the
/// biological literature anchor required by ADR-027. This declaration is in
/// the dogfood family so the gate is not blocking; it is advisory.
#[antigen(
    name = "tristate-collapse-to-binary",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"doc_contains("Undefined")"#,
    family = "dogfood",
    summary = "A predicate or verdict has three real states (match / no-match / \
               precondition-absent) but is represented as two; the absent-precondition \
               state collapses silently into no-match or match, producing a false \
               negative or vacuous positive. Six instances: fingerprint body-predicate \
               vacuous-not (#1), orient-until malformed (#2), anergy-until malformed (#3), \
               supply-chain no-sidecar-vs-failed (#4), AuditVerdict pre-SubstrateGap (#5), \
               WitnessTier precondition-missing (#6). Fix: represent Undefined explicitly \
               with Kleene-strong algebra (not(Undefined)=Undefined; all_of propagates \
               Undefined conservatively). Biology cognate: T-cell anergy — distinct \
               cellular program from clone-absent, not a weak form of activation.",
    references = ["ADR-010", "ADR-029", "ADR-031"]
)]
pub struct TristateCollapseToBinary;

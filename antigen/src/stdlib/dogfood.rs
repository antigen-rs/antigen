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
/// silent. The defended producer in this crate is `#[immune]` below with a test
/// witness that pins FNV-1a to its published vector.
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

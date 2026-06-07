//! Immunity-verdict computation — the core `audit()` pass.
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). This module owns the heavy lifting that the
//! monolithic `audit()` used to inline: the per-immunity witness audit (code-
//! witness resolution + substrate-witness predicate evaluation), the per-site
//! immune-state verdict computation (`compute_presentation_verdicts`), and the
//! shared witness-resolution infrastructure (the workspace function index, the
//! sidecar loader, the attestation-tier/hint mappers). `audit::audit` is a thin
//! re-export of `run` here. A pure fn of `&ScanReport` (+ workspace root, for
//! the function-index + sidecar reads); it holds no stop-authority (the
//! single-conductor invariant, ADR-036).
//!
//! API-invisible: `audit` is re-exported from the `audit` root via `pub use`;
//! the sidecar-infra trio (`FilesystemAuditContext` / `SidecarLoad` /
//! `load_sidecar`) is re-exported `pub(crate)` for the prescriptive detector,
//! which shares the witness-evaluation substrate.

use std::path::{Path, PathBuf};

use antigen_macros::{dread, presents};

use super::{
    AuditHint, AuditReport, ImmuneVerdict, ImmunityAudit, InheritedUnaddressed,
    PresentationVerdict, WitnessKind, WitnessStatus, WitnessTier, evidence_kind_from_status,
};
use crate::scan::{Immunity, ScanReport};

/// Filesystem-backed [`antigen_attestation::EvaluationContext`] for use
/// during real audit runs. Reads docs and oracles directly from disk; reads
/// git trailers by shelling out to `git interpret-trailers`. Tests in
/// `antigen-attestation` use an in-memory context instead (see
/// `evaluate.rs` `TestContext`).
pub struct FilesystemAuditContext;

impl antigen_attestation::EvaluationContext for FilesystemAuditContext {
    fn today(&self) -> chrono::NaiveDate {
        chrono::Local::now().date_naive()
    }

    fn read_doc(&self, path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn read_oracle(&self, path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    // v0.1: git trailers require subprocess + git; returns empty vec when
    // git is unavailable or the item has no commits. A3+ work wires this
    // to a proper git2 adapter or subprocess; for now the trait contract
    // is satisfied and `SignedTrailer` leaf evaluates to false.
    fn read_git_trailers(
        &self,
        _item_source_file: &std::path::Path,
        _item_path: &str,
    ) -> Vec<String> {
        Vec::new()
    }
}

/// Outcome of [`load_sidecar`] — distinguishes "file absent" from "file
/// present but structurally/semantically invalid" so the audit can emit the
/// correct hint in each case.
pub enum SidecarLoad {
    /// File does not exist (or I/O error reading it).
    Missing,
    /// File exists but failed JSON deserialization or semantic `validate()`
    /// (NFA-17 guard: `CryptoSigned` requires `signature` field, etc.).
    ///
    /// The audit emits `DisciplineSidecarSchemaInvalid` rather than
    /// `DisciplineSidecarMissing` so the adopter can distinguish "sidecar
    /// missing" (needs `cargo antigen attest scaffold`) from "sidecar
    /// present but broken" (needs to be fixed or re-signed).
    SchemaInvalid,
    /// File loaded and passed semantic validation.
    Ok(antigen_attestation::Ratification),
}

/// Attempt to load and deserialize a `.attest/<antigen_name>.json` sidecar.
///
/// Returns [`SidecarLoad`] to distinguish "missing" from "invalid" — the
/// audit emits different hints for each case. Returns `SchemaInvalid` when
/// the file exists but fails JSON deserialization OR semantic validation.
///
/// The validation call is the NFA-17 guard
/// (forward/serde-validate-post-deserialize-systematic): serde's derived
/// `Deserialize` does not enforce semantic invariants (e.g., `CryptoSigned`
/// strength requires `signature` field). Calling `validate()` after `from_str`
/// ensures a semantically-invalid sidecar is treated as schema-invalid rather
/// than trusted — preventing tier inflation without cryptographic backing.
pub fn load_sidecar(immunity_file: &Path, antigen_type: &str) -> SidecarLoad {
    let Some(dir) = immunity_file.parent() else {
        return SidecarLoad::Missing;
    };
    // Antigen type may be a fully-qualified path (`crate::antigens::SomeAntigen`);
    // use only the last segment as the filename component for v0.1 convention.
    let stem = antigen_type.rsplit("::").next().unwrap_or(antigen_type);
    let sidecar_path = dir.join(".attest").join(format!("{stem}.json"));
    let Ok(content) = std::fs::read_to_string(&sidecar_path) else {
        return SidecarLoad::Missing;
    };
    let Ok(ratification) = serde_json::from_str::<antigen_attestation::Ratification>(&content)
    else {
        return SidecarLoad::SchemaInvalid;
    };
    // Post-deserialization semantic validation (NFA-17 guard + delta-chain
    // invariants). Use workspace defaults — load_sidecar has no workspace
    // config context, so the hard-floor defaults are the correct boundary.
    if ratification
        .validate(
            antigen_attestation::schema::DEFAULT_DELTA_CHAIN_CAP,
            antigen_attestation::schema::DEFAULT_DELTA_RATIONALE_MIN_CHARS,
        )
        .is_err()
    {
        return SidecarLoad::SchemaInvalid;
    }
    SidecarLoad::Ok(ratification)
}

/// Audit one **code-witness** immunity (`witness = <fn>`, no `requires =`):
/// resolve the witness identifier against the workspace function index and
/// derive its tier / hint / evidence-kind. Lifted out of the inline `audit()`
/// loop (ADR-036) so the loop is a thin two-branch dispatch (this fn for
/// code-witnesses, [`audit_substrate_witness`] for substrate-witnesses). A pure
/// fn of its inputs; `report` is read only to detect a companion `requires =`
/// immunity (DX finding 3).
fn audit_code_witness(
    immunity: &Immunity,
    workspace_functions: &FunctionIndex,
    report: &ScanReport,
) -> ImmunityAudit {
    let status = validate_witness(&immunity.witness, workspace_functions);
    let witness_tier = WitnessTier::from_status(&status);
    let audit_hint = AuditHint::from_status(&status);
    let evidence_kind = evidence_kind_from_status(&status);
    // DX finding 3: this is a code-witness site (no `requires =`).
    // If a `.attest/<antigen>.json` sidecar exists for it anyway,
    // the adopter scaffolded + signed a substrate-witness sidecar
    // that audit will NEVER credit (sidecars are evaluated only on
    // the `requires =` path). Detect it so the printer can warn,
    // rather than letting the adopter believe they've attested.
    //
    // BUT: a `witness =` and a `requires =` immunity can be STACKED
    // on the same item (ADR-028 hybrid / compound evidence). In that
    // case the sidecar is legitimately owned by the companion
    // `requires =` record — flagging the `witness =` record as
    // "sidecar ignored" would be a false positive (ATK-W7-I). Only
    // warn when no companion `requires =` immunity on the SAME ITEM
    // claims the sidecar. "Same item" = same file AND item_target
    // AND antigen_type — the file dimension is load-bearing: a
    // `requires =` immunity for the same antigen in a DIFFERENT file
    // (e.g. another test fixture) is NOT a companion, and omitting
    // the file check let an unrelated workspace immunity suppress the
    // warning across files (a regression the f3 test caught).
    let has_companion_requires = report.immunities.iter().any(|other| {
        other.requires_predicate.is_some()
            && other.antigen_type == immunity.antigen_type
            && other.item_target == immunity.item_target
            && other.file == immunity.file
    });
    let code_witness_sidecar_ignored = !has_companion_requires
        && matches!(
            load_sidecar(&immunity.file, &immunity.antigen_type),
            SidecarLoad::Ok(_)
        );
    ImmunityAudit {
        immunity: immunity.clone(),
        witness_status: status,
        witness_tier,
        audit_hint,
        evidence_kind,
        signature_strength: None,
        compound_evidence: false,
        evaluated_predicate: None,
        code_witness_sidecar_ignored,
        // Code-witness path: no substrate-witness predicate was
        // evaluated, so there are no per-leaf outcomes (Finding 7).
        leaf_outcomes: Vec::new(),
    }
}

/// Run audit against a [`ScanReport`].
///
/// For each immunity declaration, attempts to validate the witness identifier
/// by walking the workspace looking for the function it names.
///
/// `workspace_root` is used to look for witness functions; passing the same
/// path used for [`crate::scan::scan_workspace`] is typical.
///
/// Files that fail to parse during the function-index walk are silently
/// skipped (matching `scan_workspace`'s behavior); this function does not
/// itself surface IO errors to the caller.
#[must_use]
pub fn audit(report: &ScanReport, workspace_root: &Path) -> AuditReport {
    let workspace_functions = collect_function_index(workspace_root);

    let mut audits = Vec::new();
    for immunity in &report.immunities {
        // Substrate-witness path (ADR-019 P3c) when `requires = <predicate>` was
        // declared; code-witness path otherwise (validate witness identifier via
        // function index). Both branches are pure fns of their inputs — the loop
        // owns no stop-authority (single-conductor invariant, ADR-036).
        let immunity_audit = immunity.requires_predicate.as_ref().map_or_else(
            || audit_code_witness(immunity, &workspace_functions, report),
            |predicate_json| audit_substrate_witness(immunity, predicate_json),
        );
        audits.push(immunity_audit);
    }

    let mut audit_report = AuditReport {
        audits,
        ..AuditReport::default()
    };
    for a in &audit_report.audits {
        match &a.witness_status {
            WitnessStatus::Resolved { .. } => audit_report.resolved_count += 1,
            WitnessStatus::External { .. } => audit_report.external_count += 1,
            WitnessStatus::Ambiguous { .. } => audit_report.ambiguous_count += 1,
            WitnessStatus::NotFound { .. } => audit_report.broken_count += 1,
            WitnessStatus::Missing => audit_report.missing_count += 1,
        }
    }

    // State 7 detection (ADR-018 §"7-state interaction matrix"): an
    // inherited Presentation (inherited_from = Some(_)) without matching
    // Immunity or Toleration on the descendant site is unaddressed.
    // `unaddressed_presentations()` already encodes the "no matching
    // immune/tolerance" check; we filter its output to the inherited
    // subset.
    for u in report.unaddressed_presentations() {
        if u.presentation.inherited_from.is_some() {
            audit_report
                .inherited_unaddressed
                .push(InheritedUnaddressed {
                    presentation: u.presentation,
                    audit_hint: AuditHint::InheritedPresentationNotReAttested,
                });
        }
    }

    // ADR-029: compute the per-presents-site immune-state verdicts by cross-
    // referencing each `#[presents(X)]` against the `#[defended_by(X)]` witnesses
    // and the (deprecated, still-honored) `#[immune]` audits already computed
    // above. Immunity is observed here — by the audit — never declared at the site.
    audit_report.presentation_verdicts =
        compute_presentation_verdicts(report, &audit_report.audits);

    audit_report
}

/// Cross-reference every `#[presents(X)]` site against the `#[defended_by(X)]`
/// code-tier witnesses and the (deprecated) `#[immune]` audits to compute the
/// per-site immune-state verdict (ADR-029).
///
/// Matching is **class-level** for `#[defended_by]`: a witness registered for
/// failure-class X defends all presents-sites of class X (the witness declares
/// *what it defends*; whether it exercises a *specific* site's failure mode is
/// the documented open semantic-gap, future coverage-join work). Wrong-class
/// witnesses do not cross-reference — `#[defended_by(WrongClass)]` never grants
/// a `RightClass` site a defended verdict.
///
/// Backward-compat: a same-item `#[immune(X, ...)]` (the deprecated declared-
/// immunity path) still contributes its tier, so adopters migrate from
/// `#[immune]` to `#[defended_by]` gradually without losing verdicts. A
/// substrate-tier `#[immune(X, requires=P)]` whose predicate fails yields
/// `substrate-gap` rather than `undefended` — intent present, substrate drifted.
fn compute_presentation_verdicts(
    report: &ScanReport,
    immunity_audits: &[ImmunityAudit],
) -> Vec<PresentationVerdict> {
    let mut verdicts = Vec::new();

    for p in &report.presentations {
        // ADR-029 verdicts grade DECLARED defense intent: only explicit
        // `#[presents(X)]` markers, never fingerprint-inferred matches. An
        // inferred match is the scan's broad triage signal (see
        // `ScanReport::partitioned_presentations` — `inferred` is triage-first,
        // not CI-gateable); grading it `undefended` would flood the verdict
        // surface with structural-pattern noise the developer never declared.
        // The developer who wrote `#[presents]` declared the site; the audit
        // observes its defense.
        if p.match_kind != crate::scan::MatchKind::ExplicitMarker {
            continue;
        }

        // Code-tier witnesses: `#[defended_by(X)]` registrations matching this
        // site's failure-class. Class-level match (ADR-029) — strict on
        // antigen_type so a wrong-class witness cannot pollute the verdict.
        //
        // The witness MUST be a function: `#[defended_by]` is scoped to
        // `#[test]` / proptest functions (ADR-029 §Scope) — a runnable code-tier
        // witness. Both free functions (scan `item_kind == "fn"`) and methods
        // inside an `impl` block (`"impl_fn"` — a `#[test] fn` in an
        // `impl Tests {}`) are runnable witnesses. A `#[defended_by]` on a
        // struct/enum/impl-block/trait is a misuse: a non-fn item cannot be
        // executed as a test, so it provides no evidence and must not grant a
        // Defended verdict (ADR-005 sub-clause F — a non-runnable witness is not
        // a witness). Scan records it (recall-tuned); audit is the trust boundary
        // that refuses to credit it.
        // `defense_addresses` is the shared canonical-path-aware class-level
        // match (so a cross-crate `#[defended_by(Foo)]` does not credit a
        // foreign `Foo` presents-site — ATK-ADR029-21/ATK-G2-22); the fn-kind
        // guard is verdict-specific (only a runnable witness grants Defended).
        let code_witnesses: Vec<&crate::scan::Defense> = report
            .defenses
            .iter()
            .filter(|d| {
                crate::scan::defense_addresses(d, p)
                    && (d.item_kind == "fn" || d.item_kind == "impl_fn")
            })
            .collect();

        // Deprecated declared-immunity path: a same-item `#[immune]` audit for
        // the same antigen still contributes. Same-item match (file +
        // item_target) mirrors `addresses_for_immunity` — an immune claim is
        // about the item it sits on, not the whole class.
        //
        // Three-valued-logic / stacked-immunity fix
        // (forward/immune-stacked-same-item-substrate-gap-mask):
        //
        // `find()` returns only the FIRST match. With stacked same-antigen
        // same-item `#[immune]` declarations, the first match may have no
        // `requires=` (→ Defended) while a later entry has a FAILING `requires=`
        // whose substrate gap would be silently masked. The fix:
        //   - `immune_audit` (for tier) still uses `find()` — the best code-tier
        //     evidence from any matching entry is sufficient for `Defended`.
        //   - `immune_any_substrate_gap` scans ALL matching entries via `any()` —
        //     if ANY stacked immunity for this item is a substrate gap, the gap
        //     surfaces regardless of the other entries' states.
        let immune_audit: Option<&ImmunityAudit> = immunity_audits.iter().find(|a| {
            a.immunity.antigen_type == p.antigen_type
                && a.immunity.file == p.file
                && a.immunity.item_target == p.item_target
        });
        let immune_any_substrate_gap = immunity_audits.iter().any(|a| {
            a.immunity.antigen_type == p.antigen_type
                && a.immunity.file == p.file
                && a.immunity.item_target == p.item_target
                && immune_audit_is_substrate_gap(a)
        });

        // Verdict precedence:
        //   1. any defending evidence (code witness or immune audit with tier
        //      > None) => Defended at the strongest observed tier
        //   2. an immune(requires=) whose predicate failed => SubstrateGap
        //   3. otherwise => Undefended
        let code_tier = if code_witnesses.is_empty() {
            None
        } else {
            // v0.3 audit does not invoke cargo test / coverage, so a registered
            // code-tier witness lands at Reachability — the honest tier ("this
            // witness exists and names this class"). Execution/FormalProof are
            // coverage-confirmed / substrate / phantom tiers, computed elsewhere.
            Some(WitnessTier::Reachability)
        };
        let immune_tier = immune_audit
            .map(|a| a.witness_tier)
            .filter(|t| *t != WitnessTier::None);

        // Substrate-tier evidence folded onto the presents-site (ADR-029 R5):
        // `#[presents(X, requires = P)]`. Evaluate the predicate against the
        // `.attest/` sidecar exactly as the deprecated `#[immune(requires=P)]`
        // path does — reusing `audit_substrate_witness` via a LOCAL adapter
        // Immunity. The adapter never enters `report.immunities` (it is not a
        // ghost record polluting any count) — it is purely the evaluation input
        // shape the existing pipeline expects. A passing predicate grades the
        // site Defended at the substrate tier; a non-passing one is a
        // substrate-gap (intent present, substrate drifted).
        let site_requires_eval = p.requires_predicate.as_ref().map(|json| {
            let adapter = Immunity {
                antigen_type: p.antigen_type.clone(),
                witness: String::new(),
                requires_predicate: Some(json.clone()),
                file: p.file.clone(),
                line: p.line,
                item_kind: p.item_kind.clone(),
                item_target: p.item_target.clone(),
                canonical_path: p.canonical_path.clone(),
                structural_fingerprint: p.structural_fingerprint.clone(),
            };
            audit_substrate_witness(&adapter, json).witness_tier
        });
        let site_requires_tier = site_requires_eval.filter(|t| *t != WitnessTier::None);

        // Phantom-tier evidence folded onto the presents-site (ADR-029 R5):
        // `#[presents(X, proof = <expr>)]`. The proof is a type-system
        // construction whose mere existence is the evidence — it compiled, so
        // the construction is valid (FormalProof). No sidecar/runtime evaluation:
        // the presence of `proof` IS the witness, recognized structurally (same
        // posture as the deprecated `#[immune(witness = <phantom>)]` path, which
        // graded `WitnessKind::PhantomType` → FormalProof).
        //
        // Defense-in-depth (the observe-half) for the empty-proof overclaim: the
        // macro now rejects a string-literal `proof =` at authoring time, but a
        // hand-built / deserialized Presentation could still carry an empty or
        // whitespace `proof` string. An empty proof is not a construction — do
        // NOT grade it FormalProof (it would silently claim the strongest tier
        // with no substance). Only a non-blank proof expression counts.
        let site_proof_tier = p
            .proof
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|_| WitnessTier::FormalProof);

        let best_tier = [code_tier, immune_tier, site_requires_tier, site_proof_tier]
            .into_iter()
            .flatten()
            .max_by_key(|t| *t as u8);

        // ADR-029 Amendment 1 (2026-05-31): substrate-intent precedence.
        //
        // When a declared substrate precondition is PRESENT and FAILING, emit
        // `SubstrateGap` regardless of any code witness — the developer declared
        // substrate intent that is not met. A code witness operates in a different
        // channel and does not resolve a broken substrate predicate.
        //
        // Two channels carry an evaluated-and-failed state:
        //   (1) site_requires_eval == Some(WitnessTier::None):
        //       site-side `requires=` (ADR-029 R5) was declared and its predicate failed.
        //         None          → no `requires=` on this site
        //         Some(None)    → `requires=` present but predicate failed
        //         Some(tier>0)  → `requires=` present and passed at `tier`
        //   (2) immune_any_substrate_gap (was: immune_audit.is_some_and(immune_audit_is_substrate_gap)):
        //       deprecated `#[immune(requires=)]` whose predicate failed. Same masking
        //       risk: a code witness must not hide a drifted deprecated substrate claim.
        //       Uses `any()` over ALL matching entries so stacked same-item immunities
        //       cannot mask each other's substrate gaps.
        //       (forward/immune-stacked-same-item-substrate-gap-mask + forward/immune-channel-gate-missing-from-adr029-amd1)
        //
        // The existing `site_requires_tier` (which filters out `None`) is used for
        // the `best_tier` computation; the gate checks `site_requires_eval` directly to
        // distinguish "requires= absent" (None) from "requires= present but failed" (Some(None)).
        let requires_present_and_failed =
            site_requires_eval == Some(WitnessTier::None) || immune_any_substrate_gap;

        let verdict = if requires_present_and_failed {
            // Substrate intent declared and broken — SubstrateGap even when a code
            // witness exists. The two channels are independent; code evidence does not
            // patch a drifted substrate.
            ImmuneVerdict::SubstrateGap
        } else {
            match best_tier {
                Some(tier) => ImmuneVerdict::Defended { tier },
                // No PASSING evidence. Defense intent that engaged the substrate-
                // witness pipeline but did not pass is a substrate gap (intent
                // present; substrate drifted) — distinguished from undefended (no
                // intent at all). Either a site-attached requires= (ADR-029 R5) or a
                // deprecated #[immune(requires=)] can be the engaged-but-failing
                // intent.
                None if site_requires_eval.is_some() || immune_any_substrate_gap => {
                    ImmuneVerdict::SubstrateGap
                },
                None => ImmuneVerdict::Undefended,
            }
        };

        let defended_by = code_witnesses
            .iter()
            .map(|d| format!("{}:{}", d.file.display(), d.line))
            .collect();

        verdicts.push(PresentationVerdict {
            presentation: p.clone(),
            antigen_type: p.antigen_type.clone(),
            verdict,
            defended_by,
        });
    }

    verdicts
}

/// True when an immunity audit represents an engaged-but-failing substrate-
/// witness (`requires =`) — the defense intent is present but the current
/// substrate does not satisfy the predicate. Used to distinguish `substrate-gap`
/// (intent + drift) from `undefended` (no intent) in the ADR-029 verdict.
fn immune_audit_is_substrate_gap(a: &ImmunityAudit) -> bool {
    // Only the substrate-witness path (`requires =`) can yield a substrate gap;
    // it sets `evaluated_predicate`. A code-witness with a NotFound/Missing
    // witness is `undefended`, not `substrate-gap`.
    //
    // Three-valued-logic gate (forward/three-valued-logic-api-boundary-layer):
    // `DisciplinePredicateDeferred` is NOT a substrate gap — it means the predicate
    // contains supply-chain leaves that require `audit_supply_chain()`, i.e., "not
    // yet evaluated here." Collapsing `deferred` → `SubstrateGap` conflates
    // "we tried and it's broken" with "we haven't tried yet." The correct verdict
    // for a deferred predicate is `Indeterminate` (handled by the supply-chain audit
    // path), not `SubstrateGap`. Exclude deferred predicates so only genuinely
    // evaluated-and-failed predicates gate as substrate gaps.
    a.evaluated_predicate.is_some()
        && a.witness_tier == WitnessTier::None
        && a.audit_hint != AuditHint::DisciplinePredicateDeferred
}

/// Evaluate a substrate-witness predicate for one immunity declaration and
/// return the populated [`ImmunityAudit`].
///
/// Called from `audit()` when `immunity.requires_predicate` is `Some`.
///
/// Defends [`crate::stdlib::dogfood::AuditFingerprintSelfReferential`]: this
/// function previously compared a signer's `signed_against_fingerprint` to the
/// sidecar's own stored `current_fingerprint` (Audit-SF-1 — self-referential,
/// staleness always cleared). It now feeds the scan-recomputed
/// `immunity.structural_fingerprint` (read from the item on disk) so real drift
/// is detected. The witness pins that fix.
// ADR-029 migration: this fn IS the failure-locus for AuditFingerprintSelfReferential
// (it once compared a signer's fingerprint to the sidecar's own stored value);
// it `#[presents]` that class. The test
// `audit_sf1_scan_fingerprint_overrides_sidecar_stored_fingerprint` declares it
// defends the class via `#[defended_by]`. `cargo antigen audit` cross-references
// the two and observes the verdict — immunity is observed, not declared.
#[presents(AuditFingerprintSelfReferential)]
pub fn audit_substrate_witness(immunity: &Immunity, predicate_json: &str) -> ImmunityAudit {
    use antigen_attestation::evaluate::evaluate_predicate_with_kind;

    // Deserialize the predicate. The JSON was emitted at macro-expand time by
    // `antigen-macros` and round-trips through the doc marker; any failure here
    // means the marker was corrupted in transit (shouldn't happen, but we surface
    // it as sidecar-schema-invalid rather than panicking).
    let Ok(predicate) = serde_json::from_str::<antigen_attestation::Predicate>(predicate_json)
    else {
        let result = antigen_attestation::EvaluatedPredicate::sidecar_schema_invalid();
        // Sidecar not yet loaded; default kind to Immunity (kind field on this path is unused).
        return immunity_audit_from_evaluated(
            immunity,
            result,
            predicate_json.to_string(),
            antigen_attestation::RatificationKind::Immunity,
        );
    };

    // Load the sidecar. Distinguish missing from schema-invalid so the audit
    // can emit the appropriate hint in each case.
    let sidecar = match load_sidecar(&immunity.file, &immunity.antigen_type) {
        SidecarLoad::Missing => {
            let result = antigen_attestation::EvaluatedPredicate::sidecar_missing();
            return immunity_audit_from_evaluated(
                immunity,
                result,
                predicate_json.to_string(),
                antigen_attestation::RatificationKind::Immunity,
            );
        },
        SidecarLoad::SchemaInvalid => {
            // Sidecar present but failed validation (e.g. NFA-17: CryptoSigned
            // without signature). Emit schema-invalid so the adopter knows the
            // sidecar needs repair, not just re-scaffolding.
            let result = antigen_attestation::EvaluatedPredicate::sidecar_schema_invalid();
            return immunity_audit_from_evaluated(
                immunity,
                result,
                predicate_json.to_string(),
                antigen_attestation::RatificationKind::Immunity,
            );
        },
        SidecarLoad::Ok(r) => r,
    };

    // Match the sidecar item by `item_path` (the rendering produced by
    // `ItemTarget::label()`). Using `items.first()` here was a v0.1 shortcut
    // that silently audited a per-item predicate against the WRONG item
    // whenever two `#[immune]` sites in the same file shared an antigen sidecar
    // — the second site's `audit_substrate_witness` call would evaluate the
    // FIRST site's signers + fingerprint, so signers who signed `first_fn`
    // would silently pass `second_fn`'s immunity (findings/
    // sidecar-first-item-wrong-audit / ATK adversarial pin). The scaffold and
    // sign paths (cargo-antigen/src/main.rs ~2949 and ~2964) write
    // `item_path: item_target.label()`, and existing lookups elsewhere in
    // main.rs (~3479, ~3610) already match on `item.item_path == args.item_path`,
    // so this is the established matching surface. A missing entry (sidecar
    // exists but has no item with this label — e.g., a stale sidecar predating
    // a rename) falls through to `sidecar_missing`, the same failure mode as
    // an entirely-missing sidecar.
    let immunity_label = immunity.item_target.label();
    let Some(item) = sidecar
        .items
        .iter()
        .find(|item| item.item_path == immunity_label)
    else {
        let result = antigen_attestation::EvaluatedPredicate::sidecar_missing();
        return immunity_audit_from_evaluated(
            immunity,
            result,
            predicate_json.to_string(),
            sidecar.kind,
        );
    };

    let ctx = FilesystemAuditContext;
    // Audit-SF-1 (RESOLVED): stale-signer detection uses the scan-computed
    // structural digest rather than the sidecar's stored value.
    //
    // `immunity.structural_fingerprint` is populated at scan time by
    // `antigen_fingerprint::structural_digest` and reflects the item's current
    // code on disk. The evaluator compares `s.signed_against_fingerprint ==
    // current_fingerprint` — using the scan-time digest means a signer who
    // signed against an old fingerprint is now correctly detected as stale when
    // the item's code has changed.
    //
    // Fallback for the legacy deserialization path (sidecars serialized before
    // this field was added): if structural_fingerprint is empty, fall back to the
    // sidecar's stored value (the old self-referential behavior). This preserves
    // backwards-compat for pre-existing sidecars without forcing a re-sign.
    let current_fingerprint: &str = if immunity.structural_fingerprint.is_empty() {
        &item.current_fingerprint
    } else {
        &immunity.structural_fingerprint
    };
    let result = evaluate_predicate_with_kind(
        &predicate,
        item,
        current_fingerprint,
        &immunity.file,
        sidecar.kind,
        &ctx,
    )
    .unwrap_or_else(|_| antigen_attestation::EvaluatedPredicate::sidecar_schema_invalid());

    immunity_audit_from_evaluated(immunity, result, predicate_json.to_string(), sidecar.kind)
}

/// Build an [`ImmunityAudit`] from the output of `evaluate_predicate_with_kind`.
fn immunity_audit_from_evaluated(
    immunity: &Immunity,
    result: antigen_attestation::EvaluatedPredicate,
    predicate_json: String,
    sidecar_kind: antigen_attestation::RatificationKind,
) -> ImmunityAudit {
    let status = WitnessStatus::Resolved {
        location: immunity.file.clone(),
        witness_kind: WitnessKind::SubstrateWitness { kind: sidecar_kind },
    };
    ImmunityAudit {
        immunity: immunity.clone(),
        witness_status: status,
        witness_tier: map_attestation_tier(result.witness_tier),
        audit_hint: map_attestation_audit_hint(result.audit_hint),
        evidence_kind: result.evidence_kind,
        signature_strength: result.signature_strength,
        compound_evidence: false,
        evaluated_predicate: Some(predicate_json),
        // Substrate-witness path: the sidecar IS credited here (this is the
        // `requires =` path), so the code-witness-orphan-sidecar warning does
        // not apply.
        code_witness_sidecar_ignored: false,
        leaf_outcomes: result.leaf_outcomes,
    }
}

/// Map [`antigen_attestation::WitnessTier`] to [`WitnessTier`].
///
/// The two enums are intended to stay in lock-step (defined as peers per
/// `tier.rs`) but are distinct types to avoid a circular crate dependency.
///
/// What enforces that lock-step — and what does NOT:
/// - **Enforced by the compiler (one direction only):** adding a variant to
///   `antigen_attestation::WitnessTier` breaks the exhaustive `match` below.
/// - **Enforced by test, not the compiler:** adding a variant *here* without a
///   peer there, derive parity (e.g. `Hash` on one side and not the other),
///   discriminant parity, and variant-doc parity are guarded by
///   `antigen/tests/atk_witness_tier_parity.rs`. The compiler does not catch
///   any of those — only the parity test does. (This is the
///   `ParallelStateTrackersDiverge` shape, mirroring the framing in
///   `tier.rs`'s `WitnessTier` doc-comment.)
///
/// For v0.1 the mapping is lossless; a future ADR that intentionally diverges
/// the two enums would widen it.
const fn map_attestation_tier(tier: antigen_attestation::WitnessTier) -> WitnessTier {
    match tier {
        antigen_attestation::WitnessTier::None => WitnessTier::None,
        antigen_attestation::WitnessTier::Reachability => WitnessTier::Reachability,
        antigen_attestation::WitnessTier::Execution => WitnessTier::Execution,
        antigen_attestation::WitnessTier::FormalProof => WitnessTier::FormalProof,
    }
}

/// Map [`antigen_attestation::AuditHint`] to [`AuditHint`].
///
/// 1:1 mapping — every substrate-witness hint variant in the attestation
/// crate has a peer in [`AuditHint`]. The two enums are deliberately
/// duplicated so the runtime crate stays serde-stable for the on-disk
/// audit format while the attestation crate can evolve the substrate-
/// witness vocabulary independently.
///
/// rc.1 collapsed substrate hints into `NoneApplicable` /
/// `ExternalToolPrefixRecognized`, which made it impossible to read what
/// the substrate-witness pipeline actually found. rc.2 surfaces the real
/// state — the hint a CI gate or reviewer reads now names the case.
const fn map_attestation_audit_hint(hint: antigen_attestation::AuditHint) -> AuditHint {
    use antigen_attestation::AuditHint as AH;
    match hint {
        AH::DisciplineSidecarMissing => AuditHint::DisciplineSidecarMissing,
        AH::DisciplineSidecarSchemaInvalid => AuditHint::DisciplineSidecarSchemaInvalid,
        AH::DisciplinePredicateFailed => AuditHint::DisciplinePredicateFailed,
        AH::DisciplinePredicateDeferred => AuditHint::DisciplinePredicateDeferred,
        AH::DisciplineSubstrateStale => AuditHint::DisciplineSubstrateStale,
        AH::DisciplineSubstrateDeltaChainNearCap => AuditHint::DisciplineSubstrateDeltaChainNearCap,
        AH::DisciplinePredicatePassedViaDeltaChain => {
            AuditHint::DisciplinePredicatePassedViaDeltaChain
        },
        AH::DisciplinePredicatePassedSubstrateCurrent => {
            AuditHint::DisciplinePredicatePassedSubstrateCurrent
        },
        AH::ToleranceVibesGrade => AuditHint::ToleranceVibesGrade,
        AH::ToleranceSidecarMissing => AuditHint::ToleranceSidecarMissing,
        AH::TolerancePredicateFailed => AuditHint::TolerancePredicateFailed,
        AH::TolerancePredicatePassedSubstrateCurrent => {
            AuditHint::TolerancePredicatePassedSubstrateCurrent
        },
        AH::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance => {
            AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance
        },
        AH::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity => {
            AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity
        },
        AH::DisciplineImmunityToleranceContradiction => {
            AuditHint::DisciplineImmunityToleranceContradiction
        },
    }
}

/// One entry in the function index — a single (path, kind) pair for a name.
#[derive(Debug, Clone)]
struct FunctionEntry {
    location: PathBuf,
    kind: WitnessKind,
}

/// Index of function name → all (file path, kind) pairs sharing that name.
///
/// W7 (A2) extends the flat name index to track *all* candidates for a name,
/// so `validate_witness` can detect ambiguity (ATK-A2-005). When more than one
/// function shares a name, the witness resolves to `WitnessStatus::Ambiguous`
/// rather than silently picking whichever was indexed last.
///
/// Cross-cutting limitations remaining for A3+:
/// - Module-qualified paths (`crate::foo::bar` parsing) require module-graph
///   resolution; for v0.1 we detect ambiguity and require the user to qualify
///   the witness (e.g., rename one of the conflicting functions, or use a
///   path that is unique).
/// - Functions inside `impl` blocks (method names, not free functions);
///   currently recorded with the same shape — matching is name-only.
type FunctionIndex = std::collections::HashMap<String, Vec<FunctionEntry>>;

// P0b dogfood mark (v0.4 keystone): the silent-skip twin of
// `scan::walk::scan_workspace_inner`. This audit-time index walk reads every
// `.rs` file with `let Ok(content) = read_to_string(..) else { continue };` and
// then `if let Ok(file) = parse_file(..)` — so an unreadable OR unparseable file
// is silently dropped from the witness-function index, and a later witness lookup
// can report "function not found" (an audit miss) when the truth is "the file
// holding it could not be read." Same felt-class as the scanner twin: an
// incomplete index presented as a complete one. The shared shape (WalkDir +
// read-or-continue + parse-or-skip, with `read_to_string`/`parse_file`/`visit_file`
// calls) is exactly what the PROPOSE anti-unifier clusters these two on.
#[dread(
    trigger = "collect_function_index silently `continue`s past a file it cannot read \
               and skips one it cannot parse, so the witness-function index is built \
               over an INCOMPLETE corpus; a downstream 'witness function not found' \
               cannot be told apart from 'the file was unreadable/unparseable'."
)]
fn collect_function_index(root: &Path) -> FunctionIndex {
    use syn::visit::Visit;
    use walkdir::WalkDir;

    let exclusions = ["target", ".git", "node_modules"];
    let mut index = FunctionIndex::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|x| *x == name)
            } else {
                true
            }
        })
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };

        if let Ok(file) = syn::parse_file(&content) {
            let mut visitor = FunctionIndexVisitor {
                file_path: entry.path().to_path_buf(),
                source: &content,
                index: &mut index,
            };
            visitor.visit_file(&file);
        }
    }

    index
}

struct FunctionIndexVisitor<'a> {
    file_path: PathBuf,
    /// Source text of the file being walked. Carried for symmetry with
    /// `scan::ScanVisitor` and for future span-anchored diagnostics; the
    /// pre-W5 textual `source.contains("proptest!")` sentinel was removed
    /// when `visit_macro` took over proptest classification.
    #[allow(
        dead_code,
        reason = "reserved for span-anchored diagnostic work \
        that mirrors scan::ScanVisitor::source"
    )]
    source: &'a str,
    index: &'a mut FunctionIndex,
}

impl FunctionIndexVisitor<'_> {
    /// Classify a function by its own attributes.
    ///
    /// W5 (sweep A2): the prior heuristic — `self.source.contains("proptest!")`
    /// — over-classified every function in any file mentioning the string
    /// `proptest!` (including doc comments) as `WitnessKind::Proptest`.
    /// Replaced by structural detection: `visit_macro` registers
    /// proptest-internal function names with `WitnessKind::Proptest` directly.
    ///
    /// W7 (sweep A2): distinguish `#[test] #[ignore]` from a running `#[test]`.
    /// Per ADR-005 Amendment 3 and ATK-A2-012, an ignored test is weaker
    /// evidence than a runnable test — `cargo test` skips it by default.
    /// We tag it as `WitnessKind::IgnoredTest` so the audit can emit the
    /// `TestAttributePresentIgnoreSkipped` hint.
    fn detect_kind(attrs: &[syn::Attribute]) -> WitnessKind {
        let has_test = attrs.iter().any(|a| a.path().is_ident("test"));
        let has_ignore = attrs.iter().any(|a| a.path().is_ident("ignore"));
        match (has_test, has_ignore) {
            (true, true) => WitnessKind::IgnoredTest,
            (true, false) => WitnessKind::Test,
            (false, _) => WitnessKind::Function,
        }
    }
}

/// Extract top-level `fn IDENT` names from a `proptest! { ... }` macro body.
///
/// `proptest!` is a function-like macro that takes a sequence of test-shaped
/// declarations:
///
/// ```ignore
/// proptest! {
///     #[test]
///     fn name(args in strategy) { body }
///     ...
/// }
/// ```
///
/// The body's tokens contain `fn IDENT` at the top level for each test;
/// nested function definitions live inside `Group` tokens (the body block of
/// each fn) which a top-level token-iterator does not descend into. So a
/// linear walk that yields `name` whenever it sees `fn` followed by an
/// identifier captures exactly the proptest test names — no more, no less.
///
/// Why not parse with `syn` directly? `proptest!`'s grammar (`fn name(args
/// in strategy)`) is not a valid Rust function signature: the `in` keyword
/// inside the parameter list is custom syntax. `syn::ItemFn::parse` rejects
/// the body. The token walk below is grammar-aware enough for our purpose
/// (extracting names) without committing to parsing the strategy expressions.
fn extract_proptest_fn_names(tokens: &proc_macro2::TokenStream) -> Vec<String> {
    use proc_macro2::TokenTree;
    let mut names = Vec::new();
    let mut iter = tokens.clone().into_iter();
    while let Some(tt) = iter.next() {
        if let TokenTree::Ident(i) = &tt {
            if i == "fn" {
                if let Some(TokenTree::Ident(name)) = iter.next() {
                    names.push(name.to_string());
                }
            }
        }
    }
    names
}

/// Whether a macro path's last segment is `name`. Mirrors the
/// `attr_is`-style test in `scan.rs`: matches both `#[proptest!(...)]`-style
/// bare names and `proptest::proptest!(...)` path-qualified forms.
fn macro_path_last_is(path: &syn::Path, name: &str) -> bool {
    path.segments.last().is_some_and(|s| s.ident == name)
}

impl FunctionIndexVisitor<'_> {
    /// Push a candidate entry for `name`. W7: every distinct (location, kind)
    /// pair is preserved so `validate_witness` can detect ambiguity. The
    /// pre-W7 behavior (silent first-wins) is the bug ATK-A2-005 named.
    fn push(&mut self, name: String, kind: WitnessKind) {
        self.index.entry(name).or_default().push(FunctionEntry {
            location: self.file_path.clone(),
            kind,
        });
    }
}

impl<'ast> syn::visit::Visit<'ast> for FunctionIndexVisitor<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        self.push(name, kind);
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        let name = item.sig.ident.to_string();
        let kind = Self::detect_kind(&item.attrs);
        self.push(name, kind);
        syn::visit::visit_impl_item_fn(self, item);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        // W5: structural proptest! detection. When the macro path's last
        // segment is `proptest`, walk its tokens for `fn IDENT` patterns
        // and register each name with `WitnessKind::Proptest`.
        if macro_path_last_is(&mac.path, "proptest") {
            for name in extract_proptest_fn_names(&mac.tokens) {
                self.push(name, WitnessKind::Proptest);
            }
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Determine the witness status for a single witness identifier string.
///
/// Resolution priority per ADR-013 + ADR-005 Amendment 3:
/// 1. Empty witness → `Missing`
/// 2. External-tool prefix (`clippy::`, `kani::`, ...) → `External`
/// 3. Phantom-type witness shape → `Resolved { PhantomType }`
/// 4. Workspace function lookup → `Resolved` / `Ambiguous` / `NotFound`
fn validate_witness(witness: &str, index: &FunctionIndex) -> WitnessStatus {
    // Normalize whitespace: the scan path records witnesses via ToTokens, which
    // emits spaced token form (`clippy :: no_panic_in_drop`, `PolarityProof :: < T > :: verified`).
    // Collapse all spacing around `::` and `<>` so every downstream detector
    // works on compact form regardless of source (hand-written or scan-path).
    let normalized_owned: String = {
        let collapsed = witness.split_whitespace().collect::<Vec<_>>().join(" ");
        collapsed
            .replace(" :: ", "::")
            .replace(":: ", "::")
            .replace(" ::", "::")
            .replace("< ", "<")
            .replace(" >", ">")
    };
    let trimmed = normalized_owned.trim();
    if trimmed.is_empty() {
        return WitnessStatus::Missing;
    }

    // Detect external-tool delegations.
    if let Some(tool) = detect_external_tool(trimmed) {
        return WitnessStatus::External {
            tool_hint: tool.to_string(),
        };
    }

    // Detect phantom-type witness shapes (ADR-013): `Path::<Args>::ctor` or
    // `Path::<Args>` or `Path` with trailing `()`. The shape recognition is
    // structural — we don't validate that the type exists.
    if let Some(phantom) = detect_phantom_type_witness(trimmed) {
        return WitnessStatus::Resolved {
            location: PathBuf::new(),
            witness_kind: phantom,
        };
    }

    // Resolve as a workspace-local function. The witness might be a path
    // (`module::function`); take the last segment as the function name.
    let function_name = trimmed
        .rsplit("::")
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches("()")
        .trim();

    let candidates = index.get(function_name);
    let Some(candidates) = candidates else {
        return WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        };
    };

    match candidates.as_slice() {
        [] => WitnessStatus::NotFound {
            reason: format!(
                "no function named `{function_name}` found in any .rs file under the scan root"
            ),
        },
        [only] => WitnessStatus::Resolved {
            location: only.location.clone(),
            witness_kind: only.kind.clone(),
        },
        many => WitnessStatus::Ambiguous {
            candidates: many.iter().map(|e| e.location.clone()).collect(),
        },
    }
}

/// Recognize a phantom-type witness shape per ADR-013.
///
/// Matches: `Type`, `Type::ctor`, `Type::<Args>`, `Type::<Args>::ctor`,
/// optionally with trailing `()`. The `<Args>` group, when present, contains
/// comma-separated type parameters.
///
/// We deliberately accept *any* type-name shape (capital-leading identifier
/// path) here because v0.1's audit has no symbol table — we cannot tell
/// whether `PolarityProof` refers to a real type. The `audit_hint` carries
/// the warning to verify the constructor is sealed; that's the recognize-
/// and-warn discipline ADR-013 §OQ1 specifies.
///
/// Returns `None` when the witness looks more like a function path than a
/// phantom-type construction (lowercase final segment with no type-param
/// list and no trailing `()`-after-`::`-segment ambiguity). The heuristic:
/// if the path contains a `<...>` segment, OR the last segment starts with
/// an uppercase letter (typical Rust type-name convention) AND there are
/// no trailing `()` (which would indicate a function call), treat it as
/// a phantom-type witness candidate.
fn detect_phantom_type_witness(witness: &str) -> Option<WitnessKind> {
    // Input is pre-normalized by validate_witness — compact token spacing guaranteed.
    let trimmed = witness.trim().trim_end_matches("()").trim();
    let has_turbofish = trimmed.contains("::<");
    if !has_turbofish {
        // No turbofish = not a phantom-type witness shape we recognize. The
        // bare-type-name shape (`Foo`) is indistinguishable from a function
        // path at this layer; we let the function-index path handle it.
        return None;
    }

    // Split into pre-turbofish, type-params, post-turbofish-ctor.
    let (before, after) = trimmed.split_once("::<")?;
    let (params_raw, ctor_part) = after.split_once('>')?;

    // Guard: nested generics like `Foo::<Option<Bar>, Baz>::new` make
    // split_once('>') fire at the inner `>`, leaving params_raw with
    // unbalanced `<`. Return None rather than emit FormalProof for a
    // garbled parse — let it fall through to function-index (NotFound).
    let open_count = params_raw.chars().filter(|&c| c == '<').count();
    if open_count > 0 {
        return None;
    }

    let proof_type = before.trim().to_string();
    let type_params: Vec<String> = params_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // Strip any remaining closing `>`s and the `::` separator left by
    // split_once('>') on nested-generic inputs like `Foo::<Bar<Baz>>::new`.
    let constructor = ctor_part
        .trim_start_matches(['>', ':'])
        .trim()
        .trim_end_matches("()")
        .trim();
    let constructor = if constructor.is_empty() {
        None
    } else {
        Some(constructor.to_string())
    };

    Some(WitnessKind::PhantomType {
        proof_type,
        type_params,
        constructor,
    })
}

/// Detect whether the witness references an external tool we recognize.
fn detect_external_tool(witness: &str) -> Option<&'static str> {
    let lower = witness.to_ascii_lowercase();
    if lower.starts_with("clippy::") || lower.contains("clippy_") {
        Some("clippy")
    } else if lower.starts_with("kani::") || lower.contains("kani_proof") {
        Some("kani")
    } else if lower.starts_with("prusti::") {
        Some("prusti")
    } else if lower.starts_with("creusot::") {
        Some("creusot")
    } else if lower.starts_with("verus::") {
        Some("verus")
    } else if lower.starts_with("mutants::") {
        Some("cargo-mutants")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    //! White-box unit tests of the witness-resolution machinery (the function
    //! index, `validate_witness`, `detect_external_tool` / `detect_phantom_type_witness`,
    //! and the W5 structural-`proptest!` detection). Moved here from the former
    //! monolithic `audit.rs` test module per ADR-036 (tests follow the code they
    //! exercise); they construct `FunctionIndexVisitor` directly, so they belong
    //! with the internals rather than reaching them through a test-only re-export.
    use std::path::PathBuf;

    use super::{
        FunctionEntry, FunctionIndex, FunctionIndexVisitor, detect_external_tool,
        detect_phantom_type_witness, extract_proptest_fn_names, macro_path_last_is,
        validate_witness,
    };
    use crate::audit::{WitnessKind, WitnessStatus};
    #[test]
    fn detect_clippy_external_tool() {
        assert_eq!(
            detect_external_tool("clippy::no_panic_in_drop"),
            Some("clippy")
        );
    }

    #[test]
    fn detect_kani_external_tool() {
        assert_eq!(
            detect_external_tool("kani::proof_drop_safety"),
            Some("kani")
        );
    }

    #[test]
    fn detect_no_tool_for_local_function() {
        assert_eq!(detect_external_tool("safe_type_drop_no_panic_test"), None);
    }

    #[test]
    fn validate_witness_strips_path_prefix() {
        let mut idx = FunctionIndex::new();
        idx.insert(
            "my_test".to_string(),
            vec![FunctionEntry {
                location: PathBuf::from("src/lib.rs"),
                kind: WitnessKind::Test,
            }],
        );

        let status = validate_witness("module::path::my_test", &idx);
        assert!(matches!(status, WitnessStatus::Resolved { .. }));
    }

    #[test]
    fn validate_witness_reports_missing_when_empty() {
        let idx = FunctionIndex::new();
        let status = validate_witness("", &idx);
        assert_eq!(status, WitnessStatus::Missing);
    }

    #[test]
    fn validate_witness_reports_not_found_for_unknown() {
        let idx = FunctionIndex::new();
        let status = validate_witness("nonexistent_test", &idx);
        assert!(matches!(status, WitnessStatus::NotFound { .. }));
    }

    // ========================================================================
    // W5 — structural proptest! witness detection.
    //
    // Pre-W5, `detect_kind` did `self.source.contains("proptest!")` as a
    // sentinel — if the source string contained that text anywhere, every
    // function in the file was tagged `WitnessKind::Proptest`. Doc comments
    // mentioning the macro for explanatory purposes triggered the same
    // over-classification.
    //
    // W5 lifts this to structural detection via `visit_macro` + token-walking
    // the macro body for `fn IDENT` patterns. These tests are the contract
    // pinning the W5 behavior without needing a full filesystem fixture.
    // ========================================================================

    /// Run the function-index walk against an in-memory source string.
    /// Mirrors what `collect_function_index` does per-file but without
    /// touching disk — gives the W5 unit tests a tight feedback loop.
    fn index_from_str(source: &str) -> FunctionIndex {
        use syn::visit::Visit;
        let file = syn::parse_file(source).expect("source must parse");
        let mut index = FunctionIndex::new();
        let mut visitor = FunctionIndexVisitor {
            file_path: PathBuf::from("<test>.rs"),
            source,
            index: &mut index,
        };
        visitor.visit_file(&file);
        index
    }

    /// Helper for tests that expect a single index entry for a name.
    /// Panics with a clear message if the name is unindexed or ambiguous.
    fn unique_kind(idx: &FunctionIndex, name: &str) -> WitnessKind {
        let entries = idx.get(name).unwrap_or_else(|| panic!("{name} indexed"));
        assert_eq!(
            entries.len(),
            1,
            "expected single index entry for {name}, got {entries:?}",
        );
        entries[0].kind.clone()
    }

    #[test]
    fn w5_proptest_inner_fns_are_classified_proptest() {
        let src = r"
            proptest! {
                #[test]
                fn first_proptest(x in 0u32..100) {
                    assert!(x < 100);
                }

                #[test]
                fn second_proptest(x in 0u32..100, y in 0u32..100) {
                    assert!(x + y < 200);
                }
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(unique_kind(&idx, "first_proptest"), WitnessKind::Proptest);
        assert_eq!(unique_kind(&idx, "second_proptest"), WitnessKind::Proptest);
    }

    #[test]
    fn w5_proptest_path_qualified_macro_is_recognized() {
        // The fixture canonical form is `proptest::proptest!`, matching how
        // the `proptest` crate is typically imported. The W5 helper
        // `macro_path_last_is` checks the LAST segment, so any path ending
        // in `proptest` matches.
        let src = r"
            proptest::proptest! {
                #[test]
                fn qualified_form_proptest(x in 0u32..100) {
                    assert!(x < 100);
                }
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "qualified_form_proptest"),
            WitnessKind::Proptest,
        );
    }

    #[test]
    fn w5_test_function_outside_proptest_is_classified_test() {
        // A regular `#[test]` outside any proptest! block must remain
        // `WitnessKind::Test`. The pre-W5 sentinel would have over-classified
        // this as Proptest if the file contained the string `proptest!`
        // anywhere; this test exercises the negative case directly.
        let src = r"
            // Doc-style comment mentioning proptest! for explanation purposes.
            // Pre-W5 this string in the source was sufficient to flag every
            // function in the file as Proptest. W5 must not regress to that.
            #[test]
            fn plain_test() {
                assert_eq!(2 + 2, 4);
            }

            proptest! {
                #[test]
                fn proptest_one(x in 0u32..10) {
                    assert!(x < 10);
                }
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "plain_test"),
            WitnessKind::Test,
            "plain_test outside proptest! must be Test, not Proptest, even when \
             the same file contains a proptest! invocation",
        );
        assert_eq!(unique_kind(&idx, "proptest_one"), WitnessKind::Proptest);
    }

    #[test]
    fn w5_doc_comment_mentioning_proptest_does_not_over_classify() {
        // The exact regression the pre-W5 textual sentinel had: a doc
        // comment containing the literal string `proptest!` would tag
        // every function in the file as Proptest. W5's structural detection
        // only fires on actual macro invocations, so this `#[test]` stays Test.
        let src = r"
            /// This function has nothing to do with proptest! — the macro
            /// is named here only for documentation.
            #[test]
            fn doc_comment_only_test() {
                assert!(true);
            }
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "doc_comment_only_test"),
            WitnessKind::Test,
            "doc-comment mention must not trigger Proptest",
        );
    }

    #[test]
    fn w5_plain_function_is_classified_function() {
        let src = r"
            fn no_attribute_function() {}
        ";
        let idx = index_from_str(src);
        assert_eq!(
            unique_kind(&idx, "no_attribute_function"),
            WitnessKind::Function,
        );
    }

    #[test]
    fn w5_extract_proptest_fn_names_skips_nested() {
        // Nested function definitions inside a fn body live in a Group token;
        // the top-level token walk should not descend into them. This locks
        // the "nested fn doesn't get registered as a proptest test" invariant.
        use proc_macro2::TokenStream;
        let tokens: TokenStream = r"
            #[test]
            fn outer(x in 0u32..10) {
                fn nested_helper() {}
                assert!(x < 10);
            }
        "
        .parse()
        .unwrap();
        let names = extract_proptest_fn_names(&tokens);
        assert_eq!(names, vec!["outer".to_string()]);
    }

    #[test]
    fn w5_macro_path_last_is_handles_qualified_paths() {
        let bare: syn::Path = syn::parse_str("proptest").unwrap();
        let qualified: syn::Path = syn::parse_str("proptest::proptest").unwrap();
        let unrelated: syn::Path = syn::parse_str("other_crate::other_macro").unwrap();
        assert!(macro_path_last_is(&bare, "proptest"));
        assert!(macro_path_last_is(&qualified, "proptest"));
        assert!(!macro_path_last_is(&unrelated, "proptest"));
    }

    #[test]
    fn detect_phantom_nested_generic_returns_none() {
        // `Witnessed::<Option<MyType>, MyWitness>::try_new` has a nested `<>`
        // inside the type-param region. split_once('>') fires at the inner `>`,
        // producing malformed fields. The balanced-bracket guard must return None
        // so audit falls through to function-index (NotFound), not FormalProof.
        assert_eq!(
            detect_phantom_type_witness("Witnessed::<Option<MyType>, MyWitness>::try_new"),
            None,
        );
        // Simple non-nested shape must still work.
        assert!(matches!(
            detect_phantom_type_witness("PolarityProof::<FrameTranslation>::verified"),
            Some(WitnessKind::PhantomType { .. }),
        ));
    }
}

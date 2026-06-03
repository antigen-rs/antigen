//! Witness validation and immunity audit.
//!
//! The audit module operates a layer above [`crate::scan`]: where scan finds
//! antigen-related declarations as syntactic facts, audit reasons about whether
//! the immunity claims are actually backed by working witnesses.
//!
//! This is the "trust-boundary check" required by ADR-005 (sub-clause F at every
//! trust boundary). A declaration of `#[immune(X, witness = Y)]` is meaningful
//! only if `Y` resolves to a real function, test, lint reference, or proof that
//! demonstrates immunity. A marker without a working witness is not a claim.
//!
//! ## What audit checks (v0.0.1)
//!
//! - Witness identifiers resolve to a function/test in the workspace
//! - Witness functions have a recognized testing attribute (`#[test]`, recognizable
//!   `proptest!` invocation, or known external delegations like `clippy::lint_name`)
//!
//! ## What audit doesn't check (yet)
//!
//! - **Witness execution**: doesn't actually run the test/proptest. The team
//!   should add `cargo test` integration in sweep A3+.
//! - **Witness semantics**: doesn't verify the witness asserts the antigen's
//!   specific failure pattern. That requires fingerprint-aware reasoning.
//! - **External tool delegation**: clippy/kani/prusti adapters are stubbed with
//!   "external; manual validation required" status. Sweep A3+ adds adapters.
//! - **Cross-crate witnesses**: a witness that lives in a dependency isn't
//!   followed. v0.0.1 audit is workspace-local only — A3 sweep extends this
//!   via cross-crate source walking (per scope-lock).

use std::path::{Path, PathBuf};

use antigen_macros::{antigen_tolerance, presents};
use serde::{Deserialize, Serialize};

use crate::scan::{Immunity, ScanReport};

mod types;
pub use types::{
    evidence_kind_from_status, AuditHint, AuditReport, FrameState, ImmuneVerdict, ImmunityAudit,
    InheritedUnaddressed, PresentationVerdict, WitnessKind, WitnessStatus, WitnessTier,
    WorkVerdict, CLONAL_ITERATIONS_DEFAULT_FLOOR, IGG_HISTORICAL_SPAN_DEFAULT_FLOOR,
};

// ============================================================================
// Deferred-Defense Family audit (ADR-023)
// ============================================================================

/// Audit result for a single deferred-defense declaration.
///
/// Each deferred-defense site is evaluated against the current UTC date
/// and the relevant workspace config caps to produce a hint that reflects
/// its current state in the loudness-as-discipline lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredDefenseAudit {
    /// The original deferred-defense declaration from the scan.
    pub declaration: crate::scan::DeferredDefense,
    /// The hint code reflecting this declaration's current state.
    pub hint: AuditHint,
}

/// Aggregate deferred-defense audit report.
///
/// Consumed by `cargo antigen defer status` and `cargo antigen audit`
/// to surface the loudness-as-discipline state of all deferred defenses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeferredDefenseAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<DeferredDefenseAudit>,
    /// Count of active (not yet expired) deferred defenses.
    pub active_count: usize,
    /// Count of expired deferred defenses past their `until` date.
    pub expired_count: usize,
    /// Count of stale deferred defenses (significantly past `until`).
    pub stale_count: usize,
}

/// Evaluate all deferred-defense declarations in a `ScanReport` against
/// the current UTC date, producing a `DeferredDefenseAuditReport`.
///
/// This is the v0.2 audit implementation. All date comparisons use UTC
/// per ADR-023 §Enforcement-Surface.
///
/// `stale_grace_days`: how many days past `until` before `anergy-stale`
/// (vs `anergy-co-stimulation-not-arrived`). Default 30 days.
#[must_use]
pub fn audit_deferred_defenses(
    scan: &crate::scan::ScanReport,
    stale_grace_days: i64,
) -> DeferredDefenseAuditReport {
    use chrono::Utc;

    let today = Utc::now().date_naive();
    let mut audits = Vec::new();
    let mut active_count = 0usize;
    let mut expired_count = 0usize;
    let mut stale_count = 0usize;

    for decl in &scan.deferred_defenses {
        let hint = evaluate_deferred_defense_hint(decl, today, stale_grace_days);

        // Tally
        match &hint {
            AuditHint::AnergyActive
            | AuditHint::ImmunosuppressActive
            | AuditHint::PoxpartyActive
            | AuditHint::OrientActive => {
                active_count += 1;
            }
            AuditHint::AnergyCostimulationNotArrived
            | AuditHint::ImmunosuppressExpired
            | AuditHint::PoxpartyOutcomePending
            | AuditHint::OrientPendingActionRequired => {
                expired_count += 1;
            }
            AuditHint::AnergyStale | AuditHint::ImmunosuppressDurationCapExceeded => {
                // Cap-exceeded is the most-overstayed immunosuppress state — it
                // outlived its own declared hard cap; classify as stale (escalate)
                // alongside anergy-stale, not merely expired.
                stale_count += 1;
            }
            _ => {}
        }

        audits.push(DeferredDefenseAudit {
            declaration: decl.clone(),
            hint,
        });
    }

    DeferredDefenseAuditReport {
        audits,
        active_count,
        expired_count,
        stale_count,
    }
}

/// Derive the `AuditHint` for a single deferred-defense declaration.
///
/// UTC date comparison throughout per ADR-023.
fn evaluate_deferred_defense_hint(
    decl: &crate::scan::DeferredDefense,
    today: chrono::NaiveDate,
    stale_grace_days: i64,
) -> AuditHint {
    use crate::scan::DeferredDefenseKind;

    match &decl.kind {
        DeferredDefenseKind::Anergy => {
            // `#[anergy]` REQUIRES `until` (ADR-023 A5: anergy without a time-bound
            // degrades to silent tolerance). Mirror the Orient arm: match on the
            // PRESENCE of `until` first so a present-but-malformed date does NOT
            // collapse into the absent-date grace path. Previously `unwrap_or("")`
            // made `until=None` and `until=Some("not-a-date")` indistinguishable —
            // both parsed to `None` and fell to AnergyActive, so a typo'd deadline
            // silently granted permanent active status with zero diagnostic.
            match decl.until.as_deref() {
                // Absent `until`: only arises for hand-built/legacy scan records
                // (the macro parse-gate requires a valid `until`). Legacy grace path.
                None | Some("") => AuditHint::AnergyActive,
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::AnergyActive,
                    // Genuine past date → tally by staleness against the grace window.
                    Some(until) => {
                        let days_past = (today - until).num_days();
                        if days_past > stale_grace_days {
                            AuditHint::AnergyStale
                        } else {
                            AuditHint::AnergyCostimulationNotArrived
                        }
                    }
                    // Present-but-unparseable (typo like "2026-13-99", "soon"): an
                    // INTENDED deadline that resolves to nothing → unresolved
                    // co-stimulation, not a grace. Escalate (not silently active).
                    None => AuditHint::AnergyCostimulationNotArrived,
                },
            }
        }
        DeferredDefenseKind::Immunosuppress => {
            // Duration-cap enforcement (ADR-023): `#[immunosuppress(since = D,
            // duration_cap = N)]` is capped at N days from D. Once `since + cap`
            // is in the past, the suppression has overstayed its hard cap —
            // emit ImmunosuppressDurationCapExceeded. This is checked FIRST
            // because an exceeded cap is the loudest state (a suppression that
            // outlived its own declared limit). Previously `since`/`duration_cap`
            // lived only as unparsed `see[]` string tags, so this hint had zero
            // emission sites; they are now typed fields the audit can read.
            if let (Some(since), Some(cap)) = (decl.since.as_deref(), decl.duration_cap) {
                if let Some(since_date) = parse_iso_date(since) {
                    let elapsed = (today - since_date).num_days();
                    if let Ok(cap_days) = i64::try_from(cap) {
                        if elapsed > cap_days {
                            return AuditHint::ImmunosuppressDurationCapExceeded;
                        }
                    }
                }
            }
            // `until` is REQUIRED (ADR-023). Same Orient-style split: present-but-
            // malformed must escalate, not silently stay Active. Previously a typo
            // like "2026/01/01" (slash format, looks like a past date to a human but
            // fails ISO parse) collapsed via `unwrap_or("")` to ImmunosuppressActive.
            match decl.until.as_deref() {
                // Absent `until`: hand-built/legacy only (macro requires it). Grace.
                None | Some("") => AuditHint::ImmunosuppressActive,
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::ImmunosuppressActive,
                    // Past date OR present-but-unparseable: the suppression's declared
                    // expiry arrived (or was a typo that resolves to nothing). Either
                    // way it has outlived its intended bound → Expired, not Active.
                    _ => AuditHint::ImmunosuppressExpired,
                },
            }
        }
        DeferredDefenseKind::Poxparty => {
            // `until` is REQUIRED (ADR-023). Same Orient-style split: present-but-
            // malformed must escalate, not silently stay Active. Previously a typo
            // like "soon" collapsed via `unwrap_or("")` to PoxpartyActive forever.
            match decl.until.as_deref() {
                // Absent `until`: hand-built/legacy only (macro requires it). Grace.
                None | Some("") => AuditHint::PoxpartyActive,
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::PoxpartyActive,
                    // Past date OR present-but-unparseable: the isolation window's
                    // declared horizon arrived (or was a typo) → the outcome is due
                    // for recording, not silently active. Escalate.
                    _ => AuditHint::PoxpartyOutcomePending,
                },
            }
        }
        DeferredDefenseKind::Orient => {
            // ADR-023: `#[orient]` REQUIRES `until` (the orientation-period
            // horizon). The audit OBSERVES that date: once it has passed, the
            // orientation period elapsed without the failure-class being
            // resolved — surface OrientPendingActionRequired loudly rather than
            // perpetually reporting OrientActive (a deferred defense whose
            // deadline arrived must escalate, not stay silently green).
            // Match on the PRESENCE of `until` first, so a present-but-malformed
            // date does NOT collapse into the absent-date grace path. Previously
            // `unwrap_or("")` made `until=None` and `until=Some("not-a-date")`
            // indistinguishable — both fell to OrientActive, so a typo'd deadline
            // silently granted permanent green with zero diagnostic.
            match decl.until.as_deref() {
                // Absent `until`: only arises for hand-built/legacy scan records
                // (the macro parse-gate requires a valid `until`). Legacy grace
                // path — don't fabricate an escalation.
                None | Some("") => AuditHint::OrientActive,
                // Future deadline → still active. Everything else for a PRESENT
                // `until` escalates: a past date is an elapsed orientation, and a
                // present-but-unparseable date (typo like "2026-13-99", "2099/01/01",
                // or "soon") is an INTENDED-but-broken deadline that resolves to
                // nothing. Both are unresolved orientations needing action — not a
                // grace. (Only an ABSENT until takes the legacy grace path above.)
                Some(s) => match parse_iso_date(s) {
                    Some(until) if until >= today => AuditHint::OrientActive,
                    _ => AuditHint::OrientPendingActionRequired,
                },
            }
        }
    }
}

/// Parse an ISO-8601 date string for audit-time UTC comparison.
/// Returns `None` if the string is not a valid date.
fn parse_iso_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Filesystem-backed [`antigen_attestation::EvaluationContext`] for use
/// during real audit runs. Reads docs and oracles directly from disk; reads
/// git trailers by shelling out to `git interpret-trailers`. Tests in
/// `antigen-attestation` use an in-memory context instead (see
/// `evaluate.rs` `TestContext`).
struct FilesystemAuditContext;

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
enum SidecarLoad {
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
fn load_sidecar(immunity_file: &Path, antigen_type: &str) -> SidecarLoad {
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
        // function index).
        let immunity_audit = immunity.requires_predicate.as_ref().map_or_else(
            || {
                let status = validate_witness(&immunity.witness, &workspace_functions);
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
            },
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
                }
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
fn audit_substrate_witness(immunity: &Immunity, predicate_json: &str) -> ImmunityAudit {
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
        }
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
        }
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
        }
        AH::DisciplinePredicatePassedSubstrateCurrent => {
            AuditHint::DisciplinePredicatePassedSubstrateCurrent
        }
        AH::ToleranceVibesGrade => AuditHint::ToleranceVibesGrade,
        AH::ToleranceSidecarMissing => AuditHint::ToleranceSidecarMissing,
        AH::TolerancePredicateFailed => AuditHint::TolerancePredicateFailed,
        AH::TolerancePredicatePassedSubstrateCurrent => {
            AuditHint::TolerancePredicatePassedSubstrateCurrent
        }
        AH::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance => {
            AuditHint::DisciplineSidecarKindMismatchExpectedImmunityGotTolerance
        }
        AH::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity => {
            AuditHint::ToleranceSidecarKindMismatchExpectedToleranceGotImmunity
        }
        AH::DisciplineImmunityToleranceContradiction => {
            AuditHint::DisciplineImmunityToleranceContradiction
        }
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

// ============================================================================
// Supply-Chain Defense Family audit (ADR-025)
// ============================================================================

/// One result of evaluating a supply-chain witness leaf against a workspace.
///
/// Each entry carries the source-side identity (file + line where the
/// `#[immune(X, requires = <supply-chain-leaf>)]` was declared), the
/// antigen type it backs, and the [`AuditHint`] the evaluation produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplyChainAudit {
    /// The antigen type the leaf is backing (`ContentHashMismatch`,
    /// `UnpinnedDependency`, etc.). Mirrors `Immunity::antigen_type`.
    pub antigen_type: String,
    /// Source file the immunity declaration lives in.
    pub file: PathBuf,
    /// Line number of the immunity attribute.
    pub line: usize,
    /// Crate name the leaf targeted (extracted from leaf args).
    pub crate_name: String,
    /// Crate version the leaf targeted (if applicable). Empty for
    /// leaves that don't take a version (e.g., `dep_pinned`).
    pub version: String,
    /// The audit hint the evaluation produced.
    pub hint: AuditHint,
    /// Optional structured detail (e.g., for `ContentHashMismatch`,
    /// the recorded vs current hash strings).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Aggregate result of [`audit_supply_chain`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SupplyChainAuditReport {
    /// Per-immunity audit entries (one per supply-chain leaf encountered).
    pub audits: Vec<SupplyChainAudit>,
    /// Count of entries whose hint denotes the leaf evaluation passed.
    pub pass_count: usize,
    /// Count of entries whose hint denotes a failure (mismatch,
    /// malformed sidecar, missing attestation, rubber-stamp, etc.).
    pub fail_count: usize,
}

impl SupplyChainAuditReport {
    /// True when no failure hints were emitted.
    #[must_use]
    pub const fn all_pass(&self) -> bool {
        self.fail_count == 0
    }
}

/// Audit supply-chain substrate-witness leaves across a scan report.
///
/// Walks every [`crate::scan::Immunity`] in the report. When the immunity
/// has a `requires_predicate` containing one or more of the five
/// supply-chain leaves (`dep_pinned`, `dep_attested`,
/// `maintainer_unchanged`, `content_hash_matches`, `sandbox_clean`), the
/// audit evaluates each leaf via
/// [`crate::supply_chain::evaluate`] against `workspace_root` and emits a
/// [`SupplyChainAudit`] entry per leaf.
///
/// **Why the standard `audit()` doesn't do this**: the standard
/// substrate-witness pipeline returns `false` for the supply-chain leaves
/// (honest-tier-naming per ADR-005 Amendment 2). The supply-chain audit
/// is the dedicated evaluator that knows how to drive
/// `evaluate_content_hash_matches` against `Cargo.lock` +
/// `.attest/supply-chain/content-hash/`, etc. Callers SHOULD invoke
/// both `audit()` and `audit_supply_chain()` for full coverage.
#[must_use]
pub fn audit_supply_chain(report: &ScanReport, workspace_root: &Path) -> SupplyChainAuditReport {
    let mut audits: Vec<SupplyChainAudit> = Vec::new();

    for immunity in &report.immunities {
        let Some(json) = &immunity.requires_predicate else {
            continue;
        };
        let Ok(predicate) = serde_json::from_str::<antigen_attestation::Predicate>(json) else {
            continue;
        };

        // ATK-SC-7: serde's derived Deserialize does NOT call Predicate::validate(),
        // so a hand-crafted JSON like `{"kind":"all_of","children":[]}` bypasses the
        // ZeroLeafComposition guard and evaluates vacuously to passed=true with no
        // leaves. Validate after deserialization and emit a diagnostic rather than
        // silently proceeding to a vacuous evaluation.
        if predicate.validate().is_err() {
            audits.push(SupplyChainAudit {
                antigen_type: immunity.antigen_type.clone(),
                hint: AuditHint::MalformedRequiresPredicate,
                file: immunity.file.clone(),
                line: immunity.line,
                crate_name: String::new(),
                version: String::new(),
                detail: Some(
                    "requires_predicate deserialized but failed structural validation \
                     (e.g., empty combinator — vacuous pass with no leaves)"
                        .to_string(),
                ),
            });
            continue;
        }

        // Evaluate the predicate tree as a whole so combinator semantics
        // hold: a Mismatch inside `any_of([match_X, no_attest_Y])` does
        // NOT surface if `match_X` passes (ATK-SC-AUDIT-1). The
        // evaluator returns the per-leaf audit entries that should be
        // surfaced after combinator-aware pruning.
        let entries = eval_supply_chain_predicate(
            &predicate,
            workspace_root,
            &immunity.antigen_type,
            &immunity.file,
            immunity.line,
        );
        audits.extend(entries.entries);
    }

    let mut pass_count = 0usize;
    let mut fail_count = 0usize;
    for a in &audits {
        if is_supply_chain_pass_hint(&a.hint) {
            pass_count += 1;
        } else {
            fail_count += 1;
        }
    }

    SupplyChainAuditReport {
        audits,
        pass_count,
        fail_count,
    }
}

/// Result of evaluating a sub-predicate over supply-chain leaves.
///
/// The combinator-aware evaluator returns both the boolean predicate
/// outcome AND the per-leaf audit entries that should be surfaced.
/// Per ATK-SC-AUDIT-1: leaf-fail entries inside a satisfied `any_of`
/// MUST NOT surface — the sibling pass discharges them.
struct SupplyChainEval {
    /// Whether the sub-predicate evaluated to logical-true.
    passed: bool,
    /// Per-leaf audit entries to surface for this sub-tree, AFTER
    /// combinator-aware pruning. Per ATK-SC-AUDIT-1 / the `any_of`-
    /// discharge rule: only entries that contribute to the
    /// load-bearing answer should appear here.
    entries: Vec<SupplyChainAudit>,
}

/// Combinator-aware evaluator over supply-chain leaves.
///
/// Per ATK-SC-AUDIT-1: a satisfied `any_of` discharges its losing
/// children's audit hints. The naive "collect every leaf and emit
/// every fail" approach surfaces false positives when a sibling
/// branch passes.
fn eval_supply_chain_predicate(
    pred: &antigen_attestation::Predicate,
    workspace_root: &Path,
    antigen_type: &str,
    file: &Path,
    line: usize,
) -> SupplyChainEval {
    use antigen_attestation::Predicate;
    match pred {
        Predicate::Leaf(l) => {
            // Per-leaf evaluation. Pass entries through unconditionally
            // at the leaf level; combinator parents prune.
            let entry = audit_supply_chain_leaf(l, workspace_root, antigen_type, file, line);
            let passed = entry
                .as_ref()
                .is_some_and(|e| is_supply_chain_pass_hint(&e.hint));
            // Non-supply-chain leaves return None — treat them as
            // logically-true for the supply-chain sub-evaluation (the
            // standard substrate-witness audit handles them). This
            // means a `requires = all_of([content_hash_matches(...),
            // ratified_doc(...)])` still surfaces the supply-chain
            // half's verdict correctly.
            let logical_passed = entry.as_ref().is_none_or(|_| passed);
            SupplyChainEval {
                passed: logical_passed,
                entries: entry.into_iter().collect(),
            }
        }
        Predicate::AllOf { children } => {
            let mut entries = Vec::new();
            let mut all_pass = true;
            for c in children {
                let sub = eval_supply_chain_predicate(c, workspace_root, antigen_type, file, line);
                if !sub.passed {
                    all_pass = false;
                }
                entries.extend(sub.entries);
            }
            SupplyChainEval {
                passed: all_pass,
                entries,
            }
        }
        Predicate::AnyOf { children } => {
            // Per ATK-SC-AUDIT-1: evaluate each child; if ANY passes,
            // discharge the others' fail entries (keep only the
            // passing entries for documentation). If none pass,
            // surface every child's failure entries.
            let mut pass_entries = Vec::new();
            let mut fail_entries = Vec::new();
            let mut any_pass = false;
            for c in children {
                let sub = eval_supply_chain_predicate(c, workspace_root, antigen_type, file, line);
                if sub.passed {
                    any_pass = true;
                    pass_entries.extend(sub.entries);
                } else {
                    fail_entries.extend(sub.entries);
                }
            }
            let entries = if any_pass { pass_entries } else { fail_entries };
            SupplyChainEval {
                passed: any_pass,
                entries,
            }
        }
        Predicate::Not { child } => {
            // `not(P)` — the supply-chain audit cannot meaningfully
            // surface "the failed leaf" because the failure is the
            // intended outcome. We invert `passed` and DROP the inner
            // entries (they describe what we WANTED to fail; surfacing
            // them as hints would be misleading). v0.2 supports `not`
            // structurally but emits no documentary entries from the
            // negated sub-tree.
            let sub = eval_supply_chain_predicate(child, workspace_root, antigen_type, file, line);
            SupplyChainEval {
                passed: !sub.passed,
                entries: Vec::new(),
            }
        }
    }
}

/// Evaluate a single supply-chain leaf and produce a [`SupplyChainAudit`]
/// entry. Returns `None` for non-supply-chain leaves.
fn audit_supply_chain_leaf(
    leaf: &antigen_attestation::Leaf,
    workspace_root: &Path,
    antigen_type: &str,
    file: &Path,
    line: usize,
) -> Option<SupplyChainAudit> {
    use antigen_attestation::Leaf;

    let (crate_name, version, hint, detail) = match leaf {
        Leaf::DepPinned { crate_name } => {
            eval_dep_pinned_to_hint(workspace_root, crate_name.as_deref())
        }
        Leaf::DepAttested {
            crate_name,
            version,
            exact_version,
            ..
        } => eval_dep_attested_to_hint(workspace_root, crate_name, version, *exact_version),
        Leaf::MaintainerUnchanged {
            crate_name,
            since_version,
        } => eval_maintainer_unchanged_to_hint(workspace_root, crate_name, since_version),
        Leaf::ContentHashMatches {
            crate_name,
            version,
        } => eval_content_hash_matches_to_hint(workspace_root, crate_name, version),
        Leaf::SandboxClean {
            crate_name,
            sandbox_kind,
        } => eval_sandbox_clean_to_hint(crate_name, sandbox_kind),
        // Non-supply-chain leaves: not our pipeline's responsibility.
        Leaf::RatifiedDoc { .. }
        | Leaf::Signers { .. }
        | Leaf::SignedTrailer { .. }
        | Leaf::OraclesComplete { .. }
        | Leaf::FreshWithinDays { .. } => return None,
    };

    Some(SupplyChainAudit {
        antigen_type: antigen_type.to_string(),
        file: file.to_path_buf(),
        line,
        crate_name,
        version,
        hint,
        detail,
    })
}

/// Return `(crate, version, hint, detail)` for `dep_pinned`.
fn eval_dep_pinned_to_hint(
    workspace_root: &Path,
    crate_name: Option<&str>,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::DepPinnedState};
    let state = evaluate::evaluate_dep_pinned(workspace_root, crate_name);
    let (hint, detail) = match &state {
        DepPinnedState::AllPinned => (AuditHint::FunctionResolves, None),
        DepPinnedState::Unpinned { unpinned_deps } => (
            AuditHint::UnpinnedDependency,
            Some(format!("unpinned: {unpinned_deps:?}")),
        ),
        DepPinnedState::NotInManifest { crate_name: cn } => (
            AuditHint::UnpinnedDependency,
            Some(format!("crate not in manifest: {cn}")),
        ),
    };
    (
        crate_name.map_or_else(|| "*".to_string(), str::to_string),
        String::new(),
        hint,
        detail,
    )
}

/// Return `(crate, version, hint, detail)` for `dep_attested`.
fn eval_dep_attested_to_hint(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
    exact_version: bool,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::DepAttestedState};
    let state = evaluate::evaluate_dep_attested(workspace_root, crate_name, version, exact_version);
    let (hint, detail) = match &state {
        DepAttestedState::Attested { .. } => (AuditHint::FunctionResolves, None),
        DepAttestedState::AttestedWithoutReviewableArtifact => {
            (AuditHint::DepAttestWithoutReviewableArtifact, None)
        }
        DepAttestedState::SidecarMissing => (AuditHint::UnattestedDependencyInclusion, None),
        DepAttestedState::SidecarMalformed { error } => (
            AuditHint::UnattestedDependencyInclusion,
            Some(format!("sidecar malformed: {error}")),
        ),
        DepAttestedState::AttestationStale {
            attested_version,
            requested_version,
        } => (
            AuditHint::DepAttestationStale,
            Some(format!(
                "attested: {attested_version}; requested: {requested_version}"
            )),
        ),
    };
    (crate_name.to_string(), version.to_string(), hint, detail)
}

/// Return `(crate, since_version, hint, detail)` for `maintainer_unchanged`.
fn eval_maintainer_unchanged_to_hint(
    workspace_root: &Path,
    crate_name: &str,
    since_version: &str,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::MaintainerState};
    let state = evaluate::evaluate_maintainer_unchanged(workspace_root, crate_name, since_version);
    let (hint, detail) = match &state {
        MaintainerState::Unchanged => (AuditHint::FunctionResolves, None),
        MaintainerState::Changed { added, removed } => (
            AuditHint::MaintainerChangeWithoutReattestation,
            Some(format!("added: {added:?}; removed: {removed:?}")),
        ),
        MaintainerState::SnapshotMissing => (
            AuditHint::MaintainerChangeWithoutReattestation,
            Some("snapshot missing".to_string()),
        ),
        MaintainerState::CratesIoQueryUnavailable => (AuditHint::CratesIoMetadataQueryFailed, None),
    };
    (
        crate_name.to_string(),
        since_version.to_string(),
        hint,
        detail,
    )
}

/// Return `(crate, version, hint, detail)` for `content_hash_matches`.
fn eval_content_hash_matches_to_hint(
    workspace_root: &Path,
    crate_name: &str,
    version: &str,
) -> (String, String, AuditHint, Option<String>) {
    use crate::supply_chain::{evaluate, witness::ContentHashState};
    let state = evaluate::evaluate_content_hash_matches(workspace_root, crate_name, version);
    let (hint, detail) = match &state {
        ContentHashState::Matches => (AuditHint::FunctionResolves, None),
        ContentHashState::Mismatch { recorded, current } => (
            AuditHint::ContentHashMismatch,
            Some(format!("recorded: {recorded}; current: {current}")),
        ),
        ContentHashState::NoAttestation => (AuditHint::ContentHashNoAttestation, None),
        ContentHashState::CrateNotInLockfile { crate_name: cn } => (
            AuditHint::ContentHashNoAttestation,
            Some(format!("crate not in Cargo.lock: {cn}")),
        ),
        ContentHashState::SidecarMalformed { error } => (
            AuditHint::ContentHashSidecarMalformed,
            Some(format!("malformed: {error}")),
        ),
    };
    (crate_name.to_string(), version.to_string(), hint, detail)
}

/// Return `(crate, version, hint, detail)` for `sandbox_clean`. v0.2:
/// tooling not available — emit the awareness hint.
fn eval_sandbox_clean_to_hint(
    crate_name: &str,
    sandbox_kind: &str,
) -> (String, String, AuditHint, Option<String>) {
    let hint = if sandbox_kind == "proc-macro" {
        AuditHint::UnsandboxedProcMacro
    } else {
        AuditHint::UnsandboxedBuildScript
    };
    (
        crate_name.to_string(),
        String::new(),
        hint,
        Some(format!(
            "v0.2 sandbox tooling not yet available; kind={sandbox_kind}"
        )),
    )
}

/// Distinguish supply-chain pass hints from fail hints. `FunctionResolves`
/// is the borrowed-from-standard-audit "predicate passed" hint that the
/// supply-chain audit emits for clean evaluations.
const fn is_supply_chain_pass_hint(hint: &AuditHint) -> bool {
    matches!(hint, AuditHint::FunctionResolves)
}

// ============================================================================
// Convergent-Evidence Family audit (ADR-024)
// ============================================================================

/// One result of auditing a convergent-evidence declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergentEvidenceAudit {
    /// The original declaration from the scan.
    pub declaration: crate::scan::ConvergentEvidence,
    /// The hint(s) the audit emitted for this declaration. A single
    /// declaration may surface multiple hints (e.g., `#[diagnostic]`
    /// can be both class-collapsed AND modality-insufficient).
    pub hints: Vec<AuditHint>,
}

/// Aggregate convergent-evidence audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvergentEvidenceAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<ConvergentEvidenceAudit>,
    /// Count of declarations whose hint set is empty (clean).
    pub clean_count: usize,
    /// Count of declarations whose hint set is non-empty (concerns
    /// surfaced).
    pub concern_count: usize,
}

impl ConvergentEvidenceAuditReport {
    /// True when no concerns were surfaced.
    #[must_use]
    pub const fn all_clean(&self) -> bool {
        self.concern_count == 0
    }
}

/// Audit convergent-evidence declarations across a scan report.
///
/// Walks every [`crate::scan::ConvergentEvidence`] in the report and
/// produces [`ConvergentEvidenceAudit`] entries surfacing the relevant
/// audit hints per ADR-024 §Audit-hint-vocabulary.
#[must_use]
pub fn audit_convergent_evidence(report: &ScanReport) -> ConvergentEvidenceAuditReport {
    let known_antigen_names: std::collections::HashSet<&str> = report
        .antigens
        .iter()
        .map(|a| a.type_name.as_str())
        .collect();

    let mut audits: Vec<ConvergentEvidenceAudit> = Vec::new();

    for decl in &report.convergent_evidences {
        let hints = evaluate_convergent_evidence_hints(decl, &known_antigen_names);
        audits.push(ConvergentEvidenceAudit {
            declaration: decl.clone(),
            hints,
        });
    }

    let mut clean_count = 0usize;
    let mut concern_count = 0usize;
    for a in &audits {
        if a.hints.is_empty() {
            clean_count += 1;
        } else {
            concern_count += 1;
        }
    }

    ConvergentEvidenceAuditReport {
        audits,
        clean_count,
        concern_count,
    }
}

fn evaluate_convergent_evidence_hints(
    decl: &crate::scan::ConvergentEvidence,
    known_antigen_names: &std::collections::HashSet<&str>,
) -> Vec<AuditHint> {
    use crate::scan::ConvergentEvidenceKind;

    let mut hints = Vec::new();
    match decl.kind {
        ConvergentEvidenceKind::Diagnostic => {
            if decl.modality_classes.is_empty() {
                hints.push(AuditHint::DiagnosticModalitiesEmpty);
                return hints;
            }
            let distinct: std::collections::HashSet<&str> =
                decl.modality_classes.iter().map(String::as_str).collect();
            // Class-collapse: many entries, all same class (per C1)
            if distinct.len() == 1 && decl.modality_classes.len() > 1 {
                hints.push(AuditHint::DiagnosticModalitiesClassCollapsed);
            }
            if let Some(min) = decl.min_independent {
                if min == 0 {
                    // A zero threshold is semantically null: it can never fire
                    // DiagnosticModalityInsufficient regardless of how many (or
                    // few) independent classes exist. Surface the misconfiguration
                    // explicitly rather than silently accepting it. (ATK-CE-5)
                    hints.push(AuditHint::DiagnosticMinIndependentZero);
                } else if u64::try_from(distinct.len()).unwrap_or(u64::MAX) < min {
                    hints.push(AuditHint::DiagnosticModalityInsufficient);
                }
            }
        }
        ConvergentEvidenceKind::Clonal => {
            // Fixed-seed in scan output: the proc-macro rejects this at
            // parse time, but the scan walks raw source — pre-cap source
            // can surface here. Surface the hint explicitly.
            if matches!(decl.seed_kind.as_deref(), Some("Fixed")) {
                hints.push(AuditHint::ClonalFixedSeedDetected);
            }
            if let Some(iters) = decl.iterations {
                if iters < CLONAL_ITERATIONS_DEFAULT_FLOOR {
                    hints.push(AuditHint::ClonalIterationsBelowThreshold);
                }
            }
        }
        ConvergentEvidenceKind::Igg => {
            if let Some(span) = decl.historical_span {
                if span < IGG_HISTORICAL_SPAN_DEFAULT_FLOOR {
                    hints.push(AuditHint::IggSpanTooShort);
                }
            }
            // Per ATK-CE-3-B: count UNIQUE witnesses, not raw count.
            // The same identity signing twice doesn't add reattestation
            // independence — the discipline is about independent re-
            // verification, not raw signature count. Raw-count check
            // (`witnesses.len() >= min_re`) is misleading because
            // duplicate identities inflate the apparent count.
            let unique_count: std::collections::HashSet<&str> =
                decl.witnesses.iter().map(String::as_str).collect();
            if let Some(min_re) = decl.min_reattestations {
                if min_re == 0 {
                    // Zero reattestations required is a null threshold — it can
                    // never fire IggReattestationsInsufficient. Surface the
                    // misconfiguration explicitly. (ATK-CE-5)
                    hints.push(AuditHint::IggMinReattestationsZero);
                } else if u64::try_from(unique_count.len()).unwrap_or(u64::MAX) < min_re {
                    hints.push(AuditHint::IggReattestationsInsufficient);
                }
            }
            // Identity-collapse: best-effort at scan time — if the
            // recorded witnesses all collapse to one identity, surface
            // the warning. Real signer-identity tracking is v0.3+.
            if decl.witnesses.len() > 1 && unique_count.len() == 1 {
                hints.push(AuditHint::IggIdentityCollapseWarning);
            }
        }
        ConvergentEvidenceKind::Crossreactive => {
            for fp in &decl.fingerprints {
                if !known_antigen_names.contains(fp.as_str()) {
                    hints.push(AuditHint::CrossreactiveFingerprintUnresolved);
                    break;
                }
            }
        }
        ConvergentEvidenceKind::Polyclonal
        | ConvergentEvidenceKind::Monoclonal
        | ConvergentEvidenceKind::Adcc => {
            // v0.2: marker primitives. Lineage-count enforcement
            // (polyclonal) and mechanism-pairing detection (adcc)
            // require co-located witness sites; deferred to v0.3+ when
            // the scan layer cross-links convergent declarations with
            // their on-item witness companions. monoclonal is
            // documentary by definition. v0.2 emits no automatic
            // concerns for any of the three.
        }
    }
    hints
}

// ============================================================================
// Recurrent-Emergence audit (ADR-024 §Family 2)
// ============================================================================

/// Per-declaration recurrent-emergence audit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrentAudit {
    /// The original declaration from the scan.
    pub declaration: crate::scan::RecurrentDeclaration,
    /// Hints surfaced for this declaration (may be empty = clean).
    pub hints: Vec<AuditHint>,
}

/// Aggregate recurrent-emergence audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecurrentAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<RecurrentAudit>,
    /// Count of declarations whose hint set is empty (clean).
    pub clean_count: usize,
    /// Count of declarations whose hint set is non-empty (concerns surfaced).
    pub concern_count: usize,
}

impl RecurrentAuditReport {
    /// True when no concerns were surfaced.
    #[must_use]
    pub const fn all_clean(&self) -> bool {
        self.concern_count == 0
    }
}

/// Default review horizon (days) past which a `#[chronic]` `since` date
/// surfaces `chronic-signal-past-review-date`. One year — chronic states
/// older than this warrant explicit re-review per ADR-024 recurrent
/// discipline. Configurable via workspace config in v0.3+.
const CHRONIC_REVIEW_HORIZON_DAYS: i64 = 365;

/// Audit recurrent-emergence declarations across a scan report (ADR-024
/// §Family 2). Walks every [`crate::scan::RecurrentDeclaration`] and
/// surfaces the relevant hints per the recurrent audit taxonomy.
#[must_use]
pub fn audit_recurrent(report: &ScanReport) -> RecurrentAuditReport {
    // Antigen-type names that have downstream action (an #[immune] or
    // #[presents] referencing them). Used for the recurrence-anchor
    // threshold-reached-no-action check.
    let acted_on: std::collections::HashSet<&str> = report
        .immunities
        .iter()
        .map(|i| i.antigen_type.as_str())
        .chain(report.presentations.iter().map(|p| p.antigen_type.as_str()))
        .collect();

    // Antigen-type names declared by #[itch] entries. Used to check that
    // #[recurrence_anchor] has upstream noticing preconditions (ATK-RECURRENT-2).
    let itch_antigen_types: std::collections::HashSet<&str> = report
        .recurrent_declarations
        .iter()
        .filter(|d| d.kind == crate::scan::RecurrentKind::Itch)
        .filter_map(|d| d.antigen_type.as_deref())
        .collect();

    // Direct child→parent edges (bare names), the substrate for the
    // lineage-aware from_itches check (ADR-024 Amendment 3). The
    // noticing-precondition is class-specific BUT lineage-aware: noticing an
    // ANCESTOR class is legitimate upstream evidence for committing to track a
    // descendant (inheritance is provenance — ADR-018 Amd1; parent-recurrence
    // is evidence the lineage recurs). Built once here; `ancestors_of` walks it
    // transitively per anchor.
    let parent_of: std::collections::HashMap<&str, Vec<&str>> = {
        let mut m: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        for e in &report.lineage_edges {
            m.entry(e.child.as_str())
                .or_default()
                .push(e.parent.as_str());
        }
        m
    };

    let mut audits: Vec<RecurrentAudit> = Vec::new();
    for decl in &report.recurrent_declarations {
        let hints = evaluate_recurrent_hints(decl, &acted_on, &itch_antigen_types, &parent_of);
        audits.push(RecurrentAudit {
            declaration: decl.clone(),
            hints,
        });
    }

    let mut clean_count = 0usize;
    let mut concern_count = 0usize;
    for a in &audits {
        if a.hints.is_empty() {
            clean_count += 1;
        } else {
            concern_count += 1;
        }
    }

    RecurrentAuditReport {
        audits,
        clean_count,
        concern_count,
    }
}

/// Heuristic: is `s` a recognizable version tag (vs. a calendar date or
/// garbage)? Per ATK-RECURRENT-4a the chronic `since` field tolerates
/// version anchors but flags unparseable strings. A version tag is an
/// optional leading `v`/`V` followed by at least one dot-separated numeric
/// component (e.g. `v0.2.0`, `1.4`, `2.0.0-rc.1`). Pre-release/build
/// suffixes after the numeric core are allowed.
fn is_version_tag(s: &str) -> bool {
    let had_v_prefix = s.starts_with(['v', 'V']);
    let core = if had_v_prefix { &s[1..] } else { s };
    // The numeric core runs until the first `-`/`+` (where a pre-release or
    // build suffix like `-rc.1` or `+build` begins).
    let numeric_core: &str = core.split(['-', '+']).next().unwrap_or("");
    if numeric_core.is_empty() {
        return false;
    }
    // Every dot-separated component of the numeric core must be all digits.
    let mut component_count = 0usize;
    for part in numeric_core.split('.') {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        component_count += 1;
    }
    // A version tag is either `v`-prefixed (e.g. `v3`, `v0.2.0`) OR has at
    // least major.minor structure (≥2 dot-separated numeric components).
    // A bare single integer like `"3"` is ambiguous garbage, not a version.
    had_v_prefix || component_count >= 2
}

/// Transitive ancestor set of `antigen` over the `child → parent` lineage map
/// (ADR-024 Amendment 3 lineage-aware `from_itches`). Walks every
/// `#[descended_from]` chain upward, cycle-guarded (a malformed cyclic lineage
/// must not loop here — cycle detection is the scanner's job, but this walk is
/// defensively bounded by the `visited` set). The anchor's OWN type is NOT
/// included (the caller checks self-match separately); only strict ancestors.
fn ancestors_of<'a>(
    antigen: &'a str,
    parent_of: &std::collections::HashMap<&'a str, Vec<&'a str>>,
) -> std::collections::HashSet<&'a str> {
    let mut acc: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut stack: Vec<&str> = parent_of.get(antigen).cloned().unwrap_or_default();
    while let Some(parent) = stack.pop() {
        if acc.insert(parent) {
            if let Some(grandparents) = parent_of.get(parent) {
                stack.extend(grandparents.iter().copied());
            }
        }
    }
    acc
}

/// Evaluate hints for a single recurrent declaration.
///
/// **ATK-RECURRENT-2 fix (dd51d4b)**: this function now checks BOTH the
/// upstream precondition (`itch_antigen_types` contains anchor's antigen type)
/// AND the downstream action (`acted_on` contains the antigen type). See
/// [`crate::stdlib::dogfood::AuditHintWithNoUpstreamPreconditionCheck`].
///
/// **ADR-024 Amendment 3 (class-specific, lineage-aware `from_itches`)**: a
/// `from_itches` entry satisfies the noticing-precondition iff it names the
/// anchor's OWN `antigen_type` (or a lineage ANCESTOR of it) AND that class has
/// a scan-resident `#[itch]`. A pure cross-class reference (an unrelated class,
/// even one with its own itch) carries ZERO precondition-evidence for this
/// anchor — noticing `AntigenY` tells you nothing about whether `AntigenX`
/// recurred. The prior global membership test silently widened the precondition
/// to "does the workspace contain any itch at all", the vacuous-guard shape;
/// this realigns the impl with the audit-hint doc's already-stated intent
/// ("the same antigen type" — a `RatifiedSpecDriftFromImpl` fix, not a new
/// design choice). The lineage exception is the one legitimate
/// "cross-class" case and is intra-lineage, not cross-class: inheritance is
/// provenance (ADR-018 Amd1), so parent-recurrence is evidence the descended
/// lineage recurs.
// ADR-029 migration: this fn `#[presents]` AuditHintWithNoUpstreamPreconditionCheck
// (it once emitted the hint without checking the upstream precondition). The
// integration test `atk_recurrent_2_recurrence_anchor_without_matching_itch_emits_hint`
// (tests/atk_recurrent_adversarial.rs) declares it defends the class via
// `#[defended_by]`; the audit cross-references and observes the verdict.
#[presents(AuditHintWithNoUpstreamPreconditionCheck)]
fn evaluate_recurrent_hints(
    decl: &crate::scan::RecurrentDeclaration,
    acted_on: &std::collections::HashSet<&str>,
    itch_antigen_types: &std::collections::HashSet<&str>,
    parent_of: &std::collections::HashMap<&str, Vec<&str>>,
) -> Vec<AuditHint> {
    use crate::scan::RecurrentKind;

    let mut hints = Vec::new();
    match decl.kind {
        RecurrentKind::Itch => {
            if decl.antigen_type.is_none() {
                hints.push(AuditHint::ItchNoticedNotAnchored);
            }
        }
        RecurrentKind::RecurrenceAnchor => {
            // Anchor has no upstream itch preconditions — temporal progression
            // (itch → anchor → crystallize) bypassed (ATK-RECURRENT-2).
            //
            // Two bypass vectors (ATK-RECURRENT-7 adds the second):
            //   (a) from_itches is empty AND the anchor's antigen type has no
            //       corresponding #[itch] in the scan — temporal progression skipped.
            //   (b) from_itches is non-empty but ALL listed itches are phantom
            //       references — they name itch types that have no #[itch] declaration
            //       in the scan. A non-empty phantom list bypassed the is_empty() guard
            //       while providing zero real precondition evidence. We now validate
            //       that from_itches entries actually resolve to scan-resident itches.
            if let Some(antigen) = decl.antigen_type.as_deref() {
                // ADR-024 Amendment 3: a from_itches entry is valid ONLY when it
                // names the anchor's own class OR a lineage ancestor of it, AND
                // that class has a scan-resident #[itch]. A pure cross-class
                // reference is a phantom for THIS anchor (no precondition
                // evidence), exactly as ATK-RECURRENT-7 treats non-scan-resident
                // phantoms — the class-scoped test below subsumes the old global
                // membership test (an entry must still be itch-resident, but now
                // it must additionally be in-lineage).
                let ancestors = ancestors_of(antigen, parent_of);
                let in_lineage = |itch: &str| itch == antigen || ancestors.contains(itch);
                let has_valid_from_itches = !decl.from_itches.is_empty()
                    && decl.from_itches.iter().any(|itch| {
                        in_lineage(itch.as_str()) && itch_antigen_types.contains(itch.as_str())
                    });
                let has_implicit_itch = itch_antigen_types.contains(antigen);

                if !has_valid_from_itches && !has_implicit_itch {
                    // No real itch precondition: either no from_itches + no implicit
                    // itch, OR from_itches is entirely phantom references that don't
                    // resolve to any scan-resident #[itch] declaration. (ATK-RECURRENT-7)
                    hints.push(AuditHint::RecurrenceAnchorNoItchPrecondition);
                }
                // Anchor crossed threshold but nothing downstream addresses it.
                if !acted_on.contains(antigen) {
                    hints.push(AuditHint::RecurrenceThresholdReachedNoAction);
                }
            }
        }
        RecurrentKind::Crystallize => {
            // A crystallization with neither a formal antigen NOR source
            // itches crystallized nothing into anything.
            if decl.antigen_type.is_none() && decl.from_itches.is_empty() {
                hints.push(AuditHint::CrystallizeWithoutAntigen);
            }
        }
        RecurrentKind::Chronic => {
            if decl.managed_by.is_none() {
                hints.push(AuditHint::ChronicSignalUnmanaged);
            }
            // Three-path `since` resolution per ATK-RECURRENT-4a:
            //   (1) ISO-8601 date → enforce the review horizon
            //       (past-horizon → past-review-date hint).
            //   (2) version tag (e.g. "v0.2.0", "1.4.3-rc.1") → tolerate;
            //       the chronic state is anchored to a release, not a
            //       calendar, so no date check applies.
            //   (3) neither → `since` is a malformed anchor; emit
            //       chronic-since-not-a-date.
            if let Some(since) = decl.since.as_deref() {
                if let Ok(since_date) = chrono::NaiveDate::parse_from_str(since, "%Y-%m-%d") {
                    let age = chrono::Utc::now().date_naive() - since_date;
                    if age.num_days() > CHRONIC_REVIEW_HORIZON_DAYS {
                        hints.push(AuditHint::ChronicSignalPastReviewDate);
                    }
                } else if !is_version_tag(since) {
                    hints.push(AuditHint::ChronicSinceNotADate);
                }
            }
        }
        RecurrentKind::Saturate => {
            if decl.contributing_to.is_none() {
                hints.push(AuditHint::SaturateNoAnchor);
            }
        }
        RecurrentKind::Strand => {
            if decl.anchored_by.is_empty() {
                hints.push(AuditHint::StrandNoAnchors);
            }
        }
    }
    hints
}

// ============================================================================
// Mucosal Boundary audit (ADR-027 + Amendment 1)
// ============================================================================

/// Per-declaration mucosal-boundary audit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MucosalAudit {
    /// The original declaration from the scan.
    pub declaration: crate::scan::MucosalDeclaration,
    /// Hints surfaced for this declaration (may be empty = clean).
    pub hints: Vec<AuditHint>,
}

/// Aggregate mucosal-boundary audit report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MucosalAuditReport {
    /// Per-declaration audit results.
    pub audits: Vec<MucosalAudit>,
    /// Count of declarations whose hint set is empty (clean).
    pub clean_count: usize,
    /// Count of declarations whose hint set is non-empty.
    pub concern_count: usize,
}

impl MucosalAuditReport {
    /// True when no concerns were surfaced.
    #[must_use]
    pub const fn all_clean(&self) -> bool {
        self.concern_count == 0
    }
}

/// Minimum rationale lengths per ADR-027 + Amendment 1 Change 6 (risk-
/// proportionate: tolerance is riskier than defense, so its floor is higher).
const MUCOSAL_RATIONALE_FLOOR: usize = 20;
const MUCOSAL_TOLERANT_RATIONALE_FLOOR: usize = 40;

/// Audit mucosal-boundary declarations across a scan report (ADR-027 +
/// Amendment 1).
///
/// Implements the Change-5 three-tier delegate diagnosis via set-membership
/// kind-matching against the handler functions' `#[mucosal]` declarations.
///
/// **Residual risk**: `handler_kinds` is built from intra-crate `#[mucosal]`
/// declarations only. Cross-crate handlers are not in the index; delegates
/// pointing at them will false-positive as `MucosalDisciplineDelegateTargetMissing`.
/// See [`crate::stdlib::agentic_coordination::DelegateCrossCrateResolutionGap`].
/// Structural fix is v0.3+ scope (multi-crate scan pass).
#[must_use]
#[presents(DelegateCrossCrateResolutionGap)]
#[antigen_tolerance(
    DelegateCrossCrateResolutionGap,
    rationale = "Accepted v0.2 limitation: handler_kinds is built from intra-crate #[mucosal] \
                 declarations only, so a #[mucosal_delegate] pointing at a cross-crate handler \
                 false-positives as MucosalDisciplineDelegateTargetMissing. The structural fix is a \
                 multi-crate scan pass (v0.3+ scope, same boundary as --include-deps cross-crate \
                 addressing). Until then the false-positive is the conservative failure (flags rather \
                 than silently trusts an unresolvable delegate).",
    until = "v0.3"
)]
pub fn audit_mucosal(report: &ScanReport) -> MucosalAuditReport {
    use crate::scan::{ItemTarget, MucosalKindTag};

    // Build handler-function → set-of-mucosal-kinds index from every
    // `#[mucosal]` declaration sitting on a function. The delegate
    // kind-matching (Change 5c) is set-membership against this index;
    // hybrid handlers (multiple `#[mucosal(kind = X)]`) contribute multiple
    // kinds to their function's set.
    let mut handler_kinds: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
        std::collections::HashMap::new();
    // Track which source files each bare function name appears in, so we can detect
    // same-name ambiguity (findings/mucosal-same-name-fn-collision). A bare fn name
    // that appears in more than one file is ambiguous: delegates pointing to it by
    // bare name alone cannot unambiguously identify the target.
    let mut handler_files: std::collections::HashMap<
        &str,
        std::collections::HashSet<&std::path::Path>,
    > = std::collections::HashMap::new();
    for decl in &report.mucosal_declarations {
        if decl.tag == MucosalKindTag::Mucosal {
            if let ItemTarget::Fn(fn_name) = &decl.item_target {
                if let Some(kind) = decl.boundary_kind.as_deref() {
                    handler_kinds
                        .entry(fn_name.as_str())
                        .or_default()
                        .insert(kind);
                }
                handler_files
                    .entry(fn_name.as_str())
                    .or_default()
                    .insert(decl.file.as_path());
            }
        }
    }
    // A name is ambiguous when it maps to 2+ distinct source files.
    let ambiguous_names: std::collections::HashSet<&str> = handler_files
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(name, _)| *name)
        .collect();

    let mut audits: Vec<MucosalAudit> = Vec::new();
    for decl in &report.mucosal_declarations {
        let hints = evaluate_mucosal_hints(decl, &handler_kinds, &ambiguous_names);
        audits.push(MucosalAudit {
            declaration: decl.clone(),
            hints,
        });
    }

    let mut clean_count = 0usize;
    let mut concern_count = 0usize;
    for a in &audits {
        if a.hints.is_empty() {
            clean_count += 1;
        } else {
            concern_count += 1;
        }
    }

    MucosalAuditReport {
        audits,
        clean_count,
        concern_count,
    }
}

fn evaluate_mucosal_hints(
    decl: &crate::scan::MucosalDeclaration,
    handler_kinds: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    ambiguous_names: &std::collections::HashSet<&str>,
) -> Vec<AuditHint> {
    use crate::scan::MucosalKindTag;

    let mut hints = Vec::new();
    match decl.tag {
        MucosalKindTag::Mucosal => {
            if decl.boundary_kind.is_none() {
                hints.push(AuditHint::MucosalKindMismatch);
            }
            if decl
                .rationale
                .as_deref()
                .is_none_or(|r| r.len() < MUCOSAL_RATIONALE_FLOOR)
            {
                hints.push(AuditHint::MucosalRationaleInsufficient);
            }
        }
        MucosalKindTag::MucosalDelegate => {
            if decl.boundary_kind.is_none() {
                hints.push(AuditHint::MucosalKindMismatch);
            }
            // Change 5 three-tier diagnosis on the delegate handler.
            match decl.handled_by.as_deref() {
                None => hints.push(AuditHint::MucosalDisciplineDelegateTargetMissing),
                Some(handler) => {
                    // Ambiguity check (findings/mucosal-same-name-fn-collision):
                    // If the bare handler name matches multiple source files, the
                    // delegate is ambiguous and must be flagged BEFORE attempting
                    // kind-resolution — the merged kind-set union would silently
                    // grant the wrong file's kinds. Only emitted for resolved-but-
                    // ambiguous names (handler in ambiguous_names), NOT for missing
                    // targets (Tier 1 below catches those).
                    if ambiguous_names.contains(handler) {
                        hints.push(AuditHint::MucosalDisciplineDelegateTargetAmbiguous);
                    } else {
                        match handler_kinds.get(handler) {
                            // Tier 1: handler doesn't resolve to any #[mucosal]-fn.
                            None => hints.push(AuditHint::MucosalDisciplineDelegateTargetMissing),
                            Some(kinds) if kinds.is_empty() => {
                                // Tier 2: resolves but carries no mucosal kind.
                                hints.push(AuditHint::MucosalDisciplineDelegateTargetNotMucosal);
                            }
                            Some(kinds) => {
                                // Tier 3: set-membership kind match (NOT exact-equality).
                                let matches = decl
                                    .boundary_kind
                                    .as_deref()
                                    .is_some_and(|b| kinds.contains(b));
                                if !matches {
                                    hints.push(
                                        AuditHint::MucosalDisciplineDelegateTargetKindMismatch,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        MucosalKindTag::MucosalTolerant => {
            if decl.boundary_kind.is_none() {
                hints.push(AuditHint::MucosalKindMismatch);
            }
            if decl
                .rationale
                .as_deref()
                .is_none_or(|r| r.len() < MUCOSAL_TOLERANT_RATIONALE_FLOOR)
            {
                hints.push(AuditHint::MucosalTolerantRationaleInsufficient);
            }
            if decl.accepts.as_deref().is_none_or(|a| a.trim().is_empty()) {
                hints.push(AuditHint::MucosalTolerantAcceptsEmpty);
            }
            if decl.reviewed_by.is_none() {
                hints.push(AuditHint::MucosalTolerantWithoutReviewer);
            }
            // Past-review-date: only when `until` parses as an ISO date.
            if let Some(until) = decl.until.as_deref() {
                if let Ok(until_date) = chrono::NaiveDate::parse_from_str(until, "%Y-%m-%d") {
                    if chrono::Utc::now().date_naive() > until_date {
                        hints.push(AuditHint::MucosalTolerantPastReviewDate);
                    }
                }
            }
        }
    }
    hints
}

mod category;
pub use category::{audit_category, CategoryAudit, CategoryAuditReport};
mod lineage_fidelity;
pub use lineage_fidelity::{
    audit_lineage_fidelity, LineageFidelityAudit, LineageFidelityAuditReport,
};

mod coverage;
pub use coverage::{audit_coverage, CoverageAuditReport, UnreachedCause, UnreachedSite};
// ============================================================================
// Prescriptive Work-Orchestration audit (ADR-033) — "code IS the board"
// ============================================================================

/// The satisfaction of a single who-step (one `filled_by` / `reviewed_by` /
/// `ordered_by` / `triaged_by` reference) in a prescriptive work-need.
///
/// Each step resolves to one of three states, projected from the ADR-019/020
/// categorical sidecar read (the SAME spine the immunity audit uses — no fork):
/// the work-need's verdict is the per-shape composition of its steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepState {
    /// The who-ref is attested at the current fingerprint (a fresh signer
    /// entry for this name exists in the site's sidecar). The step is closed.
    Attested,
    /// The sidecar was readable and the item present, but this who-ref is NOT
    /// attested at the current fingerprint (no signer, or only stale entries).
    /// The step is open — evaluable, just not yet satisfied.
    Unattested,
    /// The step is **un-evaluable**: the site has no sidecar, the sidecar is
    /// schema-invalid, or the item entry is absent. The audit cannot tell
    /// whether the work is done — it is OUT OF FRAME, never "overdue"
    /// (the ATK-PRES-8 gem guard, the prescriptive analog of ATK-3V-4).
    Unevaluable,
}

/// Why a work-need landed `OutOfFrame` — the sub-cause inside the un-evaluable
/// verdict (math-researcher `SubCauseCollapseInTheUnit`, the Layer-2 sibling of
/// the cardinality-collapse).
///
/// [`WorkVerdict::OutOfFrame`] is a single atomic value reached from several
/// DISTINGUISHABLE causes whose remedies genuinely differ. Under ADR-034
/// (audit-output IS the board), the bare verdict value cannot route the remedy —
/// so the verdict carries this typed sub-cause, exactly mirroring
/// [`UnreachedCause`] + [`UnreachedCause::remedy`] for the coverage audit. The
/// gem guard ([`WorkVerdict::OutOfFrame`] ≠ [`WorkVerdict::Overdue`], ATK-PRES-8)
/// is UNTOUCHED — this refines *within* `OutOfFrame`, it does not split the
/// four-valued verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutOfFrameCause {
    /// A who-ref's sidecar is missing or schema-invalid, or the item entry is
    /// absent — the who-step is un-evaluable. Remedy: scaffold + sign the
    /// `.attest/<item>.json` sidecar for the named who-ref.
    UnknownWhoRef,
    /// The shape declares NO who-step at all (a bare `#[panel]`, an `#[rx]` with
    /// no `filled_by`) — there is nothing to attest, so satisfaction is
    /// structurally un-evaluable. Remedy: declare the missing who-step.
    MissingWorkStep,
    /// The frame string is present but not a parseable ISO-8601 date — the
    /// deadline is un-readable, so the audit cannot place the need in or out of
    /// frame. Remedy: fix the malformed `due`/`until`/`runs_until`/`re_triage_due`.
    UnparseableFrame,
    /// An S3 `triage.priority_order` code-site ref does not resolve to a scanned
    /// site (ADR-017-Amd1) — the ordering is over sites the audit cannot see.
    /// Remedy: fix the dangling ref (or wait for multi-crate Layer-2 if it is a
    /// cross-crate site).
    UnresolvableRef,
}

impl OutOfFrameCause {
    /// The remedy class this sub-cause routes to — rendered into the board so an
    /// adopter learns *what to do* about an `OutOfFrame` need, not merely *that*
    /// it is un-evaluable. Distinct per cause: collapsing them would re-fuse the
    /// `SubCauseCollapseInTheUnit` this enum exists to prevent (mirrors
    /// [`UnreachedCause::remedy`]).
    #[must_use]
    pub const fn remedy(self) -> &'static str {
        match self {
            Self::UnknownWhoRef => {
                "scaffold + sign the .attest/<item>.json sidecar so the named \
                 who-ref's attestation is readable"
            }
            Self::MissingWorkStep => {
                "declare the missing who-step (filled_by / ordered_by / triaged_by \
                 / closure) — an empty work-need has nothing to attest"
            }
            Self::UnparseableFrame => {
                "fix the malformed frame date (due / until / runs_until / \
                 re_triage_due must be ISO-8601 YYYY-MM-DD)"
            }
            Self::UnresolvableRef => {
                "fix the dangling priority_order code-site reference (or, for a \
                 cross-crate target, await multi-crate Layer-2 resolution)"
            }
        }
    }
}

/// One resolved who-step in a prescriptive verdict.
///
/// Carries the role the reference plays in its shape + the reference text + its
/// resolved state. Rendered by the board so an adopter sees WHICH step blocks a
/// work-need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDetail {
    /// The chain-role this reference plays (`ordered_by` / `filled_by` /
    /// `reviewed_by` / `triaged_by` / `investigator` / `closure`), for display.
    pub role: String,
    /// The who-ref text (a signer name) or, for S3 ordering, a code-site ref.
    pub reference: String,
    /// How the step resolved against the site's sidecar.
    pub state: StepState,
}

/// The audit's per-site verdict for one prescriptive work-need declaration.
///
/// Pairs a [`crate::scan::PrescriptiveDeclaration`] with the four-valued
/// [`WorkVerdict`] the audit computed for it + the per-step detail that
/// explains the verdict (the board renders `blocking` loudly for `Overdue`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrescriptiveVerdict {
    /// The work-need declaration being graded.
    pub declaration: crate::scan::PrescriptiveDeclaration,
    /// The computed four-valued work verdict.
    pub verdict: WorkVerdict,
    /// Per-who-step resolution detail (the chain for S1, the rule-outs for S2,
    /// the ordering refs for S3, the closure witness for S4).
    pub steps: Vec<StepDetail>,
    /// A short human gloss of what blocks this work-need from `Fulfilled`
    /// (the un-attested step, the elapsed frame, the unresolvable ref).
    /// `None` when `Fulfilled`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking: Option<String>,
    /// The TYPED sub-cause when `verdict == OutOfFrame` (math-researcher
    /// `SubCauseCollapseInTheUnit` fix): which of the distinguishable
    /// un-evaluable causes fired, so the board routes a per-cause remedy rather
    /// than fusing them. `None` for every non-`OutOfFrame` verdict (the field is
    /// meaningful only inside the un-evaluable verdict).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_of_frame_cause: Option<OutOfFrameCause>,
}

/// Aggregate prescriptive audit report.
///
/// Every work-need declaration projected to a [`WorkVerdict`]. This is the
/// substrate the audit board renders (ADR-033 §Decision 4 + ADR-034: a live
/// projection, recomputed every run, never stored).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrescriptiveAuditReport {
    /// One verdict per prescriptive declaration discovered in the scan.
    pub verdicts: Vec<PrescriptiveVerdict>,
}

impl PrescriptiveAuditReport {
    /// True when no work-need is `Overdue` — the board has no loud rows.
    /// Tier-honest: this is NOT "all fulfilled" (`Pending` + `OutOfFrame` are
    /// quiet-but-open states); it is "nothing is late".
    #[must_use]
    pub fn is_clean(&self) -> bool {
        !self.verdicts.iter().any(|v| v.verdict.is_loud())
    }

    /// Count of `Overdue` (loud) work-needs — the headline number the board
    /// sorts to the top.
    #[must_use]
    pub fn overdue_count(&self) -> usize {
        self.verdicts
            .iter()
            .filter(|v| v.verdict == WorkVerdict::Overdue)
            .count()
    }

    /// Count of work-needs at a given verdict — lets the board show a
    /// per-state summary line.
    #[must_use]
    pub fn count_by_verdict(&self, verdict: WorkVerdict) -> usize {
        self.verdicts
            .iter()
            .filter(|v| v.verdict == verdict)
            .count()
    }

    /// The verdicts ordered for the board: `Overdue` first (loud), then
    /// `OutOfFrame` (needs investigation), then `Pending`, then `Fulfilled`
    /// (clean) — the loudness gradient ADR-023 mandates. Stable within a band
    /// (preserves scan order, which is file+line order).
    #[must_use]
    pub fn board_ordered(&self) -> Vec<&PrescriptiveVerdict> {
        let mut ordered: Vec<&PrescriptiveVerdict> = self.verdicts.iter().collect();
        ordered.sort_by_key(|v| match v.verdict {
            WorkVerdict::Overdue => 0u8,
            WorkVerdict::OutOfFrame => 1,
            WorkVerdict::Pending => 2,
            WorkVerdict::Fulfilled => 3,
        });
        ordered
    }
}

/// The resolved `(state)` of one who-ref against a site's sidecar, plus whether
/// the read itself was possible. Reuses the ADR-019/020 categorical evaluator
/// (the verdict-lattice isomorphism — prescriptive who-step satisfaction is the
/// same substrate-current categorical read as immunity discipline-witness).
///
/// `against = Current` pins the read to the item's structural fingerprint
/// (NFA-21): a signer who signed against an older fingerprint does NOT satisfy
/// the step (the leaf fails), so re-attestation is forced when code mutates.
fn resolve_who_step(decl: &crate::scan::PrescriptiveDeclaration, who_ref: &str) -> StepState {
    use antigen_attestation::evaluate::evaluate_predicate_with_kind;
    use antigen_attestation::predicate::{Leaf, Predicate, SignerCurrency};
    use antigen_attestation::AuditHint as AH;

    // Load the site's sidecar. The prescriptive declaration carries no
    // `antigen_type`, so the sidecar filename is the annotated item's identity
    // (`item_target.label()` last segment) — the same `.attest/<stem>.json`
    // convention `load_sidecar` resolves. A missing or schema-invalid sidecar
    // means the step is UN-EVALUABLE (OutOfFrame), never overdue — the gem.
    let item_label = decl.item_target.label();
    let sidecar = match load_sidecar(&decl.file, &item_label) {
        SidecarLoad::Ok(r) => r,
        SidecarLoad::Missing | SidecarLoad::SchemaInvalid => return StepState::Unevaluable,
    };
    let Some(item) = sidecar.items.iter().find(|i| i.item_path == item_label) else {
        // Sidecar exists but has no entry for this item (e.g. a stale sidecar
        // predating a rename) — un-evaluable, same as a missing sidecar.
        return StepState::Unevaluable;
    };

    // NFA-21 fingerprint-pin: prefer the scan-computed current digest (reflects
    // the item's code on disk now); fall back to the sidecar's stored value for
    // legacy pre-fingerprint declarations (mirrors `audit_substrate_witness`).
    let current_fingerprint: &str = if decl.structural_fingerprint.is_empty() {
        &item.current_fingerprint
    } else {
        &decl.structural_fingerprint
    };

    // Build a `signers(required=[who])` leaf, current-pinned. A passing
    // predicate ⇒ the name is attested at the current fingerprint ⇒ Attested.
    // A failing predicate (name absent, or only stale entries) ⇒ Unattested
    // (evaluable, just not satisfied — distinct from un-evaluable).
    let predicate = Predicate::leaf(Leaf::Signers {
        required: vec![who_ref.to_string()],
        roles: std::collections::BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: Vec::new(),
        signature_prefer: None,
    });
    let ctx = FilesystemAuditContext;
    let evaluated = evaluate_predicate_with_kind(
        &predicate,
        item,
        current_fingerprint,
        &decl.file,
        sidecar.kind,
        &ctx,
    )
    .unwrap_or_else(|_| antigen_attestation::EvaluatedPredicate::sidecar_schema_invalid());

    // Read `(satisfied, evaluable)` off the EvaluatedPredicate:
    //   - sidecar-missing / schema-invalid hints ⇒ un-evaluable (defensive;
    //     the load_sidecar branch above already caught the file-level cases,
    //     but the evaluator can also emit these for a malformed predicate).
    //   - any non-None witness tier ⇒ the predicate PASSED ⇒ Attested. (With
    //     `against=Current`, a passing predicate means a current-fingerprint
    //     entry exists for the name; a stale-only name fails the leaf, so it
    //     cannot reach a passing tier — NFA-21 holds.)
    //   - otherwise (DisciplinePredicateFailed / Deferred) ⇒ Unattested.
    match evaluated.audit_hint {
        AH::DisciplineSidecarMissing | AH::DisciplineSidecarSchemaInvalid => StepState::Unevaluable,
        _ if evaluated.witness_tier != antigen_attestation::WitnessTier::None => {
            StepState::Attested
        }
        _ => StepState::Unattested,
    }
}

/// Resolve whether an S3 `triage.priority_order` code-site reference resolves to
/// a real scanned site (ADR-017 Amendment 1). An unresolvable ref makes the
/// whole triage **`OutOfFrame`** (un-evaluable), never silent-satisfied (the gem,
/// ATK-PRES-14) and never `Overdue`.
///
/// v0.3 ceiling: resolution is INTRA-WORKSPACE — a ref resolves iff it matches a
/// site the scan walked (an `item_target` label, or a `file::item` string).
/// Cross-crate `priority_order` refs are a Layer-2 concern (multi-crate scan,
/// ADR-017-Amd1); until that lands, a cross-crate ref reads as unresolvable here
/// (tier-honest: the scan did not see it, so the audit cannot resolve it).
fn priority_order_ref_resolves(report: &ScanReport, ref_text: &str) -> bool {
    let needle = ref_text.trim();
    if needle.is_empty() {
        return false;
    }
    // A QUALIFIED ref (`Type::method`, `mod::item`) must match precisely: either
    // the full label, or a label whose qualified suffix is the ref (so a ref
    // `Foo::bar` resolves a label `crate::Foo::bar`). An UNQUALIFIED ref (a bare
    // ident, no `::`) is recall-tuned (scan-discipline): it resolves against a
    // label's LEAF segment. The split prevents the over-match where two distinct
    // sites `a::bar` and `b::bar` would both satisfy a bare `bar` *and* where a
    // qualified `a::bar` would wrongly satisfy an unrelated `b::bar` (a label-tail
    // collision the precise path forbids). Full cross-crate canonical-path
    // resolution is Layer-2 (ADR-017-Amd1); this is the intra-workspace floor.
    let needle_is_qualified = needle.contains("::");
    let needle_leaf = needle.rsplit("::").next().unwrap_or(needle).trim();

    // Collect every scanned item label across the declaration families that
    // carry an item_target (the resolvable code sites the scan reached).
    // `AntigenDeclaration` is excluded deliberately — it names a TYPE (the
    // failure-class), not a code-site the way `priority_order` references one;
    // the resolvable sites are the presents/immune/prescriptive item-targets.
    let labels = report
        .presentations
        .iter()
        .map(|p| p.item_target.label())
        .chain(report.immunities.iter().map(|i| i.item_target.label()))
        .chain(
            report
                .prescriptive_declarations
                .iter()
                .map(|d| d.item_target.label()),
        );
    for label in labels {
        let label = label.trim().to_owned();
        if needle_is_qualified {
            // Precise: exact match, or the label ends with `::<ref>` (the ref is
            // a qualified suffix of a more-qualified label).
            if label == needle || label.ends_with(&format!("::{needle}")) {
                return true;
            }
        } else {
            // Recall-tuned: a bare ident resolves against a label's leaf segment.
            let label_leaf = label.rsplit("::").next().unwrap_or(&label).trim();
            if label == needle || label_leaf == needle_leaf {
                return true;
            }
        }
    }
    false
}

/// Audit the prescriptive work-orchestration declarations (ADR-033).
///
/// Projects each to a four-valued [`WorkVerdict`] via the per-shape satisfaction
/// semantics aristotle ruled (decisions.md §Verdict-semantics-per-shape). The
/// satisfaction read REUSES the ADR-019/020 categorical spine (`load_sidecar`
/// plus [`antigen_attestation::evaluate::evaluate_predicate_with_kind`]) — there
/// is NO new witness machinery, only a new COMPOSITION of who-step states per
/// shape.
///
/// The four shapes compose their steps thus:
///
/// - **S1 `RoleWorkflow`** (`panel`/`rx`/`refer`/`biopsy`): conjunction over the
///   ordered chain `ordered_by` → ALL `filled_by` → ALL `reviewed_by`. A
///   `reviewed_by` step is credited ONLY when every `filled_by` step is attested
///   ("you cannot review what is not filled" — ATK-PRES-15 = ALL, not ANY).
/// - **S2 Elimination** (`ddx`): each `rule_out` alternative is independently
///   closeable; satisfied when the investigator/reviewer who-steps close.
/// - **S3 Ordering** (`triage`): satisfied = `triaged_by` attested AND within
///   `re_triage_due` AND every `priority_order` code-site ref resolves. An
///   unresolvable ref ⇒ un-evaluable ⇒ `OutOfFrame` (ATK-PRES-14). A standing
///   ordering — `triaged_by` alone does NOT permanently fulfill; the frame
///   expires it (freshness, not bypass).
/// - **S4 `FrameOnly`** (`culture`/`quarantine`): satisfied requires POSITIVE
///   CLOSURE (a closure attestation), NEVER frame-expiry alone. Frame elapsed +
///   un-closed ⇒ `Overdue`, never `Fulfilled` — the `fresh_through`-bypass guard
///   (ATK-PRES-13), the load-bearing S4 invariant.
///
/// `evaluable = false` (⇒ `OutOfFrame`) whenever a who-ref's sidecar is
/// unreadable, an S3 `priority_order` ref is unresolvable, or the frame string
/// is unparseable — the gem guard against silent `Overdue` false-alarms.
#[must_use]
pub fn audit_prescriptive(report: &ScanReport, _workspace_root: &Path) -> PrescriptiveAuditReport {
    use crate::scan::WorkShape;

    let today = chrono::Local::now().date_naive();
    let mut verdicts = Vec::with_capacity(report.prescriptive_declarations.len());

    for decl in &report.prescriptive_declarations {
        let frame = FrameState::classify(decl.frame.as_deref(), today);
        let mut steps: Vec<StepDetail> = Vec::new();

        // Per-shape satisfaction + evaluability. Each arm fills `steps` and
        // computes `(satisfied, evaluable, blocking-gloss, un-evaluable-cause)`;
        // the verdict is the shared `WorkVerdict::project` (the gem guard lives
        // there). The cause is `Some` only on the un-evaluable path the arm took.
        let (satisfied, evaluable, blocking, shape_cause) = match decl.kind.shape() {
            WorkShape::RoleWorkflow => eval_role_workflow(decl, &mut steps),
            WorkShape::Elimination => eval_elimination(decl, &mut steps),
            WorkShape::Ordering => eval_ordering(report, decl, frame, &mut steps),
            WorkShape::FrameOnly => eval_frame_only(decl, frame, &mut steps),
        };

        // An unparseable frame is itself an un-evaluable input (we cannot read
        // the deadline) — fold it into evaluability so `project` lands OutOfFrame
        // rather than guessing Pending/Overdue.
        let frame_unparseable = matches!(frame, FrameState::Unparseable);
        let evaluable = evaluable && !frame_unparseable;

        let verdict = WorkVerdict::project(satisfied, evaluable, frame);
        let blocking = if verdict == WorkVerdict::Fulfilled {
            None
        } else {
            Some(blocking)
        };

        // The typed sub-cause is meaningful ONLY for OutOfFrame (the
        // SubCauseCollapseInTheUnit fix). An unparseable frame is its own cause
        // and takes precedence — when both a shape-cause and a bad frame are
        // present, the frame is reported (a frame we cannot read blocks every
        // other reading). Otherwise the shape's own un-evaluable cause is used.
        let out_of_frame_cause = if verdict == WorkVerdict::OutOfFrame {
            if frame_unparseable {
                Some(OutOfFrameCause::UnparseableFrame)
            } else {
                shape_cause
            }
        } else {
            None
        };

        verdicts.push(PrescriptiveVerdict {
            declaration: decl.clone(),
            verdict,
            steps,
            blocking,
            out_of_frame_cause,
        });
    }

    PrescriptiveAuditReport { verdicts }
}

/// S1 — conjunction over the ordered chain `ordered_by` → ALL `filled_by` →
/// ALL `reviewed_by`. Returns `(satisfied, evaluable, blocking-gloss, cause)`
/// where `cause` is the typed `OutOfFrameCause` when the un-evaluable path fired.
fn eval_role_workflow(
    decl: &crate::scan::PrescriptiveDeclaration,
    steps: &mut Vec<StepDetail>,
) -> (bool, bool, String, Option<OutOfFrameCause>) {
    // Build the ordered chain of (role-label, who-ref). `ordered_by` is a single
    // optional ref; `filled_by` and `reviewed_by` are lists (refer's `to` and
    // biopsy's `deep_investigation_by` land in `filled_by` per scan extraction).
    let mut chain: Vec<(&str, &str)> = Vec::new();
    if let Some(orderer) = decl.ordered_by.as_deref() {
        chain.push(("ordered_by", orderer));
    }
    for f in &decl.filled_by {
        chain.push(("filled_by", f));
    }
    // reviewed_by is held back: it is credited ONLY when every filled_by is
    // attested ("you cannot review what is not filled").
    for (role, who) in &chain {
        let state = resolve_who_step(decl, who);
        steps.push(StepDetail {
            role: (*role).to_string(),
            reference: (*who).to_string(),
            state,
        });
    }

    // A site with NO who-steps at all (e.g. a bare `#[refer]` with no `to`) has
    // an empty chain — there is nothing to attest, so it is structurally
    // un-evaluable (we cannot say it is late or done). OutOfFrame, never Overdue.
    // Snapshot the filler-derived state as owned values BEFORE the reviewer loop
    // mutates `steps` (the chain pushed so far IS exactly the filler steps).
    let no_fillers = steps.is_empty();
    let any_unevaluable = steps.iter().any(|s| s.state == StepState::Unevaluable);

    // Witness-forgery guard (ATK / forward/s1-bare-orderer-fulfills-bypass):
    // `ordered_by` is an OPENING witness — it records that the work was ordered,
    // NOT that it was performed. Ordering ≠ performing ≠ reviewing (ADR-003:
    // presentation ≠ clearance). A bare `ordered_by` (no `filled_by`) must
    // NEVER alone fulfill the need; crediting the orderer as a closer is
    // accepting a positive non-closure event as closure — the witness-forgery
    // sibling of the three-valued gem (fix = TIGHTEN the satisfaction predicate,
    // not widen the codomain). The orderer's step is still REQUIRED to attest
    // (the chain conjunction holds), but a genuine closing step — at least one
    // `filled_by` — must also exist and attest. A bare-orderer site is therefore
    // Pending (awaiting fill), never Fulfilled.
    let has_closing_step = steps.iter().any(|s| s.role != "ordered_by");
    let all_fillers_attested =
        has_closing_step && steps.iter().all(|s| s.state == StepState::Attested);

    // reviewed_by: credited only when every filler is attested (ALL / conjunction).
    let mut reviewers_attested = true;
    let mut any_reviewer = false;
    let mut reviewer_unevaluable = false;
    for r in &decl.reviewed_by {
        any_reviewer = true;
        // A reviewer present while a filler is un-attested is PREMATURE — not
        // credited. We still resolve+display its state for the board, but it
        // does not contribute to satisfaction unless all fillers are attested.
        let state = resolve_who_step(decl, r);
        if state == StepState::Unevaluable {
            reviewer_unevaluable = true;
        }
        if !all_fillers_attested || state != StepState::Attested {
            reviewers_attested = false;
        }
        steps.push(StepDetail {
            role: "reviewed_by".to_string(),
            reference: r.clone(),
            state,
        });
    }

    let evaluable = !any_unevaluable && !reviewer_unevaluable && !no_fillers;
    let satisfied = all_fillers_attested && (!any_reviewer || reviewers_attested);

    // Typed un-evaluable sub-cause (only consulted when the verdict is
    // OutOfFrame): no who-step at all is MissingWorkStep; an unreadable who-ref
    // sidecar is UnknownWhoRef.
    let cause = if no_fillers {
        Some(OutOfFrameCause::MissingWorkStep)
    } else if any_unevaluable || reviewer_unevaluable {
        Some(OutOfFrameCause::UnknownWhoRef)
    } else {
        None
    };
    let blocking = if no_fillers {
        "no who-step declared — nothing to attest (declare filled_by/ordered_by)".to_string()
    } else if any_unevaluable || reviewer_unevaluable {
        "a who-step is un-evaluable (no sidecar / unknown who-ref) — out of frame".to_string()
    } else if !has_closing_step {
        // Bare ordered_by (possibly attested) with no filled_by: the work was
        // ordered but no one has performed it. ordered_by opens; it never alone
        // fulfills (witness-forgery guard).
        "awaiting fill: ordered but no filled_by step — an opener never alone fulfills".to_string()
    } else if !all_fillers_attested {
        "awaiting fill: not every filled_by step is attested".to_string()
    } else {
        "awaiting review: reviewed_by not yet attested (all fillers done)".to_string()
    };
    (satisfied, evaluable, blocking, cause)
}

/// S2 — each `rule_out` alternative (held in `items`) is independently
/// closeable; the ddx is satisfied when its closing who-steps (`investigator`
/// in `filled_by`, `reviewer` in `reviewed_by`) attest. Returns
/// `(satisfied, evaluable, blocking-gloss, cause)`.
fn eval_elimination(
    decl: &crate::scan::PrescriptiveDeclaration,
    steps: &mut Vec<StepDetail>,
) -> (bool, bool, String, Option<OutOfFrameCause>) {
    // Record the rule-out alternatives for the board (display only — they are
    // the differential, closed collectively by the investigator/reviewer).
    for alt in &decl.items {
        steps.push(StepDetail {
            role: "rule_out".to_string(),
            reference: alt.clone(),
            state: StepState::Unattested,
        });
    }
    // Closure who-steps: investigator (filled_by) + reviewer (reviewed_by).
    let mut closure_refs: Vec<(&str, &str)> = Vec::new();
    for f in &decl.filled_by {
        closure_refs.push(("investigator", f));
    }
    for r in &decl.reviewed_by {
        closure_refs.push(("reviewer", r));
    }
    let mut any_unevaluable = false;
    let mut all_attested = !closure_refs.is_empty();
    for (role, who) in &closure_refs {
        let state = resolve_who_step(decl, who);
        if state == StepState::Unevaluable {
            any_unevaluable = true;
        }
        if state != StepState::Attested {
            all_attested = false;
        }
        steps.push(StepDetail {
            role: (*role).to_string(),
            reference: (*who).to_string(),
            state,
        });
    }
    let evaluable = !any_unevaluable && !closure_refs.is_empty();
    let satisfied = all_attested;
    let cause = if closure_refs.is_empty() {
        Some(OutOfFrameCause::MissingWorkStep)
    } else if any_unevaluable {
        Some(OutOfFrameCause::UnknownWhoRef)
    } else {
        None
    };
    let blocking = if closure_refs.is_empty() {
        "no investigator/reviewer declared — the differential cannot be closed".to_string()
    } else if any_unevaluable {
        "a closure who-step is un-evaluable (no sidecar / unknown who-ref)".to_string()
    } else {
        "awaiting elimination: investigator/reviewer not yet attested".to_string()
    };
    (satisfied, evaluable, blocking, cause)
}

/// S3 — triage is a standing re-validated ORDERING. Satisfied = `triaged_by`
/// (held in `filled_by`) attested AND within `re_triage_due` (the frame, checked
/// by the caller's `project`) AND every `priority_order` code-site ref (held in
/// `items`) resolves. An unresolvable ref ⇒ un-evaluable ⇒ `OutOfFrame`
/// (ATK-PRES-14, ADR-017-Amd1). Returns `(satisfied, evaluable, blocking-gloss, cause)`.
fn eval_ordering(
    report: &ScanReport,
    decl: &crate::scan::PrescriptiveDeclaration,
    frame: FrameState,
    steps: &mut Vec<StepDetail>,
) -> (bool, bool, String, Option<OutOfFrameCause>) {
    // 1. Resolve every priority_order code-site ref. An unresolvable ref makes
    //    the whole triage un-evaluable (we cannot grade an ordering over sites
    //    that don't exist) — OutOfFrame, never silent-satisfied.
    let mut all_refs_resolve = !decl.items.is_empty();
    let mut unresolved: Vec<&str> = Vec::new();
    for ref_text in &decl.items {
        let resolves = priority_order_ref_resolves(report, ref_text);
        if !resolves {
            all_refs_resolve = false;
            unresolved.push(ref_text);
        }
        steps.push(StepDetail {
            role: "priority_order".to_string(),
            reference: ref_text.clone(),
            state: if resolves {
                StepState::Attested
            } else {
                StepState::Unevaluable
            },
        });
    }

    // 2. Resolve triaged_by attestation (held in filled_by per scan extraction).
    let mut triaged_attested = !decl.filled_by.is_empty();
    let mut triager_unevaluable = false;
    for who in &decl.filled_by {
        let state = resolve_who_step(decl, who);
        if state == StepState::Unevaluable {
            triager_unevaluable = true;
        }
        if state != StepState::Attested {
            triaged_attested = false;
        }
        steps.push(StepDetail {
            role: "triaged_by".to_string(),
            reference: who.clone(),
            state,
        });
    }

    // evaluable ⇔ all refs resolve AND the triager who-step is readable. (A
    // missing triaged_by makes it Pending, not un-evaluable — the ordering is
    // declared, just not yet attested.)
    let evaluable = all_refs_resolve && !triager_unevaluable && !decl.items.is_empty();

    // S3 is a STANDING re-validated ordering, not a terminal task. aristotle's
    // ruling: Fulfilled = triaged_by attested AND WITHIN re_triage_due. A triage
    // attested but PAST re_triage_due is **Overdue** (the ordering is stale —
    // re-triage owed), NOT Fulfilled. So the re_triage_due frame elapsing must
    // de-satisfy even a fresh attestation: fold Past into un-satisfaction here so
    // `project(satisfied=false, frame=Past)` lands Overdue. (Within/None frame +
    // attested ⇒ satisfied ⇒ Fulfilled. Re-triaging re-attests, which — combined
    // with a fresh re_triage_due — re-earns Fulfilled.) This is the freshness
    // discipline that keeps a triage honest, not the bypass it guards against.
    let satisfied = triaged_attested && !matches!(frame, FrameState::Past);
    // Typed un-evaluable sub-cause: empty ordering is MissingWorkStep; a dangling
    // priority_order ref is UnresolvableRef (takes precedence over the triager —
    // we cannot grade an ordering over sites that don't exist); an unreadable
    // triaged_by sidecar is UnknownWhoRef.
    let cause = if decl.items.is_empty() {
        Some(OutOfFrameCause::MissingWorkStep)
    } else if !all_refs_resolve {
        Some(OutOfFrameCause::UnresolvableRef)
    } else if triager_unevaluable {
        Some(OutOfFrameCause::UnknownWhoRef)
    } else {
        None
    };
    let blocking = if decl.items.is_empty() {
        "no priority_order declared — nothing to order".to_string()
    } else if !all_refs_resolve {
        format!("priority_order ref(s) do not resolve to a scanned code site: {unresolved:?} — out of frame (ADR-017-Amd1)")
    } else if triager_unevaluable {
        "triaged_by is un-evaluable (no sidecar / unknown who-ref)".to_string()
    } else if !triaged_attested {
        "awaiting triage: triaged_by not yet attested".to_string()
    } else {
        "re-triage owed: triaged_by attested but re_triage_due elapsed (the ordering is stale)"
            .to_string()
    };
    (satisfied, evaluable, blocking, cause)
}

/// S4 — frame-only (`culture`/`quarantine`). Satisfaction requires POSITIVE
/// CLOSURE (a closure attestation in the site's sidecar), NEVER frame-expiry
/// alone. A site whose frame has elapsed WITHOUT a closure attestation is
/// `Overdue`, never `Fulfilled` — the `fresh_through`-bypass guard (ATK-PRES-13).
/// Returns `(satisfied, evaluable, blocking-gloss, cause)`.
///
/// v0.3 IMPLEMENTATION CEILING (tier-honest): the ratified §Proc-Macro-Surface
/// gives the S4 macros NO closure who-ref field (`culture` = `test_kind` /
/// `duration` / `runs_until`; `quarantine` = `scope` / `until` / `reason`). So
/// `decl.filled_by` is empty for every S4 site today, and the positive-closure
/// EVENT (a release
/// attestation, or the named test going green) is not yet OBSERVABLE — that is
/// the same Layer-2 cross-reference machinery as triage ref-resolution. The
/// consequence is the SAFE direction: with no closure who-ref, `satisfied` is
/// always false, so an S4 site is Pending within frame / **Overdue** past it,
/// and NEVER Fulfilled by expiry. The bypass is structurally impossible; the
/// path to Fulfilled is gated, not collapsed.
///
/// This function is FORWARD-COMPATIBLE: it reads the closure who-ref from
/// `filled_by` exactly as the other shapes do, so when the macro+scan gain a
/// closure field (the Layer-2 follow-up: `quarantine.released_by` /
/// `culture.green_by` mapping to `filled_by`), the Fulfilled path lights up with
/// no change here. Routed to the team as a question (the §Proc-Macro-Surface vs
/// §Verdict-semantics field gap) rather than guessed at solo.
///
/// Absent closure ⇒ unsatisfied (evaluable) — Pending within frame, Overdue past
/// it. The frame expiring does NOT flip unsatisfied→satisfied (the bypass guard).
fn eval_frame_only(
    decl: &crate::scan::PrescriptiveDeclaration,
    _frame: FrameState,
    steps: &mut Vec<StepDetail>,
) -> (bool, bool, String, Option<OutOfFrameCause>) {
    // The closure who-step(s). If none declared, the site can NEVER reach
    // Fulfilled via frame-expiry — it stays Pending (within frame) / Overdue
    // (past), which is exactly the positive-closure guard: a culture/quarantine
    // with no closure witness is never silently fulfilled.
    let mut closure_attested = !decl.filled_by.is_empty();
    let mut any_unevaluable = false;
    for who in &decl.filled_by {
        let state = resolve_who_step(decl, who);
        if state == StepState::Unevaluable {
            any_unevaluable = true;
        }
        if state != StepState::Attested {
            closure_attested = false;
        }
        steps.push(StepDetail {
            role: "closure".to_string(),
            reference: who.clone(),
            state,
        });
    }

    // evaluable: if a closure who-ref is declared but un-evaluable (no sidecar),
    // the site is OutOfFrame. If NO closure who-ref is declared, the site is
    // still EVALUABLE — we CAN tell it is un-closed (and therefore Pending /
    // Overdue by frame). This is the key S4 distinction: "no closure declared"
    // is unsatisfied-but-evaluable (it WILL go Overdue when the frame elapses),
    // NOT un-evaluable. Un-evaluable is reserved for "the sidecar can't be read".
    let evaluable = !any_unevaluable;
    let satisfied = closure_attested;

    // Typed un-evaluable sub-cause: an S4 site is OutOfFrame ONLY when a declared
    // closure who-ref's sidecar is unreadable (UnknownWhoRef). The "no closure
    // declared" case is evaluable-unsatisfied (Pending/Overdue, not OutOfFrame),
    // so it carries no cause — the verdict logic above keeps it out of OutOfFrame.
    let cause = if any_unevaluable {
        Some(OutOfFrameCause::UnknownWhoRef)
    } else {
        None
    };
    let blocking = if decl.filled_by.is_empty() {
        "no closure attestation declared — frame-expiry alone never fulfills (positive-closure guard, ATK-PRES-13)".to_string()
    } else if any_unevaluable {
        "closure who-step is un-evaluable (no sidecar / unknown who-ref)".to_string()
    } else {
        "awaiting closure: the named closure attestation is not yet recorded".to_string()
    };
    (satisfied, evaluable, blocking, cause)
}

#[cfg(test)]
mod tests {
    use super::*;
    // `#[defended_by]` (ADR-029) is used only on witness tests in this module;
    // import it here rather than at crate-module scope (where it would be unused).
    use antigen_macros::defended_by;

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

    // ========================================================================
    // Recurrent-Emergence audit (ADR-024 §Family 2)
    // ========================================================================

    fn recurrent_decl(
        kind: crate::scan::RecurrentKind,
        antigen_type: Option<&str>,
    ) -> crate::scan::RecurrentDeclaration {
        crate::scan::RecurrentDeclaration {
            kind,
            name: None,
            antigen_type: antigen_type.map(str::to_string),
            description: None,
            instances: None,
            since: None,
            rationale: None,
            from_itches: Vec::new(),
            anchored_by: Vec::new(),
            managed_by: None,
            contributing_to: None,
            file: std::path::PathBuf::from("test.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("t".to_string()),
        }
    }

    #[test]
    fn audit_recurrent_itch_without_antigen_flags_not_anchored() {
        let mut report = ScanReport::default();
        report
            .recurrent_declarations
            .push(recurrent_decl(crate::scan::RecurrentKind::Itch, None));
        let out = audit_recurrent(&report);
        assert_eq!(out.concern_count, 1);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::ItchNoticedNotAnchored));
    }

    #[test]
    fn audit_recurrent_itch_with_antigen_is_clean() {
        let mut report = ScanReport::default();
        report.recurrent_declarations.push(recurrent_decl(
            crate::scan::RecurrentKind::Itch,
            Some("SomeAntigen"),
        ));
        let out = audit_recurrent(&report);
        assert!(out.all_clean());
    }

    #[test]
    fn audit_recurrent_anchor_without_downstream_action_flags() {
        let mut report = ScanReport::default();
        report.recurrent_declarations.push(recurrent_decl(
            crate::scan::RecurrentKind::RecurrenceAnchor,
            Some("MsrvCreep"),
        ));
        let out = audit_recurrent(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::RecurrenceThresholdReachedNoAction));
    }

    #[test]
    fn audit_recurrent_crystallize_empty_flags_without_antigen() {
        let mut report = ScanReport::default();
        report.recurrent_declarations.push(recurrent_decl(
            crate::scan::RecurrentKind::Crystallize,
            None,
        ));
        let out = audit_recurrent(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::CrystallizeWithoutAntigen));
    }

    #[test]
    fn audit_recurrent_chronic_without_managed_by_flags_unmanaged() {
        let mut report = ScanReport::default();
        report.recurrent_declarations.push(recurrent_decl(
            crate::scan::RecurrentKind::Chronic,
            Some("FlakeyStep"),
        ));
        let out = audit_recurrent(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::ChronicSignalUnmanaged));
    }

    #[test]
    fn audit_recurrent_chronic_old_iso_since_flags_past_review_date() {
        let mut report = ScanReport::default();
        let mut decl = recurrent_decl(crate::scan::RecurrentKind::Chronic, Some("X"));
        decl.managed_by = Some("team".to_string());
        decl.since = Some("2020-01-01".to_string()); // far past horizon
        report.recurrent_declarations.push(decl);
        let out = audit_recurrent(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::ChronicSignalPastReviewDate));
    }

    #[test]
    fn audit_recurrent_chronic_version_since_skips_date_check() {
        // Non-ISO `since` (version tag) must NOT false-positive the
        // past-review-date check AND must NOT emit not-a-date.
        let mut report = ScanReport::default();
        let mut decl = recurrent_decl(crate::scan::RecurrentKind::Chronic, Some("X"));
        decl.managed_by = Some("team".to_string());
        decl.since = Some("v0.2.0".to_string());
        report.recurrent_declarations.push(decl);
        let out = audit_recurrent(&report);
        assert!(!out.audits[0]
            .hints
            .contains(&AuditHint::ChronicSignalPastReviewDate));
        assert!(!out.audits[0]
            .hints
            .contains(&AuditHint::ChronicSinceNotADate));
    }

    #[test]
    fn audit_recurrent_chronic_garbage_since_emits_not_a_date() {
        // Per ATK-RECURRENT-4a: `since` that is neither ISO date nor
        // version tag → chronic-since-not-a-date.
        let mut report = ScanReport::default();
        let mut decl = recurrent_decl(crate::scan::RecurrentKind::Chronic, Some("X"));
        decl.managed_by = Some("team".to_string());
        decl.since = Some("not-a-date".to_string());
        report.recurrent_declarations.push(decl);
        let out = audit_recurrent(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::ChronicSinceNotADate));
    }

    #[test]
    fn is_version_tag_recognizes_versions_rejects_garbage() {
        assert!(is_version_tag("v0.2.0"));
        assert!(is_version_tag("V1.4.3"));
        assert!(is_version_tag("1.4"));
        assert!(is_version_tag("2.0.0-rc.1"));
        assert!(is_version_tag("1.0.0+build42"));
        // Rejections — these should emit chronic-since-not-a-date.
        assert!(!is_version_tag("not-a-date"));
        assert!(!is_version_tag("yesterday"));
        assert!(!is_version_tag("v"));
        assert!(!is_version_tag(""));
        assert!(!is_version_tag("release-2"));
        // A bare integer "3" has no dot-separated structure → not a version.
        assert!(!is_version_tag("3"));
    }

    #[test]
    fn audit_recurrent_chronic_iso_date_not_flagged_not_a_date() {
        // Recent ISO date: no past-review-date AND no not-a-date.
        let mut report = ScanReport::default();
        let mut decl = recurrent_decl(crate::scan::RecurrentKind::Chronic, Some("X"));
        decl.managed_by = Some("team".to_string());
        let recent = (chrono::Utc::now().date_naive() - chrono::Duration::days(10))
            .format("%Y-%m-%d")
            .to_string();
        decl.since = Some(recent);
        report.recurrent_declarations.push(decl);
        let out = audit_recurrent(&report);
        assert!(out.audits[0].hints.is_empty());
    }

    #[test]
    fn audit_recurrent_saturate_without_contributing_to_flags() {
        let mut report = ScanReport::default();
        report
            .recurrent_declarations
            .push(recurrent_decl(crate::scan::RecurrentKind::Saturate, None));
        let out = audit_recurrent(&report);
        assert!(out.audits[0].hints.contains(&AuditHint::SaturateNoAnchor));
    }

    #[test]
    fn audit_recurrent_strand_without_anchors_flags() {
        let mut report = ScanReport::default();
        report
            .recurrent_declarations
            .push(recurrent_decl(crate::scan::RecurrentKind::Strand, None));
        let out = audit_recurrent(&report);
        assert!(out.audits[0].hints.contains(&AuditHint::StrandNoAnchors));
    }

    #[test]
    fn audit_recurrent_hint_serializes_kebab_case() {
        let s = serde_json::to_string(&AuditHint::ItchNoticedNotAnchored).unwrap();
        assert_eq!(s, "\"itch-noticed-not-anchored\"");
        let s2 = serde_json::to_string(&AuditHint::ChronicSignalPastReviewDate).unwrap();
        assert_eq!(s2, "\"chronic-signal-past-review-date\"");
    }

    // ========================================================================
    // Mucosal Boundary audit (ADR-027 + Amendment 1)
    // ========================================================================

    fn mucosal_decl(
        tag: crate::scan::MucosalKindTag,
        boundary_kind: Option<&str>,
        rationale: Option<&str>,
        target_fn: &str,
    ) -> crate::scan::MucosalDeclaration {
        crate::scan::MucosalDeclaration {
            tag,
            boundary_kind: boundary_kind.map(str::to_string),
            rationale: rationale.map(str::to_string),
            handled_by: None,
            accepts: None,
            reviewed_by: None,
            until: None,
            file: std::path::PathBuf::from("test.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn(target_fn.to_string()),
        }
    }

    #[test]
    fn audit_mucosal_clean_when_kind_and_rationale_present() {
        use crate::scan::MucosalKindTag;
        let mut report = ScanReport::default();
        report.mucosal_declarations.push(mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("UserInput"),
            Some("public form input; sanitized at template-render layer"),
            "handle_form",
        ));
        let out = audit_mucosal(&report);
        assert!(out.all_clean());
    }

    #[test]
    fn audit_mucosal_short_rationale_flags_insufficient() {
        use crate::scan::MucosalKindTag;
        let mut report = ScanReport::default();
        report.mucosal_declarations.push(mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("UserInput"),
            Some("short"),
            "f",
        ));
        let out = audit_mucosal(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::MucosalRationaleInsufficient));
    }

    #[test]
    fn audit_mucosal_delegate_missing_handler_flags_tier1() {
        use crate::scan::MucosalKindTag;
        let mut report = ScanReport::default();
        let mut d = mucosal_decl(
            MucosalKindTag::MucosalDelegate,
            Some("UserInput"),
            Some("delegated to sanitizer module for central handling"),
            "outer",
        );
        d.handled_by = Some("nonexistent_handler".to_string());
        report.mucosal_declarations.push(d);
        let out = audit_mucosal(&report);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::MucosalDisciplineDelegateTargetMissing));
    }

    #[test]
    fn audit_mucosal_delegate_kind_mismatch_flags_tier3() {
        use crate::scan::MucosalKindTag;
        // Handler `sanitize_db` carries #[mucosal(kind = DatabaseQuery)] only;
        // the delegate points UserInput at it → tier-3 kind-mismatch.
        let mut report = ScanReport::default();
        report.mucosal_declarations.push(mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("DatabaseQuery"),
            Some("parameterized queries enforced at this data-access layer"),
            "sanitize_db",
        ));
        let mut delegate = mucosal_decl(
            MucosalKindTag::MucosalDelegate,
            Some("UserInput"),
            Some("delegated to the shared sanitizer used across endpoints"),
            "outer",
        );
        delegate.handled_by = Some("sanitize_db".to_string());
        report.mucosal_declarations.push(delegate);
        let out = audit_mucosal(&report);
        let delegate_audit = out
            .audits
            .iter()
            .find(|a| a.declaration.tag == MucosalKindTag::MucosalDelegate)
            .unwrap();
        assert!(delegate_audit
            .hints
            .contains(&AuditHint::MucosalDisciplineDelegateTargetKindMismatch));
    }

    #[test]
    fn audit_mucosal_delegate_matching_kind_is_clean() {
        use crate::scan::MucosalKindTag;
        // Handler carries the matching kind → delegate is clean (set-membership).
        let mut report = ScanReport::default();
        report.mucosal_declarations.push(mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("UserInput"),
            Some("central user-input sanitizer; escapes + length-bounds"),
            "sanitize_input",
        ));
        let mut delegate = mucosal_decl(
            MucosalKindTag::MucosalDelegate,
            Some("UserInput"),
            Some("delegated to the central user-input sanitizer routine"),
            "outer",
        );
        delegate.handled_by = Some("sanitize_input".to_string());
        report.mucosal_declarations.push(delegate);
        let out = audit_mucosal(&report);
        let delegate_audit = out
            .audits
            .iter()
            .find(|a| a.declaration.tag == MucosalKindTag::MucosalDelegate)
            .unwrap();
        assert!(
            delegate_audit.hints.is_empty(),
            "matching-kind delegate must be clean; got {:?}",
            delegate_audit.hints
        );
    }

    #[test]
    fn audit_mucosal_delegate_hybrid_handler_set_membership() {
        use crate::scan::MucosalKindTag;
        // Hybrid handler carries TWO #[mucosal(kind)] on the same fn — the
        // delegate matches via set-membership, not first-declaration-only.
        let mut report = ScanReport::default();
        report.mucosal_declarations.push(mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("UserInput"),
            Some("hybrid handler: user-input branch sanitized here"),
            "hybrid_handler",
        ));
        report.mucosal_declarations.push(mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("ShellArgument"),
            Some("hybrid handler: shell-arg branch escaped here"),
            "hybrid_handler",
        ));
        let mut delegate = mucosal_decl(
            MucosalKindTag::MucosalDelegate,
            Some("ShellArgument"),
            Some("delegated to the hybrid handler covering both kinds"),
            "outer",
        );
        delegate.handled_by = Some("hybrid_handler".to_string());
        report.mucosal_declarations.push(delegate);
        let out = audit_mucosal(&report);
        let delegate_audit = out
            .audits
            .iter()
            .find(|a| a.declaration.tag == MucosalKindTag::MucosalDelegate)
            .unwrap();
        assert!(
            delegate_audit.hints.is_empty(),
            "hybrid-handler set-membership must match ShellArgument; got {:?}",
            delegate_audit.hints
        );
    }

    #[test]
    fn audit_mucosal_tolerant_floors_and_fields() {
        use crate::scan::MucosalKindTag;
        let mut report = ScanReport::default();
        let mut d = mucosal_decl(
            MucosalKindTag::MucosalTolerant,
            Some("UserInput"),
            Some("twenty-five char rationale!!"), // < 40
            "intake",
        );
        d.accepts = None; // missing
        d.reviewed_by = None; // missing
        report.mucosal_declarations.push(d);
        let out = audit_mucosal(&report);
        let h = &out.audits[0].hints;
        assert!(h.contains(&AuditHint::MucosalTolerantRationaleInsufficient));
        assert!(h.contains(&AuditHint::MucosalTolerantAcceptsEmpty));
        assert!(h.contains(&AuditHint::MucosalTolerantWithoutReviewer));
    }

    #[test]
    fn audit_mucosal_tolerant_complete_is_clean() {
        use crate::scan::MucosalKindTag;
        let mut report = ScanReport::default();
        let mut d = mucosal_decl(
            MucosalKindTag::MucosalTolerant,
            Some("ApiRequest"),
            Some("internal admin endpoint behind VPN; trusted-network assumption documented"),
            "admin_intake",
        );
        d.accepts = Some("admin-panel form posts".to_string());
        d.reviewed_by = Some("security-team".to_string());
        report.mucosal_declarations.push(d);
        let out = audit_mucosal(&report);
        assert!(out.all_clean());
    }

    #[test]
    fn audit_mucosal_hint_serializes_kebab_case() {
        let s =
            serde_json::to_string(&AuditHint::MucosalDisciplineDelegateTargetKindMismatch).unwrap();
        assert_eq!(s, "\"mucosal-discipline-delegate-target-kind-mismatch\"");
    }

    // ATK-MUCOSAL-1: same-name function in two different files — handler_kinds
    // is keyed by bare fn_name with no file path.  The two kind-sets are MERGED
    // under a single HashMap entry, so a delegate whose intended target carries
    // only one kind silently passes kind-checks that should fire because the
    // OTHER same-named function's kind bleeds into the merged set.
    //
    // Concrete exploit scenario:
    //   src/a.rs::process  #[mucosal(kind = "UserInput")]
    //   src/b.rs::process  #[mucosal(kind = "DatabaseQuery")]
    //   handler_kinds after build: "process" -> {"UserInput", "DatabaseQuery"}
    //
    //   A delegate intended for src/b.rs::process writes boundary = "UserInput"
    //   by mistake.  Correct audit: MucosalDisciplineDelegateTargetKindMismatch
    //   (b.rs only knows "DatabaseQuery").  Actual audit (broken): CLEAN because
    //   a.rs's "UserInput" is in the merged set.
    //
    // ATK-MUCOSAL-1 (FIXED): when two #[mucosal] functions share a bare name
    // in different source files, a delegate targeting that name by bare-string
    // must emit MucosalDisciplineDelegateTargetAmbiguous — the delegate is
    // underspecified. Before the fix, handler_kinds merged kind-sets under a
    // single bare-name key, so a kind-mismatch could silently pass if the
    // OTHER file's kind happened to match. Fix: build an ambiguous_names set
    // (names that map to 2+ distinct files) and check before kind-resolution.
    #[test]
    fn atk_mucosal_1_same_name_collision_masks_kind_mismatch() {
        use crate::scan::MucosalKindTag;

        let mut report = ScanReport::default();

        // src/a.rs::process — kind "UserInput"
        let mut mucosal_a = mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("UserInput"),
            Some("public form input sanitized at template-render layer"),
            "process",
        );
        mucosal_a.file = std::path::PathBuf::from("src/a.rs");
        report.mucosal_declarations.push(mucosal_a);

        // src/b.rs::process — kind "DatabaseQuery" (different file, same name)
        let mut mucosal_b = mucosal_decl(
            MucosalKindTag::Mucosal,
            Some("DatabaseQuery"),
            Some("parameterized query builder; never interpolates raw user input"),
            "process",
        );
        mucosal_b.file = std::path::PathBuf::from("src/b.rs");
        report.mucosal_declarations.push(mucosal_b);

        // Delegate: intended for src/b.rs::process but says boundary = "UserInput"
        // by mistake. Should flag MucosalDisciplineDelegateTargetKindMismatch.
        let mut delegate = mucosal_decl(
            MucosalKindTag::MucosalDelegate,
            Some("UserInput"),
            Some("delegated to the central process handler for sanitisation"),
            "outer",
        );
        delegate.file = std::path::PathBuf::from("src/c.rs");
        delegate.handled_by = Some("process".to_string());
        report.mucosal_declarations.push(delegate);

        let out = audit_mucosal(&report);
        let delegate_audit = out
            .audits
            .iter()
            .find(|a| a.declaration.tag == MucosalKindTag::MucosalDelegate)
            .unwrap();

        // CORRECT post-fix behavior (Option A ruling, findings/mucosal-same-name-fn-collision):
        // When two #[mucosal] functions share the same bare name in different files,
        // the delegate is AMBIGUOUS — emit MucosalDisciplineDelegateTargetAmbiguous
        // rather than attempting kind-resolution through the merged kind-set union.
        // This surfaces the structural problem (the delegate is underspecified) rather
        // than either silently passing or emitting a misleading kind-mismatch hint.
        assert!(
            delegate_audit
                .hints
                .contains(&AuditHint::MucosalDisciplineDelegateTargetAmbiguous),
            "ATK-MUCOSAL-1: delegate targeting 'process' when both src/a.rs::process \
             and src/b.rs::process exist must emit MucosalDisciplineDelegateTargetAmbiguous. \
             The delegate is underspecified — the bare name is not enough to identify the \
             target uniquely. hints: {:?}",
            delegate_audit.hints
        );
    }

    // ========================================================================
    // Antigen-Category audit (ADR-028 — G1 scan-time-only)
    // ========================================================================

    fn antigen_decl(
        type_name: &str,
        category: Vec<crate::category::AntigenCategory>,
    ) -> crate::scan::AntigenDeclaration {
        crate::scan::AntigenDeclaration {
            name: type_name.to_lowercase(),
            type_name: type_name.to_string(),
            file: std::path::PathBuf::from("test.rs"),
            line: 1,
            family: None,
            summary: None,
            fingerprint: None,
            canonical_path: None,
            category,
        }
    }

    #[test]
    fn audit_category_flags_absent_category() {
        let mut report = ScanReport::default();
        report
            .antigens
            .push(antigen_decl("LegacyAntigen", Vec::new()));
        let out = audit_category(&report);
        assert_eq!(out.defaulted_count, 1);
        assert_eq!(out.explicit_count, 0);
        assert!(!out.all_explicit());
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenCategoryDefaultedImplicitFunctional));
    }

    #[test]
    fn audit_category_clean_when_explicit() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        // FunctionalCorrectness (not SubstrateAlignment): an explicit category
        // with no witness is a plain coverage gap and emits no audit hint. A
        // witnessless SubstrateAlignment antigen would now (correctly) trip the
        // silence-no-witness advisory — that case is covered by
        // `silence_no_witness_fires_for_substrate_alignment_with_no_witness`;
        // this test isolates "explicit category → no G1 defaulted hint".
        report.antigens.push(antigen_decl(
            "VerbCorrectness",
            vec![AntigenCategory::FunctionalCorrectness],
        ));
        let out = audit_category(&report);
        assert_eq!(out.explicit_count, 1);
        assert_eq!(out.defaulted_count, 0);
        assert!(out.all_explicit());
        assert!(out.audits.is_empty());
    }

    #[test]
    fn audit_category_mixed_counts() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl("A", Vec::new()));
        report.antigens.push(antigen_decl(
            "B",
            vec![AntigenCategory::FunctionalCorrectness],
        ));
        report.antigens.push(antigen_decl("C", Vec::new()));
        let out = audit_category(&report);
        assert_eq!(out.defaulted_count, 2);
        assert_eq!(out.explicit_count, 1);
        assert_eq!(out.audits.len(), 2);
    }

    #[test]
    fn audit_category_hint_serializes_kebab_case() {
        let s =
            serde_json::to_string(&AuditHint::AntigenCategoryDefaultedImplicitFunctional).unwrap();
        assert_eq!(s, "\"antigen-category-defaulted-implicit-functional\"");
    }

    // ========================================================================
    // G2 category↔witness-type cross-check (ADR-028 + Amendment 2)
    // ========================================================================

    /// Build an immunity addressing `antigen_type`. `substrate` selects the
    /// witness type: `true` → substrate-witness (`requires = <predicate>`,
    /// empty `witness`); `false` → code-witness (non-empty `witness`).
    fn immunity_for(antigen_type: &str, substrate: bool) -> crate::scan::Immunity {
        crate::scan::Immunity {
            antigen_type: antigen_type.to_string(),
            witness: if substrate {
                String::new()
            } else {
                "some_test".to_string()
            },
            requires_predicate: if substrate {
                Some("{\"leaf\":\"doc\"}".to_string())
            } else {
                None
            },
            file: std::path::PathBuf::from("test.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("witness_site".to_string()),
            canonical_path: None,
            structural_fingerprint: String::new(),
        }
    }

    #[test]
    fn g2_substrate_alignment_with_only_code_witness_is_mismatch() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        // Code-witness immunity only — wrong type for a substrate-alignment
        // category, which needs a substrate-witness.
        report.immunities.push(immunity_for("DriftAntigen", false));
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 1);
        assert!(!out.no_category_witness_mismatch());
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType));
    }

    #[test]
    fn g2_functional_correctness_with_only_substrate_witness_is_mismatch() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "BugAntigen",
            vec![AntigenCategory::FunctionalCorrectness],
        ));
        report.immunities.push(immunity_for("BugAntigen", true));
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 1);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType));
    }

    #[test]
    fn g2_matching_witness_type_is_clean() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        report.immunities.push(immunity_for("DriftAntigen", true));
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 0);
        assert!(out.no_category_witness_mismatch());
        assert!(out.audits.is_empty());
    }

    #[test]
    fn g2_hybrid_needs_both_witness_types() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "HybridAntigen",
            vec![
                AntigenCategory::SubstrateAlignment,
                AntigenCategory::FunctionalCorrectness,
            ],
        ));
        // Only a substrate-witness — missing the code-witness axis. Per
        // aristotle's G3 F1 ruling, a hybrid with exactly one axis witnessed
        // is INCOMPLETE evidence, not a full claim-inconsistent violation.
        report.immunities.push(immunity_for("HybridAntigen", true));
        let out = audit_category(&report);
        assert_eq!(
            out.mismatch_count, 1,
            "hybrid with only one axis is a mismatch"
        );
        assert!(
            out.audits[0]
                .hints
                .contains(&AuditHint::AntigenCategoryHybridIncompleteEvidence),
            "hybrid with one axis witnessed → hybrid-incomplete-evidence"
        );

        // Add the code-witness axis — now clean.
        report.immunities.push(immunity_for("HybridAntigen", false));
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 0, "hybrid with both axes is clean");
    }

    #[test]
    fn g3_hybrid_with_zero_axes_is_claim_inconsistent_not_incomplete() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "HybridAntigen",
            vec![
                AntigenCategory::SubstrateAlignment,
                AntigenCategory::FunctionalCorrectness,
            ],
        ));
        // An immunity exists but is... neither: simulate a declared-but-empty
        // immunity by giving it neither a predicate nor a witness. (Both axes
        // unwitnessed → full violation, not partial.)
        report.immunities.push(crate::scan::Immunity {
            antigen_type: "HybridAntigen".to_string(),
            witness: String::new(),
            requires_predicate: None,
            file: std::path::PathBuf::from("test.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("witness_site".to_string()),
            canonical_path: None,
            structural_fingerprint: String::new(),
        });
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 1);
        assert!(
            out.audits[0]
                .hints
                .contains(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType),
            "hybrid with ZERO axes witnessed → claim-inconsistent (full violation)"
        );
    }

    #[test]
    fn g3_hybrid_incomplete_evidence_hint_serializes_kebab_case() {
        let s = serde_json::to_string(&AuditHint::AntigenCategoryHybridIncompleteEvidence).unwrap();
        assert_eq!(s, "\"antigen-category-hybrid-incomplete-evidence\"");
    }

    #[test]
    fn g2_no_immunity_is_not_a_mismatch() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        // Explicit category but zero immunities addressing it — that's a
        // coverage gap, not a category↔witness-type mismatch. G2's count stays
        // zero. (For a SubstrateAlignment antigen the no-witness case ALSO
        // emits the silence-no-witness advisory — see
        // `silence_no_witness_fires_for_substrate_alignment_with_no_witness` —
        // so this test uses a FunctionalCorrectness antigen to isolate the
        // pure G2 "no immunity is not a mismatch" assertion.)
        report.antigens.push(antigen_decl(
            "UncoveredAntigen",
            vec![AntigenCategory::FunctionalCorrectness],
        ));
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 0);
        assert!(out.no_category_witness_mismatch());
        assert!(out.no_silence_witness_mismatch());
        assert!(out.audits.is_empty());
    }

    // ========================================================================
    // Silence-witness shape-mismatch hints (scientist design + aristotle gate,
    // forward/silence-witness-shape-mismatch-{hint,impl}, 2026-05-27).
    //
    // Hint 1 (no-witness): a SubstrateAlignment antigen with NO witness at all —
    //   the silence-by-absence generator. Fills the gap G2 deliberately leaves
    //   (G2 treats no-witness as an orthogonal coverage gap and bails).
    // Hint 2 (wrong-tier): a SubstrateAlignment antigen whose ONLY witnesses are
    //   code-tier (witness=fn / #[defended_by]) with no requires= predicate.
    //   Co-emitted alongside G2's claim-inconsistent (same root cause), adding
    //   the silence-generator framing. Suppressed when a substrate witness is
    //   also present.
    // ========================================================================

    #[test]
    fn silence_no_witness_fires_for_substrate_alignment_with_no_witness() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        // No immunity, no defense at all.
        let out = audit_category(&report);
        // Not a G2 mismatch — there is no witness TYPE to cross-check.
        assert_eq!(out.mismatch_count, 0);
        assert!(out.no_category_witness_mismatch());
        // But the silence-by-absence advisory fires.
        assert!(!out.no_silence_witness_mismatch());
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenWitnessShapeMismatchForSilenceNoWitness));
    }

    #[test]
    fn silence_no_witness_does_not_fire_for_functional_correctness_no_witness() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        // A FunctionalCorrectness antigen with no witness is a plain coverage
        // gap, NOT silence-by-absence — the no-witness advisory is
        // SubstrateAlignment-only.
        report.antigens.push(antigen_decl(
            "BugAntigen",
            vec![AntigenCategory::FunctionalCorrectness],
        ));
        let out = audit_category(&report);
        assert!(out.no_silence_witness_mismatch());
        assert!(out.audits.is_empty());
    }

    #[test]
    fn silence_wrong_tier_co_emits_with_g2_for_substrate_alignment_code_only() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        // Code-witness only — wrong tier for substrate-alignment.
        report.immunities.push(immunity_for("DriftAntigen", false));
        let out = audit_category(&report);
        // G2 still fires its claim-inconsistent verdict (count unchanged)...
        assert_eq!(out.mismatch_count, 1);
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType));
        // ...and the silence wrong-tier advisory rides alongside it on the
        // SAME audit entry, adding the silence-generator framing.
        assert!(!out.no_silence_witness_mismatch());
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenWitnessShapeMismatchForSilenceWrongTier));
    }

    #[test]
    fn silence_wrong_tier_suppressed_when_substrate_witness_also_present() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        // Both a substrate-witness AND a code-witness. The code test is now
        // supplementary — the wrong-tier advisory must NOT fire (scientist's
        // suppression rule), and G2 is clean (substrate axis satisfied).
        report.immunities.push(immunity_for("DriftAntigen", true));
        report.immunities.push(immunity_for("DriftAntigen", false));
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 0);
        assert!(out.no_silence_witness_mismatch());
    }

    #[test]
    fn silence_wrong_tier_fires_for_substrate_alignment_defended_by_code_only() {
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        // A `#[defended_by]` registration is a CODE-TIER witness (ADR-029) — so
        // a SubstrateAlignment antigen defended ONLY by it is the wrong-tier
        // case, exactly as a `witness = fn` immunity would be. Mirrors the G2
        // defended_by handling: canonical_path=None matches.
        report.defenses.push(crate::scan::Defense {
            antigen_type: "DriftAntigen".to_string(),
            file: std::path::PathBuf::from("test.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("defending_test".to_string()),
            canonical_path: None,
        });
        let out = audit_category(&report);
        assert_eq!(out.mismatch_count, 1);
        assert!(!out.no_silence_witness_mismatch());
        assert!(out.audits[0]
            .hints
            .contains(&AuditHint::AntigenWitnessShapeMismatchForSilenceWrongTier));
    }

    #[test]
    fn silence_witness_hints_serialize_kebab_case() {
        let no_witness =
            serde_json::to_string(&AuditHint::AntigenWitnessShapeMismatchForSilenceNoWitness)
                .unwrap();
        assert_eq!(
            no_witness,
            "\"antigen-witness-shape-mismatch-for-silence-no-witness\""
        );
        let wrong_tier =
            serde_json::to_string(&AuditHint::AntigenWitnessShapeMismatchForSilenceWrongTier)
                .unwrap();
        assert_eq!(
            wrong_tier,
            "\"antigen-witness-shape-mismatch-for-silence-wrong-tier\""
        );
    }

    #[test]
    fn g2_hint_serializes_kebab_case() {
        let s =
            serde_json::to_string(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType)
                .unwrap();
        assert_eq!(
            s,
            "\"antigen-category-claim-inconsistent-with-predicate-type\""
        );
    }

    // ========================================================================
    // ATK-G2-adr029-migration: G2 cross-check wired for post-ADR-029 witnesses
    //
    // FIXED (findings/g2-crosscheck-blind-to-adr029-witnesses): audit_category()
    // now consults report.defenses (#[defended_by] registrations) in addition to
    // report.immunities (#[immune] declarations) when computing has_code_witness.
    //
    // Prior gap: ADR-029 witnesses use report.defenses, not report.immunities.
    // When a SubstrateAlignment antigen was defended only via #[defended_by], the
    // G2 cross-check saw has_any_immunity=false and early-returned without a hint.
    // Any adopter migrating from #[immune] to #[defended_by] silently bypassed G2.
    //
    // Fix: a matching Defense (by antigen_type) now sets has_any_immunity=true and
    // has_code_witness=true in the category loop, exactly as a witness=fn immunity
    // did before. SubstrateAlignment antigens defended only by code-tier
    // #[defended_by] witnesses now correctly receive AntigenCategoryClaimInconsistentWithPredicateType.
    // ========================================================================

    #[test]
    fn atk_g2_substrate_alignment_with_only_defended_by_triggers_g2_hint() {
        // ATK-G2-migration (FIXED): a SubstrateAlignment antigen defended ONLY by
        // `#[defended_by]` (code-tier, ADR-029 style) now correctly triggers G2's
        // witness-type cross-check. Before the fix, G2 read `report.immunities`
        // only and silently passed SubstrateAlignment antigens defended via
        // `report.defenses`. The fix: `audit_category()` consults `report.defenses`
        // too, setting `has_any_immunity=true` and `has_code_witness=true` when a
        // matching `Defense` is found — which correctly triggers the
        // `AntigenCategoryClaimInconsistentWithPredicateType` hint for a
        // SubstrateAlignment antigen with a code-tier-only witness.
        use crate::category::AntigenCategory;
        let mut report = ScanReport::default();
        report.antigens.push(antigen_decl(
            "DriftAntigen",
            vec![AntigenCategory::SubstrateAlignment],
        ));
        // ADR-029 style: #[defended_by(DriftAntigen)] on a test function.
        // This is a CODE-TIER witness — wrong for SubstrateAlignment.
        // After the fix, G2 consults report.defenses and correctly flags this.
        report.defenses.push(crate::scan::Defense {
            antigen_type: "DriftAntigen".to_string(),
            file: std::path::PathBuf::from("tests/test.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("test_drift_antigen".to_string()),
            canonical_path: None,
        });
        let out = audit_category(&report);

        // FIXED: G2 now reports a mismatch — report.defenses is consulted.
        assert_eq!(
            out.mismatch_count, 1,
            "ATK-G2-migration (fixed): SubstrateAlignment antigen with only a \
            code-tier #[defended_by] witness must trigger AntigenCategoryClaimInconsistentWithPredicateType. \
            audit_category() now consults report.defenses alongside report.immunities."
        );
        assert_eq!(
            out.audits.len(),
            1,
            "ATK-G2-migration (fixed): exactly one audit entry for the wrong-type ADR-029 witness"
        );
        assert!(
            out.audits[0]
                .hints
                .contains(&AuditHint::AntigenCategoryClaimInconsistentWithPredicateType),
            "ATK-G2-migration (fixed): the audit entry must include the category-mismatch hint"
        );
    }

    // ========================================================================
    // Audit-SF-1 regression (structural_fingerprint from scan overrides
    // sidecar's stored current_fingerprint for staleness detection)
    // ========================================================================

    #[test]
    #[defended_by(AuditFingerprintSelfReferential)]
    fn audit_sf1_scan_fingerprint_overrides_sidecar_stored_fingerprint() {
        // This test confirms Audit-SF-1 is resolved: before the fix, audit
        // used the sidecar's stored current_fingerprint for stale-signer
        // detection. A signer who signed against "fp-old" looked current
        // because both sides of the comparison came from the same sidecar
        // (self-referential). After the fix, audit uses the scan-computed
        // structural_fingerprint — so a signer at "fp-old" is correctly
        // detected as stale when the real item digest is "fp-new".
        use antigen_attestation::predicate::{Leaf, SignerCurrency};
        use antigen_attestation::schema::{
            AntigenIdentifier, ItemRatification, Ratification, RatificationKind, SchemaVersion,
            Signer, SignerBasis,
        };
        use chrono::NaiveDate;
        use std::collections::BTreeMap;

        let tmp = tempfile::tempdir().unwrap();
        let source_file = tmp.path().join("src").join("lib.rs");
        std::fs::create_dir_all(source_file.parent().unwrap()).unwrap();
        // The actual source file doesn't need to exist for this test —
        // we only need the sidecar to be loadable from .attest/.
        let attest_dir = tmp.path().join("src").join(".attest");
        std::fs::create_dir_all(&attest_dir).unwrap();

        // Sidecar: signer signed against "fp-old". The sidecar stores
        // current_fingerprint: "fp-old" — under the old self-referential
        // behavior the signer would appear current (both sides == "fp-old").
        let sidecar = Ratification {
            schema_version: SchemaVersion::V1,
            kind: RatificationKind::Immunity,
            antigen: AntigenIdentifier {
                name: "DriftTestAntigen".to_string(),
                defined_in: None,
            },
            source_file: source_file.clone(),
            items: vec![ItemRatification {
                item_path: "the_fn".to_string(),
                current_fingerprint: "fp-old".to_string(),
                doc_ref: None,
                signers: vec![Signer {
                    name: "alice".to_string(),
                    role: None,
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    signed_against_fingerprint: "fp-old".to_string(),
                    basis: SignerBasis::Fresh {
                        reasoning: Some("reviewed".to_string()),
                    },
                    strength: antigen_attestation::tier::SignatureStrength::TextStamp,
                    signature: None,
                }],
                oracles: vec![],
                fresh_through: None,
                extensions: BTreeMap::new(),
            }],
        };
        let sidecar_json = serde_json::to_string_pretty(&sidecar).unwrap();
        std::fs::write(attest_dir.join("DriftTestAntigen.json"), sidecar_json).unwrap();

        // Predicate: alice must be current (signed against the item's live digest).
        let pred = antigen_attestation::Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let pred_json = serde_json::to_string(&pred).unwrap();

        // Immunity: structural_fingerprint = "fp-new" (the code has drifted from
        // what alice signed — she signed fp-old but the item is now fp-new).
        let immunity = crate::scan::Immunity {
            antigen_type: "DriftTestAntigen".to_string(),
            witness: String::new(),
            requires_predicate: Some(pred_json),
            file: source_file,
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("the_fn".to_string()),
            canonical_path: None,
            structural_fingerprint: "fp-new".to_string(),
        };

        let pred_json_ref = immunity.requires_predicate.as_deref().unwrap();
        let result = audit_substrate_witness(&immunity, pred_json_ref);

        // The signer is stale: alice signed fp-old but the item is now fp-new.
        // Audit-SF-1 fix: structural_fingerprint wins, not sidecar's stored value.
        //
        // With against=Current, eval_signers compares signed_against_fingerprint
        // against the live structural_fingerprint ("fp-new"). Alice signed "fp-old"
        // which does not match "fp-new", so the predicate fails. Under the OLD
        // self-referential behavior (sidecar's stored current_fingerprint = "fp-old"),
        // alice's "fp-old" would have matched — falsely passing the predicate.
        // DisciplinePredicateFailed IS the correct staleness signal here (the
        // predicate correctly rejects alice because she's stale vs the live code).
        assert_eq!(
            result.audit_hint,
            AuditHint::DisciplinePredicateFailed,
            "Audit-SF-1: scan-computed structural_fingerprint (fp-new) must override \
             sidecar's stored current_fingerprint (fp-old) for staleness detection. \
             Alice signed fp-old but the item is now fp-new → predicate correctly fails. \
             Old behavior: sidecar's fp-old == alice's fp-old → false-green. Got: {result:?}"
        );
        assert_eq!(
            result.witness_tier,
            WitnessTier::None,
            "failed predicate maps to tier=None"
        );
    }

    #[test]
    fn audit_sf1_legacy_path_no_structural_fingerprint_uses_sidecar_stored() {
        // When structural_fingerprint is empty (legacy sidecar / pre-SF-1 report),
        // audit falls back to the sidecar's stored current_fingerprint. This
        // preserves backwards-compatibility and avoids falsely marking all
        // existing sidecars as stale.
        use antigen_attestation::predicate::{Leaf, SignerCurrency};
        use antigen_attestation::schema::{
            AntigenIdentifier, ItemRatification, Ratification, RatificationKind, SchemaVersion,
            Signer, SignerBasis,
        };
        use chrono::NaiveDate;
        use std::collections::BTreeMap;

        let tmp = tempfile::tempdir().unwrap();
        let source_file = tmp.path().join("src").join("lib.rs");
        std::fs::create_dir_all(source_file.parent().unwrap()).unwrap();
        let attest_dir = tmp.path().join("src").join(".attest");
        std::fs::create_dir_all(&attest_dir).unwrap();

        // Signer signed against "fp-consistent" and sidecar stores it.
        let sidecar = Ratification {
            schema_version: SchemaVersion::V1,
            kind: RatificationKind::Immunity,
            antigen: AntigenIdentifier {
                name: "LegacyAntigen".to_string(),
                defined_in: None,
            },
            source_file: source_file.clone(),
            items: vec![ItemRatification {
                item_path: "legacy_fn".to_string(),
                current_fingerprint: "fp-consistent".to_string(),
                doc_ref: None,
                signers: vec![Signer {
                    name: "alice".to_string(),
                    role: None,
                    date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    signed_against_fingerprint: "fp-consistent".to_string(),
                    basis: SignerBasis::Fresh {
                        reasoning: Some("reviewed".to_string()),
                    },
                    strength: antigen_attestation::tier::SignatureStrength::TextStamp,
                    signature: None,
                }],
                oracles: vec![],
                fresh_through: None,
                extensions: BTreeMap::new(),
            }],
        };
        let sidecar_json = serde_json::to_string_pretty(&sidecar).unwrap();
        std::fs::write(attest_dir.join("LegacyAntigen.json"), sidecar_json).unwrap();

        let pred = antigen_attestation::Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let pred_json = serde_json::to_string(&pred).unwrap();

        // Empty structural_fingerprint → legacy path (use sidecar's stored value).
        let immunity = crate::scan::Immunity {
            antigen_type: "LegacyAntigen".to_string(),
            witness: String::new(),
            requires_predicate: Some(pred_json),
            file: source_file,
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("legacy_fn".to_string()),
            canonical_path: None,
            structural_fingerprint: String::new(),
        };

        let pred_json_ref = immunity.requires_predicate.as_deref().unwrap();
        let result = audit_substrate_witness(&immunity, pred_json_ref);

        // Alice is current under the legacy (self-referential) path:
        // sidecar stores fp-consistent; alice signed fp-consistent → match.
        assert_eq!(
            result.audit_hint,
            AuditHint::DisciplinePredicatePassedSubstrateCurrent,
            "legacy path: empty structural_fingerprint falls back to sidecar's stored \
             current_fingerprint for backwards-compat. Got: {result:?}"
        );
        assert_eq!(result.witness_tier, WitnessTier::Execution);
    }

    // ========================================================================
    // ATK-SIDECAR-FIRST-ITEM: when an antigen sidecar holds ratifications for
    // multiple items (two `#[immune]` sites in the same file sharing the same
    // antigen sidecar), `audit_substrate_witness` must look up the entry by
    // `item_path` matching the immunity's `item_target.label()` — NOT use
    // `items.first()`. The first()-shortcut silently audited each immunity
    // against the FIRST item's signers/fingerprint, so `second_fn`'s immunity
    // would pass spuriously when `alice` had only signed `first_fn`.
    // (findings/sidecar-first-item-wrong-audit, adversarial.)
    // ========================================================================

    #[test]
    fn sidecar_per_item_lookup_does_not_use_first_item_for_second_immunity() {
        use antigen_attestation::predicate::{Leaf, SignerCurrency};
        use antigen_attestation::schema::{
            AntigenIdentifier, ItemRatification, Ratification, RatificationKind, SchemaVersion,
            Signer, SignerBasis,
        };
        use chrono::NaiveDate;
        use std::collections::BTreeMap;

        let tmp = tempfile::tempdir().unwrap();
        let source_file = tmp.path().join("src").join("lib.rs");
        std::fs::create_dir_all(source_file.parent().unwrap()).unwrap();
        let attest_dir = tmp.path().join("src").join(".attest");
        std::fs::create_dir_all(&attest_dir).unwrap();

        // Discriminating fixture: first_fn UNSIGNED, second_fn SIGNED by alice
        // against the live digest (fp-2). Immunity addresses second_fn.
        //   - Pre-fix `items.first()`: consults first_fn's entry → signers=[]
        //     → alice missing → DisciplinePredicateFailed (FAIL).
        //   - Fixed per-item lookup: consults second_fn's entry → alice signed
        //     fp-2 == live fp-2 → DisciplinePredicatePassedSubstrateCurrent (PASS).
        // The PASS is the load-bearing signal; only the per-item lookup delivers
        // it. The mirror fixture (first signed, second unsigned, immunity on
        // second) does NOT discriminate — both old and new return FAIL there,
        // just for different reasons. Falsified 2026-05-28: this test FAILS
        // against the items.first() shortcut and PASSES against the lookup fix.
        let sidecar = Ratification {
            schema_version: SchemaVersion::V1,
            kind: RatificationKind::Immunity,
            antigen: AntigenIdentifier {
                name: "TwoFnAntigen".to_string(),
                defined_in: None,
            },
            source_file: source_file.clone(),
            items: vec![
                ItemRatification {
                    item_path: "first_fn".to_string(),
                    current_fingerprint: "fp-1".to_string(),
                    doc_ref: None,
                    signers: vec![], // first_fn: UNSIGNED (would fail signers=[alice])
                    oracles: vec![],
                    fresh_through: None,
                    extensions: BTreeMap::new(),
                },
                ItemRatification {
                    item_path: "second_fn".to_string(),
                    current_fingerprint: "fp-2".to_string(),
                    doc_ref: None,
                    // second_fn: alice signed against the live digest fp-2.
                    signers: vec![Signer {
                        name: "alice".to_string(),
                        role: None,
                        date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                        signed_against_fingerprint: "fp-2".to_string(),
                        basis: SignerBasis::Fresh {
                            reasoning: Some("reviewed second_fn".to_string()),
                        },
                        strength: antigen_attestation::tier::SignatureStrength::TextStamp,
                        signature: None,
                    }],
                    oracles: vec![],
                    fresh_through: None,
                    extensions: BTreeMap::new(),
                },
            ],
        };
        let sidecar_json = serde_json::to_string_pretty(&sidecar).unwrap();
        std::fs::write(attest_dir.join("TwoFnAntigen.json"), sidecar_json).unwrap();

        let pred = antigen_attestation::Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        });
        let pred_json = serde_json::to_string(&pred).unwrap();

        // Immunity addressing SECOND_FN (the signed item).
        let immunity = crate::scan::Immunity {
            antigen_type: "TwoFnAntigen".to_string(),
            witness: String::new(),
            requires_predicate: Some(pred_json.clone()),
            file: source_file,
            line: 10,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("second_fn".to_string()),
            canonical_path: None,
            structural_fingerprint: "fp-2".to_string(),
        };

        let result = audit_substrate_witness(&immunity, &pred_json);

        assert_eq!(
            result.audit_hint,
            AuditHint::DisciplinePredicatePassedSubstrateCurrent,
            "ATK-SIDECAR-FIRST-ITEM (FIXED): second_fn's immunity must consult \
             second_fn's ratification (alice signed fp-2 == live fp-2 → PASS). A \
             DisciplinePredicateFailed result here means the lookup regressed to \
             `items.first()` and was reading first_fn's UNSIGNED entry. Got: {result:?}"
        );
        assert_eq!(result.witness_tier, WitnessTier::Execution);
    }

    // ========================================================================
    // ADR-029 — per-presents-site immune-state verdicts (presentation_verdicts)
    //
    // Immunity is observed, not declared: audit() cross-references each
    // #[presents(X)] against the #[defended_by(X)] witnesses + #[immune] audits
    // and grades defended / undefended / substrate-gap. These pin that surface;
    // the adversarial ATK (atk_adr029_defended_by_audit) exercises it end-to-end.
    // ========================================================================

    fn presents_site(antigen: &str, file: &str, line: usize) -> crate::scan::Presentation {
        crate::scan::Presentation {
            antigen_type: antigen.to_string(),
            file: PathBuf::from(file),
            line,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Unknown { line },
            match_kind: crate::scan::MatchKind::ExplicitMarker,
            canonical_path: None,
            inherited_from: None,
            structural_fingerprint: String::new(),
            requires_predicate: None,
            proof: None,
        }
    }

    fn defended_by_witness(antigen: &str, file: &str, line: usize) -> crate::scan::Defense {
        crate::scan::Defense {
            antigen_type: antigen.to_string(),
            file: PathBuf::from(file),
            line,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn(format!("witness_{antigen}")),
            canonical_path: None,
        }
    }

    #[test]
    fn verdict_defended_by_grants_reachability() {
        // A #[defended_by(X)] witness cross-references a #[presents(X)] site:
        // the verdict is Defended at Reachability (v0.3 audit does not run
        // coverage; code-tier witness = Reachability, the honest tier).
        let mut report = ScanReport::default();
        report
            .presentations
            .push(presents_site("FailureClass", "src/lib.rs", 10));
        report
            .defenses
            .push(defended_by_witness("FailureClass", "src/tests.rs", 5));

        let out = audit(&report, Path::new("."));
        assert_eq!(out.presentation_verdicts.len(), 1);
        let v = &out.presentation_verdicts[0];
        assert_eq!(v.antigen_type, "FailureClass");
        assert_eq!(
            v.verdict,
            ImmuneVerdict::Defended {
                tier: WitnessTier::Reachability
            },
            "a registered code-tier witness grants Defended/Reachability; got {:?}",
            v.verdict
        );
        assert_eq!(v.defended_by, vec!["src/tests.rs:5".to_string()]);
        assert!(
            out.undefended_verdicts().is_empty(),
            "the defended site must not appear in undefended_verdicts()"
        );
    }

    #[test]
    fn verdict_no_witness_is_undefended() {
        // A #[presents(X)] with no #[defended_by(X)] and no #[immune] is
        // Undefended — the CI-gateable failure state.
        let mut report = ScanReport::default();
        report
            .presentations
            .push(presents_site("FailureClass", "src/lib.rs", 10));

        let out = audit(&report, Path::new("."));
        assert_eq!(out.presentation_verdicts.len(), 1);
        assert_eq!(
            out.presentation_verdicts[0].verdict,
            ImmuneVerdict::Undefended
        );
        assert_eq!(out.undefended_verdicts().len(), 1);
    }

    #[test]
    fn verdict_wrong_class_witness_does_not_pollute() {
        // ADR-029 / ATK-ADR029-3: a #[defended_by(WrongClass)] witness must NOT
        // grant a RightClass presents-site a defended verdict. Class-level match
        // is strict on antigen_type.
        let mut report = ScanReport::default();
        report
            .presentations
            .push(presents_site("RightClass", "src/lib.rs", 10));
        report
            .defenses
            .push(defended_by_witness("WrongClass", "src/tests.rs", 5));

        let out = audit(&report, Path::new("."));
        assert_eq!(out.presentation_verdicts.len(), 1);
        assert_eq!(
            out.presentation_verdicts[0].verdict,
            ImmuneVerdict::Undefended,
            "WrongClass witness must not cross-reference to RightClass; got {:?}",
            out.presentation_verdicts[0].verdict
        );
        assert!(out.presentation_verdicts[0].defended_by.is_empty());
    }

    #[test]
    fn verdict_immune_backward_compat_still_defends() {
        // The deprecated #[immune] path still contributes: a same-item
        // #[immune(X, witness=fn)] audit grants the presents-site a Defended
        // verdict so adopters migrate to #[defended_by] gradually.
        let mut report = ScanReport::default();
        report
            .presentations
            .push(presents_site("PanickingInDrop", "src/lib.rs", 20));
        // Co-located #[immune] on the same item (same file + Unknown{line:20}).
        report.immunities.push(crate::scan::Immunity {
            antigen_type: "PanickingInDrop".to_string(),
            witness: "no_panic_drop_test".to_string(),
            requires_predicate: None,
            file: PathBuf::from("src/lib.rs"),
            line: 20,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Unknown { line: 20 },
            canonical_path: None,
            structural_fingerprint: String::new(),
        });
        // Provide the witness fn so the immune audit resolves to a real tier.
        // (No function index entry → NotFound → tier None; the verdict then
        // falls through to Undefended. That is the honest outcome for an
        // unresolvable witness — we assert Undefended here to pin it, then a
        // resolvable-witness case is covered by the ATK integration test which
        // walks a real workspace root.)
        let out = audit(&report, Path::new("."));
        assert_eq!(out.presentation_verdicts.len(), 1);
        // The witness `no_panic_drop_test` doesn't exist under "." → NotFound →
        // tier None → the immune path contributes nothing → Undefended. This
        // pins that a BROKEN immune witness does not falsely defend.
        assert_eq!(
            out.presentation_verdicts[0].verdict,
            ImmuneVerdict::Undefended,
            "an unresolvable #[immune] witness must not grant a defended verdict"
        );
    }

    #[test]
    fn verdict_skips_fingerprint_inferred_presentations() {
        // ADR-029 verdicts grade DECLARED intent only. A fingerprint-inferred
        // presentation (MatchKind::FingerprintMatch) is the scan's broad triage
        // signal — it must NOT get a verdict, or the surface floods with
        // structural-pattern noise the developer never declared.
        let mut report = ScanReport::default();
        let mut inferred = presents_site("SomeClass", "src/lib.rs", 10);
        inferred.match_kind = crate::scan::MatchKind::FingerprintMatch;
        report.presentations.push(inferred);
        // An explicit marker for a different class, to prove the filter is
        // per-presentation (not all-or-nothing).
        report
            .presentations
            .push(presents_site("ExplicitClass", "src/lib.rs", 20));

        let out = audit(&report, Path::new("."));
        assert_eq!(
            out.presentation_verdicts.len(),
            1,
            "only the explicit presents-site gets a verdict; the fingerprint-\
             inferred match is skipped. got: {:?}",
            out.presentation_verdicts
        );
        assert_eq!(out.presentation_verdicts[0].antigen_type, "ExplicitClass");
    }

    // ========================================================================
    // ATK-PV-REQUIRES-MASKED (FIXED per ADR-029 Amendment 1, 2026-05-31):
    // Substrate-intent precedence — a failing requires= is not masked by a code witness.
    //
    // When a presents-site has BOTH a requires= predicate (substrate intent) AND a
    // #[defended_by] code witness, AND the requires= fails, the verdict must be
    // SubstrateGap — the developer declared substrate intent that is broken. A code
    // witness operates in a different channel and does not resolve a broken substrate
    // predicate (sub-clause F + ADR-029 Amendment 1).
    //
    // Previously: max(code_tier=Reachability, substrate_tier=None) = Reachability →
    // Defended(Reachability). The substrate gap was invisible.
    // ========================================================================

    fn presents_site_with_requires(
        antigen: &str,
        file: &str,
        line: usize,
        pred_json: &str,
    ) -> crate::scan::Presentation {
        let mut site = presents_site(antigen, file, line);
        site.requires_predicate = Some(pred_json.to_string());
        site
    }

    #[test]
    fn atk_pv_requires_masked_by_code_witness() {
        // A presents-site has both:
        //   (a) requires = <predicate>  — substrate intent, will FAIL (no sidecar under ".")
        //   (b) a #[defended_by] code witness — exists, grants Reachability
        //
        // CORRECT behavior (ADR-029 Amendment 1): SubstrateGap. The failing requires=
        // declares substrate intent that is broken. The code witness is in a different
        // channel; it does not resolve the substrate gap. Substrate-intent takes precedence.
        //
        // Any valid predicate JSON — the sidecar won't exist under "." so
        // audit_substrate_witness returns WitnessTier::None regardless of predicate content.
        let pred_json = r#"{"Signers":{"required":["alice"],"roles":{},"against":"Current","signature_allow":[],"signature_prefer":null}}"#;

        let mut report = ScanReport::default();
        report.presentations.push(presents_site_with_requires(
            "SubstrateDriftClass",
            "src/lib.rs",
            10,
            pred_json,
        ));
        // Code witness exists — previously this masked the substrate gap.
        report.defenses.push(defended_by_witness(
            "SubstrateDriftClass",
            "src/tests.rs",
            5,
        ));

        // No sidecar under "." → requires= predicate fails → site_requires_eval=Some(None).
        // ADR-029 Amendment 1: requires_present_and_failed=true → SubstrateGap,
        // even though code_tier=Some(Reachability).
        let out = audit(&report, Path::new("."));
        assert_eq!(out.presentation_verdicts.len(), 1);
        let v = &out.presentation_verdicts[0];

        assert_eq!(
            v.verdict,
            ImmuneVerdict::SubstrateGap,
            "ATK-PV-REQUIRES-MASKED: a failing requires= predicate must surface \
             SubstrateGap even when a code witness exists. The developer declared \
             substrate intent (requires=) that is broken; a code witness in a different \
             channel does not resolve it. verdict: {:?}",
            v.verdict
        );
    }

    // ========================================================================
    // ATK-PV-IMMUNE-CHANNEL: deprecated #[immune(requires=)] substrate gap
    // must not be masked by a code witness. (forward/immune-channel-gate-missing-from-adr029-amd1)
    //
    // Mirrors atk_pv_requires_masked_by_code_witness but exercises the IMMUNE
    // channel: an #[immune(requires=)] whose predicate failed (→ immune_audit
    // with witness_tier=None + evaluated_predicate=Some) alongside a code witness.
    // ADR-029 Amendment 1 §Channel-generality extends the gate to the immune channel.
    // ========================================================================

    #[test]
    fn atk_pv_immune_channel_substrate_gap_not_masked_by_code_witness() {
        // Construct a scan report with a presents-site defended by both:
        //   (a) a code witness (#[defended_by])  — grants Reachability
        //   (b) an #[immune] immunity with a failing requires= predicate
        //       → this produces an ImmunityAudit whose predicate evaluated and failed
        //
        // The presents-site and the immunity address the same antigen class. Under the
        // pre-fix implementation, best_tier=Some(Reachability) from the code witness
        // would send the verdict down the Defended arm, masking the immune channel gap.
        //
        // Post-fix: immune_audit.is_some_and(immune_audit_is_substrate_gap) gates the
        // verdict to SubstrateGap regardless of the code witness tier.
        let pred_json = r#"{"Signers":{"required":["alice"],"roles":{},"against":"Current","signature_allow":[],"signature_prefer":null}}"#;

        let mut report = ScanReport::default();
        // Site presenting the failure class.
        let site = presents_site("ImmuneChannelClass", "src/lib.rs", 20);
        report.presentations.push(site);
        // Code witness — grants Reachability on the immune-verdict computation.
        report.defenses.push(defended_by_witness(
            "ImmuneChannelClass",
            "src/tests.rs",
            15,
        ));
        // Deprecated #[immune(requires=)] immunity with a failing predicate
        // (no sidecar under "." → DisciplineSidecarMissing/failed → None tier).
        // item_target must match the presents-site's Unknown{line:20} for immune_audit
        // lookup to find this entry (compute_presentation_verdicts matches on
        // antigen_type + file + item_target).
        let imm = crate::scan::Immunity {
            antigen_type: "ImmuneChannelClass".to_string(),
            witness: String::new(),
            requires_predicate: Some(pred_json.to_string()),
            file: std::path::PathBuf::from("src/lib.rs"),
            line: 20,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Unknown { line: 20 },
            canonical_path: None,
            structural_fingerprint: String::new(),
        };
        report.immunities.push(imm);

        // audit() evaluates the immunity against "." — no sidecar exists, so the
        // substrate predicate fails → immune_audit will have witness_tier=None +
        // evaluated_predicate=Some → immune_audit_is_substrate_gap returns true.
        // ADR-029 Amendment 1 §Channel-generality: the gate fires, SubstrateGap is emitted.
        let out = audit(&report, Path::new("."));
        assert_eq!(out.presentation_verdicts.len(), 1);
        let v = &out.presentation_verdicts[0];

        assert_eq!(
            v.verdict,
            ImmuneVerdict::SubstrateGap,
            "ATK-PV-IMMUNE-CHANNEL: a failing #[immune(requires=)] (deprecated channel) \
             must surface SubstrateGap even when a code witness exists. The deprecated \
             substrate claim is broken; a code witness in a different channel does not \
             resolve it. ADR-029 Amendment 1 §Channel-generality covers this case. \
             verdict: {:?}",
            v.verdict
        );
    }

    // ========================================================================
    // ADR-023: #[orient] until-date observed (forward/time-bound-claim-staleness)
    //
    // Orient REQUIRES `until`; the audit must OBSERVE it. Before this fix, the
    // Orient arm unconditionally emitted OrientActive, ignoring the deadline —
    // a deferred defense whose horizon arrived stayed silently green.
    // ========================================================================

    fn orient_decl(until: &str) -> crate::scan::DeferredDefense {
        crate::scan::DeferredDefense {
            kind: crate::scan::DeferredDefenseKind::Orient,
            antigen_type: Some("SomeClass".to_string()),
            text: String::new(),
            until: Some(until.to_string()),
            expected_co_stimulation: None,
            signed_by: None,
            see: Vec::new(),
            since: None,
            duration_cap: None,
            file: PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("t".to_string()),
        }
    }

    #[test]
    fn orient_future_until_is_active() {
        let mut report = ScanReport::default();
        report.deferred_defenses.push(orient_decl("2999-12-31"));
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(out.audits.len(), 1);
        assert_eq!(out.audits[0].hint, AuditHint::OrientActive);
        assert_eq!(out.active_count, 1);
        assert_eq!(out.expired_count, 0);
    }

    #[test]
    fn orient_past_until_escalates_to_action_required() {
        // The orientation horizon passed without resolution → the audit observes
        // it and escalates, rather than perpetually reporting OrientActive.
        let mut report = ScanReport::default();
        report.deferred_defenses.push(orient_decl("2000-01-01"));
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(out.audits.len(), 1);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientPendingActionRequired,
            "a past orient until-date must escalate to OrientPendingActionRequired, \
             not stay OrientActive (ADR-023: orient observes its required deadline)"
        );
        assert_eq!(out.expired_count, 1);
        assert_eq!(out.active_count, 0);
    }

    // ========================================================================
    // ATK — orient degenerate inputs (adversarial probe, 2026-05-27)
    //
    // The bf60e5d fix correctly branches on `until >= today`, but three edge
    // paths in evaluate_deferred_defense_hint need adversarial coverage:
    //
    //   (a) until = None  →  parse_iso_date("") → None → OrientActive (grace path)
    //   (b) until = Some("not-a-date")  →  parse_iso_date → None → OrientActive
    //   (c) until = Some("9999-99-99")  →  parse_iso_date → None → OrientActive
    //   (d) two orient decls on same item, one past + one future → both evaluated
    //       independently; counts must both be individually correct
    //
    // (a) is the SILENT-FOREVER-GREEN path: a hand-built DeferredDefense with
    // until=None never escalates. The comment says this is intentional for
    // legacy records, but it means any record that escapes the macro parse-gate
    // (fuzz, JSON injection, or future code path that constructs DeferredDefense
    // directly) gets permanent OrientActive with no escalation. This test
    // documents the behavior so any future change to the None arm is visible.
    // ========================================================================

    fn orient_decl_no_until() -> crate::scan::DeferredDefense {
        crate::scan::DeferredDefense {
            kind: crate::scan::DeferredDefenseKind::Orient,
            antigen_type: Some("SomeClass".to_string()),
            text: String::new(),
            until: None, // deliberately absent — legacy/hand-built record
            expected_co_stimulation: None,
            signed_by: None,
            see: Vec::new(),
            since: None,
            duration_cap: None,
            file: PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: crate::scan::ItemTarget::Fn("t".to_string()),
        }
    }

    #[test]
    fn atk_orient_none_until_is_silently_active_forever() {
        // ATK-orient(a): until=None → grace path → OrientActive regardless of
        // how long ago the record was created. This is the silent-forever-green
        // failure mode for hand-built or fuzz-generated records.
        //
        // BEHAVIOR IS INTENTIONAL per the comment in evaluate_deferred_defense_hint,
        // but documenting it as a test ensures any future change to the None arm
        // is immediately visible as a failing test.
        let mut report = ScanReport::default();
        report.deferred_defenses.push(orient_decl_no_until());
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(out.audits.len(), 1);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientActive,
            "ATK-orient(a): orient with until=None must land in OrientActive (grace path for \
             hand-built/legacy records). If this changes, the None arm's escalation logic must \
             be deliberately designed, not accidental."
        );
        assert_eq!(
            out.active_count, 1,
            "ATK-orient(a): None-until orient counts as active"
        );
        assert_eq!(out.expired_count, 0);
    }

    #[test]
    fn atk_orient_invalid_date_string_escalates_not_silently_active() {
        // FIXED (findings/orient-unparseable-until-silent-green): until="not-a-date"
        // is a PRESENT-but-unparseable deadline — the author intended a deadline
        // but it resolves to nothing. The audit now escalates to
        // OrientPendingActionRequired rather than collapsing into the absent-date
        // grace path (which would silently grant permanent OrientActive).
        let mut report = ScanReport::default();
        let mut decl = orient_decl("not-a-date");
        decl.until = Some("not-a-date".to_string());
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(out.audits.len(), 1);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientPendingActionRequired,
            "a present-but-unparseable orient `until` must escalate (the author \
             intended a deadline; a broken one is unresolved, not a grace)"
        );
        assert_eq!(out.active_count, 0);
    }

    #[test]
    fn atk_orient_slash_date_format_typo_escalates_not_silently_active() {
        // FIXED: "2099/01/01" (slash format) looks like a future date to a human
        // but parse_iso_date rejects it. It's a present-but-unparseable deadline
        // → escalates to OrientPendingActionRequired, not silent OrientActive. The
        // typo trap (future-looking but unparseable behaving like no-deadline) is
        // closed: present-but-broken now reads loudly as action-required.
        let mut report = ScanReport::default();
        let mut decl = orient_decl("2099-01-01"); // valid baseline
        decl.until = Some("2099/01/01".to_string()); // slash typo — fails parse
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientPendingActionRequired,
            "a slash-format (unparseable) orient `until` must escalate to \
             action-required, not silently fall to OrientActive"
        );
    }

    #[test]
    fn atk_orient_two_decls_one_past_one_future_counted_independently() {
        // ATK-orient(d): two orient decls on the same item — one past (expired),
        // one future (active). Both must be evaluated independently.
        // Degenerate input: what if someone mistakenly adds two orients? The audit
        // must not short-circuit on the first match.
        let mut report = ScanReport::default();
        report.deferred_defenses.push(orient_decl("2000-01-01")); // expired
        report.deferred_defenses.push(orient_decl("2999-12-31")); // active
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits.len(),
            2,
            "ATK-orient(d): both decls must be evaluated"
        );
        assert_eq!(out.active_count, 1, "ATK-orient(d): one active orient");
        assert_eq!(out.expired_count, 1, "ATK-orient(d): one expired orient");
        // Verify which is which (order preserved from push order)
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientPendingActionRequired,
            "ATK-orient(d): past orient must escalate"
        );
        assert_eq!(
            out.audits[1].hint,
            AuditHint::OrientActive,
            "ATK-orient(d): future orient must stay active"
        );
    }

    #[test]
    fn atk_orient_extreme_past_1970_escalates() {
        // ATK-orient(e): Unix epoch — the most extreme past date representable
        // in common date libraries. Must escalate, not crash.
        let mut report = ScanReport::default();
        report.deferred_defenses.push(orient_decl("1970-01-01"));
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientPendingActionRequired,
            "ATK-orient(e): epoch date (1970-01-01) must escalate to OrientPendingActionRequired"
        );
    }

    #[test]
    fn atk_orient_empty_string_until_is_silently_active() {
        // ATK-orient(f): until=Some("") — an explicitly empty string. This is
        // different from until=None but falls through the same parse_iso_date("")
        // → None → OrientActive path. The macro should reject this, but a
        // hand-built record could have it.
        //
        // NOTE: orient_decl("") produces this — verify the existing helper and
        // the grace-path behavior are consistent.
        let mut report = ScanReport::default();
        report.deferred_defenses.push(orient_decl(""));
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::OrientActive,
            "ATK-orient(f): orient with until=Some(\"\") (empty string) lands in OrientActive \
             via the same None-parse grace path. SILENT GAP: an empty string looks like \
             'field was set' but behaves like 'field was absent'."
        );
        assert_eq!(out.active_count, 1);
    }

    // ========================================================================
    // ATK-IMMUNOSUPPRESS-DURATION-CAP-UNREACHABLE
    //
    // ImmunosuppressDurationCapExceeded (AuditHint, line 308) is declared but
    // cannot be emitted by evaluate_deferred_defense_hint().
    //
    // ROOT CAUSE (three-layer gap):
    //   1. scan.rs parses duration_cap correctly from #[immunosuppress(duration_cap=Nd)]
    //      into ScanImmunosuppressArgs.duration_cap: Option<u64>.
    //   2. scan.rs stores it as a string tag in DeferredDefense.see[]:
    //      see.push(format!("duration_cap:{cap}d"))
    //      — NOT as a typed field on DeferredDefense.
    //   3. audit.rs evaluate_deferred_defense_hint() Immunosuppress arm (lines
    //      1059-1065) reads only decl.until and never parses decl.see[] for
    //      "duration_cap:Nd" entries. The hint variant therefore has zero
    //      emission sites.
    //
    // SECONDARY GAP: even if an audit arm tried to compute duration, it cannot
    // — `since` date is also stored in see[] as "since:DATE" rather than as a
    // typed field. Computing (today - since).num_days() > duration_cap requires
    // parsing two strings from see[], neither of which has a typed API.
    //
    // FIX PATH (not in this test; tests document the gap):
    //   - Add `duration_cap: Option<u64>` to DeferredDefense struct.
    //   - Add `since: Option<String>` (or date type) to DeferredDefense.
    //   - Populate both during the immunosuppress scan push.
    //   - In evaluate_deferred_defense_hint Immunosuppress arm: parse
    //     since_date, compute age_days, compare to cap; emit
    //     ImmunosuppressDurationCapExceeded if exceeded.
    //
    // This test is a DOCUMENTATION LOCK: it will remain a documentation test
    // (assert_eq! confirming current behavior) until the fix lands, at which
    // point the assertion must be inverted.
    // ========================================================================

    fn immunosuppress_decl_with_duration_cap(
        duration_cap_days: u64,
        since: &str,
    ) -> crate::scan::DeferredDefense {
        use crate::scan::{DeferredDefenseKind, ItemTarget};
        use std::path::PathBuf;
        crate::scan::DeferredDefense {
            kind: DeferredDefenseKind::Immunosuppress,
            antigen_type: None,
            text: "test rationale".to_string(),
            until: None,
            expected_co_stimulation: None,
            signed_by: None,
            see: Vec::new(),
            // ADR-023 fix: scan now stores since + duration_cap as TYPED fields
            // (was `see[]` string tags), so the audit can compute elapsed-vs-cap.
            since: Some(since.to_string()),
            duration_cap: Some(duration_cap_days),
            file: PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: ItemTarget::Fn("suppress_me".to_string()),
        }
    }

    #[test]
    fn atk_immunosuppress_duration_cap_exceeded_is_emitted() {
        // ADR-023 fix (since + duration_cap now typed fields, not see[] tags):
        // #[immunosuppress(since="2020-01-01", duration_cap=30d)] — 6+ years
        // elapsed; the cap is dramatically exceeded → the audit now EMITS
        // ImmunosuppressDurationCapExceeded (was unreachable while the data lived
        // only as unparsed see[] string tags). The hint also tallies as stale.
        let decl = immunosuppress_decl_with_duration_cap(30, "2020-01-01");
        let mut report = ScanReport::default();
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);

        assert_eq!(
            out.audits[0].hint,
            AuditHint::ImmunosuppressDurationCapExceeded,
            "an immunosuppress past its since+duration_cap must emit \
            ImmunosuppressDurationCapExceeded (the cap is now enforceable at \
            audit time via typed since/duration_cap fields)"
        );
        assert_eq!(
            out.stale_count, 1,
            "a cap-exceeded immunosuppress tallies as stale (overstayed its cap)"
        );
    }

    #[test]
    fn atk_immunosuppress_duration_cap_within_limit_is_active() {
        // ADR-023 fix: #[immunosuppress(since="2099-01-01", duration_cap=30d)] —
        // `since` is far-future, so elapsed-days is negative and the cap is NOT
        // exceeded → ImmunosuppressActive. Paired with the exceeded test above,
        // this confirms the audit now DISCRIMINATES exceeded from within-limit
        // (it didn't before — both were silently Active because see[] was never
        // parsed).
        let decl = immunosuppress_decl_with_duration_cap(30, "2099-01-01");
        let mut report = ScanReport::default();
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);

        assert_eq!(
            out.audits[0].hint,
            AuditHint::ImmunosuppressActive,
            "a within-cap immunosuppress (since far-future, cap not exceeded) \
            stays Active — the discrimination exceeded-vs-within-limit now works"
        );
    }

    #[test]
    fn atk_immunosuppress_malformed_since_silently_skips_cap_check() {
        // ATK-IMMUNOSUPPRESS-MALFORMED-SINCE (2026-05-27, adversarial):
        //
        // audit.rs:1072 uses `if let Some(since_date) = parse_iso_date(since)`.
        // If `since` is malformed (not ISO 8601), parse_iso_date returns None
        // and the entire cap-exceeded check is skipped silently. The suppression
        // then falls through to the until-date check and returns ImmunosuppressActive
        // with no diagnostic.
        //
        // SAME PATTERN as ATK-orient-unparseable-until (findings/orient-unparseable-
        // until-silent-green): the None arm of the parse result collapses "absent"
        // and "malformed" into identical silent behavior. A typo in since= (e.g.,
        // "2026-5-27" instead of "2026-05-27") silently defeats the duration_cap
        // enforcement, granting the suppression infinite duration.
        //
        // Fix direction (parallel to the orient fix): split the None arm --
        //   - since = None: skip cap check (intentional; cap is optional without since)
        //   - since = Some(s) where parse_iso_date(s) = None: emit parse failure
        //     diagnostic rather than silently treating as absent.
        //
        // This test DOCUMENTS the current behavior as a regression anchor.
        let mut decl = immunosuppress_decl_with_duration_cap(1, "2020-01-01");
        // Override with malformed since after construction.
        decl.since = Some("not-a-date".to_string());

        let mut report = ScanReport::default();
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 9999);

        // CURRENT BROKEN BEHAVIOR: malformed since skips the cap check entirely.
        // The suppression is Active despite since being unparseable and cap=1 day.
        // No diagnostic for the malformed since string.
        // CURRENT BROKEN BEHAVIOR: malformed since skips the cap check entirely.
        // The suppression is Active despite since being unparseable and cap=1 day.
        // No separate parse_failures surface exists in DeferredDefenseAuditReport;
        // the only observable is the Active hint (silent skip leaves no trace).
        assert_eq!(
            out.audits[0].hint,
            AuditHint::ImmunosuppressActive,
            "ATK-IMMUNOSUPPRESS-MALFORMED-SINCE (documented gap): malformed since= \
            silently skips the duration_cap check, yielding ImmunosuppressActive. \
            A typo in since= grants the suppression infinite duration -- the cap \
            enforcement is completely invisible. Fix: split the None arm -- \
            since=Some(bad) should emit a parse failure diagnostic instead of \
            silently treating since as absent."
        );
        // The stale_count is 0 (the suppression appears active from audit's perspective,
        // not stale). This confirms the cap-exceeded path was never reached.
        assert_eq!(
            out.stale_count, 0,
            "ATK-IMMUNOSUPPRESS-MALFORMED-SINCE: stale_count is 0 -- the cap-exceeded \
            path was never reached because since parse failed silently"
        );
    }

    // ========================================================================
    // ATK-DEFERRED-UNTIL-1/2/3: anergy, immunosuppress, poxparty silently
    // treat a present-but-malformed `until` as "active forever" (the Orient
    // arm was fixed to distinguish None vs Some(invalid), but the other three
    // still use `unwrap_or("")` which makes None and Some("bad") identical).
    //
    // Concrete failure:
    //   #[anergy(until = "2026-13-01")]   — month 13, invalid date
    //   evaluate_deferred_defense_hint:   unwrap_or("") → parse_iso_date("2026-13-01")
    //                                     → None → AnergyActive
    //
    // The developer INTENDED an expiry deadline. A typo silently grants the
    // anergy (or immunosuppress or poxparty) permanent Active status. No
    // AnergyStale, no AnergyCostimulationNotArrived, no diagnostic at all.
    //
    // Fix direction (parallel to Orient fix at evaluate_deferred_defense_hint
    // lines 1176-1191): match on decl.until.as_deref() FIRST, then parse.
    //   None | Some("") → Active (legacy grace path)
    //   Some(s) → match parse_iso_date(s):
    //     Some(date) if date >= today → Active
    //     _ → Expired/Stale (a present-but-broken until is unresolved, not a grace)
    //
    // These tests DOCUMENT the current broken behavior (each asserts the wrong
    // Active outcome). They will FAIL once the fix lands — update them to the
    // correct escalation hints when that happens.
    // ========================================================================

    fn anergy_decl_with_until(until: &str) -> crate::scan::DeferredDefense {
        use crate::scan::{DeferredDefenseKind, ItemTarget};
        crate::scan::DeferredDefense {
            kind: DeferredDefenseKind::Anergy,
            antigen_type: Some("SomeClass".to_string()),
            text: "test reason".to_string(),
            until: Some(until.to_string()),
            expected_co_stimulation: None,
            signed_by: None,
            see: Vec::new(),
            since: None,
            duration_cap: None,
            file: std::path::PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: ItemTarget::Fn("deferred_fn".to_string()),
        }
    }

    fn immunosuppress_decl_with_until(until: &str) -> crate::scan::DeferredDefense {
        use crate::scan::{DeferredDefenseKind, ItemTarget};
        crate::scan::DeferredDefense {
            kind: DeferredDefenseKind::Immunosuppress,
            antigen_type: Some("SomeClass".to_string()),
            text: "test rationale".to_string(),
            until: Some(until.to_string()),
            expected_co_stimulation: None,
            signed_by: None,
            see: Vec::new(),
            since: None,
            duration_cap: None,
            file: std::path::PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: ItemTarget::Fn("suppressed_fn".to_string()),
        }
    }

    fn poxparty_decl_with_until(until: &str) -> crate::scan::DeferredDefense {
        use crate::scan::{DeferredDefenseKind, ItemTarget};
        crate::scan::DeferredDefense {
            kind: DeferredDefenseKind::Poxparty,
            antigen_type: Some("SomeClass".to_string()),
            text: "UserInput".to_string(),
            until: Some(until.to_string()),
            expected_co_stimulation: None,
            signed_by: None,
            see: Vec::new(),
            since: None,
            duration_cap: None,
            file: std::path::PathBuf::from("src/lib.rs"),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: ItemTarget::Fn("pox_fn".to_string()),
        }
    }

    // ATK-DEFERRED-UNTIL-1: anergy with a present-but-malformed `until` must
    // ESCALATE, not silently stay Active. A typo'd deadline ("not-a-date") is an
    // intended-but-broken bound that resolves to nothing → AnergyCostimulationNotArrived
    // (the unresolved-co-stimulation escalation), tallied as expired. Before the
    // Orient-style split this collapsed to AnergyActive via `unwrap_or("")`.
    #[test]
    fn atk_deferred_until_1_anergy_malformed_until_escalates() {
        let decl = anergy_decl_with_until("not-a-date");
        let mut report = ScanReport::default();
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::AnergyCostimulationNotArrived,
            "ATK-DEFERRED-UNTIL-1: anergy with until=Some('not-a-date') must escalate \
             to AnergyCostimulationNotArrived (present-but-broken deadline = unresolved), \
             not silently land in AnergyActive. The author intended a deadline; a typo \
             must not grant permanent active status."
        );
        assert_eq!(
            out.active_count, 0,
            "ATK-DEFERRED-UNTIL-1: a malformed-until anergy must NOT count as active"
        );
        assert_eq!(out.expired_count, 1);
        assert_eq!(out.stale_count, 0);
    }

    // ATK-DEFERRED-UNTIL-2: immunosuppress with a present-but-malformed `until` must
    // ESCALATE to ImmunosuppressExpired, not silently stay Active. "2026/01/01" (slash
    // format) looks like a past date to a human but fails ISO parse; the developer
    // intended an expiry, so the suppression has outlived its declared bound.
    #[test]
    fn atk_deferred_until_2_immunosuppress_malformed_until_escalates() {
        let decl = immunosuppress_decl_with_until("2026/01/01");
        let mut report = ScanReport::default();
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::ImmunosuppressExpired,
            "ATK-DEFERRED-UNTIL-2: immunosuppress with until=Some('2026/01/01') \
             (present-but-unparseable) must escalate to ImmunosuppressExpired, not \
             silently stay Active. A suppression intended to expire must not run forever."
        );
        assert_eq!(out.active_count, 0);
        assert_eq!(out.expired_count, 1);
    }

    // ATK-DEFERRED-UNTIL-3: poxparty with a present-but-malformed `until` must
    // ESCALATE to PoxpartyOutcomePending, not silently stay Active. "soon" is not a
    // date at all — an intended bound that resolves to nothing → the outcome is due.
    #[test]
    fn atk_deferred_until_3_poxparty_malformed_until_escalates() {
        let decl = poxparty_decl_with_until("soon"); // not a date at all
        let mut report = ScanReport::default();
        report.deferred_defenses.push(decl);
        let out = audit_deferred_defenses(&report, 30);
        assert_eq!(
            out.audits[0].hint,
            AuditHint::PoxpartyOutcomePending,
            "ATK-DEFERRED-UNTIL-3: poxparty with until=Some('soon') must escalate to \
             PoxpartyOutcomePending, not silently stay Active. An intended-but-broken \
             expiry must surface as outcome-pending, not permanent green."
        );
        assert_eq!(out.active_count, 0);
        assert_eq!(out.expired_count, 1);
    }

    // ========================================================================
    // Lineage-fidelity audit (DescendedFromFingerprintDivergence) — ADVISORY
    // ========================================================================

    fn antigen_with_fp(type_name: &str, fingerprint: &str) -> crate::scan::AntigenDeclaration {
        crate::scan::AntigenDeclaration {
            fingerprint: Some(fingerprint.to_string()),
            ..antigen_decl(type_name, Vec::new())
        }
    }

    fn lineage_edge(child: &str, parent: &str) -> crate::scan::LineageEdge {
        crate::scan::LineageEdge {
            child: child.to_string(),
            parent: parent.to_string(),
            file: PathBuf::from("src/lib.rs"),
            line: 1,
            parent_canonical_path: None,
            child_canonical_path: None,
        }
    }

    #[test]
    fn lineage_fidelity_flags_item_kind_divergence() {
        // Parent pins `item = struct`; child pins `item = enum` — disjoint kinds,
        // so the child is NOT a refinement. Advisory hint fires.
        let mut report = ScanReport::default();
        report
            .antigens
            .push(antigen_with_fp("Parent", "item = struct"));
        report
            .antigens
            .push(antigen_with_fp("Child", "item = enum"));
        report.lineage_edges.push(lineage_edge("Child", "Parent"));

        let out = audit_lineage_fidelity(&report);
        assert_eq!(out.divergences.len(), 1);
        assert_eq!(
            out.divergences[0].hint,
            AuditHint::DescendedFromFingerprintDivergence
        );
        assert!(out.divergences[0].detail.contains("item"));
    }

    #[test]
    fn lineage_fidelity_flags_missing_parent_doc_substring() {
        // Parent requires doc_contains("error"); child's doc_contains("panic")
        // does NOT include "error" → child can match where parent can't → not a
        // refinement.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp(
            "P",
            r#"item = struct, doc_contains("error")"#,
        ));
        report.antigens.push(antigen_with_fp(
            "C",
            r#"item = struct, doc_contains("panic")"#,
        ));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);
        assert_eq!(out.divergences.len(), 1);
        assert!(out.divergences[0].detail.contains("doc_contains"));
    }

    #[test]
    fn lineage_fidelity_clean_when_child_refines_parent() {
        // Same item-kind + child doc_contains SUPERSTRING of parent's needle
        // (child requires "parse error" ⊇ parent's "error") → child matches a
        // subset of parent → a genuine refinement → no advisory.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp(
            "P",
            r#"item = struct, doc_contains("error")"#,
        ));
        report.antigens.push(antigen_with_fp(
            "C",
            r#"item = struct, doc_contains("parse error")"#,
        ));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);
        assert!(
            out.divergences.is_empty(),
            "a genuine refinement (same kind + superstring doc) must not flag; got: {:?}",
            out.divergences
        );
    }

    #[test]
    fn lineage_fidelity_silent_when_a_fingerprint_is_absent() {
        // ADR-009 Amendment 1: a verify-only antigen has no fingerprint.
        // Refinement is undefined → the advisory stays silent (no false positive).
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp("P", "item = struct"));
        report.antigens.push(antigen_decl("C", Vec::new())); // fingerprint: None
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);
        assert!(
            out.divergences.is_empty(),
            "an absent (verify-only) fingerprint must not produce a divergence advisory"
        );
    }

    // ATK-LF-1: parent item-kind nested inside all_of — child_top_item_kind() misses it
    //
    // `child_top_item_kind()` iterates only `Constraint::Item` at the TOP LEVEL of
    // the fingerprint's constraints Vec. But a fingerprint like
    // `all_of(item = struct, doc_contains("error"))` places `item = struct` inside
    // an `AllOf` constraint, not at the top level.
    //
    // `child_top_item_kind` returns `None` for both parent and child → the item-kind
    // check is skipped entirely (the `if let (Some(pk), Some(ck))` guard fails).
    //
    // A child with `all_of(item = enum, doc_contains("error"))` does NOT refine a
    // parent with `all_of(item = struct, doc_contains("error"))` — enum and struct
    // are disjoint item-kinds. But the advisory stays SILENT, producing a false
    // negative.
    //
    // Contrast: `antigen_fingerprint::Fingerprint::node_kind()` DOES descend into
    // `AllOf` via `Constraint::node_kind_hint()` (fingerprint/src/lib.rs:399-403).
    // `child_top_item_kind` diverges from this behavior, creating a coverage gap.
    //
    // Fix direction: `child_top_item_kind` should use the same `node_kind_hint()`
    // traversal as `Fingerprint::node_kind()`, or delegate to it directly.
    //
    // This test pins CURRENT BROKEN BEHAVIOR (no divergence emitted for nested
    // item-kind mismatch). When the fix lands, the assertion should invert:
    // expect divergences.len() == 1 with DescendedFromFingerprintDivergence.
    #[test]
    fn atk_lf_1_item_kind_nested_in_all_of_silently_bypasses_divergence_check() {
        // Parent: all_of(item = struct, doc_contains("error"))
        // Child:  all_of(item = enum,   doc_contains("error"))
        // The item-kind divergence (struct vs enum) is NOT at the top level —
        // it's nested inside an all_of. child_top_item_kind returns None for both,
        // skipping the item-kind check. Advisory stays silent — false negative.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp(
            "P",
            r#"all_of([item = struct, doc_contains("error")])"#,
        ));
        report.antigens.push(antigen_with_fp(
            "C",
            r#"all_of([item = enum, doc_contains("error")])"#,
        ));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);

        // FIXED: fingerprint_nonrefinement_reason now delegates to
        // Fingerprint::node_kind(), which descends into AllOf via
        // Constraint::node_kind_hint. parent.node_kind() returns
        // Some(ItemKind::Struct), child.node_kind() returns Some(ItemKind::Enum),
        // disjoint kinds → advisory fires.
        assert_eq!(
            out.divergences.len(),
            1,
            "ATK-LF-1 (FIXED): parent `all_of(item=struct, ...)` and child `all_of(item=enum, ...)` \
             must fire DescendedFromFingerprintDivergence — node_kind() descends into AllOf and \
             surfaces the disjoint item-kinds. A zero-length result means the item-kind check no \
             longer delegates to node_kind(). Got: {:?}",
            out.divergences
        );
        assert_eq!(
            out.divergences[0].hint,
            AuditHint::DescendedFromFingerprintDivergence
        );
    }

    // ATK-LF-2: doc_contains nested in all_of — parent requirement missed
    //
    // The doc_contains check in `fingerprint_nonrefinement_reason` iterates
    // `parent.constraints` for top-level `Constraint::DocContains`. If the parent
    // has `all_of(item = struct, doc_contains("error"))`, the `doc_contains("error")`
    // is nested inside AllOf — not a top-level `Constraint::DocContains`.
    //
    // The loop at audit.rs:3344 `for c in &parent.constraints` iterates the
    // outer Vec, finding only `AllOf(...)` — not the nested `DocContains`. The
    // parent's doc-substring requirement is missed entirely.
    //
    // A child with `item = struct` (no doc_contains at all) is NOT a refinement
    // of `all_of(item = struct, doc_contains("error"))`. But the advisory stays
    // SILENT — false negative.
    //
    // Fix direction: the doc_contains iteration should also look inside AllOf
    // constraints to collect all required doc substrings from parent.
    //
    // This test pins CURRENT BROKEN BEHAVIOR. Invert when fix lands.
    #[test]
    fn atk_lf_2_parent_doc_contains_nested_in_all_of_silently_bypasses_divergence_check() {
        // Parent: all_of(item = struct, doc_contains("error"))
        // Child:  item = struct  (no doc_contains requirement — broader match set)
        // The child can match structs WITHOUT "error" in their doc → NOT a refinement.
        // But the parent's doc_contains is nested inside all_of → missed.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp(
            "P",
            r#"all_of([item = struct, doc_contains("error")])"#,
        ));
        report.antigens.push(antigen_with_fp("C", "item = struct"));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);

        // FIXED: collect_doc_contains_allof_only descends into AllOf children,
        // so parent's nested doc_contains("error") is collected as a required
        // substring. Child has no doc_contains anywhere → cannot cover the
        // parent's requirement → advisory fires (the child can match structs
        // without "error" in their doc, so it is not a refinement).
        assert_eq!(
            out.divergences.len(),
            1,
            "ATK-LF-2 (FIXED): parent `all_of(item=struct, doc_contains('error'))` requires \
             'error' in the doc — child `item = struct` (no doc_contains) does not cover it. \
             collect_doc_contains_allof_only must descend into AllOf and surface the nested \
             requirement. A zero-length result means the AllOf descent was removed. Got: {:?}",
            out.divergences
        );
        assert_eq!(
            out.divergences[0].hint,
            AuditHint::DescendedFromFingerprintDivergence
        );
    }

    // ATK-LF-3: fingerprint index keyed by bare type_name -- cross-crate name collision.
    //
    // audit_lineage_fidelity builds HashMap<&str, Fingerprint> by bare type_name
    // (audit.rs:3263). Two antigens named "Foo" from different crates cause collect()
    // to silently deduplicate. The lineage lookup fingerprints.get("Foo") returns an
    // ARBITRARY entry (non-deterministic HashMap order).
    //
    // Failure mode: Bar refines crate A's Foo (struct->struct, valid, no advisory).
    // If crate B's Foo (item=fn) wins the race, struct vs fn fires spuriously.
    //
    // Fix: key by (type_name, canonical_path) tuple (ADR-017 discipline).
    #[test]
    fn atk_lf_3_bare_type_name_index_cross_crate_collision_non_deterministic_advisory() {
        let mut report = ScanReport::default();
        // Crate A Foo: item = struct (real parent)
        let mut foo_a = antigen_with_fp("Foo", "item = struct");
        foo_a.canonical_path = Some("crate-a@1.0".to_string());
        report.antigens.push(foo_a);
        // Crate B Foo: item = fn (collision -- same bare name, different crate)
        let mut foo_b = antigen_with_fp("Foo", "item = fn");
        foo_b.canonical_path = Some("crate-b@2.0".to_string());
        report.antigens.push(foo_b);
        // Child: Bar with item = struct -- valid refinement of crate A Foo
        let mut bar = antigen_with_fp("Bar", "item = struct");
        bar.canonical_path = Some("crate-a@1.0".to_string());
        report.antigens.push(bar);
        // Edge: Bar descended_from Foo, both in crate-a
        let mut edge = lineage_edge("Bar", "Foo");
        edge.child_canonical_path = Some("crate-a@1.0".to_string());
        edge.parent_canonical_path = Some("crate-a@1.0".to_string());
        report.lineage_edges.push(edge);

        let out = audit_lineage_fidelity(&report);
        // FIXED: the fingerprint index is keyed by (type_name, canonical_path),
        // so Bar's parent edge (Foo @ crate-a) resolves DETERMINISTICALLY to
        // crate-A's `item = struct` Foo — a valid refinement of Bar's
        // `item = struct` → zero divergences. crate-B's `item = fn` Foo
        // (@ crate-b) is a different key and is never confused for the parent.
        // Pre-fix this was 0-or-1 depending on HashMap iteration order; now it
        // is always 0. If this regresses (len == 1): the index key dropped
        // canonical_path and the wrong-crate Foo collided back in.
        assert_eq!(
            out.divergences.len(),
            0,
            "ATK-LF-3 (FIXED): (type_name, canonical_path)-keyed index resolves Bar's parent \
             to crate-A's struct Foo deterministically — a valid refinement, no divergence. \
             A non-zero result means the cross-crate Foo collided back in. Got: {:?}",
            out.divergences
        );
    }

    // ATK-LF-4: naive fix for ATK-LF-2 would false-positive on any_of-nested doc_contains.
    //
    // The proposed fix for ATK-LF-2 (collect doc_contains from nested constraints inside
    // AllOf) carries a hazard: a naive implementation that collects from ANY nested combinator
    // — including AnyOf — would require the child to cover doc_contains strings from OR-arms
    // that the child is NOT required to satisfy.
    //
    // CONCRETE CASE:
    //   Parent: any_of([doc_contains("A"), doc_contains("B")])
    //   Child:  doc_contains("A")
    //
    // The child IS a valid refinement: everything that matches the child (docs containing "A")
    // also satisfies the parent (docs contain "A" OR "B"). Child.matches ⊆ parent.matches.
    //
    // But a naive "collect all doc_contains from any nested combinator" fix would see the parent
    // has "A" and "B" requirements (from AnyOf arms), demand the child cover both, and fire
    // DescendedFromFingerprintDivergence spuriously — a false positive.
    //
    // The CORRECT fix for ATK-LF-2 is ALL-OF-ONLY descent: collect doc_contains from AllOf
    // children only (every AllOf child must be satisfied, so the parent requirement is real).
    // AnyOf children are OR-branches — the parent is satisfied by ANY one; collecting all
    // would over-require the child. Not/leaf are irrelevant to doc_contains.
    //
    // This test CURRENTLY passes (no advisory fires — correct: child IS a refinement).
    // It guards against a future regression where the ATK-LF-2 fix naively descends into
    // AnyOf and fires a spurious advisory for this valid refinement.
    //
    // If this test FAILS after the ATK-LF-2 fix lands: the fix descended into AnyOf and
    // introduced a false-positive. The fix is too broad — restrict descent to AllOf only.
    #[test]
    fn atk_lf_4_any_of_nested_doc_contains_must_not_false_positive() {
        // Parent: any_of([doc_contains("A"), doc_contains("B")])
        //   → matches docs containing "A" OR "B"
        // Child:  doc_contains("A")
        //   → matches docs containing "A" (strict subset of parent — valid refinement)
        //
        // A naive fix that collects ALL doc_contains from nested combinators would see parent
        // requires "A" AND "B" (from the two any_of arms), demand the child cover both, and
        // fire a spurious DescendedFromFingerprintDivergence. The CORRECT behavior: silence.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp(
            "P",
            r#"any_of([doc_contains("A"), doc_contains("B")])"#,
        ));
        report
            .antigens
            .push(antigen_with_fp("C", r#"doc_contains("A")"#));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);

        // CORRECT BEHAVIOR NOW AND AFTER FIX: no divergence (child IS a valid refinement).
        // If this fires: the ATK-LF-2 fix descended into AnyOf and false-positived.
        // Restrict descent to AllOf only.
        assert_eq!(
            out.divergences.len(),
            0,
            "ATK-LF-4: a child with doc_contains('A') IS a valid refinement of a parent with \
             any_of([doc_contains('A'), doc_contains('B')]) — the child's match-set is a subset \
             of the parent's OR-union. No DescendedFromFingerprintDivergence should fire. \
             If this assertion FAILS after ATK-LF-2 fix landed: the fix naively descends into \
             AnyOf and requires the child to cover both arms — a false positive. Restrict \
             doc_contains collection to AllOf children only."
        );
    }

    // ATK-LF-6 (FIXED): child has no item kind — parent-item-kind broader,
    // not a refinement — advisory must fire.
    //
    // Parent: item = struct, doc_contains("error") — only matches structs.
    // Child:  doc_contains("error")                — no item kind, matches ALL items.
    // Child is STRICTLY BROADER in the item dimension; this is NOT a refinement.
    //
    // Unlike ATK-LF-5 (AnyOf over kinds — undecidable), this case IS decidable:
    // parent=Some(Struct) + child=None → child is unconditionally wider.
    // Fix: added a (Some(pk), None) arm to the item-kind match in
    // fingerprint_nonrefinement_reason that returns a divergence reason.
    #[test]
    fn atk_lf_6_child_no_item_kind_flags_as_non_refinement() {
        // Parent: item = struct, doc_contains("error")
        // Child:  doc_contains("error")  — no item kind, matches ALL item types
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp(
            "P",
            r#"item = struct, doc_contains("error")"#,
        ));
        // Child has only doc_contains, no item kind: wider than parent in item dimension.
        report
            .antigens
            .push(antigen_with_fp("C", r#"doc_contains("error")"#));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);

        // CORRECT: one divergence fires — parent has Some(Struct), child has None.
        // The (Some(pk), None) arm in fingerprint_nonrefinement_reason catches this.
        assert_eq!(
            out.divergences.len(),
            1,
            "ATK-LF-6: child `doc_contains('error')` with no item kind is wider \
             than parent `item=struct, doc_contains('error')` — not a refinement. \
             DescendedFromFingerprintDivergence must fire. parent=Some(Struct) + \
             child=None is unconditionally broader (decidable, unlike ATK-LF-5). \
             divergences: {:?}",
            out.divergences
        );
    }

    // ATK-LF-5: child has any_of over item-kinds — now flagged as a widening.
    //
    // CONCRETE CASE:
    //   Parent: item = struct
    //   Child:  any_of([item = struct, item = enum])   (a WIDENING, not a refinement)
    //
    // This is NOT a refinement (child.matches ⊃ parent.matches — child matches both structs
    // and enums, parent only matches structs). The advisory SHOULD fire here.
    //
    // node_kind() returns None for the child (AnyOf yields no single kind hint, lib.rs:407).
    // PREVIOUSLY this was a documented FALSE NEGATIVE: the item-kind check required
    // `(Some(pk), Some(ck))` so a `None` child silently skipped the check. The ATK-LF-6 fix
    // added a `(Some(pk), None)` arm — a parent with a definite kind and a child with NO
    // resolvable kind means the child matches a BROADER item set than the parent. That arm
    // closes this false negative too: an any_of-over-kinds child (whose node_kind is None) is
    // unconditionally broader in the item dimension and is correctly flagged.
    //
    // This is the conservative-yet-correct posture: `node_kind()=None` means "matches all
    // item kinds", which is genuinely wider than any single-kind parent. The only `None`
    // children are no-item-constraint (LF-6), any_of-over-kinds (here), and top-level `not` —
    // all genuinely broader in the item dimension. No refinement produces a `None` node_kind
    // against a `Some` parent (AllOf descends to find a pinned kind), so the arm does not
    // false-positive on real refinements.
    #[test]
    fn atk_lf_5_any_of_item_kind_widening_flagged_via_none_node_kind() {
        // Parent: item = struct  (matches only structs)
        // Child:  any_of([item = struct, item = enum])  (matches structs AND enums)
        // Child is WIDER than parent — NOT a refinement. any_of → node_kind None →
        // the (Some(pk), None) arm fires the widening divergence.
        let mut report = ScanReport::default();
        report.antigens.push(antigen_with_fp("P", "item = struct"));
        report.antigens.push(antigen_with_fp(
            "C",
            r"any_of([item = struct, item = enum])",
        ));
        report.lineage_edges.push(lineage_edge("C", "P"));

        let out = audit_lineage_fidelity(&report);

        // CORRECT (post ATK-LF-6 fix): one divergence fires — parent=Some(Struct),
        // child=None (any_of yields no kind hint) → child is unconditionally wider.
        // The (Some(pk), None) arm in fingerprint_nonrefinement_reason catches this,
        // closing the former known-limitation false negative.
        assert_eq!(
            out.divergences.len(),
            1,
            "ATK-LF-5: child `any_of([item=struct, item=enum])` is wider than parent \
             `item=struct` — not a refinement. node_kind() is None for the child (any_of \
             has no single kind hint), and the (Some(pk), None) arm flags this as a widening. \
             Previously a silent false negative; the ATK-LF-6 fix closes it. divergences: {:?}",
            out.divergences
        );
    }
}

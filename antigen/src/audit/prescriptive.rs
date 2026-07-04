//! Prescriptive Work-Orchestration audit (ADR-033) — "code IS the board".
//!
//! Extracted from the former monolithic `audit.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). A pure detector module: a fn of `&ScanReport`
//! returning its own report; no detector calls another (single-conductor
//! invariant, ADR-036). API-invisible: re-exported from the `audit` root via
//! `pub use`.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{FilesystemAuditContext, FrameState, SidecarLoad, WorkVerdict, load_sidecar};
use crate::scan::ScanReport;

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
/// [`UnreachedCause`](crate::audit::UnreachedCause) +
/// [`UnreachedCause::remedy`](crate::audit::UnreachedCause::remedy) for the
/// coverage audit. The
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
    /// [`UnreachedCause::remedy`](crate::audit::UnreachedCause::remedy)).
    #[must_use]
    pub const fn remedy(self) -> &'static str {
        match self {
            Self::UnknownWhoRef => {
                "scaffold + sign the .attest/<item>.json sidecar so the named \
                 who-ref's attestation is readable"
            },
            Self::MissingWorkStep => {
                "declare the missing who-step (filled_by / ordered_by / triaged_by \
                 / closure) — an empty work-need has nothing to attest"
            },
            Self::UnparseableFrame => {
                "fix the malformed frame date (due / until / runs_until / \
                 re_triage_due must be ISO-8601 YYYY-MM-DD)"
            },
            Self::UnresolvableRef => {
                "fix the dangling priority_order code-site reference (or, for a \
                 cross-crate target, await multi-crate Layer-2 resolution)"
            },
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
    use antigen_attestation::AuditHint as AH;
    use antigen_attestation::evaluate::evaluate_predicate_with_kind;
    use antigen_attestation::predicate::{Leaf, Predicate, SignerCurrency};

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
        },
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
        format!(
            "priority_order ref(s) do not resolve to a scanned code site: {unresolved:?} — out of frame (ADR-017-Amd1)"
        )
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

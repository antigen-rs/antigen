//! ATK-ADR029 — `#[defended_by]` audit integration contracts.
//!
//! ADR-029 (Immunity Is Observed, Not Declared) introduces `#[defended_by(X)]`
//! as the code-tier witness registration: a test/proptest declares what
//! failure-class it defends. `cargo antigen audit` cross-references those
//! registrations to `#[presents(X)]` sites and issues verdicts.
//!
//! **The silent failure this file defends against**:
//! `ScanReport::defenses` is populated by scan, but `audit()` could iterate only
//! `report.immunities` and never read `report.defenses` — a `#[defended_by]`
//! test would register correctly yet be silently ignored by the verdict
//! computation, leaving its presents-sites with no verdict at all.
//!
//! ADR-029 implementation (pathmaker, 2026-05-27): `audit()` computes a
//! per-presents-site verdict surface, `AuditReport::presentation_verdicts:
//! Vec<PresentationVerdict>`, with `verdict: ImmuneVerdict =
//! Defended { tier } | Undefended | SubstrateGap`. The verdict is
//! **presents-keyed**, not immunity-keyed: a `#[defended_by(X)]` witness is
//! class-level (it defends ALL `#[presents(X)]` sites), so it cannot map to a
//! single `Immunity`. The legacy `audits: Vec<ImmunityAudit>` stays
//! immunity-keyed (backward-compat — `#[immune]` deprecated but still honored).
//! These tests target `presentation_verdicts`; ATK-ADR029-4 also pins that the
//! `#[immune]` `audits` surface is unaffected.
//!
//! Substrate check:
//!   `cargo test --package antigen --test atk_adr029_defended_by_audit`

use antigen::audit::{audit, ImmuneVerdict, WitnessTier};
use antigen::scan::{Defense, Immunity, ItemTarget, MatchKind, Presentation, ScanReport};
use std::path::PathBuf;

/// Synthesize a minimal `ScanReport` with:
/// - one `#[presents(FailureClass)]` site at src/lib.rs:10
/// - one `#[defended_by(FailureClass)]` test at src/tests.rs:5
/// - no `#[immune]` declarations (old model not used)
fn report_with_defended_by_only() -> ScanReport {
    let mut report = ScanReport::default();

    let presentation = Presentation {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
    };
    report.presentations.push(presentation);

    let defense = Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    };
    report.defenses.push(defense);

    report
}

// ATK-ADR029-1: a defended_by registration must produce a verdict
//
// When scan has a Defense with antigen_type "FailureClass" and a Presentation
// with the same antigen_type, audit() must cross-reference them and emit a
// per-presents-site verdict. The silent failure (audit ignores report.defenses)
// would leave presentation_verdicts empty.
#[test]
fn atk_adr029_1_defended_by_produces_verdict() {
    let report = report_with_defended_by_only();
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let audit_result = audit(&report, workspace_root);

    assert!(
        !audit_result.presentation_verdicts.is_empty(),
        "ATK-ADR029-1: audit() must cross-reference report.defenses to \
         report.presentations and emit a per-presents-site verdict. \
         report.defenses.len() = {}, report.presentations.len() = {}, \
         presentation_verdicts.len() = {}",
        report.defenses.len(),
        report.presentations.len(),
        audit_result.presentation_verdicts.len(),
    );
    // The single FailureClass site must be Defended (a matching witness exists).
    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "FailureClass")
        .expect("a verdict for the FailureClass presents-site");
    assert!(
        matches!(v.verdict, ImmuneVerdict::Defended { .. }),
        "ATK-ADR029-1: a presents-site with a matching #[defended_by] witness \
         must be Defended; got {:?}",
        v.verdict
    );
}

// ATK-ADR029-2: defended_by must produce at least Reachability tier
//
// A registered #[defended_by] witness (test fn) must produce
// WitnessTier >= Reachability. Tier::None / Undefended means "no evidence" — a
// registered defense that produces None tier is a silent failure. v0.3 audit
// does not invoke coverage, so the honest tier is exactly Reachability.
#[test]
fn atk_adr029_2_defended_by_produces_at_least_reachability() {
    let report = report_with_defended_by_only();
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let audit_result = audit(&report, workspace_root);

    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "FailureClass")
        .expect("a verdict for the FailureClass presents-site");
    match &v.verdict {
        ImmuneVerdict::Defended { tier } => assert!(
            *tier >= WitnessTier::Reachability,
            "ATK-ADR029-2: #[defended_by] witness must produce >= Reachability \
             tier; got {tier:?}. A registered defense with None tier means the \
             defense circuit is wired but produces no evidence."
        ),
        other => {
            panic!("ATK-ADR029-2: FailureClass site must be Defended at a tier; got {other:?}")
        }
    }
}

// ATK-ADR029-3: wrong-antigen defended_by must not pollute other presents-sites
//
// A #[defended_by(WrongClass)] test must not grant a defense verdict to a
// RightClass presents-site. Cross-antigen contamination is a silent failure
// where one test accidentally satisfies an unrelated presents-site's defense.
#[test]
fn atk_adr029_3_wrong_antigen_defended_by_does_not_pollute() {
    let mut report = ScanReport::default();

    // presents-site for RightClass
    report.presentations.push(Presentation {
        antigen_type: "RightClass".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 10,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 10 },
        match_kind: MatchKind::ExplicitMarker,
        canonical_path: None,
        inherited_from: None,
        structural_fingerprint: String::new(),
    });

    // witness defending WrongClass — does NOT match RightClass
    report.defenses.push(Defense {
        antigen_type: "WrongClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    // The RightClass presents-site must be Undefended (WrongClass defense does
    // not cross-reference to RightClass).
    let v = audit_result
        .presentation_verdicts
        .iter()
        .find(|v| v.antigen_type == "RightClass")
        .expect("a verdict for the RightClass presents-site");
    assert_eq!(
        v.verdict,
        ImmuneVerdict::Undefended,
        "ATK-ADR029-3: WrongClass defense must not grant RightClass a defense \
         verdict; got {:?}",
        v.verdict
    );
    assert!(
        v.defended_by.is_empty(),
        "ATK-ADR029-3: RightClass verdict must list no defending witnesses; got {:?}",
        v.defended_by
    );
}

// ATK-ADR029-4: immune path (audits surface) still works alongside defended_by
//
// Regression guard: the existing #[immune] `audits` surface must not be broken
// when the defended_by cross-reference is added. A report with both an immunity
// (old model) and a defense (new model) for different antigens must still
// produce an immunity audit entry for the immune path.
#[test]
fn atk_adr029_4_immune_audits_unaffected_by_defended_by() {
    let mut report = ScanReport::default();

    // Old-model: #[immune] on site
    report.immunities.push(Immunity {
        antigen_type: "PanickingInDrop".to_string(),
        witness: "some_test_fn".to_string(),
        file: PathBuf::from("src/lib.rs"),
        line: 20,
        item_kind: "impl".to_string(),
        item_target: ItemTarget::Unknown { line: 20 },
        canonical_path: None,
        requires_predicate: None,
        structural_fingerprint: String::new(),
    });

    // New-model: #[defended_by] on a different antigen
    report.defenses.push(Defense {
        antigen_type: "FailureClass".to_string(),
        file: PathBuf::from("src/tests.rs"),
        line: 5,
        item_kind: "fn".to_string(),
        item_target: ItemTarget::Unknown { line: 5 },
    });

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let audit_result = audit(&report, workspace_root);

    // The immunity audit must still produce an entry for PanickingInDrop — the
    // legacy audits surface is unaffected by the new presentation_verdicts pass.
    assert!(
        !audit_result.audits.is_empty(),
        "ATK-ADR029-4: audit() must still process #[immune] entries when defenses are present"
    );
    assert_eq!(
        audit_result.audits[0].immunity.antigen_type, "PanickingInDrop",
        "ATK-ADR029-4: immunity audit entry must be for the right antigen type"
    );
}

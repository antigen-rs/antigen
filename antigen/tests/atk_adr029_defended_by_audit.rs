//! ATK-ADR029 — `#[defended_by]` audit integration contracts.
//!
//! ADR-029 (Immunity Is Observed, Not Declared) introduces `#[defended_by(X)]`
//! as the code-tier witness registration: a test/proptest declares what
//! failure-class it defends. `cargo antigen audit` is supposed to cross-reference
//! those registrations to `#[presents(X)]` sites and issue verdicts.
//!
//! **The silent failure this file defends against**:
//! `ScanReport::defenses` is populated correctly by scan, but `audit()` in
//! audit.rs iterates only `report.immunities` — it never reads `report.defenses`.
//! A `#[defended_by]`-annotated test registers correctly but is silently ignored
//! by the audit verdict computation. All corresponding presents-sites appear
//! absent from audit output (not even `undefended`) — they fall through with no
//! verdict at all.
//!
//! These tests will FAIL until `audit()` is updated to cross-reference
//! `report.defenses` when computing immunity verdicts for presents-sites.
//!
//! Substrate check:
//!   `cargo test --package antigen --test atk_adr029_defended_by_audit`

use antigen::audit::{audit, WitnessTier};
use antigen::scan::{Defense, Immunity, ItemTarget, MatchKind, Presentation, ScanReport};
use std::path::PathBuf;

// ============================================================================
// ATK-ADR029-1: A presents-site with a matching defended_by registration
//               must produce a non-None verdict in the audit output.
//
// Before ADR-029 implementation in audit.rs: audit() only iterates immunities.
// A report with only `defenses` (no `immunities`) produces zero audits.
// After implementation: audit() must cross-reference defenses to presentations
// and produce at least one verdict for matching antigen_type pairs.
//
// This test asserts what SHOULD be true. It FAILS until audit.rs is updated.
// ============================================================================

/// Synthesize a minimal ScanReport with:
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

// ATK-ADR029-1: defended_by registration must produce at least one audit entry
//
// When scan has a Defense with antigen_type "FailureClass" and a Presentation
// with the same antigen_type, audit() must produce at least one verdict.
// Currently produces zero (audit ignores defenses entirely).
#[test]
fn atk_adr029_1_defended_by_produces_audit_entry() {
    let report = report_with_defended_by_only();
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let audit_result = audit(&report, workspace_root);

    // EXPECTED (after ADR-029 impl): audit_result.audits is non-empty because
    // audit() cross-references defenses to presentations.
    //
    // ACTUAL (before impl): audit_result.audits is empty because audit() only
    // iterates report.immunities, which is empty in this report.
    assert!(
        !audit_result.audits.is_empty(),
        "ATK-ADR029-1 FAILING: audit() ignores report.defenses. \
         A presents-site with only a #[defended_by] witness produces no audit \
         entries at all. Fix: audit() must cross-reference report.defenses to \
         report.presentations and issue per-presentation verdicts. \
         report.defenses.len() = {}, report.presentations.len() = {}, \
         audit_result.audits.len() = {}",
        report.defenses.len(),
        report.presentations.len(),
        audit_result.audits.len(),
    );
}

// ATK-ADR029-2: defended_by must produce at least Reachability tier
//
// After ADR-029 implementation, a registered #[defended_by] witness (test fn)
// must produce WitnessTier >= Reachability. Tier::None means "no evidence" —
// a registered defense that produces None tier is a silent failure.
#[test]
fn atk_adr029_2_defended_by_produces_at_least_reachability() {
    let report = report_with_defended_by_only();
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();

    let audit_result = audit(&report, workspace_root);

    // Precondition: this test only fires meaningfully once ATK-ADR029-1 passes.
    // If no audits, skip — ATK-ADR029-1 handles the absent-audit case.
    if audit_result.audits.is_empty() {
        // Still counts as a failure since the whole suite is about implementation
        // completeness, but let ATK-ADR029-1 carry the primary failure message.
        return;
    }

    for audit_entry in &audit_result.audits {
        assert!(
            audit_entry.witness_tier >= WitnessTier::Reachability,
            "ATK-ADR029-2: #[defended_by] witness must produce >= Reachability tier; \
             got {:?}. A registered defense with None tier means the defense circuit \
             is wired but produces no evidence.",
            audit_entry.witness_tier
        );
    }
}

// ATK-ADR029-3: wrong-antigen defended_by must not pollute other presents-sites
//
// A #[defended_by(WrongClass)] test must not grant a defense verdict to a
// RightClass presents-site. Cross-antigen contamination is a silent failure
// where one test accidentally satisfies an unrelated presents-site's defense
// requirement.
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

    // After ADR-029 impl: RightClass presents-site must appear as undefended
    // (WrongClass defense does not cross-reference to RightClass).
    // We verify: no audit entry has tier > None for the RightClass site.
    for audit_entry in &audit_result.audits {
        // The audit entry's immunity's antigen_type must be checked;
        // if it's for RightClass it must be undefended or None tier.
        if audit_entry.immunity.antigen_type == "RightClass" {
            assert_eq!(
                audit_entry.witness_tier,
                WitnessTier::None,
                "ATK-ADR029-3: WrongClass defense must not grant RightClass a defense verdict. \
                 got tier {:?}",
                audit_entry.witness_tier
            );
        }
    }
}

// ATK-ADR029-4: immune path still works alongside defended_by
//
// Regression guard: the existing #[immune] path must not be broken when
// defended_by cross-reference is added. A report with both an immunity
// (old model) and a defense (new model) for different sites must produce
// audit entries for the immunity path as before.
#[test]
fn atk_adr029_4_immune_path_unaffected_by_defended_by_changes() {
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

    // The immunity audit must still produce an entry for PanickingInDrop.
    assert!(
        !audit_result.audits.is_empty(),
        "ATK-ADR029-4: audit() must still process #[immune] entries when defenses are present"
    );
    assert_eq!(
        audit_result.audits[0].immunity.antigen_type, "PanickingInDrop",
        "ATK-ADR029-4: immunity audit entry must be for the right antigen type"
    );
}

//! ATK-A3 D1.5: audit-side state-7 diagnostic.
//!
//! ADR-018 §"7-state interaction matrix" + §"Audit diagnostic text":
//! inherited Presentations on a descendant that lack matching Immunity
//! or Toleration emit a warn-level diagnostic per `InheritedUnaddressed`
//! record. The default audit returns these in `inherited_unaddressed`
//! without failing; the CLI `--strict` flag promotes to error exit.

use std::path::{Path, PathBuf};

use antigen::audit::{AuditHint, audit};
use antigen::scan::scan_workspace;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn state7_unaddressed_inherited_emits_diagnostic() {
    let fx = fixture("atk_a3_d15_linear_chain");
    let scan = scan_workspace(&fx, None).expect("scan completes");
    let report = audit(&scan, &fx);

    // Child inherits Parent's presentation but has no #[immune] /
    // #[antigen_tolerance] on its own site -> state 7.
    assert!(
        !report.inherited_unaddressed.is_empty(),
        "linear chain fixture's Child should be reported as state 7; \
         got: {:?}",
        report.inherited_unaddressed
    );
    let on_child: Vec<_> = report
        .inherited_unaddressed
        .iter()
        .filter(|iu| {
            iu.presentation.item_target == antigen::scan::ItemTarget::Struct("Child".to_string())
                && iu.presentation.antigen_type == "Parent"
        })
        .collect();
    assert_eq!(
        on_child.len(),
        1,
        "exactly one state-7 record for Parent on Child; got: {on_child:?}"
    );
    assert_eq!(
        on_child[0].audit_hint,
        AuditHint::InheritedPresentationNotReAttested,
        "the audit hint must name the inherited-not-re-attested case"
    );
}

#[test]
fn state6_tolerance_covers_inherited_no_state7_diagnostic() {
    // tolerance_covers fixture: Child #[descended_from(Parent)] AND
    // #[antigen_tolerance(Parent, ...)]. State 4 (tolerated) absorbs
    // the inherited presentation — state-7 diagnostic must NOT fire.
    let fx = fixture("atk_a3_d15_tolerance_covers");
    let scan = scan_workspace(&fx, None).expect("scan completes");
    let report = audit(&scan, &fx);

    let on_child: Vec<_> = report
        .inherited_unaddressed
        .iter()
        .filter(|iu| {
            iu.presentation.item_target == antigen::scan::ItemTarget::Struct("Child".to_string())
        })
        .collect();
    assert!(
        on_child.is_empty(),
        "tolerance covers the inherited Parent presentation on Child; \
         state-7 diagnostic must NOT fire; got: {on_child:?}"
    );
}

//! Scan extraction for the Prescriptive Work-Orchestration family (ADR-033).
//!
//! `cargo antigen scan` reads the eight prescriptive attributes (`#[panel]`,
//! `#[rx]`, `#[refer]`, `#[biopsy]`, `#[ddx]`, `#[triage]`, `#[culture]`,
//! `#[quarantine]`) from SOURCE — like the recurrent family, they are
//! validating pass-through macros with no expansion, so the scanner walks the
//! un-expanded attribute. Each becomes a [`PrescriptiveDeclaration`] with a
//! [`PrescriptiveKind`] that maps to a [`WorkShape`].
//!
//! These tests pin: (1) each kind is extracted with the right `PrescriptiveKind`;
//! (2) the per-shape field NAMES map onto the shared declaration slots
//! (`needs`/`rule_out`/`priority_order` → `items`; the who-refs → `filled_by`; the
//! frames → `frame`; etc.); (3) the four-shape mapping (`PrescriptiveKind::shape`).

use antigen::scan::{PrescriptiveKind, WorkShape, scan_workspace};

/// Stage a single-crate workspace whose lib.rs uses every prescriptive macro,
/// then return the scan report's prescriptive declarations.
fn scan_staged(lib_src: &str) -> Vec<antigen::scan::PrescriptiveDeclaration> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"staged\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
         [lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::write(src.join("lib.rs"), lib_src).unwrap();
    let report = scan_workspace(tmp.path(), None).expect("scan completes");
    report.prescriptive_declarations
}

#[test]
fn scan_extracts_all_eight_prescriptive_kinds() {
    let src = r#"use antigen::{panel, rx, refer, biopsy, ddx, triage, culture, quarantine};

#[panel(needs = ["audit error path", "check overflow"], filled_by = ["alice"], reviewed_by = ["bob"], ordered_by = "carol", due = "2027-01-01")]
pub fn p() {}

#[rx(treatment = "add retry with backoff", diagnosis = "transient-failure", filled_by = ["dave"], due = "2027-02-01")]
pub fn r() {}

#[refer(to = "platform-team", response_due = "2027-03-01")]
pub fn rf() {}

#[biopsy(location = "parser::fast_path", request_text = "why two allocations", deep_investigation_by = "erin")]
pub fn b() {}

#[ddx(symptom = "slow query", rule_out = ["missing index", "n+1"], investigator = "frank", reviewer = "grace")]
pub fn d() {}

#[triage(priority_order = ["src/a.rs::foo", "src/b.rs::bar"], triaged_by = "reviewer", re_triage_due = "2027-04-01")]
pub fn t() {}

#[culture(test_kind = "24h soak", runs_until = "2027-05-01")]
pub fn c() {}

#[quarantine(scope = "legacy::mod", until = "2027-06-01", reason = "pending upstream fix")]
pub fn q() {}
"#;
    let decls = scan_staged(src);
    assert_eq!(
        decls.len(),
        8,
        "all eight prescriptive sites must be extracted: {decls:#?}"
    );

    let kinds: std::collections::HashSet<PrescriptiveKind> = decls.iter().map(|d| d.kind).collect();
    for expected in [
        PrescriptiveKind::Panel,
        PrescriptiveKind::Rx,
        PrescriptiveKind::Refer,
        PrescriptiveKind::Biopsy,
        PrescriptiveKind::Ddx,
        PrescriptiveKind::Triage,
        PrescriptiveKind::Culture,
        PrescriptiveKind::Quarantine,
    ] {
        assert!(kinds.contains(&expected), "missing kind {expected:?}");
    }
}

#[test]
fn panel_fields_map_onto_shared_slots() {
    let src = r#"use antigen::panel;
#[panel(needs = ["a", "b"], filled_by = ["alice"], reviewed_by = ["bob"], ordered_by = "carol", due = "2027-01-01")]
pub fn p() {}
"#;
    let decls = scan_staged(src);
    let panel = decls
        .iter()
        .find(|d| d.kind == PrescriptiveKind::Panel)
        .expect("panel extracted");
    // needs → items
    assert_eq!(panel.items, vec!["a", "b"]);
    assert_eq!(panel.filled_by, vec!["alice"]);
    assert_eq!(panel.reviewed_by, vec!["bob"]);
    assert_eq!(panel.ordered_by.as_deref(), Some("carol"));
    assert_eq!(panel.frame.as_deref(), Some("2027-01-01")); // due → frame
}

#[test]
fn ddx_rule_out_maps_to_items_and_symptom_to_need_text() {
    let src = r#"use antigen::ddx;
#[ddx(symptom = "slow query", rule_out = ["missing index", "n+1"])]
pub fn d() {}
"#;
    let decls = scan_staged(src);
    let ddx = decls
        .iter()
        .find(|d| d.kind == PrescriptiveKind::Ddx)
        .expect("ddx extracted");
    assert_eq!(ddx.items, vec!["missing index", "n+1"]); // rule_out → items
    assert_eq!(ddx.need_text.as_deref(), Some("slow query")); // symptom → need_text
}

#[test]
fn triage_priority_order_maps_to_items_and_triaged_by_to_filled_by() {
    let src = r#"use antigen::triage;
#[triage(priority_order = ["src/a.rs::foo", "src/b.rs::bar"], triaged_by = "nav")]
pub fn t() {}
"#;
    let decls = scan_staged(src);
    let triage = decls
        .iter()
        .find(|d| d.kind == PrescriptiveKind::Triage)
        .expect("triage extracted");
    assert_eq!(triage.items, vec!["src/a.rs::foo", "src/b.rs::bar"]);
    assert_eq!(triage.filled_by, vec!["nav"]); // triaged_by → filled_by slot
}

#[test]
fn quarantine_reason_to_need_text_scope_to_label_until_to_frame() {
    let src = r#"use antigen::quarantine;
#[quarantine(scope = "legacy::mod", until = "2027-06-01", reason = "pending upstream fix")]
pub fn q() {}
"#;
    let decls = scan_staged(src);
    let q = decls
        .iter()
        .find(|d| d.kind == PrescriptiveKind::Quarantine)
        .expect("quarantine extracted");
    assert_eq!(q.need_text.as_deref(), Some("pending upstream fix")); // reason → need_text
    assert_eq!(q.label.as_deref(), Some("legacy::mod")); // scope → label
    assert_eq!(q.frame.as_deref(), Some("2027-06-01")); // until → frame
}

#[test]
fn kind_to_shape_mapping_is_four_shapes() {
    // ADR-033 §Decision 1: eight names, four shapes.
    assert_eq!(PrescriptiveKind::Panel.shape(), WorkShape::RoleWorkflow);
    assert_eq!(PrescriptiveKind::Rx.shape(), WorkShape::RoleWorkflow);
    assert_eq!(PrescriptiveKind::Refer.shape(), WorkShape::RoleWorkflow);
    assert_eq!(PrescriptiveKind::Biopsy.shape(), WorkShape::RoleWorkflow);
    assert_eq!(PrescriptiveKind::Ddx.shape(), WorkShape::Elimination);
    assert_eq!(PrescriptiveKind::Triage.shape(), WorkShape::Ordering);
    assert_eq!(PrescriptiveKind::Culture.shape(), WorkShape::FrameOnly);
    assert_eq!(PrescriptiveKind::Quarantine.shape(), WorkShape::FrameOnly);
}

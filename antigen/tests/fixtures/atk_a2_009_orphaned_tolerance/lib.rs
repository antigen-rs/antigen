// ATK-A2-009 fixture: tolerance marker for an antigen that doesn't exist in
// this workspace. Per ADR-011 §Mechanics + biological cognate (peripheral
// suppression continuing after the antigen it suppressed is no longer
// present — autoimmune dysregulation).
//
// `OldAntigen` is referenced by tolerance but never declared. The scan
// must surface this as an orphaned tolerance.

#[antigen_tolerance(
    OldAntigen,
    rationale = "Test fixture deliberately constructs the orphan scenario."
)]
fn deliberately_constructs_orphan() {}

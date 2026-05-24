//! Adversarial fixture: #[triage_commit] without triage_decision must reject.

use antigen_macros::triage_commit;

#[triage_commit(
    rollback_target = "abc1234",
    triaged_by = "navigator",
    rationale = "twenty-character-rationale-text-here",
    rollback_due_within_minutes = 30,
)]
fn _triage_marker() {}

fn main() {}

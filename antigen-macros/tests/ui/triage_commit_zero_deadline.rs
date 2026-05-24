//! Adversarial fixture: rollback_due_within_minutes = 0 must be rejected.
//! Zero deadline degrades the time-bound discipline.

use antigen_macros::triage_commit;

#[triage_commit(
    triage_decision = TriageDecision::Red,
    rollback_target = "abc1234",
    triaged_by = "navigator",
    rationale = "twenty-character-rationale-text-here",
    rollback_due_within_minutes = 0,
)]
fn _triage_marker() {}

fn main() {}

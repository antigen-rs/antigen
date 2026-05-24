//! Adversarial fixture: rationale below 20 characters must be rejected.
//! "short rationale" is 15 chars.

use antigen_macros::triage_commit;

#[triage_commit(
    triage_decision = TriageDecision::Red,
    rollback_target = "abc1234",
    triaged_by = "navigator",
    rationale = "short rationale",
    rollback_due_within_minutes = 30,
)]
fn _triage_marker() {}

fn main() {}

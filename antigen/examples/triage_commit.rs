// ADR-029 deprecation-window: uses the deprecated-but-functional #[immune] API;
// full migration to #[defended_by]/#[presents(requires=)] is a tracked follow-on.
#![allow(deprecated)]

//! Example: `#[triage_commit]` — decisional rollback-as-triage (ADR-026).
//!
//! `#[triage_commit]` is the SPEECH-ACT that turns a rollback function into a
//! chart entry: "I have triaged this system state. Here is my classification,
//! my target, my authority, my rationale, and my time-bound."
//!
//! ## The central distinction — `#[triage_commit]` vs `#[orient]`
//!
//! ```
//! orient        says: "I don't have a defense yet. I'll have one by <date>."
//! triage_commit says: "I have diagnosed this. I am rolling back NOW. Here is why."
//! ```
//!
//! - `#[orient]` is a **deferral**: acknowledges a gap, commits to closing it.
//! - `#[triage_commit]` is a **decision**: diagnoses a live condition, commits
//!   to a bounded action.
//!
//! Different speech-acts. Different scopes. Using `#[orient]` on a rollback site
//! is structurally wrong — it says "I'll address this later" when the system
//! is already in a state requiring immediate action.
//!
//! ## The 5-color triage scale (ADR-026 §Triage-decision)
//!
//! | Color  | Meaning                                                              |
//! |--------|----------------------------------------------------------------------|
//! | Black  | System-down / data-loss imminent / catastrophic regression confirmed |
//! | Red    | Vital-metric regression confirmed; tight-window rollback required    |
//! | Yellow | Concerning signal; investigation pending; decision deferred          |
//! | Green  | No regression; analysis chain attests non-regression                 |
//! | White  | Out of scope; explicit non-action chart entry                        |
//!
//! `Black` and `Red` mandate rollback. `Yellow` defers the decision.
//! `Green` and `White` are non-rollback chart entries — explicit non-action
//! is also a valid triage outcome that deserves documentation.
//!
//! ## What the substrate-witness confirms
//!
//! The `#[immune]` + `requires = signed_trailer(key = "Triage-Decision")` on a
//! rollback site tells `cargo antigen audit`: "This rollback is defended IF the
//! commit has a `Triage-Decision:` git trailer." The trailer is written by the
//! engineer performing the triage; the macro ensures it is non-negotiable
//! by making the ANNOTATION compile-time required before the code compiles.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example triage_commit --package antigen
//! ```
//!
//! See ADR-026 for the full rollback-as-triage discipline, commit-hook
//! enforcement, and audit-hint lifecycle.

#![allow(dead_code, unused_variables, unused_imports)]

use antigen::stdlib::vcs_info_loss::RollbackWithoutTriageCommit;
use antigen::{immune, orient, triage_commit, TriageDecision};

// ============================================================================
// Speech-act contrast — orient vs triage_commit
//
// These two forms look similar but mean fundamentally different things.
// ============================================================================

/// Code under orientation: new auth module not yet reviewed against the
/// session-fixation failure-class.
///
/// `#[orient]` says: "I acknowledge this gap. I will close it by 2026-11-01."
/// This is a DEFERRAL — appropriate when there is no live incident, just a
/// training gap.
#[orient(
    learning_path = "Review auth module against session-fixation failure-class before v1 tag",
    until = "2026-11-01"
)]
pub const fn new_auth_handler(token: &str) -> Result<(), String> {
    // Simplified auth handling.
    Ok(())
}

/// Emergency rollback of auth module — active session-fixation exploit confirmed.
///
/// `#[triage_commit]` says: "I have classified this incident. I am acting NOW.
/// Here is my classification, my target, my authority, and my time-bound."
/// This is a DECISION — the system is in an active incident state.
///
/// If you reach for `#[orient]` here, the macro rejects it: orient says
/// "I'll address this later," which is structurally wrong for an active
/// incident requiring a bounded rollback.
#[triage_commit(
    triage_decision = TriageDecision::Red,
    rollback_target = "a83f2c1",
    triaged_by = "oncall-navigator",
    rationale = "Session-fixation exploit confirmed via WAF logs (INCIDENT-8841); \
                 a83f2c1 is the last-clean-CI snapshot before the vulnerable auth path merged",
    rollback_due_within_minutes = 30
)]
#[immune(
    RollbackWithoutTriageCommit,
    requires = signed_trailer(key = "Triage-Decision"),
    rationale = "triage_commit annotation + Triage-Decision git trailer document \
                 the diagnosis; signed_trailer confirms the trailer is present in \
                 the rollback commit at audit time."
)]
pub fn rollback_auth_to_last_clean(snapshot_sha: &str) -> Result<(), String> {
    println!("[ROLLBACK] Auth module → {snapshot_sha}");
    Ok(())
}

// ============================================================================
// The 5-color scale in practice
//
// Each color represents a distinct diagnostic outcome. All five are valid
// triage_commit outcomes — even Green and White, which document explicit
// non-action decisions that would otherwise be invisible in the git log.
// ============================================================================

/// Black triage — system-down, data-loss imminent.
///
/// The most severe level. `rollback_due_within_minutes` is minimal (5) because
/// Black triage means the system cannot serve requests at all.
#[triage_commit(
    triage_decision = TriageDecision::Black,
    rollback_target = "7d4bc89",
    triaged_by = "sre-lead",
    rationale = "Payment processor reporting 100% transaction failure; pod OOMKilled loop; \
                 7d4bc89 is the pre-incident head per runbook step 3",
    rollback_due_within_minutes = 5
)]
pub fn rollback_payment_processor(snapshot_sha: &str) -> Result<(), String> {
    println!("[ROLLBACK] Payment processor → {snapshot_sha}");
    Ok(())
}

/// Yellow triage — concerning signal, investigation pending.
///
/// Yellow is not a rollback: the decision is deferred while investigation
/// continues. `rollback_due_within_minutes` reflects the investigation window.
/// This is still a `triage_commit` — the non-rollback decision is chart-documented.
#[triage_commit(
    triage_decision = TriageDecision::Yellow,
    rollback_target = "HEAD",
    triaged_by = "oncall-navigator",
    rationale = "p99 latency elevated 40% in EU-WEST region; no vital-metric breach yet; \
                 investigating correlation with deploy ac91def; decision pending",
    rollback_due_within_minutes = 60
)]
pub fn investigate_latency_regression() {
    println!("[YELLOW] Investigating — no rollback yet");
}

/// Green triage — no regression, explicit non-action.
///
/// Green documents that a potential regression was investigated and ruled out.
/// Without this marker, the investigation is invisible in the git log; the next
/// on-call engineer might re-investigate the same signal from scratch.
#[triage_commit(
    triage_decision = TriageDecision::Green,
    rollback_target = "HEAD",
    triaged_by = "senior-navigator",
    rationale = "Alert fired on elevated error-rate; investigation shows traffic spike \
                 from load-test in staging leaking to prod metrics; no actual regression; \
                 no rollback warranted; suppressing alert for 2h",
    rollback_due_within_minutes = 120
)]
pub fn close_false_positive_alert() {
    println!("[GREEN] Investigated — no regression found, alert suppressed");
}

/// White triage — out of scope for this triage event.
///
/// White explicitly records that a question was considered but ruled out of
/// scope. Like Green, it makes invisible reasoning visible in the substrate.
#[triage_commit(
    triage_decision = TriageDecision::White,
    rollback_target = "HEAD",
    triaged_by = "oncall-navigator",
    rationale = "Cache layer was flagged during triage of INCIDENT-9011 but was determined \
                 unrelated to the active p0; explicitly scoped out to avoid distraction; \
                 cache investigation to continue post-incident in separate thread",
    rollback_due_within_minutes = 240
)]
pub fn scope_out_cache_investigation() {
    println!("[WHITE] Cache layer explicitly scoped out of INCIDENT-9011 triage");
}

fn main() {
    println!("=== antigen triage_commit example ===");
    println!();
    println!("Speech-act distinction — #[orient] vs #[triage_commit]:");
    println!("  orient:        'I don't have a defense yet. I will by <date>.' (DEFERRAL)");
    println!("  triage_commit: 'I have diagnosed this. Acting NOW. Here is why.' (DECISION)");
    println!();
    println!("new_auth_handler:            #[orient] — training gap, not a live incident");
    println!(
        "rollback_auth_to_last_clean: #[triage_commit] Red — active exploit, bounded rollback"
    );
    println!();
    println!("The 5-color triage scale:");
    println!("  Black  — system-down; catastrophic; 5min window");
    println!("  Red    — vital-metric regression confirmed; tight rollback window");
    println!("  Yellow — concerning signal; decision pending; investigation window");
    println!("  Green  — no regression found; explicit non-action documented");
    println!("  White  — out of scope; explicit non-action documented");
    println!();
    println!("All five are valid triage_commit outcomes. Green and White make");
    println!("invisible non-action decisions visible in the git substrate.");
    println!();

    let _ = new_auth_handler("tok_abc");
    let _ = rollback_auth_to_last_clean("a83f2c1");
    let _ = rollback_payment_processor("7d4bc89");
    investigate_latency_regression();
    close_false_positive_alert();
    scope_out_cache_investigation();

    println!();
    println!("Mandatory fields (all required — a triage without them cannot compile):");
    println!("  triage_decision        = TriageDecision::{{Black|Red|Yellow|Green|White}}");
    println!("  rollback_target        = \"<sha>\"  — last-known-good snapshot");
    println!("  triaged_by             = \"<role>\" — informed-consent author identity");
    println!("  rationale              = \"...\"    — chart-documentation (>= 20 chars)");
    println!("  rollback_due_within_minutes = N   — bounded time window (> 0)");
}

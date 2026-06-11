//! BYPASS-3 (ADR-048 §Q9): a `::new` associated constructor.
//!
//! There is intentionally NO `PromotedDraft::new(...)` — the ONLY minter is the
//! gate (`promote_if_safe` / `propose`). A `::new` would be a second, gate-skipping
//! promotion path (`ParallelStateTrackersDiverge` for the safety gate, ADR-048 §Q2).
//! Its absence MUST surface as a compile error.

use antigen::learn::self_tolerance::PromotedDraft;
use antigen_fingerprint::Fingerprint;

fn main() {
    let fp = Fingerprint::parse(r#"all_of([item = fn, body_calls("unwrap")])"#).unwrap();
    // Must fail: no associated `new` exists (the gate is the only minter).
    let _forged = PromotedDraft::new(fp);
}

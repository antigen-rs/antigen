//! BYPASS-5 (ADR-048 §Q2): a `Default` constructor.
//!
//! `PromotedDraft` does NOT derive or implement `Default` (ADR-048 Mechanics §1).
//! A `Default::default()` token would be a gate-skipping mint of an empty/degenerate
//! fingerprint — assertable with no gate run. The missing impl MUST be a compile
//! error.

use antigen::learn::self_tolerance::PromotedDraft;

fn main() {
    // Must fail: PromotedDraft does not implement Default.
    let _forged: PromotedDraft = Default::default();
}

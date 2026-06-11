//! BYPASS-4 (ADR-048 §Q2): a `From<Fingerprint>` conversion.
//!
//! `PromotedDraft` intentionally does NOT implement `From<Fingerprint>` (ADR-048
//! Mechanics §1: "No `pub fn new`, no `From<Fingerprint>`, no `Default`"). A
//! `Fingerprint::into()` that yielded a token would let a raw, un-gated draft become
//! promotable by an ordinary `.into()` — the exact bypass the newtype forbids. The
//! missing impl MUST be a compile error.

use antigen::learn::self_tolerance::PromotedDraft;
use antigen_fingerprint::Fingerprint;

fn main() {
    let fp = Fingerprint::parse(r#"all_of([item = fn, body_calls("unwrap")])"#).unwrap();
    // Must fail: there is no `From<Fingerprint> for PromotedDraft`.
    let _forged: PromotedDraft = fp.into();
}

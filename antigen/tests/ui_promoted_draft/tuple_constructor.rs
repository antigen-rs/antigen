//! BYPASS-1 (ADR-048 §Q9): the tuple constructor.
//!
//! `PromotedDraft` has a private field, so `PromotedDraft(fingerprint)` from
//! outside the gate's module MUST NOT compile. If it did, any caller could mint a
//! token wrapping an UN-GATED fingerprint — the autoimmune draft asserted with no
//! gate ever run. The seal IS the private field.

use antigen::learn::self_tolerance::PromotedDraft;
use antigen_fingerprint::Fingerprint;

fn main() {
    let fp = Fingerprint::parse(r#"all_of([item = fn, body_calls("unwrap")])"#).unwrap();
    // Must fail: the field is private; there is no public tuple constructor.
    let _forged = PromotedDraft(fp);
}

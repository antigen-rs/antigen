//! BYPASS-2 (ADR-048 §Q9): the struct-literal with the private field.
//!
//! Even knowing the field names, a struct literal `PromotedDraft { fingerprint,
//! tier }` from outside the module MUST NOT compile — the fields are private. This
//! is the same seal as the tuple constructor, attacked via the named-field path
//! (a forger who read the source and tries to name the fields directly).

use antigen::finding::Provenance;
use antigen::learn::self_tolerance::PromotedDraft;
use antigen_fingerprint::Fingerprint;

fn main() {
    let fp = Fingerprint::parse(r#"all_of([item = fn, body_calls("unwrap")])"#).unwrap();
    // Must fail: both fields are private to `self_tolerance`.
    let _forged = PromotedDraft {
        fingerprint: fp,
        tier: Provenance::DEFAULT,
    };
}

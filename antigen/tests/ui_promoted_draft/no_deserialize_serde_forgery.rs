//! BYPASS-6 (ADR-048 §5): the serde-forgery surface — the THIRD member of the
//! shared gate-bypass class.
//!
//! The private-field / no-public-constructor seal does NOT, by itself, cover the
//! serde construction path: `#[derive(Deserialize)]` can construct private fields,
//! so a hand-written JSON `{"fingerprint": ..., "tier": ...}` would **forge a
//! `PromotedDraft` token from disk** that never passed the gate. The ruling
//! (ADR-048 §5): `PromotedDraft` does NOT derive `Deserialize`. (`Serialize` is
//! fine — emitting the token's fingerprint is safe; it is *construction-from-
//! untrusted-bytes* that forges.) So `serde_json::from_str::<PromotedDraft>` MUST
//! NOT compile — the type does not satisfy `Deserialize`.
//!
//! Without this, the whole capability-token discipline launders through a JSON file:
//! persist a token, hand-edit the JSON to wrap an autoimmune draft, deserialize it
//! back, and assert it — the gate bypassed at the persistence boundary.

use antigen::learn::self_tolerance::PromotedDraft;

fn main() {
    // A forger's hand-written JSON purporting to be a gated token.
    let forged_json = r#"{"fingerprint":{"constraints":[]},"tier":"Imagined"}"#;
    // Must fail: PromotedDraft does NOT implement Deserialize (the serde-forgery
    // guard). There is no `Deserialize` bound to satisfy `from_str`.
    let _forged: PromotedDraft = serde_json::from_str(forged_json).unwrap();
}

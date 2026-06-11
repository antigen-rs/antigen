//! POSITIVE CONTROL (the teeth-check for the ATK-048 seal suite).
//!
//! Every compile-fail fixture in `ui_promoted_draft/` asserts a *construction*
//! bypass fails. But a compile-fail "passes" whether it failed for the RIGHT reason
//! (the constructor is sealed) or a WRONG one (the import is typo'd, the type is
//! unreachable, the crate doesn't build). This fixture removes that ambiguity: it
//! names the gate's intended PUBLIC surface and MUST COMPILE. If this breaks, the
//! compile-fails are false-greens — the seal isn't being tested, the path is just
//! broken.
//!
//! It is a COMPILE-only control (`t.pass`): it asserts the public *type surface* is
//! reachable and correctly typed, WITHOUT asserting any runtime promotion verdict
//! (the gate's accept/route/refuse decision is the keystone's own moving target,
//! defended by the born-red unit tests in `self_tolerance.rs`, not here). Coupling
//! this control to a specific verdict would make the seal-suite's teeth-check
//! brittle to the gate's in-flight tuning — the wrong dependency. So it references
//! the surface through a function it never calls at runtime.

use antigen::learn::propose::{propose, ProposeOutcome};
use antigen::learn::self_tolerance::{promote_if_safe, PromotedDraft, ToleranceVerdict};
use antigen_fingerprint::Fingerprint;

/// Names every public read-accessor of the capability token — proving the surface
/// (ADR-048 Mechanics §4) is reachable and typed as documented. Never executed; the
/// `t.pass` harness only needs it to *type-check*.
#[allow(dead_code)]
fn the_public_read_surface(token: PromotedDraft) {
    let _fp: &Fingerprint = token.fingerprint(); // read access
    let _t = token.tier(); // the gate-assigned score
    let _r: &Fingerprint = token.as_ref(); // AsRef<Fingerprint>
    let _owned: Fingerprint = token.into_fingerprint(); // one-way downgrade
}

/// Names the two minters' signatures — proving the ONLY paths to a token are the
/// gate (`promote_if_safe`) and the cluster→token convenience (`propose`), each
/// returning a `Result` whose `Ok` is the token and whose `Err` is the legible
/// verdict. (Type-checked, not executed.)
#[allow(dead_code)]
fn the_minter_signatures(draft: Fingerprint, cluster: &[syn::Item], corpus: &[syn::Item]) {
    let _gate: Result<PromotedDraft, ToleranceVerdict> = promote_if_safe(draft, corpus);
    let _cluster_path: Result<PromotedDraft, ProposeOutcome> = propose(cluster, corpus);
}

fn main() {}

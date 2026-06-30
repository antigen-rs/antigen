//! ATK-FRAME-LASTCHANGED — concurrent `last_changed` writes resolve by `max(mtime)` (ADR-070 §4.5a / A4).
//!
//! ## The claim this defends (ADR-070 §4.5a ruling, attack A4)
//! ADR-067 LE6 makes `last_changed` a maintenance-stamped `#[salsa::input]` but is SILENT on the
//! concurrent-write merge-order (the STOCK multi-writer seam). The ruling: concurrent writers resolve
//! by `max(observed mtime)` — take the LATER timestamp (most-recent-knowledge wins). This is a
//! deterministic, monotone, idempotent join: both writers derive from the SAME fs source, so whichever
//! `set` wins LAST is correct regardless of order. The general `SovereignMerge` arbiter COLLAPSES to
//! `max` for this field (compose-clean, no authority adjudication).
//!
//! ## The PROCESS this binds: the maintenance `set` is monotone — `to(max(current, observed))`, NEVER
//! a blind overwrite-with-older. A born-red obligation (§8 ATK-FRAME-LASTCHANGED).
//!
//! ## A SEAM NOTE the builder must honor (surfaced)
//! The skeleton ships `Revision(pub u64)` and the `last_changed` field but NOT the merge helper. This
//! ATK is written against a `merge_last_changed(current, observed) -> Revision` seam the builder owes
//! (a free function or `Revision::merge`). It is the place the §4.5a ruling becomes code. If the
//! builder names it differently, retarget + de-ignore. (Tracked in the registry; seam-gap noted.)
//!
//! ## Teeth (the negative control)
//! The "older write does not regress" direction IS the teeth: a blind last-write-wins impl would PASS
//! the order-independence of two-increasing writes but FAIL the regression check. The two together
//! force `max`, not LWW.

use antigen_stroma::node::node::Revision;

// The merge seam the §4.5a ruling requires — FILLED: delegates to the builder's `Revision::merge`
// (the monotone `max` join). De-ignored: the tests now run as forever regression guards.
fn merge_last_changed(current: Revision, observed: Revision) -> Revision {
    current.merge(observed)
}

// ATK-FRAME-LASTCHANGED (born-red): two concurrent sets with t1 < t2 leave t2 REGARDLESS OF ORDER.
#[test]
fn atk_frame_lastchanged_max_is_order_independent() {
    let t1 = Revision(1000);
    let t2 = Revision(2000);

    // Order A: observe t1 then t2.
    let order_a = merge_last_changed(merge_last_changed(Revision(0), t1), t2);
    // Order B: observe t2 then t1.
    let order_b = merge_last_changed(merge_last_changed(Revision(0), t2), t1);

    assert_eq!(
        order_a, t2,
        "ATK-FRAME-LASTCHANGED: order A (t1 then t2) did not settle on the LATER timestamp t2."
    );
    assert_eq!(
        order_a, order_b,
        "ATK-FRAME-LASTCHANGED: the merge is ORDER-DEPENDENT — different write orders gave different \
         results. The §4.5a ruling requires a commutative monotone join (max), not last-write-wins."
    );
}

// NEGATIVE CONTROL (teeth): a single write with an OLDER mtime than the current value MUST NOT regress
// last_changed backward. A blind LWW impl regresses here; `max` does not.
#[test]
fn nc_frame_lastchanged_older_write_does_not_regress() {
    let current = Revision(2000);
    let stale_observed = Revision(500);

    let after = merge_last_changed(current, stale_observed);
    assert_eq!(
        after, current,
        "NC: an older observed mtime regressed last_changed backward (2000 -> 500). This is blind \
         last-write-wins, not the monotone `max` the ruling requires — most-recent-knowledge LOST."
    );
}

// NEGATIVE CONTROL (teeth, idempotence): merging a value with itself is a no-op (the join is
// idempotent — a property the §4.5a ruling names explicitly).
#[test]
fn nc_frame_lastchanged_is_idempotent() {
    let v = Revision(1500);
    assert_eq!(
        merge_last_changed(v, v),
        v,
        "NC: merging a timestamp with itself changed it — the join is not idempotent."
    );
}

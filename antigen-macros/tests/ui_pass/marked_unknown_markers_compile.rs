//! Pass-fixture (ADR-041): the three marked-unknown markers compile cleanly when
//! given a stated `trigger`, and are pure identity transforms (the annotated item
//! still works normally). If the macro ever stops being identity-preserving — or
//! the required-trigger accept path breaks — this file goes red.

use antigen_macros::{aura, dread, red_flag};

#[aura(trigger = "this retry loop has no jitter; under load it might thundering-herd")]
fn retry_request() -> u32 {
    7
}

struct Connection;

#[dread(trigger = "the teardown drops the guard before the flush; the ordering feels wrong")]
impl Drop for Connection {
    fn drop(&mut self) {}
}

#[red_flag(trigger = "this auth check is reachable with an empty token on the cache-hit path")]
fn authorize() -> bool {
    true
}

fn main() {
    // The markers are identity transforms — the items work normally.
    assert_eq!(retry_request(), 7);
    let _c = Connection;
    assert!(authorize());
}

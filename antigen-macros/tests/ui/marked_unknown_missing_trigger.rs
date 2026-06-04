//! Compile-fail fixture (ADR-041 guard 3, the KEYSTONE): a triggerless
//! `#[dread]` must reject. A marked-unknown with no stated trigger is the
//! contentless "this seems off" graffiti the marker exists to prevent — the
//! `trigger` field is REQUIRED, not optional. Anchored to `MarkerArgs::validate`.

use antigen_macros::dread;

#[dread]
fn no_trigger() {}

fn main() {}

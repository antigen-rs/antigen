// Negative control for ATK-FRAME-NONIDEM: the SOUND path MUST compile. An IDEMPOTENT semiring
// (`Detection`) run on a raw graph is closure-safe on cycles, so `run_closure::<Detection>` type-checks.
// A gate that rejected EVERY semiring would be useless — this proves the gate admits the sound path
// while (see `tests/ui/nonidem_run_closure_blast_forbidden.rs`) rejecting the non-idempotent one.

use antigen_stroma::engine::{run_closure, Detection, ReachabilityRelation};

fn main() {
    let rel = ReachabilityRelation::from_edges(&[]);
    let _matrix = run_closure::<Detection>(&rel);
}

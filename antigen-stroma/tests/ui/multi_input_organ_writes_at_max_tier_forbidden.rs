// ATK-SIGNAL-B2PRIME-MAXTIER (ADR-071 B2′; nb 028 §5) — a MULTI-input organ MUST NOT be able to write an
// afferent output at (or below) the MAX of the tiers it read. "read-lower, write-higher" is a COMPILE-TIME
// impossibility for multi-lineage organs exactly as for the single-input case: `write_back2` mechanically
// returns `Up<Max<A, B>>`, one tier ABOVE the max of the two inputs. Here the inputs are at tiers 0 (Base)
// and 2 (Up<Up<Base>>), so max = 2 and the door yields tier 3 (Up<Up<Up<Base>>>). Annotating the result at
// the max tier itself (tier 2 = Up<Up<Base>>) — i.e. writing AT the highest tier it read — forces a type
// mismatch. No door in the `signal` module yields an output at a tier ≤ any tier it read.
//
// TEETH: if the wall did NOT exist (i.e. `write_back2` returned `Max<A, B>` instead of `Up<Max<A, B>>`),
// this annotation would type-check and the fixture would COMPILE → trybuild would report "expected
// compile_fail but it succeeded" → RED. The `ui_pass` control (`multi_input_organ_writes_above_max_tier`)
// proves the sound multi-lineage climb still compiles, so the wall is not vacuous.

use antigen_stroma::signal::{ingest, write_back, write_back2, Base, OrganOutput, Up};

fn main() {
    let t0 = ingest(1u32); // OrganOutput<u32, Base>          (tier 0)
    let t1 = write_back(&t0, 2u32); // OrganOutput<u32, Up<Base>>      (tier 1)
    let t2 = write_back(&t1, 3u32); // OrganOutput<u32, Up<Up<Base>>>  (tier 2)

    // `write_back2` yields `Up<Max<Base, Up<Up<Base>>>>` = tier 3; claiming it is tier 2 (the MAX we read,
    // Up<Up<Base>>) MUST NOT compile.
    let _at_max: OrganOutput<u32, Up<Up<Base>>> = write_back2(&t0, &t2, 4u32);
}

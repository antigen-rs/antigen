// ATK-SIGNAL-B2PRIME-WRITE-AT-READ-TIER (ADR-071 B2′) — an organ MUST NOT be able to write an afferent
// output at the tier it read from. "read-lower, write-higher" is a COMPILE-TIME impossibility, not a
// runtime check: `write_back` mechanically returns the successor tier `Up<R>`, so annotating the result
// at the read tier `R` (here `Base`) forces a type mismatch (`Up<Base>` vs `Base`). No function in the
// `signal` module yields an `OrganOutput` at a tier ≤ the tier it read — the afferent hierarchy is a DAG
// by construction (no organ can feed its own causal upstream).
//
// TEETH: if the wall did NOT exist (i.e. `write_back` returned `R` instead of `Up<R>`), this annotation
// would type-check and the fixture would COMPILE → trybuild would report "expected compile_fail but it
// succeeded" → RED. The `ui_pass` control (`organ_output_climbs_the_tower`) proves the sound path still
// compiles, so the wall is not vacuous.

use antigen_stroma::signal::{ingest, write_back, Base, OrganOutput};

fn main() {
    let read = ingest(1u32); // OrganOutput<u32, Base>
    // `write_back` yields `Up<Base>`; claiming it is `Base` (the tier we read) MUST NOT compile.
    let _same_tier: OrganOutput<u32, Base> = write_back(&read, 2u32);
}

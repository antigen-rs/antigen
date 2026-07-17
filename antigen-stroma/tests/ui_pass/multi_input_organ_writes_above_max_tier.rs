// Negative control for ATK-SIGNAL-B2PRIME-MAXTIER (ADR-071 B2′; nb 028 §5): the SOUND multi-input path
// MUST compile. A latent organ with SEVERAL afferent inputs at DIFFERENT tiers writes strictly above the
// MAX of the tiers it reads, via the type-level `Max` and the `write_back2` / `write_back3` doors. A wall
// that rejected the legitimate multi-lineage climb would be useless; this proves it admits it, and — via
// the explicit tier annotations — that the computed output tier is exactly `Up<Max<..>>`.

use antigen_stroma::signal::{ingest, write_back, write_back2, write_back3, Base, OrganOutput, Up};

fn main() {
    // Build inputs at three distinct tiers.
    let t0 = ingest(10u32); // OrganOutput<u32, Base>              (tier 0)
    let t1 = write_back(&t0, 11u32); // OrganOutput<u32, Up<Base>>          (tier 1)
    let t2 = write_back(&t1, 12u32); // OrganOutput<u32, Up<Up<Base>>>      (tier 2)

    // 2-input organ reading tiers 0 and 2: max = 2, so it writes at tier 3 = Up<Up<Up<Base>>>.
    // The annotation PINS the output tier — if `Max` computed the wrong tier, this would not compile.
    let two: OrganOutput<u32, Up<Up<Up<Base>>>> = write_back2(&t0, &t2, 100u32);
    assert_eq!(*two.value(), 100u32);

    // Order-independence: swapping the two reads yields the SAME output tier (max is symmetric).
    let two_swapped: OrganOutput<u32, Up<Up<Up<Base>>>> = write_back2(&t2, &t0, 100u32);
    assert_eq!(*two_swapped.value(), 100u32);

    // Equal-tier inputs: max(1, 1) = 1, so the organ writes at tier 2.
    let t1b = write_back(&t0, 21u32); // second tier-1 input
    let two_equal: OrganOutput<u32, Up<Up<Base>>> = write_back2(&t1, &t1b, 200u32);
    assert_eq!(*two_equal.value(), 200u32);

    // 3-input organ reading tiers 0, 1, 2: max = 2, so it writes at tier 3 = Up<Up<Up<Base>>>.
    let three: OrganOutput<u32, Up<Up<Up<Base>>>> = write_back3(&t0, &t1, &t2, 300u32);
    assert_eq!(*three.value(), 300u32);
}

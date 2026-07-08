// ATK-SIGNAL-B2PRIME-EFFERENT-IN-AFFERENT (ADR-071 B2′; research nb 022 §11.4) — a signed EFFERENT
// output MUST NOT be consumable by the afferent write door. `write_back` takes `&OrganOutput` (an
// afferent, monotone value), never an `EfferentChannel` (a signed / cancelling actuator output), so a
// `Should − Is` inverse can never enter the afferent semiring. This is the wave-optics MRT Re/Im
// channel-split as a type: the efferent computation is a SEPARATE output target, not a signed value in
// the afferent accumulator — "you may have the inverse, but only in a channel whose afferent projection
// is non-negative."
//
// TEETH: the sound path (`ui_pass/organ_output_climbs_the_tower`) feeds a real `OrganOutput` and
// compiles; only the efferent channel is rejected here.

use antigen_stroma::signal::{write_back, EfferentChannel};

fn main() {
    let efferent = EfferentChannel::new(1u32);
    // Feeding an efferent (signed) channel into the afferent write door MUST NOT compile.
    let _ = write_back(&efferent, 2u32);
}

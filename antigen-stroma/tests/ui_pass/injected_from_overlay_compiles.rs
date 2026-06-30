// ATK-FRAME-INJECT-FROM-OVERLAY · NEGATIVE CONTROL (must COMPILE) — the sole legitimate door.
//
// Proves the type-state is not a blanket ban: `InjectedException::from_overlay(&overlay)` is the ONE
// constructor and it type-checks. If this failed to compile, the invariant would forbid ALL injected
// exceptions (useless) rather than only the overlay-less ones.
//
// NOTE: from_overlay's BODY is a frame-epoch todo!() — this fixture proves the call TYPE-CHECKS (the
// door exists and takes an &OverlayMarker), not that it runs. The compile-state class asserts the
// constructor surface; the runtime behavior is a separate (born-red) concern.

use antigen_stroma::write::{InjectedException, OverlayMarker};

// A reference to an OverlayMarker is all the door requires at the type level. We never call it (the
// body is todo!()); we only assert the construction path type-checks through from_overlay.
fn _door_typechecks(overlay: &OverlayMarker) -> fn(&OverlayMarker) -> InjectedException {
    let _ = overlay;
    InjectedException::from_overlay
}

fn main() {}

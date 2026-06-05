//! Marked-Unknown markers — `#[aura]` / `#[dread]` / `#[red_flag]` (ADR-041).
//!
//! The single most perishable piece of knowledge in software is the *felt-but-
//! unnamed danger*: the unease that something is wrong here that evaporates the
//! moment you move on. These three markers let you record it **structurally**,
//! at the site, before it's gone — without having to name the failure-class yet.
//!
//! They sit on a 2-D plane — **magnitude** × **existence-certainty** — and each
//! one *fixes* its corner; you supply only the one **required** field, `trigger`
//! (what you saw that made you feel this):
//!
//! - `#[aura]` — low magnitude: "something *may* be off, check later."
//! - `#[dread]` — high magnitude, low certainty (*angor animi*): "something *is*
//!   wrong here, I can't name it, look now."
//! - `#[red_flag]` — high existence-certainty: "I'm *sure* something is wrong,
//!   can't name it, act now." Auto-escalates the *internal* Finding's severity
//!   (`Sure` → `High`); see "What surfaces where" below for the projection caveat.
//!
//! Run:
//!
//! ```sh
//! cargo run --example marked_unknown --package antigen
//! ```
//!
//! See the markers in the scan's structured `Finding` stream:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples --format json
//! ```
//!
//! ## What surfaces where (be honest with yourself)
//!
//! Each marker stamps a discoverable doc-marker the scanner reads and surfaces in
//! the JSON report under the top-level `report.marked_unknowns` array (fields:
//! `marker`, `magnitude`, `existence_certainty`, `trigger`, `file`). All three are
//! PASSIVE records — none gates CI, fails the build, or alerts. Internally each
//! also emits a `FindingBody::MarkedUnknown` record carrying a `severity` field
//! (`#[red_flag]`/`Sure` and `#[dread]` → `High`, `#[aura]` → `Medium`) — a routing
//! field for a future cytokine-routing organ (chartered), so `#[red_flag]`'s
//! "escalation" is just that it records at `High`, not that anything fires. That
//! severity lives on the internal Finding, NOT on the scan-report projection, whose
//! `severity` reads `null` today. The
//! human-readable scan report does **not** render marked-unknowns yet (the
//! audit-time confidence dial that surfaces them is a later wave) — so today a
//! marker is a structural record you query (`--format json`), not a console line.
//! The mark is never lost; that is the whole point.

// The marked functions have placeholder bodies (the example teaches the
// *markers*, not the function logic); clippy would suggest making them `const`,
// which is noise here.
#![allow(clippy::missing_const_for_fn)]

use antigen::{aura, dread, red_flag};

/// `#[aura]` — the light corner. A mild substrate-smell: it never gates, never
/// nags; it's a note-to-future-self that something *might* be off here.
#[aura(trigger = "this retry loop has no jitter; under load it might thundering-herd")]
pub fn retry_request() {
    // ... a retry with no backoff jitter ...
}

/// `#[dread]` — high magnitude, low certainty. You can't prove the bug, but the
/// shape feels wrong and the cost-if-real is high. Look now.
#[dread(trigger = "the teardown drops the guard before the flush; \
                   I can't prove a leak but the ordering feels wrong")]
pub fn shutdown() {
    // ... teardown whose ordering you're uneasy about ...
}

/// `#[red_flag]` — high existence-certainty, still unnameable.
///
/// You're *sure* this is exploitable; you just can't yet pin the exact class.
/// Records at the internal Finding's `High` severity (not the scan-report
/// projection — see the header); never gates, never alerts.
#[red_flag(trigger = "this auth check can be reached with an empty token in \
                      the cache-hit path; I'm sure this is exploitable")]
pub fn authorize(_token: &str) -> bool {
    // ... an auth path you're certain is wrong ...
    true
}

// ── The graffiti guard (ADR-041 guard 3): `trigger` is REQUIRED ──────────────
//
// A marked-unknown with no stated trigger is contentless "this seems off"
// graffiti — exactly what the markers exist to prevent — so it is a COMPILE
// ERROR, not a silent no-op. Both of these fail to compile (uncomment to see the
// error: "#[dread] requires `trigger = \"...\"` — what did you see that made you
// feel this?"):
//
//     #[dread]                 // ← rejected: no trigger
//     fn no_trigger() {}
//
//     #[aura(trigger = "")]    // ← rejected: empty trigger
//     fn empty_trigger() {}

fn main() {
    retry_request();
    shutdown();
    let _ = authorize("");
    println!(
        "marked_unknown example: 3 markers declared (aura/dread/red_flag). \
         Run `cargo antigen scan --root antigen/examples --format json` to see \
         them under report.marked_unknowns."
    );
}

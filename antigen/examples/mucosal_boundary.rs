//! Example: the Mucosal Boundary Family (ADR-027 + Amendment 1).
//!
//! Mucosal boundaries are the trust surfaces where a system meets the
//! outside world with SELECTIVE permeability — not the binary trust/distrust
//! of a hard skin barrier, but graded admission with active discipline. Per
//! ADR-027 the biology grounds the TIER-CLAIM (mucosal surfaces are a
//! distinct immune tier) + four functional disciplines, NOT per-variant
//! tissue mapping.
//!
//! Three response states (ADR-027 Amendment 1 Change 6), parallel to
//! ADR-016's immune/tolerance/undeclared triad at the boundary tier:
//!
//! - `#[mucosal]` — ACTIVE DEFENSE at the boundary.
//! - `#[mucosal_delegate]` — defense DELEGATED to a named handler that must
//!   itself carry a matching `#[mucosal(kind = X)]` (Change 5 kind-matching).
//! - `#[mucosal_tolerant]` — ACTIVE TOLERANCE: the boundary intentionally
//!   permits input, documented with a higher (≥40-char) rationale floor.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example mucosal_boundary --package antigen
//! cargo run --bin cargo-antigen -- antigen mucosal-map --root antigen/examples
//! cargo run --bin cargo-antigen -- antigen mucosal-map --tolerant
//! ```

// The boundary handler functions are illustration sites; some have trivial
// bodies that trip missing_const_for_fn. Example-scoped allow (does not
// affect the library crate).
#![allow(clippy::missing_const_for_fn)]

use antigen::{mucosal, mucosal_delegate, mucosal_tolerant};

// ============================================================================
// #[mucosal] — active boundary defense.
// ============================================================================

/// The webhook intake handler is the defended boundary for inbound API
/// requests carrying caller-supplied payloads.
#[mucosal(
    kind = MucosalKind::ApiRequest,
    rationale = "Inbound webhook endpoint; HMAC signature verified + payload schema-validated before dispatch."
)]
pub fn handle_webhook(_payload: &[u8]) -> Result<(), String> {
    Ok(())
}

/// The central user-input sanitizer — the handler that delegate sites point
/// at. Carries `#[mucosal(kind = UserInput)]` so the delegate kind-matching
/// (Change 5c) resolves.
#[mucosal(
    kind = MucosalKind::UserInput,
    rationale = "Central user-input sanitizer: HTML-escapes, length-bounds, and rejects control characters."
)]
pub fn sanitize_user_input(raw: &str) -> String {
    raw.replace('<', "&lt;").replace('>', "&gt;")
}

// ============================================================================
// #[mucosal_delegate] — defense delegated to a named handler.
// ============================================================================

/// The comment-form endpoint delegates its user-input boundary discipline.
///
/// Delegates to the central sanitizer above. The handler carries a matching
/// `#[mucosal(kind = UserInput)]`, so the three-tier delegate audit passes.
#[mucosal_delegate(
    boundary = MucosalKind::UserInput,
    handled_by = sanitize_user_input,
    rationale = "Comment-form input delegated to the shared central sanitizer routine."
)]
pub fn submit_comment(body: &str) -> String {
    sanitize_user_input(body)
}

// ============================================================================
// #[mucosal_tolerant] — active tolerance (intentional permission).
// ============================================================================

/// A public anonymous-feedback firehose.
///
/// DELIBERATELY accepts unauthenticated input by design — active tolerance,
/// not absence of defense — documented with the higher ≥40-char rationale
/// floor + a named reviewer.
#[mucosal_tolerant(
    kind = MucosalKind::UserInput,
    rationale = "Public anonymous-feedback firehose; accepts unauthenticated submissions by design because requiring auth would defeat the feature's purpose.",
    accepts = "Anonymous JSON feedback payloads up to 64KB; rate-limited per-IP.",
    reviewed_by = "trust-and-safety",
    until = "2026-12-31"
)]
pub fn anonymous_feedback_intake(_payload: &[u8]) -> Result<(), String> {
    Ok(())
}

fn main() {
    println!("=== antigen mucosal-boundary family example ===");
    println!();
    println!("Three response states demonstrated:");
    println!("  #[mucosal]           — handle_webhook (active defense, ApiRequest)");
    println!("  #[mucosal]           — sanitize_user_input (delegate target, UserInput)");
    println!("  #[mucosal_delegate]  — submit_comment → sanitize_user_input (kind-matched)");
    println!("  #[mucosal_tolerant]  — anonymous_feedback_intake (active tolerance)");
    println!();
    println!("Run `cargo antigen mucosal-map` to map the boundaries;");
    println!("`--tolerant` lists the active-tolerance boundaries for reviewer audit.");

    let _ = handle_webhook(b"event");
    let _ = sanitize_user_input("<script>");
    let _ = submit_comment("hello <b>world</b>");
    let _ = anonymous_feedback_intake(b"feedback");
}

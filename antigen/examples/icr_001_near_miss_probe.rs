//! ICR-001 reproduction probe — the GATE-G near-miss verdict on the Hole-I shape.
//!
//! Demonstrates, against the COMMITTED (Amendment-2-fixed) `is_near_miss`, that a
//! single-discriminator draft `[impl, Drop, body_calls("aaa")]` is NOT a near-miss
//! of a bare `Drop` impl that shares only the structural skeleton — so the gate
//! routes-to-human instead of fabricating a near-miss and promoting (the vacuity
//! hole the gate's own audit found). Run:
//!
//! ```sh
//! cargo run --example icr_001_near_miss_probe --package antigen
//! ```

use antigen::learn::self_tolerance::{has_discriminating_conjunct, is_near_miss};
use antigen_fingerprint::{Constraint, Fingerprint, ItemKind};

fn drop_impl(src: &str) -> syn::Item {
    syn::parse_str(src).expect("parses")
}

fn main() {
    // The Hole-I draft: the WHOLE discriminating signal is ONE conjunct.
    let draft = Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
            Constraint::BodyCalls("aaa".into()),
        ],
    };

    // A bare `Drop` impl that does NOT call `aaa` — shares ONLY the skeleton.
    let bare = drop_impl("impl Drop for W { fn drop(&mut self) { let _ = 1; } }");

    println!(
        "draft has_discriminating_conjunct = {}",
        has_discriminating_conjunct(&draft)
    );
    println!(
        "draft.matches(bare)               = {}",
        draft.matches(&bare)
    );
    println!(
        "is_near_miss(draft, bare)         = {}",
        is_near_miss(&draft, &bare)
    );

    // The remainder after dropping the sole discriminator — what the FIX inspects.
    let remainder = Fingerprint {
        constraints: vec![
            Constraint::Item(ItemKind::Impl),
            Constraint::ImplOfTrait("Drop".into()),
        ],
    };
    println!(
        "remainder [impl, Drop] discriminates = {}  <- Amd2: a near-miss remainder MUST discriminate",
        has_discriminating_conjunct(&remainder)
    );
}

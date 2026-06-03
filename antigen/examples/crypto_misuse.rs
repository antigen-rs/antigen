//! Crypto-Misuse family — the admitting-specimen (affinity-pair exhibit).
//!
//! This example is the **admitting-specimen** for
//! [`antigen::stdlib::crypto_misuse::NonConstantTimeSecretComparison`] — the
//! affinity-pair (a failing case the fingerprint *binds* + a clean sibling it
//! must *not* bind) that the ADR-039 §C worth-multiplier requires. It is also
//! the masterclass exhibit and the dogfood self-catch: scanning this file finds
//! the vulnerable site and spares the safe one.
//!
//! Run:
//!
//! ```sh
//! cargo run --example crypto_misuse --package antigen
//! ```
//!
//! Scan it to see the affinity-pair separate (the bad binds, the good is spared):
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```
//!
//! ## The fingerprint, in one line
//!
//! `all_of([body_calls("verify"), not(body_calls("ct_eq"))])` — a crypto
//! `verify` path **without** a constant-time comparison present. The *absence*
//! of `ct_eq` is the tell; the safe sibling calls `ct_eq` and is spared by the
//! `not` branch.
//!
//! ## BIOSAFETY NOTE
//!
//! The "bad" path below is a *toy stand-in*, not real crypto — it exists purely
//! to exhibit the call-shape the fingerprint matches. The constant-time
//! functions are local stubs (no `subtle`/`ring` dependency in the example).
//! Do **not** copy the bad path; it is the specimen of what NOT to do.

use antigen::{antigen, defended_by, presents};

/// A secret/MAC compared through a non-constant-time path — a timing oracle.
///
/// Drop impls must not panic; secret comparisons must not leak timing. Comparing
/// a MAC / token byte-by-byte (or via `==`) short-circuits on the first mismatch,
/// so the comparison time correlates with how many leading bytes matched — an
/// attacker recovers the secret one byte at a time.
#[antigen(
    name = "non-constant-time-secret-comparison",
    category = AntigenCategory::FunctionalCorrectness,
    fingerprint = r#"all_of([body_calls("verify"), not(body_calls("ct_eq"))])"#,
    family = "crypto-misuse",
    summary = "A crypto verify path compares a secret/MAC/token without a constant-time comparison present — a timing-attack oracle.",
    references = ["https://arxiv.org/abs/1806.04929", "RUSTSEC#crypto-failure"],
)]
pub struct NonConstantTimeSecretComparison;

/// Toy constant-time-equality stub — stands in for `subtle::ConstantTimeEq`.
/// In real code this is the safe step whose **presence** spares the site.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    // A real ct_eq folds every byte with no early return; this toy keeps the
    // call-shape (the `ct_eq` callee) so the fingerprint's safe-branch fires.
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// The crypto verify entrypoint the fingerprint anchors on (toy stand-in).
fn verify(provided: &[u8], expected: &[u8]) -> bool {
    // Stand-in for a real MAC/tag verify. The fingerprint anchors on the
    // `verify` call in the bodies below, not on this definition.
    provided == expected
}

/// BAD (the bind): verifies a MAC with **no** constant-time comparison present.
///
/// `body_calls("verify")` matches AND `not(body_calls("ct_eq"))` matches
/// (`ct_eq` is absent) → the `all_of` **binds**. This is the vulnerable specimen.
#[presents(NonConstantTimeSecretComparison)]
fn check_mac_vulnerable(provided: &[u8], expected: &[u8]) -> bool {
    // Non-constant-time: `verify` here short-circuits; no ct_eq guard.
    verify(provided, expected)
}

/// GOOD (the spare): verifies through the constant-time comparison.
///
/// `body_calls("verify")` matches, but `not(body_calls("ct_eq"))` does NOT
/// (`ct_eq` IS present) → the `all_of` is **spared**. The presence of the safe
/// step is exactly what the absence-grammar tell looks for.
#[presents(NonConstantTimeSecretComparison)]
fn check_mac_safe(provided: &[u8], expected: &[u8]) -> bool {
    // Constant-time: route the comparison through ct_eq.
    let _ = verify(provided, expected);
    ct_eq(provided, expected)
}

/// Witness: proves the safe path uses the constant-time comparison.
/// `#[defended_by]` declares this test's intent toward the failure-class; audit
/// observes whether the circuit covers the safe site.
#[allow(dead_code)]
#[defended_by(NonConstantTimeSecretComparison)]
fn safe_mac_uses_constant_time_test() {
    let secret = b"correct-horse";
    assert!(check_mac_safe(secret, secret));
    assert!(!check_mac_safe(secret, b"wrong"));
}

fn main() {
    println!("antigen crypto-misuse example: see source for the affinity-pair.");
    println!("Run `cargo run --bin cargo-antigen -- antigen scan` to see the bad path flagged and the safe path spared.");

    let secret = b"correct-horse";
    // Exercise both so the example is functional.
    let _ = check_mac_vulnerable(secret, secret);
    let _ = check_mac_safe(secret, secret);
    safe_mac_uses_constant_time_test();
}

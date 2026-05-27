//! Phantom-type witness example: ADR-013.
//!
//! Phantom-type witnesses encode the immunity proof in the type system
//! itself — a value of `NonPanickingProof::<DropImpl>` exists ONLY if a
//! `verified()` constructor produced it, and the constructor's existence
//! is the proof. This is a compile-time witness: if the code compiles,
//! the proof holds.
//!
//! `cargo antigen audit` recognizes phantom-type witnesses by the turbofish
//! syntax (`Foo::<T>::constructor`) in the `witness = ...` argument. The
//! audit classifies them at `WitnessTier::FormalProof` — the strongest
//! tier the audit can ascribe at reachability-level analysis. Behavioral
//! verification (does the constructor actually encode a sealed proof?) is
//! A4-A5 work; the v0.1 audit emits an `AuditHint::PhantomTypeShapeRecognized`
//! hint reminding the user that the shape is recognized but the constructor's
//! soundness is the developer's responsibility to seal.
//!
//! Run with JSON output to see the `FormalProof` tier classification:
//!
//! ```sh
//! cargo run --bin cargo-antigen -- antigen audit --root antigen/examples --format json
//! ```
//!
//! In the JSON output, look for an audit entry with `"witness_tier": "formal_proof"`
//! and `"hint": "PhantomTypeShapeRecognized"`. The human-readable `audit` output
//! only shows warnings for claims *below* Execution tier — a `FormalProof` claim
//! does not appear in warnings because it is not a warning. Use `--format json` to
//! inspect the full classification.

#![allow(dead_code)]

use antigen::{antigen, presents};
use std::marker::PhantomData;

#[antigen(
    name = "drop-panic-class",
    family = "boundary-violation",
    fingerprint = r#"name = matches("Phantom*DropImpl")"#,
    summary = "Drop impls must not panic; panic-during-unwind aborts the process."
)]
pub struct DropPanicClass;

/// Phantom-type proof token: a value of `NonPanickingProof::<T>` exists
/// ONLY if the `verified()` constructor produced it.
///
/// The constructor's existence is the proof. Callers cannot construct an
/// instance directly because the field is private and there are no public
/// free constructors.
///
/// In real code the constructor would carry additional guarantees (e.g.,
/// kani-verified, or a sealed-by-trait-bound invariant). This example shows
/// the shape; the soundness is the developer's responsibility to seal.
pub struct NonPanickingProof<T> {
    _marker: PhantomData<T>,
    _seal: (),
}

impl<T> NonPanickingProof<T> {
    /// Constructs a proof token. In production code this would only succeed
    /// after a verification step (kani run, proptest exhaustion, etc.). Here
    /// the construction is unconditional to keep the example focused on
    /// shape-recognition.
    #[must_use]
    pub const fn verified() -> Self {
        Self {
            _marker: PhantomData,
            _seal: (),
        }
    }
}

/// The type whose `Drop` impl is provably non-panicking via the phantom token.
pub struct PhantomVerifiedDropImpl;

/// ADR-029: phantom-type proof goes in `proof=` on `#[presents]`.
/// The audit recognizes turbofish-shaped `proof=` expressions and classifies
/// them at `WitnessTier::FormalProof`. Immunity is observed by audit, not
/// declared by `#[immune]`.
#[presents(
    DropPanicClass,
    proof = NonPanickingProof::<PhantomVerifiedDropImpl>::verified
)]
impl Drop for PhantomVerifiedDropImpl {
    fn drop(&mut self) {
        // GOOD: no unwrap, no expect, no panic. The phantom proof asserts
        // this property at compile time.
    }
}

fn main() {
    println!("antigen phantom-witness example: type-system-encoded immunity proof.");
    println!();
    println!("Run `cargo run --bin cargo-antigen -- antigen audit --root antigen/examples`");
    println!("to see the immunity classified at WitnessTier::FormalProof with the");
    println!("PhantomTypeShapeRecognized hint.");
}

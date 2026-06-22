// Migrated to ADR-029 idiom: #[presents(requires=...)] replaces the deprecated
// #[immune(requires=...)] API. (forward/audit-json-schema-immunities-to-presentation-verdicts)

//! Substrate-witness example — declare a discipline-antigen with a compound
//! `requires` predicate, scaffold a sidecar, sign it, and watch the audit
//! verdict climb. (ADR-019 substrate-witness predicate family.)
//!
//! # What this example demonstrates
//!
//! - Declare `SignedZeroDiscipline` antigen with full doc-comment so the
//!   memory of WHY this matters is captured at the type level
//! - Mark `signed_zero_preserving_sinh` with `#[presents(..., requires=...)]`
//!   carrying a compound substrate predicate (ADR-029 observe-don-t-declare idiom)
//! - Show the four-step operator workflow (declare -> mark -> scaffold -> sign)
//! - Audit-verdict: without a sidecar reports substrate-gap; with signed sidecar
//!   reports defended at Execution tier.

#![allow(dead_code, unused_imports)]

use antigen::{antigen, presents};

/// Odd functions must preserve IEEE 754 signed zero.
#[antigen(
    name = "signed-zero-discipline",
    family = "forgotten-lesson",
    fingerprint = r#"all_of([item = fn, any_of([name = matches("sinh*"), name = matches("tanh*"), name = matches("sin*"), name = matches("asin*"), name = matches("atan*"), name = matches("asinh*"), name = matches("atanh*")])])"#,
    summary = "Odd mathematical functions must preserve IEEE 754 signed zero.",
    references = [
        "https://github.com/antigen-rs/antigen/blob/main/docs/origin.md",
        "IEEE 754-2019 §5.5.1 (signed-zero semantics)",
    ],
)]
pub struct SignedZeroDiscipline;

/// A sinh implementation that preserves signed zero.
///
/// The `#[presents(..., requires=...)]` declaration binds the function to a
/// compound substrate predicate (ADR-029 observe-don-t-declare idiom):
///
/// | Sidecar state | Verdict |
/// |---|---|
/// | Missing | substrate-gap |
/// | Pred passes, all current | defended (Execution) |
#[presents(
    SignedZeroDiscipline,
    requires = all_of([
        signers(required = ["alice"], roles = {alice = "reviewer"}),
        ratified_doc(path = "docs/disciplines/ieee754-odd-functions.md", min_version = "1.0"),
        fresh_within_days(days = 180),
    ])
)]
pub fn signed_zero_preserving_sinh(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return x;
    }
    if x == 0.0 {
        return x;
    }
    let ax = x.abs();
    let sign = if x.is_sign_negative() { -1.0 } else { 1.0 };
    let t = ax.exp_m1();
    sign * t * (t + 2.0) / (2.0 * (t + 1.0))
}

/// A naive sinh that does NOT preserve signed zero.
#[presents(SignedZeroDiscipline)]
pub fn naive_sinh_loses_sign_at_zero(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return x;
    }
    let ax = x.abs();
    let sign = if x.is_sign_negative() { -1.0 } else { 1.0 };
    let t = ax.exp_m1();
    sign * t * (t + 2.0) / (2.0 * (t + 1.0))
}

fn main() {
    println!("antigen substrate-witness example -- signed-zero discipline at sinh.");
    let x = -0.0_f64;
    let r1 = signed_zero_preserving_sinh(x);
    let r2 = naive_sinh_loses_sign_at_zero(x);
    println!(
        "  signed_zero_preserving_sinh(-0.0) bits = {:#018x}",
        r1.to_bits()
    );
    println!(
        "  naive_sinh_loses_sign_at_zero(-0.0) bits = {:#018x}",
        r2.to_bits()
    );
}

//! Example: `#[diagnostic]` — multi-modality convergent evidence.
//!
//! ADR-024 convergent-evidence family. `#[diagnostic]` asserts that a
//! defense rests on multiple *distinct* `WitnessClass` categories
//! converging on the same conclusion. The
//! `min_independent` count is over distinct CLASSES, not raw witness
//! count — running the same kind of test three times doesn't add
//! evidence.
//!
//! ## When to use `#[diagnostic]`
//!
//! - You have property-tests AND formal proofs AND manual review on a
//!   load-bearing function
//! - You want the structural memory of "this is defended via N distinct
//!   modalities" rather than "we wrote tests"
//! - You want CI to fail if the modality count silently drops
//!
//! ## Biological cognate (per ADR-024 §dual-axis biology)
//!
//! Clinical-medicine — a diagnostic workup integrates symptoms,
//! imaging, blood panels, biopsy. Each modality contributes
//! independent information; convergence raises confidence above any
//! single result.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example convergent_diagnostic --package antigen
//! ```

// Note: `WitnessClass` is referenced inside the #[diagnostic] macro args
// as a path that the macro tokenizes; not a real Rust value path.
#[allow(unused_imports)]
use antigen::WitnessClass;
use antigen::{antigen, diagnostic};

#[antigen(
    name = "checked-arithmetic-overflow",
    fingerprint = r#"item = fn, name = matches("*_sum")"#,
    family = "boundary-violation",
    summary = "Arithmetic operation may overflow; checked variant must be used."
)]
pub struct CheckedArithmeticOverflow;

/// Sum two i64 values, returning None on overflow.
///
/// Defense converges via:
/// 1. **`PropertyTest`** — proptest harness `prop_sum_never_panics`
/// 2. **`FormalVerification`** — Kani proof `kani_sum_bounds`
/// 3. **`ManualReview`** — PR review attested in commit history
///
/// Per ADR-024: `min_independent = 3` requires
/// THREE distinct `WitnessClass` categories. The macro rejects
/// `min_independent` exceeding the number of distinct categories
/// supplied (vacuously unsatisfiable claim).
#[diagnostic(
    modalities = [
        WitnessClass::PropertyTest,
        WitnessClass::FormalVerification,
        WitnessClass::ManualReview,
    ],
    min_independent = 3,
)]
pub const fn checked_sum(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}

/// A weaker defense with only one distinct category.
///
/// Two proptests and one fuzzing run all map to "randomized
/// exploration" — collapse to a single `PropertyTest` class. The
/// audit hint `diagnostic-modality-insufficient` fires because the
/// distinct CLASS count (1) is below `min_independent = 2`.
#[diagnostic(
    modalities = [WitnessClass::PropertyTest, WitnessClass::PropertyTest],
    min_independent = 1,
)]
pub const fn weaker_sum(a: i64, b: i64) -> Option<i64> {
    a.checked_add(b)
}

fn main() {
    println!("=== antigen convergent-evidence: #[diagnostic] example ===");
    println!();
    println!("Two diagnostic declarations:");
    println!();
    println!("1. checked_sum");
    println!("   modalities: PropertyTest, FormalVerification, ManualReview");
    println!("   min_independent: 3 (all three distinct categories present)");
    println!("   audit: clean");
    println!();
    println!("2. weaker_sum");
    println!("   modalities: PropertyTest, PropertyTest (duplicate class)");
    println!("   min_independent: 1");
    println!("   audit: diagnostic-modalities-class-collapsed warning");
    println!("          (classes, not raw count)");
    println!();
    println!("Per ADR-024:");
    println!("  min_independent COUNTS DISTINCT CATEGORIES, not witnesses.");
    println!("  Three property-tests = ONE PropertyTest class = ONE independent.");
    println!();
    println!("Sample evaluations:");
    println!("  checked_sum(100, 200) = {:?}", checked_sum(100, 200));
    println!(
        "  checked_sum(i64::MAX, 1) = {:?}",
        checked_sum(i64::MAX, 1)
    );
    println!("  weaker_sum(5, 7) = {:?}", weaker_sum(5, 7));
}

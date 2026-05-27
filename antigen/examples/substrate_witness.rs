// ADR-029 deprecation-window: uses the deprecated-but-functional #[immune] API;
// full migration to #[defended_by]/#[presents(requires=)] is a tracked follow-on.
#![allow(deprecated)]

//! Substrate-witness example — declare a discipline-antigen with a compound
//! `requires` predicate, scaffold a sidecar, sign it, and watch the audit
//! tier climb. (ADR-019 substrate-witness predicate family.)
//!
//! # The story
//!
//! You're maintaining a numerical library. One day you discover that someone
//! shipped a `sinh` whose body returns `+0.0` for `sinh(-0.0)` — silently
//! dropping the IEEE 754 sign bit. The bug propagated through a complex-
//! derivative path; a downstream user spent two days chasing it.
//!
//! You fix the bug. You also notice it's a CLASS of bug: every odd function
//! has the same contract (`f(-0.0) = -0.0`, sign-bit preserved). `tanh`,
//! `sin`, `asin`, `atan`, `asinh`, `atanh` — same shape, same risk.
//!
//! You COULD write a test for each one. You did. But tests sit in test files
//! that may or may not be maintained; the contract isn't visible from the
//! function's declaration site. The next refactor that "simplifies" the
//! function body could silently elide the `if x == 0.0 { return x; }`
//! short-circuit and the test could pass anyway (because a constant-folded
//! `0.0` is value-equal to `-0.0`, just bit-different).
//!
//! What you want is a STRUCTURAL pointer at the declaration site that says:
//! "this function is bound to a discipline; here's the discipline's record;
//! here's WHO reviewed compliance; here's WHEN it expires." That's the
//! substrate-witness primitive.
//!
//! # What this example demonstrates
//!
//! - Declare `SignedZeroDiscipline` antigen with full doc-comment so the
//!   memory of WHY this matters is captured at the type level
//! - Mark `signed_zero_preserving_sinh` with `#[immune(...)]` carrying a
//!   compound `requires` predicate (`all_of` [`signers`, `ratified_doc`,
//!   `fresh_within_days`])
//! - Show the four-step operator workflow (declare → mark → scaffold → sign)
//! - The audit-tier story: without a sidecar the audit reports
//!   `discipline-sidecar-missing` at `Reachability`; with a signed sidecar
//!   it reports `discipline-predicate-passed-substrate-current` at
//!   `Execution` with `EvidenceKind::SubstrateState`.
//!
//! Run:
//!
//! ```sh
//! cargo run --example substrate_witness --package antigen
//!
//! # See what the scan reports:
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//!
//! # Audit the example (after scaffolding + signing the sidecar):
//! cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
//! ```

#![allow(dead_code, unused_imports)]

use antigen::{antigen, immune, presents};

/// Odd functions must preserve IEEE 754 signed zero.
///
/// For every odd function `f`, `f(-0.0)` MUST return `-0.0` (sign-bit
/// preserved), not `+0.0`. The IEEE 754 contract is `sign(f(-x)) = -sign(f(x))`
/// for ALL x including ±0. The bit-pattern distinction is load-bearing —
/// `-0.0 == 0.0` is true in value semantics, but `(-0.0_f64).to_bits()
/// != 0.0_f64.to_bits()` and downstream code that branches on
/// `x.is_sign_negative()` after composition gets the wrong branch.
///
/// **Failure mode**: a "simplification" refactor elides the
/// `if x == 0.0 { return x; }` short-circuit OR replaces `return x` with
/// `return 0.0`. The function still satisfies most tests (value-equal
/// passes); only a bit-exact `to_bits()` assertion catches the sign loss.
///
/// **Family**: forgotten-lesson. The IEEE 754 contract is well-documented
/// but easy to forget when reading code that LOOKS like it preserves sign
/// by accident-of-arithmetic.
#[antigen(
    name = "signed-zero-discipline",
    family = "forgotten-lesson",
    fingerprint = r#"all_of([item = fn, any_of([name = matches("sinh*"), name = matches("tanh*"), name = matches("sin*"), name = matches("asin*"), name = matches("atan*"), name = matches("asinh*"), name = matches("atanh*")])])"#,
    summary = "Odd mathematical functions (sinh, tanh, sin, asin, atan, asinh, atanh, …) \
               MUST preserve IEEE 754 signed zero — f(-0.0) returns -0.0, not +0.0, with \
               the sign bit intact (bit-exact, not just value-equal). Sign-bit loss \
               propagates through derivative computations and branch decisions downstream.",
    references = [
        "https://github.com/antigen-rs/antigen/blob/main/docs/origin.md",
        "IEEE 754-2019 §5.5.1 (signed-zero semantics)",
    ],
)]
pub struct SignedZeroDiscipline;

/// A `sinh` implementation that preserves signed zero per
/// `SignedZeroDiscipline`.
///
/// The `if x == 0.0 { return x; }` short-circuit at the top is the
/// load-bearing line — without it, the `0.5 * (e.exp() - (-x).exp())`
/// arithmetic loses the sign at zero (returns `+0.0` for `sinh(-0.0)`).
///
/// The `#[immune]` claim binds the function to a compound predicate:
///
/// - `signers(required = ["alice"], roles = {alice = "math-researcher"})` —
///   signer `alice` must have reviewed with role `math-researcher` (audit-time
///   check against the sidecar's `signers` list; `roles` asserts role identity,
///   not just name presence)
/// - `ratified_doc(path = "docs/disciplines/ieee754-odd-functions.md",
///    min_version = "1.0")` — a discipline doc at that path must exist
///   with frontmatter `version >= 1.0`
/// - `fresh_within_days(days = 180)` — the most recent signer's date must
///   be within 180 days of audit-evaluation time
///
/// All three leaves must pass (`all_of`). The audit reports the result at
/// the right `WitnessTier` per ADR-019 §M5:
///
/// | Sidecar state | Tier | Hint |
/// |---|---|---|
/// | Missing | `Reachability` | `discipline-sidecar-missing` |
/// | Schema-invalid | `None` | `discipline-sidecar-schema-invalid` |
/// | Predicate fails | `None` | `discipline-predicate-failed` |
/// | Pred passes, signer stale | `Reachability` | `discipline-substrate-stale` |
/// | Pred passes, all current | `Execution` | `discipline-predicate-passed-substrate-current` |
#[immune(
    SignedZeroDiscipline,
    requires = all_of([
        signers(required = ["alice"], roles = {alice = "math-researcher"}),
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
    // Load-bearing line: preserves the sign bit at zero. sinh(-0.0) = -0.0.
    if x == 0.0 {
        return x;
    }
    // Small-x stable formula: t * (t + 2) / (2 * (t + 1)) where t = e^|x| - 1.
    // For |x| < 22 this is cancellation-free; the example uses std `exp_m1`.
    let ax = x.abs();
    let sign = if x.is_sign_negative() { -1.0 } else { 1.0 };
    let t = ax.exp_m1();
    sign * t * (t + 2.0) / (2.0 * (t + 1.0))
}

/// A naive `sinh` that DOES NOT preserve signed zero.
///
/// This is the failure the antigen names — same shape, same fingerprint
/// match, no short-circuit at zero. The function is value-correct for
/// nonzero inputs but silently violates the IEEE 754 odd-function
/// contract at ±0.
///
/// The `#[presents]` marker declares "this function exhibits the
/// failure-class" without claiming immunity. The audit reports the site
/// as `unaddressed-presentation` — operator-facing prompt to either add
/// an `#[immune]` with witness, mark `#[antigen_tolerance]` with rationale,
/// or fix the bug.
#[presents(SignedZeroDiscipline)]
pub fn naive_sinh_loses_sign_at_zero(x: f64) -> f64 {
    // BAD: no zero-short-circuit. For x = -0.0, ax = 0.0, t = 0.0, and the
    // expression evaluates to `sign * 0.0 * 2.0 / (2.0 * 1.0) = -1.0 * 0.0
    // = -0.0` BY ACCIDENT-OF-ARITHMETIC. But a "simplification" that drops
    // the `sign` multiplication (correct for nonzero) would return `+0.0`
    // for `naive_sinh_loses_sign_at_zero(-0.0)` and the bug ships silently.
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
    println!("antigen substrate-witness example — signed-zero discipline at sinh.");
    println!();
    println!("Compare the two implementations:");
    let x = -0.0_f64;
    let immune_result = signed_zero_preserving_sinh(x);
    let naive_result = naive_sinh_loses_sign_at_zero(x);
    println!(
        "  signed_zero_preserving_sinh(-0.0).to_bits() = {:#018x}",
        immune_result.to_bits()
    );
    println!(
        "  naive_sinh_loses_sign_at_zero(-0.0).to_bits() = {:#018x}",
        naive_result.to_bits()
    );
    println!(
        "  expected (-0.0_f64).to_bits()                = {:#018x}",
        (-0.0_f64).to_bits()
    );
    println!();
    println!("Both functions compute the same value for nonzero inputs:");
    for &probe in &[0.1_f64, 1.0, 2.0, -1.0, -2.0] {
        println!(
            "  sinh({probe}) = immune={} naive={}",
            signed_zero_preserving_sinh(probe),
            naive_sinh_loses_sign_at_zero(probe),
        );
    }
    println!();
    println!("The four-step substrate-witness workflow for this example:");
    println!("  1. scan (operator-facing prompt):");
    println!("     cargo run --bin cargo-antigen -- antigen scan --root antigen/examples");
    println!();
    println!("  2. scaffold a sidecar at the immune site:");
    println!("     cargo run --bin cargo-antigen -- antigen attest scaffold \\");
    println!("       --antigen SignedZeroDiscipline \\");
    println!("       --source-file antigen/examples/substrate_witness.rs \\");
    println!("       --item-path signed_zero_preserving_sinh \\");
    println!("       --fingerprint <use-cargo-antigen-scan-to-get-this>");
    println!();
    println!("  3. sign the sidecar (records WHO reviewed):");
    println!("     cargo run --bin cargo-antigen -- antigen attest sign \\");
    println!("       --sidecar antigen/examples/.attest/SignedZeroDiscipline.json \\");
    println!("       --item-path signed_zero_preserving_sinh \\");
    println!("       --signer alice --role math-researcher    # role matches roles = {{alice = \"math-researcher\"}} \\");
    println!("       --fingerprint <same-as-scaffold> \\");
    println!("       --reasoning \"reviewed sinh body: explicit x == 0.0 short-circuit \\");
    println!("                    preserves sign bit; bit-exact test locks contract\"");
    println!();
    println!("  4. audit reports Execution-tier evidence:");
    println!("     cargo run --bin cargo-antigen -- antigen audit --root antigen/examples");
}

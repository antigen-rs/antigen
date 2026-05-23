//! Example: `#[clonal]` — iterated witness evaluation with
//! non-deterministic seed enforcement.
//!
//! ADR-024 convergent-evidence family. `#[clonal]` captures the
//! pattern of running a witness across many independent iterations —
//! the B-cell clonal-expansion analog. Per adversarial C2, the macro
//! REJECTS `seed = SeedKind::Fixed(_)` at parse time: a fixed seed
//! makes "independent iterations" a contradiction.
//!
//! ## When to use `#[clonal]`
//!
//! - You have a property-test or fuzzer running many iterations
//! - You want the structural memory of "this witness runs N times"
//! - You want CI to fail if someone silently lowers the iteration
//!   count or pins the seed
//!
//! ## Biological cognate
//!
//! Immunology proper — B-cell clonal expansion. A single B-cell
//! recognizing an antigen divides into thousands of daughter cells,
//! each with slightly variable receptors (somatic hypermutation).
//! Independence across iterations is structural; collapsing to one
//! receptor template (fixed seed) erases the expansion.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example convergent_clonal --package antigen
//! ```

// Note: SeedKind is referenced inside the #[clonal] macro args as a
// path that the macro tokenizes; it doesn't need to be in scope as a
// real Rust import for the example to compile. The `use` line below
// keeps the visible cue for adopters reading the file.
#[allow(unused_imports)]
use antigen::SeedKind;
use antigen::{antigen, clonal};

#[antigen(
    name = "string-parse-roundtrip",
    fingerprint = r#"item = fn, name = matches("*_parse_*")"#,
    family = "trust-boundary-violation",
    summary = "parse() must roundtrip with to_string() for all valid inputs."
)]
pub struct StringParseRoundtrip;

/// Parse a `u32` from string with clonal-defended roundtrip property.
///
/// Defense: `10_000` iterations of `prop_parse_roundtrip` with non-
/// deterministic seed.
///
/// **The Fixed-seed rejection** (try uncommenting the second
/// `#[clonal]` below to see the compile error):
///
/// ```ignore
/// #[clonal(witness = prop_parse_roundtrip, iterations = 10_000, seed = SeedKind::Fixed(42))]
/// // ERROR: #[clonal] rejects `seed = SeedKind::Fixed(_)` — a fixed seed
/// // makes 'iterations' a misnomer (every iteration replays the same RNG state).
/// ```
#[clonal(
    witness = prop_parse_roundtrip,
    iterations = 10_000,
    seed = SeedKind::Random
)]
pub fn parse_u32(s: &str) -> Option<u32> {
    s.parse().ok()
}

/// A second clonal-attested function — uses CI-entropy seed so
/// run-to-run reproducibility is per-CI but cross-CI independent.
#[clonal(
    witness = prop_format_roundtrip,
    iterations = 5_000,
    seed = SeedKind::EntropyFromCi
)]
pub fn format_u32(n: u32) -> String {
    n.to_string()
}

fn main() {
    println!("=== antigen convergent-evidence: #[clonal] example ===");
    println!();
    println!("Two clonal declarations:");
    println!();
    println!("1. parse_u32");
    println!("   witness: prop_parse_roundtrip");
    println!("   iterations: 10_000");
    println!("   seed: SeedKind::Random (non-deterministic)");
    println!();
    println!("2. format_u32");
    println!("   witness: prop_format_roundtrip");
    println!("   iterations: 5_000");
    println!("   seed: SeedKind::EntropyFromCi (per-CI deterministic)");
    println!();
    println!("Per ADR-024 adversarial C2:");
    println!("  `seed = SeedKind::Fixed(_)` is REJECTED at parse time.");
    println!("  A fixed seed makes 'iterations' a misnomer — each");
    println!("  iteration replays the same RNG state.");
    println!();
    println!("  Try uncommenting a Fixed-seed clonal above to see the");
    println!("  compile error.");
    println!();
    println!("Sample evaluations:");
    println!("  parse_u32(\"42\") = {:?}", parse_u32("42"));
    println!("  parse_u32(\"abc\") = {:?}", parse_u32("abc"));
    println!("  format_u32(123) = {:?}", format_u32(123));
}

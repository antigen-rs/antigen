//! Example: `UnpinnedDependency` — exact-pin discipline.
//!
//! ADR-025 supply-chain defense family. `UnpinnedDependency` makes
//! the failure-class memory explicit: a `[dependencies]` entry
//! without an exact `=X.Y.Z` version specifier is the structural
//! analog of a loose-binding PRR receptor — it admits unbounded
//! version variation under the same trust boundary.
//!
//! ## Cargo specifier semantics
//!
//! - `"=1.0.197"` → exact pin (only this version)
//! - `"^1.0.197"` (or bare `"1.0.197"`) → caret (`>=1.0.197, <2.0.0`)
//! - `"~1.0.197"` → tilde (`>=1.0.197, <1.1.0`)
//! - `"*"` → wildcard (any version)
//! - `"?"` → undetermined (any version)
//!
//! Per ADR-025: exact-pin (`=`) is the high-specificity discipline.
//! Caret/tilde admit drift; wildcards admit unbounded chains.
//!
//! ## Narrow `UnpinnedTransitiveDependency` (per B9-R)
//!
//! The `UnpinnedTransitiveDependency` antigen has a NARROW definition:
//! a direct dep that uses `*`/`?` for ITS OWN deps. The broad form
//! ("any transitive dep with non-exact pins") was REJECTED because
//! ~100% of transitive deps are non-exact-pinned in practice and the
//! lockfile makes the resolved versions stable.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example supply_chain_unpinned --package antigen
//! ```

// Note: stdlib antigen paths in #[presents] / #[immune] are tokenized
// by the proc-macros; they don't need to resolve as Rust values.
#[allow(unused_imports)]
use antigen::stdlib::supply_chain::{UnpinnedDependency, UnpinnedTransitiveDependency};
use antigen::{immune, presents};

/// A function whose direct deps are exact-pinned via `dep_pinned()`.
/// The substrate-witness evaluator reads `Cargo.toml` and confirms
/// every entry uses `=X.Y.Z`.
#[presents(UnpinnedDependency)]
#[immune(
    UnpinnedDependency,
    requires = dep_pinned(),
)]
pub fn process_payload(data: &str) -> String {
    data.to_uppercase()
}

/// A narrow `UnpinnedTransitiveDependency` claim — only fires when a
/// direct dep uses `*`/`?` for its OWN deps. NOT the broad form.
#[presents(UnpinnedTransitiveDependency)]
#[immune(
    UnpinnedTransitiveDependency,
    // Same dep_pinned witness, scoped to a single named direct dep.
    requires = dep_pinned("serde"),
)]
pub fn delegate_to_serde(data: &str) -> String {
    data.to_lowercase()
}

fn main() {
    println!("=== antigen supply-chain: UnpinnedDependency example ===");
    println!();
    println!("Two unpinned-* declarations:");
    println!();
    println!("1. process_payload");
    println!("   antigen: UnpinnedDependency");
    println!("   witness: dep_pinned()");
    println!("   → evaluates: every Cargo.toml [dependencies] entry");
    println!();
    println!("2. delegate_to_serde");
    println!("   antigen: UnpinnedTransitiveDependency");
    println!("   witness: dep_pinned(\"serde\")");
    println!("   → evaluates: only the `serde` dep");
    println!();
    println!("Per ADR-025 B9-R: `UnpinnedTransitiveDependency` is NARROW.");
    println!("  CORRECT: direct dep with `*`/`?` for ITS OWN deps");
    println!("  REJECTED: 'any transitive dep with non-exact pins'");
    println!("    (~100% false-positive rate)");
    println!();
    println!("CLI defense:");
    println!("  cargo antigen verify deps");
    println!("    → flags every non-exact dep spec in Cargo.toml");
    println!("  cargo antigen verify dep-pin");
    println!("    → pins all unpinned deps in one sweep");
    println!();
    println!("Sample evaluations:");
    println!(
        "  process_payload(\"hello\") = {}",
        process_payload("hello")
    );
    println!(
        "  delegate_to_serde(\"WORLD\") = {}",
        delegate_to_serde("WORLD")
    );
}

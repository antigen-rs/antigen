// ADR-029 deprecation-window: uses the deprecated-but-functional #[immune] API;
// full migration to #[defended_by]/#[presents(requires=)] is a tracked follow-on.
#![allow(deprecated)]

//! Example: `UnsandboxedProcMacro` — the higher-risk supply-chain
//! attack surface (B3-R).
//!
//! ADR-025 supply-chain defense family. `UnsandboxedProcMacro`
//! captures the structural failure-class: external proc-macro
//! dependencies execute in-rustc at compile time with arbitrary
//! code-execution privilege. A compromised proc-macro is *strictly
//! more dangerous* than a compromised `build.rs` — it can rewrite
//! source token streams, embed payloads in compiler output, and
//! inspect every macro-expansion site in the workspace.
//!
//! ## Why proc-macro > build.rs (per B3-R)
//!
//! - `build.rs` runs in a separate process with the developer/CI
//!   user's privileges. Damage scope = file system + network.
//! - Proc-macros run *inside* `rustc`. Damage scope = file system +
//!   network + every byte of source the compiler sees + every byte
//!   of compiler output (binaries, dylibs, metadata).
//!
//! Per ADR-025: proc-macro sandboxing is HIGHER risk and gets its own
//! antigen. The biology cognate is the same — macrophage phagosome
//! containment — but the privileged compartment is the rustc process.
//!
//! ## v0.4+ scheduling
//!
//! Active sandbox execution is deferred to v0.4+ tooling-phase 3. The
//! v0.2 implementation emits the `unsandboxed-proc-macro` audit hint
//! as an awareness signal; adopters audit proc-macro deps manually
//! and pin tightly.
//!
//! ## Run this example
//!
//! ```sh
//! cargo run --example supply_chain_unsandboxed_proc_macro --package antigen
//! ```

// Note: stdlib antigen paths in #[presents] / #[immune] are tokenized
// by the proc-macros; they don't need to resolve as Rust values.
#[allow(unused_imports)]
use antigen::stdlib::supply_chain::UnsandboxedProcMacro;
use antigen::{immune, presents};

/// A function whose proc-macro dependency would be sandbox-attested.
///
/// `sandbox_clean("derive_more", sandbox_kind = "proc-macro")` is a
/// v0.4+ witness — in v0.2 it surfaces `unsandboxed-proc-macro` as
/// an awareness hint, NOT a passing evaluation. Per ADR-005 Amendment
/// 2 (honest-tier-naming): the audit names the tooling limitation
/// rather than silently passing.
#[presents(UnsandboxedProcMacro)]
#[immune(
    UnsandboxedProcMacro,
    requires = sandbox_clean("derive_more", sandbox_kind = "proc-macro"),
)]
pub const fn build_data_structure() -> u64 {
    // In real code this would use derive_more for trait derivations.
    42
}

fn main() {
    println!("=== antigen supply-chain: UnsandboxedProcMacro example ===");
    println!();
    println!("Higher-risk supply-chain attack surface (per ADR-025 B3-R).");
    println!();
    println!("Why proc-macro > build.rs:");
    println!("  build.rs runs in a separate process.");
    println!("  Proc-macros run INSIDE rustc — arbitrary code-execution");
    println!("  privilege inside the compiler process. They can rewrite");
    println!("  source tokens, embed payloads in compiler output, and");
    println!("  inspect every macro-expansion site in the workspace.");
    println!();
    println!("v0.2 state: tooling not yet available.");
    println!("  `cargo antigen verify proc-macro-sandbox` is a v0.4+ stub.");
    println!("  The audit emits `unsandboxed-proc-macro` as an AWARENESS");
    println!("  signal — adopters audit proc-macro deps manually.");
    println!();
    println!("Per ADR-005 Amendment 2 (honest-tier-naming):");
    println!("  The audit NAMES the limitation rather than silently passing.");
    println!("  Passing without a sandbox-clean witness would be a tier");
    println!("  inversion — a Reachability-tier claim presented as");
    println!("  Execution-tier.");
    println!();
    println!("Sample evaluation:");
    println!("  build_data_structure() = {}", build_data_structure());
}

//! Tolerance comparison — vibes-grade vs sidecar-attested.
//! (ADR-019 §tolerance tier + ADR-011 gap closure.)
//!
//! # The story
//!
//! Two engineers on the same codebase each declare tolerance for the same
//! failure-class. One writes a one-line rationale and moves on. The other
//! spends an hour reviewing the code with a math expert, writes a sidecar
//! capturing the review, and signs it.
//!
//! Both are "tolerance" — both opt out of immunity. But should the audit
//! treat them the same?
//!
//! ADR-019's answer is NO. The tolerance tier itself is bifurcated:
//!
//! - **Vibes-grade tolerance** (`#[antigen_tolerance(X, rationale = "...")]`):
//!   the rationale lives on the page. The audit reports the site as
//!   `tolerance-vibes-grade` at `WitnessTier::None` with `EvidenceKind::None`.
//!   No substrate was consulted; no signer reviewed; the discipline is
//!   "I considered this and accepted it." Honest declaration of intent,
//!   but the audit cannot distinguish thoughtful acceptance from
//!   rubber-stamp acceptance.
//!
//! - **Sidecar-attested tolerance** (`#[antigen_tolerance(X, requires = ...)]`):
//!   operator scaffolds a tolerance sidecar via
//!   `cargo antigen tolerate scaffold`; signs it with a math-expert
//!   reviewer; the audit reports `tolerance-predicate-passed-substrate-current`
//!   at `WitnessTier::Execution` with `EvidenceKind::SubstrateState` and
//!   the `Signer` list as evidence. Same opt-out, but now the audit can
//!   tell which sites had a math-expert review and which were thoughtful-
//!   but-unreviewed.
//!
//! This bifurcation closes the ADR-011 gap. v3 originally had a single
//! tolerance tier; ADR-019 ratified the two-tier shape so reviewed
//! tolerance is structurally distinguishable from documented-but-
//! unreviewed tolerance.
//!
//! # What this example demonstrates
//!
//! - A site with vibes-grade tolerance (one-line rationale, no sidecar)
//! - A site with sidecar-attested tolerance (predicate-bound, requires
//!   sidecar + math-expert signer)
//! - The operator workflow that distinguishes the two at scan + audit time
//!
//! Run:
//!
//! ```sh
//! cargo run --example tolerance_attested --package antigen
//!
//! # Scan reports both sites at the right tier:
//! cargo run --bin cargo-antigen -- antigen scan --root antigen/examples
//! ```

#![allow(dead_code, unused_imports)]

use antigen::{antigen, antigen_tolerance, presents};

/// Test scaffolding: marker macro the antigen fingerprint matches on.
/// Real code would have actual recursion shape; this is example fixture.
macro_rules! recurse_marker {
    () => {
        ()
    };
}

/// Recursive functions on user-supplied data without explicit depth
/// bounds can overflow the stack on adversarial input.
///
/// The discipline requires either (a) iterative reformulation, (b) explicit
/// depth check with `?` short-circuit, or (c) tolerance with documented
/// analysis of the input distribution showing depth is bounded in practice.
#[antigen(
    name = "unchecked-recursion",
    family = "boundary-violation",
    fingerprint = r#"all_of([item = fn, body_contains_macro("recurse_marker")])"#,
    summary = "Functions that recurse on user-supplied data without explicit \
               depth bounds can overflow the stack on adversarial input. \
               Discipline requires iterative reformulation, explicit depth \
               check with short-circuit, or sidecar-attested tolerance with \
               documented input-distribution analysis.",
    references = [
        "https://github.com/antigen-rs/antigen/blob/main/docs/decisions.md#adr-011",
    ],
)]
pub struct UncheckedRecursion;

/// Site A — vibes-grade tolerance.
///
/// The author considered the failure-class, accepts it, writes the
/// rationale on the page. No sidecar. No signer review. The audit
/// reports this site as `tolerance-vibes-grade` at `WitnessTier::None`
/// — operator-facing prompt that "you've declared tolerance but there's
/// no substrate to corroborate; promote to sidecar-attested if this
/// matters."
///
/// This is the right level for many sites. Not every tolerance needs an
/// expert review — sometimes "I considered it, here's why" is sufficient
/// and the friction of scaffolding + signing isn't warranted. Vibes-grade
/// is honest: it says "I documented this" without claiming "I reviewed
/// this with substrate evidence."
#[presents(UncheckedRecursion)]
#[antigen_tolerance(
    UncheckedRecursion,
    rationale = "Walks a config tree whose depth is bounded by schema-validation \
                 at parse time (max depth = 8 per Schema v1.2). Recursion is safe \
                 by construction of the input domain, not by structural enforcement \
                 here. If schema-validation is later relaxed, this needs revisit."
)]
pub fn walk_config_tree_vibes_grade(node: &ConfigNode) -> usize {
    recurse_marker!(); // body-contains-macro hook for the fingerprint
    1 + node
        .children
        .iter()
        .map(walk_config_tree_vibes_grade)
        .sum::<usize>()
}

/// Site B — sidecar-attested tolerance.
///
/// Same failure-class, same opt-out, but the operator chose to invest
/// the friction. The macro carries a `requires` predicate naming what
/// evidence must exist (presence of `requires` implies sidecar-attested
/// — vibes-grade is the macro's default when `requires` is absent).
/// After scaffolding +
/// signing `.attest/UncheckedRecursion.json` with a math-expert reviewer,
/// the audit reports this site at `WitnessTier::Execution` with
/// `EvidenceKind::SubstrateState` and the signer list as the substrate
/// trail.
///
/// The distinction the audit makes between Site A and Site B is
/// load-bearing: a CI gate that requires `>= tolerance-sidecar-attested`
/// for tolerance on hot-path code can enforce expert review structurally,
/// while still allowing vibes-grade on cold-path code where the friction
/// isn't worth it. ADR-019 §tolerance tier elevates the audit's
/// discriminative power from "tolerated or not" to "what KIND of
/// tolerance."
#[presents(UncheckedRecursion)]
#[antigen_tolerance(
    UncheckedRecursion,
    requires = all_of([
        signers(required = ["math-expert"]),
        fresh_within_days(days = 365),
    ]),
    rationale = "Newton-Raphson iteration on a domain provably bounded by \
                 the Lipschitz constant of the target function. A math-expert \
                 reviewer is required to attest that the bound calculation \
                 is correct for the specific function class this is applied to. \
                 Re-review annually as the function-class set evolves."
)]
pub fn newton_iterate_sidecar_attested(initial: f64, target: f64) -> f64 {
    recurse_marker!(); // body-contains-macro hook for the fingerprint
    if (initial - target).abs() < 1e-12 {
        return initial;
    }
    let next = f64::midpoint(initial, target);
    newton_iterate_sidecar_attested(next, target)
}

/// Test fixture for `walk_config_tree_vibes_grade`.
pub struct ConfigNode {
    /// Child nodes; recursion happens here.
    pub children: Vec<Self>,
}

fn main() {
    println!("antigen tolerance comparison — vibes-grade vs sidecar-attested.");
    println!();
    println!("Two sites in this file declare tolerance for UncheckedRecursion:");
    println!();
    println!("  Site A: walk_config_tree_vibes_grade");
    println!("    #[antigen_tolerance(UncheckedRecursion, rationale = \"...\")]");
    println!("    → audit reports: tolerance-vibes-grade at WitnessTier::None");
    println!("    → operator path: \"I considered this and accept it\"");
    println!();
    println!("  Site B: newton_iterate_sidecar_attested");
    println!("    #[antigen_tolerance(UncheckedRecursion, sidecar = true,");
    println!("                        requires = all_of([signers(required = [\"math-expert\"]),");
    println!("                                           fresh_within_days(days = 365)]))]");
    println!("    → audit reports: tolerance-predicate-passed-substrate-current");
    println!("                     at WitnessTier::Execution + EvidenceKind::SubstrateState");
    println!("    → operator path: \"a math-expert reviewed this; here's the sidecar trail\"");
    println!();
    println!("Both are tolerance; both opt out of immunity. The difference is");
    println!("whether the audit can corroborate the opt-out with substrate.");
    println!();
    println!("Operator workflow to lift Site B from vibes-grade to attested:");
    println!();
    println!("  1. cargo antigen tolerate scaffold \\");
    println!("       --antigen UncheckedRecursion \\");
    println!("       --source-file antigen/examples/tolerance_attested.rs \\");
    println!("       --item-path newton_iterate_sidecar_attested \\");
    println!("       --fingerprint <use-cargo-antigen-scan-to-get-this>");
    println!();
    println!("  2. cargo antigen tolerate sign \\");
    println!("       --sidecar antigen/examples/.attest/UncheckedRecursion.json \\");
    println!("       --item-path newton_iterate_sidecar_attested \\");
    println!("       --signer claire --role math-expert \\");
    println!("       --fingerprint <same-as-scaffold> \\");
    println!("       --reasoning \"verified Lipschitz bound for this function class\"");
    println!();
    println!("  3. cargo antigen audit --root antigen/examples");

    // Exercise both fixtures so the example links cleanly.
    let leaf = ConfigNode { children: vec![] };
    let _ = walk_config_tree_vibes_grade(&leaf);
    let _ = newton_iterate_sidecar_attested(1.0, 1.0);
}

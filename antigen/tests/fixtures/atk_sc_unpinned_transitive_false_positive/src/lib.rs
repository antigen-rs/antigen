// ATK-SC-4 FALSE POSITIVE GUARD
//
// This fixture models a codebase where:
// - Direct dependency (serde) is exactly pinned
// - The direct dep's OWN dependencies may be non-exact
//
// UnpinnedTransitiveDependency MUST NOT fire here.
// The NARROW definition is: "direct dep with `*`/`?` for ITS OWN deps."
// This codebase does NOT control serde's own Cargo.toml, so antigen
// has no basis to fire.
//
// If the audit produces `unpinned-transitive-dependency` here,
// it has implemented the WIDE definition (false positive storm).

use antigen::presents;
use antigen::immune;

/// ATK-SC-4: This struct presents UnpinnedTransitiveDependency only
/// to verify the audit logic is tested. The actual false-positive check
/// is: does the audit fire on a workspace with only exact-pinned direct
/// deps, even though serde's own (transitive) deps are not exact-pinned?
#[presents(UnpinnedTransitiveDependency)]
pub struct MyService;

/// A correct `#[immune]` claim for this site — direct deps ARE exactly pinned.
#[immune(UnpinnedTransitiveDependency, witness = verify_direct_deps_are_exact_pinned)]
pub struct MyServiceGuarded;

#[cfg(test)]
fn verify_direct_deps_are_exact_pinned() {
    // In a real codebase this would check Cargo.toml for `=` specifiers.
    // For the fixture, the presence of the test is the claim.
}

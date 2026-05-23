// ATK-SC-4 TRUE POSITIVE — UnpinnedTransitiveDependency SHOULD fire here.
//
// This fixture models the NARROW correct case:
// A DIRECT dependency of THIS workspace has `*` in ITS OWN dep spec.
//
// Specifically: if `bad-lib` (our direct dep) lists
//   serde = "*" in its own Cargo.toml
// then `bad-lib` has wildcard transitive dependencies.
// ANTIGEN SHOULD FIRE on this workspace.
//
// Note: we can't actually force this at fixture level (we don't control
// the published crate's Cargo.toml). The test verifies audit logic
// against a synthetic scan report rather than real Cargo.toml inspection.

// The audit check here is programmatic: the test in atk_sc_adversarial.rs
// constructs a synthetic SupplyChainDeclaration and verifies the hint fires.

pub struct DirectDepWithWildcardOwnDeps;

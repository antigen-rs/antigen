// Fixture: UnpinnedDependency presentation — no immunity.
//
// Purpose: verify that scan_workspace finds the UnpinnedDependency presentation
// and that the audit flags it as unaddressed.
//
// NOTE: `#[presents]` on `mod` blocks falls through to `ItemTarget::Unknown`
// because the scanner does not yet implement `visit_item_mod`. Supply-chain
// antigens often want module-level annotation (the whole module uses a dep).
// This is a known scanner gap to address in the supply-chain implementation.
// See: supply_chain_scan::scan_mod_target_falls_through_to_unknown.
//
// This fixture uses `fn` targets until `visit_item_mod` is added.
//
// Also demonstrates the NARROW UnpinnedTransitiveDependency distinction:
// UnpinnedDependency covers DIRECT deps without `=` pins.
// UnpinnedTransitiveDependency (NARROW) covers only direct deps that specify
// `*` or `?` for THEIR OWN transitive deps — NOT any transitive dep that
// happens to have non-exact pins (that would be 100% false-positive rate).
//
// ADR-025 §UnpinnedDependency + §UnpinnedTransitiveDependency (NARROW per B9-R).

#[antigen(
    name = "UnpinnedDependency",
    fingerprint = "item = fn"
)]
pub struct UnpinnedDependency;

/// This function uses `serde` which is specified as `serde = "^1.0"` in
/// Cargo.toml (range version, not exact-pinned). If serde releases a
/// 1.x version with a supply-chain compromise, cargo update would silently
/// pick it up on the next build.
#[presents(UnpinnedDependency)]
pub fn serialize_data() {}

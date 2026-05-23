// Fixture: All 11 supply-chain defense antigens presented on distinct fn items.
//
// Purpose:
// 1. Verify scan_workspace finds ALL 11 supply-chain antigen presentations
// 2. Verify antigen type names match the ADR-025 canonical names exactly
// 3. Baseline "coverage" test — if a new supply-chain antigen is added or
//    renamed, this fixture needs updating
//
// This is the failing-as-passing scaffold for supply-chain antigen coverage:
// a test using this fixture passes if-and-only-if all 11 antigens are found.
// If an antigen is renamed or dropped, the test fails — naming the regression.
//
// NOTE: All presentations use `fn` targets. The scanner does not yet support
// `mod` items (falls through to `ItemTarget::Unknown`). When `visit_item_mod`
// is added (supply-chain implementation), the fixture can be updated to use
// more natural annotation sites.
//
// ADR-025 §Decision — 11 stdlib antigens, all required (ADR-007 anti-YAGNI).

// ============================================================================
// Inline antigen declarations (stdlib-mirror for fixture testing)
// In production these would come from the antigen crate's stdlib.
// ============================================================================

#[antigen(name = "ContentHashMismatch", fingerprint = "item = fn")]
pub struct ContentHashMismatch;

#[antigen(name = "UnsandboxedProcMacro", fingerprint = "item = fn")]
pub struct UnsandboxedProcMacro;

#[antigen(name = "UnpinnedDependency", fingerprint = "item = fn")]
pub struct UnpinnedDependency;

#[antigen(name = "UnattestedDependencyInclusion", fingerprint = "item = fn")]
pub struct UnattestedDependencyInclusion;

#[antigen(name = "DependencyUpgradeWithoutDiffReview", fingerprint = "item = fn")]
pub struct DependencyUpgradeWithoutDiffReview;

#[antigen(name = "SuddenDependencyExpansion", fingerprint = "item = fn")]
pub struct SuddenDependencyExpansion;

#[antigen(name = "UnsandboxedBuildScript", fingerprint = "item = fn")]
pub struct UnsandboxedBuildScript;

#[antigen(name = "UnpinnedTransitiveDependency", fingerprint = "item = fn")]
pub struct UnpinnedTransitiveDependency;

#[antigen(name = "MaintainerChangeWithoutReattestation", fingerprint = "item = fn")]
pub struct MaintainerChangeWithoutReattestation;

#[antigen(name = "AutoDependencyChainWithoutPinning", fingerprint = "item = fn")]
pub struct AutoDependencyChainWithoutPinning;

#[antigen(name = "PostInstallScriptInDependency", fingerprint = "item = fn")]
pub struct PostInstallScriptInDependency;

// ============================================================================
// Presentations — one per antigen, distinct fn items
// ============================================================================

#[presents(ContentHashMismatch)]
pub fn uses_serde_dep_content_hash() {}

#[presents(UnsandboxedProcMacro)]
pub fn uses_proc_macro_dep() {}

#[presents(UnpinnedDependency)]
pub fn uses_range_pinned_dep() {}

#[presents(UnattestedDependencyInclusion)]
pub fn added_dep_without_attestation() {}

#[presents(DependencyUpgradeWithoutDiffReview)]
pub fn bumped_dep_without_diff_review() {}

#[presents(SuddenDependencyExpansion)]
pub fn dep_with_sudden_loc_delta() {}

#[presents(UnsandboxedBuildScript)]
pub fn uses_dep_with_build_script() {}

#[presents(UnpinnedTransitiveDependency)]
pub fn direct_dep_has_wildcard_for_own_deps() {}

#[presents(MaintainerChangeWithoutReattestation)]
pub fn dep_with_new_maintainer() {}

#[presents(AutoDependencyChainWithoutPinning)]
pub fn dep_chain_has_wildcard() {}

#[presents(PostInstallScriptInDependency)]
pub fn dep_runs_at_install_time() {}

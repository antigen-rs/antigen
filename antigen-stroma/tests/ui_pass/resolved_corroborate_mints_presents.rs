// ATK-FRAME-TIER-CAP · NEGATIVE CONTROL (must COMPILE) — the cap is tier-keyed, not a blanket ban.
//
// Proves a presents-grade verdict IS constructible via the single privileged door: `corroborate` over
// two fresh + independent resolved sources. If this failed to compile, the cap would forbid ALL
// presents verdicts (useless) rather than only the syntactic-sourced ones.
//
// SEAM NOTE: mirrors the §3.2 recommended door. Retarget alongside the ui/ fixture if the builder's
// realization differs. The invariant: there EXISTS a path (corroborate of fresh-independent resolved
// inputs) that mints presents — and it is the ONLY one.

use antigen_stroma::read::tier::{corroborate_presents, ResolutionTier};

fn main() {
    // Two independent resolved-tier sources corroborate → a presents verdict is minted. This is the
    // ONLY constructor. (Signature per the §3.2 recommendation; `Option` because two dread inputs, or
    // a stale input, yield None.)
    let minted = corroborate_presents(ResolutionTier::Resolved, ResolutionTier::Resolved);
    assert!(minted.is_some());
}

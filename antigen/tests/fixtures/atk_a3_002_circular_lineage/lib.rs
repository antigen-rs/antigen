// ATK-A3-002 fixture: a circular #[descended_from] chain.
//
// `Alpha` descends from `Beta`; `Beta` descends from `Alpha` — a 2-cycle.
// scan_workspace must detect the cycle (not loop indefinitely) and surface
// it as a structural ParseFailure with the chain in the error message.
//
// Both antigens are also declared via #[antigen] so the lineage edges are
// not orphans — the only failure should be the cycle itself.

#[antigen(name = "alpha", fingerprint = "item: struct")]
#[descended_from(Beta)]
pub struct Alpha;

#[antigen(name = "beta", fingerprint = "item: struct")]
#[descended_from(Alpha)]
pub struct Beta;

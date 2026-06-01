// ATK-3V-4 fixture: deferred supply-chain predicate must NOT produce SubstrateGap.
//
// This fixture declares an #[immune] site with a dep_pinned (supply-chain)
// requires= predicate. The sidecar exists and is valid (so we pass sidecar_missing),
// but the dep_pinned leaf is DEFERRED by the standard audit path. The correct
// verdict is not SubstrateGap — the predicate was not evaluated here, it needs
// a supply-chain audit pass. Before the three-valued-logic fix, immune_audit_is_substrate_gap()
// returned true for this state (conflating deferred with failed).

#[antigen(
    name = "deferred-predicate-class",
    fingerprint = "item: fn"
)]
pub struct DeferredPredicateClass;

#[presents(DeferredPredicateClass)]
#[immune(
    DeferredPredicateClass,
    requires = dep_pinned()
)]
pub fn guarded_fn() {
    // Body intentionally empty; we're testing the deferred-predicate verdict.
}

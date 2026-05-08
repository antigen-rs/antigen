// ATK-W5-007 fixture: free function with same name appears BEFORE the proptest! block.
//
// Walk order: visit_item_fn fires for `shadowed_by_free_fn` at line 12 first,
// inserting (name, Function) into the index. Then visit_macro fires for the
// proptest! block and tries or_insert_with — but the entry already exists, so
// the Proptest classification is silently dropped.
//
// Expected behavior (after fix): the proptest! function should win because it
// is the MORE SPECIFIC classification — a function inside proptest! is a proptest
// witness regardless of whether a same-named free function exists elsewhere.
//
// Current behavior (bug): the first-seen free function poisons the index entry;
// the proptest function is classified as WitnessKind::Function.

use antigen::immune;

fn shadowed_by_free_fn() {
    // Plain free function, no #[test], asserts nothing.
    // This function appears BEFORE the proptest! block in source order.
    // It poisons the index entry for `shadowed_by_free_fn`.
}

proptest::proptest! {
    #[test]
    fn shadowed_by_free_fn(x in 0u32..100) {
        // The real proptest witness — should be Proptest kind.
        assert!(x < 100);
    }
}

#[immune(PanickingInDrop, witness = shadowed_by_free_fn)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

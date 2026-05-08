// ATK-W3-005 fixture: two items of Unknown ItemTarget kind falsely match.
//
// If the visitor cannot classify an item (e.g., a type alias, a const, or
// a future Rust item kind), it falls back to ItemTarget::Unknown. The
// structural equality `Unknown == Unknown` means any two Unknown-kind items
// match each other in unaddressed_presentations — even if they are completely
// different items.
//
// Concrete scenario: two type aliases with #[presents] and one with #[immune].
// The immune should only address ONE of the presentations, but Unknown equality
// causes it to appear to address both (or neither, depending on iteration order).
//
// In practice, type aliases and consts are the most common Unknown-kind items.
// This fixture uses type aliases since they're valid Rust and the visitor
// doesn't have a visit_item_type_alias handler.

// Two type aliases both presenting the same antigen — they are different sites.
#[presents(PanickingInDrop)]
type VulnerableAlias1 = Vec<u8>;

#[presents(PanickingInDrop)]
type VulnerableAlias2 = Vec<String>;

// One immunity — it can only meaningfully cover ONE of the two aliases.
// If Unknown == Unknown, the single immune might "match" both presentations,
// producing zero unaddressed when there should be one.
#[immune(PanickingInDrop, witness = alias_witness)]
type ProtectedAlias = Vec<u8>;

#[test]
fn alias_witness() {
    assert!(true);
}

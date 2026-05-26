// Fixture: #[presents] on a trait associated type declaration.
// ScanVisitor has visit_trait_item_fn and visit_trait_item_const, but no
// visit_trait_item_type override. A #[presents(X)] on a `type Item;`
// inside a trait body silently compiles but should produce scan output.

use antigen::presents;

pub struct AssociatedTypeViolation;

pub trait Iterator {
    #[presents(AssociatedTypeViolation)]
    type Item;
}

// Fixture: #[presents] on a trait-body macro invocation (TraitItemMacro).
// ScanVisitor overrides visit_trait_item_fn, visit_trait_item_const, and
// visit_trait_item_type, but has no visit_trait_item_macro override. A macro
// invocation inside a trait body annotated with #[presents(X)] is silently
// dropped. Trait macro invocations are real patterns (blanket impl helpers,
// proc-macro trait-body generators) and may present failure classes.

use antigen::presents;

pub struct TraitContractViolation;

pub trait Expandable {
    #[presents(TraitContractViolation)]
    blanket_requirements!();
}

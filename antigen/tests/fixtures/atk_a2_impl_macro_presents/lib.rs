// Fixture: #[presents] on an impl-block macro invocation (ImplItemMacro).
// ScanVisitor overrides visit_impl_item_fn, visit_impl_item_const, and
// visit_impl_item_type, but has no visit_impl_item_macro override. A macro
// invocation inside an impl block annotated with #[presents(X)] is silently
// dropped. Macro invocations inside impl blocks are a real code pattern
// (e.g., delegate!, forward_to_inner!) and may present failure classes.

use antigen::presents;

pub struct MacroExpansionHazard;

pub struct Delegating;

impl Delegating {
    #[presents(MacroExpansionHazard)]
    forward_to_inner!();
}

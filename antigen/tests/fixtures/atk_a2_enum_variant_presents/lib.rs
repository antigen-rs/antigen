// Fixture: #[presents] on an enum VARIANT (not the enum itself).
// The scanner has visit_item_enum but no visit_variant override.
// syn::visit::visit_item_enum traverses variants but check_attrs is never
// called on variant.attrs — so this compiles silently and produces ZERO
// scan output. The presentation is invisible to the failure-class memory.

use antigen::presents;

pub struct BoundaryViolation;

pub enum RequestKind {
    /// This variant presents vulnerability to BoundaryViolation.
    #[presents(BoundaryViolation)]
    External { payload: Vec<u8> },
    Internal { payload: Vec<u8> },
}

// Fixture: digest contamination guard for visit_trait_item_type.
// Two trait associated types with different bounds are annotated with #[presents].
// Each must produce a DIFFERENT structural fingerprint. If visit_trait_item_type
// fails to assign self.current_item_digest before check_attrs, both types get
// the same (contaminated) digest — the same class as the impl_item_type fix.

use antigen::presents;

pub struct TypeAViolation;
pub struct TypeBViolation;

pub trait Contract {
    #[presents(TypeAViolation)]
    type Output: Clone;

    #[presents(TypeBViolation)]
    type Error: std::fmt::Display;
}

// Fixture: #[presents] on a top-level const.
// The scanner has visit_item_fn and visit_item_struct but no visit_item_const
// override. A #[presents(X)] on a const compiles cleanly but produces zero
// scan output.

use antigen::presents;

pub struct BoundaryViolation;

#[presents(BoundaryViolation)]
pub const MAX_REQUEST_SIZE: usize = 65536;

// Fixture: #[presents] on an impl-block const.
// The scanner has visit_impl_item_fn but no visit_impl_item_const override.
// A #[presents(X)] on an `impl` block's associated const silently compiles
// but produces zero scan output.

use antigen::presents;

pub struct BoundaryViolation;

pub struct Parser;

impl Parser {
    #[presents(BoundaryViolation)]
    pub const MAX_INPUT_BYTES: usize = 65536;
}

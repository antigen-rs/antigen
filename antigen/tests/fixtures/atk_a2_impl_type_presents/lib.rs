// Fixture: #[presents] on an impl-block associated type.
// ScanVisitor has visit_impl_item_fn and visit_impl_item_const, but no
// visit_impl_item_type override. A #[presents(X)] on a `type Foo = Bar;`
// inside an impl block silently compiles but should produce scan output.

use antigen::presents;

pub struct NullabilityViolation;

pub trait Wrapper {
    type Inner;
}

pub struct Box<T>(T);

impl Wrapper for Box<u32> {
    #[presents(NullabilityViolation)]
    type Inner = u32;
}

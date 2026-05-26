// Fixture: two associated types with #[presents] in an impl block.
// Used to detect digest contamination in visit_impl_item_type:
// if the visitor sets current_item_digest correctly, both types get their own
// unique fingerprints. If digest is omitted/contaminated, both fingerprints
// will be identical (contaminated by the preceding item's digest).

use antigen::presents;

pub struct NullabilityViolation;

pub trait Wrapper {
    type A;
    type B;
}

pub struct MyStruct(u32, u64);

impl Wrapper for MyStruct {
    #[presents(NullabilityViolation)]
    type A = u32;

    #[presents(NullabilityViolation)]
    type B = u64;
}

use antigen::presents;
pub struct BoundaryViolation;
pub trait Bounded {
    #[presents(BoundaryViolation)]
    const MAX_SIZE: usize;
}

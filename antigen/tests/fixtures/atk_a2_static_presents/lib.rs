use antigen::presents;
pub struct BoundaryViolation;
#[presents(BoundaryViolation)]
pub static GLOBAL_LIMIT: usize = 65536;

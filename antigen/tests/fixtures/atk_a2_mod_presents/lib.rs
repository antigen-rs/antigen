use antigen::presents;
pub struct BoundaryViolation;

#[presents(BoundaryViolation)]
pub mod dangerous_subsystem {
    pub fn entry_point() {}
}

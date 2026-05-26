use antigen::presents;
pub struct BoundaryViolation;

#[presents(BoundaryViolation)]
pub union RawUnion {
    pub integer: u64,
    pub bytes: [u8; 8],
}

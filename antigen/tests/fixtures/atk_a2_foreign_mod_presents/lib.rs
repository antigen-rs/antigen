use antigen::presents;
pub struct BoundaryViolation;

#[presents(BoundaryViolation)]
extern "C" {
    pub fn dangerous_ffi_call(ptr: *mut u8, len: usize) -> i32;
}

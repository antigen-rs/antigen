use antigen::presents;

#[presents(my_crate::PanickingInDrop)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

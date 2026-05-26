use antigen::presents;
pub struct SilentIntentNullification;

#[presents(SilentIntentNullification)]
macro_rules! discard_all_args {
    ($($arg:tt)*) => {};
}

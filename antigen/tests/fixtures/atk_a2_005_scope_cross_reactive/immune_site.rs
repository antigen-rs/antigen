// This file declares an immunity. The witness `verify_boundary` is intended
// to reference the #[test] function in tests.rs — but there is also a non-test
// function with the same name in utils.rs. The flat FunctionIndex cannot
// distinguish them; whichever file is walked last wins.

#[immune(PanickingInDrop, witness = verify_boundary)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

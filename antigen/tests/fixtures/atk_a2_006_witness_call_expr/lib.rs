// Fixture: immune with witness written as a call expression (with parens).
// This tests whether validate_witness correctly strips the call-expression
// suffix to resolve the function name.

#[immune(PanickingInDrop, witness = my_test_fn())]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

#[test]
fn my_test_fn() {
    assert!(true);
}

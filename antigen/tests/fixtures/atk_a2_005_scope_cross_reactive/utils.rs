// A non-test utility function with the same name as the test witness.
// The FunctionIndex cannot distinguish this from tests.rs::verify_boundary.
// If this file is indexed AFTER tests.rs, it overwrites the Test kind with
// Function kind. The audit then resolves the witness as WitnessKind::Function
// regardless of which function the developer intended to cite.
fn verify_boundary() {
    // This function asserts nothing. It is not a test.
    let _ = "boundary checking logic placeholder";
}

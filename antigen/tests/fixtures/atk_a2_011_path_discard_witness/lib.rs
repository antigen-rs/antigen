// ATK-A2-011 fixture: witness path with nonexistent prefix resolves cleanly.
//
// validate_witness does rsplit("::").next() — it discards everything before
// the last "::" and only looks up the last segment. So:
//
//   witness = nonexistent_crate::nonexistent_module::real_function_name
//
// resolves cleanly if `real_function_name` exists anywhere in the workspace,
// even though the path as written points nowhere coherent.
//
// The immune declaration below uses a fabricated path prefix. The function
// `real_function_name` exists in THIS file but has no connection to
// `nonexistent_crate::nonexistent_module`. The audit should surface this
// mismatch, not silently resolve.

#[immune(PanickingInDrop, witness = nonexistent_crate::nonexistent_module::real_function_name)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

// This function exists, but its module path is nothing like what the witness
// declaration claims. The witness says "nonexistent_crate::nonexistent_module"
// but this function is in the crate root.
#[test]
fn real_function_name() {
    assert!(true, "this test exists but is in the wrong module");
}

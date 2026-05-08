// ATK-PARSER-5: Keyword-adjacent identifiers in witness paths.
// Tests that various edge-case identifiers don't crash the scan parser.

// A witness using `gen` as part of a name (not a keyword in edition 2021).
#[immune(PanickingInDrop, witness = gen_boundary_check)]
impl Drop for TypeA {
    fn drop(&mut self) {}
}
struct TypeA;

// A witness using a path with `self` (lowercase — this is not the keyword
// in path position, but an identifier).
#[immune(PanickingInDrop, witness = check_self_type)]
impl Drop for TypeB {
    fn drop(&mut self) {}
}
struct TypeB;

fn gen_boundary_check() {}
fn check_self_type() {}

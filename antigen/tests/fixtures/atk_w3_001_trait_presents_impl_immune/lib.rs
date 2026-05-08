// ATK-W3-001 fixture: presents on a trait method, immune on the impl method.
//
// The structural question: when #[presents] lives on the trait definition and
// #[immune] lives on the impl, are they "on the same item" for matching purposes?
//
// Biology: the antigen is presented by the surface (trait signature); immunity
// is claimed at the expression site (impl body). These are structurally different
// items but logically the same vulnerability+proof pair.
//
// W3 must decide: does ImplFn("MyTrait", "dangerous_op") match
// TraitFn("MyTrait", "dangerous_op")? If not, every trait+impl pair produces
// a false unaddressed presentation.

trait SafeOps {
    #[presents(PanickingInDrop)]
    fn dangerous_op(&self);
}

struct MyType;

impl SafeOps for MyType {
    #[immune(PanickingInDrop, witness = dangerous_op_safety_test)]
    fn dangerous_op(&self) {
        // safe implementation
    }
}

#[test]
fn dangerous_op_safety_test() {
    assert!(true, "dangerous_op cannot panic");
}

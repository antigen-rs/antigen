// ATK-W3-004 fixture: generic impl with multiple instantiations.
//
// The structural question: if presents is on `impl<T> Container<T>` and immune
// is also on `impl<T> Container<T>`, does item-identity matching handle the
// generic parameter correctly? What if there are TWO separate impl blocks —
// one for `impl Container<i32>` (concrete, presents) and one for `impl<T>`
// (generic, immune)?
//
// The cross-crate concern (A3 territory): if Container<i32> in crate A presents
// and Container<String> in crate B also presents, are they the same "item" for
// inheritance purposes? W3 is workspace-local so this is deferred, but the
// item-identity representation must not preclude A3's answer.
//
// For W3: generic impls should match by impl-target type name ("Container"),
// not by full monomorphization path.

struct Container<T> {
    inner: T,
}

#[presents(PanickingInDrop)]
impl<T> Drop for Container<T> {
    fn drop(&mut self) {
        // might do something that could panic
    }
}

#[immune(PanickingInDrop, witness = container_drop_test)]
impl<T> Container<T> {
    fn safe_method(&self) {}
}

#[test]
fn container_drop_test() {
    let c: Container<i32> = Container { inner: 42 };
    drop(c);
    let c2: Container<String> = Container { inner: "hello".to_string() };
    drop(c2);
}

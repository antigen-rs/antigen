// ATK-W3-002 fixture: multiple impl blocks for the same type in one file.
//
// The structural question: if MyType has two impl blocks, and presents is in
// the first and immune is in the second, does W3's item-identity matching
// correctly associate them — or does it see them as different items because
// they're in different impl blocks?
//
// This is the proximity heuristic's most common failure: if the two impl blocks
// are more than 20 lines apart, the heuristic already fails. W3's fix must
// handle this correctly by matching on impl-target type, not line proximity.

struct MyType {
    value: i32,
}

#[presents(PanickingInDrop)]
impl MyType {
    fn new(value: i32) -> Self {
        Self { value }
    }
}

// Deliberately separated by many lines to break the 20-line heuristic.
// A real codebase might have dozens of methods between impl blocks.




#[immune(PanickingInDrop, witness = my_type_drop_safety_test)]
impl Drop for MyType {
    fn drop(&mut self) {
        // safe drop — no panics
    }
}

#[test]
fn my_type_drop_safety_test() {
    let t = MyType { value: 42 };
    drop(t);
}

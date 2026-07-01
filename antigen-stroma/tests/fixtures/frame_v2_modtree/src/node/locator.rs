// SPECIMEN (frame-v2#4 · module_chain regular-file branch): items in `src/node/locator.rs` live in
// module `node::locator` → chain = `["node", "locator"]` (full path-without-ext). Guards adapter.rs:132
// (the fall-through: not lib, not main, not mod → the whole chain).
pub struct LocatorItem {
    field: u8,
}

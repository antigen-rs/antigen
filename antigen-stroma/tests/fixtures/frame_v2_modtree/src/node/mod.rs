// SPECIMEN (frame-v2#4 · module_chain mod.rs branch): items in `src/node/mod.rs` live in module
// `node` → chain = `["node"]` (the `mod` leaf is stripped). Guards adapter.rs:127-128 (mod special-case
// + the strip-leaf arithmetic `segments.len() - 1`).
pub struct NodeModItem {
    field: u8,
}

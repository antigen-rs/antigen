// SPECIMEN (frame-v2#4 · module_chain lib.rs branch): items in `src/lib.rs` live in the CRATE ROOT
// → module chain is `[]` → fq_path = `crate::CrateRootItem`. Guards adapter.rs:119 (the lib special-case).
pub struct CrateRootItem {
    field: u8,
}

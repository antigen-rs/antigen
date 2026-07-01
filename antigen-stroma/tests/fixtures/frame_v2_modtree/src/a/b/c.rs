// SPECIMEN (frame-v2#4 · module_chain deep-nesting): items in `src/a/b/c.rs` live in module
// `a::b::c` → chain = `["a", "b", "c"]`. Guards the multi-segment join (adapter.rs:108-112, :132) and
// distinguishes the regular-file branch from the mod/lib/main special-cases at depth > 1.
pub struct DeepItem {
    field: u8,
}

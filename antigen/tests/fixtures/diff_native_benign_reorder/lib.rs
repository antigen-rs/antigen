// diff-native DETECT fixture — the DEGENERATE / benign-line-shift case
// (the un-run degenerate). Parsed-as-text.
//
// SAME items as diff_native_guard_before, SAME structure for each — but
// REORDERED in the file, with blank lines and comments inserted. Every item's
// structural_digest is UNCHANGED (the digest is structure-keyed, not
// line-position-keyed). So the (name, structural_digest) set-diff vs
// diff_native_guard_before must surface NOTHING — no phantom churn from a
// reorder/insert that changed no item's structure.
//
// This is the false-positive guard: a snapshot-blind tool that keyed on file+line
// would report every item as "changed" after a reorder. The diff-native modality
// must NOT.

// helper moved to the top, with a fresh comment above it.
pub fn helper(x: u32) -> u32 {
    x.wrapping_add(1)
}


// process moved up too.
fn process(_input: &[u8]) {}


// validate now last — but its body is byte-for-byte the BEFORE version (guard
// intact). Only its POSITION changed.
pub fn validate(input: &[u8], max: usize) -> Result<(), ()> {
    if input.len() > max {
        return Err(());
    }
    process(input);
    Ok(())
}

// E0 fixture — a CONSUMER crate with ZERO antigen declarations but REAL stdlib
// footguns. Parsed-as-text by the scanner (no compile, no macro crate). This
// stands in for "a Rust dev installs antigen and runs `cargo antigen scan` on
// their own crate" — the zero-hits-cliff case the bundled catalog must close.
//
// There is NO `#[antigen(...)]`, NO `#[presents(...)]`, NO `use antigen::...`
// anywhere in this file — that is the whole point. Without the bundled catalog
// the synthesis pass has an EMPTY fingerprint set and reports a FALSE all-clear
// (finalize.rs: `if !fingerprints.is_empty()` short-circuits). With the bundled
// catalog it must surface real flagship-family hits.
//
// The footguns below are chosen to match shipped Constructable-provenance
// flagship members so every E0 finding's class_provenance is in
// {Constructable, Encountered}:
//
//   - GetUncheckedWithoutProof  (panic-on-index family; Constructable;
//       fingerprint any_of([body_calls("get_unchecked"), body_calls("get_unchecked_mut")]))
//   - PanicInDrop               (drop-and-panic family; Constructable;
//       all_of([item=impl, impl_of_trait("Drop"), any_of([body_calls("unwrap"), ...])])

pub fn fast_index(buf: &[u8], i: usize) -> u8 {
    // get_unchecked: the unchecked-indexing escape hatch — OOB is UB.
    unsafe { *buf.get_unchecked(i) }
}

pub struct Resource {
    handle: u32,
}

impl Drop for Resource {
    fn drop(&mut self) {
        // A panic source inside a real Drop impl — double-panic-on-unwind aborts.
        let _ = flush(self.handle).unwrap();
    }
}

fn flush(_h: u32) -> Result<(), ()> {
    Ok(())
}

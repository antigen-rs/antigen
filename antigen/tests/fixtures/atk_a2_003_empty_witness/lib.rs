// Fixture: an immune declaration whose witness function has an empty body.
// The witness asserts nothing — it is reachability-only, not execution-tier.

#[immune(PanickingInDrop, witness = empty_witness)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

fn empty_witness() {}

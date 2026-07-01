// SPECIMEN (frame-v2#4 · mutation-kill for adapter.rs:119 `&&`→`||`): a MULTI-segment path whose
// FIRST segment is literally `lib` (`src/lib/sub.rs` → chain `["lib", "sub"]`). The lib special-case
// must fire ONLY when the WHOLE path is exactly `lib` (len==1 AND segments[0]=="lib"), not merely
// when the first segment is `lib`. A `&&`→`||` mutant would collapse this to the crate root (`[]`) —
// this specimen is the come-apart that catches it.
pub struct LibSubItem {
    field: u8,
}

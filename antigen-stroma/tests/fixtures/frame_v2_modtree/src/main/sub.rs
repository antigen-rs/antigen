// SPECIMEN (frame-v2#4 · mutation-kill for adapter.rs:123 `&&`→`||`): a MULTI-segment path whose
// FIRST segment is literally `main` (`src/main/sub.rs` → chain `["main", "sub"]`). The main
// special-case must fire ONLY when the WHOLE path is exactly `main` (len==1 AND segments[0]=="main"),
// not merely when the first segment is `main`. A `&&`→`||` mutant would collapse this to the crate
// root (`[]`) — this specimen is the come-apart that catches it.
pub struct MainSubItem {
    field: u8,
}

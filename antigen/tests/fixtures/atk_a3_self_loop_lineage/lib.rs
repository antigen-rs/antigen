// A3 hardening fixture: a #[descended_from] self-loop.
//
// Solo declares descent from itself (Solo -> Solo). The most degenerate
// cycle shape — single edge, single node. Scan must detect this without
// looping indefinitely.

#[antigen(name = "solo", fingerprint = "item: struct")]
#[descended_from(Solo)]
pub struct Solo;

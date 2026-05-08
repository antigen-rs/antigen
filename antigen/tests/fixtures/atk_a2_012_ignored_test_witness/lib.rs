// ATK-A2-012 fixture: #[test] #[ignore] witness passes audit as Test-kind.
//
// A witness function with #[test] AND #[ignore] attributes never runs by
// default (cargo test skips #[ignore] tests unless --include-ignored is passed).
// Audit should distinguish #[test] from #[test] #[ignore] — the latter is
// strictly weaker evidence of immunity.
//
// The concrete pathological scenario: a developer marks a witness as #[ignore]
// because it's "not ready yet" or "requires external setup." The audit reports
// it as WitnessKind::Test (well-formed) even though cargo test won't execute it
// by default.

#[immune(PanickingInDrop, witness = not_yet_ready_witness)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

#[test]
#[ignore = "witness not implemented yet — audit should flag this"]
fn not_yet_ready_witness() {
    // This witness is marked ignore. It never runs in standard CI.
    // Audit sees #[test] and classifies as WitnessKind::Test.
    // But the immunity claim is backed by a test that doesn't run.
    panic!("not implemented");
}

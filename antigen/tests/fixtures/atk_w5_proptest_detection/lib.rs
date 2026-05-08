// ATK-W5 fixture: various proptest! detection edge cases.
//
// This file contains functions that should and should NOT be classified as
// WitnessKind::Proptest by W5's structural detection.
//
// Expected classifications (W5 target):
//   real_proptest_fn       → WitnessKind::Proptest  (inside proptest! block)
//   second_proptest_fn     → WitnessKind::Proptest  (second fn in same block)
//   not_proptest_despite_comment → WitnessKind::Test (only in doc comment)
//   plain_test             → WitnessKind::Test       (plain #[test], no proptest)

use antigen::immune;

/// This function is NOT a proptest witness, despite this doc comment
/// mentioning the `proptest!` macro for documentation purposes.
#[test]
fn not_proptest_despite_comment() {
    assert!(true, "this is a plain #[test], not inside proptest!");
}

/// A genuinely plain test function with no proptest involvement.
#[test]
fn plain_test() {
    assert!(true);
}

// The actual proptest! invocations — these inner functions should be
// detected as Proptest kind by W5's structural detection.
//
// NOTE: This fixture uses a simplified syntax. Real proptest! requires
// the proptest crate. Since we're testing the DETECTION logic (not running
// the tests), we simulate the macro invocation shape.
//
// W5's structural detection must:
// 1. Walk macro invocations in the AST
// 2. Identify invocations where the macro path is `proptest`
// 3. Scan the token stream body for `fn NAME` patterns
// 4. Register those function names as WitnessKind::Proptest in the index
//
// The simulation below represents the source shape W5 must handle.

proptest::proptest! {
    #[test]
    fn real_proptest_fn(x in 0u32..100) {
        assert!(x < 100);
    }

    #[test]
    fn second_proptest_fn(x in 0u32..100, y in 0u32..100) {
        assert!(x + y < 200);
    }
}

#[immune(PanickingInDrop, witness = real_proptest_fn)]
impl Drop for SomeType {
    fn drop(&mut self) {}
}

struct SomeType;

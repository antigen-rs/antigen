// ATK-A3 substrate-witness-pipeline fixture.
//
// rc.1 bug regression: `#[immune(X, requires = ...)]` declarations were
// silently dropped at scan time because the substrate-witness predicate
// only survived as a `#[doc = " antigen:requires:v1:..."]` marker emitted
// AFTER macro expansion — but `syn::parse_file` reads the WRITTEN SOURCE,
// not post-expansion. So scan never saw the marker, the audit always
// fell through to the code-witness branch, and every substrate-witness
// claim reported `tier=None, hint=NoneApplicable / "missing witness"`.
//
// rc.2 fix: scan parses `requires = <predicate>` directly from the source
// attribute via the shared `antigen_attestation::parser`. This fixture
// declares a substrate-witness immunity with no `.attest/` sidecar; the
// audit MUST route via the substrate-witness path and report a
// substrate-specific hint (`discipline-sidecar-missing`), not a
// code-witness fallthrough.

#[antigen(
    name = "signed-zero-discipline",
    fingerprint = "item: fn"
)]
pub struct SignedZeroDiscipline;

#[presents(SignedZeroDiscipline)]
#[immune(
    SignedZeroDiscipline,
    requires = all_of([
        signers(required = ["reviewer"]),
        fresh_within_days(180),
    ])
)]
pub fn signed_zero_preserving_sinh(x: f64) -> f64 {
    if x == 0.0 {
        return x;
    }
    x.sinh()
}

// ATK-SCIP-RECON-001 · FIXTURE 2 (the negative control — must be Resolved).
//
// A plain (non-macro) function call at a well-defined call-site. The reference occurrence nests under
// exactly ONE caller definition — clean lexical enclosure.
//
// REQUIRED VERDICT: EdgeReconstruction is `Resolved(edge)` at tier=resolved.
//
// WHY LOAD-BEARING: without this NC, a trivially-failing impl (always return `Unreconstructible`)
// would pass Fixture 1 without implementing reconstruction at all. The NC proves the test
// distinguishes the degenerate case from the happy path.

fn callee_target() {}

fn caller() {
    // A plain call — the occurrence of `callee_target` is lexically enclosed by `caller`, exactly one
    // candidate. Enclosure reconstructs cleanly => Resolved.
    callee_target();
}

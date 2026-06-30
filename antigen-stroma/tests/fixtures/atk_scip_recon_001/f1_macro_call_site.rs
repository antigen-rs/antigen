// ATK-SCIP-RECON-001 · FIXTURE 1 (the degenerate case — the BORN-RED ATK).
//
// A macro that expands to a function call. The expanded call-site's SCIP occurrence does NOT nest
// cleanly under the textual caller definition (macro expansion breaks the lexical-enclosure
// assumption the enclosure-reconstruction relies on).
//
// REQUIRED VERDICT: EdgeReconstruction for the macro-expanded call MUST be `Ambiguous(..)` or
// `Unreconstructible` — NEVER `Resolved`. Picking one candidate and stamping `resolved` is the
// cardinal sin (a confident-wrong edge = observational-autoimmunity). This fixture FAILS if any
// macro-call-site edge carries presents-grade without verified enclosure.

fn callee_target() {}

macro_rules! expands_to_a_call {
    () => {
        // The call originates inside the macro expansion, not at a textual call-site under `driver`.
        callee_target()
    };
}

fn driver() {
    // After expansion this is a call to `callee_target`, but the occurrence's enclosing-definition
    // reconstruction cannot cleanly attribute it (the lexical span is the macro, not `driver`).
    expands_to_a_call!();
}

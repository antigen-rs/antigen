// ATK-FRAME-TIER-CAP (ADR-070 §3.2) — a source=syntactic read MUST be UNABLE to construct a
// presents-grade verdict. The cap is a COMPILE-TIME impossibility (type-state), not a runtime check.
//
// THE RECOMMENDED IDIOM (ADR-070 §3.2): a `PresentsVerdict` has NO public constructor — it is mintable
// ONLY by `corroborate(a, b)` (the single privileged upgrade door, which checks fresh + independent).
// A syntactic read returns an answer at its source ceiling (dread) and has NO path to mint a
// PresentsVerdict. Because the constructor is private, a syntactic read literally cannot type-check a
// presents answer.
//
// THIS FIXTURE: attempt to construct a `PresentsVerdict` directly (bypassing `corroborate`) from a
// syntactic context. It MUST fail to compile (no public constructor / private field).
//
// SEAM NOTE (surfaced): the skeleton currently realizes tier-honesty as a RUNTIME refusal
// (`TieredAnswer::read_at_least`), NOT the §3.2 type-state. This fixture asserts the type-state the
// §3.2 PROCESS recommends. If the builder lands the cap as the recommended `PresentsVerdict`-no-ctor
// door, this fixture's `.stderr` is the privacy/constructor error and it goes green. If the builder
// picks a DIFFERENT compile-time realization that still makes a syntactic presents UNCONSTRUCTIBLE,
// retarget this fixture onto that surface — the INVARIANT binds (compile-time cap), the type is open.
// Until a compile-time cap exists, this fixture COMPILES → born-red (RED harness).

use antigen_stroma::read::tier::PresentsVerdict;

fn main() {
    // A syntactic read has only a dread-grade answer in hand. Forging a presents verdict directly
    // must be impossible: `PresentsVerdict` has no public constructor (mintable only via corroborate).
    let _forged = PresentsVerdict::new(); // MUST NOT COMPILE — no public constructor.
}

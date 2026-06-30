//! ATK-SCIP-RECON-001 — SCIP reconstruction is TIER-HONEST (the three-way distinction binds).
//!
//! ## The claim this defends (ADR-070 §5.2, attack A5; island `frame-impl-scip-ingestion`)
//! SCIP gives OCCURRENCES; a call EDGE requires reconstructing "this reference-occurrence is
//! LEXICALLY ENCLOSED BY that definition-occurrence." That enclosure step is the UNVERIFIED step. The
//! reconstruction surface MUST encode a THREE-WAY tier-honest distinction:
//!
//! - cleanly-reconstructed → `Resolved` (presents-grade eligible)
//! - nests under >1 candidate → `Ambiguous` (DEMOTE, do NOT pick)
//! - enclosure failed (macro/generated) → `Unreconstructible` (drop to syntactic / mark unknown)
//!
//! The cardinal sin: PICKING one candidate and stamping it `Resolved` — a confident-wrong edge.
//!
//! ## The 3-fixture spec (all in ONE module — attack A5)
//!   F1 (the ATK): a macro-call-site MUST be `Ambiguous`/`Unreconstructible`, NEVER `Resolved`.
//!   F2 (NC): a plain call-site MUST reconstruct cleanly to `Resolved`. (Without F2, a trivial
//!            always-`Unreconstructible` impl would pass F1 vacuously.)
//!   F3 (NC): a malformed SCIP symbol MUST fall through to syntactic, never construct a node with the
//!            malformed symbol as `fq_path`.
//! Fixtures live as readable specimens under `tests/fixtures/atk_scip_recon_001/`.
//!
//! ## Born-red status
//! `ingest_scip` is ENGINE-epoch `todo!()` (the scip-run wiring). The FRAME-epoch claim is the TYPE:
//! the contract cannot even REPRESENT a silently-resolved macro edge (`EdgeReconstruction` has no
//! "guessed-resolved" variant). These tests are `#[ignore]` until the resolved feeder is wired; the
//! frame-epoch type-honesty is asserted by the compile-level test below (which is NOT ignored once
//! the crate compiles — it proves the three-way distinction EXISTS in the contract).

use antigen_stroma::scip::EdgeReconstruction;

const FIXTURES: &str = "tests/fixtures/atk_scip_recon_001";

// FRAME-EPOCH (not ignored once the crate compiles): the three-way distinction EXISTS in the type.
// This is the tier-honesty the frame ships regardless of the engine wiring. A builder who collapsed
// `EdgeReconstruction` to a 2-way (resolved/failed) or who added a `GuessedResolved` variant would
// break this. The match is exhaustive: if a variant is added/removed, this fails to compile — a
// guard that the three-way distinction stays exactly three-way.
#[test]
fn atk_scip_recon_001_three_way_distinction_exists_in_contract() {
    fn classify(r: &EdgeReconstruction) -> &'static str {
        match r {
            EdgeReconstruction::Resolved(_) => "resolved",
            EdgeReconstruction::Ambiguous(_) => "ambiguous-demote",
            EdgeReconstruction::Unreconstructible => "unreconstructible",
        }
    }
    // Build the one variant constructible without engine population (the others carry ResolvedEdge,
    // an engine-fill type). Unreconstructible is the floor and must always be representable.
    let floor = EdgeReconstruction::Unreconstructible;
    assert_eq!(classify(&floor), "unreconstructible");
}

// F1 — ATK (born-red until ingest_scip is wired): the macro-call-site MUST NOT be Resolved.
#[test]
#[ignore = "born-red until ingest_scip resolved-feeder is wired (engine epoch); de-ignore on fill"]
fn atk_scip_recon_001_f1_macro_call_site_is_never_resolved() {
    let index = scip_index_for(&format!("{FIXTURES}/f1_macro_call_site.rs"));
    let edges = antigen_stroma::scip::ingest_scip(&index);

    // EVERY edge attributable to the macro-expanded call must be Ambiguous or Unreconstructible.
    // (There is exactly one call in the fixture: callee_target via the macro.)
    let any_resolved_macro_edge = edges
        .iter()
        .any(|e| matches!(e, EdgeReconstruction::Resolved(_)));
    assert!(
        !any_resolved_macro_edge,
        "ATK-SCIP-RECON-001 F1: a macro-call-site edge was stamped `Resolved` on a guessed enclosure \
         — the cardinal sin (confident-wrong edge / observational-autoimmunity). It MUST be \
         `Ambiguous` or `Unreconstructible`."
    );
}

// F2 — NC (proves F1 is not vacuous): the plain call-site MUST be Resolved.
#[test]
#[ignore = "born-red until ingest_scip resolved-feeder is wired (engine epoch); de-ignore on fill"]
fn atk_scip_recon_001_f2_plain_call_site_resolves_cleanly() {
    let index = scip_index_for(&format!("{FIXTURES}/f2_plain_call_site.rs"));
    let edges = antigen_stroma::scip::ingest_scip(&index);

    let has_clean_resolved = edges
        .iter()
        .any(|e| matches!(e, EdgeReconstruction::Resolved(_)));
    assert!(
        has_clean_resolved,
        "NC F2: a plain (non-macro) call-site did NOT reconstruct to `Resolved`. The reconstruction \
         is over-conservative (a trivial always-Unreconstructible impl) — F1 would then pass \
         vacuously. The test must distinguish the degenerate case from the happy path."
    );
}

// F3 — NC (malformed-symbol fall-through): a malformed SCIP symbol must NOT become a node fq_path.
#[test]
#[ignore = "born-red until ingest_scip resolved-feeder + symbol validation are wired; de-ignore on fill"]
fn atk_scip_recon_001_f3_malformed_symbol_falls_through_to_syntactic() {
    let index = scip_index_for(&format!("{FIXTURES}/f3_malformed_symbol.scip.txt"));
    let edges = antigen_stroma::scip::ingest_scip(&index);

    // No edge may carry a Resolved verdict built from the malformed symbol — it falls through to the
    // syntactic tier (dread) or is Unreconstructible. (When a `StromaNodeId` accessor lands on
    // ResolvedEdge, strengthen this to assert no node's fq_path equals the malformed raw string.)
    let any_resolved = edges
        .iter()
        .any(|e| matches!(e, EdgeReconstruction::Resolved(_)));
    assert!(
        !any_resolved,
        "NC F3: a malformed SCIP symbol produced a `Resolved` edge — the ingestion constructed a \
         node from an unparseable symbol instead of falling through to the syntactic-tier locator. \
         This is an observational-autoimmunity seed (garbage matches in the Locator intern table)."
    );
}

/// Build/locate the SCIP index for a fixture. **ENGINE-epoch helper stub** — when the resolved feeder
/// is wired, this runs `rust-analyzer scip` over the fixture (or loads a checked-in `.scip`). The
/// `#[ignore]` on the callers keeps this from running until that wiring lands.
fn scip_index_for(_fixture_source: &str) -> std::path::PathBuf {
    todo!(
        "engine epoch: produce a SCIP index for the fixture (run r-a scip OR load a checked-in .scip)"
    )
}

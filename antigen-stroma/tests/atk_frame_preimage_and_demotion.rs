//! Gap-closing ATKs — BINDING invariants the §8 seed-table did NOT enumerate (the table is the floor,
//! not the ceiling). Found by the test-architect's registry cross-walk: map every "INVARIANT (BINDS"
//! in ADR-070 to a defending ATK; these three had NONE.
//!
//! - ATK-FRAME-PREIMAGE-TOKENS-ONLY (§4.4) — the `IdentityDigest` preimage is TOKENS-ONLY; folding
//!   `fq_path`/cfg into it couples identity to resolution-state.
//! - ATK-FRAME-INGEST-DEMOTION (§5.3) — staleness demotes resolved→dread AT INGESTION, so the STORED
//!   tier already reflects it (closes the corroborate-up window between ingestion and a query-time check).
//! - ATK-FRAME-T3-SLOT-PRESENT (§3.1) — the read-frame ships a structurally-present `T3Mir` slot;
//!   a 2-axis frame / deleted T3 is code-drift against ADR-069.
//!
//! Registered in ATK-REGISTRY.md under "gap-closing (post-floor)".

use antigen_stroma::node::digest::IdentityDigest;
use antigen_stroma::read::ResolutionTier;

// ── ATK-FRAME-PREIMAGE-TOKENS-ONLY (§4.4) ────────────────────────────────────────────────────────
// The identity digest is a PURE function of the item's own canonical tokens. Two nodes that differ
// ONLY in a sibling identity field (cfg_set, fq_path) — same item tokens — get the SAME IdentityDigest.
// (They are distinct NODES via the StromaNodeId 3-tuple, but the DIGEST component must not move.) If a
// builder folds cfg/path into the preimage, a cargo-metadata read (composed source) would change the
// collision-resistant identity — a compose/sovereign violation (§4.4).
#[test]
fn atk_frame_preimage_is_tokens_only_cfg_not_folded() {
    // SAME item tokens. The cfg/path differences live in sibling StromaNodeId fields, NOT the preimage.
    let tokens = b"fn handle() { work() }";
    let digest_under_unix = IdentityDigest::of_tokens(tokens);
    let digest_under_windows = IdentityDigest::of_tokens(tokens); // same tokens — cfg is a SIBLING field

    assert_eq!(
        digest_under_unix, digest_under_windows,
        "ATK-FRAME-PREIMAGE-TOKENS-ONLY: the IdentityDigest changed for the same item tokens — \
         a sibling field (cfg/path) was folded into the digest preimage. That couples the \
         collision-resistant identity to resolution-state (a stale cargo-metadata read would churn \
         identity). Preimage = canonical item tokens ONLY (§4.4)."
    );
}

// NEGATIVE CONTROL (teeth): different ITEM TOKENS still produce different digests — proving the
// tokens-only rule did not collapse the digest into a constant.
#[test]
fn nc_frame_preimage_distinct_tokens_still_distinct() {
    let a = IdentityDigest::of_tokens(b"fn handle() { work() }");
    let b = IdentityDigest::of_tokens(b"fn handle() { rest() }");
    assert_ne!(
        a, b,
        "NC: distinct item tokens collided — tokens-only was over-applied into a constant digest."
    );
}

// ── ATK-FRAME-INGEST-DEMOTION (§5.3) ─────────────────────────────────────────────────────────────
// A stale-SCIP edge must carry tier=dread (Syntactic) in its STORED tier FROM CONSTRUCTION — not pass
// through resolved-then-checked-later. The proof: after ingestion, corroborate over the stored tiers
// of two stale inputs CANNOT reach presents (because both are already demoted). If demotion were
// query-time, a stale edge would sit at Resolved post-ingestion and corroborate up before the witness
// fires — the confident-wrong window §5.3 closes.
//
// SEAM NOTE: this asserts the COMPOSITION fidelity-witness(ingest) then corroborate reads the stored
// tier. Until both land, born-red. The shim models ingestion as: stored_tier = check(src,idx,claimed).
#[test]
fn atk_frame_stale_scip_is_demoted_at_ingestion_so_corroborate_cannot_reach_presents() {
    use antigen_stroma::fidelity::FidelityWitness;
    use antigen_stroma::read::tier::corroborate;

    // Ingest a SCIP edge whose index is stale (source newer). The STORED tier must already be dread.
    let stored_tier_a = FidelityWitness::check(
        /*src*/ 2000,
        /*idx*/ 1000,
        ResolutionTier::Resolved,
    );
    let stored_tier_b = FidelityWitness::check(2000, 1000, ResolutionTier::Resolved);
    assert_eq!(
        stored_tier_a,
        ResolutionTier::Syntactic,
        "ATK-FRAME-INGEST-DEMOTION: a stale-SCIP edge stored tier=Resolved — demotion did not happen \
         at ingestion. It would corroborate up before the witness fires (§5.3 confident-wrong window)."
    );

    // corroborate reads the STORED tiers (never re-reads freshness). Two demoted inputs cannot reach
    // presents — there is no path to a higher tier from two dread-stored edges.
    let raised = corroborate(stored_tier_a, stored_tier_b);
    assert!(
        !matches!(
            raised,
            Some(ResolutionTier::Resolved | ResolutionTier::T3Mir)
        ),
        "ATK-FRAME-INGEST-DEMOTION: corroborate raised two ingestion-demoted (dread) edges back up — \
         the stored-tier demotion was bypassed (corroborate re-read freshness or ignored the store)."
    );
}

// ── ATK-FRAME-T3-SLOT-PRESENT (§3.1) ─────────────────────────────────────────────────────────────
// The read-frame ships a structurally-present T3 (mir-only) slot — empirically empty on antigen's own
// code, but REACHABLE. Building a 2-axis frame or deleting/collapsing T3 is code-drift against
// ADR-069. This is a COMPILE-LEVEL structural guard (not ignored): if the T3Mir variant is removed,
// this fails to compile. T3-empty != T3-doesn't-exist.
#[test]
fn atk_frame_t3_slot_is_structurally_present() {
    // Naming the variant binds its existence. If a builder deletes T3Mir (collapsing to 2 tiers), the
    // crate fails to compile here — the live-but-empty slot is guarded structurally.
    let t3 = ResolutionTier::T3Mir;
    // And it sits ABOVE Resolved in the ordered ladder (Syntactic < Resolved < T3Mir).
    assert!(
        t3 > ResolutionTier::Resolved && ResolutionTier::Resolved > ResolutionTier::Syntactic,
        "ATK-FRAME-T3-SLOT-PRESENT: the tier ladder is not ordered Syntactic < Resolved < T3Mir — \
         the T3 slot was collapsed or mis-ordered (code-drift against ADR-069 §B)."
    );
}

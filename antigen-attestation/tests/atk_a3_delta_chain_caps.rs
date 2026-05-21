//! Integration test for the three-layer anti-laundering safeguards on
//! `SignerBasis::DeltaFrom` per ADR-019 §M3 + adversarial T2R-A/B/C.
//!
//! ## What this test guards
//!
//! ADR-019 commits to three layered safeguards on delta-attestation entries:
//!
//! - **T2R-A (chain-depth cap)**: `chain_depth <= cap`; workspace can
//!   tighten the cap below `HARD_DELTA_CHAIN_CAP_MAX = 10` but cannot
//!   loosen it above. Workspace cannot disable enforcement by setting
//!   cap below `HARD_DELTA_CHAIN_CAP_MIN = 1`.
//! - **T2R-B (rationale minimum length)**: `rationale.len() >=
//!   delta_rationale_min_chars` (default 20). Workspace can tighten
//!   above the floor `HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR = 10` but
//!   cannot loosen below.
//! - **T2R-C (non-empty rationale)**: even before the length check,
//!   trimmed-empty rationales are rejected.
//!
//! Removing ANY ONE of these safeguards re-opens the laundering surface;
//! the three together close it. This test asserts the boundary behavior
//! at the exact thresholds so a regression that subtly relaxes one
//! safeguard (e.g., off-by-one on `chain_depth <= cap` vs `< cap`) is
//! caught immediately.
//!
//! ## Why this test ships in v0.1
//!
//! Per ADR-019 §Decision: "Removing any one re-opens the laundering
//! surface; the three together close it." The 3-safeguard discipline is
//! load-bearing for tier-honesty — if delta entries can be rubber-stamped
//! or chained indefinitely, the `Execution`-tier reports on delta-chain
//! audits are silently overclaiming. This test is the runtime invariant
//! lock for the schema commitment.

use antigen_attestation::{
    schema::{
        validate_chain_cap, validate_rationale_min_chars, AntigenIdentifier, ItemRatification,
        ValidationError, DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS,
        HARD_DELTA_CHAIN_CAP_MAX, HARD_DELTA_CHAIN_CAP_MIN, HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR,
    },
    Ratification, RatificationKind, SchemaVersion, SignatureStrength, Signer, SignerBasis,
};
use chrono::NaiveDate;
use std::collections::BTreeMap;
use std::path::PathBuf;

const fn sample_date() -> NaiveDate {
    // chrono ≥ 0.4.40 makes from_ymd_opt const; MSRV is 1.85 so we lean on it.
    NaiveDate::from_ymd_opt(2026, 5, 19).expect("hard-coded valid date")
}

/// 20-char rationale — meets `DEFAULT_DELTA_RATIONALE_MIN_CHARS` exactly.
const MIN_VALID_RATIONALE: &str = "reviewed; consistent"; // 20 chars trimmed

/// Comfortable rationale used for tests not specifically exercising the
/// length boundary.
const VALID_RATIONALE: &str = "reviewed diff against prior; invariant-preserving change";

fn fresh_signer(name: &str, date: NaiveDate, fp: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date,
        signed_against_fingerprint: fp.to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn delta_signer(name: &str, date: NaiveDate, chain_depth: u32, rationale: &str) -> Signer {
    // Per NFA-12 (committed schema invariant): at chain_depth=1, the
    // cumulative_root_fingerprint MUST equal prior_fingerprint. For
    // higher chain depths, cumulative_root tracks back to the original
    // Fresh basis. Test fixtures use identical values for both fields
    // so the chain-depth-1 invariant always holds — tests not exercising
    // that specific invariant don't need to construct multi-step chains.
    Signer {
        name: name.to_string(),
        role: None,
        date,
        signed_against_fingerprint: "fp-current".to_string(),
        basis: SignerBasis::DeltaFrom {
            prior_fingerprint: "fp-prior".to_string(),
            cumulative_root_fingerprint: "fp-prior".to_string(),
            chain_depth,
            rationale: rationale.to_string(),
        },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn make_ratification(signers: Vec<Signer>) -> Ratification {
    Ratification {
        schema_version: SchemaVersion::V1,
        kind: RatificationKind::Immunity,
        antigen: AntigenIdentifier {
            name: "TestAntigen".to_string(),
            defined_in: None,
        },
        source_file: PathBuf::from("src/test.rs"),
        items: vec![ItemRatification {
            item_path: "test_item".to_string(),
            current_fingerprint: "fp-current".to_string(),
            doc_ref: None,
            signers,
            oracles: vec![],
            fresh_through: None,
            extensions: BTreeMap::new(),
        }],
    }
}

// ============================================================================
// T2R-A: chain-depth cap
// ============================================================================

#[test]
fn t2r_a_chain_depth_at_cap_accepted() {
    // chain_depth == cap is INSIDE the allow-range. The schema invariant
    // is `chain_depth <= cap`, not `chain_depth < cap`. An off-by-one
    // regression that switches to `<` would fail here.
    let signer = delta_signer(
        "alice",
        sample_date(),
        DEFAULT_DELTA_CHAIN_CAP,
        VALID_RATIONALE,
    );
    let rat = make_ratification(vec![signer]);
    assert!(
        rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .is_ok(),
        "chain_depth == cap (boundary inside) was incorrectly rejected; \
         ADR-019 §M3 specifies <=, not <"
    );
}

#[test]
fn t2r_a_chain_depth_above_cap_rejected() {
    // chain_depth > cap is OUTSIDE the allow-range. Must fire
    // ChainDepthExceeded. A regression that fails to fire here re-opens
    // the unbounded-chain laundering surface.
    let signer = delta_signer(
        "alice",
        sample_date(),
        DEFAULT_DELTA_CHAIN_CAP + 1,
        VALID_RATIONALE,
    );
    let rat = make_ratification(vec![signer]);
    let err = rat
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("chain_depth > cap must be rejected (T2R-A)");
    match err {
        ValidationError::ChainDepthExceeded {
            chain_depth, cap, ..
        } => {
            assert_eq!(chain_depth, DEFAULT_DELTA_CHAIN_CAP + 1);
            assert_eq!(cap, DEFAULT_DELTA_CHAIN_CAP);
        }
        other => panic!(
            "expected ChainDepthExceeded, got {other:?}; \
             T2R-A regression: chain-depth cap not enforced"
        ),
    }
}

#[test]
fn t2r_a_workspace_config_cannot_exceed_hard_ceiling() {
    // Per adversarial T2R-C: workspace TOML cannot loosen the chain cap
    // above HARD_DELTA_CHAIN_CAP_MAX. If validate_chain_cap accepts a
    // cap > MAX, an admin could effectively disable the safeguard via
    // workspace config. This test FAILS if that protection regresses.
    let err = validate_chain_cap(HARD_DELTA_CHAIN_CAP_MAX + 1)
        .expect_err("validate_chain_cap must reject values > HARD_DELTA_CHAIN_CAP_MAX (T2R-C)");
    assert!(
        matches!(err, ValidationError::WorkspaceConfigOutOfBounds { .. }),
        "expected WorkspaceConfigOutOfBounds for above-ceiling cap, got {err:?}"
    );
}

#[test]
fn t2r_a_workspace_config_cannot_go_below_hard_floor() {
    // Anti-bypass: a workspace cannot set cap=0 to disable enforcement
    // entirely. HARD_DELTA_CHAIN_CAP_MIN = 1 (zero would mean "no delta
    // entries permitted ever," which is a different policy — but more
    // importantly, an admin attempting to bypass enforcement by setting
    // cap=0 to "disable" the check would silently allow unbounded chains
    // if validate_chain_cap accepted it).
    let err = validate_chain_cap(HARD_DELTA_CHAIN_CAP_MIN.saturating_sub(1))
        .expect_err("validate_chain_cap must reject values < HARD_DELTA_CHAIN_CAP_MIN");
    assert!(
        matches!(err, ValidationError::WorkspaceConfigOutOfBounds { .. }),
        "expected WorkspaceConfigOutOfBounds for below-floor cap, got {err:?}"
    );
}

#[test]
fn t2r_a_workspace_config_accepts_within_bounds() {
    // Sanity: values within [HARD_MIN, HARD_MAX] are accepted. Locks
    // the boundary behavior (off-by-one regressions here would prevent
    // valid configs from being accepted).
    assert!(validate_chain_cap(HARD_DELTA_CHAIN_CAP_MIN).is_ok());
    assert!(validate_chain_cap(DEFAULT_DELTA_CHAIN_CAP).is_ok());
    assert!(validate_chain_cap(HARD_DELTA_CHAIN_CAP_MAX).is_ok());
}

// ============================================================================
// T2R-B: rationale minimum length
// ============================================================================

#[test]
fn t2r_b_rationale_at_min_length_accepted() {
    // rationale.chars().count() == min_chars is INSIDE the allow-range
    // (>=, not >). An off-by-one regression to > would fire here.
    assert_eq!(
        MIN_VALID_RATIONALE.trim().chars().count(),
        DEFAULT_DELTA_RATIONALE_MIN_CHARS,
        "MIN_VALID_RATIONALE constant must be exactly {} chars trimmed",
        DEFAULT_DELTA_RATIONALE_MIN_CHARS,
    );
    let signer = delta_signer("alice", sample_date(), 1, MIN_VALID_RATIONALE);
    let rat = make_ratification(vec![signer]);
    assert!(
        rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .is_ok(),
        "rationale at exactly DEFAULT_DELTA_RATIONALE_MIN_CHARS was \
         rejected; ADR-019 T2R-B specifies >=, not >"
    );
}

#[test]
fn t2r_b_rationale_below_min_length_rejected() {
    // Rubber-stamp rationales (per T2R-B): "ok" (2), "fine" (4),
    // "reviewed" (8), "changes are safe" (16) all pass non-empty BUT
    // fall below the 20-char minimum. Each must be rejected.
    for rubber_stamp in &["ok", "fine", "reviewed", "changes are safe"] {
        let signer = delta_signer("alice", sample_date(), 1, rubber_stamp);
        let rat = make_ratification(vec![signer]);
        let err = rat
            .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .expect_err(&format!(
                "expected error for rubber-stamp rationale `{rubber_stamp}`, got Ok"
            ));
        assert!(
            matches!(err, ValidationError::RationaleTooShort { .. }),
            "expected RationaleTooShort for `{rubber_stamp}`, got {err:?}"
        );
    }
    // Lock the variant fields for the "ok" case specifically.
    let signer = delta_signer("alice", sample_date(), 1, "ok");
    let rat = make_ratification(vec![signer]);
    let err = rat
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("'ok' rationale must be rejected by T2R-B");
    match err {
        ValidationError::RationaleTooShort {
            actual_chars,
            min_chars,
            ..
        } => {
            assert_eq!(actual_chars, 2, "'ok' is 2 chars trimmed");
            assert_eq!(min_chars, DEFAULT_DELTA_RATIONALE_MIN_CHARS);
        }
        other => panic!("expected RationaleTooShort, got {other:?}"),
    }
}

#[test]
fn t2r_b_workspace_config_cannot_go_below_hard_floor() {
    // Per adversarial T2R-C extension to min-chars: workspace cannot
    // loosen the minimum below HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR.
    let err = validate_rationale_min_chars(HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR.saturating_sub(1))
        .expect_err("validate_rationale_min_chars must reject below-floor values");
    assert!(
        matches!(err, ValidationError::WorkspaceConfigOutOfBounds { .. }),
        "expected WorkspaceConfigOutOfBounds for below-floor min_chars, got {err:?}"
    );
}

#[test]
fn t2r_b_workspace_config_accepts_at_and_above_floor() {
    // Sanity: floor + default + tighter configurations are all accepted.
    assert!(validate_rationale_min_chars(HARD_DELTA_RATIONALE_MIN_CHARS_FLOOR).is_ok());
    assert!(validate_rationale_min_chars(DEFAULT_DELTA_RATIONALE_MIN_CHARS).is_ok());
    assert!(validate_rationale_min_chars(100).is_ok());
}

// ============================================================================
// T2R-C: non-empty rationale (independent of length)
// ============================================================================

#[test]
fn t2r_c_empty_rationale_rejected() {
    let signer = delta_signer("alice", sample_date(), 1, "");
    let rat = make_ratification(vec![signer]);
    let err = rat
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("empty rationale must be rejected by T2R-C");
    assert!(
        matches!(err, ValidationError::EmptyDeltaRationale { .. }),
        "expected EmptyDeltaRationale, got {err:?}"
    );
}

#[test]
fn t2r_c_whitespace_only_rationale_rejected() {
    // T2R-C operates on trimmed content — pure-whitespace rationales
    // (`"   \t\n  "`) must be rejected, not silently accepted as
    // "technically non-empty bytes."
    let signer = delta_signer("alice", sample_date(), 1, "   \t\n  ");
    let rat = make_ratification(vec![signer]);
    let err = rat
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("whitespace-only rationale must be rejected by T2R-C");
    assert!(
        matches!(err, ValidationError::EmptyDeltaRationale { .. }),
        "expected EmptyDeltaRationale for whitespace-only input, got {err:?}"
    );
}

// ============================================================================
// Three-safeguard interaction: removing any one re-opens the surface
// ============================================================================

#[test]
fn all_three_safeguards_together_close_the_surface() {
    // Per ADR-019 §Decision: "Removing any one re-opens the laundering
    // surface; the three together close it."
    //
    // This test exercises the happy-path: a delta entry that satisfies
    // ALL THREE safeguards (chain_depth <= cap, rationale >= min_chars
    // AND non-empty) is accepted. If a regression to any one safeguard
    // accidentally relaxes the policy, this test still passes — but
    // tests t2r_a/b/c above each fire to surface the specific gap.
    //
    // The point of THIS test is the converse: at the boundary of all
    // three, the schema is correctly permissive. If a regression to
    // OR-logic (any one safeguard satisfied passes) accidentally
    // relaxed the policy further, this would still pass but the
    // t2r_a/b/c failure-mode tests would surface it.
    let signer = delta_signer(
        "alice",
        sample_date(),
        DEFAULT_DELTA_CHAIN_CAP, // at-cap, allowed
        MIN_VALID_RATIONALE,     // at-min, allowed
    );
    let rat = make_ratification(vec![signer]);
    let result = rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS);
    assert!(
        result.is_ok(),
        "all-safeguards-satisfied delta entry was incorrectly rejected: {:?}",
        result.err()
    );
}

#[test]
fn signer_basis_zero_chain_depth_rejected() {
    // Schema invariant: chain_depth >= 1 for DeltaFrom entries
    // (chain_depth=0 is reserved for Fresh basis; a DeltaFrom with
    // chain_depth=0 would be a structural confusion).
    let signer = delta_signer("alice", sample_date(), 0, VALID_RATIONALE);
    let rat = make_ratification(vec![signer]);
    let err = rat
        .validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
        .expect_err("DeltaFrom with chain_depth=0 must be rejected");
    assert!(
        matches!(err, ValidationError::ZeroDeltaChainDepth { .. }),
        "expected ZeroDeltaChainDepth, got {err:?}"
    );
}

// ============================================================================
// Fresh basis is unaffected by delta safeguards
// ============================================================================

#[test]
fn fresh_basis_signers_unaffected_by_delta_safeguards() {
    // Fresh signers don't have rationale/chain_depth at all (the
    // optional Fresh.reasoning is documentation-of-review, not a
    // delta-justification). The delta safeguards apply ONLY to
    // SignerBasis::DeltaFrom. A regression that tried to apply the
    // 20-char rationale minimum or chain-depth cap to Fresh signers
    // would fail here.
    let fresh = fresh_signer("alice", sample_date(), "fp-current");
    let rat = make_ratification(vec![fresh]);
    assert!(
        rat.validate(DEFAULT_DELTA_CHAIN_CAP, DEFAULT_DELTA_RATIONALE_MIN_CHARS)
            .is_ok(),
        "Fresh-basis signer was rejected by delta-safeguard validation; \
         the safeguards must NOT apply to Fresh entries"
    );
}

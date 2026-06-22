//! Adversarial tests for the freshness bypass failure class.
//!
//! ## Failure class: perpetual-freshness-bypass-fresh-through
//!
//! `eval_fresh_within_days` accepts `fresh_through` as an anchor date even
//! when:
//!   - the sidecar has NO signers at all (NFA-23), or
//!   - the only signers are against a STALE fingerprint (NFA-23-variant),
//!
//! A sidecar writer can set `fresh_through = today` to satisfy
//! `fresh_within_days(N)` without a real reviewer re-attesting. This is
//! the witness-forgery class applied to temporal witnesses.
//!
//! ## ATK-FT-1: `fresh_through` with zero signers must NOT satisfy freshness
//!
//! FAILING TEST — asserts what the code SHOULD produce. The test currently
//! fails because `eval_fresh_within_days` accepts `fresh_through = today`
//! with an empty signer list and returns `WitnessTier::Execution`.
//!
//! Fix direction: `eval_fresh_within_days` should require at least one
//! signer entry signed against the current fingerprint before `fresh_through`
//! can anchor freshness. `fresh_through` alone (no signer) should result in
//! `DisciplinePredicateFailed`.
//!
//! ## ATK-FT-2: `fresh_through` with only stale-fingerprint signers must NOT satisfy freshness
//!
//! FAILING TEST — same bypass, different surface. Signers ARE present but
//! all against a stale fingerprint. `latest_signer` resolves to `None`
//! (NFA-21 filter), then the fallback `Some(fresh_through)` is used — the
//! forgery succeeds via the `fresh_through` anchor without any current
//! reviewer.
//!
//! Fix direction: same as ATK-FT-1. `fresh_through` must not substitute
//! for a current-fingerprint signer date.
//!
//! ## ATK-FT-3: `compare_versions` u64 overflow bypass
//!
//! FAILING TEST — `compare_versions` parses version components as `u64`.
//! A component string that exceeds `u64::MAX` causes `parse::<u64>()` to
//! return `Err`; `unwrap_or(0)` falls back to `0`. A sidecar's
//! `min_version = "18446744073709551616.0"` (`u64::MAX` + 1) is thus parsed
//! as `(0, ...) >= (doc_major, ...)` for any doc with major >= 1 — the
//! gate is vacuously satisfied. The fix: `validate()` must reject
//! `min_version` strings whose first component is out-of-range (non-numeric
//! or overflows `u64`).
//!
//! ## ATK-FT-4: `fresh_through` future date is correctly rejected
//!
//! PASSING TEST (regression guard) — a `fresh_through` date set in the
//! future (e.g., next year) makes `diff_days = today - future < 0`, which
//! is `!fresh`. Confirms the future-date bypass does not work (the
//! code's negative-diff guard is correct). This is an asymmetric fixture:
//! it only passes because the code is correct on this surface.
//!
//! ## Findings
//!
//! ATK-FT-1 and ATK-FT-2 cover the perpetual-freshness-bypass /
//! fresh-through path. ATK-FT-3 is a new finding
//! (`compare_versions` overflow).

use std::collections::BTreeMap;
use std::path::Path;

use antigen_attestation::{
    AuditHint, EvaluationContext, ItemRatification, Leaf, Predicate, SignatureStrength, Signer,
    SignerBasis, evaluate::evaluate_predicate,
};
use chrono::NaiveDate;

// ---------------------------------------------------------------------------
// Shared infrastructure — mirrors atk_a3_substrate_leaves pattern
// ---------------------------------------------------------------------------

struct Ctx {
    today: NaiveDate,
}

impl Ctx {
    const fn new(today: NaiveDate) -> Self {
        Self { today }
    }
}

impl EvaluationContext for Ctx {
    fn today(&self) -> NaiveDate {
        self.today
    }

    fn read_doc(&self, _path: &Path) -> Option<String> {
        None
    }

    fn read_oracle(&self, _path: &Path) -> Option<String> {
        None
    }

    fn read_git_trailers(&self, _item_source_file: &Path, _item_path: &str) -> Vec<String> {
        vec![]
    }

    fn delta_chain_cap(&self) -> u32 {
        3
    }
}

const fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).expect("hard-coded valid date")
}

const fn future_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2027, 5, 19).expect("hard-coded valid date")
}

fn stale_signer(name: &str) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date: today(), // signed today, but against the WRONG (stale) fingerprint
        signed_against_fingerprint: "fp-old".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn item_no_signers_fresh_through(fresh_through: Option<NaiveDate>) -> ItemRatification {
    ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers: vec![],
        oracles: vec![],
        fresh_through,
        extensions: BTreeMap::new(),
    }
}

fn item_stale_signers_fresh_through(fresh_through: Option<NaiveDate>) -> ItemRatification {
    ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers: vec![stale_signer("alice")],
        oracles: vec![],
        fresh_through,
        extensions: BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// ATK-FT-1: fresh_through alone (no signers) must NOT satisfy freshness
//
// FAILING TEST — asserts the CORRECT behavior; currently fails because
// eval_fresh_within_days accepts fresh_through with zero signers.
// ---------------------------------------------------------------------------
#[test]
fn atk_ft1_fresh_through_no_signers_must_not_satisfy_freshness() {
    // A sidecar with zero signers but fresh_through = today.
    // Nobody has attested, yet the freshness leaf is satisfied.
    // This test asserts that the correct behavior is FAILURE.
    //
    // CURRENTLY FAILS: the code returns WitnessTier::Execution (passing
    // fresh_within_days) even though no reviewer attested to the current
    // fingerprint. A sidecar writer who edits fresh_through = today can
    // bypass the freshness gate entirely without opening the files.
    let item = item_no_signers_fresh_through(Some(today()));
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 60 });
    let ctx = Ctx::new(today());
    let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicateFailed,
        "ATK-FT-1: fresh_through with zero signers must NOT satisfy fresh_within_days; \
         a sidecar writer who edits fresh_through=today bypasses review entirely; \
         fix: require at least one current-fingerprint signer for fresh_through to count"
    );
}

// ---------------------------------------------------------------------------
// ATK-FT-2: fresh_through + only stale-fingerprint signers — freshness leaf
//           passes but overall result is SubstrateStale, not a clean failure
//
// FAILING TEST — nuanced bypass variant. NFA-21 correctly filters out stale
// signers, leaving latest_signer = None. The fallback to fresh_through=today
// then satisfies the freshness LEAF, making the predicate pass. Then
// classify_passed_predicate detects all signers are stale and returns
// DisciplineSubstrateStale instead of DisciplinePredicateFailed.
//
// The result is ambiguous: it looks like "stale for other reasons" rather
// than "nobody reviewed the current fingerprint for freshness." An auditor
// reading SubstrateStale might interpret this as "someone reviewed but
// didn't update" rather than "the freshness check was bypassed via fresh_through."
//
// Correct behavior: when fresh_through is the ONLY freshness anchor (no
// current-fingerprint signer date exists), the result should be
// DisciplinePredicateFailed (freshness gate not met), not SubstrateStale.
// The distinction matters: SubstrateStale says "reviewed but old";
// PredicateFailed says "freshness requirement not met."
// ---------------------------------------------------------------------------
#[test]
fn atk_ft2_fresh_through_with_stale_signers_only_gives_ambiguous_result() {
    // Alice signed today but against fp-old (stale fingerprint).
    // NFA-21 correctly filters her out from latest_signer.
    // fresh_through = today is used as sole anchor → freshness leaf passes.
    // classify_passed_predicate then returns SubstrateStale (not PredicateFailed).
    // The freshness bypass succeeds — just at a lower tier than Execution.
    let item = item_stale_signers_fresh_through(Some(today()));
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 60 });
    let ctx = Ctx::new(today());
    let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
    // CORRECT behavior: should be DisciplinePredicateFailed (freshness not met).
    // ACTUAL behavior: DisciplineSubstrateStale (freshness leaf passed, staleness
    // detected afterward — ambiguous signal, not a clean freshness rejection).
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicateFailed,
        "ATK-FT-2: fresh_through with stale-only signers must return DisciplinePredicateFailed, \
         not DisciplineSubstrateStale; the SubstrateStale result is ambiguous — it looks like \
         staleness rather than freshness bypass; fix: when fresh_through is the sole anchor \
         (no current-fp signer date), treat as freshness failure, not stale review"
    );
}

// ---------------------------------------------------------------------------
// ATK-FT-3: compare_versions u64 overflow — huge min_version vacuously passes
//
// FAILING TEST — a min_version component larger than u64::MAX is parsed
// by compare_versions as 0 (unwrap_or(0) on parse failure). Any document
// with a real version >= "1.0" then satisfies the gate, but the gate was
// intended to require a version that is unobtainably large.
//
// Attack: sidecar declares min_version = "18446744073709551616.0" (u64::MAX+1).
// compare_versions("1.0", "18446744073709551616.0"):
//   a = [(1, "1"), (0, "0")], b = [(0, "18446744073709551616"), (0, "0")]
//   Comparison: a[0].0=1 > b[0].0=0 → Greater → gate PASSES.
// But the requirement was: doc version >= 1.844×10^19 (impossible).
// The huge version string silently lowered the bar to 0.
//
// Fix direction: validate() must reject min_version strings whose components
// cannot be parsed as u64 (non-numeric or overflow); a PredicateParseError
// variant should cover "version component not a valid u64".
// ---------------------------------------------------------------------------
#[test]
fn atk_ft3_compare_versions_u64_overflow_vacuous_bypass() {
    // min_version = u64::MAX + 1 — a string that overflows u64 parse.
    // This should be REJECTED by validate() because the version component
    // is not a valid u64 and compare_versions would silently treat it as 0.
    let huge_version = format!("{}.0", u128::from(u64::MAX) + 1); // "18446744073709551616.0"
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(std::path::PathBuf::from("docs/test.md")),
        min_version: Some(huge_version.clone()),
        anchor: None,
        sibling_json: false,
    });
    let err = pred.validate();
    assert!(
        err.is_err(),
        "ATK-FT-3: validate() must reject min_version '{huge_version}' \
         whose major component overflows u64; compare_versions parses it as 0, \
         which causes any document version >= 1.0 to vacuously satisfy the gate; \
         fix: reject non-u64-parseable version components in validate()"
    );
}

// ---------------------------------------------------------------------------
// ATK-FT-4: future fresh_through is correctly rejected (regression guard)
//
// PASSING TEST — a fresh_through date set in the future does NOT bypass
// freshness (diff_days = today - future < 0, so fresh = false).
// This is an asymmetric fixture: it only passes when the code is correct
// on this surface. If someone removes the `diff_days >= 0` guard, this
// test catches the regression.
// ---------------------------------------------------------------------------
#[test]
fn atk_ft4_future_fresh_through_is_not_fresh() {
    // fresh_through set a year in the future.
    // diff_days = today - future_date = negative → not fresh.
    // This should NOT be treated as "very fresh" — it should fail
    // (the item's freshness has not yet been established).
    let item = item_no_signers_fresh_through(Some(future_date()));
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 60 });
    let ctx = Ctx::new(today());
    let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicateFailed,
        "ATK-FT-4 (regression guard): future fresh_through must NOT satisfy freshness; \
         diff_days = today - future < 0; the negative-diff guard must hold; \
         if this fails, someone removed the `diff_days >= 0` check"
    );
}

//! Adversarial precision test for the v0.1 substrate-witness leaf primitives
//! (P3e slice 5 / 7).
//!
//! ## What this test guards
//!
//! ADR-019 §Decision seals the v0.1 leaf set at exactly FIVE primitives:
//!
//! 1. `ratified_doc(path?, min_version?, anchor?, sibling_json?)`
//! 2. `signers(required, roles?, against?, signature_allow?, signature_prefer?)`
//! 3. `signed_trailer(key, role?, count?)`
//! 4. `oracles_complete(files)`
//! 5. `fresh_within_days(n)`
//!
//! Each leaf gets a positive case (pass), a negative case (fail), and a
//! shape-edge or contract-boundary lock. Together with the closed combinator
//! grammar in [`Predicate`] and the parser-boundary lock in
//! `atk_a3_unification_guardrail`, this corpus prevents the leaf set from
//! silently growing or shrinking under refactor.
//!
//! ## Why each leaf needs its own slice
//!
//! Leaves are the IO boundary of substrate-witness evaluation. Each one
//! invokes a distinct `EvaluationContext` method (or none, for `signers` /
//! `fresh_within_days` which read sidecar-internal state). A regression
//! that breaks one leaf without touching the others is the high-probability
//! refactor-induced silent failure. Per-leaf lock-down catches that.
//!
//! Note: detailed delta-chain anti-laundering for `signers` is covered in
//! [`atk_a3_delta_chain_caps`]; tolerance-kind discrimination is in
//! [`atk_a3_tolerance_audit_hint`]; the unification guardrail is in
//! [`atk_a3_unification_guardrail`]. This slice covers each leaf's basic
//! pass/fail contract + serde tag-name stability.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use antigen_attestation::{
    evaluate::evaluate_predicate, predicate::SignerCurrency, AuditHint, EvaluationContext,
    EvidenceKind, ItemRatification, Leaf, Predicate, SignatureStrength, Signer, SignerBasis,
    WitnessTier,
};
use chrono::NaiveDate;

// --- Shared infrastructure ---

struct LeafCtx {
    today: NaiveDate,
    docs: BTreeMap<PathBuf, String>,
    oracles: BTreeMap<PathBuf, String>,
    trailers: BTreeMap<(PathBuf, String), Vec<String>>,
    cap: u32,
}

impl LeafCtx {
    const fn new(today: NaiveDate) -> Self {
        Self {
            today,
            docs: BTreeMap::new(),
            oracles: BTreeMap::new(),
            trailers: BTreeMap::new(),
            cap: 3,
        }
    }
    fn with_doc(mut self, path: &str, content: &str) -> Self {
        self.docs.insert(PathBuf::from(path), content.to_string());
        self
    }
    fn with_oracle(mut self, path: &str, content: &str) -> Self {
        self.oracles
            .insert(PathBuf::from(path), content.to_string());
        self
    }
    fn with_trailers(mut self, file: &str, item: &str, trailers: Vec<&str>) -> Self {
        self.trailers.insert(
            (PathBuf::from(file), item.to_string()),
            trailers.into_iter().map(String::from).collect(),
        );
        self
    }
}

impl EvaluationContext for LeafCtx {
    fn today(&self) -> NaiveDate {
        self.today
    }
    fn read_doc(&self, path: &Path) -> Option<String> {
        self.docs.get(path).cloned()
    }
    fn read_oracle(&self, path: &Path) -> Option<String> {
        self.oracles.get(path).cloned()
    }
    fn read_git_trailers(&self, item_source_file: &Path, item_path: &str) -> Vec<String> {
        self.trailers
            .get(&(item_source_file.to_path_buf(), item_path.to_string()))
            .cloned()
            .unwrap_or_default()
    }
    fn delta_chain_cap(&self) -> u32 {
        self.cap
    }
}

const fn sample_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 5, 19).expect("hard-coded valid date")
}

fn current_signer(name: &str, date: NaiveDate) -> Signer {
    Signer {
        name: name.to_string(),
        role: None,
        date,
        signed_against_fingerprint: "fp-current".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    }
}

fn item_with(signers: Vec<Signer>) -> ItemRatification {
    ItemRatification {
        item_path: "sinh".to_string(),
        current_fingerprint: "fp-current".to_string(),
        doc_ref: None,
        signers,
        oracles: vec![],
        fresh_through: None,
        extensions: BTreeMap::new(),
    }
}

fn passes(p: &Predicate, item: &ItemRatification, ctx: &LeafCtx) -> bool {
    let r = evaluate_predicate(p, item, "fp-current", Path::new("src/test.rs"), ctx).unwrap();
    matches!(
        r.audit_hint,
        AuditHint::DisciplinePredicatePassedSubstrateCurrent
    )
}

fn fails(p: &Predicate, item: &ItemRatification, ctx: &LeafCtx) -> bool {
    let r = evaluate_predicate(p, item, "fp-current", Path::new("src/test.rs"), ctx).unwrap();
    matches!(
        r.audit_hint,
        AuditHint::DisciplinePredicateFailed | AuditHint::DisciplineSubstrateStale
    ) && r.witness_tier != WitnessTier::Execution
}

/// ADR-035 leaf-sweep: a substrate-absent / input-unreadable leaf is ⊥
/// (could-not-evaluate), which surfaces as `DisciplinePredicateDeferred` — NOT
/// `DisciplinePredicateFailed`. The two `*_when_*_missing` tests below assert
/// this corrected three-valued behavior (they previously asserted `fails`,
/// which encoded the ⊥→false collapse the leaf-sweep closes).
fn not_evaluated(p: &Predicate, item: &ItemRatification, ctx: &LeafCtx) -> bool {
    let r = evaluate_predicate(p, item, "fp-current", Path::new("src/test.rs"), ctx).unwrap();
    matches!(r.audit_hint, AuditHint::DisciplinePredicateDeferred)
        && r.witness_tier != WitnessTier::Execution
}

// ============================================================================
// Leaf 1: ratified_doc
// ============================================================================

#[test]
fn ratified_doc_passes_when_doc_exists_and_min_version_met() {
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(PathBuf::from("docs/discipline.md")),
        min_version: Some("1.0.0".to_string()),
        anchor: None,
        sibling_json: false,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date()).with_doc(
        "docs/discipline.md",
        "---\nversion: 1.2.0\n---\n# Discipline doc body\n",
    );
    assert!(
        passes(&pred, &item, &ctx),
        "doc exists at version 1.2.0 ≥ 1.0.0 must pass"
    );
}

#[test]
fn ratified_doc_not_evaluated_when_doc_path_missing() {
    // ADR-035 leaf-sweep: an absent doc is ⊥ (the substrate could not be read),
    // NOT a genuine evaluated-and-failed. The leaf must be Deferred
    // (evaluated:false), not Failed — collapsing ⊥ to a failure is the Shape-C
    // lie the leaf-sweep closes (forward/adr035-leaf-sweep-bottom-to-false).
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(PathBuf::from("docs/nonexistent.md")),
        min_version: None,
        anchor: None,
        sibling_json: false,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        not_evaluated(&pred, &item, &ctx),
        "missing doc must be un-evaluable (⊥ → Deferred), not a failure"
    );
}

#[test]
fn ratified_doc_fails_when_min_version_unmet() {
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(PathBuf::from("docs/discipline.md")),
        min_version: Some("2.0.0".to_string()),
        anchor: None,
        sibling_json: false,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date())
        .with_doc("docs/discipline.md", "---\nversion: 1.2.0\n---\n# body\n");
    assert!(fails(&pred, &item, &ctx), "version 1.2.0 < 2.0.0 must fail");
}

#[test]
fn ratified_doc_anchor_check_finds_anchor_substring() {
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(PathBuf::from("docs/d.md")),
        min_version: None,
        anchor: Some("## ratchet-discipline".to_string()),
        sibling_json: false,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx =
        LeafCtx::new(sample_date()).with_doc("docs/d.md", "# Doc\n## ratchet-discipline\nbody\n");
    assert!(
        passes(&pred, &item, &ctx),
        "anchor substring present must pass"
    );
}

#[test]
fn ratified_doc_anchor_check_fails_when_anchor_absent() {
    let pred = Predicate::leaf(Leaf::RatifiedDoc {
        path: Some(PathBuf::from("docs/d.md")),
        min_version: None,
        anchor: Some("## different-anchor".to_string()),
        sibling_json: false,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date()).with_doc("docs/d.md", "# Doc\n## something-else\n");
    assert!(
        fails(&pred, &item, &ctx),
        "anchor substring absent must fail"
    );
}

// ============================================================================
// Leaf 2: signers
// ============================================================================

#[test]
fn signers_passes_with_required_name_against_current() {
    let pred = Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: None,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(passes(&pred, &item, &ctx));
}

#[test]
fn signers_fails_when_required_name_absent() {
    let pred = Predicate::leaf(Leaf::Signers {
        required: vec!["bob".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: None,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        fails(&pred, &item, &ctx),
        "required signer not in list must fail"
    );
}

#[test]
fn signers_fails_when_role_unmet_jointly_with_currency() {
    // NFA-13 regression: role and currency must be evaluated JOINTLY per
    // candidate, not independently. Locked here at the leaf-test layer.
    let pred = Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: {
            let mut m = BTreeMap::new();
            m.insert("alice".to_string(), "reviewer".to_string());
            m
        },
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: None,
    });
    let alice_stale_with_role = Signer {
        name: "alice".to_string(),
        role: Some("reviewer".to_string()),
        date: sample_date(),
        signed_against_fingerprint: "fp-OLD".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    };
    let alice_current_no_role = current_signer("alice", sample_date());
    let item = item_with(vec![alice_stale_with_role, alice_current_no_role]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        fails(&pred, &item, &ctx),
        "no single alice entry is BOTH current AND reviewer; NFA-13 lock"
    );
}

#[test]
fn signers_against_any_passes_with_stale_signature() {
    // `against=any` accepts stale signatures (semantic: "this name has ever signed").
    let pred = Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Any,
        signature_allow: vec![],
        signature_prefer: None,
    });
    let alice_stale = Signer {
        name: "alice".to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: "fp-old".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    };
    let item = item_with(vec![alice_stale]);
    let ctx = LeafCtx::new(sample_date());
    // Predicate passes (against=Any matched). The classifier sees stale signers
    // and emits the stale hint, not the passed-substrate-current hint. Both are
    // "predicate evaluated to true", but the post-classification surfaces the
    // staleness. This is the correct ratchet-asymmetry behavior.
    let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
    assert_eq!(r.audit_hint, AuditHint::DisciplineSubstrateStale);
    assert_eq!(r.witness_tier, WitnessTier::Reachability);
}

#[test]
fn signers_signature_allow_filter_rejects_disallowed_strength() {
    let pred = Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![SignatureStrength::CryptoSigned],
        signature_prefer: None,
    });
    // alice signs with GitTrust but allow-list only permits CryptoSigned.
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        fails(&pred, &item, &ctx),
        "signature_allow=[CryptoSigned] must reject GitTrust signers"
    );
}

// ATK-A3-SIGNATURE-PREFER-NOOP: v0.1 documents `signature_prefer` as advisory
// (accepted, stored, no runtime effect). This test locks that contract so a
// future refactor cannot silently promote it to a hard gate.
#[test]
fn signers_signature_prefer_has_no_runtime_effect_v01() {
    // signature_prefer = CryptoSigned, but signer only has GitTrust strength.
    // If signature_prefer were enforced, this would fail. It must pass.
    let pred = Predicate::leaf(Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: Some(SignatureStrength::CryptoSigned),
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        passes(&pred, &item, &ctx),
        "ATK-A3-SIGNATURE-PREFER-NOOP: signature_prefer=CryptoSigned must not \
         gate a GitTrust signer in v0.1; it is advisory-only. If this fails, \
         signature_prefer was silently promoted to a hard gate — that is a \
         behavior change requiring an ADR and explicit version bump."
    );
}

// ============================================================================
// Leaf 3: signed_trailer
// ============================================================================

#[test]
fn signed_trailer_passes_with_matching_trailer_present() {
    let pred = Predicate::leaf(Leaf::SignedTrailer {
        key: "Discipline-Verified-By".to_string(),
        role: None,
        count: 1,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date()).with_trailers(
        "src/test.rs",
        "sinh",
        vec!["Discipline-Verified-By: alice <a@x>"],
    );
    assert!(passes(&pred, &item, &ctx));
}

#[test]
fn signed_trailer_fails_with_no_matching_trailer() {
    let pred = Predicate::leaf(Leaf::SignedTrailer {
        key: "Discipline-Verified-By".to_string(),
        role: None,
        count: 1,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(fails(&pred, &item, &ctx));
}

#[test]
fn signed_trailer_count_two_fails_with_only_one_match() {
    let pred = Predicate::leaf(Leaf::SignedTrailer {
        key: "Discipline-Verified-By".to_string(),
        role: None,
        count: 2,
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date()).with_trailers(
        "src/test.rs",
        "sinh",
        vec!["Discipline-Verified-By: alice <a@x>"],
    );
    assert!(
        fails(&pred, &item, &ctx),
        "count=2 with one match must fail (threshold contract)"
    );
}

#[test]
fn signed_trailer_count_zero_is_schema_invalid_at_validate() {
    // count=0 would be vacuously satisfied (no trailers required to match);
    // it's caught at predicate validation, not at evaluation. Lock the validate
    // boundary as the rejection mechanism.
    let pred = Predicate::leaf(Leaf::SignedTrailer {
        key: "Discipline-Verified-By".to_string(),
        role: None,
        count: 0,
    });
    let r = pred.validate();
    assert!(
        r.is_err(),
        "count=0 is a vacuous-trailer schema bug; validate must reject"
    );
}

// ============================================================================
// Leaf 4: oracles_complete
// ============================================================================

#[test]
fn oracles_complete_passes_with_status_complete_oracle() {
    let pred = Predicate::leaf(Leaf::OraclesComplete {
        files: vec![PathBuf::from("docs/oracles/o.md")],
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date())
        .with_oracle("docs/oracles/o.md", "---\nstatus: complete\n---\nbody\n");
    assert!(passes(&pred, &item, &ctx));
}

#[test]
fn oracles_complete_not_evaluated_when_oracle_file_missing() {
    // ADR-035 leaf-sweep instance 4: an ABSENT oracle file is ⊥ (the file could
    // not be read), NOT a genuine evaluated-and-failed. The leaf must be Deferred
    // (evaluated:false), distinct from a PRESENT-but-incomplete oracle (the
    // sibling test below, which genuinely fails). The old fused
    // "missing OR not-complete → fail" arm collapsed these.
    let pred = Predicate::leaf(Leaf::OraclesComplete {
        files: vec![PathBuf::from("docs/oracles/missing.md")],
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        not_evaluated(&pred, &item, &ctx),
        "an absent oracle file must be un-evaluable (⊥ → Deferred), not a failure"
    );
}

#[test]
fn oracles_complete_fails_when_oracle_status_not_complete() {
    let pred = Predicate::leaf(Leaf::OraclesComplete {
        files: vec![PathBuf::from("docs/oracles/o.md")],
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date())
        .with_oracle("docs/oracles/o.md", "---\nstatus: pending\n---\nbody\n");
    assert!(
        fails(&pred, &item, &ctx),
        "status:pending must fail oracles_complete"
    );
}

#[test]
fn oracles_complete_empty_files_is_schema_invalid_at_validate() {
    // Empty oracle list is vacuous (zero things to check ≡ passes). The validate
    // step must reject before evaluation.
    let pred = Predicate::leaf(Leaf::OraclesComplete { files: vec![] });
    let r = pred.validate();
    assert!(
        r.is_err(),
        "empty oracles list is a vacuous-bypass; validate must reject"
    );
}

#[test]
fn oracles_complete_two_files_fails_if_one_incomplete() {
    let pred = Predicate::leaf(Leaf::OraclesComplete {
        files: vec![
            PathBuf::from("docs/oracles/a.md"),
            PathBuf::from("docs/oracles/b.md"),
        ],
    });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date())
        .with_oracle("docs/oracles/a.md", "---\nstatus: complete\n---\n")
        .with_oracle("docs/oracles/b.md", "---\nstatus: in-progress\n---\n");
    assert!(
        fails(&pred, &item, &ctx),
        "ALL listed oracles must be complete; one incomplete must fail"
    );
}

// ============================================================================
// Leaf 5: fresh_within_days
// ============================================================================

#[test]
fn fresh_within_days_passes_with_recent_signature() {
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        passes(&pred, &item, &ctx),
        "today's signature within 30 days must pass"
    );
}

#[test]
fn fresh_within_days_fails_with_old_signature() {
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
    let old_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let item = item_with(vec![current_signer("alice", old_date)]);
    let ctx = LeafCtx::new(sample_date());
    assert!(
        fails(&pred, &item, &ctx),
        "2025-01-01 signature with today=2026-05-19 must fail 30-day freshness"
    );
}

#[test]
fn fresh_within_days_uses_only_current_fingerprint_signers_nfa21() {
    // NFA-21: stale-fingerprint signer dates must NOT satisfy freshness.
    // alice's old signature against fp-OLD is dated today, but doesn't count
    // toward fp-current freshness.
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
    let alice_stale_today = Signer {
        name: "alice".to_string(),
        role: None,
        date: sample_date(),
        signed_against_fingerprint: "fp-OLD".to_string(),
        basis: SignerBasis::Fresh { reasoning: None },
        strength: SignatureStrength::GitTrust,
        signature: None,
    };
    let item = item_with(vec![alice_stale_today]);
    let ctx = LeafCtx::new(sample_date());
    let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
    // Predicate fails (no current-fp signer freshness datum).
    assert_eq!(r.audit_hint, AuditHint::DisciplinePredicateFailed);
}

#[test]
fn fresh_within_days_with_no_signers_fails() {
    let pred = Predicate::leaf(Leaf::FreshWithinDays { days: 30 });
    let item = item_with(vec![]);
    let ctx = LeafCtx::new(sample_date());
    let r = evaluate_predicate(&pred, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
    assert_eq!(
        r.audit_hint,
        AuditHint::DisciplinePredicateFailed,
        "no signers means no date to check; freshness must fail (no vacuous pass)"
    );
}

// ============================================================================
// Leaf serde: tag-name stability (the public wire format)
// ============================================================================
//
// The internally-tagged enum uses `tag = "name"` with snake_case rename. These
// tag names are the SIDECAR JSON contract — any rename silently breaks all
// existing on-disk sidecars. Lock them.

#[test]
fn leaf_tag_ratified_doc_serializes_snake_case() {
    let l = Leaf::RatifiedDoc {
        path: None,
        min_version: None,
        anchor: None,
        sibling_json: false,
    };
    let s = serde_json::to_string(&l).unwrap();
    assert!(
        s.contains("\"name\":\"ratified_doc\""),
        "tag name `ratified_doc` is wire-format contract: {s}"
    );
}

#[test]
fn leaf_tag_signers_serializes_snake_case() {
    let l = Leaf::Signers {
        required: vec!["alice".to_string()],
        roles: BTreeMap::new(),
        against: SignerCurrency::Current,
        signature_allow: vec![],
        signature_prefer: None,
    };
    let s = serde_json::to_string(&l).unwrap();
    assert!(
        s.contains("\"name\":\"signers\""),
        "tag name `signers`: {s}"
    );
}

#[test]
fn leaf_tag_signed_trailer_serializes_snake_case() {
    let l = Leaf::SignedTrailer {
        key: "K".to_string(),
        role: None,
        count: 1,
    };
    let s = serde_json::to_string(&l).unwrap();
    assert!(
        s.contains("\"name\":\"signed_trailer\""),
        "tag name `signed_trailer`: {s}"
    );
}

#[test]
fn leaf_tag_oracles_complete_serializes_snake_case() {
    let l = Leaf::OraclesComplete {
        files: vec![PathBuf::from("o.md")],
    };
    let s = serde_json::to_string(&l).unwrap();
    assert!(
        s.contains("\"name\":\"oracles_complete\""),
        "tag name `oracles_complete`: {s}"
    );
}

#[test]
fn leaf_tag_fresh_within_days_serializes_snake_case() {
    let l = Leaf::FreshWithinDays { days: 30 };
    let s = serde_json::to_string(&l).unwrap();
    assert!(
        s.contains("\"name\":\"fresh_within_days\""),
        "tag name `fresh_within_days`: {s}"
    );
}

// ============================================================================
// Leaf set exhaustivity: exactly ten variants exist (v0.2)
// ============================================================================
//
// Per ADR-019 §Decision (v0.1 sealed at 5 primitives) AND ADR-025 §Substrate-
// witness-leaves (+5 supply-chain primitives in v0.2), the leaf set is now
// exactly 10. A future amendment that adds an 11th leaf MUST update the
// relevant ADR, this test, the unification guardrail test, and every
// parser/evaluator branch. The match below is `exhaustive` (no _ pattern)
// so the compiler forces the review.

#[test]
fn leaf_set_exhaustivity_ten_variants() {
    let leaves = [
        Leaf::RatifiedDoc {
            path: None,
            min_version: None,
            anchor: None,
            sibling_json: false,
        },
        Leaf::Signers {
            required: vec![],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        },
        Leaf::SignedTrailer {
            key: String::new(),
            role: None,
            count: 1,
        },
        Leaf::OraclesComplete { files: vec![] },
        Leaf::FreshWithinDays { days: 0 },
        // ADR-025 Supply-Chain Defense Family additions
        Leaf::DepPinned { crate_name: None },
        Leaf::DepAttested {
            crate_name: "serde".to_string(),
            version: "1.0.0".to_string(),
            exact_version: true,
            reviewable_artifact: None,
        },
        Leaf::MaintainerUnchanged {
            crate_name: "serde".to_string(),
            since_version: "1.0.0".to_string(),
        },
        Leaf::ContentHashMatches {
            crate_name: "serde".to_string(),
            version: "1.0.0".to_string(),
        },
        Leaf::SandboxClean {
            crate_name: "serde".to_string(),
            sandbox_kind: "build".to_string(),
        },
    ];
    assert_eq!(
        leaves.len(),
        10,
        "ADR-019 v0.1 (5 primitives) + ADR-025 v0.2 supply-chain (5 primitives) = 10"
    );
    for leaf in &leaves {
        // Exhaustive match — any new variant breaks compilation here.
        match leaf {
            Leaf::RatifiedDoc { .. }
            | Leaf::Signers { .. }
            | Leaf::SignedTrailer { .. }
            | Leaf::OraclesComplete { .. }
            | Leaf::FreshWithinDays { .. }
            | Leaf::DepPinned { .. }
            | Leaf::DepAttested { .. }
            | Leaf::MaintainerUnchanged { .. }
            | Leaf::ContentHashMatches { .. }
            | Leaf::SandboxClean { .. } => {}
        }
    }
}

// ============================================================================
// EvidenceKind on all-leaves: every leaf reports SubstrateState
// ============================================================================
//
// Per ADR-019 §Decision, all v0.1 leaves are substrate-witnesses → every
// passing leaf MUST report `EvidenceKind::SubstrateState`. If a future leaf
// reports a different kind, the categorical commitment is broken.

#[test]
fn all_v01_leaves_when_passing_report_substrate_state_evidence_kind() {
    let item = item_with(vec![current_signer("alice", sample_date())]);
    let ctx = LeafCtx::new(sample_date())
        .with_doc("docs/d.md", "---\nversion: 1.0.0\n---\n")
        .with_oracle("docs/oracles/o.md", "---\nstatus: complete\n---\n")
        .with_trailers("src/test.rs", "sinh", vec!["K: alice"]);

    let leaves: Vec<Predicate> = vec![
        Predicate::leaf(Leaf::RatifiedDoc {
            path: Some(PathBuf::from("docs/d.md")),
            min_version: Some("1.0.0".to_string()),
            anchor: None,
            sibling_json: false,
        }),
        Predicate::leaf(Leaf::Signers {
            required: vec!["alice".to_string()],
            roles: BTreeMap::new(),
            against: SignerCurrency::Current,
            signature_allow: vec![],
            signature_prefer: None,
        }),
        Predicate::leaf(Leaf::SignedTrailer {
            key: "K".to_string(),
            role: None,
            count: 1,
        }),
        Predicate::leaf(Leaf::OraclesComplete {
            files: vec![PathBuf::from("docs/oracles/o.md")],
        }),
        Predicate::leaf(Leaf::FreshWithinDays { days: 90 }),
    ];

    for p in &leaves {
        let r = evaluate_predicate(p, &item, "fp-current", Path::new("src/test.rs"), &ctx).unwrap();
        assert_eq!(
            r.evidence_kind,
            EvidenceKind::SubstrateState,
            "every v0.1 leaf MUST report SubstrateState; predicate: {p:?}"
        );
    }
}

//! ADVERSARIAL SURVEY ATTACKS — the-frame survey wave (adversarial role).
//!
//! These are the attacks I'm mounting to try to BREAK the frame's claims. Each is labeled:
//!   - ATK: an adversarial case that SHOULD reject a bad input / hold an invariant
//!   - NC: a negative control that SHOULD pass (proves the test has teeth)
//!   - PROBE: a probe with a comment — if behavior is surprising, that's a finding
//!
//! Default-to-refuted: each passing ATK means the frame holds for that input.
//! Any unexpected result is a FINDING to escalate.

use antigen_stroma::fidelity::FidelityWitness;
use antigen_stroma::node::digest::IdentityDigest;
use antigen_stroma::node::node::Revision;
use antigen_stroma::node::path::{refine_with_scip, syntactic_fq_path};
use antigen_stroma::read::{ResolutionTier, corroborate_presents};

// ── ATTACK 1: FQ-identity — degenerate inputs ──────────────────────────────────────────────────────

// ATTACK 1a: empty crate name — must not panic, must include item name.
#[test]
fn atk_survey_empty_crate_name_does_not_panic() {
    let p = syntactic_fq_path("", &[], "item");
    assert!(
        p.path.contains("item"),
        "empty crate name broke path construction: got '{}'",
        p.path
    );
    assert_eq!(p.tier, ResolutionTier::Syntactic);
}

// ATTACK 1b: empty item name — MUST be distinct from a real item name in the same module.
// If empty-name == real-name path, any module's item could be aliased by an empty-name emission.
#[test]
fn atk_survey_empty_item_name_distinct_from_real_item() {
    let real = syntactic_fq_path("crate", &["mod".to_string()], "MyStruct");
    let empty = syntactic_fq_path("crate", &["mod".to_string()], "");
    assert_ne!(
        real, empty,
        "FINDING: empty item name collides with a real item name — silent alias attack is possible."
    );
}

// ATTACK 1c: two empty item names in the SAME module must be EQUAL (deterministic, not counter-based).
#[test]
fn atk_survey_empty_item_name_is_deterministic() {
    let a = syntactic_fq_path("crate", &["mod".to_string()], "");
    let b = syntactic_fq_path("crate", &["mod".to_string()], "");
    assert_eq!(
        a, b,
        "Empty item name path must be deterministic — same inputs, same output."
    );
}

// ATTACK 1d: Unicode item name — no collision with ASCII cousin.
#[test]
fn atk_survey_unicode_item_name_no_collision_with_ascii() {
    let ascii = syntactic_fq_path("crate", &["mod".to_string()], "item");
    let unicode = syntactic_fq_path("crate", &["mod".to_string()], "\u{00ED}tem"); // accented i
    assert_ne!(
        ascii, unicode,
        "FINDING: Unicode item name collides with ASCII cousin — path is losing encoding."
    );
}

// ATTACK 1e: deeply nested module chain (100 levels) — no panic, correct path shape.
#[test]
fn atk_survey_deeply_nested_module_chain_no_panic() {
    let deep_chain: Vec<String> = (0..100).map(|i| format!("m{i}")).collect();
    let p = syntactic_fq_path("crate", &deep_chain, "item");
    assert!(
        p.path.starts_with("crate::m0::m1"),
        "path must start with crate::m0::m1, got: '{}'",
        p.path
    );
    assert!(
        p.path.ends_with("::m99::item"),
        "path must end with ::m99::item, got: '{}'",
        p.path
    );
    assert_eq!(p.tier, ResolutionTier::Syntactic);
}

// ATTACK 1f: 1000 items with the SAME item name in DIFFERENT modules — all paths must be distinct.
// A construction that only uses the item name (not the module chain) would hash-collide these.
#[test]
fn atk_survey_same_item_name_1000_modules_all_distinct() {
    use std::collections::HashSet;
    let paths: HashSet<String> = (0..1000)
        .map(|i| syntactic_fq_path("crate", &[format!("mod{i}")], "item").path)
        .collect();
    assert_eq!(
        paths.len(),
        1000,
        "FINDING: some module::item paths collide across 1000 distinct modules — \
         the construction is dropping the module chain."
    );
}

// ATTACK 1g: same crate name, different item names, empty module chain — still distinct.
#[test]
fn atk_survey_two_items_at_crate_root_are_distinct() {
    let a = syntactic_fq_path("crate", &[], "foo");
    let b = syntactic_fq_path("crate", &[], "bar");
    assert_ne!(
        a, b,
        "FINDING: crate::foo and crate::bar collide at the crate root."
    );
}

// ── ATTACK 2: malformed SCIP symbol fall-through ──────────────────────────────────────────────────

// ATTACK 2a: whitespace-only SCIP symbol — MUST fall through to syntactic.
#[test]
fn atk_survey_whitespace_only_scip_symbol_falls_through() {
    let syntactic = syntactic_fq_path("crate", &["mod".to_string()], "item");
    let refined = refine_with_scip(syntactic.clone(), Some("   "));
    assert_eq!(
        refined.tier,
        ResolutionTier::Syntactic,
        "FINDING: whitespace-only SCIP symbol was accepted as Resolved — malformed-symbol \
         fall-through invariant broken."
    );
    assert_eq!(
        refined.path, syntactic.path,
        "FINDING: whitespace-only symbol changed the path — fall-through must preserve syntactic."
    );
}

// ATTACK 2b: newline-only symbol — trim() strips newlines, so this must fall through.
#[test]
fn atk_survey_newline_only_scip_symbol_falls_through() {
    let syntactic = syntactic_fq_path("crate", &["m".to_string()], "f");
    let refined = refine_with_scip(syntactic, Some("\n\n\n"));
    assert_eq!(
        refined.tier,
        ResolutionTier::Syntactic,
        "FINDING: newline-only SCIP symbol produced Resolved — trim() should catch this."
    );
}

// ATTACK 2c: None symbol — must return syntactic unchanged.
#[test]
fn atk_survey_none_scip_symbol_returns_syntactic() {
    let syntactic = syntactic_fq_path("crate", &["m".to_string()], "f");
    let refined = refine_with_scip(syntactic.clone(), None);
    assert_eq!(
        refined, syntactic,
        "None symbol must return syntactic unchanged."
    );
    assert_eq!(refined.tier, ResolutionTier::Syntactic);
}

// ATTACK 2d: well-formed symbol — MUST raise to Resolved.
#[test]
fn atk_survey_well_formed_scip_symbol_becomes_resolved() {
    let syntactic = syntactic_fq_path("crate", &["m".to_string()], "f");
    let refined = refine_with_scip(syntactic, Some("crate . m / f#"));
    assert_eq!(
        refined.tier,
        ResolutionTier::Resolved,
        "A non-empty, non-whitespace SCIP symbol must produce Resolved tier."
    );
}

// PROBE 2e: null-byte embedded in a SCIP symbol — the frame-epoch gate is conservative (!trim().is_empty()).
// A null-byte symbol is NOT whitespace, so it WILL pass the current gate and become Resolved.
// This is an EXPECTED gap (full SCIP grammar validation is engine-epoch, named-deferred in ADR-070 §5.2).
// We document the behavior rather than fail.
#[test]
fn probe_survey_null_byte_scip_symbol_conservative_gate_behavior() {
    let syntactic = syntactic_fq_path("crate", &["mod".to_string()], "item");
    let null_symbol = "foo\x00bar"; // non-empty, non-whitespace → current gate accepts it
    let refined = refine_with_scip(syntactic, Some(null_symbol));
    // The current frame-epoch gate only checks !trim().is_empty().
    // A null-byte symbol passes that gate and becomes Resolved.
    // This is acceptable at frame-epoch (SCIP grammar validation is engine-epoch per ADR-070 §5.2).
    // DOCUMENT the behavior so it's visible:
    assert_eq!(
        refined.tier,
        ResolutionTier::Resolved,
        "PROBE (expected at frame-epoch): null-byte symbol IS accepted as Resolved by the \
         conservative frame-epoch gate. This is named-deferred to engine-epoch full SCIP grammar \
         validation (ADR-070 §5.2). If this assert FAILS, the gate has been tightened \
         beyond the frame-epoch spec — verify that is intentional."
    );
}

// ── ATTACK 3: §4.3 come-apart — load-bearing attrs that MUST change identity ─────────────────────

// ATTACK 3a: `#[aura]` is load-bearing (grade claim) — forging it MUST change identity.
#[test]
fn atk_survey_forging_aura_changes_identity() {
    let plain: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let forged: syn::Item = syn::parse_str(r#"#[aura("high")] fn f() {}"#).unwrap();
    assert_ne!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&forged),
        "FINDING: `#[aura]` is LOAD_BEARING but forging it did NOT change identity — \
         a grade-claim forgery is invisible to the signing digest."
    );
}

// ATTACK 3b: `#[red_flag]` is load-bearing (grade claim) — must change identity.
#[test]
fn atk_survey_forging_red_flag_changes_identity() {
    let plain: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let forged: syn::Item = syn::parse_str("#[red_flag] fn f() {}").unwrap();
    assert_ne!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&forged),
        "FINDING: `#[red_flag]` is LOAD_BEARING but forging it left identity unchanged."
    );
}

// ATTACK 3c: `#[anergy]` is a defense grant (load-bearing) — must change identity.
#[test]
fn atk_survey_forging_anergy_changes_identity() {
    let plain: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let forged: syn::Item = syn::parse_str("#[anergy] fn f() {}").unwrap();
    assert_ne!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&forged),
        "FINDING: `#[anergy]` is LOAD_BEARING (defense grant) but forging it left identity unchanged."
    );
}

// ATTACK 3d: `#[quarantine]` is load-bearing (isolation decision) — forging it must change identity.
#[test]
fn atk_survey_forging_quarantine_changes_identity() {
    let plain: syn::Item = syn::parse_str("struct S;").unwrap();
    let forged: syn::Item = syn::parse_str("#[quarantine] struct S;").unwrap();
    assert_ne!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&forged),
        "FINDING: `#[quarantine]` is LOAD_BEARING (isolation) but forging it left identity unchanged."
    );
}

// ATTACK 3e: `#[mucosal_tolerant]` is load-bearing (trust grant) — must change identity.
#[test]
fn atk_survey_forging_mucosal_tolerant_changes_identity() {
    let plain: syn::Item = syn::parse_str("fn trusted() {}").unwrap();
    let forged: syn::Item = syn::parse_str("#[mucosal_tolerant] fn trusted() {}").unwrap();
    assert_ne!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&forged),
        "FINDING: `#[mucosal_tolerant]` is LOAD_BEARING but forging it left identity unchanged."
    );
}

// ATTACK 3f: `#[triage_commit]` is load-bearing (rollback-authority) — must change identity.
#[test]
fn atk_survey_forging_triage_commit_changes_identity() {
    let plain: syn::Item = syn::parse_str("fn fix() {}").unwrap();
    let forged: syn::Item = syn::parse_str("#[triage_commit] fn fix() {}").unwrap();
    assert_ne!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&forged),
        "FINDING: `#[triage_commit]` is LOAD_BEARING but forging it left identity unchanged."
    );
}

// NC 3g: `#[immune]` is PURE (wrapper/container) — toggling it MUST NOT change identity.
#[test]
fn nc_survey_immune_wrapper_is_pure_does_not_change_identity() {
    let plain: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let wrapped: syn::Item = syn::parse_str("#[immune] fn f() {}").unwrap();
    assert_eq!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&wrapped),
        "FINDING: `#[immune]` is PURE (wrapper) but changed identity — over-narrow strip causes \
         identity churn on pure annotation edits."
    );
}

// NC 3h: `#[itch]` is pure (recurrent pattern doc) — must NOT change identity.
#[test]
fn nc_survey_itch_is_pure_does_not_change_identity() {
    let plain: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let annotated: syn::Item = syn::parse_str("#[itch] fn f() {}").unwrap();
    assert_eq!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&annotated),
        "FINDING: `#[itch]` is PURE (documentary) but changed identity — over-narrow strip."
    );
}

// NC 3i: `#[polyclonal]` is pure (witness-classification label) — must NOT change identity.
#[test]
fn nc_survey_polyclonal_is_pure_does_not_change_identity() {
    let plain: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let annotated: syn::Item = syn::parse_str("#[polyclonal] fn f() {}").unwrap();
    assert_eq!(
        IdentityDigest::of_item(&plain),
        IdentityDigest::of_item(&annotated),
        "FINDING: `#[polyclonal]` is PURE but changed identity — misclassified as load-bearing."
    );
}

// ATTACK 3j: multiple stacked load-bearing attrs — each combination must produce a DISTINCT digest.
// Attack: are the digests actually distinct for each combination, or does stacking produce collisions?
#[test]
fn atk_survey_stacked_load_bearing_attrs_produce_distinct_digests() {
    let base: syn::Item = syn::parse_str("fn f() {}").unwrap();
    let one: syn::Item = syn::parse_str("#[presents(\"x\")] fn f() {}").unwrap();
    let two: syn::Item =
        syn::parse_str("#[presents(\"x\")] #[defended_by(\"y\")] fn f() {}").unwrap();
    let d_base = IdentityDigest::of_item(&base);
    let d_one = IdentityDigest::of_item(&one);
    let d_two = IdentityDigest::of_item(&two);
    assert_ne!(d_base, d_one, "Adding #[presents] must change identity.");
    assert_ne!(
        d_one, d_two,
        "Adding #[defended_by] on top must further change identity."
    );
    assert_ne!(
        d_base, d_two,
        "Two load-bearing attrs must differ from zero."
    );
}

// ATTACK 3k: BLAKE3 determinism — same input always → same digest (no randomness, no time-dependency).
#[test]
fn atk_survey_identity_digest_is_deterministic() {
    let item: syn::Item = syn::parse_str("pub fn compute(x: u32) -> u32 { x * 2 }").unwrap();
    let d1 = IdentityDigest::of_item(&item);
    let d2 = IdentityDigest::of_item(&item);
    assert_eq!(
        d1, d2,
        "IdentityDigest must be deterministic — same item, same digest every time."
    );
}

// ATTACK 3l: empty token stream — no panic, deterministic.
#[test]
fn atk_survey_identity_digest_of_tokens_empty_is_deterministic() {
    let d1 = IdentityDigest::of_tokens(&[]);
    let d2 = IdentityDigest::of_tokens(&[]);
    assert_eq!(d1, d2, "BLAKE3 of empty bytes must be deterministic.");
}

// ATTACK 3m: huge token stream (1MB) — no panic, deterministic.
#[test]
fn atk_survey_identity_digest_of_tokens_huge_no_panic() {
    let huge = vec![0xffu8; 1_000_000];
    let d1 = IdentityDigest::of_tokens(&huge);
    let d2 = IdentityDigest::of_tokens(&huge);
    assert_eq!(
        d1, d2,
        "BLAKE3 of 1MB input must be deterministic and not panic."
    );
}

// ── ATTACK 4: fidelity / tool-independence ────────────────────────────────────────────────────────

// ATTACK 4a: overflow — source_mtime = u64::MAX with saturating_add.
// saturating_add(1) = u64::MAX >= u64::MAX → stale → demotion. Must not panic.
#[test]
fn atk_survey_fidelity_u64_max_source_mtime_does_not_panic_and_is_stale() {
    let tier = FidelityWitness::check(u64::MAX, u64::MAX, ResolutionTier::Resolved);
    assert_eq!(
        tier,
        ResolutionTier::Syntactic,
        "u64::MAX source + u64::MAX index (same-second tie) must be stale → demoted to Syntactic."
    );
}

// ATTACK 4b: source_mtime = u64::MAX, index_mtime = 0 — both are extremely stale.
// saturating_add: u64::MAX + 1 = u64::MAX >= 0 → stale.
#[test]
fn atk_survey_fidelity_max_source_zero_index_is_stale() {
    let tier = FidelityWitness::check(u64::MAX, 0, ResolutionTier::Resolved);
    assert_eq!(
        tier,
        ResolutionTier::Syntactic,
        "u64::MAX source, 0 index: must be stale (source far in future relative to index)."
    );
}

// ATTACK 4c: both zero — same-second tie → stale per the guard band.
#[test]
fn atk_survey_fidelity_both_zero_is_stale() {
    let tier = FidelityWitness::check(0, 0, ResolutionTier::Resolved);
    assert_eq!(
        tier,
        ResolutionTier::Syntactic,
        "source=0, index=0 (same second) must be stale — guard band says same-second = potentially stale."
    );
}

// ATTACK 4d: index vastly in the future — must be fresh.
#[test]
fn atk_survey_fidelity_future_index_is_fresh() {
    let tier = FidelityWitness::check(1_000, 1_000_000, ResolutionTier::Resolved);
    assert_eq!(
        tier,
        ResolutionTier::Resolved,
        "index far in the future (>> source + 1) must be fresh → pass Resolved through."
    );
}

// ATTACK 4e: T3Mir is demoted when stale (higher tier is NOT a freshness exemption).
#[test]
fn atk_survey_fidelity_t3mir_demoted_when_stale() {
    let tier = FidelityWitness::check(100, 100, ResolutionTier::T3Mir);
    assert_eq!(
        tier,
        ResolutionTier::Syntactic,
        "FINDING: T3Mir was NOT demoted when stale — a higher-tier claim survived a stale index. \
         T3Mir is not exempt from the fidelity guard."
    );
}

// ATTACK 4f: Syntactic passes through unchanged (already the floor; stale or fresh doesn't matter).
#[test]
fn atk_survey_fidelity_syntactic_passthrough_regardless_of_staleness() {
    let stale = FidelityWitness::check(100, 50, ResolutionTier::Syntactic);
    let fresh = FidelityWitness::check(50, 100, ResolutionTier::Syntactic);
    assert_eq!(
        stale,
        ResolutionTier::Syntactic,
        "Syntactic must pass through when stale."
    );
    assert_eq!(
        fresh,
        ResolutionTier::Syntactic,
        "Syntactic must pass through when fresh."
    );
}

// ATTACK 4g: exactly one second apart (source + 1 == index) — this is the boundary of the guard band.
// source=100, index=101: 100 + 1 = 101, NOT >= 101 is false — wait: 101 >= 101 → stale!
// The guard is saturating_add(COARSE_FS_MARGIN=1) >= index_mtime.
// 100 + 1 = 101 >= 101 → TRUE → STALE. The guard band means even exactly-1-second offset = stale.
#[test]
fn atk_survey_fidelity_exactly_one_second_gap_is_stale() {
    // source=100, index=101: 100+1=101 >= 101 → stale
    let tier = FidelityWitness::check(100, 101, ResolutionTier::Resolved);
    assert_eq!(
        tier,
        ResolutionTier::Syntactic,
        "FINDING: source=100, index=101 (1-second gap) was FRESH but should be STALE. \
         The guard band (source + 1 >= index) means even a 1-second margin = stale."
    );
}

// ATTACK 4h: two seconds apart (source + 2 == index) — the FIRST truly fresh gap.
// source=100, index=102: 100+1=101, 101 >= 102 → FALSE → FRESH.
#[test]
fn atk_survey_fidelity_two_second_gap_is_fresh() {
    let tier = FidelityWitness::check(100, 102, ResolutionTier::Resolved);
    assert_eq!(
        tier,
        ResolutionTier::Resolved,
        "source=100, index=102 (2-second gap): 100+1=101 < 102 → fresh. Resolved must pass through."
    );
}

// ── ATTACK 5: corroborate — adversarial tier combinations ────────────────────────────────────────

// ATTACK 5a: Syntactic + Syntactic → None (cannot corroborate up from the floor).
#[test]
fn atk_survey_corroborate_syntactic_plus_syntactic_is_none() {
    assert!(
        corroborate_presents(ResolutionTier::Syntactic, ResolutionTier::Syntactic).is_none(),
        "FINDING: Syntactic + Syntactic produced a PresentsVerdict — \
         two floor-tier sources should NEVER earn presents-grade."
    );
}

// ATTACK 5b: Syntactic + Resolved → None (lower-never-corroborates-up).
#[test]
fn atk_survey_corroborate_syntactic_plus_resolved_is_none() {
    assert!(
        corroborate_presents(ResolutionTier::Syntactic, ResolutionTier::Resolved).is_none(),
        "FINDING: Syntactic + Resolved produced PresentsVerdict — lower-never-up invariant broken."
    );
    assert!(
        corroborate_presents(ResolutionTier::Resolved, ResolutionTier::Syntactic).is_none(),
        "FINDING: Resolved + Syntactic produced PresentsVerdict — order-asymmetric."
    );
}

// ATTACK 5c: Syntactic + T3Mir → None (T3Mir cannot lift a Syntactic partner).
#[test]
fn atk_survey_corroborate_syntactic_plus_t3mir_is_none() {
    assert!(
        corroborate_presents(ResolutionTier::Syntactic, ResolutionTier::T3Mir).is_none(),
        "FINDING: Syntactic + T3Mir produced PresentsVerdict — even the highest tier cannot lift \
         a Syntactic (demoted-stale or form-only) source."
    );
    assert!(
        corroborate_presents(ResolutionTier::T3Mir, ResolutionTier::Syntactic).is_none(),
        "FINDING: T3Mir + Syntactic produced PresentsVerdict — order-asymmetric with T3Mir."
    );
}

// NC 5d: Resolved + Resolved → Some (the NC — the normal presents-grade path).
#[test]
fn nc_survey_corroborate_resolved_plus_resolved_produces_presents() {
    let v = corroborate_presents(ResolutionTier::Resolved, ResolutionTier::Resolved);
    assert!(
        v.is_some(),
        "Resolved + Resolved must produce a PresentsVerdict (the NC path)."
    );
    assert_eq!(v.unwrap().earned_at(), ResolutionTier::Resolved);
}

// NC 5e: Resolved + T3Mir → Some, earned T3Mir (higher of the two).
#[test]
fn nc_survey_corroborate_resolved_plus_t3mir_reaches_t3mir() {
    let v = corroborate_presents(ResolutionTier::Resolved, ResolutionTier::T3Mir);
    assert!(
        v.is_some(),
        "Resolved + T3Mir must produce PresentsVerdict."
    );
    assert_eq!(
        v.unwrap().earned_at(),
        ResolutionTier::T3Mir,
        "FINDING: corroborate(Resolved, T3Mir) earned wrong tier — expected T3Mir (the max)."
    );
}

// NC 5f: T3Mir + T3Mir → Some, earned T3Mir.
#[test]
fn nc_survey_corroborate_t3mir_plus_t3mir_reaches_t3mir() {
    let v = corroborate_presents(ResolutionTier::T3Mir, ResolutionTier::T3Mir);
    assert!(v.is_some(), "T3Mir + T3Mir must produce PresentsVerdict.");
    assert_eq!(v.unwrap().earned_at(), ResolutionTier::T3Mir);
}

// ── ATTACK 6: Revision (last_changed) — monotone join adversarial ────────────────────────────────

// ATTACK 6a: merge of two equal values is idempotent.
#[test]
fn atk_survey_revision_merge_idempotent() {
    let r = Revision(500);
    assert_eq!(r.merge(r), r, "merge(r, r) must equal r — idempotent.");
}

// ATTACK 6b: merge is commutative.
#[test]
fn atk_survey_revision_merge_commutative() {
    let a = Revision(100);
    let b = Revision(200);
    assert_eq!(a.merge(b), b.merge(a), "merge must be commutative.");
}

// ATTACK 6c: a decreasing sequence of merges holds the maximum.
#[test]
fn atk_survey_revision_merge_decreasing_sequence_holds_max() {
    let mut r = Revision(1000);
    for t in (0..1000u64).rev() {
        r = r.merge(Revision(t));
    }
    assert_eq!(
        r,
        Revision(1000),
        "FINDING: a decreasing merge sequence regressed below the initial maximum."
    );
}

// ATTACK 6d: interleaved large/small values — result is the global max.
#[test]
fn atk_survey_revision_merge_interleaved_is_global_max() {
    let values = [10u64, 9999, 1, 42, 9998, 2, 9999, 0, u64::MAX];
    let r = values
        .iter()
        .fold(Revision(0), |acc, &v| acc.merge(Revision(v)));
    assert_eq!(
        r,
        Revision(u64::MAX),
        "FINDING: interleaved merge did not reach u64::MAX."
    );
}

// ATTACK 6e: u64::MAX merge with 0 — max wins, no overflow.
#[test]
fn atk_survey_revision_merge_u64_max_with_zero() {
    let r = Revision(u64::MAX).merge(Revision(0));
    assert_eq!(r, Revision(u64::MAX));
}

// ── ATTACK 7: tier ordering invariants ───────────────────────────────────────────────────────────

// ATTACK 7a: Syntactic < Resolved < T3Mir (the ladder must be ordered).
#[test]
fn atk_survey_tier_ordering_is_correct() {
    assert!(
        ResolutionTier::Syntactic < ResolutionTier::Resolved,
        "FINDING: Syntactic is not less than Resolved — the tier ordering is wrong."
    );
    assert!(
        ResolutionTier::Resolved < ResolutionTier::T3Mir,
        "FINDING: Resolved is not less than T3Mir — the tier ordering is wrong."
    );
    assert!(
        ResolutionTier::Syntactic < ResolutionTier::T3Mir,
        "FINDING: Syntactic is not less than T3Mir — transitivity of ordering broken."
    );
}

// ATTACK 7b: detection_ceiling for each tier.
#[test]
fn atk_survey_detection_ceiling_values() {
    use antigen_stroma::read::DetectionGrade;
    assert_eq!(
        ResolutionTier::Syntactic.detection_ceiling(),
        DetectionGrade::Dread,
        "FINDING: Syntactic detection ceiling is not Dread."
    );
    assert_eq!(
        ResolutionTier::Resolved.detection_ceiling(),
        DetectionGrade::Presents,
        "FINDING: Resolved detection ceiling is not Presents."
    );
    assert_eq!(
        ResolutionTier::T3Mir.detection_ceiling(),
        DetectionGrade::Presents,
        "FINDING: T3Mir detection ceiling is not Presents."
    );
}

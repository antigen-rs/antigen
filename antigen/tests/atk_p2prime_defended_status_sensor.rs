//! ATK-P2' — the DEFENDED-STATUS sensor pins tier>None per CLASS, not a homonym (BORN-RED).
//!
//! **STATUS: BORN-RED.** The per-class defended-status roll-up
//! (the-discriminator-is-blind-for-silent-classes, route-book row 3') is
//! being built in `antigen/src/audit/` now. This is the INDEPENDENT criterion-check
//! (parallel to core's test-first self-close — the closest thing to no-self-witness
//! inside the build). It pins the REAL criterion and ATTACKS the homonym.
//!
//! # The criterion (do-now RESOLUTION roll-up only)
//!
//! WELL-DEFENDED ≠ OBSOLETE requires a **per-CLASS (`antigen_type`) aggregation of
//! "does this class carry ANY live witness?"** — i.e. ∃ an `ImmunityAudit` for the
//! class with `witness_tier > WitnessTier::None` (Reachability or higher = the witness
//! identifier resolves / "resolves-exists"). Built on the existing
//! `ImmunityAudit.witness_tier` (`audit/types.rs:698`) + `Immunity.antigen_type`,
//! rolled up GROUP-BY `antigen_type`.
//!
//! Scope: the do-now is the RESOLUTION roll-up (tier>None). The WOULD-CATCH /
//! exercised-coverage axis (mutation-testing, SEAM-B) is do-LATER — NOT pinned here.
//!
//! # The homonym this ATK ATTACKS (the whole point — the named-not-found silent failure)
//!
//! If the sensor uses a HOMONYM instead of the real aggregation, WELL-DEFENDED
//! collapses into OBSOLETE and antigen FORGETS ITS OWN WORKING IMMUNITIES (the worst
//! CURATE failure). The two homonyms, each killed by a test below:
//!   - "the class has SOME audit row" (count rows, ignore tier) — a class with rows
//!     that are ALL `tier==None` (broken/missing/unresolved witnesses) would be
//!     wrongly called defended. KILLED by `all_none_tier_rows_is_not_defended`.
//!   - "the scanner enumerated this class" (presence ⇒ defended) — enumeration is not
//!     defense. KILLED by the same: a class present in the report with no tier>None row
//!     is NOT defended.
//!
//! A WEAK test ("the class appears in the defended map", "the roll-up is non-empty")
//! passes for BOTH homonyms — it goes green-by-luck while the criterion stays
//! undefended. The STRONG tests below assert the tier-discrimination directly.
//!
//! ----------------------------------------------------------------------------
//! STATUS: P2' LANDED — the born-red gate is DROPPED; these asserts now compile
//! against the real `AuditReport::is_class_defended` and are GREEN (the live
//! independent criterion-check). The implementation added `is_class_defended` (the
//! per-class `any(tier > None)` query) to satisfy this ATK's contract.
//! ----------------------------------------------------------------------------

// TWO GOTCHAS a born-red here must know (both real, both verified):
// (1) VISIBILITY: the audit/scan types are re-exported at the MODULE root
//     (audit/mod.rs:31, scan/mod.rs:1245); the `types` submodule is PRIVATE. Import
//     from `antigen::audit::` / `antigen::scan::`, NOT `::types::` (else E0603).
// (2) DUAL WitnessTier: there are TWO `WitnessTier` enums — `antigen::audit::WitnessTier`
//     (audit/types.rs:133) and `antigen_attestation::WitnessTier` (tier.rs:47), kept in
//     lock-step but DISTINCT types. `ImmunityAudit.witness_tier` uses the AUDIT one —
//     use `antigen::audit::WitnessTier`, not the attestation one (else E0308 type-mismatch).
use antigen::audit::{AuditHint, AuditReport, ImmunityAudit, WitnessStatus, WitnessTier};
use antigen::scan::{Immunity, ItemTarget};
use antigen_attestation::EvidenceKind;

// PROPOSED SURFACE (core's roll-up — confirm the real name when it lands; the ASSERT
// is the contract, the spelling negotiable). The roll-up is BUILT ON the existing
// per-row `ImmunityAudit::has_witness()` (audit/types.rs:760 — `tier != None`, exactly
// the do-now resolution criterion), grouped by `antigen_type`:
//   `report.is_class_defended(antigen_type: &str) -> bool`  ==  ∃ row: row.immunity
//        .antigen_type == antigen_type && row.has_witness()
// NOTE TO CORE: fixture-construction below builds ImmunityAudit/Immunity by hand
// (neither derives Default). Confirm the field set against the real structs and swap in
// a test-helper if you have one — the ASSERTS are the contract, not my constructors.
//
// CRITICAL: the roll-up must use `has_witness()` (tier>None — the do-now RESOLUTION
// criterion), NOT `is_well_formed()` (audit/types.rs:771 — Execution-tier+). Those
// are two existing methods and using the wrong one is a homonym (see the
// reachability_tier test below).

/// Build a minimal `Immunity` for a class — only `antigen_type` is read by the roll-up.
fn immunity(antigen_type: &str) -> Immunity {
    Immunity {
        antigen_type: antigen_type.into(),
        witness: String::new(),
        requires_predicate: None,
        file: std::path::PathBuf::from("test.rs"),
        line: 1,
        item_kind: "fn".into(),
        item_target: ItemTarget::Struct("Defended".into()),
        canonical_path: None,
        structural_fingerprint: String::new(),
    }
}

/// Build an `ImmunityAudit` row for `antigen_type` at a given witness tier. All fields
/// explicit (`ImmunityAudit` does not derive `Default`); only `immunity.antigen_type` and
/// `witness_tier` are load-bearing for the roll-up.
fn audit_row(antigen_type: &str, tier: WitnessTier) -> ImmunityAudit {
    ImmunityAudit {
        immunity: immunity(antigen_type),
        witness_status: WitnessStatus::NotFound {
            reason: "fixture".into(),
        },
        witness_tier: tier,
        audit_hint: AuditHint::NoneApplicable,
        evidence_kind: EvidenceKind::None,
        signature_strength: None,
        compound_evidence: false,
        evaluated_predicate: None,
        code_witness_sidecar_ignored: false,
        leaf_outcomes: Vec::new(),
    }
}

fn report_with(audits: Vec<ImmunityAudit>) -> AuditReport {
    AuditReport {
        audits,
        ..Default::default()
    }
}

/// POSITIVE: a class with at least one `tier>None` (Reachability) row IS well-defended.
#[test]
fn class_with_a_resolving_witness_is_defended() {
    let report = report_with(vec![audit_row(
        "ParallelStateTrackersDiverge",
        WitnessTier::Reachability,
    )]);
    assert!(
        report.is_class_defended("ParallelStateTrackersDiverge"),
        "a class with a witness at Reachability (tier>None — the identifier resolves) \
         is WELL-DEFENDED; the resolution roll-up must report it defended."
    );
}

/// HOMONYM-KILLER #1 (the load-bearing test): a class whose audit rows are ALL at
/// `tier==None` (broken/missing/unresolved witnesses) is NOT defended. A sensor that
/// counts ROWS instead of checking TIER — or that treats enumeration as defense —
/// FAILS this. This is the test that separates the real criterion from the homonym.
#[test]
fn all_none_tier_rows_is_not_defended() {
    // Two audit rows for the class, BOTH tier==None (the witnesses don't resolve).
    let report = report_with(vec![
        audit_row("SilentlyBrokenClass", WitnessTier::None),
        audit_row("SilentlyBrokenClass", WitnessTier::None),
    ]);
    assert!(
        !report.is_class_defended("SilentlyBrokenClass"),
        "a class whose audit rows are ALL tier==None carries NO live witness — it must \
         be NOT-defended (it collapses into OBSOLETE, correctly). A sensor reporting it \
         DEFENDED used the homonym 'has SOME audit row' / 'was enumerated' instead of \
         'carries a tier>None witness' — and antigen would FORGET a class it cannot \
         actually defend. THIS is the homonym the roll-up must not use."
    );
}

/// DISCRIMINATION (the WELL-DEFENDED ≠ OBSOLETE split, end to end): given two silent
/// classes — one with a live witness, one with only None-tier rows — the sensor must
/// DISTINGUISH them. If it calls both defended (or both not), WELL-DEFENDED has
/// collapsed into OBSOLETE and the moral-center curation is blind.
#[test]
fn well_defended_is_distinguished_from_obsolete() {
    let report = report_with(vec![
        audit_row("LiveImmunity", WitnessTier::Execution), // a real, exercised witness
        audit_row("DeadImmunity", WitnessTier::None),      // no live witness
    ]);
    assert!(
        report.is_class_defended("LiveImmunity"),
        "the class with a live (Execution-tier) witness is WELL-DEFENDED."
    );
    assert!(
        !report.is_class_defended("DeadImmunity"),
        "the class with only None-tier rows is NOT defended (OBSOLETE-eligible). The \
         sensor MUST split these — calling both the same collapses WELL-DEFENDED into \
         OBSOLETE and antigen forgets its own working immunities."
    );
}

/// MIXED: a class with SOME None rows and ONE tier>None row IS defended — ANY live
/// witness defends the class (the aggregation is `any`, not `all`). Guards the dual
/// error (an over-strict `all(tier>None)` would wrongly forget a class defended by one
/// live witness among several unresolved ones).
#[test]
fn any_one_resolving_witness_defends_the_class() {
    let report = report_with(vec![
        audit_row("MixedClass", WitnessTier::None),
        audit_row("MixedClass", WitnessTier::Reachability), // one resolves
        audit_row("MixedClass", WitnessTier::None),
    ]);
    assert!(
        report.is_class_defended("MixedClass"),
        "ANY tier>None witness defends the class (the roll-up is `any(tier>None)`, not \
         `all`). One live witness among unresolved ones still means the class IS \
         defended — an over-strict `all` would forget it."
    );
}

/// HOMONYM-KILLER #2 (two existing methods, easy to confuse): the do-now RESOLUTION
/// sensor uses `has_witness()` (tier>None), NOT `is_well_formed()` (Execution-tier+,
/// audit/types.rs:771). A class defended ONLY at `Reachability` tier (the witness
/// IDENTIFIER resolves but isn't execution-verified) IS defended-for-the-resolution-
/// sensor — it carries a live witness, so it must NOT be forgotten as OBSOLETE. A
/// roll-up that gates on `is_well_formed()` (Execution+) would wrongly forget every
/// Reachability-only class. (The exercised-coverage / Execution split is do-LATER,
/// SEAM-B — the do-now sensor must not pre-empt it by over-gating.)
#[test]
fn reachability_tier_class_is_defended_for_the_resolution_sensor() {
    let report = report_with(vec![audit_row(
        "ReachabilityOnlyClass",
        WitnessTier::Reachability,
    )]);
    assert!(
        report.is_class_defended("ReachabilityOnlyClass"),
        "a Reachability-tier class (witness IDENTIFIER resolves) carries a live witness \
         — the do-now RESOLUTION sensor must report it DEFENDED (has_witness(), tier>None). \
         If this is false, the roll-up used `is_well_formed()` (Execution-tier+) — the \
         WRONG existing method — and would forget every Reachability-only class as OBSOLETE. \
         The exercised-coverage axis (Execution+) is do-LATER (SEAM-B), not the do-now gate."
    );
}

/// ABSENT: a class with NO audit rows at all is NOT defended (no witness ⇒ not
/// defended) — and must not panic on the empty group.
#[test]
fn class_absent_from_audits_is_not_defended() {
    let report = report_with(vec![]);
    assert!(
        !report.is_class_defended("NeverAudited"),
        "a class with no audit rows carries no witness — not defended; the empty-group \
         case must be a clean false, never a panic."
    );
}

// ===========================================================================
// HARDENING (degenerate-input — the no-self-witness check
// inside the build). These read the RICH roll-up (audit_defended_status) so the
// aggregation FIELDS (max_witness_tier / site_count / resolving_site_count) are
// pinned, not just the boolean.
// ===========================================================================

use antigen::audit::audit_defended_status;

/// DEGENERATE: an EMPTY report (no immunities at all) rolls up to an EMPTY `by_class` —
/// no panic, no phantom class, no false-defend. The zero case must be clean.
#[test]
fn empty_report_rolls_up_to_empty_by_class() {
    let report = report_with(vec![]);
    let rollup = audit_defended_status(&report);
    assert!(
        rollup.by_class.is_empty(),
        "an empty AuditReport must roll up to an EMPTY by_class — zero immunities means \
         zero classes, no panic, no phantom-defended class fabricated from nothing."
    );
    assert!(
        rollup.undefended_classes().is_empty(),
        "and undefended_classes() over an empty report is empty (no class to forget)."
    );
}

/// DEGENERATE: MIXED sites in ONE class — some resolving (tier>None), some bare
/// (None). The roll-up must take `max` (any live witness ⇒ defended) AND count
/// correctly: `site_count` = all sites, `resolving_site_count` = only the live ones,
/// `max_witness_tier` = the strongest. A roll-up that took `min` or `last` would
/// wrongly forget a class with one live + several bare witnesses.
#[test]
fn mixed_sites_in_one_class_max_rolls_up_and_counts_split() {
    let report = report_with(vec![
        audit_row("MixedSiteClass", WitnessTier::None), // bare
        audit_row("MixedSiteClass", WitnessTier::Execution), // live, strongest
        audit_row("MixedSiteClass", WitnessTier::None), // bare
        audit_row("MixedSiteClass", WitnessTier::Reachability), // live, weaker
    ]);
    let rollup = audit_defended_status(&report);
    let status = rollup
        .by_class
        .get("MixedSiteClass")
        .expect("the class must be present in the roll-up");

    assert_eq!(
        status.max_witness_tier,
        WitnessTier::Execution,
        "max_witness_tier must be the STRONGEST tier across the class's sites (Execution \
         here), not the last/first/min — a `min` or `last` roll-up would understate the \
         defense and wrongly forget a class that IS defended."
    );
    assert_eq!(
        status.site_count, 4,
        "site_count counts ALL sites (4) for the class."
    );
    assert_eq!(
        status.resolving_site_count, 2,
        "resolving_site_count counts ONLY the tier>None sites (2 of 4) — the bare \
         witnesses don't resolve."
    );
    assert!(
        status.is_defended_on_resolution(),
        "ANY live witness defends the class (max>None) — the class is defended despite \
         two bare sites."
    );
}

/// DEGENERATE: the SAME class declared at TWO different FILES (duplicate `antigen_type`
/// across sites). The group-by key is `antigen_type` ALONE (file is not in the key),
/// so the roll-up MUST aggregate them as ONE class — not double-count into two entries
/// and not split. A split would fracture a class's defended-status across files and
/// the discriminator would see two half-views of one class.
#[test]
fn duplicate_antigen_type_across_files_aggregates_as_one_class() {
    // Two sites for the SAME class, but in DIFFERENT files (audit_row fixes file to
    // test.rs; build these by hand to vary the file).
    let mut row_a = audit_row("CrossFileClass", WitnessTier::None);
    row_a.immunity.file = std::path::PathBuf::from("module_a.rs");
    let mut row_b = audit_row("CrossFileClass", WitnessTier::Reachability);
    row_b.immunity.file = std::path::PathBuf::from("module_b.rs");

    let report = report_with(vec![row_a, row_b]);
    let rollup = audit_defended_status(&report);

    assert_eq!(
        rollup.by_class.len(),
        1,
        "the same antigen_type declared in TWO files must aggregate into ONE class entry \
         — the group-by key is antigen_type alone (file is NOT in the key). Two entries \
         would mean the roll-up split a class by file, fracturing its defended-status."
    );
    let status = &rollup.by_class["CrossFileClass"];
    assert_eq!(
        status.site_count, 2,
        "both file-sites count toward the one class (2)."
    );
    assert!(
        status.is_defended_on_resolution(),
        "the cross-file class is defended (the module_b.rs site resolves at Reachability) \
         — aggregating both files is what surfaces the live witness."
    );
}

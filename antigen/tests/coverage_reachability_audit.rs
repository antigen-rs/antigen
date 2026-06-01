//! The coverage / reachability audit — the ignorance frontier as a per-site
//! verdict (`infra/reachability-primitive-cross-tier`, v03-vision-buildout).
//!
//! Immunological ignorance is the 4th canonical peripheral-tolerance mechanism
//! (Khan & Ghazanfar 2018): a functional self-antigen the immune system never
//! *encounters*. Software cognate: a real `#[presents]` site the scanner never
//! reaches. `audit_coverage` projects the ignorance frontier
//! ([`ScanCoverage`]'s enumerated-but-unscanned members) into per-site
//! `UnreachedSite` verdicts.
//!
//! ## What these tests pin
//!
//! 1. **The cardinality is exactly three** — `UnreachedCause` partitions the
//!    scanner pipeline `{enumerate → parse → match}` at its three pre-evaluation
//!    drop-stages. The enum must carry exactly Barrier / `SubThreshold` / Cryptic.
//! 2. **Each cause routes a distinct remedy** — the verdict is never a bare
//!    reached/not bool (the cardinality-collapse the whole arc fights); the
//!    three remedy strings must differ.
//! 3. **Barrier verdicts fire live** off `ScanCoverage.unscanned_members()` —
//!    each enumerated-but-unscanned member yields one Barrier `UnreachedSite`,
//!    deduped (the frontier is a set).
//! 4. **No `scan_coverage` ⇒ empty report** — a flat scan has no member concept,
//!    so it cannot claim completeness; tier-honest absence, not a false-green.
//! 5. **The complete-frontier happy path** reports no unreached sites.
//! 6. **`coverage_was_applicable` is the three-state discriminator** — the
//!    `(is_complete, coverage_was_applicable)` pair tells verified-complete
//!    (`true,true`) from incomplete (`false,true`) from not-applicable
//!    (`true,false`), so a 2-valued `is_complete` no longer collapses the
//!    first and third (adversarial 2026-06-01 type-discipline gap).

use antigen::audit::{audit_coverage, UnreachedCause};
use antigen::scan::{ScanCoverage, ScanReport};

/// A `ScanReport` carrying a `ScanCoverage` with the given member sets and
/// nothing else (all other fields default-empty) — the minimal substrate the
/// coverage audit reads.
fn report_with_coverage(enumerated: &[&str], scanned: &[&str]) -> ScanReport {
    ScanReport {
        scan_coverage: Some(ScanCoverage {
            enumerated_members: enumerated.iter().map(ToString::to_string).collect(),
            scanned_members: scanned.iter().map(ToString::to_string).collect(),
        }),
        ..Default::default()
    }
}

#[test]
fn unreached_cause_cardinality_is_exactly_three() {
    // The three causes are a principled partition of the scanner pipeline's
    // three pre-evaluation drop-stages — not enumerated-by-luck. This test is a
    // contract: a fourth cause (or a collapse to fewer) would mean the partition
    // changed and the remedy-routing must be re-examined.
    //
    // Exhaustive match — adding/removing a variant forces this test to be
    // updated, surfacing any cardinality change at review time.
    let all = [
        UnreachedCause::Barrier,
        UnreachedCause::SubThreshold,
        UnreachedCause::Cryptic,
    ];
    for cause in all {
        match cause {
            UnreachedCause::Barrier | UnreachedCause::SubThreshold | UnreachedCause::Cryptic => {}
        }
    }
    assert_eq!(
        all.len(),
        3,
        "the scanner pipeline {{enumerate → parse → match}} has exactly three \
         pre-evaluation drop-stages, so non-reach has exactly three causes"
    );
}

#[test]
fn each_cause_routes_a_distinct_remedy() {
    // The verdict carries the cause precisely so the remedy can differ per cause
    // (extend-patrol / widen-recall / pre-process). Collapsing them to one
    // undifferentiated "unreached" would lose the remedy-routing — the
    // cardinality-collapse the arc exists to prevent. So the three remedies must
    // be pairwise distinct.
    let b = UnreachedCause::Barrier.remedy();
    let s = UnreachedCause::SubThreshold.remedy();
    let c = UnreachedCause::Cryptic.remedy();
    assert_ne!(
        b, s,
        "Barrier and SubThreshold must route different remedies"
    );
    assert_ne!(
        s, c,
        "SubThreshold and Cryptic must route different remedies"
    );
    assert_ne!(b, c, "Barrier and Cryptic must route different remedies");
    assert!(
        !b.is_empty() && !s.is_empty() && !c.is_empty(),
        "every remedy must be actionable text, never empty"
    );
}

#[test]
fn barrier_verdict_fires_for_each_unscanned_member() {
    // "b@1" was enumerated but never scanned — a Barrier-cause unreached region.
    let report = report_with_coverage(&["a@1", "b@1", "c@1"], &["a@1", "c@1"]);
    let out = audit_coverage(&report);

    assert!(
        !out.is_complete(),
        "an enumerated-but-unscanned member means the detectable frontier is non-empty"
    );
    assert_eq!(
        out.unreached_sites.len(),
        1,
        "exactly one member (b@1) was unscanned: {:?}",
        out.unreached_sites
    );
    let site = &out.unreached_sites[0];
    assert_eq!(
        site.region, "b@1",
        "the unreached region is the unscanned member"
    );
    assert_eq!(
        site.cause,
        UnreachedCause::Barrier,
        "an unscanned member is lost at the enumerate stage = Barrier cause"
    );
    assert_eq!(
        site.remedy,
        UnreachedCause::Barrier.remedy(),
        "the verdict inlines the cause's remedy (no re-derivation by the consumer)"
    );
    assert_eq!(
        out.count_by_cause(UnreachedCause::Barrier),
        1,
        "count_by_cause must total the Barrier verdicts"
    );
    assert_eq!(
        out.count_by_cause(UnreachedCause::SubThreshold),
        0,
        "no SubThreshold verdicts fire from ScanCoverage alone (Layer-2-gated)"
    );
}

#[test]
fn barrier_frontier_is_a_set_no_duplicate_verdicts() {
    // A degenerate duplicate in enumerated_members must not produce two verdicts
    // for one member — unscanned_members() dedups (the frontier is a set), and
    // the verdict layer inherits that.
    let report = report_with_coverage(&["dup@1", "dup@1"], &[]);
    let out = audit_coverage(&report);
    assert_eq!(
        out.unreached_sites.len(),
        1,
        "the frontier is a set — one verdict per unscanned member: {:?}",
        out.unreached_sites
    );
    assert_eq!(out.unreached_sites[0].region, "dup@1");
}

#[test]
fn no_scan_coverage_yields_empty_report_tier_honest() {
    // A flat scan (no --workspace) has no member concept, so scan_coverage is
    // None. The coverage audit must return empty — NOT because coverage is
    // complete, but because the member-set needed to ask the question is absent.
    // The absence is tier-honest: it is not a completeness claim.
    let report = ScanReport::default();
    let out = audit_coverage(&report);
    assert!(
        out.is_complete(),
        "no scan_coverage ⇒ empty unreached list (the question cannot be asked)"
    );
    assert!(out.unreached_sites.is_empty());
    // The discriminator: a flat scan's is_complete()==true must NOT read as a
    // verified-complete audit — the coverage question was never applicable.
    assert!(
        !out.coverage_was_applicable(),
        "a flat scan has no member set, so coverage was not applicable — \
         is_complete() here means 'nothing detectable missed', not 'verified complete'"
    );
}

#[test]
fn complete_coverage_reports_no_unreached_sites() {
    // Every enumerated member was scanned — the detectable frontier is empty.
    let report = report_with_coverage(&["a@1", "b@1"], &["b@1", "a@1"]);
    let out = audit_coverage(&report);
    assert!(
        out.is_complete(),
        "all enumerated members scanned ⇒ no unreached sites: {:?}",
        out.unreached_sites
    );
    // The discriminator: a member-aware scan that reached every member IS a
    // verified-complete audit — coverage was applicable AND complete.
    assert!(
        out.coverage_was_applicable(),
        "a member-aware scan makes the coverage question applicable — \
         is_complete() here is a content-ful 'every member reached' claim"
    );
}

#[test]
fn coverage_was_applicable_is_the_three_state_discriminator() {
    // The core type-discipline pin (adversarial 2026-06-01): is_complete() is a
    // 2-valued bool over a 3-state domain. Without coverage_was_applicable(), a
    // library consumer cannot tell case (1) verified-complete from case (3)
    // not-applicable — both collapse to is_complete()==true. This test asserts
    // the three states are now distinguishable by the (is_complete,
    // coverage_was_applicable) PAIR.
    //
    //   state           is_complete  coverage_was_applicable
    //   (1) verified     true         true
    //   (2) incomplete   false        true
    //   (3) N/A (flat)   true         false
    let verified = audit_coverage(&report_with_coverage(&["a@1"], &["a@1"]));
    let incomplete = audit_coverage(&report_with_coverage(&["a@1", "b@1"], &["a@1"]));
    let not_applicable = audit_coverage(&ScanReport::default());

    let s1 = (verified.is_complete(), verified.coverage_was_applicable());
    let s2 = (
        incomplete.is_complete(),
        incomplete.coverage_was_applicable(),
    );
    let s3 = (
        not_applicable.is_complete(),
        not_applicable.coverage_was_applicable(),
    );

    assert_eq!(
        s1,
        (true, true),
        "state 1: member-aware, every member reached"
    );
    assert_eq!(
        s2,
        (false, true),
        "state 2: member-aware, a member unreached"
    );
    assert_eq!(
        s3,
        (true, false),
        "state 3: flat scan, coverage not applicable"
    );

    // The whole point: the three states are PAIRWISE distinct now. Before the
    // applicable flag, s1 and s3 were the same pair (true) — indistinguishable.
    assert_ne!(
        s1, s3,
        "verified-complete must be distinguishable from not-applicable"
    );
    assert_ne!(s1, s2);
    assert_ne!(s2, s3);
}

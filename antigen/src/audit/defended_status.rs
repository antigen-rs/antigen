//! P2' — the DEFENDED-STATUS sensor (the per-class witness-resolution roll-up).
//!
//! # Why this exists (the discriminator is blind for silent classes)
//!
//! The v0.6 obsolete/dormant/evaded/well-defended trichotomy
//! (`loops/discriminator-is-the-shared-dual-sensor`) wants to tell a class that is
//! **obsolete** (the failure-shape no longer occurs — safe to forget) apart from
//! one that is **well-defended** (the failure-shape would still occur, but a live
//! witness holds it back — must NOT forget). On the *silence axis* alone — "does
//! the fingerprint fire? does it harm?" — these two are **identical**: both are
//! fingerprint-silent with no harm. The only difference is the counterfactual
//! (would-it-fire-if-the-guard-were-removed), which the silence axis cannot see.
//! Antigen's whole population is **silent** failures with no prod signal, so the
//! afferent runtime discriminator is blind *exactly* where antigen is needed.
//!
//! This sensor supplies the **second axis** that breaks the tie: **witness
//! liveness**. A well-defended class carries a *live, resolving witness at its
//! sites*; an obsolete one does not. The audit already resolves witnesses
//! per-immunity (`witness_tier` / `has_witness` on
//! [`ImmunityAudit`](crate::audit::ImmunityAudit)); what was missing is the
//! **per-class roll-up** — audit emits one
//! [`ImmunityAudit`](crate::audit::ImmunityAudit) per immunity *site*, never a
//! verdict per failure-*class*.
//! Without the roll-up, WELL-DEFENDED collapses into OBSOLETE (a silent class with
//! a resolving witness looks identical to a dead one to the discriminator).
//!
//! # Scope (the do-now resolution axis, NOT the exercised-coverage axis)
//!
//! "Live" smuggles four meanings (the outsider's naive-Q): (1) EXISTS — the
//! witness identifier **resolves**; (2) RUNS — not `#[ignore]`'d; (3) PASSES —
//! green; (4) WOULD-CATCH — exercises the guarded shape (remove the guard → RED).
//! Only (4) is *real* defense; (1)–(3) can be theatre. **This sensor is the
//! RESOLUTION axis (1): does the class carry ANY witness whose tier is
//! `> None`** (the identifier resolves / defers to a real tool)? That is what the
//! existing audit cheaply computes, and it is the do-now scope (route-book row
//! 3'). The WOULD-CATCH / exercised-coverage axis (4) is a *mutation-testing*
//! organ (SEAM-B / `cargo-mutants`) and is explicitly **do-later** — so this
//! sensor reports the resolution tier honestly and never claims a class is
//! *exercised-defended* when it only knows the witness *resolves*. (A
//! `Reachability`-tier roll-up is "a witness resolves," not "the guard was
//! proven to catch" — the report carries the tier so a consumer cannot mistake
//! one for the other.)

use std::collections::BTreeMap;

use super::types::{AuditReport, WitnessTier};

/// One failure-class's defended-status — the per-class witness-resolution verdict.
///
/// The discriminator reads this to tell **well-defended** (carries a live resolving
/// witness → do NOT forget) from **obsolete** (no live witness → forgettable).
/// Computed by rolling the per-immunity [`ImmunityAudit`](super::ImmunityAudit)s up
/// by failure-class.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClassDefenseStatus {
    /// The failure-class (the `#[antigen(name = "...")]` / `antigen_type` key).
    pub antigen_type: String,
    /// The **strongest** witness tier any of this class's immunity sites carries —
    /// `max` over the class's [`ImmunityAudit::witness_tier`](super::ImmunityAudit).
    /// `None` means *no* site carries a resolving witness (the class is
    /// witness-silent → reads as OBSOLETE-candidate, not WELL-DEFENDED). Anything
    /// `> None` means at least one site's witness resolves → WELL-DEFENDED on the
    /// resolution axis.
    pub max_witness_tier: WitnessTier,
    /// How many immunity *sites* defend this class (the number of
    /// [`ImmunityAudit`](super::ImmunityAudit)s that rolled into this class).
    pub site_count: usize,
    /// How many of those sites carry a *resolving* witness (`tier > None`). When
    /// this is `0` the class is undefended-on-resolution even if `site_count > 0`
    /// (every site asserted immunity without a resolving witness).
    pub resolving_site_count: usize,
}

impl ClassDefenseStatus {
    /// Is this class **defended on the resolution axis** — does at least one of its
    /// sites carry a witness whose tier is `> None`?
    ///
    /// This is the bit the discriminator reads to keep WELL-DEFENDED distinct from
    /// OBSOLETE. It is the RESOLUTION reading (the witness *resolves*), **not** a
    /// claim the guard was proven to catch (that is the do-later exercised-coverage
    /// axis — read [`max_witness_tier`](Self::max_witness_tier) for the strength).
    #[must_use]
    pub fn is_defended_on_resolution(&self) -> bool {
        self.max_witness_tier > WitnessTier::None
    }
}

/// The per-class defended-status roll-up over a whole [`AuditReport`].
///
/// One [`ClassDefenseStatus`] per distinct `antigen_type` present in the report's
/// per-immunity audits. This is the sensor the obsolete/well-defended
/// discriminator reads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct DefendedStatusReport {
    /// Per-class defended-status, keyed and **sorted** by `antigen_type`
    /// (`BTreeMap` for deterministic, reproducible output — the same report
    /// renders identically across runs/machines, the git-native determinism the
    /// life-record/catalog rely on).
    pub by_class: BTreeMap<String, ClassDefenseStatus>,
}

impl DefendedStatusReport {
    /// The classes that are **undefended on the resolution axis** — no site carries
    /// a resolving witness (`max_witness_tier == None`). These are the
    /// OBSOLETE-*candidates* the discriminator may forget (subject to the other
    /// axes); a well-defended class is, by construction, NOT in this set.
    #[must_use]
    pub fn undefended_classes(&self) -> Vec<&ClassDefenseStatus> {
        self.by_class
            .values()
            .filter(|c| !c.is_defended_on_resolution())
            .collect()
    }
}

/// Roll the report's per-immunity audits up into a per-class defended-status
/// sensor (P2').
///
/// Groups [`AuditReport::audits`] by `immunity.antigen_type` and, per class,
/// computes the **max** witness tier across its sites (the strongest resolving
/// evidence any site carries) plus the site counts. A class is **well-defended on
/// the resolution axis** iff its `max_witness_tier > None`.
///
/// This reads only what the audit already resolved — it invokes no witnesses and
/// runs no code (the resolution axis, not the exercised-coverage axis).
#[must_use]
pub fn audit_defended_status(report: &AuditReport) -> DefendedStatusReport {
    let mut by_class: BTreeMap<String, ClassDefenseStatus> = BTreeMap::new();

    for audit in &report.audits {
        let class = &audit.immunity.antigen_type;
        let resolves = audit.witness_tier > WitnessTier::None;
        let entry = by_class
            .entry(class.clone())
            .or_insert_with(|| ClassDefenseStatus {
                antigen_type: class.clone(),
                max_witness_tier: WitnessTier::None,
                site_count: 0,
                resolving_site_count: 0,
            });
        entry.site_count += 1;
        if resolves {
            entry.resolving_site_count += 1;
        }
        if audit.witness_tier > entry.max_witness_tier {
            entry.max_witness_tier = audit.witness_tier;
        }
    }

    DefendedStatusReport { by_class }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::audit::{AuditHint, ImmunityAudit, WitnessStatus};
    use crate::scan::{Immunity, ItemTarget};

    /// Build a minimal `ImmunityAudit` for `class` at `tier` (the only fields this
    /// sensor reads are `immunity.antigen_type` and `witness_tier`).
    fn audit_for(class: &str, tier: WitnessTier) -> ImmunityAudit {
        ImmunityAudit {
            immunity: Immunity {
                antigen_type: class.to_owned(),
                witness: "w".to_owned(),
                requires_predicate: None,
                file: PathBuf::from("lib.rs"),
                line: 1,
                item_kind: "fn".to_owned(),
                item_target: ItemTarget::Fn("w".to_owned()),
                canonical_path: None,
                structural_fingerprint: String::new(),
            },
            witness_status: WitnessStatus::Missing,
            witness_tier: tier,
            audit_hint: AuditHint::NoneApplicable,
            evidence_kind: antigen_attestation::EvidenceKind::None,
            signature_strength: None,
            compound_evidence: false,
            evaluated_predicate: None,
            code_witness_sidecar_ignored: false,
            leaf_outcomes: Vec::new(),
        }
    }

    /// THE LOAD-BEARING CRITERION (the homonym trap the route-book warns about): a
    /// silent class with a RESOLVING witness (WELL-DEFENDED) must be
    /// DISTINGUISHABLE from a silent class with NO resolving witness (OBSOLETE).
    /// The sensor distinguishes them by `max_witness_tier > None` per class — NOT
    /// by "has SOME audit row" (both have rows). That distinction is the whole
    /// point of the sensor (without it WELL-DEFENDED collapses into OBSOLETE).
    #[test]
    fn well_defended_is_distinguishable_from_obsolete_by_resolution_tier() {
        let report = AuditReport {
            audits: vec![
                // A well-defended class: its site carries a resolving witness.
                audit_for("well-defended-class", WitnessTier::Reachability),
                // An obsolete-candidate class: a site exists but no witness resolves.
                audit_for("obsolete-class", WitnessTier::None),
            ],
            ..Default::default()
        };

        let sensor = audit_defended_status(&report);

        let well = &sensor.by_class["well-defended-class"];
        let obsolete = &sensor.by_class["obsolete-class"];

        // The homonym ("has SOME audit row") would call BOTH defended — both have a
        // row. The real criterion separates them:
        assert!(
            well.is_defended_on_resolution(),
            "a class with a resolving (tier>None) witness is WELL-DEFENDED on the \
             resolution axis — the discriminator must NOT forget it."
        );
        assert!(
            !obsolete.is_defended_on_resolution(),
            "a class with only tier=None sites carries NO resolving witness — it is \
             an OBSOLETE-candidate, NOT well-defended. If the sensor called this \
             defended (the 'has-a-row' homonym), WELL-DEFENDED would collapse into \
             OBSOLETE and the discriminator would refuse to forget dead classes."
        );
    }

    /// The roll-up takes the MAX tier across a class's sites: one resolving site is
    /// enough to make the class well-defended even if a sibling site is bare.
    #[test]
    fn max_tier_rolls_up_across_a_classes_sites() {
        let report = AuditReport {
            audits: vec![
                audit_for("multi-site", WitnessTier::None),
                audit_for("multi-site", WitnessTier::Execution),
                audit_for("multi-site", WitnessTier::Reachability),
            ],
            ..Default::default()
        };

        let sensor = audit_defended_status(&report);
        let c = &sensor.by_class["multi-site"];

        assert_eq!(c.site_count, 3, "all three sites roll into the one class");
        assert_eq!(
            c.resolving_site_count, 2,
            "two of the three sites resolve (tier>None)"
        );
        assert_eq!(
            c.max_witness_tier,
            WitnessTier::Execution,
            "the class's defended-strength is the MAX tier across its sites — one \
             Execution-tier site makes the class Execution-defended."
        );
        assert!(c.is_defended_on_resolution());
    }

    /// A class every site of which is tier=None is undefended-on-resolution and
    /// shows up in `undefended_classes()` (the discriminator's forget-candidate set).
    #[test]
    fn all_bare_sites_make_a_class_undefended() {
        let report = AuditReport {
            audits: vec![
                audit_for("bare", WitnessTier::None),
                audit_for("bare", WitnessTier::None),
            ],
            ..Default::default()
        };

        let sensor = audit_defended_status(&report);
        let undefended = sensor.undefended_classes();

        assert_eq!(undefended.len(), 1);
        assert_eq!(undefended[0].antigen_type, "bare");
        assert_eq!(undefended[0].resolving_site_count, 0);
    }
}

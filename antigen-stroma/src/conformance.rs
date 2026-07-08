//! The conformance-monitor — runs the intent-substrate's COMPARE against REAL scan output.
//!
//! [`crate::intent`] built the setpoint (`Should`) and a three-valued [`compare`] over an *abstract*
//! observed-`Is`. This module is its first LIVE caller: it derives the observed-`Is` from an actual
//! [`ScanReport`] (the afferent scan of a codebase) and runs COMPARE per locus — so the three-valued
//! conformance-gap runs against a real repository, not a hand-built fixture. It MEASURES the residual;
//! it does not generate a `Should`, route a response, or act on the gap (those are `compare-hinge`,
//! `route-arm`, and the effector charters).
//!
//! ## The adapter: scan records → observed strength
//!
//! For each `(locus, class)`, the observed [`Strength`] is derived from the scan's markers:
//! - a `#[defended_by]` registration (`Defense`, ADR-029) or a legacy `#[immune]` site (`Immunity`),
//!   or a `Presentation` carrying site-attached defense evidence (`proof` / `requires_predicate`) →
//!   **`Guarded`** (actively defended);
//! - a bare `Presentation` (the class is *presented* — acknowledged as a vulnerability — but not
//!   defended) → **`Present`**;
//! - no marker for the class at the locus → **`None`** (absent from the observed `Is`).
//!
//! Strengths merge by **max** across markers (a class with both a bare presents AND an immunity reads
//! `Guarded`).
//!
//! ## Fidelity boundary (honest scope)
//!
//! Locus-matching keys on `<file>::<item-label>` ([`ItemTarget::label`]) — a stable, authorable key,
//! but NOT a canonical module-path resolution. A setpoint written as a module path (`crate::foo::bar`)
//! and a scan site (`src/foo.rs::bar`) match only when their keys agree; precise path resolution is
//! deferred to the resolution layer (ADR-069), exactly as the scan itself runs at syntactic tier until
//! SCIP/mir resolution lands. Tolerations (`#[antigen_tolerance]`) are not yet folded in — the
//! `Disposition::Tolerated` path is exercised by the abstract [`compare`] tests today; wiring the
//! scan's declared exceptions is a named next increment.

use std::collections::BTreeMap;
use std::path::Path;

use antigen::scan::{ItemTarget, ScanReport};

use crate::intent::{ConformanceGap, ObservedDefense, Setpoint, Strength, compare};

/// The locus key a scan site is grouped under: `<file>::<item-label>`.
///
/// A setpoint's `locus` is matched against this. See the module's fidelity-boundary note — this is an
/// authorable key, not a canonical module-path resolution.
#[must_use]
pub fn locus_key(file: &Path, item: &ItemTarget) -> String {
    format!("{}::{}", file.display(), item.label())
}

/// Raise a class's recorded strength to at least `strength` (the max-merge across markers).
fn raise(observed: &mut BTreeMap<String, Strength>, class: &str, strength: Strength) {
    let entry = observed.entry(class.to_string()).or_insert(Strength::None);
    if strength > *entry {
        *entry = strength;
    }
}

/// Derive the observed defenses at `locus` from the scan report (the adapter — `Is`, per locus).
#[must_use]
pub fn observed_at(scan: &ScanReport, locus: &str) -> Vec<ObservedDefense> {
    let mut by_class: BTreeMap<String, Strength> = BTreeMap::new();

    // Presentations: a bare `#[presents(X)]` is `Present` (acknowledged, undefended); one carrying
    // site-attached defense evidence (`proof` / `requires`) is `Guarded`.
    for p in &scan.presentations {
        if locus_key(&p.file, &p.item_target) == locus {
            let strength = if p.proof.is_some() || p.requires_predicate.is_some() {
                Strength::Guarded
            } else {
                Strength::Present
            };
            raise(&mut by_class, &p.antigen_type, strength);
        }
    }

    // Defenses: a `#[defended_by(X)]` code-tier witness registration (ADR-029, the current path) is an
    // active defense → `Guarded`.
    for d in &scan.defenses {
        if locus_key(&d.file, &d.item_target) == locus {
            raise(&mut by_class, &d.antigen_type, Strength::Guarded);
        }
    }

    // Immunities: a legacy `#[immune]` site is an active defense → `Guarded`.
    for im in &scan.immunities {
        if locus_key(&im.file, &im.item_target) == locus {
            raise(&mut by_class, &im.antigen_type, Strength::Guarded);
        }
    }

    by_class
        .into_iter()
        .map(|(class, strength)| ObservedDefense { class, strength })
        .collect()
}

/// The per-locus conformance result: the three-valued gaps for one setpoint's locus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocusConformance {
    /// The locus this setpoint governs.
    pub locus: String,
    /// The three-valued verdict for each of the setpoint's expectations.
    pub gaps: Vec<ConformanceGap>,
}

/// The whole-codebase conformance report — one [`LocusConformance`] per setpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceReport {
    /// Per-locus conformance, in setpoint order.
    pub loci: Vec<LocusConformance>,
}

/// Run COMPARE for every setpoint against the scan's observed `Is` — the conformance-monitor's whole job.
///
/// For each setpoint, adapt the scan into observed defenses at its locus, then emit the three-valued
/// conformance-gap ([`crate::intent::compare`]).
#[must_use]
pub fn conformance_report(setpoints: &[Setpoint], scan: &ScanReport) -> ConformanceReport {
    let loci = setpoints
        .iter()
        .map(|setpoint| {
            let observed = observed_at(scan, &setpoint.locus);
            LocusConformance {
                locus: setpoint.locus.clone(),
                gaps: compare(setpoint, &observed),
            }
        })
        .collect();
    ConformanceReport { loci }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use antigen::scan::{Defense, ItemTarget, MatchKind, Presentation, ScanReport};

    use super::{conformance_report, locus_key, observed_at};
    use crate::intent::{Attestation, Conformance, Disposition, Expectation, Setpoint, Strength};

    const FILE: &str = "src/boundary.rs";

    fn fn_item(name: &str) -> ItemTarget {
        ItemTarget::Fn(name.to_string())
    }

    fn presentation(class: &str, item: &str, defended: bool) -> Presentation {
        Presentation {
            antigen_type: class.to_string(),
            file: PathBuf::from(FILE),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: fn_item(item),
            match_kind: MatchKind::ExplicitMarker,
            canonical_path: None,
            inherited_from: None,
            structural_fingerprint: String::new(),
            requires_predicate: None,
            proof: if defended {
                Some("NonPanicProof::verified".to_string())
            } else {
                None
            },
        }
    }

    fn defense(class: &str, item: &str) -> Defense {
        Defense {
            antigen_type: class.to_string(),
            file: PathBuf::from(FILE),
            line: 1,
            item_kind: "fn".to_string(),
            item_target: fn_item(item),
            canonical_path: None,
        }
    }

    fn setpoint_guarding(classes: &[&str]) -> Setpoint {
        Setpoint {
            locus: locus_key(&PathBuf::from(FILE), &fn_item("deserialize")),
            expects: classes
                .iter()
                .map(|c| Expectation {
                    class: (*c).to_string(),
                    disposition: Disposition::Guarded,
                })
                .collect(),
            attestation: Attestation {
                who: "alice".to_string(),
                why: "trust boundary".to_string(),
            },
        }
    }

    #[test]
    fn live_scan_yields_the_three_valued_gap() {
        // `deserialize()` SHOULD guard three classes. Scan reality:
        //   - UntrustedInput has a `#[defended_by]`      → Conforms
        //   - PanicOnMalformed presented but undefended  → Underdeveloped
        //   - IntegerOverflow not mentioned at the locus → Absent
        // This is the keystone made LIVE — a bare boolean "is it handled?" cannot tell these apart.
        let scan = ScanReport {
            presentations: vec![presentation("PanicOnMalformed", "deserialize", false)],
            defenses: vec![defense("UntrustedInput", "deserialize")],
            ..Default::default()
        };
        let sp = setpoint_guarding(&["UntrustedInput", "PanicOnMalformed", "IntegerOverflow"]);
        let report = conformance_report(&[sp], &scan);
        assert_eq!(report.loci.len(), 1);
        let verdicts: Vec<_> = report.loci[0]
            .gaps
            .iter()
            .map(|g| (g.class.as_str(), g.verdict))
            .collect();
        assert_eq!(
            verdicts,
            vec![
                ("UntrustedInput", Conformance::Conforms),
                ("PanicOnMalformed", Conformance::Underdeveloped),
                ("IntegerOverflow", Conformance::Absent),
            ]
        );
    }

    #[test]
    fn a_defense_at_a_different_locus_does_not_count() {
        // The defense sits on a DIFFERENT fn → the setpoint's locus sees Absent, not Conforms. This is
        // the locus-matching having teeth (a whole-report OR would falsely conform).
        let scan = ScanReport {
            defenses: vec![defense("UntrustedInput", "some_other_fn")],
            ..Default::default()
        };
        let report = conformance_report(&[setpoint_guarding(&["UntrustedInput"])], &scan);
        assert_eq!(report.loci[0].gaps[0].verdict, Conformance::Absent);
    }

    #[test]
    fn observed_strength_merges_by_max() {
        // A class both PRESENTED (bare) and DEFENDED at one locus reads Guarded (max), not Present.
        let scan = ScanReport {
            presentations: vec![presentation("X", "deserialize", false)],
            defenses: vec![defense("X", "deserialize")],
            ..Default::default()
        };
        let locus = locus_key(&PathBuf::from(FILE), &fn_item("deserialize"));
        let observed = observed_at(&scan, &locus);
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].strength, Strength::Guarded);
    }

    #[test]
    fn site_attached_proof_counts_as_guarded() {
        // A `#[presents(X, proof = ...)]` (defense evidence on the presents-site) reads Guarded.
        let scan = ScanReport {
            presentations: vec![presentation("X", "deserialize", true)],
            ..Default::default()
        };
        let locus = locus_key(&PathBuf::from(FILE), &fn_item("deserialize"));
        assert_eq!(observed_at(&scan, &locus)[0].strength, Strength::Guarded);
    }
}

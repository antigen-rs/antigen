//! The intent-substrate — the load-bearing `Should` (the hinge's setpoint).
//!
//! The two-winged body (research nb 022 §4) names this as the **keystone**: the COMPARE hinge is
//! `Should − Is`, and without a real `Should` to subtract against, antigen is "a sensor with a twitch,
//! not a loop." A declared failure-class alone is a *thin* `Should` — a single fact to fold over. The
//! intent-substrate is what turns that thin declared `Should` into a genuine **reference / setpoint**:
//! the thing the efferent wing regulates toward, and the seed `propose` unfolds from.
//!
//! ## The cheapest real anchor: attested `(who, why)` (ADR-020)
//!
//! Intent is captured as **attestation, not inference** (ADR-020): a `Should` carries WHO declared it
//! and WHY. This is rank-1 today (an attested provenance tier); the deeper "what was actually MEANT"
//! version (the AIRE-analog the learning-core named) is genuinely research-open and NOT built here.
//! This module is the honest, load-bearing floor — a setpoint you can subtract `Is` against — not the
//! ceiling.
//!
//! ## What COMPARE produces: a three-valued conformance-gap
//!
//! COMPARE against the setpoint is **three-valued** ([`Conformance`]), matching antigen's three-valued
//! trust law (a boundary either conforms, is unmet, or is present-but-weak — never a bare pass/fail).
//! That three-valued gap is the hinge *bearing load*: the error is a genuine, actionable conformance
//! gap, not a near-empty difference.
//!
//! ## Scope (honest boundary)
//!
//! This is the setpoint TYPE + a minimal COMPARE over an *abstract* observed-`Is` ([`ObservedDefense`]).
//! Wiring COMPARE to the real scan pipeline (the afferent `Is`), the deep what-was-meant capture, and
//! the 4-dim control-observer (once the efferent wing servos around this setpoint) are all deferred —
//! the setpoint is authored CONFIG (ADR-067 §F.13), built ahead of the callers that structurally
//! require it (`conformance-monitor`, the effector, `propose`).

/// WHO declared a `Should`, and WHY (ADR-020 — intent as attestation, not inference).
///
/// The cheapest real anchor of intent: a rank-1 `(who, why)` provenance. Aligns with the richer
/// signer/rationale model in the `antigen-attestation` crate, but is kept a minimal local primitive
/// here (compose later; do not couple the setpoint to the full attestation-evaluation layer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attestation {
    /// The signer / author who declared this `Should`.
    pub who: String,
    /// The rationale — why this locus is expected to defend what it does.
    pub why: String,
}

/// The intended DISPOSITION of a defense at a locus — the role-typology.
///
/// Absorbed from pith's argument-role classification (research nb 023 Part IV): what KIND of defense
/// the `Should` expects, not merely that *a* defense exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    /// The locus must ACTIVELY defend the class — a real, adequate witness is required.
    Guarded,
    /// The locus must carry a WITNESS (a test / proptest / clippy / kani / … attestation), but the
    /// witness need only exist, not be the strongest tier.
    Witnessed,
    /// The locus is EXPLICITLY allowed to present the class without defending it — a declared,
    /// attested exception (the tolerated / self-tolerant disposition; never a silent gap).
    Tolerated,
}

/// A single expected defense at a locus: failure-`class` should be handled with `disposition`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expectation {
    /// The failure-class name the locus is expected to defend against.
    pub class: String,
    /// The intended disposition for that class at this locus.
    pub disposition: Disposition,
}

/// The SETPOINT — the load-bearing `Should` at one structural locus (module / trait / fn).
///
/// Authored CONFIG (ADR-067 §F.13): the sovereign, never-recomputed half — a human/agent declares it,
/// antigen never infers it. This is the reference the COMPARE hinge subtracts `Is` against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setpoint {
    /// The structural locus this `Should` is attached to (a module path / trait / function).
    pub locus: String,
    /// What SHOULD be defended here, and with what disposition.
    pub expects: Vec<Expectation>,
    /// WHO declared this `Should`, and WHY (the rank-1 intent anchor).
    pub attestation: Attestation,
}

/// The `Is` side, abstracted: what a locus is OBSERVED to defend, and how strongly.
///
/// A minimal stand-in for the afferent scan output, so COMPARE can be demonstrated and tested without
/// the full scan wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedDefense {
    /// The failure-class name observed to be handled at the locus.
    pub class: String,
    /// The observed strength of that handling (the afferent grade — see [`Strength`]).
    pub strength: Strength,
}

/// The observed strength of a defense (the `Is`-side grade COMPARE weighs against the disposition).
///
/// Ordered `None < Present < Guarded`: `None` is no handling at all; `Present` is a witness exists but
/// is not an active guard; `Guarded` is an active, adequate defense. This mirrors the afferent
/// detection-grade ladder (form-only vs content-grade) without importing it — the intent-substrate
/// weighs its own three levels against the [`Disposition`] the setpoint demands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Strength {
    /// No handling observed.
    None,
    /// A witness is present (the class is acknowledged / tested) but not actively guarded.
    Present,
    /// The class is actively, adequately guarded.
    Guarded,
}

/// The three-valued conformance verdict for ONE expectation (antigen's three-valued trust law).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conformance {
    /// The expected defense is present AND adequate for its disposition.
    Conforms,
    /// The setpoint is UNMET — the expected defense is missing entirely (`Strength::None`).
    Absent,
    /// The expected defense is present but WEAKER than its disposition demands (e.g. a `Guarded`
    /// expectation met only by a `Present` witness).
    Underdeveloped,
}

/// The conformance of one expectation against the observed `Is` at the locus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceGap {
    /// The failure-class this verdict concerns.
    pub class: String,
    /// The three-valued verdict.
    pub verdict: Conformance,
}

/// The minimal strength a disposition demands to count as [`Conformance::Conforms`].
const fn required_strength(disposition: Disposition) -> Strength {
    match disposition {
        // A guarded expectation needs an active guard; a witnessed one needs at least a present
        // witness; a tolerated one is satisfied at any strength (the exception is the point).
        Disposition::Guarded => Strength::Guarded,
        Disposition::Witnessed => Strength::Present,
        Disposition::Tolerated => Strength::None,
    }
}

/// COMPARE = `Should − Is` at one locus: for each expectation in the setpoint, weigh the observed
/// strength against the disposition's requirement and emit a three-valued [`ConformanceGap`].
///
/// This is the hinge *bearing load* — the setpoint turns "is class X handled?" (a bare boolean fold)
/// into "is X handled the way it was MEANT to be?" (a three-valued gap against an attested reference).
///
/// - observed `>=` required → [`Conformance::Conforms`];
/// - observed is `None` (nothing at all) → [`Conformance::Absent`];
/// - observed is present but `<` required → [`Conformance::Underdeveloped`].
#[must_use]
pub fn compare(setpoint: &Setpoint, observed: &[ObservedDefense]) -> Vec<ConformanceGap> {
    setpoint
        .expects
        .iter()
        .map(|expectation| {
            let strength = observed
                .iter()
                .find(|o| o.class == expectation.class)
                .map_or(Strength::None, |o| o.strength);
            let required = required_strength(expectation.disposition);
            let verdict = if strength >= required {
                Conformance::Conforms
            } else if strength == Strength::None {
                Conformance::Absent
            } else {
                Conformance::Underdeveloped
            };
            ConformanceGap {
                class: expectation.class.clone(),
                verdict,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        Attestation, Conformance, Disposition, Expectation, ObservedDefense, Setpoint, Strength,
        compare,
    };

    fn setpoint(expects: Vec<Expectation>) -> Setpoint {
        Setpoint {
            locus: "crate::boundary::deserialize".to_string(),
            expects,
            attestation: Attestation {
                who: "alice".to_string(),
                why: "trust boundary: untrusted input crosses here".to_string(),
            },
        }
    }

    fn expect(class: &str, disposition: Disposition) -> Expectation {
        Expectation {
            class: class.to_string(),
            disposition,
        }
    }

    fn observed(class: &str, strength: Strength) -> ObservedDefense {
        ObservedDefense {
            class: class.to_string(),
            strength,
        }
    }

    #[test]
    fn guarded_expectation_met_by_a_guard_conforms() {
        let sp = setpoint(vec![expect("UntrustedDeserialize", Disposition::Guarded)]);
        let gaps = compare(&sp, &[observed("UntrustedDeserialize", Strength::Guarded)]);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].verdict, Conformance::Conforms);
    }

    #[test]
    fn guarded_expectation_with_no_defense_is_absent() {
        // The setpoint is UNMET — the class the locus was MEANT to guard is not handled at all.
        let sp = setpoint(vec![expect("UntrustedDeserialize", Disposition::Guarded)]);
        let gaps = compare(&sp, &[]);
        assert_eq!(gaps[0].verdict, Conformance::Absent);
    }

    #[test]
    fn guarded_expectation_met_only_by_a_witness_is_underdeveloped() {
        // The three-valued distinction the setpoint BUYS: a bare boolean fold ("is it handled?") would
        // say yes; the setpoint says "guarded was MEANT, only a witness is present" → underdeveloped.
        let sp = setpoint(vec![expect("UntrustedDeserialize", Disposition::Guarded)]);
        let gaps = compare(&sp, &[observed("UntrustedDeserialize", Strength::Present)]);
        assert_eq!(gaps[0].verdict, Conformance::Underdeveloped);
    }

    #[test]
    fn witnessed_expectation_met_by_a_witness_conforms() {
        let sp = setpoint(vec![expect("PanicOnEmpty", Disposition::Witnessed)]);
        let gaps = compare(&sp, &[observed("PanicOnEmpty", Strength::Present)]);
        assert_eq!(gaps[0].verdict, Conformance::Conforms);
    }

    #[test]
    fn tolerated_expectation_conforms_at_any_strength() {
        // A tolerated class is an attested exception — present or not, it conforms (the exception IS
        // the point; it is never a silent gap because it was declared with a disposition + rationale).
        let sp = setpoint(vec![expect("KnownAllocInHotPath", Disposition::Tolerated)]);
        assert_eq!(
            compare(&sp, &[]).first().unwrap().verdict,
            Conformance::Conforms
        );
        assert_eq!(
            compare(&sp, &[observed("KnownAllocInHotPath", Strength::Present)])
                .first()
                .unwrap()
                .verdict,
            Conformance::Conforms
        );
    }

    #[test]
    fn stronger_than_required_still_conforms() {
        // A witnessed expectation over-met by an active guard conforms (>= required, not == required).
        let sp = setpoint(vec![expect("PanicOnEmpty", Disposition::Witnessed)]);
        let gaps = compare(&sp, &[observed("PanicOnEmpty", Strength::Guarded)]);
        assert_eq!(gaps[0].verdict, Conformance::Conforms);
    }

    #[test]
    fn teeth_a_vacuous_always_conforms_compare_would_fail_the_gap_cases() {
        // NEGATIVE CONTROL: the load-bearing claim is that COMPARE produces a GENUINE three-valued gap,
        // not a rubber stamp. A vacuous `compare` that always returned `Conforms` would pass the
        // conforms tests above but FAIL here — so these three verdicts, together, have teeth.
        let sp = setpoint(vec![
            expect("A", Disposition::Guarded), // met by a guard   → Conforms
            expect("B", Disposition::Guarded), // missing entirely → Absent
            expect("C", Disposition::Guarded), // only a witness   → Underdeveloped
        ]);
        let gaps = compare(
            &sp,
            &[
                observed("A", Strength::Guarded),
                observed("C", Strength::Present),
            ],
        );
        let verdicts: Vec<_> = gaps.iter().map(|g| g.verdict).collect();
        assert_eq!(
            verdicts,
            vec![
                Conformance::Conforms,
                Conformance::Absent,
                Conformance::Underdeveloped,
            ]
        );
    }
}

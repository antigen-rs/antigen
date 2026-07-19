//! # `ds` — Dempster-Shafer evidence on the binary frame `{Real, ¬Real}`: belief, plausibility, conflict.
//!
//! Clone #3 of the spine (nb 030 §4), the **H¹-obstruction stain** (§7) and the empirical
//! **hold-all-vs-Dempster** test (§8).
//!
//! Each instrument contributes a mass function over the frame `Θ = {Real, NotReal}`: mass on `{Real}`, on
//! `{NotReal}`, and on `Θ` itself (`unknown` = "committed, but can't say which"). `Belief(Real) = m(Real)`
//! is the lower bound; `Plausibility(Real) = m(Real) + m(Θ)` the upper — the `[Bel, Pl]` bracket that IS the
//! tier bracket (§2). Dempster's rule combines two independent masses; the mass landing on `∅` (one says
//! Real, the other `NotReal`) is the **conflict K**, and the rule normalizes by `1/(1−K)` — *discarding* K.
//!
//! That discard is the whole interest: DS's signature strength over probability is that it separates
//! **conflict** (`K`, active disagreement) from **ignorance** (`m(Θ)`, nobody committed) — but its
//! *combination* then erases K. hold-all keeps both witnesses, so it preserves the very conflict Dempster
//! throws away. This clone stains the obstruction AND measures whether that erasure loses signal.

/// A basic probability assignment on the binary frame `{Real, NotReal}`. Invariant:
/// `real + not_real + unknown == 1`, where `unknown` is the mass on the whole frame `Θ`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mass {
    /// Mass committed to `{Real}` — the edge/fact is real.
    pub real: f64,
    /// Mass committed to `{NotReal}` — the edge/fact is not real.
    pub not_real: f64,
    /// Mass on the whole frame `Θ` — committed, but undetermined which (ignorance, not conflict).
    pub unknown: f64,
}

impl Mass {
    /// `Belief(Real)` — total mass committed to Real (lower bound of the evidential interval).
    #[must_use]
    pub const fn belief_real(&self) -> f64 {
        self.real
    }

    /// `Plausibility(Real)` — mass not committed against Real, i.e. `real + unknown` (upper bound).
    #[must_use]
    pub const fn plausibility_real(&self) -> f64 {
        self.real + self.unknown
    }

    /// Dempster-combine with another independent mass. Returns the combined mass AND the **conflict K**
    /// (the mass that fell on `∅`, which the rule normalizes away). Total conflict (`K == 1`) is undefined
    /// and returns `None`.
    #[must_use]
    // The combine terms are the exact Dempster products (intersections that hit each focal element); the
    // asymmetry is correct, and mul_add would obscure the belief-function structure for no real accuracy gain.
    #[allow(clippy::suspicious_operation_groupings, clippy::suboptimal_flops)]
    pub fn combine(self, other: Self) -> Option<(Self, f64)> {
        let k = self.real * other.not_real + self.not_real * other.real; // mass on ∅
        let norm = 1.0 - k;
        if norm <= f64::EPSILON {
            return None;
        }
        let real =
            (self.real * other.real + self.real * other.unknown + self.unknown * other.real) / norm;
        let not_real = (self.not_real * other.not_real
            + self.not_real * other.unknown
            + self.unknown * other.not_real)
            / norm;
        let unknown = (self.unknown * other.unknown) / norm;
        Some((
            Self {
                real,
                not_real,
                unknown,
            },
            k,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Mass;

    fn m(real: f64, not_real: f64, unknown: f64) -> Mass {
        Mass {
            real,
            not_real,
            unknown,
        }
    }
    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn belief_never_exceeds_plausibility() {
        let x = m(0.6, 0.1, 0.3);
        assert!(x.belief_real() <= x.plausibility_real()); // real ≤ real + unknown
        assert!(approx(x.plausibility_real(), 0.9));
    }

    #[test]
    fn agreement_raises_belief_with_no_conflict() {
        // Two instruments both leaning Real → belief climbs above either, K = 0.
        let (combined, k) = m(0.8, 0.0, 0.2).combine(m(0.8, 0.0, 0.2)).unwrap();
        assert!(approx(k, 0.0));
        assert!(combined.belief_real() > 0.8); // 0.96
        assert!(approx(
            combined.real + combined.not_real + combined.unknown,
            1.0
        ));
    }

    #[test]
    fn contradiction_produces_conflict_that_normalization_then_discards() {
        // syn says Real, SCIP says NotReal → high K; but the combined mass sums to 1 with K GONE from it.
        let syn = m(0.8, 0.0, 0.2);
        let scip = m(0.0, 0.8, 0.2);
        let (combined, k) = syn.combine(scip).unwrap();
        assert!(approx(k, 0.64)); // 0.8·0.8 — the conflict is large
        // ...yet the result is a bland 0.44/0.44/0.11 that no longer encodes the disagreement:
        assert!(approx(combined.real, combined.not_real));
        assert!(approx(
            combined.real + combined.not_real + combined.unknown,
            1.0
        ));
        assert!(combined.real < 0.5); // looks like mild uncertainty, not a violent conflict
    }

    #[test]
    fn ignorance_is_not_conflict() {
        // syn says Real, SCIP is purely ignorant (all mass on Θ) → K = 0 (no disagreement, just unknowing).
        let (_, k) = m(0.8, 0.0, 0.2).combine(m(0.0, 0.0, 1.0)).unwrap();
        assert!(approx(k, 0.0)); // the DS signature: ignorance ≠ conflict
    }

    #[test]
    fn total_conflict_is_undefined() {
        assert!(m(1.0, 0.0, 0.0).combine(m(0.0, 1.0, 0.0)).is_none());
    }
}

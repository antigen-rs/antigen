//! The resolution tier ladder + the lower-never-corroborates-up invariant.
//!
//! `ResolutionTier` is an ordered enum. The detection grade a read can earn is CAPPED by its source
//! tier ([`ResolutionTier::detection_ceiling`]) — this is the type-level enforcement of ADR-069 §A:
//! a syntactic read cannot construct a `presents`-grade verdict.

/// The source-resolution tier of a fact (ADR-069 §A). Ordered: `Syntactic < Resolved < T3Mir`.
///
/// `T3Mir` is a live slot — structurally present but unpopulated on a dyn/async/closure-free
/// codebase (antigen's own). The frame reads arbitrary user codebases; a tokio/dyn-heavy one
/// populates it. If a read comes back at `T3Mir` tier with an empty value, treat it as
/// empty-but-structurally-valid, not an error: mir-exact population is computed by the
/// datalog-closure layer, scoped to the queried region rather than the whole graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolutionTier {
    /// T1 — syntactic (syn walk; r-a-free). Detection ceiling = `dread`-grade, NEVER `presents`.
    Syntactic,
    /// T2 — resolved (SCIP symbol-level). Detection ceiling = `presents`-grade.
    Resolved,
    /// T3 — mir-only (live slot; empty on dyn/async/closure-free code). Detection ceiling = `presents`-grade.
    T3Mir,
}

/// The detection grade a verdict can carry (the immune-attribute grade vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DetectionGrade {
    /// Form-only; the floor. A syntactic source can reach at most this.
    Dread,
    /// Content-grade. Requires a resolved source.
    Presents,
}

/// A witness that a `presents`-grade verdict was legitimately earned — the C3 tier-cap made a
/// COMPILE-TIME impossibility (ADR-070 §3.2), not a runtime check.
///
/// **`PresentsVerdict` has NO public constructor.** Its single field is private, so no code outside
/// this module can mint one by hand — a `source = syntactic` read literally cannot *type-check* a
/// `presents` answer. The ONE privileged door is [`corroborate_presents`], which mints it ONLY when
/// two fresh-independent resolved-or-higher sources converge. This is the lower-never-corroborates-up
/// law (ADR-069 §A) encoded in the type system: antigen's hold-you-honest-by-construction thesis
/// dogfooded onto its own read contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentsVerdict {
    /// The convergent tier the corroboration earned (always ≥ `Resolved`). Private — the field's
    /// privacy is what makes the verdict unforgeable from outside this module.
    earned_at: ResolutionTier,
}

impl PresentsVerdict {
    /// The tier the corroboration earned this presents-grade verdict at (≥ `Resolved`).
    #[must_use]
    pub const fn earned_at(self) -> ResolutionTier {
        self.earned_at
    }
}

/// THE single privileged door that mints a [`PresentsVerdict`] (ADR-070 §3.2). Convergence raises
/// grade ONLY across fresh-independent sources; this is the ONLY constructor of a presents witness.
///
/// Returns `Some` iff [`corroborate`] succeeds — i.e. neither input is `Syntactic` (a form-only /
/// demoted-stale source can't lift content) and the two converge to a resolved-or-higher tier.
/// `corroborate_presents(Syntactic, _)` and `corroborate_presents(_, Syntactic)` ⇒ `None`; two
/// independent resolved sources ⇒ `Some` (the negative-control path the cap must still permit).
#[must_use]
pub fn corroborate_presents(a: ResolutionTier, b: ResolutionTier) -> Option<PresentsVerdict> {
    corroborate(a, b).map(|earned_at| PresentsVerdict { earned_at })
}

impl ResolutionTier {
    /// The cap on detection grade for a read sourced at this tier (ADR-069 §A, lower-never-up).
    ///
    /// `Syntactic` caps at `Dread`; `Resolved` and `T3Mir` reach `Presents`. There is no path from a
    /// `Syntactic` source to a `Presents` grade except the single privileged [`corroborate`] door.
    #[must_use]
    pub const fn detection_ceiling(self) -> DetectionGrade {
        match self {
            // T1 syntactic is form-only — it can NEVER on its own earn a content-grade verdict.
            Self::Syntactic => DetectionGrade::Dread,
            // T2 resolved (SCIP) and T3 mir-exact carry content-grade truth.
            Self::Resolved | Self::T3Mir => DetectionGrade::Presents,
        }
    }
}

/// THE ONLY tier-raising door (ADR-069: convergence raises tier ONLY across fresh-independent
/// sources). No other path constructs a higher grade.
///
/// Freshness is ALREADY folded into the tier: a stale capture was demoted to a lower tier at
/// INGESTION time (§5.3 / [`crate::fidelity`]), so a `dread` input here is either genuinely-
/// syntactic OR a demoted-stale source — either way it cannot corroborate up. `corroborate` reads
/// the STORED tier and NEVER re-reads freshness; that is what keeps it a pure, query-time-stable
/// function of the two inputs.
///
/// **The lower-never-corroborates-up law, made a value-level refusal:**
/// - a `Syntactic` (dread) input on EITHER side ⇒ `None` (a form-only source can't lift content).
///   `corroborate(dread, dread)`, `corroborate(dread, resolved)` ⇒ `None`.
/// - two genuinely-resolved-or-higher INDEPENDENT sources ⇒ `Some(max(a, b))` — convergence earns
///   the higher of the two convergent tiers (a resolved+mir agreement reaches mir-exact).
///
/// Independence is the caller's contract (the two answers must come from author-distinct sources —
/// the no-self-witness invariant at the read level); this door cannot re-derive it from the tiers
/// alone, so it assumes the caller passed independent sources and enforces the freshness/floor half.
#[must_use]
pub fn corroborate(a: ResolutionTier, b: ResolutionTier) -> Option<ResolutionTier> {
    // A syntactic (dread-grade) source on either side cannot lift the pair to a content grade.
    if a == ResolutionTier::Syntactic || b == ResolutionTier::Syntactic {
        return None;
    }
    // Both sides are resolved-or-higher and (by the caller's independence contract) fresh-independent:
    // convergence earns the higher of the two tiers. `max` is the monotone, order-independent join.
    Some(a.max(b))
}

// FRESHNESS is itself a tier-attribute (ADR-069 §A.3): a stale capture is a lower tier. The
// index-staleness guard (resolved -> dread when index older than source, ADR-069 Open-seam-1) is an
// always-on demotion in the read path — applied via `fidelity::FidelityWitness`.

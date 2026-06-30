//! STEP 1 — the tier-carrying answer + the read-session handle.

use super::tier::{DetectionGrade, ResolutionTier};

/// An answer that CARRIES its tier and refuses to be read below a floor (ADR-069 tier-honesty).
///
/// A consumer that needs `presents`-grade gets `None`/refusal from a `dread`-grade answer — the tier
/// dishonesty is unconstructible, not merely documented.
#[derive(Debug, Clone)]
pub struct TieredAnswer<T> {
    value: T,
    tier: ResolutionTier,
    grade: DetectionGrade,
}

impl<T> TieredAnswer<T> {
    /// Construct a tier-honest answer. **The grade is DERIVED from the source tier**
    /// ([`ResolutionTier::detection_ceiling`]) — it is NOT a caller-supplied value. This is the
    /// load-bearing tier-cap: a `Syntactic` source yields a `Dread`-grade answer by construction,
    /// so `{ tier: Syntactic, grade: Presents }` is **unconstructible**, not merely rejected at
    /// runtime. The single privileged path to a higher grade than the source earns is
    /// [`super::tier::corroborate`] (see [`Self::corroborated`]).
    #[must_use]
    pub const fn answered(value: T, tier: ResolutionTier) -> Self {
        Self {
            value,
            tier,
            grade: tier.detection_ceiling(),
        }
    }

    /// Construct an answer whose grade was EARNED by corroboration across two fresh-independent
    /// sources — the ONLY door that lifts a grade above what a single source's tier caps it at.
    /// Returns `None` when [`super::tier::corroborate`] refuses (a syntactic input on either side,
    /// or non-convergent tiers), so a corroboration that the law forbids cannot mint an answer.
    #[must_use]
    pub fn corroborated(value: T, a: ResolutionTier, b: ResolutionTier) -> Option<Self> {
        let tier = super::tier::corroborate(a, b)?;
        Some(Self::answered(value, tier))
    }

    /// The tier this answer was sourced at.
    #[must_use]
    pub const fn tier(&self) -> ResolutionTier {
        self.tier
    }

    /// The detection grade this answer carries (capped by its source tier at construction).
    #[must_use]
    pub const fn grade(&self) -> DetectionGrade {
        self.grade
    }

    /// Read the value only if it meets the caller's required grade floor; else refuse.
    ///
    /// A consumer that demands `Presents` gets `None` from a `Dread`-grade answer — the tier
    /// dishonesty (silently serving a form-only answer where content was required) is impossible.
    /// This is the false-quiet defense at the answer level: refusal is explicit (`None` →
    /// route-to-human), never a silent degrade.
    #[must_use]
    pub fn read_at_least(&self, floor: DetectionGrade) -> Option<&T> {
        if self.grade >= floor {
            Some(&self.value)
        } else {
            None
        }
    }
}

/// A held read-session over ONE published revision. While this is alive (`&db` borrow), no
/// maintenance writer can advance the base — detection→field→provenance all observe one version.
///
/// This is the atomic-publish keystone made API-true: it is a borrow of the db, so the borrow
/// checker excludes `&mut db` (publish) for the session's whole lifetime.
pub struct SnapshotHandle<'db> {
    /// The borrowed database the session is pinned to. Public so a read session can be opened with a
    /// struct literal (`SnapshotHandle { db: &db }`) as well as [`SnapshotHandle::open`]; either way
    /// holding the handle holds `&db`, which is what excludes `&mut db` (publish) for the session.
    pub db: &'db crate::db::StromaDb,
}

impl<'db> SnapshotHandle<'db> {
    /// Open a read-session over the currently-published revision. Holding this handle holds `&db`,
    /// so the borrow checker excludes any `&mut db` (publish) for the session's whole lifetime —
    /// detection→field→provenance reads through one `SnapshotHandle` all observe ONE revision, and
    /// a torn read (observing a half-published base) is a COMPILE error, not a runtime lock (the
    /// atomic-publish keystone, ADR-067 §C5 / the gem, §3.5; FREE for the frame's batch cadence).
    #[must_use]
    pub const fn open(db: &'db crate::db::StromaDb) -> Self {
        Self { db }
    }

    /// The database this session reads from (the one published revision the session is pinned to).
    #[must_use]
    pub const fn db(&self) -> &'db crate::db::StromaDb {
        self.db
    }
}

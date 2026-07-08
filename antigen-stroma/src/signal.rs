//! The signal-algebra tier-wall — B2′ acyclicity made a COMPILE-TIME impossibility.
//!
//! ADR-071 B2′ (RATIFIED 2026-07-07) states the afferent organ hierarchy is a DAG: *an organ writes
//! facts at a strictly higher tier than any it reads as afferent input* — read-lower, write-higher, up
//! the DAG; no organ can feed its own causal upstream. ADR-071 Amendment 1 (B4′) adds the Fock-boundary
//! constraint the tree stands inside. Until now that invariant was **process-enforced** (the build
//! wrote phases in an order that happened not to loop); this module makes it **type-enforced**.
//!
//! ## Why a wall, not a convention (the proof)
//!
//! The math-witness (research nb 022 §11, ratified into ADR-068 Am1 / ADR-071 Am1) supplies the proof
//! that this boundary is *algebraic*, not merely stylistic: the **Operator-Boundary Theorem 22.1** —
//! an effect requires a non-additive (non-commutative) accumulation operator iff it is order-dependent.
//! The afferent side is the monotone, order-independent `max`-join ([`crate::read::tier`]); it
//! *provably cannot* compute the order-dependent efferent (suppress/route) decision. So the two wings
//! must live in separate channels, and the afferent one must be acyclic. This module encodes both:
//!
//!   1. **The tier tower** ([`OrganTier`]: [`Base`], [`Up`]) — an *unbounded* (ℕ) organ-output tier,
//!      DISTINCT from the bounded 3-value [`crate::read::tier::ResolutionTier`] (which grades a single
//!      read's *source-resolution*; this grades an organ's *position* in the afferent DAG). Unbounded
//!      because the latent tree is unbounded-in-principle (ADR-007; ADR-071 B4) — bounded only in
//!      practice by the dimensional-Nyquist depth-cap `D ≈ log₂(N)` (ADR-071 Am1, conjecture-tier).
//!
//!   2. **The write door** ([`write_back`]) — mechanically yields the successor tier `Up<R>`, so
//!      writing at the tier you read (or below it) is not a runtime check but an *unconstructible*
//!      state: no function in this module produces an `OrganOutput` at a tier `≤` the tier it read
//!      from. Strictness is *by construction*, not by a trait bound (the peano reflexive-successor +
//!      transitivity relation is not expressible on stable Rust without the heavy typenum `Cmp`
//!      pattern; the successor is the canonical minimal strict-higher tier, and dense tiers — no
//!      skipped levels — are what we want anyway). A latent organ with SEVERAL afferent inputs at
//!      different tiers (write above the *max*) needs a type-level `MaxTier`; that door is deferred
//!      until multi-input latent organs are built — the single-input successor wall proves the B2′
//!      acyclicity core (no organ can feed its own causal upstream).
//!
//!   3. **The channel split** ([`EfferentChannel`]) — the efferent (signed/cancelling) output is a
//!      DISTINCT type the afferent fold cannot consume, so a `Should − Is` inverse can never enter the
//!      afferent (monotone) semiring. The wave-optics MRT Re/Im split (nb 022 §11.4) as a type: "you
//!      may have the inverse, but only in a separate channel whose afferent projection is non-negative."
//!
//! This reuses the [`crate::read::tier::PresentsVerdict`] idiom (a private field + a single privileged
//! door) so an [`OrganOutput`]'s tier is an *unforgeable* witness of where in the DAG it sits.

use core::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}

/// A tier in the afferent organ-output tower (ADR-071 B2′). Unbounded (ℕ), sealed — only this module
/// defines the tower, so no downstream organ can mint a rogue tier or ordering.
pub trait OrganTier: sealed::Sealed {}

/// Tier 0 — an organ whose afferent input is a raw stroma signal (ADR-071 B2, a tier-0 organ).
#[derive(Debug, Clone, Copy)]
pub struct Base;
impl sealed::Sealed for Base {}
impl OrganTier for Base {}

/// The successor tier: an organ that senses from a tier-`N` organ's written-back output writes here.
/// `Up<Up<Base>>` is tier 2, and so on — unbounded.
#[derive(Debug, Clone, Copy)]
pub struct Up<N: OrganTier>(PhantomData<N>);
impl<N: OrganTier> sealed::Sealed for Up<N> {}
impl<N: OrganTier> OrganTier for Up<N> {}

/// An afferent organ's output, tagged at the type level with the tier it was written at.
///
/// The value is PRIVATE: an `OrganOutput` at tier `Tier` can be minted only through [`ingest`] (tier
/// [`Base`]) or [`write_back`] (a strictly-higher tier). So its `Tier` is an *unforgeable* witness of
/// its position in the afferent DAG — a downstream organ cannot fabricate an output claiming a tier it
/// did not legitimately climb to (the [`crate::read::tier::PresentsVerdict`] no-public-constructor
/// idiom, applied to the organ hierarchy).
pub struct OrganOutput<T, Tier: OrganTier> {
    value: T,
    _tier: PhantomData<Tier>,
}

impl<T, Tier: OrganTier> OrganOutput<T, Tier> {
    /// Borrow the carried afferent value (the fact this organ wrote back).
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

/// Ingest a raw stroma signal as a tier-[`Base`] afferent output — the ONLY door to tier 0.
pub const fn ingest<T>(value: T) -> OrganOutput<T, Base> {
    OrganOutput {
        value,
        _tier: PhantomData,
    }
}

/// THE write-back door (ADR-071 B2′ + B5): a latent organ reads a tier-`R` afferent output and writes
/// its own output one tier higher, at the successor tier `Up<R>`.
///
/// The return type is mechanically `Up<R>`, so the acyclicity invariant is a type wall by construction:
///   - there is NO function in this module that yields an `OrganOutput<T, R>` from a `&OrganOutput<T, R>`
///     — so an organ **cannot** write at the tier it reads (coercing the result to `R` is a type error);
///   - nor at any lower tier (there is no downward door at all).
///
/// No organ can write at or below the tier it reads, so the afferent hierarchy is a DAG by construction
/// — no organ can feed its own causal upstream (ADR-071 B2′). The `&OrganOutput` borrow is the
/// afferent-input witness: producing a higher-tier output requires a lower-tier input *in hand*.
pub const fn write_back<T, R: OrganTier>(
    read: &OrganOutput<T, R>,
    produced: T,
) -> OrganOutput<T, Up<R>> {
    // The read is the afferent-input witness; the borrow proves a lower-tier input was in hand. There
    // is no payload to thread here — the tier climb is the whole obligation (B5 write-back-as-facts).
    let _ = read;
    OrganOutput {
        value: produced,
        _tier: PhantomData,
    }
}

/// The EFFERENT channel — a signed / cancelling actuator output (ADR-071 B2′; nb 022 §11.4).
///
/// A DISTINCT type the afferent fold cannot consume: [`write_back`] takes `&OrganOutput`, never an
/// `EfferentChannel`, so a `Should − Is` inverse can never enter the afferent (monotone) semiring. This
/// is the wave-optics MRT Re/Im channel-split as a type — the efferent computation is a separate output
/// target, not a signed value in the afferent accumulator. The afferent-observable projection stays
/// non-negative because the signed value is *elsewhere*, in this channel, by construction.
pub struct EfferentChannel<T> {
    signal: T,
}

impl<T> EfferentChannel<T> {
    /// Wrap an efferent (signed / cancelling) actuator signal into its own channel.
    pub const fn new(signal: T) -> Self {
        Self { signal }
    }

    /// Take the efferent signal — consumed by the actuator wing, NEVER by the afferent fold.
    pub fn into_signal(self) -> T {
        self.signal
    }
}

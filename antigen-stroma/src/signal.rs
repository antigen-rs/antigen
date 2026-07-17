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
//!      different tiers writes above the *max* of the tiers it reads, via the [`TierMax`] type-level
//!      `max` and the multi-input doors [`write_back2`] / [`write_back3`] (nb 028 §5 promoted this from
//!      deferred to load-bearing for overlapping multi-lineage organs). `Up<Max<…>>` is strictly above
//!      every input, so the B2′ acyclicity core holds for multi-lineage organs exactly as the
//!      single-input successor holds it (no organ can feed its own causal upstream).
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

/// Type-level `max` over the [`OrganTier`] tower — the higher of two tiers, computed at compile time.
///
/// A latent organ with SEVERAL afferent inputs at *different* tiers must write strictly above the
/// **max** of the tiers it reads (ADR-071 B2′; nb 028 §5 promoted this from deferred to load-bearing for
/// overlapping multi-lineage organs). `TierMax` is the minimal hand-rolled Peano comparison that yields
/// it — no `typenum` dependency, three non-overlapping impls on the `Base`/`Up` tower:
///
///   - `max(0, b)   = b`        — [`Base`] on the left is dominated by anything.
///   - `max(a+1, 0) = a+1`      — [`Base`] on the right is dominated by any successor.
///   - `max(a+1, b+1) = max(a, b) + 1`  — peel a level off both and recurse.
///
/// The three `Self`/`Rhs` shapes (`Base`/*, `Up`/`Base`, `Up`/`Up`) are pairwise disjoint, so the impls
/// are coherent without specialization. Because `TierMax: OrganTier` and [`OrganTier`] is sealed, no
/// downstream crate can add a rogue `TierMax` impl or fabricate a bogus ordering — the seal propagates.
///
/// Use the [`Max`] alias rather than naming `<A as TierMax<B>>::Output` directly.
pub trait TierMax<Rhs: OrganTier>: OrganTier {
    /// The higher of `Self` and `Rhs`.
    type Output: OrganTier;
}

// max(Base, B) = B — tier 0 loses to (or ties) anything.
impl<B: OrganTier> TierMax<B> for Base {
    type Output = B;
}

// max(Up<A>, Base) = Up<A> — any successor beats tier 0. (Disjoint from the Base impl above: Self is
// `Up<_>`, not `Base`. Disjoint from the Up/Up impl below: Rhs is `Base`, not `Up<_>`.)
impl<A: OrganTier> TierMax<Base> for Up<A> {
    type Output = Self;
}

// max(Up<A>, Up<B>) = Up<max(A, B)> — peel one level off each side and recurse one tier down.
impl<A: OrganTier + TierMax<B>, B: OrganTier> TierMax<Up<B>> for Up<A> {
    type Output = Up<<A as TierMax<B>>::Output>;
}

/// The higher of two [`OrganTier`]s, as a type. `Max<A, B>: OrganTier` always holds, so `Up<Max<A, B>>`
/// is the strictly-higher write tier a two-input organ climbs to. See [`TierMax`].
pub type Max<A, B> = <A as TierMax<B>>::Output;

/// The highest of three [`OrganTier`]s — the associative fold `max(max(A, B), C)`. `Up<Max3<A, B, C>>`
/// is the strictly-higher write tier a three-input organ climbs to. See [`write_back3`].
pub type Max3<A, B, C> = Max<Max<A, B>, C>;

/// THE two-input write-back door (ADR-071 B2′ + B5; nb 028 §5).
///
/// A latent organ reads TWO afferent outputs at tiers `A` and `B` and writes its own output one tier
/// above the *max* of the two, at `Up<Max<A, B>>`.
///
/// The return tier is mechanically `Up<Max<A, B>>` — strictly above *both* inputs (max ≥ each, and `Up`
/// adds one more), so the B2′ acyclicity invariant holds by construction for the multi-input case exactly
/// as [`write_back`] holds it for the single-input case: there is no door in this module that yields an
/// output at a tier `≤` either tier it read, so no multi-lineage organ can feed its own causal upstream.
/// The two `&OrganOutput` borrows are the afferent-input witnesses — producing the higher-tier output
/// requires both lower-tier inputs *in hand*.
///
/// Order-independent: `Max<A, B>` and `Max<B, A>` denote the same tier, so swapping the two reads yields
/// the same output tier (the afferent side is the monotone `max`-join of Operator-Boundary Theorem 22.1).
pub const fn write_back2<T, A, B>(
    read_a: &OrganOutput<T, A>,
    read_b: &OrganOutput<T, B>,
    produced: T,
) -> OrganOutput<T, Up<Max<A, B>>>
where
    A: OrganTier + TierMax<B>,
    B: OrganTier,
{
    // Both reads are the afferent-input witnesses; the borrows prove two lower-tier inputs were in hand.
    // As with `write_back` there is no payload to thread — the tier climb is the whole obligation.
    let _ = read_a;
    let _ = read_b;
    OrganOutput {
        value: produced,
        _tier: PhantomData,
    }
}

/// THE three-input write-back door (ADR-071 B2′ + B5; nb 028 §5).
///
/// A latent organ reads THREE afferent outputs at tiers `A`, `B`, `C` and writes one tier above their
/// max, at `Up<Max3<A, B, C>>`.
///
/// The max fold is associative (`max(max(a, b), c) = max(a, max(b, c))`), so the nesting order of the
/// bound is immaterial to the resulting tier — this is the same monotone join as [`write_back2`], applied
/// once more. Higher arities follow the identical associative pattern (`Up<Max<Max3<A, B, C>, D>>`, …);
/// they are mechanical to add if a consumer needs them (see the module note on N-input generality).
pub const fn write_back3<T, A, B, C>(
    read_a: &OrganOutput<T, A>,
    read_b: &OrganOutput<T, B>,
    read_c: &OrganOutput<T, C>,
    produced: T,
) -> OrganOutput<T, Up<Max3<A, B, C>>>
where
    A: OrganTier + TierMax<B>,
    B: OrganTier,
    C: OrganTier,
    Max<A, B>: TierMax<C>,
{
    let _ = read_a;
    let _ = read_b;
    let _ = read_c;
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

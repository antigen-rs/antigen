//! The CONFIG/OUTPUT split (ADR-067 §F.13) made a type-level boundary.
//!
//! For every primitive, the authored CONFIG (sovereign, D2/D2a, external-witness) is type-distinct
//! from the recomputable OUTPUT (compose, D1/materialized-D1, parity-guardable). Never let one word
//! name both. These newtypes carry the distinction into signatures so a primitive's authored knob
//! and its derived result can never be conflated in an API.

use std::marker::PhantomData;

/// An authored, stable value — the sovereign half. Not recomputable from input; carries an
/// external witness; merged (never re-derived) when it lives on the sovereign side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config<T>(pub T, PhantomData<()>);

/// A derived, recomputable value — the compose half. A pure function of input; parity-guardable;
/// self-heals via salsa invalidation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output<T>(pub T, PhantomData<()>);

impl<T> Config<T> {
    /// Wrap an authored, sovereign value (the stable/CONFIG half — not recomputable from input).
    #[must_use]
    pub const fn new(authored: T) -> Self {
        Self(authored, PhantomData)
    }
}

impl<T> Output<T> {
    /// Wrap a derived, recomputable value (the changing/OUTPUT half — a pure function of input).
    #[must_use]
    pub const fn new(derived: T) -> Self {
        Self(derived, PhantomData)
    }
}

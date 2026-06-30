# antigen-stroma

The **stroma**: a sovereign immune lattice over composed resolution — the read-write-constitute
coordinate frame + base node-set every antigen organ snaps to (ADR-067 / ADR-068 / ADR-069).

> This README is the crate's stranger-facing surface. It ships with the crate when the skeleton is
> lifted into the workspace. (Crate READMEs are a doc-coverage frontier — keep it code-true.)

## What it is

A salsa-clocked relational base of collision-free, cfg-aware, **tier-honest** nodes, read through a
3-axis coordinate frame (SOURCE × PERSPECTIVE × POLARITY). The base is **constituted** (derived from
syntactic + resolved sources), never authored — on the compose base, *write collapses into
constitute*. A lower resolution tier can never corroborate up: a syntactic read literally cannot
construct a `presents`-grade verdict.

## The two epochs (both in this one crate)

- **Frame epoch** — the read CONTRACT + the constituted BASE + query STUBS. (This skeleton.)
- **Engine epoch** — the ascent semiring-datalog closure + 4 semirings + condensation + SCIP
  population. Every `todo!("engine epoch")` is its fill-point.

## The keystone (free from salsa + the borrow checker)

Reads take `&StromaDb`; advancing the base takes `&mut StromaDb`. A torn read — observing a
half-published base — is a **compile error**, not a runtime lock. The atomic-publish invariant falls
out of Rust's borrow rules.

## Build order

See `../BUILD-PLAN.md` — the dependency-ordered sequence (db → read-contract → constitute → query →
write). Building out of order stalls.

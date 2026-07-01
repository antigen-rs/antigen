# antigen-stroma

The **stroma**: a sovereign immune lattice over composed resolution — the read-write-constitute
coordinate frame + base node-set every antigen organ snaps to (ADR-067 / ADR-068 / ADR-069).

## What it is

A salsa-clocked relational base of collision-free, cfg-aware, **tier-honest** nodes, read through a
3-axis coordinate frame (SOURCE × PERSPECTIVE × POLARITY). The base is **constituted** (derived from
syntactic + resolved sources), never authored — on the compose base, *write collapses into
constitute*. A lower resolution tier can never corroborate up: a syntactic read literally cannot
construct a `presents`-grade verdict.

## Two layers (both in this one crate)

- The **read/constitute layer** — the read contract, the constituted base, and the point-wise query
  signatures.
- The **datalog-closure layer** — the ascent semiring-datalog closure, the four semirings,
  SCC-condensation, and SCIP population.

The point-wise query functions (`reachable_from`, `field_at`, `provenance_of`, `blast_from`) panic
when called: they read from the reachability closure but do not compute it. Their signatures are the
frozen query contract every organ compiles against.

## The keystone (free from salsa + the borrow checker)

Reads take `&StromaDb`; advancing the base takes `&mut StromaDb`. A torn read — observing a
half-published base — is a **compile error**, not a runtime lock. The atomic-publish invariant falls
out of Rust's borrow rules.

## The public surface

- [`StromaDb`](src/db.rs) — the salsa database everything attaches to.
- The **read frame** ([`read`](src/read)) — the 3-axis `ReadCoord`, the tier-capped `TieredAnswer`,
  and the point-wise queries.
- [`constitute`](src/constitute) — populate the base from a `SourceWitness` (there is no separate
  `write`; population is always re-derivation from sources).

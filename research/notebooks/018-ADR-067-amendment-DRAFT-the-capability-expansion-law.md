# DRAFT — ADR-067 Amendment: The Capability Expansion Law

**Status:** CONVERGED — post-council ([020](020-adr-067-071-deconstruction-council.md))
+ human ratification rulings ([021](021-capability-expansion-and-the-afferent-organ-hierarchy.md)).
Ready for the `decisions.md` ratification ceremony. Supersede-not-erase amendment to
ADR-067 (The Stroma). Derived from notebook [017](017-the-0.7-organ-council-boyd-ruling.md).

---

## Context

ADR-067 established the stroma as the substrate everything senses on. The surrounding
0.6.x charters carried an implicit **charter posture** — *"enrich, never replace"*:
ship flat floors now, add stroma-enriched ceilings later, keep the flat path beside the
stroma indefinitely.

> **Disambiguation (do not conflate):** this "enrich, never replace" is an *internal
> build-strategy* posture living only in 0.6.x charters + two ATK test files — it is
> **not** the ratified *"enrichment, not gating"* ADR (the user-facing API-layering
> decision: minimum-viable → enriched → richest surfaces). This amendment retires the
> *charter posture* and does **not** touch the user-facing layering ADR — different axis.

One fact retires the charter posture: **a permanent dual substrate violates the stroma's
reason to exist.** By Ashby's Law of Requisite Variety, the regulating organ must hold at
least the variety of everything it regulates — so the stroma must *absorb* all function,
not sit beside a parallel flat path. Carrying two sources of truth is precisely the
dual-source-of-truth drift antigen exists to catch. *Full, or it's dishonest.*

## Decision

Retire the "enrich, never replace" charter posture. Adopt **the Capability Expansion
Law** — one uniform discipline over every capability that **builds**, in the sequence
`[stroma, frame, tier-0 organs, signal-algebra, latent organs]`:

> **BUILD** the capability → **ABSORB** every beneficiary onto it → **PROVE** it
> exhaustively. Each phase is a full expedition-pass. No capability is done leaving its
> beneficiaries on the old path or its surface unproven.

**1 · Expansion-before-consumer (invariant).** The signal a feature needs is built into
the substrate *first*; no consumer is ever built on non-stroma substrate. This fires for
every capability *that builds* (the stroma's own migration is merely its first and
largest instance — its beneficiary-set is "everything that already exists"). *(A pure
JOIN/scheduling event has no beneficiary-absorb; but per [019] the capstone is not a pure
join — it builds latent organs, which build and therefore absorb like anything else.)*

**2 · Refactor via EXPAND-AND-CONTRACT (process).** The new stroma-native implementation
lives **alongside** the old, in separate functions — *both work, old marked
`#[deprecated]`.* **ABSORB** re-points every **internal** caller onto the new path; the
old path remains only for external back-compat. Within a thematic era everything is
**additive + deprecated-but-functional**, so the whole `0.x.y` line stays
cargo-compatible with zero user-facing breaks. This replaces "full-refactor-as-we-go /
drop the old path immediately" — the old path is *deprecated*, not dropped, until §3.

**3 · Breaks are BATCHED at the era boundary (contract).** At a minor bump (`0.8.0`), a
contract pass **removes** that era's deprecated old code and breaks the links. This is
clean because the era's absorbs already wired everything internal onto the new — internal
removal is a no-op; only external users who ignored a whole era of deprecation warnings
feel it, and they get an announced, batched, era-boundary break plus a migration tool.

**4 · Unconditional — do what the codebase needs; no tech debt even if it means user
debt.** There is **no adoption trigger.** Refactor for whatever the codebase's health
requires, always. User impact is *always handled, never dumped* — every refactor
discharges it one of three ways: **(a)** keep the user-facing API the same, **(b)** ship
it as a documented breaking-change version, or **(c)** provide an old→new migration tool.
*(This deletes the draft's "while adoption is negligible" trigger, which both council
lenses flagged as an outcome-in-disguise; the three valves subsume every worry it was
guarding — a published version is superseded, not unpublished; an on-disk format break
ships a migration tool; internal path-dep consumers migrate too.)*

**5 · ABSORB-done = exhaustive-AND-proven (no assumptions).** ABSORB is done when:
- **regression is clean** against existing tests — *except* where behavior was changed on
  purpose, in which case the old test asserted the wrong thing → fix it to the new intent
  + ship docs + a migration tool (§4). *(Tests serve reality; new behavior earns new,
  born-red tests that fail on the old path and pass only on the new one.)*
- the **full JBD cycle** has run on the new component —
  *dream → research → deconstruct → converge → test → build → survey → document* everything
  it enables. There is **no hard end**; the soft signal is *"we've done exhaustive-enough
  work that finding more needs user feedback or another dream wave."*
- **total coverage is proven** — the entire codebase, whole doc suite, all help-line
  output, doc comments, line comments, parameter names, everything — *checked* to confirm
  nothing was missed. **No assumptions. Proven.** (The dogfood ethos turned inward: you
  don't *claim* coverage, you scan for it.)

**Build structure.** Every capability = **BUILD → ABSORB → PROVE, three full
expeditions** over the same charter set (charters re-homed + un-sealed between passes), or
one expedition re-voyaged three times. Each expedition-end **tags + ships a release**
(additive `0.7.x`); the **`prove`-tagged release is the one to rely on.**

## Consequences

- **Single substrate** (Ashby-complete): no permanent two-substrate seam.
- **Less complexity, not more**: deletes the enrich/defer machinery and the dual source
  of truth.
- **Cheap reversibility**: because the new lives alongside the deprecated old, a bad
  refactor is dropped (old still works) rather than emergency-reverted.
- **Semver stays additive within an era**; the minor steps only at a deliberate
  era-boundary contract (see [021] §C and [019]).
- Each capability's definition-of-done is the exhaustive-proven ABSORB above.

## Supersedes / overrides

This **ratifies an invariant** (single-substrate; expansion-before-consumer) that
**overrides** the non-ADR *"enrich, never replace"* charter posture (0.6.x charters +
ATK files), rebuilding those floors substrate-native where still useful. It does **not**
touch the ratified *"enrichment, not gating"* user-facing layering ADR. *(ADRs supersede
ADRs; they override non-ADR postures by being higher authority — not by "superseding a
charter.")*

## Open questions — resolved at convergence

- **Trigger metric?** — *Resolved:* there is no trigger; §4's unconditional + three-valve
  user-impact rule replaces it.
- **Revert vs semver?** — *Resolved:* expand keeps the old path alongside the new; the
  era-boundary contract removes it; a breaking release is a *new* semver version
  (supersede), never an un-publish.

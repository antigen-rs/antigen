# Antigen — Roadmap

> User-facing trajectory document. What's shipped, what's planned, what's
> aspirational. Substrate-grounded confidence intervals; no firm calendar
> dates beyond what's actually committed.

**This is the adopter-facing roadmap.** Internal sweep planning lives in
[`sweeps/`](../sweeps/); ratified architecture lives in
[`docs/decisions.md`](decisions.md); expedition substrate lives in
[`docs/expedition/`](expedition/).

---

## Shipped (v0.1.0-rc.1)

The core vocabulary, scan + audit tooling, and discipline substrate are
all live:

- **Five macros**: `#[antigen]`, `#[presents]`, `#[immune]`,
  `#[descended_from]`, `#[antigen_tolerance]`
- **`cargo antigen scan`** with workspace-wide scanning, item-identity
  matching (W3), fingerprint detection, tolerance recognition, and
  orphaned-tolerance reporting
- **`cargo antigen audit`** with the `WitnessTier` gradient
  (Reachability / Execution / FormalProof / ExternalUnvalidated /
  Missing) — tier-honest reporting per ADR-005 Amendment 3
- **Fingerprint grammar v1** — seven item-level operators (`item`,
  `name`, `variants`, `has_method`, `attr_present`, `doc_contains`,
  `body_contains_macro`) plus composition (`all_of`, `any_of`, `not`);
  proc_macro2 canonicalization per ADR-010 Amendment 5
- **Phantom-type witness recognition** (ADR-013) — `Witnessed<T,W>`,
  `typewit::TypeEq`, hand-rolled `PhantomData<T>` shapes recognized at
  FormalProof tier with explicit human-readable output
- **Cross-crate identity infrastructure** — `canonical_path` at
  `name@version` granularity (ADR-017); cross-crate `#[descended_from]`
  propagation and registry source-walking land in A3 (v0.2)
- **Documentation**: tutorial, fingerprint grammar reference,
  usage-patterns cookbook, where-to-look conventions, troubleshooting,
  origin narrative, comprehensive ADR substrate

See [`CHANGELOG.md`](../CHANGELOG.md) for the full v0.1.0-rc.1 manifest.

---

## Planned for v0.2

Items committed by structural necessity (ADR-007 anti-YAGNI:
structurally-guaranteed-need). Each lands when its substrate matures;
ordering may shift.

- **Body-level fingerprint operators** via ast-grep subprocess
  (per ADR-015). Enables fingerprints that match against function
  bodies, not just item-level signatures. Closes the recall-tuned-filter
  gap for failure-classes whose structural pattern lives in
  implementation, not declaration.
- **`cargo antigen new`** — scaffold a new antigen declaration with
  guided prompts. Tooling for first-time-adopter ergonomics; reduces
  the friction of authoring well-formed fingerprints from scratch.
- **`cargo antigen vaccinate`** — apply known immunity pattern across
  a structural family. Bulk-applies `#[presents]` and `#[immune]`
  annotations to sites matching a known antigen's fingerprint, with
  human review of the proposed change-set.
- **Engine-canonicalization for operators beyond `has_method`** — the
  ADR-010 Amendment 5 pre-tokenization pattern extends to other
  string-comparison operators where tokenization asymmetries surface
  in practice (recognition-not-design: lands when substrate-grounded).

---

## Planned for v0.3+

Items in active substrate-accrual; ratified or in-flight ADRs commit
the direction even where the implementation lands later.

- **Sweep A4: composition rules + witness-type pluralism completion** —
  Eiffel-style D1/D2/D4 composition invariants; full
  kani/prusti/verus/creusot/flux witness recognition with harness
  invocation through the audit pipeline.
- **Sweep A5: `antigen-stdlib` v0.1** — ecosystem-shared failure-class
  memory library. 10-20 stdlib antigens covering all 8 first-principles
  failure classes; antigens importable via dev-dependency or feature
  flag; ratified contribution model (recognition-grounded, not
  spec-grounded — see [A5 governance encounter in
  deferred-substrate.md](expedition/deferred-substrate.md)).
- **Sweep A6: rust-analyzer plugin / IDE integration** — real-time
  fingerprint match surfacing as you type; inline annotations for
  presentations + immunity status; recognition at the moment of
  authorship rather than build time. Maps to Component 7 (real-time /
  CI feedback) of multi-component immunity.

---

## Aspirational (post-v1.0; substrate-watch)

Substantive architectural ambitions held below the ADR-006 threshold
for ratification. Each lands when its substrate-grounded trigger
surfaces.

### Multi-language extension

Antigen-the-vocabulary is language-agnostic in principle. The five
primitives (declare/present/immune/descended_from/tolerance) describe a
structural architecture of failure-class memory that doesn't depend on
Rust.

Per-language implementations are components in the multi-component
framing (see [`expedition/multi-component-immunity.md`](expedition/multi-component-immunity.md)):

- **Python**: ast-module or tree-sitter-based fingerprint engine;
  pip-installable tool with `python -m antigen scan` invocation
- **JavaScript / TypeScript**: Babel or tree-sitter-based fingerprint
  engine; npm-installable tool
- **Framework-specific**: React-tier, Django-tier, Rails-tier
  antigens — each operating on the framework's metaprogramming surface

Failure-classes generalize across languages at the structural-shape
level. "Drop impl must not panic" (Rust) is structurally cognate to
"context manager `__exit__` must not raise" (Python), "destructor must
not throw" (C++), and similar patterns in other languages. The
taxonomy operates above any specific language; adding a fail-class can
inform all language implementations.

**No version commitment**; multi-language work begins when Rust
substrate is mature enough that splitting attention is productive
rather than dilutive.

### Cross-tier antigen surfaces

The architectural class recurses across abstraction tiers, not just
within codebases. Future antigen surfaces could operate at:

- **Organization-tier**: decision-failure-classes (charter without
  rationale; spec-grounded when recognition-grounded is correct)
- **Team-tier**: coordination-failure-classes (substrate-currency drift
  across routing; tier-honesty drift at handoff)
- **Process-tier**: discipline-failure-classes (premature closure;
  recognition-not-design violations; framing-without-substrate)
- **AI-agent-tier**: context-failure-classes (pre-compaction summary
  trusted as current state; memory-based hallucination)

At each tier, mechanism differs; the compositional property (structural
failure-class memory) recurses. See
[`expedition/antigen-applied-to-antigen.md`](expedition/antigen-applied-to-antigen.md)
for the substrate exploring this recursion.

**No version commitment**; cross-tier surfaces develop alongside per-
language work as substrate accrues.

### Ecosystem flywheel

- **Cross-organization antigen registries** — teams within larger
  organizations share antigens via internal registries without
  publishing to crates.io
- **Antigen declarations in CVE / RFC / security-advisory databases** —
  failure-classes from external security substrate become structural
  memory in your codebase
- **Multi-maintainer attestation for stdlib antigens** — threshold
  signatures, signed declarations, distributed trust models for
  ecosystem-scale failure-class memory

These are post-A5 governance territory; substrate accrues as antigen-
stdlib adoption grows.

---

## Adoption pathways

Antigen meets you where you are. The adoption gradient is continuous;
there is no cliff:

**Floor — antigen-as-linter**
Drop the cargo subcommand into your toolchain. Run `cargo antigen scan`.
Get structural failure-class memory of whatever antigens are declared
in your dependencies. Zero buy-in beyond installation.

**Pragmatic dev mode — declare your own**
Write project-specific antigens for failure-classes you've encountered.
The vocabulary makes lessons structural without requiring full discipline
overhead.

**Integrated team mode — witness pluralism**
Link witnesses to your existing test suite, proptest harnesses, formal-
verification tools, and clippy lints. Audit reports tier honestly across
the full witness spectrum.

**Bridged-knowledge organization**
Attach references (PRs, ADRs, CVEs, post-mortems) to antigens. Failure-
class memory becomes a knowledge-graph node bridging code to lived
context.

**Lineage-aware long-lived codebase**
Manage failure-class taxonomy via `#[descended_from]`. Track immunity
history across versions. Treat version-boundary transitions as
recognition opportunities.

**Ecosystem participant**
Use antigens from dependencies. Contribute candidate stdlib antigens.
Participate in cross-organization failure-class memory sharing.

Each tier multiplies leverage without requiring the others. See
[`expedition/multi-component-immunity.md`](expedition/multi-component-immunity.md)
for the deeper architectural framing.

---

## How decisions get made

This roadmap is recognition-grounded, not spec-grounded:

- **Ratified ADRs** (in [`docs/decisions.md`](decisions.md)) commit the
  architectural direction
- **Sweeps** (in [`sweeps/`](../sweeps/)) execute toward those
  commitments
- **Expedition substrate** (in [`docs/expedition/`](expedition/))
  matures ahead of ratification; not all expedition substrate ratifies

Per ADR-006 (recognition-not-design): new antigens, new witness types,
new composition rules land when they recognize existing structure in
the substrate — not when they extend the design speculatively. The
ADR-006 threshold is three independent substrate-grounded instances.

Per ADR-007 (anti-YAGNI structurally-guaranteed need): when the
project's structural commitments guarantee a feature will be needed,
it gets built upfront. Items in "Planned for v0.2" and "Planned for
v0.3+" are mostly in this category.

Per the [encounters discipline](expedition/encounters-proposal.md)
(in flight): observations that aren't yet ratified-eligible get
formally registered so subsequent recurrences recognize each other
rather than getting treated as fresh first-recognitions. The encounters
discipline is itself in this category — it's pre-ratification substrate
about how the project handles pre-ratification substrate.

---

## Showcase by building

The substrate produced by antigen's own development is evidence of
value. Not "we built a tool; here are claims about what it does." More:
"we built a tool by using the tool; the substrate's quality is the
proof."

The recursion is structural: the discipline that produced antigen is
the discipline antigen formalizes. Six instances of "antigen applied to
antigen" surfaced in a single sweep (A3.5). See
[`expedition/antigen-applied-to-antigen.md`](expedition/antigen-applied-to-antigen.md)
for the framing.

When you adopt antigen, you join the same recursion at a different
scale. The tool will help you develop the discipline by demanding it,
and the discipline will help you use the tool by recognizing what to
declare. The pathway from "I installed cargo-antigen" to "structural
failure-class memory is operational in our practice" is the same
co-evolutionary pathway that produced the tool itself.

---

## Questions

- *Why isn't there a calendar in this roadmap?* Per Tekgy's no-rush
  framing — release-readiness drives timing, not calendar dates.
  Substrate maturity is the actual signal. Versions ship when substrate
  is ready; sweeps close when their scope-locks are satisfied. The
  trajectory is real; the dates are not.

- *How do I know when "ready" is?* Recognition, not specification. The
  v0.1.0-rc.1 release substrate is *substantive demonstration* of
  capability; v0.1.0 final ships after rc adopters surface real-world
  friction; v0.2 ships when body-level operators + ergonomic tools
  mature.

- *Where do I follow progress?* [`sweeps/`](../sweeps/) tracks current
  sweep status; [`CHANGELOG.md`](../CHANGELOG.md) tracks shipped
  substrate; expedition substrate shows what's maturing pre-ratification.

- *Can I contribute?* Yes — see
  [`CONTRIBUTING.md`](../CONTRIBUTING.md). The most valuable
  contributions right now are real-world failure-class proposals
  (Rust failures that fit or refine the taxonomy), witness type
  integration refinements, and adoption feedback once v0.1.0 lands.

---

*Roadmap authored 2026-05-12 as Phase 4 deliverable of Sweep A3.5
(Onboarding). Subject to revision as substrate matures. The trajectory
is real; the destination is recursive.*

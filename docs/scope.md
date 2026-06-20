# Antigen — Scope

> This document articulates the **architectural class** antigen belongs to and
> the **adoption strategy** it pursues. The [README](../README.md) is the GitHub
> front door; [`origin.md`](origin.md) is the founding-incident narrative;
> [`roadmap.md`](roadmap.md) tracks what has shipped and where it's going.

---

## The category being defined

Software engineering has structurally-safe primitives for several memory
classes:

- **Memory safety** (RAM/heap/stack — Rust's headline)
- **Type safety** (mismatches caught at compile time)
- **Thread safety** (data races prevented)

It does not have a structurally-safe primitive for **domain-knowledge memory** —
the lessons learned about WHY classes of code fail. That memory has lived
in carriers that drift:

- Developer minds (lost when developers leave or context cycles)
- AI session context windows (lost the moment compaction triggers)
- Commit messages (decay into archive nobody reads)
- Code review comments (lost when the PR closes)
- Docstrings (drift from the code they describe)
- Post-mortems (vanish when platforms change)
- Mentorship (lossy, slow, single-threaded transmission)

When that memory drifts, the same failure-class re-emerges in slightly different
costumes across projects, across teams, across AI agent sessions. Lessons
re-learned. Bugs re-shipped. Engineering effort wasted on rediscovery.

**Antigen instantiates structural domain-knowledge memory as a first-class
artifact in the type system.** Failure-class declarations persist past the bugs
that motivated them. Inheritance propagates immunity. Tooling validates
witnesses. Drift is detected at scan time.

This isn't "another linter" or "more tests" or "TDD with extra steps." It's a
new category of structural verification — the way testing-as-practice was a new
category before it became standard.

---

## Why the architecture matters beyond the Rust ecosystem

Four independent fields have been converging on structural-memory-with-
recognition-and-inheritance as an architectural pattern:

1. **Biological immunology** — vertebrate immune systems have been iterating on
   this design for ~500 million years. The vocabulary is rich (antigens,
   antibodies, B-cell memory, MHC presentation, T-cell receptors, peripheral
   tolerance, vaccination, affinity maturation, innate vs adaptive immunity);
   the architecture is empirically refined.

2. **Programming-language theory** (Hoare 1969 → Eiffel 1992 → Liquid Haskell →
   Flux 2024). Named structural properties attached to program artifacts and
   propagated through composition is a 50+ year research thread. Antigen is
   the first instantiation that's adoption-oriented for a mainstream Rust
   ecosystem.

3. **Pattern-recognition from a different starting point.** Independent of the
   biology, reasoning about signal-lossy boundaries and naming-makes-checkable
   arrives at antigen's foundational insights — that a failure-class only
   becomes checkable once it is named, and that unnamed structure recurs
   silently. The convergence here isn't biology-driven; it's
   recognition-architecture-driven from first principles.

4. **ML graph-memory research.** Independent ML-research convergence on
   structural-memory-with-relationships-first-class as agent-memory
   architecture. Named differently; same primitive.

Four windows on the same architecture means the project isn't building "a useful
Rust tool"; it's instantiating a **scale-invariant recognition architecture** in
Rust as the first domain to get a fully-articulated, adoption-ready instantiation.

The architecture is governed by a no-fixed-point property: the failure mode of
recognition systems (silent recurrence of structural variants) manifests at
every operational scale — from cross-team coordination down to within-single-
drafting-pause. The pattern is fractal. Antigen's contribution is making the
recognition mechanism that addresses it explicit, structural, and ergonomically
adoptable.

---

## Adoption-first: this is the destination

Papers articulate the category for the field. Tools enable adoption. **Wide
adoption as standard practice is the project's mission**; papers and academic
contribution serve adoption, not vice versa.

The adoption flywheel:

1. **The core**: macros, scan, audit, the fingerprint grammar, the WitnessTier
   gradient, phantom-type recognition, antigen-tolerance.
2. **A bundled stdlib of antigens**: ready-made antigens covering the
   first-principles failure classes. Add the stdlib as a dev-dependency and
   gain immunity to common Rust failure-classes without authoring antigens
   yourself — the way clippy ships default lints.
3. **Project-specific antigens accumulate**: teams declare antigens for their
   domain's failure-classes. Each PR that fixes a bug can include an antigen
   declaring the failure-class. The fix doesn't just patch this codebase;
   it adds to the global library.
4. **Cross-crate propagation**: antigens declared in one crate apply to
   consumers. A crate's seed antigens are visible to projects depending on it;
   the bundled stdlib's antigens are visible to everyone who adopts it.
5. **IDE integration**: a rust-analyzer plugin surfaces fingerprint matches
   inline as you type. Real-time predictive flagging — like a type checker for
   failure-class shapes.
6. **Community contribution**: PRs to the bundled stdlib become contributions to
   collective Rust-ecosystem memory. The global failure-class library grows
   with the community's accumulated experience.

**Low friction OOTB. Comprehensive when worked.** Like clippy: install, get
default value immediately. Customize, get more. No mandatory ADR registry, no
required project ceremony, no friction-tax for adoption.

---

## The current shape (what exists)

The concrete macro vocabulary and `cargo antigen` surface are documented in
[`macros.md`](macros.md) and [`output-formats.md`](output-formats.md); the
live ship-state and trajectory are tracked in [`roadmap.md`](roadmap.md). The
stable core:

**The marker vocabulary** — `#[antigen]` declares a failure-class with a
structural fingerprint; `#[presents]` marks a site as exposed to a known class;
`#[defended_by]` records the witness that defends a presenting site;
`#[descended_from]` propagates markers through inheritance; `#[antigen_tolerance]`
opts a legitimate fingerprint match out of flagging.

**The `cargo antigen` subcommand** — `scan` finds unaddressed presentations and
catalog-match candidates; `audit` verifies witness resolution and tier validation;
plus the family-specific drivers (`attest`, `tolerate`, `verify`, `vcs`,
`mucosal-map`, `fingerprint`).

**The fingerprint grammar** — item-kind, name-glob, variant-count,
method-presence, attribute-presence, and docstring-substring matchers, composed
with `all_of` / `any_of` / `not`, plus the body-level matchers (`body_calls`,
trait-impl context) that let a fingerprint reach inside a function.

**Witness pluralism** — tests, property tests, phantom-type witnesses,
formal-verification harnesses (kani / prusti / verus / creusot), and custom lints
are all recognized as witness families; `audit` validates each at the appropriate
tier.

**Architectural commitments** (see [`decisions.md`](decisions.md) for the
ratified ADRs that govern these):
- Structural memory not documentary (ADR-001)
- Compose, don't compete (ADR-002)
- Biological metaphor is load-bearing (ADR-003)
- Implicit-to-explicit elevation (ADR-004)
- Sub-clause F at every trust boundary (ADR-005)
- Recognition, not design (ADR-006)
- Anti-YAGNI: structurally-guaranteed need (ADR-007)
- Named-observer terminus + adoption gradient (ADR-008, ADR-009)
- Fingerprint grammar with a body-level path (ADR-010 and amendments)
- Phantom-type witness recognition + WitnessTier gradient (ADR-013)
- Antigen-tolerance opt-out (ADR-011)
- Temporal recognition surface — verified-at, stale-after, evidence (ADR-016)
- Observe-don't-declare: `#[presents]` / `#[defended_by]` supersede the
  deprecated `#[immune]` (ADR-029)

---

## The full shape (where it's going)

The biological immune system carries many more primitives than the core macro
vocabulary instantiates. As adoption surfaces specific needs, the project
instantiates additional primitives — recognizing the shape from real instances
rather than designing it speculatively (ADR-006).

### Comprehensive immune-system primitive map

> **Scale note**: the table below is a **seed set for unbounded ecosystem
> growth**, not a bounded enumeration of "the primitives antigen will
> eventually have." Vertebrate immune systems carry on the order of 10^11
> distinct antibody specificities; the antigen ecosystem's eventual scale —
> across stdlib + community + domain-specific antigen libraries +
> per-project antigens — is comparably unbounded. Each named primitive
> below is a category that could spawn many specific instances. The
> complete list is the open ecosystem itself, not this table.

| Biology | Potential Rust ecosystem instantiation |
|---|---|
| Macrophages (phagocytosis: consume + present) | Code-consumer tools that walk macro outputs / build.rs generation / external dependencies and present what's inside as antigen-knowable substrate |
| Dendritic cells (bridge innate to adaptive) | Audit pass that takes scan-detected patterns and routes them to specific defenses with provenance |
| Complement system (tag for destruction) | Refactor-suggestion tools that mark code presenting antigens with structural fix recommendations |
| Affinity maturation (B-cells refine antibodies over time) | The Learning-Core: antigen *generates* candidate fingerprints by anti-unifying clustered marked sites, and refines them against the corpus — fingerprints that improve as community contributions arrive |
| NK cells (recognize abnormal without specific antigen) | **Operational**: the fingerprint synthesis pass fires on code matching a failure-class shape even without a `#[presents]` marker — structurally-abnormal code flagged without a named per-site declaration. Full outlier detection (flagging code unusual against the whole corpus) is later territory. |
| Cytokines (signaling propagation) | Cross-crate antigen propagation signals; `#[descended_from]` is one shape; richer propagation is forward work |
| Innate vs adaptive immunity layers | **Operational**: passive surface (fingerprint scan) + active surface (explicit markers) |
| Inoculation (small controlled exposure) | Test harnesses that deliberately apply antigen patterns to verify witness behavior |
| Plasma cells (short-lived antibody factories) | Witness templates that generate defenses from patterns; code generation for witnesses |
| MHC Class I vs II (intracellular vs extracellular antigen) | "Internal state" antigens vs "external contract" antigens — different visibility surfaces |
| Memory cells (long-term persistence) | **Operational**: `#[antigen]` declarations |
| Regulatory T-cells (prevent overreaction) | `#[antigen_tolerance]` is the primitive; future expansion: auto-tolerance learning when patterns reveal themselves as systematically over-flagged |
| Pathogen Recognition Receptors (PRRs) | **Operational**: structural-pattern matchers in `cargo antigen scan`'s fingerprint engine |

These aren't metaphors *of* what we're building — they're literal architectural
patterns the immune system has already solved. We are not analogizing; we're
copying structural answers across substrates. Biology has been iterating on
recognition-with-memory-and-inheritance for hundreds of millions of years. The
project gets to build on that work rather than re-derive it.

Each potential instantiation in the table above is a future tool surface,
recognized as adoption produces real instances per ADR-006. Recognition, not
design.

The destination is **wide adoption as standard practice in Rust development**.
The core macros and `cargo antigen` start the flywheel; cross-crate propagation
and IDE integration accelerate it; the deeper immune-system primitives deepen
the architecture as the community surfaces what it needs. See
[`roadmap.md`](roadmap.md) for the tracked trajectory.

---

## AI dev tooling implications

The AI industry currently spends enormous resources trying to teach models to
avoid bug patterns through fine-tuning, RLHF, custom harnesses, training-data
curation, and adjacent approaches. Each has structural problems:

- Fine-tuning doesn't propagate to new models without re-training
- Implicit weights can't be inspected, audited, or version-controlled
- Documentation/comments meant to teach models rot
- Each new model trains on stale data
- The lessons live in opacity, not substrate

**Antigen offers a structurally different solution: lessons live in code, not
in weights.** Inspectable, version-controlled, propagable to ANY model or human
reading the codebase. A new LLM picks up a Rust workspace and reads `#[antigen]`
declarations like any other code. It KNOWS about the failure-classes because
they're substrate, not training-data-residue.

This makes antigen relevant to multiple audiences simultaneously:

- **Rust developers**: gain failure-class memory across teams, time, and
  refactors. Onboarding new developers becomes easier (lessons are in code,
  not in tribal knowledge). Refactoring is safer (defenses are
  preserved structurally).
- **AI-assisted-coding teams**: the AI agent has access to the codebase's
  failure-class memory regardless of which model is running. No training-data
  staleness; no fine-tuning required; no harness rot. The lessons travel
  structurally with the code.
- **Open-source maintainers**: contribute antigens upstream alongside bug
  fixes. The maintenance burden is offloaded to the structural carrier;
  contributors don't need to remember tribal knowledge.
- **Academic / research community**: structural memory with measurable
  empirical properties (biology-as-search-heuristic precision; colonization
  ratio over authoring envelope; four-window convergence; scale-invariance
  of no-fixed-point property) is a contribution to programming-language
  theory and software-engineering foundations.

The framing isn't "antigen helps Rust developers" — though it does. It's
**"antigen is a structural alternative to the AI industry's current approach
to embedding failure-class knowledge in software development."** That's a
different problem class entirely, and it's one antigen is positioned to be
the first ergonomically-adoptable instantiation of.

---

## Multiple-paper trajectory

The architecture supports several distinct contributions to the literature, each
landing as its substrate matures:

- **Tool paper**: empirical defense of antigen as a working Rust ecosystem tool —
  biology-as-search-heuristic precision, colonization ratio, adversarial
  confirmation rate, adopter signal.
- **Foundational paper**: the four-window convergence + scale-invariance +
  no-fixed-point property as the architectural framing — positioning antigen as
  the first instantiation of a domain-general scale-invariant recognition
  architecture.
- **Methodology paper**: the substrate-over-memory, recognition-not-design,
  and rationale-as-required-field disciplines as a software-engineering practice
  for AI-assisted development.
- **AI-dev-tooling paper**: structural failure-class memory in code as an
  alternative to fine-tuning weights for embedding domain knowledge in AI
  development workflows.

Each lands when its substrate matures; the project's daily work produces
substrate for all of them simultaneously.

---

## The case study (grounded)

Apply everything above to the project's founding incident:

**The incident**: a computational-mathematics project's `DeterminismClass` shipped with `meet = min` because
the polarity-inversion-when-strongest-first pattern wasn't recognized. Caught
and fixed (commit references in [`origin.md`](origin.md)). The lesson was
learned: "when class enum has strongest-first discriminants, lattice meet must
be max." Stored in: GAP-BIT-EXACT-1 issue, fix commit message, the docstring
on `DeterminismClass::meet()`, the team's collective memory.

Months later, drafting `CommutativityClass` — structurally identical pattern.
The lesson didn't transfer. `meet = min` was specified again. Almost shipped a
second time. Only caught because a multi-agent development team's discipline forced
re-derivation from worked examples. In a less-disciplined team or a single-
agent fresh-context session, it would have shipped.

**With antigen + everything in this scope document**:

When the original DeterminismClass bug was fixed, the PR includes one antigen:

```rust,ignore
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = "item: enum, has_method('meet', '(self, Self) -> Self'), \
                   variants: 3..=8, attr_present('repr(u8)'), doc_contains('strength')",
    references = ["GAP-BIT-EXACT-1", "DEC-030 §1.1", "commit bb918d2"],
    summary = "When class enum represents strength-of-claim with strongest-first \
               discriminants, lattice meet must use max not min; the polarity inverts.",
)]
pub struct PolarityInvertedClassMeet;
```

Months later, `CommutativityClass` lands. Within milliseconds:

- **Pattern-hunting proactive scan** flags it as matching the fingerprint
- **Real-time IDE annotation** (as IDE integration lands) surfaces the warning
  AS the developer types the meet method
- **Library-wide propagation** means every Rust project that depends on a crate
  carrying this antigen inherits the lesson without anyone re-learning it
- **Provenance is verifiable**: a future reader traces back through `references`
  to GAP-BIT-EXACT-1 + DEC-030 + the original fix commit
- **AI agent context**: whatever Claude / GPT / future-model picks up the work,
  whatever fine-tuning generation, whatever training data — the lesson is in
  substrate. Not in weights, not in docs, not in mentorship: in code.

The same case applies to any failure-class. Polarity inversion is just one
example. Panicking-in-Drop, Lock-Order-Inversion, Use-After-Move-Conceptually-
Equivalent, Async-In-Sync-Context, Premature-Specialization-In-Generic-Bounds —
each gets a structural carrier that propagates regardless of who reads the code.

That's the project's contribution. Not "another Rust tool." Not "more tests."
**Domain-knowledge-memory-safety as the fourth structural property of secure-
by-default Rust development.**

---

*This document articulates the project's scope — the architectural class antigen
belongs to and the adoption strategy it pursues. The [README](../README.md) is
the welcoming front door; [`origin.md`](origin.md) is the founding-incident
narrative; [`roadmap.md`](roadmap.md) tracks what has shipped and where it's
going.*

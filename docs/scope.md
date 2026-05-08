# Antigen — Scope

> The comprehensive vision: what antigen does today, what it is becoming, why
> the architecture matters beyond the Rust ecosystem, and how the project's
> work-streams ladder toward standard-practice adoption.
>
> This document is **scope at depth**. The README is the GitHub front door;
> [`origin.md`](origin.md) is the founding-incident narrative; this document
> articulates the project's full ambition.

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

3. **Pre-project AI gardening** (March-April 2026 entries by past instances of
   the project's AI contributors). Garden entries on signal-lossy boundaries
   and naming-makes-checkable predicted antigen's foundational insights before
   the project existed. The convergence wasn't biology-driven; it was pattern-
   recognition-driven from a different starting point.

4. **2026 ML graph-memory research** (arxiv 2602.05665 and adjacent work).
   Independent ML-research convergence on structural-memory-with-relationships-
   first-class as agent-memory architecture. Named differently; same primitive.

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

1. **v0.1.0 ships the core**: macros, scan, audit, fingerprint grammar v1,
   WitnessTier gradient, phantom-type recognition, antigen-tolerance.
2. **antigen-stdlib v0.1 (Sweep A5)**: 10-20 ready-made antigens covering the
   8 first-principles failure classes. Add antigen-stdlib as a dev-dependency
   and gain immunity to common Rust failure-classes without authoring antigens
   yourself — the way clippy ships default lints.
3. **Project-specific antigens accumulate**: teams declare antigens for their
   domain's failure-classes. Each PR that fixes a bug can include an antigen
   declaring the failure-class. The fix doesn't just patch this codebase;
   it adds to the global library.
4. **Cross-crate propagation (Sweep A3)**: antigens declared in one crate
   apply to consumers. Tambear's seed antigens are visible to projects
   depending on tambear; antigen-stdlib's antigens are visible to everyone
   adopting antigen-stdlib.
5. **IDE integration (Sweep A6)**: rust-analyzer plugin surfaces fingerprint
   matches inline as you type. Real-time predictive flagging — like a type
   checker for failure-class shapes.
6. **Community contribution**: PRs to antigen-stdlib become contributions to
   collective Rust-ecosystem memory. The global failure-class library grows
   with the community's accumulated experience.

**Low friction OOTB. Comprehensive when worked.** Like clippy: install, get
default value immediately. Customize, get more. No mandatory ADR registry, no
required project ceremony, no friction-tax for adoption.

---

## The current shape (what exists)

As of v0.1.0 substrate (release imminent — see [`sweeps/A2-core-macros/`](../sweeps/A2-core-macros/)):

**Five core macros**:
- `#[antigen(name, fingerprint, family?, summary?, references?, adr?)]` — declare a failure-class
- `#[presents(antigen)]` — mark code as vulnerable
- `#[immune(antigen, witness = ...)]` — declare immunity with a witness
- `#[descended_from(...)]` — propagate markers through inheritance
- `#[antigen_tolerance(antigen, rationale, until?, see?)]` — opt-out for legitimate fingerprint matches

**`cargo antigen` subcommand**:
- `scan` — find unaddressed presentations + 5-state interaction matrix output
- `audit` — verify immunity claims at the appropriate WitnessTier (Reachability /
  Execution / FormalProof; BehavioralAlignment reserved)
- `vaccinate` (planned, A5) — apply immunity across a structural family
- `new` (planned, A5) — scaffold a new antigen declaration

**Fingerprint grammar v1** (six item-level operators + composition):
- `item: <kind>` — match struct/enum/trait/fn/impl
- `name: matches(<glob>)` — type-name patterns
- `variants: <range>` — enum variant count constraints
- `has_method(<name>, <signature>)` — method-presence checks
- `attr_present(<path>)` — attribute-presence checks
- `doc_contains(<substring>)` — docstring substring matching
- `all_of` / `any_of` / `not` — composition operators

**Witness pluralism** (5+ families recognized):
- Tests (`#[test]`)
- Property tests (`proptest!`, `quickcheck`)
- Phantom-type witnesses (`Witnessed<T,W>`, `typewit::TypeEq`, hand-rolled `PhantomData<T>`)
- Formal-verification harnesses (`kani::proof`, `prusti::trusted`, `verus::proof`, `creusot::ensures`)
- Custom lints (clippy, dylint)

**Architectural commitments** (see [`docs/decisions.md`](decisions.md) for ratified ADRs):
- Structural memory not documentary (ADR-001)
- Compose, don't compete (ADR-002)
- Biological metaphor is load-bearing (ADR-003)
- Implicit-to-explicit elevation (ADR-004)
- Sub-clause F at every trust boundary (ADR-005, with Amendment 2 for
  rationale-as-required-field, Amendment 3 for audit-tier-honesty)
- Recognition, not design (ADR-006)
- Anti-YAGNI: structurally-guaranteed need (ADR-007)
- Named-observer terminus + adoption gradient (ADR-008, ADR-009)
- Fingerprint grammar v1 + body-level v2 path + ast-grep subprocess engine
  (ADR-010 + amendments + ADR-015)
- Phantom-type witness recognition + WitnessTier gradient (ADR-013)
- Antigen-tolerance opt-out (ADR-011)
- Antigen-generates for proc-macro authors (ADR-014)
- Temporal recognition surface — verified_at, stale_after, evidence (ADR-016)

---

## The full shape (where it's going)

The biological immune system has many more primitives than the five macros
v0.1.0 ships. As adoption surfaces specific needs, the project will instantiate
additional primitives. Some that are forward substrate now, awaiting
recognition (per ADR-006: real instances in adoption surface them; the project
recognizes rather than designs):

### Comprehensive immune-system primitive map (forward substrate)

| Biology | Potential Rust ecosystem instantiation |
|---|---|
| Macrophages (phagocytosis: consume + present) | Code-consumer tools that walk macro outputs / build.rs generation / external dependencies and present what's inside as antigen-knowable substrate |
| Dendritic cells (bridge innate to adaptive) | Audit pass that takes scan-detected patterns and routes them to specific immunity claims with provenance |
| Complement system (tag for destruction) | Refactor-suggestion tools that mark code presenting antigens with structural fix recommendations |
| Affinity maturation (B-cells refine antibodies over time) | Antigen fingerprints that improve as community contributions arrive; v1 catches X%, v2 catches X+Y% as patterns refine |
| NK cells (recognize abnormal without specific antigen) | Anomaly-detection tooling that flags structurally-unusual code even when no antigen has been named |
| Cytokines (signaling propagation) | Cross-crate antigen propagation signals; descended_from is one shape; richer propagation is forward work |
| Innate vs adaptive immunity layers | Already operational: passive surface (fingerprint scan) + active surface (explicit markers) |
| Innoculation (small controlled exposure) | Test harnesses that deliberately apply antigen patterns to verify witness behavior — `examples/broken_witness.rs` is a primitive instance |
| Plasma cells (short-lived antibody factories) | Witness templates that generate immunity claims from patterns; code generation for witnesses |
| MHC Class I vs II (intracellular vs extracellular antigen) | "Internal state" antigens vs "external contract" antigens — different visibility surfaces |
| Memory cells (long-term persistence) | Already operational: `#[antigen]` declarations |
| Regulatory T-cells (prevent overreaction) | `#[antigen_tolerance]` is the primitive; future expansion: auto-tolerance learning when patterns reveal themselves as systematically over-flagged |
| Vaccine modalities (live, inactivated, subunit, mRNA) | Different strategies for applying immunity patterns: `cargo antigen vaccinate` for bulk; per-site declarations; descended-from inheritance; future modality differentiation |
| Pathogen Recognition Receptors (PRRs) | Already operational: structural-pattern matchers in `cargo antigen scan`'s fingerprint engine |

These aren't metaphors *of* what we're building — they're literal architectural
patterns the immune system has already solved. We are not analogizing; we're
copying structural answers across substrates. Biology has been iterating on
recognition-with-memory-and-inheritance for hundreds of millions of years. The
project gets to build on that work rather than re-derive it.

Each potential instantiation in the table above is a future ADR / sweep / tool
surface, surfaced as adoption produces real instances per ADR-006. Recognition,
not design.

### Future work-streams (post-v0.1)

- **A3** — Cross-crate scan + `#[descended_from]` propagation across workspaces
- **A4** — Composition rules + witness-type pluralism completion (kani/prusti/
  verus/creusot harness invocation; tier-aware audit at execution tier; cross-
  crate antigen versioning)
- **A5** — `cargo antigen vaccinate` + audit-extension + antigen-stdlib v0.1
  population (10-20 stdlib antigens covering all 8 first-principles failure
  classes)
- **A6** — rust-analyzer plugin / IDE integration; real-time fingerprint match
  surfacing as you type
- **A7+** — Comprehensive immune-system primitive expansion as adoption
  surfaces specific needs

The destination is **wide adoption as standard practice in Rust development**.
v0.1.0 starts the flywheel; A5 + A6 accelerate it; A7+ deepen the architecture
as the community surfaces what it needs.

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
  not in tribal knowledge). Refactoring is safer (immunity claims are
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

As the project develops, multiple papers will land:

- **Tool paper** (post-v0.1.0): empirical defense of antigen as a working Rust
  ecosystem tool. Biology-as-search-heuristic precision, colonization ratio,
  ATK confirmation rate, tambear adoption signal. Venues: ICFP, OOPSLA,
  Rust-specific publications.
- **Foundational paper** (post-v0.2.0 with antigen-stdlib + cross-crate scan):
  the four-window convergence + scale-invariance + no-fixed-point property as
  the architectural framing. Position antigen as the first instantiation of
  a domain-general scale-invariant recognition architecture. Venues: POPL,
  programming-language theory venues, software-engineering foundations.
- **Methodology paper** (post-A2 closure narrative): JBD-team-with-substrate
  discipline as a software-engineering practice. The verification protocol,
  recognition-not-design, depth-shift discipline, substrate-currency, and
  rationale-as-required-field as transverse principles. Venues: software-
  engineering methodology, AI-assisted development.
- **AI-dev-tooling paper** (after AI-industry comparison data accumulates):
  structural failure-class memory in code as alternative to fine-tuning
  weights for embedding domain knowledge in AI development workflows.

None of these are A2's headline. Each lands when its substrate matures. The
project's daily work produces substrate for all of them simultaneously.

---

## The case study (grounded)

Apply everything above to the project's founding incident:

**The incident**: tambear's `DeterminismClass` shipped with `meet = min` because
the polarity-inversion-when-strongest-first pattern wasn't recognized. Caught
and fixed (commit references in [`origin.md`](origin.md)). The lesson was
learned: "when class enum has strongest-first discriminants, lattice meet must
be max." Stored in: GAP-BIT-EXACT-1 issue, fix commit message, the docstring
on `DeterminismClass::meet()`, the team's collective memory.

Months later, drafting `CommutativityClass` — structurally identical pattern.
The lesson didn't transfer. `meet = min` was specified again. Almost shipped a
second time. Only caught because JBD-team's multi-agent discipline forced
re-derivation from worked examples. In a less-disciplined team or a single-
agent fresh-context session, it would have shipped.

**With antigen + everything in this scope document**:

When the original DeterminismClass bug was fixed, the PR includes one antigen:

```rust,ignore
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = "item: enum, has_method('meet', '(Self, Self) -> Self'), \
                   variants: 3..=8, attr_present('repr(u8)'), doc_contains('strength')",
    references = ["GAP-BIT-EXACT-1", "DEC-030 §1.1", "commit bb918d2"],
    summary = "When class enum represents strength-of-claim with strongest-first \
               discriminants, lattice meet must use max not min; the polarity inverts.",
)]
pub struct PolarityInvertedClassMeet;
```

Months later, `CommutativityClass` lands. Within milliseconds:

- **Pattern-hunting proactive scan** flags it as matching the fingerprint
- **Real-time IDE annotation** (post-A6) surfaces the warning AS the developer
  types the meet method
- **Library-wide propagation** means every Rust project that has adopted
  antigen-stdlib (or tambear-antigens, or the relevant downstream) inherits
  the lesson without anyone re-learning it
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

## Status snapshot (2026-05-08)

- **Sweeps**: A1 (design ratification) closed; A2 (core macros + WitnessTier +
  fingerprint grammar v1) in flight, day-2 with 4 of 9 W-streams shipped
- **Code**: 181+ tests passing as of W6a completion; build clean; clippy clean
- **ADRs ratified**: ADR-001 through ADR-016 + amendments to ADR-001/002/005/008/010
- **Postures cataloged**: 7 (sub-clause F, recognition-not-design, compose-don't-
  compete, anti-YAGNI structurally-guaranteed, implicit-to-explicit elevation,
  rationale-as-required-field, depth-shift discipline)
- **Empirical defenses**: biology-as-search-heuristic precision (5/5);
  colonization ratio over authoring envelope (8/5 = 160%); four-window
  convergence; scale-invariance of no-fixed-point property; multiple
  fractal-tier instances of the recognition-recursion pattern
- **Tambear integration**: live as exploratory adopter; will migrate to
  crates.io version-pin during v0.1.0-rc.1 smoke-test window

The substrate has earned the scope this document articulates. The work
continues toward the destination of wide adoption as standard practice in
Rust development.

---

*This document is canonical for the project's scope. The README is the
welcoming front door; [`origin.md`](origin.md) is the founding-incident
narrative; this is the comprehensive vision. Updated as the substrate
deepens.*

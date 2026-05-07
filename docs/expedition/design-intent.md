# Antigen — Design Intent

> The pre-team substrate. Captures what antigen IS, what it is NOT, why now, and the
> structural-failure taxonomy from first principles. Future JBD team extends and ratifies.

## What antigen IS

Antigen is a vocabulary of **declarations** that carry the structural memory of failure-classes
inside the Rust type system, and a cargo extension that scans, applies, and audits those
declarations.

Three primitives:
- `#[antigen(name = "...", fingerprint = "...")]` — *Build* a named failure-class with a
  structural recognition pattern.
- `#[presents(antigen_name)]` — *Give* a marker to code that is vulnerable to a known
  failure-class.
- `#[immune(antigen_name, witness = ...)]` — declare immunity, with a proof requirement
  (a proptest, a phantom-type construction, a structural pattern-matcher).

Plus an inheritance primitive:
- `#[descended_from(other_function_or_type)]` — propagate `presents` and `immune` markers
  through derivation, copy-paste, or structural similarity.

Plus the cargo extension:
- `cargo antigen scan` — find all `#[presents]` markers without corresponding `#[immune]`.
- `cargo antigen new <name>` — scaffold a new antigen declaration.
- `cargo antigen vaccinate <antigen> <pattern>` — apply immunity across a structural family.
- `cargo antigen audit` — comprehensive immunity coverage report.

## What antigen IS NOT

1. **Not a documentation system.** Documentation drifts. Antigen declarations are
   *checked by tooling*. The whole point is that the failure-class memory survives in the
   substrate, not in human attention.

2. **Not a replacement for tests, lints, deprecations, or formal verification.** Each of
   these handles a piece of the immune system already. Antigen *composes* them — it is a
   meta-layer that names what they're collectively trying to do, gives them shared
   vocabulary, and adds the missing pieces (failure-class memory, structural inheritance).

3. **Not a logic-bug catcher.** Antigen catches *failure-classes that have been named*.
   It does not surface novel logic errors. The library of antigens grows when humans
   name new failure-classes; the tooling cannot predict them.

4. **Not a static analyzer competitor.** Tools like clippy, kani, prusti, and creusot
   each address slices of the immune system (lints, model-checking, formal proofs).
   Antigen is the *connective tissue* between them — a way to say "this site is vulnerable
   to a known failure-class; here's the immunity check." It can DELEGATE to clippy lints,
   kani harnesses, or proptests; it does not replace them.

5. **Not a bureaucracy.** The cost of using antigen must be lower than the cost of the
   bugs it prevents. If declaring an antigen takes longer than fixing the bug, the project
   has failed. The tooling must scaffold antigens, propose vaccinations, and integrate
   with normal development flow without ceremony.

## Why now

Three forces make the timing right:

1. **Post-COVID vocabulary**: "antigen" is everyday language. People understand "antibody"
   and "vaccination" without explanation. The biological metaphor is universally accessible
   and emotionally grounded — important for adoption.

2. **Rust ecosystem maturity**: proc-macros, cargo extensions, custom diagnostics, and
   property-based testing are all stable. The infrastructure exists to build coherent
   tooling. Five years ago this would have been a research project; today it can be
   shipped.

3. **AI-assisted coding era**: agents lose context between sessions. Implicit memory of
   failure patterns is no longer a viable strategy because the practitioner is not
   continuous. Either the memory becomes structural (antigen) or it gets re-derived from
   scratch every session, which scales worse than mainstream programming culture already
   handles. AI-coding makes structural failure-class memory necessary, not optional.

## The 8-class failure taxonomy (first principles)

The kinds of bugs/errors that have **memory-loss-as-failure-mode**:

| # | Failure class | Description | Antibody shape |
|---|---|---|---|
| 1 | Frame-translation | Semantic interpretation drifts when crossing context boundaries. e.g., `meet=min` vs `meet=max` when each frame is locally consistent but the interface between them is broken. | Declare semantic invariants on type definitions; propagate to consumers via `#[fragile_to(frame_translation)]`. |
| 2 | Forgotten-lesson | Corrected designs lose the failure-class that motivated them. The fix is in the design, but the *memory* of why was needed is in someone's head. New types in the same family inherit the design but not the immunity. | Structural-pattern markers that propagate to similar types via `#[descended_from(...)]` chains. |
| 3 | Implicit-coupling | Changes to A break B through unstated dependency. Common when A's behavior changes in a way that's "obviously fine" from A's perspective but B was relying on the old behavior. | Explicit capability declarations + build-time graph analysis. |
| 4 | Stale-context | Using outdated information confidently. The classic substrate-over-memory failure: developer trusts mental model rather than checking current state. | Freshness markers requiring re-verification at boundaries. |
| 5 | Premature-abstraction | Generalized too early; the structure doesn't fit later cases. The abstraction was made against limited evidence and is now load-bearing for code that doesn't fit. | Track WHEN abstraction was made and against what evidence; require re-justification at expansion. |
| 6 | Incompatible-merger | Two correct things combined produce wrong things. Autoimmunity-shape: the components look like they belong together but don't. | Composition-class declarations with explicit compatibility predicates. |
| 7 | Boundary-violation | Trust-boundary check (sub-clause F shape) skipped at a structural boundary. The unchecked input poisons the downstream. | Structural enforcement of trust boundaries (could be type-system, could be lint with named diagnostic). |
| 8 | Optionality-collapse | Conditional structure becomes unconditional through routing. e.g., team-lead routes "lean X but Y's call" and downstream sees "team-lead said X." Information loss in composition. | Preserve structural conditional shape through composition with explicit weakening declarations. |

This taxonomy is **first principles** — not project-specific. Each class has been observed
in multiple projects (some in tambear; many in mainstream Rust codebases). The library of
named antigens grows by populating these classes with concrete instances.

## Biological-system → Rust constructs mapping

| Biology | Rust ecosystem |
|---|---|
| Pathogen Recognition Receptors (PRRs) | structural pattern matchers in `cargo antigen scan` |
| MHC Class I/II presentation | `#[presents(antigen)]` — every cell shows what's inside it |
| T-cell receptors | named failure-class fingerprints; specific recognition |
| B-cell memory | `#[antigen(name = "...")]` declarations that persist past specific bugs |
| Antibody | failing-as-passing test, structural-pattern proptest, phantom-type witness |
| Cytokine signaling | when an antigen presentation fires, build-time signal propagates through call graph |
| Inflammation | local response that escalates if antigen presentation increases |
| Tolerance / autoimmunity | distinguishing legitimate code from flagged-as-fragile; preventing over-triggering |
| Vaccination | `cargo antigen vaccinate <antigen> <pattern>` applying known immunity to structural family |
| Innate immunity | always-on structural checks (compile-time, type-system phantom types) |
| Adaptive immunity | failure-pattern-specific tests, named ATK/GAP markers |

The metaphor is *load-bearing*, not decorative. When the biology says "B-cell memory
persists across infections," the Rust analog must persist across compile units. When the
biology says "antibodies inherit through B-cell lineage," the Rust analog must propagate
through `#[descended_from]`. If the metaphor breaks, name where; refine; do not abandon.

## What we want, what we don't

### Want
- Ergonomic adoption (declaring an antigen takes seconds, not hours)
- Composability with existing Rust ecosystem (clippy, proptest, kani, etc.)
- Clear adoption pathway (start with stdlib antigens; community grows the library)
- Honest scope (what antigen catches; what it doesn't)
- Cross-crate inheritance (antigens declared in one crate apply to consumers)
- IDE integration potential (rust-analyzer plugin showing antigen presentations inline)
- Developer joy (the tooling should feel like having a careful peer reviewer, not bureaucracy)

### Don't want
- Yet-another-lint that nobody runs
- Documentation-shaped artifacts (we already have rustdoc; antigen is type-system shaped)
- Dependency on heavy formal-verification toolchains (kani is optional witness, not required)
- Centralized antigen registry (community-driven; antigens live with the code that needs them)
- Boilerplate that survives past usefulness (opinionated tooling defaults; minimal user typing)
- Replacing existing tools with reinvented versions (compose, don't compete)

## Adoption pathway

1. **Phase 1 — Reserved namespace.** `antigen` and `cargo-antigen` published as `0.0.1`
   placeholders with this design intent.

2. **Phase 2 — Core macros + scan command.** Implement `#[antigen]`, `#[presents]`,
   `#[immune]` macros. Implement `cargo antigen scan` walking the type graph.

3. **Phase 3 — Stdlib antigens crate.** `antigen-stdlib` provides ready-made antigens for
   common Rust failure-classes (use-after-move-conceptually-equivalent, panicking-in-Drop,
   async-in-sync-context, lock-order-inversion, etc.).

4. **Phase 4 — Community library.** Encourage projects to publish their antigens as
   crates. Document the contribution pattern.

5. **Phase 5 — IDE / rust-analyzer integration.** Show antigen presentations inline,
   suggest immunity declarations.

6. **Phase 6 — Cross-tool composition.** Witness types that delegate to clippy lints,
   kani harnesses, proptest properties, or custom diagnostics. The witness IS the
   pre-existing tool; antigen orchestrates.

## Open questions for the JBD team

1. How does antigen interact with proc-macro hygiene? Some failure-classes are about
   *macro-expanded* code rather than source code. Can `#[presents]` be applied to
   macro-generated functions?

2. Should antigens form a hierarchy? e.g., `frame-translation` is a parent;
   `meet-polarity-inversion` is a child. If so, how does inheritance compose with
   `#[descended_from]`?

3. What's the right default for `cargo antigen scan` — flag every unaddressed presentation
   as warning, error, or info? Configurable per-project, or shipped with strong defaults?

4. How to handle anti-squatting (the policy on crates.io that allows reserved names to
   be reclaimed)? Periodic version bumps as design matures? Public roadmap?

5. Is there a research-paper opportunity here? "Antigen: structural failure-class memory
   for type systems" — submitted to PLDI / OOPSLA / ICFP / ECOOP?

6. How does antigen relate to formal verification frameworks (kani, prusti, creusot,
   verus)? Is antigen a *layer above* them (orchestrator), *parallel* to them (alternative
   coverage), or *complementary* (different failure classes)?

7. The naming itself — "antigen" is suggestive of attack/disease, which is accurate but
   slightly negative. Worth considering "antibody" as the verb-form ("declare an antibody
   against frame-translation") for ergonomic phrasing? Or stick with antigen consistently?

These questions are *for the team*, not pre-answered here.

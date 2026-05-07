# Antigen — Revolutionary and Not

> Honest assessment of what's genuinely new and what's existing-tools-recomposed. This is
> the document that protects future-team from over-claiming OR under-claiming.

## What's revolutionary

### 1. Failure-class memory as first-class structural artifact

Most programming culture treats failure-class memory as **tacit knowledge**. Senior
engineers learn through career-pain. Mentorship transmits some of it. Documentation
captures fragments. Commit messages decay.

Antigen makes failure-class memory **first-class** — declared in the type system,
checked by tooling, propagated by composition. This is not an incremental improvement on
documentation; it's a structural relocation of where memory lives.

This matters more in the AI-coding era because agents lose context between sessions. The
implicit-memory strategy that works (badly) for human teams doesn't work at all for agent
teams. Antigen is the *only* viable memory architecture for cross-session, cross-agent,
cross-team coordination on failure-classes.

### 2. Structural inheritance of immunity through composition

When `MyType` is `#[descended_from(YourType)]`, MyType automatically inherits YourType's
antigen presentations and (conditionally) immunities. This is **inheritance of immunity
through derivation** — a primitive that doesn't exist in any current Rust tool.

The biological analog (B-cell lineage, antibody clonal expansion) is real. The Rust
ecosystem has nothing equivalent today. Existing tools handle:
- Code-level inheritance (trait impls, generics)
- Test-level inheritance (parameterized tests)
- Lint-level inheritance (clippy's structural patterns)

But none of these inherit *immunity-to-named-failure-classes*. That's the new primitive.

### 3. The composition operation on failure-class declarations

Antigen treats failure-classes as composable objects. Two related failure-classes can be
joined into a parent class (with a `family` field). Children of a class inherit
fingerprint matchers from the parent. The class taxonomy is itself a refinement-lattice.

This composition operation on failure-class declarations is genuinely new. Existing tools
treat each lint, each test, each verification harness as a flat artifact. Antigen treats
them as elements of a structured hierarchy with composition rules.

### 4. Vaccination as a development action

`cargo antigen vaccinate <antigen> <pattern>` is a new kind of development action: apply
known immunity across a structural family. This is bulk operation on the failure-class
graph — analogous to a refactoring tool but operating on the immune-system layer rather
than the syntax layer.

No current Rust tool does this. The closest analog is `cargo fix` (auto-applies certain
lint suggestions), but that's per-site, not structural-family.

## What's NOT revolutionary

### 1. Most pieces exist

Almost every individual piece of antigen exists somewhere in the Rust ecosystem:

| Antigen feature | Existing analog |
|---|---|
| `#[antigen]` declarations | `#[deprecated]` (memory-of-failure for one specific case) |
| `#[presents]` markers | `#[allow(...)]` lint suppressions (acknowledging vulnerability) |
| `#[immune]` with witness | `#[test]` or `#[cfg_attr(test, ...)]` (proof of behavior) |
| `cargo antigen scan` | `cargo clippy` (structural pattern recognition) |
| `cargo antigen vaccinate` | `cargo fix` (auto-applying patterns) |
| Witness types | `kani::proof`, `prusti::trusted`, `proptest!` |
| Cross-crate inheritance | Standard Rust trait/derive mechanics |

The contribution is **the composition** — naming what these tools collectively are, giving
them shared vocabulary, and adding the missing primitives (failure-class memory,
structural inheritance, vaccination).

### 2. The biological metaphor is suggestive, not load-bearing for adoption

The metaphor is *useful for design* (suggests primitives like B-cell memory, T-cell
receptors, vaccination, autoimmunity). It's *suggestive for vocabulary* (people understand
"antigen" post-COVID). But the metaphor is NOT what gets adoption — the *ergonomics* and
the *failure-class memory* are.

If the macros are awkward to use, no biology language saves the project. If the failure-
class library is sparse, no clever subcommand naming compensates.

### 3. The 8-class failure taxonomy is not novel

Each of the 8 first-principles failure classes has been observed and named in software
engineering literature:
- Frame-translation: see "boundary errors" in distributed systems literature
- Forgotten-lesson: well-documented in technical-debt research
- Implicit-coupling: classic OOP critique
- Stale-context: known as "TOCTOU" in security; broader than security
- Premature-abstraction: see "speculative generality" in refactoring literature
- Incompatible-merger: classic composability research
- Boundary-violation: trust-boundary literature
- Optionality-collapse: Schmidt's "premature commitment" in HCI

The novelty is treating these as **a structured taxonomy with shared vocabulary** rather
than as separate folklore items. The taxonomy is generative — every class can be
populated with named instances.

### 4. Cargo extension pattern is well-trodden

`cargo-mutants`, `cargo-fuzz`, `cargo-careful`, `cargo-bisect`, `cargo-flamegraph`,
`cargo-edit`, `cargo-watch` — the cargo subcommand pattern is mature. Antigen doesn't
innovate on the subcommand mechanics. It uses the existing pattern.

## Adoption pathway

### Realistic adoption sequence

1. **Reservation phase** (weeks): namespace claimed, design docs published, GitHub
   community visible.

2. **Core macros + scan** (months): basic `#[antigen]`, `#[presents]`, `#[immune]` work;
   `cargo antigen scan` identifies presentations without immunity. Used by early adopters
   willing to write their own antigens.

3. **Stdlib antigens** (months): `antigen-stdlib` provides 20-50 well-known failure-classes
   for common Rust patterns. Adoption barrier drops sharply because users get value
   without writing antigens themselves.

4. **Witness ecosystem** (months): integrations with proptest, kani, prusti make witness
   declarations cheap. Delegating to existing tools instead of inventing new ones.

5. **IDE integration** (year+): rust-analyzer plugin shows antigen presentations inline.
   This is the ergonomics threshold — when antigen markers are visible during editing,
   adoption accelerates significantly.

6. **Community library** (ongoing): projects publish their antigens as crates. The library
   grows from 50 to 500+ over 1-2 years. Cross-project failure-class patterns become
   visible.

7. **PLDI/OOPSLA/ECOOP paper** (research-track parallel): the structural inheritance and
   composition primitives may be paper-worthy. Not the goal but a viable side outcome.

### What could kill it

- **Boilerplate burden**: if declaring an antigen takes more than 60 seconds, nobody does
  it. The macro must scaffold aggressively; defaults must be sensible.

- **Slow scan**: if `cargo antigen scan` takes more than 5 seconds on a moderate workspace,
  it doesn't get run in CI. Performance is non-negotiable.

- **False-positive noise**: if scan flags many sites that are legitimately not vulnerable
  ("autoimmune"), users disable it. The recognition fingerprints must be precise enough
  to minimize over-flagging.

- **Conflicting-tool perception**: if antigen is perceived as competing with clippy,
  proptest, or kani, adoption stalls. The composition message must be clear and visible.

- **Lack of stdlib antigens**: if early users have to write their own antigens, only the
  most motivated adopt. The stdlib library is the adoption flywheel.

## What antigen doesn't replace

- **rustc itself**: type errors, ownership errors, lifetime errors are caught by the
  compiler. Antigen handles failure-classes that compile but are wrong.

- **Tests**: unit tests, integration tests, property tests catch bugs. Antigen
  *complements* tests by adding cross-project memory; it does not replace them.

- **Documentation**: README, rustdoc, design docs explain WHAT and WHY. Antigen marks
  WHAT-FAILURE-CLASS and WITH-WHAT-WITNESS. Different surface.

- **Code review**: human reviewers catch judgment errors. Antigen catches *named pattern*
  errors. Reviewers can use antigen as a checklist; antigen does not replace review.

- **Formal verification**: kani, prusti, creusot prove specific properties. Antigen marks
  presentations and delegates to formal-verification witnesses for those that benefit.
  Antigen is broader (more failure-classes covered) but shallower (less rigorous proof).

## Why everyone WOULD use it (the optimistic case)

If antigen ships well:
- **Onboarding**: new contributors to a project see antigen declarations and learn the
  failure-classes the project considers important. The codebase teaches itself.
- **Cross-project memory**: failure-classes named in one project propagate via
  `antigen-stdlib` to many projects. Lessons learned compound.
- **CI confidence**: adopting `required = [...]` antigens in `Cargo.toml` provides a
  structured assurance that important failure-classes are addressed.
- **AI-coding amplification**: agents read antigen declarations, understand the project's
  failure-class context, and produce code that respects the discipline without needing
  the discipline re-explained.
- **Cross-crate composition**: when a dependency declares antigens, consumers can
  immediately see what they're inheriting and what immunity claims propagate.

## Why everyone WOULDN'T use it (the pessimistic case)

If antigen mis-ships:
- Boilerplate burden too high → only fanatics adopt
- Stdlib library too sparse → users write their own, give up
- Performance too slow → not run in CI, becomes documentation
- Ecosystem fragmentation → competing antigen libraries with incompatible vocabularies
- Misuse pattern → `#[immune]` without real witness becomes the new "trust me" comment
- Tooling decay → antigen library entries don't track Rust evolution; obsolescence

These are real risks. The team must engineer against them.

## The honest summary

**Antigen is not a paradigm shift. It's a structural relocation of memory.**

The failure-class memory that currently lives in human heads, commit messages, and stale
documentation gets moved into the type system and the cargo tooling. That's the contribution.

The novelty is in the *composition* of existing tools (lints, tests, formal verification,
deprecations) under a coherent vocabulary with shared primitives. Each piece is familiar;
the collection is new.

Whether this is enough to change the Rust ecosystem depends entirely on execution quality,
adoption flywheel, and stdlib library curation — not on conceptual cleverness. The idea is
straightforward; the engineering is the project.

**It's worth doing.** Asymmetric upside (could change how Rust ecosystem handles
failure-class memory) vs. moderate cost (months of focused engineering). Even if it
doesn't fully succeed, the named failure-class taxonomy and the composition primitives
benefit the broader ecosystem as research artifacts.

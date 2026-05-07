# Contributing to Antigen

Thank you for your interest in antigen — structural memory of failure-classes for Rust.

> **Status: Design phase.** The project is in active design. Code contributions are
> premature until the design phase resolves and v0.1 ships. Until then, the most
> valuable contributions are: design feedback, prior-art surfacing, and proposing
> failure-classes for the eventual stdlib library.

## What we welcome right now

### 1. Design feedback

The design substrate lives in [`docs/expedition/`](docs/expedition/). Read:
- [`design-intent.md`](docs/expedition/design-intent.md) — what antigen IS, what it ISN'T, why now
- [`api-shape.md`](docs/expedition/api-shape.md) — sketch of macros and cargo subcommands
- [`revolutionary-and-not.md`](docs/expedition/revolutionary-and-not.md) — honest claims and limits
- [`origin.md`](docs/origin.md) — the post-mortem narrative that motivated the project
- [`decisions.md`](docs/decisions.md) — ratified ADRs (foundational)
- [`process.md`](docs/process.md) — formal lifecycle: how decisions get drafted, reviewed, ratified, and govern downstream work

If you spot a design flaw, an over-claim, an under-claim, an ambiguity, or a missing
consideration — open an issue with the `design-discussion` template.

### 2. Prior-art surfacing

Antigen aims to compose existing Rust ecosystem tools, not compete with them. If you
know of a tool we haven't surveyed in [`docs/expedition/ecosystem-composition.md`](docs/expedition/ecosystem-composition.md),
please tell us — open an issue with `prior-art-surfacing` in the title.

If you know of academic work that's directly relevant (refinement types, design-by-
contract, named-effect type systems, lightweight verification), please tell us too —
[`docs/expedition/academic-context.md`](docs/expedition/academic-context.md) is the
landing page.

### 3. Failure-class proposals

The 8 first-principles failure classes are:
1. Frame-translation
2. Forgotten-lesson
3. Implicit-coupling
4. Stale-context
5. Premature-abstraction
6. Incompatible-merger
7. Boundary-violation
8. Optionality-collapse

If you've encountered a real-world Rust failure that doesn't seem to fit any of these
classes, propose either (a) a refinement to an existing class, or (b) a new class.
Open an issue with the `failure-class-proposal` template.

If you've encountered a real-world Rust failure that DOES fit one of the classes and
you think it's worth including in the eventual `antigen-stdlib` library, propose it
via the `antigen-stdlib-candidate` template.

### 4. Use-case stories

If you're working on a Rust codebase that has the failure-class-memory problem
described in [`docs/origin.md`](docs/origin.md), and you'd like to be an early adopter
when antigen ships, please introduce yourself via a Discussions thread. Real adoption
stories shape the project's trajectory.

## What we don't welcome right now

### Code PRs against the placeholder crates

The `0.0.1` versions of `antigen` and `cargo-antigen` are namespace placeholders. They
will be substantially rewritten when the design phase resolves. Code PRs against them
are unlikely to land — please contribute design feedback instead.

### Premature optimization or extension

Until the v1 design ratifies (expected after the antigen JBD team completes its first
sweeps), proposing API extensions or optimizations is premature. The design substrate
documents many open questions; those are the right surface for input now.

### Disagreement framed as bug reports

If you disagree with a design decision (e.g., "the biological metaphor is silly,"
"this should be a clippy plugin not its own crate"), please open a Discussions thread,
not a bug report. The decisions are ratified ADRs (see `docs/decisions.md`); changes
go through the amendment process described in that file.

## The formal process

Antigen has a formal architectural process inherited from the tambear project, captured
in [`docs/process.md`](docs/process.md). Highlights:

- **ADR lifecycle**: Draft → Phase 1-8 deconstruction (the witness) → Adversarial review → Math/systems review → Scientist validation → Ratification → Enforcement → Reference and propagation
- **Sweep planning**: larger units of work that ratify or implement multiple ADRs together. Sweep READMEs cite blocking work-streams, unlocked downstreams, and ADRs operated under
- **ADRs govern code, sweeps, and other ADRs** through cross-references. Drift in any direction surfaces via the cross-references.
- **Team roles** (when the JBD team is active): pathmaker, navigator, scout, naturalist, observer, math/systems-researcher, adversarial, scientist, aristotle. Each has process responsibilities.

When the project ships its first real version, this CONTRIBUTING.md will be updated
with:
- Code-style guidelines (rustfmt + clippy strict)
- Test requirements (every public API has tests; every antigen has property tests)
- PR review process (at least one maintainer approval; CI must pass)
- Antigen-stdlib contribution process (failure-class evidence, fingerprint precision,
  witness coverage)
- Witness type extension process (for adding new witness providers)
- Release cadence and versioning policy

For now: design feedback, prior-art surfacing, and failure-class proposals are the
highest-value contributions.

## Releases

Releases happen via git tag. Tag a commit `v<version>` (e.g., `v0.0.2`), push the tag,
and GitHub Actions publishes both crates to crates.io and creates a GitHub release.
The CHANGELOG.md must have an entry for the released version. See
[`.github/workflows/release.yml`](.github/workflows/release.yml) for the workflow
mechanics.

Only maintainers can push tags. Contributors propose version bumps via PR; tagging
follows merge.

## Code of Conduct

This project follows the [Rust Code of Conduct](CODE_OF_CONDUCT.md). Adopt the same
standards in all communication: be welcoming, be considerate, be respectful, be
careful in the words you choose, when we disagree try to understand why.

## Communication channels

- **GitHub Issues**: design discussions, failure-class proposals, prior-art surfacing
- **GitHub Discussions**: introductions, use-case stories, broad questions
- **PRs**: only for documentation typos and clear design-doc improvements until v0.1

Thank you for contributing to building structural memory for the Rust ecosystem.

# Contributing to Antigen

Thank you for your interest in antigen — structural memory of failure-classes for Rust.

> **Status: actively shipping.** v0.1.0-rc.3 is published on crates.io; the substrate-witness pipeline, Oracle 5-state lifecycle, and cross-cutting attestation are all live. The architectural-posture-shift event that opened v0.2 ratified ten ADRs as the v0.2 direction lock. We welcome substantive contributions across multiple paths.

---

## How to contribute

### Failure-class proposals (most valuable right now)

Antigen's stdlib grows from real-world failure-class encounters. If you've hit a Rust failure that:

- Doesn't fit any of the eight first-principles classes (Frame-translation, Forgotten-lesson, Implicit-coupling, Stale-context, Premature-abstraction, Incompatible-merger, Boundary-violation, Optionality-collapse), OR
- Fits one of the classes and you think it belongs in the eventual `antigen-stdlib` library,

please open an issue with the `failure-class-proposal` template. Include:

- A minimal reproduction or a real-world commit where it surfaced
- The structural shape (item kind, key methods, attribute patterns) — enough to derive a fingerprint
- The witness you used (test, proptest, formal proof, type-system shape, external lint)
- References (PR threads, post-mortems, RFCs, CVEs) if any exist

Failure-class proposals are the single highest-impact contribution path. Each accepted proposal becomes a candidate for the v0.2+ stdlib expansion.

### Code PRs

PRs are welcome against `main`. We'll guide them through our internal architectural discipline (see "What happens to your PR" below). You don't need to know our internal tooling to contribute.

Before opening a substantive PR:

1. **Open a discussion or issue first** if the change is non-trivial. This avoids you doing significant work on something we'd want to reshape during review.
2. **Run the local gates**:
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
   ```
   CI runs these on every push.
3. **Write tests** for new behavior. The workspace currently runs 554 tests; new public API should add coverage.
4. **Match existing conventions** — rustfmt-default style; comments explain non-obvious *why*, not redundant *what*.

### Prior-art surfacing

Antigen composes existing Rust ecosystem tools rather than competing where composition serves adopters (ADR-002 Amendment 2). If you know of a tool, RFC, or academic work we haven't engaged with, open an issue tagged `prior-art-surfacing`.

Particularly valuable: refinement types, design-by-contract, named-effect type systems, lightweight verification frameworks, supply-chain integrity tooling, version-boundary recognition work.

### Discussions

For broader questions — use cases, design rationale, where antigen fits in your workflow, ecosystem positioning — use GitHub Discussions rather than issues.

If you disagree with a ratified ADR, GitHub Discussions is the right surface. Decisions are tracked in [`docs/decisions.md`](docs/decisions.md); changes go through the amendment process. Disagreement framed as bug reports gets reframed as discussion.

---

## What happens to your PR

Antigen has a formal internal architectural process inherited from the tambear project. **You don't need to interact with it directly.** When you submit a PR, the antigen team takes it through our discipline:

1. **First-principles review** — an aristotle-role agent (or maintainer) deconstructs the change against the ratified ADRs, surfaces any hidden assumptions, identifies cross-cutting concerns
2. **Adversarial review** — failure-mode hunting; what's the worst input? what silent failures could pass tests?
3. **Substrate validation** — does the change match what the codebase actually does? are claims in the PR description accurate?
4. **Merge or revision request** — clean PRs land; revision requests come with specific findings

This typically takes a few days for non-trivial changes. Your PR description is the primary substrate we work from — clearer description = faster review.

If you're curious about the internal discipline itself (the ADR lifecycle, sweep planning, role responsibilities), see [`docs/process.md`](docs/process.md). It documents how the antigen team coordinates internally — not what we ask of contributors.

---

## What we don't merge

- **Premature optimization** before the relevant subsystem stabilizes. v0.2 work is substantively in flight; optimization PRs to surfaces still under design will be deferred.
- **Speculative API extensions** without substrate. Per ADR-006 Amendment 1, adopter-side extensions need substrate-grounded encounters; stdlib growth is research-driven (different discipline, different criteria — see ADR-022).
- **Cosmetic or scope-creep changes** mixed into substantive PRs. One concern per PR keeps the review graph clean.

---

## Code style + test requirements

- **rustfmt** with workspace default config (`cargo fmt --all -- --check` must pass)
- **clippy** with workspace pedantic + nursery lints (`cargo clippy --workspace --all-targets -- -D warnings` must pass)
- **All new public API has tests.** Property tests via proptest are encouraged for parser/grammar surfaces.
- **Rustdoc warnings** treated as errors (`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`)
- **No `unsafe`** in this workspace (`unsafe_code = "forbid"` at workspace level)
- **No `.unwrap()` in non-test code** (`unwrap_used = "deny"`)
- **MSRV is 1.95** — antigen tracks recent stable (~stable-minus-1), revisited each release; the MSRV-aware resolver (`resolver = "3"`) keeps deps within the floor

CI gates all of the above.

---

## Releases

Releases happen via git tag. Tag a commit `v<version>` (e.g., `v0.1.0-rc.4`), push the tag, and GitHub Actions publishes all five crates to crates.io and creates a GitHub release. CHANGELOG.md must have an entry for the released version. See [`.github/workflows/release.yml`](.github/workflows/release.yml) for the workflow mechanics.

Only maintainers can push tags. Contributors propose version bumps via PR; tagging follows merge.

---

## Code of Conduct

This project follows the [Rust Code of Conduct](CODE_OF_CONDUCT.md). Adopt the same standards in all communication: be welcoming, be considerate, be respectful, be careful in the words you choose, when we disagree try to understand why.

---

## Communication channels

- **GitHub Issues**: failure-class proposals, prior-art surfacing, bug reports, design questions
- **GitHub Discussions**: introductions, use-case stories, broad questions, disagreement with ratified decisions
- **Pull Requests**: code changes, doc improvements, test additions

Thank you for contributing to building structural memory for the Rust ecosystem.

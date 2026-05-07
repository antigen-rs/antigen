# Changelog

All notable changes to the antigen project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The project is in active design. The first functional release will be `0.1.0` after
the antigen JBD team completes its initial sweeps.

### Planned for 0.1.0

- `#[antigen(name, fingerprint, family, summary, references)]` macro
- `#[presents(antigen)]` marker
- `#[immune(antigen, witness)]` marker with witness validation
- `#[descended_from(parent)]` propagation
- `cargo antigen scan` (find unaddressed presentations)
- `cargo antigen new <name>` (scaffold a new antigen)
- Witness adapters for `#[test]`, `proptest!`, and clippy lints
- Initial `antigen-stdlib` companion crate with seed antigens for the 8 first-principles
  failure classes

### Planned for 0.2.0+

- `cargo antigen vaccinate` (bulk apply immunity across structural families)
- `cargo antigen audit` (comprehensive coverage report)
- Witness adapters for kani, prusti, creusot, verus, cargo-mutants
- Phantom-type witness templates
- IDE integration plugin (rust-analyzer)
- Cross-crate antigen versioning
- Tier-1 amendment template for new ClaimPayload constructors

## [0.0.1] — 2026-05-07

### Added

- Workspace scaffolding (Cargo.toml, dual MIT/Apache-2.0 license)
- `antigen` crate placeholder (lib) with module-doc explaining design phase
- `cargo-antigen` crate placeholder (bin) with reserved subcommand notice
- Design substrate documents:
  - `docs/origin.md` — narrative post-mortem motivating the project
  - `docs/expedition/design-intent.md` — what antigen IS, what it ISN'T
  - `docs/expedition/api-shape.md` — sketch of API surface
  - `docs/expedition/revolutionary-and-not.md` — honest claims and limits
  - `docs/expedition/team-briefing.md` — for the JBD team at spawn time
  - `docs/expedition/failure-class-instances.md` — real-world Rust ecosystem instances
    of the 8 first-principles failure classes
  - `docs/expedition/ecosystem-composition.md` — composition opportunities with existing
    Rust tools
  - `docs/expedition/academic-context.md` — relationship to existing academic work
  - `docs/expedition/inheritance-from-tambear.md` — disciplines and patterns inherited
    from the tambear project
- Foundational ADRs (ADR-001 through ADR-008) ratified by Tekgy + Claude in pre-team
  scaffolding
- `docs/glossary.md` — vocabulary anchor
- `docs/process.md` — formal ADR lifecycle and governance (how decisions get drafted,
  reviewed, ratified, and govern downstream work; inherited from tambear DEC discipline
  and adapted for antigen)
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md` — open-source hygiene
- `.github/workflows/ci.yml` — CI scaffolding (cargo check + test + fmt + clippy + doc)
- `.github/workflows/release.yml` — release workflow (git-tag-triggered crates.io
  publish + GitHub release)
- GitHub issue templates and PR template
- 9 starter campsites for the future antigen JBD team

### Reserved

- Crate name `antigen` on crates.io
- Crate name `cargo-antigen` on crates.io
- Org name `antigen-rs` on github.com
- Repository name `antigen-rs/antigen` on GitHub

[Unreleased]: https://github.com/antigen-rs/antigen/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/antigen-rs/antigen/releases/tag/v0.0.1

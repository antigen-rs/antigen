# Changelog

All notable changes to the antigen project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Tracking work for v0.1.0 final + post-v0.1.0 sweeps. Cf. `sweeps/`.

## [0.1.0-rc.1] — 2026-05-08

First functional release candidate. Sweep A2 (core macros + scan + audit
completion) closed with 187 passing tests across 18 suites; clippy + doc gates
clean. Cuts the substrate the JBD team built across A1 (10 ratified ADRs +
4 amendments) and A2 (W1-W8 implementation work-streams).

### Added

#### Macros (`antigen-macros`, re-exported from `antigen`)

- `#[antigen(name, fingerprint, family, summary, references)]` — declare a
  named failure-class with a structural fingerprint per ADR-001 + ADR-010
- `#[presents(antigen_type)]` — mark code as exhibiting an antigen's
  structural pattern
- `#[immune(antigen, witness, rationale?)]` — declare immunity with a
  witness reference; required-witness enforcement at parse time
- `#[descended_from(parent)]` — propagate antigen markers through
  structural derivation (cross-crate walking is A3 work)
- `#[antigen_tolerance(antigen, rationale, until?, see?)]` — mark
  fingerprint matches that are deliberate-not-vulnerable (ADR-011);
  rationale required and non-empty, `until` non-empty if present
- All five macros emit token-precise error spans per ADR-008 (W4); errors
  anchor at the offending literal or the macro's argument list, never at
  call_site

#### Fingerprint grammar (`antigen-fingerprint`)

- v1 DSL parser per ADR-010 Amendment 1 (Path C: custom `syn::ParseBuffer`,
  not `syn::parse2::<Expr>`)
- Seven item-level operators per ADR-010 Amendment 3 Clause C:
  - `item: <kind>` — struct / enum / trait / fn / impl / type / mod
  - `name: matches("<glob>")` — bespoke 20-line `glob_match_ident` with
    `*` and `?` metachars (no external glob dep)
  - `variants: M..=N` — inclusive enum variant-count range
  - `has_method("<name>", "<sig>")` — signature pre-parsed at load time
    per Performance Invariant 2 (cached as `Option<String>` on
    `MethodPattern::normalized_signature`)
  - `attr_present("<path>")` — outer attribute path matcher
  - `doc_contains("<substring>")` — case-sensitive doc-text search
  - `body_contains_macro("<name>")` — native syn::Block walker for
    `panic!`/`unreachable!`/etc. (Clause C, NOT delegated to a body engine)
- Composition: `all_of([...])`, `any_of([...])`, `not(...)` with
  `not`-only-inside-`all_of`-with-positive-sibling enforcement per OQ3
- Performance invariants honored: single-pass walks, pre-parsed sigs,
  depth ≤ 10 + node-count ≤ 256 caps at parse time, node-kind dispatch
- Compile-time DSL validation: `#[antigen]` rejects malformed fingerprints
  at macro-compile time per Clause E

#### Scan (`antigen::scan` + `cargo antigen scan`)

- `scan_workspace` walks the source tree, collects explicit declarations,
  then synthesizes fingerprint-match `Presentation`s with
  `MatchKind::FingerprintMatch` per ADR-001 Amendment 1 Change 2
- Item-identity matching via `ItemTarget` (W3) — replaces the pre-W3
  proximity heuristic; methods inside `impl` blocks carry their enclosing
  trait + target type so two `drop` methods on different types don't
  collide
- Tolerance recognition: `#[antigen_tolerance]` markers acknowledge
  fingerprint matches; `unaddressed_presentations` consults tolerances
- `ScanReport::orphaned_tolerances()` flags tolerances referencing antigens
  not declared in the workspace (ATK-A2-009)
- Span-precise line tracking via `syn::spanned::Spanned::span().start().line`
  (requires `proc-macro2` `span-locations` feature)
- Path-qualified attribute name handling (`#[antigen::antigen]`,
  `#[my_alias]` after `use ... as`)
- 5-state CLI output: explicit / fingerprint match / tolerated / unaddressed
  / immunity claims, with remediation guidance

#### Audit (`antigen::audit` + `cargo antigen audit`)

- Four-tier `WitnessTier { None=0, Reachability=1, Execution=2,
  FormalProof=4 }` per ADR-005 Amendment 3 (discriminant 3 reserved for
  future BehavioralAlignment)
- Parallel `AuditHint` axis with structured variants for per-case
  disambiguation (FunctionResolves / TestAttributePresentNotInvoked /
  TestAttributePresentIgnoreSkipped / ExternalToolPrefixRecognized /
  PhantomTypeShapeRecognized / AmbiguousResolution / ...)
- `WitnessKind` extended with `IgnoredTest` (anergic-B-cell cognate),
  `PhantomType { proof_type, type_params, constructor }` per ADR-013
- `WitnessStatus::Ambiguous { candidates }` for collision detection;
  `FunctionIndex` tracks all candidate locations rather than silently
  picking one
- `is_well_formed() = meets_tier(Execution)` — informational only; not
  wired to `--strict` at v0.1 (see ATK-CLI-003 below)
- `audit --strict` gates on `all_meet_tier(Reachability)` — exits 1 for
  `Missing`/`NotFound`/`Ambiguous` witnesses (`WitnessTier::None`);
  `Execution`-tier gating arrives in A3 with `cargo test` integration
  (ATK-CLI-003: previously always-exited-1, training users to disable)
- Phantom-type witness recognition for `Type::<Args>::ctor` shapes
- Structural `proptest!` witness detection (W5) — replaces the pre-W5
  textual `source.contains("proptest!")` sentinel that over-classified
  every function in any file mentioning the macro
- `#[ignore]` distinction in `detect_kind` (W7 + ATK-A2-012)

#### Workspace + tooling

- `antigen-fingerprint` workspace member per ADR-010 Amendment 3 Clause E
  — both `antigen-macros` (compile-time validation) and `antigen`
  (scan-time matching) consume it
- `cargo antigen scan|audit|new|vaccinate` subcommand binary; `new` and
  `vaccinate` are scaffolded with helpful "design phase" messages
  (real implementations land A3+/A5)
- Workspace-level `[lints.rust]` `unsafe_code = forbid` and
  `missing_docs = warn`; `[lints.clippy]` pedantic + nursery at warn level
  with explicit ergonomic allow-list

### Known v0.1.0 limitations (deferred to later sweeps)

- **No function-body fingerprint patterns** (ADR-012 amendment) — v1
  grammar matches at the item level. `body_contains_macro` is the one
  body-level operator shipped (native syn walker). General `body_pattern`
  awaits the W6b ast-grep subprocess decision per ADR-015.
- **No macro-output recognition** (ADR-014 `#[antigen_generates]`) —
  derives, declarative macros, and proc-macros expand outside the scan's
  view. v1 scans pre-expansion source only per ADR-010 Amendment 3 Clause A.
- **No cross-crate `#[descended_from]` propagation** — A3 sweep work.
- **No witness execution** — audit reports Reachability for `#[test]`
  resolution; promotion to Execution requires actual `cargo test`
  invocation (A3+).
- **No external-tool invocation** — `clippy::`/`kani::` prefixes get
  Reachability + `ExternalToolPrefixRecognized` hint until A3+ runs the
  tools.
- **No `cargo antigen vaccinate`** — A5 work; requires fingerprint grammar
  v1 and witness library to be stable (both ship in 0.1.0).
- **No `antigen-stdlib`** — A5 populates the 8 first-principles failure
  classes. `panicking-in-drop` exercises body_contains_macro end-to-end
  in `antigen/examples/basic.rs` as a standing demo.
- **No fabricated-path-prefix detection** — `nonexistent::module::real_fn`
  silently drops the prefix at this layer; the underlying tier-honesty
  catches it (ATK-A2-011), but full module-graph resolution is A3 work.
- **Same-name proptest+free-fn collision is `Ambiguous`, not silently
  picked** — by design (ATK-W5-007 reframe under W7); the user resolves
  by renaming or qualifying. Consistent with ATK-A2-005's discipline.

### Reserved / placeholders

- `antigen-fingerprint` and `antigen-macros` are workspace-internal
  crates published to crates.io alongside `antigen` (because Cargo
  requires them to be) but documented as "use `antigen` instead." The
  evaluator-trait public-vs-private question (ADR-015 §S3) is reserved
  for the second-backend ratification.

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
- `docs/vision-pitch.md` — 1500-word ecosystem-outreach pitch
- `docs/expedition/case-study-determinism-class.md` — pseudocode walkthrough of how
  antigen would have caught the originating bug pattern (closes the loop origin.md
  opens)
- `docs/expedition/stdlib-seed-antigens.md` — 10 concrete antigen declarations for
  the eventual `antigen-stdlib` v0.1 catalog
- `docs/expedition/first-sweep-plan.md` — concrete plan for Sweep A1 (design
  ratification + scope-lock for Sweep A2)
- `docs/expedition/risk-register.md` — adversarial-perspective catalog of what
  could kill the project
- `docs/expedition/conventions.md` — naming, file layout, witness type abbreviations
- ADR-009 (Adoption gradient — antigen meets consumers at any discipline level) and
  ADR-010 (Fingerprint grammar v1 — syn-based AST visitor pattern) ratified into
  `docs/decisions.md`
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

[Unreleased]: https://github.com/antigen-rs/antigen/compare/v0.1.0-rc.1...HEAD
[0.1.0-rc.1]: https://github.com/antigen-rs/antigen/releases/tag/v0.1.0-rc.1
[0.0.1]: https://github.com/antigen-rs/antigen/releases/tag/v0.0.1

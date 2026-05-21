# Changelog

All notable changes to the antigen project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-rc.1] — 2026-05-20

First release candidate. Consolidates A2 (core macros + scan + audit completion)
+ A3.5 (onboarding sweep) + the discipline-witnesses arc (ADR-019, ADR-020,
ADR-021) into a single shipped rc. The earlier 2026-05-08 `[0.1.0-rc.1]` entry
below documents preliminary substrate that was planned but never shipped to
crates.io — its content is absorbed here.

### Discipline-witnesses arc (NEW — 2026-05-19/20 session)

#### ADR-019 — Substrate-witness predicate family (RATIFIED)

Extends witness vocabulary beyond code-side substrate (test_fn / proptest! /
clippy:: / phantom-type) to **substrate other than the code being audited**:
ratified docs, sign-off records, signed git trailers, oracle completion
markers, attestation sidecars. Closed combinator grammar (`all_of` / `any_of`
/ `not`) over sealed leaf primitives (`signers`, `ratified_doc`,
`signed_trailer`, `oracles_complete`, `fresh_within_days`). Tier-honesty
preserved via three-axis output (`WitnessTier × AuditHint × EvidenceKind`).
ADR-005 Amendment 3 extended to substrate-witness recognition surface;
ratchet-asymmetry property named explicitly; bounded audit-of-audit recursion
named explicitly.

#### ADR-020 — Cross-cutting attestation primitive (RATIFIED)

`attested = (who, allowed_types, why, scope)` available as a macro parameter
on any antigen-related macro (`#[antigen]`, `#[immune]`, `#[antigen_tolerance]`,
possibly `#[descended_from]`). Distinct from `requires =` substrate-witness
predicates — attestation is the *declaration* of who attests; substrate-witness
predicates *evaluate* against that declaration. Layer 1 adoption gradient
(ADR-009) compliance: light-touch attestation reaches every adopter without
requiring the full predicate language. Notary-arc biology grounding (B6 from
naturalist work).

#### ADR-021 — OracleRef generalization + Oracle artifact-class (RATIFIED)

Oracle as **structurally distinguished artifact** (Model B per Tekgy decision)
rather than typed pointer. Five-state lifecycle:
`Draft → Complete → {Deprecated, Retired, Revoked}`. Dedicated stewardship
role separate from signers. State transitions are steward-authorized events
with provenance trail. Signers cannot attest against DRAFT oracles
(`oracle_state_at_attestation` field enforces). OracleRef as tagged union
covers LocalFile, Url, Doi, Arxiv, GitHubIssue, Other — same structural
treatment regardless of physical location. Audit validates metadata +
completion-marker + version-pin but **never reads/interprets oracle content**
(substantive judgment lives at sign-time human/LLM work; tier-honesty caps
oracles at Execution tier). Additive-only schema evolution discipline ratified
(no migration framework needed). Five Class-1 biology cognates including
immune-memory + V(D)J recombination.

#### Tolerance-ratification (scout S1 — plugs ADR-011 vibes-grade gap)

`#[antigen_tolerance(X, sidecar = true)]` opt-in enables structured
attestation for tolerance claims; schema **isomorphic to immunity sidecars**.
Audit emits new `tolerance-vibes-grade` hint with `EvidenceKind::None` for
unattested tolerance — makes the tier-honesty gap visible.

#### Three signature tiers

`SignatureStrength = TextStamp | GitTrust | CryptoSigned`. Categorical, NOT
ordinal — trust is project-declared per-antigen, not inherent in the enum.
TextStamp (name + timestamp; no infra required) opens adoption to LLM agents
and reviewers without git config. GitTrust (git config user.name/email +
fingerprint pin) is the v0.1 default for human teams. CryptoSigned slot
reserved for v0.4+ DSSE envelope + Sigstore identity-bound activation path.

#### Delta-attestation with anti-laundering safeguards

`cargo antigen attest delta` records `SignerBasis::DeltaFrom { ... }` carrying
chain-depth cap (default 3, hard max enforced), cumulative-fingerprint
tracking (to last Fresh-basis signature), and required non-empty rationale
(minimum char count enforced at CLI + schema). Closes the laundering surface
where small carry-forwards could smuggle substantive change.

#### Process discipline: cross-ADR substrate-grep sub-routine

`docs/process.md` amended with Phase 3 cross-ADR surface check — prevents
naming collisions (e.g., F28-R2 where `attest oracle complete` would have
collided with `oracle complete` lifecycle verb). Caught at draft-time rather
than ship-time.

### Implementation (v0.1-rc shipping)

#### New crate

- `antigen-attestation` — Ratification schema + substrate-witness predicate
  evaluator. Separate workspace member; `serde_json` + `chrono` deps only.
  Includes Oracle schema (5-state lifecycle), SignerBasis enum (Fresh /
  DeltaFrom with anti-laundering fields), OracleRef tagged union (6 variants),
  Provenance struct, StateTransition event log. v0.0.1 name reserved on
  crates.io 2026-05-20 prior to this ship.

#### CLI families

- `cargo antigen attest scaffold | sign | check | delta | list | gc` — full
  immunity-sidecar lifecycle. `attest delta` enforces anti-laundering caps
  + rationale minimum at CLI layer.
- `cargo antigen tolerate scaffold | sign | check | list` — parallel family
  for tolerance ratifications via isomorphic schema.
- `cargo antigen oracle list | status | declare | complete | deprecate |
  retire | revoke` — full Oracle artifact-class lifecycle CLI (slice e per
  ADR-021).
- Removed: `attest migrate` (additive-only schema discipline obviates), `attest
  move` (error-path enforcement via gc + scan/audit yelling provides discipline
  through consequences; convenience verb unneeded).

#### Audit output extensions

- `EvidenceKind` enum (TypeSystemProof | Behavioral | SubstrateState) as
  third audit-output axis.
- `signature_strength` field per signer on audit output (git-trust default;
  text-stamp + crypto-signed as Tekgy verdict 2026-05-20).
- New hints: `discipline-predicate-passed-substrate-current`,
  `discipline-substrate-stale`, `discipline-predicate-passed-via-delta-chain`,
  `discipline-substrate-delta-chain-near-cap`, `tolerance-vibes-grade`,
  `oracle-in-draft`, `oracle-completion-attested`, `oracle-reference-malformed`,
  + others. Tier-honesty mapping documented in `docs/witness-tiers.md`.

#### Tambear adoption (Phase 4 shipped)

Tambear's sinh/cosh signed-zero discipline declared and substrate-witnessed
end-to-end against the v0.1-rc primitives. First-user adoption arc closed
against the originating motivation.

### A3.5 Onboarding sweep

#### Documentation (new)

- `docs/tutorial.md` — five-step narrative (declare → presents → scan → immune → audit);
  real scan/audit output throughout; teaching point on `.expect()` vs `body_contains_macro`
- `docs/fingerprint-grammar.md` — all 10 operators documented with behavior, examples,
  and receiver-rendering reference table; explicit tokenization-asymmetry warning
- `docs/troubleshooting.md` — all observable error categories from live scan+audit output;
  856 fingerprint match count explained; 39 parse failure categories; quick diagnostic table

#### Documentation (updated)

- `docs/fingerprint-grammar.md` — receiver-rendering reference table; `has_method` item-kind
  scope clarified (impl sites only); `"(self, Self)"` corrected throughout (was `"(Self, Self)"`)
- `docs/decisions.md` — ADR-010 concrete example + ratified declaration corrected to
  `"(self, Self) -> Self"` (Receiver token renders as `"self"`, not `"Self"`)
- `docs/tutorial.md`, `docs/scope.md`, `docs/where-to-look-for-antigens.md`,
  `docs/expedition/stdlib-seed-antigens.md`, `docs/expedition/case-study-determinism-class.md`
  — same receiver-form correction
- `README.md` — full narrative deep-draft replacing terse status block; what/what-not/
  vocabulary/workflow/architecture/tambear-origin/v0.1.0-scope/setup/license
- `docs/usage-patterns.md` — `#[antigen_tolerance]` decision tree + good/weak rationale
  examples + `until` field usage
- All four crate-level doc-comments improved; per-macro ADR references; stale "future"
  references removed; `antigen-fingerprint` positioned as canonical-implementation crate

#### Examples (new)

- `antigen/examples/descended_from.rs` — inheritance chain; scan produces state-7 inherited
  Presentation on `UseAfterFreeClass` with `inherited_from`
- `antigen/examples/antigen_tolerance.rs` — opt-out pattern; tolerance absorbs cross-reactive
  match; `until = "v0.2"` flags re-evaluation
- `antigen/examples/phantom_witness.rs` — phantom-type witness; audit classifies as
  `WitnessTier::FormalProof` with `PhantomTypeShapeRecognized` hint; `--format json` shows tier

#### Examples (updated)

- `antigen/examples/broken_witness.rs` — fingerprint narrowed to `matches("Looks*")`;
  workspace-wide cross-reactivity eliminated; docstring teaches the lesson

#### Engine

- `antigen-fingerprint`: symmetric canonicalization of `has_method` signature strings via
  proc_macro2 round-trip at parse time. User-natural Rust syntax works: `"(&mut self)"`,
  `"(& mut self)"`, and sloppy-whitespace variants all canonicalize to the same form and
  match the same signatures. Pre-A3.5 the engine required the spaced form `"(& self, ...)"`;
  that footgun is eliminated. (ATK-W6a-013 / ATK-W6a-013b; first real instance:
  tambear's PanickingInDrop, surfaced during A3.5 onboarding cross-check)
- `normalize_signature_canonical` now returns `Option<String>`; strict fail on malformed
  signature string (proc_macro2 parse error → `None`, not silent fallback to plain
  `normalize_ws(raw)`). Grounds: ADR-005 §1 sub-clause F — lenient fallback reintroduces
  the spacing asymmetry the fix exists to eliminate. Malformed patterns surface a
  compile-time parse error anchored at the offending literal. (Amendment 5 OQ1 ratified
  strict; bb22e56)

#### CLI

- `cargo-antigen`: `new` and `vaccinate` subcommands hidden from `--help` (not yet
  implemented; surface when A5 ships them)

#### Audit output

- `cargo antigen audit` human-readable output now distinguishes `FormalProof` and
  `Reachability` witnesses. Option A: per-tier sub-counts in the resolved summary
  ("N formal-proof", "N execution", "N declared (Reachability)"). Option B:
  confirmed-claims section parallel to warnings block, listing Execution+ tier claims
  with tier name and audit hint. Phantom-type witnesses now produce explicit positive
  feedback in human output. (ATK-A3-019)

#### Tests

- 240 passing, 31 ignored (up from 187/18-suites at rc.1); 21 suites
- ATK-W6a-013 inverted: was "must NOT match" (documenting bug); now "must match" (fix verified)
- ATK-W6a-013b added: tambear footgun — `has_method("drop", "(&mut self)")` now matches
  across natural/canonical/sloppy whitespace variants
- ATK-W6a-017 added: Self/self token-class distinction guard — `"(Self, Self) -> Self"` must
  NOT match `fn meet(self, other: Self)`; two positive controls included (receiver pattern matches
  receiver sig; static pattern matches static sig) (cd33c96)
- ATK-W6a-018 added: four malformed-signature cases verify the strict `None` path — unbalanced
  open paren, extra closing paren, unterminated string literal, raw backtick (bb22e56)
- ATK-A3-019 activated (was `#[ignore]`): asserts human audit output contains both
  "FormalProof" (Option B confirmed-claims) and "formal-proof" (Option A summary)

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

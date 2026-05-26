# Antigen — Roadmap

> User-facing trajectory document. What's shipped, what's planned, what's
> aspirational. Substrate-grounded confidence intervals; no firm calendar
> dates beyond what's actually committed.

**This is the adopter-facing roadmap.** Internal sweep planning lives in
[`sweeps/`](../sweeps/); ratified architecture lives in
[`docs/decisions.md`](decisions.md); expedition substrate lives in
[`docs/expedition/`](expedition/).

---

## Shipped (v0.2.0-alpha.2, unreleased)

### Supply-Chain Defense Family (ADR-025)

Makes the supply-chain trust boundary first-class structural memory. Eleven stdlib
antigens targeting the 2026+ threat landscape; adversarial-verified correctness
(ATK-SC-1-A, ATK-SC-2-A, ATK-SC-AUDIT-1 fixes).

- **`ContentHashMismatch`** — defends the chalk/debug/eslint-config (2025) content-
  replacement-at-fixed-version attack. Cargo.lock pins VERSION not CONTENT-HASH.
  Requires proactive first-attestation. **The NON-NEGOTIABLE antigen.**
- **`UnsandboxedProcMacro`** — external proc-macro dep executes in-rustc; higher risk
  than `build.rs`.
- **`UnpinnedDependency`**, `UnpinnedTransitiveDependency` (NARROW: direct dep
  with `*/?` for its own deps), `UnattestedDependencyInclusion`,
  `DependencyUpgradeWithoutDiffReview`, `AutoDependencyChainWithoutPinning`,
  `MaintainerChangeWithoutReattestation` (CI sequencing constraint: BEFORE `cargo update`),
  `SuddenDependencyExpansion`, `UnsandboxedBuildScript`, `PostInstallScriptInDependency`.
- **17 `AuditHint` variants** + `audit_supply_chain()` with combinator-aware
  `AnyOf`/`AllOf` predicate evaluation.
- **`antigen::supply_chain`** runtime: schema, witness, evaluate, manifest modules.
- **5 new `antigen_attestation::Leaf` variants** for supply-chain predicates.
- **3 examples**: `supply_chain_content_hash`, `supply_chain_unpinned`,
  `supply_chain_unsandboxed_proc_macro`.

### Convergent-Evidence Family (ADR-024)

First family of the temporal-arc cohort. Seven macros for backward-looking evidence
aggregation; adversarial-verified correctness (ATK-CE-1, ATK-CE-2, ATK-CE-3-B fixes).

- **`#[diagnostic]`** — clinical-medicine grounding (differential diagnosis). Counts
  distinct `WitnessClass` CATEGORIES for `min_independent` (not raw witness count).
  Parse-time error if `min_independent` exceeds distinct categories.
- **`#[clonal]`** — B-cell clonal expansion analog. `SeedKind::Fixed(_)` is COMPILE ERROR.
- **`#[igg]`** — IgG affinity-matured evidence with temporal span + unique reattestation
  count enforcement (ATK-CE-3-B: unique signers, not raw count).
- **`#[crossreactive]`**, **`#[polyclonal]`**, **`#[monoclonal]`**, **`#[adcc]`** —
  marker + structural primitives.
- **`antigen::WitnessClass`** enum (6 variants) — public, re-exported.
- **`antigen::SeedKind`** enum (4 variants, `Fixed(u64)` rejected) — public, re-exported.
- **11 `AuditHint` variants** + `audit_convergent_evidence()`.
- **`ScanReport::convergent_evidences`** (additive, serde compat).
- **3 examples**: `convergent_diagnostic`, `convergent_clonal`, `convergent_igg`.
- **Trybuild fixtures** for compile-time enforcement (CE-1 class-collapse, CE-2 fixed seed).

---

## Shipped (v0.2.0-alpha.1, unreleased)

### Deferred-Defense Family (ADR-023)
- **`#[anergy]`** — deferred-but-muted posture; `until` REQUIRED; 20-char minimum reason; aging escalation
- **`#[immunosuppress]`** — surgical silencing with duration cap enforced at parse time (default 90d)
- **`#[poxparty]`** — intentional exposure with structural cfg-gate isolation; `antigen-poxparty` feature
- **`#[orient]`** — see-also context; all fields optional; lightest-weight deferred-defense primitive
- `audit_deferred_defenses()` function + `DeferredDefenseAuditReport`; feeds `cargo antigen defer status`
- 16 new `AuditHint` variants; `ScanReport::deferred_defenses` field (additive, serde compat)

---

## Shipped (v0.1.0-rc.3)

The core vocabulary, scan + audit tooling, substrate-witness pipeline, Oracle artifact lifecycle, and team-coordination tooling are all live across 5 crates on crates.io (`antigen`, `antigen-macros`, `antigen-attestation`, `antigen-fingerprint`, `cargo-antigen`).

### Vocabulary + macros
- **Five macros**: `#[antigen]`, `#[presents]`, `#[immune]`, `#[descended_from]`, `#[antigen_tolerance]`
- **Cross-cutting attestation parameter**: `attested = (who, allowed_types, why, scope)` per ADR-020
- **Phantom-type witness recognition** (ADR-013) — `Witnessed<T,W>`, `typewit::TypeEq`, hand-rolled `PhantomData<T>` shapes recognized at FormalProof tier
- **Cross-crate identity** — `canonical_path` at `name@version` granularity (ADR-017); cross-crate `#[descended_from]` propagation in v0.2

### CLI surface (`cargo antigen ...`)
- **`scan`** — workspace-wide scanning, item-identity matching (W3), fingerprint detection, tolerance recognition, orphaned-tolerance reporting
- **`audit`** — `WitnessTier` gradient (None / Reachability / Execution / FormalProof) per ADR-005 Amendment 3; substrate-witness pipeline wired end-to-end via the rc.2 hotfix
- **`attest`** subcommands — manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019): `scaffold`, `sign`, `check`, plus design-phase `list`, `delta`, `gc`
- **`tolerate`** subcommands — manage tolerance-ratification sidecars (ADR-019 §tolerance tier)
- **`oracle`** subcommands — manage Oracle artifact-class records (ADR-021 §D3): `list`, `status`, `declare`, `complete`, `deprecate`, `retire`, `revoke`
- **`--version`** flag (rc.3) — introspects the installed `cargo-antigen` version for tooling integration

### Fingerprint engine
- **Fingerprint grammar v1** — seven item-level operators (`item`, `name`, `variants`, `has_method`, `attr_present`, `doc_contains`, `body_contains_macro`) plus composition (`all_of`, `any_of`, `not`); proc_macro2 canonicalization per ADR-010 Amendment 5

### Substrate-witness machinery (ADR-019)
- **`#[immune(X, requires = <predicate>)]`** form with substrate-witness leaves: `signers(required = [...])`, `fresh_within_days(N)`, `ratified_doc(path = ...)`, `oracles_complete(files = [...])`, `signed_trailer(...)`
- **Predicate combinators** — `all_of`, `any_of`, `not`
- **Three-tier SignatureStrength** (per ADR-019 v1+3): WORKSPACE-LOCAL, OIDC-IDENTITY, KEY-SIGNED with explicit audit-time reporting
- **Sidecar discovery** — `.attest/<Antigen>.json` co-located with declaration

### Oracle 5-state lifecycle (ADR-021)
- **`#[oracle]` artifact-class** with full state machine: Draft → Complete → Deprecated/Retired/Revoked + Reopened
- **Per-Oracle signers + stewards + provenance trail**
- **Audit integration** — `oracles_complete(...)` predicate checks Oracle state at audit time

### Documentation
- **Adopter-facing**: README, quickstart, tutorial, examples-guide, witness-tiers, usage-patterns, fingerprint-grammar, where-to-look-for-antigens, anti-patterns, troubleshooting
- **Conceptual**: concepts, structural-memory, postures, immune-system-primitive-map, vision-pitch, scope, origin narrative
- **Process**: decisions (ADRs), process, glossary, roadmap, contributing
- **LLM-collaborator protocol**: `for-llm-collaborators.md`

See [`CHANGELOG.md`](../CHANGELOG.md) for the full release manifest.

---

## Path to 0.1.0 (drop the `-rc.N` suffix)

`0.1.0-rc.3` is a release candidate: the API shape we believe will be
0.1.0 final, pending validation against real adoption. Promoting to
`0.1.0` (no rc qualifier) means committing to:

- **Schema stability** (additive-only per ADR-021)
- **Five leaf primitives sealed at use-site** (`signers`, `ratified_doc`,
  `signed_trailer`, `oracles_complete`, `fresh_within_days`)
- **Three combinators closed** (`all_of`, `any_of`, `not`)
- **Three-axis audit output frozen** (`WitnessTier × AuditHint ×
  EvidenceKind`) + `signature_strength`
- **Five-state Oracle lifecycle frozen** (Draft / Complete / Deprecated
  / Retired / Revoked)
- **CLI subcommand surface frozen** (`scan / audit / attest * /
  tolerate * / oracle *`)
- **Sidecar location conventions frozen** (`.attest/<AntigenName>.json`
  + `.antigen/oracles/<OracleId>.json`)

### Trinity of self-adoption (the 0.1.0 readiness gate)

Rather than wait for a non-us external adopter as a gate, antigen
proves its shape via **three independent self-adoption streams** that
each exercise the WHOLE primitive stack on different stress profiles:

1. **Layer 1 — antigen on its own source.** Add `#[antigen]` declarations
   for failure-classes antigen DEFENDS AGAINST in its own code
   (infinite-recursion in predicate walker, path-traversal in sidecar
   read, silent arithmetic overflow in chain_depth, etc.); use
   `#[immune(...)]` for the spots already addressed. Add Oracle
   declarations for our own design decisions; coordination claims with
   multi-signer `requires`; discipline-attestation for schema
   commitments. The WHOLE primitive stack against ONE codebase.
   Source-code-as-canonical-reference: every defensive declaration
   doubles as a worked example.
2. **External adopters.** Independent projects built on antigen via the
   public API (subprocess composition or library link per ADR-002)
   provide leg-2 evidence: antigen's primitives hold up to real downstream
   needs. External adopters are tracked in their own release histories,
   not antigen's roadmap — antigen's promotion gate considers their
   substrate as evidence of API durability without claiming their
   milestones as antigen's.
3. **Tambear discipline + numerical-correctness adoption.** Tambear's
   Phase 4 work (sinh/cosh signed-zero) extends to more numeric
   functions + more disciplines + Oracle lifecycle for the numerics
   specs. Cross-crate trust extension between tambear → antigen at the
   external-adopter API. The WHOLE stack against cross-project
   adoption + a real numerical-correctness domain.

Each leg of the trinity exercises every primitive (predicate /
audit / oracle / lifecycle / signers / coordination / discipline /
feature-specific defenses) but on different substrate. **Three
independent witnesses of "yes this primitive holds up."** Cross-crate
machinery only exercises under real external adopters; it's not theoretical.

### Additional 0.1.0 readiness items

Alongside the trinity:

1. **T4 resolved** (compound evidence overclaim surface) — when
   immune+tolerance attestations land on the same site, can we report
   that without users misreading "two attestations = stronger
   evidence"? Aristotle F11 flagged this. Either ship a resolution or
   explicitly document the surface as "do not depend on
   additive-evidence interpretation."
2. **T6 resolved** (severity-class scout substrate-grep) — was anything
   in ADR-008 Amendment 1 about severity ever wired into scan output?
   Quick mechanical check; if YES we document, if NO we defer to v0.2
   explicitly.
3. **A "production deployment" guide** in `docs/` — how does a team
   actually integrate antigen into their release cadence? Currently
   tutorial covers "how the primitive works"; a deployment guide
   covers "how to integrate this into CI / PR review / release flow."
4. **Any rc-cycle bug fixes** — anything the trinity surfaces that
   reveals breaking-change pressure gets resolved before 0.1.0 ships.
   If breaking changes are needed, they roll into rc.2.
5. **README install snippet** — current `cargo add antigen` resolves
   to the v0.0.1 placeholder (since rc.N is pre-release per semver).
   Either accept this until 0.1.0 final ships (so `cargo add antigen`
   works without flags), or document `cargo add antigen --version
   "0.1.0-rc.N"` explicitly in README. Current decision: accept-as-is;
   resolved naturally when 0.1.0 ships.

### Realistic timeline

The trinity work is days-scale, not months-scale. The three legs can
build in parallel:

- Layer 1 source-dogfood: days to first declarations; sessions to full
  coverage
- External adopter feedback: ongoing as adopters exercise antigen's API
  surface and surface real-world friction signal
- Tambear discipline expansion: ongoing as tambear's numerics team
  hits more failure-classes worth attesting

If all three converge without surfacing breaking changes + the
additional items close, we promote rc.N → 0.1.0. If breaking changes
are needed, they ship as rc.N+1. The rhythm is "build + use + cycle
rc's as needed; promote when shape is stable across all three witnesses."

---

## Planned for v0.2

The v0.2 cycle is structured around an **architectural-posture-shift ratification event** — 10 ADRs ratifying together (one process.md amendment alongside) committing antigen to a **comprehensive immune-system stdlib** rather than the narrower v0.1 framing.

This shift is grounded in the **generation-inspection asymmetry** that characterizes modern dev (humans + LLM agents + human-LLM teams generate code faster than any can inspect). Antigen's role is **memory-to-structure transformation**: convert passive memory (TODOs, comments, ADRs, Slack decisions) into co-native structure (compile-checked, audit-surfaced, stale-aware, sign-required) that surfaces itself. See [`docs/expedition/the-comprehensive-vision.md`](expedition/the-comprehensive-vision.md) for the full synthesis.

### Ratified architectural commitments (v0.2 ceremony)

- **AMEND-ADR-002** (compose-or-compete amended) — antigen owns surfaces where cohesion-within-antigen serves adopters better; composition stays the default for external expertise + low integration cost
- **AMEND-ADR-003** (biology dual-role) — the immune-system metaphor is BOTH a teaching tool AND a systematic discovery framework for stdlib coverage; each unused immune-system component is a research-arc prompt
- **AMEND-ADR-006** (recognition-not-design split) — recognition discipline for ADOPTER extensions; research-driven discipline for STDLIB growth (substrate-citable from postmortems / literature / training-data / predictive analysis / biological-component-mapping)
- **NEW-ADR-022** (stdlib-vs-extension two-disciplines) — formalizes the dual architecture. Extension contract = first-class public API (semver-stable). Stdlib growth = research-driven, deliberately comprehensive
- **NEW-ADR-023** (deferred-defense family) — `#[anergy]`, `#[immunosuppress]`, `#[poxparty]`, `#[orient]` as primitives; loudness-as-discipline; 5-mode matrix; cfg-gated structural isolation for `#[poxparty]` via Cargo feature env var
- **NEW-ADR-024** (convergent / recurrent / prescriptive families) — three sibling primitive families covering 21 new macros
- **NEW-ADR-025** (supply-chain defense family) — `ContentHashMismatch`, `UnsandboxedProcMacro`, `UnpinnedTransitiveDependency` (narrow definition); biology grounding via distributed-boundary-innate-immunity (multi-cell-type system)
- **NEW-ADR-026** (VCS-information-loss family) — `ForcePushErasingHistory`, `RollbackWithoutTriageCommit`, etc.; central cognate is measles-induced immune amnesia (catastrophic memory-loss); rollback-as-triage discipline
- **NEW-ADR-027** (mucosal boundary taxonomy + mapping discipline) — `#[mucosal_delegate]` primitive; `cargo antigen mucosal-map` tool; v0.2 covers filesystem / env-vars / shell-args; WebSocket / CI-CD deferred to v0.3+
- **NEW-ADR-028** (substrate-alignment vs functional-correctness antigen-category) — first-class category metadata on antigen declarations; shapes witness type, audit layer, lifecycle phase, responder role
- **process.md amendment** — Phase-3 sub-routine requiring each ADR to specify enforcement-tier × enforcement-scope via §Enforcement-Surface table (resolves the cross-ADR enforcement-mechanism-ambiguity 3rd-instance convergence finding)

### Macro family expansions (~50+ primitives total when fully shipped)

Per the comprehensive vision §7, v0.2 ships major chunks of the macro vocabulary:

- **Honest-debt / deferred-defense family**: `#[anergy]`, `#[immunosuppress]`, `#[poxparty]`, `#[orient]`, `#[vaccinate]` — loudness-as-discipline; aging escalation; structural isolation
- **Convergent evidence family**: `#[diagnostic]`, `#[clonal]`, `#[igg]`, `#[crossreactive]`, `#[polyclonal]`, `#[monoclonal]`, `#[adcc]` — independent-modality convergence, repeated derivation, historical re-attestation
- **Recurrent emergence family**: `#[itch]`, `#[recurrence_anchor]`, `#[crystallize]`, `#[chronic]`, `#[saturate]`, `#[strand]` — below-threshold notice, threshold-crossing, ideation maturation; multi-layer subsystem with future git/MemPalace/chat integration
- **Prescriptive (work-orchestration) family**: `#[panel]`, `#[ddx]`, `#[rx]`, `#[triage]`, `#[refer]`, `#[biopsy]`, `#[culture]`, `#[titer]`, `#[quarantine]` — substrate-resident team coordination (Asana-as-substrate); multi-axis triage (priority + level-of-care + treatment-kind + outcome-acceptance)
- **Supply-chain defense family** (Arc 9 — URGENT given chalk/debug/eslint-config landscape)
- **VCS-information-loss family** (Arc 10 — rollback-as-triage)
- **Mucosal boundary taxonomy** (Arc 11)

### Tooling

- **`#[cfg(feature = "antigen-poxparty")]`** structural isolation with proc-macro env-var check; feature NOT in default set
- **`cargo antigen mucosal-map`** — enumerate input boundaries; map to mucosal taxonomy
- **`cargo antigen verify content-hash`** (supply-chain defense)
- **`cargo antigen migrate categories`** — soft-default migration for v0.1 antigen carryover (v0.2.1+ polish)
- **Body-level fingerprint operators** via ast-grep subprocess (per ADR-015)
- **`cargo antigen new`** — scaffold a new antigen declaration with guided prompts
- **`cargo antigen vaccinate`** — apply known immunity pattern across a structural family with human review

### Engine refinements

- **Engine-canonicalization for operators beyond `has_method`** — ADR-010 Amendment 5 pre-tokenization pattern extended to other string-comparison operators where tokenization asymmetries surface in practice (recognition-not-design: lands when substrate-grounded)

### Deferred from v0.1-rc.1 — warm handoff substrate

Items the rc.1 work surfaced + deliberately scoped to v0.2 or later.
What we know going in:

- **T2: CODEOWNERS interop UX** — `signers(required = [...])` accepts
  literal names today. v0.2 adds `required_role` for CODEOWNERS-style
  role resolution. Open question is whether to (a) parse the project's
  CODEOWNERS file at audit time and resolve role names against it, or
  (b) just accept role strings as opaque labels and let the team's own
  tooling resolve them. Forge-side coupling (a) is convenient but
  couples antigen to GitHub specifically; (b) is forge-agnostic but
  shifts ergonomic burden to adopters. Probably ship (b) first, add (a)
  as an opt-in feature flag if pressure surfaces.

- **T5: Leaf-contract enforcement mechanism for witness-provider crates** —
  v0.1 sealed leaf set is structurally required per F7 + T1-R. v0.2+
  ADR specifies leaf-contract (deterministic / terminating /
  side-effect-bounded / declared-tier) + default-cap at Reachability +
  workspace-config opt-in for higher tiers. Three enforcement
  mechanisms to choose between: WASM sandbox (robust, expensive),
  `no_std` + restricted-deps build-time check (pre-screen only),
  subprocess isolation with timeout + memory cap (runtime, medium
  cost). Adversarial T1-R confirmed docs-only insufficient — must be
  ACTUAL enforcement, not just contract documentation. The choice
  shapes which kinds of leaf-provider crates become possible.

- **T7 / FA-2: Fingerprint-scheme evolution across version bumps** —
  when antigen ships v0.2 with a refined fingerprint scheme, existing
  sidecars with `signed_against_fingerprint` from v0.1 become
  stale-mismatched. Need cross-version migration story. Options:
  audit treats v0.1 fingerprints as legacy + emits hint;
  `attest migrate-fingerprints` CLI rebases pins to new scheme;
  schema carries `fingerprint_scheme_version` field. Aristotle F12
  worked this; needs concrete-pressure trigger (first fingerprint
  scheme bump) to ratify.

- **T8 / FA-5: descended_from predicate inheritance** — can a
  consuming crate declare `#[descended_from = "A::X"]` but supply a
  WEAKER `requires` predicate than A's? Tier-honesty implications.
  Aristotle F10 + adversarial FA-5 worked this; resolution likely
  uses Eiffel-style variance rules (precondition-weakening prohibited;
  postcondition-strengthening allowed). Scout's Eiffel rhyme already
  surfaced in academic-context.md as candidate design. Lands when
  cross-crate descended_from sees real adoption pressure.

- **DSSE envelope + Sigstore identity-bound signatures (v0.4+ target)** —
  `Signer.signature: Option<Signature>` slot exists today; activation
  via DSSE pre-authentication-encoding (don't roll our own envelope —
  PAE is non-obvious) + Sigstore Fulcio + Rekor transparency log
  follows the notary-institution 800-year design arc (git-trust →
  OIDC + transparency log). Compose-don't-compete with the existing
  ecosystem.

- **Lifetime on discipline claims** — `permanent | temporal(cadence) |
  transitional(condition)`. v0.1 ships with implicit "permanent"
  semantics; v0.2 adds explicit lifetime so disciplines that should
  re-attest periodically (e.g., security review every 90 days) can
  express that structurally. Scout flagged this in expedition substrate.

- **`--prioritized` flag for `attest list --pending`** — annotation-
  fatigue mitigation. Sort pending attestations by antigen-severity +
  fingerprint-confidence so adopters see the load-bearing items first.
  Cross-domain rhyme from software-ergonomics literature (scout S4).
  Useful when teams have many in-flight attestation surfaces.

- **TUF k-of-n threshold signatures** — `signers(required_threshold =
  K, candidates = [...])`. Cross-domain analog from TUF specification;
  scout S4 + CAP-theorem framing makes this a principled extension of
  current `required = [...]` shape. Useful when teams want "any 3 of
  these 5 reviewers" rather than "all of these 3."

- **T3: `discipline_doc` field dual-jobs separation** — aristotle F9
  frontier-flag. Current field does Job 1 (canonical reference) AND
  Job 2 (review-grounded binding). Future amendment might split into
  `canonical_reference` + `review_grounded` so the claims can vary
  independently. Deferred until adoption substrate accumulates enough
  to tell us whether the dual-jobs actually need to vary in practice.

- **Layer 1 source dogfood + Layer 4 ADR-as-Oracle** — antigen using
  antigen on antigen's own code (Layer 1) and treating ADRs as
  Oracles (Layer 4). Layer 1 is a 0.1.0-readiness item; Layer 4 is
  a deeper recursion that grows naturally once Layer 1 + adopter
  validation are established. Initial seed: 8 empirical fail-classes from antigen's
  own git fix history (UnanchoredGitignorePattern,
  MsrvAccidentallyRaisedByTransitiveDep, NonIdempotentReleaseStep,
  CratesIoPublishBlockerMissingMetadata, BrokenIntraDocLink,
  SilentCliCommandFailure, UnboundedRecursionInProcMacro,
  MacroEmittedSubstrateNotSeenBySourceScan) are catch-once-build-antigen
  candidates per the internal-tooling memory. Layered approach:
  identified-spots first, then walk the codebase for additional sites,
  then `examples/` directory for curated demos of every primitive
  (Tier-A in-source for real fail-classes; Tier-B in-source for
  natural educational coverage; Tier-C in `examples/` for primitives
  that don't fit antigen's source naturally).

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
- **Cross-crate scan reachability (ADR-001 C7 activation path)** —
  what `cargo antigen scan --include-deps` does *today* (v0.2) is
  scan each dependency crate **independently**: every dep's antigens
  appear in their own `dep_reports` entry, with `canonical_path`
  stamped to `name@version` (ADR-017 identity model). It does **not**
  yet do cross-crate *matching*: a dependency's `#[presents]` site is
  not resolved against the consuming workspace's `#[immune]`
  declarations, and a fingerprint declared in crate A is not
  synthesized against items in crate B. Each crate's report stays its
  own bag of antigens.

  The activation path — **cross-crate `addresses()` matching +
  cross-crate fingerprint synthesis** — is what realizes ADR-001
  Amendment 1 C7's commitment ("cross-crate consumption is in-scope for
  v1+") into the scanner. Two realizations, smallest-first:
  - *Workspace-internal cross-crate* (the dogfood case): scan the whole
    antigen workspace as one root so every in-repo `#[antigen]`
    declaration is in fingerprint-scope for every in-repo item. This is
    what makes antigen-on-antigen self-scanning reach instances that
    live in a different workspace crate than their declaration (e.g. an
    antigen declared in `antigen::stdlib::dogfood` matching a
    `#[presents]`/fingerprint instance in `antigen-macros`).
  - *True cross-registry-crate* (the full ADR-001 C7 build): apply a
    dependency crate's declared fingerprints to the consumer's items
    via the `cargo metadata`-driven dep walk, honoring the ADR-017
    `name@version` trust boundary.

  Deferred from v0.1 by the Sweep A3 scope-lock (deliberate, not
  unbuilt). Reopening it is an ADR-scope decision, not an incremental
  scanner tweak — the per-crate-isolation model is load-bearing for the
  current trust-boundary semantics. Tracked because the commitment is
  foundational (ADR-001 C7) and the dogfood self-scan case makes the
  workspace-internal realization concretely useful before v1.

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

*Roadmap originally authored 2026-05-12 as Phase 4 deliverable of Sweep A3.5
(Onboarding). Substantially expanded 2026-05-22 to reflect v0.2 comprehensive
vision direction, the 10 ADRs ratifying together as architectural-posture-shift
event, antigen rc.3 published, and the layered dogfood approach (Tier-A/B/C).
External adopter validation milestones are tracked in those projects' own
release histories — antigen's roadmap notes them as adopter-evidence under
the trinity-of-self-adoption section, not as antigen deliverables. Subject to revision as substrate matures. The trajectory is
real; the destination is recursive.*

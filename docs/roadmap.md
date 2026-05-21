# Antigen — Roadmap

> User-facing trajectory document. What's shipped, what's planned, what's
> aspirational. Substrate-grounded confidence intervals; no firm calendar
> dates beyond what's actually committed.

**This is the adopter-facing roadmap.** Internal sweep planning lives in
[`sweeps/`](../sweeps/); ratified architecture lives in
[`docs/decisions.md`](decisions.md); expedition substrate lives in
[`docs/expedition/`](expedition/).

---

## Shipped (v0.1.0-rc.1)

The core vocabulary, scan + audit tooling, and discipline substrate are
all live:

- **Five macros**: `#[antigen]`, `#[presents]`, `#[immune]`,
  `#[descended_from]`, `#[antigen_tolerance]`
- **`cargo antigen scan`** with workspace-wide scanning, item-identity
  matching (W3), fingerprint detection, tolerance recognition, and
  orphaned-tolerance reporting
- **`cargo antigen audit`** with the `WitnessTier` gradient
  (None / Reachability / Execution / FormalProof) — tier-honest
  reporting per ADR-005 Amendment 3
- **Fingerprint grammar v1** — seven item-level operators (`item`,
  `name`, `variants`, `has_method`, `attr_present`, `doc_contains`,
  `body_contains_macro`) plus composition (`all_of`, `any_of`, `not`);
  proc_macro2 canonicalization per ADR-010 Amendment 5
- **Phantom-type witness recognition** (ADR-013) — `Witnessed<T,W>`,
  `typewit::TypeEq`, hand-rolled `PhantomData<T>` shapes recognized at
  FormalProof tier with explicit human-readable output
- **Cross-crate identity infrastructure** — `canonical_path` at
  `name@version` granularity (ADR-017); cross-crate `#[descended_from]`
  propagation and registry source-walking land in A3 (v0.2)
- **Documentation**: tutorial, fingerprint grammar reference,
  usage-patterns cookbook, where-to-look conventions, troubleshooting,
  origin narrative, comprehensive ADR substrate

See [`CHANGELOG.md`](../CHANGELOG.md) for the full v0.1.0-rc.1 manifest.

---

## Path to 0.1.0 (drop the `-rc.N` suffix)

`0.1.0-rc.1` is a release candidate: the API shape we believe will be
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
2. **Camp build.** Per-project Rust crate (`<project>/camp/`) where
   each campsite is a module declaring an Oracle with required
   signers + state machine. `cargo check` IS the team-status query.
   Camp's whole purpose is dogfooding antigen for team coordination —
   the WHOLE stack against multi-crate workflow + multi-role signers +
   real lifecycle state transitions. Adds the cross-crate dimension
   (camp crate depends on antigen crate from crates.io).
2. **Tambear discipline + numerical-correctness adoption.** Tambear's
   Phase 4 work (sinh/cosh signed-zero) extends to more numeric
   functions + more disciplines + Oracle lifecycle for the numerics
   specs. Cross-crate trust extension between tambear → antigen at the
   external-adopter API. The WHOLE stack against cross-project
   adoption + a real numerical-correctness domain.

Each leg of the trinity exercises every primitive (predicate /
audit / oracle / lifecycle / signers / coordination / discipline /
feature-specific defenses) but on different substrate. **Three
independent witnesses of "yes this primitive holds up."** Cross-crate
machinery only exercises under camp + tambear; it's not theoretical.

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
- Camp build: weeks to MVP; depends on adoption shape but the
  underlying primitives exist
- Tambear discipline expansion: ongoing as tambear's numerics team
  hits more failure-classes worth attesting

If all three converge without surfacing breaking changes + the
additional items close, we promote rc.N → 0.1.0. If breaking changes
are needed, they ship as rc.N+1. The rhythm is "build + use + cycle
rc's as needed; promote when shape is stable across all three witnesses."

---

## Planned for v0.2

Items committed by structural necessity (ADR-007 anti-YAGNI:
structurally-guaranteed-need). Each lands when its substrate matures;
ordering may shift.

- **Body-level fingerprint operators** via ast-grep subprocess
  (per ADR-015). Enables fingerprints that match against function
  bodies, not just item-level signatures. Closes the recall-tuned-filter
  gap for failure-classes whose structural pattern lives in
  implementation, not declaration.
- **`cargo antigen new`** — scaffold a new antigen declaration with
  guided prompts. Tooling for first-time-adopter ergonomics; reduces
  the friction of authoring well-formed fingerprints from scratch.
- **`cargo antigen vaccinate`** — apply known immunity pattern across
  a structural family. Bulk-applies `#[presents]` and `#[immune]`
  annotations to sites matching a known antigen's fingerprint, with
  human review of the proposed change-set.
- **Engine-canonicalization for operators beyond `has_method`** — the
  ADR-010 Amendment 5 pre-tokenization pattern extends to other
  string-comparison operators where tokenization asymmetries surface
  in practice (recognition-not-design: lands when substrate-grounded).

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

- **Camp skill build** — antigen-native team coordination crate;
  per-project Rust crate where campsites are modules with Oracle
  declarations. Designed; not yet built. Will become a 0.1.0
  promotion-gate input (one of the trinity-of-self-adoption legs).
  Design substrate at `~/.claude/skills/camp/SKILL.md` (locally) +
  prior session captured the architecture decisions.

- **Layer 1 source dogfood + Layer 4 ADR-as-Oracle** — antigen using
  antigen on antigen's own code (Layer 1) and treating ADRs as
  Oracles (Layer 4). Layer 1 is a 0.1.0-readiness item; Layer 4 is
  a deeper recursion that grows naturally once Layer 1 + camp are
  established. Both deepen the dogfood story significantly.

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

*Roadmap authored 2026-05-12 as Phase 4 deliverable of Sweep A3.5
(Onboarding). Subject to revision as substrate matures. The trajectory
is real; the destination is recursive.*

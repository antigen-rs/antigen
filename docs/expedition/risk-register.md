# Antigen — Risk Register

> Adversarial-perspective catalog of what could kill the project. Captured pre-team
> so the antigen JBD team starts with explicit awareness of failure modes rather than
> discovering them mid-sweep. Each risk has: likelihood, severity, signals that it's
> happening, and mitigation strategies.
>
> This is a living document. As the team works, new risks surface and existing risks
> resolve, intensify, or transform. Treat it as substrate, not prophecy.

---

## How to read this register

Each risk has four fields:

- **Likelihood** — best estimate of probability over the project's adoption arc
  (Low / Medium / High). Independent of severity.
- **Severity** — impact if the risk materializes (Low / Medium / High / Existential).
  Existential = project dies or is irrecoverably damaged.
- **Signals** — concrete observations that would indicate this risk is materializing.
- **Mitigation** — strategies for preventing or recovering from the risk.

Risks are grouped by source: **adoption-killing** (people don't adopt), **engineering-
killing** (the tool doesn't work), **ecosystem-killing** (the Rust community rejects
or competes), and **project-killing** (the team or substrate breaks down).

---

## Adoption-killing risks

### Risk A1: Boilerplate burden too high

**Likelihood**: Medium. **Severity**: High.

The 60-second-declaration target may not be reachable without significant macro
work. If declaring an antigen takes 5 minutes (writing fingerprint, witness, etc.),
adopters give up.

**Signals**:
- Early adopter feedback: "writing antigens feels like writing tests AND lints AND
  docs all at once"
- Time-to-first-antigen exceeds 30 minutes for new users
- `antigen-stdlib` adoption is low because importing existing antigens is preferred
  to writing new ones
- Issue reports complaining about the macro syntax verbosity

**Mitigation**:
- `cargo antigen new <name>` aggressively scaffolds — fills in defaults, suggests
  fingerprint based on context, generates witness stubs from existing patterns
- IDE integration (rust-analyzer plugin) provides hover-suggestions and quick-fixes
- The seed antigens in `antigen-stdlib` cover common cases so users don't have to
  write antigens to get value
- Documentation leads with Layer 1 (minimum-viable) examples per ADR-009; teaches
  enrichment progressively
- User-experience review at every API decision (per ADR-008)

### Risk A2: Stdlib library too sparse

**Likelihood**: Medium. **Severity**: High.

If `antigen-stdlib` only covers 5-10 antigens at v1.0, users have to write their
own for everything beyond the trivial. Most won't.

**Signals**:
- Most user-written antigens duplicate patterns that should have been in stdlib
- The community antigen ecosystem fragments into many overlapping niche libraries
  rather than converging on stdlib
- Stdlib release cadence is slow (months between additions)

**Mitigation**:
- Sweep A5 (stdlib first 20-50 antigens) is high priority
- Aggressive community-contribution intake — issue templates make it easy to
  propose new stdlib entries
- The seed catalog at `stdlib-seed-antigens.md` is the v0.1 floor; expansion to
  50+ entries is the v0.2 target
- Review prior-art surveys regularly to surface candidates from other ecosystems
  (CWE, FindBugs, ESLint rules)

### Risk A3: Slow scan performance

**Likelihood**: Low (with v1 grammar). **Severity**: High.

If `cargo antigen scan` takes more than 5 seconds on a moderate workspace, it
doesn't get run in CI. Becomes documentation rather than enforcement.

**Signals**:
- Scan times exceeding 10 seconds on workspaces under 100k LoC
- Performance complaints in early-adopter feedback
- CI integration patterns that skip antigen scan on PRs (only run on main)
- Profiling reveals visitor pattern hot-spots

**Mitigation**:
- Per ADR-010, the v1 grammar uses syn::parse2 + visitor pattern with O(n × m)
  complexity; explicit performance bound documented
- Cargo's incremental compilation applies; antigen scan caches per-file results
- Fingerprint AST depth caps (initial: 10) prevent pathological complexity
- Performance regression tests on the antigen-stdlib catalog ensure scan stays fast
  as the library grows
- v2+ may introduce more sophisticated indexing (trigram, n-gram) for very large
  workspaces

### Risk A4: False-positive noise (autoimmunity)

**Likelihood**: Medium. **Severity**: High.

If scan flags many sites that are legitimately not vulnerable, users disable it.
"Antigen is too noisy" becomes the project's reputation.

**Signals**:
- Issue reports about false positives in antigen-stdlib
- High volume of `#[antigen_tolerance]` markers in adopter codebases
- "Antigen disabled in CI" pattern in adopter `Cargo.toml` configs
- Adoption stalls early; no community antigen contributions

**Mitigation**:
- Fingerprint precision is non-negotiable in stdlib curation. Each stdlib antigen's
  fingerprint must match real instances at >90% precision (low false-positive rate)
- Community feedback loops: adopters report false positives; stdlib refines
- The `antigen-tolerance` mechanism is documented as a legitimate exit hatch with
  clear "use sparingly" guidance
- Sweep A5 (stdlib quality) explicitly tests each antigen's fingerprint against
  curated positive AND negative example codebases

### Risk A5: Cultural rejection due to "yet another tool"

**Likelihood**: Medium. **Severity**: Medium.

The Rust ecosystem has many tools. Adopters may not have appetite for one more,
especially one positioned as composing existing tools — "if it's just composition,
why don't I just use clippy + proptest directly?"

**Signals**:
- Reception at Rust conferences / blog posts is lukewarm
- Issue tracker has frequent "why not just use X" questions
- Adoption among prominent Rust projects is slow

**Mitigation**:
- The composition message is explicit in the README, vision-pitch, and academic-
  context docs. Antigen is NOT yet-another-lint; it's the composition layer
- The case study (`case-study-determinism-class.md`) shows what antigen catches
  that no existing tool catches alone
- Phase 5 ecosystem outreach (post-v0.1) explicitly engages with tool authors
  rather than just users
- If skepticism is widespread, antigen's value proposition adapts — the "structural
  failure-class memory at ecosystem level" message may need different framing for
  different audiences

### Risk A6: Multi-contributor workflow friction

**Likelihood**: Medium. **Severity**: Medium.

*Surfaced by adversarial during Sweep A1 (ATK-008-1); recommended for register
addition by naturalist convergence-check 1.*

In a multi-contributor codebase, branches are often in mid-progress. A branch
introducing a new presentation site without yet declaring its immunity is a
**legitimate intermediate state**, not a defect. If `cargo antigen scan` is
configured to fail CI on any unaddressed presentation, contributors hit
hard-stops on transient states — and the project gets blamed for the friction.

This risk is structurally similar to clippy's `-D warnings` flag friction in
mid-stream development. Contributors learn to disable rather than navigate.

**Biological framing** *(added 2026-05-07 per naturalist H7 finding)*: this
is not just a workflow accident — it is the **immune lag phase** mapped onto
code-development. The biological immune system has a mandatory lag of
5-7 days for primary antibody response and 1-3 days for secondary (memory-
boosted) response. The system *cannot* produce instant immunity to a new
presentation; the lag is physiological, not pathological. Antigen's current
zero-tolerance default for the presentation→immunity gap is biologically
backwards. The lag is not a bug to suppress; it is a phase to design for.
See inventory v0 H7 for the full chain.

**Signals**:
- Issue reports about CI failures on intermediate-state branches
- `#[antigen_tolerance(reason = "wip")]` markers proliferating in active
  development branches
- Project READMEs that recommend disabling `cargo antigen audit` in CI for
  feature-branch builds
- Adopters describing antigen as "noisy" not because of false-positives but
  because of state-of-progress flagging

**Mitigation**:
- `cargo antigen audit` severity (warn vs error) must be configurable per ADR
  amendment (ATK-008-1 recommends amending ADR-008's enforcement clause). Default
  to warn for unaddressed presentations; error only when explicit opt-in.
- Audit output should distinguish "presentation lacks immunity" (potentially
  in-progress) from "immunity declared but witness invalid" (a real problem).
- Documentation must clearly explain when to use `--strict` vs default modes,
  and how to integrate antigen with mid-stream development workflows.
- Stories from real adopters about how they integrate antigen into PR/branch
  workflows feed into the documentation.

---

## Engineering-killing risks

### Risk E1: The fingerprint grammar is fundamentally inadequate

**Likelihood**: Medium. **Severity**: High.

ADR-010's v1 grammar uses syn-based AST visitor pattern. If it can't express the
patterns real-world failure-classes need (e.g., subtle data-flow patterns,
inter-function patterns, runtime conditions), antigen-stdlib stalls.

**Signals**:
- Multiple stdlib antigen attempts can't be expressed in the grammar
- The fingerprint field is increasingly free-text rather than structured
- Adopters complain about expressing their domain-specific patterns
- Workarounds proliferate (regex over source, post-scan filters)

**Mitigation**:
- ADR-010 is v1; the team explicitly anticipates v2 with richer grammar
- Sweep A5 (stdlib) is also a stress test for the grammar; failures inform v2
- Tree-sitter integration and pattern-macro shorthand are reserved as v2+ tools
- The team can ratify a new ADR for grammar v2 when the v1 limits become clear

### Risk E2: Cross-crate inheritance breaks

**Likelihood**: Medium. **Severity**: High.

`#[descended_from]` propagation across crate boundaries is novel territory (per
inheritance-from-tambear). Implementation may be harder than expected — semver
interactions, compiler version pinning, IDE integration limits.

**Signals**:
- `#[descended_from(other_crate::Type)]` doesn't work reliably
- Cargo's incremental compilation breaks antigen propagation
- IDE integrations don't reflect inheritance correctly
- Cross-crate antigen versioning produces silent breakage

**Mitigation**:
- ADR-010 explicitly defers cross-crate fingerprint inheritance to a future ADR
- Sweep A4 (composition rules) tests cross-crate scenarios extensively
- The first cross-crate consumer (likely `tambear` adopting `antigen-stdlib`) is
  a testbed; issues surface there before being merged into general API
- Fall-back: if cross-crate inheritance proves intractable, the project may
  accept "within-crate inheritance only for v1; cross-crate in v2"

### Risk E3: Witness validation infrastructure underestimated

**Likelihood**: High. **Severity**: Medium.

Validating a witness — checking that a `#[test]` exists, runs, asserts the right
property — sounds simple but has many edge cases. Witnesses depend on test
framework conventions, conditional compilation, async runtimes, feature flags.

**Signals**:
- High volume of "my witness isn't being recognized" issues
- Witness coverage gaps in audit reports for valid witnesses
- Cargo-antigen audit output includes warnings about "couldn't determine if this
  test exercises the antigen"

**Mitigation**:
- ADR-005 (sub-clause F at trust boundaries) requires structural validation
- Sweep A2 (core macros) and Sweep A3 (cargo antigen scan) explicitly include
  witness-validation work
- The witness-pluralism design (multiple witness types) means the team can ship
  v1 with simple witness types (just `#[test]` and proptest) and add complexity
  later
- Documentation on writing GOOD witnesses helps users avoid edge cases

### Risk E4: Form-blind fingerprints

**Likelihood**: High. **Severity**: High.

*Surfaced empirically by tambear adoption (UlpDistanceRolledByHand, 2026-05-07);
routed by navigator with biological framing; filed by naturalist as H8 in
inventory v0. See `docs/expedition/tambear-adoption-log.md` "Third antigen +
first real cleanup" for the full case.*

A fingerprint declared from one structural manifestation of a failure-class
will systematically miss other structural manifestations of *the same*
failure-class. The cleanup that addresses the inline form of a pattern leaves
the multi-statement form unguarded. **Worse than no fingerprint**: a
form-blind fingerprint creates *false confidence* — the team believes the
class is structurally covered when it is only partially covered.

**Empirical evidence**: tambear's `UlpDistanceRolledByHand` case. The
`tambear-substrate/src/pattern.rs` detector covered the inline expression
form. Two weeks later, the same failure-class appeared in multi-statement
function-body form. The detector was form-blind. ULP-CANON-1 was reported
"complete" on 2026-04-25 but was actually incomplete-as-of-2026-05-07
because new sweeps reintroduced the pattern in the uncovered form.

**Biological framing** *(per naturalist H8)*: the immune system handles the
same problem via **multiple epitopes per antigen**. Linear epitopes
(continuous amino-acid sequence) and conformational epitopes (3D shape)
can recognize the same antigen molecule in different presentation states.
Antibodies are raised against *multiple* epitopes simultaneously, not
against a single representative form. Antigen's design currently supports
one fingerprint per declaration; biology predicts the fingerprint slot
should accept *alternative forms* of the same pattern.

**Signals**:
- A failure-class believed-covered re-emerges in a different syntactic form
  weeks or months after declaration
- `cargo antigen scan` reports pattern matches only at one syntax site;
  manual grep finds additional sites of the same class
- Adopter feedback: "this antigen catches X but not Y, even though Y is
  structurally the same failure"
- Pattern.rs / clippy / similar detectors require multiple per-failure-class
  rules in their codebase (a known prior-art signal that the multi-form
  problem is real)

**Mitigation**:
1. **Top-level `any_of` in fingerprint grammar** (ADR-010): allow a single
   antigen declaration to specify *multiple alternative structural patterns*
   that all match the same antigen. The syntax already supports `any_of`
   nested inside `all_of`; lifting it to top-level is an amendment.
2. **`cargo antigen new` scaffolding**: when generating an antigen from a
   sample failing site, suggest alternative structural forms the
   practitioner should consider (e.g., "this is the inline form; have you
   considered the function-body form?").
3. **antigen-stdlib documentation discipline**: every stdlib antigen
   documents which structural forms it covers AND which it knowingly
   doesn't. The "covered forms" enumeration becomes part of the antigen's
   self-description, surfaced by audit.
4. **Adversarial review at antigen creation**: when an antigen is added,
   the adversarial role explicitly searches for alternative forms of the
   same failure-class. This is a recognition pass per ADR-006.

**Cross-reference**: this risk and H8 motivate a refinement to ADR-010
(grammar) and ADR-006 (recognition). Together with H1 (affinity maturation
/ descendant fingerprint refinement) the project has two
fingerprint-refinement paths: at-declaration multi-form support (H8) and
at-descendant refinement (H1). The first is structurally required; the
second is deferred to v0.2+.

---

## Ecosystem-killing risks

### Risk EC1: Conflict with existing tool authors

**Likelihood**: Low. **Severity**: High.

If clippy, kani, prusti, or similar tool authors perceive antigen as competing
with their work, they may discourage adoption or build incompatible features.

**Signals**:
- Negative reception from tool authors in early outreach
- Tool authors declining to integrate or refusing witness-mechanism API stability
- Blog posts critiquing antigen as duplicating existing tool work

**Mitigation**:
- ADR-002 (compose, don't compete) is foundational; every API decision filters
  through this lens
- Vision pitch explicitly frames antigen as composition layer, not replacement
- Direct engagement with tool authors (per Phase 4 of adoption pathway) is a
  priority — co-design integration patterns rather than imposing
- If a tool author objects to a specific integration, antigen ADAPTS its witness
  adapter to be compatible. Antigen doesn't dictate

### Risk EC2: Standard-track adoption interest

**Likelihood**: Low. **Severity**: Medium (could be Low if managed).

If the Rust core team or governance signals that antigen-like features should be
language-level rather than third-party, the project's value proposition shifts.

**Signals**:
- RFC discussions in rust-lang/rfcs mentioning structural failure-class memory
- Core team members blog/talk about "compiler-level antigen support"
- Tooling RFCs that overlap with antigen's scope

**Mitigation**:
- Antigen's value proposition is the COMPOSITION + ECOSYSTEM library, not just
  the macro primitives. Even if macros become language-features, the ecosystem
  composition stays valuable
- Engagement with the Rust core team is part of Phase 4 outreach
- The project remains adaptable to upstream changes; if a piece becomes language-
  level, antigen drops the corresponding piece and integrates

### Risk EC3: Fragmentation at antigen-stdlib semantics

**Likelihood**: Medium. **Severity**: Medium.

Multiple competing antigen libraries emerge with overlapping or contradicting
declarations of the same failure-class. Users pick libraries and get inconsistent
coverage.

**Signals**:
- Multiple `*-antigens` crates with similar names but different fingerprints
- Issue reports about "this antigen library says X, but my project's antigen says
  not-X"
- Cross-crate antigen consumption produces conflicts

**Mitigation**:
- `antigen-stdlib` is curated by the antigen team with explicit acceptance criteria
- Cross-crate fingerprint validation (per ADR-005) catches overlapping declarations
  with incompatible fingerprints
- Community-coordinated antigen catalogs (under antigen-rs org) reduce fragmentation
- Documentation guides projects to extend stdlib rather than fork

---

## Project-killing risks

### Risk P1: Team-lead bandwidth disappears

**Likelihood**: Medium. **Severity**: High.

The team-lead role is high-bandwidth. If the human team-lead's attention is
diverted (work, life, other projects), the team's coordination layer breaks.

**Signals**:
- Long lag between agent escalations and team-lead responses
- Decisions getting deferred indefinitely
- Sweeps that span weeks rather than sessions
- Naturalist's closure narratives become rare

**Mitigation**:
- The substrate-as-source-of-truth discipline reduces team-lead dependence; agents
  can self-coordinate via campsite logs
- Navigator role escalates to team-lead with stories from the trail, not status
  reports — keeps escalation overhead low
- Sweep planning includes "what could go wrong if we don't have team-lead access"
  as a recovery scenario
- Pre-team scaffolding (this whole substrate) reduces team-lead-needed-for-detail
  scenarios

### Risk P2: Substrate drift across team sessions

**Likelihood**: High. **Severity**: Medium.

Across multiple sessions, agents may drift from the established substrate
(briefing, decisions, glossary, process). Without explicit re-loading, they revert
to implicit defaults.

**Signals**:
- Naturalist observation: "we keep re-deriving things"
- Vocabulary drift in code or new docs (terminology not matching glossary)
- New ADRs that contradict prior ratifications
- Pre-team-scaffolding insights are lost

**Mitigation**:
- Substrate-over-memory discipline (ADR-001 + general team-briefing) is the
  primary defense
- Glossary is canonical and referenced from every doc
- Each fresh session: agent reads the team-briefing first
- Naturalist explicitly tracks substrate drift and flags it
- Process doc requires Phase 1-8 to reference prior ADRs explicitly

### Risk P3: Over-investment in pre-team scaffolding

**Likelihood**: Medium (already happening to some degree). **Severity**: Low to Medium.

Spending too much pre-team-time on substrate may produce diminishing returns. Some
substrate the team would have produced anyway; some might be wrong from the
substrate's perspective and need rewriting.

**Signals**:
- The team's first-sweep finds many amendments needed to pre-team substrate
- Team complaints about "the pre-team docs got it wrong"
- Phase 1-8 deconstruction takes longer than expected because there's so much
  substrate to deconstruct

**Mitigation**:
- This is largely already done; the substrate is what it is
- The team explicitly has authority to amend, supersede, or contradict pre-team
  substrate (per the team-briefing)
- Sweep A1's first deliverable is Phase 1-8 of existing ADRs, which catches errors
- The substrate is starting context, not authority — the team-briefing says this
  explicitly

---

## Risk monitoring discipline

The team should review this register periodically:

- **At sweep start**: which risks are most likely to manifest in this sweep?
- **At sweep mid-point**: are any signals appearing?
- **At sweep close**: which risks materialized? Which mitigations worked? Which
  didn't?

The naturalist may add observations to this register based on sweep activity. The
adversarial role may add new risks discovered through Phase 8 forced rejection of
ratified ADRs.

The register itself is subject to update via PR — risks resolve, transform, or get
added as the project's situation evolves.

---

## What this register IS NOT

- A list of things that WILL happen — these are RISKS, with probability < 1
- A reason to be pessimistic — naming risks is not predicting failure
- A constraint on the team's autonomy — the team navigates risks; they're not
  constrained by them
- Comprehensive — new risks will emerge that we couldn't anticipate; that's normal

The register is **substrate for explicit-mode operation**. Knowing the risks
explicitly lets the team work without implicit anxiety. Naming the failure modes
makes them less powerful, not more.

---

## References

- [`docs/decisions.md` ADR-002](../decisions.md#adr-002--compose-dont-compete) — compose, don't compete
- [`docs/decisions.md` ADR-005](../decisions.md#adr-005--sub-clause-f-at-every-trust-boundary) — sub-clause F
- [`docs/decisions.md` ADR-008](../decisions.md#adr-008--named-observer-position-as-terminal-stratum) — named-observer ergonomics
- [`docs/expedition/revolutionary-and-not.md`](revolutionary-and-not.md) "What could kill it" — earlier risk surface
- [`docs/expedition/inheritance-from-tambear.md`](inheritance-from-tambear.md) — disciplines that mitigate many risks

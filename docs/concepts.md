# Antigen — Concepts

> The architectural concepts behind antigen, in adopter-facing form. For
> the user's first-15-minutes walkthrough, see
> [`tutorial.md`](tutorial.md). For internal substrate, see
> [`docs/expedition/`](expedition/).

---

## What antigen is

Antigen makes **structural memory of failure-classes** part of your codebase.

When you fix a bug, you learn something about *why* a class of code fails. Most of that lesson lives in implicit carriers — your head, a commit message, a Slack thread, a docstring that drifts. None of those carriers are drift-resistant. Six months later, the same shape of bug appears in code written by someone (human or LLM) who never saw the lesson.

Antigen names the lesson, gives it a structural fingerprint, and makes it checkable by cargo tooling. The lesson survives developer turnover, AI agent context cycling, time, and refactors — because it lives in the type system, not in human memory.

### Why now: the generation-inspection asymmetry

Modern software development is characterized by a **structural asymmetry**: generation throughput vastly exceeds inspection throughput for all actor types.

- Humans can't read all the code they ship (especially in AI-pair workflows)
- Vibe coders generate code via prompting that they may not fully understand
- LLM agents can't track across sessions — context resets; summarization drifts
- Human-LLM teams have throughput advantages, but the inspection bottleneck stays bounded by both actor types
- Docs / comments / ADRs / Slack ship faster than they're read

This asymmetry is at a **historic maximum in 2026** and growing. The historical assumption that "the team has read everything" hasn't held in years; it now fails catastrophically.

Antigen's whole reason for existing: the asymmetry guarantees passive memory (docs, comments, "last reviewed" stamps) will fail; the only viable alternative is **structural memory that surfaces itself** at compile time and audit time. See [`vision-pitch.md`](vision-pitch.md) for the deeper articulation.

---

## The third pillar

The Rust safety story currently offers three structural properties:

- **Memory safety** — no use-after-free, no buffer overflow
- **Type safety** — the type system catches mismatches
- **Thread safety** — no data races

Antigen adds a fourth:

- **Domain-knowledge-memory safety** — the *lessons* learned about WHY
  classes of code fail persist structurally; they propagate through
  inheritance; they survive across change.

This is structurally as significant a gap as the gap *testing-as-practice*
filled. Before testing became standard practice, code worked or didn't,
and lessons were tribal. Before *documentation-as-practice* became
standard, knowledge lived in heads. Both filled real gaps but both have
the same weakness: they require ongoing maintenance to stay current.
Testing and documentation are **maintenance-tier** practices.

Antigen operates at **structural-tier**. The vocabulary lives in code
alongside the failure-classes it names. Drift isn't possible silently
— when fingerprints fail to match, the scan notices. The lesson and
the lesson's enforcement are the same artifact.

This is the third pillar: testing checks *this code does X*; documentation
records *we decided X*; antigen captures *this class of code has
historically failed in this structural way, and here is what defends
against it*.

---

## The vocabulary

Antigen's vocabulary is five attribute macros + one cross-cutting parameter. Together they form a *shared coordination layer* — the protocol the various antigen components use to coordinate.

| Macro / parameter | Purpose |
|---|---|
| `#[antigen(name = ..., fingerprint = ..., ...)]` | Declare a named failure-class with a structural fingerprint |
| `#[presents(AntigenName)]` | Mark code as vulnerable to a declared failure-class |
| `#[immune(AntigenName, witness = ...)]` | Claim immunity backed by a named code witness (test, proptest, formal proof, lint, phantom-type) |
| `#[immune(AntigenName, requires = <predicate>)]` | Substrate-witness form (v0.1+, ADR-019): claim immunity backed by a *sidecar predicate* (signers / freshness / Oracle state / etc.) |
| `#[descended_from(Parent)]` | Declare inheritance between failure-classes |
| `#[antigen_tolerance(AntigenName, rationale = ...)]` | Explicitly tolerate a fingerprint match |
| `attested = (who, allowed_types, why, scope)` | Cross-cutting attestation parameter (ADR-020) — adds attestation context to any of the above |

Plus five cargo subcommands:

- `cargo antigen scan` — find every site exhibiting a declared failure-class
- `cargo antigen audit` — verify immunity claims with tier-honest reporting
- `cargo antigen attest` — manage `.attest/<Antigen>.json` substrate-witness sidecars (v0.1+, ADR-019)
- `cargo antigen tolerate` — manage tolerance-ratification sidecars (v0.1+, ADR-019 §tolerance tier)
- `cargo antigen oracle` — manage Oracle artifact-class records (v0.1+, ADR-021)

The five primitives describe a structure that doesn't depend on Rust.
Each could be implemented for other languages (Python, JavaScript,
TypeScript) using that language's metaprogramming or AST tooling. The
vocabulary is the architectural primitive; the Rust implementation is
one realization.

See [`macros.md`](macros.md) for the full reference, and
[`fingerprint-grammar.md`](fingerprint-grammar.md) for the fingerprint
DSL.

---

## Multi-component architecture

Antigen is not a single tool — it's a vocabulary that lets you
compose multiple kinds of structural immunity. Adopting at the floor
gets you one component (the linter); growing into deeper composition
unlocks more.

Seven components currently named (the enumeration is open and may
extend):

### 1. Dev-in-the-loop immunity

You write `#[antigen]` declarations by hand based on your judgment of
what failure classes exist in your domain. The structure forces you to
name the failure, name the witness, justify the rationale.

**Floor**: a single declaration gives the lesson structural form.
**Ceiling**: rich taxonomy with full lineage and witnesses.

### 2. Passive scan/lint/tool immunity

`cargo antigen scan` walks your codebase, finds antigen declarations,
matches fingerprints against unmarked code, reports unaddressed
presentations. `cargo antigen audit` verifies witness validity at the
appropriate tier.

**Floor**: install `cargo-antigen`, run scan, see output. No
declarations required.
**Ceiling**: scan in CI, audit-gated PRs, structural enforcement at
build time.

### 3. Test-integration immunity

Your existing tests become structural memory. The `witness = ...`
field on `#[immune]` links to actual `#[test]` or `proptest!` functions
in your workspace. Audit reports verification at the appropriate
[witness tier](witness-tiers.md).

**Floor**: point at one passing test.
**Ceiling**: full witness pluralism across test, proptest,
phantom-type, formal-verification adapters.

### 4. Knowledge-ecosystem immunity

References attached to antigen declarations point to PR threads, ADRs,
CVEs, post-mortems, RFCs. The structural memory in code becomes a node
in your team's knowledge graph.

```rust
#[antigen(
    name = "stale-cache-after-config-reload",
    references = [
        "pr:owner/repo#1234",
        "issue:owner/repo#567",
        "adr:internal-ADR-042",
        "https://blog.example.com/postmortem-2024-cache-issue",
    ],
    // ...
)]
pub struct StaleCacheAfterConfigReload;
```

**Floor**: one URL.
**Ceiling**: comprehensive bidirectional knowledge graph linking code
to lived context.

### 5. Cross-version / lineage immunity

`#[descended_from]` chains track inheritance, evolution, and
specialization across failure-classes. Cross-version
recognition (`canonical_path` at `name@version` granularity per
ADR-017) handles dependency upgrades correctly.

**Floor**: a single `descended_from(Parent)` declaration.
**Ceiling**: rich inheritance trees with version-boundary handling.

### 6. Cross-crate / ecosystem immunity

Antigen declarations propagate across crate boundaries. Future
`antigen-stdlib` will provide ecosystem-wide failure-class memory; you
inherit common Rust failure-class coverage without authoring antigens
yourself.

**Floor**: `cargo antigen scan --include-deps`.
**Ceiling**: ecosystem-level shared failure-class memory.

### 7. Real-time / CI feedback immunity

PR-scope diff against scan baseline; inline annotations; recognition
at the moment of change rather than at build time. (Planned for a
future sweep; not yet shipped.)

**Floor**: future tooling.
**Ceiling**: rust-analyzer plugin surfacing matches as you type.

---

### Composition

These are not levels of one practice. They are **distinct components**
that compose. A team can deploy components 1+2 only (dev + linter) and
get real value. Another team uses 1+2+3 (adds test integration). A
mature ecosystem participant uses 1-6.

The composition is genuinely orthogonal in most cases. You adopt what
fits your team's existing practice; the components compose without
requiring each other.

See [`docs/expedition/multi-component-immunity.md`](expedition/multi-component-immunity.md)
for the deeper architectural framing (substrate; expected to canonicalize
post-A3.5 ratification).

---

## Substrate-witness pipeline (v0.1+)

Some disciplines can't be witnessed by a single in-tree function. Examples:

- "I reviewed this code against Higham §6.3" — the witness is a human review, not a function
- "The discipline holds because signers A, B, and C attested" — multi-signer attestation
- "This is valid for 180 days after last review" — temporal freshness
- "This claim depends on Oracle X being in `Complete` state" — depends on a separate artifact's lifecycle

The **substrate-witness pipeline** (ADR-019) makes these checkable at audit time. The `#[immune(X, requires = <predicate>)]` form names a predicate over a `.attest/<Antigen>.json` sidecar file co-located with the declaration. The predicate is composed from five leaf operators:

- `signers(required = [...])` — the sidecar must contain signatures from named identities
- `fresh_within_days(N)` — the most recent signature must be within N days
- `ratified_doc(path = ...)` — pointer to a ratified ADR or external doc
- `oracles_complete(files = [...])` — depends on named Oracle records being in `Complete` state
- `signed_trailer(...)` — git-trust-style commit-signed integration

Plus three combinators: `all_of(...)`, `any_of(...)`, `not(...)`.

The audit reports the predicate's evaluation result with **three-tier SignatureStrength** (WORKSPACE-LOCAL / OIDC-IDENTITY / KEY-SIGNED) for each signature, so the *strength of evidence* is visible — not just yes/no.

See [`witness-tiers.md`](witness-tiers.md) for the full tier model. Worked example: [`substrate_witness.rs`](../antigen/examples/substrate_witness.rs).

---

## Oracle artifacts (v0.1+, ADR-021)

When your discipline depends on an *external reference* — a paper, an ADR, a spec — Oracle artifacts make that reference first-class:

- **5-state lifecycle**: Draft → Complete → Deprecated / Retired / Revoked (+ Reopened)
- **Stewardship**: each Oracle has signers who attested and stewards who maintain the reference
- **Audit integration**: `oracles_complete(files = [...])` checks Oracle state at audit time
- **Provenance trail**: who declared, who transitioned states, when, why

This closes the "URLs go stale" problem at the substrate level. The reference is stewarded, versioned, lifecycle-tracked — and immunity claims that depend on it stay honest as the reference evolves.

Worked example: [`oracle_lifecycle.rs`](../antigen/examples/oracle_lifecycle.rs).

---

## Antigen category — substrate-alignment vs functional-correctness

A v0.2 distinction (ratifying via NEW-ADR-028): antigens come in two structural categories.

- **Substrate-alignment antigens** — when the *representation* of state diverges from actual state. Substrate-witness; scan/commit-time; observer-role catches. Examples: `UnanchoredGitignorePattern` (git's view of disk ≠ disk), `DocClaimVsCodeImplementationMismatch` (docs drift), `RollbackWithoutTriageCommit` (history drift).
- **Functional-correctness antigens** — when a *verb produces wrong output*. Code-witness; test/runtime; adversarial + scientist roles catch. Examples: `PanickingInDrop` (Drop produces process abort), `SignedZeroDiscipline` (sinh produces wrong sign at -0.0), `SilentCliCommandFailure` (CLI exit code lies).

The category metadata shapes witness type, audit layer, lifecycle phase, and responder role. Many of v0.1's antigens are functional-correctness; v0.2 substantially expands substrate-alignment via supply-chain, VCS-info-loss, mucosal-boundary, and the antigen-category metadata itself.

---

## The biology cognate

The biological metaphor is **load-bearing, not decorative**. The
project's design has consistently emerged from immunological structure:

| Biology | Antigen analog |
|---|---|
| Pathogen Recognition Receptors (PRRs) | Fingerprint engine (passive structural matching) |
| MHC Class I/II presentation | `#[presents(antigen)]` |
| T-cell receptors | Named failure-class fingerprints |
| Antibody | Test, proptest, phantom-type witness, lint reference |
| B-cell memory | `#[antigen]` declarations persisting past specific bugs |
| Antibody titer | `verified_at` temporal field (ADR-016) |
| B-cell lineage | `#[descended_from]` propagation |
| Peripheral tolerance / Tregs | `#[antigen_tolerance]` for legitimate matches |
| Antigenic drift / shift | Version-boundary recognition (ADR-017) |
| Epidemiological surveillance | Cross-crate / ecosystem propagation (Component 6) |
| Dendritic-cell processing | Audit pipeline (witness resolution + tier reporting) |

When the biology predicts a primitive, the project builds it. When
the biology breaks at a boundary, that silence is itself information:
biology has reached its honest extent, and engineering extends past it.

The biology metaphor is also **post-COVID accessible** — antigen,
antibody, vaccination, immunity are everyday language. Adopters carry
intuition from lived experience, not specialized training.

---

## Co-native by design

Antigen is built so both human developers and LLM agents can read
the same vocabulary natively, without translation.

- Declarations in code are readable by humans and parseable by LLMs
- The biology metaphor is universal lived experience for humans and
  unambiguous semantic cognate for LLMs
- Audit reports come in both human-readable and JSON forms
- The vocabulary's structure is the same for both audiences

This co-native property matters because:

- Failure-class memory survives AI agent context cycling
- New team members (human or LLM) inherit the failure-class taxonomy
  by reading what's already in the code
- The lessons travel structurally — they propagate to *any* model or
  human reading the codebase, not just to fine-tuned weights

For agents specifically collaborating on antigen-using code, see
[`for-llm-collaborators.md`](for-llm-collaborators.md).

---

## Three adopter pathways

Antigen meets you where you are. Three pathways with three different
relationships between tool and existing practice:

### Junior adopters

Someone learning Rust+antigen together as one practice (like learning
Rust+cargo+tests together) develops both in parallel. The tool teaches
the discipline by demanding it: declaring an antigen forces naming
the failure-class, naming the witness, justifying the rationale.

**The tool produces the discipline through use.** Biology cognate:
*developmental immunology* — building the recognition machinery itself.

### Senior adopters with partial discipline

Developers with existing failure-class awareness (tribal knowledge,
post-mortem discipline, code-review judgment) but no structural-
memory layer get the missing tier from antigen. They extend their
existing practice rather than rebuild.

**The tool amplifies existing discipline.** Biology cognate:
*vaccination* — existing recognition machinery meets new structural
targets via the tool.

### Mature organizations with explicit discipline

Teams with ADR culture, post-mortem rigor, refactoring discipline
already have antigen-like practices in narrative form. The tool gives
them structural enforcement of claims they already make.

**The tool formalizes existing discipline.** Biology cognate:
*immune surveillance* — making existing recognition externally
verifiable.

All three pathways are real; the "ideal user" property of the
project's own development (we had the discipline before the tool) is
replicable through onboarding for the first; extended through the tool
for the second; formalized through the tool for the third.

---

## Recognition-not-design (amended for two disciplines)

The project operates under a discipline named in ADR-006, **amended** in the v0.2 ratification ceremony to formalize a two-disciplines architecture (NEW-ADR-022).

### For ADOPTER extensions: recognition discipline

When *you* are adding antigens for your team's codebase:

> When uncertain whether to design something or recognize something, lean toward recognition. New antigens, new witness types, new composition rules are added when they recognize existing structure in the substrate — not when they extend the design speculatively.

- **Don't design speculative antigens.** Wait until you've encountered the failure-class in real code (yours or a dependency's). The discipline catches premature abstraction.
- **Adopter additions land when three independent substrate-grounded instances surface.**
- **Recognition leaves substrate**: when you declare an antigen, point to references that ground it. When you tolerate a fingerprint match, rationale is required at parse time. The discipline is structural.

### For STDLIB growth: research-driven discipline

When *antigen-the-project* expands its core vocabulary:

> Stdlib growth is research-driven, deliberately comprehensive. New primitives are substrate-citable from postmortems / literature / training-data / predictive analysis / biological-component-mapping — not constrained to "wait for the third instance."

The biological immune system serves as the systematic discovery framework. Each unused immune-system component is a research-arc prompt. v0.2's macro family expansions (~50+ primitives across 9 tiers per the [biological discovery framework](expedition/biological-component-discovery-framework.md)) are research-driven, not recognition-gated.

This split matters because the two disciplines have different cost asymmetries. Speculative *adopter* extensions bloat noise; speculative *stdlib* extensions cover failure-classes adopters haven't yet hit but should be protected against. The amended ADR-006 + new ADR-022 formalize this split.

---

## What antigen is NOT

Critical for understanding what's new:

- **Not "another testing tool"** — tests verify *this code does X*;
  antigen captures *this class of code has historically failed in this
  structural way*. Different artifact, different lifecycle, different
  contribution.
- **Not "another linter"** — clippy catches style and common mistakes;
  antigen catches *named failure-class patterns* with structural
  fingerprints + delegated witness validation. Antigen composes WITH
  clippy.
- **Not a documentation system** — documentation drifts; antigen
  declarations are checked by cargo tooling; drift fails the build.
- **Not a replacement for tests, lints, or formal verification** —
  antigen composes them (witness pluralism). It delegates verification
  to whichever tool proves immunity for a given antigen.
- **Not a logic-bug catcher** — antigen catches *named* failure-classes;
  novel logic errors are still tests' job.
- **Not a fine-tuning or training-data alternative** — lessons live in
  code, inspectable and version-controlled. They propagate to any AI
  model or human reading the codebase, not just to fine-tuned weights.

Antigen is the third pillar alongside testing and documentation. The
existing pillars stay; antigen adds structural memory they couldn't
provide.

---

## Open enumeration

The enumeration of seven components is **provisional and open**. The
project's discipline (recognition-not-design) holds that new components
land when substrate-grounded instances accumulate. Future components
may surface from:

- Tooling tiers we haven't yet built (rust-analyzer plugin, language
  servers, IDE integration)
- Cross-language extensions (Python, JavaScript, framework-specific)
- Cross-tier extensions (organization-tier governance failure-classes;
  team-coordination failure-classes)

The vocabulary is the spine. Components attach to the spine. The
spine is stable; the fabric grows.

---

## Where to go next

- [`tutorial.md`](tutorial.md) — your first 15 minutes
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) —
  placement conventions
- [`usage-patterns.md`](usage-patterns.md) — common patterns
- [`macros.md`](macros.md) — full macro reference
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`witness-tiers.md`](witness-tiers.md) — tier semantics
- [`output-formats.md`](output-formats.md) — scan/audit output reference
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide
- [`roadmap.md`](roadmap.md) — trajectory
- [`for-llm-collaborators.md`](for-llm-collaborators.md) — co-native
  protocol for AI agents

For internal substrate:
- [`docs/origin.md`](origin.md) — the founding incident
- [`docs/decisions.md`](decisions.md) — ratified ADRs
- [`docs/postures.md`](postures.md) — architectural postures
- [`docs/scope.md`](scope.md) — comprehensive vision
- [`docs/expedition/`](expedition/) — design substrate, including
  multi-component immunity deep-dive and antigen-applied-to-antigen
  recursion

# Antigen — Concepts

> The architectural concepts behind antigen, in adopter-facing form. For
> the user's first-15-minutes walkthrough, see
> [`tutorial.md`](tutorial.md).

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

Antigen's vocabulary is a set of attribute macros that form a *shared coordination layer* — the protocol the various antigen components use to coordinate.

| Macro / parameter | Purpose |
|---|---|
| `#[antigen(name = ..., fingerprint = ..., ...)]` | Declare a named failure-class with a structural fingerprint |
| `#[presents(AntigenName)]` | Mark code as a site that exhibits a declared failure-class |
| `#[defended_by(AntigenName)]` | Register a test / proptest function as a **code-tier witness** for a failure-class (ADR-029) |
| `#[presents(AntigenName, requires = <predicate>)]` | Attach a **substrate-tier witness** predicate to a presents-site — evidence that lives outside the code (sidecar signers, freshness, ratified docs, etc.) |
| `#[presents(AntigenName, proof = <expr>)]` | Attach a **phantom-type / formal-proof** witness (a type-system construction whose existence IS the evidence) |
| `#[descended_from(Parent)]` | Declare inheritance between failure-classes |
| `#[antigen_tolerance(AntigenName, rationale = ...)]` | Explicitly tolerate a fingerprint match |
| `attested = (who, allowed_types, why, scope)` | Cross-cutting attestation parameter (ADR-020) — adds attestation context to any of the above |

> **ADR-029 observe-don't-declare**: immunity is **observed** by audit, not claimed at the site.
> `#[defended_by]` registers intent; `#[presents(requires=)]` declares substrate evidence.
> The audit cross-references them and reports a per-site verdict:
> `defended` (evidence found), `undefended` (no evidence), or `substrate-gap` (predicate declared but failing).
> The deprecated `#[immune]` API (v0.1) directly claimed immunity from the site — the new idiom
> separates the vulnerability marker (`#[presents]`) from the evidence registration (`#[defended_by]`,
> `requires=`, `proof=`). Audit observes; code never claims.

Plus five cargo subcommands:

- `cargo antigen scan` — find every site exhibiting a declared failure-class
- `cargo antigen audit` — observe per-site defense verdicts (defended / undefended / substrate-gap)
- `cargo antigen attest` — manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019)
- `cargo antigen tolerate` — manage tolerance-ratification sidecars (ADR-019 §tolerance tier)
- `cargo antigen oracle` — manage Oracle artifact-class records (ADR-021)

These primitives describe a structure that doesn't depend on Rust.
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

Your existing tests become structural memory. `#[defended_by(AntigenName)]`
on a `#[test]` or `proptest!` function registers it as a code-tier witness.
Audit observes the registration and reports the defense at the appropriate
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

Recognition at the moment of change rather than at build time. Two
surfaces ship the floor of this component:

- **Editor-inline (flycheck).** The catalog-match spine renders findings
  in rust-analyzer's `check.overrideCommand` schema
  (`cargo antigen scan --message-format json`), so matches surface as
  squiggles at keystroke speed with no custom LSP server (ADR-043
  Client B). See [`output-formats.md`](output-formats.md).
- **PR-scope structural diff (DETECT).** The *diff-native* modality
  (ADR-046) matches a structural DELTA between two commits, not a
  snapshot: an item whose structural digest changed (or was
  added/removed) between `HEAD~1` and `HEAD` is surfaced as "this
  item's structure changed." This is the floor that keeps a green
  snapshot scan from *laundering* a guard removal on a PR. The DETECT
  slice is shipped; the *classify-the-removed-guard* slice (a
  before/after predicate pair) is the next increment.

**Floor (shipped)**: `--message-format json` flycheck + diff-native
DETECT.
**Ceiling**: rust-analyzer plugin with exact spans; the CLASSIFY
guard-regression matcher; the agent-query endpoint at generation time
(ADR-043 Client C).

---

### Composition

These are not levels of one practice. They are **distinct components**
that compose. A team can deploy components 1+2 only (dev + linter) and
get real value. Another team uses 1+2+3 (adds test integration). A
mature ecosystem participant uses 1-6.

The composition is genuinely orthogonal in most cases. You adopt what
fits your team's existing practice; the components compose without
requiring each other.

See [`immune-system-primitive-map.md`](internal/immune-system-primitive-map.md)
for the deeper multi-component architectural framing.

---

## The catalog-match spine

On its own, a scanner that built its fingerprint table only from the
`#[antigen]` declarations **in the tree it was scanning** could flag a
failure-class only in a crate that had **declared that class itself**. A
fresh crate that declared nothing would get zero findings, even when its
code structurally matched a dozen well-known failure-classes — the first
ninety seconds of a newcomer's experience would be a false all-clear.

The **catalog-match spine** closes this: a single
callable scan service that matches a crate against antigen's **bundled
stdlib catalog** — antigen's own vetted fingerprints — without the
adopter declaring anything. `cargo antigen scan` on a fresh crate
produces real findings from antigen's class memory.

The spine is built once; **four renders** ride on top of it, differing
only in serializer and transport:

| Render | Surface | What it serves |
|---|---|---|
| **CLI** | `cargo antigen scan` (`--bundled-catalog` / auto-detect) | the dev at the terminal |
| **Editor-inline** | `--message-format json` (rust-analyzer flycheck) | the dev at every keystroke |
| **Agent-query** | MCP endpoint (per-fragment) | the careful agent, *before* it emits code |
| **Session-prime** | batch digest of top failure-classes | the agent that doesn't know to ask |

The two CLI/render surfaces ship today; the agent-query and
session-prime renders are sequenced follow-ons (the spine is the same;
they are new transports).

**The honest claim-scope.** A bundled-catalog match is a *structural
fingerprint match* — a syntactic FACT: "this site's structure matches a
known failure-class, at a calibrated tier." It is **not** an audited
verdict. A match does not assert a defense was checked, nor that the
site is all-clear; it says only *a fingerprint matched here, go look*.
The canonical phrasing the editor render carries per-diagnostic is:
**"a fingerprint match to inspect, not an audited verdict."** This is
not a UX nicety — it is **claim-scope honesty** made structural (see
below).

---

## The Learning-Core loop (the keystone)

Everything above APPLIES failure-class memory. The Learning-Core is the
one organ that *generates* it — antigen's affinity-maturation engine,
the biological cognate of how a germinal center matures antibodies. It
is a closed loop:

> **cluster → propose → test (promote / prune) → with a self-tolerance
> governor holding the whole loop honest.**

1. **Cluster.** Group structurally-similar *marked-unknown* sites — the
   `#[dread]` / `#[aura]` marks where a developer felt "something is
   wrong here" but couldn't yet name the class. Sites cluster by a
   name-insensitive **shape digest** so two sites of the same shape with
   different names land together.
2. **Propose (C).** *Anti-unify* a cluster into a draft fingerprint. The
   shared structure (item-kind, trait, body-calls every member makes)
   becomes the skeleton; the signals only *some* members carry become an
   `any_of([...])` disjunction. The draft is a **HYPOTHESIS** — a
   ratifiable suggestion carrying provenance, **never** an
   auto-asserted `#[presents]` or an auto-named class.
3. **Test / self-tolerance (B).** A draft is *promotable* only if it
   **spares a clean corpus** — known-good sibling code the draft must
   NOT flag. This is **germinal-center negative selection**: screening a
   newly-generated draft against self, the same checkpoint that culls a
   hypermutated B-cell which gained self-reactivity against the self it is
   shown. As in the body, the screen is only as good as that corpus — it
   spares the clean code it samples. A draft that matches clean code would,
   once promoted, flood that clean code with false positives — antigen's own
   **autoimmunity**. B rejects it.
4. **Promote / prune.** A draft that spares the clean corpus is promoted
   to a candidate fingerprint (still a suggestion at a calibrated tier,
   awaiting a human or incident to *ratify* it into a named class). A
   draft that binds clean code is pruned.

### The one safety line: C ══ B

The single most load-bearing fact about the Learning-Core: **the
generator (C) can never promote a draft without the selector (B)
passing.** A candidate generator shipped *without* its self-tolerance
gate doesn't merely under-perform — it actively ships autoimmunity
(false positives flooding the codebase). So this co-ship is a **safety
constraint**, not a sequencing preference. In the code this line is
**type-enforced**, not convention: the only function that returns a
promotable fingerprint routes every draft through the spare-clean gate,
and the gate **refuses an empty clean corpus** ("cannot certify safety
against nothing" — a vacuous pass would be autoimmunity with a green
check). The full first-principles account of *why* the loop closes
safely is in [`the-keystone-explained.md`](the-keystone-explained.md).

### What ships — and what it does NOT

This matters for setting expectations honestly:

- The Learning-Core is a **library** (`antigen::learn`) with a user-facing verb on
  top of it: **`cargo antigen propose`**. The verb re-acquires a marked cluster,
  collects an operator-supplied clean corpus, runs the learner, and renders the
  outcome as a ratifiable suggestion — it observes, it does not name a class. See
  [`cli-reference.md`](cli-reference.md#propose).
- Its production caller is that verb. The two safety-relevant hardenings that
  underpin it are built: the near-miss non-vacuity gate (ADR-047 — the gate
  certifies a draft only against a corpus that exercises it) and the
  `PromotedDraft` capability-token (ADR-048 — a promotable token only the gate can
  mint). On antigen's own marks the verb **routes the draft to a human ratifier**
  (the corpus holds no near-miss); it does not name a class for itself — that
  self-immunizing promote payoff needs abstract-recall clustering. v0.6 built the
  organs a learned class matures and is curated through (see *Drift-detection: the
  maturing organism* below, library-complete); the live CLI curation loop is the v0.7
  frontier.
- The falsification gate is real: antigen carries three genuinely-felt
  `#[dread]` marks on its **own** production source, and the dogfood
  proof anti-unifies the two *silent-skip twins* among them (a directory
  walk that swallows an IO/parse error and reports clean over an
  incomplete corpus) into a draft, governed by B. The *mechanism* is
  proven; the specific dogfood draft over-fits its near-identical twins
  (a recall increment, located honestly).

---

## Drift-detection: the maturing organism

The Learning-Core above is the **afferent** arc: it takes a felt cluster and proposes a
draft. v0.6 adds the organs that let a *learned class* live — mature toward a better
fingerprint, sense when it has gone obsolete or is being evaded, and be curated
(eventually forgotten) when it stops earning its keep. This is antigen modeled as a
maturing organism, not a static catalog.

> **Library-complete, live loop is v0.7.** Every organ below ships as a tested, composable
> `antigen::learn::*` library API. What does **not** ship yet is a `cargo antigen` verb that
> drives the whole sense → classify → act loop end-to-end — no production CLI caller wires
> these together today. The live curation loop is the v0.7 frontier. The organs are real;
> the verb that runs them is not yet built.

The organs, in the order a class flows through them:

- **The life-record (STOCK).** A class's append-only autobiography (`antigen::learn::life_record`).
  Before v0.6, proposing was memoryless; the life-record is the persistent trajectory of a
  class's affinity over time — the substrate every sensor below reads. Its recomputable
  input (the SZZ `(defect, fix)` corpus) stays derivable from git via `cargo antigen mine`.
- **Maturation (MATURE).** The `Affinity { recall, precision }` 2-vector is the *height* a
  draft climbs; the maturation engine (`antigen::learn::maturation`) takes a rough
  anti-unified draft and matures it toward the Pareto frontier of (recall, precision) — the
  germinal-center analog.
- **The reader (SENSE).** `antigen::learn::reader` watches a class's relationship to the live
  code and reports whether it has gone dormant, obsolete, or is being evaded.
- **ADWIN (SENSE, the loud half).** `antigen::learn::adwin` is a batch drift-detector over a
  class's affinity-trajectory. Its defining honesty is a third verdict beyond drift/no-drift:
  **`UnderPowered`** — "I cannot yet see drift for this class, and here is exactly when I
  will be able to." At antigen's current scale (a class has matured only a handful of times)
  `UnderPowered` is the *default* — and that is the correct, valuable behavior: a detector
  that fires zero and says-so, rather than one that guesses on too little data. The same
  organ fires correctly once trajectories lengthen, with no code change.
- **The discriminator (CLASSIFY).** `antigen::learn::discriminator::fused_classify` fuses the
  streamless sensors into one `ClassVerdict` per failure-class — the build-once share lives at
  the classifier, not duplicated in each sensor.
- **CURATE (ACT) — the moral center.** `antigen::learn::curate` is the efferent decision-layer:
  the forget-gate. Every other organ senses or classifies; CURATE alone *acts*. Its
  conservative default **holds** — it never forgets — whenever any channel is blind (ADWIN
  `UnderPowered` or the static sensor `Indeterminate`), per ADR-057. The forgetting is the
  trust: a learned class is only ever decayed through a reversible, human-ratifiable ladder,
  never silently dropped.

For the drift-detector's verdict type and the honest-scope of its statistical bound, see
ADR-065 in [`decisions.md`](decisions.md) and the `library-api.md` surface for
[`DriftVerdict`](library-api.md#drift-detection-driftverdict-adr-065).

---

## Claim-scope honesty (the cross-cutting discipline)

A single discipline runs under the catalog-match spine, the
Learning-Core, and the diff-native modality alike — and it is worth
naming as its own concept because it is *why* these surfaces can be
trusted.

> **Claim-scope honesty**: every component that emits a verdict or
> generates a candidate reports **what it actually proved** — scoped to
> its real reach, with its own boundary — never the potential-maximum
> its category could theoretically provide.

The grounding is not stylistic; it is **computability-forced**. Every
failure-class capability splits into two halves:

- A **syntactic half** — decidable, machine-tractable, reproducible:
  match a fingerprint, anti-unify a cluster, set-diff two commits' item
  digests. antigen DOES this half and reports a FACT at a calibrated
  tier.
- A **semantic half** — *is this matched site a real defect? does this
  draft name a real failure-class? was the removed guard required?*
  These are non-trivial semantic properties of programs, and by **Rice's
  theorem** they are undecidable. antigen does NOT (cannot) do this
  half; a human, a CI context, or an incident **ratifies**.

The boundary between the halves is exactly the syntactic/semantic line,
which is exactly the decidable/undecidable line. That coincidence is the
discipline's whole mechanism: antigen asserts up to the line and
*labels honestly* past it. (Formally: soundness-without-completeness in
the Cousot & Cousot 1977 sense; the trade is forced by Rice 1953.)

This is made **structural** in the type system, not left to
discipline:

- A scan match is a `FingerprintMatch`, a distinct sum-type variant that
  **cannot** masquerade as an audited `DialVerdict`. The two are
  different shapes; the machine can state what it matched but is
  structurally unable to state that it ratified.
- A learning-core draft is a labeled hypothesis; the only promotable
  path runs through the self-tolerance gate (the C ══ B line above).
- The diff-native modality reports "a guard-shaped call was removed
  here," never "regression" — requiredness is the semantic half, left
  to the reviewer.

> **Naming note.** "claim-scope honesty" names this *epistemic*
> discipline, distinct from the **coverage frontier** — the *spatial*
> concept of what the scan did not reach. The two rhyme but are
> orthogonal: the coverage frontier is *what we did not inspect*;
> claim-scope honesty is *don't over-claim what we DID inspect*. See the
> [glossary](glossary.md).

---

## Substrate-witness pipeline

Some disciplines can't be witnessed by a single in-tree function. Examples:

- "I reviewed this code against Higham §6.3" — the witness is a human review, not a function
- "The discipline holds because signers A, B, and C attested" — multi-signer attestation
- "This is valid for 180 days after last review" — temporal freshness
- "This claim depends on Oracle X being in `Complete` state" — depends on a separate artifact's lifecycle

The **substrate-witness pipeline** (ADR-019) makes these checkable at audit time. The `#[presents(X, requires = <predicate>)]` form attaches a predicate to a presents-site, evaluated against a `.attest/<Antigen>.json` sidecar co-located with the declaration. The predicate is composed from five leaf operators:

- `signers(required = [...])` — the sidecar must contain signatures from named identities
- `fresh_within_days(N)` — the most recent signature must be within N days
- `ratified_doc(path = ...)` — pointer to a ratified ADR or external doc
- `oracles_complete(files = [...])` — depends on named Oracle records being in `Complete` state
- `signed_trailer(...)` — git-trust-style commit-signed integration

Plus three combinators: `all_of(...)`, `any_of(...)`, `not(...)`.

The audit reports the predicate's evaluation result with **three-tier SignatureStrength** (WORKSPACE-LOCAL / OIDC-IDENTITY / KEY-SIGNED) for each signature, so the *strength of evidence* is visible — not just yes/no.

See [`witness-tiers.md`](witness-tiers.md) for the full tier model. Worked example: [`substrate_witness.rs`](../antigen/examples/substrate_witness.rs).

---

## Oracle artifacts (ADR-021)

When your discipline depends on an *external reference* — a paper, an ADR, a spec — Oracle artifacts make that reference first-class:

- **5-state lifecycle**: Draft → Complete → Deprecated / Retired / Revoked (+ Reopened)
- **Stewardship**: each Oracle has signers who attested and stewards who maintain the reference
- **Audit integration**: `oracles_complete(files = [...])` checks Oracle state at audit time
- **Provenance trail**: who declared, who transitioned states, when, why

This closes the "URLs go stale" problem at the substrate level. The reference is stewarded, versioned, lifecycle-tracked — and defenses that depend on it stay honest as the reference evolves.

Worked example: [`oracle_lifecycle.rs`](../antigen/examples/oracle_lifecycle.rs).

---

## Antigen category — substrate-alignment vs functional-correctness

A structural distinction (ADR-028): antigens come in two categories.

- **Substrate-alignment antigens** — when the *representation* of state diverges from actual state. Substrate-witness; scan/commit-time; observer-role catches. Examples: `UnanchoredGitignorePattern` (git's view of disk ≠ disk), `DocClaimVsCodeImplementationMismatch` (docs drift), `RollbackWithoutTriageCommit` (history drift).
- **Functional-correctness antigens** — when a *verb produces wrong output*. Code-witness; test/runtime; adversarial + scientist roles catch. Examples: `PanickingInDrop` (Drop produces process abort), `SignedZeroDiscipline` (sinh produces wrong sign at -0.0), `SilentCliCommandFailure` (CLI exit code lies).

The category metadata shapes witness type, audit layer, lifecycle phase, and responder role. The substrate-alignment category spans supply-chain, VCS-info-loss, mucosal-boundary, and the antigen-category metadata itself.

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

The project operates under a discipline named in ADR-006, **amended** to formalize a two-disciplines architecture (ADR-022).

### For ADOPTER extensions: recognition discipline

When *you* are adding antigens for your team's codebase:

> When uncertain whether to design something or recognize something, lean toward recognition. New antigens, new witness types, new composition rules are added when they recognize existing structure in the substrate — not when they extend the design speculatively.

- **Don't design speculative antigens.** Wait until you've encountered the failure-class in real code (yours or a dependency's). The discipline catches premature abstraction.
- **Adopter additions land when three independent substrate-grounded instances surface.**
- **Recognition leaves substrate**: when you declare an antigen, point to references that ground it. When you tolerate a fingerprint match, rationale is required at parse time. The discipline is structural.

### For STDLIB growth: research-driven discipline

When *antigen-the-project* expands its core vocabulary:

> Stdlib growth is research-driven, deliberately comprehensive. New primitives are substrate-citable from postmortems / literature / training-data / predictive analysis / biological-component-mapping — not constrained to "wait for the third instance."

The biological immune system serves as the systematic discovery framework. Each unused immune-system component is a research-arc prompt. The macro family expansions (~50+ primitives across 9 tiers per the [biology primitive map](internal/immune-system-primitive-map.md)) are research-driven, not recognition-gated.

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

Going deeper:
- [`the-keystone-explained.md`](the-keystone-explained.md) — why the
  Learning-Core loop closes safely, from first principles
- [`the-immune-system-a-programmers-guide.md`](the-immune-system-a-programmers-guide.md)
  — the biology cognate walked end to end, the argument for ADR-003
- [`war-stories/learning-from-its-own-wounds.md`](war-stories/learning-from-its-own-wounds.md)
  — antigen running the Learning-Core on its own honest self-doubt
- [`origin.md`](origin.md) — the founding incident
- [`decisions.md`](decisions.md) — ratified ADRs
- [`postures.md`](internal/postures.md) — architectural postures
- [`scope.md`](scope.md) — the architectural class and adoption strategy

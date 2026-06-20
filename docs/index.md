# Antigen — Documentation

> Welcome to antigen. This is the documentation index. Pick a path
> below depending on what you want to do.

For the project's README (install, quickstart, project framing), see
[`../README.md`](../README.md).

---

## I just want to try antigen

For a five-minute taste before committing: **[`quickstart.md`](quickstart.md)**
walks you through `cargo install cargo-antigen` → first scan → first
`#[antigen]` declaration without leaving the page.

Want your **literal first session** narrated step-by-step — install, first
scan (watch the bundled-catalog auto-detect fire), reading one real
finding field-by-field, and wiring your editor — with every command run for
real? See **[`getting-started.md`](getting-started.md)**.

Already past quickstart? Continue with **[`tutorial.md`](tutorial.md)**
for the full first-15-minutes walkthrough (declare → scan → defend →
audit, end-to-end).

The shortest possible version, inline:

```sh
cargo install cargo-antigen
cd /path/to/your/rust/project
cargo antigen scan
```

On a fresh codebase with no antigens declared yet, scan does **not** report a
false all-clear: antigen **auto-injects its bundled stdlib catalog** and
surfaces real fingerprint-match candidates from the shipped failure-classes (e.g.
an unchecked index, a `panic` in a `Drop` impl). These are *a fingerprint match
to inspect, not an audited verdict* — see [`quickstart.md`](quickstart.md) Step 2
and [`reading-a-verdict.md`](reading-a-verdict.md). Then add the dependency and
declare your own first antigen — quickstart shows you how:

```toml
[dependencies]
antigen = "0.5.0-beta.1"   # check crates.io for the latest version
```

---

## Pick your path

Six ways in, depending on why you're here. Each row names where to start; the
sections below expand each path.

| If you want to… | Start here | Then |
|---|---|---|
| **See it work in 30 seconds** | [the felt arc](the-felt-arc.md) | one real run, no install |
| **Use it on your code** | [quickstart](quickstart.md) | [getting-started](getting-started.md) → [tutorial](tutorial.md) → [concepts](concepts.md) |
| **Understand how it works** | [concepts](concepts.md) | [the maturing organism](the-maturing-organism.md) → [drift-detection and the moral center](drift-detection-and-the-moral-center.md) → [the v0.6 anatomy](the-v06-anatomy.md) → [library-api](library-api.md) → [the immune-system guide](the-immune-system-a-programmers-guide.md) |
| **Know why it's shaped this way** | [decisions](decisions.md) | [the keystone explained](the-keystone-explained.md) |
| **Get unstuck** | [troubleshooting](troubleshooting.md) | [reading a verdict](reading-a-verdict.md) → [i-scanned-and](i-scanned-and.md) |
| **Collaborate as an AI agent** | [for LLM collaborators](for-llm-collaborators.md) | the co-native protocol, first-class |

The detail for each:

### "I'm new to antigen; show me what it is and how to use it"

Read in order:

1. **[`quickstart.md`](quickstart.md)** — 5-minute taste before
   committing
2. **[`concepts.md`](concepts.md)** — what antigen is, the third
   pillar framing, the vocabulary, the seven components, the biology
   cognate, the three adopter pathways
3. **[`tutorial.md`](tutorial.md)** — your first 15 minutes, end-to-end
4. **[`case-study.md`](case-study.md)** — narrative walkthrough of a
   real failure-class (less "follow these steps", more "here's what
   actually happened")
5. **[`examples-guide.md`](examples-guide.md)** — progressive
   walkthrough of `antigen/examples/`
6. **[`composition.md`](composition.md)** — how antigen fits with
   clippy, proptest, kani/prusti/verus/creusot, phantom types, tests,
   ADR culture, CI
7. **[`where-to-look-for-antigens.md`](where-to-look-for-antigens.md)**
   — conventions for locating antigen declarations in your project
8. **[`usage-patterns.md`](usage-patterns.md)** — common patterns for
   real failure-classes
9. **[`anti-patterns.md`](anti-patterns.md)** — common mistakes when
   adopting antigen, with the structural reason each is wrong

### "I want a reference for a specific thing"

- **[`cli-reference.md`](cli-reference.md)** — the whole `cargo antigen`
  command surface in one place: every subcommand, one line each, with links to
  the detail
- **[`macros.md`](macros.md)** — full reference for `#[antigen]`,
  `#[presents]`, `#[defended_by]`, `#[presents(requires=)]`,
  `#[descended_from]`, `#[antigen_tolerance]`, plus the cross-cutting
  `attested = (...)` parameter (and the deprecated `#[immune]` form)
- **[`fingerprint-grammar.md`](fingerprint-grammar.md)** — fingerprint
  DSL (seven item-level operators + composition)
- **[`witness-tiers.md`](witness-tiers.md)** — `WitnessTier` gradient
  semantics (FormalProof / Execution / Reachability / None), three-tier
  `SignatureStrength` for substrate-witness signatures, and audit hints
- **[`output-formats.md`](output-formats.md)** — `cargo antigen scan` /
  `audit` / `attest` / `tolerate` / `oracle` human + JSON output reference,
  including the `--message-format json` editor-flycheck surface
- **[`editor-integration.md`](editor-integration.md)** — wire
  `cargo antigen scan` into your editor (rust-analyzer flycheck) so fingerprint
  matches render inline as warnings; the `--message-format json` reference
- **[`library-api.md`](library-api.md)** — using antigen as a **library** (not
  the CLI): `antigen::scan` (typed `ScanReport`) and `antigen::learn` (the
  Learning-Core), with runnable snippets
- **[`glossary.md`](glossary.md)** — every project term anchored to
  its biological referent and Rust analog

### "Show me the headline surfaces"

Two surfaces a docs-reader should know about up front:

- **Bundled stdlib catalog (auto-detect).** A fresh crate with zero antigen
  declarations does not report a false all-clear — scan auto-injects antigen's
  shipped failure-class fingerprints. See [`quickstart.md`](quickstart.md) Step 2
  and the `--bundled-catalog` section of
  [`output-formats.md`](output-formats.md).
- **Editor flycheck (`--message-format json`).** `cargo antigen scan` speaks the
  rustc/cargo line-protocol, so rust-analyzer renders fingerprint matches inline
  as warnings — no custom LSP server. See
  [`editor-integration.md`](editor-integration.md).

Antigen also ships the **Learning-Core** loop (a worry → cluster → propose →
self-tolerance gate → a human ratifier) — live as `cargo antigen propose`. See
[`the-learning-loop.md`](the-learning-loop.md) for where it sits in the system,
[`the-felt-arc.md`](the-felt-arc.md) for what it feels like to run,
[`cli-reference.md`](cli-reference.md#propose) for the command,
[`library-api.md`](library-api.md) to call it as an API, and
[`the-keystone-explained.md`](the-keystone-explained.md) for why the safety line
holds.

**New in v0.6 — the maturing organism.** The organs that let a *learned* class
mature and be curated over its life — a persistent life-record, an
affinity-maturation engine, an honest-blind drift-detector, and a conservative
forget-gate — ship today as a **library** (`antigen::learn`); the `cargo antigen`
verb that drives the full curation loop is the v0.7 frontier. See
[`concepts.md`](concepts.md#drift-detection-the-maturing-organism) for the organ
tour and [`library-api.md`](library-api.md#drift-detection-driftverdict-adr-065)
for the `DriftVerdict` surface.

### "Something isn't working; help me debug"

- **[`troubleshooting.md`](troubleshooting.md)** — diagnostic guide
  for common audit/scan output

### "I'm an LLM agent collaborating on antigen-using code"

- **[`for-llm-collaborators.md`](for-llm-collaborators.md)** —
  protocol for AI agents reading and writing antigen-using code

### "I'm deciding whether antigen fits my team"

- **[`../README.md`](../README.md)** — project framing + value
  proposition
- **[`concepts.md`](concepts.md)** — architectural overview
- **[`case-study.md`](case-study.md)** — what adopting antigen
  actually looks like, narrated through a real failure-class
- **[`composition.md`](composition.md)** — how antigen fits alongside
  the tools you already use
- **[`roadmap.md`](roadmap.md)** — what's shipped (through the v0.6 maturing organism) and what's coming
- **[`scope.md`](scope.md)** — comprehensive vision; multi-paper
  publication trajectory; cross-domain convergence
- **[`vision-pitch.md`](vision-pitch.md)** — ecosystem-outreach pitch
- **[`structural-memory.md`](structural-memory.md)** — whitepaper:
  what antigen is, why it exists, what it means for software
  teams collaborating across human and AI cognition

### "I'm a researcher or want the design substrate"

- **[`origin.md`](origin.md)** — the founding incident; the
  determinism-class / commutativity-class post-mortem
- **[`decisions.md`](decisions.md)** — ratified ADRs and amendments
- **[`postures.md`](internal/postures.md)** — architectural postures (seven
  postures threaded through the ADRs)
- **[`process.md`](internal/process.md)** — formal ADR lifecycle and
  governance
- **[`testing-patterns.md`](testing-patterns.md)** — when/how
  testing-and-antigen co-operate
- **[`cross-domain-architectural-map.md`](internal/cross-domain-architectural-map.md)**
  — 16+ academic fields converging on the same architectural class
- **[`immune-system-primitive-map.md`](internal/immune-system-primitive-map.md)**
  — comprehensive biology primitive catalog
- **[`contact-graph-and-recognition-tiers.md`](internal/contact-graph-and-recognition-tiers.md)**
  — 3-tier × 7-mode recognition framework

### "I want the project's roadmap"

- **[`roadmap.md`](roadmap.md)** — what's shipped, what's planned,
  what's aspirational; multi-language extension; cross-tier surfaces

### "I want to contribute"

- **[`../CONTRIBUTING.md`](../CONTRIBUTING.md)** — contribution guide
- **[`roadmap.md`](roadmap.md)** — where contributions matter most
- **[`postures.md`](internal/postures.md)** — the architectural postures
  contributions should thread through
- **[`process.md`](internal/process.md)** — ADR lifecycle for proposing
  architectural changes

---

## Document map

A flat catalog of every doc with one-line purpose:

### User-facing concept + tutorial

| Doc | Purpose |
|---|---|
| [`quickstart.md`](quickstart.md) | 5-minute taste before committing |
| [`concepts.md`](concepts.md) | What antigen is, architecturally |
| [`tutorial.md`](tutorial.md) | First 15 minutes, end-to-end |
| [`case-study.md`](case-study.md) | End-to-end narrative of a real failure-class |
| [`case-study-determinism-class.md`](case-study-determinism-class.md) | The founding determinism-class case study (how antigen would have caught it) |
| [`examples-guide.md`](examples-guide.md) | Progressive walkthrough of `antigen/examples/` |
| [`the-felt-arc.md`](the-felt-arc.md) | The learning core as you live it — dread → propose → route-to-human, four beats |
| [`the-learning-loop.md`](the-learning-loop.md) | Where `propose` sits in the system — one organ in a living loop |
| [`the-keystone-explained.md`](the-keystone-explained.md) | Why the learning core routes to a human — what `cargo antigen propose` is, the safety line, from first principles |
| [`composition.md`](composition.md) | How antigen composes with clippy, proptest, kani/prusti/verus, etc. |
| [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) | Placement conventions |
| [`usage-patterns.md`](usage-patterns.md) | Common patterns / cookbook |
| [`anti-patterns.md`](anti-patterns.md) | Common mistakes + correct shape |
| [`diagrams.md`](diagrams.md) | Mermaid visuals for vocabulary, flow, architecture |

### User-facing reference

| Doc | Purpose |
|---|---|
| [`cli-reference.md`](cli-reference.md) | The whole `cargo antigen` command surface, one line per subcommand |
| [`macros.md`](macros.md) | Five macros' full attribute syntax |
| [`stdlib-families.md`](stdlib-families.md) | Scan-and-find catalog of the shipped stdlib failure-class families (what each catches, tier, fingerprint, example) |
| [`fingerprint-grammar.md`](fingerprint-grammar.md) | Fingerprint DSL |
| [`witness-tiers.md`](witness-tiers.md) | `WitnessTier` gradient semantics |
| [`output-formats.md`](output-formats.md) | scan/audit human + JSON output |
| [`editor-integration.md`](editor-integration.md) | Wire scan into your editor (rust-analyzer flycheck) — the `--message-format json` reference |
| [`troubleshooting.md`](troubleshooting.md) | Diagnostic guide |
| [`glossary.md`](glossary.md) | Vocabulary anchor |
| [`roadmap.md`](roadmap.md) | Trajectory + planned features |

### Newcomer onboarding — reading antigen's output

| Doc | Purpose |
|---|---|
| [`getting-started.md`](getting-started.md) | Your literal first session, narrated step-by-step with every command run for real (install → first scan → read a finding → wire flycheck) |
| [`reading-a-verdict.md`](reading-a-verdict.md) | Decoder: what each scan/audit line means (read before your first scan) |
| [`i-scanned-and.md`](i-scanned-and.md) | Symptom-indexed FAQ ("I scanned and ___") |
| [`three-places-to-see-it.md`](three-places-to-see-it.md) | Where each thing (class-defense, fingerprint-spare, bind/spare) is actually visible |

### Adopter operations

| Doc | Purpose |
|---|---|
| [`immune-migration-guide.md`](immune-migration-guide.md) | Migrate deprecated `#[immune]` → `#[defended_by]` / `#[presents(requires=)]` |
| [`migrating-0.3-to-0.4.md`](migrating-0.3-to-0.4.md) | Upgrading from 0.3: what's new, what to turn on (additive — nothing breaks) |
| [`deployment-ci-integration.md`](deployment-ci-integration.md) | Wire `cargo antigen audit` into CI (exit codes, gating, GitHub Actions) + editor/IDE flycheck (`--message-format json`, rust-analyzer) |
| [`meta-finding-pattern.md`](meta-finding-pattern.md) | When a team's recurring drift becomes a typed antigen — the next-layer-up discipline |

### Failure-class deep dives

| Doc | Purpose |
|---|---|
| [`pathology/`](pathology/) | Per-family case-files (Presentation → Etiology → … → Prognosis) |
| [`the-failure-class-cookbook.md`](the-failure-class-cookbook.md) | Intent→defense recipes ("you have untrusted JSON →") |
| [`the-immune-system-a-programmers-guide.md`](the-immune-system-a-programmers-guide.md) | The biology cognate as a narrative course |

### War stories

| Doc | Purpose |
|---|---|
| [`war-stories/the-self-catch.md`](war-stories/the-self-catch.md) | Antigen catching itself — the thesis turned inward, every catch git-traceable |
| [`war-stories/learning-from-its-own-wounds.md`](war-stories/learning-from-its-own-wounds.md) | Antigen proposing a new failure-class from a cluster of its own defective sites — and refusing to promote it until it spares known-clean code |

### Co-native

| Doc | Purpose |
|---|---|
| [`for-llm-collaborators.md`](for-llm-collaborators.md) | Protocol for AI agents |

### Vision + framing

| Doc | Purpose |
|---|---|
| [`origin.md`](origin.md) | The founding incident narrative |
| [`scope.md`](scope.md) | Comprehensive vision |
| [`vision-pitch.md`](vision-pitch.md) | Ecosystem outreach pitch |
| [`structural-memory.md`](structural-memory.md) | Whitepaper: structural memory of failure-classes |

### Architecture + governance

| Doc | Purpose |
|---|---|
| [`decisions.md`](decisions.md) | Ratified ADRs |
| [`postures.md`](internal/postures.md) | Architectural postures |
| [`process.md`](internal/process.md) | ADR lifecycle |
| [`testing-patterns.md`](testing-patterns.md) | Testing and antigen together |

### Research substrate

| Doc | Purpose |
|---|---|
| [`cross-domain-architectural-map.md`](internal/cross-domain-architectural-map.md) | Academic convergence map |
| [`immune-system-primitive-map.md`](internal/immune-system-primitive-map.md) | Biology primitive catalog |
| [`contact-graph-and-recognition-tiers.md`](internal/contact-graph-and-recognition-tiers.md) | 3-tier × 7-mode recognition framework |

The design substrate above — postures, ADR lifecycle, development conventions,
and research maps — lives under [`internal/`](internal/README.md).

---

## How the docs relate to each other

Different docs for different jobs:

- **Quickstart** is the 5-minute taste; **tutorial** is the 15-minute
  walkthrough
- **Concepts** explains the *why* and *what*; **diagrams** is the
  visual companion
- **Case study** is the narrative version of "what adopting antigen
  feels like"; **examples-guide** is the curated tour of the shipped
  example code
- **References** (macros, fingerprint-grammar, witness-tiers, output-
  formats) describe the *exact syntax and behavior*
- **Patterns** (usage-patterns, where-to-look) describe *how to apply*;
  **anti-patterns** describes *what not to do*
- **Composition** describes how antigen sits alongside the rest of
  the Rust toolchain (clippy, proptest, kani/prusti/verus, phantom
  types, tests, ADR culture, CI)
- **Troubleshooting** helps when something goes wrong
- **Vision** (scope, vision-pitch, roadmap, structural-memory) is for
  adopters and partners deciding fit
- **Architecture** (decisions, postures, process) is for those
  shaping the project itself
- **Research substrate** (cross-domain, immune-system primitive,
  contact-graph) is for researchers and deep-dive readers

Most adopters need: README + quickstart + tutorial + macros + where-
to-look. The rest is available when you need it.

---

## Substrate-currency note

This index is maintained alongside the docs themselves. When a new
doc lands, this index updates. When a doc is renamed or retired, this
index updates. If you find a discrepancy — a doc listed here that
doesn't exist, or a doc that exists but isn't listed — please open an
issue or submit a fix (see [`postures.md`](internal/postures.md) §1).

---

## A note on doc-vs-substrate

Per ADR-006 (recognition-not-design), the project maintains a
distinction between **documentation** (these `.md` files) and
**substrate** (the code, the ratified ADRs, the workspace
configuration). Documentation describes substrate; substrate is the
source of truth.

When this index and the actual `docs/` directory contents differ, the
directory contents are authoritative — this index is corrected.

Same discipline applies to all doc cross-references: when documentation
and code differ, the code is authoritative.

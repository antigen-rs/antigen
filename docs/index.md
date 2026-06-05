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

Already past quickstart? Continue with **[`tutorial.md`](tutorial.md)**
for the full first-15-minutes walkthrough (declare → scan → defend →
audit, end-to-end).

The shortest possible version, inline:

```sh
cargo install cargo-antigen
cd /path/to/your/rust/project
cargo antigen scan
```

On a fresh codebase with no antigens declared yet, this returns clean.
Then add the dependency and declare your first antigen — quickstart
shows you how:

```toml
[dependencies]
antigen = "=0.3.0-beta.2"   # v0.3 prerelease — has the prescriptive family; v0.2.0 is the current stable
```

---

## Pick your path

Different starting points for different people.

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
  `audit` / `attest` / `tolerate` / `oracle` human + JSON output reference
- **[`glossary.md`](glossary.md)** — every project term anchored to
  its biological referent and Rust analog

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
- **[`roadmap.md`](roadmap.md)** — trajectory and what's coming
- **[`scope.md`](scope.md)** — comprehensive vision; multi-paper
  publication trajectory; cross-domain convergence
- **[`vision-pitch.md`](vision-pitch.md)** — ecosystem-outreach pitch
- **[`structural-memory.md`](structural-memory.md)** — whitepaper
  (V0): what antigen is, why it exists, what it means for software
  teams collaborating across human and AI cognition

### "I'm a researcher or want the design substrate"

- **[`origin.md`](origin.md)** — the founding incident; the
  tambear `DeterminismClass` / `CommutativityClass` post-mortem
- **[`decisions.md`](decisions.md)** — ratified ADRs (through
  ADR-018 + amendments + the v0.2 architectural-posture-shift batch:
  AMEND-002/003/006 + NEW-022/023/024/025/026/027/028)
- **[`postures.md`](postures.md)** — architectural postures (seven
  postures threaded through the ADRs)
- **[`process.md`](process.md)** — formal ADR lifecycle and
  governance
- **[`testing-patterns.md`](testing-patterns.md)** — when/how
  testing-and-antigen co-operate
- **[`cross-domain-architectural-map.md`](cross-domain-architectural-map.md)**
  — 16+ academic fields converging on the same architectural class
- **[`immune-system-primitive-map.md`](immune-system-primitive-map.md)**
  — comprehensive biology primitive catalog
- **[`contact-graph-and-recognition-tiers.md`](contact-graph-and-recognition-tiers.md)**
  — 3-tier × 7-mode recognition framework
- **[`expedition/`](expedition/)** — design substrate (in flight;
  pre-ratification material)

### "I want the project's roadmap"

- **[`roadmap.md`](roadmap.md)** — what's shipped, what's planned,
  what's aspirational; multi-language extension; cross-tier surfaces

### "I want to contribute"

- **[`../CONTRIBUTING.md`](../CONTRIBUTING.md)** — contribution guide
- **[`roadmap.md`](roadmap.md)** — where contributions matter most
- **[`postures.md`](postures.md)** — the architectural postures
  contributions should thread through
- **[`process.md`](process.md)** — ADR lifecycle for proposing
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
| [`composition.md`](composition.md) | How antigen composes with clippy, proptest, kani/prusti/verus, etc. |
| [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) | Placement conventions |
| [`usage-patterns.md`](usage-patterns.md) | Common patterns / cookbook |
| [`anti-patterns.md`](anti-patterns.md) | Common mistakes + correct shape |
| [`diagrams.md`](diagrams.md) | Mermaid visuals for vocabulary, flow, architecture |

### User-facing reference

| Doc | Purpose |
|---|---|
| [`macros.md`](macros.md) | Five macros' full attribute syntax |
| [`stdlib-families.md`](stdlib-families.md) | Scan-and-find catalog of the shipped stdlib failure-class families (what each catches, tier, fingerprint, example) |
| [`fingerprint-grammar.md`](fingerprint-grammar.md) | Fingerprint DSL |
| [`witness-tiers.md`](witness-tiers.md) | `WitnessTier` gradient semantics |
| [`output-formats.md`](output-formats.md) | scan/audit human + JSON output |
| [`troubleshooting.md`](troubleshooting.md) | Diagnostic guide |
| [`glossary.md`](glossary.md) | Vocabulary anchor |
| [`roadmap.md`](roadmap.md) | Trajectory + planned features |

### Newcomer onboarding — reading antigen's output

| Doc | Purpose |
|---|---|
| [`reading-a-verdict.md`](reading-a-verdict.md) | Decoder: what each scan/audit line means (read before your first scan) |
| [`i-scanned-and.md`](i-scanned-and.md) | Symptom-indexed FAQ ("I scanned and ___") |
| [`three-places-to-see-it.md`](three-places-to-see-it.md) | Where each thing (class-defense, fingerprint-spare, bind/spare) is actually visible |

### Adopter operations

| Doc | Purpose |
|---|---|
| [`immune-migration-guide.md`](immune-migration-guide.md) | Migrate deprecated `#[immune]` → `#[defended_by]` / `#[presents(requires=)]` |
| [`deployment-ci-integration.md`](deployment-ci-integration.md) | Wire `cargo antigen audit` into CI (exit codes, gating, GitHub Actions) |

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
| [`structural-memory.md`](structural-memory.md) | Whitepaper: structural memory of failure-classes (V0) |

### Architecture + governance

| Doc | Purpose |
|---|---|
| [`decisions.md`](decisions.md) | Ratified ADRs |
| [`postures.md`](postures.md) | Architectural postures |
| [`process.md`](process.md) | ADR lifecycle |
| [`testing-patterns.md`](testing-patterns.md) | Testing and antigen together |

### Research substrate

| Doc | Purpose |
|---|---|
| [`cross-domain-architectural-map.md`](cross-domain-architectural-map.md) | Academic convergence map |
| [`immune-system-primitive-map.md`](immune-system-primitive-map.md) | Biology primitive catalog |
| [`contact-graph-and-recognition-tiers.md`](contact-graph-and-recognition-tiers.md) | 3-tier × 7-mode recognition framework |
| [`expedition/`](expedition/) | Pre-ratification design substrate |

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
  contact-graph, expedition) is for researchers and deep-dive readers

Most adopters need: README + quickstart + tutorial + macros + where-
to-look. The rest is available when you need it.

---

## Substrate-currency note

This index is maintained alongside the docs themselves. When a new
doc lands, this index updates. When a doc is renamed or retired, this
index updates. If you find a discrepancy — a doc listed here that
doesn't exist, or a doc that exists but isn't listed — please open an
issue or submit a fix; substrate-over-memory is the discipline (see
[`postures.md`](postures.md) §1).

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

# Antigen — Documentation

> Welcome to antigen. This is the documentation index. Pick a path
> below depending on what you want to do.

For the project's README (install, quickstart, project framing), see
[`../README.md`](../README.md).

---

## I just want to try antigen

Five-minute path:

```sh
cargo install cargo-antigen
cd /path/to/your/rust/project
cargo antigen scan
```

On a fresh codebase with no antigens declared yet, this returns clean.
Now add the dependency and declare your first antigen:

```toml
[dependencies]
antigen = "=0.1.0-rc.1"
```

Then follow **[`tutorial.md`](tutorial.md)** for the full first-15-
minutes walkthrough.

---

## Pick your path

Different starting points for different people.

### "I'm new to antigen; show me what it is and how to use it"

Read in order:

1. **[`concepts.md`](concepts.md)** — what antigen is, the third
   pillar framing, the vocabulary, the seven components, the biology
   cognate, the three adopter pathways
2. **[`tutorial.md`](tutorial.md)** — your first 15 minutes, end-to-end
3. **[`where-to-look-for-antigens.md`](where-to-look-for-antigens.md)**
   — conventions for locating antigen declarations in your project
4. **[`usage-patterns.md`](usage-patterns.md)** — common patterns for
   real failure-classes

### "I want a reference for a specific thing"

- **[`macros.md`](macros.md)** — full reference for `#[antigen]`,
  `#[presents]`, `#[immune]`, `#[descended_from]`,
  `#[antigen_tolerance]`
- **[`fingerprint-grammar.md`](fingerprint-grammar.md)** — fingerprint
  DSL (six operators + composition)
- **[`witness-tiers.md`](witness-tiers.md)** — `WitnessTier` gradient
  semantics (FormalProof / Execution / Reachability / None) + audit
  hints
- **[`output-formats.md`](output-formats.md)** — `cargo antigen scan`
  and `cargo antigen audit` human + JSON output reference
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
- **[`roadmap.md`](roadmap.md)** — trajectory and what's coming
- **[`scope.md`](scope.md)** — comprehensive vision; multi-paper
  publication trajectory; cross-domain convergence
- **[`vision-pitch.md`](vision-pitch.md)** — ecosystem-outreach pitch

### "I'm a researcher or want the design substrate"

- **[`origin.md`](origin.md)** — the founding incident; the
  tambear `DeterminismClass` / `CommutativityClass` post-mortem
- **[`decisions.md`](decisions.md)** — ratified ADRs (through
  ADR-018 + amendments)
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
| [`concepts.md`](concepts.md) | What antigen is, architecturally |
| [`tutorial.md`](tutorial.md) | First 15 minutes, end-to-end |
| [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) | Placement conventions |
| [`usage-patterns.md`](usage-patterns.md) | Common patterns / cookbook |

### User-facing reference

| Doc | Purpose |
|---|---|
| [`macros.md`](macros.md) | Five macros' full attribute syntax |
| [`fingerprint-grammar.md`](fingerprint-grammar.md) | Fingerprint DSL |
| [`witness-tiers.md`](witness-tiers.md) | `WitnessTier` gradient semantics |
| [`output-formats.md`](output-formats.md) | scan/audit human + JSON output |
| [`troubleshooting.md`](troubleshooting.md) | Diagnostic guide |
| [`glossary.md`](glossary.md) | Vocabulary anchor |
| [`roadmap.md`](roadmap.md) | Trajectory + planned features |

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

- **Concepts** explains the *why* and *what*
- **Tutorial** walks you through the *how*, end-to-end
- **References** (macros, fingerprint-grammar, witness-tiers, output-
  formats) describe the *exact syntax and behavior*
- **Patterns** (usage-patterns, where-to-look) describe *how to apply*
- **Troubleshooting** helps when something goes wrong
- **Vision** (scope, vision-pitch, roadmap) is for adopters and
  partners deciding fit
- **Architecture** (decisions, postures, process) is for those
  shaping the project itself
- **Research substrate** (cross-domain, immune-system primitive,
  contact-graph, expedition) is for researchers and deep-dive readers

Most adopters need: README + tutorial + macros + where-to-look. The
rest is available when you need it.

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
**substrate** (the code, the ADRs, the campsites, the sweep records).
Documentation describes substrate; substrate is the source of truth.

When this index and the actual `docs/` directory contents differ, the
directory contents are authoritative — this index is corrected.

Same discipline applies to all doc cross-references: when documentation
and code differ, the code is authoritative.

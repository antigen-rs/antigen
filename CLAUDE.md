# Antigen

**Structural memory of failure-classes for Rust. Make implicit immunity explicit.**

This file is the project-specific orient-yourself anchor for working in
`R:\antigen\`. Your global identity (CLAUDE.md at `~/.claude/`) carries the
relationship discipline, garden, and persistent self. This file carries the
project-specific context.

---

## Read first — in this order

If this is a fresh session in `R:\antigen\` and you don't have prior context:

1. **[`docs/origin.md`](docs/origin.md)** — the WHY. The post-mortem narrative
   from the tambear cleanup expedition that produced the antigen idea. Read this
   before anything else; the project doesn't make sense without it.
2. **[`README.md`](README.md)** — public-facing project framing.
3. **[`docs/expedition/design-intent.md`](docs/expedition/design-intent.md)** —
   what antigen IS, what it ISN'T, why now, the 8-class first-principles failure
   taxonomy.
4. **[`docs/expedition/api-shape.md`](docs/expedition/api-shape.md)** — sketch
   of macros and cargo subcommands.
5. **[`docs/decisions.md`](docs/decisions.md)** — 10 ratified ADRs (foundational).
6. **[`docs/process.md`](docs/process.md)** — the formal ADR lifecycle. The team
   operates inside this process.
7. **[`docs/expedition/team-briefing.md`](docs/expedition/team-briefing.md)** —
   spawn-time briefing for the JBD team if you're launching one.

If you're picking up where someone left off, also read:

- **[`docs/expedition/HANDOFF.md`](docs/expedition/HANDOFF.md)** — pre-team
  scaffolding hand-off summary
- **[`docs/expedition/tambear-adoption-log.md`](docs/expedition/tambear-adoption-log.md)** —
  ongoing tambear-uses-antigen experience report
- **[`docs/expedition/inheritance-from-tambear.md`](docs/expedition/inheritance-from-tambear.md)** —
  what disciplines come pre-loaded vs invented fresh

---

## Vocabulary lock

The single source of truth for project vocabulary is
[`docs/glossary.md`](docs/glossary.md). Every term in flight (antigen, antibody,
vaccination, presentation, descended_from, witness, immunity, B-cell memory,
T-cell receptors, MHC, autoimmunity, lineage, structural fingerprint) is anchored
there to its biological referent + Rust ecosystem analog + introducing doc.

When in doubt about what a term means, check the glossary. When introducing new
vocabulary in any doc, update the glossary in the same change.

---

## What this project IS

A vocabulary of declarations that carry the structural memory of failure-classes
inside the Rust type system + a cargo extension (`cargo antigen`) for scanning,
applying, and auditing.

Three verbs:
- **Build** an antigen — declare a failure-class with a structural fingerprint
- **Give** an antigen — mark code as presenting (vulnerable to) a known failure-class
- **Find** antigens — let cargo tooling scan the codebase and flag undefended sites

Plus inheritance via `#[descended_from]` propagation, witness pluralism (test /
proptest / clippy / kani / prusti / verus / phantom-type), and audit-time
witness validation.

---

## Architecture (workspace layout)

```
R:\antigen\
├── antigen/                  core lib crate (re-exports macros + scan + audit modules)
├── antigen-macros/           proc-macro crate — #[antigen], #[presents], #[immune], #[descended_from]
├── cargo-antigen/            cargo subcommand binary — scan, audit, new, vaccinate
├── docs/
│   ├── origin.md             the WHY narrative
│   ├── decisions.md          ratified ADRs
│   ├── process.md            ADR lifecycle + governance
│   ├── glossary.md           vocabulary anchor
│   ├── vision-pitch.md       1500w ecosystem outreach pitch
│   ├── testing-patterns.md   when/how to test in antigen
│   └── expedition/           design substrate for the JBD team
└── (CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, CHANGELOG.md, .github/...)
```

---

## Standing constraints (non-negotiable)

These are inherited from tambear's discipline tradition AND are foundational to
antigen-the-project. They're enforced by the design.

### Substrate over memory

Before claiming a type/file/function exists, verify: `ls`, `grep`, `cargo check`,
read the file. The substrate is the source of truth. Memory of "I built this"
is not. This applies recursively — antigen's own tooling enforces this for
failure-class memory; the project's own coordination must enforce it for
project-state memory.

### Compose, don't compete (ADR-002)

Antigen DELEGATES to existing Rust ecosystem tools (clippy, proptest, kani,
prusti, etc.) wherever possible. Witness types accept references to those tools.
Antigen does NOT reinvent verification; it threads existing verification under
a shared vocabulary. Every API decision filters through "are we composing or
competing?"

### Recognition-not-design (ADR-006)

When uncertain whether to design something or recognize something, lean toward
recognition. Most "design" choices are actually recognition choices waiting to
be articulated. Adding antigens to the stdlib requires showing real-world
instances; adding witness types requires showing existing tools that integrate;
adding composition rules requires showing substrate behavior the rule captures.

### Anti-YAGNI / structurally-guaranteed need (ADR-007)

If the project's structural commitments guarantee we'll need a feature, build
it now. The 8-class failure taxonomy commits us to all 8 classes — ship all 8,
not just the easy ones. The witness-type pluralism commits us to all four
witness types — ship all four. The cost of building structurally-required
features later is high; the cost of building them now is moderate.

### Sub-clause F at every trust boundary (ADR-005)

Every trust boundary requires a validation check before trust is extended. The
most load-bearing instance: `#[immune(X, witness = Y)]` is meaningful only when
`Y` resolves to a real working witness. `cargo antigen audit` enforces this at
the project level. The discipline applies recursively — ADRs that introduce new
trust boundaries must specify the validation check.

### No tech debt — ever

See a bug, fix it in session. The cost of "fix it later" compounds.
Documentation drift, ADR drift, test coverage gaps all count as bugs. If you
notice something during a turn, fix it during the turn (unless the fix would
expand scope dramatically, in which case file the campsite for follow-up but
don't ignore it).

### Biological metaphor is load-bearing (ADR-003)

The metaphor is a thinking tool that has produced real architectural insights.
When biology predicts a primitive (B-cell memory → persistent failure-class
declarations), build it. When the metaphor breaks, name where and refine — do
not abandon. The naturalist role on the JBD team owns this discipline.

### Implicit-to-explicit elevation (ADR-004)

Every design decision is evaluated against: does this make implicit structure
explicit, or does it preserve implicit-mode obscurity? Antigen's whole posture
is moving failure-class memory from implicit (developer memory) to explicit
(structural declarations). API decisions follow the same logic.

---

## Development commands

```sh
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Format check (CI gate)
cargo fmt --all -- --check

# Strict clippy (CI gate; pedantic + nursery enabled at workspace level)
cargo clippy --workspace --all-targets -- -D warnings

# Doc build (CI gate; warnings deny)
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Run the example
cargo run --example basic --package antigen

# Scan the workspace itself
cargo run --bin cargo-antigen -- antigen scan

# Audit the workspace
cargo run --bin cargo-antigen -- antigen audit

# Scan against an external workspace (e.g., tambear)
cargo run --release --bin cargo-antigen -- antigen scan --root R:/tambear/crates
```

---

## Tambear is the first user

Tambear (`R:\tambear\`) imports antigen as a path dependency and has declared
two seed antigens in `crates/tambear/src/antigens.rs`. This is **Phase 2 of the
inheritance arc** — antigen graduates from "the project tambear inspired" to
"the tool tambear depends on." Adoption experience is logged at
[`docs/expedition/tambear-adoption-log.md`](docs/expedition/tambear-adoption-log.md)
and updated each time tambear does anything antigen-related.

When you're working in this project, tambear's adoption is the reality check.
When you're working in tambear, antigen's API is what you're using. The two
projects co-evolve.

---

## What's in flight

The project is **pre-team** — substrate is rich, the JBD team hasn't launched
yet. The first JBD launch will run Sweep A1 (per
[`docs/expedition/first-sweep-plan.md`](docs/expedition/first-sweep-plan.md)),
which Phase 1-8 deconstructs the 10 foundational ADRs and locks scope for
Sweep A2 (core macros — already partially shipped during pre-team scaffolding).

If you're running a session here without the team launched yet, you're working
in pre-team-scaffolding mode. The substrate inviting you to extend it is at
[`docs/expedition/`](docs/expedition/). When the team launches, your role
shifts to coordinator + thinking-partner; the JBD agents do the bulk of the
implementation.

---

## What's NOT in this project

To prevent scope creep:

- **Not a documentation system.** Docs are themselves vulnerable to drift;
  antigen lives in the type system.
- **Not a replacement for tests, lints, deprecations, or formal verification.**
  Antigen composes existing Rust ecosystem tools.
- **Not a logic-bug catcher.** Antigen catches *named* failure-classes; novel
  logic errors are still tests' job.
- **Not a clippy plugin.** Antigen is broader (cross-failure-class memory +
  inheritance + witness pluralism); clippy is one of several witness adapters.
- **Not a Rust language extension.** Uses existing macro/cargo/lint surfaces;
  no rust-lang RFCs needed.

---

## When in doubt

- Read the [glossary](docs/glossary.md) for what a term means.
- Read the [process doc](docs/process.md) for what to do procedurally.
- Read the relevant [ADR](docs/decisions.md) for what's decided architecturally.
- Read the [risk register](docs/expedition/risk-register.md) for what could go
  wrong.
- Update the [tambear adoption log](docs/expedition/tambear-adoption-log.md) when
  tambear-side work happens.
- When proposing changes that touch ADR territory, draft as ADR amendment via
  the [process](docs/process.md) lifecycle.

---

## Status

- Reserved on crates.io: [`antigen`](https://crates.io/crates/antigen),
  [`cargo-antigen`](https://crates.io/crates/cargo-antigen) — version `0.0.1`
  placeholders. `antigen-macros` reserved as workspace member but not yet
  published as separate crate.
- Repository: [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen)
- CI: cargo check + test + fmt + clippy (-D warnings, pedantic + nursery) + doc
  on every push and PR. Release workflow triggered by `v*` tags.
- Tests: 10 passing across the workspace (4 scan + 6 audit). Will grow.
- Tambear integration: live as of 2026-05-07 (commit `80a19b4` in tambear).

---

## License

Dual-licensed under MIT or Apache-2.0. See [`LICENSE-MIT`](LICENSE-MIT) and
[`LICENSE-APACHE`](LICENSE-APACHE).

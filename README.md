# Antigen

**The immune system for software development in the era where generation outpaces inspection.**

Comprehensive, co-native, structural memory of failure-classes, defenses, attestations, and coordination — accessible natively to both human and AI agents. Built for the age of agentic dev, vibe-coding, and human-LLM collaboration.

> **Status**: a working **beta** (`0.5.0-beta.1`), published on crates.io. The
> surface: core macros, fingerprint grammar (parse · match · serialize), the
> `scan + propose + audit + attest + tolerate + oracle + verify + vcs +
> mucosal-map + fingerprint + mine` CLI, Oracle 5-state
> lifecycle, cross-cutting attestation, substrate-witness predicates, ADR-029
> observe-don't-declare vocabulary (`#[defended_by]`, extended `#[presents]`),
> Match3 three-valued fingerprint evaluation, the prescriptive work-orchestration
> family (8 macros), the titer/scalar witness kind, live-projection reporting,
> member-aware multi-crate scan, **eight failure-class stdlib families** that
> ship members (11 members total) **plus a chartered ninth** (crypto-misuse — the
> class is named, no honest call-only fingerprint exists yet), and the
> `#[aura]`/`#[dread]`/`#[red_flag]` marked-unknown markers — catalogued in
> [`docs/stdlib-families.md`](docs/stdlib-families.md). For the published version
> and per-release manifest see [`CHANGELOG.md`](CHANGELOG.md) and
> [crates.io](https://crates.io/crates/antigen).

> **What's new in v0.6 — the maturing organism.** Beyond declaring and scanning
> failure-classes, antigen is growing the organs to let a *learned* class mature
> and be curated over its life: a persistent life-record (the class's
> autobiography), an affinity-maturation engine, a drift-detector that honestly
> says "I can't see drift yet" rather than guessing, and a conservative forget-gate.
> These ship today as a **library** (`antigen::learn`) — typed, tested, composable;
> the `cargo antigen` verb that drives the full curation loop end-to-end is the v0.7
> frontier. See [`docs/concepts.md`](docs/concepts.md#drift-detection-the-maturing-organism)
> for the organ tour and [`docs/library-api.md`](docs/library-api.md#drift-detection-driftverdict-adr-065)
> for the public surface.

---

## The problem antigen addresses

Modern software development is characterized by a structural asymmetry: **generation throughput vastly exceeds inspection throughput for all actor types.**

- **Humans** can't read all the code they ship, especially in AI-pair workflows. Read-speed is bounded; generation isn't.
- **Vibe coders** generate code they may not fully understand. The tooling that helped them generate has to help them validate.
- **LLM agents** can't track across sessions. Context resets; summarization drifts; the lesson from last week's fix isn't there for this week's code.
- **Human-LLM teams** generate faster than either type can fully inspect.
- **Docs, comments, ADRs, Slack** ship faster than they're read. The historical assumption that "the team has read everything" hasn't held in years; in 2026 it fails catastrophically.

This asymmetry is at a **historic maximum** and growing. There is no scaling solution within passive memory. More docs means less reading-per-doc. More comments means less attention-per-comment. RAG and embedding are probabilistic compensations — useful for retrieval, inadequate for *binding* discipline (deciding what's required, what's stale, who needs to attest, what's blocking).

**Antigen's reason for existing**: the asymmetry guarantees passive memory will fail. The only viable alternative is structural memory that surfaces itself.

---

## The mechanism: memory-to-structure transformation

Antigen converts the things you currently write as passive docs and comments into co-native structure that survives.

The table below shows the full transformation vocabulary antigen is building. It
spans the core observe-don't-declare vocabulary (`#[presents]` / `#[defended_by]` /
`#[antigen_tolerance]`), the deferred-defense and recurrent-emergence families, and
the prescriptive work-orchestration family (`#[panel]`, `#[ddx]`, …). For the
authoritative per-macro ship state see [`CHANGELOG.md`](CHANGELOG.md) and
[`docs/roadmap.md`](docs/roadmap.md); the point of the table is the *shape* of the
transformation. Every right-side stays current OR fails loudly when stale — that
property is the mechanism.

| Memory form (rots) | Structure form (surfaces itself) |
|---|---|
| `/// assumes X never panics` | `#[presents(X, requires = ...)]` + `#[defended_by(X)]` on test |
| README "we follow Y discipline" | `#[antigen(Y)]` + per-site `#[presents(Y, requires = ...)]` |
| `// Last reviewed: 2024-01-15` | `#[presents(..., requires = fresh_within_days(N))]` |
| `// intentional, don't touch` | `#[antigen_tolerance(rationale = "...")]` |
| Generated-code provenance | `#[presents(GeneratedCodeWithoutHumanAttestation, requires = signers([reviewer]))]` |
| `// TODO: refactor this` | `#[itch(...)]` or `#[panel(...)]` |
| `// FIXME: hack` | `#[anergy(rationale = "...")]` |
| `// HACK: until Q3` | `#[poxparty(until = "...")]` |
| Code review "did you consider Z?" | `#[ddx(rule_out = [Z, ...])]` |
| `// see ADR-017` | `#[orient(adr = "017")]` |
| Recurring Slack mention | `#[recurrence_anchor(...)]` |
| `// blocks on Bob's signoff` | `#[panel(reviewed_by = "bob")]` |
| `// TBD null handling` | `#[ddx(symptom = "null cases unspecified")]` |
| Asana ticket assignment | `#[panel(filled_by = "alice", reviewed_by = "bob")]` |
| "We keep hitting this in standup" | `#[recurrence_anchor(surfaced_in = [...])]` |

Every left-side rots. Every right-side stays current OR fails loudly when stale.

The approach is **co-native**: the structural form works natively for both humans and AI agents. Not RAG. Not fuzzy matching. Not an external dashboard. The macro is in the code where the discipline applies — substrate-resident, compiler-checked, exact.

---

## A concrete example

When you fix a bug, you learn something about *why* a class of code fails. That lesson lives in your head, in a commit message, in a Slack thread, in a docstring that drifts as the code evolves. None of those carriers are drift-resistant. Six months later, structurally identical code appears in another module — and the lesson is gone.

Antigen makes the lesson **structural**: declared in code, propagated through inheritance, checked by cargo tooling.

```rust
#[antigen(
    name = "panicking-in-drop",
    fingerprint = r#"item = impl, has_method("drop", "(& mut self)"), body_contains_macro("panic")"#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
)]
pub struct PanickingInDrop;
```

That declaration is the failure-class memory. `cargo antigen scan` finds every site in your codebase that structurally matches it. `cargo antigen audit` verifies that every claimed immunity has a working witness at the appropriate tier. The lesson is now durable substrate — not implicit knowledge.

When the same polarity-inversion bug appears in two sibling enums, antigen propagates the lesson from the first fix structurally to the second — the same illness does not ship twice. See [`docs/origin.md`](docs/origin.md) for the founding incident that motivated this.

---

## What antigen is NOT

- **Not a lint tool.** Clippy catches broad-spectrum patterns; antigen catches *named failure-classes* with structural fingerprints and delegated witness validation. They compose — clippy lints are valid witness types.
- **Not a testing tool.** Tests verify *this code does X*; antigen declares *this class of code has historically failed in this structural way*. Different artifact, different lifecycle, both necessary.
- **Not a documentation system.** Documentation drifts; antigen declarations are checked by cargo tooling. A stale docstring is invisible to CI; a stale antigen fingerprint produces a scan-time discrepancy.
- **Not RAG or vector search.** Probabilistic retrieval gives you "probably the relevant doc" — not "this IS the binding decision." Antigen provides exact, structural, substrate-resident memory.
- **Not an external dashboard.** Your Asana board drifts from your code. `#[panel(filled_by = "alice", reviewed_by = "bob")]` IS the coordination in the code where the work happens.
- **Not a replacement for tests, lints, or formal verification.** Antigen *composes* them — witness pluralism delegates to whichever tool proves immunity for a given antigen.

Antigen is a **third pillar** alongside testing and documentation. Both of those filled gaps that couldn't be filled with what preceded them. Antigen fills the gap of structural failure-class memory — which testing and documentation have historically tried to address as side-effects of their primary jobs.

---

## The core vocabulary

Five macros form antigen's core vocabulary:

- **`#[antigen(...)]`** — declare a named failure-class with a structural fingerprint
- **`#[presents(AntigenName)]`** — mark code as presenting (vulnerable to) a known failure-class; extended form `#[presents(AntigenName, requires = P, proof = P, min_tier = T)]` adds substrate evidence requirements
- **`#[defended_by(AntigenName)]`** — place on a test to declare that test is the *observed defense* for a presentation; this is the code-tier witness (ADR-029). `#[immune]` was the v0.1 equivalent and is now deprecated — use `#[defended_by]` on tests instead
- **`#[descended_from(Parent)]`** — declare structural inheritance between failure-classes
- **`#[antigen_tolerance(AntigenName, rationale = ...)]`** — explicitly tolerate a fingerprint match the team has reviewed

The shift from `#[immune]` to `#[defended_by]` reflects a key design correction (ADR-029): immunity is **observed** (a defense you can witness), not **declared** (a verdict you stamp). `#[defended_by(X)]` on a test says "this test IS the defense" — the claim is falsifiable and audit-verifiable. The old `#[immune(X)]` on the implementation site declared a verdict without a carrier; cargo antigen audit couldn't validate what it couldn't witness.

Plus the `cargo antigen` subcommands (run `cargo antigen --help` for the live list):

- **`cargo antigen scan`** — find unaddressed presentations across your codebase
- **`cargo antigen audit`** — verify every defense has a working witness at the appropriate tier; also renders the prescriptive work board (ADR-033)
- **`cargo antigen attest`** — substrate-witness sidecar management (`scaffold`, `sign`, `check`, `delta`, `list`, `gc`)
- **`cargo antigen tolerate`** — tolerance-ratification sidecar management
- **`cargo antigen oracle`** — Oracle artifact-class records (ADR-021)
- **`cargo antigen verify`** — supply-chain defense verifications: content-hash, dep-pin (ADR-025)
- **`cargo antigen vcs`** — VCS-information-loss observations, incl. recurrence mining (ADR-026)
- **`cargo antigen mucosal-map`** — map mucosal trust boundaries across the workspace (ADR-027)
- **`cargo antigen fingerprint`** — print the structural fingerprint of a scanned item
- **`cargo antigen propose`** — the learning verb: anti-unify a cluster of marked unknowns (`#[dread]`/`#[aura]`) into a candidate fingerprint, gated against an operator-supplied clean corpus (ADR-045/047/048). Renders a ratifiable suggestion — it observes, it does not name a class (the source tree is left byte-unchanged)

---

## The comprehensive vocabulary

Antigen's vocabulary spans several families, each grounded in the biological immune system. The metaphor is the systematic discovery framework for what each family needs to be — each immune-system component maps to a code discipline with its own primitive.

**Shipped today**: The five core macros above plus the deferred-defense, recurrent-emergence, mucosal-boundary, VCS-information-loss, agentic-coordination, and supply-chain families. See the vocabulary table above and [`docs/examples-guide.md`](docs/examples-guide.md) for pattern walkthroughs. Also shipped:

**Prescriptive / work-orchestration family** — `#[panel]`, `#[ddx]`, `#[rx]`, `#[triage]`, `#[refer]`, `#[biopsy]`, `#[culture]`, `#[quarantine]`. Code-site-local work-needs expressed directly in the type system — "code IS the Asana board." `cargo antigen audit` renders per-site verdicts (`Pending` / `Fulfilled` / `Overdue` / `OutOfFrame`) as a live-projected board section. See [`docs/macros.md`](docs/macros.md) for the full reference and [`docs/examples-guide.md`](docs/examples-guide.md) (`prescriptive_board` example) for a walkthrough with live audit output.

**Titer / scalar witness kind** — `#[ignorance]` (scan-coverage, member one) and raw `#[titer(source=...)]`. A second witness kind that attests a *measured value* (no verdict, trend-trackable) rather than a categorical defense verdict. Antigen reports the value; the threshold-judgment lives downstream. `#[ignorance]` / scan-coverage is retroactively recognized as member-one; no code change required.

**Eight failure-class stdlib families that ship members (11 members total)** — ready-to-import `#[antigen]` declarations covering common Rust footguns, each with an honest confidence tier (`named` = high-confidence, "if it doesn't fire you're covered"; `suspected` = a look-here correlator that may also fire on clean code; `chartered` = the class is named and tracked but **no member ships yet**, because no honest fingerprint exists in the current grammar):

| Family | Catches | Tier |
|---|---|---|
| deserialization-trust-boundary | streaming `from_reader` DoS · silent unknown-field drop | named · suspected |
| time-and-ordering-hazards | `SystemTime` clock-skew panic (silent in tests) | suspected |
| drop-and-panic-discipline | a `Drop` impl that can panic → process abort | named |
| panic-on-index | `get_unchecked` — out-of-bounds is UB, not a panic | named |
| resource-lifecycle-leak | `mem::forget` / `Box::leak` skipping `Drop` | suspected |
| async-soundness | a hand-written `unsafe impl Send/Sync` | named |
| numeric-truncation-overflow | `size_of`-in-element-count raw-copy foot-cannon | suspected |
| unsafe-soundness-boundary | `transmute` · uninit-assumed-init · `from_utf8_unchecked` | named (×3) |

Plus the **marked-unknown markers** `#[aura]` / `#[dread]` / `#[red_flag]` (ADR-041) — structural records for the danger you *feel* but can't yet name (they never gate, never nag). A ninth family, **crypto-misuse**, is *chartered* — the class is named, but no honest call-only fingerprint exists in the current grammar yet (better honest-deferred than dishonest-shipped). Full catalog with fingerprints + runnable examples (and scan fixtures for the two `unsafe`-primitive families): [`docs/stdlib-families.md`](docs/stdlib-families.md).

**On the roadmap** (see [`docs/roadmap.md`](docs/roadmap.md) for the full trajectory):

**Biological-component family** — `#[macrophage]`, `#[neutrophil]`, `#[treg]`, `#[complement]`, `#[dendritic]`, and ~30 more. Each maps to a real code discipline; each discovered via the biological metaphor, not speculation.

**Dysregulation states** — `#[immunodeficient]`, `#[immunocompromised]`, `#[sepsis]`. For regions of deliberate non-defense, made structurally explicit and auditable. *Autoimmunity* (the immune system over-firing on self) is deliberately **not** a site-marker — naming a marker `#[autoimmune]` would read backwards, since autoimmunity is the pathology, not the discipline. It is planned to surface instead as an audit-mode **screen** that flags fingerprints over-firing on their own clean siblings, with the per-site preventer being the already-shipped `#[antigen_tolerance]` (ADR-041 ruling).

The full vocabulary target is laid out in [`docs/vision-pitch.md`](docs/vision-pitch.md).

---

## Install and first scan

```sh
cargo install cargo-antigen   # installs the latest published release
```

Run `cargo antigen scan` in any Rust project. On a fresh codebase with no antigen
declarations of its own, antigen does **not** report a false all-clear — it
auto-injects its **bundled stdlib catalog** and surfaces real footgun
candidates from the shipped failure-classes:

```text
Scanning workspace: .

Scanned 1 files, found 2 antigen-related declarations:
  - 0 antigen declarations
  - 0 explicit #[presents] markers
  - 2 fingerprint matches (candidate sites — see below)

2 fingerprint match(es) across 2 antigen type(s) — candidate sites (expected noise; the witness layer refines them, per the filter/proof split). Not a TODO list.

  .\src\lib.rs:1  get-unchecked-without-proof on fn [fingerprint match]
  .\src\lib.rs:6  panic-in-drop on impl [fingerprint match]
```

These are **fingerprint matches to inspect, not audited verdicts**. `cargo antigen
scan --message-format json` emits editor flycheck for rust-analyzer. See
[`docs/quickstart.md`](docs/quickstart.md) and [`docs/output-formats.md`](docs/output-formats.md).

Add antigen as a dependency:

```toml
[dependencies]
antigen = "0.5.0-beta.1"   # check crates.io for the latest version
```

Now declare your first antigen. The full walkthrough lives in [`docs/tutorial.md`](docs/tutorial.md) — your first 15 minutes, end-to-end, with a real failure-class.

---

## The biology cognate

The biological metaphor is **load-bearing, not decorative**. The immune system is the most sophisticated pattern-recognition and response system biology has produced — evolved to handle an unbounded number of pathogen types, including pathogens that didn't exist when the host was born. The mapping to software fail-classes is structurally dense and predictive.

| Biology | Antigen analog |
|---|---|
| Pathogen Recognition Receptors (PRRs) | structural pattern matchers in `cargo antigen scan` |
| MHC Class I/II presentation | `#[presents(antigen)]` |
| B-cell memory (pattern layer) | `#[antigen(name = "...")]` declarations |
| Antibody | `#[defended_by(X)]` on a test — the observed defense, the code-tier witness (the API word for antibody is *witness*; see glossary) |
| Substrate sensing (germinal-center history, signed records) | `requires =` substrate-witness predicate — B-cell memory, commit trailers, oracle markers (ADR-019) |
| Serotiter (scalar magnitude) | `#[ignorance]` / `#[titer(source=...)]` — scalar witness kind; attests a measured value, no verdict (ADR-019 Amendment 1) |
| B-cell lineage (clonal expansion) | `#[descended_from]` propagation |
| Peripheral tolerance / Tregs | `#[antigen_tolerance]` for legitimate matches |
| Innate vs adaptive immunity | passive surface (fingerprint scan) vs active surface (explicit markers) |
| Antigenic drift | version-boundary recognition (ADR-017) |
| Deferred immunity / anergy | `#[anergy]`, `#[immunosuppress]` |
| Mucosal boundary defense | `#[mucosal]` — input validation primitives |
| Dysregulation | `#[sepsis]`, `#[anaphylaxis]` (an over-firing screen is on the roadmap — not a `#[autoimmune]` marker) |

When the biology predicts a primitive, the project builds it. See [`docs/decisions.md`](docs/decisions.md) (ADR-003) for the discipline.

---

## For whom

- **Teams shipping faster than they can manually review** — structural memory carries the lessons that review can't catch at throughput.
- **Adopters of AI coding assistants** — agents lose context between sessions; antigen makes failure-class memory survive session boundaries.
- **Multi-agent dev workflows** — shared substrate both humans and agents read natively, without translation layers.
- **Long-running codebases where institutional memory rots** — antigen declarations don't rot the same way docs and comments do.
- **Open-source maintainers managing contribution review at scale** — known failure-classes encoded structurally; new contributors collide with them before merge.
- **Anyone fighting docs/code drift** — the transformation table above is the tool.

---

## Why now

- **Generation-inspection asymmetry at historic maximum** (2026). AI-generated code volume rises faster than any inspection-capacity enhancement. This is the dev environment now.
- **Probabilistic compensation hits ceiling.** RAG, embedding, and fuzzy matching are useful for retrieval; they're inadequate for binding discipline. Structural memory is the only viable approach for sufficient-precision discipline.
- **Post-COVID vocabulary**: "antigen," "antibody," "vaccination" are everyday language; the biological metaphor is universally accessible.
- **Mature Rust ecosystem**: cargo extensions, proc-macros, custom diagnostics, proptest, and the formal-verification cohort (kani/prusti/verus/creusot/flux) are all stable enough to compose with rather than reinvent.
- **AI-coding era**: agents lose context between sessions. Implicit memory of failure patterns is no longer viable; structural memory is required.

---

## What's actually shipped

> For the current published version, see [crates.io](https://crates.io/crates/antigen)
> and the [GitHub releases](https://github.com/antigen-rs/antigen/releases) — those
> are the single source of truth, not this page.

**Core macros**: `#[antigen]`, `#[presents]`, `#[defended_by]`, `#[descended_from]`, `#[antigen_tolerance]`, plus extended `#[presents(X, requires = P, proof = P, min_tier = T)]` for substrate-witness predicates (ADR-019/ADR-029), plus `attested = (who, allowed_types, why, scope)` for cross-cutting attestation (ADR-020). `#[immune]` (v0.1) is deprecated — use `#[defended_by]` on tests for code-tier witnesses.

**Cargo subcommands**: `cargo antigen scan` (item-identity fingerprint matching, cross-crate scanning, cycle detection, diamond inheritance dedup); `cargo antigen audit` (`WitnessTier` gradient with `WitnessTier × AuditHint × EvidenceKind` output); `cargo antigen attest` (substrate-witness sidecar management); `cargo antigen tolerate` (tolerance-ratification sidecar management).

**Fingerprint grammar v1**: seven item-level operators plus `all_of`, `any_of`, `not` composition.

**Substrate-witness predicate language** (ADR-019): five sealed leaf primitives (`ratified_doc`, `signers`, `signed_trailer`, `oracles_complete`, `fresh_within_days`) with combinator grammar. JSON sidecars at `.attest/<Antigen>.json`.

**Oracle 5-state lifecycle**: `Draft → Active → Complete / Deprecated / Retired / Revoked + Reopened`.

**Anti-laundering safeguards**: chain-depth cap, minimum rationale character count, cumulative-root fingerprint for drift detection.

**Test coverage**: a broad workspace suite (unit + adversarial ATK + trybuild UI) gates every change in CI. See the CI badge / Actions for the live count.

Not yet shipped (honest "design phase" stubs in the CLI): `cargo antigen attest oracle`, `cargo antigen new`, `cargo antigen vaccinate`, `antigen-stdlib`.

**What `scan` / `audit` show you today — and what's coming.** `scan` reports declared *presentations* and fingerprint *candidates*; `audit` grades each presentation *defended* / *undefended* / *substrate-gap* at a witness tier; the `#[aura]` / `#[dread]` / `#[red_flag]` markers surface under `scan --format json` (`report.marked_unknowns`). Two surfaces are honestly **not** in the human report yet, and sit on the roadmap: a **per-site** fingerprint verdict (today a witness credits a failure-*class*, not the individual site it exercises), and a **console confidence-dial / marked-unknown rendering** (queryable via JSON today). So if you go looking for a per-site "this exact line is defended" verdict, or a dial in the console, and don't find it — that's the roadmap, not you. The catalog's [`docs/stdlib-families.md`](docs/stdlib-families.md) explains the current behavior in detail; [`docs/reading-a-verdict.md`](docs/reading-a-verdict.md) decodes what each line means.

See [`docs/roadmap.md`](docs/roadmap.md) for the planned trajectory.

---

## Where to go next

**If you're getting started:**
- [`docs/getting-started.md`](docs/getting-started.md) — your literal first session, narrated step-by-step (install → first scan → read a finding → wire your editor), every command run for real
- [`docs/tutorial.md`](docs/tutorial.md) — your first 15 minutes with antigen, end-to-end
- [`docs/where-to-look-for-antigens.md`](docs/where-to-look-for-antigens.md) — conventions for locating antigen declarations in your project
- [`docs/usage-patterns.md`](docs/usage-patterns.md) — common patterns for applying antigen's vocabulary to real failure-classes

**If you want reference docs:**
- [`docs/macros.md`](docs/macros.md) — full reference for all five attribute macros
- [`docs/witness-tiers.md`](docs/witness-tiers.md) — WitnessTier gradient reference
- [`docs/fingerprint-grammar.md`](docs/fingerprint-grammar.md) — full fingerprint DSL reference
- [`docs/glossary.md`](docs/glossary.md) — vocabulary anchor for every project term
- [`CHANGELOG.md`](CHANGELOG.md) — what shipped in this release

**If you want the architecture:**
- [`docs/origin.md`](docs/origin.md) — the post-mortem narrative that motivated the project
- [`docs/vision-pitch.md`](docs/vision-pitch.md) — the full vision; supersedes all narrower framings
- [`docs/structural-memory.md`](docs/structural-memory.md) — foundational whitepaper on what structural memory means and why it matters for hybrid human-AI teams
- [`docs/decisions.md`](docs/decisions.md) — ratified ADRs and amendments
- [`docs/internal/postures.md`](docs/internal/postures.md) — architectural postures threaded through the ADRs

**If you're a researcher or practitioner:**
- [`docs/internal/cross-domain-architectural-map.md`](docs/internal/cross-domain-architectural-map.md) — the cross-domain architectural map (16+ academic fields)
- [`docs/internal/immune-system-primitive-map.md`](docs/internal/immune-system-primitive-map.md) — the multi-component immunity framing, primitive by primitive

---

## Contributing

Most valuable contributions right now:

- **Real-world failure-class proposals** — Rust failures that fit (or refine) the project's growing taxonomy; issue templates at [`.github/ISSUE_TEMPLATE`](.github/ISSUE_TEMPLATE)
- **Antigen-stdlib candidates** — specific patterns to bundle in the eventual stdlib library with real-world instance evidence (not speculation)
- **Witness type integrations** — kani/prusti/verus/creusot/flux harness recognition refinements
- **Adoption feedback** — real-world adoption signal from Rust workspaces

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for detail.

---

## Status

- crates.io: [`antigen`](https://crates.io/crates/antigen), [`cargo-antigen`](https://crates.io/crates/cargo-antigen), [`antigen-macros`](https://crates.io/crates/antigen-macros), [`antigen-fingerprint`](https://crates.io/crates/antigen-fingerprint), [`antigen-attestation`](https://crates.io/crates/antigen-attestation) — see crates.io for the published version
- Repository: [github.com/antigen-rs/antigen](https://github.com/antigen-rs/antigen)
- CI: cargo check + test + fmt + clippy (-D warnings) + doc (-D warnings) on every push and PR

---

## Acknowledgments

The originating insight came from a cleanup expedition on the [tambear](https://github.com/tambear-rs/tambear) project. The immune-system architecture framing came from the project lead. The naming, three-verb framing, taxonomy, and design substrate emerged in pre-team scaffolding, then were ratified across a series of design sweeps that produced the ADRs, empirical validations, and the comprehensive vision.

See [`docs/origin.md`](docs/origin.md) for the founding incident; [`docs/vision-pitch.md`](docs/vision-pitch.md) for the full vision.

---

## License

Dual-licensed under MIT or Apache-2.0. See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

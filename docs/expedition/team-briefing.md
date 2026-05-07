# Team Briefing — antigen project

> The single source of truth for project-specific context the team needs at spawn time.
> Agent identity lives in `~/.claude/agents/<role>.md`; this briefing holds project context.

## The mission

Build the **antigen** ecosystem — a structural memory system for failure-classes in Rust.
Make implicit immunity explicit. Compose existing tools (lints, tests, deprecations, formal
verification) under a coherent vocabulary with shared primitives.

This is a contribution to the Rust ecosystem, not an internal tool. Quality bar: "would
the rust-lang ecosystem accept this if matured?"

## Read first

1. `docs/origin.md` — the post-mortem narrative motivating the project. Read this first; everything else makes more sense after.
2. `docs/expedition/design-intent.md` — what antigen IS, what it ISN'T, why now, the 8-class failure taxonomy, biological→Rust mapping.
3. `docs/expedition/api-shape.md` — sketch of macros, cargo subcommands, witness types, composition rules. **This is a sketch, not a spec — you refine, contradict, and ratify.**
4. `docs/expedition/revolutionary-and-not.md` — honest assessment of what's genuinely new vs. existing-tools-recomposed. Adoption pathway, what could kill it, what it doesn't replace.
5. `docs/decisions.md` — 8 foundational ADRs (ADR-001 through ADR-008). These are pre-ratified by team-lead in pre-team scaffolding.
6. `docs/process.md` — the formal ADR lifecycle and governance process. **You operate within this process.** When drafting new ADRs, follow the lifecycle described there.
7. `docs/expedition/inheritance-from-tambear.md` — what disciplines come pre-loaded from tambear vs what antigen invents fresh. Includes the future-reciprocity section on tambear adopting antigen.
8. `docs/expedition/failure-class-instances.md` — 36 real-world Rust ecosystem instances of the 8 failure classes (research subagent output).
9. `docs/expedition/ecosystem-composition.md` — 20+ Rust tools mapped against the 8 classes (research subagent output).
10. `docs/expedition/academic-context.md` — relationship to refinement types, design-by-contract, named-effect type systems (research subagent output).
11. `docs/glossary.md` — vocabulary anchor; reference whenever terminology feels ambiguous.
12. `README.md` — public-facing project framing.

The design docs are *captured shape* documents from pre-team conversation. They are not
ratified. The team's first job is to deconstruct them, contradict where appropriate, and
arrive at a real v1 design through Phase 1-8 discipline (or whatever discipline the team
establishes).

## Standing constraints

1. **Rust ecosystem quality bar.** This project must be acceptable to the broader Rust
   community eventually. Idiomatic Rust, well-documented, ergonomic, performant.

2. **The biological metaphor is load-bearing, not decorative.** When the metaphor predicts
   a primitive (B-cell memory → persistent failure-class declarations), build it. When the
   metaphor breaks, name where and refine; do not abandon. The metaphor is a thinking
   tool that has produced real architectural insights — preserve it.

3. **Compose, don't compete.** Antigen does not replace tests, lints, deprecations, or
   formal verification. Witness types DELEGATE to existing tools. If a new feature
   duplicates clippy's work, reconsider — what's the COMPOSITION pattern instead?

4. **Ergonomics matter.** If declaring an antigen takes more than 60 seconds for a known
   failure-class, the project has failed adoption. The macros must scaffold aggressively;
   defaults must be sensible; CLI must be helpful.

5. **Honest scope.** Antigen catches *named* failure-classes; it does not detect novel
   logic errors. State this plainly in docs and error messages.

6. **No tech debt.** See a bug, fix it in session. The cost of "fix it later" compounds.
   This applies to the design docs too — if a section is wrong, edit it.

7. **Anti-YAGNI / YAWNI.** If the structural primitives guarantee we'll need a feature,
   build it now. The 8-class failure taxonomy is the structural commitment; build for all
   8, not just the easy ones.

8. **Substrate over memory.** Verify before claiming. Run `cargo check`, `cargo test`,
   read the actual files. Past-team's design docs are starting context, not authority.

9. **Test counts and CI green at every commit.** Same discipline as tambear. Pre-impl
   property tests for each macro; cargo-antigen scan must pass on its own codebase.

10. **The 8-class failure taxonomy is the structural anchor.** Every named antigen the
    project ships maps to one or more of the 8 classes. The classes themselves may be
    refined or split during the expedition, but they're the load-bearing taxonomy.

## Expected team composition

Default JBD tambear team (9 agents) adapted for antigen:

- **pathmaker** — builds the macros, the cargo extension scaffolding, the witness types.
- **navigator** — coordinates routing, owns campsite logbook, escalates with stories from the trail.
- **scout** — surveys existing Rust ecosystem (cargo-mutants, cargo-careful, kani, prusti, clippy internals, RFC history). What's already done? What's available to compose?
- **naturalist** — keeps the biological metaphor honest. When a primitive predicted by biology doesn't fit Rust naturally, names where and offers refinement. Cross-domain inspiration.
- **observer** — lab notebook, neutral record. Audits design docs, tracks what changes from session to session.
- **math-researcher** — *acts as systems-researcher* for this project. Reads RFCs, papers on lints/macros/ecosystem patterns. The "first-principles via prior art" role.
- **adversarial** — designs the failure-class taxonomy at full strength. Stress-tests the antigen library by imagining the worst-case adoption scenarios. Writes the failing-as-passing tests.
- **scientist** — publication-grade write-up. Drafts the paper that might eventually go to PLDI / OOPSLA / ECOOP.
- **aristotle** — first-principles deconstruction of "what is immunity, structurally?" Phase 1-8 on every load-bearing API decision.

## Coordination pattern

Same JBD as tambear. Navigator routes; team-lead is thinking-partner, not micromanager.
Idle is an invitation. Stories from the trail > status updates. Convergence checks at
garden entry/exit.

Campsites are the shared substrate — `~/.claude/skills/campsite/campsite` from project
root.

## Suggested early flow (not a contract)

1. **Scout maps prior art** — which Rust ecosystem tools cover pieces of the antigen surface? (cargo-mutants, cargo-careful, clippy, kani, prusti, creusot, verus, deprecation, cargo-fix). What's their composition pattern? What's missing?

2. **Aristotle Phase 1-8 on the design intent** — deconstruct `design-intent.md`. Which assumptions are load-bearing? Which are speculative? Which need to be elevated to ratified DECs (yes, antigen-the-project should have its own DEC discipline)?

3. **Math-researcher / systems-researcher reads** — RFCs for proc-macros, cargo extensions, custom diagnostics. How does similar tooling get adopted in Rust? What's the engineering cost vs. ergonomics tradeoff?

4. **Adversarial drafts the failure-class taxonomy at full strength** — for each of the 8 classes, name 3-5 concrete instances from real-world Rust codebases. Test the taxonomy against substrate.

5. **Naturalist roams** — biological metaphor kept honest. Where does it predict a primitive that fits Rust naturally? Where does it break and need refinement?

6. **Pathmaker drafts macros** — `#[antigen]`, `#[presents]`, `#[immune]`, `#[descended_from]`. Aristotle code-reviews. Adversarial attacks. Scientist documents.

7. **Cargo extension scaffolding** — `cargo-antigen scan` first (read-only, lowest risk). Then `new`, `vaccinate`, `audit`.

8. **Stdlib antigens crate** — populate the 8 failure classes with concrete instances. This is the adoption flywheel.

The team chooses its own pacing. JBD energy: explore freely; commit when work feels whole.

## What this expedition is NOT

- A research project. We're shipping working tools, not just papers. Papers may emerge as
  side-effect of the work, but the goal is adoption.
- A Rust language extension. We use existing macro/cargo capabilities; we do not need
  RFC processes through rust-lang.
- A formal verification tool. Antigen DELEGATES to formal verification when needed; it is
  not a verification framework.
- A documentation system. Documentation is itself vulnerable to drift; antigen lives in
  the type system.
- A clippy replacement. Antigen COMPOSES with clippy via witness delegation.

## Sweep boundaries

The expedition naturally has multiple sweeps. Suggested rough phasing:

- **Sweep A1**: ratify design via Phase 1-8; produce the first ADR-equivalent (perhaps named "AntigenADR-001 — failure-class memory architecture").
- **Sweep A2**: implement core macros (`#[antigen]`, `#[presents]`, `#[immune]`).
- **Sweep A3**: implement `cargo-antigen scan` and cross-crate antigen discovery.
- **Sweep A4**: implement `#[descended_from]` propagation + composition rules.
- **Sweep A5**: implement vaccinate + audit + stdlib antigens (10-20 to start).
- **Sweep A6**: ergonomics polish + IDE integration scaffolding + rust-analyzer plugin design.

These are suggested; the team adjusts as work clarifies.

## Garden, museum, library

Same access as tambear:
- `~/.claude/garden/` is each agent's private creative space.
- `~/.tessera/museum/` is curated for sharing with team-lead.
- `~/.claude/garden/books-you-can-read-and-journal-about/` is the reading library.

Reading-notebook entries on relevant prior art especially welcome from scout and
math-researcher.

## Reference

- Project README: `README.md`
- Design intent: `docs/expedition/design-intent.md`
- API shape: `docs/expedition/api-shape.md`
- Revolutionary-and-not: `docs/expedition/revolutionary-and-not.md`
- Workspace Cargo.toml: `Cargo.toml`
- Crates.io reservations: [`antigen`](https://crates.io/crates/antigen) (lib), [`cargo-antigen`](https://crates.io/crates/cargo-antigen) (bin) — version 0.0.1 placeholders signal intent.
- GitHub: `https://github.com/antigen-rs/antigen`

## Closing thought from team-lead

The reason this project exists: in May 2026, during a tambear cleanup expedition, adversarial
gardened a meta-observation that "corrected designs don't carry the failure that motivated
them." Tekgy connected this to immune-system architecture and proposed building structural
failure-class memory as a Rust ecosystem contribution. The 8-class failure taxonomy emerged
from first-principles thinking through council perspectives.

You're picking up that thread. The substrate above (`design-intent.md`, `api-shape.md`,
`revolutionary-and-not.md`) is where the conversation got to before handing off to a
focused team. Treat it as starting context, not authority. Refine, contradict, ratify.

Welcome to the antigen expedition.

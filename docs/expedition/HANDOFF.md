# Antigen — Hand-off

> Where the antigen project stands at the moment of hand-off from
> winrapids-cwd-scaffolding to fresh-cwd-team-launch.

## What's done

### Workspace structure (verified `cargo check` clean)

```
R:\antigen\
├── Cargo.toml                          ← workspace manifest
├── README.md                           ← public-facing project framing
├── LICENSE-MIT                         ← dual-license
├── LICENSE-APACHE                      ← dual-license
├── .gitignore                          ← campsites/, target/, Cargo.lock excluded
├── antigen/                            ← lib crate
│   ├── Cargo.toml
│   └── src/lib.rs                      ← module-doc explaining design phase
├── cargo-antigen/                      ← bin crate
│   ├── Cargo.toml
│   └── src/main.rs                     ← prints design-phase message + reserved subcommands
├── docs/expedition/
│   ├── HANDOFF.md                      ← this file
│   ├── design-intent.md                ← what IS, what ISN'T, why now, 8-class taxonomy
│   ├── api-shape.md                    ← macros, cargo subcommands, witness types
│   ├── revolutionary-and-not.md        ← honest claims, adoption pathway
│   └── team-briefing.md                ← for the JBD team at spawn time
└── campsites/                          ← gitignored
    └── antigen-design/                 ← 9 starter campsites, one per role
```

### Design substrate

Four design documents totaling ~25k words in `docs/expedition/`:
- **design-intent.md** — what antigen IS, what it ISN'T, why now (post-COVID vocabulary, AI-coding-era memory loss, mature Rust ecosystem), 8-class first-principles failure taxonomy, biological→Rust constructs mapping
- **api-shape.md** — three-verb framing (build/give/find), macro primitives, cargo subcommands, witness types, composition rules
- **revolutionary-and-not.md** — honest assessment of what's genuinely new vs. existing-tools-recomposed, adoption pathway, what could kill it, what it doesn't replace
- **team-briefing.md** — single source of truth for project context the JBD team reads at spawn time

### Campsites seeded (9)

All under `campsites/antigen-design/` (gitignored, on-disk only):
- `coordination` (navigator)
- `prior-art-scan` (scout)
- `api-design` (pathmaker)
- `biological-metaphor` (naturalist)
- `lab-notebook` (observer)
- `failure-taxonomy` (adversarial)
- `first-principles` (aristotle)
- `manuscript` (scientist)
- `systems-research` (math-researcher in systems-researcher mode)
- `naturalist-roam` (naturalist's wandering thread)

### Git state

- `git init` done; 13 files staged
- **No initial commit yet** — left for you to make with your preferred message
- Suggested commit message:
  ```
  Initial scaffolding: workspace + design substrate + campsites

  Reserved namespace for antigen + cargo-antigen on crates.io.
  Pre-team design substrate captures the 8-class failure taxonomy,
  three-verb API framing, biological→Rust mapping, and honest
  revolutionary-and-not assessment.

  See docs/expedition/team-briefing.md for the JBD team's spawn-time
  context. The expedition launches in a fresh Claude Code session
  from R:\antigen\.
  ```

## What you need to do

### Phase 1 — Reservation (do these soon)

1. **Create GitHub org**: `github.com/antigen-rs` (Free plan)
2. **Create the repo**: `github.com/antigen-rs/antigen`
3. **Push the scaffolding**:
   ```
   cd R:\antigen
   git commit -m "Initial scaffolding..."     # see suggested message above
   git remote add origin https://github.com/antigen-rs/antigen.git
   git branch -M main
   git push -u origin main
   ```
4. **crates.io account**: log in at https://crates.io/me with your GitHub account
5. **API token**: create one at https://crates.io/settings/tokens (scope: publish-new + publish-update)
6. **Login locally**:
   ```
   cargo login <your-token>
   ```
7. **Publish placeholder crates**:
   ```
   cd R:\antigen
   cargo publish -p antigen
   cargo publish -p cargo-antigen
   ```
   Each is `0.0.1`. Both names get reserved.

### Phase 2 — Defer (decide later)

- `.rs` domain ($70). Not urgent. GitHub URL is the canonical project home for now.
- LinkedIn / Twitter / Mastodon presence for the project. Defer until there's something to announce.
- README polishing. Current version is fine for reservation; team can refine when work is real.

### Phase 3 — Launch the JBD team (when ready)

In a fresh Claude Code session opened with `cd R:\antigen`:

1. Tell me: "Launch the JBD tambear team for antigen using the team-briefing in `docs/expedition/team-briefing.md`."
2. I'll read the briefing, verify substrate, spawn the 9 agents in parallel.
3. Suggested team name: `antigen-design-2026-05-07` (or whatever date the launch happens).
4. The team self-orients from the briefing + claims their already-seeded campsites.

The agents will go through: scout maps prior art → aristotle Phase 1-8 on design intent →
math-researcher reads RFCs → adversarial drafts failure-class taxonomy at full strength →
naturalist keeps the metaphor honest → pathmaker drafts macros → scientist documents.

Expedition pacing: probably 2-3 sessions to get to a v1 design ratification (an antigen-
project DEC), then a sweep-by-sweep implementation expedition.

## Standing decisions captured

- **Naming**: `antigen` (not `anamnesis`). Adoption-friendly, three-verb shape, post-COVID familiar.
- **GitHub org**: `antigen-rs` (Rust ecosystem convention).
- **License**: Dual MIT + Apache-2.0 (Rust ecosystem standard).
- **Workspace**: two-crate (lib + bin); `antigen-stdlib` and other extension crates added as members later.
- **Campsites**: gitignored on-disk substrate (matches tambear convention).
- **Biology metaphor**: load-bearing, not decorative. Preserve through expedition.
- **Compose, don't compete**: antigen DELEGATES to existing tools (clippy, kani, prusti, proptest); does NOT replace them.

## Open questions captured for the team

In `design-intent.md` § "Open questions for the JBD team":
1. proc-macro hygiene interaction
2. Antigen hierarchy semantics
3. `cargo antigen scan` default severity
4. Anti-squatting strategy on crates.io
5. Research-paper opportunity assessment
6. Relationship to formal-verification frameworks
7. Naming consistency (antigen vs antibody for verb-form)

In `api-shape.md` § "What this API shape DOESN'T address":
- Cross-crate antigen versioning
- Anti-pattern: `#[immune]` without real witness
- Performance / scan strategy
- IDE integration mechanics
- Privacy of antigen declarations

These are *for the team*, not pre-answered.

## What I (team-lead) take into the next session

- The substrate is ready
- The team-briefing is in place
- The reservation steps are clear
- Phase-2 decisions deferred appropriately
- The JBD team launch is one fresh-session command away

When you're ready, open Claude Code at `R:\antigen` and we go.

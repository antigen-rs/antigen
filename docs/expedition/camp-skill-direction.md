# Camp Skill Direction — Antigen-on-Antigen Adoption

> **Status**: design captured 2026-05-20 after v0.1-rc Phase 4 shipped.
> Implementation deferred to post-v0.1-rc-tag window. Companion to
> `~/.claude/skills/camp/SKILL.md` (the skill's own design + usage spec).
> This doc lives in the antigen project because camp's development IS the
> canonical antigen-on-antigen adoption example — the substrate for the
> "what does using antigen actually look like" story external adopters will
> ask for.

> **Authorship**: Tekgy + Claude design conversation, 2026-05-20, immediately
> after antigen v0.1-rc Phase 4 (tambear sinh/cosh signed-zero discipline)
> completed. Decisions captured here so the post-v0.1-rc-tag work has clear
> direction without re-derivation.

---

## Origin

The campsite skill (`~/.claude/skills/campsite/`) worked well under previous
harness and model versions. Long-running JBD teams that worked for days made
durable status DB queries valuable. Under Opus 4.7 + current harness, the
team shape changed:

- Teams wake fresh each day (no in-memory continuity across sessions)
- Agents naturally substrate-check (`ls`, `git log`, read files) for current
  state rather than querying a separate status layer
- Multi-session pathmaker parallelism means outbox ≠ inbox ≠ disk; the DB
  drifts as the source-of-truth check
- Campsite CLI invocation cost discouraged status updates mid-work; agents
  reach for SendMessage + substrate-check instead

Symptoms observed during 2026-05-20 session:
- Pathmaker reporting "already shipped this, sixth duplicate routing this session"
- Navigator hot-spinning empty idle notifications
- "Task already shipped, substrate-check confirms" routing-loops
- The campsite DB was the SQL of record but the work was passing around it

Rather than patch the friction, Tekgy proposed: **rebuild as antigen's first
internal user.** The substrate antigen audits IS the substrate agents already
check. Camp-on-antigen aligns coordination with how work actually flows.

---

## The decision

Build a new skill at `~/.claude/skills/camp/` — intentionally fresh starting
point, not "campsite-v2." Per-project Rust crate. Campsites as modules in
shared `camp` crate. `cargo check` IS the team-status query. Each campsite
declared as Oracle artifact-class per ADR-021. Required signers per campsite
declared by navigator (whose role expands to include camp stewardship).

Old campsite skill stays in service for in-flight team work as the previous
team wraps up. New team launches into camp. Clean break; no migration.

---

## Why this is canonical-adoption-example for antigen

Three reasons camp is the right canonical adopter:

### 1. Dogfood proves the primitives

Antigen v0.1-rc shipped three primitives (substrate-witness predicate family,
cross-cutting attestation, Oracle artifact-class with state machine). If we
can use antigen to coordinate our own JBD teams, that's the strongest possible
adoption story for v0.1-rc external users: "we built it because we needed it,
and we use it for everything we do."

If we CAN'T — if camp encounters friction antigen can't resolve — that's
diagnostic substrate for antigen's v0.2 evolution.

### 2. Real-use friction tells us what's missing

Synthetic examples can't generate the kind of "I tried to use this and the
primitive bent" findings that real use generates. Camp's development becomes
the friction-capture mechanism for antigen's continued evolution. Each
"hmm, this is awkward" moment during camp build is either resolved by antigen
primitives (good — antigen's working) or surfaces what antigen needs next
(good — v0.2 inputs).

### 3. The recursion is productive

Tool-as-lens-for-building-next-tool is generative when each level surfaces what's
missing at the next level. We built antigen to formalize a discipline we
wanted for ourselves. Now we're using antigen as the discipline that organizes
how we build the next thing (camp). The recursion isn't decorative — it's
the substrate-truth check at the meta level.

---

## What this contributes to antigen's adoption story

When external adopters land on antigen and ask "what does this look like in
real use?", camp is the answer:

- **A real team coordination system, not a toy example**
- **Multi-role discipline-attestation**: navigator + pathmaker + observer +
  others must sign before campsite is "done"
- **Three signature tiers in use**: text-stamp for LLM agents, git-trust for
  humans with git config, crypto-signed reserved
- **Oracle artifact-class lifecycle**: campsites move DRAFT → ACTIVE → DONE
  (and BLOCKED, REOPENED) via steward-authorized transitions
- **Cross-cutting attestation**: campsites can carry `attested` on their
  declarations as well as `requires` predicates on completion
- **`cargo check` as status query**: the compiler is the team's
  coordination-substrate validator
- **Substrate-currency at scale**: fresh team wakes with `cargo check` and
  has full team state from second one

This is what we want adopters to see. Not synthetic docs examples — a real
system we use ourselves, every day, for our own work.

---

## Architectural decisions LOCKED

(Mirrors `~/.claude/skills/camp/SKILL.md` §"Design decisions LOCKED")

- **Per-project scope** — one `camp/` per project; not per-team-instance.
  Teams are ephemeral; project substrate is durable.
- **Modules in shared crate** — campsites as modules in shared `camp` crate.
  `cargo check` on one crate = full team status. Import order can express
  journey-order.
- **`cargo check` = status query** — substrate IS the status. No SQL DB, no
  status-logging ceremony.
- **Oracle promotion up-front** — heavier schema than alternatives but
  clearer semantics. Lifecycle states, stewardship, provenance all benefit
  from explicit representation.
- **Navigator role expanded** — primary driver of camp tooling for the
  team. Per-campsite required-signer + done-definition decisions are
  navigator's lane. When main-thread Claude (or Tekgy) creates campsites,
  navigator inherits stewardship and may edit.
- **Antigen v0.1-rc surface is structurally sufficient** — no new antigen
  primitives needed for camp; use what's shipped.
- **Fresh team wakes to substrate via `cargo check`** — one command, full
  state. No ceremony.

## Decisions DEFERRED to build-time

(See `~/.claude/skills/camp/SKILL.md` §"Open design questions" for details.)

- Workspace member vs standalone Cargo.toml
- Single `CampsiteOpen` antigen vs multiple state-specific antigens
- Oracle-per-campsite granularity
- Reopening semantics (remove-signature vs formal state transition)
- Per-role substrate folder convention (fixed set vs minimal default)
- General-purpose state-machine primitive for antigen v0.2+ (camp may
  inspire; not blocker)

## Decisions REJECTED with reasoning

- **Refactor old campsite**: rejected. The old campsite was right for its
  era; replacing the implementation underneath active work disturbs more
  than it improves. Clean break with new name in new place honors past
  work without trapping current work.
- **Per-team-instance camp**: rejected. Teams are ephemeral under Opus 4.7
  harness; per-team would lose substrate between sessions. Per-project
  preserves the journey across team-wake-ups.
- **Crate-per-campsite**: rejected as heavyweight. Modules in shared crate
  give one `cargo check` for full team status.
- **SQLite status DB**: rejected. Substrate-check pattern agents naturally
  use IS the coordination check; SQL adds a layer that drifts.

---

## Build sequence (proposed)

Post-v0.1-rc-tag window:

1. Resolve workspace integration decision (workspace member or standalone)
2. Sketch `camp/src/antigens.rs` — declare CampsiteOpen + CampsiteBlocked + roles
3. Sketch one campsite module (Phase5Release or similar) — minimal end-to-end
4. Implement `camp` CLI as wrapper around `cargo antigen`
5. Test end-to-end: `camp new <slug>`, `camp sign --as <role>`, `camp status`
6. Document discipline in `~/.claude/skills/camp/SKILL.md`
7. Capture friction as we go; reframe through antigen lens
8. Update `~/.claude/skills/jbd/SKILL.md` + `docs/expedition/team-briefing.md`
   to point teams at `/camp` not `/campsite`
9. Old campsite skill remains in service for in-flight work; new team launches
   into camp

---

## What this means for antigen's roadmap

- **v0.1-rc**: ships with three ratified primitives. Camp uses these as-is.
- **v0.2+ inputs from camp development**: any friction camp surfaces feeds
  back into antigen's v0.2 design. Specifically watch for: general-purpose
  state-machine primitive (campsite states don't perfectly match Oracle
  vocabulary), per-role-substrate-folder conventions (might want antigen
  primitives for "this folder is the stewardship substrate for this role
  for this artifact"), CI integration patterns (cargo check in CI = team
  status enforcement at PR-merge time)
- **Tambear continues as first external adopter** (sinh/cosh signed-zero
  discipline shipped Phase 4). Camp is first internal adopter. Two
  adoption substrates feeding antigen evolution from two angles.

---

## When camp ships

Likely impact on antigen project:
- This doc updates with build-time decisions resolved
- Camp's friction log becomes input to antigen v0.2+ amendments
- The "antigen-applied-to-antigen" expedition doc gets a new instance:
  team-coordination-via-discipline-attestation as the canonical internal
  adoption case
- README + vision-pitch update to name camp as canonical adopter when ready

---

## Related expedition substrate

- `~/.claude/skills/camp/SKILL.md` — the skill's own design + usage spec
  (contains design rationale, architecture sketch, CLI sketch, discipline
  guidance, build sequence)
- `~/.claude/garden/2026-05-20-camp-skill-and-antigen-eating-itself.md` —
  Claude's first-person reflection on the design conversation (private to
  Claude's garden; referenced here for the record of why this happened)
- `docs/expedition/antigen-applied-to-antigen.md` — existing expedition doc
  cataloging antigen's self-application; camp becomes a major new instance
  once shipped
- `docs/expedition/inheritance-from-tambear.md` — the future-reciprocity
  discipline that rhymes with camp's adoption story
- `docs/process.md` — ADR lifecycle that the substrate-grep sub-routine
  (committed during this session) was added to per F28-R2

---

*Camp development resumes after v0.1-rc tag ships. This doc serves as the
substrate the next session picks up from. The substrate is the trail
marker; the trail is journey-shaped.*

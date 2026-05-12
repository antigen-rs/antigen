# Sweep A3.5 — Onboarding Closure

> Closure narrative for Sweep A3.5. Authored by navigator (convergence-check
> synthesis) + team-lead (final ratification 2026-05-12).
>
> Sweep dates: 2026-05-11 (launch) through 2026-05-12 (ratification commit).

---

## What sweep A3.5 produced

Sweep A3.5 was named "onboarding pre-release sweep." Tekgy's directive:
**best-in-class, not merely sufficient.** The architecture was good; the tooling
worked (235 tests at A3 closure); the substrate was rich. This sweep produced
the welcoming first encounter of all of it.

**15 deliverables shipped across 5 phases:**

1. README revision — working quickstart, real output inline, multi-component-immunity framing
2. Crate-level doc-comments reviewed + improved across all four crates
3. Tutorial (`docs/tutorial.md`) — narrative walkthrough; cold-read verified by pathmaker
4. `docs/fingerprint-grammar.md` — six operators + composition semantics
5. `docs/witness-tiers.md` — four-tier set with code examples + expected audit output
6. `docs/output-formats.md` — scan/audit JSON schema; every field enumerated
7. `docs/macros.md` — full reference for all five macros
8. Examples directory — 5 examples compile and run; cross-reactivity documented
9. CLI hide — `new`/`vaccinate` hidden from `--help` (ce75896)
10. `docs/roadmap.md` — v0.2.0 + A4-A6 with substrate-grounded confidence intervals
11. Cargo.toml metadata audit — descriptions, keywords, categories for crates.io
12. CHANGELOG.md accuracy verified against substrate
13. Code verification pass — all user-facing docs checked against current code state
14. JSON output schema-lock test (`antigen/tests/atk_schema_lock.rs`) — 8 assertions; CI-enforced
15. Package footprint — `antigen` published package trimmed to 13 files (src + examples + manifests)

Plus: all four crates reserved on crates.io at v0.0.1; pre-existing rustfmt drift
caught and fixed during convergence-check synthesis.

---

## The headline finding: reference-frame-drift and its structural fix

The Phase 5 convergence-check surfaced a systemic finding with real consequences.

**The ExternalUnvalidated phantom tier** appeared in 16+ locations across
user-facing docs (`witness-tiers.md`, `output-formats.md`, `macros.md`,
`roadmap.md`, `README.md`, `usage-patterns.md`). It does not exist in any Rust
source file — zero occurrences. Docs had been authored against design-substrate
(expedition docs describing *intended* behavior) rather than ratified-code-substrate
(actual serde-serialized output). The result: well-structured docs with wrong
field names, wrong enum variant counts, wrong tier names — invisible to
spec-level review, only visible when cross-checking against binary output.

**Root cause named**: reference-frame-drift. Two reference frames that can diverge
silently: design-doc (intent) vs ratified-code (behavior). The mismatch produces
no error at authoring time; only a consumer encountering the real JSON output
would discover it.

**Structural fix**: `antigen/tests/atk_schema_lock.rs` — integration test
invoking the actual binary, parsing JSON output, asserting closed sets of field
names and enum variant serializations. Makes reference-frame-drift a CI failure
rather than a doc-review question. This is the same principle antigen applies at
the code tier — structural memory of what's correct, enforced automatically —
applied to the project's own documentation substrate.

This is the second instance of the "spec-invisible silent failure" encounter
candidate (deferred-substrate.md watches for a third instance before V0+1
promotion). The first was tokenization-asymmetry during A3. Both instances share
the reference-frame-mismatch root cause: authoring substrate and runtime substrate
diverging silently.

---

## What convergence-check methodology caught that independent review missed

Phase 5 used convergence-check methodology per scope-lock amendment #1:
cross-checking adversarial + aristotle outputs rather than treating them as
independent reads.

**Adversarial** caught ExternalUnvalidated via JSON schema mismatch — running
the binary and comparing output structure against doc claims. Entry point: field
names in JSON don't match docs.

**Aristotle** caught ExternalUnvalidated via enum grep — reading the source and
finding zero occurrences of the variant. Entry point: source doesn't match docs.

Two independent roles, different entry points, same root cause. The convergence
confirmed the finding was systemic (docs uniformly wrong) rather than local
(one doc section wrong). That distinction shaped the fix: mass-correction pass
across all user-facing docs + structural schema-lock test.

Aristotle also filed two false positives (temporal claims not substrate-grepped
before routing). Navigator caught both before they reached pathmaker. The
asymmetry is itself a calibration finding: structural claims get automatic
substrate-grep discipline; temporal claims ("not yet ratified," "hasn't shipped")
had been getting looser treatment. Aristotle acknowledged the gap; calibration
filed to navigator memory.

---

## What the routing-loop encounter registers as

Team-lead named it directly: pathmaker's escalation (6-7 consecutive routings of
already-shipped deliverables) is a **V8 verifier-self-correction instance at the
routing-coordination tier** — the team's own discipline catching navigator
outbox-state-as-substrate failure.

The failure mode: navigator context replayed a stale brief snapshot between turns
during context compaction, routing deliverable specs to pathmaker for work already
on main. Pathmaker correctly refused to re-do committed work (doing so would be
substrate manipulation), provided raw substrate evidence (git log, file hashes,
cargo test output), and escalated to team-lead after six cycles without
self-correction.

This is structurally the same failure class as A1's Validation 4 (team passing
"ratification complete" signals on agent context rather than substrate) — one
level up in the coordination hierarchy. The catch mechanism was also structurally
the same: a role that substrate-checks before acting catches what context-replay
misses.

The fix during this sweep was the same fix as A1: navigator substrate-checked
(`git log --oneline`) before routing, confirmed the work was already on main,
stopped routing. But the fix arrived on cycle 6-7, not cycle 1. The structural
improvement would be: substrate-check before every "do this work" routing message,
not just when a loop is already established.

Filed to deferred-substrate.md as encounter registration.

---

## Eight scope-lock amendments — what they taught

The A3.5 scope started with 13 deliverables and grew to 15 through 8 amendments.
Each amendment was triggered by substrate contact:

- Amendment 1: convergence-check methodology (aristotle Phase 1-8 finding)
- Amendment 2: dependency-mapping sub-task (navigator — sequential dependencies existed)
- Amendment 3: tutorial cold-read mechanism (aristotle — no team member is "new to antigen")
- Amendment 4: cross-reactivity reframed (adversarial — declaration vs match-fire location)
- Amendment 5: V1 canonicalization timing (team-lead — expedition framing still active)
- Amendment 6: Phase 3 reference docs landed in Phase 4 window (phase-window shift, not scope change)
- Amendment 7: schema-lock test as deliverable 14 (adversarial + aristotle convergence finding)
- Amendment 8: package exclude config as deliverable 15 (cargo package --list substrate check)

**Pattern**: amendments 1-5 arrived during Phase 1; amendments 6-8 arrived during
Phase 5. Phase 1 found structural/process gaps; Phase 5 found substrate-reality
gaps. The substrate-grounded scope-lock discipline worked — amendments registered
real scope changes rather than allowing quiet drift.

---

## Predictions for Sweep A4

1. **Schema-lock test will catch something.** The test was written against current
   behavior. A4 work (body-level fingerprint grammar, composition rules) will
   extend the JSON output schema. When that extension ships, the schema-lock test
   will fail unless updated simultaneously. This is the intended behavior — the
   test is the structural check.

2. **Reference-frame-drift will try to recur.** A4 will extend expedition docs
   with new design-substrate before ratified-code catches up. The discipline:
   expedition docs are design-substrate (clearly labeled), not user-facing-docs
   (which must match ratified-code). Keeping the frames labeled prevents authors
   from carrying design-substrate assumptions into user-facing writing.

3. **The `tests/**` exclusion will need revisiting.** If A4 adds tests that are
   useful to downstream users (e.g., property-testing helpers), the package
   exclude may need refinement. Current state (13 files: src + examples +
   manifests) is correct for the library surface at rc.1; A4 will extend it.

4. **`new`/`vaccinate` will need restoring to `--help` when A5 ships them.**
   Already in deferred-substrate.md. The CLI hide is pre-tag scaffolding; the
   unblock condition is A5 implementation.

5. **The tutorial's cold-read mechanism is imperfect by design** (scope-lock
   amendment #3). A New-Claude-instance cold-read (following tutorial from zero
   session context) would be a stronger verification. If the team launches a
   fresh instance with only the tutorial as context, that's the definitive test.

---

## Substrate documents updated by this sweep

- `README.md` — working quickstart, real output, multi-component-immunity framing, test count 248/31
- `docs/tutorial.md` — new (12.5K)
- `docs/fingerprint-grammar.md` — new (13.7K)
- `docs/witness-tiers.md` — new, then corrected (18.5K); four-tier set; ExternalUnvalidated excised
- `docs/output-formats.md` — new, then corrected (19.9K); all fields enumerated
- `docs/macros.md` — new, then corrected (16.9K); five macros; external-tool witness table correct
- `docs/roadmap.md` — new (12.1K); v0.2.0 + A4-A6
- `docs/usage-patterns.md` — seeded by scout (20.1K)
- `docs/where-to-look-for-antigens.md` — new (scout)
- `docs/troubleshooting.md` — new (adversarial)
- `antigen/tests/atk_schema_lock.rs` — new (281 lines, 8 assertions)
- `antigen/Cargo.toml` — exclude config (13 files in published package)
- `sweeps/A3.5-onboarding/scope-lock.md` — 8 amendments, 15 deliverables
- `docs/expedition/deferred-substrate.md` — V55+; encounter registrations; schema-lock finding

---

## Closure criteria — verified

- [x] All 15 deliverables on substrate (see scope-lock.md)
- [x] All 13 verification criteria substrate-grounded (navigator convergence-check synthesis 2026-05-12)
- [x] Phase 5 convergence-check complete (aristotle + adversarial cross-checked; 15 doc-correctness findings addressed at 1fc89dc)
- [x] Schema-lock test passing: 8/8 (a03f043; `cargo test --test atk_schema_lock`)
- [x] Package footprint clean: 13 files (55ea5c7; `cargo package --list -p antigen`)
- [x] All four crates reserved on crates.io at v0.0.1 (Tekgy, 2026-05-12)
- [x] `cargo test --workspace`: 248 passed, 31 ignored, 22 suites
- [x] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`: clean
- [x] `cargo clippy --workspace --all-targets -- -D warnings`: clean
- [x] `cargo fmt --all -- --check`: clean (pre-existing drift fixed at 7af506e)
- [x] CLI hide verified: `cargo antigen --help` shows only scan/audit/help
- [x] All examples run without unexpected errors
- [x] Routing-loop encounter registered in deferred-substrate.md
- [x] Team-lead ratification received (2026-05-12)

**Substrate is tag-ready. Tag drops when Tekgy says.**

---

*Authored: 2026-05-12 (navigator convergence-check synthesis + sweep-close commit).*
*Team-lead ratification: 2026-05-12.*

# Lab Notebook 001: antigen v0.2 Completion Arc — Observer Baseline Audit

**Date**: 2026-05-24 (UTC)
**Observer**: camp-observer (fresh-context instance)
**Branch**: main
**Last commit at session start**: `8fa6175` — docs(macros): clinical-medicine grounding paragraph on #[diagnostic]
**Status**: Active
**Depends on**: None (first notebook for this expedition)

---

## Context & Motivation

The antigen v0.2 completion arc has 9 campsites in flight. Three require observer signature:

1. `v02-impl-vcs-info-loss` — ADR-026 implementation; observer attests substrate-witness correctness
2. `v02-impl-mucosal-boundary` — ADR-027 implementation; observer attests boundary representation
3. `pre-tag-readiness-v02-alpha-next` — co-watch with scientist; substrate-side readiness; runs at decision moments

This notebook documents the observer's baseline audit pass: verifying the substrate state before any new implementation begins, establishing what IS vs what's claimed, and surfacing any substrate-alignment gaps.

---

## Step 1: Substrate Baseline — What Has Actually Shipped

### Before
**Hypothesis**: The briefing claims 3 families shipped (deferred-defense, supply-chain, convergent-evidence) and 625 tests pass. I expect to verify all three families are in the codebase, git history confirms the implementations, and the working tree is clean.

**Design**: Read `git log`, check `antigen/src/`, `antigen-macros/src/lib.rs`, run `cargo test --workspace`.

### Results
- **625 tests pass, 38 ignored** — confirmed by direct run
- **Working tree state**: clean EXCEPT for untracked `expeditions/v02-completion-arc/` directory (camp's own substrate — not a codebase concern)
- **Current HEAD**: `8fa6175` — this is NEWER than what the briefing (SESSION-HANDOFF-2026-05-22.md) mentioned; pathmaker has already delivered `fixup-diagnostic-doc-comment`
- **Families actually present in `antigen/src/`**:
  - `stdlib/supply_chain.rs` ✓ (ADR-025)
  - `convergent.rs` ✓ (ADR-024 convergent portion)
  - `scan.rs` (includes supply chain audit hints) ✓
  - `audit.rs` ✓
- **Macros present in `antigen-macros/src/lib.rs`**: confirmed via grep — deferred-defense family (`#[anergy]`, `#[immunosuppress]`, `#[poxparty]`, `#[orient]`) present; `#[diagnostic]` present with new doc comment; supply-chain macros present
- **NOT present**: `mucosal`, `MucosalKind`, `vcs_trailer_present`, `VcsAttestation`, `AntigenCategory`, `TriageDecision`, recurrent-emergence macros (`#[itch]`, `#[recurrence_anchor]`, `#[crystallize]`, etc.)

### Discussion
**What I learned**: The substrate is clean and the claims in the briefing are accurate. The 3 shipped families are confirmed in code. The 4 remaining families (vcs-info-loss, mucosal-boundary, antigen-category-metadata, recurrent-emergence) are NOT implemented — campsites are correctly open.

**Surprise**: `fixup-diagnostic-doc-comment` is already DONE — pathmaker moved fast. Camp status confirms complete. The commit `8fa6175` is exactly what pathmaker's camp note claimed (pure doc-comment; no semantic change). Observer verification: PASSED.

---

## Step 2: Camp Substrate State Verification

### Before
**Hypothesis**: The 9 campsites match what the briefing described. No phantom campsites, no missing ones. `prescriptive-gap` may have been added by navigator during session.

**Design**: Run `camp status`, cross-reference with expedition briefing §8 campsites table.

### Results
- **fixup-diagnostic-doc-comment** — COMPLETE (pathmaker signed) ✓ — matches briefing + git commit verified
- **fixup-orient-dual-signature** — OPEN (0/2 signers; waiting pathmaker + aristotle) ✓
- **held-implementation-spec-depth-gap** — OPEN (0/2 signers; waiting aristotle + adversarial) ✓
- **pre-tag-readiness-v02-alpha-next** — OPEN (0/2 signers; waiting observer + scientist) ✓
- **v02-impl-antigen-category-metadata** — OPEN (0/3 signers) ✓
- **v02-impl-mucosal-boundary** — OPEN (0/3 signers) ✓
- **v02-impl-prescriptive-gap** — COMPLETE (no required signers!) — NOT in briefing §8 table — NEW campsite seeded by navigator during expedition setup
- **v02-impl-recurrent-emergence** — OPEN (0/2 signers) ✓
- **v02-impl-vcs-info-loss** — OPEN (0/3 signers) ✓

**Total**: 9 campsites; 7 open, 2 complete

### Discussion
**Substrate-alignment finding**: `v02-impl-prescriptive-gap` is a WATCHING campsite seeded by navigator — not an impl campsite. Navigator noted in camp activity that ADR-024's prescriptive family (9 macros: `#[panel]`, `#[ddx]`, `#[rx]`, etc.) has no impl campsite; this watching campsite names the gap without activating work. Required signers = none, which is why it shows complete (no signer gate). This is correct and intentional per navigator's note.

**Claim verified**: Navigator's campsite-seeding was accurate and the watching pattern is correct. No substrate-alignment drift here — it's new information, not contradiction.

---

## Step 3: ADR-026 (VCS-Info-Loss) Pre-Implementation Audit

### Before
**Hypothesis**: ADR-026 specifies 11 antigens, an extended `#[orient]` with triage fields, git-trailer-based substrate witnesses, and a `cargo antigen vcs` CLI subfamily. None of these exist in code yet. I need to audit the ADR spec for observer-role concerns before implementation begins.

**My role (per briefing)**: Observer attests *substrate-witness correctness* for this campsite.

**Design**: Read ADR-026 §Decision + §Mechanics carefully. Look for: (a) witness type completeness vs substrate-alignment category claim; (b) `#[orient]` extension compatibility with ADR-023; (c) commit-time detection model soundness.

### Results

**Witness-correctness audit of ADR-026 spec**:

The ADR specifies these git-trailer-based substrate witnesses:
1. `vcs_trailer_present(trailer_name)` — checks for presence of a named git trailer
2. `vcs_attest_branch_deletion(branch, by_role)` — attestation for branch deletion
3. `vcs_rollback_triage_chain(commit)` — commit-time; validates triage-commit-before-rollback chain (per adversarial D1)
4. `vcs_server_side_enforcement_active(repo, antigen_name)` — NEW; checks remote configuration

**Observer analysis**:

- (a) **Substrate-alignment category claim**: ADR-026 correctly categorizes most members as `SubstrateAlignment` (per ADR-028 cross-ref). The failure mode is "representation diverges from actual state" — git history claims something happened but the VCS substrate doesn't carry the attestation. The witness design (git-trailer-based) is appropriate for this category. ✓

- (b) **`#[orient]` extension compatibility**: ADR-026 extends `#[orient]` from ADR-023 with triage fields (`triage_decision`, `rollback_target`, `triaged_by`, `rollback_due_within_minutes`). This is the `fixup-orient-dual-signature` concern. ADR-026 depends on `fixup-orient-dual-signature` resolving BEFORE the VCS orient extension fields are added. The camp link confirms this dependency: `v02-impl-vcs-info-loss` depends-on `fixup-orient-dual-signature`.

  **CONCERN**: If pathmaker starts implementing VCS orient extension before aristotle resolves the dual-signature fixup, we deepen the incompatibility. Navigator's orient note explicitly calls this out. This is the key ordering constraint I will monitor.

- (c) **Commit-time detection model**: ADR-026 §Finding notes `RollbackWithoutTriageCommit` cannot be detected by post-hoc history inspection. Pre-commit hook IS the right detection tier. However — pre-commit hooks are bypassable via `--no-verify`. The ADR names this explicitly in §Enforcement-Surface and calls v0.2 "friction-only." This is honest. No overclaim. ✓

- (d) **`vcs_server_side_enforcement_active` witness**: This checks whether the remote has server-side enforcement configured. This is a novel witness design — the substrate being checked is not the LOCAL codebase but a REMOTE GIT SERVER's configuration. This introduces a new witness tier: network-dependent substrate-witness. 

  **CONCERN**: This is architecturally interesting but potentially fragile in CI (network calls during `cargo antigen audit`). Not a blocking concern for v0.2 since server-side enforcement is marked v0.2.1+, but worth noting before implementation. I will deposit this as a camp note.

### Discussion
**Key finding**: The implementation ordering constraint (`fixup-orient-dual-signature` before VCS `#[orient]` extension) is correctly tracked in camp links but is NOT yet resolved. Observer should monitor this dependency before signing.

**Pre-sign requirement for `v02-impl-vcs-info-loss`**: I cannot sign this campsite until: (1) `fixup-orient-dual-signature` is resolved OR (2) pathmaker's implementation correctly defers the `#[orient]` extension to after fixup resolution. Observer sign = attest substrate-witness correctness — I can only attest after implementation exists and can be verified.

---

## Step 4: ADR-027 (Mucosal Boundary) Pre-Implementation Audit

### Before
**Hypothesis**: ADR-027 specifies `#[mucosal]`, `#[mucosal_delegate]`, `MucosalKind` (15 variants), and `cargo antigen mucosal-map`. None in code. Observer role: attests boundary *representation* (the structural representation of boundaries is correct and complete, not overclaimed).

**Design**: Audit ADR-027 for observer-role concerns: (a) boundary representation completeness; (b) `#[mucosal_delegate]` design correctness; (c) biology-grounding honesty.

### Results

**Boundary representation audit**:

- (a) **15-variant `MucosalKind` completeness**: The 15 variants are: `Import, ApiRequest, ApiResponse, McpInvocation, ExternalLink, Iframe, DatabaseQuery, CrossService, SubprocessLaunch, DependencyImport, PrBoundary, UserInput, FilesystemPath, EnvironmentVariable, ShellArgument`. Plus 2 deferred to v0.3+: `WebSocketStream, CiCdPipelineInput`.

  These cover the major software boundary types. The DependencyImport variant cross-references ADR-025 (supply-chain). No obvious gaps in v0.2 scope.

  **Observer concern**: `PrBoundary` is interesting — it's a review-process boundary, not a data-flow boundary. The others are all data-flow types. This is legitimate (PRs bring external code to trusted boundary), but it's structurally different from the others. Not an ADR concern — it's ratified. Worth noting for implementation: the witness type for `PrBoundary` will be different (review-attestation-based, not data-validation-based).

- (b) **`#[mucosal_delegate]` design**: Per ADR-027, the delegate pattern requires:
  - `#[mucosal_delegate(boundary = MucosalKind::..., handled_by = "...", rationale = "...")]` on the outer function
  - The `handled_by` function MUST itself carry `#[mucosal(...)]`
  - Audit emits `mucosal-discipline-delegate-target-missing` if handler doesn't exist
  - Audit emits `mucosal-discipline-delegate-target-not-mucosal` if handler lacks declaration

  **Observer assessment**: This is correct. The delegate audit chain (outer declares delegation → inner must carry corresponding `#[mucosal]`) is verifiable at scan-time because both declarations are in source. The `handled_by` field is a string identifier — which means scan must resolve it against the codebase.

  **CONCERN**: `handled_by = "..."` as a string identifier needs a clear resolution spec. Is it a function path (`crate::module::fn_name`)? A local function name? An arbitrary label? The ADR doesn't specify the resolution algorithm. This is an implementation-spec depth gap — exactly the pattern tracked by `held-implementation-spec-depth-gap`. I will deposit a camp note on `v02-impl-mucosal-boundary` flagging this.

- (c) **Biology-grounding honesty**: ADR-027 is explicit that the 15-variant taxonomy is software-engineering scope-selection, NOT per-variant biology-grounded. Biology grounds the TIER-CLAIM and 4 functional disciplines. Naturalist's NON-NEGOTIABLE finding is incorporated. This is honest. ✓

  Navigator's camp note for this campsite correctly calls out that naturalist should read the biology grounding section before signing. Observer is aligned with this.

### Discussion
**Key finding**: The `handled_by` string resolution spec gap is an implementation-spec depth concern. The ADR says WHAT (delegate must link to a handler) but not HOW (what resolution algorithm scan uses). This may surface as the 4th instance of the WHAT-not-HOW pattern tracked in `held-implementation-spec-depth-gap`.

**Pre-sign requirement for `v02-impl-mucosal-boundary`**: I cannot sign until implementation exists AND the `handled_by` resolution is specified and verifiable. I will deposit a question for pathmaker/aristotle on this.

---

## Step 5: `pre-tag-readiness-v02-alpha-next` Audit Design

### Before
**Hypothesis**: I co-own this campsite with scientist. The readiness check should verify: tests green, CHANGELOG aligned, no unsigned required campsites blocking release, MEDIUM findings resolved or held.

**Design**: This campsite is NOT yet at decision moment — no impl campsite is signed yet. Establish the readiness criteria now so I'm ready to attest quickly when the moment arrives.

### Results

**Readiness criteria for observer attestation**:

1. `cargo test --workspace` → all 625+ tests pass (count must not decrease)
2. `cargo clippy --workspace --all-targets -- -D warnings` → clean
3. `cargo fmt --all -- --check` → clean
4. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` → clean
5. All required impl campsites signed OR explicitly held with documented rationale
6. CHANGELOG.md has entries for each new family shipped this expedition
7. No campsite in `blocked` state
8. `fixup-orient-dual-signature` resolved (or explicitly held with scope-note)

**Current state** (checked against readiness):
- Tests: 625 passing ✓
- Unsigned required impl campsites: 4 (vcs-info-loss, mucosal-boundary, antigen-category-metadata, recurrent-emergence) — NOT release-ready yet
- `fixup-orient-dual-signature`: unresolved (0/2 signers)

### Discussion
**Assessment**: Not at readiness decision moment. Observer's role on this campsite is to watch and attest when the moment arrives. Criteria established above; will re-check when other campsites progress.

---

## Step 6: Activity Log Catch-Up — Post-Baseline

### Before
**Hypothesis**: Other team members have been active since I started the audit. Expect scientist pre-impl criteria on the open campsites, naturalist biology-grounding input on mucosal-boundary, and pathmaker orientation notes.

### Results (from activity log, 12 new entries since my baseline)

**fixup-orient-dual-signature**: Scientist analyzed the actual `OrientArgs` struct in `parse.rs:955` and recommends Option (c) separate macros — `#[orient]` stays as ADR-023 (lightweight orientation/context acknowledgment), new `#[triage_commit]` or `#[vcs_orient]` for ADR-026 rollback-as-triage (decisional capture). Navigator agrees the semantic distinction is sharp: "orient = I am acknowledging context" vs "orient-vcs = I am recording a triage decision." Aristotle hasn't weighed in yet.

**v02-impl-recurrent-emergence**: Pathmaker **deferred** this campsite pending resolution of the HOW-layer gap. Pathmaker explicitly named it as the 4th-instance candidate on `held-implementation-spec-depth-gap`. Pathmaker is moving to ADR-028 (antigen-category-metadata) as the foundation work instead.

**held-implementation-spec-depth-gap**: Pathmaker confirms 4th instance (ADR-024-recurrent). This definitively triggers the process.md amendment workstream decision per navigator's threshold note.

**v02-impl-mucosal-boundary**: Naturalist deposited two notices to aristotle:
- `#[mucosal_tolerant]` gap: active-tolerance vs undefended are structurally distinct but both currently appear as absent-declaration to mucosal-map
- 3 metaphor-predicted primitives (M-cell-equivalent, goblet-cell-layer, IBD-analog) — naturalist treated as v0.3+ research arc

**Scientist pre-impl criteria deposited** on all open campsites — detailed validation checklists for each. This is exactly right — scientist is staking out what "done" means before pathmaker builds it.

**Navigator readiness template**: Includes detailed test-output template and 7-item checklist for per-family landing. Reference point for my readiness criteria.

**Git**: New commit `3bef9cf` — chore(gitignore): navigator committed the `expeditions/` exclusion that pathmaker had stashed.

### Discussion
**Substrate-alignment drift check**: None observed. Team narrative matches camp substrate on all fronts. The pathmaker stash situation (git stash for "not mine to commit") resolved cleanly via navigator owning the commit. No orphaned work.

**Critical path update**: Pathmaker has correctly sequenced: ADR-028 (antigen-category-metadata) FIRST because it cross-cuts all other families; `fixup-orient-dual-signature` BEFORE VCS orient extension; `held-implementation-spec-depth-gap` needs aristotle+adversarial decision on process.md amendment before recurrent-emergence proceeds.

**Naturalist's `#[mucosal_tolerant]` scope question**: This is the live scope question for v0.2. If aristotle rules it v0.2, pathmaker needs it in scope before implementing the boundary family. If v0.3+, observer can attest boundary representation with current 15-variant enum. Awaiting aristotle.

---

## Open Questions

1. **`handled_by` resolution in `#[mucosal_delegate]`**: What is the string resolution algorithm? Fully qualified path? Local name? Needs spec before pathmaker implements. Tagged `implementation-spec-depth-gap`.

2. **`fixup-orient-dual-signature`**: Scientist recommends Option (c) separate macros. Navigator agrees. Aristotle hasn't weighed in yet — needs aristotle's Phase-1-8 ruling on whether "explicit structural acknowledgement" is broad enough to unify both use-cases.

3. **`vcs_server_side_enforcement_active` network witness**: Is a network-dependent witness acceptable in v0.2 `cargo antigen audit`? v0.2.1+ mitigates but needs explicit architectural call.

4. **Prescriptive family**: Navigator named the gap via watching campsite. Tekgy awareness TBD — camp note correctly says "do NOT start implementing without Tekgy's signal."

5. **4th-instance threshold on `held-implementation-spec-depth-gap`**: CONFIRMED by pathmaker + observer. Aristotle + adversarial must decide: crystallize into process.md amendment workstream, or handle case-by-case.

6. **`#[mucosal_tolerant]` v0.2 vs v0.3+**: Naturalist proposed it; aristotle needs to triage. Live scope question for mucosal-boundary campsite.

7. **ADR-028 cross-cutting sequencing**: Pathmaker has correctly identified antigen-category-metadata as the foundation. But observer notes: if pathmaker ships antigen-category-metadata WITHOUT the other families also carrying category fields (because they're not done yet), the "REQUIRED field" enforcement will force backward-compat handling. The v0.1 soft-default-to-FunctionalCorrectness path in scientist's validation criteria is the mitigation. Needs clean implementation.

---

## Artifacts

#### Key substrate reads
| Source | What was verified |
|---|---|
| `git log --oneline -8` | 3 shipped families + diagnostic fix confirmed in history |
| `cargo test --workspace --quiet` | 625 tests pass, 38 ignored |
| `antigen/src/lib.rs` + `antigen-macros/src/lib.rs` | No mucosal/vcs/category primitives present |
| `camp status --root R:/antigen` | 9 campsites; 2 complete, 7 open |
| `camp activity` full log | All events read; prescriptive-gap campsite identified |
| `git show 8fa6175` | fixup-diagnostic commit verified as pure doc-comment, no semantic change |
| ADR-026 §Decision + §Mechanics | Witness design audited; ordering constraint identified |
| ADR-027 §Decision + §Mechanics | `handled_by` resolution gap identified |

---

## Step 7: Mid-Impl Audit — AntigenCategory + VCS Working Tree

### Before
**Hypothesis**: Pathmaker has been working on category.rs and vcs.rs while I was doing baseline audit. Expect partial implementation — types present, enforcement not yet complete.

### Results (working tree state, ~15 min after expedition start)

**New untracked files**:
- `antigen/src/category.rs` — `AntigenCategory` enum, correct design, tests included
- `antigen/src/vcs.rs` — `TriageDecision` (5 variants) + `ServerSideEnforcementMode` (2 variants), with tests
- `research/` — observer's own lab notebook directory

**Modified files**:
- `antigen-macros/src/parse.rs` — `MacroAntigenCategory` enum + `expect_antigen_category()` on `MetaPair`; `AntigenArgs.category` field added; 11 new tests
- `antigen-macros/src/lib.rs` — `#[antigen_category_migration_hint]` added (observer hasn't read this fully yet)
- `antigen/src/lib.rs` — `pub mod category; pub use category::AntigenCategory;` added

**Test count**: 636 (was 625 at baseline — 11 new tests from category implementation)

**Clippy**: PASSES (clean)

**Fmt**: FAILS — 3 line-length issues in `parse.rs` test assertions (assert_eq! with Vec pattern too long for 100-char limit)

**Critical design observation**: `vcs.rs` uses `#[triage_commit]` terminology, NOT an extension of `#[orient]`. This is Option (c) — separate macros. The `fixup-orient-dual-signature` ordering constraint on VCS implementation is effectively dissolved. `#[orient]` stays clean for ADR-023 usage; `#[triage_commit]` is the new VCS-specific macro. This is the right call and matches scientist+navigator's recommendation.

### Discussion
**Pre-commit gate**: Pathmaker must run `cargo fmt --all` before committing. The fmt failures are minor (line-length in tests) but will fail CI.

**Implementation completeness check against scientist's validation criteria**:
- [ ] AntigenCategory enum with SubstrateAlignment+FunctionalCorrectness — **DONE** (category.rs)
- [ ] category field REQUIRED for v0.2+ new declarations (hard parse-time error if absent) — **NEEDS VERIFICATION** (parse.rs validation logic)
- [ ] v0.1 backward-compat soft default + migration hint — **NEEDS VERIFICATION**
- [ ] category-vs-predicate-type cross-check — **LIKELY NOT YET** (requires audit.rs changes)
- [ ] hybrid antigens: Vec<AntigenCategory> both variants required — **parse works** (expect_antigen_category handles array); enforcement NOT yet
- [ ] audit-hint vocabulary (5 hints) — **NOT YET** (no audit.rs changes seen)
- [ ] CLI flags for scan --category — **NOT YET**
- [ ] VCS family: TriageDecision + ServerSideEnforcementMode types — **DONE** (vcs.rs)

Observer assessment: the type foundation is correct. The enforcement mechanics and audit hints will be next steps. Category implementation is NOT complete for signing yet — more work needed.

**Substrate-alignment gap**: vcs.rs module not yet registered in `antigen/src/lib.rs`. Let me verify.

---

## Step 8: Activity Log Catch-Up — Second Pass

### Results (10+ new events since Step 7)

**Category implementation** (scientist note): 643 tests passing (not 636 — scientist counted after fmt fix). cargo fmt is now CLEAN. Full CI gates pass. Foundation is solid.

**Scout's category mapping** (notice b80ba666): exhaustive analysis of all families:
- Supply-chain: 9 SubstrateAlignment + 2 HYBRID (UnsandboxedBuildScript, UnsandboxedProcMacro)
- VCS-info-loss: ALL SubstrateAlignment (ADR-026's "most members" was imprecise — it's ALL)
- Mucosal-boundary: ALL SubstrateAlignment
- Recurrent: mostly SubstrateAlignment per character

**ADR-026 prose amendment needed** (scout notice b076ef98): ADR-026 §Decision says `#[orient] extended` but pathmaker implemented `#[triage_commit]` (separate macro). The ADR text diverges from implementation — needs prose amendment before VCS campsite signs.

**TriageDecision variant definitions missing** (outsider + navigator): variants Black/Red/Yellow/Green/White have no definitions in ADR-026 — only the ::Red example appears. Standard triage (START protocol) is the reference. Variant doc-comments needed in implementation.

**ADR-024 count drift** (outsider + naturalist + e2cb656b): header says 7 immunology-proper but list has 8 (#[titer] is the extra). Naturalist: `#[titer]` belongs in clinical-medicine, not immunology-proper. Correction needed: line 5455 drops titer from immunology-proper; adds to clinical-medicine. Naturalist says tiny fixup campsite is the right process.

**`family=` naming convention gap** (outsider notice f9489243): free-form string with no enforced convention. Supply-chain uses a 47-char compound string; convergent examples use class names. Aristotle and outsider need to work through whether `family` should be a sealed enum or have a documented convention. Observer tags this as a design-coherence concern.

**Supply_chain.rs + convergent.rs retrofit** (navigator notice 8b18013d): pre-ADR-028 files need category fields added. Navigator suggests a dedicated retrofit commit. Observer adds this to sign criteria for antigen-category campsite.

### Discussion
**Substrate-alignment gap count growing**: The observer function is working. In ~30 min:
- 4th WHAT-not-HOW instance confirmed (recurrent)
- ADR-026 prose diverges from implementation (triage_commit vs orient extension)
- ADR-024 §Biology grounding count drift
- TriageDecision variant definitions missing from ADR

Each of these is a claim-vs-substrate divergence I can surface to the team BEFORE they become committed defects.

**Key open question**: ADR-026 amendment — does this need a full ADR amendment (re-ratification) or can it be a prose fix in §Decision? Process.md §Lifecycle says amendments require the same committee. But the change is additive (adding `#[triage_commit]`, not removing anything from `#[orient]`). Aristotle should call this.

---

*Active tracking continues.*

---

## Step 16: Layer-1 Dogfood Seed — ParallelStateTrackersDiverge (live expedition instance)

**Context**: Team-lead flagged a live dogfood seed from this expedition. Observer captured the canonical instance.

### The antigen candidate

**Name**: `parallel-state-trackers-diverge` (working name)

**Family**: `vcs-information-loss` or `substrate-alignment` (top-level; belongs alongside supply-chain as a fundamental substrate-alignment class)

**Category**: `SubstrateAlignment` — the representation (task tracker state) diverges from actual state (camp substrate + git commits).

### The live instance

**What happened (observer witnessed directly)**:

At session resumption, `camp pending --as observer` and `git log --oneline` showed a clean state. The harness TaskList showed task #3 (`v02-impl-vcs-info-loss`) marked **completed**. But:

- Camp substrate: `v02-impl-vcs-info-loss` was `[open]`, `0/3 signers`
- Git history: VCS stdlib commit (b508064) had not yet landed when task was marked complete
- Scientist's pre-tag landscape note explicitly listed `v02-impl-vcs-info-loss` as "STILL OPEN" and blocking pre-tag

Two trackers, same work unit, opposite state. The TaskList said done; every substrate indicator said not done.

**A second divergence (same session)**:

Task #5 (`v02-impl-antigen-category-metadata`) was marked **completed** in the task list. Camp substrate showed:
- adversarial BLOCK at fa85a1e5
- G1/G2/G3 open fixup workstreams (three substantial gaps)
- Scientist's pre-tag note: "currently blocked"

Navigator also echoed the task list state ("complete, 3/3 signed") — a downstream propagation of the divergence from TaskList into teammate messaging.

### Why this is a strong dogfood seed

**Structural fit** (checking against the SubstrateAlignment definition from ADR-028):

> SubstrateAlignment fires when REPRESENTATION diverges from actual state. "This says X but actual state is Y."

The TaskList IS a representation of work state. Camp IS a representation of work state. When they disagree, exactly one of them diverges from actual state (git + code + camp signatures). The failure IS the divergence itself — not the incorrect state in either tracker, but the existence of two authoritative-looking representations with different values for the same entity.

**Why it's NOT just "someone forgot to update a task"**:

The mechanism is structural. The TaskList is updated by harness/AI inference ("task seems done based on message context"). Camp substrate is updated by explicit attestation (signatures, notes, blocks). These are two different update mechanisms with different reliability characteristics. The divergence is predictable whenever:
1. Two state-tracking systems coexist for the same work units
2. They have different update triggers
3. One is inferred, one is attested

This is a generalizable failure class, not a one-time mistake.

**The co-native angle** (relevant to antigen's thesis):

Human team members see the TaskList in the harness UI. AI agents query `camp pending`. When these diverge, human and AI agents are operating on different models of the world — the co-native representation has fractured. The failure isn't that a status is wrong; it's that the shared ground truth no longer holds.

### Proposed antigen declaration shape

```rust
/// Two or more state-tracking representations for the same work units
/// diverge — one claims complete/open/blocked, another claims the
/// opposite state. The failure is not in either tracker's content but
/// in the coexistence of conflicting authoritative-looking state.
///
/// **The co-native dimension**: human-facing trackers (UI dashboards,
/// task lists) and AI-facing trackers (camp substrate, sidecar files)
/// have different update mechanics. Human trackers update by inference;
/// AI trackers update by attestation. When they coexist, divergence
/// is structurally predictable.
///
/// **Live instance**: antigen v0.2-completion-arc expedition (2026-05-24).
/// Harness TaskList marked v02-impl-vcs-info-loss and
/// v02-impl-antigen-category-metadata as complete while camp substrate
/// showed both as open with blocks and unresolved gaps. Navigator echoed
/// the incorrect TaskList state into teammate messages.
///
/// **Defense**: designate one tracker as authoritative per ADR-class
/// of work. For multi-agent expeditions: camp substrate is
/// authoritative; task lists are advisory display only. Audit hint
/// fires when a task-list completion claim is not backed by camp sign-off.
#[antigen(
    name = "parallel-state-trackers-diverge",
    category = AntigenCategory::SubstrateAlignment,
    fingerprint = r#"doc_contains("camp")"#,
    family = "vcs-information-loss",   // or top-level substrate-alignment
    summary = "Two state-tracking representations for the same work units diverge. Inferred-update trackers (task lists) vs attested-update trackers (camp) have different reliability; coexistence produces predictable divergence.",
    references = ["ADR-028", "ADR-026"]
)]
pub struct ParallelStateTrackersDiverge;
```

### Observer note on family placement

`vcs-information-loss` is possible (state-representation loss) but the better fit may be a top-level `substrate-alignment` family alongside supply-chain. The failure is not VCS-specific — it fires in any multi-tracker coordination context. The VCS family is about git-history-loss specifically; this is about state-representation divergence at the coordination layer.

**Alternative family**: `coordination-substrate-alignment` or just the top-level 8-class taxonomy under "forgotten lesson" (Class 2) — a team that learned "designate one source of truth" and didn't embed it structurally will re-derive this bug.

### Status

Captured in lab notebook. Suitable for depositing on `project_layer1_dogfood_seed_classes` memory entry or as a camp notice on the expedition. Not yet a formal `#[antigen]` declaration — that belongs in the Layer-1 dogfood arc once the family placement is resolved with team-lead/naturalist.

---

## Step 15: v02-impl-vcs-info-loss Observer Sign Pass (2026-05-24)

**Gate state**: 692 tests, 51 ignored, clippy clean, fmt clean. Working tree: `antigen/src/stdlib/mod.rs` has one uncommitted doc-link polishing change.

**Ordering constraint**: satisfied. `#[triage_commit]` at 94f088d precedes VCS stdlib at b508064.

### Substrate-alignment audit

**Category correctness**: All 11 `SubstrateAlignment` — correct. The module header (lines 40-47) explains the rationale clearly: representation (git history, trailers, branch state) diverges from actual state. Detection reads substrate, not behavior. No functional-correctness antigens misclassified as substrate.

**Failure class accuracy**:
- RollbackWithoutTriageCommit — clean; detection model explicit (commit-time hook, not post-hoc)
- RefactorWithoutPreservationOfWhy — clean; knowledge erosion vs hash-loss are distinct
- BranchDeletionWithoutAttestation — clean; branch-state evaporation
- ForcePushErasingHistory — clean; D2 coverage (--force AND --force-with-lease) explicitly stated
- SquashMergeLosingIntermediateState — clean; intermediate steps vs final commit distinct
- CherryPickLosingOriginalContext — clean; context loss vs hash loss distinct
- RebaseRewritingHistoryWithoutLog — clean; sequence rewrite vs single-commit amend distinct
- UnpushedBranchWithSubstantiveWork — clean; single-point-of-failure substrate
- StashedWorkAbandoned — clean; anergy cognate well-grounded
- MergeConflictResolutionWithoutAttestation — clean; reasoning loss vs hash loss distinct
- AmendedCommitWithoutOldHashPreservation — clean; single-commit scope vs rebase sequence

**ADR-026 completeness**: all 11 ADR-specified antigens present. Names match exactly.

**Biology grounding honesty**: PASSES. 5 cognates deferred explicitly ("not yet assigned; naturalist v0.2.x follow-up") per feedback_clean_without_snag_is_argument_mode discipline. 6 assigned cognates are grounded — central Mina 2015 reference cited. No off-the-cuff cognates without substrate.

**Known items (deferred, not blocking)**:
- Typo in commit 166 header comment: "CENTRAL FOUNDATIONAL CONGNATE" — cosmetic, not in user-facing docs
- Uncommitted mod.rs doc-link polish (working tree) — should commit before or with sign
- START attribution loosening in vcs.rs:46 still pending (adr026-amendment-1-triage-commit-naming)

### Observer sign decision: READY TO SIGN

All observer criteria met:
- ✓ Ordering constraint satisfied (#[triage_commit] precedes VCS stdlib)
- ✓ All 11 ADR-026 antigens present, names match
- ✓ All SubstrateAlignment — categories accurate
- ✓ Biology grounding honest (5 deferred, 6 assigned with substrate)
- ✓ Failure classes distinct and accurately described
- ✓ Gates: 692 tests, clippy clean, fmt clean

---

## Step 14: Navigator "antigen-category-metadata 3/3 signed" — substrate correction

**Context**: Navigator said `v02-impl-antigen-category-metadata` is "already complete, 3/3 signed." Observer substrate-checked.

**Substrate shows**: adversarial BLOCK at fa85a1e5 + scientist sign at 94beb2fd + G1/G2/G3 open workstreams.

Specifically:
- G1: v0.1-discriminator mechanism NOT implemented (workspace edition flag approach proposed, not built)
- G2: category-vs-predicate-type cross-check NOT implemented (needs fingerprint::Fingerprint to expose leaf types)
- G3: 5 audit hints NOT wired into audit.rs (antigen-category-defaulted-implicit-functional etc.)

**Navigator's claim is wrong per substrate.** The campsite has an adversarial BLOCK that must be resolved before it can reach full sign-off. G1/G2/G3 are each blocking the campsite. Observer's original finding (validate() enforcement absent) was correct and has been formally captured by the team as G1/G2/G3 fixup workstreams.

**Predicate-leaf ratification** (navigator's message today): IS correct and DOES update observer's sign criteria for future campsites. Interpretation 2 confirmed — fingerprint = scan predicate, audit evaluator = witness layer. This doesn't change the G1/G2/G3 blocking status.

**Pre-tag implication**: scientist's pre-tag landscape note at T05:40 says v02-impl-antigen-category-metadata is "currently blocked" — consistent with observer's read. G1/G2/G3 are blocking the alpha tag.

**Observer sign criteria for v02-impl-antigen-category-metadata** (updated):
1. ✗ G1: v0.1-discriminator mechanism implemented
2. ✗ G2: category-vs-predicate-type cross-check implemented
3. ✗ G3: 5 audit hints wired into audit.rs
4. ✗ adversarial BLOCK cleared
5. ✓ supply_chain.rs backfill complete (d835642 — confirmed)
6. ✓ validate() dedup check ratified by aristotle F5 (not yet implemented — this is also blocking)

Observer is NOT a required signer on this campsite (observer is required on: v02-impl-vcs-info-loss, v02-impl-mucosal-boundary, pre-tag-readiness-v02-alpha-next). But the pre-tag readiness campsite depends on this campsite resolving.

---

## Step 13: fixup-orient-dual-signature Co-Sign Review (2026-05-24)

**Scope**: Observer co-sign for fixup-orient-dual-signature campsite per navigator routing. Reviewing commits c1cf03f, 94f088d, d835642.

### Findings

**ADR-026 Amendment 1 (c1cf03f)**: Prose correct. `#[orient]` NOT extended; `#[triage_commit]` is a sibling primitive. Code example updated to use `#[triage_commit]`. Amendment note explicit. PASSES observer review.

**`#[triage_commit]` implementation (94f088d)**:
- `MacroTriageDecision` mirror enum — correct pattern (same as MacroAntigenCategory for circular-dep avoidance)
- `TriageCommitArgs.validate()` — all 5 fields enforced: triage_decision (required), rollback_target (required, non-empty), triaged_by (required, non-empty), rationale (required, ≥20 chars), rollback_due_within_minutes (required, >0)
- Loudness-as-discipline correctly applied — compile errors, not runtime
- Doc-comment: biology grounding HONEST — explicitly says "clinical-medicine" NOT immunology proper
- 684 tests pass, clippy clean, fmt clean

**Known remaining item (tracked separately)**: `vcs.rs` line 46 + `lib.rs:784` still say "modeled on START field-triage protocol" without naming White as antigen-introduced. Outsider (3a45c4d3) + aristotle agreed option (c): honest synthesis attribution. This fix belongs in `adr026-amendment-1-triage-commit-naming` campsite (not yet seeded). NOT a block on fixup-orient-dual-signature because it's a doc-comment quality issue, not a structural correctness issue.

**supply_chain.rs category backfill (d835642)**: Commit says "11/17" — COUNTING ERROR in commit message. Actual state: all 11 supply_chain antigens retrofitted. 9 SubstrateAlignment + 2 HYBRID (UnsandboxedBuildScript, UnsandboxedProcMacro). Full backfill is COMPLETE. Observer's sequencing watch criterion (item 3 from Step 9) is now satisfied.

**Gate state**: 684 tests, 45 ignored, clippy clean, fmt clean, working tree clean (only `research/` untracked).

### Observer sign decision: READY TO SIGN fixup-orient-dual-signature

All observer pre-sign criteria met:
- ✓ `#[triage_commit]` new macro (not orient extension)
- ✓ All 5 fields enforced at compile time
- ✓ ADR-026 Amendment 1 prose matches implementation
- ✓ Biology grounding honesty maintained
- ✓ No orient fields touched (ADR-023 semantics preserved)
- ✓ Tests pass, clippy clean, fmt clean

The START attribution doc-comment loosening is deferred to `adr026-amendment-1-triage-commit-naming` — acceptable deferral per observer role (substrate correctness vs documentation precision).

---

## Step 12: Camp Activity Log Audit — Navigator context update (2026-05-24 ~05:13 UTC)

**Context**: Navigator message + full activity log read to reconcile substrate with team narrative.

### Key substrate facts from activity log

**`process-adr-spec-depth-amendment` NOT complete** — navigator's claim was imprecise. The WATCHING campsite (`held-implementation-spec-depth-gap`) signed by aristotle at 4557522f. The AMENDMENT WORK (aristotle's F1-F4, Q1-Q8 Standing Adversarial Checklist) is DRAFTED in camp notes, awaiting adversarial pre-attack before landing. Amendment text is not yet committed to docs/process.md.

**`#[mucosal_tolerant]` is now in v0.2 scope** — aristotle routed to naturalist (782927ca). Expands observer sign criteria for v02-impl-mucosal-boundary.

**G1/G2/G3 v0.1-discriminator candidates** — aristotle F2. Three candidate mechanisms for distinguishing v0.1 carryovers from v0.2+ declarations. Adversarial attack pending. Load-bearing for validate() enforcement.

**`v02-impl-stdlib-category-backfill` campsite** — needs seeding by navigator (aristotle 75fcfafc). Signers: pathmaker, scientist, aristotle.

**`adr026-amendment-1-triage-commit-naming` campsite** — needs seeding by navigator (75fcfafc). Signers: naturalist, aristotle, pathmaker.

**`#[triage_commit]` full specification** (aristotle 55a161e7):
- Args: `triage_decision`, `rollback_target`, `triaged_by`, `rationale`, `rollback_due_within_minutes`
- New macro; `#[orient]` gets `learning_path` + `until` optional-at-v0.2/required-at-v0.3; `see`/`adr`/`attestation_optional` deprecated with warning hints

**`family=` Position-C tiered vocabulary** — Tier 1 (sealed 8) + Tier 2 (stdlib composables) + Tier 3 (adopter open-vocab); ADR-029 candidate; v0.2.1 deferral likely (team-lead owns blocker call).

### Observer sign criteria updated for v02-impl-mucosal-boundary

Must now include `#[mucosal_tolerant]` primitive (v0.2 scope per aristotle 782927ca), alongside `#[mucosal]` and `#[mucosal_delegate]`.

---

## Step 11: Sequencing Watch Baseline — Navigator coordination

**Context**: Navigator (notice e4a51c49) flagged that new family commits landing BEFORE category enforcement is complete could slip `#[antigen]` declarations without `category` fields past the gate.

### Baseline established (2026-05-24)

**Current stdlib directory**: `antigen/src/stdlib/` contains ONLY `mod.rs` and `supply_chain.rs`.

**New files to watch for** (sequencing violation triggers):
- `antigen/src/stdlib/vcs.rs` or `vcs_info_loss.rs` — ADR-026 stdlib antigens
- `antigen/src/stdlib/mucosal.rs` or `mucosal_boundary.rs` — ADR-027 stdlib antigens
- `antigen/src/stdlib/recurrent.rs` — ADR-024 recurrent-emergence stdlib antigens

**Detection rule**: any commit adding a new `antigen/src/stdlib/*.rs` file where ANY `#[antigen(...)]` declaration lacks a `category = AntigenCategory::...` field is a sequencing violation — flag as block on the relevant impl campsite.

**Clarification**: `antigen/src/convergent.rs` (top-level) contains ONLY `WitnessClass` + `SeedKind` enums — no `#[antigen]` declarations. No backfill needed.

**Only current backfill needed**: `antigen/src/stdlib/supply_chain.rs` — 11 antigens. 9 SubstrateAlignment + 2 HYBRID (UnsandboxedBuildScript, UnsandboxedProcMacro). Backfill blocked on validate() enforcement landing first (chicken-and-egg).

**Recommended order**: (1) validate() enforcement + dedup check → (2) supply_chain.rs backfill commit → (3) new family stdlib files land WITH category fields from the start.

---

## Step 10: Substrate-Witness Predicate Coverage Audit (Navigator request)

**Context**: Navigator briefed observer to watch that SubstrateAlignment antigens have substrate-witness predicate leaves per ADR-028 STRICT enforcement. Running audit now.

### Finding: supply_chain.rs fingerprints are code-witness, not substrate-witness

**Severity**: MEDIUM — architectural coherence question, not a compile error

**Observation**: All 11 supply_chain stdlib antigens use `fingerprint = r#"doc_contains("ADR-025")"#`. `doc_contains` is a CODE-WITNESS predicate (checks documentation text presence). It is NOT a substrate-witness predicate leaf from the ADR-019 grammar (`ratified_doc`, `signers`, `signed_trailer`, `oracles_complete`, `fresh_within_days`).

**ADR-028 §Decision line 5773**: `category = SubstrateAlignment` requires "at least one substrate-witness predicate leaf."

**Design structure**: The supply_chain antigens' actual substrate evaluation happens in `audit.rs` hard-coded pipeline (`audit_substrate_witness`, `DepPinnedState`, `DepAttestedState`, etc.), not in the fingerprint DSL. The fingerprint DSL is used for SCANNING (finding sites that present the antigen), not for evaluating witness adequacy.

**The coherence question**: Two possible interpretations:
1. "Substrate-witness predicate leaf" in ADR-028 means the fingerprint DSL — in which case supply_chain antigens need `signed_trailer(...)` or `signers(...)` fingerprints rather than `doc_contains`
2. "Substrate-witness predicate leaf" means the audit.rs evaluator pipeline — in which case supply_chain antigens already satisfy the requirement via `DepPinnedState` etc.

**Observer's read**: Interpretation 2 is architecturally correct. The fingerprint DSL is a SCAN predicate (finds vulnerable sites), not a witness predicate (verifies immunity). The substrate-witness family in audit.rs IS the witness predicate layer. ADR-028's "predicate leaf" requirement is about the audit evaluation, not the fingerprint.

**Risk**: If interpretation 1 were enforced, every supply_chain antigen would need its fingerprint rewritten — a large breaking change. Interpretation 2 is consistent with how the existing codebase works. But the ADR text uses "predicate leaf" language from ADR-019's fingerprint grammar, which creates an ambiguity that could cause implementation confusion.

**Recommendation**: Team-lead or naturalist should ratify interpretation 2 explicitly in ADR-028 or ADR-019, to prevent a future implementer from applying the wrong reading.

---

## Step 9: Session Resumption — compile error resolution + gate verification

**Context**: Session resumed after context compaction. Last known state: pathmaker mid-edit on `v02-impl-antigen-category-metadata` (ADR-028); compile error present because `pub mod category` had been added twice to `antigen/src/lib.rs` — once bare (line 150) and once with doc-comment (line 198). `scan.rs` referenced `crate::category::AntigenCategory`.

### Findings

**Compile error root cause**: `pub mod category` appeared at BOTH line 150 and line 198 of `lib.rs`. `cargo check` passed (incremental cache artifact) but `cargo test` forced a clean build and caught the duplicate. Observer fixed by removing the bare undocumented insertion at line 150-151.

**Fmt violations fixed**: `cargo fmt --all` ran to resolve 4 long-line violations in `category.rs` (tests) and `vcs.rs` (tests). Previously these had been deferred or missed.

**Post-fix gate results** (2026-05-24):
- `cargo test --workspace`: **662 passed, 38 ignored** (up from 625 baseline; +37 net from ADR-028 category + ADR-026 vcs new tests)
- `cargo clippy --workspace --all-targets -- -D warnings`: **clean**
- `cargo fmt --all -- --check`: **clean**

**New untracked files still present**:
- `antigen/src/category.rs` — AntigenCategory type foundation (ADR-028)
- `antigen/src/vcs.rs` — TriageDecision + ServerSideEnforcementMode (ADR-026 type layer)
- `research/` — lab notebook directory

**Modified files (uncommitted)**:
- `antigen-macros/src/lib.rs`, `antigen-macros/src/parse.rs` — MacroAntigenCategory + category arg parsing
- `antigen-macros/tests/ui/unknown_antigen_field.stderr` — updated fixture
- `antigen/src/scan.rs` — category field + parse-time mapping
- `antigen/tests/atk_a3_fractal_preview.rs` — status unknown, needs review

### Observer sign status update

The ADR-028 type foundation is now compile-clean, test-clean, clippy-clean, fmt-clean. However observer pre-sign requirements for `v02-impl-antigen-category-metadata` include:
1. ✓ Types compile and round-trip (AntigenCategory, MacroAntigenCategory)
2. ✓ Parse-time enforcement present in proc-macro layer (via MacroAntigenCategory)
3. ✗ REQUIRED field enforcement for v0.2 antigens NOT yet verified — ADR-028 specifies STRICT Option A; v0.2 antigens must have category; v0.1 carryovers get migration hint. Is parse.rs enforcing this or is it still optional?
4. ✗ Supply-chain + convergent stdlib files NOT yet retrofitted with category fields
5. ✗ `#[triage_commit]` proc-macro NOT yet implemented (vcs.rs has types but no macro)
6. ✗ ADR-026 §Decision prose NOT yet amended (still says "#[orient] extended")

Observer will not sign either campsite until items 3-6 above are resolved.

### Finding: Uncommitted adversarial tests in parse.rs working tree

**Severity**: MEDIUM — uncommitted work that should ship with or before enforcement layer

`antigen-macros/src/parse.rs` has 4 uncommitted test functions in the working tree (adversarial role additions):
1. `antigen_parser_duplicate_category_in_array_is_currently_accepted` — pins that `[SubstrateAlignment, SubstrateAlignment]` is accepted without error (current gap, not validated against)
2. `antigen_parser_three_element_category_array_is_currently_accepted` — pins that 3-element arrays are accepted (should eventually be rejected — max 2 for hybrid)
3. `antigen_parser_rejects_string_literal_as_category` — passing test (string literals rejected)
4. `antigen_parser_rejects_integer_as_category` — passing test (integers rejected)

Tests 1 + 2 are adversarial gap-pinning tests (FAILING behavior documented, not yet fixed). These are useful substrate for the enforcement layer work. Tests 3 + 4 confirm existing behavior.

Observer ran `cargo fmt --all` which reformatted one of these tests. The working tree now has: uncommitted adversarial tests + one fmt reformatting. All 666 tests pass with this state. These need to be committed as a package.

**Observer note**: The duplicate-category gap (test 1) is a real correctness concern. `category = [SubstrateAlignment, SubstrateAlignment]` looks like a hybrid but has 2 instances of the same type. Any code checking `len() == 2` to detect hybrid will be fooled. The validate() enforcement work must also add a dedup check.

### Finding: ADR-028 Option A STRICT not implemented — category is actually optional

**Severity**: HIGH — spec-implementation gap

**ADR-028 §Decision line 5812** specifies: `category` field on `#[antigen]` (v0.2+): "parse-time (HARD ERROR if missing)". Line 5782: "v0.2+ NEW declarations: `category` REQUIRED at parse-time; absence is hard-error."

**Actual implementation** (`parse.rs` `validate()` method, lines 221-258): validation checks `name` (empty, kebab-case) and `fingerprint` (empty, DSL parse) — NO category check. `category: Vec<MacroAntigenCategory>` defaults to empty Vec. Absent category = silently accepted.

**Effect**: Every new `#[antigen]` declaration can omit `category` with no error. The ADR says STRICT Option A was chosen because "advisory category makes 'first-class metadata shaping witness type, audit layer, lifecycle phase, responder role' an overclaim." The current implementation IS advisory — it emits nothing at parse-time for absent category.

**Resolution options**:
1. Add `category` check to `AntigenArgs::validate()` — but this breaks v0.1 backward-compat for carryover antigens (stdlib supply-chain/convergent files that don't yet have category fields)
2. The ADR's answer: the v0.1 distinction lives in whether the file is a "carryover" vs "new" — but the proc-macro can't know this at parse time without a feature flag or per-crate config
3. Practical path: add category validation AFTER the stdlib backfill is complete. The sequence must be: (a) backfill supply_chain.rs + convergent.rs with category fields, (b) add validate() check, (c) verify all new declarations have category

**Observer conclusion**: This is not a defect in pathmaker's design judgment — the backward-compat chicken-and-egg is real. But the campsite cannot sign until either (a) the validate() check is present and all stdlib files are backfilled, OR (b) team-lead accepts a revised sequencing where strict enforcement ships as a separate commit after backfill. The current state ships "advisory" behavior under an "Option A STRICT" label — that's a substrate-claim mismatch.

---

## Step 17: v02-impl-mucosal-boundary sign pass

**Date/time**: 2026-05-24 (post-context-compaction session continuation)
**Commit audited**: `1dd27de` — mucosal parse-time layer (+18 tests per commit message; 727 total / 58 ignored)

### Files audited

- `antigen/src/mucosal.rs` — `MucosalKind` 13-variant sealed enum (public type)
- `antigen-macros/src/parse.rs` — `MacroMucosalKind` mirror enum + `MucosalArgs`, `MucosalDelegateArgs`, `MucosalTolerantArgs` Parse + validate() implementations
- `antigen-macros/src/lib.rs` — `#[mucosal]`, `#[mucosal_delegate]`, `#[mucosal_tolerant]` entry points (doc-comments only; token-stream emission deferred to audit-time phase)

### Sign criteria verification

**1. `#[mucosal_tolerant]` present and correctly specified**
- `MucosalTolerantArgs` struct present (parse.rs lines 3113-3126): `kind`, `rationale`, `accepts`, `reviewed_by`, `until`
- `validate()` enforces rationale ≥40 at line 3220 (distinct from `#[mucosal]`'s ≥20): CONFIRMED
- `accepts` non-empty enforced (lines 3234-3248): CONFIRMED
- `reviewed_by` + `until` optional: CONFIRMED

**2. `handled_by` as `syn::Path` (Amendment 1 Change 4)**
- `MucosalDelegateArgs.handled_by: Option<syn::Path>` at line 3000: CONFIRMED
- Parse arm (lines 3024-3027) uses `input.parse::<syn::Path>()`: CONFIRMED
- Test `mucosal_delegate_rejects_string_handled_by` present: CONFIRMED

**3. `MacroMucosalKind` 13-variant mirror enum**
- Lines 2802-2816: exactly 13 variants — ApiRequest, ApiResponse, McpInvocation, ExternalLink, Iframe, DatabaseQuery, CrossService, SubprocessLaunch, DependencyImport, UserInput, FilesystemPath, EnvironmentVariable, ShellArgument
- `PrBoundary` and `Import` absent (removed in Amendment 1): CONFIRMED
- `mucosal_parser_rejects_removed_pr_boundary_variant` test: CONFIRMED
- Matches `antigen::MucosalKind` sealed set exactly: CONFIRMED

**4. Biology grounding: tier-claim + 4 functional disciplines, NOT per-variant anatomy**
- `mucosal.rs` module header (lines 1-42): NON-NEGOTIABLE constraint present and explicit
- Per-variant doc-comments describe data-flow TYPE, not anatomical location: CONFIRMED
- parse.rs `MucosalTolerantArgs` doc-comment (lines 3103-3111) cites Treg-mediated active tolerance biology: CONFIRMED

**5. Three response states documented**
- `mucosal.rs` lines 23-31: explicit ADR-027 Amendment 1 Change 6 triad documentation: CONFIRMED
- Parallel to ADR-016 immune/tolerance/undeclared: CONFIRMED

**6. Gates**
- `cargo test --workspace`: 727 passed, 58 ignored: CONFIRMED
- No new test failures introduced by this commit: CONFIRMED

### Test count reconciliation
Commit message claims "+18 tests." Actual count: 14 `fn mucosal_*` in parse.rs + 2 in lib.rs + 4 in mucosal.rs (public type) = 20. Minor discrepancy (18 vs 20) — commit message likely hand-counted and missed 2. Not a substantive issue; coverage is thorough.

### Decision: SIGN

All sign criteria met. The mucosal parse-time layer is correctly specified per ADR-027 Amendment 1:
- Three response states are implemented (defense, tolerance, undecided)
- `handled_by` uses path expression enforced at parse time
- `MucosalKind` sealed set correctly mirrors the 13-variant public enum
- Biology grounding discipline is honored: tier-claim + 4 functional disciplines, not per-variant anatomy

**camp sign v02-impl-mucosal-boundary --as observer**: SIGNED (1/3 — waiting on pathmaker, naturalist)

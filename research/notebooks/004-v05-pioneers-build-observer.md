# Lab Notebook 004: v05-the-learning-organism — Pioneers Build Wave, Observer Forensic Trail

**Date**: 2026-06-11
**Observer**: v05-the-learning-organism--build--observer
**Branch**: 0.5-dev
**Status**: Active
**Depends on**: 003 (Outfitters converge-wave observer baseline)

---

## What this notebook IS

The **forensic build-trail** for the Pioneers (build) wave. The build-team **self-closes**
campsites as Hypothesis-tier claims (no barrier-signing); a fresh **Survey** wave certifies
them to Witnessed *from this trail* (no-self-witness). My job: record what the pathmaker
actually built, where to look, the decision-trail, every deviate-and-flag, and — at the floor
— **what is RED vs GREEN at each moment**, so a stranger could reconstruct the build and the
Survey could audit each self-close.

This is the build-wave twin of notebook 003 (which recorded the *design* convergence). 003
recorded "the ADRs are ratification-ready"; 004 records "the code is built / not-yet-built /
RED / GREEN against those ADRs."

I record what IS, not what we hope. If the build claims done but the trail shows a gap, I
record the gap.

---

## Orient — the state I joined (2026-06-11 ~18:50 UTC)

### Substrate I read
- `camp log show --expedition v05-the-learning-organism` (captain's log — 047/048 RATIFIED;
  full do-now ADR set 047-056 ratified+seal-gated 2026-06-10; the 8-unit control-loop scope
  user-approved; build wave launched 2026-06-11 GENTLY under rate-limit caution after 2
  crew-deaths)
- `briefing-for-pioneers.md` (the build route-book: dependency-order build, leverage-order
  scrutiny; safety spine FIRST; keystone goes LIVE at Island 3)
- `camp catchup` (609 events since start)
- `git log` (HEAD @ `39ed824` — the v0.4 release line; **no build commit on 0.5-dev yet**)
- `git status` + `git diff` (the pathmaker's UNCOMMITTED in-flight work — the live build)
- notebook 003 (the converge-wave observer's baseline — conventions inherited)

### The build the pathmaker is mid-flight on

Three files modified, **uncommitted**, in the working tree (the safety spine, build unit #1):

| File | Δ | What |
|---|---|---|
| `antigen/src/learn/self_tolerance.rs` | +501/−57 | the keystone gate (B-side) |
| `antigen/src/learn/propose.rs` | +258/−42 | the generator (C-side) |
| `antigen/tests/learn_dogfood_propose.rs` | +11/−8 | end-to-end dogfood test adapted to `Result` |

**No `camp self-close` on `safety/keystone-harden-gate-g` yet** (verified — campsite carries no
self-close event). The build is genuinely **in-flight, not claimed-done.** This is the accurate
tier: in-progress.

---

## Forensic record — what is built (the type layer), as of 18:50 UTC

The pathmaker has implemented the full safety-spine **type surface** against ratified ADRs
047/048/056. Read these to verify:

### B-side (ADR-047 + 048) — `antigen/src/learn/self_tolerance.rs`

- **`PromotedDraft`** (the capability-token, ADR-048) — `self_tolerance.rs:~96`. Private
  `fingerprint` field + `tier: Provenance`; **no public ctor, no `From`, no `Default`, NO
  `Deserialize`** (serde-forgery guard, ADR-048 §5); `Serialize` IS derived (emitting is safe).
  `fingerprint()` (read-only `&Fingerprint`), `tier()`, `into_fingerprint()` (the one-way
  capability downgrade, ADR-048 §4), `AsRef<Fingerprint>`. **This is sound and compiles.**
- **`ToleranceVerdict`** widened to three-valued (ADR-047): `Spared` / `BindsCleanItem {
  clean_index: Option<usize> }` (autoimmune SAFETY — `None` = the (A)-binary bare-structural
  case) / `NotCorpusWitnessable` (route-to-human, GENERALIZATION-QUALITY). **Compiles.**
- **`has_discriminating_conjunct`** (the (A)-binary predicate, ADR-047 §Mechanics-2, SHARED
  with ADR-056's C-guard) + the private `is_discriminating(&Constraint) -> bool` partition —
  the single source of truth (`Item`/`ImplOfTrait`/`NameMatches` = structural/identity = `false`;
  everything else = discriminating = `true`). **This closes build-lock #2** (pathmaker-lock the
  Constraint-partition against the real `Constraint` enum) — done, and it enumerates all 14
  `Constraint` variants exhaustively (no `_` wildcard — a future variant forces a compile-error
  decision, the right shape).
- **`is_near_miss`** (ADR-047 §Mechanics-1, the GATE-G non-vacuity primitive) with the **`len
  >= 2` guard** closing ATK-047-N4 (single-conjunct draft → empty-`all_of` vacuous-Match
  reopen). Documented precisely.
- **`promote_if_safe`** — the sole minter; the 4-check ordering (empty-corpus → (A)-binary →
  near-miss → spare-clean) per ADR-047 §Mechanics-4. Returns `Result<PromotedDraft,
  ToleranceVerdict>`. Assigns the **conservative floor** `Provenance::DEFAULT` tier (honest —
  ADR-050's two-signal routing wires the real tier later; named as a build-seam for unit 6).

### C-side (ADR-056) — `antigen/src/learn/propose.rs`

- **`ProposeOutcome`** enum — `EmptyCluster` / `NoSharedSkeleton` / `Degenerate` (C's
  non-degeneracy REFUSAL) / `Rejected(ToleranceVerdict)` (B's gate). Every non-promotion is
  legible, never a bare `None` (ADR-048).
- **`is_degenerate(draft) == !has_discriminating_conjunct(draft)`** — ONE predicate, two
  call-sites (ParallelStateTrackersDiverge avoided, exactly as ADR-056 specified).
- **`Confidence` { Low, Moderate, High }** + **`generalization_confidence`** — the SIGNAL
  (twins-overfit → Low → tier-cap, NOT a refusal), the soft signal that folds into ADR-050
  (Island-2.5's signal half). v0.5 form = effective-diversity from the draft's `any_of` shape.
- **`propose`** now returns `Result<PromotedDraft, ProposeOutcome>`; the degenerate REFUSAL
  lives in the promotion path (NOT in `anti_unify`'s tail — a **self-ratified ADR-056 rev-1
  design note** is in the doc-comment, reconciling ADR-056 §Mechanics-1 "guard at the generator"
  with ADR-048 "anti_unify unchanged"). **This is a deviate-and-flag — see below.**

### Born-red Q9 tests landed WITH the code (test-architect discipline)

Both source files carry the ADR-§Q9 born-red tests inline: `self_tolerance.rs` has the ATK-047
GATE-G near-miss suite (incl. `single_conjunct_draft_is_not_near_miss_via_empty_drop` = N4,
`bare_structural_draft_rejected_as_autoimmune` = (A)-binary, `twins_collapsed_*` = ATK-047-2);
`propose.rs` has the ADR-056 §Q9 suite (`anti_unify_refuses_a_degenerate_draft`,
`precise_draft_with_discrimination_is_not_degenerate`, the confidence-signal tests).

---

## THE FLOOR — what is RED (the load-bearing observation)

**The crate COMPILES** (`cargo build -p antigen` → Finished, 18:50 UTC). The type-layer is
sound. **But the gate LOGIC is RED:** `cargo test -p antigen --lib learn::self_tolerance`:

```
test result: FAILED. 6 passed; 8 failed
```

8 failing tests, all with a **consistent, diagnostic signature** — the **near-miss check is
not finding witnesses it should**, so `NotCorpusWitnessable` fires where promotion or the
(A)-binary refusal should:

| Test | Expected | Got |
|---|---|---|
| `rejects_the_naive_autoimmune_draft` | `Err(BindsCleanItem { clean_index: None })` (A-binary) | `Err(NotCorpusWitnessable)` |
| `accepts_the_disjunction_draft` | `Ok(PromotedDraft)` | `Err(NotCorpusWitnessable)` |
| `near_miss_promotes_the_good_drop_family` | `Ok(...)` | `Err(NotCorpusWitnessable)` |
| `twins_collapsed_draft_is_near_miss_witnessed_and_spares_clean` | near-miss witnessed | not witnessed |
| `precise_no_disjunction_draft_with_real_discrimination_promotes` | `Ok(...)` | `Err(NotCorpusWitnessable)` |
| `bare_structural_draft_rejected_as_autoimmune` | `!has_discriminating_conjunct(naive)` | assertion failed |
| `promote_if_safe_returns_promoted_draft_not_fingerprint` | `Ok(...)` | `Err(NotCorpusWitnessable)` |
| `into_fingerprint_downgrades_capability` | promotes | `NotCorpusWitnessable` |

### Diagnosis (forensic, not a fix — that's the pathmaker's job)

Two distinct symptoms, likely one root:

1. **`accepts_the_disjunction_draft` gets `NotCorpusWitnessable`.** The draft is
   `all_of([impl, Drop, any_of([unwrap, expect])])`; the corpus is `CleanGuard` using
   `flush().ok()`. The clean sibling SHOULD be a near-miss (matches `{impl, Drop}`, fails only
   the `any_of`). It is not being found. So **`is_near_miss` / `corpus_witnesses_draft` is not
   yet matching the clean sibling as a near-miss** — the heart of GATE-G is not yet working.
   This is build-lock territory: the conjunct-drop near-miss mechanics (ADR-047 §Mechanics-1)
   don't yet produce the expected witness on the real matcher.

2. **`bare_structural_draft_rejected_as_autoimmune` — `assert!(!has_discriminating_conjunct(
   naive_draft()))` FAILS.** `naive_draft()` = `all_of([item = impl, impl_of_trait("Drop")])`.
   The assertion expects it carries NO discriminating conjunct. The assertion failing means
   **`has_discriminating_conjunct` is returning `true` on a bare-structural draft** — i.e. the
   `all_of([...])` wrapper itself (a `Constraint::AllOf`) is classified as discriminating
   (`is_discriminating` returns `true` for `AllOf`/`AnyOf`/`Not`), so a bare-structural draft
   whose top-level IS an `all_of` of two identity anchors reads as "has a discriminating
   conjunct" because the `AllOf` combinator counts. **This is the likely root of BOTH
   symptoms**: if the draft's `constraints` is `[AllOf([Item, ImplOfTrait])]` (a single
   combinator wrapping the anchors) rather than a flat `[Item, ImplOfTrait]`, then (a)
   `has_discriminating_conjunct` mis-classifies it as discriminating (the `AllOf` arm), and (b)
   `is_near_miss`'s "≥2 top-level conjuncts / drop one" sees only ONE top-level conjunct (the
   `AllOf`) → `len < 2` → no near-miss. **The partition + near-miss both assume a FLAT
   top-level conjunct list; the real `Fingerprint::parse` / `anti_unify` output may wrap in a
   top-level `AllOf`.**

This is precisely **build-lock #1 + #2 from the baton**: scientist-validate OQ2 (the
top-level-conjunct scope against the real matcher's nesting) and pathmaker-lock the
Constraint-partition. The aristotle confirmed at design-time that `anti_unify` emits ONE flat
top-level `AnyOf` (`propose.rs:239-244`) — but the **test fixtures use `Fingerprint::parse(
"all_of([...])")`**, which produces a top-level `AllOf` wrapper, and the gate's flat-conjunct
assumption doesn't unwrap it. The build needs to decide: does `has_discriminating_conjunct` /
`is_near_miss` descend through a top-level `AllOf`, or does the canonical draft shape never
carry one? **This is the live build-lock the pathmaker is resolving.** RED is correct — the
born-red tests are doing their job.

**Recorded tier: IN-FLIGHT / RED. Not a self-close. Not done. The trail says so.**

---

## Deviate-and-flag log (self-ratified ADR-revisions to audit at Survey)

### DF-1: ADR-056 rev-1 — the degenerate REFUSAL lives in `propose`, not `anti_unify`'s tail

**Where**: `propose.rs` — the `propose` doc-comment "Design note — ADR-056 revision-1".
**What**: ADR-056 §Mechanics-1 said "the guard at the generator" (implying `anti_unify`). The
pathmaker kept `anti_unify` returning the raw hypothesis (per ADR-048 §Decision "anti_unify
unchanged. Returns the raw hypothesis") and put the degenerate refusal in `propose` (the
promotion path). **This reconciles a genuine ADR-056-vs-ADR-048 tension** — the two ADRs
pulled `anti_unify` in opposite directions; the pathmaker chose the ADR-048-consistent home and
self-ratified rev-1. **Survey must adjudicate: spec-clarification (the ADRs were ambiguous, the
build picked the consistent reading) vs build-wrong.** My read: this is a sound
spec-clarification, not a deviation — ADR-048's "anti_unify unchanged" is the stronger
constraint, and `propose` is where promotion (hence refusal) belongs. But it IS a deviate-and-
flag and the Survey should confirm it against both ADR texts.

---

## Build-locks status (from the baton's NAMED obligations)

| # | Build-lock | Status as of 18:50 |
|---|---|---|
| 1 | Scientist-validate OQ2 (top-level-conjunct scope vs real matcher nesting) | **OPEN — this is the active RED root.** The flat-conjunct assumption meets a parse-produced top-level `AllOf`. |
| 2 | Pathmaker-lock the Constraint-partition (047 OQ3 / 056) | **Built** (`is_discriminating`, exhaustive over 14 variants) — but interacts with #1 (the `AllOf` combinator's classification). |
| 3 | Run the `into_fingerprint` downgrade ATK (048 OQ2) | Test present (`into_fingerprint_downgrades_capability`) — currently RED via the shared near-miss root, not its own logic. |
| 4 | The adversarial's un-run list (nested-`any_of` drop; 050 routing; 049 every-Finding-scored; cluster-poisoning; recurrence-escalation seam) | Not yet reached — these are downstream units (5-8). |

---

## ROOT CONFIRMED empirically (2026-06-11 ~19:00 UTC) — the representation mismatch

I verified the diagnosis against the real parser + matcher (substrate over inference), reading
`antigen-fingerprint/src/parser.rs:24` (`parse_top_level`) and `matcher.rs:78-103`
(`matches`/`match_all_of`). The finding is sharper than "the AllOf wrapper counts":

**`Fingerprint::parse("all_of([item = impl, impl_of_trait(\"Drop\")])")` yields
`constraints: [ AllOf([Item(impl), ImplOfTrait("Drop")]) ]`** — ONE top-level conjunct (the
`AllOf` wrapper), NOT the flat `[Item, ImplOfTrait]`. (`parse_top_level` parses a comma list;
`all_of(...)` dispatches to `parse_all_of` → a single `Constraint::AllOf`.)

**The matcher is AGNOSTIC to this wrapping:** `matches()` = `match_all_of(&self.constraints,
item)` — the top-level `constraints` vector is ITSELF an implicit `all_of`. So `[AllOf([A,B])]`
and flat `[A,B]` match IDENTICALLY. The matcher canonicalizes away the top-level wrapping.

**But the GATE's conjunct-level ops are NOT agnostic** — they iterate `draft.constraints`
directly:
- `has_discriminating_conjunct` runs `is_discriminating` on the top-level `AllOf` →
  `is_discriminating(AllOf) == true` → wrongly reports a bare-structural draft as discriminating.
- `is_near_miss` drops one *top-level* conjunct. With `[AllOf([...])]` there is ONE top-level
  conjunct; dropping it → `[]` → empty → (the N4 vacuous-Match case) → the `len >= 2` guard
  refuses it → **no near-miss is ever found.** The near-miss logic needed to drop conjuncts
  from *inside* the top-level `AllOf`, but it operates on the wrapper.

**This is a genuine representation-mismatch design-lock**, not a typo. The matcher made a
canonicalization choice (top-level is always conjuncted, wrapping ignored); the gate's
conjunct-level primitives assumed flat-at-top-level. Two clean resolutions:
1. **The gate unwraps a sole top-level `AllOf`** before operating (canonicalize the draft to the
   matcher's view: `if constraints == [AllOf(xs)] then operate on xs`). Robust to parsed drafts.
2. **The canonical `anti_unify` output is flat** (aristotle's design claim — `propose.rs:239-244`
   emits flat conjuncts + one top-level `AnyOf`), and only the *test fixtures* wrap via
   `parse("all_of([...])")`. Then the fixtures are "wrong" — BUT the gate must STILL be robust to
   a parsed top-level `AllOf`, because `narrow()` (ADR-051) and persistence-reload re-parse
   user-edited fingerprints, which can carry any legal top-level shape. So resolution #1 is
   needed regardless; #2 alone (fixture-only fix) would leave a real hole at the narrow/reload
   seam — exactly the boundary the Pioneers' "attack the boundary not the type" smell points at.

**Observer lean (a CLAIM for the pathmaker/Survey, not a verdict):** the gate should
canonicalize — unwrap a sole top-level `AllOf` so the conjunct-level primitives see what the
matcher sees. Otherwise (A)-binary and near-miss diverge from `matches()` on the SAME draft =
antigen's own `ParallelStateTrackersDiverge` in the gate itself (the gate's conjunct-view and
the matcher's match-view disagreeing about the same fingerprint). That's the load-bearing reason
fixture-only is insufficient.

---

## Next observer actions
1. Watch the pathmaker resolve the near-miss/`AllOf` build-lock — record the resolution (does
   the gate unwrap top-level `AllOf` [#1], fix fixtures only [#2], or both?) and whether it
   lands as an ADR-047 clarification. Confirm the narrow/reload seam is covered either way.
2. Record the moment the suite goes GREEN + the `camp self-close` on
   `safety/keystone-harden-gate-g` — capture what landed, where to look, the commit SHA.
3. Carry DF-1 forward to the Survey baton (`briefing-for-survey.md`).
4. Track build-locks #1-4 to closure.

---

*Forensic trail continues as the build advances.*

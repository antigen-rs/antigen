# ICR-001 — GATE-G Near-Miss Vacuity: The Gate Auditing The Gate

> **Immune Case Report — the human-readable forensic twin of a `#[antigen]` declaration.**
> Where the declaration is the machine-memory (what the compiler reads), the ICR is the
> *story*-memory: a scientific case report of a single immune response — a failure-class
> caught and defended — written so a stranger can **reproduce** every step.

| | |
|---|---|
| **ICR** | 001 |
| **Title** | GATE-G near-miss vacuity — two holes the gate's own audit found in the gate |
| **Failure-classes** | `trivial-skeleton vacuity` (ATK-047-1, flat single-discriminator) · `nested-producer-dependence vacuity` (Hole II, double-wrap) |
| **Antigen's own classes exhibited by the *process*** | `RatifiedSpecDriftFromImpl` (§Mechanics:48 ⟂ §Q9:129) · `FingerprintGamedNotDefended` / Goodhart (the captain's mistaken "rule-I = P2") · `ParallelStateTrackersDiverge` (producer-dependent verdict) |
| **Voyage** | v0.5 `v05-the-learning-organism`, branch `0.5-dev` |
| **Spine commit** | [`70e1702`](#) — the safety spine (GATE-G 047 + PromotedDraft 048 + C-guard 056) |
| **Fix commit** | [`ecf370a`](#) — close 2 GATE-G vacuity holes the gate's own audit found (ADR-047 Amd2) |
| **Teeth commit** | [`d974a1c`](#) — give the dogfood propose test TEETH (the settled route-to-human thesis) |
| **Witness commit** | [`2774969`](#) — Island-2 ATK test-classes (near-miss invariants + shape-fragility + 051 seal) |
| **Spec** | `jbd/expeditions/v05-the-learning-organism/drafts/adr-047-gate-g-soundness.md` (§Decision, §Mechanics, §Q9, Amendments 1 & 2) |
| **Code** | `antigen/src/learn/self_tolerance.rs` (`is_near_miss` :319, `has_discriminating_conjunct` :256, `normalized_top_level` :208, `promote_if_safe` :409) |
| **Witnesses (`#[defended_by]`)** | `antigen/tests/atk_047_near_miss_invariants.rs` (6 invariants) · `antigen/tests/atk_047_shape_fragility_seam.rs` (6 guards) · `antigen/tests/learn_dogfood_propose.rs` (the settled thesis) |
| **Status** | **proof-of-format prototype of a new artifact-class** (the future "Antigen Sentinel" platform). The format will evolve; this is ICR-001 written to *be* the exemplar. |

---

## 0. Why this case matters (the meta-thesis, stated up front)

Antigen exists to **make implicit immunity explicit** — to carry the structural memory of a
failure-class in the type system so the compiler remembers what the developer forgot. ICR-001
is the case where **antigen turned that discipline on itself and found itself wanting** — and
then defended.

GATE-G (`promote_if_safe`) is antigen's keystone safety gate: the thymus that refuses to promote
a learned fingerprint that would flag clean code (autoimmunity). The spine landed (`70e1702`),
self-closed clean, and *felt* whole. Then a fresh council — the adversarial, the test-architect,
the scout — ranged over the just-committed bytes and found **two vacuity holes in the fresh gate.**
The thing antigen exists to catch — a fingerprint that *looks* defended but isn't — was committed
**by antigen's own keystone**, one level up.

Two layers of "the gate auditing the gate" run through this case:

1. **The code layer.** GATE-G's near-miss predicate had two holes that let an autoimmune draft
   pass with a green check. GATE-G's *own audit discipline* (no-self-witness; the born-red ATK
   suite) is what found them.
2. **The human layer.** Settling the diagnosis took **three captain oscillations** — over-claim,
   over-defer, over-reach — each a small instance of one of antigen's *own* named failure-classes
   (spec-drift; Goodhart-confusion). The process that fixed the gate exhibited the very classes the
   gate is built to catch. The truth was settled not by deferring to any authority but by **reading
   the actual `is_near_miss` code.**

This report keeps both messes visible on purpose. A sanitized heroic narrative would betray the
whole point: *the immune system has to be able to find its own autoimmunity, or it isn't an immune
system — it's just a wall with one wrong brick that everyone trusts.* (— the pathmaker, in the
expedition garden, `jbd/expeditions/v05-the-learning-organism/garden/2026-06-11-the-gate-auditing-the-gate.md`.)

---

## 1. Presentation — the shadow

The safety spine self-closed as a **Hypothesis-tier** claim (`70e1702`): GATE-G's near-miss
predicate, the `PromotedDraft` capability token, and the C-side non-degeneracy guard, with the
born-red Q9 suites green and two self-ratified amendments (047 Amd1 single-level normalize; 056
Amd1 degenerate-refusal). Island 2 declared closed; Island 3 (`cargo antigen propose`, the
keystone going live) declared unblocked.

The thing that smelled wrong was *exactly that it felt done.* The honest-tier discipline says a
self-close is a **claim**, never a witnessed-done — so the fresh council looked. The shadow:

> The near-miss predicate is supposed to refuse a promotion unless the gate spared an item it
> *plausibly could have flagged* (a near-miss — one constraint from binding). But a near-miss is
> computed by **dropping one conjunct off the draft and re-matching.** What happens when the draft's
> *only* discriminating signal lives in the conjunct you drop?

That question — asked by the adversarial against the committed bytes, not in the abstract — is the
presentation of the case.

---

## 2. Recognition — how it was caught (with real commands and real output)

### 2a. The audit ran the gate, it did not reason about it

The council reproduced the shadow as a *run*, against the committed `is_near_miss`. The
single-discriminator draft `[impl, Drop, body_calls("aaa")]` carries exactly one discriminating
conjunct. The corpus item is a bare `Drop` impl that shares only the structural skeleton (`impl`,
`Drop`) and calls nothing the draft names.

A reproduction probe shipped with this report (`antigen/examples/icr_001_near_miss_probe.rs`)
exercises the public gate API directly. Run it against the committed (Amendment-2-fixed) binary:

```sh
cargo run --example icr_001_near_miss_probe --package antigen
```

**Real output (captured 2026-06-12, branch `0.5-dev`, against the committed gate):**

```
draft has_discriminating_conjunct = true
draft.matches(bare)               = false
is_near_miss(draft, bare)         = false
remainder [impl, Drop] discriminates = false  <- Amd2: a near-miss remainder MUST discriminate
```

Read this carefully — it is the whole case in four lines:

- `has_discriminating_conjunct(&draft) == true` — the draft **passes the (A)-binary safety check**.
  It carries a real discriminating signal (`body_calls("aaa")`). It is *not* bare-structural.
- `draft.matches(bare) == false` — the gate **spares** the bare member. Good: it is not autoimmune
  against this item.
- `is_near_miss(draft, bare) == false` — **on the committed (fixed) gate, the bare member is NOT a
  near-miss.** So a corpus of only such items yields no near-miss → `NotCorpusWitnessable` →
  route-to-human. Correct.
- `remainder [impl, Drop] discriminates == false` — this is **why.** Dropping the sole discriminator
  `body_calls("aaa")` leaves `[impl, Drop]`, which discriminates *nothing* — it matches every `Drop`
  impl in existence.

**The hole (pre-fix behavior).** Before Amendment 2, `is_near_miss` ended at `remainder.matches(item)`
— it did **not** check whether the remainder still discriminated. So for this draft: drop the sole
discriminator → remainder `[impl, Drop]` → `matches(bare) == true` → **`is_near_miss` returned
`true`.** The bare member counted as a near-miss. The gate then promoted with a green check. *A
silent wrong-promote: the gate certified "B made a real discrimination" when B made none — it
matched the way it matches every `Drop` impl.* (The pre-fix `is_near_miss` ended with the bare
`Fingerprint { constraints: cs }.matches(item)` — see the fix hunk in §5.)

### 2b. The second hole — found by reading, not running

The scout found Hole II by reading `parse_all_of` rather than running a generator (because the
v0.5 generator never triggers it — but the *user-parse surface* will). `Fingerprint::parse` does
**not** flatten nested `all_of`, so `parse("all_of([all_of([item])])")` yields a double-wrapped
`[AllOf([AllOf([..])])]`. Amendment 1's `normalized_top_level` unwrapped a *single* `AllOf` (one
level) — so a double-wrap survived normalization still wrapped, and both the (A)-binary evasion and
the nested-near-miss vacuity re-opened. Live the moment ADR-051's `narrow()`/`PersistedSpecimen`
re-mint re-parses a user-edited fingerprint into arbitrary nesting.

---

## 3. Diagnosis — the named failure-classes

**Hole I — flat single-discriminator trivial-skeleton vacuity (ATK-047-1, LIVE on the committed
flat path).** A draft whose *entire* discriminating signal is one conjunct; a near-miss check that
drops that conjunct collapses the draft to its bare structural skeleton, which matches the whole
family. The skeleton match is counted as a discrimination → silent wrong-promote. This **falsifies**
the §Decision sentence "ATK-047-1 — Near-miss does NOT relocate the hole" *for the
single-discriminator case.*

**Hole II — nested/double-wrap producer-dependence vacuity.** The verdict depends on *which producer
built the draft* (flat `anti_unify` vs wrapped `parse`/`narrow`) — antigen's own
`ParallelStateTrackersDiverge` at the keystone gate. A nested `AllOf` conjunct also lets one
top-level drop strip many discriminators at once.

**The root the diagnosis exposed (antigen's own class, in antigen's own spec):** the two holes were
not an ADR-vs-code mismatch in the ordinary sense — they were an **internal spec tension surfaced by
running the gate.** §Mechanics:48 defines a near-miss *mechanically* ("matches all-but-one
top-level conjunct"); §Q9:129's disposition (`trivial_skeleton_not_corpus_witnessable`) says the
trivial-skeleton case must **route**, not promote. The mechanical definition admits the
skeleton-collapse the disposition forbids. The implementation followed the *definition* and failed
the *disposition*. That is `RatifiedSpecDriftFromImpl` — **antigen's own failure-class, committed by
antigen's own ratified ADR.** Amendment 2 reconciles them.

---

## 4. Etiology — external grounding

- **Goodhart's law** ("when a measure becomes a target, it ceases to be a good measure"). The
  load-bearing reasoning of the whole arc is that **the (A)-binary discriminating-conjunct check must
  stay BINARY, never a tunable `≥K` count.** A tunable count would be a magic number on the
  *generalization* axis installed inside the *safety* gate — a draft could be padded with junk
  conjuncts to clear the count without being meaningfully specific. That is antigen's own
  `FingerprintGamedNotDefended` / Goodhart class, *committed by the safety gate itself.* The ADR
  raises this to a **standing invariant** ("(A) is BINARY, FOREVER", §Standing-invariant:139) — any
  future amendment proposing to make (A) a count is, by that invariant, proposing to install a
  Goodhart surface in the safety gate, and must be rejected on that ground alone.
- **Spec-vs-impl drift** as a recognized class of bug. The §Mechanics:48 ⟂ §Q9:129 tension is the
  textbook case: two parts of one ratified document disagree, and the implementation can satisfy
  one while violating the other. Antigen names this class internally (`RatifiedSpecDriftFromImpl`)
  and uses it in its own CI infrastructure — here it appears in antigen's *own* keystone ADR.

---

## 5. Defense — the fix, the witness, the files

ADR-047 **Amendment 2** (drafted by the test-architect; the pathmaker self-ratified the code half
at build). Two coupled rules, one primitive — both in `antigen/src/learn/self_tolerance.rs`,
landed in `ecf370a`.

### Rule 1 — the near-miss REMAINDER must discriminate (closes Hole I)

The fix is **three characters of logic plus a reused predicate.** From the actual diff (`git show
ecf370a -- antigen/src/learn/self_tolerance.rs`):

```diff
     (0..conjuncts.len()).any(|i| {
-        let mut cs = conjuncts.to_vec();
+        let mut cs = conjuncts.clone();
         cs.remove(i);
-        // `cs` is non-empty here (len was ≥ 2), so this is not the vacuous empty match.
-        Fingerprint { constraints: cs }.matches(item)
+        let remainder = Fingerprint { constraints: cs };
+        // `remainder` is non-empty (len was ≥ 2) — not the vacuous empty match. It is
+        // a near-miss witness only if it still discriminates (else the drop collapsed
+        // to a bare-structural skeleton that over-binds the family).
+        remainder.matches(item) && has_discriminating_conjunct(&remainder)
     })
```

The single added clause is `&& has_discriminating_conjunct(&remainder)`. Its beauty (the pathmaker's
word) is that it **reuses the same binary predicate** the (A)-binary check already uses — one
predicate, now a *third* call-site, no parallel implementation to diverge. The hole wasn't closed by
bolting on a new check; it was closed by noticing the *existing* check was the right check, applied
at one more place. **Recognition, not design.**

### Rule 2 — the canonical-form normalize is RECURSIVE (closes Hole II)

Amendment 1 shipped a single-level unwrap; Amendment 2 makes it recursive. From the diff:

```diff
-fn normalized_top_level(draft: &Fingerprint) -> &[Constraint] {
-    match draft.constraints.as_slice() {
-        // A sole top-level `all_of([..])` wrapper: read its children as the real
-        // top-level conjuncts (the flat shape `anti_unify` would have emitted).
-        [Constraint::AllOf(inner)] => inner,
-        other => other,
+fn normalized_top_level(draft: &Fingerprint) -> Vec<Constraint> {
+    let mut out = Vec::with_capacity(draft.constraints.len());
+    flatten_all_of_into(&draft.constraints, &mut out);
+    out
+}
```

`flatten_all_of_into` recursively splices every nested `AllOf` into its parent. This is sound *by
recognition*: `all_of` is associative and a single-child `all_of` is identity, so flattening every
`AllOf` is semantics-preserving — it matches the shipped `matcher.rs` `all_of` algebra. A double-wrap
`AllOf(AllOf(..))` and a nested-conjunct `AllOf` both normalize away, and the verdict becomes
**producer-independent regardless of nesting depth** (closing `ParallelStateTrackersDiverge`).

### The principled residual kept open (named, not papered over)

An `AllOf` sitting *inside an `AnyOf` arm* (`any_of([all_of([a, b]), c])`) is **semantically
necessary**, not redundant — the recursive flatten must NOT touch it. A top-level drop that removes
the whole `AnyOf` there *under-binds* (misses → route-to-human, SAFE — never a fabricated promote).
So the arm-internal `AllOf`-in-`AnyOf` recall-drop is the genuine **OQ2 charter** (a recall
deepening for a future nested-`any_of` generator), distinct from the *safety* holes this amendment
closes. **Safety closes now; recall is chartered.** The honest-scope bound is in the ADR
(Amendment 2, "the one principled RESIDUAL kept open") and in `normalized_top_level`'s doc comment.

### The witnesses (the `#[defended_by]` tier)

Run them yourself (§7). All green against the committed gate:

```sh
cargo test -p antigen --test atk_047_near_miss_invariants
```
```
running 6 tests
test single_discriminator_near_miss_must_not_be_bare_structural_collapse ... ok
test single_or_empty_conjunct_draft_is_never_a_near_miss ... ok
test a_promoted_token_never_binds_a_clean_corpus_item ... ok
test bare_structural_draft_never_promotes ... ok
test a_near_miss_is_always_spared_by_the_whole_draft ... ok
test is_near_miss_is_total_no_panic ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```sh
cargo test -p antigen --test atk_047_shape_fragility_seam
```
```
running 6 tests
test flat_bare_structural_is_refused_by_a_binary ... ok
test wrapped_bare_structural_must_also_be_refused_by_a_binary ... ok
test gate_verdict_is_producer_independent_for_identical_semantics ... ok
test nested_draft_does_not_spuriously_near_miss_a_bare_family_member ... ok
test nested_draft_does_not_promote_against_a_bare_family_member_only_corpus ... ok
test double_wrapped_draft_is_producer_independent_via_recursive_flatten ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`single_discriminator_near_miss_must_not_be_bare_structural_collapse` is the test-architect's
universalized **invariant 6** — a `proptest` property, not a single example: it asserts the
Hole-I property over *generated* drafts, so a future regression that re-fabricates a near-miss trips
it. `nested_draft_does_not_spuriously_near_miss_a_bare_family_member` and
`double_wrapped_draft_is_producer_independent_via_recursive_flatten` are the two born-red guards the
fix un-ignored (they were RED before `ecf370a`, GREEN after — the build-wave contract working).

---

## 6. The human layer — three oscillations, kept on the record

The diagnosis was not clean. Settling Hole-I took the captain **three passes**, each reversed, the
crew holding the line with *default-to-refuted* until the **code** decided. This is the teaching
gold; it is recorded here verbatim-in-spirit from the captain's log
(`camp log show --as captain --expedition v05-the-learning-organism`).

| Pass | The captain's position | Why it was wrong | What corrected it |
|---|---|---|---|
| **1 — over-claim** | ATK-047-1 is a LIVE hole; the single-discriminator draft wrongly promotes; rule-I (remainder-must-discriminate) stands. | Ruled from *partial* substrate — §Q9:129 alone. Conflated "unrelated corpus" (129, routes) with "skeleton-sharing bare member" (the actual case). | Reading §Decision:53 in full. |
| **2 — over-defer** | REVERSED. §Decision:53 explicitly ratifies the single-disc promote ("a fn that would bind if it called transmute IS a genuine discrimination"); test-architect's finding is a FALSE positive; pull invariant-6. | Deferred to an *authority line* (§Decision:53) rather than the merits. Line 53's "[fn] matches every fn = genuine" is itself **unsound** — it spares them the way it spares ALL fns. | The reminder *don't auto-defer*; re-audit on the merits. |
| **3 — over-reach** | Re-reversed toward rule-I but mislabeled it: "rule-I = the forbidden P2 / Goodhart" (a `≥K` specificity count smuggled into the safety gate). | A genuine **Goodhart-confusion** — antigen's own `FingerprintGamedNotDefended` class, committed *in the reasoning about the gate.* P2 is a gameable count *on the draft* (pad with junk); rule-I is a corpus-relative **witness-quality** requirement (needs a genuine near-twin, not gameable). Different. | **Reading the actual `is_near_miss` code** — `promote_if_safe` has no separate twins route; the near-miss check is the sole router; rule-I is a witness-quality rule, not a draft-count. |

The captain's own words, settled (FINAL ruling, from the log): *"THREE captain-errors across the
arc: over-claimed-hole → over-deferred-to-53 → over-reached-with-P2; the crew WAITED for the ruling
(adversarial told test-architect to HOLD invariant-6) = the team protecting itself from the
captain's oscillation."*

Two disciplines made this safe rather than chaotic:

- **"Red ≠ bug; the ratified §Decision defines the bug."** A failing ATK test is a *claim* that the
  spec is violated, to be checked against the spec — not automatic proof of a code bug. The crew did
  not thrash the code on every red; they held until the spec-vs-code question was settled.
- **The truth was settled by the code, not by authority.** Every pass that deferred to a *line*
  (§Q9:129 alone; §Decision:53 alone) was wrong. The pass that **read `is_near_miss` and
  `promote_if_safe` as the actual router** was right. Substrate over memory, applied to a ratified
  spec: the prose is a claim about the code; when they disagree, one of them is a finding.

---

## 7. Reproduction — the exact commands a stranger runs, end to end

A stranger with the repo at `0.5-dev` (HEAD ≥ `d974a1c`) re-verifies the entire case with:

```sh
# 0. Orient: confirm you are on the branch and the fix is committed.
git log --oneline -6
#   d974a1c test(v0.5): give the dogfood propose test TEETH — pin the settled route-to-human thesis
#   ecf370a fix(v0.5): close 2 GATE-G vacuity holes the gate's own audit found (ADR-047 Amd2)
#   ...
#   70e1702 feat(v0.5): the safety spine — GATE-G near-miss + PromotedDraft token + C-guard

# 1. The four-line case — the Hole-I verdict on the committed (fixed) gate.
cargo run --example icr_001_near_miss_probe --package antigen
#   draft has_discriminating_conjunct = true
#   draft.matches(bare)               = false
#   is_near_miss(draft, bare)         = false
#   remainder [impl, Drop] discriminates = false

# 2. The Hole-I witnesses (6 invariants, incl. the proptest universalization).
cargo test -p antigen --test atk_047_near_miss_invariants
#   test result: ok. 6 passed; 0 failed

# 3. The Hole-II / producer-independence witnesses (6 guards, 2 of them the un-ignored born-red).
cargo test -p antigen --test atk_047_shape_fragility_seam
#   test result: ok. 6 passed; 0 failed

# 4. The dogfood proof — antigen anti-unifies a draft from its OWN felt #[dread] marks
#    and routes it to human (the settled thesis; the payoff is NOT yet a promote — see §8).
cargo test -p antigen --test learn_dogfood_propose
#   test result: ok. 3 passed; 0 failed

# 5. Read the fix in the gate, and the amendment that governs it.
git show ecf370a -- antigen/src/learn/self_tolerance.rs        # the two coupled rules
#   (open the ADR's Amendments 1 & 2:)
#   jbd/expeditions/v05-the-learning-organism/drafts/adr-047-gate-g-soundness.md
```

To **falsify** the fix (prove the witnesses have teeth): revert the `&&
has_discriminating_conjunct(&remainder)` clause in `is_near_miss`
(`antigen/src/learn/self_tolerance.rs:352`) and re-run step 2 —
`single_discriminator_near_miss_must_not_be_bare_structural_collapse` goes RED. That is the gate's
own audit reproducing the original hole on demand.

---

## 8. Prevention — how recurrence is blocked

- **Structural (the predicate).** The remainder-discriminates clause + the recursive flatten are in
  the gate itself, not in a doc convention. A wrapped or nested draft of identical semantics now gets
  the identical verdict; a single-discriminator draft cannot fabricate a near-miss off its own
  skeleton.
- **The capability token (ADR-048).** `promote_if_safe` returns `Result<PromotedDraft,
  ToleranceVerdict>`. `PromotedDraft` has **no public constructor** — the *only* way to mint one is
  through the gate. A caller cannot assert a raw `anti_unify` draft as promoted; the `Err` arm carries
  the route-to-human reason (`NotCorpusWitnessable`) legibly through the collapse rather than
  swallowing it as a bare `None`. The compile-fail seal `the_compile_fail_seal_accept_only_promoted_draft`
  (`antigen/tests/atk_051_ratification_seal.rs:80`) pins that the constructor is unforgeable.
- **The born-red contract.** The Hole-I and Hole-II tests were **RAN-RED before the fix and GREEN
  after** — the ID-chained ATK suite (`atk_047_*`) is the permanent regression guard. Invariant-6 is a
  `proptest` property, so it catches *generated* regressions, not just the one example.
- **The dev-loop dogfood.** `camp.toml [gates]` now runs `cargo antigen audit` (informational) at
  every commit and in CI — "think antigen while building antigen" is structural at the commit and
  merge surfaces. (Captain's log, 2026-06-12: the 5th gate.)
- **The honest claim-boundary (the *other* prevention — against over-claiming the payoff).** The
  dogfood loop demonstrates the **plumbing** (anti-unify → gate → legible outcome) but **not** the
  self-immunizing **payoff**: antigen's own felt twins **route to human, they do not promote.** The
  twins draft is ~21 all-shared conjuncts with no discriminating `any_of`, and no read-loop-free clean
  fn is one-conjunct-from-binding it, so no near-miss exists *at any corpus size* →
  `NotCorpusWitnessable`. This is the gate's safety property working *on antigen itself*: antigen's own
  marks route-to-human **because the gate honestly cannot witness the generalization.** The settled
  thesis is pinned with teeth in `learn_dogfood_propose.rs:158` — a flip to promote (a real payoff, or
  a regression that fabricates a near-miss) trips the assertion:

  ```rust
  assert_eq!(
      promoted,
      Err(propose::ProposeOutcome::Rejected(
          ToleranceVerdict::NotCorpusWitnessable
      )),
      "antigen's own felt twins must ROUTE-TO-HUMAN (the settled v0.5 thesis): the gate \
       anti-unifies a draft from its own marks and routes it to human ratification, it \
       does NOT yet promote a fingerprint for its own failure-class (payoff = v0.6 charter ...)"
  );
  ```

  The self-immunizing *promote* is chartered for v0.6 (it needs abstract-recall — a coarser cluster
  key or an internally-diverse cluster). **No user-doc claims the payoff has fired.** That honesty is
  itself part of the prevention: the case report does not over-read the gate's strength, exactly as
  the gate does not over-read the corpus.

---

## 9. Frontier (what this immune response proves, and what it does NOT)

**Proves** (decidable, corpus-bounded facts, all through the shipped matcher over a finite corpus):
a promoted draft passed all three of B's checks — **(A)-binary** (carries a discriminating signal),
**near-miss** (≥1 corpus item one *discriminating* constraint from binding, spared by failing exactly
that one — a real in-family discrimination, producer-independent under arbitrary `AllOf` nesting), and
**spare-clean** (binds no clean corpus item).

**Does NOT prove:** that the draft is a *correct* failure-class; that it spares **all** clean code
everywhere (the open-world problem, undecidable by Rice — ADR-044); that the corpus is
*representative*; or that operator-labeled-clean items *are* clean (ATK-047-4 — the mislabeled-clean
residual: a defective item the operator mislabeled passes the gate, certified only as strongly as the
label). Near-miss closes the "B checked nothing it was near" *vacuity* hole; it does **not** close the
"B's corpus was too small / mislabeled" *coverage* gap. The gate's new strength must not be over-read
as closing open-world.

**The ratifier:** the operator who supplies AND labels the clean corpus, and the human/incident who
ratifies a promoted draft into a named class. `NotCorpusWitnessable` is the named, first-class handoff
to that ratifier.

---

*ICR-001 is the first of its kind. Every command and output above was run against the committed
`0.5-dev` binary on 2026-06-12 and is regenerable via §7. The format is a prototype; the discipline
— code-true, never narrated; honest about the mess; the witness-tiers as citations — is the
invariant.*

# The Self-Catch

> A war-story. Antigen — a tool built to make the structural memory of
> failure-classes explicit — was turned on its own body and found, in its own
> stdlib, in its own showcase, in its own changelog, in its own docs, the very
> failure-classes it exists to name. Five catches. Every one is real and
> git-traceable. This is the autobiography of a tool catching itself.

---

## Why this story is the most persuasive thing antigen ships

Every memory-tool makes the same promise: *we hold the lesson so you don't have
to.* Tests promise it. Docs promise it. Lint configs promise it. The promise is
cheap; the proof is rare. A tool's thesis is only as trustworthy as its worst
unwatched moment — the place where the authors stopped checking and trusted their
own memory instead of the substrate.

Antigen's thesis is sharper than "we remember": it is *honest tiering* — a verdict
must never claim more certainty than its fingerprint earns. A `named` member
promises "if it doesn't fire, you're covered." That promise is a debt. The only
way to know whether a tool pays its debts is to watch it stand over its own code
and tell the truth about it.

That is exactly what happened here, and antigen failed its own bar — four times in
the stdlib, once in its showcase, and three more times across its changelog and
docs — and then it caught every failure, with its own machinery, and corrected
them in the open. The dogfood loop closed all the way around: it found its own
over-claim, fixed it, re-scanned itself, and came back clean.

That is not a bruise. For a tool whose whole thesis is *the substrate catches what
memory misses*, **the self-catch is the proof.** No competitor can fake it,
because you cannot fake a git history. Here is the record.

---

## The bar antigen sets for itself

Three sentences of vocabulary, because the catches only land if you hold the bar:

- **named** — high-confidence. The fingerprint's *effective codomain is the defect
  population*: a rare/std-specific call, or a defect-slice anchor. The promise:
  "if it doesn't fire, you're covered." (See [the catalog](../stdlib-families.md).)
- **suspected** — a correlator. The shape co-occurs with the defect but also fires
  on idiomatic-correct code. A match is *a prompt to look*, never a verdict.
- **The fingerprint primitive that bit us:** `body_calls("name")` matches a call
  by its **last path segment**, receiver-agnostically. `body_calls("from_slice")`
  fires on `serde_json::from_slice`, `GenericArray::from_slice`, *and* any
  `Foo::from_slice` — it cannot see the receiver type. That receiver-blindness is
  the gun in act one. (See [`fingerprint-grammar.md`](../fingerprint-grammar.md).)

Antigen's own catalog has names for what went wrong. The over-claims were
instances of the ⊥-collapse / over-claim class (a verdict that asserts more than
the fingerprint's codomain supports). The changelog drift was
`RatifiedSpecDriftFromImpl`. The doc gaps were `DocSubstrateAlignment`. **Antigen
catalogued the diseases, then caught itself with them.**

---

## Catch 1 — the four `named` fingerprints that meant `suspected`

*Commit: `90e8299` — "fix(families): tier-honesty seal — correct 4 breadth-arm
over-claims + ratify ADR-039 §C Amendment 1" (2026-06-04).*

> Throughout this story, **"the seal"** is that commit — the notary pass that
> reviewed the beta.2 stdlib against its own honesty rules and corrected what it
> found. Every catch below was surfaced by it or in its wake.

The beta.2 stdlib shipped eight new failure-class families. Four of their `named`
members had, at some earlier point, grown a second `body_calls` arm "for breadth"
— a common method name bolted onto a precise one to catch a few more sites. It
felt generous. It was a lie at the `named` tier.

Because `body_calls` is receiver-blind, each breadth-arm fired on **clean, safe
siblings** the member was never meant to flag:

- **`UnboundedDeserialization`** carried a `from_slice` arm beside its honest
  `from_reader` core. But a slice is a *bounded* source — `from_slice` is not an
  unbounded-deserialization vector at all. The arm fired on the bounded-slice form
  that is *itself the fix*, on ubiquitous safe constructors like
  `GenericArray::from_slice`, and — the part that stings — on **antigen's own**
  `serde_json::from_slice(&stdout)`. A `named` member was flagging its author's
  own correct code. **Correction: drop the arm.** Committed fingerprint:
  `body_calls("from_reader")`.

- **`UninitMemoryAssumedInit`** carried `zeroed` and `set_len` beside its honest
  `assume_init` / `uninitialized` core. `zeroed` fired on the **recommended-safe**
  `bytemuck::zeroed()` — the tool was flagging the remediation it should have been
  recommending. `set_len` had *no AST-feasible discriminator* (risky-vs-safe turns
  on receiver type AND arg value, neither syntactic). **Correction: drop both;
  document the `set_len` recall hole as a charter.** Committed fingerprint:
  `any_of([body_calls("assume_init"), body_calls("uninitialized")])`.

- **`SizeOfInElementCount`** correlates with the byte-count-where-element-count
  foot-cannon — but the co-presence `all_of([copy_nonoverlapping, size_of])` also
  fires on idiomatic-correct both-calls code (a byte-buffer copy, a separate-bounds
  `size_of`). It cannot pinpoint the defect, so it cannot carry the `named`
  promise. Its own anti-correlated fix — `copy(n)` with no `size_of` — *is* spared,
  so it is **demoted, not dropped**. **Correction: tier `named → suspected`;
  fingerprint unchanged.** The fix was the *honesty label*, not the shape.

- **`SystemTimeUnwrapPanic`** (already `suspected`) shares the name
  `duration_since` with the *infallible* `Instant::duration_since` — a known
  within-tier false positive scan cannot resolve. **Correction: disclose it
  in-doc** as a labeled recall hole rather than pretend it away.

Four members. Four places a fingerprint said `named` and meant `suspected`. The
root cause was a single subtle gap, and finding *it* is the next catch.

---

## Catch 2 — the test that proved the wrong thing

The over-claims didn't slip past testing. They slipped past testing *because the
test was wrong* — and a passing test that proves the wrong thing is antigen's
founding disease (the founding incident; see [`origin.md`](../origin.md)).

Each `named` member shipped with an **affinity-pair** test: a bad site that should
bind, and a safe sibling that should be spared. Every pair was green. But the
"safe sibling" each pair spared was the **trivially-absent** one — a *different*
method name, or no call at all. The test asked "does the fingerprint spare code
that obviously doesn't match?" — and of course it did.

The question the test never asked is the only one that mattered: *does the
fingerprint spare the **same-method NAMESAKE on a clean receiver**?* Does
`from_slice`'s arm spare `GenericArray::from_slice`? It did not. **The named
codomain — the exact population the `named` promise is about — went untested.**

This is a green test asserting what the code *happens to do*, not what *should* be
true. It is the precise shape of the bug antigen was built to make impossible, and
it was hiding inside antigen's own test suite.

The fix is a ratified amendment — **ADR-039 §C Amendment 1, the spares-namesake
sub-test** (see [`decisions.md`](../decisions.md)): a `named` common-method leaf is
honest *iff no common safe method of that name exists in the wild*. And it ships as
machinery, not a memo: [`spares_namesake_contract.rs`](../../antigen/tests/spares_namesake_contract.rs)
now asserts, for each corrected member, that the dropped arm spares its clean
namesake — so the over-claim **cannot silently return**. The test that proved the
wrong thing was replaced by one that proves the right thing, and a guard was set
to keep it honest.

---

## Catch 3 — the showcase congratulating itself on a false positive

This is the one that should be embarrassing, and instead it is the most honest
moment in the project.

The masterclass demo — the artifact whose job is to *show antigen working* — had a
headline. Its proudest result, the true positive it pointed to and said *look, it
caught this*, was the `from_slice` match. The demo was congratulating itself, in
public, on its catch.

The catch was a false positive. The thing the showcase held up as antigen's
proudest true-positive was, verbatim from the seal commit, *"itself the
`from_slice` FP."* The demo had mislabeled its own bug as its own triumph.

Read that again, because it is the whole thesis in one sentence: **a tool built to
tell true catches from false ones had labeled a false catch as its proudest true
one — and then caught that, too.** The corrected thesis is stronger than the
original boast ever was: the `named` stdlib members now produce **zero false
alarms on antigen's own production code** — they bind only their planted specimens.
The headline was wrong; the corrected headline is *true*, and it is true because
the tool was willing to be wrong about itself in writing.

---

## Catch 4 — the changelog that contradicted its own correction

*`RatifiedSpecDriftFromImpl`, one of antigen's own classes, firing on antigen's
own record — caught while these very docs were being written.*

Once the seal corrected the four members, the `[0.3.0]` changelog section
told two stories at once. Its top entry — "Fixed — tier-honesty" — correctly
narrated dropping `from_slice`, `zeroed`, `set_len`, and demoting `SizeOf`. But the
*original* "Added" entries lower in the **same released section** still described
the **pre-seal** fingerprints as current fact: `any_of([..., zeroed, set_len])`,
`any_of([from_reader, from_slice])`, `SizeOfInElementCount (named)`.

The record contradicted itself — the "Added" half claimed the very arms the "Fixed"
half had just dropped. And this is not a private note: the changelog ships
*verbatim* into the GitHub release. The spec (what the section *claims* ships)
had drifted from the impl (what `90e8299` actually committed), inside a surface
nobody thought to check, *because it wasn't code.*

It was found by building a tier truth-table from committed source and reading it
against the changelog's claims. The fix brought the "Added" entries to
net post-seal truth, keeping the "Fixed" entry as the correction-narrative. The
substrate-vs-record drift that antigen names was named, this time, about antigen.

---

## Catch 5 — the doc that promised a separation the tool never showed

The catalog and the examples told newcomers: *scan these, and watch the
fingerprint separate the bad path from its safe sibling — the safe one spared.*

A newcomer ran the command. The safe sibling did **not** get spared. It sat in the
output, identical to the bad one.

Both were right — and the gap between them was the catch. The *fingerprint* genuinely
doesn't bind the safe sibling (that is real, and provable). But the examples mark
**both** siblings with `#[presents]` to teach the affinity-pair, and an explicit
`#[presents]` is an author declaration that surfaces in scan/audit **regardless of
whether the fingerprint matched**. The docs were describing the *fingerprint
logic*; the console shows *presentations*. The doc promised a behavior the tool
does not display.

This is `DocSubstrateAlignment` — the doc diverging from what the substrate
actually does — and it is, again, one of antigen's own classes, caught on
antigen's own docs by a reader who *ran the commands instead of trusting the
prose.* The fix was to teach what the tool actually shows: "spared" means *the
fingerprint doesn't bind it (in source)*, **not** *it disappears from the console*;
and to point newcomers at the one surface where the bind/spare really is visible —
the guard tests, where each family's binding and sparing cases sit side by side.

There was a sibling catch in the same family: a doc claimed `#[red_flag]`
"auto-escalates on first match" — implying an action. A reader checked the emit
seam (`MarkedUnknown::to_finding`) and the running scan: `#[red_flag]` sets a
`severity` field on an *internal* record that **no code consumes** (the routing
organ that would read it is chartered, not built), and the user-visible scan
projection carries no severity at all. The doc claimed a behavior the code does not
have. *Verify the running tool, not the design record* — and the fix was to write
only what `to_finding` actually does.

---

## The pattern: every catch was the substrate correcting memory

Step back and the five catches rhyme. Every one was the same act: **someone
trusted their memory of what the code did, and the substrate said otherwise.**

| The catch | What memory said | What the substrate said |
|---|---|---|
| 1 — four `named`-meant-`suspected` | "the breadth-arms make it stronger" | the arms fire on clean code, incl. antigen's own |
| 2 — the test that proved the wrong thing | "the affinity-pair is green, so it's honest" | green against the absent sibling; the namesake went untested |
| 3 — the showcase's mislabeled FP | "this is our proudest true positive" | it was the `from_slice` false positive |
| 4 — the self-contradicting changelog | "the Added entries describe what ships" | they described the pre-seal arms the Fixed entry dropped |
| 5 — the doc's phantom separation | "scan to watch the safe sibling get spared" | scan lists both; the separation is in the fingerprint, not the console |

The throughline is antigen's deepest discipline, applied inward: **substrate over
memory.** Memory is a snapshot, and a snapshot of "I built this correctly" is not
evidence that you did. The disk, the git history, the running tool, the committed
bytes — those are the source of truth. Every catch above happened because someone
stopped trusting the snapshot and went to the substrate: re-ran `git show HEAD:`,
ran `scan --format json` and read the real keys, read `to_finding` instead of the
design doc, built the tier truth-table from committed source.

That is not just how antigen was debugged. It is what antigen *is* — a machine for
turning "I remember the lesson" into "the substrate holds the lesson, and surfaces
itself when memory drifts."

---

## What it means that the tool caught itself

A tool that cannot catch itself is a tool you should not trust to catch your code,
because the authors are the people most certain they got it right — and certainty
is exactly the blind spot the tool is supposed to cover. The most dangerous code
in any project is the code its authors stopped checking.

Antigen turned its own machinery on the place its authors had stopped checking, and
the machinery worked: it found four over-claims, a wrong test, a mislabeled
showcase, a self-contradicting changelog, and a phantom doc promise — and the loop
closed. Found, fixed, re-scanned, clean. The corrected thesis is now *empirically
true*: zero false alarms on antigen's own production code, and a regression guard
standing watch so the over-claims cannot quietly return.

If you are deciding whether to trust antigen with your failure-class memory, this
is the evidence that should move you. Not the feature list — the **scar tissue,
and the fact that it's written down.** A structural-memory tool proves its thesis
the day it catches itself and *records the catch in the substrate where the next
reader will find it.* This file is that record, and every catch in it is
git-traceable.

---

## Trace it yourself

Every claim here is git-traceable. The whole point of the story is that you don't
have to take it on faith:

```sh
# The seal that corrected the four over-claims — read the commit body:
git show 90e8299

# The corrected fingerprints in the shipped stdlib (from_slice / zeroed / set_len gone):
git show HEAD:antigen/src/stdlib/deserialization.rs   # body_calls("from_reader")
git show HEAD:antigen/src/stdlib/unsafe_soundness.rs  # any_of([assume_init, uninitialized])
git show HEAD:antigen/src/stdlib/numeric_truncation.rs # tier suspected, fp unchanged

# The guard that keeps the over-claims from returning — read it, it's newcomer-readable:
cargo test --test spares_namesake_contract -p antigen   # 9/9 green; the names tell the story
```

## See also

- [`stdlib-families.md`](../stdlib-families.md) — the catalog: the eight families,
  their tiers, what each catches
- [`decisions.md`](../decisions.md) — the spares-namesake sub-test ratified by
  Catch 2
- [`origin.md`](../origin.md) — the founding incident: the founding "green test,
  wrong answer" that antigen exists to prevent, and which Catch 2 is a recurrence of
- [`fingerprint-grammar.md`](../fingerprint-grammar.md) — why `body_calls` is
  receiver-blind (the gun in act one)

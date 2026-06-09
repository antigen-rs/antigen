# Learning From Its Own Wounds

> A war-story. The first version of antigen could only *remember* failure-classes
> a human had already named. This version can *propose* one — anti-unify a cluster
> of structurally-similar defective sites into a draft fingerprint, then refuse to
> promote it unless it spares known-clean code. The proof that the proposer is
> trustworthy is not a synthetic fixture. It is that antigen turned the proposer on
> a cluster of its **own** unease — two places in its own source where a directory
> walk silently swallows an error — and generalized a real fingerprint out of them.
> A tool built to make the memory of failure explicit, learning a new failure-shape
> from wounds it felt in itself.

---

## The shape of the new power

Naming a failure-class by hand is the expensive step. Someone has to *notice* the
shape — "panic in a `Drop` impl," "deserialize without a size bound" — and write a
fingerprint that binds it. Antigen has always held those fingerprints once written.
What it could not do was write one.

The learning core changes that. Given a cluster of items that *feel* like the same
defect — several functions, several `impl`s, all carrying the same structural tell
— it anti-unifies them into a **draft fingerprint**: a candidate that binds every
member of the cluster. The draft is a hypothesis, not a verdict. It is offered for
a human or an incident to ratify into a named class; antigen never asserts on its
own that the draft names a real disease. (That line — observe, don't declare — is
load-bearing, and the rest of this story is mostly about it.)

Generalizing from examples is easy to do badly. The naive move — take the features
the cluster members share, drop the ones they differ on — over-generalizes, and the
direction it over-generalizes in is the dangerous one: *toward clean code*. A
`panic-in-Drop` cluster `{ .unwrap(), .expect() }` shares "is a `Drop` impl" and
differs on the panic call. Drop the differing leaf and the draft is "any `Drop`
impl" — which binds every clean `Drop` in the codebase. The generator's own output
*is* the false positive. A tool that ships that is a tool that flags its users'
correct code. Antigen has a word for a system attacking its own healthy tissue:
**autoimmunity.** Shipping a generator without a brake on it ships autoimmunity by
construction.

So the learning core is two halves that cannot be separated, and the rest of this
story is what each half does and how we proved it on antigen itself.

---

## The bar: two halves that must co-ship

Three sentences of vocabulary, because the story only lands if you hold the bar:

- **PROPOSE (the generator).** Anti-unifies a cluster *to disjunction*, not to the
  naive drop-leaves collapse. The features every member shares become a
  conjunction (AND'd); the features only *some* members carry become an
  `any_of([...])` disjunction — the discriminating wall that carries the cluster's
  signal without collapsing to the whole shared skeleton. The runnable example
  `cargo run --example learn_propose` makes this concrete on a toy `Drop` cluster
  whose two members both flush-then-`.take()` before panicking — one via
  `.unwrap()`, one via `.expect()`. The draft it prints is

  ```text
  item = Impl
  impl_of_trait("Drop")
  body_calls("flush")
  body_calls("take")
  any_of([body_calls("expect"), body_calls("unwrap")])
  ```

  — `flush` and `take` are shared by both members, so they stay AND'd; `unwrap`
  and `expect` distinguish them, so they anti-unify into the `any_of`. That
  `any_of` arm is the reason a clean sibling calling `.ok()` is spared: it carries
  neither `unwrap` nor `expect`, so the disjunction is `NoMatch`, so the whole
  `all_of` is `NoMatch`. (That cluster is the synthetic teaching fixture. The rest
  of this story runs the same machine on antigen's *own* source, where the cluster
  is two felt wounds and the shared conjunct is a different call.)

- **The self-tolerance gate (the selector).** A draft is *promotable* only if it
  spares every item in a corpus of known-clean code. A draft that binds a clean
  sibling is rejected — promoting it would flag that clean code. This is negative
  selection in the germinal center: a newly-mutated B-cell that gained
  self-reactivity is culled before it ever leaves, so generating new recognition
  and screening it against self are one coupled step. In the source it is
  `promote_if_safe`, and it is the only door to a promotable fingerprint.

- **The one safety-tangle.** Anti-unify-to-disjunction *reduces* autoimmunity but
  does not eliminate it: a cluster whose distinguishing leaf happens to also appear
  in clean code still over-binds. Only the corpus-checked gate eliminates it. So
  the generator must **never promote a draft except through the gate** — the two
  halves co-ship or neither ships. This is the highest-stakes line in the learning
  core, and it is enforced structurally, not by convention: `propose` is the only
  path to a promotable fingerprint, and it routes every draft through the gate.

Hold those three. Now watch antigen apply them to itself.

---

## The wounds: two places antigen distrusts its own code

Antigen carries a quiet, non-gating mark for a site its author *felt uneasy about*
but could not yet name a class for — `#[dread]`. It is an honest admission written
into the source: *something here might be wrong, and I do not have a fingerprint for
it.* The mark records the felt trigger; it does not assert a verdict.

Two of antigen's own functions carry one, and they are twins:

- **`scan_workspace_inner`** — the scanner's directory walk. When it hits a file it
  cannot read, it does `let Ok(content) = read_to_string(..) else { continue };`
  and proceeds. An unreadable file (permissions, non-UTF-8) is silently skipped, so
  the scan reports a **clean result over an incomplete corpus.** The trigger, as
  written in the source, names exactly this: *an unreadable file lowers coverage
  without lowering the all-clear verdict — 'reported clean' conflates 'found
  nothing' with 'could not look'. No counter, no surfaced skip.*

- **`collect_function_index`** — the auditor's directory walk. Same shape: read a
  file or `continue`, parse it or skip it, building the witness-function index over
  whatever it managed to read. A later "witness function not found" cannot be told
  apart from "the file holding it was unreadable." An **incomplete index presented
  as a complete one.**

These are not synthetic fixtures. They are two real functions in antigen's shipped
infrastructure, each marked because someone reading the code felt the wrongness and
wrote it down. And they are instances of a class antigen *itself* exists to catch:
silent failure — an incomplete result presented as whole. The tool's own infra
carries the disease the tool was built to name, and the tool's own author knew it,
and said so in a `#[dread]` mark instead of pretending the code was fine.

That honesty is the corpus. The learning core's job is to turn two felt-but-unnamed
wounds into a single drafted fingerprint that binds both.

---

## Catch the strange loop: a fingerprint anti-unified from antigen's own unease

Here is the loop closing. The proof re-acquires the two twins' syntax trees from
the committed source — antigen's own marks ride its own source as the corpus —
anti-unifies them into a draft, and confirms the draft is real.

It binds both twins. That much is guaranteed by construction: extraction and
matching walk the syntax the same way, so a draft generalized from a member binds
that member. The sharper question is whether the draft is *non-degenerate* — whether
it captured the shared silent-skip shape, or collapsed to a shapeless "any
function." It captured the shape: both twins call `read_to_string`, so the draft
carries `body_calls("read_to_string")` as a conjunct — the structural signature of
the read-or-skip pattern, and the proof asserts exactly that conjunct is present.

But be precise about what the draft is, because the precise version is the
load-bearing one. The two twins are *concretely* similar walks — both build a
`WalkDir`, filter entries, read a file, parse it — so their intersection is wide:
the draft is `body_calls("read_to_string")` AND a score of other calls the two
walks happen to share, and *no* `any_of` (a two-member cluster where neither twin
carries a discriminating signal the other lacks collapses to its shared core, by
construction). That is a **broad** draft, not a tight one. It is non-degenerate —
it is not "any function" — but it is wider than the single silent-skip signature,
and a broad conjunction can over-bind clean code that happens to share the same
walk vocabulary. This is not a flaw in the demonstration; it is the demonstration.
A broad draft is exactly the case the next section exists for: the generator
reduces autoimmunity where it can, and the gate catches the residue where it
can't.

Then the safety-tangle. The draft is promoted *only* through the gate, against a
clean directory-walk sibling — a function that walks files but propagates its read
error with `?` instead of swallowing it with `else { continue }`. The clean sibling
is the anti-correlated safe case: same domain, opposite behavior. If the promoted
draft ever bound that clean sibling, the gate would have been bypassed and
autoimmunity shipped. It does not. Either the draft promotes and spares the clean
walk (the gate verified it), or the generalization reached the clean sibling and
the gate refused to promote at all — and a refusal is also safe, because a refusal
ships nothing. **The one outcome that cannot happen is a promoted draft that flags
clean code.** That is the line the gate exists to hold, proven on antigen's own
source.

Read what just happened in one sentence: **antigen felt two of its own wounds,
generalized a failure-fingerprint out of them, and could not promote that
fingerprint until it proved the fingerprint would not turn on antigen's own healthy
code.** The tool that makes the memory of failure explicit learned a new
failure-shape from itself — and the same self-tolerance discipline that protects a
user's clean code protected antigen's own.

---

## The honest seam: where the proof found its own limit

The proof is not a clean triumph, and the place it isn't is the most trustworthy
part of it.

Antigen clusters marked sites by a `shape_digest` — a structural hash — before
proposing. The two felt twins do **not** share an exact shape digest. They are
abstractly similar (both are silent-skip directory walks) but concretely
heterogeneous: different visitor constructions, different return shapes. The
clustering key is computed over the exact body shape, and the exact bodies differ.
So the digest *under-clusters* relative to what the anti-unifier can actually
generalize: the generalizer abstracts away body differences that the digest keeps.

This means the automated clustering would not, on its own, have grouped these two
twins — the proof acquires them by name and hands them to the generalizer directly.
That is a real seam: the clustering heuristic is stricter than the generalizer it
feeds. It is not hidden. It is written into the test that carries this proof, flagged
as a clustering-recall note, and the under-clustering does not weaken the
demonstration — the anti-unifier provably generalizes the two sites; the clustering
key is simply a coarser instrument that would have missed the pairing. The clean
≥2-member clustering proof rides a separate, shape-homogeneous corpus; this proof
rides the *felt* corpus and reports exactly what the felt corpus can and cannot
show.

That is the discipline the whole learning core is built on, applied to its own
demonstration: **say what you proved, and say where the proof stops.** A drafted
fingerprint proves it binds its cluster and spares the supplied clean corpus. It
does *not* prove it names a real disease, or that it spares all clean code
everywhere, or that the cluster is a true family. Those stay with the human or the
incident that ratifies the draft. The proof of the proposer holds itself to the same
standard it holds every draft to — and when it reached the limit of what the felt
corpus could show, it wrote the limit down instead of rounding up.

---

## Why this is the persuasive thing, not the feature

A learning system is exactly the kind of thing you should distrust by default,
because the failure mode is invisible: a generalizer that over-fits ships false
positives that look like diligence. The question to ask of any such system is not
"does it generalize?" — they all do — but "what stops it from generalizing onto
clean code, and can you watch that stop *work*?"

Antigen's answer is structural and it is checkable. The brake is the self-tolerance
gate; the gate is the only door to a promotable fingerprint; and the proof that the
door holds was run on antigen's own source, against antigen's own clean code, born
from antigen's own felt wounds. No competitor can stage this, because you cannot
fake a `#[dread]` mark written months before the generalizer existed, sitting in a
git history, on a function that really does swallow its errors.

A structural-memory tool earns the right to *propose* a failure-class the day it can
generalize one from its own unease and refuse to promote it until it has proven it
will not attack itself. This file is the record that antigen can.

---

## Trace it yourself

Every claim here is checkable against the committed source. The proof is a test, and
the test names the story:

```sh
# The dogfood proof: antigen anti-unifies a draft from its own felt twins,
# promoted only through the self-tolerance gate. Three tests, all green:
cargo test --test learn_dogfood_propose -p antigen

#   the_p0b_marks_exist_and_are_surfaced_by_antigens_own_scan
#       — the felt corpus is non-empty (the dread marks are real, not theater)
#   propose_anti_unifies_antigens_own_felt_twins_into_a_real_draft
#       — the draft binds both twins AND carries read_to_string (non-degenerate)
#   propose_promotes_the_felt_draft_only_through_b_sparing_the_clean_sibling
#       — a promoted draft can never bind the clean walk sibling

# The two felt wounds in antigen's own source — read the #[dread] triggers:
git show HEAD:antigen/src/scan/walk.rs        # scan_workspace_inner
git show HEAD:antigen/src/audit/immunity.rs   # collect_function_index

# The generator and its safety gate (newcomer-readable; the doc-comments
# carry the C ══ B co-ship rule and the empty-corpus refusal):
git show HEAD:antigen/src/learn/propose.rs
git show HEAD:antigen/src/learn/self_tolerance.rs

# The synthetic safety gate — the ≥2-member clustering proof and the
# autoimmune-draft rejections on a shape-homogeneous corpus:
cargo test --test autoimmunity_safety_gate -p antigen
```

## See also

- [`the-self-catch.md`](the-self-catch.md) — the companion war-story: antigen
  turning its machinery on its own *existing* code and catching its own
  over-claims. This file is the sequel — antigen *generating* a new fingerprint
  from its own unease, under the same self-tolerance discipline.
- [`../the-felt-arc.md`](../the-felt-arc.md) — the same two wounds and the same
  loop, told from the *inside*: what it's like, beat by beat, to mark a worry, watch
  the draft form, and watch the gate spare clean code. This file is the record; that
  one is the experience.
- [`../the-keystone-explained.md`](../the-keystone-explained.md) — *why* the loop is
  trustworthy, stripped to first principles: why the generator must route through the
  gate, and exactly where the guarantees stop.
- [`../decisions.md`](../decisions.md) — ADR-044 (observe-don't-declare: a draft is
  a hypothesis, never an asserted class) and ADR-045 (the generator-and-gate
  co-ship: never promote without the gate green).
- [`../fingerprint-grammar.md`](../fingerprint-grammar.md) — `any_of` and
  `body_calls`: the leaves the anti-unifier builds a draft out of.
- [`../stdlib-families.md`](../stdlib-families.md) — the hand-written failure-class
  catalog the learning core is built to *extend* with drafted candidates.

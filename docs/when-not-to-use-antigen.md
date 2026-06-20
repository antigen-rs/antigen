# When *Not* to Use antigen

> The most useful thing a tool can tell you is where it stops helping. This page is
> antigen's, written plainly — not to disarm a critic, but because a reader deciding
> whether to adopt deserves the boundary as clearly as the capability.

antigen is a compile-time immune system for a codebase: you describe the *shape* of a
failure once, and antigen checks that every place that shape could appear is defended by a
live witness. That's a real and narrow thing. It is not a linter, not a type system, not a
test framework, and not an AI that fixes your bugs. Below are the situations where reaching
for antigen is the wrong move — and, where there is one, the thing to reach for instead.

If you read only one section, read the last one: **the honest edges of v0.6 specifically.**

---

## Don't use it for a one-off bug

antigen's unit of work is a **failure-*class*** — a pattern that can recur at many sites,
in your code or in code not yet written. Describing a class costs more than fixing a single
bug, and it only pays back when the shape actually repeats.

If a bug appears once, has an obvious local cause, and won't generalize — fix it and write
a normal test. The break-even is recurrence. A good signal you've crossed it: you've fixed
"the same kind of thing" two or three times in different files, and you can say out loud
what the shape is. That sentence is the fingerprint. Before that, antigen is overhead with
no class to amortize it against.

## Don't use it to find unknown bugs

antigen does not discover failures you haven't characterized. It is not a fuzzer, a static
analyzer hunting for undefined behavior, or a heuristic bug-finder. It checks that a shape
*you already named* is defended everywhere it could occur. The recognition comes from you
(or from a learned draft *you ratify*); antigen supplies the *everywhere*, not the *what*.

If your question is "what's wrong with this code that I don't know about?", that's a
different tool — a fuzzer, a sanitizer, a static analyzer. antigen answers "is this *known*
shape defended at every site?" Bring it a named pathogen; it won't invent one for you.

## Don't expect it to enforce semantic correctness

A fingerprint is a **structural** pattern — it matches the *syntax* a failure-class tends
to take. It cannot know what your code *means*. It can tell that a `Drop` impl unwraps a
cleanup result; it cannot tell whether that unwrap is genuinely dangerous in your specific
context. Whether a matched site is a real instance of the failure is a semantic judgment.

This boundary is not incidental — it's a theorem. Whether a structural pattern names a
*real* failure-class is the kind of question Rice's theorem puts permanently out of an
algorithm's reach. That's exactly why antigen's learning core **routes a proposed class to
a human to ratify** rather than promoting it itself. If you need a tool to *decide* semantic
correctness with no human in the loop, no structural matcher can do that, and antigen
doesn't claim to.

## Don't use the learning core expecting it to immunize your codebase by itself

This is the misread most worth heading off. antigen v0.6 can *learn* the shape of a new
failure-family: given a cluster of marked sites, it anti-unifies them into a draft and
screens that draft against clean code. But what it does with the draft is **route it to a
human** — it renders a ratifiable suggestion (observe-don't-declare), never an
auto-`#[presents]` or an auto-named class, and it leaves your source tree byte-unchanged.
The machine supplies the syntactic half; a human ratifies the semantic half. (Run
`cargo antigen propose --help` and the description says exactly this.)

The honest sentence is *"antigen anti-unifies a draft and routes it to a human to
ratify."* It is never *"antigen immunized itself."* If you wanted a system that curates its
own immune memory end-to-end with no person in the loop — that loop is deliberately not
closed in this release. The human is in it on purpose. Don't adopt v0.6 expecting autonomy
it intentionally withholds.

## Don't read the affinity score as a probability

When the learning core scores a draft, it reports a 2-vector — `(recall, precision)` — and
deliberately gives it *no total order*. The two numbers trade off, and a single confident
probability would hide which side of the trade-off you're on. If your workflow needs a
calibrated "this is 87% likely to be a real bug" number, antigen doesn't compute one yet,
and the 2-vector is the honest placeholder for the probability it hasn't earned. Treating
it as a probability is reading a number the tool took care not to claim.

## Don't reach for the drift detector and expect it to fire today

antigen v0.6 ships a drift detector that, at the scale any real project runs it at right
now, **returns `UnderPowered` on essentially every class** — "I can't statistically see
drift yet, and here's exactly when I'll be able to." That is the correct, designed
behavior: a class has matured only a handful of times, and a change that small is
mathematically undetectable. But if you're evaluating antigen *for its drift detection
specifically*, understand that you are adopting an organ that is honestly blind at small
scale and becomes useful as trajectories lengthen — not a detector that lights up on day
one. (Why this is a feature, not a stub: [drift-detection and the moral
center](drift-detection-and-the-moral-center.md).)

## Don't use it where the cost of a false alarm is a human relationship

antigen flags sites; a flagged site that *isn't* a real failure is a false positive, and
false positives have a social cost — they teach a team to ignore the tool. antigen is built
to make this rare (the self-tolerance gate refuses to promote a draft that flags clean
code; the precision axis is reported, not hidden). But "rare" is not "never," and the
narrower and more structural your failure-class, the lower the false-alarm rate. If you'd
deploy antigen as a hard CI gate on a broad, fuzzy, judgment-heavy class across a team that
will revolt at the first wrong flag — start it as a non-blocking advisory instead, on a
class tight enough to earn trust, and tighten the gate as the precision proves out.

---

## Where antigen *is* the right tool

For balance — the cases it's built for:

- A failure-class that **recurs** and has a recognizable **structural shape** (a
  cleanup-in-`Drop` that can panic; a deserialization site missing `deny_unknown_fields`;
  an enum match that won't catch a future variant).
- A class you want defended **at every site it could appear**, including code not yet
  written — antigen's `scan` is the *everywhere*.
- A defense you want to **prove with a witness**, not assert — `#[defended_by(X)]` makes a
  test the evidence, not a comment.
- A class whose **lineage** matters — a family of related shapes that should inherit a
  defense.

---

## The honest edges of v0.6, in one place

Stated as present-fact, so there's no surprise after adoption:

- **The new organs are a library, not a wired command.** The drift detector, the curator,
  and the classifier are tested `antigen::learn::*` APIs. There is no `cargo antigen` verb
  that runs sense → classify → act end-to-end. The one wired learning verb is `propose`.
- **`propose` routes, it does not promote.** A proposed class is a suggestion for a human
  to ratify. The source tree is left byte-unchanged.
- **The affinity score is a 2-vector, not a probability.** No total order, by design.
- **The drift detector fires zero at small scale, by design.** `UnderPowered` is the
  default; it tells you when it will be able to see.
- **The moral center's guarantee is scoped.** "Never auto-forgets a live defense on a blind
  or garbage signal" is the structural invariant — not "always curates correctly."

None of these is a hedge. Each is the line where antigen's knowing stops, drawn where it
actually is. A tool that only ever tells you what it *can* do is over-claiming by omission;
this page is the other half.

---

## See also

- [the maturing organism](the-maturing-organism.md) — what v0.6 *is*, in full
- [scope](scope.md) — the project's scope boundaries
- [getting started](getting-started.md) — if this page didn't talk you out of it
- [drift-detection and the moral center](drift-detection-and-the-moral-center.md) — why the
  drift detector is honest rather than broken

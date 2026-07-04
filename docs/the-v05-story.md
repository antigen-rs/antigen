# The v0.5 story — a tool that learns where its own knowing ends

> [`the-felt-arc.md`](the-felt-arc.md) walks the learning core one beat at a time —
> what it *feels* like to mark a worry, watch a draft form, watch a gate hold. This
> page is the *story around* those beats: what v0.5 actually ships, why the loop
> stops at a human instead of closing on itself, and why that stop is the whole
> point. If the felt-arc is the close-up, this is the wide shot.
>
> New here? Read [`reading-a-verdict.md`](reading-a-verdict.md) for the output, then
> the felt-arc for the experience. Come back when you want the *why*.

---

## What v0.5 is

For four versions antigen could *remember* failure-classes — you declared them, it
scanned for them, it audited the defenses. What it could not do was **learn a new
one.** The vocabulary of worries was something a person wrote down; antigen read it
back. v0.5 gives antigen the organ it was missing: a **learning core** that, given a
cluster of marked worries, can *draft a candidate failure-class of its own*.

The keystone of v0.5 is one new verb:

```sh
cargo antigen propose --cluster-root <marked-sites> --clean-root <code-you-vouch-for>
```

`propose` reads a **cluster** of marked failure-sites, **anti-unifies** them into a
candidate fingerprint — the structural shape they share that clean code does not —
and routes that draft through the **self-tolerance gate**. Then it does the most
important thing in the whole release: it **renders the outcome and stops.** It writes
nothing to your source tree. The machine drafts the syntactic half of a
failure-class; a human ratifies the semantic half. *Observe, don't declare.*

That stop is not a missing feature. It is the thesis. The rest of this page is about
why.

---

## The three outcomes (what you'll actually see)

`propose` has three first-class outcomes you'll meet most often, and **all three are
the gate working** — none is an error. You can run every one of them today from the
repo root against the demo in [`examples/propose-demo/`](../examples/propose-demo/).

### 1. route-to-human — *the gate is honest*

The cluster anti-unifies into a real draft, but the gate **cannot certify the draft
generalizes** against your clean corpus — no clean sibling is one discriminating
constraint away from binding it (no *near-miss*). So the gate routes the candidate to
a human ratifier rather than promote it:

```sh
cargo run --bin cargo-antigen -- antigen propose \
    --cluster-root examples/propose-demo/cluster \
    --clean-root   examples/propose-demo/clean
```
```
== drafted a candidate — routed to a human ratifier ==

Antigen anti-unified a draft from your `dread` marks, but the B-gate cannot
certify it GENERALIZES against your clean corpus (no near-miss: no clean
sibling is one discriminating constraint from binding the draft). So it
routes the candidate to a HUMAN ratifier rather than promote it.

This is the gate being honest — refusing to certify a generalization it
cannot witness is the trust-floor, not a failure. (A promote fires when the
cluster has discriminating diversity AND your corpus holds a near-miss
sibling.)
```

Read the gate's own words: *refusing to certify a generalization it cannot witness is
the trust-floor, not a failure.* This is the gate declining to vouch for something it
can't see proof of, and it is the gate at its most trustworthy.

### 2. promote — *a ratifiable suggestion, never an applied mark*

When the clean corpus holds a **near-miss** — a sibling that shares the skeleton but
does the right thing, one discriminating constraint away — the gate *can* witness the
generalization. It promotes the draft to a **ratifiable suggestion**:

```sh
cargo run --bin cargo-antigen -- antigen propose \
    --cluster-root examples/propose-demo/cluster \
    --clean-root   examples/propose-demo/clean-near-miss
```
```
== candidate failure-class fingerprint (ratifiable suggestion) ==

This is a SUGGESTION drafted from your `dread` marks and gated against your
clean corpus — inspect it and ratify by hand. It is NOT an audited verdict,
NOT an auto-`#[presents]`, and NOT a named failure-class.
```

Even here — the *success* path — antigen stops at a suggestion. The JSON render
carries `"promoted": false` **always**: a candidate is something you ratify, never
something the tool applies. The machine drafts; you name.

### The refusal underneath: self-tolerance, layered

Route-to-human is the *outer* honesty — "I can't certify this generalizes." Beneath
it sits the gate's deepest safety check, and it's the one the whole learning core is
built around: **the gate refuses a draft that binds your clean code.** If
anti-unify produces a draft so broad it would flag a known-good site in your clean
corpus, the gate names the offending item and refuses — *"the draft matches
clean-corpus item #i; promoting it would flag known-good code."* That is autoimmunity
caught in the act: the generator's own over-broad output is the false positive the
governor exists to stop, and stopping it is the gate's signature move.

The checks run in a fixed order — empty-corpus refusal, then a bare-structural
over-binder refusal, then the near-miss generalization check (which produces
route-to-human), and finally the bind-clean refusal. The ordering means
route-to-human *shadows* the bind-clean refusal: a corpus with no near-miss is turned
away for honesty before the safety check even runs, so the binds-clean refusal only
surfaces on a corpus rich enough to reach it. Either way, **there is no path to a
promoted draft that the spare-clean check didn't bless.** Both outcomes are safe: a
promotion means the gate verified the draft spares clean; a refusal means it caught an
over-binder and pruned it. There is no third door where a clean-flagging draft slips
through.

### no cluster — *the honest current frontier*

Run `propose` against antigen's own source, or your own marked tree, and you will
usually see this:

```
no `dread` cluster found under antigen/src — propose needs ≥2 marked sites sharing a
structural shape to anti-unify (found 0). Antigen's own marks are singletons in
shape-space today; auto-clustering heterogeneous marks is the v0.6 abstract-recall
frontier.
```

This is not a bug and not a dead end — it is the **honest edge of v0.5.** Today
antigen clusters by *exact* structural shape, so two worries cluster only if they
rhyme exactly. Real marks are usually singletons. Auto-clustering marks that rhyme
*loosely* — so real-world singletons find their family — is the v0.6 abstract-recall
frontier. The demo's twins are byte-identical by construction precisely so you can
*see* the interesting behavior today, before that frontier lands.

And the exactness is a *choice*, not a shortfall — the same restraint as everywhere
else here. Loose clustering buys recall, but it risks **over-merging two genuinely
distinct worries into one wrong class** — and a wrong merged class is autoimmune,
exactly the failure the whole gate exists to prevent. Under-clustering ("no cluster")
is recoverable: you mark more, or wait for v0.6. Over-clustering ships a class that
flags the wrong code. So antigen holds the tighter, safer floor and defers loose
clustering to v0.6 — not because it's unfinished, but because *merging distinct things
wrongly is a failure-class of its own*, and the floor is the response to one antigen
already paid for.

---

## Why it routes to a human — and why that's the design

Here is the question a careful stranger asks: *if the tool can draft a failure-class,
why doesn't it just keep it? Why hand it back to me?*

Because **drafting a shape and naming a class are different kinds of problem, and one
of them is not machine-tractable.** Generate a candidate and select against clean
code — that, a machine can do: it's the anti-unify-then-gate loop. But deciding that a
genuinely novel structural shape *is a real, named failure-class worth defending
against* — that is a judgment about meaning, about the world the code lives in. No
amount of structural cleverness closes that gap. (It is the same wall Rice's theorem
draws around deciding non-trivial semantic properties in general: the syntactic half
is decidable; the semantic half is not.)

So antigen does exactly the tractable half and **stops at the naming line.** This is
not antigen falling short. It is antigen *built to the shape of the problem* — the
same boundary the immune system itself draws. An immune cell builds a receptor that
binds a pathogen it has never seen, but it holds no *concept* of that pathogen; the
organism names it, after. Protection runs ahead of naming, in biology and here. The
hardest boundary antigen draws is the one biology already drew, for the same reason.

That is why the v0.5 payoff is **honest, not unfinished.** antigen *anti-unifies a
draft from felt worries and hands it to a person to ratify* — it does **not**
"immunize itself." (On antigen's own three `#[dread]` marks, the keystone today finds
*no cluster at all* — those marks are singletons in shape-space, so the CLI prints the
"no cluster found" line above. You watch the route-to-human and promote outcomes on a
real cluster: the demo's constructed twins, which exist precisely so the behavior is
visible before the v0.6 frontier makes real-world singletons cluster.) The
self-immunization payoff — promoting a defended class from antigen's own felt worries
— is the v0.6 frontier, and it is honestly unfired here. What v0.5 ships is the real,
restrained thing: a tool that knows where its own knowing ends, and routes the rest to
you.

---

## The same restraint, one tier up: how this got built

There is a second story folded into v0.5, and it rhymes with the first so exactly that
it's worth telling plainly — because it's the reason you can trust the first.

The self-tolerance gate refuses to certify a draft against a corpus that contains no
real clean siblings. A vacuous spare-clean — "this draft flags nothing bad because
there's nothing here to flag" — is **autoimmunity wearing a green check.** The gate
would rather promote nothing than promote a class it couldn't witness as safe. It
will not certify safety against *nothing*.

The team that built antigen holds the same discipline about its own work. A builder
who finishes a piece **self-closes** it — a claim, a hypothesis: *"I built this to
design."* But a self-close is never the final word. A *fresh, independent reader* who
never built the thing has to **witness** it before it's certified. A self-witnessed
certification is the same vacuous pass at a higher tier: an unfounded seal wearing a
green check. The team will not certify safety against the builder's own word, exactly
as the gate will not certify safety against an empty corpus. *You cannot certify
safety against a corpus that contains only yourself* — and the rule holds whether the
"corpus" is clean code or independent readers.

And the shape of that discipline is the *same* shape as the gate. A self-close is
itself a kind of *draft* — anti-unified from the builder's own understanding of the
thing. It binds everything the builder thought to test, and it can still **miss the
clean sibling**, the same way a naive anti-unify misses a clean sibling by dropping
the leaf that differs. So an independent reader plays the part the spare-clean corpus
plays for the gate: they run the input the builder's own flow never produces, and if a
self-close over-reached, that's where it surfaces. Independence is the spare-clean
gate for claims. *The discipline that keeps the tool from flagging its own clean code
is the discipline that keeps the team from certifying its own unwitnessed work* — the
same refusal, one tier up.

That is why the rest of this page can be honest without being a victory lap. The
restraint isn't a posture in the docs — it's the load-bearing discipline at every
tier of the thing. The tool stops at the naming line because naming isn't
machine-tractable; the team stops at the self-witness line because a self-witness
isn't independence. Same refusal, same reason, twice.

---

## The register to leave with

v0.5 is the version where antigen learned to *draft*. Not to decide — to draft, and
then to hand the deciding to you. Every outcome the keystone produces is the gate
being honest: route-to-human when it can't witness a generalization, a ratifiable
suggestion when it can, "no cluster" when there's nothing yet to learn from. None of
them writes to your tree. The machine does the bindable, structural, machine-tractable
half, and **stops exactly where a careful clinician stops** — *"I feel something here,
I can't name it, let's investigate"* — instead of guessing a diagnosis.

That is the masterclass v0.5 teaches: **restraint, not cleverness.** A felt worry,
marked honestly. A draft, offered not asserted. A gate that would rather route to a
human than vouch for what it can't see. And, underneath, a tool whose deepest move —
at every tier, in the code and in the team that wrote it — is knowing where its own
knowing ends.

---

## See also

- [`the-felt-arc.md`](the-felt-arc.md) — the four beats, slowed down enough to feel.
- [`the-learning-loop.md`](the-learning-loop.md) — the *circulation*: where propose
  sits in the whole organism (afferent/efferent), and where v0.5 sits in the arc.
- [`examples/propose-demo/`](../examples/propose-demo/) — run the route-to-human and
  promote outcomes yourself.
- [`reading-a-verdict.md`](reading-a-verdict.md) — decode every scan/audit line.
- [`i-scanned-and.md`](i-scanned-and.md) — symptom-indexed troubleshooting.
- The source, if you want to read the gate yourself:
  `antigen/src/learn/propose.rs` + `self_tolerance.rs` (the learner + the gate);
  `cargo-antigen/src/main.rs` (the `propose` verb).

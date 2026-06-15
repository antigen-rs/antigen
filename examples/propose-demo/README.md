# `cargo antigen propose` — a runnable demo

This directory is a **runnable demo** of `cargo antigen propose`, the v0.5
keystone verb. It lets a stranger *see* the learning core's interesting behavior
on a fixture they can run today — without needing antigen's own internals.

`propose` takes a **cluster** of marked failure-sites and an operator-supplied
**clean corpus**, anti-unifies the cluster into a candidate fingerprint, and
routes that draft through the self-tolerance gate (GATE-G). The gate returns one
of three first-class outcomes; this demo shows the two interesting ones:

- **route-to-human** — the draft is safe but the gate cannot certify it
  *generalizes*, so it routes the candidate to a human ratifier (this is the
  gate being honest, not failing);
- **promote** — the corpus holds a *near-miss* sibling, so the gate can witness
  the generalization and renders a ratifiable **suggestion** (a candidate
  fingerprint to ratify by hand — never an auto-applied mark).

> **What this demo is and is not.** These are **constructed fixtures** that prove
> the render *path*. They are NOT a claim that "antigen immunized itself." On
> antigen's *own* `#[dread]` marks, `propose` today routes-to-human (its marks are
> singletons in shape-space — there is no ≥2 cluster to anti-unify yet). The
> self-immunization payoff — promoting a class from antigen's own worries — is the
> v0.6 abstract-recall frontier. What v0.5 ships is honest and real: a tool that
> *anti-unifies a draft from felt marks and routes it to a human ratifier*.

## Layout

```
examples/propose-demo/
├── cluster/src/lib.rs           the DEFECT cluster — two #[dread]-marked twin
│                                directory-walks that swallow a read error
│                                (byte-identical bodies → they cluster)
├── clean/src/lib.rs             a clean corpus of UNRELATED code (no near-miss)
│                                → drives the route-to-human outcome
├── clean-near-miss/src/lib.rs   a clean corpus with a NEAR-MISS sibling (a
│                                ?-propagating walk, one constraint away)
│                                → drives the promote outcome
└── README.md                    this file
```

The `.rs` files are read **syntactically** by the scan (it reads the `#[dread]`
attribute from source text). They do **not** compile and do not need to — there is
no crate to build, no dependency on the macro crate.

## Run it

All commands are run from the **repository root**.

### Case 1 — route-to-human (the honest dogfood payoff-state)

```sh
cargo run --bin cargo-antigen -- antigen propose \
    --cluster-root examples/propose-demo/cluster \
    --clean-root   examples/propose-demo/clean
```

The two twins anti-unify into a real draft, but the `clean/` corpus is unrelated
code — nothing in it is *one discriminating constraint away* from binding the
draft (no near-miss). So the gate cannot certify the draft generalizes, and routes
it to a human. **Real captured output** (exit code `0`):

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

### Case 2 — promote (a ratifiable suggestion)

```sh
cargo run --bin cargo-antigen -- antigen propose \
    --cluster-root examples/propose-demo/cluster \
    --clean-root   examples/propose-demo/clean-near-miss
```

Here the clean corpus holds a **near-miss**: a directory-walk that propagates its
read error with `?` instead of swallowing it. It shares the walk skeleton with the
defect twins but does the right thing with the error — one discriminating
constraint away. With that near-miss present, the gate can witness the
generalization and **promotes**, rendering a ratifiable suggestion. **Real
captured output** (exit code `0`):

```
== candidate failure-class fingerprint (ratifiable suggestion) ==

  fingerprint: Fingerprint { constraints: [Item(Fn), BodyCalls("display"), BodyCalls("flatten"), BodyCalls("new"), BodyCalls("path"), BodyCalls("push"), BodyCalls("read_dir"), BodyCalls("to_string")] }
  score tier:  Imagined

This is a SUGGESTION drafted from your `dread` marks and gated against your
clean corpus — inspect it and ratify by hand. It is NOT an audited verdict,
NOT an auto-`#[presents]`, and NOT a named failure-class. The machine drafted
the syntactic half; you ratify the semantic half (observe-don't-declare).
```

The candidate fingerprint is the shared body-call signal of the twins. It is a
**suggestion**, not a declaration — `propose` writes nothing to the source tree
(observe-don't-declare, ADR-044). You inspect it and ratify by hand.

### Machine-readable output (`--format json`)

Append `--format json` to either invocation for a compact object. Note
`"promoted": false` is **always** present on the promote path: a candidate is a
ratifiable *suggestion*, never an auto-applied promotion.

```json
{
  "fingerprint": "Fingerprint { constraints: [Item(Fn), BodyCalls(\"display\"), ...] }",
  "note": "ratifiable suggestion (observe-don't-declare); inspect + ratify by hand",
  "outcome": "candidate-suggestion",
  "promoted": false,
  "tier": "Imagined"
}
```

## What you'll see on real code today

Run `propose` against antigen's own source (or your own marked tree) and you will
typically see **"no cluster found"** — because real `#[dread]` marks are usually
*singletons in shape-space* (no two share an exact structural shape to
anti-unify):

```
no `dread` cluster found under <root> — propose needs ≥2 marked sites sharing a
structural shape to anti-unify (found 0). Antigen's own marks are singletons in
shape-space today; auto-clustering heterogeneous marks is the v0.6 abstract-recall
frontier.
```

That is the honest current state: v0.5 clusters by **exact** structural shape.
Auto-clustering heterogeneous marks (so real-world singleton marks cluster) is the
v0.6 abstract-recall frontier. This demo's twins are byte-identical *by
construction* so they cluster — which is exactly why a runnable demo is needed to
see the interesting behavior today.

## The contract `propose` upholds

- **The clean corpus is operator-supplied.** Antigen never auto-labels unmarked
  code as clean (`--clean-root` is required; a missing one is a usage error, exit
  `2`). The gate spares against exactly the corpus you vouch for.
- **The CLI is plumbing; the gate is safety.** `propose` passes the cluster and
  corpus straight to the gate; it never pre-validates cleanliness itself. If you
  hand it a contaminated corpus, the *gate* catches it (autoimmune refusal) — the
  safety decision stays in the gate.
- **Observe, don't declare.** Every outcome is a render, never a source edit. A
  `propose` run leaves your tree byte-unchanged.

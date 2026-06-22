# Using antigen as a library (not the CLI)

> Most adopters drive antigen through `cargo antigen scan` / `audit`. But antigen
> is also a plain Rust library, so a build tool, an editor backend, a CI bot, or
> an agent can call the scanner and the Learning-Core directly and get typed
> results instead of parsing console output. This is the reference for that path.
>
> The two public surfaces a non-CLI consumer reaches for are **`antigen::scan`**
> (walk a tree, get a `ScanReport`) and **`antigen::learn`** (the safety-governed
> Learning-Core). For the *why* behind the Learning-Core safety line, see
> [`the-keystone-explained.md`](the-keystone-explained.md); for what it *feels*
> like, [`the-felt-arc.md`](the-felt-arc.md).

---

## Add the dependency

```toml
[dependencies]
antigen = "0.6"     # the scan + learn library surfaces   (check crates.io for the latest)
# The Learning-Core returns `antigen_fingerprint::Fingerprint` values. If you
# want to *name* that type (store it, match on it, print it), add the fingerprint
# crate too — antigen does not currently re-export it:
antigen-fingerprint = "0.6"   # check crates.io for the latest
syn = "2"                    # you supply `syn::Item`s to the learner
```

---

## `antigen::scan` — walk a tree, get a typed report

### Plain scan

```rust
use std::path::Path;
use antigen::scan::scan_workspace;

let report = scan_workspace(Path::new("."), None)?;

// Everything the CLI prints is a field on the report:
println!("{} antigen declarations", report.antigens.len());
println!("{} presentation records", report.presentations.len());
# Ok::<(), std::io::Error>(())
```

`ScanReport` is the same structure the CLI renders — `antigens`, `presentations`,
`immunities`, `tolerances`, `lineage_edges`, `parse_failures`, and
`marked_unknowns` (the `#[dread]` / `#[aura]` / `#[red_flag]` plane, which the
*library* surfaces but the CLI console does not render). See
[`output-formats.md`](output-formats.md) for the full field reference.

### Scan with the bundled catalog (close the zero-hits cliff)

```rust
use std::path::Path;
use antigen::scan::scan_workspace_bundled_catalog;

// auto_detect = true  → inject the bundled stdlib catalog ONLY when the tree has
//                       zero in-tree antigens (the consumer-crate auto-detect path)
// auto_detect = false → ALWAYS inject (augment-mode, the explicit --bundled-catalog path)
let report = scan_workspace_bundled_catalog(Path::new("."), None, true)?;

// Each catalog hit is a fingerprint match in report.presentations[].
let matches = report
    .presentations
    .iter()
    .filter(|p| p.match_kind == antigen::scan::MatchKind::FingerprintMatch)
    .count();
println!("{matches} fingerprint-match candidate(s)");
# Ok::<(), std::io::Error>(())
```

A catalog hit is a **scan-fact** (`MatchKind::FingerprintMatch`) — it is *not* an
audited defense verdict, and it structurally cannot become one (a separate
sum-type). Read it as "this structure matches a known failure-class," never "this
is broken."

---

## `antigen::learn` — the Learning-Core (cluster → propose → gate → promote/route)

The Learning-Core is a **library API**: you feed it ASTs and it gives you back a
gate outcome. This page documents that library. The CLI verb that wraps it,
**`cargo antigen propose`**, is in [`cli-reference.md`](cli-reference.md#propose);
reach for the library directly when you are building your own tooling on top of the
learner.

The public functions (call paths are from the crate root):

| Call path | Returns | What it does |
|---|---|---|
| `antigen::learn::propose::anti_unify(cluster)` | `Option<Fingerprint>` | generalize a cluster into a **raw hypothesis** draft — inspection only, never promotable |
| `antigen::learn::propose::propose(cluster, clean_corpus)` | `Result<PromotedDraft, ProposeOutcome>` | `anti_unify` **then** route through the self-tolerance gate — the **only path to a `PromotedDraft`** |
| `antigen::learn::self_tolerance::evaluate(draft, clean_corpus)` | `ToleranceVerdict` | the spare-clean verdict for one draft (`Spared` / `BindsCleanItem`) |
| `antigen::learn::self_tolerance::spare_clean(draft, clean_corpus)` | `bool` | shorthand for `evaluate(...).is_safe()` |
| `antigen::learn::self_tolerance::promote_if_safe(draft, clean_corpus)` | `Result<PromotedDraft, ToleranceVerdict>` | promote a draft *iff* the gate passes (refuses an empty corpus) |

### The safety line, in code

The promotable verb is `propose()` — `anti_unify()` followed by the gate. Possession
of the `PromotedDraft` it returns is the proof the gate passed; there is no other way
to construct one.

> **Name-collision gotcha (read this first).** The promotable function `propose`
> lives in a module *also* named `propose`. So `use antigen::learn::propose;`
> imports the **module**, and the call is `propose::propose(cluster,
> clean_corpus)`. (Import the function directly with
> `use antigen::learn::propose::propose;` if you'd rather write a bare
> `propose(...)`.) Also: a drafted `Fingerprint` implements `Debug`, **not**
> `Display` — print it with `{:?}`.

```rust
use antigen::learn::propose::{self, ProposeOutcome};
use antigen::learn::self_tolerance::ToleranceVerdict;

// `cluster`     — the marked sites you want to generalize (≥1 syn::Item)
// `clean_corpus`— known-good siblings the draft MUST NOT bind (≥1 syn::Item)
# fn run(cluster: &[syn::Item], clean_corpus: &[syn::Item]) {
match propose::propose(cluster, clean_corpus) {
    Ok(token) => {
        // A PromotedDraft: it generalized the cluster, carries a discriminating
        // signal, was exercised by a near-miss, AND spared every clean sibling.
        // It is still a SUGGESTION — a candidate to ratify, never an auto-asserted
        // `#[presents]`. A human (or an incident) names the class.
        println!("suggestion (tier {:?}): {:?}", token.tier(), token.fingerprint());
    }
    Err(ProposeOutcome::Rejected(ToleranceVerdict::NotCorpusWitnessable)) => {
        // ROUTE-TO-HUMAN: the draft is safe but the corpus holds no near-miss, so
        // the gate cannot certify it generalizes. First-class, not an error.
        println!("safe, but routed to a human ratifier (no near-miss in the corpus)");
    }
    Err(ProposeOutcome::Rejected(ToleranceVerdict::BindsCleanItem { clean_index })) => {
        // AUTOIMMUNE: the draft matched a clean-corpus item (or is bare-structural
        // and over-general). Refused, so it can't flag known-good code.
        println!("refused — the draft binds clean item {clean_index:?}");
    }
    Err(other) => {
        // Degenerate (shares only shape), EmptyCluster, NoSharedSkeleton, or the
        // defensively-handled Spared. Nothing safe to generalize.
        println!("no candidate: {other:?}");
    }
}
# }
```

The two danger rules, both enforced by the types:

1. **`anti_unify()` is inspection-only.** Its output is a raw, ungoverned draft.
   To promote, you must go through `propose()` (or call `promote_if_safe()`
   yourself) — never ship `anti_unify()`'s output directly.
2. **An empty `clean_corpus` cannot promote.** `promote_if_safe` refuses an empty
   corpus: a vacuous "it spared everything" is autoimmunity with a green check.
   Supply real clean siblings.

### Inspecting the gate verdict directly

If you want the *reason* a draft was rejected (e.g. to surface it to a user), use
`evaluate`:

```rust
use antigen::learn::{propose, self_tolerance};
use antigen::learn::self_tolerance::ToleranceVerdict;

# fn run(cluster: &[syn::Item], clean_corpus: &[syn::Item]) {
// anti_unify is the RAW hypothesis (inspection-only); evaluate() is the
// spare-clean half of the gate (the full three-check gate is promote_if_safe).
if let Some(draft) = propose::anti_unify(cluster) {
    match self_tolerance::evaluate(&draft, clean_corpus) {
        ToleranceVerdict::Spared => {
            // safe against this corpus — promote_if_safe would mint a PromotedDraft
            // (if the draft also carries a discriminating signal and a near-miss).
        }
        ToleranceVerdict::BindsCleanItem { clean_index } => {
            // the draft over-binds: it matched clean_corpus[clean_index].
            // Promoting it would flag clean code (autoimmunity) — B rejects it.
            eprintln!("draft binds clean item at index {clean_index:?}");
        }
        ToleranceVerdict::NotCorpusWitnessable => {
            // The route-to-human verdict (safe, but B can't certify the
            // generalization). evaluate() itself never returns this — it is
            // promote_if_safe's near-miss verdict — but ToleranceVerdict is one
            // sealed enum, so the match names all three.
        }
    }
}
# }
```

### Where the cluster comes from

You supply the cluster as `syn::Item`s — typically by parsing source and
selecting the marked sites. The pattern (from antigen's own dogfood proof):

```rust
use std::fs;

fn item_named_fn(file: &str, fn_name: &str) -> syn::Item {
    let src = fs::read_to_string(file).expect("read");
    let parsed = syn::parse_file(&src).expect("parse");
    parsed
        .items
        .into_iter()
        .find(|it| matches!(it, syn::Item::Fn(f) if f.sig.ident == fn_name))
        .expect("found")
}
# let _ = item_named_fn;
```

Antigen does exactly this on its **own** two `#[dread]`-marked silent-skip twins —
it clusters its own honest self-doubt, anti-unifies a draft, routes it through the
gate, and — because the draft is safe but the corpus holds no near-miss — **routes
it to a human ratifier** rather than promoting it. The plumbing closes (anti-unify →
gate → legible outcome); naming a class for itself is the v0.6 frontier. That
end-to-end run (with the test output on the page) is in
[`the-felt-arc.md`](the-felt-arc.md).

---

## The fingerprint serializer — `to_antigen_attr` (ADR-063)

`antigen-fingerprint` ships the `Fingerprint → DSL` serializer, the exact inverse of
`Fingerprint::parse`. For every parser-producible fingerprint,
`Fingerprint::parse(serialize(fp)) == fp`. This is what lets tooling generate a fingerprint
and emit an `#[antigen(fingerprint = r#"…"#)]` attribute a developer can paste verbatim.

```rust
use antigen_fingerprint::{Fingerprint, to_antigen_attr};

let fp = Fingerprint::parse(r#"body_calls("from_reader")"#).unwrap();
let attr = to_antigen_attr(&fp);
assert_eq!(attr, r##"#[antigen(fingerprint = r#"body_calls("from_reader")"#)]"##);
```

`to_antigen_attr` wraps the inner grammar in the full attribute; the `Fingerprint`'s
`Display` impl emits just the inner constraint list. See
[`fingerprint-grammar.md`](fingerprint-grammar.md) for the grammar itself.

---

## Drift-detection: `DriftVerdict` (ADR-065)

`antigen::learn::adwin::detect` is the batch drift-detector over a class's
affinity-trajectory — the v0.6 maturing-organism's loud-class sensor (see
[`concepts.md`](concepts.md#drift-detection-the-maturing-organism)). It is a **pure
function**: feed it a `&[Affinity]` and a confidence `δ`, get back a `DriftVerdict`.

| Call path | Returns | What it does |
|---|---|---|
| `antigen::learn::adwin::detect(trajectory, delta)` | `DriftVerdict` | per-axis change-point detection over the affinity-trajectory; `delta` is the false-positive bound (use `DEFAULT_DELTA` = 0.05) |

The verdict is a three-way sum-type — and the third arm is the point of the organ:

```rust
use antigen::learn::adwin::{detect, DriftVerdict, DEFAULT_DELTA};
use antigen::learn::affinity::Affinity;

// A short trajectory — a class that has matured only a handful of times.
let short = vec![
    Affinity::new(0.90, 0.95),
    Affinity::new(0.88, 0.96),
    Affinity::new(0.91, 0.94),
    Affinity::new(0.89, 0.95),
];
// Below the statistical power threshold: the detector says so rather than guessing.
assert!(matches!(detect(&short, DEFAULT_DELTA), DriftVerdict::UnderPowered { .. }));

// A long trajectory with a real recall collapse partway through.
let mut drifting: Vec<Affinity> = (0..200).map(|_| Affinity::new(0.90, 0.95)).collect();
drifting.extend((0..200).map(|_| Affinity::new(0.40, 0.95)));
match detect(&drifting, DEFAULT_DELTA) {
    DriftVerdict::Drift { cut_index, axis, .. } => {
        // The change-point and which axis moved (Recall here — the evasion/obsolescence signal).
        assert_eq!(cut_index, 200);
        println!("drift at {cut_index} on {axis:?}");
    }
    other => panic!("expected Drift, got {other:?}"),
}
```

Running that example prints `drift at 200 on Recall`, and the short trajectory is
`UnderPowered { eps_cut: 3.59…, max_observable: 1.0 }`.

The three verdicts:

- **`Drift { cut_index, axis, observed_diff, eps_cut }`** — a change-point cleared the
  δ-bounded threshold. `axis` (Recall / Precision) is decision-relevant: a recall-drop is the
  evasion/obsolescence signal; a precision-drop is the autoimmune over-broadening signal.
- **`NoDrift { tightest_margin }`** — the detector looked and saw no clearing change-point;
  `tightest_margin` is how close the closest split came.
- **`UnderPowered { eps_cut, max_observable }`** — **structurally blind**: `eps_cut ≥
  max_observable`, so *no* split could clear the maximum possible signal. This is the
  **default** at antigen's current scale (classes have matured n≈4–8 times) and is the
  organ's whole v0.6 honesty: it reports "I cannot yet see drift for this class" rather than
  masquerading as `NoDrift`. The same code fires correctly once trajectories lengthen.

> **Honest-scope (ADR-065 Amendment 2).** The detector's FULL (variance-aware) regime is
> gated behind a sample-count of 30 and is not reached at v0.6 scale; the rigorous floor
> bound governs every reachable case. A documented doc-vs-code scope note on the FULL bound's
> cutpoint-set (ADR-065 Amendment 2) does not
> affect the verdict for any trajectory a v0.6-scale caller can produce.

---

## Honest scope (what this library does and does not promise)

- **A drafted `Fingerprint` is a hypothesis, not a named class.** `propose()`
  proves a draft *binds a shape and spares the supplied clean corpus* — it does
  **not** name a failure-class. Ratification (turning a draft into a named
  `#[antigen]`) is a human/incident act.
- **The gate is corpus-bounded.** `spare_clean` is only as strong as the
  `clean_corpus` you supply — a larger, more representative corpus is a stronger
  gate. Supply a real corpus the draft *can* be tested against.
- **The "promote = `propose()` only" boundary is a type, not a convention.**
  `PromotedDraft` has no public constructor and no `Deserialize`, so the only way
  to hold one is to pass the gate — it cannot be forged from a hand-written
  `Fingerprint` or JSON. The CLI verb `cargo antigen propose` is the production
  caller; on antigen's own marks it routes the draft to a human ratifier rather
  than promoting it.

See [`the-keystone-explained.md`](the-keystone-explained.md) for the
first-principles account of *why* the safety line holds.

---

## See also

- [`output-formats.md`](output-formats.md) — `ScanReport` field reference + the
  `--bundled-catalog` / `--message-format json` CLI surfaces.
- [`the-felt-arc.md`](the-felt-arc.md) — the Learning-Core run on antigen's own
  marks, narrated, with the test green on the page.
- [`the-keystone-explained.md`](the-keystone-explained.md) — why C routes through
  B, from first principles.
- [`concepts.md`](concepts.md) — where the Learning-Core sits in antigen's
  architecture.

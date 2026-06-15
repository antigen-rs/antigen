# The felt arc — what it's like to use antigen

> The other docs decode the *output*. This one is about the *experience*: what it
> feels like to mark a worry you can't yet name, watch antigen draft a failure-class
> from it, and watch it refuse to flag clean code. Every step here runs on antigen's
> **own** source — not a fixture. The strange loop, slowed to four beats you can feel
> one at a time.
>
> New to the output? Read [`reading-a-verdict.md`](reading-a-verdict.md) first.
> Hit a surprise? [`i-scanned-and.md`](i-scanned-and.md) is symptom-indexed. This
> page is the story those two leave out.

---

## The shape of the thing

antigen has a *learning core*: given a cluster of structurally-similar marked sites,
it can **propose** a candidate fingerprint — a draft failure-class — and a
self-tolerance gate decides whether the draft is safe to keep. The machine docs call
it `cluster → propose → test → promote/prune`. Under that loop runs a **felt arc** in
four beats:

> **dread** *(a site smells off — I mark it)* → **propose** *(the learner
> anti-unifies the shape)* → **self-tolerance holds** *(it won't flag clean code)* →
> **witnessed** *(on antigen's own real source, not a toy)*.

The arc is worth feeling because it's the part you, a person, actually inhabit. Let's
walk it on antigen's own `#[dread]` marks — three sit in the production tree, and two
of them (the silent-skip twins) form the cluster the learner anti-unifies.

---

## Beat 1 — dread: a site smells off, and I can't name it yet

Here is a real function in antigen's own scanner. Read it the way its author did:

```rust
// antigen/src/scan/walk.rs
#[dread(
    trigger = "scan_workspace_inner silently `continue`s past a file it cannot read \
               (read_to_string Err -> skip), so an unreadable file lowers coverage \
               without lowering the all-clear verdict: 'reported clean' conflates \
               'found nothing' with 'could not look'. No counter, no surfaced skip."
)]
fn scan_workspace_inner(root: &Path, /* … */) -> std::io::Result<ScanReport> {
    // … walks the tree …
    let Ok(content) = std::fs::read_to_string(entry.path()) else {
        continue; // an unreadable file is silently skipped; the scan still reports "clean"
    };
}
```

The author wasn't running a checker. They were *reading*, and the floor felt rotten:
a directory walk that swallows a read error and proceeds reports a **clean** result
over an **incomplete** corpus — "found nothing" indistinguishable from "couldn't
look." That is antigen's *own* silent-failure class, living in antigen. You can't
fully formalize it in the moment. So you do the honest thing: you mark it.

`#[dread]` is the **marked-unknown** primitive — the third value made declarable.
Not "true" (a detected failure), not "false" (clean), but *"something here, I can't
name it."* Its biological referent is `angor animi`, the clinical sense of impending
doom that good clinicians *investigate* rather than dismiss, because it correlates
with real pathology before anything localizes.

Two things keep the mark honest, and both are enforced, not suggested:

- **The trigger is required.** `#[dread]` without a `trigger = "…"` won't compile.
  The earned-ness is structural — you must write down the felt thing. A triggerless
  dread is theater, and the macro refuses it.
- **It's asserted, not hypothesized.** `#[dread]` means *"I looked here and felt
  something off,"* not *"this pattern could hypothetically harbor a problem."* The
  speculative pattern-match is exactly the noise antigen rejects. The mark is a felt
  assertion with a person (or agent) behind it.

There's a lighter sibling, **`#[aura]`** — same kind of signal, lower intensity:
*"there **may** be something off here, worth a later look"* vs dread's *"something
**is** wrong here."* Both sit at the floor of the confidence dial: visible on a
scan, **never a gate, never nagging.** A live question, not a graveyard.

> **Where the marks live (stated plainly):** `#[dread]`/`#[aura]` are
> surfaced through antigen's **library** scan — `ScanReport.marked_unknowns` carries
> them, with their trigger and a structural digest. There is **no
> `cargo antigen dread` command**, and the CLI's console/JSON render does not
> surface the marked-unknown plane separately. You mark with the attribute; the
> library (and `cargo antigen propose`, which reads the marks into a cluster) reads
> them.

---

## The twin: I felt the same thing twice

The author didn't stop at one. Reading antigen's *audit* path, the floor felt rotten
in the **same shape**:

```rust
// antigen/src/audit/immunity.rs
#[dread(
    trigger = "collect_function_index silently `continue`s past a file it cannot read \
               and skips one it cannot parse, so the witness-function index is built \
               over an INCOMPLETE corpus; a downstream 'witness function not found' \
               cannot be told apart from 'the file was unreadable/unparseable'."
)]
fn collect_function_index(root: &Path) -> FunctionIndex {
    // … WalkDir + `let Ok(content) = read_to_string(..) else { continue };`
    //   + `if let Ok(file) = parse_file(..)` — read-or-skip, parse-or-skip …
}
```

Same felt class — *an incomplete result presented as a complete one* — same
structural shape (a directory walk that swallows its IO/parse error and proceeds).
**Two genuinely-felt sites that rhyme.** Hold onto that: the rhyme is what the next
beat is about.

*(The third `#[dread]` in the tree, on `cluster_key_of` in `finding.rs`, is a
different worry — a stringly-typed identity `format!("{class}@{structural_digest}")` that can
silently merge two distinct clusters onto one key. It even names the bug that made it
real: the P0a `dread@` over-merge. A single felt site, no twin — and that's fine;
not every dread has a sibling.)*

---

## Beat 2 — propose: the learner anti-unifies the shape

Now the learner. Given the **cluster** of the two silent-skip twins, `propose` reads
both, finds the shape they share that a *clean* walk does not, and drafts a candidate
fingerprint. This is a **library** call (the learner ships as an API, not a
`cargo antigen propose` command — say it plainly so you're not hunting for a verb
that doesn't exist):

```rust
use antigen::learn::propose;

// re-acquire the two felt twins from antigen's own committed source
let twins = vec![ /* scan_workspace_inner, collect_function_index */ ];

let draft = propose::anti_unify(&twins)
    .expect("the two felt twins share a fn skeleton to anti-unify");
```

The draft `binds` both twins — by construction, because the learner reads them with
the *same* syntactic walk the matcher uses. And it's **non-degenerate**: it captures
`read_to_string` (the call *both* twins make — the shared silent-skip signal), not a
shapeless `item = fn` that would match every function alive. This is exactly what
the dogfood test asserts:

```rust
// antigen/tests/learn_dogfood_propose.rs
let draft = propose::anti_unify(&twins).expect("…share a fn skeleton…");
for twin in &twins {
    assert!(draft.matches(twin), "the draft must bind each felt twin");
}
// the draft carries the shared silent-skip signal, not a shapeless match-everything:
use antigen_fingerprint::Constraint;
assert!(draft.constraints.iter().any(|c|
    matches!(c, Constraint::BodyCalls(n) if n == "read_to_string")));
```

The crucial restraint: **the draft is a hypothesis, not a verdict, and never a
name.** It is not auto-asserted as `#[presents]`. antigen does not declare a new
failure-class from it. The machine *binds* a shape; it does not *name* a class —
naming is the human's (or an incident's) job, and antigen stops at that line on
purpose.

> **Why it stops there isn't a limitation — it's the design.** An immune cell builds
> a receptor that binds a pathogen it has never seen, but it holds no *concept* of
> that pathogen; the organism (and science) does the naming, after. antigen's
> hardest boundary is the *same* boundary the immune system has, for the same
> information-theoretic reason: generate-and-select is machine-tractable; labeling a
> genuinely novel class is not. Protection goes ahead of naming — in biology and
> here.

### How the generalization actually works (so you trust it)

`anti_unify` doesn't do the naive thing. The naive least-general-generalization —
*drop the leaves that differ* — over-generalizes: a panic-in-`Drop` cluster
`{ .unwrap(), .expect() }` collapses to "any `Drop` impl," which matches a **clean**
`Drop` sibling. The generator's own output becomes the false positive. So `propose`
generalizes **to disjunction**:

- **shared signals** (present in *every* member) → AND'd conjuncts (the stable
  skeleton),
- **discriminating signals** (present in *some* members) → wrapped in `any_of([…])`
  (the load-bearing wall that carries the distinguishing signal without collapsing
  to a skeleton clean code also has).

On a `{ GuardA: .unwrap(), GuardB: .expect() }` Drop family, the draft is:

```
all_of([ item = impl, impl_of_trait("Drop"), body_calls("take"),
         any_of([ body_calls("expect"), body_calls("unwrap") ]) ])
```

— it binds both defects and **spares** `CleanGuard` (`.ok()`), because the `any_of`
arm is `NoMatch` on a sibling that has neither `unwrap` nor `expect`. The
disjunction is the precision.

---

## Beat 3 — self-tolerance holds: it won't flag clean code

Anti-unify-to-disjunction *reduces* the chance of flagging clean code, but it can't
*eliminate* it: a cluster whose distinguishing leaf happens to also appear in clean
code still over-binds. The thing that eliminates it is **self-tolerance** — the
spare-clean gate, B — and there is exactly **one** path to a promotable draft, and it
goes through B:

```rust
// the ONLY path to a promotable fingerprint (C ══ B, ADR-045):
let clean_corpus = vec![ /* strict_walk: walks files but propagates errors with `?` */ ];
let promoted: Option<Fingerprint> = propose::propose(&twins, &clean_corpus);
//   propose() == anti_unify() THEN promote_if_safe() — never one without the other.
```

> Yes, that's `propose::propose` — the promotable verb is a `fn` named `propose`
> inside the module `propose`, so you import the module (`use antigen::learn::propose;`)
> and call `propose::propose(...)`. It reads odd; it's a known library-DX wrinkle
> (the name will likely settle when the learner gets a user-facing surface). You're
> reading it right.

`promote_if_safe` does two refusals, and both matter:

```rust
// antigen/src/learn/self_tolerance.rs
pub fn promote_if_safe(draft: Fingerprint, clean_corpus: &[syn::Item]) -> Option<Fingerprint> {
    if clean_corpus.is_empty() {
        return None; // cannot certify safety against NOTHING — a vacuous spare-clean
                     // is "autoimmunity with a green check."
    }
    if spare_clean(&draft, clean_corpus) { Some(draft) } else { None }
}
```

1. **It refuses the empty corpus.** Sparing-clean is *vacuously* true against no
   clean code — and a vacuous pass is autoimmunity wearing a green check. The gate
   refuses to promote against nothing. You must supply real clean siblings.
2. **It refuses a draft that binds clean code.** If the draft would flag the clean
   `strict_walk` (the one that propagates its error with `?` instead of swallowing
   it), B returns `None`. *The generator's own over-broad output is the false
   positive the governor exists to catch.*

You cannot get a promoted draft that B didn't bless. Shipping a generator without the
selector ships autoimmunity — so the keystone makes C and B **co-ship**: that's the
one safety-tangle the whole learning core is built around. antigen's own test pins
the guarantee:

```rust
// antigen/tests/learn_dogfood_propose.rs — the line that must never pass:
if let Some(draft) = &promoted {
    assert!(!draft.matches(&clean_corpus[0]),
        "a PROMOTED draft must SPARE the clean sibling — if it binds clean, B was bypassed");
}
// promoted == None is ALSO safe: it means the draft reached clean and B correctly refused.
```

Read that last comment slowly. **Both outcomes are safe.** A promotion means B
verified the draft spares clean. A `None` means B caught an over-binder and pruned
it. There is no third outcome where a clean-flagging draft escapes. That's
self-tolerance: the discipline that keeps an immune system from attacking its own
body, made into a gate you can watch hold.

---

## Beat 4 — witnessed: none of this was a fixture

None of this is a fixture. The snippets above are illustrative, but the behaviors
they illustrate are pinned by **a real test** on antigen's own
committed source — `antigen/tests/learn_dogfood_propose.rs` is the witness, and you
can run it yourself:

```sh
# the strange loop, on the public repo:
cargo test -p antigen --test learn_dogfood_propose
#   running 3 tests
#   propose_routes_the_felt_draft_to_human_not_promote_the_settled_thesis ... ok
#   propose_anti_unifies_antigens_own_felt_twins_into_a_real_draft ... ok
#   the_p0b_marks_exist_and_are_surfaced_by_antigens_own_scan ... ok
#   test result: ok. 3 passed; 0 failed
```

The middle test names the honest outcome: antigen's own felt twins **route to a
human** — the draft is safe but the corpus holds no [near-miss](glossary.md#near-miss),
so the gate cannot certify it generalizes and hands it up rather than promote it. The
strange loop runs; it stops at the naming line.

The corpus the learner generalizes from is antigen's own honest self-doubt — the two
`#[dread]` twins, left in the production tree **on purpose**, because their value is
being living proof that the markers protect against *something, not nothing*. antigen
didn't manufacture a teaching bug. It *felt* one, twice, in itself, and kept the
feeling so you could watch the loop run on it.

And the loop closes on itself: the self-tolerance gate that spares the clean
`strict_walk` sibling in that test is the *same gate* that protects every adopter who
ever wires `propose()` into their own tooling. The thing that keeps antigen from
flagging its own clean code is the thing that will keep it from flagging yours. The
governor is universal; the demonstration is local.

---

## The register to leave with

This is **not** a victory lap. The learner proposes a *hypothesis*; the promotion is
*conditional*; the naming never happens automatically. The whole point is that
antigen does the bindable, machine-tractable part and **stops at the naming line** —
exactly where a careful clinician stops and says *"I feel something here, I can't
name it, let's investigate,"* instead of guessing a diagnosis.

The masterclass teaches **restraint, not cleverness.** A felt worry, marked
honestly. A draft, offered not asserted. A gate that would rather promote nothing
than promote something that flags clean code. That's the felt arc — and it's the
shape of the immune system, slowed down enough to feel.

---

## See also

- [`reading-a-verdict.md`](reading-a-verdict.md) — decode every scan/audit line +
  the bundled-catalog and flycheck surfaces.
- [`i-scanned-and.md`](i-scanned-and.md) — symptom-indexed troubleshooting.
- [`stdlib-families.md`](stdlib-families.md) — the catalog of shipped
  failure-classes and their tiers.
- The source, if you want to read the marks yourself:
  `antigen/src/scan/walk.rs`, `antigen/src/audit/immunity.rs`,
  `antigen/src/finding.rs` (the three `#[dread]` sites);
  `antigen/src/learn/propose.rs` + `self_tolerance.rs` (the learner + the gate);
  `antigen/tests/learn_dogfood_propose.rs` (the witnessed loop).

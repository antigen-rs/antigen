# Reading a verdict — what each scan / audit line actually means

> The one page to read **before your first scan**. `cargo antigen scan` and
> `cargo antigen audit` print a few distinct *line types*; this decodes each one in
> plain terms, so the output reads as information, not hieroglyphics.
>
> Two commands, two jobs:
> - **`scan`** *finds* — it walks the source and reports every site that **declares
>   a presentation** (`#[presents]`) or **matches a fingerprint** (a candidate).
> - **`audit`** *grades* — for each presentation, it reports whether a **witness**
>   covers it (defended) or not (undefended), and at what **tier** the proof holds.
>
> Run them on antigen's own examples to follow along:
>
> ```sh
> cargo run --bin cargo-antigen -- antigen scan  --root antigen/examples
> cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
> ```
>
> *(That's the from-source form, for running inside antigen's own repo. In **your
> own project** — once the `cargo-antigen` subcommand is installed — drop the
> `run --bin … --` and just say `cargo antigen scan` / `cargo antigen audit`.)*

---

## The first principle: a finding is not a failure

Antigen's whole posture is **honest tiering** — a verdict never claims more
certainty than it has earned. So a line on your screen is almost never "this is a
bug." It's one of: *you declared this site is in a failure-class's territory*, *a
fingerprint matched a shape here, go look*, or *this site is covered by a proof of
strength N*. **`undefended` means "no witness yet," not "broken."** Read every line
through that lens and the output stops feeling like an accusation.

---

## `scan` — the summary block

The first thing `scan` prints is a tally. Each line counts one *kind* of thing it
found:

```
Scanned 33 files, found 116 antigen-related declarations:
  - 27 antigen declarations
  - 45 explicit #[presents] markers
  - 40 fingerprint matches (candidate sites — see below)
  - 3 tolerated sites (#[antigen_tolerance])
  - 3 #[defended_by] declarations
  - 1 #[immune] declarations (deprecated — migrate to #[defended_by]/#[presents])
```

| Line | What it counts | What it means for you |
|---|---|---|
| **antigen declarations** | `#[antigen]` types — the failure-classes *declared* in the scanned tree (yours + any stdlib ones you imported). | The vocabulary in play. Nothing to act on; this is the dictionary. |
| **explicit `#[presents]` markers** | Sites *you* (or an example author) marked "this site is in this failure-class's territory." | **Author declarations** — they appear whether or not a fingerprint matched. `#[presents]` is *not* a vulnerability claim; see [the note on present ≠ vulnerable](#present--vulnerable). |
| **fingerprint matches (candidate sites)** | Sites the *scanner* flagged because their shape matched a fingerprint — and which you did **not** explicitly mark. | **Candidates, not a TODO list.** "Expected noise; the witness layer refines them." Go look; many will be fine (especially at the `suspected` tier). |
| **tolerated sites (`#[antigen_tolerance]`)** | Sites you explicitly marked "I know, and here's the rationale." | Acknowledged-on-purpose. The decision is recorded in the type system, not in your memory. |
| **`#[defended_by]` declarations** | Test functions that declare intent toward a failure-class — the **witnesses**. | These are what `audit` looks for. A presentation with a matching witness reports *defended*. |
| **`#[immune]` declarations (deprecated)** | The old v0.1 form. | Migrate to `#[defended_by]` (on a test) or `#[presents(requires = ...)]` (on the site). The audit still reads it, but it's on the way out. |

### The candidate list

After the summary, `scan` lists the fingerprint-match candidates, one per line:

```
antigen/examples/drop_panic.rs:50  PanicInDrop on impl [fingerprint match]
```

Read it as: *file:line · which antigen's fingerprint matched · the item kind ·
`[fingerprint match]`*. The header above the list says it plainly — **"candidate
sites (expected noise … Not a TODO list."** A candidate is an invitation to look,
weighted by the antigen's [tier](#tiers): a `named` match is worth defending; a
`suspected` match is a *prompt to look*, not a verdict.

> An explicitly `#[presents]`-marked site does **not** appear in this candidate list
> — it's a declared *presentation*, reported separately, not a fingerprint
> candidate. (That's why a member you marked may be "absent" from the candidates
> even though its fingerprint would match — the explicit mark takes precedence.)

### The unaddressed-presentations list

`scan` also lists explicit presentations that have no witness yet:

```
N unaddressed explicit presentation(s):
  antigen/examples/drop_panic.rs:50  PanicInDrop on impl
```

These are sites you *declared* are in a failure-class's territory and haven't yet
defended or tolerated. The fix is one of three moves (the scan prints them too):
`#[defended_by(<antigen>)]` on a test · `#[presents(<antigen>, requires = ...)]`
for substrate evidence · `#[antigen_tolerance(<antigen>, rationale = "...")]` to
record intent.

---

## `audit` — the verdict lines

`audit` is where each presentation gets graded. The two verdicts you'll see most:

```
✓ antigen/examples/basic.rs:70  PanickingInDrop — defended at Reachability by antigen/examples/basic.rs:102
✗ antigen/examples/drop_panic.rs:50  PanicInDrop — undefended (no #[defended_by] witness, no passing requires= predicate)
⚠ antigen/examples/agentic_coordination.rs:115  AgentWakeWithoutSubstrateDeltaInjection — substrate-gap (defense intent present; current substrate does not satisfy the requires= predicate)
```

| Symbol | Verdict | What it means |
|---|---|---|
| **`✓`** | `defended at <tier> by <site>` | Covered by a witness, proof holds at `<tier>` strength. The circuit is wired: the named witness site exercises the defense. `Reachability` is the everyday code-tier; stronger tiers exist (see [witness tiers](#witness-tiers)). |
| **`✗`** | `undefended (no #[defended_by] witness, no passing requires= predicate)` | No proof yet. **Not "broken" — just unwitnessed.** Add a `#[defended_by]` test, a `requires=` predicate, or an `#[antigen_tolerance]` rationale. |
| **`⚠`** | `substrate-gap (defense intent present; current substrate does not satisfy the requires= predicate)` | You *declared* a defense via `requires=`, but the substrate it checks for isn't there (e.g. a missing sidecar attestation). Not "no witness" — "witness intended, evidence absent." |

`audit` also prints a one-line tally:

```
7 defended, 26 undefended, 12 substrate-gap (across 45 presents-site(s))
```

- **defended** — covered by a witness, proof holds.
- **undefended** — no witness yet (the to-do, if the site really is vulnerable).
- **substrate-gap** — a `requires=` predicate was *evaluated and didn't pass* (e.g.
  a sidecar attestation is missing), distinct from "no witness at all."

### The broken-witness line (honesty in action)

If you name a witness that doesn't exist, `audit` says so rather than trusting you:

```
antigen/examples/broken_witness.rs:56  DemoBrokenWitness (witness = `nonexistent_test`)
    tier = None, hint = NoneApplicable
    → broken: no function named `nonexistent_test` found in any .rs file under the scan root
```

`tier = None` + `hint = NoneApplicable` means *the audit found no passing evidence*.
This is the point of the tool: a theatrical witness ("trust me, it's covered")
gets reported as `None`, not as a green check. The structural memory might say "this
site is immune"; the audit says "actually, the witness is broken." That's not a
failure of the audit — it's the audit doing its job.

---

## <a id="tiers"></a>Tiers — how loud is this verdict?

Two different things get called "tier." Don't conflate them:

### <a id="confidence-tier"></a>1. The **confidence tier** of a stdlib member (`named` / `suspected` / `chartered`)

How precisely a *fingerprint* targets the real defect — i.e. how much you should
trust a match. Straight from the [stdlib catalog](stdlib-families.md):

| Tier | What it promises | How to act on a match |
|---|---|---|
| **named** | High-confidence. The fingerprint's effective codomain *is* the defect population. "If it doesn't fire, you're covered." | Treat a match as a real site to defend or tolerate. |
| **suspected** | A correlator. The shape co-occurs with the defect but can also fire on idiomatic-correct code. | Treat a match as a *prompt to look*, not a verdict. Tolerate the benign ones explicitly. |
| **chartered** | The failure-class is real and recorded, but no honest fingerprint exists yet. Nothing ships. | Nothing to scan yet; the class is identified so the graduation path is tracked. |

`suspected` is **lower-precision, not lower-stakes** — a `suspected` match isn't
"probably fine," it's "this shape is real, and it also appears in safe code, so
*you* look." `named` here is the *tier*; it is **not** the separate sense in which
antigen "names" (declares) a class — a `suspected` family is still a declared
failure-class, just not at the `named` tier.

### <a id="witness-tiers"></a>2. The **witness tier** of a defense (`None` / `Reachability` / … / `FormalProof`)

How *strong* the proof is that a defended site is actually safe — the `<tier>` in
`defended at Reachability`. `Reachability` is the everyday code-tier (a test
exercises the path); `FormalProof` is the strongest. Full gradient in
[`witness-tiers.md`](witness-tiers.md).

> The trap: a newcomer who clicks "tier" wanting "why is this member *suspected*"
> can land on witness-tiers (defense-strength) — the **wrong** axis. Member-confidence
> tier = how precisely the *fingerprint* targets the defect ([catalog](stdlib-families.md));
> witness tier = how strong the *defense proof* is ([witness-tiers.md](witness-tiers.md)).

---

## <a id="present--vulnerable"></a>`present` ≠ `vulnerable`

The single most common newcomer surprise: you scan, and a site you *know* is safe
shows up anyway. That's correct, and it's the point. `#[presents(X)]` means **"this
site is in failure-class X's territory"** — an author's declaration that the site is
worth auditing — **not** "this site is vulnerable." A safe sibling marked
`#[presents]` lists right alongside the risky one; the difference between them is
proved by the *witness* (at `audit`) or by the *fingerprint* (in the source), not by
one of them vanishing from the console.

If that surprised you on a real scan, the symptom-by-symptom companion is
[`i-scanned-and.md`](i-scanned-and.md) ("I scanned and… both my bad and safe code
showed up"). To *see* the fingerprint actually separate a bad shape from a clean
one, [`three-places-to-see-it.md`](three-places-to-see-it.md) shows you where.

---

## See also

- [`i-scanned-and.md`](i-scanned-and.md) — symptom-indexed troubleshooting ("I
  scanned and ___").
- [`three-places-to-see-it.md`](three-places-to-see-it.md) — where each thing
  (class-level defense, the fingerprint sparing a site, every family's bind/spare)
  is actually visible.
- [`stdlib-families.md`](stdlib-families.md) — the catalog: every shipped
  failure-class, its tier, its fingerprint, what it catches.
- [`witness-tiers.md`](witness-tiers.md) — the witness-strength gradient.
- [`examples-guide.md`](examples-guide.md) — runnable lesson per example.

# "I scanned and…" — the symptom-indexed FAQ

> Troubleshooting organized by **what you saw**, not by concept — because when a
> scan surprises you, you search for the thing on your screen, not for the term you
> don't know yet. Each entry is a real first-timer surprise, with the one-line
> reason and the fix.
>
> New to the output entirely? Read [`reading-a-verdict.md`](reading-a-verdict.md)
> first — it decodes every line type. This page assumes you've seen the output and
> something didn't match your expectation.

---

## …it found **nothing** (and I expected the catalog to fire)

```
Scanned 1 files, found 0 antigen-related declarations:
  - 0 antigen declarations
  - 0 explicit #[presents] markers

All explicit presentations are addressed.
```

> **If you see this on a fresh crate, something is off.** Auto-detect
> should have injected the bundled catalog and surfaced candidates. Read on.

**Why a fresh crate is not empty:** a scanner that only matched failure-classes
*declared in the scanned tree* (your `#[antigen]` types + any stdlib ones you
`use`-imported) would find nothing in a crate that imported none **by
construction** — an empty dictionary, not a clean bill of health. antigen ships a
**bundled stdlib catalog** and **auto-injects** it when your crate declares no
antigens of its own. So a bare scan on a fresh crate surfaces real candidates:

```
Scanned 1 files, found 2 antigen-related declarations:
  - 0 antigen declarations
  - 0 explicit #[presents] markers
  - 2 fingerprint matches (candidate sites — see below)

  …/lib.rs:21  get-unchecked-without-proof on fn [fingerprint match]
  …/lib.rs:30  panic-in-drop on impl [fingerprint match]
```

**So if you still got the empty version, the cause is one of:**

- **Your crate *already* declares antigens.** Auto-detect only injects when the tree
  has none of its own — it won't surprise a repo that's already speaking antigen. To
  run the flagship catalog *on purpose* over such a tree, pass `--bundled-catalog`
  explicitly (it always injects, augmenting your declarations — and it gets loud; see
  the [firehose entry](#it-flooded-me-with-thousands-of-matches) below).
- **Genuinely nothing matched — and on a zero-declaration crate, that's now a *real*
  all-clear.** This is the subtle one: the empty output above is **byte-identical** to
  a bare empty-dictionary all-clear, but its *meaning* is different. On a crate with no
  antigens of its own, auto-detect *did* inject the bundled catalog and *did* consult
  it — you simply don't trip any flagship fingerprint. So "found 0 declarations" no
  longer means "I had no vocabulary, I checked nothing" (the cliff). On a zero-decl
  crate it now means **"I checked you against the flagship catalog and you're clean
  against it."** The words didn't change; the thing behind them did. (The output gives
  no explicit "catalog consulted" line — so if you want to *prove* to yourself the
  catalog ran, pass `--bundled-catalog` explicitly, or scan a crate that *does* have a
  footgun, and watch the candidates appear like the example above.)

**Fix (to teach antigen *your* classes):** import the stdlib members you care about
and mark the sites that present them — the catalog covers the flagships, but your
project's own footguns still want your own declarations:

```rust
use antigen::stdlib::drop_panic::PanicInDrop;
use antigen::presents;

#[presents(PanicInDrop)]
impl Drop for MyGuard { /* ... */ }
```

(Browse the available members in the [catalog](stdlib-families.md); the
fingerprint-match-vs-verdict claim-scope is in
[`reading-a-verdict.md`](reading-a-verdict.md#claim-scope).)

---

## …it flagged my **own clean code**

You scanned, and a fingerprint matched a site you *know* is fine — maybe even a
recommended-safe API.

**Why:** the match is almost certainly from a **`suspected`**-tier member. A
`suspected` fingerprint is a *correlator*: its shape co-occurs with the defect but
also appears in idiomatic-correct code. It's **lower-precision, not lower-stakes** —
the match means "this shape is real, and it also shows up in safe code, so *you*
look," not "this is a bug." (A `named`-tier match, by contrast, is high-confidence.
Check which tier fired in the [catalog](stdlib-families.md).)

**Fix:** if the site is genuinely benign, say so — in the type system, not in your
head:

```rust
#[antigen_tolerance(SizeOfInElementCount, rationale = "byte buffer copy; count is in bytes by design")]
fn copy_bytes(/* ... */) { /* ... */ }
```

Now the next scan records *acknowledged-on-purpose* instead of re-surprising you,
and the rationale travels with the code.

---

## …`audit` says **undefended**, but I **wrote a test**

You have a test that exercises the safe path, yet `audit` still prints
`✗ … undefended`.

**Why:** a plain test function is invisible to the audit. The audit looks for a
test that **declares its intent** toward the failure-class — i.e. carries
`#[defended_by(<antigen>)]`. Without that marker, the audit can't know your test is
the witness for this presentation.

**Fix:** mark the witness.

```rust
#[defended_by(PanicInDrop)]
fn safe_guard_drop_does_not_panic() { /* exercises the safe drop */ }
```

Re-run `audit` and the presentation flips to `✓ defended at Reachability by <your
test>`. (Defense lives on the **witness**, never claimed at the site — that's
ADR-029. The site says "I'm in this territory"; the test says "and here's the
proof.")

---

## …**both** my bad **and** safe code show up

You marked a risky site and its safe sibling, scanned, and `audit` lists **both** as
presentations — the safe one didn't get a pass.

**Why:** `#[presents(X)]` means **"this site is in failure-class X's territory,"**
not "this site is vulnerable." `present` ≠ `vulnerable`. An explicit `#[presents]`
is an author declaration that surfaces in `scan`/`audit` *regardless of whether the
fingerprint would match* — so a safe sibling you marked lists right next to the
risky one. This is by design: it teaches that a site can be *in the territory* and
still be fine.

**Fix:** nothing is wrong. To distinguish them at the console, give the safe one a
witness (`#[defended_by]`) so it reports `✓ defended` while the risky one stays
`✗ undefended`. The *difference* lives in the witness layer (or in the source), not
in one of them disappearing.

---

## …the **safe one didn't disappear** (I thought it'd be "spared")

The docs say the safe sibling is "spared," but you ran the scan and it's still
listed.

**Why:** "spared" has two meanings, and only one is about the console.
- **Spared by the fingerprint** = the fingerprint *doesn't bind* the safe sibling
  (no `from_reader`, a panic-free `Drop`, …). This is true, and visible **in the
  source**.
- **Disappears from the console** = only happens for an **un-marked** sibling — one
  with no `#[presents]`, which the fingerprint also doesn't match, so it simply
  never appears.

If your safe sibling is `#[presents]`-marked, it's "spared by the fingerprint" but
**still listed** (because the explicit mark surfaces it). Read "spared" as *the
fingerprint doesn't bind it*, not *it vanishes from the console*.

**See it for real:** [`three-places-to-see-it.md`](three-places-to-see-it.md) shows
you the one place a fingerprint genuinely *does* make a safe site vanish (an
un-marked sibling), plus where every family's bind-vs-spare sits side by side.

---

## …a witness I named reported **`tier = None` / broken**

```
DemoBrokenWitness (witness = `nonexistent_test`)
    tier = None, hint = NoneApplicable
    → broken: no function named `nonexistent_test` found in any .rs file under the scan root
```

**Why:** you pointed a defense at a witness that the audit can't find. Rather than
trust the claim, the audit reports `None` — no passing evidence. This is the tool
working: a theatrical witness gets named honestly, not waved through as green.

**Fix:** make the witness real — a function with that exact name must exist under
the scan root and carry the right marker. (Typo in the name? Wrong scan root? Both
are common.)

---

## …it printed a **`[fingerprint match]` candidate** I didn't ask for

A line like `…:45  CheckedArithmeticOverflow on fn [fingerprint match]` for a site
you never marked.

**Why:** that's a **candidate**, not a finding. The scanner matched a fingerprint's
*shape* at a site you didn't explicitly mark. The header says it: *"candidate sites
(expected noise … Not a TODO list.")* The witness/tier layer refines these — many
candidates, especially at the `suspected` tier, are fine.

**Fix:** look, weighted by tier. A `named` candidate is worth acknowledging
(`#[presents]` + defend/tolerate); a `suspected` candidate is a prompt to glance, not
an obligation. Nothing forces you to address a candidate.

---

## …it **flooded me with thousands of matches**

You ran `cargo antigen scan --bundled-catalog` and got a wall — tens of thousands of
candidate lines.

**Why:** **explicit** `--bundled-catalog` *always* injects the bundled catalog and
**augments** whatever your tree already declares — so on a repo that already declares
many antigens, you're now matching *every* bundled fingerprint against *every* site,
on top of your own. On antigen's own tree, that produces ~25,000 matches across ~76
types. It isn't broken; it's the augment-mode firehose.

**Fix:** match the mode to the job —

- On a **consumer crate** (no antigens of its own), use **bare scan** — auto-detect
  injects the catalog *only* when you have nothing of your own, which is exactly the
  fresh-crate case you want.
- Use **explicit `--bundled-catalog`** deliberately, scoped with `--root <subdir>`,
  when you want to run the flagship classes over a specific area on purpose.
- Remember the tier filter: most of a firehose is **`suspected`**-tier noise (a
  *prompt to look*, not a verdict). Read [reading-a-verdict.md](reading-a-verdict.md#tiers)
  on tiers before treating the count as a to-do list.

> One more: `--bundled-catalog` together with `--workspace` (multi-crate) emits a
> `W-MULTICRATE` warning — it's surfaced, not silent. See
> [troubleshooting.md](troubleshooting.md).

---

## …the matches showed up as **squiggles in my editor**

You wired antigen into rust-analyzer and now fingerprint matches appear inline as
**warnings**, like compiler diagnostics.

**Why:** that's `scan --message-format json` working as intended. It speaks
the rustc/cargo line-protocol, so an editor's flycheck consumes antigen findings as
diagnostics — no custom LSP server. Every finding emits at **`warning` level only**:
antigen never fails your build, and each squiggle carries the line *"a fingerprint
match to inspect, not an audited verdict."*

**Fix:** nothing — that's the feature. Read each squiggle as a *pointer to look*, not
a build error. (Setup + the per-diagnostic claim-scope line are in
[reading-a-verdict.md](reading-a-verdict.md#claim-scope).)

---

## …`#[immune]` is **deprecated** — what do I use?

```
- 1 #[immune] declarations (deprecated — migrate to #[defended_by]/#[presents])
```

**Why:** `#[immune]` is the v0.1 form, where immunity was *claimed at the site*.
ADR-029 moved antigen to *observe* immunity from evidence instead of claiming it.

**Fix:** migrate per case —
- a code-tier defense → `#[defended_by(X)]` on the **test**;
- a substrate-tier defense → `#[presents(X, requires = ...)]` on the **site**.

The audit still reads `#[immune]` for now, but it's on the way out.

---

## See also

- [`reading-a-verdict.md`](reading-a-verdict.md) — what every scan/audit line means
  (incl. the bundled-catalog + flycheck surfaces).
- [`the-felt-arc.md`](the-felt-arc.md) — the story: marking a felt worry
  (`#[dread]`), the learner proposing a class, self-tolerance sparing clean code.
- [`three-places-to-see-it.md`](three-places-to-see-it.md) — where the
  fingerprint's bind/spare is actually visible.
- [`stdlib-families.md`](stdlib-families.md) — the catalog (tier of each member).
- [`examples-guide.md`](examples-guide.md) — runnable lesson per example.

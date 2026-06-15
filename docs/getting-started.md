# Getting started — your first session with antigen

> The newcomer's onboarding path. Where [`quickstart.md`](quickstart.md) is a
> five-minute taste and [`tutorial.md`](tutorial.md) is the fifteen-minute
> declare-your-own-classes deep dive, **this page walks your literal first session**
> — install, first scan, reading one real finding, the one idea you must internalize,
> and wiring your editor — with **every command run for real** and the **actual
> output** shown. No prior antigen knowledge assumed; nothing here requires reading an
> ADR.
>
> Each output block below is a real run against a tiny throwaway crate with **one**
> footgun (a `.unwrap()` inside a `Drop` impl). You can build the same crate and
> follow along; the numbers will match.

---

## 0. The one sentence to carry the whole way

Everything antigen's scanner shows you is

> **a fingerprint match to inspect, not an audited verdict.**

Hold that. A `scan` line says *"this code's shape structurally resembles a known
failure-class"* — **not** *"this is a bug."* If you read every line through that lens,
the output stops feeling like an accusation and starts feeling like a colleague
pointing at a spot on the map. (The deeper "why" is
[`reading-a-verdict.md`](reading-a-verdict.md); the conceptual version is
[`concepts.md`](concepts.md).)

---

## 1. Install (30 seconds)

```sh
cargo install cargo-antigen
```

`cargo antigen scan --help` lists the flags your installed copy supports —
including **`--bundled-catalog`** and **`--message-format`**, the two surfaces this
page walks you through:

```sh
cargo antigen scan --help
```

---

## 2. Your first scan — watch the catalog fire

To follow along with the exact output below, drop this into a throwaway crate's
`src/lib.rs` (one real footgun: a `.unwrap()` inside a `Drop` impl):

```rust
//! A small crate a newcomer might scan on day one.

pub struct FileHandle {
    fd: i32,
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        close(self.fd).unwrap();   // .unwrap() inside Drop — a panic here aborts the process
    }
}

fn close(_fd: i32) -> Result<(), std::io::Error> {
    Ok(())
}
```

Now run, with **no flags, no setup, no dependency added**:

```sh
cargo antigen scan
```

Here is the part that surprises everyone, and it's the whole point of the bundled
catalog. Even though this crate declares **zero antigens of its own**, the scan is
not empty:

```
Scanning workspace: .

Scanned 1 files, found 1 antigen-related declarations:
  - 0 antigen declarations
  - 0 explicit #[presents] markers
  - 1 fingerprint matches (candidate sites — see below)

1 fingerprint match(es) across 1 antigen type(s) — candidate sites (expected
noise; the witness layer refines them, per the filter/proof split). Not a TODO list.

  ./src/lib.rs:7  panic-in-drop on impl [fingerprint match]

  These are CANDIDATES, not failures. If a site genuinely presents the
  failure-class, acknowledge it:
    #[presents(<antigen>)] to mark the site explicitly,
      then defend it: #[defended_by(<antigen>)] on a test (code-tier), or
      #[presents(<antigen>, requires = ...)] for substrate-witness evidence,
    #[antigen_tolerance(<antigen>, rationale = "...")] to document intent.

All explicit presentations are addressed.
```

**Where did `panic-in-drop` come from?** You never declared it. antigen ships a
**bundled stdlib catalog** — its flagship failure-class fingerprints, built in — and
when it finds zero antigen declarations in your tree, it **auto-injects** that catalog
and scans your code against it. So a brand-new crate gets real findings from
antigen's shipped knowledge on day one, before you've written a single `#[antigen]`.

> **The thing this fixes.** Without the bundled catalog, that same fresh crate would
> print `found 0 declarations` — which *looks* like a clean bill of health but really
> means *"I had nothing to compare your code against."* An empty dictionary dressed up
> as immunity — the "zero-hits cliff," a false all-clear. The auto-inject closes it:
> you get checked against real fingerprints from the first run.

### What if my scan really *is* empty?

If your crate genuinely has none of the catalog's footgun shapes, you'll still see:

```
Scanned 1 files, found 0 antigen-related declarations
All explicit presentations are addressed.
```

That output is *byte-identical* to a bare empty-dictionary one — antigen doesn't yet
print "I checked you against the catalog and found nothing." But it **did**
check: this is a real clean (clean against antigen's flagship fingerprints), not an
empty repertoire. (If you want to be certain the catalog ran, add an obvious footgun
— a `.unwrap()` in a `Drop` impl — and re-scan; you'll see it fire.)

### Making the catalog explicit

The auto-inject above happens **only** when your crate declares no antigens of its
own. Two situations where you'd pass the flag by hand:

```sh
cargo antigen scan --bundled-catalog
```

- **You already declare your own antigens** and *also* want the shipped catalog
  layered on. Without the flag, a crate that already "speaks antigen" uses *your*
  vocabulary and won't get a surprise injection. `--bundled-catalog` always injects.
- **Heads up — it gets loud.** Explicit `--bundled-catalog` matches *every* shipped
  fingerprint against *every* site. On a large repo that already declares antigens
  this is a firehose (tens of thousands of candidates). Most of it is `suspected`-tier
  noise — a *prompt to glance*, not a to-do list. See
  [`troubleshooting.md`](troubleshooting.md) and the tier explanation in
  [`reading-a-verdict.md`](reading-a-verdict.md#tiers). For everyday use on a consumer
  crate, prefer the **bare scan** (auto-detect injects only when you have nothing of
  your own).

---

## 3. Reading the one finding, field by field

Take the candidate line apart:

```
./src/lib.rs:7  panic-in-drop on impl [fingerprint match]
```

| Piece | Means |
|---|---|
| `./src/lib.rs:7` | **where** — file and line. Here, line 7 is an `impl Drop`. |
| `panic-in-drop` | **which** failure-class fingerprint matched (a shipped catalog member). |
| `on impl` | the **item kind** the shape was found on. |
| `[fingerprint match]` | the **claim scope** — a structural resemblance, *not* a verdict. |

Line 7 is the `impl Drop` from the source in §2 — the `.unwrap()` inside `drop()`.
antigen is saying: *"the shape of this `Drop` impl matches a known failure-class
(panic during drop)."* It has **not** decided your code is wrong — a `.unwrap()` that
can never fail is fine; the scanner can't tell. That judgment is yours. The candidate
is the invitation to make it.

> **`scan` finds; `audit` grades.** `scan` reports shapes (declared presentations +
> fingerprint candidates). `audit` is the separate command that grades the *defenses*
> on sites you've explicitly marked. A bare scan candidate has no verdict yet — that's
> correct, not incomplete.

---

## 4. The three things you can do with a candidate

You are never *required* to act on a candidate. But when one is real, you have three
honest moves — and notice that **none** of them is "claim it's immune." A site never
declares its own safety; you record evidence, and `audit` *observes* the strength.

1. **Mark it and defend it.** Put `#[presents(panic-in-drop)]` on the site to say
   "this site is in this failure-class's territory," then point at evidence —
   a test that exercises the safe behavior, marked `#[defended_by(panic-in-drop)]`.
2. **Tolerate it on purpose.** `#[antigen_tolerance(panic-in-drop, rationale = "fd
   close is infallible here")]` — record *why* it's acceptable, in the type system,
   not in your memory.
3. **Refactor it away.** Remove the shape (e.g. don't `.unwrap()` in `Drop`).

The full walkthrough of declaring, defending, and auditing is the
[`tutorial.md`](tutorial.md). The decoder for every verdict line is
[`reading-a-verdict.md`](reading-a-verdict.md). If a scan surprised you ("it flagged
my clean code", "it found nothing", "both my bad and safe code showed up"), the
symptom-indexed answers are in [`i-scanned-and.md`](i-scanned-and.md).

---

## 5. Put it in your editor (flycheck)

You don't have to run `scan` by hand. antigen can speak the **rustc
`--message-format=json` line-protocol** that rust-analyzer already understands, so
findings appear as inline warning squiggles — no custom LSP server, no plugin.

Run it once to see the shape:

```sh
cargo antigen scan --message-format json
```

```json
{"reason":"compiler-message","message":{"message":"antigen: structure matches the `panic-in-drop` failure-class fingerprint (provenance: constructable (a verified minimal case exists)). This is a fingerprint match to inspect, not an audited verdict.","level":"warning","code":{"code":"antigen::panic-in-drop"},"spans":[{"file_name":"src/lib.rs","line_start":7,"is_primary":true}],"children":[{"level":"note","message":"fingerprint match only — antigen has not audited a defense for this site. Mark it with #[presents(panic-in-drop)] + #[defended_by(...)] ..."}]}}
```

Two things to notice — both are the claim-scope made literal:

- `"level":"warning"` — **always a warning, never an error.** antigen never fails your
  build. A fingerprint match is a candidate, full stop.
- the message carries the verbatim sentence **"This is a fingerprint match to inspect,
  not an audited verdict."** — per diagnostic. The screen tells you its own scope.

Wire it into rust-analyzer (`.vscode/settings.json` or your editor's equivalent):

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "antigen", "scan", "--message-format", "json"
  ]
}
```

Now every save surfaces antigen candidates inline.

> **Expect candidate density.** flycheck draws one squiggle per fingerprint match. On
> a repo with broad fingerprints (or if you add `--bundled-catalog` to the command),
> that can be *many* `suspected`-tier squiggles — a prompt to glance, not errors to
> fix. If it's noisy, scope it: filter with `--category functional-correctness`, or
> point `--root` at a subdirectory, and read
> [`reading-a-verdict.md`](reading-a-verdict.md#tiers) on tiers before treating
> squiggles as a to-do list. The full editor-wiring story is in
> [`editor-integration.md`](editor-integration.md); the JSON field schema is in
> [`output-formats.md`](output-formats.md), and the CI side is in
> [`deployment-ci-integration.md`](deployment-ci-integration.md).

> **`--message-format json` is not `--format json`.** `--message-format json` is the
> rustc line-protocol *for editors*. `--format json` is antigen's own report envelope
> *for scripts/CI* (findings under `report.presentations`, each carrying
> `"match_kind": "fingerprint_match"`). Same findings, two different shapes, two
> different consumers.

---

## 6. Where to go from here

You've now done the whole first loop: installed antigen, scanned, read a real finding,
learned that a match is a candidate-not-a-verdict, and (optionally) wired your editor.
That's the floor — real value with zero declarations of your own. Next:

- **[`tutorial.md`](tutorial.md)** — declare your *own* first failure-class, defend a
  site, and run `cargo antigen audit` to grade the defense (the full 15-minute loop).
- **[`stdlib-families.md`](stdlib-families.md)** — the catalog: every shipped
  failure-class the bare scan checks you against, with its tier.
- **[`reading-a-verdict.md`](reading-a-verdict.md)** — the line-by-line decoder for
  every `scan` and `audit` output.
- **[`i-scanned-and.md`](i-scanned-and.md)** — symptom-first answers when a scan
  surprises you.
- **[`cli-reference.md`](cli-reference.md)** — the rest of the `cargo antigen`
  command surface (you've used `scan`; there are seven more verbs).
- **[`concepts.md`](concepts.md)** — what antigen *is*, architecturally, and why a
  match isn't a verdict by construction.
- **[`index.md`](index.md)** — the full documentation map.

> Already past day one and want the *feel* of the learning loop — marking a
> felt-but-unnamed worry, watching antigen propose a class from it, watching
> self-tolerance refuse to flag clean code? Try **`cargo antigen propose`**
> (see [`cli-reference.md`](cli-reference.md#propose) and the runnable
> [`examples/propose-demo/`](../examples/propose-demo/)); the felt story is in
> [`the-felt-arc.md`](the-felt-arc.md), and the first-principles "why it's safe"
> is in [`the-keystone-explained.md`](the-keystone-explained.md) and
> [`concepts.md`](concepts.md).

---

## What you learned, in one breath

A bare `cargo antigen scan` checks your code against antigen's bundled catalog of
known failure-classes — even with zero setup — and shows you candidate sites that
*structurally resemble* those classes. Every candidate is **a fingerprint match to
inspect, not an audited verdict.** You decide what's real; antigen remembers the
shapes so the lesson outlives your memory.

# Antigen — 5-Minute Quickstart

> The fastest path from zero to running antigen in your project. For
> the full first-15-minutes walkthrough, see [`tutorial.md`](tutorial.md).
> For the conceptual framing, see [`concepts.md`](concepts.md).

---

## Step 1 — Install the cargo subcommand (30 seconds)

```sh
cargo install cargo-antigen
```

> `cargo install cargo-antigen` installs the latest published release. The
> prescriptive work-orchestration family, the failure-class stdlib families, the
> bundled-catalog auto-detect (Step 2), and the `--message-format json` editor
> flycheck (see [output-formats.md](output-formats.md)) are all included. The
> source of truth for what your installed copy supports is `cargo antigen scan
> --help`.

Verify:

```sh
cargo antigen --help
```

You should see:

```
$ cargo antigen --help
The "antigen" subcommand of cargo

Usage: cargo antigen <COMMAND>

Commands:
  scan         Scan the workspace for antigen presentations and report unaddressed ones
  audit        Comprehensive immunity coverage report — witness resolution and tier validation
  attest       Manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019)
  tolerate     Manage tolerance-ratification sidecars (ADR-019 §tolerance tier)
  oracle       Manage Oracle artifact-class records (ADR-021 §D3)
  verify       Drive Supply-Chain Defense Family verifications (ADR-025)
  vcs          Drive VCS-Information-Loss Family observations (ADR-026)
  mucosal-map  Map mucosal trust boundaries across the workspace (ADR-027 + Amd 1)
  fingerprint  Print the structural fingerprint of a scanned item
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

You only need `scan` and `audit` for the quickstart. The other subcommands cover advanced workflows (substrate-witness sidecars, tolerance ratification, Oracle artifact lifecycle, supply-chain verification, VCS observations, fingerprint queries) — see [`witness-tiers.md`](witness-tiers.md) and the [tutorial](tutorial.md) when you're ready.

---

## Step 2 — Run scan on any Rust project (15 seconds)

```sh
cd /path/to/your/rust/project
cargo antigen scan
```

Here is the surprise — and the whole point of the bundled catalog. On a
fresh codebase **with no antigens declared yet**, scan does *not* report a false
all-clear. When antigen finds zero in-tree declarations, it **auto-injects its
bundled stdlib catalog** (the flagship failure-class fingerprints it ships with)
and scans your code against *those*:

```
Scanning workspace: .

Scanned 1 files, found 2 antigen-related declarations:
  - 0 antigen declarations
  - 0 explicit #[presents] markers
  - 2 fingerprint matches (candidate sites — see below)

2 fingerprint match(es) across 2 antigen type(s) — candidate sites (expected
noise; the witness layer refines them, per the filter/proof split). Not a TODO list.

  src/lib.rs:21  get-unchecked-without-proof on fn [fingerprint match]
  src/lib.rs:30  panic-in-drop on impl [fingerprint match]

  These are CANDIDATES, not failures. If a site genuinely presents the
  failure-class, acknowledge it:
    #[presents(<antigen>)] to mark the site explicitly, ...
```

So even before you declare a single antigen, scan surfaces real footguns from
antigen's shipped knowledge — here, an unchecked `get_unchecked` index and a
`.unwrap()` inside a `Drop` impl. These are **fingerprint matches to inspect, not
audited verdicts**: antigen is saying "this *structurally resembles* a known
failure-class," not "this is broken."

> **Why this matters.** Without the bundled catalog, a fresh crate with no
> declarations would get `found 0 declarations` — a *false* all-clear (an empty
> knowledge-base looks identical to a clean codebase). The auto-injected bundled
> catalog closes that "zero-hits cliff": an empty repertoire no longer reads as
> immunity.

### `--bundled-catalog` — make it explicit

The auto-inject above happens **only** when your crate has zero antigens of its
own. If you already declare antigens and *also* want to scan against the shipped
catalog (augment mode), pass the flag explicitly:

```sh
cargo antigen scan --bundled-catalog
```

- **Without the flag:** the catalog auto-injects *only* on a zero-declaration
  crate (ADR-043 Amendment 2). Once you declare your own antigens, bare scan uses
  *yours*.
- **With `--bundled-catalog`:** the catalog is *always* injected, augmenting your
  local antigens. On a repo that already declares many antigens this is a
  firehose — see [troubleshooting.md](troubleshooting.md). Use bare scan
  (auto-detect) on consumer crates; reach for explicit `--bundled-catalog` only
  when you deliberately want the shipped families layered on top of your own.

If your codebase genuinely has **none** of the catalog's footgun shapes, bare
scan still prints `found 0 antigen-related declarations` / `All explicit
presentations are addressed`. Here's the subtlety worth knowing: that line reads
the same as a bare "empty dictionary" output, but it *means* something
stronger — **clean against antigen's bundled flagship catalog**, not "I had
nothing to check against." The catalog *did* run; it found no matching shapes.
(The console doesn't yet say "checked against the catalog, 0 matches" on the
clean path — so until it does, read a zero-match bare scan on a fresh crate as
*"vetted against the shipped families and clean,"* not as the old false
all-clear.)

---

## Step 3 — Add antigen as a dependency (30 seconds)

```toml
# Cargo.toml
[dependencies]
antigen = "0.4.0-beta.1"   # check crates.io for the latest version
```

Run `cargo build` to fetch and compile. Antigen's runtime cost is zero — the macros are identity transforms; no code generation, no runtime overhead.

---

## Step 4 — Declare your first antigen (2 minutes)

Create `src/antigens.rs`:

```rust
use antigen::antigen;

/// Drop impls must not panic; panic-during-unwind causes process abort.
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    fingerprint = r#"
        item = impl,
        has_method("drop", "(& mut self)"),
        any_of([
            body_contains_macro("panic"),
            body_contains_macro("unreachable"),
        ])
    "#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
)]
pub struct PanickingInDrop;
```

Register the module in `src/lib.rs`:

```rust
pub mod antigens;
```

---

## Step 5 — Run scan again (15 seconds)

```sh
cargo antigen scan
```

Now you'll see your antigen declaration found, plus any sites in your codebase that structurally match its fingerprint. Real output looks like this (truncated):

```
Scanning workspace: .

Scanned 42 files, found 7 antigen-related declarations:
  - 1 antigen declaration
  - 0 explicit #[presents] markers
  - 6 fingerprint matches (candidate sites — see below)
  - 0 tolerated sites (#[antigen_tolerance])
  - 0 #[defended_by] declarations
  - 0 parse failures

6 fingerprint match(es) — structurally similar to a declared antigen:

  ./src/cleanup.rs:42  PanickingInDrop on impl
  ./src/resource.rs:117  PanickingInDrop on impl
  ...

  These are CANDIDATES, not failures. If a site genuinely presents the
  failure-class, acknowledge it:
    #[presents(<antigen>)] to mark the site explicitly,
      then defend it: #[defended_by(<antigen>)] on a test (code-tier), or
      #[presents(<antigen>, requires = ...)] for substrate-witness evidence,
    #[antigen_tolerance(<antigen>, rationale = "...")] to document intent.
```

The **fingerprint matches** are sites in your code that look structurally like the failure class you declared. Each one is a decision-point.

If your codebase doesn't have any matching sites, the output is clean. Either way, your antigen is now part of your project's structural memory.

---

## What just happened

You declared a named failure-class with a structural fingerprint. `cargo antigen scan` walks your codebase and tells you every site that structurally resembles the failure. The lesson — "drop impls must not panic" — now lives in your codebase as durable substrate, not in your head.

For each surfaced site, you have three choices. Note the model: a site never *claims* "I am immune." You mark the site with `#[presents]` and wire up evidence; `cargo antigen audit` then **observes** how strong the defense is (immunity is observed, not declared).

- **Mark the site and defend it.** Put `#[presents(PanickingInDrop)]` on the site, then provide evidence. Quick test: **can a test execute the thing you're defending?** Yes → write a test that exercises the behavior (e.g. drop and check no panic) and annotate it `#[defended_by(PanickingInDrop)]` (code-tier defense). No → the evidence lives outside the code (a ratified doc, a pinned dep, a signed discipline), so attach it on the site itself with `#[presents(PanickingInDrop, requires = ratified_doc(...))]` (or `requires = signers(...)`, etc.). Full substrate-witness path: [`tutorial.md`](tutorial.md).
- **`#[antigen_tolerance(PanickingInDrop, rationale = "...")]`** — acknowledge the match is intentional or accepted, with required justification
- **Refactor** — eliminate the failure-class shape

That's the floor. Antigen meets you at this floor and grows with your practice.

---

## Step 6 — Audit your defenses (optional, 1 minute)

Once you've wired `#[presents]` sites to `#[defended_by]` tests (or `requires =` substrate evidence), audit the workspace. Audit doesn't accept immunity claims — it **observes** the evidence behind each presents-site and reports a per-site verdict:

```sh
cargo antigen audit
```

Each presents-site gets one of three verdicts:

- **`defended`** — evidence resolved; the verdict carries the **witness tier** of that evidence (see below)
- **`undefended`** — the site presents the failure-class but has no `#[defended_by]` witness and no resolving `requires =` predicate
- **`substrate-gap`** — a `requires =` predicate was declared but its substrate is missing or stale (e.g. an attestation sidecar absent or out of date)

A `defended` verdict is graded by **witness tier**, so the strength of evidence is honest and visible:

- **`Reachability`** — the witness function is reachable from a test entry-point
- **`Execution`** — the witness function is actually exercised by tests
- **`FormalProof`** — the witness is a formal proof artifact (e.g., a Kani harness)
- **`None`** — no *passing* evidence resolved

No more "we have tests" hand-waving — audit tells you which sites have *what kind* of evidence behind them.

See [`witness-tiers.md`](witness-tiers.md) for the full witness model.

---

## The headline: code IS the board

The vocabulary above (declare → scan → defend → audit) is antigen's core loop.
A second axis builds on it: **the prescriptive / work-orchestration family**.

Instead of a TODO comment that rots, you write a macro that stays current or emits
a loud verdict when it doesn't:

```rust
use antigen::panel;

#[panel(
    needs = ["review null-handling path", "confirm error message copy"],
    filled_by = ["alice"],
    reviewed_by = ["bob"],
    due = "2026-09-01",
)]
pub fn parse_user_input(raw: &str) -> Result<Input, ParseError> {
    // ...
}
```

`cargo antigen audit` renders each work-need's verdict — `Pending` / `Fulfilled` /
`Overdue` / `OutOfFrame` — as a live-projected board section alongside your defenses.
No separate dashboard; the code IS the board.

The eight macros route to four structural shapes:

| Shape | Macros | What it models |
|---|---|---|
| **S1 Role-workflow** | `#[panel]`, `#[rx]`, `#[refer]`, `#[biopsy]` | An ordered set of who-steps to fill and review |
| **S2 Elimination** | `#[ddx]` | Competing hypotheses to eliminate one by one |
| **S3 Ordering** | `#[triage]` | Re-validatable priority order over code-site references |
| **S4 Frame-only** | `#[culture]`, `#[quarantine]` | A temporal window with an expiry |

For the full reference see [`macros.md`](macros.md) — the prescriptive section.
For a complete worked example (a parser module carrying all eight macros with
time-stable verdicts), see [`examples-guide.md`](examples-guide.md). The example
compiles and prints a brief orientation message when run directly:

```sh
cargo run --example prescriptive_board --package antigen
```

The board itself — the live-projected per-site verdicts — renders via audit:

```sh
cargo run --bin cargo-antigen -- antigen audit --root antigen/examples
```

---

## Where to go next

- **[`tutorial.md`](tutorial.md)** — your first 15 minutes, end-to-end (declare → scan → defend → audit, with substrate-witness sidecars)
- **[`concepts.md`](concepts.md)** — what antigen IS, architecturally
- **[`examples-guide.md`](examples-guide.md)** — walks all examples in `antigen/examples/`, including the stdlib family surface and the prescriptive family (`prescriptive_board`)
- **[`where-to-look-for-antigens.md`](where-to-look-for-antigens.md)** — conventions for locating declarations
- **[`usage-patterns.md`](usage-patterns.md)** — common patterns + decision tables
- **[`witness-tiers.md`](witness-tiers.md)** — the witness model + tier ladder
- **[`macros.md`](macros.md)** — full reference for all macros (five core + eight prescriptive)
- **[`fingerprint-grammar.md`](fingerprint-grammar.md)** — fingerprint DSL reference
- **[`roadmap.md`](roadmap.md)** — what's shipped, what's coming
- **[`index.md`](index.md)** — full documentation map

If you're an LLM agent collaborating on a project: see [`for-llm-collaborators.md`](for-llm-collaborators.md) for the co-native protocol.

---

## In 5 minutes you've...

- Installed antigen and the `cargo antigen` subcommand
- Run scan against your codebase
- Declared your first failure-class with a structural fingerprint
- Seen passive detection surface real candidate sites
- Made structural failure-class memory part of your project

The lesson "drop impls must not panic" is now structurally present in your codebase. It will survive developer turnover, AI agent context cycling, time, and refactors — because it lives in the type system, not in human memory.

That's antigen at its floor. There's much more to build — substrate-witness sidecars (cross-state attestation), Oracle 5-state artifact lifecycle, descended_from inheritance chains, tier-honest audits — but the floor is real value from minute one.

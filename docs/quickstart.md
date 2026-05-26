# Antigen — 5-Minute Quickstart

> The fastest path from zero to running antigen in your project. For
> the full first-15-minutes walkthrough, see [`tutorial.md`](tutorial.md).
> For the conceptual framing, see [`concepts.md`](concepts.md).

---

## Step 1 — Install the cargo subcommand (30 seconds)

```sh
cargo install cargo-antigen
```

Verify:

```sh
cargo antigen --version
cargo antigen --help
```

You should see:

```
$ cargo antigen --version
cargo-antigen-antigen 0.1.0-rc.3

$ cargo antigen --help
The "antigen" subcommand of cargo

Usage: cargo antigen <COMMAND>

Commands:
  scan      Scan the workspace for antigen presentations and report unaddressed ones
  audit     Comprehensive immunity coverage report — witness resolution and tier validation
  attest    Manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019)
  tolerate  Manage tolerance-ratification sidecars (ADR-019 §tolerance tier)
  oracle    Manage Oracle artifact-class records (ADR-021 §D3)
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

You only need `scan` and `audit` for the quickstart. The other subcommands cover advanced workflows (substrate-witness sidecars, tolerance ratification, Oracle artifact lifecycle) — see [`witness-tiers.md`](witness-tiers.md) and the [tutorial](tutorial.md) when you're ready.

---

## Step 2 — Run scan on any Rust project (15 seconds)

```sh
cd /path/to/your/rust/project
cargo antigen scan
```

On a fresh codebase with no antigens declared yet:

```
Scanning workspace: .

Scanned N files, found 0 antigen-related declarations.
```

That's clean. Antigen ran; your project has no antigen surface yet. No red flags, no failures.

---

## Step 3 — Add antigen as a dependency (30 seconds)

```toml
# Cargo.toml
[dependencies]
antigen = "=0.1.0-rc.3"
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
  - 6 fingerprint matches (unmarked sites)
  - 0 tolerated sites (#[antigen_tolerance])
  - 0 immunity claims
  - 0 parse failures

6 fingerprint match(es) — structurally similar to a declared antigen:

  ./src/cleanup.rs:42  PanickingInDrop on impl
  ./src/resource.rs:117  PanickingInDrop on impl
  ...

To address each site, use the antigen type shown above:
  #[immune(<antigen>, witness = ...)] on the same item,
  OR #[antigen_tolerance(<antigen>, rationale = "...")]
```

The **fingerprint matches** are sites in your code that look structurally like the failure class you declared. Each one is a decision-point.

If your codebase doesn't have any matching sites, the output is clean. Either way, your antigen is now part of your project's structural memory.

---

## What just happened

You declared a named failure-class with a structural fingerprint. `cargo antigen scan` walks your codebase and tells you every site that structurally resembles the failure. The lesson — "drop impls must not panic" — now lives in your codebase as durable substrate, not in your head.

For each surfaced site, you have three choices:

- **`#[immune(PanickingInDrop, ...)]`** — claim the site is protected. Quick test: **can a test execute the thing you're defending?** Yes → `witness = some_test` (a test/proptest/proof exercises the behavior — e.g. drop and check no panic); No → `requires = signers(...)` or `requires = ratified_doc(...)` (evidence lives outside the code: a stale doc, an unpinned dep, an un-reviewed discipline). Full substrate-witness path: [`tutorial.md`](tutorial.md).
- **`#[antigen_tolerance(PanickingInDrop, rationale = "...")]`** — acknowledge the match is intentional or accepted, with required justification
- **Refactor** — eliminate the failure-class shape

That's the floor. Antigen meets you at this floor and grows with your practice.

---

## Step 6 — Audit your defenses (optional, 1 minute)

If you mark sites with `#[immune(...)]`, audit the workspace to see how strong each defense actually is:

```sh
cargo antigen audit
```

Each immunity claim is classified by **witness tier**:

- **`Reachability`** — the witness function is reachable from a test entry-point
- **`Execution`** — the witness function is actually exercised by tests
- **`FormalProof`** — the witness is a formal proof artifact (e.g., a Kani harness)
- **`None`** — no *passing* evidence: either no witness resolved, or a `requires =` predicate was evaluated and failed (the `audit_hint` says which)

This makes the strength of every defense honest and visible. No more "we have tests" hand-waving — audit tells you which sites have *what kind* of evidence behind them.

See [`witness-tiers.md`](witness-tiers.md) for the full witness model.

---

## Where to go next

- **[`tutorial.md`](tutorial.md)** — your first 15 minutes, end-to-end (declare → scan → immune → audit, with substrate-witness sidecars)
- **[`concepts.md`](concepts.md)** — what antigen IS, architecturally
- **[`examples-guide.md`](examples-guide.md)** — walks all nine bundled examples in `antigen/examples/`
- **[`where-to-look-for-antigens.md`](where-to-look-for-antigens.md)** — conventions for locating declarations
- **[`usage-patterns.md`](usage-patterns.md)** — common patterns + decision tables
- **[`witness-tiers.md`](witness-tiers.md)** — the witness model + tier ladder
- **[`macros.md`](macros.md)** — full reference for the five macros
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

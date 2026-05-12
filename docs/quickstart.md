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
cargo antigen --help
```

You should see:

```
Cargo antigen — structural failure-class memory for Rust

Usage: cargo antigen <COMMAND>

Commands:
  scan   Scan for antigen declarations and unaddressed presentations
  audit  Audit immunity claims across the workspace
  help   Print this message or the help of the given subcommand(s)
```

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

That's clean. Antigen ran; your project has no antigen surface yet. No
red flags, no failures.

---

## Step 3 — Add antigen as a dependency (30 seconds)

```toml
# Cargo.toml
[dependencies]
antigen = "=0.1.0-rc.1"
```

Run `cargo build` to fetch and compile. Antigen's runtime cost is zero
(the macros are identity transforms; no code generation).

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

Now you'll see your antigen declaration found, plus any sites in
your codebase that structurally match its fingerprint (impl Drop
blocks containing `panic!`, `unreachable!`, etc.).

If your codebase doesn't have any matching sites, the output is clean.
If it does, you'll see `[fingerprint match]` annotations pointing to
each one — your codebase's first surfaced structural failure-class
sites.

---

## What just happened

You declared a named failure-class with a structural fingerprint.
`cargo antigen scan` walks your codebase and tells you every site
that structurally resembles the failure. The lesson — "drop impls
must not panic" — now lives in your codebase as durable substrate,
not in your head.

For each surfaced site, you have three choices:

- **`#[immune(PanickingInDrop, witness = ...)]`** — claim the site is
  protected, name a witness (test, proptest, formal proof, lint,
  phantom-type) that verifies it
- **`#[antigen_tolerance(PanickingInDrop, rationale = "...")]`** —
  acknowledge the match is intentional or accepted, with required
  justification
- **Refactor** — eliminate the failure-class shape

That's the floor. Antigen meets you at this floor and grows with your
practice.

---

## Where to go next

- **[`tutorial.md`](tutorial.md)** — your first 15 minutes, end-to-end
  (covers `#[immune]`, witnesses, audit)
- **[`concepts.md`](concepts.md)** — what antigen actually IS,
  architecturally
- **[`where-to-look-for-antigens.md`](where-to-look-for-antigens.md)** —
  conventions for locating declarations in your project
- **[`usage-patterns.md`](usage-patterns.md)** — common patterns
- **[`macros.md`](macros.md)** — full reference for the five macros
- **[`fingerprint-grammar.md`](fingerprint-grammar.md)** — fingerprint
  DSL reference
- **[`index.md`](index.md)** — full documentation map

If you're an LLM agent: see
[`for-llm-collaborators.md`](for-llm-collaborators.md) for the
co-native protocol.

---

## In 5 minutes you've...

- Installed antigen
- Run scan against your codebase
- Declared your first failure-class
- Seen passive detection in action
- Made structural failure-class memory part of your project

The lesson "drop impls must not panic" is now structurally present in
your codebase. It will survive developer turnover, AI agent context
cycling, time, and refactors — because it lives in the type system,
not in human memory.

That's antigen at its floor. There's much more to build, but the floor
is real value from minute one.

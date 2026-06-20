# Wiring antigen into CI (and your editor)

> How to run `cargo antigen scan` and `cargo antigen audit` as CI gates: the
> exact exit codes, the scan-vs-audit-strict relationship, where the gate sits in
> a release cadence, a worked GitHub Actions snippet, and the **editor
> flycheck** integration (`--message-format json` → rust-analyzer).

---

## The two commands

Antigen gives you two read-only inspection commands. Neither mutates your code.

- **`cargo antigen scan`** — walks the workspace, surfaces every antigen
  declaration, explicit `#[presents]` site, fingerprint match, tolerated site,
  and `#[defended_by]` registration. It answers *"what failure-class memory does
  this codebase carry, and which sites are unaddressed?"*
- **`cargo antigen audit`** — drives the witness/verdict pipeline: which
  `#[presents]` sites are `defended` / `undefended` / `substrate-gap`, which
  witnesses resolve, which substrate predicates pass. It answers *"is the
  declared failure-class memory actually backed by evidence?"*

For CI, `audit --strict` is the load-bearing gate (see
[the relationship](#scan-strict-vs-audit-strict) below). `scan` is most useful
as a non-gating informational step or for the JSON artifact.

---

## Exit codes

Both commands follow the same three-value convention. The process exit code is
exactly the `run_scan` / `run_audit` return value — `main()` returns it directly.

| Exit | Meaning |
|---|---|
| **0** | Success. (Default, non-strict runs **always** exit 0 — see below.) |
| **1** | Gate failure under `--strict` (a real, unaddressed defect was found). |
| **2** | Tooling error — bad path, unreadable workspace, malformed `--category` filter, `cargo metadata` failure, serialization error. Not a defense verdict; a "couldn't run" signal. |

### The warn-not-error default (ADR-008 Amendment 1)

**Without `--strict`, both `scan` and `audit` exit 0 even when there are
unaddressed presentations.** This is deliberate. A fresh codebase that has just
adopted antigen will have many fingerprint matches and unaddressed sites; failing
CI on day one would punish adoption. The default lets adopters make a conscious
decision about each site instead of being blocked. Verified:

```text
cargo antigen scan   (undefended site present)  → exit 0
cargo antigen audit  (undefended site present)  → exit 0
```

### What `--strict` gates on

`--strict` is what turns the inspection into a CI gate. Verified:

```text
cargo antigen scan  --strict  (undefended explicit presentation)  → exit 1
cargo antigen audit --strict  (undefended verdict)                → exit 1
cargo antigen audit --root <nonexistent>                          → exit 2
```

- **`scan --strict`** exits 1 when there are unaddressed **explicit**
  `#[presents]` markers, **orphaned tolerances**, or **broken lineage edges** (a
  `#[descended_from]` pointing at a non-existent parent, or on a non-antigen
  child). Bare *fingerprint matches* (candidate sites, not explicit markers) do
  **not** gate — they are expected noise that the witness layer refines, so
  gating on them would fail CI with a confusing message.
- **`audit --strict`** exits 1 on a **superset** of the scan gates plus the
  verdict gates:
  - an **undefended** presents-site (a `#[presents]` with no `#[defended_by]`
    witness and no passing `requires =` predicate — an open defense circuit),
  - a **state-7** inherited-and-unaddressed presentation (ADR-018),
  - any immunity witness **below the Reachability tier**,
  - **orphaned tolerances** and **broken/dangling lineage edges** (lifted from
    the scan gate so `audit --strict` alone is a complete CI gate).

  Note: a **`substrate-gap`** verdict (intent declared via `requires =` but no
  signed sidecar yet) is reported loudly but does **not** gate under `--strict` —
  the intent exists; it warrants a warning, not a hard fail, until per-antigen
  severity lands.

---

## `scan --strict` vs `audit --strict`

`audit --strict` is engineered to be a **superset** of `scan --strict`: every
structural-defect gate that scan enforces (orphaned tolerances, orphaned/dangling
lineage edges) is also enforced by audit, *plus* the verdict gates (undefended
sites, state-7, below-tier witnesses).

**Practical consequence: a single `cargo antigen audit --strict` is a complete CI
gate.** You do not need to run `scan --strict` separately to catch the structural
defects — audit covers them. Run `scan` (non-strict) only if you also want the
informational inventory or the JSON artifact.

---

## A worked GitHub Actions snippet

```yaml
name: antigen
on: [push, pull_request]

jobs:
  antigen:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      # Install the cargo subcommand (pin the version for reproducible CI).
      - name: Install cargo-antigen
        run: cargo install cargo-antigen --locked

      # Informational: full inventory + a machine-readable artifact. Non-strict,
      # so this step never fails the build on its own.
      - name: antigen scan (report)
        run: cargo antigen scan --output antigen-report.json

      - uses: actions/upload-artifact@v4
        with:
          name: antigen-report
          path: antigen-report.json

      # The gate. audit --strict is a superset of scan --strict, so this one
      # step enforces undefended sites, state-7, below-tier witnesses, orphaned
      # tolerances, and broken lineage. Exit 1 fails the job.
      - name: antigen audit (gate)
        run: cargo antigen audit --strict
```

The `--output <file>` flag on `scan` writes the full JSON render regardless of
the console format, so CI can print the human summary **and** save the machine
detail as an artifact in the same step. The file is a *render of that run* —
recomputed each run, never read back as authoritative — so running it at a tagged
commit produces that tag's defense-posture snapshot reproducibly.

> **`audit --output`.** `cargo antigen audit` also accepts `--output <file>` for
> the same reproducible-render reason. Running `cargo antigen audit --output
> defense-posture.json` at a tagged commit yields that tag's full defense-posture
> report — a regenerable SBOM-style artifact.

---

## Where the gate sits in a release cadence

Antigen's gate composes with — and runs alongside — the standard Rust CI gates.
A typical pre-release sequence:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo antigen audit --strict        #  ← the antigen gate
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

Placement guidance:

- **On every push / PR** — run `audit --strict` as a required check, the same as
  clippy and tests. This catches a newly-introduced undefended site (or a broken
  `#[descended_from]` edge) at review time, before it lands.
- **Before a release tag** — `audit --strict` is part of the pre-tag gate. A
  green audit at the release commit means every declared failure-class is
  backed by a wired witness or an explicit tolerance. Optionally capture
  `cargo antigen audit --output defense-posture.json` as a release artifact.

> **Supply-chain family only (skip if you don't use it).** `cargo antigen verify
> maintainer-changes` has a hard sequencing constraint: it **must run before
> `cargo update`**. After `cargo update`, the new maintainer's code is already in
> `Cargo.lock` and the gate has effectively already passed. Document the ordering
> in any CI script that combines the two.

---

## Tier honesty in CI (what a green gate does and doesn't claim)

A passing `audit --strict` means every `#[presents]` site is *defended* — the
witness circuit is **wired** (Reachability tier). It does **not** mean every
witness test was *executed* and *passed*: the audit does not run the witness, so
keep `cargo test` as its own CI step — antigen confirms the *defense is in place*;
`cargo test` confirms it *holds*. The two gates are complementary, not redundant.
(Execution-tier gating — the audit running the witness itself — is a recorded
graduation path; see [`roadmap.md`](roadmap.md).)

---

## Editor / IDE integration — flycheck (`--message-format json`)

CI is the gate; the editor is where the feedback is cheapest. antigen lets
`cargo antigen scan` speak the **rustc / cargo `--message-format=json`
line-protocol** — newline-delimited `compiler-message` objects, the exact shape
rust-analyzer's flycheck already consumes. Point the editor's check command at
antigen and fingerprint matches render inline as warning squiggles, on save, with
**no custom LSP server and no plugin**.

```sh
cargo antigen scan --message-format json
```

Wire it into rust-analyzer (`.vscode/settings.json`, or your editor's
equivalent):

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "antigen", "scan", "--message-format", "json"
  ]
}
```

Every diagnostic emits at **`warning` level only** — antigen never fails your
build from the editor — and carries the claim-scope verbatim: *"This is a
fingerprint match to inspect, not an audited verdict."* The `code.code` is
namespaced `antigen::<class>` so the editor groups antigen's warnings, and a
child `note` tells you the remediation (`#[presents]` + `#[defended_by]`, or
`#[antigen_tolerance]`). Full field reference + a real diagnostic in
[`output-formats.md`](output-formats.md).

Add `--bundled-catalog` to the override command if you want the shipped stdlib
footgun shapes flagged on a crate that already declares its own antigens:

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "antigen", "scan", "--bundled-catalog", "--message-format", "json"
  ]
}
```

> **`overrideCommand` replaces the default `cargo check`.** rust-analyzer runs
> *one* check command. If you point it solely at `cargo antigen scan`, you lose
> the normal `cargo check` diagnostics. Either keep antigen as a CI/manual step,
> or use a wrapper that runs `cargo check` *and* `cargo antigen scan
> --message-format json` and concatenates the JSON lines. The two are
> complementary: `cargo check` proves it compiles; antigen flags failure-class
> shapes.

---

## See also

- [`composition.md`](composition.md) — "Antigen + CI" and how antigen composes
  with your existing test/lint/verification gates
- [`immune-migration-guide.md`](immune-migration-guide.md) — if your codebase
  still has `#[immune]` sites the audit flags as deprecated
- [`decisions.md`](decisions.md) — the warn-not-error default, state-7
  inheritance, and observe-don't-declare verdicts

---

*Non-strict inspects; `--strict` gates. One `cargo antigen audit --strict` is the
whole CI surface — a superset of every scan-side structural check plus the
defense verdicts.*

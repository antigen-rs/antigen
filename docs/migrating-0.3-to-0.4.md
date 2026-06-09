# Migrating from 0.3 to 0.4

If you adopted antigen on 0.3, nothing you wrote needs to change. The 0.3 surface —
the macros, the `scan` and `audit` commands, the witness model, the sidecars — is
preserved. 0.4 is additive: it makes the immune surface discoverable and closes the
one honesty gap in the scan path. This page is the short list of what's new and how
to turn it on.

## Nothing breaks

Your `#[antigen]`, `#[presents]`, `#[defended_by]`, `#[antigen_tolerance]`, and
marked-unknown (`#[aura]` / `#[dread]` / `#[red_flag]`) declarations all behave as
they did on 0.3. Your `.attest/` sidecars are unchanged. `cargo antigen scan` and
`cargo antigen audit` take the same flags and produce the same reports. Update the
version and your build is green — the rest of this page is opt-in.

## What's new, and how to use it

### Your scans stop reporting a false all-clear

On 0.3, a crate with no antigen declarations of its own had an empty repertoire, so
a scan had nothing to match and reported as if the code were clean — indistinguishable
from "found nothing to look at." On 0.4, antigen ships a **bundled catalog** of its
flagship failure-class fingerprints, and a bare scan reaches for it automatically
when your crate declares no antigens:

```sh
cargo antigen scan
```

On a zero-declaration crate this now surfaces real fingerprint matches against the
bundled classes. To inject the catalog *on top of* your own declarations (augment
mode), pass the flag explicitly:

```sh
cargo antigen scan --bundled-catalog
```

The library entry-point is `antigen::scan::scan_workspace_bundled_catalog`. A
worked example is in [`examples/bundled_catalog_scan.rs`](../antigen/examples/bundled_catalog_scan.rs).

These are **scan-facts** — a fingerprint match says "this shape matches a known
class," not "this is broken" and not "a defense was audited." The claim-scope is
unchanged from 0.3; the bundled catalog just gives a zero-declaration crate
something to match against.

### Findings in your editor

`cargo antigen scan --message-format json` emits the rustc/cargo JSON line-protocol,
so an editor's flycheck consumes antigen findings as compiler diagnostics. Point
rust-analyzer's `check.overrideCommand` at it and matches render inline as warnings
— no custom LSP server:

```jsonc
// .vscode/settings.json (or your editor's rust-analyzer config)
"rust-analyzer.check.overrideCommand": [
  "cargo", "antigen", "scan", "--message-format", "json"
]
```

Findings emit at `warning` level only — antigen never fails your build from the
editor. See [editor-integration.md](editor-integration.md) for the full wiring.

This is distinct from `--format json`, which writes antigen's own report envelope.
`--format json` is unchanged from 0.3; `--message-format json` is the new
editor-shaped surface.

### The learning core (library API)

0.4 ships the cluster → propose → promote loop as a library API (`antigen::learn`),
governed by a self-tolerance gate. Given a cluster of structurally-similar marked
sites, `propose()` anti-unifies them into a draft fingerprint and promotes it only
if it spares a clean corpus. It is a **library API, not a CLI command** — there is
no `cargo antigen propose`. A draft is a hypothesis to ratify, never an
auto-asserted class.

If you only use the CLI, this changes nothing for you today; it is the substrate the
next cycle wires. If you want to drive it directly, the keystone example is
[`examples/learn_propose.rs`](../antigen/examples/learn_propose.rs), and the
reasoning is in [the-keystone-explained.md](the-keystone-explained.md).

## If you're coming from an older deprecation

The `#[immune(...)]` → `#[defended_by]` / `#[presents(requires = ...)]` move is a
0.2-era change, not a 0.4 one. If you still have `#[immune]` sites, the dedicated
[immune-migration-guide.md](immune-migration-guide.md) walks that conversion. 0.4
does not change it.

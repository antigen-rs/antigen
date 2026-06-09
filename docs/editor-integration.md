# Editor integration — antigen findings as inline diagnostics

> Wire `cargo antigen scan` into your editor so fingerprint matches appear as
> warning squiggles, on save, with no custom LSP server and no plugin. This page
> is the canonical reference for the `--message-format json` surface: what it
> emits, how to point an editor at it, and how the same line-protocol feeds CI.
>
> For a five-minute taste of editor wiring inside the onboarding walk, see
> [`getting-started.md`](getting-started.md#5-put-it-in-your-editor-flycheck).

---

## What it does

`cargo antigen scan --message-format json` emits findings in the **cargo/rustc
`--message-format=json` line-protocol** — newline-delimited `compiler-message`
objects, one per fingerprint match, in the exact shape rust-analyzer's flycheck
already understands. Because the protocol is the one editors already speak for
compiler diagnostics, an editor renders antigen findings inline with nothing
antigen-specific installed: point the editor's check command at the scan and
matches show up as warnings next to the code.

This is a different surface from `--format json`:

| Flag | Emits | Consumer |
|---|---|---|
| `--message-format json` | rustc line-protocol (newline-delimited `compiler-message`) | editors that speak compiler diagnostics |
| `--format json` | antigen's own report envelope (one object for the whole run) | scripts and CI that read antigen's structure |

Same findings, two shapes, two consumers. See
[`output-formats.md`](output-formats.md) for the `--format json` envelope.

---

## The output

Run it once to see the shape. Against a crate with a single footgun — a
`.unwrap()` inside a `Drop` impl — the scan emits one object:

```sh
cargo antigen scan --message-format json
```

```json
{"reason":"compiler-message","message":{"message":"antigen: structure matches the `panic-in-drop` failure-class fingerprint (provenance: constructable (a verified minimal case exists)). This is a fingerprint match to inspect, not an audited verdict.","level":"warning","code":{"code":"antigen::panic-in-drop","explanation":null},"spans":[{"file_name":"src/lib.rs","line_start":7,"line_end":7,"column_start":1,"column_end":1,"is_primary":true}],"children":[{"message":"fingerprint match only — antigen has not audited a defense for this site. Mark it with #[presents(panic-in-drop)] + #[defended_by(...)] to record the defense, or #[antigen_tolerance(panic-in-drop, rationale=...)] to accept it.","level":"note","code":null,"spans":[],"children":[],"rendered":null}],"rendered":"warning: antigen: structure matches the `panic-in-drop` failure-class fingerprint (provenance: constructable (a verified minimal case exists)). This is a fingerprint match to inspect, not an audited verdict."}}
```

One match per line. A crate with two footguns emits two lines; a clean crate
emits nothing. The protocol is line-delimited, so an editor consumes each object
as it arrives.

### The load-bearing fields

| Field | Value | Why it matters |
|---|---|---|
| `reason` | `"compiler-message"` | what makes an editor treat the object as a diagnostic at all |
| `message.level` | **`"warning"` — always, never `error`** | antigen never fails the build; a match is a candidate to inspect |
| `message.code.code` | `antigen::<class-name>` | namespaced so the editor groups antigen's diagnostics apart from rustc's |
| `message.message` | carries the verbatim claim-scope: **"This is a fingerprint match to inspect, not an audited verdict."** | the diagnostic states its own scope, per finding |
| `message.spans[]` | `file_name` + `line_start` (`is_primary: true`) | where the editor draws the squiggle — at the start of the matched item's line, not the exact token |
| `message.children[]` | a `note`-level child | the remediation hint: `#[presents]` + `#[defended_by]`, or `#[antigen_tolerance]` |

The `provenance` clause in the message text (`constructable`, `encountered`,
`imagined`) records how the failure-class was established — see
[`witness-tiers.md`](witness-tiers.md). The span column is reported as the line
start, so the squiggle marks the line rather than a precise column range.

---

## Wire it into rust-analyzer

rust-analyzer runs a check command on save and parses its `--message-format=json`
output. Override that command to point at antigen. In your editor's
rust-analyzer settings (`.vscode/settings.json`, or the equivalent for Neovim,
Helix, Emacs — any editor whose Rust support is rust-analyzer):

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "antigen", "scan", "--message-format", "json"
  ]
}
```

Now every save runs the scan and each fingerprint match renders as a warning
squiggle, with the claim-scope text in the hover.

To also flag the shipped stdlib footgun shapes on a crate that *already* declares
its own antigens, add `--bundled-catalog`:

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "antigen", "scan", "--bundled-catalog", "--message-format", "json"
  ]
}
```

Without the flag, the bundled catalog auto-injects only when a crate has no
antigens of its own; the explicit flag augments a crate that does. See
[`stdlib-families.md`](stdlib-families.md) for what the catalog checks you
against.

### `overrideCommand` replaces `cargo check`

rust-analyzer runs *one* check command. An `overrideCommand` pointed solely at
`cargo antigen scan` replaces `cargo check`, so the editor stops showing the
normal compiler diagnostics. Two ways to keep both:

- **Keep antigen out of the editor loop.** Run it as a manual command or a CI
  gate, and leave rust-analyzer on its default `cargo check`.
- **Run both and concatenate.** Point `overrideCommand` at a small wrapper that
  runs `cargo check --message-format json` *and* `cargo antigen scan
  --message-format json`, emitting both streams. The two are complementary:
  `cargo check` proves the code compiles; antigen flags failure-class shapes.

---

## It never fails your build

Every diagnostic emits at `warning` level, and the message-format surface always
exits `0` — even under `--strict`. `--strict` gates *CI* exit codes on the
human/envelope path; in `--message-format json` mode the scan emits the
diagnostics and returns, so the editor never sees an error and a save is never
blocked. A fingerprint match is a candidate, full stop.

---

## Expect candidate density

flycheck draws one squiggle per fingerprint match, and squiggles *read* as
errors-to-fix in a way console output does not. On a repo with broad
`suspected`-tier fingerprints that can be **tens of thousands** of candidates —
antigen's own tree emits over 25,000 on a bare scan, and `--bundled-catalog`
pushes it higher. Scope the check before wiring it on a large or
already-instrumented tree:

- `--category functional-correctness` (or `substrate-alignment`) to one category,
- `--root <subdir>` to a single part of the tree,

and read [`reading-a-verdict.md`](reading-a-verdict.md#tiers) on tiers before
treating squiggles as a to-do list. A `suspected`-tier match is "look here," not
"this is broken." The [`troubleshooting.md`](troubleshooting.md) firehose entry
covers the noisy case end to end.

---

## CI reads the same protocol

The line-protocol that an editor consumes is the same one a CI job can parse — it
is the rustc shape, so any tool that already reads cargo's `--message-format=json`
(annotators, problem matchers, diagnostic aggregators) reads antigen's findings
unchanged. To gate CI on antigen's verdict rather than annotate it, use the
human/envelope path with `--strict` instead; see
[`deployment-ci-integration.md`](deployment-ci-integration.md) for the CI story.

---

## See also

- [`getting-started.md`](getting-started.md) — the onboarding walk; §5 wires the
  editor in the middle of a first session.
- [`output-formats.md`](output-formats.md) — the `--format json` report envelope
  and every output surface.
- [`deployment-ci-integration.md`](deployment-ci-integration.md) — gating CI on
  the scan.
- [`reading-a-verdict.md`](reading-a-verdict.md) — what a fingerprint match means
  and how tiers calibrate it.

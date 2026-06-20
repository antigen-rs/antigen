# doc-harness — the documentation example-harness

A non-reproducing example is a lie that compiles in the reader's head. This
harness is the mechanical check that every shell example in antigen's docs still
matches the real binary — run it once per release instead of re-proving every
example by hand.

## What it does

For every documentation file (`docs/**/*.md` plus the five crate READMEs), it:

1. **Extracts** each `cargo antigen` example, in either of the two conventions
   the docs use:
   - **command-then-output** — an ` ```sh ` fence holds the command, the fence
     right after holds the claimed output ("Verify:" / "You should see:").
   - **self-contained transcript** — a single ` ```text ` fence whose first line
     is a `$ cargo antigen ...` prompt with the output below it. Crate READMEs
     use this.
2. **Classifies** each command:
   - **RUNNABLE-HERE** — deterministic in this repo with no user project, no
     network, no mutation: every `--help` surface and `--version`. These are run
     for real and diffed against their claimed output.
   - **ILLUSTRATIVE** — needs a world the harness can't conjure (`cargo install`,
     `cd /path/to/your/project`, `scan` against an unspecified tree). These are
     **surfaced** (so they're visible, never silently skipped) but not asserted
     byte-equal.
3. **Reports** every drift as a located finding: `doc:line` plus a unified diff
   of claimed-vs-actual, so a writer fixes the exact spot.

A self-contained transcript may legitimately show an **excerpt** — a README
routinely trims the `Usage:`/`Options:` apparatus and drops rows. If the claimed
output is an exact-line-*subsequence* of the real output (every claimed line
appears, in order), it's reported as **EXCERPT**, not drift. A *modified* line —
an abbreviated description the binary doesn't emit verbatim — breaks the
subsequence and surfaces as drift, because that's a claim the binary doesn't
back. EXCERPT does not gate; DRIFT does.

## Run it

```sh
cargo build -p cargo-antigen          # the harness needs the real binary
python tools/doc-harness/run.py       # human report
python tools/doc-harness/run.py --json  # machine-readable
```

Flags: `--bin PATH` (override the binary), `--docs-root DIR` (override the repo
root). Exit code `1` if any RUNNABLE-HERE example drifted, `0` if all reproduced,
`2` on harness error (binary not found).

## Two design decisions worth knowing

**It decodes the binary's output as UTF-8 explicitly.** antigen's help strings
carry em-dashes and section-signs. Python's `subprocess(text=True)` decodes with
the platform locale — cp1252 on Windows — which mangles `—` into mojibake and
produces a *false* drift on every line that contains one. A harness that cries
drift on every help block trains writers to ignore it. So the harness captures
bytes and decodes UTF-8, and the diff shows only what genuinely changed.

**It masks the version string.** The version (`0.5.0-beta.1`) is volatile — it
changes at release, and the CHANGELOG is its carrier, not the help output. The
harness normalizes any `x.y.z[-tag]` to `<VERSION>` before comparing, so a
pre-bump binary doesn't false-positive every block that happens to print a
version.

## The prose-linter (`prose_lint.py`)

The harness proves every *example* runs. The prose-linter proves every *claim in
the prose* holds the line — the second half of the standing machinery the mandate
names ("no test-counts, no internal names, no (planned), every fence
verified-run"). It is antigen's own thesis turned on its own docs: a claim whose
truth silently flips when the world moves — a version tags, a role-name leaks out
of the team, a promised feature ships — is the silent-failure class. A flagged
line is a fingerprint match to inspect, not an audited verdict; some hits are
load-bearing and a human keeps them.

```sh
python tools/doc-harness/prose_lint.py            # all rules
python tools/doc-harness/prose_lint.py --rule version-pin --rule us-leak
python tools/doc-harness/prose_lint.py --json
```

Rules: `version-pin` (a hard pre-release pin — the CHANGELOG carries versions),
`us-leak` (internal project/person/role/tool names in a USER doc), `planned-future`
(a `(planned)` promise in present-tense prose), `test-count` (a bare count as
performed evidence). It is **audience-aware**: `docs/internal/**` and
`docs/decisions.md` are contributor docs — internal by design, so ADR-numbers and
role-names there are not leaks — but a personal name and a stale version pin leak
regardless of audience. Every rule is conservative: a noisy linter is one writers
learn to ignore.

Together the two tools make "the docs are COMPLETE · CHECKED · COHESIVE" a thing a
CI job can assert, not a heroic human pass that decays the moment a subcommand is
added.

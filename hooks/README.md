# antigen git hooks

The report-as-live-projection, delivered at **commit time** — `cargo antigen`
running as a lint before each commit.

## The model: the report is a live projection, never a stored truth

A stored, release-anchored report is itself a `ParallelStateTrackersDiverge`
instance — antigen's own failure-class. The moment a report is committed it
becomes a second copy of the truth that can drift from the code it describes.

So antigen never persists a report it reads back as authoritative. `scan` /
`audit` recompute the report from the **current** code on every invocation,
exactly the way clippy reflects current source every time it runs. The hook is
just that recomputation, wired to the `pre-commit` git event. It cannot go
stale because there is nothing stored to go stale.

(When you want a saved render — for a code review, a CI artifact, a release
SBOM — use `cargo antigen scan --output report.json` or `audit --output`. That
file is a *render of a run* with a provenance envelope (`antigen_version`,
`git_sha`, `generated_at`, `report_schema_version`), regenerable any time by
re-running antigen at that commit. antigen never reads it back, so it cannot
drift.)

## Enforcement tier: friction-only

This is a **client-side** hook. That makes it
**friction-only**: it turns shipping an undefended failure-class into a
*deliberate* act (`git commit --no-verify`) rather than an *accidental* one. It
does **not** prevent a determined bypass, and it is **not** a substitute for the
CI gate.

If you need structural (non-bypassable) enforcement, gate in CI — run
`cargo antigen scan --strict` as a required status check, where `--no-verify`
does not reach.

## Install (opt-in)

antigen never writes into your `.git/` for you. Pick one:

```sh
# Symlink — the hook stays in sync with this repo copy:
ln -s ../../hooks/pre-commit .git/hooks/pre-commit

# Copy:
cp hooks/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

# Or point git at the whole dir (git 2.9+):
git config core.hooksPath hooks
```

## What it blocks

`pre-commit` runs `cargo antigen scan --strict` and (unless
`ANTIGEN_HOOK_SKIP_AUDIT=1`) `cargo antigen audit --strict`. It blocks the
commit when scan/audit would fail their strict gate:

- an **explicit** presentation left undefended,
- an **orphaned tolerance**,
- a **broken `#[descended_from]`** lineage edge,
- an immunity claim **below the required witness tier**, an undefended
  presents-site, or a state-7 inherited-unaddressed presentation (audit).

Fingerprint-match *candidates* are informational and never block — they are
expected noise the witness layer refines, not commit-blocking TODOs.

## Config

| Variable | Effect |
| --- | --- |
| `ANTIGEN_HOOK_SKIP_AUDIT=1` | run `scan` only, skip the `audit` pass |
| `ANTIGEN_HOOK_ROOT=<path>` | scan/audit root (default: repo top level) |
| `ANTIGEN_HOOK_DISABLE=1` | no-op the hook for one invocation |

Bypass when you mean it: `git commit --no-verify`.

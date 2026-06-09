# CLI reference — `cargo antigen`

> The whole command surface in one place: what each subcommand does, in one line,
> and where to read more. This page is the map, not the manual — for the full
> flags of a command, run `cargo antigen <command> --help` or follow the link.
>
> New here? Start with [`getting-started.md`](getting-started.md) (your first
> session) or [`quickstart.md`](quickstart.md) (the five-minute taste). You only
> need `scan` and `audit` to get real value; the rest are there when you reach for
> them.

```sh
cargo antigen <command>
```

| Command | What it does |
|---|---|
| [`scan`](#scan) | Walk the workspace and report sites that match a known failure-class fingerprint. |
| [`audit`](#audit) | Grade the defenses: which presented sites have a resolving witness, and how strong. |
| [`fingerprint`](#fingerprint) | Print the structural fingerprint of a scanned item. |
| [`attest`](#attest) | Manage substrate-witness sidecars — the recorded evidence a defense points at. |
| [`tolerate`](#tolerate) | Manage tolerance records — sites accepted on purpose, with a rationale. |
| [`oracle`](#oracle) | Manage oracle records — discipline artifacts with a lifecycle and a steward. |
| [`verify`](#verify) | Run the supply-chain checks (dependency pinning, attestation, content hashes). |
| [`vcs`](#vcs) | Observe version-control information-loss risks (rollback, branch-archive). |
| [`mucosal-map`](#mucosal-map) | Map the trust boundaries in the workspace and which are undefended. |

The first three are the everyday verbs. The rest drive specific failure-class
families and are reached when you adopt those families.

---

## The everyday verbs

### `scan`

Walk the workspace and report every site whose **structure matches a known
failure-class fingerprint**. A scan reports candidates to inspect, not verdicts —
a match means "this code's shape resembles a known class," not "this is a bug." A
crate with no antigen declarations of its own still gets findings: the bundled
stdlib catalog auto-injects so an empty repertoire is not a false all-clear.

```sh
cargo antigen scan
```

Output, exit codes, the `--bundled-catalog` flag, and the JSON surfaces are in
[`output-formats.md`](output-formats.md). To wire scan findings into your editor
as inline warnings, see [`editor-integration.md`](editor-integration.md). To read
a finding line by line, see [`reading-a-verdict.md`](reading-a-verdict.md).

### `audit`

Produce the **coverage report**: for each presented site, whether a witness
resolves the defense and at what tier (from a reachability test up to a formal
proof). Where `scan` asks "what shapes are here?", `audit` asks "are the declared
defenses actually backed by evidence?" Use `--strict` to make a missing or weak
defense fail CI.

```sh
cargo antigen audit
```

Witness tiers are explained in [`witness-tiers.md`](witness-tiers.md); CI gating
in [`deployment-ci-integration.md`](deployment-ci-integration.md).

### `fingerprint`

Print the **structural fingerprint** of a scanned item — the same digest a scan
computes, exposed as its own verb so you can obtain a fingerprint without
scaffolding first (for signing a sidecar, hand-editing one, or scripting). Narrow
the scan with `--antigen` and `--item-path`.

```sh
cargo antigen fingerprint --item-path src/lib.rs
```

The fingerprint grammar is in [`fingerprint-grammar.md`](fingerprint-grammar.md).

---

## The family drivers

These commands back specific failure-class families. You reach for one when you
adopt the family it serves; `audit` routes a site's `requires = ...` predicate to
the matching driver.

### `attest`

Manage the substrate-witness sidecars — the recorded evidence (a test result, a
verification artifact) that a `#[presents(..., requires = ...)]` defense points
at. `scaffold` creates a sidecar, `sign` records its digest. The witness model is
in [`witness-tiers.md`](witness-tiers.md).

### `tolerate`

Manage tolerance records: a site you accept on purpose, with a written rationale,
rather than defending. The accepted-on-purpose move is one of the three honest
responses to a candidate (the others being defend and refactor — see
[`getting-started.md`](getting-started.md)).

### `oracle`

Manage oracle records — discipline artifacts with an explicit lifecycle (draft →
complete → deprecated/retired/revoked) and a named steward. An oracle captures a
piece of ground truth a failure-class is checked against.

### `verify`

Drive the supply-chain checks: dependency pinning, dependency attestation,
content-hash recording and verification, and maintainer snapshots. These back the
supply-chain antigens; `audit` walks the `requires = ...` predicates and routes
them here.

### `vcs`

Observe version-control information-loss risks. The observation verbs surface
risk across the repo (`scan`), evaluate a single commit (`check-commit`), and
record attestations for rollbacks and branch deletions. These *observe* the git
substrate; they do not install hooks.

### `mucosal-map`

Map the **trust boundaries** in the workspace and report which are undefended.
`--undefended` lists boundaries with no mucosal declaration; `--tolerant` lists
the ones running on active tolerance (for periodic review); `--kind` filters to
one boundary kind.

---

## Exit codes

Every command shares the same exit-code contract:

| Code | Meaning |
|---|---|
| `0` | Success — the command ran. Findings are informational; antigen never fails a build on its own |
| `1` | A `--strict` gate tripped — `scan` found an unaddressed **explicit** `#[presents]` presentation (or an orphaned tolerance / broken `#[descended_from]` lineage); `audit` found an unresolved witness; a `verify` strict check found an unpinned dep |
| `2` | A usage error — the `--root` path is missing or not a directory, or an argument failed to parse |

`--strict` gates only on **explicit** presentations and unresolved witnesses.
Fingerprint matches are candidates to inspect, not failures — a scan that prints
matches but has every explicit presentation addressed exits `0`, even under
`--strict`. Strict mode is the opt-in CI gate; without it, findings are reported
and the run exits `0`.

---

## See also

- [`getting-started.md`](getting-started.md) — your first session, narrated.
- [`tutorial.md`](tutorial.md) — declare and defend your own first failure-class.
- [`output-formats.md`](output-formats.md) — scan/audit output, human and JSON.
- [`macros.md`](macros.md) — the attribute macros the CLI scans for.
- [`glossary.md`](glossary.md) — every term anchored.

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
| [`propose`](#propose) | Draft a candidate fingerprint from a cluster of marked unknowns, gated against an operator-supplied clean corpus. |
| [`attest`](#attest) | Manage substrate-witness sidecars — the recorded evidence a defense points at. |
| [`tolerate`](#tolerate) | Manage tolerance records — sites accepted on purpose, with a rationale. |
| [`oracle`](#oracle) | Manage oracle records — discipline artifacts with a lifecycle and a steward. |
| [`verify`](#verify) | Run the supply-chain checks (dependency pinning, attestation, content hashes). |
| [`vcs`](#vcs) | Observe version-control information-loss risks (rollback, branch-archive). |
| [`mucosal-map`](#mucosal-map) | Map the trust boundaries in the workspace and which are undefended. |

The first three are the everyday verbs. [`propose`](#propose) is the learning
verb — reach for it when a cluster of marked unknowns wants generalizing into a
candidate fingerprint. The rest drive specific failure-class families and are
reached when you adopt those families.

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

## The learning verb

### `propose`

Draft a candidate failure-class fingerprint from a **cluster of marked unknowns**
(`#[dread]` / `#[aura]` / `#[red_flag]` sites), gated so the draft never flags
known-good code. This is the learning core's CLI surface: it
[anti-unifies](glossary.md#anti-unify) the cluster into a draft, routes it through
the [self-tolerance gate](glossary.md#gate-g-the-self-tolerance-gate) against an
operator-supplied [clean corpus](glossary.md#clean-corpus), and **renders the
outcome as a ratifiable suggestion** — never an auto-`#[presents]`, never a named
class. A `propose` run leaves the source tree **byte-unchanged**; it observes, it
does not write markers ([observe-don't-declare](glossary.md#observe-dont-declare)).

```sh
cargo antigen propose --cluster-root <PATH> --clean-root <PATH>
```

The two roots are trust-distinct. `--cluster-root` (default: `.`) is where the
marked *defect* sites live. `--clean-root` is **required** and has no default:
antigen never labels unmarked code clean, so the operator supplies and labels the
known-good corpus the gate spares against. The gate is only as strong as that
corpus — a corpus-bounded check, not a total guarantee. (This is forced, not a
shortcut: deciding whether an arbitrary fingerprint over-flags all clean code
everywhere is undecidable, so the human supplies the corpus and ratifies the
result.)

| Flag | Default | Meaning |
|---|---|---|
| `--cluster-root <PATH>` | `.` | Root scanned for the marked defect cluster. |
| `--clean-root <PATH>` | *(required)* | Operator-supplied, operator-labeled clean corpus the gate spares against. |
| `--marker <dread\|aura\|red-flag>` | `dread` | Which marker-class is the defect cluster. A cluster mixes one class only. |
| `--format <human\|json>` | `human` | Output format for a *gated* outcome (see the JSON note below). |

On antigen's own source today, a `propose` run lands on **no cluster** — the repo's
`#[dread]` marks are singletons in shape-space (no two share a structural shape to
anti-unify), which propose needs at least two of:

```sh
cargo antigen propose --cluster-root antigen/src --clean-root antigen/src
```
```
no `dread` cluster found under antigen/src — propose needs ≥2 marked sites sharing a structural shape to anti-unify (found 0). Antigen's own marks are singletons in shape-space today; auto-clustering heterogeneous marks is the v0.6 abstract-recall frontier.
```

To see the *interesting* outcomes — a draft routed to a human, or a refusal — run
the bundled demo, which constructs a real twin-cluster:
[`examples/propose-demo/`](../examples/propose-demo/) (walked through in
[`examples-guide.md`](examples-guide.md)).

Every outcome is legible and exits `0` — a non-promotion is never an error:

| Outcome | What it means |
|---|---|
| candidate suggestion | The draft passed all three gate checks. Printed as a ratifiable suggestion (with its fingerprint and score tier) for you to inspect and name by hand — never an auto-named class. |
| routed to a human | The draft is *safe* (it spares your corpus, carries a discriminating signal) but your corpus has **no [near-miss](glossary.md#near-miss)**, so the gate cannot certify it generalizes. It hands the candidate to a human ratifier. First-class, not a failure. |
| refused (autoimmune) | The draft would match a clean-corpus item, or is bare-structural and over-general. Refused, so it cannot flag known-good code. |
| no candidate / no cluster | The marks share only their shape (no real defect signal), or fewer than two share a shape at all. Nothing safe to generalize. |

> **JSON note.** `--format json` produces a JSON object once a cluster is assembled
> and routed through the gate. The early *no-cluster* and *empty-clean-corpus*
> messages above are emitted as plain text regardless of `--format` — they precede
> the gate. A sub-two-site cluster prints the same plain line under `--format json`;
> JSON is for the gated outcomes, not the pre-gate usage messages.

The vocabulary this command uses — *marked unknown*, *anti-unify*, *clean corpus*,
*GATE-G*, *near-miss*, *route-to-human*, *PromotedDraft* — is anchored in the
[glossary](glossary.md#learning-core-terms-adr-044045047048).

### `mine`

```
cargo antigen mine [--root <PATH>] [--min-pairs <N>] [--format human|json]
```

Mines a repository's `.git` for the SZZ `(defect, fix)` corpus — the Learning-Core's
input corpus, **recomputable from git history** (it is never stored; you regenerate it).
It walks the **full object graph** (`rev-list --all`, not a tip revwalk), classifies
fix-commits, and links each to its first parent.

```
$ cargo antigen mine
SZZ corpus mined from .
  246 (defect, fix) pairs across the full object graph
```

`--min-pairs <N>` is the honest tripwire: exit non-zero (`1`) if the mined corpus has
**fewer** than `N` pairs. A near-zero count on a repo with real fix-history signals a
tip-revwalk regression (the corpus-starvation bug). Default `0` = report-only, never fail
on size.

```
$ cargo antigen mine --min-pairs 999999
SZZ corpus mined from .
  246 (defect, fix) pairs across the full object graph
  note: corpus is small — on a repo with real fix-history this can signal a tip-revwalk (mine the full graph: rev-list --all)
$ echo $?
1
```

`--format json` emits the full pair list (`{ defect_commit, fix_commit }` records) for a
tool to consume. `mine` is the one shipped verb that feeds the v0.6 maturing-organism: its
corpus is the recomputable input behind the life-record STOCK. The organs that consume it —
maturation, drift-detection, curation — are a **library** today (`antigen::learn::*`, see
[`library-api.md`](library-api.md)); the `cargo antigen` verb that drives the full curation
loop is the v0.7 frontier.

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

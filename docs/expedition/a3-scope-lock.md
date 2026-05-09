# Sweep A3 Scope-Lock

> **Status**: Draft — authored by navigator at A3 open (2026-05-09).
> Aristotle will Phase 1-8 this as the first formal A3 artifact.
>
> **Companion substrate**: scout's A3 seeds from A2 day-2 at
> `campsites/antigen-A2/20260508145642-day2/scout/20260508150446-a3-scope-lock-seeds-from-scout-day-2.md`
> This document formalizes those seeds into ratifiable scope.
>
> **Visible decision**: "A3 ships cross-crate scan + `#[descended_from]` propagation."
> Load-bearing X-1 commitment: every trust boundary introduced by cross-crate
> scanning requires a Sub-clause F validation check before trust is extended
> (ADR-005).

---

## A3 headline scope

Three interlocking deliverables — ordered by dependency:

1. **`#[descended_from]` propagation walk** — collect `LineageEdge` records during
   file scan; emit `lineage_edges` field on `ScanReport`; surface inherited
   presentations via synthesis pass.

2. **Cycle detection (ATK-A3-002)** — DFS white/gray/black coloring on the
   lineage graph; depth limit (64, configurable); emit into `parse_failures` on
   detection. **Safety requirement, not optional** — ADR-005 Amendment 3.

3. **Cross-crate source walking** — `cargo metadata` traversal + `.cargo/registry`
   path resolution; call existing `scan_workspace()` on each resolved path. ~30
   lines of new code. No structural change to the scanner.

### What A3 explicitly does NOT include

- Doc-comment embedding (`<!-- antigen:metadata:v1 {...} -->`) — verified viable
  by scout but requires ADR-001 amendment; deferred to A4+.
- Static emission via `#[cfg(doc)] pub static` — verified via attribute matrix
  by scout; zero binary cost but docs.rs visibility surface; ADR-001 amendment
  required; post-A5 territory.
- SARIF output (`--output-format sarif`) — full mapping designed by scout;
  deferred to post-A5 (stable AuditReport API prerequisite). ADR-017 candidate.
- `ParseFailure` structured enum — open question (see §5 below); decision deferred
  to implementation surface.
- `cargo-checkmate` integration — A5+ adoption vector per ADR-002.

---

## Deliverable 1: `#[descended_from]` propagation

### New type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    pub child: String,   // antigen type bearing #[descended_from]
    pub parent: String,  // argument to #[descended_from]
    pub file: PathBuf,
    pub line: usize,
}
```

### ScanReport field

```rust
pub lineage_edges: Vec<LineageEdge>   // with #[serde(default)]
```

### Collection site

`check_attrs` arm, called from `visit_item_struct` and `visit_item_enum`.
- If `item_target` is not `Struct` or `Enum`: push `parse_failure`
  ("descended_from on non-type item") and return. (Guard for impl-block misuse.)
- `child` = `item_target.label()` (bare type name).
- `parent` = parse attribute body as `syn::Path`, take
  `.segments.last().ident.to_string()`.

### Synthesis pass

After cycle detection passes clean, propagation walk reads `child → Vec<parent>`
and synthesizes inherited presentations into `ScanReport`.

### Stale reference handling (ATK-A3-003)

Stale `#[descended_from]` references (parent no longer in scan) are **not**
`parse_failures` — they are semantic warnings, parallel to `orphaned_tolerances()`.

```rust
impl ScanReport {
    /// Returns lineage edges whose parent antigen is not present in this scan.
    /// Parallel to `orphaned_tolerances()`.
    pub fn orphaned_lineage_edges(&self) -> Vec<&LineageEdge> {
        let known: std::collections::HashSet<&str> =
            self.antigens.iter().map(|a| a.type_name.as_str()).collect();
        self.lineage_edges.iter()
            .filter(|e| !known.contains(e.parent.as_str()))
            .collect()
    }
}
```

**Channel taxonomy** (substrate-grounded by scout, navigator-ratified 2026-05-08,
substrate-verified by navigator 2026-05-09):
- `parse_failures: Vec<ParseFailure>` — structural/IO errors; scan cannot complete correctly.
  `ParseFailure` is a named struct `{ file: PathBuf, error: String }`.
- `orphaned_tolerances()` / `orphaned_lineage_edges()` — semantic warnings; scan
  completed but declarations reference things no longer present.
  `orphaned_tolerances()` already landed at `antigen/src/scan.rs:659` —
  `orphaned_lineage_edges()` is the parallel method A3 adds.

---

## Deliverable 2: Cycle detection (ATK-A3-002)

**Placement**: after file walk, before `synthesis_pass` in `scan_workspace`.

```rust
fn detect_lineage_cycles(edges: &[LineageEdge]) -> Option<(PathBuf, String)>
```

Returns one error per call (first cycle found). Caller pushes into
`report.parse_failures`. Uses `ParseFailure { file, error }` struct form — current
`parse_failures` type (named fields, not a tuple), no breaking change.

**Substrate note**: `ParseFailure` is already a named struct (`pub struct ParseFailure
{ pub file: PathBuf, pub error: String }`), not a `Vec<(PathBuf, String)>` tuple.
Scout's seeds doc referenced the tuple form — the landed substrate is better. The
"candidate structured enum" open question below remains valid but the *current* form
is the struct, not a tuple.

### DFS algorithm (iterative — no stack overflow risk)

```
stack: Vec<(&str, usize)>   // (node, next-child-index)
color: HashMap<&str, u8>    // 0=white, 1=gray, 2=black
MAX_DEPTH: usize = 64       // configurable via [package.metadata.antigen]
```

Entry: mark node gray, push (node, 0). Each iteration: if current node has more
children at idx — white=push, gray=CYCLE (return error), black=skip, advance idx.
When idx >= children.len(), mark node black, pop. `stack.len() >= MAX_DEPTH`
fires depth-limit error with `ParseFailure::LineageDepthExceeded { item, depth }`.

Error message includes: child, parent (closing edge), file, line.

Both guards are **hard entry requirements** (ADR-005 Amendment 3):
1. Cycle detection — catches infinite-loop case.
2. Depth limit — bounds pathological-linear chains.

---

## Deliverable 3: Cross-crate source walking

**Mechanism**: `cargo metadata --format-version 1` enumerates workspace +
dependency crates with registry paths. For each dependency, resolve to
`~/.cargo/registry/src/<index>/<crate-version>/`, then call existing
`scan_workspace()` on that path.

**Why this is right for A3** (ruling: team-lead 2026-05-08, navigator-ratified):
- Scanner is unchanged — ~30 lines of new code in traversal + path resolution.
- Covers ~95% of realistic A3 cases (path-deps, workspace-internal,
  `.cargo/registry`).
- No proc-macro changes, no ADR-001 amendment, no new registry infrastructure.
- cargo already verifies checksums — cross-crate trust extends from cargo's own
  verification. (Sub-clause F: trust boundary validated by cargo's existing
  verification chain; ADR coverage needed for this trust model — see §6 below.)

**Closed-source / opaque-dep cases**: not in v0.1 adoption flywheel; surfaces
when antigen-stdlib propagates post-A5.

---

## ATK contracts in scope for A3

All five contracts from `antigen/tests/atk_a3_fractal_preview.rs` are A3 scope:

| Contract | Failure mode | Channel |
|---|---|---|
| ATK-A3-001 | Re-exported witness — false `NotFound` | Cross-crate witness resolution |
| ATK-A3-002 | Circular `#[descended_from]` — hang | `parse_failures` (safety req.) |
| ATK-A3-003 | Stale `#[descended_from]` ref — silent drop | `orphaned_lineage_edges()` |
| ATK-A3-004 | Proc-macro generated witness — silent miss | Cross-crate witness resolution |
| ATK-A3-005 | Cross-crate name collision — false suppress | `ItemTarget` module-path qualification |

**ATK-A3-002 is the standout**: crash-class failure, both guards non-negotiable,
implement first as design constraint before other deliverables.

**ATK-A3-005 design implication**: naive cross-crate implementation relaxes
`i.file == p.file` guard → same-named types in different crates silently
suppress each other's presentations. The fix requires module-path-qualified
`ItemTarget` — an ADR-class decision, not a one-line fix. File at A3 open so
it's in scope before implementation begins.

---

## Open question: `ParseFailure` structured enum

**Current form**: `Vec<(PathBuf, String)>` — adequate for human-readable output.

**Candidate structured form**:
```rust
pub enum ParseFailure {
    FileParse { file: PathBuf, error: String },
    FingerprintParse { file: PathBuf, antigen: String, error: String },
    CircularLineage { anchor: PathBuf, chain: Vec<String> },
    LineageDepthExceeded { file: PathBuf, item: String, depth: usize },
}
```

**ADR-006 threshold**: needs real instances of "caller needs to distinguish
failure categories" before ratifying. Watch for:
- A3 CLI needing separate exit codes for cycle-errors vs other failures.
- A3 JSON output needing machine-readable failure categorization.
- Test code matching on error message substrings (fragile → signals need for
  structure).

If breaking change warranted: ADR treatment required. `#[serde(default)]` on
both old and new forms required for backward-compat deserialization.

**Decision: defer to implementation surface.** Aristotle's Phase 1-8 on this
scope-lock should include a Phase 8 recommendation on whether to pull the enum
forward into A3 or leave the question open for A4.

---

## ADR coverage needed in A3

Two trust-boundary introductions require ADR coverage before implementation:

1. **Cross-crate trust model**: `cargo metadata` + `.cargo/registry` source
   walking introduces antigen declarations from dependencies as trusted inputs.
   Sub-clause F (ADR-005): cargo's checksum verification is the trust anchor.
   Needs a sentence in ADR-001 Amendment 2 or a new small ADR. **Don't implement
   cross-crate scan before this is documented.**

2. **Module-path-qualified `ItemTarget`** (ATK-A3-005): changing `ItemTarget`
   variants to carry crate-qualified paths (`crate_a::MyType` vs `crate_b::MyType`)
   is an ADR-class decision. Affects `addresses()` semantics, public API surface,
   serde shape. **File ADR before implementation.**

Aristotle to evaluate both in Phase 1-8. Navigator ruling: neither blocks A3
from *starting* but both must be documented before the implementing code lands.

---

## Structural note: `parse_failures` field

`parse_failures: Vec<(PathBuf, String)>` was added during W6a (fingerprint
grammar) — already used at lines 571, 722, 963, 1002, 1031, 1042, 1206 of
`antigen/src/scan.rs`. A3 cycle detection writes directly into this field.
**No structural breaking change needed.** The W6a addition pre-solved the A3
structural requirement.

---

## Substrate verification checklist (navigator-verified 2026-05-09)

- [x] `ScanReport::parse_failures` is `Vec<ParseFailure>` (named struct, not tuple) — already landed W6a
- [x] `orphaned_tolerances()` — already landed at `antigen/src/scan.rs:659`
- [x] `atk_a3_fractal_preview.rs` has all 5 contracts as `#[ignore]` (pre-impl)
- [x] `lineage_edges` does NOT yet exist on `ScanReport` — correct starting state
- [x] `detect_lineage_cycles` does NOT yet exist — correct starting state
- [x] `orphaned_lineage_edges()` does NOT yet exist — correct starting state
- [x] `cargo test --workspace` exits 0 at A3 open — 190 passing, 0 failed (verified 2026-05-09)

---

*Authored 2026-05-09 by navigator. Aristotle to Phase 1-8 as first A3 formal artifact.*

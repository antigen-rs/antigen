# Sweep A3 Scope-Lock

> **Status**: ADR-017 + ADR-018 drafts filed; team review pass open; ratification
> pending. D1+D2 landed (df72f9e). D3 in active implementation (pathmaker, green-lit
> 2026-05-09). D1.5 waits for ADR ratification + BUG-A3-002 fix. BUG-A3-001
> (duplicate edge silent) + BUG-A3-002 (child-without-antigen) in flight (pathmaker).
>
> **Companion substrate**: scout's A3 seeds from A2 day-2 at
> `campsites/antigen-A2/20260508145642-day2/scout/20260508150446-a3-scope-lock-seeds-from-scout-day-2.md`
> Aristotle Phase 1-8 trilogy at `campsites/antigen-A3/.../aristotle/`:
> `20260509163030-a3-scope-lock-phase-1-8.md` (identity model),
> `20260509170000-diamond-dedup-phase-1-8.md` (dedup mechanics),
> `20260509180000-propagation-semantics-phase-1-8.md` (synthesis pass semantics).
> ADR-017 draft at `campsites/antigen-A3/.../aristotle/20260509190000-adr-017-draft-identity-model.md`.
> ADR-018 draft at `campsites/antigen-A3/.../aristotle/20260509193000-adr-018-draft-propagation-semantics.md`.
>
> **Visible decision**: "A3 ships cross-crate scan + `#[descended_from]` propagation
> with canonical-path identity model (Approach 3-revised)."
> X-1 commitment: antigen identity is canonical declaration site, expressed as
> `canonical_path: Option<String>` on all affected types.

---

## A3 headline scope

Four interlocking deliverables — ordered by dependency:

1. **`#[descended_from]` propagation walk** — LANDED (commit df72f9e): LineageEdge
   collection, cycle detection, orphaned_lineage_edges(). Remaining: propagation
   walk (synthesis pass) + canonical_path field addition.

1.5. **Diamond inheritance dedup** — synthesis pass dedup by `(antigen_type,
   item_target)` key; `inherited_from: Option<Vec<String>>` provenance field on
   Presentation. Identity-agnostic; waits for ADR ratification to confirm field
   semantics.

2. **Cycle detection (ATK-A3-002)** — LANDED (commit df72f9e). Both guards green:
   DFS white/gray/black + MAX_LINEAGE_DEPTH=64. ATK-A3-002 contract green.

3. **Cross-crate source walking** — **LANDED** (commit 9b677c6, pathmaker, 2026-05-09):
   `enumerate_dep_crate_roots()` via cargo metadata subprocess (~10 lines not ~30,
   per P1 finding). `CrateOrigin` enum + `DepCrateRoot` struct. `DepScanResult`
   wrapper `{ package_name, version, origin, report }`. `--include-deps` flag on
   `cargo antigen scan`. `JsonReport.dep_reports` optional field (backward compat).
   216 deps enumerated, 0 antigen declarations in any dep (P5 confirmed live).
   D3 shipped WITHOUT canonical_path field (field addition waits for ADR-017
   ratification). ATK-A3-006/007/008 pre-impl contracts filed.

## Identity model (Approach 3-revised — ratified 2026-05-09)

`canonical_path: Option<String>` added to:
- `AntigenDeclaration`
- `Presentation`
- `Immunity`
- `Toleration`
- `LineageEdge`

`None` = intra-crate (v0.1 default everywhere). `Some` = cross-crate, set by
cargo-metadata-driven scanner during D3. Additive with `#[serde(default)]` —
no breaking serde change (rc.1 pre-tag window).

`addresses()` semantics: same antigen-type AND same canonical_path (when both
Some) AND structural item-target equality. ATK-A3-005 becomes green-test for this
refactor, not a documented limitation.

`canonical_path` stays `Option<String>` (never required) — leaves door open for
post-A6 shape-based identity (Approaches 4/5/8) without breaking serialization.

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

## Deliverable 1.5: Diamond inheritance dedup (aristotle Phase 1-8, 2026-05-09)

`#[descended_from]` admits multi-parent inheritance via **attribute stacking** —
`DescendedFromArgs` takes ONE parent per attribute; multiple `#[descended_from]`
attributes on the same item each call `extract_descended_from` via `check_attrs`'s
iteration. (Prior text said "multiple args" — wrong framing; phenomenon unchanged.
Substrate correction from aristotle diamond Phase 1-8, 2026-05-09.)

Diamond case: A descended-from B + C; B + C both descended-from D. That is a DAG,
not a cycle; cycle detection correctly passes it. But propagation produces duplicate
inherited presentations (D's presentations reach A via two paths).

**Implementation spec** (aristotle diamond Phase 1-8, identity-agnostic):
- New `inherited_from: Option<Vec<String>>` field on `Presentation` with `#[serde(default)]`
- Walk runs after cycle detection; edge-level dedup `(child, parent)` first
- Diamond dedup: `(antigen_type, item_target)` keys, merge `inherited_from` by set-union
- Explicit + inherited co-existence: attach inheritance info to explicit, don't parallel-record
- Reachability: don't walk through orphaned ancestors (use `orphaned_lineage_edges()`)
- ~60 lines code + ~30 lines tests
- `inherited_from` IS the provenance marker — achieves Reading 3 semantics without
  a breaking `MatchKind` enum change. Audit emits "re-verify witness" hint when
  `inherited_from` is non-empty.

**Witness identity note** (aristotle Phase 8): witnesses are presentation-keyed,
not (presentation × inheritance-path)-keyed. Two paths to the same presentation
don't require two witnesses. Document in the A3 ADR.

**Propagation semantics** (aristotle Phase 1-8, 2026-05-09): Approach 4 + Approach 8
hybrid ratified. `MatchKind` unchanged; `inherited_from: Option<Vec<String>>` is provenance,
not match-kind. Audit warns by default, errors on `--strict` (per ADR-008 Amendment 1).
5-state matrix amends to 7-state in the A3 ADR. `inherited_from` carries transitive
ancestor set (full chain, not just immediate parent). See `campsites/.../aristotle/
20260509180000-propagation-semantics-phase-1-8.md`.

**Timing**: D1.5 implementation waits for ADR ratification (drafting by aristotle).

---

## Deliverable 1: `#[descended_from]` propagation

### LANDED (commit df72f9e, pathmaker, 2026-05-09)

`LineageEdge` type, `ScanReport::lineage_edges` field, `extract_descended_from`,
`detect_lineage_failures` (iterative DFS, MAX_LINEAGE_DEPTH=64, cycle dedup via
`canonicalise_cycle`), `orphaned_lineage_edges()` query method, 14 unit tests +
2 ATK fixture tests. ATK-A3-002 + ATK-A3-003 contracts green.

The not-yet-built piece is the **propagation walk** that consumes `lineage_edges`
and synthesizes inherited presentations — see Deliverable 1.5 spec above.

### Type reference (landed)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    pub child: String,   // antigen type bearing #[descended_from]
    pub parent: String,  // argument to #[descended_from] (bare last segment)
    pub file: PathBuf,
    pub line: usize,
}
```

`ScanReport::lineage_edges: Vec<LineageEdge>` with `#[serde(default)]`.

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

## Cross-crate lineage edges — RESOLVED 2026-05-09

Approach 3-revised ratification (Tekgy, 2026-05-09) settled this question: cross-crate
lineage edges are **in scope for A3**. `canonical_path: Option<String>` on `LineageEdge`
enables cross-crate parent resolution naturally. `#[descended_from(parent)]` in crate A
naming a parent in crate B is legal; the cargo-metadata traversal (D3) resolves the
parent's canonical path during cross-crate scan.

Intra-crate lineage edges set `canonical_path = None` on both child and parent ends.
Cross-crate edges set `canonical_path = Some(crate_id)` on the parent end. The
`orphaned_lineage_edges()` channel surfaces parents whose canonical path cannot be
resolved (crate not in dependency graph or not scanned).

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

3. **`scan_workspace` signature** (scout empirical, 2026-05-09): D3 implementation
   seam requires a decision — Option A (caller stamps `canonical_path` post-scan via
   `report.stamp_canonical_path(&crate_id)`) vs Option B (`scan_workspace` takes
   `crate_id: Option<&str>` param). Scope-lock said "no structural change to scanner"
   which leans toward Option A. ADR should make explicit choice.

4. **`canonical_path` format** — **SETTLED** (scout empirical, 2026-05-09): Format MUST
   be `"name@version"` (e.g. `"foo@1.0.0"`). Name-only is already ambiguous: antigen's
   own dep graph has 4 crate names at multiple versions (`getrandom`, `hashbrown`,
   `r-efi`, `wit-bindgen`). `"name@version"` is available directly from cargo metadata's
   `name` + `version` fields — no parsing required. Version-boundary orphans (upgrade
   `foo@1.0.0` → `foo@1.1.0` makes lineage edges against the old version orphaned) are
   correct behavior — sub-clause F through time: trust boundaries require re-attestation
   at version boundaries. ADR-017 to document this as designed behavior, not limitation.
   A4+ path: semver-range descent claims (tolerate minor-version upgrades without orphan)
   is a future ADR-009 amendment — leave door open, don't implement in A3.

---

## Structural note: `parse_failures` field

`parse_failures: Vec<ParseFailure>` (named struct `{ file: PathBuf, error: String }`)
was added during W6a (fingerprint grammar) — already used across `antigen/src/scan.rs`.
A3 cycle detection writes directly into this field. **No structural breaking change needed.**
The W6a addition pre-solved the A3 structural requirement.

---

## Substrate verification checklist (navigator-verified 2026-05-09)

- [x] `ScanReport::parse_failures` is `Vec<ParseFailure>` (named struct, not tuple) — already landed W6a
- [x] `orphaned_tolerances()` — already landed at `antigen/src/scan.rs:659`
- [x] `atk_a3_fractal_preview.rs` has all 5 contracts as `#[ignore]` (pre-impl)
- [x] `lineage_edges` — LANDED commit df72f9e
- [x] `detect_lineage_failures` (cycle detection + DFS) — LANDED commit df72f9e
- [x] `orphaned_lineage_edges()` — LANDED commit df72f9e
- [x] `cargo test --workspace` — 220 passing, 25 ignored, 0 failed (post-ATK-A3-010, 2026-05-09)
- [x] D3 cross-crate scan — LANDED commit 9b677c6 (`enumerate_dep_crate_roots`, `--include-deps`, `DepScanResult`)
- [x] BUG-A3-001 duplicate lineage edge — LANDED (detect_duplicate_lineage_edges, parse_failures channel)
- [x] BUG-A3-002 dangling child — LANDED (dangling_lineage_edges() query method)
- [x] ATK-A3-006/007/008 — pre-impl contracts filed
- [x] ATK-A3-010 — filed (drift vs waning audit message category, A4+ scope); commit 94fff1b
- [x] **ADR-017 + ADR-018 — RATIFIED 2026-05-09; moved to decisions.md**
- [x] `inherited_from: Option<Vec<ProvenanceEntry>>` on `Presentation` — RATIFIED Option C (Tekgy, 2026-05-09)
- [ ] `canonical_path: Option<String>` on 5 types + LineageEdge — pending implementation (ADR-017 ratified)
- [ ] D1.5 propagation walk + diamond dedup — **UNBLOCKED** (ADR-018 ratified; pathmaker to implement)
- [ ] ATK-A3-001, ATK-A3-004, ATK-A3-005 — still `#[ignore]`; pending implementation
- [ ] ATK-A3-009 — needs reframe (name@version format eliminates original attack surface; residual risk is alt-registry same-name@version collision)

---

*Authored 2026-05-09 by navigator. Aristotle to Phase 1-8 as first A3 formal artifact.*

# Antigen — Output Formats Reference

> Format reference for `cargo antigen scan` and `cargo antigen audit`
> output. Both commands support human-readable (default) and JSON
> (`--format json`) outputs.

For semantics of the tier values in audit output, see
[`witness-tiers.md`](witness-tiers.md). For diagnostic interpretation,
see [`troubleshooting.md`](troubleshooting.md).

---

## `cargo antigen scan`

Walks the workspace, collects antigen declarations, presentations,
immunities, tolerances, lineage edges; reports unaddressed presentations
and passive fingerprint matches.

### Human-readable output

```
Scanning workspace: .

Scanned N files, found M antigen-related declarations:
  - N1 antigen declarations
  - N2 explicit #[presents] markers
  - N3 fingerprint matches (unmarked sites)
  - N4 tolerated sites (#[antigen_tolerance])
  - N5 immunity claims
  - N6 parse failures (see --format json for details)

N3 fingerprint match(es) — structurally similar to a declared antigen:

  path/to/file.rs:LINE  AntigenType on <item-kind> [fingerprint match]
  ...

  To acknowledge each site, use the antigen type shown above:
    #[presents(<antigen>)] to mark explicitly,
    #[immune(<antigen>, witness = ...)] if defended,
    #[antigen_tolerance(<antigen>, rationale = "...")] to document intent.

N7 unaddressed explicit presentation(s):

  path/to/file.rs:LINE  AntigenType on <item-kind>
  ...

To address each site, use the antigen type shown above:
  #[immune(<antigen>, witness = ...)] on the same item,
  OR #[antigen_tolerance(<antigen>, rationale = "...")]
```

Sections appear conditionally — the `fingerprint matches`, `tolerated
sites`, and `parse failures` summary lines are emitted only when their
counts are greater than zero. The `unaddressed explicit presentation(s)`
section appears only when at least one explicit `#[presents]` lacks a
matching `#[immune]` or `#[antigen_tolerance]`.

### Scan modes — flat, member-aware, and dep-inclusive

`cargo antigen scan` has three scope modes. All three emit the **same JSON
shape**; they differ only in *which* `.rs` files are walked and how
`canonical_path` is populated:

| Mode | Flag | What it scans | `canonical_path` on records |
|---|---|---|---|
| **Flat** (default) | *(none)* | `--root` as one directory tree | `null` (intra-workspace; identity is by source file) |
| **Member-aware** | `--workspace` | each Cargo workspace **member** crate, scanned independently and unioned | `<name>@<version>` per member — declarations carry the identity of the member crate they live in |
| **Dep-inclusive** | `--include-deps` | the workspace + each resolved **dependency** crate (registry/git), each scanned independently | dep records stamped `<name>@<version>`; deps appear under `dep_reports` |

**Member-aware mode (`--workspace`, v0.3)** is the substrate for cross-crate
identity (ADR-001 C7). Because each member's declarations are stamped with that
member's `<name>@<version>`, a `#[descended_from(Parent)]` in one member
resolves to a `Parent` declared in another member: the lineage edge's
`parent_canonical_path` is re-resolved to the member that actually declares the
parent antigen. The flat scan cannot do this — it gives every record the same
(`null`) identity, so member boundaries are invisible. Member-aware mode adds no
new JSON keys; it only populates `canonical_path` (which is `null` in flat mode).

**Dep-inclusive mode (`--include-deps`)** adds a top-level `dep_reports` array
(one entry per scanned dependency, each `{ package_name, version, origin, report
}`). Per the cross-crate scope-lock, each dependency is scanned independently —
no cross-crate `addresses()` matching across the workspace/dep boundary. The key
is omitted entirely when the flag is not passed (byte-identical output for
existing consumers).

`--workspace` and `--include-deps` are independent flags and may be combined.

### JSON output (`--format json`)

The top-level JSON object always has these keys: `report` (the scan report),
`unaddressed` (the convenience-rendered unaddressed-presentations list),
`orphaned_lineage_edges` (`#[descended_from]` edges whose parent antigen is not
declared — empty array when sound), and `dangling_child_lineage_edges`
(`#[descended_from]` edges whose child is not itself an `#[antigen]` —
empty array when sound). When `--include-deps` is passed, an additional
`dep_reports` key is present (see "Scan modes" above; omitted otherwise).

```json
{
  "report": {
    "antigens": [
      {
        "name": "panicking-in-drop",
        "type_name": "PanickingInDrop",
        "file": "src/antigens.rs",
        "line": 12,
        "family": "boundary-violation",
        "summary": "Drop impls must not panic; panic-during-unwind causes process abort.",
        "fingerprint": "item = impl, has_method(\"drop\", \"(& mut self)\"), body_contains_macro(\"panic\")",
        "canonical_path": null
      }
    ],
    "presentations": [
      {
        "antigen_type": "PanickingInDrop",
        "file": "src/safe_type.rs",
        "line": 42,
        "item_kind": "impl",
        "item_target": {
          "Impl": {
            "trait_path": "Drop",
            "target_type": "SomeType"
          }
        },
        "match_kind": "explicit_marker",
        "canonical_path": null,
        "inherited_from": null
      }
    ],
    "immunities": [
      {
        "antigen_type": "PanickingInDrop",
        "witness": "no_panic_test",
        "file": "src/safe_type.rs",
        "line": 42,
        "item_kind": "impl",
        "item_target": { /* same shape */ },
        "canonical_path": null
      }
    ],
    "tolerances": [
      {
        "antigen_type": "PanickingInDrop",
        "rationale": "Test fixture deliberately constructs the case.",
        "until": "2026-12-31",
        "see": [],
        "file": "src/tests/fixtures.rs",
        "line": 100,
        "item_kind": "fn",
        "item_target": { "Fn": "deliberate_panic_test" },
        "canonical_path": null
      }
    ],
    "lineage_edges": [
      {
        "child": "UseAfterFreeClass",
        "parent": "MemoryUnsafetyClass",
        "child_canonical_path": null,
        "parent_canonical_path": null,
        "file": "src/antigens.rs",
        "line": 47
      }
    ],
    "files_scanned": 73,
    "parse_failures": [
      {
        "file": "path/to/broken.rs",
        "error": "syntax error: ..."
      }
    ]
  },
  "unaddressed": [
    {
      "antigen_known": true,
      "presentation": { /* full presentation record */ }
    }
  ]
}
```

**No top-level `fingerprint_matches` array.** Fingerprint matches appear
as entries in `report.presentations[]` with `match_kind: "fingerprint_match"`
(vs `"explicit_marker"` for `#[presents]` annotations). Discriminate via
the `match_kind` field.

### Field reference

#### `antigens[]`

| Field | Type | Meaning |
|---|---|---|
| `name` | string | kebab-case identifier (from `#[antigen(name = "...")]`) |
| `type_name` | string | Rust type identifier (the struct name) |
| `file` | string | Source file path |
| `line` | integer | Line number where `#[antigen]` is declared |
| `family` | string \| null | Parent class (from `family = "..."`) |
| `summary` | string \| null | Human-readable description |
| `fingerprint` | string | Fingerprint DSL (verbatim; canonicalized for matching) |
| `canonical_path` | string \| null | Cross-crate identity at `crate@version::Type` granularity (ADR-017); null for intra-workspace declarations |

Note: the `references = [...]` field accepted by `#[antigen(...)]` is
parsed at scan time but is not currently emitted to the JSON output
surface. Future versions may surface it.

#### `presentations[]`

| Field | Type | Meaning |
|---|---|---|
| `antigen_type` | string | Type name of the antigen this presents |
| `file` | string | Source file path |
| `line` | integer | Line number of the `#[presents]` attribute (or fingerprint match site) |
| `item_kind` | string | One of: `fn`, `impl`, `struct`, `enum`, `trait`, `mod`, `type`, `const`, `static`, `use` |
| `item_target` | object | Structured target identity (see "Item target shapes" below) |
| `match_kind` | string | `explicit_marker` (from `#[presents]`) or `fingerprint_match` (passive detection) |
| `canonical_path` | string \| null | Cross-crate identity |
| `inherited_from` | array of objects \| null | `ProvenanceEntry` array for inherited presentations (see "Provenance" below); null if not inherited |

#### `immunities[]`

| Field | Type | Meaning |
|---|---|---|
| `antigen_type` | string | Type name of antigen claimed immune from |
| `witness` | string | Witness identifier (function name, phantom-type expression, external-tool reference) |
| `file` | string | Source file path |
| `line` | integer | Line number of `#[immune]` attribute |
| `item_kind` | string | (same enum as presentations) |
| `item_target` | object | (same shape as presentations) |
| `canonical_path` | string \| null | Cross-crate identity |

#### `tolerances[]`

| Field | Type | Meaning |
|---|---|---|
| `antigen_type` | string | Type name of antigen this tolerates |
| `rationale` | string | Required narrative justification |
| `until` | string \| null | Expiry condition (date, version, etc.); informational |
| `see` | array of strings | Open-vocabulary cross-references (PR URLs, ADR IDs, design docs) |
| `file`, `line` | string, integer | Site location |
| `item_kind`, `item_target` | (same shapes) | |
| `canonical_path` | string \| null | Cross-crate identity |

#### `lineage_edges[]`

| Field | Type | Meaning |
|---|---|---|
| `child` | string | Type name of the child antigen (from `#[descended_from]` site) |
| `parent` | string | Type name of the parent antigen (last segment of `#[descended_from(...)]` argument) |
| `child_canonical_path` | string \| null | Cross-crate identity for the child (ADR-017) |
| `parent_canonical_path` | string \| null | Cross-crate identity for the parent |
| `file`, `line` | string, integer | Where `#[descended_from]` was declared |

Lineage edges enable inheritance propagation (ADR-018). Cycle detection
runs on this graph; orphan edges (parent doesn't exist) surface via
`orphaned_lineage_edges()` query method; dangling edges (child not in
antigen index) surface via `dangling_child_lineage_edges()`.

#### `files_scanned`

| Field | Type | Meaning |
|---|---|---|
| `files_scanned` | integer | Count of `.rs` files successfully parsed during the scan (excludes files in `target/`, `.git/`, `node_modules/`, and any other excluded directories) |

#### `parse_failures[]`

| Field | Type | Meaning |
|---|---|---|
| `file` | string | Source file path where parsing failed |
| `error` | string | Human-readable parse error message |

Surfaces structural errors that prevent correct scan completion (file
IO, syntax errors, malformed fingerprints, malformed attribute arguments,
`#[descended_from]` cycles, lineage chain depth limits exceeded).

#### `scan_coverage` (member-aware scans only)

Present only under `--workspace` (member-aware mode); omitted entirely for a
flat scan (which has no member concept).

| Field | Type | Meaning |
|---|---|---|
| `enumerated_members` | array of strings | Every workspace member `cargo metadata` reported, as `<name>@<version>` canonical paths (sorted) |
| `scanned_members` | array of strings | The members actually scanned (sorted) — a member is here iff its per-member scan ran |

The complement (`enumerated_members` − `scanned_members`) is the **ignorance
frontier**: members whose `#[presents]` sites the scan never reached. In a full
`--workspace` scan the two sets are equal (empty frontier). A member that could
not be scanned is recorded in `parse_failures` AND left out of `scanned_members`
— so an unscannable member surfaces as an unseen (ignored) region rather than a
silent gap. This is the substrate a downstream ignorance audit reads.

#### `unaddressed[]` (top-level convenience array)

The top-level `unaddressed` key is a pre-rendered convenience array
listing all `#[presents]` sites that lack a matching `#[immune]` or
`#[antigen_tolerance]`. Consumers can either filter `report.presentations`
themselves or read this directly.

| Field | Type | Meaning |
|---|---|---|
| `antigen_known` | bool | Whether the antigen type is declared in the same scan |
| `presentation` | object | Full `Presentation` record (same shape as `report.presentations[]`) |

### Item target shapes

`item_target` is a tagged enum reflecting the kind of Rust item:

```json
{ "Fn": "function_name" }
{ "Struct": "StructName" }
{ "Enum": "EnumName" }
{ "Trait": "TraitName" }
{ "TypeAlias": "AliasName" }
{ "Const": "CONSTANT_NAME" }
{ "Static": "STATIC_NAME" }
{ "EnumVariant": { "enum_name": "EnumName", "variant_name": "VariantName" } }
{ "Impl": { "trait_path": "TraitName" | null, "target_type": "TypeName" } }
{ "ImplFn": { "trait_path": "TraitName" | null, "target_type": "TypeName", "fn_name": "method" } }
{ "ImplConst": { "trait_path": "TraitName" | null, "target_type": "TypeName", "const_name": "ASSOC_CONST" } }
{ "TraitFn": { "trait_name": "TraitName", "fn_name": "method" } }
{ "Unknown": { "line": 42 } }
```

For `impl` blocks, both inherent impls and trait impls are captured;
`trait_path` is null for inherent impls. `EnumVariant`, `ImplConst`, `Const`,
and `Static` carry attributes on positions the scanner descends into so a
`#[presents]` / `#[immune]` on a variant, an associated const, or a free
const/static is not silently dropped (ScannerBoundaryFalseNegative). A
trait-associated const reuses the `ImplConst` shape with the trait as
`target_type`. `Unknown` is the fallback for item shapes not yet modelled and
carries the source line so distinct unhandled items don't collide.

### Provenance (inherited_from)

When a presentation is inherited via `#[descended_from]` propagation,
the `inherited_from` field carries a list of `ProvenanceEntry` objects
recording the inheritance chain (ADR-018 Option C):

```json
"inherited_from": [
  {
    "antigen_type": "MemoryUnsafetyClass",
    "canonical_path": null
  }
]
```

Multiple entries indicate multi-path inheritance (diamond) deduped
with provenance preserved. Each `ProvenanceEntry` carries the parent
antigen's identity (type name + canonical path for cross-crate).

---

## `cargo antigen audit`

Walks the workspace + dependencies, runs scan, then validates every
`#[immune]` claim by resolving its witness identifier and reporting
tier-honest verification status.

### Human-readable output

The audit prints a summary, an optional confirmed-claims block (claims
at Execution tier or higher), an optional warnings block (claims below
Execution tier), and an optional state-7 block (inherited presentations
not re-attested).

```
Auditing workspace: .

Audited N immunity claim(s):
  - N_fp formal-proof (phantom-type or formal-verification tool — compile-time evidence)
  - N_ex execution (test/proptest run confirmed by audit)
  - N_re declared (witness identifier found in workspace — not yet semantically verified)
  - N_external external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - N_amb ambiguous (witness name resolves to multiple workspace functions)
  - N_broken broken (witness identifier not found)
  - N_missing missing (no witness identifier)

✓ N_confirmed immunity claim(s) at Execution tier or higher:

  path/to/file.rs:LINE  AntigenType (witness = `witness_expression`)
    tier = FormalProof, hint = PhantomTypeShapeRecognized

⚠ N_warn immunity claim(s) below Execution tier:

  path/to/file.rs:LINE  AntigenType (witness = `witness_name`)
    tier = <Tier>, hint = <AuditHint>
    → <diagnostic_text>

Resolve below-Execution claims by either:
  a) Adding test invocation that exercises the witness path (A4-A5 feature)
  b) Pointing the witness at a runnable test (#[test] without #[ignore])
  c) Renaming colliding functions or qualifying ambiguous witness paths
  d) Adding the witness function to the workspace if it's missing
  e) Tolerating the gap with `#[antigen_tolerance(...)]` if intentional

⚠ N_state7 inherited presentation(s) not re-attested on the descendant (state 7 of the 7-state interaction matrix):

  warning: inherited presentation: `AntigenType` flowed from ["AncestorType"] to `<item-kind>` via `#[descended_from]`;
  the witness inherited from the ancestor has not been re-attested on the descendant.
  Add `#[immune(AntigenType, witness = ...)]` or `#[antigen_tolerance(AntigenType, rationale = "...")]` on the descendant.
    --> path/to/descendant.rs:LINE

  Note: behavioral re-validation (does the ancestor's witness apply to the descendant?) is A4-A5 work; reachability-tier audit cannot perform this check.
  Use `cargo antigen audit --strict` to promote state-7 warnings to errors for CI gating.
```

The summary lines for `formal-proof` and `execution` are conditionally
emitted only when the count is greater than zero. The confirmed-claims
block (✓) and the warnings block (⚠) likewise appear only when their
respective sets are non-empty. The state-7 block appears only when
inherited Presentations exist that lack re-attestation on the descendant
site.

### JSON output (`--format json`)

The top-level audit JSON object has two keys: `scan` (the embedded scan
report) and `audit` (the audit report).

```json
{
  "scan": { /* full scan report — see above */ },
  "audit": {
    "audits": [
      {
        "immunity": { /* full immunity record */ },
        "witness_status": {
          "status": "resolved",
          "location": "path/to/witness.rs",
          "witness_kind": "function"
        },
        "witness_tier": "reachability",
        "audit_hint": "function-resolves"
      }
    ],
    "resolved_count": 1,
    "external_count": 0,
    "ambiguous_count": 0,
    "broken_count": 0,
    "missing_count": 0,
    "inherited_unaddressed": [
      {
        "presentation": { /* full presentation record */ },
        "audit_hint": "inherited-presentation-not-re-attested"
      }
    ],
    "presentation_verdicts": [
      {
        "presentation": { /* full presentation record — see scan output's presentations[] */ },
        "antigen_type": "MyAntigen",
        "verdict": { "Defended": { "tier": "reachability" } },
        "defended_by": ["src/tests.rs:42"]
      }
    ]
  }
}
```

### Audit field reference

#### `audit.audits[]`

| Field | Type | Meaning |
|---|---|---|
| `immunity` | object | Full immunity record (same shape as scan output's `immunities[]`) |
| `witness_status` | object | Resolution status with diagnostic — see "witness_status variants" below |
| `witness_tier` | string | One of: `formal_proof`, `execution`, `reachability`, `none` (snake_case-serialized `WitnessTier` enum; v0.1 has four tiers — see [`witness-tiers.md`](witness-tiers.md)) |
| `audit_hint` | string | Specific diagnostic hint, kebab-case-serialized — see "audit_hint values" below |

#### `audit.presentation_verdicts[]` (ADR-029)

Per-presents-site immune-state verdicts. Each `#[presents(X)]` site is cross-referenced
against `#[defended_by(X)]` code-tier witnesses and site-attached `requires=` substrate
evidence and graded: `defended`, `undefended`, or `substrate-gap`.

| Field | Type | Meaning |
|---|---|---|
| `presentation` | object | Full presentation record (same shape as scan output's `presentations[]`) |
| `antigen_type` | string | Antigen type name |
| `verdict` | tagged object | `{"Defended": {"tier": "<tier>"}}`, `"Undefended"`, or `"SubstrateGap"` |
| `defended_by` | array of strings | `"<file>:<line>"` strings for `#[defended_by]` witnesses that cover this site |

**`verdict` shapes:**

| Value | Shape | Meaning |
|---|---|---|
| `Defended` | `{"Defended": {"tier": "<tier>"}}` | At least one passing witness; `tier` is the strongest channel |
| `Undefended` | `"Undefended"` | No passing witnesses and no `requires=` predicate declared |
| `SubstrateGap` | `"SubstrateGap"` | A `requires=` predicate was declared but its evaluation failed (sidecar missing, stale, or predicate not met) — or a failing `requires=` predicate co-exists with a passing code witness (ADR-029 Amendment 1: substrate intent takes precedence) |

#### `audit.audits[].witness_status` variants

`witness_status` is a tagged enum on the `status` field. Each variant
carries different sub-fields:

| `status` value | Shape | Resulting `witness_tier` |
|---|---|---|
| `resolved` | `{status: "resolved", location: "<path>", witness_kind: <WitnessKind>}` | `formal_proof` (phantom-type), or `reachability` (test / ignored_test / proptest / function — v0.1 audit does not invoke harnesses) |
| `external` | `{status: "external", tool_hint: "<string>"}` | `reachability` (with `external-tool-prefix-recognized` audit hint) |
| `ambiguous` | `{status: "ambiguous", candidates: ["<path>", ...]}` | `none` |
| `not_found` | `{status: "not_found", reason: "<diagnostic-text>"}` | `none` |
| `missing` | `{status: "missing"}` (no other fields) | `none` |

#### `WitnessKind` values (sub-field of `Resolved` witness_status)

The `witness_kind` field inside a `Resolved` witness_status reports the
recognized witness shape. Serialized snake_case per `#[serde(rename_all
= "snake_case")]`:

| Serialized value | Source enum variant | Meaning |
|---|---|---|
| `"test"` | `WitnessKind::Test` | `#[test]` function (not `#[ignore]`) |
| `"ignored_test"` | `WitnessKind::IgnoredTest` | `#[test]` + `#[ignore]` — `cargo test` skips by default; audit reports Reachability tier per ATK-A2-012 |
| `"proptest"` | `WitnessKind::Proptest` | A `proptest!` macro invocation |
| `"function"` | `WitnessKind::Function` | Regular function (no testing attribute detected) |
| `{"phantom_type": {"proof_type": "<base>", "type_params": [<str>...], "constructor": "<name>" \| null}}` | `WitnessKind::PhantomType {...}` | Turbofish witness recognized as phantom-type proof per ADR-013 |

#### `audit_hint` values

`AuditHint` is serialized kebab-case per `#[serde(rename_all = "kebab-case")]`
(NOTE: distinct from `WitnessTier` / `WitnessStatus` / `WitnessKind`,
which use snake_case). The complete v0.1.0-rc.1 hint set:

| JSON value | Rust variant | Meaning |
|---|---|---|
| `"none-applicable"` | `NoneApplicable` | Status is Missing or NotFound; no further hint applicable |
| `"function-resolves"` | `FunctionResolves` | Identifier resolves to a function; no further check performed |
| `"test-attribute-present-not-invoked"` | `TestAttributePresentNotInvoked` | Function has `#[test]`; audit did not invoke `cargo test` |
| `"test-attribute-present-ignore-skipped"` | `TestAttributePresentIgnoreSkipped` | Function has `#[test]` AND `#[ignore]`; `cargo test` would skip it |
| `"proptest-present-not-invoked"` | `ProptestPresentNotInvoked` | `proptest!` macro invocation found; harness not invoked |
| `"external-tool-prefix-recognized"` | `ExternalToolPrefixRecognized` | External-tool prefix recognized (`clippy::`, `kani::`, etc.); tool not invoked |
| `"external-tool-invoked"` | `ExternalToolInvoked` | External tool actually invoked (A4+; not emitted in v0.1) |
| `"phantom-type-shape-recognized"` | `PhantomTypeShapeRecognized` | Phantom-type witness shape recognized; constructor sealing not validated |
| `"phantom-type-construction-validated"` | `PhantomTypeConstructionValidated` | Phantom-type construction validated (future; not emitted in v0.1) |
| `"ambiguous-resolution"` | `AmbiguousResolution` | Witness name matches more than one workspace function (ATK-A2-005) |
| `"fabricated-path-prefix"` | `FabricatedPathPrefix` | Witness path's module prefix does not exist in the workspace; last segment found but in an unrelated location (ATK-A2-011) |
| `"inherited-presentation-not-re-attested"` | `InheritedPresentationNotReAttested` | Inherited Presentation lacks re-attestation on the descendant site (state 7 of the 7-state matrix; ADR-018) |

#### `audit.inherited_unaddressed[]`

Presentations inherited via `#[descended_from]` propagation that don't
have a corresponding `#[immune]` claim on the inheriting site. The
audit hint `inherited-presentation-not-re-attested` indicates per
ADR-005 sub-clause F that inheritance does not transitively claim
immunity — descendants must re-attest.

---

## Exit codes

Both `scan` and `audit` exit 0 by default, even when unaddressed
presentations or below-Execution witnesses are present. This is
**warn-not-error semantics** per ADR-008 Amendment 1: a developer
running antigen on a fresh codebase shouldn't have CI fail until
they've made conscious decisions about each site.

| Exit code | Meaning |
|---|---|
| 0 | Scan/audit completed successfully (regardless of findings) |
| 1 | Internal error (file IO, parsing, etc.) |
| 2 | Strict mode triggered: unaddressed presentations or witnesses below threshold |

Strict mode is opt-in via `--strict` (planned; per ADR-008 Amendment 1
strict-mode is the future enforcement surface; v0.1.0-rc.1 ships
warn-only).

---

## Schema versioning

JSON output is currently considered **v0.1** schema. Field additions
are forward-compatible (new optional fields may appear in subsequent
versions); field removals or semantic changes will bump the schema
version.

Future versions may add a `schema_version` field at the top level for
explicit versioning. Until then, consumers should:
- Ignore unknown fields gracefully
- Treat `null` as semantically equivalent to "field absent"
- Match on `witness_kind` and `witness_status.status` exhaustively if
  using them for routing logic

---

## See also

- [`macros.md`](macros.md) — the macro reference that drives what gets scanned
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`witness-tiers.md`](witness-tiers.md) — tier semantics
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide
- [`tutorial.md`](tutorial.md) — first 15 minutes; shows real scan/audit output inline
- ADR-005 Amendment 3 (audit-tier-honesty)
- ADR-008 Amendment 1 (warn-not-error default)
- ADR-018 (ProvenanceEntry + diamond dedup)

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

N3 fingerprint match(es) — structurally similar to a declared antigen:

  path/to/file.rs:LINE  AntigenType on <item-kind> [fingerprint match]
  ...

  To acknowledge each site, use the antigen type shown above:
    #[presents(<antigen>)] to mark explicitly,
    #[immune(<antigen>, witness = ...)] if defended,
    #[antigen_tolerance(<antigen>, rationale = "...")] to document intent.

N6 unaddressed explicit presentation(s):

  path/to/file.rs:LINE  AntigenType on <item-kind>
  ...

To address each site, use the antigen type shown above:
  #[immune(<antigen>, witness = ...)] on the same item,
  OR #[antigen_tolerance(<antigen>, rationale = "...")]
```

Sections appear conditionally — empty sections are omitted.

### JSON output (`--format json`)

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
    "fingerprint_matches": [ /* same shape as presentations; match_kind: "fingerprint" */ ],
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
        "file": "src/antigens.rs",
        "line": 47
      }
    ],
    "parse_failures": [
      [ "path/to/broken.rs", "syntax error: ..." ]
    ]
  }
}
```

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

#### `presentations[]`

| Field | Type | Meaning |
|---|---|---|
| `antigen_type` | string | Type name of the antigen this presents |
| `file` | string | Source file path |
| `line` | integer | Line number of the `#[presents]` attribute (or fingerprint match site) |
| `item_kind` | string | One of: `fn`, `impl`, `struct`, `enum`, `trait`, `mod`, `type`, `const`, `static`, `use` |
| `item_target` | object | Structured target identity (see "Item target shapes" below) |
| `match_kind` | string | `explicit_marker` (from `#[presents]`) or `fingerprint` (passive detection) |
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
| `file`, `line` | string, integer | Site location |
| `item_kind`, `item_target` | (same shapes) | |
| `canonical_path` | string \| null | Cross-crate identity |

#### `lineage_edges[]`

| Field | Type | Meaning |
|---|---|---|
| `child` | string | Type name of the child antigen |
| `parent` | string | Type name of the parent antigen |
| `file`, `line` | string, integer | Where `#[descended_from]` was declared |

Lineage edges enable inheritance propagation (ADR-018). Cycle detection
runs on this graph; orphan edges (parent doesn't exist) surface via
`orphaned_lineage_edges()` query method.

#### `parse_failures[]`

Array of `[file_path, error_message]` tuples. Structural errors that
prevent correct scan completion (file IO, syntax errors, malformed
fingerprints).

### Item target shapes

`item_target` is a tagged enum reflecting the kind of Rust item:

```json
{ "Fn": "function_name" }
{ "Struct": "StructName" }
{ "Enum": "EnumName" }
{ "Trait": "TraitName" }
{ "Mod": "module_name" }
{ "Type": "TypeAlias" }
{ "Const": "CONSTANT_NAME" }
{ "Static": "STATIC_NAME" }
{ "Use": "imported_path" }
{ "Impl": { "trait_path": "TraitName" | null, "target_type": "TypeName" } }
```

For `impl` blocks, both inherent impls and trait impls are captured;
`trait_path` is null for inherent impls.

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

```
Auditing workspace: .

Audited N immunity claim(s):
  - N1 declared (witness identifier found in workspace — not yet semantically verified)
  - N2 external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - N3 ambiguous (witness name resolves to multiple workspace functions)
  - N4 broken (witness identifier not found)
  - N5 missing (no witness identifier)
  - N6 confirmed (FormalProof tier — phantom-type witness shape recognized)

⚠ N₁₋₅ immunity claim(s) below Execution tier:

  path/to/file.rs:LINE  AntigenType (witness = `witness_name`)
    tier = <Tier>, hint = <AuditHint>
    → <diagnostic_text>

Resolve below-Execution claims by either:
  a) Adding test invocation that exercises the witness path
  b) Pointing the witness at a runnable test (#[test] without #[ignore])
  c) Renaming colliding functions or qualifying ambiguous witness paths
  d) Adding the witness function to the workspace if it's missing
  e) Tolerating the gap with `#[antigen_tolerance(...)]` if intentional

✓ N₆ immunity claim(s) at FormalProof tier:

  path/to/file.rs:LINE  AntigenType (witness = `witness_expression`)
    tier = FormalProof, hint = PhantomTypeShapeRecognized
```

### JSON output (`--format json`)

```json
{
  "scan": { /* full scan report — see above */ },
  "audit": {
    "results": [
      {
        "immunity": { /* full immunity record */ },
        "witness_status": {
          "status": "resolved" | "external" | "ambiguous" | "not_found" | "missing",
          "location": "path/to/witness.rs" | "",
          "reason": "diagnostic text if not resolved",
          "witness_kind": {
            "phantom_type": {
              "proof_type": "NonPanickingProof",
              "type_params": ["PhantomVerifiedDropImpl"],
              "constructor": "verified"
            }
          }
        },
        "witness_tier": "formal_proof" | "execution" | "reachability" | "external_unvalidated" | "none",
        "audit_hint": "phantom-type-shape-recognized" | "function-resolves" | "external-tool-delegated" | "witness-not-found" | "witness-ambiguous" | "witness-missing" | "none-applicable"
      }
    ],
    "resolved_count": integer,
    "external_count": integer,
    "ambiguous_count": integer,
    "broken_count": integer,
    "missing_count": integer,
    "inherited_unaddressed": [
      {
        "presentation": { /* full presentation record */ },
        "audit_hint": "inherited-presentation-not-re-attested"
      }
    ]
  }
}
```

### Audit field reference

#### `audit.results[]`

| Field | Type | Meaning |
|---|---|---|
| `immunity` | object | Full immunity record (same shape as scan output) |
| `witness_status` | object | Resolution status with diagnostic |
| `witness_tier` | string | One of: `formal_proof`, `execution`, `reachability`, `external_unvalidated`, `none` |
| `audit_hint` | string | Specific diagnostic hint (see [`witness-tiers.md`](witness-tiers.md) for full enumeration) |

#### `witness_status` shapes

| `status` value | Shape | Tier |
|---|---|---|
| `resolved` | `{status, location, witness_kind?}` | FormalProof / Execution / Reachability |
| `external` | `{status, location: "", witness_kind: {external_tool: "name"}}` | ExternalUnvalidated |
| `ambiguous` | `{status, candidates: ["path1", "path2"]}` | None (Missing tier) |
| `not_found` | `{status, reason: "diagnostic text"}` | None (Missing tier) |
| `missing` | `{status}` (no witness field on `#[immune]`) | None (Missing tier) |

#### `witness_kind` variants

Within `witness_status`, the `witness_kind` field describes what kind
of witness was recognized:

```json
{ "test_function": "fn_name" }
{ "proptest_function": "fn_name" }
{ "external_tool": "clippy::lint_name" }
{ "phantom_type": {
    "proof_type": "ProofType",
    "type_params": ["T", "U"],
    "constructor": "constructor_method"
} }
```

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

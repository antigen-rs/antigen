# Antigen — Macro Reference

> Comprehensive reference for the five attribute macros that constitute
> antigen's API surface. For tutorials, see [`tutorial.md`](tutorial.md);
> for placement conventions, see
> [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md); for
> patterns, see [`usage-patterns.md`](usage-patterns.md).

All macros are imported from the `antigen` crate:

```rust
use antigen::{antigen, presents, immune, descended_from, antigen_tolerance};
```

---

## Quick reference

| Macro | Applies to | Purpose |
|---|---|---|
| `#[antigen]` | unit struct | Declare a named failure-class with a structural fingerprint |
| `#[presents]` | any item | Mark code as exhibiting a known failure-class's structural pattern |
| `#[immune]` | any item | Claim immunity backed by a named witness |
| `#[descended_from]` | unit struct (antigen) | Declare inheritance from a parent failure-class |
| `#[antigen_tolerance]` | any item | Explicitly tolerate a fingerprint match (with required rationale) |

The macros are **identity transforms** — they validate attribute syntax
at compile time and emit the original item unchanged. The semantic work
(scanning, matching, witness validation) lives in `cargo antigen scan`
and `cargo antigen audit`, which parse source independently.

---

## `#[antigen(...)]`

Declare a named failure-class with a structural fingerprint.

### Required attributes

| Field | Type | Purpose |
|---|---|---|
| `name` | string (kebab-case) | Identifier for the failure-class |
| `fingerprint` | string (DSL) | Structural pattern; see [`fingerprint-grammar.md`](fingerprint-grammar.md) |

### Optional attributes

| Field | Type | Purpose |
|---|---|---|
| `family` | string | Parent class (typically one of the 8 first-principles failure classes from `docs/decisions.md` ADR-010) |
| `summary` | string | Human-readable description; surfaces in audit output |
| `references` | array of strings | Open-vocabulary cross-references (URLs, CVE IDs, ADR IDs, RFC IDs, issue IDs, post-mortem links) |

### Applies to

Unit structs only. The struct carries no data — it's a vocabulary token.

### Example

```rust
use antigen::antigen;

/// Drop impls must not panic; panic-during-unwind causes process abort.
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    fingerprint = r#"
        item = impl,
        has_method("drop", "(& mut self)"),
        any_of([
            body_contains_macro("panic"),
            body_contains_macro("unreachable"),
        ])
    "#,
    summary = "Drop impls must not panic; panic-during-unwind causes process abort.",
    references = [
        "https://doc.rust-lang.org/std/ops/trait.Drop.html#panics",
        "CVE-EXAMPLE-2024-001",
    ],
)]
pub struct PanickingInDrop;
```

### Layer 1 minimum viable

```rust
#[antigen(
    name = "my-failure-class",
    fingerprint = r#"item = fn, name = matches("dangerous_*")"#,
)]
pub struct MyFailureClass;
```

Only `name` and `fingerprint` are required. Per ADR-009 adoption gradient,
projects can start at Layer 1 and add `family`, `summary`, `references`
as discipline matures.

### Behavior

`cargo antigen scan` collects every `#[antigen]` declaration in the
workspace and treats each as a recognized failure-class. The
`fingerprint` field drives passive detection — any item in the codebase
matching the structural pattern is reported as a `[fingerprint match]`,
even without an explicit `#[presents]` marker.

### Discipline

- **kebab-case names**: enforced at parse time; `name = "PanickingInDrop"` is rejected
- **Non-empty fingerprint**: enforced at parse time
- **Reference shapes**: open-vocabulary; the tool accepts any string. Conventions documented in `docs/usage-patterns.md`.

### See also

- ADR-001 (carrier set + identity transform)
- ADR-009 (Layer 1 → Layer 2 → Layer 3 adoption gradient)
- ADR-010 (fingerprint grammar v1)
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL reference

---

## `#[presents(antigen_type)]`

Mark code as exhibiting an antigen's structural pattern (vulnerability
declaration).

### Required arguments

- One positional argument: the antigen type (unit struct declared with `#[antigen]`)

### Applies to

Any Rust item: `fn`, `impl`, `struct`, `enum`, `trait`, `mod`, `type`, `const`, `static`, `use`, etc.

### Example

```rust
use antigen::presents;
use crate::antigens::PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for VulnerableType {
    fn drop(&mut self) {
        let _val = self.data.as_ref().unwrap();  // could panic
    }
}
```

### Behavior

`cargo antigen scan` collects every `#[presents]` site and reports it
as an *unaddressed presentation* unless a co-located `#[immune]` claim
addresses it. The audit output groups presentations by antigen type.

### Discipline

- **The antigen type must resolve**: the scan checks that `PanickingInDrop`
  is a known antigen type. Unresolved references surface as errors.
- **Co-location encouraged**: put `#[presents]` at the actual vulnerable
  site (the impl, the function, the struct). For composition-boundary
  failure-classes, the consistency test is the site — see
  [`usage-patterns.md`](usage-patterns.md#antigens-at-composition-boundaries).

### See also

- ADR-001 (carrier set)
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) — placement conventions
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes

---

## `#[immune(antigen_type, witness = ..., rationale = "...")]`

Declare immunity to a known failure-class, backed by a witness.

### Required arguments

- First positional: the antigen type
- `witness = <expr>` (required) — witness identifier (see "Witness types" below)

### Optional arguments

- `rationale = "..."` — narrative justification supplementing the executable witness

### Applies to

Any Rust item; typically co-located with `#[presents]` at the defended site.

### Witness types

| Witness form | Tier (per ADR-005 Amendment 3) | Example |
|---|---|---|
| `test::fn_name` | ExecutionVerified | `witness = test::no_panic_test` |
| `proptest::fn_name` | ExecutionVerified | `witness = proptest::roundtrip_proptest` |
| `kani::fn_name` | FormalProof | `witness = kani::no_panic_proof` |
| `prusti::fn_name` | FormalProof | `witness = prusti::invariant_proof` |
| `verus::fn_name` | FormalProof | `witness = verus::correctness_proof` |
| `creusot::fn_name` | FormalProof | `witness = creusot::specification_proof` |
| `clippy::lint_name` | ExternalUnvalidated | `witness = clippy::no_panic_in_drop` |
| Phantom-type witness | FormalProof | `witness = phantom::LatticeFrameInvariant` |
| Bare identifier | Reachability (workspace-resolved) | `witness = no_panic_test` |

See [`witness-tiers.md`](witness-tiers.md) for tier semantics and
[`fingerprint-grammar.md`](fingerprint-grammar.md) for phantom-type
witness recognition (ADR-013).

### Example

```rust
use antigen::{immune, presents};
use crate::antigens::PanickingInDrop;

#[presents(PanickingInDrop)]
#[immune(
    PanickingInDrop,
    witness = safe_type_drop_no_panic_test,
    rationale = "SafeType::drop uses non-panicking accessors only; verified by test."
)]
impl Drop for SafeType {
    fn drop(&mut self) {
        if let Some(_d) = self.data.as_ref() { /* safe */ }
    }
}

#[allow(dead_code)]
fn safe_type_drop_no_panic_test() {
    drop(SafeType { data: None });
    drop(SafeType { data: Some(String::from("hello")) });
}
```

### Behavior

`cargo antigen audit` verifies every `#[immune]` claim resolves to a
real witness at the appropriate tier. Audit output reports the actual
tier achieved, not the maximal one (ADR-005 Amendment 3 audit-tier-honesty).

### Discipline

- **Witness must resolve**: audit surfaces broken/missing/ambiguous witnesses
- **Tier honesty**: external-tool delegations (kani, prusti, etc.) report
  ExternalUnvalidated unless the consuming workspace can execute them
- **Rationale recommended for production**: especially for tolerance-class
  decisions; the rationale field is the narrative justification

### See also

- ADR-002 (compose, don't compete — witnesses delegate to existing tools)
- ADR-005 (sub-clause F: witness validation at the trust boundary)
- ADR-005 Amendment 3 (audit-tier-honesty)
- ADR-013 (phantom-type witness recognition)
- [`witness-tiers.md`](witness-tiers.md) — tier semantics in detail

---

## `#[descended_from(ParentAntigen)]`

Declare structural inheritance between failure-classes.

### Required arguments

- One positional argument: the parent antigen type

### Applies to

Unit structs declared with `#[antigen]` (extends an existing antigen
declaration).

### Example

```rust
use antigen::{antigen, descended_from};

#[antigen(
    name = "polarity-inverted-class-meet",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("meet", "(self, Self) -> Self")"#,
    summary = "Class enums must use max (not min) for lattice meet.",
)]
pub struct PolarityInvertedClassMeet;

#[antigen(
    name = "polarity-inverted-class-join",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("join", "(self, Self) -> Self")"#,
    summary = "Class enums must use min (not max) for lattice join.",
)]
#[descended_from(PolarityInvertedClassMeet)]
pub struct PolarityInvertedClassJoin;
```

### Behavior

`cargo antigen scan` propagates presentations from parent to descendant
through the inheritance chain. Witnesses on the parent may apply to the
descendant if structurally compatible (audit validates).

The `inherited_from` field on each `Presentation` carries a `ProvenanceEntry`
recording the chain (ADR-018 Option C). Diamond inheritance is dedup'd
correctly (multiple paths to the same parent produce one presentation,
not duplicates).

### Discipline

- **Cycles are caught**: A descended-from B descended-from A is detected
  via DFS white/gray/black coloring (ADR-005 Amendment 3 crash-resistance;
  ATK-A3-002)
- **Depth limit**: 64 levels by default, configurable via
  `[package.metadata.antigen]`
- **Orphaned references**: parents that no longer exist surface via
  `orphaned_lineage_edges()` query method

### See also

- ADR-008 (descended_from carrier)
- ADR-018 (propagation semantics + ProvenanceEntry + diamond dedup)
- [`usage-patterns.md`](usage-patterns.md) — inheritance patterns

---

## `#[antigen_tolerance(antigen_type, rationale = "...", until = "...")]`

Explicitly tolerate a fingerprint match the team has reviewed.

### Required arguments

- First positional: the antigen type the match was flagged against
- `rationale = "..."` (required) — narrative justification; required at parse time

### Optional arguments

- `until = "..."` (optional) — expiry date or condition (e.g., `"2026-12-31"`, `"v1.0"`)
- `see = [...]` (optional) — open-vocabulary cross-references (links to ADR, PR, issue)

### Applies to

Any Rust item that has been flagged by a fingerprint match the team
deliberately wants to retain.

### Example

```rust
use antigen::antigen_tolerance;
use crate::antigens::PolarityInvertedClassMeet;

// This test fixture deliberately constructs the inverted-polarity case
// to verify the fingerprint catches it.
#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "Test fixture that deliberately constructs the inverted case \
                 to verify the fingerprint catches it. Vulnerability is the point.",
    until = "2026-12-31",
)]
#[test]
fn test_fingerprint_detects_inverted_meet() {
    // ... test body
}
```

### Behavior

`cargo antigen scan` recognizes tolerances and reports tolerated sites
separately from unaddressed presentations. The audit tracks tolerance
status, including stale tolerances (where the underlying fingerprint
match no longer surfaces).

### Discipline

- **Rationale is required at parse time** — an empty or missing rationale
  is a compile error (ADR-011)
- **Until clauses are not enforced automatically** in v0.1.0-rc.1 — they
  surface in audit output for human review (future tooling may surface
  expired tolerances structurally)
- **Tolerance ≠ immunity**: tolerance acknowledges the failure-class is
  present and accepted; immunity claims the failure-class is structurally
  prevented. They are different lifecycle dispositions.

### See also

- ADR-011 (antigen_tolerance carrier with required rationale)
- [`usage-patterns.md`](usage-patterns.md) — when to tolerate vs claim immunity
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) — tolerance placement

---

## Compile-time validation

All five macros validate their input at compile time:

| Failure | Detected by | Result |
|---|---|---|
| Missing required field | proc-macro parser | Compile error with span pointing to the macro invocation |
| Non-kebab-case `name` | proc-macro parser | Compile error |
| Empty fingerprint string | proc-macro parser | Compile error |
| Unknown attribute field | proc-macro parser | Compile error (forward-compat: unknown fields rejected) |
| Empty `rationale` on tolerance | proc-macro parser | Compile error |
| Macro applied to wrong item kind (e.g., `#[antigen]` on a non-unit struct) | proc-macro parser | Compile error |

Span-aware error messages point at the specific token that's wrong (W4
substrate; ADR-005 Amendment 3 §Mechanics §3 crash-resistance).

trybuild fixtures in `antigen-macros/tests/ui/` codify the compile-error
contracts.

---

## Common patterns

### Co-locating presents + immune

```rust
#[presents(MyAntigen)]
#[immune(MyAntigen, witness = my_test)]
fn defended_function() { /* ... */ }
```

The presentation declares the vulnerable shape; the immunity declares
the defense. They commonly live on the same item.

### Single immune on the witness site

```rust
#[immune(MyAntigen, witness = comprehensive_proptest)]
#[proptest]
fn comprehensive_proptest(input: MyInput) { /* ... */ }
```

For composition-boundary antigens, the witness IS the test, and immunity
lives with it. See
[`usage-patterns.md`](usage-patterns.md#antigens-at-composition-boundaries).

### Tolerance for fixture-constructed cases

```rust
#[antigen_tolerance(
    MyAntigen,
    rationale = "Fixture constructs the vulnerable shape to test detection.",
)]
fn fixture_for_antigen_detection_test() { /* ... */ }
```

### Inheritance chain

```rust
#[antigen(...)] pub struct Generic;
#[antigen(...)] #[descended_from(Generic)] pub struct Specialized;
```

Witnesses on `Generic` may apply to `Specialized`; audit validates.

---

## What macros do NOT do

To prevent confusion:

- **They don't emit runtime code.** Identity transforms only. Zero runtime
  overhead; binary size and compile time are unaffected.
- **They don't validate witness existence.** Compile-time validation is
  syntactic only. Witness resolution happens at `cargo antigen audit` time.
- **They don't enforce naming consistency across crates.** Cross-crate
  identity uses `canonical_path` at `name@version` (ADR-017); the macros
  themselves only validate per-declaration syntax.
- **They don't expand into helper code.** No generated traits, no
  generated tests, no generated impls. The macros are pure pass-through.

---

## See also

- [`tutorial.md`](tutorial.md) — your first 15 minutes
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`where-to-look-for-antigens.md`](where-to-look-for-antigens.md) — placement conventions
- [`usage-patterns.md`](usage-patterns.md) — pattern recipes
- [`witness-tiers.md`](witness-tiers.md) — WitnessTier gradient semantics
- [`output-formats.md`](output-formats.md) — scan/audit output reference
- [`troubleshooting.md`](troubleshooting.md) — error diagnostics
- [`decisions.md`](decisions.md) — ratified ADRs

The macro implementations live in `antigen-macros/`. The crate-level
doc-comments in `antigen-macros/src/lib.rs` provide an alternative
view of the same surface oriented to `cargo doc` consumers.

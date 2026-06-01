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
| `#[presents]` | any item | Mark code as a site that exhibits a known failure-class |
| `#[defended_by]` | test / proptest fn | Register a test function as a **code-tier witness** for a failure-class (ADR-029) |
| `#[presents(requires=)]` | any item | Attach a **substrate-tier witness** predicate (sidecar evidence) to a presents-site |
| `#[presents(proof=)]` | any item | Attach a **phantom-type / formal-proof** witness to a presents-site |
| `#[immune]` | any item | *(Deprecated — use `#[defended_by]` or `#[presents(requires=)]` instead)* |
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

`cargo antigen scan` collects every `#[presents]` site and reports it.
`cargo antigen audit` then observes whether the site is *defended* (via
`#[defended_by]` on a test or `requires=` predicate) or *undefended*.
The audit output groups presentations by antigen type with per-site verdicts.

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

## `#[defended_by(antigen_type)]`

Register a test or proptest function as a **code-tier witness** for a failure-class (ADR-029).

This is the v0.2 primary idiom for code-tier defense. The macro is placed on the test function
(or proptest function), not on the vulnerable site. Audit cross-references it against all
`#[presents(AntigenType)]` sites and reports `defended at Reachability` when the test is
reachable, `defended at Execution` when it is confirmed executed.

### Required arguments

- First positional: the antigen type path (e.g. `PanickingInDrop` or `crate::antigens::Foo`)

### Applies to

Functions annotated with `#[test]`, `proptest!`, or any runnable test harness.

### Example

```rust
use antigen::defended_by;

#[test]
#[defended_by(PanickingInDrop)]
fn resource_handle_drop_does_not_panic() {
    let _ = ResourceHandle { id: 42 };
    // If drop panics, the test fails — correctly catching PanickingInDrop
}
```

The `#[presents(PanickingInDrop)]` marker stays on the Drop impl site; this test is the
evidence that the class is defended there.

### Key distinction from `#[immune]` (deprecated)

`#[defended_by]` registers evidence *at the witness site* — audit observes the defense.
The deprecated `#[immune]` was placed at the *vulnerable site* and *claimed* immunity.
The observe-not-declare inversion (ADR-029) means the site never asserts its own defense;
audit cross-references the evidence and reports the verdict.

### See also

- ADR-029 (Immunity Is Observed, Not Declared)
- `#[presents]` — marks the vulnerable site
- `#[presents(requires=)]` — substrate-tier witness for evidence outside the code

---

## `#[immune(antigen_type, witness = ..., rationale = "...")]`

> **Deprecated since ADR-029 (v0.2).** `#[immune]` is the v0.1 immunity-claim API and will emit
> a compiler deprecation warning. For v0.2, use:
> - **Code-tier defense**: `#[defended_by(AntigenType)]` on the test/proptest function
> - **Substrate-tier defense**: `#[presents(AntigenType, requires = <predicate>)]` on the site
> - **Phantom/formal-proof**: `#[presents(AntigenType, proof = <expr>)]` on the site
>
> Audit observes these registrations and reports per-site verdicts (`defended` / `undefended` /
> `substrate-gap`). The deprecated `#[immune]` form is still accepted for backwards compatibility
> but will continue to emit deprecation warnings guiding you toward the new idiom.

Declare immunity to a known failure-class, backed by a witness (deprecated — see above).

### Required arguments

- First positional: the antigen type
- `witness = <expr>` (required) — witness identifier (see "Witness types" below)

### Optional arguments

- `rationale = "..."` — narrative justification supplementing the executable witness

### Applies to

Any Rust item; typically co-located with `#[presents]` at the defended site.

### Witness types

`WitnessTier` in v0.1.0-rc.1 has four variants: `None`, `Reachability`,
`Execution`, `FormalProof`. The `Execution` tier requires the audit to
invoke a harness (A4-A5 work); v0.1 does not invoke harnesses, so
witnesses that *will* reach Execution in future versions sit at
Reachability today, with audit hints disambiguating the case. The table
below reports the **actual v0.1 tier** and the audit hint that
distinguishes the witness shape.

| Witness form | v0.1 tier (audit hint) | Example | Future promotion |
|---|---|---|---|
| `#[test]` function identifier | Reachability (`TestAttributePresentNotInvoked`) | `witness = no_panic_test` | Execution at A4-A5 (harness invocation) |
| `#[test] + #[ignore]` function | Reachability (`TestAttributePresentIgnoreSkipped`) | `witness = skipped_test` | (stays Reachability — `cargo test` skips by default) |
| `proptest!` function identifier | Reachability (`ProptestPresentNotInvoked`) | `witness = roundtrip_proptest` | Execution at A4-A5 (harness invocation) |
| `kani::fn_name` | Reachability (`ExternalToolPrefixRecognized`) | `witness = kani::no_panic_proof` | FormalProof at A4-A5 (verifier-invocation) |
| `prusti::fn_name` | Reachability (`ExternalToolPrefixRecognized`) | `witness = prusti::invariant_proof` | FormalProof at A4-A5 |
| `verus::fn_name` | Reachability (`ExternalToolPrefixRecognized`) | `witness = verus::correctness_proof` | FormalProof at A4-A5 |
| `creusot::fn_name` | Reachability (`ExternalToolPrefixRecognized`) | `witness = creusot::specification_proof` | FormalProof at A4-A5 |
| `clippy::lint_name` | Reachability (`ExternalToolPrefixRecognized`) | `witness = clippy::no_panic_in_drop` | Execution at A4-A5 (lint-invocation) |
| Phantom-type turbofish | FormalProof (`PhantomTypeShapeRecognized`) | `witness = NonPanickingProof::<MyType>::verified` | (already FormalProof) |
| Bare identifier (no test attr) | Reachability (`FunctionResolves`) | `witness = my_helper_fn` | (stays Reachability) |

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
- **Tier honesty**: external-tool delegations (kani, prusti, clippy, etc.)
  report Reachability tier with the `ExternalToolPrefixRecognized` hint
  in v0.1; harness invocation (A4-A5) will promote them to Execution or
  FormalProof when the tool actually runs and confirms
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
- [`usage-patterns.md`](usage-patterns.md) — when to tolerate vs defend
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

### Marking a site and registering its defense

```rust
// On the vulnerable site: declare only the shape it presents.
#[presents(MyAntigen)]
fn risky_function() { /* ... */ }

// On a test elsewhere: register the code-tier defense.
#[defended_by(MyAntigen)]
#[test]
fn my_test() { /* exercises the invariant */ }
```

Under ADR-029's observe-not-declare model the presents-marker and the defense are
*separate*: the site declares only the vulnerable shape, the defense evidence lives
on the witness, and `cargo antigen audit` cross-references them to report the per-site
verdict (`defended` / `undefended` / `substrate-gap`). The site never claims its own
immunity. (The deprecated `#[immune(..., witness=)]` form co-located a claim on the
site — see the deprecated-API section above.)

### Substrate-tier defense on the site

```rust
#[presents(MyAntigen, requires = signers(required = ["reviewer"]))]
fn governed_function() { /* ... */ }
```

When the defense is substrate state a test cannot execute — sidecar signers,
freshness, ratified docs — attach it as a `requires=` predicate on the presents-site
instead of a code-tier witness. See
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

# Antigen — Witness Tier Reference

> The `WitnessTier` gradient that `cargo antigen audit` reports for every
> `#[immune]` claim. This document explains what each tier means, when
> each applies, and how the audit reports them honestly per ADR-005
> Amendment 3.

For a tutorial introduction, see [`tutorial.md`](tutorial.md). For audit
output format, see [`output-formats.md`](output-formats.md).

---

## Why tiers (not just pass/fail)

Witness validation is not a binary "verified / not verified." Different
witness types provide different *strengths* of guarantee. A
`#[test]` function passing tells you the code ran without panicking on
the inputs the test exercised. A formal proof tells you the code is
correct on all possible inputs in the proven domain. A clippy lint
reference tells you a pattern-matcher external to antigen has been
configured to catch this case.

These are categorically different. Reporting them all as "verified"
would silently overclaim the weaker witnesses up to the strongest.
ADR-005 Amendment 3 (audit-tier-honesty) requires the audit to report
the *actual* strength of verification performed, never a stronger one.

The four tiers in v0.1.0-rc.1 (per `WitnessTier` enum in
`antigen/src/audit.rs`):

| Tier | Strength | When it applies |
|---|---|---|
| **FormalProof** | Mathematical guarantee on all inputs in the proven domain | Phantom-type witnesses (ADR-013) — turbofish pattern (`Foo::<T>::constructor`) recognized as a sealed type-system proof |
| **Execution** | Empirically verified on tested inputs | Reserved for A4-A5: requires the audit to actually invoke `cargo test` / proptest harness and confirm the witness passes. Not emitted in v0.1 |
| **Reachability** | Witness identifier resolves; audit has not verified runtime behavior | All v0.1 non-FormalProof resolutions: `#[test]` / `#[test]+#[ignore]` / `proptest!` / regular functions / external-tool prefixes (`clippy::`, `kani::`, etc.). The audit hint disambiguates which case |
| **None** | No witness present or witness fails to resolve | `Missing`, `NotFound`, or `Ambiguous` witness status — audit reports the gap honestly |

**Why only Reachability for `#[test]` in v0.1**: per ADR-005 Amendment 3
(audit-tier-honesty), the audit reports the work the audit ACTUALLY
PERFORMED. v0.1 walks the workspace and indexes functions; it does NOT
invoke `cargo test`. A `#[test]` function whose run was not invoked sits
at Reachability — its existence is verified, its passing is not.
Promotion to Execution tier requires harness invocation, planned for A4-A5.

**Disambiguating Reachability cases via audit hints**: although all
non-FormalProof resolutions collapse to Reachability tier in v0.1, the
parallel `AuditHint` field distinguishes them. The hint is what the
human-readable diagnostic emits and what consumers should match on:

| Witness shape | Tier | Audit hint |
|---|---|---|
| `#[test]` function (not ignored) | Reachability | `TestAttributePresentNotInvoked` |
| `#[test]` + `#[ignore]` | Reachability | `TestAttributePresentIgnoreSkipped` |
| `proptest!` macro | Reachability | `ProptestPresentNotInvoked` |
| Bare function, no test attribute | Reachability | `FunctionResolves` |
| External-tool prefix (`clippy::`, `kani::`, …) | Reachability | `ExternalToolPrefixRecognized` |
| Phantom-type turbofish (`Foo::<T>::ctor`) | FormalProof | `PhantomTypeShapeRecognized` |

---

## FormalProof tier

**Strength**: mathematical guarantee covering all inputs in the proven
domain.

**Recognized witnesses** (v0.1.0-rc.1):

- **Phantom-type witnesses** (ADR-013) — the type system itself proves
  immunity. Recognition shape: a path with turbofish syntax
  (`Foo::<TypeParam>::constructor`). The audit's
  `detect_phantom_type_witness` matches this shape and classifies as
  `WitnessKind::PhantomType { proof_type, type_params, constructor }`.

The witness IS the type structure; no runtime test needed because the
proof lives in the compiler's type-checking pass. Construction of the
proof token can only happen via the sealed constructor, so if the code
compiles, the proof holds.

**Example** (full version in `antigen/examples/phantom_witness.rs`):

```rust
use antigen::immune;
use std::marker::PhantomData;

pub struct NonPanickingProof<T> {
    _marker: PhantomData<T>,
    _seal: (),  // private field — only the sealed constructor can produce one
}

impl<T> NonPanickingProof<T> {
    pub const fn verified() -> Self {
        Self { _marker: PhantomData, _seal: () }
    }
}

pub struct PhantomVerifiedDropImpl;

#[immune(
    DropPanicClass,
    witness = NonPanickingProof::<PhantomVerifiedDropImpl>::verified,
    rationale = "Phantom-type token constructible only via the sealed `verified` constructor."
)]
impl Drop for PhantomVerifiedDropImpl { /* ... */ }
```

The turbofish form (`::<>`) is what triggers phantom-type recognition.
Audit reports FormalProof tier with the `PhantomTypeShapeRecognized`
hint when the shape recognizes. (Recognition is shape-only; behavioral
verification that the constructor IS sealed is the developer's
responsibility — the audit recognizes the shape but cannot prove the
constructor's soundness, so the hint name explicitly names the
recognition-but-not-validation surface.)

**Future tier extensions** (not v0.1.0-rc.1; see [`roadmap.md`](roadmap.md)):

- `kani::proof_fn` / `prusti::specification` / `verus::proof` /
  `creusot::specification` witnesses where the consuming workspace
  actually executes the verifier and the proof passes. v0.1.0-rc.1
  reports these as Reachability tier with the
  `ExternalToolPrefixRecognized` hint (see below); A4-A5 will ship the
  harness invocation that can promote them to FormalProof tier when the
  verifier confirms.

---

## Execution tier (reserved for A4-A5)

**Strength**: empirically verified by the audit actually invoking the
witness harness and confirming a passing run.

**Status in v0.1.0-rc.1**: NOT EMITTED. The Execution tier exists in the
`WitnessTier` enum as a forward-compatibility slot, but the v0.1 audit
does not invoke `cargo test`, the proptest harness, or any external
verifier. All test-function witnesses sit at `Reachability` tier with
their corresponding `TestAttributePresentNotInvoked` /
`ProptestPresentNotInvoked` / `TestAttributePresentIgnoreSkipped` hint.

**When Execution tier ships (A4-A5 per [`roadmap.md`](roadmap.md))**:

- `#[test]` function whose `cargo test` invocation passed → promoted from
  Reachability to Execution
- `proptest!` function whose harness invocation completed without
  property violation → promoted from Reachability to Execution
- External-tool prefixes (`kani::`, `prusti::`, `verus::`, `creusot::`,
  `flux::`) whose tool invocation produced a passing proof → promoted
  from Reachability to FormalProof (the external tool produces the
  guarantee; antigen recognizes the invocation result, not the proof
  structure)

**Example** (current v0.1 behavior — reports Reachability, not Execution):

```rust
use antigen::immune;
use crate::antigens::PanickingInDrop;

#[immune(
    PanickingInDrop,
    witness = safe_drop_no_panic_test,
    rationale = "SafeType::drop verified panic-free by direct test."
)]
impl Drop for SafeType { /* ... */ }

#[test]
fn safe_drop_no_panic_test() {
    drop(SafeType { data: None });
    drop(SafeType { data: Some(String::from("x")) });
}
```

In v0.1: the audit reports `tier = Reachability`, `hint =
TestAttributePresentNotInvoked` for this immunity claim — accurately
reflecting that the function exists with a `#[test]` attribute but its
run was not invoked by the audit. CI is responsible for actually running
the test; the audit reports the work IT performed, not what CI might do.

---

## Reachability tier

**Strength**: the function exists in scope; audit hasn't verified it was
executed or that it actually defends.

**When it applies**:

- Bare-identifier witness resolves to a workspace function that isn't
  recognized as `#[test]`, `#[proptest]`, or a known external-tool prefix
- The function exists; the audit can find it; but the audit doesn't
  know what kind of verification it provides

**Example**:

```rust
#[immune(MyAntigen, witness = my_helper_function)]
fn defended() { /* ... */ }

fn my_helper_function() { /* not a test, just a function */ }
```

The audit reports Reachability tier with audit-hint `FunctionResolves`.
This is **not an error** — it's honest reporting that the witness
exists but its semantic meaning is unverified.

**To upgrade to Execution tier**: convert the helper to a `#[test]`
function (or write a new test) and point the witness at it.

---

## External-tool witnesses (Reachability tier + ExternalToolPrefixRecognized hint)

**Note**: there is no separate `ExternalUnvalidated` tier in v0.1.0-rc.1.
External-tool witnesses resolve to `WitnessStatus::External { tool_hint }`,
which the audit maps to `WitnessTier::Reachability` with the
`ExternalToolPrefixRecognized` audit hint. The discipline is the same as
the legacy "ExternalUnvalidated" framing (external delegation is weaker
than execution); only the tier name differs.

**When it applies**:

- Witness names a known external tool: `clippy::`, `kani::`, `prusti::`,
  `verus::`, `creusot::`, `flux::`
- The consuming workspace can't (or doesn't) execute the external tool
  as part of `cargo antigen audit`

**Example**:

```rust
#[immune(MyAntigen, witness = clippy::no_panic_in_drop)]
impl Drop for SafeType { /* clippy lint covers this case */ }
```

Audit reports `tier = Reachability, hint = ExternalToolPrefixRecognized`
(JSON: `"reachability"` / `"external-tool-prefix-recognized"`). The
witness IS recognized; antigen delegates the actual verification to
clippy (compose-don't-compete per ADR-002), but clippy isn't executed
inside antigen's audit pipeline in v0.1.0-rc.1.

**Future upgrade** (per [`roadmap.md`](roadmap.md) Sweep A4): harness
invocation will execute the external tool and upgrade resolved witnesses
to Execution or FormalProof tier as appropriate. The
`ExternalToolInvoked` hint exists in the enum as a forward-compatibility
slot but is not emitted in v0.1.

**Cross-crate witnesses**: when a witness like `dep_crate::test_fn`
points to a function in a dependency that the consuming workspace can't
execute, the witness reports Reachability tier with the
`ExternalToolPrefixRecognized` hint by the same discipline (ADR-005
Amendment 3, A3.5 substrate).

---

## None tier

**Strength**: no witness present or witness fails to resolve. The
`WitnessTier::None` variant (serialized `"none"` in JSON).

**When it applies**:

- `#[immune]` without a `witness =` field → `WitnessStatus::Missing` →
  audit hint `NoneApplicable`
- `witness = fn_name` where `fn_name` doesn't exist in the workspace →
  `WitnessStatus::NotFound { reason }` → audit hint `NoneApplicable` (or
  `FabricatedPathPrefix` when the path's module prefix doesn't exist —
  ATK-A2-011)
- `witness = fn_name` where multiple functions named `fn_name` exist →
  `WitnessStatus::Ambiguous { candidates }` → audit hint `AmbiguousResolution`

**Example (witness not found)**:

```rust
#[immune(MyAntigen, witness = nonexistent_test)]
impl Drop for SafeType { /* ... */ }
```

Audit reports `tier = None, hint = NoneApplicable` and surfaces the
diagnostic:

```
⚠ MyAntigen — witness `nonexistent_test` not found in workspace
  Site: src/safe_type.rs:42
```

**Resolution paths** (audit suggests):

a) Add a `#[test]` function exercising the witness path (Execution tier upgrade)
b) Point the witness at a runnable test (`#[test]` without `#[ignore]`)
c) Rename colliding functions or qualify ambiguous witness paths
d) Add the witness function to the workspace if it's missing
e) Tolerate the gap with `#[antigen_tolerance(...)]` if intentional

---

## Audit hints

Each audit entry carries an `AuditHint` that explains the specific case
behind its tier. The hint is what consumers should match on for routing
logic — tier alone collapses several distinct shapes into the same value
(particularly at `Reachability`).

`AuditHint` is serialized **kebab-case** in JSON (per `#[serde(rename_all
= "kebab-case")]` in `antigen/src/audit.rs`); the Rust variant names are
PascalCase. The table below lists both forms.

| JSON hint | Rust variant | Resulting tier | Meaning |
|---|---|---|---|
| `phantom-type-shape-recognized` | `PhantomTypeShapeRecognized` | FormalProof | Turbofish phantom-type witness shape matched (ADR-013); constructor sealing not validated |
| `phantom-type-construction-validated` | `PhantomTypeConstructionValidated` | FormalProof | Phantom-type construction validated (future; not emitted in v0.1) |
| `test-attribute-present-not-invoked` | `TestAttributePresentNotInvoked` | Reachability | Function has `#[test]`; audit did not invoke `cargo test` |
| `test-attribute-present-ignore-skipped` | `TestAttributePresentIgnoreSkipped` | Reachability | Function has `#[test]` AND `#[ignore]`; `cargo test` would skip it by default |
| `proptest-present-not-invoked` | `ProptestPresentNotInvoked` | Reachability | `proptest!` macro invocation found; harness not invoked |
| `function-resolves` | `FunctionResolves` | Reachability | Workspace function exists with no testing attribute; behavior not verified |
| `external-tool-prefix-recognized` | `ExternalToolPrefixRecognized` | Reachability | External-tool prefix recognized (`clippy::`, `kani::`, …); tool not invoked |
| `external-tool-invoked` | `ExternalToolInvoked` | (future) | External tool actually invoked (A4+; not emitted in v0.1) |
| `ambiguous-resolution` | `AmbiguousResolution` | None | Witness name matches more than one workspace function (ATK-A2-005) |
| `fabricated-path-prefix` | `FabricatedPathPrefix` | None | Witness path's module prefix doesn't exist; last segment found but in an unrelated location (ATK-A2-011) |
| `none-applicable` | `NoneApplicable` | None | Catch-all for Missing / NotFound when no more-specific hint applies |
| `inherited-presentation-not-re-attested` | `InheritedPresentationNotReAttested` | (state-7 diagnostic, separate channel) | Inherited Presentation lacks re-attestation on the descendant site; state 7 of the 7-state interaction matrix (ADR-018). Surfaces via `audit.inherited_unaddressed[]` rather than as a per-immunity audit entry |

The complete hint set lives in `antigen/src/audit.rs::AuditHint`; this
table reflects v0.1.0-rc.1 hint enumeration.

---

## Reading audit output

Human-readable audit output groups by tier. The per-tier sub-counts
(`formal-proof`, `execution`) are emitted only when their count is
greater than zero; the conventional summary lines (`declared`,
`external`, `ambiguous`, `broken`, `missing`) are always emitted.

```
Auditing workspace: .

Audited 12 immunity claim(s):
  - 6 formal-proof (phantom-type or formal-verification tool — compile-time evidence)
  - 3 declared (witness identifier found in workspace — not yet semantically verified)
  - 1 external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - 0 ambiguous (witness name resolves to multiple workspace functions)
  - 2 broken (witness identifier not found)
  - 0 missing (no witness identifier)

✓ 6 immunity claim(s) at Execution tier or higher:
  path/to/file.rs:LINE  AntigenType (witness = `witness_expression`)
    tier = FormalProof, hint = PhantomTypeShapeRecognized
  ...

⚠ 6 immunity claim(s) below Execution tier:
  ...
```

The confirmed-claims section (added in A3.5 per ATK-A3-019) makes
FormalProof and Execution tier achievements visible in human output.
Adding a phantom-type witness now produces visible positive signal,
not silent classification.

---

## Tier-honesty discipline

Per ADR-005 Amendment 3, the audit:

- **Reports actual tier, not maximal tier**. If `proptest` is the
  witness, v0.1 audit reports Reachability with hint
  `ProptestPresentNotInvoked` — not Execution, because the audit did
  not invoke the proptest harness.
- **Surfaces below-Execution claims explicitly**. All claims at
  `Reachability` or `None` tier produce ⚠ warnings in human-readable
  output.
- **Never silently upgrades**. The witness shape determines the tier;
  the audit cannot promote a Reachability witness to Execution without
  evidence (which v0.1 cannot produce).
- **Reports external-tool delegation honestly**. A witness with an
  external-tool prefix (`clippy::`, `kani::`, …) reports Reachability
  tier with the `ExternalToolPrefixRecognized` hint — the prefix is
  recognized, the tool is not invoked.

This is sub-clause F (ADR-005) applied to the audit reporting surface:
every trust claim requires validation at the strength it's claimed,
and downgrades are explicit when validation can't reach the claimed
strength.

---

## Choosing a witness type

Practical guidance for v0.1.0-rc.1 (tier values reflect what the audit
actually emits; future tier-promotion paths noted in parentheses):

| If you have… | Use this witness | v0.1 tier (hint) |
|---|---|---|
| A `#[test]` function | `witness = test_fn_name` | Reachability (`TestAttributePresentNotInvoked`) — promotes to Execution at A4-A5 |
| A `proptest!` covering broad input space | `witness = proptest_fn_name` | Reachability (`ProptestPresentNotInvoked`) — promotes to Execution at A4-A5 |
| A phantom-type proof token | `witness = NonPanickingProof::<MyType>::verified` | FormalProof (`PhantomTypeShapeRecognized`) |
| A clippy lint rule | `witness = clippy::lint_name` | Reachability (`ExternalToolPrefixRecognized`) — promotes to Execution at A4-A5 |
| A formal proof in kani/prusti/verus/creusot/flux | `witness = kani::proof_fn` (etc.) | Reachability (`ExternalToolPrefixRecognized`) — promotes to FormalProof at A4-A5 |
| Just a helper function (not a test) | `witness = helper_fn` | Reachability (`FunctionResolves`) |
| Nothing yet — placeholder | `#[antigen_tolerance(...)]` with rationale | (tolerated; not an immunity claim) |

The discipline is honest: a theatrical test that always passes still
reports its tier honestly. The audit reports the shape it recognized,
not whether the witness actually verifies the failure class — semantic
verification (does the witness mean what it should?) is behavioral-tier
work for A4-A5. The [`troubleshooting.md`](troubleshooting.md) document
covers the "witness passes but doesn't mean what it should" failure-class
family (ATK-A2-003/004/005/011/012).

---

## See also

- ADR-002 (compose, don't compete — witness pluralism)
- ADR-005 (sub-clause F at every trust boundary)
- ADR-005 Amendment 3 (audit-tier-honesty)
- ADR-007 (anti-YAGNI: all witness families committed)
- ADR-013 (phantom-type witness recognition)
- [`macros.md`](macros.md) — `#[immune]` macro reference
- [`fingerprint-grammar.md`](fingerprint-grammar.md) — fingerprint DSL
- [`output-formats.md`](output-formats.md) — full audit output reference
- [`troubleshooting.md`](troubleshooting.md) — diagnostic guide

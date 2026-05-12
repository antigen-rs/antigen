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

The five tiers in v0.1.0-rc.1:

| Tier | Strength | When it applies |
|---|---|---|
| **FormalProof** | Mathematical guarantee on all inputs in domain | Phantom-type witnesses (ADR-013); formal-verification tool witnesses with executed proof |
| **Execution** | Empirically verified on tested inputs | `#[test]` or `proptest` witness function exists and is in scope |
| **Reachability** | Function exists in scope; not yet executed | Bare-identifier witness resolves to a function but audit hasn't verified runtime behavior |
| **ExternalUnvalidated** | External tool prefix accepted but execution not verified | `clippy::lint_name`, `kani::proof_fn` etc. where the consuming workspace can't execute the external tool |
| **Missing** | No witness present or witness fails to resolve | Audit reports the gap honestly |

---

## FormalProof tier

**Strength**: mathematical guarantee covering all inputs in the proven
domain.

**Recognized witnesses** (v0.1.0-rc.1):

- **Phantom-type witnesses** (ADR-013) — the type system itself proves
  immunity. Recognized shapes:
  - `Witnessed<T, W>` pattern
  - `typewit::TypeEq` pattern
  - Hand-rolled `PhantomData<T>` patterns that follow the convention

The witness IS the type structure; no runtime test needed because the
proof lives in the compiler's type-checking pass.

**Example**:

```rust
use antigen::immune;
use std::marker::PhantomData;
use crate::antigens::LatticeFrameInvariant;

pub struct DeterminismClass<F = LatticeFrameInvariant> {
    _frame: PhantomData<F>,
    // ...
}

#[immune(
    LatticeFrameInvariant,
    witness = phantom::DeterminismClass,
    rationale = "PhantomData<LatticeFrameInvariant> enforces frame invariant at type level."
)]
impl DeterminismClass { /* ... */ }
```

The `phantom::` prefix tells the audit "this witness is a phantom-type
pattern; verify the structural shape." Audit reports FormalProof tier
when the shape recognizes.

**Future tier extensions** (not v0.1.0-rc.1; see [`roadmap.md`](roadmap.md)):

- `kani::proof_fn` / `prusti::specification` / `verus::proof` /
  `creusot::specification` witnesses where the consuming workspace
  actually executes the verifier and the proof passes. v0.1.0-rc.1
  reports these as ExternalUnvalidated (see below) until A4 lands the
  harness invocation.

---

## Execution tier

**Strength**: empirically verified on the inputs the test exercised.

**Recognized witnesses**:

- `test::fn_name` — a `#[test]` function in the workspace
- `proptest::fn_name` — a property-based test in the workspace
- Bare identifier `fn_name` when the function resolves to a `#[test]`
  function

**Example**:

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

**Limitations**:

- Coverage depends on test inputs. A test that exercises one path and
  passes only verifies that path.
- `proptest` witnesses get Execution tier; coverage breadth depends on
  the Strategy quality.
- Audit reports Execution tier when the function exists; it does NOT
  run the test as part of the audit (that's CI's job).

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

## ExternalUnvalidated tier

**Strength**: external tool prefix accepted as a witness name; audit
hasn't executed the external tool.

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

Audit reports ExternalUnvalidated tier with audit-hint
`ExternalToolDelegated`. The witness IS recognized; antigen delegates
the actual verification to clippy (compose-don't-compete per ADR-002),
but clippy isn't executed inside antigen's audit pipeline in
v0.1.0-rc.1.

**Future upgrade** (per [`roadmap.md`](roadmap.md) Sweep A4): harness
invocation will execute the external tool and upgrade resolved witnesses
to Execution or FormalProof tier as appropriate.

**Cross-crate witnesses**: when a witness like `dep_crate::test_fn`
points to a function in a dependency that the consuming workspace can't
execute, the witness reports ExternalUnvalidated by design (ADR-005
Amendment 3 amendment from A3.5).

---

## Missing tier

**Strength**: no witness present or witness fails to resolve.

**When it applies**:

- `#[immune]` without a `witness =` field (audit surfaces "missing witness")
- `witness = fn_name` where `fn_name` doesn't exist in the workspace
  (audit surfaces "broken witness")
- `witness = fn_name` where multiple functions named `fn_name` exist
  (audit surfaces "ambiguous witness")

**Example (broken witness)**:

```rust
#[immune(MyAntigen, witness = nonexistent_test)]
impl Drop for SafeType { /* ... */ }
```

Audit reports Missing tier with audit-hint `WitnessNotFound`:

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

Each tier carries a more-specific `AuditHint` that explains the exact
case. The hints surface in JSON output (`audit --format json`) and
inform the human-readable diagnostic.

| Hint | Tier | Meaning |
|---|---|---|
| `PhantomTypeShapeRecognized` | FormalProof | Phantom-type witness shape matched (ADR-013) |
| `ExternalProofExecuted` | FormalProof | External formal-verification tool executed proof (future; A4+) |
| `TestFunctionPasses` | Execution | `#[test]` function verified passing |
| `ProptestFunctionPasses` | Execution | `proptest` function verified passing |
| `FunctionResolves` | Reachability | Workspace function exists; behavior not verified |
| `ExternalToolDelegated` | ExternalUnvalidated | External tool prefix accepted; tool not executed |
| `CrossCrateWitness` | ExternalUnvalidated | Witness in dependency; consuming workspace can't execute |
| `WitnessNotFound` | Missing | Identifier doesn't resolve to any workspace function |
| `WitnessAmbiguous` | Missing | Identifier resolves to multiple workspace functions |
| `WitnessMissing` | Missing | No `witness =` field present on `#[immune]` |
| `NoneApplicable` | Missing | (catch-all when no other hint applies) |

The complete hint set lives in `antigen/src/audit.rs::AuditHint`; this
table reflects v0.1.0-rc.1 hint enumeration.

---

## Reading audit output

Human-readable audit output groups by tier:

```
Auditing workspace: .

Audited 12 immunity claim(s):
  - 3 declared (witness identifier found in workspace — not yet semantically verified)
  - 1 external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)
  - 0 ambiguous (witness name resolves to multiple workspace functions)
  - 2 broken (witness identifier not found)
  - 0 missing (no witness identifier)
  - 6 confirmed (FormalProof tier — phantom-type witness shape recognized)

⚠ 6 immunity claim(s) below Execution tier:
  ...

✓ 6 immunity claim(s) at FormalProof tier:
  ...
```

The confirmed-claims section (added in A3.5 per ATK-A3-019) makes
FormalProof and Execution tier achievements visible in human output.
Adding a phantom-type witness now produces visible positive signal,
not silent classification.

---

## Tier-honesty discipline

Per ADR-005 Amendment 3, the audit:

- **Reports actual tier, not maximal tier**. If proptest is the witness,
  audit reports Execution (not FormalProof).
- **Surfaces below-Execution claims explicitly**. Reachability,
  ExternalUnvalidated, and Missing tiers all produce ⚠ warnings.
- **Never silently upgrades**. The witness shape determines the tier;
  the audit cannot promote a Reachability witness to Execution without
  evidence.
- **Reports cross-crate witnesses honestly**. A witness in a dependency
  is ExternalUnvalidated unless the consuming workspace can execute it.

This is sub-clause F (ADR-005) applied to the audit reporting surface:
every trust claim requires validation at the strength it's claimed,
and downgrades are explicit when validation can't reach the claimed
strength.

---

## Choosing a witness type

Practical guidance:

| If you have... | Use this witness | Tier |
|---|---|---|
| A passing `#[test]` exercising the failure-class | `witness = test_fn_name` | Execution |
| A proptest covering broad input space | `witness = proptest::strategy_fn` | Execution |
| A phantom-type pattern enforcing invariant at compile time | `witness = phantom::TypePattern` | FormalProof |
| A clippy lint rule that catches the pattern | `witness = clippy::lint_name` | ExternalUnvalidated |
| A formal proof in kani/prusti/verus/creusot/flux | `witness = kani::proof_fn` (etc.) | ExternalUnvalidated (v0.1) |
| Just a helper function (not a test) | `witness = helper_fn` | Reachability |
| Nothing yet — placeholder | `witness = todo` (or `#[antigen_tolerance]`) | Missing |

Tier achievement is honest. A theatrical test that always passes will
still report Execution tier — but won't actually verify anything. The
[`troubleshooting.md`](troubleshooting.md) document covers the
"witness passes but doesn't mean what it should" failure-class (the
ATK-A2-003/004/005/011/012 family).

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

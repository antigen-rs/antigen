# Antigen — Usage Patterns

> Concrete recipes for applying antigen's vocabulary to real-world failure
> classes. Each pattern answers: what kind of failure is this, where does it
> live in code, and which vocabulary primitives express it correctly.
>
> Patterns here have been through the encounter-tier (at least one concrete
> real-world instance) and retire-to-documentation disposition: existing
> vocabulary handles them without extension; the insight is about *how* to
> apply the vocabulary, not about adding to it.

---

## Antigens at composition boundaries

**When two independent implementations must agree**

### What this pattern is

Some failure-classes don't live at a single code site — they live at the
*boundary* between two implementations that are supposed to agree on some
property. An optimized kernel path and a reference path must produce the same
result. An incremental implementation and a full recompute must converge.
Two codepaths computing the same mathematical function through different
algebraic routes must match within tolerance.

These are **composition-boundary failure-classes**: the failure is not that
either implementation is wrong in isolation, but that their relationship
(the contract between them) can break silently.

### Canonical instance

In numerical math libraries: an `ExpKernelState` carries partial
intermediate values (`expm1_r`, shift `k`) and reconstructs the final
result via `(1 + expm1_r) << k`. A standalone `exp.rs` computes the same
quantity through a different code path. Both paths are correct individually;
they can diverge in edge cases (different rounding at intermediate steps,
different handling of subnormal inputs) in ways that neither path's own
unit tests detect.

The failure-class is: **cross-implementation consistency can silently break**.

### How to declare the antigen

Declare the antigen on the type or module that participates in the
composition — typically the kernel or internal implementation type:

```rust
#[antigen(
    name = "KernelReconstructionDivergence",
    summary = "Kernel state reconstruction and standalone implementation diverge \
               on edge-case inputs; neither path's unit tests catch the disagreement",
    fingerprint = r#"
        all_of([
            item = struct,
            name: matches("*KernelState*")
        ])
    "#,
    references = ["internal:exp-reconstruction-analysis"],
)]
pub struct ExpKernelState { ... }
```

> **v1 grammar note**: the v1 fingerprint grammar matches at item level —
> `item`, `name`, `variants`, `has_method`, `attr_present`, `doc_contains`,
> `body_contains_macro`. There is no `has_field_named` operator yet
> (body-level structure matching is W6b / ADR-015 work). The `name: matches`
> operator is the v1 proxy for naming-convention-based structural patterns.
> Antigen declarations for composition-boundary antigens typically rely on
> `#[presents]` at the consistency test site rather than passive fingerprint
> detection — the fingerprint here is a belt-and-suspenders passive scanner.

### Where to put `#[presents]`

**Mark the consistency test**, not the implementations.

The failure lives in the *relationship* between two code sites, not in
either site individually. In executable terms, the consistency test IS the
composition boundary — it's where the two paths meet and where divergence
becomes observable. The test function is the right proxy for the edge.

```rust
// tests/kernel_consistency.rs

#[presents(KernelReconstructionDivergence)]
#[proptest]
fn kernel_vs_standalone_agrees(input: f64) {
    let kernel_result = ExpKernelState::compute(input);
    let standalone_result = exp_standalone(input);
    prop_assert!((kernel_result - standalone_result).abs() < TOLERANCE,
        "kernel path and standalone path diverged at input={}", input);
}
```

**Why not mark both implementations?** The failure-class is a property of
their *relationship*, not of either implementation's code location. Marking
`ExpKernelState` and `exp_standalone` separately would suggest each is
independently vulnerable — but neither is. The shared antigen on the
consistency test correctly captures: this test is the site where the
failure-class matters and where immunity is demonstrated.

### How to claim immunity

Immunity is on the same site — the consistency test — once the test
actually verifies the invariant:

```rust
#[immune(KernelReconstructionDivergence,
    witness = proptest::kernel_vs_standalone_agrees)]
#[presents(KernelReconstructionDivergence)]  // still presents; immunity covers it
#[proptest]
fn kernel_vs_standalone_agrees(input: f64) { ... }
```

Alternatively, separate the declaration from the proof:

```rust
// The vulnerable boundary:
#[presents(KernelReconstructionDivergence)]
fn run_kernel_consistency_suite() { ... }

// The witness that defends it:
#[immune(KernelReconstructionDivergence,
    witness = proptest::kernel_vs_standalone_proptest)]
#[proptest]
fn kernel_vs_standalone_proptest(input: f64) { ... }
```

### Witness tier profile

Composition-boundary antigens get **ExecutionVerified** tier from proptest
witnesses — stronger than a single-input `#[test]` because property-based
testing explores the input space and is more likely to find edge cases where
divergence occurs.

For mathematical implementations where the domain is bounded and the divergence
profile is predictable, consider:

- `proptest` with a custom `Strategy` that focuses on edge cases (subnormal
  inputs, inputs near range boundaries, inputs where both paths take different
  code branches)
- Oracle-based witnesses: one path is the "reference" whose output is trusted;
  the other is the "optimized path" whose output must match

The tier does NOT reach **FormalProof** via proptest — proptest is empirical
coverage, not a mathematical guarantee. If `name@version` is
mission-critical, a phantom-type witness or formal verification adapter
(kani/creusot) is appropriate; see ADR-013 and the formal-verification
witness pattern.

### What the audit surfaces

With the pattern above, `cargo antigen audit` reports:

- The consistency test site as a **Presentation** (vulnerable to divergence)
- The proptest witness as **ExecutionVerified** (proptest ran and passed)
- The audit hint: "proptest witness — coverage depends on input strategy
  quality; add focused edge-case inputs if divergence has been observed"

Without `#[immune]`, the audit reports the site as an unaddressed
presentation: "this composition boundary has no verified consistency
witness." That's the signal to write the consistency test.

### What to watch for

- **Tests that always pass because the boundary is never actually exercised**:
  if the proptest `Strategy` generates inputs only in the "easy" region
  where both paths agree, the witness is theatrical (ATK-A3-011 class). Use
  domain knowledge to construct a strategy that hits the edge cases.
- **Marking the implementations instead of the consistency test**: this
  produces two unaddressed presentations (one per implementation) that never
  relate to each other. The audit can't see the connection. Mark the boundary
  (the consistency test), not the endpoints.
- **Using a unit test instead of a proptest**: a single-input consistency
  check proves agreement at one point. Proptest gives broader coverage.
  For mathematical correctness, proptest or a formal tool is the right
  witness tier.

---

## References

- [`docs/decisions.md`](decisions.md) — ADR-002 (compose-not-compete),
  ADR-005 (sub-clause F + witness tier honesty), ADR-013 (phantom-type
  witness recognition + WitnessTier gradient)
- [`docs/testing-patterns.md`](testing-patterns.md) — property test
  conventions, failing-as-passing pattern
- [`docs/glossary.md`](glossary.md) — antigen, presentation, immunity,
  witness, WitnessTier

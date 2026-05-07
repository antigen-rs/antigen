# Antigen — API Shape

> Sketch of the API surface as imagined pre-team. Future JBD team refines, contradicts,
> and ratifies. This is a *captured shape* document, not a specification.

## Macro primitives — three verbs

### Build: declaring a failure-class

```rust
// In any crate; gets exported and re-used by consumers.
#[antigen(
    name = "frame-translation",
    family = "semantic-drift",
    fingerprint = SemanticFingerprint::polarity_inverted_when_strongest_first,
    summary = "Two locally-consistent frames produce wrong results at their interface \
               because the same word means different things in each frame.",
    references = ["https://github.com/.../meet-min-vs-max-incident"],
)]
pub struct FrameTranslation;
```

What this generates:
- A type-level marker (`FrameTranslation`)
- A registration in the cargo-antigen tooling's discoverable graph
- Optional structural-fingerprint matcher that `cargo antigen scan` uses

### Give: presenting an antigen (vulnerability)

```rust
// On a function, type, trait, or method — anywhere a failure-class might surface.
#[presents(FrameTranslation)]
pub fn meet(a: Class, b: Class) -> Class {
    // ... implementation ...
}
```

What this generates:
- A compile-time annotation discoverable by `cargo antigen scan`
- If no matching `#[immune]` is found, this is FLAGGED in audits
- Optional cytokine-style propagation: callers of this function may inherit a
  `#[derived_presentation]` marker

### Immune: declaring immunity with witness

```rust
#[immune(
    FrameTranslation,
    witness = meet_polarity_proptest,  // names a #[test] or proptest function
    rationale = "Class lattice strengthens upward; meet returns the strictly-looser of \
                 two classes. Proptest enumerates all 16 (a, b) pairs and verifies \
                 (Strict, X) → X for all X."
)]
pub fn meet(a: Class, b: Class) -> Class {
    // ... implementation matched by the witness ...
}
```

The `witness` is required — a marker without proof is not a claim. Witnesses can be:
- A `#[test]` function in the same module
- A `proptest!` macro invocation
- A phantom-type construction proof (advanced)
- A `kani::proof` or `prusti::trusted` annotation (delegated formal verification)
- A `clippy_lint!` reference (delegated lint)

### Inherit: structural propagation

```rust
#[descended_from(crate::other::meet)]
pub fn refined_meet(a: RefinedClass, b: RefinedClass) -> RefinedClass {
    // ... implementation ...
}
```

What this generates:
- All `#[presents]` markers on the source function propagate (with weakening if
  appropriate)
- All `#[immune]` markers either propagate (if witness still applies) or are FLAGGED as
  needing re-justification

## Cargo subcommands

### `cargo antigen scan`

Walk the codebase. For every `#[presents(X)]`, check if there's a corresponding
`#[immune(X, witness = ...)]` either on the same item or transitively via
`#[descended_from]`. Report:

- Unaddressed presentations (vulnerable code with no immunity claim)
- Stale immunity (witness function missing or failing)
- Inheritance chains that lost immunity (descended-from with weakened witness)

Output formats: human-readable, JSON for CI integration, SARIF for IDE consumption.

### `cargo antigen new <name>`

Scaffold a new antigen declaration. Interactively asks for:
- Family (must be one of the 8 first-principles classes, or `custom`)
- Brief summary
- Structural fingerprint (free text now; structural-pattern grammar later)
- Initial witness pattern (test? proptest? phantom-type? lint?)

Generates a starter declaration file in the user's choice of crate.

### `cargo antigen vaccinate <antigen> <pattern>`

Apply a known immunity pattern across a structural family. e.g.:

```
$ cargo antigen vaccinate FrameTranslation 'enum *Class'
Found 3 enum types matching `*Class`:
  - tambear::lattice::CommutativityClass     [no immune declaration]
  - tambear::lattice::PrecisionClass         [no immune declaration]
  - tambear::lattice::DeterminismClass       [already immune]

Apply meet_polarity_proptest pattern to first two? [Y/n]
```

This is the *vaccination* surface: known immunity propagates to the structural family.

### `cargo antigen audit`

Comprehensive coverage report. Per-crate, per-family, per-named-antigen statistics.
Suitable for CI dashboards. Identifies:

- Coverage trend (immunity growing or shrinking over commits)
- High-impact unimmunized presentations (many descendants, no immunity)
- Antigens with no presentations (might be obsolete?)
- Cross-crate antigen propagation (which antigens are imported and used downstream?)

## Witness types — proof-carrying immunity

The `witness` parameter on `#[immune]` enforces the rule: **a marker without proof is not
a claim**. Acceptable witness shapes (sketch; future-team refines):

### Test witness

```rust
#[test]
fn meet_polarity_proptest() {
    // proptest body that exercises the failure pattern explicitly
}

#[immune(FrameTranslation, witness = meet_polarity_proptest)]
pub fn meet(...) { ... }
```

The witness function is run by `cargo test` AND validated by `cargo antigen scan` (which
checks the witness exists, runs, and asserts something structurally).

### Proptest witness

```rust
proptest! {
    #[test]
    fn meet_polarity_under_all_pairs(a in any::<Class>(), b in any::<Class>()) {
        let result = meet(a, b);
        prop_assert!(result <= a && result <= b);
        prop_assert!(meet(a, b) == meet(b, a));
    }
}
```

### Phantom-type witness (advanced)

```rust
struct PolarityProof<T>(PhantomData<T>);

impl PolarityProof<FrameTranslation> {
    pub fn established_by_construction() -> Self {
        // The construction itself encodes the proof.
        // Compile-time-impossible-to-construct unless meet's polarity is correct.
        PolarityProof(PhantomData)
    }
}

#[immune(FrameTranslation, witness = PolarityProof::<FrameTranslation>::established_by_construction)]
pub fn meet(...) { ... }
```

The strongest form. Compile-time impossible to violate. Expensive to design.

### Delegated witness

```rust
#[immune(
    PanickingInDrop,
    witness = clippy::lints::no_panic_in_drop,
)]
impl Drop for MyType { ... }
```

Antigen *delegates* to clippy. The lint enforces the immunity; antigen makes it visible
that the delegation exists.

## Composition rules

How antigen markers propagate through the type system:

### Through derivation (`#[descended_from]`)

- All `#[presents]` markers propagate from source to derived
- All `#[immune]` markers propagate IF the witness still applies; otherwise FLAGGED
- The propagation is recorded in cargo-antigen's graph for audit purposes

### Through trait implementation

When a trait declares `#[presents(X)]` on a method, every `impl` of that trait must
either:
- Re-declare the presentation (acknowledging the inherited vulnerability)
- Declare immunity with a witness specific to the impl
- Use `#[immune_via_trait_blanket]` if a blanket impl provides the immunity

### Through function calls

By default, callers do NOT inherit `#[presents]` markers. The vulnerability is local to
the called function. (Unlike biological cytokine propagation, function-call propagation
would explode the marker count.) Optional opt-in via `#[propagates_presentations]` for
specific cases where the vulnerability genuinely flows through.

### Through generic instantiation

When a generic function is instantiated with a type that has `#[presents(X)]` markers,
the instantiation may or may not inherit the markers depending on whether the generic
function actually exercises the vulnerable behavior. This requires structural analysis;
default behavior is "do not inherit, but flag for human review."

## Test-time integration

Antigen integrates with `cargo test` through the witness mechanism. Three integration
patterns:

1. **Direct integration**: antigen witness IS a `#[test]` function. Runs via cargo test.
2. **Proptest integration**: witness IS a `proptest!` block. Runs via cargo test.
3. **Antigen-test integration**: `cargo antigen test` runs ONLY the witness functions for
   diagnostic purposes; useful when developing the antigen library.

## Compile-time integration

For phantom-type and structural-pattern witnesses, antigen integrates at compile time:
- Phantom-type witnesses are checked by the type system (impossible to compile if invalid)
- Structural-pattern witnesses are checked by the cargo-antigen proc-macro at expansion
- Custom diagnostics surface antigen failures with the antigen's name in the error message
  (e.g., "antigen FrameTranslation: meet polarity violated; expected strongest-as-top")

## Configuration

`Cargo.toml`:
```toml
[package.metadata.antigen]
# Required immunity for these antigens (CI fails if any presentation lacks immunity)
required = ["FrameTranslation", "BoundaryViolation"]

# Recommended (warnings, not errors)
recommended = ["StaleContext", "OptionalityCollapse"]

# Antigen library imports
imports = ["antigen-stdlib", "tambear-antigens"]

# Custom witness directory (where the proof functions live)
witness_dir = "tests/antigen_witnesses/"
```

## What this API shape DOESN'T address (yet)

- Cross-crate antigen versioning (when an antigen's fingerprint changes, what happens to
  consumers' immunity claims?)
- Anti-pattern: someone declaring `#[immune]` without a real witness, just to silence
  audits
- Performance: scanning a large workspace must stay fast; what's the index strategy?
- IDE integration: how does rust-analyzer surface antigen presentations inline?
- Privacy: should antigen declarations be public or private by default? (lean public —
  consumers benefit from knowing what their dependencies are vulnerable to)

These are open questions for the JBD team.

# Antigen — Composing With Your Existing Tools

> **Idiom note**: Code examples in this doc use the current (ADR-029) idiom —
> `#[defended_by(X)]` on test functions (code-tier), `#[presents(X, requires=...)]`
> for substrate evidence, and `#[presents(X, proof=...)]` for phantom-type and
> external-tool witnesses. The old `#[immune(...)]` form was **removed** (ADR-029) —
> see the [migration guide](immune-migration-guide.md) if you still have `#[immune]` sites.

> Antigen doesn't replace your existing testing, linting, or
> verification toolchain. It *composes* with them — using each for
> what each does best. This doc shows how antigen fits alongside the
> tools you're already using.

For the architectural framing of "compose, don't compete" (ADR-002),
see [`decisions.md`](decisions.md). For the witness-tier semantics
that operationalize this composition, see
[`witness-tiers.md`](witness-tiers.md).

---

## The composition posture

Antigen is a vocabulary for structural failure-class memory. The
*verification* that a defense holds lives in whichever tool
proves it best:

- **Tests** verify behavior on specific inputs
- **Property tests** verify behavior across input spaces
- **Linters** catch style + common-mistake patterns
- **Formal verification** proves correctness across all inputs in
  domain
- **Type system** enforces invariants at compile time

Antigen delegates to whichever proves the defense for a given antigen.
It doesn't reinvent verification; it threads existing verification
under a shared vocabulary of *what's being defended against*.

---

## Antigen + clippy

**Clippy** catches style violations and common-mistake patterns.
**Antigen** catches *named failure-class patterns* with structural
fingerprints.

They compose at the witness layer:

```rust
// clippy lint catches the panic pattern; we configure the lint at warn level.
#[presents(PanickingInDrop, proof = clippy::panic_in_result_fn)]
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        // implementation
    }
}
```

The `proof = clippy::lint_name` witness tells the audit "this defense is
backed by a clippy lint." The audit reports the tier honestly
(`Reachability` + `ExternalToolPrefixRecognized` hint): it recognizes the clippy
prefix but does not run clippy. Executing the lint and promoting to `Execution`
where appropriate is a recorded graduation path; see [`roadmap.md`](roadmap.md).

**Why they compose**:

- Clippy is a *general* mistake-catcher; antigen is a *team-specific
  failure-class memory*. Both matter; they don't overlap functionally.
- Clippy's lint surface is enormous (700+ lints); antigen's vocabulary
  catalogs the failure-classes your team has *learned about specifically*.
- When a clippy lint catches a pattern your team has named as an
  antigen, the defense makes that connection explicit.

**What antigen adds beyond clippy alone**: structural memory of *why*
the lint matters in your project. Clippy says "this might be a panic";
antigen says "this pattern is `PanickingInDrop` which caused the
incident in PR #1234 (see references)."

---

## Antigen + proptest

**Proptest** explores input spaces to find violations of properties.
**Antigen** names what those violations would be.

```rust
use antigen::{antigen, defended_by, presents};
use proptest::prelude::*;

#[antigen(
    name = "polarity-inverted-class-meet",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("meet", "(self, Self) -> Self")"#,
    summary = "Class enums must use max (not min) for lattice meet.",
)]
pub struct PolarityInvertedClassMeet;

// The site presents the shape; the proptest below registers the defense.
#[presents(PolarityInvertedClassMeet)]
pub enum DeterminismClass {
    Strict, Loose, Loose2,
}

impl DeterminismClass {
    pub fn meet(self, other: Self) -> Self {
        // implementation must use max for lattice-meet semantics
        if (self as u8) > (other as u8) { self } else { other }
    }
}

proptest! {
    // Proptest verifies meet returns the larger discriminant across the full enum space.
    #[test]
    #[defended_by(PolarityInvertedClassMeet)]
    fn meet_is_max_proptest(a: u8, b: u8) {
        let a = DeterminismClass::from(a % 3);
        let b = DeterminismClass::from(b % 3);
        let meet = a.meet(b);
        prop_assert!((meet as u8) >= (a as u8));
        prop_assert!((meet as u8) >= (b as u8));
    }
}
```

The proptest witness gives broader coverage than a `#[test]` function
because it explores input space; antigen names the *category of
failure* that proptest is defending against.

---

## Antigen + formal verification (kani / prusti / verus / creusot / flux)

Formal verification tools prove correctness across all inputs in their
proven domain. Antigen names what's being proven.

```rust
// kani verifies no out-of-bounds access for all symbolic inputs.
#[presents(UnsafePointerArithmetic, proof = kani::no_oob_proof)]
unsafe fn safe_indexed_access(buf: &[u8], idx: usize) -> Option<u8> {
    // ...
}
```

Formal-verification witnesses report `Reachability` tier with the
`ExternalToolPrefixRecognized` hint: the audit recognizes the kani prefix but does
not invoke kani, so it cannot confirm the proof passes. Harness invocation — running
the verifier and promoting a confirmed proof to `FormalProof` tier — is a recorded
graduation path; see [`roadmap.md`](roadmap.md).

The composition: formal verification provides the *strongest possible*
witness; antigen provides the *vocabulary that names what each proof
demonstrates*.

---

## Antigen + phantom-type patterns

**Phantom types** prove invariants at compile time. **Antigen**
recognizes the proof shape.

```rust
use antigen::presents;
use std::marker::PhantomData;
use crate::antigens::FrameTranslationDrift;

pub struct NonPanickingProof<T> {
    _marker: PhantomData<T>,
    _seal: (),  // private — only the sealed constructor can produce one
}

impl<T> NonPanickingProof<T> {
    pub const fn verified() -> Self {
        Self { _marker: PhantomData, _seal: () }
    }
}

pub struct PhantomVerifiedDropImpl;

// Phantom-type token constructible only via the sealed `verified` constructor.
#[presents(
    FrameTranslationDrift,
    proof = NonPanickingProof::<PhantomVerifiedDropImpl>::verified
)]
impl Drop for PhantomVerifiedDropImpl { /* ... */ }
```

The turbofish syntax (`NonPanickingProof::<PhantomVerifiedDropImpl>::verified`)
is what antigen recognizes as a phantom-type witness. Audit reports
`FormalProof` tier with `PhantomTypeShapeRecognized` hint.

This is the strongest tier antigen recognizes — the type
system itself is the witness, so if the code compiles, the proof
holds.

See [`witness-tiers.md`](witness-tiers.md#formalproof-tier) for the
full pattern.

---

## Antigen + tests

The most common composition. Tests verify behavior; antigen names what
class of failure the test defends against.

```rust
#[antigen(
    name = "off-by-one-in-bounds-check",
    fingerprint = r#"item = fn, name = matches("*_bounds")"#,
    summary = "Bounds-check functions commonly off-by-one; verify exact boundary behavior.",
)]
pub struct OffByOneInBoundsCheck;

#[presents(OffByOneInBoundsCheck)]
pub fn validate_in_bounds(idx: usize, len: usize) -> bool {
    idx < len
}

// Test exercises both inclusive boundaries explicitly.
#[test]
#[defended_by(OffByOneInBoundsCheck)]
fn bounds_check_boundary_test() {
    assert!(validate_in_bounds(0, 1));      // lower inclusive
    assert!(validate_in_bounds(0, 10));
    assert!(validate_in_bounds(9, 10));     // upper exclusive bound
    assert!(!validate_in_bounds(10, 10));   // off-by-one would mis-handle this
    assert!(!validate_in_bounds(0, 0));
}
```

Audit reports `Reachability` tier with the
`TestAttributePresentNotInvoked` hint: the test is wired, not run. The tier upgrade
that comes with harness invocation is a recorded graduation path; see
[`roadmap.md`](roadmap.md).

---

## Antigen + ADR culture

If your team uses ADRs (Architectural Decision Records), antigen
makes ADR claims structurally enforceable:

```rust
#[antigen(
    name = "configuration-loaded-without-cache-invalidation",
    summary = "Configuration loads must trigger cache invalidation per ADR-042.",
    references = [
        "adr:internal-ADR-042",
        "pr:internal/repo#1234",
    ],
    fingerprint = r#"item = fn, name = matches("*reload_config*")"#,
)]
pub struct ConfigLoadWithoutCacheInvalidation;
```

The `references` field links the antigen to ADR-042. When a new
developer (or LLM agent) encounters a `reload_config*` function in
your codebase, `cargo antigen scan` surfaces it, and they can follow
the ADR reference to understand the lesson.

Your ADR culture becomes *structurally checkable* rather than just
documented.

---

## Antigen + CI

Standard composition: run `cargo antigen scan` and `cargo antigen
audit` as CI gates. By default (warn-not-error), both exit 0 even with
unaddressed presentations — adopters make conscious decisions about
each site rather than CI failing on a fresh codebase.

Add `--strict` to fail CI when unaddressed presentations or
below-Execution witnesses exist: the run then exits non-zero so CI can
gate on it. Strict mode is the opt-in enforcement surface — without it,
findings are reported loudly but the run still succeeds.

---

## Antigen + your knowledge ecosystem

Antigen's `references` field is open-vocabulary. It accepts:

- URLs (`https://...`)
- GitHub issues/PRs (`owner/repo#123`)
- CVE identifiers (`CVE-2024-12345`)
- RFC references (`rfc:2119`)
- ADR identifiers (`adr:internal-042`)
- Internal post-mortem references
- Any string your team finds useful

The composition: each antigen declaration becomes a *node* in your
team's knowledge graph, bridging the structural failure-class memory
in code to the lived context in other systems (Jira, Linear, Slack
archives, blog post archives, ADR directories).

When an LLM agent encounters an antigen declaration during code
review, the references field tells them *where else to look* for
context.

---

## What antigen does NOT compose with

**Antigen does not delegate verification to itself** — there's no
recursive antigen-verifies-antigen layer. The witness layer is always
external to antigen's vocabulary surface.

**Antigen does not extend type-checking** — it operates on attribute
syntax that the compiler treats as pass-through. The fingerprint
matching happens at `cargo antigen scan` time, not at `cargo build`
time.

**Antigen is not a build-time analyzer** in the rustc-plugin sense.
The scan is a separate pass that runs out-of-band.

**Antigen does not replace your runtime error handling**. It captures
patterns; your error handling logic does the actual safe-cleanup work.

---

## When you don't need antigen

Antigen is most valuable for:

- Codebases with accumulated tribal knowledge
- Teams that have had "the same bug twice"
- Projects where developers (human or LLM) cycle frequently
- Long-lived codebases where lessons need to outlive their teachers

If your codebase is short-lived (a one-off script), or single-author,
or has no accumulated failure-class memory yet, antigen may be premature.
That's fine — the project's recognition-not-design discipline (ADR-006)
explicitly says don't add antigens without instances. Same applies at
the adoption level.

---

## A composition example: full stack

Here's how a hypothetical team composes antigen with their full
toolchain:

```rust
// src/antigens.rs — team's failure-class memory
use antigen::antigen;

#[antigen(
    name = "shared-mutable-state-without-lock",
    family = "concurrency-hazard",
    fingerprint = r#"
        item = fn,
        any_of([
            name = matches("*shared_*"),
            attr_present("shared"),
        ])
    "#,
    summary = "Shared-mutable-state functions must hold a lock; race conditions when not.",
    references = [
        "adr:internal-ADR-038",
        "issue:internal/repo#892",
        "https://blog.example.com/concurrency-postmortem-2024",
    ],
)]
pub struct SharedMutableStateWithoutLock;
```

```rust
// src/state.rs — vulnerable site
// The defense is registered on the loom test with #[defended_by(SharedMutableStateWithoutLock)]:
// a loom-based concurrent test exercises the lock paths across all interleavings.
#[presents(SharedMutableStateWithoutLock)]
pub fn update_shared_counter(value: i64) -> Result<(), Error> {
    let mut counter = SHARED_COUNTER.lock().map_err(|_| Error::LockPoisoned)?;
    *counter += value;
    Ok(())
}
```

Composition at multiple layers:
- **Antigen vocabulary**: declares the failure-class
- **Loom**: provides the concurrent test witness
- **References**: point to ADR + post-mortem + tracking issue
- **Clippy** (configured separately): catches related but distinct patterns
- **CI**: runs `cargo antigen audit` + `cargo test` + `cargo clippy`

Each tool does what it does best. Antigen threads them under a shared
vocabulary of *what's being defended*.

---

## See also

- [`concepts.md`](concepts.md) — architectural concepts
- [`witness-tiers.md`](witness-tiers.md) — witness tier semantics
- [`macros.md`](macros.md) — full macro reference
- [`decisions.md`](decisions.md) — especially compose, don't compete, and
  phantom-type witness recognition
- [`testing-patterns.md`](testing-patterns.md) — when/how
  testing-and-antigen co-operate

---

*Antigen is one pillar of three. Tests check behavior; documentation
records decisions; antigen captures structural failure-class memory.
Each pillar holds something the others can't. Composition is the
discipline.*

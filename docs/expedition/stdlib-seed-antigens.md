# Antigen-stdlib — Seed antigens

> Ten concrete antigen declarations drawn from the failure-class-instances research.
> These are the **seed catalog** for the eventual `antigen-stdlib` companion crate.
> Each maps to one or more real Rust ecosystem failures, has a proposed structural
> fingerprint following ADR-010, and identifies witness mechanisms that can DELEGATE
> to existing tools (per ADR-002).
>
> **Status**: Pseudocode declarations. Syntax conforms to ADR-009 (adoption gradient)
> and ADR-010 (fingerprint grammar v1). The team refines and ratifies during sweep A5.

## Purpose

The seed antigens move antigen from "framework" to "content." A user adopting
antigen-stdlib v0.1 gets immediate value because these ten declarations cover
real, recurring Rust ecosystem failure patterns — they don't have to write
their own antigens to start.

The selection criteria for v0.1 inclusion:
- **Multiple real-world instances** documented in `failure-class-instances.md`
- **Clear structural fingerprint** matchable by ADR-010's grammar
- **Existing witness mechanism** in the Rust ecosystem (clippy, proptest, kani,
  etc.) so antigen DELEGATES rather than reinvents
- **Universal applicability** — these patterns appear across many Rust projects, not
  just niche cases

The 10 antigens map across all 8 first-principles failure classes, with bias toward
classes that have strong ecosystem coverage (so witnesses are easier) AND classes
that are ANTIGEN-NATIVE (so antigen ships value others can't).

---

## Antigen 1: `polarity-inverted-class-meet`

**Family**: `frame-translation`
**Source pattern**: tambear DeterminismClass GAP-BIT-EXACT-1; CommutativityClass
near-miss documented in `case-study-determinism-class.md`.

```rust
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    summary = "Class enum with strongest-first discriminants must use max (not min) \
               for lattice meet. The lattice ordering is reverse-strictness vs the \
               discriminant ordering; meet returns the lattice-weaker (discriminant-\
               larger) variant.",
    fingerprint = "
        item: enum,
        name: matches('*Class'),
        variants: 3..=8,
        has_method('meet', '(Self, Self) -> Self'),
        any_of([
            attr_present('repr(u8)'),
            doc_contains('strength'),
            doc_contains('lattice'),
            doc_contains('meet')
        ])
    ",
    references = [
        "https://github.com/tambear-rs/tambear/issues/...",  // GAP-BIT-EXACT-1
    ],
)]
pub struct PolarityInvertedClassMeet;
```

**Witness pattern**: proptest verifying meet returns the lattice-weaker variant
(see `case-study-determinism-class.md` for full witness).

**Why ship in stdlib**: tambear's domain produced it, but the pattern generalizes —
any project building lattice-based class hierarchies risks this. ULID, version,
priority, severity enums all have similar shape.

---

## Antigen 2: `panicking-in-drop`

**Family**: `boundary-violation`
**Source pattern**: classic Rust footgun; tracked in clippy's `unwrap_in_drop` lint.

```rust
#[antigen(
    name = "panicking-in-drop",
    family = "boundary-violation",
    summary = "Drop impls must not panic. Panic during Drop while another panic is \
               unwinding causes process abort. Code in Drop must use panic-free \
               error handling — log, swallow, or use catch_unwind explicitly.",
    fingerprint = "
        item: impl,
        trait_name: 'Drop',
        body_contains_any: [
            'unwrap()', 'expect(', 'panic!(', 'unreachable!()',
            'assert!(', 'assert_eq!(', 'assert_ne!(',
            'todo!()', 'unimplemented!()',
            'array_index_with_constant_or_runtime'
        ]
    ",
    references = [
        "https://rust-lang.github.io/rust-clippy/master/index.html#panic_in_drop",
        "https://doc.rust-lang.org/std/ops/trait.Drop.html#panics",
    ],
)]
pub struct PanickingInDrop;
```

**Witness mechanism**: delegates to clippy
```rust
#[immune(PanickingInDrop, witness = clippy::panic_in_drop)]
impl Drop for SafeType { ... }
```

When the consumer's CI runs clippy and the lint passes for the impl block, immunity
holds. cargo-antigen audit verifies clippy was run.

**Why ship in stdlib**: every Rust project with non-trivial Drop has this risk;
existing clippy coverage makes the witness trivially adoptable.

---

## Antigen 3: `mutexguard-cross-await-point`

**Family**: `incompatible-merger`
**Source pattern**: well-known async + mutex pitfall; tracked by clippy's
`await_holding_lock`.

```rust
#[antigen(
    name = "mutexguard-cross-await-point",
    family = "incompatible-merger",
    summary = "MutexGuard (and parking_lot variants) held across an .await point in \
               an async fn risks deadlock when the executor schedules another task \
               on the same mutex. Drop the guard before awaiting, or use an async-\
               aware mutex (tokio::sync::Mutex, async_lock::Mutex).",
    fingerprint = "
        item: fn,
        is_async: true,
        body_contains: 'MutexGuard with await downstream'
        // The actual fingerprint requires AST-walk: detect MutexGuard binding,
        // detect any .await between binding and drop. ADR-010's visitor handles.
    ",
    references = [
        "https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_lock",
        "https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use",
    ],
)]
pub struct MutexGuardCrossAwaitPoint;
```

**Witness mechanism**: delegates to clippy `await_holding_lock` lint.

**Why ship in stdlib**: async-Rust adoption is universal; this footgun is one of
the top onboarding hazards for async; clippy already has the detection.

---

## Antigen 4: `nan-comparison-trap`

**Family**: `frame-translation`
**Source pattern**: floating-point comparison surprise; widely-known but un-named.

```rust
#[antigen(
    name = "nan-comparison-trap",
    family = "frame-translation",
    summary = "f32/f64 comparison with NaN always returns false (PartialOrd). Code \
               that branches on float comparison without explicit NaN handling \
               produces silently wrong results when NaN appears. Use total_cmp(), \
               is_nan() guard, or ordered-float wrapper types.",
    fingerprint = "
        item: any,
        body_contains: [
            'binop: < > <= >= on f32 or f64',
            'no preceding is_nan() check on operands'
        ]
    ",
    references = [
        "https://doc.rust-lang.org/std/primitive.f64.html#method.partial_cmp",
        "https://docs.rs/ordered-float/",
    ],
)]
pub struct NaNComparisonTrap;
```

**Witness mechanism**: delegates to a custom dylint OR a phantom-type wrapper
(NotNan<f64> from ordered-float).

**Why ship in stdlib**: numeric code in scientific computing, finance, ML, and games
all hit this. clippy doesn't catch it (it's not a syntactic pattern; it's a domain
error). Antigen-native value.

---

## Antigen 5: `lock-order-inversion`

**Family**: `incompatible-merger`
**Source pattern**: classic deadlock pattern; harder to detect statically.

```rust
#[antigen(
    name = "lock-order-inversion",
    family = "incompatible-merger",
    summary = "Acquiring multiple locks in inconsistent order across code paths \
               causes deadlock. Establish a project-wide lock ordering (often by \
               address or by name) and document it. Witness: ML-style proof or \
               runtime detection via tokio-console / lock_api hooks.",
    fingerprint = "
        item: any,
        body_contains: '2+ Mutex::lock() or RwLock::write() calls without explicit \
                        ordering convention'
    ",
    references = [
        "https://en.wikipedia.org/wiki/Deadlock_(computer_science)#Avoidance",
        "https://github.com/tokio-rs/tokio/blob/master/tokio/src/sync/mutex.rs",
    ],
)]
pub struct LockOrderInversion;
```

**Witness mechanism**: project-defined. Common patterns: phantom-type lock-order
proof; runtime detection via parking_lot's `deadlock_detection` feature; integration
test that exercises all lock-ordering paths.

**Why ship in stdlib**: deadlock bugs are catastrophic when they occur; existing
ecosystem detection is weak. Antigen's role here is NAMING the failure class so
projects can declare their lock ordering policy structurally.

---

## Antigen 6: `silent-truncation-on-cast`

**Family**: `boundary-violation`
**Source pattern**: `as` cast that loses bits silently; clippy partially covers.

```rust
#[antigen(
    name = "silent-truncation-on-cast",
    family = "boundary-violation",
    summary = "as-casts between integer types of different sizes silently truncate \
               or sign-extend. For loss-of-precision-sensitive code, use TryFrom or \
               explicit min/max clamps. clippy::cast_possible_truncation covers most \
               cases; this antigen names the failure class explicitly.",
    fingerprint = "
        item: any,
        body_contains: [
            'as_cast between integer types of different bit widths',
            'no guard on input range'
        ]
    ",
    references = [
        "https://rust-lang.github.io/rust-clippy/master/index.html#cast_possible_truncation",
        "https://rust-lang.github.io/rust-clippy/master/index.html#cast_lossless",
    ],
)]
pub struct SilentTruncationOnCast;
```

**Witness mechanism**: delegates to clippy `cast_possible_truncation` and
`cast_lossless` lints.

**Why ship in stdlib**: numeric code, networking, file-format parsers all hit this.
clippy coverage is mature; antigen makes the failure class first-class so projects
can say "we are immune to this" structurally.

---

## Antigen 7: `hash-collision-iteration-order`

**Family**: `implicit-coupling`
**Source pattern**: HashMap iteration order assumed stable across runs; broken by
RandomState; broken differently between Rust versions.

```rust
#[antigen(
    name = "hash-collision-iteration-order",
    family = "implicit-coupling",
    summary = "Iteration over HashMap, HashSet, or HashBrown produces order that is \
               NOT stable across runs (RandomState seeds vary), NOT stable across \
               Rust versions, and NOT stable under hash-collision changes. Code \
               that depends on iteration order — especially serialization, \
               diff-comparison, or display ordering — must use BTreeMap, IndexMap, \
               or explicit sort.",
    fingerprint = "
        item: any,
        body_contains: [
            'HashMap::iter() or HashSet::iter() result feeding into',
            'serialization, comparison, or display'
        ]
    ",
    references = [
        "https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.iter",
        "https://github.com/rust-lang/rust/issues/...",
    ],
)]
pub struct HashCollisionIterationOrder;
```

**Witness mechanism**: project-defined; usually a proptest that runs the iteration
multiple times with shuffled keys and verifies the output property holds regardless
of iteration order.

**Why ship in stdlib**: countless real bugs from this; clippy doesn't catch most;
antigen-native value.

---

## Antigen 8: `untracked-default-impl-divergence`

**Family**: `forgotten-lesson`
**Source pattern**: trait default impl is overridden in some impls but not others;
divergent behavior surfaces under polymorphism.

```rust
#[antigen(
    name = "untracked-default-impl-divergence",
    family = "forgotten-lesson",
    summary = "Trait method has a default implementation. Some impls override it; \
               others don't. The DIFFERENCE in behavior between overriding and \
               non-overriding impls can be load-bearing. Document the default's \
               semantics; require either explicit override OR explicit \
               #[inherits_default] in every impl.",
    fingerprint = "
        item: trait,
        has_default_method: true,
        any_impl_overrides: true,
        any_impl_does_not_override: true
    ",
    references = [
        "https://rust-lang.github.io/api-guidelines/predictability.html",
    ],
)]
pub struct UntrackedDefaultImplDivergence;
```

**Witness mechanism**: project-defined; usually a property test that exercises both
the default-implementing impl and an overriding impl with the same input and asserts
the documented relationship between their outputs.

**Why ship in stdlib**: trait default impls are everywhere in idiomatic Rust;
divergence-driven bugs are common; antigen makes the failure class addressable.

---

## Antigen 9: `subtle-pub-use-semver-break`

**Family**: `optionality-collapse`
**Source pattern**: `pub use` re-exports widely; changes in upstream type signatures
silently propagate; semver breaks in dependents that don't realize they're exposed.

```rust
#[antigen(
    name = "subtle-pub-use-semver-break",
    family = "optionality-collapse",
    summary = "pub use re-exports an item from a private dependency. When the upstream \
               item changes (signature, generics, trait bounds), it silently affects \
               consumers of this crate's public API. Either commit to the upstream's \
               semver guarantees explicitly OR re-wrap rather than re-export.",
    fingerprint = "
        item: use_statement,
        is_pub: true,
        path_root_is_external_crate: true,
        not_self_authored: true
    ",
    references = [
        "https://github.com/obi1kenobi/cargo-semver-checks",
        "https://semver.org/",
    ],
)]
pub struct SubtlePubUseSemverBreak;
```

**Witness mechanism**: delegates to `cargo-semver-checks`. The antigen makes the
failure class explicit; the witness verifies semver hasn't broken via the existing
tool.

**Why ship in stdlib**: this is a major source of accidental breaking changes in
the Rust ecosystem; cargo-semver-checks is mature; antigen wraps it under a named
failure class.

---

## Antigen 10: `optional-dependency-implicit-feature`

**Family**: `implicit-coupling`
**Source pattern**: optional dependencies in Cargo.toml create implicit features
with the same name; downstream crates enable them inadvertently.

```rust
#[antigen(
    name = "optional-dependency-implicit-feature",
    family = "implicit-coupling",
    summary = "An optional dependency in Cargo.toml automatically creates a feature \
               flag with its name. Adding such a dependency without explicit feature \
               declaration creates a public-API surface that downstream crates may \
               depend on. Use 'dep:foo' explicit syntax (Rust 1.60+) and declare \
               'foo' as a named feature with explicit dependencies.",
    fingerprint = "
        cargo_toml_pattern: 'optional = true on [dependencies] without dep: prefix \
                              and matching [features] entry'
    ",
    references = [
        "https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies",
        "https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html",
    ],
)]
pub struct OptionalDependencyImplicitFeature;
```

**Witness mechanism**: cargo-antigen scan extends to Cargo.toml inspection (a
project-level fingerprint, not a code-level one). Witness: explicit `dep:` syntax
and matching `[features]` entries.

**Why ship in stdlib**: this is a known footgun for ecosystem maintainers; the
Rust 1.60+ syntax is the documented fix; antigen names the class so projects can
declare their cargo-feature hygiene structurally.

---

## Coverage across the 8 failure classes

| Class | Stdlib seed antigen(s) |
|-------|------------------------|
| Frame-translation | 1 (polarity-inverted-class-meet), 4 (nan-comparison-trap) |
| Forgotten-lesson | 8 (untracked-default-impl-divergence) |
| Implicit-coupling | 7 (hash-collision-iteration-order), 10 (optional-dependency-implicit-feature) |
| Stale-context | (DEFERRED — no clean stdlib pattern; antigen-native; team to populate) |
| Premature-abstraction | (DEFERRED — no clean stdlib pattern; team to populate) |
| Incompatible-merger | 3 (mutexguard-cross-await-point), 5 (lock-order-inversion) |
| Boundary-violation | 2 (panicking-in-drop), 6 (silent-truncation-on-cast) |
| Optionality-collapse | 9 (subtle-pub-use-semver-break) |

Note: stale-context and premature-abstraction don't have clean v0.1 stdlib instances
because their failure shapes are hard to express as syntactic fingerprints. They're
**antigen-native territory** (per `ecosystem-composition.md`) and the antigen team
will design v0.2 instances during sweep A5 with potentially novel fingerprint
mechanisms.

This is honest — the v0.1 stdlib doesn't claim universal coverage; it ships the
patterns where the witness ecosystem is mature and the fingerprints are clean.
v0.2+ extends to the harder territory.

---

## How to use this catalog

Each antigen above can be:
- **Used as a starting point** for the eventual `antigen-stdlib` crate's source code
- **Refined during sweep A5** by the antigen JBD team — fingerprint precision,
  witness mechanism details, references curation
- **Tested against real codebases** to verify the fingerprint matches genuine
  vulnerable sites without false positives
- **Documented in `antigen-stdlib`'s README** as the v0.1 catalog

The pseudocode shown is **intentionally aspirational**. The actual fingerprint
grammar (per ADR-010) will refine. The actual macro syntax may differ. The shape
should remain.

---

## What this catalog DOES NOT cover

This is v0.1 seed. The eventual `antigen-stdlib` should grow to 50-100 named antigens
covering:

- **Lifetime-related footguns** (incorrect bounds, hidden 'static requirements)
- **Send/Sync auto-trait surprises**
- **Async runtime mismatches** (block_on inside async, etc.)
- **Proc-macro hygiene failures**
- **Format string injection** (especially for log targets)
- **Panicking default impls** beyond Drop
- **Integer overflow in arithmetic** (clippy::arithmetic_side_effects)
- **Implicit unwrap chains** (unwrap_or vs ok_or vs map_err patterns)
- **Cycle in Rc / Arc** (memory leak; needs Weak)
- **Cargo-feature additivity violations**
- **Cross-platform behavior divergence** (file paths, line endings, time zones)
- **Test-only dependencies leaking into production**
- **`unsafe` block scoping that's too broad**
- **Drop order assumptions in struct fields**
- **Slice indexing without bounds check**
- **Display impl that's also accidentally Debug**
- **Default impl that allocates**

Each of these can become an antigen with proper fingerprint + witness. Many will be
contributed by the broader Rust community as antigen adoption grows. The seed
catalog of 10 is enough to ship v0.1 with real value; community contributions
expand it from there.

---

## References

- [`docs/expedition/failure-class-instances.md`](failure-class-instances.md) — 36 real-world Rust failures from research
- [`docs/expedition/ecosystem-composition.md`](ecosystem-composition.md) — composition opportunities with existing tools
- [`docs/decisions.md` ADR-002](../decisions.md#adr-002--compose-dont-compete) — compose-don't-compete (witness delegation)
- [`docs/decisions.md` ADR-009](../decisions.md#adr-009--adoption-gradient-antigen-meets-consumers-at-any-discipline-level) — adoption gradient
- [`docs/decisions.md` ADR-010](../decisions.md#adr-010--fingerprint-grammar-v1-syn-based-ast-visitor-pattern) — fingerprint grammar v1
- [`docs/expedition/case-study-determinism-class.md`](case-study-determinism-class.md) — full case study walkthrough

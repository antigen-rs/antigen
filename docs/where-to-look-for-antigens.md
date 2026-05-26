# Where to look for antigens

> Structural conventions for locating antigen declarations in a Rust project.
> These aren't enforced by the tooling (yet) — they're the conventions that
> emerge from the first real adoption and that make a project's antigen
> vocabulary navigable.

---

## The short answer

| What | Where |
|---|---|
| Antigen declarations (`#[antigen]`) | `src/antigens.rs` (preferred) or inline at declaration site |
| Presentations (`#[presents]`) | At the vulnerable site, wherever that lives in the source tree |
| Immunity claims (`#[immune]`) | At the site being defended — typically co-located with `#[presents]` or at the test |
| Tolerances (`#[antigen_tolerance]`) | At the matching site, wherever the scan found it |
| Inherited from `antigen-stdlib` | Declared in `antigen-stdlib`; `#[presents]` and `#[immune]` live in your code |

---

## Where a marker can physically go (placement rules)

Before *which* file: there's a hard constraint on *which Rust position* a marker
can sit on. `#[antigen]`, `#[presents]`, `#[immune]`, `#[antigen_tolerance]`, and
`#[descended_from]` are **attribute proc-macros**, and Rust only allows
attribute-macro invocations on **item-level** positions. The matrix
(each row `cargo build`-verified):

| Position | `#[presents]` / `#[immune]` / … | Why |
|---|---|---|
| `struct` / `enum` (the **type**) | ✅ compiles, scans, docs-clean | item position |
| `fn` | ✅ compiles | item position |
| `impl` block | ✅ compiles | item position |
| **enum variant** (`Foo::Bar`) | ❌ **compile error** | proc-macro attrs forbidden on variants |
| **struct field** | ❌ **compile error** | proc-macro attrs forbidden on fields |

Putting a marker on a variant or field produces, at `cargo build`:

```
error: expected non-macro attribute, found attribute macro `presents`
```

Only *inert* attributes (`#[cfg]`, `#[deprecated]`, `#[doc]`) are allowed on
variants and fields — proc-macro attributes are not. So:

- **Mark the enclosing type.** A failure-class that lives in one enum variant or
  one struct field is marked on the whole `enum` / `struct`. If you need to point
  at the specific variant/field, say so in the antigen's `summary` or the doc
  comment, not with a marker on the variant/field itself.
- **Or use fingerprint-scan for sub-type granularity.** A fingerprint can match
  at finer granularity than a marker can be placed (the scanner reads variant and
  field syntax even though a *marker* can't compile there). When you need
  variant/field-level recognition, lean on the fingerprint, not a direct marker.

> **The trap worth knowing** (it has bitten this project's own contributors):
> `cargo antigen scan` *can read* a `#[presents]` written on an enum variant in a
> parse-only test fixture — the scanner parses source via `syn`, which accepts the
> attribute *syntax*. That does **not** mean the marker compiles. `rustc` rejects
> it at attribute resolution. **Scanner-reads-it ≠ it-compiles.** A green scanner
> test on a fixture is not evidence a real adopter can use the marker; only
> `cargo build` on a real crate is. If you're deciding "can a marker go here?",
> compile a minimal crate — don't trust a parse/scan test.

### Foundation crates can't carry markers at all

Crates *upstream* of `antigen` in the dependency graph — for antigen's own
workspace, that's `antigen-macros` and `antigen-fingerprint` — cannot depend on
`antigen` (it would be a dependency cycle), so the marker macros aren't even in
scope there. A failure-class that lives in such a foundation crate is defended by
**fingerprint-scan + structural evidence** (a test that pins the invariant),
recognized via the antigen's `summary` / doc-comment rather than an in-situ
marker. Antigen's own `BiologyGroundingClaimDrift` (defended fingerprint-only) is
the worked example of this pattern.

---

## Antigen declarations: `src/antigens.rs`

The canonical location for a crate's own antigen declarations is a dedicated
`antigens.rs` module at the crate root:

```
my-crate/
├── src/
│   ├── lib.rs         ← mod antigens;
│   ├── antigens.rs    ← all #[antigen] declarations live here
│   ├── foo.rs
│   └── bar.rs
```

Declare the module from `lib.rs`:

```rust
// src/lib.rs
pub mod antigens;
```

Make it public. The failure-class vocabulary is part of the crate's public
contract — downstream crates that re-use your types may need to present or
claim immunity against your antigens. Hiding them limits structural inheritance.

### What goes in `antigens.rs`

Each antigen declaration is a public unit struct with the `#[antigen]` attribute.
The struct itself carries no data — it's a zero-cost vocabulary token:

```rust
// src/antigens.rs

use antigen::antigen;

/// Frame-translation failure at the class-meet boundary.
/// [documentation: see summary and references in the attribute]
#[antigen(
    name = "polarity-inverted-class-meet",
    family = "frame-translation",
    fingerprint = r#"item = enum, name = matches("*Class"), has_method("meet", "(self, Self) -> Self")"#,
    summary = "Class enums with strongest-first discriminants must use max (not \
               min) for lattice meet. ...",
    references = ["issue-123", "https://..."],
)]
pub struct PolarityInvertedClassMeet;
```

The doc comment on the struct is visible to `rustdoc` and to IDE hover — write
it to explain the failure-class in human terms. The `summary` field in the
attribute is the machine-readable version used by `cargo antigen audit`.

### When to keep declarations inline instead

For a small utility crate with a single, highly localized failure-class, an
inline declaration (on the module or type that owns the failure-class) is
acceptable:

```rust
// src/state_machine.rs

#[antigen(
    name = "invalid-transition-panic",
    fingerprint = r#"item = fn, attr_present("transition")"#,
    summary = "Transition functions must return Result, not panic on invalid state.",
)]
pub struct InvalidTransitionPanic;
```

The tradeoff: inline declarations are harder for downstream crates to discover
via a single import path. Once a project has more than two or three antigens,
the dedicated module pays for itself in navigability.

---

## Presentations: at the vulnerable site

`#[presents]` lives at the code location that is vulnerable to the named
failure-class. That's wherever the vulnerability actually is:

```rust
// src/kernel.rs — the vulnerable site is this struct
#[presents(KernelReconstructionDivergence)]
pub struct ExpKernelState { ... }
```

```rust
// tests/consistency.rs — for composition-boundary antigens, the test is the site
#[presents(KernelReconstructionDivergence)]
#[proptest]
fn kernel_vs_standalone_agrees(input: f64) { ... }
```

```rust
// src/drop_impl.rs — Drop impl with panic-capable operations
#[presents(PanickingInDrop)]
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        self.cleanup().expect("cleanup failed");  // #[presents] signals this is flagged
    }
}
```

**`cargo antigen scan` finds presentations, not declarations.** Scan walks the
source tree and reports every `#[presents]` site alongside its immunity status.
Declarations (in `antigens.rs`) are the vocabulary; presentations are the
annotations that say "this vocabulary applies here."

### Presentations you didn't add

Some presentations are added by `cargo antigen vaccinate`, which bulk-applies
a presentation to all sites matching an antigen's fingerprint. These look
identical to hand-written presentations — there's no mechanical distinction
in the source. They appear wherever the fingerprint matched.

If you're reading code and see a `#[presents]` you don't recognize, check
`src/antigens.rs` (or the antigen-stdlib) for the declaration that defines it.
The `#[antigen]` attribute's `summary` field explains what the fingerprint
detected.

---

## Immunity: co-located with the defense

`#[immune]` goes at the site that carries the defense — usually the test,
proptest, or type-system witness that actually verifies the failure-class
doesn't obtain:

```rust
// The test is both the presentation site and the immunity site
#[immune(KernelReconstructionDivergence,
    witness = proptest::kernel_vs_standalone_agrees)]
#[presents(KernelReconstructionDivergence)]
#[proptest]
fn kernel_vs_standalone_agrees(input: f64) { ... }
```

For phantom-type witnesses, the immunity declaration goes at the type site:

```rust
#[immune(FrameTranslationDrift,
    witness = phantom::LatticeFrameInvariant)]
pub struct DeterminismClass { ... }
```

The rule of thumb: `#[immune]` goes where you would tell a reviewer to look
for evidence that the failure-class has been handled. The witness field names
the exact evidence.

---

## Tolerances: at the matching site

`#[antigen_tolerance]` lives at whichever site the scan flagged. There's no
preferred module for tolerances — they belong with the code that's being
excepted:

```rust
// The fingerprint matched because this is the test that CONSTRUCTS the pattern
#[antigen_tolerance(
    PolarityInvertedClassMeet,
    rationale = "Test fixture that deliberately constructs the inverted-polarity \
                 case to verify the fingerprint catches it. Vulnerability is the point."
)]
fn test_fingerprint_detects_inverted_meet() { ... }
```

See [`docs/usage-patterns.md`](usage-patterns.md) for the decision tree on
when `#[antigen_tolerance]` is appropriate vs. writing a witness and claiming
`#[immune]`.

---

## Antigens from `antigen-stdlib`

When `antigen-stdlib` ships, it will provide ready-made antigens for common
Rust ecosystem failure patterns. You don't declare these — you import them
and add `#[presents]` or `#[immune]` at your code sites:

```rust
// Cargo.toml
[dependencies]
antigen = "0.x"  // antigen-stdlib included via `stdlib` feature flag

// src/resource.rs
use antigen_stdlib::PanickingInDrop;

#[presents(PanickingInDrop)]
impl Drop for ResourceHandle { ... }
```

The declarations live in `antigen-stdlib`; your code contributes only the
site annotations. `cargo antigen scan` sees both the stdlib antigen and your
`#[presents]` annotation and reports accordingly.

**Fingerprint-based passive detection** (the scanner finding pattern matches
without explicit `#[presents]`) works the same way with stdlib antigens — the
fingerprint in the antigen declaration drives passive scanning regardless of
where the declaration lives.

---

## Large projects: multiple antigens modules

For a crate with many failure-classes, or for a project that has developed
multiple domain-specific antigens across distinct modules, a directory-based
layout can help:

```
my-crate/
├── src/
│   ├── lib.rs
│   ├── antigens/
│   │   ├── mod.rs         ← pub mod frame_translation; pub mod boundary; ...
│   │   ├── frame_translation.rs
│   │   ├── boundary.rs
│   │   └── coupling.rs
│   └── ...
```

This is mostly a navigability choice — the tooling doesn't care about module
structure, only about which types carry `#[antigen]`. Group by failure-class
family (matching the `family` field in the declarations) for coherence.

---

## What `cargo antigen scan` looks for

To orient yourself when exploring a project:

1. **Find declarations**: `grep -r "#\[antigen(" src/` or look in `src/antigens.rs`
2. **Find presentations**: `grep -r "#\[presents(" src/ tests/`
3. **Find immunity claims**: `grep -r "#\[immune(" src/ tests/`
4. **Full picture**: `cargo antigen audit` — gives declarations, presentation
   coverage, immunity status, and tolerance inventory across the workspace

The audit output is the navigational surface — it tells you what's declared,
what's been annotated, and what's addressed. The source tree's file structure
tells you *where* those annotations live.

---

## References

- [`docs/macros.md`](macros.md) — full reference for all five attribute macros
  with attribute syntax, examples, and field-by-field documentation
- [`docs/decisions.md`](decisions.md) — ADR-004 (implicit-to-explicit elevation),
  ADR-009 (adoption gradient: Layer 1 through 4)
- [`docs/usage-patterns.md`](usage-patterns.md) — when/how to apply each macro
- [`docs/testing-patterns.md`](testing-patterns.md) — witness conventions and
  test placement
- [`docs/glossary.md`](glossary.md) — antigen, presentation, immunity, tolerance,
  witness, vaccination
